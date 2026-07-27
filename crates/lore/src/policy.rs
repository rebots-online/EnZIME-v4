#![forbid(unsafe_code)]

//! §12.3 extraction policy and §12.4 consolidation rules.
//!
//! The policy is opt-in per conversation; the service is constructed without
//! auto-promotion by default. Candidate classes are the six allowed by
//! §12.3 ("stable preference", "explicit biographical assertion by the
//! user", "project constraint", "named relationship", "correction of an
//! existing memory", "recurring workflow preference"); the forbidden
//! classes enumerated by §12.3 (inferred sensitive attribute, medical
//! diagnosis, legal status, credential or secret, incidental one-time
//! request, content quoted from a source as if it described the user, and
//! model speculation) are filtered out before any candidate is persisted.
//!
//! No candidate becomes accepted without an explicit user action via
//! [`ports::lore::LoreService::accept`] or a user-configured deterministic
//! promotion policy (see [`PromotionPolicy`]).
//!
//! Consolidation (§12.4) groups candidates by their normalized
//! `(subject_key, predicate_key, object_json)` triple. Two candidates with
//! identical triples are merged into one candidate carrying the union of
//! their evidence rows; two candidates with contradictory objects are kept
//! separate. The consolidation never rewrites a statement, never deletes
//! evidence, and never promotes a candidate — exactly the may-not list in
//! §12.4.

use serde::{Deserialize, Serialize};

/// Allowed candidate classes per §12.3.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateClass {
    /// "I prefer X" / "I like using X" / "My favourite X is Y"
    StablePreference,
    /// "I live in X" / "I work at Y" / "My name is Z" (non-sensitive only)
    BiographicalAssertion,
    /// "Don't X" / "Always Y" / "In this project, Z"
    ProjectConstraint,
    /// "X is my colleague" / "I work with Y"
    NamedRelationship,
    /// "Actually, X" / "No, Y" / "Correcting: Z"
    Correction,
    /// "When I X, I Y" / "I usually Z"
    RecurringWorkflowPreference,
}

impl CandidateClass {
    /// Normalized predicate key persisted as `lore_candidates.predicate_key`
    /// and `lore_memories.predicate_key`.
    pub fn predicate_key(self) -> &'static str {
        match self {
            CandidateClass::StablePreference => "preference",
            CandidateClass::BiographicalAssertion => "biographical",
            CandidateClass::ProjectConstraint => "constraint",
            CandidateClass::NamedRelationship => "relationship",
            CandidateClass::Correction => "correction",
            CandidateClass::RecurringWorkflowPreference => "workflow",
        }
    }

    /// Base extraction confidence for the class, clamped to `[0.0, 1.0]` to
    /// satisfy the schema's `CHECK(confidence >= 0.0 AND confidence <= 1.0)`.
    pub fn base_confidence(self) -> f32 {
        match self {
            CandidateClass::StablePreference => 0.85,
            CandidateClass::BiographicalAssertion => 0.80,
            CandidateClass::ProjectConstraint => 0.90,
            CandidateClass::NamedRelationship => 0.75,
            CandidateClass::Correction => 0.70,
            CandidateClass::RecurringWorkflowPreference => 0.80,
        }
    }
}

/// Output of a successful classification of one sentence.
#[derive(Clone, Debug)]
pub struct ExtractedAssertion {
    pub class: CandidateClass,
    /// Human-readable statement summarised from the source sentence. Never
    /// rewrites the user's words; only normalises a leading prefix.
    pub statement: String,
    /// Structured object value derived from the source sentence.
    pub object: serde_json::Value,
}

impl ExtractedAssertion {
    pub fn locator(
        &self,
        conversation_id: &str,
        message_id: &str,
        inference_run_id: &str,
        start_char: u32,
        end_char: u32,
    ) -> serde_json::Value {
        serde_json::json!({
            "conversation_id": conversation_id,
            "message_id": message_id,
            "inference_run_id": inference_run_id,
            "start_char": start_char,
            "end_char": end_char,
        })
    }
}

/// Inspect one sentence, returning `Some(assertion)` if it matches an allowed
/// §12.3 class and contains nothing forbidden.
///
/// Returns `None` for any forbidden class per §12.3 (inferred sensitive
/// attribute, medical diagnosis, legal status, credential or secret, content
/// quoted from a source as if it described the user, model speculation, or
/// any one-time incidental request that does not match an allowed class).
pub fn classify(sentence: &str) -> Option<ExtractedAssertion> {
    let trimmed = sentence.trim();
    if trimmed.is_empty() {
        return None;
    }
    if is_forbidden(trimmed) {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if let Some(a) = try_correction(trimmed, &lower) {
        return Some(a);
    }
    if let Some(a) = try_constraint(trimmed, &lower) {
        return Some(a);
    }
    if let Some(a) = try_preference(trimmed, &lower) {
        return Some(a);
    }
    if let Some(a) = try_biographical(trimmed, &lower) {
        return Some(a);
    }
    if let Some(a) = try_relationship(trimmed, &lower) {
        return Some(a);
    }
    if let Some(a) = try_workflow(trimmed, &lower) {
        return Some(a);
    }
    None
}

/// §12.3 forbidden-class filter. Returns `true` if the sentence must never
/// seed a candidate. The matchers are deliberately conservative — they err
/// on the side of dropping, never on the side of inventing a sensitive
/// memory.
fn is_forbidden(sentence: &str) -> bool {
    let s = sentence.to_lowercase();

    // Content quoted from a source as if it described the user.
    let starts_quoted = sentence.starts_with('"') || sentence.starts_with('\u{201C}');
    let ends_quoted = sentence.ends_with('"') || sentence.ends_with('\u{201D}');
    if starts_quoted && ends_quoted && sentence.len() > 2 {
        return true;
    }

    // Model speculation markers.
    for marker in &[
        "maybe ",
        "perhaps ",
        "i think ",
        "i guess ",
        "i wonder ",
        "might be ",
        "could be ",
        "possibly ",
        "i'm not sure ",
        "i am not sure ",
    ] {
        if s.contains(marker) {
            return true;
        }
    }

    // Inferred sensitive attribute.
    for term in &[
        "race",
        "racial",
        "ethnicity",
        "ethnic",
        "religion",
        "religious",
        "sexual orientation",
        "sexual preference",
        "gender identity",
        "political party",
        "political affiliation",
        "nationality",
        "immigrant status",
    ] {
        if s.contains(term) {
            return true;
        }
    }

    // Medical diagnosis.
    for term in &[
        "diagnosed with",
        "my illness",
        "my disease",
        "my disorder",
        "my condition is",
        "my disability",
        "prescribed",
        "my medication",
        "my therapist",
        "my psychiatrist",
        "i suffer from",
        "i have cancer",
        "i have diabetes",
        "i'm depressed",
        "i am depressed",
        "i'm bipolar",
        "i am bipolar",
        "my tumor",
        "my tumour",
        "chronic illness",
        "i'm in chronic pain",
    ] {
        if s.contains(term) {
            return true;
        }
    }

    // Legal status.
    for term in &[
        "convicted",
        "conviction",
        "i was arrested",
        "i am arrested",
        "i'm suing",
        "i am suing",
        "i'm being sued",
        "i am being sued",
        "my visa",
        "i'm a citizen",
        "i am a citizen",
        "i'm undocumented",
        "i am undocumented",
        "on probation",
        "on parole",
        "i'm married",
        "i am married",
        "i'm divorced",
        "i am divorced",
        "i'm filing for bankruptcy",
    ] {
        if s.contains(term) {
            return true;
        }
    }

    // Credentials and secrets.
    for term in &[
        "password",
        "passwd",
        "api key",
        "apikey",
        "secret key",
        "my secret",
        "my pin ",
        "social security",
        "ssn",
        "credit card",
        "bank account",
        "routing number",
        "private key",
        "seed phrase",
        "mnemonic phrase",
        "two-factor code",
        "2fa code",
        "mfa code",
        "one-time password",
        "otp code",
        "access token",
        "bearer token",
        "refresh token",
    ] {
        if s.contains(term) {
            return true;
        }
    }

    false
}

fn object_value(value: &str) -> serde_json::Value {
    let trimmed = value.trim().trim_end_matches('.').trim();
    serde_json::json!({ "value": trimmed })
}

fn try_correction(sentence: &str, lower: &str) -> Option<ExtractedAssertion> {
    const PREFIXES: &[&str] = &[
        "actually, ",
        "actually ",
        "no, ",
        "correcting ",
        "correction: ",
        "update: ",
        "wrong: ",
        "to clarify, ",
        "to correct: ",
        "i retract: ",
    ];
    for p in PREFIXES {
        if let Some(_rest_lower) = lower.strip_prefix(p) {
            let after = &sentence[p.len()..];
            let statement = format!("Correction: {}", after.trim());
            let object = object_value(after);
            return Some(ExtractedAssertion {
                class: CandidateClass::Correction,
                statement,
                object,
            });
        }
    }
    // "no, " is common enough to also try as an infix correction.
    if lower.starts_with("no ") {
        let after = &sentence["no ".len()..];
        let statement = format!("Correction: {}", after.trim());
        let object = object_value(after);
        return Some(ExtractedAssertion {
            class: CandidateClass::Correction,
            statement,
            object,
        });
    }
    None
}

fn try_constraint(sentence: &str, lower: &str) -> Option<ExtractedAssertion> {
    const PREFIXES: &[&str] = &[
        "don't ",
        "do not ",
        "never ",
        "always ",
        "must ",
        "we should ",
        "in this project, ",
        "constraint: ",
        "rule: ",
        "hard rule: ",
    ];
    for p in PREFIXES {
        if lower.starts_with(p) {
            let after = &sentence[p.len()..];
            let statement = format!("Project constraint: {}", after.trim());
            let object = object_value(after);
            return Some(ExtractedAssertion {
                class: CandidateClass::ProjectConstraint,
                statement,
                object,
            });
        }
    }
    None
}

fn try_preference(sentence: &str, lower: &str) -> Option<ExtractedAssertion> {
    const MARKERS: &[&str] = &[
        "i prefer ",
        "my preferred ",
        "i like to use ",
        "i like using ",
        "i always use ",
        "i favour ",
        "i favor ",
        "my favorite ",
        "my favourite ",
    ];
    for m in MARKERS {
        if let Some(idx) = lower.find(m) {
            let after = &sentence[idx + m.len()..];
            let statement = format!("User prefers: {}", after.trim());
            let object = object_value(after);
            return Some(ExtractedAssertion {
                class: CandidateClass::StablePreference,
                statement,
                object,
            });
        }
    }
    None
}

fn try_biographical(sentence: &str, lower: &str) -> Option<ExtractedAssertion> {
    const MARKERS: &[&str] = &[
        "i live in ",
        "i'm based in ",
        "i am based in ",
        "i work at ",
        "i work as ",
        "i study at ",
        "i study ",
        "i speak ",
        "my name is ",
        "i'm from ",
        "i am from ",
    ];
    for m in MARKERS {
        if let Some(idx) = lower.find(m) {
            let after = &sentence[idx + m.len()..];
            let statement = format!("Biographical: {}", sentence.trim());
            let object = object_value(after);
            return Some(ExtractedAssertion {
                class: CandidateClass::BiographicalAssertion,
                statement,
                object,
            });
        }
    }
    None
}

fn try_relationship(sentence: &str, lower: &str) -> Option<ExtractedAssertion> {
    const MARKERS: &[&str] = &[
        " is my ",
        " are my ",
        "i work with ",
        "i collaborate with ",
    ];
    for m in MARKERS {
        if lower.contains(m) {
            let statement = format!("Relationship: {}", sentence.trim());
            let object = object_value(sentence);
            return Some(ExtractedAssertion {
                class: CandidateClass::NamedRelationship,
                statement,
                object,
            });
        }
    }
    None
}

fn try_workflow(sentence: &str, lower: &str) -> Option<ExtractedAssertion> {
    const MARKERS: &[&str] = &[
        "when i ",
        "every time i ",
        "i usually ",
        "i typically ",
        "my routine ",
        "my workflow ",
    ];
    for m in MARKERS {
        if lower.contains(m) {
            let statement = format!("Workflow: {}", sentence.trim());
            let object = object_value(sentence);
            return Some(ExtractedAssertion {
                class: CandidateClass::RecurringWorkflowPreference,
                statement,
                object,
            });
        }
    }
    None
}

/// Split a body of markdown into coarse sentence candidates.
///
/// Operates on `.`, `!`, `?` and line breaks. Fragments shorter than five
/// characters are dropped (they cannot carry a useful assertion). Each
/// surviving fragment is trimmed and given a single terminating full stop so
/// excerpts read naturally when displayed in the Memory Centre inbox.
pub fn split_sentences(text: &str) -> Vec<String> {
    text.split(|c: char| matches!(c, '\n' | '\r' | '.' | '!' | '?'))
        .map(|s| s.trim())
        .filter(|s| s.chars().count() >= 5)
        .map(|s| {
            let cleaned = s.trim_end_matches(|c: char| c == ',' || c == ';').trim();
            if cleaned.is_empty() {
                return String::new();
            }
            let mut out = cleaned.to_string();
            if !out.ends_with('.') && !out.ends_with('!') && !out.ends_with('?') {
                out.push('.');
            }
            out
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// User-configured deterministic promotion policy per §12.3 ("user-configured
/// deterministic promotion policy whose rule and evidence are displayed in
/// the Memory Centre").
///
/// A rule matches a freshly-extracted candidate when its `predicate_key`
/// equals the candidate's, the candidate's `confidence` is at least
/// `min_confidence`, and the candidate has at least `min_evidence_count`
/// evidence rows. On match, the candidate is auto-accepted with
/// `authored_by = 'policy'`; the rule's `reason` is recorded as the
/// resulting revision's `correction_reason` so the Memory Centre can show
/// the user exactly which rule promoted which candidate.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PromotionPolicy {
    pub rules: Vec<PromotionRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromotionRule {
    pub predicate_key: String,
    pub min_confidence: f32,
    pub min_evidence_count: u32,
    pub reason: String,
}

impl PromotionPolicy {
    /// Return the first rule whose thresholds the candidate satisfies, or
    /// `None` if no rule matches.
    pub fn matches(
        &self,
        predicate_key: &str,
        confidence: f32,
        evidence_count: u32,
    ) -> Option<&PromotionRule> {
        self.rules.iter().find(|r| {
            r.predicate_key == predicate_key
                && confidence >= r.min_confidence
                && evidence_count >= r.min_evidence_count
        })
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn classifies_stable_preference() {
        let a = classify("I prefer Vim editor").unwrap();
        assert_eq!(a.class, CandidateClass::StablePreference);
        assert_eq!(a.object["value"], "Vim editor");
    }

    #[test]
    fn rejects_speculation() {
        assert!(classify("I think maybe Vim is better").is_none());
    }

    #[test]
    fn rejects_medical() {
        assert!(classify("I am depressed").is_none());
    }

    #[test]
    fn rejects_secret() {
        assert!(classify("My password is hunter2").is_none());
    }

    #[test]
    fn rejects_legal_status() {
        assert!(classify("I'm married").is_none());
    }

    #[test]
    fn rejects_quoted_content() {
        assert!(classify("\"The user is from Paris\"").is_none());
    }

    #[test]
    fn classifies_constraint_imperative() {
        let a = classify("Always cite page numbers").unwrap();
        assert_eq!(a.class, CandidateClass::ProjectConstraint);
    }

    #[test]
    fn classifies_biographical() {
        let a = classify("I live in Tokyo").unwrap();
        assert_eq!(a.class, CandidateClass::BiographicalAssertion);
    }

    #[test]
    fn split_sentences_handles_punctuation() {
        let s = split_sentences("I prefer Vim. I prefer Emacs! Why not?");
        assert_eq!(s.len(), 3);
        assert!(s[0].ends_with('.'));
    }
}
