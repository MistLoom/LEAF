# Ecosystem plugins

English · [中文](ecosystem-plugins.zh-CN.md)

Optional abilities. They are not part of the default assembly. `claude_mod` and `dsh_plugin` live in `common/ability`, with `open: true`, and share the Node host in `common/ability/plugin_host`. `apps/web-agent` is the reference application that enables them.

LEAF does not embed a Claude Code or DSH runtime. Enable the two abilities in assembly config. Each keeps its own plugin list. When both are enabled they share one Node host process.

| Ability | Plugins |
|---|---|
| `claude_mod` | Claude Code mods (`hooks/hooks.json`) |
| `dsh_plugin` | DSH plugins (single-file scripts or Cordis packages) |

```toml
frame-compose = { features = ["ability-dispatch", "ability-claude-mod", "ability-dsh-plugin"] }
```

Plugin paths are set in Expose `config.plugins`. `doctor` checks Node and those paths.

## Invoke

`claude_mod` forwards session and turn events to mod hooks (for example `session.created` → `session.start`). Direct invoke:

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

Tools registered by a DSH plugin appear on `dispatch` under names that start with `eco__dsh__`, and run through `dispatch.run` when the model calls them in the current turn.

Reference application:

```bash
cargo run -p web-agent -- doctor
cargo test -p ability-plugin-host
```
