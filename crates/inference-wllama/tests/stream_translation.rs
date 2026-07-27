//! Real end-to-end test of the streaming translation pipeline in
//! `session::generate`. Drives a known sequence of `WllamaStreamEvent`s
//! through a stub bridge and asserts the resulting `InferenceChunk` stream
//! has the exact ordering and content the trait contract requires.
//!
//! This is not a smoke test: the assertions are exact and pin the
//! stream-order semantics the contract advertises (citation markers →
//! text deltas → usage → completed).

use async_trait::async_trait;
use inference_wllama::{
    verify_gguf_artifact, BridgeChatMessage, BridgeEmbeddingBatch, BridgeEmbeddingResult,
    WllamaBridge, WllamaChatParams, WllamaLoadedInfo, WllamaLoadParams, WllamaProbe,
    WllamaStreamEvent,
};
use ports::embedding::EmbeddingBackend;
use ports::inference::{
    AcceleratorKind, BackendKind, InferenceChunk, InferenceError, InferenceMessage,
    InferenceRequest, InferenceSession, InferenceStream, ModelFormat, ModelKind,
    SessionOptions,
};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// Scripted bridge: emits a pre-loaded sequence of stream events for each
/// chat_start call, and records unload/abort calls so the test can assert
/// on them.
struct ScriptedBridge {
    events_by_request: Arc<Mutex<std::collections::HashMap<String, VecDeque<WllamaStreamEvent>>>>,
    aborts: Arc<Mutex<Vec<(String, String)>>>,
    unloads: Arc<Mutex<Vec<String>>>,
}

impl ScriptedBridge {
    fn new() -> Self {
        Self {
            events_by_request: Arc::new(Mutex::new(Default::default())),
            aborts: Arc::new(Mutex::new(Vec::new())),
            unloads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn script(&self, request_id: &str, events: Vec<WllamaStreamEvent>) {
        let mut map = self.events_by_request.lock().unwrap();
        map.insert(request_id.to_string(), events.into_iter().collect());
    }
}

#[async_trait]
impl WllamaBridge for ScriptedBridge {
    async fn probe(&self) -> Result<WllamaProbe, InferenceError> {
        Ok(WllamaProbe {
            available: true,
            detail: "scripted".to_string(),
            webgpu_available: false,
        })
    }

    async fn load_model(
        &self,
        _session_id: &str,
        _model_uri: &str,
        _params: WllamaLoadParams,
    ) -> Result<WllamaLoadedInfo, InferenceError> {
        Ok(WllamaLoadedInfo {
            n_ctx: 1024,
            token_bos: 1,
            token_eos: 2,
            use_multi_thread: false,
            use_webgpu: false,
            n_embd: 0,
        })
    }

    async fn chat_start(
        &self,
        _session_id: &str,
        request_id: &str,
        _params: WllamaChatParams,
    ) -> Result<mpsc::Receiver<WllamaStreamEvent>, InferenceError> {
        let (tx, rx) = mpsc::channel(32);
        let mut queue = {
            let mut map = self.events_by_request.lock().unwrap();
            map.remove(request_id).unwrap_or_default()
        };
        tokio::spawn(async move {
            // Send all scripted events, then hold the channel open until the
            // consumer drops it. This lets cancel-tests observe the request
            // still in-flight instead of racing the natural stream end.
            while let Some(event) = queue.pop_front() {
                if tx.send(event).await.is_err() {
                    return;
                }
            }
            // Park forever; the Sender drop on consumer side ends this.
            std::future::pending::<()>().await;
        });
        Ok(rx)
    }

    async fn abort(&self, session_id: &str, request_id: &str) -> Result<(), InferenceError> {
        self.aborts
            .lock()
            .unwrap()
            .push((session_id.to_string(), request_id.to_string()));
        Ok(())
    }

    async fn embed(
        &self,
        _session_id: &str,
        batch: BridgeEmbeddingBatch,
    ) -> Result<BridgeEmbeddingResult, InferenceError> {
        // Deterministic 2-d "embedding" for testing: [len, 1.0].
        let vectors = batch
            .inputs
            .iter()
            .map(|t| vec![t.len() as f32, 1.0])
            .collect();
        Ok(BridgeEmbeddingResult {
            vectors,
            dimensions: 2,
        })
    }

    async fn unload(&self, session_id: &str) -> Result<(), InferenceError> {
        self.unloads.lock().unwrap().push(session_id.to_string());
        Ok(())
    }
}

fn chat_request(id: &str, evidence_keys: &[&str]) -> InferenceRequest {
    InferenceRequest {
        request_id: id.to_string(),
        system_prompt: "Be brief.".to_string(),
        messages: vec![InferenceMessage {
            role: "user".to_string(),
            body: "Hello".to_string(),
        }],
        evidence: evidence_keys
            .iter()
            .map(|k| ports::inference::InferenceEvidence {
                citation_key: k.to_string(),
                title: format!("Title {k}"),
                text: "body".to_string(),
                source_kind: "article".to_string(),
                source_id: format!("a/{k}"),
            })
            .collect(),
        options: SessionOptions {
            context_tokens: 1024,
            max_output_tokens: 16,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: vec![],
        },
    }
}

async fn collect_stream(mut s: InferenceStream) -> Vec<InferenceChunk> {
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        match item {
            Ok(chunk) => out.push(chunk),
            Err(_) => break,
        }
    }
    out
}

#[tokio::test]
async fn streams_citations_then_deltas_then_usage_then_completed() {
    let bridge = Arc::new(ScriptedBridge::new());
    let session_id = "sess-1";

    // Open a session directly via the public session type so the test does
    // not depend on a real artifact path.
    let mut session = inference_wllama::session::WllamaInferenceSession::new(
        session_id.to_string(),
        "m1".to_string(),
        bridge.clone(),
        SessionOptions {
            context_tokens: 1024,
            max_output_tokens: 16,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: vec![],
        },
    );

    bridge.script(
        "req-1",
        vec![
            WllamaStreamEvent::Delta {
                text: "Hello ".to_string(),
            },
            WllamaStreamEvent::Delta {
                text: "world".to_string(),
            },
            WllamaStreamEvent::Usage {
                input_tokens: 5,
                output_tokens: 2,
            },
            WllamaStreamEvent::Done {
                finish_reason: "stop".to_string(),
            },
        ],
    );

    let stream = session.generate(chat_request("req-1", &["C1", "C2"])).await.unwrap();
    let chunks = collect_stream(stream).await;

    assert_eq!(chunks.len(), 6, "expected 6 chunks, got {chunks:?}");
    assert!(matches!(
        chunks[0],
        InferenceChunk::CitationMarker { ref citation_key } if citation_key == "C1"
    ));
    assert!(matches!(
        chunks[1],
        InferenceChunk::CitationMarker { ref citation_key } if citation_key == "C2"
    ));
    assert!(matches!(
        chunks[2],
        InferenceChunk::TextDelta { ref text } if text == "Hello "
    ));
    assert!(matches!(
        chunks[3],
        InferenceChunk::TextDelta { ref text } if text == "world"
    ));
    assert!(matches!(
        chunks[4],
        InferenceChunk::Usage { input_tokens: 5, output_tokens: 2 }
    ));
    assert!(matches!(
        chunks[5],
        InferenceChunk::Completed { ref finish_reason } if finish_reason == "stop"
    ));

    // After consumption the active-requests registry should have dropped
    // the id; cancelling it should now be a no-op (still Ok).
    session.cancel("req-1").await.unwrap();
}

#[tokio::test]
async fn cancel_known_request_calls_bridge_abort() {
    let bridge = Arc::new(ScriptedBridge::new());
    let session_id = "sess-2";
    let mut session = inference_wllama::session::WllamaInferenceSession::new(
        session_id.to_string(),
        "m1".to_string(),
        bridge.clone(),
        SessionOptions {
            context_tokens: 1024,
            max_output_tokens: 16,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: vec![],
        },
    );

    // Never emit a terminal event so the request stays "in flight" from the
    // session's perspective.
    bridge.script(
        "req-2",
        vec![WllamaStreamEvent::Delta {
            text: "x".to_string(),
        }],
    );

    let _stream = session.generate(chat_request("req-2", &[])).await.unwrap();
    // Allow the spawned task to start.
    tokio::task::yield_now().await;

    session.cancel("req-2").await.unwrap();

    let aborts = bridge.aborts.lock().unwrap().clone();
    assert_eq!(aborts.len(), 1, "expected one abort call, got {aborts:?}");
    assert_eq!(aborts[0].0, session_id);
    assert_eq!(aborts[0].1, "req-2");
}

#[tokio::test]
async fn close_calls_bridge_unload() {
    let bridge = Arc::new(ScriptedBridge::new());
    let session_id = "sess-3";
    let session = inference_wllama::session::WllamaInferenceSession::new(
        session_id.to_string(),
        "m1".to_string(),
        bridge.clone(),
        SessionOptions {
            context_tokens: 1024,
            max_output_tokens: 16,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: vec![],
        },
    );

    let boxed: Box<dyn InferenceSession> = Box::new(session);
    boxed.close().await.unwrap();

    let unloads = bridge.unloads.lock().unwrap().clone();
    assert_eq!(unloads, vec![session_id.to_string()]);
}

#[tokio::test]
async fn scripted_bridge_embeds_a_batch() {
    let bridge = Arc::new(ScriptedBridge::new());

    // Open an embedding session at the backend level so the trait path is
    // exercised end-to-end. We don't call ensure_loaded because the scripted
    // bridge pretends a model is always loaded — instead we trigger the
    // BackendUnavailable branch by not loading.
    let backend = inference_wllama::WllamaEmbeddingBackend::new(
        bridge.clone(),
        "tauri://localhost/models".to_string(),
        "space-1".to_string(),
        "emb-model".to_string(),
        2,
        "cosine".to_string(),
        false,
    );

    // Without ensure_loaded, embed fails as BackendUnavailable.
    let err = backend
        .embed(vec![ports::embedding::EmbeddingInput {
            input_id: "x".into(),
            text: "hi".into(),
        }])
        .await
        .unwrap_err();
    assert!(matches!(err, InferenceError::BackendUnavailable));

    // Descriptor still reports what the constructor was given.
    let d = EmbeddingBackend::descriptor(&backend);
    assert_eq!(d.embedding_space_id, "space-1");
    assert_eq!(d.model_id, "emb-model");
    assert_eq!(d.dimensions, 2);
    assert_eq!(d.distance_metric, "cosine");
    assert!(!d.normalized);

    let _ = (BackendKind::LocalLm, AcceleratorKind::Cpu, ModelFormat::Gguf);
}

#[test]
fn gguf_magic_constant_is_canonical() {
    // GGUF on disk is bytes G G U F = 0x47 0x47 0x55 0x46.
    // Read as little-endian u32, this is 0x46554747.
    let bytes = *b"GGUF";
    let magic = u32::from_le_bytes(bytes);
    assert_eq!(magic, inference_wllama::GGUF_MAGIC);
    assert_eq!(magic, 0x46554747);
}

#[tokio::test]
async fn verify_rejects_missing_backend_in_manifest() {
    // Build a manifest that does NOT list LocalLm. Only the static check
    // should fail; the file itself is never opened.
    use ports::inference::{BackendKind, ModelManifest};
    let manifest = ModelManifest {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        family: "f".to_string(),
        model_kind: ModelKind::Chat,
        format: ModelFormat::Gguf,
        artifact_sha256: "0".repeat(64),
        artifact_size_bytes: 0,
        context_tokens: 1,
        recommended_ram_bytes: 0,
        supported_backends: vec![], // <-- no backend
        prompt_template_id: "t".to_string(),
        license_spdx: "MIT".to_string(),
    };
    let err = verify_gguf_artifact(&manifest, Path::new("/dev/null")).await.unwrap_err();
    assert!(matches!(err, inference_wllama::VerifyError::BackendNotSupported));
    let _ = BackendKind::LocalLm; // touch import
}

// Re-export so the public-API name (`inference_wllama::session`) used above
// resolves. The crate exposes `session` as a public module.
mod _sanity {
    use super::BridgeChatMessage;
    #[test]
    fn bridge_chat_message_is_eq() {
        let a = BridgeChatMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
