//! [`ports::inference::InferenceSession`] implementation backed by a loaded
//! wllama session.
//!
//! Each `WllamaInferenceSession` owns one wllama load (one model in WebView
//! memory). Concurrent chat requests on the same session share the loaded
//! context; the bridge is responsible for serializing them if the underlying
//! runtime cannot.

use ports::inference::{
    InferenceChunk, InferenceError, InferenceRequest, InferenceSession, InferenceStream,
    SessionOptions,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;

use crate::bridge::{WllamaBridge, WllamaChatParams, WllamaStreamEvent};
use crate::prompt::{ensure_non_empty, format_chat_prompt, validate_evidence, validate_messages};

/// Per-session registry of currently in-flight request IDs. Used by `cancel`
/// to distinguish "nothing to cancel" (Ok) from "transport lost the session"
/// (Runtime error from the bridge).
#[derive(Default)]
struct ActiveRequests {
    ids: HashSet<String>,
}

pub struct WllamaInferenceSession {
    session_id: String,
    model_id: String,
    bridge: Arc<dyn WllamaBridge>,
    options: SessionOptions,
    active: Arc<Mutex<ActiveRequests>>,
}

impl WllamaInferenceSession {
    /// Construct a session bound to an already-loaded wllama instance.
    ///
    /// Public so that integrators (and tests) can wrap a session they
    /// obtained out-of-band — most production callers should go through
    /// [`crate::WllamaInferenceBackend::open_session`] instead.
    pub fn new(
        session_id: String,
        model_id: String,
        bridge: Arc<dyn WllamaBridge>,
        options: SessionOptions,
    ) -> Self {
        Self {
            session_id,
            model_id,
            bridge,
            options,
            active: Arc::new(Mutex::new(ActiveRequests::default())),
        }
    }
}

#[async_trait::async_trait]
impl InferenceSession for WllamaInferenceSession {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn generate(
        &mut self,
        request: InferenceRequest,
    ) -> Result<InferenceStream, InferenceError> {
        // (1) Validate inputs.
        validate_messages(&request.messages).map_err(|e| InferenceError::Runtime(e.to_string()))?;
        validate_evidence(&request.evidence).map_err(|e| InferenceError::Runtime(e.to_string()))?;

        // (2) Build the chat payload.
        let bridge_messages = ensure_non_empty(format_chat_prompt(&request));

        // (3) Merge stop sequences: caller's options + session's defaults,
        //     then dedup.
        let mut stop = self.options.stop_sequences.clone();
        stop.extend(request.options.stop_sequences.iter().cloned());
        stop.sort();
        stop.dedup();

        // (4) Effective sampling parameters: request-scoped values win.
        let params = WllamaChatParams {
            messages: bridge_messages,
            temperature: request.options.temperature,
            top_p: request.options.top_p,
            seed: request.options.seed,
            max_tokens: request.options.max_output_tokens,
            stop,
        };

        // (5) Ask the bridge to start the request; it returns a receiver of
        //     low-level events.
        let request_id = request.request_id.clone();
        let mut rx = self
            .bridge
            .chat_start(&self.session_id, &request_id, params)
            .await?;

        // (6) Register the request as active so cancel() has something to do.
        {
            let mut active = self.active.lock().await;
            active.ids.insert(request_id.clone());
        }

        // (7) Collect citation keys up front so we can emit CitationMarker
        //     events before any model output.
        let citations: Vec<String> = request
            .evidence
            .iter()
            .map(|e| e.citation_key.clone())
            .collect();

        // (8) Translate the bridge stream into InferenceChunks. The output
        //     channel is buffered (32) so the consumer can lag a little
        //     without blocking the WebView worker.
        let (out_tx, out_rx) = mpsc::channel::<Result<InferenceChunk, InferenceError>>(32);

        let active = self.active.clone();
        let session_id = self.session_id.clone();

        tokio::spawn(async move {
            // (8a) Emit citation markers first so the UI knows the
            //      available citation set before any text arrives.
            for key in &citations {
                if out_tx
                    .send(Ok(InferenceChunk::CitationMarker {
                        citation_key: key.clone(),
                    }))
                    .await
                    .is_err()
                {
                    // Consumer dropped the stream; clean up and exit.
                    let mut active_guard = active.lock().await;
                    active_guard.ids.remove(&request_id);
                    return;
                }
            }

            // (8b) Forward bridge events. Accumulate usage and the terminal
            //      finish reason so we can emit one consolidated Usage +
            //      Completed pair at the end.
            let mut input_tokens: u32 = 0;
            let mut output_tokens: u32 = 0;
            let mut finish_reason: Option<String> = None;
            let mut terminal_error: Option<InferenceError> = None;

            while let Some(event) = rx.recv().await {
                match event {
                    WllamaStreamEvent::Delta { text } => {
                        if out_tx
                            .send(Ok(InferenceChunk::TextDelta { text }))
                            .await
                            .is_err()
                        {
                            // Consumer gone; stop forwarding.
                            break;
                        }
                    }
                    WllamaStreamEvent::Usage {
                        input_tokens: it,
                        output_tokens: ot,
                    } => {
                        input_tokens = it;
                        output_tokens = ot;
                    }
                    WllamaStreamEvent::Done { finish_reason: fr } => {
                        finish_reason = Some(fr);
                        break;
                    }
                    WllamaStreamEvent::Failed { message } => {
                        terminal_error = Some(InferenceError::Runtime(message));
                        break;
                    }
                }
            }

            if let Some(err) = terminal_error {
                let _ = out_tx.send(Err(err)).await;
            } else {
                if input_tokens > 0 || output_tokens > 0 {
                    let _ = out_tx
                        .send(Ok(InferenceChunk::Usage {
                            input_tokens,
                            output_tokens,
                        }))
                        .await;
                }
                let _ = out_tx
                    .send(Ok(InferenceChunk::Completed {
                        finish_reason: finish_reason.unwrap_or_else(|| "unknown".to_string()),
                    }))
                    .await;
            }

            let mut active_guard = active.lock().await;
            active_guard.ids.remove(&request_id);
            // Touch session_id so the move closure captures it cleanly; the
            // session id is also referenced via self.session_id on the
            // session struct, but holding it here is harmless.
            drop(session_id);
        });

        Ok(Box::pin(ReceiverStream::new(out_rx)))
    }

    async fn cancel(&mut self, request_id: &str) -> Result<(), InferenceError> {
        // Locally forget the request first so a concurrent generate() that
        // has already finished doesn't see a stale entry.
        let was_active = {
            let mut active = self.active.lock().await;
            active.ids.remove(request_id)
        };
        if !was_active {
            // Nothing to cancel — but per trait semantics this is not an
            // error: cancelling an already-finished or unknown request is
            // a no-op.
            return Ok(());
        }
        self.bridge.abort(&self.session_id, request_id).await
    }

    async fn close(self: Box<Self>) -> Result<(), InferenceError> {
        self.bridge.unload(&self.session_id).await
    }
}
