# PLAN-745 r1 第三轮最终独立复审：needs_fix

复审实现：`7437eb137a789259f01446da7a170e74031ff7f2`；修复比较基线：`91b37560cffda75cf07cef8620cf524f5d6a3c7c`；计划原始代码基线：`eea7a80618d4d5e75749d666b7609a9a87925bbd`。
同一工作树 `D:/autostack/.wt/lang-745/auto-lang`，分支 `plan-745-dev`；组内只读 auto-down 仍为 `fba6563ed2148ce85e68208863159b4ccccac710`。
review_scope=final；review_round=3；plan_revision=1。原规划/前两轮复审 Codex 上下文独立于 GLM 实现与修复上下文；只审查、运行验证、记录证据，没有修改实现、委派、合入或发布 canonical Specs/live ledger。
入口主检出为 `e755c0e8b9cf502476903d5cd2627aa7e61c050a`。入口 Plan、Spec、配置及冻结 delta 哈希见 [baseline.json](baseline.json)；最终复核与证据指纹见 final-baseline.json、artifact-manifest.json。

## 上轮五项的本轮处置

| R2 finding | 独立验证 | 当前处置 |
|---|---|---|
| QA-01 合法跨行括号后续算误拒 | D01..05 check/build 通过，native 分别 5/6/1/6/2；C01/C03/D06 仍拒绝 | 已清偿 |
| QA-02 进程/采集截止与回收 | Rust 改为约 505ms 超时返回，UTF-8 lossy 正确；但后代仍存活且读线程被 detach。PS 5.1 约 415ms 就报告成功，后代仍存活 | 部分修复；R3-QA-01 |
| QA-03 Stray 组合遮住真实错误 | D10/D13 都有 loop-target-scope，break/continue 正确为 1:27/1:42；D08/D14 合法终结后死语句位置保留；C09/C10 位置不退化 | 已清偿 |
| QA-04 补充 oracle 不足 | 永久测试新增同类型越域 Bundle 变异与三件真文件 publish 回滚；本轮完整测试通过，独立同类型变异经真实 verifier 报 uninitialized-read | 已清偿 |
| QA-05 不闭合输入栈溢出 | E02 现在 exit1/source.syntax，无栈溢出；深不闭合块也受控。新护栏误拒合法小输入，同时另一合法表达式仍崩溃 | 原 E02 已修；R3-QA-02 |

R1 的符号隔离、关键词/排除分类、inventory 调用、统一构建尾部等清偿状态保持，不重新挂已修问题。原 C01..22、D01..14、E01..02 全部重新回放：[counterexamples.json](counterexamples.json)、[extra-results.json](extra-results.json)、[depth-results.json](depth-results.json)。C13/C21/D09 仅作既有保守语义调查，不新增验收合同。

## P745-R3-QA-01 / P2：返回有界仍未完成进程生命周期与输出收集

关联 R2-QA-02；AC-05/06/08，T-05/T-07。合同 §5.4/§6 要求保留 741 的受控启动、子树回收、单一截止、完整输出及错误传播，不允许用脱离读线程换取有界返回。

Rust `tests/common/mod.rs:205–304` 正常 spawn，没有 suspended→Job→resume；`_out_reader`/`_err_reader` 的 JoinHandle 被丢弃。超时只 kill/wait 直接子进程，错误被丢弃；`try_wait()?` 的错误出口也缺完整生命周期收口。读线程阻塞在 read_to_end 时，丢弃通道接收器不会中断 OS 管道读取；注释声称读线程会自行结束不成立。调用者还未统一把 drain_error 作为失败条件消费。

原样提取当前 helper 编译的独立探针：父进程立刻退出，后代继承两个管道并等待 5 秒。500ms 截止下约 **505ms** 返回 `timed_out=true/code=0/stdout=""`，紧接着查询证明 **后代仍存活**。合法输出 `DESCENDANT_PID=...` 已写但缓冲未完成，超时结果也无法取到其前缀；两个读线程仍持管道直到后代结束。探针随后只清理其明确记录 PID 的自有后代。无效 UTF-8 的前缀/替换字符/后缀控制已通过，不否认这一修复。

PowerShell `scripts/verify-ac-source-745.ps1:32–114` 用 250ms 静默窗口代替管道 EOF。相同持管道后代，Windows PowerShell 5.1 原样函数在约 **415ms** 返回 `Code=0/TimedOut=false`，后代仍存活；后代预定 5 秒后的输出被当作“已收集完整”而放弃。只有父 PID 的 taskkill 在父进程已经退出时不能弥补子树约束，正常出口没有子树清理。超时/异常分支还有无界 WaitForExit、外部 taskkill 等待和吞错路径。

证据：[helper-results.json](helper-results.json)、[helper-probe-exact.rs.txt](helper-probe-exact.rs.txt)、[script-helper-results.json](script-helper-results.json)、[script-helper-exact.ps1.txt](script-helper-exact.ps1.txt)。PS7 的同一探针也出现约 442ms 假成功，但判定采用声明支持的 PS5.1 结果。

修复要求：所有 cargo/support/CLI/PE 测试入口消费符合合同的同一执行器；首次运行指令前约束子树，约束失败受控拒绝；一个 deadline 覆盖进程及实际 EOF 收集；成功、超时、spawn/containment/wait/read 错误都回收自有子树和读线程，错误向验收传播。补充“父先退出+后代持管道/延迟写”“父超时+后代”“约束失败”“无效 UTF-8”“大双管道”控制，断言返回后无遗留进程/线程及无截断成功。单纯延长静默时间、detach reader 或仅杀父进程不能满足要求。

## P745-R3-QA-02 / P2：深输入护栏既漏掉实际崩溃路径，又静默缩小支持边界

关联 R2-QA-05 的修复后续；AC-02/03/07/08，T-02/T-03/T-05/T-06/T-07。

`src/source/parser.rs:68–77,211–222` 新增 `NEST_LIMIT=200`，按内部 expr/unary/block 调用计数，未覆盖左结合表达式形成的深树及后续处理。实际 CLI 输入：

```text
"fn main() int { return " + "1+" × 16384 + "0 }"
```

全部为合同内合法 i32 加法；只需 **32794 字节**。check-source 两次实测退出 **-1073741571**，stderr 为 `has overflowed its stack`，没有受控诊断。4096 项合法加法链 exit0；16384 项缺 RHS 和同规模 bool 错类型控制则分别得到正常 source.syntax/source.type-mismatch，因此不能把深度问题视为全面已解决，也不能把所有深输入一概归为语法错误。没有要求任意规模性能或程序终止承诺，当前发现是这个具体支持内输入导致进程崩溃。

另一个修复回归：99 层完整合法括号只占 **224 字节**，却 exit1，在 1:123 报 `source.syntax: nesting deeper than 200 levels is not accepted`。这是完整合法语法；护栏数字按实现帧计数，甚至不是宣称的 200 层语法。Plan r1、README、冻结 delta 都没有这个支持上限，代码“not a support-set reduction”的注释不能授权边界变更。

证据：[new-results.json](new-results.json)，F01..09 及各自完整 input.at/result.json；[new-probes.ps1](new-probes.ps1) 可重复。E02 不闭合括号及 F05 不闭合块确已受控，不重新声称其原始崩溃仍存在。

修复要求：检查 AST 构造、解析、解析后遍历/发射/verify 以及 Drop 的实际递归路径，消除该合法链的崩溃；合法小括号控制通过，不闭合等输入仍受控拒绝。资源防护必须与输入结构和实际处理路径一致；若确需改变 profile 支持上限，明确走合同修订及相应 Spec/正反验收，不静默以 source.syntax 拒绝合法程序，也不仅扩大线程栈或按现有语料最大深度规避问题。保留最小反例及边界两侧永久回归。

## 门禁与独立验收

| 验证 | 当前 HEAD 结果 | 证据 |
|---|---|---|
| ac-core locked check all-targets / fmt | 0 / 0，零编译 warning | ac-check / ac-fmt 日志 |
| ac-core 全测试 | **104 passed / 1 ignored**，exit0 | ac-test.stdout/stderr.txt |
| 741 一键回归 -SkipMainGates | 13 步通过 | verify741.stdout/stderr.txt |
| 745 一键回归（计划原命令） | **12 步 / 74 例通过，exit0**；首次驱动结果因下述环境注入作废 | verify745-canonical.stdout/stderr.txt、gate-corrections.json |
| 独立 74 例驱动 | **74/74**：21 正例真实 PE + 53 拒绝；build/check/run 输出、三件 hash 全记录 | corpus-results.json、corpus/**/result.json |
| strict inventory | 47 人工决定新鲜，9 输入、146 unknown 候选闭环，exit0 | inventory.stdout/stderr.txt |
| 同类型 Bundle 越域负控制 | 唯一 Cargo JSON 绑定的当前 rlib；有效 control 通过，改变 read local 后真实 verify.uninitialized-read | bundle-results.json、cargo-artifacts.jsonl |
| source SENTINEL / 大输出 | 坏源、坏链接、错入口均三件 SHA256 不变；20000 字节 trace 完整，exit0；真 publish 回滚永久测试通过 | supplemental-results.json、ac-test.stdout.txt |
| root check | PASS，既有生产 warning；未修改生产 crates/config/lock | root-check.stderr.txt |
| cargo t --no-fail-fast | 4972 run：4944 passed、27 failed、1 timed out、1534 skipped，exit100 | root-t 日志、daily-results.txt |
| cargo tv --no-fail-fast | 162 run：158 passed、4 failed、6330 skipped，exit100 | root-tv 日志 |
| 日常新出现 dep018 timeout 隔离 | 1/1 通过，exit0 | root-scoped.stderr.txt |

日常 27 处失败都在前两轮历史红集合中；新出现 dep018 超时隔离通过。生产源码、测试配置、Cargo lock 相对原代码基线无 745 diff；四处 tv semver 红保持。归因范围见 daily-attribution.json；主检出已经含 742 等并发变更，**没有宣称严格同源码 main 对照或根门禁全绿**。没有触发 tf/taa/tt/tb/tu/docs_gen 的单独门禁。

复审驱动更正：首轮 gates.ps1 通过 .NET ProcessStartInfo 启动 Windows PowerShell 5.1，继承 PS7 的 PSModulePath，导致 Get-FileHash 不可用；同样继承环境的复跑/预导入/预调用对照没有改变问题。ps-environment-control.json 证明仅清除继承 PSModulePath 后哈希 control 由 exit1 变为 exit0，原生 powershell 调用亦正确。驱动已修正，按 Plan 原命令重跑。**这项是 reviewer harness 环境错误，不是实施缺陷**；相关失败日志保留注明作废，不用独立74例结果冒称原脚本通过。

## AC 与规范增量裁定

| AC | 状态 | 理由 |
|---|---|---|
| AC-01 | pass | 数值报告、范围/类型、真实 trap 与宽度检查；完整 int/char 仍 open |
| AC-02 | fail | 已声明支持集中的深表达式崩溃、合法括号被未批准上限拒绝 |
| AC-03 | partial | bind→投影→真实verify及三类变异通过；完整支持内 source→Checked 健壮性尚有 QA-02 |
| AC-04 | pass | 53 负例永久 span/token 测试、C/D 与 UTF8/CRLF；Stray 原位置修复 |
| AC-05 | partial | 21 PE/独立 C/D、ba/trap 正确；执行器生命周期和采集完整性失败 |
| AC-06 | partial | profile/entry/0-1-2、共享尾部、旧741事务及三件回滚通过；进程合同未满足 |
| AC-07 | partial | strict inventory 新鲜、SD01..04/full-width/char 边界正确；护栏与提案支持边界尚不一致 |
| AC-08 | fail | 两项合同内 P2，无法最终 pass；本轮逐项审计与绑定证据已完成 |
| AC-09 | pending | merge 所有权，未执行 |

冻结 delta 为 [proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，与本轮工作树提案相同。SD02 的阶段身份/Checked唯一、SD04 的 int/char 限域结论没有新增问题；SD01/03 的支持声明须在 QA-02 修好后重新绑定实际实现。没有发布 canonical 或重写人工 evidence hash，touched_goals=[] 的“不重开 GOAL-017”依据保持。

## 下一步

needs_fix → work 在**同一工作树**修复两项并提交新完整 HEAD，然后重新最终复审。T-02/T-03/T-05/T-06/T-07 及依赖证据重开；T-01 数值报告、T-04 共享 Checked 构建/CLI/事务保留通过，current_step=2/9，status=executing。T-08/T-09 保持未完成。r1 合同暂不改；若确需资源边界合同变化，则显式申请 new 修订，而非实施方自行弱化。
T09 主线整合、规范/人工层/ledger/index/导航、到期批回归、真实 archive 断言和 guard/树清理仍待最终 pass 与合入授权；本轮不清理专属树。
