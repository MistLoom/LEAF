//! 业务逻辑：run

use std::time::Duration;

use ability_support::{decode_text_bytes, AbilityCtx, Error, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Deserialize)]
struct Args {
    command: String,
    working_directory: Option<String>,
    #[serde(default)]
    block_until_ms: Option<u64>,
}

pub async fn run(_ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
    let a: Args = serde_json::from_value(args).map_err(|e| Error::invalid_args(e.to_string()))?;
    let ms = a.block_until_ms.unwrap_or(30_000);
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        // 优先 UTF-8 代码页，减少 type/Get-Content 混用时的编码歧义
        c.args(["/C", &format!("chcp 65001>nul & {}", a.command)]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &a.command]);
        c
    };
    if let Some(dir) = &a.working_directory {
        cmd.current_dir(dir);
    }
    cmd.kill_on_drop(true);
    let fut = async {
        cmd.output()
            .await
            .map_err(|e| Error::failed(format!("bash 启动失败: {e}")))
    };
    let output = match timeout(Duration::from_millis(ms), fut).await {
        Ok(r) => r?,
        Err(_) => {
            return Err(Error::failed(format!(
                "bash 超时 ({ms} ms): {}",
                a.command
            )))
        }
    };
    Ok(json!({
        "command": a.command,
        "exit_code": output.status.code(),
        "stdout": decode_text_bytes(&output.stdout),
        "stderr": decode_text_bytes(&output.stderr),
    }))
}
