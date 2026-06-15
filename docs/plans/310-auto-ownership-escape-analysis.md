# Plan 310: Auto 所有权逃逸分析与智能指针回退

> **Status**: Draft
> **关系**: 本方案是 [`docs/design/04-memory-ownership.md`](../design/04-memory-ownership.md) "Planned" 区与 "Open Questions" 的落地设计。它**不修改** 04 文档的 view/mut/move trinity 与 Lifetime Levels(L0–L7)定义,而是回答"如何让用户**永远不需要写生命周期标注**,同时生成合法的 Rust 代码"。
> **范围**: 本文档定稿同步代码逃逸分析引擎(Phase 0–2)。async 捕获、Send 升级、自引用拒绝(Phase 3–4)仅在 §6 留接口,算法另行设计。

---

## §1 背景与动机

### 1.1 问题陈述

Rust 最劝退的特性是生命周期标注。Auto 的设计目标是:**用户从不写生命周期**,编译器通过分析自动决定:
- 能用借用(`&` / `&mut`)就用 —— 零成本;
- 分析无法判定时,**回退到智能指针**(`Rc<RefCell<T>>`),发 warning 透明告知用户;
- 实在不行(跨线程、自引用),由用户解决静态歧义。

经验估计:90%+ 的场景可自动解决,不需要标注。

### 1.2 与 04 设计文档的关系

[`04-memory-ownership.md`](../design/04-memory-ownership.md) 的 Open Questions 中,本方案直接解答:

| 04 的 Open Question | 本方案的回答 |
|---|---|
| "Whether lifetime annotations (`@Scope`, `@Task`) should be explicit syntax or purely inferred." | **纯推断,无显式语法。** 用户永远不写生命周期。 |
| "The exact rules for when the compiler can prove 'last use' and automatically insert a move." | **§4 逃逸分析条件集 + Tier 1 判定。** |
| "How `hold` path binding interacts with the enum pattern matching system." | Phase 2 之外,本文档不展开(留后续)。 |
| "Whether `shared` should be a keyword or a library type." | **都不是。** 见 §8 风险 4 —— 现有 `shared` 关键字语义冲突,本方案**不复用**它,Rc 回退由编译器隐式决定。 |

04 文档的 Lifetime Levels 表中,本方案聚焦 **L6(Scope,局部借用)与 L2(Auto/RC,引用计数)之间的自动降级**。

### 1.3 不可逾越的硬约束

**Auto 的逃逸分析是 `rustc` 借用检查器的"保守超集"。** Auto 生成的 Rust 代码仍要过 rustc,所以:

- **Auto 说"这是安全借用" → rustc 必须也认为安全**(否则是 bug,生成的代码编译失败)。
- **Auto 不确定 → 回退 `Rc<RefCell<T>>`**(rustc 一定接受)。
- **错误方向只能有一个**:False positive(本可借用却用了 Rc,损失性能)可以接受;False negative(生成了 `&` 但 rustc 拒绝)是必须消灭的 bug。

这条约束直接决定:**Auto 的逃逸分析宁可过度保守。** 它不需要、也不应该精确复刻 rustc 的 NLL / 两阶段借用 —— 那是无底洞。它只需要一个**足够保守的近似**,让"我说安全的就是安全的"。

---

## §2 设计目标与已确认决策

以下 7 项决策均经设计评审确认,作为后续实现的不可变基准。

### 决策 1 — Own-by-default(默认拥有)

struct 字段、返回值默认 owned(`String` 而非 `&str`);借用只是**函数体内的局部优化**。

**理由**:
1. **消灭结构体生命周期参数** —— Rust 生命周期痛点的最大来源。`struct Parser<'a> { src: &'a str }` 这类东西在 Auto 里直接不存在,字段就是 `String`。借用只在函数体内部作为局部临时量出现,作用域极短,逃逸分析极简单。
2. **可预测** —— 用户心智模型是"我拥有这个东西",而非"我借了这个东西,主人是谁"。
3. **借用成为优化而非约束** —— 分析失败只是多一次 clone 或一次 Rc,不会破坏程序结构。

**代价**:看似全是 owned 会"很多 clone",但 escape analysis 能救回绝大多数短生命周期的临时借用;真正的热点可手动 `view` 提示。

### 决策 2 — 默认 Rc,Send 边界升 Arc

单线程场景优先 `Rc<RefCell<T>>`(零锁开销);静态分析发现值进入 `tokio::spawn` / `.go` / channel 等 Send 边界时,自动升级为 `Arc<Mutex<T>>`。

### 决策 3 — view / mut / move 是提示(hint),可被覆盖

这三个关键字表达用户"希望借用"的**意图**,编译器尽量满足;分析判定不安全时自动升级为 own / Rc 并发 warning。**保持向后兼容,平滑过渡** —— 现有写 `view` 的代码不会破坏,只是语义从"强制借用"变为"建议借用"。

### 决策 4 — 每次回退都 warning(W0007)

每次从 `&` 降级到 Rc 都发 warning,带行号 + 原因(如 "value escapes closure at line N")+ 修复建议(如 "add explicit clone, or restructure")。透明可学习。

### 决策 5 — Copy / 小类型自动 clone

凡逃逸且 `T` 是 Copy 或体积 ≤ 阈值(默认 32 字节,可配),自动 clone 而非 Rc;其余才 Rc。最大化性能。

### 决策 6 — async 默认 move,共享才 Arc

async 块默认 `async move {}`(全部 owned 捕获);只有检测到"多个 async 块/task 共享同一变量"才升级为 Arc。规则简单,消灭约 80% 的 async 借用错误。

### 决策 7 — 先做同步逃逸分析

本设计文档定稿同步代码逃逸分析 + Rc 回退;async / 线程安全 / Arc 升级(决策 2、6 的运行时部分)与自引用拒绝留 Phase 3–4,另行设计。

---

## §3 五层 Tier 回退模型

把"一个值怎么持有"分成 5 个 tier,按编译器能证明到哪一层来选:

| Tier | 策略 | 运行时成本 | 触发条件 |
|---|---|---|---|
| 0 | `'static` / owned | 0 | 字面量、本就拥有的数据 |
| 1 | `&T` / `&mut T` 局部借用 | 0 | 逃逸分析**证明**不逃出有效作用域 |
| 2 | clone | 1 次 alloc | 逃逸 + `T` 是 Copy 或 ≤ 阈值字节 |
| 3 | `Rc<RefCell<T>>` | alloc + 运行时检查 | 单线程,分析无法判定 |
| 4 | `Arc<Mutex<T>>` | alloc + 锁 | 多线程(Phase 3) |

**只有 Tier 1 需要精确分析;Tier 2/3/4 都是"不确定时的安全网"。** 这把"难的部分"隔离在 Tier 1,其余都是确定性策略。

### 3.1 Tier 之间如何选择(决策流程)

```
对每个 binding,问:
  (1) 值本身是 'static / owned 吗? → Tier 0
  (2) 逃逸分析能证明它仅在当前函数体内、当前作用域内、不跨越 await、无 mut 冲突吗?
       YES → Tier 1(&T 或 &mut T)
       NO  → 继续
  (3) T 是 Copy 或 sizeof(T) ≤ CLONE_THRESHOLD 吗?
       YES → Tier 2(clone)
       NO  → 继续
  (4) 值可能跨线程吗?(检测 Send 边界,Phase 3)
       YES → Tier 4(Arc<Mutex>)
       NO  → Tier 3(Rc<RefCell>)
```

### 3.2 view/mut/move hint 如何影响 Tier

- 用户写 `view` → 期望 Tier 1;分析判定不行时降到 Tier 2/3,发 W0007。
- 用户写 `mut` → 期望 Tier 1 的 `&mut`;分析判定不行时降级,发 W0007。
- 用户写 `move` → 直接 Tier 0(无条件转移所有权,不分析)。
- 用户什么都不写 → 走默认决策流程(§3.1)。

---

## §4 逃逸分析算法(本方案核心)

### 4.1 可证明安全的条件集(Tier 1 触发)

一个 binding 可降级到 Tier 1,当且仅当**同时满足**:

1. **局部性**:值仅在定义函数体内使用 —— 不 `return`、不赋值给 struct 字段、不存入集合元素、不进入 closure 捕获列表。
2. **同步性**:借用不跨越 `await` 点(本设计范围;async 见 Phase 3)。
3. **无冲突**:无与 mutable borrow 冲突的同周期使用(同一作用域内 `&mut x` 与 `&x` 或另一次 `&mut x` 交叠)。
4. **无别名穿透**:值不被传入"签名不透明"的外部函数(`use.rust` 导入的 Rust API),除非该 API 的参数类型本身是 owned(可自动 deref)。

> **关键**:这是**充分条件集** —— 满足这些条件,Auto 生成的 `&` 必然被 rustc 接受。它远比 rustc 的实际规则保守,但**保证 no false negative**。

### 4.2 逃逸的判定(降级到 Tier 2/3)

出现以下任一情况,即判定为"逃逸",降级:

| 逃逸形式 | 示例 | 降级到 |
|---|---|---|
| 返回值 | `fn f() -> &T { return &x }` | Tier 0(强制 owned,生成 `fn f() -> T`) |
| 存入 struct 字段 | `let p = Point{x: &y}` | Tier 0(struct 字段永远 owned) |
| 存入集合 | `vec.push(&y)` | Tier 0(集合元素 owned) |
| closure 捕获 | `let c = () => print(x)` | Tier 2/3(按大小) |
| 传给外部 `use.rust` 函数 | `rust_fn(&y)` | Tier 0(按对方签名 deref) |
| 跨越 await | `let r = &y; async {... r.await}` | Tier 2/3(Phase 3 前,一律降级) |

### 4.3 算法形式

**作用域栈 + def-use 链**:

1. 遍历 AST,对每个 `Stmt::Store`(let/var 绑定)建立 `BindingId`。
2. 维护作用域栈(`Vec<Scope>`),进入 block / for / if 分支时 push,退出时 pop。
3. 对每个 binding,记录所有使用点(`Expr::Ref` / `Expr::Dot` / `Expr::Index` 等)及其"使用形状"(读 / 写 / 借出 / 移动 / 传入外部)。
4. 基于"使用形状"套用 §4.1 / §4.2 规则,给出 Tier 决策。

**声明:这是保守近似,不精确复刻 rustc NLL。** 只要满足"我说安全的就是安全的"即可。rustc 会做最终的精确判定 —— 如果 Auto 给了 Tier 1 而 rustc 拒绝,那是 Auto 的 bug,需收紧条件集(而非放宽)。

### 4.4 显式无法分析的场景(一律保守降级)

以下场景,分析器**一律降级**(Tier 2/3),不尝试精确分析:

- **跨函数别名**:值的引用通过函数返回传到调用者,再被别名。
- **动态分发**:通过 `spec` / `tag` 的方法体内部对 `&self` 的使用(方法实现不可见)。
- **递归数据结构环**:链表、树的双向引用。
- **`use.rust` API 内部行为**:外部 Rust 函数对传入引用的保存行为不可知。

这些场景的降级**不是缺陷**,而是保守策略的正确应用。用户若需优化,可手动重构或加 `view` 提示(分析器会尊重并尝试,失败仍发 warning)。

---

## §5 落地架构

基于代码探索,以下是精确的插入点与改动清单。

### 5.1 插入位置

逃逸分析作为 **a2r 转译器内部 pass**,插在 [`transpile_rust`](../../crates/auto-lang/src/trans/rust.rs) 的 CTEE transform 之后、`RustTrans::trans` 之前:

```
transpile_rust (rust.rs:10768)
  ├─ parser.parse()                    // :10773
  ├─ CTEE::transform(&mut ast)         // :10776-10777
  ├─ ★ EscapeAnalysis::analyze(&ast) ★ // 新增,10777 后、10780 前
  ├─ RustTrans::trans(ast, &mut out)   // :10780-10781
  └─ post_process(&mut out.body)       // :10784
```

**为什么是转译器内部 pass,而非独立编译阶段?** 因为 a2r 是独立链路([`rust.rs:10768`](../../crates/auto-lang/src/trans/rust.rs)),不经过 VM 链路 A 的 typeck/infer/ownership。逃逸分析只对 Rust 输出有意义,放转译器内最自然。

### 5.2 新模块:`crates/auto-lang/src/trans/escape/`

**独立于现有 `ownership/` 模块** —— 后者是占位框架(见 §5.3),不可直接用。新模块结构:

| 文件 | 职责 |
|---|---|
| `analyzer.rs` | 主分析器:作用域栈 + def-use 遍历,产 `EscapeMap` |
| `escape_map.rs` | `HashMap<BindingId, OwnershipTier>` 决策表 + 查询 API |
| `report.rs` | W0007 warning 生成,从 Tier 决策 + 失败原因构造 `Warning` |

**遍历骨架抄** [`ownership/cfa.rs:22-180`](../../crates/auto-lang/src/ownership/cfa.rs) 的 `LastUseAnalyzer`(`analyze_stmt` / `analyze_expr` + HashMap 状态),但要**补三件它没做的事**:
- Call 参数分析(cfa.rs:139 当前注释 "skip detailed arg analysis" —— 必须补,否则漏掉主要逃逸路径)。
- 作用域栈(cfa.rs 无,只有扁平的 `last_uses` map)。
- 控制流分支合并(if 多分支的 last-use 交集)。

### 5.3 为什么不复用现有 `ownership/` 模块

探索发现 [`ownership/`](../../crates/auto-lang/src/ownership/) 是**占位框架而非可用工具**:

- [`cfa.rs:122-123`](../../crates/auto-lang/src/ownership/cfa.rs) 注释明确:"conservatively mark every use as a potential last use" —— 即把**每次** `Expr::Ref` 都标为 last use,等于没分析。
- [`cfa.rs:139`](../../crates/auto-lang/src/ownership/cfa.rs) Call 不分析参数。
- 无作用域栈、无控制流合并、无跨函数追踪。
- **未被任何生产代码引用**:整个 `trans/` 目录搜索 `ownership` / `BorrowChecker` / `LastUseAnalyzer` 零命中,仅 [`tests/ownership_tests.rs:7`](../../crates/auto-lang/src/tests/ownership_tests.rs) 单元测试在用。

**结论**:可复用其**遍历骨架**和 **API 命名**,但分析逻辑必须从零写。新模块放在 `trans/escape/` 而非扩充 `ownership/`,避免与占位代码纠缠。

### 5.4 RustTrans 改动清单

| 改动点 | 位置 | 内容 |
|---|---|---|
| 新增字段 | [`rust.rs:64-198`](../../crates/auto-lang/src/trans/rust.rs) `RustTrans` 结构体 | 加 `escape_results: EscapeMap`、`warnings: Vec<Warning>` |
| `Expr::View` 决策 | [`rust.rs:1344-1348`](../../crates/auto-lang/src/trans/rust.rs) | 查 `escape_results`:Tier 1 → `&x`;否则 → `Rc::clone(&x)` + warning |
| `Expr::Mut` 决策 | [`rust.rs:1351-1356`](../../crates/auto-lang/src/trans/rust.rs) | Tier 1 → `&mut x`;否则 → 升级 own / Rc + warning |
| `Expr::Move` | [`rust.rs:1358-1362`](../../crates/auto-lang/src/trans/rust.rs) | 不变(无条件 move) |
| Dot 路径 `.view/.mut` 特判 1 | [`rust.rs:1082-1098`](../../crates/auto-lang/src/trans/rust.rs) | 同 View/Mut 决策 |
| Dot 路径 `.view/.mut` 特判 2 | [`rust.rs:2176-2193`](../../crates/auto-lang/src/trans/rust.rs) | 同上 |
| let 绑定类型推断 | [`rust.rs:6234-6249`](../../crates/auto-lang/src/trans/rust.rs) | Tier 1 → `&T`;降级 → `T` 或 `Rc<RefCell<T>>` |
| 函数参数类型 | [`rust.rs:6541`](../../crates/auto-lang/src/trans/rust.rs) | 参数层语义 view:heavy 类型 → `&T`;trivial → 值传递(已有,见下) |

**注意**:函数参数层已有部分 view 语义 —— [`rust.rs:858-863`](../../crates/auto-lang/src/trans/rust.rs) `rust_param_type_name` 对 str 类型生成 `&str`,调用站点 [`rust.rs:5543`](../../crates/auto-lang/src/trans/rust.rs) 自动加 `.as_str()`。本方案**扩展**这套机制到所有 heavy 类型,而非推倒重来。

### 5.5 Warning 通道(关键:不破坏测试)

**现状**:[`error.rs:1026`](../../crates/auto-lang/src/error.rs) 已有 `Warning` 枚举(W0001–W0006),但:
- `RustTrans` 结构体无 `warnings` 字段。
- `transpile_rust` 返回 `Sink`,无 warning 通道。
- [`lib.rs:1086`](../../crates/auto-lang/src/lib.rs) `run_a2r_file_test` 仅逐字节对比转译产物与 `.expected.rs`,不捕获 stdout/stderr。

**改动**:
1. [`error.rs:1026`](../../crates/auto-lang/src/error.rs) 新增 variant:
   ```rust
   #[error("ownership fallback to smart pointer")]
   #[diagnostic(code(auto_warning_W0007), severity(warning),
       help("..."))]
   EscapeFallback { name: String, reason: String, tier: u8, span: SourceSpan }
   ```
2. `RustTrans` 加 `warnings: Vec<Warning>` 字段。
3. `transpile_rust` 返回签名扩展(或在 `Sink` 加 `warnings` 字段),把 warnings 透传给调用方。
4. **铁律:warning 绝不 `write!` 进 `Sink`** —— 否则破坏 [`lib.rs:1086`](../../crates/auto-lang/src/lib.rs) 的字节对比。warning 走独立通道。

**测试影响**:`.expected.rs` 对比**完全不受影响**(warning 不入产物)。若需验证 warning 本身,另加 `.expected.warnings` 可选对比机制(Phase 2 引入)。

### 5.6 类型信息获取

逃逸分析需要类型信息(判断 Copy / 大小 / owned vs ref)。两条路:

| 方案 | 改动 | 优劣 |
|---|---|---|
| **A. 透传 `infer_ctx`(推荐)** | 改 `transpile_rust`,从 parser 取出 `infer_ctx`([`parser.rs:198`](../../crates/auto-lang/src/parser.rs))传给分析 pass | 改动小,复用 parser 已攒的类型映射 |
| B. pass 内重跑推断 | 分析 pass 自建 `InferenceContext`([`infer/context.rs:42`](../../crates/auto-lang/src/infer/context.rs)),对 AST 跑 `infer_expr` | 纯增量不动现有代码,但重复计算 |

**推荐 A**,但需注意 parser 的 `infer_ctx` 在 parse 完成后是否保留(探索显示 parser 边解析边推断,`infer_ctx` 在 parser 结构体内,parse 结束仍在)。

### 5.7 复用 `Target::from_expr`

[`ownership/borrow.rs:85`](../../crates/auto-lang/src/ownership/borrow.rs) 的 `Target::from_expr(&Expr) -> Target`(归一化为 `Variable` / `Path` / `Index` / `Unknown`)可直接复用 —— 它已处理 `View/Mut/Move/Take` 解包、`obj.field` 路径、`arr[index]`。逃逸分析追踪"值逃到哪"可用它做表达式规范化。

---

## §6 分阶段实施计划

### Phase 0 — 审计 + 地基(P0)

**目标**:确认现状 + 修低悬果 + 接通 warning 通道(不破坏测试)。

| 任务 | 位置 | 内容 |
|---|---|---|
| 0.1 确认 own-by-default | a2r 各 `borrow_*.expected.rs` | 验证 struct 字段已 owned;局部借用已是 `&` |
| 0.2 修 union 的 unsafe 包装 | [`rust.rs:8472`](../../crates/auto-lang/src/trans/rust.rs) `union_decl` | 生成 unsafe 访问方法,否则下游 rustc 报错 |
| 0.3 补 delegation TODO | [`rust.rs:7818`](../../crates/auto-lang/src/trans/rust.rs) | 委托方法体当前生成 `// TODO: Implement ...`,需补真实转发 |
| 0.4 接通 warning 通道 | [`error.rs:1026`](../../crates/auto-lang/src/error.rs) + `RustTrans` + `transpile_rust` | 加 `Warning::EscapeFallback` (W0007) + `RustTrans.warnings` 字段;**此阶段不实际发任何 warning**,只搭管道 |

**验收**:`cargo test -p auto-lang` 全绿;现有 `.expected.rs` 无任何变化。

### Phase 1 — 逃逸分析引擎(P0)

**目标**:实现分析器,对每个函数体跑分析,产 `EscapeMap`。**先只记录,不改变转译输出。**

| 任务 | 内容 |
|---|---|
| 1.1 `trans/escape/analyzer.rs` | 抄 `cfa.rs` 骨架,补 Call 参数分析 + 作用域栈 + 控制流合并 |
| 1.2 `trans/escape/escape_map.rs` | `HashMap<BindingId, OwnershipTier>` + 查询 API |
| 1.3 `trans/escape/report.rs` | 从 Tier 决策 + 失败原因构造 W0007(**此阶段 warning 只记录到 `RustTrans.warnings`,不改变输出**) |
| 1.4 插入 pass | `transpile_rust` 在 CTEE 后调 `EscapeAnalysis::analyze`,结果存 `RustTrans.escape_results` |
| 1.5 验证 | 加调试输出(可 feature flag),人工核对几个典型函数的 EscapeMap 是否正确 |

**验收**:分析跑通,EscapeMap 对已知用例(§7 测试组)给出预期 Tier;**转译产物字节不变**(证明未误改)。

### Phase 2 — Tier 1 + 2 接入(P0)

**目标**:查表生成 `&T` 或 clone;接通 W0007 warning;新增测试。

| 任务 | 内容 |
|---|---|
| 2.1 改 `Expr::View/Mut` + 两处 Dot 特判 | 查 `escape_results`:Tier 1 → `&x`/`&mut x`;Tier 2 → clone;Tier 3 → `Rc::clone` |
| 2.2 改 let 绑定类型推断 | 同步降级类型标注 |
| 2.3 发 W0007 | 每次降级发 warning,带行号 + 原因 + 建议 |
| 2.4 全量 cargo check 回归 | 把所有 a2r 测试产物喂 `cargo check`,验证 Tier 1 输出的 `&` 全部被 rustc 接受(**核心验收**) |
| 2.5 新增测试组 | `test/a2r/19_ownership/`,覆盖 §7 全部逃逸模式 |

**验收**:
- a2r 全部测试通过(`.expected.rs` 对比);
- 全部产物 `cargo check` 通过(无 false negative);
- W0007 warning 在预期位置出现。

### Phase 3 — async + Send(后续设计,本文档只留接口)

| 任务 | 接口预留 |
|---|---|
| 3.1 async 块强制 move 捕获 | 改 [`rust.rs:2365`](../../crates/auto-lang/src/trans/rust.rs) `Expr::AsyncBlock`,生成 `async move {}`;当前实现是"单语句白名单 + 静默跳过其余"([rust.rs:2390-2394](../../crates/auto-lang/src/trans/rust.rs)),需改成显式 move |
| 3.2 Send 边界检测 | 分析器新增"跨线程使用"标记:`tokio::spawn` / `.go`([rust.rs:2439](../../crates/auto-lang/src/trans/rust.rs)) / channel send 点;标记到的 binding 升 Tier 4 |
| 3.3 Tier 4 生成 | `Arc::new(Mutex::new(...))` + `Arc::clone` + `.lock().unwrap()` |

### Phase 4 — 硬场景(后续设计)

| 任务 | 接口预留 |
|---|---|
| 4.1 自引用拒绝 | 检测 struct 字段类型包含自身引用 → 编译错误 + 建议(拆分 / 用 Rc 打破环) |
| 4.2 Rc 环 Weak 提示 | 已知模式(双向链表、父→子→父)→ W0008 提示用 `Weak<T>` |

---

## §7 测试策略

### 7.1 新增测试组:`test/a2r/19_ownership/`

| 编号 | 场景 | 期望 Tier | 期望产物 |
|---|---|---|---|
| 001 | 局部短借用(不逃逸) | Tier 1 | `&x` |
| 002 | 返回借用 | Tier 0 | 强制 owned |
| 003 | 存入 struct 字段 | Tier 0 | owned 字段 |
| 004 | 存入 Vec | Tier 0 | owned 元素 |
| 005 | closure 捕获(小类型) | Tier 2 | clone |
| 006 | closure 捕获(大类型) | Tier 3 | `Rc::clone` |
| 007 | 传给 `use.rust` 函数 | Tier 0 | deref |
| 008 | view hint 但逃逸 | Tier 3 + W0007 | `Rc::clone` + warning |
| 009 | move hint | Tier 0 | 无条件 move |
| 010 | 多 mut 冲突 | Tier 3 + W0007 | 降级 + warning |

### 7.2 warning 对比机制

`.expected.rs` 字节对比**不受影响**(warning 不入 Sink)。为验证 warning 本身,Phase 2 引入可选的 `.expected.warnings` 文件:
- 存在则对比 `RustTrans.warnings` 序列化(JSON 或文本);
- 不存在则跳过(向后兼容现有测试)。

### 7.3 保守性验证(核心)

**Phase 2 末必做**:把 `test/a2r/` 与 `test/cookbook/` 全部产物的 `.expected.rs` 收集到一个临时 crate,跑 `cargo check`。**任何 Tier 1 输出的 `&` 被 rustc 拒绝,都是 Phase 1 条件集的 false negative,必须收紧条件集(而非放宽)**。

这是本方案"保守超集"约束的硬验收。

---

## §8 风险与缓解

| # | 风险 | 缓解 |
|---|---|---|
| 1 | **生成 `&` 被 rustc 拒绝(false negative)** | §4.1 保守条件集 + Phase 2 末全量 `cargo check` 回归(§7.3)。rustc 是最终裁判,Auto 只在能证明安全时才给 Tier 1。 |
| 2 | **Rc 回退过多导致性能退化** | 决策 4:每次回退发 W0007 透明可见,用户可针对性优化。长期可加 `#[borrow]` 强约束属性让用户"要求"借用(失败则编译错误)。 |
| 3 | **与后处理正则冲突** | [`rust.rs:9976`](../../crates/auto-lang/src/trans/rust.rs) `fix_borrowing_issues` 及 [`rust.rs:9332`](../../crates/auto-lang/src/trans/rust.rs) 起的一批 regex 后处理,可能误匹配新生成的 `Rc::clone` / `Rc::new`。Phase 2 验证新生成代码不被正则误改;必要时调整正则或把后处理逻辑前置到转译期。 |
| 4 | **`shared` 关键字语义冲突** | 现状 `shared` 是全局 `Lazy<Mutex<T>>`([`store.rs:10`](../../crates/auto-lang/src/ast/store.rs)、[`rust.rs:6187`](../../crates/auto-lang/src/trans/rust.rs)),与 Rc 回退是两码事。**本方案不复用 `shared` 名字**;Rc 回退由编译器隐式决定,用户无显式语法。若后续需要用户显式控制,另加 `#[shared]` 属性(不与关键字冲突)。 |
| 5 | **AST/Type 缺口** | 无 `RcExpr` / `RefCellExpr`,Type 无智能指针 variant([`types.rs:27-71`](../../crates/auto-lang/src/ast/types.rs))。本方案**不新增 AST/Type variant** —— 回退产物在转译期直接生成 `Rc::new(...)` / `Rc::clone(...)` 字符串,作为代码生成层的模板,不进 AST。这避免污染 AST 语义层。 |
| 6 | **`LastUseAnalyzer` 占位代码误导** | [`ownership/cfa.rs`](../../crates/auto-lang/src/ownership/cfa.rs) 的实现是"每次 use 都标 last use",可能被误以为可用。文档与代码注释明确标注其为占位,新实现独立于 `trans/escape/`。 |

---

## 附录 A:与现有代码的精确对应表

| 本方案概念 | 现有代码位置 | 关系 |
|---|---|---|
| Tier 1 生成 `&` | [`rust.rs:1344-1363`](../../crates/auto-lang/src/trans/rust.rs) `Expr::View/Mut` | 改为查表决策 |
| `.view/.mut` postfix | [`rust.rs:1082-1098`](../../crates/auto-lang/src/trans/rust.rs)、[`rust.rs:2176-2193`](../../crates/auto-lang/src/trans/rust.rs) | 同上(两处 Dot 特判) |
| let 绑定借类型 | [`rust.rs:6234-6249`](../../crates/auto-lang/src/trans/rust.rs) | 改为查表 |
| 函数参数 view 语义 | [`rust.rs:6541`](../../crates/auto-lang/src/trans/rust.rs)、[`rust.rs:858-863`](../../crates/auto-lang/src/trans/rust.rs) | 扩展现有机制 |
| 调用站点 auto-borrow/clone | [`rust.rs:5414-5555`](../../crates/auto-lang/src/trans/rust.rs) | 部分复用(类型驱动的现有逻辑) |
| `is_copy_type` | [`rust.rs:905`](../../crates/auto-lang/src/trans/rust.rs) | Tier 2 判定复用 |
| Warning 枚举 | [`error.rs:1026`](../../crates/auto-lang/src/error.rs) | 加 W0007 |
| 测试对比 | [`lib.rs:1086`](../../crates/auto-lang/src/lib.rs) | 不变(warning 不入 Sink) |
| a2r 入口 | [`rust.rs:10768`](../../crates/auto-lang/src/trans/rust.rs) `transpile_rust` | 插入分析 pass |
| 现有 view 测试 | `test/a2r/07_ownership/`(4 个) | Phase 2 期望输出可能变化(降级),需更新 `.expected.rs` |

---

## 附录 B:术语表

| 术语 | 含义 |
|---|---|
| **Tier** | 持有策略层级(Tier 0–4),见 §3 |
| **逃逸(Escape)** | 值的使用超出可证明安全的作用域,见 §4.2 |
| **保守超集** | Auto 的分析结论必须是 rustc 的超集 —— Auto 说安全,rustc 必同意;Auto 不说,rustc 可能更宽松(但 Auto 已降级到 Rc,无妨) |
| **False positive** | 本可借用却用了 Rc(损失性能,可接受) |
| **False negative** | 生成了 `&` 但 rustc 拒绝(必须消灭的 bug) |
| **BindingId** | 每个 `let`/`var` 绑定的唯一标识,逃逸分析的追踪单位 |
| **hint** | view/mut/move 关键字的新语义:用户意图,可被编译器覆盖 |

---

## 变更日志

| 日期 | 变更 |
|---|---|
| 2026-06-15 | 初稿(Plan 310 立项) |
