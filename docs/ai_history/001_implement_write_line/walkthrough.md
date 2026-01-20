# Walkthrough - Implement `write_line` for File

I have implemented the `write_line` method for the `File` class, enabling writing strings with a newline to files in both transpiled C and VM modes.

## Changes

### 1. Standard Library Interface (`stdlib/auto/io.at`)
Added `write_line` method signature to `File` type declaration.

```rust
    #[pub]
    fn write_line(s str);
```

### 2. C Implementation (`stdlib/auto/io.c.at`)
Implemented the C-specific logic for `write_line` using `fputs`.

```rust
    #[pub]
    fn write_line(s str) {
        fputs(s, .file)
        fputs(c"\n", .file)
    }
```

### 3. VM Interface (`stdlib/auto/io.vm.at`)
Added `#[vm]` declaration to `File` extension to signal VM support.

```rust
    #[vm]
    fn write_line(s str)
```

### 4. VM Implementation (`crates/auto-lang/src/vm/io.rs`)
Implemented the Rust backend logic for `write_line`.

### 5. VM Registration (`crates/auto-lang/src/vm.rs`)
Registered the new method in the VM registry.

## Verification Results

### Tests
- **VM Test**: `test_std_file_write_line` passed (verified registration).
- **C Transpilation Test**: `test_c_trans_std_file_write` passed. Confirmed generation of `File_WriteLine` call in the output C file.
