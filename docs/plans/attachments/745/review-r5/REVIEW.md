# PLAN-745 r1 第五轮最终独立复审：needs_fix

日期 2026-10-07；review_scope=final，plan_revision=1，review_round=5。
实现提交 **15fd9263f6f4c400a6ba1d8ec5cb2cae44354d41**，上一轮 dc5e95be757dba647ef758aa283e7e2788015747，原代码基线 eea7a80618d4d5e75749d666b7609a9a87925bbd。
主检出入口 a137fa4ec99387dbe91dc07d6ee21a831deb6d4b；实现树 D:/autostack/.wt/lang-745/auto-lang，branch plan-745-dev；只读 auto-down 依赖 fba6563ed2148ce85e68208863159b4ccccac710。
入口双方 clean。原规划及历轮复审 Codex 上下文，独立于 GLM 实施/修复上下文；没有实施产品修复、另建树、委派、合入或发布 canonical/live ledger。Plan 取自主检出，代码与全部门禁在登记原专属树。
[baseline.json](baseline.json) 绑定入口 Plan/Spec/配置；[final-baseline.json](final-baseline.json) 绑定终审稳定性；[artifact-manifest.json](artifact-manifest.json) 为附件指纹。

## 修复处置

| 上轮项 | 第五轮独立证据 | 结论 |
|---|---|---|
| R4-QA-01 未入 Job 的挂起子泄漏 | 原样 RunControl 驱动真实 AssignProcessToJobObject 失败，13ms 返回 containment unavailable，记录 PID 已死亡 | 清偿 |
| R4-QA-01 读取错误伪装成功 | 原样 RunControl 驱动真实 ReadFile 错误6，30ms 返回 Error，子已死亡；有效控制完整 READ_CONTROL | 清偿 |
| R4-QA-01 wrapper 失败传播 | 只将当前 wrapper 调用接到真实失败 seam，约束/读取两路均抛 contained executor refused | 清偿 |
| R4-QA-02 200层完整括号误拒 | G01/G02 199/200层、H06连续100000层均check0；H09/H10混合优先级真实native111/5 | 原具体反例清偿；剩余边界转R5-QA-02 |
| R3-QA-01 正常执行器生命周期 | Rust原样helper与PS原样runner：持管道统一截止、迟到LATE完整、父树超时无幸存、UTF8 lossy保留 | 清偿保持 |
| 历史C/D/E/F项 | 全回放；换行、符号、Stray、token、16384项合法链与畸形链均维持正确 | 清偿保持 |

PS控制详见 [csharp-control-results.json](csharp-control-results.json)、[csharp-controls.ps1](csharp-controls.ps1)、[csharp-runner-exact.ps1.txt](csharp-runner-exact.ps1.txt)；9/9控制通过。Rust详见 [helper-results.json](helper-results.json)，唯一当前Cargo JSON rlib绑定。
正常持管道/失控父树PS实测572/519ms（500ms预算含清理）、迟到输出668ms完整；Rust520/514ms、迟到697ms，无存活后代。

## P745-R5-QA-01 / P2：非连续括号嵌套仍使源码编译器栈溢出

AC-02/04/08，T-02/T-05/T-07。原深链及连续括号问题已修，此项是新解析路径覆盖缺口，仍属原合同“合法支持集通过、畸形输入受控无panic”的义务。

`experimental/ac-core/src/source/parser.rs:642,686,854–878` 只把**连续**左括号消为循环，随后仍 `self.expr(true)`；`1+(1+(…))` 每层都经 binary→unary→parenthesis→expr 递归，而且已不计括号护栏。连续括号塔不是所有括号嵌套的等价控制。

独立输入构造：`fn main() int { return ` + `1+(` 重复N次 + `0` + `)` 重复N次 + ` }`。

| 输入 | 字节数/结果 |
|---|---|
| 1500、2000、2500、3000层完整 | check0，约0.13–0.21s |
| 同深度缺全部右括号 | check1，text/source.syntax，约0.02–0.03s，无panic |
| **4096层完整（J4096-True）** | **16410字节**；stderr `thread 'main' (…) has overflowed its stack`，5s仍未返回，复审驱动终止回收 |
| **4096层未闭合（J4096-False）** | **12314字节**；同样stack overflow，5s仍未返回，驱动终止回收 |
| 连续100000层完整括号H06 | check0，证明新失败不等同连续塔或单纯token数量 |

[crash-results.json](crash-results.json)、[crash-probes.log.txt](crash-probes.log.txt)、[crash-probes.ps1](crash-probes.ps1)、extra-probes/J4096-{True,False}/input.at 与 result.json 保存精确输入/原始stderr。初次H04/H05的10s timeout驱动没保留stderr；已修复**复审驱动**在kill后排空并以J组重新确认，不能把前次空stderr当作实现无错误。
这里报告的是实际CLI栈溢出与受控截止结果，不虚构自然退出码或调试器逐帧阶段定位。所有本次超时进程已在驱动中tree-kill并wait，未修改产品实现。

修复要求：覆盖非连续/RHS括号递归及组合输入，完整合法表达式通过实际source→Checked，未闭合输入返回受控source诊断；保留优先级、换行、效果/溢出顺序与准确span。补独立CLI正/负永久回归，不能仅增加连续括号数量或线程栈大小来宣称该路径完成。

## P745-R5-QA-02 / P2：调用/块上限仍未进入批准合同

延续R4-QA-02的未完成边界子项，AC-02/07/08，T-02/T-05/T-06/T-07。
Parser `NEST_LIMIT=1000`（81行），enter（217行）用于函数体/block与调用参数表（429/834行）。函数体已占一层，因此实际CLI **999层嵌套调用/if可接受，1000层完整合法输入已拒绝**，并非所有入口都能接受1000层。

K01/K02的999层调用及if均真实build-source→PE→退出1；K03/K04的1000层完整语法均check1，text/source.syntax，在1:2052/1:9016报 `nesting deeper than 1000 levels is not accepted`。H07/H08的1001层也拒绝。[resource-results.json](resource-results.json)、[resource-probes.ps1](resource-probes.ps1)、[novel-results.json](novel-results.json)保存边界两侧。

当前主Plan仍revision1；§5.2支持括号/直接调用/块，没有批准上述排除；冻结proposed delta也未载此边界。执行收据承认r1无此语法排除、如需入合同属new修订权限，但“语料最深约4层/实测溢出阈值/资源防护”不能代替上一轮明确要求的合同处理。本轮不默许降低支持集，也不提出任意规模性能承诺。

修复要求：保留原合同支持面；如选择有限资源支持域，必须显式new修订批准合同，定义计数单位/函数体等各入口/组合路径、资源诊断、支持边界与两侧验收，同步SD01/03/README后重新复审。不能继续只把阈值放大，并在revision1下将有效语法当作syntax错误宣称语义未变。R5-QA-01的畸形输入无panic义务仍须完成，不能由边界修订自动豁免。

## 本轮门禁与验收证据

| 检查 | 实际结果 | 附件 |
|---|---|---|
| locked ac-core all-targets check / fmt | 0/0；ac-core无新warning | ac-check/ac-fmt日志 |
| ac-core全测试 | **113 passed / 1 ignored**，exit0 | ac-test日志 |
| 741一键 -SkipMainGates | **13步通过** | verify741日志 |
| 745一键 | **13步/74例通过** | verify745日志 |
| 独立JSON全部源码 | **74/74**；21个真实PE/53个stage-code拒绝，输入与三件制品hash | corpus-results.json，corpus/** |
| strict ACC inventory | **47决定新鲜**，9输入/146 unknown闭环，exit0 | inventory日志 |
| 同I32越域真实Bundle变异 | 有效control通过；确实改local后真实verify.uninitialized-read | bundle-results.json及精确rlib绑定 |
| 事务/排空 | 坏源码/坏link/错入口三件SHA均保持；永久真publish回滚测试通过；20000 trace完整 | supplemental-results.json，ac-test |
| 额外native | 300项链退出300、trap先于mark_b为70/空stderr、复杂named/binary为24/baab、混合括号111/5、999call/if退出1 | boundary/novel/resource-results.json |
| root check | PASS；生产既有382warning，非新ac-core warning | root-check日志 |
| cargo t --no-fail-fast | **4939/4972 passed，33 failed，1534 skipped**，exit100 | root-t日志 |
| cargo tv --no-fail-fast | **158/162 passed，4 failed，6330 skipped**，exit100 | root-tv日志 |
| 唯一新增红名隔离 | plan707_cancel_active_job_aborts_and_frees_permit **1/1通过**；同树唯一缓存单测exe/完整命令与hash绑定 | root-scoped.json/stdout/stderr |

32个日常失败名已在历轮39个历史红名集合中；唯一新增plan707_cancel隔离通过。tv仍为4个semver族。745相对原基线在生产crates、根Cargo/lock、.cargo/.config无diff；主检出另有并发计划变更，历史名归因不能称严格同源码main对照、根门禁全绿或只按计数证明零新增。见 [daily-attribution.json](daily-attribution.json)。
全部门禁串行，未跑无触面tf/taa/tt/tb/tu/docs_gen单独档。日常门禁生成的3个已核对普通叶文件已清理，原树最终clean。附件中临时PE/obj删除，文本收据与制品SHA保留。
复审C# wrapper首次注入脚本漏PS布尔`$`而解析失败（initial-error日志保留），已更正后9/9重跑通过；属于复审驱动错误，未作为实施发现。两次自动审批等待超时未执行相关命令；之后只读当前runner/缓存binary检查成功，无未完成审批阻塞项。

## AC / Spec delta / 状态

| AC | 结果 | 对应任务/证据 |
|---|---|---|
| AC-01 | pass | T01/03/05，数值报告、范围/精确类型/运行trap和宽度/char限制 |
| AC-02 | fail | T02/05，R5-QA01/02；其余74/C/D/F行为通过 |
| AC-03 | pass | T03/04/05，真实bind→verify→Checked，实际Bundle变异，741负矩阵 |
| AC-04 | partial | T02/03/05，正常原span/UTF8/CRLF/token通过；未闭合J4096仍崩溃无受控诊断 |
| AC-05 | pass | T05，21PE/trace/trap/控制流；正常与失败执行器控制清偿 |
| AC-06 | pass | T04/05，显式CLI/entry/profile/capability/user-shadow/shared finish_build，三件事务/失败回收 |
| AC-07 | partial | T01/06，strict盘点/SD02/04新鲜；SD01/03支持边界不一致 |
| AC-08 | fail | T07/08，两项合同内P2，不能最终pass |
| AC-09 | pending | T09，merge所有权，未执行 |

SD01..04冻结副本SHA256=da129e1dcaa2e0bbc6dbba9202813046c8bda45316375b94e1bd729587378b82，与当前实现提案一致。终审四份canonical Specs、四份验证配置、HEAD与依赖无漂移；SD01/03不能定稿，SD02阶段身份/Checked唯一凭证及SD04完整int/char保持open无新冲突。不发布canonical/live ledger，不重开GOAL-017。

needs_fix → 同一树work修复，提交新完整HEAD后重新最终复审。T02/T05/T06/T07及依赖证据重开，保留T01/T03/T04完成，**executing 3/9**，revision1保持；T08/T09待完成。选择资源边界变更则转显式new修订后work。T09主线整合、Specs/人工层/ledger/index/导航、批回归到期判断、归档及guard/tree清理仍待最终pass和合入授权。