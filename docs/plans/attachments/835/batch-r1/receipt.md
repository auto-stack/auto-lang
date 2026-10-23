# PLAN-835 postlanding batch — known-reds-only

Main-only, single instance, covered 6991731ab3e6fb7f64865a2962b9cbd2e2a3bab5. Complete tf6282/6282; tt full retry5702/5703 with1 existing chart helper red; tb full retry5307/5307. Structural skipped cases are explicit. No new regression identified, no code/assertion/configuration edit or forced test exemption. All raw initial failures and retries retained.

Initial tt3 worker timing assertions failed around29–30s; each passed the same assertion isolated (0.46–0.86s) and ALL3 passed in complete parallel tt retry (1.22/1.40/2.00s). No permanent fix is claimed. Cold preparation/load is a hypothesis; full-scope replay distinguishes these from the remaining reproducible chart red. Initial tb chart failure also passes isolated and in full tb retry.

Remaining tt chart `plan484_024_charts_streaming_recompute` prints handler_LineChart_Init/CpuRunnable and misses t29. The test directly calls `drive_scheduler_to_quiescence`; its current progressed predicate excludes cpu.slices_run and silent_done_transitions. This is exactly pre-existing committed D-762-1 (known-debt-baseline.txt), not an environmental waiver based only on a passing isolated test. The earlier762 diagnosis already records this multi-slice helper exposure and requires a separately reviewed decision before modifying the production helper. This batch does NOT repair it or weaken the assertion. The known red remains visible in the receipt, not greenwashed. No new fix-forward Plan is needed for an already registered debt with its existing owner.

The code window since prior covered6336ab95 contains only a shim doc-comment marker change (root-code-window.diff); no runtime/compiler/test/config/input delta. AutoGraphics835 code lives exclusively on autographics-dev. No AAVM trigger, so no taa/t3. Shared main documentation WIP preserved, no code WIP.

{
  "tf": {
    "seconds": 113.536,
    "run": 6282,
    "passed": 6282,
    "failed": 0,
    "skipped": 518,
    "exit": 0,
    "evidence": "tf.txt"
  },
  "tt": {
    "seconds": 106.981,
    "run": 5703,
    "passed": 5702,
    "failed": 1,
    "skipped": 1800,
    "exit": 100,
    "evidence": "tt-retry.txt"
  },
  "tb": {
    "seconds": 75.264,
    "run": 5307,
    "passed": 5307,
    "failed": 0,
    "skipped": 1548,
    "exit": 0,
    "evidence": "tb-retry.txt"
  }
}
