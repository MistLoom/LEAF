# Write an Ability

English · [中文](writing-abilities.zh-CN.md)

## Package layout

```
common/ability/<name>/       # shared across apps
apps/<app>/ability/<name>/   # owned by that app
  ability.yml
  build.rs
  src/
    logic/          # business code
    generated/      # codegen output
  cli|mcp|tools|grpc|skill/
    info.yml
```

## Handler and factory

- **Handler** (default): one `src/logic/<method>.rs` per method; export `generated::provider::provider`
- **Factory** (`factory: true`): custom `Ability` + `into_factory` when bind-time config constructs the instance

## Minimal declaration

```yaml
framework_version: "1"
id: echo
protocols: [tools, mcp, cli, http, fun]
depends: []
stateful: false
methods:
  - id: run
    description: Echo text
    input:
      type: object
      properties:
        text: { type: string }
      required: [text]
```

Keep `framework_version` aligned with the framework. Protocol `info.yml` files, when present, must match.

## Visibility

`open` defaults to false: only the owning app may call the Ability. Set `open: true` when other apps may call it. Event intercept stays with the owner. Other apps may listen to an open Ability through `AppControl`.

## Cross-ability calls

Depend on the peer **client** contract only:

```rust
use ability_echo::EchoExt;

ctx.echo().run(args).await?;
```

## Surface checklist

| Goal | Declare |
|---|---|
| LLM tools | `tools` in `protocols` |
| MCP | `mcp` + `mcp/info.yml` |
| CLI | `cli` + `cli/info.yml` |
| HTTP / Fun | `http` / `fun` in `protocols` |

Enable the instance through `AbilityExpose` in the app. See [Catalog & expose](catalog-and-expose.md).
