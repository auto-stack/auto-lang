# Plan 068 Phase 9.6: AutoVM REPL Persistence

## Status: 部分完成 ⚠️

### 当前实现

**已完成** ✅：
- 单个 AutoVM 实例保持（跨输入）
- 函数定义持久化（exports 表）
- 堆对象持久化（heap_objects, arrays, closures, iterators, channels）
- REPL 命令支持（`:stats`, `:reset`, `:help`, `:quit`）
- Session 统计信息

**限制** ⚠️：
- 局部变量**不**持久化（栈机器架构限制）
- 没有全局变量系统（类似 Universe）

### 架构对比

#### 旧 REPL（Interpreter + Universe）

```rust
pub struct ReplSession {
    pub session: CompileSession,
    pub engine: Rc<RefCell<ExecutionEngine>>,
    pub scope: Shared<Universe>,  // ← 所有可能的值
}
```

**持久化机制**：
- `Universe` 持有所有变量：`locals: HashMap<String, Value>`
- 每次执行使用同一个 `scope`
- 变量自然持久化

**执行流程**：
```rust
pub fn run(&mut self, code: &str) -> AutoResult<String> {
    // 创建新的 Interpreter，但使用同一个 scope
    let mut interpreter = Interpreter::new_with_session_and_scope(
        &mut self.session,
        self.scope.clone()  // ← 同一个 scope！
    );
    interpreter.interpret(code)?;
    Ok(interpreter.result.repr().to_string())
}
```

#### 当前 AutoVM REPL

```rust
pub struct AutovmReplSession {
    vm: AutoVM,  // ← 单个实例，跨输入
    globals: HashMap<String, Value>,  // ← 未实现
    // ...
}
```

**持久化机制**：
- VM 实例保持 → heap_objects, arrays, closures 等保持 ✅
- 函数定义保持 → exports 表保持 ✅
- **每次创建新 task** → 栈被重置 → 局部变量丢失 ❌

**执行流程**：
```rust
pub fn run(&mut self, code: &str) -> AutoResult<String> {
    // 编译代码
    // ...

    // 创建新 task（新栈）
    let task_id = self.vm.spawn_task(new_code_start, 1024);  // ← 新栈！

    // 执行
    self.vm.run_task_loop().await;

    // 获取结果（栈上的值）
    task.ram.pop_i32()  // ← 栈会被清空
}
```

### 为什么局部变量不持久化？

**AutoVM 是栈机器**（Stack-Based VM）：

```
栈帧结构：
┌─────────────────────┐
│  局部变量 0          │  ← bp+0 (STORE_LOC_0, LOAD_LOC_0)
│  局部变量 1          │  ← bp+1 (STORE_LOC_1, LOAD_LOC_1)
│  ...                │
├─────────────────────┤
│  返回地址            │
│  旧 bp              │
├─────────────────────┤
│  临时计算空间        │  ← sp (栈顶)
└─────────────────────┘
```

**操作码**：
- `STORE_LOC_N` - 将栈顶值存储到 bp+N
- `LOAD_LOC_N` - 将 bp+N 的值加载到栈顶

**问题**：每次创建新 task 时，新栈被分配，bp=0, sp=0，所以局部变量丢失。

### 当前实现的行为

**可以工作的** ✅：
```auto
// 1. 函数定义持久化
fn add(a int, b int) int { return a + b }
add(10, 20)  // → 30

// 2. 堆对象持久化（如果实现了）
let list = List.new()
list.push(1)
// list 在 VM.heap_objects 中保持
```

**不工作的** ❌：
```auto
// 局部变量不持久化
let x = 10
x + 1  // → 错误或错误结果（x 丢失）
```

### 解决方案

#### 方案 1：全局变量系统（推荐）

**实现**：
1. 添加新的操作码：
   - `LOAD_GLOBAL name_index` - 从全局变量表加载
   - `STORE_GLOBAL name_index` - 存储到全局变量表

2. 在 AutoVM 中添加全局变量存储：
```rust
pub struct AutoVM {
    // ...
    pub globals: HashMap<String, Value>,  // 全局变量
}
```

3. 编译器识别全局变量：
```auto
global x = 10  // ← 使用 global 关键字
x + 1  // → LOAD_GLOBAL "x"
```

**优点**：
- 类似 Python 的 `global` 关键字
- 明确的变量作用域
- 与栈机器模型兼容

**缺点**：
- 需要修改编译器
- 需要添加新的操作码
- 用户需要显式声明全局变量

#### 方案 2：REPL 级别的变量存储（临时方案）

**实现**：
在 `AutovmReplSession` 中维护全局变量：
```rust
pub struct AutovmReplSession {
    // ...
    pub repl_globals: HashMap<String, Value>,  // REPL 级别的全局变量
}
```

每次执行前：
1. 将 `repl_globals` 加载到栈上
2. 执行代码
3. 将栈上的值保存回 `repl_globals`

**优点**：
- 不需要修改 VM 架构
- 相对简单实现

**缺点**：
- 不适用于函数内部
- 性能开销（每次复制变量）

#### 方案 3：混合方案（当前方向）

**实现**：
- 保持当前实现（函数定义持久化）
- 文档化限制（局部变量不持久化）
- 推荐用户使用 `old-repl` 进行复杂交互

**优点**：
- 无需额外工作
- 清晰的使用场景

**缺点**：
- 用户体验不一致

### 推荐路径

**短期**（当前）：
- ✅ 完成当前实现
- ✅ 文档化限制
- ✅ 提供 `old-repl` 作为替代

**中期**（Plan 069）：
- 实现全局变量系统（方案 1）
- 添加 `LOAD_GLOBAL` / `STORE_GLOBAL` 操作码
- 编译器支持 `global` 关键字

**长期**（Plan 070+）：
- 考虑更先进的 REPL 持久化方案
- 支持 REPL 级别的元编程
- 交互式调试和 introspection

### 用户指南

**当前 AutoVM REPL 适合**：
- 函数定义和测试
- 堆对象操作（List, Map 等）
- 快速表达式求值

**当前 AutoVM REPL 不适合**：
- 多步骤状态管理
- 依赖局部变量持久化的代码

**替代方案**：
- 使用 `auto.exe old-repl` 获得完整的变量持久化
- 将代码保存到 `.at` 文件并运行

### 测试

```bash
# 测试当前 AutoVM REPL
auto.exe
> fn add(a int, b int) int { return a + b }
> :stats
> add(10, 20)  # → 30 ✅

# 对比 old-repl
auto.exe old-repl
>> let x = 10
>> x + 1  # → 11 ✅
```

## 总结

当前 AutoVM REPL 实现了**部分持久化**：
- ✅ 函数定义保持
- ✅ 堆对象保持
- ❌ 局部变量不保持

这是由 AutoVM 的栈机器架构决定的。要实现完整的变量持久化，需要添加全局变量系统（Plan 069）。

对于需要完整变量持久化的场景，用户应该使用 `old-repl` 或将代码保存到文件中。
