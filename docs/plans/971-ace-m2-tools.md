---
plan_id: PLAN-971
status: executing
feature_name: 调试LSP测试与Playground接管
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 6
current_step: 4
total_steps: 11
depends_on: [PLAN-970, PLAN-966]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-tools.md]
touched_goals: [GOAL-002]
---

# [PLAN-971] 调试LSP测试与Playground接管

## 0. 变更摘要

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

## 5. 详细设计

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
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 真实LSP owner与工具全入口的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-A-PROOF | work | 坏源诊断定位原文件，恢复后诊断消失；测试失败非零、未知协议/错binary拒绝、停止回收；合法近邻仍可执行。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共11任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

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

- [ ] **T-02｜M2-B｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。

- [ ] **T-03｜M2-B｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。

- [ ] **T-08｜M2-B｜owner_stage: work｜AC-06**：LSP已迁auto-ui/crates/auto-lsp，本包直接核其现存completion/hover/signature/semantic tokens/goto/inlay/workspace能力及编辑器客户端；将多文件诊断、测试发现和调试源码映射分批验收。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [ ] **T-04｜M2-B｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。

- [ ] **T-05｜M2-B｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-B｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-B｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-971/auto-lang`，分支 `plan-971-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-971/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: goal | PLAN-971 | r6 | round: 25 | from: work(975) | to: work(975) | status: executing | outcome: pass(975 T-02 sqlite :memory:腿) | 记录: 967钉版provider以:memory:开启（零973依赖），独立oracle复核SUM=9，双负控（bb8d56e07 5.6s）；Auto API面=console foreign缝具名后续 | next: 975 redis RESP腿→T-03..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 24 | from: work(975) | to: work(975) | status: executing | outcome: pass(975 T-01冻结+T-02 JSON encode族) | 记录: 975开工——T-01三域选型冻结（JSON自有codec/967 FFI sqlite/RESP over 974 net）+973分账；T-02首增量 std.json encode族交付（bc19182d9，14 oracle 0.09s） | next: 975 sqlite :memory:/redis腿→T-03..07；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 23 | from: work(974) | to: work(依赖链) | status: executing | outcome: pass(**974整包归档——M2A-03标准库门获正式来源**) | 记录: PLAN-974 整包A交付完成：T-05复审pass(0阻塞)→T-06落主681efa26b+spec m2-stdlib-network sha16 a8338b81e6b5f0ee+归档7534f4eef→T-07清理(worktree/分支/plan-artifacts全删,guard clean)。M2A依赖链：**973/977PY/978/974已闭**；剩973(在途)/979(在途)/981(在途)/975/976/980(部分在途980已见)/982/983/984/OS064/977GD+SCENE | next: 无主包975/976/982/983/984继续或等在途落主→985 T-A03..A05→M2A-08→971 M2-B

stage: goal | PLAN-971 | r6 | round: 22 | from: work(974) | to: review(974/T-05) | status: executing | outcome: pass(974/T-04全V行) | 记录: T-04冻结并集执行——ace-av 1257+ace-native 280+ace-source 597 全PASS（2134例）；V-01..05逐行状态+具名pending（长流背压/上传文件973/TLS-provider道）；974→5/8 | next: 974/T-05独立复审→T-06/07归档（整包A交付喁M2A-03）；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 21 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-08 WebSocket帧层) | 记录: net_tcp_write_region raw面+真RFC-6455客户端往返（101/掩码帧/无掩码回显），net974 9/9绿（206fdf901） | next: 974 TLS/proxy provider道→T-04..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 20 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-08 HTTP方法族) | 记录: std.net全方法族（put/patch/delete/head加入），net974 8/8绿（5925768e9） | next: 974 WebSocket→TLS/proxy→T-04..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 19 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-08 URL族) | 记录: std.url落地（encode族+join_path，net974 7/7绿 c397befd2）；decode/query族=子串物化核心决定具名pending | next: 974 HTTP扩面→WebSocket→TLS/proxy→T-04..07；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 18 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-08分母核) | 记录: T-08协议分母核交付（13族逐项：已交7核心=TCP双向/HTTP client+server核心/SSE解析/redirect/timeout，证据net974 6/6；具名pending=URL族11符号/TLS/proxy/WebSocket/HTTP全方法面/多跳redirect/可配置timeout/upload-download973腿；分母零删行+关闭序冻结） | next: 974实现腿（URL族→HTTP扩面→WebSocket→TLS/proxy）→T-04..07；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 17 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-03缝裁定+std.net包装) | 记录: 装配缝裁定=(c)PLAN-1041内联约定；stdlib/ace/std/net.at落地+内联包装测试真回环6010012；net974 6/6绿（bd173aff0）；packages面接线具名后续 | next: 974/T-08协议分母核→T-04..07整包A交付；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 16 | from: work(974) | to: user | status: executing | outcome: pass(974 T-03装配缝探针) | 记录: T-03首个有界决定已定案输入——console道单源为设计事实，多模块std.net包装须先裁定装配缝（console前门接纳packages产物 vs 新宿主车道复用execute_platform）；974已交面稳定（net974 5/5绿） | next: 974/T-03缝决定→包装→T-08→T-04..07；或在途包落主后985 T-A03；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 15 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-02增量4) | 记录: SSE流式解析+redirect跟随+拒绝负控交付（cf1b29c9e）；net974 5/5绿（SSE 20002/redirect 30200→20000/refused -8） | next: 974 std.net包装→T-03/T-08→T-04..07 或按裁定；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 14 | from: work(974) | to: work(974) | status: executing | outcome: pass(974回环谜题关闭) | 记录: 具名开放发现已关闭（d090a48bb）：根因=补丁脚本转义损坏客户端实发29字节LF（非内核）+非阻塞复原+paced read三修；net974 4/4绿（server腿0.56s稳定） | next: 974/SSE/redirect/timeout 或按裁定；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 13 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-02增量3)+named-open-finding | 记录: platform.net/1 server面落地（23d47952f：listen/accept/local_port+paced accept）；978已由他会话归档（M2A链推进）；server腿E2E具名挂起（回环29/31部分投递谜题，#[ignore]带因非静默） | next: 974谜题关闭/SSE/redirect 或按裁定；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 12 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-02增量2) | 记录: Auto侧HTTP协议API面交付（54e09069e）：旧a2p http_client_sync义务经platform.net/1真TCP承载，GET/builder/post三腿独立oracle精确命中（6010012），net974三测全绿 | next: 974/T-02余量（SSE/server/redirect/timeout）或按裁定；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 11 | from: work(974) | to: work(974) | status: executing | outcome: pass(974/T-02 OS薄适配增量) | 记录: 974/T-02 首增量交付（c226a7215，+754行）：platform.net/1 全栈（av-core四op/caps/预算/台账+Win32 std::net后端+mir_verify准入+core源表+native具名拒）；net_974双测绿（真实TCP HTTP往返 oracle 20043 + 六负控含准入拒零dispatch）；回归48/48 | next: 974/T-02续（http API面/SSE/server腿）或按裁定；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 10 | from: work(依赖链) | to: work(974 A批) | status: executing | outcome: pass(974/T-01冻结) | 记录: 按既定裁定模式开工974：T-01 A期冻结交付（974→r9 executing 1/8；S1选型=扩展platform_io增platform.net/1；973依赖分账=核心TCP/HTTP/SSE腿不前置、File/Stream/upload文件腿具名阻塞；worktree .wt/lang-974/auto-lang已建） | next: 974/T-02（platform.net/1薄适配+最小真实HTTP往返）；GD/SCENE待Godot；971/B 等 M2A-08

stage: goal | PLAN-971 | r6 | round: 9 | from: work(977 A批) | to: work(依赖链) | status: executing | outcome: pass(977/PY批A切片落主) | 记录: 977 B-PY-V(9fc1d2a34→4b9c367ba)+B-PY-R+phase fold 完成：**master=56aadead4**，spec m2-other-targets sha16 83af83e29cd299f4；相位门 ace-a2x 247+a2py 105 全PASS；977→4/16(T-01/B-PY-I/V/R)；**985/A 的 COVERAGE Python腿(A2PY路径)已获正式来源** | next: 无主包剩余974/975/976/980/982/983/984（M2A-03标准库门建议974/975；M2A-05 RG-A建议982/983）或在途包落主后985 T-A03；GD/SCENE待Godot；971/B 等 M2A-08


stage: goal | PLAN-971 | r6 | round: 8 | from: work(977 A批) | to: work(977 A批) | status: executing | outcome: pass(977/B-PY-I) | 记录: 977/B-PY-I 已交付（977 worktree 727f45616，977→2/16）：旧a2p公开面9/9家族真实Python 3.14.2车道全绿（1.56s），路由事实冻结（instances-cap/cold_aggregates/is表达式），零车道缺口 | next: 977/B-PY-V（全语料case扫描+负控）→ B-PY-R；GD/SCENE待Godot；971/B 等 M2A-08（977/PY批R后phase fold即补COVERAGE Python腿）

stage: goal | PLAN-971 | r6 | round: 7 | from: user(选项A裁定) | to: work(977 A批) | status: executing | outcome: pass(977/T-01) | 记录: 用户裁定选项A→977 A批开工：T-01 A期冻结已交付（前置全归档核实/python.rs漂移重锚定/1042消费面登记/环境冻结Python3.14.2在机·Godot缺席→PY批先行·GD+SCENE不半途开工/命令平台冻结；977→r9 executing 1/16，worktree .wt/lang-977/auto-lang 已建） | next: work——977/B-PY-I（消费1042扩Python provider完整映射+保旧公开格+adapter+永久fixture），其后 B-PY-V/R；GD/SCENE 待Godot环境；REVERSE属B。971/B 仍待 M2A-08（977/PY落主后补齐COVERAGE Python腿之一）

stage: goal | PLAN-971 | r6 | round: 6 | from: work(985链) | to: user | status: executing | outcome: pass(985/T-09已交来源部分)+blocked(985 A批到边界) | 记录: 985/T-09 已交来源部分交付（f0全集33/33实测、FFI行逐行绑定、986/987 pending项；985→4/14）。**985 A批当前可执行边界已到**：T-A03..A05 阻塞于 973-984/OS064 各包A批（973/978/979/981 他会在途；974/975/976/977/980/982/983/984 无主）；T-08[A] 阻塞于 RG 正式输入+File(973) | next: user——选项A：指定无主包A批开工（建议977，阻塞COVERAGE Python腿；次选974/975喂M2A-03）；选项B：等在途包（973/978/979/981）落主后985继续T-A03；971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 5 | from: work(985链) | to: work(985链) | status: executing | outcome: pass(985/T-A02) | 记录: 985/T-A02 已交付（985 worktree d6342f534，985→3/14）：m2-cover两配方语料+冻结父表真值+四准入族具名红控绑定+V-A-COVER命令冻结 | next: 985/T-09/T-08[A]（已交来源部分）或用户裁定转无主包A批；971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 4 | from: work(985链) | to: work(985链) | status: executing | outcome: pass(985/T-A01) | 记录: 985/T-A01 义务矩阵+V-A-MAP红控已交付（985 worktree 0dff19cd4，簿记 247c345e6，985→executing 2/14）：162行 `autolang.ab-obligations/1` 矩阵（959冻结行+FFI/LIB增补+早期切片去重）+ 7例真实红控（缺行/重复/错source/非收据路径/词表无owner/pending虚报,0.03s绿,`cargo test -p auto-lang-core --lib obligation_ledger`） | next: 985/T-A02（下一就绪）或按用户改道无主包A批；971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 3 | from: user(续行指令) | to: work(985链) | status: executing | outcome: blocked(外部依赖扩大) | 记录: 2026-11-16 续行指令"推进985 A批至T-A05/V-A-LAND后回971 B"已执行第一步——985/T-01 A期冻结完成并提交（df9a75568，985 r16 executing 1/14，worktree .wt/lang-985/auto-lang 已建）；985 冻结结论：**M2A-08 需 973-984/OS064 各包A批正式落主**，其中 973/979/981 在途属其他会话、978 reviewed收口中——本会话不可替代；无主可开工的包：974/975/976/977/980/982/983/984；985 下一就绪任务=T-A01 义务矩阵 | next: user/work——后续轮次按续行指令继续（985 T-A01 或无主包A批），971/B 仍待 M2A-08

stage: goal | PLAN-971 | r6 | round: 2 | from: merge | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-971/auto-lang（保留） | commit: 2f1eed27f (lang) / c1561d94 (auto-ui) | next: user——M2-B 阻塞于 M2A-08（985 A批未闭），A切片已正式合入可消费；同目标恢复B期即可

stage: merge | phase fold (A期PROOF切片；plan 保持 executing，同树保留至B——不归档/不清理/plan-artifacts 在途保留) | PLAN-971 | r6 | outcome: pass (phase) | prepared: 审查基线=phase re-pass e1a600ebe/2f70bbf7 + 相位门绿（ace-facade 28例18.0s、UI全包）；spec增量=新增 `docs/specs/ace-migration/m2-tools.md`（A切片28行已验接口与边界） | landed: 两仓均 rebase 后 `--ff-only`（lang master 增量仅978 docs提交、ui master 增量仅085 docs提交，零代码触面，证据基线有效；rebase后冒烟 fixture 0.50s/CLI 0.18s 绿）；**lang delivery `2f1eed27f`**（range-diff 全等：a2faad900→a1b78d9b4、e1a600ebe→82d659bba、fb838e3da→2f1eed27f）、**auto-ui delivery `c1561d94`**（a6065a59→b156de4e、2f70bbf7→c1561d94）；master tip=delivery ✓ 两仓 | canonical_verified: `docs/specs/ace-migration/m2-tools.md` @2f1eed27f，sha256前16 `c69d0afd1aa1fdf3`（5100字节）；无 legacy ledger 写；specs.json 未触碰 | archived: N/A（phase——B期未完不归档） | cleaned: N/A（phase——worktree/分支保留：`.wt/lang-971/{auto-lang,auto-ui}` 均与各自 master tip 同步；detached 依赖兄弟 auto-web/auto-down 保留至B；plan-artifacts/plan-971 在途保留含规范草案源稿） | 部署观察: 本切片纯库API+测试，无发行二进制/打包产物受影响；批量回归不因相位落地触发（971%5=1，归档时另核） | next: M2-B 全部任务阻塞于 M2A-08（985 A批未闭），goal 以具名阻塞停止

stage: review | phase re-pass (B-PROOF-R, A期PROOF切片) | reviewer: 同上局限（实施会话内） | PLAN-971 | r6 | outcome: pass (phase) | reviewed_commit: e1a600ebe（lang，含a2faad900+R-01修复）/ 2f70bbf7（ui，含a6065a59+R-02修复） | 相位门: `cargo t ace-facade`=PASS 28例 18.0s（热构建，冻结修复版）；`cargo test -p auto-lsp`=全绿（10 lib+0+4 protocol+2 新CLI+4 integration，10.8s 含构建）；修复复验：fixture 0.82s 绿、project单测绿 | F-971-R-01/R-02 闭合（closure: 发现→不变量[世代内预算/取消不丢]→修复→单测+端到端负控→当前结果全绿；剩余边界=端到端墙判别受冻结步预算限制，如实记录于规范边界） | 机器占用：等待外部 `cargo t ace-artifacts ace-semantics ace-a2x ace-av ace-native generated-delivery-controls ace-m2-build`（01:32启动）至0进程后启动lang门；ui门前再查（无输出=空闲）；ace-facade启动瞬间观察到12个进程（前任务收尾/新任务启动不明——本组结果18s热跑不受影响，如实记录） | next: phase fold 合入A切片（两仓），plan保持executing，同树保留至B（M2A-08前置）

stage: goal | PLAN-971 | r6 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-971/auto-lang | commit: a2faad900 (lang) / a6065a59 (auto-ui) | next: review（B-PROOF-R，A切片独立复审）

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

## 10. 待澄清事项

**M2-B 开工阻塞（2026-11-16 goal 会话具名）**：M2-B 全部任务（T-02/03/08/04 及收口 T-05/06/07）按§5阶段合同须先核 **M2A-08 已合入收据**（PLAN-985 A批 T-A05/V-A-LAND，当前985 r15 drafting 0/14 未闭）。解锁动作：985 A批完成并正式合入 M2A-08 收据后，以同一目标（`/goal 完成计划971` 或等价授权）恢复本计划 B 期——同树 `D:/autostack/.wt/lang-971/{auto-lang,auto-ui}` 保留在途，resync 后继续。B期开工时还须按§5核 A接口无变化（A接口变化回能力owner修复并定向更新A收据）。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
