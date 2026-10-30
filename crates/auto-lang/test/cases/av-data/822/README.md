# av-data-822 identity receipts (PLAN-822 corpus)

## Suite

- `av-data-822.suite.ad` — schema `autolang.suite/2`, includes the cases
  below. Registered through the shared `test_cases` dispatch
  (`ac-av/collections-run-v1`, PLAN-822 append-only registration).

## Cases

- `av-collections-first-case.case.ad` (`av.collections.first`) — the
  frozen `collections:first-case` fixture through the formal recipe. No
  `.at` source exists: the collections profile has NO source surface by
  contract (non-goal); the input is the worker's programmatic typed
  fixture (builder output is UNTRUSTED — the real `verify_mir` →
  `encode_abc` → `verify_abc_bytes` gates admit it).

## Hand-audit receipts (frozen oracle, independent of the worker)

first-case pipeline (see
`experimental/ace/crates/av/src/collections_fixture.rs`):

1. `CONST Str "first"` — arena allocation id0, 5 bytes.
2. `AGGREGATE Note { text, n: 1 }` — record owns the id0 share.
3. `LIST_NEW List<Note> cap 2` — arena list buffer id1 (0 elements).
4. `LIST_PUSH` — note transfers in (slot killed, ownership moved).
5. `LIST_FIELD_SET list[0].text = "replaced"` — id2 (8 bytes) allocates;
   the OLD id0 share frees at the in-place field replace (5B free first).
6. `LIST_REMOVE list[0]` — the record moves out (owned transfer; the
   tail-shift billing is empty for the last element).
7. `VARIANT_PACK Outcome::Ok(note)` — payload transfers in.
8. `VARIANT_GUARD disc 0` — guard fact recorded (true at runtime).
9. `VARIANT_PROJECT disc 0` — owned payload moves out (arm → moved).
10. `DROP got` — id2 frees (8B).
11. `DROP outcome` — payload-moved arm: no double free.
12. Frame exit / host release — the list buffer id1 frees last.

Frozen face: `result = {"kind":"i32","bits":"0"}`,
`allocs bytes = [5, 0, 8]` (creation order ids 0,1,2),
`frees ids = [0, 2, 1]`, `live_after_release = 0`.

The PARENT-side table `COLLECTIONS_CASES` (ac_av.rs) carries the same
values as an independent copy; the `AV822_COLLECTIONS_EXPECT_SABOTAGE`
seam mutates the parent's copy (result/ledger) as the V-10
counter-control.
