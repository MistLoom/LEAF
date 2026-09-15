//! devkit 与 codegen fixture 集成测试

use ability_support::load_ability_package;

#[test]
fn fixture_decl_valid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let meta = load_ability_package(root).expect("load");
    assert_eq!(meta.decl.id, "codegen.fixture");
    assert_eq!(meta.decl.methods.len(), 2);
}

#[test]
fn provider_compiles() {
    let _ = ability_codegen_fixture::provider();
}
