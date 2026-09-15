# 架构概览

[English](architecture.md) · 中文

## 模型

**Ability** 是能力单元。外部门户与跨能力调用共用同一条 **Gateway** 信封：`instance + method + args`。

```
Ports (CLI / HTTP / MCP / Fun / …)
        │
     Gateway
        │
  Ability 实例
        │
  AbilityCtx::call   （内部组合；不受 Expose 限制）
```

## 分层

| 层 | 路径 | 职责 |
|---|---|---|
| 契约 | `frame/support` | Manifest、Ctx、生命周期、`#[client]` |
| 传输 | `frame/core`、`frame/port-*` | Gateway、Port、Expose、控制器 |
| 引擎 | `apps/ability-core` | Host、EventBus、Chain |
| 组合 | `frame/compose` | Catalog、Bootstrap、ToolHost |
| 能力 | `ability/*` | 一 crate 一 Ability |
| 应用 | `apps/*` | 场景装配（`runtime`、`agent-test` 等） |
| 外部 SDK | `external/*` | 实验性的仓外编写面 |

## 约束

1. Port 只依赖 `Gateway`，不直接调用 Ability crate
2. Ability 不按「来自 MCP / CLI / LLM tools」分支
3. Expose 只裁 **Port → Gateway**；`ctx.call` 不受 Expose 限制
4. 协议身份属于声明与 Port，而不是内核业务类型
5. 外部包不得依赖 `frame-core`、port 或 `ability-core`

## 状态

- `external/` 为实验性
- 计划支持多语言 Ability（Python、TypeScript 等）；当前以 Rust 为主
- `apps/agent-test` 为简易 Agent 示例
