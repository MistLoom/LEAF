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

冲突时显写 `framework_version`。语义优先对齐 `SDK/ability-sdk` Guest；实现可用 `ability-sdk-ts` / `-py` / `-cj`。跨语言布局见 `SDK/README.md`。

## 能力矩阵

支持：manifest/methods/lifecycle；ctx.call/dep/catalog；local_get/set；emit/subscribe/handle/event_next；load/unload；feedback；chain_status/share；push_effect+undo。  
拒绝：`register_aspect`。

帧输出、长任务等能力差异以仓库根 README「Guest SDK 能力」表为准；协议细节见 `manual/ability-guest-protocol.zh-CN.md`。

## 步骤

1. `ability.yml` 用上表 version；methods 与进程内同模型。  
2. Node：`serveAbility({...})`；Python：`serve_ability(...)`；仓颉：`serveAbility(handlers)`。  
3. Handler 不按 Port 分支。  
4. 内部测本地跑通且保持 gitignore。

## 新增 Guest 能力门禁

1. **先**更新协议 MUST / 能力说明与 README 对等矩阵；  
2. 四语同列落地，或在矩阵**显式标缺口**（不得宣称完成）；  
3. 补探针 / 回归（含管道握手、写后 flush、stdin EOF）；  
4. **禁止**只改产品仓 SDK 热修而不上提 Leaf。

## 自检

- [ ] version 与入口匹配；method 与 handler 一致  
- [ ] 未改 `/SDK` 迁就本包；公开侧无内部测例  
- [ ] 若动到协议能力：README 对等矩阵与协议 MUST 已同步  
- [ ] 传输：stdout 写后 flush；stdin EOF 干净退出（本语言 SDK 已具备，勿绕开）  
- [ ] 若使用长任务：本语言在矩阵中为 ✅（或接受 ⚠️ 限制）
