# Plan 212: Rust FFI 动态加载

> **Phase 1 Status: ✅ COMPLETE** — MVP string→string FFI 已验证
> **Phase 2 Status: 📋 PLANNED** — 扩展支持 struct method 和 primitive I/O
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 AutoVM 对外部 Rust crate 的动态加载和调用。Phase 1 已完成 string→string MVP；Phase 2 扩展覆盖 82 个 B-tier MISSING_DEP 测试。

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

### 后续计划（已迁移到 Phase 2）

Phase 1 后续计划已在上方 **Phase 2** 中系统化规划。剩余远期目标：
- Feature flags (`dep serde(features: ["derive"])`)
- Git dependencies (`dep my_lib(git: "...")`)
- 并行编译多个 deps
- GC 清理未使用的编译缓存
- `auto fetch` 预构建命令
- 版本冲突解决
- 闭包/迭代器跨 FFI 传递（Phase 2.3 complex 场景）

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

---

## Phase 2: 扩展 FFI 覆盖 MISSING_DEP 测试

**日期**: 2026-05-09
**状态**: 📋 PLANNED
**目标**: 将 FFI 从 string→string MVP 扩展为支持 opaque struct pointer 和 primitive I/O，解锁 82 个 B-tier MISSING_DEP 测试中的大部分。

### Phase 2.0: MISSING_DEP 测试分类

81 个 B-tier 测试使用外部 crate，按 FFI 签名需求分为 4 类：

| 签名类型 | 数量 | 占比 | 说明 | Phase |
|----------|------|------|------|-------|
| string→string | 3 | 4% | 当前 Plan 212b 已支持 | ✅ |
| primitive I/O | 7 | 9% | int/bool/float 参数或返回值 | Phase 2.1 |
| struct method | 39 | 48% | 创建外部 struct 实例并调用方法 | Phase 2.2 |
| complex | 32 | 40% | 闭包传递、迭代器、宏、多 crate 交互 | Phase 2.3+ |

**按 crate 分布（Top 6，占 53%）:**

| Crate | 测试数 | 签名类型 | 核心操作 |
|-------|--------|----------|----------|
| rand | 9 | primitive + struct | `rand::random()`, `thread_rng().gen_range()` |
| csv | 7 | struct method | `Reader::from_reader()`, `StringRecord::get()` |
| log + env_logger + tracing | 10 | special | `debug!()`/`info!()` 宏，可简化为 `print()` |
| chrono | 6 | struct method | `Local::now()`, `NaiveDate::parse_from_str()` |
| semver | 6 | struct method | `Version::parse()`, `VersionReq::parse()` |
| regex | 5 | struct method | `Regex::new()`, `.is_match()`, `.replace()` |

**8 个关键 struct 解锁 39 个 struct method 测试：**

| Struct | Crate | 测试数 | 典型用法 |
|--------|-------|--------|----------|
| `Regex` | regex | 5 | `Regex::new(pattern).is_match(text)` |
| `Url` | url | 5 | `Url::parse(url_str).host_str()` |
| `Version` | semver | 5 | `Version::parse(semver_str).major` |
| `Local`/`Utc`/`NaiveDateTime` | chrono | 6 | `Local::now().format(fmt)` |
| `Reader`/`Writer` | csv | 7 | `Reader::from_path(file).records()` |
| `WalkDir` | walkdir | 4 | `WalkDir::new(dir).into_iter()` |
| `Complex` | num | 3 | `Complex::new(re, im).norm()` |
| `BigInt` | num | 1 | `BigInt::parse_bytes(bytes, radix)` |

### Phase 2.1: Primitive I/O 扩展（7 个测试）

**目标**: 支持非 string 的基础类型参数/返回值。

**需要修改的 shim 签名模式:**

当前 MVP 只生成 `(*const c_char) -> *const c_char` 的 shim。需要扩展为：

```rust
// 当前（Phase 1）
pub extern "C" fn auto_from_str(ptr: *const c_char) -> *const c_char

// Phase 2.1 扩展
pub extern "C" fn auto_random_i64() -> i64
pub extern "C" fn auto_gen_range(lo: i64, hi: i64) -> i64
pub extern "C" fn auto_gen_f64() -> f64
pub extern "C" fn auto_parse_int(s: *const c_char) -> i64
```

**实现要点：**

1. **签名推导**: 在 wrapper crate 生成时，根据 `use.rust` 导入的函数名推导签名
   - 维护一个 `known_signatures` 表：函数名 → 参数类型列表 + 返回类型
   - 例如：`rand::random<T>() → T`, `rand::Rng::gen_range(low, high) → T`

2. **VM 端 marshaling 扩展**:
   - `RustFfiBridge` 已有 `RustType` enum（String, I32, I64, F64, Bool 等）
   - 扩展 `register_function()` 接受任意 `RustSignature`（不再限 string→string）
   - VM `push_to_stack` / `pop_from_stack` 已有 int/f64/bool 支持

3. **wrapper crate 生成模板**:
   - 为每种签名组合生成对应的 `#[no_mangle] extern "C"` 函数
   - primitive 参数直接传值（i64, f64, bool），不走 CStr

**涉及文件：**
- `crates/auto-cache/src/sandbox.rs` — `generate_wrapper_crate()` 扩展签名模板
- `crates/auto-lang/src/ffi.rs` — `RustFfiBridge` 注册 primitive 签名
- `crates/auto-lang/src/vm/codegen.rs` — 调用时按签名类型 push 不同参数

**验证测试：** `rand::random()` → `auto_random()` → VM 获得随机 i64

### Phase 2.2: Opaque Struct Pointer（39 个测试）

**目标**: 支持外部 crate 的 struct 实例创建和方法调用。

这是 **性价比最高** 的扩展 — 解锁 48% 的 MISSING_DEP 测试。

**核心设计：Opaque Handle**

AutoVM 不需要理解外部 struct 的布局，只需持有一个 opaque pointer：

```rust
// wrapper crate 中生成的代码
use std::ffi::{c_char, CStr, CString};

// 构造函数：返回 opaque pointer
#[no_mangle]
pub extern "C" fn auto_Regex_new(pattern: *const c_char) -> *mut () {
    let pat = unsafe { CStr::from_ptr(pattern) }.to_str().unwrap_or("");
    let regex = regex::Regex::new(pat).unwrap();
    Box::into_raw(Box::new(regex)) as *mut ()
}

// 方法调用：接收 opaque pointer + 参数
#[no_mangle]
pub extern "C" fn auto_Regex_is_match(handle: *mut (), text: *const c_char) -> bool {
    let regex = unsafe { &*(handle as *const regex::Regex) };
    let txt = unsafe { CStr::from_ptr(text) }.to_str().unwrap_or("");
    regex.is_match(txt)
}

// 析构：释放 opaque pointer
#[no_mangle]
pub extern "C" fn auto_Regex_drop(handle: *mut ()) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle as *mut regex::Regex)); }
    }
}

// 字段访问：从 opaque pointer 提取字段
#[no_mangle]
pub extern "C" fn auto_Version_major(handle: *const ()) -> u64 {
    let ver = unsafe { &*(handle as *const semver::Version) };
    ver.major
}
```

**AutoVM 端映射：**

在 Auto 代码中，外部 struct 的实例在 VM 中存储为 `u64`（opaque pointer）：

```auto
// Auto 代码
use.rust regex::Regex

fn main() {
    let re = Regex.new(r"\d+")
    if re.is_match("hello 123") {
        print("found digits")
    }
}
```

编译为 VM bytecode 时：
1. `Regex.new(pattern)` → `CALL_NAT(auto_Regex_new, pattern)` → 返回 `u64` handle
2. `re.is_match(text)` → `CALL_NAT(auto_Regex_is_match, handle, text)` → 返回 `bool`
3. 离开作用域 → `CALL_NAT(auto_Regex_drop, handle)` (或依赖 GC)

**实现步骤：**

#### Task 2.2.1: 扩展签名类型系统

**文件：** `crates/auto-lang/src/ffi.rs`

扩展 `RustSignature` / `RustType` 支持：
```rust
enum RustType {
    // 已有
    String, I32, I64, F64, Bool, Void,
    // 新增
    OpaqueHandle,  // *mut () — 指向外部 struct 的 opaque pointer
}
```

扩展 `RustSignature`：
```rust
struct RustSignature {
    params: Vec<RustType>,
    return_type: RustType,
    /// 如果是 struct 方法，记录构造函数/方法/析构函数
    call_kind: RustCallKind,
}

enum RustCallKind {
    Function,                          // 普通函数
    Constructor { struct_name: String }, // 构造函数，返回 opaque handle
    Method { struct_name: String },      // 方法，第一个参数是 handle
    FieldGet { struct_name: String, field: String }, // 字段读取
    Drop { struct_name: String },        // 析构函数
}
```

#### Task 2.2.2: wrapper crate 生成模板扩展

**文件：** `crates/auto-cache/src/sandbox.rs`

为 struct method 调用生成三类 shim：
- `auto_{Struct}_{method}(handle, ...args)` — 方法调用
- `auto_{Struct}_new(...args)` — 构造函数
- `auto_{Struct}_drop(handle)` — 析构函数

需要一个 **struct schema** 来描述外部 crate 的 struct 接口：
```rust
struct StructSchema {
    name: String,
    crate_name: String,
    constructors: Vec<FunctionSchema>,
    methods: Vec<FunctionSchema>,
    fields: Vec<FieldSchema>,
}

struct FunctionSchema {
    name: String,
    params: Vec<(String, RustType)>,
    return_type: RustType,
}

struct FieldSchema {
    name: String,
    rust_type: RustType,
}
```

这些 schema 可以：
- 手动编写常用 crate 的 schema（regex, url, semver, chrono 等）
- 或从 Rust doc JSON 自动生成（远期目标）

#### Task 2.2.3: VM codegen 处理 struct method 调用

**文件：** `crates/auto-lang/src/vm/codegen.rs`

当遇到 `re.is_match("text")` 这样的 dot call 时：
1. 识别 `re` 是 opaque handle 类型（来自 `Regex.new()` 返回值）
2. 将 dot call 转换为 `CALL_NAT(auto_Regex_is_match, handle, "text")`
3. handle 作为第一个参数 push 到栈上

**关键问题：类型追踪**

当前 VM codegen 不做跨表达式类型追踪。需要轻量级追踪：
- 构造函数返回值标记为 `OpaqueHandle("Regex")`
- 变量赋值时传播类型
- dot call 时查找 `OpaqueHandle("Regex")` 的方法映射

#### Task 2.2.4: Struct Schema 注册

**文件：** 新建 `crates/auto-lang/src/vm/ffi/struct_schemas.rs`

为 Top 8 结构体手动编写 schema：

```
regex::Regex — 3 methods: new, is_match, replace
url::Url — 4 methods: parse, host_str, path, fragment
semver::Version — 2 methods + 3 fields: parse, major, minor, patch
chrono::Local — 1 method + 1 field: now, format
csv::Reader — 2 methods: from_path, records
walkdir::WalkDir — 2 methods: new, into_iter
num::Complex — 3 methods: new, norm, conj
num::BigInt — 2 methods: parse_bytes, to_string
```

### Phase 2.3: Complex 场景（32 个测试，远期目标）

这类测试需要根本性的 VM 能力扩展，不在本次 Phase 2 范围内：

| 能力 | 涉及测试 | 难度 |
|------|----------|------|
| 闭包作为 FFI 参数 | rayon, crossbeam (6) | 高 |
| 外部迭代器适配器 | csv records, walkdir entries (部分) | 高 |
| Rust 宏调用 | log, tracing (10) | 中 — 可简化为 print |
| 自定义类型反序列化 | serde derive (4) | 高 |
| 多 crate 交互 | tar+flate2, etc. (4) | 中 |

**log/tracing 特殊处理（10 个测试）:**

这些测试使用 `debug!()`/`info!()`/`error!()` 宏，不是普通函数。在 AutoVM 中可以：
- 将 `use.rust log` 映射为内置的 `print()` 调用
- 不需要实际加载 log crate
- 在 native_registry 中直接注册 `auto_log_debug`, `auto_log_info` 等 shim

### Phase 2 实施顺序

```
Phase 2.1 (Primitive I/O, 7 tests)
  ├── Task 2.1.1: 扩展 RustSignature 支持 primitive 类型
  ├── Task 2.1.2: 扩展 wrapper crate 生成模板
  ├── Task 2.1.3: VM marshaling 扩展
  └── Task 2.1.4: 验证测试 (rand::random)

Phase 2.2 (Opaque Struct, 39 tests)
  ├── Task 2.2.1: 扩展签名类型系统 (RustCallKind)
  ├── Task 2.2.2: wrapper crate struct 模板
  ├── Task 2.2.3: VM codegen struct method 调用
  ├── Task 2.2.4: 手动编写 Top 8 struct schema
  └── Task 2.2.5: 验证测试 (regex::Regex, url::Url)

Phase 2.3-log (log/tracing 快捷方案, 10 tests)
  ├── Task 2.3.1: 内置 log shim（不加载外部 crate）
  ├── Task 2.3.2: 验证测试
  └── Task 2.3.3: #macro 调用语法（见 Phase 2.4）

Phase 2.4 (#macro 调用语法)
  ├── Task 2.4.1: Lexer — 识别 #ident( 为宏调用 token
  ├── Task 2.4.2: AST — 添加 MacroCall 节点
  ├── Task 2.4.3: Parser — 解析 #ident(args) 为 MacroCall
  ├── Task 2.4.4: VM codegen — #debug(...) 路由到 Log.debug
  ├── Task 2.4.5: a2r transpiler — #debug(...) 转译为 debug!(...)
  └── Task 2.4.6: 更新 cookbook 测试文件

总计覆盖: 3 (已有) + 7 + 39 + 10 = 59/82 (72%)
剩余 23 个 complex 测试留待 Phase 3+
```

### Phase 2 成功标准

- [ ] Phase 2.1: `rand::random()` 在 AutoVM 中返回随机整数
- [ ] Phase 2.2: `Regex.new(r"\d+").is_match("abc123")` 在 AutoVM 中返回 true
- [ ] Phase 2.2: `Url.parse("https://example.com/path").host_str()` 返回 "example.com"
- [ ] Phase 2.2: `Version.parse("1.2.3").major` 返回 1
- [ ] Phase 2.3-log: `debug!("msg")` 在 AutoVM 中输出 `[DEBUG] msg`
- [ ] Phase 2.4: `#debug("msg")` 语法在 VM 中输出 `[DEBUG] msg`
- [ ] Phase 2.4: `#debug("msg")` 经 a2r 转译为 `debug!("msg")`
- [ ] MISSING_DEP 测试通过率从 0% 提升到 72%

---

## Phase 2.4: `#macro` 调用语法

**日期**: 2026-05-09
**状态**: 🔧 IN PROGRESS
**目标**: 为 Auto 添加 `#macro_name(...)` 宏调用语法，与现有 `#if`/`#for`/`#{...}` 编译期体系一致。

### 设计

Auto 的编译期操作统一使用 `#` 前缀：
- `#if cond { ... }` — 编译期条件
- `#for x in 0..4 { ... }` — 编译期循环
- `#{ expr }` — 编译期表达式
- **`#macro_name(args)` — 宏调用**（新增）

宏调用的语义：
- **VM 执行**：`#debug("msg")` 路由到已有的 `Log.debug` native shim，等效于 `debug("msg")`
- **a2r 转译**：`#debug("msg")` → `debug!("msg")`（去掉 `#`，加 `!`）
- **声明语法**：暂不设计，本阶段只实现调用

示例：
```auto
use.rust log::debug
use.rust log::info

fn main() {
    #debug("starting operation")    // VM: Log.debug / a2r: debug!("...")
    let value = 42
    #info(f"value = $value")        // VM: Log.info  / a2r: info!("...")
    print("done")
}
```

### Task 2.4.1: Lexer — 识别 `#ident(` 宏调用

**文件：** `crates/auto-lang/src/lexer.rs`

在处理 `#` 字符时，当前已有逻辑识别 `#if`、`#for`、`#is`、`#{`。需要新增：

当 `#` 后面跟着标识符字符但不是已知关键字（if/for/is/{）时：
1. 读取完整标识符（如 `debug`）
2. 检查后面是否有 `(`
3. 如果有 `(`：发射 `HashIdent` token（值为标识符名）
4. 如果没有 `(`：报错或 fallback（当前 `#` 后面只能是已知关键字）

新增 token：
```rust
TokenKind::HashIdent  // #ident — macro invocation prefix
```

Token text 存储标识符名（不含 `#`），如 `"debug"`。

### Task 2.4.2: AST — 添加 MacroCall 节点

**文件：** `crates/auto-lang/src/ast/comptime.rs`（或 `ast.rs`）

新增 AST 节点：

```rust
/// #name(args) — Macro invocation
///
/// Calls a macro at compile time. In VM mode, routes to built-in shims.
/// In a2r mode, transpiles to Rust macro syntax: name!(args).
///
/// # Example
/// ```auto
/// #debug("message")
/// #info(f"value = $x")
/// ```
#[derive(Debug, Clone)]
pub struct MacroCall {
    /// Macro name (without # prefix), e.g., "debug"
    pub name: Name,
    /// Arguments
    pub args: Vec<Expr>,
}
```

在 `Stmt` enum 中添加：
```rust
MacroCall(MacroCall),
```

> **注意**：宏调用可以作为 Stmt（语句位置）或 Expr（表达式位置）。目前先作为 Stmt 实现，因为 log/tracing 宏都不返回有意义的值。如果将来有返回值的宏（如 `#include_str!`），可以扩展为同时支持 Expr。

### Task 2.4.3: Parser — 解析 `#ident(args)`

**文件：** `crates/auto-lang/src/parser.rs`

在 `parse_stmt()` 中，`HashIdent` token 的处理：

```rust
TokenKind::HashIdent => {
    let name = self.current().text.to_string();
    self.advance(); // consume HashIdent
    self.expect(TokenKind::LParen)?;
    let args = self.parse_arg_list()?;  // reuse existing arg parsing
    self.expect(TokenKind::RParen)?;
    Stmt::MacroCall(MacroCall { name, args })
}
```

### Task 2.4.4: VM codegen — `#debug(...)` 路由

**文件：** `crates/auto-lang/src/vm/codegen.rs`

在 `compile_stmt()` 中添加 `Stmt::MacroCall` 分支：

```rust
Stmt::MacroCall(macro_call) => {
    // Route #name(args) to the same native shim as name(args)
    // e.g., #debug("msg") → Log.debug, #info("msg") → Log.info
    let routed_name = match macro_call.name.as_str() {
        "debug" => "Log.debug",
        "info" => "Log.info",
        "warn" => "Log.warn",
        "error" => "Log.error",
        other => {
            log::warn!("Unknown macro: #{}", other);
            return Ok(());
        }
    };
    // Compile args + CALL_NAT (reuse existing native call logic)
    for arg in &macro_call.args {
        self.compile_expr(arg)?;
    }
    if let Some(&id) = self.intrinsics.get(routed_name) {
        self.emit(OpCode::CALL_NAT);
        self.emit_u16(id);
    }
    self.last_expr_type = ObjectType::Void;
}
```

### Task 2.4.5: a2r transpiler — `#debug(...)` → `debug!(...)`

**文件：** `crates/auto-lang/src/trans/rust.rs`

在 a2r 的 `transpile_stmt()` 中添加 `MacroCall` 处理：

```rust
Stmt::MacroCall(macro_call) => {
    // #name(args) → name!(args)
    write!(out, "{}!(", macro_call.name)?;
    for (i, arg) in macro_call.args.iter().enumerate() {
        if i > 0 { write!(out, ", ")?; }
        self.transpile_expr(arg, out)?;
    }
    writeln!(out, ");")?;
}
```

### Task 2.4.6: 更新 cookbook 测试文件

更新 devtools 目录下的 log 测试文件，将 `debug!("...")` 改为 `#debug("...")`：

- `test/a2r/cookbook/devtools/001_log_debug/log_debug.at`
- `test/a2r/cookbook/devtools/002_log_error/log_error.at`
- `test/a2r/cookbook/devtools/003_log_stdout/log_stdout.at`
- `test/a2r/cookbook/devtools/005_log_env/log_env.at`
- `test/a2r/cookbook/devtools/006_log_mod/log_mod.at`
- `test/a2r/cookbook/devtools/007_log_timestamp/log_timestamp.at`
- `test/a2r/cookbook/devtools/009_log_custom_location/log_custom_location.at`
- `test/a2r/cookbook/devtools/010_tracing_console/tracing_console.at`

同时更新对应的 `.expected.rs` 文件，确认输出包含 `debug!(...)` 而非 `debug(...)`。

### 实施要点

1. **Lexer 优先**：`#` 后面先检查已知关键字（if/for/is/{），不匹配则尝试读标识符
2. **Parser 复用**：参数解析复用现有的 `parse_arg_list()` 或 `parse_call_args()`
3. **VM 侧行为不变**：`#debug(...)` 和 `debug(...)` 在 VM 中产生完全相同的 CALL_NAT
4. **a2r 侧是核心价值**：`#` 前缀让 a2r 知道这是一个 Rust 宏调用，需要加 `!`
5. **向后兼容**：`debug("msg")`（不带 `#`）仍然可以在 VM 中工作，但 a2r 会转译为 `debug("msg")`（函数调用，非宏）
