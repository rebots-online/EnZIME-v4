#![forbid(unsafe_code)]

//! §11 cited conversation service.
//!
//! Orchestrates the staged retrieval-and-citation pipeline:
//!
//! 1. normalize the user query without discarding the original spelling;
//! 2. resolve the explicit scope (`RetrievalScope`);
//! 3. retrieve candidates via [`Retriever`] (lexical + vector + merge +
//!    dedup + rerank + diversity, all in the provider implementation);
//! 4. allocate the context budget and map evidence to deterministic
//!    citation keys via [`ContextBuilder`];
//! 5. generate via an injected [`InferenceSession`];
//! 6. validate every emitted citation key against the stored evidence
//!    (A-08 — a successful assistant message never carries a citation that
//!    does not resolve to real stored retrieval evidence);
//! 7. return a [`CitedConversation`] carrying the assistant text, the
//!    validated citations, and a [`ConversationStatus`]. A grounded scope
//!    that produced factual text with zero resolvable citations is
//!    returned as [`ConversationStatus::Failed`] so the caller persists
//!    the turn with status `failed` and surfaces it as an actionable
//!    retry (§11.4) — never as a successful cited answer.
//!
//! The service depends only on `domain` and `ports` (A-02). The retriever,
//! context builder and inference session are injected by the host.

use std::collections::HashSet;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use ports::inference::{
    InferenceChunk, InferenceError, InferenceEvidence, InferenceMessage, InferenceRequest,
    InferenceSession, SessionOptions,
};
use ports::retrieval::{
    CitationLocator, ContextBuilder, ContextSet, RetrievalError, RetrievalQuery, RetrievalScope,
    Retriever,
};

pub use ports::retrieval::{ContextEvidence, RetrievalHit};

/// Maximum evidence chunks per §11.3 ("maximum 12 chunks").
pub const MAX_EVIDENCE_CHUNKS: usize = 12;

/// Maximum adjacent chunks from one document per §11.3.
pub const MAX_ADJACENT_CHUNKS_PER_DOCUMENT: u32 = 3;

/// Minimum chunk length per §11.3 ("no chunk below 40 characters").
pub const MIN_CHUNK_CHARS: usize = 40;

/// System-reserve cap per §11.3 (`min(2048, floor(C * 0.12))`).
pub const SYSTEM_RESERVE_CAP: u32 = 2048;

/// Conversation-reserve cap per §11.3 (`min(4096, floor(C * 0.25))`).
pub const CONVERSATION_RESERVE_CAP: u32 = 4096;

/// A user turn submitted to [`AskService::ask`].
///
/// `history` excludes the current `user_message`; the service appends the
/// user turn before submitting to inference so callers do not have to
/// remember to duplicate it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AskRequest {
    pub request_id: String,
    pub user_message: String,
    pub scope: RetrievalScope,
    pub retrieval_limit: u32,
    pub include_source_kinds: Vec<String>,
    pub history: Vec<InferenceMessage>,
    pub system_prompt: String,
    pub session_options: SessionOptions,
}

/// A citation emitted by the assistant that resolves to exactly one
/// [`ContextEvidence`]. Citations that do not resolve are excluded from
/// this list (§11.4: "Unknown keys are rendered as invalid and excluded
/// from `message_citations`").
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidCitation {
    pub citation_key: String,
    pub retrieval_chunk_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub locator: CitationLocator,
}

/// Token usage reported by the inference backend, if any.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Why a grounded turn was marked [`ConversationStatus::Failed`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FailureReason {
    /// §11.4: grounded scope + non-empty assistant text + zero valid
    /// citations. Persist with status `failed` and present as an
    /// actionable retry.
    NoValidCitationsInGroundedScope,
}

/// Outcome classification for one assistant turn.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConversationStatus {
    /// Either at least one emitted citation resolved to stored evidence,
    /// or the turn was ungrounded (the context builder supplied no
    /// evidence to cite).
    Ok,
    /// §11.4 failure — the turn ran to completion but must be persisted
    /// with status `failed` and surfaced as a retry.
    Failed(FailureReason),
}

/// The full result of one cited-conversation turn. Carries everything the
/// host needs to persist the turn and its citations atomically (§11.1
/// step 13): assistant text, validated citations, the context set used as
/// grounding, and the run's classification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitedConversation {
    pub request_id: String,
    pub user_message: String,
    pub assistant_text: String,
    /// Every citation key the model emitted, in emission order, including
    /// unknown ones. Kept for audit; only the resolvable subset appears in
    /// `valid_citations`.
    pub emitted_citation_keys: Vec<String>,
    pub valid_citations: Vec<ValidCitation>,
    pub context: ContextSet,
    pub usage: Option<UsageRecord>,
    pub finish_reason: Option<String>,
    pub status: ConversationStatus,
}

impl CitedConversation {
    /// True when at least one piece of evidence grounded the turn. An
    /// ungrounded turn (no evidence supplied to the model) is never
    /// failed for missing citations.
    pub fn is_grounded(&self) -> bool {
        !self.context.evidence.is_empty()
    }
}

/// Pipeline-level error — the turn could not be completed. Distinct from
/// [`ConversationStatus::Failed`], which is a completed-but-rejected turn.
#[derive(Clone, Debug, thiserror::Error)]
pub enum AskError {
    /// Retriever or context builder returned [`RetrievalError`].
    #[error("retrieval failed: {0}")]
    Retrieval(#[from] RetrievalError),
    /// Inference backend returned [`InferenceError`].
    #[error("inference failed: {0}")]
    Inference(#[from] InferenceError),
    /// The submitted `user_message` was empty or whitespace-only.
    #[error("user message is empty")]
    EmptyMessage,
}

/// The §11 cited-conversation orchestrator.
///
/// Construct one per host with the injected Ring-2 retriever and context
/// builder. The [`InferenceSession`] is passed per turn so the host can
/// pool, replace or telemetry-wrap it without the service owning its
/// lifetime.
pub struct AskService<'deps> {
    retriever: &'deps dyn Retriever,
    context_builder: &'deps dyn ContextBuilder,
}

impl<'deps> AskService<'deps> {
    pub fn new(
        retriever: &'deps dyn Retriever,
        context_builder: &'deps dyn ContextBuilder,
    ) -> Self {
        Self {
            retriever,
            context_builder,
        }
    }

    /// Run the §11 pipeline for one turn.
    ///
    /// The caller owns `session` (opened from an `InferenceBackend`) and
    /// may reuse it across turns. This service never opens or closes it.
    pub async fn ask(
        &self,
        session: &mut dyn InferenceSession,
        request: AskRequest,
    ) -> Result<CitedConversation, AskError> {
        if request.user_message.trim().is_empty() {
            return Err(AskError::EmptyMessage);
        }

        // §11.1 step 1: normalize the query without discarding the user's
        // spelling. The normalized form drives retrieval; the original
        // `user_message` is what gets stored and shown.
        let normalized = normalize_query(&request.user_message);

        // §11.3 evidence budget — derived from SessionOptions so the
        // retriever and context builder see the same cap the host
        // configured for the inference session.
        let budget_tokens = compute_evidence_budget(&request.session_options);

        let query = RetrievalQuery {
            text: normalized,
            scope: request.scope.clone(),
            limit: request.retrieval_limit,
            context_budget_tokens: budget_tokens,
            include_source_kinds: request.include_source_kinds.clone(),
        };

        // §11.1 steps 3–8: candidate retrieval through diversity enforcement
        // live inside the Retriever provider. The service treats it as a
        // black box that returns ranked, deduplicated, reranked hits.
        let hits = self.retriever.retrieve(query.clone()).await?;

        // §11.1 steps 9–10: budget allocation and citation-key mapping live
        // inside the ContextBuilder provider. The returned ContextSet
        // already carries deterministic citation keys ([S1]..[S12]) on
        // each ContextEvidence.
        let context = self.context_builder.build(&query, hits).await?;

        let valid_key_set: HashSet<&str> = context
            .evidence
            .iter()
            .map(|e| e.citation_key.as_str())
            .collect();

        // §11.4: "The model is instructed to cite only known keys." The
        // structured evidence reaches the backend through InferenceEvidence;
        // the system prompt carries the citation policy.
        let inference_evidence: Vec<InferenceEvidence> = context
            .evidence
            .iter()
            .map(|e| InferenceEvidence {
                citation_key: e.citation_key.clone(),
                title: e.title.clone(),
                text: e.text.clone(),
                source_kind: e.source_kind.clone(),
                source_id: e.source_id.clone(),
            })
            .collect();

        let system_prompt = compose_system_prompt(&request.system_prompt, &valid_key_set);

        // Append the current user turn to history. Callers pass `history`
        // WITHOUT the in-flight user message; the service owns the join so
        // it cannot be double-counted.
        let mut messages = request.history.clone();
        messages.push(InferenceMessage {
            role: "user".to_string(),
            body: request.user_message.clone(),
        });

        let inference_request = InferenceRequest {
            request_id: request.request_id.clone(),
            system_prompt,
            messages,
            evidence: inference_evidence,
            options: request.session_options.clone(),
        };

        let mut stream = session.generate(inference_request).await?;

        let mut assistant_text = String::new();
        let mut emitted_keys: Vec<String> = Vec::new();
        let mut usage: Option<UsageRecord> = None;
        let mut finish_reason: Option<String> = None;

        // §11.1 step 12: validate emitted citation keys against the stored
        // evidence. Only resolvable keys survive into `valid_citations`.
        while let Some(chunk) = stream.next().await {
            match chunk? {
                InferenceChunk::TextDelta { text } => assistant_text.push_str(&text),
                InferenceChunk::CitationMarker { citation_key } => {
                    if !emitted_keys.contains(&citation_key) {
                        emitted_keys.push(citation_key);
                    }
                }
                InferenceChunk::Usage {
                    input_tokens,
                    output_tokens,
                } => {
                    usage = Some(UsageRecord {
                        input_tokens,
                        output_tokens,
                    });
                }
                InferenceChunk::Completed {
                    finish_reason: reason,
                } => {
                    finish_reason = Some(reason);
                }
            }
        }

        let valid_citations: Vec<ValidCitation> = emitted_keys
            .iter()
            .filter_map(|key| {
                let evidence = context
                    .evidence
                    .iter()
                    .find(|e| &e.citation_key == key)?;
                Some(ValidCitation {
                    citation_key: evidence.citation_key.clone(),
                    retrieval_chunk_id: evidence.retrieval_chunk_id.clone(),
                    source_kind: evidence.source_kind.clone(),
                    source_id: evidence.source_id.clone(),
                    title: evidence.title.clone(),
                    locator: evidence.locator.clone(),
                })
            })
            .collect();

        let grounded = !context.evidence.is_empty();
        let has_factual_text = !assistant_text.trim().is_empty();
        let status = if grounded && has_factual_text && valid_citations.is_empty() {
            ConversationStatus::Failed(FailureReason::NoValidCitationsInGroundedScope)
        } else {
            ConversationStatus::Ok
        };

        Ok(CitedConversation {
            request_id: request.request_id,
            user_message: request.user_message,
            assistant_text,
            emitted_citation_keys: emitted_keys,
            valid_citations,
            context,
            usage,
            finish_reason,
            status,
        })
    }
}

/// §11.1 step 1: normalize a query without discarding the user's spelling.
///
/// Trims leading/trailing whitespace and collapses internal whitespace
/// runs to single spaces. Case, punctuation and spelling are preserved
/// — the normalized form is what the retriever sees; the original form is
/// what the user wrote and what the assistant message log stores.
fn normalize_query(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_ws = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out
}

/// §11.3 evidence budget:
/// `C - min(2048, floor(C*0.12)) - min(4096, floor(C*0.25)) - max_output_tokens`.
///
/// Floors via integer arithmetic so the result is reproducible across
/// hosts; `saturating_sub` clamps a malformed configuration at zero
/// rather than underflowing.
pub fn compute_evidence_budget(options: &SessionOptions) -> u32 {
    let context = options.context_tokens;
    let system_reserve = std::cmp::min(SYSTEM_RESERVE_CAP, (context * 12) / 100);
    let conversation_reserve = std::cmp::min(CONVERSATION_RESERVE_CAP, (context * 25) / 100);
    let output_reserve = options.max_output_tokens;
    context
        .saturating_sub(system_reserve)
        .saturating_sub(conversation_reserve)
        .saturating_sub(output_reserve)
}

/// Compose the system prompt: the host-supplied base template plus the
/// §11.4 citation policy. The policy names every valid key explicitly so
/// the model has an unambiguous allowlist.
fn compose_system_prompt(base: &str, valid_keys: &HashSet<&str>) -> String {
    let mut keys_sorted: Vec<&&str> = valid_keys.iter().collect();
    keys_sort_natural(&mut keys_sorted);

    let mut out = String::with_capacity(base.len() + 256);
    out.push_str(base.trim());
    if !out.is_empty() {
        out.push_str("\n\n");
    }

    out.push_str(
        "Citation policy: when you rely on a supplied piece of evidence, mark it with its \
         bracketed citation key (for example [S1]). Only cite keys that appear in the evidence \
         list; never invent or guess a key. If a claim cannot be grounded in the evidence, \
         state that plainly instead of citing.",
    );

    if !keys_sorted.is_empty() {
        out.push_str(" Valid keys for this turn: ");
        let mut first = true;
        for key in keys_sorted {
            if !first {
                out.push_str(", ");
            }
            first = false;
            out.push('[');
            out.push_str(key);
            out.push(']');
        }
        out.push('.');
    }

    out
}

/// Sort citation keys by their numeric suffix (S1, S2, …, S10, S11) so the
/// system prompt allowlist is presented to the model in canonical order
/// regardless of how the context builder enumerated its evidence.
fn keys_sort_natural(keys: &mut Vec<&&str>) {
    fn key_index(k: &str) -> (usize, &str) {
        let stripped = k.strip_prefix("S").unwrap_or(k);
        let idx = stripped.parse::<usize>().unwrap_or(usize::MAX);
        (idx, k)
    }
    keys.sort_by(|a, b| {
        let (ai, _) = key_index(a);
        let (bi, _) = key_index(b);
        ai.cmp(&bi)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_whitespace_and_preserves_spelling() {
        assert_eq!(normalize_query("  What   is\n\tPZN?  "), "What is PZN?");
        assert_eq!(normalize_query(""), "");
        assert_eq!(normalize_query("Elephant"), "Elephant");
    }

    #[test]
    fn evidence_budget_uses_spec_formula() {
        // C = 4096, max_output = 512:
        //   system = min(2048, floor(4096*0.12)) = min(2048, 491) = 491
        //   conv   = min(4096, floor(4096*0.25)) = min(4096, 1024) = 1024
        //   budget = 4096 - 491 - 1024 - 512 = 2069
        let opts = SessionOptions {
            context_tokens: 4096,
            max_output_tokens: 512,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: Vec::new(),
        };
        assert_eq!(compute_evidence_budget(&opts), 2069);
    }

    #[test]
    fn evidence_budget_clamps_caps_at_spec_maxima() {
        // C = 32768, max_output = 1024:
        //   system = min(2048, 3932) = 2048  (cap engaged)
        //   conv   = min(4096, 8192) = 4096  (cap engaged)
        //   budget = 32768 - 2048 - 4096 - 1024 = 25600
        let opts = SessionOptions {
            context_tokens: 32768,
            max_output_tokens: 1024,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: Vec::new(),
        };
        assert_eq!(compute_evidence_budget(&opts), 25600);
    }

    #[test]
    fn evidence_budget_never_underflows() {
        let opts = SessionOptions {
            context_tokens: 1024,
            max_output_tokens: 4096,
            temperature: 0.0,
            top_p: 1.0,
            seed: 0,
            stop_sequences: Vec::new(),
        };
        assert_eq!(compute_evidence_budget(&opts), 0);
    }

    #[test]
    fn system_prompt_lists_valid_keys_in_natural_order() {
        let mut keys: HashSet<&str> = HashSet::new();
        for k in ["S10", "S2", "S1"] {
            keys.insert(k);
        }
        let prompt = compose_system_prompt("Be helpful.", &keys);
        assert!(prompt.contains("Be helpful."));
        assert!(prompt.contains("Citation policy:"));
        assert!(prompt.contains("[S1], [S2], [S10]"));
    }

    #[test]
    fn system_prompt_omits_key_list_when_ungrounded() {
        let keys: HashSet<&str> = HashSet::new();
        let prompt = compose_system_prompt("Be helpful.", &keys);
        assert!(prompt.contains("Citation policy:"));
        assert!(!prompt.contains("Valid keys for this turn:"));
    }
}
