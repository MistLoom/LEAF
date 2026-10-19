# Getting started

English · [中文](getting-started.zh-CN.md)

## Requirements

- Recent Rust toolchain (workspace edition 2024)
- Work from the repository root

## Create an application

Add a crate at `apps/<name>`, list it in workspace `members`, and depend on `frame-compose`. Keep the default features so `common/ability` is linked.

```rust
use frame_compose::AppBuilder;

let app = AppBuilder::new()
    .app("my-app")
    .bootstrap()
    .await;
```

`app` is the entry application and matches the directory name. `AppHandle` exposes `gateway`, `host`, `expose`, `catalog`, and `control()`.

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

Same-app `ctx.call` is not gated by `AbilityExpose`. A call into another app requires `open: true` on the target. `common` is open to the entry app.

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

`listen` delivers events from this app, from `common`, and from other apps' open Abilities. `intercept` applies to this app and to `common`.

## Example applications

`apps/agent-test` is a minimal agent example: session, turn loop, and tool execution on the same Gateway.

```bash
cargo run -p apps-agent-test -- run -p "Explain README.md"
cargo run -p apps-agent-test -- serve
```

`apps/web-agent` loads shared Abilities from `common/ability` and the open agent loop from `agent-test`. Ecosystem plugins are optional; see [Ecosystem plugins](ecosystem-plugins.md).

```bash
cargo run -p web-agent -- doctor
cargo run -p web-agent -- serve
```

## Next

- [Catalog & expose](catalog-and-expose.md) — protocol lists and instance filters in code
- [Write an Ability](writing-abilities.md)
