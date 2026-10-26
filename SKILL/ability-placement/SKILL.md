---
name: ability-placement
description: >-
  Choose where a Leaf Ability belongs (abilitys/, apps/*/ability/, or external/).
  Use when creating an Ability or deciding shared vs app-owned vs Guest demo placement.
---

# Ability 落点选择

## 站立约束

- `apps`：除 `ability` 子树外不限制语言与架构；框架根保持整洁。
- 禁止把业务 Ability 写入 `/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`。
- 跨应用组合可建子应用开放 Ability。

## 决策

| 落点 | 何时 |
|------|------|
| `abilitys/<分类>/<name>` | 跨应用共享 |
| `apps/<app>/ability/<name>` | 应用自有；`open` 控制外借 |
| `external/<name>` | 示例、夹具、Guest 演示 |
| `SDK/ability-sdk-*` | 不是 Ability（Guest SDK / 更新面） |

1. 多 App 复用？→ `abilitys/...`  
2. 否则某 App 私有？→ `apps/<app>/ability/...`  
3. 否则示例/夹具？→ `external/...`  
4. Rust 进程内 vs Guest → `guest-ability`

实现步骤见 `develop-ability`。
