# PLAN-786 R5 frozen Spec delta — final pass

plan_revision: 1
reviewed_commit: bb3cd34b78610ddbdc39d50e682eec7174896892
source_versions: ec934b0006acb9b9838def82d0fcaf438ae0153f (both target Specs)

## SD-01 — modify docs/specs/auto-man/project.md

Before: PLAN-780 pilot text states production pac.rs/vue.rs/rust_ui.rs readers have zero switching; four-name/locale responsibilities are already defined.
Source SHA256: C18FFAAFD22FB30244E9886B661F56C97BEED87EAD9CB3670C660B4914B195A4.

After: Vue project generation obtains name/title/title_zh once from the internal project_display_reader, the first finite production use of the project-model DTO. Field admission requires matching line-gate/provenance, no duplicate key and equivalence of the unique declaration's old single-line normalization. Other cases use one explicit compatibility scanner preserving first-match name (including empty) and title-family empty skipping. Reasons are ParseError, HiddenOrPhantomLine, DuplicateKey, UnsupportedValue, LineFormMismatch and DeepNesting. Locale/env choice, title/name/default fallbacks stay at existing callers. Other fields/readers, materialization and VM paths remain unchanged; this does not remove auto-man's language dependency or add a reverse edge. The project_model_adapter remains a test/probe consumer.

AC01/02/03; rationale: eliminate the selected duplicate loops while preserving observed values. Do not claim net line reduction: the two Vue loops became one compatibility scanner plus admission/tests.

## SD-02 — modify docs/specs/project-model/project.md

Before: production entry switching remains future work; static model stays a pilot with its documented unsupported syntax and normalization.
Source SHA256: C9CACD0D58A1FC5B6D023365D322A02D995A0282DDDB2B63A6BF1B6FD407C133.

After: auto-man's Vue display reader is a limited production consumer; its field-equivalence admission (including equal numeric astr cases) does not enlarge the model grammar. Before parse_document the consumer counts every literal '[' and '{' character in the original text, with no lexical skipping or decrement. A count greater than 64 routes all three fields to explicit DeepNesting legacy compatibility without invoking the model. This conservative count can include string/comment characters and independent flat groups; it is not a measured nesting depth or syntax limit. It can route shallow documents to compatibility while preserving old field values. Counts <=64 still undergo the normal parser/admission classification; no guarantee of static syntax is inferred from the count. The reviewed 64/65 arrays/objects/nodes/mixed/flat/string/comment boundary and historical process-isolated counterexamples demonstrate the chosen consumer envelope. Model-wide recursive parser governance remains outside this change; do not claim unlimited-size or arbitrary-thread-stack safety.

Registry/loader, other Pac/VM consumers, remaining text readers and locale evaluation remain outside this migration. Retain unsupported/error and semantic-normalization rules already documented by PLAN-780.

AC01/02; rationale: describe actual limited wiring, precise compatibility boundary, and consumer safety preflight.

supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/project-model/project.md]
new_spec_components: []
touched_goals: [] (internal organization/compatibility; no language/UI goal closure)

Migration follow-up: PLAN-784 t04 migration-inventory already names PLAN-786 display-reader and the Vue/Rust UI migration surface. Preserve the new internal reader and its compatibility scanner as part of that Man migration; other reader copies remain explicit future work.

Canonical Specs and live ledger are not changed by review. Merge must deposit both rules, reconcile the old zero-switch statements, and refresh the derived index/ledger.
