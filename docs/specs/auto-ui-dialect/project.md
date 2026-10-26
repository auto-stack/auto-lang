# auto-ui-dialect

> **Status**: active
> 路径：auto-ui 仓 `crates/auto-ui-dialect`  | 技术栈：Rust
>
> UI owned AST、完整 UI 文法、版本化 codec 与领域组合端口的单一来源。

## 模型与依赖

本 crate 拥有 `UiStmt`/`UiCode`、widget/store/actions/view fragment/style/
msg/model/view/use.web 类型、`ViewNode`/`ExtImport`、跨文件模块 AST、
visitor 与序列化实现。`SceneDecl`、Core on-events/task/msg、普通 `msg`
标识符与 `Expr::View` 借用不属于 UI 私有模型，留在 Core。

normal 闭包仅 auto-parser-api 及其轻层依赖；expr/type/body 等 Core 文法经
`DomainParseContext` 委托，不复制 Pratt、类型文法或 Parser。独立消费门为
`ui-grammar-consumer`、`ui-ast-model-consumer` 与上层
`ui-dialect-consumer`。

## 文法与所有权

`UiDialect` 实现 `DomainDialect<Output=UiStmt>`，完整处理
widget/store/component/view/fragment/on/actions/style/timer/routes/menubar/
toolbar/use/setup/expose/watch/computed/bind。msg/model 的 Builtin 与显式
ExternalSubset 策略共用单一字段算法；ExternalSubset 仍允许裸字段并让行
注解，Builtin 不因此扩大。

`Parser::parse_domain(&UiDialect)` 直接产出 `UiCode`，无默认进程级文法表、
inventory 装配或 Core `Stmt` UI 兼容臂。旧 Parser UI 返回门面与
`ParserBridge::parser()` escape hatch 均不存在。fn/on 体内 UI 关键字让行
Core；on-handler 无论成功或错误都恢复 on 旗标并退出参数 scope。

## codec、投影与寿命

- `UI_AST_SCHEMA_VERSION` 信封同时校验 domain/schema/tag；未知 domain、
  旧版本与坏 tag 在 decode/compile 前具名拒绝，不把旧字节当新模型。
- 缓存身份包含生成器指纹与 AST schema；旧缓存失效并重新生成。真实解析
  的 span、源行、字段与跨文件模块 owner 经 roundtrip 保持。
- `UiAstVisitor` 递归覆盖 UI/View/Core 节点；Clone/解码对象随 `UiCode`
  owned 移动，Weak 控制证明最后 owner 退出，无全局对象表或别名悬空。
- Display/ToAtom/ToNode 延续冻结 golden，包括 `UseWeb` 的单复数和
  `ToAtom = "use.web"`。同一 PLAN-799 golden 的 9 个 UI 段由
  `ui-ast-model-consumer` 永久重放。

## 历史错误边界

`msg { Inc` 未闭合到 EOF 的无界怪癖不因迁移冒称已修复；未闭合 fragment
的隔离探针也只能经外部截止时间执行。合法输入继续沿用 Parser 的
body_depth/既有截止，不新增任意限额。
