# av-data-824 corpus (PLAN-824 `av.dynamic/1`)

Identity receipts + hand-audit of the frozen first case executed through
the formal `ac-av/dynamic-run-v1` recipe (`avworker/1` envelope; the
worker never sees expected outcomes — the parent judges).

## first-case

- recipe: `ac-av/dynamic-run-v1`, entry `dynamic:first-case`
- profile: `av.dynamic/1` (id 5), wire `AVAD` v4, producer `plan824/av-dynamic`
- pipeline: programmatic typed fixture (UNTRUSTED) → real `verify_mir` →
  `encode_abc` → store (A23 code layer / A24 portable layer, separate
  domains) → `verify_abc_bytes` → AV execute
- frozen face (hand-audited 2026-10-27):
  - result `{"bits":"9","kind":"i32"}` — pack/is/take over
    Note{Str,i32}, List<Note>, Opt{none,some(i32)}: 7 (taken n) + 1 (list
    len) + 1 (some disc) = 9; any wrong axis short-circuits to 0
  - allocs bytes `[3, 0]`: "dyn" Str (3B) + the list buffer (0-elem at
    LIST_NEW — the ledger records the element count at alloc time)
  - frees ids `[1, 0]`: LIFO — the list (1 elem at release) then the Str
  - `live_after_release = 0` (the Dyn containers hold SHARES — no extra
    allocations, no leaks)
- negative oracles: `AV824_DYNAMIC_EXPECT_SABOTAGE=result|ledger` must
  redden `dynamic-result` / `dynamic-frees`; `AV798_FAULT=exit9|extra-frame|
  wrong-hash` must be `Status::Error` (infra, never a silent green).
