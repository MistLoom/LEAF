//! 业务逻辑：mixed

use ability_support::{AbilityCtx, Result};
use serde_json::{json, Value};

pub async fn mixed(_ctx: &dyn AbilityCtx, _args: Value) -> Result<Value> {
    let tags = ["ok", "bad_tag", "x"];
    ability_support::feedback!(tags, "mixed");
    Ok(json!({ "ok": true }))
}
