use percent_encoding::percent_decode_str;
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceRoute {
    Pack {
        pack_id: String,
        entry_key: String,
    },
    Annotation {
        annotation_id: String,
        asset_id: String,
    },
}
pub(super) fn component(raw: &str) -> Result<String, String> {
    // Reject malformed escapes; percent_decode_str deliberately accepts them.
    let bytes = raw.as_bytes();
    for (i, c) in bytes.iter().enumerate() {
        if *c == b'%'
            && (i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit())
        {
            return Err("invalid escape".into());
        }
    }
    let decoded = percent_decode_str(raw)
        .decode_utf8()
        .map_err(|_| "invalid UTF-8")?;
    if decoded.is_empty()
        || decoded == "."
        || decoded == ".."
        || decoded
            .chars()
            .any(|c| c == '/' || c == '\\' || c.is_control())
    {
        return Err("invalid resource component".into());
    }
    Ok(decoded.into_owned())
}
pub fn parse_resource_path(path: &str) -> Result<ResourceRoute, String> {
    if path.len() > 16_384 || !path.starts_with('/') {
        return Err("invalid resource path".into());
    }
    let parts: Vec<_> = path[1..]
        .split('/')
        .map(component)
        .collect::<Result<_, _>>()?;
    match parts.as_slice() {
        [route, id, namespace, rest @ ..]
            if route == "pack"
                && namespace.len() == 1
                && (namespace.as_bytes()[0].is_ascii_alphabetic() || namespace == "-")
                && !rest.is_empty() =>
        {
            Ok(ResourceRoute::Pack {
                pack_id: id.clone(),
                entry_key: parts[2..].join("/"),
            })
        }
        [route, id, asset] if route == "annotation" => Ok(ResourceRoute::Annotation {
            annotation_id: id.clone(),
            asset_id: asset.clone(),
        }),
        _ => Err("unknown resource route".into()),
    }
}
pub fn byte_range(header: Option<&str>, length: u64) -> Result<(u64, u64, bool), String> {
    let Some(value) = header else {
        return Ok((0, length, false));
    };
    let value = value.strip_prefix("bytes=").ok_or("invalid range unit")?;
    if value.contains(',') || length == 0 {
        return Err("unsatisfiable range".into());
    }
    let (start, end) = value.split_once('-').ok_or("invalid byte range")?;
    let numeric = |s: &str| -> Result<u64, String> {
        if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) {
            return Err("invalid byte range".into());
        }
        s.parse().map_err(|_| "range overflow".into())
    };
    if start.is_empty() {
        let suffix = numeric(end)?;
        if suffix == 0 {
            return Err("empty suffix".into());
        }
        Ok((length.saturating_sub(suffix), length, true))
    } else {
        let start = numeric(start)?;
        let end = if end.is_empty() {
            length
        } else {
            numeric(end)?.saturating_add(1).min(length)
        };
        if start >= length || end <= start {
            return Err("unsatisfiable range".into());
        }
        Ok((start, end, true))
    }
}
pub fn passive_mime(mime: &str) -> bool {
    if mime.len() > 256 || mime.chars().any(char::is_control) {
        return false;
    }
    matches!(
        mime.split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "image/png"
            | "image/jpeg"
            | "image/gif"
            | "image/webp"
            | "image/avif"
            | "image/bmp"
            | "image/x-icon"
            | "image/vnd.microsoft.icon"
            | "image/svg+xml"
            | "audio/ogg"
            | "audio/webm"
            | "audio/mpeg"
            | "audio/mp4"
            | "audio/wav"
            | "audio/x-wav"
            | "audio/flac"
            | "video/webm"
            | "video/mp4"
            | "video/ogg"
            | "text/css"
            | "font/woff"
            | "font/woff2"
            | "font/ttf"
            | "font/otf"
            | "application/font-woff"
            | "application/vnd.ms-fontobject"
            | "application/x-font-ttf"
            | "application/x-font-opentype"
    )
}
