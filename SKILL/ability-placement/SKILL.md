---
name: ability-placement
description: >-
  Choose where a Leaf Ability belongs (abilitys/, apps/*/ability/, or external/).
  Use when creating an Ability or deciding shared vs app-owned vs Guest demo placement.
---

# Ability 落点选择

## 站位约定

- `apps` 内的 `ability` 与外部不同源：随仓架构、框架更新策略不同。
- 禁止把业务 Ability 写入 `/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`。
- 跨应用调用可靠开放应用内 Ability。

## 落点

| 落点 | 何时 |
|------|------|
| `abilitys/<域>/<name>` | 跨应用共享 |
| `apps/<app>/ability/<name>` | 应用私有（`open` 可跨应用） |
| `external/<name>` | 示例、教具、Guest 演示 |
| `SDK/ability-sdk-*` | 框架 Ability（Guest SDK / 协议面） |

1. 多 App 共用 → `abilitys/...`  
2. 只属某 App 私有 → `apps/<app>/ability/...`  
3. 仅作示例/教具 → `external/...`  
4. Rust 原生 vs Guest 见 `guest-ability`

实现步骤见 `develop-ability`。

## 框架自带身份（识别，非填写）

落点/区分随仓交付与 `external` 时，用下列标记**识别**（只读判定）：

| 对象 | 判定 |
|------|------|
| Ability | 其 `ability.yml` 存在 `leaf_builtin: true` |
| App | 该 app 根目录 `LICENSE` 含约定行 `LEAF-BUILTIN-APPS` |

- **填写**上述标记不是业务作者职责；识别与冲突处理见 `framework-update`。
- `external/`、模板脚手架、用户新建包**不要**伪装成框架自带内容。

监测相关的更新冲突时先问用户，见 `framework-update`。
