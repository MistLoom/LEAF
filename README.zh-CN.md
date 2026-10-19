# LEAF

**一套 Ability，同时服务 Agent、MCP、CLI 与 HTTP。**

LEAF 是面向 AI 系统的 Rust 能力运行时。工具、Skill、MCP、服务统一建模为 **Ability**，经同一条 `Gateway` 调用（`instance + method + args`）。

业务实现一次，在装配阶段投影到 LLM `tool_calls`、MCP tools、CLI 或 HTTP，无需在能力内部绑定协议身份。可用该框架实现个人或企业内部的 AI 系统，以及连接模型、工具与插件的 harness。

[功能](#功能) · [创建应用](#创建应用) · [调用](#调用) · [订阅事件](#订阅事件) · [快速开始](#快速开始) · [手册](#手册) · [状态](#状态)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-early%20preview-yellow.svg)](#状态)

English: [README.md](README.md)

---

## 功能

- **协议无关的 Ability**：CLI / HTTP / MCP / Fun / gRPC / LLM tools 均为投影，而非内核类型
- **单一调用管道**：Port 与 Agent `dispatch` 均经 `Gateway.invoke`
- **面向 Agent 的积木**：会话、多轮工具循环、Skill、OpenAI 兼容 chat、文件与终端能力等
- **可组合**：typed client + `ctx.call`；实例可热装载；对外开放由组合根裁剪
- **应用级事件**：入口应用通过 `AppControl` 监听与拦截 Ability 事件
- **声明驱动 codegen**：`ability.yml` 与协议 `info.yml` 生成 Provider，业务代码位于 `src/logic/`
- **`external/` 为实验性**：独立作者 SDK 与探针仍可能变更

## 创建应用

在 `apps/` 下新增 crate，并写入 workspace `members`。依赖 `frame-compose` 并保留默认 features，以链入 `common/ability` 中的通用能力。

```rust
use frame_compose::AppBuilder;

let app = AppBuilder::new()
    .app("my-app")
    .bootstrap()
    .await;
```

`app` 为入口应用，名称与 `apps/<name>` 目录一致。`bootstrap` 返回 `AppHandle`（`gateway`、`host`、`expose`、`catalog`、`control`）。

| 路径 | 归属 |
|---|---|
| `common/ability/<name>/` | 通用 Ability。入口应用可调用、监听并拦截 |
| `apps/<app>/ability/<name>/` | 该应用私有。其他应用仅在 `open: true` 时可调用 |

`apps/agent-test` 为 Agent 回路示例。`apps/web-agent` 为 Web 参考应用，并使用上述两类已开放 Ability。装配细节见 [快速开始](manual/getting-started.zh-CN.md)。

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

Ability 之间只依赖对方 **client**，不依赖其 Provider：

```rust
use ability_echo::EchoExt;

ctx.echo().run(args).await?;
```

`AbilityExpose` 裁剪 Port 上的加载与可见性，不限制同应用 `ctx.call`。跨应用调用要求目标 `open: true`。见 [清单与裁剪](manual/catalog-and-expose.zh-CN.md)。

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

`AppHandle::control()` 返回入口应用的 `AppControl`。`listen` 投递本应用、`common`，以及其他应用中 `open: true` 的 Ability 事件。`intercept` 作用于本应用与 `common`。

## 快速开始

需要较新的 Rust 工具链（workspace 使用 edition 2024）。

```bash
cargo run -p web-agent -- doctor
cargo run -p web-agent -- serve
cargo run -p apps-agent-test -- run -p "解释 README.md 的结构"
```

## 编写 Ability

```
common/ability/<name>/          # 通用
apps/<app>/ability/<name>/      # 归属该应用
  ability.yml
  build.rs
  src/logic/
  src/generated/
  mcp|cli|tools|…/info.yml
```

见 [编写 Ability](manual/writing-abilities.zh-CN.md)。

## 仓库结构

```
frame/                   support、core、compose、port-*、protocol
common/ability/          通用 Ability
apps/
  ability-core/          引擎
  agent-test/ability/    Agent 回路 Ability
  web-agent/             Web 参考应用
external/                实验性：作者 SDK、codegen、探针
manual/                  对外公开手册（中英双语）
```

## 手册

| English | 中文 | 内容 |
|---|---|---|
| [getting-started.md](manual/getting-started.md) | [getting-started.zh-CN.md](manual/getting-started.zh-CN.md) | 创建应用、调用与订阅 |
| [catalog-and-expose.md](manual/catalog-and-expose.md) | [catalog-and-expose.zh-CN.md](manual/catalog-and-expose.zh-CN.md) | 协议清单与 Expose 过滤 |
| [writing-abilities.md](manual/writing-abilities.md) | [writing-abilities.zh-CN.md](manual/writing-abilities.zh-CN.md) | 编写 Ability |
| [architecture.md](manual/architecture.md) | [architecture.zh-CN.md](manual/architecture.zh-CN.md) | 公开架构说明 |
| [ecosystem-plugins.md](manual/ecosystem-plugins.md) | [ecosystem-plugins.zh-CN.md](manual/ecosystem-plugins.zh-CN.md) | 可选：Claude Code Mod 与 DSH |

## 状态

早期预览，API 与布局可能变化。原生 Agent 以 `apps/agent-test` 为参考，Web 参考应用为 `apps/web-agent`。

**`external/` 为实验性**，作者体验定型前可能发生破坏性变更。

**路线图 — 多语言 Ability。** 当前以 Rust 为主；计划支持 Python、TypeScript 等语言的 Ability 编写与加载，使同一套 Gateway / Port / Expose 承载多语言能力平面。

## 贡献

1. 保持依赖单向（见 [architecture.zh-CN.md](manual/architecture.zh-CN.md)）
2. 业务放在 Ability crate，不写入引擎
3. 优先通过新 Ability 或 Port 扩展，避免特化 `Gateway`

## 许可证

本项目采用 [Apache License 2.0](LICENSE)。
