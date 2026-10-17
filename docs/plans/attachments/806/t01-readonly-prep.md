# PLAN-806 T-01 只读预检（前置未满足期的允许范围）

2026-10-17 / stage: work（预检）/ baseline: master `80f40af641d087823ba40910d390b1224abc9632`。
依 §9 起草记录与 §10："799/801审查合入后，work以真实API执行T-01。此前仅只读准备"。
本件为主检出（master）只读审计收据，**未建树、未写任何 Rust/manifest、未读取或复制
801 worktree 的未交付 API**。T-01 中依赖 801 最终端口的部分保持 pending。

## 1. 前置状态核对（2026-10-17 实测）

| 前置 | 状态 | 证据 |
|---|---|---|
| PLAN-799 审查合入 | ✅ 已合入 | `docs/plans/archive/799-frontend-ast-model-extraction.md`；主检出 `crates/auto-ast/` 在位 |
| PLAN-801 审查合入 | ❌ 未合入 | `docs/plans/801-*.md` 未归档、`status: execution_done`（R1 修复闭合 code `49512993b`，送 T-07 二轮复审，merge 门 pending）；主检出 `crates/` **无 `auto-parser-api`**，全仓无 `parse_model_subset`/`ExternalModelDialect` 生产符号 |

结论：T-01 硬前置"799/801审查合入"半满足，work 仍 blocked；本件仅为允许的只读准备。

## 2. 三算法定位与行为冻结点（master 实锚，全部 `crates/auto-lang/src/parser.rs`）

| 算法 | 位置 | 关键行为（迁出时不得收紧/扩展） |
|---|---|---|
| `parse_msg_decl`（pub 薄包装→Stmt） | 14750 | `parse_msg_decl_inner` 后包 `Stmt::MsgDecl` |
| `parse_msg_decl_inner` | 14756–14840 | `expect_ident("msg")`；**旧 `msg Name {` 名字读弃**（Plan 448A 注释明示名字从未语义化）；quoted variant（`TokenKind::Str` 记 `MsgVariant.quoted`，Vue codegen 合同面）；多 payload（043 M5#1）；**named payload 探测 = lexer 单 token 回推**（`self.lexer.next()`+`push_token`，Ident/Str 后看 Colon；435 挂牌②，名字入 `payload_names`，位置形态 None）；空行/逗号宽容（循环头尾 `skip_empty_lines`）；缺 `}` → `expect(RBrace)` 报错 |
| `parse_model_block`（pub 薄包装→Stmt） | 14843 | 同构包 `Stmt::ModelBlock` |
| `parse_model_block_inner` | 14849–14901 | `var` 可变旗标（Plan 130）；`name` 后若非 `Asn` 则**必 parse_type**（`=` 缺省仍必类型——内建默认**无裸字段**，与 778 冻结一致）；`=` 后 parse_expr，否则 `Expr::Nil`；字段四元 flags（name/ty/init/mutable/is_primary/decorators） |
| `parse_model_field_annotations` | 14905–14970 | 多注解循环（`#` 后非 `[` 直接返回已有值）；空注解 `#[]` 跳过；`primary` 布尔；`Consume/Provide/NavParam` 可选单 Str 参数（缺参宽容）；**未知注解 → `SyntaxError::Generic`，消息含合法清单原文、span=`pos_to_span(self.cur.pos)`**；尾部 `]` 宽容缺省 |

迁出等价实现需覆盖的硬点：msg named-payload 的 lexer 回推 lookahead（801 端口上须以
checkpoint/advance/restore 或等价安全形态表达——具体以 801 合入后实际端口为准，本轮不预写）。

## 3. 生产调用点全图（迁出后须"真实委托"的完整清单）

| 路径 | 位置 | 形态 |
|---|---|---|
| 顶层（语句位置） | `dialect/ui.rs:49` `"msg" => p.parse_msg_decl()?`；`:50` `"model" => p.parse_model_block()?` | 经 pub(crate) escape hatch `bridge.parser()`（PLAN-778）；前置护栏：`p.in_on_body || p.in_fn_body` 直接 `Ok(None)`（Plan 028 T14——on/fn 体同名词不拦截） |
| widget 体 | `parser.rs:13098` msg / `:13101` model（`parse_widget_decl` 内循环） | **直连 inner，不经方言表**（778 冻结："方言派发只覆盖语句位置；widget 体循环直连 parse_model_block_inner"） |
| store 体 | `parser.rs:14371` / `:14374`（`parse_store_decl`） | 同上直连 inner |
| component fn 体 | `parser.rs:15841` / `:15845`（`parse_component_fn_decl`） | 同上直连 inner |
| 生产 import/新 Parser | `Parser::new_with_type_store(...)` + `with_session`：`compile.rs:1883/1933`、`lib.rs:1658/2200`、`lib.rs:3715`（`Parser::from(...).with_session(...)`） | 默认装配点=`build_dialects`（`parser.rs:655-662`：UI session 恰推一个 `UiDialect` 进 builtin 段）；import 文件与主文件走同一默认装配 → UiDialect 分支委托改新实现后天然同源 |

默认装配/三段表：PLAN-778 冻结 `Override → builtin → Append`（`dialects_pre/builtin/post`，
`parser.rs:666-677` 段表 + `DialectPriority`）。外置后不得把默认装配伪装成用户 Override 注册。

## 4. pilot / bare-field 状态（默认 vs 显式子集，T-04 输入）

- master 上**不存在** `parse_model_subset`/`ExternalModelDialect` 生产符号（grep 全仓零命中）——
  801 的"model 单份迁入+独立 model consumer"是其 worktree 未合入交付，T-04 只能等真实合入。
- 778 已合入的外部子集试点在 fixture：`test/fixtures/modularity/ui-dialect-consumer/`
  （含 `tests/r2_bare_field_boundary.rs` 四象限矩阵）——子集 `model { [var] name [type] [= expr] }`，
  **类型/初值可同时省略（裸字段）**，遇 `#[` 注解 checkpoint 回滚交内建（778 冻结记录 §6）。
- 默认内建 `parse_model_block_inner` **不做裸字段**（§2）——806 正式切换不得顺手把 pilot
  扩展变默认文法（计划 §4 已钉死）。

## 5. 旧臂锚点（V02/V03/V05 冻结对照的入口起点，穷举归 T-01/T-05）

- parser.rs 内联测试模块：`18386/18396`（`.msg(...)` 事件绑定 + `msg Msg { step(int, str) }` 旧名形态）、
  `18428/18434`（`.msg` glue 复现守护）、`18790/18823/18870/18911`（`msg Msg { S/Save }` 场景）。
- 778 fixture：`ui-dialect-consumer/tests/r2_bare_field_boundary.rs`（默认/pilot 差异正反控已存在，
  T-04 须保住"既删 bare-field 正控又宣称兼容"红线——复用或收编该矩阵）。
- 消费臂（V05）：`test/a2ark/**` golden 族、gallery 相关测试、`ui_gen` 生成器 golden（quoted/name
  payload 的 Vue/Rust 代表生成）——T-05 按实际触面收敛清单。

## 6. 801 合入后 T-01 仍须完成的剩余部分（本轮未做，不得视为已冻结）

1. 以**主检出真实** `auto-parser-api` 端口（ParseContext/checkpoint/恢复语义，R1 后含签发作用域
   快照失效）复核 msg named-payload lookahead 等价表达；若需新增通用端口，按 §5 具名说明+测试。
2. 801 pilot（parse_model_subset/ExternalModelDialect 实际形态）→ 单份 model 算法策略表达。
3. 全部 callsite/可见性/port 差额冻结表 + 旧臂 golden 清单穷举。
4. 建树 `D:/autostack/.wt/lang-806/auto-lang`（branch 按 §6：`codex/plan-806-dev`）后方可在
   代码上执行 T-01 验证。
