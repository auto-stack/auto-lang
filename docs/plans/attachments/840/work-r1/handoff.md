# PLAN-840 r1 work handoff: needs_replan

Current status: **executing 1/10**, T-01 complete; T-02 incomplete;
T-03..10 have not started. No API delivery, execution_done, independent final
review or merge pass is claimed. No canonical Spec/ledger changes were made.

User authorized PLAN-840 work on 2026-10-24 using the auto-musk repository's
auto-plan-work skill. That skill and PLAN-840 §5.1 require returning to new if
the bounded two-candidate feasibility cannot satisfy the key contract. This is
that explicit stop condition; not a request to reapprove unchanged work.

| Handoff field | Identity |
|---|---|
| Repository / tree | auto-lang; D:/autostack/.wt/lang-840/auto-lang |
| Branch | plan-840-dev |
| Base | a239653365a0c3451c85aa9921189fcf011bd8cf, formal839 delivered autographics-dev |
| Code | `4fb2fda5a` (full SHA in receipt.json.txt) |
| Dependencies | SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9; AGP unchanged from formal839 |
| Driver lock | 2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db |
| AGP lock | c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf |
| Next role | new: bounded contract/SDK prerequisite revision; no T-03 implementation yet |

The worktree is retained and clean. Only two files are committed: a scratch
feasibility integration test and its detailed research report. Both are within
the authorized driver write surface. Implementation/SDK/AGP/root source and
locks are unchanged. The main checkout's unrelated document changes were
preserved; implementation never ran there. No agents/chats were created.

## T-01 and current evidence

839 is archived10/10, its final-state receipt, actual branch SHA, review/complete
gate, knowledge/archive/guard removal/not-due receipt were read and reconciled.
Main code WIP was empty. Tree/branch identity was asserted before the first edit.
`prepare_sdk.py` verifies680 files; `verify_source.py` verifies source278 and
metadata479 packages /17 local /29 path edges. `verify_plan839.py --controls`
passed LF valid control, source-drift / missing own GLB / missing original GLB /
lock-drift rejection and restoration. Those source-set results bind the untouched
base before the two new probe files, not a new PLAN-840 source-delivery closure.

## T-02 actual results and remaining invariant

| Finding / invariant | Affected paths and durable case | Current result / remaining gap |
|---|---|---|
| B-840-T02-01: admitted inert text cannot become imports; intact single-file snapshot must not trigger untracked module resolution | Source candidate: `sdk_preprocessing_must_not_treat_block_comment_as_dependency` and `sdk_preprocessing_must_not_treat_string_as_dependency`; actual Parser asserts no Use/Dep, legitimate empty callbacks compile, then actual Session::new | Both fail at SDK preprocessing `Module not found: plan840_missing_dependency`. No user admission/API implementation is asserted. |
| Same invariant, second permitted candidate | `public_module_path_must_not_treat_inert_text_as_dependency`; normal controlled module loads, fresh CompileSession loads the explicit snapshot containing an AST-admitted block comment | Fails on the same nonexistent nested dependency. Public module service shares the defective scan before Parser; switching candidates cannot close it. |
| Distinguishing accepted mechanism control | `intact_user_callbacks_globals_group_read_after_write_gpu_stop_same_binary` | PASS: two user source variants in one unchanged executable,120 total Auto ticks, nonzero initializer/parent, group write→child-world read→other-leaf write, independent position and all-mesh GPU pixels, exact Stop image and Editor identity restoration, runtime weak-owner released. This is scratch profile6, not formal UserV1/profile7/native CLI acceptance. |

The first scratch attempt incorrectly reran app_start on ticks; the independent
2D position check rejected it. The trusted bridge was corrected and the initial
failure preserved. No user-byte rewriting or assertion relaxation was used.

Final command:

```text
cargo test --locked --test user_behavior_feasibility -- --nocapture --test-threads=1
```

Exit101, **4 selected /1 passed /3 failed /0 ignored /0 filtered**. These are
expected *red-phase contract evidence*, not passing implementation tests.
See [frozen log](feasibility-frozen.log.txt). Final formatted code was used.
`cargo check --locked --all-targets` exit0; [log](check.log.txt). SDK146 inherited
warnings, driver zero new warnings. Single-file rustfmt --check and git diff
--check exit0. Final pinned SDK verify-only passes after commit.

No root t/tv/tt/tb/tu/taa/tf/docs_gen, complete driver/AGP suite, or full native
acceptance campaign was run. They cannot cure this source semantic defect and
remain future merge-owned gates after complete work and independent review.

## Source diagnosis and precise unblock action

Fixed SDK `create_vm_from_source` calls dependency scanners before Parser.
`scan_use_statements` trims and scans individual lines without tracking block
comments/string state. Fresh and cached module loading both call resolve_uses
before module parsing. This diagnosis is grounded in the pinned680-file SDK,
not current master source. The module probe uses ordinary ignored target files
and fresh sessions, no links/cwd changes/ambient external dependency fixtures.

PLAN-840 forbids SDK changes, altered user literals/logic, untracked dependencies,
and a duplicate compiler/runtime pipeline. Silently rejecting these legal inert
texts reduces the agreed supported surface. A driver that strips user text or
copies a new AST→codegen→linker pipeline is not either approved existing entry.

**Recommended bounded next-stage proposal:** authorize a prerequisite SDK fix
for lexically correct dependency discovery, preserving real import behavior and
covering block comments, multiline strings and fresh/cache module paths. After
that prerequisite is delivered, revise840's SDK pin, lock/closure/source identity
and replay these exact three red cases plus the existing accepted control before
continuing T-02. Alternative: explicitly agree a public dependency-free in-memory
SDK compilation seam with source/diagnostic identity, without copying a compiler
pipeline into this driver. Neither scope change is implemented here.

T-02's signature/diagnostic/API-error/budget and formal source-change/owner proof
still require completion; the scratch success does not close them. AC-01's base
portion is evidenced, but full AC-01 and AC-02..08 remain incomplete. T-09/10 and
AC-09/10 are review/merge-owned and pending. Proposed SD-01..04 are unchanged.

Detailed mechanism, code-path reasoning, boundary and reproduction:
[tree report](D:/autostack/.wt/lang-840/auto-lang/experimental/autographics-auto-driver/research/user-behavior-feasibility.md).
