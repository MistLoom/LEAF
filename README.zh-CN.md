# LEAF

**LEAF 是以 Rust 构建、为 AI 生态打造的轻量化 AI 开发框架**，一套 **Ability** 同时服务 Agent、MCP、CLI、Skill、Tools 与 HTTP 等协议。

帮助团队与个人快速构建**安全、可靠、高性能**的 harness、个人助手、企业内部 AI、Tools 网关等系统与应用。

**多协议 AI 能力层**支撑同一份能力被统一组织、装配、暴露和治理，借助 Ability 实现**跨应用、跨系统调度**。

对于多 Ability 间通信，支持**同一片内存空间**、**多进程**、**跨设备网络协议**等不同模型，可按性能、可用性等要求进行定制。

> **实验性特性**：支持 DeepSeek harness 插件、Claude Mod 等生态；以 Ability 在系统边缘实现兼容，并提供参考示例。详见 [生态插件](manual/ecosystem-plugins.zh-CN.md)。

[传统 AI 系统问题](#传统-ai-系统问题) · [核心结构](#核心结构app--ability--gateway) · [运行时形态](#运行时形态的理由) · [关键特性](#关键特性) · [手册](#手册) · [项目状态](#项目状态) · [贡献](#贡献)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-early%20preview-yellow.svg)](#项目状态)

English: [README.md](README.md)

---

## 传统 AI 系统问题

目前一些 AI 应用系统区别于传统专有系统设计，内部往往存在不同子场景划分。除了可能需要对接旧系统外，不同子应用间还可能存在联动、协同工作的需要，例如：

- 对话侧触发检索与工单
- 在同一业务流中串联文档理解与命令执行
- 在采集完数据后，由其他系统进行分析或文章撰写。

在 AI 系统里，一个「识别图片」或「执行命令」的能力，经常需要同时出现在多个地方：

| 出口      | 形态           |
| ------- | ------------ |
| Agent   | `tool_calls` |
| MCP     | tools        |
| CLI     | 命令           |
| HTTP    | API          |
| Harness | 内部调度入口       |

部分能力还需以 **Skill** 等方式暴露给统一的 Agent 入口。若为其各写一套适配代码，会带来几个工程问题：

- 同一业务逻辑**重复实现**
- 不同协议下**行为不一致**
- **鉴权、限流、观测**分散在各处
- 裁剪最小运行时需要**改业务代码**
- 新增协议出口要**动很多地方**

LEAF 的设计出发点：**能力实现保持协议无关，协议映射推迟到装配阶段完成**。

---

## 核心结构：App + Ability + Gateway

| 概念          | 职责     | 关键点                                             |
| ----------- | ------ | ----------------------------------------------- |
| **Ability** | 实现具体能力 | 协议无关，只关心「做什么」                                   |
| **App**     | 装配和治理  | 决定加载哪些 Ability、绑定什么配置、暴露哪些协议端口                  |
| **Gateway** | 统一调用管道 | 所有调用走 `instance + method + args`，形成单一观测、鉴权、限流入口 |

> **Ability** 是稳定的原子，**App** 是多变的装配策略，**Gateway** 是统一的执行入口。

## 运行时形态的理由

LEAF 要保证能力按统一方式被调用、暴露和治理。这需要运行时控制权：

| 诉求       | 说明                                                                |
| -------- | ----------------------------------------------------------------- |
| **统一入口** | 库形态下调用方可绕过 Gateway 直接调业务函数，统一观测和治理会失效                             |
| **协议投影** | 同时作为 MCP server、CLI、HTTP API 暴露，涉及注册表、路由、配置加载和进程生命周期              |
| **装配裁剪** | `AbilityExpose` 等配置决定加载、禁用与参数绑定，属于应用装配阶段                          |
| **声明驱动** | `ability.yml`、协议 `info.yml`、生成 Provider、`src/logic/` 等业务布局属于框架式约定 |

App 持有装配与治理，Ability 保持纯粹与可复用。同一套 Ability 仓库可装配出不同形态的运行时：

- `AbilityExpose` 控制 Port 可见性
- 同应用内 `ctx.call` 不受限制
- 跨应用调用要求目标 `open: true`

详见 [清单与裁剪](manual/catalog-and-expose.zh-CN.md)。

LEAF 的范围收敛在**能力层运行时**，应用主循环由使用方管理。内部和边缘提供库 / SDK 形态：

- `Ability SDK`
- `Gateway client`
- 协议适配器
- 配置解析器
- 代码生成器

Ability Guest SDK 各语言实现（位于 `SDK/`，均走 `guest-protocol-1` stdio JSON 行帧）：

| 语言 SDK | 目录 | `framework_version` | 工程特征 | Guest 入口 | manifest/methods/lifecycle | ctx.call/dep/catalog | local_get/set | emit/subscribe/handle/event_next | load/unload | feedback | chain_status/share | push_effect+undo | register_aspect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Rust（线协议库） | `SDK/ability-sdk` | `rust@1` | `Cargo.toml` | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅（进程内） |
| Node | `SDK/ability-sdk-ts` | `node@1`（`ts@1` 同义） | `package.json` | `src/serve.mjs` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌（Guest 拒绝） |
| Python | `SDK/ability-sdk-py` | `python@1`（`py@1` 同义） | `pyproject.toml` | `src/serve.py` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌（Guest 拒绝） |
| 仓颉 | `SDK/ability-sdk-cj` | `cangjie@1`（`cj@1` 同义） | `cjpm.toml` | `src/serve.cj` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌（Guest 拒绝） |

Guest 与进程内 Rust 的差异：Guest 不支持 `register_aspect`（进程内切面仅 Rust 运行时可用）。协议细节见 [Guest 线协议](manual/ability-guest-protocol.zh-CN.md)。

既保证统一治理，也允许局部复用。架构细节见 [架构概览](manual/architecture.zh-CN.md)。

---

## 关键特性

| 特性                  | 说明                                                                    |
| ------------------- | --------------------------------------------------------------------- |
| **一次实现，多协议投影**      | 同一份 Ability 可装配为 MCP、CLI、HTTP 或 Agent 工具调用，业务逻辑集中在能力层                 |
| **统一 Gateway 调用管道** | 调用格式统一为 `instance + method + args`，单一观测、鉴权与限流入口                       |
| **装配期裁剪**           | `AbilityExpose` 以实例为单位控制加载、禁用与参数绑定，同一套代码可装配不同功能组合与最小运行时               |
| **声明驱动与代码生成**       | `ability.yml` 与协议 `info.yml` 生成 Provider，业务逻辑在 `src/logic/`，减少样板与适配分叉 |
| **面向 AI 系统的原生积木**   | 会话、多轮工具循环、Skill、OpenAI 兼容 chat 等；Agent 是消费者之一                         |
| **Rust 运行时**        | Workspace 使用 Rust edition 2024，侧重内存安全、高性能与可部署性                        |
| **可选监测**             | `apps/monitor`：调用链与生命周期事件、统计与图、CLI / 实时 TUI / 可选 HTTP / 进程内查询；不需要时可不引入该包 |
| **生态适配在边缘**         | 外部兼容通过独立 Ability 或装配模块接入，核心 Ability 与引擎保持稳定                           |

---

## 适合谁

| 场景              | 说明                                  |
| --------------- | ----------------------------------- |
| **多协议出口**       | 需要同时服务 Agent、MCP、CLI、HTTP 等多种暴露面的团队 |
| **网关与 Harness** | 构建 MCP 服务、工具网关或 Harness 运行时         |
| **统一治理**        | 希望收敛鉴权、限流、观测与装配策略的基础设施团队            |
| **按场景裁剪**       | 不同部署需要不同能力组合                        |
| **早期共建**        | 愿意参与运行时设计的 Rust / AI 基础设施工程师        |

---

## 可选监测

LEAF 提供可选的监测能力，位于 `apps/monitor`（包名 `leaf-monitor`）。需要时加上依赖并配置 `monitor.yml`；不需要时不引入即可。

### 能观测什么

事件与进行中的快照用于**梳理调用关系与生命周期状态变化**，而不只是成败日志：

- **调用链** — 一次根调用及其嵌套调用，可还原调用树
- **时间与顺序** — 起止时间与顺序，便于衡量耗时
- **生命周期** — Ability 的加载、卸载与状态变化
- **队列统计** — 近期事件量、排队深度与丢弃情况（有界队列，不拖慢业务调用）

可用 `leaf_monitor::graph::build_chain_graph` 重建调用树，或通过下方入口查询图数据。

### 怎么查看

| 方式 | 说明 |
|---|---|
| 异步 JSONL 日志 | 一行一条事件 |
| 查询 CLI | 查询链、事件、日志、**统计**与**调用图**；支持离线读日志 |
| 实时 TUI | `cargo run -p leaf-monitor -- tui`；Tab 在链 / 事件 / **图**之间切换 |
| 可选 HTTP | 默认关闭；本机提供同样视图的接口 |
| 进程内 API | 在同一进程内查询链、事件、统计与图 |

详情见 `apps/monitor/README.md` 与技能 `SKILL/use-monitoring`。

## 手册

| 中文                                              | English                                          | 内容                    |
| ----------------------------------------------- | ------------------------------------------------ | --------------------- |
| [快速开始](manual/getting-started.zh-CN.md)         | [Getting started](manual/getting-started.md)     | 创建应用、调用与订阅            |
| [清单与裁剪](manual/catalog-and-expose.zh-CN.md)     | [Catalog & expose](manual/catalog-and-expose.md) | 协议清单与 Expose 过滤       |
| [编写 Ability](manual/writing-abilities.zh-CN.md) | [Writing abilities](manual/writing-abilities.md) | 编写 Ability            |
| [架构概览](manual/architecture.zh-CN.md)            | [Architecture](manual/architecture.md)           | 公开架构说明                |
| [生态插件](manual/ecosystem-plugins.zh-CN.md)       | [Ecosystem plugins](manual/ecosystem-plugins.md) | Claude Code Mod 与 DSH |

---

## 项目状态

LEAF 目前是**早期预览**：

| 维度  | 说明                      |
| --- | ----------------------- |
| 方向  | 架构明确，API 与布局仍可能调整       |
| 适用  | 评估设计、原型、参与贡献            |
| 慎用  | 未锁定版本且未充分测试时，不宜用于关键生产路径 |

**路线图**

- 提供更多 AI 系统特性的原生支持
- 完善 Ability SDK 与 Gateway client 的库形态
- 增强协议端口与装配配置能力
- 支持 Python、TypeScript 等多语言 Ability
- 补全观测、鉴权与限流等治理能力
- 提供更多可复用的 App 装配示例

**参考应用**：`apps/agent-test`（Agent 回路）、`apps/web-agent`（Web）

**`external/`** 为实验性目录，作者 SDK 与探针仍可能变更。

---

## 设计原则

1. **能力协议无关** — Ability 保持协议中立
2. **装配决定形态** — App 决定加载、暴露与绑定
3. **调用收敛到 Gateway** — 统一入口与治理
4. **业务逻辑在 Ability crate** — 运行时内核保持纯粹
5. **生态适配在边缘** — 外部兼容逻辑独立接入
6. **边界清晰优先** — 先稳定结构，再扩展功能

---

## 贡献

欢迎参与早期建设。贡献前请先阅读 [架构概览](manual/architecture.zh-CN.md) 与 [编写 Ability](manual/writing-abilities.zh-CN.md)。

**重点方向**

- Ability 示例与最佳实践
- 协议适配器
- 装配配置与治理策略
- 测试、文档与基准
- 多语言 Ability 支持

如果你正在构建多协议 AI 能力层，或对「能力运行时」有不同判断，欢迎开 Issue 讨论架构取舍。项目更关心**边界清晰**、**治理收敛**与**场景可验证性**。

---

## License

以仓库中的 [LICENSE](LICENSE) 文件为准（Apache License 2.0）。
