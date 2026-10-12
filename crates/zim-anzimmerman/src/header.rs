//! Compatibility forwarding to canonical AnZimmermanLib.
pub use anzimmerman::header::{Uuid, ZimHeader, NO_PAGE, SUPPORTED_MAJOR_VERSIONS, ZIM_MAGIC};
use ports::zim::ZimError;
pub fn parse_header(buf: &[u8]) -> Result<ZimHeader, ZimError> {
    anzimmerman::header::parse_header(buf).map_err(crate::convert::adapter_error)
}
pub fn validate_header_offsets(header: &ZimHeader, file_size: u64) -> Result<(), ZimError> {
    anzimmerman::header::validate_header_offsets(header, file_size)
        .map_err(crate::convert::adapter_error)
}
