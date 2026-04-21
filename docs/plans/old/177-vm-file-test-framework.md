# Plan 177: VM 文件测试框架 (File-Based Test Framework)

## Context

当前 AutoVM 测试全部内联在 `crates/auto-lang/src/tests/vm_tests.rs`（3048 行），难以维护。需要像 a2r_tests 一样建立基于文件的测试框架，支持 `.expected.out`（stdout）、`.expected.result`（返回值）、`.expected.error`（运行时错误）三种断言模式。

**关键问题**: `print()` 在 VM 中直接调用 `println!()`（`native.rs:388-452`），没有 stdout 捕获机制。需先添加输出缓冲区。

## 实施步骤

### Task 1: 给 AutoVM 添加 stdout 捕获能力

**修改文件**: `crates/auto-lang/src/vm/engine.rs`

在 `AutoVM` 结构体中添加可选的输出缓冲区：

```rust
// 在 AutoVM struct 中添加:
pub output_buffer: Option<Arc<RwLock<String>>>,
```

在 `AutoVM::new()` 中初始化为 `None`。

添加 `new_with_capture()` 构造函数：

```rust
pub fn new_with_capture(flash: VirtualFlash, ram_size: usize) -> (Self, Arc<RwLock<String>>) {
    let mut vm = Self::new(flash, ram_size);
    let buffer = Arc::new(RwLock::new(String::new()));
    vm.output_buffer = Some(buffer.clone());
    (vm, buffer)
}
```

### Task 2: 修改 print shim 写入缓冲区

**修改文件**: `crates/auto-lang/src/vm/native.rs`

修改 4 个 print shim（`shim_print`, `shim_print_i32`, `shim_print_f32`, `shim_print_str`）：

提取辅助函数 `vm_print(vm: &AutoVM, s: &str)` 统一处理输出：

```rust
fn vm_print(vm: &AutoVM, s: &str) {
    if let Some(ref buf) = vm.output_buffer {
        buf.write().unwrap().push_str(s);
        buf.write().unwrap().push('\n');
    } else {
        println!("{}", s);
    }
}
```

每个 print shim 调用 `vm_print()` 代替 `println!()`。

### Task 3: 添加 `run_with_capture()` 公共 API

**修改文件**: `crates/auto-lang/src/lib.rs` + `crates/auto-lang/src/execution_engine.rs`

添加 `run_with_capture()` 函数返回 `(result, stdout_output)`:

```rust
// lib.rs
pub fn run_with_capture(code: &str) -> AutoResult<(String, String)> {
    let engine = execution_engine::ExecutionEngine::get();
    execution_engine::execute_with_engine_capture(engine, code)
}

// execution_engine.rs
pub fn execute_with_engine_capture(engine: ExecutionEngine, code: &str) -> AutoResult<(String, String)> {
    match engine {
        ExecutionEngine::AutoVM => crate::run_autovm_capture(code),
        #[allow(deprecated)]
        ExecutionEngine::Evaluator => crate::run_autovm_capture(code),
    }
}
```

在 `lib.rs` 中添加 `run_autovm_capture()` 函数，类似 `execute_autovm()` 但使用 `AutoVM::new_with_capture()` 并返回 stdout。

### Task 4: 创建 `test_vm()` 测试框架

**新建文件**: `crates/auto-lang/src/tests/vm_file_tests.rs`

```rust
fn test_vm(case: &str) -> AutoResult<()> {
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let name = dir_name.splitn(2, '_').nth(1).unwrap_or(dir_name);

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = read_to_string(d.join(format!("test/vm/{}/{}.at", case, name)))?;

    // 检查 .expected.error
    let err_path = d.join(format!("test/vm/{}/{}.expected.error", case, name));
    if err_path.is_file() {
        let result = run(&src);
        assert!(result.is_err(), "Expected error but got: {:?}", result);
        return Ok(());
    }

    // 正常执行 (带 stdout 捕获)
    let (result, stdout) = run_with_capture(&src)?;

    // 检查 .expected.out (stdout 输出)
    let out_path = d.join(format!("test/vm/{}/{}.expected.out", case, name));
    if out_path.is_file() {
        let expected_out = read_to_string(&out_path)?;
        if stdout != expected_out {
            std::fs::write(d.join(format!("test/vm/{}/{}.wrong.out", case, name)), &stdout)?;
        }
        assert_eq!(stdout, expected_out);
    }

    // 检查 .expected.result (返回值)
    let res_path = d.join(format!("test/vm/{}/{}.expected.result", case, name));
    if res_path.is_file() {
        let expected_res = read_to_string(&res_path)?;
        if result != expected_res {
            std::fs::write(d.join(format!("test/vm/{}/{}.wrong.result", case, name)), &result)?;
        }
        assert_eq!(result, expected_res);
    }

    Ok(())
}
```

### Task 5: 注册模块 + 创建示例测试

**修改文件**: `crates/auto-lang/src/tests.rs`

添加 `mod vm_file_tests;`

**创建测试用例**:

```
crates/auto-lang/test/vm/
├── 01_basics/
│   ├── 001_hello/
│   │   ├── hello.at
│   │   └── hello.expected.out
│   ├── 002_arithmetic/
│   │   ├── arithmetic.at
│   │   └── arithmetic.expected.result
│   └── 003_str_upper/
│       ├── str_upper.at
│       └── str_upper.expected.out
```

**001_hello/hello.at**:
```auto
fn main() {
    print("hello")
}
```
**001_hello/hello.expected.out**: `hello\n`

**002_arithmetic/arithmetic.at**: `1 + 2`
**002_arithmetic/arithmetic.expected.result**: `3`

**003_str_upper/str_upper.at**:
```auto
fn my_upper(s str) str {
    s.upper()
}

fn main() {
    print(my_upper("hello"))
}
```
**003_str_upper/str_upper.expected.out**: `HELLO\n`

### Task 6: 验证

1. `cargo build` 编译通过
2. `cargo test -p auto-lang -- vm_file_tests` 新测试通过
3. `cargo test -p auto-lang` 全部测试通过（无回归）

## 关键文件

| 文件 | 操作 |
|------|------|
| `crates/auto-lang/src/vm/engine.rs` | 修改: 添加 `output_buffer` 字段和 `new_with_capture()` |
| `crates/auto-lang/src/vm/native.rs` | 修改: print shim 检查 output_buffer |
| `crates/auto-lang/src/lib.rs` | 修改: 添加 `run_with_capture()` 和 `run_autovm_capture()` |
| `crates/auto-lang/src/execution_engine.rs` | 修改: 添加 `execute_with_engine_capture()` |
| `crates/auto-lang/src/tests.rs` | 修改: 添加 `mod vm_file_tests;` |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | 新建: `test_vm()` 框架 + 测试函数 |
| `crates/auto-lang/test/vm/01_basics/*` | 新建: 示例测试用例 |
