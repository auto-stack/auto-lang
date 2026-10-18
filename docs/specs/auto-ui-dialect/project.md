# auto-ui-dialect

> **Status**: active
> 路径：`crates/auto-ui-dialect`  | 技术栈：Rust（依赖仅 auto-parser-api）
>
> PLAN-806 交付：UI msg 与 model 声明文法正式外置——完整 msg/model/字段
> 注解算法自 auto-lang `Parser` 逐字迁出为单一份实现，经对象安全端口
> `dyn ParseContext` 驱动；默认 UI 生产路径（顶层派发与 widget/component/
> store 结构路径、import 新建 Parser 的默认装配）全部真实委托，Parser 原
> 算法删除、旧公开方法仅薄兼容委托。本 spec 为该冻结增量的 canonical 沉淀。

## 模块与依赖

`crates/auto-ui-dialect` 是 UI 声明文法（msg/model）的唯一实现：
`src/declarations.rs`（`parse_msg_decl`/`parse_model_block`/
`parse_model_block_subset` + `ModelGrammar`/`ModelSubset`）。normal 依赖闭包
仅 auto-parser-api（其闭包 = auto-ast/auto-lex/auto-diagnostics + auto-val
轻层）——无 auto-lang/Parser/lexer 实现/TypeStore/VM/UI renderer 反向边
（module-boundaries 域 `lang-core`；合法声明边恰两条：auto-lang→本 crate、
本 crate→auto-parser-api）。expr/type 沿端口委托核心文法
（`ParseContext::parse_expr/parse_type`），不复制 Pratt/类型文法。
外部消费门：`test/fixtures/modularity/ui-declarations-consumer/` 独立
manifest（依赖 auto-ui-dialect + auto-lex + auto-parser-api，消费方 toy
`ParseContext` 驱动真算法）构建+测试+`verify-deps.sh`（cargo tree
normal/build/dev 无 auto-lang/auto-man/auto-vm/auto-ui）即轻闭包证据（V01）。

## 文法合同（迁出冻结，未收紧/未扩展）

**msg**（`parse_msg_decl`，游标在 `msg` 上）：旧 `msg Name {` 名字读弃
（Plan 448 A——名字从未语义化）；variant 名可带引号（`MsgVariant.quoted`
合同性 emit 名）；多 payload（Plan 043 M5 #1）；named payload
`Change(value: str)`（Plan 435 挂账②，名字入 `payload_names` 逐位平行，
位置形态 None）；空行/逗号宽容；载荷列表缺 `)` 经 `expect` 报错
（UnexpectedToken，位置/文本与旧臂逐字一致）。
**已知历史怪癖（不修，适用边界如实）**：**缺块闭括号且源到 EOF**
（如 `msg { Inc` 后无 `}`）时变体循环无 EOF 守卫，旧臂即为无界形态
（不收敛到错误出口）——不得描述为已覆盖的错误恢复路径；载荷列表内
的 `)` 缺失不在此列（如上，`expect` 确定性报错）。

**model**（`ModelGrammar` 两策略共用一份块循环与字段算法）：

- `Builtin`（生产默认）：`#[primary]`/`#[Consume("k")]`/`#[Provide("k")]`/
  `#[NavParam("r")]` 注解（可多注解、`#` 后非 `[` 宽容返回、空注解跳过）；
  `var` 可变旗标（Plan 130）；`name [type] [= expr]`——**无裸字段**
  （`=` 缺省仍必 parse_type）；未知注解错误文本与 span 冻结
  （`SyntaxError::Generic`，消息含合法清单原文，span=当前 token 位）。
- `ExternalSubset`（778 冻结外部子集，**仅显式 pilot 策略启用**）：无注解
  （字段首遇 `#` → `NotClaimed`，调用方 checkpoint 回滚整块后 `Ok(None)`
  让行内建）；字段名必 Ident（否则 Err "model field name expected
  identifier, got '…'"）；**裸字段允许**（类型/初值同时省略，终止于行界/
  块界/EOF → ty=Unknown + init=Nil）；EOF 在循环守卫中。
  消费方：`test/fixtures/modularity/dialect-api-consumer/`（PLAN-801 起的
  model 子集方言；PLAN-806 起不保第二份字段算法，委托本 crate 子集入口）。

## 端口映射与等价性（PLAN-806 T-01 D1–D7）

- 关键字消费：复刻 `Parser::expect_ident`（纯文本比较、错误文本
  "Expected 'x', got 'y'"、当前 token 位 span）——**不用**桥面
  `expect_keyword`（其文本 "expected 'x'" 与 Ident kind 检查均不等价）。
- named-payload lookahead：端口原语 `probe_next_kind`（原始单 token
  探测，PLAN-806 具名通用能力；F806-R2-01 引入、R3-01 定形）——与旧
  `lexer.next()`+`push_token` 前瞻**逐字同构**（含词法失败路径）：
  成功路径零副作用/幂等（不跳注释 token、不收 pending_docs、不记词法
  错误、游标不动；Colon 命中后 `advance` 消费的正是被回推的同一
  token）；失败路径遵循旧回推前瞻的失败消费协议（是否推进输入由
  Lexer 失败模式决定、错误丢弃不记录、不承诺幂等——声明文法依赖该
  协议保持旧臂错误语义，如字符字面量非法转义在旧臂即为"消费失败
  前缀+错误丢弃"）。（曾以 checkpoint+advance/restore 实现，R2 复审
  证实其经 `Parser::next` 引入注释跳过/docs 收集/词法错误记录的副作用
  扩散与"注释分隔伪命名载荷"文法扩大，已废弃；成功/失败两路径协议
  均有永久回归钉住。）
- 错误出口：未知注解经 `ctx.syntax_error`（桥面实现与旧臂逐字同构）；
  `expect` 族经桥面走 `Parser::expect` 同一实现。

## 生产接线（auto-lang 上层，单源无第二份）

- 顶层：`dialect/ui.rs::UiDialect`（builtin 段）`"msg"`/`"model"` 分支 →
  `Parser::parse_msg_decl`/`parse_model_block`（pub 兼容委托，签名与
  `Stmt::MsgDecl`/`Stmt::ModelBlock` 包装不变）→ 本 crate。
- 结构路径：`parse_widget_decl`/`parse_store_decl`/`parse_component_fn_decl`
  体内 `parse_msg_decl_inner`/`parse_model_block_inner` 一行建桥委托
  （`ParserBridge` 实现 `ParseContext`）；`parse_model_field_annotations`
  自 Parser 删除。
- import：新 Parser 经 `with_session`/`build_dialects` 默认装配（仅
  `Scenario::UI` 推 `UiDialect`），主文件与被导入文件同源；默认装配不是
  用户 Override 注册（三段表 pre/builtin/post 与注册传播不变，PLAN-778
  冻结）。Core 护栏保持：fn/on 体内 `msg`/`model` 是标识符用法
  （`UiDialect::try_parse_stmt` 前置 `in_on_body/in_fn_body` 让行）。

## 边界与未迁清单（不得冒称完整 UI 文法已拆出）

未迁（仍在 auto-lang Parser，另立计划）：`widget` 壳与 view/on/actions/
style/computed/setup/expose、store/component 壳文法、`use`/import 扫描、
`ParserBridge` 的 `pub(crate) parser()` escape hatch（UiDialect 其余分支
仍经其取回完整 Parser）。auto-lang 依赖本 crate 的组合宿主剩余边已按
module-boundaries 声明（lang-core 域内合法边），后续 Parser/组合宿主
分层再清理。
