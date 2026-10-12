# Verification contracts for high-risk Plans

Read when a Plan changes compiler/IR semantics, trusted admission/validation,
resource lifetimes, publication transactions, or a contract used by multiple
consumers. Use the applicable parts, not a universal exhaustive checklist.
Ordinary low-impact changes use their scoped checks.

## Put decisions and oracles in the contract

Keep the repository's numbered Plan sections and five lifecycle states.
In sections 5–8, distinguish the current contract from linked historical
receipts. Specify behavior and externally checkable results; leave routine
implementation choices to work. Unresolved architecture belongs in a bounded
investigation task, not invented detailed pseudocode.

For each risky rule, explain what it protects and choose representative cases
that would fail under a plausible wrong implementation. Section 6 can use:

| Case ID | AC / invariant | Input or setup | Entry point | Observable expected result | Assertion / evidence | Coverage boundary |
|---|---|---|---|---|---|---|
| V-01 | AC-01, mapped argument types | Different parameter types, reordered evaluation with valid bindings | Actual binder/verifier and native runner | Valid mapping accepted; native result matches; side effects retain evaluation order | Result, trace, diagnostic stage | Declared profile; not every future type |
| V-02 | AC-01, mapped argument types | Same input with invalid bindings | Actual verifier | Controlled rejection before backend | Nonzero plus specific diagnostic; no unchecked consumer | Mapping/type gate |
| V-03 | AC-02, failed publication preserves old output | Existing valid artifact set; representative failure during publication | Public build/publish entry | Old bytes preserved if not committed; cleanup or accurate residual/recovery report | Hashes and old executable result, owned residual paths | Specified artifact set and failure stages |

These are illustrative cases, not mandatory IDs or universal feature requests.
Select inputs from the real syntax/API. A proposed interface can name future
fixtures and the task that will make them runnable; do not report them as
executed before implementation. Existing supported inputs should use actual
code or source evidence.

For each applicable risk, cover representative axes rather than every Cartesian
combination:

- Identity/type: both ownership directions; heterogeneous mappings; values
  crossing local, parameter, return, and branch boundaries when supported.
- Graph structure: tiny cycles, disconnected components, valid nesting; ensure
  invalid structure is rejected before an unsafe traversal.
- Resource/publication: preparation, execution, output collection, publication,
  rollback and cleanup failure exits; distinguish timeout from successful reap.
- Trusted data: shape, semantic validity, freshness, uniqueness and applicability;
  check downstream admission if promised, not only the final CLI exit code.
- Classification: the declared binding/owner forms and ambiguity policy;
  legitimate controls must remain accepted.
- Workflow/validator: partial and complete states, active and archived paths,
  required references and exact target identity; mutations occur and a failing
  executed control makes the overall gate fail.

Explicitly state exclusions and pending decisions. A later review must identify
whether a finding violates this contract, clarifies it, or proposes new scope.
Do not retroactively treat every new case as an original explicit requirement.

## Separate task ownership and review roles

Use `owner_stage` in the Markdown task/AC tables or handoff. These are workflow
instructions; they do not claim new backend schema or parser support.

| Task | owner_stage | Outcome and linked AC | Evidence |
|---|---|---|---|
| T-01 | work | Implementation and regression cases | Current committed result |
| T-02 | review | Independent acceptance decision | Final reviewed commit and cases |
| T-03 | merge | Actual archive references/counts/cleanup | Actual-state checks and receipts |

All unique tasks count toward total_steps. current_step counts actually completed
tasks; a pending review or cleanup is not marked done to make handoff look
complete. work enters execution_done when its owned tasks are complete, final
review enters reviewed when its criteria pass, merge finishes the remaining
explicit closeout gates before delivered pass.
Merge-owned criteria concern closeout, not deferring implementation correctness.
At the archive checkpoint, cleanup may still be pending and counts must say so;
only after actual cleanup are those tasks completed and final counts verified.

For an explicitly split workflow, record roles and routing in section 6:

```text
Implementation: assigned implementation agent.
Internal check: assigned checker; pass -> execution_done, final review next.
Final review: assigned independent reviewer; pass -> reviewed, merge next.
Closeout: merger; actual final-state check precedes cleanup.
```

Preserve user-assigned model choices; do not create or contact another agent
without authority. With no split roles, the normal independent review remains
final. An implementation-session check states its limitation and cannot stand
in for an explicitly assigned separate final reviewer.

## Keep evidence verifiable and the contract readable

Record case IDs, commands, observed values/diagnostics, implementation and
dependency fingerprints, relevant artifact identity, and raw result location.
For split/high-risk workflows, identify the skill source/content version and
exposed reviewer/context identity so stale instructions or routing are visible.
Counts derive from actual invocations/results. List skipped or future gates as
such. A helper must consume the intended artifact; an arbitrary cached binary
or an ALL-PASS banner is not provenance.

During repair, use a concise closure table:

| Finding | Violated invariant | Applicable paths/consumers/states | Fix and permanent cases | Current result / remaining gap |
|---|---|---|---|---|

After two failed final reviews of the same delivery, review this table and the
oracle before another repair. Preserve scope authorization and continue work;
ask only for a genuinely changed requirement or missing decision.

Keep one current task row per stable ID. Update current requirements and link
prior revisions/findings/receipts rather than accumulating repeated active
checkboxes. Detailed logs belong in durable evidence reports. Reuse an existing
repository closeout gate when it fits; writing a new general checker is its own
bounded implementation task, not a requirement for every Plan.
