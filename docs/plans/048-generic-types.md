# Plan: Implement Generic Type Definitions

**Status:** 📋 Planned
**Created:** 2025-01-22
**Priority:** HIGH - Core language feature for user-defined generic types

## Overview

Implement user-defined generic types in AutoLang, enabling syntax like `type List<T> { ... }` and `tag May<T> { ... }`.

## Background

### Current State
- **Limited generic support**: `Type::List(Box<Type>)` and `Type::May(Box<Type>)` are hardcoded
- **Type definition structures**: `TypeDecl` and `Tag` lack type parameter fields
- **Parser limitations**: `parse_type()` doesn't handle `<` as generic parameter start
- **User requirement**: Need to define custom generic types

### Desired Syntax
```auto
// Define generic Tag
tag May<T> {
    nil Nil
    val T
    err int
}

// Define generic Type
type List<T> {
    // members
}

// Use generic instances
let x May<int>
let y List<string>
```

---

## Implementation Strategy: Minimum Viable Version

### Phase 1: AST Extensions (1-2 days)

#### 1.1 Add Type Parameter Structures

**File**: `crates/auto-lang/src/ast/types.rs`

```rust
/// Type parameter (single)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: Name,                      // Parameter name (e.g., "T", "K")
    pub constraint: Option<Box<Type>>,  // Type constraint (future extension)
}

/// Generic type instance (e.g., List<int>, May<string>)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericInstance {
    pub base_name: Name,       // Base type name (e.g., "List", "May")
    pub args: Vec<Type>,        // Type parameter list
}
```

#### 1.2 Extend TypeDecl

**File**: `crates/auto-lang/src/ast/types.rs:227`

```rust
pub struct TypeDecl {
    pub name: Name,
    pub kind: TypeDeclKind,
    pub parent: Option<Box<Type>>,
    pub has: Vec<Type>,
    pub specs: Vec<Spec>,
    pub type_params: Vec<TypeParam>,  // ← NEW FIELD
    pub members: Vec<Member>,
    pub delegations: Vec<Delegation>,
    pub methods: Vec<Fn>,
}
```

#### 1.3 Extend Tag

**File**: `crates/auto-lang/src/ast/tag.rs:7`

```rust
pub struct Tag {
    pub name: Name,
    pub type_params: Vec<TypeParam>,  // ← NEW FIELD
    pub fields: Vec<TagField>,
    pub methods: Vec<super::Fn>,
}
```

#### 1.4 Extend Type Enum

**File**: `crates/auto-lang/src/ast/types.rs`

```rust
pub enum Type {
    // ... existing variants ...
    GenericInstance(GenericInstance),  // ← NEW
    // Keep compatibility: List<T> and May<T> still work
    List(Box<Type>),
    May(Box<Type>),
}
```

---

### Phase 2: Parser Implementation (2-3 days)

#### 2.1 Modify parse_type() to Support Generic Instances

**File**: `crates/auto-lang/src/parser.rs:4252`

```rust
pub fn parse_type(&mut self) -> AutoResult<Type> {
    match self.cur.kind {
        TokenKind::Question => {
            self.next();
            let inner_type = self.parse_type()?;
            Ok(Type::May(Box::new(inner_type)))
        }
        TokenKind::Ident => self.parse_ident_or_generic_type(),  // ← CHANGE
        TokenKind::Star => self.parse_ptr_type(),
        TokenKind::LSquare => self.parse_array_type(),
        _ => { /* error handling */ }
    }
}
```

#### 2.2 Add Generic Instance Parsing

**File**: `crates/auto-lang/src/parser.rs`

```rust
/// Parse identifier type or generic instance
fn parse_ident_or_generic_type(&mut self) -> AutoResult<Type> {
    let ident = self.parse_ident()?;

    match ident {
        Expr::Ident(name) => {
            // Check if generic instance (e.g., List<int>)
            if self.cur.kind == TokenKind::Lt {
                // Context check: ensure < is followed by a type
                if self.next_token_is_type() {
                    return self.parse_generic_instance(name);
                }
            }

            // Regular type name
            Ok(self.lookup_type(&name).borrow().clone())
        }
        _ => { /* error handling */ }
    }
}

/// Parse generic instance (e.g., List<int>, Map<str, int>)
fn parse_generic_instance(&mut self, base_name: Name) -> AutoResult<Type> {
    self.expect(TokenKind::Lt)?;

    let mut args = Vec::new();
    args.push(self.parse_type()?);

    while self.cur.kind == TokenKind::Comma {
        self.next();
        args.push(self.parse_type()?);
    }

    self.expect(TokenKind::Gt)?;

    // Special handling: List<T> → Type::List, May<T> → Type::May
    match base_name.as_str() {
        "List" if args.len() == 1 => {
            Ok(Type::List(Box::new(args.into_iter().next().unwrap())))
        }
        "May" if args.len() == 1 => {
            Ok(Type::May(Box::new(args.into_iter().next().unwrap())))
        }
        _ => {
            Ok(Type::GenericInstance(GenericInstance {
                base_name,
                args,
            }))
        }
    }
}

/// Parse single type parameter (e.g., T, K, V)
fn parse_type_param(&mut self) -> AutoResult<TypeParam> {
    match self.cur.kind {
        TokenKind::Ident => {
            let name = self.parse_name()?;
            Ok(TypeParam {
                name,
                constraint: None,
            })
        }
        _ => Err(SyntaxError::Generic {
            message: format!("Expected type parameter, got {}", self.cur.text),
            span: pos_to_span(self.cur.pos),
        }.into()),
    }
}
```

#### 2.3 Modify parse_tag() to Parse Type Parameters

**File**: `crates/auto-lang/src/parser.rs` (find parse_tag function)

```rust
pub fn parse_tag(&mut self) -> AutoResult<Stmt> {
    self.expect(TokenKind::Tag)?;
    let name = self.parse_name()?;

    // Parse type parameter list (optional)
    let mut type_params = Vec::new();
    if self.cur.kind == TokenKind::Lt {
        self.next();  // consume <

        type_params.push(self.parse_type_param()?);

        while self.cur.kind == TokenKind::Comma {
            self.next();  // consume ,
            type_params.push(self.parse_type_param()?);
        }

        self.expect(TokenKind::Gt)?;  // consume >
    }

    self.expect(TokenKind::LBrace)?;

    // ... parse fields and methods ...

    Ok(Stmt::Tag(Tag {
        name,
        type_params,  // ← NEW
        fields,
        methods,
    }))
}
```

#### 2.4 Modify type_decl_stmt_with_annotation()

**File**: `crates/auto-lang/src/parser.rs:3603`

```rust
fn type_decl_stmt_with_annotation(&mut self, check_pub: bool) -> AutoResult<Stmt> {
    // ... existing annotation parsing ...

    let name = self.parse_name()?;

    // Parse type parameter list (NEW)
    let mut type_params = Vec::new();
    if self.cur.kind == TokenKind::Lt {
        self.next();

        type_params.push(self.parse_type_param()?);

        while self.cur.kind == TokenKind::Comma {
            self.next();
            type_params.push(self.parse_type_param()?);
        }

        self.expect(TokenKind::Gt)?;
    }

    // ... parse inheritance, interfaces, composition, members, methods ...

    Ok(Stmt::TypeDecl(TypeDecl {
        name,
        kind,
        parent,
        has,
        specs,
        type_params,  // ← NEW
        members,
        delegations,
        methods,
    }))
}
```

---

### Phase 3: C Transpiler (1-2 days)

#### 3.1 Update Type Name Generation

**File**: `crates/auto-lang/src/trans/c.rs:1500`

```rust
fn c_type_name(&mut self, ty: &Type) -> String {
    match ty {
        // ... existing types ...

        Type::GenericInstance(inst) => {
            // Map<K, V> → map_k_v, List<int> → list_int
            let args: Vec<String> = inst.args.iter()
                .map(|t| self.c_type_name(t))
                .collect();
            format!("{}_{}",
                inst.base_name.to_lowercase(),
                args.join("_")
            )
        }
        _ => { /* existing logic */ }
    }
}
```

#### 3.2 Generate Generic Macros (Optional)

**File**: `crates/auto-lang/src/trans/c.rs`

Generate macros for stdlib generics:

```rust
fn generate_generic_macro(&mut self, tag: &Tag) -> AutoResult<()> {
    if tag.type_params.is_empty() {
        return Ok(());
    }

    let params: Vec<&str> = tag.type_params.iter()
        .map(|p| p.name.as_str())
        .collect();

    writeln!(self.out, "#define DEFINE_{}({}) \\",
        tag.name.to_uppercase(),
        params.join(", ")
    )?;

    // Generate structs and enums
    // ... detailed implementation in Plan agent output ...

    Ok(())
}
```

---

### Phase 4: Type Substitution Support (0.5-1 day)

#### 4.1 Implement Type::substitute()

**File**: `crates/auto-lang/src/ast/types.rs`

```rust
impl Type {
    /// Substitute type parameters
    ///
    /// # Examples
    /// - `T` replace with `int` → `int`
    /// - `List<T>` replace `T` with `int` → `List<int>`
    pub fn substitute(&self, params: &[Name], args: &[Type]) -> Type {
        match self {
            // Basic types: return directly
            Type::Int | Type::Bool | Type::Void => self.clone(),

            // Type parameters: lookup and replace
            Type::User(decl) => {
                if let Some(idx) = params.iter().position(|p| p == &decl.name) {
                    args[idx].clone()
                } else {
                    self.clone()
                }
            }

            // Compound types: recursive substitution
            Type::List(elem) => {
                Type::List(Box::new(elem.substitute(params, args)))
            }
            Type::Array(arr) => {
                Type::Array(ArrayType {
                    elem: Box::new(arr.elem.substitute(params, args)),
                    len: arr.len,
                })
            }

            // Generic instances: recursive substitution
            Type::GenericInstance(inst) => {
                Type::GenericInstance(GenericInstance {
                    base_name: inst.base_name.clone(),
                    args: inst.args.iter().map(|t| t.substitute(params, args)).collect(),
                })
            }

            _ => self.clone(),
        }
    }
}
```

---

### Phase 5: Testing (1 day)

#### 5.1 Create Test Cases

**Directory**: `crates/auto-lang/test/a2c/060_generic_tag/`

**Test file**: `generic_tag.at`
```auto
tag May<T> {
    nil Nil
    val T
    err int
}

fn main() {
    let x May<int>
    x = May.val(42)
    let is_val = x.is_val()
    x
}
```

#### 5.2 Validation Steps

```bash
# 1. Build check
cargo build -p auto-lang

# 2. Run tests
cargo test -p auto-lang test_060_generic_tag

# 3. Check generated output
cat crates/auto-lang/test/a2c/060_generic_tag/generic_tag.wrong.c
cat crates/auto-lang/test/a2c/060_generic_tag/generic_tag.wrong.h

# 4. If correct, rename to expected
mv *.wrong.* *.expected.*
```

---

## Key Implementation Files

### Files to Modify (by priority)

1. **`crates/auto-lang/src/ast/types.rs`**
   - Add `TypeParam` struct
   - Add `GenericInstance` struct
   - Extend `TypeDecl` with `type_params` field
   - Extend `Type` enum with `GenericInstance` variant
   - Implement `Type::substitute()` method

2. **`crates/auto-lang/src/ast/tag.rs`**
   - Extend `Tag` with `type_params` field
   - Update constructor methods

3. **`crates/auto-lang/src/parser.rs`**
   - Modify `parse_type()` to support generic instances (~line 4252)
   - Add `parse_ident_or_generic_type()`
   - Add `parse_generic_instance()`
   - Add `parse_type_param()`
   - Modify `parse_tag()` to parse type parameters
   - Modify `type_decl_stmt_with_annotation()` to parse type parameters

4. **`crates/auto-lang/src/trans/c.rs`**
   - Modify `c_type_name()` to handle `GenericInstance`
   - Optional: add `generate_generic_macro()`

5. **`crates/auto-lang/src/trans/rust.rs`**
   - Modify `rust_type_name()` to handle `GenericInstance`

6. **`crates/auto-lang/src/trans/python.rs`**
   - Modify `python_type_name()` to handle `GenericInstance`

---

## Implementation Steps Summary

### Step 1: AST Extensions
- [ ] Add `TypeParam` struct
- [ ] Add `GenericInstance` struct
- [ ] Extend `TypeDecl` with `type_params`
- [ ] Extend `Tag` with `type_params`
- [ ] Extend `Type` enum with `GenericInstance`

### Step 2: Parser Implementation
- [ ] Add `parse_type_param()`
- [ ] Add `parse_generic_instance()`
- [ ] Modify `parse_type()` to call `parse_ident_or_generic_type()`
- [ ] Add `parse_ident_or_generic_type()`
- [ ] Modify `parse_tag()` to parse type parameters
- [ ] Modify `type_decl_stmt_with_annotation()` to parse type parameters

### Step 3: Transpiler Updates
- [ ] C transpiler: Update `c_type_name()` for generic instances
- [ ] Rust transpiler: Update `rust_type_name()` for generic instances
- [ ] Python transpiler: Update `python_type_name()` for generic instances

### Step 4: Type System
- [ ] Implement `Type::substitute()` method

### Step 5: Testing
- [ ] Create generic Tag test cases
- [ ] Create generic instance usage tests
- [ ] Create nested generic tests (`List<List<int>>`)
- [ ] Create multi-parameter generic tests (`Map<K, V>`)

---

## Success Criteria

### Minimum Viable Version
- [ ] Define single-parameter generic Tag (`tag May<T>`)
- [ ] Define single-parameter generic Type (`type List<T>`)
- [ ] Instantiate generic types (`let x May<int>`)
- [ ] Generate compilable C code
- [ ] Pass basic test cases

### Complete Version
- [ ] Multi-parameter generics (`Map<K, V>`)
- [ ] Nested generics (`List<List<int>>`)
- [ ] Rust/Python transpiler support
- [ ] Type substitution and instantiation

---

## Time Estimate

- **AST Extensions**: 1-2 days
- **Parser Implementation**: 2-3 days
- **Transpiler Updates**: 1-2 days
- **Type System**: 0.5-1 day
- **Testing**: 1 day
- **Total**: 5.5-9 days

---

## Potential Challenges and Solutions

### 1. Lexical Ambiguity: `<` Symbol

**Problem**: `<` could be generic parameter start OR less-than comparison

**Solution**:
- Context-aware parsing based on left-hand side
- `TypeName < ...` → Generic parameters
- `expr < expr` → Comparison operator

### 2. C No Native Generics

**Solutions**:
- **Option A**: Macro generation (suitable for stdlib)
- **Option B**: `void*` erasure (suitable for user generics)
- **Option C**: Hybrid strategy (recommended)

### 3. Type Inference

**Phased implementation**:
- Phase 1: Require explicit type annotations
- Phase 2: Expression-based type inference
- Phase 3: Full Hindley-Milner type inference

---

## Validation Tests

### Test Case Examples

```auto
// test_generic_tag.at
tag May<T> {
    nil Nil
    val T
    err int
}

fn main() {
    // Single-parameter generic
    let x May<int>
    x = May.val(42)

    // Multi-parameter generic
    tag Pair<K, V> {
        first K
        second V
    }

    let p Pair<int, str>

    // Nested generic
    tag List<T> {
        // ...
    }

    let list_list List<List<int>>
}
```

### Running Tests

```bash
# Build
cargo build --release

# Run all tests
cargo test -p auto-lang -- trans

# Run specific test
cargo test -p auto-lang test_060_generic_tag
```

---

## References

- Existing `Type::List(Box<Type>)` and `Type::May(Box<Type>)` implementation
- Generic syntax examples in `stdlib/auto/may.at`
- Rust generic system design reference
- C preprocessor macro patterns
