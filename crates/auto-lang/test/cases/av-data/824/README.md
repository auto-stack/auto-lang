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

## T-11 追加：take-probe 固定面（D02 正式入口）

- recipe `ac-av/dynamic-run-v1`，entry `dynamic:take-probe`（worker 冻结表 DYNAMIC_CASES 第 4 行）。
- 冻结面：result `{"bits":"7","kind":"i32"}`；allocs `[{id:0,bytes:3}]`（"prb" 3B）；
  frees `[{id:0,bytes:3}]`；live=0。is(Alias 类错型)=false 永不 trap；take 复原 n=7 证明
  失败探针未消耗容器（错型 take 本身=take-mismatch trap fixture，证据在 probe 首例）。
- binding：`crates/auto-lang/test/cases/ace-m1/binding.tsv` D02 行 av 格已翻
  `ok:av-dyn-824`；映射与收据见 `docs/plans/attachments/824/852-validation/`。
