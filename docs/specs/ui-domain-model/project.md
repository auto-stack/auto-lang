# ui-domain-model

> **Status**: active（PLAN-841 SD-05）
> 路径：auto-ui 仓 `crates/auto-ui-dialect/src/{ast,model,validation,codec}.rs`
> 关联：[auto-ui-dialect](../auto-ui-dialect/project.md)、[auto-parser-api](../auto-parser-api/project.md)

## 所有权与数据流

UI 领域 AST 由 UI 包唯一拥有，Core 只保留真正通用的 AST 与扩展/编译准入
契约。生产路径为：

`Parser::parse_domain(&UiDialect) -> DomainCode<UiStmt> -> UiCode -> 校验/lowering -> Core Code + UI owned 执行表示`

类型、VM、四生成器和 renderer 只消费该受检路径；未 lowering 的领域产物
在执行前具名拒绝。禁止 Core UI 兼容 enum、全局 Any/裸指针、AST side-table
或以字段改名把 UI 模型塞回 Core。

## 分类

- UI 私有：WidgetDecl、StoreDecl、ActionsDecl、ViewFragmentDecl、
  StyleRecipeDecl、MsgDecl、ModelBlock、ViewBlock、UseWeb 及其视图/事件/
  样式/导入模型。
- Core 保留：SceneDecl/Godot、Expr::View 借用、OnEvents、TaskDef、普通
  msg 标识符、fn/on 体内同名变量，以及 View/On 共享 Token 分类。

## 版本、源位置与寿命

codec 以 `UI_AST_SCHEMA_VERSION` 和 domain/tag 校验持久输入；旧 schema
明确拒绝并重新生成缓存，不能静默读绿。缓存身份包含 schema 与生成器
指纹。Clone、嵌套/跨文件 import、codec roundtrip 保持字段、源文件、行号
和 byte span；各模块只从实际 owner 取 AST。Weak/ drop 控制验证最后 owner
退出，无全局泄漏或别名悬空。

## 验证与过渡

V-01..V-05 分别覆盖 Core 独立消费、UI/Core 同名语义、codec/缓存/寿命、
真实四生成器与 VM/Iced/外部发行、原测试 ID 与失败归因。PLAN-799 的 UI
投影 golden 不由新实现重生成。旧全局文法装配和 Core UI facade 已在 841 退出；Aura `with_registry_and_imports` 旧入口及旧方言优先级测试适配在 842 完成退役和迁移，不影响保留的 DynamicComponent 导入能力。

## PLAN-841 来源与验证

冻结 UI 领域所有权、准入/lowering、版本 codec、源位置/生命周期与842上层过渡适配边界。 规范与验收来源：[PLAN-841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md)；复审及统一门证据见其归档记录。
