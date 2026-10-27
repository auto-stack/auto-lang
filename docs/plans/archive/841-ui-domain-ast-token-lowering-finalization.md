---
plan_id: PLAN-841
status: archived
feature_name: UI领域模型、AST与Token及lowering终态解耦
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-27
plan_revision: 1
current_step: 12
total_steps: 12
supersedes_spec_components: [docs/specs/auto-ast/project.md, docs/specs/auto-lex/project.md, docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/auto-ui-dialect/design/full-grammar.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md, docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
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
| SD-02 | modify | docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/auto-ui-dialect/design/full-grammar.md | UiParseContext具体UI返回/旧文法消费→受控owned产物及校验/lowering端口 | 领域/通用契约分离 | AC-01..05 |
| SD-03 | modify | docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md | 生成器/renderer消费基础Stmt UI臂→消费UI owned模型与受检执行表示 | 真生产切换 | AC-01..05 |
| SD-04 | modify | docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md | SDK/前端/发行的旧模型入口→新准入/序列化缓存版本、源身份与外部消费者 | 终态可分发 | AC-01..05 |
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
| [x] | T-02 | work | G841；T-01 | [✅ R2 修复复证] dialect 校验与 `serde_stacker` 深视图支持保持；decode 在信封后强制 `Deserializer::end()`，拒绝垃圾/第二份 JSON，合法空白通过；codec 9 项含 48 层、字段/span/source_lines、schema/tag/语义/payload/Weak 全绿，见 [R2 修复验收](attachments/841/repair-r2-closeout.md) | AC-01,03 |
| [x] | T-03 | work | G841；T-02 | [✅ 已完成] View/On 共享 Token 保留，UI 文法仅经 `Parser::parse_domain(&UiDialect)`；SDK 安装器/入口函数表装配退役，Core `.view`/普通 `msg` 与裸 parse 拒绝面 3 项控制绿 | AC-01,02 |
| [x] | T-04 | work | G841；T-03 | [✅ 已完成] Core UI Stmt/投影与旧 Parser UI 返回门面退出；`lower_ui` 校验后分离 UI owned/Core Code，十类语句裸 parse 拒绝 + 领域正控，Core 3 项与 ui_ast 4 项绿；代表 lower→VM/四生成器保持 | AC-01,02 |
| [x] | T-05 | work | G841；T-04 | [✅ R3 修复复证] R2 工程准入保持；共享 file/module 预检先按各自目录 `prepare_style_recipe_imports` 再 parse/lower，冷态 local/named/wildcard full/incremental 实际 class 全过，私有/缺失/前一文件残留负控保持；tools 6 项与既有 recipe 9 项绿，见 [R3 验收](attachments/841/repair-r3-closeout.md) | AC-02,04 |
| [x] | T-06 | work | G841；T-05 | [✅ R2 修复复证] [R2 修复验收](attachments/841/repair-r2-closeout.md)：codec 完整 artifact 检查恢复，合法空白/深视图/字段/span/source_lines/版本/tag/语义/Weak 保持；payload 形状反例新增；codec 9 项绿 | AC-03 |
| [x] | T-07 | work | G841；T-06 | [✅ F841-R5-01 修复] 嵌套 Array/Object 快照递归持有，重播逐层物化为 ListData/ObjectData；UI `bfad5149`、Man源pin `6330ce00`，实际引用身份与最后释放永久控制齐备 | AC-04,05 |
| [x] | T-08 | work | G841；T-07 | [✅ R5修复复证] 原两败例及直接对照完成queue/replay/Init/最后释放；复合cancel/missing/drop/别名及R5既有owner重播通过；受影响定向27项全绿，见 [R5修复验收](attachments/841/repair-r5/closeout.md)；merge门pending | AC-01..05 |
| [x] | T-09 | work | G841；T-08 | [✅ R5修复冻结] 触面rustfmt/diff-check通过，临时probe撤除，三实施树干净；候选SD-03补递归容器规则，lang `ecc2f54a7`、UI `bfad5149`、Man `6330ce00`，交受影响review | AC-05 |
| [x] | T-10 | review | G841；T-09冻结提交 | [✅ R6受影响final pass] F841-R5-01关闭；原producer三形态、混合Init/视图消费、失败后别名与全部容器最后释放通过，18个具名行为测试全绿，见 [R6复审](attachments/841/review-r6/review.md)；统一门merge-owned pending | AC-06 |
| [x] | T-11 | merge | G841；T-10 pass | [✅ 完成] 统一门全绿（GATE-01..06 关闭链见 §9 merge 收据）：ui t（a4/a5 金丝雀绿，FAIL⊆预存对）/tu/ti、lang t 3183/tv 163/tt 3555（tu 归因 B855-FEATURE-01 既有阻断）、Man t/tcore/ti 42/43（816-smoke 预存）、发行 DST/FLR 14/14、真实 792 39/39、777 strict 0 violations（基线显式重采样 1c81c8f4）；有序合入 UI 4903bef → lang f16bb468b（两轮 rebase、代码 delta 哈希等价 f6e6d7fd、goals.md 并集）→ Man 9a75d3b，全部 ff-only；来源读回=792 收据绑定 lang 1c81c8f42 | AC-07 |
| [x] | T-12 | merge | G841；T-11全部交付收据 | [✅ 完成] canonical SD-01..05 随分支落地（ui-domain-model 新模块+INDEX/overview+goals 841 条目，spec-index 幂等）；ledger guarded offline projection v100→v101（P841-D1/T/R6/M 四项，HTTP store 不可达存证在案）；归档 docs/plans/archive/ + 收据提交；逐树 wt-guard clean 清理（保留组内外来 auto-down WIP 与历史 scratch） | AC-07 |

G841未满足前禁止创建/认领以下组、分支或work agent。未来组`D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，每个实际需写仓各一树/分支`codex/plan-841-dev`；只读consumer用固定版本clone而非借其他计划WIP，未需写仓不多建树。work/review/merge沿同组顺序交接，不建第二审查树；若auto-down为现有可选依赖则具名固定版本/detached兄弟，只解析不改实现。禁止junction/symlink，主检出只簿记，清理前每树fresh guard clean且保留实际交付/legacy恢复源。

## 9. 复审记录


2026-10-27 / stage: merge（merge-r2 轮，GATE-05/06 修复链收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（T-11/T-12 完成，U841 闭环）** / delivery_commit: lang `f16bb468bce13987cb6c69c24e985561da3a7fda`（21 提交两次 rebase 后 ff-only；rebase 前冻结链 ecc2f54a7→9d96d0f76→1c81c8f42，代码面 delta 哈希等价 `f6e6d7fd`，docs/plans 冲突一律取主检出权威簿记、goals.md 双侧并集[GOAL-003/007 的 841 条目保留+master 853-858 条目保留]、.autoos/specs.json 对 master 零增量）、UI `4903bef64532750d1597627b1197f3d2cac1effc`（15 提交纯 ff-only）、Man `9a75d3bab6eaeda961d5dcbf7932258f89e650af`（10 提交纯 ff-only）；down 固定 `c39ac7f9` 只读不变 / ordered_landing: UI→lang→Man 全 ff-only（master tip==delivery commit 逐仓核验；主检出冒烟 `cargo check -p auto-lang -p auto-lang-core` 绿；主检出他 session 文档 WIP 经 stash→merge→pop 全程保全，00-intro 章号并集 49） / ledger_refreshed: `.autoos/specs.json` v100→v101，guarded offline projection（HTTP store 不可达，[探测](attachments/841/merge-r2/store-channel-r2.json.txt)；条目 P841-D1/T/R6/M 四项带 file/tags/source-sha；byte 级备份 `.autoos/specs.json.bak-plan841-merge`；schema/读回/无关段零扰动守卫全过；live 状态词表镜像补 `active`） / findings_closed: **M841-GATE-05**（parked Init 段 resume 失败不回写需求相位→静默翻 Done；修复=两处 resume 失败臂 `mark_init_demand_failed_by_fn` + max_steps=1 确定性回归，UI 4903bef，独立受影响复审 pass 含红-绿反证，[finding](attachments/841/merge-r2/M841-GATE-05/finding.md)/[复审](attachments/841/merge-r2/repair-review-gate05.md)）、**M841-GATE-06**（AutoStr=ecow::EcoString 别名 serde 面缺口——安装态 Core SDK 独立闭包 346 编译错；修复=auto-val serde feature 自链 ecow/serde，lang 9d96d0f76，独立受影响复审 pass 含反事实因果证明，[复审](attachments/841/merge-r2/repair-review-gate06.md)；附带 Man 主/Core 锁 serde_stacker+serde 边同步 e32bed1/9a75d3b、777 基线显式重采样 1c81c8f42[stale_baseline 按设计拒绝默默通行]） / gates: ui-tool-build ✓、ui-t ✓（a4/a5 两跑金丝雀全绿、FAIL 集精确⊆预存对[index_css golden+merged_api_client，pre-841 主检配对同签名复现](attachments/841/merge-r2/preexisting-reds-attribution.md)）、ui-tu ✓、ui-ti ✓、lang-t 3183/3183 ✓、lang-tv 163 ✓、lang-tt 3555 ✓、te-child --locked ✓、lang-tu=既有 B855-FEATURE-01 编译阻断（av_source_827/av_owned_821，归因 821/827/822，fix-forward 859，841 未触碰该文件）、Man t ✓/tcore ✓/ti 42/43（816-smoke 预存族，832 先例同集）、发行 DST-1..7+FLR-1..6 **14/14** ✓（[distribution-receipt](attachments/841/merge-r2/distribution-receipt-r2.json.txt)）、真实 792 **39/39** ✓（[收据](attachments/841/merge-r2/c-792-receipt.json) 绑定 lang 1c81c8f42）、777 strict 0 violations ✓；新冻结基线复跑：发行门/777/792/lang t/tv/tt 全部绑定最终交付身份 / known_flake_registered: plan492 金丝雀恢复段（:460 36≠44）1/4 整档频率、隔离/2 并发恒绿、失败运行零 Init 失败证据——M841-KNOWN-FLAKE-01 登记债项（KNOWN-DEBT 工作树+本收据），后续批量命中按债项路由 / evidence: [merge-r2 全目录](attachments/841/merge-r2/)（gates json、各档日志、归因与复审报告、[range-diff 映射](attachments/841/merge-r2/lang-range-diff.txt)） / next: 归档+清理（T-12 收尾） / cleaned: 三树 fresh wt-guard clean（auto-lang/auto-ui/auto-man 逐树核验，tracked 零脏）→ worktree+`codex/plan-841-dev` 分支移除（branch -d 证实已合并：f16bb468b/4903bef/9a75d3b）；组目录保留外来 auto-down WIP 树、auto-down-clean 与 scratch-792*（既往收据明令不动）；发行产物观察=[无主检出 release 产物，auto-musk web bundle 2026-10-03 非 841 触面](attachments/841/merge-r2/artifact-observations-r2.json.txt)；批量回归到期判定=[due（855 遗留 B855-FEATURE-01 编译阻断，859 fix-forward 待执行；本计划未触碰阻断文件，重试归 855/859 流）](attachments/841/merge-r2/batch-due-check.json.txt) / outcome: **merge pass，U841 闭环**。

2026-10-27 / stage: review / review_scope: final（R6，F841-R5-01受影响复审） / reviewer_identity: Codex `/root`，参与过G04实施/R5复审，未实施本次bfad514修复；从冻结提交/真实输入重建结论，非全Plan全新独立context，未创建agent / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / reviewed_commit: lang `ecc2f54a7ada58cd5cd015a9f21bea48debb4284`、UI `bfad5149f8f1d1981465d06d759e95cc8127ab75`、Man `6330ce00e594c99a4ef0a931dc85137ac47474fb` / base_commit: lang `88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80`、UI `2f311868f082bc4ab6fac9d484f71f18d13b1c95`、Man `9b4302fa585e6cda4a91738a6e9057fae12c9402` / dependency_revisions: down固定只读 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，依赖/配置/输入hash见 [baseline](attachments/841/review-r6/baseline.json.txt) / acceptance_results: AC-01/02与未变codec等证据保留；AC-03/04/05受影响递归所有权及消费者通过，AC-06 pass，AC-07 pending / findings: **F841-R5-01 CLOSED，无新增阻断**；两条原反例与直接对照转绿，递归快照/逐层堆容器配平，混合Init实际读7/视图显示7，Failed后共享别名存活及source和四层容器最终Weak失效 / evidence: [R6报告](attachments/841/review-r6/review.md)、[结束冻结检查](attachments/841/review-r6/final-state.json.txt)；13永久+3原输入重播+2混合行为=18个具名测试通过，另有AST诊断；原canary healthy36/runtime14不变，初始不适用探针及换行复制错误保留并解释；fmt/diff/pin/hash检查通过，临时probe撤除，三实施树clean且HEAD未变 / spec_inputs: SD-03候选delta SHA256 `2ff00020fa7de68fd51041356cedc3f1efbbd7f0d3cc2f6b4013fe6233423708`，规则与代码一致，metadata/goals不变，无canonical/live ledger发布 / pending_closeout_gates: 裸cargo t、适用tv/tt/tu、UI/Man自有档、冻结777/792/三仓发行及有序交付、Specs/受管ledger/归档/guard均merge-owned pending；未跑整档 / next: **merge**在当前冻结提交执行门禁及交付；Plan **reviewed 10/12**，T-10完成，T-11/12 pending；不宣称合入或U841。

2026-10-27 / stage: work（F841-R5-01 needs_fix 修复） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（execution_done 9/12，待受影响复审）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: lang `88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80`、UI `2f311868f082bc4ab6fac9d484f71f18d13b1c95`、Man `9b4302fa585e6cda4a91738a6e9057fae12c9402` / code_commit: lang `ecc2f54a7ada58cd5cd015a9f21bea48debb4284`（仅候选Spec）、UI `bfad5149f8f1d1981465d06d759e95cc8127ab75`、Man `6330ce00e594c99a4ef0a931dc85137ac47474fb`（仅源pin） / dependency_revisions: down固定只读 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` / task_ids: T-07,T-08,T-09 / evidence: [根因、关闭矩阵与readiness](attachments/841/repair-r5/closeout.md)、[冻结hash与日志](attachments/841/repair-r5/baseline.json.txt)；原R5两败例在修复前重放为直接对照过/嵌套两败，修复后真实producer三形态均完成queue/replay/Init/最后释放；复合cancel/missing/drop/共享别名、R5三项owner及既有图表/状态/生命周期共 **27项定向全绿**，无选择内skip；fmt/diff-check通过，临时probe撤除，三实施树clean / findings: **F841-R5-01实施侧关闭，受影响验收review pending**；快照递归持有Array/Object，重播逐层物化并沿用直接子引用RC协议，未复制引用身份、未永久保留快照、未改变Int分类/调度/失败语义 / spec_inputs: SD-03候选delta SHA256 `2ff00020fa7de68fd51041356cedc3f1efbbd7f0d3cc2f6b4013fe6233423708`，未发布canonical/live ledger / blockers: 无本finding的work-owned缺口或未决语义；T-10受影响复审及T-11/T-12统一门/发行/合入/沉淀/归档pending / next_responsible_role: review——核验递归所有权、堆容器最后释放、真实producer及候选SD；禁止把本work记录当独立验收或旧门收据当新冻结门通过。


2026-10-27 / stage: review / review_scope: final（R5，M841-GATE-04受影响复审） / reviewer_identity: Codex `/root`，本会话参与前轮实施，明确非新独立context；从冻结代码/实际producer和补充反例重建结论，未委派agent / plan_id: PLAN-841 / plan_revision: 1 / outcome: **needs_fix** / reviewed_commit: lang `88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80`、UI `2f311868f082bc4ab6fac9d484f71f18d13b1c95`、Man `9b4302fa585e6cda4a91738a6e9057fae12c9402` / dependency_revisions: down固定 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，源码/配置/输入hash见 [baseline](attachments/841/review-r5/baseline.json.txt) / acceptance_results: AC-01/02及未变codec等证据保留；AC-03 runtime ownership partial/fail、AC-04/05/06 fail，AC-07 pending / findings: G04原非法输入拒绝/恢复及合法Init失败隔离例已通过；新增 **F841-R5-01 P2 OPEN**——实际producer接受 `[[.source]]`、`[{inner: .source}]`，但快照引用遍历只覆盖顶层/直接元素，父字段替换后原对象在Child仍Queued时已释放；直接数组对照通过 / evidence: [复审报告](attachments/841/review-r5/review.md)、[finding与逐字输入/关闭oracle](attachments/841/review-r5/finding-F841-R5-01.md)；本轮15个具名定向case=13过/2败，原canary命中healthy36/runtime14保持；共享owner取消、别名重播、CPUpark换代3项通过；fmt/diff/pin检查通过；临时probe撤去，三树clean且HEAD未变 / spec_inputs: SD-03修复delta SHA256 `14cc305bfe2db4ca0b0aed6c6d892f5ae1e6e11296fada9a46a65aefd4950718`，目标正确但完整所有权未满足，不能把direct-element描述当缩减已支持属性形态的批准 / pending_closeout_gates: 所有统一门与三仓发行/777/792仍merge-owned pending；已知blocker上未跑整档 / next: work修复实际Array/Object producer的queue/replay/回收路径后提交受影响复审；Plan回 **executing 6/12**，T-07..T-10未完成，不合入/不放行U841。


2026-10-27 / stage: work（M841-GATE-04 needs_fix 修复） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（execution_done 9/12，待受影响复审）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: lang `34733e23cb6d6436de24258e435a5f3407c7202b`、UI `7f270e51ea10a4c584a96a3eb8101e57dff35908`、Man `4ad2208c5e624f5c031cebf5694eec16981d7ad0` / code_commit: UI `2f311868f082bc4ab6fac9d484f71f18d13b1c95`、Man `9b4302fa585e6cda4a91738a6e9057fae12c9402`（仅发行源pin）；语言 Rust/依赖/配置未改，候选 Spec/存证另提文档提交 / dependency_revisions: down 固定只读 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` / task_ids: T-07,T-08,T-09 / evidence: [根因、关闭矩阵与 readiness](attachments/841/repair-g04/closeout.md)、[冻结源码/配置与日志](attachments/841/repair-g04/baseline.json.txt)；**20项具名定向全绿**（runtime12、state邻接2、既有Init生命周期6），原未定义变量输入保留为正式准入拒绝/同文件恢复，合法VM零除金丝雀实际到Init Failed，三个兄弟Done，命中区 healthy36/runtime14，原相对比较/≥12底线不变；新增queue/replay/cancel/missing/dedup/drop与数组引用/真整数对照；fmt/diff-check通过 / findings: **M841-GATE-04 实现侧修复完成，独立验收复审 pending**；不吞模块错误、不关RC canary、不跳测试、不加宽时限 / spec_inputs: SD-03已有runtime目标补延迟快照所有权，候选delta及hash见baseline，无新能力或合同修订 / blockers: 无尚未修复的 work-owned finding、验证缺口或未决语义；T-10受影响复审与T-11/T-12合入/统一门/发行/ledger/归档仍pending / next_responsible_role: review——审新生产所有权路径及候选规范，未影响的Core/codec/四生成器/冷态工具准入按冻结输入复用，禁止把旧整档通过当新版本最终门通过。


2026-10-27 / stage: review / review_scope: final（R4，R3修复受影响复审） / reviewer_identity: Codex `/root`，延续R1/R2/R3独立审查上下文，未参与实施/未委派agent / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / reviewed_commit: lang `ecfa187f72d3fcf7ba662ae6cd3e1170aad75735`、UI `eb50ce67930179f51db041b74faa6d1fbc72c211`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / base_commit: R3 lang `6892bad1ae9556f93da743bb1af265625330d809`、UI `60011b3029a7e8a7b9f95aeecff821fbd448e962`，Man未变 / dependency_revisions: down clean `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；依赖/配置/输入hash见[baseline](attachments/841/review-r4/baseline.json.txt) / spec_inputs: [候选SD冻结](attachments/841/review-r4/spec-delta-reviewed.diff) SHA256 `cf1a23cd0120152df7be2ed11890336a51620023979dc614b9bc0b562b5a0c54`，SD-01..05通过；补齐SD-02/04已有候选的full-grammar与auto-lang/project目标映射，无合同语义变更 / acceptance_results: AC-01..06 pass（未变项证据按实现/依赖/配置/输入范围复用），AC-07 merge-owned pending / findings: **F841-R3-01 CLOSED**；R1/R2各finding保持CLOSED，无新增阻断 / evidence: [R4报告与AC/SD证据映射](attachments/841/review-r4/review.md)；最终22项定向通过（独立4、tools6、recipe9、compiler3）；覆盖cold local/named/wildcard full/incremental实际class、同名stale拒绝和恢复、主/子模块自身目录红蓝class、private/missing/transitive/collision及原无效数据/零输出；本轮两Rust文件fmt与repair diff-check通过；三实施树退出clean、HEAD不变 / pending_closeout_gates: **裸cargo t与适用tv/tt/tu、完整发行/777/792及三仓合流交付均仍merge-owned pending**，随后canonical/ledger/索引/归档/fresh guard清理；tf仅到期主检出单实例批量；未跑整档/未合入，不放行U841 / next: merge按T-11/T-12在冻结三仓输入完成统一门与实际交付收尾；如代码/依赖/配置/输入改变，按影响面复审。Plan **reviewed 10/12**，本pass不表示交付完成。

2026-10-26 / stage: work（R3 needs_fix 修复收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（execution_done，9/12；交受影响独立复审）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: R3 lang `6892bad1ae9556f93da743bb1af265625330d809`、UI `60011b3029a7e8a7b9f95aeecff821fbd448e962`，Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / code_commit: lang `ecfa187f72d3fcf7ba662ae6cd3e1170aad75735`、UI `eb50ce67930179f51db041b74faa6d1fbc72c211`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（固定只读 clean clone）；serde_stacker `0.1.14` / task_ids: T-05,T-08,T-09 / evidence: [R3 初始化顺序修复与 cold/负控验收](attachments/841/repair-r3-closeout.md)；原兼容 probe 2、原 request 反例 1、tools 6、recipe 9、compiler 3 全绿；full/incremental 实际 class 与 private/missing/stale 控制齐备；全 Plan 触面 rustfmt 与 diff-check 通过；三树提交后干净；R2 codec 与 R1 深度/格式证据保留 / blockers: 无 work-owned findings、覆盖缺口或未决合同；统一裸 cargo t、适用 tv/tt/tu、完整发行/777/792 与三仓交付门仍 merge-owned pending，本轮未执行整档、未合入、未沉淀 canonical/ledger / next_responsible_role: review——按 R3 finding、冷态合法矩阵、负控和候选 SD 做受影响独立复审

2026-10-26 / stage: review / review_scope: final（R3 受影响复审） / reviewer_identity: Codex `/root`，延续R1/R2独立审查上下文，未参与实施/未委派agent / plan_id: PLAN-841 / plan_revision: 1 / outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / reviewed_commit: lang `6892bad1ae9556f93da743bb1af265625330d809`、UI `60011b3029a7e8a7b9f95aeecff821fbd448e962`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / base_commit: R2 lang `a0473d515129ad1016a164fc0f3201c9334805cc`、UI `a056dd60171b9f4a1ee61fc1ade32b45d4a4ba1c`，Man未变 / dependency_revisions: down clean `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，Cargo/配置与输入hash见[baseline](attachments/841/review-r3/baseline.json.txt) / spec_inputs: [候选SD冻结](attachments/841/review-r3/spec-delta-reviewed.diff) SHA256 `a25d999e4d9b3b325eecadf547463870c9d87bb0c73d96b96e9025e0e0faf11b` / acceptance_results: AC-01保留未变范围pass，AC-03受影响codec通过；AC-02/04/05/06 fail、AC-07 pending / findings: F841-R1-01原无效数据/工具准入与F841-R2-01完整JSON项 **CLOSED**；R1深度/格式保持CLOSED；新增 **F841-R3-01 P2 OPEN（修复回归）**：共享file预检在导入recipe装载前直接解析，Vue full/incremental冷状态下合法named/wildcard样式报undefined variable；同一源码原组件入口实际class生成成功、已装载后工程成功，本地配方对照成功 / evidence: [R3报告、可复现输入/状态矩阵与关闭oracle](attachments/841/review-r3/review.md)；原request反例1、原compiler/codec/Core probe5、tools永久4、codec9全过；新增兼容probe2败，覆盖冷/已装载、named/wildcard/local、full/incremental；8触面rustfmt与repair diff-check通过 / pending_closeout_gates: 全部裸cargo t、适用tv/tt/tu、三仓交付与沉淀/归档仍merge-owned pending，未执行整档/合入 / next: work同树修导入上下文/预检时序及永久cold合法对照，保留无效输入拒绝/零输出与已闭codec；按实际状态改动复证私有/缺失配方和邻接模块后提交受影响复审。重开T-05/T-08/T-09，executing **6/12**，保留其余产出和R1/R2失败诊断；三树退出clean、HEAD不变，不放行U841。

2026-10-26 / stage: work（R2 needs_fix 修复收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（execution_done，9/12；交受影响独立复审）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: R2 lang `a0473d515129ad1016a164fc0f3201c9334805cc`、UI `a056dd60171b9f4a1ee61fc1ade32b45d4a4ba1c`，Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / code_commit: lang `6892bad1ae9556f93da743bb1af265625330d809`、UI `60011b3029a7e8a7b9f95aeecff821fbd448e962`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（固定只读 clean clone）；serde_stacker `0.1.14` / task_ids: T-02,T-05,T-06,T-08,T-09 / evidence: [两轮 final 失败诊断、真实入口映射与 R2 修复验收](attachments/841/repair-r2-closeout.md)；原 request 工程反例 1、tools 永久 4、原 codec 后缀反例 1、compiler 3、codec 9 全绿；全 Plan 触面 rustfmt 与 diff-check 通过；三树提交后干净；R1 深层 codec/格式及未变 runtime/SDK 证据保留 / blockers: 无 work-owned findings、覆盖缺口或未决合同；统一裸 cargo t、适用 tv/tt/tu、完整发行/777/792 与三仓交付门仍 merge-owned pending，本轮未执行整档、未合入、未沉淀 canonical/ledger / next_responsible_role: review——按 R2 findings、诊断、真实 request/incremental 控制和候选 SD 做受影响独立复审

2026-10-26 / stage: review / review_scope: final（R2 受影响复审） / reviewer_identity: Codex `/root`，延续R1独立审查上下文，未参与实施或委派agent / plan_id: PLAN-841 / plan_revision: 1 / outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / reviewed_commit: lang `a0473d515129ad1016a164fc0f3201c9334805cc`、UI `a056dd60171b9f4a1ee61fc1ade32b45d4a4ba1c`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / base_commit: R1 lang `acd8d99d2eb180f4861fb235d783f19dba307a38`、UI `f7eeb82d0170f69a6a53cf0e4d625caea7eddab9`，Man未变 / dependency_revisions: down clean `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，serde_stacker及锁/配置hash见[baseline](attachments/841/review-r2/baseline.json.txt) / spec_inputs: [SD冻结副本](attachments/841/review-r2/spec-delta-reviewed.diff) SHA256 `84019277499b4edc6c73d68f6db3bc58a0074c37c2a8e8b902de96da2a90379f` / acceptance_results: AC-01保留未变范围pass，AC-02/03/04/05/06仍未过，AC-07 pending / findings: **F841-R1-01 P1仍OPEN**（原Rust/Vue高层反例已闭，但正式request的Jet/Ark/Rust工程生成三路均接受重复widget并写盘）；F841-R1-02/03 **CLOSED**（原48层codec与181触面格式检查通过）；新增 **F841-R2-01 P2 OPEN**（手动codec解码缺完整消费检查，合法信封后追加garbage、{}、旧schema信封全被接受，原完整JSON对照均拒绝） / evidence: [R2报告、实际入口/可重放输入/关闭oracle](attachments/841/review-r2/review.md)；独立probe4过1败；工程入口6调用=3合法成功+3无效仍成功；compiler3、codec7、runtime2、SDK1通过；rustfmt181文件0失败，三仓diff-check通过 / pending_closeout_gates: 全部裸cargo t及tv/tt/tu等适用档、三仓统一交付及沉淀/归档仍merge-owned pending；未执行整档/合入 / next: work同树修本轮全部finding，并先给出两轮final失败后的诊断/覆盖修正（真实工具工程入口与SDK backend对账、错误传播及decoder完整输入/失败出口），定向反例/邻接合法控制提交后再复审。重开T-02/T-05/T-06/T-08/T-09，executing **4/12**；保留其余已完成产出和全部历史。三实施树退出审查均clean、HEAD未漂移；不放行U841。

2026-10-26 / stage: work（R1 needs_fix 修复收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（execution_done，9/12；交受影响独立复审）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: lang `100172980efb0343fe7016a05eb40f18d0014d74`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、Man `1b7a9ce0de9055f7953129d78a25777662508258` / code_commit: lang `a0473d515129ad1016a164fc0f3201c9334805cc`、UI `a056dd60171b9f4a1ee61fc1ade32b45d4a4ba1c`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（固定只读 clean clone）；serde_stacker `0.1.14` / task_ids: T-02,T-05,T-06,T-08,T-09 / evidence: [R1 修复验收与 finding 关闭矩阵](attachments/841/repair-r1-closeout.md)；原公开生成器/48层 codec 反例各 1 绿；compiler 准入 3、ui_ast 4、codec 7、runtime 2、SDK 1、外部方言 48 全绿；三仓触面 rustfmt 与 diff check 通过；三树提交后干净 / blockers: 无 work-owned findings、覆盖缺口或未决合同；统一裸 cargo t + 适用 tv/tt/tu 与三仓统一交付门仍 merge-owned pending，本轮未执行整档、未合入、未沉淀 canonical/ledger / next_responsible_role: review——按 R1 findings、修复提交和候选 SD 做受影响独立复审

2026-10-26 / stage: review / review_scope: final / reviewer_identity: 独立审查会话 Codex `/root`（未参与实施，未委派 agent，不虚构模型/session ID） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / reviewed_commit: lang `acd8d99d2eb180f4861fb235d783f19dba307a38`、UI `f7eeb82d0170f69a6a53cf0e4d625caea7eddab9`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / base_commit: lang `100172980efb0343fe7016a05eb40f18d0014d74`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、Man `1b7a9ce0de9055f7953129d78a25777662508258` / dependency_revisions: down clean clone `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，三仓 Cargo/源码/配置随冻结 commit / spec_inputs: [baseline/hashes](attachments/841/review-r1/baseline.json.txt)，[候选 SD 冻结副本](attachments/841/review-r1/spec-delta-reviewed.diff) SHA256 `9a15abe00455fd72e9aab9b11380db42702b8ee2e54ab1045f42ec502191f2a6` / acceptance_results: AC-01 pass、AC-02/03/05/06 fail、AC-04 partial/fail、AC-07 pending / findings: **F841-R1-01 P1** 生产 Rust/Vue 入口绕过 lower_ui，接受其明确拒绝的重复 widget；runtime/SDK 同型源码风险须一并接线，**F841-R1-02 P2** 合法48层视图4253B自身 codec 输出不能 decode（recursion limit exceeded），**F841-R1-03 P2** 触面 rustfmt 30文件失败 / evidence: [完整报告、反例与关闭 oracle](attachments/841/review-r1/review.md)，Core3/SDK5/codec5/lowering4/cache1/UI golden1/Man CLI7通过，777 strict=0 violations；独立确认 probe=1 pass/2 fail；外部方言 fixture 本轮因磁盘满中止，不算产品回归或通过；792旧收据按最终仅增receipt的差量复用，不覆盖新反例 / pending_closeout_gates: 全部裸cargo t与tv/tt/tu等适用档、三仓统一交付门仍merge-owned pending；本轮未执行整档/合入/沉淀/归档 / next: work同树修复全部finding、定向反例与合法邻接、格式/SD和来源刷新后提交，交受影响复审。重开T-02/T-05/T-06/T-08/T-09，保留T-01/T-03/T-04/T-07已完成产出，executing **4/12**，U841不放行。

2026-10-26 / stage: work（T-09 最终实施收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（execution_done，9/12；交独立 review）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: lang `6edb3fbae`、UI `4f6104612`、Man `1b7a9ce0d` / code_commit: lang `acd8d99d2eb180f4861fb235d783f19dba307a38`、UI `f7eeb82d0170f69a6a53cf0e4d625caea7eddab9`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（固定只读 clean clone） / task_ids: T-09 / evidence: AC-01..05 全部定向完成；[V-01..05/owner 映射](attachments/841/v07-v08-closeout.md)、[SD/842交接](attachments/841/handoff.md)、[最终792=39/39](attachments/841/v08-792-receipt.json.txt)；777 strict=0 violations；旧 UI AST/Core 兼容臂/空装配/死 import 扫描零命中；V-03 codec/cache/Weak 既有证据经最终 dialect 全套复测确认；三仓代码提交与干净工作树已核对 / blockers: 无 work-owned findings 或未决合同；统一裸 cargo t + tv/tt/tu 等 merge 门 pending，不由本记录冒充通过 / next_responsible_role: review——T-10 独立完整 AC/SD/来源/健康审查

2026-10-26 / stage: work（T-07/T-08 阶段收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（8/12；T-07/T-08 已勾选，T-09 待最终 792 收据）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}` / branch: `codex/plan-841-dev` / base_commit: lang `6edb3fbae`、UI `4f6104612`、Man `1b7a9ce0d` / code_commit: lang `338c95021e328ebb0a1787c7f1976b50d21227e6`、UI `f7eeb82d0170f69a6a53cf0e4d625caea7eddab9`、Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（固定只读 clean clone；原 detached 兄弟的无关 WIP 未纳入） / task_ids: T-07,T-08 / evidence: plan050=1、plan810=21+1 ignored、compiler doctest=19、dialect=91+2、ui_ast=4、Man CLI=7；独立 Core/SDK/AST/lex/dialect/grammar consumers 与 UI golden 全绿；777 strict=0 violations；[V-01..05 与测试 owner 映射](attachments/841/v07-v08-closeout.md)，792 首轮 39/39，最终同 SHA 复验进行中 / blockers: 无实施阻塞；统一裸 cargo t + tv/tt/tu 等 merge 门 pending / next_responsible_role: work——T-09 最终 792 收据、零残边与冻结提交核对后交 review

2026-10-26 / stage: work（T-03..T-06 阶段收口） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（6/12；T-03/T-04/T-05/T-06 已核验勾选，整体未送审）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / base_commit: lang `1fed8a306`、UI `471c7f9` / code_commit: lang `6edb3fbae7bb15b0d13118016e7d4e576dc7c526`、UI `4f6104612dd60e5c81814bb9bb6d19b7e30d0b2c` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（只读）、Man `1b7a9ce0de9055f7953129d78a25777662508258` / task_ids: T-03,T-04,T-05,T-06 / evidence: Core plan841 3、dialect codec 5、compiler ui_ast 4、`ui_cache_schema` 1、design_tokens 9 全绿；`cargo check -p auto-ui-runtime` PASS；缓存真实落盘/重载与旧 schema 失效、Weak 最后 owner、跨文件 owner 见 [V-03 证据](attachments/841/v03-closeout.md)；迁移触面警告按基线 diff 交叉仅 4 条且全清，runtime 余 216 为预存，不记作本计划迁移警告 / blockers: T-07 待迁 `auto-ui-tools` 23 个旧 Core UI AST 编译错及 auto-lang `vue_capabilities`/`component_registry_test`/`regen_page` 等旧消费点；3 个预存环境红与 aura plan050 桩仍按既有归因，merge 统一裸 `cargo t` + 触面档 pending / next_responsible_role: work——T-07 三仓调用方/manifest/固定版本/生成 import/测试 ID 选择面同步

2026-10-26 / stage: work（T-05 行为验证收口：零迁移致败） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（runtime 全量 1873 过/7 败，7 败全部归因为非迁移因素——基线对照实证）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / code_commit: UI `471c7f9`（p370 timer 携带修复——plan370_test_support 三处 store-as-child 转换原 `timer: None`，靠已退役的构造器兜底补 timer；现与生产一致携带 `store_decl.timer`，plan536 T1 三探针全绿） / evidence: **基线对照**（lang `100172980` + UI `aee499e` 双仓检出实测）：`launch_provision_lifecycle`（os error 10013 端口/Hyper-V 保留段——AGENTS.md 已载环境类）、`hatch_provisions`、`desktop_mcp_switcher`（跨线程时序 flake，两次运行败点不同）三者在 T-01 前基线**本就红**；`autocenter_translates`/`p656_scroll`/`diff_dirs_envelope` 单跑绿（序列失败=上述预存红的下游状态污染）；`aura plan050`=退役路径占位桩（T-07 oracle 重写）。**结论：迁移致败已清零** / 累计 T-05 交付：编译链全绿（runtime/SDK/workspace 除 auto-cosmic-demo 预存）+ 1873/7（含 drain-on-check 测试隔离修复与中立 external-ident 端口两根因修复） / blockers: 无迁移致败；7 败中 3 预存环境红建议按 KNOWN-DEBT 流程登记（非本计划修复面）、1 桩待 T-07、3 污染受害者随预存红修复自愈 / next_responsible_role: work——T-04 剩余（旧 Parser UI 门面 parse_widget_decl 等死代码删除+Core 拒绝面复核）→ 测试面 ~215 迁移遗留警告清理 → T-06 codec/缓存/last-owner 完整 V-03

2026-10-26 / stage: work（T-05 可编译化完成 + T-03 SDK 转换完成） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（编译链全绿：runtime/SDK/dialect/compiler core 检查零错；runtime 测试 1872/1903）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / code_commit: UI `20c0b52`、lang `1fed8a306`（含前序 WIP `a26cb66`） / evidence: ①`cargo check -p auto-ui-runtime`=0 错、`cargo check -p auto-lang`=0 错（SDK 解锁）、`cargo check --workspace` 唯一余错=auto-cosmic-demo 预存（与本计划无关）；②runtime lib 测试 1872 过/9 败/16 忽略（--test-threads=1）；③dialect 90、compiler 1121、core plan841 正反控 3 全绿 / 关键修复: ①collect_module_imports Core/StoreDecl 走查拆分 + `import_store_decls` 并行通道全穿线（15+ 调用点）；②**根因修复**：导入 style recipe 名在新端口下报 "undefined variable"（旧 core 侧 design_tokens 查询随文法表退役而断链）——core 新增中立类型化 `set_external_ident_checker(fn(&str)->bool)` 端口（name-checker 末位咨询，无 UI 数据入 core），recipe 注册表安装；38→28 失败由此修复；③VmBridge 构造改 drain-on-check（各构建只对本构建的模块解析失败负责——生产单构建语义不变，测试进程隔离恢复）；28→9 / blockers: 剩余 9 败已归因三簇——A) plan536×3+plan656+plan703_diff_dirs=store-timer 派发在通道转换下的行为缺口（下会话首目标：疑 plan370_test_support 的 timer 消费或 dynamic.rs 兜底删除的覆盖面）；B) aura plan050=已退役路径的占位桩（T-07 oracle 重写）；C) renderer/native_projector×3=环境相关（剪贴板/窗口），需单跑归因。测试面 215 警告待清（多为迁移遗留 unused） / next_responsible_role: work T-05 收尾（9 败簇 A）→ T-04 剩余（旧 Parser UI 门面 parse_widget_decl 等死代码删除）→ 警告清理

2026-10-26 / stage: work（T-05 可编译化进行中） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（T-05 WIP 已提交，83→24 错；错误全归 runtime 自身——§10 原"core 门控面"为误判，实为依赖仓警告上下文的错误归属）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / code_commit: UI `a26cb66`（WIP，信息含完整剩余清单） / evidence: ①澄清：`cargo check -p auto-ui-runtime` 的 82 错全部位于 runtime 自身文件（先前"vm/native.rs 72 错"是把依赖仓警告的 `-->` 行误计为错误；core 无门控面破损，无需 T-04 侧修复）；②已完成：dialect 依赖补齐 + 8 文件 UI 类型路径改写；dynamic_build 11 解析点全转 `parse_domain`+UiCode、单臂循环/混合 walk 大部分转换、`load_ext_imports_for_vm` 签名 `&Code`→`&UiCode`；ext_stubs 全面迁移（`collect_useweb_imports(&UiCode)`、`scan_call_arities` 泛型化 + 新 `scan_ui_code_arities`、scan_stmt UI 臂移除）；vm_bridge/dynamic/renderer/bridge/aura_view_builder/lib.rs 单点修复 / blockers: 剩余 24 错已逐行映射（WIP commit message 内）：dynamic_build 的 collect_module_imports 走查拆分 + `import_store_decls` 并行通道（654-697 及消费者 1464/1786 + dynamic.rs 698/736）、三处混合 mod_ast walk（843/872、1180/1212、1336/1359）、1412 or_else 臂、1599 根走查、416 scan 泛型调用、bridge/aura_view_builder/renderer 各 1 单臂点 / next_responsible_role: work T-05 续（按 WIP commit 剩余清单收口至 runtime check 绿，随后按序补 SDK/测试验证）

2026-10-26 / stage: work（T-03 部分单元） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **executing（T-03 未勾选；两个可验证单元已落地，SDK 语义转换被 runtime 编译破损阻断）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / code_commit: lang `ce44edf12`（T-03u1：Core 同名语义正反控 3 项永久测试——`.view` 借用、fn 体内 `msg` 普通标识符、裸 `parse()` 在 Core/UI 双场景拒绝 UI 语句〔无隐式文法装配反控〕；parser-api/parser_bridge 陈旧 UiGrammar/ui_context 文档改写为 domain 端口现实）+ `3fab3ab30`（T-03u2：SDK 安装器四处 `set_ui_grammar(ui_grammar())` 与 `UiGrammarSource` inventory 摘除，宿主能力钩子注册保持 PLAN-779 合同不变；验证=悬空引用错误 4→0 错误签名差分） / evidence: `cargo test -p auto-lang-core --lib plan841` 3 绿；`cargo check -p auto-lang-core -p auto-parser-api` PASS；auto-lex 勘定=无 UI 专属 Token 分类需迁（View/On 共享分类保持，反控覆盖）；rustfmt 触面文件通过（SDK lib.rs 被**预存**缺失 `mod mem_profile` 阻断，未触碰该声明） / blockers: T-03 剩余=SDK `ui_build` 等入口 plain-parse→parse_domain 语义转换 + 旧 Parser UI 返回门面（parser.rs 死代码 parse_widget_decl 等零调用，删除归 T-04 行）；SDK 全部验证被 runtime 无条件依赖阻断（见 §10 拓扑，T-05 owner） / next_responsible_role: work T-03 续（runtime 修复后补 SDK 语义转换与验证）或按 §10 拓扑先做 T-04/T-05 可编译化

2026-10-26 / stage: work（T-02） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（T-02 完成；Plan 保持 executing 2/12）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / base_commit: 承 T-01（lang `eccf8601e`、UI `992fbe4`） / code_commit: lang `50eb22dc9`、UI `d6d499b` / dependency_revisions: 同 T-01（down detached 只读、Man 固定版本） / task_ids: T-02 / evidence: `cargo check` lang 五包（ast/lex/parser-api/val/lang-core）PASS 且 lang 侧改动为纯 serde derive 零语义变化；`cargo test -p auto-ui-dialect` 90 单测 + 2 doctest 全绿（codec 4 测：往返+clone、坏 domain/version/tag 具名拒绝、visitor 计数、真实解析 span/源行/字段往返）；`cargo test -p auto-ui-compiler --lib` 1121 全绿（plan426 await 拒绝经领域端口保持）+ `--test design_tokens_recipe` 9 绿；dialect 包零警告（T-01 迁移遗留警告已清）；触面文件 rustfmt 通过 / 关键决策: ①design_tokens 回归因 dev-循环依赖类型分裂（dialect↔compiler）不可留在 dialect，迁 `auto-ui-compiler/tests/design_tokens_recipe.rs`（正向依赖类型同源）；②plan778 三段注册表测试改断言 builtin 恒空（内建 UI 方言已移入显式领域端口），None-回滚不变量经自定义 DomainDialect 在 `parse_domain` 复证；③vue plan426 测试改经领域端口（裸 `parse()` 不再载 UI 文法=T-01 终态设计，该红为 T-01 预存） / blockers: 无阻塞 T-02 验收，但登记两项 T-01 携带的预存破损供后续任务收口——①`auto-ui-runtime` 自 T-01 起 82 编译错（消费已删的 Core UI 臂/`ast::ui`/`resolve_use_module`，SDK `auto-lang` 因其失败；修复 owner=T-05 runtime 迁移，merge 前必须闭环）；②`auto-ui-compiler` 19 个 doctest 红（`auto_lang` crate 不可达，PLAN-814 迁移遗留；owner=T-07 测试ID/选择面同步时归因处置） / next_responsible_role: work T-03（Token 分类/文法端口/ParserBridge 迁移与 Core 同名语义正反控）

2026-10-26 / stage: work（T-01） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **pass（T-01 完成；Plan 保持 executing 1/12）** / worktree_path: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui}` / branch: `codex/plan-841-dev` / base_commit: lang `100172980efb0343fe7016a05eb40f18d0014d74`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc` / code_commit: lang `eccf8601e`、UI `992fbe4` / dependency_revisions: down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（detached 只读）、Man `1b7a9ce0de9055f7953129d78a25777662508258` / task_ids: T-01 / evidence: [contract-baseline](attachments/841/contract-baseline.md) §6；Core check、UI两包check、全部测试目标构建、代表lower→Codegen→VM=1、四生成器与跨文件导入通过 / blockers: 无；全仓 fmt 被 auto-cosmic/linux.rs 与 auto-lang/examples/ui_container.rs 预存语法问题阻断，触面包 fmt 及复核通过；merge 统一门 pending / next_responsible_role: work T-02（模型/visitor/codec与独立所有权控制）

2026-10-26 / stage: work（G841 启动门核验） / plan_id: PLAN-841 / plan_revision: 1 / outcome: **G841 满足，进入 T-01；Plan drafting→executing 0/12** / gate_evidence: U831=r3 archived 12/12，最终 review/统一门/landing/ledger v84/归档/三树 guard 清理闭环，lang delivery `d7f378b676e3a9dac7257f609936455e5d8248fe` 与 UI delivery `aee499e87093ff727f68ac8467c506e9ecf6ecdc` 均为当前 master 祖先/HEAD；U832=r3 archived 12/12，R5 收口/统一门/landing/ledger v89/归档/四树清理闭环，lang delivery `3934ed4f28e8c994e92129a44f987bc9067e9e21` 与 Man delivery `1b7a9ce0de9055f7953129d78a25777662508258` 均为当前 master 祖先/HEAD / baseline: auto-lang `100172980efb0343fe7016a05eb40f18d0014d74`、auto-ui `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、auto-man `1b7a9ce0de9055f7953129d78a25777662508258` / worktrees: `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，各仓 `codex/plan-841-dev`，根路径与分支断言通过 / observations: 语言主检出他 session 文档 WIP 及未跟踪构建产物未纳入，三仓主检出代码零修改 / next_responsible_role: work T-01（冻结来源/语法/消费图与整链可行性决策）

2026-10-24 / stage: new / plan_id: PLAN-841 / revision: 1 / outcome: blocked（上游整Plan启动门未齐；文档合同预检通过）/ next: 等G841真实满足后读回规范/交付/归档版本，再work T-01 / changed: T-01..12、AC-01..07、SD-01..05、V-01..05。本Plan drafting 0/12，无实施/fixture/probe/测试已完成，未创建树或agent；[预检](attachments/m2-governance-batch-20261024/preflight.md)。

## 10. 待澄清事项

当前调度/范围已明确，不需重复确认；T-01负责§2机制、实际三仓路径/消费者与资源策略的有界冻结，门前不得提前probe。无法保持合同或需改变长期支持/源码语义/后端/远端动作则具体needs_replan；不静默缩面、豁免源身份或把本Plan必需集成延期给后继。全部实施等前置完整交付合入/归档收尾，不以status/单阶段pass越门。

**T-02 盘点的破损拓扑（供 T-03..T-05 排期，不需用户裁定但须知晓）**：`cargo check -p auto-ui-runtime`（UI 树，组内 ../auto-lang 解析生效）= 83 错两组——①runtime 自身消费已删 Core UI 臂/`ast::ui`/`resolve_use_module`（owner=T-05）；②feature 统一点点亮的 auto-lang-core 门控面（vm/native.rs 72、ffi/engine/trans 等，owner=T-04 VM 准入或 T-05 视具体面）。`auto-ui-runtime` 是 SDK `auto-lang` 的**无条件**依赖（default-features=false 仍必选），故 SDK（含其 lib.rs 内 4 处 `set_ui_grammar(ui_grammar())` 悬空引用与 inventory UiGrammarSource 装配，owner=T-03）在 T-05 完成前任何配置都不可编译——T-03 的 SDK 侧改动只能以 core/parser-api/dialect/compiler 档验证，SDK 自身验证顺延至 T-05 后补，merge 门（AC-06/07）前必须全绿闭环。


### Merge R1：门禁失败、定向修复与受影响复审

初次统一门禁发现 UI parser 测试入口遗漏（M841-GATE-01）及两项 native timeout，未合入。修复回到 work；测试输入/断言/时限未放宽。32 项定向与两项超时串行重放通过，提交 `4a879088519716a45c700588052a4019c3b405ba`；受影响复审 pass，已明确同会话修复/复审限制。生产实现/规范未改，原 R4 未受影响证据沿用。当前 reviewed 10/12，统一门禁在新冻结提交上 pending；不以 scoped 通过代替交付。详见 [修复复审](attachments/841/merge-r1/repair-review.md)。


### Merge R2：导入错误 oracle 修复与受影响复审

第二次统一门禁 1125 pass / 1 fail；M841-GATE-02 为旧测试读取已消费的全局错误表，正式构造已返回具名错误。测试改为验证实际构造拒绝、错误字段、清空与修复后恢复；39 项定向通过，提交 `cced32bc7d1eb09a419147e7e01e49a2a8f71de0`。同会话受影响复审 pass，生产与 SD 未改，原独立 R4 保持适用；[报告](attachments/841/merge-r1/repair-review-2.md)。统一门禁新冻结重跑 pending，仍 reviewed 10/12、未合入。


### Merge R3：UI feature 条件测试迁移修复

语言 t 3183、tv 163、tt 3555 全绿后，UI t 编译发现 M841-GATE-03：仅 ui feature 下的 button helper 引用不存在的 src。已改用实际 widget_src，原输入/生成断言保留；显式 ui feature 下3项按钮控制、11项缓存控制通过。UI `7f270e51ea10a4c584a96a3eb8101e57dff35908`，Man来源脚本 `4ad2208c5e624f5c031cebf5694eec16981d7ad0`。同会话受影响复审 pass，限制与 feature 覆盖调整、源指纹变化见 [报告](attachments/841/merge-r1/repair-review-3.md)。原独立生产/SD审查保持；新冻结统一门 pending，仍 reviewed 10/12、未合入。


### Merge 环境诊断：系统保留端口

Attempt 4 UI t 编译已恢复，但默认3358位于Windows保留段3323–3422，socket控制复现10013；现有正式 AUTO_GALLERY_PROXY_PORT 配置设为54480后原生命周期定向1项通过，无代码/断言/选择面/时限修改。详见 [环境诊断](attachments/841/merge-r1/environment-port-resolution.md)。同一冻结源码在显式配置上继续统一门；仍未合入。


### Merge 最终交接：needs_fix（M841-GATE-04）

2026-10-27 / stage: merge / plan_revision: 1 / outcome: **needs_fix** / current: **executing 6/12**，T-07/T-08/T-09/T-10 重开；T-11/T-12 pending。冻结源码 lang `cced32bc7d1eb09a419147e7e01e49a2a8f71de0`、UI `7f270e51ea10a4c584a96a3eb8101e57dff35908`、Man `4ad2208c5e624f5c031cebf5694eec16981d7ad0`；down 固定只读 c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。

已完成三轮自动定向修复与受影响复审（M841-GATE-01..03 CLOSED）。Attempt 3 的语言 t 3183、tv 163、tt 3555 曾全绿；随后 UI 测试源修复改变生成器源指纹，不能冒充当前冻结输入上的最终统一全绿。系统保留端口已通过正式配置入口解决且原生命周期对照1项绿。当前 UI t 在同一冻结源码/54480配置下 **2147 pass / 1 fail**，273项因 fail-fast 未运行。

新增 **M841-GATE-04 P2 OPEN**：Plan492 未定义变量金丝雀原应到 Init 并验证 bar 命中区减少、兄弟命中区≥12；实际在导入 bar_chart.at:336:22 parse error 后构造被拒绝，尚未到原 oracle。定向重放健康1过/金丝雀1败，完整源码与hash、旧合同、已证实链路/未决根因和关闭条件见 [finding](attachments/841/merge-r1/M841-GATE-04/finding.md)。禁止删除负控、吞模块错误或把任意提前拒绝当作 Init 隔离通过。

三次自动修复上限已到，按 auto-plan-work 交还诊断，不做第四轮实现修改或在已知 blocker 上继续全档。**没有合入、没有更新 live ledger、没有归档/清理/放行 U841**。规范/导航准备仅留在实施分支，三树和恢复分支保留。下一责任 work：完成 canary 有效性/实际路径诊断，按合同修复并定向验证，再受影响复审；新冻结输入的统一门与三仓777/792/发行门仍merge-owned pending。总收据见 [merge receipt](attachments/841/merge-r1/receipt.md)。


### G04 修复后的当前交接

当前状态以本条及§9最新 work 记录为准：**execution_done 9/12**。延迟 Init 的快照与根字段已各自持有并平衡引用；原解析期非法输入及运行期故障隔离分别验证，M841-GATE-04 已实施侧修复，20项定向通过。暂无待澄清的实现语义或接受延期；下一步为受影响独立复审。没有执行整档、合入、ledger、归档或清理；三实施树和历史失败证据保留。


### R5 修复后的当前交接

以§9最新work记录为准：**execution_done 9/12**。F841-R5-01已在实施侧关闭：嵌套Array/Object快照递归持有，重播逐层物化为堆容器并平衡最后释放；真实producer三形态及取消/缺失/Drop/共享别名等27项定向通过。下一责任为受影响独立review；统一门、发行、合入、ledger、归档与清理仍pending，未宣称U841完成。


### R6 受影响复审后的当前状态

以§9最新R6记录为准：**pass / reviewed 10/12**。F841-R5-01已由复审关闭，无新增阻断；原递归producer反例、重播/最后释放及混合Init/视图消费有实际证据。三实施树clean，当前冻结lang `ecc2f54a7`、UI `bfad5149`、Man `6330ce00`；下一责任merge，统一门与三仓发行/实际交付、canonical/ledger/归档/guard仍pending。历史失败与诊断原样保留，未合入或宣称U841。
