//! Build script for the EnZIME reader shell.
//!
//! Compiles the mandatory shared-resource storage policy into the binary:
//! the six `.env` keys below are read from the repository-root `.env` and the
//! crate-local `.env` (process environment wins over both files), validated,
//! and emitted as `rustc-env` so `shared_storage.rs` reads them with `env!()`
//! at compile time. Optional roots may be empty (platform defaults / no
//! configured card); a storage scope other than `common` fails the build.

use std::path::PathBuf;

/// The only `.env` keys this build consumes. Nothing else is forwarded.
const STORAGE_ENV_KEYS: [&str; 6] = [
    "VITE_APP_NAMESPACE",
    "VITE_STORAGE_SCOPE",
    "ENZIME_STORAGE_NAMESPACE",
    "ENZIME_SHARED_STORAGE_ROOT",
    "ENZIME_SHARED_STORAGE_ROOT_ANDROID",
    "ENZIME_SHARED_SDCARD_ROOT",
];

/// Fixed defaults; optional roots default to empty (platform default).
const STORAGE_ENV_DEFAULTS: [(&str, &str); 3] = [
    ("VITE_APP_NAMESPACE", "mba.robin.enzime"),
    ("VITE_STORAGE_SCOPE", "common"),
    ("ENZIME_STORAGE_NAMESPACE", "mba.robin"),
];

/// Minimal dotenv parse: `KEY=VALUE` lines, `#` comments, no interpolation.
fn parse_env_file(bytes: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(bytes);
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = match line.split_once('=') {
            Some((key, value)) => (key.trim(), value.trim().trim_matches('"')),
            None => continue,
        };
        if key.is_empty() {
            continue;
        }
        out.push((key.to_string(), value.to_string()));
    }
    out
}

fn main() {
    // Repository root (three levels above this crate) and the crate itself.
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let env_files = [
        manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|root| root.join(".env")),
        Some(manifest_dir.join(".env")),
    ];
    let mut file_values: Vec<(String, String)> = Vec::new();
    for file in &env_files {
        let Some(file) = file else { continue };
        // Track both known .env paths even while absent, so creating one
        // later triggers recompilation.
        println!("cargo:rerun-if-changed={}", file.display());
        if let Ok(bytes) = std::fs::read(file) {
            file_values.extend(parse_env_file(&bytes));
        }
    }
    for key in STORAGE_ENV_KEYS {
        println!("cargo:rerun-if-env-changed={key}");
    }
    for key in STORAGE_ENV_KEYS {
        // Process environment wins, then files, then the fixed default.
        let value = std::env::var(key)
            .ok()
            .or_else(|| {
                file_values
                    .iter()
                    .find(|(file_key, _)| file_key == key)
                    .map(|(_, value)| value.clone())
            })
            .unwrap_or_else(|| {
                STORAGE_ENV_DEFAULTS
                    .iter()
                    .find(|(default_key, _)| *default_key == key)
                    .map(|(_, value)| value.to_string())
                    .unwrap_or_default()
            });
        if key == "VITE_STORAGE_SCOPE" && value != "common" {
            panic!(
                "VITE_STORAGE_SCOPE must be 'common': large resources are shared by \
                 design and a private storage mode is not permitted in this build \
                 (found '{value}')"
            );
        }
        println!("cargo:rustc-env={key}={value}");
    }
    tauri_build::build()
}
