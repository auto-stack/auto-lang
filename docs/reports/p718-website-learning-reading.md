# PLAN-718 基线与验证修正报告（T-01）

状态：baseline 完成（基线截图与断言通过）；本报告记录执行前事实基线与 T-01 的验证修正。
执行树：`D:/autostack/.wt/lang-718/auto-lang`（分支 `plan-718-dev`，基点 `e0fb4e4e4`）。

## 1. 版本与依赖核实

| 组件 | 计划假设 | 实测 | 结论 |
|---|---|---|---|
| VitePress | 1.6.3 范围 | `website/node_modules` 1.6.4 | 在计划范围内；Layout 插槽/`editLink.pattern` 函数形式/VPLocalNav 行为均按本地 1.6.4 核实（见 §4/§6） |
| Vue / Playwright | ^3.5 / ^1.59 | 3.5.x / 1.59.1 | 一致 |
| Node | — | 25.2.1 | — |

依赖安装：worktree 内 `website && npm ci`、`packages/auto-playground-vue && npm ci`（该包有自己的
`@codemirror/*` 依赖树；主检出同布局。未创建任何 junction/symlink）。

## 2. 八书源与跨仓解析修正

- `D:/autostack/book` 存在全部 8 本书（tapl、rust、typescript、typescript-deepdive、little-c、
  modern-c、byte-of-python、think-python）。book 仓远端：`git@gitee.com:auto-stack/book.git`
  （分支 `master`，`origin/HEAD → origin/master`）；auto-lang 主仓远端
  `git@github.com:auto-stack/auto-lang.git`（分支 `master`）。
- **缺陷（已修）**：`prepare-content.js` 的 `BOOK_ROOT = REPO_ROOT/../book` 在 worktree 布局
  （`.wt/lang-718/auto-lang`）解析到 `.wt/lang-718/book`（不存在），八本书全部缺失、playground
  notes book 采集为 0。
- **修正**：按 AGENTS.md 跨仓依赖序实现 `resolveBookRoot()`：`AUTO_BOOK_ROOT` env 覆盖 → 相邻
  sibling（主检出布局命中）→ 组目录上溯三级（worktree 布局命中主检出相邻），候选以 `tapl`
  目录存在性验证。修正后 prepare-content 输出 8 本书全部物化、notes 采集 `book total: 634`。

## 3. 截图脚本验证缺口（SD-05 前半）

- **缺陷确认**：旧 `p715-shot.cjs` 主循环 `const url = p.zh ? '/zh' + p.url : p.url`，而 PAGES
  的 zh 页 URL 已含 `/zh/` 前缀 → 产生 `/zh/zh/...`；715 的 final 中文截图全部落在该漂移路径
  （`docs/reports/p715-website-ui/final-docs-zh-*` 为 404 页），不能作为中文正文视觉验收。
- **修正**（`website/scripts/p715-shot.cjs` + 新公共库 `website/scripts/shot-common.cjs`）：
  1. `PAGES.url` 即真实 URL，不再二次拼装；
  2. `page.goto` 后断言：pathname 未漂移（归一化比较）、locale 与 URL `/zh` 前缀一致、正文非
     404（默认主题 `.NotFound`）；任一失败脚本退出码非 0；
  3. 浏览器 context 固定 `locale: 'en-US'`（语言跳转按 URL 判定，宿主 OS locale 不泄入）；
  4. preview 服务改直接 spawn `node .../vitepress/bin/vitepress.js`（原 `shell:true` + npx 链
     在 Windows 下 vitepress 进程脱离 taskkill /T 进程树，实测 PID 19416 存活占住 stdio 管道，
     脚本永不退出）。

## 4. 移动阅读导航基线（AC-04 前置测量）

本地 VitePress 1.6.4 `VPLocalNav.vue`：`position: sticky; top: 0`（<960px），`z-index:
var(--vp-z-index-local-nav)`；≥1280px 隐藏。主题侧 `UnifiedNavbar`（layout-top 插槽）同为
`sticky; top: 0; z-index: 50`，且 `style.css` 的 `scroll-margin-top` 只按 `--site-nav-height`
（56px）计算，未叠加 local nav 高度。

实测（`scripts/p718-shot.cjs` baseline，390px 宽）：静止态 VPLocalNav 在顶栏正下方
（localNavTop=57=navbar 底），但**滚动 800px 后 16/16 行重叠**——两个 sticky 元素同时钉在
`top: 0`，移动目录（菜单按钮+目录弹层锚点）整体被统一顶栏盖住（数据
`docs/reports/p718-website-ui/reading-baseline/baseline-report.json` 的 `localNavScrolled`）。
1440px 下 VPLocalNav 正确隐藏（`display:none`，0/16 可见）。修正方向沿原生组件：窄屏给
`.VPLocalNav` `top: var(--site-nav-height)`，并把叠层实际高度计入锚点 `scroll-margin-top`（T-03）。

## 5. 编辑器生产请求图（AC-09 基线，T-05 目标）

工具：`website/scripts/scan-dist-chunks.cjs`（按内容标记 `cm-editor`/`EditorState` 定位编辑器
模块所在 chunk，不依赖文件名）。

| 事实 | 证据 |
|---|---|
| 主题入口 chunk `chunks/theme.<hash>.js` = **1,113,790 字节**，同时包含 CodeMirror（编辑器）、SnippetRunner、CodeView、ScriptShipView、NotesExplorer、AutoPlayground、AutoFence | 扫描器输出 `EDITOR-CHUNK chunks/theme.C4RLsl5o.js` |
| 首页（index.html）modulepreload 直接加载该 theme chunk（另有 framework、HomeDemo、AutoShellPreview、EvidenceImage 等） | `dist/index.html` 资源清单 |
| 结论：冷访问首页即下载全套 CodeMirror/运行器代码 | — |

T-05 拆分目标：编辑器/运行器 chunk 与主题入口分离；冷访问首页、docs/books hub、未打开 runner
的书页不请求编辑器 chunk、不调用 `/api/run`；e2e 按本扫描器输出的 chunk 清单断言。

## 6. 学习入口与导航现状（T-02/T-03 输入）

- **docs hub**：`generateDocsIndex` 输出静态 Quick Links/Sections 平铺；侧栏按目录再文件字母序，
  无学习优先级；页尾 next 为 Autocache（715 调查在案，T-03 e2e 红转绿）。
- **books hub**：`generateBooksIndex` 八书平铺；侧栏所有书一棵树、tapl 默认展开——跨书 prev/next
  由此产生（tapl 末章 next 落 rust 首章方向）。
- **EN 书侧栏 `.cn` 泄漏（实测）**：无 SUMMARY 的书（little-c 等）回退叶列表包含 `*.cn` 条目，
  如 `little-c/Getting started.cn` 与 `Getting started` 并列（`sidebar-books-en.ts`）。EN 侧栏
  不应列出中文重复项；页面 URL 不删除（AC-02 口径：导航去重，目的地不丢）。
- **书籍章节事实**（学习导航数据只陈述这些）：tapl/rust/typescript 有 SUMMARY.md；其余五本为
  `chNN-*.md` 平铺章节（零填充序号即章节序）。
- **editLink 现状（缺陷确认）**：en/zh 配置 pattern
  `https://github.com/autostack/auto-lang/edit/main/docs/:path` 双重失实——org 应为 `auto-stack`、
  分支应为 `master`；且 `:path` 直用生成文件路径（zh 页 EN 回退时并非作者源文件）。book 页无
  独立映射。修正见 T-03（源路径映射 + 生成入口页关闭 editLink）。

## 7. 阅读矩阵固定 URL（T-07 输入）

| 用途 | EN | ZH |
|---|---|---|
| docs hub | `/docs/` | `/zh/docs/` |
| books hub | `/books/` | `/zh/books/` |
| 真实长文（3 个 `##`、1 个 auto 围栏、有 .cn.md） | `/docs/features/actor-concurrency` | `/zh/docs/features/actor-concurrency` |
| 真实书章节（3 个 auto 围栏、有 .cn.md） | `/books/tapl/ch01-getting-started` | `/zh/books/tapl/ch01-getting-started` |

脚本：`website/scripts/p718-shot.cjs`（32 图 = 4 页 × EN/ZH × 390/1440 × 深/浅；额外记录
localNav 叠层几何、auto-fence 数、outline 数、页面级横向溢出）。360/768/1024 由
`tests/reader-loading.spec.ts` 几何断言覆盖（T-06）。

## 8. 基线运行回执

- p715 修正脚本 baseline 矩阵：**96/96 图，landing 断言 0 失败**（无 `/zh/zh` 漂移、无 404、
  pathname 无漂移），PROBLEM 行 0（无页面级横向溢出、hero 图全部 `within15`）。脚本退出码 0，
  进程无残留（`shell:true`+npx 链改直接 spawn node 后复测确认）。
  产物：`docs/reports/p718-website-ui/baseline/baseline-report.json` + 96 PNG。
- p718 阅读矩阵 baseline：**32/32 图，landing 断言 0 失败**；PROBLEM 行 16/32——全部为 390 宽
  滚动态 `localNavScrolled.overlapsNavbar=true`（移动目录被统一顶栏叠压，§4 修正项，T-03 关闭）；
  1440 宽 VPLocalNav 全部正确隐藏；无页面级横向溢出。
  产物：`docs/reports/p718-website-ui/reading-baseline/`（32 PNG + report.json）。
- 构建：`npm run build` 成功（122s）。构建告警 8913 条 `The language 'auto' is not loaded`
  为 715 既有基线（shiki 未注册 auto 围栏语言由 AutoFence 承接；语法高亮扩展不在本计划范围），
  非新增。水合错误：0。

## 9. T-07 视觉复核回执

- 原矩阵 final：96/96 图 landing 断言 0 失败、problem 行 0（脚本 receipts；zh 页首次全部
  落在真实 `/zh/` URL——715 的 zh 截图 `/zh/zh/` 404 缺口就此闭合）。
- 阅读矩阵 final：32/32 图 landing 断言 0 失败、problem 行 0（滚动态 localNav 叠压 0/16，
  T-03 修正生效）。
- 人工看图（视觉验收代理 ×2，逐张判定见 `docs/reports/p718-website-ui/VISUAL-REVIEW.md`）：
  阅读矩阵 32/32 pass；原矩阵子集 30/32——4 张 playground 桌面图 hero 标题顶部裁切为
  **715 已入库同缺陷（既存，非本计划引入）**，记入 KNOWN-DEBT-AND-RISKS，不扩围修复。
- 715 历史失效范围：`p715-website-ui/final-*-zh-*.png`（/zh/zh/ 404）由本计划 baseline/
  final 的 zh 图取代；715 EN 图仍有效；未覆写 715 目录。
