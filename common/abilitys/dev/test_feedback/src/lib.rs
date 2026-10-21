//! 测试用 Ability `test.feedback`。不装入 runtime。

mod generated;
mod logic;

use serde_json::Value;

pub use generated::provider::provider;

/// 无调用链时投递（不经 invoke）。
pub fn emit_detached() {
    ability_support::feedback!(["idle"], "background");
}

pub(crate) fn note_from_args(args: &Value) {
    let desc = args
        .get("desc")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tags: Vec<String> = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let refs: Vec<&str> = tags.iter().map(String::as_str).collect();
    ability_support::feedback!(refs, "{desc}");
}
