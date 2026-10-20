# PLAN-797 verify-autoabi-797 receipt

- scope: closeout
- head: 3bcc4195431bf17f493ff0d40487b93c6bf8efe5
- generated: 2026-10-20T09:58:02+08:00

| step | ok | seconds | detail |
|---|---|---|---|
| context (HEAD + toolchain + lock identity) | True | 0.1 |  |
| Cargo.lock tracked in git (F-797-R1-03) | True | 0.1 |  |
| cargo metadata --locked --offline | True | 0.3 |  |
| cargo check --locked --offline (probe crate zero-warning) | True | 0.6 |  |
| cargo test --locked --offline (probe workspace) | True | 93.1 |  |
| cargo fmt --check (probe crate) | True | 0.2 |  |
| cargo test builds with zero crate warnings | True | 0.2 |  |
| abi_binding_ real-contract tests + FORMAL hard gates (probe) | True | 28.6 |  |
| abi/interop-v1 corpus + lifecycle + parent-path gates (unified runner) | True | 31.8 |  |
| retirement receipt + finalized T-07 materials | True | 0 |  |
| T-07 independently-preparable materials present | True | 0 |  |
| closeout checklist (honest pending vs done) | True | 0 |  |

Raw cargo outputs and receipt lines: D:\autostack\.wt\lang-797\auto-lang\experimental\auto-abi-probe\target\autoabi-verify-receipts\797\raw-20261020-095527
