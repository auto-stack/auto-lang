# Plan 234: A3UI A2Vue Replica — Auto-Lang A2UI Composer Clone

## Status

**Phase 0: NOT STARTED** — Analysis & scaffolding

## Goal

Replicate the `a3ui` project (a Vue/shadcn-vue clone of Google's A2UI Composer gallery, located at `../a3ui`) using Auto language and the `a2vue` backend generator. The final output should be a functionally equivalent Vue 3 application generated from Auto source code (`*.at` files).

## Background

### What is a3ui?
`a3ui` (`D:\autostack\a3ui`) is a Vue 3 + shadcn-vue + Tailwind CSS project that replicates the Google A2UI Composer interface (`https://a2ui-composer.ag-ui.com/gallery`). It provides:

- **7 page views**: Create, Gallery, Widget Editor, Basic Catalog, Custom Catalog, Icons, Theater
- **A2UI JSON Runtime Renderer**: `A2UIRenderer.vue` dynamically renders A2UI component trees from JSON
- **30+ Gallery Widget presets**: Flight Status, Email Compose, Calendar Day, Weather, Product Card, etc.
- **3-panel Widget Editor**: JSON editor / Live preview / AI chat simulation
- **Theater Mode**: JSONL streaming playback with speed controls
- **Pinia state management** with localStorage persistence

### What is a2vue?
`a2vue` (`crates/auto-lang/src/ui_gen/vue.rs`) is Auto's Vue 3 SFC backend generator. It compiles AURA widgets (written in Auto's `widget {}` syntax) into Vue 3 Single File Components. It supports:

- shadcn-vue component mappings (Button, Input, Card, Tabs, Dialog, Table, etc.)
- Tailwind CSS class generation
- Vue router integration (`routes {}` block)
- Reactive state (`model {}` → Vue `ref()`)
- Event handlers (`on {}` → Vue methods)

### Existing Related Work
- `examples/component-gallery/` — 46 shadcn-vue components documented in pure Auto
- `examples/a2ui-composer/` — Simplified A2UI Composer (Phase 0-1 of Plan 217)
- `docs/plans/old/217-a2ui-composer-implementation.md` — Prior A2UI composer plan
- `docs/design/new/a2ui-composer-analysis.md` — A2UI ↔ AutoUI architecture mapping

---

## Architecture Strategy

### Core Challenge
The a3ui project is fundamentally a **runtime JSON renderer** (A2UIRenderer takes JSON and renders UI dynamically). Auto's a2vue is a **compile-time static generator** (AURA widgets are compiled to Vue SFCs at build time).

### Hybrid Approach
We use a **hybrid strategy**: Auto generates the application shell, navigation, and static pages. The dynamic A2UI JSON rendering is delegated to a pre-built Vue component embedded in the generated project.

```
┌─────────────────────────────────────────────────────────────┐
│                    Auto Source (.at files)                   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────────────┐ │
│  │  App    │ │ Gallery │ │ Create  │ │  WidgetEditor     │ │
│  │(shell)  │ │(static) │ │(static) │ │ (shell + embed)   │ │
│  └────┬────┘ └────┬────┘ └────┬────┘ └─────────┬─────────┘ │
│       │           │           │                  │          │
│       └───────────┴───────────┴──────────────────┘          │
│                          │                                   │
│                    a2vue generator                           │
│                          │                                   │
│                          ▼                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Generated Vue 3 Project                 │    │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐  │    │
│  │  │ App.vue│ │Gallery │ │Create  │ │WidgetEditor  │  │    │
│  │  │(static)│ │.vue    │ │.vue    │ │.vue          │  │    │
│  │  └────────┘ └────────┘ └────────┘ └──────┬───────┘  │    │
│  │                                          │          │    │
│  │       ┌──────────────────────────────────┘          │    │
│  │       ▼                                             │    │
│  │  ┌─────────────┐    ┌─────────────┐                │    │
│  │  │A2UIRenderer │◄───│ JSON Editor │                │    │
│  │  │ (embedded)  │    │  (embedded) │                │    │
│  │  └─────────────┘    └─────────────┘                │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Component Strategy

| a3ui Component | Auto Approach | Notes |
|---------------|---------------|-------|
| `AppLayout` / `AppSidebar` | Pure Auto widget | Static layout, router links |
| `CreateView` | Pure Auto widget | Textarea + button, simple form |
| `GalleryView` | Pure Auto widget | `for` loop over static widget data |
| `WidgetEditorView` | Auto shell + embedded JS | 3-panel layout in Auto; JSON editor & A2UIRenderer as embedded Vue components |
| `BasicCatalogView` | Pure Auto widget | Static component documentation |
| `CustomCatalogView` | Pure Auto widget + embed | Static docs + embedded A2UIRenderer for live examples |
| `IconsView` | Pure Auto widget | Grid of icon names |
| `TheaterView` | Auto shell + embedded JS | Control bar in Auto; JSONL stream & renderer as embedded components |
| `A2UIRenderer` | **Pre-built Vue component** | Copied into `gen/vue/src/components/` post-generation |
| Gallery widget data | Auto `const` / `model` | Static JSON-like data structures in Auto |
| Pinia stores | Auto `model` + `on` handlers | Auto's reactive state replaces Pinia |
| localStorage | Auto `#[persist]` or JS bridge | May need JS FFI for localStorage |

---

## Phase Breakdown

### Phase 0: Scaffolding & A2UIRenderer Extraction (1-2 days)

**Goal**: Extract the A2UI renderer from a3ui and establish the build pipeline.

| # | Task | Details |
|---|------|---------|
| 0.1 | Create `examples/a3ui-replica/` directory | New Auto project with `pac.at` |
| 0.2 | Extract `A2UIRenderer` + sub-renderers from a3ui | Copy `A2UIRenderer.vue`, `a2ui-renderers/*.vue`, `useRenderer.ts`, `types/a2ui.ts` to a standalone npm package or into the generator template |
| 0.3 | Extract gallery widget data | Convert `gallery-widgets.ts` to Auto-compatible data format |
| 0.4 | Verify `auto build` → Vue project pipeline | Ensure `auto build` generates a working Vue project that can import the A2UIRenderer |
| 0.5 | Add A2UIRenderer to a2vue generator template | Ensure the generated `package.json` includes necessary dependencies (`lucide-vue-next`, etc.) |

**Verification**: `auto build` succeeds; opening `gen/vue/dist/index.html` shows a blank page with no errors and the A2UIRenderer component is importable.

---

### Phase 1: Application Shell & Navigation (2-3 days)

**Goal**: Replicate the a3ui sidebar navigation and layout using pure Auto.

| # | Task | Details |
|---|------|---------|
| 1.1 | Implement `App` widget with routes | 7 routes: `/`, `/gallery`, `/widget/:id`, `/basic-catalog`, `/custom-catalog`, `/icons`, `/theater` |
| 1.2 | Implement `AppSidebar` widget | Glassmorphic sidebar with logo, nav items, active state styling |
| 1.3 | Implement `AppLayout` widget | Sidebar + main content area with proper margin/width |
| 1.4 | Style system | Map a3ui's color tokens (`bg-bg-card`, `text-text-primary`, `border-border-default`, `accent-purple`) to Tailwind classes |
| 1.5 | Widget list in sidebar | Dynamic list of created widgets (from model state) |

**Key Auto Syntax**:
```auto
widget App {
    routes {
        "/" -> use CreatePage
        "/gallery" -> use GalleryPage
        "/widget/:id" -> use WidgetEditorPage
        "/basic-catalog" -> use BasicCatalogPage
        "/custom-catalog" -> use CustomCatalogPage
        "/icons" -> use IconsPage
        "/theater" -> use TheaterPage
    }
    // ...
}
```

**Verification**: Navigating between all 7 routes works; sidebar highlights active route; mobile responsive (optional for Phase 1).

---

### Phase 2: Create Page & Gallery Page (2-3 days)

**Goal**: Static pages that match a3ui's Create and Gallery views.

| # | Task | Details |
|---|------|---------|
| 2.1 | `CreatePage` widget | Centered textarea, "Create" button, "Start Blank" link |
| 2.2 | `GalleryPage` widget | Grid layout (`grid-cols-1 md:grid-cols-2 xl:grid-cols-3`) |
| 2.3 | Gallery cards | Card with title + preview area containing embedded `<A2UIRenderer>` |
| 2.4 | Click-to-navigate | Clicking a gallery card creates a widget in model state and navigates to `/widget/:id` |
| 2.5 | Load gallery widget data | Static data from extracted `gallery-widgets.ts` |

**Challenge**: Embedding `<A2UIRenderer>` inside Auto-generated Vue code.

**Solution Options**:
- **Option A**: Use Auto's `extern` syntax to reference external Vue components
- **Option B**: Post-process generated Vue files to inject `<A2UIRenderer>` tags
- **Option C**: Use `div` with `v-html` or raw Vue `component :is` via Auto's escape hatch

**Recommended**: Option A — investigate if Auto supports `extern component A2UIRenderer from "@/components/A2UIRenderer"` or similar.

**Verification**: Gallery page shows 30+ cards with rendered A2UI previews; clicking navigates to Widget Editor.

---

### Phase 3: Widget Editor Shell (3-4 days)

**Goal**: The 3-panel Widget Editor layout.

| # | Task | Details |
|---|------|---------|
| 3.1 | Top bar | Widget name (editable), "Copy JSON" button, "Download" button |
| 3.2 | Left panel (40%) | JSON editor textarea with line numbers |
| 3.3 | Center panel (35%) | Preview area with dot-grid background + `<A2UIRenderer>` |
| 3.4 | Right panel (25%) | AI chat simulation panel (static/mock) |
| 3.5 | Data model editor | Bottom sub-panel for editing data model JSON |
| 3.6 | Live update | Changes in JSON editor debounce-update the preview |
| 3.7 | Copy/Download JSON | Button handlers using JS clipboard API |

**Challenge**: Complex layout with percentage widths, JSON parsing in handlers, debounced updates.

**Auto Limitations to Watch**:
- Debouncing may require JS FFI or custom Auto logic
- JSON parsing (`JSON.parse`) is JS-only; may need `#[js]` annotation or extern function
- Clipboard API is browser-only

**Mitigation**: Use Auto's `#[c]` / `#[js]` annotations to embed raw JavaScript for browser APIs.

**Verification**: Editing JSON in left panel updates preview in real-time; copy/download buttons work.

---

### Phase 4: Catalog Pages (2-3 days)

**Goal**: Basic Catalog and Custom Catalog documentation pages.

| # | Task | Details |
|---|------|---------|
| 4.1 | `BasicCatalogPage` widget | Left sidebar with component list; right panel with preview + usage + props table |
| 4.2 | `CustomCatalogPage` widget | Flight Card, Sales Dashboard, and other composed examples |
| 4.3 | Component documentation data | Static data from `basic-catalog.ts` and `custom-catalog.ts` |
| 4.4 | Props table | Simple Auto-generated table using shadcn-vue `Table` |
| 4.5 | Live preview per component | Each catalog item embeds `<A2UIRenderer>` with its component JSON |

**Verification**: All basic components documented; custom components (Flight Card, Sales Dashboard) render correctly.

---

### Phase 5: Theater & Icons Pages (2-3 days)

**Goal**: Theater playback and Icons grid.

| # | Task | Details |
|---|------|---------|
| 5.1 | `IconsPage` widget | Responsive grid of icon names; click-to-copy functionality |
| 5.2 | `TheaterPage` widget shell | Top control bar (play/pause/seek/speed/reset/clear) |
| 5.3 | Theater JSONL stream panel | Left panel showing streaming JSONL chunks |
| 5.4 | Theater preview panel | Right panel with `<A2UIRenderer>` showing accumulated state |
| 5.5 | Playback simulation | `setInterval`-based chunk pushing at configurable speed |
| 5.6 | Scenario selection | Load different mock scenarios |

**Challenge**: Theater requires `setInterval`, progressive JSONL parsing, and accumulated state updates — all highly dynamic.

**Mitigation**: Implement Theater's core logic as an embedded Vue component (`TheaterPlayer.vue`) that accepts scenario data as props. Auto generates the shell and controls; the player component handles the dynamic logic.

**Verification**: Play button streams JSONL chunks; preview updates incrementally; seek/speed controls work.

---

### Phase 6: State Management & Persistence (2-3 days)

**Goal**: Replace Pinia with Auto's reactive state and add persistence.

| # | Task | Details |
|---|------|---------|
| 6.1 | Widget model | `Widget` type with `id`, `name`, `components`, `dataModel` |
| 6.2 | Widget CRUD | Create, update, delete, set active |
| 6.3 | Navigation state | Active route tracking |
| 6.4 | localStorage persistence | Auto-save widgets to localStorage |
| 6.5 | Hydration | Load widgets from localStorage on app init |

**Challenge**: Auto may not have native `localStorage` API.

**Mitigation**: 
- Use `#[js]` annotation to emit raw JS for localStorage access
- Or implement a small JS bridge module imported by the generated Vue app

**Verification**: Creating a widget persists after refresh; sidebar shows persisted widgets.

---

### Phase 7: Polish & Feature Parity (3-5 days)

**Goal**: Match a3ui's visual polish and remaining features.

| # | Task | Details |
|---|------|---------|
| 7.1 | Toast notifications | Use shadcn-vue `Toaster` for copy success, errors |
| 7.2 | Modal dialogs | Dialog for confirmations |
| 7.3 | Responsive design | Mobile drawer, collapsible sidebar |
| 7.4 | Animations | Page transitions, card hover effects, toast slide-in |
| 7.5 | Keyboard shortcuts | Ctrl+Enter to submit Create page |
| 7.6 | Search/filter | Filter gallery widgets by name |
| 7.7 | External links | Tutorial link to CopilotKit docs |
| 7.8 | Empty states | "No widgets yet" message |

---

## Risk Analysis

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Auto parser cannot express complex layouts (3-panel, percentage widths) | Medium | High | Use embedded Vue components for complex layouts; or extend a2vue generator |
| Auto cannot import external Vue components (A2UIRenderer) | Medium | High | Post-process generated files; or add `extern component` support to Auto parser |
| Auto model/state lacks features for widget CRUD | Medium | High | Use flat boolean flags (Plan 217 approach) or arrays of simple types |
| JSON parsing/clipboard APIs unavailable in Auto | High | Medium | Use `#[js]` escape hatch or JS bridge |
| localStorage persistence not supported | Medium | Medium | JS bridge or cookie-based fallback |
| `for` loop limitations in view context | Medium | High | Simplify dynamic lists; use static enumeration where possible |
| a3ui's A2UIRenderer depends on `lucide-vue-next` icons not in component-gallery | Low | Medium | Ensure `lucide-vue-next` is in generated `package.json` |

---

## Dependencies

- `crates/auto-lang/src/ui_gen/vue.rs` — a2vue generator (must support all needed shadcn-vue components)
- `examples/component-gallery/` — Reference for complex Auto UI patterns
- `../a3ui/my-app/src/` — Source of truth for A2UIRenderer, widget data, and styling
- `docs/plans/old/217-a2ui-composer-implementation.md` — Lessons learned from prior attempt

---

## Success Criteria

1. `auto build` in `examples/a3ui-replica/` produces a working Vue 3 app
2. All 7 pages are navigable and visually match a3ui
3. Gallery shows 30+ widgets with live A2UI previews
4. Widget Editor supports JSON editing with live preview
5. Theater supports JSONL playback with controls
6. Widgets persist across page refreshes (localStorage)
7. The generated app is visually indistinguishable from a3ui at first glance

---

## Estimated Effort

| Phase | Days | Cumulative |
|-------|------|------------|
| Phase 0: Scaffolding | 1-2 | 2 |
| Phase 1: Shell & Nav | 2-3 | 5 |
| Phase 2: Create & Gallery | 2-3 | 8 |
| Phase 3: Widget Editor | 3-4 | 12 |
| Phase 4: Catalogs | 2-3 | 15 |
| Phase 5: Theater & Icons | 2-3 | 18 |
| Phase 6: State & Persistence | 2-3 | 21 |
| Phase 7: Polish | 3-5 | 26 |

**Total: ~4-5 weeks** (assuming full-time focus, accounting for Auto language limitations and iteration)

---

## Appendix A: A3UI → Auto Component Mapping

### Layout (Auto native)
| a3ui | Auto |
|------|------|
| `<div class="flex flex-col">` | `col {}` |
| `<div class="flex">` | `row {}` |
| `<div class="grid ...">` | `div (style: "grid ...") {}` |
| `<aside>` (sidebar) | Custom widget or `div` |

### shadcn-vue (Auto native via a2vue)
| a3ui | Auto |
|------|------|
| `Button` | `button {}` |
| `Input` | `input {}` |
| `Textarea` | `textarea {}` |
| `Tabs` | `tabs {}` / `tab {}` |
| `Badge` | `badge {}` |
| `Dialog` | `modal {}` |
| `Slider` | `slider {}` |
| `Table` | `table {}` |
| `Select` | `select {}` |
| `Card` | `card {}` |
| `Separator` | `divider {}` |
| `ScrollArea` | `scroll {}` |
| `Tooltip` | `tooltip {}` |
| `Toast` | `toast {}` |

### Embedded Vue (requires `extern` or post-processing)
| a3ui | Strategy |
|------|----------|
| `A2UIRenderer` | Pre-built component copied to gen output |
| `JsonEditor` | Pre-built component with line numbers |
| `TheaterPlayer` | Pre-built component for JSONL streaming |
| `CodeBlock` | May use shadcn-vue + prismjs, or pre-built |

---

## Appendix B: Files to Create

```
examples/a3ui-replica/
├── pac.at                              # Workspace config
├── source/
│   └── front/
│       ├── app.at                      # Root app with routes
│       ├── pac.at                      # Front-end config
│       ├── widgets/
│       │   ├── app_layout.at
│       │   ├── app_sidebar.at
│       │   └── page_header.at
│       ├── pages/
│       │   ├── create_page.at
│       │   ├── gallery_page.at
│       │   ├── widget_editor_page.at
│       │   ├── basic_catalog_page.at
│       │   ├── custom_catalog_page.at
│       │   ├── icons_page.at
│       │   └── theater_page.at
│       ├── components/
│       │   ├── a2ui_renderer_proxy.at  # Wrapper for external A2UIRenderer
│       │   ├── gallery_card.at
│       │   ├── code_block_proxy.at
│       │   └── props_table.at
│       ├── data/
│       │   ├── gallery_widgets.at      # Static widget definitions
│       │   ├── basic_catalog.at        # Component docs
│       │   ├── custom_catalog.at       # Custom component docs
│       │   ├── icons.at                # Icon name list
│       │   └── theater_scenarios.at    # Mock JSONL scenarios
│       └── stores/
│           └── widget_store.at         # Auto equivalent of Pinia store
```

---

## Appendix C: A2UI Renderer Integration Detail

The A2UIRenderer from a3ui must be available in the generated Vue project. Two approaches:

### Approach A: Post-Generation Copy (Recommended)
After `auto build` generates the Vue project, copy pre-built Vue components into `gen/vue/src/components/external/`:

```bash
# In build script or post-build hook
cp -r ../a3ui/my-app/src/components/a2ui-renderers/* gen/vue/src/components/external/
cp ../a3ui/my-app/src/components/A2UIRenderer.vue gen/vue/src/components/external/
cp ../a3ui/my-app/src/types/a2ui.ts gen/vue/src/types/
```

Then reference them in Auto using a special syntax or placeholder:
```auto
// In widget_editor_page.at
view {
    // ...
    div (style: "preview-area") {
        // This generates a comment/placeholder that post-processing replaces
        // with <A2UIRenderer :components="..." :data-model="..." />
        a2ui-renderer (components: .widgetComponents, dataModel: .widgetDataModel) {}
    }
}
```

### Approach B: Extend a2vue Generator
Add native support for `a2ui-renderer` tag in `crates/auto-lang/src/ui_gen/vue.rs`:
- Recognize `a2ui-renderer` as a special element
- Generate `<A2UIRenderer>` Vue component with proper prop bindings
- Add `A2UIRenderer.vue` to the generator's template files

**Recommendation**: Start with Approach A (faster), migrate to Approach B if the pattern proves valuable.
