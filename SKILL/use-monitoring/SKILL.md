---
name: use-monitoring
description: >-
  Use Leaf built-in low-level monitoring (apps/monitor): attach, CLI, TUI, HTTP, fun, JSONL,
  stats/graph, and how to build monitoring-style services on top. Use when integrating monitor,
  reading MonitorEvent/ChainStatus, or implementing dashboards/alerts/audit export.
---

# 使用底层监测

## 站立约束

- 监测的**唯一产品面**是可删的 `apps/monitor`（crate：`leaf-monitor`）。删除该 app（及入口对它的依赖）即关闭运行面。
- Cargo `monitor` feature 是实现细节：由本 app 的依赖自动启用；**不要**把它当成第二套产品级开关来记。
- 不要改禁止目录（`/frame`、`/SDK`、`/apps/ability-core`、`apps/plugin-host`）来「开监测」；挂载与查询都走 `apps/monitor`。
- 主要注意事项也可对照公开 `AGENT.md` 中的框架边界。
- 整包更新时监测面冲突：先问用户，见 `framework-update`。

## 底层监测是什么

`apps/monitor` 提供：

- 进程内挂载（`attach`）与配置（`monitor.yml`）
- 异步 JSONL 落盘（有界队列；满时丢最旧）
- 查询：CLI、实时 TUI、可选 HTTP、开放 `fun`
- 统计与图：`stats`（含丢弃计数）、`graph`（显式父子边）

内核在 invoke / lifecycle / host 路径埋点；无 Sink 或卸掉监测时，热路径不保留监测成本。

## 如何使用

### 1. 挂载

```rust
use leaf_monitor::{attach, MonitorConfig};

let app = attach(&host, MonitorConfig::load(None));
// … 使用 host …
app.shutdown(); // 卸 Sink
```

配置文件 `monitor.yml`（缺省用 `MonitorConfig::default`）常见项：`log_path`、`queue_capacity`、`recent_capacity`、`http.enabled` / `http.bind`、`tui_refresh_ms`。

### 2. CLI

进程内：`leaf_monitor::cli::run_cli`（chains / events / log / stats / graph）。

离线日志：

```bash
cargo run -p leaf-monitor -- log --file monitor.jsonl --tail 64
```

### 3. 实时 TUI

```bash
cargo run -p leaf-monitor -- tui
cargo run -p leaf-monitor -- tui --refresh-ms 200 --pump-ms 300
cargo run -p leaf-monitor -- tui --no-demo   # 无 demo 泵；独立进程下几乎无事件
```

嵌入宿主（LIVE，无 DEMO 标）：

```rust
use leaf_monitor::tui::{run_tui, run_tui_opts, TuiOpts};
run_tui(&app)?;
// 或 run_tui_opts(&app, TuiOpts { demo: false, ..Default::default() })
```

| 键 | 作用 |
|----|------|
| `q` / `Esc` | 退出 |
| `Tab` | 在 chains / events / graph 间切换焦点 |
| `g` | 切到链路图视图 |
| `↑` `↓` / `j` `k` | 滚动 |
| `PgUp` `PgDn` | 翻页 |
| `Home` / `End` | 更旧 / 回到最新（事件） |

标题栏可见 `LIVE` 或 `DEMO`、刷新间隔、距上次刷新、以及 `dropped/queued/recent`。

### 4. HTTP（默认关闭）

开启后本机绑定（默认形如 `127.0.0.1:8792`，与 mgmt 端口分离）：

| 方法 | 路径 | 含义 |
|------|------|------|
| `GET` | `/monitor/chains` | 在途链快照 |
| `POST` | `/monitor/events` | 最近事件（body 可含 `tail`） |
| `POST` | `/monitor/log` | JSONL 尾行 |
| `GET` | `/monitor/stats` | `{dropped,queued,recent}` |
| `GET` | `/monitor/graph` | 由最近事件重建的 `{nodes,edges}` |

### 5. Fun

`leaf_monitor::fun_call` 目标：`monitor.chains` / `monitor.events` / `monitor.log` / `monitor.stats` / `monitor.graph`。可从 `FunPort` 或其它 Ability 复用。未知目标返回明确错误（例如「未知监测 fun 目标」）。

### 6. JSONL

默认写入配置的 `log_path`（如 `monitor.jsonl`）：**一行一条**完整 `MonitorEvent` JSON，便于事后审计或外部管道消费。

## 事件与链字段（建模用）

### MonitorEvent（要点）

| 字段 | 用途 |
|------|------|
| `at` / `seq` | 时间戳与进程内顺序 |
| `kind` | `invoke` / `lifecycle` / `host` … |
| `phase` | 如 `before` / `after` / `error` / `load` / `unload` / 生命周期相位 |
| `instance` / `method` / `ability_id` | 实例、方法、能力 id |
| `chain_id` | 一次根调用链 |
| `span` | 嵌套深度（冗余；拓扑以 id 为准） |
| `span_id` / `parent_span_id` | **显式**调用树边；无 parent 则不要猜边 |
| `started_at` / `ended_at` | 段起止，可算耗时 |
| `from` / `to` | 生命周期/host **状态转移** |
| `error` / `detail` | 错误与补充 |

### ChainStatus（在途快照）

含 `chain_id`、`instance`、`method`、`span`、`running`，以及 `started_at`、`span_id`、`parent_span_id`、`ability_id` 等。快照主要反映当前在途；**历史拓扑**靠事件流 / `graph` 重建。

### Stats

`{ "dropped": N, "queued": N, "recent": N }` —— 队列丢最旧时 `dropped` 增加。做完整审计前先看 `dropped`。

## 如何借底层监测做监测类服务

面向集成方/产品方的组合方式（在 `apps/monitor` 之上，而不是改内核）：

1. **订阅 / 消费事件流**
   - 进程内：挂载后的 Sink / 最近事件缓冲 / `fun_call`
   - 文件：尾随 JSONL
   - 网络：开启 HTTP 后由自有服务拉取
   - 用途：自有面板、告警、审计导出、对接外部 APM

2. **链路图**
   - 使用 `leaf_monitor::graph::build_chain_graph(&events)`，或 HTTP/fun 的 `graph`
   - **只**根据 `span_id` → `parent_span_id` 建边；没有 parent 就不要发明边

3. **状态图**
   - 用 `kind` + `from`/`to`（及 `phase`）还原实例生命周期与 host load/unload
   - 单次 invoke：`before` → `after` | `error`，耗时用 `started_at`/`ended_at`

4. **完整性**
   - 任何「全量」统计或告警都先读 `stats.dropped`；`dropped > 0` 时结果只能标为不完整或降级

5. **组合根挂载**
   - 入口应用依赖 `leaf-monitor`，在 Host 就绪后 `attach`，进程退出前 `shutdown`
   - 需要 LIVE TUI 时在同一进程 `run_tui`；独立 `monitor tui` 默认 DEMO 泵仅便于观看

6. **Guest / external 边界**
   - 监测是框架产品面，不是业务 Ability；不要把监测伪装成普通业务能力，也不要在 `external/` 演示里伪造框架内置身份（见 `ability-placement`、`guest-ability`）。

## 反模式

- 把 Cargo feature 当成第二套产品开关，或要求使用者同时记「feature + app」
- 静默丢事件却仍宣称统计/审计完整（忽略 `stats.dropped`）
- 没有 `parent_span_id` 时用深度或时间顺序瞎猜父子边
- 为「开监测」去改 `/frame` 或 `ability-core`
- 整包更新时静默覆盖本侧对 `apps/monitor` 或监测契约的改动（应先问用户，见 `framework-update`）

## 自检

- [ ] 入口只通过 `apps/monitor` 挂载与查询；删除 app 后运行面关闭
- [ ] 需要图时使用 `span_id`/`parent_span_id` 或 `build_chain_graph`，不猜边
- [ ] 完整审计前检查 `monitor.stats.dropped`
- [ ] 未把监测标成业务 builtin；未改四个禁止目录
