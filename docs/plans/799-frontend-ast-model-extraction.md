---
plan_id: PLAN-799
status: drafting
feature_name: 前端 AST 模型独立与解析便利层分离
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md]
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
| [ ] | T-01 | work | 789/793归档、合同确认 | ast.rs/ast全目录/ParserExt消费者与impl可见性审计；冻结V02旧臂/真实DAG，界定解析便利新模块与共享窗口 | 01/02/04 |
| [ ] | T-02 | work | T01 | 新crates/auto-ast/Cargo.toml与src模型；搬纯模型/方法/序列化，cargo check -p auto-ast | 01/02 |
| [ ] | T-03 | work | T02 | 上层ast_parse_ext及Parser耦合测试迁位；auto-lang ast薄门面、ParserExt旧路径/六调用保持；定向V03 | 02/03 |
| [ ] | T-04 | work | T03 | 实际Parser/types/infer/codegen/trans/ui_gen引用适配，不改算法；删除旧模型，代表消费+check | 02/03 |
| [ ] | T-05 | work | T04 | 新独立consumer/V01..05、777精确domain规则及生产单一性；按合流表验792/794兼容 | 01–04 |
| [ ] | T-06 | work | T05 | 搬迁/净删/剩余边和SD/来源收据，绑定792/794实际SHA；提交冻结代码，work项闭合 | 01–04 |
| [ ] | T-07 | review | T06 | 同树独立验收全部模型/解析/消费/负控及SD；整档留merge | 05 |
| [ ] | T-08 | merge | final pass | 同步最新master，复审受影响变动、冻结门及合流交接；Spec/ledger/归档/guard清理 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认、共享写面释放后work | 0/8。无792/794/800交付硬依赖，推荐合入顺序不改变depends_on；未实施/运行Rust。

## 10. 待澄清事项

T01有界查清当前全部模型闭包和trait/内部访问；若新发现生产反向依赖必须搬Parser/推断，先具名修订，不能公开全Parser或复制AST/语法绕过。UI兼容AST仍保留，其终态退出必须另立合同。
