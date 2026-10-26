# PLAN-841 T-01 合同基线

2026-10-26；输入为 auto-lang `100172980efb0343fe7016a05eb40f18d0014d74`、auto-ui `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、auto-man `1b7a9ce0de9055f7953129d78a25777662508258`。G841 的 U831/U832 归档、终审、统一门、落地、ledger 与清理收据已在主计划核验。

## 1. 实际生产与消费图

| producer / owner | 当前数据或状态 | consumer / 终态要求 |
|---|---|---|
| auto-lex | `TokenKind::View` 同时服务 Core 不可变借用与参数模式；`On` 服务 Core on-events/task/msg 与 UI | 保留 Core 分类及合法对照；UI 语句只经领域方言接管 |
| auto-ast | `Code -> Vec<Stmt>`；`Stmt` 含 Widget/Store/Actions/ViewFragment/StyleRecipe/Msg/Model/View/Scene/UseWeb；`ast::ui` 持有 UI 树 | Core 仅保留通用 AST 与 `SceneDecl` 等真实非 UI 语义；UI 类型单源迁 auto-ui-dialect |
| auto-parser-api | `ParseContext`；`UiParseContext` 与 `UiGrammar` 直接暴露 UI 返回类型 | 改为中立 `DomainParseContext` 与关联产出 `DomainDialect`；不提供 UI 字段、全局类型擦除或 side-table |
| auto-lang-core ParserBridge | 实现 ParseContext/UiParseContext；style recipe 直接注册 UI decl | 仅实现中立端口；领域名称经通用 scope 绑定，UI 注册在 UI owner 完成 |
| auto-lang-core Parser | `parse() -> Code`；内建 UiDialect 和公开 UI 门面把 UI 变体写入 Stmt | 保持纯 Core `parse() -> Code`；新增泛型领域模块解析，直接产出 `DomainCode<D>`，不经过 Core Stmt |
| auto-ui-dialect | 完整 UI 文法，返回 Core `Stmt`/Msg/Model | 拥有 `UiCode/UiStmt/UiAst`、文法与 codec；泛型端口直接返回强类型 UiStmt |
| auto-lang-core infer/dep/indexer/trans/VM | 匹配共享 UI Stmt 臂；Godot tscn/gdscript 消费 SceneDecl | 删除 UI 专属匹配；Core 遇未 lowering 领域项具名拒绝；SceneDecl/Core on/task/msg 保留 |
| auto-ui-compiler | aura/style/action/registry 与 vue/jet/ark/rust 生成器消费共享 AST | 改为消费校验后的 UiCode/UiStmt；四生成器输出与源位置保持 |
| auto-ui-runtime | dynamic_build/dynamic/vm_bridge/aura_view_builder/renderer 消费共享 AST | UI 解析、校验和 lowering 后进入执行；VM 不接收未准入领域数据 |
| auto-man / 外部工程 | 经 auto-ui-tool 与 SDK 固定版本生成、构建、安装 | 仅经公开 UI 入口消费；三仓 source、锁、import 与发行身份一致 |

## 2. 逐语法分类

- **UI 专属**：WidgetDecl、StoreDecl、ActionsDecl、ViewFragmentDecl、StyleRecipeDecl、ModelBlock、ViewBlock、UseWeb，以及 `ast::ui` 的视图/事件/样式/导入模型。
- **需消费语义后判定**：MsgDecl 当前仅由 UI 文法生产并用于 UI 消息；Core task/msg 使用 TaskDef/OnEvents 等独立节点，不把字符串 `msg` 全量删除。
- **Core 保留**：SceneDecl 有 tscn/gdscript 非 UI 消费者；`Expr::View` 是不可变借用；`TokenKind::View` 还用于参数模式；OnEvents、TaskDef、普通标识符 `msg` 与 fn/on 体内同名变量保持。
- **最终删除**：Core AST/UI 兼容臂、Core UiGrammar/UiParseContext 与内建 UiDialect、低层 infer/dep/indexer/trans 的 UI 专属匹配，以及旧 Parser UI 返回门面。兼容只在 auto-lang 上层经 UI 新类型再导出。

## 3. 冻结机制与寿命

采用泛型领域模块，不采用序列化往返作为进程内传输：

```text
Parser::parse_domain<D: DomainDialect>
  -> DomainCode<D::Output>
  -> DomainStmt::Core(auto_ast::Stmt) | Domain(D::Output)

UiDialect::Output = UiStmt
UiCode = DomainCode<UiStmt> 的 UI 包装/投影
```

`DomainParseContext` 只含通用 token/checkpoint/raw cursor/scope/core expr/type/stmt 委托。`UiParseContext` 在 UI 仓作为该中立端口的 UI 使用面，不要求 Core 实现 UI trait。样式声明解析时用通用 `bind_var` 让后续表达式可解析；完整 recipe 注册、导入可见性和语义校验由 UI owner 执行。

 owned 数据随 `UiCode`/`UiStmt` 移动和 Clone；无全局 Any、裸指针或 AST side-table。持久 codec 使用显式 `UI_AST_SCHEMA_VERSION`，未知 domain/tag/version 在 decode/compile 前具名拒绝；旧 schema 不把字节当新模型读取。缓存生成身份新增该 schema 版本，旧缓存失效并重新生成。

## 4. 支持、错误与资源边界

- 保持现行 UI 语法、求值序、消息/绑定/样式、跨文件 import、四生成器、VM/Iced 和 Man CLI；不新增 backend、IR、ABI 或远端动作。
- Core 无 UI checkout 的 normal/build/dev 仍 parse/compile/VM；UI 公开入口未组合时具名拒绝。Core compile/VM 遇任何未 lowering 领域项具名拒绝，不静默跳过。
- 源文件、行号、ViewNode byte span、错误 span 保持；跨文件只从实际模块 owner 取 AST，不用全局缓存拼接。
- codec 只接受有界输入；递归沿用 parser body_depth/既有截止，不为合法程序新增任意限额。所有临时对象随调用栈 drop；进程级 registry 仅保留已存在且明确 owner 的 UI 状态。
- T-01 probe 的关闭条件：真实 widget/model/store/style/action + Core fn/借用/on + 跨文件 import 代表输入，经新泛型端口得到强类型 UI，校验/lowering 后类型/VM与至少一个真实生成出口成功；非法 domain/version 与未 lower 执行均拒绝。若无法保持，则 needs_replan，不交空壳。

## 5. 可执行 oracle 映射

| Case | 区分性输入与断言 |
|---|---|
| V-01 | ast-model/lex/parser-api/language-sdk 独立 consumer；cargo metadata/tree normal/build/dev 无 UI；Core fn/view borrow/on/task/Godot 实跑；UI 实现负控依赖被拒绝 |
| V-02 | UI 五族代表工程 + Core 同名对照；UiStmt 类型、错误、求值序与 golden；未 lower 项在 compile/VM 前具名拒绝 |
| V-03 | 跨文件、Clone、codec roundtrip、坏 tag/旧 schema、缓存重启与 drop；字段/span 相等，旧数据拒绝或迁移，资源最后 owner 退出 |
| V-04 | 真实 UI compiler/runtime 与 auto-ui-tool/Man；四生成器、VM/Iced、外部 fresh/locked/重定位；产物行为与 source 唯一 |
| V-05 | 原 ast-model/ui-grammar/language-sdk/generated-ui/tool-client 测试 ID 继续可选；反例先红后绿；版本/兼容清单交 842 |

## 6. T-01 实际 probe 记录

2026-10-26，泛型领域模块方案判定可行，未采用 Core 扩展 enum/Any/裸指针/全局 side-table 或 AST codec 往返。

- `cargo check -p auto-lang-core`（auto-lang worktree）：PASS；Core AST/Scene/类型/VM/转译无 UI 专属变体。
- `cargo check -p auto-ui-dialect`、`cargo check -p auto-ui-compiler`（auto-ui worktree）：PASS；UI owned 模型、文法、组合与 lowering 实编译。
- `cargo test -p auto-ui-compiler --no-run`：PASS，全部既有测试目标可构建。
- `cargo test -p auto-ui-compiler ui_ast::tests -- --test-threads=1`：代表测试通过。`style/actions/store/widget + Core fn/.view` 经 `parse_ui → lower_ui → Codegen → AutoVM` 实际输出 `1`；Rust/Vue/Jet/Ark 四生成器均有真实产物；`answers.at` 跨文件导入经 `collect_use_module_fns` 返回 `answer`，host 同时保留 Core Use 与 UiStmt::WidgetDecl。
- 区分性反例与最小对照：保留字 `shared` 的模块名仍按 Core 词法报错，不因新端口放宽；`.view` 借用保留，UI `view` 声明归 UiStmt；最小 store/widget 与完整五族均通过。

T-02 所需机制、owner、数据寿命、准入和真实生成/VM 路径已证明。codec schema/缓存/最后 owner 与 Man/发行矩阵仍归 T-06..08，不因本 probe 提前宣称完成。
