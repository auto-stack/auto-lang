# 22 - AutoUI Base Styles and Cross-Backend Visual Parity Specification

> 📦 **归位注记（2026-08-28，Plan 468）**：本文档原为 `docs/design/autoui/base-styles-and-visual-parity.md`（Design 22），经审计属需求级/专题类设计而非域级章，按模块归位原则移入autoui/。历史文献中的“Design 22”即指本文。

## 1. Overview & Design Philosophy

AutoUI provides a unified UI description language (`.at` widgets with AURA IR) that compiles to multiple execution targets:
- **Web / Vue**: Vue 3 + Tailwind CSS + Shadcn UI
- **Desktop VM**: AutoVM + Iced native renderer (`auto run -r vm`)
- **Native Rust**: Transpiled native Iced desktop application (`a2r` / `auto run -r rust`)
- **Android**: Jetpack Compose (`a2jet`)
- **HarmonyOS**: ArkTS (`a2ark`)

### 1.1 Why Not Use 1990s Browser User-Agent Stylesheets?

Traditional browsers apply default User-Agent styles to HTML headings (`font-size: 2em`, `margin: 0.67em 0`, unoptimized letter spacing). These defaults were designed for static document rendering in the 1990s. In modern application interfaces:
1. Arbitrary margin-collapsing disrupts Flexbox and Grid layouts (e.g., `center { h1 "..." }` will be vertically offset).
2. Large headings without negative letter-spacing (`tracking-tight`) appear loose and unpolished.
3. Plain black text does not integrate with the design token / theme system (`--primary` HSL colors, dark mode).

Modern application design systems (Tailwind CSS, Shadcn UI, Apple HIG, Material Design 3) reset unstyled HTML elements and apply structured, proportional typography scales and form presets.

### 1.2 The Single Source of Truth for Default Styles

To achieve **Visual Parity** across all targets, AutoUI establishes a standard typography and base element scale. All backends (Vue, VM, Rust, Jet, Ark) adhere to this unified specification.

---

## 2. Typography Scale Specification (h1 ~ h6)

| Tag | Token / Class | Size (px) | Weight | Spacing & Color | Semantic Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`h1`** | `text-4xl` | 36px (2.25rem) | Bold (`font-bold` / 700) | `tracking-tight text-primary mb-4` | Page Title / Hero Title |
| **`h2`** | `text-3xl` | 30px (1.875rem) | Bold (`font-bold` / 700) | `tracking-tight text-primary mt-8 mb-4` | Section Heading |
| **`h3`** | `text-xl` | 20px (1.25rem) | Semibold (`font-semibold` / 600) | `text-primary mb-3` | Subsection / Card Title |
| **`h4`** | `text-lg` | 18px (1.125rem) | Semibold (`font-semibold` / 600) | `mb-2` | Group Title |
| **`h5`** | `text-base` | 16px (1.0rem) | Semibold (`font-semibold` / 600) | `mb-1` | Highlight Label |
| **`h6`** | `text-sm` | 14px (0.875rem) | Semibold (`font-semibold` / 600) | `mb-1` | Small Header / Eyebrow |
| **`p` / `text`** | `text-base` | 16px (1.0rem) | Normal (`font-normal` / 400) | `leading-7 text-muted-foreground` | Body Text |

---

## 3. Form Controls & Interactive Primitives Defaults (Shadcn Parity)

When no custom `style` or `class` is specified in AutoUI source files, form elements and controls adopt standard Shadcn-Vue design presets:

| Component | Default Preset (Tailwind & Shadcn Token Equivalent) | Visual Properties (VM / Iced Target) |
| :--- | :--- | :--- |
| **`input`** | `border border-input bg-background rounded-md px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground` | Border: 1px `resolve_border_rgb()`; Radius: 6px; Bg: `Color::Background`; Text: `Color::OnBackground`; Placeholder: `Color::OnSurface`; Padding: 12px H, 8px V |
| **`textarea`** | `border border-input bg-background rounded-md px-3 py-2 text-sm text-foreground min-h-[80px]` | Border: 1px `resolve_border_rgb()`; Radius: 6px; Bg: `Color::Background`; Text: `Color::OnBackground`; Min-Height: 80px |
| **`button` (default/primary)** | `bg-primary text-primary-foreground font-medium rounded-md h-10 px-4 text-sm` | Bg: `Color::Primary` (accent-driven); Text: `Color::OnPrimary`; Radius: 6px; Height: 40px; Padding: 16px H |
| **`button` (secondary)** | `bg-secondary text-secondary-foreground font-medium rounded-md h-10 px-4 text-sm` | Bg: `Color::Secondary`; Text: `Color::OnSecondary`; Radius: 6px; Height: 40px |
| **`button` (destructive)** | `bg-destructive text-destructive-foreground font-medium rounded-md h-10 px-4 text-sm` | Bg: `Color::Error` (red-600); Text: White; Radius: 6px; Height: 40px |
| **`button` (outline)** | `border border-input bg-background text-foreground rounded-md h-10 px-4 text-sm` | Border: 1px `resolve_border_rgb()`; Bg: `Color::Background`; Radius: 6px; Height: 40px |
| **`button` (ghost)** | `rounded-md h-10 px-4 text-sm hover:bg-accent` | Transparent bg; Radius: 6px; Height: 40px |
| **`button` (icon)** | `h-7 w-7 px-0 py-0 rounded-md` (or `h-10 w-10`) | Square aspect ratio; centered icon |
| **`checkbox`** | `h-4 w-4 rounded border border-primary` | Border: 1px `Color::Primary`; Radius: 4px; Size: 16x16px |
| **`badge`** | `inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold` | Radius: 9999px (full); Padding: 10px H, 2px V; Font: 12px semibold |

---

## 4. Base Containers & Border Color Fallback

| Tag / Utility | Default Styling | Border Fallback Rule |
| :--- | :--- | :--- |
| **`col` / `column`** | `flex flex-col gap-4` | Layout primitive |
| **`row`** | `flex flex-row gap-4` | Layout primitive |
| **`center`** | `flex flex-col items-center justify-center h-full` | Centering container |
| **`grid`** | `grid` | Grid layout |
| **`scroll`** | `overflow-auto` | Scrollable viewport |
| **`container`** | `max-w-7xl mx-auto` | Content max-width wrapper |
| **`border` utility** | `border-width: 1px` | **Fallback border color**: Must resolve to `resolve_border_rgb()` (`--border`: zinc-800 in Dark Mode, zinc-200 in Light Mode). Never fallback to transparent. |
| **`bg-card`** | `Color::Surface` | Surface container (`hsl(222.2 47.4% 10%)` in Dark Mode, gray-50 in Light Mode). |

---

## 4.5 Markdown Renderer Internal Block Rhythm（markdown 块间节奏，2026-09-02 新增）

> 来源：auto-musk PLAN-056 T6（Block 全家福演示实机对拍）。`.at` 视图无法表达
> 第三方/内置 markdown 渲染器的内部 DOM，故本节为**默认样式规约**：Vue 侧已在
> musk `inject_styles.web-only.ts` 实现，VM 侧 markdown 渲染器按本节对齐。

**规约**：markdown 渲染输出的相邻顶层块之间保持 `0.75rem`（12px）垂直间距。

- Vue 实现锚点：`.streaming-document .markdown-renderer > .node-slot + .node-slot { margin-top: 0.75rem }`
  （`@autodown/vue` 0.2.0 快照缺 markstream 的 slot 间距段，且宿主 tailwind preflight
  清零元素默认 margin，必须显式补齐；上游自带的相邻 slot 内容边缘剥边规则
  `:first-child{margin-top:0!important}` / `:last-child{margin-bottom:0!important}`
  继续防止双倍间距）。
- VM 实现方向：markdown 渲染器逐块排版时，块与块之间留 12px 垂直间距（首块前/
  末块后不加），等价 Visual Parity。
- 段内行距不受影响：`p` 内部行距 `leading-7`、段落自身 `margin: .5rem 0`（vendor
  自带）维持现状。

## 4.6 Markdown Renderer Dark Theme Color Mapping（markdown 暗色主题，2026-09-02 登记）

> 来源：auto-musk PLAN-054 B1 + PLAN-056（`.dark` 对拍现场）。vendor style.css
> 硬编码浅色 token，暗色主题下必须改挂主题变量。

**规约**（`.dark` 域内）：

| 元素 | 颜色 |
| :--- | :--- |
| 正文 / 标题 h1-h3 / 表格单元格 td,th / 代码块内文字 / details summary | 主题 `--foreground` |
| 表格表头 th、代码块 pre 背景 | 主题 `--muted`（约 50% 透明度叠加） |
| 行内 code 背景 | `--muted / 0.5`，边框 `--border` |
| blockquote 边框 | `--border`，文字 `--foreground` |
| admonition | 保色相、明度降至暗底可读 |

Vue 实现锚点：musk `inject_styles.web-only.ts` `.dark .streaming-document …` 规则组
（vendor scoped data-attr 特异性打平，靠注入顺序取胜）。VM 实现：按同一映射挂
VM 主题 token。

---

## 5. Multi-Backend Implementation Architecture

### 5.1 Vue / Web Implementation
1. **`@layer base` in global CSS (`index.css` / `generate_base_css`)**:
   Injects the base typography rules so any raw HTML `<h1..h6>` tag receives standard AutoUI typography even if utility classes are stripped or customized:
   ```css
   @layer base {
     h1 { @apply text-4xl font-bold tracking-tight text-primary mb-4; }
     h2 { @apply text-3xl font-bold tracking-tight text-primary mt-8 mb-4; }
     h3 { @apply text-xl font-semibold text-primary mb-3; }
     h4 { @apply text-lg font-semibold mb-2; }
     h5 { @apply text-base font-semibold mb-1; }
     h6 { @apply text-sm font-semibold mb-1; }
   }
   ```
2. **Cascading Order**:
   Because `@layer base` has lower specificity than utility classes, any explicit `class: "text-sm font-normal"` supplied by the user will override base rules cleanly.
3. **Component Generation**:
   Generates Shadcn-Vue SFC components (`Input.vue`, `Button.vue`, `Textarea.vue`) with integrated `@apply` and Tailwind utility classes.

### 5.2 VM / Iced Implementation (`aura_view_builder.rs` & `renderer.rs`)
1. **View Builder Preset Injection (`aura_view_builder.rs`)**:
   When converting `input`, `textarea`, `button`, or `h1..h6`, the builder merges default preset classes with user-specified classes:
   ```rust
   let user = self.extract_string_with(props, "class", bindings)
       .or_else(|| self.extract_string_with(props, "style", bindings));
   let default_preset = "border rounded-md bg-background px-3 py-2 text-sm";
   let merged = match user.as_deref() {
       None => default_preset.to_string(),
       Some(c) => format!("{} {}", default_preset, c),
   };
   let style = Style::parse(&merged).ok();
   ```
2. **Container Border Color Resolution (`renderer.rs`)**:
   When `is.border` or `is.border_width > 0` is set without an explicit color, `is.border_color` falls back to `resolve_border_rgb()` instead of `TRANSPARENT`.
3. **Input Styling (`renderer.rs`)**:
   Applies background (`Color::Background`), border (1px `resolve_border_rgb()`), border radius (6px `rounded-md`), padding (`px-3 py-2`), font size (14px `text-sm`), and foreground colors.

### 5.3 Rust Native Implementation (`rust.rs`)
1. In `heading_default_style()` and component generators, prepends standard default classes to user-provided styles.

---

## 6. Verification & Testing

Every visual parity change must be tested against:
1. `auto run` (Vue dev server @ port 30xx)
2. `auto run -r vm` (Iced native window)
3. `examples/ui/001-helloworld` (Single heading smoke test)
4. `examples/ui/003-converter` (Inputs, bidirectional binding, and card border)
5. `examples/widgets-gallery` (Full typography & component catalog)

---

## 7. AutoDown Document Face Specification（引擎文档面规约，PLAN-051 立章，2026-09-04）

> 来源：auto-down PLAN-051（vue 轨 view 正确性收口 + 主题规约化）。§4.5/§4.6
> 先例的**引擎域**延伸：`.at` 视图与 markdown 渲染器都无法表达 autodown 引擎
> 组件（EngineEditor / StreamingRenderer / 家族 widget）内部 DOM 的样式，本节
> 为引擎文档面的默认样式规约——**取值单源于本节**，vue 侧实现锚点（engine
> CSS）与 VM 侧实现锚点（`autodown_blocks.rs` 常量/类串、`hljs_scope_map.rs`
> 生成物）都只做投影。排查纪律：VM/vue 分叉时**先对表**（本节行值），再查
> 两侧锚点实现；无规约行对应的语义值出现在任一侧实现里即为违规（PARITY #13
> 事故模式）。
>
> 主题状态入口（声明层）：`dark_mode`（bool，app model 状态，VM 轨经 D-GAP
> 推 `theme::set_dark_mode`，vue 轨经引擎组件 `darkMode` prop）；`accent`
> （§7.3 五色名，vue 轨经 `accent` prop；VM 轨 document accent 消费为
> PARITY 豁免行，PLAN-051 待澄清 3）。

### 7.1 中性色板（zinc/gray 系）

浅色档=vue 现行实值（PLAN-039/042 对齐纪律的既成事实）；深色档=VM 现行
zinc 实值（FENCE_CHROME 暗档/iced 深盘，VM 零改动，vue 深色档向其对齐）。

| 语义 | CSS 变量 | 浅色 | 深色 | vue 锚点 | VM 锚点 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 正文 fg | `--ad-fg` | `#111827` (gray-900) | `#fafafa` (zinc-50) | autodown-editor.css `:55`、StreamingRenderer `:456` | FENCE_BODY_FG (250,250,250) |
| muted fg | `--ad-muted` | `#6b7280` (gray-500) | `#a1a1aa` (zinc-400) | 同上 `:61` 等 | FENCE_HEADER_FG (161,161,166) |
| 边框 | `--ad-border` | `#e5e7eb` (gray-200) | `#3f3f46` (zinc-700) | 同上 `:14`（fallback 值） | FENCE_BORDER (63,63,70) |
| 面板面 surface | `--ad-surface` | `#ffffff` | `#09090b` (zinc-950) | 同上 `:16` | FENCE_BG (9,9,11) |
| 面 muted 叠层 | — | `hsl(220 9% 46% / α)`，α∈{.02,.03,.06,.08,.1,.14,.15,.18} | zinc 同系 alpha 叠加 | 两文件多处 | PANEL_CHROME `bg-muted` |
| 交互主色 fallback | `--ad-primary` | `220 90% 56%`（≈blue-600）/attr-host fallback `#3b82f6` | 同 hue 提亮（T4 定） | 同上 `:21` | theme token（`text-primary` 等） |

### 7.2 Accent 五色盘（document 面）

三件套 (accent / strong / soft)，浅色档 tailwind 600/700/50 步进；深色档
strong 取 400 步（深底可读性，规约行内分档，非实现侧临场）、soft 取 accent
@15% alpha 叠加。swatch 展示色=popover 钮（bg-*-500）。

| 名 | swatch | accent | strong(浅/深) | soft(浅/深) |
| :--- | :--- | :--- | :--- | :--- |
| indigo | bg-indigo-500 | `#4f46e5` | `#4338ca` / `#818cf8` | `#eef2ff` / `#4f46e5`@15% |
| coral | bg-rose-500 | `#e11d48` | `#be123c` / `#fb7185` | `#fff1f2` / `#e11d48`@15% |
| ocean | bg-sky-500 | `#0284c7` | `#0369a1` / `#38bdf8` | `#f0f9ff` / `#0284c7`@15% |
| sage | bg-emerald-500 | `#059669` | `#047857` / `#34d399` | `#ecfdf5` / `#059669`@15% |
| amber | bg-amber-500 | `#d97706` | `#b45309` / `#fbbf24` | `#fffbeb` / `#d97706`@15% |

vue 锚点：引擎 CSS `--ad-accent*` 定义（PLAN-051 T3 收敛为单源后）+
`[data-accent='<名>']` 覆盖组；标题色=`--ad-accent-strong`（深色档自动随
strong 深档值）。VM 锚点：➖（document accent 豁免，待澄清 3）。

### 7.3 排版（引擎文档面档——区别于 §2 应用级标尺）

引擎文档面用自档排版（vue 现行实值，两 pane 一致）：正文 `0.95rem`/
line-height 1.6；h1 `1.58rem` / h2 `1.33rem` / h3 `1.18rem`，700、
line-height 1.3，色=accent-strong；p `margin .5rem 0`；块间节奏引 §4.5
（12px）。行内 code `0.85em` mono；fence code `0.88rem`/1.5 mono。
**分叉登记**：VM 只读臂 heading 用 §2 应用级类表（text-4xl=36px 等），
VM 编辑壳 `heading_size`=30/24/20px——vue 文档面 1.58rem(≈25.3px) 与两臂
均不一致，系既有分叉（PARITY 十六项外新登记，PLAN-051 T11 落表）。

### 7.4 块家族 chrome（浅 / 深）

| 面 | 浅色 | 深色（VM zinc 基准） | vue 锚点 | VM 锚点 |
| :--- | :--- | :--- | :--- | :--- |
| fence 容器 | bg `#f9fafb`·border `#e5e7eb`·r8 | bg zinc-950·border zinc-700·r-lg | `.code-block-container`（editor css :908 / renderer :1001） | FENCE_CHROME(_LIGHT) |
| fence header | bg `#e5e7eb`·字 `#374151`·下边 `#d1d5db` | bg zinc-800·字 zinc-400 | `.code-block-header`（:915/:1014） | 同上 header 类串 |
| fence pre | bg `#f9fafb`·pad `.85em 1em` | 同容器深档 | `pre[data-language]`（:945/:1094） | 同上 body `p-4` |
| header 动作钮 | 字 `#4b5563`·hover bg gray-500/14% | 深档同规则 | `.code-action-btn`（:962） | ➖（折叠/复制钮 VM 只读豁免，PARITY #15 注） |
| 行内 code | bg gray-500/8%·字 `#111827`·r4 | bg zinc-400/12%·字 zinc-50 | `code`（:216/:980） | 段落 code span（core.rs mono 臂） |
| blockquote | 左边 3px `#e5e7eb`·字 muted·pad-l 1rem | 左边 zinc-700·字 zinc-400 | `.blockquote`（:166） | QUOTE_CHROME |
| 表格 | 边 `#e5e7eb`·th bg `--ad-accent-soft`·偶行 `#f9fafb`·pad `.9rem .6rem` | 边 zinc-700·th bg accent@15%·偶行 zinc-900 | `.table-node`（:173） | FAMILY_TABLE 类串 |
| callout 容器 | 边 `#e5e7eb`·r12·pad `1.1rem 1rem 1rem` | r-lg + §下行语义色 | `.autodown-callout`（:486）/`.admonition`（renderer :495） | CALLOUT_CHROME |
| callout 语义色（note/info/tip/warning/danger） | 三件 (bg/边/标题)×5：note `#eff6ff`/`#bfdbfe`/`#2563eb`·info `#f0f9ff`/`#7dd3fc`/`#0284c7`·tip `#f0fdf4`/`#86efac`/`#16a34a`·warning `#fffbeb`/`#fcd34d`/`#d97706`·danger `#fef2f2`/`#fca5a5`/`#dc2626` | VM 现行 alpha 档（`*-500/50` 边+`*-500/10` bg+`*-400` 字）——与 vue 浅色系**结构性分叉**，深色档规约值=VM alpha 档（vue 深色对齐之） | `:566-604`/renderer `:536-574` | callout_kind_classes |
| details | 边 `#e5e7eb`·r8·summary bg gray-500/6%·开合箭头=accent | 边 zinc-700·summary zinc-400/10% | `.autodown-details`（:606）/`details`（renderer :577） | DETAILS_CHROME |
| 分隔线 | 上边 1px `#e5e7eb` | 上边 zinc-700 | `hr`（:225/:1135） | BREAK_CHROME |
| math | 边透明/`#e5e7eb`·r8·pad `.75rem 1rem` | 深档边 zinc-700 | `.autodown-math-block`（:788）/`.math-block`（renderer :847） | PANEL_CHROME（web-only 降级，PARITY #9） |
| mermaid | 容器 `#ffffff`·header bg gray-500/8%·源码面 `#f8f9fa`·图主题参数（codeBlockLanguage.ts，Material 系）`#ffca28`/`#42a5f5`/`#ff7043`/`#0288d1`/`#fdd835` | 深档 zinc 系（图参数随 mermaid 深主题另定，web-only 面不阻） | `.mermaid-*`（renderer :861+） | PANEL_CHROME（web-only） |
| 断图 fallback | bg gray-500/6%·边 `#e5e7eb`·字 `#9ca3af` | 深档 zinc 系 | `.autodown-image-fallback`（:257）/`.image-error`（:1111） | ➖ |
| 任务勾选框 | checked=accent·unchecked `#9ca3af` | 同规则深档 | `.checkbox-icon`（renderer :1160） | List 家族 |

### 7.5 hljs 语法色双档（选择器组级）

**双轨单源**：`packages/core/rust/src/hljs_scope_map.rs` `hljs_group_rgb(group, dark)`
（a2r 生成物）为唯一真值；vue 侧 light 规则组现值与其逐值相同（PLAN-041
P041-4 镜像），vue 深色规则组（PLAN-051 T4 新增）与 VM syntect 主题
（`autodown-hljs-dark`）同源取值。

| 组（.hljs-* 类） | 浅色（github-light） | 深色（github-dark） |
| :--- | :--- | :--- |
| Keyword（keyword/selector-tag/doctag/section） | `#d73a49` | `#ff7b72` |
| Title（title/title.function_/function>title） | `#6f42c1` | `#d2a8ff` |
| String（string/regexp/addition） | `#032f62` | `#a5d6ff` |
| Constant（number/literal/variable/template-variable/attr/attribute） | `#005cc5` | `#79c0ff` |
| Comment（comment/quote/deletion，italic） | `#6a737d` | `#8b949a` |
| Meta（meta/meta-keyword/meta-string） | `#176f2c` | `#7ee787` |
| Tag（tag/name/built_in/type） | `#22863a` | `#7ee787` |
| 基础 fg（无 token） | `#09090b` | `#fafafa` |

vue 锚点：editor css `:1019-1074` / renderer `:789-844`（light 现行）+
T4 新增 `.is-dark` 作用域同组深色规则。VM 锚点：hljs_scope_map 生成物 →
`hljs_syntax_theme(dark)` 烘焙 `autodown-hljs-{dark,light}` 主题。
**已知债**：VM buffer 主题构建期选定，运行时翻转不重刷（DEBTS 050，
PLAN-051 T10 处置）。

### 7.6 编辑壳交互面（vue-only，PARITY #12 长期线）

斜杠菜单/气泡菜单/代码块语言菜单/表格菜单/块边界插入柄等编辑态交互 chrome
（surface/边框/阴影 `0 4px 12px rgba(0,0,0,.15)` 系/hover 规则）为 vue 引擎
独有面（VM 无对应交互，#12）；样式值从 §7.1 中性板取，深色档随组件根
`.is-dark` 翻转，无独立规约行需求。编辑器内滚动条（`rgba(0,0,0,.15)` thumb）
同属性（demo 级滚动条另有 PARITY #8）。

### 7.7 盘点口径（零游离值核对方法）

来源三文件：`engine/src/editor/styles/autodown-editor.css`（hex 出现 99 处，
去重 34 值）、`engine/src/render/StreamingRenderer.vue` scoped 段（63 处，
去重 33 值，与前者差集仅 `#3b82f6`——§7.1 已收）、家族 widget 组件
（CodeBlock/MathBlock/TableBlock/MermaidBlockWidget.vue + codeBlockLanguage.ts，
新增值仅 `#6366f1`（TableBlock 编辑态图标，indigo-500——随 §7.2 accent 语义
收编）、`#92400e`/`#fef3c7`/`#b91c1c`（Math/Mermaid 降级面语义色，随 §7.4
math/mermaid 行收）、其余均为已收值复用。核对命令：
`grep -oE '#[0-9a-fA-F]{6}\b' <file> | sort -u` 逐值对表，任何表外值=盘点
缺口。callout 15 语义值逐值见 vue 锚点行号段，以锚点为准。
