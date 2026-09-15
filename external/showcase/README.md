# Showcase（外部项目）

本目录是**独立于 leafEX workspace 的外部工程**，模拟第三方如何接入/访问已开放能力。

能力开放表来自框架示例（外部不自持开放配置）：

- `../../config/tool_expose.example.yaml`
- 说明：`../../docs/capability-expose.md`

```bash
cd external/showcase
cargo run
```

框架侧独立 CLI（与本示例无关，供对照）：

```bash
# 在仓库根目录
cargo run -p apps-runtime --bin tool-bin -- build
cargo run -p apps-runtime -- --config config/tool_expose.example.yaml invoke read_file --args "{\"path\":\"README.md\"}"
```
