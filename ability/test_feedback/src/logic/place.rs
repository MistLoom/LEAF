//! 业务逻辑：place

use ability_support::{AbilityCtx, MethodId, Result};
use serde_json::{json, Value};

const ID: &str = "test.feedback";

pub async fn place(ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    ability_support::feedback!(["order"], "begin");
    let mid = MethodId::new("charge").expect("charge");
    ctx.call(ID, &mid, args.clone()).await?;
    ability_support::feedback!(["order"], "done");
    Ok(json!({ "ok": true }))
}
