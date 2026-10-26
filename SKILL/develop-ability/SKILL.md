---
name: develop-ability
description: >-
  Create or change a Leaf Ability with correct ability.yml, package layout,
  handlers, and protocol projections. Use when adding or editing an Ability
  in abilitys/, apps/*/ability/, or when the user asks how to write a standard Ability.
---

# 编写规范 Ability

## 站立约束

- Ability 是应用层能力：跨应用调度 / 协同 / 开放，或应用内统一链路、追踪、日志、子系统调度；也可支撑第三方扩展。
- 组合能力优先用多个 Ability，必要时建子应用开放；不要改框架内核。
- **禁止更改**：`/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`。
- Ability 与 frame 不读场景配置；由 compose 加载后注入。
- 内部完备测例不进公开 git。

落点：`ability-placement`。Guest：`guest-ability`。可见性：`expose-governance`。

## 步骤

### 1. 包布局（Rust 进程内）

```
ability.yml
build.rs
Cargo.toml
src/logic/
src/generated/
cli|mcp|tools|grpc|skill/info.yml   # 按需
```

### 2. `ability.yml`

| 字段 | 含义 |
|------|------|
| `framework_version` | 进程内 `"1"`（或与框架一致的 `rust@1`） |
| `id` | 稳定 id，如 `demo.counter` |
| `protocols` | 如 `[tools, mcp, cli, http, fun]` |
| `depends` / `stateful` / `open` | 依赖、会话态；`open` 默认 `false` |
| `methods[]` | `id`、`description`、JSON Schema 风格 `input` |

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

### 3. Handler

- 每 method → `src/logic/<method>.rs`，导出生成 provider。
- 不要按 MCP / CLI / LLM 来源分支。
- `factory: true` 时实现自定义 Ability + into_factory。

### 4. 协议投影

| 目标 | 做法 |
|------|------|
| tools | `protocols` 含 `tools` |
| MCP / CLI | 对应目录 + `info.yml`，版本对齐 |
| HTTP / Fun | `protocols` 含 `http` / `fun` |

### 5. 跨能力

只依赖对方 client 契约，不依赖对方内部模块。

### 6. 自检

- [ ] 未改四个禁止目录；声明与实现一致
- [ ] 无 Port 分支；不读场景配置；公开树无内部完备测例

## 反模式

- 改 `/frame` 或 `ability-core` 修业务
- 用 `open: true` 代替 Expose 解决 Port 可见性
