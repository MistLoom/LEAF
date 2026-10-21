//! 业务逻辑：scaffold

use ability_support::{AbilityCtx, Error, Result};
use serde_json::{Value, json};

use crate::generated::handlers_trait::ScaffoldArgs;
use crate::scaffold::scaffold as run_scaffold;

pub async fn scaffold(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: ScaffoldArgs =
        serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
    run_scaffold(&a)?;
    Ok(json!({ "ok": true, "path": a.out }))
}
