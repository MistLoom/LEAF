# LEAF

**LEAF is a lightweight AI development framework for the AI ecosystem, built with Rust.** One set of **Abilities** serves Agent, MCP, CLI, Skill, Tools, and HTTP (and more).

It helps teams and individuals build **secure, reliable, high-performance** harnesses, personal assistants, in-house AI systems, tool gateways, and related applications.

The **multi-protocol AI capability layer** keeps the same capability **organized, assembled, exposed, and governed**, with Abilities enabling **cross-app and cross-system scheduling**.

Ability-to-Ability communication supports **shared memory**, **multi-process**, and **cross-device networking** models, so you can tune for performance, availability, and other requirements.

> **Experimental**: DeepSeek harness plugins, Claude Mods, and similar ecosystem integrations ship at the edge via Abilities, with reference examples. See [Ecosystem plugins](manual/ecosystem-plugins.md).

[The problem](#the-problem-with-typical-ai-systems) · [Core model](#core-model-app--ability--gateway) · [Runtime](#why-a-runtime-not-just-a-library) · [Features](#key-features) · [Manual](#manual) · [Status](#project-status) · [Contributing](#contributing)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-early%20preview-yellow.svg)](#project-status)

中文：[README.zh-CN.md](README.zh-CN.md)

---

## The problem with typical AI systems

Unlike traditional single-product designs, many AI application stacks split into sub-scenarios. Besides integrating legacy systems, sub-applications often need to coordinate—for example:

- Retrieval and ticketing triggered from a chat flow
- Chaining document understanding with command execution in one business path
- Handing collected data to other systems for analysis or content generation

In AI systems, a capability such as “recognize an image” or “run a command” often must appear in many places at once:

| Surface | Form |
|---|---|
| Agent | `tool_calls` |
| MCP | tools |
| CLI | commands |
| HTTP | APIs |
| Harness | internal dispatch entry points |

Some capabilities also need to surface as **Skills** on a unified agent entry. Writing a separate adapter for each surface leads to:

- **Duplicated** business logic
- **Inconsistent** behavior across protocols
- **Auth, rate limits, and observability** scattered everywhere
- Trimming a minimal runtime requires **changing business code**
- Adding a new protocol surface **touches many call sites**

LEAF’s design starting point: **keep capability implementations protocol-agnostic; defer protocol mapping to assembly time.**

---

## Core model: App + Ability + Gateway

| Concept | Role | Key point |
|---|---|---|
| **Ability** | Implements a capability | Protocol-agnostic—“what,” not how it is exposed |
| **App** | Assembly and governance | Chooses which Abilities load, which config binds, which protocol ports expose |
| **Gateway** | Unified invoke pipe | Every call is `instance + method + args`—one observability, auth, and rate-limit entry |

> **Ability** is the stable atom, **App** is the variable assembly policy, **Gateway** is the single execution entry.

## Why a runtime (not just a library)

LEAF needs runtime control so capabilities are invoked, exposed, and governed the same way everywhere:

| Need | Why |
|---|---|
| **Unified entry** | As a library-only integration, callers can bypass the Gateway and call business functions directly, breaking unified observability and governance |
| **Protocol projection** | Serving MCP, CLI, and HTTP at once involves registries, routing, config loading, and process lifecycle |
| **Assembly trimming** | `AbilityExpose` and similar config decide load, disable, and parameter binding—assembly-time concerns |
| **Declaration-driven layout** | `ability.yml`, protocol `info.yml`, generated Providers, and `src/logic/` layout are framework-level contracts |

The App owns assembly and governance; Abilities stay pure and reusable. The same Ability codebase can assemble into different runtime shapes:

- `AbilityExpose` gates Port visibility
- Same-app `ctx.call` is unrestricted
- Cross-app calls require `open: true`

See [Catalog & expose](manual/catalog-and-expose.md).

LEAF scopes itself to the **capability-layer runtime**; the application main loop stays with you. Libraries and SDKs at the core and edge include:

- `Ability SDK`
- `Gateway client`
- Protocol adapters
- Config parsers
- Code generators

Ability Guest SDK implementations (under `SDK/`, all speaking `guest-protocol-1` stdio JSON lines):

| Language SDK | Directory | `framework_version` | Project marker | Guest entry | manifest/methods/lifecycle | ctx.call/dep/catalog | local_get/set | emit/subscribe/handle/event_next | load/unload | feedback | chain_status/share | push_effect+undo | register_aspect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Rust (wire-protocol lib) | `SDK/ability-sdk` | `rust@1` | `Cargo.toml` | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (in-process) |
| Node | `SDK/ability-sdk-ts` | `node@1` (`ts@1` alias) | `package.json` | `src/serve.mjs` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (rejected as Guest) |
| Python | `SDK/ability-sdk-py` | `python@1` (`py@1` alias) | `pyproject.toml` | `src/serve.py` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (rejected as Guest) |
| Cangjie | `SDK/ability-sdk-cj` | `cangjie@1` (`cj@1` alias) | `cjpm.toml` | `src/serve.cj` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (rejected as Guest) |

Guest vs in-process Rust: Guests do not support `register_aspect` (in-process aspects are Rust-runtime only). Protocol details: [Guest wire protocol](manual/ability-guest-protocol.zh-CN.md).

Unified governance where it matters, local reuse where it helps. Details: [Architecture](manual/architecture.md).

---

## Key features

| Feature | Description |
|---|---|
| **Implement once, project to many protocols** | One Ability can assemble into MCP, CLI, HTTP, or agent tool calls; business logic stays in the capability layer |
| **Unified Gateway invoke pipe** | Calls use `instance + method + args`—one observability, auth, and rate-limit surface |
| **Trim at assembly time** | `AbilityExpose` controls load, disable, and binding per instance; one codebase, different feature sets and minimal runtimes |
| **Declaration-driven codegen** | `ability.yml` and protocol `info.yml` generate Providers; business logic lives in `src/logic/` with less boilerplate and fewer adapter forks |
| **Building blocks for AI systems** | Sessions, multi-turn tool loops, Skills, OpenAI-compatible chat, and more; the agent is one consumer among many |
| **Rust runtime** | Workspace targets Rust edition 2024—memory safety, performance, and deployability |
| **Ecosystem at the edge** | External compatibility via separate Abilities or assembly modules; core Abilities and the engine stay stable |

---

## Who it is for

| Scenario | Description |
|---|---|
| **Many protocol surfaces** | Teams that must expose the same capabilities via Agent, MCP, CLI, HTTP, and more |
| **Gateways and harnesses** | Teams building MCP services, tool gateways, or harness runtimes |
| **Unified governance** | Platform teams converging auth, rate limits, observability, and assembly policy |
| **Per-deployment trimming** | Deployments that need different capability mixes |
| **Early co-design** | Rust / AI infrastructure engineers shaping an early runtime |

---

## Manual

| English | 中文 | Contents |
|---|---|---|
| [Getting started](manual/getting-started.md) | [快速开始](manual/getting-started.zh-CN.md) | Create an app, invoke, subscribe |
| [Catalog & expose](manual/catalog-and-expose.md) | [清单与裁剪](manual/catalog-and-expose.zh-CN.md) | Protocol lists & expose filters |
| [Writing abilities](manual/writing-abilities.md) | [编写 Ability](manual/writing-abilities.zh-CN.md) | Authoring Abilities |
| [Architecture](manual/architecture.md) | [架构概览](manual/architecture.zh-CN.md) | Public architecture model |
| [Ecosystem plugins](manual/ecosystem-plugins.md) | [生态插件](manual/ecosystem-plugins.zh-CN.md) | Claude Code Mods and DSH |

---

## Project status

LEAF is in **early preview**:

| Dimension | Notes |
|---|---|
| Direction | Architecture is clear; APIs and layout may still change |
| Fit | Design evaluation, prototypes, contributions |
| Caution | Not for critical production paths without pinned versions and thorough testing |

**Roadmap**

- More first-class AI system features
- Library forms of Ability SDK and Gateway client
- Stronger protocol ports and assembly configuration
- Python, TypeScript, and other polyglot Abilities
- Observability, auth, and rate-limit governance
- More reusable App assembly examples

**Reference apps**: `apps/agent-test` (agent loop), `apps/web-agent` (web)

**`external/`** is experimental—author SDKs and probes may still change.

---

## Design principles

1. **Capabilities stay protocol-agnostic** — Abilities remain neutral
2. **Assembly defines shape** — The App decides load, expose, and bind
3. **Calls converge on the Gateway** — One entry, one governance model
4. **Business logic in Ability crates** — The runtime kernel stays thin
5. **Ecosystem at the edge** — External compatibility plugs in separately
6. **Clear boundaries first** — Stabilize structure, then expand features

---

## Contributing

Early contributions are welcome. Read [Architecture](manual/architecture.md) and [Writing abilities](manual/writing-abilities.md) first.

**Focus areas**

- Ability examples and best practices
- Protocol adapters
- Assembly config and governance policy
- Tests, docs, and benchmarks
- Polyglot Abilities

If you are building a multi-protocol AI capability layer—or disagree on what a “capability runtime” should be—open an Issue on trade-offs. The project cares about **clear boundaries**, **converged governance**, and **verifiability in real scenarios**.

---

## License

See [LICENSE](LICENSE) in this repository (Apache License 2.0).
