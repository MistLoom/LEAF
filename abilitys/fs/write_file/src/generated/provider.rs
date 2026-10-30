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
            "run",
            "Write contents to a file.",
            serde_json::json!({"anyOf":[{"required":["contents"]},{"required":["bytes_base64"]}],"description":"Provide contents or bytes_base64. If both are provided, bytes_base64 takes precedence.","properties":{"bytes_base64":{"description":"Standard Base64-encoded bytes to write. Takes precedence over contents when both are provided.","type":"string"},"contents":{"description":"UTF-8 text to write. Required unless bytes_base64 is provided.","type":"string"},"overwrite":{"default":true,"description":"Whether to overwrite an existing file. Defaults to true.","type":"boolean"},"path":{"type":"string"}},"required":["path"],"type":"object"}),
            handler_impl!(run),
        )
        .into_provider()
}
