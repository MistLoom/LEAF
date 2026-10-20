# 架构概览

[English](architecture.md) · 中文

## 模型

**应用**拥有自己的 Ability，路径为 `apps/<app>/ability/<name>`。外部门户与跨能力调用共用同一条 **Gateway** 信封：`instance + method + args`。

```
Ports (CLI / HTTP / MCP / Fun / …)
        │
     Gateway
        │
   入口应用
        ├─ 本应用 Ability
        └─ 其他应用的 open Ability
```

`ability.yml` 的 `open` 默认为 false。其他应用只能调用 `open: true` 的 Ability。

事件的监听和拦截由入口应用的 `AppControl` 登记。拦截只作用于本应用的 Ability。监听还可以收到其他应用已开放 Ability 的事件。过滤由引擎实现。

## 分层

| 层 | 路径 | 职责 |
|---|---|---|
| 契约 | `frame/support` | Manifest、Ctx、生命周期、`#[client]` |
| 传输 | `frame/core`、`frame/port-*` | Gateway、Port、Expose、控制器 |
| 引擎 | `apps/ability-core` | Host、EventBus、调用与事件准入 |
| 组合 | `frame/compose` | Catalog、Bootstrap、`AppControl` |
| 能力 | `common/ability/*`、`apps/<app>/ability/*` | 通用能力，或归属某个场景应用 |
| 应用 | `apps/*` | 场景进程（`agent-test`、`web-agent`） |
| 外部 SDK | `external/*` | 实验性的仓外编写面 |

## 归属

| 归属 | Ability |
|---|---|
| `common` | 通用能力（配置、存储、文件、shell、任务、知识库及参考示例），以及可选的 `claude_mod`、`dsh_plugin`。`open`。入口应用可以调用、监听并拦截 |
| `agent-test` | Agent 回路（`session`、`turn`、`dispatch`、`chat`、`prompt`、`skill`），`open` |
| `web-agent` | Web 参考应用。兼容夹具在其测试中 |

## 约束

1. Port 只依赖 `Gateway`，不直接调用 Ability crate
2. Ability 不按「来自 MCP / CLI / LLM tools」分支
3. Expose 只裁 **Port → Gateway**；它不授予跨应用调用
4. 跨应用调用要求目标 `open: true`
5. 拦截属于属主应用。`common` 能力可由入口应用拦截。监听其他场景应用的 Ability 要求目标 `open: true`
6. 协议身份属于声明与 Port，而不是内核业务类型
7. `external` 等仓外包不得依赖 `frame-core`、port 或 `ability-core`；消费运行时只依赖 `frame-compose`（由其再导出所需类型）

## 状态

- `external/` 为实验性
- 计划支持多语言 Ability（Python、TypeScript 等）；当前以 Rust 为主
- `apps/agent-test` 为简易 Agent 示例
- `apps/web-agent` 为 Web 参考应用
