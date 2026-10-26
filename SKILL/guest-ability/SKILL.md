---
name: guest-ability
description: >-
  Implement a Leaf Guest Ability in Node, Python, or Cangjie (guest-protocol-1).
  Use when writing serve.mjs / serve.py / serve.cj, framework_version node@1|python@1|cangjie@1, or external Guest packages.
---

# Guest Ability

## 站立约束

- 落在 `external/<name>` 或允许的 Ability 落点；不要为单个 Ability 改 `/SDK` 契约。
- 内部完备测例不进公开 git。
- Guest 拒绝 `register_aspect`。
- Guest/`external` 包**不是**框架内置能力，**不要**伪装成框架内置身份。

## 运行时

| 语言 | `framework_version` | 标记 | 入口 |
|------|---------------------|------|------|
| Node | `node@1`（`ts@1`） | `package.json` | `src/serve.mjs` |
| Python | `python@1`（`py@1`） | `pyproject.toml` | `src/serve.py` |
| 仓颉 | `cangjie@1`（`cj@1`） | `cjpm.toml` | `src/serve.cj` |

冲突时显写 `framework_version`。语义优先对齐 `SDK/ability-sdk` Guest；实现可用 `ability-sdk-ts` / `-py` / `-cj`。

## 能力矩阵

支持：manifest/methods/lifecycle；ctx.call/dep/catalog；local_get/set；emit/subscribe/handle/event_next；load/unload；feedback；chain_status/share；push_effect+undo。  
拒绝：`register_aspect`。

## 步骤

1. `ability.yml` 用上表 version；methods 与进程内同模型。  
2. Node：`serveAbility({...})`；Python：`serve_ability(...)`；仓颉：`serveAbility(handlers)`。  
3. Handler 不按 Port 分支。  
4. 内部测本地跑通且保持 gitignore。

## 自检

- [ ] version 与入口匹配；method 与 handler 一致  
- [ ] 未改 `/SDK` 迁就本包；公开侧无内部测例
