# Gotchas — form/wizard

### Letting users reach a step whose prerequisites are unmet

**Wrong**
Rendering all steps' fields at once and enabling Finish unconditionally.

**Why**
The wizard is a state machine, not a long form: Finish must be reachable only
through the validated path. Otherwise submit arrives with half the steps
never completed, and the server error can't be mapped back to a step.

**Right**
Gate `Next` on the current step's validation; gate `Finish` on
`.step == total - 1`. Only the active step renders fields.

### Destroying entered data on Back

**Wrong**
Re-creating the step model when the index changes — fields come back empty.

**Why**
Users experience data loss as a broken wizard; "Back" must be a safe
navigation, not a reset. This is part of the spec's acceptance list.

**Right**
Keep one model for all steps' fields; the step index only selects which
`step_content` region renders. `Back` decrements the index, nothing else.

### Committing on Next instead of Finish

**Wrong**
```auto
on .Next -> {
    submit_step(.step)   // partial commit per step
}
```

**Why**
Per-step commits turn the blueprint into a chain of independent requests:
no atomicity, no clean cancel, and the dataSource contract (`submit(steps)`)
stops matching reality.

**Right**
`Next`/`Back` only move the state machine; the single commit happens in
`Finish` via `dataSource.submit`. If an app needs resumable partial commits,
declare it as a spec'd variant, not a local patch.
