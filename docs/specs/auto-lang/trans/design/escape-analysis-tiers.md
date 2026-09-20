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
  - **PLAN-667 (F-03)**：闭包/lambda 捕获真实下降（源序遮蔽 + 嵌套域恢复的
    自由变量收集器）——旧实现的捕获检测为空集（gather_var_refs 不下降闭包）。
  - **PLAN-667 (F-04)**：未覆盖语句/表达式形式 **fail-closed**（全可见绑定升级）；
    Try/Reply 覆盖。
- `escape/escape_map.rs`：`OwnershipTier`（`is_borrow()` / `is_smart_pointer()` 分层判断）、
  `BindingId`（名字 + 词法作用域深度）、`EscapeMap::lookup(scope_depth, name)`。
  - **PLAN-667 (F-04) 身份契约**：`analyze_fn/analyze_body` 完成时
    `fold_to_root()`——同名绑定按 max-ordinal（逃逸获胜）合并到 depth 0 根条目。
    原因：生成侧 `current_scope_depth` 恒为 0（从不递增），按深度身份记录的
    嵌套/兄弟作用域决策在生成侧不可见（漏报逃逸 = 不健全 `&`）。合并是保守
    方向（同名兄弟损失借用机会，false positive 可接受）。
- `escape/report.rs:build_warning`：回退时构造 W0007 warning（行号 + 原因 + 修复建议）。

接线：`transpile_rust` 在 CTEE 之后、`trans` 之前跑分析，结果存入
`RustTrans.escape_results: HashMap<AutoStr, EscapeMap>`；代码生成期用
`current_fn_name` + `current_scope_depth` 查询（plan-310 Phase 2 起接入 `Expr::View/Mut` 生成点）。

## 已确认决策（plan-310 §2，七条；PLAN-667 修订以 ➤ 标注）

1. **Own-by-default**：字段/返回值默认 owned（`String`），借用只是函数体内局部优化。
2. **默认 Rc，Send 边界升 Arc**：进 `tokio::spawn`/`.go`/channel 自动升级 `Arc<Mutex<T>>`。
   ➤ **PLAN-667 (F-05) 修订**：Arc 声明改写未实现——Send 边界捕获在 `.view/.mut`
   使用点为**明确 Auto 侧诊断拒绝**（错误消息指名 Send/Arc 边界），不再以
   `.clone()` fallback 伪装支持。
3. **view/mut/move 是提示（hint）**：表达意图，分析判不安全时自动升级并发 warning，向后兼容。
   ➤ **PLAN-667 (F-05) 修订**：`.mut` 于逃逸（Clone/RcRefCell 层）绑定是**明确
   拒绝**（mutate 一个 clone/Rc 副本会静默偏离 VM 语义——VM 对共享 `.mut` 以
   `auto.rc.assert_unique` 运行期拒绝）；诊断给出重构建议。`.view` 于逃逸绑定
   仍走 clone/Rc 降级 + W0007。
4. **每次回退都 warning（W0007）**：透明可学习；warning 绝不写入输出字节（保护 .expected.rs 逐字节比对）。
5. **Copy/小类型自动 clone**：逃逸且 ≤ 阈值（默认 32 字节）时 clone 而非 Rc。
6. **async 默认 move**：共享同一变量才升 Arc。
7. **先做同步逃逸分析**，async 场景后续迭代。

## rustc 边界（PLAN-667 声明）

safe Rust 生成以**实际 rustc 编译**为门禁（非文本比对）。闭包**读**捕获逃逸出
函数（返回/存储闭包）依赖 rustc 借用检查拒绝（E0373 族）——这是声明的边界，
非有害 fallback；写捕获有 `Rc<RefCell>` 真实通路（`Rc::clone` + `move` 前缀）。

## 显式非目标

- 不精确复刻 rustc NLL / 两阶段借用（无底洞，只做足够保守的近似）。
- 不在结构体上引入生命周期参数（决策 1 的直接推论）。

> 来源: docs/plans/archive/310-auto-ownership-escape-analysis.md；crates/auto-lang/src/trans/escape/（analyzer.rs、escape_map.rs、report.rs）；crates/auto-lang/src/trans/rust.rs（escape_results 接线）
