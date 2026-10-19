# 快速开始

[English](getting-started.md) · 中文

## 环境

- 较新的 Rust 工具链（workspace 使用 edition 2024）
- 在仓库根目录执行命令与开发

## 创建应用

在 `apps/<name>` 新增 crate，加入 workspace `members`，依赖 `frame-compose`。保留默认 features，以链入 `common/ability`。

```rust
use frame_compose::AppBuilder;

let app = AppBuilder::new()
    .app("my-app")
    .bootstrap()
    .await;
```

`app` 是入口应用，名称与目录一致。`AppHandle` 提供 `gateway`、`host`、`expose`、`catalog` 与 `control()`。

## 调用

```rust
use frame_core::{Gateway, InstanceId, InvokeRequest, MethodId};
use serde_json::json;

let response = app
    .gateway
    .invoke(InvokeRequest::new(
        InstanceId::new("read_file")?,
        MethodId::new("run")?,
        json!({ "path": "README.md" }),
    ))
    .await?;
```

同应用的 `ctx.call` 不受 `AbilityExpose` 限制。调用其他应用的 Ability 要求目标 `open: true`。`common` 对入口应用开放。

## 订阅事件

```rust
use ability_support::{EventClass, EventFilter, EventListener};

let mut events = app
    .control()
    .listen(EventFilter {
        class: Some(EventClass::Ability),
        ..Default::default()
    })
    .await?;

while let Some(event) = events.next().await {
    println!("{} {}", event.name(), event.payload());
}
```

`listen` 收到本应用、`common` 以及已开放的其他应用 Ability 事件。`intercept` 作用于本应用与 `common`。

## 示例应用

`apps/agent-test` 是简易 Agent 示例：会话、回合循环与工具执行共用同一 `Gateway`。

```bash
cargo run -p apps-agent-test -- run -p "解释 README.md"
cargo run -p apps-agent-test -- serve
```

`apps/web-agent` 装配 `common/ability` 的通用能力与 `agent-test` 开放的 Agent 回路。生态插件为可选，见 [生态插件](ecosystem-plugins.zh-CN.md)。

```bash
cargo run -p web-agent -- doctor
cargo run -p web-agent -- serve
```

## 后续阅读

- [清单与裁剪](catalog-and-expose.zh-CN.md) — 协议清单与实例过滤的代码用法
- [编写 Ability](writing-abilities.zh-CN.md)
