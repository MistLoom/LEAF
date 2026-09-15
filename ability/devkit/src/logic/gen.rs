//! 业务逻辑：gen

use ability_support::{AbilityCtx, Error, Result};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct PathArgs {
    path: String,
}

pub async fn gen(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: PathArgs =
        serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
    let out = std::path::Path::new(&a.path).join("src").join("generated");
    ability_sdk_codegen::generate(&a.path, &out)?;
    Ok(json!({ "ok": true, "out": out.display().to_string() }))
}
