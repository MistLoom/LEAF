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
            "charge",
            "扣款",
            serde_json::json!({"type":"object"}),
            handler_impl!(charge),
        )
        .on(
            "place",
            "下单",
            serde_json::json!({"type":"object"}),
            handler_impl!(place),
        )
        .on(
            "note",
            "投递一条反馈",
            serde_json::json!({"type":"object"}),
            handler_impl!(note),
        )
        .on(
            "mixed",
            "含非法标签的投递",
            serde_json::json!({"type":"object"}),
            handler_impl!(mixed),
        )
        .into_provider()
}
