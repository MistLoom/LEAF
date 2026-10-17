//! 静态核对：钩子分拣 + Mod 出站 API 映射表。

use ability_plugin_host::{classify_hook, is_mapped_outbound, mod_outbound_surfaces};
use plugin_compat::{
    hook_names_from_source, mod_fixture_mjs_paths, outbound_calls_from_source, read_fixture,
};

#[test]
fn token_weather_hooks_classified() {
    let src = read_fixture("token-weather/hooks/token-weather.mjs");
    for name in hook_names_from_source(&src) {
        assert!(
            classify_hook(&name).is_some(),
            "未分拣钩子: {name}"
        );
    }
}

#[test]
fn token_weather_outbound_mapped() {
    let src = read_fixture("token-weather/hooks/token-weather.mjs");
    for (ns, method) in outbound_calls_from_source(&src) {
        assert!(
            is_mapped_outbound(&ns, &method),
            "未映射出站 API $.{ns}.{method}"
        );
    }
}

#[test]
fn host_surface_table_covers_dispatch_mapping() {
    assert!(mod_outbound_surfaces().iter().any(|(n, m)| *n == "session" && *m == "usage"));
}

#[test]
fn every_declared_outbound_surface_is_mappable() {
    for (ns, method) in mod_outbound_surfaces() {
        assert!(
            is_mapped_outbound(ns, method),
            "契约表未纳入 is_mapped_outbound: {ns}.{method}"
        );
    }
}

#[test]
fn all_mod_fixture_mjs_hooks_and_outbound() {
    let paths = mod_fixture_mjs_paths();
    assert!(
        paths.len() >= 3,
        "应至少包含 token-weather / blast-radius / replay-theater: {paths:?}"
    );
    for rel in paths {
        let src = read_fixture(&rel);
        let dir = rel.split('/').next().unwrap_or("fixture");
        for name in hook_names_from_source(&src) {
            assert!(
                classify_hook(&name).is_some(),
                "{dir}: 未分拣钩子: {name}"
            );
        }
        for (ns, method) in outbound_calls_from_source(&src) {
            assert!(
                is_mapped_outbound(&ns, &method),
                "{dir}: 未映射出站 $.{ns}.{method}"
            );
        }
    }
}

fn scan_fixture_mod(rel_dir: &str, rel_mjs: &str) {
    let src = read_fixture(rel_mjs);
    for name in hook_names_from_source(&src) {
        assert!(
            classify_hook(&name).is_some(),
            "{rel_dir}: 未分拣钩子: {name}"
        );
    }
    for (ns, method) in outbound_calls_from_source(&src) {
        assert!(
            is_mapped_outbound(&ns, &method),
            "{rel_dir}: 未映射出站 API $.{ns}.{method}"
        );
    }
}

#[test]
fn blast_radius_fixture() {
    scan_fixture_mod(
        "blast-radius",
        "blast-radius/hooks/blast-radius.mjs",
    );
}

#[test]
fn replay_theater_fixture() {
    scan_fixture_mod(
        "replay-theater",
        "replay-theater/hooks/replay-theater.mjs",
    );
}
