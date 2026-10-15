# PLAN-782 final R3 — pass

2026-10-15；stage: review；review_scope: final；plan_revision: 1。
Reviewer：本次 Codex `/root` 独立检查上下文，未参与实施；未创建额外 agent；实际模型/会话标识未暴露，不虚构。技能沿用 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` 与 verification-contract.md。

## 冻结基线

- 主计划取自主检出；入口 execution_done=7/9，出口 reviewed=8/9；语义合同 revision 1 未修改。
- 唯一实施树 `D:/autostack/.wt/lang-782/auto-lang`，`plan-782-dev`；reviewed_commit `17508b703c84e6f24496ede7b908621a1c05f365`；base_commit `4568e4ea795996a759b4747fc56b30a30c4308db`。入口/出口 HEAD 一致、工作树 clean。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean；未修改兄弟仓。
- 依赖、源码、测试、gate/runtime/tool 文件指纹：baseline-hashes.tsv；canonical 输入：spec-inputs.tsv，与 R2 七项输入均相同。
- 规范提案冻结副本 proposed-spec-delta.frozen.md，SHA256 `6c5ea9dd9dd3b16310160ffe8a8f4b935983a1570479103aaac052e0698b5daf`。
- 本轮只在 main 写计划簿记、review 附件与债务闭合；未修改实现、canonical、ledger，未合入/归档/清树。

## 两连败诊断检查点与修复审查

已核实 verification.md §5c 的诊断表，不只接受 executor 的“完成”结论：

| 不变式 | 先前遗漏原因 | 代码与本轮证明 |
|---|---|---|
| 嵌套聚合存在标记可组合 | 旧 nested 例全真且首 if return，根本未执行所称第二条 skip；没有内层 take→skip 重入 | native LogicalRhs 执行边转发已有 Agg.present，只有 None 才传1，外层 skip传0；R2两原例+三层永久例+本轮 ctor/clone/call 矩阵通过 |
| 字段写不能复活整体 moved 根 | 只修 replace_frees 调度，没复核根是否还可写 | text_resources 在分派字段终型前检查 place非空且 target在本语句moved或快照dead；三终型×同/跨语句六控制均verify.use-after-move；完整重建和存活替换仍正确 |

修复覆盖不变式而非输入名特判，未增加部分初始化能力，未弱化完整语句尾寿命、统一 LIFO 或旧域隔离。

## 本轮实际复验

所有编译/验证/PE 命令工作目录为实施树，探针使用本轮 freshly-built 工具及同一真实 support lib，子进程60秒截止；证据输入与结果可在本目录直接重放。

1. `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test text_record_hir --test text_record_source --test text_record_resources --test text_record_native`：**18+21+27+24=90 passed，0 failed/ignored**。包含256层/4096循环/0次、Token/直接HIR、真实变异、同异源wrapper、V07五故障控制及本轮四永久回归。
2. `cargo build --manifest-path experimental/ac-core/Cargo.toml --locked`、locked all-target cargo check、cargo fmt -- --check、`git diff --check 4568e4ea7..HEAD`：均exit0，零编译警告。
3. `replay.py`：R1原文14项，results.json/replay-output.txt；三隐式资源复制拒，合法clone通过、move-return重建及fresh对照正确、单层loop无invalid free、混合两向LIFO正确；nested borrow拒。mixed_temp_order原本是Str/str失配负控制，不计作PE成功。
4. `extra.py`/`more.py`：R2八个原文控制；extra-results.json/more-results.json。内层&&/|| take→skip、skip→take、外层skip均正确；partial/nested reown前门verify.use-after-move；live field replacement与whole rebuild的精确值/trace正确。
5. `api-replay.ps1`：真实Bundle9控制，valid CHECKED，call arg/param、ctor arg、FieldId、TypeId、Move.inner、cycle、requires八项受控拒，无panic；api-results.txt。
6. `matrix.py`：**15/15 asserted pass**（matrix-results.json/matrix-output.txt）：
   - ctor/clone/owned call-return三种聚合临时×正常/fail@3/fail@4=9个真实PE。先取后跳的内层短路在后续实参分配失败时，已合流的present=1/0分别正确消费；fail@3仅回滚id2，fail@4第二轮不走陈旧聚合；均无FREE-INVALID。正常exit3、尾部跨类型LIFO和根出口唯一释放已读原trace确认；fatal70保持live根OS回收边界。
   - Str/嵌套资源record/标量三字段×同语句/跨语句move=6个前门拒，均verify.use-after-move。

现有 `f782_r2_01_nested_marker_rides_the_rollback_releaser` 的失败点实际上在外层join前，单独不足证明“已合流标记消费”；本轮矩阵把故障放在完整逻辑实参之后，真实覆盖该路径。这是本次补证边界说明，不将测试名当证明，也未要求新语义。

## R1/R2 finding 关闭

| Finding | 关闭证据 |
|---|---|
| F-782-R1-01 | 资源FieldGet的let/assign/return/ctor拥有形态门、原文拒绝、clone/copyable合法对照；90测试+重放 |
| F-782-R1-02 | 完整根move-return重建与fresh目标，单次最终释放；原文及永久native |
| F-782-R1-03 / F-782-R2-01 | 单层与多层&&/||路径存在性、循环重入、三聚合形态及join后故障；extra+matrix精确trace |
| F-782-R1-04 | pending统一创建序，exit统一消费buffer_frees；双向文本/聚合临时和局部trace正确 |
| F-782-R1-05 | 结构界检先于组合语义；API变异真修改后verify.bad-reference，无panic |
| F-782-R1-06 | V07真实CLI注入到既有runtime符号、部分ctor/clone回滚与旧目标不提交；预算8/7临界、ledger、零长；90测试中全部实际通过 |
| F-782-R2-02 | 根liveness门覆盖所有字段终型与两语句期；matrix六拒+合法完整重建/字段替换 |

上述八项合同内阻断均关闭；本轮无新增阻断，未把未完成合同转移为债务。D-771-A/B/C原登记的前置语法/诊断限制保持原范围，不借本计划宣称修复。

## AC→Task→实现/证据

| AC | Tasks | 结果与证据 |
|---|---|---|
| AC01 | T01/02 | pass：descriptor/HIR/source/CLI身份与writer roundtrip、四旧域隔离；18HIR+profile gate原始收据 |
| AC02 | T02/03/04/05 | pass：类型/构造/资源分类/move/clone/参数返回的source/HIR/native，90测试与R1原文 |
| AC03 | T02/04/05 | pass：借用来源/alias/替换/控制流/真实API变异；resources27、source21、native24、matrix字段六拒 |
| AC04 | T04/05 | pass：唯一/LIFO/临时/故障状态边界；真实V07、三聚合×三执行路径矩阵及历史原文 |
| AC05 | T02/03/05/06 | pass：256深、4096生命周期、0次、畸形门和静态检查本轮重跑；能力与三件发布原始gate证据按下述理由复用 |
| AC06 | T06/07 | pass：新旧联合gate已在代码相同基线真实跑；本轮核验24个输入hash、文档only差异与SD冻结；Token/non-goal边界保持 |
| AC07 | T08 | pass：本次独立验收、Spec与全部历史findings闭合；无实现期自检冒充独立复审 |
| AC08 | T09 | pending merge：合入/沉淀/归档/到期判定/guard清理未执行 |

复用依据：work-gate-20261015-134523.txt 的真实head为 `7e59e06f3e996d519d3d7dbcb21ff6a1b95d85f6`，包含452/0、runtime20/20、旧771/741(-SkipMainGates)/745/767/770、能力/故意失败/SENTINEL。该head至受审17508b703只有verification/delta两文档差异，代码/测试/依赖/配置/fixtures/gate未变；24个原始gate输入hash逐一核验一致（gate-input-recheck.json），support lib与合同未变。770 scope唯一预期失败由782允许集合精确归因，源码已查真实越界仍失败。本轮没有重新跑整套旧联合gate，不声称它是本轮新运行；这份历史证据不能豁免后续merge-owned门禁。

健康：本轮check/fmt/diff检查通过；未见新增产品调试打印。verification.md §2仍含旧V07计划形状索引、§6有历史hash节选；§5b/5c和本轮冻结证据明确取代其完成断言。归档/沉淀时应使用本轮实际hash/实际案例，不能从旧节选复制工具或PE指纹。此为既有材料的使用边界，不是未交付运行行为。

## SD01..06审核

提案目标与metadata准确，touched_goals=[]有限Rust实验profile不关闭GOAL-017/ACC桥有书面理由。SD01独立组合语言支持/拒绝、SD02 D03/04/05/07资源和标量copy适用域、SD03公共HIR/派生资源可信门、SD04有限lowering与回滚边界、SD05工程能力、SD06 Token取证与enum/List/use/主体未交付均与代码及证据一致。本轮只变更附A闭合收据，没有扩大语义合同。canonical七个输入与R2相同，无并发冲突；新增source-text-record.md仍只在提案阶段。批准此冻结提案作为merge沉淀输入；canonical发布应提炼持久规则，不复制附A/B执行日记。

## 结论与pending_closeout_gates

outcome: **pass**（final，代码/验收/规范审查）。status=reviewed，current_step=8/9；T08完成，T09未完成。此结论不是交付完成，不执行合入。

merge在冻结提交与实际共同基线上统一执行本计划适用的experimental/ac-core/runtime/text locked check/test/fmt、新782与旧联合收口gate及必要能力/事务控制，记录真实结果后才能合入。当前无crates/根Schema改动，根cargo t/tv/tt/tb/tu/taa/docs_gen不适用；tf仅主检出单实例批量档到期判定。若merge冲突/代码/依赖/配置/输入改变，受影响证据失效，修复提交后复审受影响面并重跑门禁。其后才可canonical/ledger/index、归档计数、wt-guard clean后清理。

next: auto-plan-merge；本次请求仅检查，未合入、归档或删除工作树。
