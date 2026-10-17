# leafEX

**One Ability. Many surfaces — Agent, MCP, CLI, and HTTP.**

leafEX is a Rust runtime for AI systems. Tools, skills, MCP endpoints, and services are modeled as **Abilities** and invoked through a single `Gateway` (`instance + method + args`).

Implement a capability once. Project it onto LLM `tool_calls`, MCP tools, CLI, or HTTP at assembly time — without encoding protocol identity into business logic.

[Features](#features) · [Usage in apps](#usage-in-apps) · [Quick start](#quick-start) · [Manual](#manual) · [Status](#status)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-early%20preview-yellow.svg)](#status)

中文：[README.zh-CN.md](README.zh-CN.md)

---

## Features

- **Protocol-agnostic Abilities** — CLI, HTTP, MCP, Fun, gRPC, and LLM tools are projections, not kernel types
- **Single invoke pipe** — Ports and agent `dispatch` call `Gateway.invoke`; side doors are out of scope
- **Agent-oriented building blocks** — session, multi-turn tool loops, skills, OpenAI-compatible chat, file and shell helpers
- **Composable** — typed clients + `ctx.call`; hot-load instances; outbound visibility controlled by the composition root
- **Declaration-driven codegen** — `ability.yml` plus protocol `info.yml` generate providers; business code stays in `src/logic/`
- **`external/` is experimental** — the standalone author SDK and probes may change

## Usage in apps

New applications under `apps/` typically discover Abilities, filter by protocol and expose policy, then bootstrap a `Gateway`.

### List Abilities that declare `tools` or `mcp`

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

`sources_by_protocol` is a **static workspace view** (declaration-time). It does not require instances to be loaded.

### Filter instances with `AbilityExpose`

Expose is keyed by **instance id**. It controls whether a source is loaded and which bind-time `config` is applied. It does **not** redefine methods, and it does **not** restrict ability-to-ability `ctx.call`.

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

// tools ∩ linked ∩ expose-enabled
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

### Bootstrap a scenario app

| App | Role |
|-----|------|
| `apps/agent-test` | **Native** abilities only (session, turn, dispatch) |
| `apps/web-agent` | **External ecosystems** (Claude Mod / DSH); see [ecosystem-plugins.md](manual/ecosystem-plugins.md) |

Custom apps can follow the `agent-test` assembly pattern; for Mod/DSH use `web-agent` as the reference:

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
let gateway = app.gateway.clone(); // Arc<dyn Gateway> — see AppHandle in frame-compose

let _ = gateway
    .invoke(InvokeRequest::new(
        InstanceId::new("read_file")?,
        MethodId::new("run")?,
        json!({ "path": "README.md" }),
    ))
    .await?;
```

Protocol visibility on a Port is registration + access gate: only loaded instances that declared the Port’s protocol remain visible on that surface. Details: [manual/catalog-and-expose.md](manual/catalog-and-expose.md).

## Quick start

Requires a recent Rust toolchain (workspace edition 2024).

```bash
cargo run -p apps-runtime -- catalog list
cargo run -p apps-agent-test -- run -p "Explain the structure of README.md"
cargo build -p ability-echo
```

## Write an Ability

```
ability/<name>/
  ability.yml
  build.rs
  src/logic/          # business code
  src/generated/      # codegen output
  mcp|cli|tools|…/info.yml
```

Cross-ability calls use the peer **client** (`use XxxExt`), never the peer Provider. See [manual/writing-abilities.md](manual/writing-abilities.md).

## Repository layout

```
ability/                 Built-in Abilities
frame/                   support, core, compose, port-*, protocol
apps/
  ability-core/          Engine
  runtime/               Reference runtime
  agent-test/            Minimal agent example
external/                Experimental author SDK, codegen, probes
manual/                  Public handbook (EN + ZH)
```

## Manual

| English | 中文 | Contents |
|---|---|---|
| [getting-started.md](manual/getting-started.md) | [getting-started.zh-CN.md](manual/getting-started.zh-CN.md) | Build & first bootstrap |
| [catalog-and-expose.md](manual/catalog-and-expose.md) | [catalog-and-expose.zh-CN.md](manual/catalog-and-expose.zh-CN.md) | Protocol lists & expose filters |
| [writing-abilities.md](manual/writing-abilities.md) | [writing-abilities.zh-CN.md](manual/writing-abilities.zh-CN.md) | Authoring Abilities |
| [architecture.md](manual/architecture.md) | [architecture.zh-CN.md](manual/architecture.zh-CN.md) | Public architecture model |

## Status

Early preview. APIs and layout may change. Native agent: `apps/agent-test`. Ecosystem plugins: `apps/web-agent`.

**`external/` is experimental** and may break while the authoring story settles.

**Roadmap — multi-language Abilities.** Rust is primary today. First-class Python and TypeScript ability authoring/loading is planned so the same Gateway / Port / Expose model can host polyglot planes.

## Contributing

1. Keep dependency edges one-way ([architecture](manual/architecture.md))
2. Keep business logic in Ability crates, not the engine
3. Extend via new Abilities or Ports rather than special-casing `Gateway`

## License

Licensed under the [Apache License 2.0](LICENSE).
