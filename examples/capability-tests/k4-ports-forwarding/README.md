# k4-ports-forwarding (canary)

Status: **GREEN** — `auto build` + `vue-tsc --noEmit` end to end.

Question (Plan 424): can a port `.at` module forward **component and
composable symbols** via ES re-export, so callers reference
`ports/<域>.at` instead of wiring npm/`.vue`/`.ts` sources at every call
site — with the caller-side machinery (tag registration, composable
auto-call + local binding) identical to a direct reference?

## Scenario

`src/front/ports/symbols.web.at` mixes all three `use.web` kinds:

- `fn relLabel from "src/front/label_fmt.ts"` → plain import + wrapper fn
  (the PLAN-037 Phase 5 path, unchanged);
- `component Badge from "src/front/badge.vue"` → `export { default as
  Badge } from '@/ext/src/front/badge.vue'` (`.vue`/`platform:` sources are
  default exports, so the re-export aliases `default`);
- `composable useTick from "src/front/composables/useTick.ts"` → named
  `export { useTick } from '@/ext/src/front/composables/useTick'`.

`app.at` references everything through the stable port name
`ports/symbols.at` (adapter selection: `symbols.web.at` wins on the vue
build — PLAN-037 Phase 6). A `.at` source transpiles to a named-export TS
module, so the caller emits named imports from the port — tag registry and
composable auto-call behave exactly as with a direct reference:

- `import { Badge } from '@/ext/src/front/ports/symbols.web'`;
- `import { useTick } from '@/ext/src/front/ports/symbols.web'` +
  `const tick = useTick()` — the auto-call lands in the CALLER's setup, so
  reactivity is preserved (the port only re-exports the real `useTick`);
- `import { portLabel } from '@/ext/src/front/ports/symbols.web'`.

Verified: `<Badge :text=... @select=...>` instantiates (tag registry keyed
off the declared name), `tick.label()` renders, `portLabel(3)` renders,
`@select` fires into `.last` — `vue-tsc --noEmit` green.

## npm sources

The npm path (lucide-vue-next, markstream-vue) is exercised by the
auto-musk migration (Plan 424 T3: icons/renderer/composables domains) —
this canary stays hermetic with local files only; unit tests pin the npm
specifier passthrough (`export { X } from 'pkg'`).

## Verify

```bash
cd <canary>
auto build
(cd gen/front/vue && npx vue-tsc --noEmit && echo GREEN)
```
