# SKILL

每项技能以 `SKILL/<name>/SKILL.md` 为入口。

| 目录 | 用途 |
|------|------|
| `develop-ability` | 编写规范 Ability |
| `ability-placement` | Ability 落点 |
| `guest-ability` | Guest（Node / Python / 仓颉） |
| `expose-governance` | Expose / open / 事件治理 |
| `framework-update` | 框架更新与同步 |
| `use-monitoring` | 使用可选底层监测、在其上做监测类服务 |
| `register-skills` | 把本仓技能挂到全局 Cursor skills（默认不执行） |

## 框架自带身份（给作者的边界）

普通 Ability / App 作者**不要**去「申报内置」；`external/`、模板脚手架或用户新建包不要伪装成「框架自带」内容。框架自带身份的**识别**与维护约定见 **`framework-update`**（落点边界见 `ability-placement`、`guest-ability`）。

## 底层监测

可选底层监测在 `apps/monitor`。如何接入、查询、画链路图/状态图，以及如何在其上做监测类服务：见 **`use-monitoring`**。拉取更新时与本地改动冲突：见 `framework-update`（先问用户）。
