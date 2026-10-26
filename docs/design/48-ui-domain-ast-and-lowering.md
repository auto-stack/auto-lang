# Design 48：UI 领域 AST 与受检 Lowering

2026-10-26，PLAN-841。本文冻结 UI 领域模型与 Core 解析/执行边界的架构取舍；实际能力以模块 Spec 为准。

## 问题

Core `Stmt` 同时承载通用语言和 UI 声明，导致 AST、Parser API、类型推断、转译与 SDK 必须认识 UI 字段。Rust 上游 enum 不可由下游扩展，但复制完整 AST 或以 `Any`/裸指针/全局 side-table 绕过都会破坏单源、寿命和版本边界。

## 决策

Core 提供泛型领域解析端口，不提供 UI 容器或 UI trait：

- `DomainParseContext` 暴露通用 token、checkpoint、raw cursor、scope 与核心文法委托。
- `DomainDialect` 以关联类型返回领域语句；`DomainCode<D>` 保留 `Core(Stmt)` 与 `Domain(D)` 的顺序、源行和文件属性。
- `Parser::parse_domain` 直接返回强类型领域模块。普通 `parse` 仍只返回 Core `Code`，不接受未 lower 领域项。
- UI 仓拥有 `UiStmt`、`UiCode`、视图/消息/模型/样式/导入模型、文法、校验、codec 和 lowering。Core AST 删除 UI 兼容臂。
- `SceneDecl`、借用 `Expr::View`、Core `OnEvents`/task/msg 与参数模式 `view` 留 Core；不能按关键字文本归类。

## 准入与执行

UI 源码经 UI 方言得到 `UiCode`，校验声明关系、导入和 codec schema 后才可进入生成或执行。生成器直接消费 UiCode；VM/renderer 消费 lowering 后的通用执行表示与 UI 运行时数据。Core 类型推断、编译和 VM 对未 lower 领域项具名拒绝，不静默忽略。

进程内强类型传递不做序列化往返。持久 codec 有显式 schema/domain/tag/version，坏数据与旧版本不得按新模型读取；缓存身份包含 schema 版本。Clone、源位置与 drop 由 owned Rust 值自然继承，不使用全局对象表。

## 兼容与退役

旧 `auto_lang` 路径只在上层再导出 UI 新类型或提供具名适配，不在 Core 保留第二份模型。842 依据本 Plan 的适配清单退役过渡入口；稳定 Core 语法和三仓消费不得因清理而缩减。

## 验证

独立 Core consumer、Core 同名语法对照、UI 五族代表工程、跨文件/clone/codec/cache/drop、四生成器与 VM/Iced、Man/外部 fresh/locked/重定位共同作为验收。负控必须能区分：UI 实现混入 Core、坏 tag/version、未 lower 执行、错误 owner 与丢失源位置。
