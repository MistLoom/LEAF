# 测试结果说明

复现脚本：`run_access_test.ps1`（含编译、真实配置 CLI、tool-bin build、第三方 showcase）。

当前能力开放与独立 CLI 用法见：

- `../../docs/capability-expose.md`
- `../../README.md`

```bash
# 仓库根目录
powershell -File external/showcase/run_access_test.ps1

# 或手动
cargo run -p apps-runtime --bin tool-bin -- build
cargo run -p apps-runtime -- --config config/tool_expose.example.yaml catalog list
cd external/showcase && cargo run
```
