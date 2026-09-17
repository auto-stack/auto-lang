# Gotchas — dashboard/overview

### Faking charts inside the blueprint

**Wrong**
Inlining hand-drawn bars / fake sparklines so the bp "looks complete"
without chart packages.

**Why**
Chart tags are not WidgetRegistry-registered (PLAN-484 retirement; PLAN-640
DEBT). Fake chart markup dodges the palette-drift gate and ships pixels
that lie about data. The `charts` region exists precisely so real chart bps
mount cleanly later.

**Right**
Keep `charts` a declared EDIT slot and mount official chart packages at the
app layer. An empty slot with a placeholder beats a lying chart.

### Metrics without a loading skeleton

**Wrong**
Binding stat cards straight to zero-valued model fields.

**Why**
Zeros read as "no users, no revenue" — a data-dependent lie during fetch.
The acceptance list requires skeletons until `dataSource.metrics` resolves.

**Right**
Render `skeleton` cards while `.loading`; swap to the fetched values on
resolve and surface failures via the error branch.

### Period state that refresh ignores

**Wrong**
Refresh re-fetches a default period while the header select shows another.

**Why**
The header is the single source of period truth. Divergent refresh makes
the dashboard inconsistent — numbers won't match the visible selector.

**Right**
`RefreshClicked` and `PeriodChanged` both call `dataSource.metrics(.period)`
— the model's period drives every fetch.
