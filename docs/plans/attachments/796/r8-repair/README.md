# PLAN-796 R8 repair replay

2026-10-19. The R8 review probes were re-linked against the repaired
implementation (`experimental/ac-semantics/target/debug/deps`, probe rlib
`libac_sema_probe-88b107016c5fd572.rlib`, core
`libac_sema_core-b770c428b74e20ea.rlib`). Update the `--extern` paths if
the build hashes change.

## Long-name memory (F-796-R8-01)

`memory-combinations.log.txt` (allocator wrapper is review-observation
only):

| tag | source bytes | real live peak | meter peak | rejections | result |
|---|---:|---:|---:|---:|---|
| short (10k) | 167,806 | 6,579,144 | 7,892,712 | 0 | PublishedMiss |
| long1k (1000×1000-char) | 1,014,806 | 2,346,637 | 4,800,712 | 1 | ResourceExhausted (page ledger 1,027,998 > 524,288) |
| long2k | 2,031,806 | 2,066,956 | 1,572,864 | 1 | ResourceExhausted (scratch 9,588,712 > 8,388,608) |
| long3k | 3,048,806 | 4,131,340 | 1,572,864 | 1 | ResourceExhausted (scratch 14,376,712 > 8,388,608) |

The rejected rows never reach the >8MiB live window: the page ledger
rejects an unencodable page before clone/encode, and the solve/publish
reservations reject the long-name multiplicity before allocation. No
codec error remains on these paths. The 1.5MiB floor in rejected rows
is the pre-read reservation.

## Method signatures (F-796-R8-02)

`method-probes.log.txt`:

- `before-current-publication=Ok(Miss)` — the v2 method signature no
  longer reuses the v1 page.
- `same-artifact=false` — v2 publishes a new artifact.
- `cached-current` (hot store) and `fresh-current` (independent new
  store) both serve `art9ee92…` with `get(x: Bool) -> Bool` — the
  hot-tree result now equals the fresh-store oracle.

Permanent controls (bound by the acceptance script):
`sema_cache_r8_method_signature_is_a_freshness_input`,
`sema_budget_long_names_reject_before_encode`,
`sema_budget_oversized_page_rejected_as_resource_before_encode`,
`sema_budget_long_name_readback_is_charged`.
