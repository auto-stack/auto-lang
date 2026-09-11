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
| 发布宣传主页 | `/zh/v05/` | `/v05/` | 一页式长滚动，8 大节（含三大设计理念节） |
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
   **发布规模统计直接并入 Hero**（用户裁定 2026-09-07）：`hide-code` 隐藏 hello world
   代码窗，默认插槽放 4 张 StatCard（1080px 出框横排）+ 量化足迹注解小字；
   不设"发布规模"节标题。
2. **三大设计理念**（2026-09-09 增补）：Hero 之后、AutoOS 之前——三张理念卡
   （动静相宜🌓 / 前后解耦🔌 / LAOS🖥️），每卡 = 名称 + 英文副题 + 金句 + 正文 + 3 支撑点 +
   机读 proof 条；Hero description 同步点名三理念。口径基准与素材见 §10。
3. **AutoOS 虚拟桌面**（ShowcaseSection + 截图/视觉）→ 段落尾部"了解 AutoOS →"（/zh/autoos/）。
4. **四大主打应用**（4 张 FeatureCards：AutoMusk 🤖 / AutoShell 🐚 / AutoDown 📄 / AutoUI Apps 🎨，
   各带一行定位语 + 链接到独立落地页）。
5. **新版 Playground**（ShowcaseSection：Debug 支持 + 几乎全部 Auto 示例集成；
   统计：语料笔记规模 1280+（vm-golden 460 / aavm 158 / 书页围栏 634 / demo 28））。
6. **语言进展**（三列）：
   - Rust 生态 **Alpha**：作为 Rust 的脚本语言，支持 90%+ Rust 代码调用；
   - **自举²**：(avm, a2r) × (aavm, aa2r) 2×2 自执行矩阵（specs GOAL-017：自举达成）；
   - Python 生态 **PreAlpha**：普通 Python 脚本 + PyTorch 支持。
   - 口径注：发布说明里 Rust=Beta/Python=Alpha 是能力面分级；宣传页按用户最新口径
     取保守分级（Alpha/PreAlpha），以用户大纲为准。
7. **v0.6 展望**（六张路线卡，按用户大纲）：
   AutoOS（独立 Linux 发行版 / 跨平台虚拟桌面 / AutoWeb 远程桌面）、
   AutoUI（鸿蒙 / Android+iOS via Jetpack Compose）、ROS2 生态、Godot 生态、
   MCU 生态、AutoAI（AutoMusk Beta / AI App 通讯架构）。
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

## 10. 理念层叙事口径基准（2026-09-09 增补；同日用户复调扩充）

**背景**：用户裁定 v0.5 宣传以三大独特创新点为魂——**动静相宜 / 前后解耦 / LAOS**，
已落地为宣传主页 Hero 之下的"三个设计理念"节（zh/en 同步，§4 第 2 节）。

**复调记录（2026-09-09 用户三条，已吸收进卡片文案）**：
1. **动静相宜不止 Rust 生态**（AutoVM 动态开发 / a2r 静态释出只是主战场），是**所有生态
   共用的设计原点**——UI 开发、MCU 嵌入式、Godot、科学计算等均按此模式赋能；且**着重强调
   动态态的热重载**（改代码不重启、状态不丢、所见即所改）。
2. **前后解耦是通用架构元模式**，不止 UI：auto-ai（前端=各类 Agent/AI-App，后端=ai-daemon
   统一调度 LLM 资源）；auto-os 应用架构（前端=auto-ui，后端=auto-compositor，RenderQueue
   通讯）；auto-os 顶层架构（前端=OS 外壳，后端=OS 内核——Windows 上=虚拟桌面+Windows 内核；
   未来 AutoOS 发行版=auto-os 桌面+Linux 内核直接耦合；后端还可切换 OpenHarmony）。
3. **LAOS 补充**：Auto 语言模拟 OS 架构，把各 OS 模块**语言化**——每个模块既可选择 Auto
   自己实现，也可通过生态桥调用现成实现（Windows 上：app/UI 转译 Rust/iced + 内核调用
   Windows 能力；MCU 里：app/UI 转译 C/lvgl + 内核换 Auto 自写 RTOS）。

| 理念 | 宣传一句话 | 事实源 |
|---|---|---|
| 动静相宜（Dynamic Dev · Static Ship） | 开发态 VM 解释执行（秒级周转/**热重载**/REPL/LSP）+ 发布态 a2r→Rust / a2c→C；一份源码两态同源；**全生态通用**（UI / MCU / Godot / 科学计算——strategy §0 动静矩阵即此表的对外版）；parity 机器守护一致性（三后端对拍全等、20+ 三方库复刻语料、automusk 前端 148 项全等） | [strategy/ecosystem-portfolio-strategy](../strategy/ecosystem-portfolio-strategy.md) §0（定为全生态设计原点）；[specs/parity](../../specs/parity/project.md)；[Design 28](../28-rust-interop-architecture.md) |
| 前后解耦（Decoupled at Every Layer） | 通用架构元模式，四层实例：①AutoUI↔渲染引擎（Vue/iced/ArkTS/Jetpack 可替换）②auto-ui↔auto-compositor（RenderQueue 共享内存无锁通讯）③Agent/AI-App↔ai-daemon（LLM 资源统一调度）④OS 外壳↔OS 内核（Windows 内核 → 未来 Linux 直接耦合 → 可切 OpenHarmony）；接缝两端可同栈（auto-down 411 文件/3.2 万行全 .at、automusk 五视图 148 对拍全等） | [Design 20](../20-autoui-separation-architecture.md)；[Design 08](../08-ui-systems.md)；[web-ecosystem-strategy](../strategy/web-ecosystem-strategy.md)；[Design 15](../15-ai-daemon-infrastructure.md)；[harmonyos-ecosystem-strategy](../strategy/harmonyos-ecosystem-strategy.md)（OpenHarmony 终态方向）；本文 §8 口径修正 |
| LAOS（Language as OS） | 语言模拟 OS 架构，OS 模块**语言化 + 双路供给**（Auto 自实现 ∥ 生态桥调用现成实现）：Actor=调度器、view/mut/move=内存管理、标准库=系统调用、渲染臂=外设驱动；装机示例——Windows=转译 Rust/iced+调用 Windows 内核能力，MCU=可转译 C/LVGL+Auto 自写 RTOS 内核；v0.5 现实=AutoOS 虚拟桌面（WM-as-App）+ auto-os-config + AutoAI Client/Daemon | [docs/LAOS.ms](../../LAOS.ms)；[Design 12](../12-concurrency.md)；[Design 04](../04-memory-ownership.md)；releases/v0.5 |

**诚实口径红线**（写宣传文案时必须区分已实现/设计中）：
- aillmd 全面形态（Key Vault/预算/fallback 路由）为 **Designed 待实施**——只宣传
  "Client/Daemon 架构已在 AutoAI 落地"，不宣传密钥托管。
- **前后解耦卡的四层实例时态分级**：AutoUI↔渲染臂（Vue/iced 已可用，ArkTS/Jetpack 为
  可行性 Demo 级）；auto-compositor/RenderQueue 为 Design 20 协议（386 复活承接）；OS 内核
  三形态中只有"Windows 虚拟桌面+Windows 内核"是现在时，Linux 直接耦合与 OpenHarmony 均
  终态方向——文案必须用"未来的 AutoOS 发行版里 / 还可以切换"等模态表述。
- **LAOS 卡的 MCU 装机例**（C/LVGL + Auto 自写 RTOS）为**路线图愿景**（C 轨暂缓中、LVGL
  为远期渲染臂）——用"可转译 / 可换成"模态，不用现在时；Windows 例是现在时。
- parity 语料数量取 **specs/parity 的 20+ 三方库**口径（overview 的 30+ 为宽口径，不采用）。
- `#[nopanic]`、`.!`、MCU 热重载、typed holes 均 Designed/Planned，禁止进入已实现叙事。

## 11. 候选创新点储备（后续宣传素材，含实现状态）

2026-09-09 全量扫描设计文档（01–16/20/28/29 + autoui/blocks/strategy + specs）所得，
按宣传硬度排序；供 v0.6+ 宣传页/发布会/README 取材。诚实口径同 §10 红线。

| # | 创新点 | 一句话主张 | 实现状态 | 事实源 |
|---|---|---|---|---|
| 1 | **自举²（AAVM）** | 用 Auto 写出编译 Auto 的编译器，再用 Auto 写出把它转译成 Rust 的转译器——自举回路里没有离不开 Rust 的芯（(avm,a2r)×(aavm,aa2r) 2×2 矩阵） | ✅ 达成（GOAL-017，代际对拍 8/8） | specs/aavm/project.md |
| 2 | **Parity 机器** | "解释器与编译器行为一致"不是承诺而是门禁：同一测试跑 VM/转译 Rust/原生 Rust 自动比对，20+ 三方库复刻语料常态回归 | ✅ 已实现（五向矩阵） | specs/parity/project.md |
| 3 | **AutoUI 活体 MCP** | Agent 不用截图猜界面：`autoui_vtree` 返回与渲染 1:1 的实时虚拟树（bbox/样式/事件/源码行号），配全套操作与像素截图双通道 | ✅ 已实现 | Design 14/15 |
| 4 | **编译预言机（Rust 互操作三时刻）** | "任何关于类型的问题都可以再编一段代码去问"——直面 Rust 无反射无稳定 ABI 的铁律，Auto 可调用 90%+ Rust 代码 | ✅ Beta（430 管线/1600 shim） | Design 28 |
| 5 | **AIE 增量编译 + AutoCache** | 编译器即数据库：声明级切片 + 接口哈希断级联 + 内容寻址缓存，改一个函数不惊动全仓库 | ✅ 已实现 | Design 09 |
| 6 | **AI 原生（Intent IR）** | 编译器为机器消费者设计输出：`--ai` JSON 结构化诊断（错误码+AST 路径+期望类型）供 Agent 自纠错；MCP `self_describe` 让 Agent 现场学会 Auto | ✅ 已实现（typed holes 规划中） | Design 09/16 |
| 7 | **修复轮次 N 评测法** | 不吹一键生成：Rung 0-5 能力阶梯 × M1-M6 基准 app，用"spec→green build 修复轮次"量化 AI 生成能力 | ✅ 体系在案 | Design 16 |
| 8 | **view/mut/move 三元组** | 成本写在语法上：三种 O(1) 访问模式 + 显式 clone 括号 = 视觉性能剖面图，AI 生成代码错误一眼可见 | ✅ 已实现 | Design 04 |
| 9 | **Task/Msg Actor + `.go`** | 语言级零共享并发：task 静态块编译期验证、`~T` 蓝图双向 RPC、`.go` 结构化微并发 | ✅ 已实现 | Design 12 |
| 10 | **三级错误 + 后缀算子** | `?T`/`!T`/裸 T 三分数据缺席/操作失败/逻辑炸弹，`.?`/`.!!` 口诀传播；`#[nopanic]` 静态证明绝不 panic | 大部分✅（nopanic Designed） | Design 03 |
| 11 | **Comptime 复用 VM** | 编译期与运行期共用同一个解释器：一个执行器两种时刻，元编程语义无分叉 | ✅ 已实现 | Design 09 |
| 12 | **三形态 enum + str/Str** | 一个 `enum` 统一 C 枚举/共享载荷/ADT 三形态；字符串按所有权分层自动单向提升 | ✅ 已实现 | Design 02 |
| 13 | **ABC 字节码 + 数字孪生 VM** | PC 上的 VM 即嵌入式模拟器：同一字节码跨 2KB RAM 单片机到桌面 | ✅ 已实现（MicroVM 三档） | Design 05 |
| 14 | **AutoDown Flip 方言** | Markdown+YAML 只加三个逃逸符，遇逻辑翻转为纯 Auto AST——文档即程序，多端转译 | ✅ 已实现（411 文件/3.2 万行） | raw/auto-down.md |
| 15 | **Blocks as Skills** | 中层组件的原子是"规格"不是代码：spec.md 机器可检契约 + 参考实现双产物，AI 用 widget 现场组装并自持源码 | 设计+示例在案 | blocks/blocks-first-class.md |
| 16 | **aillmd 共享 LLM Harness** | LLM 并发槽像 GPU 显存一样由系统 daemon 仲裁：应用零密钥、per-App 预算、优先级队列 | 📝 Designed（Client/Daemon 已在 AutoAI 落地） | Design 15 |
| 17 | **全 AI 研发范式** | 本仓即 AI 编程能力的可审计实证：1000 亿 token、5,711 commits、57.8 万行 Rust——AutoPlan 账本管线全程可溯 | ✅ 已实现 | autoplan-spec-ledger.md |

> 优先级建议（若精选 5 个做 v0.6 宣传）：①自举² ②Parity 机器 ③AutoUI 活体 MCP
> ④编译预言机 ⑤修复轮次 N 评测法——均有硬数据且为竞品空白。
