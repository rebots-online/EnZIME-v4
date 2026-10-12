//! §7.4 conformance fixtures, executed as real integration assertions.
//!
//! Each test name matches one fixture ID from ARCHITECTURE.md §7.4. The
//! `support::ZimBuilder` writes genuine v6 ZIM bytes; the assertions
//! exercise the engine against the resulting file.

mod support;

use std::sync::Arc;

use anzimmerman::model::{TitleSearchQuery, ZimArchive, ZimEngine, ZimError, ZimOpenRequest};
use anzimmerman::{AnZimmermanZimEngine, ENGINE_ID};
use support::{ZimBuilder, NO_PAGE, ZIM_MAGIC};

async fn open(path: &str) -> Arc<dyn ZimArchive> {
    let req = ZimOpenRequest {
        path: path.to_string(),
        expected_sha256: None,
        expected_size_bytes: None,
    };
    AnZimmermanZimEngine::new()
        .open(req)
        .await
        .expect("open archive")
}

fn minimal_html_article(title: &str, body: &str) -> Vec<u8> {
    format!(
        "<html><head><title>{}</title></head><body>{}</body></html>",
        title, body
    )
    .into_bytes()
}

#[path = "conformance/cases_1.rs"]
mod cases_1;
#[path = "conformance/cases_2.rs"]
mod cases_2;
#[path = "conformance/cases_3.rs"]
mod cases_3;

fn read_u64_le(buf: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(buf[at..at + 8].try_into().unwrap())
}
