# a3ui-replica vs Original a2ui — Visual Diff Analysis

**Date**: 2026-05-01
**Method**: Playwright full-page screenshots at 1440×900 viewport, side-by-side comparison
**Local URL**: http://localhost:3458 (hash router: `/#/route`)
**Remote URL**: https://a2ui-composer.ag-ui.com (HTML5 history router)

---

## Summary Statistics

| Category | Count | Severity |
|----------|-------|----------|
| Global / Shared UI | 5 | High |
| Create Page | 5 | Medium |
| Gallery Page | 4 | High |
| Basic Catalog | 5 | High |
| Custom Catalog | 5 | High |
| Icons Page | 5 | Medium |
| Theater Page | 5 | High |
| **Total** | **34** | — |

---

## 1. Global / Shared UI (affects every page)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| G1 | **Background gradient** | Soft purple/pink/yellow gradient across all pages | Plain `bg-slate-100` light gray | High |
| G2 | **Sidebar logo icon** | Purple gear/settings icon next to "A2UI COMPOSER" | No icon, text only | Medium |
| G3 | **Sidebar active state** | White rounded-lg background pill on active item | No distinct active state styling | Medium |
| G4 | **Sidebar section header casing** | "Widgets" (sentence case) | "WIDGETS" (all caps) | Low |
| G5 | **Router mode** | `createWebHistory` — clean URLs (`/gallery`) | `createWebHashHistory` — hash URLs (`/#/gallery`) | Low |

### Screenshots
- `analysis/screenshots/compare_create.png` — shows background + sidebar differences most clearly

---

## 2. Create Page (`/`)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| C1 | **Textarea shape** | Pill-shaped / `rounded-full` with soft border | Rectangular `rounded-xl` with standard border | Medium |
| C2 | **Create button placement** | White/gray text button inside the textarea | Purple button overlapping textarea bottom-right | Medium |
| C3 | **"Powered by CopilotKit"** | Has sparkle ✨ emoji/icon before "CopilotKit" | Plain text, no icon | Low |
| C4 | **"or Start Blank" styling** | Plain text link with underline on hover | Dark pill button with purple text | Medium |
| C5 | **Vertical spacing** | More generous padding above/below form | Tighter spacing | Low |

### Screenshots
- `analysis/screenshots/local_create.png`
- `analysis/screenshots/remote_create.png`
- `analysis/screenshots/compare_create.png`

---

## 3. Gallery Page (`/gallery`)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| GL1 | **Widget dataset** | 6 widgets: Flight Status, Chat Message, Recipe Card, Email Compose, Coffee Order, Contact Card | 6 widgets: Flight Status, Email Compose, Calendar Day, Weather, User Profile, Login Form | High |
| GL2 | **Layout mode** | Masonry (Pinterest-style, variable height cards) | Regular 3-column CSS grid (`grid-cols-3`) | High |
| GL3 | **Card preview richness** | Rich previews with images, avatars, tabs, star ratings | Simple text-based previews via A2UIRenderer | High |
| GL4 | **Missing widget types** | Chat Message, Recipe Card, Coffee Order, Contact Card | — | High |

### Notes
The widget data in `gallery.at` is hardcoded inline. The original site pulls from a larger, curated gallery dataset. To match, we need to update the `galleryWidgets` array in `model` to match the original's 6 widgets, or expand to include all original widgets.

### Screenshots
- `analysis/screenshots/local_gallery.png`
- `analysis/screenshots/remote_gallery.png`
- `analysis/screenshots/compare_gallery.png`

---

## 4. Basic Catalog (`/basic-catalog`)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| B1 | **Page architecture** | Sidebar navigation listing all components; main area shows ONE component's full docs | Single long scrollable list of all components | High |
| B2 | **Documentation depth** | Preview + Usage (code snippet) + Props table (name, type, default, description) | Title + 1-line description + simple live preview only | High |
| B3 | **Missing components** | Video, AudioPlayer, DateTimeInput, ChoicePicker, Navigation, Modal, Decoration | Not present | High |
| B4 | **Props tables** | Detailed tables with TypeScript types and defaults | None | High |
| B5 | **Breadcrumbs** | "Concepts / Component Catalog / Reference: Component Gallery" | None | Low |

### Notes
The original Basic Catalog is essentially a documentation site with rich API docs. Our version is a simple component showcase. A full match would require significant new Auto syntax support (code blocks, tables, tabbed sections) or embedding markdown/VitePress-style docs.

### Screenshots
- `analysis/screenshots/local_basic_catalog.png`
- `analysis/screenshots/remote_basic_catalog.png`
- `analysis/screenshots/compare_basic_catalog.png`

---

## 5. Custom Catalog (`/custom-catalog`)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| CU1 | **Layout architecture** | Left sidebar (Assembled / Catalog Components) + tabbed main area | Simple vertical list of 2 example cards | High |
| CU2 | **Tab navigation** | Tabs: "Flight Card" \| "Sales Dashboard" and "Component Data" with sub-tabs | No tabs | High |
| CU3 | **Data richness** | Real airline logos (favicons), prices, flight numbers, realistic data | Hardcoded simple text data | High |
| CU4 | **Live JSON editor** | Right panel shows editable JSON data driving the preview | No data panel | High |
| CU5 | **Breadcrumbs** | "Concept: Catalogs" and "Guide: Define your own" | None | Low |

### Screenshots
- `analysis/screenshots/local_custom_catalog.png`
- `analysis/screenshots/remote_custom_catalog.png`
- `analysis/screenshots/compare_custom_catalog.png`

---

## 6. Icons Page (`/icons`)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| I1 | **Icon library** | Material Icons (Google) | Lucide icons | Medium |
| I2 | **Icon count** | 100 most commonly used | ~60 icons | Medium |
| I3 | **Grid cell styling** | White cards with shadow, rounded-xl | Bordered cells with light border | Medium |
| I4 | **"Browse all icons" link** | External link at top-right | Missing | Low |
| I5 | **Background** | Gradient (same as other pages) | Plain gray | Low |

### Screenshots
- `analysis/screenshots/local_icons.png`
- `analysis/screenshots/remote_icons.png`
- `analysis/screenshots/compare_icons.png`

---

## 7. Theater Page (`/theater`)

| # | Difference | Original | Auto (Current) | Severity |
|---|-----------|----------|----------------|----------|
| T1 | **Top tab bar** | "Events" \| "Data" \| "Config" tabs | No tabs | High |
| T2 | **Playback controls** | Play/pause, skip back/forward, progress scrubber, speed selector (1×) | Simple "Play" / "Reset" buttons | High |
| T3 | **Preview frame** | Mock browser chrome with URL bar, traffic lights, "React Renderer" label | Plain white box with "Waiting for stream..." | High |
| T4 | **JSONL stream panel** | "Pretty" / "Wire" format toggle | No toggle | Medium |
| T5 | **Overall fidelity** | Professional media-player feel | Basic prototype feel | High |

### Screenshots
- `analysis/screenshots/local_theater.png`
- `analysis/screenshots/remote_theater.png`
- `analysis/screenshots/compare_theater.png`

---

## Severity Legend

| Severity | Meaning |
|----------|---------|
| **High** | Visually obvious on first glance; significantly impacts perceived quality |
| **Medium** | Noticeable on close inspection; affects polish but not core functionality |
| **Low** | Minor detail; casual users may not notice |

---

## Root Cause Categories

| Category | Issues | Root Cause |
|----------|--------|-----------|
| **Missing gradient background** | G1, I5 | `App.vue` uses `bg-slate-100` instead of gradient |
| **Missing sidebar polish** | G2, G3, G4 | Sidebar markup/styling in `App.vue` is simplified |
| **Create form styling** | C1–C5 | `create.at` uses basic Tailwind classes, not matching original design tokens |
| **Gallery data mismatch** | GL1–GL4 | `gallery.at` has different hardcoded widget dataset |
| **Gallery layout** | GL2 | Uses `grid-cols-3` instead of masonry |
| **Catalog architecture** | B1, B2, CU1, CU2 | Requires rich documentation features not yet in Auto syntax |
| **Missing components** | B3 | Basic Catalog needs more A2UI components implemented |
| **Theater complexity** | T1–T5 | Theater page needs custom player UI not generatable from Auto |
| **Icon library mismatch** | I1 | Auto uses Lucide; original uses Material Icons |
| **Router mode** | G5 | Generator emits `createWebHashHistory` instead of `createWebHistory` |
