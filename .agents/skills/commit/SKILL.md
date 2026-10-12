---
name: commit
description: Commit verified changes in the current dedicated worktree, following this repository's scoped gates and Plan workflow.
---

# Commit verified work

1. Read AGENTS.md and the identified Plan. Verify that the current directory and branch belong to that Plan's dedicated worktree; implementation never goes directly onto master.
2. Inspect the diff and staged paths. Preserve other sessions' changes and local settings; stage only this task's intended files.
3. Run the verification required by the actual change category in AGENTS.md. Asset/docs-only work does not run cargo t or docs_gen; scoped Rust work uses cargo check and module checks, then cargo t plus required surface tiers before review. cargo tf belongs to the single-instance batch regression gate on the main checkout.
4. Commit with a Conventional Commit message and the Plan ID, recording actual validation and any identified baseline failures in the Plan.
5. Follow auto-plan:review and auto-plan:merge for landing, knowledge deposit, archival and guarded cleanup. Push only when the user has authorized publishing to the remote.
