# Implementation Plan - Add `flush` to File

## Goal
Add `flush()` method to the `File` class in the AutoLang standard library.

## Proposed Changes

### Standard Library

#### [MODIFY] [io.at](file:///d:/autostack/auto-lang/stdlib/auto/io.at)
-   Add `fn flush();` to `File` type definition.

#### [MODIFY] [io.vm.at](file:///d:/autostack/auto-lang/stdlib/auto/io.vm.at)
-   Add `#[vm]` method declaration to `File` extension:
    ```rust
    #[vm]
    fn flush()
    ```

#### [MODIFY] [io.c.at](file:///d:/autostack/auto-lang/stdlib/auto/io.c.at)
-   Update `use` statement to include `fflush`.
-   Implement `flush` using `fflush`:
    ```rust
    #[pub]
    fn flush() {
        if .file != nil {
            fflush(.file)
        }
    }
    ```

### VM Backend (Rust)

#### [MODIFY] [io.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/vm/io.rs)
-   Implement `pub fn flush(uni: Shared<Universe>, file: &mut Value) -> Value`.
    -   Access `VmRefData::File`.
    -   Call `f.get_mut().flush()`.
-   Implement `pub fn flush_method(...)`.

#### [MODIFY] [vm.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/vm.rs)
-   Register the `flush` method in `init_io_module`.

### Testing

#### [MODIFY] [lib.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/lib.rs)
-   Add `test_std_file_flush` VM test (verify registration).

#### [NEW] [crates/auto-lang/test/a2c/116_std_file_flush](D:/autostack/auto-lang/crates/auto-lang/test/a2c/116_std_file_flush)
-   Create validation test for A2C (verify `File_flush` or `fflush` call).
