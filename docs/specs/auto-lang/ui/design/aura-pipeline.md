# AURA 提取管线

## 范围

从 `WidgetDecl` AST 到 AURA IR 的提取、校验与分发机制。对应代码 `crates/auto-lang/src/aura/`（7 模块：atom/extract/schema/schema_loader/types/validate/mod）与 `schema/aura.at`。

## 设计原则

结构与逻辑绝对解耦。AURA 只从 widget 声明中提取三个纯元素：

1. **视图树**（UI 骨架）：纯布局 + 绑定，无逻辑。
2. **状态定义**（响应式模型）：带类型的状态签名与默认值。
3. **事件处理器**（消息路由）：以 `LogicPayload` 原样保留——`AstBlock` 供 AOT 后端，`Bytecode` 供 VM 动态执行。

## 机制

- **解析**：parser 产出原生 `WidgetDecl` 节点（不做脱糖）。关键字激活见 [scenario-dialect](scenario-dialect.md)。
- **提取**：`extract.rs:extract_widget_from_decl` / `extract_view_tree` / `extract_store_from_decl` 把 `model`/`view` 提取为 1:1 无损 AURA 结构（`types.rs:AuraWidget`、`AuraStore`、`AuraNode`）。
- **校验**：`schema/aura.at` 用 Auto 自身语法定义元素/prop 约束（`PropType`/`PropDef`/`ElementSchema`），`schema_loader.rs:SchemaLoader.load` 载入为 `schema.rs:AuraSchema`，供编译期校验与 LSP 补全；`suggest_similar` 支持拼写纠错。
- **分发**：AURA 馈入目标生成器（a2vue/a2jet/a2ark/a2lvgl 规划）或 VM 渲染路径（`ui/vnode_converter.rs`）。

## 关键数据结构

| 类型 | 位置 | 作用 |
|---|---|---|
| `AuraWidget` | aura/types.rs | 一个 widget 的完整 IR（view_tree/model/routes/props） |
| `AuraRoutes` / `AuraRoute` | aura/types.rs | 路由表（见 [router](router.md)） |
| `AuraStore` | aura/types.rs | 跨 widget 共享 store（与 AuraWidget 同构减 view/routes/props） |
| `LogicPayload` | aura/types.rs | handler 载荷：`AstBlock` \| `Bytecode` |
| `AuraApp` | aura/types.rs | app 级容器（router Phase 3 方向） |
| `AuraSchema` / `ElementDef` / `PropDef` | aura/schema.rs | 元素类别（`ElementCategory`）与 prop 类型（`PropType`）校验表 |

## 不变量

- model/view 提取 1:1 无损——AURA 不得静默丢弃源码信息。
- handler 在提取期不转译，保持 AST/字节码原貌（ADR-02）。
- 语法糖（`center`、primary prop 简写、trailing style、content-first）在解析/提取层展开，后端只见规范形。

## 显式非目标

- AURA schema 的运行时 vs 编译期强制边界未定（docs/design/08 Open Questions）。
- a2c+LVGL 的响应式策略（dirty-flag / 编译期跟踪 / 轮询）未定。
- Design Token Compiler 不在本管线内（`src/tokens/` 未实现；token 引用在 widget style 中的编译期解析是其设计目标）。

> 来源: docs/design/08-ui-systems.md；crates/auto-lang/src/aura/{types,extract,schema,schema_loader}.rs；schema/aura.at
