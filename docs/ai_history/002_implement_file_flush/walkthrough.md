# File.flush() Implementation Walkthrough

Implemented `File.flush()` to support flushing file buffers, with support for both C transpilation and the VM backend.

## Changes

### 1. Standard Library Interface
-   **[io.at](file:///d:/autostack/auto-lang/stdlib/auto/io.at)**: Added `fn flush()` to `File` type.
-   **[io.c.at](file:///d:/autostack/auto-lang/stdlib/auto/io.c.at)**: Implemented using `c.stdio.fflush`.
-   **[io.vm.at](file:///d:/autostack/auto-lang/stdlib/auto/io.vm.at)**: Added `#[vm] fn flush()` declaration.

### 2. VM Backend
-   **[vm/io.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/vm/io.rs)**: Implemented `flush` function relying on `std::io::Write::flush`.
-   **[vm.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/vm.rs)**: Registered `flush` in the VM method registry.

## Verification

### Automated Tests
Ran `cargo test` on:
1.  **VM Registration**: `test_std_file_flush` passed. This test now executes actual AutoLang code in the VM that opens a file and calls `flush()`, confirming method dispatch and execution.
2.  **C Transpilation**: `test_116_std_file_flush` passed (verified A2C transpiles to code calling `File_flush` or inner logic).

### Test Coverage
-   **VM**: Verified presence of method. Runtime functionality depends on `File.open` mode (currently read-only by default in some contexts, but underlying Rust `flush` handles errors gracefully).
-   **C**: Verified correct mapping to `fflush`.
