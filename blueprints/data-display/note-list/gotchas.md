# Gotchas — data-display/note-list

### Hardcoding the data source

**Wrong**
```auto
on .Init -> { .items = db_all_notes() }
```

**Why**
The blueprint now depends on a specific backend function. It can't be reused for
a different collection or app.

**Right**
Declare `dataSource.list` in the spec; the consumer binds their `#[api]` fetcher
to it. The blueprint only owns the list/search/state-machine.

### Forgetting empty / loading / error branches

**Wrong**
Only rendering `for note in .items` — nothing for the loading fetch, the failed
fetch, or an empty collection.

**Why**
These are part of the blueprint's behavior contract (spec `acceptance`), not
afterthoughts. A data-display blueprint without them is broken by definition.

**Right**
Always render all three branches (`loading`, `error != ""`, empty), each in its
own EDIT region.

### Missing the list key

**Wrong**
Rendering a `for` loop with no stable per-item identity.

**Why**
Selection / reconciliation breaks (active item drifts on re-render).

**Right**
Each item carries a stable `id`; selection is `SelectItem(id)`, never an index.
