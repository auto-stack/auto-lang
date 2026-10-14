# PLAN-770 第三轮最终独立复审：needs_fix

2026-10-14；stage=review；review_scope=final；plan_revision=2。
用户请求后的独立审查上下文，未参与该次实施，从合同、代码和独立重放重建裁决；未创建额外 agent。未修改实施代码、合入、归档或发布 canonical Specs/live ledger。

## 基线与门禁

- worktree=`D:/autostack/.wt/lang-770/auto-lang`，branch=`plan-770-dev`。
- reviewed_commit=`41291aa10d22eb2e9a263e7aa574361fb55eafd1`；base_commit=`e849e06f976816dda3db0c68a520d485a69904a7`。
- 入场/结束实施树 clean，HEAD 未变。主检出 `eb9ae764fd3b079b6809e979f56437a089f0abe4` 相对基面对本计划 ac-core/相关 Specs 零变化；772/774 已合入，774 canonical 沉淀提交 `930d05fae798917ce2ab3026bcfa8d0ab81713d5`。
- [baseline.json](baseline.json) 绑定依赖、Spec、代码/配置/hash；[Plan 输入](plan-input.frozen.md)；[SD 冻结](proposed-spec-delta.frozen.md)，SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`。
- 本轮独立执行 `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-770.ps1`：**13/13 PASS**，原收据 gate-20261014-145312.txt；[持久副本](gate-pass.txt)、[原始输出](gate-raw/)。ac-core **218 passed / 1 ignored**，runtime **20 passed**；两 workspace check 零警告、fmt clean；741/745/767 实跑通过。既有 ignored 为手动 60s executor 用例，未计为通过。
- 无产品 crates/、根测试配置或 ACC binding 修改，未跑根 t/tv/tf/docs_gen。771 尚未合入，不运行组合 profile gate。

## 历史修复与诊断调整核验

两次 final needs_fix 后的 §9.7 已提供漏测原因、修复位置与永久 oracle，满足重新入场所需诊断材料；但完整不变量仍未闭合。

本轮重放原 r2 **17 source + 19 HIR** 冻结输入，输入字节未改：见 [历史结果](historical-results.json)、[原例与命令](historical-r2/)。8 正控制双腿仍合法；p01/p02/p07/p08/p09 双腿恢复预期行为，p03/p04/p05/p06 双腿受控拒绝；mut01/02 受控拒绝。完整门禁实际运行新增 18 项永久重放测试；QA-01 错域控制通过，QA-08 锁文件已受版本管理。F-01 marks 双腿持续通过。

因此旧例 closure 成立；不把新组合反例冒称旧输入仍失败。QA-03 完整控制流清理、QA-05 完整语句寿命、QA-06 五类来源排他、QA-02 实际清理需求的 full-invariant closure 尚不成立，且需求扫描新增非法引用 panic。

## 当前阻断

### P770-R3-01（P1）：分支合流清理仍不完整，含漏释放、重复释放与合法输入误拒

影响 AC-03/04/05，T-02/04/05/06。

- [q01](q01-nested-skipped-move.at)：owned x 外层 `if true` 内有 `if false { consume(x.move) }`，随后 `return 0`。source/HIR check/build=0，PE=0，但仅 `LIT len=3\nLIT id=1\n`，**没有 FREE**。未执行的内层 move 被 `block_definite_moves` 的语法子树并集算成外层必移交，撤销 caller 的清理责任。
- [q04](q04-conditional-drop.atom)：真实 Bundle 中保留合法 `x.len()` let，向 `if true` 的 block 追加 `Drop(x)`（[构造器](emit-probes.rs) assert changed=1）。verify 返回 Checked，HIR build=0/PE=0，trace 为 `LIT...TEXT-SCALARS n=3\nFREE id=1\nFREE-INVALID\n`。`merge_children` 合并 dead 却保留 parent.live，Drop 后仍安排 parent 出口清理。
- [c01](c01-skipped-move-read.at)：`if false { consume(x.move) }; return x.len()` 是合法控制，source/HIR 均被 `verify.use-after-move` 误拒；常量不可达臂仍合入 dead。

代码：`src/text_resources.rs:528`、If join/AwaitIf、`:1490`。三例分别证明不能只补一层常量判断或只扫描 move 节点。需由可达正常/终止出口的实际责任状态计算 join，统一 move/Drop，保留不可达代码的静态类型合法性但不污染运行期责任。覆盖嵌套、return/break/continue、正常 fall-through 和合法未执行臂后读取；source/HIR 分腿并精确计数。

### P770-R3-02（P2）：普通借入参数的同源 mut/view 排他漏检

影响 AC-03/04，T-02/04/06。

[q02](q02-view-param-alias.at)：`relay(s str) { return both(s,s) }`，both 接受 `mut a str,b str`。source/HIR check/build=0，PE=6；同一只读借入来源同时作为 mut/view 被放行。774 D-04 要求五类来源均可追踪、mut 活跃期无只读 view；“首批无 mutation 方法”不能豁免该前门。

`owned_reach` (`src/text_resources.rs:662`) 只纳入 owned/mut 参数与 owned backing；普通 view 参数没有来源身份，两个实参 reach 均空。[c02](c02-view-param-independent.at) 两个不同借入来源仍合法，双腿 PE=4、LIFO FREE2/FREE1。需追踪借入参数身份及派生视图，拒绝确定同源；不靠把所有独立 view 参数一概拒绝完成修复。

### P770-R3-03（P2）：临时拥有值仍提前释放，偏离完整语句尾合同

影响 AC-03/05、SD，T-05/06/07。

[q03](q03-temp-statement-tail.at)：`return ("a"+"b").len()+mark_a()`，source/HIR PE=3，trace `CONCAT...TEXT-SCALARS n=2\nFREE id=1\na`。冻结 774 D-04 和 design.md §4.4 要求临时存活至完整语句尾，因此 mark 的 `a` 必须先于 FREE。当前 `native.rs:2479` 在 Len 消费后调用 `free_backings` (`:1706`)，`pending_temps` 仅 push、未接语句尾释放。旧 p07 消费前有效已修复，但 full-statement lifetime 尚未实现。

§9.7 以 4096 链 guard-page 为由放弃语句尾模型没有修改已批准合同；这是合同内缺口，不接受默改 expected/SD 为早释放。需实现合规寿命及 4096 bounded frame，或先走经用户批准的合同修订；保持冻结 trace oracle。

### P770-R3-04（P1）：派生运行时需求遗漏纯清理义务，仍可绕过库合同

影响 AC-01/04/06，T-02/05/06。

[q05](q05-cleanup-no-requires.atom)：模块含 `unused(move s Str) int { return 0 }` 与 main=0；程序化 Bundle 确认 requires.clear。真实 verify 返回 Checked、requires_runtime_text=false、capabilities_check([])=Ok。HIR build 未提供 capability，使用**无 sidecar 的 runtime 库副本**仍 build=0/PE=0（[JSON](q05-cleanup-no-requires.hir.json)）。不声称 main 实际执行 unused；问题是已发射函数的清理导入仍必须受门保护。

`body_needs_text_runtime` (`text_resources.rs:60`) 只查表达式，漏 owned move 参数/exit_frees；native 的 `scan_body_rt_needs` (`native.rs:1301`) 却按 resource plan 导入 `ac_rt_own_free`。需统一真实运行时需求（操作、调用返回与清理义务）并接 declaration/provision/sidecar 三门。既有 mut02 只覆盖 len，不能证明该面。

### P770-R3-05（P1）：新需求扫描先于结构验证，程序化非法引用 panic

影响 AC-04，T-02/06。

[构造器](emit-probes.rs) 从合法文本 Eq Bundle 定向将唯一 Binary.lhs 变为 u32::MAX，assert changed=1，再 catch_unwind 调 `verify_text`。结果 **PANIC=true**，`text_resources.rs:77:33`：len=5、index=4294967295（[原始输出](bundle-probes.txt)）。`verify.rs:143` 在 `verify_domain` 检查前调用新扫描且直接索引，破坏 AC-04 的 TypeId/引用越界受控拒绝。此为 r2 修复新增回归。先完成结构/引用合法性检查再派生需求，或全程安全访问并保留正确诊断；永久断言 Err 而非 panic，覆盖 Binary/text operand/callee 相关坏引用。

## 验收映射与规范审计

| AC → Task | evidence | 裁决 |
|---|---|---|
| AC-01 → T02/T05 | 域三维/旧 profiles 门禁与 QA01绿；q05 | partial |
| AC-02 → T02/T03/T04/T05 | Unicode/char/source/HIR/native/runtime 全 suite，历史正控制 | pass（当前值语义受测面） |
| AC-03 → T02/T03/T05 | runtime20绿、历史寿命例恢复；q01/q02/q03/q04 | fail |
| AC-04 → T02/T04 | Bundle门与历史变异恢复；q02/q05/q06 | fail |
| AC-05 → T04/T05 | marks/静态span/4096链/loop fresh原测试绿；q01/c01/q03 | partial |
| AC-06 → T06/T07 | fresh13/13/hash/lock/scope齐备；运行时信任门与SD寿命未闭合 | partial |
| AC-07 → T08 | 本轮 independent review+遗漏/债务审计 | fail |
| AC-08 → T09 | merge-owned | pending |

SD-01..05 目标路径/现行 before/experimental 边界有效；len_bytes/marks.obj/PLAN772 来源修正通过，touched_goals 空解释有效。SD-01 的资源寿命与“排他前门已实现”、SD-02/03 的验证/清理完成声明必须等当前缺口关闭才能沉淀。冻结 SD，不发布 canonical。

遗漏/workaround：所有上述发现均为原合同内阻断，未批准延后；特别是以消费者即释放规避栈帧风险不构成合同修订。非目标 List/record/生产ABI/ACC Gen1 无扩围，不引入完整 borrow checker 要求。探针覆盖继承控制流、五类来源、程序化 Drop、真实声明/提供/合同门与非法输入；未声称穷举任意 CFG/Unicode/分配规模。

## 重放与下一步

原始 JSON 记录输入 SHA256、HEAD、check/build/run 32位 Code、分离输出、deadline与读错误。CLI 使用仓库既有 AcContainedRunner（60s），探针不解释源码/HIR。新的 helper 仅为实际 API 输入构造/变异。

在受审树执行（先完整 gate，后探针串行避免占用 exe）：

```powershell
$e = 'D:/autostack/auto-lang/docs/plans/attachments/770/review-r3'
$deps = (Resolve-Path experimental/ac-core/target/debug/deps).Path
$lib = (Get-ChildItem $deps -Filter 'libauto_ac_prototype-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
rustc --edition 2021 "$e/emit-probes.rs" --extern "auto_ac_prototype=$lib" -L "dependency=$deps" -L native=C:/Users/zhaop/.cargo/registry/src/mirrors.aliyun.com-0671735e7cc7f5e7/windows_x86_64_msvc-0.52.6/lib -C link-arg=/STACK:33554432 -o experimental/ac-core/target/review-r3-emit.exe
& experimental/ac-core/target/review-r3-emit.exe $e
powershell -NoProfile -ExecutionPolicy Bypass -File "$e/replay.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "$e/historical-r2/replay.ps1"
```

不能以脚本退出0宣称这些案例通过；对照各项 expected 与 raw actual。
outcome=needs_fix；status=executing，保留 T01/T03，重开 T02/T04/T05/T06/T07，current_step=2/9；T08/T09 未完成。work 同树修复五项并更新完整闭合表、永久 oracle、fresh gates，再交 final review。合入/SD沉淀/ledger/index/归档/guard/批回归到期判定为 pending closeout，不能承担未完实现义务。
