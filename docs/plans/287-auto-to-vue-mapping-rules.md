# Plan 287 - Auto Widget → Vue Component Transpilation Rules

## Date: 2026-06-08

## Overview

Define the rules for how AutoLang widget files (`.at`) are transpiled into Vue SFC (`.vue`) files. The design principle: **AutoLang has its own module/widget model; the Vue output is an adaptation, not the source of truth.**

## Auto's Module/Widget Model

- **File = Module**: `editor.at` defines the module `editor`
- **Widget = Type**: `widget EditorPanel` inside `editor.at` has the full name `editor.EditorPanel`
- **One module can have multiple widgets**:
  ```auto
  // editor.at
  widget EditorPanel(note: Note) { ... }
  widget EditorToolbar(actions: List) { ... }
  ```
- **Import syntax**:
  ```auto
  use editor: EditorPanel          // single import
  use editor: EditorPanel, Toolbar // multiple imports
  ```

## Vue Transpilation Rules

### Rule 1: One Widget → One .vue File

Each widget generates exactly one `.vue` SFC file. Vue requires one component per file — this is a Vue constraint, not Auto's.

### Rule 2: Module-Based Directory Layout

Output files are organized into directories matching the module name:

```
editor.at  →  components/editor/EditorPanel.vue
                components/editor/EditorToolbar.vue

sidebar.at →  components/sidebar/Sidebar.vue

app.at     →  App.vue   (entry point, top-level)
```

General pattern:
```
src/{module}.at        →  src/components/{module}/{WidgetName}.vue
src/front/{module}.at  →  src/components/{module}/{WidgetName}.vue
```

### Rule 3: Import Path Generation

`use editor: EditorPanel` in Auto → Vue import:

```vue
import EditorPanel from '@/components/editor/EditorPanel.vue'
```

### Rule 4: Entry Point (app.at)

The `app.at` file is the entry point and generates `App.vue` at the top level. It is NOT placed in a subdirectory.

### Rule 5: shadcn-vue Components

UI primitives (Button, Input, Textarea, etc.) remain at `@/components/ui/` — they are not part of Auto's widget system and are not affected by these rules.

## File Structure Summary

```
src/front/
├── app.at              → App.vue
├── editor.at           → components/editor/EditorPanel.vue
├── sidebar.at          → components/sidebar/Sidebar.vue
├── note_item.at        → components/note_item/NoteItem.vue
└── types.at            → (no .vue output — type definitions only)

src/components/
├── editor/
│   └── EditorPanel.vue
├── sidebar/
│   └── Sidebar.vue
├── note_item/
│   └── NoteItem.vue
└── ui/                 ← shadcn-vue primitives (unchanged)
    ├── button/
    ├── input/
    └── textarea/
```

## Implementation Notes

### Build Pipeline Changes (crates/auto-man/src/vue.rs)

1. **Read `use` statements** from `app.at` using `use_scanner::scan_use_statements()`
2. **For each `use module: WidgetName`**:
   - Locate `{module}.at` in the same directory
   - Parse it to extract the widget declaration
   - Generate `components/{module}/{WidgetName}.vue`
3. **Pass widget names** to `VueGenerator::with_sub_widgets()` for import generation
4. **Generate imports** using the `@/components/{module}/{WidgetName}.vue` path pattern

### Vue Generator Changes (crates/auto-lang/src/ui_gen/vue.rs)

- Update import path generation: `@/components/{module}/{WidgetName}.vue`
- Track which module each widget belongs to

### Fallback: Files Without `use` Statements

For backwards compatibility, if `app.at` has no `use` statements, fall back to the current directory-scanning behavior (scan all `.at` files in `front/`).
