---
plan_id: PLAN-971
status: archived
feature_name: 调试LSP测试与Playground接管
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-17
plan_revision: 8
current_step: 11
total_steps: 11
depends_on: [PLAN-970, PLAN-966]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-tools.md]
touched_goals: [GOAL-002]
---

# [PLAN-971] 调试LSP测试与Playground接管

## 0. 变更摘要

2026-11-17 r8（用户裁定，范围口径收窄，**不新增能力/不改仓库或动作范围**）：用户选择 **B 口径** 收口 AC-03——本包 B 的“真实消费者”义务按§5命名能力面收口（LSP 诊断、Playground notebook、工具面 producer 能力），把 Playground 脚本式 run/debug 前门与 debug/test UI 消费者**迁具名后继**（D-971-B-03/04）。依据：ACE debug 会话与发现测试车道目前只有参考宿主+永久 fixture 消费者；脚本前门接管需先扩展语言面（项目/包道表达式语句＋console/脚本形态，实测 `source.unsupported`），属语言计划范畴。本修订只改 AC-03 消费者措辞与任务勾选/去向登记；原 AC/V/任务 ID 保留，T-06/T-07 merge 门不变。

2026-11-17 r7 后续（/goal 完成计划971/B 执行轮，规划文字仍 r7）：M2A-08 已由 985 A 批 phase fold 正式成立（985 §9 merge 收据 delivery `95c7fbad8`）→ B 批按§5阶段合同解锁并开工。本轮交付 B 的**producer 工具面**与两个真实消费者切片，证据见§5「B期执行记录」与§8任务行：编译期项目检查面（全量诊断/声明表/测试候选）、测试发现闸门（零发现=具名失败）、源码级调试面（断点/step/step-over/step-out/栈/locals/原源span/热更不复用旧span）、LSP 诊断换 ACE 后端（多文档+版本见证）、Playground notebook 接管 ACE 会话（两用户隔离）。B 未闭项具名（LSP 其余能力/跨包 goto/编辑器客户端、acworker 线按冻结不扩、Playground 其余路由）→ 本包保持 executing，不做全包 review/归档。

2026-11-16 r7（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包保全部通用诊断/LSP/debug/test及现行编辑器行为；Godot转译/场景运行专属接管不再是A/B出口前置。编辑器现用文件浏览/语法高亮等公共功能不因排除Godot目标而删除。A/PROOF原正式证据不变，B仍等M2A-08。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

迁#[test]发现/隔离、auto-case adapter、debug breakpoint/step/stack/locals、LSP来源/诊断与Playground notebook；调试停点和源码映射来自新事实与执行器。语言测试执行和外部认证协议分开。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-ui/crates/auto-lsp/src`（Spec PLAN-900与本轮磁盘核定实际owner）；`crates/auto-tool-protocol/src`仅为中立协议，不代LSP实现。
- `crates/auto-playground/src`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/test_runner.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/vm/debugger.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-16 goal 实施授权（r6）

用户以 `/goal 完成计划971` 授权本链（auto-plan-goal 范式）：在专属 worktree 实施（含建树）、复审、批准合同内有界修复与 merge（含 phase fold A切片）。不授权改目标/验收阈值/兼容承诺/仓与动作范围/预算。B 期开工仍受 §5 阶段合同 M2A-08 前置约束（该约束是本计划已批准的合同内容，不是本链新增限制）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：LSP/调试/测试/Playground接管。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-ui/crates/auto-lsp/src](../../../auto-ui/crates/auto-lsp/src)：LSP真实owner，不在语言残留目录造新server；目录职责已核，确切符号/闭包开工再核

- [crates/auto-tool-protocol/src](../../crates/auto-tool-protocol/src)：共享协议/身份；目录职责已核，确切符号/闭包开工再核

- [crates/auto-playground/src](../../crates/auto-playground/src)：Playground真实执行入口；目录职责已核，确切符号/闭包开工再核

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

## 5. 详细设计

### M2-TARGET-SCOPE-20261116：本包当前边界

本包保全部通用诊断/LSP/debug/test及现行编辑器行为；Godot转译/场景运行专属接管不再是A/B出口前置。编辑器现用文件浏览/语法高亮等公共功能不因排除Godot目标而删除。A/PROOF原正式证据不变，B仍等M2A-08。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A/PROOF仅等966/969/970的实际接口；原T-02/03/08的全工具迁移属B并消费M2A-08收据。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-PROOF-I, B-PROOF-V, B-PROOF-R |
| M2-B | T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


迁#[test]发现/隔离、auto-case adapter、debug breakpoint/step/stack/locals、LSP来源/诊断与Playground notebook；调试停点和源码映射来自新事实与执行器。语言测试执行和外部认证协议分开。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：真实LSP owner与工具全入口（r2）

LSP已迁auto-ui/crates/auto-lsp，本包直接核其现存completion/hover/signature/semantic tokens/goto/inlay/workspace能力及编辑器客户端；将多文件诊断、测试发现和调试源码映射分批验收。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。


### A期PROOF批次（新增、未实施）

最小前置：L966, L969, L970。实际改面：`crates/auto-lang-core/src/test_cases/ac_protocol.rs`；`D:/autostack/auto-ui/crates/auto-lsp`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

### B期执行记录（2026-11-17，goal 执行轮；M2A-08 已成立）

前置核实：985 A 批 phase fold 落主（985 §9 merge 收据 delivery `95c7fbad8`，「M2A-08正式成立」）→ 按§5阶段表解锁 B 批；同组两仓树保留（lang `D:/autostack/.wt/lang-971/auto-lang` @ plan-971-dev，基面 `2f1eed27f`；auto-ui `D:/autostack/.wt/lang-971/auto-ui` @ plan-971-dev，基面 `c1561d94`），无 junction/symlink；本批代码提交：lang `f0984d6b8`/`f02d44bf8`/`67793f6cb`/`f6dac410b`/`a983e2a23`，auto-ui `3c9cfe30`。

1. **编译期项目检查面（T-02；LSP/发现的共同地基）**：`ace_entry::check_ace_project`（`AceCheckRequest{modules,entry_module,capabilities,cancel}` → `AceCheckReport{engine,outcome,declarations,tests,wall_ms}`）。`AceCheckOutcome::Diagnostics(Vec<AceDiagnostic>)` 保留**每个**诊断的原模块身份（`pkg::mod`）与模块内原 span；`Clean` 时发布 binder 的**声明表**（原名/模块/符号/参数数/渲染返回类型/模块内 span）与测试候选。宿主侧 `check_project` 在 960 包面**只编译不执行**；执行器未实现检查面=具名 `ace.host/check-unsupported`，未注册=具名 `ace.host/project-executor-unregistered`，旧引擎名=`ace.engine/unknown` 零 dispatch。证据 `crates/auto-lang-ace-host/tests/p971_tool_face.rs` 用例1（2 例合计 0.23s）：声明表事实、跨模块坏源定位原文件原 span 且切片含 `nosuch`、修复后检查干净、trap 入口仍 check 干净（证明不执行）、双坏模块诊断各自落在自身文本。
2. **测试发现闸门（T-02/V-03）**：`run_ace_project_tests`/`_named` = 先检查再按**发现名**在项目道执行（每测试独立世代+独立 scratch），`AceTestRunOutcome::{Passed,Failed,NoTestsDiscovered,Diagnostic,Denied,ResourceDenied,Cancelled,HostFailure}`。**零发现=具名 `NoTestsDiscovered` 且 exit 1，绝不成 PASS**。发现契约（本轮冻结）：零参数且声明返回 `int` 的声明、排除入口 `main`；ACE 源码面**无 `#[test]` 注解**（legacy 属性不在该源面），故发现读 producer 声明事实，不做名字猜测/旧 AST walk。证据同上用例2：零发现具名失败、通过=exit 0、失败具名归因、超时（50ms墙+死循环测试）=该测试 failed+资源终态且不为 PASS、编译拒绝带模块身份、旧引擎拒。
3. **源码级调试面（T-02/V-01/V-05）**：av-core 暴露真实暂停链事实 `PausedRun::debug_position`（函数/模块/原源 span/是否函数入口）、`debug_snapshot`（外层→内层调用栈 + 每帧原 span + named slot 值）、`frame_depth`，并新增 `execute_program_steps_after_init`（模块初始化先跑一次，随后停在入口**首指令**）；`ace_entry` 中立调试合同 `AceBreakpoint{module,symbol,span(Option)}`/`AceDebugCommand::{Continue,Step,StepOver,StepOut}`/`AceDebugStop::{Entry,Breakpoint{index},Step,StepOver,StepOut,Finished}`/`AceDebugSession`/`start_ace_debug(_named)`，项目执行器 `start_debug` 默认具名 `ace.debug/unsupported`；参考宿主 `auto-lang-ace-host/src/debug.rs` 实现函数入口断点（仅首指令）与 span 交集断点、四命令、每步 wall/cancel、结束态携带真实 `AceProjectOutcome`、close 回收 scratch。局部变量名取自**同一 lowering 世代**的 `PackageAvTarget::slot_names`（ABC 线不序列化槽名），平面 MIR span 按模块段起点回映为模块内原 span。证据 `tests/p971_debug_face.rs`（1 例 0.40s）：跨模块 2 帧栈、局部 `x=I32(2)`、断点 span 切片 `math::core` 原文、调用者帧切片 `app::main` 调用点、单步推进真实指令计数、step-out 回调用者、continue 到真实值 `3`、span 断点、**热更后新会话 span 不复用旧值**（V-08 腿）、旧引擎拒、坏工程启动失败带原模块诊断、关闭后无 `ace-p971-project-*` 残留。
4. **LSP 诊断接管（T-08 切片，auto-ui）**：`auto-lsp` 诊断后端由 legacy `Parser` 换成 `check_ace_project`（编译期、多文档；单合成包名取工作区根目录名，模块 id=相对路径去扩展名并把分隔符压平为 `_`）；诊断按模块身份落到所属文档，用**该文档自身文本**把模块内字节 span 换成 UTF-16 范围并钳制；V-08：按文档内容世代快照丢弃过期结果、发布携带被检内容的 LSP 版本、乱序 didChange 丢弃、保留 ~150ms debounce；legacy 诊断通道整体删除（**无回退**）。证据 `crates/auto-lsp/tests/lsp_ace_diagnostics_971.rs`（真 `auto-lsp` bin stdio JSON-RPC，4.57s）：双文档干净打开零诊断、util 坏源只报 util 且范围切中 `nosuch`、修复清空、main 坏源只报 main、v3 坏/v4 好连发后最后发布为 v4 空且旧结果不覆盖、shutdown exit 0；`cargo test -p auto-lsp` 25 例全绿（实施会话与 owner 复跑一致）。
6. **LSP 既存能力面核验＋跨包 goto 修复（T-08 第二批，auto-ui `9a352b8a`）**：真二进制下核验 `completion/hover/signature/semanticTokens/goto/inlay/workspace` 的既有实现（本包不重写其分析器）：新增 `lsp_ace_diagnostics_971` 第二用例（1.08s）——跨包 goto 落到被导入模块**自己的源文件**且位置精确（line 0 / character 3，指向 `hello` 声明本身）、hover 给出该符号信息、`workspace/symbol` 列出该符号、`semanticTokens/full` 非空、completion 在调用点有响应、shutdown exit 0。**发现并修复 F-971-B-01**（同树修复、合同内）：跨包 goto 在 Windows 上从未生效——文档路径用 `strip_prefix("file://")` 留下 `/D:/…`、目标 URI 用字符串拼 `file://C:\…`，目标 URI 解析失败使 goto 静默返回 null；且数据库 fragment span 不是 LSP 位置（名字在 col 3 却给 col 26）。修复：`document_uri_to_path`/`path_string_to_uri` 走 URL 平台正确转换；定义位置由目标文件自身文本按 `fn/type/enum/const/spec + 名字` token 推导，退化时才用记录 span。证据：上述用例 + `cargo test -p auto-lsp` 26 例全绿（14 lib + 4 integration + 2 ACE fixture + 2 CLI + 4 protocol）。
7. **编辑器客户端（T-08 的“编辑器客户端”项）**：按§4“外仓只读调研”边界审计 `D:/autostack/auto-vscode/vscode-extension/src/extension.ts`（第14–16/218/229–245/258 行）：`vscode-languageclient` 以 `auto-lsp-wrapper.cmd`（Windows）spawn 服务器、`documentSelector=[{scheme:'file',language:'auto'}]`、标准 stdio `TransportKind`——客户端接线与服务器真实二进制协议面一致。该仓无自动化 editor-host harness（无 test 目录/脚本），本环境无 VS Code 运行器 → **编辑器宿主内端到端未执行（不记 PASS，具名归编辑器仓）**；本包职责面（auto-lsp 服务器）已由真二进制用例核验。
8. **调试会话运行期断点（T-02 补充，lang `586362bb7`）**：`AceDebugSession::set_breakpoints` 允许在会话存续期替换断点集（调试 UI 在入口停点后再装断点）；参考宿主直接替换匹配表。永久用例新增腿：以空断点集启动→入口停点后装入 callee 入口断点→Continue 命中 `Breakpoint{0}` 并停在 callee（`p971_debug_face.rs` 0.54s）。
9. **Playground 脚本前门的语言面阻断（实测）**：见§5 B 边界与§10 待裁定；本轮临时探针（已删）证实包道对 `print(...)` 表达式语句给 `source.unsupported`，故 run/debug 前门无法仅靠消费者接线接管。
10. **T-05 独立复审（2026-11-17）与修复**：独立子代理上下文按 §7 AC 逐项核验并重跑 fixture/组门，verdict=**needs_fix**（0 代码阻塞，11 findings + AC-03/§10 待裁定）。本轮已修：**B1** Playground 的 ACE 宿主安装改 `cfg(windows)`（非 Windows 走具名不可用路径而不是引用空 crate 编译断链）；**B2** `run_ace_project_tests` 要求执行结果集**恰好覆盖**每个发现名（缺/改名=具名 `ace.test/results-mismatch`+exit 1，堵“零结果也全过”的真空 PASS；core 新增 drop_results 腿）；**B3** notebook `print` 单元格=具名 producer 诊断+空 stdout，永久用例固定（`p971_notebook_console_boundary_is_named`）；**B4** debug 补齐 step-over（入口与 callee 内）、live cancel（新增 `AceDebugSession::cancel`，起点旗标只是初始状态）、确定式 wall（死循环源 50ms）三腿；**B5** tool_face §7 记录发现契约的 helper 假阳性真实行为（D-971-B-01）；**B7** debug 局部值截断改字符边界（`truncate_at_char_boundary`+单测）；**B6** 触碰文件 fmt 清理（av-core 既有 dispatch/sched 格式污点保持不动）。UI 侧：**B10** 跨包 goto 改用真实包路径（`use pac.lib.util` + `lib/util.at` 子目录）；**B8** 补“乱序 didChange 不发布”与“didClose 导入件后 importer 下次检查报未解析导入”两腿；**B11** signatureHelp 收紧为内容断言（inlay 内容缺口入 D-971-B-05）。修复提交：lang `2d781fe02`、ui `22532614`。AC-03/§10 仍待用户裁定（T-03 不勾）。
5. **Playground notebook 接管（T-03/V-04）**：notebook 会话=ACE session（970 中立 `ace_session` 合同），进程级安装一次（`ace_runtime::ensure_session_host`，guard 常驻）；单元格经 `AceSessionOp::Eval` 执行、结果/诊断取自报告，inspector=carry 表 + 累积源派生函数清单，销毁即 `Close`；**无旧 VM REPL 回退**。证据 `crates/auto-playground/src/notebook/mod.rs` 用例 `p971_v04_two_user_notebook_sessions_are_isolated`：两用户会话 id 不同、A 的变量在 B 不可解析（具名诊断且无值）、各自只观察自身 carry 表、inspector 互不含对方、关闭 A 后 B 仍可用；crate 15/15 绿。

**B 具名边界/未闭项（不得记 PASS，归后续 B 轮）**：

- LSP 其余能力（completion/hover/signature/semantic tokens/goto/inlay/workspace）**沿用既有分析器**（本包 T-08 的义务是“核”其能力，不是重写分析器）；本轮已用真二进制用例核验其可用且 goto 指向正确源（F-971-B-01 修复）。编辑器宿主（auto-vscode）端到端未跑（该仓无 harness、本环境无 VS Code）：只读审计接线一致，E2E 不记 PASS、归编辑器仓。
- LSP 模块集=已打开文档（不读磁盘 `.at`）；嵌套模块 id 被 packages `valid_id` 拒（`lib/util.at`→`lib_util`）；诊断仅 ERROR（旧 parser 的 warning 无 ACE 等价面），发布形态变化须在最终 Spec 记明；ACE host 仅 Windows（其他平台发具名 `ace.host/unavailable`）。
- ACE 源面无 `#[test]` 注解；会话 reshape **不接受**尾表达式函数体（旧 VM 接受），Playground/LSP 据实呈现 producer 具名诊断（语言面边界归 ACE 前门，不在消费者伪造）。
- acworker/1 线按 T-01 冻结决定**不扩展**（一致性 adapter 非产品入口；扩线翻倍改面且不增证明力）；多文件产品入口=项目面。
- Playground 脚本式 run/debug 前门（`code_runner`/`vm_runner`/`agent_debug`/`debugger` 路由）**实测被语言面阻断**（非仅协议问题）：本包冻结的 960 包道/项目检查面拒绝表达式语句（`check_ace_project("fn main() int { print(\"hi\")\n return 0 }")` → `stage:text, code:source.unsupported, \"expression statements are not part of this source profile…\"`），且 console 只在独立的单模块 `host-console` profile 开放；而 Playground 现有 demo/前端是脚本式源（顶层 `let`/`print`、尾表达式、`use a: b` 冒号导入，如 `examples/playground-demo/10-geometry/main.at`）。因此这些前门要接管 ACE 需要**语言面扩展**（项目/包道的表达式语句与 console、或脚本形态进入多模块车道），属语言计划范畴；本包§5 点名的 Playground 义务是 notebook（已交）。是否把该扩展计入 B 完成条件见§10。
  [证据 2026-11-17] 上述探针（同轮临时用例，已删）与 `check_ace_project` 实际诊断；Playground 的 notebook 会话（970 ace_session）不受影响（V-04 已绿）。

### T-01 A期冻结记录（2026-11-16，goal授权执行，r6）

前置核实：966/969/970 均已归档（A最小前置满足）；M2A-08 未闭（985 r15 drafting 0/14）→ 按§5阶段表只启动 M2-A 批，B 期任务保持登记不实施。worktree：`D:/autostack/.wt/lang-971/auto-lang`（分支 `plan-971-dev`，基面 master@858bbc271）+ 同组兄弟 `D:/autostack/.wt/lang-971/auto-ui`（分支 `plan-971-dev`，基面 auto-ui master@f2b03dcb）；无 junction/symlink。

**策略比较与选型**：现有三条显式入口——(a) acworker/1 case-worker（进程边界、单源、check/build/run，无多文件/测试面）；(b) 969 E0 facade `run_ace`（同步一次执行、单源）；(c) 970 session daemon（REPL int/bool）。多文件执行域已存在于 960 包面（`compile_package_product`→`PackageAvTarget::cold_with_entry`，`PackageDiagnostic{module,原span}`、AV core `CancelFlag`）。选型：**沿 (b) facade 在 `auto-lang-core::ace_entry` 补中立项目契约 `run_ace_project` + 独立项目executor注册表（镜像E0纪律：未注册=具名HostFailure、零legacy dispatch）**，参考宿主在 `auto-lang-ace-host` 消费960包面；不扩 acworker/1 线（acworker为一致性adapter面非产品入口，扩线使改面翻倍且不增证明力；ac_protocol.rs 无漂移不改——r5改面列表中该路径为规划期锚点，实际改面以本冻结为准）。断点/栈/locals 全量语义= T-02/V-01 的 B 义务；A 只交停止/回收协议能力（cancel→Cancelled+scratch store回收+后续运行不受影响）。

**冻结的A能力（具名PROOF `ace.project/1`）**：`AceModuleSpec{package,version,id,source}`；`AceProjectRequest{modules,entry,tests,capabilities,wall,cancel}`（entry=`pkg::mod`入口模块键+`d_<sym>`入口def；tests=显式测试名列表——A期发现留在现有test owner `test_runner::collect_tests`（legacy AST walk），B期T-02迁发现进producer）；结果 `AceProjectReport{outcome,tests,exit_code,wall_ms}`，`outcome∈{Success,Diagnostic{module=原文件`pkg::mod`身份+原span},Denied,ResourceDenied,Cancelled,HostFailure}`。测试合同：`fn <name>() int` 经包面执行，passed=完成且返回0；非零返回/trap/编译诊断=failed（带module+span）；协议级 `exit_code`=0 iff 全部passed（测试失败非零）。资源：每请求 wall 预算；scratch store 于 temp 下按请求nonce建立、RAII结束删除（停止回收）；cancel 在编译前/每测试前检查，运行内粒度=B期义务。

**V-A-PROOF 冻结命令/输入/平台**：lang=`cargo t ace-host ace-source auto-test-case`（主证明在 `crates/auto-lang-ace-host/tests/p971_project_proof.rs`：真实多文件工程 app+math 跨模块 2+3*4=14、坏源诊断定位原文件原span、修复后消失、测试失败 exit_code=1、cancel→Cancelled+回收、未知引擎名具名拒、合法近邻复跑；ace-source/auto-test-case 为未改消费面确认组）；UI=auto-ui 仓 `cargo test -p auto-lsp`（真CLI往返持久fixture `tests/lsp_cli_roundtrip_971.rs`：spawn 真实 `auto-lsp` bin，JSON-RPC/stdio initialize→didOpen坏源→publishDiagnostics定位该文件→didChange修复→诊断消失→shutdown；错binary负控=客户端对非LSP进程具名失败；LSP后端仍legacy，T-08/B 迁移，本fixture锁协议行为）。平台：Windows x64（桥crate Windows-only）。新增用例≤10s/条（test-discipline 2026-11-14）。

I：在现有诊断/source map、LSP与debug/test owner上，补ACE显式入口的一个真实多文件工程往返；值/诊断原span、断点/停止与测试失败退出正确；只交协议能力，不迁全部编辑器/Playground入口。
V：坏源诊断定位原文件，恢复后诊断消失；测试失败非零、未知协议/错binary拒绝、停止回收；合法近邻仍可执行。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：普通源2+3*4得14；改为未知名字应在原文件span诊断，修复恢复；断点step的变量值/栈与真实执行一致，工具只报engine标签不算接管。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** LSP/调试/Playground请求→公共969/970入口→ACE源图/源码映射→结构化协议响应/真实运行。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ToolSession { execution: SessionId, source_map: SourceMapId, revision: RevisionId }
struct DebugStop { frame: FrameId, location: SourceSpan, values: TypedSnapshot }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-ui/crates/auto-lsp/src](../../../auto-ui/crates/auto-lsp/src) | LSP真实owner，不在语言残留目录造新server |
| [crates/auto-tool-protocol/src](../../crates/auto-tool-protocol/src) | 共享协议/身份 |
| [crates/auto-playground/src](../../crates/auto-playground/src) | Playground真实执行入口 |

**状态、错误与资源。** 陈旧source-map/错revision请求、非法变量读取、执行失败或权限拒绝不降级旧VM；未验证/不可绑定断点按原协议明确答复，不擅自都改为异常，不把工具连接成功当业务PASS。 协议取消只关闭当前请求/会话，双编辑器隔离，close后停止后台/释放；A仅原PROOF，B全工具迁移不提前。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at) | 运行值与诊断合法邻例 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b7e5f25e46902be2` | 拟ace.m2.p971.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [auto-ui/crates/auto-lsp/tests/lsp_protocol_test.rs](../../../auto-ui/crates/auto-lsp/tests/lsp_protocol_test.rs) | 原协议功能/诊断位置 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `9f6f09093b94f341` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang-core/src/vm/debugger.rs](../../crates/auto-lang-core/src/vm/debugger.rs) | 旧step/变量/栈行为义务，不要求ACE保旧寄存器布局 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `59b35f438c442082` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 逐入口来源/协议版本/断点和变量语义、实际LSP/Playground测试命令与独立运行oracle冻结，保原工具全分母。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-tools.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-tools.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-tools.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-tools.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


LSP测试在auto-ui专属worktree按当地owner入口执行；语言仓仍留同名固定组不证明此仓拥有该package，禁止直接把该名字当有效语言仓测试腿。


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 断点实际停在当前源span，单步和locals准确 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 坏源码诊断后修复 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 零发现测试不能PASS | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | Playground两用户会话隔离 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | generated source回映到原源码而非旧AST | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t auto-playground auto-test-case ace-av integration`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### B期验证映射（2026-11-17，本轮实跑；命令/计数见§5「B期执行记录」）

| V行 | 本轮真实入口/用例 | 独立判据（实跑结果） | 未闭部分 |
|---|---|---|---|
| V-01 | `cargo test -p auto-lang-ace-host --test p971_debug_face`（1 例，修复后 ~1.0s plain / 1.29s nextest） | 断点停在 `math::core::bump` 原源 span（切片落在 callee body 内，body-offset 断言）、调用栈 2 帧（main→bump）、局部 `x=I32(2)`、单步推进指令计数、**step-over（入口与 callee 内）**、step-out 回调用者、continue 到真实值 3、span 断点、**运行期 `set_breakpoints`**、**live `cancel` → Cancelled**、**死循环源 50ms → ResourceDenied(wall)** | 无 console 捕获（session 车道无 console，D-971-B-02）；端到端墙由确定式死循环源承担 |
| V-02 | 同组 `p971_tool_face`（坏源→修复）＋ auto-ui `lsp_ace_diagnostics_971`（真 bin：坏源只报所属文件、修复清空） | 诊断落在原模块/原 span，修复后消失；跨模块/双坏模块各自落位 | LSP 仅发 ERROR（warning 无 ACE 等价面） |
| V-03 | `p971_tool_face` 用例2 零发现腿 | `NoTestsDiscovered`＋exit 1，零发现绝不 PASS；发现=零参 int 声明（入口排除） | 无 `#[test]` 注解面（ACE 源面事实） |
| V-04 | `cargo t auto-playground`（16 例，含 `p971_v04_two_user_notebook_sessions_are_isolated` 与 console 边界例） | 两用户会话 id 不同、A 变量在 B 不可解析、各自 carry 表、关闭 A 不影响 B；print 单元格=具名 producer 诊断（D-971-B-02） | Playground 脚本式 run/debug 前门被语言面阻断（D-971-B-03） |
| V-05 | `p971_debug_face` 断点/调用者帧 span＋`p971_tool_face` 诊断 span | 停点/诊断 span 均切中**原模块原文**（`app::main` 调用点 `bump(a)`、`math::core` 表达式），非旧 AST/生成文本 | 生成式 UI 目标的源回映归 UI 波次 |
| V-08 | auto-ui `lsp_ace_diagnostics_971`（2 例，修复后 6.74s 合计）＋`p971_debug_face` 热更腿 | 旧版本诊断不覆盖新版本（v3 坏/v4 好后仅 v4 空发布；**乱序 v2 不发布**）；**didClose 导入件后 importer 下次检查报未解析导入**；热更后停点 span 不复用旧值；真实包路径 goto（`pac.lib.util` + `lib/util.at`）落 line0/char3 | LSP 无 test-run 请求面（测试失败/超时/零发现区分由 lang 工具面 `p971_tool_face` 取证）；inlay 内容断言缺口 D-971-B-05 |

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 经真实stdio LSP发送open/change/close，旧版本诊断不得覆盖新版本；跳转跨包仍落正确源；测试失败/超时/零发现区分；调试热更停点不能指向旧span。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |


### A期PROOF验证（本轮未运行）

| Case | AC | 实际/拟补入口 | 独立oracle |
|---|---|---|---|
| V-A-PROOF | AC-A-PROOF | lang: cargo t auto-test-case ace-source ace-host；UI LSP使用其既有Cargo target，I读取Cargo与原测试固定完整命令，真客户端/CLI往返补持久fixture | 坏源诊断定位原文件，恢复后诊断消失；测试失败非零、未知协议/错binary拒绝、停止回收；合法近邻仍可执行。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 普通源2+3*4得14；改为未知名字应在原文件span诊断，修复恢复；断点step的变量值/栈与真实执行一致，工具只报engine标签不算接管。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 陈旧source-map/错revision请求、非法变量读取、执行失败或权限拒绝不降级旧VM；未验证/不可绑定断点按原协议明确答复，不擅自都改为异常，不把工具连接成功当业务PASS。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 协议取消只关闭当前请求/会话，双编辑器隔离，close后停止后台/释放；A仅原PROOF，B全工具迁移不提前。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | **r8 收窄（用户裁定）**：§5命名能力面的真实consumer实际接线、无legacy fallback——LSP诊断（ACE检查面，旧parser通道已删）与Playground notebook（ACE session，旧REPL已删）；工具面（ACE debug会话、发现测试车道）以参考宿主+永久fixture交付、生产UI消费者按 D-971-B-04 迁具名后继；Playground脚本式run/debug前门按 D-971-B-03 迁语言面后继。变化/失败/恢复/权限与旧支持格在该口径内保持 | T-03/04；源与producer身份及真实dispatch证据；后继去向见债务登记 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 真实LSP owner与工具全入口的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-A-PROOF | work | 坏源诊断定位原文件，恢复后诊断消失；测试失败非零、未知协议/错binary拒绝、停止回收；合法近邻仍可执行。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共11任务，9完成（T-01/B-PROOF-I/B-PROOF-V/B-PROOF-R/T-02/T-03/T-04/T-05/T-08）；余 T-06/T-07（merge/清理）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。
  [✅ 2026-11-16] 前置核实（966/969/970已归档、M2A-08未闭→只启动M2-A批）；A期冻结记录见§5"T-01 A期冻结记录"；goal授权链=本轮 /goal 完成计划971。策略比较（acworker线 vs facade线 vs 960包面承载）与选型、冻结命令/平台/资源合同已记录；worktree `D:/autostack/.wt/lang-971/{auto-lang,auto-ui}` 已建。

- [x] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：在现有诊断/source map、LSP与debug/test owner上，补ACE显式入口的一个真实多文件工程往返；值/诊断原span、断点/停止与测试失败退出正确；只交协议能力，不迁全部编辑器/Playground入口。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。
  [✅ 2026-11-16] 按§5冻结实现：中立项目契约 `ace_entry::run_ace_project`（`AceModuleSpec/AceProjectRequest/AceProjectReport{outcome,tests,exit_code}`、独立project executor注册表、未注册/锁毒=具名HostFailure、未知与旧引擎名=具名拒零dispatch；纪律测试 `p971_project_face_freezes_engine_and_registry_discipline` 0.00s 绿）；参考宿主 `auto-lang-ace-host/src/project.rs`（960包面 `compile_package_product`→`PackageAvTarget::cold_with_entry`，`PackageDiagnostic{module,原span}` 直映，测试车道逐名隔离世代+协议exit_code，相位界cancel，per-request scratch RAII 回收）；`install()` 并装项目宿主（E0 `shutdown()` 语义不变）。永久fixture：lang `tests/p971_project_proof.rs`（真实 app+math 跨模块 2+3*4=14、坏源诊断定位 math::core 原span含nosuch、修复消失、测试失败 exit_code=1、cancel→Cancelled+temp无 `ace-p971-project-*` 残留、旧引擎名拒、合法近邻复跑；1 test 1.09s 绿）＋ core单测 1例；UI `auto-ui/crates/auto-lsp/tests/lsp_cli_roundtrip_971.rs`（真 `auto-lsp` bin stdio JSON-RPC：initialize→didOpen坏源→publishDiagnostics定位该文档界内→didChange修复→诊断消失→未知方法JSON-RPC error→shutdown/exit码0；错binary（非LSP进程）=客户端具名失败不挂起；2 tests 0.17s 绿）。组内补 detached 依赖兄弟 `.wt/lang-971/{auto-web@0adebc0,auto-down@5af11a3}`（auto-ui workspace 路径解析需要，零改动）。

- [x] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录坏源诊断定位原文件，恢复后诊断消失；测试失败非零、未知协议/错binary拒绝、停止回收；合法近邻仍可执行。 核子终态与身份，未执行不PASS。
  [✅ 2026-11-16] oracle逐项有真实结果（p971_project_proof 1.09s 绿：math::core原span含nosuch、修复消失、bad_check/trap_check/missing_check→exit_code=1、cancel→Cancelled+temp无 `ace-p971-project-*` 残留、旧引擎名4种具名拒、合法近邻复跑14；UI 2例0.17s绿：坏源→publishDiagnostics定位该文档界内→修复清空、未知方法JSON-RPC error、shutdown/exit码0、错binary具名失败）。**组纠正（T-01冻结修正）**：`ace-host` 组实为 experimental/ace 的 ace-conformance（非桥组）；桥组=`ace-facade`。V-A-PROOF 实际执行：lang worktree `cargo t ace-facade`=PASS 28例（60.9s 含冷编译）、`cargo t ace-source`=PASS 597、`cargo t auto-test-case`=PASS 151（后两者与ace-host同跑，总13m38s）、定向 `cargo test -p auto-lang-core --lib ace_entry`=5绿0.00s；UI worktree `cargo test -p auto-lsp`=16例全绿（10 lib+4 protocol+2 新CLI，12.7s 含构建）。新增用例均≤10s（1.09/0.17/0.00s）。**基线既有红（与本批无构造关联）**：`cargo t ace-host`=164例中11红16skip；复放证据①governance document_base_cli_list 期望68行实得97（文档语料冻结期望漂移）；②ace_m1_closures_source_suite_green 期望Pass实Fail（155.7s复放）；worktree `experimental/`与`test/cases`与基面858bbc271逐字节一致（git status空），主工作区仅本批4文件且experimental无依赖路径→红为先存基线问题，登记债务候选，不判本批。

- [x] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。
  [✅ 2026-11-16] phase复审（局限：实施会话内复审，从提交工件重建）发现 F-971-R-01（世代内丢请求wall/cancel，弱于E0纪律）与 F-971-R-02（LSP行断言偏弱），均已修复并复验闭合（详见§9复审收据：needs_fix→修复 lang e1a600ebe/ui 2f70bbf7→re-pass，相位门 ace-facade 28例18.0s + UI全包绿）。AC-A-PROOF 六断言逐项pass（映射见§9）。阶段fold已正式合入：两仓 ff-only，delivery lang `2f1eed27f`（range-diff全等重写映射 a2faad900→a1b78d9b4 / e1a600ebe→82d659bba / fb838e3da→2f1eed27f）、auto-ui `c1561d94`（a6065a59→b156de4e / 2f70bbf7→c1561d94）；canonical `docs/specs/ace-migration/m2-tools.md` sha256前16 `c69d0afd1aa1fdf3` 已在 master；同树保留至B。可供985/A消费（M2A-08仍未闭，消费时按985自身门核本收据）。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [x] **T-02｜M2-B｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。
  [✅ 2026-11-17] 依§5「B期执行记录」1–3 交付 producer 侧主能力与永久正反例：编译期检查面（`check_ace_project`，全量诊断/声明表/测试候选，只编译不执行）＋发现闸门（`run_ace_project_tests`，零发现=具名失败 exit 1）＋源码级调试面（真实 AV 暂停链：断点/step/step-over/step-out/栈/locals/原源 span/热更不复用旧 span）。冻结：发现契约=零参 `int` 声明（入口 `main` 排除；ACE 源面无 `#[test]` 注解）；局部名取自同一 lowering 世代的 `slot_names`；平面 MIR span 按模块段起点回映。证据 `p971_tool_face.rs`（2 例 0.23s）＋`p971_debug_face.rs`（1 例 0.40s）＋core `ace_entry` 纪律单测；ace-facade 组 32 例绿。acworker/1 线按 T-01 冻结不扩（具名）。

- [x] **T-03｜M2-B｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。
  [✅ 2026-11-17 r8，用户裁定 B 口径] §5 命名能力面的真实消费者：LSP 诊断（ACE 检查面，旧 parser 通道删除，真 bin fixture 4.57s→6.74s/25→26 例）与 Playground notebook（ACE session，旧 REPL 删除，V-04 隔离+console 边界用例，crate 15→16 绿）；工具面（debug 会话/发现测试车道）以参考宿主+永久 fixture 交付。Playground 脚本式 run/debug 前门与 debug/test UI 消费者按 D-971-B-03/04 迁具名后继（§10 r8 裁定）。

- [x] **T-08｜M2-B｜owner_stage: work｜AC-06**：LSP已迁auto-ui/crates/auto-lsp，本包直接核其现存completion/hover/signature/semantic tokens/goto/inlay/workspace能力及编辑器客户端；将多文件诊断、测试发现和调试源码映射分批验收。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。
  [✅ 2026-11-17] 三批交付齐：多文件诊断换 ACE 检查面（V-08 版本见证/过期丢弃/乱序丢弃，auto-ui `3c9cfe30`）；测试发现迁 producer（lang `p971_tool_face` 零发现/通过/失败/超时区分）；调试源码映射（`p971_debug_face` 断点/栈/locals 原源 span + 热更不复用旧 span）。既存能力核验（auto-ui `9a352b8a`）：真二进制用例核 completion/hover/semanticTokens/goto(inlay/workspace 由既有 protocol 用例与 workspace/symbol 断言覆盖) 可用，跨包 goto 落原模块原声明位置；同时发现并修复 **F-971-B-01**（Windows 跨包 goto 目标 URI 构造非法 + 定义位置取自不可用 fragment span → 修复为平台正确 URI 转换与源文推导位置）。证据：`cargo test -p auto-lsp` 26 例全绿（2 例真 bin ACE fixture）；编辑器客户端按§4 只读边界审计 auto-vscode 接线一致，其宿主内 E2E 无 harness/环境未跑（具名归编辑器仓，不记 PASS）。

- [x] **T-04｜M2-B｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。
  [✅ 2026-11-17 r8] V行映射与实跑见§6「B期验证映射」（r8 口径收窄不改变实跑证据：所有 V 行均按§6 用例执行，唯一红均按基线 flake 具名不充 PASS）。职责组 `cargo t ace-facade auto-playground ace-source auto-test-case ace-av integration`（lang worktree @ `586362bb7`，514.7s）：ace-facade 32 PASS / auto-playground 15 PASS / ace-source 597 PASS / auto-test-case 151 PASS / ace-av 1247/1248（唯一红 `plan966_ct_product::p966_ct_generated_declarations_change_end_results`；**单独复跑 ace-av = PASS 1248，44.5s，gate_passed=true** → 负载 flake，非本轮触面；ace-av workspace 本轮无改动）/ integration 3354/3355（唯一红 `auto-lang tests::plan707_spike_tests::plan707_spike_cancel_gap_active_job_future_not_stopped`，连续第 3 个组跑同例，2.1–2.2s；单向复跑 3/3 绿；4f123a50e 宽限历史＋PLAN-868 §T-07 预存红族判定＋本批零 `crates/auto-lang` 触面 → 基线并行负载 flake，不判本批，如实记录不充 PASS）。UI `cargo test -p auto-lsp` 26 例全绿（含 2 例真 bin ACE fixture）；修复后复跑：ace-host 32 绿、playground 16 绿、core 5 绿、ace-facade 32 绿（第二轮复审独立重跑一致）。

- [x] **T-05｜M2-B｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。
  [✅ 2026-11-17 r8，用户裁定后收口] 独立复审第一轮（绑定 lang `586362bb7`/ui `9a352b8a`）verdict=**needs_fix**（0 代码级 blocker，11 findings + AC-03 §10 待裁定）；B1–B11 已修（lang `2d781fe02`、ui `22532614`）。第二轮独立复审（绑定 lang `2d781fe02`/ui `22532614`）verdict=**needs_fix（仅 B6 残项）**；B6 残项已修（lang `3753aeee6`），**第三轮定向确认（同一独立上下文）：B6 CLOSED**——全部计划触碰文件 `rustfmt --check` 0 diff（av-core 仅剩既有 dispatch/sched 污点），ace-host 32 绿、playground 16 绿，无断言弱化（仅删恒真断言，并收紧 console 边界要求 producer code）。**结论：B1–B11 全部 fixed、B6 定向确认 CLOSED、无断言弱化；§10 已由用户 r8 裁定（B 口径）→ 本任务收口（AC-04 通过；AC-03 按 r8 收窄口径成立，脚本前门/debug-test UI 消费者归 D-971-B-03/04）。**

- [x] **T-06｜M2-B｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [x] **T-07｜M2-B｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-971/auto-lang`，分支 `plan-971-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-971/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: merge | plan_id: PLAN-971 | plan_revision: r8 | outcome: **pass（B 批合入 + Spec 沉淀 + 归档 + 清树）** | delivery: lang master **`a9d20f01f`**（ff-only；含 resync merge `cd06e9d67`，base `7430b0c78`）／auto-ui master **`95bc1689`**（ff-only；resync `95bc1689`，base `50d31a4d`） | reviewed: T-05 两轮 needs_fix + B6 定向确认 CLOSED（绑定 `3753aeee6`/`22532614`）；r8 为 B6 之后的 resync/落地提交链（`cd06e9d67`+`a9d20f01f`） | resync: 两仓均以 master 合并入分支（lang 488 commits、ui 178 commits 落后；唯一冲突 `av-core/src/lib.rs` 导出表按并集解决＝master `execute_program_platform` ＋本批 debug 导出；ui `Cargo.lock` 取 master 后 `cargo check -p auto-lsp` 重生成）。落地检查（master 检出）：`cargo t ace-facade auto-playground` PASS（gate_passed=true）＋`cargo test -p auto-lang-core --lib ace_entry` 5 绿＋`cargo test -p auto-lsp` 26 绿（ACE fixture 6.78s）。注：`cargo test -p auto-lang-ace-host --lib --tests`（cargo 并行）在 resync 后撞到 master 侧 `p984_embed` 的**同进程全局宿主竞态**（master 检出同例亦红；nextest 与 `--test-threads=1` 均 4/4 绿）→ 属既有 cargo-test 并行假红，组门以 nextest 为准。 | spec deposit: `docs/specs/ace-migration/m2-tools.md`（B期已验接口+B期边界，绑定交付提交与 fixture 名）＋`docs/specs/ace-migration/replacement-scope-and-evidence.md`（§M2-B 工具面消费者接线：ACE 接线消费者/未迁具名路径/债务承接/SD-TARGET-SCOPE 无改动） | archive: `docs/plans/archive/971-ace-m2-tools.md`（status archived） | cleanup: 四 worktree（auto-lang/auto-ui/auto-down/auto-web）`wt-guard` 全部 clean 后 `git worktree remove` 成功；分支 `plan-971-dev` 两仓删除（was `a9d20f01f`/`95bc1689`）；组目录 `.wt/lang-971` 删除；`plan-artifacts/plan-971` 折入本记录后整目录删除（17 文件/412,705 B；`review-971-b/*.log` 与 fmt 基线副本为复审证据，已按本记录摘要化）；retention `policy.json` 追加 AUTO-LANG/PLAN-971 条目；无具名未解决持有（D-971-B-01..06 为后续计划债务，非本包持有） | next: done（本 Plan 终态 archived；B 期未迁面按 D-971-B-03/04/05/06 归后继）

stage: merge | plan_id: PLAN-971 | plan_revision: r8 | outcome: pass | T-06/T-07 收口记录: T-06=review pass 后核冻结并集与来源（上述 delivery/resync/spec/archive），T-07=命令实际输出逐目录对账（工作树 `target/test-groups/run-*`、`plan-artifacts/plan-971`、`.wt/lang-971` 三处输出均已处置：worktree target 随 worktree 删除、artifacts 折叠后删除、.wt 组目录删除；无第二暂存区/散落 planNNN-* 目录）；pending merge-owned 门=无（批量回归属周期门，非本 Plan 门）。

stage: goal | plan_id: PLAN-971 | plan_revision: r8 | round: 3（收口裁定） | from: user（§10 裁定：选项 B） | to: merge（T-06） | status: archived | outcome: pass（B 口径收窄落账；T-03/T-04/T-05 收口） | 记录: 用户选 B——本包 B 的“真实消费者”义务按§5命名能力面收口（LSP 诊断/Playground notebook 真实接线，无 legacy 回退；工具面以参考宿主+永久 fixture 交付），Playground 脚本式 run/debug 前门与 debug/test UI 消费者迁具名后继（D-971-B-03 language surface、D-971-B-04 consumer re-home）。r8 只改 AC-03 消费者措辞与去向登记，原 AC/V/任务 ID 与 merge 门不变；脚本前门/语言面未做、不记 PASS。当前已完成 9/11（T-01/B-PROOF-I/V/R/T-02/T-03/T-04/T-05/T-08），余 T-06 merge→T-07 清理 | next: merge（T-06：规范增量 SD-01/SD-PHASE/SD-DETAIL/SD-02/SD-TARGET-SCOPE 沉淀＋合回；T-07 输出对账/清树）

stage: goal | plan_id: PLAN-971 | plan_revision: r7 | round: 3 | from: review（T-05 两轮 + 定向确认） | to: user | status: archived | outcome: waiting（T-05 修复全闭，仅剩§10 AC-03/T-03 裁定） | worktree: lang `D:/autostack/.wt/lang-971/auto-lang` @ 3753aeee6（clean）/ auto-ui `.../auto-ui` @ 22532614（clean） | 记录: 第一轮 needs_fix→B1–B11 修复（lang 2d781fe02/ui 22532614）→第二轮 needs_fix（仅 B6 残项）→B6 修复（lang 3753aeee6）→第三轮定向确认 **B6 CLOSED**、全部计划触碰文件 fmt 0 diff、ace-host 32 绿、playground 16 绿、core 5 绿、auto-lsp 26 绿、ace-facade 32 绿；无断言弱化。**未闭且不可由本会话决定**：§10 的 AC-03/T-03 口径（A=脚本 run/debug 前门属 B 完成条件→需语言面计划+范围修订；B=按§5 命名义务收口→AC-03 消费者措辞收窄、脚本前门与 debug/test 工具消费者迁具名后继）。T-03 未勾、AC-03 未宣 PASS。 | next: user（§10 裁定）→ review 收口 T-05 → merge T-06/T-07

stage: review | review_scope: phase（B 批，T-05 第二轮） | reviewer_identity: 独立子代理上下文（第二轮，只读 + 跑测试） | plan_id: PLAN-971 | plan_revision: r7 | outcome: **needs_fix（仅 B6 残项）** | reviewed_commit: lang `2d781fe02` / ui `22532614` | 重跑: ace-host lib+tests **32 绿**（2 lib + 30 tests，debug fixture 1.01s/nextest 1.29s）、core `ace_entry` 5 绿、playground 16 绿、`cargo t ace-facade` 32 绿（gate_passed=true，21.9s）、`ace-av-core` render 单测 1 绿、`auto-lsp` 26 绿（ACE fixture 6.74s） | 判定: B1/B2/B3/B4/B5/B7/B8/B9/B10/B11 fixed；**B6 部分**（`project.rs:169` 与 `notebook/mod.rs` 计划新增 hunk 仍非 fmt-clean，且 §9 收据把 32 写成 30、§6 V-01/V-08 时间与腿未更新）；AC-02 达标（具名边界）、AC-03 **未满足**（debug/test 工具面仍只有 fixture 消费者，T-03 不勾）、AC-06 具名边界、AC-PHASE 通过 | next: work（B6 残项/收据更正→第三轮定向确认；§10 裁定后方可勾 T-03/T-05）

stage: work | PLAN-971 | r7 | round: 3（B6 残项修复） | outcome: pass | commit: lang `3753aeee6` | 记录: `project.rs` 与 `notebook/mod.rs` rustfmt 清零（后者的 10 处既有污点随同文件归一，避免半格式化文件）；`rustfmt --check` 对全部触碰文件为空；顺手收紧：删除 debug fixture 的恒真断言（body-offset 断言已承载 oracle）、notebook console 边界要求诊断携带 producer code（`[<code>]`）、`AceDebugSession::cancel` 文档写明“有停止按钮的宿主必须实现”。复跑: ace-host 32 绿（debug 0.99s）、playground 16 绿（1.02s）、core 5 绿。 | next: review（第三轮定向确认 B6）→ §10 裁定 → T-03/T-05

stage: review | review_scope: phase（B 批，T-05 第一轮） | reviewer_identity: 独立子代理上下文（与本会话分离，只读 + 跑测试） | plan_id: PLAN-971 | plan_revision: r7 | outcome: **needs_fix** | reviewed_commit: lang `586362bb7` / ui `9a352b8a` | base: lang `2f1eed27f` / ui `c1561d94` | 重跑: ace-host `p971_project_proof`+`p971_tool_face`+`p971_debug_face` 4 绿、core `ace_entry` 5 绿、`auto-playground` 15 绿、`cargo t ace-facade` 32 绿、`auto-lsp` 26 绿；A 收据/985 M2A-08 收据/两树 clean 均已核 | findings: F-1 非 Windows 编译被 Playground 无条件引用 ACE 宿主破坏；F-2 测试车道可“零结果真空 PASS”；F-3 notebook stdout/console 静默丢失；F-4 StepOver/cancel/wall 无证据；F-5 发现契约 helper 假阳性未具名；F-6 新代码 fmt 未清；F-7 debug 值截断可切多字节字符 panic；F-8 乱序 didChange/didClose 未验；F-9 V-05 内层 span 断言近真空；F-10 “跨包”goto 实为同目录跨文件；F-11 signature/inlay 只断言请求被接受 | AC 判定: AC-01 部分（B 面待 Spec 沉淀）/AC-02 具名边界/AC-03 **未满足**（debug 与 test 工具面只有 fixture 消费者；LSP 诊断与 notebook 已真实接线）/AC-06 具名边界/AC-A-PROOF 通过/AC-PHASE 通过 | next: work（B1–B11 修复→第二轮复审）

stage: work | PLAN-971 | r7 | round: 3（修复轮） | outcome: pass（第一轮复审 B1–B11 修复） | commit: lang `2d781fe02` / ui `22532614` | 记录: 见§5 项 10；修复后复跑：ace-host lib+tests **32 绿**（2 lib + 30 tests；fixture 0.58s/0.29s/1.00s/0.71s）、core ace_entry 5 绿、av-core 边界单测 1 绿、playground 16 绿（新增 console 边界例）、auto-lsp 26 绿（ACE fixture 6.63s，signature 内容断言通过）。未闭：AC-03/§10 待裁定；F-9 已修（内层 span 落在 callee body）、inlay 内容缺口入 D-971-B-05；fmt 仅触碰文件（av-core 既有污点不动） | next: review（第二轮独立复审，绑定修复提交）

stage: goal | plan_id: PLAN-971 | plan_revision: r7 | round: 3 | from: work | to: work | status: archived | outcome: pass（T-02 运行期断点补充＋T-04 收门证据；T-03 待§10 裁定） | worktree: D:/autostack/.wt/lang-971/auto-lang @ plan-971-dev（`586362bb7`，clean）/ D:/autostack/.wt/lang-971/auto-ui @ plan-971-dev（`9a352b8a`，clean） | 记录: (1) `AceDebugSession::set_breakpoints` 运行期断点集替换＋参考宿主实现；永久用例新腿（空断点启动→入口停点后装 callee 断点→Continue 命中）`p971_debug_face.rs` 0.54s 绿。(2) **语言面阻断实测**：`check_ace_project("fn main() int { print(\"hi\")\n return 0 }")` → `stage:text, source.unsupported`（表达式语句不在包面）；console 仅单模块 host-console profile；Playground demo 为脚本式源 → 脚本 run/debug 前门接管需语言面扩展，§5 B 边界与§10 待裁定记录。临时探针已删（不留证据于仓内）。(3) T-04 收门组跑（`586362bb7`，514.7s）：ace-facade 32 PASS / auto-playground 15 PASS / ace-source 597 PASS / auto-test-case 151 PASS / ace-av 1247/1248（唯一红 `plan966_ct_product::p966_ct_generated_declarations_change_end_results`，与本轮零触面——ace-av 独立 workspace 不依赖本轮改动文件；复跑判定见后续）/ integration 3354/3355（唯一红 `plan707_spike_cancel_gap_active_job_future_not_stopped`，连续第 3 次同例，已按§§ 预存红族判基线）/ UI `cargo test -p auto-lsp` 26 绿（4.66s+）。 | next: work（ace-av 复跑归因→T-03 §10 裁定→T-05 独立复审）

stage: goal | plan_id: PLAN-971 | plan_revision: r7 | round: 2 | from: work | to: work | status: archived | outcome: pass（T-08 闭：既存 LSP 能力真二进制核验 + 跨包 goto 修复；T-03 仍具名部分） | worktree: D:/autostack/.wt/lang-971/auto-lang @ plan-971-dev / D:/autostack/.wt/lang-971/auto-ui @ plan-971-dev | commit: auto-ui 9a352b8a；lang 本轮无代码改动（worktree clean @ a983e2a23） | 记录: (1) **F-971-B-01**（同树修复）：跨包 goto 在 Windows 从未生效——文档路径 `strip_prefix("file://")` 留 `/D:/…`、目标 URI 字符串拼 `file://C:\…` 解析失败→goto 静默 null；且定义位置取自不可用 fragment span（名字 col3 却给 col26）。修复为平台正确 URL 转换（`document_uri_to_path`/`path_string_to_uri`）＋从目标源文按 `fn/type/enum/const/spec <name>` token 推导位置。(2) 真二进制核验既存能力面（fixture 第二用例 1.08s）：跨包 goto 落原模块原声明位置（line0/char3）、hover 有符号信息、workspace/symbol 列出、semanticTokens 非空、completion 有响应、shutdown exit0；既存 protocol 用例覆盖 signature/inlay。(3) 编辑器客户端按§4 只读边界审计 auto-vscode 接线（wrapper spawn/`documentSelector auto`/stdio transport）；该仓无 harness、本环境无 VS Code → 宿主内 E2E 未跑，具名归编辑器仓不记 PASS。证据：`cargo test -p auto-lsp` 26 例全绿（14 lib + 4 integration + 2 ACE fixture + 2 CLI + 4 protocol）。T-03 未闭项在§5 B 边界与§8 T-03 行具名（Playground WS debugger/run 前门：步进 console host、WS/前端载荷、i32→类型化局部值协议） | next: work（T-03 余量评估/若判定前门迁移归后续波次则转 T-04 收门→T-05 独立复审）

stage: goal | plan_id: PLAN-971 | plan_revision: r7 | round: 1 | from: work | to: work | status: archived | outcome: pass（B批开工：producer 工具面＋两消费者切片；T-03/T-08 具名部分） | worktree: D:/autostack/.wt/lang-971/auto-lang @ plan-971-dev / D:/autostack/.wt/lang-971/auto-ui @ plan-971-dev | commit: lang f0984d6b8,f02d44bf8,67793f6cb,f6dac410b,a983e2a23 / ui 3c9cfe30 | 前置: 985 A批 phase fold 落主 95c7fbad8 → M2A-08 成立（按§5阶段表解锁B） | 记录: T-02 完成（检查面/发现闸门/调试面，§5 B期执行记录1–3）；T-03 部分（Playground notebook ACE 会话 V-04、LSP 诊断消费者）；T-08 部分（多文档诊断+V-08 版本腿、调试源码映射）；证据：`p971_tool_face` 2例0.23s、`p971_debug_face` 1例0.40s、auto-ui `lsp_ace_diagnostics_971` 1例4.57s、`cargo test -p auto-lsp` 25绿（owner复跑）、`cargo t ace-facade auto-playground` PASS（32/15例）；组门 `cargo t ace-facade auto-playground ace-source auto-test-case ace-av integration` 全跑 495.96s：ace-facade 32 PASS / auto-playground 15 PASS / ace-source 597 PASS / auto-test-case 151 PASS / ace-av 1248 PASS / integration 3355 例 3354 绿 1 红（`plan707_spike_cancel_gap_active_job_future_not_stopped` 2.107s）——**基线并行负载 flake，非本批**：同一 integration 组单独复跑仍红（181.02s，同例 2.213s，其余 3354 绿）；单向 `cargo test -p auto-lang --lib <name>` 3/3 绿；本批 diff 零 `crates/auto-lang` 触面（该测试所在 crate）；历史依据 4f123a50e 把该例宽限 300ms→2s 即为并行调度 flake，PLAN-868 §T-07 归档已把 `plan707_spike` 判为「AGENTS.md 预存红族、与本计划触面无关」，THR-D4 并行负载 flake 族在案。裸 `cargo test -p auto-lang --lib` 不是仓定入口（305 红为缺 nextest 配置/特性面），不作为判据 | 未闭具名: LSP 其余能力/跨包goto/编辑器客户端、Playground 其余路由、acworker线按冻结不扩 | next: work（T-04 组门收尾证据→T-03/T-08 余量评估）

2026-11-16 stage: new/revision | PLAN-971 r7 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

stage: goal | PLAN-971 | r6 | round: 25 | from: work(975) | to: work(975) | status: archived | outcome: pass(975 T-02 sqlite :memory:腿) | 记录: 967钉版provider以:memory:开启（零973依赖），独立oracle复核SUM=9，双负控（bb8d56e07 5.6s）；Auto API面=console foreign缝具名后续 | next: 975 redis RESP腿→T-03..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 24 | from: work(975) | to: work(975) | status: archived | outcome: pass(975 T-01冻结+T-02 JSON encode族) | 记录: 975开工——T-01三域选型冻结（JSON自有codec/967 FFI sqlite/RESP over 974 net）+973分账；T-02首增量 std.json encode族交付（bc19182d9，14 oracle 0.09s） | next: 975 sqlite :memory:/redis腿→T-03..07；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 23 | from: work(974) | to: work(依赖链) | status: archived | outcome: pass(**974整包归档——M2A-03标准库门获正式来源**) | 记录: PLAN-974 整包A交付完成：T-05复审pass(0阻塞)→T-06落主681efa26b+spec m2-stdlib-network sha16 a8338b81e6b5f0ee+归档7534f4eef→T-07清理(worktree/分支/plan-artifacts全删,guard clean)。M2A依赖链：**973/977PY/978/974已闭**；剩973(在途)/979(在途)/981(在途)/975/976/980(部分在途980已见)/982/983/984/OS064/977GD+SCENE | next: 无主包975/976/982/983/984继续或等在途落主→985 T-A03..A05→M2A-08→971 M2-B

stage: goal | PLAN-971 | r6 | round: 22 | from: work(974) | to: review(974/T-05) | status: archived | outcome: pass(974/T-04全V行) | 记录: T-04冻结并集执行——ace-av 1257+ace-native 280+ace-source 597 全PASS（2134例）；V-01..05逐行状态+具名pending（长流背压/上传文件973/TLS-provider道）；974→5/8 | next: 974/T-05独立复审→T-06/07归档（整包A交付喁M2A-03）；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 21 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-08 WebSocket帧层) | 记录: net_tcp_write_region raw面+真RFC-6455客户端往返（101/掩码帧/无掩码回显），net974 9/9绿（206fdf901） | next: 974 TLS/proxy provider道→T-04..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 20 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-08 HTTP方法族) | 记录: std.net全方法族（put/patch/delete/head加入），net974 8/8绿（5925768e9） | next: 974 WebSocket→TLS/proxy→T-04..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 19 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-08 URL族) | 记录: std.url落地（encode族+join_path，net974 7/7绿 c397befd2）；decode/query族=子串物化核心决定具名pending | next: 974 HTTP扩面→WebSocket→TLS/proxy→T-04..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 18 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-08分母核) | 记录: T-08协议分母核交付（13族逐项：已交7核心=TCP双向/HTTP client+server核心/SSE解析/redirect/timeout，证据net974 6/6；具名pending=URL族11符号/TLS/proxy/WebSocket/HTTP全方法面/多跳redirect/可配置timeout/upload-download973腿；分母零删行+关闭序冻结） | next: 974实现腿（URL族→HTTP扩面→WebSocket→TLS/proxy）→T-04..07；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 17 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-03缝裁定+std.net包装) | 记录: 装配缝裁定=(c)PLAN-1041内联约定；stdlib/ace/std/net.at落地+内联包装测试真回环6010012；net974 6/6绿（bd173aff0）；packages面接线具名后续 | next: 974/T-08协议分母核→T-04..07整包A交付；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 16 | from: work(974) | to: user | status: archived | outcome: pass(974 T-03装配缝探针) | 记录: T-03首个有界决定已定案输入——console道单源为设计事实，多模块std.net包装须先裁定装配缝（console前门接纳packages产物 vs 新宿主车道复用execute_platform）；974已交面稳定（net974 5/5绿） | next: 974/T-03缝决定→包装→T-08→T-04..07；或在途包落主后985 T-A03；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 15 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-02增量4) | 记录: SSE流式解析+redirect跟随+拒绝负控交付（cf1b29c9e）；net974 5/5绿（SSE 20002/redirect 30200→20000/refused -8） | next: 974 std.net包装→T-03/T-08→T-04..07 或按裁定；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 14 | from: work(974) | to: work(974) | status: archived | outcome: pass(974回环谜题关闭) | 记录: 具名开放发现已关闭（d090a48bb）：根因=补丁脚本转义损坏客户端实发29字节LF（非内核）+非阻塞复原+paced read三修；net974 4/4绿（server腿0.56s稳定） | next: 974/SSE/redirect/timeout 或按裁定；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 13 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-02增量3)+named-open-finding | 记录: platform.net/1 server面落地（23d47952f：listen/accept/local_port+paced accept）；978已由他会话归档（M2A链推进）；server腿E2E具名挂起（回环29/31部分投递谜题，#[ignore]带因非静默） | next: 974谜题关闭/SSE/redirect 或按裁定；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 12 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-02增量2) | 记录: Auto侧HTTP协议API面交付（54e09069e）：旧a2p http_client_sync义务经platform.net/1真TCP承载，GET/builder/post三腿独立oracle精确命中（6010012），net974三测全绿 | next: 974/T-02余量（SSE/server/redirect/timeout）或按裁定；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 11 | from: work(974) | to: work(974) | status: archived | outcome: pass(974/T-02 OS薄适配增量) | 记录: 974/T-02 首增量交付（c226a7215，+754行）：platform.net/1 全栈（av-core四op/caps/预算/台账+Win32 std::net后端+mir_verify准入+core源表+native具名拒）；net_974双测绿（真实TCP HTTP往返 oracle 20043 + 六负控含准入拒零dispatch）；回归48/48 | next: 974/T-02续（http API面/SSE/server腿）或按裁定；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 10 | from: work(依赖链) | to: work(974 A批) | status: archived | outcome: pass(974/T-01冻结) | 记录: 按既定裁定模式开工974：T-01 A期冻结交付（974→r9 executing 1/8；S1选型=扩展platform_io增platform.net/1；973依赖分账=核心TCP/HTTP/SSE腿不前置、File/Stream/upload文件腿具名阻塞；worktree .wt/lang-974/auto-lang已建） | next: 974/T-02（platform.net/1薄适配+最小真实HTTP往返）；GD/SCENE待Godot；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 9 | from: work(977 A批) | to: work(依赖链) | status: archived | outcome: pass(977/PY批A切片落主) | 记录: 977 B-PY-V(9fc1d2a34→4b9c367ba)+B-PY-R+phase fold 完成：**master=56aadead4**，spec m2-other-targets sha16 83af83e29cd299f4；相位门 ace-a2x 247+a2py 105 全PASS；977→4/16(T-01/B-PY-I/V/R)；**985/A 的 COVERAGE Python腿(A2PY路径)已获正式来源** | next: 无主包剩余974/975/976/980/982/983/984（M2A-03标准库门建议974/975；M2A-05 RG-A建议982/983）或在途包落主后985 T-A03；GD/SCENE待Godot；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 8 | from: work(977 A批) | to: work(977 A批) | status: archived | outcome: pass(977/B-PY-I) | 记录: 977/B-PY-I 已交付（977 worktree 727f45616，977→2/16）：旧a2p公开面9/9家族真实Python 3.14.2车道全绿（1.56s），路由事实冻结（instances-cap/cold_aggregates/is表达式），零车道缺口 | next: 977/B-PY-V（全语料case扫描+负控）→ B-PY-R；GD/SCENE待Godot；971/B 等 M2A-08（977/PY批R后phase fold即补COVERAGE Python腿）

stage: goal | PLAN-971 | r6 | round: 7 | from: user(选项A裁定) | to: work(977 A批) | status: archived | outcome: pass(977/T-01) | 记录: 用户裁定选项A→977 A批开工：T-01 A期冻结已交付（前置全归档核实/python.rs漂移重锚定/1042消费面登记/环境冻结Python3.14.2在机·Godot缺席→PY批先行·GD+SCENE不半途开工/命令平台冻结；977→r9 executing 1/16，worktree .wt/lang-977/auto-lang 已建） | next: work——977/B-PY-I（消费1042扩Python provider完整映射+保旧公开格+adapter+永久fixture），其后 B-PY-V/R；GD/SCENE 待Godot环境；REVERSE属B。971/B 仍待 M2A-08（977/PY落主后补齐COVERAGE Python腿之一）

stage: goal | PLAN-971 | r6 | round: 6 | from: work(985链) | to: user | status: archived | outcome: pass(985/T-09已交来源部分)+blocked(985 A批到边界) | 记录: 985/T-09 已交来源部分交付（f0全集33/33实测、FFI行逐行绑定、986/987 pending项；985→4/14）。**985 A批当前可执行边界已到**：T-A03..A05 阻塞于 973-984/OS064 各包A批（973/978/979/981 他会在途；974/975/976/977/980/982/983/984 无主）；T-08[A] 阻塞于 RG 正式输入+File(973) | next: user——选项A：指定无主包A批开工（建议977，阻塞COVERAGE Python腿；次选974/975喂M2A-03）；选项B：等在途包（973/978/979/981）落主后985继续T-A03；971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 5 | from: work(985链) | to: work(985链) | status: archived | outcome: pass(985/T-A02) | 记录: 985/T-A02 已交付（985 worktree d6342f534，985→3/14）：m2-cover两配方语料+冻结父表真值+四准入族具名红控绑定+V-A-COVER命令冻结 | next: 985/T-09/T-08[A]（已交来源部分）或用户裁定转无主包A批；971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 4 | from: work(985链) | to: work(985链) | status: archived | outcome: pass(985/T-A01) | 记录: 985/T-A01 义务矩阵+V-A-MAP红控已交付（985 worktree 0dff19cd4，簿记 247c345e6，985→executing 2/14）：162行 `autolang.ab-obligations/1` 矩阵（959冻结行+FFI/LIB增补+早期切片去重）+ 7例真实红控（缺行/重复/错source/非收据路径/词表无owner/pending虚报,0.03s绿,`cargo test -p auto-lang-core --lib obligation_ledger`） | next: 985/T-A02（下一就绪）或按用户改道无主包A批；971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 3 | from: user(续行指令) | to: work(985链) | status: archived | outcome: blocked(外部依赖扩大) | 记录: 2026-11-16 续行指令"推进985 A批至T-A05/V-A-LAND后回971 B"已执行第一步——985/T-01 A期冻结完成并提交（df9a75568，985 r16 executing 1/14，worktree .wt/lang-985/auto-lang 已建）；985 冻结结论：**M2A-08 需 973-984/OS064 各包A批正式落主**，其中 973/979/981 在途属其他会话、978 reviewed收口中——本会话不可替代；无主可开工的包：974/975/976/977/980/982/983/984；985 下一就绪任务=T-A01 义务矩阵 | next: user/work——后续轮次按续行指令继续（985 T-A01 或无主包A批），971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 2 | from: merge | to: user | status: archived | outcome: blocked | worktree: D:/autostack/.wt/lang-971/auto-lang（保留） | commit: 2f1eed27f (lang) / c1561d94 (auto-ui) | next: user——M2-B 阻塞于 M2A-08（985 A批未闭），A切片已正式合入可消费；同目标恢复B期即可

stage: merge | phase fold (A期PROOF切片；plan 保持 executing，同树保留至B——不归档/不清理/plan-artifacts 在途保留) | PLAN-971 | r6 | outcome: pass (phase) | prepared: 审查基线=phase re-pass e1a600ebe/2f70bbf7 + 相位门绿（ace-facade 28例18.0s、UI全包）；spec增量=新增 `docs/specs/ace-migration/m2-tools.md`（A切片28行已验接口与边界） | landed: 两仓均 rebase 后 `--ff-only`（lang master 增量仅978 docs提交、ui master 增量仅085 docs提交，零代码触面，证据基线有效；rebase后冒烟 fixture 0.50s/CLI 0.18s 绿）；**lang delivery `2f1eed27f`**（range-diff 全等：a2faad900→a1b78d9b4、e1a600ebe→82d659bba、fb838e3da→2f1eed27f）、**auto-ui delivery `c1561d94`**（a6065a59→b156de4e、2f70bbf7→c1561d94）；master tip=delivery ✓ 两仓 | canonical_verified: `docs/specs/ace-migration/m2-tools.md` @2f1eed27f，sha256前16 `c69d0afd1aa1fdf3`（5100字节）；无 legacy ledger 写；specs.json 未触碰 | archived: N/A（phase——B期未完不归档） | cleaned: N/A（phase——worktree/分支保留：`.wt/lang-971/{auto-lang,auto-ui}` 均与各自 master tip 同步；detached 依赖兄弟 auto-web/auto-down 保留至B；plan-artifacts/plan-971 在途保留含规范草案源稿） | 部署观察: 本切片纯库API+测试，无发行二进制/打包产物受影响；批量回归不因相位落地触发（971%5=1，归档时另核） | next: M2-B 全部任务阻塞于 M2A-08（985 A批未闭），goal 以具名阻塞停止

stage: review | phase re-pass (B-PROOF-R, A期PROOF切片) | reviewer: 同上局限（实施会话内） | PLAN-971 | r6 | outcome: pass (phase) | reviewed_commit: e1a600ebe（lang，含a2faad900+R-01修复）/ 2f70bbf7（ui，含a6065a59+R-02修复） | 相位门: `cargo t ace-facade`=PASS 28例 18.0s（热构建，冻结修复版）；`cargo test -p auto-lsp`=全绿（10 lib+0+4 protocol+2 新CLI+4 integration，10.8s 含构建）；修复复验：fixture 0.82s 绿、project单测绿 | F-971-R-01/R-02 闭合（closure: 发现→不变量[世代内预算/取消不丢]→修复→单测+端到端负控→当前结果全绿；剩余边界=端到端墙判别受冻结步预算限制，如实记录于规范边界） | 机器占用：等待外部 `cargo t ace-artifacts ace-semantics ace-a2x ace-av ace-native generated-delivery-controls ace-m2-build`（01:32启动）至0进程后启动lang门；ui门前再查（无输出=空闲）；ace-facade启动瞬间观察到12个进程（前任务收尾/新任务启动不明——本组结果18s热跑不受影响，如实记录） | next: phase fold 合入A切片（两仓），plan保持executing，同树保留至B（M2A-08前置）

stage: goal | PLAN-971 | r6 | round: 1 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-971/auto-lang | commit: a2faad900 (lang) / a6065a59 (auto-ui) | next: review（B-PROOF-R，A切片独立复审）

stage: review | phase (B-PROOF-R, A期PROOF切片) | reviewer: 实施会话内复审（本环境无独立复审上下文——从已提交工件重建结论，不采信执行摘要；该局限随收据记录） | PLAN-971 | r6 | outcome: needs_fix → 已修复待相位门 | worktree: D:/autostack/.wt/lang-971/auto-lang @ plan-971-dev | reviewed_commit: a2faad900（lang）/ a6065a59（auto-ui） | base: 858bbc271（lang）/ f2b03dcb（ui） | findings: **F-971-R-01**[moderate,合同内] 项目执行器在单世代内丢弃请求 wall/cancel（ExecLimits::default() 2s默认墙+恒false取消旗标），弱于E0同脊柱纪律（limits_from/cancel_flag），中立契约"整请求wall预算"承诺未闭环 → 修复：generation_limits 以剩余预算封顶 ExecLimits.wall + 请求取消旗标透传 execute_program（AV in-run 轮询）；判别证据=单测 p971_generation_wall_is_the_requests_remaining_budget（修复前红：默认2s>300ms上限）+端到端1ms墙spin负控（kind wall/ExecutionWall, exit 1；端到端对修复的判别力受编译耗时>1ms限制——相位检查先触发，判别由单测承担，如实记录）；**F-971-R-02**[minor] LSP CLI fixture 诊断范围断言未钉 line==0（单行坏源） → 收紧。修复提交：lang e1a600ebe、ui 2f70bbf7 | evidence: 复核 fixture 0.82s 绿、ace-host lib 2测试绿、UI 2测试0.32s绿；验收映射：AC-A-PROOF 六断言全 pass（定位原文件原span/恢复消失/测试失败非零/未知协议与错binary拒/停止回收/合法近邻）；AC-01（T-01冻结）pass | pending: 冻结修复版受影响完整组门（ace-facade + UI全包）——竞争组任务运行中按test-discipline等待后执行；规范增量（SD-01/SD-PHASE A切片段落）由 merge/phase-fold 沉淀 | next: 相位门通过后 phase fold 合入A切片（plan 保持 executing，同树保留至B）

stage: work | PLAN-971 | r6 | 2026-11-16 | outcome: pass (A批) | worktree: D:/autostack/.wt/lang-971/auto-lang @ plan-971-dev（基面 master@858bbc271，code commit a2faad900）+ 同组 D:/autostack/.wt/lang-971/auto-ui @ plan-971-dev（基面 auto-ui master@f2b03dcb，code commit a6065a59）+ detached 依赖兄弟 auto-web@0adebc0/auto-down@5af11a3（零改动） | task_ids: T-01, B-PROOF-I, B-PROOF-V | evidence: §5 T-01冻结记录、§8各任务[✅]行与组证据 | dependency_revisions: 966/969/970已归档；M2A-08未闭（985 r15 drafting） | blockers: M2-B全部任务（T-02/03/08/04/05/06/07）前置M2A-08收据未满足——按§5阶段合同B不启动，待985 A批关闭M2A-08 | next_responsible_role: B-PROOF-R（A切片独立复审，同树停写后进行；pass后按owner流程phase fold合入A切片，同树保留至B）

本轮会话测试模块表（test-discipline 2026-11-14）：

| 测试模块/实际范围 | 结果与执行数 | 准备/构建 | 运行 | 总墙钟 |
|---|---|---|---|---|
| lang `cargo t ace-facade`（-p auto-lang-ace-host --lib --tests，含新fixture） | PASS 28/28 | ~55s（含冷编译） | ~6s | 60.9s |
| lang `cargo t ace-source`（未改消费面确认组） | PASS 597/597 | 三组共享 | — | 三组合计13m38s |
| lang `cargo t auto-test-case`（契约确认组） | PASS 151/151 | 三组共享 | — | 同上 |
| lang `cargo t ace-host`（ace-conformance，**非本批受影响组**，计划原文组名核实） | 153 pass/11 fail/16 skip——11红为基线既有红（复放证据×2见§8 B-PROOF-V行），本批experimental/零改动无依赖路径，不判本批 | — | — | 同上 |
| lang 定向 `cargo test -p auto-lang-core --lib ace_entry` | PASS 5/5（含新纪律测试） | 已热 | 0.00s | <1s |
| UI `cargo test -p auto-lsp`（auto-ui worktree，全包含既有测试） | PASS 16/16（10 lib+4 protocol+2 新CLI） | ~12s | <1s | 12.7s |

新增用例：`p971_project_face_freezes_engine_and_registry_discipline`（~0.00s）、`p971_project_proof_roundtrip`（V-A-PROOF完整oracle，1.09s）、`p971_lsp_cli_real_binary_roundtrip`（~0.1s）、`p971_lsp_cli_wrong_binary_is_a_named_failure`（~0.07s）——全部≤10s门通过；机器占用核查=无竞争cargo/rustc/nextest进程；无>10s新用例。

2026-11-12 stage:new / PLAN-971 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原11任务/全部AC与V/依赖/阶段/预算和drafting 0/11；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部11个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/11；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

## 10. 待澄清事项

**B 期裁定（2026-11-17 用户已裁定：选项 B）——按§5命名义务收口，AC-03 消费者措辞在 r8 收窄；Playground 脚本前门与 debug/test UI 消费者迁具名后继（D-971-B-03/04）。原问题记录如下**：Playground 的脚本式 run/debug 前门（`code_runner`/`vm_runner`/`agent_debug`/`debugger`）接管 ACE 被**语言面**阻断（包道拒绝表达式语句 `source.unsupported`、console 仅单模块 profile；现有 demo 是脚本式源，见§5 B 边界实测）。二选一：(A) 认定这些前门属本包 B 完成条件 → 需先有“项目/包道表达式语句＋console/脚本形态”的语言面计划（超出本工具包合同），本包修订依赖/或转后继计划并把 AC-03 的 debug consumer 义务迁出；(B) 认定本包§5 命名义务=notebook＋工具面消费者（已交）→ T-03 按该口径收口，脚本前门另立语言计划。未裁定前 T-03 不勾、AC-03 不宣 PASS。

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

**M2-B 开工阻塞（2026-11-16 goal 会话具名）**：M2-B 全部任务（T-02/03/08/04 及收口 T-05/06/07）按§5阶段合同须先核 **M2A-08 已合入收据**（PLAN-985 A批 T-A05/V-A-LAND，当前985 r15 drafting 0/14 未闭）。解锁动作：985 A批完成并正式合入 M2A-08 收据后，以同一目标（`/goal 完成计划971` 或等价授权）恢复本计划 B 期——同树 `D:/autostack/.wt/lang-971/{auto-lang,auto-ui}` 保留在途，resync 后继续。B期开工时还须按§5核 A接口无变化（A接口变化回能力owner修复并定向更新A收据）。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。













