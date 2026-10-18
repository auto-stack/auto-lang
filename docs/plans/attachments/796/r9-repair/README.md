# PLAN-796 R9 repair replay

2026-10-19. The R9 review probes were re-linked against the repaired
implementation (`experimental/ac-semantics/target/debug/deps`, probe rlib
`libac_sema_probe-88b107016c5fd572.rlib`, core
`libac_sema_core-b770c428b74e20ea.rlib`). Update the `--extern` paths if
the build hashes change.

## Method record return (F-796-R9-01)

`method-record-return.log.txt`:

- `before-current-publication=Ok(Miss)` — the v2 return type `B` no
  longer reuses the v1 page.
- `same-artifact=false` — v2 publishes a new artifact.
- `cached-current` (hot store) and `fresh-current` (independent store)
  both serve the same artifact with `get(x:A, y:B) -> record:B`.
- The fix is the FULL `Ann` canonical encoding: `record:A` vs `record:B`
  keep their names, and `unsupported:<what>` keeps its description (a
  tag-only code collided them). Body-only edits remain reusable.

## Long target name (F-796-R9-02)

`target-name-probes.log.txt`:

| name len | source bytes | live peak | meter peak | rejections | result |
|---:|---:|---:|---:|---:|---|
| 1,001 | 1,023 | 14,284 | 1,572,864 | 0 | PublishedMiss |
| 400,001 | 400,023 | 3,202,490 | 1,572,864 | 0 | PublishedMiss |
| 600,001 | 600,023 | 3,002,251 | 1,572,864 | 1 | ResourceExhausted (page ledger 604,428 > 524,288) |

The page estimate is now computed from the ACTUAL manifest that will be
encoded (config/source-map/fn-body origin = target name/fn-sig/negative
fields), so the final page is covered before the clone/encode; 600k is
a typed resource instead of a late codec error.
