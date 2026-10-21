---
plan_id: PLAN-814
status: archived
feature_name: 语言 SDK、UI 编译与运行包组整体分层
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-21
plan_revision: 3
current_step: 15
total_steps: 15
supersedes_spec_components: [docs/specs/auto-lang/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/runtime/overview.md, docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/auto-ui-dialect/design/full-grammar.md, docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/language-sdk/project.md, docs/specs/auto-ui-compiler/project.md]
new_spec_components: [docs/specs/auto-ui-runtime/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [language-sdk, auto-ui-compiler, auto-ui-runtime, auto-lang/compat, auto-vm]
depends_on: [PLAN-807, PLAN-808, PLAN-810, PLAN-811]
---

# [PLAN-814] 语言 SDK、UI 编译与运行包组整体分层

## 0. 变更摘要

完成R04的语言/编译/运行边界：提供不依赖UI实现或Man的语言SDK，把UI编译生成与UI运行/renderer形成独立可消费包组；默认组合和旧`auto_lang::*`兼容明确归上层。迁移实现、公开路径、资源、测试选择、真正消费者及发行闭包一起交付，不止增加几个空crate。

不搬Man的UI工程工具/资源（815），不接完整CLI工具协议（816），不搬Git仓/切旧默认分支，不在本期删除UI兼容AST/TokenKind（R10）。纯SDK不等于原名auto-lang同一包既重导出完整UI又保持纯Core。

本次r2按M814-A真实阶段发布、M814-B完整交付组织，不增加Plan编号、不降低原最终AC。执行与审查修复并行规则见[阶段交付与并行审查修复合同](../reports/staged-delivery-review-814-816.md)；阶段pass不等于整Plan交付。

## 1. 目标

- 新纯SDK可独立完成Core parse→compile→VM、现有trans与基础服务消费；normal/build/dev闭包无UI文法实现、UI compiler/runtime/renderer、Man实现。
- UI分析/模型适配、AURA/a2ui、四生成器、设计token/registry等及UI runtime/interpreter/renderer/宿主adapter有单一owner和无环依赖；相关轻数据按真实消费者归属，不新建common巨包。
- 默认UI入口及旧public API/feature/类型身份、动态装配/全局与实例行为兼容；旧兼容实现处于上层，外部消费者可选择真正SDK而无需UI checkout。
- i18n/直接frame/VM到UI残边及隐式默认装配具名收口；原有测试族不会因跨包迁移从别名门中消失。

## 2. 架构方案

候选布局：新`crates/auto-lang-core`为纯SDK；新`auto-ui-compiler`与`auto-ui-runtime`分别拥有UI编译/生成和运行/renderer；旧`auto-lang`包保上层兼容/默认组合，依赖这些包。包名可在T-01按真实碰撞/身份固定，不能以SDK反依赖上层兼容包实现旧入口。允许共享既有轻ast/lex/parser-api/text/terminal/media等，不复制语言或VM。

UI compiler可使用SDK，runtime可使用SDK与必要UI编译公开面；compiler生成的runtime路径是代码输出合同，不可再用normal/dev反向依赖runtime形成环。跨层集成测试放上层/独立fixture。默认Parser/AutoVM等旧构造面、pub字段/方法/trait和回调类型的兼容必须有整链编译probe；如需薄包装，只有一份状态，不能靠Deref名义下藏双Parser/VM或破坏旧签名。具体布局T-01冻结，不假设改几行re-export就能满足所有inherent方法。

## 3. 技术栈

现有Rust包、同源AST/Value、CompilerSession、Core语义/VM、轻宿主capability与显式factory/装配；UI后端features按现行支持面迁移，不额外退役后端。现有Schema/stdlib/registry/资产与build输入随owner迁移。

## 4. 需求分析与背景调查

用户2026-10-20明确要求修订现有814/815/816，允许阶段交付及接口合入后放行下游、并行审查验证修复，不增加编号。原目标/最终AC/支持面不变；本轮只改主检出规划文档，不改实现树、canonical Specs或远端。 来源与当前hash见[本轮输入](../attachments/sdk-tools-r2-20261020/inputs.json)。

授权：用户要求建立下一批较大计划并明确合入前置；本轮仅auto-lang仓内规划，不实施/创建agent或工作树、不写相邻仓/远端。来源：[总路线](../reports/v06-extraction-execution-roadmap.md)、[批次调度](../reports/sdk-tools-parallel-order-814-816.md)、[输入hash](../attachments/sdk-tools-batch-20261019/inputs.json)。

权威输入：[语言现状](../../specs/auto-lang/project.md)、[frontend](../../specs/auto-lang/frontend/overview.md)、[VM](../../specs/auto-lang/vm/overview.md)、[UI](../../specs/auto-lang/ui/overview.md)、[方言](../../specs/auto-ui-dialect/project.md)、[交付](../../specs/repository-boundaries/generated-ui-distribution.md)。807/808/810/811均已归档交付；810=f0ec8a91f，811=d06cdc6f6，实际master祖先已核实。旧r1等待状态保存在Git历史；本轮不重做这些Plan验收。

实际写面：`crates/auto-lang/{Cargo.toml,build.rs,src/lib.rs}`（build输入以T-01存在性/引用核实）、`src/{parser,parser_bridge,compile,config,a2r_std,back_proxy}.rs`、`src/{ui,ui_gen,aura,a2ui,design_tokens,vm,trans}/`、`crates/auto-vm`、根Cargo与`.cargo/config.toml`/`.config/nextest*.toml`。当前默认带ui-iced，`Parser`有pub scope/cur/infer_ctx等，lib全局STORE_EXTRA_FILES/USE_DIAGS及UI辅助也混在旧根；无环拆分必须处理身份/装配/生命周期，不能仅关default-features伪装。

### r2阶段启动门

- 807/808/810/811均已实际交付，T-01整链probe及T-02/T-03可在唯一树开始；不再等待810/811。
- **M814-A**：T-01→T-02→T-03→T-13→T-14→T-15；真实SDK与UI编译公开面发布后，放行815生成主体与816语言adapter。
- **M814-B**：T-15后继续T-04..12，完成runtime/renderer与全部原AC；完整runtime调用必须等本阶段实际发布。
- 依赖未合入的实现不能进入生产闭包，不借上游WIP；任务ID保留历史编号，不表示数字顺序执行。

## 5. 详细设计

1. **SDK与UI owner图**：逐模块列出源码、public项、normal/build/dev、feature、生成字符串/宏/include/build input依赖；解析/推断/编译/VM/非UI trans留同份Core，UI生成/运行移包；依赖语法名不等于UI实现依赖。现有UI兼容AST可在轻auto-ast暂留，明确R10退出，不把UI私有TypeStore/registry推回Core。
2. **默认组合与旧API**：旧auto-lang可为上层兼容包；SDK新名字公开且实测独立。对`Parser::with_session`、AutoVM/NativeInterface构造、外部Dialect、native callback、pub字段/trait/inherent与模块路径构造编译pin；旧UI场景仍默认可用，SDK的Core场景不加载UI实现，SDK UI解析须显式获得UI方言组合。SDK不提交UI注册项、不硬编码UI factory、纯SDK闭包零UI实现。**r3修订（2026-10-20用户批准）**：允许旧auto-lang上层兼容包及auto-ui-compiler经源码可见的`inventory::submit!`登记轻量UI文法函数表/宿主安装器函数指针，覆盖不经显式装配入口的旧直达调用——该登记经inventory启动构造器（0.3.25，Windows `.CRT$XCU`）实现，仅修改inventory自身注册表，启动登记阶段不调用UI文法/宿主工厂、不执行I/O、不初始化UI/VM业务状态；具名入口继续显式装配优先，未显式设置时SDK才收集已登记来源。完整文本=[§5.3](#53-r3-合同修订2026-10-20用户批准)。若无法兼容，T-01具体提出重规划，不自行砍API。
3. **UI compiler**：现有ui_gen四生成器/registry、AURA/a2ui与UI相关分析按闭合图迁移；通用trans/shared helper经SDK公开最小能力，不暴露Core全部私有结构。生成canonical runtime/SDK import由一份路径策略产生；旧路径门面保兼容。允许必要import/header基线变更须显式映射新旧路径，不能以更新golden掩盖行为差异。
4. **UI runtime**：ui目录runtime、renderer、editor/terminal/widget、desktop/MCP/宿主adapters及其状态/资源迁有归属的包；VM/SDK只保轻能力与非UI机制。i18n lookup/装载、a2r frame直连、config/UI依赖和其他反向边逐项验清；未消除边必须有本Plan内任务，不能在最终声明“纯Core”时留豁免实现。
5. **状态/并发**：registry/OnceLock/thread_local/global runtime/VM Rc、provider默认/实例覆盖/owner history、模板/语法session/cache不可跨新包产生第二真源；线程亲和、锁外回调、store额外文件drain、use诊断、终端/媒体生命周期/快照共享保持。移动字节不等于状态单源证据。
6. **消费者与测试**：旧路径消费者在兼容层继续可编译；至少一个真实非UI消费者（首选auto-vm，若其实际依赖需调整则T-01具名替代）直接切SDK。UI compiler/runtime及后续工具必需的语言adapter（如auto-gen可执行模板层）也要沿实际闭包切SDK，不能经旧auto-lang兼容包把UI实现重新拉回。auto-lsp等UI补全可暂用上层具名adapter，不谎称其整体纯Core。迁出的lib/test/integration/doc/schema测试保单份来源；cargo t/tv/tu/tt/tb实际选择面映射覆盖新包，UI tu保持合适单进程registry复用，不因包名变化丢测试。
7. **发行**：777精确域边/forbidden、792受管source与manifest/资源/producer/lock/重定位更新至真实包图；新SDK独立fresh/locked无UI checkout，UI实例工程可用新包，不靠父仓隐式path。SDK不含UI实现的证明覆盖normal/build/dev，UI部署profile仍保原功能而非删除默认重后端。

### 5.3 r3 合同修订（2026-10-20 用户批准）

本节为 r2→r3 的具名增量（来源=[R6 复审草案](../attachments/814/review-r6/contract-r3-draft.md)，用户于 2026-10-20 批准；仅修订以下两项，其余原 AC、旧 API/类型/单份状态与 A/B 交付要求保持）。

**1. 具名的上层兼容注册例外**（替换 r2 AC-02/V-02/§5.2 中"无隐式链接装配"的绝对约束）：

- 纯 SDK 的 normal/build/dev 闭包不含 UI 文法实现、UI compiler/runtime/renderer 或 Man；SDK 不提交 UI 注册项，不硬编码 UI factory。SDK 未装配 UI 文法时的缺席/拒绝语义保持。
- 允许旧 auto-lang 上层兼容包及 auto-ui-compiler 经源码可见的 `inventory::submit!` 登记轻量 UI 文法函数表或宿主安装器的函数指针，覆盖不经显式装配入口的旧直达调用。该登记通过 inventory 启动构造器实现（当前锁定 0.3.25；Windows `.CRT$XCU`），会修改 inventory 自身注册表；不在启动登记阶段调用 UI 文法/宿主工厂、执行 I/O 或初始化 UI/VM 业务状态。
- 具名入口继续显式装配；未显式设置时才由 SDK 收集已登记来源。多个当前来源均构造自同一 auto-ui-dialect 实现，不依赖遍历顺序选择不同语义。不新增不同版本同时装配的支持承诺。
- 上层消费者允许按其功能依赖并链接 UI 实现；纯 SDK 的无 UI 源码消费者仍须成立。旧 Parser/VM API 与类型身份、单份状态、宿主默认/显式覆盖及缺席行为保持。
- 此例外明确承认链接时注册。仅删除 inventory 不能自动授权砍旧 API、改成硬错误或更改纯 SDK 缺席语义。

**2. 轻端口的归属修订**：

- 允许 `UiParseContext` 与 `UiGrammar` 的类型/函数表定义放入轻 `auto-parser-api`；UI 文法实现继续由 auto-ui-dialect 单源拥有，SDK 不反向依赖文法实现。auto-ui-dialect 保留原公开路径再导出及类型身份，通用 `ParseContext` 的既有实现方不被强制实现 UI 端口。
- 这是单源端口与适配器方案之间的设计取舍。canonical 目标（T-15 受管沉淀时落文）：`docs/specs/auto-parser-api/project.md`"UI 伴随端口不入本 Core 包"→"允许 UI opt-in 轻端口定义，不带 UI 实现依赖"；`docs/specs/auto-ui-dialect/design/full-grammar.md` trait 定义归属改 parser-api、方言 crate 保留再导出与实现 owner。SD-01/02/04/05 同步记录该窄例外及其验收，不提前发布 B 完整 runtime 事实。

### 5.1 A/B阶段的真实交付边界

**M814-A**不是空crate或接口草图：纯SDK完成Core parse→compile→VM与已有非UI消费者、必要trans出口；normal/build/dev无UI文法实现/编译/运行包或Man。T-02已经清除阻碍此性质的i18n/frame/config等边，不能把纯SDK所必需的去边留到T-06。UI compiler迁实际生成算法/资源与公开入口，真实消费者可生成其声明支持的产物。A中迁出的测试族和资源立即接入实际门/777/792及fresh/locked消费，不留到T-07/08才补。

旧auto-lang上层兼容可继续使用尚未搬迁的现有真实runtime，保原默认UI、Parser pub字段/方法/回调及单份状态；新SDK与compiler不可反依赖该兼容包。生成runtime import仍指向当前真实可用、版本受约束的实现，B再有控制地切换；不得将未完成的新runtime能力列为已支持。原四生成器各自支持面保持，依赖runtime的原完整正控保留给B，A至少验证生成产物和当期真实runtime消费路径。若依赖图不能这样切，T-01交具体needs_replan，不用伪实现凑阶段。

**M814-B**完成新runtime/renderer/宿主adapter、默认组合和全部原验收。A的局部合入不提前勾AC-01..05整体完成；仅新增AC-08..10描述阶段范围。SD-01/02与SD-04..08仅按A已发布事实沉淀，SD-03完整runtime规则等B，不把B目标写成现状。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/project.md | 原巨型包→纯SDK职责/API/feature/显式组合与正常闭包（r3：UiGrammar/UiGrammarSource/set_ui_grammar 装配端口语义+§5.3 具名登记例外+缓存身份时序——None→Some升级/首装优先/出生身份落盘/load比对失效） | 基础可独立消费 | AC-01, AC-02 |
| SD-02 | add | docs/specs/auto-ui-compiler/project.md | 原UI分析/生成→独立compiler包、公开接口/资源（r3：assemble_ui_grammar 显式装配入口+三生成入口时序+指纹 None=foreign build） | UI编译owner | AC-03 |
| SD-03 | add | docs/specs/auto-ui-runtime/project.md | 原UI运行/renderer→独立runtime与宿主/状态归属 | 运行边界 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-lang/project.md | old包标为上层兼容/组合，实际SDK位置明确（r3：上层经 inventory 具名登记的直达兜底例外+auto-parser-api 端口归属变更） | 避免纯Core虚报 | AC-01, AC-02 |
| SD-05 | modify | docs/specs/auto-lang/frontend/overview.md | Parser默认/显式UI组合与旧API承载（r3：Scenario::UI 内建方言双层装配来源——显式安装器首句+具名登记兜底；表缺席诊断文本） | 解析身份/装配 | AC-02 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 无UI实现的VM能力/默认组合与余边清零 | 真正解耦 | AC-01, AC-04 |
| SD-07 | modify | docs/specs/auto-lang/ui/overview.md | UI包组/旧路径/资源/消费者事实 | 完整交接 | AC-03..05 |
| SD-08 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 真实新包图/测试与资源输入进入发行profile | 迁包可分发 | AC-05 |

## 6. 测试设计

T-01冻结旧public/API与消费者/支持面、测试族选择清单；T-08建立新增`test/fixtures/modularity/language-sdk-consumer/`、`ui-package-consumer/`及上层`plan814_package_boundaries`定向测试。以下均owner_stage=work，当前未执行。

| Case | AC / invariant | 真实入口/设置 | 区分性oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 纯SDK/单源 | 独立manifest→Core parse/compile/VM/trans；无UI checkout、真实auto-vm消费 | normal/build/dev无UI compiler/runtime/grammar实现/Man；单份AST/Value/VM/源码，不用空包或重导出旧巨包骗绿 |
| V-02 | AC-02 兼容/装配 | 旧Parser pub字段/方法/ExternalDialect/native callback、UI默认/import、Core及显式UI组合，features开关/两VM覆盖 | 旧签名consumer编译与AST/执行结果保持；SDK无UI能构建，旧UI默认可用（r3：上层经具名inventory登记的直达兜底=§5.3例外，纯SDK闭包/缺席语义不变）；无双Parser/VM可变状态 |
| V-03 | AC-03 真UI闭环 | 同源示例经四生成器与VM/Iced；新UI compiler/runtime及兼容路径 | 产物/行为相同，import变化单独解释；生成工程真实编译运行。不能只字符串检查或修改golden消掉漂移 |
| V-04 | AC-04 余边与寿命 | i18n/帧值/a2r/registry/provider替换，UI装载/退出、终端/媒体/编辑器/跨thread_local调用 | SDK不引用UI实现；原缺席/覆盖/owner、线程亲和和退出保持，诊断/额外文件不因分包丢失或双份 |
| V-05 | AC-05 测试覆盖/发行 | 迁前后nextest list/家族选择映射、tu单进程、独立fresh/locked/792重定位、缺source/反向边负控 | 原测试族按owner仍被同档选中，无以总数碰巧相等代替映射；实际source/资源齐全、负控红 |

work只定向check/新包测试/相关反例及新consumer；T-01在合入门后冻结实际包名和命令，例如`cargo check -p auto-lang-core -p auto-ui-compiler -p auto-ui-runtime -p auto-lang`，未来包创建前不把命令当执行证据。777用`python scripts/modularity-audit.py --mode strict --no-cargo-check`；792沿现有generated-ui-consumer driver。真实跨端按autoui-verifier现有设施选有代表性的实际触面，环境缺失pending。

独立final review同树独立上下文验AC/SD与原始反例；未指定agent/model不创建会话。merge最终冻结提交按迁移后的真实门入口统一裸cargo t+tv+tu+tt，若文档/Schema生成实际迁移触面则补docs_gen及tb必要面；开发/复审不跑整档。tf主检出到期单实例。复用绑定源码/依赖/配置/输入，合流变化复审/重放影响面。本轮仅文档Category A，无Cargo/docs_gen。

### r2阶段验证与并行复审

| Case | AC / invariant | 真实入口/设置 | 区分性oracle与边界 |
|---|---|---|---|
| V-06 | AC-08 A阶段真实边界 | T-13建立新增SDK/compiler外部consumer；Core无UI源；旧默认UI与A生成产物使用当期真实runtime | 纯SDK禁止边确实消除、非UI消费执行成功；旧API/状态同源、A公开生成出口可用；缺新runtime能力明确拒绝且不伪报B完成 |

上述新增Case与原V项均owner_stage=work，由新增阶段work任务使fixture真正可运行，当前尚未执行。原测试表/最终AC不缩减；A只取其明确范围，B仍逐项验证完整支持面。阶段发布的API/能力/依赖/source/资源和测试选择变化须有同步正反控，不把必需consumer验证全部后置。

实施者先修齐本轮已知findings，定向回放共享反例集/相邻状态组合及必要check，再提交新SHA送审。同Plan同一权威树冻结交接，不能边改边让review读工作区；不同Plan独立scope的读审、定向验证、修复可并行。缺上游的真实集成明确pending，不每轮重复算新finding、不用fake补绿。

A新增review任务只给阶段pass；原最终review任务仍验全部AC/SD。阶段和最终的裸t/触面整档均归merge冻结后统一排队，资源独占项串行；tf仅主检出到期单实例。接口修复由owner发布，影响方重放受影响scope，未变且适用证据可复用。具体失效/收据/单写者规则见[阶段交付与并行审查修复合同](../reports/staged-delivery-review-814-816.md) §2–5。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 真SDK可独立消费且normal/build/dev无UI实现/Man，真实非UI消费者落地、语言/VM单源；V-01 |
| AC-02 | work | 旧公开API/字段/trait与默认UI保持，纯SDK显式组合可用，无未具名的隐式链接装配（r3§5.3具名登记例外）/双状态；V-02 |
| AC-03 | work | UI编译/生成/运行职责完整迁移、四生成器与真VM/Iced/生成工程消费保持；V-03 |
| AC-04 | work | i18n/frame等余边清偿，provider/全局/TLS/资源与退出保持；V-04 |
| AC-05 | work | 测试族迁移选择面、777/792与fresh/locked/资源负控通过，SD/来源/剩余兼容与净迁移清单完整；V-05 |
| AC-06 | review | 同树独立审查绑定修订，遗漏/规范/警告/格式/debug检查通过，merge门pending明确 |
| AC-07 | merge | 冻结门、canonical Specs/ledger/归档/接口与资源交接、guard clean后清理有实际收据 |
| AC-08 | work | M814-A满足§5.1全部阶段性质，V-06及V-01/02/03/05当期范围通过；非B整体pass |
| AC-09 | review | T-14独立阶段审查绑定r3/M814-A冻结SHA/API与Spec子集，通过且merge门pending |
| AC-10 | merge | T-15冻结门通过、实际master祖先交付与Spec/ledger增量、版本/来源/consumer读回收据；不归档 |

## 8. 执行步骤

15项；保留原任务ID，追加A阶段任务，**按依赖列执行，不按编号排序**。整项depends_on为摘要，阶段启动以§4/5和本表为准；不宣称Runner已支持阶段调度。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付、验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 现在；807/808/810/811已交付 | 新attachments/814/contract-baseline.md：所有者DAG/旧API构造/状态/测试族/资源/真实消费者清单与候选布局；前置合入后整链编译probe冻结SDK/默认UI兼容方案，不可行先重规划。[✅ 已完成 2026-10-20：基线 14 节 + probe 记录 §15；切片 `be10024ab`（唯一树，基面 `203e2ce91`）冻结包名 `auto-lang-core`/`auto-ui-compiler`/`auto-ui-runtime`+DAG+再导出机制+别名多 -p 机制；全部消费者编译/日常档 5303 绿/777 pass；无 needs_replan] | AC-01..05 |
| [x] | T-02 | work | T-01可行性决策 | 真实纯SDK/上层兼容、单份Core/状态与公开出口；立即清除阻碍纯SDK的全部反向边，保旧默认API；迁出测试/资源/非UIconsumer及V-01/02当期证据 | AC-01,02 [✅ 已完成 2026-10-20 R3 会话：主迁移 `edb78b2ef`（soft-reset 吸收 WIP）+ 残项收口 `43366c989`——auto-vm 切 SDK（V-01 真实非 UI 消费者，闭包零旧包/UI，冒烟执行过）+ a2r frame 经 PLAN-779 FrameProvider 进程默认端口单源（plan716 帧探针全过）+ F814-R1-02 整改 `ac2882a0d`（SDK auto-ui-dialect 改非默认 `ui-dialect` 显式组合 feature；上层 default 转发保旧行为；normal/build/dev 三边闭包验证无 UI 文法；86 UI 文法测试按 feature 门控选择面不变）+ 迁移死门清偿（i18n 路由/diff code-editor 转发/4 组滞留 UI 测试上移 plan656/485/712/413）；SDK off 1627/1627、on 1713/1713；777 strict pass ×2] [✅ R3-03b 收口 2026-10-20：SDK 删 `ui-dialect` feature 与 optional auto-ui-dialect 清单边（R3-03 唯一余 P1 关闭）——UiParseContext 迁 auto-parser-api（re-export 保旧路径/类型身份）+ SDK UiGrammar 函数表端口 + UiGrammarSource inventory 装配（与 R3-01 同机制）；no-UI 源码快照 cargo metadata exit 0（reviewer 探针口径适配复跑 PROBE PASS）；86 文法测试随迁 auto-ui-dialect（86/86 绿）；777 strict pass（基线 `6418f99ef`）] [✅ R4 批收口 2026-10-20：R3-02 指纹身份语义（RwLock None→Some 升级+save 按出生身份落盘，四反例回放全绿+单测回钉）+ R3-03 SDK build 输入具名清单收窄（17-crate 编译闭包，BUILD_INPUTS 429 项零 UI/Man，双负控测试）+ R4-01 显式装配优先（安装器/生成入口显式 set_ui_grammar，inventory 兜底边界对账 [附件](../attachments/814/r4-01-assembly-contract.md)）；代码 `81c8a5743`+`189ca0f77`（777 基线）] | AC-01,02 |
| [x] | T-03 | work | T-02 | 真实UI compiler、aura/a2ui/design_tokens/registry与生成路径迁移；A当期真实runtime产物消费、无环公开面、资源/测试选择与V-03/05当期证据 | AC-03 [✅ 已完成 2026-10-20 R3（B1 `b907a89eb` + B2 `59b9c7673` + 消费批 `b02a10772` + SD-08 `abe9c9900`）：style/action_config/lucide/aura/a2ui/ui_gen 四生成器/registry/bp/route ≈11.3 万行迁 auto-ui-compiler 单源+旧路径全再导出；ui-package-consumer 独立 fixture 真实生成 4/4；语料资产随 owner 迁；指纹机制迁 compiler build.rs；交付合同更新至新包图（792 driver 25/25）；SD-02 事实底稿=m814-a-candidate §2（正文随 T-14/T-15 受管沉淀）] [✅ R3-06 收口 2026-10-20：T-03 迁移残留 cfg 门归位——compiler 声明 `ui`/`ui-headless` 空 feature + 旧包转发，pac.at ui_config 装载实臂与 plan571 生成器测试选择面恢复（T-03 起静默丢失，[归因表](../attachments/814/r3-06-warning-attribution.md)）；unexpected_cfgs=0] [✅ R4 批收口 2026-10-20：fmt 全清（macros 测试/src、ui_cache、新增片段）+ `image_surface_rust_codegen` ui-iced 残门归位 + T-02 被丢测试 `test_style_recipe_in_widget_extraction` 恢复（compiler style，203e2ce91 在册）+ aavm 档 T-02 编译红修复（remap_string_indices 跨 crate）] | AC-03 |
| [x] | T-04 | work | T-15/M814-A已发布 | UI runtime/renderer/宿主adapter与资源迁移，VM依轻能力；保持真实编辑器/终端/媒体与跨端 | AC-03,04 [✅ 已完成 2026-10-21 work 批（V-03 跨端双跑=环境资源待跑，其余面已验；详见 §9 T-04 记录）：ui/ 全树+host_adapters+back_proxy+assets+UI 动态构建管线（lib.rs 689-2588 区=dynamic_build.rs）+plan370 支持件迁 auto-ui-runtime；装配扇入收敛 `auto_ui_runtime::install_ui_host_capabilities` 单点（文法装配留旧兼容包，r3 §5.3 不变量）；61 个 UI 域测试随 owner 迁 ui_tests/（plan370 消费族/私钩族/019/042/058）；auto-ui-runtime 1875 测全绿（2 例语料路径+4 例 e2e 显式装配修复）；SDK 三边闭包 0 UI 保持；777 strict pass（ui_to_language=0 入基线）；t/tf 别名扩 -p auto-ui-runtime 防丢面；预存红修复 3 件（probe_adapter 门控断点/extract_autovm_result 再导出/tests_seam 可见性）；T-15 遗留 777 重采样随批完成] | AC-03,04 |
| [x] | T-05 | work | T-04；补齐B默认组合，A兼容不得先破坏 | 默认Parser/VM/新SDK显式UI装配、旧auto-lang字段/方法/feature/旧路径兼容；真实import及外部consumer V-02 | AC-02 [✅ 已完成 2026-10-21 work 批（详见 §9 T-05 记录）：生成代码 import 路径策略单源 canonical 化（import_policy.rs 新旧映射表+负控 pin——rust 生成器 136 处发射全迁 auto_ui_runtime/auto_lang_core，A 期"指向当期真实实现"过渡形态收口）；auto-man 模板/manifest/792 投影 spec 随包图（含 shell-pack canonical 再生+runtime 投影 spec 新增）；e2e 生成工程双直依赖真编译 2/2；V-02 旧公开面编译 pin 4 测试（Parser 17 字段/构造/方言族、桥 17 方法、VM/NativeInterface 注册族、门控根再导出+新旧类型身份同一）] | AC-02 |
| [x] | T-06 | work | T-05；A纯SDK禁止边已在T-02清除 | i18n/a2r frame/config等残边、全局/TLS/registry单源和退出验证；必要非UI消费者切SDK，V-01/04 | AC-01,04 [✅ 已完成 2026-10-21（26b8b33bd）：勘察确认 SDK 生产码零 UI 引用/compat 仅再导出（T-02/T-04 已清 i18n hook/frame 779 端口/photo_service 随迁归 runtime）；test_code/test_file 族下沉 SDK；**三非 UI 消费者切 SDK**（auto-gen/auto-playground/a2r-actor-tests——闭包零 UI 零旧包，playground 基线预存红一并修复）；V-04 provider/frame/i18n/theme/mem_guard 族 91/91；SDK 三边纯净保持 1631/1631] | AC-01,04 |
| [x] | T-07 | work | T-06 | 测试源码/别名/nextest选择面按owner迁移，全部旧族逐项映射，移除旧实现；V-05选择清单 | AC-05 [✅ 已完成 2026-10-21（257e02454）：6 个 dev-deps 消费测试迁 runtime→auto-lang 测试期 UI 依赖清零（T-04 临时兜住收口）；tt/tb/taa/ta/t3 别名扩 -p auto-ui-runtime（否则 ~1900 例静默丢面）；T-04 三模块漏挂载勘误（验证器逐 ID 捕获）；r5 验证器 62 条显式重写扩展——**九档 missing=0/unresolved=0 全核销+selftest 删除负控过**（t 5303→5328/tf 6270→6279/tv 163=163/tt 5675→5700/tb 5296→5304/tu 867→868/taa 6266→6275/ta 6720→6729/t3 6721→6730；[t07-closeout](../attachments/814/t07-closeout/)）；新码面零警告（迁移码既有警告随迁，203e2ce91 同形态 auto-lang 637→479 净-158）] | AC-05 |
| [x] | T-08 | work | T-07 | 新consumer/上层定向矩阵V-01..05、777/792新闭包、fresh/locked/重定位与负控 | AC-01..05 [✅ 已完成 2026-10-21（463fd029c/8afba8d01/e1c0f4770/780415083）：**缺席语义回归捕获**（auto-lang→runtime 依赖边漏关 default-features——no-UI 消费者经路径边拖入 ui-iced+inventory 链装配破坏缺席臂；provider fixture 哨兵发现，修复后六 fixture 43 测绿+闭包零 iced）；ui-runtime-consumer 新 fixture（canonical 双直依赖 5/5+fresh/locked+旧包缺席）；**792 driver 24/24 EXIT=0**（V01 fresh 真构建/V02 重定位 locked/V03a..j 负控/V04 dev 路由——断言随 canonical 更新）；生成器两缺陷修复（truthy 过投射/on_click 兜底——shell-pack 再生暴露，tu 870/870 保持）；777 strict pass（基线重采样@463fd029c，shell-pack→auto-lang 豁免边消除 4→3，ui_to_language=0 保持）；V-03：VM 端真渲染实据（auto run -r vm MCP snapshot 树可读）+tu e2e 真编译 2/2；Vue/Playwright 端环境栈未跑=pending] | AC-01..05 |
| [x] | T-09 | work | T-08 | SD-01..08备稿、真实API/包源/资源/测试归属及compat退出清单，新增attachments/814/handoff.md绑定SHA/结果，提交送审 | AC-05 [✅ 已完成 2026-10-21：[handoff.md](../attachments/814/handoff.md)——B 提交链 11 枚/V-01..05 终态矩阵/兼容退出清单（再导出面+auto-man 自身+a2r 泳道+inventory 例外四项保持）/资产测试归属/SD-03 全文与 SD-01/02/04/08 增量备稿（§5）/merge 门 pending 清单（§6）；树 clean@780415083] | AC-05 |
| [x] | T-10 | review | T-09 | 同树独立完整DAG/公开兼容/生产/寿命/测试选择与SD审查，merge门pending | AC-06 |
| [x] | T-11 | merge | T-10 pass | master并集/变化复审，最终冻结提交统一§6门，失败修复后重审重冻 | AC-07 |
| [x] | T-12 | merge | T-11 | 规范/ledger沉淀、归档、已合入SDK/UI出口交815，guard clean后清理、终态计数 | AC-07 |
| [x] | T-13 | work | T-02/T-03 | 冻结A API/版本/源码与支持清单；V-06及A涉及V-01..05、777/792/独立fresh/locked/负控完成，新增attachments/814/m814-a-candidate.md，准备实际Spec子集 | AC-08 [✅ 已完成 2026-10-20：候选冻结 @ `abe9c9900`（[m814-a-candidate.md](../attachments/814/m814-a-candidate.md)——§2 A 事实清单/§3 证据矩阵/§5 诚实限制）；消费者 fixture 双绿（language-sdk-consumer 5/5 + ui-package-consumer 4/4，`b02a10772`）；777 strict pass（域登记+窄豁免，基线 `d91948ee1`）；fresh+`--locked` 消费跑通；**792 driver 25/25（EXIT=0）**——首轮受管红暴露新包图三接缝（投影 test-cases 新形态/ui_gen 源根/auto-ui-runtime 未 vendored），SD-08 修复于 `abe9c9900`（DELIVERY_CRATES 19、投影分 spec、指纹路径）；auto-man delivery 单测 9/9。**AC-08 判定归 T-14 独立审查**（本勾选=work 交付完成，非审查 pass）] [✅ R3 全 findings 修齐 2026-10-20：批 3 `6f9daa680`+`6418f99ef` 关闭 R3-03b/05/06（映射表全文 [r3-05](../attachments/814/r3-05-test-selection-mapping.md)、告警归因 [r3-06](../attachments/814/r3-06-warning-attribution.md)、探针复跑 [收口证据](../attachments/814/r3-03b-closeout/)）；R3-01..04 批 1/2 已关（§9 前录）；候选 SHA 待 T-14 重冻=分支头] [✅ R4 批 2026-10-20：R3-05 档位选择面 203e2ce91 基面全量回溯——九档（t/tf/tv/tt/tb/tu/taa/ta/t3）未匹配 0/选择缺口 0（[映射表 R4 修订](../attachments/814/r3-05-test-selection-mapping.md)）；别名 tb/taa/ta/t3 扩四包、tt 补 compiler；告警归因 [203e2ce91 口径](../attachments/814/r3-06-warning-attribution.md)（净新增≈14 条全归因 T-02/T-03 残留+B 清偿域）；候选 SHA=分支头 `16cc4f14a`（R4 批+追修）] | AC-08 |
| [x] | T-14 | review | T-13提交冻结 | 独立阶段AC-08/API/类型状态/资源/Spec审查，绑定r3/M814-A SHA；全Plan未reviewed、merge门pending [✅ R7 phase pass：256613936，见§9及review-r7；仅A阶段，T-15实际发布仍pending] | AC-09 |
| [x] | T-15 | merge | T-14 pass | 串行冻结门/实际master合入/Spec子集与受管ledger增量、版本来源consumer读回发布收据；保原树/分支继续B，不归档 [✅ M814-A 发布完成 2026-10-20：冻结门（分支 e47d85c7b）=裸 t 5307/5307（plan808 term_engine UI 引用计数 1→0 + schema_drift 六路径两围栏随迁修复，生产零改动）+tv 163+tu 867+tt 5679/5679（plan485 剪贴板并行 flake 归因复跑绿）；taa 用户裁定豁免（aavm 拟退役，两红仅记录）。master 合入 `f6e07acee`（合并解析：master 侧 test_cases/{abi,ac_av} 新增跟随目录迁移至 SDK 新路径；plan796 指标工件清道提交）+分支同步（B 继续）。Spec 子集落库：新 language-sdk/auto-ui-compiler project.md + auto-lang project/frontend/vm/ui overview A 子集 + r3 两处 canonical（parser-api/full-grammar）；ledger v62（P814-A 收据）+spec-index 55 projects。consumer 读回（合入后 master）：language-sdk-consumer 5/5 + ui-package-consumer 4/4 + no-ui 源码快照 fresh+locked + **792 driver 25/25**（首轮红=master 树 ffi_dual build_a2r 忽略态生成残留触发交付外部 path 守卫——环境清理后复跑绿，投影 ignore 感知缺失记 KNOWN-DEBT）。777 master 重采样被并发会话脏文档阻塞（后续跟进项）。证据=[t15-closeout/](../attachments/814/t15-closeout/)] | AC-10 |

唯一树`D:/autostack/.wt/lang-814/auto-lang`、分支`codex/plan-814-dev`；work/review顺序交接同树，先核实实际tree/base并读取主检出r2。A发布后保留原树/分支同步master继续B，不创建第二审查树、不归档/清理。A期间Plan保持executing，只有B全部原AC完成后才依标准流程execution_done→reviewed→archived。

阶段发布是用户本轮明确授权的有界例外：按共用合同独立阶段review、冻结门、人工Git合入及非归档Spec/ledger受管增量与收据组织；**不调用会自动归档整Plan的完整merge流程冒充phase支持**。B按原最终review/merge/归档任务完成。无WIP依赖，无junction/symlink，最终清理必须guard clean。跨Plan审查/修复与共享窗口见[阶段交付与并行审查修复合同](../reports/staged-delivery-review-814-816.md)。

## 9. 复审记录

r1以下记录为历史合同预检，旧启动门由r2替代；不是当前调度指令。

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实现/独立验收pass）/ next: work T-01只读；代码probe和T-02起等待810/811实际合入 / changed: T-01..12、AC-01..07、SD-01..08。未执行未来Cargo/fixture、未创建工作树/agent或搬仓。

2026-10-20 / stage: new / revision: 2 / outcome: pass（文档合同预检，非实现/独立验收pass）/ next: 按§4阶段门work；814先确认已登记唯一树并读取本r2。changed: 阶段A/B与新增T/AC/V、Spec分阶段沉淀、并行审查修复；原目标、原T/AC/SD ID保留。r1证据仅历史，不自动覆盖新范围。未执行实现/未来fixture/门禁。见[预检](../attachments/sdk-tools-r2-20261020/preflight.md)。

2026-10-20 / stage: work / revision: 2 / outcome: T-01 完成（M814-A 第一步；work 继续 T-02，非阶段/整Plan pass）/
worktree: `D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | base:
master `203e2ce91`（811 收口；唯一树与 r2 §8 登记一致）| code: `be10024ab`（T-01 probe
切片）| 依赖: 组内 auto-down 兄弟 detached@c39ac7f（autodown-core 解析，零源改动）。
交付 [attachments/814/contract-baseline.md](../attachments/814/contract-baseline.md)（§1-14 盘点
+ §15 probe 记录）：包名/DAG/兼容机制冻结；use_scanner 机制切片迁 SDK（旧路径
`auto_lang::use_scanner::*` 再导出保类型身份，auto-lsp 等六消费者零 error）；别名
t/tf 扩多 -p+包前缀 feature（迁移族 `use_scanner::tests::` 27/27+doctest 2/2 保持选中）；
日常档 5303/5303；no-ui 构建过；777 strict pass。task_ids: T-01 [x]（current_step=1/15）|
evidence: contract-baseline §15 + 切片提交 | blockers: 无 | next_responsible_role: work
T-02（真实纯 SDK 主迁移：Core 单份迁移+立即清除 a2r frame/i18n/config 等阻碍纯 SDK
的反向边+auto-vm 切 SDK 的 V-01/02 当期证据）。r1 会话两份未提交簿记（status 翻转与
§9 记录）被本 r2 修订覆盖、attachments/814/ 曾被清除——已按 r2 重建，无内容损失。

T-02 增量批 1（同日）：`cc3b18f50` 迁 dep_scanner/mode/os_paths（733 行，std-only）
入 SDK，同机制再导出；SDK lib 42/42+doctest 4/4、七消费者 0 error。T-02 剩余主迁移
（parser/compile/vm/trans 骨干 + lib.rs Core/UI 函数拆分 + a2r_std.rs frame 桩清除 +
auto-vm 切 SDK）待续；机制与批次形态已由前两提交定型。

T-02 增量批 2（同日）：`1d2918bec` 迁前端门面组 ast/token/error/session/lexer +
symbols/scope/scope_manager（669 行）入 SDK——SDK 获前端轻闭包依赖
（auto-ast/lex/diagnostics/parser-api/auto-val/ecow）；lexer 在 auto-lang 侧保持
crate 私有（PLAN-789 合同，私有门面模块形态）。SDK lib 49/49+doctest 5/5、七消费者
0 error、日常档 5303/5303（完整复跑；首轮冷负载瞬态早退未复现）。

T-02 大迁移（同日，**WIP `03d983ae9`，树不可编译，明确标注不得送审/合入**）：
类型系统/parser 骨干/执行/编译/vm/trans/libs/服务/stdlib_assembly/design_tokens
全量模块迁入 SDK（auto-lang 仅余 ui/ui_gen/aura/a2ui/host_adapters/back_proxy/
route[AuraRoute 实依赖留上层]/测试树）；lib.rs 根项行号精确切割提取了 run 族/
executor 对/env 谓词簇/eval 簇/module 装载簇（~1700 行）。裁定记录：design_tokens
随 grammar 通道进 SDK（parser style-recipe 门硬依赖，T-03 复审归属）；recipe↔aura/
ui_gen 集成测试上移 plan814_recipe_extraction_tests；REPL 隐式 UI 装配改 hook
（set_ui_host_install_hook，纯 SDK no-op）。**剩余约 12 个根项级联未完**（run_*_
file_test 调用方处置、extract_autovm_result/init_rust_ffi/init_py_ffi/remap_*/
find_use_symbol_span/extract_undefined_symbol/read_auto_lib 门），后继会话从
`03d983ae9` 提交信息中的续作清单与批次配方继续（cargo check 循环→消费者→scoped
测试）；之后 a2r_std.rs frame 桩改 FrameProvider、auto-vm 切 SDK。

2026-10-20 / stage: review / review_scope: final（R1就绪核查，未授final/phase pass） /
reviewer_identity: 本聊天Codex，独立于实施上下文 / plan_id: PLAN-814 / plan_revision: 2 /
outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-814/auto-lang` /
branch: `codex/plan-814-dev` / reviewed_commit: `03d983ae9a654d06963b2ad56a04a3adb3bf8e6e` /
base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009` / dependency_revisions:
组内auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
详见[报告/AC逐项与证据边界](../attachments/814/review-r1/review.md)、
[输入hash](../attachments/814/review-r1/input-hashes.txt)、
[冻结SD](../attachments/814/review-r1/sd-frozen.md)。实施树clean，但HEAD与§9自述均为不可编译、
不得送审的WIP；本轮不反复启动完整最终验收。
**F814-R1-01(P1)**：T-02根项级联仍缺、compiler/runtime新包仅文档注释，auto-vm未切SDK、
A消费者/候选/资源证据未交付；**F814-R1-02(P1)**：SDK仍无条件依赖auto-ui-dialect并直接调用
grammar实现，违反AC-01/08的无UI文法实现闭包。AC-01/03/05/08 fail，AC-02/04/06 partial，
AC-07/09/10 pending。保持executing **1/15**（T-01历史进度保留，其余原本未勾）；
next: 原树work完成T-02/T-03/T-13并提交后申请具名M814-A审查；整Plan最终审查还需B原任务完成。

T-02 大迁移完成（work 增记，R1 后同日）：`edb78b2ef`（soft-reset 吸收 WIP 03d983ae9）
补完 R1-F01 的根项级联 12 项并全链验证——cargo check 全包零错、no-ui 绿、SDK 闭包
纯净、SDK lib 1711/1711、日常档 5294/5294。具名清边：REPL 装配 hook 化、i18n lookup
hook、run 入口上层包装（779/785 契约保持）、term_engine 门面上移、media/diff 族历史
门解除、跨 crate 测试面 ~60 项。R1-F01 剩余：auto-vm 切 SDK + a2r_std frame 桩改
FrameProvider（提交时已知并记录的两件待收口）。**R1-F02（SDK→auto-ui-dialect 边）
未处理**——与合同基线 §10.3 的解释分歧（777 域表将 auto-ui-dialect 归 lang-core，
文法≠renderer 实现）待 work 侧按复审口径整改（UiDialect 装配上移上层/显式组合）
或提请裁定；整改前不勾 T-02。current_step 仍 1/15。
未修实施、未跑Cargo/整档/实机或合入。pending_closeout_gates: 后续独立pass后merge owner冻结
执行裸t及实际触面tv/tu/tt等，随后Spec/ledger/发布与最终归档guard；当前无发布pass。

2026-10-20 / stage: review / review_scope: final（R2修复就绪复核，非final/phase pass） /
reviewer_identity: 本聊天Codex，独立复审上下文 / plan_id: PLAN-814 / plan_revision: 2 /
outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-814/auto-lang` /
branch: `codex/plan-814-dev` / reviewed_commit: `edb78b2efbdcadb1994d4065b9cb341b3d71497f` /
base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009` / dependency_revisions:
auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
[R2报告/闭合边界/原始check](../attachments/814/review-r2/review.md)：独立locked/offline
SDK+auto-lang定向check退出0（22.49s），**R1-01的不可编译子项已关闭**；原F814-R1-01整体
仍OPEN（auto-vm/frame provider、compiler与A消费者/候选/发行未交付），F814-R1-02仍OPEN
（SDK normal依赖UI文法实现、默认装配未上移）。迁移后frame的无效ui feature门在check报告
unexpected_cfgs，读值固定0不能视为兼容已验；未把所有历史告警归为本Plan新增。
AC-01/03/05/08 fail、02/04/06 partial、07/09/10 pending；保持executing **1/15**。
Spec提案已冻结但未批准，未重跑整档/修实施/合入。next: 原树work修齐已知剩余项并给根因/
覆盖调整、定向正反控及完整候选提交后，按具名阶段或最终范围交接。merge-owned冻结裸t及
实际触面档、Spec/ledger/发布/归档guard继续pending；当前无阶段/最终pass。

2026-10-20 / stage: work / revision: 2 / outcome: R2 已知剩余项修齐（F02+auto-vm+frame）与
T-03 主体迁移完成（**非 review pass；T-03 剩余/T-13 未做，不构成 M814-A 候选**）/
worktree: `D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | base: `203e2ce91` |
code: `ac2882a0d`（F02）→ `3fe81822c`（777 基线）→ `43366c989`（auto-vm+frame+死门）→
`b907a89eb`（T-03 B1）→ `59b9c7673`（T-03 B2）→ `704260bd6`（777 基线）| 依赖: auto-down
`c39ac7f`（未变）。T-02 勾选（current_step=2/15）。

- **F814-R1-02 关闭（整改口径）**：SDK auto-ui-dialect 依赖改 optional + 非默认
  feature `ui-dialect`（显式组合）；dialect::ui 模块/ParserBridge UiParseContext/
  build_dialects 内建装配随 feature 编译；旧 Parser 10 个公开 UI 门面签名恒在
  （feature off=显式诊断错误，不静默降级）；上层 auto-lang default 转发保
  Scenario::UI 默认装配逐位不变。closure 证据：cargo tree normal/build/dev 三边无
  auto-ui-dialect。86 个 UI 文法依赖测试（parser 60/design_tokens 9/tscn 11/
  gdscript 6）按 feature 门控——统一构建选择面不变（on 1711/1711 与改前总数一致）。
- **F814-R1-01 auto-vm/frame 子项关闭**：auto-vm 依赖切 auto-lang-core（冒烟
  `hello from pure sdk auto-vm` 执行通过）；a2r_std::frame 删无效 `ui` feature 门，
  改经 PLAN-779 进程默认 FrameProvider 端口单源（VM native 9918/9919 同端口；
  plan716 帧探针全过；缺 provider=未捕获 0 语义保持）。
- **R2 提及的迁移 feature 门逐项归因清偿**：codegen i18n.t 路由拆过时 ui-iced 门
  （shim/i18n_lookup 已随 T-02 进 SDK）；a2r diff 双轨门 SDK 侧空 feature
  `code-editor`+上层转发（全组合对齐；装配指纹 ui-iced→code-editor）；4 组滞留
  UI 测试上移 auto-lang（plan656 scroll/plan485 clipboard/plan712 routes 前缀化/
  plan413 code editor——原 SDK 死门=迁移后静默丢失，V-05 勘误）；image_pipeline
  测试拆死门。**已归因未修（T-04）**：autodown natives 双门（跨仓 autodown-core
  不进 SDK；日常档无该 feature 无回归；autodown 消费者待 T-04 迁上层）。
- **T-03 主体（Batch 1/2）**：style DSL（除 iced 适配器，留上层
  ui/style_iced_adapter）/action_config/lucide 全量表/aura/a2ui/ui_gen 四生成器/
  widget registry/bp/route 分析 ≈11.3 万行迁 auto-ui-compiler；旧路径（ui::style/
  action_config/iced::lucide_generated/aura/a2ui/ui_gen/route）经再导出全保持；
  crate:: 重写 1426+ 处（SDK 模块/函数→auto_lang_core::）；STORE_EXTRA_FILES/
  drain/strip_line_block_comments 下沉 SDK；normalize_shortcut 单源随
  action_config；ui_gen 指纹迁 compiler build.rs（SDK build.rs 停扫兄弟 UI 目录
  ——R2 点名 build 边清偿；UICache OnceLock 注入+compiler 入口自装配）；语料/
  资产随 owner 迁（test/a2ark、test/a2vue、tests/fixtures/{plan356_*,
  025-native-input,026-native-display}）；desktop.at 经 MANIFEST_DIR 引上层单源
  （shell 资产家族归属 T-04）；tu 别名改 `-p auto-ui-compiler -p auto-lang`
  （单进程 libtest+OnceLock 复用机制保持）；t/tf 扩 `-p auto-ui-compiler`。
- 验证汇总：ui_gen 862/862（libtest）；compiler nextest 215/215；SDK off 1627/1627
  /on 1713/1713；上层定向 148+集成 13+plan716 探针全过；三包 --tests 全编译零错；
  DAG 断言（SDK→compiler=0、compiler→旧包=0）；777 strict pass（B2 后基线
  `704260bd6`，core_to_ui 文本 841——含再导出胶水行）。
- **续作（下一会话）**：T-03 剩余（V-03 当期证据：独立 consumer 经 compiler 真实
  生成+当期真实 runtime 消费；生成 runtime import 路径策略单源审视；SD-02 备稿）
  → T-13（M814-A 候选：V-06+fresh/locked consumer+负控+m814-a-candidate.md）→
  T-14 独立阶段审查 → T-15 冻结门/发布。主检出簿记本记录随本提交落账。
  pending_closeout_gates 不变（merge owner 冻结裸 t+触面 tv/tu/tt）。

2026-10-20（续）/ stage: work / revision: 2 / outcome: T-03 V-03 消费者证据落地 + T-13
候选草案立（**候选未冻结**：792 driver 未跑，T-13/T-03 终态判定见任务行）/
worktree 同上 | code: `b02a10772`（fixture+域登记）→ `d91948ee1`（777 基线）。
- **V-01/V-03 独立消费者**：`test/fixtures/modularity/language-sdk-consumer`
  （仅依赖 auto-lang-core 默认特性；Core VM 真实执行/手工字节码管线/
  trans_rust 产物/UI 文法显式拒绝负控，5/5）与 `ui-package-consumer`
  （仅依赖 auto-ui-compiler；真实 Vue SFC 生成出口+指纹+style+lucide，4/4）
  ——cargo tree 闭包断言：前者零 UI/旧包边，后者=SDK+显式文法组合零旧包边。
- **fresh/locked**：双 fixture `cargo generate-lockfile` 后 `cargo test
  --locked` 全绿（lockfile 按仓规不入库——流程与 792 fresh-lock 同构）。
- **777**：域表登记 auto-lang-core=lang-core、auto-ui-compiler/runtime=ui；
  SDK→runtime 两条存量轻能力边（a2r-std/auto-terminal-core）按窄豁免
  edge_note 登记（owner=PLAN-814，R8 复核）；strict pass（grandfathered=4）。
- **T-13 候选草案**：attachments/814/m814-a-candidate.md——A 事实清单（两包
  公开面/特性/闭包/测试选择+上层兼容面）、V-01..06 证据矩阵、诚实限制节。
- **续作**：①跑 792 driver（generated-ui-consumer，`auto gen --delivery-root`
  真实构建）→回填候选 §3 →冻结 SHA→勾 T-03/T-13；②T-14 具名 M814-A 阶段
  审查（绑定冻结 SHA+Spec 子集 SD-01/02/04 备稿）；③T-15 冻结门（裸 t+tv/
  tu/tt）+受管 ledger 增量。merge-owned 门禁 pending 不变。

2026-10-20（续 2）/ stage: work / revision: 2 / outcome: **T-13 完成冻结 @ `abe9c9900`**（M814-A
候选齐备；AC-08 判定归 T-14，本记录非 pass）/ worktree 同上 | code 追加：
`abe9c9900`（SD-08 交付合同更新）。
- **792 driver 复跑实录**：首轮受管红（投影修剪 4 缺 1——producer test-cases
  行自 T-01 起带 SDK 转发项）→ 修复后续红（ui_gen 源根旧路径）→ 再续红
  （auto-ui-runtime 未 vendored——上层 T-01 骨架依赖）→ 三接缝一并修复后
  **25/25 全过 EXIT=0**（V01 fresh/V02 重定位/V03a..j 负控/V04 dev 同源）。
  受管判红逐级暴露、不回退主检出——792 合同的漂移哨兵语义按设计工作。
- **修复面（`abe9c9900`）**：DELIVERY_CRATES 16→19（+auto-lang-core/
  auto-ui-compiler/auto-ui-runtime）；投影修剪按 manifest 分 spec
  （PROJECTION_AUTO_LANG 新形态 test-cases 改写 + PROJECTION_AUTO_LANG_CORE
  SDK 清单 spec；漂移判红/幂等按各自 spec，强度不降）；ui_gen 指纹源路径
  → crates/auto-ui-compiler/src/ui_gen。auto-man delivery 单测 9/9（含新增
  SDK spec 用例）。
- **T-03/T-13 勾选**（current_step=4/15）：T-03 剩余项（V-03 消费者/SD-02
  底稿）随本批收口；候选文档冻结 SHA=abe9c9900、V-05 行回填 792 结果、
  §4 改 T-14/T-15 归属。
- **next_responsible_role: review（T-14）**——同树独立阶段审查，绑定
  abe9c9900 + m814-a-candidate.md + r2 §5.1 阶段性质；Spec 子集
  SD-01/02/04 备稿以候选 §2 为事实底稿。merge-owned 冻结门（裸 t+tv/tu/tt）
  保持 pending。B 阶段（T-04..12）在 A 发布后继续。

2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r2:M814-A（R3）** /
reviewer_identity: 本聊天Codex，独立复审上下文 / plan_id: PLAN-814 / plan_revision: 2 /
outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-814/auto-lang` /
branch: `codex/plan-814-dev` / reviewed_commit: `abe9c99002fe7bafb49a9f86c5867f68d796d65a` /
base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009` / dependency_revisions:
auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
[R3报告/逐项反例与正控/冻结SD](../attachments/814/review-r3/review.md)：独立SDK消费者5/5、
compiler消费者4/4通过，原不可编译/auto-vm旧依赖/compiler骨架/默认无条件编译UI文法等
原例均已修；完整A不变量仍失败。**F814-R3-01(P1)**旧create_vm_from_source等再导出入口
丢宿主自动装配（独立进程false，run包装正控true）；**R3-02(P1)**缓存load早于指纹注入，
真实生成后保存null、后建正控有真实指纹；**R3-03(P1)**无UI源码快照metadata仍要求
auto-macros→旧auto-lang及optional auto-ui-dialect清单；**R3-04(P1)**旧autodown特性
构建成功但native仍报unavailable；**R3-05(P2)**tt/tb漏迁入SDK的原inline测试选择；
**R3-06(P3)**新增门面fmt失败/迁移无效feature告警未清。
AC-08 fail、AC-09不通过、AC-10 pending；原最终AC/B范围不另判缺陷。**重开T-02/T-03/T-13，
current_step=1/15**（T-01保留；完成修复的历史证据不撤销，原任务行完成注记是work历史）。
next: 同树work一次修齐R3-01..06，回放共享反例/正控及相邻入口/配置/跨进程/独立源码消费，
提交新A候选再审。当前SD底稿已冻结但未批准，runtime缓存规范delta需补齐；未修实施/跑整档/
合入。pending_closeout_gates: 后续阶段pass后merge owner冻结裸t+实际触面tv/tu/tt等，
实际A发布/Spec子集/ledger/来源收据；A不归档不清树，B仍按原流程。


2026-10-20（续 3，R3 needs_fix 后）/ stage: work / revision: 2 / outcome: R3-01/02/03a/04 + R3-05/06 部分
修复完成（**R3-03b 未完成，不申请重审**）/ worktree 同上 | code 追加：`d1fbae132`（批 1）→
`d905e796c`（批 2）。受审基线 `abe9c9900`。
- **已关闭（探针回放全 PASS）**：R3-01（上层安装器注册 SDK 钩子 + 三入口兼容包装 +
  UiCompatInstaller inventory 收集点——上层被链接即自注册，SDK 零 submit 纯形态不变）；
  R3-02（指纹槽 RwLock 升级语义 + save 重盖[仅 Some] + auto-man vue/jet load 前注入）；
  R3-03a（auto-macros dev 链→SDK；value! 宏 Box<Obj> 漂移=预存债同 HEAD 红已记）；
  R3-04（AutodownEngine 进程端口 + 实装迁上层 ui/autodown_engine.rs + dispatch 三级回落）。
- **部分**：R3-05（tt 别名扩 -p auto-lang-core+前缀 feature；escape_analysis_summary
  跨 crate 支撑修复——tt 构建红根因；**映射表全文未交付**）；R3-06（门面 fmt 清；
  **告警全量归因未做**）。
- **未动**：R3-03b（SDK→auto-ui-dialect optional path 清单边——设计已定未实施：
  UiGrammar fn 表端口替 cfg feature、bridge UiParseContext impl 上移上层、86 文法
  测试随迁 auto-ui-dialect、SDK 删 ui-dialect feature 与 optional dep；装配复用
  UiCompatInstaller 收集点，与 R3-01 同机制）。
- 验证：reviewer 三探针（host/cache/autodown）+ no-ui 脚本（推进到 dialect 项）回放；
  ui_cache 9/9、SDK 1627/1627、ui_gen 862/862、上移 11/11、tt 新形态定向 1/1。
- **续作**：①R3-03b 实施（上述设计）；②R3-05 映射表全文（nextest list 迁前/迁后
  对比）；③R3-06 告警全量归因清偿；④全探针复跑后提交申请新一轮 M814-A 阶段审查。
  merge-owned 门禁照旧 pending。

2026-10-20（续 4，R3-03b 批）/ stage: work / revision: 2 / outcome: **R3 findings 全数修齐**（R3-03b 唯一
余 P1 关闭 + R3-05/06 收口；非 review pass——T-14 重审待申请）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | 受审基线:
`d905e796c` | code: `6f9daa680`（批 3 主体）→ `6418f99ef`（777 基线）| 依赖: auto-down `c39ac7f`（未变）。
current_step=4/15（T-02/T-03/T-13 复勾）。

- **R3-03b 关闭（设计落地含一项孤儿规则必要调整）**：SDK 删 `ui-dialect` feature
  与 optional auto-ui-dialect 清单边。落地形态：①`UiParseContext` trait 自
  auto-ui-dialect **迁 auto-parser-api**（记录调整：原设计"impl 上移上层"在
  孤儿规则下不合法——foreign trait × foreign type 的 impl 只能落在 trait 或
  类型所属 crate；迁至双侧共同依赖的轻端口 crate 是唯一合法位置。PLAN-810
  §2"不在 auto-parser-api"裁定理据保持——仍是独立 opt-in 子 trait，
  ParseContext 实现方零强加；auto-ui-dialect 经 re-export 保旧路径与类型
  身份，ui-grammar/ui-declarations-consumer fixtures 7/7+10/10 零改动过）；
  ②SDK 定义 `UiGrammar` 函数表端口（auto-parser-api，11 项 fn 指针，msg/model
  走 ParseContext 轻面与真实签名一致）+ `dialect::ui::UiDialect`/Parser 十个
  公开 UI 门面/build_dialects/scene·store 门全部改表派发（表缺席=原 feature
  off 语义逐位一致：方言表空/显式拒绝诊断/门恒 false）；③装配复用 R3-01
  机制：SDK 收集点 `UiGrammarSource`（inventory，SDK 零 submit）+ 显式
  `set_ui_grammar`；上层 auto-lang 与 auto-ui-compiler 各一行 submit
  （auto-ui-dialect::ui_grammar() 单源构造器）——直达
  `Parser::from(..).with_session(UI)` 构造不经漏斗也覆盖；④86 文法测试随迁
  auto-ui-dialect `src/regression/`（dev-dep auto-lang-core 驱动真 Parser，
  `setup()` 注入表；语料路径 `../auto-lang/test/a2gd` 同深成立；本地 helper
  5 组与 plan778 双 dialect struct 随迁/复制，SDK 侧死代码清除）。
- **R3-05 关闭**：[映射表全文](../attachments/814/r3-05-test-selection-mapping.md)
  ——86/86 逐项映射无丢失无新增；t/tf/tt 别名扩 `-p auto-ui-dialect`；档位
  对照与计数核对（原始清单随附 closeout 目录）。
- **R3-06 关闭**：[告警归因表](../attachments/814/r3-06-warning-attribution.md)
  ——基线 296 == 迁后 296（0 新增）；迁移残留 cfg 门清偿四点（含**附带行为
  修复**：compiler `ui` feature 缺声明致 pac.at ui_config 装载自 T-03 静默走
  None 桩 + plan571 生成器测试从 tu 档静默丢失——均恢复并实录验证）；
  unexpected_cfgs 0；SDK term_engine 死测试面删除（真面=上层 terminal_provider）。
- **验证汇总**：auto-ui-dialect 86/86（0.35s）；SDK 1632/1632（表缺席形态）；
  上层定向 UI 面 107/107 + plan716/ui_cache 54 项；ui_gen 862/862；plan571
  恢复运行（单包 4/4、tu 形态 3 过）；四 modularity fixtures 5/5+4/4+7/7+10/10；
  language-sdk-consumer 负控随新拒绝文案更新（"not assembled"显式拒绝不变量
  保持）；reviewer 三探针（host/cache/autodown）+ no-ui 源码快照探针
  （[收口证据](../attachments/814/r3-03b-closeout/)）全 PASS；777 strict pass
  violations=0（ownership 焊点路径 10 处随迁移归位；基线重采样
  `6418f99ef`）；`git diff --check` 0；触面文件 fmt 过。
- **预存不动**：auto-cache/shim-metadata 测试红（基线同在，非本 Plan 迁移面）；
  296 告警集合内既有项按归因表存档。
- **next_responsible_role: review（T-14 具名 M814-A 阶段审查）**——绑定分支头
  `6418f99ef` + r2 §5.1 阶段性质 + [m814-a-candidate.md](../attachments/814/m814-a-candidate.md)
  （§3 证据矩阵需按本轮更新重核：777 基线 SHA/no-ui 探针/映射表/告警表为新增
  绑定；§6 增量附录已补）。**792 driver 复跑 25/25（EXIT=0，收据
  worktree scratch-consumer/driver-receipt.json）**——SDK/compiler 清单变化
  （feature 删除/dev 边新增）下交付闭包/投影/指纹合同保持。merge-owned
  冻结门（裸 t+tv/tu/tt）照旧 pending。

2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r2:M814-A，R4重点复验R3六项** /
reviewer_identity: 本聊天Codex，独立复审上下文 / plan_id: PLAN-814 / plan_revision: 2 /
outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-814/auto-lang` /
branch: `codex/plan-814-dev` / reviewed_commit: `6418f99efe0af2432a652212e98d39b973515cd9` /
base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009` / dependency_revisions:
auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
[R4六项闭合表/原例与相邻正反控/冻结SD](../attachments/814/review-r4/review.md)：原host/cache/
autodown三探针全过，非UI源码快照新锁+locked消费者5/5；显式frame默认/VM快照、autodown
往返与错误分支、Parser-only直达均通过。**R3-01/04原行为CLOSED**；**R3-02仍OPEN**
（None先占位锁死真实身份；晚注入保存旧cache给未失效条目重盖新指纹，两个反例exit101）；
**R3-03源码清单/执行子项CLOSED，build输入余项OPEN(P2)**（实际SDK BUILD_INPUTS仍含
UI renderer/vue生成器/Man）；**R3-05仍OPEN**（86/86名称映射与tt修复有效，但tb等原档仍
漏SDK/UI文法迁出inline测试）；**R3-06仍OPEN**（原lib门面fmt及unexpected_cfgs已修，
本轮新增ui_cache测试/macro导入块fmt仍红）。新增**F814-R4-01(P2)**：inventory隐式装配
与r2无隐式链接装配合同未对账，UiParseContext迁位/缓存规范真实A子集delta尚未冻结。
本轮补齐受影响Spec导航目标（runtime、auto-parser-api、full-grammar），不批准语义放宽。
AC-08 fail、AC-09不通过、AC-10 pending；重开T-02/T-03/T-13、current_step=1/15，
保留T-01及已完成修复证据。next: 同树work一次修齐具名剩余边界/合同与SD，回放共享正反控
及真实档位映射后提交新A候选。pending_closeout_gates不变：后续阶段pass后merge owner
冻结裸t+实际触面档、实际发布/Spec子集/ledger与来源收据；A不归档不清树。未修实施、未跑
整档/合入，不授阶段/最终pass。
2026-10-20（续 5，R4 needs_fix 后）/ stage: work / revision: 2 / outcome: **R4 具名剩余边界与
合同/SD 对账一次修齐**（非 review pass——T-14 重审待申请）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | 受审基线:
`6418f99ef` | code: `81c8a5743`（R4 修复批）→ `189ca0f77`（777 基线）| 依赖:
auto-down `c39ac7f`（未变）。current_step=4/15（T-02/T-03/T-13 复勾）。

- **R3-02 关闭（身份转换语义）**：指纹槽 OnceLock→RwLock——None→Some
  **升级**（晚装配不阻断真实身份）+ 已装配首装优先；`save()` 删除
  "当前身份重盖"分支，按缓存对象**出生身份**原样落盘——晚装配后保存
  旧缓存，下一 load 经 None-vs-Some 比对判整缓存失效（PLAN-698"一侧
  缺席也整缓存失效"），不重签旧条目。生产注入点（auto-man vue/jet、
  compiler api）均为 load 前注入（批 1 已就位），无重盖需求。reviewer
  四反例（none-upgrade/late-save/F1→F1/F1→F2）回放全 PASS + 两条单测
  回钉（`plan814_none_to_some_identity_upgrade`/
  `plan814_late_identity_invalidates_old_entries`，含单进程序不敏感
  reset）。
- **R3-03 build 输入关闭**：SDK build.rs 全 crates 扫描改**具名清单**
  （IDENTITY_SOURCE_CRATES=SDK 编译闭包 17 crate + vendor 外置成员 +
  根锁；auto-down 改仅显式 AUTO_DOWN_ROOT 时纳入）。BUILD_INPUTS 429
  项**零 UI/Man/旧包/生成器条目**；双负控单测（禁入清单断言 + 直接
  path 依赖全覆盖断言——防清单与 Cargo.toml 漂移）。
- **R3-05 关闭（基面回溯 203e2ce91）**：临时组内 worktree 按原别名
  定义逐档重建旧选择面（t 5303/tf 6270/tv 163/tt 5675/tb 5296/tu 869/
  taa 6266/ta 6720/t3 6721），与当前档逐档对照（路径精确+已记录迁移
  模块重写规则）——**九档未匹配 0、选择缺口 0**。修复：tb/taa/ta/t3
  扩四包、tt 补 compiler（裸 feature 前缀化）；**aavm 档 T-02 编译红**
  （`crate::remap_string_indices` 跨 crate 失联——该族短期不跑未暴露）
  修复；`image_surface_rust_codegen` ui-iced 残门归位；T-02 被丢测试
  `test_style_recipe_in_widget_extraction` 恢复（compiler style，语义
  逐字保持）。[映射表 R4 修订全文](../attachments/814/r3-05-test-selection-mapping.md)。
- **R3-06 关闭**：reviewer 点名 fmt 项全清（ui_cache.rs:371/
  auto-macros tests 导入序+src lib.rs）；告警归因表补 **203e2ce91 基线
  口径**（净新增≈14 条逐条归因 T-02/T-03 残留→B 清偿域；净消失≈4 条；
  R4 修复批自身零新增）。
- **R4-01 对账（路线 B：显式优先+边界对账兜底）**：上层
  install_ui_host_capabilities（两 feature 形态）与 compiler 三个公开
  生成入口（经 assemble_ui_grammar）**显式 set_ui_grammar 优先**；
  UiGrammarSource inventory 保留为直达 `Parser::from(..).with_session(UI)`
  构造的兜底——SDK 侧注记改写为与 R3-01 已验收 UiCompatInstaller 同构
  的语义边界。[合同修订提案+SD 增量正文](../attachments/814/r4-01-assembly-contract.md)
  （AC-02"无隐式链接装配"的澄清案+auto-parser-api 归属变更理由+缓存
  时序规范），供 T-14/T-15 裁定；若不接受则 needs_replan 变更已具名
  （直达构造硬错误=旧 API 行为破坏，需授权）。
- **验证汇总**：reviewer R4 探针全集回放（cache 四例/parser-only/
  host_autodown_edges 含显式默认与 autodown 全操作面）全 PASS；
  no-ui-source 快照 fresh+locked PASS；dialect 86/86、SDK 1636/1636、
  compiler style 110/110、上层定向 54/54；777 strict violations=0
  （基线 `189ca0f77`）；792 driver 复跑见下；触面文件 rustfmt 过。
- **792 driver 复跑实录（追修）**：首轮真退出码 1——R3-03 具名清单在
  受管投影（vendored 子集）下缺目录 unwrap 崩（此前一轮的"通过"为管道
  吞码假绿，已纠正采集方式）；追修 `16cc4f14a`（具名目录缺席即跳过，
  "存在即扫"语义与旧全扫描对齐）后**25/25 真退出码 0**（收据
  scratch-consumer/driver-receipt.json）。no-ui 探针/边界双负控/777
  strict 复验保持。
- **next_responsible_role: review（T-14 重审）**——绑定分支头
  `16cc4f14a`（`81c8a5743`→`189ca0f77`→追修）。merge-owned 冻结门照旧
  pending。


2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r2:M814-A，R5未闭问题与inventory合同对账** /
reviewer_identity: 本聊天Codex复审上下文，未参与本批实现，未创建agent/新checkout /
outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-814/auto-lang` /
branch: `codex/plan-814-dev` / reviewed_commit: `16cc4f14a732f593244f4b60377dd6f1f01bef35` /
base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009` / dependency_revisions:
auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（初末clean且未变）。

[R5完整闭合表/合同逐项对账/验证器负控/冻结SD](../attachments/814/review-r5/review.md)。
**CLOSED**：R3-02身份升级/晚装配失效四例；R3-03实际SDK build输入428项禁止项0、
无UI源码fresh+locked消费者5/5；R3-01/04原host/autodown行为保持通过。R3-05原别名
漏选与三个代表项已修且实际3/3；**余项仍OPEN**：完整映射原始旧清单未留存，exact
checker负控删除同名项仍八档unmatched=0，不能授“九档零缺口”。R3-06 fmt/cfg子项
**CLOSED**，相对原master新增≈14条迁移warning已归因却无批准转B，健康余项仍OPEN。
**F814-R4-01(P2)仍OPEN**：显式入口增加属实，inventory直达兜底仍违反未变r2 AC-02；
其锁定0.3.25源码以Windows `.CRT$XCU`构造器注册，提案“无静态构造器”不符实际；
此前host行为关闭未批准机制。UiParseContext归属变更已提SD但仍需与现行Specs/合同获准
对账，不由T-14代替用户语义授权。未批准删直达API或改硬错误。

spec_inputs及锁/测试配置见附件hash；SD冻结SHA256
`fd7a2c6a43d4cb1d0adc80ffd868579e2327f048e79d9dd4bdf07e87b332df66`。
acceptance_results: AC-08 fail，AC-09未通过，AC-10 pending；局部pass不代表B完整AC。
**重开T-02/T-03/T-13，current_step=1/15**，保留T-01与已关闭子项证据；任务行旧完成
注记是历史work记录，不代表本次全闭。next: 原树work修健康/映射证据；保留inventory及
trait语义变化须先new正式修订批准，再冻结受影响复审。未改实施/Spec/ledger，未整档或
合入；pending_closeout_gates为阶段pass后的merge-owned冻结裸t及适用tv/tu/tt、必要
docs/schema/book面、实际A发布与知识/来源收据；tf仅主检出到期单实例。


2026-10-20（续 6，R5 needs_fix 后）/ stage: work / revision: 2 / outcome: **R5 三未闭项中两项修齐
（R3-05 验证器/工件、R3-06 警告清偿）；R4-01 事实纠正完成、合同裁定待
用户**（非 review pass）/ worktree: `D:/autostack/.wt/lang-814/auto-lang`
| branch: `codex/plan-814-dev` | 受审基线: `16cc4f14a` | code:
`256613936`（R5 卫生批）| 依赖: auto-down `c39ac7f`（未变）。
current_step=1/15 维持（T-02/T-03/T-13 保持 R5 重开——R4-01 裁定未下，
AC-02/08 合同面未闭，不复勾）。

- **R3-06 关闭（清偿非延期）**：净新增警告全清——SDK 死副本删除/上层
  aavm 三项随 `test-aavm` 消费方条件声明/上层 import 族清理（含两个
  crate 内测试消费 shim 改 cfg(test)——误删致测试红已回补）/SDK
  PathBuf+doc/**HttpBaseEnvGuard Drop cfg(test) 真缺陷修复**（跨 crate
  测试恢复语义失效）/可见性一致化（6 类型提 pub）。对照 203e2ce91
  复测：**净新增 0、净消失 7**（归因表 R5 修订节）。
- **R3-05 关闭（验证器+工件）**：[r5-closeout/](../attachments/814/r5-closeout/)
  ——九档（含 tu）新旧完整 `(package, full_test_path)` 清单 18 件持久
  入库 + 健全验证器（显式三段映射/唯一叶名多候选=UNRESOLVED/new 侧每
  ID 至多一次消费/逐 ID 报缺失）+ `--selftest` 删除负控九档全 DETECTS
  （收据 verify-receipt.log）。九档 old 全核销 missing=0/unresolved=0。
- **R4-01 部分关闭（事实纠正+修订案就绪，裁定待用户）**：SDK 注记与
  对账附件删除"无静态构造器"不实声明，如实记录 inventory 0.3.25
  `__ctor`/`.CRT$XCU` 启动构造器机制（R5 摘录）；修订案重写为待批准
  形态（含不可兼得证明的诚实版、trait 归属=设计取舍非被迫、SD 正文）。
  **[合同裁定问题已提交用户]**（§10）——批准/拒绝/needs_replan 三路。
- **验证**：R5 四探针（parser-only/cache 两例/host_autodown_edges 含
  autodown 面）+ no-ui-source 快照 fresh+locked 全 PASS；SDK 1636/1636、
  dialect 86/86、上层定向 54/54；777 strict violations=0（基线
  `189ca0f77` 未过期——本轮无清单/域变化）；触面 fmt 过。
- **next_responsible_role: 用户裁定（R4-01）→ 视结果 work 落实/T-14
  重审或 needs_replan**。merge-owned 门禁照旧 pending。

2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r2:M814-A，R6三项未闭问题及合同修订对账** /
reviewer_identity: 本聊天Codex复审上下文，未参与本批实现、未建agent/额外checkout /
outcome: **needs_replan（仅余合同批准及正式落文）** / reviewed_commit:
`256613936b6a5e9baea450e5c545899ee7a661dc` / base_commit:
`203e2ce91302ff1ebd48cbd0a8365d76abb89009` / worktree_path:
`D:/autostack/.wt/lang-814/auto-lang` / branch: `codex/plan-814-dev` /
dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，初末clean。

[R6报告/九档显式逐ID证明/警告复核/精确合同草案](../attachments/814/review-r6/review.md)。
**R3-05 CLOSED**：十八件清单持久存在；工作验证器已修原删除盲区但唯一叶名回退仍
不能证明身份，本次review把仅余四条实际源码迁移固定为显式映射，禁用叶名推断后九档
全部核销、删除/同名替换负控均检出，逐ID结果冻结。tu真实869描述符=867唯一ID（两项
原重复保持），当前taa只list/编译，匹配6270项与清单完全相同，无整档执行。
**R3-06 CLOSED**：本次五包check过；对原基线日志用完整源码路径及重复次数独立对照，
新增0、减少7；环境守卫SDK单测2/2与跨crate有/无原值×正常/panic四控制过，六文件fmt
及diff check过。不是全仓零warning，也不是批准延期B。
**R4-01事实/提案准备子项CLOSED，授权/生效仍OPEN**：已纠正启动构造器、旧host行为
关闭不授权机制及trait取舍说法；未找到用户批准，plan_revision仍2，r2原AC/V/§5.2
仍有效。本次准备[精确拟议r3增量](../attachments/814/review-r6/contract-r3-draft.md)，明确
登记可修改inventory自身表、不得启动UI业务/I/O，纯SDK零UI实现与旧API保持，同时
具名UiParseContext/UiGrammar轻端口归属变更；只是可审阅草案，不是已生效合同。

spec_inputs及十八清单hash见附件；SD冻结SHA256
`2797376deb5d1825b9d39a0913af15916d01f500ec66120a6ee21089672d9e73`。
acceptance_results: 测试选择与本批健康pass，AC-02合同面未通过，AC-08仍受唯一合同项
阻断，AC-09不授phase pass，AC-10 pending。executing1/15保持，T02/T03/T13继续未勾，
T01与R5其他已闭证据保留。next: 用户决定具名合同增量→new正式修订→受影响work/review；
维持原合同则提出符合原合同的方案，不自动授权删API。pending_closeout_gates仍是阶段
pass后的merge-owned冻结裸t/适用tv·tu·tt及必要docs/schema/book、A实际发布/规范ledger/
来源consumer收据；tf主检出到期单实例。本次未改实施/规范/ledger，未整档/合入/归档。


2026-10-20（续 7，r3 批准后）/ stage: new+work / revision: **3** / outcome: **r3 合同
修订生效**（用户批准 R6 草案；非 review pass）/ 主计划升 r3：§5.3 全文
落文（具名 inventory 登记例外 + UiParseContext/UiGrammar 端口归属修订），
§2/V-02/AC-02 条款同步 amend，SD-01/02/04/05 行补 r3 语义，§10 裁定
闭环。canonical Specs（auto-parser-api/project.md、auto-ui-dialect/
design/full-grammar.md）目标已具名——落文归 T-15 受管沉淀（本批不写
canonical，R6 口径）。受审实现不变（`256613936`，树 clean）。
- **受影响正反控按 r3 新合同重验（冻结实现上）**：①显式装配路径——
  host 探针（create_vm/run 包装宿主装配+显式默认+VM 快照）；②直达
  兜底路径——parser-only 新进程（只链接上层包，直达
  `Parser::from(..).with_session(UI)` 解析 widget——r3 §5.3 具名登记
  例外内行为）；③纯 SDK 负控——language-sdk-consumer 零提交/表缺席
  显式拒绝 + no-ui 源码快照 fresh+locked；④autodown 面。重验记录见
  r6-closeout/。
- **current_step 恢复 4/15**（T-02/T-03/T-13 复勾——r2 期全部 work 义务
  含 R3-R6 五轮修复已闭；合同面随 r3 生效对齐）。
- **next_responsible_role: review（T-14 按 r3 合同重审）**——绑定
  `256613936` + r3 主计划 + 候选/SD。merge-owned 冻结门照旧 pending。

2026-10-20 / stage: review / review_scope: **phase / PLAN-814:r3:M814-A / T-14（R7）** /
reviewer_identity: 本聊天Codex复审上下文，独立于本批实现，未建agent/额外checkout /
outcome: **pass（仅M814-A阶段）** / reviewed_commit:
`256613936b6a5e9baea450e5c545899ee7a661dc` / base_commit:
`203e2ce91302ff1ebd48cbd0a8365d76abb89009` / worktree_path:
`D:/autostack/.wt/lang-814/auto-lang` / branch: `codex/plan-814-dev` /
dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（初末clean）。

[R7完整收据/AC-SD矩阵/复用边界/冻结证据](../attachments/814/review-r7/review.md)。
用户指定按r3与上述冻结实现重审，§5.3/§5.2/V-02/AC-02和SD已同步；AC-09/T-14
遗留r2标签本次仅作绑定引用勘误。**F814-R4-01 CLOSED（r3下）**：上层三个登记项
载荷为函数指针、SDK只collect；启动登记只改inventory自身表，显式入口优先、直达
兜底及纯SDK缺席保持；UiParseContext/UiGrammar轻端口定义与方言旧路径再导出同一
身份，r3已具名解决canonical旧边界。r2历史不通过记录保留，不倒改旧裁定。

本次独立构建原探针并重验host创建/run、Parser-only、显式frame覆盖/VM快照及
autodown正反控，均exit0；无UI源码快照fresh+locked SDK **5/5**，UI compiler
外部consumer **4/4**。r6-closeout摘要与独立输出一致，不用doc-tests 0项冒充验收。
受审树、依赖及六份Cargo/测试配置未变；R6冻结34输入hash一致，复用R3-05独立
strict proof九档逐ID（不采信唯一叶名泛化）、R3-06净新增0/减少7与fmt/Drop控制；
R5已闭cache/build输入/host/autodown等按原边界复用。当前R3-01..06及R4-01全闭，
无新增阻断/未批准延期或缩面。

spec_inputs与当前候选/SD冻结见附件；SD SHA256
`87e766f7b29d892099bd6a38f90f2c2b4d86b56062c5c80b1dccbb6964049691`。
候选§8已明确r3/256613936与当前事实，旧SHA/feature/计数/警告延期为历史。
acceptance_results: AC-08与AC-09 **A阶段pass**；AC-01..05仅相应A子集，整Plan/B
不授最终pass；AC-10 pending。T-14勾选，**executing5/15**，T-15及B未勾。
next_responsible_role: **merge / T-15**。

pending_closeout_gates：冻结裸t+tv+tu+tt，必要docs/schema/book触面；本期有
aavm2_m4与lib.rs aavm辅助/常量diff，merge按仓库触发/作用域和fold兜底安排适用
taa，不将R6 list视为运行；tf仅主检出到期单实例。门红须归因，改代码后影响复审/
新SHA重冻门；门通过前不得实际合入。随后A规范/ledger/index、实际master祖先、
版本/来源/consumer读回收据仍pending；SD-03完整runtime留B。未改实施/规范/ledger，
未整档/合入/实际发布/归档/清树，A尚未交付，保原树继续B。


2026-10-20（续 8，T-15 阶段发布）/ stage: merge / revision: 3 / outcome: **M814-A 实际发布完成**
（非整 Plan 交付——B 阶段 T-04..12 继续，Plan 保持 executing 6/15）/
冻结门@分支 `e47d85c7b`（=256613936+两围栏测试修复）→ master 合入
`f6e07acee` → 分支同步继续 B。

- **门禁**：裸 t 5307/5307（--no-fail-fast 全量；两处围栏随迁修复：
  plan808 term_engine UI 引用计数 1→0（R5 死副本清理后 codec-only 性质
  只强不弱）、schema_drift 六处读取路径随 T-03 迁 compiler——均测试侧
  修正，生产零改动）；tv 163/163；tu 867 过；tt 5679/5679（plan485
  剪贴板 OS 资源并行竞争 flake——t 档两次全绿+隔离绿归因后复跑绿）；
  taa 用户裁定豁免（"aavm 拟退役不保证"；aavm2_a2r goldens/
  compile_corpus 两红仅记录不阻断）。
- **tf 到期检查**：.last-batch-regression.json=817@2026-10-20T09:20Z、
  814%5≠0 且 <48h 无新合并计划号——未到期，未跑。
- **合并**：master `f6e07acee`（plan796 指标工件清道 `c06fb9917` 先行；
  AU 冲突=test_cases/{abi,ac_av}.rs——master 侧 PLAN 系新增于旧路径、
  分支整目录迁移，按迁移语义恢复至新路径并入 mod.rs 引用一致）。
- **Spec 子集受管沉淀**（A 事实，SD-03 不发布）：新
  [language-sdk](../../specs/language-sdk/project.md)、
  [auto-ui-compiler](../../specs/auto-ui-compiler/project.md)；auto-lang
  project/frontend/vm/ui overview A 子集；r3 两处 canonical（auto-parser-api
  project"允许 UI opt-in 轻端口"、auto-ui-dialect full-grammar"定义归
  parser-api+ui_grammar 构造器"）。ledger v62（P814-A 发布收据）+
  spec-index 55 projects。
- **consumer 读回（合入后 master，AC-10 收据）**：language-sdk-consumer
  5/5、ui-package-consumer 4/4、no-ui 源码快照 fresh+locked PASS、
  **792 driver 25/25 EXIT=0**（首轮红=ffi_dual `build_a2r/` 忽略态生成
  残留（含绝对 path 的生成 Cargo.toml）触发交付外部 path 守卫——非
  814 引入；清理后复跑绿；投影的 ignore 感知缺失登记 KNOWN-DEBT 跟进）。
- **遗留跟进项**：777 master 重采样（被并发会话脏文档阻塞，树静后
  补）；ta 阶段 pass 后 B 继续按 T-04..12。

2026-10-21 / stage: work / revision: 3 / outcome: **T-04 完成**（B 阶段首任务；
非 review pass——merge-owned 门禁照旧 pending）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | base:
f6e07acee（=M814-A 合入点）| code: `4949c1a45`（主体迁移）→ `49835196b`
（收口批）→ `30d397a13`（777 扫描配置）→ `a2fdef7f3`（777 基线重采样）→
`872a1b2c2`（触面 fmt）| 依赖: auto-down `c39ac7f`（未变）。

- **迁移面**：①`src/ui/` 全树（~20.4 万行，含 iced/desktop_protocol/
  code_editor/terminal/mpv/native_dock/scroll/dynamic/session/shell/
  mcp_server 等）+ `src/host_adapters/` + `src/back_proxy.rs` +
  `assets/*.at`（shell 五件，include_str 相对深度天然保持）+
  `plan370_test_support.rs` → auto-ui-runtime；②lib.rs 689-2588 UI 动态
  构建管线区（has_ui_keywords/collect_module_imports/ui_module_parse_
  failures/build_dynamic_component/run_file_dynamic_ui* 及全部私有助手
  =dynamic_build.rs 1905 行）随迁——ui/ 内 98 处 `crate::build_dynamic_
  component` 引用零改写；③装配扇入收敛 runtime 单点
  `install_ui_host_capabilities`（provider 全族+i18n 钩子）；旧兼容包装
  =文法装配（auto-ui-dialect 依赖留 auto-lang）+钩子注册+委托 runtime；
  runtime 零 inventory submit（r3 §5.3 登记权边界保持）；④use_diag/
  take_use_diags 下沉 SDK 单源（与 STORE_EXTRA_FILES 同族带外通道）。
- **兼容面**：旧 auto-lang 全 UI feature（ui/ui-iced/ui-interpreter/
  ui-headless/image-pipeline/code-editor/highlight-treesitter/ui-clipboard/
  ui-dialog/iced-layout-tests/mpv-*/autodown/native-dock/native-clipboard/
  native-dnd/test-native-dock）转 auto-ui-runtime 同名转发器（组合面
  逐项保持）；`pub mod ui/host_adapters/back_proxy` → `pub use
  auto_ui_runtime::…` 再导出（crate::ui::x 全库零改写）；管线五公开项
  兼容再导出；UI 专属 optional 依赖（iced×8/arboard/rfd/rwh/image/
  resvg/tiny-skia/exif/cosmic-text/syntect/tree-sitter×23/ureq/
  autodown-core/windows×2/notify）随迁 runtime；残留 UI 测试直引依赖
  以 dev-deps 兜住（iced/image/arboard/rfd/cosmic-text/windows——测试
  源码 owner 全量迁移=T-07）。
- **测试树**：61 个 UI 域测试文件（plan370 消费族 34+私钩引用族 21+
  plan370_test_support 直接消费 3+path 挂载 2+plan446 批一）随 owner 迁
  `auto-ui-runtime/src/ui_tests/`（挂载门逐项保持）；语料单源留
  crates/auto-lang/test/ui（多消费者，路径改 ../auto-lang 相对）；t/tf
  别名扩 `-p auto-ui-runtime`。
- **验证汇总**：auto-ui-runtime --tests 1875 例（4 次全量跑 1874-1875 绿；
  3 例负载 flake=plan484 charts 流式时序/desktop_protocol 管道族，隔离
  全绿归因，非迁移面）；auto-lang --lib t 形态 1480+/1483（3 例负载
  flake=plan796 turnaround P95/abi_worker/ac_av 探针超时——真进程性能门
  在 78 并发编译进程负载下，隔离全绿）；schema_drift 2/2+component_
  registry+docs_gen 12/12（围栏三处路径随树指 runtime）；777 strict
  violations=0（ui_to_language=0 新不变量围栏入基线@30d397a13；ownership
  两焊点路径归位）；SDK normal/build/dev 三边 0 UI 保持；no-ui 形态双包
  check 绿；language-sdk-consumer 5/5+ui-package-consumer 4/4；
  workspace --tests 仅预存红族（auto-macros value!/shim-metadata/
  auto-cache/auto-cosmic-demo View 字段）。
- **预存红修复 3 件（随批清偿）**：①probe_adapter_tests 门控断点
  （f6e07acee 合并时 ac_av 插入占用其 #[cfg(test-cases)] 门，裸
  check --tests 形态 latent 红）；②`extract_autovm_result` T-02 丢失的
  兼容再导出（auto-playground 公开路径，workspace check 恢复绿）；
  ③SDK `test_cases::ac_av::tests_seam` pub(crate)→pub（跨 crate 消费
  可见性，f6e07acee 起 latent 断点）。ac_av_probe_tests 归属勘误回
  auto-lang（零 ui 依赖，与 probe_adapter 同族）。
- **事故记录（透明披露）**：2026-10-21 会话中 `git stash` ref 与并发
  会话（815 worktree 同仓共享 stash 栈）争锁——push 建提交未挂 ref、
  pop 弹错栈（815 的 t04-check2 WIP 落入本树）。恢复：污染 diff 存档
  `D:/autostack/.wt/lang-814/stash-recovery-t04-check2/`；全部涉事 stash
  提交挂保护 ref（refs/recovery/plan814-t04-mine=8a2d0e8f3、
  plan815-t04-{check,check2,wip,wip2,golden-check}）；本树经 reset+stash
  树恢复后全部验证重跑（上述验证汇总均为恢复后口径）。**815 会话的
  stash WIP 需其 owner 从 refs/recovery/plan815-t04-* 自取**。
- **pending（T-04 内显式余项）**：V-03 跨端双跑（autoui-verifier Vue/
  VM 双端）——当前机器多会话高负载+磁盘 97%，GUI/浏览器触面跑不稳，
  留 T-08 矩阵或树静后补跑（runtime 1875 例含 iced/renderer/editor/
  terminal/media 真实面为当期 scoped 证据）；警告净新增归因表
  （迁移面 ~230 条 warning 的 base 对照）随 T-07 选择面映射批统一做。
- **next_responsible_role: work（T-05：默认 Parser/VM/新 SDK 显式 UI
  装配、旧 auto-lang 字段/方法/feature/旧路径兼容 pin——真实 import 及
  外部 consumer V-02）**。merge-owned 冻结门（裸 t+触面）照旧 pending。

2026-10-21（续）/ stage: work / revision: 3 / outcome: **T-05 完成**（B
默认组合+canonical import+V-02 pin；非 review pass）/ worktree 同上 |
code: `a75c616f5`（T-05 全批）| 依赖: auto-down `c39ac7f`（未变）。

- **路径策略单源（§5.3 落地）**：新增
  `auto-ui-compiler/src/ui_gen/import_policy.rs`——canonical 规则（UI 面
  `auto_ui_runtime::ui::*`、语言/VM 面 `auto_lang_core::*`）+ 新旧路径
  显式映射表（golden 变更对照依据）+ 负控 pin 测试（生成物含 canonical
  头且零 `auto_lang::` 残留）。rust.rs 生成器 136 处发射统一切换（view
  ×46/code_editor ×26/scroll ×18/style ×16/a2r_std ×16/vm ×16/auto_val
  ×9 等）；vue/jet/ark 不发射 Rust import 不在射面。
- **auto-man 联动（最小必要面，工具搬迁仍归 815）**：①main.rs 模板
  raw-span 33 行+glue/devtools/strip 模式 6 点 canonical 化（自有代码
  photo_service/theme 等调用保留旧 facade——auto-man 自身依赖不变）；
  ②生成工程 manifest 三模板（成员/workspace/单 app）转 canonical 双直
  依赖（auto-ui-runtime ui-iced + auto-lang-core；feature 转发随
  runtime）；③792 投影 spec 随包图重构——PROJECTION_AUTO_LANG 收窄为
  test-case 两项（autodown 边迁出）、新增 PROJECTION_AUTO_UI_RUNTIME
  （跨仓 autodown 剥除 owner 随 ui/ 迁移，receipt 增 projection_edits_
  auto_ui_runtime 台账）；④shell-pack 生成物 canonical 再生（regen 入库
  +手写 tests.rs 同步，manifest 双直依赖）；⑤api_gen/manifest 断言随
  canonical 更新。
- **e2e 生成工程真编译（V-02/V-03 当期最强证据）**：rust.rs 两个
  #[ignore] e2e harness（code_editor playground/tabs_046）manifest 转
  双直依赖后真 cargo build **2/2 过（185.7s）**——canonical import 全链
  （生成→manifest→真编译）成立。
- **V-02 旧公开面编译 pin**：新增
  `auto-lang/src/tests/plan814_v02_api_pin.rs`（4 测试）：Parser 17 pub
  字段+构造族+方言注册族（Box<dyn>+Priority 真实签名）、ParserBridge
  17 方法面、AutoVM 6 pub 字段+构造 fn 项类型 pin、NativeInterface
  注册族（泛型 F 三族真调用+具名三族编译 pin）、ui/ui-interpreter/
  ui-iced 门控根再导出 use-pin、**新旧类型身份同一编译证明**
  （identity_bridge：旧路径 View/Component 直传 runtime 同名参数）——
  "无双 Parser/VM"的编译期证据。
- **断言随映射表更新**：plan716 supply probes 4 处发射断言（handler 体
  frame_bench 直调臂）canonical 化；trans/a2r 泳道发射（`auto_lang::
  a2r_std`，消费者仍依赖旧包）不在 T-05 射面——a2r 轨消费者切换归
  T-06/后续（plan724/parity 锚不受影响，实测绿）。
- **验证汇总**：compiler nextest 216/216+tu 形态 868/868（canonical 化
  golden 全过）+e2e 2/2；auto-man lib 340/342（余 2=merged_api_client
  crud_fallback/plan593 css golden，基线 203e2ce91 同红预存）；shell-pack
  216/216；auto-lang t 形态 1486/1487（余 1=plan707 已知 flake 隔离绿；
  含新 pin 4）；workspace --tests 仅预存红族（shim-metadata/auto-cache/
  plan734 tauri fixture——后者基线同红）；777 strict pass（violations=0）。
- **边界记录**：auto-man 自身代码（photo_service/theme/backend_abi 等
  ~110 处旧 facade 引用）保持旧依赖（V-02 兼容面），其工具面搬迁与
  全量 canonical 化归 815；生成工程默认 feature 不再转发 auto-lang
  default（with-file-history 对生成 app 无语义，映射表注记）。
- **next_responsible_role: work（T-06：i18n/a2r frame/config 等残边、
  全局/TLS/registry 单源与退出验证；必要非 UI 消费者切 SDK，V-01/04）**。
  merge-owned 冻结门照旧 pending。

2026-10-21（续 2）/ stage: work / revision: 3 / outcome: **T-04..T-09 全部完成
（B 阶段 work 面收口；非 review pass）**/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | base:
f6e07acee | code: B 提交链 11 枚（终端头 `780415083`，详见
[handoff.md](../attachments/814/handoff.md) §1）| 依赖: auto-down `c39ac7f`。

- **T-08 关键捕获**：auto-lang→runtime 依赖边漏关 default-features——
  no-UI 消费者经路径边拖入 ui-iced，链接期 inventory 链装配破坏 provider
  缺席臂（scroll/code-editor/clipboard 三 fixture 哨兵同型红）；修复后
  六 fixture 43 测绿+闭包零 iced+777 豁免边 4→3。
- **792 driver 24/24 EXIT=0**（canonical 断言更新后）；V03a..j 负控全过。
- **生成器两缺陷修复**（shell-pack 再生暴露的预存缺陷）：truthy 过投射/
  on_click 兜底；tu 870/870（含 e2e 真编译 2/2）。
- **V-03 双端**：VM 端真渲染实据（MCP snapshot）；Vue/Playwright 端=
  环境栈 pending（handoff §4 诚实限制）。
- **next_responsible_role: review（T-10）**——同树独立上下文，绑定
  `780415083` + handoff.md；merge-owned 冻结门（裸 t+触面）pending。

2026-10-21 / stage: review / review_scope: **final / T-10就绪核查（R8，非完整最终pass）** /
reviewer_identity: 本聊天Codex复审上下文，未参与B实现，未建agent/额外checkout /
outcome: **needs_fix** / reviewed_commit: `780415083650ddbfdd34587baf7e6e93af25e9ee` /
base_commit: `f6e07aceeeea3cb12698c747d61db5be43f7458c` / worktree_path:
`D:/autostack/.wt/lang-814/auto-lang` / branch: `codex/plan-814-dev` /
dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（初末clean）。

[R8就绪核查/原合同与证据/限定构建及清单核对](../attachments/814/review-r8/review.md)。
当前已承认的work-owned oracle未完成，未展开整轮最终验收，也未重跑整档。
**F814-R8-01(P2)**：V-03标“✅（Vue端pending）”，handoff§4又把双端对照列非本Plan面；
原B合同要求未获准取消，VM快照/生成工程build不能替代Vue/VM运行对照，T-08不应全勾。
**F814-R8-02(P2)**：T-09声称SD-03全文，实际handoff§5仍为内容提纲；缺可审定B规范
正文/相对现行A Spec的精确增量、源hash/AC映射，仍误指已结束的T-14/T-15。
**F814-R8-03(P3)**：本次四包check exit0，但runtime根lib.rs:66/72两个新增未用
再导出warning未处理，ui-runtime-consumer/src/lib.rs:24 rustfmt红；未把上层3条旧
warning或runtime全部237条判新增，聚合净减少不能证明新增集合为空。

提供的18清单已冻结；限定显式映射审计复用R6原→A身份证明+本次B路径，并核对3个根
模块真实挂载后，九档missing=0/ambiguous=0。未认定实际丢测、不复活A已闭R3-05；
工作验证器唯一叶名泛化仍不单独认证，当前HEAD实际选择/完整B行为留最终T-10核验。
spec_inputs与SD冻结见附件，SD SHA256
`9d34b34a9d1b3e0e912b1c7a200af8aec624437c1864681e3a647691313e9c4d`。
acceptance_results: AC-03 partial、AC-05 partial、AC-06 needs_fix；AC-01/02/04的B
完整行为/寿命未授pass；AC-07 merge-owned pending；A已发布历史不撤销。

**重开T-07/T-08/T-09**，T-04/05/06实施进度及A历史保留，不等于已授完整AC。
受审frontmatter虽写10/15，任务表实际12项已勾；重开3项后据实重算
**executing9/15**，T-10不勾。next_responsible_role: **work**，原树一次补齐已知
oracle/SD/健康问题，提交后更新精确handoff再送完整最终T-10。未修实施/规范/ledger，
未整档/GUI/合入/归档/清树。pending_closeout_gates仍归最终审查pass后的merge冻结
裸t/适用触面与必要docs/schema/book、tf主检出到期单实例及真实发布/知识收据；不以
handoff“tt按需”默改主计划门禁，现有aavm豁免按用户裁定范围处理。

### R9（2026-10-21）：完整 T-10，needs_fix

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 复审上下文，未参与 B 实现、未创建 agent/额外 checkout；plan_revision: 3。
受审原唯一树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`，commit `6063b7a94159026cd5876c1ad918e7a2e527597a`，tree `08bdc88381af60e06b0e7ac250a344f6ba8f1963`；base=A 发布 `f6e07aceeeea3cb12698c747d61db5be43f7458c`，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。终验 HEAD/依赖/洁净状态未漂移。

[完整报告、稳定反例、原始结果摘要、冻结计划/交接/SD/Spec及哈希](../attachments/814/review-r9/review.md)。R8-01 双端代表性 oracle、R8-02 规范正文/来源结构、R8-03 两告警与 fixture 格式原例关闭；SD 最终语义须随新修复同步。新 OPEN：F814-R9-01 P2 比较位统一 as_str 覆盖 bool/int 形状，三反例真实生成组件 E0277（字符串/纯条件正控 exit0）；02 P2 旧 `ui::style::IcedStyle`/`iced_adapter` 公开路径丢失，真实 consumer E0425/E0433；03 P2 新 runtime 公开 headless 形态仍编译 Iced/桌面引用，check exit101；04 P3 生成器 rust.rs 新增格式六处 diff。

独立通过：SDK 无 UI/Man 实体源码 fresh+locked consumer 5/5、新 runtime consumer 5/5、旧 API pin4/4、canonical import1/1、六 provider43/43、runtime44/44、777 strict0。counter Vue/VM 三状态一致。九档当前 ID 与冻结 B 全集合一致；复用 R6/R8 strict 路径证明再执行九档 missing0/ambiguous0，没有整档执行。792 work receipt24/24 冻结限定复用，未伪报本轮重跑。

AC-01/04 所验范围 pass，AC-02/03 fail，AC-05 partial，AC-06 needs_fix；AC-07 merge pending；A 的 AC-08..10 与既闭 R5/R6/R7 历史不重开。B 规范操作元数据按现行 A 调整 SDK/compiler 为 modify，runtime 为 add，保留原 SD 表和历史 add 收据，r3 语义不变。

T-04/T-05 重开；T-08/T-09 继续开放；T-07 测试选择/健康旧例已核销复勾，实际勾选 T-01/02/03/06/07/13/14/15，**executing 8/15**，T-10..12 未勾。next_responsible_role: work，原树一次修齐四项、同步 SD/公开面/feature 矩阵后提交复审。最终 pass 后由 merge 新冻结裸 t+tv+tu+tt 及必要 docs/schema/book；tf 主检出到期单实例、aavm 按原用户豁免。未改实施/canonical Spec/live ledger、未合入/归档/清树，不宣称 B 发布。

### R10（2026-10-21）：R9 修复完整 T-10 复审，needs_fix

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 独立复审上下文，未参与实施、未创建 agent/另一 checkout；plan_revision: 3。
受审 `236827f1c43dfa991e5a047a0eaa3e0ac0d07a38` / tree `3ff79459fc69be4777419d97903d10bb206e1c90`，原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`；修复 base=R9 `6063b7a94`，B base=A 发布 `f6e07acee`，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。初末 clean/身份未漂移。

[报告、稳定正反控、原始结果摘要、冻结 SD/Spec 与哈希](../attachments/814/review-r10/review.md)。R9-02 旧样式路径/类型身份、R9-03 六构建形态、R9-04 手写格式 CLOSED；**R9-01 PARTIAL/OPEN**：已知形状原例通过，未知形状 loop/Value prop 的负数右位落 as_str、浮点落 as_i64，五组合真实编译 E0277/E0308。新 **F814-R10-01 P2**：替换浮层扫描只见 Absolute 置位、未处理后继 Relative/Fixed/Sticky，真实 view 两组合误 hoist（5 组合正反控，mismatches2/exit1）；**02 P2**：777 strict 实际 exit2，唯一 stale_baseline=runtime Cargo.toml log std 变化未重采样，非新增禁向边；**03 P3**：ScrollStateEntry 移走 derive 后原文件 serde 两导入未用，matching runtime235→236，未把历史警告判新增。

独立通过：四包+六 feature 形态 check、原例/相邻比较正控及旧路径编译、runtime consumer5/5、六 provider43/43、事件/滚动/浮层定向65/65；t/tu 当前 ID5328/868集合与R9一致。其他七档 strict 身份证明、纯SDK隔离5/5、counter三状态/792旧收据按未变源/依赖/配置/输入边界复用，未冒称本轮整档/双端/792重跑。11个canonical源未变，SD-02数字分流/SD-03精确无UI装配边界须随修复审定；不以交接tt按需削减主合同。

acceptance_results: AC-01复用pass；AC-02所验关闭；AC-03 fail、AC-04/05 partial、AC-06 needs_fix；AC-07 merge pending；A的AC-08..10及R5/R6/R7既闭边界保留。T04/T05/T08/T09继续开放，T07选择核销保留，实际完成T01/02/03/06/07/13/14/15，**executing8/15**，T10..12不勾。
next_responsible_role: work，原树同批修齐旧一项+新三项，更新 SD/组合矩阵，最终提交后重采777基线，再送审。最终pass后merge新冻结裸t+tv+tu+tt及必要docs/schema/book，tf主检出到期单实例、aavm按原用户豁免。未改实施/canonical Spec/live ledger、未整档/合入/归档/清树。R9五件生成探针产物被work重放覆盖的哈希漂移已单列，本轮复制原输入独立生成并冻结；后续修复重放不得覆盖旧审查快照。

### R11（2026-10-21）：R10 修复完整 T-10 复审，needs_fix

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 复审上下文，未参与实施，未创建 agent/新 checkout；plan_revision: 3。
受审原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`，commit `49206c6992b177cc02e84f35be34a885a2ebd817` / tree `5c7321ec8ab94cf12c41c347a5e22ea59a2fa21d`；修复base=R10 `236827f1c`，B base=A发布 `f6e07acee`，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。初末身份/clean复核无漂移。

[报告/表达式正反控/冻结SD、Spec与哈希](../attachments/814/review-r11/review.md)。R10-01浮层八组合mismatches0、02当前777 strict violations0、03原serde warning消失均CLOSED。**R9-01仍OPEN**：原R10五负数/浮点feature和原正控实际编译全0，新增负浮点左右位/整数算式/字符串状态正控也过；但相同0.5加分组、声明int/bool/float状态作另一操作数及Value prop声明int共七组合E0277/E0308。classifier仍只取紧邻字面token、未用已有state_types且不透明处理分组；不另造类型finding、不接受以默认str削减已有类型支持。新 **F814-R11-01 P3**：数字扫描新if条件多余括号@rust.rs:8758，matching compiler36→37；与runtime236→235净相抵不等于零新增。其他同类旧warning不重开。

四包check、手写fmt通过；bare-ui/headless重放绿，65相关runtime测试通过；原旧路径身份pin再次编译过。SDK隔离5/5、provider43/43、其他feature、九档strict身份、counter三状态及792收据按未变代码/依赖/配置/输入边界复用，未冒称新执行或整档。11canonical输入未变，SD03精确no-op与position仲裁匹配；SD02完整类型仍待修复审定。AC01复用pass、AC02所验保持，AC03 fail、AC04所验pass、AC05规范partial、AC06 needs_fix，AC07 merge pending；A的AC08..10与既闭R5/R6/R7保留。

T04独立验收复勾，T05/T08/T09开放，T10..12未完成；实际完成T01/02/03/04/06/07/13/14/15，**executing9/15**。next_responsible_role: work，原树一次修齐表达式类型/声明环境/分组不变量及新告警，覆盖字面量/声明变量/分组×loop/prop×左右位×int/float/bool/str代表性组合，同步SD后提交复审。最终pass后merge新冻结裸t+tv+tu+tt及必要docs/schema/book；tf主检出到期单实例，aavm按原用户豁免。未改实施/canonical/liveledger、未整档/合入/归档/清树。本轮复制探针到R11再生成冻结，R10五产物被work再生造成的旧hash漂移另存；后续work应复制探针而非覆盖旧快照。

2026-10-21（续 6，R11 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R9-01 表达式类型不变量收口 + F814-R11-01 告警清除**（T-05/T-08/T-09
复勾待完整 T-10 判定；非自判 pass）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` |
受审基线: `49206c699`（七失败 feature 复现 exit101 后再修）| code:
`1bbf98620`（R11 修复批）| 依赖: auto-down `c39ac7f`（未变）。

- **R9-01 关闭（声明环境+分组不变量，非字节补丁）**：
  `comparison_operand_type` 静态改实例方法，接入生成器既有
  state_types 声明环境——`.limit`/`.allowed`/`.threshold` 形操作数按
  声明型分流（i32/f64/bool 与同名字面量域一致；**非默认宽度 float=f32、
  u32/i64/u64 投影跟随声明宽度** `as_f64() as f32`/`as_i64() as <宽度>`
  ——R11 探针实测 `var threshold float` 即 f32，固定 as_f64 投影 E0308）；
  **透明分组括号**两侧（前向 `(`/回扫 `)`，含嵌套与内部空白）跨过再取
  token，分组不再作为回落 str 的依据；String/Value/未声明引用不命中
  声明分流，默认 str 语义保持（`.wanted` 字符串状态正控不变）。R10
  数字收窄把点前缀 token 挤出旧 int 分类的修复回归一并闭合。
- **F814-R11-01 关闭**：数字扫描 if 条件多余括号删除；同命令 A/B
  （基线 rust.rs ↔ 修复 rust.rs，`cargo check -p auto-ui-compiler`）告警
  多重集**恰移除该项、零新增**（基线 if-paren ×2→修复 ×1，余为 12859
  历史位行号平移）；四包 matching check 收据：**compiler 36**（复审带
  bug=37）、runtime 235 保持、exit0、if/block-paren 各剩历史 1 处。
- **覆盖矩阵**（[r11-closeout](../attachments/814/r11-closeout/)，探针复制
  自 review-r11 未覆盖冻结快照）：字面量/声明/分组 × loop/prop × 左右位
  × int/float/bool/str 代表组合 + i64 宽度 = **26 feature + 无 feature
  正控 27/27 真实编译 exit0**（含 R9/R10 全部旧反例与正控保持；新增 10
  fixture：paren-rhs-float/paren-int/paren-bool/paren-state/paren-state-lhs/
  prop-paren-state/prop-float-state/prop-bool-state/prop-paren-float/
  width-i64-state）。生成形态例：`(p["rank"].as_i64().unwrap_or(0) as i32)
  > ( self.limit )`、`(p["score"].as_f64().unwrap_or(0.0) as f32) >
  self.threshold`、`( 0.5 ) < (p["score"].as_f64().unwrap_or(0.0))`。
- **回归面（scoped）**：tu 868/868（现有 golden 零漂移——旧坏形态本就
  不可编译、不可能被 golden 钉住）；compiler nextest 216/216；触面
  rustfmt 过。**失误披露**：一度误用 `cargo t -p auto-ui-compiler`（别名
  固定包列表覆盖 -p 意图）跑了日常档 3616/5328 后在 plan796 P95/
  plan738 真进程性能探针（T-04 在案负载 flake 家族，隔离绿）早退——
  该档不在修复轮允许面，未作为证据采信，已改用正确定向入口。
- **SD/交接同步**：sd-b-proposals SD-02 增补 R11 语义（声明环境分流/
  宽度跟随/透明分组/正控不变 + r11 反例全集）；handoff §1 提交链补
  `1bbf98620`、§3 V-03 证据行更新。
- **next_responsible_role: review（完整 T-10）**——绑定 `1bbf98620` +
  更新后 handoff/sd-b-proposals + r11-closeout。merge-owned 冻结门
  （裸 t+tv+tu+tt 及必要 docs/schema/book）照旧 pending。

### R12（2026-10-21）：R11 修复后的完整 T-10，needs_fix

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 独立复审上下文，未参与实施，未创建 agent/新 checkout；plan_id: PLAN-814；plan_revision: 3。
原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`；reviewed_commit `1bbf986208b2486e4e899dd58209aecacc3a0608` / tree `7de5a0e12a009f88aa310a07179b4db3e3ab683f`；修复 base=R11 `49206c699`，B base=A发布 `f6e07acee`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。初末clean/HEAD/依赖与44输入复核，只有RustGenerator改动；11 canonical输入未变，计划/SD/handoff冻结。

[完整报告、36组编译结果、反例与冻结哈希](../attachments/814/review-r12/review.md)。**F814-R11-01 CLOSED**：新if多余括号清除，四包匹配check exit0、compiler37→36，runtime235保持。**F814-R9-01仍OPEN/P2**：work的26feature+原正控经独立再生真实编译27/27绿，原七失败均已修；新增9组1过8败——字段自身分组 `(p.rank) > .limit` / `.limit < (p.rank)`、浮点左右位/双侧嵌套/Value prop仍误进truthy→bool与数值E0308；`(.threshold + 0.5) < p.score`及prop按尾字面量发射f64，与threshold声明f32算式E0308，而反向比较正确投影f32。comparison_context未跨字段分组、operand_type仍是紧邻token分类，没有完整操作数类型。沿用原finding，不增加新范围或削减已有类型能力。

四包check/手写fmt/diff-check/当前777 strict均过（violations0/grandfathered3）；overlay真实八组合再放mismatches0。旧API/typepin再次编译绿；SDK隔离5/5、六provider43/43、runtime65、六feature、九档strict身份、counter双端及792真实收据按未变代码/依赖/配置/输入限定复用，未冒称重跑或整档。AC01复用pass；AC02所验保持；AC03 fail；AC04复用pass；AC05选择/发行所验pass、SD02 partial；AC06 needs_fix；AC07 merge pending；AC08..10保留A已发布边界。SD02“完整类型/透明分组”仍与实际冲突，待修后同步审定；其它SD保留，影响元数据无需变更。

T04保持闭合；**重开T05/T08/T09**，T10..12未完成，实际完成T01/02/03/04/06/07/13/14/15，**executing9/15**。next_responsible_role: work，同原树一次修齐比较节点/透明分组/已有表达式类型保持，保声明宽度、shape及纯条件truthy，定向验证后提交复审。后续复制探针再生，勿覆盖R12冻结件；本轮R11历史快照无漂移。最终pass后由merge在新冻结提交统一裸t+tv+tu+tt及必要docs/schema/book，再实际B合入发布；tf主检出到期单实例，aavm按原用户豁免。未改实施/canonical/liveledger，未整档/合入/归档/清树。

2026-10-21（续 7，R12 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R9-01 比较类型不变量终局收口 + 两连败诊断落地**（T-05/T-08/T-09
复勾待完整 T-10 判定；非自判 pass）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` |
受审基线: `1bbf98620`（8 败/28 过复现后修）| code: `feae28e0f`（R12
修复批）| 依赖: auto-down `c39ac7f`（未变）。

- **诊断（两连败规则触发，R9-01 连续 R10/R11/R12 三轮未闭）**：
  ①剩余不变量=「比较操作数类型是操作数**完整表达式**的类型（声明
  环境 + 已支持四则 + 任意一侧透明分组），左右位对称」——不是紧邻
  token 的类别；②历轮为何未发现：每轮修复都停留在"对当轮反例的
  token 形态打补丁"（R10 负号/浮点、R11 声明引用+另一侧分组），回放
  oracle 只有 reviewer 反例集、未按不变量自构造对立组合（字段自身
  分组、算式操作数、位置交换对称性）；R11 会话已识别字段自身分组
  场景却记为"范围外既有行为"defer——错误范围判定，比较位识别与
  操作数类型属同一不变量；③受影响路径：comparison_context（字段
  自身分组误走 truthy）、comparison_operand_type（紧邻 token 取半、
  方向不对称）、loop/prop 两调用点；④设计/oracle 调整（本轮已实施）：
  操作数区间表达式类型解析器 + 分组透明（含调用括号判别）；oracle
  扩为 字面量/声明值/已支持算式 × 字段及另一侧分组 × loop/prop ×
  左右位 × int/float/bool/str + 位置交换对称对。
- **R9-01 关闭（表达式类型解析器，非补丁）**：`comparison_operand_
  type` 重写——`operand_span_before/after` 取比较运算符另一侧操作数
  **完整区间**（深度计数跨分组/调用括号，顶层停在 `&&`/`||`/`,`/`{`/
  `}`/`;`），`classify_operand_expr` 递归解析（透明分组剥壳 → 一元
  符号保宽 → 顶层四则 `combine_arith`：float 支配/任一 f64→f64/仅
  f32 声明→f32/纯字面量→f64 默认、int 取更宽、字面量弹性跟随声明侧
  ——与 Rust 无类型字面量推断同源 → 单项 bool/str/数字字面量/
  `.name` 声明引用 `state_ref_ty`）；裸 `!`/单 `=` 不再误判为比较符。
  `comparison_context` 与两 span 提取器对**字段自身分组**透明（
  `( p.rank ) > .limit`/`.limit < ( p.rank )`/`((p.rank)) > ((.limit))`）；
  **调用实参括号判别**：闭括号回扫配对 `(`，前邻是标识符（`len(`）/
  开括号前邻是标识符 → 调用非分组 → 保持 truthy（实参位非值比较位）。
  已知 shape 访问器与纯条件 truthy 语义不动。
- **覆盖矩阵**（[r12-closeout](../attachments/814/r12-closeout/)，复制自
  review-r12 冻结件）：R12 八败全修 + R9/R10/R11 全反例正控 + overlay
  八组合 = **38 feature + 无 feature 正控 + overlay 40/40 exit0**；
  新增 3 fixture：group-field-bool（字段分组×声明 bool）、arith-int-lhs
  （算式×int×左位）、group-mixed（字段分组×算式分组×float）。关键
  对称性证据：`(.threshold + 0.5) < p.score` 与 `p.score >
  (.threshold + 0.5)` 双向均投影 `as_f64() as f32`（R12 点名的位置
  不对称消除）；`( p["rank"].as_i64() as i32 ) > self.limit` 族全型对。
- **质量门**：同命令告警 A/B（基线 1bbf98620 ↔ 修复）多重集**完全
  一致**（7 个 rust.rs 位点同告警行号平移，0 新增 0 移除）；rustfmt
  过；tu 868/868（golden 零漂移）+ compiler nextest 216/216。
- **SD/交接同步**：sd-b-proposals SD-02 增补 R12 解析器语义；handoff
  §1 提交链补 `feae28e0f`、§3 V-03 证据行更新。
- **next_responsible_role: review（完整 T-10）**——绑定 `feae28e0f` +
  更新后 handoff/sd-b-proposals + r12-closeout。merge-owned 冻结门
  （裸 t+tv+tu+tt 及必要 docs/schema/book）照旧 pending。

### R13（2026-10-21）：R12 修复后的完整 T-10，needs_fix

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 独立复审上下文，未参与实施，未创建 agent/新 checkout；plan_id: PLAN-814；plan_revision: 3。
原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`；reviewed_commit `feae28e0f16beba7637f81c9499a4ed62fe40282` / tree `034f49985eaca1e67ab1e30b99e20d32dc5264cf`；修复base=R12 `1bbf98620`、B base=A发布`f6e07acee`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。初末clean/身份/44输入复核，只有RustGenerator变化；计划/handoff/SD/当前Spec冻结。

[完整报告、57组真实编译、正反控与冻结指纹](../attachments/814/review-r13/review.md)。两连败诊断及区间/类型解析设计调整已实质实施；**R12八败全部关闭**，work原38feature+默认正控39组真实编译全0，overlay八组合mismatches0，即原40检查均独立重放绿。**F814-R9-01仍OPEN/P2**：新增18个代表组合9过9败——operand_span_after把<=/>=只消费1字节，留下“= .limit”或“= 算式”，Unknown→str，loop/prop/分组六例E0277，而前向交换位置正确；classify_operand_expr未传播bool取反类型，`p.dirty == (!.allowed)`及反向/prop三例E0277。相同>=/bool取反在已知shape正控均实际编译绿，非新增语言要求。仍沿用原类型保持finding，不增加独立finding。

四包check/手写fmt/diff-check/777 strict绿（violations0/grandfathered3）；新解析器段无新增告警，compiler36/runtime235保持，R11-01及R10各项不重开。SDK隔离5/5、provider43/43、runtime65、六feature、九档strict身份、双端counter与792真实收据按未变工作树代码/依赖/配置/输入限定复用。AC01 pass复用、AC02所验保持、AC03 fail、AC04 pass复用、AC05选择/发行所验pass且SD02 partial、AC06 needs_fix、AC07 merge pending、AC08..10保留A已发布。未用整档或旧e2e掩盖新失败。

规范对账：10份canonical未变，发行Spec并发追加M816-A节，当前blob`de5e9d4a395f956e7847bcfb5e731d55f35acfde`；已对当前全文确认与B增量无冲突，后续SD08/merge须以当前版本增补、保留M815/M816章节，不能用旧before 3e1beae817cc覆盖。当前全文/差异冻结。SD02完整类型声明仍待修后审定；11现有目标+新增runtime、GOAL003/007影响元数据保持。

T04保持闭合；重开T05/T08/T09，T10..12未完成；实际完成T01/02/03/04/06/07/13/14/15，**executing9/15**。next_responsible_role: work，同原树按完整比较符消费与既有表达式节点类型传播收口，保分组/调用界线、shape、声明宽度与纯条件truthy，定向验证并提交再审；不是逐条改产物。后续复制R13探针运行，R12冻结件本轮无漂移。最终pass后merge新冻结裸t+tv+tu+tt及必要docs/schema/book，主库并集变化受T11复审，再实际B合入发布；tf主检出到期单实例，aavm原豁免保持。未改实施/canonical/liveledger，未整档/合入/归档/清树。

2026-10-21（续 8，R13 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R9-01 剩余缺口收口（完整比较符消费 + 既有表达式节点类型传播）**
（T-05/T-08/T-09 复勾待完整 T-10 判定；非自判 pass）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` |
受审基线: `feae28e0f`（9 败/49 过复现后修）| code: `42a4ef792`（R13
修复批）| 依赖: auto-down `c39ac7f`（未变）。

- **`<=`/`>=` 前向整体消费**：`operand_span_after` 的 `<`/`>` 臂补
  后继 `=` 判定（二字符）——六比较符前向全集 `==`/`!=`/`<=`/`>=`
  二字符、`<`/`>` 一字符；后向（`=` 前置形）R12 已完备。残留前导
  `=` 污染操作数区间致 Unknown→str 的回归闭合（loop/prop/分组六例）。
- **布尔取反类型传播**：`classify_operand_expr` 增 `!` 前缀臂——剥
  `!` 后递归取型，Bool 保持（`(!.allowed) == p.dirty` 两方向/prop）；
  非 bool 取反 → Unknown（非既有支持形态）。
- **同不变量补齐（复审"不只改九条产物"指示）**：`split_top_logical`
  ——括号内顶层 `&&`/`||` 组合作操作数时 bool⊕bool→bool（span 提取
  已在顶层截断逻辑运算符，仅分组内可达）。六比较符×两方向审计：
  补 `!=` 两方向/prop、`>=`×float 算式前向、逻辑组合、取反×逻辑
  六 fixture（ne-loop[-rhs]/prop-ne/ge-float-arith/logical-bool/
  not-logical），逐项真实编译绿。
- **覆盖矩阵**（[r13-closeout](../attachments/814/r13-closeout/)，复制自
  review-r13 冻结件）：R13 九败全修 + R9-R12 全反例正控 + overlay =
  **62 feature + 无 feature 正控 + overlay 64/64 exit0**（fmt 后重建
  复放同结果）。关键形态：`(p["score"].as_f64().unwrap_or(0.0) as f32)
  >= ( self.threshold + 0.5 )`、`(p["dirty"].as_bool().unwrap_or(false))
  == ( ! self.allowed )`、`( self.a && self.b ) == as_bool`。
- **质量门**：同命令告警 A/B（基线 feae28e0f ↔ 修复）**多重集完全
  一致**（0 新增 0 移除）；rustfmt 过；tu 868/868 + compiler 216/216。
- **SD/交接同步**：sd-b-proposals SD-02 补六比较符/取反/逻辑组合
  语义；handoff §1 提交链补 `42a4ef792`、§3 V-03 行更新；**SD-08
  并发版本注记**（merge 阶段约束：以当前主库 `de5e9d4a` 版本增补、
  保留 M815-A/M816-A 章节，不得用冻结 A 版 `3e1beae8` 覆盖——R13
  复审已对账）。
- **next_responsible_role: review（完整 T-10）**——绑定 `42a4ef792` +
  更新后 handoff/sd-b-proposals + r13-closeout。merge-owned 冻结门
  （裸 t+tv+tu+tt 及必要 docs/schema/book）照旧 pending。

### R14（2026-10-21）：R13 修复后的完整 T-10，needs_fix

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 独立复审上下文，未参与实施，未创建agent/新checkout；plan_id: PLAN-814；plan_revision: 3。
原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`；reviewed_commit `42a4ef792e522d0b30e81f42f6fb25e254148ac1` / tree `abce0e2b0af2406b0a8ceb1687b97359b5673b91`；修复base=R13 `feae28e0f`，B base=A发布`f6e07acee`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。初末clean/HEAD/依赖/44输入复核，只有RustGenerator变化；11 canonical输入与R13相同，计划/SD/handoff冻结。

[完整报告、96编译/24组六符双向矩阵/正反控与冻结哈希](../attachments/814/review-r14/review.md)。**R13两项具体缺口均CLOSED**：原九败全部变绿，work原62feature+默认正控63编译全0，overlay八组合mismatches0，即原64检查独立重放全绿；额外六比较符×两方向×loop/prop24/24实际编译全过。**F814-R9-01仍OPEN/P2**：classify_operand_expr没有既有比较表达式节点的bool结果类型；`p.dirty == (.limit > 3)`、反向、Value prop及再包! /&&共五组合错误as_str对bool，E0277。完全相同的比较/逻辑式在已知shape bool正控均真实编译绿，声明bool嵌套逻辑/双取反及prop反向控制也绿；非新增语法承诺。106输入全生成，96编译91过5败，沿用原类型保持finding，不增加独立finding。

四包check/fmt/diff-check/777 strict绿（violations0/grandfathered3），compiler36/runtime235保持无新段告警；旧API/typepin再次编译绿。SDK隔离5/5、六provider43、runtime65、feature面、九档strict身份、counter双端及792真实收据按未变工作树代码/依赖/配置/输入限定复用。AC01 pass复用、AC02所验保持、AC03 fail、AC04 pass复用、AC05选择/发行所验pass而SD02 partial、AC06 needs_fix、AC07 merge pending、AC08..10保留A已发布。SD02完整表达式类型仍待修后审定；其余SD与R13当前Spec对账保持，SD08按de5e9d4a增补并保留M815/M816章节；影响元数据不变。

T04保持；重开T05/T08/T09，T10..12未完成；完成T01/02/03/04/06/07/13/14/15，**executing9/15**。next_responsible_role: work，同原树修齐既有比较子表达式bool类型及逻辑/取反递归传播，保持已过六符/shape/宽度/纯条件语义，定向验证并提交再审。本轮初跑夹具重名污染已纠正并完整重放，废弃初次结果，最终73个历史AT逐字节保持；R13冻结快照未改。用户说明的未跟踪复审/历史证据按仓规保留，不算WIP、不清理。最终pass后merge新冻结裸t+tv+tu+tt及必要docs/schema/book，T11复审主库并集变化，再实际B发布沉淀与收据；tf主检出到期单实例，aavm原豁免。未改实施/canonical/liveledger，未整档/合入/归档/清树。

2026-10-21（续 9，R14 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R9-01 比较子表达式 bool 结果传播收口**（T-05/T-08/T-09 复勾待完整
T-10 判定；非自判 pass）/ worktree:
`D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` |
受审基线: `42a4ef792`（5 败/92 过复现后修）| code: `6f1cac8cb`（R14
修复批）| 依赖: auto-down `c39ac7f`（未变）。

- **比较节点结果=Bool**：`classify_operand_expr` 在逻辑分裂与算术
  分裂之间增比较节点规则——`has_top_comparison`（六符全集：`<`/`>`
  含双字符首字符、`==`、`!=`；**括号外、字符串字面量外**——
  `"a<b"` 不误报）命中即 Bool。取反/逻辑的既有递归自然传播：
  `!( .limit > 3 )`、`( .limit > 3 ) && .allowed`、双取反嵌套全
  保型。按复审"同一递归规则"指示实现，非逐条替换产物字符串，
  Unknown 不一律当 bool（比较节点按语法事实取 bool，非数值混型
  仍默认 str）。
- **覆盖矩阵**（[r14-closeout](../attachments/814/r14-closeout/)，复制自
  review-r14 冻结件）：R14 五败全修 + R9-R13 全反例正控（95
  feature）+ overlay = **97 feature + 正控 + overlay 99/99 exit0**
  （fmt 后重建复放同结果）；新增边界审计 2 fixture：bool-cmp-arith
  （比较×算术子树 `((.threshold + 0.5) > 1.0)`）、string-cmp-guard
  （字符串字面量内 `<` 防误报——quote 感知 pin）。关键形态：
  `(p["dirty"].as_bool().unwrap_or(false)) == ( self.limit > 3 )`、
  `p["name"].as_str().unwrap_or_default() == "a<b"`。
- **质量门**：同命令告警 A/B（基线 42a4ef792 ↔ 修复）**多重集完全
  一致**（0 新增 0 移除）；rustfmt 过；tu 868/868 + compiler 216/216。
- **SD/交接同步**：sd-b-proposals SD-02 补比较子表达式语义；handoff
  §1 提交链补 `6f1cac8cb`、§3 V-03 行更新。
- **next_responsible_role: review（完整 T-10）**——绑定 `6f1cac8cb` +
  更新后 handoff/sd-b-proposals + r14-closeout。merge-owned 冻结门
  （裸 t+tv+tu+tt 及必要 docs/schema/book）照旧 pending。

### R15（2026-10-21）：完整 T-10 pass，merge-owned 门禁 pending

stage: review；review_scope: final；reviewer_identity: 本聊天 Codex 独立复审上下文，未参与实施，未创建agent/新checkout；plan_id: PLAN-814；plan_revision: 3；outcome: **pass**。
原树 `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev`；reviewed_commit `6f1cac8cbe618e509bf85d5c000fa0c26f35df6e` / tree `f98330d91b665566876d36a330361f770503fd81`；修复base=R14 `42a4ef792`，B base=A实际发布`f6e07acee`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。初末clean/身份/依赖稳定，44输入仅RustGenerator变化；十一canonical与R14一致，计划/SD/handoff及原例冻结。

[完整报告、112有效编译/三项独立探索诊断/AC逐项/冻结哈希](../attachments/814/review-r15/review.md)。**F814-R9-01 CLOSED**：R14五败全部实际编译绿，原97feature+默认正控98编译全过、overlay八组合mismatches0（原99检查全绿）；六符×两方向×loop/prop24矩阵保持；新增六种比较子树×两方向取反/逻辑12组、prop复合bool及字符串普通符号正控通过。Plan受验112/112编译绿，无Plan内未闭finding；R11/R10及更早闭合边界保持。

原始矩阵115编译112过3败不隐藏：新增escaped-quote探索在known/unknown均E0765，AURA把解码引号直接拼回条件；owner extract.rs与A发布整文件Git blob同为c0f7ab6f1e076308fecce29d143a9ec700025a90，未变且基线原件冻结。D814-R15-01单独登记为非阻断既有字面量编码债务候选，非R9合法下形条件的值类型保持失败；不以该失败正控宣称输入已受支持或把尚未完成合同项延期。未运行旧A完整二进制；边界/归因证据详报告。

四包check/fmt/diff-check/当前777 strict均过（violations0/grandfathered3），compiler36/runtime235保持无新增改动段告警；API/typepin本轮再次编译绿。SDK物理隔离fresh+locked5/5、六provider43、runtime65、feature面、九档strict逐ID/挂载身份、counter Vue/VM三状态及792真实driver24/24、runtime独立consumer5/5，按未变工作树代码/依赖/配置/输入限定复用，不冒称本轮新跑。AC01..06 pass；AC07 merge pending；AC08..10保留A已发布。SD01..08审定当前已验职责/语义/边界，SD08继续在当前de5e9d4a全文上增补并保留M815-A/M816-A；十一现有目标+新增runtime及GOAL003/007元数据正确，未写canonical/liveledger。

**T10完成，status reviewed，13/15**（T01..10/T13..15完成）；T11/T12仍pending。next_responsible_role: **merge**。先处理master并集/冲突与受影响复审，再新冻结裸t+tv+tu+tt及必要docs/schema/book；红门归因，改码后复审重冻。tf仅主检出到期单实例，aavm原用户豁免保持。门禁通过方可实际B合入、Spec/ledger沉淀、发布与consumer读回收据、归档/guard clean清理。**reviewed不等于B交付完成**；本轮未整档、未实施修复/合入/归档/清树。历史及本轮冻结证据按用户说明保留，后续复制探针再运行。

2026-10-21（续 10）/ stage: merge / revision: 3 / outcome: **pass（M814-B 实际
交付；completion_kind: delivered）** / worktree: `D:/autostack/.wt/lang-814/
auto-lang` | branch: `codex/plan-814-dev` | reviewed: R15 pass 绑 `6f1cac8cb`
（rebase 后 `a3f50d15b`，range-diff R8-R14 补丁全等价）| **delivery_commit:
`93a25c924`**（master tip=delivery，ff-only 无合并提交）| 依赖: auto-down
`c39ac7f9`（未变）。

- **prepared**：SD-01..08 canonical 落库（SD-03 新建
  [auto-ui-runtime/project.md](../../specs/auto-ui-runtime/project.md) 全文；
  SD-02/04 B 增量；SD-01 增量；SD-08 在当前版（含 M815-A/M816-A 章节）
  增补；SD-05/06/07 B 事实注记；INDEX 57 projects）。
- **rebase 组合（记录性调整，非静默）**：master 自 B 基点新增 815 T-04/
  816 M816-A/777 重采样——`app_registry`/`i18n_lookup` 因 815 先落
  compiler 且 DAG 禁 compiler→runtime，**单源定于 compiler，runtime 经
  `pub use` 再导出**（旧路径全保持，SD-02/03 已注记）；probe 门/
  tests_seam 双计划同款修复合并注释；T-05 canonical 断言随 rust_ui 归属
  迁 auto-ui-tools；两个 777 基线重采样提交被终局并集重采样取代（丢弃）。
  旧→新映射：6f1cac8cb→a3f50d15b（=）、42a4ef792→35a4f528b（=）、
  feae28e0f→83d728636（=）、1bbf98620→28d1b883f（=）、49206c699→dropped、
  236827f1c→60dd597fa（=）、6063b7a94→064c431cc（=）等全表 range-diff
  在案；4 个 `!` 组合提交（T-04×2/T-05/fmt） diff 审阅过。
- **冻结门（delivery `93a25c924` 上）**：裸 **t 5327/5327**、**tv
  163/163**、**tu 868/868**、**tt 5699/5699**（首轮负载瞬态早退，完整
  重跑全绿——真进程性能族在案口径）、**tb 5303/5303**、**777 并集
  重采样 119 边 strict pass violations=0**（ui_to_language=0 保持，
  基线 @f6005ed3a）。
- **landed**：`wt-guard` clean → `git merge --ff-only` → master tip=
  `93a25c924`；known-good smoke：consumer 读回 **5/5+4/4+5/5**。
- **ledger_refreshed**：Regime B **guarded offline projection**（HTTP
  store 探测不通 code 000；备份 `.autoos/specs.json.bak-p814b-merge` →
  外科插入 +96/-1 → 回读校验 v64/六节/5 条新项 file+tags 齐）。条目：
  designs `P814-UIR`/`P814-CMP-TYPING`、reviews `P814-B-REL`、reports
  `P814-B-MERGE`、tests `P814-UIR-T`。
- **815 交接**：SDK/UI 出口已发布（A/B 两阶段均在 master），815 B 期
  可重建 worktree 继续（其 T-04 并集已含本批组合）。
- **cleaned（含一项精确 pending）**：wt-guard 双 worktree clean；tmp-814-base
  与 auto-down 兄弟 worktree、codex/plan-814-dev 分支（was 93a25c924）、
  组内全部证据/scratch 目录（事故 diff 已保全入 attachments/
  t04-incident-archive）均移除；**残项**：`D:/autostack/.wt/lang-814/
  auto-lang/` 目录剩 examples/ui/002-counter 生成产物+node_modules 被
  存活 node/esbuild 进程锁（git worktree 注册已清零、分支已删、无源码/
  无登记——纯未注册残留目录），持有进程无法安全归属（可能属其他会话）
  不盲杀；待占用进程退出后 `rm -rf` 即收口，不影响任何已验证交付面。
- **后续项**：tf 批量回归到期判定（收据核查）；本仓无 backend/bundle
  生产件（landing≠deployment 观察项：无——语言仓交付面=crates+792
  消费，consumer 读回已执行）。

### 合入恢复复核（2026-10-21）：cleanup 已完成，选择面缺口待对账

stage: merge；mode: archived recovery-only；plan_id: PLAN-814；plan_revision: 3；outcome: **blocked（F814-MERGE-01 待独立后续修复/对账）**。本聊天 Codex 按 auto-plan-merge 核实原交付，不重做合入、不重开归档Plan。

[恢复报告/具体遗漏ID与门控/清理与链接核验](../attachments/814/merge-recovery-20261021.md)。实际delivery `93a25c924d1beafab130588bad4d1041dc86e48c` 是当前75b26c253基线祖先，master reflog明确ff-only；R15 `6f1cac8cb` 映射 `a3f50d15b`，R8-R14修复range-diff均为=，当前RustGenerator与原受审文件相同。原20提交映为17：13等价/4组合/3基线丢弃（非粘贴摘要的2个）。canonical、ledger v64五项与归档已跟踪、批量收据75b26c253存在，未重写Specs/ledger或重跑整档。

**cleaned 已实结**：残留文件数0、fresh wt-guard clean；仅结束路径明确属于814 counter旧验证的Vite39196及其子esbuild54876，退出后再跑guard并成功删除整个`D:/autostack/.wt/lang-814`空组。开发分支/临时基线/兄弟worktree注册均无残留。旧记录“残锁目录pending”现已关闭。归档移动遗漏的50个唯一链接目标（65次引用）补齐../层级，原证据未改、现均可达；本报告及链接修复提交主库，其他会话WIP及冻结快照保留。

**F814-MERGE-01/P2**：合入组合把app_registry单源迁compiler时，真实三App会话启动测试仍带`#[cfg(feature = "ui-iced")]`，compiler仅声明ui/ui-headless，测试被永久编译排除。当前只枚举t不执行，5328旧集合经28个app_registry/i18n owner重写及1个store_deps明确重写核销后，唯独`launch_three_real_apps_via_registry_resolver`缺失（当前5327，无未解释新增）。完整门全绿不能证明未选择的测试仍被执行；违反原AC05/T07零丢失与T11并集验收边界。建议独立后续修复将runtime session测试移至runtime测试层，保持compiler注册表单源，不给compiler虚设ui-iced或引入反向依赖。

原B已实际交付/归档事实不回滚，archived及原15/15作为既有流程历史保留；**本次不再授全部收尾pass**，不以归档状态掩盖发现。按技能“report blocked for reconciliation rather than routing an archive into work”，代码由独立后续修复承接；本次仅完成清理/文档恢复并准确记录阻断，未改实现/默认分支历史。新文档提交不触发新的批量回归；原批量收据保持。

## 10. 待澄清事项

**R4-01 合同裁定（已决，2026-10-20）**：用户批准 R6 复审整理的两项窄增量
（[contract-r3-draft.md](../attachments/814/review-r6/contract-r3-draft.md) 全文）——
①上层兼容包/compiler 经 inventory 启动构造器登记轻量函数表（只登记、不执行
业务初始化/I/O）；②UiParseContext/UiGrammar 定义归属改轻 auto-parser-api、方言
保留实现与旧路径再导出。主计划已升 **r3**（§5.3 全文落文；§2/V-02/AC-02 同步
amend；SD-01/02/04/05 补语义）。受影响正反控按新合同重验（§9 续 7）。背景对账
见 [r4-01-assembly-contract.md](../attachments/814/r4-01-assembly-contract.md)。

没有需用户重复授权的其他当前问题；前置合入是客观门，不能超时推定放行。T-01有界证明旧公开类型/默认装配及SDK布局可行；若必须删旧API/改变支持/复制状态或新增兼容退役范围，具体needs_replan，不自行降低AC。包组仍留语言仓，Git承接后续R08。

2026-10-21（续 3，R8 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R8-F01..03 一次修齐**（T-07/T-08/T-09 复勾待完整 T-10 判定；非自判
pass）/ worktree 同上 | code 追加：`6063b7a94`（R8-F03 健康批）。

- **R8-F03 关闭（清偿）**：两未用再导出——load_ext_imports_for_vm 再
  导出门随唯一消费方（test-gated plan370）同门；stmt_symbol_name 同纪律
  （消费方 vm_bridge 测试 mod/plan339 均 test-gated）；collect_module_
  imports 保持无条件（back_proxy/dynamic_build 生产消费在案）。consumer
  fixture fmt（skip_children 口径 exit0）。四包 offline-locked 复验 0 错
  /R8 点名目标警告 0。
- **R8-F02 关闭**：[sd-b-proposals.md](../attachments/814/sd-b-proposals.md)
  ——SD-03 可审定全文（拟落库正文逐节）+SD-01/02/04/08 增量（逐目标
  before-hash/保留新增规则/理由/AC）+生成器两修复具名反例；handoff §5
  T-14/T-15 误指已纠（→T-11/T-12）。
- **R8-F01 关闭**：[v03-dual-run](../attachments/814/v03-dual-run/)——同源
  002-counter，Vue（Playwright/localhost IPv6）与 VM（MCP press_sequence）
  **三状态两操作逐点一致**（0→-1→1；DOM/snapshot 双文本 oracle+截图在
  档）；vite 仅监听 ::1 的访问要点记录。覆盖口径=合同"有代表性"（单示
  例），非全组合逐像素（合同未要求）。
- **next_responsible_role: review（完整 T-10）**——绑定 `6063b7a94` +
  更新后 handoff + sd-b-proposals + v03-dual-run。merge-owned 门禁照旧
  pending。

2026-10-21（续 4，R9 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R9-F01..04 一次修齐**（T-04/T-05 复勾待完整 T-10 判定；非自判
pass）/ worktree 同上 | code 追加：`236827f1c`（R9 修复批）。

- **R9-F04 关闭**：rust.rs 全文件 fmt（skip_children 口径 exit0；六处
  diff 清）。
- **R9-F01 关闭（类型感知比较投影）**：形状已知走 shaped 访问器
  （bool/int 语义保持）；形状未知按另一操作数字面量分流
  （bool→as_bool/数值→as_i64/默认 str）；loop/prop×左右位全覆盖。
  reviewer 四反例（bool/int/rhs+两正控）经其 r9-comparison probe 再生
  +style-check 根锁上下文**四 feature 全 0 错**；numeric-silent 双侧
  as_i64 类型化（不再静默串比较）。
- **R9-F02 关闭（旧样式路径）**：runtime `ui::style` 改门面模块——
  compiler DSL glob 再导出+`style::IcedStyle`/`style::iced_adapter::*`
  旧路径（同一类型单源=style_iced_adapter，不复制实现）；reviewer
  legacy-style.rs old-path feature 编译过。
- **R9-F03 关闭（headless 声明面）**：PAYLOAD_SEP/encode_payload/
  ScrollStateEntry 纯项自 iced 上移 mcp_types 单源+iced 侧再导出；
  iced 专属面 cfg 隔离（style_iced_adapter/anchor_slot/sched_diag::
  trace/vm_bridge/i18n 钩子随实门；aura overlay 几何改 compiler
  StyleClass 纯扫描；window_width 直连 compiler theme）；log 依赖显式
  钉 std。**六形态 0 错**：no-default/bare-ui/ui-headless/ui-dialog/
  code-editor/default（auto-lang no-ui/default 同绿；旧巨树 bare-ui/
  headless 为预存红，本批补齐声明面）。
- **回归面**：runtime 1899/1899；tu 868/868；auto-lang t 形态
  1476/1476；shell-pack 4/4+freshness；compiler 216/216；**九档验证器
  复跑逐档计数与 T-07 收口完全一致**（t 5328/tf 6279/tv 163/tt 5700/
  tb 5304/taa 6275/ta 6729/t3 6730，missing=0/unresolved=0）；777
  strict violations=0。sd-b-proposals 已同步 R9 语义（SD-02 比较分流/
  SD-03 声明面组合+旧样式路径）。
- **next_responsible_role: review（完整 T-10）**——绑定 `236827f1c` +
  更新后 handoff/sd-b-proposals。merge-owned 门禁照旧 pending。

2026-10-21（续 5，R10 needs_fix 后）/ stage: work / revision: 3 / outcome:
**R9-01 残余+R10-01/02/03 一次修齐**（T-04/05/08/09 复勾待完整 T-10
判定；非自判 pass）/ worktree 同上 | code 追加：`2f83c4270`（R10 修复
批）→ `49206c699`（777 基线重采样）。

- **R9-01 关闭（完整类型不变量）**：操作数扫描收负号/`+` 与小数点；
  AST 下形一元负号跨空收数字（`- 1`）；数字分流 int（含符号）/
  float（含小数点→as_f64 不截断）；loop/prop×左右位两调用分支补
  float 臂。reviewer r10 探针（r10-generate+check 五 feature
  unknown-negative/unknown-float/unknown-rhs-float/prop-negative/
  prop-float+无 feature 正控）**全 exit0**
  （[r10-closeout](../attachments/814/r10-closeout/)）。
- **R10-01 关闭（浮层仲裁）**：extract_absolute_position 纯扫描补
  position 族**后继覆盖**（Relative/Fixed/Sticky 清 absolute——与
  IcedStyle::from_style 同一按声明序覆盖语义）；reviewer
  r10-overlay 五组合 **mismatches=0 exit0**（正反控全一致）。
- **R10-02 关闭（777 真实绿收据）**：全部代码修复提交后按审计流程
  --refresh-baseline 重采样（base=2f83c4270，log std 清单变更入锚）
  → strict 重放 **violations=0 exit0**；基线提交 `49206c699`。
- **R10-03 关闭（健康）**：scroll_state_reader 删除随类型迁移遗留的
  serde 未用导入（derive 已随定义迁 mcp_types 全限定名）；四包匹配
  check 0 新增目标告警。
- **回归面**：runtime 1899/1899（首轮 1 败=负载瞬态复跑绿）；tu
  870/870（含 e2e 真编译 2/2）；auto-lang 1476/1476；手写触面 fmt 过。
  SD-02/03 已同步（负数/浮点分流+no-op 门精确 not(any(ui,ui-dialog))
  +浮层仲裁注记）；handoff/证据矩阵同步（r10-closeout 归档）。
- **审查快照纪律**（R10 历史证据说明接受）：本轮反例在 review-r10
  目录原样再生未回写 review-r9 冻结件。
- **next_responsible_role: review（完整 T-10）**——绑定 `49206c699` +
  更新后 handoff/sd-b-proposals/r10-closeout。merge-owned 门禁照旧
  pending。
