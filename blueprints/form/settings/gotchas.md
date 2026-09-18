# Gotchas — form/settings

### Auto-saving behind the user's back

**Wrong**
```auto
on .EditDisplayName(s) -> {
    .display_name = s
    save(s)   // fire-and-forget per keystroke
}
```

**Why**
Per-keystroke saves multiply requests, race each other, and make `error`
unattributable — the user can't tell which edit failed. The blueprint's
contract is explicit dirty -> save, not a write-behind cache.

**Right**
Flip `.dirty` on edit; persist once via `dataSource.save` on the Save action.
If an app genuinely needs autosave, that's a declared variant (promotions
review), not a silent local change.

### Unguarded destructive actions

**Wrong**
The danger-zone button calls the delete handler directly on click.

**Why**
Irreversible actions need a confirmation beat. Without one, a stray click is
catastrophic — and the spec's acceptance checklist fails review.

**Right**
Render destructive actions in the `danger_zone` EDIT region behind an
`alert-dialog` confirm; only the dialog's confirm handler proceeds.

### Losing the dirty state on tab switch

**Wrong**
Each tab pane keeps its own draft model, so edits vanish when the user
switches tabs and back.

**Why**
The settings screen owns ONE draft of the config; per-tab drafts fork state
(the workspace-store singleton lesson at bp scale).

**Right**
Hold a single draft model at the widget root; tabs are presentation over it.
Reset restores the last `dataSource.load` snapshot into that one draft.
