//! Bounded, single-frame decoding into caller-owned storage.
use crate::{
    cluster::{cluster_encoding, validate_blob_offsets},
    ZimError,
};
use bytes::Bytes;
use std::io::{Read, Seek, SeekFrom, Write};
#[derive(Clone, Copy, Debug)]
pub struct ClusterDecodeLimits {
    pub maximum_output_bytes: u64,
    pub maximum_blob_bytes: u64,
    pub maximum_decoder_window_bytes: u64,
}
impl Default for ClusterDecodeLimits {
    fn default() -> Self {
        Self {
            maximum_output_bytes: 2147483648,
            maximum_blob_bytes: 33554432,
            maximum_decoder_window_bytes: 67108864,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DecodedCluster {
    pub length: u64,
    pub offset_width: u8,
}
fn invalid(e: impl std::fmt::Display) -> ZimError {
    ZimError::InvalidHeader(e.to_string())
}
fn emit(
    output: &mut dyn Write,
    bytes: &[u8],
    count: &mut u64,
    maximum: u64,
) -> Result<(), ZimError> {
    let next = count
        .checked_add(bytes.len() as u64)
        .filter(|n| *n <= maximum)
        .ok_or(ZimError::DecompressionLimit)?;
    output
        .write_all(bytes)
        .map_err(|e| ZimError::Io(e.to_string()))?;
    *count = next;
    Ok(())
}
fn copy_bounded(
    input: &mut dyn Read,
    output: &mut dyn Write,
    maximum: u64,
) -> Result<u64, ZimError> {
    let mut buf = [0u8; 65536];
    let mut count = 0;
    loop {
        let n = input.read(&mut buf).map_err(invalid)?;
        if n == 0 {
            return Ok(count);
        }
        emit(output, &buf[..n], &mut count, maximum)?;
    }
}
fn deflate(
    input: &mut dyn Read,
    output: &mut dyn Write,
    limits: ClusterDecodeLimits,
) -> Result<u64, ZimError> {
    let mut prefix = [0u8; 2];
    input.read_exact(&mut prefix).map_err(invalid)?;
    let wrapped =
        prefix[0] & 15 == 8 && prefix[0] >> 4 <= 7 && u16::from_be_bytes(prefix) % 31 == 0;
    if wrapped && prefix[1] & 32 != 0 {
        return Err(ZimError::UnsupportedFeature(
            "zlib preset dictionary".into(),
        ));
    }
    let window = if wrapped {
        1u64 << ((prefix[0] >> 4) + 8)
    } else {
        32768
    };
    // flate2 may retain its full 32 KiB dictionary even for smaller advertised windows.
    if window.max(32768) > limits.maximum_decoder_window_bytes {
        return Err(ZimError::DecompressionLimit);
    }
    let mut input = std::io::Cursor::new(prefix).chain(input);
    let mut decoder = flate2::Decompress::new(wrapped);
    let mut buffer = [0u8; 65536];
    let mut out = [0u8; 65536];
    let (mut begin, mut end, mut eof, mut count) = (0, 0, false, 0);
    loop {
        if begin == end && !eof {
            end = input.read(&mut buffer).map_err(invalid)?;
            begin = 0;
            eof = end == 0;
        }
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let status = decoder
            .decompress(
                &buffer[begin..end],
                &mut out,
                if eof {
                    flate2::FlushDecompress::Finish
                } else {
                    flate2::FlushDecompress::None
                },
            )
            .map_err(invalid)?;
        let consumed = (decoder.total_in() - before_in) as usize;
        let produced = (decoder.total_out() - before_out) as usize;
        begin += consumed;
        emit(
            output,
            &out[..produced],
            &mut count,
            limits.maximum_output_bytes,
        )?;
        if status == flate2::Status::StreamEnd {
            return Ok(count);
        }
        if consumed == 0 && produced == 0 {
            if eof || begin < end {
                return Err(invalid("truncated or invalid DEFLATE frame"));
            }
        }
    }
}
pub fn decode_cluster_to(
    input: &mut dyn Read,
    output: &mut dyn Write,
    limits: ClusterDecodeLimits,
) -> Result<DecodedCluster, ZimError> {
    if limits.maximum_output_bytes == 0
        || limits.maximum_decoder_window_bytes == 0
        || limits.maximum_blob_bytes == 0
    {
        return Err(ZimError::DecompressionLimit);
    }
    let mut tag = [0];
    input.read_exact(&mut tag).map_err(invalid)?;
    let (codec, width) = cluster_encoding(tag[0])?;
    let length = match codec {
        0 | 1 => copy_bounded(input, output, limits.maximum_output_bytes)?,
        2 => deflate(input, output, limits)?,
        4 => xz(input, output, limits)?,
        5 => {
            let mut decoder = zstd::stream::Decoder::new(input)
                .map_err(invalid)?
                .single_frame();
            let log = 63 - limits.maximum_decoder_window_bytes.leading_zeros();
            decoder.window_log_max(log).map_err(invalid)?;
            copy_bounded(&mut decoder, output, limits.maximum_output_bytes)?
        }
        _ => {
            return Err(ZimError::UnsupportedFeature(format!(
                "cluster codec {codec}"
            )))
        }
    };
    Ok(DecodedCluster {
        length,
        offset_width: width,
    })
}
fn offset(input: &mut (impl Read + Seek), position: u64, width: u8) -> Result<u64, ZimError> {
    input.seek(SeekFrom::Start(position)).map_err(invalid)?;
    let mut bytes = [0; 8];
    input
        .read_exact(&mut bytes[..width as usize])
        .map_err(invalid)?;
    Ok(u64::from_le_bytes(bytes))
}
pub fn read_cluster_blob(
    input: &mut (impl Read + Seek),
    decoded: DecodedCluster,
    blob_index: u32,
    maximum_bytes: u64,
) -> Result<Bytes, ZimError> {
    let width = decoded.offset_width;
    if width != 4 && width != 8 {
        return Err(invalid("invalid offset width"));
    }
    let table = offset(input, 0, width)?;
    if table < 2 * width as u64 || table % width as u64 != 0 || table > decoded.length {
        return Err(invalid("invalid blob offset table"));
    }
    if blob_index as u64 >= table / width as u64 - 1 {
        return Err(ZimError::EntryNotFound(format!("blob {blob_index}")));
    }
    let start = offset(input, blob_index as u64 * width as u64, width)?;
    let end = offset(input, (blob_index as u64 + 1) * width as u64, width)?;
    validate_blob_offsets(
        start,
        end,
        table,
        decoded.length,
        maximum_bytes.min(33554432),
    )?;
    let mut bytes = vec![0; (end - start) as usize];
    input.seek(SeekFrom::Start(start)).map_err(invalid)?;
    input.read_exact(&mut bytes).map_err(invalid)?;
    Ok(bytes.into())
}

fn xz(
    input: &mut dyn Read,
    output: &mut dyn Write,
    limits: ClusterDecodeLimits,
) -> Result<u64, ZimError> {
    let mut decoder =
        xz2::stream::Stream::new_stream_decoder(limits.maximum_decoder_window_bytes, 0)
            .map_err(invalid)?;
    let mut buffer = [0u8; 65536];
    let mut out = [0u8; 65536];
    let (mut begin, mut end, mut eof, mut count) = (0, 0, false, 0);
    loop {
        if begin == end && !eof {
            end = input.read(&mut buffer).map_err(invalid)?;
            begin = 0;
            eof = end == 0;
        }
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let status = decoder
            .process(
                &buffer[begin..end],
                &mut out,
                if eof {
                    xz2::stream::Action::Finish
                } else {
                    xz2::stream::Action::Run
                },
            )
            .map_err(invalid)?;
        let consumed = (decoder.total_in() - before_in) as usize;
        let produced = (decoder.total_out() - before_out) as usize;
        begin += consumed;
        emit(
            output,
            &out[..produced],
            &mut count,
            limits.maximum_output_bytes,
        )?;
        if status == xz2::stream::Status::StreamEnd {
            return Ok(count);
        }
        if consumed == 0 && produced == 0 && (eof || begin < end) {
            return Err(invalid("truncated or invalid XZ frame"));
        }
    }
}
