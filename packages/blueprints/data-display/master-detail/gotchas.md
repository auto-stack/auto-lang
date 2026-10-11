# Gotchas — data-display/master-detail

### Rendering a blank pane for "nothing selected"

**Wrong**
The detail pane just renders empty when `.selected_id == ""` — a white hole.

**Why**
A blank pane reads as broken. First-use users don't know selection is
expected; after a failed fetch they can't tell error from emptiness. The
no-selection state is part of the spec's acceptance list.

**Right**
Render the `empty_selection` EDIT region ("Select an item to preview…") as
an explicit state. Keep `empty_selection` / `loading` / `error` / `detail`
as four distinct branches.

### Coupling the detail fetch to the list lifecycle

**Wrong**
One `loading` flag for both panes: refetching the list blanks the detail.

**Why**
The two panes have independent lifecycles. A list refresh must not wipe an
open detail, and a detail failure must not take the list down with it —
that's the spec's independent-branches acceptance.

**Right**
`list` and `detail` each own their loading/error state. Only `Select(id)`
starts a detail fetch; only search changes start a list fetch.

### Keeping the two-pane layout on a phone

**Wrong**
Fixing `row` layout regardless of viewport — two 50% panes on a 375px
screen.

**Why**
Below the declared breakpoint the panes become unusably narrow. The
`breakpoint` prop exists precisely so consumers declare their stacking
threshold; ignoring it fails the responsive acceptance.

**Right**
Honor `props.breakpoint`: at or above it render side-by-side; below it show
one pane at a time (list ↔ detail) with an explicit back affordance.
