# PLAN-787 final review R1 — needs_fix

2026-10-16；stage: review；review_scope: final（readiness 拒绝及有界风险审查）。
reviewer_identity: 当前 Codex 会话，未参与该实施；未创建代理，未修实现。
本次不把既有测试数量或 work 收据视为验收，不宣称未检查路径通过。

## 冻结基线

- plan_revision: 1；共享计划来自主检出 `D:/autostack/auto-lang`，主检出 HEAD `2f6b06c747c1318feeac217939a5e91ffd1307a9`。
- worktree: `D:/autostack/.wt/lang-787/auto-lang`；branch: `plan-787-dev`。
- reviewed_commit: `4eed69e67085eb150d30443d5c2f41d2530e34c6`。
- base_commit / merge-base: `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`（783 合入基线）。实施树开始/结束 status 均 clean。
- dependency_revisions: auto-atom、auto-ac-prototype、ac-artifact-probe 均经 path 解析到**同一实施树**，不是 work 收据声称的主检出依赖。版本由该 Git commit 及 `baseline-hashes.tsv` 中清单/锁文件 hash 绑定。
- spec_inputs: 主检出现行 Specs/Design39/40 的 SHA-256 见 `spec-inputs.tsv`。
- 冻结规范增量: `spec-delta.md`，SHA-256 `EEC8FB8D32AB7FC613ACD151AFF921FF22B4921A6D03E9467D9CD097B9FFCAC7`。该冻结稿未获通过，不能发布为规范。

## 实际验证

1. 在原实施树运行 `./scripts/verify-ac-syntax-787.ps1 -Scope acceptance`，13 族通过，合计 78 次通过记录（含测试模块重复编入，不是独立用例总数）；perf_/syntax_budget_ 被脚本 skip-closeout。
2. 独立反例 `probes/counterexamples.rs`：**1 pass / 6 fail / 0 ignored**，完整命令/输出见 `counterexamples.log`。通过控制为 100 声明的普通持久化多页往返；失败均在实际公开读写入口重放。
3. `cargo check --manifest-path experimental/ac-syntax/Cargo.toml --workspace` 成功，但新 runtime/grammar/artifact adapter 分别产生 7/1/15 条警告（另有既有 shim-metadata 警告），见 `check.log`。
4. 对新增语法工作区 Rust 文件执行 `rustfmt --edition 2021 --check <files>` 失败，见 `fmt.log`。`cargo fmt --all` 自身因依赖根工作区缺 auto-down 兄弟路径无法启动，未据此指责代码；直接 rustfmt 已证实本次新代码存在格式漂移。`git diff --check bf773ca9b HEAD` 另报 artifacts/Cargo.toml:13、artifacts/src/lib.rs:1186 空尾行。
5. 不执行 closeout 全套。只复现其第一个 probe 命令的参数解析：`cargo test --manifest-path experimental/ac-artifact-probe/Cargo.toml '--lib 2>&1'`，立即失败，见 `closeout-argv.log`；没有运行根整档或 docs_gen。

沙箱首次禁止原树构建目录写入，获得工具自动批准后重跑定向检查成功；这不是实现失败。独立探针和记录写在主检出的计划附件，不修改实施源码。

## Findings（全部为现有合同缺口，登记不批准延期）

| ID / severity | AC / tasks | 证据与要求 |
|---|---|---|
| F787-R1-01 / P1 | AC-01/05；T-02/06 | `runtime/src/snapshot.rs:47` 的淘汰以 Rc==1 判断，但 Snapshot:82 只持 PageRef hash，不持页引用/lease。1 页预算中创建仍活跃的 a 页，再创建 b 文档，旧快照读首字节即 `Storage(page ... evicted)`。见 live_snapshot_survives_other_document_pressure。必须让当前及显式旧快照真实保活，且压力下能外存再读；验证释放后的可回收性，不仅检查 hash 不变。 |
| F787-R1-02 / P1 | AC-01；T-02 | `runtime/src/position.rs:120` 跨页续行把页起点当行起点。16394 个 x 的单行在 byte 16389 返回 (1,6)，应 (1,16390)。见 line_column_inside_cross_page_line。UTF-16 使用相同行起点也受影响；修复跨多页续行与 Unicode/CRLF 组合并做正反往返。 |
| F787-R1-03 / P1 | AC-04；T-05 | `runtime/src/edit/mod.rs:114` 在检查收敛前调用 lex_from 扫到 EOF，:209 却只把对齐点距离报为 rescan_bytes。约 1MiB mini 文档中部一个名字字符替换，报告 27B，PageStore 实际源页读取 1,367,732B。见 local_lexer_real_reads_are_bounded。必须在 scanner 真正停止/有界窗口路径收敛，直接计量实际读/扫描工作，补禁止全后缀扫描的反控。不能用所需逻辑跨度替代已完成工作量。 |
| F787-R1-04 / P1 | AC-04；T-04/05 | `runtime/src/parse/mod.rs:368 perform_graft` 对所有未改声明逐节点逐边复制进新 Vec，CstTree 本身也是单体 arena。reused_decls 只数跳过解析的声明，未验证计划要求的“未变内容页真实复用≥95%”；开头插入仍复制所有后缀节点。需交付不可变共享内容块及长度目录，验证内容块身份/分配与实际复制量；保留 fresh 全字段/诊断/重建等价。若需改变方案，应先修订合同，不把复制更名为页复用。 |
| F787-R1-05 / P1 | AC-02/05；T-06 | `adapters/artifacts/src/lib.rs:52 tokens_key` 未使用 p，无 GrammarId/hash；get_tokens 也不验证身份。写入 api grammar 的 token，再以同名 endpoint 变体读同源，得到 Some(old tokens)，应 miss。见 grammar_variant_tokens_do_not_hit_old_grammar。A02 只写 token，没有完整 scanner checkpoint/mode；put_tokens edges 为空也没有所称 A02→A01 依赖。需补身份/版本/checkpoint 格式及跨 grammar 真读拒绝/往返。 |
| F787-R1-06 / P1 | AC-05；T-06 | StreamingCstBuilder::finish:896/929 的目录页及 manifest 都只 required 最后一个数据页，manifest 不 required 目录页。rooted_stream_directory_survives_gc 在已注册当前根后运行合法强制 GC，目录被删，活根无法查询。普通 put_cst:233 只依赖 page_ids[0]，而数据页依赖方向是后页→前页，也不能靠该根覆盖全页。全量历史 artifact 被分别软根及 cst-manifest 硬根登记，视图没有 store lease/当前根替换释放流程。stream push_chunk 不持久化源字节或 A02/A01 required 链，结束后不能从 store 重建原文。必须补完整 required-data 图、源页、真实当前根/旧 lease 生命周期，证明 GC 保活与释放后回收；改 adapter，不擅修 783 GC。 |
| F787-R1-07 / P1 | AC-05；T-06 | `adapters/artifacts/src/lib.rs:633` 直接索引 manifest.pages[d.page]；有效 frame/hash 下，decl.page=999/pages=[] 通过 open_cst 后在 :635 索引 panic。见 valid_framed_bad_manifest_ref_is_controlled。decode_page 只查边数组区间，不查子节点目标/环/树跨度；错 kind 也被忽略。既有“截断页”测试在 CAS hash 层先失败，未证明结构拒绝。需在真实读入口验证目录/节点/边/owner/grammar/kind 合同，坏数据受控错误，不能静默丢边后返回成功。 |
| F787-R1-08 / P1 | AC-05；T-06/07 | 64MiB 测试先 gen_chunks 收集全部源至 Vec<String>（tests/syntax_budget_.rs:25/46），与“生成器另进程、runtime 不整源驻留”相悖。StreamingCstBuilder:763 的 decls 持续累积，2,568,000 个 DeclEntry 至少约 58.8MiB（不含 capacity/其它索引），到 finish 才分页，已超过 8MiB staging/16MiB 总受管预算。断言只统计 CstView hot，忽略目录/构建工作集、I/O 与 OS private bytes/峰值；matrix 自认 OS 证据缺失却记 PASS。需端到端有界、在线落盘索引、生成器与 runtime 分列、实际工作集/读页payload与 OS 峰值，巨声明受控资源拒绝。合同明确该缺口不能判 pass；不得默许放宽数字。 |
| F787-R1-09 / P2 | AC-03/05/06；T-04/06/07 | V-09 只有同进程 drop/reopen，旧 snap/grammar/tree 都存活，没有真正新进程恢复及 parser 禁调控制；双工程测试丢弃 space report，无共享/诊断隔离断言；预算只 drop(view) 冒称 lease 释放。depth_nested_parens_interleaved 每两轮闭合一次，4096 次循环并不构成 4096 嵌套，且 depth 族未覆盖持久化编解码。perf 无旧 parser 对照，回退不断言确实回退，无场景 I/O/内存原始样本，报告称增量含 fresh 计时但代码计时区不含 fresh。verify 把 work-owned perf/budget 仅放 closeout；closeout 的 --lib/--test/重定向被拼成单一参数，第一条即报 unexpected argument，且不是工作区完整独立套件。需补真实 oracle/负控制/失败传播及文证一致性，功能验收在 work 完成，完整回归留 merge。SD-03/05/06 的页复用/重启/required pin/同预算主张亦须依修复证据重审；增量缺目标版本与真实目标逐条 before/after，touched_goals=[] 需给无影响说明。 |
| F787-R1-10 / P2 | AC-07；T-07 | 新实施产生 23 条编译警告并存在大量格式漂移；其中 register_root Result 被两处忽略（:236/:932），flush_batch .expect("page put") 将存储失败转为 panic。需处理新增警告、触面格式与可恢复错误传播，保留既有 shim-metadata 警告的归属记录，不以执行器隐藏编译输出代替 health check。 |

## AC / SD 判定与覆盖边界

| AC | 结果 | 本次结论 |
|---|---|---|
| AC-01 | fail | F01/02；现有坐标/代际族通过不能覆盖页压力和跨页续行。 |
| AC-02 | partial | 配置驱动/拒绝族重跑通过，确有通用规则机和配置变更控制；跨 grammar 产物隔离 fail 见 F05。未对生成器全部组合声称通过。 |
| AC-03 | partial | lossless/recovery/core_projection 既有族通过；深度全路径及持久化 oracle 不完整见 F09，未宣称所有严格投影组合完备。 |
| AC-04 | fail | F03/04；200 次 replay/fresh 相同并不证明有限实际工作或内容共享。 |
| AC-05 | fail | F01/05/06/07/08/09；D-783 依赖基线存在，问题属于本 adapter，不归咎于 783。 |
| AC-06 | fail | 生产面 diff 为空、旧 API 控制通过；性能/内存证据及 SD 不成立，见 F08/09。 |
| AC-07 | fail | 本记录绑定代码并完成本轮风险扫描，但未清阻断，不能勾 T-08。 |
| AC-08 | pending merge-owned | 未进入 merge；完整语法及受影响 783 独立套件、Spec/索引/ledger、合流归档/guard 清理仍待冻结新提交审查通过后执行。 |

SD-01/02 部分描述可由现有结构/配置测试支持，但 AC 尚未闭合；SD-03/05/06 存在与实际代码冲突的具体主张，SD-04 仅导航不足以替代 AC-06。全部保留拟议状态，未发布 canonical Specs 或 ledger。

## 下一步

outcome: **needs_fix**；Plan 回 executing；保留 T-01/T-03，重开 T-02/T-04/T-05/T-06/T-07，current_step=2/9（已完成任务计数），T-08/T-09 不勾选。
下一步在同树按 auto-plan-work 修复全部 finding、补完整不变量及定向验收，提交新基线后复审；本次未授权或实施修复/合入。先交付诊断和覆盖调整，不能只调低断言、改报告为边界或把工作验收后置 merge。

重放：从原工作树设置 RUSTC_WRAPPER 为空、CARGO_TARGET_DIR 指向其 experimental/ac-syntax/target，然后 `cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r1/probes/Cargo.toml --test counterexamples -- --nocapture --test-threads=1`。该附件清单使用当前唯一实施树绝对路径；`probes/Cargo.lock` 固定审查依赖。反例期望是正确行为，当前失败为预期记录；修复后应全部通过。
