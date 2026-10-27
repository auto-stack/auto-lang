# PLAN-852 r2 — T-07 final re-review R2

**Outcome: needs_fix.** This is the independent review context that issued R1, not the implementation context. No agents were created. No implementation was modified.

## Frozen baseline and scope

- HEAD `17a959ba097dd7fd247171f7ce43568c75b937ba`, branch `codex/plan-852-dev`, existing tree `D:/autostack/.wt/lang-852/auto-lang`.
- Prior reviewed HEAD `4b3c9b254487f81322670fb62cc6b7953718cadb`; substantive repair `a19e5597f`; other merged changes inventoried separately. Original merge-base `6c02d13628f7c6fcd31555226621ebc2c1fea9b7`.
- Dependencies unchanged: auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`, auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. Tracked implementation clean before and after probes; existing untracked probe-t01 retained.
- Scoped checks: receipt rejection/reuse, actual compiler identity, explicit-only certification, replacement stream reader ordering/overflow/wall, formatting and durable SD. Source/oracle/case inputs and existing decoder/recipe contracts are unchanged; reuse R1 baseline evidence for those surfaces. A new 97-check snapshot run also passed under the documented Windows PowerShell 5.1.

## Finding closure and reproducible remaining defects

### F-852-R1-01 — partially repaired; remains open (AC-03/04/06, T-03/04)

Old malformed schema and string-flag mutation now reject; unchanged valid receipt reuses. Those original example closures are confirmed in replay-output.txt. Two required shared invariants remain broken:

**R2-A [P1], actual compiler identity is mislabeled.** certify() chooses its executable from runtime `$RUSTC` (or current PATH), but uses BUILD_TOOLCHAIN_IDENTITY whenever the CLI --rustc option is absent. It does not bind that stored identity to the compiler actually launched. This violates §5.2/G05/G07 and the frozen certification toolchain requirement; it is not a new platform support promise.

Reproducer in replay.ps1: on a COPY of the committed corpus, set RUSTC to `rustup which --toolchain nightly rustc`, then `ace-certify --corpus <copy> --only b01 --force`. Actual installed compiler prints `rustc 1.90.0-nightly (bfc046a4b 2025-07-12)`; certification succeeds and writes toolchain.identity=`rustc 1.98.0 (88d9e12ae 2026-08-18)`. A subsequent daily snapshot under Windows PowerShell 5.1 accepts this mislabeled certificate: 26 a2r checks, 10 STRICTLY valid certificates, 97 full-suite checks PASS (`snapshot-nightly-ps51.txt`). Legitimate control: pass the same nightly compiler using explicit --rustc; the certificate then correctly records 1.90 nightly (`explicit-nightly-control.txt`).

Cause established: runtime compiler selection and build-time identity have separate sources. Closing oracle: fresh certification must bind the exact selected executable/profile and actual std/dependency context; CLI, RUSTC and PATH selection must obey one identity rule. Daily validation may consume frozen identity without spawning compiler processes, but must not turn unrelated build-tool identity into certification evidence. Replay this two-toolchain case and unchanged valid reuse; retain same-output generator reuse. No full regression required.

**R2-B [P2], strict receipt parser still drops invalid data.** validate_receipt converts flags and deps.external with filter_map(as_str), so `flags=["--edition=2021", null, "-O"]` and `external=[null,17]` both return reused/exit 0. Unknown top-level fields are also ignored (`unknown_contract="must reject"` returns reused). Reproducer and output: replay.ps1 / replay-output.txt. Baseline valid receipt reuses; existing invalid schema/string flags reject. Existing rule: contract-decision §5 expressly requires unknown fields to reject, and receipt arrays must have the declared types (not silently filtered). Rust and PowerShell validators also diverge: snapshot counts all tokens while Rust filters them.

Closing oracle: strict typed versioned schema with unknown-field and wrong-type rejection at every nested structure; no filtering/defaulting missing/invalid fields into valid empty sets. Test independent null/number/object/missing/extra cases in flags, deps and nested receipt objects through both consumers, plus valid control; broken receipts must not be overwritten even with --force.

### F-852-R1-02 — implicit-certification repair confirmed (AC-04, T-03/04/06)

acceptance no longer invokes certify; forced certification and resource probes are #[ignore] explicit operations, outside normal daily test execution. Snapshot no longer builds its own binary. Replacement process observer is a periodic CIM ancestry sample, not a lossless process-creation event count; zero samples cannot prove zero short-lived spawns, and all descendant cargo processes are classified maintenance without checking their command. Its reported zero must not be used as stronger evidence than this boundary. Code inspection confirms no remaining direct certification dispatch in the daily path; retain that closure while repairing R2-A/B. No demand to run daily tiers during work/re-review.

### F-852-R1-03 — stream fix confirmed, cleanup exit still incomplete (AC-07, T-04/05)

The exact stderr-first 128KiB reference now certifies; direct stdout/stderr readers are independent and per-stream overflow is propagated. Independently ran only `cargo test -p auto-lang-core --features test-cases --lib test_cases::ace_certify::tests:: -- --ignored --nocapture`: all 3 selected stream-order/overflow/wall tests PASS (`governance-tests.txt`, 30.23s). This closes the original pipe deadlock and cap propagation defects. Existing warning output is from unchanged code; do not represent it as zero repository warnings.

**R2-C [P2], identity failure skips scratch cleanup.** certify() creates scratch_root and then calls capture_toolchain_contained(path)? BEFORE entering the result/cleanup block. On missing --rustc it returns directly and leaves the directory behind, contrary to the all-exit cleanup claim. Exact replay: `ace-certify --corpus <committed corpus> --only b01 --rustc X:/missing/rustc.exe --scratch <new temp path>` returns exit 2 and scratch_still_exists=True (`missing-tool-cleanup.txt`). Only disposable new scratch paths were used. Closing oracle: cleanup guard spans identity capture and every subsequent exit; replay missing tool and failed identity plus successful certification, verifying only owned scratch is removed. No stream-suite rerun needed unless the stream mechanism changes again.

### F-852-R1-04 — mostly repaired, formatting remains (AC-08, T-06)

contract-decision.md is now tracked; wrapper-main-v1.rs and ten receipts are tracked; SD-03/04/05 have concrete proposed text and empty touched_goals rationale. Formatting of the original three added Rust files is fixed. Expanded check of all changed Rust files finds unformatted new code at `crates/auto-lang-core/src/test_cases/stale.rs:79` (`format-check.txt`, exit 1). Reformat that new insertion, then check the repair's complete changed Rust set. Changed implementation diff whitespace checks pass. Historical R1 raw fmt logs contain intentional diff whitespace; do not rewrite raw evidence to make repository-wide git diff --check appear clean. Unrelated Plan 822 evidence whitespace is outside 852.

## Oracle, denominator and Spec audit

Independently inspected all 58 Auto sources and all 43 handwritten references, not just counts. Per-ID frozen value results agree with the independent reference algorithms:

- B01..B12: 14,-21,42,42,42,0,30,7,120,42,7,-2.
- T01..T05: 38,27,23,128512,42 (Unicode scalar/byte distinction and NUL preserved).
- R01..R08: 19,9,23,51,42,21,42,42 (copy/clone/move/mut call/parallel-assignment intent preserved).
- L01..L05/L07: 42,42,0,42,42,42.
- V01..V07: 42,42,3,42,42,42,42.
- K01/K02/K03/K05/K06: 42 each; owned/borrowed/copyable closure forms remain explicit capability gaps where not source-supported.

9 diagnostics inspected: B16 unknown name in dead branch, B17 type mismatch, B18 immutable assignment, B19 missing return, T06 borrowed text escape, T07 surrogate scalar, R09 nominal-type mismatch, R10 use after move, K04 borrowed closure escape. 6 traps inspected: B13 i32 overflow, B14 zero division, B15 MIN%-1, B20 evaluated RHS division, T08 scalar slice bounds, L06 collection bounds. Explicit legacy/front-end differences remain in binding; K04's observed grammar rejection is labeled as such, not proof of full closure semantics. The old-i64 differences are not silently normalized. Wrapper i32 encoding is confined to supported scalar set; wide typed rows are not passed off as final source execution.

63 corpus and 40 scenario required rows are preserved; no selector shrinking or native executable restart masquerading as online HMR. Source/reference inputs unchanged from R1, so original runtime evidence and this inspection can be retained independently of failed receipt validity.

Frozen drafts copied beside this report; SHA256 in spec-hashes.txt:

- SD-01 ace-m1-conformance.md `93a75d42fe0ab8dff7c6544883b7ddb389121a9cbd09271e77ff2a97fa95a8bc`.
- SD-02 a2r-golden-certification.md `a70f81ca89659eced7df0143348af4ebf457594c96fb8f7853cdb36fc4f0e901`.
- contract-decision.md `c0f6fe811c57adfe28fddd7d481440c88aa0865a6414d5af5df49496d186b75ea`.

SD-01/04/05 denominator/navigation/unchanged decoder directions acceptable; SD-04's phrase “64源参考” needs a count clarification (actual 58 primary source / 43 reference / 5 typed source). SD-02/03 cannot pass assertions of complete strict identity and all-exit cleanup until R2-A/B/C close. Process-count prose must describe sampling accurately or use a verified lossless control. No canonical Spec or ledger was published during review.

## Acceptance map / next

| AC | R2 result | Evidence boundary |
|---|---|---|
| AC-01 | partial | source identity and 97-check baseline retained/passed; valid certification axis blocked R2-A/B |
| AC-02 | pass (review) | all 43 value references/9 diagnostics/6 traps inspected against fixed oracle and actual supported bindings; gaps preserved |
| AC-03 | fail | actual execution demonstrated, but compiler identity mislabeled R2-A |
| AC-04 | partial | implicit certification removed; daily stale/validity compromised by R2-A/B; sampling limitation stated |
| AC-05 | pass | separate baseline/M1 conclusions and full required denominator retained |
| AC-06 | fail | old decoder/registry unchanged, new receipt still accepts invalid types/unknown fields |
| AC-07 | partial | lifecycle source unchanged, 97 checks pass, stream/wall 3 tests pass; cleanup R2-C open |
| AC-08 | partial | frozen revision/evidence/SD review present; no final pass until remaining repairs |
| AC-09 | pending merge | bare cargo t and actual applicable surface gates after final review passes on frozen commit; tf batch-only main |

Probe correction: initial snapshot invocations used PowerShell 7 and hit its changed negative -split semantics plus nonexistent redirection-directory behavior. Those raw records remain, but are **not product findings**: the documented `powershell -File` Windows PowerShell 5.1 path passes, including a newly nonexistent OutDir (`new-outdir-ps51.txt`). The nightly-mislabeled certificate PASS above was reproduced with PS5.1. Do not fix expectations to accommodate the invalid initial harness choice.

No whole-tier cargo t/tv/tt/tb/tu/taa/tf or docs_gen was run. Next: work repairs R2-A/B/C + changed-file formatting in same tree, commits and hands off targeted invariant/negative controls. This is the second failed final review: the next handoff must include diagnosis/coverage adjustment for the identity and strict-schema shared mechanisms, rather than only the original four examples. Preserve confirmed stream and implicit-certification closures; do not replay unrelated global/performance families.
