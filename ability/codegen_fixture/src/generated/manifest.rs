use ability_support::AbilityManifest;

pub const ABILITY_ID: &str = "codegen.fixture";
pub const DECL_FRAMEWORK_VERSION: &str = "1";

pub fn manifest() -> AbilityManifest {
    AbilityManifest {
        id: ability_support::AbilityId::new(ABILITY_ID).expect("id"),
        depends: vec![],
        stateful: false,
        config_schema: serde_json::json!(null),
        methods: vec![ability_support::MethodMeta::new("run", "codegen 测试用回显", serde_json::json!({"properties":{"text":{"type":"string"}},"required":["text"],"type":"object"})), ability_support::MethodMeta::new("ping", "codegen 测试用 ping", serde_json::json!({"type":"object"}))],
        events: vec![],
    }
}
