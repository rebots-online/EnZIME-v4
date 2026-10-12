//! Compatibility forwarding to canonical AnZimmermanLib.
pub use anzimmerman::cluster::{
    COMPRESSION_BZIP2, COMPRESSION_DEFAULT, COMPRESSION_EXTENDED, COMPRESSION_LZMA,
    COMPRESSION_NONE, COMPRESSION_ZLIB, COMPRESSION_ZSTD,
};
use ports::zim::ZimError;
pub fn decompress_cluster(compression: u8, payload: &[u8]) -> Result<Vec<u8>, ZimError> {
    anzimmerman::cluster::decompress_cluster(compression, payload)
        .map_err(crate::convert::adapter_error)
}
pub fn extract_blob(decompressed: &[u8], blob_number: u32) -> Result<&[u8], ZimError> {
    anzimmerman::cluster::extract_blob(decompressed, blob_number)
        .map_err(crate::convert::adapter_error)
}
pub fn decompress_cluster_blobs(raw: &[u8]) -> Result<Vec<Vec<u8>>, ZimError> {
    anzimmerman::cluster::decompress_cluster_blobs(raw).map_err(crate::convert::adapter_error)
}
