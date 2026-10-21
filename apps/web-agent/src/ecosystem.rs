//! 生态 Ability 调用（Mod / DSH）。

use std::sync::Arc;

use frame_core::{AbilityExpose, Gateway};
use serde_json::{Value, json};

use crate::gateway::invoke;

const ECO_IDS: [&str; 2] = ["claude_mod", "dsh_plugin"];

pub async fn invoke_hook(
    gateway: &Arc<dyn Gateway>,
    expose: &AbilityExpose,
    ability: &str,
    tool: &str,
    args: Value,
) -> ability_support::Result<Value> {
    if !ECO_IDS.contains(&ability) {
        return Err(ability_support::Error::invalid_args(format!(
            "未知生态 Ability: {ability}"
        )));
    }
    if !expose.is_enabled(ability) {
        return Err(ability_support::Error::failed(format!(
            "装配未启用 {ability}"
        )));
    }
    invoke(
        gateway,
        ability,
        "invoke",
        json!({ "tool": tool, "args": args }),
    )
    .await
}
