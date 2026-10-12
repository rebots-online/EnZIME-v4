//! Lossless application boundary conversions; archive paths are not opaque IDs.
use anzimmerman::model as lib;
use domain::identity::{ArticleKey, ContentDigest, DigestAlgorithm, EntryKey};
use ports::zim as app;
pub(crate) fn adapter_error(v: anzimmerman::ZimError) -> app::ZimError {
    use anzimmerman::ZimError as L;
    match v {
        L::UnreadablePath => app::ZimError::UnreadablePath,
        L::LengthMismatch => app::ZimError::LengthMismatch,
        L::DigestMismatch => app::ZimError::DigestMismatch,
        L::RedirectCycle => app::ZimError::RedirectCycle,
        L::AllocationLimit => app::ZimError::AllocationLimit,
        L::DecompressionLimit => app::ZimError::DecompressionLimit,
        L::InvalidHeader(v) => app::ZimError::InvalidHeader(v),
        L::UnsupportedFeature(v) => app::ZimError::UnsupportedFeature(v),
        L::EntryNotFound(v) => app::ZimError::EntryNotFound(v),
        L::Io(v) => app::ZimError::Io(v),
    }
}
pub(crate) fn open_request(v: &app::ZimOpenRequest) -> lib::ZimOpenRequest {
    lib::ZimOpenRequest {
        path: v.path.clone(),
        expected_sha256: v.expected_sha256.clone(),
        expected_size_bytes: v.expected_size_bytes,
    }
}
fn kind(v: lib::ZimEntryKind) -> app::ZimEntryKind {
    match v {
        lib::ZimEntryKind::Article => app::ZimEntryKind::Article,
        lib::ZimEntryKind::Resource => app::ZimEntryKind::Resource,
        lib::ZimEntryKind::Redirect => app::ZimEntryKind::Redirect,
        lib::ZimEntryKind::Metadata => app::ZimEntryKind::Metadata,
    }
}
fn digest(v: anzimmerman::ContentDigest) -> ContentDigest {
    ContentDigest {
        algorithm: match v.algorithm {
            anzimmerman::DigestAlgorithm::Sha256 => DigestAlgorithm::Sha256,
        },
        hex: v.hex,
    }
}
fn link(v: lib::InternalLink) -> app::InternalLink {
    app::InternalLink {
        href: v.href,
        target: match v.target {
            lib::InternalLinkTarget::Article { article_key } => app::InternalLinkTarget::Article {
                article_key: ArticleKey(article_key.0),
            },
            lib::InternalLinkTarget::Resource { entry_key } => app::InternalLinkTarget::Resource {
                entry_key: EntryKey(entry_key.0),
            },
            lib::InternalLinkTarget::Unsupported { reason } => {
                app::InternalLinkTarget::Unsupported { reason }
            }
        },
    }
}
pub(crate) fn metadata(v: lib::ZimMetadata) -> app::ZimMetadata {
    app::ZimMetadata {
        uuid: v.uuid,
        title: v.title,
        description: v.description,
        language_bcp47: v.language_bcp47,
        creator: v.creator,
        publisher: v.publisher,
        date: v.date,
        article_count: v.article_count,
        media_count: v.media_count,
        main_page: v.main_page.map(|k| ArticleKey(k.0)),
        checksum: v.checksum,
    }
}
pub(crate) fn entry(v: lib::ZimEntryDescriptor) -> app::ZimEntryDescriptor {
    app::ZimEntryDescriptor {
        entry_key: EntryKey(v.entry_key.0),
        namespace: v.namespace,
        path: v.path,
        title: v.title,
        mime_type: v.mime_type,
        kind: kind(v.kind),
        redirect_target: v.redirect_target.map(|k| EntryKey(k.0)),
        byte_length: v.byte_length,
    }
}
pub(crate) fn article(v: lib::ArticleDocument) -> app::ArticleDocument {
    app::ArticleDocument {
        article_key: ArticleKey(v.article_key.0),
        title: v.title,
        mime_type: v.mime_type,
        raw_html: v.raw_html,
        normalized_text: v.normalized_text,
        content_digest: digest(v.content_digest),
        internal_links: v.internal_links.into_iter().map(link).collect(),
    }
}
pub(crate) fn resource(v: lib::ResourceDocument) -> app::ResourceDocument {
    app::ResourceDocument {
        entry_key: EntryKey(v.entry_key.0),
        mime_type: v.mime_type,
        bytes: v.bytes,
        content_digest: digest(v.content_digest),
    }
}
pub(crate) fn hit(v: lib::TitleSearchHit) -> app::TitleSearchHit {
    app::TitleSearchHit {
        article_key: ArticleKey(v.article_key.0),
        title: v.title,
        score: v.score,
    }
}
pub(crate) fn finding(v: lib::ZimVerificationFinding) -> app::ZimVerificationFinding {
    app::ZimVerificationFinding {
        code: v.code,
        entry_key: v.entry_key.map(|k| EntryKey(k.0)),
        detail: v.detail,
    }
}
pub(crate) fn report(v: lib::ZimVerificationReport) -> app::ZimVerificationReport {
    app::ZimVerificationReport {
        valid: v.valid,
        engine_id: v.engine_id,
        engine_version: v.engine_version,
        checked_entries: v.checked_entries,
        warnings: v.warnings.into_iter().map(finding).collect(),
        errors: v.errors.into_iter().map(finding).collect(),
    }
}
pub(crate) fn projection(
    v: anzimmerman::sanitize::ArticleProjection,
) -> crate::sanitize::ArticleProjection {
    crate::sanitize::ArticleProjection {
        normalized_text: v.normalized_text,
        internal_links: v.internal_links.into_iter().map(link).collect(),
    }
}
pub(crate) fn layout_entry(
    v: anzimmerman::archive::ZimLayoutEntry,
) -> crate::archive::ZimLayoutEntry {
    crate::archive::ZimLayoutEntry {
        entry_key: v.entry_key,
        namespace: v.namespace,
        path: v.path,
        title: v.title,
        mime_type: v.mime_type,
        kind: kind(v.kind),
        redirect_target: v.redirect_target.map(|k| EntryKey(k.0)),
        cluster_index: v.cluster_index,
        blob_index: v.blob_index,
    }
}
pub(crate) fn layout(
    v: anzimmerman::archive::ZimLayoutSnapshot,
) -> crate::archive::ZimLayoutSnapshot {
    crate::archive::ZimLayoutSnapshot {
        cluster_byte_ranges: v.cluster_byte_ranges,
        entries: v.entries.into_iter().map(layout_entry).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_error_variants_preserve_variant_and_payload() {
        use anzimmerman::ZimError as L;
        let cases = [
            L::UnreadablePath,
            L::LengthMismatch,
            L::DigestMismatch,
            L::InvalidHeader("offset 80".into()),
            L::UnsupportedFeature("codec 4".into()),
            L::EntryNotFound("A/Café".into()),
            L::RedirectCycle,
            L::AllocationLimit,
            L::DecompressionLimit,
            L::Io("read failed".into()),
        ];
        for error in cases {
            let expected = format!("{error:?}");
            assert_eq!(format!("{:?}", adapter_error(error)), expected);
        }
    }
}
