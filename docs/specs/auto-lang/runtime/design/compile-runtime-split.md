# 编译期/运行期分层（Plan 064 架构）

## 范围

`runtime.rs`（ExecutionEngine/StackFrame）与 `scope.rs`（Sid/SymbolTable/Scope）构成的
编译期-运行期双态结构。不含 AutoVM 字节码执行循环（属 vm 模块）。

## 原则

- 编译期状态持久、运行期状态易失；两者只能单向引用（运行期 → 编译期）。
- 一个符号表可对应多个栈帧（递归），反之不然。
- 运行期不复制声明信息，只持 `scope_sid` 链接。

## 细节

### 数据结构

```text
Compile-time (Database)          Runtime (ExecutionEngine)
┌──────────────────┐            ┌──────────────────┐
│   SymbolTable    │◄───────────│   StackFrame      │
│ - kind, sid      │  scope_sid │ - scope_sid       │
│ - parent, kids   │            │ - vals: name→VID  │
│ - symbols, types │            │ - moved_vars      │
└──────────────────┘            │ - parent_frame    │
                                └──────────────────┘
```

- `Sid`：点分路径作用域标识（`"a.b.c"`），空串为全局（Display 为 `🌳`）。
  `kid_of`/`parent`/`name` 提供层级运算；`SID_PATH_GLOBAL` 为全局单例。
- `SymbolTable`（scope.rs:157）：`kind`（Global/Mod/Type/Fn/Block）+ `sid` + 层级 +
  `symbols`/`types` 两张 `HashMap<AutoStr, Rc<Meta>>`。无任何运行期值。
- `StackFrame`（runtime.rs:51）：`vals: HashMap<AutoStr, ValueID>`、
  `moved_vars: HashSet<AutoStr>`（use-after-move 跟踪）、`cur_block`（break/continue 定位）、
  `parent_frame`（返回链）。
- `ExecutionEngine`（runtime.rs:126）：`call_stack: Vec<StackFrameId>` +
  `frames: HashMap<StackFrameId, RefCell<StackFrame>>`；值本体集中在
  `values: HashMap<ValueID, Rc<RefCell<ValueData>>>`，配 `weak_refs` 供清理；
  另持 `vm_refs`（文件句柄、集合等 VM 资源）、`env_vals`、`args`、`builtins`、
  `shared_vals`（跨多次求值共享）。

### 不变量

- `lookup_var` 自栈顶向栈底查找，内层帧遮蔽外层（词法作用域）。
- `push_frame` 自动把新帧链接到栈顶帧为父；`pop_frame` 只出栈不销毁帧数据
  （代码注释明示留待检查用途，孤儿帧清理是未来工作）。
- 帧内只存 `ValueID` 不存值；值的生命周期由 `values` 表的 `Rc` 决定。
- `SymbolTable` 在增量编译间持久；`StackFrame` 随作用域退出而失效。

### 遗留

`Scope`（scope.rs:259）是分层前的混合结构，已标 DEPRECATED：`get_val` 恒返 `None`，
`vals` 已改存 `ValueID`。`SymbolTable::from_scope` 是官方迁移路径。

## 显式非目标

- 不做 AutoVM 字节码级栈帧（VirtualRAM/BP/SP 属 vm 模块的 `AutoTask`）。
- 不做垃圾回收策略设计；`weak_refs` 仅是清理挂钩，非 GC。
- 不保证 `pop_frame` 后的帧数据回收（已知遗留，见代码注释）。

> 来源: docs/plans/old/064-split-universe-compile-runtime.md、crates/auto-lang/src/runtime.rs、crates/auto-lang/src/scope.rs、docs/design/05-vm-runtime.md（Streaming/REPL 节的"语句级栈平衡"原则）
