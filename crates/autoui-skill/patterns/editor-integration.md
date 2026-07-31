# Pattern: AutoDownEditor Integration

## Use case

Integrating the AutoDownEditor rich-text component. This is the most
error-prone pattern in 015-notes due to Tiptap lifecycle complexity.

## The golden rule

**Single instance + prop toggle, never dual instance v-if.**

## Standard .at structure

```auto
widget EditorPanel {
    use store: NotesStore

    model {
        var editing bool = false
        var edit_title str = ""
        var edit_body str = ""
    }

    view {
        col {
            // Header with Edit/Save/Cancel buttons
            row {
                if .editing {
                    button "Save"   { onclick: .Save }
                    button "Cancel" { onclick: .Cancel }
                }
                if !.editing {
                    button "Edit" { onclick: .Edit }
                }
            }

            // Editor — SINGLE instance, mode controlled by can_edit prop
            autodown_editor {
                content: if .editing { .edit_body } else { .note.body },
                can_edit: .editing,
            }
        }
    }

    on {
        .Edit -> {
            .edit_title = .note.title
            .edit_body = .note.body
            .editing = true
        }
        .Save -> {
            store.UpdateNote(.note.id, .edit_title, .edit_body)
            .editing = false
        }
        .Cancel -> {
            .editing = false
        }
    }
}
```

## Key constraints

- **Single `autodown_editor` instance** — NEVER create two in v-if branches
- `can_edit` prop controls read/edit mode, not v-if switching
- Content binding: `content: if .editing { .edit_body } else { .note.body }`
- Edit/Cancel/Save handlers manage `.editing` state

## Pitfalls

- **P1**: Dual instance v-if switching breaks Tiptap (most common bug)
- **C1/C7**: Even though the generator now assigns unique keys to dual
  instances, prefer the single-instance approach for state continuity
- Edit/Save/Cancel all need to properly manage `.editing` state
- Make sure the note content is copied to `.edit_body` on Edit, and
  restored/discarded on Cancel

## Validation checklist

- [ ] Only ONE `autodown_editor` in the template (R007)
- [ ] `can_edit` prop drives read/edit mode (not v-if)
- [ ] Edit copies current content to edit buffers
- [ ] Cancel properly discards edits
- [ ] Save persists to backend via store
