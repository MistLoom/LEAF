//! 业务逻辑：run

use std::path::Path;

use ability_support::{AbilityCtx, Error, Result};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Args {
    path: String,
    contents: String,
}

pub async fn run(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: Args = serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
    let path = Path::new(&a.path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::io(format!("write_file create_dir {}: {e}", parent.display()))
            })?;
        }
    }
    std::fs::write(path, a.contents.as_bytes())
        .map_err(|e| Error::io(format!("write_file {}: {e}", a.path)))?;
    Ok(json!({ "path": a.path, "bytes": a.contents.len() }))
}
