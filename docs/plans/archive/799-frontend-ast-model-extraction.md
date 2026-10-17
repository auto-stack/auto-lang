---
plan_id: PLAN-799
status: archived
feature_name: 前端 AST 模型独立与解析便利层分离
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-17
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/auto-ast/project.md]
touched_goals: [GOAL-003]
affects: [auto-lang/frontend, auto-ast]
depends_on: [PLAN-789, PLAN-793]
---

# [PLAN-799] 前端 AST 模型独立与解析便利层分离

## 0. 变更摘要

新建auto-ast，把AST数据、类型表达与ToNode/ToAtom/AtomWriter等模型行为抽成无Parser/VM/UI实现依赖的单一模型包。ast/parsers.rs中的ParserExt实现及依赖Parser的测试留在auto-lang上层，旧auto_lang::ast路径兼容。

## 1. 目标

交付独立可构造/检查/序列化的AST模型，真实Parser/类型推断/VM/转译/UI消费者使用同一类型身份；物理删除旧模型实现。非目标：改变文法、推断算法或AST序列化；不迁TypeStore/InferenceContext，不迁整个Parser，不实施UiAst最终分离或整仓搬迁。

## 2. 架构方案

auto-lang → auto-ast → 既有轻值/词法类型；auto-ast normal/build依赖不得包含auto-lang、Parser、TypeStore、推断、VM、renderer或Man。精确轻依赖按T01闭包冻结，不能为省事导入大包。原auto-lang::ast为薄重导出；本轮包含原UI兼容AST变体，属于无renderer依赖的共享兼容模型，不宣称纯Core AST。ParserExt trait仍归上层，其对Code/Expr/Name/Is/Arrow/OnEvents的impl移到新的上层适配模块；不把Parser注入底层。

## 3. 技术栈

Rust新workspace成员auto-ast，原AST各子模块与序列化接口；auto-lex/auto-val按实际所需使用。独立consumer、旧序列化投影、真实多消费方编译及777边界审计。

## 4. 需求分析与背景调查

取材主检出 auto-lang@`30d3e1d2a011e2a34243bb8a2b141259af721edc`；[输入指纹](attachments/model-decoupling-20261016/inputs.json)记录相关Specs、AST全目录和Man resolver代码SHA256。以Spec为现行合同、源码为事实，不以账本或旧勾选替代验收。
用户2026-10-16明确授权建立“前端模型解耦”和“Man依赖模型解耦”两份实施计划，并要求明确与792/794并行边界及合入顺序。本轮只建计划和更新设计/路线，不执行Rust、不创建agent/工作树、不发布Spec、不操作旧仓。后续确认对应合同再进入work。
789/793/795均已合入归档；本计划无792/794交付硬依赖。取材时792为execution_done（有needs_fix复审记录，实际由该线继续修复，不冒称通过），794为executing；最终实际状态须work重新核对。其他796/797/798实验线不设全局前置，只按真实写面协调。
实际ast.rs含mod parsers；ast/parsers.rs依赖Parser/ParserExt与AutoResult；部分ast内测试使用Parser和ScopeManager。生产模型外部引用主要是token::Pos，其他crate::ast引用属于模型内部递归。789已抽词法、793已抽诊断，error→AST反向边已消。Name=AutoStr，ModulePath/PathPrefix、UI声明、泛型/闭包等均有现有消费者，不能只搬几个结构体交差。
T01须穷举AST production/test依赖、inherent/trait impl与pub(crate)/私有子模块使用点，避免跨crate后孤儿规则/可见性变化才被动复制类型。当前观察不是已经证明全部AST闭包可搬的验收结果。
本计划不需要800的新API；800可继续使用旧auto_lang::ast::ModulePath，799保持其同一性。与在途工作按[并行与合流合同](reports/model-decoupling-parallel-order-20261016.md)执行：主要实现可并行；共享manifest/导出/777基线短窗口串行。若多项同时ready，推荐792→794→799→800；这不是depends_on，先完成者可以先合，后合入者承担同步与受影响复审/冻结门，不能等待792未完成的接口或扩大其职责。
新crate或source身份影响792交付闭包时，先合入者给出包/路径/接口清单，后合入者按实际已交付接口更新清单并重放相关fresh/locked/重定位与资源保护控制。792若尚未交付，只做兼容接线、不得伪造其不存在的验收证据。

## 5. 详细设计

T01先提交有界决策工件：模型/解析适配/测试三分清单、真实crate DAG、公共/内部访问面与所有需要迁位的impl。保持全部现有支持类型及方法；确有反向实现边无法等价隔离时先给具名环与修订方案，不能把目标降为只有报告或若干DTO。
T02/T03搬模型单一实现及其内聚方法；保留原树的布局、字段、关键traits和所有既有UI兼容节点，不借抽取删UI关键词/AST。模块内自引用改为新crate合法路径。跨crate必要可见性最小化并列清，不公开整个Parser/符号表/TypeStore。
解析便利层及解析耦合测试迁到auto-lang新建src/ast_parse_ext.rs（名称可等价调整，须记录路径）；保留ParserExt的旧导入路径和六类调用语义，包括OnEvents旧skip_check。不得在auto-ast dev-deps重新引入auto-lang形成隐蔽大包闭包；纯模型测试随模型，依赖解析的测试仍在上层，保留语义断言。
新/旧AST路径必须是同一Rust类型（re-export，不是重复struct/转换副本）。生产Parser、infer/types、VM codegen、trans、ui_gen不重写算法；必要引用适配范围在T01登记。行为改动、孤儿API退役、格式变化均不属于本计划。

与792的窄接线例外：若本计划后合入而新增auto-ast落入其已交付闭包，允许在792既有source清单/依赖投影处补本包及producer身份并更新相关fixture；先核对该线接口，不改交付策略、CLI协议或staging/admission/失败保护算法。若必须改算法，先具名修订协调，不能一面禁止所有792文件写入、一面把漏包责任留空。

### 规范增量

| delta_id | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ast/project.md | 新增共享兼容AST模型单一来源、无解析/执行依赖、序列化与类型身份 | 模型可独立消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | AST薄门面/解析便利层归上层；剩余Parser/type-store/UI文法边明确 | 真实进度，不冒称完整前端独立 | AC-03/04 |
| SD-03 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 792交付闭包清单窄接线：10→11 crate，PLAN-799 后含 auto-ast（§5 预授权的合流例外，非语义改写） | auto-lang 必需 path 依赖演进的事实同步 | AC-04 |

## 6. 测试设计

work、独立final review、merge分别交接；未分配额外agent或模型。各角色使用本计划同一棵实现树，以下新fixture由work创建并运行；本次建计划不算测试证据。work先完成全部修复、定向验收与必要cargo check，提交后送final review；整档门后置merge冻结提交，tf仍主检出到期批量单实例。Category A本轮禁止cargo整档/docs_gen。

| Case / owner | AC | 输入/生产入口 | oracle、边界与反控 |
|---|---|---|---|
| V01 / work | 01 | 新test/fixtures/modularity/ast-model-consumer，只依赖auto-ast | fresh manifest构建/测试；构造Core及UI兼容节点、ModulePath、Type并序列化，normal/build及本crate测试不拉Parser/VM/UI |
| V02 / work | 02 | 旧提交冻结的Core/UI、泛型/模式/任务/闭包/导入与嵌套AST的ToNode/ToAtom/Display投影 | 原字段/格式逐项一致；期望来自旧实现或既有固定oracle，不能由新实现再造golden |
| V03 / work | 02/03 | 真实Code/Expr/Name/Is/Arrow/OnEvents::parse及坏输入；ParserBridge/Core view参数/UI view/on等既有对照 | 六便利调用语义、错误/恢复保持；同一AST通过新/旧路径互传及trait方法可编译；不靠包装/复制模型 |
| V04 / work | 03/04 | 原Parser→类型/VM、trans、ui_gen代表消费，actual metadata/AST源码清单 | 生产真用新模型，无隐蔽Parser dev依赖、无第二份实现；外部UI类型互传无source双实例 |
| V05 / work | 04 | 合流时与792实际交付面比对；新auto-ast包/重导出及794record/参数类型检查 | 受影响的792fresh/locked/重定位和保护反控按合流合同重放；794八native接口不改，不要求启动未交付API |

拟树D:/autostack/.wt/lang-799/auto-lang，branch codex/plan-799-dev，一计划一棵树。定向模型、便利解析及消费测试和cargo check -p auto-lang均work-owned。AST跨VM/trans/UI生成面，merge-owned裸cargo t+tv+tt+tu；tu保持单进程。无Schema/语法参考变更不跑docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | auto-ast全选定模型独立可用，正常/构建无语言实现反向边，V01通过 |
| AC-02 | work | 旧公开路径/字段/类型身份/序列化及六解析便利入口保持，V02/03通过 |
| AC-03 | work | 模型实现单一、解析与耦合测试归上层，真实消费者V04通过 |
| AC-04 | work | 剩余DAG/consumer/搬迁与净删清单、并行合流证据及SD素材完整，V05触面闭合 |
| AC-05 | review | 独立模型/解析/消费/反控与SD审查通过，绑定提交及依赖 |
| AC-06 | merge | 冻结t+tv+tt+tu及必要合流门、Spec/ledger/归档、guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 789/793归档、合同确认 | ast.rs/ast全目录/ParserExt消费者与impl可见性审计；冻结V02旧臂/真实DAG，界定解析便利新模块与共享窗口 | 01/02/04 |
    [✅ 2026-10-17] 审计工件 attachments/799/t01-audit.md（基面143aff0a2=取材基线+0 AST diff）；三分清单：31子模块+根入auto-ast（外部依赖仅auto-val/auto-lex::Pos/serde::Serialize，95子模块测试全纯模型零Parser引用），parsers.rs六impl+markdown_tests入上层ast_parse_ext（私有mod，无外部路径消费），门面=单行glob；V02 golden冻结（36 case×3投影，SHA256 c39bdfb4…，探针attachments/799/v02-probe.rs）；窄接线面定位：rust_ui.rs DELIVERY_CRATES(792)+module-boundaries.toml lang-core(777)；无孤儿规则风险、无具名环 |
| [x] | T-02 | work | T01 | 新crates/auto-ast/Cargo.toml与src模型；搬纯模型/方法/序列化，cargo check -p auto-ast | 01/02 |
    [✅ 2026-10-17 r2修复后闭合] 逐字搬移+4类文本适配（原证见§9 r1记录）；F-799-R1-01：AutoResult回私有（r1反例本树最新rlib修复前E0659/E0308→修复后exit0）；F-799-R2-01：导入排序一行移位（8069076b7），复审同款根入口 rustfmt --check（lib.rs递归全模块树）exit0；auto-ast 74+1+5 全绿 |
    [✅ 2026-10-17] git mv 逐字搬移（33文件 rename 可追溯）：ast.rs(去mod parsers+markdown_tests)+31子模块→crates/auto-ast/src/ast；仅4类文本适配（Pos→auto_lex::token、4处doctest路径auto_lang→auto_ast、去两块迁出代码）；closure=auto-val+auto-lex+serde(frozen T-01)；cargo check -p auto-ast 零错误；cargo test -p auto-ast 74单测+5doctest全绿；新代码零警告（411条为仓库存量） |
| [x] | T-03 | work | T02 | 上层ast_parse_ext及Parser耦合测试迁位；auto-lang ast薄门面、ParserExt旧路径/六调用保持；定向V03 | 02/03 |
    [✅ 2026-10-17] src/ast_parse_ext.rs（私有mod，六ParserExt impl含OnEvents skip_check+5测试+markdown_tests 426行test/ast oracle）；门面=单行 pub use auto_ast::ast::*；cargo check -p auto-lang 零错误（全部crate::ast消费者路径零改动）；定向11测试全绿（六便利+markdown 01-06）；V02门面重放：同探针经auto_lang::ast输出与冻结golden 36/36 case逐字节一致 |
| [x] | T-04 | work | T03 | 实际Parser/types/infer/codegen/trans/ui_gen引用适配，不改算法；删除旧模型，代表消费+check | 02/03 |
    [✅ 2026-10-17] 零适配需要：门面保持全部crate::ast路径（git status 全R/RM rename，无副本无第二实现；metadata验证auto-lang→auto-ast normal边、auto-ast无auto-lang任何kind边）；代表消费定向测试：parser::tests 180/infer:: 107/vm::codegen 28/trans::gdscript 63/ui_gen::rust 83 全绿（+2预存ignored）；不改任何算法 |
| [x] | T-05 | work | T04 | 新独立consumer/V01..05、777精确domain规则及生产单一性；按合流表验792/794兼容 | 01–04 |
    [✅ 2026-10-17 r2修复后闭合] 原证见§9 r1记录（fixture/consumer/777/792driver/794）；r2复审独立重放792 driver 25/25（A/B内AST源hash=受审源）+777 strict pass+794 5/5 已核；外部ui-dialect-consumer本轮--offline 26/26补齐（fixture经显式error::AutoResult导入与ast门面面无关，修复后表面=基面等价、输入未变——不涉镜像环境恢复依赖） |
    [✅ 2026-10-17] ast-model-consumer fixture（manifest仅auto-ast，根exclude）：V01 3测试+V02 golden重放36 case全绿；crate内永久重放（日常档）+门面探针diff三路径逐字节一致；777域=lang-core，基线重采样@ea291d3ab（strict pass 0违例，2存量grandfathered）；792窄接线（DELIVERY_CRATES 10→11+spec闭包行）：auto-man delivery 9/9+generated-ui-consumer driver 25/25（fresh/locked/重定位/全负控，非复用旧收据，收据review-r2/driver-receipt.json.txt——worktree 侧副本随树清理丢失，r2 复审持久副本为准）；794 scroll-provider-consumer 5/5；ParserBridge ui-dialect-consumer 26/26；生产单一性=git全rename+metadata零auto-lang反向边 |
| [x] | T-06 | work | T05 | 搬迁/净删/剩余边和SD/来源收据，绑定792/794实际SHA；提交冻结代码，work项闭合 | 01–04 |
    [✅ 2026-10-17 r2修复后闭合] 冻结代码链 ea291d3ab→d35d4c084→4194fccc9→8069076b7（修复链全序）；SD素材含r1口径修正（auto-ast/project.md导出面以无撞名为限）+SD-03元数据补全（r2已核验）；收据attachments/799/{t01-audit,t06-receipts§3-r1口径,v02-*}+review-r1/r2材料（792收据=review-r2/driver-receipt.json.txt）；merge-owned pending显式不变 |
    [✅ 2026-10-17] 冻结代码=ea291d3ab（33文件git-mv零复制，文本适配仅4类：Pos导入/4 doctest/迁出块/同源re-export），基线chore=d35d4c084；792绑定delivery 80293e2c1、794绑定5ddef1315；SD素材：auto-ast/project.md（新）+frontend/overview.md PLAN-799段+闭包行附带；收据attachments/799/{t01-audit.md,t06-receipts.md,v02-probe.rs,v02-golden.txt}+792收据=review-r2/driver-receipt.json.txt（worktree侧replay副本随树清理丢失，r2复审持久副本为准——post-merge-audit后补注）；表面增量披露（同源re-export六原语+Pos）在t06-receipts §3供review审；merge-owned pending显式：裸t+tv/tt/tu、tf到期判定、Spec/ledger/归档、wt-guard |
| [x] | T-07 | review | T06 | 同树独立验收全部模型/解析/消费/负控及SD；整档留merge | 05 |
| [x] | T-08 | merge | final pass | 同步最新master，复审受影响变动、冻结门及合流交接；Spec/ledger/归档/guard清理 | 06 |
    [✅ 2026-10-17 merge 收口] 同步：两段 rebase over master@800（代码线，range-diff 4/4 全等，映射 ea291d3ab→e278855d9/d35d4c084→3e958144e/4194fccc9→0bafa81fe/8069076b7→3e7dd87fe）+ over 802/788 纯簿记（6/6 全等，零代码/配置面，冻结门输入不变）；基线 merge 期再采样含 auto-package-resolver 边（56a3fc000，strict pass 0 违例）；冻结门禁：裸 t 5261/5261 + tv 163/163 + tt 5633/5633（三次负载 flake：clipboard_image/plan492 编译链/plan498+499 图表族，各 solo 秒绿归因后整档过——plan484_024 同案先例）+ tu 867+2 ignored + auto-man delivery 9/9；delivery 9b510f79a ff-only（master tip=delivery）；主检出冒烟 check 零错误+auto-ast 80 测试绿；沉淀：SD-01..03 随分支入库+auto-ast/plans.md+GROUPS+INDEX 再生 39 项目+ledger v39→v40（P799-D1/T/R3/M，guarded offline projection 备份 v39-799，回填 f2c78c207）；归档+cleaned 见 §9 收口记录 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认、共享写面释放后work | 0/8。无792/794/800交付硬依赖，推荐合入顺序不改变depends_on；未实施/运行Rust。

2026-10-17 stage=work | PLAN-799 | plan_revision=1 | outcome=pass | worktree=D:/autostack/.wt/lang-799/auto-lang | branch=plan-799-dev | base=143aff0a2 | code=ea291d3ab（+chore d35d4c084 基线重采样） | deps=792@80293e2c1、794@5ddef1315、auto-val/auto-lex/auto-diagnostics@master | tasks=T-01..T-06 6/6 | evidence=AC01:独立consumer 3+1测试全绿、metadata闭包auto-val/lex/serde零反向边；AC02:V02 golden（旧实现生成SHA256 1e74a8df…）三路径36/36逐字节一致+六便利11测试+markdown oracle；AC03:全rename零副本+代表消费5族431测试绿；AC04:777 strict pass+792 driver 25/25+794 5/5+收据/SD素材齐 | blockers=无 | next=review（T-07同树独立验收；整档门merge-owned pending）

2026-10-17 stage=review | PLAN-799 | r1 | outcome=needs_fix | 受审=d35d4c084/实现ea291d3ab | F799-R1-01/P2：ast::AutoResult（auto_val版）公开再导出经门面glob泄漏，旧组合 ast::*+error::* 中 AutoResult E0659 歧义+E0308（两个不同item，"同源不歧义"论证只对同一item成立）；AC02 缺口 | 报告 attachments/799/review-r1/（REVIEW.md+baseline.json.txt+glob-compat.rs+parse-identity.rs+spec-delta.patch）| 其余面 pass；AC04 要求 metadata 补 generated-ui-distribution.md 目标 | 整档门仍 merge-owned pending

2026-10-17 stage=work | PLAN-799 | plan_revision=1 | outcome=pass（r1修复后再送审） | worktree=D:/autostack/.wt/lang-799/auto-lang | branch=plan-799-dev | base=143aff0a2 | code=4194fccc9（修复链：ea291d3ab→d35d4c084→4194fccc9） | F-799-R1-01闭合：finding→不变量（ast门面glob不得引入与其他常用glob面不同item同名导出）→受影响路径（7名全集×error/token/lexer/parser/vm审计，仅AutoResult撞不同item）→修复（回私有use；保留集{AutoStr,AutoNode,Op,Shared,Value,Pos}无撞名，Pos与token同一item）→永久用例（tests/ast_facade_glob_compat.rs反例固化编译断言+Pos双路径互传；tests/ast_cross_path_identity.rs parse-identity探针Cargo化：双向互传/坏输入/双路径投影一致）→复现与复绿（复审反例本树最新rlib修复前E0659/E0308→修复后exit0；重放：check三crate零错、auto-ast 74+1+5、fixture 3+1、ast_parse_ext 11、parser::tests 180、新测试零警告fmt clean）→剩余gap=无 | SD/收据同步：auto-ast/project.md导出面限缩口径+t06-receipts §3 r1口径 | metadata补全：frontmatter+SD表增SD-03（generated-ui-distribution.md闭包行） | blockers=无 | next=review（T-07 r2；merge期须同步已落地的800新master基线并复审受影响面）

2026-10-17 stage=review | review_scope=final | reviewer=当前 Codex 独立审查上下文（无实施历史；未暴露模型/session ID） | PLAN-799 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-799/auto-lang | branch=plan-799-dev | reviewed_commit=d35d4c0846d112ea661cb90fb950dff8c00d56d9 | base_commit=143aff0a258fdca8bda9a3b8068dc8a9cced623c | dependency_revisions/spec_inputs=attachments/799/review-r1/baseline.json | acceptance_results=AC01/03 pass、AC02 fail、AC04 partial、AC05 fail、AC06 pending | findings=F799-R1-01 P2：公开auto_val::AutoResult经旧ast门面导出，使既有ast::*+error::*调用E0659/E0308；显式导入控制组通过 | evidence=[R1审核](attachments/799/review-r1/REVIEW.md)、glob-compat.rs、parse-identity.rs；模型80/80、独立consumer4/4、解析11/11、生产消费者461通过+2ignored、check/格式/777通过 | SD冻结=spec-delta.patch SHA256 5c355fde73c3809ab4e944154132327378ab982f9fdc208bd0133712189ab812 | pending_closeout_gates=merge-owned裸t+tv+tt+tu/合流门/tf到期/Spec-ledger/归档/guard；本轮V05完整独立重放仍pending | next=原树work修F01、永久兼容反例及SD元数据后提交复审。T02/03/05/06重开，T01/04保留完成，current_step=2/8；先前work勾选/收据留作历史，不表示当前已通过。

2026-10-17 stage=review | review_scope=final | round=R2 | reviewer=本聊天Codex独立审查上下文（未暴露model/session ID） | plan_id=PLAN-799 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-799/auto-lang | branch=plan-799-dev | reviewed_commit=4194fccc98f5c945288cceb030afd283726e1a8e | base_commit=143aff0a258fdca8bda9a3b8068dc8a9cced623c | dependency_revisions/spec_inputs=[baseline](attachments/799/review-r2/baseline.json.txt) | acceptance_results=AC01/03/04 pass、AC02 partial（ui-dialect外部oracle镜像阻断）、AC05 fail、AC06 pending | finding_closure=F799-R1-01 closed（旧反例exit0+永久3测试+组合glob编译通过）；F799-R2-01 P2 open（ast.rs导入顺序导致根入口rustfmt --check exit1） | evidence=[R2报告](attachments/799/review-r2/REVIEW.md)、format.log.txt、原始/永久回归日志；模型74+1+5、独立consumer3+1、解析11、794consumer5通过；792正式driver独立25/25（fresh+locked重定位+全部保护负控，A/B内AST源hash=受审源）、777 strict pass；三crate check通过 | SD冻结=spec-delta.patch SHA256 62834ddfc34b92d650272c0f755396784a1eafd267fbd452605287cf282cab6f；SD03元数据已补齐 | pending_closeout_gates=merge-owned裸t+tv+tt+tu/同步master受影响审查与合流门/tf到期/Spec-ledger/归档/guard | next=原树work按同一根入口formatter修F02并提交、恢复或举证复用外部ui-dialect必验oracle后复审；两轮final非pass，下次handoff简述根模块fmt漏检原因与覆盖调整。T03兼容修复验收恢复完成，T02/05/06仍待闭合；T01/03/04=3/8；未修改实现或合入。

2026-10-17 stage=work | PLAN-799 | plan_revision=1 | outcome=pass（r2修复后再送审） | worktree=D:/autostack/.wt/lang-799/auto-lang | branch=plan-799-dev | base=143aff0a2 | code=8069076b7（修复链：ea291d3ab→d35d4c084→4194fccc9→8069076b7） | F-799-R2-01闭合：finding→不变量（被修改模块必须过crate级格式门，非仅新增文件）→受影响面（仅ast.rs导入块排序，一行移位零语义）→修复（cargo fmt -p auto-ast）→oracle确认（复审同款根入口命令 rustfmt --check --edition 2021 auto-ast/src/lib.rs + ast_parse_ext.rs + 两永久测试 → exit0，lib.rs递归覆盖全模块树）→剩余gap=无 | 根模块fmt漏检根因（复审要求简述）：R1修复时只对新增的两个测试文件跑了rustfmt --check，未对被修改的crates/auto-ast/src/ast.rs重跑crate级格式门；R1拆分私有/公开导入改变了排序约束而漏检——覆盖调整已内化为：任何源码改动的定向验证必须含受影响包的crate级fmt根入口命令 | 外部ui-dialect V03 oracle补齐：--offline 26/26（显式error::AutoResult导入与ast门面无关+修复后表面=基面等价+输入未变，镜像环境不构成依赖） | 重放：auto-ast 74+1+5、永久回归1+2全绿；树净 | T-02/T-05/T-06按r1+r2累计证据重勾（含两轮findings闭合），current_step=6/8 | blockers=无 | next=review（T-07 r3；AC02外部oracle已补齐）

2026-10-17 stage=review | review_scope=final | round=R3 | reviewer=本聊天Codex独立审查上下文（model/session ID未暴露） | plan_id=PLAN-799 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-799/auto-lang | branch=plan-799-dev | reviewed_commit=8069076b7c1b2dcba43bebb794ba9f901be1b6d2 | base_commit=143aff0a258fdca8bda9a3b8068dc8a9cced623c | dependency_revisions/spec_inputs=[baseline](attachments/799/review-r3/baseline.json.txt) | acceptance_results=AC01..05 pass、AC06 merge-owned pending | findings=F799-R1-01 closed、F799-R2-01 closed；无新增 | evidence=[R3报告](attachments/799/review-r3/REVIEW.md)；根入口递归fmt exit0、diff-check exit0、永久回归3/3、外部UI方言26/26（锁文件SHA未改）、模型74+1+5全通过；R2其余定向/792driver25/25/794/777证据按报告逐项给出不受一行导入排序影响的复用理由 | SD冻结=spec-delta.patch SHA256 62834ddfc34b92d650272c0f755396784a1eafd267fbd452605287cf282cab6f；三个目标impact metadata齐 | pending_closeout_gates=同步最新master受影响复审、冻结裸t+tv+tt+tu及必要合流门、tf到期判定、Spec/ledger/归档/guard | next=merge；status=reviewed、T07完成、7/8，未合入，门通过前不得宣称交付完成。

### §9.5 合入收口记录（merge）

2026-10-17 stage=merge | PLAN-799 | plan_revision=1 | outcome=pass | delivery=9b510f79a（master tip=delivery，ff-only 零 merge commit） | 映射=两段 rebase（reviewed 8069076b7→3e7dd87fe） | checkpoints: prepared=SD01..03+plans.md+GROUPS+INDEX+ledger 预备全在 9b510f79a 内 | landed=ff-only ✓（802/788 簿记插入后二次 rebase 6/6 全等再落地） | ledger_refreshed=v40 guarded offline projection（本会话 Regime B：无 store writer 可达；备份 v39-799、范围守卫 +50/-1、JSON/字段集回读校验、delivery 回填 f2c78c207） | archived=archive/799-frontend-ast-model-extraction.md（本文件，含 review-r1/r2/r3 附件一并入库；0d8a32e9c） | cleaned=✓（双树 wt-guard clean → auto-lang/auto-down 工作树+plan-799-dev 分支（was 9b510f79a merged）+组目录 .wt/lang-799 零残留移除；终态 8/8 archived；批量回归不到期：799%5=4、收据 covered=800 当日新鲜） | 冻结门=裸 t 5261+tv 163+tt 5633+tu 867+delivery 9/9 全绿（tt 三负载 flake solo 归因记录在案） | tf 批量判定=不到期（799%5=4；收据 last_covered=800 当日新鲜） | 部署注记=auto-lang 无运行期常驻产物；生产 CLI/发布二进制属发布流程非本合并窗口（观察项：target/release 产物若存在则先于本 delivery，下次发布构建自然刷新）

### §9.6 交付后收口补正（post-merge-audit 两项）

2026-10-17 stage=merge-closeout-repair | F799-CLOSEOUT-01 closed：旧活动路径 docs/plans/799-frontend-ast-model-extraction.md 的删除此前未暂存（首笔归档提交 reset 重做时丢失 rename 的删除半边），HEAD 一度同时含 drafting 旧版与 archived 终态——本提交补齐删除，HEAD 唯一终态=archive/ 副本。F799-CLOSEOUT-02 closed：review-r1/r2/r3 的 baseline.json 与 r2 的 792 driver-receipt/driver.log/check-*/format/regressions/ui-dialect/model 日志共 19 项被 .gitignore（*.json:161 / *.log:44）静默排除未入库——按仓内既有 .json.txt/.log.txt 惯例改名入库并同步全部引用（含本文件与 REVIEW.md 路径级更新，事实零改动）；worktree 侧 replay-792-driver-receipt.json 副本随树清理丢失，权威收据=r2 复审持久副本 review-r2/driver-receipt.json.txt（driver.log.txt 同在）。核查报告 attachments/799/post-merge-audit.md 一并入库。代码/规范/ledger 零改动（核查确认已合 AST 代码与通过版本一致）。

## 10. 待澄清事项

T01有界查清当前全部模型闭包和trait/内部访问；若新发现生产反向依赖必须搬Parser/推断，先具名修订，不能公开全Parser或复制AST/语法绕过。UI兼容AST仍保留，其终态退出必须另立合同。
