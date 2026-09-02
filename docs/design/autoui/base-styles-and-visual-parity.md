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
