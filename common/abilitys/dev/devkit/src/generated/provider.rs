// 自动生成，勿手改
use std::sync::Arc;

use ability_support::{AbilityProvider, AbilitySpec};

use super::bridge::instance;
use super::manifest::{ABILITY_ID, DECL_FRAMEWORK_VERSION};

macro_rules! handler_impl {
    ($method:ident) => {
        move |ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value| {
            let handlers = instance();
            ability_support::box_fut(async move {
                handlers.$method(ctx, args).await
            })
        }
    };
}

pub fn provider() -> Arc<dyn AbilityProvider> {
    assert_eq!(
        DECL_FRAMEWORK_VERSION,
        ability_support::FRAMEWORK_VERSION,
        "ability.yml framework_version 与当前框架不匹配"
    );
    AbilitySpec::try_new(ABILITY_ID).expect("id")
        .on(
            "scaffold",
            "从模板生成新能力目录",
            serde_json::json!({"properties":{"id":{"type":"string"},"out":{"type":"string"},"protocols":{"items":{"type":"string"},"type":"array"},"template":{"type":"string"}},"required":["template","id","out"],"type":"object"}),
            handler_impl!(scaffold),
        )
        .on(
            "validate",
            "校验能力声明文件",
            serde_json::json!({"properties":{"path":{"type":"string"}},"required":["path"],"type":"object"}),
            handler_impl!(validate),
        )
        .on(
            "codegen",
            "触发 codegen 生成",
            serde_json::json!({"properties":{"path":{"type":"string"}},"required":["path"],"type":"object"}),
            handler_impl!(codegen),
        )
        .into_provider()
}
