# leaf-monitor（`apps/monitor`）

Leaf 唯一可删的底层监测产品面：全局配置、异步 JSONL、查询 CLI、**实时 TUI**、可选 HTTP、开放 fun。

开关就是本 app 是否存在 / 被入口依赖；删除 crate 即关闭运行面。内核探针由本 crate 依赖自动启用 Cargo `monitor` feature。

## 实时 TUI

```bash
# 独立打开（默认 DEMO：事件泵 + 慢 echo 调用，标题栏标 DEMO）
cargo run -p leaf-monitor -- tui

# 更快刷新
cargo run -p leaf-monitor -- tui --refresh-ms 200 --pump-ms 300

# 关闭 demo 泵（独立进程下几乎无事件）
cargo run -p leaf-monitor -- tui --no-demo

# 冒烟例
cargo run -p leaf-monitor --example tui_smoke
```

嵌入宿主（真实 LIVE，无 DEMO 标）：

```rust
use leaf_monitor::{attach, MonitorConfig};
use leaf_monitor::tui::run_tui;

let app = attach(&host, MonitorConfig::load(None));
run_tui(&app)?; // 或 run_tui_opts(&app, TuiOpts { demo: false, .. })
```

### 键位

| 键 | 作用 |
|----|------|
| `q` / `Esc` | 退出 |
| `Tab` | 切换 chains / events 面板焦点 |
| `↑` `↓` / `j` `k` | 滚动当前面板 |
| `PgUp` `PgDn` | 翻页 |
| `Home` `End` | 事件：更旧 / 回到最新；链：顶 / 底 |

### 能看到什么

- **标题栏**：`LIVE` 或 `DEMO`、刷新间隔、距上次刷新年龄、当前焦点
- **chains**：`ChainStatus` 字段 — `chain_id`、`instance`、`method`、`span`、`running`（无开始时间字段，不编造 duration）
- **events**：`MonitorEvent` — 时钟时间、年龄、`kind`（invoke/lifecycle/host）、`phase`（before/after/error/load/unload…）、instance、method、chain_id、span、error

## 其它入口

| 入口 | 说明 |
|------|------|
| `monitor log --file monitor.jsonl --tail 64` | 离线 JSONL |
| `leaf_monitor::cli::run_cli` | 进程内 chains / events / log |
| HTTP（默认关） | `GET /monitor/chains`，`POST /monitor/events`，`POST /monitor/log` |
| `fun_call` | `monitor.chains` / `monitor.events` / `monitor.log` |

配置见 `monitor.yml`（缺省用 `MonitorConfig::default`）：`log_path`、`queue_capacity`、`recent_capacity`、`http.enabled`/`bind`、`tui_refresh_ms`。


## Modeling fields (chain / state / stats)

Events and chain snapshots now carry topology and timing fields so clients can build graphs without guessing:

| Field | Where | Purpose |
|-------|--------|---------|
| `span_id` / `parent_span_id` | MonitorEvent, ChainStatus | Explicit call-tree edges (no parent ⇒ no invented edge) |
| `ability_id` | Event / ChainStatus | Ability id vs instance name |
| `started_at` / `ended_at` | Event (invoke) | Duration / span lifetime |
| `from` / `to` | lifecycle / host | State-transition edges |
| `seq` | Event | Process-local order |
| `dropped` / `queued` / `recent` | `monitor.stats` | Drop-aware queue stats |

Query: CLI `Stats`/`Graph`, HTTP `GET /monitor/stats` + `GET /monitor/graph`, fun `monitor.stats` / `monitor.graph`.
Rebuild: `leaf_monitor::graph::build_chain_graph(&events)`.
TUI: press `g` for graph tree (also `Tab` cycles chains → events → graph). Header shows `dropped/queued/recent`.
