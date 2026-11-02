# examples/ui — moved to auto-ui

All managed AutoUI teaching examples, the shared `stylekit` library and the
`remote-frame-e2e` verification fixture now live in the UI framework repository:

**`auto-ui/examples/ui/`** (source repo: <https://github.com/autostack/auto-ui>,
local checkout sibling `../auto-ui`)

Relocated by auto-ui PLAN-007 (T-02, phase A source relocation, 2026-11-02).
The migration preserved every file byte-identical (per-file SHA-256 manifest,
381 tracked files at auto-lang `5923d4dee`); cross-repo history is readable
through the two commits: this deletion commit (source side) and the matching
addition commit in auto-ui (target side).

Language-side notes:

- UI example assets are no longer runtime inputs for this repository.
  Language tests that need UI-shaped inputs use small, fixed, provenance-marked
  fixtures instead of the live example tree.
- Historical documents, plan archives and test snapshots that reference
  `auto-lang/examples/ui` keep their historical paths; the migration mapping
  (old path → `auto-ui/examples/ui/<same-relative-path>`) explains them.
- Do not recreate runnable `.at` copies here, and do not add junctions/symlinks
  to the new location.
