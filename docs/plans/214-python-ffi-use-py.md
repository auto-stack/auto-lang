# Plan 214: AutoVM Python FFI — `use.py` 嵌入 Python 解释器

> **Status: 📋 READY** (Plan 212 Rust FFI E2E ✅ complete, dependency resolved)
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** AutoVM 通过 PyO3 嵌入 CPython 解释器，支持 `use.py json5::{dumps, loads}` 直接调用 Python 库函数。MVP 阶段只支持 string→string 签名，验证完整管线。

**Architecture:** 镜像 Plan 212 的 Rust FFI 管线，但替换 DLL 加载为 PyO3 运行时导入。无 wrapper crate 生成 — Python 模块直接在解释器中 import。

**Tech Stack:** PyO3, `PyFfiBridge`, pip subprocess, `#[cfg(feature = "python")]` feature gate

**Scope:** String-only MVP — 所有参数和返回值以 string 形式传递，与 Plan 212b Rust FFI 一致。

---

## 与 Plan 212 的对比

| 组件 | Rust FFI (Plan 212) | Python FFI (Plan 214) |
|------|---------------------|----------------------|
| Bridge | `RustFfiBridge` (libloading) | `PyFfiBridge` (pyo3) |
| Wrapper | 生成 cdylib + cargo build | 无（直接 import） |
| Dep 解析 | `sandbox.compile_dep()` → cargo build | `pip install {pkg}` |
| Shim 签名 | `Fn(&mut AutoTask, &AutoVM)` | 相同接口 |
| Marshaling | C strings `*const c_char` | PyO3 `.extract::<String>()` |
| Native 注册 | `BIGVM_NATIVES` `"rust.{fn}"` | `BIGVM_NATIVES` `"py.{fn}"` |
| Codegen map | `rust_native_map` | `py_native_map` |
| Return types | `fn_return_types` | `fn_return_types` |
| Feature gate | 无 | `#[cfg(feature = "python")]` |
| Native ID range | 300+ (RustFfiBridge) | 400+ (PyFfiBridge) |

---

## 端到端流程

```
用户代码:
  dep json5
  use.py json5::{dumps, loads}
  let json = dumps(data)
  let obj = loads(json)
  print(obj)

执行流程:
  1. Parser 解析 dep/use.py → DepStmt + Use(Py)
  2. collect_py_imports() → py_imports: {"json5": ["dumps", "loads"]}
  3. resolve_deps() → pip install json5 (若未缓存)
  4. resolve_uses() → register in TypeStore, collect imports
  5. init_py_ffi() → PyO3 嵌入解释器, import json5, 注册 shims
  6. Codegen handle_py_import() → py_native_map → fn_return_types
  7. 遇到 dumps(data) → 查 py_native_map → CALL_NAT(py_id)
  8. VM CALL_NAT → shim: pop string → PyO3 call → push string result
```

---

## Task 1: 添加 PyO3 依赖和 Feature Gate

**文件：**
- 修改：`crates/auto-lang/Cargo.toml`
- 创建：`crates/auto-lang/src/py_ffi.rs`（feature-gated 模块）

**目标：** 添加 `pyo3` 作为可选依赖，创建 `PyFfiBridge` 结构体骨架。

**Step 1: 修改 Cargo.toml**

```toml
[features]
default = []
python = ["pyo3"]

[dependencies]
pyo3 = { version = "0.23", optional = true, features = ["auto-initialize"] }
```

`auto-initialize` feature 让 PyO3 在首次使用时自动调用 `Py_Initialize()`，无需手动初始化。

**Step 2: 在 lib.rs 中注册模块**

```rust
#[cfg(feature = "python")]
pub mod py_ffi;
```

**Step 3: 创建 PyFfiBridge 骨架**

在 `crates/auto-lang/src/py_ffi.rs` 中：

```rust
//! Plan 214: Python FFI Bridge — embed CPython via PyO3
//!
//! Mirrors RustFfiBridge pattern: register Python functions as native shims,
//! marshal arguments/returns as strings through the PyO3 GIL boundary.

use crate::vm::error::VMError;
use crate::vm::native::NativeInterface;
use std::collections::HashMap;
use std::sync::Arc;

/// Bridge between AutoVM and Python via PyO3.
///
/// Each imported Python function becomes a native shim that:
/// 1. Acquires the GIL
/// 2. Pops string argument from AutoVM stack
/// 3. Calls the Python function via PyO3
/// 4. Extracts string result
/// 5. Pushes tagged string index back to AutoVM stack
pub struct PyFfiBridge {
    /// Imported Python modules: module_name → pyo3::Py<PyModule>
    modules: HashMap<String, pyo3::Py<pyo3::types::PyModule>>,

    /// Registered functions: "module.function" → native_id
    functions: HashMap<String, u16>,

    /// Next native ID (starts at 400 to avoid collision with RustFfiBridge's 300+)
    next_native_id: u16,

    /// Native interface for registering shims
    native_interface: NativeInterface,
}

impl PyFfiBridge {
    pub fn new() -> Result<Self, VMError> {
        // Verify Python interpreter is available
        pyo3::Python::with_gil(|py| {
            pyo3::PyErr::fetch(py); // Clear any pending errors
        });

        Ok(Self {
            modules: HashMap::new(),
            functions: HashMap::new(),
            next_native_id: 400,
            native_interface: NativeInterface::new(),
        })
    }

    /// Import a Python module by name.
    pub fn import_module(&mut self, module_name: &str) -> Result<(), VMError> { ... }

    /// Register a Python function as a native shim.
    pub fn register_function(
        &mut self,
        module_name: &str,
        function_name: &str,
    ) -> Result<u16, VMError> { ... }

    /// Create a native shim closure for a Python function.
    fn create_py_shim(
        &self,
        module_name: String,
        function_name: String,
    ) -> impl Fn(&mut crate::vm::task::AutoTask, &crate::vm::AutoVM) -> Result<(), VMError> + Send + Sync + 'static { ... }

    pub fn native_interface(&self) -> Arc<NativeInterface> { ... }
}
```

**Step 4: 构建验证**

```bash
cargo build -p auto-lang --features python
cargo build -p auto-lang  # 验证不含 feature 时仍能编译
```

**Step 5: Commit**

```bash
git commit -m "feat(py-ffi): add pyo3 dependency and PyFfiBridge scaffold (Plan 214 Task 1)"
```

---

## Task 2: `dep` → pip install 管线

**文件：**
- 修改：`crates/auto-cache/src/sandbox.rs`
- 修改：`crates/auto-lang/src/compile.rs`

**目标：** `dep json5` 触发 `pip install json5`（若未缓存），缓存到 `~/.auto/sandbox/py-packages/`。

**Step 1: 添加 pip install 到 Sandbox**

在 `Sandbox` 中添加方法：

```rust
/// Install a Python package via pip.
///
/// Checks cache first: ~/.auto/sandbox/py-packages/{pkg}/installed
/// If not cached, runs: pip install {pkg}=={version} --target ~/.auto/sandbox/py-packages/{pkg}/lib
pub fn install_py_package(
    &self,
    package_name: &str,
    version: Option<&str>,
) -> Result<PathBuf> {
    let pkg_dir = self.root.join("py-packages").join(package_name);
    let marker = pkg_dir.join("installed");

    // Check cache
    if marker.exists() {
        log::info!("Using cached Python package: {}", package_name);
        return Ok(pkg_dir.join("lib"));
    }

    // Install via pip
    let target = pkg_dir.join("lib");
    std::fs::create_dir_all(&target)?;

    let version_spec = match version {
        Some(v) => format!("{}=={}", package_name, v),
        None => package_name.to_string(),
    };

    let output = std::process::Command::new("pip")
        .args(["install", &version_spec, "--target", &target.to_string_lossy()])
        .output()
        .map_err(|e| SandboxError::CompilationFailed(format!("pip: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SandboxError::CompilationFailed(format!(
            "pip install {} failed:\n{}", package_name, stderr
        )));
    }

    // Write marker file
    std::fs::write(&marker, version.unwrap_or("*"))?;

    Ok(target)
}
```

**Step 2: 添加 collect_py_imports() 到 CompileSession**

在 `compile.rs` 中：

```rust
/// Plan 214: Python imports collected from use.py statements
py_imports: HashMap<String, Vec<String>>,
```

```rust
/// Collect use.py imports from source
pub fn collect_py_imports(&mut self, source: &str) {
    let use_stmts = scan_use_statements(source);
    for stmt in use_stmts {
        if stmt.is_python_import {
            let module_name = stmt.module.split("::").next().unwrap_or("").to_string();
            self.py_imports
                .entry(module_name)
                .or_default()
                .extend(stmt.items.iter().cloned());
        }
    }
}
```

**Step 3: 修改 resolve_deps() 触发 pip install**

在 `resolve_deps()` 中，对于 Python deps（在 `py_imports` 中）：

```rust
// After registering all deps, install Python packages
for (pkg_name, _functions) in &self.py_imports {
    if self.declared_crates.contains(pkg_name) {
        if let Some(ref sandbox) = self.sandbox {
            let version = self.dep_versions.get(pkg_name).map(|s| s.as_str());
            match sandbox.install_py_package(pkg_name, version) {
                Ok(lib_path) => log::info!("Installed {} -> {}", pkg_name, lib_path.display()),
                Err(e) => log::error!("Failed to install {}: {}", pkg_name, e),
            }
        }
    }
}
```

**Step 4: 添加 AST 支持 — UseKind::Py**

在 `ast/use_.rs` 中添加：

```rust
pub enum UseKind {
    Auto,
    C,
    Rust,
    Py,  // ← Plan 214
}
```

在 parser/lexer 中添加 `use.py` 识别（类似 `use.rust` 的模式）。在 `scan_use_statements` 中设置 `is_python_import = true`。

**Step 5: Commit**

```bash
git commit -m "feat(py-ffi): add pip install pipeline and use.py AST (Plan 214 Task 2)"
```

---

## Task 3: VM Codegen 处理 UseKind::Py

**文件：**
- 修改：`crates/auto-lang/src/vm/codegen.rs`

**目标：** `handle_use_stmt()` 处理 `UseKind::Py`，为导入的 Python 函数注册 native shim。

**Step 1: 添加 py_native_map 字段**

在 `Codegen` struct 中：

```rust
/// Plan 214: Python FFI function name → (module_name, full_path)
py_native_map: HashMap<String, (String, String)>,
```

**Step 2: 添加 handle_py_import()**

```rust
/// Plan 214 Task 3: Handle use.py statement
fn handle_py_import(&mut self, use_stmt: &crate::ast::Use) {
    let module_path = if let Some(ref mp) = use_stmt.module_path {
        mp.display()
    } else if !use_stmt.paths.is_empty() {
        use_stmt.paths.join("::")
    } else {
        return;
    };

    let module_name = module_path.split("::").next().unwrap_or(&module_path).to_string();

    if !use_stmt.items.is_empty() {
        for item in &use_stmt.items {
            let local_name = item.as_str();
            let full_path = format!("{}::{}", module_path, local_name);
            self.py_native_map.insert(
                local_name.to_string(),
                (module_name.clone(), full_path),
            );
            // Python FFI functions return String
            self.fn_return_types.insert(local_name.to_string(), Type::Str(0));
        }
    }
}
```

**Step 3: 修改 handle_use_stmt() 路由**

```rust
fn handle_use_stmt(&mut self, use_stmt: &crate::ast::Use) {
    match use_stmt.kind {
        crate::ast::UseKind::Rust => {
            self.handle_rust_import(use_stmt);
            return;
        }
        crate::ast::UseKind::Py => {
            self.handle_py_import(use_stmt);
            return;
        }
        crate::ast::UseKind::C => { ... }
        _ => { /* Auto import */ }
    }
}
```

**Step 4: 在 native_id 解析中添加 py_native_map 检查**

在函数调用的 native_id 解析链中，紧接 `rust_native_map` 之后：

```rust
// Plan 214: Check py_native_map for Python FFI functions
else if self.py_native_map.contains_key(name) {
    let qualified = format!("py.{}", name);
    if let Some(id) = BIGVM_NATIVES.lock().unwrap().resolve_qualified(&qualified) {
        Some(id)
    } else {
        let id = BIGVM_NATIVES.lock().unwrap().register(&qualified);
        Some(id)
    }
}
```

**Step 5: 添加 last_expr_type 跟踪**

在 CALL_NAT 返回类型跟踪中，与 Rust FFI 一致：

```rust
// Plan 214: Python FFI functions return String
if self.py_native_map.contains_key(name) {
    self.last_expr_type = ObjectType::String;
}
```

**Step 6: 类型构造器保护**

在 Store handler 的类型构造器逻辑中，排除 `py_native_map`：

```rust
// 与 Rust FFI 一致，排除 Python 函数名被误判为类型
if self.is_type(type_name)
    && !self.rust_native_map.contains_key(type_name.as_str())
    && !self.py_native_map.contains_key(type_name.as_str()) {
```

**Step 7: Commit**

```bash
git commit -m "feat(vm): handle UseKind::Py in codegen, emit CALL_NAT for python imports (Plan 214 Task 3)"
```

---

## Task 4: 运行时初始化 — `init_py_ffi()`

**文件：**
- 修改：`crates/auto-lang/src/lib.rs`
- 修改：`crates/auto-lang/src/py_ffi.rs`

**目标：** AutoVM 执行前，初始化 PyO3 解释器，导入 Python 模块，注册 native shims。

**Step 1: 实现 PyFfiBridge 核心方法**

在 `py_ffi.rs` 中实现：

```rust
impl PyFfiBridge {
    pub fn import_module(&mut self, module_name: &str) -> Result<(), VMError> {
        pyo3::Python::with_gil(|py| {
            let module = py.import(module_name)
                .map_err(|e| {
                    let err_msg = format!("Failed to import Python module '{}': {}", module_name, e);
                    VMError::FFI(err_msg)
                })?;
            self.modules.insert(
                module_name.to_string(),
                module.into(),
            );
            Ok(())
        })
    }

    pub fn register_function(
        &mut self,
        module_name: &str,
        function_name: &str,
    ) -> Result<u16, VMError> {
        let native_id = self.next_native_id;
        self.next_native_id += 1;

        let qualified = format!("{}.{}", module_name, function_name);
        self.functions.insert(qualified.clone(), native_id);

        // Create shim closure
        let module = self.modules.get(module_name)
            .ok_or_else(|| VMError::FFI(format!("Module {} not imported", module_name)))?
            .clone();
        let func_name = function_name.to_string();

        let shim = self.create_py_shim(module, func_name);

        self.native_interface.register(native_id, Box::new(shim));

        Ok(native_id)
    }
}
```

**Step 2: 实现 create_py_shim**

```rust
fn create_py_shim(
    &self,
    module: pyo3::Py<pyo3::types::PyModule>,
    function_name: String,
) -> impl Fn(&mut crate::vm::task::AutoTask, &crate::vm::AutoVM) -> Result<(), VMError> + Send + Sync + 'static {
    move |task: &mut crate::vm::task::AutoTask, _vm: &crate::vm::AutoVM| {
        // Pop string argument from stack (tagged as -(idx+1))
        let raw = task.ram.pop_i32();
        let str_idx = if raw < 0 { (-(raw) - 1) as usize } else { raw as usize };
        let input_string = task.string_pool.get(str_idx)
            .cloned()
            .unwrap_or_default();

        // Call Python function via PyO3
        let result_string = pyo3::Python::with_gil(|py| {
            let mod_ref = module.bind(py);
            let func: pyo3::Bound<'_, pyo3::types::PyAny> = mod_ref.getattr(&function_name)
                .map_err(|e| VMError::FFI(format!("Python function '{}' not found: {}", function_name, e)))?;

            // Call with string argument
            let py_input = pyo3::types::PyString::new(py, &input_string);
            let py_result = func.call1((py_input,))
                .map_err(|e| VMError::FFI(format!("Python call {}() failed: {}", function_name, e)))?;

            // Extract string result
            let result: String = py_result.extract()
                .map_err(|e| VMError::FFI(format!("Python return value not a string: {}", e)))?;

            Ok::<String, VMError>(result)
        })?;

        // Push result string onto stack
        let idx = task.string_pool.len();
        task.string_pool.push(result_string.into_bytes());
        task.ram.push_i32(-(idx as i32) - 1);

        Ok(())
    }
}
```

**Step 3: 在 lib.rs 中添加 init_py_ffi**

```rust
#[cfg(feature = "python")]
fn init_py_ffi(session: &compile::CompileSession) -> Option<Arc<NativeInterface>> {
    let py_imports = session.py_imports();
    if py_imports.is_empty() {
        return None;
    }

    let mut bridge = PyFfiBridge::new()?;

    for (module_name, functions) in py_imports {
        if let Err(e) = bridge.import_module(&module_name) {
            log::warn!("Failed to import Python module '{}': {:?}", module_name, e);
            continue;
        }

        for func_name in &functions {
            match bridge.register_function(&module_name, func_name) {
                Ok(native_id) => {
                    log::info!("Registered Python FFI: {}.{} (native_id={})", module_name, func_name, native_id);
                    let qualified = format!("py.{}", func_name);
                    BIGVM_NATIVES.lock().unwrap().register_with_id(&qualified, native_id);
                }
                Err(e) => {
                    log::warn!("Failed to register Python function {}.{}: {:?}", module_name, func_name, e);
                }
            }
        }
    }

    Some(bridge.native_interface())
}
```

**Step 4: 在 execute_autovm() 中调用**

```rust
#[cfg(feature = "python")]
let py_ffi_native_interface = init_py_ffi(&session);
#[cfg(not(feature = "python"))]
let py_ffi_native_interface: Option<Arc<NativeInterface>> = None;

// ... after VM creation ...
if let Some(py_ni) = py_ffi_native_interface {
    vm.merge_native_interface(&py_ni);
}
```

**Step 5: Commit**

```bash
git commit -m "feat(py-ffi): add init_py_ffi runtime init with PyO3 shims (Plan 214 Task 4)"
```

---

## Task 5: 端到端集成测试

**文件：**
- 创建：`crates/auto-lang/test/vm/21_python_ffi/001_json5/json5.at`
- 创建：`crates/auto-lang/test/vm/21_python_ffi/001_json5/json5.expected.out`
- 修改：`crates/auto-lang/src/tests/vm_file_tests.rs`

**目标：** 第一个端到端测试：从 Auto 代码调用 Python `json5` 库。

**Step 1: 创建测试用例**

```
21_python_ffi/001_json5/json5.at:
dep json5
use.py json5::{dumps, loads}

let data = "{\"name\":\"auto\",\"ver\":1}"
let obj = loads(data)
print(obj)
```

```
21_python_ffi/001_json5/json5.expected.out:
{'name': 'auto', 'ver': 1}
```

注意：Python `json5.loads()` 返回 Python dict，在 string 模式下需要 `str()` 转换。可能需要调整 shim 以调用 `str()` 包装返回值。或者使用 `json.dumps()` 作为测试（更可控的输出）。

**备选测试（更可控）：**

```
dep json5
use.py json5::{dumps, loads}

let data = "{\"name\":\"auto\",\"ver\":1}"
let obj = loads(data)
let back = dumps(obj)
print(back)
```

```
expected.out:
{"name": "auto", "ver": 1}
```

**Step 2: 标记为 #[ignore] 测试**

```rust
#[test]
#[ignore] // Requires Python + pip, run with: cargo test -p auto-lang --features python -- --ignored
fn test_21_python_ffi_001_json5() { test_vm("21_python_ffi/001_json5").unwrap(); }
```

**Step 3: 测试并调试**

```bash
cargo test -p auto-lang --features python -- test_21_python_ffi_001_json5 --ignored --nocapture
```

**Step 4: Commit**

```bash
git commit -m "test: add E2E Python FFI test for json5 (dep → pip install → PyO3 → call) (Plan 214 Task 5)"
```

---

## Task 6: 清理和计划更新

**目标：** 清理所有 debug 日志，验证完整测试套件无回归，更新计划状态。

**Step 1: 清理**
- 移除所有 `eprintln!("[214...")` debug 语句
- 确保 `#[cfg(feature = "python")]` 正确包裹所有 Python FFI 代码
- 验证不含 `--features python` 时编译正常

**Step 2: 测试**
- `cargo test -p auto-lang` — 无 feature，无回归
- `cargo test -p auto-lang --features python` — 含 feature，Python 测试通过

**Step 3: 更新计划状态**

**Step 4: Commit**

```bash
git commit -m "docs: update Plan 214 status to complete (Plan 214 Task 6)"
```

---

## 重要设计说明

### Feature Gate 策略

所有 Python FFI 代码使用 `#[cfg(feature = "python")]` 包裹，确保：
- 默认编译不需要 Python 开发环境
- 用户 opt-in 安装：`cargo build --features python`
- `PyFfiBridge` 完全隔离在 `py_ffi.rs` 模块中
- `lib.rs` 中的 `init_py_ffi` 用 `#[cfg(feature = "python")]` 包裹
- `codegen.rs` 中的 `py_native_map` 用 `#[cfg(feature = "python")]` 包裹，或始终编译（零成本）

### Native ID 范围

| 范围 | 用途 |
|------|------|
| 0-99 | 内置 intrinsics (print, etc.) |
| 100-299 | BIGVM_NATIVES 基础注册 |
| 300-399 | RustFfiBridge |
| **400-499** | **PyFfiBridge** |
| 500+ | 预留给未来 FFI (Plan 214+) |

### Python GIL 管理

- 每次调用 shim 时通过 `pyo3::Python::with_gil(|py| { ... })` 获取 GIL
- GIL 在 shim 返回后自动释放
- MVP 阶段不涉及多线程，GIL 管理足够
- `pyo3` 的 `auto-initialize` feature 确保解释器自动初始化

### 错误处理

- Python import 失败 → `VMError::FFI("Failed to import Python module ...")`
- Python 调用失败 → `VMError::FFI("Python call failed: ...")`，包含 Python traceback
- pip install 失败 → log::warn，不阻止编译（运行时调用时报错）
- Python 未安装 → `PyFfiBridge::new()` 返回 Err

### 缓存策略

- pip install 缓存：`~/.auto/sandbox/py-packages/{pkg}/installed` marker file
- 后续运行跳过已安装的包
- 清理：`rm -rf ~/.auto/sandbox/py-packages/` 或后续实现 GC
- 不使用 virtualenv — 直接 `--target` 安装到 sandbox 目录

### 后续扩展（不在 MVP 范围）

- Rich type marshaling: int/float/bool/list/dict ↔ Python
- Python 对象方法调用 (`arr.reshape()`)
- Python 异常类型映射到 AutoVM error types
- `dep numpy(version: "1.26")` 版本指定
- virtualenv 隔离
- 多 Python 版本支持
- `use.py` wildcard imports (`use.py numpy::*`)

### 执行前提

- 系统已安装 Python 3.8+
- pip 可用（`pip --version` 成功）
- `cargo build --features python` 需要有效的 Python 开发头文件（PyO3 编译时）
- Windows: 需要 Python development headers (通常随 Python 安装)
- Linux: `sudo apt install python3-dev`
- macOS: Xcode command line tools

---

## 验证清单

- [ ] `cargo build -p auto-lang` — 无 feature，编译成功
- [ ] `cargo build -p auto-lang --features python` — 含 feature，编译成功
- [ ] `cargo test -p auto-lang` — 无 Python 测试运行，无回归
- [ ] `cargo test -p auto-lang --features python -- test_21_python_ffi_001_json5 --ignored` — E2E 通过
- [ ] `use.py json5::{dumps, loads}` — 从 Auto 代码调用 Python 函数
- [ ] 错误处理：不存在的模块 → 友好错误信息
- [ ] 错误处理：不存在的函数 → 友好错误信息
