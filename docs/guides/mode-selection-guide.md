# Mode Selection Guide

AutoLang supports multiple execution and transpilation modes, allowing you to choose the best approach for each part of your project.

## Execution Modes

### AutoVM (Default)
- **Description**: Fast bytecode VM execution
- **Use Case**: General-purpose applications, scripts, servers
- **Advantages**:
  - Fast execution (compiled bytecode)
  - Low memory footprint
  - Works on all platforms (PC, MCU, cloud)
  - Supports all AutoLang features

### Evaluator
- **Description**: Legacy TreeWalker interpreter
- **Use Case**: Debugging, testing, legacy code
- **Note**: Deprecated in favor of AutoVM (Plan 068)

### C Transpilation
- **Description**: Transpiles AutoLang to C code
- **Use Case**: Embedded systems, microcontrollers, low-level hardware
- **Advantages**:
  - Generates portable C code
  - Can be compiled with any C compiler (gcc, clang, msvc)
  - Suitable for resource-constrained environments
  - Easy integration with existing C codebases

### Rust Transpilation
- **Description**: Transpiles AutoLang to Rust code
- **Use Case**: Native applications, high-performance systems
- **Advantages**:
  - Memory safety guarantees
  - Modern tooling (cargo)
  - Excellent performance
  - Rich ecosystem

## pac.at Mode Selection

### Global Project Mode

Set the default execution mode for your entire project:

```auto
// pac.at
name: "myapp"
version: "1.0.0"

// Set execution mode for this project
mode: "autovm"  // Options: "autovm", "evaluator", "c", "rust"

app("myapp") {
    // All dependencies use "autovm" by default
}
```

### Per-Package Mode Override

Specify different modes for individual dependencies:

```auto
// pac.at
name: "mixed_mode_app"
version: "1.0.0"
mode: "autovm"  // Main code uses AutoVM

app("mixed_mode_app") {
    dependencies: [
        "std:core",           // Uses default (AutoVM)

        // Force specific dependencies to use different modes
        ("hal", mode: "c"),           // Hardware layer → C transpilation
        ("crypto", mode: "rust"),     // Crypto library → Rust transpilation
        ("utils", mode: "autovm"),    // Utilities → AutoVM bytecode
    ]
}
```

## Mode Aliases

Short aliases are supported for convenience:

| Mode | Aliases |
|------|---------|
| `autovm` | `vm`, `bytecode` |
| `evaluator` | `eval`, `tree`, `treewalker` |
| `c` | `a2c`, `transpile-c` |
| `rust` | `a2r`, `transpile-rust` |

Example:
```auto
mode: "vm"        // Same as "autovm"
mode: "a2c"       // Same as "c"
mode: "eval"      // Same as "evaluator"
```

## Examples

### Example 1: Embedded Systems Project

Main code runs on microcontroller, hardware abstraction layer in C:

```auto
// pac.at
name: "mcu_firmware"
version: "0.2.0"
mode: "c"  // Transpile entire project to C

app("mcu_firmware") {
    // All code transpiled to C for microcontroller
}
```

**Generated Output**:
- `mcu_firmware.c` - C source code
- `mcu_firmware.h` - C header file
- Can be compiled with `gcc -mmcu=atmega328p`

### Example 2: Server Application

Main application in AutoVM, crypto library in Rust for performance:

```auto
// pac.at
name: "web_server"
version: "1.0.0"
mode: "autovm"

app("web_server") {
    dependencies: [
        "std:core",
        "std:io",
        ("crypto", mode: "rust"),  // High-performance crypto in Rust
    ]
}
```

**Compilation Result**:
- `web_server.bc` - AutoVM bytecode for main app
- `crypto.rs` - Rust code for crypto library
- Crypto functions linked via FFI

### Example 3: Desktop Application with Mixed Modes

```auto
// pac.at
name: "desktop_app"
version: "2.0.0"
mode: "autovm"

app("desktop_app") {
    dependencies: [
        "std:core",
        "std:gui",
        ("graphics", mode: "c"),      // Low-level graphics in C
        ("database", mode: "rust"),   // Database driver in Rust
        ("network", mode: "autovm"),  // Network stack in AutoVM
    ]
}
```

### Example 4: Legacy Project (Evaluator Mode)

For projects that haven't migrated to AutoVM yet:

```auto
// pac.at
name: "legacy_app"
version: "0.9.0"
mode: "evaluator"  // Use old TreeWalker interpreter

app("legacy_app") {
    // Runs with deprecated interpreter
}
```

**Note**: Evaluator mode is deprecated. Consider migrating to AutoVM.

## Mode Selection Decision Tree

```
Need hardware access or embedded deployment?
├─ Yes → Use C transpilation (mode: "c")
└─ No
    ├─ Need memory safety and modern tooling?
    │   ├─ Yes → Use Rust transpilation (mode: "rust")
    │   └─ No → Use AutoVM (mode: "autovm") ← DEFAULT
```

## When to Use Each Mode

### Use AutoVM (default) when:
✅ Building general-purpose applications
✅ Writing scripts or automation tools
✅ Developing server-side applications
✅ Need fast development iteration
✅ Want consistent behavior across platforms
✅ Don't require native code generation

### Use C Transpilation when:
✅ Targeting microcontrollers or embedded systems
✅ Integrating with existing C codebases
✅ Need fine-grained hardware control
✅ Have strict memory constraints
✅ Want to use industry-standard C toolchains
✅ Need bare-metal execution

### Use Rust Transpilation when:
✅ Need memory safety guarantees
✅ Building high-performance native applications
✅ Want modern tooling (cargo, crates.io)
✅ Integrating with Rust ecosystem
✅ Need concurrency without data races
✅ Want zero-cost abstractions

### Use Evaluator when:
⚠️ Only for debugging/legacy support
⚠️ Deprecated - use AutoVM instead

## Mixed-Mode Best Practices

### 1. Performance-Critical Code in Native Languages

```auto
// pac.at
mode: "autovm"

app("app") {
    dependencies: [
        ("crypto", mode: "rust"),  // Crypto algorithms in Rust
        ("compression", mode: "c"), // Compression in C
    ]
}
```

**Why**: Crypto and compression are performance-critical. Native implementations are faster than bytecode.

### 2. Hardware Abstraction Layer in C

```auto
// pac.at
mode: "autovm"

app("firmware") {
    dependencies: [
        ("hal", mode: "c"),  // Hardware access in C
        "std:core",          // Business logic in AutoVM
    ]
}
```

**Why**: C provides direct hardware access and is portable across microcontrollers.

### 3. Business Logic in AutoVM

```auto
// pac.at
mode: "autovm"

app("business_app") {
    dependencies: [
        "std:core",
        "std:database",
        // Keep business logic in AutoVM for fast iteration
    ]
}
```

**Why**: AutoVM is fast enough for most business logic and allows rapid development.

## Mode-Specific Features

### AutoVM Features
- ✅ All AutoLang features supported
- ✅ Hot code reloading (in REPL)
- ✅ Debugging support
- ✅ Profile-guided optimization
- ✅ Works on all platforms

### C Transpilation Features
- ✅ Generates portable C99 code
- ✅ Compatible with any C compiler
- ✅ Can link with existing C libraries
- ✅ Suitable for bare-metal systems
- ⚠️ Some high-level features may be limited

### Rust Transpilation Features
- ✅ Generates idiomatic Rust code
- ✅ Uses Rust's type system
- ✅ Can use Cargo for builds
- ✅ Integrates with crates.io
- ⚠️ Some dynamic features may be limited

## Migration Guide

### From Feature Flags to Mode Selection

**Old way** (feature flags):
```bash
# Build with AutoVM (required feature flag)
cargo build --features use-bigvm

# Build with Evaluator (default)
cargo build
```

**New way** (mode selection):
```auto
// pac.at
mode: "autovm"  // or "c", "rust", "evaluator"
```

No feature flags needed!

### Environment Variable Override

You can still override the execution mode at runtime:

```bash
# Force Evaluator mode for testing
export AUTO_EXECUTION_ENGINE=evaluator
auto run myapp.at

# Force AutoVM mode
export AUTO_EXECUTION_ENGINE=autovm
auto run myapp.at
```

## Advanced: Mode-Dependent Code

You can write code that behaves differently based on the target mode:

```auto
#[cfg(mode: "c")]
fn platform_specific() {
    // C-specific implementation
    say("Running on C platform")
}

#[cfg(mode: "rust")]
fn platform_specific() {
    // Rust-specific implementation
    say("Running on Rust platform")
}

#[cfg(mode: "autovm")]
fn platform_specific() {
    // AutoVM implementation
    say("Running on AutoVM")
}

fn main() {
    platform_specific()
}
```

**Note**: This feature is planned but not yet implemented. For now, use separate files for mode-specific code.

## Troubleshooting

### Issue: "Cannot find dependency in mode X"

**Cause**: Dependency mode not specified or not found.

**Solution**:
```auto
// Explicitly specify the dependency mode
("problematic_dep", mode: "autovm")
```

### Issue: "C transpilation not yet implemented"

**Cause**: Multi-mode compiler doesn't yet support C transpilation integration.

**Current Workaround**: Use `auto trans_c` directly:
```bash
auto trans_c hal.at
```

**Status**: See [Plan 081 Phase 4](../plans/081-phase5-complete.md) for progress.

### Issue: "Rust transpilation not yet implemented"

**Cause**: Multi-mode compiler doesn't yet support Rust transpilation integration.

**Current Workaround**: Use `auto trans_rust` directly:
```bash
auto trans_rust crypto.at
```

**Status**: See [Plan 081 Phase 4](../plans/081-phase5-complete.md) for progress.

## See Also

- [Plan 081: AutoVM as Default](../plans/081-autovm-default-mode.md)
- [Phase 2 Completion Summary](../plans/081-phase2-complete.md)
- [Phase 5: FFI Layer](../plans/081-phase5-complete.md)
- [FFI Usage Guide](ffi-usage-guide.md)
