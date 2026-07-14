# Gotchas — editor/note-editor

### Forgetting to bind the `content` prop

**Wrong**
```auto
autodown_editor {}
```

**Why**
With no `content` binding the editor renders blank — the note body never reaches
it. Users see an empty editor even though `.body` holds the document.

**Right**
Render `autodown_editor { content: .body, onupdate: .BodyChanged }` so the
editor's document is driven by model state and edits flow back via `@update`.

### Not handling the save event

**Wrong**
Rendering the editor and title inputs but never wiring `@save` / `@update`, so
edits sit in local model state and vanish on navigation.

**Why**
The block owns the chrome but not persistence; without routing edits to a msg
handler that calls `dataSource.save`, every keystroke is lost the moment the
consumer unmounts the block.

**Right**
Bind `onupdate`/`oninput` to model-mutating msgs, mark `.dirty`, and wire the
toolbar Save button to a `.Save` msg that invokes
`dataSource.save(.note.id, .title, .body)` then clears `.dirty`.

### Hardcoding the save endpoint instead of using dataSource

**Wrong**
```auto
on .Save -> { api_post("/notes/" + .note.id, .body) }
```

**Why**
The block is now welded to one backend and one URL shape. It can't be reused
for a different API, a mock for tests, or an offline-first store.

**Right**
Declare `dataSource.save = "(id: int, title: str, body: str) -> Note"` in the
spec; the consumer binds their `#[api]` fetcher to it. The block only owns the
editor chrome and the save state machine.
