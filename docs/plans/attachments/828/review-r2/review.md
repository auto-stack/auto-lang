# PLAN-828 r2 修复复审 R2

2026-10-20 ｜ `review_scope: final` ｜ **outcome: needs_fix**。

本次用户提出修复后检查；实际只有 `repair r1a` 局部修复。主计划仍为 `executing 0/11`，最新 work 记录明确 `outcome: in_progress`，并列出 F01/02/04/05b/10/11/12 等“继续修复，不得送审”。因此本轮执行**就绪核查、原例重放及受影响接缝的相邻控制**，不重启完整最终验收，也不将 admitted gaps 误判通过。

## 冻结基线

- 实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev`，受审 `791c62ab251b5d0f36d3d5610fade600760d2f52`，修复diff base `940445bf57674780bc6ff6c68928c0c7148a0dbf`；原合同基面 `f79b1b83591310b45b3828dd300e6d368987d6c0`。本轮修复9文件，395 additions/127 deletions。
- 主计划读取自主检出，revision2，输入SHA256 `39c0f485966f0a1687434374aa4c094dbf596263776b5cb167123c074eff7db1`；主检出入口HEAD `d1d22ba2eb45dacef37ce262b4b8a04468a7829b`。未改语义合同。
- reviewer：本聊天 Codex `/root`，与R1相同独立审查上下文，未实施这些修复；未创建代理/其他聊天，不声明额外模型/会话身份。实施树检查前后clean，未写实现。
- auto-down仍为 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未消费。配置、依赖锁、执行测试二进制、主Spec/Design、skill与输入哈希见 [baseline.json.txt](baseline.json.txt)。主检出其他WIP原样保留。
- Spec delta从当前主计划重新提取，[冻结表](spec-delta-input.md) SHA256仍为 `d2cf61b3dd120e7e08edd9997877618df6e31fec0199a2effc9762cbfc42f383`，与R1相同；没有以文档追认必需面延期，canonical/ledger未写。

## 原始执行与证据

cwd均为实施树；本目录的 review harness 由原R1源码复制，仅把两处 `credential_current(&cf, cf.context_revision())` 适配为修复后的单参数API，并增加4项相邻控制。原始7个反例的源码输入和期望未降低。

| 验证 | 命令/方法 | 实际结果 |
|---|---|---|
| 永久修复例 | `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 6/6 pass，0 ignored，0.10s |
| 原828 suite | `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | 17/17 pass，0 ignored，12.51s |
| 独立原例+相邻控制 | `powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-r2/run-replay.ps1` | **8 pass / 3 fail**，exit101，0.20s；[源码](replay.rs)、[raw](replay-output.txt) |
| 真实脚本正反控 | `powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-r2/script-controls.ps1` | exit0；frontend确实执行2个旧前端例、1个递归修复例、6个scalar差分；真实零选择与Cargo错误均被拒。[脚本](script-controls.ps1)、[raw](script-output.txt) |
| 格式 | `rustfmt --check --edition 2021 --config skip_children=true` 后接 function_code/program_snapshot/session/formal_828/function_queries 五模块 | exit0，原格式子项关闭 |
| diff | `git diff --check 940445bf5 HEAD` | exit0 |

不能将重复scope中相同测试相加为不同验收覆盖。原7例现**6 pass / 1 fail**；新增4项控制**2 pass / 2 fail**。互递归编辑保持旧snapshot结果控制、changed函数自身含trace的合法控制通过。

存在一项**本次修复引入的新告警**：`probe/src/session.rs:228` 的 `context_revision` 字段从外部参数核验改为FnEnv自证后不再读取，触发dead_code。它与既有 `shim-metadata/src/classify.rs:213` unused doc comment区分，后者不是本计划新问题。新告警计入F14剩余健康项。

没有运行整档t/tv/tt/tb/tu/taa/tf或docs_gen；本轮不做正式性能/内存完整验收。现有进程内17例中的P95测试不能替代仍未实现的正式门。

## 14项关闭表

`CLOSED`仅指该finding的具体不变式，其他finding仍可影响同一任务。本轮关闭3项、部分修复5项、其余6项仍OPEN；不是“14项均完成”。

| R1 finding | 当前状态 | 代码/本轮证据与剩余边界 |
|---|---|---|
| F-828-R1-01 | OPEN | session仍组装全部stub并 `compile_program_parts`，整Program resolve/typecheck/Atom render-bind路径未替换；计数仍未绑定实际未改函数工作。work也自认未完成 |
| F-828-R1-02 | OPEN | ProgramSnapshot仍在每次link克隆全部blocks/slots/spans后经merged派发；本次只有排版变化，未改变粒度 |
| F-828-R1-03 | **CLOSED** | self-call现经 `fn:<self>` import。原非首位递归体编辑3→6通过；新增互递归体编辑4→6、冷参考6、pinned旧snapshot仍4通过 |
| F-828-R1-04 | OPEN | 原13B前部插入反例仍返回旧trap `70..75`，新源冷编为`83..88`；相对span/snapshot映射未实现 |
| F-828-R1-05 | PARTIAL | 原跨会话候选已被私有mint session_id拒绝，B仍revision0/值102；取消/关闭态仍不存在且work明确开放。不能把cross-session例通过写成整个F05关闭 |
| F-828-R1-06 | PARTIAL | 原调用者字符串伪造环境例已闭；现由FnEnv自持fingerprint并比owner。但新相邻**Core/Scalar域切换反例失败**，完整语义上下文适用性未闭，见下文 |
| F-828-R1-07 | **CLOSED** | restore现在同步current与rec.code/history；原真实store恢复后编辑main Arc身份保持。此结论不替代F02的实际派发payload、F04来源或F10资源/GC合同 |
| F-828-R1-08 | PARTIAL | intrinsic启动旧例通过；intrinsic在changed函数内的热编辑通过。但intrinsic在未改callee内、修改caller时仍拒绝，旧标量支持面未完整保留，见下文 |
| F-828-R1-09 | PARTIAL | warmup对超4MiB原例正确拒绝；新增front_door也接prepare。prepare的完整新源准入位于snapshot编辑/增量parse/全文物化之后，且缺typed资源传播、32MiB驻留/候选/旧代预算及失败释放证据；不能据冷启动一例宣称整个资源门关闭 |
| F-828-R1-10 | OPEN | 无函数HIR A07 codec/producer/恢复消费；实际body依赖探针、store GC/lease/Weak释放水位等仍明确待做 |
| F-828-R1-11 | OPEN | commit仍 `persist_all_units` 遍历并clone/encode/put所有函数；manifest仍未驱动恢复/引用保活 |
| F-828-R1-12 | OPEN | 正式recipe/持续worker/父侧judge、30样本raw和双线内存及真实退化反控尚未接线；coverage-manifest仍写batch/后续修订，原合同不允许这样延期 |
| F-828-R1-13 | **CLOSED** | 参数数组和ac-syntax manifest已纠正；frontend真实正控过，零匹配确实抛错，Cargo无目标返回101确实传播。本轮未重复7Scope全部测试，不把work总数当本轮执行 |
| F-828-R1-14 | PARTIAL | 原fmt子项已过，主计划也保持executing；contract/coverage/successor现状重整与具体SD仍未交付，并新增session dead_code告警 |

### F06完整不变式未闭：Scalar凭证可流入Core环境

AC-05/08，V-03/04，T-03/05/09；沿用R1 F06，不增加新的产品支持要求。

独立例 `credential_cannot_cross_hir_domains` 使用相同源码和`same.at`，分别经真实 `compile_scalar_source` / `compile_source` 生成各自通过域验证的bundle，确认domain不同。Scalar bundle铸造CheckedFunction，在自身FnEnv判current为true（合法控制）；放入Core FnEnv仍为true，应拒绝。

原因：`FnEnv`没有保存HirDomain/授权等适用轴，`credential_current`只比def id、owner及`declaration_context_fingerprint`；fingerprint只来自module声明，不包含bundle domain。CheckedFunction虽携domain，消费门未读取它。修复已消除“外部传入旧字符串”的原例，仍未封闭R1要求的实际环境适用性。需要把真实域及适用授权/版本事实纳入受检环境、核对原有准入面，并保留同域合法控制。

### F08完整不变式未闭：trace位于未改函数时正常caller体编辑被拒

AC-01/05/08，V-01/03/09，T-02/03/09；沿用R1 F08，属于既有标量和intrinsic支持，不是扩展新语法。

独立例 `trace_in_unchanged_function_survives_body_edit`：

```text
fn helper() int { return mark_a() }
fn main() int { return helper() + 1 }
fn tail() int { return 9 }
```

无trace授权启动必须拒绝（通过）；授权后原值2，冷编`main +2`得3（通过）。仅将main的`+1`改为`+2`，helper及调用依赖不变，却得到 `EditOutcome { ok:false, revision:0, diagnostics:["body credential minted under a different declaration context"] }`。

对照：trace在被编辑helper自身时，`mark_a()+1→mark_a()+2`可安装并得3。当前stub整Program只根据changed真实body重新派生intrinsic声明，未改helper的mark_a在stub中消失，环境fingerprint漂移，触发拒绝。需保存受检接口/固有函数环境，不能将所有未改body替换为空后据此推导整个环境；与F01的架构修复一起闭合，补未改callee/changed函数两路径及授权负控。

## AC/V/SD状态与覆盖边界

- AC01/02/04/05/07/08仍不通过；AC03/06有已验证局部能力，但不完整。AC09本次结论needs_fix；AC10保持merge-owned pending。
- V01有双函数和递归/互递归正控，仍缺真实粒度证明；V02原位置反例失败；V03/04新增domain反例失败，完整依赖/信任矩阵未达；V05已有快照可见性与本轮互递归旧snapshot控制过；V06跨会话原例闭，取消/关闭未交付；V07恢复身份闭，但GC/释放未交付；V08/09正式/资源与真实反控仍缺，脚本自身入口/零选择已修。
- SD01..06的原目标路径/impact保持。GOAL-002在`docs/specs/goals.md`有真实登记。SD03的递归缺陷修复、SD04的跨会话/恢复身份和SD06的脚本改进只能关闭相应子项；完整delta仍被F01/02/04/05/06/08/09/10/11/12阻断。未发布canonical或ledger，不把后继合同中的零前端/已完成声明当事实。
- 未修改的结构缺口通过当前修复diff和实际调用点复核；未复用旧测试来给变更后的完整路径授pass。7原例已重新执行；未来正式IPC、资源/GC和新前端/执行器若重写，必须提供新路径证据，不能借本轮原例通过继承完整验收。

## 下次就绪前的诊断与路由

这是第2次最终送审请求，R1/R2均因必需工作未完成而只进行了就绪/有界核查；不把本轮改名为internal以抹去失败轮次。下次ready前，work须提交诊断及覆盖调整，而非只报“原例6绿”。

| 剩余不变式 | 早期验证为何漏掉 | 路径/状态 | 必需调整 |
|---|---|---|---|
| 实际环境适用性 | 只测签名不同，未测同签名不同domain | public FnEnv/CheckedFunction、恢复/复用 | 输入适用轴→环境/凭证→消费门映射；同域正控/跨域负控，原例同时回放 |
| trace支持面不减 | 只测cold启动或trace就在changed body | trace在未改callee、caller体编辑 | 固有函数环境持久化；changed/unchanged两路径与未授权反控 |
| 真正函数级前端/派发/持久化 | 计数挂在caller封装、只看code Arc、formal绕开store | 热源编辑→HIR→实际dispatch及store | 完成F01/02/11，计数/identity绑定真实生产与消费；强制实际全量必须红 |
| 来源/寿命/正式资源 | 测changed span、map条数和进程内耗时 | 未改代码位移、旧调用、失败/关闭、GC、正式worker | 完成F04/05b/09/10/12及原计划的完整oracle，不能转batch/未来 |

保持 **executing 0/11**，全部work任务完整断言仍受未闭finding影响；保留已闭子项和历史正控证据，不要求重做已验证且未受后续修改影响的内容。原树完成剩余实现、诊断/覆盖、具体SD及健康检查并提交，再申请最终复审。之后才由merge执行新冻结提交上的裸t+tv、实际触面/独立workspace/consumer门、Spec/ledger/归档/guard；本轮未修实施、未合入或清理。
