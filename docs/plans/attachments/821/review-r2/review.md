# PLAN-821 final R2 — needs_fix（修复就绪性定向审查）

2026-10-25；r5；受审 `e34a2c99940c0fec8558deffa27447a067a7553a`，同树 `D:/autostack/.wt/lang-821/auto-lang` / `plan-821-dev`；上一受审 `7c72d657dd16a0865216258684645f342550a808`，修复提交 e8e1e369d/e34a2c999。当前独立审查聊天延续 R1，未参加实现。入审/出审实施 clean，无实现改动。

**不能认定“11/11 全关闭”。** 交接与 SD 自认 mut record/V05 尚待和解，源码还直接保留 whole frontend；按 review 技能 readiness 条款，本轮先核已知合同内缺口及受影响反例，不启动完整最终验收/统一回归。此为第二次 final 提交的 needs_fix，不以“就绪性”命名重置轮数。

绑定：[baseline.json](baseline.json)、[Plan 入审快照](plan-r5-at-entry.md)、[SD 冻结件](spec-delta-frozen.md)。原 R1 Spec/依赖未因这两个修复提交变化，既有不受影响检查保留；新 RecordView/owned_session 不能套用旧 pass。

## 实际验证

- 在实施树执行 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_r1_replay --test av_owned_821_incremental`：**7+5=12 passed，0 ignored**。伴随4条新 unused_variables 警告（下面 health finding）。
- 不改原 R1 probe，直接重放 `cargo run --quiet --manifest-path docs/plans/attachments/821/review-r1/probe/Cargo.toml`：[raw](r1-probe-replay.txt)。Dropped/ended-local 返回、MutRef→View、Call view roots、StrToOwned 目的地反例现在正确拒绝；合法结束MutRef后view仍得3；跨unit static view=3；Aggregate显式Drop释放序[A,B]；8192槽实际峰值1,376,762B，记账1,597,504B，旧35MB/4160反例修复。
- 对新会话的真实入口与新clone深度分支跑独立 holdout：`cargo run --quiet --manifest-path docs/plans/attachments/821/review-r2/probe/Cargo.toml --target-dir D:/autostack/.wt/lang-821/auto-lang/experimental/ac-av-probe/target`。[源码](probe/src/main.rs) / [raw](probe-output.txt)。无修改产品/测试源码。
- 未跑完整acceptance、裸cargo t/触面整档、tf、性能矩阵、完整资源与恢复族。已确认必修残项时不消耗完整最终review；本轮未重核的F10及其新进程边界保持pending，不能写成独立关闭。

## 原finding闭环表

| 原 ID | 本轮结论 | 实际修复/仍缺内容 |
|---|---|---|
| F821-R1-01 | CLOSED（原输入与相邻合法对照） | Return dropped/provenance修复；原probe正确拒绝局部view逃逸，合法原对照保留 |
| F821-R1-02 | 原反例CLOSED；完整新增RecordView不变量 pending | 原三个loan反例翻转、合法结束loan对照绿；本轮不以7例宣称全部新增borrow模式/跨Call路径验收通过 |
| F821-R1-03 | CLOSED | 原单元常量域冲突例现在Success(3)，已解析字节随静态view传播 |
| F821-R1-04 | CLOSED（原输入/提交点检查） | 原Aggregate+显式Drop序[A,B]，FieldSet/Aggregate成功commit后kill逻辑已加入；失败提交前保源形状保留 |
| F821-R1-05 | CLOSED（原计量缺口） | 稀疏roots+真实布局bound，8192槽实测1.37MB<bound1.6MB |
| F821-R1-06 | PARTIAL/OPEN | 字节计步改善；另以未经批准的128层运行期拒绝替代显式有界遍历，合法typed fixture被错误拒绝，见下 |
| F821-R1-07 | OPEN，新增修复回归 | 每编辑仍整模块前端，checker漏前端/身份；候选lower失败破坏旧会话，见下 |
| F821-R1-08 | PARTIAL/OPEN | borrowed record正例落地；mut record仍lower.mut-record拒，测试把缺功能当期望；未有授权缩减合同 |
| F821-R1-09 | PARTIAL/OPEN（代码审查） | 父侧ChildPeakSampler与128MiB比较已添加；32MiB分区用磁盘目录字节代证，反控未调用同门，完整要求未闭 |
| F821-R1-10 | pending 独立修复复核 | 看到新增restore-worker和自洽坏内层测试；本轮因已知未完成项未运行其完整针对性复核，不宣称通过 |
| F821-R1-11 | PARTIAL/OPEN | SD07有回填，但写入仍whole前端现状、mut列非目标且同时待授权，不能发布为已验收规范 |

## 必须继续修复的最小包

### F821-R1-06 / P1 / AC03/05 / T03/04：合法typed输入被新深度拒绝

R1 要求显式栈、有界计量，不能用源码前门128层作为所有MIR类型图的隐含限制。修复在 `core/src/av.rs:647` 加 `depth > MAX_FRONTEND_NESTING`，但MIR/verifier/layout根本没有这个类型图准入规则；源码语法深度与命名record链深度不是同一量。

独立 `clone_depth_probe` 构造130个单字段命名record，底层1B Str，131类槽、单块、非递归图，**wire仅5619B**；真实encode/decode/ABC授信全过，默认预算执行Clone实际 `Infra("clone depth exceeds the frozen nesting cap (admission violation)")`。现有small/large字节测试无法识别该回归（1MiB样本还可能在Const耗fuel后就拒，不能单凭该例证明Clone路径完成）。

关闭：按原合同对实际类型图/值遍历实行有界显式栈及预算，不在work增加深度语言排除；上述有效小输入须正确执行，超预算明确Resource，合法小值/故障清理对照保持。重放新深度分支及旧字节/事务例即可，不附加全档。

### F821-R1-07 / P1 / AC09 / T09：全前端仍在，失败安装破坏旧快照

**旧不变量未闭**：`probe/src/owned_session.rs:154–156` 每次edit仍调用 `compile_text_record_source(src,...)` 对整个module编译；只对完成后的HIR Debug文本去span/hash、筛选重lower函数。没有D828驻留CheckedEnv单函数parse/recheck/emit通道，也没保留未改HIR生产对象。把测试包装挪入生产文件、从每函数重编减少到每修订一次整编，不等于全链路函数增量。

独立运行：warmup `frontends=1`，只改main `n:7→n:8` 后 `frontends=2`，`incremental_gate=Ok(())`，helper code不动、结果8。checker只比较Production(lower/encode/admit)计数，漏掉实际整前端；gate接口也不读取HIR/code对象/代际/实际执行载荷与IO，注释声称“object swap without mint bump REDDENS”没有对应检查。

**修复新回归（沿原F07保持ID）**：`edit_batch:234` 前端成功后立即修改revision/src，`:253–265` 逐项移除/更新live residents，然后才可能lower失败；未build-aside。独立从合法 `BASE`（结果7）把source候选改为前端合法但owned不支持的mut调用：

```text
expected_revision=1 actual_revision=2
edit_result=Err("lower d_main: lower.mut-record")
production_same=false
after_rejected_lower: panic snapshot import `fn:d_read` does not resolve
```

这是合法的“后端不支持候选应保留旧版”失败对照，**不是要求此输入先实现mut才能测回滚**。现有测试只拒frontend不通过的中间形，所以没走已改live状态的分支。

关闭：复用已有D828单函数前端/私有凭证/精确依赖及真实观测；结构门覆盖整个真实生产与执行载荷，不能以hash代身份。prepare候选全部lower/verify/admit/link成功（及约定执行检查）后原子发布，所有后门失败保source/revision/production ownership/code集合与旧输出；重放本输入、合法main-only、受检link/资源/取消失败对照。暂不跑无关scalar战役或总acceptance。

### F821-R1-08 / P1 / AC01–04 / T03/06：mut源码缺口自认未完成

新增永久测试末段明确期望原 `MUT_RECORD` 被 `lower.mut-record` 拒绝；`hir_adapter.rs:923/:1694` 仍精确拒写回。只是把“land with T03”换为“beyond frozen expressiveness”，没有兑现原支持面。

Plan§9和SD仍自认待授权(a)补写回(b)缩范围，revision仍5，没有批准缩减。review不能代用户选择(b)，也不能将原finding“关闭”再留一个无ID待裁定。回work按原合同补齐；如架构/语义方案确需修订，回new明确修订，缩支持须实际获授权。保持borrowed与move正控；mut写后caller可见须有永久成功例。

### F821-R1-09 / P1 / AC05/06 / T02/04/06：受管分区指标仍错配

新增 `av_owned_821_r1_worker_memory_hard_gate_parent_sampled` 已有parent采样和进程128MiB比较，这部分修复有效。但 `resources.rs:827+` 的 `dir_size(store)` 是磁盘占用，不能证明hot8/staging16/execution8受管内存分区；8个很小的源码版本也没有覆盖新增analysis/session驻留状态。所谓“同门零阈值必红”实为末尾独立 `assert!(max_peak>0)`/`assert!(store_bytes>0)`，未把零阈值传入同一判定入口/验证失败传播。

关闭：真实受管内存计数按适用分区核、parent峰值保留，预算文本与现行依赖差异按原F09解决；真实同checker预算反控应可观测失败。已有parent采样机制保留，不要求反复全性能矩阵。

### F821-R1-11 / P2 / AC07 / T06：SD/完成记录仍与合同和残项冲突

当前SD04把mut列“超出冻结论域”且SD末又“待用户裁定”，SD07将每修订whole frontend描述成已回填增量交付。Plan§0仍当前executing1/9而frontmatter为execution_done7/9，任务行又同时“R1当前重开”和“修复完成”；旧记录可以保留历史，当前状态必须只有一套。

关闭：先解决实质残项再冻结精确SD，不以SD改写原验收；由当前Spec版本/hash和实际通过证据导出before/after规则。V05原必需矩阵仍pending reconciliation，不能以11项关闭口径掩去；下一次ready前必须明确处置。

### F821-R2-01 / P2 / AC07（仓库health）/ T06：修复引入4条编译警告

本轮12测试输出：`core/src/av.rs:680/681` 的v/func_idx未用，`probe/tests/av_owned_821_r1_replay.rs:96/152` 的t/zero未用。原R1 scoped check干净；当前不满足仓规“zero unhandled compiler warnings”。关闭只需处理这些真实未用项、受影响package/test的check/compile无警告；不要求全档测试或改变业务行为。

## 当前AC、任务、下一次handoff

AC01/02/03/04/05/06仍partial，AC09 fail，AC07 fail，AC08 merge-owned pending；本轮不是其他未核项的隐含pass。保留T01、T05已完成工作；重开T02/03/04/06/T09，**executing 2/9**。T05的新恢复实现独立复核pending，未发现新的失败不虚构重开其实现。

这是第二次final needs_fix。work下次ready前须提交简短覆盖诊断：为什么“测试移入生产文件”仍漏全前端、为什么只测frontend失败漏lower/link失败、为什么常量字节测试漏clone深度路径、为什么受控拒被算功能关闭；写清设计/oracle调整与实际新结果，再交新冻结SHA/closure表。不能再以聚合总数或11/11标签代替剩余不变量。

最终pass后才由merge统一跑裸cargo t+tv及实际触面/独立workspace约定档；本轮未改实现、canonical Spec/ledger、未合入/归档/清树。
