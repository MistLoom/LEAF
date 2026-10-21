//! 业务逻辑：触发 ability-sdk-codegen。

use ability_support::{AbilityCtx, Error, Result};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct PathArgs {
    path: String,
}

pub async fn codegen(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: PathArgs =
        serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
    let out = std::path::Path::new(&a.path).join("src").join("generated");
    ability_sdk_codegen::generate(&a.path, &out).map_err(|e| Error::invalid_args(e))?;
    Ok(json!({ "ok": true, "out": out.display().to_string() }))
}
