---
name: register-skills
description: >-
  Copy Leaf skills from the repository SKILL/ directory into a Cursor project
  skills folder (.cursor/skills) or the user global skills folder (~/.cursor/skills).
  Use only when the user explicitly asks to install or register these skills.
disable-model-invocation: true
---

# 注册技能

源目录为本仓库根 `SKILL/<name>/SKILL.md`。仅在用户明确要求时执行拷贝；默认不注册。

## 目标位置

| 范围 | 路径 |
|------|------|
| 项目 | `<目标仓库>/.cursor/skills/<name>/` |
| 全局 | `~/.cursor/skills/<name>/` |

不要写入 `~/.cursor/skills-cursor/`。

## 清单

- `develop-ability`
- `ability-placement`
- `guest-ability`
- `expose-governance`
- `framework-update`
- `register-skills`

## 操作

仓库根执行：

```powershell
.\SKILL\register-skills\scripts\register.ps1 -Scope Project
# 或
.\SKILL\register-skills\scripts\register.ps1 -Scope Global
```

`-Scope Project` 默认拷到当前仓库 `.cursor/skills/`；可用 `-ProjectRoot` 指定其它仓库。
