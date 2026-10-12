#![forbid(unsafe_code)]

//! §7.2 reader navigation, title search and persisted reading position.
//!
//! Wraps the [`ZimArchive`] port with the stateful pieces the reader UI
//! needs:
//!
//! 1. [`NavHistory`] — pure back/forward cursor over article keys, with
//!    standard browser semantics (a new forward move discards the forward
//!    stack; `replace_current` covers redirect canonicalization without
//!    producing a history entry).
//! 2. [`ReaderNav`] — the navigation orchestrator. Owns one navigation
//!    session per open archive and delegates byte-level work to the
//!    injected [`ZimArchive`] (A-02). Provides main-page entry, explicit
//!    article navigation, redirect-aware internal-link following, and
//!    back/forward.
//! 3. [`search_titles`] + [`normalize_search_text`] — title search with
//!    query normalization and fixed page size; an empty normalized query
//!    returns an empty page rather than an error so the UI can render
//!    "type something" guidance.
//! 4. [`ReadingPosition`] + [`ReadingPositionStore`] — persisted scroll
//!    anchor (content-derived selector, never a pixel offset, so it
//!    survives font-size changes).
//! 5. [`Bookmark`] + [`BookmarkStore`] — named saved positions across
//!    articles.
//!
//! The persistence traits follow the same pattern as [`PackActivator`]:
//! the application layer owns the contract; the storage layer (Ring 2)
//! provides the implementation. This module depends only on `domain` and
//! `ports` (A-02) and never names a concrete provider crate.
//!
//! [`PackActivator`]: crate::library::PackActivator

use async_trait::async_trait;
use domain::identity::{ArticleKey, EntryKey};
use serde::{Deserialize, Serialize};

use ports::zim::{
    ArticleDocument, InternalLink, InternalLinkTarget, ResourceDocument, TitleSearchHit,
    TitleSearchQuery, ZimArchive, ZimError,
};

/// Upper bound on redirect-chain traversal when resolving an internal
/// link target. The port's `resolve_redirect` enforces cycle detection
/// up to this hop count.
const MAX_REDIRECT_HOPS: u8 = 8;

/// Title-search page size. The application always asks the archive for
/// this many hits per call; the caller paginates by submitting
/// increasing offsets. The value is small enough to keep one page of
/// suggestions cheap on a low-end Android reader and large enough that a
/// typical "I typed three letters" query is covered in one round trip.
const SEARCH_PAGE_SIZE: u32 = 25;

// ===========================================================================
// Errors
// ===========================================================================

/// Navigation-level failure — the session could not move to the requested
/// article, or no article is loaded yet.
#[derive(Clone, Debug, thiserror::Error)]
pub enum NavError {
    /// The underlying [`ZimArchive`] returned [`ZimError`].
    #[error("zim archive error: {0}")]
    Archive(#[from] ZimError),
    /// The user invoked back/forward/go_to before any article was loaded.
    #[error("no current article to navigate from")]
    NoCurrentArticle,
    /// §7.2 `main_page` returned `Ok(None)` — the archive has no main page.
    #[error("no main page in this archive")]
    NoMainPage,
    /// The user asked to go back/forward but the relevant stack is empty.
    #[error("cannot navigate {which} from this position")]
    AtBoundary {
        /// `"back"` or `"forward"`.
        which: &'static str,
    },
}

/// Persistence failure from a [`ReadingPositionStore`] implementation.
#[derive(Clone, Debug, thiserror::Error)]
pub enum PositionError {
    /// The storage backend reported a write or read failure. The
    /// implementation owns the detail string; no host path is ever
    /// embedded (A-05).
    #[error("storage error: {0}")]
    Storage(String),
}

/// Persistence failure from a [`BookmarkStore`] implementation.
#[derive(Clone, Debug, thiserror::Error)]
pub enum BookmarkError {
    /// The storage backend reported a read or write failure.
    #[error("storage error: {0}")]
    Storage(String),
    /// `remove` was called with an id no current bookmark owns.
    #[error("bookmark not found: {0}")]
    NotFound(String),
}

// ===========================================================================
// Navigation history (pure)
// ===========================================================================

/// In-memory navigation cursor for one reader session. Tracks the current
/// article plus the back and forward stacks the user traverses with
/// explicit commands.
///
/// `current` is `None` until the first successful navigation; from then
/// on it is always `Some`. The stacks hold canonical article keys — the
/// keys returned by [`ZimArchive::article`], never raw link hrefs — so a
/// back/forward step always names an article the archive can serve.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NavHistory {
    back: Vec<ArticleKey>,
    forward: Vec<ArticleKey>,
    current: Option<ArticleKey>,
}

impl NavHistory {
    /// Empty history — no current article, nothing to traverse.
    pub fn new() -> Self {
        Self::default()
    }

    /// The article the user is currently reading, if any.
    pub fn current(&self) -> Option<&ArticleKey> {
        self.current.as_ref()
    }

    /// True iff [`Self::go_back`] would succeed.
    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// True iff [`Self::go_forward`] would succeed.
    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Number of articles behind the cursor. Bounded by user-driven
    /// navigation only — no implicit growth.
    pub fn back_len(&self) -> usize {
        self.back.len()
    }

    /// Number of articles ahead of the cursor.
    pub fn forward_len(&self) -> usize {
        self.forward.len()
    }

    /// Record an explicit user navigation to `target`. The previous
    /// current article (if any) is pushed onto the back stack and the
    /// forward stack is discarded — standard browser semantics, so a new
    /// forward move always forgets any forward history.
    pub fn visit(&mut self, target: ArticleKey) {
        if let Some(prev) = self.current.take() {
            self.back.push(prev);
        }
        self.forward.clear();
        self.current = Some(target);
    }

    /// Replace the current article key in place, without producing a
    /// history entry. Used when the canonical form of the current
    /// article differs from the key the user navigated to (redirect
    /// canonicalization): the user's conceptual position is unchanged,
    /// so the back stack must not grow.
    ///
    /// Returns [`NavError::NoCurrentArticle`] if no article is loaded.
    pub fn replace_current(&mut self, target: ArticleKey) -> Result<(), NavError> {
        match self.current.as_mut() {
            Some(c) => {
                *c = target;
                Ok(())
            }
            None => Err(NavError::NoCurrentArticle),
        }
    }

    /// Move one step back in history. The current article (if any) is
    /// pushed onto the forward stack. Returns the new current article.
    pub fn go_back(&mut self) -> Result<&ArticleKey, NavError> {
        let prev = self
            .back
            .pop()
            .ok_or(NavError::AtBoundary { which: "back" })?;
        if let Some(cur) = self.current.take() {
            self.forward.push(cur);
        }
        self.current = Some(prev);
        // SAFETY-by-construction: we just set current to Some.
        Ok(self
            .current
            .as_ref()
            .expect("current was just assigned Some(prev)"))
    }

    /// Move one step forward in history. The current article (if any) is
    /// pushed onto the back stack. Returns the new current article.
    pub fn go_forward(&mut self) -> Result<&ArticleKey, NavError> {
        let next = self
            .forward
            .pop()
            .ok_or(NavError::AtBoundary { which: "forward" })?;
        if let Some(cur) = self.current.take() {
            self.back.push(cur);
        }
        self.current = Some(next);
        Ok(self
            .current
            .as_ref()
            .expect("current was just assigned Some(next)"))
    }

    /// Reset to the empty session. Used when the archive is closed.
    pub fn clear(&mut self) {
        self.back.clear();
        self.forward.clear();
        self.current = None;
    }
}

// ===========================================================================
// Reader navigation service
// ===========================================================================

/// The §7.2 reader-navigation orchestrator. Owns one navigation session
/// per open archive; the [`ZimArchive`] is borrowed for the session's
/// lifetime so the host can keep the archive open across many nav calls
/// without re-opening it.
pub struct ReaderNav<'a> {
    archive: &'a dyn ZimArchive,
    history: NavHistory,
}

/// Outcome of following an [`InternalLink`]. Only the [`Article`][LinkFollowOutcome::Article]
/// variant mutates history — the other two are read-only inspections of
/// link-referenced content.
#[derive(Clone, Debug)]
pub enum LinkFollowOutcome {
    /// The link pointed at an article; the article body has been fetched
    /// and the navigation cursor advanced.
    Article {
        /// The fetched article body.
        document: ArticleDocument,
    },
    /// The link pointed at a resource (image, stylesheet, etc.). The
    /// resource bytes have been fetched but the cursor has not moved.
    Resource {
        /// The fetched resource body.
        document: ResourceDocument,
    },
    /// The link parser could not classify the href (external URL,
    /// javascript:, mailto:, …). Nothing was fetched; the cursor has not
    /// moved.
    Unsupported {
        /// The reason the link parser left this target as Unsupported.
        reason: String,
    },
}

impl<'a> ReaderNav<'a> {
    /// Open a navigation session over `archive`. The session starts with
    /// no current article; the first call to [`Self::open_main`] or
    /// [`Self::go_to_article`] loads one.
    pub fn new(archive: &'a dyn ZimArchive) -> Self {
        Self {
            archive,
            history: NavHistory::new(),
        }
    }

    /// Read-only view of the navigation cursor.
    pub fn history(&self) -> &NavHistory {
        &self.history
    }

    /// §7.2 main-page entry. Loads the archive's main page and makes it
    /// the current article. History is reset — opening the main page is a
    /// fresh start, not a navigation step, so it does not push the
    /// previous article onto the back stack.
    pub async fn open_main(&mut self) -> Result<ArticleDocument, NavError> {
        let main_key = self
            .archive
            .main_page()
            .await?
            .ok_or(NavError::NoMainPage)?;
        let document = self.archive.article(&main_key).await?;
        self.history.clear();
        // visit() will push any residual current to the back stack; clear()
        // above guarantees that the back stack is empty afterwards.
        self.history.visit(main_key);
        Ok(document)
    }

    /// Navigate to `target` as an explicit user action (history-producing).
    /// Returns the fetched article body. The key recorded in history is
    /// the canonical key from the returned [`ArticleDocument::article_key`]
    /// — if the input key was a redirect entry, history records the
    /// canonical destination, never the redirect, so subsequent back/forward
    /// always names a fetchable article.
    pub async fn go_to_article(
        &mut self,
        target: ArticleKey,
    ) -> Result<ArticleDocument, NavError> {
        let document = self.archive.article(&target).await?;
        self.history.visit(document.article_key.clone());
        Ok(document)
    }

    /// Follow an [`InternalLink`] from the current article.
    ///
    /// - [`InternalLinkTarget::Article`] produces a navigation (the cursor
    ///   advances and the article body is returned).
    /// - [`InternalLinkTarget::Resource`] fetches the resource bytes; the
    ///   cursor does not move.
    /// - [`InternalLinkTarget::Unsupported`] returns the parser's reason
    ///   without touching the network.
    pub async fn follow_link(
        &mut self,
        link: &InternalLink,
    ) -> Result<LinkFollowOutcome, NavError> {
        match &link.target {
            InternalLinkTarget::Article { article_key } => {
                let document = self.go_to_article(article_key.clone()).await?;
                Ok(LinkFollowOutcome::Article { document })
            }
            InternalLinkTarget::Resource { entry_key } => {
                let document = self.archive.resource(entry_key).await?;
                Ok(LinkFollowOutcome::Resource { document })
            }
            InternalLinkTarget::Unsupported { reason } => {
                Ok(LinkFollowOutcome::Unsupported {
                    reason: reason.clone(),
                })
            }
        }
    }

    /// Walk one step back in history and re-fetch that article. Fails
    /// with [`NavError::AtBoundary`] if there is nothing to go back to.
    pub async fn back(&mut self) -> Result<ArticleDocument, NavError> {
        let key = self.history.go_back()?.clone();
        let document = self.archive.article(&key).await?;
        // go_back() already moved the cursor; replace_current keeps
        // history consistent if the archive canonicalized the key.
        let _ = self.history.replace_current(document.article_key.clone());
        Ok(document)
    }

    /// Walk one step forward in history and re-fetch that article. Fails
    /// with [`NavError::AtBoundary`] if there is nothing to go forward to.
    pub async fn forward(&mut self) -> Result<ArticleDocument, NavError> {
        let key = self.history.go_forward()?.clone();
        let document = self.archive.article(&key).await?;
        let _ = self.history.replace_current(document.article_key.clone());
        Ok(document)
    }

    /// Resolve `key` through the archive's redirect chain, returning the
    /// canonical entry key. Useful when the caller holds an [`EntryKey`]
    /// (e.g. from a copy-link action) and wants to know where it
    /// ultimately lands without fetching the body. Honours the
    /// `MAX_REDIRECT_HOPS` cap; the archive rejects cycles before that.
    pub async fn resolve_entry(&self, key: &EntryKey) -> Result<EntryKey, NavError> {
        let resolved = self
            .archive
            .resolve_redirect(key, MAX_REDIRECT_HOPS)
            .await?;
        Ok(resolved)
    }
}

// ===========================================================================
// Title search
// ===========================================================================

/// Normalize a free-text title-search query. Trims leading/trailing
/// whitespace and collapses internal whitespace runs to single spaces.
/// Case, spelling and punctuation are preserved — the archive's own
/// title index sees the user's actual spelling, never a re-spelled
/// approximation. Matches the §11.1 step-1 normalization rule.
pub fn normalize_search_text(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_ws = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out
}

/// One page of title-search results. `has_more` is `true` when the page
/// came back full (`hits.len() == limit`) — a hint that another page is
/// likely available at `offset + hits.len()`. A short page always sets
/// `has_more = false`.
#[derive(Clone, Debug)]
pub struct SearchPage {
    pub hits: Vec<TitleSearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub has_more: bool,
}

/// Run one page of title search against `archive`.
///
/// An empty normalized query returns an empty page (`has_more = false`)
/// rather than an error — the UI uses this to render "type something"
/// guidance, not as a failure. A non-empty query is forwarded to the
/// archive at the requested offset with [`SEARCH_PAGE_SIZE`] as the
/// per-page limit.
pub async fn search_titles(
    archive: &dyn ZimArchive,
    raw_query: &str,
    offset: u32,
) -> Result<SearchPage, NavError> {
    let normalized = normalize_search_text(raw_query);
    if normalized.is_empty() {
        return Ok(SearchPage {
            hits: Vec::new(),
            offset,
            limit: SEARCH_PAGE_SIZE,
            has_more: false,
        });
    }
    let hits = archive
        .search_titles(TitleSearchQuery {
            text: normalized,
            offset,
            limit: SEARCH_PAGE_SIZE,
        })
        .await?;
    let has_more = hits.len() as u32 == SEARCH_PAGE_SIZE;
    Ok(SearchPage {
        hits,
        offset,
        limit: SEARCH_PAGE_SIZE,
        has_more,
    })
}

// ===========================================================================
// Persisted reading position
// ===========================================================================

/// A persisted scroll anchor inside one article. `selector` is a
/// content-derived locator (for example, a heading id or a stable
/// paragraph index produced by the article normalizer) — never a pixel
/// offset, so the saved position survives font-size changes and screen
/// rotations.
///
/// One position per article: saving again for the same `article_key`
/// overwrites the previous value. Positions belong to the user (A-06) —
/// they are never coupled to an evictable archive or model binary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadingPosition {
    pub article_key: ArticleKey,
    pub selector: String,
    pub updated_at_ms: i64,
}

/// Persistence contract for [`ReadingPosition`]. Implementations live in
/// the storage layer (Ring 2) and write to the `reading_positions` table;
/// the application layer only consumes the contract.
#[async_trait]
pub trait ReadingPositionStore: Send + Sync {
    /// Persist `position`. Overwrites any prior position for the same
    /// `article_key`.
    async fn save(&self, position: ReadingPosition) -> Result<(), PositionError>;

    /// Return the saved position for `article_key`, if any.
    async fn load(
        &self,
        article_key: &ArticleKey,
    ) -> Result<Option<ReadingPosition>, PositionError>;

    /// Drop the saved position for `article_key`. Idempotent — clearing
    /// an article with no saved position is `Ok(())`.
    async fn clear(&self, article_key: &ArticleKey) -> Result<(), PositionError>;
}

// ===========================================================================
// Bookmarks
// ===========================================================================

/// A user-created bookmark. A bookmark names a position inside an
/// article (or the article's headline if `selector` is `None`) with a
/// title the user chose at creation time. Bookmarks belong to the user
/// (A-06) and survive archive replacement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub bookmark_id: String,
    pub article_key: ArticleKey,
    pub title: String,
    /// Optional content-derived selector matching [`ReadingPosition::selector`].
    /// `None` means "the article headline".
    pub selector: Option<String>,
    pub created_at_ms: i64,
}

/// Persistence contract for [`Bookmark`]. Implementations live in the
/// storage layer (Ring 2) and write to the `bookmarks` table.
#[async_trait]
pub trait BookmarkStore: Send + Sync {
    /// Persist a new bookmark. The implementation assigns the
    /// `bookmark_id` and returns the stored record.
    async fn add(&self, bookmark: Bookmark) -> Result<Bookmark, BookmarkError>;

    /// Remove the bookmark with this id. Fails with
    /// [`BookmarkError::NotFound`] if no current bookmark owns it.
    async fn remove(&self, bookmark_id: &str) -> Result<(), BookmarkError>;

    /// All bookmarks whose `article_key` matches. Empty `Vec` is a
    /// legitimate result — the article has no bookmarks.
    async fn list_for_article(
        &self,
        article_key: &ArticleKey,
    ) -> Result<Vec<Bookmark>, BookmarkError>;

    /// Every bookmark, newest first. The implementation defines the
    /// global ordering; callers must not assume alphabetical.
    async fn list_all(&self) -> Result<Vec<Bookmark>, BookmarkError>;
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    //! Real assertions over the pure history cursor, query normalization,
    //! search pagination, and the navigation/search flow against an
    //! in-memory `ZimArchive` implementation backed by a HashMap. No
    //! stubs: every trait method of `MapArchive` is a real implementation,
    //! not `todo!()` or `unimplemented!()`.
    use super::*;
    use bytes::Bytes;
    use domain::identity::ContentDigest;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- helpers -------------------------------------------------------

    fn article_key(s: &str) -> ArticleKey {
        ArticleKey(s.to_string())
    }

    fn entry_key(s: &str) -> EntryKey {
        EntryKey(s.to_string())
    }

    fn digest(n: u8) -> ContentDigest {
        ContentDigest::sha256_from_hex(
            std::iter::repeat_n((b'a' + (n % 6)) as char, 64).collect::<String>(),
        )
        .expect("64 lowercase hex chars")
    }

    fn article_doc(key: &str, body: &str, links: Vec<InternalLink>) -> ArticleDocument {
        ArticleDocument {
            article_key: article_key(key),
            title: key.to_string(),
            mime_type: "text/html".to_string(),
            raw_html: Bytes::from_static(b"<html></html>"),
            normalized_text: body.to_string(),
            content_digest: digest(key.as_bytes()[0]),
            internal_links: links,
        }
    }

    fn link_to_article(href: &str, target: &str) -> InternalLink {
        InternalLink {
            href: href.to_string(),
            target: InternalLinkTarget::Article {
                article_key: article_key(target),
            },
        }
    }

    fn link_to_resource(href: &str, target: &str) -> InternalLink {
        InternalLink {
            href: href.to_string(),
            target: InternalLinkTarget::Resource {
                entry_key: entry_key(target),
            },
        }
    }

    fn link_unsupported(href: &str, reason: &str) -> InternalLink {
        InternalLink {
            href: href.to_string(),
            target: InternalLinkTarget::Unsupported {
                reason: reason.to_string(),
            },
        }
    }

    // ---- in-memory ZimArchive implementation ---------------------------

    /// Real, complete `ZimArchive` impl backed by HashMaps. Used to test
    /// the navigation and search flows without standing up the AnZimmerman
    /// provider. Every method is implemented — no `todo!`.
    struct MapArchive {
        metadata: ports::zim::ZimMetadata,
        main_page: Option<ArticleKey>,
        articles: HashMap<ArticleKey, ArticleDocument>,
        resources: HashMap<EntryKey, ResourceDocument>,
        redirects: HashMap<EntryKey, EntryKey>,
        titles: Vec<(ArticleKey, String)>,
    }

    impl MapArchive {
        fn builder() -> MapArchiveBuilder {
            MapArchiveBuilder {
                main_page: None,
                articles: Vec::new(),
                resources: Vec::new(),
                redirects: Vec::new(),
            }
        }
    }

    struct MapArchiveBuilder {
        main_page: Option<ArticleKey>,
        articles: Vec<(ArticleKey, ArticleDocument)>,
        resources: Vec<(EntryKey, ResourceDocument)>,
        redirects: Vec<(EntryKey, EntryKey)>,
    }

    impl MapArchiveBuilder {
        fn main_page(mut self, key: &str) -> Self {
            self.main_page = Some(article_key(key));
            self
        }
        fn article(mut self, key: &str, body: &str, links: Vec<InternalLink>) -> Self {
            self.articles.push((article_key(key), article_doc(key, body, links)));
            self
        }
        fn resource(mut self, key: &str, bytes: &'static [u8], mime: &str) -> Self {
            self.resources.push((
                entry_key(key),
                ResourceDocument {
                    entry_key: entry_key(key),
                    mime_type: mime.to_string(),
                    bytes: Bytes::from_static(bytes),
                    content_digest: digest(key.as_bytes()[0]),
                },
            ));
            self
        }
        fn redirect(mut self, from: &str, to: &str) -> Self {
            self.redirects.push((entry_key(from), entry_key(to)));
            self
        }
        fn build(self) -> MapArchive {
            let article_count = self.articles.len() as u64;
            let mut articles = HashMap::new();
            let mut titles = Vec::with_capacity(self.articles.len());
            for (k, doc) in self.articles {
                titles.push((k.clone(), doc.title.clone()));
                articles.insert(k, doc);
            }
            let resources: HashMap<_, _> = self.resources.into_iter().collect();
            let redirects: HashMap<_, _> = self.redirects.into_iter().collect();
            let media_count = resources.len() as u64;
            let main_page = self.main_page.clone();
            let title = "MapArchive".to_string();
            MapArchive {
                metadata: ports::zim::ZimMetadata {
                    uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                    title,
                    description: String::new(),
                    language_bcp47: "en".to_string(),
                    creator: "test".to_string(),
                    publisher: "test".to_string(),
                    date: "2026-01-01".to_string(),
                    article_count,
                    media_count,
                    main_page,
                    checksum: None,
                },
                main_page: self.main_page,
                articles,
                resources,
                redirects,
                titles,
            }
        }
    }

    #[async_trait]
    impl ZimArchive for MapArchive {
        fn metadata(&self) -> &ports::zim::ZimMetadata {
            &self.metadata
        }
        async fn main_page(&self) -> Result<Option<ArticleKey>, ZimError> {
            Ok(self.main_page.clone())
        }
        async fn entry(
            &self,
            key: &EntryKey,
        ) -> Result<ports::zim::ZimEntryDescriptor, ZimError> {
            // Articles first
            let article_key_form = ArticleKey(key.0.clone());
            if self.articles.contains_key(&article_key_form) {
                return Ok(ports::zim::ZimEntryDescriptor {
                    entry_key: key.clone(),
                    namespace: 'A',
                    path: key.0.clone(),
                    title: article_key_form.0.clone(),
                    mime_type: "text/html".to_string(),
                    kind: ports::zim::ZimEntryKind::Article,
                    redirect_target: None,
                    byte_length: 0,
                });
            }
            if self.resources.contains_key(key) {
                return Ok(ports::zim::ZimEntryDescriptor {
                    entry_key: key.clone(),
                    namespace: 'I',
                    path: key.0.clone(),
                    title: key.0.clone(),
                    mime_type: self
                        .resources
                        .get(key)
                        .map(|r| r.mime_type.clone())
                        .unwrap_or_default(),
                    kind: ports::zim::ZimEntryKind::Resource,
                    redirect_target: None,
                    byte_length: 0,
                });
            }
            if self.redirects.contains_key(key) {
                return Ok(ports::zim::ZimEntryDescriptor {
                    entry_key: key.clone(),
                    namespace: 'A',
                    path: key.0.clone(),
                    title: key.0.clone(),
                    mime_type: String::new(),
                    kind: ports::zim::ZimEntryKind::Redirect,
                    redirect_target: self.redirects.get(key).cloned(),
                    byte_length: 0,
                });
            }
            Err(ZimError::EntryNotFound(key.0.clone()))
        }
        async fn article(&self, key: &ArticleKey) -> Result<ArticleDocument, ZimError> {
            self.articles
                .get(key)
                .cloned()
                .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))
        }
        async fn resource(&self, key: &EntryKey) -> Result<ResourceDocument, ZimError> {
            self.resources
                .get(key)
                .cloned()
                .ok_or_else(|| ZimError::EntryNotFound(key.0.clone()))
        }
        async fn resolve_redirect(
            &self,
            key: &EntryKey,
            max_hops: u8,
        ) -> Result<EntryKey, ZimError> {
            let mut current = key.clone();
            for _ in 0..max_hops {
                match self.redirects.get(&current) {
                    Some(next) => {
                        if next == key {
                            return Err(ZimError::RedirectCycle);
                        }
                        current = next.clone();
                    }
                    None => return Ok(current),
                }
            }
            Err(ZimError::RedirectCycle)
        }
        async fn search_titles(
            &self,
            query: TitleSearchQuery,
        ) -> Result<Vec<TitleSearchHit>, ZimError> {
            let q = query.text.to_lowercase();
            let mut scored: Vec<TitleSearchHit> = self
                .titles
                .iter()
                .filter_map(|(k, title)| {
                    let lower = title.to_lowercase();
                    let idx = lower.find(&q)?;
                    // Exact prefix match scores 1.0, substring elsewhere
                    // scores 0.5. The point is a deterministic ordering
                    // that respects "prefix beats infix"; the precise
                    // numbers do not matter to the application layer.
                    let score = if idx == 0 { 1.0 } else { 0.5 };
                    Some(TitleSearchHit {
                        article_key: k.clone(),
                        title: title.clone(),
                        score,
                    })
                })
                .collect();
            scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            let start = (query.offset as usize).min(scored.len());
            let end = start.saturating_add(query.limit as usize).min(scored.len());
            Ok(scored[start..end].to_vec())
        }
        async fn entries(&self) -> Result<ports::zim::ZimEntryStream, ZimError> {
            let items: Vec<Result<ports::zim::ZimEntryDescriptor, ZimError>> = self
                .titles
                .iter()
                .map(|(k, _)| ports::zim::ZimEntryDescriptor {
                    entry_key: EntryKey(k.0.clone()),
                    namespace: 'A',
                    path: k.0.clone(),
                    title: k.0.clone(),
                    mime_type: "text/html".to_string(),
                    kind: ports::zim::ZimEntryKind::Article,
                    redirect_target: None,
                    byte_length: 0,
                })
                .map(Ok)
                .collect();
            let stream = futures_util::stream::iter(items);
            Ok(Box::pin(stream))
        }
        async fn verify_structure(&self) -> Result<ports::zim::ZimVerificationReport, ZimError> {
            Ok(ports::zim::ZimVerificationReport {
                valid: true,
                engine_id: "map.test.v1".to_string(),
                engine_version: "0".to_string(),
                checked_entries: self.titles.len() as u64,
                warnings: Vec::new(),
                errors: Vec::new(),
            })
        }
    }

    // ---- pure NavHistory tests -----------------------------------------

    #[test]
    fn empty_history_has_no_current_and_no_stacks() {
        let h = NavHistory::new();
        assert!(h.current().is_none());
        assert!(!h.can_go_back());
        assert!(!h.can_go_forward());
        assert_eq!(h.back_len(), 0);
        assert_eq!(h.forward_len(), 0);
    }

    #[test]
    fn first_visit_sets_current_without_back_history() {
        let mut h = NavHistory::new();
        h.visit(article_key("A"));
        assert_eq!(h.current(), Some(&article_key("A")));
        assert!(!h.can_go_back());
        assert_eq!(h.back_len(), 0);
    }

    #[test]
    fn second_visit_pushes_previous_to_back_and_clears_forward() {
        let mut h = NavHistory::new();
        h.visit(article_key("A"));
        h.visit(article_key("B"));
        assert_eq!(h.current(), Some(&article_key("B")));
        assert_eq!(h.back_len(), 1);
        assert_eq!(h.back, vec![article_key("A")]);
        // Now go back, then visit a new article — forward must clear.
        h.go_back().unwrap();
        assert_eq!(h.current(), Some(&article_key("A")));
        assert!(h.can_go_forward());
        h.visit(article_key("C"));
        assert!(!h.can_go_forward(), "forward stack must be cleared by visit");
        assert_eq!(h.forward_len(), 0);
    }

    #[test]
    fn go_back_then_forward_roundtrips_cursor() {
        let mut h = NavHistory::new();
        h.visit(article_key("A"));
        h.visit(article_key("B"));
        h.visit(article_key("C"));

        assert_eq!(h.current(), Some(&article_key("C")));
        assert_eq!(h.go_back().unwrap(), &article_key("B"));
        assert_eq!(h.go_back().unwrap(), &article_key("A"));

        // At the bottom of back stack.
        let err = h.go_back().unwrap_err();
        assert!(matches!(err, NavError::AtBoundary { which: "back" }));

        assert_eq!(h.go_forward().unwrap(), &article_key("B"));
        assert_eq!(h.go_forward().unwrap(), &article_key("C"));

        let err = h.go_forward().unwrap_err();
        assert!(matches!(err, NavError::AtBoundary { which: "forward" }));
    }

    #[test]
    fn replace_current_requires_loaded_article() {
        let mut h = NavHistory::new();
        let err = h.replace_current(article_key("X")).unwrap_err();
        assert!(matches!(err, NavError::NoCurrentArticle));

        h.visit(article_key("A"));
        h.replace_current(article_key("A_canonical")).unwrap();
        assert_eq!(h.current(), Some(&article_key("A_canonical")));
        // No back-stack growth — replace is in place.
        assert_eq!(h.back_len(), 0);
    }

    #[test]
    fn clear_resets_to_empty() {
        let mut h = NavHistory::new();
        h.visit(article_key("A"));
        h.visit(article_key("B"));
        h.go_back().unwrap();
        assert!(h.can_go_forward() || h.can_go_back());
        h.clear();
        assert!(h.current().is_none());
        assert!(!h.can_go_back());
        assert!(!h.can_go_forward());
    }

    // ---- normalize_search_text ----------------------------------------

    #[test]
    fn normalize_search_text_trims_and_collapses_whitespace() {
        assert_eq!(normalize_search_text(""), "");
        assert_eq!(normalize_search_text("   "), "");
        assert_eq!(normalize_search_text("  PZN  "), "PZN");
        assert_eq!(normalize_search_text("  Packet   Zone\nNetwork  "), "Packet Zone Network");
        // Case and punctuation preserved.
        assert_eq!(normalize_search_text("What is PZN?"), "What is PZN?");
    }

    // ---- search_titles empty-query ------------------------------------

    #[tokio::test]
    async fn empty_query_returns_empty_page_not_error() {
        let archive = MapArchive::builder()
            .article("Alpha", "body", vec![])
            .build();
        let page = search_titles(&archive, "   ", 0).await.expect("empty is ok");
        assert!(page.hits.is_empty());
        assert!(!page.has_more);
        assert_eq!(page.offset, 0);
        assert_eq!(page.limit, SEARCH_PAGE_SIZE);
    }

    #[tokio::test]
    async fn prefix_match_beats_infix_match() {
        let archive = MapArchive::builder()
            .article("Packet", "body", vec![])
            .article("Packet Zone Network", "body", vec![])
            .article("Zone Packet", "body", vec![])
            .build();
        let page = search_titles(&archive, "Packet", 0).await.unwrap();
        assert_eq!(page.hits.len(), 3);
        // Two prefix matches score 1.0 and beat the infix match.
        assert!(page.hits[0].score > page.hits[2].score);
        assert!(page.hits[0].title.starts_with("Packet"));
        assert!(page.hits[1].title.starts_with("Packet"));
        assert_eq!(page.hits[2].title, "Zone Packet");
    }

    #[tokio::test]
    async fn unknown_query_returns_empty_page() {
        let archive = MapArchive::builder()
            .article("Alpha", "body", vec![])
            .build();
        let page = search_titles(&archive, "zzz", 0).await.unwrap();
        assert!(page.hits.is_empty());
        assert!(!page.has_more);
    }

    // ---- ReaderNav -----------------------------------------------------

    fn three_page_archive() -> MapArchive {
        MapArchive::builder()
            .main_page("Home")
            .article(
                "Home",
                "welcome",
                vec![
                    link_to_article("A/Chapter1", "Chapter1"),
                    link_to_resource("I/cover.png", "cover.png"),
                    link_unsupported("https://example.invalid/", "external"),
                ],
            )
            .article("Chapter1", "chapter one body", vec![])
            .resource("cover.png", b"\x89PNG\x00", "image/png")
            .build()
    }

    #[tokio::test]
    async fn open_main_loads_main_and_resets_history() {
        let archive = three_page_archive();
        let mut nav = ReaderNav::new(&archive);
        // Pre-dirt the history to prove open_main clears it.
        nav.history.visit(article_key("Chapter1"));
        let doc = nav.open_main().await.expect("main page loads");
        assert_eq!(doc.article_key, article_key("Home"));
        assert_eq!(nav.history().current(), Some(&article_key("Home")));
        assert_eq!(nav.history().back_len(), 0);
        assert!(!nav.history().can_go_back());
    }

    #[tokio::test]
    async fn open_main_without_main_page_errors_no_main_page() {
        let archive = MapArchive::builder()
            .article("Orphan", "body", vec![])
            .build();
        let mut nav = ReaderNav::new(&archive);
        let err = nav.open_main().await.unwrap_err();
        assert!(matches!(err, NavError::NoMainPage));
    }

    #[tokio::test]
    async fn follow_link_to_article_advances_history() {
        let archive = three_page_archive();
        let mut nav = ReaderNav::new(&archive);
        nav.open_main().await.unwrap();
        let link = link_to_article("A/Chapter1", "Chapter1");
        let outcome = nav.follow_link(&link).await.expect("article link");
        match outcome {
            LinkFollowOutcome::Article { document } => {
                assert_eq!(document.article_key, article_key("Chapter1"));
            }
            _ => panic!("expected Article outcome"),
        }
        assert_eq!(nav.history().current(), Some(&article_key("Chapter1")));
        assert!(nav.history().can_go_back());
        assert_eq!(nav.history().back, vec![article_key("Home")]);
    }

    #[tokio::test]
    async fn follow_link_to_resource_does_not_move_cursor() {
        let archive = three_page_archive();
        let mut nav = ReaderNav::new(&archive);
        nav.open_main().await.unwrap();
        let link = link_to_resource("I/cover.png", "cover.png");
        let outcome = nav.follow_link(&link).await.expect("resource link");
        match outcome {
            LinkFollowOutcome::Resource { document } => {
                assert_eq!(document.entry_key, entry_key("cover.png"));
                assert_eq!(document.mime_type, "image/png");
            }
            _ => panic!("expected Resource outcome"),
        }
        // Cursor stayed on Home.
        assert_eq!(nav.history().current(), Some(&article_key("Home")));
        assert!(!nav.history().can_go_back());
    }

    #[tokio::test]
    async fn follow_unsupported_link_does_not_fetch() {
        let archive = three_page_archive();
        let mut nav = ReaderNav::new(&archive);
        nav.open_main().await.unwrap();
        let link = link_unsupported("https://example.invalid/", "external");
        let outcome = nav.follow_link(&link).await.unwrap();
        match outcome {
            LinkFollowOutcome::Unsupported { reason } => {
                assert_eq!(reason, "external");
            }
            _ => panic!("expected Unsupported outcome"),
        }
        assert_eq!(nav.history().current(), Some(&article_key("Home")));
    }

    #[tokio::test]
    async fn back_and_forward_roundtrip_refetches_articles() {
        let archive = three_page_archive();
        let mut nav = ReaderNav::new(&archive);
        nav.open_main().await.unwrap();
        nav.follow_link(&link_to_article("A/Chapter1", "Chapter1"))
            .await
            .unwrap();
        assert_eq!(nav.history().current(), Some(&article_key("Chapter1")));

        let back_doc = nav.back().await.expect("back to Home");
        assert_eq!(back_doc.article_key, article_key("Home"));
        assert_eq!(nav.history().current(), Some(&article_key("Home")));
        assert!(nav.history().can_go_forward());

        let fwd_doc = nav.forward().await.expect("forward to Chapter1");
        assert_eq!(fwd_doc.article_key, article_key("Chapter1"));
    }

    #[tokio::test]
    async fn back_at_boundary_returns_at_boundary_error() {
        let archive = three_page_archive();
        let mut nav = ReaderNav::new(&archive);
        nav.open_main().await.unwrap();
        let err = nav.back().await.unwrap_err();
        assert!(matches!(err, NavError::AtBoundary { which: "back" }));
    }

    #[tokio::test]
    async fn resolve_entry_follows_redirect_chain() {
        let archive = MapArchive::builder()
            .redirect("Old", "Mid")
            .redirect("Mid", "New")
            .article("New", "body", vec![])
            .build();
        let nav = ReaderNav::new(&archive);
        let resolved = nav
            .resolve_entry(&entry_key("Old"))
            .await
            .expect("chain resolves");
        assert_eq!(resolved, entry_key("New"));
    }

    #[tokio::test]
    async fn resolve_entry_detects_cycle() {
        let archive = MapArchive::builder()
            .redirect("A", "B")
            .redirect("B", "A")
            .article("A", "body", vec![])
            .article("B", "body", vec![])
            .build();
        let nav = ReaderNav::new(&archive);
        let err = nav.resolve_entry(&entry_key("A")).await.unwrap_err();
        assert!(matches!(err, NavError::Archive(ZimError::RedirectCycle)));
    }

    // ---- persistence traits (in-memory impls used by UI/tests) --------

    /// In-memory `ReadingPositionStore` for tests. Real implementation,
    /// not a stub — every method writes to the map.
    struct MapPositionStore {
        inner: Mutex<HashMap<ArticleKey, ReadingPosition>>,
    }

    impl MapPositionStore {
        fn new() -> Self {
            Self {
                inner: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ReadingPositionStore for MapPositionStore {
        async fn save(&self, position: ReadingPosition) -> Result<(), PositionError> {
            self.inner
                .lock()
                .expect("pos lock")
                .insert(position.article_key.clone(), position);
            Ok(())
        }
        async fn load(
            &self,
            article_key: &ArticleKey,
        ) -> Result<Option<ReadingPosition>, PositionError> {
            Ok(self
                .inner
                .lock()
                .expect("pos lock")
                .get(article_key)
                .cloned())
        }
        async fn clear(&self, article_key: &ArticleKey) -> Result<(), PositionError> {
            self.inner.lock().expect("pos lock").remove(article_key);
            Ok(())
        }
    }

    #[tokio::test]
    async fn reading_position_save_overwrites_prior_for_same_article() {
        let store = MapPositionStore::new();
        let key = article_key("A");
        store
            .save(ReadingPosition {
                article_key: key.clone(),
                selector: "para-1".to_string(),
                updated_at_ms: 1_000,
            })
            .await
            .unwrap();
        store
            .save(ReadingPosition {
                article_key: key.clone(),
                selector: "para-9".to_string(),
                updated_at_ms: 2_000,
            })
            .await
            .unwrap();
        let loaded = store.load(&key).await.unwrap().expect("present");
        assert_eq!(loaded.selector, "para-9");
        assert_eq!(loaded.updated_at_ms, 2_000);

        store.clear(&key).await.unwrap();
        assert!(store.load(&key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn reading_position_clear_when_absent_is_idempotent() {
        let store = MapPositionStore::new();
        store.clear(&article_key("never")).await.unwrap();
    }

    /// In-memory `BookmarkStore` for tests. Mirrors what the storage
    /// layer does: assigns a monotonically increasing id on add.
    struct MapBookmarkStore {
        next_id: Mutex<u64>,
        rows: Mutex<Vec<Bookmark>>,
    }

    impl MapBookmarkStore {
        fn new() -> Self {
            Self {
                next_id: Mutex::new(1),
                rows: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl BookmarkStore for MapBookmarkStore {
        async fn add(&self, mut bookmark: Bookmark) -> Result<Bookmark, BookmarkError> {
            let id = {
                let mut g = self.next_id.lock().expect("bm id lock");
                let v = *g;
                *g += 1;
                v
            };
            bookmark.bookmark_id = format!("bm.{id}");
            self.rows.lock().expect("bm rows lock").push(bookmark.clone());
            Ok(bookmark)
        }
        async fn remove(&self, bookmark_id: &str) -> Result<(), BookmarkError> {
            let mut rows = self.rows.lock().expect("bm rows lock");
            let before = rows.len();
            rows.retain(|b| b.bookmark_id != bookmark_id);
            if rows.len() == before {
                return Err(BookmarkError::NotFound(bookmark_id.to_string()));
            }
            Ok(())
        }
        async fn list_for_article(
            &self,
            article_key: &ArticleKey,
        ) -> Result<Vec<Bookmark>, BookmarkError> {
            Ok(self
                .rows
                .lock()
                .expect("bm rows lock")
                .iter()
                .filter(|b| &b.article_key == article_key)
                .cloned()
                .collect())
        }
        async fn list_all(&self) -> Result<Vec<Bookmark>, BookmarkError> {
            // newest first: higher created_at_ms first
            let mut rows = self.rows.lock().expect("bm rows lock").clone();
            rows.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
            Ok(rows)
        }
    }

    #[tokio::test]
    async fn bookmark_add_assigns_id_and_persists() {
        let store = MapBookmarkStore::new();
        let stored = store
            .add(Bookmark {
                bookmark_id: String::new(), // assigned by store
                article_key: article_key("A"),
                title: "First".to_string(),
                selector: Some("p-1".to_string()),
                created_at_ms: 1_000,
            })
            .await
            .unwrap();
        assert_eq!(stored.bookmark_id, "bm.1");
        assert_eq!(stored.title, "First");

        let stored2 = store
            .add(Bookmark {
                bookmark_id: String::new(),
                article_key: article_key("A"),
                title: "Second".to_string(),
                selector: None,
                created_at_ms: 2_000,
            })
            .await
            .unwrap();
        assert_eq!(stored2.bookmark_id, "bm.2");

        let for_a = store.list_for_article(&article_key("A")).await.unwrap();
        assert_eq!(for_a.len(), 2);

        let all = store.list_all().await.unwrap();
        assert_eq!(all[0].created_at_ms, 2_000);
        assert_eq!(all[1].created_at_ms, 1_000);

        store.remove("bm.1").await.unwrap();
        assert_eq!(store.list_all().await.unwrap().len(), 1);

        let err = store.remove("bm.1").await.unwrap_err();
        assert!(matches!(err, BookmarkError::NotFound(_)));
    }

    #[tokio::test]
    async fn bookmark_list_for_article_empty_when_no_match() {
        let store = MapBookmarkStore::new();
        let rows = store.list_for_article(&article_key("X")).await.unwrap();
        assert!(rows.is_empty());
    }

    // ---- end-to-end: open article, save+restore position, bookmark ----

    #[tokio::test]
    async fn reading_session_round_trip() {
        let archive = three_page_archive();
        let positions: std::sync::Arc<MapPositionStore> = std::sync::Arc::new(MapPositionStore::new());
        let bookmarks: std::sync::Arc<MapBookmarkStore> = std::sync::Arc::new(MapBookmarkStore::new());

        let mut nav = ReaderNav::new(&archive);
        nav.open_main().await.unwrap();
        nav.follow_link(&link_to_article("A/Chapter1", "Chapter1"))
            .await
            .unwrap();

        let cur = nav.history().current().expect("on Chapter1").clone();
        positions
            .save(ReadingPosition {
                article_key: cur.clone(),
                selector: "heading-2".to_string(),
                updated_at_ms: 9_999,
            })
            .await
            .unwrap();

        bookmarks
            .add(Bookmark {
                bookmark_id: String::new(),
                article_key: cur.clone(),
                title: "Chapter 1 — heading 2".to_string(),
                selector: Some("heading-2".to_string()),
                created_at_ms: 9_999,
            })
            .await
            .unwrap();

        // Simulate a session restart: rebuild nav, navigate explicitly to
        // the previously-saved article, and hydrate position+bookmarks.
        let mut nav2 = ReaderNav::new(&archive);
        nav2.go_to_article(cur.clone()).await.unwrap();
        let restored_pos = positions.load(&cur).await.unwrap().expect("position present");
        assert_eq!(restored_pos.selector, "heading-2");
        let restored_bms = bookmarks.list_for_article(&cur).await.unwrap();
        assert_eq!(restored_bms.len(), 1);
        assert_eq!(restored_bms[0].title, "Chapter 1 — heading 2");
    }
}
