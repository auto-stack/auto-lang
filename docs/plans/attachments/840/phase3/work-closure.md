# PLAN-840 r5 Phase3 work closure

Frozen49261f09e747e09a1194726e8a219e38cd1de834 atopf181fc47a; same lang-840 tree/downc39. Existing tb red5190pass/2fail/119cancelled is retained, not reported as pass.

The real Init/CpuRunnable producer may yield before completion; synchronous geometry consumers must continue driving rather than read an incomplete view. Added one test-gated shared method in plan370_test_support, iterating real render/demand + one scheduler round with a30s deadline. No production helper/VM/config/input changes. Fifteen calls across484/492/498/499/536 use it. Existing437 early-exit negative and geometry positive remain untouched; two new cases directly verify shared driver multi-slice geometry and pending deadline failure.

Scoped48/48 pass, no fixture SKIPPED branch. All existing caller logic/oracles verified identical after reversing only method replacement and canonical rustfmt. Original production dynamic.rs byte-identical. Existing musk non-Init Fetch8-round driver audited and retained. Source formatting/whitespace clean. SD-P3-01 records the actual test-only completion requirement; D-762-1 remains open for separately authorized production helper changes.

Phase2 feature/timing fixes retained unchanged; reuse r4 evidence for them with source/config/lock identity. New fixture edits invalidate full daily/tt/tb gates; those are explicitly merge-owned pending after final affected review. No work-owned finding remains.
