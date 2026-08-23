# 038 — v-show (`show:` prop)

Visibility toggling **without unmounting** — the DSL form of Vue's `v-show`
(gap 52). Until now jade-garden replicated `v-show` by hand with
`style_obj: { display: ... }` (MainArea keep-alive tabs); `show:` is the
first-class form.

## Syntax

Any element, component instantiation, or `dyn` node may take a `show:` prop
whose value is an arbitrary bound expression:

```auto
div(show: .active_path == "graph", class: "graph-pane") { ... }
// → <div v-show="active_path == 'graph'" class="graph-pane"> ... </div>

EditorTab(key: tab.path, path: tab.path, show: tab.path == .active_path)
// → <EditorTab v-show="tab.path == active_path" ... />

dyn (.Teleport) { show: .open, ... }
// → <component :is="(Teleport) as any" v-show="open" ... />
```

Brace prop form works too:

```auto
div {
    show: .visible
    text "hi"
}
```

## Semantics

- The node **stays mounted**; Vue only toggles inline `display`. Component
  state, DOM state (scroll, editor contents), and lifecycle hooks are
  preserved — unlike `if`/`v-if`, which destroys the subtree.
- `v-show` on a component lands on its root element (standard Vue behavior).

Use `show:` when toggling frequently or preserving state matters; keep `if`
when the hidden subtree is expensive or should truly unmount.
