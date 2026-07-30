# Pre-Commit Checklist

Before committing .at file changes, verify:

## Code quality

- [ ] `auto build` passes without errors
- [ ] No Plan 361 validation warnings (R001-R007)
- [ ] `auto ui inspect <file.at>` shows expected widget structure

## Testing

- [ ] New features have `acceptance.atd` T-entries
- [ ] New features have `.spec.ts` skeletons (or implemented tests)
- [ ] Bug fixes have regression test entries (marked with "历史教训")
- [ ] `pnpm test:smoke` passes (all 13 smoke tests green)
- [ ] `cargo test -p auto-lang -- ui_snapshots` passes

## Contracts

- [ ] No generator contract violations (C1-C9)
- [ ] AutoDownEditor: single instance + canEdit (not dual v-if)
- [ ] Handler names: exact matching (especially `.ToggleDarkMode`)
- [ ] Store syntax: `use store: Name` (not `use back.store:`)
- [ ] CSS variable scoping: target element matches `.dark` rule element (C8)

## For AI agents

When generating .at code, run through this checklist before presenting
the result to the user. Flag any violations explicitly.
