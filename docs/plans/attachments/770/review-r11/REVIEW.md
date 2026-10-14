# PLAN-770 第十一轮最终独立复审

2026-10-15；plan_revision=2；review_scope=final；outcome=**pass**；findings=无新增阻断项；Plan reviewed（8/9）。
当前审查上下文未实施r10返工，未创建额外agent，按实际合同、代码和fresh执行重建结论；实现未修改。

受审树 `D:/autostack/.wt/lang-770/auto-lang`，branch `plan-770-dev`，HEAD `afe45e2d65332b0cd448f3a2b5edf562903296b6`，代码提交 `9033cdb39`；base `6787bbd4a0257067af43845c4450f3644125af6e`。771 `e279ee018f58cc4e9c89f8125e37155f66fb3aad`为祖先，772/774在基面；main入场 `92998d19a73a2caba7702fc42034afc229b3cbec`，相关canonical/774与受审树hash一致。冻结Plan、相关输入、SD与制品指纹见本目录plan-input.frozen.md、baseline-hashes.txt和final-stability.txt。

## 修复及高风险不变量

**P770-R10-01已闭合**：原字节p01零次迭代、p02移交前break，源码/HIR检查和构建均报 `verify.ownership-required`，无PE。真实Bundle同样受控拒绝。永久text_review_r10覆盖全部11输入，合法运行要求精确trace及alloc=free；未以没有FREE-INVALID代替释放配对。

`src/text_resources.rs:1944..1989` 从真实自然完成和目标break构造after-loop incoming，逐entry-owner计数：全live保留caller义务，全dead撤销caller义务，混合拒绝；return不进入after-loop。包装for的条件假分支带入零次迭代live状态。清理责任与保守读取dead/backing状态分别决定，修复范围与已有if-join责任规则一致。

独立新增组合：7个合法source/HIR双腿、2个混合责任拒绝双腿，另2个直接HIR Drop输入，合计 **9 source+11 HIR**。嵌套确定单次move、双臂move后break均PE0/callee唯一FREE；两个owner中只转移一个，PE3/两id各唯一FREE；静态false循环、无移交自然循环、嵌套return/break之后读owner均PE3/caller唯一FREE；fresh owner两轮移交PE6/两id各唯一FREE。嵌套/外层混合break责任受控拒；嵌套确定Drop PE0/唯一FREE，混合Drop拒。精确退出码/stdout/stderr断言通过，见assert-replays.ps1、assertion-results.txt、replay-output.txt、results-summary.txt、drop-probes。

Drop构造第一次替换移交语句时遗留未使用effect/read表达式，真实validator正确以eval-position拒绝；删除遗留表达式后才作为有效Drop正反控。两份初始仪器输出独立保存，未计为产品缺陷或最终有效Drop结果。

历史 **89 source+95 HIR=184** 原字节输入串行重放，SHA256逐项与R10对照一致，未重写旧反例。唯一四项行为变化是R10 p01/p02的source/HIR两腿从漏释放成功恢复责任不一致拒绝；其余check/build/PE退出与trace不变。历史原R9首次move恢复、新鲜owner、真实循环回边陈旧view与重复move拒绝、恒定终止、短路临时与scope/exclusivity控制全部保持；R2..R10受测阻断集闭合。无timeout/仪器Error，见historical-input-hashes.txt、historical-counts.txt、historical-diff.txt及各层完整输出。

r9内部组合测试由动态wrapper改为恒真wrapper已核对：动态wrapper仍可能零次执行，按R10既有责任一致性要求属于混合路径；恒真wrapper才是确定单次转移控制。原始历史输入没有被改写以制造闭合。

## 验证与规范增量

定向check无warning；text_review_r9+r10 **22/22**通过。fresh最终联合gate **14/14 PASS**，gate-20261015-014555；core **361 passed/1 ignored**、runtime **20 passed**，两workspace check无warning/fmt clean；741/745/767实跑，record gate **24/24、23 cases**。重复core执行不累计新测试数。完整gate-pass.txt/gate-raw已持久化；先完成代码/验收/SD审计及串行重放，再启动gate。无门禁重跑/可执行文件占用冲突。

SD冻结SHA256 `7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`；SD01..05逐项核对路径/before/after与当前Core/Scalar/Record canonical：增加实验性source-text/HIR/阶段/native/runtime及有限ACC取证，无生产ABI、ACC Gen1、完整lexer/token或text-record组合能力宣称。supersedes四项project/stage-contract、new source-text、空touched_goals的GOAL017不关闭解释一致。现行data-resource-contract仍约束每分配恰一个责任方；没有批准缩围或未处置合同内延后项。两个no_std staticlib共链限制/预算注入归runtime/越界码边界沿用已裁定记录，不阻断本profile。

| AC | Task/实现 | 证据与判定 |
|---|---|---|
| AC01 | T01/02/05；descriptor/hir/verify/main | 身份/类型/真实Bundle变异、capability/sidecar、旧域隔离；**pass** |
| AC02 | T02/03/04/05；source/text/native/runtime | Unicode标量/字节/NUL/char/方法与真实PE、明确拒绝；**pass** |
| AC03 | T02/03/05；TextResourcePlan/native/runtime | R10漏洞闭合、正常退出唯一FREE、runtime fault/budget/trap；**pass** |
| AC04 | T02；verify/资源前门 | 真实Bundle拒绝与合法兄弟、无panic，混合责任受控拒绝；**pass** |
| AC05 | T02/04/05；控制流/顺序/来源/深输入 | 原字节历史、组合正反控、4096/marks/短路/静态span；**pass** |
| AC06 | T06/07；joint gate/SD/hash | 冻结输入、fresh旧profiles与record集成、SENTINEL与scope；**pass** |
| AC07 | T08 | 合同/SD/遗漏债务独立审计完成，**pass** |
| AC08 | T09 | merge-owned pending |

T09实际合入、SD/导航/ledger/index、归档/计数、批回归到期判定、guard清理仍pending。根cargo t/tv/tf/docs_gen不适用此experimental-only改动；没有运行。受审基线或相关依赖/配置改变时须重跑受影响验收并重新绑定review，不能用本次凭证覆盖未来实现。

next：merge（T09）；状态reviewed，8/9。实现HEAD/clean/SD/源配置结束核验无漂移；main在审查中推进至ad0ee41d216271c72243fa4010f0c4295cb3fa29，相关Plan和canonical未漂移。只写main复审簿记；未合入/归档/清理/发布canonical或ledger。
