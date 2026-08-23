# examples/ui — AutoUI 应用示例（App 轨道）

这里只放**应用性质**的示例：每个目录是一个可 `auto run` 起来的完整 UI 应用，
以"完成一件事"为目标，而不是展示某个语言特性。

仓库自 2026-08-23 起**分轨**：

| 轨道 | 位置 | 内容 |
|---|---|---|
| **App 轨道**（本目录） | `examples/ui/` | 应用示例与孵化器，未来可升级为真正的应用 |
| 能力样板（fixture） | [`examples/capability-tests/`](../capability-tests/) | 单特性 e2e 钉子（原 021-block-static、026–040），随特性测试化而退役 |
| 组件画廊 | [`examples/widgets-gallery/`](../widgets-gallery/) | 全部 shadcn widgets 的文档画廊（原 024） |

## 运行方式

```bash
cd <示例目录>
auto build          # .at → gen/front/vue（Vue 项目）
auto run            # 构建并启动 dev server
auto run --render vm   # 038/041 等支持：直接跑 AutoVM 解释器（原生窗口）
```

- 默认 `render: vue`；`038-minesweeper`、`041-auto-edit` 支持 vm 模式。
- 带后端的示例（015/017/018/022/023）会同时起 `back_port`。
- 测试设施（按各示例 README 惯例运行）：
  - **MCP 桌面测试** `tests/desktop_mcp.py`（013 惯例）：011、013、015、038、041
  - **Playwright 冒烟** `tests/smoke.spec.ts`：015、017、018、022、023
  - **ATD 验收** `tests/acceptance.atd`（Plan 366 DSL）：011、013、015、017、018、022、023

## 编号说明（024–040 空洞的来历）

编号有空洞，均有明确去向。**新示例优先填入空洞**（从 024 起，编号顺序
即构建/复杂度顺序，详见 [Design 21 §2](../../docs/design/21-examples-app-track.md)）；
填入时在本 README 加一行历史注记。历史去向：

- **024**：原 widget-gallery，已升级为顶级 [`examples/widgets-gallery/`](../widgets-gallery/)。
- **025**：原 notes 前端丰富度临时 fork，能力并入 015-notes 后删除（Plan 354 §7）。
- **021-block-static、026–040**：能力样板，迁至 [`examples/capability-tests/`](../capability-tests/)（同号共存，引用带全路径即无歧义）。
- **038/041 保留**：撞号的 038-vshow 已随迁移离开，038 现在专指扫雷。

## 示例总览

| 编号 | 名称 | 一句话 | 端口 | 状态 |
|---|---|---|---|---|
| 001 | helloworld | 最简静态文本 | — | 基础 |
| 002 | counter | 加减计数器（Elm 架构入门） | — | 基础 |
| 003 | converter | 双向温度换算（7GUIs #2） | — | 基础 |
| 004 | profile-card | 静态资料卡 | — | 基础 |
| 005 | login | 登录表单 | — | 基础 |
| 006 | hero-section | 落地页 hero | — | 基础 |
| 007 | stats-board | 指标仪表卡 | — | 基础 |
| 008 | pricing-table | 三档定价表 | — | 基础 |
| 009 | article-feed | 文章卡片流 | — | 基础 |
| 010 | contact-form | 联系表单 + 提交反馈 | — | 基础 |
| 011 | calculator | 四则计算器 | — | 🔀 升级拆出（Plan 401） |
| 012 | stopwatch | 秒表 + 计圈 | — | 基础 |
| 013 | todo | TodoMVC 完整实现 | — | ✅ 有 MCP 测试 |
| 014 | weather | 天气仪表盘 | — | 基础 |
| 015 | notes | 两栏笔记（真实应用形态） | — | ✅ Plan 354 升级 |
| 016 | calendar | 月历 + 事件高亮 | — | 基础 |
| 017 | chat | 微信风即时聊天 | — | ✅ 有 playwright/ATD 验收 |
| 018 | book-reader | 多页电子书阅读器 | 3018/8018 | ✅ Plan 401 升级（playwright 10/10） |
| 019 | video-app | 视频浏览（B 站风） | — | ⬜ 待升级 |
| 020 | music-player | 迷你音乐播放器 | — | ⬜ 待升级 |
| 021 | blog-viewer | 博客列表 + 详情 | — | ⬜ 待升级 |
| 022 | kanban | Trello 风看板 | 3022/8022 | ✅ Plan 401 升级（playwright 6/6） |
| 023 | realworld | Conduit（Medium 克隆） | 3023/8023 | ✅ Plan 405（playwright 14/14） |
| 038 | minesweeper | 经典扫雷（双后端） | 4038 | 🎯 严肃应用，持续扩展 |
| 041 | auto-edit | 文本编辑器 | 4041 | 🎯 严肃应用，持续扩展 |

---

## 详细说明

示例分层沿用 [Plan 183](../../docs/plans/archive/183-unified-ui-examples.md) 的
Tier 分类；升级状态由 [Plan 401 纲领](../../docs/plans/401-autoui-examples-upgrade.md)跟踪。

### Tier 1 · Basics（001–005）— 语言入门

**[001-helloworld](001-helloworld/) — 最简静态文本**
能跑起来的最小 AURA widget：居中一行文本。讲授 view 树结构、`text`、
`col` 纵向布局与 `class` 工具类样式。一切从这开始。

**[002-counter](002-counter/) — 交互计数器**
加/减/重置三按钮计数器，Elm 架构（model/view/on）的教科书示例：
`model {}` 状态、`msg Msg` 消息枚举、`onclick: .Inc` 事件绑定、
f-string 插值 `` `Counter: ${.count}` ``。

**[003-converter](003-converter/) — 双向温度换算**
摄氏 ⇄ 华氏双向编辑互相同步（7GUIs Task #2）。核心是 `input` 的
`value` 绑定 + `oninput` 每击键更新 model 的受控输入模式。

**[004-profile-card](004-profile-card/) — 静态资料卡**
头像、状态徽章、简介、操作按钮的视觉组合（PrimeBlocks 风格）。
演示 `image`、col/row 混合嵌套、渐变/阴影/圆角等纯样式能力，无交互状态。

**[005-login](005-login/) — 登录表单**
邮箱 + 密码 + 条件错误提示 + 提交按钮的完整表单模式：
密码型 `input`、`if` 条件渲染错误信息、卡片式表单布局。

### Tier 2 · Blocks（006–010）— 布局与组合

**[006-hero-section](006-hero-section/) — 落地页 hero**
标题层级 + 渐变背景 + CTA 按钮的经典 landing 首屏，flex 居中布局。

**[007-stats-board](007-stats-board/) — 指标仪表卡**
Revenue/Users/Orders/Growth 四张等宽指标卡（`row` + `flex-1`），
静态数据展示与卡片重复模式。

**[008-pricing-table](008-pricing-table/) — 三档定价表**
Basic/Premium/Enterprise 三档 + 月付/年付 `switch` 开关切换价格，
布尔状态驱动的条件内容。

**[009-article-feed](009-article-feed/) — 文章卡片流**
三篇文章卡的纵向信息流：标题/摘录/作者日期页脚，`max-w-2xl` 限宽排版。

**[010-contact-form](010-contact-form/) — 联系表单**
姓名/邮箱/留言三字段 + 提交后条件成功提示：`textarea` 多行输入与
提交反馈模式。

### Tier 3 · Mini Apps（011–016）— 迷你应用

**[011-calculator](011-calculator/) — 四则计算器** 🎯 AutoOS 毕业候选
显示区 + 5×4 按钮网格（数字/运算符/=/C/%/.）。链式运算状态机
（`display`/`prev_value`/`operator`/`new_number`）是真正的应用逻辑，
不是玩具。已在 Plan 401 纲领下拆出专项升级（grid 重构 + MCP 桌面测试 +
多模式）。操作系统默认应用候选。

**[012-stopwatch](012-stopwatch/) — 秒表 + 计圈**
MM:SS.cc 显示、起/停/重置三态控制、计圈列表。随 `.Tick` 定时器机制
（setInterval 驱动）落地而生，是该特性的实机载体。

**[013-todo](013-todo/) — TodoMVC** 
完整 [TodoMVC](https://todomvc.com) 规格：增删、勾选、全选、筛选
（all/active/completed）、双击编辑、清除已完成。是 `tests/desktop_mcp.py`
MCP 桌面测试惯例（"013 惯例"）的发源地。

**[014-weather](014-weather/) — 天气仪表盘**
当前天气卡（渐变背景）+ daily/hourly 标签页切换 + 湿度风速信息，
`divider` 分隔与 tab 条件渲染。

**[015-notes](015-notes/) — 两栏笔记** ✅ [Plan 354](../../docs/plans/archive/354-015-notes-real-app.md) 升级
树状文件夹/笔记导航 + AutoDown（Tiptap）所见即所得编辑器 + 后端 CRUD
持久化 + 暗色模式。已吸收原 025-notes-extended 的 SharedStore + 多路由
概念，是"demo 升级为真实应用"的完整先例。有 MCP 桌面测试。

**[016-calendar](016-calendar/) — 月历** 🎯 AutoOS 毕业候选
`grid { cols: 7 }` 七列月历：日期格、事件高亮点、上/下月导航。
操作系统默认应用候选；重复事件（RRULE）语义按约定留到升级为正式应用时
才引入，demo 阶段不背这个负担。

### Tier 4 · Real Apps（017–023）— 真实应用形态

**[017-chat](017-chat/) — 即时聊天**
微信风双栏：联系人列表 + 气泡会话流（收发双方不同气泡样式）、
时间戳、底部输入条、空输入禁发、消息区自动滚动。
有 playwright 冒烟与 ATD 验收（`tests/`）。

**[018-book-reader](018-book-reader/) — AutoRead 电子书阅读器** ✅ [Plan 401](../../docs/plans/401-autoui-examples-upgrade.md) 升级
书架 → 详情 → 沉浸阅读的多路由 SPA（vue-router + `<outlet>`），
配强类型 Rust 后端（章节 CRUD + 阅读进度持久化 PUT）与运行时明暗主题
切换（escape-hatch 定制组件）。Plan 401 首个完成升级的示例，
playwright 10/10 全绿。

**[019-video-app](019-video-app/) — 视频浏览** ⬜ 待升级（Plan 401）
B 站风：搜索栏、分类 chips、推荐/热门/关注 tabs、`grid { cols: 3 }`
响应式缩略图卡片墙。135 行单文件，升级优先级：中。

**[020-music-player](020-music-player/) — 迷你音乐播放器** ⬜ 待升级（Plan 401）
Spotify 风：渐变专辑封面、上一曲/播放暂停/下一曲、可拖动 `progress`
进度条、"Up Next" 播放队列。115 行单文件，升级优先级：中。

**[021-blog-viewer](021-blog-viewer/) — 博客阅读** ⬜ 待升级（Plan 401）
左列表右详情双栏：文章卡（标题/作者/日期/摘录/Read More）、
SelectArticle/BackToList 视图切换。89 行单文件，升级优先级：中。

**[022-kanban](022-kanban/) — Trello 风看板** ✅ [Plan 401](../../docs/plans/401-autoui-examples-upgrade.md) 升级
To Do / In Progress / Done 三列看板：加卡、删卡、HTML5 跨列拖拽，
配强类型 Rust 后端（`#[api]` + db.at）。多模块前端结构
（app.at 路由壳 + board_store.at 共享 store + pages/board.at），
playwright 6/6 全绿。

**[023-realworld](023-realworld/) — Conduit（Medium 克隆）** ✅ [Plan 405](../../docs/plans/405-023-realworld.md)
[RealWorld](https://github.com/gothinkster/realworld) 规格实现：
认证（登录/注册/设置/登出）、带标签过滤的全局 feed、文章详情、
文章 CRUD 编辑器、评论、关注、收藏、个人资料（分页与 markdown 渲染
暂缓；stage 1/2 认证为 mock）。playwright 14/14 全绿。

### 独立应用项目（038、041）

这两个不是"教学示例"，而是**严肃应用**，持续迭代、未来扩展，
是 AutoOS 默认应用的第一批毕业候选。

**[038-minesweeper](038-minesweeper/) — 经典扫雷** 🎯
完整可玩：首击安全、空白区洪水填充展开、右键插旗、三档难度
（9×9 / 16×16 / 30×16）、计时器与剩余雷数计数。**首个双后端示例**——
同一份纯 AutoLang 游戏逻辑，既编译为 Vue store（vue 后端）又直接跑在
AutoVM 解释器上（vm 后端，原生窗口）；[Plan 407](../../docs/plans/407-minesweeper-rust-backend.md)
又落地了 rust 第三后端做三端对比。有 MCP 桌面测试。

**[041-auto-edit](041-auto-edit/) — auto-edit 文本编辑器** 🎯（原 041-code-editor）
我们自己的文本编辑器。原生 `code_editor` widget（CodeMirror 6 /
syntect，`lang: "auto"` 复用 widget-tag 关键字高亮）编辑 `.at` 源码，
多 tab + menubar/toolbar（[Plan 414](../../docs/plans/414-auto-edit-ux.md) UX）、
动作注册表 + 全局快捷键（[Plan 418](../../docs/plans/archive/418-auto-edit-actions-and-config.md)
`auto-edit.at`，命名对齐 auto-os-config 实体先例）、迷你 "Auto Playground"
运行按钮触发 VM 求值。默认 vm 渲染实机运行，MCP 动作矩阵 28/28。
进行中：tabs/workspace（[Plan 420](../../docs/plans/420-auto-edit-tabs-workspace.md)）、
代码折叠（[Plan 428](../../docs/plans/428-code-folding-phase-b.md)）；
[Plan 386](../../docs/plans/386-autoui-renderqueue-future-optimization.md)
Stage 1 的 golden 对照样例。

---

## 升级与毕业（AutoOS）

> 本节仅摘要；完整战略见 [Design 21 — 应用示例轨道与 AutoOS 默认应用矩阵](../../docs/design/21-examples-app-track.md)（分轨、编号约定、默认应用矩阵、填洞路线 024–033、demo 边界、五闸门、golden 三件套、计划结构）。

- **示例内升级**（教学 toy → 真实应用形态）：由
  [Plan 401 纲领](../../docs/plans/401-autoui-examples-upgrade.md)统一跟踪，
  已完成 018/022/023（+011 拆出），待办 019/020/021。
- **毕业进 AutoOS**：当示例开始被真实日常使用、且平台需要它作长期回归
  样板时，迁出本目录成为正式应用（pac 定名 + 动作对齐 auto-os-config +
  MCP 测试纳入 CI）。当前毕业候选：**auto-edit（041）**、计算器（011）、
  日历（016）、扫雷（038）。
- **新示例**：优先填空洞（024–037、039/040，路线见 Design 21 §5，起点
  024-charts）；单特性样板一律进
  [`examples/capability-tests/`](../capability-tests/)，不进本目录。
