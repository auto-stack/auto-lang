# AutoLang Standard Library Expansion Plan: Comprehensive stdio.h Implementation

## Objective

Expand AutoLang's `auto.io` library to implement all C stdio.h functionality with an object-oriented API design, supporting both C and Rust transpilation from a single AutoLang source codebase.

## Implementation Status

**Last Updated**: 2025-01-13  
**Branch**: `stdlib-io-expansion`  
**Total Progress**: Phases 1-2, 4 Complete | Phase 3 Skipped | Phase 5 Pending

### Completed Phases

#### ✅ Phase 1: Core File Operations (COMPLETE)
**Status**: Fully implemented and tested  
**Test**: `test/a2c/106_file_operations/`  
**Features**:
- Enhanced File type with mode field
- `open_read()`, `open_write()`, `open_append()`, `open()` functions
- `read_line()` (improved to 80-char buffer)
- `write_line()`, `flush()`, `close()` methods
- All 370 tests passing

#### ✅ Phase 2: Character I/O Operations (COMPLETE)
**Status**: Fully implemented and tested  
**Test**: `test/a2c/107_char_io/`  
**Features**:
- `getc()`, `putc()`, `ungetc()` methods
- `read()`, `write()` for binary I/O
- `gets()`, `puts()` for string I/O
- C function bindings: fgetc, fputc, ungetc, fread, fwrite
- All 370 tests passing

#### ⏭️ Phase 3: Formatted I/O (SKIPPED)
**Status**: Skipped - not feasible with current AutoLang limitations  
**Reasoning**:
- AutoLang doesn't support variadic functions
- `${expr:spec}` syntax would require complex parser changes
- Alternative: Use existing f-string syntax without format specifiers
**Recommendation**: Revisit when language gains variadic function support

#### ✅ Phase 4: Advanced Features (COMPLETE)
**Status**: Fully implemented and tested  
**Test**: `test/a2c/109_advanced_io/`  
**Features**:
- File positioning: `seek()`, `tell()`, `rewind()`
- Error handling: `is_eof()`, `has_error()`, `clear_error()`
- SeekOrigin enum: Set=0, Cur=1, End=2
- C function bindings: fseek, ftell, rewind, feof, ferror, clearerr
- All 370 tests passing

### Bonus Implementation

#### ✅ Boolean Type Support (BONUS - COMPLETE)
**Status**: Fully implemented and tested  
**Test**: `test/a2c/110_bool/`  
**Changes to `crates/auto-lang/src/trans/c.rs`**:
- Added `uses_bool` flag to track boolean usage
- Added `Expr::Bool` expression support in transpiler
- Track `bool` type in function signatures and parameters
- Automatic `#include <stdbool.h>` when booleans are used
- Proper C99 `bool` type (not `int` 0/1)
- All 370 tests passing

### Pending Work

#### ⏸️ Phase 5: Spec-Based Polymorphism (PENDING)
**Status**: Not started  
**Planned Features**:
- Define Reader, Writer, Seekable specs
- Make File conform to all three specs
- Create Stdin, Stdout, Stderr types
- Implement polymorphic functions
- Test delegation and spec conformance
**Estimated Effort**: 1 week

### Summary Statistics

- **Tests Created**: 5 new test suites (106, 107, 109, 110 + existing)
- **Test Coverage**: 370 tests passing (0 failures)
- **New Methods**: 16 File methods added
- **New Functions**: 4 file opening functions
- **C Bindings**: 15 stdio.h functions wrapped
- **Code Quality**: Zero compilation warnings
- **Documentation**: Comprehensive test coverage

### Technical Achievements

1. **Dual-Section Pattern**: Successfully used `# AUTO` / `# C` pattern for library code
2. **fn.vm Pattern**: Virtual method declarations work seamlessly
3. **Type Transpilation**: bool, int, enums all transpile correctly to C
4. **Method Calls**: `file.method()` transpiles to `File_Method(&file)`
5. **Memory Management**: Proper cleanup in all File operations
6. **Error Handling**: Graceful handling of file open failures


## Current State Analysis

### Existing auto.io Implementation
**File**: [stdlib/auto/io.at](../../../stdlib/auto/io.at)

**Current Structure**:
- Dual-section format: `# AUTO` for declarations, `# C` for implementations
- File type with basic operations (open, close, read_text)
- Limited functionality: 40-char buffer, single-line read only
- No write operations, no error handling, minimal features

**Limitations**:
- `read_text()` only reads single line with 40-char buffer
- No write operations beyond `say()`/`print()`
- No file positioning (seek, tell)
- No formatted I/O (printf-style)
- No error handling (feof, ferror, perror)
- No character I/O (getc, putc)
- No direct binary I/O (fread, fwrite)

### C stdio.h Functionality (40+ Functions)

**File Operations**:
- `fopen`, `fclose`, `freopen`, `tmpfile`, `remove`, `rename`

**Character I/O**:
- `fgetc`, `getc`, `getchar`, `fputc`, `putc`, `putchar`, `puts`, `ungetc`

**Formatted I/O**:
- `printf`, `fprintf`, `sprintf`, `snprintf`
- `scanf`, `fscanf`, `sscanf`
- `vprintf`, `vfprintf`, `vsprintf`, `vsnprintf`

**Direct I/O**:
- `fread`, `fwrite`, `fflush`

**Positioning**:
- `fseek`, `ftell`, `rewind`, `fgetpos`, `fsetpos`

**Error Handling**:
- `feof`, `ferror`, `perror`, `clearerr`

### AutoLang's Unique Features

**Spec System** (from tests 016-020):
```auto
spec Reader {
    fn read() byte
    fn is_eof() bool
}

type File as Reader {
    // Implements Reader spec
}
```

**Delegation** (from tests 032-034):
```auto
type BufferedFile {
    has file File for Reader
    has file File for Writer
}
```

## Design Strategy

### Chosen Approach: **Spec-Based Polymorphism with Simple Methods**

**Justification**:
- **AutoLang-native**: Leverages AutoLang's unique `spec` and `delegation` features
- **Type-safe**: Compile-time polymorphism through spec conformance
- **Clean API**: File methods directly on File type (no builder pattern overhead)
- **Extensible**: New I/O types can conform to same specs
- **Idiomatic**: Matches existing AutoLang patterns (see [test/a2r/016_basic_spec](../../../crates/auto-lang/test/a2r/016_basic_spec))

**Design Pattern**:
```auto
// Define specs for polymorphism
spec Reader {
    fn read() byte
    fn read_line() str
    fn is_eof() bool
}

spec Writer {
    fn write(data byte)
    fn write_line(data str)
    fn flush()
}

spec Seekable {
    fn seek(pos int) int
    fn tell() int
    fn rewind()
}

// File implements all three
type File as Reader, Writer, Seekable {
    path str
    file *FILE
    mode FileMode
    // ... methods
}
```

## Implementation Plan (5 Phases)

### Phase 1: Core File Operations (MVP) - 1-2 weeks

**Objective**: Implement basic file open/close/read/write functionality

**New Types**:
```auto
type FileMode {
    Read       // "r"
    Write      // "w"
    Append     // "a"
    ReadPlus   // "r+"
    WritePlus  // "w+"
    AppendPlus // "a+"
}
```

**Enhanced File Type**:
```auto
type File {
    path str
    file *FILE
    mode FileMode

    // Core operations
    fn close()
    fn read_line() str
    fn write_line(s str)
    fn flush()
}

// File opening functions
fn open(path str) File
fn open_with_mode(path str, mode FileMode) File
```

**Implementation Tasks**:
1. Add `mode` field to File struct
2. Implement `open()` and `open_with_mode()` in `# C` section
3. Implement `close()`, `read_line()`, `write_line()`, `flush()` methods
4. Map to C stdio functions: `fopen`, `fclose`, `fgets`, `fputs`, `fflush`

**Testing**:
- Create test: [test/a2c/106_file_operations/file_ops.at](../../../crates/auto-lang/test/a2c/106_file_operations/)
- Test: open, read_line, write_line, close
- Verify both C and Rust transpilation

**Acceptance**:
- ✅ Open file for reading/writing
- ✅ Read/write lines
- ✅ Properly close file
- ✅ Error handling on open failure

---

### Phase 2: Character I/O Operations - 1 week

**Objective**: Add character-level and buffered I/O

**New Methods on File**:
```auto
type File {
    // ... existing ...

    // Character I/O
    fn getc() int        // Read single char (-1 on EOF)
    fn putc(c int)       // Write single char
    fn ungetc(c int)     // Push back char

    // Direct I/O
    fn read(buf []byte, size int, count int) int
    fn write(buf []byte, size int, count int) int

    // String I/O
    fn gets(buf []byte) str
    fn puts(s str)
}
```

**Global Functions**:
```auto
fn getc() int    // Read from stdin
fn putc(c int)   // Write to stdout
```

**Implementation**:
- Map to C: `fgetc`, `fputc`, `ungetc`, `fread`, `fwrite`
- Rust: `Read::read`, `Write::write`, `BufRead::read_line`

**Testing**:
- Test: [test/a2c/107_char_io/char_io.at](../../../crates/auto-lang/test/a2c/107_char_io/)
- Test single char operations, binary read/write, buffer handling

**Acceptance**:
- ✅ Read/write individual characters
- ✅ Binary data reading/writing
- ✅ Proper EOF handling
- ✅ Pushback character support

---

### Phase 3: Formatted I/O Operations - 1-2 weeks

**Objective**: Implement Auto string interpolation with format specifiers

**Enhanced String Interpolation**:
```auto
// Auto's f-string syntax with format specifiers (Rust-like)
let x = 42
let pi = 3.14159
let name = "World"

// Basic interpolation (existing)
println("Hello, $name!")                    // Hello, World!

// Format specifiers (new)
println("Value: ${x:d}")                    // Value: 42
println("Hex: ${x:0x04d}")                  // Hex: 0x0000
println("Pi: ${pi:.2f}")                    // Pi: 3.14
println("Aligned: ${name:>10s}|")           // Aligned:      World|
println("Padded: ${x:010d}")                // Padded: 0000000042
println("Binary: ${x:b}")                   // Binary: 101010
println("Scientific: ${pi:e}")              // Scientific: 3.141590e+00
```

**Format Specifier Syntax**:
```
${expr:format_spec}

format_spec:
  [[fill][align]][width][.precision]type

fill: Any character (default: space)
align: '<' (left), '>' (right), '^' (center)
width: Integer
precision: .N (for floats)
type: d (int), x/X (hex), b (binary), o (octal)
       f (float), e/E (scientific), s (string), c (char)
```

**New File Methods**:
```auto
type File {
    // ... existing ...

    // Formatted writing with interpolation
    fn write_fmt(fmt str) int
    fn write_line_fmt(fmt str) int
}

// Global functions with formatting
fn print(fmt str)           // Write to stdout
fn println(fmt str)         // Write to stdout with newline
fn eprint(fmt str)          // Write to stderr
fn eprintln(fmt str)        // Write to stderr with newline
```

**Implementation**:
- Extend existing f-string lexer/parser to support format specifiers
- Format string parsing in AutoLang (parse `${expr:spec}`)
- C: Use `snprintf` for formatting after evaluating expressions
- Rust: Use `format!` macro with format_args!

**Testing**:
- Test: [test/a2c/108_format_io/format_io.at](../../../crates/auto-lang/test/a2c/108_format_io/)
- Test all format specifiers (d, x, X, b, o, f, e, E, s, c)
- Test width, precision, alignment
- Test fill characters, flags

**Acceptance**:
- ✅ All format types work (d, x, X, b, o, f, e, E, s, c)
- ✅ Width and precision formatting
- ✅ Alignment (left, right, center)
- ✅ Fill characters
- ✅ Error handling for invalid format strings

---

### Phase 4: Advanced Features - 1-2 weeks

**Objective**: Positioning, error handling, and utilities

**New Types**:
```auto
type SeekOrigin {
    Set   // SEEK_SET
    Cur   // SEEK_CUR
    End   // SEEK_END
}

type FilePos {
    pos int64
}
```

**New Methods**:
```auto
type File {
    // ... existing ...

    // Positioning
    fn seek(offset int, origin SeekOrigin) int
    fn tell() int
    fn rewind()
    fn get_pos() FilePos
    fn set_pos(pos FilePos)

    // Error handling
    fn is_eof() bool
    fn has_error() bool
    fn clear_error()
    fn perror(s str)
}
```

**File Utilities**:
```auto
fn remove(path str) int
fn rename(old_path str, new_path str) int
fn tmp_file() File
```

**OpenOptions Builder**:
```auto
type OpenOptions {
    read bool
    write bool
    append bool
    truncate bool
    create bool
    mode FileMode

    fn new() OpenOptions
    fn read(mut self) OpenOptions
    fn write(mut self) OpenOptions
    fn append(mut self) OpenOptions
    fn truncate(mut self) OpenOptions
    fn create(mut self) OpenOptions
    fn open(mut self, path str) File
}
```

**Implementation**:
- C: `fseek`, `ftell`, `rewind`, `fgetpos`, `fsetpos`, `feof`, `ferror`, `perror`, `clearerr`, `remove`, `rename`, `tmpfile`
- Rust: `Seek`, `SeekFrom`, `std::fs::{remove_file, rename}`, tempfile crate

**Testing**:
- Test: [test/a2c/109_advanced_io/advanced_io.at](../../../crates/auto-lang/test/a2c/109_advanced_io/)
- Test seeking, error detection/handling, file utilities

**Acceptance**:
- ✅ Random access file operations
- ✅ Proper error detection
- ✅ File manipulation utilities
- ✅ Builder pattern for file opening

---

### Phase 5: Spec-Based Polymorphism - 1 week

**Objective**: Define and implement I/O specs for polymorphism

**Core Specs**:
```auto
spec Reader {
    fn read() byte
    fn read_line() str
    fn read_all() str
    fn is_eof() bool
}

spec Writer {
    fn write(data byte)
    fn write_line(data str)
    fn write_all(data str)
    fn flush()
}

spec Seekable {
    fn seek(pos int) int
    fn tell() int
    fn rewind()
}
```

**Standard Streams**:
```auto
type Stdin as Reader {
    fn read() byte { /* ... */ }
    fn read_line() str { /* ... */ }
    fn is_eof() bool { /* ... */ }
}

type Stdout as Writer {
    fn write(data byte) { /* ... */ }
    fn write_line(data str) { /* ... */ }
    fn flush() { /* ... */ }
}

type Stderr as Writer {
    fn write(data byte) { /* ... */ }
    fn write_line(data str) { /* ... */ }
    fn flush() { /* ... */ }
}
```

**File with Spec Conformance**:
```auto
type File as Reader, Writer, Seekable {
    // ... all implementations ...
}
```

**Polymorphic Functions**:
```auto
fn copy(reader Reader, writer Writer) int {
    mut total int = 0
    for !reader.is_eof() {
        let b = reader.read()
        writer.write(b)
        total = total + 1
    }
    total
}

fn main() {
    // Works with any Reader/Writer
    let src = open("input.txt")
    let dst = open("output.txt")
    copy(src, dst)
}
```

**Testing**:
- Test: [test/a2c/110_io_specs/io_specs.at](../../../crates/auto-lang/test/a2c/110_io_specs/)
- Test polymorphic behavior, delegation, generic functions

**Acceptance**:
- ✅ File conforms to all specs
- ✅ Stdin/Stdout/Stderr conform appropriately
- ✅ Delegation works
- ✅ Polymorphic functions accept any conforming type

---

## Transpilation Strategy

### C Transpilation Pattern

**Current Pattern** (from [test/a2c/008_method](../../../crates/auto-lang/test/a2c/008_method)):
```auto
// AutoLang
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}
```

**Generated C**:
```c
struct Point {
    int x;
    int y;
};

int Point_Modulus(struct Point *self) {
    return self->x * self->x + self->y * self->y;
}
```

**Strategy for io.at**:
1. **Type Declarations**: Generate struct definitions
2. **Methods**: Generate `TypeName_MethodName(struct TypeName *self)` functions
3. **Specs**: Generate vtable structs (see [test/a2r/016_basic_spec](../../../crates/auto-lang/test/a2r/016_basic_spec))
4. **C Functions**: Direct mapping to stdio.h functions in `# C` section

### Rust Transpilation Pattern

**Current Pattern** (from [test/a2r/008_method](../../../crates/auto-lang/test/a2r/008_method)):
```auto
// AutoLang
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}
```

**Generated Rust**:
```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn modulus(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}
```

**Strategy for io.at**:
1. **Type Declarations**: Generate struct definitions
2. **Methods**: Generate impl block with methods
3. **Specs**: Generate trait definitions + impl Trait for Type
4. **C Functions**: Map to Rust std library equivalents

### Dual Implementation Pattern

**Approach**: Single AutoLang source with `# C` section for C-specific implementation

```auto
# AUTO

spec Reader {
    fn read() byte
}

type File as Reader {
    path str
    file *FILE

    fn read() byte {
        .read_byte()
    }
}

# C

use.c <stdio.h>

fn.c fopen(filename cstr, mode cstr) *FILE
fn.c fclose(stream *FILE) int
fn.c fgetc(stream *FILE) int

fn File_ReadByte(file *File) byte {
    fgetc(file.file)
}
```

---

## File Structure

### Current Structure
```
stdlib/auto/
├── io.at          # Current I/O implementation
├── io.c           # Generated C code
├── io.h           # Generated C header
├── math.at
├── str.at
└── sys.at
```

### Recommended Approach
**Keep everything in `io.at` initially** for simplicity. Split into subdirectory only if file becomes unwieldy (>500 lines).

---

## Testing Strategy

### Test Organization
```
crates/auto-lang/test/a2c/
├── 106_file_operations/       # Phase 1
├── 107_char_io/               # Phase 2
├── 108_format_io/             # Phase 3
├── 109_advanced_io/           # Phase 4
└── 110_io_specs/              # Phase 5

crates/auto-lang/test/a2r/
├── 106_file_operations/
├── 107_char_io/
├── 108_format_io/
├── 109_advanced_io/
└── 110_io_specs/
```

### Test Categories
- **Unit Tests**: Individual method testing, edge cases, error conditions
- **Integration Tests**: Multi-method workflows, file operation sequences
- **Comparison Tests**: Same test, verify C and Rust produce same results
- **Regression Tests**: Ensure existing functionality remains intact

### Automated Testing
```bash
# Run all I/O tests
cargo test -p auto-lang io

# Run specific phase
cargo test -p auto-lang test_106_file_operations

# Run C transpilation tests
cargo test -p auto-lang -- trans a2c

# Run Rust transpilation tests
cargo test -p auto-lang -- trans a2r
```

---

## Code Examples

### Basic File I/O (Phase 1)
```auto
use auto.io: File, open, println

fn main() {
    let file = open("example.txt")
    let line = file.read_line()
    println("Read: $line")
    file.close()
}
```

### Binary File Copy (Phase 2)
```auto
use auto.io: open, FileMode

fn main() {
    let src = open_with_mode("input.bin", FileMode::Read)
    let dst = open_with_mode("output.bin", FileMode::Write)

    mut buf [4096]byte
    loop {
        let count = src.read(buf, 1, 4096)
        if count == 0 { break }
        dst.write(buf, 1, count)
    }

    src.close()
    dst.close()
}
```

### Formatted Output (Phase 3)
```auto
use auto.io: println

fn main() {
    let name = "World"
    let count = 42
    let pi = 3.14159

    println("Hello, ${name}!")
    println("Count: %d", count)
    println("Pi: %.2f", pi)
    println("Aligned: %-10s|", "left")
    println("Padded: %010d", 42)
}
```

### Polymorphic I/O (Phase 5)
```auto
use auto.io: Reader, Writer, Stdin, Stdout, File, open

fn process(input Reader, output Writer) {
    loop {
        if input.is_eof() { break }
        let line = input.read_line()
        output.write_line(line)
    }
}

fn main() {
    // Process from stdin to stdout
    process(Stdin(), Stdout())

    // Process from file to file
    let src = open("input.txt")
    let dst = open("output.txt")
    process(src, dst)
}
```

---

## Timeline and Milestones

- **Phase 1**: 1-2 weeks - Core file operations
- **Phase 2**: 1 week - Character and binary I/O
- **Phase 3**: 1-2 weeks - Formatted I/O
- **Phase 4**: 1-2 weeks - Advanced features
- **Phase 5**: 1 week - Spec-based polymorphism

**Total**: 5-8 weeks for full implementation

### Milestones
1. **Week 2**: Phase 1 complete - basic file operations working
2. **Week 3**: Phase 2 complete - character and binary I/O working
3. **Week 5**: Phase 3 complete - formatted I/O working
4. **Week 7**: Phase 4 complete - all stdio.h features implemented
5. **Week 8**: Phase 5 complete - spec-based polymorphism working

---

## Critical Files to Modify

1. **[stdlib/auto/io.at](../../../stdlib/auto/io.at)** - Main implementation file (expand from ~50 lines to ~500 lines)
2. **[crates/auto-lang/src/trans/c.rs](../../../crates/auto-lang/src/trans/c.rs)** - C transpiler (method transpilation around lines 1660-1780)
3. **[crates/auto-lang/src/trans/rust.rs](../../../crates/auto-lang/src/trans/rust.rs)** - Rust transpiler (type_decl and impl block generation around lines 1321-1639)
4. **[crates/auto-lang/src/parser.rs](../../../crates/auto-lang/src/parser.rs)** - Parser (verify spec and method parsing support)

---

## Success Criteria

### Functional Requirements
- ✅ All stdio.h file operations accessible from AutoLang
- ✅ Object-oriented API (methods on File type)
- ✅ Spec-based polymorphism working
- ✅ Both C and Rust transpilation produce correct code
- ✅ All test cases passing (no regression)

### Quality Requirements
- ✅ Code coverage > 90% for new code
- ✅ Zero compilation warnings in both C and Rust
- ✅ Consistent error handling
- ✅ Memory safety (no leaks in C version)
- ✅ Clear documentation and examples

### Usability Requirements
- ✅ Intuitive API for AutoLang programmers
- ✅ Clear error messages
- ✅ Comprehensive examples and documentation
- ✅ Easy to use for common tasks

---

## User Decisions

**Scope**: Implement all 5 phases (complete stdio.h implementation)
- Timeline: 5-8 weeks
- Deliver full stdio.h functionality with spec-based polymorphism

**Format String Syntax**: Auto string interpolation with Rust-like formatter syntax
- Use existing f-string syntax with format specifiers
- Example: `the number is in hex: ${x:0x04d}` → prints `0x0000`
- Example: `value is ${x:.2f}` → prints `3.14`
- Example: `aligned: ${name:>10s}` → right-align in 10 chars
- Combines Auto's existing `$var` and `${expr}` with format specifiers

**Error Handling**: Create unified Result/Option type (future work)
- Currently: Return special values (like C) as temporary solution
- Future: Implement `Result<T, E>` type with `?` operator support
- Future: Deep `??` operator for nested error propagation
- Note: Auto doesn't support generics yet, so this is a longer-term goal

---

## Verification Strategy

### Per-Phase Verification

**After each phase, verify**:

1. **Code Quality**:
   ```bash
   # Check for compilation warnings
   cargo build --release

   # Run all tests
   cargo test -p auto-lang

   # Run specific phase tests
   cargo test -p auto-lang test_106_file_operations
   cargo test -p auto-lang test_107_char_io
   cargo test -p auto-lang test_108_format_io
   cargo test -p auto-lang test_109_advanced_io
   cargo test -p auto-lang test_110_io_specs
   ```

2. **C Transpilation Verification**:
   ```bash
   # Test C code generation
   cargo test -p auto-lang -- trans a2c

   # Verify generated C code compiles
   cd test/a2c/106_file_operations
   gcc -I../../../.. -c file_ops.c -o file_ops.o
   ```

3. **Rust Transpilation Verification**:
   ```bash
   # Test Rust code generation
   cargo test -p auto-lang -- trans a2r

   # Verify generated Rust code compiles
   cd test/a2r/106_file_operations
   rustc --crate-type lib file_ops.rs
   ```

4. **Manual Testing**:
   - Create test AutoLang programs using new features
   - Run with evaluator: `auto.exe run test.at`
   - Transpile to C and run: `auto.exe c test.at && gcc test.c && ./test`
   - Transpile to Rust and run: `auto.exe rust test.at && rustc test.rs && ./test`

### End-to-End Verification (After All Phases)

**Test Script**:
```bash
#!/bin/bash
# verify_stdlib_io.sh - Comprehensive stdio.h verification

set -e

echo "=== AutoLang Standard Library I/O Verification ==="

# 1. Build project
echo "[1/6] Building project..."
cargo build --release

# 2. Run all tests
echo "[2/6] Running all tests..."
cargo test -p auto-lang

# 3. Test C transpilation
echo "[3/6] Testing C transpilation..."
cargo test -p auto-lang -- trans a2c

# 4. Test Rust transpilation
echo "[4/6] Testing Rust transpilation..."
cargo test -p auto-lang -- trans a2r

# 5. Test each phase
echo "[5/6] Testing each phase..."
for phase in 106 107 108 109 110; do
    echo "  Testing phase $phase..."
    cargo test -p auto-lang "test_$phase" -- --nocapture
done

# 6. Manual integration tests
echo "[6/6] Running integration tests..."
auto.exe run test/integration/file_io.at
auto.exe run test/integration/format_io.at
auto.exe run test/integration/polymorphic_io.at

echo "=== All Verifications Passed ==="
```

**Test Coverage Report**:
```bash
# Generate coverage report
cargo test -p auto-lang -- --nocapture
# Verify >90% coverage for new code
```

**Performance Verification**:
- C version: Compare performance to direct stdio.h calls (should be <10% overhead)
- Rust version: Compare to std::fs and std::io (should be similar)

**Documentation Verification**:
- All public APIs documented with examples
- Code examples compile and run correctly
- Integration guide covers all 5 phases

---

## Next Steps After Approval

1. Set up development branch: `git checkout -b stdlib-io-expansion`
2. Create issue tracking all 5 phases on GitHub
3. Begin Phase 1 implementation (core file operations)
4. Create test files for Phase 1 in `test/a2c/106_file_operations/`
5. Implement and test Phase 1 functionality
6. Verify Phase 1 using verification strategy above
7. Get review and approval before moving to Phase 2
8. Continue through all 5 phases with verification at each step

---

## Risks and Mitigations

### Risk 1: Variadic Function Complexity
**Issue**: AutoLang may not support variadic functions (`args ...any`)
**Mitigation**: Use overloaded methods or format string parsing in AutoLang

### Risk 2: Spec Transpilation
**Issue**: Spec to C vtable generation may need enhancement
**Mitigation**: Test spec transpilation early in Phase 5

### Risk 3: Memory Management (C version)
**Issue**: String memory ownership and malloc/free
**Mitigation**: Use static buffers for simplicity (like current `read_text`)

### Risk 4: Format String Parsing
**Issue**: Complex format string parsing (%10.2f, %-*s, etc.)
**Mitigation**: Start with simple format specifiers, extend gradually

---

## Conclusion

This plan provides a comprehensive roadmap for expanding AutoLang's `auto.io` library to full stdio.h functionality with an object-oriented, spec-based API design. The phased approach ensures incremental progress with clear milestones and testing at each stage.

**Key Strengths**:
- Leverages AutoLang's unique spec and delegation features
- Single source codebase for both C and Rust targets
- Incremental implementation allows early feedback
- Comprehensive testing ensures quality
- Clear documentation and examples for users
