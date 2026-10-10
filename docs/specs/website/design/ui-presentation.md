# website — UI 呈现规范（宣传/文档统一视觉）

> **Status**: active
> 来源：plan-715（VitePress 全站 UI 与 v0.5 发布展示改进）
> 适用：`website/` 全站——宣传页（landing）、文档、书籍、Playground；规则经 e2e 与截图矩阵验证。

## 1. 设计令牌与排版（SD-02）

- 全站令牌定义在 `theme/style.css` `:root`：`--site-nav-height: 56px`、`--site-max-width: 1200px`、`--site-text-width: 720px`、`--site-radius`、`--site-body-size: 1rem`、`--site-body-lh: 1.75`。组件与页面从令牌取值，不得再各自硬编码宽度/高度/行高。
- 宣传主体（`.landing-page` 各区块）最大宽 `--site-max-width`；长文列宽 `--site-text-width`；正文 16–17px、行高 1.65–1.8。章节标题/简介与正文同向对齐（概览可居中，长文靠左）；`.landing-page` 作用域隔离宣传规则，禁止污染文档组件（VPFeature 等同样使用 `.title`）。
- 紫色（`--vp-c-brand-1: #6366f1` 族）用于品牌、主按钮、当前位置与强调；背景光晕/彩边弱化（`HomeHero` 光晕 opacity ≤0.22），实图与正文是主要展示，卡片只承载短事实与并列选择。
- 文档/书籍页保全 VitePress 默认版式：侧栏、阅读宽度、代码局部横滚、移动目录；`vp-doc h1..h4` 的 `scroll-margin-top` = 顶栏高 + 16px，锚点定位不被 sticky 顶栏遮挡。

## 2. 响应式与无障碍硬规则（SD-02/SD-06）

- 断点矩阵：360 / 390 / 768 / 1024 / 1440px 下网格与 hero 不得超出父宽；网格子项 `min-width: 0`（grid 轨道默认 min-content 会把长代码行/长 token 撑破视口——基线实测 449>390 根因）；长斜杠/URL token 用 `overflow-wrap`（StatCard 描述 `anywhere`，landing 兜底 `break-word`）；禁止用全站 `overflow-x: hidden` 掩盖裁切。
- UnifiedNavbar 桌面导航断点为 **1200px**（1024 视口下导航+操作区总宽实测溢出 39px）；1024–1200 走分组手机菜单。
- 触控目标 ≥44×44px（导航图标钮移动端 44px，桌面收窄为 36px）。
- 键盘：菜单/下拉 `aria-expanded`/`aria-controls`，Esc 收起并回焦触发器，导航后自动收起；当前项 `aria-current="page"`、父组 `"true"`；dialog 打开/关闭有焦点往返（EvidenceImage 模式）。
- 深浅主题无不可见文字/白块；主题切换跨页保持（统一走 VitePress `isDark`）；`prefers-reduced-motion` 下 reveal 类动效禁用且内容恒可见（opacity > 0.9）。
- 覆盖矩阵：EN/ZH × 360/390/768/1024/1440 × 深/浅——新增/重排页面必须过该矩阵（`website/scripts/p715-shot.cjs`，产物入 `docs/reports/p715-website-ui/`）。

## 3. 发布展示结构（SD-04）

- v0.5 及后续发布页用 `ReleaseLanding.vue` + `theme/data/release-*.ts` 数据驱动（EN/ZH 同构），页面 md 只保留 frontmatter 与组件调用；Markdown 路由与 metadata 不变。
- 章节顺序：主视觉（真实桌面主图，eager + 尺寸占位，top < 1.5×视口高）→ 三项亮点（一句话+例子）→ 桌面 → AutoUI 双端（一源两端对照图，不暗示超出验证范围的全等）→ 旗舰应用 → 系统应用/语言工具链/Playground → 理念及历程 → 展望 → 开始使用。
- 理念呈现双层：正文一句话+例子；完整论述进原生 `<details>`（键盘可展开、hash 可定位），重组内容时旧论述逐条保全（映射表先行，见 `docs/reports/p715-website-ui-baseline.md` §6 模板）。
- 规模统计与数据来源注记（快照日期、Token 来源、无冻结标签声明）放历程区，不放首屏；计数/日期/来源逐字保全。
- 章节导航 `SectionNav.vue`：桌面吸顶（top = `--site-nav-height`）横向快捷条、手机可展开目录；当前章节 IntersectionObserver 跟踪 + hash 点击即时反馈；目标章节 `scroll-margin-top` = 顶栏 + 导航条高度，标题不被两层 sticky 遮挡。
- 无有效实图的应用保留文字、不放假占位图/死链；开始菜单等补图须用来源匹配的入库实图。
- PLAN-756 用户明确要求先修文本、截图留空：本轮首页/v0.5/产品专题可用有标题、场景和 `SHOT-01..12` 的文字待拍槽。槽不渲染图片、不冒充已完成展示，拍摄合同在 `docs/reports/website-v05-capture-guide.md`；新图验收后替换为 EvidenceImage。现有获准桌面、Shell 和 AutoEdit 专题原图保留。四主应用从 applicationCopy 取数；AutoDown/Jade Garden 作为相关资料，与 JadeEdit 分开。发布状态明示里程碑与待冻结工件，运行/转译/调试明示后端条件。
- PLAN-720 桌面截图：本批发布首图使用无展开窗口的深色桌面；桌面选择器先呈现浅/深主题，再看 Launcher、游戏多窗口、编辑器工作布局。作者数据在 `desktop-showcase.ts` 单源维护，中英文同构；OS 总览仅双主题预览并引导到完整专题。caption 明确小组件应用启动并最小化的前提，源 PNG 字节保全、2560×1600 占位、非首图 lazy、原图可访问。主题对照表示截图切换，不宣称操作静态图片会切换实际桌面状态。

## 4. 测试服务隔离与回归纪律（SD-06）

PLAN-719 补充：OS/AI 介绍、AutoOS 与 desktop 专题使用默认文档阅读布局，概要/双语关系图由 `IntroductionFrame.vue` 呈现，Markdown 正文默认展开且可 SSR/search。关系示意显著标注，呈现宿主内核及渲染/合成设施；真实图遵循 PLAN-720，禁止用假代码输出/无限模型/缺依据数字墙作为能力证据。历史文章用目录、锚点和来源链接；五宽度两主题几何及十二路由 × 390/1440 × 深浅共 48 图校验实际正文。规则见 [OS/AI introduction](os-ai-introduction.md)。

PLAN-718 补充（交付契约，规则见 [learning-reading](design/learning-reading.md)）：移动阅读 `.VPLocalNav` 的 sticky 偏移包含统一顶栏 56px 内部行和 1px 底边框；锚点 scroll-margin 按"顶栏+目录条"两层实际高度让位（`--site-reading-offset`）；窄屏目录弹层触发按钮触达 ≥44px（清零弹层根节点垂直 padding，条高保持不变）。AutoFence 操作栏在源码上方占独立文档流空间。截图使用数据中的原始 URL，不按 locale 再拼前缀；校验 pathname、正文非 404、语言正确后才接收图片（截图 manifest 随报告入库，`*.json` 有 `!docs/reports/**` 白名单）。715 原中文 `/zh/zh/` 图片无效，新证据使用修正后的脚本；独立 preview 直接启动 Node 进程，避免 Windows shell 子进程残留。

- e2e/截图端口由 `AUTO_WEBSITE_TEST_PORT` 控制（默认 4173 兼容；strictPort，baseURL 与 webServer 同源）——并行 worktree 各选独占空闲端口（715 用 4186），不复用未知服务；**Windows 下 `shell:true` spawn 的 kill 只杀 shell 不杀 node 子进程**，长驻 preview 会固化旧 dist 的文件清单导致重建后 404（715 实证），脚本清理须 `taskkill /PID <pid> /T /F`。
- Playwright 配置钉定 `locale: 'en-US'`：否则宿主 OS 中文 locale 泄入 `navigator.language`，首页的浏览器语言自动跳转会把 `/` 的测试整个搬到 `/zh/` 上（715 实证）。
- 新增/重排页面的回归面：`tests/site-ui.spec.ts`（导航/搜索/键盘/断点/画廊/ash 回归/md 缩进代码块泄漏锁）+ `tests/home-demo.spec.ts`（预置/实时/失败/超时/取消/静态降级）+ 既有 `spa-routes` / `playground-notes`；spa-routes 的 gallery 直连等重 SPA 用例标 `test.slow()`（4 worker 并行下 networkidle 需更宽时间窗）。
- 已知预存项（非回归，测试内注记豁免）：`/playground` 的 hydration mismatch 告警（AutoPlayground 运行时态）；`/ui/a2ui/` 目录 URL 优先命中 VitePress 专题页（sirv extensions 行为，SPA 规范入口为 `/ui/a2ui/index.html`）。
- mock 只证明网站反馈；示例内容真实性凭仓内源码/金样/a2r 产物另行验证（证据 hash 记录于计划/报告）。
