# Gotchas — form/signup

### Baking the endpoint into the blueprint

**Wrong**
```auto
on .Submit -> {
    let r = http_post("/api/register", .email, .password)
    ...
}
```

**Why**
The blueprint is now welded to one app's backend. It can't be reused, and the
endpoint/shape can't vary per consumer — defeating the whole point of a
blueprint.

**Right**
Declare `dataSource.register` in the spec and have the consumer wire their own
`#[api]` signup fn at the call site. The blueprint only owns the form + state
machine; it calls the injected fetcher.

### Skipping the confirm-password / terms gating

**Wrong**
Submitting whatever is in the fields — mismatched passwords and unchecked
terms go straight to `register` and bounce with a server error.

**Why**
Client-side gating is part of the blueprint's behavior contract. Round-tripping
to the server for violations the form already knows about burns the user's
time and pollutes the error region with noise.

**Right**
Check `.password == .confirm` and `.terms_accepted` before setting the button
into its pending state; only then call `dataSource.register`.

### Dropping the invite token from the contract

**Wrong**
Reading the invite code from a global inside the widget, or silently ignoring
it.

**Why**
Props are the blueprint's data interface. A global read hides the dependency
(two instances with different tokens can't coexist) and an ignored token
breaks invite flows that looked wired.

**Right**
Declare `invite_token` in spec `props`, seed the model from it, and pass it
through to `dataSource.register` so the consumer's fetcher decides what to do
with it.
