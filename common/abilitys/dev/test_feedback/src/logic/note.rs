//! 业务逻辑：note

use ability_support::{AbilityCtx, Result};
use serde_json::{Value, json};

use crate::note_from_args;

pub async fn note(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    note_from_args(&args);
    Ok(json!({ "ok": true }))
}
