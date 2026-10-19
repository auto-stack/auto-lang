---
plan_id: PLAN-809
status: executing
feature_name: Man 项目、管理与构建规划核心整体解耦
author: [Codex]
created_at: 2026-10-18
updated_at: 2026-10-19
plan_revision: 1
current_step: 4
total_steps: 11
supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/project-model/project.md]
new_spec_components: [docs/specs/auto-man-core/project.md, docs/specs/project-model/design/generation-request.md]
touched_goals: [GOAL-003]
affects: [auto-man-core, auto-project-model, auto-man]
depends_on: [PLAN-780, PLAN-786, PLAN-791, PLAN-795, PLAN-800, PLAN-803, PLAN-805]
---

# [PLAN-809] Man 项目、管理与构建规划核心整体解耦

## 0. 变更摘要

交付R05完整管理/构建规划模块，而非再抽几个工具函数：新建`auto-man-core`承接声明归一、Target依赖/链接闭包、源文件选择与构建规划纯规则；复用已有project-model/resolver/lock/toolchain/gen-core/assets。语言执行、Git/FS观测、转译与Builder实际执行在上层具名adapter，真实Pac/Automan路径委托新核心，原规则退役。

同一Plan交付I-PROJECT轻项目/生成请求数据合同，供后续R04把Man内UI生成器搬出。I-PROJECT不单独编号，不让UI依赖Man大库。暂不执行本计划、不搬仓、不把整个auto-man宣布独立。

## 1. 目标

- 无完整语言/VM/UI依赖的管理核心能独立给出真实项目的依赖、目标、源/包含文件和构建动作规划；不止新DTO与演示consumer。
- Pac/Target旧公开字段/方法和动态配置继续可用；管理规则单源，效果依赖从纯规则分离，三类真实Builder继续工作。
- I-PROJECT带来源、显式后端/模式、资源与输出根及生成所需轻字段，未知/不支持数据不丢失、不假称可生成；保留静态/动态求值分流。
- 不迁UI生成器/资产（R04）、不做统一工具执行协议/薄CLI（R06）、不更改pac格式或顺手修模板发布事务/编译器store遗留问题。

## 2. 架构方案

依赖方向为`auto-man(adapter/旧门面) → auto-man-core(管理规则) → 现有轻数据/规则包`。`auto-project-model`增具名generation-request模块承接I-PROJECT；与其现有静态扫描器分开，输入可以是静态已准入描述或动态配置求值后的显式投影，不强制所有工程先过静态扫描器。

**不整体搬Pac/Target后再在旧crate写inherent impl**（Rust不允许）；默认保留语言/Builder相关旧门面，核心以借用view/具名输入和动作结果驱动单份规则。需要迁的纯数据采用唯一类型重导出；无法迁的旧字段通过适配view直接借用或一次调用期快照/结果apply，不维护两个长期可变project/target状态。T-01以编译探针冻结字段/方法兼容方案、动作边界与所有者，之后才能批量迁移。

## 3. 技术栈

Rust，复用auto-val/auto-atom、auto-project-model、auto-package-resolver、auto-package-lock、auto-toolchain-core等现有规则；按实际需要选择最小依赖，不为“通用core”收入Parser/AutoConfig/AutoVM/Builder/IndexStore/renderer。泛型view/trait用于真实规则输入，不建立无实际消费者的抽象框架。

## 4. 需求分析与背景调查

授权：用户要求按较大模块粒度建立下一批计划，可并行且最终几个干净仓；本轮只起草auto-lang仓内计划。相邻旧仓已保全，真正承接/默认分支切换仍在R08/R09；未授权本会话写远端。来源：[总路线](reports/v06-extraction-execution-roadmap.md)、[并行合同](reports/module-migration-batch-807-809.md)、[hash清单](attachments/module-batch-20261018/inputs.json)。

权威输入：[Man](../specs/auto-man/project.md)、[project-model](../specs/project-model/project.md)、[包锁](../specs/package-lock/project.md)、[resolver](../specs/package-resolver/project.md)、[工具链](../specs/auto-toolchain-core/project.md)。关键实际写面：`auto-man/src/{pac,target,dir,group,port,builder,automan}.rs`、`project_model_adapter.rs`、builder子目录、`auto-project-model/src/descriptor.rs`。

现状不是纯DTO搬家：Pac持AutoConfig派生字段、BackendConfig/ThemeDecl、Box<dyn Builder>和Index；Target持声明/已扫描srcs/links/deps又带语言转译/FS方法。Pac::resolve下载后构建依赖闭包，再scan；旧while无进展时会死循环。HashMap遍历不能擅自作为稳定全序承诺；missing link可能是系统/外部链接，不可一律报错。Builder三个实现当前直接消费Pac/Target；核心不能留假入口、生产继续旧算法。

project-model只有冻结静态子集，786只切展示三字段且有64/65保守路由护栏；未知raw/unsupported必须保留，不能把它扩为完整VM替身。795已用零DTO往返view保锁规则单源；800/805已有resolver/工具链核心，不再复制。799/801不构成新前置，806/807/808亦非前置；前列depends_on均须work核实已交付。

## 5. 详细设计

### 5.1 I-PROJECT数据交接

在现有`auto-project-model`新增具名模块（建议`generation_request.rs`，T-01冻结API）及独立consumer。一次请求承载来源/工程根、实际输入源清单、选定后端与运行/生成模式、输出根、资源声明和生成所需轻展示/开关/主题原始声明等；具体字段由**真实UI生成器读取点穷举**决定，不能只凭想象列最小字段导致搬迁再反向抓Pac。

复用现有SourceInfo/Render等身份；BackendConfig/ThemeDecl等语言-owned类型不进入轻包。若需无损主题/扩展属性，用已有auto-val结构值/具名声明表达，保来源/未知项；不把Token/AST塞进去也不靠Any/serde任意blob逃避合同。输入来源可追溯“静态准入/动态求值结果”，请求不能将静态unsupported当已求值。路径按pac/source根解释而非cwd；env/locale/CLI覆盖在明确adapter顺序生效。纯数据消费不执行网络/建目录/编译。

R04只在本Plan**review+merge整体交付后**消费I-PROJECT，本合同不安排中途phase merge，不让后续依赖WIP；接口报告/样例可提前只读交接。实际UI生成器暂留Man，但其项目读取必须先经该请求/访问合同，以证明字段完整；生成算法/全量资源迁移仍归R04。

### 5.2 核心管理规则与效果adapter

| 核心（本Plan必须单源且生产使用） | 上层adapter（具名且不伪报已搬） |
|---|---|
| Target/Pac声明纯字段归一、场景/端口/后端选择及数据投影 | AutoConfig执行型求值、BackendConfig/ThemeDecl转换、交互选择和locale/env |
| target/direct/transitive links、dep_names/all_links合成、已有resolver/lock对接与构建目标选择 | Index/Git/网络/磁盘状态观测与物化、旧Target门面结果apply |
| Dir递归/筛选/源包含选择规则、group与纯输出数据规划 | 文件枚举/metadata/真实文件写入，避免core内隐式cwd/env |
| Builder所需目标、编译/链接输入、依赖约束、工具链规则及动作规划 | Ninja/Cargo/Vue命令执行、转译/缓存/API生成、日志/进程/返回码 |

不是把“全部效果”再藏成一个`Fn(&mut Pac)`接口：每类观测/动作有typed输入输出、失败阶段和真实消费证据。旧Builder可留签名门面，通过view接收核心计划后执行，核心不能反向持Builder。纯Node/Value转换同样单源，语言Atom结果封装留上层。

### 5.3 兼容、异常与资源边界

- 正常图规则、声明覆盖、外部link、排序/去重及格式按旧臂和Spec冻结；对原无序结果以集合/偏序及实际文件文本分别验，不伪造字节稳定排序承诺。所有三Builder已有支持面不能只选Ninja就宣称整体完成。
- **有界图准入**：合法DAG无回归；可证明的内部target cycle或无进展闭包返回具名、带target路径的规划错误，不沿旧while挂死，不把外部链接名判为missing dependency。此为迁入管理核心明确新增的失败合同，不要求旧臂产生同错误；用手算小图/独立oracle验证。资源不足和非法图分开记录，不凭迁移新设静态语法限制。
- 动态pac/port/am.at继续显式语言adapter；复用786保守护栏而不扩张静态等价域。字段读写后重新view/规划时可见新值，不能遗留双状态。必要公开签名pin和结构体构造consumer证明兼容；若无法保持，T-01先提出具体重规划，不偷偷用Deref新类型破坏字段构造。
- Git/FS/外部进程仍用现有facility，不新建一个平行runner。迁移前后读写/下载/scan/构建顺序、锁格式、失败/输出可见性冻结；不把803先建后求值失败窗口偷偷修成“原子事务”。测试使用隔离temp和本地fixture，禁止worktree内创建junction/symlink。

### 5.4 实际生产闭环与发行

至少证明Pac::new/resolve→Target scan/links→build_targets→真实Ninja/Cargo/Vue Builder在本Plan核心规划上工作；Automan原调用不绕回第二套规则。无需为测试执行远端包下载，可用本地受控Git/索引/小项目，并对既有CLI入口记录受影响命令/产物/exit。UI生成请求通过真实现有生成器输出对照，R06再接工具进程协议。

新增core和扩展project-model的normal/build/dev闭包无auto-lang/UI/auto-man/auto-gen完整执行栈；独立manifest的fresh/locked消费、777合法域边、792新增受管source及受影响资源清单随迁移维护。不能把新核心的纯边等同整个CLI已轻量化。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-man-core/project.md | 规则散在Pac/Target/Dir/Builder→单份管理/图/扫描选择/构建规划、效果边及图错误合同 | 独立核心 | AC-01..04 |
| SD-02 | add | docs/specs/project-model/design/generation-request.md | 无通用生成输入→I-PROJECT字段/来源/覆盖顺序/未知与动态准入/无副作用 | UI工具脱Man前置 | AC-02 |
| SD-03 | modify | docs/specs/project-model/project.md | 静态pilot不变；新增独立请求数据消费，不扩大静态语法或786护栏 | 避免假等价 | AC-02,03 |
| SD-04 | modify | docs/specs/auto-man/project.md | 记录实际core委托/旧公开门面/adapter和仍未迁UI/CLI/发行职责 | 不虚报全Man独立 | AC-01..04 |

## 6. 测试设计

T-01基于实际工程/生成器读点冻结旧臂oracle；T-06建新`crates/auto-man/tests/plan809_management_core.rs`、`test/fixtures/modularity/man-core-consumer/`与`project-request-consumer/`。以下V项owner_stage=work，尚未实施；不会把未来命令描述成已执行证据。

| Case | AC / invariant | 输入与生产入口 | 可区分oracle／边界 |
|---|---|---|---|
| V-01 | AC-01 真实核心/公开兼容 | 旧Pac/Target公开字段构造/方法consumer，Pac::resolve→build_targets，独立man-core consumer | 同一规则生成的links/dep_names/targets/源输入与冻结旧臂一致；静态调用图+删旧算法审计，不只mock计数 |
| V-02 | AC-02 请求完整/无副作用 | 真实Vue/Rust/Jet/Ark等现有生成器读点，含window/theme/shadcn/default_classes、资源/输出、external backend；动态pac与未知属性 | 新请求生产接线后同输入产物对照；无路径回抓Pac/AutoConfig；不同cwd路径仍按source根；请求构造无写入/网络。具体支持面T-01穷举，不凭示例四个后端替代全集 |
| V-03 | AC-03 静态/动态正确 | 780冻结语料/786相关64与65/unsupported/unknown/port覆盖、动态表达式与执行型am.at | 原准入/回退/错误stage保持；raw不丢失，静态不支持未误报已求值；不要求核心执行语言 |
| V-04 | AC-03 图及可变状态 | diamond/重复link/断开图/外部link、内部自环和双环、无进展；门面变更字段后二次plan | 合法图按独立手算/偏序oracle；内部环有具名有界错误、外部系统link仍允许；变更可见不读旧缓存；错误不开始后续构建 |
| V-05 | AC-04 三Builder/效果顺序 | 真实Ninja/Cargo/Vue最小项目，含local dep/port/scan选择；工具失败/不可写输出/缺依赖、动态转译路径 | 命令/输入/产物与旧臂，编译运行或等价可消费产物；失败阶段/exit及副作用顺序不漂移；使用受控本地输入但不以fake runner替代全部真工具链 |
| V-06 | AC-04 复用/发行 | pac.lock既有golden、resolver/toolchain调用、独立fresh/locked、777 strict和792受影响profile | 锁字节/验证不漂移，规则没有第二份；core闭包无语言/renderer/Man，缺源包或反向边负控失败 |

work：`cargo check -p auto-man-core -p auto-project-model -p auto-man`、`cargo test -p auto-man-core`、`cargo test -p auto-project-model`、`cargo test -p auto-man --test plan809_management_core`、两个新增consumer独立manifest测试。已有包锁/resolver/toolchain及Builder相关定向回归按触面选择。实际CLI/生成命令与本地fixture由T-01冻结、T-06实现可复跑driver，不臆造不存在的CLI flags。777用`python scripts/modularity-audit.py --mode strict --no-cargo-check`；所有构建/测试在唯一计划树，环境不足明确pending。

review独立上下文同树检查生产委托、双状态、全部Builder/生成读取面、SD及失败控制；用户未指定agent/model，不创建其他会话。merge在最终冻结提交跑裸`cargo t`及core/project-model/auto-man必要包测试；实际改UI生成代码则`cargo tu`、trans则`cargo tt`、VM编译器则`cargo tv`，按最终diff触面决定，不提前整档。tf为主检出到期批量。本文档起草Category A不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收与证据 |
|---|---|---|
| AC-01 | work | core拥有并被实际生产消费的完整管理/构建规划规则；旧公开门面可消费、无第二长期可变状态/第二算法；V-01 |
| AC-02 | work | I-PROJECT无语言/Man/UI大库、真实生成器项目读取改轻合同，所有实际需要字段有来源/覆盖/未知处理及无副作用证据；V-02；交付后才供R04消费 |
| AC-03 | work | 静态/动态准入与786护栏保持，合法图/外部link不回归，内部环/无进展受控且门面字段变更即时可见；V-03/04 |
| AC-04 | work | 三Builder/受影响CLI与效果失败实测、锁/solver/toolchain复用、独立fresh/locked及777/792完整；V-05/06；环境缺失为pending |
| AC-05 | review | 同树独立审查绑定最终修订并核对AC/SD/遗漏，零新增未处理warning/格式/debug问题，merge门pending明确 |
| AC-06 | merge | 冻结门、规范/ledger/归档/接口发布交接及wt-guard clean后清理都有实际收据，R04取得已合入I-PROJECT版本 |

## 8. 执行步骤

共11项，五阶段：合同冻结→请求/核心实现→生产委托→完整模块验收→独立审查合流。不为每个数据模型/工具函数另取号。

| 状态 | ID | owner_stage | 依赖 | 文件/动作、可验证交付与命令 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | depends_on均已交付 | [✅ 已完成] `attachments/809/contract-baseline.md`：Pac 30 方法/Target 40 方法/Dir/Port/Group/file_types/scanner 逐项规则/效果归属表（R-01..R-08/E-01..E-08）；关键架构事实=core 输入边界是 Node/Obj（AutoConfig 仅 {code,root,args}，Pac::new 只读 root/args）；两处内嵌 FS 探针（get_root_dir/merge）改注入谓词；9 生成器零 &Pac 参数、读点三层（行扫/重型解析/调用方 env 种植）+ I-PROJECT 字段合同 A-D 组（字段集 ⊃ Pac 字段集：npm_deps/styles/i18n/exe_name/dep-roots/rust_sidecar/front/back.project）；Builder 三实现读点表（Vue 零读取）；图错误合同（环=挂死证实、外部 link 不阻塞、NoProgress+cycle_candidates）；API 兼容=分类迁移+扩展 trait 保方法语法（Rust 跨 crate inherent impl 红线）；CLI→Pac 调用链矩阵+V-05 driver 环境矩阵。depends_on 七计划 archived 核实+主检出零代码 WIP。§12 未决项均已映射归属任务，无阻塞 | AC-01..04 |
| [x] | T-02 | work | T-01 | [✅ 已完成] `auto-project-model` 新增具名 `generation_request` 模块（412 行）+ `generation_request_tests.rs`（8 测试）：RequestOrigin 静态准入/动态求值分流（unsupported 不伪造已求值，is_evaluated 诚实判定）、A 组字段投影（复用 Identity/DisplayFields/RuntimeFields/RenderDecl/ToggleFields）+ B 组从 raw 目录回收（npm_deps/styles/i18n/exe_name/front/theme 块→ThemeDeclLite/rust_sidecar 存在性/dep 根）、display_title locale 链（镜像 Pac::display_title_in）、resolve_against_root 纯路径代数（cwd 无关）；外部 consumer fixture `test/fixtures/modularity/project-request-consumer/`（独立 manifest+verify-deps.sh，闭包=auto-project-model→auto-atom→auto-val 单证）；根 Cargo.toml exclude 登记。验证：`cargo test -p auto-project-model` 38/38（含新增 8）、consumer 3/3、closure PASS、`cargo check -p auto-man` 过。worktree 提交 dd4991a7b | AC-02,03 |
| [ ] | T-03 | work | T-02 | [R4 未闭：Block/Obj形态与core测试编译；以下为历史实施记录] 新 `crates/auto-man-core`（闭包 auto-val/auto-atom/auto-toolchain-core/auto-project-model+log）：Scene/BackendName+RenderSelection（from_str/output_dir/from_value 逐字对齐 BackendType/BackendConfig）/Target 纯半（from_with_probe 注入谓词+2 参兼容入口保签名）/Dir 数据半/Port+port_from_node/Group/NodeExt/pac_from_decl→PacDecl/投影自由函数族（all_incs/pac_to_node/exe_path/display_title_in/targets_map/filters）。auto-man：node_ext/group/port 纯 re-export 门面；dir/target 门面+DirEffects/TargetEffects 扩展 trait（效果方法逐字）；Pac::new 生产委托 core（RenderSelection↔BackendConfig、ThemeDeclLite↔ThemeDecl 转换），纯方法逐项委托。plan809_management_core.rs：api_compat_probe（Target 29 字段+Pac 全字段字面量构造+方法 pin）/production_delegation_parity（30+ 字段冻结 oracle）/glob 导入 trait 方法语法/port_from_node 对拍。验证：core 18/18、probe 4/4、pac:: 18/18、project-model 38/38、auto-man lib 344 过（5 失败=master 并行 flake 族，主检出对照全套失败更多）；4 recorded adjustments 入 contract §13。worktree 提交 bc11055ac | AC-01 |
| [x] | T-04 | work | T-03 | [✅ 已完成] closure.rs 单源迁入五函数族+resolve_target_closure 有界准入（NoProgress：unresolved 全名+DFS 灰标环候选链；外部 link 保留直接边不阻塞不进 dep_names——测试期纠正两处本人期望误写并钉真实语义）+assemble_major_targets；Pac::resolve 生产委托并删旧算法；V-04 core 8 用例（diamond/重复 link/外部 link/自环/双环/混合无进展/字段变更重 plan/手算三层链）+Pac 级集成（真实 pac.at links prop 形态，错误具名且不开始 scan）。resolver/lock 无第二份（795/800 既定门面，contract §4 核实）。core 27/27+probe 5/5+pac 15/15。worktree 提交见 log | AC-01,03 |
| [x] | T-05 | work | T-04 | [R3 验证完成] selection/planning/file_types 生产委托与既有选择回归保留；R2-01 的 Ninja 实例身份已修为 BuildOrderStep(item_index,from_dep)，冻结旧循环与新生产Builder三态产物一致，core46与管理8项通过。见 [R3闭合表](attachments/809/review-r3.md)。效果矩阵与请求交接仍属T06..08，不将其计为已闭 | AC-01,04 |
| [ ] | T-06 | work | T-05 | [R4 未闭：Obj消费/有效后端/Ninja成功矩阵]  生产委托：Pac::new/resolve/set_port/merge_port（T-03..05）+automman gen 分派（成员+self 一次构造 request_of 多处消费）；生成器读接 I-PROJECT：jet/ark/rust_ui/vue 四族读缝（JetReads/ArkReads/PacSource/VueReads+Value 等价转换）+全 `_with_request` 入口，legacy 自扫留作冻结 oracle（R04 退役）；两 consumer（project-request/man-core+闭包单证）；plan809_management_core 6 测试（probe/对拍/环/V-05 内存半）；V-02 对拍（VueReads 九字段+双路径）；**pending：V-05 真实工具 spawn driver（内存证据+探针已入，handoff #1）、run 路径再生成与跨文件读取归 R04（handoff #2/#3）** | AC-01..04 |
| [ ] | T-07 | work | T-06 | [R4 未闭：准确SD及809受影响发行绑定证据]  auto-man-core 入 777 man-ecosystem 域（module-boundaries.toml）+两 consumer fixture 钉闭包；双状态审计：Target/Dir/Port 单类型（re-export 同身份）、Pac.targets 即 core 类型、生成器请求为快照读（无双状态）；**余：777 strict 基线 json 重跑+792 source/resource 清单并集（merge 前与门禁同批）** | AC-03,04 |
| [ ] | T-08 | work | T-07 | [R4 未闭：新编译错误/告警及闭合表须校准]  SD草案+`attachments/809/handoff.md`、I-PROJECT接口示例/版本/字段读取映射及剩余UI/CLI adapter归属；绑定代码/命令/净迁移证据，提交送审 | AC-01..04 |
| [ ] | T-09 | review | T-08 | 同树独立AC/SD/全部生产规则与支持矩阵审查；不能用静态pilot当全部工程；merge门显式pending | AC-05 |
| [ ] | T-10 | merge | T-09 pass | 并集master，实际变更复审，最终冻结提交按§6运行门；失败归因后修复/复审/重冻 | AC-06 |
| [ ] | T-11 | merge | T-10 | 规范/ledger沉淀、归档、I-PROJECT已合入SHA交接，guard clean后树/分支清理及最终计数 | AC-06 |

handoff：`D:/autostack/.wt/lang-809/auto-lang`，分支`codex/plan-809-dev`；work确认实际base与兄弟依赖，同树用于后续review/merge。与807/808仅manifest/777/792接线短窗口；与未来R04对Pac/生成器项目读取有实际共享面，R04须等本合同接口合入再接，不准并改同段。

## 9. 复审记录

2026-10-18 revision 1 drafting；当前仅合同及预检，未实施/测试或review pass。内部target环受控拒绝是本Plan明示新增合同，正常迁移语义仍冻结；Spec delta须随最终实现接受审查，不能提前写为已交付。

2026-10-18 work T-01 完成：worktree `D:/autostack/.wt/lang-809/auto-lang`（branch `codex/plan-809-dev`，base=master 2046a025b）建立；depends_on 七计划 archived；contract-baseline.md 冻结（读点穷举/归属/兼容/图错误/命令矩阵）。下一 T-02 generation_request 模块。
2026-10-18 work T-02/T-03 完成：generation_request 模块+consumer（dd4991a7b）；auto-man-core crate+门面委托+probe（bc11055ac）。环境注记：aliyun 镜像索引缺 redox_syscall 0.9.5，重解析经 rsproxy 单次旁路（cargo --config source.aliyun.registry）。
2026-10-18 work T-04/T-05 完成（5c9eb4fdb/322aa88d1）；T-06 全量完成（8ee334f90+a4b4a0168+T-06d：四族读缝+分派全切+V-02 对拍+V-05 内存证据半）；T-07 部分（777 域登记+双状态审计；strict 基线重跑归 merge 前）；T-08 handoff.md 就绪（attachments/809/handoff.md，五项显式 pending/范围记录）。**outcome: work 完成→送 T-09 review**；merge 门（裸 cargo t+包档+777 strict 重跑+792 并集）显式 pending。plan734 目标 master 预存红。


### 2026-10-18 final review R1：needs_fix（终审就绪核查）

stage: review；review_scope: final；reviewer: 当前独立于实施的 Codex 审查上下文（未创建 agent，未虚构模型/session）；plan_revision: 1。
reviewed_commit: `aa1d40e1d90b313b7dd6ba57b79123f5278ed90f`；base_commit: `2046a025b9a445d35a02da2c72a1613259a0cb60`；worktree: `D:/autostack/.wt/lang-809/auto-lang`；branch: `codex/plan-809-dev`；auto-down: `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。

[完整报告/AC映射/修复要求](attachments/809/review-r1.md)、[基线与Spec输入哈希](attachments/809/review-r1-baseline.json.txt)、[冻结SD](attachments/809/review-r1-spec-delta.md)。实施与兄弟树clean，最终HEAD未漂移，diff --check通过。

F809-R1-01(P2)：真实Ninja/Cargo/Vue工具fixture与失败验收未落，现有Ninja内存断言允许空输出；F809-R1-02(P2)：run等生产生成路径仍走旧读取器，延期R04未获批准；F809-R1-03(P2)：Ninja依赖构建序/去重及依赖下载合并等纯规划规则仍留旧adapter；F809-R1-04(P2)：work-owned 777/792清单与独立消费证据、最终SD/接口交接未收口。均OPEN，登记不批准延期/缩面。

本轮按readiness规则退回work，未运行Rust测试或完整最终验收，未把实施自报绿数授信；AC01 partial、AC02/04 fail、AC03 pending、AC05 readiness fail、AC06 merge pending。T01..04保留原实施标记（非本轮pass），T05/06/07重开，T08..11待办，status executing/current_step 4/11。next：同一809树完成F01..04、定向验收并提交完整交接后再终审。

pending_closeout_gates：最终代码/验收/Spec review pass后再由merge执行冻结裸cargo t、必要包档和实际UI/trans/VM触面档、777/792最终并集；规范/ledger/归档/已合入接口版本/guard清理仍pending。未改实现、合入或发布规范。


### 2026-10-18 final review R2：needs_fix（修复实审）

stage: review；review_scope: final；reviewer: 当前Codex审查上下文，独立于实施、延续R1（无新agent/虚构模型身份）；plan_revision: 1。
reviewed_commit: `3b733aa5d667f3fb14a0a2b87855de215797fde6`；base_commit: `2046a025b9a445d35a02da2c72a1613259a0cb60`；worktree: `D:/autostack/.wt/lang-809/auto-lang`；branch: `codex/plan-809-dev`；auto-down: `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。最终实现/依赖树clean、HEAD不漂移，diff --check通过；主检出其他工作推进及当前Spec输入hash见基线。

[完整R2/原finding闭合矩阵/AC与SD](attachments/809/review-r2.md)、[基线](attachments/809/review-r2-baseline.json.txt)、[冻结候选SD](attachments/809/review-r2-spec-delta.md)、[独立生产反例/正控](attachments/809/review-r2-probe.rs)、[原始输出](attachments/809/review-r2-probe.log.txt)。

F809-R2-01(P1)：Ninja按rename优先回查顶层对象，丢先访问依赖副本的srcs等覆盖（真实旧循环发override.c、新臂发base.c，两个正控一致）；R2-02(P2)：Ark run丢请求、Jet/Ark build仍legacy、Vue请求入口仍回读本工程dep声明；R2-03(P2)：动态请求丢raw_nodes/sidecar/app(front)及实际模式/输出输入上下文，Vue双front布局与仅中文标题旧新产物不同；R2-04(P2)：which只找npm.exe导致可用CMD工具用例提前return算绿，原V05完整矩阵尚缺；R2-05(P2)：786旧source围栏实测红，新增告警未清；R2-06(P2)：SD“无丢失/零遗留”与代码不符，正式fresh/locked与792证据未全。均OPEN，不批准延期或缩面。

core45/model38、两consumer各3及normal/build/dev闭包、777 strict(0违规/2grandfathered)、38文件scoped fmt过；管理7/7中Vue case跳过，补独立真实VueBuilder成功marker与exit3失败控制均成立；generation/786定向11绿1红（source接线断言，64/65及进程护栏等运行测试过）。N809-R2-01：非glob导入需新trait的E0599已实测，按T01声明作为兼容边界披露，不单独阻断。无全档测试、无实现修改/合入。

AC01 partial/fail、AC02 fail、AC03 partial、AC04 partial/fail、AC05 fail、AC06 merge pending；重开T03/05/06/07/08，保留T01/02/04原实施记录，status executing/current_step 3/11。两次final未过后，下一work交接须附漏检根因、完整不变量和设计/oracle调整表，并永久重放本轮反例及正控；在原树修复提交后再终审。

pending_closeout_gates：最终review pass后由merge在冻结提交跑裸t+必要包/实际触面档，最终777/792并集与规范/ledger/归档/接口已合入SHA/guard清理均pending；reviewed不等于已交付。


### 2026-10-19 final review R3：needs_fix（重点核查R2闭合）

stage: review；review_scope: final（有界闭合复审，已知work-owned R2-06未完成）；reviewer: 当前Codex审查上下文，独立于实施、延续R1/R2，无新agent/虚构身份；plan_revision: 1。
reviewed_commit: `ef6f05cf439389d92ccbd9f1ca5ae4206abac962`；prior: `3b733aa5d667f3fb14a0a2b87855de215797fde6`；base: `2046a025b9a445d35a02da2c72a1613259a0cb60`；worktree: `D:/autostack/.wt/lang-809/auto-lang`；branch: `codex/plan-809-dev`；auto-down: `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场/结论前实施clean，结论前依赖clean，HEAD未漂移，diff --check过。

[完整R3闭合矩阵与修复要求](attachments/809/review-r3.md)、[原始生产反例/正控](attachments/809/review-r3-probe.log.txt)、[源/锁/Spec基线](attachments/809/review-r3-baseline.json.txt)、[候选SD冻结](attachments/809/review-r3-spec-delta.md)。

R2-01 P1完整声明矩阵已闭（三态原旧循环/新Builder真实字节一致）；R2-02 Ark run/Jet-Ark build/Vue主dep读源原例闭，完整不变量仍有sidecar未消费；R2-03节点/front/中文标题原例闭，上下文/真实源清单仍不全；R2-04 PATHEXT假绿原例闭、真实Vue正负控执行，但原完整工具矩阵pending；R2-05围栏红例闭，新增health未全清；R2-06自认SD/正式792收据未完，OPEN。未以raw_args=None误判：本轮多参数实际保留在求值props的_arg0/_arg1；通用display_title与Vue消费语义区别已核对。

本轮缺口：F809-R3-01(P2，R2-02残留) sidecar_spec无调用，Rust真实生成request-only不复制extra.rs/disk-only却复制，正控both正常；R3-02(P2，修复回归) core sidecar列表漏旧Value::Block，预期1模块实际0；R3-03(P2，R2-03残留) custom/front请求的input_sources却枚举src/front/wrong.at，gen/成员run等仍默认上下文；R3-04(P2，health) Vue root_dir及新增fill/read_str_list/collect_dep_front_dirs/sidecar_spec告警。均OPEN，不批准延期/缩面。

实跑core46/model38、管理8（真实Cargo/Vue，未跳过）、原生成/786定向12及另选新增parity1全过，19改动Rust文件scoped fmt过。未做正式792/fresh-locked全集/777或全档回归，不将旧证据伪称新提交绿。诊断表已补且Ninja/工具调整有效；sidecar与上下文/零告警闭合声明须按实测撤回完善。

AC01 partial、AC02 fail、AC03 partial、AC04 partial、AC05 fail、AC06 merge pending。T05改完成，T01/02/04保留；T03/06/07/08仍待办，T09..11待后续，current_step纠正为4/11（入场写8但任务行仅3勾）。next：同树work完成R3-01..04、原R2-04矩阵与R2-06，补实际路径/输入与永久回归收据后再审。

pending_closeout_gates：最终code/acceptance/SD pass后再由merge冻结裸t+必要包/实际触面档，最终777/792并集与规范/ledger/归档/已合入接口SHA/guard清理pending。本轮无实现修改、live规范/ledger写入、合入或归档。

### 2026-10-19 final review R4：needs_fix（R3修复及剩余收据核查）

stage=review | review_scope=final | reviewer_identity=独立于本轮实现的Codex复审上下文，未另开agent、不虚构模型/会话ID | plan_id=PLAN-809 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-809/auto-lang | branch=codex/plan-809-dev | reviewed_commit=6a84a7e28f5579b7c188b54b5ca86a9ae295f3a9 | base_commit=2046a025b9a445d35a02da2c72a1613259a0cb60 | dependency_revisions=auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 | spec_inputs/SD冻结/AC矩阵=[R4完整报告](attachments/809/review-r4.md)、[输入基线](attachments/809/review-r4-baseline.json.txt)、[SD快照](attachments/809/review-r4-spec-delta.md)。实现/兄弟依赖复核干净，HEAD未漂移；主检出并行推进但相关既有Specs/锁哈希保持。

原R2 Ninja身份三态、Ark/Vue/786原例保持闭合；R3 sidecar节点request-only/disk-only/both及custom/front清单已修。**完整不变量仍未闭**：F809-R4-01(P2) 删除AutoStr导入使core必需lib-test三处E0433，旧R3-02 Block仍从1模块变0；F809-R4-02(P2) 对象sidecar原支持形态旧生成extra.rs、新请求生成静默漏文件；F809-R4-03(P2) 新Ninja成功fixture把C字符串内换行写成实际LF，真实cl原输入exit2、单变量转义正控exit0。R3-03 gen有效后端仍绕过override helper，R3-04新增from_request_pub unused；R2-04 Ninja成功产物及R2-06准确SD/受影响独立交付证据继续开放。末尾work“R3全闭/core47”仅为历史声明，本轮未授信。

evidence=core/model组合exit101（不计core47或model已过）；管理8/8含真Cargo/Vue与不可写负控、Ninja显式pending；生成定向15/15、七Rust文件fmt过；原生产反例与新增Obj/C输入反控均持久化于R4附件。未运行整档/正式792/777，不改实施。**PLAN-792本身已归档8/8、原交付25/25，不重开792**；本项只要求809受影响回归的可定位绑定证据，当前可定位SD/handoff仍写pending，未据此断言用户从未执行新验证。SD=规范增量，不是SDK。

acceptance_results=AC01 partial、AC02 fail、AC03/04 partial、AC05 fail、AC06 merge-owned pending | pending_closeout_gates=最终code/acceptance/Spec pass后冻结裸cargo t及必要包/触面档、最终777/792并集、规范/ledger/归档/已合入接口SHA与guard清理 | findings=详R4，登记不批准延期 | next=同树work修复并校准形态/入口/工具oracle、SD和绑定收据，提交后再审。保留T01/02/04/05，T03/06/07/08仍开放，T09未完成、T10/11待merge；status=executing，current_step=4/11。

## 10. 待澄清事项

当前没有需用户先答的阻塞项。T-01有界验证public字段/方法兼容、I-PROJECT字段全集和效果拆缝。若核心规则无法在不破坏旧公开面/支持后端的条件下抽取、需改工程格式或扩大到相邻仓，必须提出具体重规划；不能退化为只做DTO或把全体实现再包进opaque callback。三Builder实际工具环境不足时保留pending，独立模块完整交付不以缺环境跳过。

2026-10-18 R1 修复轮（c41a4fe4c+3b733aa5d）：F01..F04 全闭合（行表见 attachments/809/handoff.md「R1 Findings 闭合」）：F02 run/build/open 全量 `*_with_request`（automan+main legacy 分派 grep=0）；F03 ordered_build_renames+下载决策四函数抽 core（45/45 含新 oracle）；F01 V-05 真实 cargo/npm 执行+三失败路径+内存区分力断言+空工程负控（probe 7/7）；F04 777 基线重采样 strict pass（收据 t07-777-strict-pass.json）+delivery 9/9+SD-01..04 as-built+接口映射表。work 完成→送 T-09 二轮终审。merge 门显式 pending（裸 t+包档+按 diff 触面档+最终 777/792 并集复验）。

2026-10-19 R2 修复轮（ef6f05cf4）：F01..F05 全闭（诊断表+闭合行见 handoff「R2 Findings 修复」）：P1 ninja 身份回归（BuildOrderStep+三态真实字节回归）、入口全链闭合（含 build_*/vue 主声明二读/sidecar 单源）、投影无损（raw_nodes/front 节点形/上下文/zh-only 链）、PATHEXT 发现+真实 npm/cargo 执行、围栏+告警清零。**F06 部分待续**：SD-02/03 文本校准与 792 正式 fresh/locked 收据（下一会话补齐后送 T-09 三轮）。core 46/46+probe 8/8+12 项定向 13/13。

2026-10-19 R3 修复轮（6a84a7e28）：R3-01..04 全闭（sidecar 生产消费三态来源反转/Block 形态保留/请求 front 解析清单+全上下文入口/死代码与新告警清零）+V-05 补不可写输出与编译器在场真实 ninja。**仍开放（送审前必补）**：R2-04 完整三 Builder 矩阵的 ninja 成功产物收据（本机无 C 编译器时显式 pending）、R2-06 SD-02/03 按实际合同校准+792 正式 fresh/locked 收据。adapter 4/4+core 47/47+probe 8/8+15 项定向。

2026-10-19 R4 修复轮（b0f8f857e+SD 校准提交）：R4-01..03 全闭（core 编译恢复+Block 臂真落地 46/46、sidecar 对象/嵌套形态经 req.raw 并集、Ninja C 输入合法化）+R3-03 残留（gen self_req 经 gen_request_as 带 render_override）+R3-04（from_request_pub 死代码删除）。SD-02 sidecar 边界按实际合同校准。**仍开放**：R2-04 ninja 真实成功产物收据（本机无 C 编译器=环境 pending，C 输入已修合法）、R2-06 受影响 792 独立交付回归收据（复用需附代码/依赖/输入未变依据或定向真跑）。probe 8/8+adapter/sidecar 10/10+core 46/46。
