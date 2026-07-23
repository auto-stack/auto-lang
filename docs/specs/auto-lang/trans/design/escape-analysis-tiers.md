# 逃逸分析与所有权分层（a2r）

> 范围：`trans/escape/` 模块如何为 a2r 的 `view`/`mut`/move 生成决策提供依据。

## 问题

Auto 的 `.view` / `.mut` / `.take` 要映射成 Rust 的 `&` / `&mut` / move，但生成的代码
必须过 rustc 借用检查。plan-310 定下的硬约束：**Auto 的逃逸分析是 rustc 借用检查器的保守超集**——

- Auto 说"安全借用" ⇒ rustc 必须也认为安全（否则是 bug）；
- 不确定 ⇒ 回退到 `Rc<RefCell<T>>`（rustc 一定接受）；
- 错误方向只能是 false positive（本可借用却回退，损性能），false negative 是必须消灭的 bug。

## 机制

- `escape/analyzer.rs:EscapeAnalyzer`：`analyze_fn(&Fn) -> EscapeMap`，静态分析函数体内每个绑定的逃逸情况。
- `escape/escape_map.rs`：`OwnershipTier`（`is_borrow()` / `is_smart_pointer()` 分层判断）、
  `BindingId`（名字 + 词法作用域深度）、`EscapeMap::lookup(scope_depth, name)`。
- `escape/report.rs:build_warning`：回退时构造 W0007 warning（行号 + 原因 + 修复建议）。

接线：`transpile_rust` 在 CTEE 之后、`trans` 之前跑分析，结果存入
`RustTrans.escape_results: HashMap<AutoStr, EscapeMap>`；代码生成期用
`current_fn_name` + `current_scope_depth` 查询（plan-310 Phase 2 起接入 `Expr::View/Mut` 生成点）。

## 已确认决策（plan-310 §2，七条）

1. **Own-by-default**：字段/返回值默认 owned（`String`），借用只是函数体内局部优化。
2. **默认 Rc，Send 边界升 Arc**：进 `tokio::spawn`/`.go`/channel 自动升级 `Arc<Mutex<T>>`。
3. **view/mut/move 是提示（hint）**：表达意图，分析判不安全时自动升级并发 warning，向后兼容。
4. **每次回退都 warning（W0007）**：透明可学习；warning 绝不写入输出字节（保护 .expected.rs 逐字节比对）。
5. **Copy/小类型自动 clone**：逃逸且 ≤ 阈值（默认 32 字节）时 clone 而非 Rc。
6. **async 默认 move**：共享同一变量才升 Arc。
7. **先做同步逃逸分析**，async 场景后续迭代。

## 显式非目标

- 不精确复刻 rustc NLL / 两阶段借用（无底洞，只做足够保守的近似）。
- 不在结构体上引入生命周期参数（决策 1 的直接推论）。

> 来源: docs/plans/archive/310-auto-ownership-escape-analysis.md；crates/auto-lang/src/trans/escape/（analyzer.rs、escape_map.rs、report.rs）；crates/auto-lang/src/trans/rust.rs（escape_results 接线）
