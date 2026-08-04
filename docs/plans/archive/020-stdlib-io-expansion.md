# AutoLang Standard Library I/O Implementation

**Status**: 🔄 **ACTIVE** - Basic File I/O implemented using new ext mechanism
**Last Updated**: 2025-01-20
**Architecture**: Multi-platform via ext (.at + .vm.at + .c.at)

## Current Implementation Status

### ✅ Completed: Basic File I/O (VM Platform)

**Files**:
- `stdlib/auto/io.at` - Interface definition with File type
- `stdlib/auto/io.vm.at` - VM implementation via ext
- `crates/auto-lang/src/vm/io.rs` - VM registry functions

**Implemented Methods**:
```auto
type File {
    #[pub]
    path str

    #[pub]
    static fn open(path str) File

    #[pub]
    fn read_text() str

    #[pub]
    fn read_line() str

    #[pub]
    fn close()
}
```

**VM Implementation** ([io.vm.at:24-36](stdlib/auto/io.vm.at#L24-L36)):
```auto
ext File {
    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str

    #[vm]
    fn read_line() str

    #[vm]
    fn close()
}
```

**Test Status**: ✅ Passing
- `test_std_file` - Basic file operations
- `test_std_io_say` - Console output
- All standard library tests passing

### Architecture: Multi-Platform via ext

**Key Innovation**: ext mechanism supports adding methods and platform-specific fields

**File Organization**:
```
stdlib/auto/
├── io.at        # Public interface (what users see)
├── io.vm.at     # VM implementation (Rust-based VM)
└── io.c.at      # C implementation (future: transpiled C code)
```

**Interface Definition** ([io.at:1-16](stdlib/auto/io.at#L1-L16)):
```auto
type File {
    #[pub]
    path str     # Public field

    #[pub]
    static fn open(path str) File

    #[pub]
    fn read_text() str

    #[pub]
    fn read_line() str

    #[pub]
    fn close()
}
```

**VM Implementation** ([io.vm.at:24-36](stdlib/auto/io.vm.at#L24-L36)):
```auto
ext File {
    // Methods with #[vm] are implemented in Rust
    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str

    #[vm]
    fn read_line() str

    #[vm]
    fn close()
}
```

**VM Registry** ([vm/io.rs:6-54](crates/auto-lang/src/vm/io.rs#L6-L54)):
- `open()` - Opens file using Rust's std::fs::File
- `read_text()` - Reads entire file into string
- `read_line()` - Reads one line with BufReader
- `close()` - Drops file handle

### ❌ Previously Completed Phases (Legacy C Implementation)

The following phases were completed using hand-written C code (old approach):
- Phase 1: Core File Operations
- Phase 2: Character I/O Operations
- Phase 4: Advanced Features (positioning, error handling)
- Phase 5: Spec-Based Polymorphism

**Status**: These implementations exist but need to be migrated to the new architecture:
1. Convert hand-written C to AutoLang in .at files
2. Use ext mechanism for platform-specific code
3. Use `#[vm]` for VM implementations
4. Use `#[c]` or plain functions for C implementations

## New Architecture Principles

### 1. AutoLang-First Implementation

**Old Approach** (❌ Deprecated):
```
Hand-written C code → Manual FFI → AutoLang
```

**New Approach** (✅ Current):
```
AutoLang source (.at) → a2c transpiler → C code
                     → VM evaluator → Execution
```

### 2. Multi-Platform via ext

**Interface Layer** ([io.at](stdlib/auto/io.at)):
```auto
type File {
    #[pub]
    path str

    #[pub]
    static fn open(path str) File

    #[pub]
    fn read_text() str
}
```

**VM Implementation** ([io.vm.at](stdlib/auto/io.vm.at)):
```auto
ext File {
    #[vm]
    static fn open(path str) File

    #[vm]
    fn read_text() str
}
```

**C Implementation** ([io.c.at](stdlib/auto/io.c.at) - Future):
```auto
ext File {
    // Can add private fields for C platform
    _fp *FILE

    // C implementations (transpiled to C)
    static fn open(path str) File {
        let f = fopen(path, c"r")
        // ...
    }

    fn read_text() str {
        // C implementation
    }
}
```

### 3. Loading Order

**Critical**: Files must be loaded in correct order ([parser.rs:2283](crates/auto-lang/src/parser.rs#L2283-L2288)):

```rust
fn get_file_extensions(&self) -> Vec<&'static str> {
    match self.compile_dest {
        CompileDest::Interp => vec![".at", ".vm.at"],     // VM: Interface → Implementation
        CompileDest::TransC => vec![".at", ".c.at"],      // C: Interface → Implementation
        CompileDest::TransRust => vec![".at", ".rust.at"],
    }
}
```

**Why?** Interface declarations (`.at`) must be loaded first, then implementations (`.vm.at`/`.c.at`) can override/complete them.

## Future Work

### High Priority Extensions

1. **Write Operations**:
   - `File.open_write(path str) File`
   - `File.open_append(path str) File`
   - `File.write_line(s str)`

2. **Character I/O**:
   - `File.getc() int` - Read single character
   - `File.putc(c int)` - Write single character

3. **File Positioning**:
   - `File.seek(offset int, origin int) int`
   - `File.tell() int`
   - `File.rewind()`

4. **Error Handling**:
   - `File.is_eof() bool`
   - `File.has_error() bool`
   - `File.clear_error()`

### Implementation Pattern

For each new method:

1. **Add declaration** to [io.at](stdlib/auto/io.at):
   ```auto
   type File {
       #[pub]
       fn write_line(s str)
   }
   ```

2. **Add VM implementation** to [io.vm.at](stdlib/auto/io.vm.at):
   ```auto
   ext File {
       #[vm]
       fn write_line(s str)
   }
   ```

3. **Implement in Rust** ([vm/io.rs](crates/auto-lang/src/vm/io.rs)):
   ```rust
   pub fn write_line(uni: Shared<Universe>, file: &mut Value, s: Value) -> Value {
       // Implementation using std::io::Write
   }
   ```

4. **Register in VM** ([vm.rs](crates/auto-lang/src/vm.rs)):
   ```rust
   file_type.methods.insert("write_line", io::write_line_method as VmMethod);
   ```

5. **Add tests**:
   ```bash
   cargo test -p auto-lang test_std_file
   ```

## Testing

### Current Tests

- ✅ `test_std_file` - Basic file open/read/close
- ✅ `test_std_io_say` - Console output
- ✅ `test_std_test` - Module system
- ✅ All 14 standard library tests passing

### Test Structure

```
crates/auto-lang/test/
├── a2c/
│   ├── 100_std_hello/
│   ├── 101_std_getpid/
│   ├── 102_std_file/
│   └── 113_std_test/
└── integration/
    └── stdlib/
```

## Technical Details

### VM Function Registration

**File**: [crates/auto-lang/src/vm.rs:102-125](crates/auto-lang/src/vm.rs#L102-L125)

```rust
// Register File.open as static function
io_module.functions.insert(
    "File.open".into(),
    VmFunctionEntry {
        name: "File.open".into(),
        func: io::open,
        is_method: false,
    },
);

// Register File type with instance methods
let mut file_type = VmTypeEntry {
    name: "File".into(),
    methods: HashMap::new(),
};

file_type.methods.insert("close".into(), io::close_method as VmMethod);
file_type.methods.insert("read_text".into(), io::read_text_method as VmMethod);
file_type.methods.insert("read_line".into(), io::read_line_method as VmMethod);
```

### VM Implementation Pattern

**File**: [crates/auto-lang/src/vm/io.rs:6-54](crates/auto-lang/src/vm/io.rs#L6-L54)

```rust
pub fn open(uni: Shared<Universe>, path: Value) -> Value {
    match path {
        Value::Str(p) => {
            let f = File::open(p.as_str());
            match f {
                Ok(file) => {
                    // Create File instance with VM reference
                    let reader = std::io::BufReader::new(file);
                    let id = uni.borrow_mut().add_vmref(VmRefData::File(reader));
                    let mut fields = Obj::new();
                    fields.set("id", Value::USize(id));
                    Value::Instance(Instance {
                        ty: auto_val::Type::from(ty),
                        fields,
                    })
                }
                Err(e) => Value::Error(format!("File {} not found: {}", p, e).into()),
            }
        }
        _ => Value::Nil,
    }
}
```

**Key Points**:
- Returns `Value::Instance` with File type
- Stores VM reference ID in `id` field
- Uses `VmRefData::File(BufReader<File>)` for internal storage
- Error handling returns `Value::Error`

## Related Documentation

- [Plan 027: Standard Library C Foundation](027-stdlib-c-foundation.md) - Overall stdlib architecture
- [Design: stdlib-organization.md](../design/stdlib-organization.md) - Multi-platform stdlib design
- [Tutorial: stdlib-organization.md](../tutorials/stdlib-organization.md) - How to use ext mechanism

## Summary

**Current Status**: Basic File I/O working for VM platform

**Architecture**: Clean separation between interface and implementation
- `.at` files define public interface
- `.vm.at` files provide VM implementations
- Future `.c.at` files will provide transpiled C implementations

**Next Steps**:
1. Add write operations
2. Add character I/O
3. Add file positioning
4. Add error handling
5. Migrate legacy C implementations to new architecture
