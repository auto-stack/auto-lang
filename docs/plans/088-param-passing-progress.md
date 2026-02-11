# Plan 088 Phase 4 + Plan 087 Phase 3 实现进度报告

**日期**: 2025-02-10
**状态**: ✅ **90% 完成**

## 已完成的工作

### 1. ✅ 实例方法 Receiver 作为第一个函数参数

**问题**: 实例方法调用 `c.get()` 没有将 receiver 作为参数传递给函数。

**解决方案**:
- 修改 `Expr::Dot` 编译逻辑，将 receiver 作为参数 0
- 使用 `compile_call_arg` 实现智能参数传递
- 在 `codegen.rs` 中添加 `is_instance_method_call` 标志

**代码位置**: `codegen.rs:2054-2138`

### 2. ✅ 区分参数和局部变量的栈偏移

**问题**: 参数在 BP 之前，局部变量在 BP 之后，但 `LOAD_LOC_0` 假设所有局部变量都在 BP 之后。

**解决方案**:
- 在 `Codegen` 结构体中添加 `current_fn_n_args` 字段跟踪当前函数的参数数量
- 修改 `emit_load_loc` 来区分：
  - 参数 (index < n_args): 编码为 0x80 + index，从 BP 之前读取
  - 局部变量 (index >= n_args): 从 BP+1 之后读取
- 修改 `LOAD_LOCAL` VM 指令来处理参数编码 (0x80+)

**代码位置**:
- `codegen.rs:105-106` (添加 current_fn_n_args 字段)
- `codegen.rs:294-295` (设置 current_fn_n_args)
- `codegen.rs:2481-2527` (修改 emit_load_loc)
- `engine.rs:1895-1928` (修改 LOAD_LOCAL 指令)

**栈帧布局**:
```
调用前:        [arg0, arg1, ...]
CALL 之后:    [arg0, arg1, ..., return_addr, old_bp]
                                    ^- BP-1    ^- BP
参数位置:      arg0 在 BP-n_args, arg1 在 BP-n_args+1, ...
局部变量:      local0 在 BP+1, local1 在 BP+2, ...
```

### 3. ✅ 修复 GET_GENERIC_FIELD 和 SET_GENERIC_FIELD

**问题**: GET_GENERIC_FIELD 和 SET_GENERIC_FIELD 使用 `TypeTag::GenericInstance("")` 来比较类型标签，但实际实例有 `TypeTag::GenericInstance("Counter")`，导致类型检查失败。

**解决方案**:
- 使用 `matches!` 宏检查 GenericInstance 变体（任意 mono_name）
- 使用 `as_any().downcast_ref()` 进行类型转换
- 修改 GET_GENERIC_FIELD 从代码流读取 field_index（不是从栈）

**代码位置**:
- `engine.rs:901-938` (GET_GENERIC_FIELD)
- `engine.rs:951-986` (SET_GENERIC_FIELD)

### 4. ✅ 测试验证

**测试文件**:
- `tmp/test_method_simple.at`: ✅ 通过，输出 42
- `tmp/test_method_readonly.at`: ✅ 通过，输出 42
- `tmp/test_field_access.at`: ✅ 通过，输出 42

**测试结果**:
```bash
$ cargo run --release -- run tmp/test_method_readonly.at
42

$ cargo run --release -- run tmp/test_field_access.at
42
```

## 待完成的工作

### 1. ⏸️ 实现 `self.field = value` 赋值语句编译

**问题描述**: `Counter.increment()` 方法中的 `self.count = self.count + 1` 赋值语句没有被正确编译为 SET_GENERIC_FIELD 指令。

**当前行为**:
- `self.count = ...` 赋值被忽略或编译错误
- 导致 Counter.increment() 返回后，main 函数的 `c` 变量被破坏

**解决方案**:
- 检查 `Expr::Dot` 作为左值（赋值目标）的编译逻辑
- 修改 STORE 语句处理 `Expr::Dot` 的情况
- 确保生成 SET_GENERIC_FIELD 指令

**测试文件**: `tmp/test_simple_method.at`

### 2. ⏸️ 多参数方法支持

**当前限制**: VM 中的 LOAD_LOCAL 硬编码 `n_args = 1`，只支持单参数方法（实例方法）。

**解决方案**:
- 在函数入口存储 n_args（可能作为函数元数据）
- 修改 LOAD_LOCAL 从函数元数据读取 n_args

## 技术细节

### 参数编码方案

**编码方式**:
- 参数 index 编码为 `0x80 + index` (0x80 = param 0, 0x81 = param 1, ...)
- 局部变量 index 保持原样 (0, 1, 2, ...)

**为什么这样编码**:
- 简单：只需检查高位 (>= 0x80)
- 可扩展：支持最多 128 个参数
- 与现有指令兼容：不影响局部变量编码

### 栈偏移计算

**参数偏移**:
```rust
let n_args = 1;  // TODO: 从函数元数据读取
let offset = n_args - param_idx;  // param 0 -> offset=1
let actual_offset = offset + 1;  // +1 for return_addr
// 读取 BP - actual_offset
```

**示例**: 对于 `Counter.get(self)` (n_args=1)
- param 0 (self) 在 BP-2
  - BP - (1 - 0) - 1 = BP - 2 ✅

## 文件修改清单

### 修改的文件
1. `crates/auto-lang/src/vm/codegen.rs`
   - 添加 `current_fn_n_args` 字段
   - 修改 `emit_load_loc` 区分参数和局部变量
   - 修改 `Expr::Dot` 编译逻辑将 receiver 作为参数 0

2. `crates/auto-lang/src/vm/engine.rs`
   - 修改 `LOAD_LOCAL` 指令处理参数编码
   - 修改 `GET_GENERIC_FIELD` 使用 matches! 检查
   - 修改 `SET_GENERIC_FIELD` 使用 matches! 检查

### 测试文件
1. `tmp/test_method_simple.at` - ✅ 通过
2. `tmp/test_method_readonly.at` - ✅ 通过
3. `tmp/test_field_access.at` - ✅ 通过
4. `tmp/test_simple_method.at` - ⏸️ 待完成（需要赋值支持）

## 总结

**核心成就**:
1. ✅ 实例方法调用 `c.get()` 正常工作
2. ✅ 字段访问 `c.count` 正常工作
3. ✅ Receiver 作为参数正确传递
4. ✅ 参数和局部变量的栈偏移正确计算

**剩余工作**:
1. ⏸️ 实现 `self.field = value` 赋值语句编译
2. ⏸️ 支持多参数方法（修复硬编码 n_args=1）

**下一步**: 修改 STORE 语句编译逻辑来处理 `Expr::Dot` 作为左值的情况。
