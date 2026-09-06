# frontend 相关 plan 索引

> 状态以 plan 文件自身标注为准；未标注的写"未标注"。归档列为当前位置
> （`plans/` 活跃，`archive/` 归档；历史 `old/` 目录已并入 `archive/`，下表 old/ 均按 archive/ 读）。
> 编号注意：docs/plans/archive/013-unify-args-and-props.md 与 docs/plans/013-auto-ai-port-to-auto.md
> **重号但不同题**——本模块相关的是 old/ 的 013（args/props 合一），plans/ 的 013 是
> auto-ai 移植，与本模块无关。重编号批次（327/336/337/338/342/351/355/359 →
> 317/318/320/322/330/346/347/348）均不涉及本模块。

## AST / Atom 序列化（来源 plan-report 01）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 001 | vm-function-integration | ✅（验收项全过；文首仍写 Planning） | archive/ | `use` 命中 VM 模块时懒加载注册，函数调用走 per-universe 缓存派发 |
| 002 | to-atom-ast | ✅ | archive/ | 全 AST 的 `to_atom()` 与 markdown 对比测试格式（输入 + `---` + 期望） |
| 003 | to-node-trait-refactoring | ✅（plan-report 01；文件尾仍写 Ready） | archive/ | 拆出 `ToNode` 直返 `Node`，消除 42 处 unwrap |
| 004 | to_atom_refactor_plan | ✅（plan-report 01） | archive/ | `ToAtom` 收窄为文本序列化，返回 `AutoStr` |
| 005 | to-atom-text-refactor-plan | ✅ | archive/ | `AtomWriter` 流式写 S 表达式，免中间字符串 |
| 006 | fix-atomwriter-implementations | ✅（plan-report 01） | archive/ | 7 类格式对齐手写期望；结构体构造器靠首字母大写启发式 |
| 011 | auto-atom-refactoring | ⏳ Planning | archive/ | auto-atom 生产化路线：AtomError/查询 API/JSON/schema |
| 012 | node-refactoring-indexmap | ✅ COMPLETED | archive/ | NodeBody/Obj 迁 IndexMap：O(1) 查找 + 插入序 |
| 013 | unify-args-and-props | ⏳ Planning | archive/ | 用 `num_args` 边界计数器把 args 并入 props IndexMap（重号见上） |
| 014 | unify-body-nodes-kids | ⏳ | archive/ | `body`/`body_ref`/`nodes` 三字段合一为 `kids: Kids` |
| 015 | atom-builder-api | ✅ 完成 | archive/ | 链式 `with_*` + Builder 两层构造 API（~735 行，77 测试） |
| 016 | atom-macro-dsl | ✅ 已完成 | archive/ | `value!/atom!/node!` proc-macro 复用 AutoLang parser，`#{var}` 插值 |

## 语法 / parser

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 026 | property-keywords | ✅ COMPLETE | archive/ | 属性关键字（`.view` 系复合 token）落地 |
| 056 | dot-expression-field-access | ✅ 已完成 | archive/ | `.field` 点表达式与字段访问解析 |
| 058 | type-alias-syntax | ✅（任务逐项 ✅） | archive/ | `type X = Y` 类型别名语法（`Stmt::TypeAlias`） |
| 060 | closure-syntax | ✅ Complete | archive/ | `x => expr` / `(a, b) => expr` 闭包（`Expr::Closure`） |
| 090 | remove-universe-from-parser | ✅ 完成 | archive/ | Parser 去 Universe：符号入 TypeStore，辅助入 parser_helpers.rs |
| 121 | task-msg-system | ✅ COMPLETED | archive/ | `task` 关键字解析为 `Stmt::TaskDef` |
| 156 | unified-enum-migration | ✅（Phase 1 起逐项 ✅） | archive/ | 统一 enum AST：`EnumKind` 区分标量/异构枚举 |
| 162 | method-keyword-to | 文首"待实现"，代码已实现 | archive/ | `.as(Type)`/`.to(Type)` → `Expr::Cast`/`Expr::To`（文档过时） |
| 228 | hetero-enum-tuple-syntax | ✅ 已完成 | archive/ | 异构 enum 多参数变体强制括号元组语法 |

## 模块 / use

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 078 | automan-integration | ✅ COMPLETE | archive/ | `ModuleResolver` trait 由来：解析策略可插拔（Stage 2） |
| 092 | rust-ffi-sandbox | ✅（Phase 1-6） | archive/ | `use.rust` 导入形式进 use_scanner |
| 106 | router-use-syntax | 未标注 | archive/ | router 场景的 use 语法改进 |
| 131 | module-path-syntax-design | ✅（2025-03-18 逐项 ✅） | archive/ | `super`/`pac` 前缀 + `ModulePath`/`PathPrefix` AST |
| 167 | module-system | ✅ 已完成 | archive/ | 模块系统完整实现，含 `pub use` |
| 184 | cross-module-function-calls | 未标注 | archive/ | 跨模块函数调用解析与派发 |
| 214 | python-ffi-use-py | ✅ COMPLETE | archive/ | `use.py` 导入形式进 use_scanner |

## 曾列为活跃的 plan（2026-09-07 复核：均已归档，状态以归档文件为准）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 325 | autovm-enum-method-and-cross-module-bugs | ✅（2026-08-01 复核缺陷 1/2/3 全部已修复） | archive/ | enum 方法调用与跨模块字符串缺陷，阻塞后端 Auto 代码 |
| 332 | derive-to-atom-proc-macro | ✅ 收官归档（2026-08-27，S1+S2 完成，serde 裁定取代自定义 trait 路线） | archive/ | `#[derive(ToAtom)]`/`FromAtom` 标注驱动 .at 序列化 |
| 367 | codegen-quality-improvements | ✅ COMPLETE（2026-07-30） | archive/ | 含 view fragment 语法（P2-3）落入 parser（`Stmt::ViewFragmentDecl`） |

## 2026-08 增补（Plan 471）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 425 | component-fn-sugar-retirement | ✅ | archive/ | component fn AST 级糖化（parse_component_fn_decl 产出等价 WidgetDecl） |
| 426 | setup-preamble-slot | ✅ | archive/ | parser 新增 AST SetupBlock（块级 refs 声明；await MVP 拒绝） |
| 435 | autoui-component-schema-unification | ✅ | archive/ | schema/aura.at 唯一契约源（aliases/tier/backends 字段扩展） |
| 448 | autoui-syntax-improvements | ✅ archived（2026-09-07 复核：验收全过、无阻断债） | archive/ | msg 去名+内联 lambda；铸名 mint_inline_event_handlers 进 parser |
| 470 | use-rs-alias | ✅ | archive/ | `use.rs` 现行拼写（与 use.py 对齐）+use.rust W0005 deprecation；双拼写同 UseKind::Rust，仓内 134 .at+文档全量迁移（外部仓批次三顺延 DEBT） |
| 550 | null-family-audit（frontend 侧） | ✅（reviewed→archived） | archive/550-null-family-audit.md | nil 拼写退役：literal/atom 双臂 W0005 DeprecatedFeature（语义不变，同落 PUSH_NIL）+CLI 直跑路径 parser 警告可见化（stderr 按名去重）；#[script] 文件级 pragma（script_pragma→CompileSession.script_marked）；生产者门控 lint：无 pragma 含 use.py/null/nil→迁移提示（只警告不拒绝，.as 指引归 W1） |
| 555 | script-mode-w1-dispatch-foundation（frontend 侧） | ✅（reviewed→archived） | archive/555-script-mode-w1-dispatch-foundation.md | ScriptMode 八格矩阵（.as≡隐式 #[script]，#[rust] 文件级 pragma 压回，优先序 rust>script>扩展名）+CompileSession.script_mode 回填（550 script_marked 派生兼容）+门控 lint 按 ScriptMode 统一判定（.as 自动豁免）；W1 passthrough，语义激活归 W2 |
| 567 | script-mode-w25-tail-w3-oracle（frontend 侧） | ✅（reviewed→archived） | archive/567-script-mode-w25-tail-w3-oracle.md | with-as 绑定语法（P560-D1 销号）：with_header pratt 截断+块形态降低（py_enter 绑定/try-catch-finally 出口保证/py_raise 479 再抛）+convert_last_block 纯 pair 收窄+单测 ×3；账本 P567-1..6 |
| 560 | script-mode-w2-sugar-batch（frontend 侧） | ✅（reviewed→archived） | archive/560-script-mode-w2-sugar-batch.md | with 关键字（无 as 形态直产 py_with·as 形态 Cast 歧义响亮拒绝）+Power(**) token+@/**/is 中缀糖解析层直产桥调用+#[with] 注解名撞位修复；债 P560-D1 |
