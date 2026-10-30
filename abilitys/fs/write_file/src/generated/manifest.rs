use ability_support::AbilityManifest;

pub const ABILITY_ID: &str = "write_file";
pub const DECL_FRAMEWORK_VERSION: &str = "1";

pub fn manifest() -> AbilityManifest {
    AbilityManifest {
        id: ability_support::AbilityId::new(ABILITY_ID).expect("id"),
        depends: vec![],
        stateful: false,
        config_schema: serde_json::json!(null),
        methods: vec![ability_support::MethodMeta::new("run", "Write contents to a file.", serde_json::json!({"anyOf":[{"required":["contents"]},{"required":["bytes_base64"]}],"description":"Provide contents or bytes_base64. If both are provided, bytes_base64 takes precedence.","properties":{"bytes_base64":{"description":"Standard Base64-encoded bytes to write. Takes precedence over contents when both are provided.","type":"string"},"contents":{"description":"UTF-8 text to write. Required unless bytes_base64 is provided.","type":"string"},"overwrite":{"default":true,"description":"Whether to overwrite an existing file. Defaults to true.","type":"boolean"},"path":{"type":"string"}},"required":["path"],"type":"object"}))],
        events: vec![],
    }
}
