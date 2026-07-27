//! Build the chat messages for wllama. wllama applies its own jinja chat
//! template from the GGUF metadata; this crate's job is only to assemble
//! the messages in the right order, fold the system prompt and evidence
//! block in, normalize roles, and surface a degenerate `messages: []`
//! request as a single empty user turn so wllama never sees an empty array.
//!
//! `InferenceEvidence` is folded into the system prompt as a numbered
//! citation block. The companion `CitationMarker` events are emitted by
//! [`crate::session`] at the start of each stream so the UI can render the
//! available citations inline before the model output begins.

use ports::inference::{InferenceEvidence, InferenceMessage, InferenceRequest};
use std::collections::HashSet;

use crate::ipc::BridgeChatMessage;

const KNOWN_ROLES: [&str; 3] = ["system", "user", "assistant"];

/// Compose the chat messages for the bridge, in the order wllama expects
/// (system → conversation). The evidence block is appended to the request's
/// system prompt so the model sees citations before any user turn.
///
/// Returns an empty vector only if the request had no system prompt, no
/// evidence, and no messages — callers feed this through
/// [`ensure_non_empty`] before handing it to the bridge.
pub fn format_chat_prompt(request: &InferenceRequest) -> Vec<BridgeChatMessage> {
    let mut out = Vec::with_capacity(request.messages.len() + 1);

    let mut system = String::new();
    system.push_str(request.system_prompt.trim_end_matches('\n'));
    if !request.evidence.is_empty() {
        system.push_str("\n\nCited evidence:\n");
        for (i, ev) in request.evidence.iter().enumerate() {
            system.push_str(&format!(
                "[{}] ({}) {}\n  source: {}={}\n  {}\n\n",
                i + 1,
                ev.citation_key,
                ev.title,
                ev.source_kind,
                ev.source_id,
                ev.text.trim_end_matches('\n'),
            ));
        }
        system.push_str(
            "Cite evidence inline by writing [n] where n is the bracketed \
             number above. Do not invent sources.",
        );
    }
    if !system.is_empty() {
        out.push(BridgeChatMessage {
            role: "system".to_string(),
            content: system,
        });
    }

    let known: HashSet<&str> = KNOWN_ROLES.iter().copied().collect();
    for m in &request.messages {
        let role = if known.contains(m.role.as_str()) {
            m.role.clone()
        } else {
            "user".to_string()
        };
        out.push(BridgeChatMessage {
            role,
            content: m.body.clone(),
        });
    }

    out
}

/// Coalesce an empty message list into a single empty user turn, so wllama
/// never receives an empty `messages` array.
pub fn ensure_non_empty(messages: Vec<BridgeChatMessage>) -> Vec<BridgeChatMessage> {
    if messages.is_empty() {
        vec![BridgeChatMessage {
            role: "user".to_string(),
            content: String::new(),
        }]
    } else {
        messages
    }
}

/// Reject any message body containing a NUL byte — wllama's worker would
/// silently truncate it, which would change semantics for the caller.
pub fn validate_messages(messages: &[InferenceMessage]) -> Result<(), &'static str> {
    for m in messages {
        if m.body.contains('\u{0000}') {
            return Err("message body contains a NUL byte");
        }
    }
    Ok(())
}

/// Validate evidence text the same way: NUL bytes are a transport hazard
/// across the JS bridge.
pub fn validate_evidence(evidence: &[InferenceEvidence]) -> Result<(), &'static str> {
    for ev in evidence {
        if ev.text.contains('\u{0000}')
            || ev.title.contains('\u{0000}')
            || ev.citation_key.contains('\u{0000}')
        {
            return Err("evidence field contains a NUL byte");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::inference::{
        InferenceEvidence, InferenceMessage, InferenceRequest, SessionOptions,
    };

    fn opts() -> SessionOptions {
        SessionOptions {
            context_tokens: 1024,
            max_output_tokens: 64,
            temperature: 0.7,
            top_p: 0.9,
            seed: 1,
            stop_sequences: vec![],
        }
    }

    #[test]
    fn evidence_block_is_appended_to_system_prompt() {
        let req = InferenceRequest {
            request_id: "r1".into(),
            system_prompt: "You are careful.".into(),
            messages: vec![InferenceMessage {
                role: "user".into(),
                body: "Hi".into(),
            }],
            evidence: vec![InferenceEvidence {
                citation_key: "C1".into(),
                title: "T".into(),
                text: "E".into(),
                source_kind: "article".into(),
                source_id: "a/b".into(),
            }],
            options: opts(),
        };
        let out = format_chat_prompt(&req);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].role, "system");
        assert!(out[0].content.contains("You are careful."));
        assert!(out[0].content.contains("Cited evidence:"));
        assert!(out[0].content.contains("[1] (C1) T"));
        assert_eq!(out[1].role, "user");
        assert_eq!(out[1].content, "Hi");
    }

    #[test]
    fn unknown_role_is_coerced_to_user() {
        let req = InferenceRequest {
            request_id: "r1".into(),
            system_prompt: "".into(),
            messages: vec![InferenceMessage {
                role: "tool".into(),
                body: "payload".into(),
            }],
            evidence: vec![],
            options: opts(),
        };
        let out = format_chat_prompt(&req);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].role, "user");
    }

    #[test]
    fn empty_request_yields_empty_vec_then_user_turn() {
        let req = InferenceRequest {
            request_id: "r1".into(),
            system_prompt: "".into(),
            messages: vec![],
            evidence: vec![],
            options: opts(),
        };
        let out = format_chat_prompt(&req);
        assert!(out.is_empty());
        let final_messages = ensure_non_empty(out);
        assert_eq!(final_messages.len(), 1);
        assert_eq!(final_messages[0].role, "user");
        assert!(final_messages[0].content.is_empty());
    }

    #[test]
    fn nul_byte_in_message_is_rejected() {
        let messages = vec![InferenceMessage {
            role: "user".into(),
            body: "ab\u{0000}cd".into(),
        }];
        assert!(validate_messages(&messages).is_err());
    }

    #[test]
    fn nul_byte_in_evidence_is_rejected() {
        let evidence = vec![InferenceEvidence {
            citation_key: "k".into(),
            title: "t".into(),
            text: "x\u{0000}y".into(),
            source_kind: "article".into(),
            source_id: "id".into(),
        }];
        assert!(validate_evidence(&evidence).is_err());
    }
}
