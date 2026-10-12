//! Compatibility forwarding to canonical AnZimmermanLib.
pub use anzimmerman::directory::{
    ArticleEntry, DeletedEntry, LinktargetEntry, RawEntry, RedirectEntry, MIMETYPE_DELETED,
    MIMETYPE_LINKTARGET, MIMETYPE_REDIRECT,
};
use ports::zim::ZimError;
pub fn parse_entry(buf: &[u8]) -> Result<(RawEntry, usize), ZimError> {
    anzimmerman::directory::parse_entry(buf).map_err(crate::convert::adapter_error)
}
