# 035 — v-for `:key` Override

Explicit `:key` support for view `for` loops.

## The problem

The Vue codegen auto-emits a `:key` on component instantiations inside loops:

```auto
for tab in .tabs {
    EditorTab(path: tab.path)
}
// → <EditorTab :path="tab.path"
//     :key="'EditorTab-1-' + (tab?.id ?? tab)" v-for="tab in tabs" />
```

When the item type has **no `id` field**, `tab?.id` is `undefined` for every
item, so the fallback stringifies the object itself — every iteration gets
the same constant key (`'EditorTab-1-[object Object]'`). Vue then reuses a
single component instance across all iterations, breaking per-instance
state (this killed the v-show keep-alive contract of editor tabs in
jade-garden).

## The fix: `key:` prop on the loop body

Any node in a loop body may declare an explicit `key:` prop. It is emitted
as `:key="<expr>"` and **wins** over the auto-generated key — no duplicate
attribute, no `?.id` chain:

```auto
for tab in .tabs {
    EditorTab(key: tab.path, path: tab.path, label: tab.label)
}
// → <EditorTab :key="tab.path" :label="tab.label" :path="tab.path"
//     v-for="tab in tabs" />
```

Works on component instantiations (known sub-widgets, external `use`
components, generic Vue components) and on plain elements:

```auto
for name in .names {
    span(key: name) { text "x" }
}
// → <span :key="name" v-for="name in names">…</span>
```

## Index-var fallback fix

In an indexed loop (`for i, tab in .tabs`), the loop var is the primitive
int index — the old heuristic emitted the meaningless `i?.id`. The auto-key
now uses the index itself:

```auto
for i, tab in .tabs {
    EditorTab(path: tab.path)
}
// → :key="'EditorTab-1-' + i"   (was: 'EditorTab-1-' + (i?.id ?? i))
```

## This example

`src/front/app.at` renders a tab strip of `EditorTab` child components over
`TabInfo[]` data (`src/front/utils/tabs.ts`) that deliberately has **no
`id` field** — the exact shape that breaks the auto-key heuristic. The
generated SFC (`gen/front/vue/src/App.vue`) shows a single
`:key="tab.path"` per tab.

Build:

```sh
auto build -d .
```
