---
name: framework-update
description: >-
  Upgrade Leaf framework cores or sync them into a downstream project.
  Use when updating frame/, SDK/, apps/ability-core, or apps/plugin-host, or when the user asks how to pull framework updates.
---

# 框架更新

## 站位约定

- **禁止改动**：`/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`
- 拉取更新时**完整覆写**上述目录

## 核心目录

| 路径 | 职责 |
|------|------|
| `/frame` | support、core、compose、port-*、protocol |
| `/SDK` | Guest SDK |
| `/apps/ability-core` | 能力宿主 |
| `/apps/plugin-host` | 插件宿主 |
| `/apps/monitor` | 可删除产品面（可选：可保留本地删改） |

## 如何判断是否框架自带

更新/合并/冲突处理时按下列标记**识别**（只读判定，勿教业务作者填写）：

| 对象 | 判定 |
|------|------|
| Ability | 其 `ability.yml` 存在 `leaf_builtin: true` |
| App | 该 app 根目录 `LICENSE` 含约定行 `LEAF-BUILTIN-APPS` |

- 不要另造第二套标记文件或用 Cargo feature 冒充产品开关。
- 对外开发辅助 skill（如 `develop-ability`）不应指导业务作者写入上述标记；本 skill 仅供框架更新与维护识别。
- 落点边界见 `ability-placement`；Guest/`external` 见 `guest-ability`。

## 监测与冲突

- 监测**唯一产品面**是可删除 `apps/monitor`。如何使用、如何借其做监测类服务：见 **`use-monitoring`**。
- 拉取更新时若监测面（含 `apps/monitor` 或其相关契约）与本地改动**冲突**，**先询问用户**再取舍（保留本地 / 采用框架侧 / 手工合并）；禁止静默覆盖或默默丢弃。
- 嵌套调用契约：同链加深 `span`，**仅在 `span==0` 返回时**拆链（细节见 `use-monitoring`）。

## 冲突处理（总则）

完整覆写 `/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`（以及若附带 `apps/monitor` 等）时，若与本地改动冲突，**先询问用户**再取舍，禁止静默覆盖或默默丢弃。

## 维护方

1. 先定宿主契约 / Gateway、Port / 协议 / Compose / SDK。
2. 计划对比下游 `framework_version` / Guest 前缀，写迁移说明。
3. 业务只落 `abilitys/`、`apps/<产品>/`、`external/`。
4. 在 workspace 做 Guest 回归与升级。

## 步骤

1. 核对版本差异与破坏性为 0。
2. 完整替换四个核心目录（若 app 同步则同步监测）。
3. 勿改本地 `ability.yml` / handler；勿动业务内核。

## 反模式

- 下游 fork `/frame`
- 业务时代码进 `ability-core`
