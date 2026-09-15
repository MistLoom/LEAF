use ability_support::AbilityManifest;

pub const ABILITY_ID: &str = "test.feedback";
pub const DECL_FRAMEWORK_VERSION: &str = "1";

pub fn manifest() -> AbilityManifest {
    AbilityManifest {
        id: ability_support::AbilityId::new(ABILITY_ID).expect("id"),
        depends: vec![],
        stateful: false,
        config_schema: serde_json::json!(null),
        methods: vec![ability_support::MethodMeta::new("charge", "扣款", serde_json::json!({"type":"object"})), ability_support::MethodMeta::new("place", "下单", serde_json::json!({"type":"object"})), ability_support::MethodMeta::new("note", "投递一条反馈", serde_json::json!({"type":"object"})), ability_support::MethodMeta::new("mixed", "含非法标签的投递", serde_json::json!({"type":"object"}))],
        events: vec![],
    }
}
