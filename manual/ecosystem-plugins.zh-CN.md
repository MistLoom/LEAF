# 外部生态插件

[English](ecosystem-plugins.md) · 中文

Leaf 不内置 Claude Code 或 DSH 运行时。在装配配置中启用可选能力 `claude_mod`、`dsh_plugin`。二者各自维护插件列表；同时启用时共用同一 Node 宿主进程。

| 能力 | 加载对象 |
|---|---|
| `claude_mod` | Claude Code Mod（`hooks/hooks.json`） |
| `dsh_plugin` | DSH 插件（thin 单文件或 Cordis 包） |

## 环境

- 本机已安装 Node.js
- 默认宿主入口：`apps/plugin-host/run.mjs`（可用 `LEAF_PLUGIN_HOST`、`LEAF_PLUGIN_NODE` 覆盖）
- 使用 Cordis 路径时，首次使用前在 `apps/plugin-host` 执行 `npm ci`

## 配置

插件写在 `abilities.<id>.config.plugins` 下，无顶层 `plugins` 字段。

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

| 值 | 适用场景 |
|---|---|
| `thin` | 单文件 `ctx.tools.register`，不依赖 `@deepseek-ai/*` |
| `cordis` | 市场 Cordis 插件（`defineTool`、`apply`、inject） |
| `auto` | 由宿主读文件探测，按 Cordis 启发式选择 |
| `external` | 通过 `LEAF_DSH_RUNTIME` 指向自管运行时模块 |

npm 钉扎与受支持范围：`apps/plugin-host/dsh-runtime.compat.json`。`doctor` 会报告内嵌 DSH 状态；设置 `LEAF_DSH_STRICT=1` 可在版本漂移时失败。

### Claude Mod 有界契约（`hooks_command`）

契约清单：`apps/plugin-host/claude-mod.compat.json`（与 DSH compat 并列）。

| 支持 | 不支持 |
|---|---|
| `hooks.json` 仅 command 钩子，或 `modules` 仅 `.mjs`/`.js` 单文件 | `modules` 含 `.ts`/`.tsx`（官方 `register`、桌面 pane 类 UI） |

- `vendor_plugins_dir` 展开时**自动跳过** `hooks_module` 插件（日志 `skipped`），不把未支持形态送进宿主。
- 用 `path:` **显式**指向 `hooks_module` 插件时，装配合并或 `doctor` **直接失败**（错误码 `claude_mod_unsupported:hooks_modules`）。
- 任一已配置插件在宿主 `init` 中加载失败时，默认**阻止** `claude_mod` / `dsh_plugin` 启动；仅开发时可设 `LEAF_CLAUDE_MOD_STRICT=0` 降级为告警。
- `vendor_plugins_dir` 下若存在 module 插件且需强制失败（不跳过），可设 `LEAF_CLAUDE_MOD_STRICT=1`。

非目标：Claude Code 桌面 UI / pane 对等；勿指望官方 vendor 中 `code-modernization` 等 `modules` 插件在 Leaf 默认可用。

## 工具与钩子

启动时宿主合并插件列表并注册 `EcosystemDispatchAspect`。模型可见工具名：

`eco__{origin}__{instanceId}__{toolName}`

（`instanceId` 可配置，默认为插件路径哈希。）

插件逻辑在 Node 中执行。文件与进程访问经宿主出站 API（`$.fs.*`、`$.process.run` 等）映射到 Leaf 能力，而非在 Rust 侧再实现一套文件栈。出站面列表：`ability_plugin_host::mod_outbound_surfaces()`。

`claude_mod` 将会话/回合事件转发到 Mod 钩子（例如 `session.created` → `session.start`）。也可通过 `gateway.invoke(claude_mod, invoke, { tool, args })` 直接调用钩子。

## 组合根 feature

```toml
frame-compose = { features = ["ability-dispatch", "ability-claude-mod", "ability-dsh-plugin"] }
```

## 参考应用 `web-agent`

端到端验证请使用 `apps/web-agent`。`apps/agent-test` 不加载生态插件。

| 项 | 路径 |
|---|---|
| 装配 | `apps/web-agent/assembly.yaml` |
| 默认插件清单 | `apps/web-agent/dsh/web-profile.plugins.yaml` |
| 示例清单 | `apps/web-agent/dsh/official.web.plugins.example.yaml` |

```bash
cargo run -p web-agent -- serve
cargo run -p web-agent -- doctor
cargo run -p web-agent -- --plugins apps/web-agent/dsh/official.web.plugins.example.yaml serve
```

`serve` 后 Web UI：`http://127.0.0.1:8790`。

## 验证

```bash
cargo test -p plugin-compat
cargo test -p ability-plugin-host
cargo test -p web-agent --test boot --test ecosystem --test profile_merge
cargo run -p web-agent -- doctor
```

用例目录：`apps/plugin-compat/tests/fixtures`、`apps/web-agent/tests/fixtures`。

### 市面插件（可选）

拉取 vendor 后运行带 `ignore` 的集成测：

```powershell
apps/web-agent/scripts/fetch-ecosystem-vendor.ps1
```

```bash
cargo test -p web-agent --test market_plugins -- --ignored
cargo run -p web-agent -- --plugins apps/web-agent/market/market.plugins.yaml doctor
```

清单：`apps/web-agent/market/market.plugins.yaml`。

## 范围

| Leaf 侧支持 | 非目标 |
|---|---|
| Mod 加载、钩子分发、`tool.check`、出站 host API | Claude Code 桌面 UI 对等 |
| DSH thin + 内嵌 Cordis → `dispatch` | DSH Harness 全量预设、热更新 |
| `web-agent` doctor / invoke / Web SSE 观测 `ui.*` 钩子 | 应用内市场一键安装 |
