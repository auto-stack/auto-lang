---
plan_id: PLAN-841
status: executing
feature_name: UI领域模型、AST与Token及lowering终态解耦
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-26
plan_revision: 1
current_step: 2
total_steps: 12
supersedes_spec_components: [docs/specs/auto-ast/project.md, docs/specs/auto-lex/project.md, docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md, docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/ui-domain-model/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ast, auto-lex, auto-parser-api, language-sdk, auto-ui-dialect, auto-ui-compiler, auto-ui-runtime]
depends_on: [PLAN-831, PLAN-832]
---

# [PLAN-841] UI领域模型、AST与Token及lowering终态解耦

> **全Plan启动锁G841**：U831且U832。包含T-01盘点/probe/度量、fixture/测试和建树，门未齐一律不开始；只核验上游状态/完整交付收据。启动证据定义见[共用合同](reports/m2-governance-order-841-844.md)§1。本轮仅规划。

## 0. 变更摘要

完成R10：把UI专属节点/词法领域分类/解析产物的生产所有权归UI模块，Core只保真正通用模型与扩展/编译准入契约；真实UI模型经校验/lowering后进入类型、VM和四生成器。旧共享AST的UI兼容臂必须退出，不能只换文件位置。

本Plan是4份大模块合同之一，包含实际生产改动、旧实现退出、测试/三仓消费/版本与发行同步，不另按小探针取号。不接手831/832尚未完成的搬仓验收，不扩大为全语言IR替换、新backend/功能或远端发布。

## 1. 目标

- 基础AST/词法/Parser API/SDK normal/build/dev无UI领域模型/实现与Man边；Core同名/共享语义保持。
- UI owned节点、源位置/导入、ToNode/Atom等现行投影、Clone/生命周期及缓存身份明确；未准入领域产物显式拒绝。
- 类型/执行/生成器/renderer与外部消费者全部接新模型，保持现行支持面与三仓版本/发行；原UI模型实现删除。

## 2. 架构方案

T-01必须做有界整链可行性决策：在同一组内对一个含Core表达式/跨文件导入、widget/model/store/style/action的代表工程，证明可在不反依UI实现的Core契约中保存/传递owned领域产物并完成校验/lowering→类型/VM/真实生成。比较显式领域module wrapper、中立受控扩展载荷或其他等价方案；确定source/codec/schema、visitor/clone与缓存寿命、旧投影/数据版本策略、所有消费出口后才能T-02。不能把closure enum无法跨crate扩展当作复制Ast的理由；无法保持合同则needs_replan，不交空壳。

UI私有类型必须由UI发布包拥有；Core接口如需中立扩展，只承载具名有版本/校验/所有权规则的契约，不能把UI字段改名塞回Core或使用全局Any/裸指针/side-table规避所有权。模块持有的领域数据若采用受控组合是候选，须T-01证明其类型/序列化/最后owner退出和拒绝未校验输入，不预先认定已经可行。

逐语法区分UI专属与Core共享：View在借用/类型/普通标识符、OnEvents/task/msg、SceneDecl的Godot等真实非UI消费者必须做合法对照；不能按字符串把所有View/On/Msg/Scene节点删除或重命名。旧序列化/缓存数据要么版本化迁移保持承诺，要么在准入前明确拒绝并给迁移方式，不能把新模型误当旧字节读绿。新数据版本切换由本Plan负责，842再退役过渡适配。

UI转换到通用执行表示是本Plan完整生产功能，不是新AC/AV/ABI路线或全语言IR替换；现有类型/求值序/消息与绑定/样式/诊断源位置/资源退出不可漂移。低层移除所有UI专属承载后，必要兼容只在UI上层，清单交842；必需consumer与发行同步不得延期。

## 3. 技术栈

现有Rust/Cargo与三仓固定Git版本、单源AST/值/词法/Parser API/SDK、UI文法/生成器/runtime及Man CLI，复用777边界、792发行与已有真实consumer/双端设施；不新增通用框架/平行runner。实际包名/路径以启动后上游来源映射冻结，当前源码仅观察基线。

## 4. 需求分析与背景调查

用户2026-10-24明确要求按4份较大计划规划交付目标、验收与顺序/831832启动前置；继承此前保守排程，全部实施任务严格等完整前置交付。**本轮只在语言主检出建立合同/设计/路线，不改实现或canonical Specs/ledger、不建树/agent、不写旧仓/远端**。已有授权只覆盖目标与规划，未来work按明确执行指令和门启动；常规机制选择不需重复批准，改变既有支持/兼容承诺则具体needs_replan。

从[Spec入口](../specs/overview.md)定位，[AST](../specs/auto-ast/project.md)、[词法](../specs/auto-lex/project.md)、[Parser API](../specs/auto-parser-api/project.md)、[SDK](../specs/language-sdk/project.md)、[前端](../specs/auto-lang/frontend/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[旧仓](../specs/repository-boundaries/project.md)与[发行](../specs/repository-boundaries/generated-ui-distribution.md)为当前依据。部分旧Spec数量/路径滞后，不以旧聚合描述覆盖实际模块代码/交付；观察与假设、源SHA/hash见[输入](attachments/m2-governance-batch-20261024/inputs.json)。

已观察写面：crates/auto-ast/src/ast.rs::Stmt与ast/ui.rs；auto-lex/src/token.rs的View/On与词法分类；auto-parser-api/src/{lib,ui_context}.rs；SDK parser.rs/ParserBridge/dialect、types/infer、vm/codegen与trans；UI grammar/aura/a2ui/ui_gen/registry/runtime、三仓生成/CLI消费。832后路径以来源映射重定位。

当前831 executing 3/12、832 drafting 0/12；都未完成M1交付。新机制/真正可删候选/未来路径不声称已证明，T-01在G841后承担有界决策和实际编译/消费probe，关闭条件见§2/8。

## 5. 详细设计

### 5.1 启动、所有权与执行入口

G841对T-01..12逐行生效，depends_on是全Plan启动门，不是只约束最终集成。U定义为全部AC/final review/冻结门/实际合入/Spec与受管ledger/来源交接/归档收尾闭环；阶段pass不放行。门未齐只读上游状态/交付收据，保持drafting 0/12，不提前盘点/扫描/probe/改fixture/建树或启动实施agent；无新后端status/调度功能。

G841后T-01读取所有已发布三仓规范/agent指令、前置delivery commit/路径映射，新增attachments/841/contract-baseline.md：具体producer→数据/状态owner→consumer/产物、支持/预算/错误/退出规则、正反例与可执行oracle、未知决策/owner/关闭条件。假设必须在依赖实现前用最小真实整链probe证明；无解则保范围转needs_replan，不用单元假绿偷降AC。

### 5.2 三仓生产接线与来源

按共用合同§2–4及§2本Plan专门设计实施。所有类型/状态/算法单一owner；normal/build/dev与feature图无UI/Man反向污染SDK，不靠宽化777或临时[patch]双source过门。变动同时更新真实三仓consumer、固定版本、测试ID/alias/CI、build输入/生成import/资源/producer与缓存身份；832的旧发行失败合同继承，不新造弱事务。

版本升级/兼容差额记录在来源清单；外部正常消费与具名错误/缺件/取消/退出必须实际验证，未测环境pending。正式API或序列化不兼容仅按本Plan明确版本迁移合同实施，需改变既有承诺则停止该项给具体决策，不把用户规划授权扩张为随意删除稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-ast/project.md, docs/specs/auto-lex/project.md | 共享AST的UI兼容节点/Token分类→UI拥有领域模型与中立Core分类，保非UI同名语义 | 移出最初UI焊点 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md | UiParseContext具体UI返回/旧文法消费→受控owned产物及校验/lowering端口 | 领域/通用契约分离 | AC-01..05 |
| SD-03 | modify | docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md | 生成器/renderer消费基础Stmt UI臂→消费UI owned模型与受检执行表示 | 真生产切换 | AC-01..05 |
| SD-04 | modify | docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/repository-boundaries/generated-ui-distribution.md | SDK/前端/发行的旧模型入口→新准入/序列化缓存版本、源身份与外部消费者 | 终态可分发 | AC-01..05 |
| SD-05 | add | docs/specs/ui-domain-model/project.md | 无终态领域模型规格→UiAst所有权/分类/准入/lowering/codec/寿命与支持边界 | R10完整交付 | AC-01..05 |

## 6. 测试设计

下列Case均owner_stage=work，由T-01冻结实际输入/入口与oracle，T-02..08逐步建立永久定向控制并补真实集成；**本轮尚未执行**。原ast-model/ui-grammar/language-sdk/generated-ui/tool-client等consumer与语料/投影golden按真实owner复用，不以重写golden遮回归。新增fixture用`test/fixtures/modularity/`下具名目录，具体接口T-01冻结；新接口未存在前不把未来命令当已执行。

| Case | AC / invariant | 输入/真实入口 | 可观察oracle与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 | 独立Core consumer无UI checkout；插入UI实现依赖/残留UI字段/重复类型负控 | metadata/tree及结构/类型consumer共同验零专属承载；Core parse/compile/VM真实成功，不只grep零命中 |
| V-02 | AC-02 | UI widget/store/model/style/actions与Core view借用/on/task/msg/Godot场景、fn体同名变量 | 各自分类/结果/错误/求值序与冻结oracle一致，错误域/未lower产物在执行前具名拒绝，不误删共享合法语法 |
| V-03 | AC-03 | 嵌套/跨文件import、clone、序列化往返、坏tag/旧schema、缓存重启、drop前后owner | UI/源范围与错误字段保持；旧数据依合同迁移或拒绝；Weak/可观测资源证明最后owner退出，无全局泄漏/别名悬空 |
| V-04 | AC-04 | 真实三仓consumer→新UI AST→四生成器/VM/Iced与已支持生成工程；fresh/locked/重定位 | 真实产物构建/行为一致，新import/source正确且AST/Value/协议唯一；无fake正控或只更新golden |
| V-05 | AC-05 | 原UI AST投影/解析/生成测试ID映射、资源缺失/数据版本错及新consumer | 按ID原档仍选中、反控红恢复绿，投影差额具名，新增模块和测试单一owner |

work/review只必要check（真实SDK/被改包）、相关包/模块/fixture和生产反例、实际外部工程构建运行；777 strict与792适用面/缺源负控不能省。类型/VM/trans/UI生成触面final review通过后，merge在冻结三仓输入统一裸cargo t及tv/tt/tu等适用档（按迁后alias/owner等价入口），tf到期语言主检出单实例并覆盖迁出家族。修复先完成本轮全部findings和反例/邻接组合再提交送审；无单finding全档/边改边审。缺环境/上游不能fake正控，当前无指定agent/model，不创建会话；本轮Category A无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | V-01 Core AST/词法/端口零UI专属承载与反向实现依赖，分类单源、真实独立Core消费。 |
| AC-02 | work | V-02 UI owned模型/校验/lowering及全部旧UI专属臂退出，Core共享语义/正常与错误程序不退化。 |
| AC-03 | work | V-03 跨文件/clone/序列化/缓存/源位置/最后owner与拒绝未校验数据有可观察控制。 |
| AC-04 | work | V-04 真UI四生成器、VM/renderer及CLI/外部consumer/fresh/locked全部闭环，三仓source单身份。 |
| AC-05 | work | V-05 原测试ID/门/资源与数据版本迁移齐备，无未批准旁路、SD/来源/兼容清单交842。 |
| AC-06 | review | 独立最终review绑定r1/实际三仓commit及依赖/配置/输入，全AC/SD/遗漏/警告/格式/debug审查通过；merge门pending |
| AC-07 | merge | 并集合流影响复审/冻结门通过，实际源码与consumer版本交付、canonical/受管ledger/来源读回、归档收尾与guard清理闭环；不以部分发布放行后继 |

## 8. 执行步骤

12项，9 work+1 review+2 merge；全Plan启动门覆盖所有行，保守串行，不提前做独立小任务。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G841完整核验 | [✅ 已完成] [contract-baseline](attachments/841/contract-baseline.md)：三仓消费图/语法分类/寿命与资源边界冻结；泛型 DomainCode/UiStmt 整链实证（Core check、UI两包check、全部测试目标构建、代表lower→Codegen→VM=1、四生成器、跨文件导入与Core同名对照） | AC-01..05 |
| [x] | T-02 | work | G841；T-01 | [✅ 已完成] UI owned 模型/visitor/codec 与 Core serde 中立契约落地：`UI_AST_SCHEMA_VERSION` 版本信封（domain/schema/tag 具名拒绝）、递归 `UiAstVisitor`（UI/View/Core 全走）、`Shared<T>` 仅经显式 serde adapter（无全局对象表）、Core `ast/route.rs` 模型副本删除（RouteDef 单源归 dialect）；回归套件全部迁 `parse_domain(&UiDialect)` 显式端口（55 项文法回归 + plan778 回滚不变量经新端口复证 + design_tokens 9 项迁 compiler/tests + trans 两族纯 Core）；codec 含真实解析 span/源行/字段往返与坏 domain/version/tag 反例 | AC-01,03 |
| [ ] | T-03 | work | G841；T-02 | 迁UI专属Token分类/文法端口/ParserBridge返回与默认/显式装配；Core同名语义正反控 | AC-01,02 |
| [ ] | T-04 | work | G841；T-03 | 实现完整校验/lowering和类型/编译/VM消费准入；删除低层UI专属Stmt/投影臂，保源诊断与求值序 | AC-01,02 |
| [ ] | T-05 | work | G841；T-04 | 迁UI分析/四生成器/registry/runtime/renderer生产消费到新owned模型，非UI trans保持 | AC-02,04 |
| [ ] | T-06 | work | G841；T-05 | 跨文件/Clone/投影/缓存版本与重启/资源last-owner拒绝路径完整V-03；过渡上层adapter逐项登记 | AC-03 |
| [ ] | T-07 | work | G841；T-06 | 三仓调用方/manifest/固定版本/生成import/数据迁移/777与792、测试ID选择面同步 | AC-04,05 |
| [ ] | T-08 | work | G841；T-07 | V-01..05完整正反控、真正UI与外部工程fresh/locked/重定位，类型/source/资源/失败归因齐备 | AC-01..05 |
| [ ] | T-09 | work | G841；T-08 | 删除原模型实现并复核零残边，冻结SD-01..05及attachments/841/handoff.md、交842的过渡清单，提交送审 | AC-05 |
| [ ] | T-10 | review | G841；T-09冻结提交 | 同组独立完整AC/SD/实际consumer/反例/来源与健康审查；不先跑整档，不将merge门当已通过 | AC-06 |
| [ ] | T-11 | merge | G841；T-10 pass | 并集影响复审、冻结三仓适用门与有序实际合入/固定版本/来源读回；若改变输入重审影响面，暂不放行后继 | AC-07 |
| [ ] | T-12 | merge | G841；T-11全部交付收据 | canonical/受管ledger/索引沉淀、归档与逐树fresh wt-guard clean清理，整Plan闭环构成U841才放行下一份；保历史/恢复/交付分支 | AC-07 |

G841未满足前禁止创建/认领以下组、分支或work agent。未来组`D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，每个实际需写仓各一树/分支`codex/plan-841-dev`；只读consumer用固定版本clone而非借其他计划WIP，未需写仓不多建树。work/review/merge沿同组顺序交接，不建第二审查树；若auto-down为现有可选依赖则具名固定版本/detached兄弟，只解析不改实现。禁止junction/symlink，主检出只簿记，清理前每树fresh guard clean且保留实际交付/legacy恢复源。

## 9. 复审记录

2026-10-26 / stage: work（T-03 部分单元） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（T-03 未勾选；两个可验证单元已落地，SDK 语义转换被 runtime 编译破损阻断）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / code_commit: lang `ce44edf12`（T-03u1：Core 同名语义正反控 3 项永久测试——`.view` 借用、fn 体内 `msg` 普通标识符、裸 `parse()` 在 Core/UI 双场景拒绝 UI 语句〔无隐式文法装配反控〕；parser-api/parser_bridge 陈旧 UiGrammar/ui_context 文档改写为 domain 端口现实）+ `3fab3ab30`（T-03u2：SDK 安装器四处 `set_ui_grammar(ui_grammar())` 与 `UiGrammarSource` inventory 摘除，宿主能力钩子注册保持 PLAN-779 合同不变；验证=悬空引用错误 4→0 错误签名差分） / evidence: `cargo test -p auto-lang-core --lib plan841` 3 绿；`cargo check -p auto-lang-core -p auto-parser-api` PASS；auto-lex 勘定=无 UI 专属 Token 分类需迁（View/On 共享分类保持，反控覆盖）；rustfmt 触面文件通过（SDK lib.rs 被**预存**缺失 `mod mem_profile` 阻断，未触碰该声明） / blockers: T-03 剩余=SDK `ui_build` 等入口 plain-parse→parse_domain 语义转换 + 旧 Parser UI 返回门面（parser.rs 死代码 parse_widget_decl 等零调用，删除归 T-04 行）；SDK 全部验证被 runtime 无条件依赖阻断（见 §10 拓扑，T-05 owner） / next_responsible_role: work T-03 续（runtime 修复后补 SDK 语义转换与验证）或按 §10 拓扑先做 T-04/T-05 可编译化

2026-10-26 / stage: work（T-02） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（T-02 完成；Plan 保持 executing 2/12）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / base_commit: 承 T-01（lang `eccf8601e`、UI `992fbe4`） / code_commit: lang `50eb22dc9`、UI `d6d499b` / dependency_revisions: 同 T-01（down detached 只读、Man 固定版本） / task_ids: T-02 / evidence: `cargo check` lang 五包（ast/lex/parser-api/val/lang-core）PASS 且 lang 侧改动为纯 serde derive 零语义变化；`cargo test -p auto-ui-dialect` 90 单测 + 2 doctest 全绿（codec 4 测：往返+clone、坏 domain/version/tag 具名拒绝、visitor 计数、真实解析 span/源行/字段往返）；`cargo test -p auto-ui-compiler --lib` 1121 全绿（plan426 await 拒绝经领域端口保持）+ `--test design_tokens_recipe` 9 绿；dialect 包零警告（T-01 迁移遗留警告已清）；触面文件 rustfmt 通过 / 关键决策: ①design_tokens 回归因 dev-循环依赖类型分裂（dialect↔compiler）不可留在 dialect，迁 `auto-ui-compiler/tests/design_tokens_recipe.rs`（正向依赖类型同源）；②plan778 三段注册表测试改断言 builtin 恒空（内建 UI 方言已移入显式领域端口），None-回滚不变量经自定义 DomainDialect 在 `parse_domain` 复证；③vue plan426 测试改经领域端口（裸 `parse()` 不再载 UI 文法=T-01 终态设计，该红为 T-01 预存） / blockers: 无阻塞 T-02 验收，但登记两项 T-01 携带的预存破损供后续任务收口——①`auto-ui-runtime` 自 T-01 起 82 编译错（消费已删的 Core UI 臂/`ast::ui`/`resolve_use_module`，SDK `auto-lang` 因其失败；修复 owner=T-05 runtime 迁移，merge 前必须闭环）；②`auto-ui-compiler` 19 个 doctest 红（`auto_lang` crate 不可达，PLAN-814 迁移遗留；owner=T-07 测试ID/选择面同步时归因处置） / next_responsible_role: work T-03（Token 分类/文法端口/ParserBridge 迁移与 Core 同名语义正反控）

2026-10-26 / stage: work（T-01） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（T-01 完成；Plan 保持 executing 1/12）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / base_commit: lang `100172980efb0343fe7016a05eb40f18d0014d74`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc` / code_commit: lang `eccf8601e`、UI `992fbe4` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（detached 只读）、Man `1b7a9ce0de9055f7953129d78a25777662508258` / task_ids: T-01 / evidence: [contract-baseline](attachments/841/contract-baseline.md) §6；Core check、UI两包check、全部测试目标构建、代表lower→Codegen→VM=1、四生成器与跨文件导入通过 / blockers: 无；全仓 fmt 被 auto-cosmic/linux.rs 与 auto-lang/examples/ui_container.rs 预存语法问题阻断，触面包 fmt 及复核通过；merge 统一门 pending / next_responsible_role: work T-02（模型/visitor/codec与独立所有权控制）

2026-10-26 / stage: work（G841 启动门核验） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **G841 满足，进入 T-01；Plan drafting→executing 0/12** / gate_evidence: U831=r3 archived 12/12，最终 review/统一门/landing/ledger v84/归档/三树 guard 清理闭环，lang delivery `d7f378b676e3a9dac7257f609936455e5d8248fe` 与 UI delivery `aee499e87093ff727f68ac8467c506e9ecf6ecdc` 均为当前 master 祖先/HEAD；U832=r3 archived 12/12，R5 收口/统一门/landing/ledger v89/归档/四树清理闭环，lang delivery `3934ed4f28e8c994e92129a44f987bc9067e9e21` 与 Man delivery `1b7a9ce0de9055f7953129d78a25777662508258` 均为当前 master 祖先/HEAD / baseline: auto-lang `100172980efb0343fe7016a05eb40f18d0014d74`、auto-ui `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、auto-man `1b7a9ce0de9055f7953129d78a25777662508258` / worktrees: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，各仓 `codex/plan-841-dev`，根路径与分支断言通过 / observations: 语言主检出他 session 文档 WIP 及未跟踪构建产物未纳入，三仓主检出代码零修改 / next_responsible_role: work T-01（冻结来源/语法/消费图与整链可行性决策）

2026-10-24 / stage: new / plan_id: PLAN-841 / revision: 1 / outcome: blocked（上游整Plan启动门未齐；文档合同预检通过）/ next: 等G841真实满足后读回规范/交付/归档版本，再work T-01 / changed: T-01..12、AC-01..07、SD-01..05、V-01..05。本Plan drafting 0/12，无实施/fixture/probe/测试已完成，未创建树或agent；[预检](attachments/m2-governance-batch-20261024/preflight.md)。

## 10. 待澄清事项

当前调度/范围已明确，不需重复确认；T-01负责§2机制、实际三仓路径/消费者与资源策略的有界冻结，门前不得提前probe。无法保持合同或需改变长期支持/源码语义/后端/远端动作则具体needs_replan；不静默缩面、豁免源身份或把本Plan必需集成延期给后继。全部实施等前置完整交付合入/归档收尾，不以status/单阶段pass越门。

**T-02 盘点的破损拓扑（供 T-03..T-05 排期，不需用户裁定但须知晓）**：`cargo check -p auto-ui-runtime`（UI 树，组内 ../auto-lang 解析生效）= 83 错两组——①runtime 自身消费已删 Core UI 臂/`ast::ui`/`resolve_use_module`（owner=T-05）；②feature 统一点点亮的 auto-lang-core 门控面（vm/native.rs 72、ffi/engine/trans 等，owner=T-04 VM 准入或 T-05 视具体面）。`auto-ui-runtime` 是 SDK `auto-lang` 的**无条件**依赖（default-features=false 仍必选），故 SDK（含其 lib.rs 内 4 处 `set_ui_grammar(ui_grammar())` 悬空引用与 inventory UiGrammarSource 装配，owner=T-03）在 T-05 完成前任何配置都不可编译——T-03 的 SDK 侧改动只能以 core/parser-api/dialect/compiler 档验证，SDK 自身验证顺延至 T-05 后补，merge 门（AC-06/07）前必须全绿闭环。
