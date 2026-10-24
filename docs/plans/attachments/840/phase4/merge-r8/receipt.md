# PLAN-840 r8 completed consolidation

**Delivered, archived26/26 and cleaned.** All original support retained; repairs added as Phase2/3/4 in the same840, no new Plan.

| Stage | Frozen evidence / result |
|---|---|
| Original graphics | reviewed1f1939525; autographics-dev8b62db5cba54b9bfe7f759f4920b22ccb9b75e88; original63 complete invocations/native+independent consumer support retained |
| SDK source artifact | codex/plan-840-sdk932ad4877e5968f1aa7227d9dcd7fb69b4c69a21 retained |
| Final repair review | ba6479da53b7999e490dd0e8ac8d701d5ffe86b2; r7/r8 affected final PASS, same-session independence limitation recorded in ../review.md |
| Work controls | FFI23/23 per full/t3;34 affected controls;200 repeated key controls;4 frozen review replay controls; type/format/lock checks pass |
| Frozen complete merge gates | bare t5345/5345; tv163/163; tt5717/5717; tb5316/5316 |
| Delivery | docs-only descendant53298cd273e0c04fabc508745ce342655394f1a4 fast-forwarded to master; code/config/input/six locks unchanged; source and ledger hashes read back |
| Main-only complete batch | tf6291/6291; tt5717/5717; tb5316/5316, zero failures/timeouts, original selections plus3 new tests; named mutex/source/dependency guards pass |
| Batch receipt | docs/plans/.last-batch-regression.json835→840 after all three complete legs and exact preexisting report restoration |
| Canonical / projection | SDK state-lock, auto-lang FFI binding and stdlib generator-registry rules plus navigation; prior Init/RQHost and original graphics/SDK rules retained; guarded offline ledger80→81, six targets and1183 unrelated raw items preserved |
| Archive | docs/plans/archive/840-autographics-public-behavior-api-and-user-scripts.md; archive/evidence commitc0705306371d4c2c399f9040577be17aa52d3fcf precedes cleanup |
| Cleanup | lang-840/auto-lang and auto-down separately fresh guard clean then Git worktree remove; merged plan-840-dev deleted; receipts in ../cleanup.json.txt; non-Git consumer/native repro assets preserved |

All old failures, the invalid initial POSIX-unlink fixture, deterministic red controls and nested mutation are retained.47 raw text logs are whitespace-normalized for repository hygiene; exact original bytes are reconstructible and hash-verified via ../raw-byte-preservation.json.txt (zlib+base64). JSON backups remain byte exact. No ignored test, weaker threshold, reduced input, removed assertion or narrowed support closes a failure. D-762-1 production UI helper remains separate existing debt, not claimed fixed.

The old batch file-lock output omitted errno; the real Windows error5 path is directly demonstrated and fixed, and complete-pool original counter assertions pass. Windows ambiguous ACCESS_DENIED now retries within the existing deadline; genuine hard errors and eventual timeout retain best-effort downgrade, not a mutual-exclusion guarantee after downgrade. Other preexisting compiler warnings are recorded outside changed executable lines; no new warnings or temporary diagnostics introduced.

Final bookkeeping closes both remaining merge tasks only after actual guarded cleanup. Foreign main-checkout WIP was preserved through surgical staging of the owned Known-Debt append. Original SDK source-only pin and graphics delivery were not repurposed or removed.
