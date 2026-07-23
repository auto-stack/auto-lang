# 所有权与借用

## 范围

view/mut/move 三模式、`ParamMode`、borrow checker、生命周期、last-use 分析、
`hold` 路径绑定、逃逸分析回退（plan-310）。

## 原则

- 零开销安全：无 GC，内存错误编译期捕获；成本透明：语法暴露每个操作的运行时成本；
  安全处隐式：默认规则覆盖约 80% 所有权决策。
- 一切访问归约到三种 O(1) 模式（ADR-04）：`view`（只读借，默认）/ `mut`（可写借）/
  `move`（所有权转移）；`clone()` 显式、O(N)、带括号作视觉警告。
- 对称性：定义点与调用点默认都是 view；要更强能力两侧必须一致
  （定义 `mut`/`move` ↔ 调用 `.mut`/`.move`）。

## 细节

### ParamMode（ast/fun.rs:180）

`View`（默认）/ `Mut` / `Move`；`Copy` 已删除语义、`Take` 为 `Move` 的 deprecated 别名
（Display 输出均为 `move`）。move 参数在函数体内隐式可变。
资源类型（list、文件、`!T`）禁止 `=` 赋值转移，必须显式 `.move`。

### ownership/ 模块三件套

| 文件 | 机制 |
|---|---|
| `ownership/borrow.rs:BorrowChecker` | Rust 式借用规则：多 view 共存、mut 独占、mut 与 view 互斥、借用不得悬垂；`Target` 归一化（`x`/`view x`/`obj.field` 归并到同一基目标）做冲突检测 |
| `ownership/lifetime.rs:Lifetime(u32)` | 生命周期为编号区域，`STATIC=0`；`outlives` 规则：ID 小者命长（`a.0 <= b.0`），交集取短命者 |
| `ownership/cfa.rs:LastUseAnalyzer` | 控制流分析检测变量最后使用点，供自动清理/自动 move 插入 |

不变量（borrow.rs 头注释）：mut 借用唯一；借用不得超过数据本身寿命。
已知漂移：本模块术语仍为 view/mut/**take**（`BorrowKind::Take`），与 `ParamMode::Move` 同义。
线性类型基础（`Linear`/`MoveState`/`MoveTracker`）来自 auto-val crate，经 ownership/mod.rs 重导出。

### 参数传递实现（ABO-01，plan-088）

语义一律 view；转译器对平凡类型（int/float/bool/char/byte）生成寄存器值传递，
重类型生成引用传递。前端类型检查无论底层怎么传都按不可变引用执法（ADR-05）。

### hold 路径绑定（ast/hold.rs）

`hold x.y.z as value { ... }`：记录访问路径 → 块入口锁定中间结构 → 物化为可变引用 →
块出口释放。编译期算偏移，等价于受限可变借用，零额外开销。
现状：parser 已支持，与 borrow checker 的深度集成未完成（design/04 Partial）。

### 逃逸分析回退（plan-310，trans/escape/）

编译期做同步逃逸分析：证明安全 → 借用（Tier 1，零成本）；证明不了 →
`Rc<RefCell<T>>`（Send 边界升 `Arc`）并发 W0007 warning；view/mut/move 仅作 hint 可被覆盖；
Copy/小类型自动 clone；async 默认 move。显式不复用 ownership/（§5.3：一个管执法、一个管代码生成决策）。

### 生命周期分级（设计层）

L0 Immortal（NVM）→ L1 Process（全局）→ L2 Auto（RC 堆对象）→ L3 Task →
L4 Start/Stop → L5 Period（帧）→ L6 Scope（局部默认）→ L7 Instant（临时）。
是否引入显式 `@Scope` 标注仍是 Open Question；当前实现只有编号区域（lifetime.rs）。

## 显式非目标

- 不做 GC；跨任务共享走 `shared`（ARC）而非全局堆。
- 不要求用户写生命周期标注（plan-310 硬约束）。
- NLL、作用域退出自动清理、ParamChecker 接入管线为已知遗留（plan-indices/04）。
- 嵌入式任务的静态栈分析属并发模块，不在本模块执法。

> 来源: docs/design/04-memory-ownership.md、docs/plans/archive/310-auto-ownership-escape-analysis.md、crates/auto-lang/src/ownership/、ast/fun.rs、ast/hold.rs
