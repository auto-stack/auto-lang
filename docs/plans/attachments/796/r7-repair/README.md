# PLAN-796 R7 repair replay

2026-10-19. The R7 review's independent probe sources (`graph-probes.rs`,
`memory-probes.rs`) were re-linked against the repaired implementation
(`experimental/ac-semantics/target/debug/deps`, probe rlib
`libac_sema_probe-88b107016c5fd572.rlib`, core
`libac_sema_core-b770c428b74e20ea.rlib`). If the build hashes change,
update the `--extern` paths in the replay scripts.

## Graph closure (F-796-R7-01)

`graph-probes.log.txt` (replayed after the repair):

- `leaf-edit`: the current A artifact (`art5107…`) is NEW; its required
  edge resolves to the current B (`art4c74…`). GC with only the entry A
  rooted keeps A and B alive and both serve the current closure.
- `cycle-forms`: before GC, `lookup_only(A)` now returns the CURRENT
  `RecGroup` (`artc75e…`), not the historical singleton; after GC with
  the group rooted, A/B serve the same group page.

## Memory accounting (F-796-R7-02)

`memory-probes.log.txt` (allocator wrapper is review-observation only):

| N | real live increment | meter scratch peak | rejections | result |
|---:|---:|---:|---:|---|
| 10 | 15,535 | 1,572,864 | 0 | PublishedMiss |
| 10,000 | 6,579,145 | 7,794,932 | 0 | PublishedMiss |
| 20,000 | 526,860 | 1,572,864 | 1 | ResourceExhausted (wants 15,594,932 > 8,388,608) |
| 30,000 | 526,860 | 1,572,864 | 1 | ResourceExhausted (wants 23,394,932 > 8,388,608) |

The ledger now covers the real 10k live peak (7.79 MiB reservation vs
6.58 MiB actual) and rejects 20k/30k BEFORE solve/publish as a typed
resource state (no codec-error back door). The 1.5 MiB floor in the
rejected rows is the pre-read reservation (`READ_RESERVE`).

Permanent controls in-repo (bound by the acceptance script):
`sema_cache_r7_three_level_leaf_edit_republishes_current_closure`,
`sema_cache_r7_scc_formation_and_split_change_the_carrier`,
`sema_cache_r7_external_group_dependency_tracks_effective_version`,
`sema_budget_large_inference_rejects_before_publish`,
`sema_budget_long_names_are_accounted`,
`sema_budget_cache_readback_reserves_before_decode`.
