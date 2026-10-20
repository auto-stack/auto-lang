# PLAN-797 独立 final re-review R16

2026-10-20（Asia/Shanghai），**needs_fix / executing 4/9**。受审 `cf9e4c7cd3cb8d94c06c917a84e7755356ba9c08`。R15三项 **F-R4-07、F-R15-01、F-R15-02均CLOSED**。新增一项 **F-797-R16-01/P2（repair regression）**：递归扫描把目录前缀重复拼入完整路径，已声明的嵌套产物未被排除，清理该产物后同输入误miss并额外调用producer。本轮没有观察到该反例返回错误PE或超1s。

## 基线与独立性

- stage review；review_scope final；PLAN-797 r1。当前独立Codex复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露。先核读R15后的诊断/覆盖调整，再从源、真实C/PE和状态变更控制重建结论。这是第十一次完整final验收（此前R4/R5/R8/R9/R10/R11/R12/R13/R14/R15）；就绪核查不混计。
- 唯一worktree `D:/autostack/.wt/lang-797/auto-lang`、branch `plan-797-dev`；reviewed_commit `cf9e4c7cd3cb8d94c06c917a84e7755356ba9c08`，前轮 `e00133e626a3d8a7857fcd0bff21c4b5b101658c`，diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD一致、实现树clean，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。主HEAD保持 `de9a9054e47c432a3790c9c8228388a4a37abc67`。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788满足，双锁/依赖/验证配置/相关Spec输入稳定。
- rustc1.98.0、MSVC14.44.35207目录（banner19.44.35222）、SDK10.0.26100.0、sysroot rust-lld；构建当前release库/worker及debug父入口，SHA见[tools-and-binaries](tools-and-binaries.json.txt)，仅进程内清空RUSTC_WRAPPER。
- 6份模块canonical Spec主/树一致；goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`、树内 `ec5170b6…`，仍为已协调811历史条目差异，touched_goals[]依据有效。
- SD SHA256 **`26dc388426878577b3dcf4514f771462b798bbbcae7751836cb5b4263a6c7b1f`**；冻结[spec-delta](reviewed-spec-delta.md)、[support-matrix](reviewed-support-matrix.md)、[源码行](reviewed-source-excerpts.txt)。未改实现、canonical Spec或live ledger。

## 本轮实跑

Cargo均在原树、`--locked --offline`，没有整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 范围 | 实测 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 24+7+6=37 pass | [准入/三腿/资源](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_` | 24 pass | [缓存](cache-tests.log.txt) |
| standalone `--test t06_binding abi_binding_` | 3 pass | [绑定](binding-tests.log.txt) |
| standalone `--test t06_binding abi_worker_` | 2 pass | [worker](worker-tests.log.txt) |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 11 pass | [父ABI](parent-tests.log.txt) |
| `cargo test -p auto-test-case` | 87 pass | [契约](contract-tests.log.txt) |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner](runner-tests.log.txt) |
| standalone `--test t05_cache_turnaround abi_turnaround_ -- --nocapture` | 1 pass，210样本全ok | [六场景](turnaround-tests.log.txt)、[summary](turnaround-summary.md)、[原始清单](turnaround-manifest.json.txt) |
| scoped fmt、root三文件rustfmt、git diff --check | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[空白](diff-check.log.txt) |

共 **191不同定向测试pass、1既有ignored**，见[verification-summary](verification-summary.json.txt)。本计划改动文件无自身warning；既有依赖/root告警不冒称全仓清零，未发现额外调试输出。独立探针另计。

独立Rust探针按新PathBuf参数适配，仅声明实际创建的输出完整路径，经[runner](run-rust-probe.py)针对当前release库编译、真实cl/rust-lld/PE执行。命令保存在各`*-command.json.txt`；重放用新的scratch标签（runner第二参数）。只写本计划target私有源/store/产物，没有实施代码修改或创建链接。

## 原finding关闭

| Finding | R16结论 | 独立证据 |
|---|---|---|
| F-797-R4-07/P1 | **CLOSED（原错误依赖/错误对象不变式）** | 完整路径排除后，原声明src/value.obj而真正头sub/value.obj的控制现miss/PE29；config.tmp、父路径、相对/绝对工程、异toolchain以及上限不缓存控制均正确。下面的新问题是嵌套产物未排除造成误miss，不把它继续报为错误PE。 |
| F-797-R15-01/P2 | **CLOSED** | 裸Path("same.c")现成功编译/PE17；永久裸源测试含重复hit，原相对cwd工程隔离也正确。 |
| F-797-R15-02/P1 | **CLOSED** | `env|4`进入generation；独立旧v3原材料构造得到与新键不同的实际key，升级miss→编译→发布17成功；再请求hit=true且编译数仍2。 |

证据：[原产物名反例](cache-artifact-path.log.txt)、[裸源](cache-bare-source.log.txt)、[旧v3同域升级与重复hit](cache-v3-key-collision.log.txt)、[相对cwd](cache-relative.log.txt)、[旧域/Unicode](cache-old-replay.log.txt)、[config.tmp](cache-tmp-header.log.txt)、[父路径](cache-transitive-parent.log.txt)、[1025目录上限](cache-overcap.log.txt)。R9身份错用、R10原触发及其他历史项维持关闭。

F-R4-10内存维持CLOSED：本轮[共享DAG](layout-memo-reuse.log.txt)900名控制接受、峰5,880,880B，1300/1700名拒绝且峰<8MiB，24准入/布局测试通过；未变layout/依赖/配置/输入的R13其余计量可复用。F-R4-09也保持关闭：CLI合法PASS/true-true、错误期望FAIL/false-true、写失败ERROR/exit2，[结果](raw-row-control-results.json.txt)、[写失败](raw-publish-failure-results.json.txt)、[完成后时点](raw-span-boundary-results.json.txt)均重验。

## F-797-R16-01/P2：递归完整路径重复拼目录，使声明产物参与失效

AC-04，T-05（影响T-06/T-07整体收口）；**新修复回归**。`cache.rs:185`的`dir_s/base`在每层已是当前目录完整路径，`:198–202`的`rel`又包含从扫描根累计的prefix；`:204`递归传入e.path()和累计rel后，`:206`用base+rel拼完整路径，重复了相对目录。

例如扫描根src，递归到src/sub/sub时，某文件的实际路径为`src/sub/sub/value.obj`，rel为`sub/sub/value.obj`，比较用的full却成为`src/sub/sub/sub/sub/value.obj`。它不会匹配调用方如实声明的实际产物路径。

独立[cache-nested-output-cleanup.rs](cache-nested-output-cleanup.rs)、[结果](cache-nested-output-cleanup.log.txt)、[命令](cache-nested-output-cleanup-command.json.txt)使用真实C源及缓存：

1. producer将obj写到`src/sub/sub/value.obj`，该完整PathBuf明确传入artifact_paths。source及依赖位于私有project，store在搜索树外。
2. 首次成功PE17、miss、producer=1；产物保留时相同输入重复 **hit=true、producer仍1**。
3. **仅删除这个已声明的临时obj**。source、头、toolchain、flags、artifact_paths和持久化对象不变。下一查询实际 **hit=false、producer=2、PE17**；预期该文件既已声明排除，清理它不应改变缓存输入域，应直接复用已存对象且producer仍1。
4. 后续再新增真正近端头29时，仍能正确miss/PE29；本轮也另测嵌套目录中的头变化，没有观察到错误对象。因此本项仅为P2排除/复用回归，不夸大成P1错误执行或实测性能超限。

这不是要求所有无关文件变化都必须命中：无关header可以保守失效；此处变化的是API中**明确声明排除的实际产物**，SD-03亦要求完整路径排除，work交接还明确要求输出存在与否路径稳定。递归构造错误使该约定不成立。

应从每个目录项的实际完整路径进行规范化比较，或从固定根+累计rel拼接；不要同时使用已深入的当前目录与累计前缀。保留源根/一层/多层产物、存在/清理、真正头变化控制。修复应只改变产物排除，不扩大不可见文件集合。

## 性能、规范与AC

六场景各30+拒绝30，共210条全ok；standalone body-edit/reject P95=**267.5/0.3ms**，[summary](turnaround-summary.md)和原始JSONL已保全。其固定consumer helper测量不替代新排除路径的证明。

正式父路径body-edit(edit+request) P50/P95/P99=**602/722/723ms**、reject P50/P95=**84/87ms**，两硬门通过；[61行正式样本](formal-gates.jsonl.txt)、[summary](formal-summary.txt)保全，完成后span相对行内判定时点差1–3ms。没有因为正确场景通过就豁免上述排除约定。

| AC → tasks | 结论 | 依据 |
|---|---|---|
| AC-01 → T-02 | pass（有限profile） | 准入/独立C布局/坏图/预算控制。 |
| AC-02 → T-03 | pass（代表三腿） | 7项真实原生互调/反控。 |
| AC-03 → T-04 | pass（有限资源协议） | 6项资源/回滚/值校验/record mut-out。 |
| AC-04 → T-05 | partial/fail | 原错误对象/升级失败已闭；声明产物排除及相同输入复用仍有R16-01。 |
| AC-05 → T-05/T-06 | pass（当前性能/trace） | 210样本、正式61行及判定/资源/错误传播。 |
| AC-06 → T-06/T-07 | partial/fail | 绑定/退役/语料/兼容通过；SD-03完整产物排除尚未满足。 |
| AC-07 → T-08 | fail | 仅一项P2 OPEN，未批准延期。 |
| AC-08 → T-09 | merge pending | 未合入/沉淀/归档/guard清理。 |

SD-01/02/04/05有限candidate/target/声明IR/774分域及预算规则成立，SD-03版本身份/正确依赖及性能规则已通过，**完整路径产物排除partial/fail**。supersedes两条`docs/specs/auto-ac/{project,data-resource-contract}.md`、new三条`docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`准确，touched_goals[]书面理由有效。未改plan_revision/语义合同或发布canonical规范。

遗漏/延期扫描：承认全部旧具体finding关闭，单列新递归分支回归；不是将旧P1继续挂着或要求任意额外规模。唯一P2仍属已声明API/SD规则，记录债务不等于批准延期。

**next work**：同树修R16-01，提交后复审受影响路径。status executing、current_step4/9；T01–04完成、T05/T06/T07未闭，T08/T09 pending。

**merge-owned pending**：最终代码/验收/Spec pass后，在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t及实际diff触面档；tf仅主检出到期批量单实例。规范/ledger沉淀、合入/归档/wt-guard清理pending，门禁前不得合入或宣称交付。本轮没有实施修复或合入。
