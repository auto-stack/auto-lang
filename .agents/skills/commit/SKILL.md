---
name: commit
description: Commit verified changes in the current dedicated worktree, following this repository's scoped gates and Plan workflow.
---

# Commit verified work

1. Read AGENTS.md and the identified Plan. Verify that the current directory and branch belong to that Plan's dedicated worktree; implementation never goes directly onto master.
2. Inspect the diff and staged paths. Preserve other sessions' changes and local settings; stage only this task's intended files.
3. Run only the implementation-scoped verification required by AGENTS.md. Asset/docs-only work does not run cargo t or docs_gen; Rust implementation and needs_fix repairs use necessary cargo check, finding replays and scoped module tests. Committing or resubmitting a repair does not trigger whole-tier tests. After final code/acceptance review passes, merge owns the affected fixed-group union (necessary integration groups and required slow groups, per PLAN-896 `cargo t <group>`) on the frozen commit before landing; record these gates as pending at the implementation/review handoff. The periodic `cargo t all` batch belongs only to the single-instance batch regression gate on the main checkout (PLAN-898).
4. Commit with a Conventional Commit message and the Plan ID when applicable, recording actual validation and any identified baseline failures in the Plan.
5. Follow auto-plan:review and auto-plan:merge for landing, knowledge deposit, archival and guarded cleanup. Push only when the user has authorized publishing to the remote.
