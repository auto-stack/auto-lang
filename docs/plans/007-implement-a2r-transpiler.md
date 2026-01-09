# 007 - Implement Auto to Rust Transpiler (a2r)

## Objective

Implement a comprehensive Auto to Rust transpiler (a2r) in `crates/auto-lang/src/trans/rust.rs`, following the architecture and patterns established by the existing Auto to C transpiler (a2c).

## Current State

**Existing Implementation:**
- **C transpiler** (`c.rs`): Fully implemented with 2000+ lines, supporting:
  - All expression types (40+ variants)
  - All statement types (16+ variants)
  - Type declarations, structs, methods, enums, unions, tags
  - Pattern matching (is statements)
  - Control flow (if/else, for loops)
  - Function signatures and bodies
  - Type inference and scope management
  - Dual-file generation (.c and .h)

- **Rust transpiler** (`rust.rs`): Stub implementation with ~190 lines
  - ✅ Phase 1 complete: Core infrastructure
  - ✅ Type mapping system (rust_type_name)
  - ✅ Basic expression translation (expr)
  - ⏳ Phase 2-6: Pending implementation

**Test Infrastructure:**
- a2c test framework: `test/a2c/` with 25+ test cases
- No a2r tests exist yet
- Test naming: 000-099 (core features), 100-199 (stdlib)

## Progress

### ✅ Phase 1: Core Infrastructure (COMPLETED)

**Step 1.1: RustTrans Structure** ✅
- Implemented `RustTrans` struct with:
  - `indent`: usize - Indentation tracking
  - `uses`: HashSet<AutoStr> - Use statement tracking
  - `name`: AutoStr - Output file name
  - `scope`: Shared<Universe> - Symbol table
  - `edition`: RustEdition (E2021/E2024)
- Helper methods: `indent()`, `dedent()`, `print_indent()`
- Constructor: `new()`, setters: `set_scope()`, `set_edition()`

**Step 1.2: Type Mapping** ✅
- Implemented `rust_type_name()` method
- Complete type mappings:
  - Primitives: Byte→u8, Int→i32, Uint→u32, USize→usize
  - Floats: Float/Double→f64, Bool→bool, Char→char
  - Strings: Str(_)→String, CStr→&str
  - Arrays: Array(arr)→[T; N]
  - Pointers: Ptr(ptr)→&T or Box<T> (smart detection)
  - User types: User(usr)→struct name
  - Enums/Unions/Tags → enum name
  - Void → (), Unknown → /* unknown */

**Step 1.3: Expression Translation** ✅
- Implemented `expr()` method supporting:
  - Literals: Int, Uint, I8, U8, I64, Byte, Float, Double, Bool, Char
  - Strings: Str, CStr (with proper escaping)
  - Identifiers: Ident, GenName
  - Null values: Nil, Null → None
  - Binary operators: Bina (including Dot, Range, RangeEq, arithmetic, comparison)
  - Unary operators: Unary (-, !, *, &)
  - Arrays: Array → [1, 2, 3]
  - Index: Index → arr[i]
  - Stubs for Call, If (to be implemented in Phase 2)

### ⏳ Phase 2-6: Pending Implementation

See detailed implementation steps below.

## Rust-Specific Challenges and Design Decisions

### 1. No Header Files
**Challenge**: C uses dual-file generation (.c and .h), Rust uses single-file modules.

**Solution**:
- Only generate `.rs` files (ignore `sink.header`)
- Use `mod` declarations for module organization
- Auto-generate `mod.rs` for multi-file projects

### 2. Type System Differences
**Challenge**: AutoLang types need idiomatic Rust mapping.

**Type Mapping**:
```rust
AutoLang        -> Rust
-----------------------------
Byte            -> u8
Int             -> i32
Uint            -> u32
I8/U8/I64       -> i8/u8/i64
USize           -> usize
Float/Double    -> f64
Bool            -> bool
Char            -> char
Str(len)        -> String (heap-allocated)
CStr            -> &str (string slice)
Array[T, N]     -> [T; N] (fixed-size) or Vec<T> (dynamic)
Ptr(T)          -> &T (reference) or Box<T> (owned)
User(TypeDecl)  -> struct (with public fields)
Enum(EnumDecl)  -> enum (with C-like variants)
Union(Union)    -> enum (Rust-style tagged union)
Tag(Tag)        -> enum (with enum data variant)
Void            -> () (unit type)
```

### 3. Pattern Matching Translation
**Challenge**: AutoLang `is` statements vs Rust `match` expressions.

**Translation Strategy**:
```auto
# AutoLang
is x {
    1 => { print("one") }
    2 => { print("two") }
    else => { print("other") }
}
```

```rust
// Rust
match x {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("other"),
}
```

**Special Cases**:
- `EqBranch` → specific match arm
- `IfBranch` → match guard (`val if condition =>`)
- `ElseBranch` → wildcard `_` pattern

### 4. Ownership and Borrowing
**Challenge**: AutoLang has no ownership concept, Rust requires explicit ownership/borrowing.

**Strategies**:
1. **Prefer references for function parameters** (`&T` instead of `T`)
2. **Use `clone()` when ownership transfer is needed**
3. **Add `mut` keywords where mutable borrows are required**
4. **Generate `#[allow(dead_code)]` for unused fields (common in transpiled code)**

### 5. Method Call Syntax
**Challenge**: C uses `TypeName_method(&instance, args)`, Rust uses `instance.method(args)`.

**Translation**:
```auto
# AutoLang
file.read_text()
```

```rust
// Rust (idiomatic)
file.read_text()
```

**Implementation**: Direct translation (unlike C's manual `self` parameter)

### 6. Standard Library
**Challenge**: AutoLang `use auto.io: say` needs Rust equivalent.

**Strategy**:
1. **Map Auto stdlib to Rust stdlib**:
   ```auto
   use auto.io: say     -> use crate::io::say;
   print("hello")       -> println!("hello");
   ```
2. **Generate wrapper functions** when direct mapping isn't possible
3. **Support both**: Check if `std/io` module exists, else use built-in `println!`

### 7. String Types
**Challenge**: AutoLang has `Str` (owned) and `CStr` (borrowed), Rust has `String` and `&str`.

**Decision**:
- Auto `Str` → Rust `String` (heap-allocated)
- Auto `CStr` → Rust `&'static str` (static string slice)
- Use `format!` for string concatenation
- Use `.as_str()` for string slices

### 8. Main Function
**Challenge**: AutoLang auto-generates `main()`, Rust requires specific signatures.

**Supported Signatures**:
```rust
fn main() { ... }                          // No return
fn main() -> i32 { ... }                   // Int return
fn main() { return 0; }                    // Explicit return
```

**Decision**: Match AutoLang behavior - generate `fn main()` or `fn main() -> i32` based on AST.

### 9. Struct Initialization
**Challenge**: C uses designated initializers `.field = value`, Rust uses `Struct { field: value }`.

**Translation**:
```auto
# AutoLang
Point { x: 10, y: 20 }
```

```rust
// Rust
Point { x: 10, y: 20 }
```

**Benefit**: Syntax is nearly identical!

### 10. Error Handling
**Challenge**: Rust uses `Result<T, E>`, AutoLang has no built-in error handling.

**Strategy**:
1. **Phase 1**: Ignore errors (panic on failure)
2. **Phase 2**: Generate `unwrap()` calls for Result types
3. **Phase 3**: Optionally add `?` operator for propagation

## Implementation Plan

### Phase 2: Statement Translation

#### Step 2.1: Implement Statement Dispatcher

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<bool> {
        match stmt {
            Stmt::Expr(expr) => {
                self.expr(expr, &mut sink.body)?;
                Ok(true)
            }

            Stmt::Store(store) => {
                self.store(store, &mut sink.body)?;
                sink.body.write(b";")?;
                Ok(true)
            }

            Stmt::Fn(fn_decl) => {
                self.fn_decl(fn_decl, sink)?;
                Ok(true)
            }

            Stmt::For(for_stmt) => {
                self.for_stmt(for_stmt, sink)?;
                Ok(true)
            }

            Stmt::If(if_) => {
                self.if_stmt(if_, sink)?;
                Ok(true)
            }

            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, sink)?;
                Ok(true)
            }

            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, &mut sink.body)?;
                Ok(true)
            }

            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, sink)?;
                Ok(true)
            }

            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, sink)?;
                Ok(true)
            }

            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    sink.body.write(b"\n")?;
                }
                Ok(true)
            }

            _ => Err(format!("Rust Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }
}
```

#### Step 2.2: Implement Variable Declaration

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // Type inference for Unknown types
        if matches!(store.ty, Type::Unknown) {
            if let Some(inferred_type) = self.infer_expr_type(&store.expr) {
                self.scope.borrow_mut().update_store_type(&store.name, inferred_type.clone());
                // Rust can use type inference with 'let'
                write!(out, "let {} = ", store.name)?;
            } else {
                write!(out, "let {}: /* unknown */ = ", store.name)?;
            }
        } else {
            // Explicit type annotation
            match store.kind {
                StoreKind::Let => {
                    write!(out, "let {}: {} = ", store.name, self.rust_type_name(&store.ty))?;
                }
                StoreKind::Mut => {
                    write!(out, "let mut {}: {} = ", store.name, self.rust_type_name(&store.ty))?;
                }
                _ => {
                    write!(out, "let {}: {} = ", store.name, self.rust_type_name(&store.ty))?;
                }
            }
        }

        self.expr(&store.expr, out)?;
        Ok(())
    }

    // Reuse C transpiler's type inference logic
    fn infer_expr_type(&mut self, expr: &Expr) -> Option<Type> {
        // ... (similar to CTrans::infer_expr_type)
    }
}
```

#### Step 2.3: Implement Function Declaration

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;

        // Function signature
        write!(out, "fn {}", fn_decl.name)?;

        // Generics (if any)
        if !fn_decl.params.is_empty() && fn_decl.params.iter().any(|p| matches!(p.ty, Type::Unknown)) {
            write!(out, "<T>")?;
        }

        // Parameters
        write!(out, "(")?;
        for (i, param) in fn_decl.params.iter().enumerate() {
            // Prefer references for parameters (idiomatic Rust)
            let param_ty = if should_use_reference(&param.ty) {
                format!("&{}", self.rust_type_name(&param.ty))
            } else {
                self.rust_type_name(&param.ty)
            };

            write!(out, "{}: {}", param.name, param_ty)?;
            if i < fn_decl.params.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")")?;

        // Return type
        if !matches!(fn_decl.ret, Type::Void) {
            write!(out, " -> {}", self.rust_type_name(&fn_decl.ret))?;
        }

        // Function body
        write!(out, " ")?;
        self.scope.borrow_mut().enter_fn(fn_decl.name.clone());
        self.body(&fn_decl.body, sink, &fn_decl.ret, "")?;
        self.scope.borrow_mut().exit_fn();

        sink.body.write(b"\n")?;
        Ok(())
    }

    fn should_use_reference(ty: &Type) -> bool {
        // Use references for: structs, enums, large types
        matches!(ty, Type::User(_) | Type::Enum(_) | Type::Tag(_) | Type::Union(_))
    }
}
```

#### Step 2.4: Implement If Statement

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
        for (i, branch) in if_.branches.iter().enumerate() {
            if i == 0 {
                sink.body.write(b"if ")?;
            } else {
                sink.body.write(b" else if ")?;
            }

            sink.body.write(b"{ ")?;
            self.expr(&branch.cond, &mut sink.body)?;
            sink.body.write(b" ")?;
            self.body(&branch.body, sink, &Type::Void, "")?;
            sink.body.write(b" }")?;
        }

        if let Some(else_body) = &if_.else_ {
            sink.body.write(b" else ")?;
            self.body(else_body, sink, &Type::Void, "")?;
        }

        Ok(())
    }
}
```

#### Step 2.5: Implement For Loop

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn for_stmt(&mut self, for_stmt: &For, sink: &mut Sink) -> AutoResult<()> {
        match &for_stmt.iter {
            Iter::Named(name) => {
                // Range iteration: for x in start..end
                if let Expr::Range(range) = &for_stmt.range {
                    sink.body.write(b"for ")?;
                    sink.body.write(name.as_bytes())?;
                    sink.body.write(b" in ")?;
                    self.expr(&range.start, &mut sink.body)?;
                    sink.body.write(b"..")?;
                    self.expr(&range.end, &mut sink.body)?;
                    sink.body.write(b" ")?;
                    self.body(&for_stmt.body, sink, &Type::Void, "")?;
                }
            }
            Iter::Ever => {
                // Infinite loop: loop { body }
                sink.body.write(b"loop ")?;
                self.body(&for_stmt.body, sink, &Type::Void, "")?;
            }
            Iter::Call(call) => {
                // Iterator-based: while iter.next().is_some()
                sink.body.write(b"while ")?;
                self.expr(&Expr::Call(call.clone()), &mut sink.body)?;
                sink.body.write(b" ")?;
                self.body(&for_stmt.body, sink, &Type::Void, "")?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### Step 2.6: Implement Is Statement (Pattern Matching)

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn is_stmt(&mut self, is_stmt: &Is, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"match ")?;
        self.expr(&is_stmt.target, &mut sink.body)?;
        sink.body.write(b" {\n")?;
        self.indent();

        for branch in &is_stmt.branches {
            self.print_indent(&mut sink.body)?;

            match branch {
                IsBranch::EqBranch(expr, body) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b" => ")?;
                    self.body(body, sink, &Type::Void, "")?;
                    sink.body.write(b",\n")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b" if true => ")?;  // Placeholder for guard
                    self.body(body, sink, &Type::Void, "")?;
                    sink.body.write(b",\n")?;
                }
                IsBranch::ElseBranch(body) => {
                    sink.body.write(b"_ => ")?;
                    self.body(body, sink, &Type::Void, "")?;
                    sink.body.write(b",\n")?;
                }
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }
}
```

#### Step 2.7: Implement Type Declaration (Struct)

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;

        // Struct definition
        write!(out, "struct {} {{", type_decl.name)?;

        if !type_decl.members.is_empty() {
            sink.body.write(b"\n")?;
            self.indent();

            for member in &type_decl.members {
                self.print_indent(out)?;
                write!(out, "{}: {},", member.name, self.rust_type_name(&member.ty))?;
                sink.body.write(b"\n")?;
            }

            self.dedent();
            self.print_indent(out)?;
        }

        sink.body.write(b"}\n\n")?;

        // Method implementations
        for method in &type_decl.methods {
            // Method signature
            write!(out, "impl {} {{", type_decl.name)?;
            sink.body.write(b"\n")?;
            self.indent();

            self.print_indent(out)?;
            write!(out, "fn {}(&self", method.name)?;

            // Parameters
            for (i, param) in method.params.iter().enumerate() {
                write!(out, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
            }
            write!(out, ")")?;

            // Return type
            if !matches!(method.ret, Type::Void) {
                write!(out, " -> {}", self.rust_type_name(&method.ret))?;
            }

            // Method body
            sink.body.write(b" ")?;
            self.body(&method.body, sink, &method.ret, "")?;
            sink.body.write(b"\n")?;

            self.dedent();
            sink.body.write(b"}\n\n")?;
        }

        Ok(())
    }
}
```

#### Step 2.8: Implement Enum Declaration

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"enum ")?;
        sink.body.write(enum_decl.name.as_bytes())?;
        sink.body.write(b" {\n")?;
        self.indent();

        for (i, item) in enum_decl.items.iter().enumerate() {
            self.print_indent(&mut sink.body)?;
            sink.body.write(format!("{} = {},", item.name, item.value).as_bytes())?;
            sink.body.write(b"\n")?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n\n")?;
        Ok(())
    }
}
```

#### Step 2.9: Implement Use Statement

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn use_stmt(&mut self, use_stmt: &Use, out: &mut impl Write) -> AutoResult<()> {
        match use_stmt.kind {
            UseKind::Auto => {
                // Map Auto stdlib to Rust modules
                for path in &use_stmt.paths {
                    let rust_path = path.replace("auto.", "crate::");
                    write!(out, "use {};", rust_path)?;
                    self.uses.insert(path.clone());
                }
            }
            UseKind::C => {
                // Ignore C imports for Rust transpiler
            }
            UseKind::Rust => {
                // Direct Rust imports
                for path in &use_stmt.paths {
                    write!(out, "use {};", path)?;
                    self.uses.insert(path.clone());
                }
            }
        }
        Ok(())
    }
}
```

### Phase 3: Body and Block Management

#### Step 3.1: Implement Body Helper

**File**: `rust.rs` (add method)

```rust
impl RustTrans {
    fn body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        ret_type: &Type,
        insert: &str,
    ) -> AutoResult<()> {
        let has_return = !matches!(ret_type, Type::Void);
        self.scope.borrow_mut().enter_scope();
        sink.body.write(b"{\n")?;
        self.indent();

        // Insert initialization code if provided
        if !insert.is_empty() {
            self.print_indent(&mut sink.body)?;
            sink.body.write(insert.as_bytes())?;
        }

        // Process statements
        for (i, stmt) in body.stmts.iter().enumerate() {
            if !matches!(stmt, Stmt::EmptyLine(_)) {
                self.print_indent(&mut sink.body)?;
            }

            let is_last = i == body.stmts.len() - 1;

            if is_last && has_return && self.is_returnable(stmt) {
                // Last statement: no semicolon (expression position)
                self.stmt(stmt, sink)?;
            } else {
                // Regular statement: add semicolon
                self.stmt(stmt, sink)?;
                if !is_last {
                    sink.body.write(b";")?;
                }
            }
            sink.body.write(b"\n")?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        self.scope.borrow_mut().exit_scope();
        Ok(())
    }

    fn is_returnable(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => match expr {
                Expr::Call(_) => true,
                Expr::If(_) => true,
                Expr::Block(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
}
```

### Phase 4: Main Orchestration

#### Step 4.1: Implement trans() Method

**File**: `rust.rs` (add method)

```rust
impl Trans for RustTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Phase 1: Emit file header
        sink.body.write(b"//! Auto-generated Rust code\n\n")?;

        // Phase 2: Emit use statements
        if !self.uses.is_empty() {
            let mut uses: Vec<_> = self.uses.iter().cloned().collect();
            uses.sort();
            for use_stmt in uses {
                sink.body.write(format!("use {};\n", use_stmt).as_bytes())?;
            }
            sink.body.write(b"\n")?;
        }

        // Phase 3: Split into declarations and main
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main: Vec<Stmt> = Vec::new();

        for stmt in ast.stmts.into_iter() {
            if stmt.is_decl() {
                decls.push(stmt);
            } else {
                main.push(stmt);
            }
        }

        // Phase 4: Generate declarations
        for decl in &decls {
            self.stmt(decl, sink)?;
            sink.body.write(b"\n")?;
        }

        // Phase 5: Generate main function if needed
        if !main.is_empty() {
            sink.body.write(b"fn main() ")?;
            self.scope.borrow_mut().enter_fn("main".into());

            let has_return = main.iter().any(|s| self.is_returnable(s));
            if has_return {
                sink.body.write(b"-> i32 ")?;
            }

            sink.body.write(b"{\n")?;
            self.indent();

            for (i, stmt) in main.iter().enumerate() {
                self.print_indent(&mut sink.body)?;

                let is_last = i == main.len() - 1;
                if is_last && has_return && self.is_returnable(stmt) {
                    sink.body.write(b"return ")?;
                    self.stmt(stmt, sink)?;
                } else {
                    self.stmt(stmt, sink)?;
                    sink.body.write(b";")?;
                }
                sink.body.write(b"\n")?;
            }

            self.dedent();
            sink.body.write(b"}\n")?;
            self.scope.borrow_mut().exit_fn();
        }

        Ok(())
    }
}
```

#### Step 4.2: Add Helper Functions

**File**: `rust.rs` (add module-level functions)

```rust
/// Transpile AutoLang code to Rust
pub fn transpile_rust(name: impl Into<AutoStr>, code: &str)
    -> AutoResult<(Sink, Shared<Universe>)>
{
    let name = name.into();
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    let mut out = Sink::new(name.clone());
    let mut transpiler = RustTrans::new(name);
    transpiler.scope = parser.scope.clone();
    transpiler.trans(ast, &mut out)?;

    Ok((out, parser.scope.clone()))
}

/// Transpile code fragment for testing
pub fn transpile_part(code: &str) -> AutoResult<AutoStr> {
    let mut transpiler = RustTrans::new("part".into());
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new(AutoStr::from(""));
    transpiler.code(ast, &mut out)?;
    Ok(String::from_utf8(out.body).unwrap().into())
}
```

### Phase 5: CLI Integration

#### Step 5.1: Add CLI Command

**File**: `crates/auto/src/main.rs`

```rust
#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(about = "Transpile Auto to C")]
    C { path: String },

    #[command(about = "Transpile Auto to Rust")]
    Rust { path: String },  // NEW
    // ... other commands
}

// In main()
match args.command {
    Some(Commands::Rust { path }) => {
        let r = auto_lang::trans_rust(path.as_str())?;
        println!("{}", r);
    }
    // ... other commands
}
```

#### Step 5.2: Add Public API

**File**: `crates/auto-lang/src/lib.rs`

```rust
/// Transpile AutoLang file to Rust
pub fn trans_rust(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;
    let (sink, _) = trans::rust::transpile_rust(
        std::path::Path::new(path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap(),
        &code,
    )?;

    let src = sink.done()?;
    let output_path = path.replace(".at", ".rs");
    std::fs::write(&output_path, src)?;

    Ok(format!("Rust code written to: {}", output_path))
}
```

### Phase 6: Test Infrastructure

#### Step 6.1: Create Test Directory Structure

**File**: `test/a2r/`

```bash
mkdir -p test/a2r/000_hello
mkdir -p test/a2r/001_sqrt
mkdir -p test/a2r/002_array
# ... (mirror a2c structure)
```

#### Step 6.2: Implement Test Framework

**File**: `crates/auto-lang/src/trans/rust.rs` (in tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_a2r(case: &str) -> AutoResult<()> {
        use std::fs::read_to_string;
        use std::path::PathBuf;

        // Parse test case name: "000_hello" -> "hello"
        let parts: Vec<&str> = case.split("_").collect();
        let name = parts[1..].join("_");

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2r/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        let exp_path = format!("test/a2r/{}/{}.expected.rs", case, name);
        let exp_path = d.join(exp_path);
        let expected = if !exp_path.is_file() {
            "".to_string()
        } else {
            read_to_string(exp_path.as_path())?
        };

        let (mut rcode, _) = transpile_rust(name, &src)?;
        let rs_code = rcode.done()?;

        if rs_code != expected.as_bytes() {
            // Generate .wrong.rs for comparison
            let gen_path = format!("test/a2r/{}/{}.wrong.rs", case, name);
            let gen_path = d.join(gen_path);
            std::fs::write(&gen_path, rs_code)?;
        }

        assert_eq!(String::from_utf8_lossy(rs_code), expected);
        Ok(())
    }

    #[test]
    fn test_000_hello_rust() {
        test_a2r("000_hello").unwrap();
    }

    #[test]
    fn test_001_sqrt_rust() {
        test_a2r("001_sqrt").unwrap();
    }

    // ... add more tests
}
```

#### Step 6.3: Create Initial Test Cases

**File**: `test/a2r/000_hello/hello.at`
```auto
print("hello, world!")
```

**File**: `test/a2r/000_hello/hello.expected.rs`
```rust
fn main() {
    println!("hello, world!");
}
```

**File**: `test/a2r/001_sqrt/sqrt.at`
```auto
fn sqrt(x double) double { x * x }
sqrt(2.0)
```

**File**: `test/a2r/001_sqrt/sqrt.expected.rs`
```rust
fn sqrt(x: f64) -> f64 {
    x * x
}

fn main() -> i32 {
    return sqrt(2.0);
}
```

#### Step 6.4: Run Initial Tests

```bash
cargo test -p auto-lang test_000_hello_rust
cargo test -p auto-lang test_001_sqrt_rust
```

## Testing Strategy

### Incremental Testing Approach

**Phase 1: Core Expressions** (tests 001-005)
- Literals (int, float, bool, char, str)
- Binary operators
- Unary operators
- Arrays
- Index operations

**Phase 2: Control Flow** (tests 010-014)
- If statements
- For loops
- Is/pattern matching
- Break statements

**Phase 3: Functions** (tests 020-029)
- Function declarations
- Function calls
- Parameters
- Return types
- Methods

**Phase 4: Types** (tests 030-039)
- Struct declarations
- Struct initialization
- Enum declarations
- Type aliases

**Phase 5: Advanced Features** (tests 040-049)
- Unions
- Tags
- Pattern matching with guards
- Closures (lambdas)

**Phase 6: Standard Library** (tests 100-199)
- I/O operations
- File operations
- String operations

### Test Creation Process

For each test case:

1. **Create .at input file** (or reuse from a2c)
2. **Generate initial Rust output** (run test, creates .wrong.rs)
3. **Review and fix .wrong.rs** to create idiomatic Rust
4. **Rename to .expected.rs** (once correct)
5. **Fix transpiler** to match expected output

## Critical Files to Create/Modify

### New Files
1. **`crates/auto-lang/src/trans/rust.rs`** (~2000 lines)
   - Main Rust transpiler implementation
   - All methods: expr(), stmt(), fn_decl(), etc.
   - Test framework (test_a2r, test_XXX_rust)

2. **`test/a2r/000_hello/hello.at`**
3. **`test/a2r/000_hello/hello.expected.rs`**
4. **`test/a2r/001_sqrt/sqrt.at`**
5. **`test/a2r/001_sqrt/sqrt.expected.rs`**
6. **...** (additional test cases)

### Modified Files
1. **`crates/auto/src/main.rs`**
   - Add `Rust` command variant
   - Add match arm for Rust transpilation

2. **`crates/auto-lang/src/lib.rs`**
   - Add `trans_rust()` public function
   - Export `trans::rust` module

3. **`crates/auto-lang/src/trans/mod.rs`**
   - Ensure `rust` module is exported (if not already)

4. **`crates/auto-lang/src/parser.rs`** (optional)
   - Add `CompileDest::TransRust` variant (if needed for scope tracking)

## Success Criteria

### Phase 1 Success (Core Features) ✅
- [x] Can transpile basic expressions (literals, operators)
- [x] Type mapping implemented
- [x] Core infrastructure complete
- [ ] Tests 000-005 pass

### Phase 2 Success (Advanced Features)
- [ ] Can transpile variable declarations (let, mut)
- [ ] Can transpile if/else statements
- [ ] Can transpile for loops
- [ ] Can transpile simple functions
- [ ] Tests 000-005 pass

### Phase 3 Success (Complete Language)
- [ ] Can transpile struct declarations and methods
- [ ] Can transpile enum declarations
- [ ] Can transpile is/pattern matching statements
- [ ] Can transpile union and tag types
- [ ] Tests 006-015 pass

### Phase 4 Success (Integration)
- [ ] All 40+ expression types supported
- [ ] All 16+ statement types supported
- [ ] All test cases from a2c (000-015) have a2r equivalents
- [ ] Generated Rust code compiles with `cargo build`
- [ ] Generated Rust code is idiomatic (passes clippy)

### Phase 5 Success (Final)
- [ ] CLI command `auto rust file.at` works
- [ ] Generated .rs files are written to disk
- [ ] Standard library tests (100-199) pass
- [ ] Documentation complete

## Verification Steps

### Manual Testing
```bash
# 1. Test basic transpilation
echo 'print("hello")' > test.at
cargo run -- test.at rust
cat test.rs

# 2. Test with actual file
cargo run -- examples/hello.at rust
rustc examples/hello.rs -o hello
./hello

# 3. Run test suite
cargo test -p auto-lang -- trans

# 4. Check generated code quality
cargo clippy --examples  # On generated Rust files
```

### Automated Testing
```bash
# Run all a2r tests
cargo test -p auto-lang test_0 --lib

# Run specific test
cargo test -p auto-lang test_000_hello_rust --lib

# Run all transpiler tests (a2c + a2r)
cargo test -p auto-lang --trans
```

## Timeline Estimate

**Phase 1 (Foundation)**: ✅ COMPLETED
- Steps 1.1-1.3: Core infrastructure and basic expressions

**Phase 2 (Statements)**: 5-6 days
- Steps 2.1-2.9: All statement types and control flow

**Phase 3 (Bodies)**: 1-2 days
- Step 3.1: Body and block management

**Phase 4 (Main)**: 2-3 days
- Steps 4.1-4.2: Main orchestration and helpers

**Phase 5 (CLI)**: 1 day
- Steps 5.1-5.2: CLI integration and public API

**Phase 6 (Tests)**: 4-5 days
- Steps 6.1-6.4: Test infrastructure and initial tests

**Total**: ~16-21 days for full implementation with comprehensive test coverage.
**Progress**: Phase 1 complete (~3 days), remaining ~13-18 days

## References

- **C Transpiler**: `crates/auto-lang/src/trans/c.rs` (2025 lines) - Reference architecture
- **AST Definitions**: `crates/auto-lang/src/ast/` - All node types
- **Type System**: `crates/auto-lang/src/ast/types.rs` - Type enum and metadata
- **Existing Tests**: `test/a2c/` - Test case examples and expectations
- **Current Implementation**: `crates/auto-lang/src/trans/rust.rs` (190 lines) - Work in progress
