# Remediation Plan: a3ui-replica → Pixel-Level a2ui Match

**Goal**: Bring the Auto-generated a3ui-replica to visual parity with https://a2ui-composer.ag-ui.com/ using Tailwind CSS + Vue + shadcn-vue.

**Strategy**: Tackle "Global" fixes first (they improve every page), then page-specific fixes in order of user impact.

---

## Phase 0 — Global Foundation (Do First)

These changes affect every page. Highest ROI.

### P0.1 Gradient Background
**File**: `source/front/app.at` (view block root styling)
**Issue**: All pages show plain `bg-slate-100`; original has soft gradient.
**Fix**: Replace `bg-slate-100` with a CSS gradient matching the original:
```
bg-gradient-to-br from-purple-50 via-pink-50 to-yellow-50
```
or use a custom `style` attribute with the exact original gradient colors.
**Effort**: 1 line change.
**Priority**: 🔴 Critical

### P0.2 Sidebar Logo Icon
**File**: `source/front/app.at`
**Issue**: No icon next to "A2UI COMPOSER".
**Fix**: Add a `settings` or `app-logo` icon (lucide-vue-next `Settings` or a custom SVG) before the title text.
**Effort**: Small — add one icon import + one node.
**Priority**: 🟡 Medium

### P0.3 Sidebar Active State
**File**: `source/front/app.at`
**Issue**: Active nav item has no distinct background.
**Fix**: Use `router-link-active` class styling. In the generated Vue code, `router-link` gets an `.router-link-active` class. We can style it with `bg-white rounded-lg shadow-sm` or add conditional classes in Auto.
**Effort**: Small — update App.vue template or add conditional style in Auto.
**Priority**: 🟡 Medium

### P0.4 Sidebar "WIDGETS" Casing
**File**: `source/front/app.at`
**Issue**: "WIDGETS" is all-caps; original uses "Widgets".
**Fix**: Change text from `"WIDGETS"` to `"Widgets"`.
**Effort**: Trivial.
**Priority**: 🟢 Low

### P0.5 Router Mode (Hash → History)
**File**: Generator `vue.rs` (router generation)
**Issue**: Generator emits `createWebHashHistory`; original uses `createWebHistory`.
**Fix**: Change `auto-man/src/vue.rs` router template from `createWebHashHistory` to `createWebHistory`.
**Effort**: 1-line change in generator.
**Priority**: 🟡 Medium

---

## Phase 1 — Create Page (`/`)

### P1.1 Textarea Shape
**File**: `source/front/pages/create.at`
**Issue**: Rectangular textarea; original is pill-shaped.
**Fix**: Change textarea style from `rounded-xl` to `rounded-full` and adjust padding/height to match the original single-line look. Also change border to softer `border-slate-200`.
**Effort**: Small style tweak.
**Priority**: 🟡 Medium

### P1.2 Create Button Inside Textarea
**File**: `source/front/pages/create.at`
**Issue**: Purple button overlaps bottom-right of textarea; original has a subtle gray/white button inside the right edge.
**Fix**: Restructure layout so the button is positioned `absolute right-2 top-1/2 -translate-y-1/2` with `bg-white text-slate-600 border border-slate-200 hover:bg-slate-50` styling.
**Effort**: Layout restructuring.
**Priority**: 🟡 Medium

### P1.3 "Powered by CopilotKit" Sparkle
**File**: `source/front/pages/create.at`
**Issue**: Missing sparkle emoji.
**Fix**: Change text from `"Powered by CopilotKit"` to `"Powered by ✨ CopilotKit"`.
**Effort**: Trivial.
**Priority**: 🟢 Low

### P1.4 "or Start Blank" as Text Link
**File**: `source/front/pages/create.at`
**Issue**: Rendered as dark pill button; original is plain underlined text link.
**Fix**: Replace `button` with `text` styled as `text-sm text-violet-600 hover:underline cursor-pointer`.
**Effort**: Small.
**Priority**: 🟡 Medium

---

## Phase 2 — Gallery Page (`/gallery`)

### P2.1 Update Gallery Widget Dataset
**File**: `source/front/pages/gallery.at`
**Issue**: Widgets don't match original (missing Chat Message, Recipe Card, Coffee Order, Contact Card; has Calendar Day, Weather, User Profile, Login Form instead).
**Fix**: Replace the `galleryWidgets` model array with the exact 6 widgets from the original:
1. Flight Status (keep, but update data to match original's richer preview)
2. Chat Message
3. Recipe Card
4. Email Compose (keep, update data)
5. Coffee Order
6. Contact Card
**Effort**: Large — requires defining new component trees for 4 new widgets.
**Priority**: 🔴 Critical

### P2.2 Masonry Layout
**File**: `source/front/pages/gallery.at`
**Issue**: Uses `grid grid-cols-3`; original uses masonry (variable height, flowing columns).
**Fix**: Use a CSS masonry approach. Options:
- Tailwind plugin `tailwindcss-masonry` (if available)
- CSS `columns-3` with `break-inside-avoid` on cards
- JavaScript masonry library (e.g., `vue-masonry`)
**Effort**: Medium — may need new dependency or CSS trick.
**Priority**: 🟡 Medium

### P2.3 Card Preview Richness
**File**: `source/front/pages/gallery.at` + A2UIRenderer
**Issue**: Previews are simple text-based; original has images, avatars, tabs.
**Fix**: Enrich the widget `components` arrays to include `Image`, `Avatar`, `Tabs` components where needed. Ensure A2UIRenderer supports all component types used in the original gallery widgets.
**Effort**: Medium — depends on A2UIRenderer capabilities.
**Priority**: 🟡 Medium

---

## Phase 3 — Icons Page (`/icons`)

### P3.1 Switch to Material Icons
**File**: `source/front/pages/icons.at` + any icon usage
**Issue**: Auto uses Lucide icons; original uses Material Icons.
**Fix**: Two options:
1. **Quick**: Keep Lucide but update the Icons page to show more icons (100+) with card-style grid
2. **Accurate**: Add `material-icons` or `material-symbols` dependency and use those in the Icons page
**Effort**: Medium if switching libraries; small if just restyling.
**Priority**: 🟡 Medium

### P3.2 Icon Grid Styling
**File**: `source/front/pages/icons.at`
**Issue**: Simple bordered cells; original has white cards with shadow.
**Fix**: Wrap each icon in a `div` with `bg-white rounded-xl shadow-sm p-4 flex flex-col items-center gap-2`.
**Effort**: Small.
**Priority**: 🟡 Medium

### P3.3 "Browse all icons" Link
**File**: `source/front/pages/icons.at`
**Issue**: Missing external link.
**Fix**: Add a top-right link: `<a href="https://fonts.google.com/icons" target="_blank">Browse all icons ↗</a>`.
**Effort**: Trivial.
**Priority**: 🟢 Low

---

## Phase 4 — Catalog Pages (`/basic-catalog`, `/custom-catalog`)

These pages require the most work because the original is essentially a documentation site with rich interactive features.

### P4.1 Basic Catalog — Sidebar Navigation
**File**: `source/front/pages/basic_catalog.at`
**Issue**: Single long list; original has sidebar nav + one component detailed at a time.
**Fix**: Restructure as a two-column layout:
- Left: scrollable list of component names
- Right: detailed view for selected component
This requires state management (`selectedComponent`) and conditional rendering.
**Effort**: Large — new architecture.
**Priority**: 🟡 Medium

### P4.2 Basic Catalog — Usage Code Snippets
**File**: `source/front/pages/basic_catalog.at`
**Issue**: No code examples; original shows Usage snippets.
**Fix**: Add a `code` block or preformatted text section for each component. May need new Auto syntax for code blocks, or use a `pre` / `code` HTML element in the view.
**Effort**: Medium — may need transpiler support for `pre`/`code` tags.
**Priority**: 🟡 Medium

### P4.3 Basic Catalog — Props Tables
**File**: `source/front/pages/basic_catalog.at`
**Issue**: No props documentation; original has detailed tables.
**Fix**: Create table markup in the view for each component's props. Could use a `table` component or grid layout.
**Effort**: Large — requires defining prop metadata for every component.
**Priority**: 🟡 Medium

### P4.4 Custom Catalog — Tabs & Data Panel
**File**: `source/front/pages/custom_catalog.at`
**Issue**: Simple list; original has tabs + JSON data editor.
**Fix**: Add tab navigation (`Flight Card` | `Sales Dashboard`) and a right-side JSON preview panel. The JSON panel can be a read-only `textarea` or `pre` block showing the component data.
**Effort**: Large — new interactive components.
**Priority**: 🟡 Medium

### P4.5 Add Missing Basic Components
**File**: Component catalog definitions
**Issue**: Missing Video, AudioPlayer, DateTimeInput, ChoicePicker, Navigation, Modal, Decoration.
**Fix**: Add these component definitions to the basic catalog. They need to be implemented in A2UIRenderer or as shadcn-vue components.
**Effort**: Large — requires implementing new UI components.
**Priority**: 🟡 Medium

---

## Phase 5 — Theater Page (`/theater`)

### P5.1 Playback Controls
**File**: `source/front/pages/theater.at`
**Issue**: Simple Play/Reset buttons; original has full media controls.
**Fix**: Build a custom player control bar with:
- Play/pause button
- Skip back/forward buttons
- Progress bar (slider)
- Speed selector dropdown
**Effort**: Large — custom component.
**Priority**: 🟡 Medium

### P5.2 Mock Browser Frame
**File**: `source/front/pages/theater.at`
**Issue**: Plain preview box; original has mock browser chrome.
**Fix**: Add a wrapper div styled like a browser window:
- Traffic lights (red/yellow/green dots)
- URL bar
- "React Renderer" label
**Effort**: Medium — pure CSS/Tailwind.
**Priority**: 🟡 Medium

### P5.3 JSONL Pretty/Wire Toggle
**File**: `source/front/pages/theater.at`
**Issue**: No format toggle.
**Fix**: Add a toggle switch or segmented control for "Pretty" / "Wire".
**Effort**: Small.
**Priority**: 🟢 Low

### P5.4 Top Tab Bar (Events / Data / Config)
**File**: `source/front/pages/theater.at`
**Issue**: No tab bar.
**Fix**: Add three tabs at the top of the page. Only "Events" needs content; others can be placeholder.
**Effort**: Small.
**Priority**: 🟡 Medium

---

## Quick Reference: Files to Modify

| Phase | File | What to Change |
|-------|------|----------------|
| P0 | `source/front/app.at` | Gradient background, sidebar logo, active state, casing |
| P0 | `crates/auto-man/src/vue.rs` | Router template: `createWebHistory` |
| P1 | `source/front/pages/create.at` | Textarea shape, button style, sparkle, link styling |
| P2 | `source/front/pages/gallery.at` | Widget data, masonry layout, card previews |
| P3 | `source/front/pages/icons.at` | Icon grid styling, material icons, browse link |
| P4 | `source/front/pages/basic_catalog.at` | Sidebar nav, code snippets, props tables |
| P4 | `source/front/pages/custom_catalog.at` | Tabs, data panel, richer previews |
| P5 | `source/front/pages/theater.at` | Player controls, browser frame, tabs, toggle |

---

## Recommended Execution Order

1. **P0.1** — Gradient background (instant visual upgrade for all pages)
2. **P1.1–P1.4** — Create page polish (landing page = first impression)
3. **P0.2–P0.4** — Sidebar polish
4. **P2.1–P2.2** — Gallery data + masonry (high-traffic page)
5. **P3.1–P3.3** — Icons page (simple fixes)
6. **P5.1–P5.4** — Theater page (bounded scope)
7. **P4.1–P4.5** — Catalog pages (largest effort, deferred)
8. **P0.5** — Router history mode (when ready for deployment)

---

## Blockers / Dependencies

| Blocker | Impact | Resolution |
|---------|--------|------------|
| A2UIRenderer missing components (Image, Avatar, Tabs) | GL3, P2.3, P4.5 | Implement in `A2UIRenderer.vue` or add shadcn components |
| Auto syntax lacks `pre`/`code` block support | P4.2 | Add raw HTML escape or new Auto syntax |
| Auto syntax lacks table support | P4.3 | Use grid layout or add table syntax |
| No masonry CSS in Tailwind by default | P2.2 | Use `columns-3` CSS or install plugin |

---

*Plan created: 2026-05-01*
*Next step: Begin Phase 0.1 (gradient background) and Phase 1 (Create page polish).*
