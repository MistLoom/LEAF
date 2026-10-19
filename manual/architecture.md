# Architecture

English · [中文](architecture.zh-CN.md)

## Model

An **app** owns its Abilities under `apps/<app>/ability/<name>`. Ports and cross-ability calls share one **Gateway** envelope: `instance + method + args`.

```
Ports (CLI / HTTP / MCP / Fun / …)
        │
     Gateway
        │
  entry app
        ├─ owned Abilities
        └─ open Abilities of other apps
```

`ability.yml` field `open` defaults to false. Another app may call the Ability only when `open: true`.

Event listen and intercept are registered through `AppControl` on the entry app. Intercept applies to Abilities that app owns. Listen also accepts events from other apps' open Abilities. The engine implements the filter; apps do not keep a second bus.

## Layers

| Layer | Path | Role |
|---|---|---|
| Contract | `frame/support` | Manifest, Ctx, lifecycle, `#[client]` |
| Transport | `frame/core`, `frame/port-*` | Gateway, Ports, Expose, controllers |
| Engine | `apps/ability-core` | Host, EventBus, admission for calls and events |
| Composition | `frame/compose` | Catalog, Bootstrap, `AppControl` |
| Abilities | `common/ability/*`, `apps/<app>/ability/*` | Shared Abilities, or one crate owned by a scenario app |
| Apps | `apps/*` | Scenario process (`agent-test`, `web-agent`) |
| External SDK | `external/*` | Experimental out-of-tree authoring |

## Ownership

| Owner | Abilities |
|---|---|
| `common` | Shared Abilities (config, store, files, shell, task, kb, the reference samples, and optional `claude_mod` / `dsh_plugin`). `open`. The entry app may call, listen, and intercept them. |
| `agent-test` | Agent loop (`session`, `turn`, `dispatch`, `chat`, `prompt`, `skill`), `open` |
| `web-agent` | Reference web application. Compat fixtures live in its tests. |

## Rules

1. Ports depend on `Gateway`; they do not call Ability crates directly
2. Abilities do not branch on whether the caller was MCP, CLI, or LLM tools
3. Expose gates **Port → Gateway** only; it does not grant a cross-app call
4. A cross-app call requires `open: true` on the target
5. Intercept stays with the owning app, except `common` Abilities, which the entry app may intercept. Listen of another scenario app requires `open: true`
6. Protocol identity lives in declarations and Ports — not as kernel business types
7. External packages must not depend on `frame-core`, ports, or `ability-core`

## Status

- `external/` is experimental
- Multi-language Abilities (Python, TypeScript, …) are planned; Rust is primary today
- `apps/agent-test` is the minimal agent example
- `apps/web-agent` is the reference web agent
