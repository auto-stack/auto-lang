---
plan_id: PLAN-793
status: drafting
feature_name: 前端诊断基础 crate 抽取与 AST 依赖切断
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md]
new_spec_components: [docs/specs/auto-diagnostics/project.md]
touched_goals: [GOAL-003]
affects: [auto-lang/frontend, auto-diagnostics]
depends_on: [PLAN-789]
---

# [PLAN-793] 前端诊断基础 crate 抽取与 AST 依赖切断

## 0. 变更摘要

把 auto-lang/src/error.rs 的通用错误/警告及诊断投影迁入新 auto-diagnostics；原 error 模块成为兼容re-export。通过现有 Name=AutoStr 的真实类型关系切断 error→ast 边，为后续 AST/parser 外置提供底层错误边界。生产管线必须使用新单一定义。

## 1. 目标

独立消费者可使用诊断/attach_source/格式化而不编译Parser/VM/UI；旧error路径和错误类型身份、code/label/help/span/警告保持。非目标：改变诊断文案、语法支持或错误阶段；不搬AST/TypeStore、不扩ParserBridge、不宣称整个前端已无环。

## 2. 架构方案

auto-lang → auto-diagnostics → auto-lex/auto-val/诊断基建；新底层不依赖auto-lang/auto-man/UI。保留789的LexerError与LexError单一定义及From映射，不复制词法错误。Name目前只是AutoStr别名，可在底层使用同一类型，不引入AST。原error.rs保留薄门面；模块专有错误转换在其所有者侧保留合法trait impl，不把route等实现迁进基础包。

## 3. 技术栈

Rust新crate、既有miette/thiserror/auto-lex/auto-val；既有诊断投影测试与独立Cargo consumer；777边界规则。

## 4. 需求分析与背景调查

取材 auto-lang@`9c773601abfdb25b80c08d06922d0ad50edc653a`；[本批输入指纹](attachments/parallel-extraction-20261016/inputs.json) 记录现行 Specs 与实际源码 SHA256。overview仅作导航，以模块Spec和真实代码为准。789/790/791均归档8/8；792为独立构建发行线，当前无交付假设。
用户要求“最好能有几个并行的计划”，本轮按该授权创建可执行合同和编排文档；未授权在本轮实施、创建agent/工作树、发布或修改旧仓refs。确认合同后各自work；本计划不依赖792完成，正式搬仓仍须满足整条路线退出条件。
787/788在途实验线不设为全局前置；实施者必须按实际diff重核类型/manifest共享面。全部实现/测试在本计划唯一组树，主检出只写计划簿记。
已读error.rs：唯一crate内导入为ast::Name；ast.rs声明pub type Name=AutoStr。error聚合包含AutoError、SyntaxError、TypeError、NameError、RuntimeError、Warning、ComptimeError与attach_source/print_error/format_error。route/discovery.rs另有From<RouteDiscoveryError> for AutoError，T01须核对迁出后trait一致性，不能只检查error.rs单文件。
主写面为error.rs和新crate；不改792的rust_ui/ui_gen/交付资源算法、不改794的native/scroll、不改795的lock/target。共享workspace/auto-lang manifest和边界规则按短窗口串行，保持兼容类型后各线源码可并行。

## 5. 详细设计

T01冻结公开类型/重导出/From实现的真实消费清单和诊断投影旧臂，检查语法、类型、名称、运行时、comptime、警告、聚合多错误及source-attached各族，不把format输出文本当唯一oracle。选定new crate依赖不得引入AST/推断。
逐职责搬运，保持AutoError和各子类型同一身份，禁止通过额外包装导致downcast或?传播变化。保留词法Raw→Msg、结构化→Lexer及source attach行为，不能重新包装成失去诊断code的字符串。SourceSpan offset/len、NamedSource及错误排序保持。
保留public旧路径和MietteError/Result等既有重导出；清除旧实现，只留下薄重导出/必要模块专有转换。记录前端剩余parser/AST/type-store/infer/session/module_cache/UI方言边，下一阶段仍需分别解耦。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-diagnostics/project.md | 单一诊断来源、类型/投影和无AST依赖合同 | 底层可消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | error改为薄门面、删除error→AST边，明确剩余环 | 不冒称全部外置 | AC-03/04 |

## 6. 测试设计

实施者未另指派，独立final review与merge分别交接，不新建或指定额外模型。下表V用例由work实现并执行，不是本轮通过证据。开发及needs_fix只做定向验收与必要cargo check；最终代码提交、final review通过后，由merge在冻结提交跑裸cargo t及实际触面档。tf仅主检出批量单实例，不随各计划并发；无Schema/生成器改动不跑docs_gen。

| Case / owner | AC | 真实入口/输入 | observable oracle / 边界 |
|---|---|---|---|
| V01 / work | 01 | 新test/fixtures/modularity/diagnostics-consumer，直接依赖auto-diagnostics | fresh manifest cargo test通过；normal/build闭包无auto-lang/Man/UI，非借rlib |
| V02 / work | 02 | 冻结旧提交各诊断族代表例：带/不带source、Unicode位置、Lex Raw/结构化、重复attach、多错误、warning | code/severity/help/labels/span/source/Display投影逐项相等；固定颜色/宽度后的format对照，不新造expected |
| V03 / work | 02/03 | 外部consumer同时通过auto_lang::error和新路径互传；真实parse/type/route错误、From及?传播 | 同一类型编译可互传；错误阶段和单调追加保持，坏输入不成功/吞错 |
| V04 / work | 03/04 | 生产Parser/VM/LSP定向诊断案例与777metadata/源码清单 | 单一实现、无AST反向边；remaining DAG收据有owner |

拟树 D:/autostack/.wt/lang-793/auto-lang，branch codex/plan-793-dev。开发定向新crate/consumer及诊断族；merge-owned裸t+tv，新crate测试；若实际触trans才加tt。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | auto-diagnostics独立构建，生产依赖无AST/VM/UI反向边，V01通过 |
| AC-02 | work | 原类型身份、诊断结构及阶段不变，V02/03通过 |
| AC-03 | work | 旧诊断实现删除、兼容出口保持，模块专有From无缺失，V03/04通过 |
| AC-04 | work | 真实消费清单/剩余前端DAG和净搬迁账可回放 |
| AC-05 | review | 独立逐项验收、反控、SD冻结且无合同内未完项 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档、guard后清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 789归档、合同确认 | error/AST Name与全仓From/消费者闭包；冻结V02投影及source/聚合错误旧臂 | 01/02/04 |
| [ ] | T-02 | work | T01 | 新crates/auto-diagnostics及精确依赖、单一错误/警告类型；cargo check新crate | 01/02 |
| [ ] | T-03 | work | T02 | 迁source/格式化/建议函数；error.rs薄重导出，保持各转换及旧路径 | 02/03 |
| [ ] | T-04 | work | T03 | 生产Parser/VM/工具定向消费、route转换与剩余DAG核对，cargo check -p auto-lang | 02/04 |
| [ ] | T-05 | work | T04 | 新外部consumer及V01..04；依赖/边界检查，不宽豁免 | 01–04 |
| [ ] | T-06 | work | T05 | 净删/搬迁、remaining边/并行合流收据、SD素材，提交冻结代码 | 01–04 |
| [ ] | T-07 | review | T06 | 独立consumer、诊断结构/负控/去重及SD复审 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+tv及实际触面门，规范/ledger/归档/guard清理；若先于792合流，通知刷新闭包 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认后work | 0/8；本次未实现、未测试Rust。

## 10. 待澄清事项

若T01发现真实TypeError载荷并非Name别名可表达，或转换出现无法保持兼容的trait约束，给具体最小环和方案后修订；不得改公开错误为String或把AST整包倒灌新crate。
