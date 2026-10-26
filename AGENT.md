# apps说明
- 除apps/[app/model/subSystem/other]/中ability目录以外，不限制 apps/[app/model/subSystem/other]的任何实现语言和架构，而框架根目录结构应尽量保持整洁
- apps仅作架构层级划分作用，位于框架架构顶层，不限制apps下子目录用途

## ability常见使用场景
1. ability一般可用于跨应用的能力调度、协同工作、功能开放使用，以及应用内需受统一链路管理、链路追踪、日志记录、子系统调度等场景使用，其属于应用层能力；除此之外也可借助ability来为产品实现第三方扩展能力支持
2. 可借助ability能力的组合调度，来实现高级统计、链路追踪、流量控制、日志管理等场景需求。例如你可以创建一个负责该场景需求的子应用来开放所需ability，供其他apps下的应用使用

## 项目限制
- 为避免影响框架更新，以下目录禁止更改：/frame、/SDK、/apps/ability-core、apps/plugin-host
- 基于框架实现的其他项目，更新时直接拉取/frame、/SDK、/apps/ability-core、apps/plugin-host即可

