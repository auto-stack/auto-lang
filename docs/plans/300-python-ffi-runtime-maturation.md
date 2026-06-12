# Plan 300: AutoVM Python FFI Runtime Maturation

## Context

Auto 的 Python FFI 管道（Plan 214/222）已实现完整链路：`use.py module::{items}` → parser → use_scanner → CompileSession → codegen → PyFfiBridge → pyo3 CPython。但存在关键限制：

1. **所有函数强制 string→string 签名**：`init_py_ffi()` (lib.rs:486) 硬编码 `PySignature::default_string_string()`，导致 `math.sqrt(2.0)` 等非字符串调用失败
2. **返回值缺少 dict→Obj 转换**：`py_auto_marshal_return` 未处理 Python dict，`json.loads(...)` 返回的 dict 变成字符串 repr
3. **`use.py module`（无 items）被跳过**：`collect_py_imports` (compile.rs:910) 中 `items.is_empty()` 时直接 continue

**目标**：让 Auto 能导入并调用任意 Python 库，参数/返回值自动 marshalling，与 Rust FFI 同等体验。

## Batch 1: Auto 参数 Marshalling（核心改进）

### Task 1.1: 添加 `PySignature::all_auto()` 构造器

**文件**: `crates/auto-lang/src/py_ffi_types.rs`

添加便捷构造器，用于创建全 Auto 签名（运行时类型检测）：

```rust
/// All-auto signature: runtime NanoValue tag detection for all params and return.
/// `param_count` 个参数，每个参数类型在运行时从 VM 栈的 NanoValue tag 自动推断。
pub fn all_auto(param_count: usize) -> Self {
    Self {
        params: vec![PyType::Auto; param_count],
        returns: PyType::Auto,
    }
}
```

当 `params` 包含 `PyType::Auto` 时，shim 使用 `pop_arith_operand` 模式检测实际类型。

### Task 1.2: 添加 `pop_auto_py_arg()` 自动参数转换

**文件**: `crates/auto-lang/src/py_ffi.rs`

添加自由函数，从 VM 栈弹出一个值并转换为 Python 对象：

```rust
fn pop_auto_py_arg<'py>(
    task: &mut AutoTask,
    vm: &AutoVM,
    py: Python<'py>,
) -> Result<Bound<'py, PyAny>, VMError> {
    // 复用 pop_arith_operand() 模式：
    // 1. peek TOS，如果是 null + 下面是非 NaN-boxed → 2-slot f64 → pop_f64 → PyFloat
    // 2. 否则 pop_nv()，检查 tag_of：
    //    TAG_I32(1) → decode_i32 → Python int
    //    TAG_STRING(2) → decode_string → 查 string pool → Python str
    //    TAG_BOOL(3) → decode_bool → Python bool
    //    TAG_NULL(4) → Python None
    //    TAG_OBJECT(5) → decode_object → 查 heap → 转为 Python dict（Phase 2）
    //    TAG_LIST(6) → decode_list → 查 heap → 转为 Python list（Phase 2）
    //    TAG_F32(7) → decode_f32 → Python float
}
```

关键：复用 `VirtualRAM::pop_arith_operand()` 的 f64 检测模式（null padding marker + raw bits）。

### Task 1.3: 修改 shim 对 `PyType::Auto` 参数的处理

**文件**: `crates/auto-lang/src/py_ffi.rs` — `register_function` 中的 shim closure

修改参数弹出循环（当前 line 75-103）：

```rust
for pt in param_types.iter().rev() {
    let py_val = match pt {
        PyType::Auto => pop_auto_py_arg(task, vm, py)?,  // 新增
        PyType::Int => { /* 现有代码 */ }
        PyType::Float => { /* 现有代码 */ }
        // ... 其他固定类型不变
    };
    bound_args.push(py_val);
}
```

### Task 1.4: 修改 `init_py_ffi` 使用 auto 签名

**文件**: `crates/auto-lang/src/lib.rs` (line 486)

将 `PySignature::default_string_string()` 替换为 `PySignature::all_auto(param_count)`。

**param_count 获取方式**：用 Python `inspect.signature()` 反射获取必填位置参数数量，失败时回退到 1：

```rust
let param_count = Python::with_gil(|py| {
    let inspect = py.import("inspect").ok()?;
    let sig = inspect.call_method1("signature", (func_obj,)).ok()?;
    let params: &Bound<'_, PyDict> = sig.getattr("parameters").ok()?.downcast().ok()?;
    // 计算无默认值的位置参数数量
    Some(params.len())
}).unwrap_or(1);
let sig = PySignature::all_auto(param_count);
```

### Task 1.5: 添加单元测试

**文件**: `crates/auto-lang/src/py_ffi.rs` (tests module)

测试 `pop_auto_py_arg` 的各种 NanoValue tag 场景（需要 `--features python`）：
- i32 → Python int
- f64 → Python float
- string → Python str
- bool → Python bool
- null → Python None

---

## Batch 2: 增强 Marshalling + 模块导入

### Task 2.1: 添加 Python dict → VM Obj 转换

**文件**: `crates/auto-lang/src/py_ffi.rs`

1. 在 `py_auto_marshal_return` 中添加 `PyDict` 检测（在 PyList 之前）：
```rust
else if let Ok(dict) = py_val.downcast::<PyDict>() {
    py_dict_to_vm_heap(dict, task, vm)?;
}
```

2. 新函数 `py_dict_to_vm_heap()`：
   - 遍历 dict items
   - 递归 marshal 每个 value（调用 `py_any_to_value`）
   - 构建 `AutoVMHashMap` (from `crate::vm::collections`)
   - 插入 VM heap，push object ID

3. 新函数 `py_any_to_value()`：递归 Python→Value 转换（处理嵌套 list/dict）

4. 更新 `py_list_to_vm_heap` 使用 `py_any_to_value` 替代当前的手动枚举

### Task 2.2: 支持无 items 的模块导入

**文件**: `crates/auto-lang/src/compile.rs` (line 910)

修改 `collect_py_imports`：移除 `items.is_empty()` 的 continue 检查，改为允许空 items：

```rust
// Before:
if !use_stmt.is_python_import || use_stmt.items.is_empty() { continue; }
// After:
if !use_stmt.is_python_import { continue; }
// Always record, even with empty items (bare module import)
```

**文件**: `crates/auto-lang/src/vm/codegen.rs`

1. 添加 `py_modules: HashSet<String>` 字段
2. 修改 `handle_py_import`：当 items 为空时，记录模块名到 `py_modules`
3. 在 Call 表达式解析（~line 5858）中，当 `Expr::Dot(Expr::Ident(module), method)` 且 module 在 `py_modules` 中时，动态注册 `"module.method"` 到 `py_native_map`

**文件**: `crates/auto-lang/src/lib.rs` (`init_py_ffi`)

对空 items 的模块：用 `dir()` + `callable()` 发现所有公共可调用函数，批量注册为 `all_auto(1)`（保守默认）。

---

## Batch 3: 集成测试

### Task 3.1: 测试 `random` 模块（int 参数/返回值）

```auto
use.py random::{randint}
fn main() {
    let n = randint(1, 100)
    print(f"random: $n")
}
```

### Task 3.2: 测试 `math` 模块（float 参数/返回值）

```auto
use.py math::{sqrt}
fn main() {
    let r = sqrt(2.0)
    print(f"sqrt(2) = $r")
}
```

### Task 3.3: 测试 `json` 模块（dict 返回值）

```auto
use.py json::{loads, dumps}
fn main() {
    let data = loads("{\"name\": \"Alice\"}")
    print(f"name: $data.name")
}
```

### Task 3.4: 测试自定义 Python 模块

创建 `tmp/py_add.py`:
```python
def add(a, b):
    return a + b
```

```auto
use.py sys::{path}
use.py py_add::{add}
fn main() {
    let result = add(3, 5)
    print(f"3 + 5 = $result")
}
```

### Task 3.5: 测试无 items 模块导入

```auto
use.py math
fn main() {
    let r = math.sqrt(2.0)
    print(f"sqrt(2) = $r")
}
```

---

## 关键文件

| 文件 | 修改内容 |
|------|---------|
| [py_ffi.rs](crates/auto-lang/src/py_ffi.rs) | `pop_auto_py_arg`、shim Auto 分支、dict→Obj、`py_any_to_value` |
| [py_ffi_types.rs](crates/auto-lang/src/py_ffi_types.rs) | `PySignature::all_auto()` 构造器 |
| [lib.rs](crates/auto-lang/src/lib.rs) | `init_py_ffi` 用 `all_auto` + inspect 反射 |
| [codegen.rs](crates/auto-lang/src/vm/codegen.rs) | `py_modules` 字段、Dot 调用动态注册 |
| [compile.rs](crates/auto-lang/src/compile.rs) | `collect_py_imports` 允许空 items |

## 关键参考

| 代码位置 | 用途 |
|---------|------|
| `nano_value.rs` tag_of/is_i32/is_null/decode_* | NanoValue tag 检测和解码 |
| `virt_memory.rs:421` `pop_arith_operand()` | f64 2-slot 检测模式（复用到 pop_auto_py_arg） |
| `vm/collections.rs` `AutoVMHashMap` | dict→VM heap 的容器类型 |
| `vm/engine.rs:4714` CALL_NAT dispatch | VM 调用 native shim 的入口 |

## 验证步骤

1. `cargo build -p auto-lang --features python` — 编译通过
2. `cargo test -p auto-lang --features python -- py_ffi` — PyFfi 单元测试通过
3. `cargo test -p auto-lang` — 无 python feature 时不回归
4. 手动测试 `auto tmp/test_random.at` — randint 返回整数
5. 手动测试 `auto tmp/test_math.at` — sqrt 返回浮点数
6. 手动测试 `auto tmp/test_json.at` — loads 返回可字段访问的对象

---

## Batch 4: REPL Python FFI 支持（Phase 2）

### Context

Batch 1-3 实现了脚本执行路径的完整 Python FFI。但 REPL 路径 (`autovm_persistent.rs`) 没有 Python FFI 集成：`resolve_use_statements()` 只处理 Auto 模块导入，跳过 `is_python_import`。REPL 中 `use.py math: sqrt` 后调用 `sqrt(2)` 报 `MissingNative` 错误。

### Task 4.1: 添加 PyFfiBridge 持久化字段

**文件**: `crates/auto-lang/src/autovm_persistent.rs`

在 `AutovmReplSession` 结构体中添加 `#[cfg(feature = "python")]` 的 `py_bridge: Option<PyFfiBridge>` 字段。在 `new()` 中初始化为 `None`，在 `reset()` 中重置为 `None`。懒初始化——只在第一次 `use.py` 时创建。

### Task 4.2: 添加 `resolve_py_imports()` 方法

**文件**: `crates/auto-lang/src/autovm_persistent.rs`

从 `init_py_ffi()` (lib.rs:468-530) 提取核心注册逻辑：
- 懒创建 PyFfiBridge
- 处理 bare module（discover callables）和 named imports（inspect param count）
- 注册到 BIGVM_NATIVES
- 调用 `vm.merge_native_interface()` 注入 shim 到 VM

### Task 4.3: 集成到 `run()` 流程

**文件**: `crates/auto-lang/src/autovm_persistent.rs`

- `resolve_use_statements()` 中跳过 `is_python_import`
- `run()` 中在 `resolve_use_statements` 后调用 `resolve_py_imports`

### 关键设计

- **懒初始化**：无 `use.py` 时零开销
- **持久化**：bridge 跨 REPL 输入持久，模块只导入一次
- **cfg-gated**：无 python feature 时编译为 no-op

### 验证

1. `cargo build -p auto --features python --no-default-features` — 编译通过
2. REPL 测试：
   ```
   AutoVM> use.py math: sqrt
   AutoVM> sqrt(2)
   1.4142135623730951
   ```
3. `:reset` 后重新 `use.py` 仍然工作
