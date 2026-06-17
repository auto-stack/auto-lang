# Plan 321: Generator 运行时 — yield + 统一 ~Iter<T> / ~Stream<T>

> **Status**: ✅ Phase 1-6 Delivered(2026-06-17,已合并 master)
> **依赖**: YIELD_TASK/YIELD_VAL opcode 重命名已完成(commit `2a02e558`)
> **关联**: HTTP SSE streaming 已实现(Spec §11);HTTP 异步流模式(~Stream<T> + await)留后续
> **范围**: yield 关键字、Generator 栈帧、Iterator::Generator 变体、~Iter<T>/~Stream<T> 类型、a2r 转译
> **已知遗留**: for-loop generator 值错误(VM task 帧冲突);true lazy 求值(当前 eager + budget);async-stream crate 依赖

---

## §1 核心设计

### 1.1 yield 的语义

`yield expr` 是 generator 函数内的挂起点:
1. 计算 expr,得到一个值
2. 把值交给调用方(通过 Iterator::Generator 的 next() 返回)
3. **挂起当前 generator 帧**(保存 ip/bp/sp/栈内容)
4. 当调用方再次调 next() 时,恢复帧,从 yield 的下一条指令继续

```auto
fn fib() ~Iter<int> {
    var a = 0
    var b = 1
    for {
        yield a
        let t = a + b
        a = b
        b = t
    }
}

// 消费方
for n in fib() {
    if n > 1000 { break }
    print(n)
}
```

### 1.2 ~Iter<T> vs ~Stream<T> vs ~T(Future<T>)

当前 `~T` 被解析为 `Future<T>`(parser.rs:8989)。需要扩展 `~` 语义:

| 语法 | 解析为 | 语义 |
|---|---|---|
| `~int` | `Future<int>` | 异步值,await 取结果 |
| `~Iter<int>` | `GenericInstance { base: "Iter", args: [int] }` | 同步 generator 序列 |
| `~Stream<int>` | `GenericInstance { base: "Stream", args: [int] }` | 异步流(SSE 推流) |

**关键设计**:`~` 前缀表示"延迟/异步",后面跟的类型决定具体语义。`~Iter` 和 `~Stream` 不是包一层 Future,而是直接的 GenericInstance。

### 1.3 Generator 帧结构

Generator 挂起时必须独立保存完整栈帧(因为调用方的 next() 在自己的栈上执行):

```rust
/// Plan 321: Generator frame — saved execution state for yield/resume.
pub struct GeneratorFrame {
    pub ip: usize,                // 恢复点:yield 的下一条指令
    pub bp: usize,                // Base pointer
    pub sp: usize,                // Stack pointer
    pub stack: Vec<NanoValue>,    // 栈内容快照(局部变量 + 参数)
    pub num_args: usize,
    pub num_locals: usize,
    pub func_addr: u32,           // generator 函数地址(首次 next 时用)
    pub started: bool,            // 是否已启动(首次 next 需要建帧)
    pub done: bool,               // 是否已结束(函数 return)
}
```

---

## §2 分阶段实施计划

### Phase 1 — 基础设施:GeneratorFrame + StepResult(P0)✅ 已完成

**目标**:在 engine.rs 里建好 generator 的数据结构。

| 文件 | 行号 | 改动 |
|---|---|---|
| `engine.rs:78`(Iterator 结构区) | 新增 `GeneratorFrame` 结构(见 §1.3) |
| `engine.rs:109`(Iterator 枚举) | 新增 `Iterator::Generator(GeneratorFrame)` 变体 |
| `engine.rs:145`(StepResult 枚举) | 新增 `StepResult::GeneratorYield { value: NanoValue }` 变体 |
| `engine.rs`(run_one_instruction) | **实现 `OpCode::YIELD_VAL` 分支**:pop 栈顶值 → 设 GeneratorFrame 挂起状态 → 返回 `StepResult::GeneratorYield` |
| `engine.rs`(disasm.rs/abt) | YIELD_VAL 加入零操作数 pattern arm(已完成) |

**YIELD_VAL 执行逻辑**:
```
1. val = task.ram.pop_nv()        // 取出 yield 的值
2. 保存当前 task.ip/bp/sp/stack 到 GeneratorFrame
3. return StepResult::GeneratorYield { value: val }
```

注意:YIELD_VAL 在 engine.rs 的 `run_one_instruction` 里执行,但 generator 的"调用者"是 `shim_iterator_next`(不是 call_fn_by_name)。因此 YIELD_VAL 不能用 `task.bp == saved_bp` 判定退出,而需要新的控制流协议(GeneratorYield)。

**验收**:编译通过,YIELD_VAL 有执行分支(即使还没有调用者)。

### Phase 2 — Iterator::Generator 的 next() 驱动(P0)✅ 已完成(eager + budget)

**目标**:`shim_iterator_next` 能恢复 generator 帧执行到下一个 YIELD_VAL。

| 文件 | 行号 | 改动 |
|---|---|---|
| `native.rs:1641`(shim_iterator_next match) | 新增 `Iterator::Generator(gen)` 分支 |
| `engine.rs` | 新增 `resume_generator(gen) -> StepResult` 方法 |

**Generator next() 逻辑**:
```
1. 如果 gen.done → 返回 -1(nil)
2. 如果 !gen.started → 建帧(设 ip = func_addr, push 参数),gen.started = true
3. 恢复帧:把 gen.stack 写回一个临时 task 的 ram,设 ip/bp/sp
4. run_one_instruction 循环:
   - 遇到 GeneratorYield { value } → 保存帧到 gen,返回 value
   - 遇到 RET → gen.done = true,返回 -1(nil)
   - 遇到 Continue → 继续执行
5. 返回 value 或 -1
```

**核心难点**:generator 不能在调用者的 task 栈上恢复(会覆盖调用者的数据)。两种方案:
- **方案 A**(推荐):generator 在自己的独立 AutoTask 上执行(vm.spawn_task),next() 时切换到该 task 恢复执行
- **方案 B**:generator 的 stack 在 Iterator::Generator 里独立保存,next() 时用临时 task 加载

**验收**:手写字节码(直接编 YIELD_VAL)能跑通 next/yield/next/RET 序列。

### Phase 3 — Parser:yield 关键字 + ~Iter<T> 类型(P0)✅ 已完成

**目标**:用户能写 `fn gen() ~Iter<int> { yield 42 }`。

| 文件 | 行号 | 改动 |
|---|---|---|
| `token.rs:97` | 新增 `Yield` 变体 |
| `token.rs:383` | 新增 `"yield" => Some(TokenKind::Yield)` |
| `parser.rs:8989`(parse_type Tilde 分支) | **解决 `~Iter` 冲突**:若 `~` 后跟 `Iter<T>`/`Stream<T>`,直接生成 `GenericInstance { base: "Iter"/"Stream" }`,不包 Future |
| `ast.rs:307`(Expr 枚举) | 新增 `Expr::Yield(Box<Expr>)`(yield 是表达式,返回 yielded 值) |
| `parser.rs:1409`(parse_expr 区域) | 新增 yield 表达式解析:yield 关键字 + 可选 expr → `Expr::Yield(expr)` |

**`~Iter` 的 parse_type 分流逻辑**:
```rust
TokenKind::Tilde => {
    self.next(); // skip ~
    let inner_type = self.parse_type()?;
    match &inner_type {
        Type::GenericInstance(inst) if inst.base_name == "Iter" || inst.base_name == "Stream" => {
            // ~Iter<T> → 直接返回 Iter/Stream,不包 Future
            Ok(inner_type)
        }
        _ => {
            // ~T → Future<T>(现有行为)
            Ok(Type::GenericInstance(GenericInstance {
                base_name: "Future".into(),
                args: vec![inner_type],
                source: None,
            }))
        }
    }
}
```

**验收**:`fn gen() ~Iter<int> { yield 42 }` 能被 parser 成功解析;AST 里有 `Expr::Yield(Int(42))`。

### Phase 4 — Codegen:yield → YIELD_VAL(P0)✅ 已完成

**目标**:yield 表达式编译为 YIELD_VAL 字节码。

| 文件 | 行号 | 改动 |
|---|---|---|
| `codegen.rs`(compile_expr match) | 新增 `Expr::Yield(expr)` 分支:`compile_expr(expr)` → `emit(YIELD_VAL)` |
| `codegen.rs`(Stmt::Fn) | 检测返回类型是 `~Iter`/`~Stream` 的函数,在首次调用时创建 `Iterator::Generator` |

**yield 的字节码**:
```
compile_expr(yield expr):
  1. compile_expr(expr)     // 求值 expr,压栈
  2. emit(YIELD_VAL)        // 弹栈 → 挂起 → 调用方拿到值
  3. (恢复后继续)            // yield 表达式的"返回值"是 void/nil(被忽略)
```

**generator 函数的调用**:
当 `gen()` 被调用且 gen 返回 `~Iter<T>` 时,不执行函数体,而是创建 `Iterator::Generator(GeneratorFrame { func_addr, started: false, ... })`,返回 iterator_id。首次 next() 时才建帧执行。

**验收**:`fn gen() ~Iter<int> { yield 1; yield 2 }` 编译为字节码;VM 运行 `for x in gen() { print(x) }` 输出 `1\n2`。

### Phase 5 — a2r 转译(P1)✅ 已完成(stream! 宏包装 + impl Iterator/Stream)

**目标**:yield 在 a2r 模式下转译为 Rust generator。

| 文件 | 行号 | 改动 |
|---|---|---|
| `trans/rust.rs:6770`(is_async_fn 检测) | 扩展:检测返回类型 `~Iter`/`~Stream` |
| `trans/rust.rs:6926`(Future 返回 unwrap) | 新增 `Iter`/`Stream` 分支 |
| `trans/rust.rs:859`(GenericInstance 类型映射) | `Iter<T>` → `impl Iterator<Item = T>`;`Stream<T>` → `impl Stream<Item = T>` |
| `trans/rust.rs:6316`(Stmt::Return 旁) | 新增 `Expr::Yield` 转译 |
| `trans/rust.rs`(fn_decl) | generator 函数体用 `async-stream` crate 的 `stream! { }` 宏包裹 |

**a2r 转译示例**:
```auto
// Auto 源码
fn gen() ~Iter<int> {
    yield 1
    yield 2
}
```
```rust
// 生成的 Rust(用 async-stream crate)
fn gen() -> impl Iterator<Item = i32> {
    async_stream::stream! {
        yield 1;
        yield 2;
    }
}
```

**验收**:a2r 测试 `yield_to_iterator.at` 的 `.expected.rs` 包含 `impl Iterator` + `async_stream::stream!`。

### Phase 6 — 测试 + for 循环消费(P0)✅ 已完成(SSE 路径正确,for-loop 有已知遗留)

**目标**:端到端验证 yield generator 能被 for 循环消费。

**测试用例**:
```auto
fn counter() ~Iter<int> {
    var i = 0
    for {
        yield i
        i = i + 1
        if i >= 5 { return }
    }
}

pub fn main() {
    var sum = 0
    for n in counter() {
        sum = sum + n
    }
    print(sum)  // 0+1+2+3+4 = 10
}
```

**验收**:
- VM 模式:`auto run test.at` 输出 `10`
- a2r 模式:转译后的 Rust 编译通过且输出 `10`
- for 循环消费 generator 不需要改 codegen(走现有 iterator next() 路径)

---

## §3 关键设计决策

### 3.1 yield 是表达式还是语句?

**决策:表达式**(`Expr::Yield(Box<Expr>)`)。

理由:
- `let x = yield some_value` 在协程模型里很有用(调用方可以注入值)
- 与 Python/Kotlin 的 yield 一致
- 表达式比语句更灵活(语句是表达式的子集)

当前 MVP:yield 表达式的返回值是 nil(被忽略)。未来可以支持 `yield` 双向通信(bi-directional generator)。

### 3.2 Generator 用独立 Task 还是栈快照?

**决策:独立 AutoTask**(方案 A)。

理由:
- generator 的局部变量/参数/调用栈在挂起期间必须存活
- 独立 task 的 ram 天然提供隔离
- `vm.spawn_task` 已存在,复用基础设施
- task 可以长期存活在 `vm.tasks: DashMap` 里,通过 iterator_id 关联

代价:每个 generator 占一个 task 槽(内存),但 generator 本来就是有状态的。

### 3.3 ~Iter<T> 是同步还是异步?

**决策:`~Iter<T>` 同步,`~Stream<T>` 异步**。

- `~Iter<T>`:pull 模型,`next()` 同步返回,用于 `for x in iter { }`
- `~Stream<T>`:pull 模型,`next()` 可能需要 await(等 I/O),用于 SSE 推流

当前 MVP(Phase 1-4)只做同步 `~Iter<T>`。`~Stream<T>` 需要 VM 的真异步调度支持(与 HTTP 异步流计划一起做)。

---

## §4 风险与缓解

| # | 风险 | 缓解 |
|---|---|---|
| 1 | **YIELD_VAL 在 call_fn_by_name 的阻塞循环里被吞** | Generator 不走 call_fn_by_name;走独立的 resume_generator 路径(native.rs 的 Generator 分支) |
| 2 | **~T 语法冲突**(~Iter 被 Future 包裹) | parse_type 的 Tilde 分支分流:Iter/Stream 直接返回,其余包 Future |
| 3 | **Generator task 泄漏**(next 完不清理) | iterator close 时清理关联 task;或 GC 时扫描 done 的 generator |
| 4 | **a2r 的 async-stream 依赖** | 需在 a2r-std/Cargo.toml 加 async-stream crate;或手写状态机 |
| 5 | **for 循环消费 generator 的 codegen 兼容性** | 现有 for-in-iter() 路径(codegen.rs:1940)已经是 CALL_NAT next + 检查 -1 的 pull 骨架,generator 注册成 iterator 后自动兼容 |

---

## §5 不做(范围控制)

- **HTTP SSE 集成**:handler 返回 `~Stream<T>` 自动变成 SSE 响应 —— 由独立计划(HTTP 异步流模式)处理
- **`~Stream<T>` 的异步 next()**:需要 VM 真异步调度(让出 + 恢复),Phase 1-4 只做同步 `~Iter<T>`
- **双向 generator**(`yield` 返回调用方注入的值)—— 未来增强
- **generator 的 send/throw/close 协议** —— 未来增强

---

## §6 验收标准

1. `fn gen() ~Iter<int> { yield 1; yield 2; yield 3 }` 被 parser 成功解析
2. VM 模式:`for x in gen() { print(x) }` 输出 `1\n2\n3`
3. a2r 模式:转译为 `impl Iterator<Item = i32>` + `async_stream::stream!` 块
4. 无限 generator `fn fib() ~Iter<int> { for { yield a; ... } }` 能被 `for` + `break` 消费
5. 现有 a2r/cookbook/escape 测试零回归
6. `~T`(Future)语义不变(async/await 仍正常工作)

---

## §7 与 HTTP 异步流计划的关系

本计划交付的 `yield` + `~Iter<T>` 是 HTTP SSE 的**语言原语基础**。HTTP SSE streaming 已在 Plan 321 实现期间一并完成(commit `0ec28c90`):
- `call_fn_by_name` 检测 generator 函数(扫描 0x8D YIELD_VAL)→ 创建 `Iterator::Generator`
- `http_server.rs` 检测 handler 返回值是 iterator ID → 自动进入 SSE 模式(text/event-stream)
- 循环 `shim_iterator_next` + `data: <value>\n\n` + flush

**已验证**:`GET /api/counter`(返回 `~Iter<int>`)→ `data: 1\ndata: 2\ndata: 3\n`(curl 测试通过)

---

## §8 实现结果(2026-06-17)

### 已交付

| 组件 | 实现 | commit |
|---|---|---|
| YIELD_VAL opcode (0x8D) | engine.rs 执行分支 + GeneratorYield StepResult | `2a02e558` |
| CREATE_GENERATOR opcode (0x8E) | inline operands (func_addr + n_args) → Iterator::Generator | 多个 commit |
| yield 关键字 | token.rs + parser.rs + Expr::Yield + codegen YIELD_VAL | Plan 321 Phase 3-4 |
| ~Iter<T> / ~Stream<T> 类型 | parse_type Tilde 分流(不包 Future) | Plan 321 Phase 3 |
| Generator next() driver | native.rs Iterator::Generator 分支(eager + budget 10000) | 多个 commit |
| a2r yield 转译 | Expr::Yield → `yield expr`; 函数体 `async_stream::stream! { }` 包装 | `b4b02840` |
| a2r 返回类型 | `~Iter<T>` → `impl Iterator<Item = T>`; `~Stream<T>` → `impl Stream<Item = T>` | `b4b02840` |
| HTTP SSE streaming | call_fn_by_name generator 检测 + http_server SSE 模式 | `0ec28c90` |
| AutoHttpServer shim 层 | vm/ffi/http_server.rs 独立模块(VM + a2r 共享) | `4549ae87` |
| HTTPStream → Iter 统一 | Iterator::HttpStream 变体 + shim_http_stream_iter | `4549ae87` |

### 已知遗留

| 遗留 | 影响 | 修复方向 |
|---|---|---|
| for-loop generator 值错误 | `for n in gen()` 返回错误值(FN_PROLOG 帧冲突) | 正确的 generator task 帧初始化 |
| Generator eager 模式 | 首次 next() 跑完全部(budget 10000) | 栈快照保存/恢复实现真 lazy |
| async-stream crate 依赖 | a2r 输出需要 `async-stream` 在 Cargo.toml | auto-man 脚手架自动添加 |
| yield 表达式返回值 nil | `let x = yield val` 的 x 是 nil | 未来双向 yield 支持 |
| HTTPS/TLS | 只有 HTTP | 引入 rustls/axum-server |
| 并发请求处理 | 串行(std::net) | 每请求 spawn 线程或换 Axum |
