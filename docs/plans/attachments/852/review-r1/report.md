# PLAN-852 r2 — T-07 independent final review R1

outcome: **needs_fix**. Reviewer: current Codex review context, independent of the work handoff in this conversation; no subagent or fabricated model/session identity. Implementation was not edited.

Baseline: `4b3c9b254487f81322670fb62cc6b7953718cadb`, branch `codex/plan-852-dev`, worktree `D:/autostack/.wt/lang-852/auto-lang`. Implementation/main merge-base `6c02d13628f7c6fcd31555226621ebc2c1fea9b7`; original work base `2e09547ee`. Tracked implementation clean before/after probes; pre-existing untracked `probe-t01/` preserved. Dependencies: auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`, auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`.

## Findings and closing oracles

### F-852-R1-01 [P1] Invalid certificates are accepted and identity is incomplete

AC-03/04/06, T-03/04. Existing rule: §5.2, G05/G07 and contract-decision §3/5 bind target, flags, dependency closure and actual results, and reject invalid receipts. `ace_certify.rs:207` only compares five content hashes, wrapper hash and short rustc version plus verdict. It ignores schema, actual hash, flags, deps, observed results and case/module consistency. Short `rustc --version` does not identify host target or std closure. Snapshot validation at `verify-ace-m1-852.ps1:230` checks four file/self-reported text hashes; it does not validate the receipt's schema/verdict/wrapper/target/dependencies or bind oracle text back to the fixed registry.

Reproduced on a copy of the committed corpus: change B01 certificate schema to `invalid/999`, toolchain.flags to `["--wrong-flag"]`, deps.closure to `changed-dependency`, observed.actual_run.stdout to `i32:999`; preserve other axes. `ace-certify --corpus <copy> --only b01` returns `reused`, exit 0 (`tampered-cert.txt`). Original corpus control also returns reused (`valid-cert-control.txt`). Snapshot against an isolated RepoRoot containing the copied corpus and unmodified a2r adapter, with AUTO_CASE_BIN pointing to the frozen worktree executable, returns PASS/10 certificates/97 green checks (`snapshot-invalid-cert.txt`).

Required closure: strict versioned receipt validation, complete frozen-context identity and an explicit non-spawning daily validity check. Invalid/changed context must be non-green/STALE rather than accepted or silently re-certified. Replay malformed/schema/flags/deps/observed/oracle/wrapper/target cases independently plus unchanged valid control; same-output generator changes remain reusable. No full regression needed for repair.

### F-852-R1-02 [P1] Acceptance and daily tests implicitly perform certification

AC-04, T-03/04/06. Plan §8 explicitly says acceptance only verifies existing evidence and never implicitly certifies; G08 requires independent explicit certification. Script line 349 calls `certify` from `acceptance`, which invokes the mutating certification entry and compiles whenever a certificate is absent/stale. Furthermore `ace_m1_certify_fresh_and_tamper` uses `force: true` for all ten cases and runs certification twice. It is registered under test-cases, selected by normal cargo t/tt (config line 73/93; nextest default-filter has no exclusion). Thus normal daily runs repeatedly compile actual/reference Rust instead of only snapshotting certified goldens.

Closing oracle: acceptance with missing/stale certificate must report the gap without certificate writes or rustc/target compilation. Normal daily test selection must not run the forced-certification workflow. Keep explicit certification tests separately invocable; instrument actual process creation rather than treating a grep of one adapter as process-count evidence. No whole-tier test execution during repair/re-review.

### F-852-R1-03 [P2] Sequential blocking pipe reads deadlock valid bounded output

AC-07, T-04/05. `ace_certify.rs:394` and `:474` read stdout and stderr sequentially in one reader. A child filling stderr while stdout remains open deadlocks the reader and child until the wall, even below 1MB cap.

Reproducer: in copied corpus replace reference/B01.rs with `use std::io::Write; fn main() { std::io::stderr().write_all(&vec![b'x'; 131072]).unwrap(); println!("i32:14"); }`, then `ace-certify --corpus <copy> --only b01 --force`. Actual wrapper succeeds; reference falsely fails at 30s (`dual-pipe-control.txt`). Direct compilation/run of this exact reference exits 0 and prints `i32:14` (`dual-pipe-direct.txt`; redirected stderr contains 128KiB plus PowerShell line encoding). This is a harness input testing stream governance, not a new Auto semantic requirement.

Also the current cap paths merely break/truncate and discard stderr; no explicit overflow failure propagates to the parent. Compilation stdout is not counted at all. `rustc_version` is an uncontained, unbounded `Command::output`, outside the claimed all-tool governance.

Closing oracle: independent concurrent draining, parent-visible per-stream overflow with prompt tree kill, bounded/contained identity invocation. Test stdout-only, stderr-first, interleaved bounded streams, each overflow and hanging descendant; retain real hanging-run wall control. Do not alter production semantics to fit this probe.

### F-852-R1-04 [P2] Health and durable handoff are unfinished

AC-08, T-06. rustfmt --check --edition 2021 on the three added Rust files exits 1 (`fmt.txt`). git diff --check also reports trailing whitespace at verifier:394 and five case-file trailing blank lines. `contract-decision.md` is linked as the frozen contract but exists only on main and is not tracked in this worktree (`git ls-files` empty); it must be made a durable versioned handoff artifact. This includes reconciling its explicit -OutDir scratch requirement with the certifier's current fixed temp_dir/PID location and no OutDir argument. No automatic semantic waiver.

Closing oracle: formatting/diff checks clean, frozen decision resolvable from a fresh committed checkout, and actual scratch policy consistent with the approved contract. Review SD-03/04/05 against exact existing Spec text, add concrete enduring delta text and explain empty touched_goals before final pass.

## Acceptance/evidence map

| AC | Result | Evidence / remaining boundary |
|---|---|---|
| AC-01 | partial | 58 formal sources + 5 typed registry entries retained, same-source suite has 97 PASS on independent snapshot run including B01/B02/B04/B09; valid four-path certification claim blocked by F01 |
| AC-02 | partial | inspected fixed oracle table, source/reference and diagnostic/trap routing, 97 base checks green; full 43-reference/9-diagnostic/6-trap independent semantic signoff remains pending, not inferred from file counts |
| AC-03 | fail | valid B01 reuse control passes, invalid receipt also reuses; actual generation and actual==golden before compile confirmed by source inspection |
| AC-04 | fail | F01/F02; output remains honestly separated into snapshot vs behavior, but identity and zero-implicit-certification contract unmet |
| AC-05 | pass (inspection) | 63 corpus + 40 scenario rows retained; acceptance explicitly refuses M1-complete claim and reports separate axes; no claim that pending language/native capabilities are 852 defects |
| AC-06 | fail | no edits to existing v1/v2 decoder or recipe registry; new receipt protocol lacks strict schema/context validation (F01); old full compatibility suite not rerun |
| AC-07 | fail | lifecycle cases' 120/720 checks passed within 97-check run, session steps/cache diagnostics inspected; native loader gaps retained; stream governance counterexample F03 |
| AC-08 | partial | this revision-bound independent receipt, findings/debt and frozen SD copies provided; final pass blocked |
| AC-09 | pending merge | bare cargo t + required actual surface tiers only after final acceptance review on frozen repaired commit; cargo tf batch-only main |

## Spec delta audit and omission scan

Frozen draft copies beside this report: ace-m1-conformance.md SHA256 `ed75d33ca766c314b52a1f913906e9834586e9ff4d496cda5c4ca24efdbb3037`; a2r-golden-certification.md SHA256 `bd0cb4aeba09623935e067a4f00473052a43515a062a7f0a54018cc683dcd89b`. Existing canonical input hashes and main-only decision hash: spec-input-hashes.txt.

SD-01 direction and two-axis/native-HMR boundaries align with r2; SD-02/03 cannot pass while describing toolchain/flags/full-file certification and all-tool containment more strongly than implementation supports. The draft explicitly defers toolchain checking until certify, contrary to frozen G05 daily stale behavior; this is an unapproved weakening, not an accepted delta. SD-04/05 exact target paths exist and decoder preservation is plausible from diff, but concrete deposits and empty-goal rationale remain incomplete. No canonical Specs or ledger published by review.

Unimplemented collection/closure/Dyn/native-online capabilities stay legitimate r2 M1 gaps with their recorded owners; they do not cause this needs_fix. In-contract certification/identity/governance gaps cannot be waived as downstream debt. The aggregate work PASS banner and 75/252 count are not independent proof. Preserve unaffected baseline source/lifecycle evidence when repairs do not touch those inputs; finish pending oracle/compatibility audit on re-review and retest affected certification mechanisms. No cargo t/tv/tt/tb/tu/taa/tf or docs_gen was run in this review.

Next: work in the same tree, repair F01–F04 with scoped controls, commit, hand off affected findings and remaining oracle/SD evidence for T-07 re-review. No merge/archive/cleanup.
