# 22 - AutoUI Base Styles and Cross-Backend Visual Parity Specification

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

Modern application design systems (Tailwind CSS, Shadcn UI, Apple HIG, Material Design 3) reset unstyled HTML elements and apply structured, proportional typography scales.

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

## 3. Base Element & Layout Defaults

| Tag | Default Styling | Notes |
| :--- | :--- | :--- |
| **`col` / `column`** | `flex flex-col gap-4` | Layout Primitive |
| **`row`** | `flex flex-row gap-4` | Layout Primitive |
| **`center`** | `flex flex-col items-center justify-center h-full` | Layout Primitive |
| **`grid`** | `grid` | Grid layout container |
| **`scroll`** | `overflow-auto` | Scrollable container |
| **`container`** | `max-w-7xl mx-auto` | Content centering container |
| **`button`** | `px-4 py-2 rounded` (plain) / shadcn variant | Interactive element |
| **`input`** | `border-primary rounded px-2 py-1` | Form element |
| **`textarea`** | `border-primary rounded px-2 py-1` | Form element |
| **`header`** | `w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60` | Semantic Header |
| **`nav`** | `flex items-center gap-4` | Semantic Navigation |
| **`aside`** | `w-64 border-r bg-background` | Semantic Sidebar |
| **`footer`** | `w-full border-t bg-background` | Semantic Footer |
| **`main`** | `flex-1` | Main Content Area |

---

## 4. Multi-Backend Implementation Architecture

### 4.1 Vue / Web Implementation
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
3. **SFC Generator (`vue.rs`)**:
   Generates explicit default classes (`text-4xl font-bold tracking-tight mb-4 text-primary` for `h1`, etc.) when no user class is specified.

### 4.2 VM / Iced Implementation (`aura_view_builder.rs`)
1. **Merge Strategy**:
   When a user writes `h1 "Hello" { style: "text-primary" }`, the VM merges the user's classes into the default style:
   ```rust
   if let Some(mut default) = default {
       if let Some(user) = style.take() {
           default.classes.extend(user.classes);
       }
       style = Some(default);
   }
   ```
2. Resolves `text-4xl` to 36px font size, `font-bold` to bold weight, and `text-primary` to the active accent theme color.

### 4.3 Rust Native Implementation (`rust.rs`)
1. In `heading_default_style()`, prepends heading default classes (`text-4xl font-bold`, etc.) to user-provided styles.

---

## 5. Verification & Testing

Every visual parity change must be tested against:
1. `auto run` (Vue dev server @ port 30xx)
2. `auto run -r vm` (Iced native window)
3. `examples/ui/001-helloworld` (Single heading smoke test)
4. `examples/widgets-gallery` (Full typography & component catalog)
