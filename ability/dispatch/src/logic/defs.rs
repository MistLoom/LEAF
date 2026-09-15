//! 业务逻辑：defs

use ability_support::{method_catalog, AbilityCtx, ProtocolKind, Result};
use serde_json::{json, Value};

use crate::bridge::catalog_to_tool_defs;

pub async fn defs(ctx: &dyn AbilityCtx, _args: Value) -> Result<Value> {
    let tools_ctx = ctx.with_protocol_filter(ProtocolKind::Tools);
    let methods = method_catalog(&tools_ctx.catalog());
    let defs = catalog_to_tool_defs(&methods);
    Ok(json!({ "tools": defs }))
}
