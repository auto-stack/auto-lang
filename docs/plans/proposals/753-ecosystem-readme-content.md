# PLAN-753 内容方案：生态 README 与语言概览

2026-10-10 · revision 1 · 待确认方案，尚未替换 README。

## 根页阅读顺序

1. Logo、Auto: AI × Lang × OS、中英切换与资料入口。
2. 两段生态概述 + 一张完整 AutoOS 桌面主图。
3. 四层生态表：AutoLang / AutoUI / AutoOS / AutoAI，职责、当前路径和深入入口。
4. 四应用表：AutoEdit / AutoShell / AutoMusk / JadeEdit；AutoDown 文档基础单列，
   系统工具与 Demo 简述，不手工维持全部应用数量。
5. AutoEdit 与 AutoShell 两张官网共享原图。
6. v0.5 进展：脚本到发布、多端 UI/桌面、Rust/Python 接入、可检视工具和曲线自举。
7. 运行一个真实例子：Hello / counter；源码构建、已有 CLI、在线内容的前置条件明确。
8. 按目的学习：语言概览、Tour、script-to-ship、UI、产品、Playground、书籍与工具链。
9. 仓库地图、贡献/开发规约和 MIT 许可。

## 首页开头文案

### 中文

# Auto · AI × Lang × OS

**从即时脚本，到跨端界面、桌面和应用。**

Auto 是围绕 AutoLang 构建的语言与应用生态。用 AutoVM 即时运行脚本，以 Rust 转译路径
构建发布程序；AutoUI 将同一份界面定义用于 Web 和原生桌面，AutoOS 组织桌面与系统应用，
AutoAI 提供应用共享的模型服务和 Agent 工具。

这些能力连接起编辑、命令自动化、AI 辅助开发和知识管理：AutoEdit、AutoShell、AutoMusk
与 JadeEdit 分别面向这些工作。本仓库是整个生态的入口，也维护语言实现、UI 框架与工具链。

### English

# Auto · AI × Lang × OS

**From instant scripts to interfaces, desktops, and applications.**

Auto is a language and application ecosystem built around AutoLang. Run scripts immediately on
AutoVM and build programs through Rust transpilation. AutoUI brings shared interface definitions
to the web and native desktop; AutoOS organizes the desktop and system applications; AutoAI supplies
shared model services and Agent tools.

AutoEdit, AutoShell, AutoMusk, and JadeEdit put these foundations to work in editing, command
automation, AI-assisted development, and knowledge management. This repository is the ecosystem's
entry point and maintains the language implementation, UI framework, and developer toolchain.

## 生态与应用表

| 项目 | 介绍重点 | 实际资料入口 |
|---|---|---|
| AutoLang | 动静结合；AutoVM 脚本；Rust 主静态发布路径；接口级互操作支持 | 新双语语言概览、docs/tour、docs/script-to-ship、crates/auto-lang |
| AutoUI | 状态/事件/视图单源；Vue/iced；VM 热重载、Rust 生成、F12/MCP；移动 Demo/路线 | website/ui/index.md 与 zh 镜像、examples/ui/README.md |
| AutoOS | 当前宿主内虚拟桌面、窗口/启动器/工作区/系统应用；LaOS 长期方向 | website/os.md、website/autoos/index.md 与 zh 镜像、auto-stack/auto-os |
| AutoAI | 共享客户端、aaid 模型服务、角色/技能/Agent；工具在应用层执行 | website/ai.md 与 zh 镜像、auto-stack/auto-ai |

四应用用途：AutoEdit（代码、文本与差异比较）、AutoShell（会话、结构化管道与脚本）、
AutoMusk（Plan/Spec 开发任务与验证）、JadeEdit（文档与本地知识）。
AutoDown 是文档格式与编辑器基础，不能与 JadeEdit 混成一个名称。
各项目采用 Auto/Rust 等不同组合，不写「整个生态全用 Auto 实现」。

## 共享截图

| 用途 | README 相对引用的真实文件 | 网站依据 / 日期 |
|---|---|---|
| 首图：桌面 | website/public/desktop-showcase/02-desktop-dark.png | desktop-showcase.ts / 2026-10-01 |
| 工作场景（备用或展开） | website/public/desktop-showcase/06-productivity-dark.png | 同上 |
| AutoEdit | website/public/apps/autoedit/overview-dark.png | 产品专题 / 用户提供 2026-10-01 |
| AutoShell | website/public/apps/autoshell/ash-01.png | AutoShellPreview.vue / 2026-09-29..30 捕获族 |

已目检工作场景、AutoEdit、AutoShell 原图；首图实施时再目检。
保留完整原图、可点击查看；桌面小组件来自已启动并最小化的应用。
中英文使用相同图片，翻译 alt/说明，不改绘图中的中文 UI，不仿造终端或生成产品截图。
不复制 PNG 到 docs/，不增第二份资产。Musk/Jade 走查尚未准备，采用文字介绍。
旧图注明来源，不能当作冻结发行候选的验证。

## 当前语言子页

新增 docs/language/overview.md、overview.cn.md；网站生成路由分别为
/docs/language/overview 与 /zh/docs/language/overview。

| 章节 | 要介绍的内容 | 核对来源 |
|---|---|---|
| 执行与发布 | .at、.as/#[script]、AutoVM、auto trans、Rust 发布、多目标边界 | CLI、auto-cli/trans Specs、v0.5 |
| 基础与函数 | let/var/const、类型、f-string、函数/闭包/控制流/集合 | docs/tour/ch01..04、ch07 及 .at |
| 类型与抽象 | type/enum、is 匹配、ext/spec/泛型 | Tour ch02/ch05/ch08/ch09、types/frontend Specs |
| 错误与并发 | ! 与 .?、Task/Msg/Actor、async/await、已支持生成器 | Tour ch06/ch11、actor-concurrency、VM 语料 |
| 模块与互操作 | 多模块、Rust dep/use.rs/shim、CPython/use.py、C FFI | Tour ch10/ch12、parity、v0.5 |
| 配置与界面 | 配置/Node 和 AutoUI model/event/view | schema/aura.at、真实 counter、UI Spec |
| 工具与深入 | CLI/AutoMan/Cache/LSP/MCP、Playground、书籍、自举 | module Specs、script-to-ship、aavm Spec |

用真实例子展示当前能力，说明各 VM/转译路径支持范围。
自举描述为 Auto→Rust→Rust 编译器的实验闭环。
不把旧 v0.2 Draft 规范称为最新语言介绍，也不承担本轮全量规范重审。

## 旧内容去向

| 原 README 内容 | 去向/更新 |
|---|---|
| Automation/Flexible/Fullstack | 四层生态与应用用途 |
| evaluator/AUTO_EXECUTION_ENGINE | 移除，改为当前 AutoVM 与 CLI 路径 |
| Lang Tour / Syntax 长代码 | 当前双语语言概览、真实 Tour 与特性专题 |
| AutoConfig/Template/Man | 概览与工具/教程入口；不保留未验证旧 pac.at |
| Shell 长例子 | 最新 AutoShell 产品/使用指南，不混作普通 .at 语法 |
| GPUI/DynamicWidget/UI 老例子 | Vue/iced 与真实 counter；移除未来静态模式旧承诺 |
| Enums/Generators/Async Planned | 当前已实现能力和后端边界 |
| Python/JS 转译长节 | 概览目标/互操作表与深入文档，核对 auto trans 真命令 |
| 仅需 Rust/Cargo | 当前 workspace/auto-down/Python 等实际前置条件 |
| 旧 Plan 相对链接 | current-state 资料；必要历史用 archive 稳定路径 |

## 交付

两份 README、两份语言概览、必要学习导航、链接/例子/共享图片检查与 GFM 窄宽屏截图，
加上独立复审和知识沉淀收据。公网域名目前占位，IP 专题路由回落；入口优先用仓内实际
作者源，未经验证不增加错误在线链接或下载地址。部署另属发布流程。
