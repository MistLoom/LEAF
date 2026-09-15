//! 业务逻辑：ping

use ability_support::{AbilityCtx, Result};
use serde_json::Value;

pub async fn ping(_ctx: &dyn AbilityCtx, _args: Value) -> Result<Value> {
    Ok(serde_json::json!({ "pong": true }))
}
