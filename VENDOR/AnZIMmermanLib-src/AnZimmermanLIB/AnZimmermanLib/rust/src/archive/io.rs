use super::*;
pub(super) fn verify_sha256(
    file: &mut File,
    size: u64,
    expected_hex: &str,
) -> Result<(), ZimError> {
    let expected = decode_sha256_hex(expected_hex)?;
    file.seek(SeekFrom::Start(0)).map_err(io_err)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; HASH_CHUNK_BYTES];
    let mut remaining = size;
    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        let n = file.read(&mut buf[..to_read]).map_err(io_err)?;
        if n == 0 {
            return Err(ZimError::Io(
                "unexpected EOF during SHA-256 verification".into(),
            ));
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }
    let hash = hasher.finalize();
    if hash.as_slice() != &expected[..] {
        return Err(ZimError::DigestMismatch);
    }
    Ok(())
}

pub(super) fn decode_sha256_hex(hex_in: &str) -> Result<[u8; 32], ZimError> {
    let hex = hex_in.trim().to_ascii_lowercase();
    if hex.len() != 64 {
        return Err(ZimError::InvalidHeader(format!(
            "expected_sha256 must be 64 hex characters, got {}",
            hex.len()
        )));
    }
    let bytes = hex::decode(&hex)
        .map_err(|_| ZimError::InvalidHeader("expected_sha256 is not valid hex".into()))?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub(super) fn read_mime_list(
    file: &mut File,
    offset: u64,
    file_size: u64,
) -> Result<Vec<String>, ZimError> {
    file.seek(SeekFrom::Start(offset)).map_err(io_err)?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut pos = offset;
    loop {
        if pos >= file_size {
            return Err(ZimError::InvalidHeader(
                "MIME list missing double-NUL terminator".into(),
            ));
        }
        let n = file.read(&mut chunk).map_err(io_err)?;
        if n == 0 {
            return Err(ZimError::InvalidHeader(
                "unexpected EOF in MIME list".into(),
            ));
        }
        bytes.extend_from_slice(&chunk[..n]);
        if let Some(idx) = find_subslice(&bytes, b"\0\0") {
            bytes.truncate(idx);
            break;
        }
        if bytes.len() as u64 > MAX_INDEX_ALLOCATION_BYTES {
            return Err(ZimError::AllocationLimit);
        }
        pos += n as u64;
    }
    let mut mimes = Vec::new();
    for piece in bytes.split(|&b| b == 0) {
        if piece.is_empty() {
            continue;
        }
        let s = std::str::from_utf8(piece)
            .map_err(|_| ZimError::InvalidHeader("MIME type is not valid UTF-8".into()))?;
        mimes.push(s.to_string());
    }
    if mimes.len() as u32 > MAX_MIME_TYPES {
        return Err(ZimError::InvalidHeader(format!(
            "MIME list declares {} entries, max {}",
            mimes.len(),
            MAX_MIME_TYPES
        )));
    }
    Ok(mimes)
}

pub(super) fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

pub(super) fn read_u64_table(
    file: &mut File,
    offset: u64,
    count: u32,
    label: &str,
) -> Result<Vec<u64>, ZimError> {
    let byte_count = (count as u64)
        .checked_mul(8)
        .ok_or_else(|| ZimError::InvalidHeader(format!("{} table size overflows u64", label)))?;
    if byte_count > MAX_INDEX_ALLOCATION_BYTES {
        return Err(ZimError::AllocationLimit);
    }
    file.seek(SeekFrom::Start(offset)).map_err(io_err)?;
    let mut buf = vec![0u8; byte_count as usize];
    file.read_exact(&mut buf).map_err(io_err)?;
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let off = i * 8;
        out.push(u64::from_le_bytes([
            buf[off],
            buf[off + 1],
            buf[off + 2],
            buf[off + 3],
            buf[off + 4],
            buf[off + 5],
            buf[off + 6],
            buf[off + 7],
        ]));
    }
    Ok(out)
}

pub(super) fn build_path_index(
    file: &mut File,
    path_ptrs: &[u64],
    file_size: u64,
) -> Result<HashMap<String, u32>, ZimError> {
    let mut map = HashMap::with_capacity(path_ptrs.len());
    for (idx, &ptr) in path_ptrs.iter().enumerate() {
        if ptr >= file_size {
            return Err(ZimError::InvalidHeader(format!(
                "path pointer {} at index {} >= file size {}",
                ptr, idx, file_size
            )));
        }
        let read_len = ((file_size - ptr).min(MAX_ENTRY_READ_BYTES)) as usize;
        file.seek(SeekFrom::Start(ptr)).map_err(io_err)?;
        let mut buf = vec![0u8; read_len];
        let n = file.read(&mut buf).map_err(io_err)?;
        buf.truncate(n);
        let (raw, _consumed) = parse_entry(&buf)?;
        let key = entry_key_string(raw.namespace(), raw.url());
        map.insert(key, idx as u32);
    }
    Ok(map)
}

pub(super) fn map_open_err(e: std::io::Error, path: &str) -> ZimError {
    match e.kind() {
        std::io::ErrorKind::NotFound
        | std::io::ErrorKind::PermissionDenied
        | std::io::ErrorKind::ConnectionRefused
        | std::io::ErrorKind::AddrInUse
        | std::io::ErrorKind::AddrNotAvailable => ZimError::UnreadablePath,
        _ => {
            if e.to_string().contains("No such file") || e.to_string().contains("permission") {
                ZimError::UnreadablePath
            } else {
                ZimError::Io(format!("cannot open {}: {}", path, e))
            }
        }
    }
}

pub(super) fn io_err(e: std::io::Error) -> ZimError {
    ZimError::Io(e.to_string())
}
