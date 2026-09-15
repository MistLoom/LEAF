# Architecture overview

English · [中文](architecture.zh-CN.md)

## Model

An **Ability** is a capability unit. External entrypoints and cross-ability calls share one **Gateway** envelope: `instance + method + args`.

```
Ports (CLI / HTTP / MCP / Fun / …)
        │
     Gateway
        │
  Ability instances
        │
  AbilityCtx::call   (internal composition; not limited by Expose)
```

## Layers

| Layer | Path | Role |
|---|---|---|
| Contract | `frame/support` | Manifest, Ctx, lifecycle, `#[client]` |
| Transport | `frame/core`, `frame/port-*` | Gateway, Ports, Expose, controllers |
| Engine | `apps/ability-core` | Host, EventBus, Chain |
| Composition | `frame/compose` | Catalog, Bootstrap, ToolHost |
| Abilities | `ability/*` | One crate per Ability |
| Apps | `apps/*` | Scenario assembly (`runtime`, `agent-test`, …) |
| External SDK | `external/*` | Experimental out-of-tree authoring |

## Rules

1. Ports depend on `Gateway`; they do not call Ability crates directly
2. Abilities do not branch on whether the caller was MCP, CLI, or LLM tools
3. Expose gates **Port → Gateway** only; `ctx.call` ignores Expose
4. Protocol identity lives in declarations and Ports — not as kernel business types
5. External packages must not depend on `frame-core`, ports, or `ability-core`

## Status

- `external/` is experimental
- Multi-language Abilities (Python, TypeScript, …) are planned; Rust is primary today
- `apps/agent-test` is the minimal agent example
