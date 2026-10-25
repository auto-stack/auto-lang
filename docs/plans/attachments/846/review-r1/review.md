# PLAN-846 final review R1 — needs_fix

Reviewed Plan revision 2, code `655c366a3a4c53160c259a0b91a50c02d1cb020a`, base `c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f`, branch `plan-846-dev`, authoritative tree `D:/autostack/.wt/lang-846/auto-lang`. Scope: final. Reviewer: Codex /root in the implementation conversation. No separate reviewer context/model was created; independence is limited accordingly. This verdict was reconstructed from the approved contract, entire diff, canonical Specs, raw reports and fresh distinguishing checks, rather than executor summaries. No implementation edits, merge, publication or full-tier regression occurred.

Baseline, dependency and canonical/proposed Spec hashes: [baseline](baseline.json.txt). SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467` differs from original `932ad487...` only in nano_value.rs and has the identical root fix blob; tag retention and auto-down c39ac7f9 verified. Three locks unchanged. All ten current canonical Spec targets match the diff base (the new target is absent on canonical); frozen proposed delta is [patch](spec-delta.patch.txt). SD01..05 match actual geometry/API/ownership/nonfinite semantics and approved boundary; GOAL-007/008 exist in specs/goals.md. No semantic revision is needed for this finding.

## Finding F-846-R1-01 — P2, existing CLI requirement gap

Affected AC-03 (CLI admission subitem), AC-06, AC-08; T-03, T-06 and dependent T-08 readiness. Plan §5.5 requires an orphan API option to report a usage error; README additionally promises a lone/unknown API option is a usage error. The committed parser maps both absent `--behavior-api` and present-but-no-following-value to None, silently omitting validation. Located at `experimental/autographics-auto-driver/src/bin/autographics-authoring.rs:407` (`match value("--behavior-api")`, `None => None`).

Reproducible command from driver cwd:

```powershell
$exe = './target/debug/autographics-authoring.exe'
$scene = './samples/spatial_projects/zone2d/scene.agscene'
& $exe --inspect $scene --behavior-api
```

Expected: exit 2, specific missing-value/usage diagnostic before inspecting the project. Observed: exit 0, `closure_verified:true` inspect result, no missing-value diagnostic. Adding `v2` while still omitting `--behavior` correctly gives exit2 and `--behavior-api requires temporary --behavior`; omitting the option entirely gives exit0 and valid inspect output. All three real invocations were repeated on the documented 32MB native profile; [commands/results](cli-profile-results.json.txt), stdout/stderr alongside. Binary SHA `50763a59badb8c923c3ebe443590de10230b35ebf6489194a00a8f1278a9f39b` binds that repeat. Thus the failure reaches the actual production CLI parser, independent of scene/source compile behavior. `--inspect` is used only to make the existing early-usage invariant observable without opening an unbounded interactive window; the orphan-with-value control confirms this same parse stage.

Established cause: the local `value` helper uses `.and_then(|i| args.get(i + 1))`; the API match's None arm cannot distinguish missing option from missing value. No unverified runtime cause is asserted. Without inspect, the same omission also bypasses the intended argument check on the normal startup path. No NaN/geometry or project-binding semantic change is requested.

Correction/closing oracle: distinguish presence from value absence and reject malformed supplied API options with exit2. Add a permanent CLI-focused regression with missing terminal value, orphan v1/v2, unknown value, legitimate absent option and valid v1/v2 temporary-source controls; preserve project-default selection. Replay this minimal finding set and relevant CLI/profile admission tests on a committed fix, refresh affected source identity/consumer export and review only affected evidence. No full driver/root/native campaign is required for the repair handoff; unified gates stay merge-owned.

## Acceptance, tasks and evidence

| AC | Tasks | Result | Evidence and coverage |
|---|---|---|---|
| AC-01 | T01/T08 | pass | Clean authoritative tree/branch/commit, SDK single-path artifact and root blob equivalence, source344/SDK681/17 packages/479 closure/29 edges verified freshly; unchanged lock/AGP/Builtin/source boundary inspected. `source-verification.txt`, baseline. |
| AC-02 | T04 | pass | Actual Host model producer and all-mesh interaction bounds, static8corners, immutable resource identities/caching inspected. Fresh spatial_geometry2 and native three-dimensional actual-content pass; independent scalar reference and second-mesh omission control inspected. |
| AC-03 | T02/T03 | partial | Explicit typed v1/v2, unchanged v1 API/bridge/reserved set, forged source/base rejection and actual helper controls pass. CLI malformed-value subitem fails F-846-R1-01; no broader version fallback found. |
| AC-04 | T04/T05 | pass | Fresh spatial_api4, feasibility3, Host3: closed/ULP/hidden/self/dimensions, all-mesh, immediate group/leaf writes, identity/finite errors. Extra review_holdout2 adds native ln NaN in all3 coordinates, second-pair operand errors, pair count single charge and latest RHS world. SDK exact module17 passes, producer→slot→pop_f32 inspected; original old-encoding mutation red retained. |
| AC-05 | T02/T05 | pass | Fresh actual128wide/deep budget;32/33 quota, callback Fault/last-good/VM closure/Stop/newRun; startup configuration live counter tests missing/duplicate/order before app_start plus valid exactly-once control pass. Fixed original CPU/time budgets inspected; no changes. |
| AC-06 | T03/T06 | partial | Same-raw API history/persistence fresh1; v1 compatibility unchanged evidence identified, new3D native OS/input/private enter-stay-exit/Step/Stop passes. Original full12cases+2narrow reopened raw reports inspected; CLI gap remains. |
| AC-07 | T04/T08 | pass | Fresh20 real lifecycle cycles pass with actual VM/source/descriptor/asset/GPU weak owners, external pins, scans/uploads/targets. Five work mutation reds and restored control assertions inspected; production paths have no mutation hooks. |
| AC-08 | T07/T08 | partial | Actual native profile zone3d14phases; transparent expected failure at phase5 pixel oracle; partial expected failure phase0 and nonzero all rerun. CornerGPU test fresh1, narrow/update/Save/reopened identities and original work14invocations inspected; source controls rerun on independent consumer all4 mutations rejected/restored pass. README/SDK/pin/limits/Spec/merge matrix inspected, malformed CLI promise remains unmet. |
| AC-09 | T09 | fail | Final needs_fix due existing work criterion above. No pass or merge authorization. |
| AC-10 | T10 | pending | Merge-only gate/landing/deposit/archive/cleanup not attempted. |

Fresh commands (driver cwd unless noted):

```powershell
$env:RUSTC_WRAPPER=''; $env:RUST_MIN_STACK='16777216'
cargo test --locked --offline --test spatial_api --test spatial_host --test spatial_project --test spatial_geometry --test spatial_samples --test spatial_feasibility --test spatial_owners -- --test-threads=1
cargo test --locked --offline --lib spatial_admission_tests -- --test-threads=1
cargo test --locked --offline --bin autographics-authoring spatial_completion_tests -- --test-threads=1
python tools/verify_source.py
rustc --edition=2021 --test .sdk-source/crates/auto-val/src/nano_value.rs -o target/review846-sdk-tests.exe
./target/review846-sdk-tests.exe
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/check_spatial_queries.py <review-output>/native-profile-selected --cases zone3d,transparent,partial
# In independent consumer846-r2 driver, not the authoritative implementation:
python tools/verify_plan846.py --controls
```

Scoped integration results are15 tests plus2 targeted lib/bin tests=17, all passed, without running the standalone whole suite. Extra independent holdout2 passed using committed driver/AGP rlibs, frozen included spatial_api.rs setup and holdout.rs. SDK17 additional direct pure-module tests passed. Raw test logs and holdout source are durable here. Holdout rustc uses `-L dependency=target/debug/deps`, exact driver/AGP rlibs and native search paths from existing build outputs, PE32MB; worker stack16MB. Its initial manual link omitted Windows native-lib search (setup failure), then default test-thread stack lacked16MB (setup failure); both retained, corrected before product assertions, not reported defects. Initial native replay observed real cursor-left and required auto-Pause at phase3; this environmental event is retained, not treated as runtime failure. Correct documented native profile rebuilt before the successful3case repeat. Binary identity changes from test/build profile are recorded, never treated as the original work binary SHA.

Evidence reuse: executor's 14 native invocations were manually counted as12 result rows plus two narrow `reopened.json` reports with passed/phase14, not14 rows. Current fresh zone3d trace matches work-final user/API/bridge/combined/geometry identities. Historical `native-binary-source-r2.json.txt` predates final source details and is not used as a current Rust source receipt; current clean Git HEAD, final source manifest and fresh native repeat bind review. Work owner mutation tests alter only test-owned retention/rescan/upload paths; fresh actual owner positive confirms current paths. Prior v1 selected module12/project4/storage5 and legacy native27phase evidence is reused only for unchanged v1 source/API/fixtures and inspected shared compatibility paths. The independent consumer selected NaN and actual multi-mesh/GPU/Stop work logs remain scoped evidence; full175 consumer is not claimed run. No whole `cargo t/tv/tt/tb/tu/taa/tf`, full AGP, full standalone or full native campaign ran in review.

## Spec, omissions, health and route

No dropped feature/support reduction, gameplay hardcoding, altered tolerance/CPU budgets, extra VM cache, per-tick rescan or unsupported SDK expansion found. Query booleans use exact closed comparisons; unavailable metadata is distinct from legal finite degeneracy. Source/VM/descriptor/history/resource ownership paths and startup failure exits inspected. All ten proposed Spec paths and SD01..05 are consistent with approved scope; canonical inputs have not drifted. Deferred production I2/I3/Schema/ACE and precise collision are approved non-goals. The sole current in-contract CLI gap is not downgraded to optional debt. Existing SDK146 warnings remain baseline; no new driver compiler warning or stray debug print found. Current formatted/code-diff checks are reused on unchanged committed source; raw logs retain their trailing blank lines.

Outcome: needs_fix; Plan returns executing6/11. Reopen T03/T06/T08 (T08 closure depended on all scoped acceptance, including CLI); leave T01/T02-SDK/T02/T04/T05/T07 and their valid evidence complete. T09 final review and T10 merge remain incomplete. Next: work in the same tree for F-846-R1-01, commit, then affected re-review. This is first failed final review, so the two-failure diagnosis checkpoint is not triggered.

Pending after eventual final pass: frozen root bare cargo t+tv; complete producer/fresh own-SDK/own-target expected175 driver, original163 AGP+16native and all83 inherited invocations plus new spatial14 each side/owners/source controls; isolated SDK root landing versus graphics-only autographics-dev; canonical/ledger/archival/guard and actual batch-due closeout. Exact existing handoff matrix remains binding; review does not waive any denominator or claim delivery.
