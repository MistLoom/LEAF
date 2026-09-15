# leafEX

**一套 Ability，同时服务 Agent、MCP、CLI 与 HTTP。**

leafEX 是面向 AI 系统的 Rust 能力运行时。工具、Skill、MCP、服务统一建模为 **Ability**，经同一条 `Gateway` 调用（`instance + method + args`）。

业务实现一次，在装配阶段投影到 LLM `tool_calls`、MCP tools、CLI 或 HTTP，无需在能力内部绑定协议身份。

[功能](#功能) · [在应用中使用](#在应用中使用) · [快速开始](#快速开始) · [手册](#手册) · [状态](#状态)

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
- **声明驱动 codegen**：`ability.yml` 与协议 `info.yml` 生成 Provider，业务代码位于 `src/logic/`
- **`external/` 为实验性**：独立作者 SDK 与探针仍可能变更

## 在应用中使用

在 `apps/` 下新建场景应用时，通常先发现 Ability，再按协议与 Expose 策略过滤，最后 `bootstrap` 得到 `Gateway`。

### 获取声明了 `tools` / `mcp` 的 Ability 清单

```rust
use ability_support::ProtocolKind;
use frame_compose::WorkspaceCatalog;

let catalog = WorkspaceCatalog::discover();

let tools = catalog.sources_by_protocol(ProtocolKind::Tools);
let mcp = catalog.sources_by_protocol(ProtocolKind::Mcp);

for entry in tools.iter().filter(|s| s.linked) {
    println!("{} ({})", entry.id, entry.rel_root);
}
```

`sources_by_protocol` 是**工作区静态视图**（声明期），不依赖实例是否已 load。

### 使用 `AbilityExpose` 过滤实例

Expose 以**实例 id** 为键，控制是否 load 以及绑定期 `config`。它不重新定义方法，也不限制能力之间的 `ctx.call`。

```rust
use ability_support::ProtocolKind;
use frame_compose::{AppBuilder, WorkspaceCatalog};
use frame_core::AbilityExpose;
use serde_json::json;

let catalog = WorkspaceCatalog::discover();

let expose = AbilityExpose::from_value(&json!({
    "unknown": "deny",
    "defaults": { "enabled": true },
    "abilities": {
        "write_file": { "enabled": false },
        "read_file": {
            "enabled": true,
            "config": { "max_size": 1_048_576 }
        }
    }
}))?;

let linked: Vec<&str> = catalog.linked_sources().map(|s| s.id.as_str()).collect();
let enabled_ids = expose.filter_enabled(linked);

// tools ∩ 已链接 ∩ Expose 启用
let tools_for_agent: Vec<_> = catalog
    .sources_by_protocol(ProtocolKind::Tools)
    .into_iter()
    .filter(|s| s.linked && expose.is_enabled(&s.id))
    .collect();

let app = AppBuilder::new()
    .expose(expose)
    .bootstrap()
    .await?;
```

### 装配场景应用

`apps/agent-test` 是**简易 Agent 示例**（会话 + 回合循环 + 工具）。自定义应用采用相同模式：

```rust
use frame_compose::AppBuilder;
use frame_core::{AbilityExpose, Gateway, InvokeRequest};
use ability_support::{InstanceId, MethodId};
use serde_json::json;

let expose = AbilityExpose::from_value(&json!({
    "unknown": "deny",
    "defaults": { "enabled": true },
    "abilities": {
        "read_file": { "enabled": true },
        "write_file": { "enabled": false }
    }
}))?;

let app = AppBuilder::new().expose(expose).bootstrap().await?;
let gateway = app.gateway.clone();

let _ = gateway
    .invoke(InvokeRequest::new(
        InstanceId::new("read_file")?,
        MethodId::new("run")?,
        json!({ "path": "README.md" }),
    ))
    .await?;
```

Port 侧可见性由「已 load + 已登记协议 + access gate」共同决定。详见 [manual/catalog-and-expose.zh-CN.md](manual/catalog-and-expose.zh-CN.md)。

## 快速开始

需要较新的 Rust 工具链（workspace 使用 edition 2024）。

```bash
cargo run -p apps-runtime -- catalog list
cargo run -p apps-agent-test -- run -p "解释 README.md 的结构"
cargo build -p ability-echo
```

## 编写 Ability

```
ability/<name>/
  ability.yml
  build.rs
  src/logic/          # 业务
  src/generated/      # codegen
  mcp|cli|tools|…/info.yml
```

跨能力调用只依赖对方 **client**（`use XxxExt`），不依赖对方 Provider。见 [manual/writing-abilities.zh-CN.md](manual/writing-abilities.zh-CN.md)。

## 仓库结构

```
ability/                 内置 Ability
frame/                   support、core、compose、port-*、protocol
apps/
  ability-core/          引擎
  runtime/               参考运行时
  agent-test/            简易 Agent 示例
external/                实验性：作者 SDK、codegen、探针
manual/                  对外公开手册（中英双语）
```

## 手册

| English | 中文 | 内容 |
|---|---|---|
| [getting-started.md](manual/getting-started.md) | [getting-started.zh-CN.md](manual/getting-started.zh-CN.md) | 构建与首次装配 |
| [catalog-and-expose.md](manual/catalog-and-expose.md) | [catalog-and-expose.zh-CN.md](manual/catalog-and-expose.zh-CN.md) | 协议清单与 Expose 过滤 |
| [writing-abilities.md](manual/writing-abilities.md) | [writing-abilities.zh-CN.md](manual/writing-abilities.zh-CN.md) | 编写 Ability |
| [architecture.md](manual/architecture.md) | [architecture.zh-CN.md](manual/architecture.zh-CN.md) | 公开架构说明 |

## 状态

早期预览，API 与布局可能变化。Agent 路径以 `apps/agent-test` 为参考。

**`external/` 为实验性**，作者体验定型前可能发生破坏性变更。

**路线图 — 多语言 Ability。** 当前以 Rust 为主；计划支持 Python、TypeScript 等语言的 Ability 编写与加载，使同一套 Gateway / Port / Expose 承载多语言能力平面。

## 贡献

1. 保持依赖单向（见 [architecture.zh-CN.md](manual/architecture.zh-CN.md)）
2. 业务放在 Ability crate，不写入引擎
3. 优先通过新 Ability 或 Port 扩展，避免特化 `Gateway`

## 许可证

本项目采用 [Apache License 2.0](LICENSE)。
