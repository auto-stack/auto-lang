# Plan 455: AutoUI Cross-Backend Parity Tracker (All examples/ui Examples)
> 📦 **终态（2026-09-01，Plan 513 B 组处置）**：矩阵跟踪职能已被 Plan 506/512 批量桌面化与 autoui-verifier 技能接管（**superseded**）；头部 Status: In Progress 为过时残留——归档。

**Status**: In Progress  
**Scope**: `crates/auto-lang/src/ui/`, `crates/auto-man/`, `examples/ui/*`  
**Goal**: Track, verify, and enforce visual & functional parity across dual backends (Vue mode `auto run` vs VM/Iced mode `auto run -r vm`) for all examples in `examples/ui/`.

---

## 1. Background & Context

AutoUI provides a unified UI DSL (`.at`) that compiles to both:
- **Vue mode** (`auto run`): Vue 3 + Tailwind CSS + shadcn-vue.
- **VM mode** (`auto run -r vm`): In-process AURA interpreter + Iced / wgpu native renderer.

To ensure developers get a truly consistent write-once-run-everywhere experience, all official examples in `examples/ui/` are systematically audited using the `/autoui-verifier` skill across layout, styling, typography, spacing, interactions, and reactive state updates.

---

## 2. Core Engine Enhancements & Widget Parity Standards

### 2.1 Input & Form Control Focus Styles
- **Focus State Detection**: Detect `Status::Focused` in `renderer.rs` for `text_input` and `text_editor`.
- **Focus Border Width**: Expand border width to 2.0px (matching Vue `focus-visible:ring-2`).
- **Focus Primary Color**: Dynamically resolve system/accent `Color::Primary` (e.g. `#3b82f6` / accent color) or component variant color.
- **Blur/Resting Recovery**: Seamlessly restore resting 1.0px border and subtle border color on blur.

### 2.2 Text Box-Model & Typography
- Support text-bearing tags (`a`, `link`, `h1`-`h6`, `small`, `strong`, `em`, `b`, `i`) in `aura_view_builder.rs`.
- Ensure uniform font sizes, weights, and letter spacings across both rendering pipelines.

### 2.3 Spacing & Margin Container Wrapping
- Support full margin semantics (`mt-*`, `mb-*`, `ml-*`, `mr-*`, `mx-auto`, `ml-auto`, `mr-auto`) across all widget primitives (`Input`, `Textarea`, `Checkbox`, `Button`, `Text`, `Container`, `Row`, `Column`).

---

## 3. `examples/ui/` Parity Tracking Matrix

| Example ID | Name | Category / Key Widgets | Verified States | Dual-Backend Parity Status |
|:---|:---|:---|:---|:---:|
| `001` | `counter` | Basic state, button, text | Initial, Inc, Dec, Reset | 🟢 Passed |
| `002` | `todo` | Input, list, checkbox, filter | Initial, Add, Check, Delete | 🟢 Passed |
| `003` | `clock` | Timer, formatting, reactive display | Realtime tick | 🟢 Passed |
| `004` | `profile-card` | Image clipping, badges, negative margins | Initial | 🟢 Passed |
| `005` | `login` | Card, input, password masking, link, error display | Initial, Empty error, Typed, Masked, Submit | 🟢 Passed (Plan 452/453) |
| `006` | `hero-section` | Gradient background, full-screen centering, CTA button | Initial, Gradient, Hover, Typography | 🟢 Passed |
| `006` | `hero-section` (Plan 458) | **Theme Settings panel**（Light/Dark + 5 accent 色板，运行时切换） | dark/light × vue/vm 初始态、面板展开、运行时切换、accent 切换 | 🟢 Passed (Plan 458) |
| `007` | `stats-board` | Grid, card, stats, badges | Layout, Responsive, Numbers, Avatars | 🟢 Passed |
| `008` | `pricing-table` | Pricing tiers, cards, lists, badges | Layout, Selection | 🟡 Pending Audit |
| `009` | `article-feed` | Feed list, avatar, tags, cards | Scroll, Item click | 🟡 Pending Audit |
| `010` | `contact-form` | Form fields, textareas, validation | Submit, Field errors | 🟡 Pending Audit |
| `011` | `calculator` | Grid buttons, display, arithmetic | Button clicks, calculations | 🟡 Pending Audit |
| `012` | `stopwatch` | Timer state, lap list, buttons | Start, Pause, Lap, Reset | 🟡 Pending Audit |
| `013` | `todo` | Input, list, checkbox, filter | Initial, Add, Check, Delete | 🟢 Passed |
| `022` | `kanban` | Drag/drop column boards, task cards | Column switch, task creation | 🟡 Pending Audit |
| `023` | `realworld` | Multi-view SPA, routing, articles, comments | Route switch, article list | 🟡 Pending Audit |
| `024` | `charts` | Chart widgets, legends, tooltips | Chart rendering | 🟡 Pending Audit |
| `025` | `dashboard` | Grid cards, summary metrics, charts | Multi-column layout | 🟡 Pending Audit |

---

## 4. Current Tasks & Phase Roadmap

### Phase 006: `006-hero-section` Parity Fixes
- [x] **Defect 1: Button Design Token & Foreground Alignment**
  - Fix `crates/auto-lang/src/ui/style/theme.rs`: In dark mode, align `Color::Primary` to shadcn `--primary` (`210 40% 98%` / `rgb(248, 250, 252)`) and `Color::OnPrimary` to `--primary-foreground` (`222.2 47.4% 11.2%` / `rgb(15, 23, 42)`).
- [x] **Defect 2: Button Hover Feedback Alignment**
  - Fix `crates/auto-lang/src/ui/aura_view_builder.rs`: Add default `hover:bg-primary/90` to button default preset, so VM captures `Status::Hovered` feedback matching Vue.
- [x] **Defect 3: Typography & Text Baseline Alignment**
  - Verify font size, font weight, line height, and text color rendering between Vue and VM for headline/subtitle.
- [x] **Defect 4: Dual-Backend Re-Verification**
  - Re-run dual-backend screenshot captures (initial and hovered states) and perform side-by-side pixel audit.

### Phase 007: `007-stats-board` Parity Fixes
- [x] **Defect 1: Vue Scaffold CVA & Reka-UI Dependency Detection**
  - Fixed `crates/auto-man/src/vue.rs` to detect `class-variance-authority` and `reka-ui` requirements when `avatar`, `badge`, or `progress` components are used in projects without buttons.
- [x] **Defect 2: VM Spacer Cross-Axis Height Collapse**
  - Fixed `crates/auto-lang/src/ui/aura_view_builder.rs` `convert_spacer` to use `Style::parse("flex-1")` (width fill only) instead of `w-full h-full`, preventing height-0 row collapses in Iced.
- [x] **Defect 3: Vue Spacer `<div class="flex-1" />` Codegen**
  - Fixed `crates/auto-lang/src/ui_gen/vue.rs` to classify `spacer` as a layout primitive and exclude layout primitives from `is_shadcn_component`, ensuring proper `<div class="flex-1" />` spacer emission.
- [x] **Defect 4: Vue Codegen Layout Gap Deduplication**
  - Fixed `crates/auto-lang/src/ui_gen/vue.rs` `extract_classes` so default `gap-4` is omitted when user style provides custom `gap-*` (e.g. `gap-0`, `gap-1`), eliminating gap conflicts.
- [x] **Defect 5: VM `ProgressBar` Styling, Dynamic Value & Margin Wrapping**
  - Fixed `crates/auto-lang/src/ui/iced/renderer.rs` & `aura_view_builder.rs`:
    - Resolved dynamic state expressions (`.target_progress`) in `extract_f64` via `resolve_expr_to_value`.
    - Applied 8px height, rounded-full pill border radius, primary purple fill, and container margin wrapping (`wrap_with_margin`) for `ProgressBar`.
- [x] **Defect 6: Playwright Full-Page Capture Viewport**
  - Configured `test_vue_playwright.mjs` with `fullPage: true` to capture the complete page scroll height including bottom `Team Performance` cards.
- [x] **Defect 7: Dual-Backend Visual & Pixel Audit**
  - Verified 100% geometry, padding, border radius, progress bar, avatar, and metric alignment across Vue (Playwright) and VM (AutoUI MCP).

### Phase 008: `008-pricing-table`
- [ ] Verify pricing tier cards, highlighted card borders, feature check icons, and CTA buttons.

### Phase 010: `010-contact-form`
- [ ] Verify multi-line textarea, input focus rings, validation errors, and submit states.
