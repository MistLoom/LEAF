# Ecosystem plugins

English · [中文](ecosystem-plugins.zh-CN.md)

Leaf does not embed Claude Code or DSH runtimes. Enable optional abilities `claude_mod` and `dsh_plugin` in assembly config. Each ability owns its plugin list; they share one Node host process when both are enabled.

| Ability | Loads |
|---|---|
| `claude_mod` | Claude Code mods (`hooks/hooks.json`) |
| `dsh_plugin` | DSH plugins (single-file thin scripts or Cordis packages) |

## Requirements

- Node.js on the host
- Default host entrypoint: `apps/plugin-host/run.mjs` (override with `LEAF_PLUGIN_HOST`, `LEAF_PLUGIN_NODE`)
- Cordis path: run `npm ci` in `apps/plugin-host` before first use

## Configuration

Plugins live under `abilities.<id>.config.plugins`. There is no top-level `plugins` key.

```yaml
abilities:
  claude_mod:
    enabled: true
    config:
      plugins:
        - path: ./vendor/my-mod
  dsh_plugin:
    enabled: true
    config:
      plugins:
        - path: ./packages/my-tool.mjs
        - path: ./packages/my-cordis-tool.mjs
          loader: cordis   # thin | cordis | external | auto
```

### DSH `loader`

| Value | Use when |
|---|---|
| `thin` | Single file with `ctx.tools.register`; no `@deepseek-ai/*` imports |
| `cordis` | Marketplace Cordis plugin (`defineTool`, `apply`, inject) |
| `auto` | Host inspects the file; Cordis heuristics apply |
| `external` | `LEAF_DSH_RUNTIME` points at your own runtime module |

Version pins and supported npm ranges: `apps/plugin-host/dsh-runtime.compat.json`. `doctor` reports embedded DSH status; set `LEAF_DSH_STRICT=1` to fail on drift.

## Tools and hooks

On startup the host merges plugin entries and registers `EcosystemDispatchAspect`. Model-visible tool names:

`eco__{origin}__{instanceId}__{toolName}`

(`instanceId` is configurable; default is a hash of the plugin path.)

Plugin code runs in Node. File and process access goes through the host outbound API (`$.fs.*`, `$.process.run`, …), mapped to Leaf abilities — not a second file stack in Rust. Surface list: `ability_plugin_host::mod_outbound_surfaces()`.

`claude_mod` forwards session/turn events to mod hooks (for example `session.created` → `session.start`). Invoke hooks directly via `gateway.invoke(claude_mod, invoke, { tool, args })`.

## Compose features

```toml
frame-compose = { features = ["ability-dispatch", "ability-claude-mod", "ability-dsh-plugin"] }
```

## Reference app: `web-agent`

Use `apps/web-agent` for end-to-end checks. `apps/agent-test` does not load ecosystem plugins.

| Item | Path |
|---|---|
| Assembly | `apps/web-agent/assembly.yaml` |
| Default plugin list | `apps/web-agent/dsh/web-profile.plugins.yaml` |
| Example list | `apps/web-agent/dsh/official.web.plugins.example.yaml` |

```bash
cargo run -p web-agent -- serve
cargo run -p web-agent -- doctor
cargo run -p web-agent -- --plugins apps/web-agent/dsh/official.web.plugins.example.yaml serve
```

Web UI (when serving): `http://127.0.0.1:8790`.

## Verify

```bash
cargo test -p plugin-compat
cargo test -p ability-plugin-host
cargo test -p web-agent --test boot --test ecosystem --test profile_merge
cargo run -p web-agent -- doctor
```

Fixtures: `apps/plugin-compat/tests/fixtures`, `apps/web-agent/tests/fixtures`.

### Market plugins (optional)

Fetch vendor trees, then run ignored integration tests:

```powershell
apps/web-agent/scripts/fetch-ecosystem-vendor.ps1
```

```bash
cargo test -p web-agent --test market_plugins -- --ignored
cargo run -p web-agent -- --plugins apps/web-agent/market/market.plugins.yaml doctor
```

Manifest: `apps/web-agent/market/market.plugins.yaml`.

## Scope

| Supported in Leaf | Not targeted |
|---|---|
| Mod load, hook dispatch, `tool.check`, outbound host API | Claude Code desktop UI parity |
| DSH thin + embedded Cordis → `dispatch` | Full DSH Harness presets, hot reload |
| `web-agent` doctor / invoke / Web SSE for `ui.*` hooks | In-app marketplace installer |
