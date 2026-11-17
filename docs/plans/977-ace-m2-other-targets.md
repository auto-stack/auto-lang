---
plan_id: PLAN-977
status: executing
feature_name: 其余A2X目标与独立转换工具接管
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-17
plan_revision: 11
current_step: 6
total_steps: 10
depends_on: [PLAN-959, PLAN-960, PLAN-962, PLAN-964, PLAN-965, PLAN-968, PLAN-1042]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-other-targets.md]
touched_goals: [GOAL-002]
---

# [PLAN-977] 其余A2X目标与独立转换工具接管

## 0. 变更摘要

2026-11-17 work当前：状态 executing **6/10**。B 阶段门已开（M2A-08 正式成立，985 A 批 phase fold delivery `95c7fbad8`；本树 resync 后为该提交的祖先链成员）。**B-REVERSE-I/V 已交**（worktree commit `f2606fa91`）：r2a 受检入口具名拒绝 + s2s ACE 规范方言与行级 source map + ACE 公开前门真实执行证据，固定组 `s2s` PASS(11)、`ace-facade` PASS(36)、`auto-lang-core` 1653/1654（1 基线红=master 侧 plan814 build-input 边界，非本批触面）。剩余 B-REVERSE-R、T-05/06/07 属 review/merge 阶段。合同（范围/AC/依赖/总任务数）未变，plan_revision 保持 r11。

2026-11-16 r11当前接续：状态仍executing 4/10。已完成T-01及PY的I/V/R，正式交付56aadead4；剩余六项为B-REVERSE-I/V/R、T-05/06/07，全部保原M2-B归属，须先有M2A-08。删除Godot只解除目标/环境义务，不解除此阶段门，不代表可立即归档或清树。本轮只核对规划，不实施/运行/恢复目标。

2026-11-16 r10（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包A只保PY，B保REVERSE及原收口；GD/SCENE六个未实施任务与对应AC/V从当前合同移至§9范围历史，不再等待Godot。总数16→10，已完成4项保持；比例变化来自用户范围调整，不是新完成。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

2026-11-15 r8当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r7当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r5当前执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-07 r3：细化为4批，§5–8为当前执行合同；仍仅规划、不施工。


本M2交Python受检provider；Godot/GDScript/TSCN按用户后置，保留原用例及历史定义。r2a独立逆向工具及s2s/emit按真实owner保留或迁中立模块，不能因trans目录删除而丢公开能力。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

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
- `crates/auto-lang-core/src/trans/python.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/trans/gdscript.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/trans/tscn.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/trans/r2a.rs`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/a2x-core`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### T-01 r9 A期冻结历史记录（目标范围已由r10更新）（2026-11-16，r9执行）

**r9授权历史（GD/SCENE范围已由r10覆盖）**：用户裁定解锁路径选项A（2026-11-16 goal 续行）——开工本包 A 批（PY/GD/SCENE）；REVERSE 及收口任务属 B（M2A-08 后）。worktree：`D:/autostack/.wt/lang-977/auto-lang`（分支 `plan-977-dev`，基面 master@b66405d6d）；材料 `plan-artifacts/plan-977/` 已登记；无 junction/symlink。

**r9前置核实历史（本版只消费PY）**：977/PY 最小前置 L1042/L960/L962/L964/L965/L968 **全部已归档** ✓（PY 批就绪）；GD 前置同（除1042）✓；SCENE 依 977/GD。

**来源重核（r6快照对照）**：`gdscript.rs`=`24e922d887d8a2f0`✓、`tscn.rs`=`c23f2c48735e72ef`✓、`r2a.rs`=`6e447b013afea867`✓；**`python.rs` 漂移**（r6 `696d82cf3f26647b` → 现值 `b6fff84a26408a59`，3258行）——r6 后某合入改动，T-01 重锚定现值，B-PY-I 以当前内容为准逐符号核公开面。具体输入锚重核：`a2p/08_generics/001_generic_func/generic_func.at`=`dec913dbcc21f70f`✓（expected.py 现值 `c4cc3b2cdbe55e99`）、`a2gd/.../generic_func.expected.gd`=`7ae77300c27f2fc6`、`ball.expected.tscn`=`9b7e88cb507172c9`、`r2a/08_generics/003_generic_fn.rs`=`eb068ed1adac2846`✓。

**1042 消费面（已交）**：`experimental/ace/crates/a2x-python`（provider/lower/print/runtime/cert/ports/foreign_bridge/seed）——Checked-HIR→确定性 Python3 模块（generated.py+a2x_main.py+版本化 runtime+source_map+manifest）、多模块包含 std.math/std.text、独立身份（auto-a2x-python/AXPY1..3/ac-a2py/*，永不别名到旧 PythonTrans/AutoVM）、checked i32 截断除法、字节级 UTF-8、显式 free/close 生命周期、stdlib port 目录按操作ID绑定。PY 批增量=完整目标/运行时映射+模块与 source map 扩面+保旧 Python 公开支持格（`trans/python.rs` 现值面）+实际目标 adapter——具体接口冻结归 B-PY-I 有界决定门。

**r9环境冻结历史（Godot不再是当前门）**：**Python 3.14.2 在机** ✓（PY 批 V 腿可真实执行）；**Godot 不在机**——GD/SCENE 批 V 腿（真实 Godot headless 执行/场景装载）= 缺工具=缺证，保持未执行/阻塞不 NA。**A 批执行顺序冻结：PY 批先行端到端（I/V/R）；GD/SCENE 不半途开工**（I 可静态准备但 V/R 无工具不可闭——待 Godot 环境或具名阻塞登记，不产生"实现完待验"的半批）。

**命令冻结（现值）**：旧面确认组 `cargo t a2py`（legacy trans::python 测试，组已核存在）；ACE 侧 a2x-python 测试入口与 PY 批新 adapter/fixture 的真实命令由 B-PY-I 按实际触面登记并接固定组。平台=Windows x64（Linux 腿按 r8 裁定归 1063）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本轮焦点：Python/逆向工具；以下Godot材料仅保历史取材，不再进入当前任务。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-lang-core/src/trans/python.rs](../../crates/auto-lang-core/src/trans/python.rs)：PY旧目标行为；读取SHA256前16 `696d82cf3f26647b`

- [crates/auto-lang-core/src/trans/gdscript.rs](../../crates/auto-lang-core/src/trans/gdscript.rs)：GD类/信号/节点语义；读取SHA256前16 `24e922d887d8a2f0`

- [crates/auto-lang-core/src/trans/tscn.rs](../../crates/auto-lang-core/src/trans/tscn.rs)：场景资源边/属性；读取SHA256前16 `c23f2c48735e72ef`

- [crates/auto-lang-core/src/trans/r2a.rs](../../crates/auto-lang-core/src/trans/r2a.rs)：逆向输入/不支持诊断；读取SHA256前16 `6e447b013afea867`

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

### 2026-11-16接续核查与授权边界（r11）

本轮用户要求核对去Godot后的剩余工作与ACE/AGE M2顺序，仅修改规划。读取主线fe75d239b745d3a2652208e9d22ac6c987c75939；977唯一树HEAD=56aadead443380d8d102d1031705a554e73b2d05、干净，该交付为master祖先，canonical `m2-other-targets.md` sha256前16=83af83e29cd299f4。Spec中旧“GD/SCENE缺Godot”仅保原已验状态，不覆盖用户范围裁定；本轮不改已验Spec。r2a/s2s/emit现有入口已静态核实为`transpile_r2a`、`lower_source`、`emit_code`；固定组`s2s`/`auto-lang-core`存在，未执行。985当前4/14、M2A-08未交；不能把直接语言前置全归档等同B开工许可。

## 5. 详细设计

### M2-TARGET-SCOPE-20261116：本包当前边界

本包A只保PY，B保REVERSE及原收口；GD/SCENE六个未实施任务与对应AC/V从当前合同移至§9范围历史，不再等待Godot。总数16→10，已完成4项保持；比例变化来自用户范围调整，不是新完成。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A的PY按原最小语言接口；REVERSE及其全部公开入口接管属B，不能由982/PROOF反等。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-PY-I, B-PY-V, B-PY-R |
| M2-B | B-REVERSE-I, B-REVERSE-V, B-REVERSE-R, T-05, T-06, T-07 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


本Plan局部批次可独立实施/验证/复核，不新增Plan状态，也不等于可单批归档/删树。本Plan唯一worktree保留至全部批次及最终review/merge完成。分支冻结SHA只供受控准备；下游最终验收须核已合入来源，批次review不冒正式D。

frontmatter是整包最终依赖并集，各批按下表最小接口前置启动。LNNN=auto-lang/PLAN-NNN；UI/AG/OS标仓身份。依赖收据必须包含repo/commit、Plan revision、接口/schema/能力域和实际consumer/oracle。只有计划名字不等接口就绪。

唯一协调组D:/autostack/.wt/lang-977/，各实际改动仓保一个兄弟tree；实施/review顺序复用，公共文件单写。外仓开工重核实际唯一树，不建junction/symlink。OS/App采用OS064规定的OS worktree/独立App clone，不套语言树规则。

| 批次 | 交付结果 | 最小前置 | owner |
|---|---|---|---|
| 977/PY | Python provider（消费1042，不复制最小producer） | L1042/L960/L962/L964/L965/L968 | auto-lang |
| 977/REVERSE | r2a与s2s独立工具 | L959/L960 + M2A-08正式来源（B阶段门） | auto-lang |

### 当前剩余工作及能否收口（唯一接续口径）

| 原任务 | 实际剩余结果 | 就绪条件与关闭证据 |
|---|---|---|
| B-REVERSE-I / work | **✅ 已交 `f2606fa91`**：r2a 分类=中立 syn 解析+自有发射（不消费 Auto 前端/AST、不经旧执行器/旧目标发射；公开命令 `transpile_r2a_file` 行为不变，37 例 golden 逐字节）+ 受检入口 `transpile_r2a_checked`/`r2a_unsupported_forms`（20 类稳定键具名拒绝，注释位扫描，未闭合标记 fail-closed）；s2s 分类=依赖旧前端、不依赖旧执行器/旧目标发射（`lower_source` 产物逐字节不变）+ `EmitDialect::{Legacy,AceCanonical}` + 行级 source map（`emit_code_with_map`/`lower_source_with_map`/`lower_source_for_ace`/`source_line_for_output`）。分类落 spec `m2-other-targets.md` §REVERSE；前置门=M2A-08（985 delivery `95c7fbad8`，本树祖先已核） | 已闭；永久用例 `trans::r2a::tests::p977_*`(5)、`trans::auto_s2s::tests_s2s::p977_*`(3) 经 `cargo t auto-lang-core`/`cargo t s2s` 实跑 |
| B-REVERSE-V / work | **✅ 已交 `f2606fa91`**：`cargo t s2s` PASS(11 例)；`cargo t auto-lang-core` 1653/1654（1 基线红=master 侧 `plan814_build_input_boundary`：`auto-tool-execution` 未列入 `IDENTITY_SOURCE_CRATES`，master 同输入机械复放同红，非本批触面，归批量回归 triage）；`cargo t ace-facade` PASS(36 例，含新 `p977_reverse_s2s` 3 例) | 已闭：ACE 公开前门真实执行 `fib(10)=55` 对独立手算；历史方言在同门 `text/source.syntax` 具名拒绝；bind 期诊断经 source map 回原源精确行（`return add(1)`）；`autovm/vm/evaluator` 具名拒绝（无旧执行器回退） |
| B-REVERSE-R / review | 独立复核 REVERSE 完整源码/反控/消费者/差额/来源 | V 齐且同树停写（已 `f2606fa91`）；批 R 不代全包 final |
| T-05 / review | 核本版PY+REVERSE全部AC、Spec增量、遗漏/延期/组合与健康；按原身份复用有效PY证据 | PY、REVERSE 两批 I/V 已交；T-05 仍须对照原99例、单文件/多模块与资源边界逐义务查映射/实际oracle，具名未闭required回原work，不用家族代表冒逐例覆盖 |
| T-06 / merge | 原review通过后正式合入、实际知识沉淀及整Plan归档 | 当前r11及冻结实现SHA（REVERSE=`f2606fa91`）的独立final pass，全部required收据和真实依赖齐；不能因B切片已交跳过 |
| T-07 / merge | 全输出路径对账、证据摘要折Plan、按现行保留规则清理与最终状态核对 | 原收尾事实齐，未知目录不删，Windows用`pwsh -NoProfile -File D:/autostack/wt-guard.ps1`核原tree后按规处理；本轮不执行 |

因此目前**是 6/10，不能直接完整归档**：B-REVERSE-I/V 已交，但 B-REVERSE-R（批内独立复审）与 T-05/06/07（全包 final/合入/清理）仍未执行。这是既定阶段门，不是Godot或976/980/981整包对REVERSE新增的技术依赖。977/PY正式来源可继续供库和985/A使用，985/A只等本包required A证据，不反等REVERSE或977全包归档，防止“985等977全包、977又等985/A”的假环。若未来要把REVERSE前移A，须另行明确修订阶段合同及相关路线；本轮保持既定A/B分工，不默作此裁定，也不新建计划。

### 977/PY：Python provider
改面（均核存在）：`crates/auto-lang-core/src/trans/python.rs`；`experimental/ace/crates/a2x-core`；`crates/auto-lang-core/src/test_cases`。
I实现：消费1042的Checked HIR→Python producer/helper后扩完整目标/运行时映射、模块与source map；保原Python公开支持格；接实际目标adapter，旧emitter仅作历史输入。
V验收：原Python语料的返回/输出/短路/异常/资源各对独立oracle；跨模块生成工程在Python执行，坏输入不替换旧输出。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 977/REVERSE：r2a与s2s独立工具
改面（均核存在）：`crates/auto-lang-core/src/trans/r2a.rs`；`crates/auto-lang-core/src/trans/auto_s2s.rs`；`crates/auto-lang-core/src/trans/emit.rs`。
I实现：分类独立逆向工具与旧前端依赖，保既有公开命令；s2s消费新源事实，r2a保中立syn解析；不要求任意语言互逆。
V验收：已支持Rust→Auto用例保行为；不支持形态具名拒绝；s2s重写源可在ACE执行且source map正确，无旧执行/发射反依赖。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[crates/auto-lang/test/a2p/08_generics/001_generic_func/generic_func.at](../../crates/auto-lang/test/a2p/08_generics/001_generic_func/generic_func.at)（下列节选L1–L11；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
#[with(T)]
fn identity(x T) T {
    return x
}

fn main() {
    let a = identity(42)
    let b = identity("hello")
    print(a)
    print(b)
}
```

本包所需独立结果：Python旧源应逐行42、hello；r2a需原Rust输入和Auto语义，不能只证明生成文件存在。Godot脚本/场景原断言留§9及旧用例，非本版required。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** Checked语义或原r2a Rust输入→各目标专有provider/反向适配→真实目标工具→模块/行为与来源。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct TargetEvidence { target: TargetId, source: SourceId, runtime: ToolIdentity, artifacts: ArtifactSet, result: ObservationSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-lang-core/src/trans/python.rs](../../crates/auto-lang-core/src/trans/python.rs) | PY旧目标行为 |
| [crates/auto-lang-core/src/trans/gdscript.rs](../../crates/auto-lang-core/src/trans/gdscript.rs) | 历史取材：GD类/信号/节点语义；用户后置，非本版改面 |
| [crates/auto-lang-core/src/trans/tscn.rs](../../crates/auto-lang-core/src/trans/tscn.rs) | 历史取材：场景资源边/属性；用户后置，非本版改面 |
| [crates/auto-lang-core/src/trans/r2a.rs](../../crates/auto-lang-core/src/trans/r2a.rs) | 逆向输入/不支持诊断 |

**状态、错误与资源。** Python坏模块/依赖及逆向无法表达形具名拒；本版Python/逆向工具缺席是缺证，不默填PASS/NA；Godot按用户后置，不成为缺工具阻塞。 当前PY/REVERSE两批各自I/V/R保留，原GD/SCENE留§9范围历史；A的正向provider与B逆向工具分期，路径/资源/runtime身份不混，close回收子进程。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/a2p/08_generics/001_generic_func/generic_func.at](../../crates/auto-lang/test/a2p/08_generics/001_generic_func/generic_func.at) | 泛型公开值42/hello，Python模板全文另保 | `generic_func.expected.py`；SHA256前16 `dec913dbcc21f70f` | 拟ace.m2.p977.source01：run-python；负控/资源见V-DETAIL-02/03；原任务 B-PY-I/B-PY-V |
| [crates/auto-lang/test/a2gd/08_generics/001_generic_func/generic_func.at](../../crates/auto-lang/test/a2gd/08_generics/001_generic_func/generic_func.at) | GD入口/泛型值 | `generic_func.expected.gd`；SHA256前16 `2ae6e2f18564556d` | `user-deferred/not-required (M2-TARGET-SCOPE-20261116)`：原GD/SCENE目标腿保留原源/金样及§9任务，非本版执行；同源普通Auto语义若属于其他required后端，沿其原owner保留 |
| [crates/auto-lang/test/a2gd/tscn/godot_demos/instancing/001_ball/ball.at](../../crates/auto-lang/test/a2gd/tscn/godot_demos/instancing/001_ball/ball.at) | scene Ball的材质bounce0.4/碰撞radius30/资源边，TSCN金样还须真实场景加载 | `ball.expected.tscn`；SHA256前16 `5f91e2d262cb475e` | `user-deferred/not-required (M2-TARGET-SCOPE-20261116)`：原GD/SCENE目标腿保留原源/金样及§9任务，非本版执行；同源普通Auto语义若属于其他required后端，沿其原owner保留 |
| [crates/auto-lang/test/r2a/08_generics/003_generic_fn.rs](../../crates/auto-lang/test/r2a/08_generics/003_generic_fn.rs) | 真实反向泛型输入，不改旧golden | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `eb068ed1adac2846` | 拟ace.m2.p977.source04：reverse-reference；负控/资源见V-DETAIL-02/03；原任务 B-REVERSE-I/B-REVERSE-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 按本版PY/REVERSE目标完整映射实际工具版本/运行oracle与r2a语义差异；用户后置Godot目标保留历史去向，普通Auto语义及required后端不受影响。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-PY-I`、`B-PY-V`、`B-REVERSE-I`、`B-REVERSE-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

PY批次消费1042正式最小producer/运行helper后扩完整Python语言与原旧语料；同一Python目标支持新公共标准库，不复制旧AST emitter。只需某能力的后继先消费已交PY切片，不等待GD/SCENE/REVERSE整包。其他批次原依赖/义务保留。

1042是最小Python接缝，不自动证明本包全部语言域支持；本包PY批次补齐完整Python语义并交正式切片，标准库后继不反等本包GD/SCENE/REVERSE整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 规范增量
| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-other-targets.md` | 分批沉淀实际支持/来源/拒绝及未闭行 | 不把计划当现状 | AC-01..06及所有AC-B-* |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 959行关联本批新producer/consumer与收据 | 替换追踪 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-other-targets.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-other-targets.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-other-targets.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |

外仓知识由owner落其模块Specs：UI ui/runtime/build、AGE auto-graphics、Man cli/modules、Web architecture/http-contracts/testing；语言侧只保通用接口和迁移指针，不复制第二份领域规范。

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-other-targets.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**Python等目标消费的公共标准库Linux腿**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

本条只作用于标准库Linux适用行；GD/SCENE已由用户另行后置，REVERSE及非Linux其余当前合同保留；Python解释器运行时依赖单列，不代AV/native自有实现。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-other-targets.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部命令为未来执行，本轮未运行。cwd前缀lang/ui/man/web/ag-examples分别为本Plan对应唯一tree。永久新fixture由I接到所列target；旧用例绿不能证明新ACE支持。参数尚待冻结的产品命令由I第一步读取现有runner/help与工程配置固定，之前不进入V，不声称未来runner当前已可用。

材料/外置target/生成安装副本只放D:/autostack/plan-artifacts/plan-977/下按仓/批次区分，普通target留树。AG仅受影响文件/case、窗口/GPU单实例；原ignore/平台/资源预算保留。发现/执行/排除、准备/运行/清理/总时与子终态分列，零选择/emit-only/未执行不能PASS。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-PY | AC-B-PY | B-PY-V；lang: cargo t ace-a2x a2py auto-test-case；目标执行复用现有Python adapter，记录实际解释器版本与源hash | 原Python语料的返回/输出/短路/异常/资源各对独立oracle；跨模块生成工程在Python执行，坏输入不替换旧输出。 |
| V-B-REVERSE | AC-B-REVERSE | B-REVERSE-V；lang: cargo t s2s auto-lang-core；r2a原用例复用并新增无legacy依赖控制 | 已支持Rust→Auto用例保行为；不支持形态具名拒绝；s2s重写源可在ACE执行且source map正确，无旧执行/发射反依赖。 |

最终T-05核全批收据适用性、遗漏、组合并集和原资源门；证据身份不变可复用，不反复全仓。每批独立复核不替整包final review，未闭in-scope阻execution_done。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PY-I/B-PY-V/B-REVERSE-I/B-REVERSE-V | Python旧源应逐行42、hello；r2a需原Rust输入和Auto语义，不能只证明生成文件存在。Godot脚本/场景原断言留§9及旧用例，非本版required。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PY-I/B-PY-V/B-REVERSE-I/B-REVERSE-V | Python坏模块/依赖及逆向无法表达形具名拒；本版Python/逆向工具缺席是缺证，不默填PASS/NA；Godot按用户后置，不成为缺工具阻塞。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PY-I/B-PY-V/B-REVERSE-I/B-REVERSE-V | 当前PY/REVERSE两批各自I/V/R保留，原GD/SCENE留§9范围历史；A的正向provider与B逆向工具分期，路径/资源/runtime身份不混，close回收子进程。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PY-I/B-PY-V/B-REVERSE-I/B-REVERSE-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

### 当前目标范围验证（拟实施，未运行）

| Case | AC/任务 | 输入→producer→consumer→独立oracle与负控 |
|---|---|---|
| V-TARGET-SCOPE | AC-TARGET-SCOPE；B-REVERSE-V，T-05独立复核 | 以§9原GD/SCENE ID→用户裁定→本版目标清单核范围；无Godot环境仍可核required PY，不能冒Godot运行PASS；误撤PY、REVERSE或同源普通Auto语义必须发现。该检查复核范围，不能代PY/REVERSE产品运行。 |

## 7. 验收标准

| ID | owner_stage | 标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/平台/输入oracle完整 | T-01及各I |
| AC-02 | work | 原整包能力经真实源码/工程运行，范围不减 | 全部I/V |
| AC-03 | work | 全consumer接线与权限/资源/恢复、无legacy fallback，跨仓身份准确 | 全部V及959行 |
| AC-04 | review | 批次及全包独立复核，原AC反例/health/Spec均核 | 各R及T-05 |
| AC-05 | merge | 正式来源/Spec/归档/清理完成 | T-06/07 |
| AC-06 | work | 原T-08补充义务分配至批次且全闭 | 所有AC-B-* |
| AC-B-PY | work | Python provider：原Python语料的返回/输出/短路/异常/资源各对独立oracle；跨模块生成工程在Python执行，坏输入不替换旧输出。 | B-PY-V/R |
| AC-B-REVERSE | work | r2a与s2s独立工具：已支持Rust→Auto用例保行为；不支持形态具名拒绝；s2s重写源可在ACE执行且source map正确，无旧执行/发射反依赖。 | B-REVERSE-V/R |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

### 当前目标范围验收

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-TARGET-SCOPE | work | 当前A仅PY、B为REVERSE与原收口；GD/SCENE原任务/AC/V在§9及旧源码/用例可追溯，未计PASS且不因缺Godot阻M2；同源普通Auto语义未误撤 | B-REVERSE-V、V-TARGET-SCOPE；T-05独立复核，T-06只核落地 |

## 8. 执行步骤

当前共10任务、已完成6项（六项Godot任务由用户后置，不计本版总数）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜owner_stage: work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。
  [✅ 2026-11-16] 冻结记录见§5"T-01 A期冻结记录"：前置全归档核实；来源重核（python.rs漂移重锚定 b6fff84a26408a59）；1042消费面登记；环境冻结（Python 3.14.2在机/Godot缺席→PY先行、GD/SCENE不半途开工）；命令与平台冻结；worktree .wt/lang-977/auto-lang 已建。B批（REVERSE+收口）只登记：前置M2A-08未闭。 【旧r9回执范围；r10起GD/SCENE由用户后置，原等待不再是当前门。】

- [x] **B-PY-I｜M2-A｜owner_stage: work｜AC-B-PY/AC-02/03/06**：消费1042正式producer/helper后扩Checked HIR→Python完整目标/运行时映射、模块与source map；保原Python公开支持格；接实际目标adapter，旧emitter仅作历史输入。 在§5改面接永久fixture，先冻结前置接口/命令。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。
  [✅ 2026-11-16, worktree commit 727f45616] **9/9 旧公开面家族在真实 Python 车道全绿**（永久探针套件 `a2x-worker/tests/cases/p977_py_public_surface.rs`，1.56s≤10s门）：if/for(条件for=Scalar)、func(Scalar)、**generics**(`fn id<T>`=Scalar+`ac.source.instances.scalar/v1` 935脱糖——caps-blind trim与packages binder拒`<`为具名路由事实非语言缺口)、methods(`static fn`/隐式self `.field`=TextRecord)、enum(变体模式=Aggregates)、str/struct(TextRecord)、**List/array**(PLAN-946 aggregates packages线 `PackageA2xTarget::cold_aggregates`+`use std.list.sum`)、option/result/is表达式+`??`(Aggregates)。oracle=真实 OS Python 3.14.2 执行的 `value:<n>` 独立手算值；旧print逐行形态→新观察面映射如实记录。**零车道缺口**：每构造皆有已准入路由。1042消费确认（provider全家族AXPY1/2/3+lower三层+ports）；实际目标adapter=1042既有 `ac_a2py_hir`（4 recipes），全语料case文档扫描归B-PY-V。

- [x] **B-PY-V｜M2-A｜owner_stage: work｜AC-B-PY**：运行V-B-PY及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。
  [✅ 2026-11-16, worktree commit 9fc1d2a34] 跟踪语料 `ace-m2/py-977`（8检查）经 **case线** `ac-a2py/hir-stdlib-run-v1`（ac_a2py_hir适配器→预构建a2x-worker run-py→真实Python 3.14.2，父侧独立裁判）：观察五轴=返回（value14/generic42）/输出（stdout面）/短路（value:7+stderr确定性marks序mark_b,mark_a）/异常（trap:Overflow/trap:DivByZero）/资源（record-text值面，零泄漏trace证在1042 pyready_02）。门：`ace_m2_p977_py_case_line_green`（1.79s）+`ace_m2_p977_py_wrong_oracle_is_red`（stdout轴RED局部化1.95s）。坏输入负控=本注册wrong-oracle RED+1042 `pyready_04_admission_negative_controls`（实测绿）。预构建纪律：`target/release/a2x-worker.exe` 显式预构建（2m56s，缺=具名Error非PASS，测试内不编译）。

- [x] **B-PY-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。
  [✅ 2026-11-16] 批内独立复审（局限：实施会话内，从提交工件重建）：基线 727f45616(I)+9fc1d2a34(V)+e49c99efa(spec)；**相位门** `cargo t ace-a2x a2py`=247+105例全PASS（7m0.6s，含新crate套件、case双门、1042全套、certification；a2py旧面不变绿）；健康=新代码零警告、fmt过、无调试打印。发现0阻塞项；边界如实记录（spec边界节）：旧99例字节金样属旧实现不动，新面=家族代表+路由台账承接（逐例扩充机械映射）；case线单文件（List腿crate级）；Linux归1063。PY批A切片可phase fold；GD/SCENE批待Godot（缺证不NA不半途开工）；REVERSE属B。 【旧r9回执范围；r10起GD/SCENE由用户后置，原等待不再是当前门。】

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [x] **B-REVERSE-I｜M2-B｜owner_stage: work｜AC-B-REVERSE/AC-02/03/06**：分类独立逆向工具与旧前端依赖，保既有公开命令；s2s消费新源事实，r2a保中立syn解析；不要求任意语言互逆。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17, worktree commit `f2606fa91`] **前置门核验**：M2A-08 正式成立（985 A 批 phase fold delivery `95c7fbad8`，`git merge-base --is-ancestor` 核为本树祖先）；本树 resync `56aadead4 → f0db9bd0f`（master tip，ff-only，432 commits）。**分类冻结（落 spec `docs/specs/ace-migration/m2-other-targets.md` §977/REVERSE）**：r2a=中立 `syn` 解析+自有文本发射，**不**消费 Auto 前端/AST、不经旧执行器（AutoVM）与旧目标发射器；公开命令 `transpile_r2a_file` 行为不变（37 例 golden 逐字节，round-trip 无 `/* unknown */`）。s2s=依赖旧**前端**（parser/AST）、**不**依赖旧执行器/旧目标发射（`lower_source` 旧产物逐字节不变：identity/幂等/py 套件 round-trip 既有断言保持）；"消费新源事实"落点=重写产物必须是新执行器可消费的源事实（规范方言+可映射 span），不是把旧产物换个执行器。**新增面**：(a) r2a 受检入口 `transpile_r2a_checked` + `r2a_unsupported_forms`（20 类稳定键具名拒绝；只认注释位标记，字面量/`//` 注释不误报；跨行/未闭合标记 fail-closed；`unimplemented!/todo!/unreachable!` 显式 panic 翻译不计有损；Rust 语法错误仍是 `Rust parse error:`）；(b) `EmitDialect::{Legacy,AceCanonical}` + `emit_code_with_map`（规范形只把 fn 签名的历史箭头归一为裸返回类型；`Legacy` 逐字节同 `emit_code`，字面量/注释内不动）；(c) 行级 source map `lower_source_with_map`/`lower_source_for_ace`/`source_line_for_output`（语句起点粒度 + 同形行精确细化，结构行不参与，改写行回落语句起点）。**命令冻结**：`cargo t s2s`、`cargo t auto-lang-core`、`cargo t ace-facade`（新增 ACE 触面）。**985 账本协调**：`m2-cert-985/ab-obligations.tsv` B-03 pin 随 spec 新哈希刷新（`da6d20d025624377`），B-08/B-09 由 pending→inflight（landed 待 T-06 落主后翻转）；该文件为 985 owner 资产，本批只做本 Plan 义务行的最小改动，T-06 落主时须与 985 终态核对。

- [x] **B-REVERSE-V｜M2-B｜owner_stage: work｜AC-B-REVERSE**：运行V-B-REVERSE及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17, worktree commit `f2606fa91`] 固定组实跑（Windows x64，冻结提交同内容）：`cargo t s2s` **PASS**（11 例全过，执行 1.34s / 组 2.6s；含 4 个新 `p977_s2s_*`）；`cargo t auto-lang-core` **1653/1654**（执行 35.6s / 组 36.9s，103 skipped 为原条件排除；1 红 = `stdlib_assembly::manifest::plan814_build_input_boundary::declared_path_deps_all_in_identity_sources`：`auto-tool-execution` 未列入 `IDENTITY_SOURCE_CRATES`——Cargo.toml/build.rs/manifest.rs 与本树 master 逐字节相同，master 同输入机械复放同红，**基线红非本批触面**，归批量回归 triage，不计 PASS）；`cargo t ace-facade` **PASS**（36 例，执行 2.95s / 组 4.9s；新 `p977_reverse_s2s` 3 例 0.038/0.095/0.117s ≤10s 门）。**V 判别证据**：① s2s 规范产物经 `run_ace(AceAV)` 真实执行递归 `fib(10)=55`（独立手算 oracle，非"与旧 VM 相等"）；② 历史方言（带 `->`）在同一门是 `text/source.syntax` **具名**拒绝（规范方言是可执行前提，否证"随便喂产物"）；③ 绑定期诊断 span 经 source map 回到原源第 6 行 `return add(1)`（精确行，不是产物行号或笼统语句起点）；④ `autovm/vm/evaluator` 引擎名具名拒绝（本链无旧执行器回退）；⑤ map 总性=每条产物行有映射且原行在源内。证据件：`plan-artifacts/plan-977/reverse-v/{s2s,auto-lang-core,ace-facade}-results.json`（sha16 `f181234913fbf792`/`82f456e13f702c64`/`ec93b5978a8d5767`）。无新增 ignore/平台/预算；Linux 腿按 r8 归 1063。

- [ ] **B-REVERSE-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final，核遗漏/延期/组合、warning/fmt/debug及Spec增量，缺必需批收据不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner依赖次序合入、落canonical并核实际消费者/接口同版本，全部齐才归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan，逐输出路径对账/删除scope/刷新retention，guard clean后清树；部分完成保留树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-PY-I/B-PY-V/B-REVERSE-I/B-REVERSE-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。



## 9. 复审记录

### 本轮用户授权后置的原Godot条目（历史，不计当前M2任务/AC/V）

原ID及验收留此供后续版本追溯；全部原为未实施，不记完成、不形成M2缺证门。本轮没有创建后继Plan。原代码/fixtures/expected未删除。

> | V-B-GD | AC-B-GD | B-GD-V；lang: cargo t ace-a2x a2gd auto-test-case；Godot既有runner版本/入口由I任务冻结，非纯emit | 原脚本具名case在真实Godot headless执行，类型/方法/事件错误有源位置；目标环境缺席保持未执行。 |

> | V-B-SCENE | AC-B-SCENE | B-SCENE-V；lang: cargo t a2gd ace-a2x；实际Godot场景启动/退出及资源检查，原runner和独立场景树oracle | 真实场景装载节点/属性/脚本执行符合原oracle；缺资源、坏绑定诊断；仓外搬迁后仍装载。 |

> | AC-B-GD | work | GDScript脚本目标：原脚本具名case在真实Godot headless执行，类型/方法/事件错误有源位置；目标环境缺席保持未执行。 | B-GD-V/R |

> | AC-B-SCENE | work | Godot/TSCN场景与资源：真实场景装载节点/属性/脚本执行符合原oracle；缺资源、坏绑定诊断；仓外搬迁后仍装载。 | B-SCENE-V/R |

> - [ ] **B-GD-I｜M2-A｜owner_stage: work｜AC-B-GD/AC-02/03/06**：实现新GDScript provider及运行时适配；分离脚本语义和场景树生成，不等待TSCN批次。 在§5改面接永久fixture，先冻结前置接口/命令。

> - [ ] **B-GD-V｜M2-A｜owner_stage: work｜AC-B-GD**：运行V-B-GD及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

> - [ ] **B-GD-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

> - [ ] **B-SCENE-I｜M2-A｜owner_stage: work｜AC-B-SCENE/AC-02/03/06**：用受检领域投影生成场景、资源引用与脚本绑定，包内相对路径闭合；禁止旧AST scene emitter回退。 在§5改面接永久fixture，先冻结前置接口/命令。

> - [ ] **B-SCENE-V｜M2-A｜owner_stage: work｜AC-B-SCENE**：运行V-B-SCENE及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

> - [ ] **B-SCENE-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

2026-11-16 stage: new/revision | PLAN-977 r10 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

stage: merge | phase fold (977/PY批；plan 保持 executing，同树保留至GD/SCENE/REVERSE) | PLAN-977 | r9 | outcome: pass (phase) | prepared: B-PY-R pass @e49c99efa + 相位门 ace-a2x 247+a2py 105 全PASS | landed: rebase后 --ff-only，**delivery `56aadead4`**（range-diff全等：727f45616→b286af57c、9fc1d2a34→4b9c367ba、e49c99efa→56aadead4）；master tip=delivery ✓；rebase后冒烟 p977+pyready_04 绿 | canonical_verified: `docs/specs/ace-migration/m2-other-targets.md` sha16 `83af83e29cd299f4`（PY批已验接口+边界）；无legacy ledger写 | archived: N/A（phase——GD/SCENE待Godot、REVERSE属B） | cleaned: N/A（worktree保留） | 部署观察: 纯测试+文档，无发行产物受影响 | next: 985/A可消费977/PY切片（COVERAGE Python腿A2PY路径已喂）；GD/SCENE待Godot环境；REVERSE待M2A-08

stage: review | phase (B-PY-R, 977/PY批) | reviewer: 实施会话内复审（局限随收据） | PLAN-977 | r9 | outcome: pass (phase) | reviewed_commit: e49c99efa（含727f45616/9fc1d2a34） | 相位门: `cargo t ace-a2x a2py`=PASS 247+105例 7m0.6s | findings: 0阻塞；边界（代表例vs逐例/case线单文件/预构建/Linux归1063）已入spec边界节与B-PY-V[✅]行 | next: phase fold 合入977/PY A切片（plan保持executing——GD/SCENE待Godot、REVERSE属B）

stage: work | PLAN-977 | r9 | 2026-11-16 | outcome: pass (B-PY-I) | worktree: D:/autostack/.wt/lang-977/auto-lang @ plan-977-dev | code_commit: 727f45616 | task_ids: B-PY-I | evidence: §8 B-PY-I[✅]行；新用例 p977_legacy_public_surface_runs_green_on_python 1.56s ≤10s门（10轮探针迭代，含3个具名路由事实发现：instances-cap脱糖/cold_aggregates线/is-表达式形态） | next_responsible_role: work（B-PY-V：经 ac_a2py_hir case线全语料扫描+返回/输出/短路/异常/资源独立oracle+坏输入负控）

stage: work | PLAN-977 | r9 | 2026-11-16 | outcome: pass (T-01) | worktree: D:/autostack/.wt/lang-977/auto-lang @ plan-977-dev（基面 master@b66405d6d，纯冻结核验无代码提交） | task_ids: T-01 | evidence: §5"T-01 A期冻结记录" | authorization: 用户裁定选项A（goal续行2026-11-16） | blockers: GD/SCENE批V腿缺Godot（缺证不NA）；B-REVERSE+T-05/06/07前置M2A-08 | next_responsible_role: work（B-PY-I：消费1042扩Python provider，有界决定门冻结接口，建永久fixture——PY批I/V/R为A批下一单元）

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r8：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-12 stage:new / PLAN-977 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原16任务/全部AC与V/依赖/阶段/预算和drafting 0/16；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部16个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/16；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-07 r3：细化4批，0/16。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

2026-11-16 stage:new/revision | PLAN-977 r11 | outcome:pass（规划接续合同预检；非实现/独立产品review） | next:等待985/A真实M2A-08，再由原owner按B-REVERSE-I→V→R→T-05→06→07接续；本轮无实施授权。4/10与全部10任务ID/勾选保持；技术输入与阶段门分开，PY证据边界保留，V-TARGET-SCOPE归work验证、T-05独立复核，不把正确性推给merge。977-A正式交付/现Spec身份已只读核实；旧GD/SCENE后置不变。

stage: work | PLAN-977 | r11 | 2026-11-17 | outcome: pass (B-REVERSE-I + B-REVERSE-V) | worktree: D:/autostack/.wt/lang-977/auto-lang @ plan-977-dev | base_commit: f0db9bd0f（resync master tip；原基面 56aadead4，fast-forward，432 commits） | code_commit: f2606fa91 | dependency_revisions: M2A-08=PLAN-985 delivery 95c7fbad8（祖先已核）；977/PY=56aadead4；worktree 内 985 账本 `ab-obligations.tsv` B-03 pin→`da6d20d025624377`、B-08/B-09→inflight（T-06 翻 landed） | task_ids: B-REVERSE-I, B-REVERSE-V | evidence: §8 两行 [✅]（分类/新增接口/命令冻结/固定组实跑/五条判别证据/基线红归属）；spec `docs/specs/ace-migration/m2-other-targets.md` §977/REVERSE；证据件 `plan-artifacts/plan-977/reverse-v/*.json`（sha16 记录于 §8） | blockers: 无——下一阶段为 review（B-REVERSE-R 批内独立复审 → T-05 全包 final） | next_responsible_role: review

stage: goal | plan_id: PLAN-977 | plan_revision: r11 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-977/auto-lang | commit: f2606fa91 | next: review（B-REVERSE-R 批内独立复审 → T-05 全包 final；同树停写）



## 10. 待澄清事项

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
