# Plan: shadcn-vue Charts Replica for Auto (a2vue/a2ts)

## Goal
Add first-class chart widget support to Auto so that `area-chart`, `bar-chart`, `line-chart`, and `donut-chart` can be used directly in Auto `view {}` blocks, compiled by a2vue into shadcn-vue/Unovis-based Vue components.

## Background
- shadcn-vue v3 (legacy, high-level) provides `AreaChart`, `BarChart`, `LineChart`, `DonutChart` built on `@unovis/vue`.
- Auto's a2vue generator uses a `WidgetRegistry` (`registry.rs`) + per-tag prop mapping (`generate_shadcn_attrs` in `vue.rs`) to emit Vue SFCs.
- Charts currently do not exist in the modern registry. Only a deprecated stub exists.

---

## Approach: Registry + Generator Extension (Recommended)

Make charts native Auto widgets, consistent with how `button`, `card`, `table`, etc. work.

### Phase 1: Widget Registry Entries
**File:** `crates/auto-lang/src/ui_gen/widget/registry.rs`

Add 4 new data widgets with Vue backend mappings:

| Auto Tag | Vue Component | Import Path |
|----------|---------------|-------------|
| `area-chart` | `AreaChart` | `@/components/ui/chart-area` |
| `bar-chart` | `BarChart` | `@/components/ui/chart-bar` |
| `line-chart` | `LineChart` | `@/components/ui/chart-line` |
| `donut-chart` | `DonutChart` | `@/components/ui/chart-donut` |

Also add a shared `chart` tag dependency mapping for `ChartContainer`, `ChartTooltip`, `ChartLegend` (already partially present in deprecated registry; migrate/ensure in modern registry).

### Phase 2: Generator Prop Mapping
**File:** `crates/auto-lang/src/ui_gen/vue.rs` in `generate_shadcn_attrs`

Add match arms for `area_chart`, `bar_chart`, `line_chart`, `donut_chart`.

**Mapped props (all charts):**
- `data` → `:data="..."` (binds to model ref; arrays are already `ref<any[]>`)
- `categories` → `:categories="..."` (array of strings)
- `index` → `index="..."` (string key)
- `colors` → `:colors="..."` (string array)
- `margin` → `:margin="..."` (object, e.g. `{ top: 10 }`)
- `filter-opacity` / `filterOpacity` → `:filter-opacity="..."`
- `show-x-axis` / `showXAxis` → `:show-x-axis="..."` (bool)
- `show-y-axis`, `show-tooltip`, `show-legend`, `show-grid-line` similarly
- `x-formatter` / `xFormatter` → `:x-formatter="..."` (function ref from model)
- `y-formatter` / `yFormatter` → `:y-formatter="..."`

**Chart-specific props:**
- `area-chart`: `curve-type`, `show-gradient`
- `bar-chart`: `type` (`stacked`/`grouped`), `rounded-corners`
- `line-chart`: `curve-type`
- `donut-chart`: `category`, `type` (`donut`/`pie`), `value-formatter`, `sort-function`

**Challenge — Function Props:**
Auto `view {}` props are static values or model references. Functions (formatters) cannot be declared inline. **Solution:** Pass them as model references. In the model, users define formatter names that resolve to handler functions in `on {}`, or we accept string expressions and transpile them via a2ts. For v1, we support model-bound formatter references (e.g., `xFormatter: .myFormatter` where `myFormatter` is a `ref` in the model).

**Challenge — Custom Tooltip:**
`customTooltip` accepts a Vue component. In Auto, this maps to a PascalCase custom component reference passed as a prop (e.g., `custom-tooltip: MyTooltip`). The generator already auto-imports PascalCase tags; we extend prop handling to emit `:custom-tooltip="MyTooltip"` when the value is PascalCase.

### Phase 3: CSS / Theme Setup
The shadcn-vue charts require Unovis CSS variables:

```css
@layer base {
  :root {
    --vis-tooltip-background-color: none !important;
    --vis-tooltip-border-color: none !important;
    --vis-tooltip-text-color: none !important;
    --vis-tooltip-shadow-color: none !important;
    --vis-tooltip-backdrop-filter: none !important;
    --vis-tooltip-padding: none !important;
    --vis-primary-color: var(--primary);
    --vis-secondary-color: 160 81% 40%;
    --vis-text-color: var(--muted-foreground);
  }
}
```

**Action:** Verify these are present in the `auto vue` project template (`index.css` or equivalent). If missing, add them to the base CSS template used by the CLI.

### Phase 4: Dependency Installation
The `auto vue` build pipeline runs `npx shadcn-vue add <component>`. Ensure chart components trigger installation of:
- `@unovis/vue`
- `@unovis/ts`

The shadcn-vue CLI should handle peer dependencies automatically when adding `chart-area`, etc. Verify in `crates/auto/src/cmd_vue.rs` or the build pipeline that these are installable.

### Phase 5: Example Gallery
Create `examples/charts-gallery/` (or extend `examples/component-gallery/`) with:

1. **Basic Area Chart** — monthly revenue data, default styling
2. **Bar Chart (Grouped)** — multi-series comparison
3. **Bar Chart (Stacked)** — `type: "stacked"`
4. **Line Chart** — trend over time with `curve-type: "monotone"`
5. **Donut Chart** — category breakdown
6. **Sparkline** — `show-x-axis: false`, `show-y-axis: false`, `show-tooltip: false`, `show-legend: false`
7. **Chart with Colors** — custom `colors` array
8. **Dashboard Block** — replace the placeholder in `dashboard_01.at` with a real `area-chart`

### Phase 6: Testing & Validation
- Run `cargo test` for the Rust generator changes
- Run `auto vue` in the example project and verify the generated `.vue` files:
  - Correct imports from `@/components/ui/chart-*`
  - Proper prop bindings (`:data`, `:categories`, `index`)
  - No TypeScript errors in generated code
- Visually verify charts render in browser

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/auto-lang/src/ui_gen/widget/registry.rs` | Add `area-chart`, `bar-chart`, `line-chart`, `donut-chart` widget specs |
| `crates/auto-lang/src/ui_gen/vue.rs` | Add `generate_shadcn_attrs` match arms for chart tags and their props |
| `crates/auto/src/cmd_vue.rs` (or CSS template) | Ensure Unovis CSS variables are in generated project |
| `examples/component-gallery/source/front/pages/blocks/dashboard_01.at` | Replace chart placeholder with real `area-chart` |
| `examples/charts-gallery/` (new) | Demo pages for each chart type |

---

## Open Questions / Risks

1. **Function prop binding:** Auto's view prop system is value-oriented. Model-bound function refs (`:x-formatter="myFn"`) need verification that the generator emits valid TypeScript when a model field holds a function type.
2. **Custom tooltip as component prop:** Need to confirm the generator can pass a PascalCase component name as a bound prop value.
3. **shadcn-vue v3 vs v4:** If the project ever upgrades to shadcn-vue v4, the high-level `AreaChart`/`BarChart` components disappear. We would need to either (a) pin to v3 or (b) later rewrite to generate raw Unovis composition. For now, targeting v3 is correct because the existing project uses `tailwindcss: ^3.4.0`.
