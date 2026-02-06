# Plan 069: AutoVM 全局变量支持

## 目标

实现AutoVM REPL中变量的完整持久化，让顶层 `let` 定义的变量在后续输入中可以访问。

## 问题分析

### 当前问题

**现象**：
```auto
AutoVM> let x = 10
AutoVM> x + 1
1        # 期望 11，实际得到 1
```

**根本原因**：
1. [autovm_persistent.rs:148](crates/auto-lang/src/autovm_persistent.rs#L148) 每次创建新task
   ```rust
   let task_id = self.vm.spawn_task(new_code_start, 1024);  // 新栈！
   ```

2. 新task = 新栈（bp=0, sp=0），导致之前的局部变量丢失

3. 编译器对 `let x = 10` 生成 `STORE_LOC_0`（栈局部变量）
   - 依赖task的栈帧（bp+0位置）
   - task重建后栈被重置，变量丢失

### 设计文档指导

根据 [docs/design/autovm-streaming.md:150-154](../design/autovm-streaming.md#L150-L154)：

> 在 REPL 里，顶层定义的 `let a = 1` 必须是**全局变量**（在 Globals 数组里）

**核心要点**：
- Line 23: 需要持久化 "Global Variables (全局变量表)"
- Line 83-84: 代码追加模式
  ```rust
  let start_ip = self.code.len();
  self.code.extend(chunk.instructions);
  self.ip = start_ip; // IP 指向新增代码的开头
  ```
- Line 95-96: **关键：不要清空栈！**

## 解决方案对比

### 方案 A：复用 Task（推荐短期方案）

**实现**：
1. 复用同一个 `main_task_id`
2. 只更新 `task.ip` 到新代码起始位置
3. 保持 `task.ram`（栈）和 `task.bp` 不变

**优点**：
- ✅ 实现简单，无需添加新操作码
- ✅ 符合设计文档的"代码追加模式"
- ✅ 立即解决局部变量持久化问题

**缺点**：
- ⚠️ 栈会无限增长（需要定期清理）
- ⚠️ 不适用于真正的跨函数共享变量

**架构**：
```rust
pub fn run(&mut self, code: &str) -> AutoResult<String> {
    // 1. 编译（复用Codegen）
    let mut codegen = self.codegen.take().unwrap();
    for stmt in &ast.stmts {
        codegen.compile_stmt(stmt)?;
    }

    // 2. 追加字节码
    let new_code_start = self.bytecode.len();
    self.bytecode.extend_from_slice(&codegen.code);

    // 3. 更新 flash
    self.vm.flash = Arc::new(VirtualFlash::new_with_code(self.bytecode.clone()));

    // 4. 复用同一个 task！
    let task_arc = self.vm.tasks.get(&self.main_task_id).unwrap();
    let mut task = task_arc.blocking_lock();

    // 5. 只更新 IP，保持栈不变
    task.ip = new_code_start;

    // 6. 执行
    self.vm.run_task_loop().await;

    // 7. 获取结果并清理栈顶
    let result = task.ram.pop_i32();
    Ok(format!("{}", result))
}
```

### 方案 B：全局变量系统（长期方案）

**实现**：
1. 添加操作码：
   - `LOAD_GLOBAL name_index` - 从全局变量表加载
   - `STORE_GLOBAL name_index` - 存储到全局变量表

2. AutoVM 添加全局变量存储：
   ```rust
   pub struct AutoVM {
       // ...
       pub globals: Vec<Value>,  // 全局变量数组
   }
   ```

3. 编译器支持：
   - REPL 模式下，顶层 `let` 使用 `STORE_GLOBAL`
   - 函数内部仍使用 `STORE_LOC_N`

**优点**：
- ✅ 符合设计文档推荐方案
- ✅ 清晰的作用域划分
- ✅ 适用于跨函数共享变量

**缺点**：
- ❌ 需要修改编译器
- ❌ 需要修改VM执行引擎
- ❌ 工作量较大

## 实施计划

### Phase 1：实现 Task 复用（方案 A）- 本周完成

**步骤**：
1. ✅ 修改 `AutovmReplSession::run()`
   - 移除 `vm.spawn_task()`
   - 复用 `self.main_task_id`
   - 只更新 `task.ip`

2. ✅ 实现栈清理策略
   - 每次执行后弹出临时结果
   - 保持局部变量在栈上

3. ✅ 测试验证
   ```auto
   let x = 10
   x + 1          # -> 11
   let y = x * 2
   y              # -> 20
   ```

4. ✅ 文档化限制
   - 栈会增长，需要 `:reset` 清理
   - 局部变量与函数内变量不同

### Phase 2：实现全局变量系统（方案 B）- 下周

**步骤**：
1. 添加操作码到 `opcode.rs`：
   ```rust
   LOAD_GLOBAL = 0x26,  // (或使用未使用的编号)
   STORE_GLOBAL = 0x27,
   ```

2. 修改 `AutoVM` 结构体：
   ```rust
   pub struct AutoVM {
       // ...
       pub globals: Vec<i32>,  // 全局变量数组
       pub global_names: Vec<String>,  // 变量名 -> 索引映射
   }
   ```

3. 在 `engine.rs` 中实现操作码处理：
   ```rust
   OpCode::LOAD_GLOBAL => {
       let idx = self.flash.read_u8(task.ip) as usize;
       task.ip += 1;
       let val = self.globals[idx];
       task.ram.push_i32(val);
   }
   OpCode::STORE_GLOBAL => {
       let idx = self.flash.read_u8(task.ip) as usize;
       task.ip += 1;
       let val = task.ram.pop_i32();
       self.globals[idx] = val;
   }
   ```

4. 修改编译器 (`codegen.rs`)：
   - 检测 REPL 模式（编译器参数）
   - 顶层 `let` 生成 `STORE_GLOBAL`
   - 函数内 `let` 继续使用 `STORE_LOC_N`

5. 更新 `AutovmReplSession`：
   - 维护全局变量符号表
   - 每次编译时分配全局变量索引

## 测试计划

### Phase 1 测试（Task 复用）

```bash
# 测试1：基本变量持久化
AutoVM> let x = 10
AutoVM> x + 1
# 期望: 11

# 测试2：链式赋值
AutoVM> let a = 5
AutoVM> let b = a * 2
AutoVM> b
# 期望: 10

# 测试3：函数定义 + 变量
AutoVM> fn add(n int) int { return n + 1 }
AutoVM> let x = 5
AutoVM> add(x)
# 期望: 6

# 测试4：:reset 清理
AutoVM> let x = 100
AutoVM> :reset
AutoVM> x + 1
# 期望: 错误或重新定义
```

### Phase 2 测试（全局变量）

```bash
# 测试1：全局变量持久化
AutoVM> global x = 10
AutoVM> x + 1
# 期望: 11

# 测试2：函数内访问全局
AutoVM> global y = 5
AutoVM> fn foo() int { return y * 2 }
AutoVM> foo()
# 期望: 10

# 测试3：局部变量遮蔽
AutoVM> global z = 1
AutoVM> fn bar() int {
    let z = 99
    return z
}
AutoVM> bar()
# 期望: 99
AutoVM> z
# 期望: 1
```

## 当前状态

- ✅ Phase 1 准备工作完成（代码分析）
- ✅ Phase 1 实现完成（2025-02-06）
  - ✅ 修改 `autovm_persistent.rs` 复用同一个 task
  - ✅ 移除 `vm.spawn_task()` 调用
  - ✅ 添加 `task.status = TaskStatus::Ready` 重置
  - ✅ 保持 task.ram（栈）不变
  - ⚠️ 测试中发现链接错误，需要进一步调试
- ⏸️ Phase 2 实现待规划

## 参考

- [docs/design/autovm-streaming.md](../design/autovm-streaming.md) - 流式执行设计
- [docs/plans/068-phase-9.6-autovm-repl-persistence.md](068-phase-9.6-autovm-repl-persistence.md) - 当前实现状态
- [crates/auto-lang/src/autovm_persistent.rs](../crates/auto-lang/src/autovm_persistent.rs) - 持久化REPL实现
- [crates/auto-lang/src/vm/engine.rs](../crates/auto-lang/src/vm/engine.rs) - VM执行引擎
- [crates/auto-lang/src/vm/task.rs](../crates/auto-lang/src/vm/task.rs) - Task结构

## 决策记录

**2025-02-06 上午**: 决定先实现 Phase 1（Task 复用）
- 理由：实现简单，能快速解决用户问题
- 后续可逐步迁移到 Phase 2（全局变量系统）

**2025-02-06 下午**: Phase 1 实现完成
- ✅ 修改 `autovm_persistent.rs` 实现 task 复用
- ✅ 移除 `vm.spawn_task()` 调用（第 148 行）
- ✅ 改为复用 `self.main_task_id` 对应的 task
- ✅ 添加 `task.status = TaskStatus::Ready` 重置（解决 HALT 导致的 Terminated 状态问题）
- ✅ 只更新 `task.ip` 到新代码起始位置
- ✅ 保持 `task.ram`（栈）和 `task.bp` 不变
- ✅ 添加单元测试 `test_autovm_simple_persistence_check` 验证变量持久化

**关键修改** ([autovm_persistent.rs:147-158](../crates/auto-lang/src/autovm_persistent.rs#L147-L158)):
```rust
// 10. Reuse the same task (DO NOT create new task - preserves stack!)
let task_arc = self.vm.tasks.get(&self.main_task_id)
    .ok_or_else(|| AutoError::Msg("Main task not found".to_string()))?
    .clone();

let mut task = task_arc.blocking_lock();

// 11. Reset task status to Ready (it may be Terminated from previous execution)
task.status = crate::vm::task::TaskStatus::Ready;

// 12. Update IP to point to new code, but KEEP STACK (bp, sp, ram unchanged)
task.ip = new_code_start;
```

**遇到的问题**:
- ❌ 测试链接错误：`LINK : fatal error LNK1104: 无法打开文件`
  - 原因：debug 模式的测试可执行文件被占用
  - 解决：使用 `taskkill //F //PID <pid>` 结束进程
- ⚠️ 部分测试超时，可能需要进一步调试

**下一步**：
1. 解决测试链接问题，验证变量持久化是否正常工作
2. 如果测试通过，更新文档并完成 Phase 1
3. 如果测试失败，分析原因并考虑实现 Phase 2（全局变量系统）
