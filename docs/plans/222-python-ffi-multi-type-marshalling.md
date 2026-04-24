# Plan 222: Python FFI Multi-Type Marshalling

## Context

Plan 214 实现了 Python FFI（`use.py`），但只支持 string→string 单参数调用。AutoVM 已通过 Plan 221 引入 NaN-boxing 双模式栈，Rust FFI 也已支持多种类型 marshalling（`RustType` + `RustSignature`）。本计划将 Python FFI 扩展为支持 int/float/bool/string/list（基础类型列表）的多类型 marshalling。

## 关键文件

| 文件 | 角色 |
|------|------|
| `crates/auto-lang/src/py_ffi.rs` (151行) | 当前 string-only Python FFI |
| `crates/auto-lang/src/py_ffi_types.rs` | **新建** — PyType/PySignature 枚举（无 pyo3 依赖） |
| `crates/auto-lang/src/ffi.rs` | Rust FFI 参考实现（RustType, RustSignature, 多类型 shim） |
| `crates/auto-lang/src/lib.rs` | `init_py_ffi()` 调用点 (~line 414) |
| `crates/auto-lang/src/vm/codegen.rs` | `handle_py_import()` (~line 3062), 返回类型跟踪 (~line 5421) |
| `crates/auto-lang/src/vm/virt_memory.rs` | 栈 API 参考（push_i32/pop_f64/push_str_idx 等） |

## 当前状态

py_ffi.rs shim 流程：`pop_str_idx()` → `vm.strings` 查找 → PyString → `func.call1()` → `extract::<String>()` → 写入 `vm.strings` → `push_str_idx()`。全程 string。

## Task 1: 创建 py_ffi_types.rs（PyType + PySignature）

**新建** `crates/auto-lang/src/py_ffi_types.rs`

提取类型定义为独立模块，不含 pyo3 依赖，确保 codegen.rs 在无 `python` feature 时也能编译。

```rust
pub enum PyType { None, Bool, Int, Float, String, List, Auto }
pub struct PySignature { pub params: Vec<PyType>, pub returns: PyType }
// Builder: new(), .param(), .returns(), .default_string_string()
```

`Auto` = 运行时自动检测返回类型（Python 动态类型），保持向后兼容。

在 `lib.rs` 添加 `pub mod py_ffi_types;`（无条件编译）。

## Task 2: 重构 register_function 和 shim 生成器

**修改** `py_ffi.rs`

1. `register_function` 签名增加 `signature: PySignature` 参数
2. 参数 marshalling：`signature.params.iter().rev()` 逐个 pop，按类型转换：

| PyType | VM pop | PyO3 构造 |
|--------|--------|-----------|
| Int | `pop_i32()` | `PyLong::new(py, val)` |
| Float | `pop_f64()` | `PyFloat::new(py, val)` |
| Bool | `pop_i32()` | `PyBool::new(py, val != 0)` |
| String | `pop_str_idx()` + `vm.strings` | `PyString::new(py, &s)` |
| None | (无) | `py.None()` |

3. 返回 marshalling：如果 `returns == Auto`，运行时检测 Python 返回值类型；否则按声明类型提取。bool 检查必须在 int 之前（Python bool 是 int 子类）。

| PyType | PyO3 提取 | VM push |
|--------|----------|---------|
| Int | `extract::<i32>()` | `push_i32(val)` |
| Float | `extract::<f64>()` | `push_f64(val)` |
| Bool | `extract::<bool>()` | `push_i32(if val { 1 } else { 0 })` |
| String | `extract::<String>()` | `vm.strings` 写入 + `push_str_idx()` |
| None | `is_none()` | `push_i32(0)` |
| List | 遍历 PyList，元素递归 marshal | 创建 heap List + `push_i32(instance_id)` |

## Task 3: List marshalling 辅助函数

**修改** `py_ffi.rs`

- `py_to_vm_value(py, py_any, task, vm) -> Result<(), VMError>` — 单个 Python 值 → VM push
- List 支持：遍历 `PyList`，每个元素按 int/float/str/bool 转为 `auto_val::Value`，创建 `GenericInstanceData("List", values)`，通过 `vm.insert_heap_object()` 存入堆

## Task 4: 更新 lib.rs 调用点

**修改** `crates/auto-lang/src/lib.rs`

`init_py_ffi()` 中 `bridge.register_function(module_name, func_name)` 改为传入 `PySignature::default_string_string()`，保持向后兼容。

## Task 5: 更新 codegen handle_py_import

**修改** `crates/auto-lang/src/vm/codegen.rs`

1. 新增字段 `py_return_types: HashMap<String, PyType>`（PyType 来自 py_ffi_types，无条件编译）
2. `handle_py_import()` 记录 `PyType::Auto` 作为默认返回类型
3. 调用类型跟踪处（~line 5421）：根据 `py_return_types` 设置 `last_expr_type`（Auto/String → ObjectType::String, Int → Int, Float → Double, Bool → Bool）

## Task 6: 测试

**修改** `py_ffi.rs` 测试模块

- 更新现有测试传入 `PySignature`
- 新增 `test_py_signature_int_float`、`test_py_signature_auto_return`
- E2E 测试（`#[ignore]`）：调用 Python `math.sqrt(float) float`，`len(string) int`，`json.dumps(obj) str`

## Task 7: Feature gate 一致性审计

确保：
- `py_ffi_types.rs` 无 feature gate（纯数据类型）
- `py_ffi.rs` 保持 `#[cfg(feature = "python")]`
- `codegen.rs` 中 `py_return_types` 字段无需 gate（依赖 py_ffi_types 而非 py_ffi）
- `cargo build -p auto-lang` 和 `cargo build -p auto-lang --features python` 均通过

## 实施顺序

```
Task 1 (py_ffi_types.rs) → Task 2 (register_function) → Task 3 (list helpers)
  ↓                                                        ↓
Task 4 (lib.rs)                                        Task 6 (tests)
  ↓
Task 5 (codegen)
  ↓
Task 7 (feature gate audit)
```

## 延迟到后续

- Dict marshalling（PyDict → VM Map，键类型复杂）
- 嵌套 List（List<List<int>> → 嵌套 heap objects）
- Parser 级别类型注解语法（`use.py math::sqrt(float) float`）
- Python kwargs 支持
- Python 异常 → Auto Result.Err 映射

## 验证

```bash
# 1. 无 feature 编译
cargo build -p auto-lang

# 2. 含 feature 编译
cargo build -p auto-lang --features python

# 3. 单元测试
cargo test -p auto-lang --features python -- py_ffi

# 4. E2E 测试（需 Python 环境）
cargo test -p auto-lang --features python -- test_21_python_ffi --ignored --nocapture

# 5. 全量回归
cargo test -p auto-lang --lib
```
