//! 业务逻辑：validate

use ability_support::{
    AbilityCtx, LintOptions, Result, lint_ability_package, load_ability_package,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct PathArgs {
    path: String,
}

pub async fn validate(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: PathArgs = serde_json::from_value(args)
        .map_err(|e| ability_support::Error::invalid_args(e.to_string()))?;
    let root = std::path::Path::new(&a.path);
    let meta = load_ability_package(root).map_err(|e| ability_support::Error::invalid_args(e))?;
    let report = lint_ability_package(&meta, &LintOptions::workspace_strict());
    if !report.ok() {
        let msg = report
            .errors
            .into_iter()
            .chain(report.warnings.into_iter().map(|w| format!("[warn] {w}")))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(ability_support::Error::invalid_args(msg));
    }
    Ok(json!({ "ok": true, "id": meta.decl.id }))
}
