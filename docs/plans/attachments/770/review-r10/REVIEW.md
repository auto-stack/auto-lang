# PLAN-770 第十轮最终独立复审

2026-10-15；plan_revision=2；review_scope=final；outcome=needs_fix。
受审树 `D:/autostack/.wt/lang-770/auto-lang`，branch `plan-770-dev`，HEAD `7a73cf0b68431073181937e3a6a384ca772101a9`，merge base `6787bbd4a0257067af43845c4450f3644125af6e`。771 `e279ee018f58cc4e9c89f8125e37155f66fb3aad` 为祖先，772/774在基面。入场main `17bc8431457848b040bffa5c261e13b057771559`；相关canonical/774在两树hash一致。当前审查上下文未实施r9返工；未创建额外agent，未修改实现。

冻结输入：plan-input.frozen.md、baseline-hashes.txt、proposed-spec-delta.frozen.md；SD SHA256 `7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`。SD01..05路径、before/after、experimental边界与空touched_goals解释仍有效；资源完成声明必须待本项修复后沉淀。

## P770-R10-01：循环可能不移交时仍撤销caller清理责任（P1 blocking）

影响AC03/04/05与T02/06/07；属于r9修复新增回归。合同V04明示正常loop/return/break/continue路径 `live=0、alloc=free`；774现行data-resource-contract要求每分配恰一个责任方。R9纠正要求已明确混合零迭代/未移交与已移交出口必须核验责任一致性。这不是新增完整borrow checker或动态条件清理要求：已有if-join策略可以拒绝不一致输入。

```auto
fn consume(move s Str) int { return s.len() }
fn main() int {
 let x Str="abc"
 var i int=0
 for i<0 { let n int=consume(x.move); break }
 return 0
}
```

p01：条件实际为false，x已分配，但callee从未取得所有权。p02恒真循环中 `if i==0 { break }` 在move前退出。这两项source/HIR均check0/build0/run0，stdout空，stderr精确为 `LIT len=3\nLIT id=1\n`，**FREE完全缺失**；真实Bundle也获得Checked。正常退出不符合alloc=free。原字节source、canonical直接HIR和真实verify_text Bundle均保留，expected-results.txt给出合法结果/受控拒绝边界。

原因：`experimental/ac-core/src/text_resources.rs:1927..1942` 合并body和目标出口后，只要dead包含入口owner就无条件从frame.st.live删除，丢弃caller后续free；没有核验零次迭代/移交前break与移交后break的责任一致性。注释及§9.21所称“与if-join一致”不成立：同文件1796..1837的if合流逐incoming统计live，混合责任会报verify.ownership-required。单纯dead并集适合拒绝后续读取，不能决定每条运行路径的唯一释放责任。p05（零次循环/return-move体）及c04（return-vs-break）正确保留caller FREE，是终止分流正控，不计为缺陷。

修复要求：区分真实after-loop入口（条件零次/后续失败、目标break等）与return/真实回边；根据所有可达incoming核验entry-owner责任一致性，不能以任一路径move就取消全部路径的free。保留确定单次return/break移交、fresh owner与真实重复move拒绝；把本轮两条漏释放反例及正反控永久化，提交后重绑证据。不能只把新输入记录为债务或修改完成声明缩围。

## 验证与AC映射

同树fresh联合gate **14/14 PASS**，gate-20261015-010244；core **350 passed/1 ignored**（旧执行器长管道案例，非新跳过），runtime **20 passed**，check无warning/fmt clean；record内嵌 **24/24、23 cases**，741/745/767实跑；重复core运行不累计测试数。持久gate-pass.txt/gate-raw，未跑根t/tv/tf/docs_gen。所有CLI重放在gate完成后串行执行，无exe占用失败/门禁重跑。

本轮 **11 source+11 HIR** 与真实BundleAPI：7个正确合法正控（含p05），2个正确拒绝反控，2个漏释放反例。c01确定move/break双腿单次callee FREE；c05 return-only move双腿exit3/单次callee FREE；c02/c03/c04/c06/p05双腿exit0/单次caller FREE。p03真实continue重复move拒use-after-move；p04不一致if责任拒ownership-required。R9-01原例已闭合，R10-01为修复新增回归；不能把旧例闭合等同整个清理不变量闭合。完整check/build/PE trace、输入hash及HEAD见replay-output.txt/results-summary.txt；子进程60s contained deadline，无timeout/panic。

历史R9 **10+10**、R8 **7+7**、R7 **7+9**、R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7**（source+HIR），合计 **78 source+84 HIR** 原字节串行重放，输入SHA256与前轮全部一致。与R9结果比较，唯一check/build/run/trace变化是R9 p03两腿由误拒恢复run3/精确LIT,len,FREE；其他结果保持。见historical-counts.txt、historical-diff.txt与各层replay-fresh-output.txt/results-summary.txt。联合gate同时覆盖身份/真实变异、Unicode/char/fault、marks/顺序/来源、4096与record值语义/隔离；没有因首个发现停止其他合同风险面。HEAD/clean/源配置与SD不漂移，见final-stability.txt。

任务元数据归一：入场表第二个T06实际内容为T07 verification/SD交付；恢复T07及依赖T06标识，保留历史文字，未修改合同/plan_revision。复现顺序：verify-ac-text-770.ps1完成后，replay-api.ps1、replay.ps1及各层historical replay.ps1串行。

| AC | Task/代码 | 判定 |
|---|---|---|
| AC01 | T01/02/05，identity/descriptor/receipt | 受测面pass |
| AC02 | T02/03/04/05，Unicode/char/native | 受测面pass，旧R9首次move恢复 |
| AC03 | T02/03/05，resource plan/runtime | fail，正常退出漏释放 |
| AC04 | T02，真实Bundle前门 | fail，不一致责任获Checked |
| AC05 | T02/04/05，控制流/清理/顺序/来源/深输入 | fail，循环退出义务缺失 |
| AC06 | T06/07，joint gate/SD/hash | partial，修复后最终证据待重绑 |
| AC07 | T08 | fail，一项P1阻断 |
| AC08 | T09 | merge-owned pending |

next：同树work返工T02/T06/T07，保留T01/T03/T04/T05完成，executing4/9；T08/T09未完成。plan_revision仍2。未合入/归档，未发布canonical/ledger；T09实际合入、SD/导航/index、计数/归档、批回归到期判定及guard清理继续pending。
