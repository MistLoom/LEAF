//! 场景：多库与 Scope 隔离。

mod common;
use common::build_kb_fun;
use serde_json::json;

#[tokio::test]
async fn multi_kb_and_scope_isolation() {
    let dir = std::env::temp_dir().join(format!("kb-sc-iso-{}", now()));
    let fun = build_kb_fun(&dir).await;

    let a = fun
        .invoke_str("kb.scope.kb_create", json!({ "name": "kb-a" }), None)
        .await
        .unwrap();
    let b = fun
        .invoke_str("kb.scope.kb_create", json!({ "name": "kb-b" }), None)
        .await
        .unwrap();
    let kb_a = a.data["kb_id"].as_str().unwrap();
    let kb_b = b.data["kb_id"].as_str().unwrap();
    assert_ne!(kb_a, kb_b);

    let sa = fun
        .invoke_str(
            "kb.scope.create",
            json!({ "kb_id": kb_a, "name": "alpha" }),
            None,
        )
        .await
        .unwrap();
    let sb = fun
        .invoke_str(
            "kb.scope.create",
            json!({ "kb_id": kb_b, "name": "beta" }),
            None,
        )
        .await
        .unwrap();
    let scope_a = sa.data["scope_id"].as_str().unwrap();
    let scope_b = sb.data["scope_id"].as_str().unwrap();

    let draft_a = json!({
        "raw_summary": "只有 A 库能搜到的秘密词 ALPHA_ONLY",
        "symbol": { "proposition": "alpha secret" },
        "metadata": { "scopes": [scope_a], "modality": "text" }
    });
    let draft_b = json!({
        "raw_summary": "只有 B 库能搜到的秘密词 BETA_ONLY",
        "symbol": { "proposition": "beta secret" },
        "metadata": { "scopes": [scope_b], "modality": "text" }
    });

    fun.invoke_str(
        "kb.miu.put",
        json!({ "kb_id": kb_a, "draft": draft_a }),
        None,
    )
    .await
    .unwrap();
    fun.invoke_str(
        "kb.miu.put",
        json!({ "kb_id": kb_b, "draft": draft_b }),
        None,
    )
    .await
    .unwrap();

    let hit_a = fun
        .invoke_str(
            "kb.retrieve.lexical",
            json!({ "kb_id": kb_a, "query": "ALPHA_ONLY" }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(hit_a.data["hits"].as_array().unwrap().len(), 1);

    let miss_a = fun
        .invoke_str(
            "kb.retrieve.lexical",
            json!({ "kb_id": kb_a, "query": "BETA_ONLY" }),
            None,
        )
        .await
        .unwrap();
    assert!(miss_a.data["hits"].as_array().unwrap().is_empty());

    let hit_b = fun
        .invoke_str(
            "kb.retrieve.lexical",
            json!({ "kb_id": kb_b, "query": "BETA_ONLY" }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(hit_b.data["hits"].as_array().unwrap().len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

fn now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".into())
}
