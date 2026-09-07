# v0.5 发布宣传页（Release Promo Landing）

> **状态**：executing（2026-09-07 立项，用户口述大纲 → 本设计）
> **来源**：2026-09-07 用户需求——"v0.5 现已发布" badge 需要一个跳转落地页；
> 与 Release Note（markdown 信息记录）不同，这是**发布宣传页面**，要漂漂亮亮、含大量截图。
> **关联**：`docs/releases/v0.5.md`（发布说明，事实基准）、
> [specs/website](../../specs/website/project.md)（站点现状）、
> [autoui/virtual-desktop.md](../autoui/virtual-desktop.md)（虚拟桌面架构）、
> [specs/aavm/project.md](../../specs/aavm/project.md)（自举² 事实源）。

## 1. 目标与非目标

**目标**：
1. 首页 Hero badge（"v0.5 现已发布" / "v0.5 is now available"）可点击，跳转到发布宣传页。
2. 宣传页 = 一页式长滚动 landing，风格与 Home/Rust/Python/AI/OS/Apps 展示页族完全一致
   （`layout: home` + landing.css 共享类 + Hero 组件族 + ShowcaseSection/StatCard/FeatureCard）。
3. AutoOS（虚拟桌面）与 4 大主打应用各有**独立段落**（宣传页内）+ **独立落地页**（集成进站点）。
4. 中英双语同步（站点既有双语纪律；en 为 zh 的镜像翻译）。

**非目标**：
- 不改 `docs/releases/v0.5.md` 发布说明本身（两表并存：宣传页链接它作为"完整清单"）。
- 不引入新的构建依赖；纯 markdown + 现有主题组件 + 少量 scoped CSS。
- 不做逐像素动画重工事效；复用 landing.css 的 reveal-on-scroll 体系。

## 2. 信息架构与路由

| 页面 | zh 路由 | en 路由 | 说明 |
|---|---|---|---|
| 发布宣传主页 | `/zh/v05/` | `/v05/` | 一页式长滚动，7 大节 |
| AutoOS 虚拟桌面 | `/zh/autoos/` | `/autoos/` | 独立落地页（产品页口径，区别于 `/zh/os` 架构页） |
| AutoMusk | `/zh/apps/automusk/` | `/apps/automusk/` | 4 大应用独立落地页 |
| AutoShell | `/zh/apps/autoshell/` | `/apps/autoshell/` | „ |
| AutoDown | `/zh/apps/autodown/` | `/apps/autodown/` | „ |
| AutoUI Apps | `/zh/apps/autoui/` | `/apps/autoui/` | „ |

- 目录 `index.md` 形式（cleanUrls → `/zh/v05/`），文件放 `website/zh/v05/index.md` 等。
- **nav 登记**（`.vitepress/config/zh.ts` + `en` 同构）："v0.5" 下拉首项插入
  "v0.5 发布专题"（→ `/zh/v05/`），发布说明退居其后。
- **badge 跳转**：`HomeHero.vue` 增加可选 prop `badge-link`，有值时 badge 渲染为 `<a>`；
  zh/en 首页传入 `/zh/v05/`、`/v05/`。
- `/zh/apps`（应用矩阵页）四大应用的卡片标题链接到对应新落地页（互链闭环）。

## 3. 素材与视觉策略

| 视觉 | 方案 |
|---|---|
| AutoUI Apps 页 | **活体嵌入**：`<iframe src="/ui/gallery/">`（public/ui/gallery SPA，无需后端）——比截图更震撼；辅以 Widgets Gallery 数据（46+ 组件、24 区块） |
| AutoOS 虚拟桌面页 | 首选真实截图（`auto-os` 仓桌面 shell 运行图，盘点见 §8）；缺席时以 arch-diagram/path-card 风格的 CSS 视觉替位，留 `screenshots/` 约定路径后续补 |
| 4 大应用页 | 首选各仓截图；缺席时 code-window 终端拟真视觉（automusk/shell 天然适合终端风）+ FeatureCard/StatCard |
| Playground 节 | 宣传页内嵌 NotesExplorer 已有能力的描述性视觉（数字卡：语料规模），链接 `/zh/playground` |
| 语言进展节 | 三列 StatCard/FeatureCard（Rust Alpha / 自举² / Python PreAlpha）+ 自举² 用 arch-diagram 风格 2×2 矩阵图 |

## 4. 宣传主页分节设计（`/zh/v05/`）

1. **Hero**（HomeHero 复用，badge="有史以来最大的一次更新"）：
   title="：v0.5 正式发布"；description 一句话定位 + 双 CTA（阅读发布说明 / 打开 Playground）。
2. **发布规模统计带**（stats-grid，4 张 StatCard）：
   `1000 亿` AI 研发 token · `5,700+` commits（v0.3 以来，实测 5,711）·
   `57.8 万` 行 Rust 代码 · `13.5 万` 行 Auto 代码。
3. **AutoOS 虚拟桌面**（ShowcaseSection + 截图/视觉）→ 段落尾部"了解 AutoOS →"（/zh/autoos/）。
4. **四大主打应用**（4 张 FeatureCard：AutoMusk 🤖 / AutoShell 🐚 / AutoDown 📄 / AutoUI Apps 🎨，
   各带一行定位语 + 链接到独立落地页）。
5. **新版 Playground**（ShowcaseSection：Debug 支持 + 几乎全部 Auto 示例集成；
   统计：语料笔记规模 1280+（vm-golden 460 / aavm 158 / 书页围栏 634 / demo 28））。
6. **语言进展**（三列）：
   - Rust 生态 **Alpha**：作为 Rust 的脚本语言，支持 90%+ Rust 代码调用；
   - **自举²**：(avm, a2r) × (aavm, aa2r) 2×2 自执行矩阵（specs GOAL-017：自举达成）；
   - Python 生态 **PreAlpha**：普通 Python 脚本 + PyTorch 支持。
   - 口径注：发布说明里 Rust=Beta/Python=Alpha 是能力面分级；宣传页按用户最新口径
     取保守分级（Alpha/PreAlpha），以用户大纲为准。
7. **v0.6 展望**（五张路线卡，按用户大纲）：
   AutoOS（独立 Linux 发行版 / 跨平台虚拟桌面 / AutoWeb 远程桌面）、
   AutoUI（鸿蒙 / Android+iOS via Jetpack Compose）、ROS2 生态、Godot 生态、MCU 生态。
8. **CTA**：快速开始 / 在线体验 / GitHub。

## 5. 独立落地页设计

每页统一骨架（对应展示页族模式）：`Hero（含该产品 badge/定位语）→ 统计带 →
Showcase×2-3（特性/架构/视觉）→ FeatureCard 网格 → CTA`。

- **AutoOS 虚拟桌面**（accent 沿用 os 页 teal/blue）：WM-as-App 一致性故事、
  单 OS 窗口虚拟合成器拓扑、跨平台路线（Win/Linux/鸿蒙）、系统应用矩阵预告、截图位。
- **AutoMusk**（pink/purple，同 ai 页）：AutoPlan 驱动 / 多提供商 / 自托管（Auto 写的 Agent）/
  auto-os-config 配置；终端拟真视觉。
- **AutoShell**（amber/blue）：结构化管道 / F3 AI 模式 / CLI·TUI·GUI 三形态 / 跨平台。
- **AutoDown**（indigo/violet）：Markdown+YAML 方言 / 可解析知识库 / 与工具链同栈。
- **AutoUI Apps**（violet/pink）：活体 gallery iframe + Demo 矩阵 + 桌面/移动形态。

## 6. 数字口径表（宣传引用基准）

| 数字 | 值 | 来源 |
|---|---|---|
| AI 研发 token | 1000 亿 | 用户给定（2026-09-07） |
| commits since v0.3 | 5,711（tag v0.3 = 2026-04-12） | `git rev-list --count v0.3..HEAD` 实测 |
| Rust 行数 | 578,238（全仓 .rs，宣传取"57.8 万+"） | `git ls-files '*.rs' | cat | wc -l` |
| Auto 行数 | 135,295（全仓 .at，宣传取"13.5 万+"；非测试语料 92,502；自举库 auto/lib 14,873） | 同法实测 |
| Playground 语料 | 1280+ 笔记（460 vm-golden + 158 aavm + 634 书页围栏 + 28 demo） | specs/website Plan 581 |
| Widgets Gallery | 46+ 组件 / 24 区块 | zh/apps.md 在案 |

## 7. 双语策略

zh 先行定稿，en 逐页镜像翻译（同构文件树：`website/index.md` 侧为根路径）。
badge-link 同步接线。en 文案以 zh 语义为准，不逐字直译。

## 8. 截图盘点（调研结论，2026-09-07）

两路 Explore 调研（仓内全库 + autostack 兄弟仓）结论与取材：

**已拷入 `website/public/v05/`（14 张）：**

| 文件 | 来源 | 用途 |
|---|---|---|
| desktop-hero.png | auto-lang `tmp/autoui-screenshot-1788442589161.png` | **虚拟桌面主视觉**（任务栏+开始菜单+DualApp/计算器多窗口，最成品感） |
| desktop-multiwindow.png | `scratch/visual-gap/vm/multiwindow.png` | 多窗口 |
| desktop-light.png | `docs/plans/attachments/559/559-vm-03-appearance-light.png` | 浅色主题 |
| desktop-launcher.png | `scratch/visual-gap/vm/launcher_open.png` | 启动器（备用） |
| autoos-config-agents/skills.png | auto-os-config 根 `screenshot-*.png` | 设置中心 |
| kanban-web/desktop.png | auto-kanban `screenshots/t16_*` | 首个 AutoOS 应用双轨 |
| automusk-app.png | auto-lang `scratch/p536_t7_musk_float.png` | AutoMusk 桌面主界面（v0.1.0） |
| automusk-workspace/plans.png | auto-musk `docs/attachments/p059-*.png` | Musk Web 工作台 |
| autodown-desktop.png | auto-down `autodown/showcase/auto/vm-059-light.png` | Jade Garden 桌面形态 |
| autodown-web.png | 同目录 `vue-059-t5-feed.png` | Jade Garden Web 形态 |
| gallery-home.png | auto-lang `scratch/p561/p561_vm_gallery_home.png` | 画廊 VM 形态（AutoUI 页副视觉） |

**素材缺口（后续补截）**：AutoShell（auto-shell 仓 `gui-test-screenshots/` 为空——本版用
code-window 终端拟真视觉替位）、Playground（无现成图——本版用统计卡视觉）。
**优化候选**：desktop-hero/light/multiwindow 均为 1.3–1.7MB 原始 PNG（本机无
ImageMagick），后续可加压缩管线（sharp/squoosh）。

**口径修正（调研所得，宣传页已采纳）**：auto-shell 本体是 Rust（AutoLang 为其内置
脚本语言、GUI 用 .at）——文案不说"用 Auto 写的 Shell"，说"AutoLang 是 ASH 的内置
脚本语言"；auto-musk 后端为 Rust、前端 5 视图 .at 单源生成（148 项对拍全等）；
auto-down 是"前后端全 .at 单源"的最强案例（411 文件/3.2 万行）。

## 9. 验证方案

- dev server（:4399）逐页走查 zh + en；对照展示页族检查 hero/统计/CTA 样式一致性。
- `npm run build` 全站编译通过（防裸 `<` 等 Vue 模板坑回归）。
- badge 点击链路：/zh/ badge → /zh/v05/ → 五个落地页互链无 404。
