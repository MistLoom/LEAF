# 生态插件

[English](ecosystem-plugins.md) · 中文

可选能力，不在框架默认装配中。`claude_mod` 与 `dsh_plugin` 位于 `common/ability`，`open: true`，共用 `common/ability/plugin_host` 的 Node 宿主。`apps/web-agent` 是启用它们的参考应用。

LEAF 不内置 Claude Code 或 DSH 运行时。在装配配置中启用这两个 Ability。二者各自维护插件列表；同时启用时共用同一 Node 宿主进程。

| Ability | 插件 |
|---|---|
| `claude_mod` | Claude Code Mod（`hooks/hooks.json`） |
| `dsh_plugin` | DSH 插件（单文件脚本或 Cordis 包） |

```toml
frame-compose = { features = ["ability-dispatch", "ability-claude-mod", "ability-dsh-plugin"] }
```

插件路径写在 Expose 的 `config.plugins`。`doctor` 检查 Node 与路径。

## 调用

`claude_mod` 将会话与回合事件转发到 Mod 钩子（例如 `session.created` → `session.start`）。直接调用：

```rust
use frame_core::{Gateway, InstanceId, InvokeRequest, MethodId};
use serde_json::json;

let _ = gateway
    .invoke(InvokeRequest::new(
        InstanceId::new("claude_mod")?,
        MethodId::new("invoke")?,
        json!({
            "tool": "session.start",
            "args": { "session_id": "s1" }
        }),
    ))
    .await?;
```

DSH 插件注册的工具进入 `dispatch`，名称以 `eco__dsh__` 开头，并在模型本轮调用时经 `dispatch.run` 执行。

参考应用：

```bash
cargo run -p web-agent -- doctor
cargo test -p ability-plugin-host
```
