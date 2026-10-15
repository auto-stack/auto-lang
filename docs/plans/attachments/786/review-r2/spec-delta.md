# PLAN-786 R2 Spec delta snapshot — not approved for merge

plan_revision: 1
reviewed_commit: db71263b5b9b7f9451c6e588fef31765accbba3d
Source versions: both targets ec934b0006acb9b9838def82d0fcaf438ae0153f.

## SD-01 modify docs/specs/auto-man/project.md

Source SHA256: C18FFAAFD22FB30244E9886B661F56C97BEED87EAD9CB3670C660B4914B195A4
Before: existing Vue field reading and four-name/locale responsibilities.
After proposed: Vue generation reads name/title/title_zh once through internal project_display_reader; consumer performs per-field DTO admission using provenance/gate line identity, duplicate checking and old single-line normalization. Legacy compatibility preserves first-match name (including empty), title-family skipping of empty values, and unsupported or malformed input results. Locale and existing name/fallback remain in Vue. Other readers are unchanged. No new VM execution, dependency reversal or project-model syntax support is implied.
Actual compatibility reasons: ParseError, HiddenOrPhantomLine, DuplicateKey, UnsupportedValue, LineFormMismatch, DeepNesting.
AC: AC01/02/03; rationale: finite production consumer consolidation with behavior preservation.

## SD-02 modify docs/specs/project-model/project.md

Source SHA256: C9CACD0D58A1FC5B6D023365D322A02D995A0282DDDB2B63A6BF1B6FD407C133
Before: future wiring section says no production entry switched.
After proposed: auto-man Vue display reader is the first limited production consumer. Consumer field-equivalence admission (including equal numeric astr values) does not expand the model's frozen subset; registry/loader and other Pac/VM paths remain unchanged. auto-man still depends on language execution elsewhere.
Current implementation selects legacy compatibility when its bracket prefilter reports depth >64, preserving returned text rather than rejecting syntax. R2 discovered undercounting for multiline strings: this is current faulty behavior, not an approved safety guarantee. After repair, freeze the actual conservative safety envelope and limitations again; do not publish the current guard as sound.
AC: AC01/02; rationale: accurate consumer/current-state documentation.

supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/project-model/project.md]
new_spec_components: []
touched_goals: [] (internal organization, no capability goal closure)

Migration follow-up: explicitly register crates/auto-man/src/project_display_reader.rs and its compat adapter in the future Man migration inventory (784 currently lists broader paths). No canonical spec or live ledger was edited during review.
