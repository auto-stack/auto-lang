# auto-parser-api

> **Status**: active
> 路径：`crates/auto-parser-api`  | 技术栈：Rust（依赖仅 auto-ast + auto-lex + auto-diagnostics）
>
> PLAN-801 交付：方言接口独立与 Parser 依赖倒置——外部文法实现只依赖本
> crate（轻端口/模型/会话）即可实现方言并委托核心解析；auto-lang 在上层
> 实现端口并接入同一生产派发表。本 spec 为该冻结增量的 canonical 沉淀。

## 模块与依赖

`crates/auto-parser-api` 是方言端口合同的唯一实现：`src/session.rs`（轻
`CompilerSession`/`Scenario` 单一定义，自 auto-lang 迁入）、`src/dialect.rs`
（`ExternalDialect` trait + `DialectPriority`）、`src/context.rs`（对象安全
端口 `ParseContext` + 不透明快照凭据 `ParseCheckpoint`）。normal 依赖闭包
仅 auto-ast（Stmt/Expr/Type 值）、auto-lex（Token/TokenKind/Pos）、
auto-diagnostics（AutoError/AutoResult）——无 auto-lang/Parser/lexer 实现/
TypeStore/推断/VM/UI/Man 反向边（module-boundaries 域 `lang-core`，规则
`lang-core-no-monolith-no-man-no-tools`）。轻层门面 re-export：`ast`/
`token`/`error` 模块名与旧 `auto_lang::{ast,token,error}` 对齐，外部方言
从 auto-lang 迁到本 crate 近似路径改名，类型身份即 799/789/793 单一实现。
外部消费门：`test/fixtures/modularity/dialect-api-consumer/` 独立 manifest
（PLAN-806 起依赖 auto-parser-api + auto-ui-dialect——model 子集方言包装在此，
字段算法单一份已委托 auto-ui-dialect `ExternalSubset` 策略，不保第二份）
构建+测试+
`verify-deps.sh`（cargo tree normal/build 无 auto-lang/auto-man/auto-vm/
auto-ui）即依赖倒置证据（V01）。

auto-lang 侧兼容出口：`crate::session`/`crate::dialect::DialectPriority`
为薄 re-export 门面（同一类型身份，旧路径/字段构造/跨 crate 类型互传
全部保持）；`ParserBridge<'p,'a>` 在上层实现 `ParseContext`（同名 inherent
方法零重复实现，泛型便利方法如 `syntax_error(impl Into<String>)` 留具体
门面，端口用对象安全 `syntax_error(String)`）。

## 端口与快照契约

- `ExternalDialect`：`matches(&CompilerSession)`/`keywords()`/
  `try_parse_stmt(&mut dyn ParseContext, keyword)`/`try_parse_token_stmt`
  （默认 `Ok(None)`）。三态语义与 PLAN-778 `Dialect` 完全一致：`Ok(Some)`
  接管、`Ok(None)` 让位（派发器强制回滚）、`Err` 报错（注册永不丢失）。
- 生产接入单表：`Parser::register_external_dialect`/`with_external_dialect`
  经 `dialect::external::ExternalDialectAdapter`（协议转换，不重复语法）把
  新接口方言推进**与旧 trait 同一张**三段有序派发表（pre→builtin→post），
  恢复不变量（Arc 注册永不出表/None 强制回滚/Err 边界恢复 context 旗标）
  由既有 `try_dialect_stmt`/`try_token_dialect_stmt` 兜底；注册跨
  `with_session` 保留。
- `ParseCheckpoint`（有作用域句柄，T-01 D3；F-801-R1-01 修订）：不透明凭据
  `{Parser 实例 id, 槽位, 全局单调 gen}`，后端为 Parser 内 `port_snapshots`
  句柄表（tombstone 槽位复用；每槽记录**签发它的方言调用作用域**，派发器
  每次调用唯一、嵌套保存/恢复）。跨上下文（实例 id 不符）/过期（调用退出
  按签发作用域失效——`port_release_scope` 置 tombstone，与槽位复用与否
  无关；已消费、gen 更替）使用在 `restore` 处**受控拒绝**（当前 token
  位置语法错误 Err，游标不动）；凭据 move-only 单用；外层存活快照不受
  内层调用退出影响（递归不覆盖外层快照）。无 unsafe 延寿、无源码 'static
  要求、无全局快照池。
- 原始单 token 探测 `probe_next_kind() -> Option<TokenKind>`（PLAN-806
  具名通用能力，F806-R2-01 引入/R3-01 定形）：成功路径零副作用/幂等
  （token 立即回推，游标与诊断不动，随后 `advance` 消费同一 token 并执行
  其全部副作用）；**失败路径遵循旧回推前瞻的失败消费协议**——是否推进
  输入由 Lexer 失败模式自身决定，错误不记录（返回 `None`），不承诺游标
  不变或连续调用幂等。实现方（auto-lang `port_probe_next_kind`）与旧
  `lexer.next()`+`push_token` 前瞻逐字同构；消费方为 UI 声明文法
  （auto-ui-dialect msg named-payload 判定）。实现方兼容性：成功路径必须
  满足零副作用/幂等/与首次 advance 一致；失败路径必须如实暴露失败消费
  结果，不得偷偷恢复状态或补记错误（那会改变声明文法旧错误语义）。
- 跨界数据面：只传 owned 文本（`cur_text() -> String`）与现有 AST 值；
  不返回 Parser/lexer/scope/TypeStore 引用。诊断单调追加（checkpoint 不
  回滚 errors/warnings，不承诺符号表级事务回滚）。

## 边界与已知限制

- 旧 `auto_lang::dialect::Dialect` + `ParserBridge` 公开面保持不变（PLAN-778
  冻结面零漂移；by-value `BridgeCheckpoint` 与端口凭据是两条独立机制，
  旧方言投机解析不经端口作用域清理）。内建 `UiDialect` 仍经 `pub(crate)
  parser()` escape hatch 使用完整 Parser——内建完整 UI 文法在 Parser 上，
  不在本 crate。
- 生产入口/import 新建 Parser 不自动继承外部注册（沿 778 冻结边界）；
  widget 体内 model 由内建结构解析器直连，不经语句方言派发。这些边的
  外置属后续"完整 UI 文法按族外置"合同（见 frontend/overview 剩余边）。
- 792 交付闭包：auto-parser-api 为 auto-lang 必需 path 依赖，
  `DELIVERY_CRATES` 已纳入（PLAN-806 起 13 成员，另含 auto-ui-dialect），
  受管交付清单/重定位/负控沿既有
  admission/staging 接口重验。

> 来源: docs/plans/801-dialect-api-dependency-inversion.md（SD-01）；
> 代码核对 crates/auto-parser-api/src/{lib,session,dialect,context}.rs、
> crates/auto-lang/src/{dialect.rs,dialect/external.rs,parser_bridge.rs,
> parser.rs}、test/fixtures/modularity/{dialect-api-consumer,ui-dialect-consumer}
