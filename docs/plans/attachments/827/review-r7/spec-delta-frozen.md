# PLAN-827 r3 — SD-01..06 frozen normative proposal, R7

This is a review attachment, not published canonical knowledge. The six targets
and GOAL-002 impact remain as approved. Existing target hashes are bound in
baseline.json.txt. New targets are absent on main. The wording below describes
inspected behavior; a final pass remains withheld for F-827-R7-01/02. Resource
and persisted-observation claims must receive the closing evidence before merge.
No language, cap, ABI, default-engine or replacement-goal expansion is proposed.

## SD-01 — add docs/specs/auto-av/language-support.md

Status: experimental. Evidence has three separate levels: actual source
compilation/execution, typed HIR/MIR fixtures, and product entry support. A
fixture or a working legacy/native program does not establish AV source support.

The AV source consumer uses the existing `auto.source.scalar-i32.draft`
front end and `av.core-i32/1` executor. It reads unchanged AP-01
`p-gcd-euclid.at`, AP-22 `p-binop-args.at` and AP-23
`p-short-circuit-trap.at`; their byte identities and independent oracles are
frozen in PLAN-827. AP-01/AP-22 return typed i32 zero; AP-23's first false
short-circuit branch is safe and its later required division traps with
`av.trap.div-by-zero`. Native exit 70 is a separate reference channel.

G0 inventory enumerates AP-01..23, C-01..43, EP-01..10, gate rows, actual VM
case/source/oracle identities, stdlib provider/file members and entry symbols.
Missing, unreadable or unsupported members retain gap/owner rows. Inventory is
a denominator, not a claim that all members execute on AV. SEM-01..08 retain
profile distinctions and future owners; old expected files and AP-18's witness
are not automatically new language norms. List/text/record/enum/closure,
modules/stdlib/host IO, async/UI/FFI and product CLI replacement require their
own delivered contracts. GOAL-002 remains open beyond this limited contribution.

## SD-02 — add docs/specs/auto-av/source-program-conformance.md

The source-program suite is `crates/auto-lang/test/cases/av-source-827/` and
reuses `ac-av/core-run-v1`, the existing bounded worker and parent-side judge.
There is no new `av/source-run-v1`, `run-source` command or workspace. The
trusted host resolves bounded source bytes/root context once; admitted bytes
are frozen across later disk changes. Original fixture bytes are read-only;
private overlay `return 0` to `return 9` changes both source identity and result.

Execution is source -> Checked HIR -> shared MIR -> ABC decode/verify -> AV ->
typed actual. Old AVM/native does not execute the new path; native is only an
independent oracle. Source/worker/nonce/request/current authorization and
artifact identities are checked at their existing gates. Parse/admission/IO,
infra/resource/cancel and precise guest trap channels remain distinct. Invalid
admission never dispatches; a runtime failure preserves its legal trace prefix.

Existing caps and memory accounting remain unchanged: input 4MiB, applicable
table/nesting/verification limits, execution fuel/frame/wall/8MiB slot cap,
hot8/staging16/execution8 MiB partitions, frontend transient A <=224MiB and
worker residual B <=128MiB. Existing original-load structural and time gates
remain required; small-file latency cannot replace them. Frame-slot allocation
is fallible after budget checks. Its failure returns the existing SlotBytes
resource outcome; successful earlier buffers are owned by frames and released
on completion/error/cancel/drop. This does not promise recoverable OOM for
every process allocation. `FrameAllocationProbe` is an opt-in thread-scoped
validation API at the real root/callee slot reserve boundary, not guest input,
an authorization capability or a global allocator replacement.

Scope success requires every exact required member once, no required
skip/missing tool/zero selection, and child failure propagation. Acceptance
includes resources. Bench evidence must bind actual current source, dependency,
configuration, input and binary identities; historical evidence is reusable only
within its unchanged scope. Both successful and failed raw samples are retained.

## SD-03 — append docs/specs/auto-av/cache-and-conformance.md

PLAN-827 consumes the existing content-addressed source/cache contract. Fresh,
same-store hit and genuinely new-worker restore all execute AV and receive a
new parent comparison; no cached PASS/actual or serialized live frame is used.
Source content/position, dependencies, profile/tool/producer/entry/current
grants participate in their existing identities and consume gates. Grant
revocation rejects before dispatch; position edits bind diagnostic spans to
current coordinates. Valid identical content can be reused across project
labels; publishing roots and required-data leases still own GC reachability.
Wrong-owner/body identity, corrupted frames and self-consistent bad inner
payloads are rejected. Conflicting publication preserves the committed good
artifact. Execution-slot ownership and persistent store leases remain separate
resource axes; failure cleanup evidence must cover each claimed axis.

## SD-04 — append docs/specs/auto-av/project.md

PLAN-827 establishes a bounded experimental scalar source-file acceptance
baseline through the existing ac-av consumer. AP-01/AP-22 execute and AP-23
traps precisely; G0 inventory and semantic differences remain explicit. This
does not switch default CLI/UI/CTE, add general scripts/stdlib/host IO, or retire
the legacy VM. Source support and future capability gaps are recorded in
language-support.md and source-program-conformance.md.

## SD-05 — append docs/specs/auto-test-case/architecture.md

PLAN-827 adds source-program corpus and validation consumers of the delivered
`ac-av/core-run-v1` recipe. Source file bytes and virtual source identity are
resolved by the existing runner; the worker receives bytes, never expected
results. The parent owns typed actual comparison and source/worker identity
admission. The private overlay and same-store/new-worker paths retain actual
execution. Unsupported source, infra failures and guest traps keep their
existing distinct judgments. No recipe registration, protocol decoder or
second execution engine is added.

## SD-06 — append docs/specs/auto-av/resident-function-install.md

PLAN-827's source editing consumes the delivered D828 session. Pure body edits
with unchanged signature/dependencies/profile/configuration/grants retain
unchanged function HIR/MIR/code objects and generations; unchanged production
stages and artifact IO stay zero. Stable source baseline 273575B/115 functions
and at least 30 distinct revisions retain independent values, exact install
targets, structural identity observations and P95<1s. Correct timing alone
cannot excuse whole-frontend/reload degradation; actual forced object
replacement with equal content must fail the identity detector. Source
signature or unauthorized edits are recorded refusals preserving the current
revision; helper body changes leave interface-only caller identity unchanged.

Paused chains pin their original snapshot and own slot buffers across resume;
completion/cancel/drop releases those references. Prepare never publishes,
cancel/drop preserve current state, and close releases owned entries. New
frame-slot reserve failures use the same production dispatch seam and preserve
these ownership rules. Persisted raw observations must contain actual baseline
and current identities, remain attributable to the reviewed input/runtime, and
survive failed verdicts. Cold/whole-source/restart paths remain separately
reported; limited source syntax refusal is not a fallback to whole compilation.
