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
| E5 owned 导出 | 迭代式导出（无递归风险），保顺序/重复节点/精确数值/含 NUL 文本 | 2000 层深文档不崩；u64>i64::MAX 仍具名拒。**已交最小闭环（`079b7410e`）**：`OwnedDocument/OwnedNode/OwnedValue` 导出保顺序/重复/精确标量/UTF-8 文本（含 NUL 字符串按字节保真）＋`DocumentCost` 真实代价；**未交**：迭代式（现递归 + 2048 深度守卫，2000 层压测未跑）、u64>i64::MAX 的拒（当前值面只到 i32/bool/str/char，u64 随值面扩） |
| E6 host 桥 | 新建 `crates/auto-lang-ace-host/src/config_document.rs`：compile→verify→execute→export→teardown，实现 `AceExecutor::eval_config` | `--test ace_config_engine` 换真桥仍绿；无桥仍 `ace.config/executor-unsupported`。**已交（2026-11-17，`308c2c07e`）**：真桥 `eval_request` 端到端（源前门→`lower_checked_owned`→`verify_mir`→ABC 准入→会话→执行→关会话→owned→中立 `ConfigDocument`），预算逐轴映射、失败路径统一 abandon 会话、无 legacy 回退；`--test p982_config_engine`（同进程真桥）与 `ace_config_engine`/`ace_config_registry` 全绿；`e0_facade`/`t08_facade` 无回退 |
| E7 Man 消费 | pac 读取切 `from_file_with_engine(.., AceAv)`＋二阶段 context/dep；legacy 仅显式 | `cargo tc`＋780 矩阵回归；B 实际 `config.engine=av`、`legacy_config_dispatch=0`；坏配置零写；同名包隔离。**已交前置（2026-11-17，`b70ec23e3`）**：context 注入最小面——`ConfigContext.args` 在文档里**按名可见**（bool/i32/char/text；`on`/`off` 内建；文档 `var` 同名遮蔽注入），超范围/不支持种类具名拒（`ace.config/inject-range`/`inject-unsupported`）；真桥用例覆盖（`port` 分支、`toggle: on`、遮蔽、SDK 门、拒绝）。**已交（2026-11-17，`6701094ff`）**：真实 pac 语料对齐——`#` 行注释（config 面 trivia）、JSON 形整份文档（字符串键/逗号/尾逗号）、块形对象值（换行分隔、嵌套、字符串键去引号）；SDK 投影与 legacy **逐字对齐**（根名 `root`、裸/字符串身份只落 `id`、括号实参进 `_arg0…` 普通 prop、i32 范围整数落 `Int`）。探针证据（`P982_PAC_CORPUS`，auto-man 同树 152 份 `pac.at`）：**149 份 ACE 成功且与 legacy 结构全等（0 处分歧）**，其余 3 份＝1 份模板文档（`lib ${name} {}`，V-CFG-TEMPLATE 未交）＋2 份计划内故意坏夹具。**已交（2026-11-17，`3d1302b78`）**：**模板插值** `${…}`（V-CFG-TEMPLATE 最小面）——普通字符串里的 `${expr}` 按 f-string 同一生成物重写为 `__fstr(...)`（`TextDisplay`/`TextConcat`），节点头裸 `${expr}` 按 legacy 口径留字面（只落身份/`_arg0`）；host-console 描述符补 `text_display`/`fstr` 两 case 与 `parts` 字段；**宿主 args 播成根属性**（legacy 语义：注入值赢、文档同名键不覆盖）。语料（152 份）：**严格口径（不注入）145 份 OK 且与 legacy 0 分歧**，6 失败＝5 份模板（无注入时 ACE 具名 `source.undefined`，legacy 留字面——故意 fail-closed 分歧）＋1 份计划内坏夹具，1 份 legacy 也拒；**注入 `name` 后模板文档全部产出 `probe` 形态**（5/5）。**已交（2026-11-17，`02a4789` man＋`81391da11` lang）**：auto-man 三个 pac 读点（`automan.rs` 探针/正式两读、`pac.rs` dep 读）与候选腿配置读统一走新 `auto_man_runtime::config_engine`：**默认显式 ACE**（桥已装 ⇒ `AceAv`；无桥宿主 ⇒ legacy 且可观测）、`AUTO_CONFIG_ENGINE=av|legacy` 覆盖、失败**不静默回退**；`observation()` 给真实派发计数，候选见证新增 `config.engine`/`config.legacy_dispatch`。**B 证据（`crates/auto --features ace-candidate --test plan982_ace_candidate_config`＝5/5）**：`config.engine=="av"`、`legacy_dispatch==0`、坏配置零写、双项目隔离；`cargo tc`＝**257 passed / 4 skipped**（含新增 `config_engine` 3 例）。附带修复 A 切片遗留的 `cargo tc` 红：`test/fixtures/plan982/{bad-config,bad-eval}` 纳入 `project_model_corpus` 的两个 allowlist（身份绑定）。**未交**：二阶段 context/dep；`780` 矩阵未单独跑（`cargo tc` 已覆盖 auto-man 7 包 lib 面） |
| E8 V-CFG 九 Case | SYNTAX/FLOW/CONTEXT/VALUE/FAIL/LIMIT/IDENTITY/PRODUCT 永久用例＋独立 oracle | 每 Case 可独立红；删行/错 producer 必被发现；之后才 B-CONFIG-R。**已交（2026-11-17，`034aef0cd` lang＋`81b27b5` man）**：语言侧七例 `auto-lang-ace-host/tests/p982_vcfg_cases.rs`（SYNTAX 三形头＋Node值 pair、FLOW 注入分支/重声明/嵌套体并带"另一分支不同结果"非空证、CONTEXT bool/int/text 分轴＋on/off＋文档遮蔽＋不支持种类具名拒、VALUE 中文/emoji/NUL/边界整数/空对象/嵌套/重复孩子、FAIL 原 span＋零残留会话＋修复后真读、LIMIT 值预算/字节/取消三轴并带默认预算非空证、IDENTITY 键集与值变身份、服务轴）；Man 侧两例 `crates/auto/tests/plan982_ace_config_b.rs`（PRODUCT：`engine=="av"`/`legacy_dispatch==0`/deps+targets 真实/执行成功；RETIRE：显式 legacy 可见且 1 次派发、未知取值不静默降级）。**已交（2026-11-17，`b26268d44`）**：FLOW 的 `for` **字面数组展开**（legacy `test_config_for_unrolls_literal_array` 语义）——迭代源是字面数组（`for d in (["a","b"])`，括号拼写是当前前门接受的形）或绑定到字面数组的文档 `var` 时，物化侧按源顺序展开体、循环变量逐项绑定；运行期列表（元素来自注入/计算）仍具名 `source.config-document/pending`；语言侧 FLOW 例补 unroll 断言（两节点 id/at 逐项）＋ pending 断言，core 侧补 `p982_config_for_unrolls_literal_lists`。**未闭（具名）**：①FLOW 的**运行期** `for`（元素来自注入/计算）仍是 `source.config-document/pending`（列表计数/取值原语未做）；②VALUE 的 `u64>i64::MAX` 与空列表 `[]`（grammar 共享面，需 config-only 空列表原子）；③LIMIT 的 2000 层深/宽输入未压测；④V-CFG-SYNTAX 的 780 静态 dep 源未逐份实跑（语料探针 145/145 已在 E7 交） |
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
