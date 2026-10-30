use ability_support::{load_ability_package, validate_schema};
use serde_json::{Value, json};

fn input_schema() -> Value {
    ability_write_file::provider()
        .manifest()
        .methods
        .into_iter()
        .find(|method| method.id.as_str() == "run")
        .expect("write_file.run")
        .input_schema
}

#[test]
fn provider_preserves_declared_input_schema() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let package = load_ability_package(root).unwrap();
    let method = package
        .decl
        .methods
        .iter()
        .find(|method| method.id == "run")
        .unwrap();
    assert_eq!(input_schema(), method.input);
}

#[test]
fn accepts_text_and_base64_inputs() {
    let schema = input_schema();
    for args in [
        json!({"path": "text.txt", "contents": "hello"}),
        json!({"path": "empty.txt", "contents": ""}),
        json!({"path": "binary.bin", "bytes_base64": "AP8=", "overwrite": false}),
        json!({"path": "binary.bin", "contents": "ignored", "bytes_base64": "AP8="}),
    ] {
        validate_schema(&args, &schema).unwrap();
    }
}

#[test]
fn rejects_missing_path_and_invalid_parameter_types() {
    let schema = input_schema();
    for (args, field) in [
        (json!({"bytes_base64": "AP8="}), "path"),
        (json!({"path": "file", "contents": 1}), "contents"),
        (
            json!({"path": "file", "contents": "text", "bytes_base64": 1}),
            "bytes_base64",
        ),
        (
            json!({"path": "file", "contents": "text", "overwrite": "false"}),
            "overwrite",
        ),
    ] {
        assert_eq!(validate_schema(&args, &schema).unwrap_err().path, field);
    }
}
