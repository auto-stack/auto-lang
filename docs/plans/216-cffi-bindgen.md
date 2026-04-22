# Plan 213: Auto C FFI — 基于 Bindgen 的自动签名提取与绑定

## Context

Auto 语言已有 `use c <stdio.h>` 语法，但当前仅在 a2c 转译器中用于生成 `#include` 指令。用户调用 C 函数仍需手写 `fn.c sqrt(x double) double` 声明，且 AutoVM 完全无法调用 C 函数。

本计划目标：**用 bindgen 自动从 C 头文件提取函数签名，同时服务于 AutoVM 运行时 FFI 和 a2c 转译器**，消除手写 `fn.c` 的需求。

**关键设计决策：** 标准头文件 (stdio/string/math/stdlib/time) 的 JSON manifest 由开发者在构建时用 `auto-bindgen` 预生成，随 auto-lang 源码发布。用户无需安装 libclang。MVP 不支持自定义 C 头文件。

## Architecture

```
┌─────────────────────────────────────────────┐
│  Build Time: auto-bindgen 工具              │
│                                             │
│  <stdio.h> ──→ bindgen ──→ JSON Manifest   │
│  <math.h>   ──→ bindgen ──→ JSON Manifest   │
│  <string.h> ──→ bindgen ──→ JSON Manifest   │
└───────────┬─────────────────┬───────────────┘
            │                 │
    ┌───────▼───────┐  ┌──────▼────────────────┐
    │  a2c 转译器    │  │  AutoVM 运行时          │
    │               │  │                       │
    │  use c <...>  │  │  use c <string.h>     │
    │  自动识别函数  │  │  libloading 加载 libc  │
    │  无需 fn.c    │  │  动态 marshal 调用     │
    └───────────────┘  └───────────────────────┘
```

三层组件：
1. **`auto-bindgen` crate** — 构建时工具，用 bindgen 提取签名，输出 JSON manifest
2. **a2c 集成** — 转译器读 manifest，自动将 C 函数识别为 extern 声明
3. **AutoVM C-FFI** — 运行时用 libloading 加载 C 库，按 manifest 的签名信息 marshal 参数并调用

## Phase 1: Bindgen 签名提取工具

### Task 1: 创建 `auto-bindgen` crate

**Files:**
- Create: `crates/auto-bindgen/Cargo.toml`
- Create: `crates/auto-bindgen/src/main.rs`
- Create: `crates/auto-bindgen/src/extractor.rs`
- Create: `crates/auto-bindgen/src/manifest.rs`
- Create: `crates/auto-bindgen/src/type_map.rs`

**`Cargo.toml` 依赖:**
```toml
[package]
name = "auto-bindgen"
version = "0.1.0"
edition = "2021"

[dependencies]
bindgen = "0.70"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**`manifest.rs` — JSON manifest 格式:**
```rust
#[derive(Serialize, Deserialize)]
pub struct CHeaderManifest {
    pub header: String,           // "string.h"
    pub library: String,          // 平台库名: "c" (由运行时解析为 msvcrt/libc)
    pub functions: Vec<CFunction>,
    pub structs: Vec<CStruct>,
}

#[derive(Serialize, Deserialize)]
pub struct CFunction {
    pub name: String,
    pub params: Vec<CParam>,
    pub return_type: CTypeDesc,
    pub variadic: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CParam {
    pub name: String,
    pub type_: CTypeDesc,
}

#[derive(Serialize, Deserialize)]
pub enum CTypeDesc {
    Void,
    Bool,
    Int, UInt, Long, ULong,
    Int8, Int16, Int32, Int64,
    UInt8, UInt16, UInt32, UInt64,
    Float, Double,
    Size,       // size_t
    Char,       // C char
    CStr,       // const char* / char*
    Ptr,        // void*
    TypedPtr(Box<CTypeDesc>),  // T*
    Struct { name: String, size: usize },
    Enum(String),
}
```

**`extractor.rs` — 使用 bindgen 提取:**
```rust
use bindgen::Builder;

pub fn extract_header(header_path: &str) -> Result<CHeaderManifest> {
    // 1. 用 bindgen::Builder 解析头文件
    let bindings = Builder::default()
        .header(header_path)
        .allowlist_function(".*")     // 提取所有函数
        .allowlist_type(".*")          // 提取所有类型
        .no_layout_tests(true)
        .no_derive_default(true)
        .generate()?;

    // 2. 遍历 bindgen 输出的 items，提取函数签名
    //    bindgen 生成 extern "C" { fn xxx(...); } 形式的 Rust 代码
    //    我们通过 parse_callbacks 或直接解析生成的 tokens 提取信息

    // 3. 映射 bindgen 的 Rust 类型到 CTypeDesc
    //    e.g., ::std::os::raw::c_int → CTypeDesc::Int
    //          *const ::std::os::raw::c_char → CTypeDesc::CStr

    // 4. 输出 CHeaderManifest
}
```

**`type_map.rs` — bindgen Rust 类型 → CTypeDesc 映射:**
```rust
pub fn map_bindgen_type(rust_type: &str) -> CTypeDesc {
    match rust_type {
        "::std::os::raw::c_int" => CTypeDesc::Int,
        "::std::os::raw::c_uint" => CTypeDesc::UInt,
        "::std::os::raw::c_long" => CTypeDesc::Long,
        "::std::os::raw::c_ulong" => CTypeDesc::ULong,
        "::std::os::raw::c_float" => CTypeDesc::Float,
        "::std::os::raw::c_double" => CTypeDesc::Double,
        "::std::os::raw::c_char" => CTypeDesc::Char,
        "::std::os::raw::c_uchar" => CTypeDesc::UInt8,
        "*const ::std::os::raw::c_char" => CTypeDesc::CStr,
        "*mut ::std::os::raw::c_char" => CTypeDesc::CStr,
        "*mut ::std::ffi::c_void" => CTypeDesc::Ptr,
        // ... etc
    }
}
```

**CLI 用法:**
```bash
auto-bindgen --header <stdio.h> --output c_bindings/stdio.json
auto-bindgen --header <math.h> --output c_bindings/math.json
auto-bindgen --header <string.h> --output c_bindings/string.json
auto-bindgen --header <stdlib.h> --output c_bindings/stdlib.json
auto-bindgen --header <time.h> --output c_bindings/time.json
```

**Verify:** `cargo run -p auto-bindgen -- --header <string.h> --output string.json`，检查 JSON 输出包含 strlen、strcmp 等函数签名。

---

### Task 2: 生成标准 C 库 manifests

**Files:**
- Create: `crates/auto-lang/src/vm/ffi/c_bindings/stdio.json`
- Create: `crates/auto-lang/src/vm/ffi/c_bindings/string.json`
- Create: `crates/auto-lang/src/vm/ffi/c_bindings/math.json`
- Create: `crates/auto-lang/src/vm/ffi/c_bindings/stdlib.json`
- Create: `crates/auto-lang/src/vm/ffi/c_bindings/time.json`

运行 auto-bindgen 对 5 个标准头文件生成 manifest。这些 manifest 同时用于 AutoVM 和 a2c。

**Verify:** 每个 JSON 文件可被 `serde_json::from_str::<CHeaderManifest>()` 解析。

---

### Task 3: 提交 Phase 1

```
feat(cffi): add auto-bindgen tool for C header signature extraction (Plan 213 Phase 1)
```

---

## Phase 2: AutoVM C-FFI 运行时

### Task 4: 创建 C-FFI 运行时模块

**Files:**
- Create: `crates/auto-lang/src/vm/ffi/c_ffi.rs` — 运行时 C-FFI 加载器
- Modify: `crates/auto-lang/src/vm/ffi/mod.rs` — 添加 `mod c_ffi`

**核心结构:**
```rust
use libloading::Library;
use std::collections::HashMap;
use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use crate::vm::engine::AutoVM;

pub struct CFfiRuntime {
    /// 已加载的 C 库
    libraries: HashMap<String, Library>,
    /// 已注册的函数: name → native_id
    functions: HashMap<String, u16>,
    /// 下一个动态 native ID (5000 起)
    next_native_id: u16,
}

impl CFfiRuntime {
    pub fn new() -> Self { ... }

    /// 加载一个 C 头文件对应的绑定
    pub fn load_header(
        &mut self,
        header: &str,
        manifest: &CHeaderManifest,
        natives: &mut NativeInterface,
        registry: &mut AutoVMNativeRegistry,
    ) -> Result<(), String> {
        // 1. 解析平台 C 库名
        let lib_name = Self::resolve_system_lib();
        let lib = Library::new(lib_name)?;

        // 2. 遍历 manifest 中的函数
        for func in &manifest.functions {
            if func.variadic { continue; } // MVP 跳过变参函数

            // 3. 获取函数符号
            let symbol = unsafe { lib.get::<*const ()>(func.name.as_bytes())? };

            // 4. 创建 marshal shim 并注册
            let native_id = self.next_native_id;
            self.next_native_id += 1;

            let shim = create_c_shim(func, *symbol as usize)?;
            natives.register_dynamic(native_id, shim);
            registry.register_with_id(&func.name, native_id);

            self.functions.insert(func.name.clone(), native_id);
        }

        self.libraries.insert(header.to_string(), lib);
        Ok(())
    }

    fn resolve_system_lib() -> &'static str {
        if cfg!(target_os = "windows") { "ucrtbase" }
        else if cfg!(target_os = "linux") { "libc.so.6" }
        else if cfg!(target_os = "macos") { "libSystem" }
        else { "c" }
    }
}
```

**Verify:** `cargo build -p auto-lang`

---

### Task 5: 实现 C 类型 Marshal

**Files:**
- Modify: `crates/auto-lang/src/vm/ffi/c_ffi.rs`

**`create_c_shim` — 根据签名生成运行时 shim:**

采用与 `RustFfiBridge` 相同的 exhaustive match 模式（`ffi.rs:689-1079`），根据签名参数和返回类型的组合生成对应的 shim closure。

```rust
fn create_c_shim(
    sig: &CFunction,
    func_ptr: usize,
) -> Result<ShimFunc, String> {
    let name = sig.name.clone();
    Ok(Arc::new(move |task: &mut AutoTask, vm: &AutoVM| {
        unsafe {
            // 根据签名 match，transmute func_ptr 为具体函数类型
            match (sig.params.as_slice(), &sig.return_type) {
                // size_t strlen(const char*)
                ([CParam { type_: CTypeDesc::CStr, .. }], CTypeDesc::Size) => {
                    let f: unsafe fn(*const c_char) -> usize = std::mem::transmute(func_ptr);
                    let s = pop_cstr_from_vm(task, vm)?;
                    let result = f(s.as_ptr());
                    task.ram.push_i64(result as i64);
                }
                // int abs(int)
                ([CParam { type_: CTypeDesc::Int, .. }], CTypeDesc::Int) => {
                    let f: unsafe fn(i32) -> i32 = std::mem::transmute(func_ptr);
                    let arg = task.ram.pop_i32();
                    let result = f(arg);
                    task.ram.push_i32(result);
                }
                // double sqrt(double)
                ([CParam { type_: CTypeDesc::Double, .. }], CTypeDesc::Double) => {
                    let f: unsafe fn(f64) -> f64 = std::mem::transmute(func_ptr);
                    let arg = pop_f64_from_vm(task, vm)?;
                    let result = f(arg);
                    push_f64_to_vm(result, task, vm)?;
                }
                // double cos(double)
                // 同 sqrt 模式，由代码生成器批量生成
                // ... 更多签名组合
                _ => return Err(VMError::RuntimeError(
                    format!("Unsupported C FFI signature for {}", name)
                )),
            }
        }
        Ok(())
    }))
}
```

**支持的 MVP 签名组合（覆盖常用 C 函数）:**

| 签名模式 | 示例函数 |
|---------|---------|
| `(int) → int` | abs, atoi, isalpha, isdigit |
| `(double) → double` | sqrt, sin, cos, ceil, floor |
| `(const char*) → size_t` | strlen |
| `(const char*, const char*) → int` | strcmp, strncmp |
| `(const char*, const char*) → char*` | strcpy, strcat |
| `(int) → void` | exit |
| `(const char*) → int` | puts, atoi (alt) |
| `(size_t) → void*` | malloc |
| `(void*) → void` | free |
| `(void) → int` | rand |
| `(void) → double` | drand48 |
| `(int, int) → int` | max, min (自定义) |
| `(double, double) → double` | fmax, fmin, pow |
| 0-3 参数的其他标量组合 | ... |

对于不在预编译 match 中的签名，返回明确的运行时错误。

**Verify:** `cargo build -p auto-lang`

---

### Task 6: 集成到 VM 初始化流程

**Files:**
- Modify: `crates/auto-lang/src/compile.rs:166-173` — `resolve_uses()` 不再跳过 C import
- Modify: `crates/auto-lang/src/vm/engine.rs` — 在 VM 初始化时加载 C 绑定
- Modify: `crates/auto-lang/src/vm/codegen.rs` — 解析 C 函数调用到 CALL_NAT

**compile.rs 修改 — 处理 `use c`:**
```rust
// 当前 (line 171): if use_stmt.is_c_import { continue; }
// 改为:
if use_stmt.is_c_import {
    // 加载 C 头文件的 manifest
    let header = use_stmt.c_header.as_ref().unwrap();
    self.c_headers.insert(header.clone());
    continue;
}
```

**engine.rs 修改 — VM 初始化时加载 C 函数:**
```rust
// 在 AutoVM 初始化后（类似 register_stdlib_ffi 的位置）
pub fn init_cffi(&mut self, headers: &HashSet<String>) -> Result<(), String> {
    for header in headers {
        let manifest = CHeaderManifest::load(header)?;
        self.cffi.load_header(header, &manifest,
            &mut self.native_interface, &mut registry)?;
    }
    Ok(())
}
```

**codegen.rs 修改 — C 函数名解析:**
```rust
// 在函数调用 codegen 中，先检查 BIGVM_NATIVES registry
// 如果函数名已注册为 C FFI 函数（有 native_id），则 emit CALL_NAT
// 否则按常规函数调用处理
```

**Verify:** `cargo build -p auto-lang`

---

### Task 7: 添加 VM 端到端测试

**Files:**
- Create: `crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at`
- Create: `crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.expected.out`
- Create: `crates/auto-lang/test/vm/17_cffi/002_math/math.at`
- Create: `crates/auto-lang/test/vm/17_cffi/002_math/math.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs` — 注册测试

**strlen.at:**
```auto
use c <string.h>

fn main() {
    let len = strlen(c"hello")
    print(len)
}
```

**strlen.expected.out:**
```
5
```

**math.at:**
```auto
use c <math.h>

fn main() {
    let x = sqrt(4.0)
    let y = abs(-10)
    print(x)
    print(y)
}
```

**math.expected.out:**
```
2
10
```

**Verify:** `cargo test -p auto-lang test_17_cffi`

---

### Task 8: 提交 Phase 2

```
feat(vm): add C FFI runtime with libloading (Plan 213 Phase 2)
```

---

## Phase 3: a2c 转译器自动绑定

### Task 9: a2c 读取 manifest 自动解析 C 函数

**Files:**
- Modify: `crates/auto-lang/src/trans/c.rs` — transpiler 读 manifest，自动将 C 函数识别为 extern

**当前问题:** a2c 要求用户手写 `fn.c sqrt(x double) double` 来声明 C 函数。有了 manifest 后，`use c <math.h>` 应该自动让 `sqrt` 可用。

**修改逻辑:**

1. 在 `Transpiler::new()` 中，当遇到 `UseKind::C` 时，加载对应的 manifest
2. 将 manifest 中的函数名注册到 transpiler 的符号表中，标记为 `CFunction`
3. 在 codegen 阶段，如果遇到对 manifest 中函数的调用：
   - 不生成函数定义（因为 C 库已有实现）
   - 在 header 中生成对应的 `extern` 声明（如果需要）
   - 直接在代码中调用函数名

**关键修改点:**
- `c.rs` `use_stmt()` (line 1309-1329): 加载 manifest
- `c.rs` `call()` 函数调用处理: 检查是否为 manifest 中的 C 函数
- `c.rs` `fn_decl()` (line 2013-2026): 对 manifest 函数自动标记为 CFunction 语义

**Verify:**
```bash
cargo test -p auto-lang -- trans
# 测试: 不写 fn.c 声明，只写 use c <math.h>，直接调用 sqrt()
```

---

### Task 10: 添加 a2c 自动绑定测试

**Files:**
- Create: `crates/auto-lang/test/a2c/200_auto_cffi_math/input.at`
- Create: `crates/auto-lang/test/a2c/200_auto_cffi_math/input.expected.c`
- Create: `crates/auto-lang/test/a2c/200_auto_cffi_math/input.expected.h`
- Modify: `crates/auto-lang/src/tests/a2c_tests.rs` — 注册测试

**input.at (无需手写 fn.c):**
```auto
use c <math.h>

fn main() {
    let x double = sqrt(4.0)
    let y = abs(-10)
}
```

**input.expected.h:**
```c
#pragma once

#include <math.h>
```

**input.expected.c:**
```c
#include "input.h"

int main(void) {
    double x = sqrt(4.0);
    int y = abs(-10);
    return 0;
}
```

**Verify:** `cargo test -p auto-lang test_200_auto_cffi_math`

---

### Task 11: 提交 Phase 3

```
feat(a2c): auto-bind C functions from header manifests (Plan 213 Phase 3)
```

---

## Phase 4: 构建集成与发布

### Task 12: 集成到 `auto build` 命令

**Files:**
- Modify: `crates/auto-lang/src/main.rs` 或 CLI 入口

在 `auto build` / `auto run` 时：
1. 扫描 `.at` 文件中的 `use c <header>` 语句
2. 检查 `c_bindings/` 目录是否有对应的 manifest
3. 如果没有，调用 `auto-bindgen` 生成
4. 将 manifest 路径传递给编译器/VM

**Verify:** `auto run examples/cffi_demo.at` 能正确调用 C 函数

---

### Task 13: 提交 Phase 4

```
feat(cli): integrate auto-bindgen into build pipeline (Plan 213 Phase 4)
```

---

## Dependency Graph

```
Phase 1 (auto-bindgen + manifests) ──> Phase 2 (AutoVM C-FFI) ──> Phase 4 (build 集成)
                                  ──> Phase 3 (a2c auto-bind)  ──/
```

Phase 2 和 Phase 3 可以并行开发（都依赖 Phase 1 的 manifest）。

## Risks

1. **变参函数 (printf)** — MVP 不支持，需要 libffi 或专门的 marshal 逻辑
2. **struct 传值** — C struct 按值传递的 ABI 因平台而异，MVP 只支持 struct 指针
3. **指针内存管理** — C 函数返回的 malloc 内存需手动 free，暂不自动管理
4. **平台差异** — Windows/Linux/macOS 的 C 库名不同，通过 `resolve_system_lib()` 处理
5. **自定义头文件** — MVP 不支持，需要用户手动添加 manifest 或后续版本支持 on-demand bindgen

## Verification

1. `cargo run -p auto-bindgen -- --header <string.h> --output /dev/stdout` — 检查 JSON 输出
2. `cargo test -p auto-lang test_17_cffi` — VM C-FFI 端到端测试
3. `cargo test -p auto-lang -- trans` — a2c 自动绑定测试
4. 手动验证: 在 Windows 上 `auto run` 一个调用 strlen 的 .at 文件
