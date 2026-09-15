//! 业务逻辑：charge

use ability_support::{AbilityCtx, Result};
use serde_json::Value;

pub async fn charge(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let amt = args.get("amt").and_then(|v| v.as_u64()).unwrap_or(0);
    ability_support::feedback!(["pay"], "charged {amt}");
    Ok(args)
}
