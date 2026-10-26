# F841-R5-01 — accepted compound props retain dangling VM references

Priority: P2; kind: incomplete shared-invariant repair; affected: AC-03/04/05/06, T-07/T-08/T-09/T-10. Reviewed UI 2f311868f082bc4ab6fac9d484f71f18d13b1c95, lang 88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80, Man 9b4302fa585e6cda4a91738a6e9057fae12c9402. Revision/config/probe hashes: [baseline](baseline.json.txt).

## Existing rule and actual input

The reviewed repair claims the queued property snapshot owns its VM heap references and safely transfers them to root fields before releasing the queue owner. This is the same lifetime obligation as M841-GATE-04; accepted compound prop expressions already exist, not a new generic container feature. `AuraViewBuilder::resolve_expr_to_value` recursively evaluates Expr::Array (aura_view_builder.rs:14110) and Expr::Object (:14124); prepare_child_render_state_snap -> ensure_child_state -> register_init_demand consumes those real values. The new visitor (vm_bridge.rs:1194–1204) handles only top-level VmRef and direct array elements and silently skips nested arrays/objects. ListData<Value>::child_refs also only sees direct VmRef elements, so merely retaining more queue leaves would not by itself prove correct root/container ownership after replay.

Both counterexamples use the actual production parser/lowering/constructor and view builder:

```auto
widget App {
    model { source List = [7] }
    view { Child(data: [[.source]]) {} }
}
widget Child(data: List) {
    model { done bool = false }
    on { .Init -> { .done = true } }
    view { text "child" {} }
}
```

The second input replaces only `[[.source]]` with `[{inner: .source}]`; the accepted control uses `[.source]`. All three parse, lower and construct successfully. A real view build puts Child in Queued. The probe reads the actual seeded data and checks its nested VmRef is the exact source heap id, then uses the public write_state to replace the parent source with Nil and forces the existing VM reclamation hook. This is a single normal owner transition, no new scale/depth budget.

## Expected / observed

Expected: pending Init/seeded property still owns source; weak heap handle remains live and vmref_to_vec(source) still returns [7].

Observed [final three-case log](compound-producer-controls.log.txt): direct array control passes; nested-array and object-inside-array controls both fail the liveness assertion because the original heap object has been freed while its VmRef remains in the accepted property data. The producer-shape/id and Child Queued assertions all passed first, so this is neither a parse-invalid probe nor a made-up queue value. Initial two-case replay is retained separately, not double-counted.

Full runnable probe: [review841_nested.rs](review841_nested.rs); [byte-exact source](review841_nested.rs.base64.txt). Copy it to the existing UI worktree's crates/auto-ui-runtime/tests/review841_nested.rs, set child env AUTO_GALLERY_PROXY_PORT=54480 and RUSTC_WRAPPER empty, and run `cargo test -p auto-ui-runtime --test review841_nested -- --nocapture`. Exit101, one pass/two fail. Remove the temporary probe afterwards. No implementation repair was made in this review.

## Cause, coverage and closing oracle

Confirmed cause: the real producer recurses into supported compound values, while new snapshot retain/release and root seeding account only for direct typed refs. Array-backed root storage also relies on the existing direct-child heap protocol. Current code leaves a borrowed nested reference with no owner once the parent's field is replaced. Already-closed direct-ref/cancellation/parked tests remain valid; they did not exercise compound producers.

Correction must cover accepted producer shapes and both queue and replayed-root lifetimes, including symmetric release. Do not fix by deleting these inputs, declaring accepted nested syntax unsupported, extending GC grace, copying away observable alias identity, or permanently retaining whole snapshots. Any proposed representation/child-edge adjustment must be checked against the existing scalar/reference distinction and container reclamation protocol; this report does not assert a one-line recursive visitor alone is sufficient.

Close with the exact two failures plus direct control; demonstrate nested values remain readable after parent replacement, actual Init dispatch/replay, and another forced collection; then reclaim after last owning field/queue owner leaves. Cover representative cancel/missing/drop and shared aliases, preserving the current successful direct-ref, scalar-id and graph failure/recovery controls. Reassess supported Array/Object value forms from the actual producer before declaring readiness. The ordinary candidate Spec must describe the verified full path; the new 'direct VmRef element' wording cannot silently narrow earlier accepted prop expressions. Unified/global gates stay merge-owned pending until the in-contract gap is fixed and affected review passes.
