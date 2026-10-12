//! Deterministically regenerate the portable conformance fixture (explicit tooling only).
#[path = "../tests/support/mod.rs"]
#[allow(dead_code)]
mod support;
use sha2::{Digest, Sha256};
fn main() {
    let mut builder = support::ZimBuilder::default()
        .add_article(
            b'A',
            "Café",
            "Café",
            "text/html",
            b"<p>Library fixture</p>".to_vec(),
        )
        .add_redirect(b'A', "Home", "Home", 0)
        .add_article(
            b'I',
            "pixel.txt",
            "pixel.txt",
            "text/plain",
            b"pixel".to_vec(),
        );
    builder.main_page_index = Some(1);
    let mut bytes = Vec::new();
    builder.write(&mut bytes);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("rust-consolidation.zim"), &bytes).unwrap();
    let expected = serde_json::json!({
        "sha256": hex::encode(Sha256::digest(&bytes)), "uuid": "00000000000000000000000000000000",
        "main_page": "A/Home", "entries": [
            {"key":"A/Café","title":"Café","mime":"text/html","payload":"<p>Library fixture</p>"},
            {"key":"A/Home","title":"Home","mime":"","redirect":"A/Café"},
            {"key":"I/pixel.txt","title":"pixel.txt","mime":"text/plain","payload":"pixel"}
        ]
    });
    std::fs::write(
        root.join("rust-consolidation.expected.json"),
        serde_json::to_string_pretty(&expected).unwrap() + "\n",
    )
    .unwrap();
}
