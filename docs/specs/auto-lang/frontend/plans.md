# frontend 相关 plan 索引

> 状态以 plan 文件自身标注为准；未标注的写"未标注"。归档列为当前位置
> （`plans/` 活跃，`old/` 旧归档）。
> 编号注意：docs/plans/old/013-unify-args-and-props.md 与 docs/plans/013-auto-ai-port-to-auto.md
> **重号但不同题**——本模块相关的是 old/ 的 013（args/props 合一），plans/ 的 013 是
> auto-ai 移植，与本模块无关。重编号批次（327/336/337/338/342/351/355/359 →
> 317/318/320/322/330/346/347/348）均不涉及本模块。

## AST / Atom 序列化（来源 plan-report 01）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 001 | vm-function-integration | ✅（验收项全过；文首仍写 Planning） | old/ | `use` 命中 VM 模块时懒加载注册，函数调用走 per-universe 缓存派发 |
| 002 | to-atom-ast | ✅ | old/ | 全 AST 的 `to_atom()` 与 markdown 对比测试格式（输入 + `---` + 期望） |
| 003 | to-node-trait-refactoring | ✅（plan-report 01；文件尾仍写 Ready） | old/ | 拆出 `ToNode` 直返 `Node`，消除 42 处 unwrap |
| 004 | to_atom_refactor_plan | ✅（plan-report 01） | old/ | `ToAtom` 收窄为文本序列化，返回 `AutoStr` |
| 005 | to-atom-text-refactor-plan | ✅ | old/ | `AtomWriter` 流式写 S 表达式，免中间字符串 |
| 006 | fix-atomwriter-implementations | ✅（plan-report 01） | old/ | 7 类格式对齐手写期望；结构体构造器靠首字母大写启发式 |
| 011 | auto-atom-refactoring | ⏳ Planning | old/ | auto-atom 生产化路线：AtomError/查询 API/JSON/schema |
| 012 | node-refactoring-indexmap | ✅ COMPLETED | old/ | NodeBody/Obj 迁 IndexMap：O(1) 查找 + 插入序 |
| 013 | unify-args-and-props | ⏳ Planning | old/ | 用 `num_args` 边界计数器把 args 并入 props IndexMap（重号见上） |
| 014 | unify-body-nodes-kids | ⏳ | old/ | `body`/`body_ref`/`nodes` 三字段合一为 `kids: Kids` |
| 015 | atom-builder-api | ✅ 完成 | old/ | 链式 `with_*` + Builder 两层构造 API（~735 行，77 测试） |
| 016 | atom-macro-dsl | ✅ 已完成 | old/ | `value!/atom!/node!` proc-macro 复用 AutoLang parser，`#{var}` 插值 |

## 语法 / parser

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 026 | property-keywords | ✅ COMPLETE | old/ | 属性关键字（`.view` 系复合 token）落地 |
| 056 | dot-expression-field-access | ✅ 已完成 | old/ | `.field` 点表达式与字段访问解析 |
| 058 | type-alias-syntax | ✅（任务逐项 ✅） | old/ | `type X = Y` 类型别名语法（`Stmt::TypeAlias`） |
| 060 | closure-syntax | ✅ Complete | old/ | `x => expr` / `(a, b) => expr` 闭包（`Expr::Closure`） |
| 090 | remove-universe-from-parser | ✅ 完成 | old/ | Parser 去 Universe：符号入 TypeStore，辅助入 parser_helpers.rs |
| 121 | task-msg-system | ✅ COMPLETED | old/ | `task` 关键字解析为 `Stmt::TaskDef` |
| 156 | unified-enum-migration | ✅（Phase 1 起逐项 ✅） | old/ | 统一 enum AST：`EnumKind` 区分标量/异构枚举 |
| 162 | method-keyword-to | 文首"待实现"，代码已实现 | old/ | `.as(Type)`/`.to(Type)` → `Expr::Cast`/`Expr::To`（文档过时） |
| 228 | hetero-enum-tuple-syntax | ✅ 已完成 | old/ | 异构 enum 多参数变体强制括号元组语法 |

## 模块 / use

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 078 | automan-integration | ✅ COMPLETE | old/ | `ModuleResolver` trait 由来：解析策略可插拔（Stage 2） |
| 092 | rust-ffi-sandbox | ✅（Phase 1-6） | old/ | `use.rust` 导入形式进 use_scanner |
| 106 | router-use-syntax | 未标注 | old/ | router 场景的 use 语法改进 |
| 131 | module-path-syntax-design | ✅（2025-03-18 逐项 ✅） | old/ | `super`/`pac` 前缀 + `ModulePath`/`PathPrefix` AST |
| 167 | module-system | ✅ 已完成 | old/ | 模块系统完整实现，含 `pub use` |
| 184 | cross-module-function-calls | 未标注 | old/ | 跨模块函数调用解析与派发 |
| 214 | python-ffi-use-py | ✅ COMPLETE | old/ | `use.py` 导入形式进 use_scanner |

## 活跃 plan（plans/）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 325 | autovm-enum-method-and-cross-module-bugs | 进行中（无完成标记） | plans/ | enum 方法调用与跨模块字符串缺陷，阻塞后端 Auto 代码 |
| 332 | derive-to-atom-proc-macro | 设计草案，待评审 | plans/ | `#[derive(ToAtom)]`/`FromAtom` 标注驱动 .at 序列化 |
| 367 | codegen-quality-improvements | 进行中 | plans/ | 含 view fragment 语法（P2-3）落入 parser（`Stmt::ViewFragmentDecl`） |
