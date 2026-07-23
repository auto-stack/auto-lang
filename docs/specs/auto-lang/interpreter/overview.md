# interpreter（编程式求值接口 + 引擎选择）

> **Status**: partial（AutoVM 薄封装可用；`call()` 未实现、结果提取不完整）

## 职责

两层职责：

1. `interpreter/`：在 AutoVM 之上提供编程式求值接口（`AutoInterpreter` / `VmInterpreter`），
   供 auto-gen 模板渲染、auto-man 资产处理、UI 桥接、编译期求值等宿主代码以
   "传字符串、拿 `Value`" 的方式执行 Auto 代码。
2. `execution_engine.rs`：执行引擎选择层（`ExecutionEngine` 枚举 + 环境变量覆盖），
   是 `lib.rs` 高层 API（`run` / `run_with_capture`）的统一入口。

**注意**：本模块已不是 TreeWalker 解释器。旧 `eval.rs` / `interp.rs`（约 7,167 行）
已在 plan-091 中删除（commit `6862bb4`），现 `interpreter/` 是 plan-091 之后重建的
AutoVM 薄封装。模块名沿用了"interpreter"，语义已变为"解释器外观的 VM 接口"。

## 现状

- `AutoInterpreter::eval` 走完整管线：Parser → Codegen（ABC 字节码）→ 重定位 →
  VirtualFlash → AutoVM 任务执行 → 从任务栈顶提取 `Value`（vm_interpreter.rs `run()`）。
- 引擎选择实质上是单引擎：`ExecutionEngine::Evaluator` 变体标记 `#[deprecated]`，
  选择后只打印警告并重定向到 AutoVM（plan-091）；`AUTO_EXECUTION_ENGINE=evaluator`
  等值同样只告警。编译期 `use-evaluator` feature 路径在代码中已不存在。
- 每次 `eval` 都是独立编译执行（parse + codegen 全量重做）；`AutoInterpreter.cache`
  字段存在但未被 `eval` 使用。持久 session / 增量编译是 `CompileSession`
  （`crates/auto-lang/src/compile.rs`）的职责，不在本模块。

## 关键入口

- `crates/auto-lang/src/interpreter/mod.rs:AutoInterpreter` — 推荐宿主接口（eval / eval_template / merge_atom）
- `crates/auto-lang/src/interpreter/vm_interpreter.rs:VmInterpreter` — VM 封装与结果提取
- `crates/auto-lang/src/execution_engine.rs:ExecutionEngine` — 引擎枚举（`get()` / `from_env()`）
- `crates/auto-lang/src/execution_engine.rs:execute_with_engine` — 按引擎执行（全部落到 `run_autovm`）
- `crates/auto-lang/src/lib.rs:run` / `run_autovm` / `run_with_capture` — 高层 API
- `crates/auto-lang/src/lib.rs:get_global_runtime` — 共享 tokio runtime（VmInterpreter 依赖）

## 使用示例

```rust
// 宿主代码求值（auto-gen 的用法，见 crates/auto-gen/src/lib.rs）
let mut interp = auto_lang::interpreter::AutoInterpreter::new().with_fstr_note('$');
let value = interp.eval_template("", "Hello $name")?;

// 一次性执行（走引擎选择层，环境变量可覆盖）
let result = auto_lang::run("1 + 2")?;            // -> "3"，实际总是 AutoVM
let (result, stdout) = auto_lang::run_with_capture(code)?;  // plan-177 捕获 stdout
```

## 已知坑

- `VmInterpreter::call()` 未实现，恒返回 `Value::Nil`（vm_interpreter.rs:170）；
  mod.rs 中 `test_function_call` 等 3 个测试因此 `#[ignore]`。
- 结果提取只覆盖栈顶的 int/f32/f64/string/object/array（按 nanbox 标记与对象 ID
  区间 1M–3M 解码）；其余类型一律返回 `Value::Nil`。
- `set_global` / `merge_atom` 写入的是 `VmInterpreter.globals` 侧表，**不会注入 VM
  执行环境**——`run()` 不读它；`get_global` 能读到，但 `eval` 的代码看不到。
- `VmInterpreter::run` 直接 `block_on` 全局 runtime，**不像** `run_autovm` 那样另开
  4MB 栈线程（plan-355(archive) 的教训）；深递归解析场景需注意调用线程栈余量。
- `docs/execution-engine-selection.md` 已过时：它宣称 Evaluator 可作 fallback
  且有编译期 feature 开关，实际代码中两者均已移除（见 architecture.md ADR-02）。

## 蒸馏来源（Phase 1）

- `docs/design/01-architecture.md`、`docs/execution-engine-selection.md`（主要来源）
- 代码核对：`crates/auto-lang/src/interpreter/`、`crates/auto-lang/src/execution_engine.rs`、`crates/auto-lang/src/lib.rs`
- plan 佐证：plan-068 / 073 / 075 / 080 / 081 / 091 / 177 / 197 / 221 / 298（见 plans.md）
