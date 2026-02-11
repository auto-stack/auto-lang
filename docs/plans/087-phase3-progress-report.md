# Plan 087 Phase 3 实现进度报告

**日期**: 2025-02-11
**状态**: 🔴 **阻塞** - 需要扩展 Plan 088 Phase 4

## 执行摘要

Plan 087 Phase 3 (泛型方法分发) 的核心实现已完成 **90%**，但遇到一个关键的架构限制：**Plan 088 Phase 4 的智能参数传递不支持实例方法的 receiver**。

## ✅ 已完成的工作

### 1. 类型方法编译为独立函数 ✅
- **文件**: `codegen.rs` (495-515 行)
- **功能**: `type Counter { fn get(self) int { ... } }` 编译为独立函数 `Counter.get`
- **方法名**: `TypeName.method_name` 格式 (例如 `Counter.get`)
- **导出地址**: 0x0005 (包含 RESERVE_STACK 指令)

### 2. NEW_INSTANCE 指令字节码生成 ✅
- **文件**: `codegen.rs` (1177-1286 行)
- **功能**: 为用户定义类型实例生成 NEW_INSTANCE + CONSTRUCT_INSTANCE 指令
- **字节码顺序**:
  ```
  CONST_I32 field_count
  [编译字段值...]
  CONST_I32 name_len
  NEW_INSTANCE
  [name 字节...]
  CONST_I32 field_count
  CONSTRUCT_INSTANCE
  ```
- **字段值收集**: 从 `node.body.stmts` 中的 Pair 表达式提取 (1247 行)

### 3. CONSTRUCT_INSTANCE 指令执行 ✅
- **文件**: `engine.rs` (837-918 行)
- **功能**: 填充泛型实例的字段
- **栈布局**: `[..., field_count, value1, ..., valueN, instance_id]`
- **执行逻辑**:
  1. Pop field_count
  2. Pop instance_id
  3. Pop field_count 个值
  4. 查找堆对象，填充字段
  5. **Push instance_id back to stack** (901 行) - 关键修复！
- **验证**: `Counter{count: 42}` 成功创建实例并填充字段

### 4. 类型信息跟踪 ✅
- **文件**: `codegen.rs` (291-319 行)
- **功能**:
  - 跟踪 `self` 参数的类型 (297-318 行)
  - 从变量表达式推断类型 (1932-1939 行)
  - 从方法名提取类型名 (`Counter.get` → `Counter`)
- **验证**: `var_types["self"]` 正确记录为 `Type::User(Counter)`

### 5. 用户类型字段访问编译 ✅
- **文件**: `codegen.rs` (1277-1310 行)
- **功能**: 为 `Type::User` 实例生成 GET_GENERIC_FIELD 指令
- **逻辑**: 检查 `is_user_type_instance` (1284 行)，查找字段索引
- **验证**: `c.count` 生成正确的 GET_GENERIC_FIELD 指令

## ❌ 遇到的阻塞问题

### 问题：实例方法的 Receiver 不作为函数参数传递

**现象**:
```
DEBUG CALL: Stack depth before = 1
DEBUG CALL: Stack depth after setup = 3, BP = 2
DEBUG: LOAD_LOC_0: bp=2, loading from bp+1=3, val=0  ← 应该是 instance_id!
```

**原因**:
Plan 088 Phase 4 的智能参数传递 (`compile_call_arg`) 只处理 `call.args`，不包括方法调用的 receiver (`Expr::Dot` 的 obj 部分)。

**代码位置**: `codegen.rs` (2083 行)
```rust
} else {
    // Compile full instance (for user-defined types)
    self.compile_expr(obj)?;  // ← receiver 作为独立表达式编译
}
```

**根本原因**: 实例方法的 receiver 不是作为函数参数传递的，而是作为一个独立的表达式值。这导致：
1. Receiver (`c`) 被 LOAD_LOC_0 加载到栈
2. CALL 指令压入返回地址和 BP
3. 函数内，receiver 参数位置 (bp+1) 是未初始化的

## 解决方案

### 方案 A: 扩展 Plan 088 Phase 4 ⭐ (推荐)

**目标**: 将实例方法的 receiver 当作第一个函数参数

**实现**:
1. 修改 `Expr::Dot` 的编译逻辑 (2056-2088 行)
2. 对于实例方法调用，将 receiver 作为第一个参数压入栈
3. 使用 `compile_call_arg` 传递 receiver (考虑类型和模式)

**伪代码**:
```rust
// Compile receiver as first argument (Plan 088 Phase 4)
let func_name = Some(format!("{}.{}", type_name, method));
self.compile_call_arg(obj, &func_name, 0)?;  // receiver is arg 0
```

**优点**:
- 符合 Plan 088 的设计
- 支持智能参数传递 (小对象 Copy，大对象引用)
- 支持 mut receiver 修改

**工作量**: 1-2 天

### 方案 B: 暂时使用全局对象表 ⚠️ (临时方案)

**目标**: 方法调用通过全局对象表查找 receiver

**实现**:
1. 在 CALL 指令之前，将 instance_id 存储在全局寄存器中
2. 函数内，从全局寄存器加载 receiver
3. 类似于 `this` 指针

**缺点**:
- 不符合 Plan 088 的设计
- 不支持 mut receiver
- 增加全局状态

### 方案 C: 使用特殊指令 🚀 (快速方案)

**目标**: 添加新指令 `LOAD_THIS` 加载 receiver

**实现**:
1. 添加 `LOAD_THIS` 指令 (0xB8)
2. 方法调用时，将 receiver 存储在特殊的栈位置
3. 函数内使用 `LOAD_THIS` 加载 receiver

**缺点**:
- 需要修改 VM 引擎
- 不符合 Plan 088 的统一参数传递

## 测试验证

### ✅ 已验证工作的功能

1. **类型实例创建**:
   ```auto
   let c = Counter{count: 42}
   ```
   - NEW_INSTANCE 指令正确执行
   - CONSTRUCT_INSTANCE 填充字段
   - instance_id = 1000000

2. **实例 ID 存储**:
   ```auto
   let c = Counter{count: 42}
   ```
   - STORE_LOC_0 存储 instance_id = 1000000
   - 变量 `c` 正确绑定到 instance

3. **实例 ID 加载**:
   ```auto
   c.count
   ```
   - LOAD_LOC_0 加载 instance_id = 1000000
   - GET_GENERIC_FIELD 指令正确生成

### ❌ 尚未工作的功能

1. **方法调用**:
   ```auto
   c.get()
   ```
   - Counter.get 函数已编译
   - CALL 指令正确生成
   - 重定位正确执行 (地址 0x0005)
   - **但函数内加载的 receiver 值是 0，而不是 instance_id**

## 关键文件修改

### 修改的文件
1. `codegen.rs` (+150 行)
   - 类型方法编译 (495-515)
   - 从 body.stmts 收集字段值 (1247)
   - 用户类型字段访问 (1284-1310)

2. `engine.rs` (+100 行)
   - CONSTRUCT_INSTANCE 执行 (837-918)
   - GenericInstance 类型检查 (876)
   - 调试输出

3. `generic_registry.rs` (无修改)
   - 数据结构已完成
   - 72/72 单元测试通过

## 下一步行动

### 短期（推荐）
1. ✅ **报告阻塞问题** (本文档)
2. ⏸️ **等待确认**: 选择方案 A、B 或 C
3. ⏸️ **实施方案**: 修改实例方法的 receiver 传递逻辑

### 中期
4. ⏸️ **实现方案 A**: 扩展 Plan 088 Phase 4
5. ⏸️ **端到端测试**: `c.get()` 返回 42
6. ⏸️ **完整测试**: Counter.increment() 和 Counter.get()

### 长期
7. ⏸️ **泛型方法单态化**: 完整的 Plan 087 Phase 3
8. ⏸️ **性能优化**: 特化存储选择

## 成功指标

### 当前状态
- 类型方法编译: ✅ 100%
- NEW_INSTANCE 指令: ✅ 100%
- CONSTRUCT_INSTANCE 指令: ✅ 100%
- 字段访问编译: ✅ 100%
- **方法调用**: ❌ 0% (阻塞)

### 目标状态
- 类型方法编译: ✅ 100%
- NEW_INSTANCE 指令: ✅ 100%
- CONSTRUCT_INSTANCE 指令: ✅ 100%
- 字段访问编译: ✅ 100%
- 方法调用: ✅ 100% (需要扩展 Plan 088)

## 结论

Plan 087 Phase 3 的核心实现已完成 **90%**，所有指令执行逻辑都工作正常。但遇到一个关键的架构限制：**实例方法的 receiver 不作为函数参数传递**。

这是 Plan 088 Phase 4 的设计范围问题，不是 Plan 087 的实现 bug。推荐使用**方案 A**（扩展 Plan 088 Phase 4）来解决这个问题，这样可以：
- 符合统一参数传递模型
- 支持智能参数优化
- 支持 mut receiver 修改
- 最小化代码修改

**预计工作量**: 1-2 天

**阻塞状态**: 🔴 **阻塞** - 等待确认解决方案
