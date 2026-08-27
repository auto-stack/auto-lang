# Plan 458: AutoUI 主题体系 —— Theme + Accent 默认值统一、CLI/pac.at 正式入口、006 主题敏感示例

**Status**: Planned
**Scope**: `crates/auto-man/src/pac.rs`, `crates/auto/src/main.rs`, `crates/auto/src/cmd_vue.rs`, `crates/auto-man/src/vue.rs`, `crates/auto-man/src/rust_ui.rs`, `crates/auto-lang/src/ui/style/theme.rs`, `crates/auto-lang/src/ui/iced/renderer.rs`, `crates/auto-lang/src/ui_gen/vue.rs`, `examples/ui/006-hero-section`, `examples/ui/README.md`
**Goal**: 让"主题（Light/Dark）"与"主题主色（Accent）"成为 AutoUI 的一等配置：CLI 与 pac.at 可指定、双后端（Vue / VM-Iced）默认值一致、应用内可运行时切换；以 `examples/ui/006-hero-section` 为首个"主题敏感示例"，内置右上角 Settings 面板（主题 + 主色切换，对标 auto-musk `settings_menu.at`）。
**关联**: Plan 455（AutoUI 双端 parity 跟踪）、`docs/design/19-theming-and-dark-mode.md`（语义 token 体系）、`docs/design/22-base-styles-and-visual-parity.md`

---

## 1. 背景与根因

### 1.1 现象

`examples/ui/006-hero-section` 在 vue / vm 双端出现过"主文字接近白色、白色背景下不可读"的问题。经实测（当前 master 双端截图），006 在渐变正常铺满时是可读的；但结构缺陷真实存在，且 001/002 这类完全依赖默认主题的示例今天就处于双端不一致状态。

### 1.2 根因（三层缺失 + 一处分叉）

1. **VM(iced) 默认 Dark**：`crates/auto-lang/src/ui/style/theme.rs:18` 线程局部 `DARK_MODE = Cell::new(true)`（Plan 408 起），默认正文色为接近白的浅色。唯一覆盖出口是"约定变量名"：渲染器每帧回读应用根组件的 `dark_mode`（Bool）与 `accent_color`（Str）状态变量（`crates/auto-lang/src/ui/iced/renderer.rs:8227-8234`）。
2. **Vue 默认 Light**：示例实际使用的生成器 `crates/auto/src/cmd_vue.rs:1407` `generate_index_html` 产出的 `index.html` **不带** `class="dark"`，`:root` 浅色 token 生效（白底）。而 `crates/auto-man/src/vue.rs:844` 的同名函数（Plan 043 M5）**会**写 `class="dark"` —— **两份模板已分叉，是双端默认主题不一致的直接源头**。
3. **语言/示例/CLI 三层都没有 theme 概念**：主题只能靠暗约定（线程局部默认值、index.html 的 dark class、魔法状态变量名）。`examples/ui/001~041` 无一声明主题；颜色是"亮度特化"的硬编码（006 根节点 `text-white` + 渐变 `app.at:9`，可读性完全依赖渐变铺满整页这一隐含条件）。
4. **CLI 无入口**：`crates/auto/src/main.rs` 的 `run` 子命令（`:348`/`:369` 两个 Run 变体）只有 `-r/--render` 等，无 theme/accent；pac.at（`crates/auto-man/src/pac.rs:51` `Pac` 结构，`:97` window、`:101` title）也无 theme/accent 字段。

### 1.3 既有的可复用管道（本次不重造）

| 机制 | VM(iced) | Vue |
|---|---|---|
| 运行时切换 | 每帧回读根组件状态变量 `dark_mode`/`accent_color`（renderer.rs:8227） | 检测 `isDark`/`dark_mode` 状态变量 → 根节点 `:class="{dark: ...}"`（`crates/auto-lang/src/ui_gen/vue.rs:1393-1395`）；检测 `accent_color` → 注入 `applyAccent` 运行时写 `--primary`（ui_gen/vue.rs `ACCENT_PALETTES`） |
| 主题色板 | `theme.rs:82+` indigo/coral/ocean/sage/amber（HSL 三元组） | `ui_gen/vue.rs` `ACCENT_PALETTES` 同名同值 |
| 语义色随主题 | `resolve_semantic_rgb`（theme.rs:95-144）按 `DARK_MODE` 分支 | shadcn token（`:root` 浅 / `.dark` 深） |
| pac.at→进程注入 | `main.rs:944-950` `AUTO_VM_WINDOW`/`AUTO_VM_TITLE` env 注入模式 | — |

结论：**默认值层缺失（本次补齐）+ 运行时切换层已有（006 的 Settings 面板直接建在其上）**。

---

## 2. 设计决策

### 2.1 主题模型（v1）

- `ThemePref`：`"dark" | "light"`。**非目标**：`auto`（跟随系统）——VM 侧需 OS 探测、Vue 侧需 matchMedia 监听，列为后续增强（参考 auto-musk `visual_apply_theme` 的 auto 分支）。
- `AccentName`：`"indigo" | "coral" | "ocean" | "sage" | "amber"`（与双端既有色板同名同值，v1 不新增自定义色值）。
- **内置默认：`dark` + `indigo`**（示例生态默认 Dark；要浅色显式指定）。

### 2.2 配置来源与优先级

```
CLI --theme/--accent  >  pac.at theme:/accent:  >  内置默认 (dark/indigo)
        │
        └─ 应用内运行时切换（dark_mode/accent_color 状态变量）> 以上所有"初始默认值"
```

- 运行时层语义（现实现已满足）：根组件**未声明** `dark_mode` → `read_state` 返回 Err → 保留启动默认值；**声明了** → 应用首帧起接管（初始值即应用声明的初值，运行中可改）。
- 因此 pac.at/CLI 的值本质是"**初始默认主题**"：对声明了状态变量的应用，仅在首帧前生效；对未声明的应用（如 001/002），全程生效。

### 2.3 注入通道：env 变量（沿用既有模式）

`main.rs` run 处理器在解析 pac.at 后（现有 `AUTO_VM_WINDOW`/`AUTO_VM_TITLE` 注入点旁，main.rs:943-974）计算有效值并注入：

- `AUTO_UI_THEME = "dark" | "light"`
- `AUTO_UI_ACCENT = "indigo" | "coral" | "ocean" | "sage" | "amber"`

消费方：
- **VM**：`crates/auto-man/src/rust_ui.rs:2423` `run_vm_ui` 在调用 `run_file` 前读取 env，调用 `iced_adapter::set_dark_mode(bool)` / `set_accent_name(&str)`（theme.rs:23/:38），线程局部在首帧前生效。
- **Vue**：`cmd_vue.rs` 与 `auto-man/vue.rs` 的 index.html 生成逻辑读取同一 env（同进程，直接 `std::env::var`）。

选型理由：run 链路跨 auto → auto-man → auto-lang 三个 crate，env 是既有已验证的横切通道（Plan 347/340 同模式）；单进程内无污染问题。备选（拒绝）：`DesktopSession` 加字段——theme_fn 虽已接收 `&DesktopSession`（renderer.rs:8013-8016），但 `run_file`→`run_dynamic_iced` 的参数链更长，且 vue 侧无对应载体，v1 不做。

### 2.4 Vue 端模板统一（关键修复）

- `cmd_vue.rs:1407` 与 `auto-man/vue.rs:844` 的 `generate_index_html` 统一为接受 `(name, theme, accent)` 参数的版本：
  - `theme == "dark"` → `<html lang="en" class="dark">`；light → 无 class。
  - accent → 内联 `<style>:root{--primary:<hsl>;} .dark{--primary:<hsl-dark-boost>;}</style>` 写入 head（hsl 值取自与 `ACCENT_PALETTES` 同源的表；**表抽为单一常量**供 cmd_vue / auto-man-vue / ui_gen 三处引用，消除第二次分叉机会）。
  - 应用已声明 `accent_color` 状态变量时，ui_gen 注入的 `applyAccent` 运行时会在 mount 后以内联样式覆盖，两者兼容；仍写内联默认，保证"未声明 accent 的应用"也能吃到 pac.at/CLI 值。
- **每次 `auto run`（vue 路径）都重写 `index.html`**（文件极小，规避"项目已存在即跳过"的 stale 缓存问题——当前 006 的 `gen/front/vue/index.html` 正是旧模板产物）。`generate_index_css` 不动（`:root`/`.dark` 双 token 集已完整）。

### 2.5 VM 端一致性小修

- `shadcn_theme`（renderer.rs:3864 附近）的 `Palette::primary` 目前硬编码 indigo-500：改为从 `ACCENT_NAME` 色板取值（经 `resolve_semantic_rgb` 同源换算），保证 CLI/pac.at 指定的 accent 对 iced 窗口级 primary（按钮默认底色等）同样生效。

### 2.6 006 示例设计（首个主题敏感示例）

保持单文件 `src/front/app.at`（不拆 store/子组件文件，规避跨组件状态可见性不确定性——`read_state`/vue 检测均面向根组件）：

- App 增加根级状态：`var dark_mode bool = true`、`var accent_color str = "indigo"`（**示例默认 Dark**，初值即默认主题；两者被双端运行时钩子识别）。
- App `msg` 增加 `SetTheme(str)`、`SetAccent(str)`；处理器写状态变量即可（VM 每帧回读生效；Vue 响应式 class + applyAccent 生效）。
- 右上角 Settings 面板（内联在 App view，绝对定位）：
  - 齿轮/设置按钮 → 展开 220px 圆角面板（样式对标 auto-musk `settings_menu.at` 的 Theme/Accent 两节，砍掉 i18n/forge/language/AutoOS）。
  - **Theme 节**：Light/Dark 两钮（active 态高亮），点击 `.SetTheme("light"/"dark")`。
  - **Accent 节**：5 个色板圆点（背景色 = 各 brand1），active 加勾/描边，点击 `.SetAccent(name)`。
  - 面板自身样式全部用语义 token（`bg-card`/`text-card-foreground`/`border` 等），保证在两种主题下都可读。
- 页面本体改为**主题条件样式**（view 已支持 `class: if cond {}`）：
  - Dark：保持现有渐变（`from-blue-500 to-purple-600`）+ `text-white`。
  - Light：浅底（如 `bg-gray-50`）+ 深字（`text-gray-900`）。
  - 可读性由构造保证：文字色与底色永远成对出现在同一分支。
- pac.at 增加 `theme: "dark"`、`accent: "indigo"`（显式声明，作示例示范；与状态变量初值一致）。

### 2.7 示例生态约定

- `examples/ui/README.md` 新增"主题约定"节：示例默认 Dark；`auto run --theme light`（或 pac.at `theme: "light"`）查看浅色；006 为主题敏感示例（内置切换面板）；后续逐步把硬编码亮度色迁移为语义 token（独立后续 plan，见 §5）。

---

## 3. 任务分解

### T1 配置入口：pac.at + CLI
- [ ] `crates/auto-man/src/pac.rs`：`Pac` 增加 `theme: Option<String>`、`accent: Option<String>` 字段与解析（校验合法值，非法值告警并回落默认）。
- [ ] `crates/auto/src/main.rs`：`run` 子命令（两个 Run 变体，:348/:369）增加 `--theme`/`--accent` 参数。
- [ ] main.rs run 处理器：按 §2.2 优先级计算有效值，注入 `AUTO_UI_THEME`/`AUTO_UI_ACCENT`（紧邻现有 AUTO_VM_WINDOW/TITLE 注入点）。
- [ ] 验证：`cargo check -p auto-man -p auto`；手工 `auto run examples/ui/001-helloworld --theme light` 观察 env 生效路径（T2/T3 完成后回归）。

### T2 VM(iced) 后端：首帧前默认值注入
- [ ] `crates/auto-man/src/rust_ui.rs` `run_vm_ui`：读取 `AUTO_UI_THEME`/`AUTO_UI_ACCENT`，在 `run_file` 前调用 `set_dark_mode`/`set_accent_name`。
- [ ] `crates/auto-lang/src/ui/iced/renderer.rs` `shadcn_theme`：Palette.primary 改为 accent 派生（§2.5）。
- [ ] 确认 `DARK_MODE` 线程局部初值语义不变（未注入时仍为 dark，兼容既有 `cargo tv` VM 文件测试）。
- [ ] 验证：`cargo check -p auto-lang`；`auto run examples/ui/001-helloworld -r vm --theme light --accent ocean` → 窗口浅底深字、主色 ocean；`-r vm`（无参）→ 深底浅字、indigo；001 声明 `dark_mode` 状态变量后运行时可覆盖（可选子实验）。

### T3 Vue 后端：模板统一 + accent 内联
- [ ] 抽取共享 accent 色板常量（cmd_vue.rs / auto-man/vue.rs / ui_gen/vue.rs 三处单一来源；允许 `pub` 常量跨 crate 引用或小工具模块）。
- [ ] `cmd_vue.rs` `generate_index_html` 参数化（§2.4）；确认/改造调用链保证**每次 run 重写 index.html**（含 stale 006 产物自愈）。
- [ ] `auto-man/vue.rs` `generate_index_html`（:844）同步参数化，消除 `class="dark"` 分叉。
- [ ] 验证：`auto run examples/ui/006-hero-section`（vue）→ 检查 `gen/front/vue/index.html` 含 `class="dark"` 与 `--primary` 内联样式；`--theme light --accent coral` → 无 dark class、coral `--primary`；页面 `body` 底色随 token 变化。

### T4 006 主题敏感示例（Settings 面板）
- [ ] `app.at`：根级 `dark_mode`/`accent_color` 状态变量 + `SetTheme`/`SetAccent` 消息（§2.6）。
- [ ] 右上角 Settings 按钮 + 弹出面板（Theme 两钮 + Accent 五色点，语义 token 样式）。
- [ ] 页面本体主题条件样式（Dark 渐变+白字 / Light 浅底+深字）。
- [ ] pac.at 显式 `theme: "dark"`、`accent: "indigo"`。
- [ ] 验证：双端手动走查——切换 Light/Dark、切换 5 色、面板自身在两主题下可读、CTA 按钮可读。

### T5 双端回归矩阵 + 文档
- [ ] autoui-verifier 截图矩阵：006 = {dark, light} × {vue, vm} + accent 切换各 1 张（共 ≥10 张）。
- [ ] 001、002 双端各 1 张回归（未声明主题的应用在 `--theme light` 下的表现）。
- [ ] `examples/ui/README.md` 增加主题约定节（§2.7）。
- [ ] Plan 455 矩阵中 006 行追加"Theme Settings"验证状态。

### 明确不做（v1 非目标）
- `auto`（跟随系统偏好）模式与持久化（storage）。
- 其余示例（004/005/007/008/038 等）硬编码亮度色的语义 token 迁移 —— 独立后续 plan（按 `docs/design/19` 迁移表执行）。
- `DesktopSession` 主题字段重构、`.at` 语言级 theme 语法。

---

## 4. 验证方案（按改动范围分级门禁）

- **Category B（局部 Rust 改动）**：`cargo check -p auto-lang`、`cargo check -p auto-man -p auto`；`cargo t ui`（含 iced/style 模块）。合入前跑一次 `cargo t`。
- **AutoUI 双端**：autoui-verifier 技能（`test_vue_playwright.mjs` + `test_vm_mcp.py`）：
  - 006：`{--theme dark, --theme light} × {vue, vm}` 初始态 + 面板展开态 + 切 accent 后各一张。
  - 001/002：默认 + `--theme light` 双端各一张。
- **不涉及** docs_gen / Schema（无文档生成器改动）。
- **独立复审清单**：§3 各任务勾完；扫描临时 hack（如硬编码 hex 绕过色板表）；零 compiler warning；无 stray dbg 打印。

---

## 5. 风险与开放问题

| 风险 | 缓解 |
|---|---|
| Vue 端 `.dark` 绑定在根 div 而非 `<html>`，`.dark` token 与 `:root` 的覆盖顺序 | 生成 CSS 中 `.dark` 定义在 `:root` 之后（现状如此）；T5 用 015/018（既有 dark 切换示例）与 006 实测确认 |
| 双份 `generate_index_html` 统一后仍有第三处模板（cmd_tauri）漂移 | T3 色板常量先收敛；cmd_tauri 列入后续 plan 一并参数化 |
| stale `gen/front/vue` 缓存吞掉新 index.html | §2.4 强制每次重写；T5 含 006 旧产物自愈验证 |
| `read_state("dark_mode")` 无法区分"未声明"与 `false` | 现语义即"未声明 → Err → 保留启动默认"，满足需求；在 renderer.rs 注释固化该契约 |
| 001 等示例若将来声明 `dark_mode` 会覆盖 CLI 默认 | 优先级已在 §2.2 文档化，属预期行为（应用内切换 > 启动默认） |

---

## 6. 施工与合入

1. worktree：`git worktree add .worktree/plan-458 -b plan-458`。
2. 任务顺序 T1 → T2/T3（可并行）→ T4 → T5；每任务一 commit（`feat(plan-458): T<n>-<x> …`）。
3. 完成 §4 门禁 + 独立复审后，按 AGENTS.md 流程归档本文件并合入 master（`feat(auto-ui): theme system — CLI/pac.at entry, dual-backend default unification, 006 theme-aware example (Plan 458)`）。

## 7. 验证记录（施工时填写）

- [ ] T1-T5 逐项验证结果
- [ ] 截图矩阵路径汇总
- [ ] 独立复审结论
