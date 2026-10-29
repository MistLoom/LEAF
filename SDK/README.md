# Ability Guest SDK — 参考布局

各语言 Guest SDK（`ability-sdk` / `-ts` / `-py` / `-cj`）都讲同一套 `guest-protocol-1` stdio JSON 行帧。
**新增第五种语言时，请按下列分层对照移植**，命名与职责尽量对齐，使各语言实现保持一致的帧输出、长任务与退出行为。

## 分层（谁管什么）

| 层 | 职责 | Rust | Node | Python | 仓颉 |
|----|------|------|------|--------|------|
| **protocol** | 帧编解码、`frame_reply` / 错误体 | `protocol`（crate 内） | `lib/protocol.js` | `protocol.py` | `protocol.cj` |
| **transport / bridge** | stdio 读写、**写后 flush**、stdin EOF→退出、`send` / `request` / `pump_line` | `guest.rs` `serve` / `StdoutSink` | `guest.js` `createBridge` + `serveStdio` | `guest.py` `_SyncBridge` + `serve_stdio` | `guest.cj` `Bridge` + `serveWithIO`；`ability.cj` `stdioWrite`/`stdioInput` |
| **ctx** | `GuestCtx` 薄转发 Host `CtxOp` | `guest.rs` ctx | `guest.js` ctx 对象 | `guest.py` ctx | `guest.cj` `GuestCtx` |
| **spec / ability** | `AbilitySpec`、`ability.yml` 装载、`serveAbility` | （作者侧 in-process） | `ability.js` | `ability.py` | `ability.cj` + `spec.cj` |
| **长任务** | `pending` + `{handle}` 轮询（协议级） | 不强制 | `pending` / `withLongTask` | `pending` / `with_long_task` | `pending` / `withLongTask` |

## 推荐主循环形状

1. `resolve_ability_root`（显式 root → `LEAF_ABILITY_ROOT` → 上溯找 `ability.yml`）  
2. `load_ability_yaml` → `build_provider`（methods 用长任务包装）  
3. `serve_*(provider)`：读行 → 忽略空行 → `dispatch` 应答 / 侧路事件 → 处理 `hello`/`create`/`invoke`/`lifecycle`/`shutdown`  
4. **每帧** `write(line)` 后 **flush**；`readline` 得 EOF / 异常 → **干净 return**

## 能力对照

各语言 SDK 的能力差异见仓库根 README 的「Guest SDK 能力」表；协议约定见 `manual/ability-guest-protocol.zh-CN.md`。

## 约定

- SDK 统一在本目录维护与发布，避免只改某一语言版本造成各实现行为不一致。
- 某语言新增支持长任务或某 CtxOp 后，请同步更新根 README 的能力表，保持说明与实现一致。  
