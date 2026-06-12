# 013-todo — TodoMVC

A complete [TodoMVC](https://todomvc.com) implementation built with Auto Language, demonstrating full-featured UI development with the AURA widget system and Vue code generation.

## Features

All standard TodoMVC features are implemented:

- **Add todo** — Type text and press Enter
- **Toggle todo** — Click checkbox to mark as done/undone
- **Toggle all** — Top checkbox toggles all todos at once
- **Delete todo** — Hover to reveal destroy button (×)
- **Edit todo** — Double-click todo text to enter edit mode
- **Commit edit** — Press Enter or click outside to save
- **Clear completed** — Remove all completed todos
- **Filter** — All / Active / Completed filter buttons
- **Item count** — Live count of remaining active items

## Concepts

- **Array model** — `var todos = []` with dynamic push/splice operations
- **Typed message variants** — `msg Msg { ToggleTodo(int), DeleteTodo(int), ... }` with parameterized events
- **For-loop rendering** — `for todo in .todos { ... }` iterates over todo list
- **Conditional rendering** — `if .filter == "all"` for filter-based visibility
- **Event handlers** — `onclick`, `ondblclick`, `onenter`, `onblur`, `oninput`
- **F-string interpolation** — `f"${.active_count} items left"` for dynamic text
- **Computed state** — `active_count` tracked incrementally on toggle/delete
- **CSS integration** — Inline todomvc-app-css styles for authentic TodoMVC appearance

## Source

See `src/front/app.at` — the entire TodoMVC application in ~285 lines of Auto code.

### Model

```auto
model {
    var input str = ""
    var todos = []
    var next_id int = 1
    var filter str = "all"
    var editing_id int = -1
    var edit_text str = ""
    var active_count int = 0
}
```

### Messages

```auto
msg Msg {
    Init,
    AddTodo,
    ToggleTodo(int),
    DeleteTodo(int),
    ToggleAll,
    EditTodo(int),
    CommitEdit,
    EditInputChanged,
    FilterAll,
    FilterActive,
    FilterCompleted,
    ClearCompleted
}
```

## How to Run

```bash
cd examples/ui/013-todo
auto gen              # Generate Vue project
auto run              # Start dev server (localhost:3001)
```

Generated project appears in `gen/front/vue/` — Vue 3 + shadcn-vue + Vite.

## Browser Verification

All TodoMVC features have been verified via automated browser testing:

| Feature | Status |
|---------|--------|
| Add todo (Enter) | ✅ |
| Toggle single todo | ✅ |
| Delete todo | ✅ |
| Toggle all | ✅ |
| Filter: All / Active / Completed | ✅ |
| Clear completed | ✅ |
| Double-click to edit | ✅ |
| Active item count | ✅ |

## Known Limitations

- **No Escape to cancel edit** — AURA has no `onescape` event mapping yet
- **Edit text update** — `EditInputChanged` handler is a no-op (`input = input`); relies on Vue `v-model` for actual state sync
- **No route-based filtering** — Uses `filter` state variable instead of Vue Router
- **No localStorage persistence** — Data is lost on page refresh (no storage binding in AURA yet)

## Inspiration

[TodoMVC](https://todomvc.com) — Reference implementation based on `todomvc/examples/vue`.
