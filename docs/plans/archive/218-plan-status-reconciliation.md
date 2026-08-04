# Plan 218: Plan 191-216 Status Reconciliation

## Status

✅ Phase 1 + Phase 2 (key tasks) complete. 2026-04-23.

## Goal

Reconcile the implementation status of Plans 191-216 with actual code evidence. Many plans were implemented in rapid iteration without updating their plan files or were implemented in sibling repositories (../book/, ../auto-vscode/). This plan tracks the work to:
1. Verify each plan's true implementation status against code
2. Complete any remaining unfinished work
3. Update each plan file's status markers to reflect reality

## Verification Results (2026-04-23)

### Phase 1: Status Reconciliation — Update plan file status markers

Update each plan file's status to match code reality. Commit after each plan update batch.

| # | Plan | Plan File Says | Code Reality | Action |
|---|------|---------------|-------------|--------|
| 1 | 191 | No markers | ✅ Fully implemented | Mark complete |
| 2 | 192 | No markers | ✅ Fully implemented | Mark complete |
| 3 | 193 | DRAFT | ✅ Fully implemented | Mark complete, remove DRAFT |
| 4 | 194 | No markers | ✅ Fully implemented | Mark complete |
| 5 | 195 | Not started | 🔧 Partial (http_stream exists, no auto.http unification) | Mark partial |
| 6 | 196 | Not started | ❌ Not implemented | No change needed |
| 7 | 197 | ✅ Complete | ✅ Verified | Confirm |
| 8 | 198 | No markers | ❌ Not implemented | No change needed |
| 9 | 199 | Not started | ❌ Not implemented (Plan 199 IS the debugger, Plan 196 renamed) | No change needed |
| 10 | 200 | Partial | ✅ Now fully complete (Task 3.3 + 3.4 done) | Mark complete |
| 11 | 201 | ✅ Complete | ✅ Verified | Confirm |
| 12 | 202 | Not started | 🔧 Partial (axum+Vue crate exists) | Mark partial |
| 13 | 203 | Not started | 🔧 Phase 1 (QualifiedName exists) | Mark partial |
| 14 | 204 | Not started | 🔧 Partial (Result/Spec in rust.rs) | Mark partial |
| 15 | 205 | Not started | 🔧 Phase 1 (VmBridge exists) | Mark partial |
| 16 | 206 | ✅ Complete | ✅ Verified | Confirm |
| 17 | 207 | ✅ Complete | ✅ Verified | Confirm |
| 18 | 208 | ✅ Complete | ✅ Verified | Confirm |
| 19 | 209 | Not started | ✅ Phase 0 done (33/33 PASS), Phase 1-6 现代化待实施 | Mark Phase 0 complete |
| 20 | 210 | Not started | ✅ Implemented in ../book/ | Mark complete + note external |
| 21 | 211 | Not started | ✅ Fully implemented (51 VM + 17 a2r tests, all pass) | Mark complete |
| 22 | 212a | Not started | ✅ Implemented in ../auto-vscode/ | Mark complete + note external |
| 23 | 212b | Not started | 🔧 Tasks 1-3 done, Task 4 (runtime bridge) pending | Mark partial |
| 24 | 213 | Not started | 🔧 95 inline tests, needs maturation | Mark partial |
| 25 | 214 | Placeholder | ❌ Blocked on 212b | No change needed |
| 26 | 215 | Not started | 🔧 85 tests, needs maturation | Mark partial |
| 27 | 216 | ✅ Complete | ✅ Verified | Confirm |

### Phase 2: Complete Remaining Unfinished Work

Pick off incomplete plans one by one. Each sub-task is a separate commit.

| # | Plan | Remaining Work | Priority |
|---|------|---------------|----------|
| 1 | 200 | .map_err() closure callback + fs module aliases | ~~P1~~ ✅ Done |
| 2 | 195 | Create auto.http module, unify http_stream, add RequestBuilder | P2 |
| 3 | 196 | SOURCE_LINE opcode + CallFrame + disassembler (5 phases) | P2 |
| 4 | 209 | ~~Phase 0~~ ✅ / Phase 1-6 现代化重写（低优先级美化） | ~~P2~~ Phase 0 done |
| 5 | 211 | Add ~43 stdlib tests to reach 80%+ coverage | ~~P2~~ ✅ Done (51 VM + 17 a2r) |
| 6 | 212b | Rust FFI E2E (Task 4 runtime bridge + Task 5 E2E test) | P2 |
| 7 | 213 | a2py maturation to 80+ tests | P3 |
| 214 | Python FFI (use.py) | P3 (blocked on 212b) |
| 8 | 215 | a2ts maturation to 80+ tests | P3 |
| 9 | 198 | Native metadata from source | P3 |

## Commit Strategy

- Phase 1: Batch updates by status group (one commit for "mark complete", one for "mark partial")
- Phase 2: One commit per completed sub-task, referencing the plan number

## Success Criteria

- [ ] All 27 plan files have accurate status markers reflecting code reality
- [ ] All "verified complete" plans have explicit completion notes with commit references
- [ ] All "partial" plans document which phases are done and which remain
- [ ] Plan 200 remaining tasks completed
