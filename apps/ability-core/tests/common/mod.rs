//! 测试用内联 Ability 构造。

use std::sync::Arc;

use ability_support::{
    Ability, AbilityId, AbilityManifest, AbilityProvider, EventDecl, EventDir, FnAbility, MethodId,
    MethodMeta, StaticProvider,
};
use serde_json::{Value, json};

pub fn method(id: &str, desc: &str) -> MethodMeta {
    MethodMeta {
        id: MethodId::new(id).unwrap(),
        description: desc.into(),
        input_schema: json!({ "type": "object" }),
        output_schema: None,
    }
}

pub fn manifest(id: &str, methods: &[&str]) -> AbilityManifest {
    AbilityManifest {
        id: AbilityId::new(id).unwrap(),
        depends: vec![],
        stateful: false,
        config_schema: Value::Null,
        methods: methods.iter().map(|m| method(m, m)).collect(),
        events: vec![],
    }
}

pub fn manifest_with_events(id: &str, methods: &[&str], emit: &[&str]) -> AbilityManifest {
    let mut m = manifest(id, methods);
    m.events = emit
        .iter()
        .map(|n| EventDecl {
            name: (*n).into(),
            dir: EventDir::Emit,
            description: String::new(),
            payload_schema: json!({ "type": "object" }),
        })
        .collect();
    m
}

pub fn echo_provider(id: &str) -> Arc<dyn AbilityProvider> {
    let m = manifest(id, &["run"]);
    let ability: Arc<dyn Ability> = Arc::new(FnAbility::new(m, |args| async move { Ok(args) }));
    Arc::new(StaticProvider::new(ability))
}
