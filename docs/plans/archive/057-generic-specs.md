# Plan 057: Generic Specs (Traits with Type Parameters)

**Status**: ✅ **COMPLETE** (2025-01-28)
**Priority**: HIGH - Required for Plan 052 Storage-based List
**Dependencies**: Plan 052 Phase 1 (Const Generic Parameters) ✅ Complete

## Completion Summary

All 6 phases completed successfully:

### ✅ Phase 1: AST Extensions
- `SpecDecl` now includes `generic_params: Vec<GenericParam>`
- `TypeDecl` now includes `spec_impls: Vec<SpecImpl>` for tracking type arguments
- New `SpecImpl` structure tracks spec implementations with concrete type arguments

### ✅ Phase 2: Parser Extensions
- Generic spec declarations: `spec Storage<T> { fn get() T }`
- Type implementations with type arguments: `type Heap<T> as Storage<T>`
- Multiple generic parameters supported: `spec Map<K, V>`

### ✅ Phase 3: Type Checking
- Validates type argument counts match generic parameter counts
- Deferred spec conformance checking for types with empty bodies (ext block support)
- Method signature validation with type parameter substitution

### ✅ Phase 4: C Transpiler Support
- **Monomorphization**: Generates specialized vtables for each concrete instantiation
- **Type Substitution**: Replaces generic parameters with concrete types in signatures
- Example: `Storage<T>` with `T=int` generates `Storage_int_vtable` with `int (*get)(void *self)`

### ✅ Phase 5: Testing
- Test 093: Generic spec declaration
- Test 094: Generic spec with ext blocks
- All 52 transpiler tests passing

### ✅ Phase 6: Documentation
- Examples integrated
- Ready for Plan 052 (Storage-based List)

### Key Features Delivered

1. **Generic Spec Declarations**: `spec Storage<T> { fn get() T }`
2. **Type Implementations**: `type Heap<T> as Storage<T> { ptr *T }`
3. **Type Substitution**: Generic T replaced with concrete types (int, etc.)
4. **Monomorphization**: Specialized vtables for each instantiation (e.g., `Storage_int_vtable`)
5. **Ext Block Support**: Methods added via ext blocks satisfy spec requirements
6. **Spec Conformance**: Validates type argument counts and method signatures

---

## Objective

Add support for generic parameters to Spec (trait) declarations, enabling type-safe interfaces like:

```auto
spec Storage<T> {
    fn data() *T
    fn capacity() u32
    fn try_grow(min_cap u32) bool
}

type Heap<T> as Storage<T> {
    ptr *T
    cap u32
}

type Inline<T, N uint> as Storage<T> {
    buffer [N]T
}
```

---

## Current Limitations

### Before Plan 057

```auto
// ❌ NOT SUPPORTED - Generic spec
spec Storage<T> {
    fn data() *T
}

// ❌ NOT SUPPORTED - Generic impl
type Heap<T> as Storage<T> {
    ptr *T
}

// ✅ Works - But no type safety
spec Storage {
    fn data() *void
    fn capacity() u32
}
```

### After Plan 057

```auto
// ✅ Generic spec with type parameter
spec Storage<T> {
    fn data() *T
    fn capacity() u32
    fn try_grow(min_cap u32) bool
}

// ✅ Type implements generic spec
type Heap<T> as Storage<T> {
    ptr *T
    cap u32
}

// ✅ Const generic parameters also supported
type Inline<T, N uint> as Storage<T> {
    buffer [N]T
}
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Spec Declaration (Generic)                                   │
│                                                              │
│  spec Storage<T> {                                          │
│      fn data() *T          ← T is generic type parameter     │
│      fn capacity() u32                                     │
│  }                                                           │
├─────────────────────────────────────────────────────────────┤
│ Type Declaration (Implements Spec)                           │
│                                                              │
│  type Heap<T> as Storage<T> {                               │
│      ptr *T              ← Must match spec signature          │
│      cap u32                                             │
│  }                                                           │
├─────────────────────────────────────────────────────────────┤
│ Usage                                                        │
│                                                              │
│  fn use_storage<S: Storage<int>>(store S) {                 │
│      let ptr = store.data()  ← Type-safe: returns *int      │
│      let cap = store.capacity()                              │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: AST Extensions (1-2 hours)

### 1.1 Update SpecDecl Structure

**File**: `crates/auto-lang/src/ast/ast.rs`

**Current**:
```rust
pub struct SpecDecl {
    pub name: Name,
    pub methods: Vec<Fn>,
    pub doc: Option<AutoStr>,
}
```

**Updated**:
```rust
pub struct SpecDecl {
    pub name: Name,
    pub generic_params: Vec<GenericParam>,  // NEW
    pub methods: Vec<Fn>,
    pub doc: Option<AutoStr>,
}
```

### 1.2 Update TypeDecl for Spec Implementation

**File**: `crates/auto-lang/src/ast/ast.rs`

**Current**:
```rust
pub struct TypeDecl {
    pub name: Name,
    pub kind: TypeDeclKind,
    pub specs: Vec<Name>,  // Names of specs this type implements
    // ...
}
```

**Updated**:
```rust
pub struct TypeDecl {
    pub name: Name,
    pub kind: TypeDeclKind,
    pub specs: Vec<Name>,  // Names of specs
    pub spec_impls: Vec<SpecImpl>,  // NEW: Generic spec implementations
    // ...
}

// NEW: Spec implementation with type arguments
pub struct SpecImpl {
    pub spec_name: Name,
    pub type_args: Vec<Type>,  // e.g., [int] for Storage<int>
}
```

---

## Phase 2: Parser Extensions (2-3 hours)

### 2.1 Parse Generic Spec Declaration

**File**: `crates/auto-lang/src/parser.rs`

**Add function**:
```rust
fn spec_decl_stmt(&mut self) -> AutoResult<Stmt> {
    self.next(); // skip `spec` keyword

    // Parse spec name
    let name = self.parse_name()?;

    // Parse generic parameters (NEW)
    let generic_params = if self.is_kind(TokenKind::Lt) {
        self.next(); // skip <
        let mut params = Vec::new();

        params.push(self.parse_generic_param()?);
        while self.is_kind(TokenKind::Comma) {
            self.next(); // skip ,
            params.push(self.parse_generic_param()?);
        }

        self.expect(TokenKind::Gt)?; // skip >
        params
    } else {
        Vec::new()
    };

    // Enter spec scope
    self.scope.borrow_mut().enter_scope();

    // Populate type parameters for use in method signatures
    for param in &generic_params {
        if let GenericParam::Type(tp) = &param {
            self.current_type_params.push(tp.name.clone());
        } else if let GenericParam::Const(cp) = &param {
            self.current_const_params.insert(cp.name.clone(), cp.typ.clone());
        }
    }

    // Parse spec body
    self.expect(TokenKind::LBrace)?;
    self.skip_empty_lines();

    let mut methods = Vec::new();
    while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
        let method = self.fn_decl()?;
        if let Stmt::Fn(fn_expr) = method {
            methods.push(fn_expr);
        }
        self.expect_eos(false)?;
    }
    self.expect(TokenKind::RBrace)?;

    // Exit spec scope
    self.scope.borrow_mut().exit_scope();

    Ok(Stmt::Spec(SpecDecl {
        name,
        generic_params,
        methods,
        doc: None,
    }))
}
```

### 2.2 Parse Type Implementing Generic Spec

**File**: `crates/auto-lang/src/parser.rs`

**Update type declaration parsing**:
```rust
// In type_decl_stmt_with_annotation()

// Parse "as Spec" clause
let spec_impls = Vec::new();
if self.is_kind(TokenKind::As) {
    self.next(); // skip `as`

    while self.is_kind(TokenKind::Ident) {
        let spec_name = self.parse_name()?;

        // Parse type arguments if present: as Storage<int>
        let type_args = if self.is_kind(TokenKind::Lt) {
            self.next(); // skip <
            let mut args = Vec::new();

            args.push(self.parse_type()?);
            while self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                args.push(self.parse_type()?);
            }

            self.expect(TokenKind::Gt)?; // skip >
            args
        } else {
            Vec::new()
        };

        spec_impls.push(SpecImpl {
            spec_name,
            type_args,
        });

        // If multiple specs: as Spec1, Spec2
        if !self.is_kind(TokenKind::Comma) {
            break;
        }
        self.next(); // skip ,
    }
}

// Later in TypeDecl initialization:
let mut decl = TypeDecl {
    // ...
    specs: spec_impls.iter().map(|s| s.spec_name.clone()).collect(),
    spec_impls,  // NEW
    // ...
};
```

---

## Phase 3: Type Checking (3-4 hours)

### 3.1 Validate Spec Implementations

**File**: `crates/auto-lang/src/trait_checker.rs` (extend existing)

**Add function**:
```rust
impl TraitChecker {
    /// Check if type correctly implements generic spec
    pub fn check_generic_spec_impl(
        type_decl: &TypeDecl,
        spec_decl: &SpecDecl,
        type_args: &[Type],
    ) -> Result<Vec<Error>, Vec<Error>> {
        let mut errors = Vec::new();

        // 1. Check type argument count
        if spec_decl.generic_params.len() != type_args.len() {
            errors.push(Error::Generic {
                message: format!(
                    "Spec {} expects {} type parameters, but {} provided",
                    spec_decl.name,
                    spec_decl.generic_params.len(),
                    type_args.len()
                ),
            });
            return Err(errors);
        }

        // 2. Create substitution map
        let mut substitution = HashMap::new();
        for (param, arg) in spec_decl.generic_params.iter().zip(type_args) {
            match param {
                GenericParam::Type(type_param) => {
                    substitution.insert(type_param.name.clone(), arg.clone());
                }
                GenericParam::Const(const_param) => {
                    // Const parameters: check if arg is compile-time constant
                    if !Self::is_const_expr(arg) {
                        errors.push(Error::Generic {
                            message: format!(
                                "Const parameter {} requires compile-time constant, got {:?}",
                                const_param.name, arg
                            ),
                        });
                    }
                    substitution.insert(const_param.name.clone(), arg.clone());
                }
            }
        }

        // 3. Check each method in spec
        for spec_method in &spec_decl.methods {
            // Find matching method in type
            let impl_method = type_decl.methods.iter()
                .find(|m| m.name == spec_method.name);

            let impl_method = match impl_method {
                Some(m) => m,
                None => {
                    errors.push(Error::Generic {
                        message: format!(
                            "Type {} implements spec {} but missing method {}",
                            type_decl.name, spec_decl.name, spec_method.name
                        ),
                    });
                    continue;
                }
            };

            // 4. Substitute type parameters in spec signature
            let spec_params_subst = spec_method.params.iter()
                .map(|p| {
                    if let Type::User(type_decl) = &p.ty {
                        if let Some(subst) = substitution.get(&type_decl.name) {
                            subst.clone()
                        } else {
                            p.ty.clone()
                        }
                    } else {
                        p.ty.clone()
                    }
                })
                .collect::<Vec<_>>();

            let impl_params = impl_method.params.iter()
                .map(|p| &p.ty)
                .collect::<Vec<_>>();

            // 5. Compare signatures
            if spec_params_subst.len() != impl_params.len() {
                errors.push(Error::Generic {
                    message: format!(
                        "Method {}: spec expects {} parameters, impl has {}",
                        spec_method.name,
                        spec_params_subst.len(),
                        impl_params.len()
                    ),
                });
                continue;
            }

            for (i, (spec_ty, impl_ty)) in spec_params_subst.iter().zip(impl_params).enumerate() {
                if spec_ty != impl_ty {
                    errors.push(Error::Generic {
                        message: format!(
                            "Method {} parameter {}: spec expects {:?}, impl has {:?}",
                            spec_method.name, i, spec_ty, impl_ty
                        ),
                    });
                }
            }

            // 6. Check return type
            let spec_ret_subst = if let Type::User(type_decl) = &spec_method.ret {
                substitution.get(&type_decl.name).cloned().unwrap_or(spec_method.ret.clone())
            } else {
                spec_method.ret.clone()
            };

            if spec_ret_subst != impl_method.ret {
                errors.push(Error::Generic {
                    message: format!(
                        "Method {} return type: spec expects {:?}, impl has {:?}",
                        spec_method.name, spec_ret_subst, impl_method.ret
                    ),
                });
            }
        }

        if errors.is_empty() {
            Ok(errors)
        } else {
            Err(errors)
        }
    }

    fn is_const_expr(ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Uint | Type::Bool => true,
            Type::User(type_decl) => {
                // Check if it's a const generic parameter
                type_decl.generic_params.iter().any(|p| matches!(p, GenericParam::Const(_)))
            }
            _ => false,
        }
    }
}
```

---

## Phase 4: C Transpiler Support (2-3 hours)

### 4.1 Transpile Generic Specs

**File**: `crates/auto-lang/src/trans/c.rs`

**Spec declarations**:
```rust
fn spec_decl(&mut self, spec: &SpecDecl, sink: &mut Sink) -> AutoResult<()> {
    // Specs don't generate C code (they're compile-time only)
    // But we could generate comments for documentation
    let out = &mut sink.header;
    write!(out, "/* Spec: {} with {} generic parameters */\n",
        spec.name,
        spec.generic_params.len()
    )?;

    Ok(())
}
```

### 4.2 Transpile Spec Implementations

**Type declarations with spec impls**:
```rust
fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
    // ... existing code ...

    // Generate comment documenting spec implementations
    for spec_impl in &type_decl.spec_impls {
        let args_str = if spec_impl.type_args.is_empty() {
            String::new()
        } else {
            format!("<{}>",
                spec_impl.type_args.iter()
                    .map(|t| self.c_type_name(t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        write!(sink.header,
            "/* Implements: {}{} */\n",
            spec_impl.spec_name, args_str
        )?;
    }

    // ... rest of type declaration ...
}
```

---

## Phase 5: Test Cases (2-3 hours)

### 5.1 Unit Tests

**File**: `crates/auto-lang/src/tests/generic_spec_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generic_spec() {
        let code = "spec Storage<T> { fn data() *T }";
        let ast = parse_once(code);
        assert!(matches!(ast.stmts[0], Stmt::Spec(_)));
    }

    #[test]
    fn test_parse_spec_with_multiple_params() {
        let code = "spec Map<K, V> { fn get(k K) V }";
        let ast = parse_once(code);
        // Verify spec has 2 generic parameters
    }

    #[test]
    fn test_parse_type_implements_generic_spec() {
        let code = "type Heap<T> as Storage<T> { ptr *T }";
        let ast = parse_once(code);
        // Verify type_decl.spec_impls contains (Storage, [T])
    }

    #[test]
    fn test_parse_spec_with_const_generic() {
        let code = "spec Array<T, N uint> { fn capacity() N }";
        let ast = parse_once(code);
        // Verify spec has 2 params: type T and const N
    }
}
```

### 5.2 Integration Tests

**File**: `crates/auto-lang/test/a2c/093_generic_specs/`

**Test 093**: Generic spec declaration
```auto
// Test generic spec declaration
spec Storage<T> {
    fn data() *T
    fn capacity() u32
}

type Heap<T> as Storage<T> {
    ptr *T
    cap u32
}

fn main() {
    let store = Heap.new()
    let cap = store.capacity()
    return 0
}
```

**Expected C**:
```c
typedef struct {
    void* ptr;
    uint32_t cap;
} Heap_void;

uint32_t Heap_void_capacity(Heap_void* self) {
    return self->cap;
}

int main(void) {
    Heap_void store = Heap_new();
    uint32_t cap = Heap_void_capacity(&store);
    return 0;
}
```

---

## Phase 6: Example Usage (1-2 hours)

### 6.1 Storage Spec (Plan 052 Integration)

```auto
spec Storage<T> {
    fn data() *T
    fn capacity() u32
    fn try_grow(min_cap u32) bool
    fn drop()
}

type Heap<T> as Storage<T> {
    ptr *T
    cap u32
}

ext Heap<T> {
    #[c, vm]
    static fn new() Heap<T>

    #[c]
    fn data() *T {
        return .ptr
    }

    #[c]
    fn capacity() u32 {
        return .cap
    }

    #[c, vm]
    fn try_grow(min_cap u32) bool

    #[c, vm]
    fn drop()
}

type Inline<T, N uint> as Storage<T> {
    buffer [N]T
}

ext Inline<T, N> {
    #[c, vm]
    static fn new() Inline<T, N>

    #[c]
    fn data() *T {
        return .buffer.ptr
    }

    #[c]
    fn capacity() u32 {
        return N
    }

    #[c, vm]
    fn try_grow(min_cap u32) bool {
        return min_cap <= N
    }

    #[vm]
    fn drop() { }
}
```

### 6.2 Generic List Using Storage Spec

```auto
type List<T, S as Storage<T>> {
    len u32
    store S
}

ext List<T, S> {
    #[c, vm]
    static fn new() List<T, S>

    #[c]
    fn push(elem T) {
        if .len >= .store.capacity() {
            if !.store.try_grow(.len + 1) {
                panic("List::push(): out of memory")
            }
        }
        let ptr = .store.data()
        ptr[.len] = elem
        .len += 1
    }

    #[c]
    fn len() u32 {
        return .len
    }

    #[c]
    fn capacity() u32 {
        return .store.capacity()
    }
}
```

---

## Success Criteria

### Phase 1: AST
1. ✅ `SpecDecl` has `generic_params` field
2. ✅ `TypeDecl` has `spec_impls` field with type arguments
3. ✅ Zero compilation warnings

### Phase 2: Parser
4. ✅ `spec Storage<T>` parses correctly
5. ✅ `type Heap<T> as Storage<T>` parses correctly
6. ✅ `type Inline<T, N uint> as Storage<T>` parses correctly
7. ✅ Multiple specs supported: `type Foo as Spec1<T>, Spec2<U>`

### Phase 3: Type Checking
8. ✅ Generic spec implementations are validated
9. ✅ Type parameter count mismatch caught
10. ✅ Method signature mismatch caught
11. ✅ Const parameter expressions validated

### Phase 4: C Transpiler
12. ✅ Generic specs transpile without errors
13. ✅ Spec implementations documented in comments
14. ✅ Generic methods transpile correctly

### Phase 5: Testing
15. ✅ All unit tests pass
16. ✅ Integration test 093 passes
17. ✅ Zero memory leaks (valgrind)

### Phase 6: Documentation
18. ✅ Examples added to docs/
19. ✅ Plan 052 can use generic specs
20. ✅ CLAUDE.md updated with generic spec syntax

---

## Time Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: AST Extensions | 1-2 hours | None |
| Phase 2: Parser Extensions | 2-3 hours | Phase 1 |
| Phase 3: Type Checking | 3-4 hours | Phase 1, Phase 2 |
| Phase 4: C Transpiler | 2-3 hours | Phase 1, Phase 2 |
| Phase 5: Testing | 2-3 hours | All previous phases |
| Phase 6: Documentation | 1-2 hours | All previous phases |
| **Total** | **11-17 hours** | |

---

## Dependencies

### Required
- ✅ Plan 052 Phase 1: Const generic parameters (COMPLETE)
- ✅ Plan 035: Ext statement and Spec (COMPLETE)
- ✅ Plan 034: Type declarations (COMPLETE)

### Optional
- Plan 058: Trait bounds (S: Storage<T>)
- Plan 059: Associated types (impl Storage<T> { type Item = T })

---

## Future Enhancements

### Short Term (Plan 058)
**Trait Bounds**: Enforce type parameter constraints
```auto
fn use_storage<S: Storage<int>>(store S) {
    // S must implement Storage<int>
    let ptr: *int = store.data()
}
```

### Long Term
**Associated Types**: More ergonomic than explicit type parameters
```auto
spec Storage {
    type Item
    fn data() *Self::Item
}

type Heap<T> as Storage {
    type Item = T
    fn data() *T { }
}
```

---

## Notes

**Key Design Decisions**:
1. **Explicit Type Arguments**: Implementation must specify type args: `as Storage<T>`
2. **Reuse Existing Infrastructure**: Generic param parsing already works (Plan 052)
3. **Compile-Time Only**: Specs don't generate runtime code (like C++ templates)
4. **Gradual Typing**: Can use generic spec without implementing (duck typing still works)

**Comparison with Rust**:
- Rust: `trait Storage<T> { fn data(&self) -> *T }`
- AutoLang: `spec Storage<T> { fn data() *T }`
- Similar concepts, different syntax

**Comparison with C++**:
- C++: `template<typename T> class Storage { T* data(); };`
- AutoLang: More explicit (less implicit magic)
