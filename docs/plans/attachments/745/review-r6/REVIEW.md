# PLAN-745 r1 第六轮最终独立复审：needs_fix

日期2026-10-07；review_scope=final，plan_revision=1，review_round=6。
实现HEAD **522b98a4c27d2064049e188d386f6958721b3a90**；上一轮15fd9263f6f4c400a6ba1d8ec5cb2cae44354d41；原代码基线eea7a80618d4d5e75749d666b7609a9a87925bbd。
入口主检出833049a74bbe02b909881830069665f78f1665b6，v0.6-dev；原专属树D:/autostack/.wt/lang-745/auto-lang，branch plan-745-dev；组内只读auto-down依赖fba6563ed2148ce85e68208863159b4ccccac710。双方入口clean。
原规划及历轮复审Codex上下文，独立于GLM实施/修复上下文；没有产品修复、另建树、委派、合入或canonical/live ledger发布。Plan读取主检出，代码及门禁在唯一登记专属树。入口与终审绑定见[baseline.json](baseline.json)、[final-baseline.json](final-baseline.json)，证据指纹见[artifact-manifest.json](artifact-manifest.json)。

## 上轮发现的处置

| 发现 | 当前独立验证 | 处置 |
|---|---|---|
| P745-R5-QA-01 非连续括号栈溢出 | J4096-True（16410字节）check0/约275ms，J4096-False（12314字节）check1/约14ms/source.syntax；J1500..3000正反均正确 | 原反例清偿 |
| P745-R5-QA-02 调用上限 | K03/H07的1000/1001层调用check0；L01的20000层调用check0；L02的4096层混合f(1+…)真实native退出4097；L03未闭合20000调用受控拒绝 | 调用子项清偿 |
| P745-R5-QA-02 块上限 | K02的999层if真实native退出1；K04的1000层完整if被syntax拒绝；r1/SD无修订 | **块子项仍未清偿** |
| 其它历轮修复 | C/D/E/F全部回放；完整74源码、实际Bundle变异、事务、Rust/PS正常与真实失败出口控制均保持 | 清偿保持；新解析回归另列 |

表达式/调用已由显式帧栈解析，resolve与adapter的CallOut和Expr Drop也迭代化；不再称原4096非连续括号或1000调用仍崩溃/误拒。连续100000括号H06仍通过。证据：[crash-results.json](crash-results.json)、[novel-results.json](novel-results.json)、[resource-results.json](resource-results.json)、[calls-results.json](calls-results.json)。

## P745-R6-QA-01 / P2：二元实参在逗号前没有归约，合法调用被拒绝

新实现回归；AC-02/05/08，T-02/T-05/T-07。`experimental/ac-core/src/source/parser.rs:1066–1074` 在逗号分支先要求栈顶为Call，而当前实参含+/*/<时，栈顶仍为Bin。没有先归约本实参内的算符、再查看最近的Call/Paren边界，直接报syntax。

| 独立输入 | 期望 | 实际 |
|---|---|---|
| M01 `f(1+2,3*4)`，f=x*10+y | native42 | check1，text/source.syntax@1:69逗号 |
| M02 `f(y:3*4,x:1+2)` | native42，按命名绑定 | check1@1:71逗号 |
| M06 `f(f(1,2)+3,f(2,1)*2)` | native192 | check1@1:74 |
| M07 bool实参 `f(1+2<4,5)` | native5 | check1@1:94 |
| H09 混合括号/调用 | 上轮native111 | 同一输入SHA，当前check1@1:79 |
| M18 `f(b:mark_b()+1,a:mark_a()*2)` | native23/stderr ba | check1@1:78 |
| M10 `f((1+2),(3*4))` | native42 | 正控制：实际42 |
| M15 `f(1,2+3)` | native15 | 正控制：实际15 |
| M17 给两trace实参分别加括号 | native23/stderr ba | 正控制：实际23/ba |

[parser-results.json](parser-results.json)、[parser-extra-results.json](parser-extra-results.json)含完整输入与原始结果；[regression-input-comparison.json](regression-input-comparison.json)确认H09前后输入一致，上轮附件证明111，不以不同输入冒充回归。

修复要求：逗号处只归约当前实参的算符，保持外层帧与值栈mark隔离，再完成实参；不能跨Paren接受非法逗号或消耗外层lhs。补positional/named/异构bool/嵌套调用及trace的正反永久回归。逐实参加额外括号只是定位控制，不是批准workaround或支持面排除。

## P745-R6-QA-02 / P2：未闭合帧被当表达式结束，非法源码还能发布制品

新实现回归；AC-02/04/06/08，T-02/T-05/T-07。
Parser `974–986`的EOF分支与`1085–1095`的普通终止分支，只在**归约前**检查栈顶Call/Paren；栈顶为Bin时跳过检查，reduce_ops移除Bin后残留Call/Paren，却直接break返回一个值。完整文件外层的`}`、`;`或语句边界可以掩盖缺失`)`。CallFrame甚至被丢掉，返回末个实参表达式。

实际反例（全部原样check-source、build-source、真实PE）：

- M03 `fn main() int { return (1+2 }`：**check0/build0/native3**。
- M04 `fn f(x int) int { return x*10 } fn main() int { return f(1+2 }`：**check0/build0/native3**，调用f被丢掉。合法闭合版本应为30；当前非法版本应在text阶段拒绝。
- M05 `return missing(1+2 }`：check0/build0/native3，连未定义callee都没有进入名字解析。
- M08/M09 初始化表达式在`;`前缺`)`：均check0/build0/native3。
- M13 `return (f()+2 }`：check0/build0/native5。
- M19 `if (1<2 { ... }` / M20 `while (n<3 { ... }`：非法输入分别check0/build0/native5/3。
- 有效负控制M11/M12：仅含字面量、缺`)`时确实check1；M14括号内非法逗号也受控拒绝。这些通过不能覆盖Bin在栈顶的组合路径。

**M21真三件SENTINEL控制**：预置普通exe/obj/receipt三文件，再对M04同源执行build-source。实际build0、PE退出3，三件SHA256全改变，`sentinel_preserved=false`。非法源码没有被前端拒绝，导致真实发布。不是模拟失败、不是删旧文件后比较，也不把所有旧事务控制误报为失效。

证据：[parser-results.json](parser-results.json)、[parser-extra-results.json](parser-extra-results.json)、[malformed-sentinel-results.json](malformed-sentinel-results.json)、[malformed-sentinel-probe.ps1](malformed-sentinel-probe.ps1)，M组各input.at/result.json与文本链接收据。M03..15新产生PE/obj/receipt三件hash已在独立回放结果保存；M21含前后逐件SHA。

修复要求：所有表达式退出路径（EOF、语句分隔、块/条件边界）在完成合法算符归约后，确认没有未闭合的Call/Paren，并保持exact mark/value invariants；缺`)`受控source.syntax且准确指向原token/EOF，失败无Checked/无lower/link/发布，已有三件不变。复验return/let/assign/if/while、嵌套调用、命名实参与换行组合，保留深输入修复。
真实binder/verify仍执行且Checked凭证唯一入口没有绕过；它们校验的是**错误parser生成的合法HIR**，不会自动发现被前端丢掉的调用。这里定位前端错误接受与构造，不能虚报verifier自身失效。

## P745-R5-QA-02 / P2：块边界未清偿，保留原finding ID

延续原支持边界子项；AC-02/07/08，T-02/T-05/T-06/T-07。调用子项已清偿，剩余仅语句块。
Parser NEST_LIMIT=1000的enter仍计函数体及嵌套块；K02的999层if（含函数体第1000层）实际native1，K04的**1000层完整if**在1:9016报text/source.syntax `nesting deeper than 1000 levels is not accepted`；H08的1001层也拒绝。完整输入与结果在resource/novel-results。

Plan revision仍1、§5.2没有块嵌套排除；proposed SD01/03也没有批准资源支持域。交接收据承认未完成，称上下文不足以安全迭代改造verify walk_block/native emit_block等路径，并呈报两个选项。这是明确的剩余工作，不能将T02/05/06/07全勾、execution_done7/9或“部分修好”当最终验收通过。

下一步：保留r1合同继续work修复块路径属于现有范围；没有额外用户选择也能按现合同执行。若选择保留资源上限，才须显式new修订并获批准，准确写计数单位/各入口/资源诊断/边界两侧验收与SD，不能由review默许放宽。review不代做实现，也不强加任意规模性能承诺。原finding保持未清偿，不重新编号来隐藏连续义务。

## 验证与实际限制

| 门禁/独立控制 | 实际结果 | 证据 |
|---|---|---|
| ac-core locked all-targets check/fmt | PASS/PASS，零新warning | ac-check/ac-fmt日志 |
| ac-core全测试 | **113 passed / 1 ignored**，exit0 | ac-test日志 |
| 741一键 -SkipMainGates | **13步通过** | verify741日志 |
| 745一键 | **13步/74例通过** | verify745日志 |
| 独立74源码 | **74/74**；21真实PE/53准确stage-code拒绝，输入与三件指纹 | corpus-results.json与corpus/** |
| strict ACC inventory | exit0，**47决定新鲜**，9输入/146 unknown闭环 | inventory日志 |
| 当前库Bundle真实变异 | 有效control通过；同I32越域local真实reverify报uninitialized-read | bundle-results.json、Cargo JSON唯一rlib/hash |
| 执行器正常/真实失败 | Rust原样helper4路通过；PS9/9（约束失败9ms无幸存、读error6/22ms且wrapper抛错）；迟到输出/EOF/UTF8正确 | helper/csharp-control-results.json |
| 已识别坏源/坏link/错entry/实际publish回滚 | 旧三件SENTINEL保持；永久真实publish回滚通过；20000 trace完整 | supplemental-results.json、ac-test日志 |
| 额外native | 300项链=300，复杂求值=24/baab，trap70先于mark_b，4096混合call=4097，999call/if=1 | boundary/calls/resource-results |
| root check | PASS；生产既有382warning | root-check日志 |
| cargo t --no-fail-fast | **4935/4972 passed，37 failed，1534 skipped**，exit100 | root-t日志 |
| cargo tv --no-fail-fast | **158/162 passed，4 failed，6330 skipped**，exit100 | root-tv日志 |

37个日常失败均在五轮历史40个唯一红名集合中，新增名0；tv仍4个semver族。生产crates、根Cargo/lock、.cargo/.config相对745原基线无diff；主检出有其它计划变更，历史名归因不等于严格同源码main对照、根门禁全绿或仅凭计数证明零新增。[daily-attribution.json](daily-attribution.json)保留口径。
门禁串行，未跑无触面的tf/taa/tt/tb/tu/docs_gen单独档。3个日常门禁生成的已核对普通叶文件已清理；原树与只读依赖最终clean、HEAD稳定。临时PE/obj不入库，文本收据/对应指纹保留。
74例及全部已有测试全通过，说明永久矩阵尚未覆盖逗号前Bin与未闭合帧组合；不能以banner代替这些实际负控制。遗漏/延后/workaround审计：块边界明确未完成、额外括号仅控制、不得将二者作为授权缩域。

## AC / Spec delta / 路由

| AC | 本轮结论 | 任务/依据 |
|---|---|---|
| AC-01 | pass | T01/03/05，数值报告不变，范围/精确类型/width/char/trap证据保持 |
| AC-02 | fail | T02/05，R6-QA01/02与R5-QA02块项 |
| AC-03 | pass | T03/04/05，真实bind→verify→Checked与实际Bundle变异；明确不拿它证明parser正确 |
| AC-04 | partial | T02/03/05，已有span/UTF8/CRLF精确token通过；部分畸形源码未产生受控诊断 |
| AC-05 | partial | T05，原21PE/trace/trap与深调用通过；合法二元实参调用误拒 |
| AC-06 | partial | T04/05，共用finish_build/CLI/capability/执行器/旧事务通过；M21非法源码错误发布 |
| AC-07 | partial | T01/06，strict盘点/SD02/04新鲜；SD01/03块支持边界未完成 |
| AC-08 | fail | T07/08，三项合同内P2，不授予最终pass |
| AC-09 | pending | T09，merge所有权，未执行 |

冻结SD01..04与当前提案SHA256一致：da129e1dcaa2e0bbc6dbba9202813046c8bda45316375b94e1bd729587378b82。四份canonical Specs、四份验证配置、依赖与HEAD无漂移。SD01/03未定稿；SD02阶段身份/唯一Checked入口与SD04完整int/char open边界保持。不修改canonical/live ledger、人工盘点绑定，不重开GOAL-017。

needs_fix → 同一原树work修复并提交新完整HEAD后再最终复审。T02/T05/T06/T07及依赖证据重开；T01/T03/T04保留完成，**executing3/9，revision1**，T08/T09待完成。T09主线整合、Specs/人工层/ledger/index/导航、批回归到期判断、真实归档与guard/树清理继续待最终pass和合入授权。