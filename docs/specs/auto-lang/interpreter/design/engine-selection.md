# 引擎选择层（ExecutionEngine）

## 范围

`crates/auto-lang/src/execution_engine.rs`（约 100 行）：`ExecutionEngine` 枚举、
环境变量覆盖、`execute_with_engine` / `execute_with_engine_capture` 分发。
它是 `lib.rs` 高层 API（`run` lib.rs:261、`run_with_capture` lib.rs:271）的唯一入口。

## 机制

```
run(code) / run_with_capture(code)
    -> ExecutionEngine::get()          # from_env() 否则 default_engine()
    -> execute_with_engine(engine, code)
        -> 所有分支: crate::run_autovm(code) / run_autovm_capture(code)
```

- `default_engine()` 恒为 `AutoVM`（plan-081，ADR-01）。
- `from_env()` 读 `AUTO_EXECUTION_ENGINE`：`autovm`/`vm` → AutoVM；
  `evaluator`/`eval`/`tree` → `eprintln!` 警告后仍返回 AutoVM（plan-091，ADR-02）；
  其他值或未设置 → 默认 AutoVM。注意 `from_env` 对任何环境值都返回 `Some`，
  不存在"回退到编译期默认"的 `None` 路径。
- `Evaluator` 变体仍在枚举中但标 `#[deprecated(since = "0.10.0")]`，
  `execute_with_engine` 中两个分支等效（都调 `run_autovm`）。

## 不变量

- **单引擎语义**：无论选择层返回什么，最终执行路径只有 AutoVM 一条。
- 选择层与 `interpreter/` 无调用关系：`AutoInterpreter` / `VmInterpreter` 不经
  `ExecutionEngine`，直接自建管线（见 design/vm-backed-interpreter.md）。
- `run_autovm` 系列在独立 4MB 栈线程里 `block_on` 全局 tokio runtime
  （lib.rs:333-358），避免 Windows 主线程 1MB 栈溢出（plan-355(archive) 的同类教训）。

## 显式非目标

- **不再是多引擎抽象**：文档 `docs/execution-engine-selection.md` 描述的
  "编译期 feature flag 双引擎"与"Evaluator fallback"均已从代码移除，该文档仅作历史参考。
- **不做按依赖选引擎**：plan-081 Phase 2（pac.at 中 per-dependency 执行模式）属
  auto-man / 构建侧职责，不在本文件。

> 来源: crates/auto-lang/src/execution_engine.rs、crates/auto-lang/src/lib.rs、docs/execution-engine-selection.md、docs/plans/old/081-autovm-default-mode.md、docs/plans/old/091-universe-removal.md
