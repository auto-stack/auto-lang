# 21 — examples/ui 应用示例轨道（App Track）与 AutoOS 默认应用矩阵

> **状态**:设计文档(定稿 v1)
> **日期**:2026-08-23
> **关联**:[Plan 183](../plans/archive/183-unified-ui-examples.md)(示例初版分层)、[Plan 401](../plans/401-autoui-examples-upgrade.md)(存量示例升级纲领)、Plan 413/414/418/420/428(auto-edit 系列)、Plan 402/407(扫雷)、[Design 16](16-app-generation-and-ai-authoring.md)/[Design 18](18-shared-store.md)、[Plan 386](../plans/386-autoui-renderqueue-future-optimization.md)(RenderQueue golden 三件套)、[examples/ui/README.md](../../examples/ui/README.md)
> **目的**:定义 `examples/ui/` 的定位、编号约定、分层、**AutoOS 默认应用矩阵**、demo 边界与毕业闸门、填洞路线与计划结构。本文是 App 轨道的**战略文档**;具体执行归计划体系(见 §10)。

---

## 1. 定位与分轨

`examples/ui/` 只放**应用性质**的示例:每个目录是一个可 `auto run` 的完整 UI 应用,以"完成一件事"为目标。单特性样板(component 能力钉子)一律进 [`examples/capability-tests/`](../../examples/capability-tests/),组件文档画廊在 [`examples/widgets-gallery/`](../../examples/widgets-gallery/)。

| 轨道 | 位置 | 内容 | 退役条件 |
|---|---|---|---|
| **App 轨道** | `examples/ui/` | 应用示例与孵化器 | 毕业(迁入 AutoOS) |
| 能力样板 | `examples/capability-tests/` | 单特性 e2e 钉子(gap 金丝雀 + 特性 fixture) | 特性获得可覆盖的内联测试 |
| 组件画廊 | `examples/widgets-gallery/` | 全 shadcn widgets 文档 | 常驻 |

**App 轨道的第一职责是钉平台缺口**:每个示例必须回答"它让 AutoUI 的哪块能力变得可验证"。营销截图型示例不立项。

## 2. 编号约定

对齐 plan 体系的哲学:**活跃序列应连续;空洞 = 已退役内容的墓碑;新增内容优先填洞**。

- 编号只增,不与**活跃含义**冲突;复用空号时在 README 加一行历史注记(如"024 原为 widget-gallery,已迁顶级")。
- 编号顺序 = 构建顺序 = 复杂度递增序,读目录即学习路径。
- `capability-tests/` 保留 021-block-static、026–040 旧编号,与 App 轨道同号共存不冲突(引用一律带全路径;fixture 退役后编号自然归还语义)。
- 当前空洞:**024–037(14 个)+ 039/040(2 个)= 16 个**,填洞路线见 §5。

## 3. 分层与现状

沿用 Plan 183 的 Tier 分类,新增"独立应用项目"层:

| 层 | 编号 | 性质 |
|---|---|---|
| Tier 1 · Basics | 001–005 | 语言入门(view 树/Elm 架构/受控输入/表单) |
| Tier 2 · Blocks | 006–010 | 布局与组合(卡片/网格/开关/信息流) |
| Tier 3 · Mini Apps | 011–016 | 迷你应用(计算器/秒表/TodoMVC/天气/笔记/日历) |
| Tier 4 · Real Apps | 017–023 | 真实应用形态(聊天/阅读器/视频/音乐/博客/看板/Conduit) |
| 独立应用项目 | 038、041 | 严肃应用,持续迭代,AutoOS 毕业第一梯队 |

逐示例的详细说明、端口与状态见 [examples/ui/README.md](../../examples/ui/README.md) 总表,不在本文重复。

## 4. AutoOS 默认应用矩阵

**原则:AutoOS 的每一款系统默认应用,都先在 App 轨道有 demo 孵化。**

| AutoOS 应用 | demo 载体 | 状态 |
|---|---|---|
| 文本编辑器 | 041-auto-edit | ✅ 在孵,最接近毕业 |
| 日历 | 016-calendar | ✅ 有 demo,待升级 |
| 计算器 | 011-calculator | ✅ 有 demo,待升级 |
| 笔记 | 015-notes | ✅ 已是真实应用形态 |
| 游戏 | 038-minesweeper | ✅ 双后端,持续扩展 |
| 天气 | 014-weather | ✅ 有 demo |
| 音乐 / 视频 | 020 / 019 | ✅ 有壳,待升级 |
| 时钟(世界钟/闹钟/计时) | 012-stopwatch | ◐ 原地升级为 clock suite,不占新号 |
| 系统监视器 | 025-dashboard | 🆕 §5 |
| 文件管理器 | 027-file-manager | 🆕 §5 |
| 启动器 | 028-launcher | 🆕 §5 |
| 图片查看器 | 029-image-viewer | 🆕 §5 |
| 邮件 | 030-email | 🆕 §5 |
| 系统设置 | 033-settings | 🆕 §5(轻,后置) |
| 终端 | — | 远期(pty + 等宽网格) |
| 浏览器 | — | 独立产品线,不属 demo 范畴 |

## 5. 填洞路线(024–033)

| 编号 | 应用 | 钉住的平台缺口 | 成本 | 依赖 |
|---|---|---|---|---|
| 024 | **charts** 图表工坊 | 模型驱动数据系列、hover/tooltip 交互、**vm 端自绘图表**(矢量) | 中 | — |
| 025 | **dashboard** 系统监视器 | charts 组合 + KPI + 轮询刷新(.Tick) + 响应式布局 | 低 | 024 |
| 026 | **database** SQLite 客户端 | DataTable 深水区(虚拟滚动/分页/排序/筛选)、**Tree 缺口**、SQL 编辑器(复用 code_editor)、rusqlite 后端 a2r 大结果集 | 高,分三阶段(只读浏览→SQL console→写操作) | — |
| 027 | **file-manager** 文件管理器 | Tree 导航、列表/网格切换、右键菜单(Plan 422 已解锁)、内联重命名、fs 后端 | 中 | — |
| 028 | **launcher** 启动器 | **焦点/键盘导航原语**(roving focus/Enter/Esc 分层,三端共同薄弱)、模糊搜索、结果网格虚拟化 | 中 | — |
| 029 | **image-viewer** 图片查看器 | 图片解码缓存管线、zoom/pan 手势、全屏、缩略图懒加载(图像) | 中 | — |
| 030 | **email** 邮件 | **万级虚拟化长列表**(与 026 表格形态互补)、已读状态同步、AutoDown 富文本复用(写邮件) | 高,mock 后端 | 015(AutoDown 先例) |
| 031 | **git-gui** | commit DAG 自绘、双栏 diff(code_editor 行高亮)、staging 右键、git2-rs 后端 | 高 | 422、code_editor |
| 032 | **admin** 管理后台 | (再评估)DataTable 交互全谱 + dialog 表单 + 权限路由;与 023/022/026 重叠度高 | 中 | 026 落地后评估 |
| 033 | **settings** 系统设置 | 表单 + tabs + 持久化;平台锻炼低但 OS 刚需,可兼作 Design 16 AI 生成试金石 | 低 | — |

剩 034–037、039–040 留空,候选池(settings 之外另有 spreadsheet/paint/terminal/contacts 等)等真实需求出现再填,**不为凑连续性造示例**。

**各 demo 的边界**(通用边界见 §6):

- launcher:不做真窗口管理/进程启动,只做 UI 壳 + 键盘流;command-palette 沉淀为可复用 widget 原语。
- image-viewer:只读本机文件(后端 fs),不做照片库管理。
- email:后端本地 mock,前端按将来 IMAP 的 API 形状设计接口(换真后端前端零改动)。
- database/文件管理器/git-gui:后端 Rust 实做,前端不 mock。

## 6. demo 边界(通用)

**不做**:持久化 schema 迁移、真实 IO 的完整错误恢复、无障碍全覆盖、性能优化、i18n、设置面板——这些是正式应用的负担,提前做会拖慢 demo 迭代并模糊特性回归焦点。

**该有**:单一场景、happy path + 少量错误态、几百行内、SPEC.md 可再生(025 的 SPEC 模式)、双模式构建绿(vue 必绿;核心应用加 vm,见 §9)。

## 7. 升级与毕业闸门

生命周期:**立项(demo)→ 升级(real-app 形态)→ 加固(hardening)→ 毕业(AutoOS)**。

五闸门全部满足才毕业:

1. **用途独立于语言特性**——一句话向最终用户描述用途且不出现 AutoLang 术语。
2. **能力闭环**——持久化、加载/错误/空三态、暗色模式、键盘可达、构建绿(Plan 354 的 DoD 是模板)。
3. **回归反哺**——毕业后长期作为平台 golden 消费者存活(见 §8),无反哺价值不毕业。
4. **宿主就绪**——AutoOS 能"安装"应用(pac 打包、auto-os-config 实体注册、窗口管理)。
5. **dogfooding**——有人日常真实使用。

**毕业动作**:迁出 `examples/ui/` → pac 定名 → 动作对齐 auto-os-config 实体 → MCP 测试纳入 CI → README/总表标"已毕业"。当前毕业第一梯队:auto-edit(041)、calculator(011)、calendar(016)、minesweeper(038)。

## 8. 渲染原语 golden 三件套(Plan 386 关联)

RenderCommand 协议(quad/text/image/clip/layer)完备性需要三类最严苛消费者,App 轨道恰好各孵化一个:

| 原语 | golden 应用 | 状态 |
|---|---|---|
| **文本 glyph** | 041-auto-edit(千级 glyph、按行高亮、高频局部重绘) | ✅ 已被 Plan 413 §7.6/386 点名 |
| **矢量** | 024-charts(曲线/多边形/轴文字) | 🆕 待立项 |
| **图像** | 029-image-viewer(解码/缩放/图层) | 🆕 待立项 |

三件套就位后,Plan 386 Stage 1 的验收样例从"编辑器单点"升级为"三原语全覆盖"。

## 9. 测试与验收惯例

- **MCP 桌面测试** `tests/desktop_mcp.py`(013 惯例):当前 011/013/015/038/041;新立项的核心应用默认配。
- **Playwright 冒烟** `tests/smoke.spec.ts`:015/017/018/022/023。
- **ATD 验收** `tests/acceptance.atd`(Plan 366 DSL):011/013/015/017/018/022/023。
- **双后端模式**(038 先例):核心应用立项即 vm + vue 双后端,同一份纯 AutoLang 逻辑跑两端;vm 端是 386 golden 的前提。

## 10. 计划结构

**一 app 一 plan,纲领总揽**:

- **纲领 plan**(下一个空闲号):跟踪矩阵覆盖度、填洞路线进度、空洞状态。对标 Plan 401 之于存量升级——401 管"已有示例升级",新纲领管"新增示例填补"。
- **每 app 独立 plan**(立项时从纲领派生,编号顺延):一 app 一 plan,大 app 内部分阶段(026 三阶段)。先例:405-023-realworld、407-038-rust-backend;auto-edit 更证明一个 app 会随成长自然长出多个 plan(413→414→418→420→428)。
- **存量加固**:成熟应用(015/017/018/022/023/038/041 等)各有遗留问题,先做一轮**逐 app 问题盘点**(对标 plans-status-audit 惯例,产出问题清单),再逐 app 立加固 plan,一个一个仔细优化——不与新示例填洞抢同一批 plan 编号,两条线并行。

**启动顺序**:024-charts 先行(025 的依赖,且是 golden 三件套的矢量件)。
