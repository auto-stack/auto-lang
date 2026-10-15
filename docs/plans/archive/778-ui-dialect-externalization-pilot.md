---
plan_id: PLAN-778
status: archived
feature_name: 外部 UI 方言与 ParserBridge 可行性探针
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-15
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md]
new_spec_components: []
touched_goals: [GOAL-007]
affects: [docs/specs/auto-lang/frontend/overview.md]
---

# [PLAN-778] 外部 UI 方言与 ParserBridge 可行性探针

## 0. 变更摘要

补Dialect最小外部parser接口、显式注册与错误退出恢复。独立Rust consumer实现真实model文法子集证明可用，默认内建UI兼容路径保留。

## 1. 目标

外部crate不访问pub(crate)字段即可解析 `model { count int = 1 }` 产出typed ModelBlock；session更新/指定嵌套/错误恢复不丢注册。非目标：全UI文法迁移、移除UI AST、改HIR/全部tokens/动态ABI/renderer/新仓。

## 2. 架构方案

继续Dialect返回core Stmt的兼容路线。ParserBridge提供游标/checkpoint/core expr/type/context/diagnostics最小面。外部fixture仅有限model子集；它不是第二份长期UI grammar真值，后续完整迁移时收编。默认UiDialect暂留；显式替换/追加优先规则由T01冻结。

## 3. 技术栈

Rust traits/session/parser/AST，独立fixture manifest显式脱离root workspace或受控exclude。不是动态库Rust ABI插件。

## 4. 需求分析与背景调查

用户在2026-10-13授权调查与报告/规划；本轮不执行重构、不创建新仓、不发布/合入。实施需确认本Plan revision 1。其他在途Plan/WIP保留。

前置777；Specs frontend/ui，Design37；hash基线 reports/v06-modularization-baseline.jsonl。实锚 `dialect.rs::Dialect`、`dialect/ui.rs::UiDialect`、`parser.rs::with_session/build_dialects/try_dialect_stmt/try_token_dialect_stmt/parse_model_block/parse_model_block_inner/parse_model_field_annotations`。pub(crate)状态与mem::take后?提前退出是实需验证点。

## 5. 详细设计

T01冻结一页最小bridge/dispatch/恢复决策，不公开整个Parser。拟新 `src/parser_bridge.rs` 与 `test/fixtures/modularity/ui-dialect-consumer/`；外部实现用bridge解析model字段类型/expr，可选择显式覆盖内建model处理，不能被内建先吃掉导致probe没实际运行。

本pilot支持明确的小子集，注解/widget等走现有实现并列为未迁；不声称其等价。with_session/导入/嵌套创建点逐一记录注册传播范围。Ok(None)不吞token（或已声明且执行rollback），Err恢复registry/context；冲突确定处理。Core view参数模式、task on、普通变量Store和Godot/AutoData必须不变。

### T-01 冻结矩阵（2026-10-15 work 核读后冻结）

实需验证点已证实：`try_dialect_stmt`/`try_token_dialect_stmt` 的 `mem::take` 整表移出 + `?` 提前返回在 Err 路径**永久丢失方言注册**（parser.rs:627-659）；Lexer 已有 `save_state/restore_state`（Plan 395 前瞻基建，含 chars 迭代器+计数器+缓冲 token 完整快照），checkpoint 无需新造轮子。冻结决策（等价接口机制范围内）：

1. **Bridge 面**：新 `src/parser_bridge.rs`，`pub struct ParserBridge<'p,'a>` 私持 `&'p mut Parser<'a>`。pub 方法 = 游标（`cur_kind/cur_text/cur_pos/is_kind/advance/expect/expect_keyword/skip_empty_lines`）+ checkpoint（`checkpoint()/restore()`，封装 lexer save_state + cur/prev 回填）+ core expr/type（`parse_type/parse_expr` 委托）+ context（`scenario/is_ui_scenario/in_on_body/in_fn_body`）+ diagnostics（`syntax_error` 工厂按当前 pos 造 SyntaxError::Generic、`push_warning`）。`pub(crate) fn parser()` escape hatch 仅供 crate 内 UiDialect，外部不可见——**不公开 Parser/lexer/scope/db/type_store**。
2. **Dialect trait 签名**：两方法改收 `&mut ParserBridge`（matches/keywords 不变）。外部实现只见 bridge（AC-01）；UiDialect 经 escape hatch 行为零变化。
3. **注册与优先级**：`pub enum DialectPriority { Override, Append }`；`Parser::register_dialect(&mut self, d, priority)` + builder `with_dialect(d, priority)`。存储三段 `dialects_pre/dialects_builtin/dialects_post`（pub 字段 `dialects` 移除，in-tree 仅 parser.rs 派发+3 测试断言同步迁移）；`with_session` 只重建 builtin 段，外部注册跨 with_session 保留（V-03）。派发序 pre→builtin→post，首个 keywords 命中且 Some 者胜；Override=外部覆盖内建（内建退为 None 后备），冲突可区分（V-05 反控）。
4. **恢复不变量（核心保证，非方言纪律）**：Err 不丢注册——派发改按 index 临时移出单条+显式 match（无 `?`），Err 路径同样移回；Ok(None) 不吞 token——派发器调用前 checkpoint、None 后强制 restore。方言内部投机解析同用 checkpoint。
5. **fixture**：`test/fixtures/modularity/ui-dialect-consumer/` 独立 manifest（path 依赖 `../../../crates/auto-lang`），根 Cargo.toml `exclude` 追加，不入 members。
6. **外部 model 子集冻结**：`model { [var] name [type] [= expr] }`（无注解）；**类型与初值为独立可选项、可同时省略**（裸字段 `name` 单独成字段，终止符=行界/块界/EOF，→ ty=Unknown + init=Nil；R2 F778-R2-01 闭合显式化）。内建 parse_model_block_inner **不做**裸字段（`=` 缺省仍必 parse_type）——内建路径行为保持，该差异属外部子集合同面而非默认文法漂移（控制测试钉住内建拒绝）。遇 `#[` 注解→外部 `Ok(None)`（checkpoint 已回滚）交内建处理，注解/var 语义列入未迁清单。widget/msg/component/actions/style/view/on/store 全部未迁（内建不变）。
7. **默认 grammar 零变化**：Core 场景空方言表、UiDialect 全关键字保持；无需公开全部 lexer/符号表私有面（bridge 面足以实现子集，不触发 §10 needs_replan）。

**R1 修订（2026-10-15，复审 needs_fix 后按等价机制更新冻结说明）**：
- 第 3/4 条修订：方言表元素改 **Arc 共享句柄**——派发时克隆调用、永不移出表
  （替代 remove/insert）。活跃方言在自身委托核心解析（闭包体再入语句派发）期间
  仍可见，合法递归派发见到完整方言表（F778-R1-01）；Err/None 结构上不可能丢注册。
- 第 4 条扩展：Err 恢复不变量含 **context 恢复**——BridgeCheckpoint 快照
  in_on_body/in_fn_body/in_dot_rhs/with_header/body_depth；派发错误边界恢复
  旗标（游标留失败位置，错误位置不变量）；核心 fn 体/watch 体/on-handler 体/
  dot-RHS 四处 `?` 提前退出泄漏位点改为 Err 平衡恢复（根源修复，内建路径同受益）。
- §T-05 限制 5 相应更新（checkpoint 含 context 旗标；errors/warnings 仍单调追加）。

### T-05 报告：已迁面 / 未迁清单 / 限制 / 后续迁移清单（2026-10-15 work）

**已外部验证面（外部 crate 仅凭公开面实现）**：`model { [var] name [type] [= expr] }`
子集（类型/初值经 bridge 委托核心文法；**R2 起含裸字段——类型/初值同时省略，
四象限组合矩阵见 r2_bare_field_boundary.rs**）；typed `Stmt::ModelBlock` 构造；显式
Override/Append 注册；跨 with_session / 嵌套 block 体 / 委托递归（R1）/ 实例隔离 /
Err 恢复（含 context 旗标，R2 起 pre-restore 更强 oracle）/ None 回滚 / 冲突区分
全部有正反控断言。

**未迁清单（继续走内建 UiDialect，外部 None 让行回退）**：widget / msg /
component / actions / style 关键字；view/on token 侧；store；model 注解
（#[primary]/#[Consume]/#[Provide]/#[NavParam]）；widget 体内部结构（见限制 4）。

**限制（pilot 边界，均非缺陷静默）**：
1. bridge 无通用 warning 出口（`Warning` 枚举无通用变体，不为此扩枚举）——外部诊断统一 Err 路径，由核心 errors/error_limit/synchronize 收纳。
2. 注册为每 Parser 实例显式：生产 UI 入口 18 处（lib.rs）与导入子解析器（compile.rs `parse_module_to_type_store`/`compile_module_to_bytecode`，Core 无 session）不自动继承；生产注入需逐入口 `with_dialect` 或 revision 引入 session 携带注册表。
3. `matches()` 场景门控现在是派发时核心契约（此前仅靠 build_dialects 注册时机巧合满足，外部显式注册下原样会泄漏到非属场景——本 pilot 修复并回归）。
4. 方言派发只覆盖语句位置（parse_stmt_inner）；widget 体循环直连 `parse_model_block_inner`（parser.rs parse_widget_decl），不经方言表——冻结传播边界。
5. BridgeCheckpoint 回滚游标 + 语句级 context 旗标（R1 扩展：in_on_body/in_fn_body/in_dot_rhs/with_header/body_depth）；errors/warnings 单调追加不回滚（与 Plan 395 同取舍）。
6. 块内错误 + 游离 `}` 的 synchronize 级联到 error_limit 为既有解析器恢复行为（内建同形态同产物），未纳入本 pilot 断言面（v04 测试注释记录）。
7. fixture 独立 workspace 冷构建 auto-lang，不在日常档；门禁命令独立执行。

**后续文法迁移清单（完整外部化路线，非本 pilot）**：① model 注解子集补齐 →
② widget/msg 等 Ident 关键字逐一 Override 并行对拍 → ③ view/on token 侧经
try_parse_token_stmt 验证 → ④ 生产入口接线（需 revision）→ ⑤ widget 内部结构
解析方言化改造 → ⑥ UI AST/消费者迁移（Design 32/37 拆仓目标，重大 revision）。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/frontend/overview.md | 内建派发→新增实际external注册/bridge支持面与限制，UI AST仍兼容 | 外部证据 | AC-01/02/03 |

SD-01 拟沉淀要点（merge 落 specs 时采用，work 不直接改 live ledger）：
- frontend/overview 增"方言外部化（PLAN-778 pilot）"小节：`Dialect` trait 以
  `ParserBridge` 为解析入口（游标/checkpoint/core expr/type/context/diagnostics
  最小面，不含 Parser/lexer/符号表）；`Parser::register_dialect/with_dialect`
  显式注册（Override 前插/Append 后接，跨 with_session 保留）；派发核心保证
  Err 不丢注册、Ok(None) 强制游标回滚、matches() 派发时场景门控。
- 声明当前外部已验证面仅 model 冻结子集；widget/msg/view/on 等仍内建（未迁
  清单+边界见 plan §T-05 报告）；UI AST/consumer 兼容不变。

## 6. 测试设计

实现者未指派；work自检后交独立上下文review，在同一唯一worktree逐项验收，不修代码；merge仅负责已验收的知识/归档/清理。本草案未派遣agent/chat。

T03创建外部入口 `cargo test --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml`；worktree `cargo check -p auto-lang`+裸`cargo t`+`cargo tv`。若改UI codegen追加tu；没触及trans/aavm不要求tt/taa/tf。fixture是拟新，当前未运行。

| V | AC/不变量 | 输入/入口 | oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/external生效 | 外部model + `model { count int = 1 }` | count/int/init=1；不调用core parse_model_block替代自身文法 | 冻结小子集 |
| V-02 | AC-02/Core | `model = 1`、`fn f(view x int)`、task on、既有Godot fixture | 旧行为/诊断保持，不吞变量 | 已支持输入 |
| V-03 | AC-02/持有 | 注册后with_session、嵌套core body、两个Parser实例 | 正确实例注册生效/不串线，禁用场景明确回退 | 明确传播路径 |
| V-04 | AC-02/恢复 | Err/None后同实例到明确恢复点继续合法语句 | registry/context仍在，None不消费token，错误位置正确 | harness建立恢复点 |
| V-05 | AC-03/生产 | 默认UI/Core corpus+冲突反控 | AST/诊断/VM golden无新增漂移，冲突可区分 | 不等完整拆仓 |

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | external真实typed结果；bridge不暴露内部字段；V01 |
| AC-02 | work | None/Err/session/上下文/冲突规则和V02–04实断言 |
| AC-03 | work | 默认兼容/gates无新增红；未迁范围明确；V05 |
| AC-04 | review | 独立AC/API/遗漏审计，bindcommit/revision |
| AC-05 | merge | SD01按pilot真实面沉淀、归档、guard-clean清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 777+确认r1 | 核读私有能力/grammar；冻结bridge/dispatch/恢复支持矩阵 → §T-01冻结矩阵（mem::take丢表bug证实、Lexer save_state复用、三段注册/优先级、Err/None核心保证）；[✅ 已完成] | 01/02 |
| [x] | T-02 | work | T01 | parser/dialect/新bridge，显式注册保持与错误恢复 → `parser_bridge.rs`（游标/checkpoint/expr/type/context/syntax_error + pub(crate) escape hatch）；trait 改收 ParserBridge；三段注册 `register_dialect/with_dialect`（Override/Append）+ `with_session` 只重建 builtin 段；派发重写（Err 移回不丢表 + None 强制回滚 + **matches() 派发时门控**——实需验证点证实并修复：旧实现从未调用 matches，靠 build_dialects 注册时机巧合）；crate 内 4 条永久回归 `test_plan778_*` 4/4 绿；**R1 修复（a8a81aad4）**：方言表 Arc 共享句柄（F778-R1-01 委托递归期间活跃方言留表）+ 派发 Err 边界 context 恢复 + fn/on/watch/dot-RHS 四处根源 `?` 泄漏平衡修复（F778-R1-02）；[✅ 已完成] | 01/02 |
| [x] | T-03 | work | T02 | 独立fixture真实model子集，V01–04正反控 → `test/fixtures/modularity/ui-dialect-consumer/`（独立 workspace + 根 exclude）12/12 绿（V01 typed结果+非内建替跑 / V02 core赋值·view参数·scene·widget全形态 / V03 with_session保留·嵌套block传播·实例隔离 / V04 Err后注册在+恢复点继续+错误位置offset8+注解None回滚 / V05 默认路径+Override/Append冲突可区分）；**R1 组合控制（r1_delegation_combinations.rs）6/6**：嵌套表达式委托递归 accepts=2、兄弟裸块控制、委托 Err 恢复点 context、成功委托正控、token None 回滚、token Err 再派发；合计 fixture 18/18；**R2 闭合（da17a3d44）**：裸字段支持 + 四象限边界矩阵 8 项收编（r2_bare_field_boundary.rs），fixture 合计 26/26；审查 boundary 3/3、independent 7/7（pinned）全绿 | 01/02 |
| [x] | T-04 | work | T03 | 默认内建UI保持，V02/V05；点检session/import创建路径 → `cargo t parser::tests` 179/179、`t dialect` 7/7、`t model` 15/15、`t widget` 68/68；创建路径记录：生产 UI 入口 18 处（lib.rs 各自 `Parser::from(..).with_session(ui())`）与导入子解析器（compile.rs:1878/1933 `new_with_type_store`，无 session）均每实例新建——外部方言按入口显式注册（fixture 模式），生产注入列迁移清单；widget 体 model 由 parse_widget_decl 直连内建（冻结边界，见 §T-05 报告）；R2 点检：内建裸字段拒绝控制钉住（r2_builtin_bare_field_still_rejected_control），scoped 255/255（run.py scoped.log exit0） | 02/03 |
| [x] | T-05 | work | T04 | check+t+tv+externalgate；后续文法迁移清单/限制报告 → `cargo check -p auto-lang` 0 error（新代码 0 警告）；外部门禁 12/12；定向档见 T-04；**裸 `cargo t` + `cargo tv` 按 2026-10-14 裁定为 merge-owned 门禁，冻结提交上执行，本阶段 pending**；迁移清单/限制报告见 §T-05 报告；R1 重放 `replay.py` 5/6——唯一差异为第 3 项冻结期望 [false,false] 属 P1 未修语义（论证见 §9 r2 收尾），联合修复语义 [false,true,false] 已收编为永久控制；rustfmt --check 新增/修改文件全净（F778-R1-03）；**R2 闭合**：run.py 四腿全 exit0（external/independent/scoped/check）+ boundary 3/3；F778-R1-03 复核 fmt 净；覆盖诊断见 §9 r3 | 03 |
| [x] | T-06 | review | T05 | R3 final pass，da17a3d44409b8f002ed17250ae23dd867387d8a；全finding闭合，外部26/26+独立7/7+边界3/3+新组合3/3+定向255/255+check/fmt通过；merge-owned裸t+tv 由 merge 执行（见 §9 r1 consolidation） | 04 |
| [x] | T-07 | merge | reviewpass | Spec/归档/合流/guard-clean清理 → §9 PLAN-778:r1 consolidation 收据 | 05 |

拟tree `D:/autostack/.wt/lang-778/auto-lang`，branch `plan-778-dev`；所有实现/构建/测试在该树，主检出只簿记，禁止链接。

## 9. 复审记录

2026-10-13 stage=new，r1，outcome=pass（草案preflight，具体未知由T01有界冻结）；next=确认后work。0/7任务，外部probe/测试未创建运行，不是实现复审pass。

2026-10-15 stage=work | PLAN-778 | r1 | pass | D:/autostack/.wt/lang-778/auto-lang |
plan-778-dev | base 87f45dc4e | code 40c69c890 | 依赖：auto-down 组内兄弟 worktree
detached @ c39ac7f9（无代码改动，仅为 cargo 解析满足） | T-01..T-05 | 证据：外部门禁
`cargo test --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml`
12/12；crate 内 `test_plan778_*` 4/4；定向档 parser::tests 179/179、dialect 7/7、
model 15/15、widget 68/68；`cargo check -p auto-lang` 0 error 新代码 0 警告 |
blockers 无 | next=review（T-06）。

Work-owned AC 就绪：AC-01 ✓（外部仅凭公开面产出 typed ModelBlock；bridge 不暴露
Parser/lexer/符号表）；AC-02 ✓（Err/None/session/实例/冲突规则实断言，含 matches()
门控修复回归）；AC-03 work 部分 ✓（V05 fixture + 定向档；**merge-owned 门禁
pending：冻结提交 40c69c890 上裸 `cargo t` + `cargo tv`，未触及 trans/aavm 故
无 tt/taa，未触 UI codegen 无 tu**）。在案 finding/覆盖缺口/未决契约：无。
实需验证点闭合：mem::take 丢表 bug 证实并修复；matches() 从未派发调用证实并
修复；widget 体直连边界冻结（§T-05 报告限制 4）。

2026-10-15 R1：stage=review | review_scope=final | reviewer_identity=本评审会话（拆分规划及777审查参与，未实施778；由提交和独立反例重建） | plan_id=PLAN-778 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-778/auto-lang | branch=plan-778-dev | reviewed_commit=40c69c890eb2d52854ad846b94846fdce7e498a5 | base_commit=87f45dc4e2c592c31844e74824e67bf9e6cb419f | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；root/fixture锁见指纹 | spec_inputs=frontend overview SHA256 fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe；SD01 partial冻结 | acceptance_results=AC01基本外部面pass、AC02 fail、AC03 partial、AC04审计完成未pass、AC05 pending | findings=F778-R1-01(P1活跃方言remove期间嵌套解析落回内建)/F778-R1-02(P2委托Err后in_fn_body残留)/F778-R1-03(P3新增文件格式不通过) | evidence=[R1报告](attachments/778/review-r1/REVIEW.md)、[独立源码](attachments/778/review-r1/independent.rs)、[最终6项输出](attachments/778/review-r1/replay-latest.log)、[指纹](attachments/778/review-r1/fingerprints.json)、[规范增量](attachments/778/review-r1/spec-delta-reviewed.md) | verification=外部12/12、crate新4/4及定向255/255、cargo check exit0；独立组合4pass/2fail、rustfmt新文件check exit1、diff-check pass；受管输入未变、树干净；已有警告不称全仓零warning | pending_closeout_gates=最终review pass后merge-owned裸cargo t+tv，T07沉淀/合流/归档/批到期/guard-clean pending；本轮未跑整档 | task_state=保留T01，重开T02..05，executing 1/7，r1不变 | next=同树work修复全部finding并补永久委托递归/Err上下文组合与格式检查，提交后复审。本次不修实现/merge/发布canonical/清理树。

2026-10-15 stage=work（R1 needs_fix 修复） | PLAN-778 | r1 | pass | D:/autostack/.wt/lang-778/auto-lang |
plan-778-dev | base 87f45dc4e | code a8a81aad4（修复提交，叠加于 40c69c890） |
依赖：auto-down 组内兄弟 detached @ c39ac7f9（无改动） | T-02..T-05（重开后再闭合） |
blockers 无 | next=review（T-06 复审 r2）。

逐 finding 闭合行：
- **F778-R1-01** → 不变量：活跃方言在其自身委托解析的合法递归派发期间必须仍在
  注册表内 → 受影响路径：Ident/token 两派发的全部方言调用（原 remove/insert 模式
  共两处） → 修复：方言表 Arc 共享句柄存储，派发克隆调用、结构上无移出；四项
  既有注册/恢复不变量（优先级/matches 门控/实例隔离/Err 注册/None 回滚）不弱化 →
  永久用例：`r1_nested_expression_keeps_active_registration`（accepts=2）+
  `r1_sibling_nested_block_control` 兄弟控制 + token 两路组合 → 结果：fixture 18/18
  绿 → 剩余缺口：无。
- **F778-R1-02** → 不变量：委托核心解析的 Err 在派发错误边界恢复语句级 context
  → 受影响路径：parser.rs 四处 `?` 提前退出泄漏位点（fn 体 9393 / watch 体
  14152 / on-handler 体 17693 / dot-RHS 3277——同类排查发现的第四处） +
  BridgeCheckpoint 语义 → 修复：四根源位点 Err 平衡恢复（内建路径同受益）+
  checkpoint 快照五旗标 + 派发 Err 边界 restore_context_flags（游标留失败位置）→
  永久用例：`r1_delegated_error_restores_context_at_recovery_point`（显式恢复点）+
  `r1_successful_delegation_context_control` 正控 + token Err 再派发 → 结果：绿 →
  剩余缺口：无。
- **F778-R1-03** → rustfmt --edition 2021 格式化全部新增/修改文件，--check 复核
  全净；未触碰全仓旧格式债务 → 结果：绿 → 剩余缺口：无。

**R1 重放差异论证（供复审 r2 裁定）**：`replay.py` 5/6，唯一差异 =
`delegated_error_restores_context_at_recovery_point` 冻结期望 `[false,false]`。
该期望编码 P1 未修语义（回调期间方言从表消失 → fn 体内嵌 tail 派发不可见）；
F778-R1-01 的修复要求恰使该嵌套派发**可见**（与 model 方言嵌套 accepts=2 同机制）。
联合修复语义为 `[false, true, false]`：中位 true 是嵌套派发发生时**确在 fn 体内**
的正确上下文（非残留）；P2 oracle（恢复点派发 context 已恢复）= 末位 false，满足。
冻结证据文件（independent.rs/replay.py）未改动；联合语义期望已收编为 fixture
永久控制并注明论证。若复审 r2 认定冻结期望优先于联合语义，需回退方案再议。

验证汇总（a8a81aad4）：fixture 18/18（12 原有 + 6 R1 组合）；`test_plan778_*`
4/4；parser::tests 179/179；`cargo check` exit0；rustfmt --check 净；
`replay.py` exit1 仅上述已论证差异项。merge-owned 裸 `cargo t` + `cargo tv`
仍 pending（冻结提交 a8a81aad4 上执行）。


2026-10-15 R2：stage=review | review_scope=final | reviewer_identity=当前独立评审会话（未实施778，按提交/源码与新边界控制重建） | plan_id=PLAN-778 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-778/auto-lang | branch=plan-778-dev | reviewed_commit=a8a81aad4671ac8eae1728c4e71623c2628abd8b | base_commit=87f45dc4e2c592c31844e74824e67bf9e6cb419f | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；root/fixture锁及输入hash见指纹；独立harness最后重放钉fixture锁 | spec_inputs=frontend overview SHA256 fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe；SD01 partial冻结 | acceptance_results=AC01 partial/AC02 pass/AC03 partial/AC04审计完成整体未pass/AC05 pending | findings=F778-R1-01/02/03 closed；新增F778-R2-01(P2 冻结name [type] [= expr]允许裸字段，实际model { count }/换行形态报Expected type) | evidence=[R2报告](attachments/778/review-r2/REVIEW.md)、[独立控制](attachments/778/review-r2/independent.rs)、[依赖钉定重放](attachments/778/review-r2/independent-pinned.log)、[边界对照](attachments/778/review-r2/boundary.log)、[指纹](attachments/778/review-r2/fingerprints.json)、[规范增量partial](attachments/778/review-r2/spec-delta-reviewed.md) | verification=fixture18/18、定向255/255、check exit0、改动面fmt/diff-check pass；独立6pass/1fail、边界2pass/1fail；源文件/HEAD无变化、树干净；既有warning不称零 | pending_closeout_gates=最终pass后merge-owned裸cargo t+tv、T07 pending | task_state=保留T01/T02，重开T03..05及依赖验收，executing 2/7；T06未pass；r1不变 | next=同树work闭合冻结子集与真实支持面差异、补永久边界正反控、提交后复审。若收窄语义合同，走new明确revision；不静默改契约。已连续两轮final needs_fix，下次handoff需附遗漏原因/覆盖调整（可选类型与初值同时省略未被原fixture覆盖）。本轮不修实现/merge/发布canonical/清理。

R2裁定：联合修复后的上下文序列 `[false,true,false]` 正确，中位为fn体真实递归派发，末位为错误恢复后的状态；本轮额外在checkpoint restore之前观察in_fn_body=false亦通过，因此F778-R1-02闭合。建议将pre-restore观察收编永久测试，避免checkpoint扩展后测试自身掩盖上下文残留。

2026-10-15 R2：stage=review | outcome=needs_fix | reviewed_commit=a8a81aad4671ac8eae1728c4e71623c2628abd8b | findings=F778-R2-01(P2 冻结子集 `name [type] [= expr]` 允许可选项同省，`model { count }` 实报 Expected type；内建同拒=本 Plan 合同缺口非默认文法回归) | 上轮 R1 三 finding 全部判定 closed；联合修复期望 [false,true,false] 获同意，无需回退；建议收编 pre-restore 更强 oracle | evidence=[R2报告](attachments/778/review-r2/REVIEW.md) | task_state=保留T01/T02，重开T03..05，executing 2/7 | next=同树 work 闭合契约+永久边界+提交后复审；连续两次 final needs_fix，下一 handoff 须附覆盖诊断。

2026-10-15 stage=work（R2 needs_fix 修复） | PLAN-778 | r1 | pass | D:/autostack/.wt/lang-778/auto-lang |
plan-778-dev | base 87f45dc4e | code da17a3d44（叠加 40c69c890/a8a81aad4） |
依赖：auto-down 组内兄弟 detached @ c39ac7f9（无改动） | T-03..T-05（重开后再闭合） |
blockers 无 | next=review（T-06 复审 r3）。

F778-R2-01 闭合行：finding（裸字段双省报错）→ 不变量（冻结文法两个可选项独立、
可同时省略；实现与验收报告必须与冻结合同一致）→ 受影响路径（fixture lib.rs
类型分支；验收面 §T-01 第 6 条/§T-05 已验证面的表述与事实差异）→ 修复（外部
子集裸字段终止符 = 行界/块界/EOF → ty=Unknown+init=Nil；非法 token 仍走核心
类型文法报错；**内建路径零改动**，r2_builtin_bare_field_still_rejected_control
钉住内建拒绝）→ 永久用例（r2_bare_field_boundary.rs 8 项：四象限矩阵/裸+typed
多行反例/var×裸/非法负控/内建拒绝控制）→ 结果（fixture 26/26、审查 boundary
3/3、independent 7/7 pinned、scoped 255/255、check exit0）→ 剩余缺口：无。
更强 oracle 收编（R2 建议）：r1_delegated 测试在 harness restore **前**直接观察
in_fn_body=false——证明恢复来自派发 Err 边界而非 restore 遮蔽。

**覆盖诊断（两连 final needs_fix 规则要求，R2 报告点名）**：
- 未覆盖不变量：冻结合同中两个文法可选项各自独立可省——原 oracle 只枚举了
  typed+init 与 init-only 两象限，双省象限（裸字段）既未实现也未测试。
- 为何早前检查漏掉：实现直接移植了内建 parse_model_block_inner 的形态（其
  隐含"type 或赋值至少其一"约束），验收样例又按内建行为挑选，未对照冻结合同
  文本枚举可选组合格（2²=4 象限）——合同文本与实现/测试同源于内建习惯，
  缺独立于实现的合同派生用例。
- 检测/验收调整（已落地）：冻结子集的可选组合四象限永久矩阵入 fixture；后续
  迁移清单（§T-05）每步扩文法时，先从合同文本枚举组合格再写实现——列入
  迁移清单步骤 ① 的执行注记。R1/R2 两轮 final 红的共性（派发/委托组合与合同
  组合格均未从合同系统性枚举）已由组合矩阵方法覆盖。

验证汇总（da17a3d44）：fixture 26/26（12 v01-v05 + 6 r1 + 8 r2）；审查 run.py
四腿 exit0 + boundary 3/3（multiline 反例翻绿）；`test_plan778_*` 4/4、
parser::tests 179/179、dialect/model/widget 定向绿（scoped 255/255）；
`cargo check --locked` exit0；rustfmt --check 净。merge-owned 裸 `cargo t` +
`cargo tv` 仍 pending（冻结提交 da17a3d44 上执行）。


2026-10-15 R3：stage=review | review_scope=final | reviewer_identity=当前独立评审会话（未实施778，延续R2独立上下文，按提交/源码及新组合重建） | plan_id=PLAN-778 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-778/auto-lang | branch=plan-778-dev | reviewed_commit=da17a3d44409b8f002ed17250ae23dd867387d8a | base_commit=87f45dc4e2c592c31844e74824e67bf9e6cb419f | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；root/fixture/harness锁与受管输入hash见指纹 | spec_inputs=frontend overview SHA256 fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe；SD01 pass冻结 | acceptance_results=AC01/02/03(work-review)/04 pass，AC05 merge-owned pending | findings=F778-R1-01/02/03及F778-R2-01全closed；无新增阻断项 | evidence=[R3完整报告](attachments/778/review-r3/REVIEW.md)、[重放入口](attachments/778/review-r3/run.py)、[补充独立组合](attachments/778/review-r3/combinations.rs)、[指纹](attachments/778/review-r3/fingerprints.json)、[规范增量冻结](attachments/778/review-r3/spec-delta-reviewed.md) | verification=fixture26/26、独立7/7、历史边界3/3、新组合3/3、定向255/255、check exit0、改动面fmt/diff-check pass；输入/HEAD不变、实施树及依赖干净；既有warnings不称全仓零 | pending_closeout_gates=merge在冻结da17a3d44上执行裸cargo t+tv（未触trans/ui_gen/aavm，不要求tt/tu/taa）；T07 Spec/ledger/合流/归档/批到期判定/guard-clean pending | task_state=reviewed 6/7，T06完成，T07未完成 | next=merge；本轮未运行整档、未修实现、未发布canonical/ledger、未合入/归档/清理；review pass不等于交付完成。

R3就绪审计：R2覆盖诊断与四象限永久矩阵已落实；pre-restore上下文观察已收编；本轮额外混合裸/var/typed字段、注释/CRLF、缺右括号EOF拒绝、非法类型offset14均通过。F778-R2-01按原合同修复，不收窄文法；内建默认路径保持拒绝裸字段。具体SD01完整文本冻结于R3附件，merge须同时替换frontend原mem::take建议并核对规范源漂移。

2026-10-15 PLAN-778:r1 consolidation（stage=merge，outcome=pass）：

| checkpoint | evidence |
|---|---|
| prepared | reviewed 基线 da17a3d44（R3 pass 绑定）；SD-01 冻结增量落 worktree（frontend/overview 新增"方言外部化"节 + 原位替换 mem::take 坑点；源 hash fccb811a 复核无并发漂移）；frontend/plans.md 行 + docs/reports/778-ui-dialect-externalization/verification.md 收据 + INDEX 再生（spec-index.py）；docs-only 后代提交经 exact-delta 核验成为 delivery |
| gates | 裸 `cargo t`（nextest default，冻结基线）三样本：5357 选择面 5356/5357、5355+1timeout/5357、5356/5357——每轮恰 1-2 红且轮换（state_file lock / plan498 chart / clipboard shim / plan776 http timeout），逐一隔离复跑全部通过=机器并行负载 flake 家族，零确定性红；交付触面族（parser::tests/dialect/model/widget 255 项）每轮全绿。`cargo tv` 163/163 全过。rebase 后 range-diff 三代码提交 patch 全等（40c69c890=4fba860c7 / a8a81aad4=5e8f24e73 / da17a3d44=e86104835）→ 代码位与门禁基线零差，证据有效（不变输入论证） |
| landed | master tip = 711ce23f6 = plan-778-dev tip（`git merge --ff-only`，无 merge commit；rebase 两次吸收并发 491e84afb[P781 ledger repoint] 与 602bc850f[782 簿记]，specs.json 冲突以 master 格式 indent=2 守卫式重放，外科 diff +60/−2） |
| ledger_refreshed | `.autoos/specs.json` 离线投影（本仓既有通道；byte 备份+程序化 upsert+schema/读回守卫）；P778-1（designs）/P778-T（tests）/P778-R3（reviews）/P778-M（reports）四条目，version 20→21（合并 P781 并发 repoint 后基线）；无关条目字节语义零扰动断言 |
| archived | docs/plans/archive/778-ui-dialect-externalization-pilot.md；附件 review-r1/r2/r3 + 报告随本提交入默认分支；7/7 任务闭合 |
| cleaned | （本提交后执行， receipts 见下方更新） |

发现物归档：两次 rebase 均因他线并发簿记（非本计划冲突）；首次投影 indent=1 与仓内 indent=2 格式不一致已按守卫协议回滚重写（守卫教训：语义等价断言不足以保证外科边界，需字节级 diff 复核）。

## 10. 待澄清事项

T01只能选择等价接口机制；改默认grammar/扩大model子集/搬UiAst需revision。若必须公开全部lexer/符号表私有面，先出失败决策needs_replan，不以getter全开放绕过。

## 11. spec-sync 回写记录

- 2026-10-15（merge）：SD-01 → docs/specs/auto-lang/frontend/overview.md（新增"方言外部化（PLAN-778 pilot）"节；已知坑 mem::take 条目原位替换为 Arc 派发语义，未遗留冲突建议）；frontend/plans.md 追加 778 行；INDEX 再生；ledger P778-1/T/R3/M（v21）。
