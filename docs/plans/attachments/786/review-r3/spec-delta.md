# PLAN-786 R3 Spec delta snapshot — needs_fix, not approved for publication

plan_revision: 1
reviewed_commit: 322719893262352ae1f25d894a833571c044e434
Both target source versions: ec934b0006acb9b9838def82d0fcaf438ae0153f.

## SD-01 modify docs/specs/auto-man/project.md

Source SHA256 C18FFAAFD22FB30244E9886B661F56C97BEED87EAD9CB3670C660B4914B195A4.
Before: existing Vue field reading and four-name/locale responsibilities.
Proposed after: Vue generation reads name/title/title_zh once through the internal project_display_reader. Per-field DTO admission checks provenance/gate line identity, duplicates and old single-line value equivalence. Legacy compatibility preserves first-match name including empty and title-family empty skipping. Locale and name/fallback remain in Vue; other readers remain unchanged. No new VM execution or reverse dependency.
Reasons implemented: ParseError, HiddenOrPhantomLine, DuplicateKey, UnsupportedValue, LineFormMismatch, DeepNesting.
AC01/02/03; finite production consolidation with existing behavior preserved.

## SD-02 modify docs/specs/project-model/project.md

Source SHA256 C9CACD0D58A1FC5B6D023365D322A02D995A0282DDDB2B63A6BF1B6FD407C133.
Before: no production entry switched, future adapter wiring only.
Proposed after: auto-man Vue display reader is the first finite production consumer; field-equivalence admission (including equal numeric astr values) does not expand model syntax. Registry/loader and other Pac/VM paths remain unchanged; auto-man still depends on language execution elsewhere.
Current consumer guard uses >64 measured bracket depth to choose whole-read legacy compatibility, not syntax rejection. R3 demonstrates that skipping broad quoted ranges is not a sound upper bound across skip_to_entry_end reentry. Do not publish a sound safety envelope until repaired and re-reviewed. Exact safety wording must be refrozen on the repaired commit; model recursion governance remains a separate follow-up.
AC01/02; accurate limited-consumer/current-state documentation.

supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/project-model/project.md]
new_spec_components: []
touched_goals: [] — internal organization/compatibility; no language or UI capability goal closure.
Migration follow-up: the 784 Man migration inventory must explicitly retain this reader and its compatibility adapter. Canonical Specs/live ledger were not changed in this review.
