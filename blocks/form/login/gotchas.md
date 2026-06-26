# Gotchas — form/login

### Baking the endpoint into the block

**Wrong**
```auto
on .Submit -> {
    let r = http_post("/api/login", .email, .password)
    ...
}
```

**Why**
The block is now welded to one app's backend. It can't be reused, and the
endpoint/shape can't vary per consumer — defeating the whole point of a block.

**Right**
Declare `dataSource.attempt` in the spec and have the consumer wire their own
`#[api]` login fn at the call site. The block only owns the form + state
machine; it calls the injected fetcher.

### Skipping the loading / error contract

**Wrong**
Calling `attempt` synchronously with no `loading`/`error` state — the user
sees a frozen button and no feedback on failure.

**Why**
Loading and error are part of the block's *behavior contract* (spec
`acceptance`), not optional polish. Every data-touching block must render
them.

**Right**
Set `.loading = true` on submit; show it on the button; on failure set
`.error` and render it in the `error_display` EDIT region; clear `.loading`
in both success and failure branches.

### Forgetting label / input association

**Wrong**
A bare `input {}` with no `id` and no associated `label`.

**Why**
Breaks accessibility — screen readers can't announce the field, and clicking
the label won't focus it. Fails the spec's a11y acceptance.

**Right**
Give each `input` an `id` and a matching `label { for: "<id>" }`.
