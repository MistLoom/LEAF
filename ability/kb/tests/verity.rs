//! 场景：多源检索相关结果集与真实度评估。

mod common;
use common::build_kb_fun;
use serde_json::json;

#[tokio::test]
async fn multi_source_related_and_verity() {
    let dir = std::env::temp_dir().join(format!("kb-sc-verity-{}", now()));
    let fun = build_kb_fun(&dir).await;

    let created = fun
        .invoke_str("kb.scope.kb_create", json!({ "name": "verity-kb" }), None)
        .await
        .unwrap();
    let kb_id = created.data["kb_id"].as_str().unwrap().to_string();

    let drafts = json!([
        {
            "raw_summary": "RRF 融合词法与向量信号用于混合检索。",
            "symbol": { "proposition": "RRF 用于混合检索" },
            "metadata": {
                "modality": "text",
                "source": { "source_type": "doc", "source_id": "guide" }
            }
        },
        {
            "raw_summary": "混合检索实践中常用 RRF 合并多路排名。",
            "symbol": { "proposition": "实践使用 RRF" },
            "metadata": {
                "modality": "text",
                "source": { "source_type": "blog", "source_id": "post-9" }
            }
        },
        {
            "raw_summary": "上下文补回依赖 prev_id 与 next_id。",
            "symbol": { "proposition": "上下文链" },
            "metadata": {
                "modality": "text",
                "source": { "source_type": "doc", "source_id": "chain" }
            }
        }
    ]);
    fun.invoke_str(
        "kb.miu.put_many",
        json!({ "kb_id": kb_id, "drafts": drafts }),
        None,
    )
    .await
    .unwrap();

    let claim = "混合检索使用 RRF 融合多路信号";

    let related = fun
        .invoke_str(
            "kb.retrieve.related",
            json!({ "kb_id": kb_id, "text": claim, "limit": 10 }),
            None,
        )
        .await
        .unwrap();
    assert!(
        related.data["count"].as_u64().unwrap() >= 1,
        "应返回相关结果集"
    );

    let multi = fun
        .invoke_str(
            "kb.retrieve.multi_source",
            json!({ "kb_id": kb_id, "text": claim, "limit": 10 }),
            None,
        )
        .await
        .unwrap();
    assert!(
        multi.data["source_count"].as_u64().unwrap() >= 2,
        "RRF 相关内容应来自多个源"
    );

    let verity = fun
        .invoke_str(
            "kb.assess.verity",
            json!({ "kb_id": kb_id, "text": claim }),
            None,
        )
        .await
        .unwrap();
    assert!(verity.data["verity"].as_f64().unwrap() > 0.0);
    assert!(verity.data["evidence"].as_array().unwrap().len() >= 1);
    let stance = verity.data["stance"].as_str().unwrap();
    assert!(matches!(stance, "supported" | "weak" | "insufficient"));

    let _ = std::fs::remove_dir_all(&dir);
}

fn now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".into())
}
