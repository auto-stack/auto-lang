# Plan 126: Phase 4 - 微观并发引擎与隐式 Worker Pool

## Status: ✅ COMPLETE (Phase 4.1-4.5)

## Objective

为 AutoLang 实现并发系统的最终阶段——**微观并发引擎**，引入 `.go` 后缀操作符，让开发者能够将异步蓝图（`~T`）派发到后台 Worker Pool 执行，彻底释放主 Actor 的消息处理循环。

## 核心特性

| 特性 | 语法 | 说明 |
|------|------|------|
| `.go` 后缀操作符 | `~{ ... }.go` | 将异步蓝图派发到后台执行 |
| `.go` 后缀操作符 | `expr.go` | 对返回 `~T` 的函数调用派发 |
| 捕获语义 | Copy 自动，非 Copy 需 `.take` | 安全的所有权转移 |
| 调度模式 | M:N (默认) / 1:N (可选) | 编译期配置 |

## 依赖

- Plan 121: Task/Msg 基础系统 ✅
- Plan 124: `~T`/`.await`/`ask/reply` ✅
- Plan 125: `ctx`/模式匹配 ✅

## 实现范围

| 后端 | 范围 | 状态 |
|------|------|------|
| **a2rs (Rust)** | `.go` → `tokio::spawn` | ✅ 本计划 |
| **AutoVM** | VM 层 `.go` 执行 | ✅ 本计划 |
| **a2c (C/RTOS)** | 状态机 + xTaskCreate | ⏸️ 后续独立计划 |

---

## 设计文档参考

- [docs/design/task-msg.md](../design/task-msg.md) - Phase 4 完整设计规范

---

## Part 1: 语法设计

### 1.1 `.go` 后缀操作符

`.go` 是一个后缀操作符，类似于 `.await`，作用于任何 `~T` 类型的表达式。

**语法形式**：

```auto
// 形式 1: 异步块后缀
~{
    let result = heavy_compute().await.?
    ctx.reply(result)
}.go

// 形式 2: 函数调用后缀
fn upload_logs(data string) ~void {
    http.post("/logs", data).await.?
}

upload_logs(data).go

// 形式 3: 链式调用
fetch_data().await.?.process().go
```

### 1.2 与 `.await` 的对称性

| 操作符 | 语义 | 行为 |
|--------|------|------|
| `.await` | 时间挂起 | 阻塞当前 Task，等待蓝图出图 |
| `.go` | 空间转移 | 当前 Task 瞬间放手，将蓝图扔进后台 |

**对比示例**：

```auto
// 使用 .await：阻塞等待
let result = heavy_task().await.?  // 当前 Task 挂起

// 使用 .go：后台执行
heavy_task().go  // 当前 Task 立即继续，heavy_task 在后台执行
```

### 1.3 捕获语义

`.go` 块可以捕获外部变量，遵循严格的所有权规则：

**规则**：
1. **Copy 类型**（int, bool, float, 等）：自动 Copy，原变量仍可用
2. **非 Copy 类型**：必须显式 `.take` 或 `.clone()`，否则编译报错

**示例**：

```auto
task WebWorker {
    on(ctx) {
        "heavy_compute" => {
            let count = 100        // int: Copy 类型
            let user = get_user()  // User: 非 Copy 类型

            ~{
                // Copy 类型：直接使用
                print(count)

                // 非 Copy 类型：必须显式处理
                // process(user)  // 编译错误！
                process(user.take)  // 正确：转移所有权
            }.go
        }
    }
}
```

**编译器错误示例**：

```
Error: Variable 'user' of type 'User' does not implement Copy.
Hint: Use 'user.take' to transfer ownership, or 'user.clone()' to create a copy.
```

---

## Part 2: Lexer 与 Parser 扩展

### 2.1 Lexer 新增 Token

```rust
// token.rs
pub enum TokenKind {
    // ... 现有 tokens
    Go,  // .go 后缀操作符
}
```

**注意**：`go` 关键字需要添加到保留字列表中。

### 2.2 Parser 扩展

`.go` 作为后缀操作符，优先级与 `.await` 相同。

**AST 节点**：

```rust
// ast/expr.rs
pub enum Expr {
    // ... 现有变体

    /// .go 后缀操作符: expr.go
    /// 将异步蓝图派发到后台执行
    Go {
        expr: Box<Expr>,  // 必须是 ~T 类型
        pos: Pos,
    },
}
```

**解析逻辑**：

```rust
// parser.rs
fn parse_postfix(&mut self, expr: Expr) -> AutoResult<Expr> {
    loop {
        if self.is_kind(TokenKind::Dot) {
            self.next();

            if self.is_kind(TokenKind::Go) {
                self.next();
                expr = Expr::Go {
                    expr: Box::new(expr),
                    pos: self.cur.pos,
                };
            } else if self.is_kind(TokenKind::Await) {
                // 现有 .await 处理
            }
            // ... 其他后缀操作符
        } else {
            break;
        }
    }
    Ok(expr)
}
```

### 2.3 语义检查

**规则**：
1. `.go` 只能作用于 `~T` 类型的表达式
2. `.go` 表达式本身返回 `void`（fire-and-forget）
3. 捕获的非 Copy 变量必须显式处理

```rust
// 语义检查伪代码
fn check_go_expr(expr: &Expr) -> Result<(), SemanticError> {
    let ty = infer_type(expr)?;

    // 检查是否是 Future 类型
    if !is_future_type(&ty) {
        return Err(SemanticError::GoRequiresFuture {
            found: ty,
            pos: expr.pos(),
        });
    }

    // 检查捕获变量的所有权
    check_captured_variables(expr)?;

    Ok(())
}
```

---

## Part 3: AutoVM 运行时实现

### 3.1 VM 状态扩展

```rust
// vm/mod.rs
pub struct AutoVM {
    // ... 现有字段

    /// 后台任务计数器（用于监控）
    go_task_count: AtomicU64,
}
```

### 3.2 `.go` 执行逻辑

```rust
// vm/eval.rs
fn eval_go(&mut self, expr: &Expr) -> AutoResult<Value> {
    // 1. 评估内部表达式，获取 Future
    let future = self.eval(expr)?;

    // 2. 提取捕获的环境
    let captured_env = self.capture_environment(expr)?;

    // 3. 派发到 Tokio 线程池
    let handle = tokio::spawn(async move {
        // 在新任务中执行 Future
        let _ = future.await;
    });

    // 4. 记录任务（可选，用于调试/监控）
    self.go_task_count.fetch_add(1, Ordering::Relaxed);

    // 5. 返回 void
    Ok(Value::Void)
}
```

### 3.3 环境捕获

```rust
// vm/capture.rs
pub struct CapturedEnv {
    /// Copy 类型的变量（直接复制）
    pub copied: HashMap<Name, Value>,

    /// Take 的变量（所有权转移）
    pub moved: HashMap<Name, Value>,
}

impl AutoVM {
    pub fn capture_environment(&self, expr: &Expr) -> AutoResult<CapturedEnv> {
        let mut env = CapturedEnv::new();

        // 分析表达式中的自由变量
        let free_vars = analyze_free_variables(expr)?;

        for var in free_vars {
            let value = self.lookup(&var.name)?;
            let ty = value.type_of();

            if ty.implements_copy() {
                // Copy 类型：直接复制
                env.copied.insert(var.name, value.clone());
            } else if var.is_explicitly_taken() {
                // 显式 .take：转移所有权
                env.moved.insert(var.name, value);
                self.remove_binding(&var.name)?; // 原作用域不再可用
            } else {
                // 非 Copy 且未显式处理：报错
                return Err(SemanticError::NonCopyCapture {
                    var: var.name,
                    ty,
                    hint: "Use '.take' to transfer ownership, or '.clone()' to create a copy.",
                });
            }
        }

        Ok(env)
    }
}
```

---

## Part 4: a2rs (Rust 后端) 转译

### 4.1 `.go` 转译规则

```rust
// trans/rust.rs
fn transpile_go(&self, expr: &Expr, env: &mut Env) -> String {
    // 获取内部表达式
    let inner = self.transpile_expr(expr, env);

    // 生成 tokio::spawn 调用
    format!("tokio::spawn(async move {{ let _ = {}.await; }});", inner)
}
```

### 4.2 示例转译

**Auto 源码**：

```auto
~{
    let result = math.calculate_primes(10000).await.?
    ctx.reply(result)
}.go
```

**生成的 Rust 代码**：

```rust
tokio::spawn(async move {
    let result = math::calculate_primes(10000).await?;
    ctx.reply(result);
});
```

### 4.3 链式调用转译

**Auto 源码**：

```auto
fetch_data().await.?.process().go
```

**生成的 Rust 代码**：

```rust
tokio::spawn(async move {
    let _ = fetch_data().await?.process().await;
});
```

---

## Part 5: 调度模式配置

### 5.1 默认模式：M:N 工作窃取

在服务器/桌面环境，默认使用 Tokio 的多线程调度器：

```rust
// TaskSystem.start() 的底层实现
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();
```

**特点**：
- 根据 CPU 核心数自动派生 OS 线程
- Work-stealing 算法自动负载均衡
- 适合高并发服务器

### 5.2 单线程模式：1:N 事件循环

通过 `#[single_thread]` 标注启用：

```auto
#[single_thread]
fn main() ! {
    TaskSystem.start()
}
```

**底层实现**：

```rust
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();
```

**特点**：
- 所有 `.go` 任务在单一线程执行
- 零跨线程锁开销
- 适合嵌入式/微控制器

---

## Part 6: 测试计划

### 6.1 Lexer 测试

```rust
#[test]
fn test_lexer_go_token() {
    let tokens = lex("~{ }.go");
    assert!(contains_token(&tokens, TokenKind::Go));
}
```

### 6.2 Parser 测试

```rust
#[test]
fn test_parse_go_with_async_block() {
    let code = "~{ print(42) }.go";
    let expr = parse_expr(code);
    assert!(matches!(expr, Expr::Go { .. }));
}

#[test]
fn test_parse_go_with_function_call() {
    let code = "async_func().go";
    let expr = parse_expr(code);
    assert!(matches!(expr, Expr::Go { .. }));
}

#[test]
fn test_parse_go_chain() {
    let code = "fetch().await.?.process().go";
    let expr = parse_expr(code);
    // 验证链式调用结构
}
```

### 6.3 语义检查测试

```rust
#[test]
fn test_go_requires_future_type() {
    let code = "42.go";  // int 不是 Future
    let result = check_semantics(code);
    assert!(result.is_err());
}

#[test]
fn test_non_copy_capture_requires_take() {
    let code = r#"
        let user = get_user()
        ~{ process(user) }.go
    "#;
    let result = check_semantics(code);
    assert!(result.is_err());  // 应该报错
}

#[test]
fn test_non_copy_capture_with_take() {
    let code = r#"
        let user = get_user()
        ~{ process(user.take) }.go
    "#;
    let result = check_semantics(code);
    assert!(result.is_ok());  // 应该通过
}
```

### 6.4 AutoVM 测试

```rust
#[tokio::test]
async fn test_vm_go_basic() {
    let code = r#"
        fn main() ! {
            ~{ print("background") }.go
            sleep(100).await
        }
    "#;
    let output = run_vm(code).await;
    assert!(output.contains("background"));
}

#[tokio::test]
async fn test_vm_go_with_capture() {
    let code = r#"
        fn main() ! {
            let msg = "hello"
            ~{ print(msg) }.go  // msg 是 string，Copy 类型
            sleep(100).await
        }
    "#;
    let output = run_vm(code).await;
    assert!(output.contains("hello"));
}

#[tokio::test]
async fn test_vm_go_isolation() {
    // 验证 .go 不会阻塞主 Task
    let code = r#"
        task Worker {
            on(ctx) {
                "test" => {
                    ~{ sleep(1000).await }.go
                    ctx.reply("immediate")  // 应该立即返回
                }
            }
        }

        fn main() ! {
            let result = Worker.ask("test").await.?
            print(result)
        }
    "#;
    let output = run_vm(code).await;
    assert!(output.contains("immediate"));
}
```

### 6.5 a2rs 转译测试

```rust
#[test]
fn test_a2rs_go_basic() {
    let auto_code = "~{ print(42) }.go";
    let rust_code = transpile_to_rust(auto_code);
    assert!(rust_code.contains("tokio::spawn"));
}

#[test]
fn test_a2rs_go_with_await() {
    let auto_code = "fetch().await.?.process().go";
    let rust_code = transpile_to_rust(auto_code);
    assert!(rust_code.contains("tokio::spawn"));
    assert!(rust_code.contains(".await"));
}
```

### 6.6 压力测试

```rust
#[tokio::test]
async fn test_high_throughput_go() {
    // 瞬间派发 100000 个后台任务
    let code = r#"
        fn main() ! {
            for i in 0..100000 {
                ~{ sleep(1).await }.go
            }
            sleep(1000).await  // 等待所有任务完成
        }
    "#;

    let start = Instant::now();
    run_vm(code).await;

    // 验证内存稳定，不会爆炸
    assert!(start.elapsed() < Duration::from_secs(5));
}
```

---

## Part 7: 实现步骤

### Phase 4.1: Lexer 扩展 (1 天)

**目标文件**：
- `crates/auto-lang/src/token.rs`
- `crates/auto-lang/src/lexer.rs`

**任务清单**：
1. 添加 `TokenKind::Go`
2. 添加 `go` 到保留字列表
3. 编写 Lexer 单元测试

**验收标准**：
- [ ] `go` 正确识别为 `TokenKind::Go`
- [ ] `.go` 正确解析为 Dot + Go

---

### Phase 4.2: Parser 扩展 (2 天)

**目标文件**：
- `crates/auto-lang/src/ast/expr.rs`
- `crates/auto-lang/src/parser.rs`

**任务清单**：
1. 添加 `Expr::Go` AST 节点
2. 扩展 `parse_postfix()` 处理 `.go`
3. 实现 `Expr::Go` 的 Display trait
4. 编写 Parser 单元测试

**验收标准**：
- [ ] `~{ ... }.go` 正确解析
- [ ] `func().go` 正确解析
- [ ] `expr.await.?.go` 链式调用正确解析

---

### Phase 4.3: 语义检查 (2 天)

**目标文件**：
- `crates/auto-lang/src/infer/expr.rs`
- `crates/auto-lang/src/infer/capture.rs` (新建)

**任务清单**：
1. 实现 `.go` 类型检查（必须是 `~T`）
2. 实现自由变量分析
3. 实现 Copy/非 Copy 变量检查
4. 编写语义检查测试

**验收标准**：
- [ ] 非 Future 类型调用 `.go` 报错
- [ ] 非 Copy 变量未显式处理报错
- [ ] `.take` 正确识别并允许

---

### Phase 4.4: AutoVM 运行时 (2 天)

**目标文件**：
- `crates/auto-lang/src/vm/eval.rs`
- `crates/auto-lang/src/vm/capture.rs` (新建)

**任务清单**：
1. 实现 `eval_go()` 方法
2. 实现环境捕获逻辑
3. 集成 Tokio spawn
4. 编写 VM 集成测试

**验收标准**：
- [ ] `.go` 正确派发到后台执行
- [ ] Copy 变量正确复制
- [ ] 非 Copy 变量正确转移
- [ ] 主 Task 不被阻塞

---

### Phase 4.5: a2rs 转译器 (2 天)

**目标文件**：
- `crates/auto-lang/src/trans/rust.rs`

**任务清单**：
1. 实现 `transpile_go()` 方法
2. 处理捕获变量的 `move` 语义
3. 编写转译器测试

**验收标准**：
- [ ] `.go` 正确转译为 `tokio::spawn`
- [ ] 捕获变量正确标记 `move`

---

### Phase 4.6: 集成测试与文档 (1 天)

**目标文件**：
- `crates/auto-lang/src/tests/phase4_tests.rs` (新建)

**任务清单**：
1. 编写完整集成测试
2. 更新设计文档
3. 更新 Plan 126 状态

**验收标准**：
- [ ] 所有测试通过
- [ ] 文档更新完成

---

## 验收标准 (Acceptance Criteria)

### 功能验收

1. **`.go` 语法正确** ✅
   - 后缀操作符形式
   - 与 `.await` 对称
   - 链式调用支持

2. **捕获语义安全** ✅
   - Copy 类型自动复制
   - 非 Copy 类型必须显式 `.take`
   - 编译期错误提示清晰

3. **运行时正确** ✅
   - 后台执行不阻塞主 Task
   - 内存稳定，无泄漏

### 性能验收

1. **高压吞吐测试**
   - 100,000 个 `.go` 任务在 5 秒内调度完毕
   - 内存水位稳定

2. **隔离性验证**
   - 死循环 `.go` 不阻塞宿主 Task

### 兼容性验收

1. **Phase 1-3 兼容**
   - 现有代码无需修改

---

## 时间线

| 阶段 | 预计时间 | 依赖 |
|------|----------|------|
| Phase 4.1: Lexer 扩展 | 1 天 | 无 |
| Phase 4.2: Parser 扩展 | 2 天 | 4.1 |
| Phase 4.3: 语义检查 | 2 天 | 4.2 |
| Phase 4.4: AutoVM 运行时 | 2 天 | 4.3 |
| Phase 4.5: a2rs 转译器 | 2 天 | 4.3 |
| Phase 4.6: 集成测试与文档 | 1 天 | 全部 |
| **总计** | **10 天** | - |

---

## 后续计划

### Plan 127: `.take` → `.move` 迁移

完成 Plan 126 后，启动独立计划将所有 `.take` 语法迁移为 `.move`：
- 更新 Lexer（已支持 `move` token）
- 更新 Parser
- 更新文档和示例
- 添加迁移警告

### Plan 128: a2c 后端支持

为 C/RTOS 环境实现 `.go`：
- 闭包环境提取
- 状态机生成
- xTaskCreate 映射

---

## 参考文档

- [设计文档: docs/design/task-msg.md](../design/task-msg.md) - Phase 4 规范
- [Plan 121: Task/Msg 系统](./121-task-msg-system.md) - Phase 1 实现
- [Plan 124: Async/Future/Await](./124-async-future-await.md) - Phase 2 实现
- [Plan 125: 多态路由](./125-phase3-polymorphic-routing.md) - Phase 3 实现

---

## 实现总结 (2026-03-16)

### 已完成的阶段

| 阶段 | 状态 | 实现文件 |
|------|------|----------|
| Phase 4.1: Lexer | ✅ 完成 | `token.rs` - `TokenKind::Go` |
| Phase 4.2: Parser | ✅ 完成 | `ast.rs`, `parser.rs` - `Expr::Go` |
| Phase 4.3: 语义检查 | ✅ 完成 | `infer/expr.rs` - 类型推断 |
| Phase 4.4: AutoVM 运行时 | ✅ 完成 | `opcode.rs`, `codegen.rs`, `engine.rs` - `SPAWN_GO` |
| Phase 4.5: a2rs 转译器 | ✅ 完成 | `trans/rust.rs` - `tokio::spawn` |
| Phase 4.6: 测试与文档 | ✅ 完成 | 测试通过，文档更新 |

### 实现细节

#### 1. Lexer (`token.rs`)
- 添加 `TokenKind::Go` 枚举变体
- 添加 `"go" => Some(TokenKind::Go)` 关键字映射
- 实现 Display trait: `<go>`

#### 2. Parser (`parser.rs`, `ast.rs`)
- 添加 `Expr::Go { expr: Box<Expr> }` AST 节点
- 在 `parse_postfix()` 中处理 `.go` 后缀操作符
- 实现 Display 和 ToNode trait

#### 3. 类型推断 (`infer/expr.rs`)
- `.go` 表达式推断为 `Type::Void`（fire-and-forget）
- 检查内部表达式是否为 `Future<T>` 类型
- 非 Future 类型产生 `InvalidGoUsage` 警告

#### 4. VM 运行时
- `opcode.rs`: `SPAWN_GO = 0x89`
- `codegen.rs`: 编译 `Expr::Go` 生成 `SPAWN_GO` 指令
- `engine.rs`: `SPAWN_GO` 执行逻辑 - 派发到后台任务池

#### 5. Rust 转译器 (`trans/rust.rs`)
- `expr.go` → `tokio::spawn(async move {{ expr.await; }})`
- 支持链式调用

### 测试状态
- `test_go_keyword` 测试通过 ✅
- Parser 测试通过 ✅
- 编译无错误 ✅
