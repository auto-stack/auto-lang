---
plan_id: PLAN-814
status: executing
feature_name: 语言 SDK、UI 编译与运行包组整体分层
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-20
plan_revision: 2
current_step: 1
total_steps: 15
supersedes_spec_components: [docs/specs/auto-lang/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-ui-dialect/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/language-sdk/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [language-sdk, auto-ui-compiler, auto-ui-runtime, auto-lang/compat, auto-vm]
depends_on: [PLAN-807, PLAN-808, PLAN-810, PLAN-811]
---

# [PLAN-814] 语言 SDK、UI 编译与运行包组整体分层

## 0. 变更摘要

完成R04的语言/编译/运行边界：提供不依赖UI实现或Man的语言SDK，把UI编译生成与UI运行/renderer形成独立可消费包组；默认组合和旧`auto_lang::*`兼容明确归上层。迁移实现、公开路径、资源、测试选择、真正消费者及发行闭包一起交付，不止增加几个空crate。

不搬Man的UI工程工具/资源（815），不接完整CLI工具协议（816），不搬Git仓/切旧默认分支，不在本期删除UI兼容AST/TokenKind（R10）。纯SDK不等于原名auto-lang同一包既重导出完整UI又保持纯Core。

本次r2按M814-A真实阶段发布、M814-B完整交付组织，不增加Plan编号、不降低原最终AC。执行与审查修复并行规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)；阶段pass不等于整Plan交付。

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

用户2026-10-20明确要求修订现有814/815/816，允许阶段交付及接口合入后放行下游、并行审查验证修复，不增加编号。原目标/最终AC/支持面不变；本轮只改主检出规划文档，不改实现树、canonical Specs或远端。 来源与当前hash见[本轮输入](attachments/sdk-tools-r2-20261020/inputs.json)。

授权：用户要求建立下一批较大计划并明确合入前置；本轮仅auto-lang仓内规划，不实施/创建agent或工作树、不写相邻仓/远端。来源：[总路线](reports/v06-extraction-execution-roadmap.md)、[批次调度](reports/sdk-tools-parallel-order-814-816.md)、[输入hash](attachments/sdk-tools-batch-20261019/inputs.json)。

权威输入：[语言现状](../specs/auto-lang/project.md)、[frontend](../specs/auto-lang/frontend/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[UI](../specs/auto-lang/ui/overview.md)、[方言](../specs/auto-ui-dialect/project.md)、[交付](../specs/repository-boundaries/generated-ui-distribution.md)。807/808/810/811均已归档交付；810=f0ec8a91f，811=d06cdc6f6，实际master祖先已核实。旧r1等待状态保存在Git历史；本轮不重做这些Plan验收。

实际写面：`crates/auto-lang/{Cargo.toml,build.rs,src/lib.rs}`（build输入以T-01存在性/引用核实）、`src/{parser,parser_bridge,compile,config,a2r_std,back_proxy}.rs`、`src/{ui,ui_gen,aura,a2ui,design_tokens,vm,trans}/`、`crates/auto-vm`、根Cargo与`.cargo/config.toml`/`.config/nextest*.toml`。当前默认带ui-iced，`Parser`有pub scope/cur/infer_ctx等，lib全局STORE_EXTRA_FILES/USE_DIAGS及UI辅助也混在旧根；无环拆分必须处理身份/装配/生命周期，不能仅关default-features伪装。

### r2阶段启动门

- 807/808/810/811均已实际交付，T-01整链probe及T-02/T-03可在唯一树开始；不再等待810/811。
- **M814-A**：T-01→T-02→T-03→T-13→T-14→T-15；真实SDK与UI编译公开面发布后，放行815生成主体与816语言adapter。
- **M814-B**：T-15后继续T-04..12，完成runtime/renderer与全部原AC；完整runtime调用必须等本阶段实际发布。
- 依赖未合入的实现不能进入生产闭包，不借上游WIP；任务ID保留历史编号，不表示数字顺序执行。

## 5. 详细设计

1. **SDK与UI owner图**：逐模块列出源码、public项、normal/build/dev、feature、生成字符串/宏/include/build input依赖；解析/推断/编译/VM/非UI trans留同份Core，UI生成/运行移包；依赖语法名不等于UI实现依赖。现有UI兼容AST可在轻auto-ast暂留，明确R10退出，不把UI私有TypeStore/registry推回Core。
2. **默认组合与旧API**：旧auto-lang可为上层兼容包；SDK新名字公开且实测独立。对`Parser::with_session`、AutoVM/NativeInterface构造、外部Dialect、native callback、pub字段/trait/inherent与模块路径构造编译pin；旧UI场景仍默认可用，SDK的Core场景不加载UI实现，SDK UI解析须显式获得UI方言组合。不能靠linker inventory偶然拉入初始化或Core硬编码UI factory。若无法兼容，T-01具体提出重规划，不自行砍API。
3. **UI compiler**：现有ui_gen四生成器/registry、AURA/a2ui与UI相关分析按闭合图迁移；通用trans/shared helper经SDK公开最小能力，不暴露Core全部私有结构。生成canonical runtime/SDK import由一份路径策略产生；旧路径门面保兼容。允许必要import/header基线变更须显式映射新旧路径，不能以更新golden掩盖行为差异。
4. **UI runtime**：ui目录runtime、renderer、editor/terminal/widget、desktop/MCP/宿主adapters及其状态/资源迁有归属的包；VM/SDK只保轻能力与非UI机制。i18n lookup/装载、a2r frame直连、config/UI依赖和其他反向边逐项验清；未消除边必须有本Plan内任务，不能在最终声明“纯Core”时留豁免实现。
5. **状态/并发**：registry/OnceLock/thread_local/global runtime/VM Rc、provider默认/实例覆盖/owner history、模板/语法session/cache不可跨新包产生第二真源；线程亲和、锁外回调、store额外文件drain、use诊断、终端/媒体生命周期/快照共享保持。移动字节不等于状态单源证据。
6. **消费者与测试**：旧路径消费者在兼容层继续可编译；至少一个真实非UI消费者（首选auto-vm，若其实际依赖需调整则T-01具名替代）直接切SDK。UI compiler/runtime及后续工具必需的语言adapter（如auto-gen可执行模板层）也要沿实际闭包切SDK，不能经旧auto-lang兼容包把UI实现重新拉回。auto-lsp等UI补全可暂用上层具名adapter，不谎称其整体纯Core。迁出的lib/test/integration/doc/schema测试保单份来源；cargo t/tv/tu/tt/tb实际选择面映射覆盖新包，UI tu保持合适单进程registry复用，不因包名变化丢测试。
7. **发行**：777精确域边/forbidden、792受管source与manifest/资源/producer/lock/重定位更新至真实包图；新SDK独立fresh/locked无UI checkout，UI实例工程可用新包，不靠父仓隐式path。SDK不含UI实现的证明覆盖normal/build/dev，UI部署profile仍保原功能而非删除默认重后端。

### 5.1 A/B阶段的真实交付边界

**M814-A**不是空crate或接口草图：纯SDK完成Core parse→compile→VM与已有非UI消费者、必要trans出口；normal/build/dev无UI文法实现/编译/运行包或Man。T-02已经清除阻碍此性质的i18n/frame/config等边，不能把纯SDK所必需的去边留到T-06。UI compiler迁实际生成算法/资源与公开入口，真实消费者可生成其声明支持的产物。A中迁出的测试族和资源立即接入实际门/777/792及fresh/locked消费，不留到T-07/08才补。

旧auto-lang上层兼容可继续使用尚未搬迁的现有真实runtime，保原默认UI、Parser pub字段/方法/回调及单份状态；新SDK与compiler不可反依赖该兼容包。生成runtime import仍指向当前真实可用、版本受约束的实现，B再有控制地切换；不得将未完成的新runtime能力列为已支持。原四生成器各自支持面保持，依赖runtime的原完整正控保留给B，A至少验证生成产物和当期真实runtime消费路径。若依赖图不能这样切，T-01交具体needs_replan，不用伪实现凑阶段。

**M814-B**完成新runtime/renderer/宿主adapter、默认组合和全部原验收。A的局部合入不提前勾AC-01..05整体完成；仅新增AC-08..10描述阶段范围。SD-01/02与SD-04..08仅按A已发布事实沉淀，SD-03完整runtime规则等B，不把B目标写成现状。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/project.md | 原巨型包→纯SDK职责/API/feature/显式组合与正常闭包 | 基础可独立消费 | AC-01, AC-02 |
| SD-02 | add | docs/specs/auto-ui-compiler/project.md | 原UI分析/生成→独立compiler包、公开接口/资源 | UI编译owner | AC-03 |
| SD-03 | add | docs/specs/auto-ui-runtime/project.md | 原UI运行/renderer→独立runtime与宿主/状态归属 | 运行边界 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-lang/project.md | old包标为上层兼容/组合，实际SDK位置明确 | 避免纯Core虚报 | AC-01, AC-02 |
| SD-05 | modify | docs/specs/auto-lang/frontend/overview.md | Parser默认/显式UI组合与旧API承载 | 解析身份/装配 | AC-02 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 无UI实现的VM能力/默认组合与余边清零 | 真正解耦 | AC-01, AC-04 |
| SD-07 | modify | docs/specs/auto-lang/ui/overview.md | UI包组/旧路径/资源/消费者事实 | 完整交接 | AC-03..05 |
| SD-08 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 真实新包图/测试与资源输入进入发行profile | 迁包可分发 | AC-05 |

## 6. 测试设计

T-01冻结旧public/API与消费者/支持面、测试族选择清单；T-08建立新增`test/fixtures/modularity/language-sdk-consumer/`、`ui-package-consumer/`及上层`plan814_package_boundaries`定向测试。以下均owner_stage=work，当前未执行。

| Case | AC / invariant | 真实入口/设置 | 区分性oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 纯SDK/单源 | 独立manifest→Core parse/compile/VM/trans；无UI checkout、真实auto-vm消费 | normal/build/dev无UI compiler/runtime/grammar实现/Man；单份AST/Value/VM/源码，不用空包或重导出旧巨包骗绿 |
| V-02 | AC-02 兼容/装配 | 旧Parser pub字段/方法/ExternalDialect/native callback、UI默认/import、Core及显式UI组合，features开关/两VM覆盖 | 旧签名consumer编译与AST/执行结果保持；SDK无UI能构建，旧UI默认可用且不靠inventory偶然性；无双Parser/VM可变状态 |
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

A新增review任务只给阶段pass；原最终review任务仍验全部AC/SD。阶段和最终的裸t/触面整档均归merge冻结后统一排队，资源独占项串行；tf仅主检出到期单实例。接口修复由owner发布，影响方重放受影响scope，未变且适用证据可复用。具体失效/收据/单写者规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md) §2–5。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 真SDK可独立消费且normal/build/dev无UI实现/Man，真实非UI消费者落地、语言/VM单源；V-01 |
| AC-02 | work | 旧公开API/字段/trait与默认UI保持，纯SDK显式组合可用，无隐式链接装配/双状态；V-02 |
| AC-03 | work | UI编译/生成/运行职责完整迁移、四生成器与真VM/Iced/生成工程消费保持；V-03 |
| AC-04 | work | i18n/frame等余边清偿，provider/全局/TLS/资源与退出保持；V-04 |
| AC-05 | work | 测试族迁移选择面、777/792与fresh/locked/资源负控通过，SD/来源/剩余兼容与净迁移清单完整；V-05 |
| AC-06 | review | 同树独立审查绑定修订，遗漏/规范/警告/格式/debug检查通过，merge门pending明确 |
| AC-07 | merge | 冻结门、canonical Specs/ledger/归档/接口与资源交接、guard clean后清理有实际收据 |
| AC-08 | work | M814-A满足§5.1全部阶段性质，V-06及V-01/02/03/05当期范围通过；非B整体pass |
| AC-09 | review | T-14独立阶段审查绑定r2/M814-A冻结SHA/API与Spec子集，通过且merge门pending |
| AC-10 | merge | T-15冻结门通过、实际master祖先交付与Spec/ledger增量、版本/来源/consumer读回收据；不归档 |

## 8. 执行步骤

15项；保留原任务ID，追加A阶段任务，**按依赖列执行，不按编号排序**。整项depends_on为摘要，阶段启动以§4/5和本表为准；不宣称Runner已支持阶段调度。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付、验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 现在；807/808/810/811已交付 | 新attachments/814/contract-baseline.md：所有者DAG/旧API构造/状态/测试族/资源/真实消费者清单与候选布局；前置合入后整链编译probe冻结SDK/默认UI兼容方案，不可行先重规划。[✅ 已完成 2026-10-20：基线 14 节 + probe 记录 §15；切片 `be10024ab`（唯一树，基面 `203e2ce91`）冻结包名 `auto-lang-core`/`auto-ui-compiler`/`auto-ui-runtime`+DAG+再导出机制+别名多 -p 机制；全部消费者编译/日常档 5303 绿/777 pass；无 needs_replan] | AC-01..05 |
| [ ] | T-02 | work | T-01可行性决策 | 真实纯SDK/上层兼容、单份Core/状态与公开出口；立即清除阻碍纯SDK的全部反向边，保旧默认API；迁出测试/资源/非UIconsumer及V-01/02当期证据 | AC-01,02 [✅ 已完成 2026-10-20 R3 会话：主迁移 `edb78b2ef`（soft-reset 吸收 WIP）+ 残项收口 `43366c989`——auto-vm 切 SDK（V-01 真实非 UI 消费者，闭包零旧包/UI，冒烟执行过）+ a2r frame 经 PLAN-779 FrameProvider 进程默认端口单源（plan716 帧探针全过）+ F814-R1-02 整改 `ac2882a0d`（SDK auto-ui-dialect 改非默认 `ui-dialect` 显式组合 feature；上层 default 转发保旧行为；normal/build/dev 三边闭包验证无 UI 文法；86 UI 文法测试按 feature 门控选择面不变）+ 迁移死门清偿（i18n 路由/diff code-editor 转发/4 组滞留 UI 测试上移 plan656/485/712/413）；SDK off 1627/1627、on 1713/1713；777 strict pass ×2] | AC-01,02 |
| [ ] | T-03 | work | T-02 | 真实UI compiler、aura/a2ui/design_tokens/registry与生成路径迁移；A当期真实runtime产物消费、无环公开面、资源/测试选择与V-03/05当期证据 | AC-03 [✅ 已完成 2026-10-20 R3（B1 `b907a89eb` + B2 `59b9c7673` + 消费批 `b02a10772` + SD-08 `abe9c9900`）：style/action_config/lucide/aura/a2ui/ui_gen 四生成器/registry/bp/route ≈11.3 万行迁 auto-ui-compiler 单源+旧路径全再导出；ui-package-consumer 独立 fixture 真实生成 4/4；语料资产随 owner 迁；指纹机制迁 compiler build.rs；交付合同更新至新包图（792 driver 25/25）；SD-02 事实底稿=m814-a-candidate §2（正文随 T-14/T-15 受管沉淀）] | AC-03 |
| [ ] | T-04 | work | T-15/M814-A已发布 | UI runtime/renderer/宿主adapter与资源迁移，VM依轻能力；保持真实编辑器/终端/媒体与跨端 | AC-03,04 |
| [ ] | T-05 | work | T-04；补齐B默认组合，A兼容不得先破坏 | 默认Parser/VM/新SDK显式UI装配、旧auto-lang字段/方法/feature/旧路径兼容；真实import及外部consumer V-02 | AC-02 |
| [ ] | T-06 | work | T-05；A纯SDK禁止边已在T-02清除 | i18n/a2r frame/config等残边、全局/TLS/registry单源和退出验证；必要非UI消费者切SDK，V-01/04 | AC-01,04 |
| [ ] | T-07 | work | T-06 | 测试源码/别名/nextest选择面按owner迁移，全部旧族逐项映射，移除旧实现；V-05选择清单 | AC-05 |
| [ ] | T-08 | work | T-07 | 新consumer/上层定向矩阵V-01..05、777/792新闭包、fresh/locked/重定位与负控 | AC-01..05 |
| [ ] | T-09 | work | T-08 | SD-01..08备稿、真实API/包源/资源/测试归属及compat退出清单，新增attachments/814/handoff.md绑定SHA/结果，提交送审 | AC-05 |
| [ ] | T-10 | review | T-09 | 同树独立完整DAG/公开兼容/生产/寿命/测试选择与SD审查，merge门pending | AC-06 |
| [ ] | T-11 | merge | T-10 pass | master并集/变化复审，最终冻结提交统一§6门，失败修复后重审重冻 | AC-07 |
| [ ] | T-12 | merge | T-11 | 规范/ledger沉淀、归档、已合入SDK/UI出口交815，guard clean后清理、终态计数 | AC-07 |
| [ ] | T-13 | work | T-02/T-03 | 冻结A API/版本/源码与支持清单；V-06及A涉及V-01..05、777/792/独立fresh/locked/负控完成，新增attachments/814/m814-a-candidate.md，准备实际Spec子集 | AC-08 [✅ 已完成 2026-10-20：候选冻结 @ `abe9c9900`（[m814-a-candidate.md](attachments/814/m814-a-candidate.md)——§2 A 事实清单/§3 证据矩阵/§5 诚实限制）；消费者 fixture 双绿（language-sdk-consumer 5/5 + ui-package-consumer 4/4，`b02a10772`）；777 strict pass（域登记+窄豁免，基线 `d91948ee1`）；fresh+`--locked` 消费跑通；**792 driver 25/25（EXIT=0）**——首轮受管红暴露新包图三接缝（投影 test-cases 新形态/ui_gen 源根/auto-ui-runtime 未 vendored），SD-08 修复于 `abe9c9900`（DELIVERY_CRATES 19、投影分 spec、指纹路径）；auto-man delivery 单测 9/9。**AC-08 判定归 T-14 独立审查**（本勾选=work 交付完成，非审查 pass）] | AC-08 |
| [ ] | T-14 | review | T-13提交冻结 | 独立阶段AC-08/API/类型状态/资源/Spec审查，绑定r2/M814-A SHA；全Plan未reviewed、merge门pending | AC-09 |
| [ ] | T-15 | merge | T-14 pass | 串行冻结门/实际master合入/Spec子集与受管ledger增量、版本来源consumer读回发布收据；保原树/分支继续B，不归档 | AC-10 |

唯一树`D:/autostack/.wt/lang-814/auto-lang`、分支`codex/plan-814-dev`；work/review顺序交接同树，先核实实际tree/base并读取主检出r2。A发布后保留原树/分支同步master继续B，不创建第二审查树、不归档/清理。A期间Plan保持executing，只有B全部原AC完成后才依标准流程execution_done→reviewed→archived。

阶段发布是用户本轮明确授权的有界例外：按共用合同独立阶段review、冻结门、人工Git合入及非归档Spec/ledger受管增量与收据组织；**不调用会自动归档整Plan的完整merge流程冒充phase支持**。B按原最终review/merge/归档任务完成。无WIP依赖，无junction/symlink，最终清理必须guard clean。跨Plan审查/修复与共享窗口见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)。

## 9. 复审记录

r1以下记录为历史合同预检，旧启动门由r2替代；不是当前调度指令。

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实现/独立验收pass）/ next: work T-01只读；代码probe和T-02起等待810/811实际合入 / changed: T-01..12、AC-01..07、SD-01..08。未执行未来Cargo/fixture、未创建工作树/agent或搬仓。

2026-10-20 / stage: new / revision: 2 / outcome: pass（文档合同预检，非实现/独立验收pass）/ next: 按§4阶段门work；814先确认已登记唯一树并读取本r2。changed: 阶段A/B与新增T/AC/V、Spec分阶段沉淀、并行审查修复；原目标、原T/AC/SD ID保留。r1证据仅历史，不自动覆盖新范围。未执行实现/未来fixture/门禁。见[预检](attachments/sdk-tools-r2-20261020/preflight.md)。

2026-10-20 / stage: work / revision: 2 / outcome: T-01 完成（M814-A 第一步；work 继续 T-02，非阶段/整Plan pass）/
worktree: `D:/autostack/.wt/lang-814/auto-lang` | branch: `codex/plan-814-dev` | base:
master `203e2ce91`（811 收口；唯一树与 r2 §8 登记一致）| code: `be10024ab`（T-01 probe
切片）| 依赖: 组内 auto-down 兄弟 detached@c39ac7f（autodown-core 解析，零源改动）。
交付 [attachments/814/contract-baseline.md](attachments/814/contract-baseline.md)（§1-14 盘点
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
详见[报告/AC逐项与证据边界](attachments/814/review-r1/review.md)、
[输入hash](attachments/814/review-r1/input-hashes.txt)、
[冻结SD](attachments/814/review-r1/sd-frozen.md)。实施树clean，但HEAD与§9自述均为不可编译、
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
[R2报告/闭合边界/原始check](attachments/814/review-r2/review.md)：独立locked/offline
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
[R3报告/逐项反例与正控/冻结SD](attachments/814/review-r3/review.md)：独立SDK消费者5/5、
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

## 10. 待澄清事项

没有需用户重复授权的当前问题；前置合入是客观门，不能超时推定放行。T-01有界证明旧公开类型/默认装配及SDK布局可行；若必须删旧API/改变支持/复制状态或新增兼容退役范围，具体needs_replan，不自行降低AC。包组仍留语言仓，Git承接后续R08。
