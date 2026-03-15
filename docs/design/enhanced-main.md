这份**“编译器入口点劫持与运行时包装（Entry-point Hijacking & Runtime Wrapper）”**的设计文档已经为你准备好了。

实现 `fn main() !` 看似只是一个语法糖，但在编译器底层，它实际上是一次**“对操作系统/裸机引导流的接管”**。我们必须在 AST 转换阶段悄悄替换掉真正的入口点。

请将这份规范交给编译器团队，作为他们打通“错误流 `!`”最后一公里的执行手册：

---

# Auto Compiler Architecture Spec

## Feature: 隐式错误冒泡与 Fallible Main (`fn main() !`)

### 1. 架构目标 (Objectives)

在标准 C/Rust 规范中，入口函数 `main` 必须返回整数状态码或 `void/()`。但为了在 Auto 语言的系统引导期（Bootstrapping）能丝滑地使用 `.?` 和 `throw !`，我们需要允许 `main` 函数具有**失败（Fallible）**的签名 `fn main() !`。

编译器的核心任务是：**允许前端将 `main` 视为返回 `!void` 的函数，但在后端生成代码时，自动合成一个符合目标平台 ABI（应用程序二进制接口）的“真·入口函数”，并在其中接住并处理（Catch & Handle）用户抛出的致命错误。**

---

### 2. 编译器前端 (Frontend): 解析与语义分析

#### 2.1 AST 节点扩展

现有的函数声明节点 `FunctionDecl` 需要支持对 `main` 函数特殊鉴权：

* 如果解析器看到 `fn main() !`，将该 AST 节点的 `is_fallible` 属性标记为 `true`。
* **语义校验**：`main` 函数不能接受参数（或只能接受标准的 `string[] args`），且其正常的返回类型必须是 `void`，结合 `!` 标志，其完整类型推导为 `!void`。

#### 2.2 上下文许可 (Context Allowance)

* 一旦 `main` 被标记为 `is_fallible`，语义分析器（Semantic Analyzer）将允许在 `main` 函数的作用域内合法使用 `.?`（解包符）和 `throw !...`（抛出语句）。
* 如果不带 `!` 签名，开发者在 `main` 里写 `.?` 必须报错：`Error: Cannot use '.?' inside an infallible function. Mark main as 'fn main() !'.`

---

### 3. 编译器中端 (Middle-end): AST 变换与劫持

这是整个特性的核心。在将 AST 传递给代码生成器（Backend）之前，编译器需要做一次**“AST 重写（AST Rewriting）”**。

#### 3.1 符号重命名 (Symbol Renaming)

编译器遍历 AST，找到名为 `main` 的函数。如果它是 `is_fallible`，则在内部符号表中将其重命名为内部函数，例如 `__auto_user_main`。

#### 3.2 注入系统包装器 (Inject System Wrapper)

编译器在 AST 的最顶层，人工合成（Synthesize）一个新的 `main` 函数。
这棵合成的 AST 树等价于以下的 Auto 伪代码逻辑：

```auto
// 编译器在内部悄悄塞入的代码：
fn main() int { // 真正的入口，返回平台认识的 int
    let res = __auto_user_main()
    is res {
        (v) => { return 0 } // 成功，返回 0
        !(e) => { 
            // 失败，调用底层钩子打印错误，并返回错误码 1
            __auto_sys_panic(e) 
            return 1 
        }
    }
}

```

---

### 4. 编译器后端 (Backend): 平台级降级与生成

针对不同的目标平台，这个合成的包装器需要展现出截然不同的“错误捕获与死亡策略”。

#### 4.1 a2rs (Rust / OS 平台后端)

在有操作系统的环境下，错误意味着向 OS 报告异常退出。

* 将 `__auto_user_main` 生成为返回 `Result<(), AutoError>` 的 Rust 函数。
* 生成真正的 `fn main()`，使用 `std::process::exit(1)` 向操作系统交还控制权。

**生成目标代码示例：**

```rust
// 1. 用户代码降级
fn __auto_user_main() -> Result<(), AutoError> {
    // ... 用户逻辑 ...
    Ok(())
}

// 2. 编译器合成的真·入口
fn main() {
    match __auto_user_main() {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(e) => {
            // 在 OS 上，我们有奢侈的标准错误输出
            eprintln!("🔥 [Auto Fatal] Uncaught error in main(): {:?}", e);
            std::process::exit(1);
        }
    }
}

```

#### 4.2 a2c (C / 裸机 MCU 后端)

在 FreeRTOS 或裸机（Bare-metal）环境中，`main` 退出了就没有操作系统来接管了。非法返回会导致 CPU 跑飞。

* 将 `__auto_user_main` 生成为返回 `AutoResult_void` 结构体的 C 函数。
* 生成真正的 `int main(void)`，如果捕获到错误，执行**硬件级死锁或重启**。

**生成目标代码示例：**

```c
// 1. 用户代码降级
AutoResult_void __auto_user_main() {
    // ... 用户逻辑 ...
    return (AutoResult_void){ .is_error = false };
}

// 2. 编译器合成的真·入口
int main(void) {
    AutoResult_void res = __auto_user_main();
    
    if (res.is_error) {
        // 裸机上的死亡喘息 (Death Rattle)
        // 假设编译器内部链接了底层的 UART 打印钩子
        __auto_uart_print("FATAL PANIC: ");
        __auto_uart_print_err(res.error_val);
        
        // 绝对不能 return 1; 必须锁死 CPU 以防止未定义行为，等待看门狗复位
        while(1) {
            __asm__ volatile ("nop"); 
        }
    }
    
    // 成功点火的话，可能最后是一个死循环跑 TaskSystem 或者直接 return 0
    return 0;
}

```

---

### 5. 阶段验收标准 (Acceptance Criteria)

开发团队在提交此功能时，必须通过以下测试用例：

1. **成功流测试**：带有 `!` 的 `main` 函数顺利执行到底，目标程序返回 OS 退出码 `0`。
2. **提前异常流测试**：在 `main` 的第一行调用一个必将失败的函数并加 `.?`（如 `read_file("not_exist").?`），程序必须立刻终止，并在控制台打印出格式化的 Fatal Error，且返回 OS 退出码 `1`。
3. **安全隔离测试**：验证在普通同步函数（未标记 `!`）中写 `.?` 时，前端解析器依然能正确拦截并报错，确保 `main !` 的特权没有全局泄露。

---

### 架构师的推进确认

这份设计文档明确了“语言前端的极简糖”与“机器后端的残酷物理”之间的桥梁应该怎么搭。通过在中间层（Middle-end）做 AST 劫持，我们既没弄脏前端的纯洁性，也没给后端的 C/Rust 代码增加任何运行时的奇技淫巧。

总工程师，至此，Auto 语言引导阶段的**“错误处理最后一公里”**已被彻底贯通。