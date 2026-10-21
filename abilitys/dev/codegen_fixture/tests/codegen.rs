//! codegen 全功能集成测试

use std::fs;
use std::path::Path;

use ability_sdk_codegen::{FRAMEWORK_VERSION, generate, write_scaffold_once};
use ability_support::load_ability_package;

fn write_minimal_rust_crate(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"codegen-test-tmp\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("Cargo.toml");
}

#[test]
fn framework_version_constant() {
    assert_eq!(FRAMEWORK_VERSION, "1");
}

#[test]
fn rejects_missing_framework_version() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(
        dir.path().join("ability.yml"),
        r#"id: test.missing
protocols: [fun]
methods:
  - id: run
    description: x
    input: { type: object }
"#,
    )
    .unwrap();
    let err = load_ability_package(dir.path()).unwrap_err();
    assert!(err.contains("framework_version"));
}

#[test]
fn rejects_incompatible_framework_version() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(
        dir.path().join("ability.yml"),
        r#"framework_version: "99"
id: test.bad
protocols: [fun]
methods:
  - id: run
    description: x
    input: { type: object }
"#,
    )
    .unwrap();
    let err = load_ability_package(dir.path()).unwrap_err();
    assert!(err.contains("不兼容"));
}

#[test]
fn generate_creates_generated_and_logic_stubs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    write_minimal_rust_crate(root);
    fs::write(
        root.join("ability.yml"),
        format!(
            r#"framework_version: "{FRAMEWORK_VERSION}"
id: test.gen
protocols: [fun]
methods:
  - id: run
    description: run
    input:
      type: object
"#
        ),
    )
    .unwrap();

    let out = root.join("src/generated");
    generate(root, &out).expect("generate");

    assert!(out.join("bridge.rs").is_file());
    assert!(root.join("src/logic/run.rs").is_file());
}

#[test]
fn regenerate_does_not_overwrite_logic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    write_minimal_rust_crate(root);
    fs::write(
        root.join("ability.yml"),
        format!(
            r#"framework_version: "{FRAMEWORK_VERSION}"
id: test.keep
protocols: [fun]
methods:
  - id: run
    description: run
    input:
      type: object
"#
        ),
    )
    .unwrap();

    let out = root.join("src/generated");
    generate(root, &out).expect("generate1");

    let logic = root.join("src/logic/run.rs");
    fs::write(&logic, "// USER_LOGIC_MARKER\n").unwrap();

    generate(root, &out).expect("generate2");
    assert!(
        fs::read_to_string(&logic)
            .unwrap()
            .contains("USER_LOGIC_MARKER")
    );
}

#[test]
fn new_method_adds_logic_stub_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("src/logic")).unwrap();
    write_minimal_rust_crate(root);
    fs::write(root.join("src/logic/mod.rs"), "pub mod run;\n").unwrap();
    fs::write(root.join("src/logic/run.rs"), "// existing\n").unwrap();
    fs::write(
        root.join("ability.yml"),
        format!(
            r#"framework_version: "{FRAMEWORK_VERSION}"
id: test.new
protocols: [fun]
methods:
  - id: run
    description: run
    input: {{ type: object }}
  - id: ping
    description: ping
    input: {{ type: object }}
"#
        ),
    )
    .unwrap();

    generate(root, &root.join("src/generated")).expect("generate");

    assert!(root.join("src/logic/ping.rs").is_file());
    assert!(
        fs::read_to_string(root.join("src/logic/run.rs"))
            .unwrap()
            .contains("existing")
    );
    assert!(
        fs::read_to_string(root.join("src/logic/mod.rs"))
            .unwrap()
            .contains("pub mod ping;")
    );
}

#[test]
fn scaffold_once_skips_existing_bin() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bin = dir.path().join("tool.rs");
    fs::write(&bin, "// custom bin").unwrap();
    assert!(!write_scaffold_once(&bin, "// generated").expect("scaffold"));
}

#[test]
fn fixture_package_loads() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let meta = load_ability_package(root).expect("fixture");
    assert_eq!(meta.decl.id, "codegen.fixture");
    assert_eq!(meta.decl.methods.len(), 2);
}

#[test]
fn fixture_provider_has_all_methods() {
    let provider = ability_codegen_fixture::provider();
    assert_eq!(provider.manifest().methods.len(), 2);
}
