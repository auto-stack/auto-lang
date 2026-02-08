# a2r Transpiler API Documentation

## Overview

The `a2r` (Auto-to-Rust) transpiler is exposed through the `RustTrans` struct and the `Trans` trait.

## Core API

### RustTrans

The main transpiler struct that handles AutoLang to Rust conversion.

```rust
use auto_lang::trans::rust::RustTrans;

pub struct RustTrans {
    // Private fields
    indent: usize,
    uses: HashSet<AutoStr>,
    scope: Option<Shared<Universe>>,
    db: Option<Arc<RwLock<Database>>>,
    edition: RustEdition,
    current_fn: Option<AutoStr>,
    current_scope: Option<Sid>,
}
```

### Constructors

```rust
impl RustTrans {
    /// Create a new transpiler instance
    pub fn new(name: AutoStr) -> Self;

    /// Create transpiler with database support
    pub fn with_database(db: Arc<RwLock<Database>>) -> Self;

    /// Create transpiler with specific Rust edition
    pub fn with_edition(edition: RustEdition) -> Self;
}
```

### Trans Trait Implementation

```rust
impl Trans for RustTrans {
    /// Transpile AutoLang AST to Rust code
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()>;
}
```

## Usage Examples

### Basic Transpilation

```rust
use auto_lang::{trans::Trans, trans::rust::RustTrans, trans::Sink};
use auto_lang::AutoStr;
use auto_val::AutoStr;

fn main() -> auto_lang::AutoResult<()> {
    // Parse AutoLang code
    let code = r#"
        fn main() {
            let x = 42
            print(x)
        }
    "#;

    let mut parser = auto_lang::parser::Parser::new(code);
    let ast = parser.parse_file()?;

    // Transpile to Rust
    let mut trans = RustTrans::new("test".into());
    let mut sink = Sink::new(AutoStr::from("output"));
    trans.trans(ast, &mut sink)?;

    // Get output
    let rust_code = String::from_utf8(sink.done()?.to_vec())?;
    println!("{}", rust_code);

    Ok(())
}
```

### With Database Support

```rust
use auto_lang::trans::Trans;
use auto_lang::trans::rust::RustTrans;
use auto_lang::compile::CompileSession;
use auto_val::AutoStr;

fn transpile_with_session(
    code: &str,
    session: &mut CompileSession,
) -> auto_lang::AutoResult<String> {
    // Parse and compile with incremental support
    let frag_id = session.compile_source(code, AutoStr::from("input"))?;

    // Get the AST from database
    let ast = session.db().read()?
        .get_fragment(&frag_id)
        .unwrap();

    // Transpile
    let mut trans = RustTrans::with_database(session.db().clone());
    let mut sink = Sink::new(AutoStr::from("output"));
    trans.trans(ast.clone(), &mut sink)?;

    Ok(String::from_utf8(sink.done()?.to_vec())?)
}
```

## Transpiler Configuration

### Rust Edition

```rust
pub enum RustEdition {
    E2021,  // Rust 2021 edition
    E2024,  // Rust 2024 edition (default)
}
```

```rust
let mut trans = RustTrans::with_edition(RustEdition::E2024);
```

## Output Format

The transpiler outputs valid Rust code with:

1. **Proper indentation** - 4 spaces per level
2. **Type annotations** - Explicit types where needed
3. **Semicolons** - All statements end with `;`
4. **Module structure** - Proper use statements
5. **Trait implementations** - Full trait system support

## AST Node Transpilation

The transpiler handles all AutoLang AST nodes:

### Expressions

| AST Node | Rust Output |
|----------|------------|
| `Int(i)` | `i` |
| `Float(f, scale)` | `f * scale` |
| `String(s)` | `"s"` |
| `Ident(name)` | `name` |
| `Binary(op, left, right)` | `left op right` |
| `Unary(op, expr)` | `op expr` |
| `If(branches, else_)` | `if/else or match` |
| `Call(func, args)` | `func(args)` |
| `Closure(params, body)` | `\|params\| body` |

### Statements

| AST Node | Rust Output |
|----------|------------|
| `Expr(expr)` | `expr;` |
| `Store(store)` | `let/var/const` declarations |
| `Fn(fn_decl)` | `fn` definitions |
| `For(for_stmt)` | `for` loops |
| `While(cond, body)` | `while` loops |
| `Is(expr, branches)` | `match` expressions |
| `Use(use_stmt)` | `use` statements |
| `Return(expr)` | `return expr;` |
| `Break` | `break;` |

### Type Declarations

| AST Node | Rust Output |
|----------|------------|
| `TypeDecl(type_decl)` | `struct` with generics |
| `SpecDecl(spec_decl)` | `trait` with generics |
| `Tag(tag)` | `enum` with generics |
| `Ext(ext)` | `impl` blocks |
| `TypeAlias(type_alias)` | `type` aliases |

## Type Inference

The transpiler preserves type information from the AST:

- **Explicit types**: Preserved as-is
- **Inferred types**: Added as type annotations
- **Generic parameters**: Fully supported
- **Closure types**: Inferred by Rust compiler

## Memory Safety

### Borrow Checking

AutoLang's ownership system transpiles to Rust's borrow checker:

```auto
fn process(data List) {
    let view = data.view   // Immutable borrow
    let mut = data.mut     // Mutable borrow
    process(view)
}
```

```rust
fn process(data: List) {
    let view = &data;      // Immutable borrow
    let mut_ref = &mut data; // Mutable borrow
    process(view);
}
```

### Pointer Operations

```auto
let ptr = x.@           // Address-of
let val = *ptr          // Dereference
```

```rust
let ptr = x as *mut _;   // Address-of
let val = *ptr;          // Dereference
```

## Error Handling

All transpiler operations return `AutoResult<T>`:

```rust
pub type AutoResult<T> = std::result::Result<T, AutoError>;
```

Errors include:
- Syntax errors (invalid AutoLang)
- Type errors (type mismatches)
- Transpilation errors (unsupported features)

## Performance Considerations

- **Compilation speed**: Transpilation is fast (< 1s for typical files)
- **Output quality**: Idiomatic Rust that compiles efficiently
- **Optimization**: Rust compiler optimizes final output
- **Incremental**: Database support for incremental builds

## Extension Points

### Custom Type Mappings

To add custom type mappings, modify the `rust_type_name()` method:

```rust
fn rust_type_name(&mut self, ty: &Type) -> String {
    match ty {
        Type::Int => "i32".to_string(),
        Type::User(name) => name.as_string(),
        // Add custom mappings here
        _ => format!("{:?}", ty),
    }
}
```

### Custom Statement Handlers

To handle custom statement types, extend the `stmt()` method:

```rust
Stmt::MyCustom(stmt) => {
    // Custom transpilation logic
    self.transpile_my_custom(stmt, out)?;
    Ok(true)
}
```

## Testing Utilities

The test framework provides utilities for testing transpilation:

```rust
#[test]
fn test_example() {
    test_a2r("example").unwrap();
}
```

Test files:
- `test/a2r/000_hello/hello.at` - Input
- `test/a2r/000_hello/hello.expected.rs` - Expected output

## Best Practices

1. **Always test transpilation** - Use the test framework
2. **Check Rust compilation** - Verify output compiles with `rustc`
3. **Preserve semantics** - Ensure transpiled code behaves identically
4. **Use idiomatic Rust** - Follow Rust naming and style conventions
5. **Document edge cases** - Add comments for complex transpilations

## See Also

- [Transpiler Guide](../../a2r-transpiler-guide.md)
- [Language Reference](../../lang/README.md)
- [C Transpiler](../a2c/README.md)
