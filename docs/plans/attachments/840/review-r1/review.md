# PLAN-840 r2 final review R1 — needs_fix

Reviewed commit: `a1da1a00b7a239a872702c948cd4722014ee0797`, base
`a239653365a0c3451c85aa9921189fcf011bd8cf`, branch `plan-840-dev`, original
`D:/autostack/.wt/lang-840/auto-lang`. Implementation clean before/after checks.
SDK `932ad4877e5968f1aa7227d9dcd7fb69b4c69a21`, exact four-file scanner-only
artifact on old9576b4d; canonical/extracted scanners match. Both locks and AGP
unchanged. Review scope is final acceptance; same implementation-session context,
**no separate reviewer session/model isolation claimed**. Verdict is reconstructed
from code, actual counterexamples and raw reports, not the work summary. No agent
or chat was created. Implementation was not repaired during review.

## Findings (all contract gaps, not approved deferrals)

### F-840-R1-01 [P2] Callback Unit-return admission misses nested expressions

AC03/05, V03/05; T03/T05. `src/authoring/behavior.rs:563` `unit_body` checks
returns in selected statement bodies but never visits Store initializer expressions
(or general expression operands) for callback returns. Generic `expression/body`
checks later traverse them without the Unit requirement. A real callback can exit
with an int even when explicitly declared void:

```auto
fn app_start() void {
    let x=if true {return 1} else {return 2}
}
fn app_update(dt float){}
```

Expected: callback admission rejects the value-return before VM construction.
Observed: source admission OK, actual SDK compile OK, actual user Play tick0
accepts `Ok(())`. The corresponding app_update example is admitted and its actual
first dispatched tick is also accepted. Direct `app_start(){return 1}` is correctly
rejected, proving the old example does not close the invariant. Session's Unit
result path discards the returned NaN-box value (`src/lib.rs:190`), so it does not
rescue the missing AST check. This is not a claim about unsafe GPU output; it
violates the existing exact callback/Unit contract and masks invalid user code.

Closing oracle: reject value returns throughout expressions executing in either
callback, including initialization/assignment/call-argument/control-flow operands,
while respecting nested function/lambda lexical boundaries. Legal helper functions
may return values: the nested helper control executes successfully and must remain
legal. Preserve bare Unit returns and normal callbacks. Replay F01 controls and
callback/init/first-tick failure+owner tests; do not attach the full merge matrix
to this finding. No verified prescription to change the language SDK is implied.

### F-840-R1-02 [P2] Constant admission silently removes supported legal source

AC02/03/06, V02/03/07; T03. `src/authoring/behavior.rs:453` `constant` recognizes
literals and a few structural variants, but rejects every identifier and FStr.
Thus it rejects an immutable constant reference/pure arithmetic initializer and
strings that are entirely constant:

```auto
const BASE=1.0
var x=BASE+2.0
fn app_start(){}
fn app_update(dt float){}
```

Expected: finite pure constant initialization remains legal under Plan5.2/5.3;
user requested no support reduction. Observed: actual pinned SDK compiles and,
after real main initialization, `read_value()` returns `Float(3.0)`. Behavior
admission rejects it as requiring app_start. Similarly `var text=f"literal"`
and a backtick literal containing inert `use nonexistent` compile in the same SDK
but are refused by `constant`. The ordinary quoted literal control works. There
are no calls, mutable input reads, external dependencies or top-level loops in
these positive inputs, so moving them into app_start is not an authorized solution.

Closing oracle: admit finite immutable constant references and pure constant
formatted/backtick strings, without opening top-level executable calls, mutable
state reads, private access, unbound/cyclic or dynamic dependency evaluation.
Audit the constant-expression classification across supported AST forms rather
than adding text exceptions for these sources. Use actual user Play and read the
initialized result; retain current top-level-call and mutable-state-read negative
controls, literal/CRLF/size/diagnostic and inert-text controls. No new package or
multifile support is requested.

### F-840-R1-03 [P2] Proposed Spec IDs no longer match the approved delta

AC08, V10; T08. `research/behavior-spec-proposal.md:7` assigns SD01 to source/
snapshot admission and SD02 to the public API. The approved Plan's stable IDs are
SD01 **add behavior-api.md** and SD02 **modify scene-play-session.md**. The tree
proposal also replaces SD04's four named source/profile/project/integration target
updates with a validation/ownership summary, and does not bind those rows to their
actual target versions. The Plan table remains correct; the reviewed proposal
cannot be safely deposited using the same IDs with different meanings.

Closing oracle: reconcile proposal SD01..05 with the Plan's exact add/modify paths,
before/after rules, AC links and current canonical target hashes/versions. Keep
existing profiles, source landing and I2/I3 limits explicit; freeze the corrected
proposal for re-review. Canonical Specs/ledger must remain merge-owned. This is
proposal repair, not permission to change the implementation contract.

## Acceptance evidence map

| AC / tasks | Result at reviewed revision | Independent verification and boundary |
|---|---|---|
| AC01 T01/01a/01b/08 | pass | authoritative tree/commit/base; source verify299 retaining old278; SDK681/479packages/17locals/29edges; exact artifact4paths and original two locks; producer/consumer raw identity controls reused on identical inputs |
| AC02 T02/03 | partial | raw snapshot bounds/BOM/UTF8/CRLF/regions/version/base-scene/no-hot-reload checks pass; legal pure-constant input admission fails F02 |
| AC03 T03 | fail | actual Parser AST duplicate/reserved/import/signature and span controls pass; new nested-return and constant-source counterexamples fail F01/F02 |
| AC04 T04 | pass | actual Auto API code and9tests; handle0/absent/domain/read-after-write/group visibility/sticky error; actual motion and24positive pixel records from unchanged final native reports |
| AC05 T05 | partial | actual lifecycle/cancellation/budget/Stop/common consumer/600tick and corrupt-output evidence valid on legal input; callback Unit requirement fails F01; affected report/CLI partial tests replayed1test/10process scenarios pass |
| AC06 T06/07 | partial | ordinary UI/CLI/source root/SaveAs/busy/focus/no-fallback and same-runner sample/relocation witnesses valid; general legal source promise incomplete F02 |
| AC07 T04/08 | pass | 128-node actual user run;20sourceA/B VM/GPU cycles; actual retained VM and fresh upload negatives plus restored control; original external-pin/ownership cases retained; no arbitrary heap quota claim |
| AC08 T07/08 | partial | API surface/docs/13native cases and independent consumer valid; proposed Spec stable-ID/target mismatch F03 must close |
| AC09 T09 | not passed | all declared risk areas inspected; three concrete in-contract findings remain |
| AC10 T10 | pending merge | no landing, complete unified matrix, canonical publication, archive or cleanup performed |

Verification run now:38 canonical scanner unit tests;9behavior+3behavior_host+
5SDK/feasibility tests;1probe_exit test covering10process scenarios; read-only
source verification; fmt; scoped current-library build; external real SDK/Play
counterexamples. Existing tests all pass; the independent10-case contract replay
exits1 with six failing expectations (three nested-return, three constant-input)
and four passing controls. Its failing exit is the finding evidence, not an
infrastructure failure. Source/SDK/meta identities and implementation cleanliness
were rechecked after all probes. SDK146 inherited warnings, no new driver warning.
Full all-target work check and unaffected old6Play/7Host/4completion evidence are
reused with identical code/dependencies/config/input; they are not newly rerun here.

All13 final native raw process/report records were audited: one executableSHA,
4positives at27phases/6pixel-records/6checks each,9specific expected failures.
The separate consumer has its own681-fileSDK/target/build,18scoped tests and2native
positives/transparent-pixel negative with the documented32MiBPE stack. The final
identity update changed only the feasibility restoration witness and receipt;
production/API/dependency bytes were asserted unchanged, final source controls
rerun. Baseline hashes and the native audit are in baseline.json.txt. No aggregate
banner is used as a substitute for individual assertions or source path inspection.

## Replay and probe classification

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/840/review-r1/replay.ps1 -Output <fresh-absolute-directory>
```

The wrapper builds only the scoped library, links a separate probe to actual
compiled driver/SDK inputs and uses32MiBPE stack. Current code must exit1; after
correct repair all normative cases should pass. `replay-production-entry` contains the
validated command/output and machine-readable assertion map. The external harness reads real regular .at files through SourceSnapshot::read and
lives in main evidence and writes only ignored build artifacts in the Plan tree;
no source/test files in the implementation were added or modified.

Exploratory raw `${`/`${"` backtick probes were invalid *according to the actual
Parser*: this Lexer dispatches backticks through fstr, despite an unused raw_str
helper's comment. They do not establish a scanner defect and are excluded from
the normative oracle. Empty-tuple Unit spelling and inert enum field names are
also exploratory only, not additional product requirements/findings. First direct
rustc linking lacked the Windows import-lib search path; corrected native path
resolved it. Windows PowerShell5 native-warning handling/UTF16 stdout in the replay
wrapper were corrected and the final wrapper was itself executed, with exit1 from
contract assertions. All initial diagnostics remain available.

## Routing and remaining gates

Return to work in the same840tree; reopen T03/T05/T08 and dependent acceptance
evidence, retain T01/01a/01b/02/04/06/07's completed bounded results. Current state
executing7/12, Plan revision remains2: no semantic contract change was approved.
Fix and commit the actual mechanisms/proposal, refresh source identity, record
finding closure and replay only affected cases plus legitimate controls. Re-review
must account for changed shared AST traversal, not merely the original strings.

All complete merge-owned gates remain pending and were not run: full locked
driver/AGP tests/builds, original16AGP cases, original games/authoring/asset/hierarchy/
play native and owner/counter/**actual content** oracles, full new13-case user
native/source/consumer matrix on the reviewed freeze. Bare root cargo t and any
applicable surface tiers are post-pass merge-owned, tf main-only batch. No root
whole-tier t/tv/tt/tb/tu/taa/tf or docs_gen ran in review. No merge/canonical Spec/
ledger publication/archival/worktree removal. Re-record reviewed state only after
all in-contract findings close and final acceptance/proposed delta pass.
