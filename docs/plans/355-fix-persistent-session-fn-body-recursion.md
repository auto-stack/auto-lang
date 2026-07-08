# Plan 355: 修复持久 Session 解析"fn 内含复合语句"的无限递归栈溢出

> **状态**：待实施
> **优先级**：高
> **影响**：ash（auto-shell）的 MS3 脚本能力严重受限——脚本里无法定义任何带控制流的函数
> **报告来源**：auto-shell v0.5（Plan 010/011 实施期间发现），已在 `auto-shell/plans/010-ms3a-trycatch-while.md` §8 记录

## 一句话描述

在 `AutovmReplSession::run`（ash 使用的持久 AutoLang session，增量编译）里，**只要 `fn` 的函数体内包含任何带 `{ }` body 的语句**（`if`/`else`/`for`/`while`/`loop`/嵌套 `fn`/`try`），调用 `session.run()` 就会**无限递归导致 OS 栈溢出**，进程崩溃。函数体内**只有**表达式/赋值（无 `{ }`）时正常。

## 1. 复现（100% 可复现）

环境：auto-lang master（commit `3e8694f0` 及更早都复现；**非** Plan 010/011 引入，在 Plan 010 之前的 commit `f82251c8` 同样复现）。通过 ash 的 persistent session 执行（ash 内部调 `AutovmReplSession::run` 增量编译）。

### 最小复现 1：fn 内一个 if

```auto
fn f() {
    if true {
        print("a")
    }
}
f()
print("done")
```

**结果**：`thread 'main' has overflowed its stack`，无任何输出，进程崩溃。

### 最小复现 2：fn 内一个 for-range

```auto
fn loopfn() {
    for i in 0..3 {
        print(i)
    }
}
loopfn()
print("done")
```

**结果**：同样栈溢出。

### 最小复现 3：甚至不调用，只定义就炸

```auto
fn loopfn() {
    for i in 0..3 {
        print(i)
    }
}
print("defined, not called")
```

**结果**：栈溢出。**注意：函数从未被调用，溢出发生在编译/解析阶段，不是执行阶段。**

### 对照组（全部正常）

| 代码 | 结果 |
|------|------|
| `fn f() { var x = 1+2; print(x) }`（fn 内只有表达式，无 `{ }`）| ✅ 正常，打印 3 |
| 顶层 `if true { print("a") }`（if 在顶层，无 fn 包裹）| ✅ 正常，打印 a |
| 顶层 `for i in 0..3 { print(i) }`（for 在顶层）| ✅ 正常，打印 0/1/2 |
| 顶层 `var i=0; while (i<3) { print(i); i=i+1 }`（while 在顶层）| ✅ 正常 |

### 关键对照：`run_file`（整文件编译）完全正常

同样的代码用 `auto <file>.at` 直接 runner（`run_file`，把整个文件作为一个编译单元）运行，**完全正常**：

```auto
// repro.at —— 用 `auto repro.at` 运行，正常
fn loopfn() {
    for i in 0..3 {
        print(i)
    }
}
fn main() {
    loopfn()
    print("done")
}
```

**结果**：✅ 打印 0/1/2/done。

→ **bug 是 `AutovmReplSession`（持久 session 增量编译）特有的**，不在 `run_file`（整文件编译）路径上。这把根因锁定在 persistent session 的某处状态/增量逻辑。

## 2. 已定位的故障阶段

在 auto-lang 源码插桩追踪后确认：

**溢出发生在 `AutovmReplSession::run` 里的 `parser.parse()` 阶段**（解析 AST 时），**不是 codegen（编译为字节码）也不是 VM 执行阶段**。

### 追踪证据

在 `session.run` 各阶段插桩 `eprintln!`：
- `[DEBUG] start` ✅ 打印
- `[DEBUG] after resolve_use` ✅ 打印
- `[DEBUG] after resolve_py, before parse` ✅ 打印
- 之后就栈溢出（`parser.parse()` 内部）

并且给以下方法加了递归深度计数器，**均未触发**（说明无限递归不经过它们）：
- `codegen::compile_stmt` —— 不触发（证明不是 codegen 阶段，也佐证是 parse 阶段）
- `parser::parse_stmt_inner` —— 不触发
- `parser::parse_body` —— 不触发
- `parser::next()`（token 消耗）—— 不触发（<5000 次，说明不是词法层面的死循环，而是**真正的深递归**，即某个解析方法 A→B→A 互相调用）

用 `RUST_MIN_STACK=134217728`（128MB）放大主线程栈**仍然溢出**，进一步证明是**无限递归**，不是有限深递归。

## 3. 可疑根因（强烈建议从这里查）

### 线索 A：codegen 跨调用累积（最初怀疑，但因阶段在 parse 而存疑）

`AutovmReplSession::run`（`crates/auto-lang/src/autovm_persistent.rs:339`）每次调用时做了 `codegen.code.clear()` 和 `codegen.relocs.clear()`（第 383-386 行），但**没有清** `codegen.jump_placeholders` 和 `codegen.jump_targets`。这两个 Vec 会跨多次 `session.run` 调用**累积**。

**但注意**：追踪显示溢出在 `parse()` 阶段而非 `compile`。所以这条线索可能不是直接根因，但 `jump_placeholders`/`jump_targets` 的累积**仍是一个独立的潜在 bug**，修复时应一并清掉。

### 线索 B：parse 阶段的状态污染（当前重点怀疑）

既然溢出在 `parse()`，且不经过 `parse_stmt_inner`/`parse_body`/`next()`，重点查 `session.run` 在 parse 之前对 parser 的注入：

```rust
// autovm_persistent.rs:347-356
let mut parser = Parser::from(code);
parser.set_type_registry(self.type_registry.clone());  // ← 注入了跨调用累积的 type_registry
parser = parser.skip_check();
let ast = parser.parse()?;  // ← 这里溢出
```

**重点检查**：
1. `self.type_registry` 是否跨 `session.run` 调用累积了某种数据（如递归的类型定义、自引用的泛型实例），导致 parser 在解析 `fn` 体内的复合语句时陷入无限递归。
2. `Parser::from(code)` 是否真的干净构造，还是有共享的内部缓存（如 `Rc<RefCell<...>>`）把上一次的状态带进来。
3. 对比 `run_file`（正常）和 `session.run`（崩溃）在解析同一个 `fn` 时，parser 内部状态（尤其是 `scope` / `type_registry` / 正在解析的 fn 栈）的差异。

## 4. 最小化定位建议（给修复者）

1. **加单元测试**：`AutovmReplSession::new()` 后，调一次 `run("fn f() { if true { print(\"a\") } }")`，断言不 panic。这个测试当前会栈溢出。

2. **隔离 parse**：把 `parse()` 单独抽出来，对一个 `fn f() { if true {} }` 字符串用**全新的** `Parser::from(code)` 解析（不注入 session 的 type_registry），看是否正常。
   - 若正常 → 是 session 注入的状态污染 parser（走线索 B）。
   - 若仍溢出 → 是 parser 本身在解析"fn+复合语句"时有递归 bug（独立于 session）。

3. **找递归方法对**：在 `parser.rs` 的解析方法群里，找一个会"互相递归"的对。最可能在 `fn_decl_stmt` / `parse_fn` / 函数体解析时，又调用了能回到自身的语句解析路径。给那个方法加深度计数 + kill 开关（参考：`parse_stmt_inner`/`parse_body`/`next()` 已确认**不**在递归路径上，所以要找的是绕过它们的方法）。

4. **清理累积状态**：无论根因如何，`session.run` 开头应同时 clear `jump_placeholders`、`jump_targets`、以及任何其他跨调用累积的 codegen/parser 状态（防止后续相关 bug）。

## 5. 验收标准

- [ ] `session.run("fn f() { if true { print(\"a\") } }")` 不再栈溢出
- [ ] `fn` 内的 `if`/`for`/`while`/`loop`/嵌套 `fn`/`try` 全部能正常定义和调用
- [ ] auto-lang 全量测试通过（含 conformance，无回归）
- [ ] ash 的 `examples/deploy.ash` 能把循环/try 移回 `fn` 内（目前被迫放顶层规避）
- [ ] `jump_placeholders`/`jump_targets` 跨 `session.run` 不再累积（顺手清理）

## 6. 环境

- auto-lang 仓库 master（commit `3e8694f0` 及更早都复现）
- auto-shell 仓库 v0.5（通过 ash 暴露此 bug；ash 只是调用方，bug 在 auto-lang）
- Windows 10，但 bug 与平台无关（栈溢出是逻辑问题）

## 7. 关联

- auto-shell 端记录：`auto-shell/plans/010-ms3a-trycatch-while.md` §8
- auto-shell 规避示例：`auto-shell/examples/deploy.ash`（循环/try 放顶层）
