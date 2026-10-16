---
plan_id: PLAN-789
status: drafting
feature_name: 词法基础 crate 抽取与前端依赖切口
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md]
new_spec_components: [docs/specs/auto-lex/project.md]
touched_goals: [GOAL-003]
affects: [auto-lang/frontend, auto-lex]
depends_on: [PLAN-784, PLAN-778]
---

# [PLAN-789] 词法基础 crate 抽取与前端依赖切口

## 0. 变更摘要

将 Token/TokenKind/Pos 与 Lexer/LexerState 的单一实现抽到新建 crates/auto-lex；auto-lang 保留薄兼容出口与错误转换。同步明确 AST/type-store/inference 的剩余依赖闭包，供下一批完整 UI 文法外置使用。本计划不宣称 parser/AST 已独立。

## 1. 目标

交付可由独立 manifest 单独构建和使用的词法 crate，生产 Parser 真实消费它，原词法代码不保留第二份。非目标：删除 UI 关键字、改变 enum/Token API、迁 AST/TypeStore、开放整个 Parser、修改语法或启用实验 AC/CST 为正式前端。

## 2. 架构方案

依赖方向为 auto-lang → auto-lex → 既有轻值/诊断依赖；auto-lex 不依赖 auto-lang/auto-man/UI renderer。保留共享词法层的 View/On 等现有 token，UI 专属 AST 退出另行验收。Lexer 的局部错误置于底层或通过等价小类型映射为现有 LexerError，auto-lang 错误聚合器不被倒灌入底层。既有 ParserBridge checkpoint 保持不透明及相同恢复语义，不复制 Lexer。

## 3. 技术栈

Rust workspace crate、现有 strum/miette/thiserror/auto-val（仅确有需要时）、777 边界工具、独立 Cargo consumer。新包名 auto-lex；若冲突必须明确修订，不能占用现有无关包。

## 4. 需求分析与背景调查

取材基线 auto-lang@`3e65540cbe6ac019d210318dc93d17accf4ebcc4`；[输入指纹](attachments/extraction-batch-20261016/inputs.json) 记录现行 Specs、代码和 781/784/785/786 归档合同的 SHA256。overview 是导航快照，以相关模块 Spec 和实际源码为准。
用户 2026-10-16 授权“建立下一批实施计划”；本轮只建立合同及更新设计/路线，不实施 Rust、不创建执行树、不更改旧仓 refs、不发布/推送。之后按确认的合同进入 work；不能把本次建计划算作实施授权或完成证据。
781/784/785/786 均已归档；旧仓主分支和所有恢复载荷继续保留。787/788 为 AC/AutoDown 实验与集成线，不是本批全局前置，实施时按实际 diff 复核共用 manifest/数据类型热点。
代码事实：token.rs 主要依赖 auto-val/strum；lexer.rs 依赖 Token/Pos、AutoResult/LexerError/span_from；error.rs 又依赖 ast::Name，不能整个搬入词法 crate。ParserBridge 的 BridgeCheckpoint 持有 LexerState，不公开内部字段；Parser 还依赖 TypeStore/InferenceContext/ModuleTracker，778 不能证明完整 frontend 无环。依据 frontend/overview、Design37 A 波。
与 790 没有实现文件交集；与 791 共改 workspace members，短窗口串行编辑/合流；792 在三项合流后冻结新闭包。

## 5. 详细设计

T01 冻结 token/位置/错误映射及真实前端 DAG：列明 ast、types、infer、session、parser、bridge 与资源依赖，每一残留边有下一阶段 owner；这个决策工件不代替 T02–T05 的生产抽取。
逐字搬运 token/lexer，测试随职责迁移；只保留兼容 re-export 或错误转换薄层。保持 keyword_kind/all_keywords、Display、f-string note、Unicode 位置、peek/buffer/checkpoint 行为。底层 LexerResult 不得偷偷变成任意错误聚合器。旧 auto_lang::token 路径和错误码/标签/源位置保持可用；Lexer 私有入口不因抽取而迫使 Parser 公有化。
记录物理搬迁行数与净删除量，更新 module-boundaries 的精确归属，不加目录级豁免。独立 consumer 正常依赖闭包不得出现 auto-lang/iced/auto-man。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lex/project.md | 新增词法单一来源、错误/位置和依赖边界 | 可独立消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | 内部Lexer改为底层crate；桥面/兼容出口/未迁AST事实 | 不冒称完整前端独立 | AC-02/04 |

## 6. 测试设计

实施、独立 final review、merge 分别交接，未指定外部模型/额外 agent。所有人使用本计划同一棵实施树；本轮只给拟定路径，不创建它。V 用例由 work 建成可执行 fixture；下表是合同而非已通过结果。
开发只运行受影响定向案例及必要 cargo check；先修完全部 findings、提交并完成 final review，merge 再在冻结提交执行裸 cargo t 和触面档。cargo tf 仅主检出批量到期回归，不能在每棵树运行。无相关 Rust/Schema 改动不跑 docs_gen；UI codegen 触面用 tu，不用改金样消红。

| V / owner | AC | 输入/入口 | oracle 与覆盖界限 |
|---|---|---|---|
| V01 / work | 01 | 新建 test/fixtures/modularity/lex-consumer；直接用 auto-lex | fresh manifest cargo test 成功；normal dependency 无语言/UI/Man，不借主库rlib |
| V02 / work | 02 | 旧提交冻结的 token 序列与 error 投影；Unicode、CRLF、注释、转义、f-string note、EOF、View/On | token kind/text/Pos及错误code/span逐项一致；不是新实现生成expected |
| V03 / work | 02/03 | peek→checkpoint→多次next→restore；坏转义/未闭合字符串后错误；ParserBridge None/Err 嵌套回退 | 游标/五旗标及诊断单调性遵守778；类型身份可跨旧路径/新crate互传 |
| V04 / work | 04 | Core view参数/task on、UI widget/model/view/on、use/import和type推断的现有定向测试 | 解析/诊断与冻结旧臂一致；不靠移除keyword或修改语料过关 |

拟实施树 D:/autostack/.wt/lang-789/auto-lang，branch codex/plan-789-dev；仅一棵树。merge-owned 裸 cargo t+tv；若实改 ui_gen 再加 tu。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | auto-lex生产实现独立可构建，正常DAG无反向边，V01通过 |
| AC-02 | work | 既有token/位置/诊断/兼容类型身份与checkpoint保持，V02/03通过 |
| AC-03 | work | 旧实现物理删除，只留薄转换；边界门和坏输入控制不吞错 |
| AC-04 | work | Parser生产消费新crate、V04通过，前端剩余边/owner与搬迁去重账可追溯 |
| AC-05 | review | 独立逐项验收并冻结SD，无未授权延后 |
| AC-06 | merge | 冻结门禁通过，规范/ledger沉淀、归档及guard清理 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 实施/产出/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 784/778交付、合同确认 | 读token/lexer/error/parser依赖；冻结V02旧臂与lexer-error映射、前端DAG决策附件 | 01/02/04 |
| [ ] | T-02 | work | T01 | 新建crates/auto-lex/Cargo.toml、src/lib.rs与token/错误模块；精确依赖和旧路径re-export；cargo check -p auto-lex | 01/02 |
| [ ] | T-03 | work | T02 | 迁lexer实现/状态；auto-lang/src/lexer.rs只留必要适配，error.rs保留聚合与等价转换；cargo check -p auto-lang | 02/03 |
| [ ] | T-04 | work | T03 | Parser/bridge生产接线、旧实现去重；定向ParserBridge/Core/UI语法验证 | 02/04 |
| [ ] | T-05 | work | T04 | 建lex-consumer、V01..04完整重放；metadata/tree与777归属规则/严格门；不宽免 | 01/02/03/04 |
| [ ] | T-06 | work | T05 | 搬迁/净删账、SD素材、错误与位置映射收据；提交冻结代码，work-owned全部闭合 | 01–04 |
| [ ] | T-07 | review | T06 | 独立consumer/负控/实际删除/SD复核，绑定提交；整档门留merge | 05 |
| [ ] | T-08 | merge | final pass | t+tv及实际触面门；SD/ledger/归档；guard后清理 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=合同确认后work | 0/8。前端完整外置仍未交付；本批只冻结有界词法抽取，未运行拟定测试或创建树。

## 10. 待澄清事项

无需先批准大包名或仓库搬迁。若T01证明词法错误无法在不引入AST/推断依赖的条件下保持诊断，给出具体环和等价方案，先修订合同；不能把全部error/AST一并搬入“大common”或只交决策文档结束本计划。
