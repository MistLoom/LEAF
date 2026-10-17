//! 供测试共用的静态解析工具。

use std::path::Path;

/// 从源码中提取 `on("...")` / `on('...')` 钩子名。
pub fn hook_names_from_source(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in src.lines() {
        let mut rest = line;
        while let Some(at) = rest.find("on(") {
            rest = &rest[at + 3..];
            let rest_trim = rest.trim_start();
            let quote = rest_trim.chars().next();
            if quote != Some('\'') && quote != Some('"') {
                continue;
            }
            let q = quote.unwrap();
            let body = &rest_trim[1..];
            if let Some(end) = body.find(q) {
                names.push(body[..end].to_string());
            }
        }
    }
    names
}

/// 从源码中提取 `$.ns.method` 出站调用。
pub fn outbound_calls_from_source(src: &str) -> Vec<(String, String)> {
    let mut calls = Vec::new();
    for line in src.lines() {
        let mut rest = line;
        while let Some(at) = rest.find("$.") {
            rest = &rest[at + 2..];
            if let Some((ns, method)) = split_call(rest) {
                calls.push((ns, method));
            }
        }
    }
    calls
}

fn split_call(src: &str) -> Option<(String, String)> {
    let mut parts = src.split('.');
    let ns = parts.next()?;
    let method = parts.next()?;
    let ns: String = ns
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    let method: String = method
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if ns.is_empty() || method.is_empty() {
        None
    } else {
        Some((ns, method))
    }
}

pub fn read_fixture(rel: &str) -> String {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    std::fs::read_to_string(base.join(rel)).expect("read fixture")
}

/// 所有 Mod 风格 fixture 的 `hooks/*.mjs`（相对 `tests/fixtures`）。
pub fn mod_fixture_mjs_paths() -> Vec<String> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&base).expect("fixtures dir").flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let hooks = path.join("hooks");
        if !hooks.is_dir() {
            continue;
        }
        for ent in std::fs::read_dir(&hooks).expect("hooks dir").flatten() {
            let p = ent.path();
            if p.extension().is_some_and(|e| e == "mjs") {
                let rel = p
                    .strip_prefix(&base)
                    .expect("under fixtures")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push(rel);
            }
        }
    }
    out.sort();
    out
}
