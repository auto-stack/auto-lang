# Scenario 与 UI 方言（条件关键字）

## 范围

编译器行为由项目场景（scenario）驱动，而非全局语言特性。本主题覆盖 `pac.at` 配置、`CompilerSession` 传递与 UI 关键字的条件激活机制。

## 机制

- **配置单一真源**：`pac.at` 声明 `scenario`（`ui`/`core`/`shell`）、后端目标与构建设置；LSP 初始化时读取它配置解析模式，诊断/hover/补全都尊重场景。
- **会话传递**：`session.rs:CompilerSession` 携带 `Scenario` 贯穿管线。
- **dialect 注册**：`dialect/ui.rs:UiDialect` 实现 `Dialect` trait，`matches()` 在 `scenario == Scenario::UI` 时生效。

## 关键字接管细节（以代码为准）

- `widget`/`msg`/`model` 是**普通标识符**（`TokenKind::Ident`），UI 场景下经 `Dialect::try_parse_stmt` 接管为声明语句。
- `view`/`on` 是**真实 TokenKind**，走 `try_parse_token_stmt`；`view` 与 core 语言的参数模式关键字（`fn foo(view x int)`）复用同一 token，靠语句位置区分。
- `view fn` 前缀为 view fragment（plan-367 P2-3）——内联展开，调用点无独立组件。
- `component fn`（plan-408）自 Plan 425 起为 **`widget` 的语法糖**：解析期直接产出 WidgetDecl（body 自动包 `view` 块、params→props），fragment 双轨已删除。**新代码请用 `widget`**；`component fn` 仅作兼容拼写保留。同文件 widget 引用自动走组件路径（`<Name/>` + `@/components/Name.vue`）。
- `widget` 体支持 **view 可选化**（Plan 425）：体以视图元素开头（无 `view` 块）时体即视图，自动包裹——`widget X { col {...} }` ≡ `widget X { view { col {...} } }`。
- `setup { ... }` 前导槽（Plan 426）：每实例同步执行、先于首渲染的通用 setup 语句槽（`let`/表达式;`await` MVP 拒绝）。`refs <binding>: [f...]` 块级声明标注 ref 字段（script 侧访问注入 `.value`）。**`use.web composable` kind 降级为糖**——自动调用 + refs 标注可由 `setup { let x = useX() }` + `refs x: [...]` 完整表达,新代码推荐 setup 块。

## 生命周期三相位语义表（Plan 426 定版）

| 相位 | 语法 | 执行时机 | a2vue 映射 | 适用 |
|---|---|---|---|---|
| **setup** | `setup { ... }` 块 | 每实例同步,先于首渲染,state/computed 定义之前 | `<script setup>` 顶层语句 | composable 调用（保 inject 语境/内部 onMounted 注册）、任意同步初始化 |
| **.Init** | `on { .Init -> {...} }` | 每实例挂载后,首渲染之后 | `onMounted(() => {...})` | DOM 测量、需要已挂载节点的初始化 |
| **.Destroy** | `on { .Destroy -> {...} }` | 每实例卸载时 | `onUnmounted(() => {...})` | 清理（订阅/定时器/监听） |

约束:setup 中 `await` 明确报错（async setup 需 Suspense 边界,另立任务）;setup 绑定与 model 变量/prop 同名为编译错误;setup 绑定 = script-setup 顶层局部绑定（≠ model 变量,不进 defineModel/不可被父绑定;需要双向时用 model 声明 + setup 内初始化组合表达）。解释器侧（AutoUI 继承 AutoVM）的每实例执行约定登记后续（auto-ui interpreter 联动）。

## 不变量

- core 场景下 `let widget = create_window()` 必须合法——UI 关键字零命名空间污染（ADR-03）。
- 场景判定只读 `CompilerSession`，解析器本身不硬编码 UI 规则。

## 演进记录

docs/design/08 描述的是"parser 直接检查 session 提升上下文关键字"；现实现已重构为 dialect 机制（docs/design/dialect-extension-diagnosis.md §6.1，`dialect/ui.rs` 头注）。以代码为准。

## 显式非目标

- 不为每个场景发明新语法——dialect 只接管既有标识符的语句位解析。
- LSP 的 scenario 同步细节不在本主题（属 LSP 模块）。

> 来源: docs/design/08-ui-systems.md §Scenario-Based Programming；crates/auto-lang/src/dialect/ui.rs；docs/design/dialect-extension-diagnosis.md §6.1
