---
name: framework-update
description: >-
  Upgrade Leaf framework cores or sync them into a downstream project.
  Use when updating frame/, SDK/, apps/ability-core, or apps/plugin-host, or when the user asks how to pull framework updates.
---

# 框架更新

## 站立约束

- **禁止更改**：`/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`  
- 下游升级：**整包拉取**上述目录  

## 更新面

| 路径 | 职责 |
|------|------|
| `/frame` | support、core、compose、port-*、protocol |
| `/SDK` | Guest SDK |
| `/apps/ability-core` | 引擎宿主 |
| `/apps/plugin-host` | 插件宿主 |

## 维护者

1. 变更落在契约 / Gateway·Port / 引擎 / Compose / SDK。  
2. 破坏性变更升 `framework_version` / Guest 前缀并写迁移说明。  
3. 业务只放 `abilitys/`、`apps/<产品>/`、`external/`。  
4. 跑 workspace 与 Guest 合规样例。  

## 下游

1. 更新面本地补丁尽量为 0。  
2. 整包替换四个目录。  
3. 改编本侧 `ability.yml` / handler，不改回内核。  

## 反模式

- 长期 fork `/frame`  
- 热修时改 `ability-core`
