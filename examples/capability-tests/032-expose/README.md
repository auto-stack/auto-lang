# 032 — `defineExpose` (Imperative Child API via Template Refs)

A DSL capability the app-capability replicas need: an Auto widget can expose
imperative methods and refs to its parent, generating Vue
`defineExpose({ ... })` in `<script setup>`. The parent holds a template ref
on the child component and calls the exposed members through it — the
jade-garden GraphView `fit()`/`relayout()` and AutoDownEditor `$el`
scenarios.

- **`expose { ... }`** — widget-level block listing dot-prefixed member
  names (comma-separated and/or one per line). Exposable members:
  - an `on` handler (`.Fit` exposes the generated `Fit` function — it is
    emitted even when the template never references it),
  - a model field or computed (exposed as the underlying ref; Vue's expose
    proxy unwraps it on parent access),
  - a template ref declared in the view (`ref: "boxEl"`),
  - a `use { fn: ... }` imported TS function.
- **`ref: "canvasRef"` on a child component** — template-ref escape hatch
  extended to sub-widget instantiations: generates a static
  `ref="canvasRef"` attribute and a `const canvasRef = ref<any>(null)`
  declaration (`any` because the child's exposed surface is unknown to the
  parent). Handlers access it as `.canvasRef` → `canvasRef.value!`.

## Syntax

```auto
widget PanCanvas {
    on {
        .Fit -> { .zoom = 100 }
    }
    expose {
        .Fit, .Reset
        .boxEl, .zoom
    }
}

widget App {
    view {
        col {
            PanCanvas { ref: "canvasRef" }
            button "fit" { onclick: .DoFit }
        }
    }
    on {
        .DoFit -> {
            .canvasRef.Fit()
            .zoom_seen = .canvasRef.zoom
        }
    }
}
```

## Scenarios

1. **call exposed method** — the parent's buttons call
   `canvasRef.Fit()` / `canvasRef.Reset()`, imperative methods implemented
   by the child's `on` handlers.
2. **read exposed state** — the parent reads `canvasRef.zoom` (a model
   field exposed as a ref, unwrapped by Vue's expose proxy).
3. **exposed template ref** — the child also exposes `boxEl` (its root DOM
   element), so the parent could measure/manipulate the child's DOM
   directly (e.g. `canvasRef.boxEl.clientWidth`).

## Build

```sh
auto build   # → gen/front/vue, runs vue-tsc + vite build
```

## Verify

- `gen/front/vue/src/components/PanCanvas.vue` contains
  `defineExpose({ Fit, Reset, boxEl, zoom })` and emits `function Fit()` /
  `function Reset()` even though the child's template never calls them.
- `gen/front/vue/src/App.vue` contains `<PanCanvas ref="canvasRef"`,
  `const canvasRef = ref<any>(null)`, and
  `canvasRef.value!.Fit()` in the `DoFit` handler.
- `pnpm run build` (vue-tsc + vite build) passes.

## Limitations

- Expose is by name only: no type-level checking that the named member
  exists. A typo surfaces as a vue-tsc error (`TS2304: Cannot find name`)
  at build time.
- Exposed handler names keep their DSL case (`.Fit` → `Fit`); parents
  written in Auto use the same name.
