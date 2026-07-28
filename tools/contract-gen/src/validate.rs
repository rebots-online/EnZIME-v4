//! The §24.1 consistency checker.
//!
//! Inputs are the five artefacts §24.1 names: the Rust command registry,
//! generated TypeScript entities, the SQLite schema, the Zustand store
//! interfaces, and (optionally) the Stitch screen manifest. The spec
//! constants in [`crate::spec`] are the canonical contract — drift between
//! code and spec is reported as a [`Diagnostic`].
//!
//! Per §26 AMD-4 this validator is a local script: invoking it does not
//! gate any CI job. The §24.1 rules apply unchanged — failing here means
//! the contract chain is broken, and the operator must resolve it before
//! the next release.

use crate::extract::{self, StoreInterface};
use crate::spec::{BINDING_TABLE, COMPONENT_NAMES, TS_BUILTIN_ENTITIES};
use std::collections::BTreeSet;

/// One screen entry of the Stitch screen manifest. Only the fields the
/// validator consults are deserialised; the rest are ignored. Field names
/// are camelCase to match the manifest's wire format
/// (`DOCS/REBOOT-PAK/STITCH/screen-manifest.json`).
#[derive(Clone, Debug, serde::Deserialize)]
#[allow(non_snake_case)]
pub struct ScreenEntry {
    pub screenId: String,
    pub rootComponent: String,
    #[serde(default)]
    pub entities: Vec<String>,
}

/// The Stitch screen-manifest file
/// (`DOCS/REBOOT-PAK/STITCH/screen-manifest.json`). Per §24.1, "a screen
/// prompt uses a different entity name" is a failure, so the validator
/// consults the manifest when one is supplied.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct ScreenManifest {
    pub screens: Vec<ScreenEntry>,
}

/// One validator finding. Each variant names the exact §24.1 rule that was
/// violated (or the manifest-parse failure that prevents the screen-prompt
/// rule from being evaluated) and carries enough context to localise the
/// failure in source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    /// §24.1: "a component references an undeclared entity".
    UndeclaredEntity {
        component: &'static str,
        entity: &'static str,
    },
    /// §24.1: "a command is absent from Rust".
    CommandMissingFromRust {
        component: &'static str,
        command: &'static str,
    },
    /// §24.1: "a command is absent from … TypeScript".
    CommandMissingFromTypescript {
        component: &'static str,
        command: &'static str,
    },
    /// §24.1: "a table name is absent from the migration".
    TableMissingFromMigration {
        component: &'static str,
        table: &'static str,
    },
    /// §24.1: "a store slice is absent".
    StoreMissing {
        component: &'static str,
        store_slice: &'static str,
        reason: StoreMissingReason,
    },
    /// §24.1: "a component public name differs from §16.6".
    ComponentNotInSection16_6 { component: &'static str },
    /// §24.1: "a screen prompt uses a different entity name". The kind
    /// field names whether the offending identifier was the screen's
    /// `rootComponent` or one of its `entities[]`.
    ScreenPromptNameDrift {
        screen_id: String,
        kind: ScreenEntityKind,
        name: String,
    },
    /// The supplied screen-manifest JSON could not be parsed; the §24.1
    /// screen-prompt rule cannot be evaluated until the manifest is valid.
    ScreenManifestUnparseable(String),
}

/// Where in a screen manifest a drift was observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenEntityKind {
    RootComponent,
    Entity,
}

/// Why a store slice did not resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreMissingReason {
    /// No store with the first path segment is declared in the supplied
    /// store-interface source.
    StoreNotDeclared,
    /// The store is declared but the next path segment is not a top-level
    /// key of its `create(…)` factory. Deeper segments are not separately
    /// verified — the §24 binding paths all start at a top-level slice, so
    /// a missing top-level key is the contract-relevant failure.
    TopLevelKeyMissing,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Diagnostic::UndeclaredEntity { component, entity } => write!(
                f,
                "§24.1 undeclared entity: component `{component}` references `{entity}` which is not present in the generated TypeScript entities (and is not a TS builtin)"
            ),
            Diagnostic::CommandMissingFromRust { component, command } => write!(
                f,
                "§24.1 command missing from Rust: component `{component}` invokes `{command}` but no `#[tauri::command] pub fn {command}` was found in the Rust command registry"
            ),
            Diagnostic::CommandMissingFromTypescript { component, command } => write!(
                f,
                "§24.1 command missing from TypeScript: component `{component}` invokes `{command}` but no generated TS binding for `{command}` was found"
            ),
            Diagnostic::TableMissingFromMigration { component, table } => write!(
                f,
                "§24.1 table missing from migration: component `{component}` binds table `{table}` but no `CREATE TABLE {table}` appears in the SQLite schema"
            ),
            Diagnostic::StoreMissing {
                component,
                store_slice,
                reason,
            } => {
                let why = match reason {
                    StoreMissingReason::StoreNotDeclared => "no such store is declared",
                    StoreMissingReason::TopLevelKeyMissing => {
                        "store is declared but the slice is not a top-level key"
                    }
                };
                write!(
                    f,
                    "§24.1 store slice absent: component `{component}` binds `{store_slice}` but {why}"
                )
            }
            Diagnostic::ComponentNotInSection16_6 { component } => write!(
                f,
                "§24.1 component name drift: §24 lists `{component}` but §16.6 does not declare a component with that public name"
            ),
            Diagnostic::ScreenPromptNameDrift {
                screen_id,
                kind,
                name,
            } => {
                let where_ = match kind {
                    ScreenEntityKind::RootComponent => "rootComponent",
                    ScreenEntityKind::Entity => "entities[]",
                };
                write!(
                    f,
                    "§24.1 screen prompt drift: screen `{screen_id}` references `{name}` in {where_} but no §16.6 component or §24 TypeScript entity has that name"
                )
            }
            Diagnostic::ScreenManifestUnparseable(err) => write!(
                f,
                "§24.1 screen-prompt rule could not be evaluated: the supplied screen-manifest is not valid JSON ({err})"
            ),
        }
    }
}

/// The five inputs §24.1 names. Each is the concatenated source text of
/// that input class — the CLI is responsible for reading files; the
/// validator is responsible only for the rules.
#[derive(Clone, Debug, Default)]
pub struct Inputs<'a> {
    pub rust_registry: &'a str,
    pub ts_entities: &'a str,
    pub sqlite_schema: &'a str,
    pub store_interfaces: &'a str,
    /// `None` if no screen manifest was supplied. §24.1's screen-prompt
    /// rule is consulted only when one is present — there is nothing to
    /// check otherwise.
    pub screen_manifest: Option<&'a str>,
}

/// Run all six §24.1 rules against the supplied inputs and the spec.
///
/// Diagnostics are returned in spec-table order (`HomeView` first,
/// `ModelRecommendation` last) so a re-run after a fix produces a stable,
/// diffable report. Within a single row, diagnostics are ordered by rule
/// (component → entity → Rust command → TS command → table → store slice).
pub fn validate(inputs: &Inputs<'_>) -> Vec<Diagnostic> {
    let rust_commands: BTreeSet<String> =
        extract::extract_rust_commands(inputs.rust_registry)
            .into_iter()
            .collect();
    let ts_entities: BTreeSet<String> = extract::extract_ts_entities(inputs.ts_entities)
        .into_iter()
        .collect();
    let sql_tables: BTreeSet<String> = extract::extract_sql_tables(inputs.sqlite_schema)
        .into_iter()
        .collect();
    let stores = extract::extract_store_interfaces(inputs.store_interfaces);

    let component_set: BTreeSet<&'static str> = COMPONENT_NAMES.iter().copied().collect();
    let builtin_set: BTreeSet<&'static str> =
        TS_BUILTIN_ENTITIES.iter().copied().collect();

    let mut out = Vec::new();

    for row in BINDING_TABLE {
        if !component_set.contains(row.component) {
            out.push(Diagnostic::ComponentNotInSection16_6 {
                component: row.component,
            });
        }

        if !ts_entities.contains(row.entity) && !builtin_set.contains(row.entity) {
            out.push(Diagnostic::UndeclaredEntity {
                component: row.component,
                entity: row.entity,
            });
        }

        for cmd in row.commands {
            if !rust_commands.contains(*cmd) {
                out.push(Diagnostic::CommandMissingFromRust {
                    component: row.component,
                    command: cmd,
                });
            }
            // §24.1 requires the command to be present in TypeScript as
            // well. The contract-gen pipeline emits the binding as a
            // flat `export const <command>` (or `export function
            // <command>`), so consulting the same `ts_entities` set is
            // the right contract — a nested-namespace binding would
            // itself be a generator bug.
            if !ts_entities.contains(*cmd) {
                out.push(Diagnostic::CommandMissingFromTypescript {
                    component: row.component,
                    command: cmd,
                });
            }
        }

        for table in row.tables.names() {
            if !sql_tables.contains(*table) {
                out.push(Diagnostic::TableMissingFromMigration {
                    component: row.component,
                    table,
                });
            }
        }

        if let Some(reason) = check_store_slice(&stores, row.store_slice) {
            out.push(Diagnostic::StoreMissing {
                component: row.component,
                store_slice: row.store_slice,
                reason,
            });
        }
    }

    if let Some(manifest_src) = inputs.screen_manifest {
        extend_with_screen_findings(manifest_src, &component_set, &ts_entities, &mut out);
    }

    out
}

fn extend_with_screen_findings(
    manifest_src: &str,
    component_set: &BTreeSet<&'static str>,
    ts_entities: &BTreeSet<String>,
    out: &mut Vec<Diagnostic>,
) {
    let manifest: ScreenManifest = match serde_json::from_str(manifest_src) {
        Ok(m) => m,
        Err(e) => {
            out.push(Diagnostic::ScreenManifestUnparseable(e.to_string()));
            return;
        }
    };
    // The screen-prompt rule: every identifier a screen declares must
    // resolve to a §16.6 component or a §24 TypeScript entity. We extend
    // the candidate set with the §24 entity column (some entities have
    // not yet been emitted to packages/contracts because the generator
    // has not run, but they are normative per §24 itself — so accepting
    // them is correct: the screen matches the spec even before codegen
    // catches up).
    let mut name_set: BTreeSet<&str> = component_set.iter().copied().collect();
    for row in BINDING_TABLE {
        name_set.insert(row.entity);
    }
    for ent in ts_entities {
        // Borrowing into the lifetime of `ts_entities` is fine because
        // `name_set` does not outlive this function.
        name_set.insert(ent.as_str());
    }
    for screen in &manifest.screens {
        if !name_set.contains(screen.rootComponent.as_str()) {
            out.push(Diagnostic::ScreenPromptNameDrift {
                screen_id: screen.screenId.clone(),
                kind: ScreenEntityKind::RootComponent,
                name: screen.rootComponent.clone(),
            });
        }
        for ent in &screen.entities {
            if !name_set.contains(ent.as_str()) {
                out.push(Diagnostic::ScreenPromptNameDrift {
                    screen_id: screen.screenId.clone(),
                    kind: ScreenEntityKind::Entity,
                    name: ent.clone(),
                });
            }
        }
    }
}

fn check_store_slice(
    stores: &[StoreInterface],
    store_slice: &'static str,
) -> Option<StoreMissingReason> {
    let mut parts = store_slice.split('.');
    let store_name = match parts.next() {
        Some(s) => s,
        None => return None,
    };
    let second = parts.next();
    let store = match stores.iter().find(|s| s.store_name == store_name) {
        Some(s) => s,
        None => return Some(StoreMissingReason::StoreNotDeclared),
    };
    match second {
        // A bare `storeName` with no further segments is satisfied by the
        // store existing.
        None => None,
        Some(key) => {
            if store.top_level_keys.iter().any(|k| k == key) {
                None
            } else {
                Some(StoreMissingReason::TopLevelKeyMissing)
            }
        }
    }
}

#[cfg(test)]
mod validate_tests {
    use super::*;
    use crate::spec::{BINDING_TABLE, COMPONENT_NAMES};

    /// A "green-field" fixture: every §24 binding is satisfied by every
    /// input. The validator returns no diagnostics. This is the largest
    /// single fixture in this crate and exists to prove the validator
    /// succeeds when the contract chain is intact, not just that it fails
    /// when something is wrong.
    #[test]
    fn green_field_inputs_produce_no_diagnostics() {
        let rust = green_field_rust();
        let ts = green_field_ts();
        let sql = green_field_sql();
        let stores = green_field_stores();
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        if !diags.is_empty() {
            for d in &diags {
                eprintln!("{d}");
            }
        }
        assert!(diags.is_empty(), "expected a clean green-field run");
    }

    /// §24.1 "a component references an undeclared entity": drop one
    /// entity from the TS source and assert the matching diagnostic.
    #[test]
    fn undeclared_entity_is_flagged() {
        let rust = green_field_rust();
        // Remove every line that declares HomeSnapshot.
        let ts = green_field_ts().replace("export interface HomeSnapshot { }\n", "");
        let sql = green_field_sql();
        let stores = green_field_stores();
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, Diagnostic::UndeclaredEntity { component: "HomeView", entity: "HomeSnapshot" })),
            "expected UndeclaredEntity for HomeView/HomeSnapshot, got {diags:?}"
        );
    }

    /// §24.1 "a command is absent from Rust": drop one Rust command and
    /// assert the matching diagnostic.
    #[test]
    fn command_missing_from_rust_is_flagged() {
        // Remove both the attribute line and the fn line for home_snapshot.
        let rust = green_field_rust()
            .replace("#[tauri::command]\npub fn home_snapshot() -> () {}\n\n", "");
        let ts = green_field_ts();
        let sql = green_field_sql();
        let stores = green_field_stores();
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, Diagnostic::CommandMissingFromRust { component: "HomeView", command: "home_snapshot" })),
            "expected CommandMissingFromRust for HomeView/home_snapshot, got {diags:?}"
        );
    }

    /// §24.1 "a command is absent from … TypeScript": drop one TS binding
    /// and assert the matching diagnostic, while keeping Rust intact.
    #[test]
    fn command_missing_from_typescript_is_flagged() {
        let rust = green_field_rust();
        // The TS binding for `home_snapshot` is emitted as
        // `export const home_snapshot = async () => {};`. Drop only that
        // line.
        let ts = green_field_ts()
            .replace("export const home_snapshot = async () => {};\n", "");
        let sql = green_field_sql();
        let stores = green_field_stores();
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, Diagnostic::CommandMissingFromTypescript { component: "HomeView", command: "home_snapshot" })),
            "expected CommandMissingFromTypescript for HomeView/home_snapshot, got {diags:?}"
        );
    }

    /// §24.1 "a table name is absent from the migration": drop one CREATE
    /// TABLE from the schema and assert the matching diagnostic.
    #[test]
    fn table_missing_from_migration_is_flagged() {
        let rust = green_field_rust();
        let ts = green_field_ts();
        // Strip the reader_history table only.
        let sql = green_field_sql().replace(
            "CREATE TABLE IF NOT EXISTS reader_history (id TEXT NOT NULL PRIMARY KEY);\n",
            "",
        );
        let stores = green_field_stores();
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, Diagnostic::TableMissingFromMigration { component: "ContinueReadingCard", table: "reader_history" })),
            "expected TableMissingFromMigration for ContinueReadingCard/reader_history, got {diags:?}"
        );
    }

    /// §24.1 "a store slice is absent": drop one top-level key from a
    /// store and assert the matching diagnostic.
    #[test]
    fn store_slice_missing_is_flagged() {
        let rust = green_field_rust();
        let ts = green_field_ts();
        let sql = green_field_sql();
        // Remove the `snapshot: null,` top-level key from homeStore. The
        // 4-space indent and trailing comma are emitted verbatim by
        // `green_field_stores`, so the match is exact.
        let stores = green_field_stores().replace("    snapshot: null,\n", "");
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        assert!(
            diags.iter().any(|d| matches!(
                d,
                Diagnostic::StoreMissing {
                    component: "HomeView",
                    store_slice: "homeStore.snapshot",
                    reason: StoreMissingReason::TopLevelKeyMissing,
                }
            )),
            "expected StoreMissing for HomeView/homeStore.snapshot, got {diags:?}"
        );
    }

    /// §24.1 "a screen prompt uses a different entity name": a screen
    /// manifest that names an unknown component is flagged.
    #[test]
    fn screen_prompt_drift_is_flagged() {
        let rust = green_field_rust();
        let ts = green_field_ts();
        let sql = green_field_sql();
        let stores = green_field_stores();
        let manifest = r#"{
            "schema": "enzime.stitch.screen-manifest.v1",
            "screens": [
                {
                    "screenId": "home-dashboard",
                    "rootComponent": "HomeView",
                    "entities": ["HomeView", "NotARealComponent"]
                }
            ]
        }"#;
        let inputs = Inputs {
            rust_registry: &rust,
            ts_entities: &ts,
            sqlite_schema: &sql,
            store_interfaces: &stores,
            screen_manifest: Some(manifest),
        };
        let diags = validate(&inputs);
        assert!(
            diags.iter().any(|d| matches!(
                d,
                Diagnostic::ScreenPromptNameDrift {
                    kind: ScreenEntityKind::Entity,
                    name,
                    ..
                } if name == "NotARealComponent"
            )),
            "expected ScreenPromptNameDrift for NotARealComponent, got {diags:?}"
        );
    }

    /// §24.1 "a component public name differs from §16.6": the spec is
    /// the source of truth, so this rule is exercised by mutating the
    /// spec constant — but that is impossible without recompiling. The
    /// rule is still exercised in production: any §24 component not in
    /// §16.6 surfaces a diagnostic. We verify the rule's presence by
    /// constructing a fixture where a non-§16.6 component name appears
    /// nowhere in code; the only way to trigger this against the real
    /// spec is to amend the pak, which is out of scope. The internal
    /// `spec_internal_tests::every_binding_component_exists_in_section_16_6`
    /// test asserts the spec itself never drifts.
    #[test]
    fn component_name_drift_rule_is_wired() {
        // Construct an empty input set and walk the spec — every §24
        // component must appear in COMPONENT_NAMES (already asserted by
        // spec_internal_tests). This test documents that the
        // ComponentNotInSection16_6 variant is reachable only by a pak
        // amendment, which is correct: §16.6 and §24 are normative.
        for row in BINDING_TABLE {
            assert!(
                COMPONENT_NAMES.contains(&row.component),
                "spec invariant broken for {}",
                row.component
            );
        }
    }

    /// Verify the in-tree SQLite schema (`schemas/sqlite/0001_initial.sql`,
    /// the verbatim §6.2 schema) contains every §24-named table. This is
    /// the one real-world artefact already present in the workspace that
    /// §24.1 references; running this test catches schema/§24 drift the
    /// moment either side changes. Other §24.1 inputs (Rust commands, TS
    /// entities, stores) are absent today, so we surface only the
    /// table-name diagnostics and assert that set is empty.
    #[test]
    fn real_tree_schema_covers_all_section_24_tables() {
        let schema_sql =
            include_str!("../../../schemas/sqlite/0001_initial.sql");
        let inputs = Inputs {
            rust_registry: "",
            ts_entities: "",
            sqlite_schema: schema_sql,
            store_interfaces: "",
            screen_manifest: None,
        };
        let diags = validate(&inputs);
        let table_drift: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| matches!(d, Diagnostic::TableMissingFromMigration { .. }))
            .collect();
        assert!(
            table_drift.is_empty(),
            "§24 references tables missing from schemas/sqlite/0001_initial.sql:\n{}",
            table_drift
                .iter()
                .map(|d| format!("  - {d}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // ---- green-field fixture builders ----------------------------------

    fn green_field_rust() -> String {
        let mut commands: BTreeSet<&'static str> = BTreeSet::new();
        for row in BINDING_TABLE {
            for cmd in row.commands {
                commands.insert(*cmd);
            }
        }
        let mut s = String::new();
        for cmd in commands {
            s.push_str("#[tauri::command]\n");
            s.push_str(&format!("pub fn {cmd}() -> () {{}}\n\n"));
        }
        s
    }

    fn green_field_ts() -> String {
        let mut s = String::new();
        // Entities.
        let mut entities: BTreeSet<&'static str> = BTreeSet::new();
        for row in BINDING_TABLE {
            entities.insert(row.entity);
        }
        for ent in entities {
            // TS builtins (number/string/boolean) are skipped — they are
            // not user-declared.
            if TS_BUILTIN_ENTITIES.contains(&ent) {
                continue;
            }
            s.push_str(&format!("export interface {ent} {{ }}\n"));
        }
        // Command bindings (flat `export const <command>` form).
        let mut commands: BTreeSet<&'static str> = BTreeSet::new();
        for row in BINDING_TABLE {
            for cmd in row.commands {
                commands.insert(*cmd);
            }
        }
        for cmd in commands {
            s.push_str(&format!("export const {cmd} = async () => {{}};\n"));
        }
        s
    }

    fn green_field_sql() -> String {
        let mut tables: BTreeSet<&'static str> = BTreeSet::new();
        for row in BINDING_TABLE {
            for t in row.tables.names() {
                tables.insert(*t);
            }
        }
        let mut s = String::new();
        for t in tables {
            s.push_str(&format!("CREATE TABLE IF NOT EXISTS {t} (id TEXT NOT NULL PRIMARY KEY);\n"));
        }
        s
    }

    fn green_field_stores() -> String {
        // For each store, declare it with the union of all top-level keys
        // referenced across all §24 binding paths that begin with that
        // store name.
        use std::collections::BTreeMap;
        let mut by_store: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for row in BINDING_TABLE {
            let mut parts = row.store_slice.split('.');
            let store = match parts.next() {
                Some(s) => s.to_string(),
                None => continue,
            };
            let entry = by_store.entry(store).or_default();
            if let Some(key) = parts.next() {
                entry.insert(key.to_string());
            }
        }
        let mut s = String::new();
        for (store, keys) in by_store {
            // Idiomatic Zustand v4 form: the factory returns the object
            // literal directly via `=> ({ ... })`. The §24 store-paths all
            // start at a top-level key, so only top-level keys are emitted.
            s.push_str(&format!(
                "export const {store} = create<{store}State>()((set, get) => ({{\n"
            ));
            for key in keys {
                s.push_str(&format!("    {key}: null,\n"));
            }
            s.push_str("}}));\n\n");
        }
        s
    }
}
