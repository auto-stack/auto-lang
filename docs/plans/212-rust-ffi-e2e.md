# Plan 212: Rust FFI 动态加载

> **Phase 1 Status: ✅ COMPLETE** — MVP string→string FFI + cdylib 管线端到端验证
> **Phase 2 Status: ✅ COMPLETE** — Phase 2.1/2.2/2.3/2.4 all done
> **Phase 2.1 Status: ✅ COMPLETE** — Primitive type (i64/f64/bool) cdylib FFI 支持
> **Phase 2.3 Opaque Shims: ✅ COMPLETE** — chrono/base64/hex/sha2/mime_guess 内置 shim
> **Phase 3 Status: ✅ COMPLETE** — Phase 3A/3B/3C-v2/3D all done
>
> **Remaining: Phase 2.3 Complex (11 impossible, 11 hard — 远期目标)**
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 AutoVM 对外部 Rust crate 的动态加载和调用。Phase 1 已完成 string→string MVP；Phase 2 扩展覆盖 82 个 B-tier MISSING_DEP 测试。

**Architecture:** AutoVM 运行时遇到 `dep` 语句时，在 `~/.auto/sandbox/` 下生成一个 wrapper crate（Cargo.toml + lib.rs），该 crate 将 `use.rust` 导入的函数包装为 `#[no_mangle] pub extern "C"` 导出，然后调用 `cargo build` 编译为 cdylib，最后通过 `libloading` 加载并通过已有的 `RustFfiBridge::register_function()` 注册到 AutoVM。

**Tech Stack:** Cargo CLI, cdylib, libloading, RustFfiBridge, auto-cache Sandbox/Registry

---

## 当前状态：管线完整 + Phase 2.3 opaque shims 完成，complex 场景远期目标

### ✅ 已完成

| 组件 | 文件 | 状态 |
|------|------|------|
| `CrateMetadata` / `CrateRegistry` | `auto-cache/src/sandbox.rs` | 完整 |
| `Sandbox.compile_dep()` | `auto-cache/src/sandbox.rs` | ✅ cdylib 编译 + 缓存 |
| `ShimType` / `FunctionShim` | `auto-cache/src/sandbox.rs` | ✅ Phase 2.1 多签名支持 |
| `RustFfiBridge` | `auto-lang/src/ffi.rs` | ✅ 40+ shim 签名模式 |
| `RustSignature` / `RustType` | `auto-lang/src/ffi.rs` | ✅ 11 种类型 |
| `known_signature()` | `auto-lang/src/ffi.rs` | ✅ Phase 2.1 签名数据库 |
| `dep` 语法解析 | lexer/parser/ast | ✅ `DepStmt` 完整 |
| `use.rust` 语法解析 + codegen | `codegen.rs` | ✅ CALL_NAT 路由 |
| `resolve_deps()` → `compile_dep()` | `compile.rs` | ✅ 管线打通 |
| `init_rust_ffi()` | `lib.rs` | ✅ 运行时加载 + 签名传播 |
| Built-in opaque shims (regex/url/semver) | `native.rs` | ✅ Phase 2.2 |
| Built-in opaque shims (chrono/base64/hex/sha2/mime_guess) | `native.rs` + `native_registry.rs` + `codegen.rs` | ✅ Phase 2.3 |
| Log/tracing shims + `#macro` 语法 | `native.rs` + `codegen.rs` | ✅ Phase 2.3-log/2.4 |
| E2E 测试 (serde_json) | `test/vm/20_rust_ffi/` | ✅ |
| Primitive type cdylib (rand::random) | `ffi.rs` + `sandbox.rs` | ✅ Phase 2.1 |

### 🔲 未完成（远期目标）

| 缺失 | 描述 | Phase |
|------|------|-------|
| **Phase 2.3 Complex** | 闭包传递、迭代器适配器、自定义反序列化、多 crate 交互 | 远期 |
| **更多 known_signatures** | chrono/csv/walkdir/num 等需手动添加到签名表 | 按需 |
| **缓存 GC** | `~/.auto/sandbox/` 无自动清理 | 远期 |
| **Feature flags** | `dep serde(features: ["derive"])` | 远期 |
| **Git deps** | `dep my_lib(git: "...")` | 远期 |
| **并行编译** | 多个 dep 同时编译 | 远期 |

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

**日期**: 2026-05-09
**状态**: ✅ COMPLETE
**目标**: 支持非 string 的基础类型参数/返回值。通过 cdylib 管线验证 `rand::random()` 返回 i64。

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

### Phase 2.2: Built-in Opaque Struct Shims（39 个测试）

**日期**: 2026-05-09
**状态**: ✅ COMPLETE
**目标**: 用方案 A（built-in shims，实际 Rust crate 作为依赖）为 regex::Regex、url::Url、semver::Version 添加内置 opaque struct shims。方案 B（cdylib 编译管线）记录为未来 TODO。

#### 核心设计：Built-in Shims + Opaque Handle

直接在 VM native.rs 中用实际 Rust crate 作为依赖，实现 native shims。不需要动态加载 .dll。

**模式**（与已有的 rand shim 一致）：
1. 构造函数：`Regex::new(pattern)` → 创建 `Mutex<regex::Regex>` 包裹在 `RustStdlibObject` 中，存入 VM heap，返回 i32 handle
2. 方法调用：`re.is_match(text)` → 从 stack 取 handle，downcast 到 `Mutex<regex::Regex>`，调用实际方法
3. 析构：依赖 VM GC

```rust
// native.rs — Regex shim 示例
fn shim_regex_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let pattern = task.ram.pop_str();
    let re = regex::Regex::new(&pattern).map_err(|e| VMError::from(e.to_string()))?;
    let obj = RustStdlibObject::new("regex::Regex", Mutex::new(re));
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

fn shim_regex_is_match(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let text = task.ram.pop_str();
    let re_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().is_match(&text);
                task.ram.push_bool(result);
                return Ok(());
            }
        }
    }
    task.ram.push_bool(false);
    Ok(())
}
```

#### ID 分配

| Range | Crate | Prefix |
|-------|-------|--------|
| 2400-2409 | regex | `auto.re_opaque.*` |
| 2500-2509 | url | `auto.url_opaque.*` |
| 2600-2609 | semver | `auto.semver_opaque.*` |

> 注：`auto.url.*` (2000-2015) 已用于字符串解析版 URL，不冲突。
> `auto.regex.*` (2400-2401) 已有简单版本，re_opaque 是新的 opaque 版本。

#### 修改文件

1. `crates/auto-lang/Cargo.toml` — 添加 `url = "2"`, `semver = "1"` 依赖
2. `crates/auto-lang/src/vm/native.rs` — 添加 Regex/Url/Semver shim 函数
3. `crates/auto-lang/src/vm/native_registry.rs` — 注册 native IDs
4. `crates/auto-lang/src/vm/codegen.rs` — 添加方法调用路由和返回类型推断

#### Regex Shims (ID 2400-2409)

| ID | Name | 功能 |
|----|------|------|
| 2400 | `auto.re_opaque.new` | `Regex::new(pattern)` → opaque handle |
| 2401 | `auto.re_opaque.is_match` | `re.is_match(text)` → bool |
| 2402 | `auto.re_opaque.find` | `re.find(text)` → string or None |
| 2403 | `auto.re_opaque.find_all` | `re.find_iter(text)` → List\<string\> |
| 2404 | `auto.re_opaque.replace_all` | `re.replace_all(text, rep)` → string |
| 2405 | `auto.re_opaque.captures` | `re.captures(text)` → opaque captures |
| 2406 | `auto.re_opaque.drop` | drop regex handle |

覆盖测试（a2r cookbook/text/）：001_regex_replace, 002_regex_email
高级测试（需额外 API）：003_regex_hashtags, 005_filter_log, 006_phone

#### Url Shims (ID 2500-2509)

| ID | Name | 功能 |
|----|------|------|
| 2500 | `auto.url_opaque.parse` | `Url::parse(url_str)` → opaque handle |
| 2501 | `auto.url_opaque.scheme` | `url.scheme()` → string |
| 2502 | `auto.url_opaque.host_str` | `url.host_str()` → string or None |
| 2503 | `auto.url_opaque.path` | `url.path()` → string |
| 2504 | `auto.url_opaque.fragment` | `url.fragment()` → string or None |
| 2505 | `auto.url_opaque.port` | `url.port()` → int or None |
| 2506 | `auto.url_opaque.query_pairs` | `url.query_pairs()` → List\<string\> |
| 2507 | `auto.url_opaque.join` | `url.join(rel)` → opaque handle |
| 2508 | `auto.url_opaque.origin` | `url.origin()` → string |
| 2509 | `auto.url_opaque.drop` | drop url handle |

覆盖测试（a2r cookbook/web/url/）：001_base, 002_parse, 003_fragment, 004_new, 005_origin

#### Semver Shims (ID 2600-2609)

| ID | Name | 功能 |
|----|------|------|
| 2600 | `auto.semver_opaque.parse` | `Version::parse(ver_str)` → opaque handle |
| 2601 | `auto.semver_opaque.major` | `v.major` → int |
| 2602 | `auto.semver_opaque.minor` | `v.minor` → int |
| 2603 | `auto.semver_opaque.patch` | `v.patch` → int |
| 2604 | `auto.semver_opaque.pre` | `v.pre.to_string()` → string |
| 2605 | `auto.semver_opaque.to_string` | `v.to_string()` → string |
| 2606 | `auto.semver_opaque.cmp_gt` | `v1 > v2` → bool |
| 2607 | `auto.semver_opaque.drop` | drop version handle |

覆盖测试（a2r cookbook/versioning/）：001_semver_parse, 003_semver_latest, 004_semver_command, 006_semver_prerelease
需额外支持：002_semver_increment (mutable fields), 005_semver_complex (VersionReq)

#### Codegen 路由

在 codegen.rs 的方法调用路由中添加 Regex/Url/Semver 路由：

```rust
// Regex routing
"new" if fname.contains("Regex") => func_name = Some("auto.re_opaque.new".into()),
"is_match" => func_name = Some("auto.re_opaque.is_match".into()),
"replace_all" => func_name = Some("auto.re_opaque.replace_all".into()),
// Url routing
"parse" if fname.contains("Url") => func_name = Some("auto.url_opaque.parse".into()),
"scheme" if fname.contains("Url") => func_name = Some("auto.url_opaque.scheme".into()),
// Semver routing
"parse" if fname.contains("Version") => func_name = Some("auto.semver_opaque.parse".into()),
```

返回类型推断：构造函数 → Int (handle), 方法 → String/Int/Bool 视情况。

#### `use.rust` 导入处理

当 `use.rust regex::Regex` 时，VM 需要识别 `Regex` 为 opaque 类型而非尝试 Rust stdlib dispatch。
在 codegen 中对 Regex/Url/Version 等已知 opaque 类型直接路由到对应 shim。

#### 实施顺序

1. **Regex** → 验证 001/002 测试通过
2. **Url** → 验证 001/003 测试通过
3. **Semver** → 验证 001 测试通过
4. 批量验证所有通过的测试

#### 未来 TODO

- 方案 B：cdylib 编译管线，将外部 crate 编译为 .dll/.so，VM 动态加载
- 高级 API shims：Regex captures_iter, captures.get(n), find_iter, cap.as_str()
- Semver mutable fields（patch += 1 等）
- VersionReq shim
- chrono, csv, walkdir, num 等 crate shims

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

- [x] Phase 2.1: `rand::random()` 在 AutoVM 中返回随机整数
- [x] Phase 2.2: `Regex.new("\\d+").is_match("abc123")` 在 AutoVM 中返回 true
- [x] Phase 2.2: `Url.parse("https://example.com/path").host_str()` 返回 "example.com"
- [x] Phase 2.2: `Version.parse("1.2.3").major` 返回 1
- [x] Phase 2.3-log: `debug!("msg")` 在 AutoVM 中输出 `[DEBUG] msg`
- [x] Phase 2.4: `#debug("msg")` 语法在 VM 中输出 `[DEBUG] msg`
- [x] Phase 2.4: `#debug("msg")` 经 a2r 转译为 `debug!("msg")`
- [x] Phase 2.1: `dep rand` + `use.rust rand::random` 通过 cdylib 返回随机整数
- [x] Phase 2.3: `Local.now().year()` 在 AutoVM 中返回当前年份
- [x] Phase 2.3: `Sha256.new().update("x").finalize()` 返回正确 hex 哈希
- [x] Phase 2.3: `encode("hello")` → `aGVsbG8=` (base64), `encode("hello")` → `68656c6c6f` (hex)
- [x] Phase 2.3: `from_path("file.pdf")` → `application/pdf`

---

## Phase 2.3 Opaque Shims: chrono/base64/hex/sha2/mime_guess

**日期**: 2026-05-10
**状态**: ✅ COMPLETE
**目标**: 为常用 crate 添加 VM 内置 opaque shim，让 Auto 脚本直接使用无需 cdylib 编译。

### 已实现的 Shim

| Crate | Native IDs | 类型 | 函数 |
|-------|-----------|------|------|
| chrono | 2700-2709 | Opaque handle (NaiveDateTime) | `Local.now()`, `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`, `timestamp()`, `format()` |
| base64 | 2710-2711 | Pure function | `encode()`, `decode()` |
| hex | 2720-2721 | Pure function | `encode()`, `decode()` |
| sha2 | 2730-2739 | Opaque handle (Sha256) | `Sha256.new()`, `update()`, `finalize()` |
| mime_guess | 2740 | Pure function | `from_path()` |

### 关键设计决策

1. **Opaque handle 模式**: chrono 和 sha2 使用 `RustStdlibObject` 存储 Mutex 包装的 Rust 对象，通过 heap ID 引用
2. **自由函数路由**: base64/hex/mime_guess 的函数调用（如 `encode("hello")`）在 codegen 中被拦截并路由到内置 shim，绕过 cdylib FFI 管线
3. **返回类型注册**: 所有 shim 在 `native_registry.rs` 中注册了正确的 `NativeRetType`，确保 codegen 生成正确的 print 操作码

### 用法示例

```auto
# chrono
dep chrono
use.rust chrono::Local
let dt = Local.now()
print(dt.format("%Y-%m-%d"))

# sha2
dep sha2
use.rust sha2::Sha256
let h = Sha256.new()
h.update("hello world")
print(h.finalize())

# base64
dep base64
use.rust base64::encode
print(encode("hello"))
```

---

## Phase 2.4: `#macro` 调用语法

**日期**: 2026-05-09
**状态**: ✅ COMPLETE
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

---

## Phase 3: 无缝 FFI — 自动 Shim、自动注册、自动签名推断

**日期**: 2026-05-26
**状态**: 📋 PLANNED
**前置条件**: Phase 2 完成
**Related**: [Plan 265 (AutoVM MCP Server)](265-autovm-mcp-server.md), [Plan 266 (VM↔a2r Conformance)](266-vm-a2r-conformance.md)

### 目标

将 Rust FFI 从"能用"提升为"无缝"。具体而言：

| 当前状态 (Phase 2 后) | 目标状态 (Phase 3 后) |
|---|---|
| 内置 shim 需要手写（regex/url/semver 等 39 个） | `#[rust_fn]` 宏自动生成绝大多数 shim |
| 手动 shim 需要 `pop_from_stack` / `push_to_stack` 样板代码 | 只写业务逻辑，marshal 自动生成 |
| Native ID 手动分配（`pub const NATIVE_FILE_READ_TEXT: u16 = 1000`） | 按名字注册，无需手动 ID |
| 手动调用 `register_shim_by_name()` 注册 | `inventory::submit!` 编译时自动注册 |
| 三方 crate 签名需要硬编码在 `known_signatures` 表 | 从 rustdoc JSON 自动推断 |
| cdylib marshal 只有 40 种固定 pattern | 从 Rust 类型自动生成 marshal 代码 |
| `use.rust any_crate::any_func` 需要提前配置 | 零配置，任意 crate 任意函数可直接调用 |

### 设计：三层自动化

```
Layer 3: 签名自动推断 (Signature Inference)
         dep any_crate → 解析 rustdoc JSON → 自动生成签名
         零配置调用任意 crate

Layer 2: 动态 Marshal 自动生成
         从签名自动生成 VM 栈 ↔ Rust 类型的转换代码
         不再需要 40 种硬编码 pattern

Layer 1: Shim 自动生成 + 自动注册
         #[rust_fn] 生成 marshal + inventory 注册
         淘汰手动 shim 和手动 ID
```

### Phase 3A: Shim 自动生成 + 手动 Shim 迁移

**目标**: 将现有 ~100 个手动 shim 迁移到 `#[rust_fn]`，消除手动 ID 和手动注册。

#### 当前三种 Shim 模式回顾

**模式 1: 手动 Marshal（原始，约 100 个）**

每个函数 10-15 行样板代码：
```rust
// 当前: crates/auto-lang/src/vm/ffi/stdlib.rs
fn shim_file_read_text_vm(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    match fs::read_to_string(&path) {
        Ok(content) => content.push_to_stack(task, vm)
            .map_err(|e| VMError::RuntimeError(e.to_string())),
        Err(_) => {
            let empty = String::new();
            empty.push_to_stack(task, vm).map_err(|e| VMError::RuntimeError(e.to_string()))
        }
    }
}

// 还需要手动分配 ID + 手动注册
pub const NATIVE_FILE_READ_TEXT: u16 = 1000;
// ...
natives.register_shim_by_name("auto.file.read_text", shim_file_read_text_vm);
natives.register_shim_by_name("auto.fs.read_text", shim_file_read_text_vm);
natives.register_shim_by_name("auto.fs.read", shim_file_read_text_vm);  // 别名也要手动
```

**模式 2: `#[rust_fn]` 宏（半自动，约 30 个）**

只写业务逻辑，宏自动生成 marshal + 注册：
```rust
// 当前已在使用: 只需 3 行
#[auto_macros::rust_fn("File.read_text")]
pub fn shim_file_read_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("File.read_text failed: {} - {}", path, e))
}
// 宏自动生成:
//   1. __shim_File_read_text(task, vm) — 自动 pop/push
//   2. inventory::submit! { StaticFFIRegistration { name: "File.read_text", shim: ... } }
```

**模式 3: 动态 cdylib（三方 crate，约 40 种 pattern）**

运行时根据签名自动 marshal：
```rust
// crates/auto-lang/src/ffi.rs — 运行时根据 RustSignature 自动生成
fn create_rust_shim_lazy(&self, signature: RustSignature) -> impl Fn(...) {
    move |task, vm| {
        for param_type in signature.params.iter().rev() {
            match param_type {
                RustType::Int => args_i32.push(task.ram.pop_i32()),
                RustType::String => { /* ... */ },
                // 40+ 种固定 pattern...
            }
        }
    }
}
```

#### Phase 3A Tasks

**Task 3A.1: 扩展 `#[rust_fn]` 宏支持更多类型**

当前 `#[rust_fn]` 限制：只支持实现了 `VMConvertible` 的类型。需要扩展：

```rust
// 当前支持的类型
impl VMConvertible for i32 { ... }
impl VMConvertible for i64 { ... }
impl VMConvertible for String { ... }
impl VMConvertible for bool { ... }
impl VMConvertible for f64 { ... }

// 需要新增的支持
impl VMConvertible for Vec<i32> { ... }         // []int
impl VMConvertible for Vec<String> { ... }       // []str
impl VMConvertible for Option<String> { ... }    // Option<str>
impl VMConvertible for () { ... }                // void
```

**文件**: `crates/auto-lang/src/vm/ffi/convert.rs`

**Task 3A.2: 支持多名称注册**

当前 `#[rust_fn("File.read_text")]` 只注册一个名字。很多函数需要别名（`auto.file.read_text`, `auto.fs.read_text`, `auto.fs.read`）。

扩展宏支持多名称：
```rust
#[auto_macros::rust_fn("File.read_text", "auto.fs.read_text", "auto.fs.read")]
pub fn shim_file_read_text(path: String) -> Result<String, String> { ... }
```

宏为每个名称生成一个 `inventory::submit!`。

**文件**: `crates/auto-macros/src/lib.rs`

**Task 3A.3: 迁移 File 系列 shim（手动 → `#[rust_fn]`）**

逐个迁移 `crates/auto-lang/src/vm/ffi/stdlib.rs` 中的 File 函数：

```rust
// 迁移前: 15 行 + 手动注册
fn shim_file_read_text_vm(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> { ... }

// 迁移后: 3 行，自动注册
#[auto_macros::rust_fn("File.read_text", "auto.fs.read_text", "auto.fs.read")]
pub fn shim_file_read_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("File.read_text failed: {}", e))
}
```

优先迁移函数列表（按使用频率）：
1. File: read_text, write_text, exists, delete, create_dir
2. Env: get, set, remove
3. Time: now_ms, now_sec, sleep_ms
4. Process: exit, args, current_dir
5. Math: abs, min, max, sqrt, floor, ceil
6. JSON: encode, decode, parse, prettify
7. URL: encode, decode, parse
8. Net/HTTP: tcp_bind, tcp_connect, http_server

**Task 3A.4: 清理手动 ID 常量**

迁移完成后，`NATIVE_FILE_READ_TEXT: u16 = 1000` 等手动 ID 常量不再需要。保留 ID range 注释作为文档，但不再通过 ID 注册。

少数无法迁移的函数（如 `str.find` 的 nanbox 兼容 shim）保留手动实现，作为明确标注的例外。

**Task 3A.5: 清理 `register_stdlib_ffi()` 函数**

迁移完成后，`register_stdlib_ffi()` 中的手动 `register_shim_by_name` 调用大幅减少。该函数只保留无法自动注册的例外 shim。

**验收标准**:
- `#[rust_fn]` 覆盖的 shim ≥ 80%
- 手动 shim < 10 个（仅 nanbox 兼容、特殊逻辑等）
- 所有现有 VM 测试通过

### Phase 3B: 动态 Marshal 自动生成

**目标**: cdylib FFI 的 marshal 不再需要 40 种硬编码 pattern，从函数签名自动生成。

#### 当前问题

`ffi.rs` 中的 `create_rust_shim_lazy` 有 40+ 个 `match` arm 处理不同签名组合：
```rust
match (signature.params.as_slice(), &signature.returns) {
    (&[RustType::String], RustType::String) => { /* string→string */ },
    (&[], RustType::Long) => { /* ()→i64 */ },
    (&[RustType::Long, RustType::Long], RustType::Long) => { /* i64,i64→i64 */ },
    (&[RustType::String], RustType::Long) => { /* str→i64 */ },
    // ... 36 more patterns
}
```

每新增一种签名组合，就需要新增一个 arm。

#### 改进方案：参数逐个 marshal + 返回值统一处理

```rust
fn create_rust_shim_lazy(&self, signature: RustSignature) -> impl Fn(...) {
    move |task, vm| {
        // 1. 按 signature 逐个 pop 参数，构建 args buffer
        let mut args = Vec::new();
        for param_type in signature.params.iter().rev() {
            let arg = param_type.pop_from_vm_stack(task, vm)?;
            args.push(arg);
        }
        args.reverse();

        // 2. 通过通用 ABI 调用 cdylib 函数
        //    使用 libloading 的 Symbol + libc ABI
        let result = unsafe { call_cdylib_function(&library, &exported_name, &args, &signature)? };

        // 3. 按返回类型 push 到 VM 栈
        signature.returns.push_to_vm_stack(result, task, vm)?;
        Ok(())
    }
}
```

核心变化：
- **参数 marshaling**: `RustType` 实现 `pop_from_vm_stack` / `push_to_vm_stack` 方法，每个类型自己知道怎么和 VM 栈交互
- **返回值 marshaling**: 同上
- **函数调用**: 通用 ABI 调用层（不再需要 40 个 match arm）

**文件**: `crates/auto-lang/src/ffi.rs`, `crates/auto-lang/src/vm/ffi/convert.rs`

**验收标准**:
- `create_rust_shim_lazy` 不再有签名 pattern 的 match arm
- 新增签名类型不需要修改 marshal 代码
- 现有 cdylib 测试（serde_json, rand::random）仍然通过

### Phase 3C: 签名自动推断

**目标**: `dep any_crate` + `use.rust any_crate::any_func` 零配置可用。

**状态**:
- Phase 3C-v1 (硬编码表扩展): ✅ COMPLETE — `known_signature()` 从 2 条扩展到 15 条，覆盖 7 个 crate
- Phase 3C-v2 (syn AST 自动推断): 📋 PLANNED — 用 syn 解析替代硬编码表

#### Phase 3C-v2: syn AST 签名自动推断（方案 Y）

**方案 Y**: 编译时用 `syn` AST 扫描 `~/.cargo/registry/src/` 下的 crate 源码，提取 `pub fn` 签名。签名信息编码在 wrapper cdylib 的导出函数名中（如 `auto_from_str_s_s`），运行时从函数名解码，无需额外缓存文件或 `known_signature()` 查询。

##### 函数名编码规则（sig_code）

导出函数名格式：`auto_{func_name}_{sig_code}`

sig_code 每个字符代表一个类型：
- `v` = Void, `i` = i32(Int), `l` = i64(Long), `f` = f64(Double)
- `b` = bool(Bool), `s` = String(CString), `p` = Pointer
- `_` 分隔参数和返回值

示例：
- `auto_random__l` — () → i64
- `auto_from_str_s_s` — (String) → String
- `auto_gen_range_ll_l` — (i64, i64) → i64
- `auto_year_s_i` — (String) → i32

##### 编译时流程

```
dep serde_json + use.rust serde_json::{from_str, to_string}
  ↓
compile.rs: resolve_deps() 收集 imports
  ↓
sandbox.rs: compile_dep()
  ↓ [NEW] syn_scan("serde_json") → 扫描 ~/.cargo/registry/src/serde_json-*/src/lib.rs
  ↓ 提取 pub fn from_str(s: &str) -> Result<Value> → 签名 (String) → String
  ↓ [NEW] 生成 auto_from_str_s_s 而不是 auto_from_str
  ↓ 编译 wrapper → .dll
```

##### 运行时流程

```
init_rust_ffi()
  ↓ 加载 DLL，调用 auto__sig_manifest() 获取签名表
  ↓ {"from_str":"s_s","to_string":"s_s"}
  ↓ 解码 sig_code → RustSignature
  ↓ register_function("serde_json", "from_str", decoded_signature)
  ↓ 不再需要 known_signature() 查询
```

注：由于 `libloading` 不支持枚举导出符号，采用 manifest 方案——在 wrapper `lib.rs` 末尾生成 `auto__sig_manifest()` 函数，返回 JSON 签名表。运行时加载后先调用此函数获取签名。

##### 实施步骤

| Step | 内容 | 文件 |
|------|------|------|
| 1 | 添加 `syn` 依赖 | `crates/auto-cache/Cargo.toml` |
| 2 | 创建 `sig_code.rs` — 编码/解码 + manifest 解析 | 新建 `crates/auto-cache/src/sig_code.rs` |
| 3 | 创建 `scanner.rs` — syn 源码扫描器 | 新建 `crates/auto-cache/src/scanner.rs` |
| 4 | 修改 `compile_dep()` 集成 scanner + sig_code | `crates/auto-cache/src/sandbox.rs` |
| 5 | 修改 `init_rust_ffi()` 使用 manifest 解码 | `crates/auto-lang/src/lib.rs` |
| 6 | 保留 `known_signature()` 作为 fallback | `crates/auto-lang/src/ffi.rs`（不改） |

##### syn 类型映射

| Rust 类型 | ShimType | sig_code |
|-----------|----------|----------|
| &str / String / &String | CString | `s` |
| i32 / u32 | I32 | `i` |
| i64 / u64 | I64 | `l` |
| f64 | F64 | `f` |
| bool | Bool | `b` |
| () | Void | `v` |
| 其他（保守回退） | CString | `s` |

##### 验收标准
- `dep url` + `use.rust url::Url::{parse}` 无需手动配置签名即可调用
- syn 扫描失败时 graceful fallback 到 `known_signature()` 硬编码表
- 签名编码在函数名中，无需额外缓存文件
- 现有 12 个 FFI dual-test 全部通过

### Phase 3D: FFI 测试覆盖

**目标**: 系统性测试 FFI 各层，与 Plan 266 conformance 测试集成。

#### 测试矩阵

| FFI 模式 | 类型 | VM 测试 | a2r 测试 | 对偶测试 |
|---|---|---|---|---|
| `#[rust_fn]` 内置 shim | File I/O | ✅ | ❌ | Phase 3D |
| `#[rust_fn]` 内置 shim | Math | ✅ | ❌ | Phase 3D |
| `#[rust_fn]` 内置 shim | JSON | ✅ | ❌ | Phase 3D |
| Opaque handle | Regex | ✅ | ✅ | Phase 3D |
| Opaque handle | Url | ✅ | ✅ | Phase 3D |
| Opaque handle | chrono | ✅ | ❌ | Phase 3D |
| cdylib FFI | string→string | ✅ | ❌ | Phase 3D |
| cdylib FFI | primitive | ✅ | ❌ | Phase 3D |
| `#macro` 调用 | log | ✅ | ✅ | ✅ |
| 自动签名推断 | 任意 crate | ❌ | ❌ | Phase 3D |

#### Task 3D.1: FFI 对偶测试基础设施

在 Plan 266 的对偶测试框架中增加 FFI 支持：
- AutoVM 路径：通过 native shim / cdylib 调用
- a2r 路径：转译为 `use serde_json::*`，直接 Rust 调用
- 比较两者的输出

#### Task 3D.2: 核心crate FFI 覆盖测试

为每个已支持的 crate 编写对偶测试：
1. serde_json: from_str, to_string, prettify
2. regex: new, is_match, replace_all
3. url: parse, scheme, host_str
4. chrono: Local.now, year, format
5. base64/hex: encode, decode

**验收标准**:
- 每个 FFI 模式至少 1 个对偶测试
- 核心crate 对偶测试覆盖率 100%

### Phase 3 实施顺序

```
Phase 3A: Shim 自动化 (技术债清理)
  ├── 3A.1: 扩展 VMConvertible 类型
  ├── 3A.2: #[rust_fn] 多名称注册
  ├── 3A.3: 迁移手动 shim → #[rust_fn]（分批）
  ├── 3A.4: 清理手动 ID 常量
  └── 3A.5: 清理 register_stdlib_ffi()

Phase 3B: 动态 Marshal 自动化
  ├── 3B.1: RustType 统一 pop/push 接口
  ├── 3B.2: 通用 cdylib 调用层
  └── 3B.3: 清理 40+ pattern match

Phase 3C: 签名自动推断（方案 Y: syn AST + sig_code）
  ├── 3C-v1: 扩展 known_signature() 硬编码表（✅ 已完成）
  ├── 3C-v2.1: 添加 syn 依赖 + sig_code 编解码
  ├── 3C-v2.2: syn 源码扫描器 (scanner.rs)
  ├── 3C-v2.3: compile_dep() 集成 scanner + sig_code
  ├── 3C-v2.4: init_rust_ffi() 使用 manifest 解码
  └── 3C-v2.5: 验证 + fallback 测试

Phase 3D: FFI 测试覆盖
  ├── 3D.1: FFI 对偶测试基础设施
  └── 3D.2: 核心 crate 对偶测试

建议执行顺序: 3A ✅ → 3D.1 ✅ → 3B ✅ → 3C-v1 ✅ → 3D.2 ✅ → 3C-v2 ✅
```

### Phase 3 成功标准

- [x] `#[rust_fn]` 覆盖 ≥ 80% 的内置 shim
- [x] 手动 shim < 10 个
- [x] cdylib marshal 不再有硬编码 pattern
- [x] `dep any_crate` + `use.rust` 无需手动配置签名（Phase 3C）
- [x] FFI 对偶测试覆盖所有已支持的 crate
- [ ] 现有所有 VM 测试和 a2r 测试通过（104 个 a2r 测试失败，非 FFI 原因）

---

## Phase 4: 技术债清理 + 功能补全

**日期**: 2026-05-26
**状态**: 🔨 IN PROGRESS
**前置条件**: Phase 3 完成
**目标**: 清理 Phase 1-3 遗留的临时方案和硬编码，补全缺失的基础功能。

### Phase 4 任务列表（按优先级排序）

#### Task 4.1: 清理 serde_json 硬编码特殊处理

**复杂度**: 中
**文件**: `crates/auto-cache/src/sandbox.rs`（第 565-612 行）

**问题**: `compile_dep()` 中有两个 `if crate_name == "serde_json"` 硬编码块，为 `from_str` 和 `to_string` 手写 body_override。syn scanner 已经能正确推断 `serde_json::from_str` 的签名为 `CString→CString`，通用的 `generate_shim()` 可以生成等效代码。

**方案**:
1. 删除两个 `if crate_name == "serde_json"` 分支
2. 在 `generate_shim()` 中添加通用的 Result 错误处理逻辑：检测 syn 扫描结果中函数返回 `Result` 类型时，自动生成 `match` 分支将 `Err` 转为 `"ERROR: {}"` 字符串
3. 或者更简单：让通用 shim 始终对返回值调用 `.to_string()`，这对 `Result<Value, Error>` 已经工作正常

**验证**: 删除硬编码后，FFI dual test 003_json_encode_parse 仍然通过。

#### Task 4.2: 修复 Math.min/max i32 签名不匹配

**复杂度**: 低
**文件**: `crates/auto-lang/src/vm/ffi/stdlib.rs`（第 1133 行附近）

**问题**: `shim_math_min` 和 `shim_math_max` 的签名是 `(i64, i64) -> i64`，但 AutoVM 默认整数为 i32，codegen 推入 `push_i32` 而 `VMConvertible<i64>::pop_from_stack` 调用 `pop_i64`（占 2 个栈槽），导致栈对齐错误。

**方案**: 将签名改为 `(i32, i32) -> i32`，与 VM 默认整数宽度一致：
```rust
pub fn shim_math_min(a: i32, b: i32) -> i32 { a.min(b) }
pub fn shim_math_max(a: i32, b: i32) -> i32 { a.max(b) }
pub fn shim_math_abs(a: i32) -> i32 { a.abs() }
```

**验证**: FFI dual test 004_math_abs 通过；`Math.min(3, 5)` 返回 3。

#### Task 4.3: 改进 manifest 字符串嵌入方式

**复杂度**: 低
**文件**: `crates/auto-cache/src/sandbox.rs`（第 620-623 行）

**问题**: 当前用 `replace('"', "\\\"")` 手动转义 JSON 嵌入到 Rust 字符串字面量中。sig_code 只包含 `[a-z_]` 字符，转义本不需要，但方法本身是 hack。

**方案**: 使用 Rust raw string `r#"..."#` 替代手动转义：
```rust
lib_rs.push_str("#[no_mangle]\npub extern \"C\" fn auto__sig_manifest() -> *const c_char {\n");
lib_rs.push_str(&format!("    let s = CString::new(r#\"{}\"#).unwrap();\n", manifest_json));
lib_rs.push_str("    s.into_raw() as *const c_char\n}\n");
```

#### Task 4.4: 解决 auto_cache 命名冲突

**复杂度**: 低
**文件**: 5 个文件

**问题**: `crates/auto-lang/src/auto_cache.rs`（本地模块缓存 `ModuleCache`）与外部 crate `auto_cache`（提供 `Sandbox`、`CrateRegistry`）同名，导致在 `lib.rs` 中 `auto_cache::sig_code` 解析到本地模块而非外部 crate。

**方案**: 将本地模块从 `auto_cache` 重命名为 `module_cache`：
- `crates/auto-lang/src/auto_cache.rs` → `crates/auto-lang/src/module_cache.rs`
- `lib.rs`: `pub mod auto_cache` → `pub mod module_cache`
- `compile.rs`: `use crate::auto_cache::` → `use crate::module_cache::`

**影响范围**: 3 个文件、~5 处引用。

#### Task 4.5: 清理 ffi.rs 中过时的 TODO 和死代码

**复杂度**: 低
**文件**: `crates/auto-lang/src/ffi.rs`

**问题**: `CFfiBridge` 中的 `create_rust_shim` 和 `register_rust_function` 是死代码（真正的 Rust FFI 走 `RustFfiBridge`）。4 个 TODO 注释中，1 个已过时。

**方案**:
1. 删除 `CFfiBridge::create_rust_shim` 和 `CFfiBridge::register_rust_function`（约 100 行死代码）
2. 删除已过时的 TODO 注释（line 277 的 `TODO: Call Rust function via FFI`）
3. 给 C FFI 的 3 个存留 TODO 添加 `// TODO(Plan-212-Phase-5): C FFI support` 标记

#### Task 4.6: dep feature flags 支持

**复杂度**: 低-中
**文件**: `crates/auto-lang/src/compile.rs`、`crates/auto-cache/src/sandbox.rs`

**问题**: `DepStmt.features` 字段已解析但从未使用。`resolve_deps()` 记录了 features（line 638）但 `compile_dep()` 的 Cargo.toml 模板不传递 features。

**方案**:
1. `FunctionShim` 添加 `features: Vec<String>` 字段（或作为 `compile_dep()` 的参数）
2. 修改 Cargo.toml 模板：当有 features 时生成 `{ version = "1", features = ["derive"] }` 格式
3. 从 `resolve_deps()` 传递 features 到 `compile_dep()`

**验证**: `dep serde(features: ["derive"])` 生成的 Cargo.toml 包含正确的 features 行。

#### Task 4.7: Sandbox 缓存 GC

**复杂度**: 中
**文件**: `crates/auto-cache/src/sandbox.rs`（新增方法）

**问题**: `~/.auto/sandbox/crates/` 和 `~/.auto/sandbox/builds/` 无自动清理。签名变更时生成新版本 DLL 但不删除旧版本。

**方案**: 在 `Sandbox` 中添加 `garbage_collect(max_size_mb: u64)` 方法：
1. 遍历 `crates/` 目录，统计文件大小和修改时间
2. 超过阈值时按 LRU（最久未用）删除旧文件
3. 同时清理 `builds/` 中无对应 DLL 的残留构建目录
4. 在 `compile_dep()` 开始时条件性触发（如随机 10% 概率，避免每次启动都扫描）

### Phase 4 实施顺序

```
建议执行顺序（按 ROI 排序）:

4.2 Math.min 签名修复         ← 影响正确性，立即修复
4.5 清理死代码和 TODO         ← 减少混淆，改善代码质量
4.4 auto_cache 命名冲突       ← 消除开发时的困惑
4.3 manifest 字符串改进       ← 消除 hack
4.1 serde_json 硬编码清理     ← 减少 per-crate 特殊处理
4.6 dep feature flags         ← 新功能，解锁更多 crate 用法
4.7 Sandbox 缓存 GC           ← 改善长期使用体验
```

### Phase 4 成功标准

- [ ] `Math.min(3, 5)` 在 AutoVM 中返回 3（非栈错乱）
- [ ] `compile_dep()` 中无 `if crate_name == "..."` 硬编码分支
- [ ] ffi.rs 中无死代码
- [ ] `lib.rs` 中 `auto_cache` 引用外部 crate 而非本地模块
- [ ] `dep serde(features: ["derive"])` 生成正确的 Cargo.toml
