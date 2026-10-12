# PLAN-745 r1 第二轮最终独立复审（2026-10-06）

**outcome: needs_fix**。上一轮多项修复真实有效；本轮仍有五项 P2，不能将内部 99 通过当作完整合同已验收。

stage=review / review_scope=final / plan_revision=1 / review_round=2。复审者为原规划及第一轮复审 Codex 对话，独立于 GLM 实现/修复上下文。本轮只审查、运行验证及保存证据，未修改实现、委派、合入或发布 canonical Specs/ledger。语义合同及验收 ID 保持。

## 基线

- 同一专属树：`D:/autostack/.wt/lang-745/auto-lang`，`plan-745-dev`。
- reviewed_commit：`91b37560cffda75cf07cef8620cf524f5d6a3c7c`，入口 clean。
- 原实现 base：`eea7a80618d4d5e75749d666b7609a9a87925bbd`；本轮增量 base：`b3cd8171966281a42f96187780342ce0ff51c5fb`。
- 主检出入口：`v0.6-dev @ bd7aa30ba153908b6ccf73b383b5437b3cf33406`；auto-down 依赖未变：`fba6563ed2148ce85e68208863159b4ccccac710`。
- [baseline.json](baseline.json) 保存原 Plan/四项 Specs/验证配置/冻结增量 SHA256；[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md) 是本轮原样审查输入；[toolchain.txt](toolchain.txt) 保存 CLI/support lib/Rust 指纹。
- 修复差异为30文件，1013新增/258删除；对 `crates/`、生产 Cargo/lock、`.cargo/.config` 的原实现到新HEAD差异仍为零。没有改变 Schema、HIR 三重身份或完整 Auto/ACC 支持边界。

## 上轮 QA-01..08 的实际处置

| 第一轮发现 | 本轮处置及证据 | 状态 |
|---|---|---|
| QA-01 非括号跨行误接受 | C01/C03现拒绝，C02括号控制通过；增加了首行约束，但 D01..03 合法嵌套换行被误拒 | 原反例已修；新增回归见 R2-QA-01 |
| QA-02 ac_start/ExitProcess 撞 native 符号 | C05 native=3、C06=4；永久测试实际执行 ac_start入口=3；D07用户sf_ac_start和ac_start同时存在=7 | 清偿，不重新挂原符号问题 |
| QA-03 精确诊断 | C09现在1:57，C10现在3:9，C18也指x；新Stray组合仍被不相关死块掩盖，D10/D13控制 | 原位置反例已修；R2-QA-03待修 |
| QA-04 关键词/排除分类 | C04完整转义str、C15 !、C12关键词、D11转义char、D12关键词实参均正确；生产56关键词覆盖缺项=0 | 清偿 |
| QA-05 有限截止/进程回收/输出 | 大输出实际完整排空，普通无限PE实际3s被杀；新助手join/脚本采集仍不受deadline，继承管道和无效UTF8控制失败 | 部分清偿；R2-QA-02待修 |
| QA-06 补充矩阵oracle | 真HIR trap结构断言、删let/错type Bundle变异、源/链接三件SENTINEL永久测试已新增；越域被替换为异类型引用，publish路径主动删除旧receipt后只验两文件 | 部分清偿；R2-QA-04待修 |
| QA-07 inventory误调用/误归因 | 错误说明明确撤回；正确目录命令strict47通过；merge不再预排write重扫、不重hash掩漂移 | 清偿 |
| QA-08 两套构建尾部 | build和build_source共用finish_build（lower/staging/tool discovery/link/同事务receipt/publish/cleanup）；前端各自解析入口和提供元信息 | 清偿；双模式测试通过 |

原始复现：[counterexamples.json](counterexamples.json)（C01..22），[extra-results.json](extra-results.json)（D01..14），[depth-results.json](depth-results.json)（E01..02）。C13/C21及D09按既有保守规则仅作调查，不新增含糊验收要求。R1记录保留为历史；本表区分具体已修缺陷与仍未满足的整体AC。

## 本轮发现

### P745-R2-QA-01 / P2：换行修复误拒合法括号内换行后的同一行续算

修复回归；关联 R1-QA-01；AC-02/05，T-02/T-05。

`src/source/parser.rs:575–580` 将整个非括号表达式固定在最初物理行，`:644` 将这一行传给后续 RHS，`:657–672` 不考虑前一个操作数已经在合法括号上下文中跨行并消费了闭括号。§5.2 要求括号内换行是空白；在闭括号所在行继续计算没有跨语句换行。

本轮实际：

```auto
fn f(x int) int { return x }
fn main() int { return f(
 2
) + 3 }
```

D01应得到5，却在4:5的`3`报source.syntax。D02 `return (1 +\n 2) + 3` 同样在2:7错误拒绝；D03多行负literal实参后续加法也拒绝。D04给整个计算再加一层括号实际native=6，D05多行call不带后续运算native=2，说明问题在合法括号子表达式退出后的外层边界，不是这些能力都未实现。

修复：按语句换行/括号上下文的实际边界判定，不能简单把全部外层token钉在最初行；在支持嵌套括号、call及多行实参的同时，继续拒绝C01/C03/D06和闭括号后真正另起语句行的非法续算。加入LF/CRLF、注释、嵌套及闭括号同行正反永久控制；不靠额外括号workaround缩小原支持集。

### P745-R2-QA-02 / P2：新deadline助手仍超时返回成功，且吞输出

原合同残留；关联 R1-QA-05；AC-05/06/08，T-05/T-07。

`tests/common/mod.rs:195–236` 的并发排空修好了第一轮4096字符死锁，但`:220–221/230–231` 的reader join仍无截止；没有Job/子树约束，父进程退出或被kill不保证后代持管道结束，`try_wait()?`错误出口也没有完整收口。`:206/211` 忽略read_to_string失败，join失败又unwrap_or_default，可能把丢失输出包装成成功。

直接从当前common文件提取**原样struct/函数**编译的探针（不是重新模拟助手）已实测：

- deadline=500ms，父进程立刻退出，后代继承stdout/stderr持管道5s：实际 **5016ms / timed_out=false / code=0**。5秒后代正常结束，本次没有留下长驻进程。
- 子进程输出 `PREFIX 0xff SUFFIX` 后exit0：助手返回 **stdout空字符串 / timed_out=false / code=0**，有效前后缀被吞。
- 当前 `verify-ac-source-745.ps1:32–51` **完全未随修复更新**。原样提取Invoke-Captured，TimeoutSec=1、同一个持管道后代：实际 **5044ms / Code=0**；WaitForExit之外的GetResult继续无限等，超时路径只Kill且不wait/收口reader。

证据：[helper-results.json](helper-results.json)、[helper-probe-exact.rs.txt](helper-probe-exact.rs.txt)、[script-helper-results.json](script-helper-results.json)、[script-helper-exact.ps1.txt](script-helper-exact.ps1.txt)。运行脚本是helper-probe.ps1；编译和输出位于原树target/review-r2，有12s外层review截止。脚本探针只调用提取的函数，不运行完整验证脚本的其它步骤。

修复：共享/复用741已成立的执行器或完整遵循其受控启动、子树约束、安全拒绝、单一截止同时覆盖执行/采集、全部出口回收合同。Rust测试助手与PS脚本两处均修；按字节排空、lossy转换并处理真正读错误，不把错误默认空输出当成功。必须加入父退出后后代持管道、父到期时子树、UTF8无效字节及失败传播的永久控制；大输出/普通无限父进程控制继续保留。增加超时数值不能修复无界join。

### P745-R2-QA-03 / P2：两个Stray分支被当成合法终结，掩盖真正循环目标错误

修复回归；关联 R1-QA-03；AC-02/04，T-03/T-05。

`src/source/adapter.rs:612–615` 把两分支Term::Stray与有效Term::Div等同，外层将后续语句拆成不连通块。verify结构检查失败会跳过dataflow（verify.rs:649起），所以循环目标诊断不再得到执行。

D10：`fn main() int { if true { break } else { continue }; return 0 }`，真正非法的break/continue分别在1:27/1:42，实际却只在1:54的`return`报verify.block-structure。D13去掉后续return，其余相同的两Stray分支又正确报1:27/1:42的verify.loop-target-scope。这是adapter制造的结构错误遮住原源码错误，并非要求改变现有verify诊断顺序。D08两个**有效**break/continue分支后的非法let和D14有效break后let均正确定位，须保留。

修复：Stray的语义和诊断可达性必须穿过if/loop等组合，不能在组合时提升为有效退出；保留整体verify，避免伪造死块使错误loop-target永远不被检查。加入两Stray、有效/Stray混合及嵌套正反控制，固定真正冒犯token/code/坐标。

### P745-R2-QA-04 / P2：两项补充oracle仍被相邻控制代替

原合同中的验证残留；关联 R1-QA-06；AC-03/06，T-03/T-05/T-07。本项**不宣称已发现产品verifier或事务破坏**。

1. `tests/source_hir.rs:551–646` 已真实变异投影后Bundle，但第三变异将I32参数读改成Bool参数读，要求verify.type-mismatch。这证明类型检查，仍不是§6规定的越域ref控制；相同类型下的作用域/可用性错误没有永久oracle。不能以“错local ref”重命名后当成完成越域项。
2. `tests/source_cli.rs:314–420` 的第三路径在调用构建前主动remove旧receipt，替换成目录；随后只比较exe/obj两文件和保留目录。无法证明原已有成功**三件文件**在真正publish失败后都被回滚保留，却在verification.md及Plan声称source三路三件byte-identical。它是有用的用户占位保护控制，应保留，但不能替代原三件套publish失败控制。

独立正反控制已补：经Cargo JSON唯一绑定**当前**rlib（没有缓存glob），用compile_parts构造合法`outer I32 / if块内inner I32 / return outer`，不变type，仅将return的ReadLocal改成inner。原Bundleverify通过，实际改变LocalId后重新verify得到 **verify.uninitialized-read**，证明产品已有相应能力。源码拒绝、坏链接、错入口三项已有SENTINEL实测也再次全部保留（supplemental-results.json）。这两组独立控制不能自动使缺失的永久验收完成。

修复：将同类型越域Bundle变异纳入永久测试，明确原合法→确实改变ref→目标不变量拒绝，不能只让不同type帮忙失败；source publish控制开始时保持三件真实旧文件，强制实际发布阶段失败并验证三件恢复（及残留/诊断），不能先删掉被保护对象。修订报告，分别描述两文件+目录保护与三文件回滚，按真正执行的oracle计数。

### P745-R2-QA-05 / P2：小体积不闭合源码导致编译器栈溢出退出

新发现的原合同缺口；AC-02/04，T-02/T-03/T-05。§6全消费矩阵要求不闭合输入受控text/source.*拒绝，无panic/Checked；这不是新增性能承诺。

E01 16层合法括号check-source=0。E02 **4122字节**、4096层未闭合`(`输入，在10s外层截止内直接退出 **-1073741571**，stderr为`thread 'main' ... has overflowed its stack`，没有source.syntax诊断和1退出码。代码路径是parser unary的括号→expr(true)→binary→unary递归，没有受控深度/结构保护。

证据：[depth-results.json](depth-results.json)和extra-probes/E02-deep-unclosed-parenthesis/input.at；可用depth-probes.ps1重放。每个进程10s外层截止，未构建native制品。

修复：对深层不闭合/畸形结构提供受控诊断并避免栈溢出；审查相同递归边界在call/blocks及后续source处理链的错误输入出口。可选择显式工作栈或有明确合同依据的资源防护，但不能以未经批准的profile支持收缩或仅扩大线程栈掩盖无界递归。永久测试保留浅合法控制及深不闭合输入，断言实际退出1、source阶段诊断、无panic/无制品。

## 验收与门禁

| AC | 结果 | 本轮证据/缺口 |
|---|---|---|
| AC-01 | pass | numeric报告限定int/i32=32，min/max HIR及signed PE边界、range排除、真add/mul overflow:Trap断言、C22负溢出70、char/full-width仍open |
| AC-02 | fail | 74原语料及已修C边界通过；R2-QA-01合法表达式误拒、QA-03错误分类/位置、QA-05栈溢出 |
| AC-03 | partial | 真bind→Bundle span投影→verify→Checked；fake marker/删let/错type拒绝；新独立同type越域拒绝已证实，QA-04永久oracle尚缺 |
| AC-04 | fail | C09/C10、UTF8/CRLF控制已正确；QA-03组合错误诊断和QA-05无受控诊断 |
| AC-05 | partial | 21源→PE语义通过、12/ba与70/a，C05=3/C06=4/C07=8/C08=21/ba/C22=70、D04=6/D05=2/D07=7；QA-01合法组合不完整，QA-02执行验收缺口 |
| AC-06 | partial | profile/entry/trace shadow/普通source三件保留、共享finish_build、旧CLI事务回归通过；QA-02和QA-04未兑现全合同 |
| AC-07 | partial | SD01..04边界/MD408/409正确strict47通过，R1-QA07已清偿；SD01/03完整能力描述待本轮修复后重新冻结/审定 |
| AC-08 | fail | 独立复审/门禁已完成，但五项待修，不能pass |
| AC-09 | pending | merge所有权：主线整合/SD/人工层/ledger/index/导航/到期批回归/真实归档断言/guard和原树清理 |

| 同树串行命令 | 本轮结果 |
|---|---|
| ac-core `cargo check --locked --all-targets --manifest-path ...` | PASS，零warning |
| ac-core fmt check | PASS |
| ac-core `cargo test --locked --manifest-path ...` | PASS，**99 passed / 1 ignored** |
| `verify-ac-741.ps1 -SkipMainGates` | PASS，13步骤 |
| `verify-ac-source-745.ps1` | PASS，12步骤/74例；脚本全绿不能覆盖未纳入的本轮反例 |
| 正确`acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions` | PASS，47新鲜人工决定/9输入/146 unknown族 |
| `cargo check -p auto-lang` | PASS，382生产既有warning；生产源/config未被745修改 |
| `cargo t --no-fail-fast` | **exit100，4972 run / 4939 pass / 33 fail / 1534 skipped，92.702s** |
| `cargo tv --no-fail-fast` | **exit100，162 run / 158 pass / 4 fail / 6330 skipped，4.112s** |
| `cargo t plan502_m5_hover_and_labels_e2e` | PASS，1/1，针对新出现的hover红单独核查，不扩跑求绿 |

没有将fail-fast未覆盖计为完成，本轮直接完整no-fail-fast。日常/语料失败完整名单在daily-results.txt/corpus-results.txt；原始日志及退出码保存在本目录各*-status.txt与*.log。33红除plan502 hover以外均在既有执行/第一轮双方红集并集中；hover本轮并行失败后隔离通过。4个tv红仍是cookbook semver parse/latest/increment/prerelease。上述事实配合生产零diff用于区分基线/负载与本计划触面，但**不称根门禁绿色，不把名称集合或一次隔离通过冒充严格同源码逐项零新增证明**。主检出已合入742，与745树生产源不是严格同源码对照；verification.md旧节的“crates同源/零新增”表述应在T07修订时明确历史与证据限度。

没有另跑tf/taa/tt/tb/tu/docs_gen专项，没有改生产源码来消红。check/fmt证明修复轮无新增warning/格式问题；未发现调试打印、源码VM/A2R回退、fake Checked入口、未经批准运算/类型扩面或外仓实现。helper-probe和bundle-probe是target内review探针，不是产品改动；原样源码、Cargo JSON绑定和二进制指纹都保存供重放。独立Bundle探针首次编译需补Cargo真实dependency及native linked_paths，最终成功结果只绑定Cargo JSON当前产物，不用失败探针结果评价产品。

supplemental.ps1沿用第一轮的坏助手**参考顺序**作为历史对照；其中poll-before-drain控制不是对当前common助手的原样调用。当前助手的证据以helper-probe为准；本轮大trace正控制20000字符完整返回（183ms）。

## 规范增量与返工

冻结SD01..04保持有限profile/类型/Char边界，Schema身份、pass状态和GOAL-017不变。SD02真实S0→S2能力有证据，SD04仅profile宽度/open保持；SD01/03完整source支持与过程合同未通过，待修后更新报告/重新冻结。人工层当前未被修改，正确盘点不要求重扫重hash；impact元数据仍是三项modify、一项add，touched_goals空因为不重开原目标。

原R1-QA02/04/07/08明确清偿，其它具体修复证据保留，本轮R2-QA01..05另行登记，不把旧全部八项假称完全未修。Plan回`executing`；T02..07（含受影响测试/健康/SD新鲜证据）重开，T01数值调查保留完成，**current_step=1 / total_steps=9**；T08/T09仍未完成。r1语义合同未修订。

下一责任方为work，在同一`plan-745-dev`树修复并提交，再绑定新完整HEAD申请独立最终复审。五项是合同内缺口；登记债务不构成批准延后。本轮不merge/archive/清理原树。
