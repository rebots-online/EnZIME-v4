//! CLI argument parsing and input loading for `contract-gen`.
//!
//! Kept minimal — the §26 AMD-4 contract is "a local script the operator
//! runs by hand". The CLI takes four required file paths (one per §24.1
//! input that always exists) and one optional path (the Stitch
//! screen-manifest, which is consulted only when supplied). It reads
//! them, hands their concatenated contents to
//! [`crate::validate::validate`], prints any diagnostics, and exits
//! non-zero when any §24.1 rule fires.

use std::path::PathBuf;

/// Errors produced by argument parsing or input loading. Each variant
/// carries enough context for the operator to correct the invocation
/// without consulting a separate log.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(
        "usage: contract-gen validate \
         --rust-registry <PATH> \
         --ts-entities <PATH> \
         --sqlite-schema <PATH> \
         --stores <PATH> \
         [--screen-manifest <PATH>]\n  \
         note: every required §24.1 input must be readable; the validator does not guess"
    )]
    Usage,

    #[error("missing required flag: --{0}")]
    MissingFlag(&'static str),

    #[error("flag --{flag} expects a path, got `{value}`")]
    BadValue { flag: &'static str, value: String },

    #[error("could not read --{flag} path `{path}`: {err}")]
    Read {
        flag: &'static str,
        path: PathBuf,
        err: String,
    },
}

/// Parsed command-line arguments.
#[derive(Clone, Debug, Default)]
pub struct Args {
    pub rust_registry: PathBuf,
    pub ts_entities: PathBuf,
    pub sqlite_schema: PathBuf,
    pub stores: PathBuf,
    pub screen_manifest: Option<PathBuf>,
}

/// Parse `argv[1..]` into [`Args`] or return a [`CliError`].
///
/// Hand-rolled: the five-flag surface is too small to pull in `clap`, and
/// keeping the dependency list short keeps the local-script invocation
/// snappy (§26 AMD-4).
pub fn parse_args<I: IntoIterator<Item = String>>(argv: I) -> Result<Args, CliError> {
    let mut slots: [Option<PathBuf>; 5] = Default::default();
    // 0=rust_registry, 1=ts_entities, 2=sqlite_schema, 3=stores, 4=screen_manifest
    let mut it = argv.into_iter().peekable();
    while let Some(arg) = it.next() {
        let slot: Option<usize> = match arg.as_str() {
            "validate" => None,           // subcommand selector, implicit
            "--help" | "-h" => return Err(CliError::Usage),
            "--rust-registry" => Some(0),
            "--ts-entities" => Some(1),
            "--sqlite-schema" => Some(2),
            "--stores" => Some(3),
            "--screen-manifest" => Some(4),
            other => {
                return Err(CliError::BadValue {
                    flag: "unknown",
                    value: other.to_string(),
                });
            }
        };
        if let Some(idx) = slot {
            let flag = match idx {
                0 => "--rust-registry",
                1 => "--ts-entities",
                2 => "--sqlite-schema",
                3 => "--stores",
                4 => "--screen-manifest",
                _ => unreachable!("slot index out of range"),
            };
            let v = it.next().ok_or(CliError::MissingFlag(flag))?;
            if v.starts_with("--") {
                return Err(CliError::BadValue { flag, value: v });
            }
            slots[idx] = Some(PathBuf::from(v));
        }
    }
    let flag_name = |idx: usize| -> &'static str {
        match idx {
            0 => "--rust-registry",
            1 => "--ts-entities",
            2 => "--sqlite-schema",
            3 => "--stores",
            _ => unreachable!(),
        }
    };
    let mut required: Vec<PathBuf> = Vec::with_capacity(4);
    for idx in 0..4 {
        required.push(
            slots[idx]
                .take()
                .ok_or_else(|| CliError::MissingFlag(flag_name(idx)))?,
        );
    }
    Ok(Args {
        rust_registry: required.remove(0),
        ts_entities: required.remove(0),
        sqlite_schema: required.remove(0),
        stores: required.remove(0),
        screen_manifest: slots[4].take(),
    })
}

/// Read all four (or five) input paths, returning their contents in the
/// order §24.1 enumerates them.
pub fn load_inputs(args: &Args) -> Result<LoadedInputs, CliError> {
    Ok(LoadedInputs {
        rust_registry: read_to_string("--rust-registry", &args.rust_registry)?,
        ts_entities: read_to_string("--ts-entities", &args.ts_entities)?,
        sqlite_schema: read_to_string("--sqlite-schema", &args.sqlite_schema)?,
        store_interfaces: read_to_string("--stores", &args.stores)?,
        screen_manifest: match &args.screen_manifest {
            Some(p) => Some(read_to_string("--screen-manifest", p)?),
            None => None,
        },
    })
}

#[derive(Clone, Debug)]
pub struct LoadedInputs {
    pub rust_registry: String,
    pub ts_entities: String,
    pub sqlite_schema: String,
    pub store_interfaces: String,
    pub screen_manifest: Option<String>,
}

fn read_to_string(flag: &'static str, path: &PathBuf) -> Result<String, CliError> {
    std::fs::read_to_string(path).map_err(|err| CliError::Read {
        flag,
        path: path.clone(),
        err: err.to_string(),
    })
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn missing_required_flag_errors() {
        let argv = ["validate", "--rust-registry", "/dev/null"];
        let err = parse_args(argv.iter().map(|s| s.to_string())).unwrap_err();
        assert!(matches!(err, CliError::MissingFlag("--ts-entities")));
    }

    #[test]
    fn all_required_flags_parse() {
        let argv = [
            "--rust-registry",
            "/a",
            "--ts-entities",
            "/b",
            "--sqlite-schema",
            "/c",
            "--stores",
            "/d",
        ];
        let args = parse_args(argv.iter().map(|s| s.to_string())).unwrap();
        assert_eq!(args.rust_registry, PathBuf::from("/a"));
        assert_eq!(args.ts_entities, PathBuf::from("/b"));
        assert_eq!(args.sqlite_schema, PathBuf::from("/c"));
        assert_eq!(args.stores, PathBuf::from("/d"));
        assert!(args.screen_manifest.is_none());
    }

    #[test]
    fn optional_screen_manifest_parse() {
        let argv = [
            "--rust-registry",
            "/a",
            "--ts-entities",
            "/b",
            "--sqlite-schema",
            "/c",
            "--stores",
            "/d",
            "--screen-manifest",
            "/e",
        ];
        let args = parse_args(argv.iter().map(|s| s.to_string())).unwrap();
        assert_eq!(args.screen_manifest, Some(PathBuf::from("/e")));
    }

    #[test]
    fn flag_value_starting_with_dash_is_rejected() {
        let argv = [
            "--rust-registry",
            "--ts-entities",
            "/b",
            "--sqlite-schema",
            "/c",
            "--stores",
            "/d",
        ];
        let err = parse_args(argv.iter().map(|s| s.to_string())).unwrap_err();
        assert!(matches!(err, CliError::BadValue { flag: "--rust-registry", .. }));
    }
}
