---
plan_id: PLAN-982
status: drafting
feature_name: AutoMan配置任务模板与生成器迁移
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 6
current_step: 0
total_steps: 40
depends_on: [PLAN-960, PLAN-964, PLAN-966, PLAN-969, PLAN-970, PLAN-971, PLAN-973, PLAN-976, PLAN-977, PLAN-978]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-man-gen.md]
touched_goals: [GOAL-002]
---

# [PLAN-982] AutoMan配置任务模板与生成器迁移

## 0. 变更摘要

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r5当前执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-07 r3：细化为11批，§5–8为当前执行合同；仍仅规划、不施工。


迁pac/am/index/state配置、命令任务、new/init模板与AutoGen/API生成；公共入口及生成工程均用ACE，静态项目模型保现行可求值范围；不能把动态配置误当纯数据以避开语言支持。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

当前正式前置：PLAN-969, PLAN-970, PLAN-966, PLAN-978, PLAN-973, PLAN-976, PLAN-977。后文旧依赖叙述补充读取本行；允许先做只读准备，不能以旧依赖较少绕过真实能力门。


正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮授权细化批次和跨仓依赖、必要时拆计划，先不施工。已核各仓Cargo、源码和测试入口：UI/Man根无AGENTS；Web守中立contract/公开SDK方向；OS域正文按当地AGENTS归OS064；没有主检出实现/测试授权。


用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-man/crates/auto-man-runtime/src/automan.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-man/crates/auto-man-core/src/target.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-gen-core/src`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-generation-core/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：AutoMan配置/命令/模板/生成工程迁移。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-man/crates/auto-man-runtime/src/automan.rs](../../../auto-man/crates/auto-man-runtime/src/automan.rs)：配置/执行真实owner；读取SHA256前16 `3699ce78450fd62d`

- [auto-man/crates/auto-man-core/src/target.rs](../../../auto-man/crates/auto-man-core/src/target.rs)：目标装配/CLI语义；读取SHA256前16 `827f96a5d400be3f`

- [crates/auto-generation-core/src](../../crates/auto-generation-core/src)：复用共享载体/失败发布，不把源码字符串另交旧VM；目录职责已核，确切符号/闭包开工再核

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A=CONFIG/PROOF；CONFIG只等960/964/969/973，PROOF补970/978；不等971整包、977/REVERSE或所有目标CLI。其他原批次全部B，消费M2A-08及原最小前置。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-PROOF-I, B-PROOF-V, B-PROOF-R, B-CONFIG-I, B-CONFIG-V, B-CONFIG-R |
| M2-B | B-EXEC-I, B-EXEC-V, B-EXEC-R, B-TEMPLATE-I, B-TEMPLATE-V, B-TEMPLATE-R, B-RUST-I, B-RUST-V, B-RUST-R, B-C-I, B-C-V, B-C-R, B-WEBTARGET-I, B-WEBTARGET-V, B-WEBTARGET-R, B-PY-I, B-PY-V, B-PY-R, B-GODOT-I, B-GODOT-V, B-GODOT-R, B-REVERSE-I, B-REVERSE-V, B-REVERSE-R, B-EXPORT-I, B-EXPORT-V, B-EXPORT-R, B-DEVSDK-I, B-DEVSDK-V, B-DEVSDK-R, T-05, T-06, T-07 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


本Plan各批独立实施/验证/复核，同一唯一worktree保留至全包归档；局部review不冒正式D。frontmatter为整包正式依赖并集，最小前置如下；LNNN=auto-lang/PLAN-NNN。下游可用冻结SHA准备，最终须已合入来源。共享文件单写，不建第二权威tree或链接。

唯一组D:/autostack/.wt/lang-982/，各实际改动仓一个兄弟tree；OS/App用OS064当地规约。

| 批次 | 交付 | 最小前置 | owner |
|---|---|---|---|
| 982/CONFIG | 配置包锁与项目模型 | L960/L964/L969/L973 | auto-man |
| 982/EXEC | run/service/req/debug/test命令 | 982/CONFIG; L970/L971 | auto-man |
| 982/TEMPLATE | new/init模板与AutoGen | 982/CONFIG; L966/L978 | auto-man + auto-lang |
| 982/RUST | Rust转译与构建 | 982/CONFIG; L978 | auto-man |
| 982/C | C转译与标准库构建 | 982/CONFIG; L978 | auto-man |
| 982/WEBTARGET | TS/JS转译 | 982/CONFIG; L976/L978 | auto-man |
| 982/PY | Python转译 | 982/CONFIG; 977/PY; L978 | auto-man |
| 982/GODOT | Godot脚本与场景工程 | 982/CONFIG; 977/GD; 977/SCENE; L978 | auto-man |
| 982/REVERSE | 逆向与源到源命令 | 982/CONFIG; 977/REVERSE | auto-man |
| 982/EXPORT | 现用发行工程格式 | 982/C; 982/RUST; L978 | auto-man |
| 982/DEVSDK | watch快路径与SDK发现 | 982/EXEC; 982/TEMPLATE; 982/RUST; 982/C; 982/WEBTARGET; 982/PY; 982/GODOT; 982/REVERSE; 982/EXPORT; L978 | auto-man |
| 982/PROOF | RG-A01真实配置与最小CLI | L982/CONFIG, L969, L970, L978 | 原owner；RG-A具名真实验证 |

### 982/CONFIG：配置包锁与项目模型
改面（核存在）：`D:/autostack/auto-man/crates/auto-man-runtime/src/automan.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/project_model_adapter.rs`；`D:/autostack/auto-man/crates/auto-package-lock`。
I实现：pac/am/index/state与参数/环境双阶段求值接ACE；路径锁身份与可求值静态子集不混用。
V验收：原动态pac与参数/环境输出一致；失败不写state/lock；同名包与配置隔离，未知语言不降成静态默认。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/EXEC：run/service/req/debug/test命令
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/main.rs`；`D:/autostack/auto-man/crates/auto/src/cmd_run.rs`；`D:/autostack/auto-man/crates/auto/src/cmd_req.rs`。
I实现：逐CLI dispatch接ACE run/session/tool协议，保stdin/捕获/退出/超时；不等待模板或全部目标生成。
V验收：真实CLI调用脚本、req持久变量、debug与test失败码；未知目标不能legacy fallback；超时后子任务回收。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/TEMPLATE：new/init模板与AutoGen
改面（核存在）：`D:/autostack/auto-man/crates/auto-man-runtime/src/asset.rs`；`crates/auto-gen-core/src`；`crates/auto-generation-core/src`。
I实现：模板求值与多文件输出用ACE/中立生成核心，去旧AutoInterpreter；模板输入、文件冲突/失败事务保持。
V验收：new/init生成原样工程再构建/运行；坏模板/取消不破坏已有workspace；独立用户模板同样运行。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/RUST：Rust转译与构建
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：原多模块工程使用新A2R，保CLI stdout/文件/成功信封和错误JSON/exit。
V验收：未修改生成Rust冷构建运行，与源expected一致，旧RustTrans实际dispatch零。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/C：C转译与标准库构建
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：新A2C接trans C/a2c-stdlib与目标配置，保ABI/链接/错误契约。
V验收：原C工程含新stdlib链接执行，CTrans入口零，错误目标保旧输出。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/WEBTARGET：TS/JS转译
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：真实CLI调用新TS/JS provider，保模块/依赖/source map。
V验收：实际JS运行与TS编译后运行，类型/运行时错误有原span，不能只打印代码。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/PY：Python转译
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：CLI接977/PY及工程配置/解释器发现。
V验收：真实Python原工程运行、错误JSON/exit正确，错误解释器身份不静默回落。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/GODOT：Godot脚本与场景工程
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：接GDScript/TSCN脚本/场景/资源工程装配。
V验收：实际Godot headless载入原工程，资源/脚本执行，缺依赖诊断。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/REVERSE：逆向与源到源命令
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：保r2a与s2s/emit已有公开命令，消费其独立工具接口。
V验收：原支持Rust输入→Auto真实运行，非法近邻错误/exit；不要求任意互逆。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/EXPORT：现用发行工程格式
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：逐CMake/IAR/GHS等现用export目标写配置和库闭包，逐子格实际工具链身份。
V验收：各原支持输出实际构建/链接/可执行或原目标验收，未知工具环境不改NA。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/DEVSDK：watch快路径与SDK发现
改面（核存在）：`D:/autostack/auto-man/scripts/man-distribution.py`；`D:/autostack/auto-man/scripts/man-install.py`；`D:/autostack/auto-man/scripts/test_install_transaction.py`。
I实现：watch和工具发现切候选ACE SDK；记录host/tool/runtime/stdlib闭包及重定位，稳定Auto编辑无Rust重编；不提前正式发布。
V验收：真实process树cargo/rustc0，Rust输入变化仍正确重建；旧cache/binary不被发现；候选安装失败回退保用户数据。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。


### A期PROOF批次（新增、未实施）

最小前置：L982/CONFIG, L969, L970, L978。实际改面：`D:/autostack/auto-man/crates/auto/src/cmd_run.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/automan.rs`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

I：从原CLI/runtime fixture冻结多模块pac、args/env及最小脚本，接显式ACE candidate入口，证明配置与实际命令消费同一SDK；不等待整个EXEC/TEMPLATE/各target迁移。
V：双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[auto-man/test/fixtures/plan446/j2check/auto/pac.at](../../../auto-man/test/fixtures/plan446/j2check/auto/pac.at)（下列节选L1–L3；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
name: "j2-key-probe"
scene: "ui"
render: "vm"
```

本包所需独立结果：原pac/am配置的name/dep/target/命令求值保真实值；模板同输入生成语义相同产物，错误配置不覆盖旧产物；真正生成服务/SDK需冷编并消费。 本轮未运行；这些断言是实施目标，不是已观察PASS。

```text
Hello, $name!
V: $version
```

上面是已有AutoGen模板原文（不是独立Auto程序），实例输入name=AGE/version=2应生成两行Hello, AGE!与V: 2并保持原末尾LF；坏模板/guard失败不动旧输出。旧pac的render:"vm"是模式来源，target/mode/执行引擎须分维度核，不把模式字样当旧VM执行许可。

**类型/代码设计。** pac/am/index/state/任务/模板普通源→ACE公共求值与编译期→既有Man模型/生成核心→各目标实际工程/安装consumer。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ProjectEvaluation { source: SourceClosure, configuration: TypedConfig, tasks: TaskGraph, toolchain: ToolchainIdentity }
struct GeneratedProject { manifest: ProjectManifest, artifacts: ArtifactSet, dependencies: DependencySet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-man/crates/auto-man-runtime/src/automan.rs](../../../auto-man/crates/auto-man-runtime/src/automan.rs) | 配置/执行真实owner |
| [auto-man/crates/auto-man-core/src/target.rs](../../../auto-man/crates/auto-man-core/src/target.rs) | 目标装配/CLI语义 |
| [crates/auto-generation-core/src](../../crates/auto-generation-core/src) | 复用共享载体/失败发布，不把源码字符串另交旧VM |

**状态、错误与资源。** 依赖/target/配置字段错、命令失败、模板坏源必须保旧产物且返回原诊断，不用静态JSON绕过动态配置求值。 十一批CONFIG/EXEC/TEMPLATE/各目标/EXPORT/DEVSDK与A/PROOF各自来源；环境/toolchain/锁变更精确失效，子进程取消与清理具名。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [auto-man/test/fixtures/plan446/j2check/auto/pac.at](../../../auto-man/test/fixtures/plan446/j2check/auto/pac.at) | 旧真实配置/工程入口 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `807c35217b9a6ea9` | 拟ace.m2.p982.source01：eval-config/generate-cold-run/dry-run按本批；负控/资源见V-DETAIL-02/03；原任务 B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V |
| [auto-man/crates/auto-man/tests/plan805_toolchain_parity.rs](../../../auto-man/crates/auto-man/tests/plan805_toolchain_parity.rs) | 工具链/配置行为 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `11e4f75454f7da67` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V |
| [auto-man/crates/auto-man/tests/plan816_t03_sidecar_request.rs](../../../auto-man/crates/auto-man/tests/plan816_t03_sidecar_request.rs) | sidecar真实请求/失败恢复 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `c6b1e2f839fb1a22` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V |
| [crates/auto-gen/tests/fixtures/templates/hello.txt.at](../../crates/auto-gen/tests/fixtures/templates/hello.txt.at) | 原模板Hello/name/version字节，输入name=AGE/version=2须Hello, AGE!和V: 2及原换行 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `df5ba351d4840bea` | 拟ace.m2.p982.source04：eval-config/generate-cold-run/dry-run按本批；负控/资源见V-DETAIL-02/03；原任务 B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V |
| [crates/auto-gen/tests/v03_failure_semantics.rs](../../crates/auto-gen/tests/v03_failure_semantics.rs) | render/guard失败保旧、dry_run零写、批次错误隔离/guard保留 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `c71c2851a0c55e7f` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 逐命令/配置/模板/生成target原行为清单与实际工程脚本冻结；所有目标保完整，不能Man单help成功冒全CLI迁移。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-CONFIG-I`、`B-CONFIG-V`、`B-TEMPLATE-I`、`B-TEMPLATE-V`、`B-RUST-I`、`B-RUST-V`、`B-PROOF-I`、`B-PROOF-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/m2-man-gen.md | 有限切片→实际已验支持/拒绝/来源 | 不把计划当现状 | AC-01..06、全部AC-B-* |
| SD-02 | modify | docs/specs/ace-migration/replacement-scope-and-evidence.md | 959行关联新producer/consumer及差额 | 替换闭合 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-man-gen.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |

| SD-DETAIL | add | `docs/specs/ace-migration/m2-man-gen.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

外仓领域知识由真实owner落当地Specs；语言侧只保通用接口/迁移指针，不重复领域规则。

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部未执行。cwd前缀lang/ui/man为本Plan唯一tree。各I读取原runner/help/项目配置固定参数/工具版本，创建永久正反例，未冻结不能进入V；现有target绿不自动证明新ACE支持。只保原支持承诺，原TODO不冒已实现，但实际承诺的缺环境格不得免验。

材料唯一D:/autostack/plan-artifacts/plan-982/，开工登记owner/用途/在途；按仓/批区分，外置target/SDK/安装副本均归scope。窗口/GPU单实例错峰，原ignore/平台/资源预算保持；准备/运行/清理、选择/执行/排除/子终态分列。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-CONFIG | AC-B-CONFIG | man: cargo tc；cargo test -p auto-package-lock --test lock_contract；新增ACE config例纳auto-man-runtime lib | 原动态pac与参数/环境输出一致；失败不写state/lock；同名包与配置隔离，未知语言不降成静态默认。 |
| V-B-EXEC | AC-B-EXEC | man: cargo test -p auto --test plan816_cli_driver；cargo test -p auto-man-runtime --test plan816_t03_non_ui_service；隔离真实CLI脚本 | 真实CLI调用脚本、req持久变量、debug与test失败码；未知目标不能legacy fallback；超时后子任务回收。 |
| V-B-TEMPLATE | AC-B-TEMPLATE | man: cargo test -p auto-man --test plan803_asset_facade_tests；lang: cargo t auto-gen auto-gen-core auto-generation-core generated-delivery-controls | new/init生成原样工程再构建/运行；坏模板/取消不破坏已有workspace；独立用户模板同样运行。 |
| V-B-RUST | AC-B-RUST | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 未修改生成Rust冷构建运行，与源expected一致，旧RustTrans实际dispatch零。 |
| V-B-C | AC-B-C | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 原C工程含新stdlib链接执行，CTrans入口零，错误目标保旧输出。 |
| V-B-WEBTARGET | AC-B-WEBTARGET | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 实际JS运行与TS编译后运行，类型/运行时错误有原span，不能只打印代码。 |
| V-B-PY | AC-B-PY | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 真实Python原工程运行、错误JSON/exit正确，错误解释器身份不静默回落。 |
| V-B-GODOT | AC-B-GODOT | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 实际Godot headless载入原工程，资源/脚本执行，缺依赖诊断。 |
| V-B-REVERSE | AC-B-REVERSE | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 原支持Rust输入→Auto真实运行，非法近邻错误/exit；不要求任意互逆。 |
| V-B-EXPORT | AC-B-EXPORT | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 各原支持输出实际构建/链接/可执行或原目标验收，未知工具环境不改NA。 |
| V-B-DEVSDK | AC-B-DEVSDK | man: python scripts/test_install_transaction.py；python scripts/test-man-install-compat.py；真实watch采样接978，安装终验归986/987 | 真实process树cargo/rustc0，Rust输入变化仍正确重建；旧cache/binary不被发现；候选安装失败回退保用户数据。 |

受影响共享IR/语义加完整职责与必要下游；T-05核全批/组合/原资源门，身份不变证据复用，不反复全仓。局部review不代全包final；未闭in-scope阻execution_done。


### A期PROOF验证（本轮未运行）

| Case | AC | 实际/拟补入口 | 独立oracle |
|---|---|---|---|
| V-A-PROOF | AC-A-PROOF | man: cargo tc；cargo test -p auto --test plan816_cli_driver；cargo test -p auto-man-runtime --test plan816_t03_non_ui_service；I补ACE公开CLI具名case，不以旧runner绿替新结果 | 双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V | 原pac/am配置的name/dep/target/命令求值保真实值；模板同输入生成语义相同产物，错误配置不覆盖旧产物；真正生成服务/SDK需冷编并消费。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V | 依赖/target/配置字段错、命令失败、模板坏源必须保旧产物且返回原诊断，不用静态JSON绕过动态配置求值。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V | 十一批CONFIG/EXEC/TEMPLATE/各目标/EXPORT/DEVSDK与A/PROOF各自来源；环境/toolchain/锁变更精确失效，子进程取消与清理具名。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/原模式/命令/资源完整 | T-01及各I |
| AC-02 | work | 完整原能力经真实原源/工程执行 | 全部I/V |
| AC-03 | work | 实际consumer/权限/资源/恢复接线、无legacy fallback | 全部V/959行 |
| AC-04 | review | 独立核各批及全包AC/差额/健康/Spec | 各R/T-05 |
| AC-05 | merge | 正式来源/规范/归档/清理齐 | T-06/07 |
| AC-06 | work | 原T-08补充义务全部分配且闭合 | 全部AC-B-* |
| AC-B-CONFIG | work | 原动态pac与参数/环境输出一致；失败不写state/lock；同名包与配置隔离，未知语言不降成静态默认。 | B-CONFIG-V/R |
| AC-B-EXEC | work | 真实CLI调用脚本、req持久变量、debug与test失败码；未知目标不能legacy fallback；超时后子任务回收。 | B-EXEC-V/R |
| AC-B-TEMPLATE | work | new/init生成原样工程再构建/运行；坏模板/取消不破坏已有workspace；独立用户模板同样运行。 | B-TEMPLATE-V/R |
| AC-B-RUST | work | 未修改生成Rust冷构建运行，与源expected一致，旧RustTrans实际dispatch零。 | B-RUST-V/R |
| AC-B-C | work | 原C工程含新stdlib链接执行，CTrans入口零，错误目标保旧输出。 | B-C-V/R |
| AC-B-WEBTARGET | work | 实际JS运行与TS编译后运行，类型/运行时错误有原span，不能只打印代码。 | B-WEBTARGET-V/R |
| AC-B-PY | work | 真实Python原工程运行、错误JSON/exit正确，错误解释器身份不静默回落。 | B-PY-V/R |
| AC-B-GODOT | work | 实际Godot headless载入原工程，资源/脚本执行，缺依赖诊断。 | B-GODOT-V/R |
| AC-B-REVERSE | work | 原支持Rust输入→Auto真实运行，非法近邻错误/exit；不要求任意互逆。 | B-REVERSE-V/R |
| AC-B-EXPORT | work | 各原支持输出实际构建/链接/可执行或原目标验收，未知工具环境不改NA。 | B-EXPORT-V/R |
| AC-B-DEVSDK | work | 真实process树cargo/rustc0，Rust输入变化仍正确重建；旧cache/binary不被发现；候选安装失败回退保用户数据。 | B-DEVSDK-V/R |
| AC-A-PROOF | work | 双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共40任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [ ] **T-01｜M2-A准备｜owner_stage: work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。

- [ ] **B-CONFIG-I｜M2-A｜owner_stage: work｜AC-B-CONFIG/AC-02/03/06**：pac/am/index/state与参数/环境双阶段求值接ACE；路径锁身份与可求值静态子集不混用。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-CONFIG-V｜M2-A｜owner_stage: work｜AC-B-CONFIG**：运行V-B-CONFIG完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-CONFIG-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：从原CLI/runtime fixture冻结多模块pac、args/env及最小脚本，接显式ACE candidate入口，证明配置与实际命令消费同一SDK；不等待整个EXEC/TEMPLATE/各target迁移。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。

- [ ] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。 核子终态与身份，未执行不PASS。

- [ ] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **B-EXEC-I｜M2-B｜owner_stage: work｜AC-B-EXEC/AC-02/03/06**：逐CLI dispatch接ACE run/session/tool协议，保stdin/捕获/退出/超时；不等待模板或全部目标生成。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-EXEC-V｜M2-B｜owner_stage: work｜AC-B-EXEC**：运行V-B-EXEC完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-EXEC-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-TEMPLATE-I｜M2-B｜owner_stage: work｜AC-B-TEMPLATE/AC-02/03/06**：模板求值与多文件输出用ACE/中立生成核心，去旧AutoInterpreter；模板输入、文件冲突/失败事务保持。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-TEMPLATE-V｜M2-B｜owner_stage: work｜AC-B-TEMPLATE**：运行V-B-TEMPLATE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-TEMPLATE-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-RUST-I｜M2-B｜owner_stage: work｜AC-B-RUST/AC-02/03/06**：原多模块工程使用新A2R，保CLI stdout/文件/成功信封和错误JSON/exit。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-RUST-V｜M2-B｜owner_stage: work｜AC-B-RUST**：运行V-B-RUST完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-RUST-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-C-I｜M2-B｜owner_stage: work｜AC-B-C/AC-02/03/06**：新A2C接trans C/a2c-stdlib与目标配置，保ABI/链接/错误契约。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-C-V｜M2-B｜owner_stage: work｜AC-B-C**：运行V-B-C完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-C-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-WEBTARGET-I｜M2-B｜owner_stage: work｜AC-B-WEBTARGET/AC-02/03/06**：真实CLI调用新TS/JS provider，保模块/依赖/source map。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-WEBTARGET-V｜M2-B｜owner_stage: work｜AC-B-WEBTARGET**：运行V-B-WEBTARGET完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-WEBTARGET-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-PY-I｜M2-B｜owner_stage: work｜AC-B-PY/AC-02/03/06**：CLI接977/PY及工程配置/解释器发现。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-PY-V｜M2-B｜owner_stage: work｜AC-B-PY**：运行V-B-PY完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-PY-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-GODOT-I｜M2-B｜owner_stage: work｜AC-B-GODOT/AC-02/03/06**：接GDScript/TSCN脚本/场景/资源工程装配。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-GODOT-V｜M2-B｜owner_stage: work｜AC-B-GODOT**：运行V-B-GODOT完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-GODOT-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-REVERSE-I｜M2-B｜owner_stage: work｜AC-B-REVERSE/AC-02/03/06**：保r2a与s2s/emit已有公开命令，消费其独立工具接口。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-REVERSE-V｜M2-B｜owner_stage: work｜AC-B-REVERSE**：运行V-B-REVERSE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-REVERSE-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-EXPORT-I｜M2-B｜owner_stage: work｜AC-B-EXPORT/AC-02/03/06**：逐CMake/IAR/GHS等现用export目标写配置和库闭包，逐子格实际工具链身份。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-EXPORT-V｜M2-B｜owner_stage: work｜AC-B-EXPORT**：运行V-B-EXPORT完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-EXPORT-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-DEVSDK-I｜M2-B｜owner_stage: work｜AC-B-DEVSDK/AC-02/03/06**：watch和工具发现切候选ACE SDK；记录host/tool/runtime/stdlib闭包及重定位，稳定Auto编辑无Rust重编；不提前正式发布。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-DEVSDK-V｜M2-B｜owner_stage: work｜AC-B-DEVSDK**：运行V-B-DEVSDK完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-DEVSDK-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final核全部批收据/组合/遗漏/延期/workaround、warning/fmt/debug和Spec；缺必需结果不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner次序合入/落规范，核真实来源/consumer同版，全部齐后归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan、输出逐目录对账/清scope/登记，guard clean后清树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-982 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原40任务/全部AC与V/依赖/阶段/预算和drafting 0/40；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部40个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/40；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-07 r3：细化11批，0/37。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留；原STATIC/TARGETS汇总批进一步逐目标拆成具名I/V/R，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
