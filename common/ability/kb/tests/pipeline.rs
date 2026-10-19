//! 场景：线索管道编排入库并词法检索（调用方组合细粒度工具）。

mod common;
use common::build_kb_fun;
use serde_json::json;

#[tokio::test]
async fn clue_pipeline_ingest_and_lexical_search() {
    let dir = std::env::temp_dir().join(format!("kb-sc-pipeline-{}", uuid_like()));
    let fun = build_kb_fun(&dir).await;

    let created = fun
        .invoke_str("kb.scope.kb_create", json!({ "name": "docs" }), None)
        .await
        .expect("create kb");
    let kb_id = created.data["kb_id"].as_str().unwrap().to_string();

    let scope = fun
        .invoke_str(
            "kb.scope.create",
            json!({ "kb_id": kb_id, "name": "engineering" }),
            None,
        )
        .await
        .expect("scope");
    let scope_id = scope.data["scope_id"].as_str().unwrap().to_string();

    let clue = json!({
        "kind": "text",
        "text": "# 检索\n\n混合检索使用 RRF 融合词法与向量信号。\n\n# 更新\n\nMIU 支持按源增量更新而不影响其它单元。",
        "source": { "source_type": "doc", "source_id": "guide-1" }
    });

    let decomp = fun
        .invoke_str("kb.clue.decompose", json!({ "clue": clue }), None)
        .await
        .expect("decompose");
    let units = decomp.data["units"].as_array().unwrap();
    assert!(units.len() >= 2, "应按标题拆成多个单元");

    let mut passed = Vec::new();
    for unit in units {
        let g = fun
            .invoke_str(
                "kb.clue.gate",
                json!({ "unit": unit, "kb_id": kb_id }),
                None,
            )
            .await
            .expect("gate");
        if g.data["decision"] == "pass" {
            passed.push(unit.clone());
        }
    }
    assert!(!passed.is_empty());

    let built = fun
        .invoke_str(
            "kb.clue.build",
            json!({ "units": passed, "scope_ids": [scope_id] }),
            None,
        )
        .await
        .expect("build");
    let drafts = &built.data["drafts"];

    let put = fun
        .invoke_str(
            "kb.miu.put_many",
            json!({ "kb_id": kb_id, "drafts": drafts }),
            None,
        )
        .await
        .expect("put_many");
    assert!(put.data["mius"].as_array().unwrap().len() >= 1);

    let hits = fun
        .invoke_str(
            "kb.retrieve.lexical",
            json!({ "kb_id": kb_id, "query": "RRF", "scope_ids": [scope_id] }),
            None,
        )
        .await
        .expect("lexical");
    assert!(
        !hits.data["hits"].as_array().unwrap().is_empty(),
        "应命中含 RRF 的单元"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn uuid_like() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".into())
}
