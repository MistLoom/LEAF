//! 场景：统计、重复概率与 hook 出站。

mod common;
use common::build_kb_fun;
use serde_json::json;

#[tokio::test]
async fn stats_dup_and_hook_delivery() {
    let dir = std::env::temp_dir().join(format!("kb-sc-stat-{}", now()));
    let fun = build_kb_fun(&dir).await;

    let created = fun
        .invoke_str("kb.scope.kb_create", json!({ "name": "stats" }), None)
        .await
        .unwrap();
    let kb_id = created.data["kb_id"].as_str().unwrap().to_string();

    fun.invoke_str(
        "kb.hook.subscribe",
        json!({
            "url": "https://example.invalid/hooks/kb",
            "topics": ["kb.miu.ingested"],
            "scope_key": kb_id
        }),
        None,
    )
    .await
    .unwrap();

    let text = "可独立检索的最小信息单元称为 MIU。";
    let built = fun
        .invoke_str(
            "kb.clue.build",
            json!({
                "units": [{ "text": text, "doc_path": "root", "ordinal": 0 }]
            }),
            None,
        )
        .await
        .unwrap();
    let draft = &built.data["drafts"][0];
    let fp = draft["metadata"]["fingerprint"].as_str().map(|s| s.to_string());

    fun.invoke_str(
        "kb.miu.put",
        json!({ "kb_id": kb_id, "draft": draft }),
        None,
    )
    .await
    .unwrap();

    let deliveries = fun
        .invoke_str("kb.hook.deliveries", json!({}), None)
        .await
        .unwrap();
    let deliveries = deliveries.data["deliveries"].as_array().cloned().unwrap_or_default();
    assert!(
        deliveries.iter().any(|d| d["event"]["topic"] == "kb.miu.ingested"),
        "写入应触发出站记录"
    );

    let dup = fun
        .invoke_str(
            "kb.stats.dup_prob",
            json!({
                "kb_id": kb_id,
                "text": text,
                "fingerprint": fp
            }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(dup.data["p_dup"].as_f64().unwrap(), 1.0);

    let dim = fun
        .invoke_str(
            "kb.stats.dim",
            json!({ "kb_id": kb_id, "dimension": "modality" }),
            None,
        )
        .await
        .unwrap();
    assert!(!dim.data["buckets"].as_array().unwrap().is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

fn now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".into())
}
