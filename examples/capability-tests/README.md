# capability-tests

Minimal Auto UI apps that each exercise **one** AutoUI platform gap from the
[025 gap enumeration](../../docs/design/16-appendix-025-gap-enumeration.md),
driving the gap's feature via TDD (initially RED → GREEN when implemented).
Tracked by [Plan 345](../../docs/plans/345-gap-canary-tests.md).

| Canary | Gap | Status | What it pins |
|---|---|---|---|
| `n1-contains-includes/` | N1 | 🟢 GREEN | `.contains` → JS `.includes` (str + list) |
| `k2-child-handler-binding/` | K2/N4 | 🟢 GREEN | parent↔child handler wiring (callback prop) |
| `n2-routing-codegen-paths/` | N2 | 🟢 GREEN | route page-module paths exist |
| `n3-handler-local-vars/` | N3 | 🟢 GREEN | local mutable vars in handler blocks |
| `oom-event-binop-arg/` | OOM | 🟢 GREEN | binop in msg-call event arg (parser OOM fix) |

Each canary's `src/front/app.at` documents the desired behavior and the
specific codegen/parser change needed ("What's needed" header comment).

## Verify a canary

```bash
cd <canary>
auto build                    # generates gen/front/vue
(cd gen/front/vue && npx vue-tsc --noEmit && echo GREEN)
```

Note: `auto build` reports "built successfully" even when vue-tsc fails —
the GREEN gate is `vue-tsc --noEmit` (the lesson from gap N1 / 025).
