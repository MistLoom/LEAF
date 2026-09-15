//! 业务逻辑：run

use ability_support::{decode_text_bytes, AbilityCtx, Error, Result};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Args {
    path: String,
    offset: Option<i64>,
    limit: Option<i64>,
}

pub async fn run(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: Args = serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
    let raw_bytes = std::fs::read(&a.path)
        .map_err(|e| Error::io(format!("read_file {}: {e}", a.path)))?;
    let raw = decode_text_bytes(&raw_bytes);
    let content = slice_lines(&raw, a.offset, a.limit);
    Ok(json!({ "path": a.path, "content": content }))
}

fn slice_lines(raw: &str, offset: Option<i64>, limit: Option<i64>) -> String {
    let lines: Vec<&str> = raw.lines().collect();
    let start = match offset {
        Some(o) if o > 0 => (o as usize).saturating_sub(1),
        Some(o) if o < 0 => lines.len().saturating_sub((-o) as usize),
        _ => 0,
    };
    let end = match limit {
        Some(l) if l > 0 => (start + l as usize).min(lines.len()),
        _ => lines.len(),
    };
    if start >= lines.len() {
        return String::new();
    }
    lines[start..end].join("\n")
}
