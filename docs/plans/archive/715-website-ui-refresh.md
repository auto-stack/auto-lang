---
plan_id: PLAN-715
status: archived
feature_name: VitePress 全站 UI 与 v0.5 发布展示改进
author: [Codex]
created_at: 2026-09-30
updated_at: 2026-09-30
plan_revision: 1
work_commit: 675306b71（worktree plan-715-dev，基线 ff32d7004）
reviewed_commit: 675306b71
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: [docs/specs/website/design/ui-presentation.md]
touched_goals: []
affects: [website]
current_step: 7
total_steps: 8
---

# [PLAN-715] website-ui-refresh

## 0. 变更摘要

在现有 **VitePress + Vue 3** 官网实施四个方向：①统一双语导航、当前位置与搜索；②首页用代码和真实应用展示工作方式；③统一全站排版、配色、留白与响应式；④复用截图、示例与应用页骨架。v0.5 发布页是重点落地面，同步重排首屏、章节顺序、理念区与截图展示。

保留刚完成的发布内容更新、能力边界和数据来源说明，不重新追溯历史。交付中英双语页面、共享组件、浏览器检查与截图证据。

## 1. 目标

- G-01：读者很快看到代码、实际产品和可操作入口。
- G-02：导航、语言、搜索、当前位置、移动菜单一致且易用。
- G-03：保留紫色品牌及深浅主题，改善文字层级和阅读节奏，修复手机首屏裁切。
- G-04：应用及发布页共用展示方式，减少中英布局分叉。

范围：auto-lang 仓 website/、本计划、验证报告及后续 website Spec 沉淀。共享样式覆盖宣传页、文档、书籍、Playground；重排首页、v0.5 和应用专题。独立 Gallery SPA 只验证实际入口和返回站点路径，不改其界面或编译 bundle。

边界：保持既有 URL、核查事实、应用状态和来源说明；不改 Rust/VM/后端协议/跨仓应用，不新建不存在的 AutoEdit 产品页，不换框架或升级 VitePress 主版本，不部署线上站点。使用已有入库实图，不生成假截图。其他会话文件不纳入本计划。

## 2. 架构方案

沿用 DefaultTheme 扩展及 website/.vitepress/theme/index.ts 的 Layout 包装，用 Vue 组件、CSS 变量和 Markdown 组合页面。

| 层 | 设计 | 约束 |
|---|---|---|
| 外壳 | UnifiedNavbar 接共享双语入口、当前位置及 VitePress 本地搜索 | 文档侧栏、移动目录、既有 SPA 完整导航保全 |
| 视觉 | style.css/landing.css 管宽度、文字、间距、圆角、边框、主题 | 宣传规则限定在 landing-page，不污染文档 |
| 首页 | 轻量源码/结果/Rust 标签预览，操作时才调用既有 API | 不自动加载 CodeMirror，不混淆预置和实时结果 |
| 发布 | 双语共享展示组件、实图首屏、章节导航、理念展开 | 先列内容映射再迁移，更新内容不丢项 |
| 应用 | 共享布局/卡片和截图画廊，沿用 EvidenceImage/AutoShellPreview | ash 原生主图保留，裁切预览可打开完整图 |

实施工作区：D:/autostack/.wt/lang-715/auto-lang；分支 plan-715-dev。代码/构建/测试只在专用 worktree，簿记在主检出；禁止 junction/symlink。

## 3. 技术栈

VitePress 1.6 系列（package 声明 ^1.6.3，调查时安装 1.6.4，以 lockfile 为准）、Vue 3/TypeScript、现有 Tailwind/CSS 变量/lucide-vue-next，优先原生 details/dialog。搜索复用 VitePress local provider；接口复用同源 /api；Playwright 验证。无 Rust 源码修改，遵守 Category A，不执行 Cargo 测试或 docs_gen。

## 4. 需求分析与背景调查

### 4.1 授权与依赖

- 用户明确网站使用 VitePress，本次授权：“OK，请你建立计划实施你推荐的4个方向的修改。”四方向及 v0.5 重点范围已同意；没有部署、跨仓修改或预算授权。
- 当前为 auto-plan:new 合同起草，revision 1 在此交付供审阅，next=work；起草检查不代表实现验收。
- 调查主检出基线 cfcf07d3c。读取 docs/specs/overview.md、docs/specs/website/project.md 及实际组件、页面、测试。website Spec 对技术栈、内容预处理、Playground、ash 实图有效，缺全站导航/排版/发布展示规则，由本计划 SD-01..06 补齐。
- PLAN-713 r3（reviewed）已在主检出形成 AutoShell 成果，复用并保全其契约，不覆盖或复用该 Plan。PLAN-711 运行时线无实现依赖。
- 取号前 .next-id=714，但 docs/plans/archive/714-treesitter-first-batch-survey.md 及同号 worktree 已占用714。排他锁内扫描活动/归档、纠正到715、执行 scripts/new-plan.sh website-ui-refresh；唯一 ID715，下一号716。714 状态历史不在此处理。
- 主检出有他会话未跟踪调试文件/截图，不修改、删除或提交。

### 4.2 观察

1. UnifiedNavbar 硬编码英文及无 /zh 的入口；en.ts/zh.ts 另有 nav，默认 VPNav 隐藏，存在来源分叉。showSearch 默认false，主题/手机菜单按钮缺名称与展开反馈。
2. HomeHero 是静态 Hello。ScriptShipView 有运行/转译，但加载 CodeMirror、默认 localhost API，不能直接作为轻量首页首屏。
3. v0.5 先统计、长理念、历程，再展示桌面；源码留有开始菜单与 AutoEdit 两个待截图位，最新工具链补充必须保全。
4. 上次浏览器检查为 PLAN-713 旧构建，早于内容更新：390×844 页高约19607px，桌面章节约y6517；hero x24..358，内容x12..370，两侧裁切。这些仅是问题线索，T-01必须重测最新版，不能当作最终基准。
5. 共享 landing.css 与 scoped 样式并存，对齐和密度分叉；需检查实际级联，不只改一个选择器。
6. EvidenceImage 已有 dialog/来源；AutoShellPreview 使用 ash-01 原生图及完整图放大。apps 中 automusk/autodown/autoui 双语页存在，AutoEdit 专题不存在。
7. public/ui/{gallery,blocks,charts,a2ui,demos} 是独立 SPA，不能拼不存在的 /zh/ui/gallery 路由。
8. Playwright 固定4173且可复用服务器，该端口曾属于另一 Gallery；必须可配置独占端口并验证来源，防止错测。

### 4.3 来源版本

SHA-256 前16位：docs/specs/overview.md=bfa32c63e5c0cf89；docs/specs/website/project.md=55e0cbc54360a1ae；website/package.json=ddb24187088c180d；theme/components/UnifiedNavbar.vue=350e18101f7586fc；HomeHero.vue=87b40e5af8fc2f4b；EvidenceImage.vue=12ab528364e460a3；ScriptShipView.vue=b58412dad8781026；theme/landing.css=a0c29f02134282bf；website/zh/v05/index.md=7a820b66539c7986；website/playwright.config.ts=7f2aac880d175453。theme 路径均相对 website/.vitepress。

参考：[VitePress 搜索](https://vitepress.dev/reference/default-theme-search)、[扩展默认主题](https://vitepress.dev/guide/extending-default-theme)。在线文档默认展示更新版本；实现仅用本地1.6源码确认存在的能力，T-01记录接线方式，不据新API顺手升级。

## 5. 详细设计

### 5.1 导航、搜索、语言

顶栏按品牌首页、语言与生态（语言/Rust/Python/AI/OS）、UI（总览/平台/Gallery）、应用、文档、教程组织；右侧搜索、语言、主题和突出显示的在线体验。v0.5 保留首页 badge 和发布入口。手机使用同一数据的分组菜单。

新增 theme/data/navigation.ts 作为入口/locale/匹配/共享SPA例外的单一来源，供 UnifiedNavbar 及 en.ts/zh.ts 使用。中文链接和 Logo 留在 /zh/；语言切换保持对应路径及有效 fragment，没有镜像则明确返回目标语言的最近有效总览，不造404。共享 SPA 标注共享入口，不造中文镜像。

当前项及父组有位置反馈/aria-current；菜单按钮有名称和 aria-expanded。支持 Tab、Enter/Space、Esc、导航后关闭、Esc后焦点回触发器。主题使用 VitePress 存储与运行时状态，消除重复初始主题来源。

新增 SiteSearch.vue 封装 VitePress local search：首页/宣传/文档/书籍及手机可打开，Ctrl/Cmd+K 不抢编辑器输入。固定查询 ownership、所有权、AutoShell 应命中正确页面及 locale。T-01明确 Vue 专题正文的索引摘要方案，不能遗漏专题检索；范围不包含图片文字、独立 SPA 和 Playground 全语料。对外称站内文档与专题搜索。

不修改 node_modules、不恢复第二条默认导航、不依赖2.x API；若使用1.6内部主题入口，记录版本约束与回归检查。

### 5.2 首页工作方式

HomeHero 保留复用 props/slot，首页简介明确脚本开发、转译发布。新增 HomeDemo.vue、data/home-demo.ts：选择仓内无外部依赖的10–18行真实例子，三视图为 Auto 源码/输出/Rust 示例，并提供复制、运行、打开当前 locale Playground。T-01记录来源/hash、真实输出及对应 a2r 产物证据，不手写假转译。

初始是标明来源的预置示例，不自动请求或加载 CodeMirror。点击运行才发同源 /api/run；转译用已确认 /api/transpile 请求/响应。实时成功替换结果，失败/超时清楚反馈并保留示例，10秒截止、可取消、离页清理、按钮恢复；stderr 不冒充 stdout。无后端静态部署仍可查看/复制，预置结果不能显示为实时成功。

Playground 导入能力在T-01核验；若不存在，则明确提供复制源码＋打开页面，不造 URL 参数。首页加入 AutoMusk/AutoShell/AutoDown 实图与专题，ash必须用 AutoShellPreview 原生主图。

### 5.3 视觉与响应式

紫色用于品牌、主按钮、当前位置，保留产品强调色，减少大面积光晕/彩边。正文和实图是主要展示，卡片承载短事实/并列选择。

共享 CSS 变量管理宽度、字号、行高、留白、圆角/边框。宣传主体约1200px，文字列680–760px；正文16–18px、行高1.65–1.8；章节标题/简介同向对齐（概览可居中，长文靠左），避免宽泛 .title 污染组件和文档。

360/390/768/1024/1440px 下网格和 hero 不超过父宽，使用 min-width:0、有界轨道和合理 gutter；不能以全站 overflow-x:hidden 掩盖裁切。标题/描述/按钮边界在内容区内。文档/书籍侧栏、阅读宽度、代码局部横滚和移动目录保全，sticky offset 统一。

深浅主题无不可见文字/白块/异常弹窗；主题/菜单触控至少44×44px；reduced-motion 下内容一直可见。共享规则覆盖 EN/ZH 首页、v05、rust/python/ai/os/ui/apps/script-as-rust、ui-desktop/ui-android/ui-harmony、apps四专题及文档/书籍/Playground；逐类检查，只重排首页、发布、应用，不重写全部技术内容。

### 5.4 v0.5 重点

EN/ZH 共用新 ReleaseLanding.vue、data/release-v05.ts，保留 Markdown 路由及 metadata。先列旧章节→新章节映射，保全工具链新内容、已知边界、计数/日期/Token来源/无冻结标签等说明。

顺序：主视觉→三项亮点→桌面→AutoUI双端→旗舰应用→系统应用/语言工具链/Playground→理念及历程→展望→开始使用。首屏标题、短简介、主要体验按钮和次要说明入口、真实桌面主图；手机自然换行。统计后移到历程；理念默认每项一句话＋例子，完整论述用 details 可键盘展开且可定位。

新增 SectionNav.vue：桌面吸顶快捷导航、手机可展开目录、当前章节、原生hash直达/刷新，标题不被两层sticky遮挡。ScreenshotGallery.vue 用主图＋深浅/巡礼选择器，不自动轮播；selected反馈、键盘操作、EvidenceImage完整放大。首图 eager 且有尺寸/比例，其余 lazy。

AutoUI 用已有 kanban-web.png/kanban-desktop.png 对照同一示例，不暗示超过验证范围的全等。开始菜单用来源匹配的 desktop-launcher.png；AutoEdit无有效实图则保留文字、去掉空白大图位，不造截图或死链。

### 5.5 应用页与复用组件

新增 AppLandingLayout.vue：简介/状态/入口→实图→核心用例→开始使用→已有边界/来源。迁移 apps.md/zh/apps.md、automusk/autodown/autoui 双语专题，保留差异，不凭空增加下载/体验/能力。

AutoShellLanding仅接共享视觉/导航及必要接口，保留原生总览、F1/F2/F3、脚本切换/复制/下载、版本/平台/来源/限制。EvidenceImage扩展尺寸、加载策略、主题容器；保留Enter/Space打开、Esc/按钮关闭、焦点返回、原图及说明。

HomeDemo/CodeView/ScriptShipView统一标签、复制/加载/错误/结果样式和双语说明；必要包装新增 DemoFrame.vue，只改网站层，不动packages或后端协议。所有新组件在 website/.vitepress/theme/components/，数据在 theme/data/。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/project.md | 补共享locale导航/当前位置/SPA例外/搜索范围 | 入口一致、真实路由 | AC-01..03 |
| SD-02 | add | docs/specs/website/design/ui-presentation.md | 新增宣传/文档排版、主题/断点、offset/键盘规则 | 稳定视觉规范 | AC-04..06 |
| SD-03 | modify | docs/specs/website/project.md | 新增轻量首页演示、同源API、预置/实时与降级 | 行为可理解且诚实 | AC-07..08 |
| SD-04 | add | docs/specs/website/design/ui-presentation.md | 新增产品前置、理念展开、内容映射/章节导航 | 产品早见、内容保全 | AC-09..10 |
| SD-05 | modify | docs/specs/website/project.md | 单页图片规则推广画廊/应用骨架，保留ash原生图 | 复用不损证据 | AC-11..12 |
| SD-06 | add | docs/specs/website/design/ui-presentation.md | 新增双语/主题/断点矩阵和测试服务隔离 | 防误测/回归 | AC-13..14 |

起草不修改canonical Spec；review定稿、merge回写project/new design、website/plans.md和派生账本/索引。

## 6. 测试设计

在专用worktree验证。沿用 website/tests/spa-routes.spec.ts、playground-notes.spec.ts；新增 site-ui.spec.ts、home-demo.spec.ts，测试用户行为而非镜像实现。

- 导航/语言/Logo/当前位置/菜单，三类页面搜索＋固定双语查询，SPA直达及实际点击导航。
- 360×800、390×844、768×1024、1024×768、1440×1000；bounding box＋截图检查无文字裁切，不只看无横滚。
- v0.5主图top小于1.5倍视口高度（390×844/1440×1000，EN/ZH）；hash直达/刷新/当前章节/offset。
- 截图切换、放大、Esc、焦点返回、完整图/来源；ash标签/复制/下载回归。
- Playwright拦截 run/transpile 成功、失败、超时，验证同源、状态、10秒恢复、无初始API请求。mock只证明网站反馈，真实例子另凭仓内源码/输出/a2r证据验证。
- 深浅主题、44px触控、reduced-motion、目录/代码阅读；SPA资源/MIME、Notes Explorer、console无新增异常。

深入截图集：首页、v05、apps目录、四应用专题、语言专题、文档、书籍、Playground，EN/ZH，390/1440，深/浅；其余覆盖矩阵页做桌面/手机smoke。报告/前后截图/内容映射/素材及示例来源/API状态存 docs/reports/p715-website-ui/（新），总报告 p715-website-ui-review.md（新）。

T-01就为 website/playwright.config.ts 增加 AUTO_WEBSITE_TEST_PORT（默认4173保兼容）；本Plan显式选独占空闲端口，baseURL和webServer同源，不复用未知服务，确认来自本worktree。后续测试使用该能力。

最终命令：website中 npm ci；packages/auto-playground-vue安装声明依赖；npm run build；设置 $env:AUTO_WEBSITE_TEST_PORT='4185' 后 npm run test:e2e（占用则换空闲并记录）；仓根 git diff --check。书籍等输入用约定真实路径或复制既有生成快照，不建junction。不跑cargo t/tf/tv/tu/docs_gen，不为网站编译语言二进制；示例优先已有CLI/API/金样证据。新错误修复，已有警告如实分类记录，不把视觉检查当语言验收。

## 7. 验收标准

| ID | 可观察结果 | 验证及预期 |
|---|---|---|
| AC-01 | 双语共享导航、有效入口/Logo/切换/当前位置 | 路由矩阵；中文不意外进英文，缺镜像有效fallback，SPA无假路由 |
| AC-02 | 宣传/文档/书籍与手机可搜索 | 按钮/快捷键，ownership/所有权/AutoShell命中正确有效页面 |
| AC-03 | 控件有名称/状态且键盘可用 | Tab/Enter/Space/Esc、焦点返回、导航后菜单收起 |
| AC-04 | 文字层级/对齐一致，不污染文档 | 视觉矩阵；正文16–18px，目录/代码保持可读 |
| AC-05 | 五断点无首屏裁切/全页溢出 | bbox＋截图；主要文字/按钮在父内容区内，360px可操作 |
| AC-06 | 深浅/触控/减少动态效果可靠 | 两主题截图、44px、reduced-motion；切页不重置主题 |
| AC-07 | 首页三视图/复制及真实产品入口 | 预置标明，实图加载、跳有效专题，ash原生主图保留 |
| AC-08 | 运行/转译反馈诚实，静态可浏览 | 成功/断网/超时；同源、10秒恢复、无初始请求，真实来源另验 |
| AC-09 | 发布主图早见，章节易达 | EN/ZH390/1440主图top<1.5×视口高，hash有效且标题不遮挡 |
| AC-10 | 理念简洁、统计后移、核查内容完整 | 旧新映射逐项核验；完整理念可展开、工具链五类/边界/注记保全 |
| AC-11 | 图可选择/完整放大/关闭，无占位 | 键盘/点击/Esc/焦点、尺寸与资源；无待截图、假图或死链 |
| AC-12 | 应用共用骨架，ash行为和证据保全 | 四专题EN/ZH；脚本标签/复制/下载、原生总览/F1..3/来源完整 |
| AC-13 | 构建/e2e/隔离服务检查通过 | worktree build、全部网站e2e、diff；无新增资源/MIME/console异常 |
| AC-14 | 可复查证据与Spec delta对应 | 矩阵/前后图/映射/来源齐备，SD-01..06逐项有证据 |

## 8. 执行步骤

均初始[ ]，完成追加命令/结果、artifact和commit。组件基路径 website/.vitepress/theme/components/，数据基路径 website/.vitepress/theme/data/；未存在项均标新。

| 任务 | 依赖 | 文件/操作 | 验证与预期 | 验收 |
|---|---|---|---|---|
| T-01 [x] 基线/接线勘定 | 无 | 专用worktree；package/lock、theme/index.ts、配置、ScriptShipView、EvidenceImage；先改playwright.config.ts支持独占端口；新报告docs/reports/p715-website-ui-baseline.md，定搜索入口、真实示例、locale fallback及内容映射 | 基线build/定向e2e/截图/来源；接线决定明确，不延期搜索或演示 | AC-02/05/07/08/10/13/14 |
| T-02 [x] 导航/搜索 | T-01 | UnifiedNavbar.vue、theme/index.ts、config/shared.ts/en.ts/zh.ts、composables/useDarkMode.ts；新data/navigation.ts、SiteSearch.vue | 定向site-ui导航/搜索/locale/主题；docs/books/SPA点击及直达通过 | AC-01..03/06/13 |
| T-03 [x] 视觉/响应式 | T-01,T-02 | theme/style.css/landing.css、HomeHero/FeatureCard/ShowcaseSection；核对AIHero/OSHero和目标页scoped样式 | 五断点和深浅截图/bbox；无裁切，文档保全 | AC-04..06/13 |
| T-04 [x] 截图/应用骨架 | T-01,T-03 | EvidenceImage、AutoShellLanding必要接口；新ScreenshotGallery/AppLandingLayout；website/apps.md/zh/apps.md及automusk/autodown/autoui双语页 | site-ui画廊/键盘/专题有效入口、ash交互回归及截图 | AC-01/04..06/11/12 |
| T-05 [x] 首页演示 | T-01,T-03,T-04 | website/index.md/zh/index.md、HomeHero；新HomeDemo、data/home-demo.ts、必要DemoFrame；统一CodeView/ScriptShipView呈现 | home-demo三视图/复制/API成功失败；真实示例来源、产品入口 | AC-04..08/12 |
| T-06 [x] 发布展示 | T-01,T-03,T-04 | website/v05/index.md/zh/v05/index.md；新ReleaseLanding、data/release-v05.ts、SectionNav；产品前置/对照/理念展开/统计后移/清占位 | site-ui首图位置/hash/画廊；映射逐项核验/手机视觉 | AC-04..06/09..11 |
| T-07 [x] 全站验证/报告 | T-02..06 | 新website/tests/site-ui.spec.ts/home-demo.spec.ts；保留spa-routes/playground-notes；docs/reports/p715-website-ui-review.md与截图目录（新） | npm run build、独占端口npm run test:e2e、git diff --check、完整视觉矩阵 | AC-01..14 |
| T-08 [ ] 独立复审/交接 | T-07 | auto-plan-review核对实际diff/AC/SD；遗漏/延期/workaround扫描，候选写KNOWN-DEBT-AND-RISKS.md | revision/commit绑定复审，reviewed后交merge沉淀/归档/guard清理 | AC-01..14、SD-01..06 |

需要定向e2e的阶段可先建立相关新测试的阶段子集，不等T-07才首次验证；T-07补齐并跑完整档。独立复审为与实现分开的证据核验步骤，不假称独立代理；只有用户明确委派才启动子代理，本起草不委派。

## 9. 复审记录

### 2026-09-30 work 交接（T-01..T-07）

- stage: work | plan_id: PLAN-715 | plan_revision: 1 | outcome: pass | code_commit: 675306b71（worktree plan-715-dev @ 基线 ff32d7004）
- task_ids: T-01..T-07 完成；T-08=独立复审待执行
- evidence: docs/reports/p715-website-ui-baseline.md（T-01 勘定：搜索接线=VitePress local+合成热键法/示例 csv_delimiter 金样三件套+classify.ash 实跑/locale 前缀规则/v05 内容映射表/端口 4185）；docs/reports/p715-website-ui-review.md（T-07：build 通过、e2e 64/64、final 截图矩阵 96 行 0 问题、AC-01..14 与 SD-01..06 逐项映射）；前后对照 baseline-*.png/final-*.png
- 实测修复（记录供复审核验）：md 插槽大块内容被 markdown 块规则切断→AppLandingLayout 收敛 hero 骨架+调用压单块+顶层去缩进+泄漏回归锁；导航 1024 溢出 39px→桌面断点 1200；StatCard 长斜杠 token 393px 溢出→overflow-wrap；宿主 OS 中文 locale 泄入 e2e→playwright locale 钉定；4185 残留 preview 固化旧 dist 清单→进程 taskkill /T 清理
- 偏差登记：①计划 §5.2 的 `/api/transpile` 实为 `/api/trans`（按实际端点）；②示例 7 行而非 10–18 行（全仓唯一证据齐全族，真实性优先）；③apps.md 目录页保持共享 landing.css 骨架（目录页≠单应用骨架）；④DemoFrame 未抽取（HomeDemo 自包含无第二使用方）；⑤a2ui 目录 URL 行为与 /playground hydration 告警为 pre-existing，测试内注记豁免
- blockers: 无
- next: review（T-08，独立复审核对 AC/SD/diff 后 reviewed 交 merge）

### 2026-09-30 合入收尾收据（PLAN-715:r1）

- stage: merge | plan_id: PLAN-715 | plan_revision: 1 | outcome: pass | delivery_commit: ddb47205e | ledger_commit: d19537b72
- **prepared** ✓：canonical 沉淀 d5d4d95e6（reviewed_commit 675306b71 的 docs/specs-only 后裔——website/design/ui-presentation.md 新建（SD-02/04/06）、website/project.md 增补（SD-01/03/05）、plans.md 回写 715 行、spec-index 重建 INDEX；实现/依赖零变化）
- **landed** ✓：rebase master（并行前进：713 行 plans.md/712 复验 c86fe01a7）后 ff-only 落 master，tip=ddb47205e 无合并提交；range-diff 三=（d7a2c47ec=2c5b1e31a、7714f030b=a2a187e5a、675306b71=c3bc4e9f8）补丁等价即安全重写证明；plans.md 冲突双行保留（713+715）
- **ledger_refreshed** ✓：d19537b72——store 写者不可用（8080 不在线、本会话无 spec 工具），循 713/714 外科插入先例：designs 段 P715-1（SD-01..05 现行知识投影，file=design/ui-presentation.md，docsha:137265b853eded5b）+ reviews 段 P715-2（复审/合入收据）；committed 形（indent1+LF，designs 118→119、reviews 180→181）roundtrip 字节守卫+语义回读全过；worktree 形（indent2+CRLF，reviews 181→182）同步插入，P712-1（plan712 会话在途 WIP）字节保全未裹挟提交
- **archived** ✓：git mv → docs/plans/archive/715-website-ui-refresh.md，status: archived，completion_kind: delivered
- **cleaned**：见下方补记
- 部署观察（landing≠deployment）：website 为静态站点，无线上部署授权（计划边界明示"不部署线上站点"）；主检出构建产物随下次部署重建，无生产进程消费本仓二进制，三项生产工件检查不适用（无后端/守护进程改动，Category A）
- 批量回归到期判定：见下方补记

### 2026-09-30 复审（T-08）

- stage: review | plan_id: PLAN-715 | plan_revision: 1 | outcome: pass | reviewed_commit: 675306b71 | base_commit: ff32d7004（master）
- dependency_revisions: 无跨仓依赖（纯 website/ 改动；auto-playground-vue 仅安装声明依赖未改源，alias 引 src）
- spec_inputs: docs/specs/website/project.md=9eb3ca3990aa0a2c（pre-merge 基线；merge 按 SD-01/03/05 回写）；docs/specs/website/design/ui-presentation.md=新建（SD-02/04/06）；docs/specs/overview.md=65e1541870a3a892
- acceptance_results: AC-01..14 全 pass——复审自证=worktree clean、零 crates/test/parity/packages 改动（231 文件均 website/+docs/reports/，Category A 成立、cargo 门禁不适用）、reviewed_commit 上新鲜重跑 build（43.3s）+ 全量 e2e 64/64（1.7m）、final 截图矩阵 96 行 problems=0（heroTop 533@390、386@1440）；工件抽验=示例证据链 hash 四件逐一比对 git hash-object 相符（bb1b46d8/9a26a7cb/2b28eacf/3bc80a25）、release-v05.ts 内容保全计数（journey leads、工具链 5 卡、roadmap 6 卡、无冻结标签注记、8 系统应用卡、launcher 实图 EN+ZH、kanban 双臂、AutoEdit 无假图）
- findings: 无新增阻塞项。登记三点：①独立性受限——本复审为实现会话自审，已按技能要求以工件重建裁定（重跑门禁+直接核验 diff/计数/hash），未假称独立代理；②两处初检 FAIL（系统卡计数/launcher 计数）经精确复核为检查脚本断言粗糙，非实现缺陷；③SD-01..06 描述与实现行为一致，canonical 回写留 merge（起草不改 Spec 分工维持）
- touched_goals 说明: 空集成立——本计划目标 G-01..04 为计划局部目标，未触及仓级 canonical goals 注册面
- evidence: docs/reports/p715-website-ui-review.md（§1 门禁表、§2 修复实录、§3 AC 映射、§4 SD 映射、§6 遗留）；docs/reports/p715-website-ui-baseline.md（接线勘定/映射表/端口）；docs/reports/p715-website-ui/{baseline,final}-report.json + 192 PNG（worktree 移除前持久化于仓内 docs/reports/）
- next: merge

### 2026-09-30 起草交接

- stage: new
- Plan: PLAN-715 / revision 1
- outcome: pass（合同完整度检查，不是实现/最终验收通过）
- next: work
- scope: T-01..08 / AC-01..14 / SD-01..06
- 四方向和发布重点全覆盖，实际路径核验，新路径标识；全部AC/SD有任务和验证；PLAN-713实图契约与共享SPA例外保留。本次不改实现/canonical Spec。
- 执行从本合同及真实源码出发，旧构建尺寸不能替代T-01最新版基线。

## 10. 待澄清事项

暂无阻塞性产品决策，采用用户同意的VitePress、紫色品牌、深浅主题及四方向。T-01的搜索适配、真实首页例子、fallback、端口、图片尺寸由执行者依据证据决定并记录；不需反复批准，不得以待调查为由丢验收。

线上后端可用性不阻塞静态展示，但AC-08仍需清楚降级和真实来源。生产部署、主版本升级、跨仓实现或新增能力超出合同，另行由用户决定；本次建立计划不触发这些动作。
