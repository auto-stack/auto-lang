# Fix AtomWriter Implementations to Match Test Expectations

## Objective

Fix AtomWriter implementations to produce output matching the hand-written test expectations in `crates/auto-lang/test/ast/*.test.md`.

## Current State

Tests 01-05 have been updated with hand-written expectations, but AtomWriter implementations produce different output. Running `cargo test test_0` shows multiple failures:

1. **test_01_literals**: FStr with binary expr - operators have quotes
2. **test_02_exprs**: Binary operators have quotes
3. **test_03_functions**: Parameters have verbose format, operators have quotes
4. **test_04_controls**: If statement has incorrect newlines, operators have quotes
5. **test_05_types**: Multiple issues (TypeDecl format, Member format, missing return type, struct constructor detection)

## Root Causes

### 1. Binary Operator Format
**File**: `crates/auto-lang/src/ast.rs:456-476`

Current output quotes operators: `bina('+', 1, 2)`
Expected output: `bina(+, 1, 2)`

### 2. If Statement Formatting
**File**: `crates/auto-lang/src/ast/if_.rs:29-39`

Current output has newlines:
```
if { bina('>', x, 1) { call print ("greater than 1") }
else { call print ("less than or equal to 1") }
 }
```

Expected: Single line with proper spacing
```
if { bina('>', x, 1) { call print ("greater than 1") } else { call print ("less than or equal to 1") } }
```

### 3. Function Parameter Format
**File**: `crates/auto-lang/src/ast/fun.rs:116-130`

Current output: `param(name("a"), type(int))`
Expected output: `(a, int)`

### 4. Function Return Type Missing
**File**: `crates/auto-lang/src/ast/fun.rs:150-200`

Current output: `fn new_point ((x, int), (y, int)) { ... }`
Expected output: `fn new_point ((x, int), (y, int)) Point { ... }`

### 5. Struct Constructor Detection
**File**: `crates/auto-lang/src/ast/fun.rs` (body output)

Current output: `call Point (x, y)`
Expected output: `node Point (x, y)`

**Challenge**: When parsing a type's methods, the TypeDecl is not yet in scope (added at parser.rs:2138 after full parsing). Need to detect struct constructors differently.

### 6. Member Format
**File**: `crates/auto-lang/src/ast/types.rs:436-446`

Current output: `member(name("x"), type(int), ...)`
Expected output: `member(x, int, ...)`

### 7. TypeDecl Format
**File**: `crates/auto-lang/src/ast/types.rs:470-482`

Current output: `type-decl(name("Point")) member(...) ...`
Expected output: `type Point { member(x, int); member(y, int) }`

## Implementation Plan

### Step 1: Fix Binary Operator Output

**File**: `crates/auto-lang/src/ast.rs` (Expr::Bina match arm, ~line 456)

Remove quotes from operator symbols:

```rust
Expr::Bina(l, op, r) => {
    // Special case for dot operator (field access)
    if *op == auto_val::Op::Dot {
        write!(f, "bina({}, {})", l.to_atom_str(), r.to_atom_str())?;
    } else {
        let op_str = match op {
            auto_val::Op::Add => "+",
            auto_val::Op::Sub => "-",
            auto_val::Op::Mul => "*",
            auto_val::Op::Div => "/",
            auto_val::Op::Eq => "==",
            auto_val::Op::Neq => "!=",
            auto_val::Op::Lt => "<",
            auto_val::Op::Le => "<=",
            auto_val::Op::Gt => ">",
            auto_val::Op::Ge => ">=",
            auto_val::Op::AddEq => "+=",
            auto_val::Op::SubEq => "-=",
            auto_val::Op::MulEq => "*=",
            auto_val::Op::DivEq => "/=",
            auto_val::Op::Range => "..",
            auto_val::Op::RangeEq => "..=",
            _ => "?",
        };
        write!(f, "bina({}, {}, {})", op_str, l.to_atom_str(), r.to_atom_str())?;
    }
}
```

### Step 2: Fix If Statement Formatting

**File**: `crates/auto-lang/src/ast/if_.rs:29-39`

Remove newlines, add proper spacing:

```rust
impl AtomWriter for If {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "if {{")?;
        for branch in &self.branches {
            write!(f, " {}", branch.to_atom_str())?;
        }
        if let Some(else_body) = &self.else_ {
            write!(f, " else {{ {} }}", else_body.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}
```

### Step 3: Fix Function Parameter Format

**File**: `crates/auto-lang/src/ast/fun.rs:116-130`

Change from verbose to simplified format:

```rust
impl AtomWriter for Param {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "({}, {})", self.name, self.ty.to_atom_str())?;
        if let Some(default) = &self.default {
            write!(f, " = {}", default.to_atom_str())?;
        }
        Ok(())
    }
}
```

### Step 4: Add Function Return Type

**File**: `crates/auto-lang/src/ast/fun.rs:150-200`

Modify `Fn::write_atom` to include return type:

```rust
impl AtomWriter for Fn {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self.kind {
            FnKind::Lambda => {
                // Lambda format: lambda(x, y) { body }
                write!(f, "lambda(")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.name)?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {{")?;
                if !self.body.stmts.is_empty() {
                    write!(f, " {}", self.body.to_atom_str())?;
                }
                write!(f, " }}")?;
            }
            FnKind::C => {
                // C function: fn.c name (param, type) return_type
                write!(f, "fn.c {} (", self.name)?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.to_atom_str())?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {}", self.ret.to_atom_str())?;
            }
            FnKind::Normal => {
                // Normal function: fn name ((param, type), ...) return_type { body }
                write!(f, "fn {} (", self.name)?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.to_atom_str())?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                
                // Add return type if not void
                if !matches!(self.ret, Type::Void) {
                    write!(f, " {}", self.ret.to_atom_str())?;
                }
                
                write!(f, " {{")?;
                if !self.body.stmts.is_empty() {
                    write!(f, " {}", self.body.to_atom_str())?;
                }
                write!(f, " }}")?;
            }
        }
        Ok(())
    }
}
```

### Step 5: Detect Struct Constructors

**File**: `crates/auto-lang/src/ast/fun.rs` (in body writing logic)

**Strategy**: Check if a Call expression's function name matches a Type::User in scope.

Add helper method to detect struct constructors:

```rust
impl Fn {
    /// Check if a call expression is a struct constructor
    fn is_struct_constructor(&self, call: &Call, scope: &Universe) -> bool {
        if let Expr::Ident(name) = call.name.as_ref() {
            // Look up the identifier in scope
            if let Some(meta) = scope.get(name.as_ref()) {
                if let Meta::Type(Type::User(_)) = meta {
                    return true;
                }
            }
        }
        false
    }
}
```

Then modify body output to use `node` instead of `call` for struct constructors:

```rust
// In Fn::write_atom, when writing body statements
for stmt in &self.body.stmts {
    if let Stmt::Expr(Expr::Call(call)) = stmt {
        if self.is_struct_constructor(call, scope) {
            write!(f, "node {} (", call_name)?;
            // ... write args ...
            write!(f, ")")?;
        } else {
            write!(f, "{}", stmt.to_atom_str())?;
        }
    } else {
        write!(f, "{}", stmt.to_atom_str())?;
    }
}
```

**Challenge**: The Fn's AtomWriter implementation doesn't have access to the Universe/scope. We'll need to:

1. Pass scope context through the AtomWriter chain, OR
2. Use a heuristic: check if the call name starts with uppercase letter and has no namespace prefix

**Fallback approach** (simpler, may be sufficient for tests):
```rust
// In body output, check if call is a potential struct constructor
if let Stmt::Expr(Expr::Call(call)) = stmt {
    if let Expr::Ident(name) = call.name.as_ref() {
        // Heuristic: starts with uppercase and no namespace
        let is_struct = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        if is_struct {
            write!(f, "node {} (", name)?;
            for (i, arg) in call.args.args.iter().enumerate() {
                match arg {
                    Arg::Pos(expr) => write!(f, "{}", expr.to_atom_str())?,
                    Arg::Name(name) => write!(f, "{}", name)?,
                    Arg::Pair(name, expr) => write!(f, "{}: {}", name, expr.to_atom_str())?,
                }
                if i < call.args.args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, ")")?;
        } else {
            write!(f, "{}", stmt.to_atom_str())?;
        }
    } else {
        write!(f, "{}", stmt.to_atom_str())?;
    }
} else {
    write!(f, "{}", stmt.to_atom_str())?;
}
```

### Step 6: Fix Member Format

**File**: `crates/auto-lang/src/ast/types.rs:436-446`

Simplify format:

```rust
impl AtomWriter for Member {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "member({}, {})", self.name, self.ty.to_atom_str())?;
        if let Some(value) = &self.value {
            write!(f, ", {}", value.to_atom_str())?;
        }
        Ok(())
    }
}
```

### Step 7: Fix TypeDecl Format

**File**: `crates/auto-lang/src/ast/types.rs:470-482`

Match expected format:

```rust
impl AtomWriter for TypeDecl {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "type {} {{", self.name)?;
        for (i, member) in self.members.iter().enumerate() {
            write!(f, " {}", member.to_atom_str())?;
            if i < self.members.len() - 1 {
                write!(f, ";")?;
            }
        }
        
        // Add methods if present
        for method in &self.methods {
            write!(f, " {}", method.to_atom_str())?;
        }
        
        write!(f, " }}")?;
        Ok(())
    }
}
```

### Step 8: Handle Missing Test Files

Tests 06-11 don't have corresponding `.test.md` files (they're in `tmp/` and incomplete).

**Action**: Comment out these test functions in `ast.rs:1129-1156`:

```rust
// #[test]
// fn test_06_declarations() {
//     run_markdown_test_file("06_declarations.test.md");
// }
// ... (same for 07-11)
```

## Critical Files to Modify

1. `crates/auto-lang/src/ast.rs` (~line 456) - Fix Expr::Bina operator format
2. `crates/auto-lang/src/ast/if_.rs:29` - Fix If statement formatting
3. `crates/auto-lang/src/ast/fun.rs:116` - Fix Param format
4. `crates/auto-lang/src/ast/fun.rs:150` - Fix Fn return type and struct constructor detection
5. `crates/auto-lang/src/ast/types.rs:436` - Fix Member format
6. `crates/auto-lang/src/ast/types.rs:470` - Fix TypeDecl format
7. `crates/auto-lang/src/ast.rs:1129` - Comment out tests 06-11

## Testing Strategy

Run tests incrementally after each fix:

```bash
# After Step 1: Binary operators
cargo test test_01_literals --lib
cargo test test_02_exprs --lib

# After Steps 2-3: If and Param
cargo test test_03_functions --lib
cargo test test_04_controls --lib

# After Steps 4-7: Full function and type support
cargo test test_05_types --lib

# Final: Run all active tests
cargo test test_0 --lib
```

## Success Criteria

- All 5 test files (01-05) pass without errors
- AtomWriter output matches hand-written test expectations exactly
- Output is consistent and follows the simplified format
- Tests 06-11 are disabled (commented out) until their test files are finalized

## Open Questions

None - user has clarified:
1. Struct constructor detection: Check for existing types in scope (use heuristic as fallback since TypeDecl isn't in scope during method parsing)
2. Missing test files (06-11): Comment out test functions for now
