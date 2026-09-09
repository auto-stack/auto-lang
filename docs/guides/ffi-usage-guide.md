# FFI Usage Guide

The Foreign Function Interface (FFI) layer enables AutoVM bytecode to call functions from C-transpiled and Rust-transpiled modules.

## Overview

**Plan 081 Phase 5**: The FFI layer provides a bridge between AutoVM bytecode and native code, enabling mixed-mode projects where different parts of your application can use different execution modes.

### Architecture

```
┌─────────────────────┐
│ AutoVM Bytecode     │
│ (CALL_NAT opcode)   │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ Native Shim         │  ← Registered in CFfiBridge
│ (Type Marshaling)   │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ C/Rust Function     │  ← Loaded via libloading
│ (Native Code)       │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ Return Value        │
│ (C → AutoVM)        │
└─────────────────────┘
```

## Basic Usage

### 1. Declaring External Functions

In your AutoLang code, declare external C functions using `extern "c"`:

```auto
// myapp.at
extern "c" {
    // C function from HAL library
    fn hal_gpio_init(pin int) int;

    // C function from crypto library
    fn crypto_sha256(data str) str;
}

fn main() {
    let result = hal_gpio_init(13);
    say("GPIO initialized: " + result)
}
```

### 2. Registration (Automatic)

When you declare `extern "c"` functions, the codegen automatically:

1. Registers the function with the FFI bridge
2. Assigns a native ID (200+ for C, 100-199 for Rust)
3. Generates `CALL_NAT <native_id>` opcode

### 3. Execution Flow

```
1. AutoVM executes CALL_NAT 200
2. Looks up native shim for ID 200
3. Shim pops arguments from stack
4. Converts AutoVM values → C types
5. Calls C function via libloading
6. Converts return value → AutoVM value
7. Pushes result onto stack
```

## Type Marshaling

### Supported Types

| AutoVM Type | C Type | Rust Type | Description |
|-------------|--------|-----------|-------------|
| `int` | `int32_t` | `i32` | Signed 32-bit integer |
| `uint` | `uint32_t` | `u32` | Unsigned 32-bit integer |
| `float` | `float` | `f32` | 32-bit floating point |
| `str` | `const char*` | `&str` | Null-terminated string |
| `void` | `void` | `()` | No return value |

### Example: Simple Function

**C Code** (`hal.c`):
```c
#include "hal.h"

int hal_gpio_init(int pin) {
    // Initialize GPIO pin
    printf("Initializing GPIO pin %d\n", pin);
    return 0;  // Success
}
```

**AutoLang Declaration** (`app.at`):
```auto
extern "c" {
    fn hal_gpio_init(pin int) int;
}

fn main() {
    let result = hal_gpio_init(13);
    if result == 0 {
        say("GPIO initialized successfully")
    }
}
```

## Native Function IDs

The FFI layer reserves specific ID ranges:

| ID Range | Purpose | Examples |
|----------|---------|----------|
| 1-99 | Standard library | `print`, `list_push`, etc. |
| 100-199 | Rust FFI functions | `crypto_hash`, etc. |
| 200+ | C FFI functions | `hal_gpio_init`, etc. |

### Registering Functions Manually

You can also register functions programmatically:

```rust
use auto_lang::ffi::{CFfiBridge, CSignature, CType};
use std::path::PathBuf;

let mut bridge = CFfiBridge::new();

// Register C function
let native_id = bridge.register_c_function(
    "hal",                         // Library name
    "gpio_init",                   // Function name
    CSignature::new()
        .param(CType::Int)         // int pin
        .returns(CType::Int),      // returns int
    PathBuf::from("target/hal.dll")  // Library path
)?;

// native_id = 200 (first C function)
```

## Advanced Usage

### Multiple Return Values

C functions can return multiple values via pointer arguments:

**C Code**:
```c
int get_sensor_data(int* temp, int* humidity) {
    *temp = 25;
    *humidity = 60;
    return 0;  // Success
}
```

**AutoLang Declaration**:
```auto
extern "c" {
    // Note: Pointer arguments not yet fully supported
    fn get_sensor_data(temp_ptr int, humidity_ptr int) int;
}
```

**Status**: Pointer argument marshaling is planned for future implementation.

### Struct Marshaling

Passing structs between AutoVM and C:

**C Code**:
```c
typedef struct {
    int x;
    int y;
} Point;

int point_distance(Point* p1, Point* p2) {
    int dx = p2->x - p1->x;
    int dy = p2->y - p1->y;
    return (int)sqrt(dx*dx + dy*dy);
}
```

**AutoLang Declaration** (future):
```auto
type Point {
    x int
    y int
}

extern "c" {
    fn point_distance(p1 Point, p2 Point) int;
}
```

**Status**: Struct marshaling is planned for future implementation.

## Error Handling

### C Function Errors

C functions typically return error codes:

```auto
extern "c" {
    // Returns 0 on success, negative on error
    fn hal_spi_write(data int) int;
}

fn write_spi_safe(data int) int {
    let result = hal_spi_write(data);
    if result < 0 {
        say("SPI write failed: " + result)
    }
    result
}
```

### FFI Errors

FFI operations can fail with `VMError::RuntimeError`:

```rust
use auto_lang::ffi::register_extern_c_function;

match register_extern_c_function("hal", "gpio_init", signature, path) {
    Ok(native_id) => println!("Registered: {}", native_id),
    Err(e) => eprintln!("FFI registration failed: {:?}", e),
}
```

## Library Loading

### Automatic Loading

The FFI bridge automatically loads libraries when functions are registered:

```rust
// Library is loaded when first function is registered
bridge.register_c_function("hal", "gpio_init", ...)?;
// ↑ Loads target/hal.dll (or .so on Linux)
```

### Library Search Paths

Libraries are searched in the following order:

1. Absolute path (if provided)
2. `target/` directory (build output)
3. System library paths (`PATH`, `LD_LIBRARY_PATH`)

### Platform-Specific Extensions

| Platform | Library Extension |
|----------|------------------|
| Windows | `.dll` |
| Linux | `.so` |
| macOS | `.dylib` |

## Best Practices

### 1. Group Related Functions

Organize external functions by library:

```auto
extern "c" {
    // GPIO functions
    fn hal_gpio_init(pin int) int;
    fn hal_gpio_write(pin int, value int) int;
    fn hal_gpio_read(pin int) int;

    // SPI functions
    fn hal_spi_init() int;
    fn hal_spi_transfer(data int) int;
}
```

### 2. Use Type Aliases for Clarity

```auto
type PinNumber int
type GPIOValue int

extern "c" {
    fn hal_gpio_init(pin PinNumber) int;
    fn hal_gpio_write(pin PinNumber, value GPIOValue) int;
}
```

### 3. Error Handling Wrapper

Create safe wrappers around FFI functions:

```auto
fn gpio_init_safe(pin int) Result<int, str> {
    let result = hal_gpio_init(pin);
    if result == 0 {
        Ok(result)
    } else {
        Err("GPIO init failed: " + result)
    }
}

fn main() {
    match gpio_init_safe(13) {
        Ok(_) => say("GPIO initialized"),
        Err(msg) => say("Error: " + msg)
    }
}
```

### 4. Document C Signatures

Keep C header files as reference:

```c
// hal.h
int hal_gpio_init(int pin);  // Returns 0 on success
```

```auto
// app.at
// Corresponds to: int hal_gpio_init(int pin);
extern "c" {
    fn hal_gpio_init(pin int) int;
}
```

## Complete Example

### Mixed-Mode Project

**Project Structure**:
```
my_project/
├── pac.at
├── app.at          # Main AutoVM code
├── hal.at          # Hardware layer (C transpilation)
└── target/
    ├── app.bc       # AutoVM bytecode
    ├── hal.c        # Generated C code
    ├── hal.h        # Generated C header
    └── hal.dll      # Compiled C library
```

**pac.at**:
```auto
name: "my_project"
version: "1.0.0"
mode: "autovm"

app("my_project") {
    dependencies: [
        "std:core",
        ("hal", mode: "c"),  # Hardware layer in C
    ]
}
```

**hal.at** (C transpilation):
```auto
#[c]
fn gpio_init(pin int) int {
    // C implementation
    0  # Return success
}

#[c]
fn gpio_write(pin int, value int) int {
    // C implementation
    0
}
```

**app.at** (AutoVM):
```auto
extern "c" {
    // Functions from hal.dll
    fn gpio_init(pin int) int;
    fn gpio_write(pin int, value int) int;
}

fn main() {
    // Initialize GPIO pin 13
    let result = gpio_init(13);

    if result == 0 {
        gpio_write(13, 1);  # Turn on LED
        say("LED turned on")
    }
}
```

**Build Process**:
```bash
# 1. Transpile hal.at to C
auto trans_c hal.at

# 2. Compile C library
gcc -shared -o target/hal.dll target/hal.c

# 3. Compile app.at to AutoVM bytecode
auto build app.at --output target/app.bc

# 4. Run with FFI bridge
auto run target/app.bc
```

## Current Limitations

### TODO Features

The following features are planned but not yet implemented:

1. **Actual libloading integration** (stubbed at line 135-142 in ffi.rs)
   - Currently logs warnings instead of calling C functions
   - Will be implemented in Phase 7

2. **Stack argument popping** (stubbed at line 285-301)
   - Returns dummy values instead of actual arguments
   - Will be implemented in Phase 7

3. **Complex type marshaling**
   - Structs, arrays, pointers not yet supported
   - Planned for future phases

4. **Callback registration** (C → AutoVM)
   - Cannot yet call AutoVM functions from C
   - Planned for Phase 7

5. **Async FFI**
   - No async/await support across FFI boundary
   - Planned for future phases

### Workarounds

For now, use simple types (int, str) and avoid complex data structures across FFI boundaries.

## Troubleshooting

### Issue: "C FFI not yet implemented"

**Cause**: Actual libloading calls are stubbed (Phase 5 limitation).

**Current Behavior**: Functions log warnings and return dummy values.

**Solution**: This is expected in Phase 5. Real FFI calls will be implemented in Phase 7.

### Issue: "Native function not found"

**Cause**: Function not registered with FFI bridge.

**Solution**: Ensure you have `extern "c"` declaration or manually register the function.

### Issue: "Library not found"

**Cause**: Library path incorrect or library not compiled.

**Solution**:
1. Verify library path: `target/hal.dll` (Windows) or `target/hal.so` (Linux)
2. Ensure library is compiled: `gcc -shared -o target/hal.dll target/hal.c`
3. Check library permissions

## Rust 库动态加载（dep + use.rs，PLAN-591 V1）

`dep` 声明 + `use.rs` 导入后，三方 Rust crate 的类型与方法直接可用（430 管线：
nightly rustdoc 提签名 → 分类/生成 shim 包 → cdylib → C3 指纹缓存 → 运行时装载）。

```auto
dep uuid(version: "=1.24.0")     // version 建议精确 pin（= 前缀），caret 会漂移到最新 1.x
use.rs uuid::{Uuid}              // 花括号形态；单类型裸形态不进 dep 管线

fn main() {
    let u = Uuid.parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap()
    print(u)                     // Display 面：impl Display 的类型 print/`to(str)` 走真 Display
    print(u.get_version_num())   // 数值面（宽整型经 i64 槽）
}
```

### 字段访问（V1 布局面）

- pub 字段读：`obj.field` —— 标量字段按**真实布局偏移直读**（探针 crate 用
  `offset_of!` 实测，与 wrapper 同 cdylib 编译保证同源）；String/嵌套对象字段
  走合成 getter（clone 语义）。
- pub 字段写：`obj.field = value` —— 仅标量字段（i*/u*/bool/f*）；写穿透到
  cdylib 堆，Rust 侧方法读回可见。
- **已知限制**：字段写后 Rust 侧缓存旧值不失效（单线程纪律）；别名句柄写语义
  未定义。登记 KNOWN-DEBT（PLAN-591）。

### Option/Result 返回语义（T2）

- `Option<T>` 返回：`Some(x)` 压 x、`None` 压 **null**（仅串/句柄槽；标量槽
  None→哨兵歧义，仍跳过）。判空写法 `x == null`（VM null ≡ a2r None ≡
  oracle `is_none()` 三轨一致）；`.unwrap()` 透传可用（a2r 发射合法 Rust，
  VM 侧 unwrap 对 dep 值恒等、None unwrap 报错）。
- `Result<T, E>` 返回：Ok 压值；**Err 经错误通道转 VMError**（message 携带
  E 的 Display 文本）。`.unwrap()` 范式三轨可用。
- 参数位 `Option<T>`：V1 仅收口返回侧（参数位显式跳过）。

### 已知分歧（详见 parity/docs/known-divergences.md DIV-DEP-15+）

- `use.rs` 单类型无花括号形态不进 dep 管线——**一律用 `{...}` 花括号形态**。
- 方法/字段名撞 Auto 内建名（`find`/`count` 等）会被 a2r 发射器劫持——命名避开。
- a2r 轨 `.to(str)` 对 rust-typed 值仍发 Debug（`print` 已是 Display）——
  断言面用 `print`/`.to_string()` 形态。
- dep 声明 version 的 caret 语义会漂移到最新兼容版——**用 `=x.y.z` 精确 pin**。

## See Also

- [Plan 081 Phase 5 Completion Summary](../plans/081-phase5-complete.md)
- [Mode Selection Guide](mode-selection-guide.md)
- [Multi-Mode Compilation](../plans/081-autovm-default-mode.md#phase-4-multi-mode-compilation-pipeline)
