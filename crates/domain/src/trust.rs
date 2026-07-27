use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureEnvelope {
    pub algorithm: String,
    pub key_id: String,
    pub signed_domain: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub state: SignatureVerificationState,
    pub key_id: String,
    pub signer_label: Option<String>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureVerificationState {
    Verified,
    Unsigned,
    Invalid,
    UnknownKey,
    RevokedKey,
}
