# capability-tests

Minimal Auto UI apps that each exercise **one** AutoUI platform gap from the
[025 gap enumeration](../../docs/design/autoui/025-gap-enumeration.md),
driving the gap's feature via TDD (initially RED → GREEN when implemented).
Tracked by [Plan 345](../../docs/plans/345-gap-canary-tests.md).

| Canary | Gap | Status | What it pins |
| `k1-shared-store-routing/` | K1/Rung-4 | 🟢 GREEN | shared store (cross-route state) |
|---|---|---|---|
| `k3-widget-composition/` | K3 | 🟢 GREEN | widget vs component-fn composition parity (PLAN-037) |
| `k4-ports-forwarding/` | K4/Plan 424 | 🟢 GREEN | ports re-export component/composable (`export {default as X}`) |
| `n1-contains-includes/` | N1 | 🟢 GREEN | `.contains` → JS `.includes` (str + list) |
| `k2-child-handler-binding/` | K2/N4 | 🟢 GREEN | parent↔child handler wiring (callback prop) |
| `n2-routing-codegen-paths/` | N2 | 🟢 GREEN | route page-module paths exist |
| `n3-handler-local-vars/` | N3 | 🟢 GREEN | local mutable vars in handler blocks |
| `oom-event-binop-arg/` | OOM | 🟢 GREEN | binop in msg-call event arg (parser OOM fix) |
| `vm-svg-chart-layers/` | VM-render | 🟢 GREEN | VM 轨 svg 图表绘制三层（字面量透传/静态路径/动态绑定路径，Plan 445 M3 渲染端 canary） |

Each canary's `src/front/app.at` documents the desired behavior and the
specific codegen/parser change needed ("What's needed" header comment).

## Feature fixtures (migrated from `examples/ui/` 021–040, 2026-08-23)

Single-capability showcase apps written **alongside the compiler feature they
pin** (usually added in the feature's own commit). They are not apps and are
not upgrade candidates (Plan 401 marks them ⏸ 不升级); they live here so that
`examples/ui/` stays a pure app incubator. Numeric prefixes keep the git
history traceable. Retire a fixture when its capability gains an in-tree
unit/integration test that subsumes it.

| Fixture | Pins | Origin |
|---|---|---|
| `021-block-static/` | `use store` inside blocks (EDGE-16 regression carrier) | EDGE-16 fix |
| `026-keyboard-mouse-events/` | keyboard + mouse event surfaces | capability demo batch |
| `027-native-css/` | native CSS passthrough | capability demo batch |
| `028-dom-escape/` | DOM escape-hatch | capability demo batch |
| `029-external-imports/` | `use { fn } from` TS import | capability demo batch |
| `030-custom-events-style/` | custom events + `style_obj` e2e | custom events e2e |
| `031-dyn-component-watch/` | dyn components + `watch` e2e | dyn/watch e2e |
| `032-expose/` | `defineExpose` | phase5 app capabilities |
| `033-slots/` | slots | phase5 app capabilities |
| `034-vmodel/` | `v-model` | phase5 app capabilities |
| `035-vfor-key/` | explicit `key:` in `v-for` | v-for key |
| `036-warnings/` | warning channel + silent-emission guards | plan 012 batch A |
| `037-stores-multi/` | multi-store build correctness | plan 012 batch B |
| `038-vshow/` | `v-show` | plan 012 P2 trio |
| `039-reserved-words/` | reserved-word contextualization | plan 012 P2 trio |
| `040-trycatch/` | `try/catch/finally` | plan 012 P2 trio |
| `041-model-deep-reactivity/` | unbound model var deep reactivity (`ref` vs defineModel) — runtime canary | plan 443 (filed from auto-os-config Plan 006 Phase 6) |

Verify the same way as a canary (`auto build` + `vue-tsc --noEmit`, or the
fixture's own README).

## Test probes (migrated from `examples/ui/`, 2026-09-05, PLAN-552)

App-shaped test corpora that never belonged in the app track — bare ids, no
tutorial value, not even gallery material. The desktop registry and the
ui-gallery both scan `examples/ui/` only, so probes living here are invisible
to both (by design). p507/p515 are still consumed by the queue e2e suite via
`crates/auto-lang/src/ui/desktop_protocol/stage3.rs` (`example_source`
resolves `examples/ui` → this directory); 459-dual-app remains embedded by
`crates/auto-lang/examples/ui_desktop.rs` / `ui_dual_app.rs` via `include_str!`.

| Probe | Role | Origin |
|---|---|---|
| `overlay-probe/` | alert-dialog 模态浮层解释模式探针 | Plan 533 T2 |
| `p051-min-ta/` | textarea 分层二分诊断语料（absolute 分层技巧先例） | Plan 051 Phase 2 |
| `p493-color-check/` | `@assistant` 字形颜色检查器具（根 + 子部件两形态） | Plan 493 |
| `p507-tier-coverage/` | Tier1+2 全家福构造示例（实机 queue e2e 语料） | Plan 507 T3/T10 |
| `p515-scroll-overflow/` | scrollable 溢出裁剪构造示例（scissor 栈 e2e 语料） | Plan 515 G1 |
| `p518-glass-sample/` | backdrop 毛玻璃样张（双端对拍已知分歧白名单，KNOWN-DEBT P518） | Plan 518 G8 |
| `459-dual-app/` | 无 pac.at 回退形态载具（同源双实例双窗 demo） | Plan 459 |
| `042-two-inputs-child/` | 双输入子 widget 焦点 / Tab 遍历语料 | Plan 483 |

## Verify a canary

```bash
cd <canary>
auto build                    # generates gen/front/vue
(cd gen/front/vue && npx vue-tsc --noEmit && echo GREEN)
```

Note: `auto build` reports "built successfully" even when vue-tsc fails —
the GREEN gate is `vue-tsc --noEmit` (the lesson from gap N1 / 025).
