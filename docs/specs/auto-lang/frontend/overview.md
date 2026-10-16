# frontend（词法/语法/AST）

> **Status**: implemented（核心管线完整；方言体系 PR-2 已落地；`.?`/`.*`/`.@` 等符号属性、位操作、Auto Flow 未实现）

## 职责

源码 → AST：词法分析（含 f-string 插值 `$var`/`${expr}`）、递归下降语法分析、
方言（dialect）场景化关键字派发、`use` 语句快速扫描与模块路径解析。
产出 `Code`/`Stmt`/`Expr` 树，供 AutoVM、trans 多目标转译（C/Rust/TypeScript/
Python/JavaScript/GDScript 等）与 aura/a2ui UI 管线消费——旧 TreeWalker 求值器
不是消费方（Plan 091 起 `ExecutionEngine::Evaluator` 仅重定向 AutoVM，
execution_engine.rs 头注；docs/design/01 §Compilation Pipeline）。

## 现状

- 管线为 `Lexer → Parser → AST`。PLAN-789 起词法层外置为底层 crate
  **auto-lex**（`crates/auto-lex`，Token/TokenKind/Pos/Lexer/LexerState 单一
  实现，依赖仅 auto-val+诊断基建）：auto-lang 的 `token`/`lexer` 模块是薄
  re-export 出口（同一类型身份），`error.rs` re-export `LexerError` 并以
  `From<LexError>` 等价投影（结构化→`AutoError::Lexer`、raw-block 字符串→
  `AutoError::Msg`，旧臂一致）；`Lexer` 在本 crate 仍不公开（`mod lexer`
  私有，外部消费者直接依赖 auto-lex）。规范见
  [auto-lex/project.md](../../auto-lex/project.md)。
- **未迁事实**（不得冒称完整前端独立）：`Parser` 仍持有 `Lexer`（经兼容
  出口）、`TypeStore`、`InferenceContext`、`ModuleTracker`、session 与方言表，
  不再依赖 `Universe`（plan-090）；`error.rs` 依赖 `ast::Name`（TypeError
  变体）阻碍错误层整体下放——残留边与 owner 见
  `docs/plans/attachments/789/t01-frontend-dag.md`（下一批 UI 文法外置）。
- 方言体系（轴 A）已实现：`Dialect` trait + `UiDialect`（`Scenario::UI` 下接管
  `widget`/`msg`/`model`/`view`/`on`），产出仍是基础 `Stmt` 变体；PLAN-778 起
  支持外部 crate 经 `ParserBridge` 显式注册方言（见"方言外部化"节）。
- `use` 体系两层：预处理用 `scan_use_statements`（字符串级，免解析）；
  解析后用 `ModulePath`（`pac`/`super` 前缀）+ `ModuleResolver`/`FilesystemResolver` 落盘。
- AST 序列化三件套 `ToNode`/`ToAtom`/`AtomWriter` 覆盖全部 AST 类型（S 表达式文本）。
- `.as` / `.to` 点属性已在 parser 实现（`Expr::Cast`/`Expr::To`；复现：
  `rg -n "Expr::Cast|Expr::To" crates/auto-lang/src/parser.rs`，2026-09-07 快照命中
  2714/3027/3037 行），早于 docs/design/10 的"未实现"描述（见分歧记录）。
- null 术语统一与生产者门控（plan-550，脚本模式 W0）：`nil` 拼写退役为
  `null` 的 deprecated 别名（literal/atom 双臂发 W0005 DeprecatedFeature，
  语义不变——运行期同落 PUSH_NIL/encode_null；CLI 直跑路径在
  lib.rs `execute_autovm_with_path` 可见化 parser 警告，stderr 按名一次
  性去重）；`#[script]` 文件级 pragma（annotation match `script` 臂登记
  `parser.script_pragma` → `CompileSession.script_marked`，不挂声明不改
  行为）；生产者门控 lint——无 pragma 文件含 `use.py`/null/nil 字面量
  （`parser.saw_bare_null`，None/Some 不计入）→ stderr 迁移提示（.as 指
  引，只警告不拒绝）。`.as` 扩展名与模式管线归 W1。
- 脚本方言模式解析（plan-555 W1）：`mode.rs` `ScriptMode` +
  `resolve_script_mode` 八格矩阵——`.as` ≡ 隐式 `#[script]`，`#[rust]`
  文件级 pragma 显式压回（parser `rust_pragma`，annotation "rust" 臂），
  优先序 `#[rust]` > `#[script]` > 扩展名；`CompileSession.script_mode`
  回填（550 `script_marked` 派生兼容），门控 lint 按 ScriptMode 统一判定
  （.as 自动豁免）。W1 语义 passthrough（脚本语义激活在 W2 lowering 批）。
- W2 语法糖批（plan-560）：`.as` 执行翻转为 **lower→compile**（s2s 改写
  管线激活）；`with` 上下文管理器关键字（无 as 形态直产 py_with 调用；
  as 形态与 Cast 中缀歧义响亮拒绝 P560-D1）；`**` Power token + `@`/`**`
  中缀直产 py_matmul/py_pow 桥调用；`is` 中缀→py_is；`#[with(...)]` 注解名
  撞 With 关键字收位修复。
- 收官波 with-as 绑定（plan-567，P560-D1 销号）：with 头部窗口旗标
  `with_header`——pratt 循环内 `as` 为终止符、Cast 臂截断（`with e as x`
  绑定位由 with_stmt 消费；正常模式 Cast 语义零变化）；as 形态直产块形态
  `Stmt::Expr(Expr::Block)`（绑定+出口保证，躲 convert_last_block 尾块
  转换）；convert_last_block 收窄为纯 pair 块才转对象。
- a2py 语义修补批（plan-598）：py_call 糖族**恒括号纪律**七臂
  （py_call/py_call_may/py_getattr/py_matmul/py_getitem/py_setitem/py_call0
  接收者恒括号发射——复合接收者原裸拼接致 `.sum()` 绑到末操作数，P539-D3
  根治；getattr_may/getitem_may 原本安全不改）；语句体闭包分类——单表达式
  块（含裸 `return e`）lambda 化（原 `lambda x: {...}` set/dict 字面量
  静默错类型），含绑定/多语句块显式编译期诊断（DIV-PY-CLOSURE-1 a2py 面）。
- py 类派生工厂 a2py 轨（plan-602）：`py_subclass` 发射臂 →
  `_auto_subclass(name, base, methods)` + helper 注入
  （needs_subclass_helper 旗标族，同 `_auto_may` 机制）——exec+setattr
  同构实现：Str 值缩进归一（非空行统一 +4，与 VM 臂同律），callable 值
  经对象字面量既有 lambda 发射、setattr 为真函数（self 绑定语义与 VM
  def 包装器对齐）；语料 py_torch_subclass 双轨 stdout 一致（p12 三轨
  4/4）。

## 关键入口

- `crates/auto-lex/src/lexer.rs:Lexer`、`lexer.rs:LexerState`（PLAN-789 单一
  实现；auto-lang 经 `src/lexer.rs` 薄 re-export 私有消费）
- `crates/auto-lex/src/token.rs:TokenKind`、`token.rs:Token`、`token.rs:Pos`
  （auto-lang 经 `src/token.rs` re-export，`auto_lang::token` 公开路径不变）
- `crates/auto-lex/src/error.rs:LexerError`/`LexError`（auto-lang `error.rs`
  re-export + `From<LexError> for AutoError` 投影）
- `crates/auto-lang/src/parser.rs:Parser`、`Parser::parse`、`Parser::parse_stmt`、`Parser::parse_expr`
- `crates/auto-lang/src/parser.rs:Parser::build_dialects`、`Parser::try_dialect_stmt`
- `crates/auto-lang/src/parser_helpers.rs:ModuleTracker`、`parser_helpers.rs:LambdaIdGenerator`
- `crates/auto-lang/src/ast.rs:Code`、`ast.rs:Stmt`、`ast.rs:Expr`
- `crates/auto-lang/src/ast.rs:ToNode`、`ast.rs:ToAtom`、`ast.rs:AtomWriter`
- `crates/auto-lang/src/ast/module_path.rs:ModulePath`、`module_path.rs:PathPrefix`
- `crates/auto-lang/src/dialect.rs:Dialect`、`dialect/ui.rs:UiDialect`
- `crates/auto-lang/src/parser_bridge.rs:ParserBridge`（PLAN-778 方言外部化最小桥面）、
  `dialect.rs:DialectPriority`
- `crates/auto-lang/src/use_scanner.rs:scan_use_statements`、`use_scanner.rs:UseStatement`
- `crates/auto-lang/src/resolver.rs:ModuleResolver`、`resolver.rs:FilesystemResolver`
- 一键入口：`crates/auto-lang/src/lib.rs:parse`（`parse(code) -> AutoResult<Code>`；
  复现：`rg -n "pub fn parse\(" crates/auto-lang/src/lib.rs`，2026-09-07 快照 2490 行）

## 使用示例

```rust
// 整段源码解析（lib.rs 公开入口）
let code = auto_lang::parse("fn add(a int, b int) int { a + b }")?;

// 预处理扫描 use 依赖（不解析全文）
let uses = auto_lang::use_scanner::scan_use_statements(source);

// 模块路径解析
let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));
let path = resolver.resolve_with_prefix(&module_path, current_file)?;
```

## 方言外部化（PLAN-778 pilot）

方言外部化（PLAN-778 pilot）：`Dialect::try_parse_stmt/try_parse_token_stmt` 接收最小
`ParserBridge`。桥面提供 token 游标、checkpoint/restore、核心表达式/类型解析、场景与
fn/on 体上下文查询、当前位置语法错误工厂；不暴露 Parser 本体、lexer、scope、符号表或
TypeStore，不提供通用 warning 出口。

`Parser::register_dialect/with_dialect` 显式注册外部方言。`Override` 注册位于内建段
之前，`Append` 位于其后；各段按注册顺序咨询，首个返回 Some 的匹配方言接管。
`with_session` 只重建内建段，外部注册保持；派发时检查 matches 场景门控。Arc 共享句柄
使外部委托核心解析时，合法递归语句派发仍能看见完整注册表。

派发器在 Ok(None) 后回滚 lexer/cur/prev 游标与 in_on_body/in_fn_body/in_dot_rhs/
with_header/body_depth 五个上下文旗标。Err 时注册不丢失，恢复这五个旗标，游标保留
失败位置；errors/warnings 单调追加。checkpoint 不承诺符号表等全部解析副作用的事务回滚。

独立外部 consumer 已验证无注解的 `model { [var] name [type] [= expr] }` 字段子集：
类型和初值可同时省略；裸字段在行界/块界形成字段，类型 Unknown、初值 Nil；EOF 后仍须
满足 model 块闭合。类型或初值存在时委托核心解析。字段注解走 None 回退内建；其他 UI
文法保持内建，UI AST/consumer 继续兼容。

裸字段支持属于显式注册的外部 pilot，默认内建 model 仍要求类型或初值，行为不变。生产
入口和导入中新建的 Parser 不自动继承外部注册；需要各入口显式注入。widget 体内 model
由结构解析器直连内建，不经过语句方言派发；widget/msg/component/actions/style/view/
on/store 的完整外部化、生产接线与 UI AST 迁移尚未完成。

## 已知坑

- `parser.rs` 超 20k 行（`wc -l crates/auto-lang/src/parser.rs` = 20562，2026-09-07 快照），
  语句/表达式/类型解析高度集中，改动需跑全量测试。
- 方言派发不再用 `mem::take` 移表（PLAN-778 已废除该模式——Err 提前退出会永久丢
  失整张方言表，且委托递归期间活跃方言不可见）：方言表元素为 Arc 共享句柄，派发
  克隆调用、结构上无移出；Err 边界恢复五旗标、None 回滚游标+旗标（见"方言外部化"
  节）。新增方言走 `register_dialect`/`with_dialect` 显式注册，勿恢复移出式派发。
- `view` 在 Core 语言是参数模式关键字、在 UI 场景是语句关键字——同一 TokenKind 两义，
  由 `try_parse_token_stmt` 按 session 场景分派。
- `scan_use_statements` 是字符串匹配：字符串字面量内的 "use xxx" 行、条件编译分支中的
  use 都会被算入，仅适合依赖预估，不能当解析结果用。
- `use_scanner` 支持 `use.rs`/`use.py`/`use c <h>` 等 FFI 导入形式，但去重按 module 名，
  同名不同 items 的 use 只保留第一条。
- **use.rs 现行拼写（Plan 470，2026-08-30）**：Rust 导入解析分派 `rust`/`rs` 双拼写同走
  `use_rust_stmt`（AST 同为 `UseKind::Rust`，转译/VM 零差异）；旧拼写 `use.rust` 发
  W0005 DeprecatedFeature，scanner 依赖收集双前缀识别；移除触发条件见
  KNOWN-DEBT P470 deprecation 周期行。

## 蒸馏来源（Phase 1）

- `docs/design/01-architecture.md`、`docs/design/10-language-syntax.md`
- `docs/plan-reports/01-ast-core.md`、`docs/plan-indices/01-ast-core.md`
- `docs/design/dialect-extension-diagnosis.md`（方言体系来源，dialect.rs 头注指向）
- 代码核对：`crates/auto-lang/src/{parser,parser_helpers,ast,dialect,use_scanner,resolver}.rs`、`ast/`、`dialect/`、
  `crates/auto-lex/src/{token,lexer,error}.rs`（PLAN-789 词法外置）
