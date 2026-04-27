# A2UI Composer

A visual UI builder built as an AutoUI web application. Three-panel layout with palette, canvas, and inspector.

## Build & Run

```bash
auto build
# Output: gen/vue/dist/
```

Open `gen/vue/dist/index.html` in a browser, or serve with:

```bash
cd gen/vue && bun run dev
```

## Features

- **Palette** (left): Toggle Text, Button, Input, Row, Column components
- **Canvas** (center): Shows active components in real-time
- **Inspector** (right): Lists active component status
- **Toolbar**: Export JSON (placeholder) and Clear All

## Architecture

Single-widget AutoUI app (`src/front/app.at`) compiled to Vue 3 via a2vue backend.

- Model: boolean flags per component type + `show_empty` helper
- View: conditional rendering with `if` blocks
- Handlers: toggle visibility, clear all, export placeholder

## Limitations

Current Auto parser constraints prevent the full node-tree composer (Map<K,V>, recursive widgets, dynamic lists in view). These are deferred to Phase 2+.
