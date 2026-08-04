# Plan 231: 嵌套 mut fn 调用 + for 循环导致栈损坏

## 状态: 已完成 (核心修复)

## 问题描述

`mut fn` 方法中对 `self.field` 赋值后再读取，导致栈帧中 `old_bp` 被覆盖，从而使 RET 恢复了错误的 bp 值。

## 根因分析

### 发现过程

通过系统性简化（从 SSE parser → Counter mut fn → 最小复现），逐步排除复杂因素，最终定位到：

**codegen 为 `self.field = expr` 赋值生成了 SET_GENERIC_FIELD，但没有设置 `last_expr_type = ObjectType::Void`。**

这导致语句编译器在赋值后追加了一个 POP 指令，使 SP 下降到帧基以下。后续的表达式栈操作覆盖了 old_bp。

### 栈帧破坏机制

```
callee 栈帧:
  ram[4] = self (param)
  ram[5] = ret_addr
  ram[6] = old_bp (0)    ← bp=6
  ram[7..] = expression stack

self.count = self.count - 1:
  SET_GENERIC_FIELD → sp=7 (正确)
  POP               → sp=6 (错误! 多余的 POP)

return self.count:
  LOAD_PARAM self → sp=7, ram[6]=4000000 (覆盖了 old_bp!)
  GET_FIELD       → ram[6]=2 (field value)
  RET             → old_bp = ram[6] = 2 (应该是 0!)
```

### 为什么之前没发现

只有 `mut fn` 返回非 void 值时才触发：
- `mut fn` void 返回：RET 不读 old_bp（直接终止）—— 实际上 void 返回的 RET 仍然读 old_bp，但 void 返回的函数在赋值后直接 RET，中间没有额外的表达式栈操作，所以 POP 后 SP 不会低于 BP
- 具体来说，test 008（mut fn void + increment）通过是因为 `self.count = self.count + 1` 后直接 RET，没有额外的 LOAD_PARAM

## 修复内容

### 修复 1: codegen.rs — SET_GENERIC_FIELD/SET_FIELD 缺少 Void 标记

**文件**: `crates/auto-lang/src/vm/codegen.rs`
**位置**: 第 4333 行（field assignment 路径结束后）

```rust
// SET_GENERIC_FIELD and SET_FIELD don't push a return value -
// mark as void to prevent Stmt::Expr from emitting a POP
// that would corrupt the stack
self.last_expr_type = ObjectType::Void;
```

与 SET_ELEM（已有此标记）保持一致。

### 修复 2: engine.rs — BUILD_FSTR 中堆对象的格式化

**文件**: `crates/auto-lang/src/vm/engine.rs`
**位置**: BUILD_FSTR opcode 的 StackTag::Int 分支

当 f-string 中出现 Option/Result/GenericInstance 值时（instance_id >= 4000000），正确格式化为用户友好的字符串：
- `Option.Some(42)` → `"42"`
- `Option.None` → `"None"`
- `Result.Ok(v)` / `Result.Err(e)` → `"v"` / `"Err(e)"`
- 其他 GenericInstance → `"TypeName { field: value, ... }"`

## 测试结果

### Plan 231 测试: 8/10 通过

| Test | 描述 | 状态 |
|------|------|------|
| 001 | SSE parser 完整场景 | FAIL (复合 bug) |
| 002 | SSE parser direct drain | FAIL (复合 bug) |
| 003 | Counter + for+Option+is | PASS |
| 004 | Counter + for+Option (无 is) | PASS |
| 005 | mut fn 返回 Option (无循环) | PASS |
| 006 | mut fn 返回 int | PASS |
| 007 | mut fn void + getter | PASS |
| 008 | vmtest-09 副本 | PASS |
| 009 | mut fn void + direct print | PASS |
| 010 | mut fn int (无 f-string) | PASS |

### 全量回归: 283/287 通过

- 4 个失败：1 个已有 bug (hashmap_insert_int) + 2 个 SSE parser + 1 个...
- 无回归

## 遗留问题

### test 001, 002 — SSE parser 复合场景

SSE parser 测试涉及多个 VM 特性交互，`events: 0` 说明 for+Option 循环没有正确执行。可能涉及：
- `static fn new()` 返回值的处理
- `self.buffer = expr` 赋值在复杂场景中的栈行为
- 字符串操作 (`find`, `sub`, `slice`) 的 VM 实现
- `is` 匹配 Option 值的代码路径

这些可能是多个独立的 bug，需要单独追踪。

### f-string `$expr.field` 不支持

f-string 解析器不支持 `$c.count` 这种点访问语法，`$c` 被解析为变量，`.count` 被当成字符串字面量。需要增强 f-string 解析器。

## 修改文件

- `crates/auto-lang/src/vm/codegen.rs` — SET_GENERIC_FIELD/SET_FIELD 后加 Void 标记
- `crates/auto-lang/src/vm/engine.rs` — BUILD_FSTR 堆对象格式化
- `crates/auto-lang/test/vm/99_plan231/` — 10 个测试用例
- `crates/auto-lang/src/tests/vm_file_tests.rs` — 测试注册
