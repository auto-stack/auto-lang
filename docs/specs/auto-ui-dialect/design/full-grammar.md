# auto-ui-dialect 完整 UI 文法设计（full-grammar）

> **Status**: active（PLAN-810 交付的 canonical 设计沉淀）
> 路径：`crates/auto-ui-dialect/src/grammar/` | 依赖闭包同
> [../project.md](../project.md)（仅 auto-parser-api 轻层）

## 1. 范围与调用图

单一份实现承载完整 UI 文法（自 auto-lang `Parser` 逐字迁出，基面
`ab08955c2` parser.rs 13223–17909）：

| 模块 | 内容 |
|---|---|
| `widget.rs` | widget 壳、props、use/setup/expose/watch/computed/on/bind 块、内联 handler mint 族（纯 AST 变换：inline lambda→msg/on、bare input 同步、modal/hovercard 铸造） |
| `view.rs` | view 块/`view fn` fragment/`component fn`（糖化 WidgetDecl）、节点族（for/key/memo/if/link/outlet/文本/fstr）、style 绑定、条件 DSL 捕获、事件修饰/取值/参数族 |
| `store.rs` | store 声明（msg/model/computed/watch/on/timer 复用同 crate块算法） |
| `scene.rs` | Godot scene 声明族（任意 scenario 生效，Plan 306）与 scene 门 lookahead |
| `actions.rs` | 顶层 actions/timer/routes/menubar/toolbar 与 style recipe 声明 |
| `helpers.rs` | 纯 helper（近失距离、事件名提取、await 检测、主属性表、fstr 模板投影等） |

生产入口（全部真实消费本实现）：`dialect/ui.rs` 薄 adapter（widget/msg/
model/component/actions/style/view/on）、`parse_stmt_inner` 直连段（scene
门/store 门/`pub style`）、Parser 同签名薄委托门面、import 新建 Parser 的
`with_session` 默认装配。msg/model 声明复用 806 `declarations.rs` 单源。

## 2. UiParseContext 端口（对象安全子 trait）

`UiParseContext: ParseContext`（定义在本 crate——UI 伴随能力**不入**
auto-parser-api Core 包，既有 `ParseContext` 实现面零改动）。由 auto-lang
`ParserBridge` 实现；文法函数签名统一 `fn(cx: &mut dyn UiParseContext)`。
能力面与事务合同（冻结于 `docs/plans/attachments/810/contract-baseline.md`
§2/§3）：

- **游标**：801 端口原面 + `prev_pos`/`expect_ident`（Core 私有 helper 的
  端口投影，单一实现）/`cur_is_soft_ident`。
- **前瞻**：`probe_next_kind`（806 冻结，零副作用/失败消费协议）；
  `probe_next_token`（全 token 形态，失败**上抛**——memo/paren `?` 位点，
  if-let-Ok 位点用 `.ok()`）；`advance_peek_kind`（注释跳过前看的具名化：
  完整 advance 副作用 → 观察 kind → 回推观察 token 并恢复 cur，prev 保持
  泄漏——scene 门/subresource/on stream 三位点旧臂共用形态）；
  `push_back_token`（双 token 回推再重同步）。
- **投机**：`raw_cursor_save/restore`（lexer+cur+prev，**无**五旗标、
  **不**回滚诊断/docs——与 checkpoint 的区别：object() 投机失败泄漏旗标
  路径行为保持；凭据复用 801 句柄表，一次性/作用域失效同规则）。
    checkpoint/restore（含旗标）仍归方言派发器 None 回滚专用。
- **事务**：`diagnostics_mark/truncate`（**仅限** raw 投机回退位点——
  computed object→body；其余路径诊断恒单调）；`set_skip_check`/
  `set_in_on_body`（保存-置位-恢复三段式由文法复刻，Err 平衡恢复保持）。
- **raw CSS**：`capture_raw_block`（前置条件=cur 为 LBrace 且 lexer 无
  缓冲；返回 owned 字节，闭 `}` 由捕获消费；嵌套括号/字符串/注释/换行/
  Unicode 由 Lexer 保持）。
- **scope/绑定**：`enter_scope/exit_scope/bind_var/define_handler_param`
  （infer_ctx 符号面；不返回 TypeStore/InferenceContext 引用；绑定寿命/
  顺序/Err 出口与旧臂逐字一致）。
- **核心委托**：`parse_body/parse_stmt/parse_closure/parse_object_pairs/
  parse_fstr/parse_dot_item/parse_on_events`（单一份 Core 文法）。
- **副作用出口**：`take_docs`、`warn_suspicious_block_keyword`（span=当前
  token 位）、`register_style_recipe`（Core design_tokens 注册表）。

## 3. 支持与失败边界

- 支持=基面行为冻结：全部既有 UI 语法（含宽容怪癖：无点 pattern、`>`/
  `->`/`=>` 混写、近失告警、`__stream_sse_*` 合成 pattern、outlet 三态、
  quoted event、setup 拒绝 await 等）逐字保持；错误文本/span/顺序/恢复
  进度不变。
- 已知怪癖基线：msg/model 的无 EOF 守卫块循环（806 spec 在案，适用边界
  如实）；迁移闭包内未发现新的无界 EOF 形态。
- 失败边界（不得冒称）：expr/type/Pratt/作用域/模块解析算法仍归 Core
  （经端口委托）；View/On TokenKind 兼容 AST 未退（R10）；`scan_use_
  statements` 与真实 import 加载归语言所有者；发行闭包含 auto-lang 是
  组合宿主现状（R04 处理）。

## 4. 验证矩阵

- 真 Parser 面：`crates/auto-lang/src/tests/plan810_full_ui_grammar.rs`
  （13 测）+ 既有 plan024/051/367/408/451/607/806、native_css、plan339/
  340/370、widget/store/view/style/actions/scene 定向面 + ui_gen 四生成
  器金样（tu 档）与 VM 语料（tv 档，merge 冻结统一跑）。
- 轻闭包面：`test/fixtures/modularity/ui-grammar-consumer/`（toy
  UiParseContext 全能力实现，7 测；受限 raw CSS 端口=token 重建，字节
  保真由真 Parser 面钉住）；`verify-deps.sh`（闭包无 auto-lang/auto-man/
  auto-vm/auto-ui）；777 strict（module-boundaries `lang-core` 域零违规）。
