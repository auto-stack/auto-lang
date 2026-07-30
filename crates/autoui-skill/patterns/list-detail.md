# Pattern: List-Detail Layout

## Use case

Display a list of items on one side, with the selected item's details on the
other side. This is the most common pattern in 015-notes (NavTree + EditorPanel).

## Standard .at structure

```auto
widget App {
    // Store connects list and detail via active_id
    use store: NotesStore

    view {
        row {
            // Left: list
            col {
                width: "280px"
                NavTree {}
            }
            // Right: detail
            col {
                flex: 1
                EditorPanel {}
            }
        }
    }
}
```

## Key constraints

- List items use `for x in .items` — keys are auto-generated (C6)
- Detail area guards with `if .items.len() > 0` for empty state
- List and detail share active selection via store (not prop chains)
- The store's `active_id` drives which item is shown in detail

## Common variants

### Variant A: Detail in right panel (master-detail)

```auto
row {
    col { width: "250px"   NavTree {} }
    col { flex: 1           EditorPanel {} }
}
```

### Variant B: Detail in modal/dialog

```auto
col {
    NavTree {}
    if .selected != "" {
        Dialog {
            EditorPanel {}
        }
    }
}
```

## Pitfalls

- List items that are stateful components (editors) can trigger C1/C7
  issues on note switching. See [editor-integration.md](editor-integration.md).
- `active_id` should be stored in the store, not passed as a prop chain.
- Empty state: always guard the detail area with a length check.

## Validation checklist

- [ ] List items have auto-generated unique keys (R006)
- [ ] Detail area has empty-state handling
- [ ] Store's `active_id` updates correctly on selection
- [ ] No fixed keys on stateful list item components
