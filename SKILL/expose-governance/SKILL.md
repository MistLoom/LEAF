---
name: expose-governance
description: >-
  Configure AbilityExpose, open, cross-app calls, and AppControl listen/intercept
  in Leaf. Use when wiring Port visibility, cross-application invoke, or event governance.
---

# 裁剪、开放与治理

## 站立约束

- Expose **只裁** Port → Gateway，不授予跨应用调用权。
- 跨应用调用要求目标 `open: true`。
- 同应用 `ctx.call` 不受 Expose 限制。
- 拦截仅属主应用；监听可含本应用、共享能力、以及已 `open` 的他应用事件。
- Port 只依赖 Gateway；Ability 不按来源 Port 分支。
- 不要改 `/frame` / `ability-core`「开权限」。

## 模型

```
Ports → Gateway (instance + method + args) → 入口应用
         ├─ 本应用 Ability
         └─ 其他应用 open Ability
```

## 步骤

1. 仅本应用 → `open: false`；需他应用调用/监听 → `open: true`。  
2. 用 AbilityExpose 裁 Port 表面。  
3. AppControl：`listen` / `intercept`（拦截不越权）。  

## 反模式

- 为「上 HTTP」设 `open: true`（应配 Expose）  
- 在 Ability 内替换 Gateway 治理

框架内置身份与 Expose 无关；落点边界见 `ability-placement`，维护约定见 `framework-update`。
