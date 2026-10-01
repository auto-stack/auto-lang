---
plan_id: PLAN-718
status: drafting
feature_name: 网站学习入口与文档阅读 UI
author: [agent]
created_at: 2026-10-01
updated_at: 2026-10-01
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/website/design/learning-reading.md]
touched_goals: []
affects: [website]
---

# [PLAN-718] 网站学习入口与文档阅读 UI

## 0. 变更摘要

PLAN-715 已归档，网站已有统一导航、搜索、首页演示、发布页与应用截图展示。本计划继续改善访问者从“看介绍”进入“学语言、查文档、试代码”的体验，沿用 **VitePress 1.6.x + Vue 3**，不另建网站框架。

四个方向：

1. **学习入口**：文档与八本教程从目录列表升级为有目的、适合人群和开始入口的学习导航。
2. **长文阅读**：保留 VitePress 原生侧栏/页内目录，改善移动端定位、章节衔接与中英文控件。
3. **代码示例**：运行/复制操作布局清楚，依赖模块的示例说明可读，按钮与链接遵循当前语言。
4. **加载反馈**：只在需要时加载编辑器/运行器，等待与加载失败时提供明确反馈并保留原文。

另有必要的验收修正：`p715-shot.cjs` 对已经包含 `/zh/` 的 URL 再加 `/zh`，产生 `/zh/zh/...`；旧中文截图不能充当正文视觉验收。本计划修正脚本、补齐基线，并采用 URL + 正文 + 图片三项证据。

## 1. 目标

- 读者能从 `/docs/`、`/books/` 及对应中文入口选择初学、已有语言经验、UI 实践或查阅资料的下一步；所有入口落在现有真实内容上。
- 文档侧栏优先呈现入门路径，完整参考/架构资料仍可发现；教程按当前书籍显示章节，上一章/下一章不串到其他书。
- 360px 起可使用移动目录、锚点、代码操作；固定顶栏不遮挡正文标题或目录按钮。
- 中英文的阅读控件、示例按钮、说明及目标路径一致；未翻译正文允许当前既定 EN 回退，但不能混淆为中文译本。
- 静态浏览不为未打开的运行示例加载 CodeMirror；加载失败可重试，原始代码始终可回到。

**不在本计划内**：重写文档或八本书的正文、语言能力/语法高亮扩展、后端执行协议、可复用 `packages/auto-playground-vue` 源码、账号/学习进度、VitePress 升级、宣传页面再设计、部署上线。已有 Playground hydration 警告不以隐藏/全站 ClientOnly 的方式处理。

## 2. 架构方案

- 内容来源继续为仓内 `docs/` 与相邻 `D:/autostack/book/`；所有生成文档与侧栏仍由 `website/scripts/prepare-content.js` 拥有。
- 新增网站作者维护的学习导航数据（`website/content/learning-navigation.mjs`，新），生成器与主题共同消费。数据只描述标题、受众、说明、真实内容路径与顺序，不复制正文、虚构教程难度或学习时长。
- 文档/教程根页采用有 SSR 正文的入口组件（`LearningHub.vue`，新）；由生成器输出搜索可索引的简要 Markdown 内容及组件调用。禁止手改 ignored 的 `website/docs/`、`website/books/`、`website/zh/docs/`、`website/zh/books/` 与 `sidebar-*.ts`。
- 长文继续使用 DefaultTheme.Layout 的受支持插槽、themeConfig 和 CSS；不复制 VitePress 的整套阅读器。书籍侧栏由现有 SUMMARY 顺序生成更具体的路径配置，最长前缀匹配到当前书。
- AutoFence 保留静态高亮槽位；运行器改为用户打开时的异步加载。主题中其他重组件注册按生产依赖图拆分，轻量入口与加载反馈留在网站主题内。
- PLAN-715 的单一导航数据、`useData().isDark`、搜索热键守卫、共享 SPA `index.html` URL、真实截图/输出标注等契约继续适用。

## 3. 技术栈

现有 VitePress 1.6.3 范围、Vue 3、主题 CSS、Node 内容生成脚本、Playwright。不增加 UI 框架或运行依赖。实现前核对本地安装版 Layout 插槽与类型定义；官方新版本文档不能替代仓内版本核验。

参考：[VitePress 扩展默认主题](https://vitepress.dev/guide/extending-default-theme)、[默认主题配置](https://vitepress.dev/reference/default-theme-config)、[Vue 异步组件](https://vuejs.org/guide/components/async.html)。

## 4. 需求分析与背景调查

### 4.1 授权与依赖

- 用户 2026-10-01：“计划715已经执行完毕；请继续规划下一个网站UI改进计划？”本次授权为调查、创建并提交本计划；没有将新计划默认为已批准实施或上线。
- 历史要求：网站用 VitePress，关注样式/UI；v0.3 至今的发布内容已在其他会话更新。因而本轮以阅读任务与反馈为中心，不再做提交内容汇编。
- L1，多文件网站改动；草案在 master，后续执行使用 `D:/autostack/.wt/lang-718/auto-lang`、`plan-718-dev`。建树前提交计划，树内禁止 junction/symlink，清理须先跑 `wt-guard.sh`。
- 当前无同范围活跃 Plan；715 已归档；712/716/717 为其他模块，不能修改其工作文件或借用其服务。
- 教程源依赖相邻 `book` 仓只读。T-01 核实八本书内容可生成；缺失时记录明确阻塞，不能把缺书的构建当成八本书验收通过。
- 无用户指定时间/令牌预算。仅网站 Category A 验证，不运行 cargo 测试或 docs_gen。

### 4.2 当前证据与边界

- `prepare-content.js:buildDocsSidebar` 按目录再文件产生导航，缺少明确学习优先级。实际 EN 文档入口截图首栏从 Architecture、CLI 等目录开始，正文仍为 Quick Links/Sections；页尾 Next page 为 Autocache，而非入门路径。
- `generateBooksIndex` 是八本书平铺列表；`buildBooksSidebar` 为所有书生成一棵侧栏，tapl 默认展开。当前生成物属于 ignored 文件，持久修改须进生成器/作者数据。
- `AutoFence.vue` 的按钮为 `Run`/`收起` 混用；模块依赖提示仅中文且固定 `/playground`；绝对定位于右上角，和原生 copy 控件是否碰撞需 T-01 实测。`SnippetRunner` 为静态 import。
- `theme/index.ts` 静态导入 AutoPlayground/NotesExplorer/CodeView/ScriptShipView；这提供加载拆分调查入口，但**尚未测量生产网络与字节数**，不能先宣称已造成特定性能损失。
- 当前 `.VPNav` 被隐藏，由 UnifiedNavbar 提供顶栏；本地 VitePress `VPLocalNav.vue` 在窄屏 sticky top=0，主题未同步移动目录偏移。T-01 测量实际遮挡，再沿原生组件修正。
- EN/ZH 配置未完整设置 sidebarMenuLabel/docFooter/outline 翻译。editLink 指向 `autostack/auto-lang/edit/main/docs/:path`，而 origin 是 `git@github.com:auto-stack/auto-lang.git`；生成路径也不能直接当源文件路径。
- 已查看 `docs/reports/p715-website-ui/final-docs-en-1440-dark.png`（正文）与 `final-docs-zh-390-dark.png`（404）。脚本中 PAGES 的 zh URL 已有 `/zh/`，随后循环再次加前缀，代码与截图共同确认验证缺口。这不等于正式中文站点本身 404。
- 不重开 715；在本计划 T-01/T-07 补充纠错证据，并在复审报告明确哪些历史图片失效。

### 4.3 调查基线

调查时主检出 HEAD：`6c4724f63a9ccb9e19b3525ce48e07db65542368`；以下 SHA256 前 12 位只锚定调查版本，执行时有变动须重新核对相关结论。

| 来源 | SHA256 前缀 |
|---|---|
| docs/specs/website/project.md | 5a1a542d5eba |
| docs/specs/website/design/ui-presentation.md | 137265b853ed |
| website/scripts/prepare-content.js | 956e16194ef5 |
| website/.vitepress/theme/index.ts | 96163db3cc4f |
| website/.vitepress/theme/components/AutoFence.vue | a3621a91882f |
| website/.vitepress/theme/style.css | fdb7fcfe99f7 |
| website/.vitepress/config/en.ts | cf8899dd277b |
| website/.vitepress/config/zh.ts | cc64ba5b1072 |
| website/scripts/p715-shot.cjs | 158e8c546d65 |

权威现状来自 `docs/specs/overview.md`、`docs/specs/website/project.md`、`docs/specs/website/design/ui-presentation.md` 与 `docs/specs/website/plans.md`；715 复审报告为历史验证来源。本轮没有准确对应的 GOAL-NNN，因此 touched_goals 留空，不把网站 UI 宣称为运行时跨端能力交付。

## 5. 详细设计

### 5.1 学习入口与生成导航

- 文档根页：推荐起点、按任务选择、参考资料三层；主要行动“开始学习”和“查语法”高于内部设计列表。UI 实践入口只链接已存在指南/专题，不伪称已有完整课程。
- 教程根页：主书优先；八本书按初学/已有 Rust、TypeScript、C、Python 背景给出简短选择依据与章节入口。只使用现有书的说明，没有来源的先修要求不编造。
- cards 使用现有 site tokens；每卡标题为真链接，不全卡嵌套多层交互；360px 单列、桌面分列。摘要在静态 HTML 可读且进入本地搜索。
- 文档侧栏在最上方增加经过核实的“开始使用”链接；完整参考目录保留，不把内部设计误呈现为初学路线。已存在 URL、锚点与搜索覆盖不被改名删除。
- 对原目录页所有现存合法出口做映射；重复快捷链接可以合并，但目的地不得静默丢失。生成两次内容与侧栏字节一致。

### 5.2 阅读布局与章节衔接

- 复用原生正文宽度、页内目录、侧栏抽屉与锚点导航。移动目录固定在 UnifiedNavbar 下方，目录弹层不会遮住全站导航；多层固定元素的实际高度用于 scroll-margin。
- 360/390/768/1024/1440px 检查菜单按钮、目录、选中项与长标题；移动按钮触达至少 44px；代码/宽表只在自身容器滚动，不用全局 overflow-x:hidden 掩盖问题。
- 原生 reader 控件双语：目录、此页内容、上一页/下一页、返回教程入口。中文正文回退既定行为不强制翻译源文件。
- 每本书的章节序与 SUMMARY 一致；首章不跳到其他书末章、末章不跳到下一本书，边界提供回当前书目录入口。文档根页不要默认“下一页 Autocache”；显示明确学习入口或取消无语义顺序。
- editLink 根据真实作者源路径映射：仓内 docs 跳到当前真实仓库/分支/源文件；相邻 book 的远端先核实，不能确定或属于纯生成入口时不显示虚假编辑链接。不推测远端文件存在。

### 5.3 代码示例操作

- AutoFence 的运行、收起、复制、依赖说明采用当前 locale；ZH Playground 为 `/zh/playground`、EN 为 `/playground`。
- 工具栏有独立空间，不覆盖第一行代码，不与 VitePress 原生 copy 叠放；复制原始源码，不把提示/按钮文字一起复制。模块依赖提示进入可换行的说明区。
- 含 use/import 的锁定规则与裸 auto 围栏采集口径不变；带 auto,ignore 属性的围栏保持普通代码块。不为锁定示例请求执行 API。
- 开关具有 aria-expanded/aria-controls、明确名称与焦点反馈；打开、收起、异步失败重试均可只用键盘完成。多代码块拥有唯一关联 ID。

### 5.4 按需加载与真实反馈

- T-01 先保存生产请求图/构建资源关系和静态页面基线。优先在 theme 注册边界及 AutoFence runner 边界用异步 import；不能只改写语法而仍从别处同步导入同一重模块。
- 冷访问首页、文档根页、教程根页及未打开的 AutoFence 章节时，不请求 CodeMirror 运行编辑器资源，也不自动调用 `/api/run`。含有主动渲染 CodeView/Playground 的专题不套用“永不加载”条件。
- 保留服务端高亮代码；点击运行才加载 SnippetRunner。等待时原文仍可读，显示已命名的加载状态；加载失败说明“运行器未加载”并可重试/收起，不能冒充程序运行错误。
- 双击/快速开合不会挂载重复 runner；收起不让焦点丢到不可见元素，离页时无重复执行。已加载后的 API 执行反馈沿现有 SnippetRunner 契约，不造成功输出。
- 不建立第二主题状态，不破坏搜索热键与路由；SSR 不以全站 ClientOnly 回避问题。需要只读预览的已有组件仍应保留其语义。

### 规范增量

本阶段只提出增量，review 最终对账，merge 才修改 Specs/索引。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/website/design/learning-reading.md | 无学习入口规则 → 作者数据驱动、八书分组、生成所有权及合法出口保留 | 避免手改生成物与入口遗漏 | AC-01, AC-02, AC-03 |
| SD-02 | add | docs/specs/website/design/learning-reading.md | 全书树/默认阅读顺序 → 当前书 SUMMARY 侧栏与章节边界、双语阅读控件、源编辑映射 | 长文定位与来源可信 | AC-04, AC-05, AC-06 |
| SD-03 | modify | docs/specs/website/design/ui-presentation.md | 仅顶栏/标题偏移 → 移动 local nav 叠层偏移、代码操作与溢出边界 | 延伸既有设计 token | AC-04, AC-07, AC-11 |
| SD-04 | modify | docs/specs/website/project.md | 静态全局编辑器注册与混语 AutoFence → 按需加载、双语运行控件、原文/加载失败契约 | 阅读不受重组件初始化阻塞 | AC-07, AC-08, AC-09, AC-10 |
| SD-05 | modify | docs/specs/website/design/ui-presentation.md | 截图仅记录图片 → 原始单一 URL、正文断言、locale 固定与 404 拒收 | 修正历史证据缺口 | AC-11, AC-12 |

已存在 Spec 只改对应章节，不退役组件，故 supersedes_spec_components 为空；新组件路径仅列新增 design 文件。

## 6. 测试设计

全部实现/构建/截图在 718 worktree；主检出仅做计划簿记。执行命令 cwd=`D:/autostack/.wt/lang-718/auto-lang/website`，使用该树自己的 node_modules，不链接主仓依赖。按 package-lock 的现有安装流程准备依赖。

1. `npm run prepare-content` 两次，比较 docs/books 根页、四份侧栏与学习导航生成数据哈希；`node scripts/build-playground-notes.mjs --check` 保持 notes 口径。八书源缺失即报告阻塞。
2. `npm run build`：SSG 成功，无新增构建/水合错误；检查入口静态 HTML 有标题、摘要和真实链接。
3. 新增 `tests/learning-reading.spec.ts`、`tests/reader-loading.spec.ts`：学习入口路由/搜索、当前书章节边界、移动目录锚点、双语复制运行、chunk 失败重试与快速开合。使用 mock 仅验证前端错误/状态，不称为真实后端执行验证。
4. 使用空闲独占端口，例如 PowerShell `$env:AUTO_WEBSITE_TEST_PORT='4198'; $env:CI='1'` 后 `npx playwright test tests/learning-reading.spec.ts tests/reader-loading.spec.ts`。CI=1 禁止复用未知旧服务，strictPort 遇占用直接失败；端口可调整并记录。脚本服务结束按 PID 树清理，仅清理自己创建的进程。
5. 定向新用例通过后 `npm run test:e2e` 跑 website 整套现存+新增测试（715 历史基线 64 条，只作历史参考，执行时记录实际数量）；不得删除旧用例或扩大全局警告忽略名单。
6. 修正 `scripts/p715-shot.cjs`，PAGES.url 即真实 URL，不再从 zh 布尔值重新拼装；page.goto 后校验 pathname/locale/body 非 404，固定浏览器 locale。重新生成其原 24 页 × 2 宽 × 2 主题矩阵，96 份有效图；新阅读矩阵另使用 `scripts/p718-shot.cjs`（新）。
7. 阅读矩阵包含 docs hub/books hub/一篇真实长文/一本书章节，EN/ZH × light/dark × 390/1440（32 图）；360/768/1024 用行为+几何断言覆盖。T-01 选取真实文件固定为显式 URL，不使用猜测章节路径。
8. 网络检查在生产预览、禁用缓存的独立 context 运行，依据构建依赖图定位编辑器模块，不只用资源文件名含 codemirror 的脆弱匹配；先请求静态页面再打开 runner，报告资源列表/字节与触发阶段。

## 7. 验收标准

- [ ] **AC-01 学习入口**：四个 EN/ZH docs/books 根页有明确推荐起点与任务选择，八书全部展示且有真实章节入口；静态 HTML 和本地搜索可找到入口摘要。验证：build 产物检查 + learning-reading e2e。
- [ ] **AC-02 导航完整**：入门链接优先，原合法出口有映射；每书全部 SUMMARY 章节保持顺序，既存 docs/books URL 不改名删除。验证：生成前后路由集合/出口映射与逐项目的页正文检查。
- [ ] **AC-03 生成稳定**：两次 prepare 内容与侧栏一致，notes --check 通过，没有提交 ignored 生成页；八书源可用。验证：哈希报告与 diff。
- [ ] **AC-04 移动阅读**：五个宽度深浅主题无页面级横向溢出；移动目录入口可见，锚点标题位于导航叠层之下，菜单不冲突；触控目标至少 44px。验证：reader 几何断言 + 390/1440 人工看图。
- [ ] **AC-05 章节衔接**：书内中间/首/末章节的 prev-next 指向本书 SUMMARY 相邻章节或目录，没有跨书跳转；文档 hub 不再给出无意义的 Autocache 下一页。验证：e2e 与生成导航审计。
- [ ] **AC-06 语言与来源**：中英文 reader 控件匹配当前语言，编辑源链接指向已核实的真实源；生成入口/未知远端不出现虚假 editLink。验证：属性/文本断言 + 源路径映射报告。
- [ ] **AC-07 代码工具栏**：运行/收起/复制与模块提示双语，360px 不重叠代码或 copy，锁定示例的 Playground 路径匹配 locale；复制文本与源码一致。验证：clipboard e2e、锁定用例与截图。
- [ ] **AC-08 交互可靠**：键盘开合/重试、唯一 aria 关联、可见焦点正常；快速开合不重复 runner/执行；锁定示例不调用运行 API。验证：交互 e2e、请求计数。
- [ ] **AC-09 静态浏览轻量**：冷访问首页、docs/books hub、关闭 runner 的书页不取 CodeMirror 执行编辑器 chunk、不调用 API；打开 runner 才取需要资源。验证：生产请求图前后记录与 reader-loading e2e。
- [ ] **AC-10 加载失败可恢复**：延迟/拒绝 runner chunk 时原始代码可读，有本地化加载/失败反馈，重试后可打开且收起回原文；未使用全站 ClientOnly 丢失 SSR。验证：网络注入失败/恢复与静态 HTML。
- [ ] **AC-11 截图有效**：修正中文重复路径，原矩阵 96 图与新增阅读 32 图均记录 URL/locale/正文校验；不能将 404/空页作为成功图；实际看图无错位/遮挡。验证：截图 manifest + 人工审查记录。
- [ ] **AC-12 回归守约**：website 全套 e2e、构建与 notes 检查通过，715 导航/搜索/主题/HomeDemo/共享 SPA/原生截图契约保留；无新增警告，无未批准删项/延后。验证：最终执行报告、完整 diff、独立 review。

## 8. 执行步骤

- [ ] **T-01 基线与验证修正**（无依赖）。在专用树核实版本、八书源、截图脚本错误、移动导航及编辑器生产请求图；修改 `website/scripts/p715-shot.cjs`，新增 `website/scripts/p718-shot.cjs` 与 `docs/reports/p718-website-learning-reading.md`（新），固定真实长文/章节 URL、源文件映射和基线截图。build + baseline 截图路径/正文断言必须通过；AC-02/03/04/06/09/11，SD-05。
- [ ] **T-02 学习入口**（T-01）。新增 `website/content/learning-navigation.mjs`、`website/.vitepress/theme/components/LearningHub.vue`；修改 prepare-content 的根页生成与 theme 注册，入口以可索引 SSR 内容输出；完成四页与八书选择说明。prepare 两次幂等、build 静态 HTML、入口链接 e2e；AC-01/02/03，SD-01。
- [ ] **T-03 阅读导航与来源**（T-02）。修改 prepare-content 的侧栏/源映射，`website/.vitepress/config/en.ts`、`zh.ts`、`theme/index.ts` 与 `style.css`；必要时新增 `ReaderContext.vue`（新，仅简短书籍/目录上下文）。实现原生目录偏移、双语控件、当前书路径配置、章节边界和可信 editLink。五宽度/锚点/首末章节/路径 e2e；AC-02/04/05/06，SD-02/03。
- [ ] **T-04 代码操作 UI**（T-01/T-03）。修改 `theme/components/AutoFence.vue`；仅在必要时调整 `theme/auto-fence-md.ts` 的静态围栏标记，保持 info==='auto' 与锁定语义。统一双语工具栏、复制与焦点/aria，长提示流式换行。新代码按钮 e2e 与 notes --check 通过；AC-07/08，SD-03/04。
- [ ] **T-05 组件加载反馈**（T-01/T-04）。修改 theme/index.ts/AutoFence.vue；按已测依赖关系调整 `CodeView.vue`、`ScriptShipView.vue` 等网站包装层，必要时新增 `ReaderLoadingState.vue`（新）。禁止编辑 packages/crates；异步注册及 runner 等待/重试保留 SSR 原文。冷访问请求图、失败/恢复/快速开合 e2e 通过；AC-08/09/10，SD-04。
- [ ] **T-06 行为回归**（T-02..T-05）。新增 `website/tests/learning-reading.spec.ts`、`reader-loading.spec.ts`；必要时增补现有 `site-ui.spec.ts`。运行 §6 定向用例后 website 全套，记录实际数量与新旧警告，不以 mock 结果充当真实程序能力。AC-01..AC-10/12，SD-01..SD-04。
- [ ] **T-07 视觉复核**（T-06）。运行修正原矩阵与新阅读矩阵，保存 `docs/reports/p718-website-ui/`（新，受控报告目录）的截图与 manifest；人工检查正文/代码/加载态/移动叠层，必要修正后只重跑受影响检查。明确历史中文图片失效范围，不覆写旧报告以掩盖问题。AC-04/07/10/11/12，SD-03/05。
- [ ] **T-08 交付复审**（T-07）。填写计划执行证据、报告与规范增量对账，status=execution_done；调用 /auto-plan:review 独立逐条复核 AC、遗漏/延后/绕过及 KNOWN-DEBT-AND-RISKS。review 通过才可进入 merge 进行 Spec/索引沉淀、归档及 guarded worktree 清理。AC-12 与全部 SD；不得把草案自查写成实现复审通过。

执行阶段按每任务追加真实命令/结果/证据；任务完成勾选属于证据索引，不能替代实测。承接者先核对源哈希变化与 authorization，再创建专用 worktree；不能在 master 直接修网站。

## 9. 复审记录

### 草案交接（非实现复审）

- stage: new
- plan: PLAN-718
- plan_revision: 1
- outcome: pass（规划契约覆盖完整，可供确认后执行；不代表新 UI 已交付）
- next: work（用户确认此具体草案后按仓库标准进入 /auto-plan:work）
- changed_tasks: T-01..T-08
- changed_acceptance: AC-01..AC-12
- spec_delta: SD-01..SD-05
- 草案检查：T/AC/SD 互相覆盖；生成物所有权、相邻书籍依赖、截图路径错误、独占预览与异步网络实测均有明确责任任务。没有修改 website 实现或 canonical Specs。

### 实现独立复审

待执行完成后填写 revision-bound review 证据、结论和债务扫描；当前不声称 reviewed。

## 10. 待澄清事项

- 没有阻止规划完成的内容偏好问题。方向默认沿用 715 的视觉 token、真实内容与双语 URL，不添加阅读设置面板。
- T-01 负责实际确认所选章节、八书源、source remote、生产依赖图和移动目录遮挡；这些是受限技术调查，不需要先向用户反复索要选择。
- 若按需加载必须改动 packages/后端协议，或实际需要重写书籍内容/改 URL，则停在对应任务，保留已完成证据，按 needs_replan 提交具体范围变化；不能静默扩大本计划。
