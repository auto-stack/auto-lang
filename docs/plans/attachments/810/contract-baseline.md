# PLAN-810 T-01 契约基线（contract-baseline）

2026-10-19。工作树 `D:/autostack/.wt/lang-810/auto-lang`，分支 `codex/plan-810-dev`，
基面 master `ab08955c2`。本文冻结完整 UI 文法的迁移闭包、调用/副作用图、
端口设计与事务合同，作为 T-02..T-08 的实施合同。行号均指基面
`crates/auto-lang/src/parser.rs`。

## 1. 迁移闭包（MIGRATION CLOSURE）

**迁入 auto-ui-dialect（完整 UI 文法 + 必要私有 helper）：**

| 区段（parser.rs 基面行号） | 内容 |
|---|---|
| 13223–14105 | `parse_widget_decl` + mint 族（`mint_inline_event_handlers/mint_view_inline/mint_events_inline/mint_bare_input_sync/modal_dialog_tag_role/hover_card_role/mint_modal_dialog_toggle(_inner)/direct_state_value_field/wire_modal_close_recursive`，纯 AST 变换） |
| 14105–14269 | `parse_widget_use_block_inner`（UI 专属 use 块 → ExtImport；不触模块加载） |
| 14269–14401 | `parse_watch_block_inner`（handler 体内 in_on_body 包裹） |
| 14401–14459 | `parse_setup_block_inner`（refs 声明 + parse_stmt 委托 + await 拒绝） |
| 14459–14502 | `parse_expose_block_inner` |
| 14502–14526 | `parse_style_block_inner`（raw CSS：`lexer.capture_raw_block`） |
| 14526–14630 | `parse_store_decl`（store 的 UI 声明；Core `parse_store_stmt`(8723) 不迁） |
| 14630–14902 | scene 族：`looks_like_scene_decl/parse_string_lit/parse_scene_decl/parse_scene_node/parse_scene_instance/parse_scene_connection/parse_scene_signal/parse_scene_prop/parse_scene_value/looks_like_subresource/parse_scene_subresource`（scene 门在任何 scenario 生效，Plan 306；一并迁入文法 crate，Core 经端口调用） |
| 14902–14928 | `parse_widget_props` |
| 14955–15053 | `parse_computed_block_inner`（object 投机 + 局部诊断回滚事务） |
| 15053–15111 | `parse_routes_block_inner` |
| 15111–15131 | `parse_actions_decl_stmt`（probe 前瞻门） |
| 15131–15228 | `parse_style_recipe_decl_stmt`（raw save/restore 投机 + `register_style_recipe` 全局副作用） |
| 15228–15332 | `parse_timer_block_inner` |
| 15332–15593 | `parse_actions_block_inner/parse_action_entry/parse_actions_string_attr/parse_actions_handler_attr/parse_actions_cond_attr` |
| 15593–15733 | `parse_menubar_block/parse_menu_entry/parse_toolbar_block/parse_menu_item_entries` |
| 15733–16036 | `parse_view_block/parse_view_block_or_fragment/parse_view_fragment_decl_body/parse_component_fn_decl/fragment_param_hint_to_type/parse_fragment_decl_body_tail/parse_view_block_inner/parse_view_root_nodes` |
| 16036–17054 | `parse_view_node/parse_style_binding/parse_view_for_loop/parse_view_key_expr/parse_view_memo_block/parse_view_conditional/parse_view_link` |
| 17054–17222 | `parse_condition_expr/capture_condition_call_args`（条件 DSL 源文本捕获） |
| 17222–17598 | `parse_event_modifiers/parse_event_value/parse_event_handler/parse_event_arg_list/parse_event_arg/parse_event_arg_object/parse_handler_params` |
| 17598–17794 | `parse_on_block/parse_bind_block` |
| 17794–17816 | `expect_ident`（**留 Core** 作具名端口能力——35 调用位经桥委托同一实现；实施勘误见 §5） |
| 17816–17854 | `get_primary_prop`（纯静态表） |
| 17854–17909 | `extract_fstr_template_and_bindings`（纯 Expr 投影；ui/desktop_protocol/client_runtime.rs:241 有同构私有拷贝，本就独立，不动） |
| 文件级自由函数 | `on_handler_event_name`(345)、`damerau_levenshtein_1`(353)、`stmt_expr_contains_await`(123) —— 均 UI 区专用，随迁为 crate 私有 |

**留在 Core（Parser）——共享/通用文法所有者，UI 经端口消费：**

- `parse_on_events`(13195) 及事件前置族 `parse_event_src/parse_cond_arrow/parse_arrow/parse_arrow_list/parse_event/parse_goto_branch`(13110–13194)：Core 非 UI `on` 回退（parse_stmt_inner:5659）与 `ast_parse_ext::ParserExt`（Event/Arrow/OnEvents）共用，归 Core。
- `grid`(13087)、`call`(13023)、`parse_nav_call`(13053)：核心表达式文法。
- msg/model 声明：PLAN-806 已在 auto-ui-dialect `declarations.rs` 单源；新文法模块**直接调用同 crate 实现**（不复制第二份）。`parse_msg_decl/parse_model_block` Parser 公开面继续薄委托。
- `parse_store_stmt`(8723)（Core 表达式级 store 语句）：不迁，与 `parse_store_decl` 按（函数+调用点）区分。
- 通用 expr/type/body/stmt/closure/atom/object/fstr/dot_item/词法/诊断/符号工具：全部留 Core，经端口。

**区外调用者（迁移后必须真实消费新实现）：**

| 入口 | 基面位置 | 说明 |
|---|---|---|
| UiDialect try_parse_stmt：widget/msg/model/component/actions/style | dialect/ui.rs:48–63 | 现 `bridge.parser()` 逃逸；改调新文法（msg/model 走 806 已有 crate 面） |
| UiDialect try_parse_token_stmt：View/On | dialect/ui.rs:91–94 | view→`parse_view_block_or_fragment`；on→Core `parse_on_events`（共享，经端口） |
| parse_stmt_inner Ident 直连段 | parser.rs:5635–5648 | scene 门（任意 scenario）→ `looks_like_scene_decl/parse_scene_decl`；store 门（UI 且 !in_on_body）→ `parse_store_decl` |
| parse_stmt_inner `pub style` | parser.rs:5343–5353 | `parse_style_recipe_decl_stmt` + is_pub 翻转 + None 回推（回推逻辑留 Core） |
| ast_parse_ext `ParserExt for OnEvents` | ast_parse_ext.rs:57–63 | `Parser::from(code).skip_check().parse_on_events()`——OnEvents 留 Core，零迁移 |
| 测试面 | plan024/051/367/408/806…+ui_gen 全家 | 经真实 Parser 全链路回归（旧 oracle） |

**派发协议（不变量，来自 PLAN-778/801）：** `try_dialect_stmt/try_token_dialect_stmt`（889–972）对 `Ok(None)` 强制 `bridge_restore`（游标+五旗标，诊断单调）；`Err` 仅 `restore_context_flags`（游标停在失败位，错误位置不变量）；Override→builtin→Append 三段序；`with_session` 重建 builtin 段（816–825，UI scenario 注册 UiDialect）；外部注册跨 with_session 保留。

## 2. Core 设施触面 → 端口能力映射（逐方法冻结）

UI 区对 Parser 内部的全部触面（基面实测，区域 13223–17909）：

| 触面（旧代码形态） | 位点数 | 端口能力（UiParseContext 新增） | 实现侧（ParserBridge→Parser） |
|---|---|---|---|
| `self.is_kind/kind/pos/cur.text/cur.pos` 读 | ~430 | 已有 `ParseContext::cur_kind/cur_text/cur_pos/is_kind` | 不变 |
| `self.next()`（副作用推进：lex 错误记录+EOF 兜底+skip_comments→pending_docs） | 239 | 已有 `advance()`（克隆返回） | 不变 |
| `self.expect` | 129 | 已有 | 不变 |
| `self.expect_ident(w)`（17794：纯文本比较、错误文本 "Expected 'x', got 'y'"） | 35 | `expect_ident(&str)` | `Parser::expect_ident` 保私有，桥面委托同一实现 |
| `self.skip_empty_lines` | 130 | 已有 | 不变 |
| `self.cur_is_soft_ident()`（1017） | 3 | `cur_is_soft_ident() -> bool` | 委托 |
| `self.prev.pos`（span 定位/起点算术 16735） | 6 | `prev_pos() -> Pos` | 读 `self.parser.prev.pos` |
| `self.lexer.next()` + `push_token`（单 token 原始前看，两种失败协议） | 12 | `probe_next_token() -> AutoResult<Token>`（成功即回推；词法错误**上抛**） | `lexer.next()` 成功 push 回、Err 原样传播；既有 `probe_next_kind`（Option，错误丢弃）= 其 kind 投影，806 冻结面不动。if-let-Ok 位点用 `probe_next_token().ok()`；`?` 位点（memo 16056/paren 16293）直接传播 |
| `self.lexer.save_state/restore_state + cur/prev clone`（**原始游标投机**：无旗标、无诊断回滚；中间 next() 的 docs/诊断副作用保留） | 3（widget 近失 13388、computed 15006、style recipe 15132） | `raw_cursor_save() -> UiCursorSave` / `raw_cursor_restore(UiCursorSave)` | Parser 侧句柄表存 `LexerState+cur+prev`（与 port_snapshots 同构、独立槽表、作用域失效同规则）；**不恢复五旗标、不回滚诊断**——与旧臂逐字等价 |
| `self.lexer.push_token(cur/…)`（双 token 回推再 next() 重同步：scene 门 14638、subresource 14875、stream 17628） | 3 | `push_back_token(Token)` | `lexer.push_token` |
| `self.lexer.capture_raw_block()`（raw CSS 字节 + 收尾 `}` 消费） | 1（14516） | `capture_raw_block() -> AutoResult<String>` | 委托；后继 `next()` 由文法自行调用 |
| `self.skip_check` 保存/置位/恢复 | 2（computed 15003、style recipe 15199） | `set_skip_check(bool) -> bool`（返回旧值） | 读改写 |
| `self.errors/warnings` truncate（投机局部回滚） | 1（computed 15009–15019） | `diagnostics_mark() -> (usize, usize)` + `diagnostics_truncate((usize, usize))` | len 快照 + truncate；**仅限 raw_cursor 投机回退位**（见 §3 事务合同） |
| `self.in_on_body` 保存/置位/恢复 | 2（watch 14359、on 17706） | `set_in_on_body(bool) -> bool` | 读改写 |
| `self.enter_scope/exit_scope`（infer_ctx push/pop） | 4 对 | `enter_scope()/exit_scope()` | `Parser::enter_scope/exit_scope` |
| `self.infer_ctx.bind_var(Name, Type)` | 5（fragment 15835/15844、component 15975、for 16700/16705） | `bind_var(&str, Type)` | `infer_ctx.bind_var` |
| `self.define(name, Meta::Store(Store{Var,…}))` | 1（on handler 参数 17699–17702） | `define_handler_param(&str)` | 桥内构造同一 Meta::Store 后 `Parser::define` |
| `self.warn(Warning::SuspiciousBlockKeyword{…, span: cur.pos})` | 1（13402–13408） | `warn_suspicious_block_keyword(String found, String suggestion)` | 桥内以 cur.pos 构造同变体 |
| `self.take_docs()` | 1（style recipe 15155） | `take_docs() -> Option<String>` | 委托（AutoStr→String） |
| `crate::design_tokens::recipe::register_style_recipe(&decl)` | 1（15212） | `register_style_recipe(&StyleRecipeDecl)` | 委托 design_tokens 注册表（Core 侧语义存储） |
| `self.parse_expr/parse_type` | 17/3 | 已有 | 不变 |
| `self.body()`（Body 块解析） | 4（watch 14361、computed 15019、on 17708） | `parse_body() -> AutoResult<Body>` | 委托 |
| `self.parse_stmt()` | 1（setup 14435） | `parse_stmt() -> AutoResult<Stmt>` | 委托 |
| `self.parse_closure()` | 1（内联 handler 17253） | `parse_closure() -> AutoResult<Expr>`（签名以基面为准） | 委托 |
| `self.atom()` | 2（均在事件族外?——实测 region 13115/13117 属 parse_event_src=Core；迁移区 0 处） | —（不进端口） | — |
| `self.object()`（对象字面量对列表） | 1（computed 15012） | `parse_object_pairs() -> AutoResult<Vec<Pair>>` | 委托 `Parser::object` |
| `self.fstr()` | 4（view 文本/fstr 值 16121/16213/16498…） | `parse_fstr() -> AutoResult<Expr>` | 委托 |
| `self.dot_item()` | 1（16235） | `parse_dot_item()`（签名以基面为准） | 委托 |
| `self.parse_on_events()` | dialect/ui.rs:94 | `parse_on_events() -> AutoResult<OnEvents>` | 委托 Core 共享文法 |
| `self.parse_event_src/parse_arrow/…`（ast/on 事件族） | region 内 0（全在 Core 事件族 13110–13194 自用） | —（不进端口；事件族留 Core） | — |

**对象安全**：所有新能力均 `&mut self`/`&self` + owned/借用无关参数（String/&str/Token/Type/Stmt/Body/…），返回 owned 值或 `AutoResult`；`UiCursorSave` 为不透明 owned 凭据（同 `ParseCheckpoint` 模式：实例 id + 槽位 + 单调 gen + 作用域失效）。无泛型方法、无 `impl Trait`、无 `Self` 返回。**既有 `ParseContext` 零改动**——外部 consumer（dialect-api-consumer 等）不受影响；`UiParseContext: ParseContext` 子 trait 定义在 auto-ui-dialect，由 auto-lang 侧 `ParserBridge` 实现（依赖方向 auto-lang→auto-ui-dialect，无反向边）。

**语法树/词法/诊断身份**：全部沿用 auto-ast（UI AST 46 类在 `auto-ast/src/ast/ui.rs`；`OnEvents` 在 `ast/on.rs`）、auto-lex Token、auto-diagnostics Warning/AutoError。无镜像 DTO。

## 3. 事务/前瞻/raw/scope 合同冻结

1. **probe（零副作用单 token 探测）**：`probe_next_kind`（806 冻结）语义不变。新 `probe_next_token` 是其全 token 形态：成功路径回推+零游标副作用（随后 advance 重新消费并执行全部副作用）；失败路径遵循旧回推前瞻失败消费协议（词法失败是否推进由 Lexer 失败模式决定，错误**上抛**——供 memo/paren_is_expr 的 `?` 位点；if-let-Ok 位点用 `.ok()` 丢弃，与旧臂逐字一致）。
2. **raw cursor 投机（区别于 checkpoint）**：`raw_cursor_save/restore` 只存/回 lexer 全状态 + cur/prev，**不含五旗标、不回滚 errors/warnings/pending_docs**——与旧臂 13388/15006/15132 逐字等价（中间 `next()` 的 docs 收集与 lex 错误记录**保留**）。不得用 `ParseContext::checkpoint/restore` 替代（其额外恢复旗标，会在 object() 投机失败泄漏旗标路径上产生行为漂移）；也不得反向用 raw save 替代方言 None 回滚（派发器继续用 bridge_restore）。
3. **computed 局部诊断事务**：object 失败→body 回退时显式 truncate errors/warnings 至 mark 位 + 恢复 skip_check + raw cursor restore。这是**唯一**允许的诊断回滚位点（diagnostics_truncate 能力文档明示仅限 raw 投机回退；checkpoint 路径诊断恒单调）。不得扩大到所有 restore。
4. **skip_check / in_on_body**：保存-置位-恢复三段式由文法代码用 `set_*` 返回旧值复刻；错误路径的平衡恢复（on 块 `body_res` 先恢复旗标再 `?`）逐字保持（PLAN-778 R1 F778-R1-02 形态）。
5. **scope/寿命**：`enter_scope/exit_scope/bind_var/define_handler_param` 只作用于 infer_ctx 符号面；不返回 TypeStore/InferenceContext 引用。fragment/component 参数与 for 绑定的 bind_var(Type::Unknown/Int)、on handler 参数的 Meta::Store define 与旧臂同序同寿命；Err 路径 exit_scope 时序逐字保持（如 on 块先 exit_scope 后 push）。源码借用不逃逸（`capture_raw_block` 返回 owned String）。
6. **raw CSS**：`capture_raw_block` 由 Lexer 在 LBrace 直后无缓冲位调用（文法侧保持调用前置条件注释），嵌套括号/字符串/注释/换行/Unicode 由 Lexer 现实现保持；后继位置=闭 `}` 后首 token（文法自行 `advance`）。
7. **样式配方注册**：`register_style_recipe` 是解析期全局副作用（design_tokens 注册表），经端口在**成功构造 decl 后**调用一次，时序/参数与旧臂一致；`has_style_recipe` 的 Core check_symbol 消费不动。

## 4. 支持矩阵与旧 oracle

- **旧 oracle = 基面 ab08955c2 的真实 Parser 行为**，由现有测试面钉住：plan024（命名 view）、plan051（timer/ext widget）、plan367（view fn/fragment）、plan408（component/computed/use）、plan806（msg/model 冻结四元组）、plan451 actions、plan607 style recipe、widget_macro/native_css/plan340/plan484/plan492…，以及 ui_gen（vue/jet/ark/rust 生成器金样）与 examples 真实消费（V-05）。迁移期间这些测试**不改动断言**；红=行为漂移。
- **已知怪癖按原样保留**：无点前缀带参 pattern（Plan 435 挂账②）、`>`/`->`/`=>` 混写宽容、近失关键字告警（damerau-1）、computed 错误信息枚举 12 关键字、on stream 合成 `__stream_sse_*` pattern、outlet 头参三态、`on "quoted-event":`、await 在 setup 拒绝、msg 名字读弃（Plan 448 A，806 冻结）等。无界 EOF 怪癖若在反例中发现，隔离进程+外部截止证基线，不偷偷修复。
- **工作期验证命令**（T-02..T-06 逐批执行）：`cargo check -p auto-ui-dialect -p auto-parser-api -p auto-lang`；定向 `cargo t plan810`、`cargo t plan806`、`cargo t plan408`、`cargo t widget`、`cargo t view`、`cargo t plan367`、`cargo t plan024`、`cargo t plan051`、`cargo t actions`、`cargo t style`、`cargo t store`、`cargo t scene`。生成器/VM 代表矩阵由 T-07 冻结可重放选择。

## 5. T-01 可行性 probe（结果）

按计划要求打通"含 model/props、computed、raw CSS、watch/on、fragment scope 与 view 节点的生产解析路径"：

1. `auto-ui-dialect` 新增 `grammar.rs`（UiParseContext trait + 首批迁移：widget 声明闭包 + view 链），`ParserBridge` 实现 `UiParseContext`。
2. probe 测试（worktree 内 `crates/auto-lang/src/tests/plan810_full_ui_grammar.rs` 雏形）：真实 `Parser::from(src).with_session(UI)` 解析含 props/model/computed/style(raw CSS)/watch/on(view 节点+for/key/memo/if/link/事件)/fragment 的样例，AST/诊断与基面一致。
3. probe 结论：端口可行（对象安全、无反向边、既有 consumer 编译不受影响）→ 进入 T-02..T-04 批量迁移。probe 结果回填于 §6。

**实测结论（2026-10-19）**：probe 以"全量迁移即 probe"形态完成——T-02 端口
（`grammar/mod.rs` UiParseContext + ParserBridge 实现 + Parser 后端）编译
通过后，T-03/T-04 一次性迁完整个闭包（约 4.9k 行），全部定向面绿
（见 §6）。对象安全由轻 consumer 的 `&mut dyn UiParseContext` 上转型
`&mut dyn ParseContext` 实测（806 declarations 单源在同一驱动内可用）；
既有 ParseContext/ExternalDialect consumer 编译零改动（777 strict 复验）。

**实施勘误（相对本文初稿）**：
- `expect_ident` 留 Core 作具名端口能力（35 调用位经桥委托同一私有实现，
  优于本地复刻）；不随区域迁出。
- `probe_next_token` 失败路径**上抛**（`?` 位点），if-let-Ok 位点 `.ok()`。
- `advance_peek_kind` 新增为注释跳过前看的具名化（scene/subresource/
  stream 三位点共用旧臂形态）。
- `parse_ident`/`expect_eos`/`atom` 不入端口——迁移区零调用（均在 Core
  事件族内自用）。
- `parse_msg_decl_inner`/`parse_model_block_inner`（806 桥接 inner）随调用
  方迁出而删除；`parse_msg_decl`/`parse_model_block` 公开委托收编为直调
  declarations。

## 6. probe 与迁移收据（work 阶段实测，2026-10-19）

- [x] UiParseContext 定义 + ParserBridge 实现编译通过（`cargo check -p
  auto-ui-dialect -p auto-parser-api -p auto-lang` 零错误；新增 warning 清零）
- [x] widget 闭包迁移 + dialect widget 臂接线，定向测试绿
- [x] view 链迁移 + fragment scope probe 绿
- [x] 剩余（component/store/scene/actions/style_recipe/routes/timer/
  menubar/toolbar）迁移绿
- [x] escape hatch 删除（全仓 `bridge.parser()` 调用清零）/派发接线/旧算法
  删除（parser.rs 残留 UI 算法/自由函数 grep 清零，仅余 8 个同签名薄委托
  + 806 收编委托 + scene 门委托）

**定向面**：plan806 18/18、plan408 20/20、plan367 6/6、plan024 16/16、
plan051 25/25（含 timer 19）、widget 68/68、store 56/56、view_ 240/240、
actions 7/7、scene 6/6、style 198/198、recipe 18/18、native_css 7/7、
plan339 7/7、plan340 12/12、plan370 18/18、plan778 4/4、plan810 13/13。
**轻闭包**：ui-grammar-consumer 7/7 + verify-deps PASS（x2 含 806）；
**777** strict verdict=pass violations=0。**792** driver 后台执行（收据见
handoff.md）。merge-owned 门（裸 t/tv/tu）按 2026-10-14 时序裁定留
merge 冻结统一跑，handoff §5 记 pending。
