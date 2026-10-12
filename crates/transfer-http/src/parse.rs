//! Header parsing helpers used by the HTTP transfer source. These are pure
//! functions so they can be tested without a live HTTP server.

/// Parsed `Content-Range` value.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedContentRange {
    pub start: u64,
    pub end_inclusive: u64,
    pub total: Option<u64>,
}

/// Parse a `Content-Range` header value of the form
/// `bytes <start>-<end>/<total>` or `bytes <start>-<end>/*`.
pub(crate) fn content_range(value: &str) -> Option<ParsedContentRange> {
    let value = value.trim();
    let value = value.strip_prefix("bytes ")?;
    let value = value.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let slash = value.find('/')?;
    let range_part = value.get(..slash)?;
    let total_part = value.get(slash + 1..)?;
    let total = if total_part == "*" {
        None
    } else {
        Some(total_part.parse::<u64>().ok()?)
    };
    let dash = range_part.find('-')?;
    let start: u64 = range_part.get(..dash)?.parse().ok()?;
    let end: u64 = range_part.get(dash + 1..)?.parse().ok()?;
    if end < start || total.is_some_and(|total| end >= total) {
        return None;
    }
    Some(ParsedContentRange {
        start,
        end_inclusive: end,
        total,
    })
}

/// Convenience: return only the total length (`/<total>` suffix) from a
/// `Content-Range` value. Used by `probe`, which only needs the total size.
pub(crate) fn content_range_total(value: &str) -> Option<u64> {
    content_range(value)?.total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_complete_content_range() {
        let parsed = content_range("bytes 0-1023/20480").unwrap();
        assert_eq!(
            parsed,
            ParsedContentRange {
                start: 0,
                end_inclusive: 1023,
                total: Some(20480),
            }
        );
    }

    #[test]
    fn parses_unknown_total() {
        let parsed = content_range("bytes 100-199/*").unwrap();
        assert_eq!(
            parsed,
            ParsedContentRange {
                start: 100,
                end_inclusive: 199,
                total: None,
            }
        );
    }

    #[test]
    fn content_range_total_extracts_total() {
        assert_eq!(content_range_total("bytes 0-0/12345"), Some(12345));
        assert_eq!(content_range_total("bytes 0-0/*"), None);
        assert_eq!(content_range_total("not a range"), None);
    }

    #[test]
    fn rejects_malformed_values() {
        assert!(content_range("bytes junk/12345").is_none());
        assert!(content_range("notbytes 0-1/10").is_none());
        assert!(content_range("bytes 0/10").is_none());
    }
}
