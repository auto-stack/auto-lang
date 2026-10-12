# Gotchas — feedback/empty-state

### Serving a generic "empty" for all three situations

**Wrong**
One message — "No data" — reused for first use, filtered-to-nothing, and
failed fetch.

**Why**
The three situations need opposite actions: create, clear filters, retry.
A generic state strands the user exactly when they need guidance most —
that's why the taxonomy is three variants, not one template.

**Right**
Pick the variant that matches the situation and let its copy + primary
action do the guiding. `first_use` invites creation; `no_result` offers
relief; `error` offers retry.

### Retry that retries nothing

**Wrong**
```auto
on .PrimaryClicked -> {
    // TODO wire this up
}
```

**Why**
An error state whose Retry is a no-op is worse than none: the user trusts
it, clicks, nothing happens. The `error` variant's contract is retry wired
to `dataSource.refresh`.

**Right**
Wire Retry -> `dataSource.refresh()`, disable the button while pending, and
let the enclosing view swap back to its data branch on success.

### Hiding the state inside a modal

**Wrong**
Surfacing empty/error states as a modal dialog over the region.

**Why**
Modals block context and stack badly (error over empty over data). Empty
states are in-place region states — they replace the region's body, not
interrupt the page.

**Right**
Render the state as the region's content, centered, with the action inline.
Reserve dialogs for confirmations and destructive guards.
