use std::path::Path;
#[test]
fn canonical_library_is_independent_and_adapter_targets_it() {
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR"));
    let canonical =
        adapter.join("../../VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib/rust");
    let library: toml::Value = std::fs::read_to_string(canonical.join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(dependencies) = library.get(section).and_then(toml::Value::as_table) {
            for (name, dependency) in dependencies {
                let package = dependency
                    .get("package")
                    .and_then(toml::Value::as_str)
                    .unwrap_or(name);
                assert!(
                    !["domain", "ports", "tauri"].contains(&package),
                    "app dependency: {package}"
                );
                assert!(!package.starts_with("tauri-"));
            }
        }
    }
    let application: toml::Value = std::fs::read_to_string(adapter.join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let path = application["dependencies"]["anzimmerman"]["path"]
        .as_str()
        .unwrap();
    assert_eq!(
        adapter.join(path).canonicalize().unwrap(),
        canonical.canonicalize().unwrap()
    );
    assert_eq!(library["package"]["name"].as_str(), Some("anzimmerman"));
}
