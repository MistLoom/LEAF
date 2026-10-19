//! 静态核对第三方 Mod：钩子名能被宿主分拣，出站 `$.ns.method` 落在契约表内。
//!
//! fixture 在 `tests/fixtures/{token-weather,blast-radius,replay-theater}`。

use std::path::Path;

use ability_plugin_host::{classify_hook, is_mapped_outbound, mod_outbound_surfaces};

#[test]
fn surfaces_are_mappable() {
    for (ns, method) in mod_outbound_surfaces() {
        assert!(
            is_mapped_outbound(ns, method),
            "契约表未纳入 is_mapped_outbound: {ns}.{method}"
        );
    }
    assert!(mod_outbound_surfaces()
        .iter()
        .any(|(ns, method)| *ns == "session" && *method == "usage"));
}

#[test]
fn fixture_hooks_and_outbound_are_covered() {
    let fixtures = fixture_mjs();
    assert!(
        fixtures.len() >= 3,
        "应包含 token-weather、blast-radius、replay-theater: {fixtures:?}"
    );
    for rel in fixtures {
        let src = read_fixture(&rel);
        let dir = rel.split('/').next().unwrap_or("fixture");
        for name in hook_names(&src) {
            assert!(classify_hook(&name).is_some(), "{dir}: 未分拣钩子 {name}");
        }
        for (ns, method) in outbound_calls(&src) {
            assert!(
                is_mapped_outbound(&ns, &method),
                "{dir}: 未映射 $.{ns}.{method}"
            );
        }
    }
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture(rel: &str) -> String {
    std::fs::read_to_string(fixture_root().join(rel)).expect("read fixture")
}

const COMPAT_FIXTURES: &[&str] = &["blast-radius", "replay-theater", "token-weather"];

fn fixture_mjs() -> Vec<String> {
    let root = fixture_root();
    let mut out = Vec::new();
    for name in COMPAT_FIXTURES {
        let hooks = root.join(name).join("hooks");
        if !hooks.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(&hooks).expect("hooks").flatten() {
            let path = file.path();
            if path.extension().is_some_and(|ext| ext == "mjs") {
                let rel = path
                    .strip_prefix(&root)
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

fn hook_names(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in src.lines() {
        let mut rest = line;
        while let Some(at) = rest.find("on(") {
            rest = &rest[at + 3..];
            let rest = rest.trim_start();
            let Some(quote) = rest.chars().next() else {
                continue;
            };
            if quote != '\'' && quote != '"' {
                continue;
            }
            let body = &rest[1..];
            if let Some(end) = body.find(quote) {
                names.push(body[..end].to_string());
            }
        }
    }
    names
}

fn outbound_calls(src: &str) -> Vec<(String, String)> {
    let mut calls = Vec::new();
    for line in src.lines() {
        let mut rest = line;
        while let Some(at) = rest.find("$.") {
            rest = &rest[at + 2..];
            if let Some(call) = split_call(rest) {
                calls.push(call);
            }
        }
    }
    calls
}

fn split_call(src: &str) -> Option<(String, String)> {
    let mut parts = src.split('.');
    let ns = ident(parts.next()?);
    let method = ident(parts.next()?);
    if ns.is_empty() || method.is_empty() {
        None
    } else {
        Some((ns, method))
    }
}

fn ident(src: &str) -> String {
    src.chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}
