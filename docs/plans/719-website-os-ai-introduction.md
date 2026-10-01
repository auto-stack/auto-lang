---
plan_id: PLAN-719
status: drafting
feature_name: Website OS 与 AI 介绍重组
author: [agent]
created_at: 2026-10-01
updated_at: 2026-10-01
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: []
new_spec_components: [docs/specs/website/design/os-ai-introduction.md]
touched_goals: []
affects: [website]
---

# [PLAN-719] Website OS 与 AI 介绍重组

## 0. 变更摘要

将 OS/AI 相关介绍从零散宣传页重组为基于架构与现状的双语说明：**LaOS 是实现理念，OS over OS 是当前产品形态，AI + Lang + OS 是长期面向人的使用方向**。三者承接而非互相替换。

具体可审阅文案在 [中英文介绍稿](proposals/719-os-ai-copy.md)，包含 OS/AI 完整正文、页面分工、配套首屏和首页卡片摘要。本计划将其落到 VitePress 网站，核对相关 AutoOS/桌面 UI/发布页内容，清除过时路线和缺证据的统计/示例。

本计划不与执行中的 PLAN-718 合并。718 负责学习入口与阅读体验；719 负责 OS/AI 介绍，实施时在独立 worktree 中吸收实际已合入的主题状态。

## 1. 目标

- 准确解释 LaOS 的组件化系统理念、跨生态适配与一致接口目标，以及它和 OS over OS 的连续关系。
- 区分宿主虚拟桌面、复用现有内核的独立系统、自有内核三种运行/发展形态。
- 描述 AI 深度参与项目开发、共享模型服务与 Agent 工作工具的现状，并呈现面向知识管理、工作生活的人本长期方向。
- 从真实项目文件说明 auto-lang（语言/框架）与 auto-os（桌面/应用/产品集成）的分工；明确当前 Auto + Rust 实现基础。
- 中英文介绍、首屏图示、桌面页、首页入口与 v0.5 相关摘要一致，读者可查到实际能力与资料入口。

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

新增 `website/.vitepress/theme/data/os-ai-introduction.ts`（新）保存两语言的介绍、状态与架构关系；新增 `OSIntroduction.vue`、`AIIntroduction.vue`（新）用于相应总览页。AutoOS 桌面页和配套页面只复用有关摘要/状态，不再复制完整理念。实施可采用等价的可索引 Markdown 正文 + 数据组件组合，但需记录取舍，不能让首屏与正文成为相互矛盾的数据源。

用介绍式首屏、实际图像、简明架构图和带文字状态的表格代替大数字墙。架构图示不是截图；宿主/内核和渲染/合成设施必须在图中出现，不能形成“Auto 直接在所有硬件上运行”的误读。

## 3. 技术栈

现有 VitePress 1.6.x、Vue 3、主题 CSS 与 Playwright。不加 UI 依赖，不升级框架。真实桌面图复用现有 `website/public/v05/desktop-hero.png`、`desktop-multiwindow.png`、`desktop-light.png` 和设置/应用截图，由既有 `EvidenceImage.vue` 或 `ScreenshotGallery.vue` 展示；图中能力、来源与截图时间按已有材料说明。

## 4. 需求分析与背景调查

### 4.1 用户授权与约束

2026-10-01 用户要求：结合 LaOS → OS over OS → AI + Lang + OS 的概述、现有网站以及 auto-lang/auto-os 现状，重新组织 OS/AI 板块，减少宣传、加强介绍；明确自有内核未做、未来会做，未来也可能探索语言底座自实现 native。

这授权本次事实调查、介绍稿和具体改版计划。按仓库 AGENTS.md L1 的“present for confirmation before executing”，将可审阅计划与完整双语稿提交确认后再实施；没有把新的内容范围加入已经 executing 的 718。

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

### 规范增量

这里只提案；review 对账，merge 才落 Canonical Specs/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/website/design/os-ai-introduction.md | 缺乏统一叙事 → LaOS/OS over OS/AI+Lang+OS 连续关系与三种形态、现状状态规则 | 忠实用户理念，避免未来能力误写 | AC-01, AC-02, AC-03 |
| SD-02 | modify | docs/specs/website/project.md | OS/AI 零散介绍 → 页面职责、同源双语介绍与来源审计 | 维护入口与详细说明一致 | AC-04, AC-05, AC-06 |
| SD-03 | modify | docs/specs/website/design/ui-presentation.md | 宣传 hero/数字墙 → 实图/关系示意/文字状态，SSR/search 与窄屏要求 | 介绍可读、可核实 | AC-07, AC-08, AC-09 |

无组件退役，仅主题展示可替换；supersedes_spec_components 留空。

## 6. 测试设计

所有实际网站修改/构建在 719 worktree，计划状态在主检出。Category A，不运行 Rust 门禁。

- `npm run build`（cwd=worktree website）：生成与 SSG 成功，OS/AI 的标题/正文/状态能在 HTML 读取。仅修改文字不写镜像文案的逐字测试。
- 新增 `website/tests/os-ai-introduction.spec.ts` 验证真实行为：八条 EN/ZH 路由正文非 404、语言切换保留页、CTA 到真实正文、图片放大键盘开关、站内搜索命中新介绍，以及新状态/关系图的可访问名称。
- 固定 locale en-US，以空闲独占端口（例 `$env:AUTO_WEBSITE_TEST_PORT='4199'; $env:CI='1'`）运行 `npx playwright test tests/os-ai-introduction.spec.ts tests/site-ui.spec.ts`；不复用他人服务，strictPort 端口冲突直接失败，退出只清自己进程树。
- 360/390/768/1024/1440 × light/dark 的正文/表格几何与移动交互检查；主要四页 EN/ZH × 390/1440 × light/dark 共 32 份图，明确 goto 实际 URL、locale、h1/body 非 404。新脚本 `website/scripts/p719-shot.cjs`（新），复用已经修正的截图流程，不复制 715 重复 /zh 的错误。
- 完成后若修改主题共享控件/全局样式，扩展至 website 全套 e2e；仅局部正文组件则以上 scoped gate 加 homepage/v05 冒烟即可。报告精确记载采用范围与原因。
- 人工内容审计以主张→来源表检查未来/现状与双语语义；搜索/构建通过不能替代事实审阅。没有对跨仓内核、平台、Agent 实际能力做新的运行验收。

## 7. 验收标准

- [ ] **AC-01 理念连续**：两语言 OS 页清楚解释 LaOS 的实现出发点、OS over OS 的产品视角以及 AI+Lang+OS 的人本长期方向，涵盖系统部件、平台适配与对外接口目标。验证：介绍稿/实际 SSR 文案人工对账。
- [ ] **AC-02 形态与状态**：宿主虚拟桌面、现有内核的独立系统、自有内核分别说明；独立发行版/OpenHarmony/自有内核与 native 自实现均不写成当前交付；没有 Pop!_OS/COSMIC 必然路线或所有平台已支持的断言。验证：主张→来源/状态审计。
- [ ] **AC-03 AI 与实现分工**：UI/窗口/命令 Shell/终端/服务区分；auto-lang/auto-os 两轴、Auto/Rust 基础与 AI 应用/Agent/client/daemon 层正确；当前知识工具与未来跨应用协作分别说明。验证：代码/Specs 与中英人工审读。
- [ ] **AC-04 双语一致**：OS/AI 正文、视觉层名、状态、CTA、AutoOS/desktop 页与首页摘要两语言语义一致，不出现“∞ 模型”、无依据的全 Auto/唯一全部应用网关、假运行输出；功能真实性保留来源。验证：双语 diff + 主张表 + 图片说明。
- [ ] **AC-05 配套更新**：AutoOS 页反映已有桌面/app 集成，desktop 页不再“即将推出”；首页两卡与 v05 的相关 OS/AI 表述一致，其余发布内容与截图契约保留。验证：八路由 + home/v05 内容审读与冒烟。
- [ ] **AC-06 入口有效**：旧路由与语言切换保持；所有新/改 CTA 有正文/真实资料，/docs/os、/docs/ai 若缺失已修正；相关历史设计资料明确标识。验证：链接清单、e2e 正文/URL。
- [ ] **AC-07 介绍可发现**：新介绍默认可读、SSR 有正文、local search 能检索核心内容；不是只有 hero 图片或数字墙。验证：生产 HTML 与搜索 e2e。
- [ ] **AC-08 视觉可用**：五宽度两主题无页面级横向溢出、乱码/标题遮挡；状态有文字、移动目标≥44px，截图放大可键盘操作；32 份图均对应真实正文。验证：几何/交互断言与人工看图。
- [ ] **AC-09 交付守约**：build 与适用 website 门禁通过，无新增警告，无跨仓代码改动/新 runtime 能力承诺；r1 文案与最终增量有对账，独立 review 检查遗漏/延后/绕过。验证：diff、执行报告、revision-bound review。

## 8. 执行步骤

- [ ] **T-01 事实与入口复核**（无依赖）。按 §4 source/hash 和介绍稿核对执行时最新已合入代码；生成实际 docs，核实旧 CTA，固定主张→来源→当前状态表。新建 `docs/reports/p719-os-ai-introduction.md`（新），记录 718 共享文件改动及资料入口。产出经核对的文字/链接表，不自行改变理念；AC-01/02/03/06，SD-01/02。
- [ ] **T-02 OS 介绍与桌面页**（T-01）。新增 theme/data/os-ai-introduction.ts、components/OSIntroduction.vue；修改 website/os.md、zh/os.md、autoos/index.md、zh/autoos/index.md。更新理念/组成/形态/状态与真实截图，适配现有 OSHero.vue 或由新组件替换其使用。build 静态正文通过；AC-01/02/03/04/07，SD-01/02/03。
- [ ] **T-03 AI 介绍**（T-01/T-02）。新增 AIIntroduction.vue，共享双语 data，修改 website/ai.md、zh/ai.md 及必要的 AIHero.vue。删除缺源代码/终端输出和数字墙，呈现当前接入与长期方向。build + 关系图/正文检查；AC-03/04/07，SD-01/02/03。
- [ ] **T-04 配套同步**（T-02/T-03）。修改 website/ui-desktop.md、zh/ui-desktop.md、index.md、zh/index.md 与 theme/data/release-v05.ts 的相关 OS/AI 字段；使用真实入口和准确后端/阶段。保留无关发布内容与导航契约。链接检查及 home/v05 冒烟通过；AC-02/04/05/06，SD-02。
- [ ] **T-05 验证与视觉复核**（T-04）。新增 tests/os-ai-introduction.spec.ts、scripts/p719-shot.cjs，补齐 §6 scoped/full gate 与 32 图；报告目录 `docs/reports/p719-website-os-ai/`（新）。人工查看所有正文与双语稿，修正发现的问题后只重验受影响范围。AC-01..AC-09，SD-01..SD-03。
- [ ] **T-06 完成交接**（T-05）。提交实现，记录 code revision 与各 AC 证据、delta 对账，状态 execution_done；按 /auto-plan:review 做独立验收、债务/遗漏扫描。review 通过后才可 /auto-plan:merge 进行 Spec 回写/索引/归档与 guarded worktree 清理。AC-09 与全部 SD；无授权不得发布。

执行时先在专用树断言路径/分支。718 若仍在执行，只依赖稳定已合入基线，不复制其 WIP 或覆盖其 index/style 变更；719 完成前回核相关文件并解决交叉改动。

## 9. 复审记录

### 规划与文案交接（非实现复审）

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

### 实现复审

待实现交付后填写，不能以本节规划 pass 代替独立 review。

## 10. 待澄清事项

- 待用户确认具体双语稿与页面分工；缘由是 AGENTS.md L1 的先展示计划再执行要求。本阶段调查和可审阅文案已完成，不再询问用户已给出的理念或技术栈。
- 未核实的新平台状态、资料链接与组件边界由 T-01 查证；若只能得到架构目标，文案保持目标/探索，不伪造兼容性验证。
- 若实际需要修改 OS/AI 运行时或改动 718 的验收范围，进入 needs_replan，不能借内容改版扩大实施面。
