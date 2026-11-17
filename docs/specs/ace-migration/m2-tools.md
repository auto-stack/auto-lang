# ace-migration: m2-tools — 调试LSP测试与Playground接管

> Status: **A期PROOF切片已交付**（PLAN-971 B-PROOF-I/V/R，2026-11-16 phase fold；
> lang `e1a600ebe`、auto-ui `2f70bbf7`）；**B期按§5命名能力面已交付**（PLAN-971 r8，
> 2026-11-17；交付 lang `a9d20f01f`、auto-ui `95bc1689`，前置 M2A-08=985 §9
> `95c7fbad8`）。B 交付面＝编译期检查/发现闸门/源码级调试/LSP 诊断接管/Playground
> notebook；**脚本式 Playground run/debug 前门与 debug/test UI 消费者未迁**（具名后继，
> 见下文 B 边界）。本文件只记录**已验证**的支持、接口、拒绝与边界；未列入的能力不视为
> 已支持。

## A期已验接口（显式ACE多文件工程往返，协议能力）

> 本节按 A 切片交付时（2026-11-16）的事实记录；B 交付后替代的条目（测试发现、LSP 分析后端、
> 调试面）以「B期已验接口」为准，A 文本保留以便追溯。

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 中立项目契约 | `auto_lang_core::ace_entry` 项目面：`AceModuleSpec{package,version,id,source}`、`AceProjectRequest{modules,entry_module,entry_def_id,tests,capabilities,wall,cancel}`、`run_ace_project`/`run_ace_project_named`、`install_ace_project_executor`（单属主、同指针幂等、`shutdown_ace_project_executor` 释放）、`AceProjectReport{engine,outcome,tests,exit_code,diag_count,wall_ms}`；未注册=具名 `ace.host/project-executor-unregistered`，未知与全部旧引擎名=`ace.engine/unknown` 零dispatch（永不切旧VM）；`PACKAGES_CAPABILITY`/`packages_capabilities()` 便捷面 | `crates/auto-lang-core/src/ace_entry.rs` |
| 项目结果分态 | `AceProjectOutcome{Success{value,steps}, Diagnostic{module=原文件`pkg::mod`身份, stage, code, message, span=模块内原span}, Denied, ResourceDenied, Cancelled, HostFailure}`；协议级 `exit_code`=0 iff entry成功且全部请求测试passed——**测试失败永不退出0** | 同上 |
| 参考项目宿主 | `auto_lang_ace_host::project::ProjectHost`（`project_host()` 单例；`install()` 并装项目宿主，E0 `shutdown()` 语义不变、不触碰项目注册表）；消费960包面 `compile_package_product`→`PackageAvTarget::cold_with_entry`；`PackageDiagnostic{module,原span}`（binder已归位到模块内）直映中立契约 | `crates/auto-lang-ace-host/src/project.rs` |
| 世代预算/取消 | 每世代 `ExecLimits.wall` 封顶为请求**剩余**预算（F-971-R-01；单测 `p971_generation_wall_is_the_requests_remaining_budget` 钉死，非静默2s默认），请求取消旗标透传 `execute_program`（AV in-run 轮询）；相位界检查（编译前/每测试前/entry前）cancel/wall；每请求/每测试 scratch store 于 temp 建立、Drop 回收（停止回收可观察：运行后无 `ace-p971-project-*` 残留） | 同上 |
| 测试车道（A合同） | 请求显式携带测试名（A期发现留 legacy `test_runner::collect_tests`——AST walk `f.is_test`；B期T-02迁发现进producer）；`fn <name>() int`：完成且返回0=passed；非零返回/trap/链接失败=failed（带module+原因）；跨模块重名=ambiguous failed；未知名=not-found failed（module空）；每测试独立世代+独立store（隔离） | 同上＋fixture |
| LSP 真 client/CLI 面 | `auto-lsp` bin（stdio JSON-RPC）：didOpen坏源→`publishDiagnostics` 定位**该文档**（单行源 line0、字符界内）；全量 didChange 修复→诊断清空（150ms debounce）；未知方法=JSON-RPC error；shutdown(params=null)→null result，exit→进程码0；非LSP进程（错binary）=客户端具名失败不挂起。**分析后端仍 legacy auto-lang-core**（T-08/B 迁移，本fixture锁协议行为） | `auto-ui/crates/auto-lsp/tests/lsp_cli_roundtrip_971.rs` |
| 永久验证 | lang：`auto-lang-ace-host/tests/p971_project_proof.rs`（真实 app+math 跨模块 `eval(2,3,4)`=2+3*4=14、math坏源诊断定位 `math::core` 原span含 `nosuch`、修复后消失、bad_check/trap_check/missing_check→exit_code=1、cancel→Cancelled+无scratch残留、1ms墙spin拒绝、旧引擎名4种具名拒、合法近邻复跑14）＋core单测（registry/engine纪律+世代墙接线） | 固定组 `cargo t ace-facade`（28例）；`cargo test -p auto-lsp`（全包） |

## B期已验接口（PLAN-971 r8，2026-11-17；lang `a9d20f01f` / auto-ui `95bc1689`）

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 编译期项目检查面 | `auto_lang_core::ace_entry`：`AceCheckRequest{modules,entry_module,capabilities,cancel}` → `AceCheckReport{engine,outcome,declarations,tests,wall_ms}`；`AceCheckOutcome::{Clean,Diagnostics(Vec<AceDiagnostic>),Cancelled,ResourceDenied,Denied,HostFailure}`；`AceDiagnostic{module=原文件`pkg::mod`,stage,code,message,span=模块内原span}` 保留**每个**诊断；`AceDeclaration{name,module,symbol,params,ret,span}` 为 producer 声明表。`check_ace_project(_named)`；执行器 `check_project` 默认具名 `ace.host/check-unsupported`，未注册=`ace.host/project-executor-unregistered`，旧引擎=`ace.engine/unknown` 零 dispatch。**只编译不执行**（trap 入口仍 Clean 已验） | `crates/auto-lang-core/src/ace_entry.rs`；`crates/auto-lang-ace-host/tests/p971_tool_face.rs` |
| 发现闸门 | `run_ace_project_tests(_named)`：先检查→发现→按发现名在项目道执行；`AceTestRunOutcome::{Passed,Failed{message},NoTestsDiscovered{message},Diagnostic,Denied,ResourceDenied,Cancelled,HostFailure}`；**零发现=具名 NoTestsDiscovered 且 exit 1**；执行结果必须恰好覆盖每个发现名（缺/改名=`ace.test/results-mismatch`，exit 1，绝无真空 PASS）；测试失败/超时（墙）/编译拒绝分别是具名终态 | 同上 |
| 发现契约（冻结） | 零参数且声明返回 `int` 的声明；排除入口 `main`；合成 `__init` 跳过。ACE 源面**无 `#[test]` 注解**，发现基于 producer 声明事实（非旧 AST walk）。**已验后果**：库模块里合法的零参 `int` helper 也会被发现并执行（非零返回即整轮失败）——契约边界的测例在 `p971_tool_face.rs`，语言侧收窄/属性面归 D-971-B-01 | 同上 |
| 源码级调试面 | `AceBreakpoint{module,symbol,span:Option<AceSpan>}`（`span=None`=函数首指令；`Some`=与该指令原源 span 相交）、`AceDebugCommand::{Continue,Step,StepOver,StepOut}`、`AceDebugStop::{Entry,Breakpoint{index},Step,StepOver,StepOut,Finished}`、`AceDebugSnapshot{stop,frames(外层→内层, module/function/span/locals),steps,outcome,message}`、`AceDebugSession{snapshot,resume,set_breakpoints,cancel,close}`、`start_ace_debug(_named)`；宿主 `start_debug` 默认具名 `ace.debug/unsupported`。停点/栈/locals 来自**真实 AV 暂停链**：`PausedRun::debug_position/debug_snapshot/frame_depth` + `execute_program_steps_after_init`（模块初始化先跑一次后停在入口首指令）。局部名取自同一 lowering 世代的 `PackageAvTarget::slot_names`（ABC 线不序列化槽名）；平面 MIR span 按模块段起点回映为模块内原 span。`cancel` 是**会话存活期**的停止句柄（start 请求的旗标只是初始状态）；`close` 回收 scratch（无 `ace-p971-project-*` 残留） | `crates/auto-lang-ace-host/src/debug.rs`；`tests/p971_debug_face.rs` |
| LSP 诊断接管 | `auto-lsp` 诊断后端＝`check_ace_project`（编译期、多文档），legacy `Parser` 通道**整体删除**（无回退）；单合成包名取工作区根目录名，模块 id=相对路径去扩展名、分隔符压平为 `_`（嵌套 `lib/util.at`→`lib_util`）；诊断按 `module` 落到所属文档并用该文档自身文本把模块内字节 span 换为 UTF-16 范围（钳制）；**V-08**：内容世代快照丢弃过期结果、发布携带被检内容版本、乱序 didChange 丢弃、~150ms debounce；模块集=**已打开文档**（didClose 导入件后 importer 下次检查报未解析导入，已验）；诊断仅 ERROR（旧 warning 无 ACE 等价面）；非 Windows 发具名 `ace.host/unavailable` | `auto-ui/crates/auto-lsp/src/{diagnostics,backend}.rs`；`tests/lsp_ace_diagnostics_971.rs` |
| 既存 LSP 能力核验（未重写分析器） | 真 bin 用例核验：跨**包路径** goto（`use pac.lib.util` + `lib/util.at`）落原模块原声明位置（line0/char3）、hover 有符号信息、`workspace/symbol` 列出、`semanticTokens/full` 非空、completion 有响应、signatureHelp 有内容；inlay 仅“请求被接受”（缺口 D-971-B-05）。修复 **F-971-B-01**：Windows 下跨文件 goto 目标 URI/文档路径改为平台正确 URL 转换，定义位置由目标源文按 `fn/type/enum/const/spec <name>` 推导（原实现 goto 恒 null、位置取不可用 fragment span） | 同上＋`tests/lsp_protocol_test.rs` |
| Playground notebook 接管 | 每个 notebook 会话＝一个 `ace_session`（PLAN-970 中立合同）会话，进程级安装一次（`ace_runtime::ensure_session_host`，`cfg(windows)`；非 Windows 走具名不可用路径）；单元格 `AceSessionOp::Eval`，结果/诊断取报告，inspector=carry 表+累积源派生定义清单，销毁即 `Close`，**旧 AutoVM REPL 已删除（无回退）**。两用户会话结构性隔离（V-04）；**console 边界**：session 车道无 console，`print` 单元格=具名 producer 诊断+空 stdout（D-971-B-02） | `crates/auto-playground/src/{ace_runtime.rs,notebook/mod.rs}` |
| 永久验证 | lang：`p971_tool_face.rs`（2 例：检查面事实/坏源定位/修复/只编译不执行；发现闸门零发现/通过/失败/超时/编译诊断/引擎拒/helper 假阳性）、`p971_debug_face.rs`（1 例：跨模块 2 帧栈/locals/原源 span/单步/step-over/step-out/span 断点/运行期断点/live cancel/死循环墙/热更不复用旧 span/无残留）、`p971_project_proof.rs`（A 面复跑）；core `ace_entry` 纪律单测；auto-ui：`lsp_ace_diagnostics_971.rs`（2 例真 bin：多文档/版本/乱序/didClose/能力核验）、`lsp_protocol_test.rs`、`lsp_cli_roundtrip_971.rs`。固定组 `cargo t ace-facade auto-playground`（32/16）、`cargo test -p auto-lsp`（26） | 见左列路径 |

## B期边界（当前开放项）

- **Playground 脚本式 run/debug 前门**（`code_runner`/`vm_runner`/`agent_debug`/`debugger`）未迁：包/项目道拒绝表达式语句（`source.unsupported`）、console 仅独立单模块 `host-console` profile，而现有 demo 是脚本式源；需语言面扩展（D-971-B-03）。
- **debug/test 工具面的生产 UI 消费者**：`start_ace_debug`/`run_ace_project_tests` 目前只有参考宿主+永久 fixture 消费者（D-971-B-04，随 r8 收窄迁后继）。
- 发现契约的 helper 假阳性（D-971-B-01）、notebook 无 console（D-971-B-02）、LSP inlay 内容未断言（D-971-B-05）、编辑器宿主 E2E 未跑（D-971-B-06）均见 `docs/plans/KNOWN-DEBT-AND-RISKS.md`。
- acworker/1 线按冻结**不扩展**（一致性 adapter 非产品入口；多文件产品入口=项目面）。
- 平台：ACE host 仅 Windows x64（其他平台具名失败）；UI 同。

## A期边界（现值，A切片交付时的开放项；其中已被B关闭或改判的见 B期边界）

- 仅协议能力：断点/step/栈/locals、运行内步级取消粒度、`#[test]` 发现迁ACE、LSP分析后端换ACE、Playground接管、多文件工程经 acworker/1 线——全部归B期（T-02/03/08；T-01冻结记录在 PLAN-971 §5）。
- 编译域=960包面 TextRecord 裁剪：`while` 不在面内（条件循环=`for <cond> { }`）；`let` 局部不可变（可变局部=`var`）；入口=`pkg::mod` 模块键＋`d_<sym>` def，entry模块 `main` 保持符号 `main`。
- run期 trap 的 span 为 AV `SpanInfo` 映射（E0 同映，stage=`run`），module 身份为空——**源级**诊断才携带 module/原span；B期 source-map 义务不因A切片扩张。
- 端到端墙判别受冻结步预算（1M instr）与编译时长限制：世代内墙**接线**由单测判别；端到端负控（1ms墙spin）证明拒绝不挂起，不为修复提供独立判别（如实记录）。
- `ac_protocol.rs`（acworker/1 镜像）无漂移不改；项目面不经 case-worker 进程（一致性adapter面非产品入口）。
- 平台：Windows x64（桥 crate Windows-only）；UI 同。
