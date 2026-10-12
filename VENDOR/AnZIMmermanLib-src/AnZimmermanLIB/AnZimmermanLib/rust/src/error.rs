#[derive(Clone, Debug, thiserror::Error)]
pub enum ZimError {
    #[error("path is not readable")]
    UnreadablePath,
    #[error("archive length mismatch")]
    LengthMismatch,
    #[error("archive digest mismatch")]
    DigestMismatch,
    #[error("invalid ZIM header: {0}")]
    InvalidHeader(String),
    #[error("unsupported ZIM feature: {0}")]
    UnsupportedFeature(String),
    #[error("entry not found: {0}")]
    EntryNotFound(String),
    #[error("redirect cycle")]
    RedirectCycle,
    #[error("allocation limit exceeded")]
    AllocationLimit,
    #[error("decompression limit exceeded")]
    DecompressionLimit,
    #[error("I/O error: {0}")]
    Io(String),
}
