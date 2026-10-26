# T-11 执行收据（raw receipts, 2026-10-27, worktree plan-824-dev）

命令与结果摘要；完整输出经 `scripts/verify-av-dynamic-824.ps1 -Scope baseline`
逐段透传（该脚本自带 selected>0/ignored=0/必需 ID 恰一次守卫）。

| # | 命令（等价） | selected/passed | 退出 | 绑定行 |
|---|---|---|---|---|
| 1 | `cargo test -p auto-lang --lib --features auto-lang/test-cases av_dynamic_824 -- --test-threads=1` | 3/3 | 0 | D01/D02（正式语料 first-case + take-probe）、I12 负控 |
| 2 | `cargo test -p auto-av-probe --test av_dynamic_824_first_case av_dynamic_824_restore_reverifies` | 1/1 | 0 | D06 |
| 3 | `cargo test -p auto-av-probe --test av_dynamic_824_semantics` | 20/20 | 0 | D02/D03/D05/D09 + W01..W05 |
| 4 | `cargo test -p auto-av-probe --test av_dynamic_824_cache_compat` | 9/9 | 0 | D07/D08（守卫 ID= good_envelope/portable_good_checksum/restore_payload_types_fp） |
| 5 | `cargo test -p auto-av-probe --test av_dynamic_824_incremental` | 5/5 | 0 | I04/I10/I11 + D08 授权生命周期（churn-512 实测 512 修订全绿, worst 0.2ms/修订, 总时长 <1s） |
| 6 | `cargo test -p auto-av-probe --test av_dynamic_824_resources av_dynamic_824_resources_worker_memory_hard_gate_parent_sampled` | 1/1 | 0 | I06（fresh release worker ×4 父侧采样 ≤128MiB） |

`pwsh -File scripts/verify-av-dynamic-824.ps1 -Scope baseline` 终行：`PLAN-824 verify [baseline]: PASS`。

## 负控与守卫

- `av_dynamic_824_negative_oracle_fails`（expect-sabotage result/ledger 双轴必红）在 #1 内绿（=负控生效）。
- `av_dynamic_824_worker_faults_are_infra`（exit9/extra-frame/wrong-hash → Error 归类）在 #1 内绿。
- 脚本 GuardSelfTest 语义沿 first-case 交付（零选择/ignored>0/缺 ID/重复 ID 均 throw）。

## a2r / 认证

本 phase 零 a2r 认证调用、零 rustc/cargo/Rust 进程 spawn 归因于认证（#1..#6 均为测试面）。
既有有效认证按内容/依赖复用（共同合同 §5）；av.dynamic/1 无 a2r 面（gap 在案）。

## binding.tsv 变更摘要

- D01..D09：av 格 `gap:av-dyn-824` → `ok:av-dyn-824`；status → `dyn-824-bound`；owner 注追加实际入口指针。
- W01..W05、I04/I05/I06/I10/I11/I12：owner 注追加 824 动态证据指针（格值未动——W 的 av 格归 820 typed 通道，I 行归增量共享机制）。
- schema/金样/通用合同零改动。
