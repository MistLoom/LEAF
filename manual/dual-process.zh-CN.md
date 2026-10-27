# 双进程参考示例（无业务语义）

位置：`examples/dual-process/`。演示 Leaf 推荐的跨进程拼法：**只读 Guest 子进程 + 宿主侧写子能力**，经统一 Gateway 调用。能力均为假能力，不含任何业务语义。

## 形态

```
dual-process-host（Rust 宿主）
├─ notebook   只读 Guest（Node 子进程，guest/，stdio NDJSON guest-protocol-1）
│    ├─ notebook.read   返回固定备忘
│    └─ notebook.ping   探活，发 read_done 事件
└─ scratch    写子假能力（进程内 Rust，内存计数）
     └─ scratch.append  追加一条，返回条数
```

## 运行

```bash
cargo run -p example-dual-process   # 需要 Node.js
```

预期输出：

```
guest read → {"memo":"sandbox-note-1"}
host append → {"count":1}
可选 HTTP 面：HttpPort::new(127.0.0.1:xxxxx) 后 serve 即可投影两侧能力
```

## 推荐拼法要点

1. **Guest 只做一侧职责**：示例中 Guest 是只读面（`read` / `ping`），写侧留在宿主进程内。多进程是隔离与资源边界，不是默认分布；按读写或信任边界拆进程，而不是按业务模块拆。
2. **统一入口**：宿主对两侧的调用都走 `AbilityHost`（即 `Gateway` 实现）——同一套 invoke 信封、错误分类与观测，不因进程边界分叉。
3. **声明一致**：Guest 用 `ability.yml` 声明（`id: notebook`、`framework_version: node@1`），与进程内能力的 Manifest 模型一致；`open: true` 表示允许其他链路调用。
4. **入口约定**：Guest 工程目录含 `ability.yml` + `src/serve.mjs`（入口由 `framework_version` 推导）；宿主经 `GuestSourceProvider::from_workspace_rel` 指向该目录即可，无需手写 spawn。
5. **投影可选**：两侧能力可经同一个 `HttpPort`（或 MCP/gRPC 端口）投影出去；对外只见统一 REST 面，进程拓扑不外泄。

相关：[Guest 线协议](ability-guest-protocol.zh-CN.md)、[清单与裁剪](catalog-and-expose.zh-CN.md)。
