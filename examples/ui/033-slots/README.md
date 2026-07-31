# 033 — Slots (Default Outlet + Named-Slot Targeting)

Vue-style slots for Auto widgets: a widget declares **slot outlets** in its
own view, and a parent **targets** them from the children block of a widget
instantiation.

- **`slot`** — default outlet in a widget view → `<slot />`. Plain children
  passed to the widget render here (behavior unchanged from before; the
  difference is that `slot` now compiles to a real outlet instead of being
  silently swallowed by the unknown-tag `<div />` fallback).
- **`slot(name: "header")`** — named outlet in a widget view →
  `<slot name="header" />`.
- **`slot(name: "header") { ... }` inside an instantiation's children
  block** — named-slot targeting at the parent side →
  `<template #header>...</template>` inside `<Panel>...</Panel>`.
- **`slot { ... }` at the parent side** — sugar: children are unwrapped
  into the default slot (same as listing them directly).
- **Build-time warning (no hard error)** — when app.at passes default-slot
  children or a `slot(name:)` template to a sub-widget that declares no
  matching outlet, `auto build` prints a `Warning:` line (children would
  not render).

## Syntax

```auto
widget Panel(title: str) {
    view {
        col {
            row {
                text .title { }
                slot(name: "header")   // → <slot name="header" />
            }
            col {
                slot                   // → <slot />
            }
        }
    }
}

widget App {
    view {
        col {
            Panel(title: "Settings") {
                slot(name: "header") { // → <template #header>
                    text "3 pending" { }
                }
                text "default-slot body"
            }
        }
    }
}
```

## Scenarios

1. **default outlet** — the body text and button passed to `<Panel>` render
   at Panel's `<slot />`.
2. **named outlet** — the "3 pending" badge is wrapped in
   `<template #header>` and renders at Panel's `<slot name="header" />`.
3. **props + slots together** — `title: "Settings"` is an ordinary widget
   prop; slots compose with it freely.

## Build

```sh
auto build   # → gen/front/vue, runs vue-tsc + vite build
```

## Verify

- `gen/front/vue/src/components/Panel.vue` contains
  `<slot name="header" />` and `<slot />`.
- `gen/front/vue/src/App.vue` contains `<template #header>` inside
  `<Panel ...>...</Panel>` with the default-slot children after it.
- `pnpm run build` (vue-tsc + vite build) passes.

## Limitations

- `slot` in a widget's own view may carry children — they become Vue
  **fallback content** (`<slot>fallback</slot>`). At the parent side a
  childless `slot(name:)` emits an empty `<template #name>`.
- Slot outlet detection is syntactic: a `slot` element hidden behind
  `if`/`for` still counts as an outlet for the warning check (Vue renders
  it conditionally at runtime).
- External components declared via `use { component: ... }` are not
  warning-checked — their slot surface is unknowable to the compiler.
- No scoped slots (slot props) yet.
