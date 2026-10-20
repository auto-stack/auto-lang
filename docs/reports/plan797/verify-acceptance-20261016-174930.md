# PLAN-797 verify-autoabi-797 receipt

- scope: acceptance
- head: defcc8eaa1fdcfa619feda99721d0b93c25035fb
- generated: 2026-10-16T17:49:30+08:00

| step | ok | seconds | detail |
|---|---|---|---|
| context (HEAD + toolchain + lock identity) | True | 0.1 |  |
| Cargo.lock tracked in git (F-797-R1-03) | True | 0.1 |  |
| cargo metadata --locked --offline | True | 0.2 |  |
| cargo check --locked --offline (probe crate zero-warning) | True | 0.4 |  |
| cargo test --locked --offline (probe workspace) | True | 101.8 |  |
| T-07 independently-preparable materials present | True | 0 |  |

Raw cargo outputs and receipt lines: D:\autostack\.wt\lang-797\auto-lang\experimental\auto-abi-probe\target\autoabi-verify-receipts\797\raw-20261016-174748
