# PLAN-745 r1 第四轮最终独立复审：needs_fix

日期：2026-10-07。review_scope=final，plan_revision=1，review_round=4。
实现 HEAD=`dc5e95be757dba647ef758aa283e7e2788015747`；上一轮 HEAD=`7437eb137a789259f01446da7a170e74031ff7f2`；原始代码基线=`eea7a80618d4d5e75749d666b7609a9a87925bbd`。
工作树 `D:/autostack/.wt/lang-745/auto-lang`，分支 `plan-745-dev`；auto-down 只读依赖仍为 `fba6563ed2148ce85e68208863159b4ccccac710`。入口主检出=`287ec2aa10567dca294b7c46daa0907d38fae328`，双方入口 clean。
复审人为原规划及前三轮复审 Codex 上下文，独立于 GLM 实现/修复上下文；没有实现修复、委派、merge 或 canonical/live ledger 发布。当前 Plan 来自主检出，代码和测试来自原专属树。
入口 Plan/Spec/配置/冻结 delta 哈希见 [baseline.json](baseline.json)；终审稳定性、工具链和证据清单见 final-baseline.json、toolchain.txt、artifact-manifest.json。

## 上轮修复的实际结果

| R3 finding | 本轮独立验证 | 处置 |
|---|---|---|
| QA-01 Rust 自制助手遗留后代/读线程 | 原样当前 helper、唯一 Cargo JSON 绑定 rlib；父先退出持管道约515ms超时且后代死亡；迟到输出完整 LATE；父+后代超时树杀；UTF8 lossy 正确 | 原 Rust 生命周期问题清偿 |
| QA-01 PS 静默窗口提前成功 | 新 C# executor 原样控制：持管道有界超时且后代死亡；迟到输出完整；失控父树杀；无效UTF8保留 | 原静默窗口问题清偿；新错误出口缺口见 R4-QA-01 |
| QA-02 合法16384项链崩溃 | F08 check exit0；F09类型错误、F02/F07缺RHS、E02/F05不闭合均受控；G07真实native 300项链退出300 | 原崩溃清偿，不能再称F08仍崩溃 |
| QA-02 合法99/100层括号误拒 | F03/F04现在exit0；199层括号、199层调用/混合表达式、199层if均check0 | 原99/100反例清偿；未批准的支持上限仍存在，R4-QA-02 |

旧 C01..22、D01..14、E01..02、F01..09 全回放。C/D 的符号、换行、Stray、准确 token、保守控制流没有退化。新 native G08 在左侧trap后不观察右侧mark_b（70/空stderr），G09复杂调用/二元求值为24/baab，迭代化的效果/trap相对顺序通过。证据：[counterexamples.json](counterexamples.json)、[extra-results.json](extra-results.json)、[depth-results.json](depth-results.json)、[new-results.json](new-results.json)、[boundary-results.json](boundary-results.json)、[helper-results.json](helper-results.json)。

## P745-R4-QA-01 / P2：新 PS 执行器错误出口仍泄漏或返回截断成功

关联 R3-QA-01 的新实现；AC-05/06/08，T-05/T-07。正常持管道/迟到输出/超时控制已过，本项不否定这些实际修复。合同 §5.4/§6 和 auto-ac 当前 Spec 要求约束失败在挂起态安全终止并回收、读取错误传播，不能因正常路径通过就省略失败出口。

1. `scripts/verify-ac-source-745.ps1:231–237,297–309`：CreateProcess成功后，Job创建或分配失败，只调用 TerminateJobObject。此时子进程尚未加入Job；finally同样只终止Job并关闭进程/线程handle。没有对未入Job的挂起子进程执行直接终止和wait；关闭handle不会终止进程。
2. 同文件 `Reader.Drain:164–172`：ReadFile返回false被直接当作EOF，置Done=true；Run成功出口没有读取错误字段，仍Code0/Error=null。读取失败可以伪装为“输出完整、成功”，与取消静默窗口的目标相同风险。

独立控制从**当前脚本原样提取** C# 与 wrapper；未改工作树实现。有效分配/读取控制 `cmd /c echo READ_CONTROL` 返回0、完整文本、子进程已退出。
约束负控制只把实际 AssignProcessToJobObject 的Job参数改成IntPtr.Zero，触发真实Win32失败；附加PID遥测，原cleanup代码不变。返回 `containment unavailable` 后**挂起子进程仍存活**（最新复现约16ms）；probe再仅终止此明确记录的自有PID。
读取负控制只把ReadFile句柄改成IntPtr.Zero，并记录原分支的OS错误；真实错误码**6**，原result/cleanup处理不变。结果仍 **Code=0 / TimedOut=false / Error=null / stdout为空**，有效控制本应含READ_CONTROL。不是模拟一个预制ALL-PASS结果，两个native调用都真实执行。

证据：[csharp-control-results.json](csharp-control-results.json)、[csharp-controls.ps1](csharp-controls.ps1)、[csharp-runner-exact.ps1.txt](csharp-runner-exact.ps1.txt)、[csharp-containment-control.cs.txt](csharp-containment-control.cs.txt)、[csharp-read-control.cs.txt](csharp-read-control.cs.txt)。Rust正常路径见 helper-results.json；本项的两个负控制针对本轮新增的PS执行器。

修复要求：对尚未入Job的挂起子进程直接终止+wait，处理Job创建失败和分配失败，保持首指令前拒绝；所有读/等待/恢复/终止错误传播到Invoke-Captured及验证失败，不把ReadFile错误一概作EOF。按真实管道EOF规则区分正常结束与失败，回收读线程/句柄；补永久有效/无效控制，断言错误出口无幸存进程、没有截断成功。保留正常路径已通过控制。

## P745-R4-QA-02 / P2：语法计数修正后，上限仍未进入批准合同

延续 R3-QA-02 的支持边界子项；AC-02/07/08，T-02/T-05/T-06/T-07。原深链崩溃与99/100括号误拒已经修复，这里只追踪未完成的边界义务。

`src/source/parser.rs:68–82,216–225,434–436` 保留NEST_LIMIT=200，现在计数括号/调用/块；函数体本身占一层。G01的199层合法完整括号check0；G02的**200层完整括号（426字节）**check1，在1:223报 `text/source.syntax: nesting deeper than 200 levels is not accepted`。它是完整合法语法，r1没有此排除项。
Plan revision仍为1，代码注释说“not a support-set reduction”；README和[冻结delta](proposed-spec-delta.frozen.md)也没有该资源支持边界。上一轮已明确：如保留输入上限，应显式走合同修订，不能只放大阈值或用“真实语料只有4层”作为授权。本轮修复把计数单位改对，却没有处理这一要求。

证据：[boundary-results.json](boundary-results.json)、G01/G02完整input.at/result.json、[boundary-probes.ps1](boundary-probes.ps1)。G03..06说明目前199层调用/if等受支持；不声称所有深输入都有崩溃，也没有提出任意规模性能承诺。

修复要求：保留r1支持集时，合法完整括号等输入应通过， malformed仍受控拒绝；若实现必须保留上限，则进入显式new合同修订，记录语法单位/各入口计数/批准支持边界/诊断与正反验收，并同步proposed Spec delta。不能在revision1下把有效语法误称syntax错误并宣称合同语义未变。本轮不自行批准或降低原合同。

## 验证与实际限制

| 门禁/控制 | 结果 | 证据 |
|---|---|---|
| ac-core locked check all-targets / fmt | 0/0，零新warning | ac-check、ac-fmt日志 |
| ac-core全部测试 | **112 passed / 1 ignored**，exit0 | ac-test.stdout/stderr.txt |
| 741一键 -SkipMainGates | **13步通过** | verify741.stdout/stderr.txt |
| 745一键（正确PS5模块环境） | **12步/74例通过** | verify745.stdout/stderr.txt |
| 独立完整源码语料 | **74/74**；21正例真实PE，53拒绝；输入、输出、三件SHA256 | corpus-results.json、corpus/** |
| strict inventory | **47决定新鲜，9输入/146 unknown闭环**，exit0 | inventory.stdout/stderr.txt |
| 真实Bundle同类型越域 | 有效control通过，改read local后真实verify.uninitialized-read | bundle-results.json、cargo-artifacts.jsonl |
| 事务/大输出 | 坏源/坏link/错入口三件不变，真publish回滚永久测试通过；20000 trace完整排空 | supplemental-results.json、ac-test日志 |
| root check | PASS；382生产既有warning | root-check.stderr.txt |
| cargo t --no-fail-fast | **4937/4972 passed，35 failed，1534 skipped**，exit100 | root-t日志、daily-results.txt |
| cargo tv --no-fail-fast | **158/162 passed，4 failed，6330 skipped**，exit100 | root-tv日志 |
| 唯一新增日常红名plan643隔离 | **1/1通过**，exit0 | root-scoped.stderr.txt |

其余34处日常失败都在前三轮历史集合中；plan643 charts红隔离通过，4处tv仍是semver族。生产crates、Cargo/lock、.cargo/.config相对745原基线无diff。历史对照仅支持上述归因边界，主检出有并发计划变更，**不称严格同源码main对照、根门禁全绿或仅凭计数证明零新增**。见daily-attribution.json。
门禁串行，未跑无触面的tf/taa/tt/tb/tu/docs_gen单独档。修正过的PSModulePath复审驱动本轮正常，不再将上轮环境错误列为实现问题。

## AC / Spec delta / 下一步

| AC | 结论 | 依据 |
|---|---|---|
| AC-01 | pass | 数值/类型/范围/trap和int/char限域报告保持 |
| AC-02 | fail | R4-QA-02未经批准的语法支持上限 |
| AC-03 | pass | 长表达式完整source→bind→verify→Checked通过；当前库真实Bundle变异与741负矩阵保持 |
| AC-04 | pass | 原来源全覆盖、UTF8/CRLF、C/D准确token，永久测试通过 |
| AC-05 | partial | PE退出/trace/trap、迭代native通过；执行器失败路径未满足 |
| AC-06 | partial | CLI/profile/entry/shared尾部/三件事务通过；R4-QA-01错误出口未满足 |
| AC-07 | partial | SD01..04/strict inventory新鲜；SD01支持边界尚与实现不一致 |
| AC-08 | fail | 两项合同内P2，不授予最终pass |
| AC-09 | pending | merge所有权，未执行 |

SD02阶段身份、Checked唯一入口与SD04完整int/char继续open没有新增冲突；SD01/03须在支持边界义务完成后重新绑定。冻结delta哈希与实现提案相同，未发布canonical/live ledger、不重hash人工层来掩盖漂移，touched_goals=[]不重开GOAL-017。

needs_fix → 同一工作树work修复并提交新完整HEAD后再最终复审。T-02/T-05/T-06/T-07及依赖证据重开；T-01数值、T-03真实Checked管线/迭代修复、T-04共享构建/CLI/事务保留完成，**executing 3/9**；T-08/T-09未完成，plan_revision暂保留1。
若选择上限合同变化，需显式new修订；不在复审中默许。T09主线整合、Specs/人工层/ledger/index/导航、批回归到期判断、真实archive及guard/树清理，仍待最终pass和合入授权。本轮原树保留。
