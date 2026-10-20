# PLAN-798 R14：修复重放与 readiness 核查（2026-10-20）

**needs_fix**。R13-01 已独立验证关闭；新增 self-call 正式 runner 腿通过。正式周转门本次仍红，另发现 F798-R14-01：提交中的 30 样本内存验收证据未闭合。保持 `executing`，重开 T-05，T-07/08 继续开放，实际完成 **6/11**。不授完整最终 pass。

review_scope=final；实际范围是受影响反例与 readiness。当前聊天 Codex 独立于实施，未委派代理。已知 work-owned 条件未完成，按 auto-plan-review 技能止于定向核查，不再跑一轮完整最终验收。技能来源 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`，SHA-256 `0d68ba05276aff05a143c8f27010ee02b0a9bded14c27a0133d3a774c20f0cf3`。

## 基线及证据边界

- 计划：主检出 `2e630a935a24e74f937c2d6bac0d8acd528a2ac6`，plan_revision=5；审前计划 SHA-256 `d118e21e638337542f4969ac0d595ddec7144f1749d595eec9d2087b6b31ba6c`。
- 原实施树 `D:/autostack/.wt/lang-798/auto-lang`，branch `plan-798-dev`，受审提交 `c8c598f0aa67f79d8b083346b5c552a655353d1d`；完整 diff base `13367d127584461449ee2f151c0c5987f0740044`；R13 后变更 8 文件 +583/-9（含修复 `a7850407e`）。审前/测试后 HEAD 相同且 clean。
- ac-core 最后变更 `e02c2dcc8094fc522569533304aea175f6aae867`；artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`；semantics-contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`。三依赖 tree 与 R13 相同。组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean。
- 计划/Spec/SD 哈希见 [baseline.json.txt](baseline.json.txt)，锁、配置、实现及提交 raw 哈希见 [source-identities.json.txt](source-identities.json.txt)。冻结 SD 见 [sd-frozen.md](sd-frozen.md)，实现合同见 [contract-decision-frozen.md](contract-decision-frozen.md)。未发布 canonical Spec 或 ledger。
- 既往 R12 unit/旧缓存、R13 core/ABC/cache 定向证据保持其原范围；本轮相关 core/cache 源码、依赖和配置未改变，因此不为 readiness 重复运行。它们不能外推为本轮完整 K14 各入口或所有 AC 的最终通过。

## 已验证的修复

R13 独立探针源原样复制，offline/locked 重建后重放：`prefix-call-probes`。整数真实 HIR 131 层及布尔 132 层现均在 admission 报 `av.resource.frontend-nesting`，带/不带分组一致拒绝；116/24 层整数合法控制分别执行 -1/1。直接 HIR→MIR→ABC 路径保持独立参照，未将 resource 当语言 trap。永久测试 `av_resources_prefix_unary_binds_to_full_call` 1/1，通过其普通 operand、布尔低深和组合控制。

精确源码和输入见 [probes/src/main.rs](probes/src/main.rs)、[inputs/](inputs/)，输出 [prefix-call-replay.log.txt](prefix-call-replay.log.txt)、[prefix-permanent-test.log.txt](prefix-permanent-test.log.txt)。**F798-R13-01 关闭**，不代表所有潜在前门形状均已证明。

正式 AutoDown 正确性定向族排除性能门后 **4/4**：corpus、负 oracle、decode 负例、worker 故障注入。corpus 实际 7 个 receipt；新增 `av.core.kind-selfcall-module` 含两次 run，末步真实 `value=42`、`cache_hit_abc=1`、`abc_units_emitted=0`、`verify_load=1`、`av_execute=1`、`av_steps=6`，并有 abc_load/abc_verify spans，证明命中仍重验执行。见 [formal-correctness.log.txt](formal-correctness.log.txt)、[kind-selfcall-receipt.json.txt](kind-selfcall-receipt.json.txt)。K14-01/05 新正式 runner 腿通过。

## 正式 30 样本门：本次未通过

只运行一次，无改阈值或删失败样本：

```text
AV798_FORMAL_SAMPLES=30 AV798_FORMAL_GATE_MS=1000
cargo test -p auto-lang --lib --features test-cases --offline --locked
  tests::ac_av_probe_tests::ac_av_probe_formal_turnaround_gates
  -- --exact --test-threads=1 --nocapture
```

| 本轮正式门 | 结果 |
|---|---:|
| edit P50/P95/P99 | 931.4 / **1195.8** / 1212.2 ms |
| sig P95 | 540.3 ms |
| 阈值/样本 | P95 <1000 ms；每路径 30 |
| 测试/退出码 | FAILED / 101；160.69s |

原始 [formal-turnaround.log](formal-turnaround.log)。启动 CPU 负载 62%，存在其他 cargo/rust-lld 进程；期间采样 43%，不能视作独占静置环境。用户此前 A/B 同红作为 work 提供的归因保留，本轮没有重复 A/B，因此不独立断言“无码回归”。本次结果不能解锁 V-11 formal，**T-07/08 仍 pending**；也不据共享负载下的单次红新增代码回归 finding。未再重跑取绿。

## F798-R14-01 [P2] 30 样本双线内存门缺少对应汇总，摘要与 raw 数字不一致

类别：验收证据收口缺口；AC-06/08，V-09/11，T-05/08。位置：`docs/reports/plan798/acceptance.md:129-135`。

提交 `c8c598f0a` 的两个新 jsonl 各有 180 行，六场景各 30 行、零失败；body edit 有 30 个不同输入 hash，16/8/8 受管分区峰值均未越界。按当前 `observations::percentile_ms` 的 `round(p*(n-1))` 重算 `turnaround-r5-30sample-pass.jsonl`：

| 指标 | 提交 raw 重算 | acceptance 摘要 |
|---|---:|---:|
| edit P50 | 828.3609 ms | 827.3 ms |
| edit P95 | **897.3352 ms** | 888.8 ms |
| edit P99 | 915.1756 ms | 910.0 ms |
| sig P95 | 41.6626 ms | 41.4 ms |

重算后两条时间硬门仍通过，不能因数字差异误判它们失败。但两个新 raw 都没有 `frontend_peak_bytes`、`private_bytes_peak`、`child_private_bytes_peak` 或两条 target_met 字段；这些字段属于 BenchSummary，不能从 SampleRow 重建。提交的 `turnaround-summary.json` 仍为旧记录（edit P95=422.1057 ms、`worker_private_bytes_target_met=false`、无 r5 A 线 target_met），不能对应上述新 raw 或证明本次两线门通过。“含显式内存字段的首跑”引用当前不可复现。没有断言内存实测超标，缺的是此次正式 30 样本的可核验证据。

修复：找回并持久保存**同一次当前基线运行**的真实 BenchSummary、原始命令/退出码、对应 raw 的 hash 与二进制/提交身份，包含 A/B 实测及 target_met/hard_gate_failures；按该 raw 更正摘要数字。如果原结果无法恢复，补一次受控的六场景×30 正式 bench，完整保留 summary 与 raw。不得手填内存字段、拼接其他轮次或用 R13 六场景×1 冒烟代替。无需改变预算或实现以解决本 finding。

完整重算见 [committed-raw-audit.json.txt](committed-raw-audit.json.txt)，原件身份见 source-identities。

## AC、规范与健康状态

| AC | 本轮结论与边界 |
|---|---|
| AC-01/03/04/05 | partial：保留已关闭的既往证据；新增 K14-01/05 正式腿通过，未因六行矩阵文字全绿授全入口 pass |
| AC-02 | R13-01 修复重放通过；其他既往范围保留 |
| AC-06 | partial：提交 raw 的六场景计数、两条 bench 时间门及受管分区通过；30 样本双线内存证据缺失，正式 runner 时间门失败 |
| AC-07 | partial：本轮正式正确性/负控 4/4；性能门未解锁 |
| AC-08 | partial：覆盖表/SD已存在，但本轮报告与原始数据须按 F14-01 收口 |
| AC-09 | 未完成，不授 reviewed |
| AC-10 | merge-owned pending |

两个 package 的 fmt、完整计划 diff-check 均 exit 0。原根 lib test 输出 566 条仓库既有 warning，本轮 root Rust 源码未变；实验包本轮未出现新增实施 warning，探针构建日志另保留。没有将根仓整体健康声称为零 warning。见 [fmt.log.txt](fmt.log.txt)、[diff-check.log.txt](diff-check.log.txt)、[probe-build.log.txt](probe-build.log.txt)。

SD-01..07 当前提案与 r5 资源政策保留，未改 semantic contract 或 plan_revision。完整知识增量终审随 readiness 恢复后执行；当前 Spec 输入未漂移。未作未授权缩围/延期，F14-01 登记为合同内必修，不以债务豁免。

下一步回原树 work：先补齐 T-05/08 的同轮汇总和证据绑定，再在可控静置条件完成原正式 runner 30 样本门；提交后送审。最终 review pass 后才由 merge 执行适用根日常/触面档、兼容整档、Spec/ledger沉淀与归档/guard清理，当前全部 pending。未运行根整档 `t/tv/tt/tb/tu/taa/tf`，未改实施、合入或清树。
