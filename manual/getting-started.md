# Getting started

English · [中文](getting-started.zh-CN.md)

## Requirements

- Recent Rust toolchain (workspace edition 2024)
- Work from the repository root

## Bootstrap a Gateway in an app

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

`AppHandle` also exposes `host`, `expose`, and `catalog` for further wiring.

## Minimal agent example

`apps/agent-test` is a minimal agent example: session, turn loop, and tool execution on the same Gateway.

```bash
cargo run -p apps-agent-test -- run -p "Explain README.md"
cargo run -p apps-agent-test -- serve
```

## Reference runtime

`apps/runtime` is the reference multi-port runtime binary.

```bash
cargo run -p apps-runtime -- catalog list
```

## Next

- [Catalog & expose](catalog-and-expose.md) — protocol lists and instance filters in code
- [Write an Ability](writing-abilities.md)
