//! Binary entry for `contract-gen`, the §24.1 entity-to-UI binding
//! validator (ARCHITECTURE.md §26 AMD-4: local script, not CI).
//!
//! Usage:
//!
//! ```text
//! contract-gen validate \
//!     --rust-registry <PATH>   \
//!     --ts-entities <PATH>     \
//!     --sqlite-schema <PATH>   \
//!     --stores <PATH>          \
//!     [--screen-manifest <PATH>]
//! ```
//!
//! Exit status is `0` when §24.1 reports no drift, `1` on any diagnostic,
//! and `2` on CLI or input-read errors.

use std::process::ExitCode;

use contract_gen::{
    diagnostic::{load_inputs, parse_args},
    validate,
};

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let loaded = match load_inputs(&args) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let inputs = validate::Inputs {
        rust_registry: &loaded.rust_registry,
        ts_entities: &loaded.ts_entities,
        sqlite_schema: &loaded.sqlite_schema,
        store_interfaces: &loaded.store_interfaces,
        screen_manifest: loaded.screen_manifest.as_deref(),
    };
    let diagnostics = validate::validate(&inputs);
    if diagnostics.is_empty() {
        let rows = contract_gen::spec::BINDING_TABLE.len();
        println!(
            "contract-gen: {rows} §24 binding rows verified — no §24.1 drift"
        );
        return ExitCode::SUCCESS;
    }
    for d in &diagnostics {
        eprintln!("{d}");
    }
    eprintln!("contract-gen: {} §24.1 failure(s)", diagnostics.len());
    ExitCode::FAILURE
}
