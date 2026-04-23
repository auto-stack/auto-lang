# Plan 212b: Rust FFI 动态加载端到端实现

> **Status: 🔧 PARTIAL** (Tasks 1-3+4 code exists, E2E test `#[ignore]`d pending runtime validation)
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 `dep serde_json` → cargo 编译 cdylib → AutoVM 加载 .dll → 调用 `serde_json::from_str` 的完整端到端链路，用一个 crate 验证整个流程。

**Architecture:** AutoVM 运行时遇到 `dep` 语句时，在 `~/.auto/sandbox/` 下生成一个 wrapper crate（Cargo.toml + lib.rs），该 crate 将 `use.rust` 导入的函数包装为 `#[no_mangle] pub extern "C"` 导出，然后调用 `cargo build` 编译为 cdylib，最后通过 `libloading` 加载并通过已有的 `RustFfiBridge::register_function()` 注册到 AutoVM。

**Tech Stack:** Cargo CLI, cdylib, libloading, RustFfiBridge, auto-cache Sandbox/Registry

---

## 当前状态：已建骨架，缺少肌肉

### 已完成（可复用）

| 组件 | 文件 | 状态 |
|------|------|------|
| `CrateMetadata` | `auto-cache/src/sandbox.rs` | 完整，含 ABI 检查 |
| `CrateRegistry` | `auto-cache/src/registry.rs` | SQLite，register/lookup/resolve/remove |
| `Sandbox` | `auto-cache/src/sandbox.rs` | rustc/target 检测，verify_abi，load_crate |
| `RustFfiBridge` | `auto-lang/src/ffi.rs` | 40+ shim 签名模式，symbol 解析，参数 marshaling |
| `RustSignature` / `RustType` | `auto-lang/src/ffi.rs` | 11 种类型 |
| `dep` 语法解析 | lexer/parser/ast | `DepStmt` 完整 |
| `use.rust` 语法解析 | parser/ast | `Use { kind: Rust, paths, items }` |
| `CompileSession::resolve_deps()` | `compile.rs` | 注册 metadata 到 registry |
| `CompileSession::create_rust_ffi_bridge()` | `compile.rs` | 创建 bridge 实例 |

### 缺失（需要实现）

| 缺失环节 | 描述 |
|----------|------|
| **Cargo 编译管线** | `resolve_deps()` 只注册 metadata，从不调用 `cargo build` |
| **Wrapper crate 生成** | 不生成 Cargo.toml + lib.rs 来构建 cdylib |
| **`use.rust` → shim 生成** | 不把 `from_str` 包装为 `#[no_mangle] pub extern "C"` |
| **VM codegen 处理** | `handle_use_stmt()` 直接跳过 `UseKind::Rust`，调用变成普通函数调用 |
| **端到端集成** | 没有任何测试验证 dep → build → load → call 全链路 |

### 断裂点分析

```
dep serde_json          ← 解析 ✅ → DepStmt ✅
    ↓
resolve_deps()          ← 注册 metadata ✅ → 但不编译 ❌
    ↓
use.rust serde_json::{from_str, to_string}  ← 解析 ✅ → 但 codegen 跳过 ❌
    ↓
from_str(json_str)      ← VM 调用时无 native shim → 报错 ❌
```

**核心断裂点：** `resolve_deps()` 和 `handle_use_stmt()` 之间没有连接。需要：
1. `resolve_deps()` 或后续步骤调用 `cargo build` 编译 wrapper crate
2. `handle_use_stmt()` 为 `UseKind::Rust` 注册 native shim
3. 调用 `from_str()` 时 emit `CALL_NAT` 而非普通 `CALL`

---

## 设计：端到端流程

```
用户代码:
  dep serde_json
  use.rust serde_json::{from_str, to_string}
  fn main() {
      let data = from_str("{\"name\":\"auto\"}")
  }

执行流程:
  1. Parser 解析 dep/use.rust → DepStmt + Use(Rust)
  2. resolve_deps("serde_json")
     → 检查 sandbox 缓存: ~/.auto/sandbox/crates/serde_json-*.dll
     → 如果未缓存:
        a. 在 sandbox 下生成 wrapper crate:
           ~/.auto/sandbox/builds/serde_json-wrapper/
           ├── Cargo.toml          (deps: serde_json)
           └── src/
               └── lib.rs          (#[no_mangle] extern "C" shims)
        b. 运行: cargo build --release
        c. 复制 .dll → ~/.auto/sandbox/crates/serde_json-1.0.128.dll
        d. 注册 metadata 到 CrateRegistry
  3. handle_use_stmt(UseKind::Rust)
     → 提取 crate_name="serde_json", items=["from_str", "to_string"]
     → 调用 RustFfiBridge::load_rust_library("serde_json", dll_path)
     → 调用 RustFfiBridge::register_function("serde_json", "from_str", signature)
     → 记录 native_id 映射: "from_str" → native_id
  4. codegen 遇到 from_str(...) 调用
     → 查找 native_id 映射
     → emit CALL_NAT(native_id)
  5. VM 执行 CALL_NAT
     → RustFfiBridge shim 调用实际的 serde_json::from_str
     → marshaling 返回值回 AutoVM
```

---

## Task 1: Sandbox 编译管线 — `compile_dep()`

**文件：**
- 修改：`crates/auto-cache/src/sandbox.rs`

**目标：** 添加 `compile_dep()` 方法，在 sandbox 中生成 wrapper crate 并调用 `cargo build`。

**Step 1: 添加 wrapper crate 生成**

在 `Sandbox` 中添加方法：

```rust
/// Compile a Rust crate dependency as a cdylib
///
/// Generates a wrapper crate that re-exports specified functions as
/// `#[no_mangle] pub extern "C"` symbols, then builds it with cargo.
///
/// # Arguments
/// * `crate_name` - e.g., "serde_json"
/// * `version` - e.g., "1.0", or "" for latest
/// * `functions` - List of function names to export
///
/// # Returns
/// Path to the compiled .dll/.so/.dylib
pub fn compile_dep(
    &self,
    crate_name: &str,
    version: &str,
    functions: &[String],
) -> Result<PathBuf> {
    let build_dir = self.root.join("builds").join(crate_name);

    // Check cache first
    let lib_path = self.crate_library_path(crate_name, version);
    if lib_path.exists() {
        log::info!("Using cached: {}", lib_path.display());
        return Ok(lib_path);
    }

    // Generate wrapper crate
    self.generate_wrapper_crate(&build_dir, crate_name, version, functions)?;

    // Build with cargo
    self.build_wrapper_crate(&build_dir, crate_name, version)?;

    Ok(lib_path)
}

fn generate_wrapper_crate(
    &self,
    build_dir: &Path,
    crate_name: &str,
    version: &str,
    functions: &[String],
) -> Result<()> {
    let src_dir = build_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Generate Cargo.toml
    let dep_line = if version.is_empty() {
        format!(r#"{} = "*""#, crate_name)
    } else {
        format!(r#"{} = "{}""#, crate_name, version)
    };

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}-wrapper"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
{dep_line}
"#
    );
    std::fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;

    // Generate lib.rs with shim functions
    let mut lib_rs = String::new();
    lib_rs.push_str(&format!("use {};\n\n", crate_name));

    for func in functions {
        // Generate a simple shim that calls through
        // We use a generic signature: (*const c_char) -> *const c_char
        // The actual type marshaling happens in RustFfiBridge
        lib_rs.push_str(&format!(
            r#"#[no_mangle]
pub extern "C" fn auto_{func}(ptr: *const std::ffi::c_char) -> *const std::ffi::c_char {{
    let input = unsafe {{ std::ffi::CStr::from_ptr(ptr) }}
        .to_str()
        .unwrap_or("");
    let result = {crate_name}::{func}(input);
    let output = format!("{{}}", result);
    let c_string = std::ffi::CString::new(output).unwrap();
    c_string.into_raw() as *const std::ffi::c_char
}}

"#
        ));
    }

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

fn build_wrapper_crate(
    &self,
    build_dir: &Path,
    crate_name: &str,
    version: &str,
) -> Result<()> {
    let output = Command::new(self.cargo_path())
        .args(["build", "--release"])
        .current_dir(build_dir)
        .output()
        .map_err(|e| SandboxError::CompilationFailed(format!("cargo: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SandboxError::CompilationFailed(format!(
            "cargo build failed for {}:\n{}",
            crate_name, stderr
        )));
    }

    // Copy compiled library to sandbox crates directory
    let lib_filename = {
        let lib_name = crate_name.replace('-', "_");
        #[cfg(target_os = "windows")]
        { format!("{}_wrapper.dll", lib_name) }
        #[cfg(target_os = "macos")]
        { format!("lib{}_wrapper.dylib", lib_name) }
        #[cfg(target_os = "linux")]
        { format!("lib{}_wrapper.so", lib_name) }
    };

    let built_lib = build_dir
        .join("target")
        .join("release")
        .join(&lib_filename);

    let dest = self.crate_library_path(crate_name, version);

    if built_lib.exists() {
        std::fs::copy(&built_lib, &dest)?;
    } else {
        return Err(SandboxError::CompilationFailed(format!(
            "Compiled library not found at {}",
            built_lib.display()
        )));
    }

    Ok(())
}
```

**注意：** 上面的 shim 生成是最简单的 `string → string` 模式。不同的函数签名需要不同的 shim。对于 E2E 验证，先用 `serde_json::from_str` 和 `serde_json::to_string` 这两个 string→string 函数。

**Step 2: 测试编译管线**

Run: `cargo test -p auto-cache -- sandbox::tests --nocapture`

**Step 3: Commit**

```bash
git add -u
git commit -m "feat(auto-cache): add compile_dep() to sandbox for cdylib wrapper generation"
```

---

## Task 2: 连接 resolve_deps() 到编译管线

**文件：**
- 修改：`crates/auto-lang/src/compile.rs`

**目标：** `resolve_deps()` 在注册 metadata 后，调用 `sandbox.compile_dep()` 编译 wrapper crate。

**Step 1: 添加 use_rust_imports 收集**

当前 `resolve_deps()` 只处理 `dep` 语句。还需要收集 `use.rust` 语句中的函数名，因为 wrapper crate 需要知道导出哪些函数。

修改 `CompileSession` 添加字段：

```rust
/// use.rust imports: crate_name -> [function_names]
rust_imports: HashMap<String, Vec<String>>,
```

**Step 2: 添加 collect_rust_imports() 方法**

```rust
/// Collect use.rust imports from source
pub fn collect_rust_imports(&mut self, source: &str) {
    let use_stmts = scan_use_statements(source);
    for stmt in use_stmts {
        if stmt.is_rust_import {
            let crate_name = stmt.module.split("::").next().unwrap_or("").to_string();
            let items = stmt.items.clone();
            self.rust_imports
                .entry(crate_name)
                .or_default()
                .extend(items);
        }
    }
}
```

**Step 3: 修改 resolve_deps() 触发编译**

在 `resolve_deps()` 末尾，对于已有 `rust_imports` 的 crate，触发编译：

```rust
// After registering all deps, compile those that have imports
for (crate_name, functions) in &self.rust_imports {
    if self.declared_crates.contains(crate_name) {
        if let Some(ref sandbox) = self.sandbox {
            let version = self.dep_versions.get(crate_name).map(|s| s.as_str()).unwrap_or("");
            match sandbox.compile_dep(crate_name, version, functions) {
                Ok(lib_path) => {
                    log::info!("Compiled {} -> {}", crate_name, lib_path.display());
                }
                Err(e) => {
                    log::error!("Failed to compile {}: {}", crate_name, e);
                }
            }
        }
    }
}
```

**Step 4: Commit**

```bash
git commit -m "feat(compile): connect resolve_deps to sandbox compilation pipeline"
```

---

## Task 3: VM Codegen 处理 UseKind::Rust

**文件：**
- 修改：`crates/auto-lang/src/vm/codegen.rs`

**目标：** `handle_use_stmt()` 不再跳过 `UseKind::Rust`，而是为每个导入的函数注册 native shim。

**Step 1: 修改 handle_use_stmt()**

在 `codegen.rs` 的 `handle_use_stmt()` 中，当前第 2893 行直接 `return`。改为：

```rust
fn handle_use_stmt(&mut self, use_stmt: &crate::ast::Use) {
    match use_stmt.kind {
        crate::ast::UseKind::Auto => {
            // ... existing Auto import handling (unchanged)
        }
        crate::ast::UseKind::Rust => {
            self.handle_rust_import(use_stmt);
        }
        crate::ast::UseKind::C => {
            // C imports handled separately
        }
    }
}
```

**Step 2: 添加 handle_rust_import()**

```rust
/// Register use.rust imports as native functions
fn handle_rust_import(&mut self, use_stmt: &crate::ast::Use) {
    if use_stmt.paths.is_empty() {
        return;
    }

    let crate_name = &use_stmt.paths[0];

    // For deep paths like use.rust serde::json::{from_str},
    // paths = ["serde", "json"], items = ["from_str"]
    // The full path is serde::json::from_str
    let module_path = use_stmt.paths.join("::");

    for item in &use_stmt.items {
        let full_path = if use_stmt.paths.len() > 1 {
            format!("{}::{}", module_path, item)
        } else {
            format!("{}::{}", crate_name, item)
        };

        // Generate a native function name for this import
        let native_name = format!("auto_{}", item);

        // Record the mapping: local_name -> (crate_name, native_name)
        self.rust_native_map
            .insert(item.to_string(), (crate_name.to_string(), native_name));
    }
}
```

需要在 `Codegen` struct 上添加字段：

```rust
/// Maps local function name -> (crate_name, native_export_name)
rust_native_map: HashMap<String, (String, String)>,
```

**Step 3: 修改函数调用 codegen**

当遇到函数调用且该函数名在 `rust_native_map` 中时，emit `CALL_NAT` 而非普通 `CALL`。

在 `compile_call()` 方法中，查找函数名：

```rust
// Check if this is a Rust FFI call
if let Some((crate_name, native_name)) = self.rust_native_map.get(&func_name) {
    // Look up the native_id registered by RustFfiBridge
    if let Some(native_id) = self.rust_native_ids.get(&func_name) {
        // Compile arguments
        for arg in &args {
            self.compile_expr(arg)?;
        }
        self.emit(OpCode::CALL_NAT);
        self.emit_u16(*native_id);
        return Ok(());
    }
}
```

**Step 4: Commit**

```bash
git commit -m "feat(vm): handle UseKind::Rust in codegen, emit CALL_NAT for rust imports"
```

---

## Task 4: 运行时桥接 — AutoVM 初始化 RustFfiBridge

**文件：**
- 修改：`crates/auto-lang/src/vm/engine.rs` 或 `crates/auto-lang/src/lib.rs`

**目标：** AutoVM 执行前，加载编译好的 .dll 并注册函数。

**Step 1: 在 run() 或 run_autovm() 中添加 FFI 初始化**

在 AutoVM 执行代码之前：
1. 解析 `dep` 和 `use.rust` 语句
2. 调用 `resolve_deps()` (会触发编译)
3. 创建 `RustFfiBridge`
4. 加载编译好的 .dll
5. 为每个 `use.rust` 导入的函数注册 shim
6. 将 native_id 映射传递给 codegen

```rust
fn init_rust_ffi(source: &str) -> Result<Option<(Arc<NativeInterface>, HashMap<String, u16>)>, AutoError> {
    let dep_stmts = scan_dep_statements(source);
    let use_stmts = scan_use_statements(source);

    let rust_imports: Vec<_> = use_stmts.iter()
        .filter(|u| u.is_rust_import)
        .collect();

    if rust_imports.is_empty() || dep_stmts.is_empty() {
        return Ok(None);
    }

    // Create session and resolve deps (triggers compilation)
    let mut session = CompileSession::new();
    session.collect_rust_imports(source);
    session.resolve_deps(source)?;

    // Create FFI bridge and load compiled libraries
    let mut bridge = session.create_rust_ffi_bridge()?;

    let mut native_ids = HashMap::new();

    for use_stmt in &rust_imports {
        let crate_name = use_stmt.module.split("::").next().unwrap_or("");
        if !session.is_dep_declared(crate_name) {
            continue;
        }

        // Load the compiled library
        let version = "";  // TODO: get from dep statement
        if bridge.load_rust_crate(crate_name, version).is_err() {
            // Try loading from direct path
            let sandbox = Sandbox::new()?;
            let lib_path = sandbox.crate_library_path(crate_name, version);
            if lib_path.exists() {
                bridge.load_rust_library(crate_name, &lib_path)?;
            } else {
                log::warn!("Compiled library not found for {}", crate_name);
                continue;
            }
        }

        // Register each imported function
        for func_name in &use_stmt.items {
            let export_name = format!("auto_{}", func_name);
            let signature = RustSignature::new()
                .param(RustType::String)
                .returns(RustType::String);

            match bridge.register_function(crate_name, &export_name, signature) {
                Ok(native_id) => {
                    native_ids.insert(func_name.to_string(), native_id);
                }
                Err(e) => {
                    log::warn!("Failed to register {}: {}", func_name, e);
                }
            }
        }
    }

    let native_interface = bridge.into_native_interface_arc();
    Ok(Some((native_interface, native_ids)))
}
```

**Step 2: 在 run_with_capture() 中调用 init_rust_ffi**

```rust
pub fn run_with_capture(code: &str) -> AutoResult<(String, String)> {
    // Initialize Rust FFI if needed
    let ffi = init_rust_ffi(code)?;

    // Pass FFI bridge to AutoVM
    // ... existing VM setup code ...
    // vm.set_native_interface(ffi.native_interface);
    // codegen.set_rust_native_ids(ffi.native_ids);
}
```

**Step 3: Commit**

```bash
git commit -m "feat(vm): initialize RustFfiBridge at runtime for dep/use.rust"
```

---

## Task 5: 端到端集成测试

**文件：**
- 创建：`crates/auto-lang/test/vm/20_rust_ffi/001_serde_json/serde_json.at`
- 创建：`crates/auto-lang/test/vm/20_rust_ffi/001_serde_json/serde_json.expected.out`
- 修改：`crates/auto-lang/src/tests/vm_file_tests.rs`

**目标：** 第一个端到端测试：从 Auto 代码调用 `serde_json::from_str`。

**Step 1: 创建测试用例**

```
20_rust_ffi/001_serde_json/serde_json.at:
dep serde_json
use.rust serde_json::{from_str, to_string}

fn main() {
    let json = "{\"name\":\"auto\",\"ver\":1}"
    let data = from_str(json)
    print(data)
    let back = to_string(data)
    print(back)
}
```

```
20_rust_ffi/001_serde_json/serde_json.expected.out:
{"name":"auto","ver":1}
{"name":"auto","ver":1}
```

**注意：** 这个测试第一次运行会很慢（需要 cargo download + compile serde_json）。后续运行使用缓存会很快。

**Step 2: 标记为 #[ignore] 测试**

因为需要网络和 cargo，标记为 `#[ignore]`，手动运行：

```rust
#[test]
#[ignore] // Requires network + cargo, run with: cargo test -- --ignored
fn test_20_rust_ffi_001_serde_json() { test_vm("20_rust_ffi/001_serde_json").unwrap(); }
```

**Step 3: 测试并调试**

```bash
cargo test -p auto-lang --lib -- test_20_rust_ffi_001_serde_json --ignored --nocapture
```

**Step 4: Commit**

```bash
git commit -m "test: add E2E Rust FFI test for serde_json (dep → compile → load → call)"
```

---

## Task 6: 修复和迭代

根据 Task 5 的测试结果修复问题：

- Wrapper crate 的 shim 签名可能需要调整
- Cargo.toml 的依赖版本可能需要调整
- `RustFfiBridge` 的 symbol 查找可能需要修改（当前查找 `auto_from_str`）
- Marshaling 细节：string → JSON → string 的往返可能需要特殊处理

---

## 重要设计说明

### Wrapper crate shim 策略

对于 E2E 验证，所有 shim 使用 `string → string` 签名：

```rust
#[no_mangle]
pub extern "C" fn auto_from_str(ptr: *const c_char) -> *const c_char {
    let input = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("");
    let result = serde_json::from_str(input);  // Returns Result<Value>
    let output = match result {
        Ok(v) => format!("{}", v),
        Err(e) => format!("ERROR: {}", e),
    };
    let c_string = CString::new(output).unwrap();
    c_string.into_raw() as *const c_char
}
```

这个策略简单，但对于更复杂的签名（如 `serde_json::Value` 的方法调用）需要后续扩展。

### 缓存策略

- 首次编译：`~/.auto/sandbox/builds/{crate}/` → `cargo build --release` → 复制到 `~/.auto/sandbox/crates/`
- 后续运行：检查 `~/.auto/sandbox/crates/{crate}-*.dll` 是否存在
- 清理：手动 `rm -rf ~/.auto/sandbox/` 或后续实现 GC

### 安全注意事项

- `cargo build` 在子进程中运行，有超时保护
- 编译的 .dll 只从 sandbox 目录加载，不接受用户指定路径
- wrapper crate 只导出 `use.rust` 中明确声明的函数

### 后续计划

- 支持非 string 签名（int, bool, struct pointers）
- Feature flags (`dep serde(features: ["derive"])`)
- Git dependencies (`dep my_lib(git: "...")`)
- 并行编译多个 deps
- GC 清理未使用的编译缓存
- `auto fetch` 预构建命令
- 版本冲突解决

### 下游依赖：Plan 214（use.py Python FFI）

**Plan 214（AutoVM Python FFI — `use.py` 嵌入 Python 解释器）依赖本计划完成。**

本计划建立的 FFI 基础设施（wrapper 生成、运行时动态加载、参数 marshaling、native shim 注册、dep 声明与 use 语句的 codegen 连接）的架构经验将直接迁移到 Python FFI。本计划完成后，需对 FFI 功能进行分析，评估 `RustFfiBridge` 模式是否可泛化为通用的 `ForeignFfiBridge`，然后确定 Plan 214 的详细实施方案。

关键迁移点：
- `resolve_deps()` → `compile_dep()` 的管线可复用于 Python 包管理（pip install + virtualenv）
- `RustFfiBridge::register_function()` 的 shim 模式可复用于 Python 函数包装
- VM codegen 中 `UseKind::Rust` 的处理可扩展为 `UseKind::Py`
- 参数 marshaling 框架可扩展为 AutoVM ↔ PyObject 转换

### 执行顺序

本计划（212）完成后的执行顺序：

1. **总结 Plan 212 经验** — 分析 FFI 管线中哪些模式可泛化、哪些是 Rust 特有的，评估 `RustFfiBridge` 是否可抽象为通用的 `ForeignFfiBridge`
2. **写 Plan 214 详细内容** — 基于上述分析，将 Plan 214 从占位文件扩展为完整实施计划
3. **执行 Plan 213**（a2py 成熟化）— Python 转译器，与 FFI 无关，可独立推进
4. **执行 Plan 214**（use.py Python FFI）— 依赖 212 的 FFI 经验和 213 的 a2py 基础
5. **执行 Plan 215**（a2ts 成熟化）及后续计划
