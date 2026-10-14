# PLAN-770 第四轮最终独立复审：needs_fix

2026-10-14；stage=review；review_scope=final；plan_revision=2。
本次为用户请求后的独立审查上下文，未参与返工，未创建额外 agent。结论从当前共享 Plan、canonical Specs、受审代码及本次运行重建；未修改实现、合入、归档或发布 Specs/ledger。

## 基线

- 唯一实施树：`D:/autostack/.wt/lang-770/auto-lang`，branch=`plan-770-dev`。
- reviewed_commit=`92c14fe6cbf1761521fb059bf41f667abeacc214`；base_commit=`e849e06f976816dda3db0c68a520d485a69904a7`。
- 主检出入场 HEAD=`882f74f5573e59144ee20c6919d23f8f710ca3a6`；前置 772/774 已在基面。当前 master 与基面在 ac-core、相关 canonical Specs 及旧 gates 无差异；771 尚未合入。
- [baseline.txt](baseline.txt) 记录实际路径/分支/完整 SHA、Plan 与 Spec 输入 hash；[Plan 冻结](plan-input.fresh.frozen.md)、[SD 冻结](proposed-spec-delta.fresh.frozen.md)。SD SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`。
- 入场实施树 clean；所有受审实现已提交。审查只更新主检出的计划簿记与证据。

## 运行与历史闭合

本轮新组合输入为 p01..p11 与 c01..c04，共 **15 source + 15 HIR**；使用既有 745 contained executor，check/build/run 各 60s，完整输出/32 位 exit。见 [重放命令](replay.ps1)、[本次原始输出](replay-fresh-output.txt) 及每例 JSON（包含输入 SHA、受审 HEAD、真实结果）。

真实 Bundle API 腿见 [构造器](emit-probes.rs)、[本次结果](bundle-probes-fresh.txt)：使用当前提交产出的 rlib，`compile_parts_with` 得到未检查 Bundle 后直接调用 `verify_text`，不是用文本 binder 冒充程序化门。[历史 API 重放](api-r3-replay/emit-probes.rs) 与 [结果](api-r3-replay/results-fresh.txt) 另对 q04 Drop、q05 删除 requires、q06 坏引用做实际变异并断言变异数/状态。

上一轮闭合表已核验，具备重新入场材料。R3-01..05 **原例闭合有效**：q01 唯一 FREE、q04 条件 Drop 无 FREE-INVALID、c01 合法读取恢复；q02 同一参数来源拒 mut-conflict、c02 异源合法；q03 mark 先于 FREE；q05 删除声明拒 capability.missing；q06 坏引用受控 verify.bad-reference。旧 r2 原例保持修复后的预期；见 [r3 本次重放](historical-r3/replay-fresh-output.txt)、[r2 本次重放](historical-r2/replay-fresh-output.txt)。不将新组合缺口冒称旧例未修。

完整门禁结果另见本报告末尾“门禁收据”。既有测试总数不能代替下述组合路径的独立断言。

## 当前阻断

### P770-R4-01（P1）：内层 owner 释放后，其外层借用仍获 Checked 并执行

影响 AC-03/04/05，T-02/04/05/06。

[p10](p10-borrowed-scope-escape.at)：

```auto
fn main() int {
    var v str=""
    if true { let x Str="abc"; v=x.to(str) }
    return v.len()
}
```

source/HIR check=0、build=0，程序化 Bundle 亦 Checked。两腿真实 PE exit=70，stderr=`LIT len=3\nLIT id=1\nFREE id=1\nRT-FAIL code=-7\n`。这是先释放后使用的非法借用，必须在前门拒绝（owned-scope-escape/borrow-escape），不能由运行时扫描已释放字节碰巧报 invalid UTF-8 来兜底；不声称所有机器上都必为 -7。

`src/text_resources.rs:1424` 块结束只安排 FREE，返回状态却不标记该 owner 已离开作用域；`:1493` join 对全无 live 的 owner 删除 parent.live。随后 `check_reads` (`:787`) 只有 backing owner 仍在 live 中时才检查词法作用域；dead 中也没有其块出口死亡原因，因而 provenance 虽保留，寿命校验失效。

修复要求：块出口生命周期必须进入状态，borrow backing 校验不能将“非 live 且非 dead”当合法；兼顾静态字面量、存活 owner、正常 fall-through、条件/循环出口、move/Drop 的区分。合法 [c04](c04-live-owner-borrow.at) 两腿 exit=3、一次 FREE，必须保留。

### P770-R4-02（P2）：短路 RHS 临时拥有值的语句尾清理使用不支配 join 的值

影响 AC-02/03/05，T-05/06/07。

[p01](p01-skipped-rhs-temp.at) 为 `false && ("a"+"b").len()>0`；p02 改为 true，p03 为动态 lhs，p04 两侧均有临时，p11 为 `true || ...`。五例 source/HIR check=0、Bundle Checked，但两腿 build=1，确切诊断 `backend/backend.lowering: define d_main: Compilation error: Verifier errors`；合法 PE 无法产生。

`src/native.rs:1715` 统一在语句尾清空 `pending_temps`，而 `:2359..2400` LogicalLhs/Rhs 仍生成跳过 RHS 的 CFG；RHS 中定义的临时指针直接加入全局 pending 列表，join 后 FREE 用它，skip 前驱没有该 SSA 定义。恢复语句尾寿命修复引入这一组合回归。

修复要求：语句尾清理记录须按实际执行路径传递/合流，跳过路径无分配也无 FREE，执行路径一次分配/一次释放；保持完整语句后续 mark 先于 FREE 与 4096 链探测帧。不得以提前释放或静默禁用合法短路替代。借用字面量 RHS [c01](c01-borrowed-shortcircuit.at) 两腿 exit=0，隔离“仅有拥有临时”这一触发因素。

### P770-R4-03（P2）：提前返回臂仍污染后续 dead/backing 状态

影响 AC-03/04/05，T-02/04/06。

[p05](p05-returning-move-arm.at)：`if i>0 { return consume(x.move) }; return x.len()`，i=0。返回臂终止，无后续读取路径；合法控制应 build/run=0/3，并唯一 FREE。source/HIR 与 Bundle API 却均拒 `verify.use-after-move`。嵌套返回 [p06](p06-nested-returning-move-arm.at) 同形误拒。

`src/text_resources.rs:1474..1480` 的 merge_children 只按 reachable 选择臂，即使其已经 return 仍合并 dead/backing；后面的 incoming 才筛 then_incoming/else_incoming，因而只有清理责任过滤了终止臂，读取状态没有过滤。

修复要求：所有 continuation 状态按真正进入 join 的出口合并，终止臂在该 join 不贡献 live/dead/backing；另保留其终止出口释放/转移。覆盖嵌套 return 与 break/continue，而非只为此处删一项 dead。未执行臂仍进行静态类型合法性检查。

### P770-R4-04（P2）：分支自己的 owned 局部被误当成跨臂责任

影响 AC-02/03/04/05，T-02/04/06。

[p08](p08-branch-local-owned.at)：动态 `if i==0 { let s Str="abc"; let n int=s.len() }; return 0`；[p09](p09-both-branch-locals.at) 两臂各声明独立 owned 局部。均是合法块作用域构造/读取/清理，source/HIR/Bundle API 却拒 `verify.ownership-required`，p09 报两枚。

`src/text_resources.rs:1493` 遍历函数全部 self.owned 比较每个入边的 live_cnt，包含只在一臂声明的局部；另一臂从未初始化它不等于清理责任不一致。加之块出口的状态未清除 own_lets，误把其当作需要 join 的 parent 责任。

修复要求：join 区分入口 owner 与子块局部 owner，子块局部在其作用域出口收口，不把未构造路径算成未履行 move。动态单臂/双臂合法都须真实 PE 与唯一 FREE；静态 true [c03](c03-static-branch-local.at) 已成功（exit0，一次 FREE），不能只依赖此控制。

### P770-R4-05（P2）：包装函数的两个 view 参数可在内部重新形成同源 mut/view

影响 AC-03/04，T-02/04/06。

[p07](p07-wrapper-alias.at)：`both(mut a str,b str)`；`relay(a str,b str) { return both(a,b) }`；main 将同一 x 的两个 `.to(str)` 传给 relay。source/HIR check/build=0，Bundle Checked，PE exit6；实际同一来源同时进入 mut/view，违反 774 D-03/04 整个函数体排他区域。首批没有 mutation 方法也不能免除明确的前门契约。

`src/text_resources.rs:697` 将不同参数局部号视作不同来源；`:978..1010` 仅按直接 callee 的 mode 检查，没有传播 relay 内部对参数对的排他约束，因此包装一层即绕过。不能将两个形式参数身份不同等同于调用者提供了不同 backing。

修复要求：将内部借用使用约束可靠传到实际调用边界，或采用符合既有支持合同的受控拒绝；不得把所有只读双参数调用一概禁用。独立来源 [c02](c02-wrapper-independent.at) 两腿 exit4、FREE2/FREE1，必须保留。旧 q02 是同一个参数重复读取，原例已修；本例是两个形参由 caller 传入同一来源，不是新增 full borrow checker 范围承诺。

## AC → Task → 实现 → 证据

| AC | Task | 实现/证据 | 裁决 |
|---|---|---|---|
| AC-01 | T01/02/05 | descriptor/verify 精确身份、q05 派生清理需求、完整旧 gate | 受测面 pass |
| AC-02 | T02/03/04/05 | text_hir/source/native 与 runtime 值语义；p01..04/p08/09/p11 合法输入失败 | partial |
| AC-03 | T02/03/04/05 | resource plan/runtime 计数；p10 UAF、p07 排他绕过，及合法路径误拒 | fail |
| AC-04 | T02/04/05 | 真实 Bundle API、变异 q04/05/06；p10/p07 非法 Checked、p05/06/p08/09 合法误拒 | fail |
| AC-05 | T04/05 | marks/4096/原 span；新短路临时后端失败与终止/词法作用域缺口 | fail |
| AC-06 | T06/07 | 新旧完整 gates/hash、scope、SD；总数不能证明五项完整性 | partial |
| AC-07 | T08 | 本次独立审查 | fail（needs_fix） |
| AC-08 | T09 | 实际 merge/沉淀/归档/批回归到期/guard | merge-owned pending |

## 规范与遗漏审计

SD-01..05 路径、before（772 后 canonical）、len_bytes、COFF marks、实验性/非 ACC Gen1 边界、touched_goals 空集解释有效。冻结 SD 与上一轮 hash 一致；未修改语义合同，因此 plan_revision 仍为 2。但资源前门与 native 已完成的声明尚不能作为事实沉淀，待上述不变量闭合并 fresh review。未发布 canonical/live ledger。

以上均为既有合同内缺口；没有把失败仅登记为非阻断债务，也没有授权移到 merge。五项已同步 KNOWN-DEBT-AND-RISKS；下一轮 work 需解释为何永久 oracle 仍漏组合路径、明确状态设计修正、重放旧例/本轮正反控并将关键组合永久化。

状态回 `executing`，保留 T01/T03，重开 T02/T04/T05/T06/T07，current_step=2/9。next=同一树 work 修复并提交；完整门禁/闭合表更新后再次独立 final review。T09 合入、知识沉淀、ledger/index、归档/计数、批回归到期判定与 guard 清理仍 pending，不得消费本次 non-pass 合入。

## 门禁收据

独立执行 `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-770.ps1`：最终 **13/13 PASS**，[完整收据](gate-pass.txt)（gate-20261014-162529.txt）、[原文](gate-raw/)。ac-core **227 passed / 1 ignored**、runtime **20 passed**；两个 workspace check 零警告、fmt clean；741/745/767 实际通过，scope/hash 步骤通过。既有 ignored 为单独手动运行的 60s executor 例，不计入通过。本轮未运行根 t/tv/tf/docs_gen（无产品 crates 或根配置改动）。

首次门禁 [原始失败](gate-first-failed.txt)（gate-20261014-162011.txt）在 `source_native::positive_corpus_executes_natively` 报 `d02-paren-multiline-then-add.at: receipt missing`，本次没有把它改写为绿。零代码变化下完整复跑成功，前几轮也有同类收据/obj 窗口；本轮只能确认为瞬态文件现象，未做 AV/进程竞争根因证明。它不能解释本报告五类可重复语义/后端缺口。

结束实施 HEAD 仍为 `92c14fe6cbf1761521fb059bf41f667abeacc214`、实施树 clean；Spec 输入、SD、受审实现没有漂移。全新 r4 双腿结果、历史 r2/r3 双腿与程序化 API 均已刷新。结果摘要见 [results-summary.txt](results-summary.txt)，原始记录保留确切输入 SHA 和运行输出。
