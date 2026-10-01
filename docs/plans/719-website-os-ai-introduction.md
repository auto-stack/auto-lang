---
plan_id: PLAN-719
status: reviewed
feature_name: Website OS 与 AI 介绍重组
author: [agent]
created_at: 2026-10-01
updated_at: 2026-10-01
plan_revision: 2
current_step: 8
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/website/design/os-ai-introduction.md]
touched_goals: []
affects: [website]
---

# [PLAN-719] Website OS 与 AI 介绍重组

## 0. 变更摘要

将 OS/AI 相关介绍从零散宣传页重组为基于架构与现状的双语说明：**LaOS 是实现理念，OS over OS 是当前产品形态，AI + Lang + OS 是长期面向人的使用方向**。三者承接而非互相替换。

具体可审阅文案在 [中英文介绍稿](proposals/719-os-ai-copy.md)，包含 OS/AI 完整正文、页面分工、配套首屏和首页卡片摘要。本计划将其落到 VitePress 网站，核对相关 AutoOS/桌面 UI/发布页内容，清除过时路线和缺证据的统计/示例。

r2 另加入两篇独立历史/展望文章，四份完整中英文稿已交付：[AutoOS 中文](proposals/719-autoos-history.zh.md)、[英文](proposals/719-autoos-history.en.md)、[AI 中文](proposals/719-auto-ai-history.zh.md)、[英文](proposals/719-auto-ai-history.en.md)。取材与引用核验见 [研究收据](proposals/719-history-sources.md)。

本计划不与执行中的 PLAN-718 合并。718 负责学习入口与阅读体验；719 负责 OS/AI 介绍，实施时在独立 worktree 中吸收实际已合入的主题状态。

## 1. 目标

- 准确解释 LaOS 的组件化系统理念、跨生态适配与一致接口目标，以及它和 OS over OS 的连续关系。
- 区分宿主虚拟桌面、复用现有内核的独立系统、自有内核三种运行/发展形态。
- 描述 AI 深度参与项目开发、共享模型服务与 Agent 工作工具的现状，并呈现面向知识管理、工作生活的人本长期方向。
- 从真实项目文件说明 auto-lang（语言/框架）与 auto-os（桌面/应用/产品集成）的分工；明确当前 Auto + Rust 实现基础。
- 中英文介绍、首屏图示、桌面页、首页入口与 v0.5 相关摘要一致，读者可查到实际能力与资料入口。
- 在概括介绍外提供两篇独立长文，解释关键架构变化的动因、历史落地与未来方向；正文附设计/提交/计划来源，从 OS/AI 总览可进入阅读。

**非目标**：实现内核/发行版/新平台支持、修改跨仓运行时或 API、重写网站框架、设计 AI 自动代理权限体系、给未来路线承诺发布日期、全面重新编辑 v0.5 或历史架构文档、替代 718 的阅读 UI 改造、部署上线。

## 2. 架构方案

保留 **VitePress + Vue 3** 和 715 设计 tokens。页面职责为：

| 页面（均有 ZH 镜像） | 职责 |
|---|---|
| `/os` | 理念连续性、组成、运行形态、当前进展、长期方向 |
| `/autoos/` | 当前桌面、窗口/应用/配置体验、真实截图及项目入口 |
| `/ai` | AI 的角色、应用/Agent/服务分层、已有接入、长期知识工作方向 |
| `/ui-desktop` | 当前 Vue/原生桌面运行与生成路径，不再写桌面后端尚未推出 |
| `/` 与 `/v05/` | 同步相关简要表述与链接，保留其他内容 |
| `/articles/autoos-history` | AutoOS 的理念起点、桌面/产品/合成接缝演进、现状与展望 |
| `/articles/auto-ai-history` | AI 服务、Role/编排、Auto 化、使用行为与知识工作方向的演进 |

新增 `website/.vitepress/theme/data/os-ai-introduction.ts`（新）保存两语言的介绍、状态与架构关系；新增 `OSIntroduction.vue`、`AIIntroduction.vue`（新）用于相应总览页。AutoOS 桌面页和配套页面只复用有关摘要/状态，不再复制完整理念。实施可采用等价的可索引 Markdown 正文 + 数据组件组合，但需记录取舍，不能让首屏与正文成为相互矛盾的数据源。

用介绍式首屏、实际图像、简明架构图和带文字状态的表格代替大数字墙。架构图示不是截图；宿主/内核和渲染/合成设施必须在图中出现，不能形成“Auto 直接在所有硬件上运行”的误读。

## 3. 技术栈

现有 VitePress 1.6.x、Vue 3、主题 CSS 与 Playwright。不加 UI 依赖，不升级框架。真实桌面图复用现有 `website/public/v05/desktop-hero.png`、`desktop-multiwindow.png`、`desktop-light.png` 和设置/应用截图，由既有 `EvidenceImage.vue` 或 `ScreenshotGallery.vue` 展示；图中能力、来源与截图时间按已有材料说明。

## 4. 需求分析与背景调查

### 4.1 用户授权与约束

2026-10-01 用户要求：结合 LaOS → OS over OS → AI + Lang + OS 的概述、现有网站以及 auto-lang/auto-os 现状，重新组织 OS/AI 板块，减少宣传、加强介绍；明确自有内核未做、未来会做，未来也可能探索语言底座自实现 native。

用户随后明确追加：除概括介绍，还写 AutoOS 历史沿革/未来展望，以及经过设计文档、代码与计划历史研究的 AI 相关文章，从网站介绍提供链接。r2 将两篇文章与双语镜像、引用核验、总览入口纳入范围；四份具体长文稿已在 proposals 中交付。

这授权本次事实调查、介绍稿、文章撰写和具体改版计划。按仓库 AGENTS.md L1 的“present for confirmation before executing”，将可审阅计划与完整双语稿提交确认后再实施；没有把新的内容范围加入已经 executing 的 718。

实施只写 auto-lang 的 website 和计划/报告；auto-os、auto-ai、auto-musk、auto-os-config、auto-shell、auto-term 与相关工具只读取材。无需各仓实现工作树。实现树为 `D:/autostack/.wt/lang-719/auto-lang`，分支 `plan-719-dev`，禁止 junction/symlink。当前主仓其他会话的 .autoos/specs.json、718 状态及未跟踪资源不得带入本计划。

暂无用户预算或自动推进次数限制。此为网站内容/展示任务：不跑 cargo 测试、不跑 docs_gen、不调用收费模型、不模拟系统启动成功。

### 4.2 现状与差异（权威 Spec + 当前实现交叉核对）

| 现有说法/问题 | 证据 | 新介绍规则 |
|---|---|---|
| OS 几乎等同 Client/Daemon/config | website/os.md、zh/os.md | 介绍整个组件体系；Daemon 为共享服务组织方式之一 |
| 固定 Pop!_OS/COSMIC 独立发行版路线 | website/os.md、autoos/index.md | 现有内核适配方向；Linux/OpenHarmony 独立产品均未来，COSMIC 只可作历史探索例子 |
| `/ui-desktop` 仍“即将推出”，列 Tauri/Winit/LVGL 为当前核心 | website/ui-desktop.md、zh/ui-desktop.md 与 UI Spec 的 VM/Vue/RQ 现状 | 如实写已有运行/生成路径，平台覆盖按证据，不抹掉已有桌面成果 |
| AutoOS 全靠 auto-lang examples，自带 app 仍是未来清单 | auto-os apps.manifest、shell/、Design 01、desktop.ps1 | 两轴分工，真实 app 与演示资源区分，不假定所有 app 同成熟度 |
| AI “4 核心 crates”“∞ 模型”、所有应用唯一网关 | website/ai.md、zh/ai.md；auto-ai Cargo.toml、server.rs | 按职责讲已配置提供商/接入应用，移除静态夸张统计 |
| AutoMusk 纯 Auto/纯 VM 的宣传式介绍 | auto-musk README 的 implementation/repository layout | Auto 源码、生成代码与 Rust 基础设施组合，应用 workflow 不等同通用 agent loop |
| AI 代码/终端输出未绑定真实源或实跑 | website/ai.md、zh/ai.md | 删除这些视觉伪证据；可用明确标注的架构关系图替代 |
| AI 长期目标只有“构建 Agent” | 用户本次说明 + 当前工具 | 增加以人为本知识/工作生活方向，与已有工具和未来协作分别说明 |
| `/docs/os`、`/docs/ai` 源根页未发现 | docs/os.md、docs/ai.md 不存在；生成器不专门补该页 | T-01 再核对生成产物；无真实落地页则替换为存在的资料/介绍入口 |
| 跨生态一致性被读成全平台全接口已完成 | 用户理念、virtual-desktop 正式裁定、Harmony 战略 Draft | 一致性是设计目标；现状逐项描述，UI 为布局/交互/主题一致，不许诺全平台像素相同 |

不将设计文档的 Designed/旧命名当当前实现：`docs/design/15-ai-daemon-infrastructure.md` 仍是 aillmd 等历史设计，实际服务名为 aaid。auto-os README 中 Stage A 等旧状态亦仅参考背景；当前 apps.manifest、shell Spec、启动脚本优先用于现状对账。

### 4.3 当前能力取材锚

- auto-lang 调查 HEAD `e0fb4e4e41d6c692db03efbf0b036a4c3e81c64c`。
- auto-os HEAD `73d02b50f535543bd635c6f6f49dad6835efff45`：`docs/design/01-stage-b-desktop-migration.md` 两轴分工；`apps.manifest` 当前 VM launch 声明；`docs/specs/shell/desktop-app-launch.md` 运行声明/边界；`shell/{shell,desktop,switcher,notification_center,dashboard}.at`；`scripts/desktop.ps1` 真实桌面启动与 RQHost 合成宿主接缝。
- auto-ai HEAD `5a50a55844d7aa3523b593f21ba0fb03d18eac48`：Cargo workspace；`crates/auto-ai-daemon/src/{server,pool}.rs` 已有模型/status/models/usage 接口及并发设施；`docs/specs/auto-ai/role-model-binding.md`、`docs/specs/auto-ai-cli/shell-execution.md` 与其实现互证。Spec 内“进入评审”不擅自改成跨仓已评审结论。
- auto-musk HEAD `22f06ba1808a3cf270795836d2528ca6f5a2d8f4`：README 明确阶段、人工监督与 Auto/Rust 混合实现。
- auto-os-config README：同一 Auto UI 源支撑 Vue/VM、配置通用编辑及现有外部 back；auto-shell README：Rust 实现、结构化跨平台命令、POSIX 风格 flag、F1/F2/F3；这些项目没有因本计划发生代码更新。
- 核心 Spec：`docs/specs/overview.md`、`docs/specs/website/project.md`、`docs/specs/website/design/ui-presentation.md`、`docs/specs/auto-lang/ui/overview.md`、`docs/specs/auto-lang/ui/design/shell-a2r-seams.md`。
- 架构依据：`docs/design/autoui/virtual-desktop.md` 与 `docs/design/strategy/harmonyos-ecosystem-strategy.md`（OpenHarmony 终态方向，不是当前可交付独立 OS）；LaOS 初始记录 `docs/design/raw/os.md` 是历史材料，语法示例不当成可运行当前代码。

| 文件 | SHA256 前 12 位 |
|---|---|
| website/os.md | 01b003e4e4e7 |
| website/zh/os.md | 65ce3cadf8f3 |
| website/ai.md | 9be59a852d88 |
| website/zh/ai.md | 80c462f9a666 |
| website/autoos/index.md | 0cf118b3a95a |
| website/zh/autoos/index.md | cb0f4b9a7469 |
| website/zh/ui-desktop.md | 2b42a1156f2c |
| docs/specs/auto-lang/ui/overview.md | d60137cc8779 |
| docs/design/autoui/virtual-desktop.md | 1397105dc212 |
| docs/design/strategy/harmonyos-ecosystem-strategy.md | 7eb05307c1de |
| D:/autostack/auto-os/apps.manifest | 41c7ed73b53f |
| D:/autostack/auto-os/scripts/desktop.ps1 | f57894ac4bee |
| D:/autostack/auto-ai/crates/auto-ai-daemon/src/server.rs | 69489a30095b |
| D:/autostack/auto-musk/README.md | 213e001aa6c6 |

本计划仅交付网站介绍，不将对应 GOAL-009 等运行时目标标成新能力达成，touched_goals 留空。用户对自有内核和 native 底座的未来设想按“长期方向”表达，不修改其他项目的路线优先级。

### 4.4 r2 历史研究与现状修正

研究材料与 33 个唯一源引用收据见 `proposals/719-history-sources.md`；版本锚仍为 §4.3 的跨仓 HEAD，本仓稿件父提交为 `731aa42bcdff59297db6f35659da40d24b3f9660`。

- OS 早期 Task/lifetime 设计可追到 2025-12-22（84cdb6dc1）；2026-08-26 虚拟桌面裁定（caa802981）；09-01 Smithay Stage 1；09-07 产品仓/桌面表面迁移（f05bcec/1dcabf0）；09 月编译壳与协议/RQHost 继续演进。不得把源码改名日期当理念诞生日。
- AI 2026-06-17 初始化（f95d6ec）、06-18 职责重构（3b4976f）；07-01 Role 命名（73aecc6）；07-16 编排下沉完成（2aa3700）；08-04 Workflow 物理退役（2bb84c9）；08-07 历史 Auto daemon 全链验证（afc803a）；08 月事件/content-details/compaction；09 月 ash-first、显式模型链及 Plan 验证机制。
- 实现核对发现 auto-ai `src/lib.at` 部分注释仍说 Workflow deprecated，而当前导出是 orchestration，workflow 源已不存在；文章按实际导出和删除提交讲述，不照抄旧注释。
- musk README 对“当前 plan-dev 单角色四阶段”的描述滞后于 `docs/specs/modules/plan-flow.md`。当前 Spec 为 advisor/coder/reviewer/assistant 的职业分工与 execute Human gate；文中把单角色作为前史，当前结论以 Spec 为准。
- 历史 live/e2e 证据仅说明当时版本，不是本次重新运行；AI 长期个人知识与会话压缩的职责区分。LaOS/OS over OS 连续关系、自有内核/native 与人本目标依据用户概述，不虚构最早口号日期或代码落地。

## 5. 详细设计

### 5.1 内容与状态表达

以介绍稿为 r1 文案基线。OS 顺序为“是什么 → 理念连续性 → 组成 → 运行形态 → 当前进展 → AI+Lang+OS”；AI 顺序为“角色 → 共享服务原因 → 应用/Agent/服务分层 → 当前接入 → 能力边界 → 人本长期方向”。

每项能力/路线采用“已有实现”“早期实现/探索”“长期方向”等明确文字；不把当前工程所有缺陷逐条堆到读者面前，也不抹掉关键支持边界。x86+Windows、arm64+Linux 等可作为目标环境例子，但没有验证证据就不能呈现为兼容认证列表。

App/桌面 Shell（窗口/通知表面）、命令 Shell（AutoShell）、终端（AutoTerm）分别说明。内核不是 Rust/AutoVM 的同义词，当前复用的宿主系统必须出现；Linux 独立发行形态还需现有内核、驱动和必要服务。

### 5.2 介绍 UI

首屏用平实标题和短说明。OS 主视觉为实际 desktop-hero，AI 主视觉为“应用/Agent → client → daemon → provider”关系示意；不能用假代码/假输出作为成功演示。状态表可在窄屏转换纵向说明，允许局部表格滚动，不遮挡正文。

现有 OSHero 中 Hardware 直连 Runtime 的图层与全英文 labels 不再用于代表当前完整独立系统；AIHero 的视觉也需检查。可参数化替换或由新组件承接，双语层名同源。复用 715 主题与图片放大能力，按钮明确命名为“当前桌面”“架构资料”“应用介绍”，去除无限模型/促销统计和“几分钟发布”的断言。

### 5.3 相关页同步与路由

保留既有 `/os`、`/ai`、`/autoos/`、`/ui-desktop` 和 ZH 路由。AutoOS 页只介绍当前产品，纠正虚拟桌面“未来才做”、全 Auto/固定平台覆盖等旧表述；ui-desktop 纠正后端现状。

首页仅改 OS/AI 两张卡片描述；release-v05.ts 仅改 OS/AI/LaOS 相关分层、现状/未来路线和误导的数量/成熟度（若无法追溯），保留 715 的产品展示、真实图与其余完整发布内容。两语言同时改。不得重写无关发布历史或把未来设想写成 v0.5 已交付。

资料链接逐一核实目标，入口名称区分“历史设计”和“当前项目资料”。原有 /docs/os、/docs/ai 没有输出正文时替换为明确存在的目标，不制造跳到 404 的 CTA。站内搜索应检索得到新介绍，不能将全部内容藏于图片、不可索引 JS 或默认折叠区。

### 5.4 独立历史与展望文章（r2）

采用普通 VitePress 文档布局（默认 layout），正文 H2/H3、阅读宽度与页内目录，避免长文套用 landing 的卡片/大数字。新增四个网站作者源：

- `website/articles/autoos-history.md`、`website/zh/articles/autoos-history.md`（新）。
- `website/articles/auto-ai-history.md`、`website/zh/articles/auto-ai-history.md`（新）。

从 OS/AI 总览的清楚命名入口进入对应文章，文章返回总览、两篇互链，EN/ZH 切换保持主题。四份 proposals 稿为 r2 文字基线，执行时逐篇复制与整合到专用工作树；网站正式作者源不放入生成/ignored docs。

文章叙述“当时的问题 → 为什么改变 → 具体变化与证据 → 仍需继续做的事”，避免编年条目替代解释。文章覆盖：

- AutoOS：早期 LaOS、UI 到桌面、WM-as-App/宿主合成、OS over OS、Linux 初期接缝、语言/产品分工、解释/编译与协议、三种未来形态、人本知识工作方向。
- AI：AutoForge 前史与共享服务拆分、Profession/Role、编排下沉与旧引擎退役、Auto 化、回合事件/工具数据/压缩、命令执行与模型选择、Plan/Specs/人工门及更广知识协作方向。

关键历史节点附近附一级项目源链接，显著标注资料截止日；事实、历史状态与作者归纳区分。编排阶段数不等于 Agent 数量，会话压缩不等于长期个人知识库，自有内核与独立发行版仍是未来方向。所有链接应在合入时可公开访问；不可访问的 local-only commit 改用已发布的等价设计/归档计划，不把失效引用悄悄删除。

### 规范增量

这里只提案；review 对账，merge 才落 Canonical Specs/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/website/design/os-ai-introduction.md | 缺乏统一叙事 → LaOS/OS over OS/AI+Lang+OS 连续关系与三种形态、现状状态规则 | 忠实用户理念，避免未来能力误写 | AC-01, AC-02, AC-03 |
| SD-02 | modify | docs/specs/website/project.md | OS/AI 零散介绍 → 页面职责、同源双语介绍与来源审计 | 维护入口与详细说明一致 | AC-04, AC-05, AC-06 |
| SD-03 | modify | docs/specs/website/design/ui-presentation.md | 宣传 hero/数字墙 → 实图/关系示意/文字状态，SSR/search 与窄屏要求 | 介绍可读、可核实 | AC-07, AC-08, AC-09, AC-12 |
| SD-04 | add | docs/specs/website/design/os-ai-introduction.md | 只有概括页 → 两篇双语独立叙事长文、来源/时点与总览互链 | 保留可追溯的架构历史与阅读入口 | AC-10, AC-11, AC-12 |

无组件退役，仅主题展示可替换；supersedes_spec_components 留空。

## 6. 测试设计

所有实际网站修改/构建在 719 worktree，计划状态在主检出。Category A，不运行 Rust 门禁。

- `npm run build`（cwd=worktree website）：生成与 SSG 成功，OS/AI 的标题/正文/状态能在 HTML 读取。仅修改文字不写镜像文案的逐字测试。
- 新增 `website/tests/os-ai-introduction.spec.ts` 验证真实行为：十二条 EN/ZH 路由（八个原介绍页 + 四个文章页）正文非 404、语言切换保留页、CTA 到真实正文、图片放大键盘开关、站内搜索命中新介绍，以及新状态/关系图的可访问名称。
- 固定 locale en-US，以空闲独占端口（例 `$env:AUTO_WEBSITE_TEST_PORT='4199'; $env:CI='1'`）运行 `npx playwright test tests/os-ai-introduction.spec.ts tests/site-ui.spec.ts`；不复用他人服务，strictPort 端口冲突直接失败，退出只清自己进程树。
- 360/390/768/1024/1440 × light/dark 的正文/表格几何与移动交互检查；原四个主要页面与两篇文章 EN/ZH × 390/1440 × light/dark 共 48 份图，明确 goto 实际 URL、locale、h1/body 非 404。新脚本 `website/scripts/p719-shot.cjs`（新），复用已经修正的截图流程，不复制 715 重复 /zh 的错误。
- 完成后若修改主题共享控件/全局样式，扩展至 website 全套 e2e；仅局部正文组件则以上 scoped gate 加 homepage/v05 冒烟即可。报告精确记载采用范围与原因。
- 人工内容审计以主张→来源表检查未来/现状与双语语义；文章关键日期与转折具备设计或提交/计划证据，引用在本地核验，落地前再验证公开源链接；区分 document move、plan archive、设计提出与实现日期。搜索/构建通过不能替代事实审阅。没有对跨仓内核、平台、Agent 实际能力做新的运行验收。

## 7. 验收标准

- [x] **AC-01 理念连续**：两语言 OS 页清楚解释 LaOS 的实现出发点、OS over OS 的产品视角以及 AI+Lang+OS 的人本长期方向，涵盖系统部件、平台适配与对外接口目标。验证：介绍稿/实际 SSR 文案人工对账。
- [x] **AC-02 形态与状态**：宿主虚拟桌面、现有内核的独立系统、自有内核分别说明；独立发行版/OpenHarmony/自有内核与 native 自实现均不写成当前交付；没有 Pop!_OS/COSMIC 必然路线或所有平台已支持的断言。验证：主张→来源/状态审计。
- [x] **AC-03 AI 与实现分工**：UI/窗口/命令 Shell/终端/服务区分；auto-lang/auto-os 两轴、Auto/Rust 基础与 AI 应用/Agent/client/daemon 层正确；当前知识工具与未来跨应用协作分别说明。验证：代码/Specs 与中英人工审读。
- [x] **AC-04 双语一致**：OS/AI 正文、视觉层名、状态、CTA、AutoOS/desktop 页与首页摘要两语言语义一致，不出现“∞ 模型”、无依据的全 Auto/唯一全部应用网关、假运行输出；功能真实性保留来源。验证：双语 diff + 主张表 + 图片说明。
- [x] **AC-05 配套更新**：AutoOS 页反映已有桌面/app 集成，desktop 页不再“即将推出”；首页两卡与 v05 的相关 OS/AI 表述一致，其余发布内容与截图契约保留。验证：八路由 + home/v05 内容审读与冒烟。
- [x] **AC-06 入口有效**：旧路由与语言切换保持；所有新/改 CTA 有正文/真实资料，/docs/os、/docs/ai 若缺失已修正；相关历史设计资料明确标识。验证：链接清单、e2e 正文/URL。
- [x] **AC-07 介绍可发现**：新介绍默认可读、SSR 有正文、local search 能检索核心内容；不是只有 hero 图片或数字墙。验证：生产 HTML 与搜索 e2e。
- [x] **AC-08 视觉可用**：五宽度两主题无页面级横向溢出、乱码/标题遮挡；状态有文字、移动目标≥44px，截图放大可键盘操作；48 份图均对应真实正文。验证：几何/交互断言与人工看图。
- [x] **AC-09 交付守约**：build 与适用 website 门禁通过，无新增警告，无跨仓代码改动/新 runtime 能力承诺；r1 文案与最终增量有对账，独立 review 检查遗漏/延后/绕过。验证：diff、执行报告、revision-bound review。

- [x] **AC-10 历史可追溯**：两篇文章包含明确的问题/动因、变化、历史证据、当前结果和未来方向；关键日期与转折有主张→来源对账，公开引用可访问。没有将重命名/归档日期当功能诞生，或将旧注释当当前架构。验证：研究收据、文案审读、提交/源链接核验。
- [x] **AC-11 长文入口完整**：新增四个文章路由有真实正文；对应 OS/AI 总览入口、文章返回总览和两篇互链均有效，EN/ZH 切换保留主题。验证：十二路由 e2e 与导航行为。
- [x] **AC-12 长文双语可读**：两篇 EN/ZH 的事件、解释与状态语义一致；正文使用默认阅读排版、可用目录/锚点、SSR 与搜索，五宽度两主题不溢出；48 图包含四个文章页。验证：双语人工对账、生产 HTML/search/几何和人工看图。

## 8. 执行步骤

- [x] **T-01 事实与入口复核**（无依赖）。按 §4 source/hash 和介绍稿核对执行时最新已合入代码；生成实际 docs，核实旧 CTA，固定主张→来源→当前状态表。新建 `docs/reports/p719-os-ai-introduction.md`（新），记录 718 共享文件改动及资料入口。产出经核对的文字/链接表，不自行改变理念；AC-01/02/03/06，SD-01/02。
- [x] **T-02 OS 介绍与桌面页**（T-01）。新增 theme/data/os-ai-introduction.ts、components/OSIntroduction.vue；修改 website/os.md、zh/os.md、autoos/index.md、zh/autoos/index.md。更新理念/组成/形态/状态与真实截图，适配现有 OSHero.vue 或由新组件替换其使用。build 静态正文通过；AC-01/02/03/04/07，SD-01/02/03。
- [x] **T-03 AI 介绍**（T-01/T-02）。新增 AIIntroduction.vue，共享双语 data，修改 website/ai.md、zh/ai.md 及必要的 AIHero.vue。删除缺源代码/终端输出和数字墙，呈现当前接入与长期方向。build + 关系图/正文检查；AC-03/04/07，SD-01/02/03。
- [x] **T-07 AutoOS 长文落地**（T-01/T-02）。已交付的 proposals/719-autoos-history.zh.md 与 .en.md 整合到新 website/articles/autoos-history.md、zh/articles/autoos-history.md；核对 LaOS/桌面/Linux/产品分工/渲染与展望的来源，正文默认文档布局。HTML 正文、源链接与双语审读通过；AC-01/02/03/10/11/12，SD-01/04。
- [x] **T-08 AI 长文落地**（T-01/T-03）。已交付的 proposals/719-auto-ai-history.zh.md 与 .en.md 整合到新 website/articles/auto-ai-history.md、zh/articles/auto-ai-history.md；根据研究收据核对 Role/Pipeline/Auto 化/事件/压缩/命令/模型/Plan 的历史与当前。完整文字及引用、前史/当前语义、默认阅读布局通过；AC-03/04/10/11/12，SD-01/04。
- [x] **T-04 配套同步**（T-02/T-03/T-07/T-08）。修改 website/ui-desktop.md、zh/ui-desktop.md、index.md、zh/index.md 与 theme/data/release-v05.ts 的相关 OS/AI 字段；使用真实入口和准确后端/阶段；OS/AI 概括页增加对应长文入口，AutoOS 桌面页可补充历史文章链接。保留无关发布内容与导航契约。链接检查及 home/v05 冒烟通过；AC-02/04/05/06/11，SD-02/04。
- [x] **T-05 验证与视觉复核**（T-04）。新增 tests/os-ai-introduction.spec.ts、scripts/p719-shot.cjs，补齐 §6 scoped/full gate 与 48 图；报告目录 `docs/reports/p719-website-os-ai/`（新）。人工查看所有正文与双语稿，修正发现的问题后只重验受影响范围。AC-01..AC-12，SD-01..SD-04。
- [x] **T-06 完成交接**（T-05）。提交实现，记录 code revision 与各 AC 证据、delta 对账，状态 execution_done；按 /auto-plan:review 做独立验收、债务/遗漏扫描。review 通过后才可 /auto-plan:merge 进行 Spec 回写/索引/归档与 guarded worktree 清理。AC-09 与全部 SD；无授权不得发布。

执行时先在专用树断言路径/分支。718 若仍在执行，只依赖稳定已合入基线，不复制其 WIP 或覆盖其 index/style 变更；719 完成前回核相关文件并解决交叉改动。

## 9. 复审记录

### r1 规划与文案交接（历史记录，非实现复审）

- stage: new
- plan_id: PLAN-719
- plan_revision: 1
- outcome: pass（草案和双语介绍稿可审阅；网站尚未实现）
- next: work（确认本具体内容稿与范围后进入 /auto-plan:work）
- changed_tasks: T-01..T-06
- changed_acceptance: AC-01..AC-09
- spec_delta: SD-01..SD-03
- evidence: proposals/719-os-ai-copy.md；§4 项目版本与事实表。
- 草案自查：全部 AC 有任务覆盖；现状与长期方向有来源区分；新旧路由与 718 交叉文件有处理责任；没有写 canonical Specs、网站代码或跨仓源。

### r2 文章范围交接（历史，非实现复审）

- stage: new
- plan_id: PLAN-719
- plan_revision: 2
- outcome: pass（完成历史研究与四份具体文章稿；可供网站落地）
- next: work（沿既有计划确认流程进入实施，不重复询问文章是否需要撰写）
- changed_tasks: T-01/T-04/T-05/T-06，新增 T-07/T-08；既有 T/AC 身份保留。
- changed_acceptance: 新增 AC-10/AC-11/AC-12；既有 AC-01..AC-09 保留。
- spec_delta: 新增 SD-04，SD-03 补充长文阅读验收。
- evidence: proposals/719-autoos-history.{zh,en}.md、719-auto-ai-history.{zh,en}.md、719-history-sources.md。
- superseded: r1 交接仅为 6 任务/9 验收的历史范围；r2 为 8 任务/12 验收/4 delta，没有覆盖或删除 r1 记录。
- implementation_status: drafting，current_step=0；本次交付的是已写完的文章稿和修改后的执行契约，尚未声称网站路由/构建通过。

### 实现复审

待实现交付后填写，不能以本节规划 pass 代替独立 review。

### 2026-10-01 实施启动

用户明确“请你实施计划719”，批准 r2 的8任务/12验收范围。复用原 plan-719-dev 工作树并同步主线 a19be0e845c767c417a69785ee0fa2c5c4b3b323，继承718已交付主题与720修正六图。语义契约不变；截图源改用已授权720的 desktop-showcase 六图（旧 v05 原图仅在相关设置/应用说明保留），不得恢复缺快捷方式的旧桌面图。采用可索引 Markdown 正文 + 同源双语介绍/架构组件的允许组合，长文使用默认阅读版式。auto-os/auto-ai只读基线仍为§4.3，musk最新653b0879fc29b9c13009a508f368f51d831653dd，当前plan-flow实际Spec已复核。

## 10. 待澄清事项

- 本次文章范围已获用户明确追加授权，研究与四份稿件已完成；无需重复确认是否写文章。2026-10-01 用户已明确要求实施计划 719，r2 全范围授权已到位。
- 未核实的新平台状态、资料链接与组件边界由 T-01 查证；若只能得到架构目标，文案保持目标/探索，不伪造兼容性验证。
- 若实际需要修改 OS/AI 运行时或改动 718 的验收范围，进入 needs_replan，不能借内容改版扩大实施面。

### 2026-10-01 实现交接

stage: work | plan_id: PLAN-719 | plan_revision: 2 | outcome: execution_done | code_commit: 3c6ea4c9ba753b7bd6ebf0b43dab3bc28174df20 | evidence_commit: 4c0fc7f7c3f784e1d4e2feb311dbcec32f18957d | next: review。

8 项任务已实施；稳定构建 142.09s，整站125 pass/5.8m/零重试，120几何与48最终图、31公开引用通过。报告 docs/reports/p719-os-ai-introduction.md 与相邻 manifest/source JSON；Category A 零 Cargo/docs_gen。图集标题丢失已修正，重建与测试重叠的历史轮次作废，未用旧图或旧通过覆盖最终实现。

### PLAN-719:r2 实现复审（2026-10-01）

stage: review | plan_id: PLAN-719 | plan_revision: 2 | outcome: pass | reviewed_commit: 4c0fc7f7c3f784e1d4e2feb311dbcec32f18957d | base_commit: a19be0e845c767c417a69785ee0fa2c5c4b3b323 | next: merge。

独立性限制：在实施会话中执行独立复核步骤，没有另建会话/代理，不声称人员独立。以提交 diff、生产 HTML、实际行为、来源状态、截图 manifest 与冻结 Spec 重建结论，不用已勾选任务充当证据。实施树已提交且 clean；8任务/12验收完整，没有未批准延后/遗漏/workaround、新增运行时或跨仓改动。

review evidence：docs/reports/p719-os-ai-introduction.md 的逐 AC/T 对账及 Spec SHA256；p719-public-sources.json（31/31）；p719-os-ai-introduction/manifest.json（120 checks/48 PNG，2026-10-01T07:40:20.473Z）和48图/8联系图；稳定 build exit0/142.09s；整站125 pass/5.8m/零重试。复核时实际清点12路由、48PNG与31成功源；code_commit 3c6ea4c9ba753b7bd6ebf0b43dab3bc28174df20→reviewed_commit 的 website 与冻结 Spec diff 为空，因此复用该稳定产物的测试/看图证据，无无谓重跑。git diff --check、范围检查与页面 Vue/JS 警告检查通过；历史失败与修正保留报告，没有覆盖。

acceptance_results：AC-01 pass（LaOS 连续关系/接口）；AC-02 pass（三形态/未来边界）；AC-03 pass（两轴/AI层次/Rust基础）；AC-04 pass（双语/实图/无伪输出）；AC-05 pass（配套home/v05/desktop/AutoOS）；AC-06 pass（CTA/locale/公开源）；AC-07 pass（SSR/search）；AC-08 pass（120几何/48图/键盘/44px）；AC-09 pass（build/125e2e/范围健康/复核）；AC-10 pass（历史来源/两处公开等价替换）；AC-11 pass（四文路线/互链/切换）；AC-12 pass（默认阅读/目录/SSR/search/双语图）。逐项实际方法见报告表，非镜像逐字文案测试。

dependency_revisions：auto-os 73d02b50f535543bd635c6f6f49dad6835efff45；auto-ai 5a50a55844d7aa3523b593f21ba0fb03d18eac48；auto-musk 6e6b6cc847639805c796e0e5a7cc6091b96f4319（前后 plan-flow Spec diff 为空）；book d7a71a7fb1fa42ddd26d6cec859715ed98161f7a。Spec inputs 见报告基线；SD-01/04 新 os-ai-introduction.md SHA256=93e021ec16ac2fed2e78876eb5aeccba9d563d2ad6ee37a02a0972256ed46fae；SD-02 project.md=15003f95fd1b19c62a38500430fe413c060c9bbb8646fce52982463dfdcf275a；SD-03 ui-presentation.md=fe0d8d50cf3167217707c02fd6605d3fec468215f06230a296ac8480e7935fef。三项与实际文件 hash 全等；SD-04 共用 SD-01 文件。plans.md 仅标准模块跟踪，并修正已归档720的旧 active 链接，不引入新行为。

spec-impact：new_spec_components=[docs/specs/website/design/os-ai-introduction.md]；supersedes_spec_components=[]，无规范退役；touched_goals=[]，继续既有网站目标，无新增 goal ID。findings：新增实现债务=0；归档后置前提为 store-mediated ledger writer，8080 当前连接拒绝，未直接写 .autoos/specs.json。代码/Canonical Specs 复核通过可先落地，ledger 未验证不得提前归档或清理工作树。