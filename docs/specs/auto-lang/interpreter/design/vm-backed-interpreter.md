# VM 薄封装解释器（VmInterpreter 执行管线）

## 范围

`crates/auto-lang/src/interpreter/` 两个文件：`mod.rs`（`AutoInterpreter`，宿主接口）与
`vm_interpreter.rs`（`VmInterpreter`，执行核心）。本文描述执行管线、结果提取协议与
状态管理的不变量。

## 执行管线（`VmInterpreter::run`，vm_interpreter.rs:28-167）

每次 `run(code)` 全量执行一次迷你编译管线：

1. **解析**：`Parser::from(code).parse()` 得 AST。
2. **编译**：`Codegen` 逐语句编译为 ABC 字节码；除最后一条表达式语句外都设置
   `should_pop_expr_result`，使脚本最终栈顶恰好剩下最后一个表达式的值。
3. **HALT 追加**：尾部压 `OpCode::HALT`。
4. **RESERVE_STACK 插入**：若 `max_locals > 0`，在字节码头部插入 2 字节
   （opcode + count），并把 `exports`、`relocs`、`jump_placeholders` 全部 +2 平移
   （见 architecture.md ADR-04）。
5. **重定位**：按 `relocs` 把函数地址小端写入代码区。
6. **装载执行**：`VirtualFlash::new_with_code_and_keys(code, object_keys, object_types)`，
   在全局 tokio runtime 上 `block_on`：`AutoVM::new(flash, 4096)` → 装载字符串池与
   泛型注册表（plan-197 Task 9）→ 以 `main` 导出地址（缺省 0）`spawn_task` →
   `run_task_loop()`。
7. **结果提取**：任务结束后读其 RAM 栈顶 `raw_nv[sp-1]`，按 nanbox 标记解码。

## 结果提取协议（vm_interpreter.rs:116-163）

- 栈顶按 `auto_val::is_string / is_f64 / is_f32` 判定后解码；
- 其余按 i32 解码，并按对象 ID 区间判定堆对象：`[1_000_000, 2_000_000)` 为对象、
  `[2_000_000, 3_000_000)` 为数组，从 `vm.objects` / `vm.arrays` 取回并重建 `Value`；
- 提取不到任何值时返回 `Value::Nil`。

不变量：**脚本能拿到返回值的前提是最后一条语句是表达式且未被 pop**（步骤 2 的
`should_pop_expr_result` 安排）。bool / nil / 函数等类型不在解码分支内，统一落 Nil。

## 状态管理

- `exports`（函数名 → 地址）在每次 `run` 后被**整体覆盖**为最新一次编译的导出表，
  因此 `has_function` / `get_functions` 只反映最近一次 `eval`。
- `globals` 是一张独立侧表：`set_global` / `get_global` / `merge_atom` 只读写它，
  `run()` 不读——**注入的全局对被执行代码不可见**（见 overview 已知坑）。
- `AutoInterpreter` 层另有 `cache: HashMap<String, Code>` 字段与 `_persistent` 标志，
  当前 `eval` 均未使用；`reset()` 会清空两侧状态。
- `AutoInterpreter::call(fn_name, args)` 转发到 `VmInterpreter::call`，后者是
  TODO 桩，恒返回 `Value::Nil`。

## 显式非目标

- **不做 TreeWalker**：旧 `eval.rs` / `interp.rs` 已删除（plan-091），本模块不重新引入 AST 直走。
- **不做增量编译 / 持久 session**：那是 `CompileSession`（compile.rs / database）的职责；
  本模块每次 eval 都是全量编译。
- **不做函数级调用**：`call()` 未实现，跨 eval 的函数复用不存在（exports 每次被覆盖）。
- **不复制 `run_autovm` 的 4MB 独立线程方案**：当前直接 `block_on` 全局 runtime。

> 来源: crates/auto-lang/src/interpreter/mod.rs、crates/auto-lang/src/interpreter/vm_interpreter.rs、docs/plans/old/091-universe-removal.md、docs/plans/old/080-autovm-stack-frame-bug.md、docs/plans/old/197-vm-adt-generic-lists-pattern-debug.md
