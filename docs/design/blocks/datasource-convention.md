# `dataSource` Wiring Convention (Plan 343, Design 17)

A block is **backend-agnostic**: it never bakes in an endpoint. Instead it
declares typed fetcher slots in its `spec.md` under `[dataSource]`, and the
consumer binds real `#[api]` functions to those slots at the call site.

## Declaring slots (block author)

In `spec.md` frontmatter:

```toml
[dataSource]
attempt = "(creds) -> Session"
list = "() -> []Note"
```

Each entry is `slot_name = "<signature>"` — the signature is prose for the AI
and the human; the slot name is what the consumer wires. The reference `.at`
illustrates the *contract* (which slots exist, when they're called, what states
they trigger) without depending on a concrete backend — data is passed via the
widget's `model` or props, mirroring `EditorPanel(note: str)` in 015-notes.

## Wiring (consumer)

When a block is dropped into an app (`auto block add` or agent generation), the
consumer connects each slot to a real `#[api]` function:

- the block emits a msg on the triggering event (e.g. `.Submit`);
- the app's handler calls the bound `#[api]` fn and feeds the result back into
  the block's model (loading / error / success).

`auto block add` prints the slots to wire:

```
# dataSource wiring (bind your #[api] fns to these slots)
  - attempt: (creds) -> Session
```

## Typing (forward pointer)

Today signatures are prose and front/back type agreement is checked manually.
A **typed** dataSource contract — where the compiler verifies the consumer's
bound `#[api]` fn matches the slot signature — is Design 16 **Rung 2** (typed
backend contract). Block `dataSource` slots are the front-end shape that Rung 2
will type-check.
