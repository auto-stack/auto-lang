# PLAN-797 独立 final re-review R17

2026-10-20（Asia/Shanghai），**pass / reviewed 8/9**。受审 **`5a54f588d2f6ea5cf50bd69ebb3d476c09e0d18e`**，PLAN-797 revision 1。F-797-R16-01 独立关闭，历史 findings 保持关闭；无新阻断或批准延期。AC-01–07 通过，AC-08 及 merge-owned 门禁仍 pending。未合入、发布有限 profile、归档或清理工作树。

## 基线与独立性

- stage review；review_scope final。当前 Codex 独立复审上下文，未实施本分支，未创建代理；具体模型/session ID 未暴露。先核对工作侧 R16 根因与覆盖调整，再从源和独立 C/PE 反例重建结论。本次是第十二次完整 final 验收（此前 R4/R5/R8/R9/R10/R11/R12/R13/R14/R15/R16）；就绪核查及无结论轮不混计。
- 唯一树 `D:/autostack/.wt/lang-797/auto-lang`、branch `plan-797-dev`；前轮 `cf9e4c7cd3cb8d94c06c917a84e7755356ba9c08`，diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾 HEAD 一致、实现树 clean，主 HEAD `a05b7f8c423411d9b4b08f052fd51ca1c7791a3d` 不变，见 [baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。仅主检出写复审证据/计划与债务簿记。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`；ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`；ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。双锁/验证配置/依赖稳定，D-796/D-788 满足。边界审计的早期草稿表由退役收据及 T-06 实际绑定记录接续，不将其早期“未满足”当成当前阻塞。
- rustc 1.98.0，MSVC 14.44.35207 目录（banner 19.44.35222）、SDK 10.0.26100.0、sysroot rust-lld。当前 release 库/worker、debug auto-case 已构建；[工具/二进制 SHA](tools-and-binaries.json.txt)。只对当前进程清空 RUSTC_WRAPPER，Cargo 均 `--locked --offline`。
- 六份模块 canonical Spec 主/树一致；goals 主 `e2367955…`、树 `ec5170b6…` 仍为已协调的 811 历史条目差异，未影响该 probe 的 touched_goals[] 理由。未发布 canonical Spec 或改 live ledger。
- 冻结 SD SHA256 **`01690e79d1a8570394a1b60dc7905970f6e1012330b6411b6e6fc8269c8c5314`**；[冻结正文](reviewed-spec-delta.md)、[支持矩阵](reviewed-support-matrix.md)、[修复 diff](repair-diff.txt)、[源码行](reviewed-source-excerpts.txt)。源/依赖/配置复用边界见 [unchanged inputs](unchanged-inputs.json.txt)。

## R16 修复与独立闭环

**F-797-R16-01/P2 CLOSED**（AC-04、T-05；原 repair regression）。`cache.rs:179–225` 的递归 walk 现在保留扫描根 root_base，只由它与累计 rel 构造 full；当前目录仅用于继续遍历，不再重复拼 prefix。仍只按完整路径/同位 staged-link 前缀排除调用方声明产物，不扩张为后缀或 basename 白名单。

[原独立反例](cache-nested-output-cleanup.rs) 在当前 release 库上重新编译，以真实 cl、独立 entry.obj、rust-lld、PE 退出值为 oracle：[原始结果](cache-nested-output-cleanup.log.txt)、[命令](cache-nested-output-cleanup-command.json.txt)。首次 PE17/miss/producer1；产物保留时重复 hit/producer1；**仅删除已声明 `src/sub/sub/value.obj` 后仍 hit、PE17、producer1**；新增真实 `src/sub/value.obj` 文本头后 **miss/PE29，与 fresh29 一致**。这同时验证旧例闭合和真正依赖不能被隐藏。

代表递归路径用同一输入/oracle，仅替换实际输出位置，另测扫描根、一层、三层；均为存在/重复/清理后 hit 且 producer 不增加，真实头新增后 miss/29。[三组源/命令/结果](cleanup-variant-results.json.txt)。产品永久测试 `abi_cache_nested_artifact_cleanup_reuses` 也通过；其嵌套目录新头控制允许保守失效，独立反例另证明实际解析改变时 PE 更新。

| 风险/历史 finding | 本轮实测与边界 | 证据 |
|---|---|---|
| F-R4-07 原同名异径/后缀/传递头错误对象 | 声明产物与真头同 basename 仍17→29；config.tmp、子路径、extensionless、父路径逃逸均 miss/29；无关新文件允许保守重编 | [产物路径](cache-artifact-path.log.txt)、[tmp](cache-tmp-header.log.txt)、[父路径](cache-transitive-parent.log.txt)、[传递子路径](cache-transitive-nested.log.txt)、[siblings](cache-siblings.log.txt) |
| F-R9-01 / F-R15-01 | 相对 cwd 两工程17/29各正确 miss；裸 `same.c` 成功 PE17；绝对域/Unicode/foreign tool-target 控制正确 | [相对域](cache-relative.log.txt)、[裸源](cache-bare-source.log.txt)、[旧域重放](cache-old-replay.log.txt) |
| F-R15-02 / F-R10-01 | 诚实旧 v3 同材料与 v4 键不同；升级 miss→编译发布17→再 hit，producer 总2，无 Conflict；无关文件变化合法重编也通过 | [旧 v3 升级](cache-v3-key-collision.log.txt)、[siblings](cache-siblings.log.txt) |
| 候选目录上限 | 1025 目录输入两次真实 PE17/miss、producer2、无缓存条目；不把超限当可证明命中 | [overcap](cache-overcap.log.txt) |
| F-R4-10 暂存预算 | 共享 DAG 900 名合法接受 peak5,880,880B；1300/1700 名拒绝，peak6,972,484/8,005,624B，均<8MiB | [独立堆计量](layout-memo-reuse.log.txt) |
| F-R4-09 判定/发布边界 | CLI 合法 PASS/true-true；错期望 FAIL/false-true；原始行写失败 ERROR/exit2；61正式行的落盘后 span 均≥行内判定时点，差0–13ms | [CLI正反](raw-row-control-results.json.txt)、[写失败](raw-publish-failure-results.json.txt)、[完成后span](raw-span-boundary-results.json.txt) |

各探针源、编译/执行命令和原始日志都在本证据目录；[runner](run-rust-probe.py) 第二参数给新 scratch，未写实现。独立探针不混入 Cargo 测试总数。layout 未变，R13 的原直链/小图/65536叶计量可按源/依赖/配置/输入一致复用；本轮24准入布局测试及上述 DAG 重新跑。R4/R5 其余已闭项、R10 原触发维持关闭，没有把 aggregate 绿当反例证明。

## 定向验收与健康

未运行整档 `t/tv/tt/tb/tu/taa/tf` 或完整 verify。

| 当前入口（均 locked/offline） | 结果 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 24+7+6=37 pass；独立 C 布局、准入拒绝、三腿及资源反控 | [native](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_ -- --nocapture` | 25 pass，含新永久清理控制及512 churn/GC | [cache](cache-tests.log.txt) |
| standalone `--test t06_binding abi_binding_ / abi_worker_` | 3+2 pass，真实语义绑定、worker并发GC/预算 | [binding](binding-tests.log.txt)、[worker](worker-tests.log.txt) |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 最终11 pass；首次10 pass+1性能门失败，见下节 | [父ABI](parent-tests.log.txt)、[首次日志](formal-attempt1/parent-tests.log.txt) |
| `cargo test -p auto-test-case` | 87 pass，公共协议/runner兼容 | [contract](contract-tests.log.txt) |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner](runner-tests.log.txt) |
| standalone `--test t05_cache_turnaround abi_turnaround_ -- --nocapture` | 1 pass；210样本全ok | [turnaround](turnaround-tests.log.txt)、[原始清单](turnaround-manifest.json.txt) |
| scoped fmt、root三文件rustfmt、diff --check | pass；无新增调试输出 | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[diff](diff-check.log.txt) |

**192不同定向测试在最终执行通过、1既有ignored**；重复父族不重复计数，首次性能失败明确保留，见 [verification summary](verification-summary.json.txt)。本计划文件无 primary warning，新增行无告警；[规范化路径/主诊断审计](health-warning-audit.json.txt) 已区别辅助引用与主位置。依赖 shim-metadata 及 root既有405/566告警仍在，不声称全仓零warning。

## 性能：保留首次失败与一次受控复测

六场景×30加拒绝30，210条全ok。standalone body-edit/reject P95=**355.2/0.6ms**，见 [summary](turnaround-summary.md)。固定 consumer helper 的六场景性能不代替新 C 依赖域正确性或正式 IPC 证据。

正式父路径首次 body-edit(edit+request) P50/P95/P99=**618/1010/1188ms**、reject P50/P95=**85/118ms**，body 硬门**未通过**。完整61行/summary/log保存在 [attempt1](formal-attempt1/summary.txt.txt)。立即检查观察到另一个树 lang-814 的 rustc/cargo 编译，后续快照仍有该树编译活动，[并发快照](formal-attempt1/concurrent-processes.json.txt)。这与环境波动一致，但仅凭进程共存不能严格证明超限因果；没有删除样本、提高1000ms阈值或把失败改为成功。

本轮其他测试/探针全部结束后，同提交/依赖/命令/规则**仅复测一次**父ABI族，没有干预外部进程；复测前也仍见外部编译进程，所以不宣称机器独占。最终11/11通过：body-edit P50/P95/P99=**589/657/693ms**、reject P50/P95=**84/106ms**，两条既定P95硬门通过。[正式61行](formal-gates.jsonl.txt)、[summary](formal-summary.txt)、前后进程快照及首次数据均保全。这是受控复测通过，不能解读为每次共享机器采样都稳定<1s；merge须在冻结提交重新运行该门，若失败按代码/负载归因，不能复用本次pass遮盖失败。

## AC / Spec delta / closeout

| AC → task → 实现/工件 | 结论 | 核验 |
|---|---|---|
| AC-01 → T-02 → descriptor/layout/validate + C header/oracle | pass（有限profile） | 24准入/布局测试，独立C/坏图/预算正反控制 |
| AC-02 → T-03 → emit/build/link + cside | pass（代表三腿） | 7项原生互调，swap/offset/漏调反控 |
| AC-03 → T-04 → resource/record/callback fixtures | pass（有限协议） | 6项真实分配/原方销毁/回滚/值校验/mut-out控制 |
| AC-04 → T-05 → cache/store/worker | pass | 原例及递归组合/真实依赖/版本发布正确，512 churn及GC/lease/用户哨兵通过 |
| AC-05 → T-05/T-06 → trace/正式adapter/原始行 | pass（受控复测） | 六场景210样本，正式双门657/106ms；首次1010ms失败独立保留 |
| AC-06 → T-06/T-07 → semref/recipe/语料/退役/终稿 | pass | D-796/D-788、contract/正式runner/三腿/冻结候选/SD与分域边界一致 |
| AC-07 → T-08 → 独立review | pass | finding全闭，源码/验收/SD/范围审查；冻结基线稳定 |
| AC-08 → T-09 → merge/规范/归档/清理 | **pending** | 本轮未执行merge-owned门禁或交付动作 |

SD-01/02：candidate、Windows x64、声明IR、descriptor/3与有限支持/拒绝/8MiB布局预算如实；SD-03：观察依赖、完整域/版本入键、负依赖和固定根完整路径排除、GC与真实请求发布边界已符合；SD-04/05：只导航并与774内部域区分。supersedes精确两项 `docs/specs/auto-ac/{project,data-resource-contract}.md`；new精确三项 `docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`；touched_goals[]书面依据有效。没有规范降格、未批准延期或要求未约定的全平台/任意规模能力。

遗漏/延期/workaround扫描：临时快照已退役，未改783/787底层或生产FFI/VM编译器；既有sema语料oracle缺失属明确登记的主分支债务，不冒称其已修。历史草稿和逐轮数字按对应时点保留，最终范围以冻结SD及本报告为准。

**next: merge；reviewed 8/9，T01–T08完成，T09 pending。** 本pass绑定该提交和上述输入，后续代码/依赖/配置变化不能自动继承。

**merge-owned pending**：新冻结提交上统一执行裸 `cargo t`、独立ABI完整套件/真实C三腿（含正式双硬门）、796/788受影响兼容；按最终diff判断触面档。当前diff仅probe/测试契约/父适配器，未改VM/编译器/trans/book/UI生成/aavm，不能机械追加无关 `tv/tt/tb/tu/taa/docs_gen`；如merge发现实际触面则补对应门。`tf`仅主检出到期批量单实例。门禁失败若需改代码，回work提交并复审影响面，再冻结重跑；不得先合入。规范/ledger/index沉淀、有限profile冻结登记、归档合入、批量到期判定、wt-guard及清树仍待实际收据。本次review不宣称交付完成。
