# n2-routing-codegen-paths (canary, GREEN)

Status: **GREEN** — `auto build` + `vue-tsc` + `vite build` all pass.

The original "gap N2" (router imports `@/pages/<name>.vue` that don't exist)
turned out to be an **undocumented convention**, not a codegen bug:

- Route-target pages MUST live in `src/front/pages/<name>.at`.
- The generator (`cmd_vue.rs`) emits each to `src/pages/<name>.vue` — the
  exact path `router/index.ts` imports.
- Declaring page widgets inline in `app.at` does NOT emit them, so the
  router's imports dangle (the RED state).

So this canary now documents the correct usage. The codegen itself is sound.
See [Plan 345](../../../docs/plans/345-gap-canary-tests.md).
