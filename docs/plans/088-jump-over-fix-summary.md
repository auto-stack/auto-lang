# Plan 088 Phase 4: jump_over 索引修复总结

**日期**: 2025-02-10
**状态**: ✅ 已修复
**解决方案**: 方案 A（全局跟踪 jump_over 占位符）

## 问题描述

当编译多个函数时，FN_PROLOG 指令的插入会导致后续函数的 `jump_over` 占位符索引失效，造成 JMP 指令跳转到错误的地址。

### 复现步骤

```auto
fn add(a int, b int) int {
    a + b
}

fn main() int {
    let result1 = add(10, 20)
    let result2 = add(30, 40)
    result1 + result2  // 应该输出 100
}
```

**预期输出**: 100 (30 + 70)
**实际结果（修复前）**: 崩溃（内存访问越界）

### 根本原因

1. 编译 add 函数时：
   - 发出 JMP jump_over_add (placeholder_idx = 1)
   - 编译函数体
   - 插入 FN_PROLOG（3 字节）到 entry_point = 3
   - 只更新了 reloc 和 exports，没有更新 jump_placeholders

2. 编译 main 函数时：
   - 发出 JMP jump_over_main (placeholder_idx = 19)
   - 插入 FN_PROLOG 到 entry_point = 16
   - 所有 >= 16 的代码后移 3 字节
   - jump_over_main 的索引应该从 19 更新到 22，但没有更新
   - patch_jump(19) 使用错误的索引，导致跳转地址错误

## 解决方案（方案 A）

### 实现

1. **添加 jump_placeholders 字段** (codegen.rs:115)
   ```rust
   pub jump_placeholders: Vec<usize>,
   ```

2. **记录所有 jump_over 索引** (codegen.rs:2431)
   ```rust
   fn emit_placeholder_i16(&mut self) -> usize {
       let idx = self.code.len();
       self.code.extend_from_slice(&0i16.to_le_bytes());
       self.jump_placeholders.push(idx);  // 记录索引
       idx
   }
   ```

3. **插入 FN_PROLOG 前更新索引** (codegen.rs:372-379)
   ```rust
   // 更新所有 > entry_point 的 jump_placeholder 索引
   for placeholder_idx in &mut self.jump_placeholders {
       if *placeholder_idx > entry_point as usize {
           *placeholder_idx += shift as usize;
       }
   }
   ```

### 关键点

- **使用 `>` 而不是 `>=`**：当前函数的 jump_over 在 entry_point 之前，不受影响
- **在 code.insert() 之前更新**：确保 patch_jump 使用正确的索引
- **只更新后续函数的索引**：当前函数的 jump_over 不需要更新

## 测试结果

### 单函数测试
```bash
$ cargo run --release -- run tmp/test_no_fn_prolog.at
输出: 42 ✅
```

### 多函数测试
```bash
$ cargo run --release -- run tmp/test_simple.at
输出: 100 ✅ (add(10,20)=30, add(30,40)=70, 30+70=100)
```

### 原始测试
```bash
$ cargo run --release -- run tmp/test_jump_over_bug.at
输出: 60 ✅
```

### Debug 输出验证

```
DEBUG patch_jump: placeholder_idx=1, target=13, anchor=3, offset=10
DEBUG patch_jump: placeholder_idx=14, target=58, anchor=16, offset=42
```

- **add 函数**: offset = 10 (正确，跳过函数体)
- **main 函数**: offset = 42 (正确，跳过函数体)

## 附加修复

### RESERVE_STACK 内存访问修复

在 engine.rs:2044 发现潜在的内存访问越界：
```rust
// 修复前：
task.ram.read_i32(task.bp - 1)  // 如果 BP=0，会访问地址 -1

// 修复后：
if task.bp >= 1 && task.bp + 1 < task.ram.raw.len() {
    task.ram.read_i32(task.bp - 1)
}
```

## 提交信息

```
Commit: 6979163
Author: puming.zhao <puming.zhao@soutek.cn>
Date:   2025-02-10

Fix Plan 088 Phase 4: jump_over index tracking for multi-function compilation

- 添加 jump_placeholders 字段跟踪所有 jump_over 占位符
- 在 FN_PROLOG 插入前更新后续函数的索引
- 修复 RESERVE_STACK 的潜在内存访问越界
- 测试验证：单函数和多函数场景都通过
```

## 相关文件

- `crates/auto-lang/src/vm/codegen.rs`: jump_over 索引跟踪
- `crates/auto-lang/src/vm/engine.rs`: RESERVE_STACK 内存访问修复
- `tmp/test_jump_over_bug.at`: 多函数测试用例
- `tmp/test_simple.at`: 简化多函数测试
- `tmp/test_no_fn_prolog.at`: 单函数测试

## 参考资料

- [Plan 088](088-param-passing-modes.md) - 参数传递模式
- [088-param-passing-progress.md](088-param-passing-progress.md) - 实现进度报告

## 下一步

- ✅ jump_over 索引问题已解决
- ✅ 多函数编译正常工作
- ✅ FN_PROLOG 指令框架完整实现
- ⏸️ 性能测试和优化（后续工作）
