# k1-shared-store-routing (canary, GREEN)

Two route pages share a module-level `store CounterStore` (a view-less widget:
`model` + `msg` + `on`). Both `use store: CounterStore` and read `store.count`
— validating cross-route shared state (Design 18 Rung 4 / Plan 351).

Status: **GREEN** — `auto build` + `vue-tsc` pass. The store composable is
generated as a module-level singleton (`useCounterStoreStore.ts`) and consumed
via `const store = useCounterStoreStore()` in each page.

Note (v1): store .at must be in `src/front/` root (not a `stores/` subdir —
`from_workspace` doesn't recurse). Also, store actions are called from local
msg handlers (`.Inc -> { store.Inc() }`), not directly from onclick (the
`.store.Inc` onclick syntax needs future event-handler extraction work).
