//! 会话快照持久化（`data/web_agent/sessions`）。

use std::path::PathBuf;
use std::sync::Arc;

use frame_core::Gateway;
use serde_json::Value;
use tracing::{info, warn};

use crate::gateway::invoke;

pub fn sessions_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data/web_agent/sessions")
}

pub async fn save_session(gateway: &Arc<dyn Gateway>, session_id: &str) -> std::io::Result<()> {
    let snap = invoke(
        gateway,
        "session",
        "export",
        serde_json::json!({ "id": session_id }),
    )
    .await
    .map_err(|e| std::io::Error::other(e.to_string()))?;
    std::fs::create_dir_all(sessions_dir())?;
    let path = sessions_dir().join(format!("{session_id}.json"));
    std::fs::write(path, serde_json::to_string_pretty(&snap).unwrap())?;
    Ok(())
}

pub async fn restore_all(gateway: &Arc<dyn Gateway>) -> usize {
    let dir = sessions_dir();
    if !dir.is_dir() {
        return 0;
    }
    let Ok(read) = std::fs::read_dir(&dir) else {
        return 0;
    };
    let mut n = 0usize;
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(snapshot) = serde_json::from_str::<Value>(&raw) else {
            warn!(path = %path.display(), "跳过无效会话快照");
            continue;
        };
        match invoke(
            gateway,
            "session",
            "import",
            serde_json::json!({ "snapshot": snapshot, "mode": "restore" }),
        )
        .await
        {
            Ok(_) => {
                n += 1;
                info!(path = %path.display(), "已恢复会话");
            }
            Err(e) => warn!(path = %path.display(), error = %e, "恢复失败"),
        }
    }
    n
}
