# Gotchas — data-display/data-table-crud

### Forgetting to reset the page on filter change

**Wrong**
```auto
on .SearchChanged(s) -> {
    .search = s
    query(.search, .page)   // keeps the old page index
}
```

**Why**
A narrower result set can leave the user stranded past the last page — an
empty table that looks like "no data" when it's really "page 9 of 1".

**Right**
Any filter/search change resets `.page = 1` before re-running
`dataSource.query`; pagination changes keep the filters as-is.

### Optimistically deleting without a confirm or an undo path

**Wrong**
The dropdown's Delete entry removes the row from `.rows` immediately and
fires the API call.

**Why**
A mis-click destroys data with no checkpoint; a failed API call then leaves
UI and server disagreeing. Destructive actions are contract-level behavior
(spec acceptance), not polish.

**Right**
Confirm (alert-dialog) before firing `actions.delete`; only on success drop
the row / re-run `dataSource.query`. On failure surface the error and keep
the row.

### Hardcoding columns into the table

**Wrong**
Writing literal header cells for one app's schema inside the reference
implementation and ignoring `props.columns`.

**Why**
The column set is the blueprint's data interface. Hardcoding forks every
consumer and makes the `columns` prop a lie — the L3 agent reading the spec
can't trust the contract.

**Right**
Drive headers and cells from `props.columns` (the reference shows one
concrete instantiation); app-specific cell decoration belongs in the
`row_actions` / per-cell EDIT regions, not in the header definition.
