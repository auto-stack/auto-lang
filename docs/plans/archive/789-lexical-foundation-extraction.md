---
plan_id: PLAN-789
status: archived
feature_name: 词法基础 crate 抽取与前端依赖切口
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 8
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
| [x] | T-01 | work | 784/778交付、合同确认 | 读token/lexer/error/parser依赖；冻结V02旧臂与lexer-error映射、前端DAG决策附件 | 01/02/04 |
| [x] | T-02 | work | T01 | 新建crates/auto-lex/Cargo.toml、src/lib.rs与token/错误模块；精确依赖和旧路径re-export；cargo check -p auto-lex | 01/02 |
| [x] | T-03 | work | T02 | 迁lexer实现/状态；auto-lang/src/lexer.rs只留必要适配，error.rs保留聚合与等价转换；cargo check -p auto-lang | 02/03 |
| [x] | T-04 | work | T03 | Parser/bridge生产接线、旧实现去重；定向ParserBridge/Core/UI语法验证 | 02/04 |
| [x] | T-05 | work | T04 | 建lex-consumer、V01..04完整重放；metadata/tree与777归属规则/严格门；不宽免 | 01/02/03/04 |
| [x] | T-06 | work | T05 | 搬迁/净删账、SD素材、错误与位置映射收据；提交冻结代码，work-owned全部闭合 | 01–04 |
| [x] | T-07 | review | T06 | 独立consumer/负控/实际删除/SD复核，绑定提交；整档门留merge | 05 |
| [x] | T-08 | merge | final pass | t+tv及实际触面门；SD/ledger/归档；guard后清理 | 06 |

> **T-01 证据**（2026-10-16）：附件 [t01-frontend-dag.md](attachments/789/t01-frontend-dag.md)
> + [v02-golden-frozen.json](attachments/789/v02-golden-frozen.json)（34 例含 7 错误例，
> v2 SHA256 `37beb238…dc21`，由基面 e4ac72a62 旧 `crate::lexer::Lexer` 生成——四文件
> 指纹与 inputs.json 一致；v1 生成器漏写 note 元数据，v2 在临时 detached 树上由旧臂
> 全量再生，期望数据与 v1 逐字段一致，记录见 t01 附件 v2 段）。词法错误无 AST/推断环，
> §10 修订分支不触发。
>
> **T-02..T-06 证据**（2026-10-16，冻结提交 `1c9168233`，基面 `e4ac72a62`）：
> **T-02** auto-lex 建成（依赖=auto-val/strum_macros/miette/thiserror+dev serde_json；
> token.rs R100 逐字；error.rs 134 行=LexerError 五变体逐字+LexError 全集+span 助手）；
> `cargo check -p auto-lex` 零警告。
> **T-03** git-mv 搬迁（lexer.rs 2137 行仅 14 行机械重命名 AutoResult→LexResult；
> token.rs 零差异）；auto-lang/token.rs 7 行、lexer.rs 8 行纯 re-export；error.rs
> -91/+22（删本地枚举与 span 助手、加 re-export+From<LexError> 投影）；parser.rs
> 一处 `.into()` 适配；`cargo check -p auto-lang` 绿（411 警告全预存，触碰文件零警告）。
> **T-04** 生产接线经 re-export 自动达成（parser/bridge/loader 调用面零改动）；
> 去重验证=`pub enum LexerError` 全仓唯一于 auto-lex；定向档：parser::tests 179/179、
> dialect 7/7、model 15/15、widget 68/68、test_double_lexer 4/4、ui-dialect-consumer
> 26/26（ParserBridge checkpoint/None/Err 语义保持）。
> **T-05** lex-consumer fixture（唯一依赖 auto-lex，独立 workspace+根 exclude）4/4；
> V02 重放 34/34（auto-lex tests/v02_golden_replay，期望=冻结件字节副本）；V03 词法
> 半边 6/6（checkpoint/嵌套回滚/push_token/坏转义/未闭合串钉旧臂/fstr note 跨状态）+
> 类型身份/错误投影 3/3（auto-lang tests/plan789_lex_extraction：跨路径互传+E0001
> code+Msg 投影）；V04 补档 plan_088 27、task 94、plan131 17、infer 129、import 59、
> use_stmt 1 全绿；module-boundaries：lang-core 域+lang-core-no-monolith-no-man-no-tools
> 规则+out-of-root from_domains 扩展，audit 9 规则零违例、DAG auto-lang→auto-lex→
> auto-val、cargo metadata cross_check 41 边零差异。
> **T-06** 搬迁账：物理搬迁 2658 行（token 521+lexer 2137）→auto-lex；auto-lang 净删
> 2712 行（token 521→7、lexer 2137→8、error 1300→1231）；新增 auto-lex 实现 2811 行+
> 重放/身份测试 363 行+consumer 177 行+golden 2305 行；SD 素材已入库=docs/specs/
> auto-lex/project.md（SD-01）+frontend/overview.md 增量（SD-02）。merge-owned 门禁
> pending（裸 cargo t+tv，复审通过后于冻结提交执行）。
>
> **R1 修复轮证据**（2026-10-16，[review-r1 报告](attachments/789/review-r1/review.md)
> 三 findings 全闭，修复提交 `7a76aef57`+基线提交 `f3726816d`）：
> **F-789-R1-01/P1** golden 以 `git add -f` 入库（.gitignore `*.json` 精确例外，非重生
> expected；blob `a09fa2c` 在 HEAD，SHA256 与旧臂冻结件一致 `37beb238…dc21`）；测试注释
> v1 指纹残留对齐 v2；忽略文件清扫=交付路径仅 consumer 构建产物（lock/target，778
> 同例不入库）。**F-789-R1-02/P2** 锚定路径变更逐项归因（7d51845ff..刷新点：根
> Cargo.toml=785+789、auto-lang manifest=781+789、boundaries 配置与 auto-lex manifest=
> 纯 789——先前变更均系已合入复审过的合法增量）后按 777 流程在 clean 树
> `--refresh-baseline`（base_revision=`7a76aef57`，41 边，text_scan_totals 记录）；
> 正控 strict verdict=**pass** 0 violations（2 条 777 存量窄豁免保留：auto-gen/auto-man
> →auto-lang，owner man-ecosystem/man-core exit R5）；负控 `--self-test` **15/15**（含
> V-10 禁向判红矩阵）；证据 [review-r2/](attachments/789/review-r2/)。**F-789-R1-03/P2**
> consumer 测试代码（imports/tokenize_all/tokenize_all_rest）收进 `#[cfg(test)] mod
> tests`，复验**零警告** 4/4。修复后受影响面重放全绿：auto-lex 67+V02 1+V03 6+doc 1、
> plan789 3/3、consumer 4/4 零警告、strict pass（基线提交后 HEAD 复跑仍绿）、树 clean。

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=合同确认后work | 0/8。前端完整外置仍未交付；本批只冻结有界词法抽取，未运行拟定测试或创建树。

2026-10-16 stage=work | plan_id=PLAN-789 | plan_revision=1 | outcome=pass（work-owned 全闭合，ready-for-review） | worktree=`D:/autostack/.wt/lang-789/auto-lang` | branch=`codex/plan-789-dev` | base_commit=`e4ac72a62`（=inputs.json 冻结基面后继，四文件指纹一致） | code_commit=`1c9168233` | dependency_revisions=auto-down 组内兄弟 detached@`c39ac7f`（PLAN-726 T-04 基面钉 master） | task_ids=T-01..T-06 | evidence=AC-01：`cargo check -p auto-lex` 零警告+lex-consumer 独立 manifest 4/4+audit DAG auto-lang→auto-lex→auto-val 零违例；AC-02：V02 冻结 golden 重放 34/34+V03 6/6+类型身份/错误投影 3/3；AC-03：auto-lang token.rs 7 行/lexer.rs 8 行纯 re-export、`pub enum LexerError` 全仓唯一、Raw→Msg/结构化→Lexer 投影与旧臂逐字一致（坏输入控制不吞错，E0001..E0005 直达）；AC-04：parser/bridge 生产消费经 re-export 零调用面改动+定向档 parser::tests 179/dialect 7/model 15/widget 68/test_double_lexer 4/plan_088 27/task 94/plan131 17/infer 129/import 59/use_stmt 1+ui-dialect-consumer 26/26 全绿+残留边 DAG 附件（t01-frontend-dag.md 逐边 owner） | blockers=无（词法错误无 AST/推断环，§10 修订分支未触发；golden v1 note 元数据缺失已由旧臂 v2 再生修复并留档） | next_responsible_role=review（T-07：独立 consumer/负控/实际删除/SD 复核，绑定 `1c9168233`；merge-owned 裸 cargo t+tv 保持 pending）

- 就绪记录：work-owned AC-01..04 全部以当前冻结提交的可执行断言为证（见上）；
  in-contract 未决 findings/覆盖缺口=空；先例重放=V02 全量冻结臂+V03 兄弟状态
  （嵌套 checkpoint LIFO/push_token 回填/未闭合串旧行为钉住）；本轮无二次
  final review 失败史（首次送审）。
- 环境注记：master 已被并行会话推进（e4ac72a62→2f6b06c74+，790/791 在途）；
  复审/合并需按当期 master 对账（本批与 790 零文件交集；与 791 共改根
  Cargo.toml members——本批先落 `crates/auto-lex`，合流串行）。sccache 端口
  冲突（os error 10013）经独立端口 `SCCACHE_SERVER_PORT=43917` 规避——并行
  会话共用机器时的已知形态。

## 10. 待澄清事项

无需先批准大包名或仓库搬迁。若T01证明词法错误无法在不引入AST/推断依赖的条件下保持诊断，给出具体环和等价方案，先修订合同；不能把全部error/AST一并搬入“大common”或只交决策文档结束本计划。


### 2026-10-16 final review R1 — needs_fix

- stage: review；review_scope: final；reviewer_identity: 本次 Codex 独立验收上下文，未参与本计划实施，未另建 agent；不虚构模型或会话 ID。
- plan_id: PLAN-789；plan_revision: 1（合同未变）；reviewed_commit: `1c916823376f748cd04a2f736e86f9fbe7f10624`；base_commit: `e4ac72a6265c85cdadeedbabc7a0c6fd93948e48`。
- worktree_path: `D:/autostack/.wt/lang-789/auto-lang`；branch: `codex/plan-789-dev`；实施树受管内容 clean；组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。主检出已有其他会话计划簿记脏项，本轮仅追加789复审材料/债表。
- spec_inputs / frozen_delta: [输入指纹及冻结SD](attachments/789/review-r1/inputs.json)；SD-01/02内容与实现一致，影响元数据路径及GOAL-003有效；本轮不沉淀canonical或ledger。源Spec SHA256 `7c2aec6d37ed8b376e04bf1422a9b0eb5541ea4f6bb7edcb1a1fb404882e49fb`。
- acceptance_results: AC-01 pass（独立consumer 4/4、normal依赖树无auto-lang/UI/man）；AC-02 partial（本机auto-lex 67单测+34例golden+6状态+1doc通过，但V02资产未提交，不能视作冻结提交可复现）；AC-03 partial（单一实现/错误映射成立，strict边界门两红）；AC-04 partial（生产接线及逐字搬迁核对通过、桥面26/26，T05/06证据收口仍缺）；AC-05 fail；AC-06 pending/merge-owned。
- findings: F-789-R1-01（P1，V02 golden被忽略未提交）、F-789-R1-02（P2，strict基线过期/配置漂移）、F-789-R1-03（P2，consumer新增3警告）。详见[完整复审报告](attachments/789/review-r1/review.md)及[严格边界原始报告](attachments/789/review-r1-boundaries.json)。遗漏/延后扫描未见额外范围裁撤；上述合同内缺口不批准作为债延期。
- pending_closeout_gates: 裸 `cargo t` + `cargo tv` 仍归merge，在修复提交复审通过后执行；无ui_gen/aavm触面，不加tu/taa。未运行整档或docs_gen，未合并/归档。
- next: 同一实施树交回work；T-05/06重开，保留T-01..04与已有证据，current_step=4/8。提交修复后重放受影响验收，再申请final review。

- R1补充复验：上层类型/错误投影3/3；parser::tests 180/180、plan_088 27/27、ast::task 22/22、infer 130/130、task 95/95、dialect 7/7、model 23/23、import 90pass+5ignored、use_stmt 1/1、plan131 17/17、test_double_lexer 4/4。原始记录[review-r1-targeted.log](attachments/789/review-r1-targeted.log)。宽widget筛选123pass/1fail/1ignored，唯一红是旧plan051的corpus app.at在基面与当前树均缺失，未进入词法解析，记F-789-P01（非本次回归）；未据此改语料或实现。两次0项筛选不计证据。受审HEAD最终复核仍为1c916823376f748cd04a2f736e86f9fbe7f10624，受管树clean。

### 2026-10-16 R1 修复轮 — work 回交（ready-for-review R2）

- stage: work | plan_id=PLAN-789 | plan_revision=1（合同未变，R1 无 needs_replan） | outcome=pass（R1 三 findings 全闭，work-owned 重新全闭合） | worktree=`D:/autostack/.wt/lang-789/auto-lang` | branch=`codex/plan-789-dev` | base_commit=`e4ac72a62` | code_commit=`f3726816d`（修复链：`1c9168233`→`7a76aef57` F-01/F-03→`f3726816d` F-02 基线） | dependency_revisions=auto-down 兄弟 detached@`c39ac7f` 不变 | task_ids=T-05/T-06（重开后重闭合） | evidence=见"R1 修复轮证据"块（golden 入库 blob `a09fa2c`+SHA256 一致+干净检出可重放；strict 正控 pass 0 violations+负控 self-test 15/15+基线归因逐项；consumer 零警告 4/4；受影响面重放 auto-lex 67+1+6+1/plan789 3/3/strict 复跑绿，树 clean） | blockers=无 | next_responsible_role=review（R2：绑定 `f3726816d` 复核三 findings 闭合与新增面；merge-owned 裸 cargo t+tv 仍 pending）
- 修复不变量说明（repair-the-invariant）：三 finding 均为交付/证据完整性缺口而非词法语义缺陷——F-01 修"重放资产可复现性"（期望数据未重生，字节=旧臂冻结件）；F-02 修"严格门证据真实性"（归因后显式刷新，非 observe 绕开/零宽豁免）；F-03 修"新代码零警告健康面"（cfg(test) 隔离，不改测试断言）。无兄弟分支/调用点共享同因：golden 仅 auto-lex V02 消费、基线仅 audit strict 消费、consumer 为独立 fixture。


### 2026-10-16 final review R2 — pass

- stage: review | review_scope: final | reviewer_identity: 沿用R1 Codex审查上下文，未参与实施/修复，未另建agent；不虚构模型/会话ID | plan_id: PLAN-789 | plan_revision: 1 | outcome: pass。
- worktree_path: `D:/autostack/.wt/lang-789/auto-lang` | branch: `codex/plan-789-dev` | reviewed_commit: `f3726816db18d38996e3f80ae496d33719edc5e5` | base_commit: `e4ac72a6265c85cdadeedbabc7a0c6fd93948e48` | dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。结束前HEAD及依赖未变，受管树clean。
- acceptance_results: AC-01..05 pass；AC-06 pending/merge-owned。F-789-R1-01/02/03全部closed：提交内golden blob可读、40142字节SHA256与旧臂冻结件相同；本轮词法67+V02 1(34例)+V03 6+doc1通过；strict pass零违例、metadata41边一致、正负控15/15；consumer4/4且无警告。生产代码/manifest/测试配置及两锁文件未变，R1类型/错误3例、桥面26例、V04定向与物理去重证据按哈希核对复用。无新finding；F-789-P01预存资源债保持。
- spec_inputs / frozen_delta: [reviewer-inputs.json](attachments/789/review-r2/reviewer-inputs.json)及同目录SD-01/02冻结副本；SD与R1相同，主规范源hash未变，影响metadata及GOAL-003有效。未沉淀canonical/ledger。
- evidence: [R2完整报告](attachments/789/review-r2/review.md)、reviewer-lex.log、reviewer-consumer.log、reviewer-strict.json、reviewer-boundary-controls.log；格式及diff-check通过。遗漏/延后/workaround扫描无未授权缺口，新增修复没有削弱断言或规则。
- pending_closeout_gates: **裸cargo t+tv pending / merge-owned**；无ui_gen/aavm触面，不加tu/taa；本轮未运行整档或docs_gen，未合入/归档/清理。
- next: Plan=reviewed，T07完成，7/8；交merge在最终冻结提交执行门禁及规范/ledger/归档/guard收尾。reviewed不等于交付完成；合流变更受审输入时重核受影响证据。

### 2026-10-16 stage=merge — PLAN-789:r1 consolidation receipt（pass，delivered）

- prepared：reviewed 基线 f3726816d（R2 pass）→rebase 9e03287aa→c87c4e99c（range-diff 3/3 全等，安全改写证明；映射 1c9168233→00eda7741 / 7a76aef57→ed8269b41 / f3726816d→c87c4e99c）→纯投影增量 8a5b93920（spec-index GROUPS 收录 auto-lex + INDEX.md 再生 33 projects）+ 48dae3c48（基线重锚 8a5b93920——旧锚 7a76aef57 落 master 后不可达会使 strict base_known=false；重锚后 strict pass 0 violations）。投影增量仅 docs/索引/基线文件，实现/依赖/测试配置零改动。
- 门禁（merge-owned，绑定交付提交 48dae3c48）：裸 cargo t **5318/5318**（1 slow 过，1550 skipped=heavy 门控族）；cargo tv **163/163**。无 ui_gen/aavm/docs_gen 触面，不加 tu/taa/docs_gen。
- landed：主检出 `git merge --ff-only codex/plan-789-dev`，master tip == 48dae3c48（无 merge commit，线性）；canonical Spec 在主分支（docs/specs/auto-lex/project.md + frontend/overview.md 增量 + INDEX）。
- ledger_refreshed：`/api/specs` 探测 http_code=000（无 store writer）→ **guarded offline projection**（通道如实记录）：字节级备份 v30→外科脚本改写→v31；+5 条目 P789-D1/D2（designs）、P789-T1（tests）、P789-R1（reviews，verified）、P789-M1（reports）；回读校验=六 section id 精确、存量条目逐字节等价、version 恰 +1；提交随本归档簿记落库。
- archived：git mv → docs/plans/archive/789-lexical-foundation-extraction.md，status=archived（8/8）；plans.md 双索引加行；KNOWN-DEBT F-789-R1-01/02/03 由 R2 裁 closed（F-789-P01 预存资源债保留）。复审附件（attachments/789/**，含 review-r1/r2）按 781 先例留盘不落库，SHA256 已录于各报告。
- cleaned：wt-guard `D:/autostack/.wt/lang-789/auto-lang` clean → 移除 worktree+分支 codex/plan-789-dev；auto-down 兄弟 worktree（detached c39ac7f，只读消费未改）经 auto-down 仓移除；组目录 `.wt/lang-789` 删除。
- 批量回归到期判定：**未到期**——789%5=4≠0；收据 last_covered=786（2026-10-16，<48h）；786 后无 %5 计划落地（787/790/791 均未合入）。下一触发=790 落地（790%5=0）时交 /auto-plan:regress。
- 部署注记（landing≠deployment，auto-lang 无 auto-musk 三件套产物）：语言仓无运行时进程/发布二进制随本计划变动需重启者——lex 抽取为库内重组，CLI/产物行为不变（冻结门禁全绿即主分支 known-good）。
- outcome: **pass**（delivery 48dae3c48 / archive docs/plans/archive/ / ledger v31 / cleaned 全验证）。

