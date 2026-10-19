# LEAF

**One Ability. Many surfaces — Agent, MCP, CLI, and HTTP.**

LEAF is a Rust runtime for AI systems. Tools, skills, MCP endpoints, and services are modeled as **Abilities** and invoked through a single `Gateway` (`instance + method + args`).

Implement a capability once. Project it onto LLM `tool_calls`, MCP tools, CLI, or HTTP at assembly time — without encoding protocol identity into business logic. The framework can be used to implement personal or in-house AI systems, and harnesses that connect a model with tools and plugins.

[Features](#features) · [Create an application](#create-an-application) · [Invoke](#invoke) · [Subscribe](#subscribe) · [Quick start](#quick-start) · [Manual](#manual) · [Status](#status)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-early%20preview-yellow.svg)](#status)

中文：[README.zh-CN.md](README.zh-CN.md)

---

## Features

- **Protocol-agnostic Abilities** — CLI, HTTP, MCP, Fun, gRPC, and LLM tools are projections, not kernel types
- **Single invoke pipe** — Ports and agent `dispatch` call `Gateway.invoke`
- **Agent-oriented building blocks** — session, multi-turn tool loops, skills, OpenAI-compatible chat, file and shell helpers
- **Composable** — typed clients + `ctx.call`; hot-load instances; outbound visibility controlled by the composition root
- **App-level events** — the entry app listens and intercepts Ability events through `AppControl`
- **Declaration-driven codegen** — `ability.yml` plus protocol `info.yml` generate providers; business code stays in `src/logic/`
- **`external/` is experimental** — the standalone author SDK and probes may change

## Create an application

Add a crate under `apps/` and list it in workspace `members`. Depend on `frame-compose` and keep its default features so shared abilities in `common/ability` are linked.

```rust
use frame_compose::AppBuilder;

let app = AppBuilder::new()
    .app("my-app")
    .bootstrap()
    .await;
```

`app` is the entry application and matches the `apps/<name>` directory. `bootstrap` returns `AppHandle` (`gateway`, `host`, `expose`, `catalog`, `control`).

| Path | Owner |
|---|---|
| `common/ability/<name>/` | Shared. The entry app may call, listen, and intercept |
| `apps/<app>/ability/<name>/` | Owned by that app. Other apps may call it only when `open: true` |

`apps/agent-test` is the agent-loop example. `apps/web-agent` is the web reference and consumes those open abilities. Setup: [Getting started](manual/getting-started.md).

## Invoke

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

Cross-ability calls use the peer **client**, never the peer Provider:

```rust
use ability_echo::EchoExt;

ctx.echo().run(args).await?;
```

`AbilityExpose` gates load and visibility on a Port. It does not limit same-app `ctx.call`. A call into another app requires `open: true`. See [Catalog & expose](manual/catalog-and-expose.md).

## Subscribe

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

`AppHandle::control()` returns the entry app's `AppControl`. `listen` delivers events from that app, from `common`, and from other apps' abilities with `open: true`. `intercept` applies to that app and to `common`.

## Quick start

Requires a recent Rust toolchain (workspace edition 2024).

```bash
cargo run -p web-agent -- doctor
cargo run -p web-agent -- serve
cargo run -p apps-agent-test -- run -p "Explain the structure of README.md"
```

## Write an Ability

```
common/ability/<name>/          # shared
apps/<app>/ability/<name>/      # owned by that app
  ability.yml
  build.rs
  src/logic/
  src/generated/
  mcp|cli|tools|…/info.yml
```

See [Write an Ability](manual/writing-abilities.md).

## Repository layout

```
frame/                   support, core, compose, port-*, protocol
common/ability/          Shared Abilities
apps/
  ability-core/          Engine
  agent-test/ability/    Agent-loop Abilities
  web-agent/             Reference web application
external/                Experimental author SDK, codegen, probes
manual/                  Public handbook (EN + ZH)
```

## Manual

| English | 中文 | Contents |
|---|---|---|
| [getting-started.md](manual/getting-started.md) | [getting-started.zh-CN.md](manual/getting-started.zh-CN.md) | Create an app, invoke, subscribe |
| [catalog-and-expose.md](manual/catalog-and-expose.md) | [catalog-and-expose.zh-CN.md](manual/catalog-and-expose.zh-CN.md) | Protocol lists & expose filters |
| [writing-abilities.md](manual/writing-abilities.md) | [writing-abilities.zh-CN.md](manual/writing-abilities.zh-CN.md) | Authoring Abilities |
| [architecture.md](manual/architecture.md) | [architecture.zh-CN.md](manual/architecture.zh-CN.md) | Public architecture model |
| [ecosystem-plugins.md](manual/ecosystem-plugins.md) | [ecosystem-plugins.zh-CN.md](manual/ecosystem-plugins.zh-CN.md) | Optional: Claude Code Mods and DSH |

## Status

Early preview. APIs and layout may change. Native agent: `apps/agent-test`. Reference web application: `apps/web-agent`.

**`external/` is experimental** and may break while the authoring story settles.

**Roadmap — multi-language Abilities.** Rust is primary today. First-class Python and TypeScript ability authoring/loading is planned so the same Gateway / Port / Expose model can host polyglot planes.

## Contributing

1. Keep dependency edges one-way ([architecture](manual/architecture.md))
2. Keep business logic in Ability crates, not the engine
3. Extend via new Abilities or Ports rather than special-casing `Gateway`

## License

Licensed under the [Apache License 2.0](LICENSE).
