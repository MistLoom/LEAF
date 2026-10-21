use ability_support::AbilityManifest;

pub const ABILITY_ID: &str = "devkit";
pub const DECL_FRAMEWORK_VERSION: &str = "1";

pub fn manifest() -> AbilityManifest {
    AbilityManifest {
        id: ability_support::AbilityId::new(ABILITY_ID).expect("id"),
        depends: vec![],
        stateful: false,
        config_schema: serde_json::json!(null),
        methods: vec![ability_support::MethodMeta::new("scaffold", "从模板生成新能力目录", serde_json::json!({"properties":{"id":{"type":"string"},"out":{"type":"string"},"protocols":{"items":{"type":"string"},"type":"array"},"template":{"type":"string"}},"required":["template","id","out"],"type":"object"})), ability_support::MethodMeta::new("validate", "校验能力声明文件", serde_json::json!({"properties":{"path":{"type":"string"}},"required":["path"],"type":"object"})), ability_support::MethodMeta::new("codegen", "触发 codegen 生成", serde_json::json!({"properties":{"path":{"type":"string"}},"required":["path"],"type":"object"}))],
        events: vec![],
    }
}
