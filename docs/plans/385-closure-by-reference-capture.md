# Plan 385：闭包 by-reference 捕获 — 让闭包能修改外部变量

> **状态**: 设计待实施
> **来源**: Plan 340 实施时发现 —— List HOF 的 forEach 闭包内修改外部变量不生效
> **影响仓库**: `auto-lang`（`crates/auto-lang/src/vm/`）
> **风险**: 中高 — 触动闭包运行时核心（CLOSURE opcode + 捕获变量存储）
> **前置**: 无（独立）

---

## 1. 问题

### 1.1 症状

Auto 的闭包（lambda）目前是 **by-value 捕获**：闭包获得外部变量的副本，闭包内修改不影响原始变量。

```auto
fn check() {
    var x int = 10
    var f = () => {
        x = x + 5    // 修改的是 x 的副本
    }
    f()
    print(x)         // 输出 10（期望 15）
}
```

实测确认（file-based 测试 `99_misc/001_closure_probe`）：
- 闭包体**确实执行了**（`print("inside")` 有输出）
- 但外部变量修改**不保留**（`x` 仍为 10）

### 1.2 影响场景

1. **List.forEach 的副作用模式**：`notes.forEach((n) => { total = total + n.amount })` 不工作（total 不累加）
2. **闭包内状态累积**：任何用闭包修改外部计数器/累加器/标志位的模式
3. **回调模式**：事件处理器修改组件状态

### 1.3 当前 workaround

用 `reduce` 替代（纯函数式，通过返回值通信）：
```auto
// forEach 不工作 → 用 reduce
var sum = notes.reduce(0, (acc int, n Note) => acc + n.id)
```

### 1.4 与其他语言的对比

| 语言 | 闭包捕获 | Auto 当前 |
|------|---------|----------|
| JavaScript | by-reference（闭包共享变量） | ❌ |
| Python | by-reference（需 `nonlocal` 声明赋值） | ❌ |
| Rust | 显式（`move` = by-value，borrow = by-ref） | ≈ 默认 move |

Auto 的设计目标是对标 JavaScript/Python 的易用性，by-reference 捕获是更符合直觉的默认行为。

---

## 2. 根因分析

### 2.1 闭包运行时机制

当前 CLOSURE opcode 的捕获机制（`crates/auto-lang/src/vm/engine.rs`）：

1. codegen 在创建闭包时，把捕获的外部变量的**当前值**复制到 `captures: HashMap` 里
2. CALL_CLOSURE 执行时，捕获的值被 push 到栈帧上作为闭包的局部变量
3. 闭包内修改这些"局部变量"只是修改了栈上的副本
4. 闭包返回后，修改丢失（不会写回原始变量位置）

### 2.2 核心问题

捕获存储的是**值**（`NanoValue`），不是**引用**（指向原始 slot 的指针/索引）。要支持 by-reference，需要：

- 捕获存储**原始变量的栈槽地址**（`bp + offset`）或**全局变量名**
- 闭包内读写捕获变量时，通过地址间接访问原始位置

---

## 3. 方案

### 方案 A：捕获改为 slot 引用（推荐）

**核心思路**：CLOSURE 的 `captures` 从 `HashMap<String, NanoValue>`（值副本）改为 `HashMap<String, CaptureRef>`，其中 `CaptureRef` 是对原始变量的间接引用。

```rust
enum CaptureRef {
    /// 局部变量：指向栈帧的相对地址（在创建闭包时记录 bp + offset）
    Local { frame_bp: usize, slot_offset: usize },
    /// 全局变量：按名字查找 vm.globals
    Global { name: String },
}
```

**CALL_CLOSURE 执行时的改动**：
1. 闭包的局部变量空间里，捕获的变量不存值副本，而是存一个**间接标记**（或通过特殊 GET_CAPTURED / STORE_CAPTURED 读写）
2. 读写捕获变量时，通过 CaptureRef 定位到原始位置

**复杂度**：中高。需改 CLOSURE opcode 的捕获编码 + CALL_CLOSURE 的栈帧设置 + GET_CAPTURED/STORE_CAPTURED 的执行体。

### 方案 B：闭包共享栈帧（简化版）

**核心思路**：闭包不创建独立栈帧，而是在**调用者的栈帧**上执行（共享局部变量）。

- 简单实现：CALL_CLOSURE 不 push 新 bp，而是沿用调用者的 bp
- 捕获变量直接是调用者的局部变量，无需间接

**优点**：改动小（CALL_CLOSURE 不切 bp）
**缺点**：闭包的参数与调用者的局部变量可能冲突（命名碰撞）；递归闭包会破坏调用者栈帧

### 方案 C：Box/Cell 模式（最小改动）

**核心思路**：不改闭包机制，而是让捕获的变量自动装箱（heap-allocated Cell）。

- codegen 检测到变量被闭包捕获时，把它从栈变量改为 heap 变量（类似 JS 的 let in closure）
- 闭包捕获 heap id 而非值
- 读写走 heap 间接

**优点**：不改 CLOSURE/CALL_CLOSURE 的核心逻辑
**缺点**：codegen 需提前分析哪些变量被捕获（escape analysis），改动分散

### 推荐：方案 A

方案 A 最干净（运行时层面改一处），虽然改动量中等，但语义清晰、不引入命名碰撞风险。

---

## 4. 实施步骤

### Phase 1：调研 + 最小复现（先行）

1. 读 CLOSURE opcode 的 codegen（`codegen.rs` 的闭包创建路径）
2. 读 CALL_CLOSURE 的执行体（`engine.rs`）
3. 读 GET_CAPTURED / STORE_CAPTURED 的当前实现
4. 确认捕获变量的存储格式和读写路径

### Phase 2：CaptureRef 类型 + CLOSURE 改造

1. 定义 `CaptureRef` 枚举
2. CLOSURE 的捕获编码从值改为引用
3. codegen 在创建闭包时记录原始变量的 slot offset（而非当前值）

### Phase 3：CALL_CLOSURE + GET/STORE_CAPTURED 改造

1. CALL_CLOSURE 设置闭包栈帧时，捕获变量存 CaptureRef 而非值
2. GET_CAPTURED：通过 CaptureRef 读原始位置
3. STORE_CAPTURED：通过 CaptureRef 写原始位置

### Phase 4：测试 + 回归

1. file-based 测试：闭包修改外部变量、forEach 副作用、嵌套闭包
2. 回归：全部既有测试零新增失败

---

## 5. 风险

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 栈帧生命周期：闭包在创建者返回后才调用（CaptureRef 指向已释放栈帧） | 高 | 高 | 捕获时检测：若闭包可能逃逸（存入变量/作为返回值），自动升级为 heap 装箱 |
| CALL_CLOSURE 栈帧布局变化破坏既有闭包 | 中 | 高 | 保留 i32 值捕获作为 fast path（不可变捕获仍 by-value） |
| codegen 需提前知道变量是否被捕获修改 | 中 | 中 | 保守策略：所有被闭包引用的外部变量都按 by-reference 处理 |

---

## 6. 非目标

- ❌ 不做 escape analysis（保守地全按 by-reference）
- ❌ 不做 Rust 式显式 `move`/`borrow` 标注
- ❌ 不改闭包的参数传递机制（仅改捕获变量）

---

## 7. 关联

- **Plan 340**（List HOF 方法）：forEach 的闭包副作用不生效，根因即本计划所述
- **Plan 060**（closure/lambda）：VM 函数值运行时机制的基础
