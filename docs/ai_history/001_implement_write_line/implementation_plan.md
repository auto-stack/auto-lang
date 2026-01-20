# Implementation Plan - Add `write_line` to File

## Goal
Add `write_line(s str)` method to the `File` class in the AutoLang standard library.
This involves:
1.  Defining the method interface in `stdlib/auto/io.at`.
2.  Providing a C implementation using `c::stdio` in `stdlib/auto/io.at`.
3.  Providing a VM interface definition in `stdlib/auto/io.vm.at`.
4.  Implementing the VM backend logic in Rust (`crates/auto-lang/src/vm/io.rs`).
5.  Registering the new VM method in `crates/auto-lang/src/vm.rs`.

## Proposed Changes

### Standard Library

#### [MODIFY] [io.at](file:///d:/autostack/auto-lang/stdlib/auto/io.at)
-   Add `fn write_line(s str);` to `File` type definition.
-   Add implementation for C target:
    ```rust
    fn write_line(s str) {
        c::stdio::fputs(s, .file)
        c::stdio::fputs(c"\n", .file)
    }
    ```
    *Note: Need to verify if `\n` needs to be a C-string `c"\n"` or if auto string converts.*

#### [MODIFY] [io.vm.at](file:///d:/autostack/auto-lang/stdlib/auto/io.vm.at)
-   Add `#[vm]` method declaration to `File` extension:
    ```rust
    #[vm]
    fn write_line(s str)
    ```

### VM Backend (Rust)

#### [MODIFY] [io.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/vm/io.rs)
-   Implement `pub fn write_line(uni: Shared<Universe>, file: &mut Value, args: Vec<Value>) -> Value`
    -   Extract string argument `s` from `args`.
    -   Get `File` reference from `file` instance (similar to `read_text`).
    -   Write `s` + `\n` to the file.
-   Note: The internal file is stored as `BufReader<File>`. `BufReader` implements `BufRead` which is for reading. `File` implements `Read` and `Write`.
    -   Wait, `VmRefData::File(reader)` stores a `BufReader`. `BufReader` does NOT implement `Write` unless the inner type does, but usually it buffers reads.
    -   If the file was opened for reading (default in `open` implementation seemingly?), we might have issues.
    -   Check `open` implementation in `io.rs`: `File::open(p)` opens read-only. `open_write` logic might be needed or `open` needs to handle modes.
    -   **CRITICAL**: The current `open` in `io.rs` uses `File::open(p)` which is read-only. Writing to it will fail or requires changing how file is opened.
    -   However, the user request specifically asked to add `write_line`. I should assume for now I implement the method, and if `open` needs adjustment, I should note it.
    -   Actually `BufReader` takes ownership of the file. To write, we might need to access the inner file or change `VmRefData` to store something that allows writing.
    -   `BufReader::get_mut()` gives mutable reference to inner writer.

#### [MODIFY] [vm.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/vm.rs)
-   Register the `write_line` method in `init_io_module`.

### Testing

#### [MODIFY] [lib.rs](file:///d:/autostack/auto-lang/crates/auto-lang/src/lib.rs)
-   Add `test_std_file_write_line` to `tests` module.
    -   Use `run()` to execute a script that opens a file (if possible to create/write) or uses a mock/stub if file creation isn't widely supported in the test env.
    -   Since `File.open` is currently read-only in logic, the test might fail if it relies on writing.
    -   **Refinement**: Can we create a temporary file in Rust, pass its path to `write_line`? No, `write_line` works on a `File` object created by `File.open`.
    -   I'll assume `File.open` might be patched or I should just try to run it. Or I can mock it by adding `File.create` or `open_write`?
    -   The existing `io.at` implies `open(path)` returns `File`.
    -   Let's check `io.rs` again. It calls `File::open(p)`. This is read-only.
    -   Tests might need to be minimal or expect failure until `open` supports write.
    -   **Actually**, I can check if `test_a2c` has logic.

#### [NEW] [crates/auto-lang/test/a2c/115_std_file_write](D:/autostack/auto-lang/crates/auto-lang/test/a2c/115_std_file_write)
-   Create folder `crates/auto-lang/test/a2c/115_std_file_write`.
-   Add `std_file_write.at` with content:
    ```rust
    // std_file_write.at
    use auto.io: File
    
    fn main() {
        // C-style open for writing simulation (or use wrapper if available)
        // For now, testing compilation of the call
        let file = File.open(c"output.txt")
        file.write_line("Hello, World!")
        file.close()
    }
    ```
-   Note: This test primarily verifies transpilation. Since we implemented `write_line` with C stdio calls, it should transpile to C correctly.
