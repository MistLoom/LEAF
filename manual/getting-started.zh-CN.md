# 快速开始

[English](getting-started.md) · 中文

## 环境

- 较新的 Rust 工具链（workspace 使用 edition 2024）
- 在仓库根目录执行命令与开发

## 在应用中装配 Gateway

```rust
use frame_compose::AppBuilder;
use frame_core::AbilityExpose;
use serde_json::json;

let expose = AbilityExpose::from_value(&json!({
    "unknown": "deny",
    "defaults": { "enabled": true },
    "abilities": {
        "read_file": { "enabled": true },
        "write_file": { "enabled": false }
    }
}))?;

let app = AppBuilder::new()
    .expose(expose)
    .bootstrap()
    .await?;

let gateway = app.gateway.clone();
```

`AppHandle` 还提供 `host`、`expose`、`catalog`，供后续接线。

## 简易 Agent 示例

`apps/agent-test` 是简易 Agent 示例：会话、回合循环与工具执行共用同一 `Gateway`。

```bash
cargo run -p apps-agent-test -- run -p "解释 README.md"
cargo run -p apps-agent-test -- serve
```

## 参考运行时

`apps/runtime` 为多 Port 参考运行时二进制。

```bash
cargo run -p apps-runtime -- catalog list
```

## 后续阅读

- [清单与裁剪](catalog-and-expose.zh-CN.md) — 协议清单与实例过滤的代码用法
- [编写 Ability](writing-abilities.zh-CN.md)
