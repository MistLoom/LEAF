# Guest Ability 线协议

面向在 Node / Python / 仓颉中编写并接入 Leaf 的能力包。帧格式与版本字段的机器可读摘要随框架发布（`guest-protocol-1` 规范）。

## 版本

| 名称 | 含义 |
|------|------|
| `protocol_version` | 帧与 `CtxOp` 形状，当前 `1` |
| `framework_version` | `ability.yml` / `AbilityManifest` 语义，与框架声明版本对齐 |

二者独立演进。

## 传输

UTF-8 JSON，一行一帧，以 `\n` 结束。空行忽略。

### 传输卫生（MUST）

各语言 Guest SDK **必须**满足下列卫生要求，否则 Host 侧可能握手超时或误判 Guest 崩溃：

| 要求 | 说明 |
|------|------|
| **写后 flush** | 每向 stdout 写出一帧（含结尾换行）后，**必须**立即 flush。重定向管道下 stdout 常为全缓冲；不 flush 则 Host 读端拿不到数据。 |
| **stdin EOF 干净退出** | stdin 关闭 / 读到 EOF 时，Guest **必须**干净退出主循环（正常 return / exit 0），**禁止**未捕获异常崩进程。 |

实现对照见仓库根 README 的 Guest SDK 对等矩阵「传输 flush / stdin EOF」行。

## 握手

1. 宿主 → `{"id":"…","body":{"type":"hello"}}`
2. 客端 → `reply_to` 指向请求 id，`body.type=manifest`
3. 宿主 → `create` + `config`
4. 客端 → `ok`

之后可交错 `invoke`、`lifecycle`、`ctx`、`event`、`run_handle`；结束发 `shutdown`。

## 能力矩阵

| 能力 | 进程内 Rust | Guest（Rust / Node / Python / 仓颉） |
|------|-------------|-------------------------------|
| manifest / methods / lifecycle | 是 | 是 |
| `ctx.call` / `dep` / `catalog` | 是 | 是（`Ctx` 帧） |
| `local_get` / `local_set` | 是 | 是 |
| `emit` / `subscribe` / `handle` / `event_next` | 是 | 是 |
| `load` / `unload` | 是 | 是 |
| `feedback` | 是 | 是 |
| `chain_status` / `share` | 是 | 是 |
| `push_effect` + invoke 边界 undo | 是 | 是 |
| `register_aspect` | 是 | **否**（Guest 显式拒绝） |

## 能力包形态（与 Rust 共用 `ability.yml`）

| 运行时 | `framework_version` | 工程特征 | Guest 入口（固定） |
|--------|-------------------|----------|-------------------|
| Rust | `1` 或 `rust@1` | `Cargo.toml` | — |
| Node | `node@1`（`ts@1` 同义） | `package.json` | `src/serve.mjs` |
| Python | `python@1`（`py@1` 同义） | `pyproject.toml` | `src/serve.py` |
| 仓颉 | `cangjie@1`（`cj@1` 同义） | `cjpm.toml` | `src/serve.cj` |

未写前缀时按目录推断：有 `Cargo.toml` 为 Rust；否则存在 `src/serve.mjs` / `src/serve.py` / `src/serve.cj`（或仅 `package.json` / `pyproject.toml` / `cjpm.toml`）。冲突时须显写前缀。

入口文件通过 Leaf Guest SDK 的 `serveAbility`（Node）或 `serve_ability`（Python）或 `serveAbility`（仓颉，`leaf_ability_sdk` 包），在固定路径注册 `ability.yml` 中的方法 handler。仓内 SDK 位于仓库根 `SDK/ability-sdk-ts`、`SDK/ability-sdk-py`、`SDK/ability-sdk-cj`（Rust 线协议库为 `SDK/ability-sdk`）。

## 装载失败分类

Guest `load` 失败时，错误对调用方可见：分类码（`ErrorKind::code()`）+ 可读信息，**禁止静默成功**。宿主侧同时记录在 `AbilityHost::load_failures()`（按实例去重，成功装载即清除）。

| 分类码 | 含义 | 典型场景 |
|--------|------|----------|
| `toolchain_missing` | 缺运行时/构建工具链 | Node/Python 解释器不在 PATH；仓颉构建器不可用 |
| `artifact_missing` | 缺预构建产物 | 仓颉 Guest 仅用预构建模式下无可执行产物 |
| `handshake_failed` | 握手/协议失败 | Guest 进程早退、应答不是 Manifest、`create` 失败 |
| `failed` / `io` / 其它 | 构建失败、管道 IO 等其它错误 | 保留原始消息语义 |

## 就绪契约（装载成功 ≠ 可 invoke）

- `load` 返回成功表示：Guest 进程拉起、握手完成（`hello`→`manifest`→`create`）、生命周期回调成功，实例进入就绪阶段。
- 任何一步失败，`load` 返回错误并回滚实例；失败不产生「在册但不可用」的静默态。
- 调用方（如对外入口的注册逻辑）应通过 `AbilityHost::readiness(instance)` 探针决定是否暴露依赖该 Guest 的能力：
  - `Ready`：实例在册且就绪，可 invoke；
  - `Failed { code, message }`：最近一次装载失败及分类码；
  - `Absent`：未装载或装载已回滚。

## 长任务：handle / 轮询（协议级）

Guest 内耗时超过一次 invoke 合理时长的任务，走**协议级 handle + 轮询**契约。不新增帧类型、不新增方法号，全部落在既有 invoke 语义内：

1. **受理**：方法 handler 把一个 Promise 标记为长任务（Node SDK：`pending(promise)`），该 invoke 立即返回 `{"task_handle": "<id>"}`，任务在 Guest 内继续执行。
2. **轮询**：调用方以**同一方法**再次 invoke，参数携带 `{"handle": "<task_handle>"}`。SDK 拦截该形态的调用（不进入业务 handler），返回：
   - `{"state": "running"}`：任务未完成；
   - `{"state": "done", "data": <结果>}`：完成，handle 随之释放；
   - `{"state": "failed", "error": "<消息>"}`：失败，保留原始错误语义，handle 随之释放；
   - `{"state": "not_found"}`：handle 未知或已释放（含重复取结果）。
3. **语义约束**：handle 仅在 Guest 进程内有效（unload/shutdown 即失效）；轮询频率由调用方决定，框架不做推送。

长任务是**协议级**能力（不新增帧类型），与语言无关；各语言 SDK 落地状态见 README Guest SDK 对等矩阵「长任务 handle/轮询」行。

| 语言 | 状态 | 入口 |
|------|------|------|
| Node | ✅ | `pending` / `withLongTask`；`serveAbility` 自动包装 |
| Python | 见矩阵 | `pending` / `with_long_task`；`serve_ability` 自动包装 |
| 仓颉 | 见矩阵 | `pending` / `withLongTask`；`serveAbility` 自动包装 |
| Rust Guest（线协议库） | 不强制 | 进程内/双工可自管；不强制薄助手 |

**未在矩阵标为完整落地的语言，不得宣称支持长任务**；Ability 作者应查矩阵后再用 `pending`。参考测试：`apps/ability-core/tests/proc_guest_subprocess.rs::subprocess_probe_ts_long_task_handle_poll`（覆盖受理、running、done、failed、handle 释放）。

## 可选仅用预构建

宿主侧 `GuestSourceProvider::prebuilt_only(true)` 启用「不现场编译、只认预构建产物」的装载路径。当前作用于仓颉 Guest：产物约定为能力包内 `target/release/<cjpm 包名>[.exe]`；缺产物归 `artifact_missing` 错误类，不静默降级。默认（关闭）时缺产物会先调用构建器现场构建，构建器不可用归 `toolchain_missing`。

## 多语言对等与新增能力门禁

新增 Guest 协议能力或 SDK 表面时：

1. 先更新本协议（MUST / 能力矩阵）与仓库根 README 的 Guest SDK 对等矩阵；
2. 四语（Rust 线协议库 / Node / Python / 仓颉）**同列落地**，或在矩阵中显式标缺口，**不得**宣称已完成；
3. 补探针 / 回归（含管道握手、stdin EOF、写后 flush）；
4. **禁止**只在产品仓（如 ModalPrism）改 SDK 热修而不上提 Leaf。

跨语言实现应保持与 `SDK/README.md` 所述参考布局一致（传输 / serve / ctx 分层），便于第五种语言对照移植。
