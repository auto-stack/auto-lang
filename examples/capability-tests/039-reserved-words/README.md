# 039 — Reserved-word contextualization

`link`, `task`, `type`, `to`, `map` used to be hard collisions: naming a
loop variable `link` misparsed the loop body as router-link garbage, and
`button { type: "button" }` silently emitted `<div>button</div>` junk
(jade-garden gaps 18/27/29/34/43/53). These tokens are now **contextual**:

| Context | Meaning |
|---|---|
| element/tag position (`link (to: "/")`) | keyword (router-link element, task decl, …) |
| expression position (`link.title`, `if link != null`) | plain identifier |
| prop key followed by `:` (`type: "button"`) | prop name |
| prop type position (`settings: map`) | built-in map type → `any` |

## What now works

```auto
for link in .items {            // loop var named link (gap 18)
    div { text link.title }     // field access, not a router-link
}

for task in .items {            // loop var named task (gap 29)
    div { text task.name }
}

on {
    .Go -> {
        var link = .items.find(l => l.id == 1)   // local named link
        if link != null { .n = link.id }
    }
}

button {                        // brace-form keyword prop keys (gap 53);
    type: "button"              // no paren-form workaround needed anymore
    class: "x"
    text "hi"
}

dyn (.Teleport) {               // `to:` on dyn (gap 27) → :to="'body'"
    to: "body"
    text "overlay"
}

widget Editor(settings: map) {  // map prop type (gap 43) → `settings: any`,
    ...                         // no broken `import type { map }`
}
```

Still genuinely ambiguous (by design, unchanged): a **bare** `link { }` /
`link (to: ...)` in view-child position is the router-link element. Name
shadowing in *element position* is not supported — the token is only an
identifier where an expression can appear.
