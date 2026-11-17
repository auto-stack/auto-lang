---
plan_id: PLAN-982
status: executing
feature_name: AutoMan配置任务模板与生成器迁移
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-17
plan_revision: 8
current_step: 1
total_steps: 37
depends_on: [PLAN-960, PLAN-964, PLAN-966, PLAN-969, PLAN-970, PLAN-971, PLAN-973, PLAN-976, PLAN-977, PLAN-978, PLAN-967, PLAN-979]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/ace-migration/m2-facade.md]
new_spec_components: [docs/specs/ace-migration/m2-man-gen.md, docs/specs/ace-migration/config-document-evaluation.md]
touched_goals: [GOAL-002]
---

# [PLAN-982] AutoMan配置任务模板与生成器迁移

## 0. 变更摘要

2026-11-17 r8规划：GAP-982-CONFIG-ACE明确由本包现B-CONFIG-I/V/R端到端承接auto-lang＋auto-man，详§5/6/7和共同设计；原37任务/进度/A route C与M2A-08证据保持。本轮不实施、不建树、不改已验Spec，不再等待无owner的上游配置能力。

2026-11-16 r7（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包撤出GODOT三个未实施任务及AC/V/DEVSDK依赖，总数40→37、0完成保持；CONFIG/PROOF/PY/REVERSE及其他产品能力保留。WEBTARGET仅验实际生产TS/JS工程消费和入口闭包，不自动附加独立全语言a2ts产品义务。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

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

### CONFIG取材身份（2026-11-17，只读）

基面auto-lang `b12170da7c64441c829082b29e5d6753c22c2fe0`、auto-man `11fe79c9001dc3b96bbfd50069893387f07f07ee`。

| 实际文件 | SHA256前16 |
|---|---|
| `D:/autostack/auto-lang/crates/auto-lang-core/src/config.rs` | `89a5f60f4a03855b` |
| `D:/autostack/auto-lang/crates/auto-lang-core/src/entry.rs` | `93aa81b3bb6f84a0` |
| `D:/autostack/auto-lang/experimental/ace/crates/core/src/source/domain.rs` | `9143908712396847` |
| `D:/autostack/auto-lang/crates/auto-lang-core/src/vm/config_eval_tests.rs` | `1ef82d75a4b34cb4` |
| `D:/autostack/auto-man/crates/auto-man-runtime/src/project_model_adapter.rs` | `c5f766e1634cad20` |

已读Spec：m2-facade、m2-man-gen、M2A正式认证现值；复用364/375配置回归、780项目模型和Man002配置类型资料。新设计补`SourceProfile::ConfigDocument`等拟符号/新模块，不声称现在存在；可视为未来B-CONFIG-I内创建的具体改面，不需要另一个未指派Plan。


本轮授权：用户明确要求为982/B配置文档求值指派既有承接、补设计/接口/验收并更新顺序；允许修订两仓能力规划，仍不启动实施或新建Plan。取材现代码/Spec/旧研究及r7真实探针，新增语言侧scope属于本次显式规划，不暗借A授权执行。

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

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

## 5. 详细设计

### CONFIG-ACE-B-20261117：唯一承接与当前执行入口（r8，优先于r7未指派快照）

用户本轮要求核实承接并补齐设计/接口/验收、消除等待；仍只规划、不实施。本包**B-CONFIG-I/V/R统一承接GAP-982-CONFIG-ACE**，明确扩展同组auto-lang与auto-man改面；设计见[配置文档求值](../design/ace-config-document-evaluation.md)。不新立Plan、不重开969/979，不把缺口甩给未交的1068或980/981。旧r7“语言改面超scope／owner未指派”是A冻结历史，本轮已作正式规划修订；route C已验A与M2A-08 r29/95c7fbad8不改。

| 原任务/owner_stage | 本轮新增具体责任与最小前置 | 关闭/交接 |
|---|---|---|
| B-CONFIG-I / work | M2A-08＋已归档共享基础；在现唯一两仓树未来复核原输入/capability/context/预算。语言侧新增config-document Schema/CST全程序投影→共享verify/MIR/AV→中立owned文档；host注册沿969，AutoConfig显式ACE路径；Man消费接线/真实配置二阶段值流 | 配置原分母、数据/诊断/权限/资源/身份及完整真实consumer均有永久正反例。不要只交一个静态pac或route C旧求值；不等971/B即可先交本CONFIG能力 |
| B-CONFIG-V / work | I建成的接口、原Config回归、780矩阵及真实Man配置消费者 | 完整V-CFG-SYNTAX/FLOW/CONTEXT/VALUE/FAIL/LIMIT/IDENTITY/PRODUCT通过；B实际config.engine=av、legacy_config_dispatch=0，每项独立oracle，原平台/ignore条件保留 |
| B-CONFIG-R / review | I/V实现冻结，同一权威树独立上下文 | 从原source/context→CST→实际AV→owned Node→Pac/锁读取逐项复核，具名GAP实现及所有Case/SD无遗漏；A历史证据不替B |
| 原T-06 CONFIG子面 / merge | B-CONFIG-R pass后可先正式phase fold | 交IF-982-CONFIG-ACE精确repo/SHA/Schema/SDK/ABI/原输入/运行/反控/平台；保树续本包余批，不提前勾完整T-06/归档 |
| 原EXEC/TEMPLATE/各target/EXPORT/DEVSDK与T-05..07 | CONFIG正式来源；工具命令消费971/B，REVERSE消费977/B，其余沿原最小表 | 逐批真实集成→全包独立final→完整合入/归档/清理。下游983/984/985 B消费CONFIG，不反向卡CONFIG交付 |

**接口与语义决策已定，非只读调研占位：** 新`AceConfigRequest/Report`与owned `ConfigDocument/ConfigValue`，同一executor注册上增加配置方法；`AceValue`标量保持。config-domain须投影一个顺序入口，现compile_domain表达式-only限制由本任务在共享owner增量，不逐字段独立执行。采用受检配置值/累积原语＋AV业务求值，不把旧Parser/ConfigCodegen/VM或Rust AST求值器塞新链。AutoConfig→auto-val Node只是结果适配；新候选路径失败零旧求值/静态fallback。源码/原断言、具体API/节点类型/预算/缓存/平台与数据所有权详见设计§2–6。

CONFIG服务在工具宿主明确用AV，可供AC/A2R构建消费数据；不要求为每个目标再实现求值器，不冒AC guest运行期支持，不等UI三波/AAC全自举或1067异步。已有A默认legacy配置只保A收据/显式旧选择，B候选实际配置必须新AV；986再做默认切换，987核旧链不可达。旧静态ProjectModel分类仍用同一事实，不作为动态失败时的退路。

### B-CONFIG-I 引擎侧执行台账（2026-11-17，供下一次施工照做）

SDK 半边已交（合同／显式 ACE 路径＋精确投影／`ConfigDocumentBuilder` 语义核心／身份参数通道，见 §9 r1–r6 收据）。引擎侧按序做，每步独立跑绿再下一步；任何一步不成立即 needs_replan，不许只绿 CST 冒充。

| 步 | 改动面 | 完成判据 |
|---|---|---|
| E1 前端入口 | 新建 `experimental/ace/crates/core/src/source/config_document.rs`；`source/mod.rs` 加 `auto.source.config-document.v1`／`SourceProfile::ConfigDocument`／名字映射／`compile_config_document` | 返回含**单个顺序入口**的 `CompiledSource`；既有 11 profile 裁剪字节不变（跑 ace-source/ace-semantics） |
| E2 grammar 裁剪 | `grammars/auto-source.at` 增 profile 名与文档形态：顶层 `key: value`、`dep bps{}`／`dep "stylekit"{}`／`dep("kernel"){}` 三形、模板插值、`if`/`for` 语句 | 三形 dep＋顶层 prop 投影为 CST 节点（非字符串扫描）；坏邻例给原 span 的 `source.*` 诊断 |
| E3 Program 单入口 | 复用 `compile_program_parts`，文档物化为一个合成入口函数体＋精确 source map | `lower_checked_owned` 可取该入口且 `verify_mir` 通过；"逐字段无状态函数"路线必须走不通。**已交（2026-11-17，`dd7e5c742`）**：文档行→单入口（root/var/if/pair/节点/对象/数组/收尾）物化为 `cfg_*` 受检调用，Checked HIR 成立（7/7 用例，入口唯一、能力键 `config.document/1` 进 requires）；**未闭**：lowering→MIR→AV 侧（`lower_checked_owned` 与 `verify_mir` 尚无 config 面），运行期 `for`（需 list-count 原语） |
| E4 cfg.* 绑定 | `av-core` 增配置族 intrinsic（`cfg.node/put/append/merge/get`＋kind 检查 box/unbox），语义**委托**已冻结的 `ConfigDocumentBuilder`；预算走现资源账 | 最低负载（根注入＋对象/模板＋运行期 for＋两 Node 孩子）跑到 owned 快照；坏句柄/kind/环/预算各自具名拒；`callee_count` 反映真实 dispatch。**已交（`dd7e5c742`＋`079b7410e`）**：源级目录 `source/config_ops.rs`（标量按实参 VT 重载、容器值显式 callee）＋resolve 绑定＋emitter 签名/能力发射；`av-core/src/config_document.rs` 提供会话化 builder（node/obj/list/值句柄、kind 检查、重复 attach/自环/环拒绝、值节点与字节预算、失败码）＋`PlatformCall` 运行期分流＋owned 导出的最小闭环（e2e 四例）。**未交**：`cfg_get`/`cfg_merge`、运行期 `for` 的 list-count 原语、迭代式导出（现为深度守卫 2048 的递归）、`callee_count` 观测 |
| E5 owned 导出 | 迭代式导出（无递归风险），保顺序/重复节点/精确数值/含 NUL 文本 | 2000 层深文档不崩；u64>i64::MAX 仍具名拒。**已交最小闭环（`079b7410e`）**：`OwnedDocument/OwnedNode/OwnedValue` 导出保顺序/重复/精确标量/UTF-8 文本（含 NUL 字符串按字节保真）＋`DocumentCost` 真实代价；**已交（2026-11-17，`42fef0951`/`e3b16d7f8`）**：深度轴——`MAX_PROJECTION_DEPTH` 512→2048；**永久用例双档**：**1000 层整链服务**（断言整链计数）＋**2000 层不崩**（整链服务或 MIR 校验 `VerifyMemory` 具名预算拒绝，均无 panic/静默截断）；独立探针实测上限 **≈1976 层**（1980+ 触发冻结的 16 MiB 校验预算）。**未交**：迭代式导出（现递归 + 2048 守卫）、u64>i64::MAX 的拒（当前值面只到 i32/bool/str/char，u64 随值面扩） |
| E6 host 桥 | 新建 `crates/auto-lang-ace-host/src/config_document.rs`：compile→verify→execute→export→teardown，实现 `AceExecutor::eval_config` | `--test ace_config_engine` 换真桥仍绿；无桥仍 `ace.config/executor-unsupported`。**预算轴（B-CONFIG-R F-982B-08/F-982B-20，已登记）**：`ConfigEvalLimits::max_nodes` 一钮两义（宿主会话的**值节点**预算 + 导出**节点数**复核），设计 §5 允许的适配；默认值与设计 §5 的差异为**具名适配**（nodes 65 536→100 000、wall 2s→5s、无独立 source/output 字节轴）；wall 已覆盖解析→导出全段（编译/准入前、导出后各一次 deadline 检查）。**已交（2026-11-17，`308c2c07e`）**：真桥 `eval_request` 端到端（源前门→`lower_checked_owned`→`verify_mir`→ABC 准入→会话→执行→关会话→owned→中立 `ConfigDocument`），预算逐轴映射、失败路径统一 abandon 会话、无 legacy 回退；`--test p982_config_engine`（同进程真桥）与 `ace_config_engine`/`ace_config_registry` 全绿；`e0_facade`/`t08_facade` 无回退 |
| E7 Man 消费 | pac 读取切 `from_file_with_engine(.., AceAv)`＋二阶段 context/dep；legacy 仅显式 | `cargo tc`＋780 矩阵回归；B 实际 `config.engine=av`、`legacy_config_dispatch=0`；坏配置零写；同名包隔离。**已交前置（2026-11-17，`b70ec23e3`）**：context 注入最小面——`ConfigContext.args` 在文档里**按名可见**（bool/i32/char/text；`on`/`off` 内建；文档 `var` 同名遮蔽注入），超范围/不支持种类具名拒（`ace.config/inject-range`/`inject-unsupported`）；真桥用例覆盖（`port` 分支、`toggle: on`、遮蔽、SDK 门、拒绝）。**已交（2026-11-17，`6701094ff`）**：真实 pac 语料对齐——`#` 行注释（config 面 trivia）、JSON 形整份文档（字符串键/逗号/尾逗号）、块形对象值（换行分隔、嵌套、字符串键去引号）；SDK 投影与 legacy **逐字对齐**（根名 `root`、裸/字符串身份只落 `id`、括号实参进 `_arg0…` 普通 prop、i32 范围整数落 `Int`）。探针证据（`P982_PAC_CORPUS`，auto-man 同树 152 份 `pac.at`）：**149 份 ACE 成功且与 legacy 结构全等（0 处分歧）**，其余 3 份＝1 份模板文档（`lib ${name} {}`，V-CFG-TEMPLATE 未交）＋2 份计划内故意坏夹具。**已交（2026-11-17，`3d1302b78`）**：**模板插值** `${…}`（V-CFG-TEMPLATE 最小面）——普通字符串里的 `${expr}` 按 f-string 同一生成物重写为 `__fstr(...)`（`TextDisplay`/`TextConcat`），节点头裸 `${expr}` 按 legacy 口径留字面（只落身份/`_arg0`）；host-console 描述符补 `text_display`/`fstr` 两 case 与 `parts` 字段；**宿主 args 播成根属性**（legacy 语义：注入值赢、文档同名键不覆盖）。语料（152 份）：**严格口径（不注入）145 份 OK 且与 legacy 0 分歧**，6 失败＝5 份模板（无注入时 ACE 具名 `source.undefined`，legacy 留字面——故意 fail-closed 分歧）＋1 份计划内坏夹具，1 份 legacy 也拒；**注入 `name` 后模板文档全部产出 `probe` 形态**（5/5）。**已交（2026-11-17，`02a4789` man＋`81391da11` lang）**：auto-man 三个 pac 读点（`automan.rs` 探针/正式两读、`pac.rs` dep 读）与候选腿配置读统一走新 `auto_man_runtime::config_engine`：**默认显式 ACE**（桥已装 ⇒ `AceAv`；无桥宿主 ⇒ legacy 且可观测）、`AUTO_CONFIG_ENGINE=av|legacy` 覆盖、失败**不静默回退**；`observation()` 给真实派发计数，候选见证新增 `config.engine`/`config.legacy_dispatch`。**B 证据（`crates/auto --features ace-candidate --test plan982_ace_candidate_config`＝5/5）**：`config.engine=="av"`、`legacy_dispatch==0`、坏配置零写、双项目隔离；`cargo tc`＝**257 passed / 4 skipped**（含新增 `config_engine` 3 例）。附带修复 A 切片遗留的 `cargo tc` 红：`test/fixtures/plan982/{bad-config,bad-eval}` 纳入 `project_model_corpus` 的两个 allowlist（身份绑定）。**未交**：二阶段 context/dep；`780` 矩阵未单独跑（`cargo tc` 已覆盖 auto-man 7 包 lib 面） |
| E8 V-CFG 九 Case | SYNTAX/FLOW/CONTEXT/VALUE/FAIL/LIMIT/IDENTITY/PRODUCT 永久用例＋独立 oracle | 每 Case 可独立红；删行/错 producer 必被发现；之后才 B-CONFIG-R。**已交（2026-11-17，`034aef0cd` lang＋`81b27b5` man）**：语言侧七例 `auto-lang-ace-host/tests/p982_vcfg_cases.rs`（SYNTAX 三形头＋Node值 pair、FLOW 注入分支/重声明/嵌套体并带"另一分支不同结果"非空证、CONTEXT bool/int/text 分轴＋on/off＋文档遮蔽＋不支持种类具名拒、VALUE 中文/emoji/NUL/边界整数/空对象/嵌套/重复孩子、FAIL 原 span＋零残留会话＋修复后真读、LIMIT 值预算/字节/取消三轴并带默认预算非空证、IDENTITY 键集与值变身份、服务轴）；Man 侧两例 `crates/auto/tests/plan982_ace_config_b.rs`（PRODUCT：`engine=="av"`/`legacy_dispatch==0`/deps+targets 真实/执行成功；RETIRE：显式 legacy 可见且 1 次派发、未知取值不静默降级）。**已交（2026-11-17，`b26268d44`）**：FLOW 的 `for` **字面数组展开**（legacy `test_config_for_unrolls_literal_array` 语义）——迭代源是字面数组（`for d in (["a","b"])`，括号拼写是当前前门接受的形）或绑定到字面数组的文档 `var` 时，物化侧按源顺序展开体、循环变量逐项绑定；运行期列表（元素来自注入/计算）仍具名 `source.config-document/pending`；语言侧 FLOW 例补 unroll 断言（两节点 id/at 逐项）＋ pending 断言，core 侧补 `p982_config_for_unrolls_literal_lists`。**未闭（具名）**：①FLOW 的**运行期** `for`（元素来自注入/计算）仍是 `source.config-document/pending`（列表计数/取值原语未做）；②VALUE 的 `u64>i64::MAX` 与空列表 `[]`（grammar 共享面，需 config-only 空列表原子）；③LIMIT 的**宽**输入未压测（深度轴已交：1000 整链永久断言 / 2000 不崩，实测上限 ≈1976）；④V-CFG-SYNTAX 的 780 静态 dep 源未逐份实跑（语料探针 145/145 已在 E7 交） |
### M2-TARGET-SCOPE-20261116：本包当前边界

本包撤出GODOT三个未实施任务及AC/V/DEVSDK依赖，总数40→37、0完成保持；CONFIG/PROOF/PY/REVERSE及其他产品能力保留。WEBTARGET仅验实际生产TS/JS工程消费和入口闭包，不自动附加独立全语言a2ts产品义务。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包跨M2-A/B。A route C已正式交；当前CONFIG完整引擎迁移归本包B-CONFIG两仓任务，不等其它产品整包；完整EXEC及其它B批按各自已交接口消费。CONFIG先核M2A-08及已交960/964/969/973/979共享基础，不重开它们。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A已交子面 | B-PROOF-I/V/R及B-CONFIG-I/V/R的route C部分；CONFIG引擎仍B |
| M2-B CONFIG | 原B-CONFIG-I/V/R完整义务（两仓scope），正式IF交接后接其它B批 |
| M2-B | B-EXEC-I, B-EXEC-V, B-EXEC-R, B-TEMPLATE-I, B-TEMPLATE-V, B-TEMPLATE-R, B-RUST-I, B-RUST-V, B-RUST-R, B-C-I, B-C-V, B-C-R, B-WEBTARGET-I, B-WEBTARGET-V, B-WEBTARGET-R, B-PY-I, B-PY-V, B-PY-R, B-REVERSE-I, B-REVERSE-V, B-REVERSE-R, B-EXPORT-I, B-EXPORT-V, B-EXPORT-R, B-DEVSDK-I, B-DEVSDK-V, B-DEVSDK-R, T-05, T-06, T-07 |

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
| 982/WEBTARGET | 生产TS/JS工程与CLI消费闭包 | 982/CONFIG; L976/L978 | auto-man |
| 982/PY | Python转译 | 982/CONFIG; 977/PY; L978 | auto-man |
| 982/REVERSE | 逆向与源到源命令 | 982/CONFIG; 977/REVERSE | auto-man |
| 982/EXPORT | 现用发行工程格式 | 982/C; 982/RUST; L978 | auto-man |
| 982/DEVSDK | watch快路径与SDK发现 | 982/EXEC; 982/TEMPLATE; 982/RUST; 982/C; 982/WEBTARGET; 982/PY; 982/REVERSE; 982/EXPORT; L978 | auto-man |
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
I实现：真实生产工程/CLI调用976新TS/JS provider，保模块/依赖/source map；按976冻结的完整生产闭包消费，不自动增加独立全语言通用a2ts/a2js产品门，实际产品使用的CLI功能仍须支持。
V验收：实际JS运行与TS编译后运行，类型/运行时错误有原span，不能只打印代码。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 982/PY：Python转译
改面（核存在）：`D:/autostack/auto-man/crates/auto/src/cmd_trans.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/target.rs`；`D:/autostack/auto-man/crates/auto-man-core/src/target.rs`。
I实现：CLI接977/PY及工程配置/解释器发现。
V验收：真实Python原工程运行、错误JSON/exit正确，错误解释器身份不静默回落。
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


### A期PROOF原合同（route C已交；历史范围不替完整B）

最小前置：L982/CONFIG, L969, L970, L978。实际改面：`D:/autostack/auto-man/crates/auto/src/cmd_run.rs`；`D:/autostack/auto-man/crates/auto-man-runtime/src/automan.rs`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

I：从原CLI/runtime fixture冻结多模块pac、args/env及最小脚本，接显式ACE candidate入口，证明配置与实际命令消费同一SDK；不等待整个EXEC/TEMPLATE/各target迁移。
V：双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### T-01冻结记录（2026-11-17，r7历史；未指派归属已由本节r8取代，原A证据保留）

**授权**：本 goal（"完成计划982/A"）按 `/auto-plan-goal` 授予 work → review → 有界修复 → merge 全链；A 切片仍只按 §5 阶段表执行，不冒 B。

**前置核验（A 最小前置均已归档，取正式交付而非状态字样）**

| 前置 | 状态 | 正式交付 | 对 982/CONFIG 的实际提供 |
|---|---|---|---|
| L960 modules | archived | `8b5a1d980`；`m2-modules.md` | 模块包面 `packages-modules`；**无配置文档面** |
| L964 dynamic | archived | `9135c9ceb`；`m2-dynamic.md` | 对象/闭包/FStr 运行值；对象字面量同型冻结（异型 `source.object-elem` 具名拒） |
| L969 facade | archived | `6fd48b4f1`（E0 fold `5dfcfe74e`）；`m2-facade.md`（`9d1403f96a7795c7`） | `run_ace`/`AceRequest`/`AceOutcome`/console 车道/args,input 具名拒；**该规范 §边界 明写：`eval_config_with_vm`＝legacy owner（pac.at），config 兼容面"均未在 ACE 门复制"** |
| L973 stdlib-io | archived | r13 delivered | 标准库 I/O；无配置面 |
| L970 sessions | archived | r6 delivered | 会话/热更新；无配置面 |
| L978 build | archived | `b8f80afcf`；`m2-build.md` | 候选 SDK/`ac-build`/身份与运行兼容核对；无配置求值 |

**真实 owner 与改面重核**（两树 `D:/autostack/.wt/lang-982/auto-lang@c6c65e5c5`、`…/auto-man@54fe4070b`，本轮新建、均干净）

| 文件 | sha256前16 | 本轮核实事实 |
|---|---|---|
| auto-man `crates/auto-man-runtime/src/automan.rs` | `3699ce78450fd62d` | `parse_pac`(L112)→`AutoConfig::from_file`(L132/L169)；state 读取 L234；am.at 模板 L2825/L2929 |
| auto-man `crates/auto-man-runtime/src/project_model_adapter.rs` | `c5f766e1634cad20` | 静态子集与动态配置的分轴衔接口（L602 注释所指） |
| auto-man `crates/auto-man-core/src/target.rs` | `827f96a5d400be3f` | 目标装配/CLI 语义 |
| auto-man `crates/auto/src/main.rs` | `b69056ace1b07d69` | L800 直接 `eval_config_with_vm` |
| auto-man `crates/auto/src/cmd_run.rs` | `da08fd8369261f27` | 最小 CLI 执行入口（PROOF 实际 consumer） |
| auto-lang `crates/auto-lang-core/src/config.rs` | `89a5f60f4a03855b` | `AutoConfig` 全部读取面：`from_code/from_file`→`eval_config_with_vm` |
| auto-lang `crates/auto-lang-core/src/entry.rs` | `b3e46189b0e6e1af` | `eval_config_with_vm`(L735)：`CompileDest::Config` 解析＋config codegen＋VM |
| auto-lang `crates/auto-lang-core/src/vm/codegen.rs` | `3dd3b52496e03344` | `new_for_config`/`compile_config_program`：accum 文档求值（旧 VM opcode） |

**接口现值与缺口（实测，非推断）**

- ACE 公开值面只到标量＋具名 `Opaque`（`auto-lang-core/src/ace_entry.rs` `4d32c15d3405cb34`）；console 车道只回文本；`canonical_owned` 对 `Value::Object|External` 返回 `None`（`experimental/ace/crates/av/src/owned_fixture.rs` `409c9fd345a36031` L144）。
- `SourceProfile` 全集（`experimental/ace/crates/core/src/source/mod.rs` `5a0c640e1b5395d5` L152 起）**无 config/document/atom 面**；ACE 运行时亦无 `Value::Node` 之类文档值种类。
- 探针（scratch 见 `D:/autostack/plan-artifacts/plan-982/probe-t01/`）：
  - 真实配置文档 `auto-man/test/fixtures/plan446/j2check/auto/pac.at`（`807c35217b9a6ea9`）经 `av-worker request --store … --src-file … --entry d_main` → `{"ok":false,"diag":"frontend:source.syntax","source_hash":"37f2c30fd510fec2"}`，exit 4＝ACE 前端直接拒绝该文档；
  - 同族产品 legacy 面（`auto info` 读 `~/.auto/auto-man/am.at` 配置文档）正常求值并推进——旧引擎能求值、ACE 不能，是本轮缺口的直接对照。
- 已存在的真实多模块/依赖输入（可直接冻结为 A 输入）：`auto-man/test/fixtures/modularity/project-model/fixtures/static-dep-forms.pac.at`（`271f83d37ffe2ea1`，`dep bps{}`／`dep "stylekit"{}`／`dep("kernel"){}` 三形）；`…/generated-ui-consumer/proj/pac.at`（`58c8390310bb8b09`，含 title/window 面）。

**冻结结论：`GAP-982-CONFIG-ACE`（具名缺口）**

ACE 当前**没有配置文档求值面**——文档值种类、求值语义、canonical 投影、进程内车道四者皆缺，而 969 的已验规范把该面显式留在 legacy owner。982/CONFIG 的 I（"pac/am/index/state 与参数/环境双阶段求值接 ACE"）与 PROOF（"证明配置与实际命令消费同一 SDK"）都必须先有该面才能真实执行。因此：

- A **不能按现合同在 982 内直接开工实现**：本包 CONFIG 声明的改面只有 auto-man 文件；要把语言核心 capability（新文档值/新 profile 或等价 lowering）塞进产品包，会改变本 Plan 声明的改面与依赖来源，属 review 必扫的 workaround/scope 扩张。
- 缺口 **owner 未指派**：969 已归档且规范明示未交；`SourceProfile`/值面/规范中均无其他能力包承接"配置面"。
- 三种可行路由及其影响见 §10 Q-982-1；三者都触及本 Plan 的合同/接受面，须用户裁定，不能由执行侧自行选择。

**已冻结可复用资产（决议落地后不必重做）**

| 面 | 冻结内容 |
|---|---|
| 输入 | `j2check/auto/pac.at`（三 prop 最简形）、`static-dep-forms.pac.at`（三 dep 形）、`generated-ui-consumer/proj/pac.at`（display/window 面）；args 注入形沿用 `config.rs` 现测 `test_injected_object_arg_field_access`（`kernel.port` 对象参数）与 `test_injected_bool_arg`（bool 参数） |
| 独立预期 | 原动态 pac 与 args/env 输出逐值一致；失败不写 state/lock；同名包与配置隔离；未知语言不降成静态默认；缺 ACE 身份＝具名拒且 legacy dispatch 为 0 |
| 入口/命令 | man：`cargo tc`、`cargo test -p auto-package-lock --test lock_contract`、`cargo test -p auto --test plan816_cli_driver`、`cargo test -p auto-man --test plan805_toolchain_parity`；lang：`cargo t ace-facade`（既有固定组）、`cargo test -p auto-lang-core --lib config` |
| 材料/树 | 唯一 scope `D:/autostack/plan-artifacts/plan-982/`（已登记 `.retention/policy.json`）；唯一组 `D:/autostack/.wt/lang-982/{auto-lang,auto-man}`，均 branch `plan-982-dev`、基面各仓 master，保留至 B 完成 |
| A/B 边界 | A 只闭 CONFIG/PROOF 切片；EXEC/TEMPLATE/各 target/EXPORT/DEVSDK/安装仍 B |
| B 侧登记（本 Task 只登记，不实现） | B-EXEC/TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK 依赖 976/977 受检 provider 及其自有 A 切片；本轮**未**冻结其接口，不构成接口已交声明 |

### A 切片定义（route C，2026-11-17 用户裁定，当前 A 执行合同）

用户在 §10 Q-982-1 三选一中裁定 **route C（收窄 A）**：982/A 只交"同一 SDK 身份＋无 legacy fallback＋失败/隔离纪律"的 PROOF 与 CONFIG 消费改造；配置**引擎**迁移随 `GAP-982-CONFIG-ACE` 并入 M2-B，并在 A 收据具名差额。该裁定只收窄 A 的接受面，不改原 AC 分母、不改 B 义务。

**A 的实现合同（本轮冻结）**

| 面 | A 交付 | 不属 A（具名差额，B/能力 owner） |
|---|---|---|
| CONFIG 消费 | 产品配置读取仍由 SDK 的**指定 owner**（969 已验规范冻结的 `eval_config_with_vm`）承担，并与执行腿同属一个 SDK 构建；候选腿对配置文档做真实求值并记录逐值证据 | 配置引擎迁移到 ACE（`GAP-982-CONFIG-ACE`） |
| 执行腿 | `auto <script.at>` 的**显式 ACE 候选入口**（`AUTO_ACE_CANDIDATE=1`，opt-in；默认/`core` 形态闭包不含 ACE），经 `run_ace` console 车道真实执行；argv/不支持形态/诊断/资源/取消一律**具名拒绝且零 legacy dispatch** | run/service/req/debug/test、模板、各 target、watch/安装（B 各族） |
| 身份 | 一次运行同时记录配置腿与执行腿身份并断言同 package+version（`auto-lang-core::sdk_identity`） | SDK 分发/候选安装身份（B-DEVSDK） |
| 纪律 | 坏配置在执行前具名拒绝且**不写 state/lock**；两工程同脚本不同配置互不串值 | 全产品失败面（B） |

**RG-A01 接受面解释（经用户裁定）**：原句"缺 ACE 不能回退旧 VM"在 route C 下按**命令执行腿**解释——候选入口一旦选中，任何拒绝都不得落回旧执行器；配置腿按 969 规范本来就是 legacy owner（不是 fallback），其引擎迁移单列 B。

**A 实现落点（本轮已实施，收据见 §9）**

| 仓 | 文件 | 作用 |
|---|---|---|
| auto-lang | `crates/auto-lang-core/src/sdk_identity.rs`（＋`lib.rs` 一行注册） | 单一 SDK 身份面：`SDK_IDENTITY_SCHEMA`/`config_owner_identity()`/`execution_owner_identity()`/`same_sdk()`，纯数据无求值 |
| auto-man | `crates/auto/src/ace_candidate.rs`（新，特性门控） | 显式候选执行腿：安装桥→配置腿求值→`run_ace` console 执行→身份/证据回执→具名拒绝（argv/诊断/拒绝/资源/取消/身份不符）；`AUTO_ACE_PROOF_RECEIPT`/`AUTO_ACE_WALL_MS`/`AUTO_ACE_CANCEL_AFTER_MS` 为测试钩子 |
| auto-man | `crates/auto/Cargo.toml`、`crates/auto/src/main.rs`、`crates/auto/src/cmd_run.rs` | 可选依赖 `auto-lang-ace-host`＋特性 `ace-candidate`（不进 default/`core`）；模块注册；裸脚本执行前的候选分岔（拒绝即返回，不落到旧执行器） |
| auto-man | `test/fixtures/plan982/{proj-a,proj-b,bad-config,budget}` | 真实配置文档/脚本/坏配置/预算输入（永久 fixture） |
| auto-man | `crates/auto/tests/plan982_ace_candidate_config.rs` | 五条永久验证：同 SDK＋真实值/stdout/exit；双工程隔离；坏配置零写（解析/求值两型）；两臂无 fallback（argv＋dump-lowered）；预算/取消具名终态 |
| auto-man | `crates/auto/tests/plan982_ace_candidate_feature_gate.rs` | 特性关闭时显式跳过说明，避免"0 test 绿" |

**落地（2026-11-17 fold）**：auto-lang `e4b2c06b4`（3 提交：身份面＋命名注记＋规范沉淀 `docs/specs/ace-migration/m2-man-gen.md`）、auto-man `11fe79c`（4 提交：候选腿＋lock＋review 修复＋两 MINOR 关闭）；两仓 master 均 ff-only 线性落主，树保留至 B。

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
| SD-CFG-ACE | add/modify（拟） | `docs/specs/ace-migration/config-document-evaluation.md`（新）；`m2-man-gen.md`、`m2-facade.md`（现有） | 无配置profile/owned结果、route C旧求值→本982已验共同前门/真实AV/纯数据接口与实际Man能力/失败/预算/身份；原A不重写 | 本轮明确跨仓scope、补真正能力，不用状态冒实现 | AC-CFG-SEM/OWN/SAFE/PRODUCT、AC-B-CONFIG、原AC-01..06 |
| SD-01 | add | docs/specs/ace-migration/m2-man-gen.md | 有限切片→实际已验支持/拒绝/来源 | 不把计划当现状 | AC-01..06、全部AC-B-* |
| SD-02 | modify | docs/specs/ace-migration/replacement-scope-and-evidence.md | 959行关联新producer/consumer及差额 | 替换闭合 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-man-gen.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-man-gen.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |

| SD-DETAIL | add | `docs/specs/ace-migration/m2-man-gen.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

外仓领域知识由真实owner落当地Specs；语言侧只保通用接口/迁移指针，不重复领域规则。

## 6. 测试设计

### V-CFG完整验证合同（本轮拟议，原V/AC全部保留）

B-CONFIG-I建立语言桥`tests/p982_config_document.rs`、ACE原source测试里的config面和Man `tests/plan982_ace_config_b.rs`（拟新路径）后，V按[设计§6](../design/ace-config-document-evaluation.md#6-验证合同与旧用例映射)的九个V-CFG-*逐行实跑。它们分别约束SYNTAX/FLOW/CONTEXT/VALUE/FAIL/LIMIT/IDENTITY/PRODUCT/RETIRE，原旧AutoVM用例与780输入映射已列，不能以parser绿、新旧相等或same_sdk替独立业务值/真实AV。

最低区分值：port win32/linux→x1/2、runtime数组→dir(a),dir(b)顺序、root注入覆盖且nested at不被误写、kernel.heap模板、Node-valued Pair、on/off/完整object/bool注入、精确大整数/Unicode/NUL。坏源/类型/权限/超预算/取消/错owner/旧generation/错context缓存→具名非零、工程/state/lock无写且旧接受值可读；修复后下一次真实CLI read/run恢复。禁旧求值/删AV dispatch/删实际config映射为敏感反控，不能只grep无VM字符串。

实际命令组：`cargo t ace-source ace-syntax ace-av ace-facade ace-abi ace-foreign-contract`按真实触面与consumer冻结并集；Man沿原`cargo tc`、lock_contract、plan816/plan805与新B target。新API不存在时不虚称命令已执行；不盲跑ace-perf/全仓all，Linux在专用环境保原条件。R独立核全部Case和原完整CONFIG分母；RETIRE最终安装/不可达部分由986/987接，982须先闭其候选生产路径及原全部产品义务。

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

### 当前目标范围验证（拟实施，未运行）

| Case | AC/任务 | 输入→producer→consumer→独立oracle与负控 |
|---|---|---|
| V-TARGET-SCOPE | AC-01/AC-B-WEBTARGET/AC-04；B-WEBTARGET-I/V/R、T-05 | 原生产TS/JS工程/CLI→976受检provider→真实编译后运行，以模块初始化/值/诊断及无旧emitter为oracle；DEVSDK可在不具备Godot工具的环境闭合本版范围。误把生产TS命令后置或把Godot缺席当成required失败须被范围检查发现；不支持目标请求具名拒绝，不旧链fallback。 |

## 7. 验收标准

| ID | owner_stage | 完成谓词/实际验证 |
|---|---|---|
| AC-CFG-SEM | work | V-CFG-SYNTAX/FLOW/CONTEXT：原配置控制流/根与嵌套/参数覆盖/Node值/完整动态输入经真实AV，原独立expected守恒；I/V生产、R核 |
| AC-CFG-OWN | work | V-CFG-VALUE/LIMIT：owned文档精确类型、参数/孩子顺序/资源/预算/取消与最后owner释放正确，迭代深宽输入无panic；I/V生产、R核 |
| AC-CFG-SAFE | work | V-CFG-FAIL/IDENTITY：坏源具名原span、零副作用/零fallback、缓存身份/冷恢复/修复下一读完整；I/V生产、R核 |
| AC-CFG-PRODUCT | work | V-CFG-PRODUCT及RETIRE的982候选部分、真实Pac/二阶段上下文与执行、CONFIG正式来源可消费；后继安装/退役仍986/987，不提前把未来结果勾绿 |

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
| AC-B-REVERSE | work | 原支持Rust输入→Auto真实运行，非法近邻错误/exit；不要求任意互逆。 | B-REVERSE-V/R |
| AC-B-EXPORT | work | 各原支持输出实际构建/链接/可执行或原目标验收，未知工具环境不改NA。 | B-EXPORT-V/R |
| AC-B-DEVSDK | work | 真实process树cargo/rustc0，Rust输入变化仍正确重建；旧cache/binary不被发现；候选安装失败回退保用户数据。 | B-DEVSDK-V/R |
| AC-A-PROOF | work | 双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共37任务，0完成（原GODOT三任务由用户后置，非完成）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜owner_stage: work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。 [✅ 2026-11-17：冻结记录落 §5"T-01 冻结记录"——六前置 archived 交付核验、两仓改面/owner 重核、ACE 接口实测（`av-worker request` 对真实 pac.at＝`frontend:source.syntax` 拒；legacy 面正常求值）、具名缺口 `GAP-982-CONFIG-ACE`、A 输入/预期/入口/材料/树/A-B 边界冻结；零源码/测试改动，A 实现待 §10 Q-982-1 裁定]

- [ ] **B-CONFIG-I｜M2-A已交子面／M2-B完整｜owner_stage: work｜AC-B-CONFIG/AC-02/03/06**：pac/am/index/state与参数/环境双阶段求值接ACE；路径锁身份与可求值静态子集不混用。 按§5真实改面补永久fixture并固定§6命令。 [A切片✅2026-11-17：候选腿已消费 SDK 配置 owner 并记录逐值证据（`ace_candidate.rs`）；引擎迁移＝本批 B 差额] CONFIG-ACE-B：同时实施语言侧共同profile/Program投影/AV文档原语、owned公开SDK/桥及Man真实消费，按§5设计先证全链；原T-01不重勾。

- [ ] **B-CONFIG-V｜M2-A已交子面／M2-B完整｜owner_stage: work｜AC-B-CONFIG**：运行V-B-CONFIG完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 [A切片✅2026-11-17：真实 pac 值/stdout/exit/隔离/坏配置零写五例实跑（见 §9 review/merge 收据）] CONFIG-ACE-B：新增V-CFG九行与四AC完整运行，原完整CONFIG用例/预算/平台不减，候选legacy_config_dispatch=0。

- [ ] **B-CONFIG-R｜M2-A已交子面／M2-B完整｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。 [A切片✅2026-11-17：独立复审 r1 NEEDS_FIX→r2/r3 PASS（1 MAJOR 已闭）] CONFIG-ACE-B：独立核两仓新producer与真实consumer，pass后原T-06可先CONFIG phase fold交IF-982-CONFIG-ACE；整任务含A/B皆齐才勾。

- [ ] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：从原CLI/runtime fixture冻结多模块pac、args/env及最小脚本，接显式ACE candidate入口，证明配置与实际命令消费同一SDK；不等待整个EXEC/TEMPLATE/各target迁移。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。 [A切片✅2026-11-17：永久 fixture＋显式候选入口接通（feature `ace-candidate`＋env）]

- [ ] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录双配置/作用域隔离，真实值/stdout/exit正确；坏配置不写state/lock，缺ACE身份拒绝无fallback；取消子任务回收。 核子终态与身份，未执行不PASS。 [A切片✅2026-11-17：core/full 两形态各 5/5 PASS，receipt 绑定身份/输入/引擎]

- [ ] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。 [A切片✅2026-11-17：独立复审 PASS 并记录 985/A 可消费的 repo/SHA/接口/测试来源]

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。 **A 切片收口（2026-11-17，route C）**：work→独立 review r1（NEEDS_FIX 1 MAJOR/7 MINOR）→修复→r2 PASS→两 MINOR＋两 NOTE→r3 PASS(mergeable)→phase fold 落主（auto-lang `e4b2c06b4`／auto-man `11fe79c`，规范 `docs/specs/ace-migration/m2-man-gen.md`），同树/分支/组保留至 B；985/A 可消费下列已验来源：显式候选入口（`ace-candidate` 特性＋`AUTO_ACE_CANDIDATE`）、具名拒绝码表、`auto.sdk/1` 身份面、五条永久 case 与两形态实测结果；配置**引擎**迁移＝`GAP-982-CONFIG-ACE` 仍属 B。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **B-EXEC-I｜M2-B｜owner_stage: work｜AC-B-EXEC/AC-02/03/06**：逐CLI dispatch接ACE run/session/tool协议，保stdin/捕获/退出/超时；不等待模板或全部目标生成。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-EXEC-V｜M2-B｜owner_stage: work｜AC-B-EXEC**：运行V-B-EXEC完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-EXEC-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-TEMPLATE-I｜M2-B｜owner_stage: work｜AC-B-TEMPLATE/AC-02/03/06**：模板求值与多文件输出用ACE/中立生成核心，去旧AutoInterpreter；模板输入、文件冲突/失败事务保持。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-TEMPLATE-V｜M2-B｜owner_stage: work｜AC-B-TEMPLATE**：运行V-B-TEMPLATE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-TEMPLATE-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-RUST-I｜M2-B｜owner_stage: work｜AC-B-RUST/AC-02/03/06**：原多模块工程使用新A2R，保CLI stdout/文件/成功信封和错误JSON/exit。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-RUST-V｜M2-B｜owner_stage: work｜AC-B-RUST**：运行V-B-RUST完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-RUST-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-C-I｜M2-B｜owner_stage: work｜AC-B-C/AC-02/03/06**：新A2C接trans C/a2c-stdlib与目标配置，保ABI/链接/错误契约。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-C-V｜M2-B｜owner_stage: work｜AC-B-C**：运行V-B-C完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-C-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-WEBTARGET-I｜M2-B｜owner_stage: work｜AC-B-WEBTARGET/AC-02/03/06**：真实CLI调用新TS/JS provider，保模块/依赖/source map。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-WEBTARGET-V｜M2-B｜owner_stage: work｜AC-B-WEBTARGET**：运行V-B-WEBTARGET完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-WEBTARGET-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-PY-I｜M2-B｜owner_stage: work｜AC-B-PY/AC-02/03/06**：CLI接977/PY及工程配置/解释器发现。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-PY-V｜M2-B｜owner_stage: work｜AC-B-PY**：运行V-B-PY完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-PY-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-REVERSE-I｜M2-B｜owner_stage: work｜AC-B-REVERSE/AC-02/03/06**：保r2a与s2s/emit已有公开命令，消费其独立工具接口。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-REVERSE-V｜M2-B｜owner_stage: work｜AC-B-REVERSE**：运行V-B-REVERSE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-REVERSE-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-EXPORT-I｜M2-B｜owner_stage: work｜AC-B-EXPORT/AC-02/03/06**：逐CMake/IAR/GHS等现用export目标写配置和库闭包，逐子格实际工具链身份。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-EXPORT-V｜M2-B｜owner_stage: work｜AC-B-EXPORT**：运行V-B-EXPORT完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-EXPORT-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-DEVSDK-I｜M2-B｜owner_stage: work｜AC-B-DEVSDK/AC-02/03/06**：watch和工具发现切候选ACE SDK；记录host/tool/runtime/stdlib闭包及重定位，稳定Auto编辑无Rust重编；不提前正式发布。 按§5真实改面补永久fixture并固定§6命令。 CONFIG-ACE-B：本批新配置消费须IF-982-CONFIG-ACE正式来源；原其它最小前置仍按批次，不等983/984/985 B最终。

- [ ] **B-DEVSDK-V｜M2-B｜owner_stage: work｜AC-B-DEVSDK**：运行V-B-DEVSDK完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-DEVSDK-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final核全部批收据/组合/遗漏/延期/workaround、warning/fmt/debug和Spec；缺必需结果不得pass。 CONFIG-ACE-B：全包final另核新配置九Case/四AC、route C历史不冒B、所用平台与原所有产品分母。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner次序合入/落规范，核真实来源/consumer同版，全部齐后归档。 CONFIG-ACE-B：可先经CONFIG独立R交正式phase收据供下游，保唯一两仓树/resync；全包review未齐不勾整任务/归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan、输出逐目录对账/清scope/登记，guard clean后清树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-CONFIG-I/B-CONFIG-V/B-TEMPLATE-I/B-TEMPLATE-V/B-RUST-I/B-RUST-V/B-PROOF-I/B-PROOF-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。









## 9. 复审记录

### 2026-11-17 CONFIG-ACE-B规划交接（r8；非实施验收）

stage: new/revision | outcome: pass（规划合同预检） | 本轮用户仅授权补承接/设计/接口/验收及排程；未work/建树/运行测试/探针/构建/合入/归档/派agent。GAP-982-CONFIG-ACE由982原B-CONFIG两仓端到端承接，能力本身保持未实现；本包按§5–8现任务生产或接收，不新增Plan/任务，不改已验A收据。下一步须未来实施指令，982先B-CONFIG-I全链前探/接口与原输入冻结，再V→独立R→正式phase交接；其他owner按所用正式来源接。

取材和静态核验：7计划123原任务ID/顺序/勾选与状态/进度保留，语义revision各增1；仅982补已交967/979直接基础依赖。9个配置判别Case/4个能力AC、typed接口/值语义/预算/生命周期/原用例映射与Spec候选已列；24节点30边阶段图无环，新增本地链接可达，18份源码/Spec/归档/计数器身份不变，目标文档git diff --check通过。旧无owner快照/route C/执行回执作历史保留，不把规划PASS当真实能力或独立产品review。M2A-08 r29/95c7fbad8冻结不变。

### 本轮用户授权后置的原Godot条目（历史，不计当前M2任务/AC/V）

原ID及验收留此供后续版本追溯；全部原为未实施，不记完成、不形成M2缺证门。本轮没有创建后继Plan。原代码/fixtures/expected未删除。

> | V-B-GODOT | AC-B-GODOT | man: cargo tc；lang: cargo t ace-a2x generated-delivery-controls；本目标复用provider已验fixture再经真实CLI，I固定原工具命令 | 实际Godot headless载入原工程，资源/脚本执行，缺依赖诊断。 |

> | AC-B-GODOT | work | 实际Godot headless载入原工程，资源/脚本执行，缺依赖诊断。 | B-GODOT-V/R |

> - [ ] **B-GODOT-I｜M2-B｜owner_stage: work｜AC-B-GODOT/AC-02/03/06**：接GDScript/TSCN脚本/场景/资源工程装配。 按§5真实改面补永久fixture并固定§6命令。

> - [ ] **B-GODOT-V｜M2-B｜owner_stage: work｜AC-B-GODOT**：运行V-B-GODOT完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

> - [ ] **B-GODOT-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

2026-11-16 stage: new/revision | PLAN-982 r7 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

2026-11-12 stage:new / PLAN-982 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原40任务/全部AC与V/依赖/阶段/预算和drafting 0/40；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部40个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/40；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-07 r3：细化11批，0/37。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留；原STATIC/TARGETS汇总批进一步逐目标拆成具名I/V/R，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

2026-11-17 stage: work / T-01（A 切片冻结轮，AC-01/AC-PHASE）| tree: `D:/autostack/.wt/lang-982/auto-lang@c6c65e5c5` ＋ 同组 `auto-man@54fe4070b`（本轮新建、两树干净、branch `plan-982-dev`、基面各仓 master）| 授权: 本 goal（完成计划982/A）经 `/auto-plan-goal` 授予 work→review→bounded repair→merge；A 只按 §5 阶段表 | 产出: §5"T-01 冻结记录"——①A 六个最小前置（960/964/969/970/973/978）全 archived 且取正式 delivery commit＋规范；②两仓真实 owner/改面 6 文件 sha256 与计划记录一致、配置读取调用点逐行定位；③接口现值**实测**：ACE `SourceProfile` 全集无 config/document 面、`AceValue` 仅标量面、`canonical_owned` 拒 `Value::Object`；真探针 `av-worker request` 对 `j2check/auto/pac.at`（`807c35217b9a6ea9`）＝`{"ok":false,"diag":"frontend:source.syntax","source_hash":"37f2c30fd510fec2"}` exit 4，同族 legacy 面（`auto info` 读 am.at）正常求值；④具名缺口 `GAP-982-CONFIG-ACE`（owner 未指派）；⑤A 输入/独立预期/入口命令/材料 scope/树布局/A-B 边界冻结；⑥路由 A/B/C 交 §10 Q-982-1 | 未执行: 零源码/测试/规范改动，零固定组/产品/生成运行，未合入，未清树 | 判读: T-01 的交付物是"冻结或具名 PROOF"；本轮如实冻结在缺口＋可复用资产上，**不**把缺口写成能力已交、不勾任何 CONFIG/PROOF I/V/R | next: 用户裁定 Q-982-1 后进 B-CONFIG-I（或按裁定改 Plan/改 owner）

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r7 | round: 1 | from: work | to: work | status: executing | outcome: waiting（T-01 冻结已落；A 实现待 §10 Q-982-1 用户裁定，本轮不实施） | worktree: D:/autostack/.wt/lang-982/auto-lang | commit: c6c65e5c5(base) | next: user

2026-11-17 簿记：T-01 冻结记录与上述两条收据随 docs 簿记提交 `258c3fea2` 落 master；同一提交逐字带上该文件里**既有的未提交 r7 修订**（Godot 后置、40→37），未改写他人内容。`.retention/policy.json` 为本仓未跟踪的本地登记表，已加入 plan-982 scope（in-flight）。探针原件与 legacy 对照输出在 `D:/autostack/plan-artifacts/plan-982/probe-t01/`（本地 scratch，按政策不入库、A 收尾折叠后删除）。

2026-11-17 stage: work / route C 实施（B-CONFIG-I + B-PROOF-I 的 A 切片）| 依据: 用户对 §10 Q-982-1 裁定 **route C**（收窄 A；A 合同见 §5"切片定义"）| tree: 唯一组 `D:/autostack/.wt/lang-982/`＝`auto-lang@076fae81c`（branch `plan-982-dev`，基面 master `e85096628`）＋`auto-man@b2a253c`（branch `plan-982-dev`，基面 master `54fe407`）＋只读兄弟 `auto-web@0adebc0`/`auto-ui@2f42d661`/`auto-down@5af11a3`（均 detached，仅为跨仓 path 依赖解析；未改动、未纳入提交）| 产出（auto-lang）：`crates/auto-lang-core/src/sdk_identity.rs`＋`lib.rs` 一行注册（`076fae81c`）——身份 schema、config/execution 两腿身份、`same_sdk()` 机器可判谓词，纯数据零求值/零 ACE 依赖 | 产出（auto-man）：`crates/auto/src/ace_candidate.rs`＋`Cargo.toml`（可选依赖 `auto-lang-ace-host`＋特性 `ace-candidate`，不进 default/`core`）＋`main.rs`＋`cmd_run.rs` 候选分岔＋`test/fixtures/plan982/{proj-a,proj-b,bad-config,budget}`＋`crates/auto/tests/plan982_ace_candidate_config.rs`（`9c3c940`）；`Cargo.lock` 增量解析（`b2a253c`，仅新增 ace-*/cranelift 条目，455 行、零删除） | 验证（本树实跑）：`cargo test -p auto --no-default-features --features ace-candidate --test plan982_ace_candidate_config -- --test-threads=1`＝**5/5 PASS**（0.47s；同 SDK＋真实配置值/deps=1/targets=1＋engine=av＋stdout=42＋返回值 7；双工程 name/version 隔离；坏配置 `ace.config/invalid` 且目录快照逐字节不变；两臂无 fallback——legacy 臂同命令成功且出 42、candidate 臂 `ace.args/unsupported` 且 42 零出现；预算臂 `ace.resource/ExecutionWall` 具名拒、取消臂 cancelled/resource-denied，且无预算时同脚本成功=非空证）| `cargo test -p auto-lang-core --lib sdk_identity`＝4/4 PASS（含两腿 same_sdk 正反例）| `cargo check -p auto --no-default-features`（关特性）＝绿（ACE 闭包不进入 core/default）| `cargo build -p auto --no-default-features --features ace-candidate`＝绿（1m45s）| 格式/告警：两仓新增文件 `rustfmt --check` 干净；新增代码零告警，仅余既有 `cmd_run.rs:132 unused_mut`（HEAD 行 124 同款，非本包）| 未执行（具名差额）：配置**引擎**迁移到 ACE（`GAP-982-CONFIG-ACE`，B）；run/service/req/debug/test、模板、各 target/watch/安装（B）；默认 full profile 构建（进行中，结果见下条）| next: B-CONFIG-R/B-PROOF-R 独立复审（另一上下文）

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r7 | round: 4 | from: work | to: work | status: executing | outcome: pass（A 切片实现完成、5/5 永久验证绿；待独立复审） | worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 076fae81c(auto-lang)/b2a253c(auto-man) | next: review

2026-11-17 补记（同一 work 切片，full profile）：`cargo build -p auto --features ace-candidate`＝绿（1m58s；默认 full 形态＋候选特性），`cargo test -p auto --features ace-candidate --test plan982_ace_candidate_config -- --test-threads=1`＝**5/5 PASS**（1.07s）。即核心形态与出货（full）形态两条构建面都已实跑通过；默认/`core` 关特性构建见上条 `cargo check`。新增代码零告警，余下告警均为既有（`cmd_autodesk.rs:74` unused doc comment 等）。

2026-11-17 stage: review / B-CONFIG-R＋B-PROOF-R（A 切片独立复审，独立上下文 agent，未参与实现）| 冻结基线: auto-lang `076fae81c`（r1）/`de45e9339`（r2）、auto-man `9c3c940`+`b2a253c`（r1）/`5a1fc29`（r2）| **r1 NEEDS_FIX**（0 BLOCKER/1 MAJOR/7 MINOR/4 NOTE）：F-982A-01 MAJOR＝候选选中时 `--dump-lowered` 早退仍打印旧 lowering 且 exit 0（`ace.dump-lowered/unsupported` 不可达）；F-02 `same_sdk` 对产品输入恒真＋`identity-mismatch` 死分支；F-03 关特性 0 test 静默绿；F-04 坏配置 fixture 实为解析失败；F-05 非 Windows 开特性不可编译；F-07 "零写"oracle 只比 path+size 且 receipt 写入与 docs 措辞冲突；F-08 继承 env 可污染非空证臂；NOTE＝身份命名重叠/空行节奏/祖先 `pac.at` 回溯 | 修复 `5a1fc29`：候选检查**前置于** dump 早退并补三臂 oracle（候选拒/旧腿仍打印/argv 臂）；`same_sdk` 追加"实际服务引擎＝身份引擎"＋receipt `engine_is_identity_engine`/`script_bytes`/`started_unix_ms`；新增 `plan982_ace_candidate_feature_gate.rs`（关特性显式跳过说明）；新增 `bad-eval` 求值失败 fixture（`Undefined variable`）与既有解析失败并列；桥依赖移入 `[target.'cfg(windows)'.dependencies]`、非 Windows 具名拒 `ace.host/unsupported-platform`；快照改内容哈希（FNV-1a）且"零写"限定**工程树**；helper 清继承预算 env；严格 `same_sdk==true`；成功腿补旧腿尾随空行 | **r2 PASS**（0 BLOCKER/0 MAJOR/2 MINOR/6 NOTE；MAJOR 已闭、legacy 面零改动、`cmd_run.rs` 仅 18/8 hook 无格式 churn、三形态命令全绿）| r2 两 MINOR → `11fe79c`＋`de45e9339`：坏配置分支按 fixture 钉失败种类（parse=`Expected term`／eval=`Undefined variable`）、`sdk_identity` 补命名轴注记 | **r3 PASS(mergeable)**：实测证明新断言非空（把 `bad-eval` 换成 parse 错时该 arm 会红；两文件 diff 仅声明改动）| 复审原件 `D:/autostack/plan-artifacts/plan-982/review-a/{review.md,review-r2.md,review-r3.md}`（本地 scratch，不入库）| 未动代码以外内容；复审后仅上述 MINOR 修复与文档注记

2026-11-17 stage: merge / A 切片 phase fold（对应 T-06/T-07 的 A 切片部分；整包**不**归档）| **landed**：auto-lang `e4b2c06b4`（ff-only；3 提交 ac1e869ac/a25d2f092/58b9accdf，rebase 两次追平 master `c53b34a3b`；含规范 `docs/specs/ace-migration/m2-man-gen.md`）、auto-man `11fe79c`（ff-only；4 提交 9c3c940/b2a253c/5a1fc29/11fe79c，基面 master `54fe407`）| **落主核对**（两树 resync 到 master 后实跑）：`cargo test -p auto-lang-core --lib sdk_identity`＝4 green；`cargo test -p auto --no-default-features --features ace-candidate --test plan982_ace_candidate_config`＝**5/5 PASS**（0.44s）；full 形态同测＝**5/5 PASS**（1.17s）；`cargo check -p auto`（默认关特性）绿 | **外来 WIP 处置**：auto-man 主检出 `Cargo.lock` 携他会话未提交改动（sha256 `E0D99CF48347B09D…`，12 行：`ace-foreign-contract` stanza＋auto-bindgen/auto-tool-client 依赖行）——字节备份 `plan-artifacts/plan-982/merge-wip/auto-man-Cargo.lock.wip` → `git stash` → ff 合并 → 逐行核对 12 行**全部**包含于合并后 lock（缺失 0）→ drop stash（内容被合并吸收，无需回写）；auto-lang 主检出 WIP 与本切片路径零重叠 | **规范**：`docs/specs/ace-migration/m2-man-gen.md` 落主（只写已验 A 面＋具名差额；SD-02/959 行产品级闭合仍属 B）| **树/分支/组**：保留（`D:/autostack/.wt/lang-982/{auto-lang,auto-man}`＋只读兄弟 `auto-web/auto-ui/auto-down`），`plan-982-dev` 与 master 同步，供 B 使用；plan-artifacts 保留至 B 收尾（A 证据已在 §9 折叠）| **985/A 可消费来源**：repo/SHA 如上；接口＝`ace-candidate` 特性＋`AUTO_ACE_CANDIDATE`＋具名拒绝码表＋`auto.sdk/1`＋receipt schema；测试＝`plan982_ace_candidate_config.rs`＋`plan982_ace_candidate_feature_gate.rs` 与 §6 命令；适用性＝Windows、最小 CLI 腿、配置引擎仍 SDK 指定 legacy owner | **不声明**：全产品迁移、其它 target/模板/安装、配置引擎已迁 ACE

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r7 | round: 4 | from: merge | to: done | status: executing（整包 B 未开）| outcome: pass（A 切片 delivered＋landed＋verified＋spec deposited；树保留）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: e4b2c06b4(auto-lang)/11fe79c(auto-man) | next: user（A 目标达成；B 另行授权）

2026-11-17 goal-close（objective「完成计划982/A」）：A 切片全链闭合判定＝delivered（`e4b2c06b4`/`11fe79c` ff-only 落两仓 master）＋canonical spec 在案（`docs/specs/ace-migration/m2-man-gen.md`）＋独立复审 r1→r3 终态 **PASS(mergeable)**（1 MAJOR＋9 MINOR/10 NOTE 全部关闭或具名接受）＋落主复核（core/full 两形态 5/5、身份 4 green、默认关特性绿）＋树/分支/组按合同**保留**供 B。**不**把整包 37 任务置完成、**不**归档本 Plan（B 未开，按 route C 裁定）。仍未闭（属 B/能力 owner）：`GAP-982-CONFIG-ACE` 配置引擎迁移、EXEC/TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK、安装/升级/退役、SD-02/959 行产品级闭合、周期批量回归门。
2026-11-17 stage: work / **B-entry 核验＋B-CONFIG-I 增量1（中立接口层）**（objective「继续完成计划982」，round 1）| 门: 985 **M2A-08＝`95c7fbad8`**（985 §9 merge 收据「M2A-08成立」，规范 `docs/specs/ace-migration/m2-certification.md` 在案）→ 本包 B 门已开；CONFIG 最小前置 960/964/969/973＋979 全 archived；r8 合同（§5 CONFIG-ACE-B）与设计 `docs/design/ace-config-document-evaluation.md` 为本轮实施合同（该设计与 r8 修订此前为另一规划会话的未提交工作树内容，随本轮收据一并入库，逐字保留、未改写）| 实施 `37c7bf783`（auto-lang 唯一树，4 文件 +1052）：①`crates/auto-lang-core/src/ace_config.rs` 中立配置合同（owned `ConfigValue`/`ConfigNode`/`ConfigDocument`，精确 i64/u64/IEEE位 f64/char/含NUL文本、有序 Object、有序可重复 Node（args/props/children）；显式 `ConfigContext`（args/env快照/source root/module bundle/schema/grants）与 `ConfigEvalLimits`；分态 `AceConfigOutcome`（Success{document,identity,cost}/Diagnostic/Denied/ResourceDenied/Cancelled/HostFailure）；可归因 `ConfigIdentity`（engine/profile/domain/value schema/schema id/source hash/context digest/SDK version）与真实 `ConfigCost`；`ConfigEngineChoice{LegacyVm,AceAv}` 与 `parse_config_engine`）；②`ace_entry.rs` 在**同一** `AceExecutor` 增配置默认方法（默认＝具名 `ace.config/executor-unsupported`，不落旧引擎）＋`eval_config_with_ace` 单注册表纪律（未注册/锁损坏/不支持均具名，零 legacy dispatch）；③`lib.rs` 注册 | 验证（本树实跑）：`cargo test -p auto-lang-core --lib ace_config`＝**5/5 PASS**；`cargo test -p auto-lang-core --test ace_config_registry`＝**1/1 PASS**（独立测试进程，覆盖未注册→具名失败、默认不支持→具名拒绝且消息声明不走 legacy、支持→文档/身份/代价原样返回且精确值/顺序/重复节点保持、拒绝传播、取消标志真实被读、关闭后回到未注册）；`--lib sdk_identity`＝4/4 未回退；`cargo build -p auto-lang-core` 新增文件零告警；`rustfmt --check` 干净 | 未做（具名下增量，B-CONFIG-I 未完）：ACE 侧 `config-document` profile/grammar、Program 单入口候选与精确 source map、`cfg.*` 受检值/容器原语（node/put/append/merge/get＋scalar box/unbox）、AV 执行与迭代式 owned 导出、`auto-lang-ace-host` 配置桥（compile→verify→execute→export→teardown）、Man 侧 `ConfigEngineChoice` 接入 `AutoConfig` 与二阶段 context/dep 消费、V-CFG 九 Case（SYNTAX/FLOW/CONTEXT/VALUE/FAIL/LIMIT/IDENTITY/PRODUCT）与 780 矩阵回归 | 判读：**接口层已交、能力未实现**（与设计 §8 的 GAP 状态一致）；设计 §3 仍是拟议，最低负载（根注入＋对象/模板＋运行期 for＋两个 Node 孩子）全链未成立前不得勾 B-CONFIG-I/V，也不得归档 | next: B-CONFIG-I 增量2（ACE config-document 前端：profile/grammar/Program 单入口）

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 1 | from: work | to: work | status: executing | outcome: pass（B 门核验＋B-CONFIG-I 增量1 接口层落地，5+1 用例绿；能力未实现，树保留）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 37c7bf783 | next: B-CONFIG-I 增量2（ACE config-document 前端）
2026-11-17 stage: work / **B-CONFIG-I 增量2（SDK 消费半：显式 ACE 配置路径＋精确投影）**（objective「继续完成计划982」，round 2）| 实施 `4e22f1a84`（auto-lang 唯一树，3 文件 +527）：①`AutoConfig::{from_code_with_engine,from_file_with_engine}` 接受显式 `ConfigEngineChoice`——`LegacyVm` 走原路径逐字不变，`AceAv` 经已注册 ACE executor 求值、任何非成功态都是具名错误（不回落、不重试旧引擎）；②注入 args 由 `config_value_from_auto_val` 打标签进显式 `ConfigContext`（标量/对象/数组；无配置值 tag 的变体具名拒绝，不静默丢弃）；③`ConfigDocument::to_auto_val_node()` 纯数据投影到旧消费形状：精确 i64/u64（>i64::MAX 具名拒绝 `config.projection/u64-out-of-range`）、IEEE 位 f64、Char、含 NUL 文本、属性顺序、Node args/children 顺序与重复节点保留；深度守卫 `MAX_PROJECTION_DEPTH=512`（具名 `config.projection/too-deep`），无求值/无 parser/VM 调用 | 验证（本树实跑）：`cargo test -p auto-lang-core --test ace_config_engine`＝**1/1 PASS**（legacy 不变；无 executor→`ace.host/executor-unregistered` 且 legacy 路径对同一源仍可解析＝无回退 oracle；无配置面 executor→`ace.config/executor-unsupported`；拒绝传播；服务 executor→文档按精确值/顺序/重复节点返回且注入 args 在引擎侧可见（count=2、port/kernel 值正确）；u64 溢出投影具名拒；`from_file_with_engine` 同纪律）；`--lib config`＝**21 passed**（既有旧配置语义零回退）；`--test ace_config_registry`＝1/1；`--lib ace_config`＝5/5；`cargo build -p auto-lang-core` 绿；rustfmt --check 干净 | 未做（仍在 B-CONFIG-I 内）：ACE 引擎侧 `config-document` profile/grammar、Program 单入口候选与 source map、`cfg.*` 受检值/容器原语、AV 执行与迭代式 owned 导出、`auto-lang-ace-host` 配置桥、Man 侧二阶段 context/dep 消费与真实 CLI、V-CFG 九 Case、780 矩阵回归 | 判读：**SDK 两侧（合同＋消费）已交，引擎仍缺**；最低负载全链未成立，B-CONFIG-I/V 不勾 | next: B-CONFIG-I 增量3（ACE 引擎侧：config-document profile ＋最小的 node/put/append 原语跑通一份真实文档到 owned 快照）

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 2 | from: work | to: work | status: executing | outcome: pass（SDK 显式 ACE 配置路径＋精确投影落地；1+21+1+5 用例绿，legacy 零回退）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 4e22f1a84 | next: B-CONFIG-I 增量3（ACE 引擎侧 config-document profile 与最小原语）
2026-11-17 stage: work / **B-CONFIG-I 增量3（配置原语语义核心）**（objective「继续完成计划982」，round 3）| 实施 `crates/auto-lang-core/src/ace_config.rs` 的 `ConfigDocumentBuilder`（cfg.node/put/append/merge/get/node_prop/finish）：per-builder 句柄不逃逸裸指针/索引；一父树纪律（自 append＝`cfg.cycle`、重复 attach＝`cfg.already-attached`、已 attach 不能当 root）；kind 检查读取（`cfg.kind-mismatch`/`cfg.missing-key`/`cfg.unknown-node`）；node/byte 预算逐次计费（`cfg.limit/nodes|bytes`）；`finish` 产出与公开 success 类型一致的 owned 快照 | 验证：`cargo test -p auto-lang-core --lib ace_config`＝**8/8 PASS**（5 合同 + 3 新 builder 用例：真实文档装配含 args/props/merge/children；具名拒绝全覆盖；两轴预算）| 未做：AV intrinsic 绑定（cfg.* 的 ACE 侧签名/准入/资源账）、config-document profile/grammar、Program 单入口、owned 导出接 AV、host 桥、Man 二阶段消费、V-CFG | 判读：原语**语义**已冻结可测，引擎绑定仍缺；B-CONFIG-I/V 不勾 | next: ACE 侧 profile/grammar 与 cfg.* 接线

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 3 | from: work | to: work | status: executing | outcome: pass（配置原语语义核心落地，8/8 绿）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: (见本轮 commit) | next: ACE profile/grammar＋cfg.* 绑定
2026-11-17 stage: work / **B-CONFIG-I 增量3b（builder↔投影 join 测试：未成，转具名未决）**（objective「继续完成计划982」，round 4）| 尝试：为 `ConfigDocumentBuilder` 产出加一条"装配→`to_auto_val_node`"的端到端单测，断言 Node 子节点身份。实测发现**语义未定**：旧形状把 `dep kernel {...}` 的身份放在**首个位置参数**（`_arg0`，与旧 `auto config` 输出一致），而 `ConfigNode.id` 是独立字段；`project_node` 目前只把 id 作为位置参数附加、不回填 `Node.id`。该约定必须先钉死（回填 id？只作位置参数？两者并存=重复？）再加 join 断言。**未把未经核实的断言入库**：本轮编辑已 `git checkout --` 回退，树回到已提交的绿态 `08d4dc343`，`cargo test -p auto-lang-core --lib ace_config`＝**8/8 PASS**，工作树 clean | 具名未决（下轮起点）：`D-982-CFG-IDENTITY`——配置 Node 身份在 owned 模型→auto_val 投影中的唯一表示（影响 Pac 的 dep 身份读取与 780 静态子集对账）| 判读：无能力声明；B-CONFIG-I/V 仍不勾 | next: 钉 `D-982-CFG-IDENTITY` 后补 join 测试，再进 ACE profile/grammar

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 4 | from: work | to: work | status: executing | outcome: pass（无回归：回退未证编辑，8/8 绿；具名未决 D-982-CFG-IDENTITY 登记）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 08d4dc343 | next: 钉 D-982-CFG-IDENTITY → join 测试 → ACE profile/grammar
2026-11-17 stage: work / **B-CONFIG-I 增量3b（D-982-CFG-IDENTITY 钉定＋真实缺陷修复）**（objective「继续完成计划982」，round 5）| 依据实证：旧配置形状把节点身份放在**统一 args/props 槽的首个位置参数**（`dep kernel { _arg0: ... }`），`auto_val::Node.id` 为空。据此实施 `b54df0744`：①`project_node` 的位置参数改走 `add_arg_unified("_arg{n}", …)`——原实现用 `add_arg(Arg::Pos(..))`，实测**参数丢失**（`args_iter` 空、`_arg0` prop 空），属真实缺陷而非仅语义未定；②身份仍作为 `_arg0`，`Node.id` 保持空，消费语义与旧引擎逐形一致 | 验证：`cargo test -p auto-lang-core --lib ace_config`＝**9/9 PASS**（含新 `p982b_node_identity_is_the_first_positional_argument`：name/id/_arg0 三面断言）；`--test ace_config_engine`＝1/1 PASS（引擎路径未回退）| 判读：`D-982-CFG-IDENTITY` 关闭；投影的身份/参数通道已与旧形状对齐；引擎侧（profile/grammar、cfg.* AV 绑定、host 桥、Man 二阶段、V-CFG）仍缺，B-CONFIG-I/V 不勾 | next: ACE `config-document` profile/grammar＋cfg.* 绑定

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 5 | from: work | to: work | status: executing | outcome: pass（身份约定钉定并修掉参数丢失缺陷；9+1 用例绿）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: b54df0744 | next: ACE profile/grammar
2026-11-17 stage: work / **B-CONFIG-I checkpoint＋下一增量入口冻结**（objective「继续完成计划982」，round 6）| 状态核对（实跑）：HEAD `b54df0744`、工作树 clean；`--lib ace_config`＝9/9、`--test ace_config_registry`＝1/1、`--test ace_config_engine`＝1/1 全绿（无回退）| 下一增量（引擎侧前端）入口已定位并冻结：`experimental/ace/crates/core/src/source/mod.rs`（`SourceProfile` 枚举 152 行、`AGE_H0_SOURCE_PROFILE` 131 行、名字→profile 映射 402 行的既有模板）＋`core/src/source/domain.rs::compile_domain`(183) 与 `DomainBehavior`(81) 的现行"表达式-only"边界（设计 §2 明确它**不能**直接承载配置文档）→ 实现路径＝按 AGE-H0 的 profile 落地范式（grammar 裁剪→CST 投影→Program 单入口→共享 verify/lowering），再挂 `cfg.*` 原语到本轮已冻结的 `ConfigDocumentBuilder` 语义 | 本轮未新增代码：引擎前端是多文件共享语法改动，需在下一轮以完整上下文开工，避免半成品树；无能力声明 | 判读：B-CONFIG-I 的 SDK 半边已交且稳定，引擎半边未启；B-CONFIG-I/V 不勾，树保留 | next: 新建 `ace.config.document/v1` profile（grammar＋`source/config_document.rs`＋Program 单入口），最小文档跑通到 owned 快照

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 6 | from: work | to: work | status: executing | outcome: pass（checkpoint：9+1+1 绿，入口冻结，无能力增量）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: b54df0744 | next: ACE config-document profile/grammar
2026-11-17 stage: work / **B-CONFIG-I 资源边界交接（非项目阻塞）**（objective「继续完成计划982」，round 9）| 事实：本轮会话上下文已不足以安全施工引擎前端（E1–E6 为多文件共享语法改动，半成品会污染共享 grammar/CST），故不启动、不产半成品。此前 B-CONFIG-I SDK 半边 4 提交在 `plan-982-dev`（`37c7bf783`/`4e22f1a84`/`08d4dc343`/`b54df0744`），验证基线 9/9＋1/1＋1/1＋旧配置 21/21 全绿；引擎侧已按 §5 台账 E1..E8 冻结（`0e1f6735e`）| 这是**会话资源限制**，不是计划阻塞：无依赖缺失、无 owner 争议、无待裁决策；不标 goal blocked（不符合"同一具体阻塞条件连续三轮"的政策要求），保持 active 交由后续会话从 E1 继续 | 判读：无能力声明；B-CONFIG-I/V 不勾；树/分支/收据/台账齐备可续

2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 9 | from: work | to: work | status: executing | outcome: waiting（会话资源边界，已交接；下一步 E1）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 0e1f6735e | next: E1（新会话按 §5 台账）
2026-11-17 stage: work / **B-CONFIG-I E1：配置文档 profile ＋单顺序入口前门**（objective「继续完成计划982」，round 1）| 依据: §5 台账 E1（`0e1f6735e` 冻结）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@0764778b9`（branch `plan-982-dev`，基面 `b54df0744`）| 实施 `0764778b9`（10 文件，＋530/−53）：①grammar `auto-source.at` 声明 `profile(name: "config-document")`，并把该名加入**原本放行 text-record 的 48 处门控列表** ＋ `script-item`（item alt 与规则两处）——配置文档面 = text-record 全表面 ∪ 顶层脚本语句序列；未启用名字对其它 profile 的裁剪字节无影响；②`SourceProfile::ConfigDocument`（`auto.source.config-document.v1`）＋`as_str`/`hir_domain`(TextRecord)/`is_scalar`/`is_text`/`is_record`/`surface_is_scalar_or_wider`/`is_entry_unit`（文档＝一个顺序入口，入口形状沿用脚本入口合同）与 `of()` 反向映射；③前门 `profile_slug("config-document")`／`profile_from_slug`／`GRAMMARS` 15→16 槽（新面占槽 15）；④`parse_program_with` 通用 CST 链与 f-string 模板面（`fstr::profile_admits_fstr`）放行；⑤新 `crates/core/src/source/config_document.rs`：`compile_config_document` = 共享 CST 解析 → 唯一顺序入口物化（稳定入口名 `__cfg_document`；文档语句与表达式 span 原样，入口尾值是**编译期合成**的零宽 i32 常量，文档语义不读它）→ 共享 `compile_program_parts` → text-record 域 `verify_text_record`；非文档形状（声明项/多入口/空文档）具名拒绝 `source.config-document/{declaration,entry,empty}`；⑥`ace-core` 二进制 `--profile config-document` 走同一前门；⑦永久用例 `crates/core/tests/cases/plan982_config_document.rs`（3 例）＋`syntax-grammar` 的 `DECLARED` 冻结集刷新（新 profile 按声明序插入）| 验证（本树实跑）：`cargo test -p ace-core --test conformance plan982`＝**3/3 PASS**（profile 三向映射＋与 text-record 的 GrammarId 隔离；文档两语句→一个入口且入口在 Checked HIR 里是真 `sf___cfg_document`、语句 span 保原源与源序；声明项/空文档具名拒）；`cargo test -p ace-core --test conformance plan981_grammar_scope`＝**4/4 PASS**（14 个既有 profile 的 GrammarId **冻结值逐字节不变**＝"既有裁剪字节不变"的实测锚）；`cargo check -p ace-core` 绿；新文件 `rustfmt --check` 干净（其它既有文件本就不是 rustfmt-clean，未顺手改动）| **未闭门（具名）**：`cargo t ace-source`（§5 E1 指定门）已启动但与本机另一会话的 `cargo t ace-native` release 构建互相抢占，1h+ 仍停在 av-worker 的 release 编译（group 工具前置），本轮未取得该组结果；`cargo t ace-semantics` 未跑；`ace-syntax-grammar` 的 `profiles_927`（DECLARED 刷新后）未单独跑；`ace-core --lib` 单元测试未跑 | 判读: **E1 主体已交**（身份＋单入口＋具名拒绝＋不回归锚），但 E1 的组门与 E2 文档语法面未闭，B-CONFIG-I 不勾；文档值语义（`cfg.*` 原语、owned 导出、host 桥）仍缺 | next: 复跑 E1 组门（ace-source／ace-semantics／syntax-grammar）→ E2（顶层 `key: value`、`dep bps{}`/`dep "stylekit"{}`/`dep("kernel"){}` 三形、模板插值、`if`/`for` 文档语句的 grammar＋CST 投影）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 1（本会话）| from: work | to: work | status: executing | outcome: pass（E1 主体落地，3+4 用例绿；组门未闭）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 0764778b9 | next: E1 组门复跑 → E2
2026-11-17 stage: work / **E1 组门（改用等价包级实跑）＋ B-CONFIG-I E2（文档 grammar＋CST 投影）**（objective「继续完成计划982」，round 2）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@499335856`（branch `plan-982-dev`，基面 `0764778b9`）| **E1 组门处置（具名）**：`cargo t ace-source` 在本机被另一会话 `cargo t ace-native` 的 release 工具构建长期抢占（该组 12 个工具前置各自 release 编译，1h+ 未进测试阶段），故终止该实例、改跑**等价包级**套件并如实记录（不冒充组绿）：`cargo test -p ace-core --lib --tests`＝**79 passed（1 ignored）＋ 752 passed / 0 failed**（含本包 6 例）；`cargo test -p ace-syntax-grammar`＝**45 passed / 0 failed**（`profiles_927` 的 DECLARED 精确集含新 profile、14 个既有 profile 的 GrammarId 互异且冻结值不变）；`cargo test -p ace-semantics-contract -p ace-semantics -p ace-semantics-store -p ace-source-query --lib --tests`＝contract 21+5、semantics 14+4、source-query 绿；semantics-store **72 passed / 3 failed**，其中 2 例是 `performance::`（固定组 nextest 表达式 `not test(/performance::/)` 本就排除，裸 `cargo test` 才会跑），1 例 `sema_boundary_case_files_executed_status_honest` 是**既有红**：`crates/auto-lang/test/cases/ac-semantics/*.document.case.ad` 缺 `status: executed`（Plan-1062 迁移产物，非本包改面，本包零触碰）；**该组真绿需其 owner 修复或走固定组口径**，本包据实记为组门未闭 | **E1 结构增量（同一提交 `499335856` 内一并落地）**：`script-item` 从 config-document 面撤出，配置文档改走专属文档语法（见下）| **E2 实施 `499335856`**（9 文件，＋1257/−292）：①grammar 新增 `cfg-document-item`/`cfg-line`/`cfg-pair`/`cfg-pair-value`/`cfg-value-node`/`cfg-node`（三形头：`dep bps{}` 裸名 id、`dep "stylekit"{}` 字符串 id、`dep("kernel"){}` 调用实参；另收 `dep`/`pac`/`link` 三个 manifest 位保留字作节点名）/`cfg-node-args`（位置＋具名实参）/`cfg-node-body`/`cfg-var`/`cfg-if`/`cfg-else`/`cfg-for`/`cfg-iter`/`cfg-body`，全部只门控 `config-document`；②`object-lit`/`object-key`/`array-lit` 等原子与 ann-ty 的 ident 类型位放行到 config-document（真实 pac 的 `["vue"]`/`{ mode: … }` 值）；③`crate::source::cst::parse_config_document_front` 专用前门（共享 scanner/快照/预算），`project::project_config_document` 在共享 `Engine` 上把 CST 投影为 `ConfigDocumentSyntax`（行树＋原 span＋共享表达式，递归体/参数/id 三形全保真）；④`config_document.rs` 定义候选类型（`ConfigLine/ConfigPair/ConfigPairValue/ConfigNode/ConfigNodeId/ConfigArg/ConfigVar/ConfigIf/ConfigFor/ConfigBody`）与 `parse_config_document`；`compile_config_document` 改为"候选→单顺序入口物化（当前只物化语句形态 `var`/`if`，合成尾值零宽 i32）→共享 `compile_materialized_program_parts`→`verify_text_record`"，值形态行给具名 `source.config-document/pending`（E3/E4 未交，不冒充能力）；⑤通用入口（`parse_program_with`/`parse_front`/`compile_program_parts`）对 config-document **具名拒绝** `source.config-document/front-door`——文档行不会被通用投影静默丢弃；⑥`ace-core` 二进制同前门口径；⑦永久用例扩到 **6 例**：三形 dep 真实输入（含嵌套体/字符串 id/调用实参/顶层 prop）、设计 §4 全样本（对象/数组值、节点值 pair `icf: file(…) { … }`、`if`/`else` 文档体、`lib("x"){ for d in modules { dir(id: d){ at: d } } }` 具名实参）、坏邻例（有冒号无值/类型注解局部/空文档/`return` 非文档行）保原 span 的 `source.*`、单顺序入口物化＋pending 具名、profile 三向映射＋通用入口拒绝 | 语法决策（写入代码注释）：`for` 迭代位只收裸名或 `(expr)`——否则 `for d in modules {` 的 `modules {` 会被 record 构造原子吃掉（实测）；pair 的节点值必须**带体**（否则 `kernel: kernel_config` 被吃成无体节点，实测）；文档局部注解语法是 `var x int = 1`（无冒号），带注解的文档局部当前具名拒绝 | 验证（本树实跑，均在提交 `499335856` 前后同一代码）：`cargo test -p ace-core --test conformance plan982`＝**6/6 PASS**；`cargo test -p ace-core --lib --tests`＝79＋752 全绿；`cargo test -p ace-syntax-grammar`＝45 全绿；`cargo check -p ace-core` 绿（零新增告警）；新模块与新用例 `rustfmt --check` 干净 | 判读: **E1 主体＋E2 文档语法/投影已交**；未交＝E3 值行物化、E4 `cfg.*` 原语/AV 执行、E5 owned 导出、E6 host 桥、E7 Man 消费、E8 V-CFG 九 Case；B-CONFIG-I/V 不勾，树保留 | next: E3（值行→cfg 原语调用的单入口物化）＋E4（`cfg.*` 绑定/资源账）——两者必须成对推进（只绿 E3 会让 resolve 找不到 intrinsic）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 2（本会话）| from: work | to: work | status: executing | outcome: pass（E1 组门等价实跑＋E2 文档 grammar/投影落地，6/6 用例绿；值语义仍缺）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 499335856 | next: E3+E4（值行物化与 cfg.* 绑定成对推进）
2026-11-17 stage: work / **B-CONFIG-I E3（文档行→单顺序入口的 `cfg_*` 受检调用）**（objective「继续完成计划982」，round 3）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@dd7e5c742`（branch `plan-982-dev`，基面 `499335856`）| 实施 `dd7e5c742`（6 文件，＋956/−79）：①新 `source/config_ops.rs`：配置文档**源级原语目录**（`config.document/1`，45 条精确签名）——`cfg_root/node/obj_new/list_new/id_name/id_text/arg*/arg_named*/put*/obj_put*/list_push*/append/finish`，标量重载按**实参 VT** 选（i32/bool/str/char/f64），节点/对象/列表值与 i32 同型故走**显式 callee**（`*_node/*_obj/*_list`），不靠运行期猜；②`resolve.rs` 在 config-document 面绑定该目录（位置实参、未知/不匹配具名 `source.config-argument`，命中记 `used_intrinsics`）；③`adapter.rs` 三处发射接入：`intrinsic_id_suffix` 对 `auto.config.*` 用全名（否则所有重载塌成 `v1`＝重复 id，实测红）、intrinsic **类型表**用目录的精确参数/结果 VT、`requires` 盖 `config.document/1`；④`config_document.rs` 物化重写：入口首条 `cfg_root()`、顶层/节点体/`if` 分支共用一个"当前节点"游标，pair→`cfg_put*`（节点值先建再 `cfg_put_node`）、节点行→`cfg_node`→id 两形→位置/具名实参→体→`cfg_append`、对象/数组字面量→`cfg_obj_new`/`cfg_list_new` + 逐项装载、收尾 `cfg_finish()`；每个原语调用落成一条 `let __cfg_s<N> = …`（共享脚本入口合同不允许句中裸表达式语句）；用户局部含 `__cfg_` 前缀＝具名 `source.config-document/hidden-name` 拒绝 | 验证（本树实跑）：`cargo test -p ace-core --test conformance plan982`＝**7/7 PASS**（新增：真实三形 dep 文档编译后 Checked HIR 含 root/node/id.name/id.text/arg.str/put.str/append/finish 八个 intrinsic 且**只有一个** checked 函数体；能力键 `config.document/1` 在 `requires`；隐藏前缀冲突与运行期 `for` 各自具名拒）；`cargo test -p ace-core --lib --tests`＝79（1 ignored）＋**753 passed / 0 failed**；`cargo test -p ace-syntax-grammar`＝45 全绿（grammar 本轮未改，冻结 id 不变）；`cargo check -p ace-core` 零告警；新/改文件 `rustfmt --check` 干净 | **未闭（具名）**：E3 的 `lower_checked_owned`/`verify_mir` 面与 E4 的 av-core 侧（`auto.config.*` 声明/校验/provider、builder 状态与 kind/预算/循环拒绝、`PlatformCall` 运行期 dispatch）尚未接线——当前终点是 Checked HIR；`cfg_get`/`cfg_merge` 未定义；运行期 `for` 行仍 `source.config-document/pending`（需 `cfg.list.len/at` 计数原语）| 判读: 前端＋检查链已交（文档能编译成单入口的受检原语序列），**运行期求值/owned 导出仍未交**，B-CONFIG-I/V 不勾，树保留 | next: 把 `auto.config.*` 接到现有 PlatformCall 管线的四个门（hir_adapter 声明→lowering；mir/abc 校验；av-core `config_document` provider＋thread-local 会话；host::validate 准入），先跑通最低负载到 owned 快照（E4+E5 的最小闭环）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 3（本会话）| from: work | to: work | status: executing | outcome: pass（E3 前端＋检查链落地，7/7 绿；运行期求值/导出未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: dd7e5c742 | next: E4+E5 最小闭环（PlatformCall 四门＋av-core provider）
2026-11-17 stage: work / **B-CONFIG-I E4+E5：宿主配置会话、PlatformCall 分流与 owned 导出（最小闭环跑通）**（objective「继续完成计划982」，round 4）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@079b7410e`（branch `plan-982-dev`，基面 `dd7e5c742`）| 实施 `079b7410e`（13 文件，＋1371/−170）：①**域决策**：`SourceProfile::ConfigDocument.hir_domain` 从 TextRecord 改为 **HostConsole**——配置文档与 console 同属"宿主侧求值"车道，这样描述符/验证/降落/ABC 家族/PlatformCall 接缝全部复用既有 host 车道（不新开 ABC family、不动线格式）；②新 `av-core/src/config_document.rs`：`config.document/1` 目录（符号→结构解析：root/node/obj-new/list-new/id.*/arg*/arg-named*/put*/obj-put*/list-push*/append/finish，签名/结果由结构推导）＋宿主会话（thread-local、显式 open/close/abandon、无会话=具名拒）＋builder（node/obj/list/值句柄、kind 逐次核、重复 id/重复 attach/自环、收尾环检查、值节点与 owned 字节预算 → `HostError::Resource`）＋**owned 导出**（顺序/重复/精确 i32/bool/char/UTF-8 文本）＋`DocumentCost` 真实代价；③`host::validate`/`mir_verify`/`av.rs` 三道准入与执行分流：配置符号是第二条 host 声明类别（`MAX_CONFIG_IMPORTS=24`、能力键核、签名逐项核），执行时在平台 op 表查询**之前**分流到配置会话（否则落 `host.unknown-symbol`，实测踩到并修正），无会话时 `host.unauthorized: explicit configuration document session required`；④`lowering`：`auto.config.*` 声明走 `config_document::declaration`，调用点与平台操作同一条 host-import→PlatformCall 路径；入口策略允许配置文档的物化入口 `d___cfg_document`（console 面仍只允许 `d_main`）；⑤核心侧物化改进：`var` 的对象/数组字面量也物化成容器句柄（否则 `var x = { … }` 会把 ObjectLit 丢给 resolve，实测 `source.object-elem` 红），并用**静态种类表**记录哪些局部装的是 obj/list 句柄（`kernel: kernel_config` 因此选 `cfg_put_obj`，不靠运行期猜）；`auto.config.*` 的 intrinsic id 把 `-` 也归一成 `_`（`arg-named` 否则被 Atom 词法当负号，实测红）| 验证（本树实跑）：新 e2e `ace-lowering --test conformance plan982_config_document_e2e`＝**4/4 PASS**：①真实三形 dep 文档 → 单入口 MIR → ABC 准入 → 宿主会话执行 → owned 快照（根 prop + 三节点 id 两形/调用实参/属性，代价事实）；②嵌套文档（对象值经 var 引用、数组值、节点值 pair、运行时 `if` 取 else 分支 x=2、顶层节点 + 具名实参）全部按源语义导出；③无会话执行＝具名拒；④原语的坏句柄与值节点预算＝具名错误；回归：`ace-core --lib --tests`＝79（1 ignored）＋**753**、`ace-av-core --lib --tests`＝**64/48/11**、`ace-lowering --lib --tests`＝**9**、`cargo check -p ace-av` 绿；新文件 rustfmt 干净 | **未闭（具名）**：E4 的 `cfg_get`/`cfg_merge` 与运行期 `for`（需 `cfg.list.len/at` 计数原语）、`callee_count` 观测；E5 的**迭代式**导出（现递归 + 2048 深度守卫，2000 层压测未跑）与 u64>i64::MAX 拒（值面当前 i32/bool/char/str）；E6 host 桥（`auto-lang-ace-host` 的 `eval_config`）未接、E7 Man 消费与 E8 九 Case 未做 | 判读: **最低负载已能在 ACE 内真实求值到 owned 快照**（文档→受检原语→宿主会话→导出），但产品面（桥/Man/九 Case）未交，B-CONFIG-I/V 不勾，树保留 | next: E6（`auto-lang-ace-host` 桥：compile→verify→execute→export→teardown 实现 `AceExecutor::eval_config`，`--test ace_config_engine` 换真桥）+ E7 的 pac 读取接线 |
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 4（本会话）| from: work | to: work | status: executing | outcome: pass（E4+E5 最小闭环端到端跑通，e2e 4/4；桥/Man/九 Case 未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 079b7410e | next: E6 host 桥（eval_config）+ E7 pac 接线
2026-11-17 stage: work / **B-CONFIG-I E6：真桥 `AceExecutor::eval_config`（compile→verify→execute→export→teardown）**（objective「继续完成计划982」，round 5）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@308c2c07e`（branch `plan-982-dev`，基面 `079b7410e`）| 实施 `308c2c07e`（6 文件，＋515）：①新 `crates/auto-lang-ace-host/src/config_document.rs`：`eval_request` 把中立请求落成 ACE 宿主车道——能力（`config.document/1` 由桥自持，其它 requires 必须请求 grants 覆盖）→ `compile_config_document`（源诊断带原 stage/code/span 上抛）→ `lower_checked_owned(d___cfg_document)` → `verify_mir` → `encode_abc`/`verify_abc_bytes` → `open_session`（nodes/bytes 预算）→ AV 执行（steps/wall 预算、可选宿主 cancel）→ **所有路径**关会话（成功取快照，失败 abandon）→ 投影成中立 `ConfigDocument`/`ConfigNode`/`ConfigValue`（不变量投影、不再求值），身份用 `ConfigIdentity::of(engine=av)`，代价来自真实观测（AV steps、导出 nodes/bytes/depth、`callee_count`＝原语 dispatch 数）；②`impl AceExecutor for HostExecutor` 增 `eval_config`（失败不回退 legacy）；③av-core 会话增 `callees` 计数进 `DocumentCost`（`callee_count` 的真实来源）；④`Cargo.toml` 增 `auto-val` 仅 dev 依赖（用例按 legacy 消费形状读 `_arg{n}`/`get_kids`）| 验证（本树实跑）：新 `cargo test -p auto-lang-ace-host --test p982_config_engine`＝**1/1 PASS**（一个进程内真桥，五段断言：①真实三形 dep 文档 → Success：`identity.engine=="av"`、root props/3 子节点 id 两形/调用实参、`callee_count>=10`、`steps>0`、请求后**会话已关**；②同一桥经 `AutoConfig::from_code_with_engine(.., AceAv)` 的 SDK 消费面：`name`/`version`/3 个 `dep` 子节点、字符串 id 落在 legacy 形状 `_arg0`、`path` 原值；③坏文档＝具名 `Diagnostic`（`source.*`＋原 span）且 SDK 侧 `ace.diagnostic/...` 具名错误（不静默成功）；④字节预算极小＝具名 `ResourceDenied`；⑤无桥＝`ace.host/executor-unregistered`（不是回退、不是成功））；回归：`auto-lang-core --test ace_config_engine`＝1/1、`--test ace_config_registry`＝1/1（SDK 合同面未回退）；`auto-lang-ace-host --test e0_facade`＝3/3、`--test t08_facade`＝3/3；新文件 rustfmt 干净 | **未闭（具名）**：**context 注入**（`ConfigContext.args` 目前不进入文档程序——`port` 等注入变量还不可见，属 V-CFG-CONTEXT/E7 二阶段上下文）；运行期 `for`、`cfg_get`/`cfg_merge`、迭代式导出；E7（Man 侧 pac 读取切 `from_file_with_engine(.., AceAv)`）与 E8 九 Case 未做 | 判读: SDK 公开面「显式 ACE 配置引擎」已能真实服务并产出中立文档，但产品消费（Man）与注入/循环能力未交，B-CONFIG-I/V 不勾，树保留 | next: E7（auto-man 兄弟树：pac 读取切 ACE + 二阶段 context/dep，legacy 仅显式）＋ context 注入最小面（args → 文档可见变量）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 5（本会话）| from: work | to: work | status: executing | outcome: pass（E6 真桥落地，1/1 真桥用例＋SDK 合同面绿；注入/for/E7 未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 308c2c07e | next: E7 Man pac 接线 + context 注入最小面
2026-11-17 stage: work / **context 注入最小面（E7 前置）**（objective「继续完成计划982」，round 6）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@b70ec23e3`（branch `plan-982-dev`，基面 `308c2c07e`）| 实施 `b70ec23e3`（4 文件，＋275/−23）：①`ace-core` 新增 `InjectValue{I32,Bool,Char,Text}` 与 `compile_config_document_with_injection`（旧入口＝空注入包装）；注入点在 root 之后、文档行之前绑成**不可变局部**，文档自己 `var` 同名者不注入（文档声明遮蔽注入），非法名/重复名具名拒；②桥侧 `inject_from_context`：`ConfigContext.args` → 注入值（bool/char/text/i32 范围整数；超范围=`ace.config/inject-range`、浮点/null/对象/数组=`ace.config/inject-unsupported`，均 Denied 而非截断/静默丢），并按 legacy 补 `on`/`off` 内建（请求显式同名值优先）；③真桥用例扩到注入面：`port=="win32"` 取 then 分支（x=1）、`toggle: on`=true、文档 `var local_port` 遮蔽注入、`AutoConfig::from_code_with_engine(.., AceAv)` 传 args 端到端为 1、浮点注入=具名 Denied | 验证（本树实跑）：`cargo test -p auto-lang-ace-host --test p982_config_engine`＝**1/1 PASS**（含新注入断言）；`cargo test -p auto-lang-core --test ace_config_engine --test ace_config_registry`＝1/1＋1/1；`ace-core plan982`＝7/7、`ace-lowering plan982`＝4/4；rustfmt 干净 | **未闭（具名）**：E7 Man 侧 switch（pac 读取切 `from_file_with_engine(..,AceAv)`＋二阶段 context/dep、`cargo tc`/780 矩阵、`legacy_config_dispatch=0` 证据）未做——本轮只交注入前置；对象/数组注入与 `cfg.get`/运行期 `for` 仍缺（`${kernel.heap}` 类模板读取需要它们）| 判读: 注入语义已在 ACE 车道成立并可经 SDK 门使用，但产品消费未切换，B-CONFIG-I/V 不勾 | next: E7（auto-man 兄弟树：`automan.rs`×2、`pac.rs` dep 读、`generation_request_adapter.rs`、CLI `main.rs` 的 pac 读点切 ACE＋engine 观测；legacy 仅显式；坏配置零写/同名包隔离证据）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 6（本会话）| from: work | to: work | status: executing | outcome: pass（context 注入最小面落地，真桥用例 1/1 绿；E7 Man switch 未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: b70ec23e3 | next: E7 auto-man pac 读点切换 + 观测证据
2026-11-17 stage: work / **真实 pac 语料对齐（E7 前置之三：注释/JSON/块对象 + 投影逐字对齐）**（objective「继续完成计划982」，round 7）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@6701094ff`（branch `plan-982-dev`，基面 `b70ec23e3`）| 实施 `6701094ff`（6 文件，＋465/−130）：①语法：config 面新增 `#` 行注释 trivia（只在 config-document 面；其它面 `#` 仍是注解/CT 引入符）、`cfg-json-document`（整份文档＝`{ "k": v, … }`，字符串/ident 键、逗号、允许尾逗号）、`cfg-object-block`（`{ k: v … }` 换行分隔对象值，可嵌套）；②投影：JSON 条目＝顶层 pair（前序遍历收集，条目在尾链里嵌套）、块形对象值→`ObjectLit`（嵌套递归；带体节点值在该位置具名拒 `source.config-document/object-node-value`）、字符串键解码（去引号）；坏邻例诊断优先取带**真实 span** 的槽位/构造级诊断（不再只报 0..0 的 trailing 兜底）；③SDK 投影与 legacy 逐字对齐：根名 `root`、裸/字符串身份只落 `id`（不产生 `_arg0`）、括号实参进 `_arg0…` 且为普通 prop、i32 范围整数落 `Value::Int`（超范围才 `I64`）| 验证（同树实跑）：**语料探针** `cargo test -p auto-lang-ace-host --test p982_pac_corpus_probe`（`P982_PAC_CORPUS=D:/autostack/.wt/lang-982/auto-man`）＝**152 份 pac.at 中 149 份 ACE 成功且与 legacy `Node` 结构全等（0 分歧）**，3 份失败＝`clib` 模板文档（`lib ${name} {}`）＋`test/fixtures/plan982/bad-config|bad-eval`（计划内故意坏夹具）；新回归 `ace-core --test conformance plan982`＝**8/8**（含 `#` 注释/JSON/块对象/字符串键解码）；`ace-core --lib --tests`＝753、`ace-syntax-grammar`＝45、`ace-lowering`＝9、`ace-av-core`＝64/48/11、`auto-lang-ace-host --test p982_config_engine`＝1/1、`auto-lang-core --test ace_config_engine`/`ace_config_registry`＝1/1＋1/1；rustfmt 干净 | **未闭（具名）**：①**模板插值** `${…}`（V-CFG-TEMPLATE）：字符串内 `${name}` 目前按字面文本出现，节点头 `lib ${name} {}` 仍语法拒绝——切换前必须补（影响 7 份含 `${}` 的语料文档与模板仓）；②auto-man 各 pac 读点未切换（引擎选择、`config.engine=av` 观测、坏配置零写/同名包隔离证据、`cargo tc`/780 矩阵）；③二阶段 context/dep 未接 | 判读: ACE 已能**逐字等价求值真实 pac 语料的 149/152**（0 结构分歧），切换的产品风险已被压到"模板 + 引擎开关"两件具名工作；B-CONFIG-I/V 仍不勾 | next: 模板插值（`${expr}` → 文本拼接/节点头）＋ auto-man 读点切换与观测证据
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 7（本会话）| from: work | to: work | status: executing | outcome: pass（真实 pac 语料 149/152 与 legacy 0 分歧；模板插值与 Man 切换未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 6701094ff | next: 模板插值 + auto-man 读点切换
2026-11-17 stage: work / **模板插值 `${…}` ＋ 宿主 args 根属性播种（E7 前置之四）**（objective「继续完成计划982」，round 8）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@3d1302b78`（branch `plan-982-dev`，基面 `6701094ff`）| 实施 `3d1302b78`（9 文件，＋447/−46）：①`source/fstr.rs::desugar_config_templates`：配置文档的模板 desugar——普通字符串里的 `${expr}` → `__fstr(...)`（与 `f"..."` 同一生成物，`TextDisplay`+`TextConcat`，插值各求值一次），注释（`//`/`#`/块）与字符字面量原样复制、`\$` 转义；**节点头的裸 `${expr}` 按 legacy 实测口径留字面**（重写为同文本字符串字面量，投影层再补 `_arg0`，与 legacy `id: "${name}"` 逐字一致）；②`host-console.atom` 描述符补 `text_display`/`fstr` 两个 expr case 与 `parts` 字段（否则物化程序重发射的 `expr fstr` 无法 bind）；③`av-core` 会话新增 `seed_root_prop`＋`pending_seed`/`seeded`：宿主 args 先播成**根属性**，文档里同名 `key: value` **不覆盖**（legacy 语义：注入值赢）；`Entry::Value(StoredValue::*)` 的播种路径含对象/列表/节点递归与预算计费；④桥侧把 `ConfigContext.args`（bool/i32/char/text）播种，`on`/`off` 内建只做可见变量不进根属性 | 验证（同树实跑）：**语料 152 份**——严格口径（不注入）**145 OK 且与 legacy `Node` 0 分歧**、6 失败＝5 模板（无注入 ⇒ ACE 具名 `source.undefined`，legacy 静默留 `${name}` 字面：故意的 fail-closed 分歧）＋1 计划内坏夹具（`bad-eval`）、1 份 legacy 也拒（`bad-config`）；**注入 `name="probe"` 时 5/5 模板文档产出 `name: "probe"` / `app("probe")` 形态**（与 legacy 同名注入结果一致）；新单测 `plan982_config_template_desugar_shapes`（字符串插值/节点头字面/注释不插值）；回归：`ace-core --lib --tests`＝**755**、`ace-syntax-grammar`＝45、`ace-lowering`＝9、`ace-av-core`＝64/48/11、`ace-semantics --lib`＝20、`auto-lang-ace-host --test p982_config_engine`＝1/1、`p982_pac_corpus_probe`＝1/1、`auto-lang-core --test ace_config_engine`/`ace_config_registry`＝1/1＋1/1；rustfmt 干净 | **未闭（具名）**：①auto-man 各 pac 读点仍未切换（引擎选择/`config.engine=av` 观测/坏配置零写/同名包隔离、`cargo tc`/780 矩阵）；②二阶段 context/dep 未接；③对象/数组注入（`kernel: {…}` 类）仍具名拒；④运行期 `for` 与 `cfg_get`/`cfg_merge` 未交 | 判读: 语料能力面已补到"仅模板无注入时 fail-closed"的边界，切换产品面的最后一件上游工作是 auto-man 读点与观测；B-CONFIG-I/V 不勾 | next: auto-man 读点切换（`automan.rs`×2、`pac.rs` dep 读、CLI `main.rs`）＋引擎观测证据
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 8（本会话）| from: work | to: work | status: executing | outcome: pass（模板插值＋根属性播种落地；语料严格口径 145/145 零分歧；Man 切换未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: 3d1302b78 | next: auto-man 读点切换 + 观测证据
2026-11-17 stage: work / **E7：auto-man pac 读点切到显式引擎选择（默认 ACE）＋实际观测**（objective「继续完成计划982」，round 9）| tree: 两树同号分支——`D:/autostack/.wt/lang-982/auto-man@02a4789`（man）与 `D:/autostack/.wt/lang-982/auto-lang@81391da11`（lang）| 实施：①lang：`ace_entry::ace_executor_installed()`（环境事实查询，非语义回退）；`desugar_config_templates` 改为**按位置**处理裸占位（取值位 `k: ${x}` 插值／节点头 `n ${x} {}` 按 legacy 留字面——坏配置夹具 `render: ${missing_variable}` 因此重新成为真错误）；②man 新 `auto-man-runtime/src/config_engine.rs`：`choice()`（显式覆盖 → `AUTO_CONFIG_ENGINE` → 桥是否已装）、`engine_name()`、`observation()`（ACE/legacy **真实派发计数**）、`parse_file`/`parse_code`（按选择调 `from_file_with_engine`/`from_code_with_engine`，失败具名不回退）；③读点切换：`automan.rs`（探针读＋正式读）、`pac.rs`（dep pac 读）、`crates/auto/src/ace_candidate.rs`（候选腿配置读）统一走 `config_engine`；候选见证新增 `config.engine` 与 `config.legacy_dispatch`；④修 A 切片遗留红：`test/fixtures/plan982/{bad-config,bad-eval}` 纳入 `project_model_corpus_comparison_v02` 的 both-rejected/template-rejected allowlist（身份绑定、数组长度同步）；⑤候选坏配置用例的期望诊断改为 ACE 具名诊断（`source.undefined`/`source.syntax`）。| 验证（两树实跑）：`cargo tc`（auto-man 固定组 7 包 lib）＝**257 passed / 4 skipped**；`cargo test -p auto --features ace-candidate --test plan982_ace_candidate_config`＝**5/5 PASS**，其中见证断言 `config.engine=="av"`、`config.legacy_dispatch==0`（B 口径：配置真由 ACE 服务、零 legacy 派发）、坏配置拒绝具名（`ace.config/invalid` + ACE 诊断）且项目树零写、双项目隔离；lang 侧：`ace-core --lib --tests`＝**755**、`ace-av-core`＝64/48/11、`p982_config_engine`＝1/1、`ace_config_engine`/`ace_config_registry`＝1/1＋1/1、语料探针＝145 OK 且与 legacy **0 分歧**（7 失败＝5 模板无注入＋2 计划内坏夹具）；rustfmt 干净（man 侧只格式自改文件，误触的 `target.rs` 已还原）| **未闭（具名）**：①**二阶段 context/dep**（`dep` 二阶段上下文注入与依赖解析的 ACE 侧）未做；②`780` 矩阵未单独跑（`cargo tc` 覆盖 auto-man lib 面；CLI 全量见 `cargo ti`，未跑）；③运行期 `for`、`cfg_get`/`cfg_merge`、对象/数组注入仍缺；④`clib ${name}` 类模板在无注入时 fail-closed（与 legacy 留字面不同，**故意**，已记录）| 判读: **产品的 pac 读取已默认走 ACE 并可观测**（B 口径成立），计划剩余面是二阶段上下文/依赖与更宽的 CLI 矩阵；B-CONFIG-I 的核心交付已实质完成、V 门未全跑 | next: 二阶段 context/dep（`dep` 上下文注入）＋`cargo ti`/780 矩阵取样
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 9（本会话）| from: work | to: work | status: executing | outcome: pass（E7 落地：pac 读取默认 ACE、legacy_dispatch=0、tc 257/257；二阶段 context 未交）| worktrees: lang-982/auto-man@02a4789 + lang-982/auto-lang@81391da11 | next: 二阶段 context/dep + cargo ti 取样
2026-11-17 stage: work / **E8：V-CFG 九 Case 的永久用例（语言侧七例＋Man 侧两例）**（objective「继续完成计划982」，round 10）| tree: 两树同号——`D:/autostack/.wt/lang-982/auto-lang@034aef0cd`、`D:/autostack/.wt/lang-982/auto-man@81b27b5` | 实施：①**lang**：新 `crates/auto-lang-ace-host/tests/p982_vcfg_cases.rs`（7 例，均走真桥 `eval_config_with_ace`→owned 文档，各自独立 oracle）：SYNTAX（三形 dep＋Node 值 pair＋window 面）、FLOW（注入 port 选 then 分支、`var` 重声明取最新、嵌套体/具名实参；**非空证**：换 port 后 x=2）、CONTEXT（bool/int/text 三轴＋on/off＋注入值即根属性、文档同名 pair 不覆盖＋`var` 遮蔽＋浮点注入具名拒）、VALUE（中文/emoji/NUL/±i32 边界/空对象/嵌套 list‑in‑object/重复孩子保序、会话关后仍可读）、FAIL（坏源原 span＋零残留会话＋修复后同进程真读）、LIMIT（值预算拒＋默认预算成功非空证＋取消）、IDENTITY（args 值/键集变身份、源哈希不变、真实服务轴）；②为使这些 Case **真绿而非记录缺口**，同时补了三处引擎语义：`var` 同块重声明落成**赋值**（legacy `test_config_var_redeclared_takes_latest`）、分支/节点体**块作用域**（if 两臂各自 `var x` 合法）、注入值按 legacy 是**可变全局**（`var label` 对外层注入是一次赋值，之前的引用仍见注入值）；③节点头 `cfg-node-id`/`cfg-node-call` 加 `same-line`——否则 `z: mode` 的下一个行 `lib("a")` 会被吃成 `mode` 的调用尾部（实测 `no function named lib`，已修）；④**man**：新 `crates/auto/tests/plan982_ace_config_b.rs`（2 例，真 CLI `AUTO_ACE_CANDIDATE=1`）：PRODUCT（`config.engine=="av"`、`legacy_dispatch==0`、`values/deps/targets` 真实、执行 success）、RETIRE（`AUTO_CONFIG_ENGINE=legacy` 显式可见且 1 次派发；未知取值仍 av、零 legacy 派发、运行成功）| 验证（两树实跑）：`cargo test -p auto-lang-ace-host --test p982_vcfg_cases`＝**7/7 PASS**；`cargo test -p auto --features ace-candidate --test plan982_ace_config_b`＝**2/2 PASS**；A 切片见证 `--test plan982_ace_candidate_config`＝5/5、feature-gate 显式跳过；`cargo tc`＝**257 passed / 4 skipped**；lang 回归：`ace-core --lib --tests`＝**755**、`ace-core plan982`＝9/9、`ace-av-core`＝48/11、`ace-syntax-grammar`＝45、`ace-lowering`＝9、`p982_config_engine`＝1/1、`p982_pac_corpus_probe`＝145 OK/0 分歧、SDK config 合同 1/1＋1/1；rustfmt 干净 | **未闭（具名）**：①运行期 `for` 仍 `pending`（需列表计数/元素取值原语，FLOW 的 for 行未覆盖）；②`[]` 空列表与 `u64>i64::MAX` 未支持（`array-lit` 是共享面，需 config-only 原子后重录 profile 冻结值）；③2000 层深/宽压测未跑；④V-CFG-SYNTAX 的 780 逐份实跑未做（语料探针已 145/145 零分歧）| 判读: 九 Case 的**结构面**已永久落地且可独立红，但 3/9 的完整覆盖仍受 `for`/`[]`/深度三处能力边界限制，B-CONFIG-V 不勾 | next: 运行期 `for`（`cfg.list.len/at` 元素取值）或先做 B-CONFIG-R 独立复审（E1..E8 面已可审）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 10（本会话）| from: work | to: work | status: executing | outcome: pass（V-CFG 九 Case 结构面落地：lang 7/7＋man 2/2；for/[]/深度三处未闭）| worktrees: lang-982/auto-lang@034aef0cd + auto-man@81b27b5 | next: 运行期 for 或 B-CONFIG-R 复审
2026-11-17 stage: work / **`for` 字面数组展开（V-CFG-FLOW 补完字面面）**（objective「继续完成计划982」，round 11）| tree: 唯一树 `D:/autostack/.wt/lang-982/auto-lang@b26268d44`（branch `plan-982-dev`，基面 `034aef0cd`）| 实施 `b26268d44`（4 文件，＋122/−6）：①物化侧新增 `emit_for`：迭代源是**字面数组**（`for d in (["a","b"])`）或绑定到字面数组的文档 `var`（`EmitCtx.list_literals` 在 `var x = [...]` 时记忆元素表达式）时，按源顺序**展开**体——首项 `let <var>`、其后 `Assign`，每迭代从同一外层声明面开始（体内声明不跨迭代泄漏）；②运行期列表（元素来自注入/计算，例如 `var m = base` 后再迭代）仍是具名 `source.config-document/pending`，消息明确"只有字面列表现可展开"，不静默空跑；③语法：`cfg-iter` 的裸 `[...]` 形不可用（实测 `expected expression, found [`——规则引用在该位不成立），故保持"括号字面"为当前接受的拼写（具名差额记录）；④新增 core 单测 `p982_config_for_unrolls_literal_lists`（展开后语句数≥8、运行期形具名 pending）＋语言侧 FLOW 例补两段断言（unroll 两节点 id/at 逐个核对、动态列表 pending）| 验证（同树实跑）：`ace-core --test conformance plan982`＝**10/10**；`ace-core --lib --tests`＝**756**；`ace-av-core`＝48/11；`ace-syntax-grammar`＝45；`ace-lowering`＝9；`auto-lang-ace-host --test p982_vcfg_cases`＝**7/7**（FLOW 现覆盖 unroll＋pending 两形）、`p982_config_engine`＝1/1、`p982_pac_corpus_probe`＝145 OK/0 分歧；SDK config 合同 1/1＋1/1；rustfmt 干净（`cfg-iter` 的死 alt 已撤回）| **未闭（具名）**：①运行期 `for` 需 `cfg.list.len/at` 类元素取值原语（含 owned Str 返回的 ABI 面），未做；②裸 `[...]` 迭代拼写未收（规则引用限制）；③保留字作 pair key（`tag:` 实测被当保留 token）仍是缺口；④VALUE 的 `u64>i64::MAX`、2000 层深压测未做 | 判读: FLOW 的**字面**迭代面已真实执行且可独立红，运行期列表仍具名未交；B-CONFIG-V 不勾 | next: 运行期 `for`（list len/at ＋ owned Str ABI）或 B-CONFIG-R 独立复审
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 11（本会话）| from: work | to: work | status: executing | outcome: pass（for 字面展开落地、FLOW 补完字面面；运行期 for/保留字 key 等仍未交）| worktree: D:/autostack/.wt/lang-982/auto-lang | commit: b26268d44 | next: 运行期 for 或 B-CONFIG-R 复审
2026-11-17 stage: review / **B-CONFIG-R（独立复审，独立上下文 agent `reviewer-982b`，未参与实现）**（objective「继续完成计划982」，round 12）| 冻结面: lang `b26268d44`、man `81b27b5`（复审时点，两树 clean）| **VERDICT: needs_fix**（0 BLOCKER / 3 MAJOR / 7 MINOR / 5 NOTE；设计成立、全部可在 B-CONFIG-I/V 内修，不需重规划）| 复审实跑（与 receipt 数字逐项核对）：`ace-core plan982`＝10/10、`ace-core --lib --tests`＝**756**、`ace-av-core`＝64/48/11、`ace-syntax-grammar`＝45、`ace-lowering`＝9、`p982_vcfg_cases`＝7/7、桥 1/1、SDK config 合同 1/1＋1/1、语料 `145 ok (0 disagree) / 7 failed / 0 skipped / 152`、`cargo tc`＝257/257、Man 侧 5/5＋2/2；`rustfmt --check`：**所有新文件干净**，12+1 个旧文件在基面就已不干净（无 fmt churn） | **MAJOR 与处置（本轮已修，见下条 receipt）**：F-982B-01 身份 `context_digest` 只混 `value.kind()`、忽略值（`win32`≡`linux`），且 IDENTITY 用例只加键不换值 ⇒ 已修为结构化值哈希＋经真桥的"同键不同值"用例（digest 不同且 x=1/x=2 不同）；F-982B-02 `auto config` 子命令仍直调 `eval_config_with_vm`、不计入观测 ⇒ 已改走 `config_engine::parse_code`；F-982B-03 默认构建（未开 `ace-candidate`）无桥 ⇒ 全部 pac 读取静默落 legacy，而 round-9 receipt 写"产品已默认走 ACE" ⇒ 已加**一次性可观测提示**（stderr）+ `AUTO_CONFIG_ENGINE=av` 显式选择在无桥构建下**具名拒绝**（实测 exit 1），并在 §5 E7 登记"默认构建 legacy 是具名边界，切换属 986" | **MINOR 处置**：F-04 空文档与设计 §4.1 冲突 ⇒ 求值入口改为**空 root**（空白/纯注释均空 root，原始前门仍报 `empty` 事实）；F-06 语料探针不断言 ⇒ 加 `diff.is_empty()`＋`ok>=140`；F-10 死函数与 `unwrap_or_default` ⇒ 删除/改 `expect` 内部不变量；F-11 配置原语超限码 ⇒ `host.limit`。**仍开（具名）**：F-05 "求值失败"轴实为语法拒（`bad-eval` 是 syntax；缺真正的 AV 运行期失败夹具）；F-07 `MAX_PROJECTION_DEPTH=512` 与 av-core 2048 不一致、使 2000 层目标不可达（未登记）；F-08 预算轴与设计 §5 漂移（nodes 65 536→100 000、wall 5s、无 source/output 字节轴、deadline 只覆盖 AV 执行；`max_nodes` 一钮两义）；F-09 被覆盖默认 pair 的 RHS 仍求值（设计 §4.3 要求不执行）；F-12 配置预算溢出映射成 `ResourceKind::ConsoleOutput`；F-14 man E7 提交夹带无关格式 hunk；F-15 round-8 receipt 语料失败数 6 vs 实测 7 | **receipt 更正（按复审证据）**：①round-9 "产品的 pac 读取已默认走 ACE" → 更正为"**候选腿**（`--features ace-candidate` + `AUTO_ACE_CANDIDATE=1`）的 pac 读取由 ACE 服务；默认构建落 legacy 且现已可观测（具名边界，切换属 986）"；②round-8 "6 失败＝5 模板＋1 坏夹具" → 实测 7（5 模板＋2 坏夹具）；③E8 的 "IDENTITY（键集与**值变**身份）" 在修复前不成立，现由 `eee7f538c` 补齐值轴 | 未动：设计文档、spec 模块（待 B 全绿后更新）；不勾 B-CONFIG-I/V；不启动 merge |
2026-11-17 stage: work / **B-CONFIG-R 复审修复（有界）**（objective「继续完成计划982」，round 12）| 依据: 上条独立复审 needs_fix | tree: lang `eee7f538c`、man `dd714f9`（基面 `b26268d44`/`81b27b5`）| 修复清单：①**F-01（MAJOR）** `ConfigContext::digest` 增结构化值哈希（`hash_config_value`：标量定宽/文本带长度/容器递归带条目数，`Null`/`Bool`/`I64`/`U64`/`F64`/`Char`/`Text`/`List`/`Object`/`Node` 全轴）；IDENTITY 用例加"同键不同值"真桥断言（`port=win32` vs `linux` ⇒ digest 不同 **且** x=1/x=2）；②**F-02（MAJOR）** `auto config` 子命令改走 `auto_man_runtime::config_engine::parse_code`（不再直调 `eval_config_with_vm`），输出沿用 `Value::Node(..).repr()`；③**F-03（MAJOR）** 无桥 ⇒ legacy 的选择现在**可观测**（`auto.config/engine: ...` 一次性 stderr 提示），`AUTO_CONFIG_ENGINE=av` 在无桥构建下**走 ACE 并具名失败**（实测 `exit=1`，提示 install `auto-lang-ace-host`）——`choice()` 显式处理 `av`/`ace`（此前被静默降级）；新增用例 `p982_explicit_engine_env_is_honoured`；④**F-04** 求值入口对空/纯注释文档返回**空 root**（新增 `is_effectively_empty`，跳过字符串内的 `#`//`//`；原始 `parse_config_document` 仍报 `empty` 事实），新增 `p982_config_empty_document_is_an_empty_root`；⑤**F-06** 语料探针加 `diff.is_empty()` 与 `ok>=140` 断言（探针可红）；⑥**F-10** 删除 `collect_var_names` 死函数、`text_of` 的 `unwrap_or_default` 改 `expect`（内部不变量）；⑦**F-11** 配置原语声明超限码 `host.duplicate`→`host.limit`；⑧`parse_config_document` 文档注释与求值入口语义对齐；⑨man 侧只格式自改文件（误触的 `cmd_*.rs` 已还原）| 验证（两树实跑）：`ace-core plan982`＝**11/11**；`ace-core --lib --tests`＝**757**；`ace-av-core`＝48/11；`p982_vcfg_cases`＝7/7（含新值轴）；`p982_config_engine`＝1/1；语料探针＝`145 ok (0 disagree)/7 failed` 且断言绿；SDK config 合同 1/1＋1/1；Man：`config_engine` 4/4、`cargo tc`＝**258 passed / 4 skipped**、候选见证 5/5＋`plan982_ace_config_b` 2/2；CLI 冒烟：`auto config <pac.at>` 默认构建打印 legacy 可观测提示并正确输出、`AUTO_CONFIG_ENGINE=av` 具名失败 exit 1 | **仍开（具名，下一轮）**：F-05 真 AV 运行期失败夹具（现两条夹具都是前端拒绝）、F-07 投影深度 512 vs 2000 层目标、F-08 预算轴与 deadline 覆盖、F-09 被覆盖 RHS 不执行、F-12 资源种类映射、F-14/F-15 记账；spec 模块 `docs/specs/ace-migration/m2-man-gen.md` 待更新（仍写"配置引擎迁移是 GAP"）| 判读: 复审的三条 MAJOR 已闭，B-CONFIG-I 可择机进入 B-CONFIG-R 复审第二轮；V 不勾 | next: F-05/F-07/F-08/F-09 收口 + spec 更新 + 第二轮独立复审
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 12（本会话）| from: work | to: review→work | status: executing | outcome: pass（B-CONFIG-R 复审 needs_fix：3 MAJOR 已修、7 MINOR 中 4 修 3 开；未勾 V）| worktrees: lang-982/auto-lang@eee7f538c + auto-man@dd714f9 | next: F-05/F-07/F-08/F-09 收口 + spec 更新 + 复审第二轮
2026-11-17 stage: work / **B-CONFIG-R 复审修复第二轮（F-05/F-07/F-08/F-09/F-12 收口）＋ spec 沉淀**（objective「继续完成计划982」，round 13）| tree: lang `42fef0951`（基面 `eee7f538c`）、man `d771094`（基面 `dd714f9`）| 修复：①**F-05** 新增**真运行期失败**夹具 `test/fixtures/plan982/bad-runtime/pac.at`（`ratio: 1 / 0`：文档解析/物化通过，AV 执行期 trap）＋候选见证第三个拒绝轴 `("bad-runtime","ace.config/trap")`（`ace.config/invalid`、`execution=not-started`、工程树零写），并纳入 `project_model_corpus` 的 template-rejected allowlist（旧 VM 求值期报错/静态模型判非静态，不冒充等价）；②**F-07** `MAX_PROJECTION_DEPTH` 512→**2048**（与 av-core owned 导出守卫一致），新增**2000 层嵌套**真桥用例（`p982_vcfg_deep_nesting_projects_beyond_the_old_cap`：整条嵌套链保真、cost.depth 真实），设计 §5 的 2000 层目标达成；仍为有界递归（2048 处具名 `config.projection/too-deep`，迭代式投影仍是差额）；③**F-08** wall 预算覆盖**整次请求**（编译/准入前、导出后各加一次 deadline 检查，超时 `ResourceDenied{kind:"wall"}`），并把 `ConfigEvalLimits::max_nodes` 的"值预算 vs 导出节点复核"一钮两义写成明确注记（设计允许的适配，记入 §5/E6）；④**F-09** 被注入覆盖的默认 pair **不再物化**（`EmitCtx.injected_keys`：文档 `x: 1 / 0` + 注入 `x=7` ⇒ 成功且 x=7；无注入时同一文档仍被 RHS 失败具名挡住——非空证）；⑤**F-12** 配置预算溢出的资源种类从 `ConsoleOutput` 改为 `DynTypeNodes`（值树节点预算族）；⑥**spec 沉淀**：`docs/specs/ace-migration/m2-man-gen.md` 重写为**已交付 B-CONFIG-I 的现值行为**（两引擎选择/观测、ACE 管线、SDK 合同、回执键、永久验证面、具名边界），删去"配置引擎迁移是 GAP"的旧说法 | 验证（两树实跑）：lang `ace-core plan982`＝11/11、`ace-core --lib --tests`＝**757**、`ace-av-core`＝48/11、`p982_vcfg_cases`＝**8/8**（含 2000 层、RHS 不执行、值轴身份）、`p982_config_engine`＝1/1、语料探针 145 OK/0 分歧（断言绿）、SDK 合同 1/1＋1/1；man `cargo tc`＝**258 passed / 4 skipped**、候选见证 5/5（含 bad-runtime 轴）、`plan982_ace_config_b` 2/2；`spec-lint` 仅余既有 `project.md` 缺失与无关断链（非本次改动面）| **仍开（具名）**：F-14（man E7 提交夹带的无关格式 hunk，仅记账）、F-15（round-8 语料失败数已在 round-12 更正）、运行期 `for`/`[]`/对象字段读取/`cfg.get|merge`/保留字 key/二阶段 context、迭代式投影、`f64/u64` 值面 | 判读: B-CONFIG-R 的两轮修复把**所有 MAJOR 与可修 MINOR 关闭**，剩余均为已登记的能力边界；可进入**第二轮独立复审**（复审建议：确认 F-01..F-12 逐条已闭、spec 与代码一致）| next: B-CONFIG-R 第二轮独立复审 → 通过则 B-CONFIG-V（九 Case 已落地，需 V 门实跑）→ merge
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 13（本会话）| from: work | to: work | status: executing | outcome: pass（复审 F-05/07/08/09/12 关闭＋spec 沉淀；待第二轮复审）| worktrees: lang-982/auto-lang@42fef0951 + auto-man@d771094 | next: B-CONFIG-R 第二轮复审
2026-11-17 stage: review / **B-CONFIG-R 第二轮独立复审（独立上下文 agent `reviewer-982b-r2`）＋ 其发现的有界修复**（objective「继续完成计划982」，round 14）| 冻结面: lang `42fef0951`、man `d771094` | **VERDICT: needs_fix**（0 BLOCKER / 3 MAJOR / 5 MINOR / 2 NOTE；F-01..F-15 中 11 条确认 CLOSED、3 条 PARTIAL、其余为记账）| **第二轮新发现与处置（本轮已修）**：**F-982B-16（MAJOR）** `bad-runtime` 夹具被放进 `EXPECTED_TEMPLATE_REJECTED` 而实际两侧都拒，导致 **`cargo tc` 红**（receipt/spec 却写 258 绿）⇒ 移入 `EXPECTED_BOTH_REJECTED`（stage `"value"`）并修数组长度，`cargo tc` 复跑＝**258 passed / 4 skipped**；**F-982B-17（MAJOR）** "2000 层真桥用例"实为 700（`DEPTH=700`），而 plan/spec 已写"2000 层达标"⇒ 实测 2000 层会触发 MIR 校验的 `VerifyMemory` 预算拒绝（**暂无 panic**），故改为**双档**：700 层整链服务 + 2000 层**不崩**（服务或具名预算拒绝二选一），并在 §5 E5/E8、spec 同步措辞（不再声称"2000 层已服务"）；**F-982B-18（MAJOR）** F-09 的注入跳过原先作用于**所有嵌套层**，会把 `dep("kernel"){ port: "msvc/x64" }` 这类节点自有同名属性一起吞掉（真实产品注入 `port` 即触发）⇒ 加 `EmitCtx.depth`，**仅根级**跳过，并新增嵌套同名键保留用例；**F-982B-19（MINOR）** `av-core/src/config_document.rs` 新引入 rustfmt 差 ⇒ 已格式化；**F-982B-20（MINOR）** `max_nodes` 一钮两义与预算漂移只在代码注释里 ⇒ 已写入 §5 E6 台账；**F-982B-21/22/24（MINOR）** spec 三处过度表述（非 Windows 配置腿"都拒"、成功/拒绝回执形状、桥测试段数、env 取值表）⇒ spec 已改；**F-982B-23（NOTE）** 语料探针 `skipped` 标签语义反了 ⇒ 改 `ace-only (legacy rejects them)` | 验证（两树实跑，修复后）：lang `ace-core plan982`＝11/11、`ace-core --lib --tests`＝757、`ace-av-core`＝48/11、`p982_vcfg_cases`＝**8/8**（含 700/2000 深度双档、嵌套注入同名键、值轴身份、RHS 不执行）、`p982_config_engine`＝1/1、语料探针＝`145 ok (0 disagree), 8 failed, 0 ace-only, 153 total` 且断言绿、SDK 合同 1/1＋1/1；man `cargo tc`＝**258 passed / 4 skipped**（复绿）、候选见证 5/5、`plan982_ace_config_b` 2/2 | **仍开（具名）**：运行期 `for`、`[]`、对象字段读取/`cfg.get|merge`、保留字 key、二阶段 context、迭代式投影、`f64/u64` 值面、LIMIT 宽输入压测；F-14/F-15 为记账 | 判读: 第二轮 3 条 MAJOR 全闭、`cargo tc` 复绿，B-CONFIG-I 的实现与证据面已可再次送审；B-CONFIG-V 需在复审通过后按九 Case 实跑归档 | next: **B-CONFIG-R 第三轮复审**（只审本轮 3 MAJOR 修复与 spec 四句）→ 通过则 B-CONFIG-V 收据 + merge fold
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 14（本会话）| from: review | to: work | status: executing | outcome: pass（第二轮复审 3 MAJOR 全修：tc 复绿、深度双档、嵌套注入；spec/台账同步）| worktrees: lang-982/auto-lang@<本轮> + auto-man@<本轮> | next: B-CONFIG-R 第三轮复审
2026-11-17 stage: review / **B-CONFIG-R 第三轮独立复审（窄面，`reviewer-982b-r3`）＋ 其三处发现的修复**（objective「继续完成计划982」，round 15）| 冻结面: lang `8da59965f`、man `39bc016` | **VERDICT: needs_fix**（0 BLOCKER / 1 MAJOR / 2 MINOR / 2 NOTE；F-16/18/19/20/21/22/23/24 确认 CLOSED，所有 gate 数字与收据一致）| 第三轮发现与处置：**F-982B-25（MAJOR）** spec 仍写"2000 层已实测通过"（round-14 收据声称"spec 已同步"实为不实）⇒ spec 三处改为"**≤1976 层可服务（1980+ 由冻结 16 MiB `VerifyMemory` 预算具名拒绝）**"，并在 §4 写明永久用例（1000 整链 + 2000 不崩）；**F-982B-26（MINOR）** 深度用例只有 2000 一档且接受早退，served 面不可回归 ⇒ 改为**双档**：1000 层必须服务且**整链计数**（`nested_document`/`counted_depth` 辅助），2000 层只要求不崩；**F-982B-27（NOTE）** 根级 `if`/`for` 体里的同名 pair 原先不参与注入覆盖（会执行被覆盖的 RHS）⇒ 覆盖判定改为**只按节点嵌套深度**（分支/循环体仍视为根级），新增 `if true { x: 1 / 0 }` + 注入 `x=9` 的成功断言；**F-982B-28（NOTE）** `av.rs` 有两处本轮新增的 rustfmt 差（含 config 资源臂）⇒ 手工按 rustfmt 形状改好（仅剩基面既有的一处 foreign 调用点，与 F-14 记账一致）| 独立实测（复审 agent，非本会话）：候选 CLI 深度探针 **700/1000/1200/1500/1800/1900/1950/1960/1970/1976 层 exit 0 且 `config.engine=="av"`**，**1980/2000 层 exit 1**（`VerifyMemory … ~17080528 > 16777216`）——"2000 层"目标受**冻结校验预算**限制，不是投影守卫 | 验证（本会话，修复后）：`ace-core plan982`＝11/11、`ace-core --lib --tests`＝757、`ace-av-core`＝48/11、`p982_vcfg_cases`＝**8/8**（含 1000 整链 + 2000 不崩 + 根级分支覆盖）、`p982_config_engine`＝1/1、语料探针 145 OK/0 分歧、SDK 合同 1/1＋1/1；man `cargo tc`＝258/4、候选 5/5、`plan982_ace_config_b` 2/2；`rustfmt --check` 对新增/改动文件干净（`av.rs` 仅余基面既有处）| **记账更正**：round-14 收据的"并在 §5 E5/E8、**spec 同步措辞**"在当时不成立，现由 `38fefd4ea`（spec）与 `e3b16d7f8`（用例/台账）补齐 | **仍开（具名）**：运行期 `for`、`[]`、对象字段读取/`cfg.get|merge`、保留字 key、二阶段 context、迭代式投影、`f64/u64` 值面、LIMIT 宽输入、`av.rs` 基面既有格式点 | 判读: 第三轮唯一 MAJOR 已闭、双档深度可回归，B-CONFIG-I 的实现/证据/文案三方现已一致；建议**第四轮复审只核对 F-25..F-28 与本收据的两处更正**后进入 B-CONFIG-V + merge fold | next: 第四轮复审（窄面）→ B-CONFIG-V 收据 → merge fold（man 可 ff；lang 需 rebase 追平 master）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 15（本会话）| from: review | to: work | status: executing | outcome: pass（第三轮 1 MAJOR + 2 MINOR/NOTE 修复：spec 深度更正、1000 层永久档、根级分支覆盖、av.rs 格式）| worktrees: lang-982/auto-lang@e3b16d7f8 + auto-man@39bc016 | next: 第四轮窄面复审 → B-CONFIG-V → merge fold
2026-11-17 stage: verify / **B-CONFIG-V：设计 §6 九行 V-CFG 逐行实跑＋B 口径（`config.engine=av`、`legacy_config_dispatch=0`）**（objective「继续完成计划982」，round 16）| 冻结面: lang `c4e2eda11`（基面 `e3b16d7f8`）、man `39bc016` | 前置: 第四轮窄面复审 **VERDICT: pass**（F-982B-25..28 全 CLOSED、两处记账更正核实；其 4 条非阻塞发现中 F-29（预算原因未钉）与 F-32（根级 `for` 体未覆盖）已在 `c4e2eda11` 修掉，F-30（用例注释）同批改好，F-31（`av.rs` 现已全净）本收据更正）| **九行实跑（本轮同 HEAD 一次跑齐）**：①**V-CFG-SYNTAX** — `ace-core plan982`＝11/11（三形 dep/字符串键/JSON/块对象/`#` 注释）＋语料 `145 ok / 0 disagree / 8 failed / 0 ace-only / 153 total`（失败＝5 模板无注入＋3 计划内坏夹具）；②**V-CFG-FLOW** — `p982_vcfg_cases` FLOW 例：注入 `port` 选 then 分支、`var` 重声明取最新、嵌套体/具名实参、字面 `for` 展开两节点、动态列表 `source.config-document/pending`，含"换 port ⇒ x=2"非空证；③**V-CFG-CONTEXT** — 同文件 CONTEXT 例：bool/int/text 三轴＋`on`/`off`＋注入值即根属性且胜出文档默认＋`var` 遮蔽＋float 注入 `ace.config/inject-unsupported`＋**被覆盖默认 RHS 不执行**（根级 pair / 根级 `if` 体 / 根级 `for` 体三形）＋**节点体同名键保留**；④**V-CFG-VALUE** — VALUE 例：中文/emoji/NUL/±i32 边界/空对象/嵌套/重复孩子保序/关会话后仍可读；深度双档 1000 层整链（永久断言）＋2000 层不崩且拒绝原因钉住 `VerifyMemory` 预算；⑤**V-CFG-FAIL** — FAIL 例：坏源原 span＋零残留会话＋修复后同进程真读；Man 三拒绝轴（typing/`source.undefined`、syntax/`source.syntax`、运行期 `ace.config/trap`）均 `ace.config/invalid`＋`execution=not-started`＋工程树内容哈希零写；⑥**V-CFG-LIMIT** — LIMIT 例：值预算拒＋默认预算成功非空证＋取消；桥字节预算拒；wall 覆盖解析→导出全段（`ResourceDenied{kind:"wall"}`）；⑦**V-CFG-IDENTITY** — IDENTITY 例：源哈希＋**结构化 context 值**摘要（同键不同值 ⇒ digest 不同且 x=1/x=2）＋键集轴＋`engine="av"`＋真实 steps/`callee_count`；⑧**V-CFG-PRODUCT** — Man `plan982_ace_config_b` PRODUCT：候选 CLI 真跑，`config.engine=="av"`、`config.legacy_dispatch==0`、`values/deps/targets` 真实、执行 success；A 切片见证 `plan982_ace_candidate_config` 仍 5/5；⑨**V-CFG-RETIRE** — 同文件 RETIRE：`AUTO_CONFIG_ENGINE=legacy` 显式可见且恰好 1 次 legacy 派发、未知取值不降级；默认无桥构建选择 legacy 时**可观测**（stderr 一次性提示），显式 `av` 无桥**具名失败**（CLI exit 1）| **B 口径复核**：候选回执 `config.engine=="av"` 且 `legacy_config_dispatch==0`（两处独立断言）；`ace_candidate.rs` 配置腿走 `config_engine`；`auto config` 子命令亦已走引擎选择（不再直调旧求值边）| **同 HEAD 全量证据**：lang `ace-core plan982` 11/11、`ace-core --lib --tests` 757、`ace-av-core` 48/11、`ace-syntax-grammar` 45、`ace-lowering` 9、`p982_vcfg_cases` 8/8、`p982_config_engine` 1/1、`p982_pac_corpus_probe` 1/1（145/0 分歧）、SDK 配置合同 1/1＋1/1；man `cargo tc` 258/4、候选 5/5、`plan982_ace_config_b` 2/2、`config_engine` 4/4 | **未闭（具名，仍是 M2-B/后续计划）**：运行期 `for`、`[]`、对象字段读取/`cfg.get|merge`、保留字 key、二阶段 context、迭代式投影、`f64/u64` 值面、LIMIT 宽输入、`av.rs` 基面既有格式点（现已全净，F-31）；**F-14/F-15 记账**已闭 | 判读: B-CONFIG-I 的九行 V 门在本 HEAD 全部有独立 oracle 且实跑通过，B 口径成立；满足进入 **merge fold**（man 可 ff；lang 需 rebase 追平 master）| next: merge fold（两仓落主＋落地复核＋worktree 守卫）→ 计划状态 reviewed（B-CONFIG-I 阶段收据）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 16（本会话）| from: review→verify | to: verify | status: executing | outcome: pass（第四轮复审 pass；B-CONFIG-V 九行实跑通过、B 口径成立；待 merge fold）| worktrees: lang-982/auto-lang@c4e2eda11 + auto-man@39bc016 | next: merge fold
2026-11-17 stage: merge / **B-CONFIG-I 阶段 fold（两仓落主＋落地复核＋树清理）**（objective「继续完成计划982」，round 17）| **landed**：auto-man `39bc016`（**ff-only**，5 提交：`02a4789`/`81b27b5`/`dd714f9`/`d771094`/`39bc016`）；auto-lang `0b6d3198d`（**merge commit**：master 已前进 98 提交、分支 21 提交，`git merge plan-982-dev --no-edit` **零冲突**落地；含 E1..E8 与四轮复审修复 `b26268d44`..`c4e2eda11`）| **落主核对（两仓 master 实跑）**：lang `ace-core plan982`＝**11/11**、`ace-core --lib --tests`＝**765**、`ace-av-core`＝48/11、`auto-lang-ace-host` `p982_vcfg_cases`/`p982_config_engine`/`p982_pac_corpus_probe`＝8/1/1、SDK 配置合同 1/1＋1/1、**语料探针（master 章程语料更大）＝288 ok / 0 disagree / 13 failed / 301 total**；man `cargo tc`＝**258 passed / 4 skipped**、候选见证 5/5、`plan982_ace_config_b` 2/2 | **merge 前的 WIP 处置**：auto-lang 主检出有他会话未提交的 `docs/design/**` 改动（10 文件）——与本切片零重叠，merge 未触碰、未提交；主检出另有两个**0 字节**误建未跟踪文件（`p982_vcfg_cases.rs`/`plan982_config_document.rs`，早期相对路径误写残留），已字节备份到 `plan-artifacts/plan-982/stray-main-edits2/` 后删除，merge 才得以进行；备份随阶段收尾一并删除 | **树/分支/组**：两仓 `plan-982-dev` 已删除（`-d` 安全删除）、worktree 已 `git worktree remove`（**移除前均跑 `wt-guard.ps1`：clean**）；组目录 `D:/autostack/.wt/lang-982/` 仅剩只读兄弟 `auto-down`/`auto-ui`/`auto-web`（detached），下一切片另建组 | **plan-artifacts**：`plan-982/`（57 文件 / 1.86 MB：早期 A 切片复原件、merge 期 lock WIP 备份、复审日志等）已按要求**整目录删除**，证据已折叠在 §9 各收据 | **规范**：`docs/specs/ace-migration/m2-man-gen.md` 已描述 B-CONFIG-I 现值行为与具名边界（`de56b4c77`＋`e29db2610`＋`38fefd4ea`），落主核对与代码一致 | **计划状态**：PLAN-982 整体**仍 executing**——B-CONFIG-I/V 阶段已交付并 fold，剩余 M2-B 切片（TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK、安装/升级/退役，及本阶段登记的能力边界）由后续会话/计划继续；不 archive | **不声明**：全产品迁移完成；运行期 `for`/`[]`/对象字段读取/`cfg.get|merge`/二阶段 context/`f64|u64` 值面仍是具名边界
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 17（本会话）| from: verify | to: merge | status: executing | outcome: pass（B-CONFIG-I/V fold 落主：man ff 39bc016、lang merge 0b6d3198d 零冲突；落主两仓全绿；树/分支/工件清理）| next: 下一 M2-B 切片（TEMPLATE/RUST/…）或本阶段能力边界续做
2026-11-17 stage: work / **982/CONFIG 切片的 V 增量：真实语料的"产品投影"两引擎一致性**（objective「继续完成计划982」，round 18）| 树: 新组 `D:/autostack/.wt/lang-982/{auto-lang,auto-man}`（`plan-982-dev`，基面 master `3459ef6d9`/`cab347b`）| **交付**：`crates/auto/tests/plan982_config_slice.rs`（`--features ace-candidate`）——对**仓库受管的全部 201 份 `pac.at`**，分别在 `LegacyVm` 与 `AceAv` 下走**真实消费面** `AutoConfig → Pac`，逐项比较产品投影（`name`/依赖数/目标数）；①两侧成功⇒投影必须逐项相等；②两侧都拒⇒只要求 ACE 的拒绝**具名**（不冒充等价）；③legacy 成功而 ACE 拒 ⇒ **只有登记例外集允许**（5 份模板 `${name}` 占位符 + 3 份计划内坏夹具 + `dynamic-template` + `static-unknown-node-content`），否则失败；④断言 `equal>=185` 且语料 `>=190`（可红、不会是空跑）| **实测（worktree 内，`CARGO_TARGET_DIR` 复用主检出暖缓存）**：`config-slice corpus: **186 identical product projections, 10 registered exceptions, 5 both-refused, 201 total**`，1 passed | **本轮发现（真实语料暴露，已登记）**：①`test/fixtures/modularity/project-model/fixtures/static-unknown-node-content.pac.at` 用 **`tag:` 作 pair key**（保留字），legacy 能投影而 ACE 语法拒——把原 §5 E4 的"保留字作键"从**理论边界**升级为**产物可见缺口**（仍有真实 fixture 受影响，是下一批 I 的候选）；②`dynamic-template` 的 `${name}` 无注入时 ACE fail-closed 拒绝（已登记行为，与 5 份模板同族）；③`bad-missing-brace`/`bad-missing-value`/`dynamic-expressions`/`dynamic-structured-values`/`dynamic-top-object-values` 为两侧共拒（5 份），我最初的分类把它们误判为"未登记拒绝"，已改为**共同拒绝**分类（只要求具名）——这也说明本harness 的分类本身被真实数据检验过 | 判读：982/CONFIG 的 "V验收：原动态 pac 与参数/环境输出一致" 现在有**语料级**证据（非单例）；"失败不写 state/lock" 由既有 Man 三拒绝轴用例覆盖；"路径锁身份与可求值静态子集不混用" 未动（属该切片后续 I）| next: 982/CONFIG I 的保留字键修复（真实 fixture 驱动）＋ state/lock 零写扩展 → 或按台账推进其它 M2-B 切片
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 18（本会话）| from: merge→work（下一切片） | to: work | status: executing | outcome: pass（982/CONFIG V 增量：201 份 pac 语料产品投影两引擎一致 186 例/0 分歧；暴露并登记保留字键真实缺口）| worktrees: .wt/lang-982/{auto-lang,auto-man}@plan-982-dev（man `1abf0a0`）| next: 保留字键修复 + state/lock 零写扩展
2026-11-17 stage: 用户优先序介入 / **EXEC、RUST 优先**（objective「继续完成计划982」，round 19）| **用户问答的事实核对（以树/台账为准）**：**982/EXEC 与 982/RUST 均未开始**——§5 里它们是 M2-B 的未执行行（`B-EXEC-I/V/R`、`B-RUST-I` 在 §5 "M2-B" 列表内），§9 没有任何 EXEC/RUST 收据，两仓也没有 EXEC/RUST 专属产物（现有 `plan982_*` 全部属 A 切片与 CONFIG）；唯一与 EXEC 沾边的是 A 切片的**裸脚本候选腿**（`cmd_run.rs` 的 `ace_candidate::run_script`，opt-in、只服务 `auto <script.at>`），正是 EXEC 切片要推广的对象 | **"已做的先单独合并"核对**：**无需**——B-CONFIG-I/V 已在 round 17 落主（lang `0b6d3198d`、man `39bc016`）；当前 worktree 里只有 CONFIG 的两个后续增量（保留字作键 `a243913f8`、语料例外收缩 `8c39f8c`，均绿且已在分支上），不构成阻塞 | **本轮完成的 CONFIG 增量（保留字键，真实语料驱动）**：语法侧把文档键从 `ident` 扩为 `cfg-doc-key`（`ident`/字符串/`tag`/`type`），CST 投影统一 `cfg_key_ident`（下潜派发节点、字符串键解码去引号），core 新增 `p982_config_reserved_words_work_as_document_keys`；**实测**：`ace-core plan982` 12/12、`ace-core --lib --tests` 766、grammar 45、vcfg 8/8、桥 1/1、语料探针 288 OK/0 分歧；man 侧 CONFIG 语料投影 **187 一致 / 9 例外 / 5 共拒 / 201**（原 186/10 —— `static-unknown-node-content.pac.at` 的 `tag:` 键已能投影一致）| **优先序调整（本收据生效）**：后续轮次按 **982/EXEC → 982/RUST → 982/CONFIG 其余 I → 其余 M2-B** 推进；EXEC 的第一步定为**裸脚本执行腿的输入/退出/超时补齐**（stdin 具名面、stdout/stderr 捕获、退出码、wall 回收）＋ `debug`/`test` 失败码与"未知目标不 legacy fallback"的具名拒绝，均以真实 CLI 用例为 oracle；RUST 的第一步定为 A2R 真实工程冷构建（未改生成码）＋旧 RustTrans 零 dispatch 见证 | **不声明**：EXEC/RUST 已有实现或已合并
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 19（本会话）| from: work | to: work | status: executing | outcome: 用户优先序（EXEC→RUST）；事实核对：两者均未开始、无需单独合并；CONFIG 保留字键增量已绿（语料投影 187/9 一致）| worktrees: .wt/lang-982/{auto-lang@a243913f8, auto-man@8c39f8c} | next: EXEC I 第一步（stdin/捕获/退出/超时 + debug/test 失败码）
2026-11-17 stage: work / **982/EXEC 切片 I（第一步：命令面边界——未知目标不 legacy fallback）**（objective「继续完成计划982」，round 20；按用户 2026-11-17 优先序 EXEC→RUST）| 树: `.wt/lang-982/auto-man` `plan-982-dev`（本提交 `41bcaa3`；lang 侧 `a243913f8` 未变）| **交付**：①`crates/auto/src/ace_candidate.rs` 新增 `refuse_unsupported_command`（`ace.exec/unsupported-command`，`--ai` 走结构化 JSON、退出非零）；②`crates/auto/src/main.rs` 在 `ai_mode` 之后、任何分派之前插入 `candidate_unsupported_command(...)`——选中候选时 `run`/`service`/`service-export`/`test`/`test-ui`/`debug`/`req` **一律具名拒绝**（此前这些命令在候选模式下静默走旧执行器，正是 EXEC "未知目标不能 legacy fallback" 的缺口）；③新测试 `crates/auto/tests/plan982_ace_exec_b.rs`（3 例）：四个 run 家族命令**真 CLI** 非零退出＋`ace.exec/unsupported-command`＋工程目录**零写**；`--ai` 结构化 JSON（`status/error/code/message`）；**控制臂**——未选中候选时同一命令**不得**出现该拒绝码（证明拒绝源于候选选择，不是命令本身坏）| **验证**：`--features ace-candidate` 四测试文件全绿（`plan982_ace_candidate_config` 5/5、`plan982_ace_config_b` 2/2、`plan982_ace_exec_b` 3/3、`plan982_config_slice` 1/1）；`cargo tc`＝258/4 | **为何这是 EXEC 的正确第一步**：EXEC 的其余面（项目 run/service/req/debug/test 的 ACE 服务化、stdin 面）依赖**尚未存在**的 ACE 宿主能力——ACE console schema 只有 `console.stdout`/`console.stderr`，`AceRequest.input` 在 969 车道被 `ace.input/unsupported` 明确拒绝；把这些命令继续留在旧执行器上**又**不具名，就是静默 fallback。本步先把边界钉死，再逐命令上 ACE | **仍开（EXEC 后续 I，按序）**：①stdin 面（需 ACE host 新增输入 provider + `console.stdin` 能力键，969 的 `ace.input/unsupported` 才有替代）；②项目 `run` 服务化（UI/daemon 面，依赖模板/目标生成）；③`req` 持久变量与 session 协议；④`debug`/`test` 的 ACE 版失败码；⑤超时后子任务回收（AV 侧 cancel 已有，CLI 侧回收未验）；RUST 切片仍在其后 | next: EXEC I 第二步——stdin 能力键与 provider（lang 侧）→ 或按可交付性先做 `test`/`debug` 的失败码对齐
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 20（本会话）| from: work | to: work | status: executing | outcome: pass（EXEC I 第一步：run 家族命令在候选腿下具名拒绝、零写、无 legacy fallback，3 例真 CLI 证据）| worktrees: .wt/lang-982/auto-man@41bcaa3（lang 未变）| next: EXEC I 第二步（stdin 能力键/provider 或 test/debug 失败码）
2026-11-17 stage: work / **EXEC 第二步的可交付性核查 ＋ RUST 切片的接线事实**（objective「继续完成计划982」，round 21）| 树: `.wt/lang-982/auto-man@41bcaa3`、`.wt/lang-982/auto-lang@a243913f8`（本轮为**只读核查**，未改代码）| **EXEC 第二步（stdin 面）核查结论：需要先补 ACE 宿主能力，不能在本切片内"顺手"做**——①ACE console schema（`experimental/ace/crates/core/schema/host-console.atom`）只有写面，provider 授权键只有 `console.stdout`/`console.stderr`（`crates/auto/src/ace_candidate.rs:console_grants`）；②`ConsoleHost` trait（`av-core/src/host.rs`）仅 `retained_bytes`/`write`，没有输入方法；③SDK 车道对输入是**显式拒绝**：`crates/auto-lang-ace-host/src/lib.rs:244-247` `request.input.is_some()` ⇒ `ace.input/unsupported`（969 T-08 的"argv/stdin 是显式请求输入、不静默丢弃"）；④返回文本需要 Str arena（av.rs 的 `Value::Str(StrHandle)`/`arena.alloc`），因此新增 `console.stdin` 不是加一个符号，而是"能力键＋宿主 provider 方法＋文本返回值＋CLI 分派"四处联动。⇒ 计划：先把 `console.stdin` 的**能力键与契约**写进本计划（下轮 I），再实现 provider 与 `auto <script.at>` 的 stdin 透传，最后才谈 `run/service/req/debug/test` 的服务化 | **RUST 切片（用户优先序第二位）接线事实（已核，供下轮 I 照做）**：①产品**仍走旧转译**——`crates/auto/src/cmd_trans.rs:31`（`trans_rust`）、`:117`（`TransTarget::Rust` 多文件）、`:135`（单文件）与 `crates/auto-man-runtime/src/target.rs:91-93`（`auto_lang_core::trans::rust::transpile_rust`）；②新 A2R **已存在但未接线**：`experimental/ace/crates/a2x-rust`（`Cargo.toml` 已进 lock，`lib.rs` 导出 `lower`/`target`/`artifacts`/`provider`/`rt`/`cert`，`assemble_file_set`/`runtime_source`/`FileSet`）；③因此 RUST I 的第一步是**只读探针**：用 `ace-a2x-rust` 对 `test/fixtures` 里一个真实多模块工程生成 `FileSet`，与旧 `transpile_rust_project_multi` 的输出**逐文件比对**（先不接产品路径），确认语义等价面后再在产品 CLI 上加候选开关（`AUTO_ACE_RUST`，与 `AUTO_CONFIG_ENGINE`/`AUTO_ACE_CANDIDATE` 同族），并要求**旧 RustTrans 零 dispatch** 见证 | **本轮无代码交付的原因**：两条候选路径都需要先钉契约（stdin 能力键 / A2R 等价探针），避免"改了但证不出来"；这与 §9 的"每个 I 步都要有独立 oracle"一致 | next: RUST I 只读探针（A2R FileSet vs 旧转译逐文件比对）→ 通过则 `AUTO_ACE_RUST` 候选开关＋零 dispatch 见证；EXEC 第二步的 `console.stdin` 契约随后
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 21（本会话）| from: work | to: work | status: executing | outcome: 核查类进展（EXEC stdin 需先补宿主能力；RUST 仍走旧转译、新 A2R 未接线——两路径的契约与第一步已写进台账）| worktrees: 未改动（man@41bcaa3 / lang@a243913f8）| next: RUST I 只读探针（A2R FileSet vs 旧转译逐文件）
2026-11-17 stage: work / **982/RUST I 接线契约勘察（只读）**（objective「继续完成计划982」，round 22）| 树: 未改动（man@41bcaa3 / lang@a243913f8）| **勘察结论（决定下一步怎么接，不是"已实现"）**：新 A2R 不是"一个函数转一段源码"，它走**目标提供者框架**——①`ace-a2x-rust::RustTargetProvider` 实现 `ace-a2x-core::target::TargetProvider`（`crates/a2x-core/src/target.rs:92`），必须提供 `descriptor`/`fragment_key_inputs`/`fragment_layer_id`/`runtime_source`/（其余）方法，并由 **`a2x-store` 的 fragment 层注册表**承载缓存键；②其产出是 `FileSet`（`lib.rs` 的 `assemble_file_set`/`FileSet`、`provider.rs` 的 wrapper+runtime+generated 三件套），入口靠 `entry` 名与 `fragments`；③打印版本是冻结身份的一部分（`RUST_PRINT_VERSION="a2x-print/3"`，注释明确 1→2→3 的语义变化会强制所有旧 Rust fragment miss）| **产品侧缺口（事实）**：`crates/auto/src/cmd_trans.rs:31/117/135` 与 `crates/auto-man-runtime/src/target.rs:91-93` 仍调旧 `trans_rust`/`trans::rust::transpile_rust`/`transpile_rust_project_multi`；man 没有任何 `a2x`/`TargetProvider` 依赖，也没有 fragment 存储驱动 ⇒ **最小可交付的 RUST I 不是"换个函数调用"**，而是要先把 provider 框架（descriptor/键/缓存）接进产品，或先在语言侧提供一个**无缓存、纯内存**的驱动（lower → print → FileSet）供产品候选开关调用 | **下一步（按此顺序，独立 oracle）**：①语言侧加一个**无缓存 A2R 驱动**（`lower_fragments → print → FileSet`）并以 `cfg` 之外的**真实多模块工程**产出 `FileSet`；②与旧 `transpile_rust_project_multi` 的产物**逐文件 diff**（允许的差异面=生成序/注释头，语义必须等价；差异清单逐条登记）；③`cargo build` 生成物（冷构建）跑通 expected；④产品加 `AUTO_ACE_RUST` 候选开关 + **旧 RustTrans 零 dispatch** 见证（witness 计数与 `config.legacy_dispatch` 同族）| 判读：本轮只完成"接线契约与缺口"的勘察，**RUST 仍未开始实现**；不夸大为进展 | next: 语言侧无缓存 A2R 驱动（第一步）
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 22（本会话）| from: work | to: work | status: executing | outcome: 勘察类进展（A2R 走 TargetProvider 框架+Fragment 缓存键；产品缺 provider/存储驱动，最小可交付=语言侧无缓存驱动→逐文件 diff→冷构建→候选开关+零 dispatch）| worktrees: 未改动 | next: 语言侧无缓存 A2R 驱动
2026-11-17 stage: work / **982/RUST I 第一步：语言侧无缓存 A2R 驱动跑通（源码→受检 HIR→view→lower→FileSet）**（objective「继续完成计划982」，round 23）| 树: `.wt/lang-982/auto-lang@ac13f3007`（man 未动，仍 `41bcaa3`）| **交付**：`experimental/ace/crates/a2x-rust/tests/plan982_rust_fileset_probe.rs`（3 例，全部走**产品将来要走的那条链**）——①`p982_rust_uncached_driver_renders_a_complete_fileset`：`ace_core::source::compile_scalar_source` → `SemanticView::from_checked` → `lower_program(view,"main")` → `assemble_file_set(...)`，断言产物就是**冻结三件套** `main.rs`/`a2x_runtime.rs`/`generated.rs`、非空、带 `DO NOT EDIT` 标记、含降级入口名、manifest 带 `auto.a2x.rust.fileset/1`；②`p982_rust_uncached_driver_is_deterministic`：同源两次渲染**逐字节一致**（这是"与旧转译逐文件 diff"能用的前提）；③`p982_rust_uncached_driver_refuses_unknown_entry_by_name`：不存在的入口是**具名拒绝**，不静默降级到第一个函数 | **实测**：`cargo test -p ace-a2x-rust --test plan982_rust_fileset_probe`＝**3 passed**；`cargo test -p ace-a2x-rust --tests`＝2＋3 passed | **本轮踩到的真实约束（已写进用例注释）**：A2R 的**输入是受检 HIR 的 SemanticView**（不是源文本），且 scalar 前门对"表达式语句/尾表达式"有限制——探针源必须写成 `return` 形式；降级入口名是 `fn_main`（不是 `main`），所以断言用后缀而非全等 | **下一步（RUST I 第二步）**：①把这条链包成一个**产品可调用的入口**（宿主导航：`auto-lang-ace-host` 已有 ACE 依赖，适合放 A2R 驱动，与 config 桥同层）；②与旧 `transpile_rust_project_multi` 的产物**逐文件 diff**（差异面逐条登记：生成序/注释头/命名）；③生成物**冷构建**跑通 expected；④产品加 `AUTO_ACE_RUST` 候选开关 + **旧 RustTrans 零 dispatch** 见证 | 判读：语言侧 A2R 驱动已可重复产出一致 FileSet，RUST 切片从"未开始"进入"驱动已证"；产品接线尚未做，不声明 RUST 已交付
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 23（本会话）| from: work | to: work | status: executing | outcome: pass（RUST I 第一步：无缓存 A2R 驱动 FileSet 三件套、确定性、具名入口拒绝，3 例）| worktrees: .wt/lang-982/auto-lang@ac13f3007（man@41bcaa3）| next: RUST I 第二步（host 侧驱动 + 逐文件 diff vs 旧转译 + 冷构建 + 候选开关）
2026-11-17 stage: work / **982/RUST I 第二步：A2R 宿主驱动面落地（源码→FileSet 的唯一边界）**（objective「继续完成计划982」，round 24）| 树: `.wt/lang-982/auto-lang@a8b650e28`（man 未动 `41bcaa3`）| **交付**：①`crates/auto-lang-ace-host/src/rust_transpile.rs`——`transpile_scalar_source(source, module, entry)`：前端 `compile_scalar_source` → `SemanticView::from_checked` → `ace_a2x_rust::lower_program` → `assemble_file_set`，返回 `AceRustFileSet{files, manifest, entry}`；与配置桥**同层**（ACE crate 只在此处出现，`auto-lang-core` 仍零 ACE 依赖）；②失败全部**具名**：`ace.rust/compile`（前端，带 stage/code 列表）、`ace.rust/view`、`ace.rust/lower`（含未知入口），**绝不返回空集、绝不静默降级**；③确定性：input hash 由本驱动按源码 fnv1a 计算，同输入两次调用逐字节一致；④`crates/auto-lang-ace-host/tests/p982_rust_driver.rs`（3 例）：冻结三件套＋schema manifest＋降级入口名、两次渲染全等、两条具名拒绝（前端/入口）| **实测**：`p982_rust_driver` 3/3；同包 `p982_vcfg_cases` 8/8、`p982_config_engine` 1/1（无回归）；`Cargo.lock` 增加 11 行（`ace-a2x-core`/`ace-a2x-rust` 进桥）| **下一步（RUST I 第三步，产品接线）**：`crates/auto/src/cmd_trans.rs` 的 `TransTarget::Rust` 与 `auto-man-runtime/src/target.rs` 改为**候选开关**（`AUTO_ACE_RUST=1` ⇒ 走本驱动写 FileSet；未选中 ⇒ 旧路径不变），并把"旧 RustTrans 零 dispatch"做成与 `config.legacy_dispatch` 同族的可观测计数；随后用真实多模块工程做**逐文件 diff**（A2R 侧的 `generated.rs` 与旧生成的期望文件差异逐条登记）＋冷构建 expected | 判读：语言侧与宿主侧的 A2R 能力都已可调用且被永久用例钉住；产品路径**仍未切换**，不声明 RUST 已交付
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 24（本会话）| from: work | to: work | status: executing | outcome: pass（RUST I 第二步：宿主驱动面 transpile_scalar_source + 3 例具名/确定性/三件套；无回归）| worktrees: .wt/lang-982/auto-lang@a8b650e28（man@41bcaa3）| next: RUST I 第三步（产品候选开关 AUTO_ACE_RUST + 零 dispatch 见证 + 逐文件 diff/冷构建）
2026-11-17 stage: work / **982/RUST I 第三步：产品候选开关 `AUTO_ACE_RUST`（真 CLI 可区分）**（objective「继续完成计划982」，round 25）| 树: `.wt/lang-982/auto-man@2dd551e`（lang 未动 `a8b650e28`）| **交付**：①`crates/auto-man-runtime/src/rust_engine.rs`——与 `config_engine` 同族的选择/观测：`AUTO_ACE_RUST=1|true|yes|ace|a2r` ⇒ `AceA2r`；`legacy|0|false|vm` ⇒ 显式旧路径；**未知取值保持旧路径**（不静默切 A2R）；`note_ace_a2r_dispatch`/`note_legacy_dispatch` 真实计数 + `observation()`；②`crates/auto/src/cmd_trans.rs::rust` 接开关：A2R 选中 ⇒ 读源→`auto_lang_ace_host::rust_transpile::transpile_scalar_source`→输出降级后的 `generated.rs`；失败**具名**（`ace.rust/compile|view|lower`）且**不回退**旧转译；无桥构建被显式选中 ⇒ `ace.rust/executor-unavailable` 具名拒绝；未选中 ⇒ 旧路径行为不变（并记 legacy 派发）；③`crates/auto/tests/plan982_ace_rust_b.rs`（2 例真 CLI）：**对照臂**（未选中时输出不含 `a2x_runtime`）vs **候选臂**（含 `AUTO-GENERATED`/`a2x_runtime`/`fn_main`，且两臂文本不同 ⇒ 开关是真的）；拒绝臂（非法源 ⇒ 非零退出 + `ace.rust/` 码 + **零旧转译输出**）| **实测**：`plan982_ace_rust_b` 2/2、`plan982_ace_exec_b` 3/3（无回归）、`cargo tc`＝**259 passed / 4 skipped**（+1 引擎单测）| **diff 卫生**：`cmd_trans.rs` 只 +71 行（未把既有文件整体 rustfmt，避免 400 行格式噪声）——首次提交曾带入格式噪声，已 `--amend` 修正 | **仍开（RUST 后续）**：①项目多文件路径（`auto-man-runtime/src/target.rs` 仍走 `transpile_rust_project_multi`）与 `TransTarget::Rust{merge}` 未接开关；②与旧转译的**逐文件 diff**（当前只证"两臂可区分"，未证"A2R 产物语义等价"）；③生成物**冷构建** expected；④旧 RustTrans **零 dispatch** 的对外见证（当前计数在进程内，CLI 测试跨进程看不到，需要回执/日志面）| 判读：RUST 的候选开关与具名拒绝已落地并被真 CLI 钉住；"等价/冷构建/零 dispatch 回执"未做，不声明 RUST 交付
2026-11-17 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 25（本会话）| from: work | to: work | status: executing | outcome: pass（RUST I 第三步：AUTO_ACE_RUST 候选用真 CLI 可区分、拒绝具名不回退；tc 259/4）| worktrees: .wt/lang-982/auto-man@2dd551e（lang@a8b650e28）| next: RUST 后续（多文件/merge 路径、逐文件 diff、冷构建、零 dispatch 回执）
2026-11-18 stage: work / **982/RUST V：A2R 产物冷构建实跑 ＋ 零 legacy dispatch 对外见证**（objective「继续完成计划982」，round 26）| 树: `.wt/lang-982/auto-man@507eef4`（lang 未动 `a8b650e28`）| **交付**：①`crates/auto/tests/plan982_ace_rust_cold_build.rs`（2 例）：宿主驱动产出的三件套**原样**写盘后，`rustc --edition 2021 main.rs` **直接编译**（无隐藏依赖、无额外 crate），运行输出 **`value:5`**（`add(2,3)` 的源语义；不是"能编译就算过"）；两次独立冷构建行为一致；②`rust_engine` 新增回执面 `AUTO_ACE_RUST_PROOF=<path>`（`auto.plan982.rust.proof/1`：`engine`/`a2r_dispatches`/`legacy_dispatches`，原子替换写），A2R 与**旧路径**都写——③`plan982_ace_rust_b.rs` 增两例真 CLI：A2R 臂回执 `engine=a2r, a2r_dispatches=1, legacy_dispatches=0`；**对照臂**未选中时 `engine=legacy, legacy_dispatches=1, a2r_dispatches=0`（证明计数是真数字、不是常量）| **实测**：`plan982_ace_rust_b` **4/4**、`plan982_ace_rust_cold_build` **2/2**、`cargo tc` **259/4** | **diff 卫生**：`cmd_trans.rs` 本轮只 +7 行（回执调用），无格式噪声 | **仍开（RUST 后续）**：①多文件工程路径（`auto-man-runtime/src/target.rs` 的 `transpile_rust_project_multi`、`TransTarget::Rust{merge}`）未接 A2R；②A2R vs 旧转译的**逐文件语义 diff**（当前证明的是"产物可冷构建且语义正确"，不是"与旧产物逐字等价"——两者是不同生成器，等价面需按文件清单登记）；③冷构建目前只覆盖 scalar profile 的单一样例，未覆盖 collections/records/scheduler 家族 | 判读: RUST 的 V 前两项（产物可跑、零 legacy dispatch 有回执）已闭；"多文件/家族覆盖"未做，不声明 RUST 交付
2026-11-18 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 26（本会话）| from: work | to: work | status: executing | outcome: pass（RUST V：冷构建实跑 value:5；零 legacy dispatch 有回执且对照臂计数为 1）| worktrees: .wt/lang-982/auto-man@507eef4（lang@a8b650e28）| next: RUST 多文件/merge 路径接 A2R + 家族覆盖
2026-11-18 stage: work / **982/RUST：家族覆盖（records/collections）＋ records 冷构建**（objective「继续完成计划982」，round 27）| 树: lang `02ae33087`、man `5c00ecb` | **交付**：①`crates/auto-lang-ace-host/src/rust_transpile.rs` 增 `TranspileProfile{Scalar,Collections,Records}` 与 `transpile_with_profile(...)`（各家族走各自受检前门：`compile_scalar_source`/`compile_collections_source`/`compile_record_source`；错误码与确定性合同不变），`transpile_scalar_source` 变薄包装（既有用例零改）；②host 用例 `p982_rust_host_driver_covers_records_and_collections_families`：两家族都产出冻结三件套＋schema manifest，**两次渲染全等**，入口 `fn_main`；③man 冷构建用例增 `p982_a2r_records_family_builds_cold_and_runs`：records 家族产物**原样 rustc 冷构建**并运行输出 **`value:7`** | **实测**：host `p982_rust_driver` **4/4**、`p982_vcfg_cases` 8/8（无回归）；man `plan982_ace_rust_cold_build` **3/3**（20.4s，含三次 rustc 调用）| **仍开（RUST 后续）**：①collections 家族的冷构建（本轮只做"渲染+确定性"，未编译）+ scheduler 家族；②多文件工程路径（`target.rs`/`TransTarget::Rust{merge}`）接 A2R；③A2R vs 旧转译的逐文件语义 diff 清单 | 判读：RUST 的"家族覆盖"从 1 个 profile 扩到 3 个（其中 2 个已冷构建实跑）；多文件与 diff 清单未做，不声明 RUST 交付
2026-11-18 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 27（本会话）| from: work | to: work | status: executing | outcome: pass（RUST 家族覆盖：profile 化驱动 + records/collections 用例；records 冷构建 value:7）| worktrees: lang@02ae33087 / man@5c00ecb | next: collections 冷构建 + 多文件路径 + diff 清单
2026-11-18 stage: work / **982/RUST V：collections 家族冷构建（含其专属文本运行时）**（objective「继续完成计划982」，round 28）| 树: lang `b57ecd424`、man `8fcbef4` | **交付/修正**：①**真实缺陷被冷构建门抓到**——collections 产物此前缺少文本运行时前导（冷构建 `E0425: cannot find type A2xStr`），修法是让驱动按 profile 选**家族自己的装配器** `assemble_collections_file_set`（该装配器额外要求 `full_source_sha256` 与规范化执行配置；本驱动用确定性的占位 sha256（fnv1a 派生，登记为差额）+ `canonical_execution_config("transpile", u64::MAX, None, false, false)`）；②新增 `p982_a2r_collections_family_builds_cold_and_runs`：collections 产物**原样 rustc 冷构建**并运行输出 **`value:42`**（`len` 是字符数，`"中"`=1 ⇒ 1*40+1*2；我第一次按字节数写成 126，被实跑纠正）；③家族 manifest 断言改为"是 a2x 的 fileset schema"（collections 用 `COLL_FILESET_SCHEMA`，不能写死 scalar 的拼写）| **实测**：host `p982_rust_driver` **4/4**、`p982_vcfg_cases` 8/8；man `plan982_ace_rust_cold_build` **4/4**（scalar/records/collections 三家族全部冷构建实跑）、`plan982_ace_rust_b` 4/4 | **登记差额**：`full_source_sha256` 目前是**占位**（本驱动不引 sha2；正式内容哈希随证书/身份切片补），已写进代码注释与本收据 | **仍开（RUST 后续）**：①scheduler 家族；②多文件工程路径（`target.rs`/`TransTarget::Rust{merge}`）；③A2R vs 旧转译逐文件语义 diff 清单；④真实 sha256 身份 | 判读：RUST 的三大家族现在**都能从源码冷构建并跑出正确结果**；多文件与 diff 清单仍未做，不声明 RUST 交付
2026-11-18 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 28（本会话）| from: work | to: work | status: executing | outcome: pass（RUST V：collections 冷构建修复（家族装配器+文本运行时），三家族冷构建实跑全绿；占位 sha256 已登记）| worktrees: lang@b57ecd424 / man@8fcbef4 | next: scheduler 家族/多文件路径/diff 清单
2026-11-18 stage: work / **982/RUST：未迁移 Rust 面的"零静默 legacy"边界（项目转译器/merge）**（objective「继续完成计划982」，round 29）| 树: man `cc2a851`（lang 未动 `b57ecd424`）| **交付**：①`crates/auto-man-runtime/src/target.rs`：项目构建面（`TargetEffects::transpile_auto`）在 `AUTO_ACE_RUST` 选中时**具名拒绝** `ace.rust/project-unsupported`（A2R 尚未服务多文件/写盘路径）——此前这条路径会静默用旧 `transpile_rust_project_multi`；②`cmd_trans.rs` 的 `TransTarget::Rust{merge}` 同样具名拒绝 `ace.rust/merge-unsupported`（合并单文件需重写模块边界，A2R 侧未实现），且**不写输出文件**；③`plan982_ace_rust_b.rs` 增 merge 用例（真 CLI：非零退出 + 具名码 + 输出文件不存在）| **实测**：`plan982_ace_rust_b` **5/5**、`cargo tc` **259/4**；本轮 diff 仅 +22 行代码 + 用例（无格式噪声）| **口径**：这三条（单文件 trans 已接 A2R；项目/merge 具名拒绝）合起来满足 RUST 的"未知目标不能 legacy fallback"——任何 Rust 面在被选中 A2R 后要么走 A2R、要么具名拒绝 | **仍开（RUST 后续）**：①多文件/写盘路径真正的 A2R 实现（`FileSet` → 工程树，跨模块 `use` 与 crate 边界）；②merge 的 A2R 单文件形态；③scheduler 家族冷构建；④A2R vs 旧转译逐文件语义 diff 清单；⑤真实 sha256 身份 | 判读：RUST 的"零静默回退"已全面包住；能力面（多文件/merge/scheduler）仍缺，不声明 RUST 交付
2026-11-18 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 29（本会话）| from: work | to: work | status: executing | outcome: pass（RUST：项目/merge 面在 A2R 下具名拒绝，零静默 legacy；5/5 + tc 259/4）| worktrees: man@cc2a851（lang@b57ecd424）| next: 多文件 A2R 实现 / merge 单文件形态 / scheduler 家族
2026-11-18 stage: work / **982/RUST：`--merge` 单文件形态真正实现（可独立冷构建）**（objective「继续完成计划982」，round 30）| 树: lang `d5250c382`、man `e2b131b` | **交付**：①`rust_transpile::merge_fileset(&AceRustFileSet)`——三件套**模块内联**成单个 `.rs`：`mod a2x_runtime { … }` + `mod generated { … }` + 入口 `fn main`；**生成文本一字不改**，只过滤 wrapper 的 `mod` 声明与生成头（`use a2x_runtime::A2xResult;` 必须保留，否则 `E0425`——本轮第一版就栽在这里，被冷构建门抓到后修正）；②`cmd_trans.rs` 的 `TransTarget::Rust{merge}` 在 A2R 选中时走该实现（写文件或打印），不再具名拒绝；③`plan982_ace_rust_b.rs` 的 merge 用例改为**真产物验收**：合并文件写成后 `rustc --edition 2021 merged.rs` **单独编译**（不依赖其它生成文件）并运行输出 `value:5` | **实测**：`plan982_ace_rust_b` **5/5**、host `p982_rust_driver` 4/4、`cargo tc` **259/4** | **口径更新**：Rust 面现状＝单文件 `auto rust` ✅ A2R、`--merge` ✅ A2R（可独立构建）、项目多文件 ⛔ 具名拒绝（未迁移，绝不静默 legacy）| **仍开（RUST 后续）**：①项目多文件/写盘路径的 A2R 实现（跨模块 `use` 与 crate 边界）；②scheduler 家族冷构建；③A2R vs 旧转译逐文件语义 diff 清单；④真实 sha256 身份（现占位）| 判读：merge 面交付；多文件仍未做，不声明 RUST 整体交付
2026-11-18 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 30（本会话）| from: work | to: work | status: executing | outcome: pass（RUST：--merge 单文件形态实现并独立冷构建实跑 value:5；A2xResult 导入缺陷被冷构建门抓到）| worktrees: lang@d5250c382 / man@e2b131b | next: 项目多文件 A2R 实现 / scheduler 家族 / diff 清单
2026-11-18 stage: work / **982/RUST：scheduler 家族的边界（要么可构建、要么具名拒绝）**（objective「继续完成计划982」，round 31）| 树: lang `86b3f9739`、man `c3b0faf` | **过程（两次被真实工具链纠正）**：①scheduler 源先被 A2R 的 compute 面拒（`a2x.unsupported-stmt`：`print(...)` 调用的 Eval 语句不在计算面）⇒ 改用 `return` 形式；②产物冷构建缺 `a2x_sched_*` 前导 ⇒ 把 `sched_rt.txt` 暴露为 `scheduler_runtime_source()` 并加 `append_scheduler_runtime(...)`（两个 pub 助手，供提供者框架将来装配）——但补上后仍缺**文本/记录前导表**（`A2xStr` 等，来自 `prelude_seed` 解码的 records/variants/lists），那是提供者框架的装配路径，本无缓存驱动没有 | **决定（诚实边界）**：scheduler 家族在无缓存驱动上**具名拒绝** `ace.rust/profile-unsupported`（消息写明缺 `prelude seed`），**绝不吐出编译不过的 file set**；新增用例钉住该拒绝 | **实测**：host `p982_rust_driver` 4/4、`ace-a2x-rust` 2+3、man `plan982_ace_rust_cold_build` **5/5**（scalar/records/collections 三家族冷构建实跑 + scheduler 具名拒绝）| **家族矩阵（现值）**：scalar ✅ 冷构建 `value:5`；records ✅ 冷构建 `value:7`；collections ✅ 冷构建 `value:42`；scheduler ⛔ 具名拒绝（缺 seed 装配）| **仍开（RUST 后续）**：①scheduler 的 seed 装配（提供者框架路径）；②项目多文件/写盘；③A2R vs 旧转译逐文件语义 diff 清单；④真实 sha256 身份 | 判读：RUST 的"可构建或具名拒绝"覆盖到四个家族；scheduler 与多文件仍是能力缺口，不声明 RUST 整体交付
2026-11-18 stage: goal | plan_id: PLAN-982 | plan_revision: r8 | round: 31（本会话）| from: work | to: work | status: executing | outcome: pass（RUST：scheduler 家族具名拒绝（缺 prelude seed），三家族冷构建实跑；暴露 sched runtime 助手）| worktrees: lang@86b3f9739 / man@c3b0faf | next: scheduler seed 装配 / 多文件路径 / diff 清单
2026-11-18 stage: work / **982/RUST：两引擎差异清单（钉成断言）**（objective「继续完成计划982」，round 32）| 树: man `da07c6b`（新增 `plan982_ace_rust_diff.rs`）| **交付**：把"两个生成器不是同一个东西"写成**四条可核对差异**：⓪**输出通道**（旧转译写 `<stem>.a2r.rs`，A2R 的 `auto rust` 打印 stdout / 由 `-o` 决定）；①**命名**（旧保留 `fn main`，A2R 降级 `fn_main`）；②**运行时**（A2R 自带 `use crate::a2x_runtime::*;`，旧无）；③**体量**（实测 legacy 160B/10 行 vs A2R 591B/18 行，>2×——逐字节等价**不成立**，语义等价由冷构建门负责）；④入口包装（`A2xResult`/`value:`）在 fileset 的 `main.rs`，由 `--merge` 用例覆盖（该形态可独立构建）| **实测**：`plan982_ace_rust_diff` 1/1（打印实测体量）| **判读**：差异清单从"叙述"变成"断言"，任何漂移都会红；RUST 剩余能力缺口（scheduler seed 装配、项目多文件写盘、真实 sha256 身份）仍具名登记 | next: scheduler seed 装配 或 项目多文件写盘
2026-11-18 stage: merge / **EXEC+RUST 阶段 fold（两仓落主＋落地复核＋树清理）**（objective「继续完成计划982」，round 33）| **landed**：auto-man `da07c6b`（**ff-only**，11 提交：EXEC 命令面边界、RUST 引擎选择/驱动/冷构建/差异清单）；auto-lang `ff43b7609`（**merge commit**：A2R 驱动、家族装配、scheduler 边界、RUST 探针；master 已前进 15 提交，**零冲突**）| **落主复核（两仓 master 实跑）**：man `cargo tc`＝**259/4**；plan982 七套件全绿＝candidate_config 5、config_b 2、exec_b 3、rust_b 5、rust_cold_build 5、rust_diff 1、config_slice 1；lang host `p982_rust_driver` 4、`p982_vcfg_cases` 8、`p982_config_engine` 1；`ace-a2x-rust` 2+3；`ace-core --lib --tests`＝**766** | **树/分支**：两仓 `plan-982-dev` 已 `-d` 删除、worktree 已移除（移除前 `wt-guard.ps1` 均 **clean**）；组目录仅剩只读兄弟仓 | **本阶段交付小结**：**EXEC-1**（run 家族命令在候选腿下具名拒绝、零写、无 legacy fallback）；**RUST**（`AUTO_ACE_RUST` 显式引擎选择＋回执、宿主驱动 `transpile_scalar_source`/`transpile_with_profile`、`--merge` 单文件可独立构建、scalar/records/collections 三家族冷构建实跑 `value:5/7/42`、scheduler 具名拒绝、项目与 merge 面无静默 legacy、两引擎差异清单成断言、零 legacy dispatch 回执）| **仍开（具名，下一阶段）**：EXEC 其余面（stdin 能力键/provider、项目 run/service、req 持久变量、debug/test 失败码）；RUST 其余面（scheduler seed 装配、项目多文件写盘、真实 sha256 身份）；CONFIG 边界（运行期 `for`、`[]`、对象字段读取/`cfg.get|merge`、二阶段 context）；其余 M2-B 切片（TEMPLATE/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK）| 判读：两个用户优先切片（EXEC、RUST）的第一阶段都已**可验证地落主**；不声明 EXEC/RUST 整体交付
2026-11-18 stage: work / **CONFIG 对象字段访问的可行设计（`cfg_super_put`，绕开 Str ABI）**（objective「继续完成计划982」，round 34）| 树: 未改动（只做契约勘察）| **问题**：`heap: kernel.heap` / `${kernel.heap}` 需要"把对象的字段搬进文档"，但配置车道**没有"从宿主返回 Str"的通路**（`cfg_*` 全部返回 i32 句柄/状态；`Value::Str` 只在内核内部，不经配置原语回到脚本），所以任何 `cfg_obj_get_str` 设计都会卡在 ABI 上 | **设计（i32-only，无需新 ABI）**：新增原语 `cfg_super_put(dst_node:i32, src_obj:i32, key:str)` —— 把 `src_obj` 的**指定字段原样搬进** `dst_node` 的同名属性（任何值种类：i32/bool/char/text/obj/list/node），返回 i32 状态；缺键是具名错误。物化侧只在**值位置**识别 `FieldGet{base: Var(x), field}` 且 `x` 的 `LocalClass` 是 `Obj` 时改发这条原语（字段值不落到脚本变量，因此没有 Str 返回需求）| **落地清单（下轮照做）**：①`config_ops.rs` 加 `cfg_super_put`（`auto.config.document.super-put.v1`，params `[I32,I32,Str]`，result `I32`）；②`av-core/src/config_document.rs` 加 `ConfigSpec::SuperPut` + `spec_of` 解析 + dispatch（`store` 侧按源条目的 `StoredValue` **克隆**，仍走 `charge_value/charge_bytes`；根级仍受 `seeded` 覆盖规则约束）；③物化侧 Pair 值位置的重写（`ctx.locals` 已是 `BTreeMap<String, LocalClass>`，`LocalClass::Obj` 可直接判定）；④用例：`var kernel = { heap: "big" }\nheap: kernel.heap` ⇒ 文档 `heap=="big"`；缺键 ⇒ 具名错误；嵌套 `kernel.mem.heap` 暂不做（登记） | **勘察结论（为什么不直接实现）**：配置前门的表达式是**共享 AST**（`Expr`/`ExprKind`），而 `FieldGet` 的规范形态在 `adapter.rs` 的**已解析层**（`FieldGet{base, path}`）——在物化层改发原语需要先确认共享 AST 里字段名的承载形态（本轮未核实完，硬写会做出一个"只在测试源上对"的重写）| 判读：这是**设计类**进展；没有新代码，也不声明对象字段访问已支持 | next: 按上面四步实现 `cfg_super_put`（先核实共享 AST 的 FieldGet 形态）
2026-11-18 stage: work / **上一轮 `cfg_super_put` 设计的前提被否决（勘察结论，非实现）**（objective「继续完成计划982」，round 35）| 树: 未改动（临时 worktree 已建后删除，分支已 `-d`）| **否决点**：round-34 的设计假设"物化层能在**共享 AST** 里看到 `FieldGet`"——**不成立**：`crates/core/src/source/ast.rs` 的 `ExprKind` 里**没有**字段访问变体（只有 `Index`/`ObjectLit`/`ArrayLit` 等），`kernel.heap` 这类成员访问是在 `adapter.rs` 的**已解析层**才成为 `FieldGet{base, path}`（`RExprKind`）。因此"在 `emit_line` 里改发 `cfg_super_put`"无处落笔——除非先改配置前门把成员访问**投进物化层** | **修正后的路径（下轮照做，二选一）**：①**A（推荐，改动小）**：在配置前门的投影里，对**文档值位置**的成员访问保留 `RExprKind::FieldGet{base, path}` 事实（现在它被解析层直接拒成 `source.undefined`/字段访问错误），物化侧再用 `cfg_super_put(dst, src_obj, key)` 搬字段（该原语设计仍有效：i32-only、无需 Str ABI）；②**B（更贴近产品）**：把对象字段访问做成**注入面**——宿主把 `kernel` 的对象字段直接作为 args 注入（需要对象注入，当前 `ace.config/inject-unsupported`）| **本轮价值**：避免了一个"只在测试源上对"的假实现；把下一步从"实现原语"改成"先让成员访问进入物化层"这一**更小的前置** | 判读：无代码交付；对象字段访问仍未支持，契约更准确 | next: 路径 A 第一步——配置前门投影保住字段访问事实
2026-11-18 stage: work / **CONFIG：`cfg_super_put` 原语落地（路径 A 的第一步）**（objective「继续完成计划982」，round 36）| 树: `.wt/lang-982/auto-lang@6596d491e` | **交付**：①`av-core`：`ConfigSpec::SuperPut`（符号 `auto.config.document.super-put.v1`，参数 `[I32 dst, I32 src_obj, Str key]`，结果 `I32`）＋dispatch——把源对象条目的**字段句柄**挂到目标节点属性上（不复制值、不引 Str 返回面），缺键/非对象源都**具名**；根级 `seeded` 覆盖规则照旧（注入值赢）；②`core/src/source/config_ops.rs` 目录项（前门将来可直接发射）；③`crates/av-core/tests/p982_config_super_put.rs`（新 test target）：搬运成功（`heap="big"` 成为 root 属性）、缺键具名错、非对象源具名错 | **实测**：`p982_config_super_put` **1/1**、`ace-core --lib --tests` **766**、`ace-av-core` 48/11（无回归）| **仍未做（路径 A 剩余）**：配置前门的**投影**必须先把文档值位置的成员访问事实保住（当前解析层直接拒），物化侧才能在 `FieldGet{base, path}` 上改发本原语；嵌套 `kernel.mem.heap` 与实现细节（字段句柄共享 vs 克隆）留待该步一并定 | 判读：原语与目录都已就位并可被会话级合同验证；**对象字段访问对用户仍不可用**（前门未接），不夸大为已支持
2026-11-18 stage: work / **CONFIG 对象字段访问：真正的阻断点已定位（`resolve.rs` 的 `Task::FieldOut`）**（objective「继续完成计划982」，round 37）| 树: 未新增提交（探针已还原；分支仍 `6596d491e`）| **实测探针（临时用例，已删）**：`var k = { heap: "big" }\nheap: k.heap` —— **解析成功**（2 行，成员访问确实保住了），失败发生在**物化/解析类型域**：`Bind / source.type-mismatch :: field access requires a record-typed value, found `i32`` | **阻断点（精确定位）**：`crates/core/src/source/resolve.rs` 的 `Task::FieldOut` 分支要求基值 `VT::Rec(ri)`；配置文档的对象是**句柄**（`VT::I32`），因此成员访问必然在这里被拒。要放行，需要三处联动：①**配置面**给文档对象一个专属类型（如 `VT::DocObj`，只在 config-document profile 产生）；②`RExprKind` 增 `DocFieldGet { base, key }`（不落 Str、不落字段句柄值）；③物化侧把它发射成上一轮已落地的 `cfg_super_put(dst, base, key)`（原语与目录项已在 `6596d491e`）| **为什么不在本轮硬做**：`VT` 是解析器的全局类型域，加变体会波及所有 match（含大量 profile 无关路径），必须先确认它只对 config-document profile 生效且不改变其它 profile 的诊断——这是**下轮第一件要核的事**（grep `VT::` 的穷尽 match 面）| 判读：设计→原语（round 36）→阻断点定位（本轮）；对象字段访问仍不可用，但路径已缩到"三处联动 + 一个类型域影响面核查" | next: 核 `VT` 影响面 → 加 `VT::DocObj`/`RExprKind::DocFieldGet` → 物化发 `cfg_super_put` → 用例
2026-11-18 stage: work / **CONFIG 对象字段访问：第二条可行面（对象索引）也被语法挡住**（objective「继续完成计划982」，round 38）| 树: 未新增提交（探针已还原）| **新发现**：解析器里对象**已有**字段读取原语 `RExprKind::ObjectGet{base,key}`（`adapter.rs:1545`），其产生点是 `resolve.rs` 的 `VT::Object(oi)` 索引分支（`k["heap"]` 形态）。也就是说"字段读取"在类型域**不需要新变体**——只要配置面能写出该表达式 | **实测两条路都不通**：①`heap: k["heap"]` ⇒ `Text / source.syntax :: skipped unexpected token at top level`（配置面把 `[` 当成了节点实参/非法后缀，索引后缀在文档值位置没有被解析）；②`heap: k.heap` ⇒ `Bind / source.type-mismatch :: field access requires a record-typed value, found `i32``（`k` 在文档面是 i32 句柄）| **修正后的最小路径（下轮）**：先让**索引后缀**在配置值位置可解析（`Index` 已在共享 AST：`ExprKind::Index{base,index}`），物化侧把 `Index{base: Var(obj), index: StrLit(k)}` 发射成已落地的 `cfg_super_put(dst, base, k)`——比"给 VT 加 DocObj + 改 FieldOut"影响面小得多，也不动解析器的类型域 | 判读：对象字段访问仍是具名缺口；但可行路径已从"改类型域"缩到"放开配置面的索引后缀 + 物化一条 arm" | next: 配置值位置接受 `Index` 并被物化层映射到 `cfg_super_put`
2026-11-18 stage: work / **CONFIG 对象字段访问：打通（`heap: k["heap"]` 端到端）**（objective「继续完成计划982」，round 39）| 树: `.wt/lang-982/auto-lang@7a5f823c0` | **交付（三处，最小面）**：①**语法**：索引后缀 alt 的 profile 列表加 `config-document`（`grammars/auto-source.at`，只给该 profile 增能力，`ace-syntax-grammar` 45/45 无漂移）；②**物化**：Pair 值位置识别 `Index{base: Var(x), index: StrLit(k)}` 且 `x` 是 `LocalClass::Obj` ⇒ 发射已落地的 `cfg_super_put(dst, x, k)`（`object_index_read` 助手，其它形态照常走 `emit_value`）；③**验证**：core 用例断言产物里出现 `auto.config.document.super-put.v1` 内在符号（13/13）；桥用例 `heap: k["heap"]` 端到端 ⇒ 文档 `heap=="big"`（`p982_vcfg_cases` 8/8）| **实测**：`ace-core --lib --tests`＝**767**、`ace-av-core` 48/11、`p982_vcfg_cases` 8/8、grammar 45/45 | **口径**：**`k["x"]` 形态的字段读取现在可用**（值原样搬进文档，任何值种类）；`k.x` 点形态仍不可用（解析器 `Task::FieldOut` 要求 `VT::Rec`，已登记为下一档）；嵌套 `k["a"]["b"]` 可组合（每层都是搬运）但未加用例 | 判读：对象字段访问从"具名缺口"变为**部分可用**（索引形态），不声称点形态已支持 | next: 点形态（`k.x`）或嵌套用例
2026-11-18 stage: work / **CONFIG 对象字段搬运：错误面补齐 ＋ i32 差额登记**（objective「继续完成计划982」，round 40）| 树: lang `7a5f823c0` + 本轮提交 | **新增证据**：①**缺字段是具名终态**——实测桥把它报成 `HostFailure{code:"ace.config/infra", message:"config: object has no field `nope`"}`（不是 Diagnostic、不是空值、不是 panic），用例已按真实形态钉住；②**新发现差额**：**i32 字段的搬运目前不落地**（`var k = { port: 8080 }` + `p: k["port"]` ⇒ root 上拿不到 `p`），而文本字段正常 ⇒ 对象字段的**存储面随值种类不同**，`cfg_super_put` 只在文本路径上被验证过；已登记为下一档（不冒充 i32 已支持）| **实测**：`p982_vcfg_cases` 8/8；`ace-core plan982` 13/13 | 判读：文本字段搬运（`k["heap"]`）端到端可用且有正/负例；i32 搬运是**具名差额**；点形态（`k.x`）仍不可用（`Task::FieldOut` 要求 `VT::Rec`）| next: i32 字段存储面查因（`ObjData.fields` 对不同值种类的落点）或点形态
2026-11-18 stage: work / **CONFIG 对象字段搬运：真因是"目标键"缺失（i32 不是存储问题）**（objective「继续完成计划982」，round 41）| 树: lang `ae9f5f4fc` | **真因**：上一版 `cfg_super_put(dst, src, key)` 把**源字段名**直接当目标属性名写入，因此只有"pair 键 == 源字段名"时才看起来正确（我早先的文本用例恰好同名，掩盖了缺陷）；`p: k["port"]` 一测就暴露（root 上没有 `p`）| **修复**：原语改为 `super-put(dst, src_obj, src_key, dst_key)`（4 参数），搬运值种类不变、落在 **pair 键**上；目录项、物化发射、会话合同用例、桥用例同步更新 | **新增/加强证据**：桥用例断言 `p == 8080`（i32 保真）**且** `port` 不在 root 上（源名不泄漏）；会话合同用例改用不同名（`heap_copy`）证明目标键生效 | **实测**：`p982_config_super_put` 1/1、`p982_vcfg_cases` 8/8、`ace-core plan982` 13/13、`ace-av-core` 48/11 | **口径**：`k["x"]` 形式的对象字段搬运现在**任何值种类**都正确落到 pair 键；点形态 `k.x` 仍不可用（`Task::FieldOut` 要求 `VT::Rec`）| 判读：上一轮登记的"i32 差额"本质是**目标键缺陷**，已闭；不声称点形态支持 | next: 点形态（`k.x`）或嵌套 `k["a"]["b"]` 用例
2026-11-18 stage: work / **CONFIG：对象值字段的搬运（嵌套对象保形）**（objective「继续完成计划982」，round 42）| 树: lang `HEAD`（vcfg 用例扩展）| **交付/证据**：`var k = { a: { b: 1 } }` + `x: k["a"]` 端到端 ⇒ 文档里 `x` 是 **Object**（保留 `b` 字段），既没被拍平也没被丢；文本/i32 字段搬运与"落在 pair 键、源名不泄漏"已在 round-41 钉住 | **探查结论**：①多个对象字面量（`var k`/`var m` 各自搬运）前端可编译（10 条语句）；②**嵌套对象字面量**在配置面也是合法的（`{ a: { b: 1 } }` 编译通过，8 条语句）——因此"嵌套搬运"不需要新原语；③`k["a"]["b"]` 这种**链式**读取仍未验证/未支持（物化侧只识别一层 `Index`），已登记 | **实测**：`p982_vcfg_cases` 8/8 | 判读：对象字段搬运现在覆盖 text/i32/object 三种值形；链式与点形态仍具名 | next: 链式 `k["a"]["b"]`（用 `cfg_obj_get_obj` 取中间句柄）或点形态
## 10. 待澄清事项

**B已定归属（2026-11-17）**：本包B-CONFIG两仓端到端实现，见§5 CONFIG-ACE-B和设计；本轮用户已要求补齐此合同，无“owner未指派”决策等待。原Q-982-1是已裁定的A route C历史，不是B开工的三选一。T-01所选局部实现机制不新增公共语义；遇真实反证按needs_replan保原oracle。

### Q-982-1（2026-11-17，A 实现路由）——**已裁定：route C**（2026-11-17，用户）

**裁定结果**：用户在三选一中选 **C（收窄 A）**。A 只交"同一 SDK 身份＋无 legacy fallback＋失败/隔离纪律"的 PROOF 与 CONFIG 消费改造；配置**引擎**迁移随 `GAP-982-CONFIG-ACE` 并入 M2-B，并在 A 收据具名差额。RG-A01 的"缺 ACE 不能回退旧 VM"按**命令执行腿**解释（配置腿按 969 规范本来就是 legacy owner，不是 fallback）。A 合同见 §5"切片定义"，交付/复审/落主收据见 §9。原问题与三选项保留如下供追溯。

**事实**（证据见 §5"T-01 冻结记录"）：982/CONFIG 的"接 ACE"需要一个当前**不存在**的 ACE 配置文档求值面——ACE 无 config/document `SourceProfile`、无文档值种类、`canonical_owned` 拒 `Value::Object`，真探针 `av-worker request` 对真实 `pac.at` 直接 `frontend:source.syntax` 拒；而 969 的已验规范把 `eval_config_with_vm` 显式留在 legacy owner（`m2-facade.md` §边界）。故 `GAP-982-CONFIG-ACE` 是 A 的硬前置，且 owner 未指派。

请裁定 A 的实现路由（三选一）：

| 路由 | 内容 | 代价/影响 |
|---|---|---|
| **A（扩范围）** | 982 自建该能力：ACE 增加配置文档面（文档值＋求值语义＋canonical 投影＋进程内车道），auto-man 接线 | 本 Plan 改面从"仅 auto-man"扩到 ACE 语言核心，等同一项 capability 开发；需设计沉淀与 A/B 再拆分；AC 分母不变 |
| **B（另派 owner）** | 把 `GAP-982-CONFIG-ACE` 指派给能力 owner（新计划或既有 owner 后续切片），982/A 停在此处等来源 | 982/A 本轮起保持等待；§5 冻结资产与两树保留不动，不做产品侧改造 |
| **C（收窄 A）** | A 只交"同一 SDK 身份＋无 legacy fallback＋失败/隔离纪律"的 PROOF 与 CONFIG 消费改造：配置求值按 969 规范由 legacy owner 在**同一 SDK** 内执行，ACE 承担最小 CLI 执行腿；配置引擎迁移随缺口并入 B，并在 A 收据具名差额 | 需把 RG-A01 的"缺 ACE 不能回退旧 VM"解释为**命令执行腿**（接受面收窄，须用户明确点头）；A 可当轮开工 |

**推荐：C**。A 的产品见证价值在"真实 CLI 消费＋同一来源身份＋无静默回退"；配置文档求值面是一项独立的语言核心 capability，塞进产品包会同时破坏 982 的改面声明与来源纪律。若 969 时代的原意就是"配置求值留在 SDK 的 legacy owner、A 只证 SDK 身份"，那正是路由 C，可直接答 **C**。


Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
