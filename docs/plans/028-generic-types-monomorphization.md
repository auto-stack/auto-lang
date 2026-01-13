# Generic Types and Monomorphization Implementation Plan

## Implementation Status: ⏳ PLANNED

**Priority:** CRITICAL - Enables type-safe collections and algorithms
**Dependencies:** Plan 024 (Ownership-Based Memory System)
**Estimated Start:** After Plan 024 Phase 2 (Owned Strings)
**Timeline:** 12-16 weeks

## Executive Summary

Implement comprehensive generic types and monomorphization for AutoLang, enabling type-safe containers (HashMap<K,V>, Vec<T>), generic algorithms, and zero-cost abstractions. This is critical for the self-hosting compiler which relies heavily on generics for clean, maintainable code.

**Current State:**
- ✅ Concrete types exist: `int`, `str`, `Point`, etc.
- ❌ No generic type parameters
- ❌ No generic functions
- ❌ No monomorphization
- ❌ Must duplicate code for each type (e.g., IntHashMap, StrHashMap)

**Target State:**
- ✅ Generic types: `type Vec<T>`, `type HashMap<K,V>`
- ✅ Generic functions: `fn id<T>(x T) T`
- ✅ Type parameter inference
- ✅ Monomorphization (generate specialized versions)
- ✅ Generic constraints (trait bounds)
- ✅ Associated types (with Plan 030)

**Timeline:** 12-16 weeks
**Complexity:** Very High (requires type system extensions, monomorphization algorithm, code generation)

---

## 1. Why Generics are Critical

### 1.1 Compiler Requirements

The self-hosting compiler needs generics for:

**Symbol Tables:**
```auto
// Without generics (BAD):
type IntSymbolTable {
    keys []int
    vals []Symbol
}

type StrSymbolTable {
    keys []str
    vals []Symbol
}

// Duplicated code for each type!
fn IntSymbolTable_get(table IntSymbolTable, key int) Symbol? { ... }
fn StrSymbolTable_get(table StrSymbolTable, key str) Symbol? { ... }
```

```auto
// With generics (GOOD):
type HashMap<K, V> {
    buckets []Bucket<K, V>
    size uint
    capacity uint
}

// Single implementation works for all types!
fn get<K, V>(mut table HashMap<K, V>, key K) V? {
    let hash = hash(key)
    let bucket = table.buckets[hash % table.capacity]
    return bucket.find(key)
}

// Usage:
let int_map = HashMap<int, Symbol>::new()
let str_map = HashMap<str, Symbol>::new()
```

**AST Traversal:**
```auto
// Generic visitor
spec Visitor {
    fn visit(mut self, node Node)
}

impl<T> Visitor for T where T: Fn(Node) {
    fn visit(mut self, node Node) {
        self(node)
    }
}

// Usage
fn print_nodes(ast Code) {
    let visitor = |node Node| { print(node) }
    traverse(ast, visitor)
}
```

**Code Generation:**
```auto
// Generic code generator
fn gen_expr<T>(expr Expr, mut out T) Result<(), Error>
    where T: Write
{
    match expr {
        Expr::Int(val) => out.write(str(val))
        Expr::Binary { op, left, right } => {
            gen_expr(left, out)?
            out.write(" ")
            out.write(op.to_str())
            out.write(" ")
            gen_expr(right, out)?
        }
    }
}
```

### 1.2 Code Duplication Problem

**Without generics:**
```auto
// Must implement for each type
fn vec_int_new() Vec_int { ... }
fn vec_str_new() Vec_str { ... }
fn vec_point_new() Vec_point { ... }

fn vec_int_push(mut v Vec_int, x int) { ... }
fn vec_str_push(mut v Vec_str, x str) { ... }
fn vec_point_push(mut v Vec_point, x Point) { ... }

// ... 100s of duplicated functions!
```

**With generics:**
```auto
// Single implementation
type Vec<T> {
    data []T
    len uint
    cap uint
}

fn new<T>() Vec<T> { ... }
fn push<T>(mut v Vec<T>, x T) { ... }

// Works for all types!
let ints = Vec<int>::new()
let strs = Vec<str>::new()
let points = Vec<Point>::new()
```

### 1.3 Comparison with Rust

Rust uses generics extensively in the compiler:

```rust
// HashMap in Rust compiler
use std::collections::HashMap;

type SymbolTable = HashMap<Name, Symbol>;

// Generic AST nodes
pub enum Expr {
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // ...
}

// Generic functions
pub fn transpile_expr<T: Write>(
    expr: &Expr,
    out: &mut T,
) -> Result<()> {
    // ...
}
```

**AutoLang must have equivalent genericity** or the compiler code will be unmaintainable.

---

## 2. Generic Types Design

### 2.1 Syntax

**Generic Type Declarations:**
```auto
// Single type parameter
type Vec<T> {
    data []T
    len uint
    cap uint

    fn new() Vec<T> {
        Vec {
            data: [],
            len: 0,
            cap: 0,
        }
    }

    fn push(mut v Vec<T>, x T) {
        // ...
    }
}

// Multiple type parameters
type HashMap<K, V> {
    buckets []Bucket<K, V>
    size uint
}

type Bucket<K, V> {
    entries []Entry<K, V>
}

type Entry<K, V> {
    key K
    value V
}
```

**Generic Function Declarations:**
```auto
// Generic function
fn id<T>(x T) T {
    return x
}

// Multiple type parameters
fn pair<T, U>(x T, y U) (T, U) {
    return (x, y)
}

// Type inference
fn double(x int) int {
    return x * 2
}

// T inferred as int
let val = id(42)  // T = int
```

**Type Parameter Constraints (Trait Bounds):**
```auto
// With trait bounds
fn clone<T>(x T) T where T: Clone {
    return x.clone()
}

fn debug<T>(x T) where T: Debug {
    x.debug()
}

// Multiple bounds
fn process<T>(x T) where T: Clone + Debug {
    x.debug()
    return x.clone()
}

// Bounded generic type
type Set<T> where T: Hash {
    data HashMap<T, ()>
}
```

### 2.2 Monomorphization

**Process:**
1. Parse generic function
2. Collect type parameters
3. When called with concrete types, generate specialized version
4. Type-check specialized version
5. Generate code (C, Rust, etc.)

**Example:**
```auto
// Generic function
fn swap<T>(mut a T, mut b T) {
    let temp = a
    a = b
    b = temp
}

// Calls
let x = 1
let y = 2
swap(x, y)  // Generates: swap_int(int* a, int* b)

let s1 = "hello"
let s2 = "world"
swap(s1, s2)  // Generates: swap_str(str* a, str* b)
```

**Monomorphized Output:**
```c
// Generated for int
void swap_int(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

// Generated for str
void swap_str(str* a, str* b) {
    str temp = *a;
    *a = *b;
    *b = temp;
}
```

---

## 3. Implementation Phases

### Phase 1: Type System Extensions (3-4 weeks)

**Objective:** Extend type system to support generics

**Deliverables:**
1. Generic type parameters in AST
2. Type variable representation
3. Substitution logic

**Files to Modify:**
```
crates/auto-lang/src/ast/
├── types.rs            # Extend Type enum
└── generics.rs         # Generic types (new file)
```

**Key Implementation:**

```rust
// ast/types.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Existing types
    Int, Uint, Float, Double, Bool, Str, CStr, Char,
    Array { elem: Box<Type>, len: Option<usize> },

    // New: Generic type variable
    GenericVar(Name),

    // New: Generic application
    App {
        func: Box<Type>,  // e.g., GenericVar("Vec")
        args: Vec<Type>,  // e.g., [Int]
    },

    // ...
}

// ast/generics.rs (new file)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParams {
    pub params: Vec<Name>,
    pub bounds: Vec<TraitBound>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub name: Name,
    pub generics: Option<GenericParams>,  // NEW
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Stmt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: Name,
    pub generics: Option<GenericParams>,  // NEW
    pub fields: Vec<StructField>,
}
```

**Success Criteria:**
- Generic types parse correctly
- Type variables representable
- Zero compilation warnings

---

### Phase 2: Generic Parsing (2-3 weeks)

**Objective:** Parse generic syntax

**Deliverables:**
1. Generic type parsing
2. Generic function parsing
3. Type parameter parsing
4. Trait bound parsing

**Files to Modify:**
```
crates/auto-lang/src/parser.rs
```

**Key Implementation:**

```rust
// parser.rs
impl Parser {
    // Parse generic parameters: <T, U: Clone>
    fn parse_generic_params(&mut self) -> AutoResult<GenericParams> {
        self.expect(TokenKind::Lt)?;

        let mut params = vec![];
        let mut bounds = vec![];

        while !self.is_kind(TokenKind::Gt) {
            // Type parameter
            let param = self.expect_ident()?;
            params.push(param);

            // Optional trait bounds
            if self.is_kind(TokenKind::Colon) {
                self.next();

                // Parse bound (e.g., Clone)
                let bound = self.parse_trait_bound()?;
                bounds.push(bound);

                // Multiple bounds: Clone + Debug
                while self.is_kind(TokenKind::Plus) {
                    self.next();
                    let bound = self.parse_trait_bound()?;
                    bounds.push(bound);
                }
            }

            if !self.is_kind(TokenKind::Comma) {
                break;
            }
            self.next();
        }

        self.expect(TokenKind::Gt)?;

        Ok(GenericParams { params, bounds })
    }

    // Parse trait bound: Clone, Iterable<T>
    fn parse_trait_bound(&mut self) -> AutoResult<TraitBound> {
        let trait_name = self.expect_ident()?;

        let mut args = vec![];
        if self.is_kind(TokenKind::Lt) {
            self.next();
            while !self.is_kind(TokenKind::Gt) {
                args.push(self.parse_type()?);
                if !self.is_kind(TokenKind::Comma) {
                    break;
                }
                self.next();
            }
            self.expect(TokenKind::Gt)?;
        }

        Ok(TraitBound { trait_name, args })
    }

    // Parse generic type: Vec<int>, HashMap<str, int>
    fn parse_generic_type(&mut self, base: Name) -> AutoResult<Type> {
        self.expect(TokenKind::Lt)?;

        let mut args = vec![];
        while !self.is_kind(TokenKind::Gt) {
            args.push(self.parse_type()?);
            if !self.is_kind(TokenKind::Comma) {
                break;
            }
            self.next();
        }

        self.expect(TokenKind::Gt)?;

        Ok(Type::App {
            func: Box::new(Type::Named(base)),
            args,
        })
    }

    // Parse function with generics
    fn parse_fn(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::Fn)?;

        let name = self.expect_ident()?;

        // Parse generics if present
        let generics = if self.is_kind(TokenKind::Lt) {
            Some(self.parse_generic_params()?)
        } else {
            None
        };

        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        let return_type = if self.is_kind(TokenKind::Arrow) {
            self.next();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Stmt::Fn(Fn {
            name,
            generics,
            params,
            return_type: return_type.unwrap_or(Type::Void),
            body,
        }))
    }
}
```

**Success Criteria:**
- Generic types parse correctly
- Generic functions parse correctly
- Trait bounds parse correctly
- 50+ test cases passing

---

### Phase 3: Type Inference for Generics (4-5 weeks)

**Objective:** Implement type parameter inference and checking

**Deliverables:**
1. Type parameter unification
2. Type inference for generic calls
3. Trait bound checking
4. Substitution and normalization

**Files to Create:**
```
crates/auto-lang/src/infer/
└── generics.rs         # Generic type inference
```

**Key Implementation:**

```rust
// infer/generics.rs
use crate::ast::generics::GenericParams;
use crate::ast::types::Type;
use crate::infer::InferenceContext;

/// Infer type arguments for generic function call
pub fn infer_generic_args(
    ctx: &mut InferenceContext,
    fn_decl: &Fn,
    arg_types: &[Type],
) -> AutoResult<Vec<Type>> {
    let generics = fn_decl.generics
        .as_ref()
        .ok_or(Error::NotGeneric)?;

    let mut mappings = HashMap::new();  // Type var -> Type

    // Unify parameter types with argument types
    for (param, arg) in fn_decl.params.iter().zip(arg_types) {
        let param_ty = ctx.resolve_type(&param.type_)?;
        let arg_ty = ctx.resolve_type(arg)?;

        // Collect type variables and their constraints
        unify_generics(ctx, &param_ty, &arg_ty, &mut mappings)?;
    }

    // Build type argument list
    let mut type_args = vec![];
    for param in &generics.params {
        let arg = mappings.get(param)
            .ok_or_else(|| Error::CannotInfer {
                param: param.clone(),
            })?
            .clone();

        type_args.push(arg);
    }

    // Check trait bounds
    for bound in &generics.bounds {
        check_trait_bound(ctx, bound, &type_args)?;
    }

    Ok(type_args)
}

/// Unify types with generic variables
fn unify_generics(
    ctx: &mut InferenceContext,
    ty1: &Type,
    ty2: &Type,
    mappings: &mut HashMap<Name, Type>,
) -> AutoResult<()> {
    match (ty1, ty2) {
        // Generic variable matches anything
        (Type::GenericVar(name), ty) => {
            if let Some(existing) = mappings.get(name) {
                // Already bound - unify with existing
                ctx.unify(existing, ty)?;
            } else {
                // Bind to this type
                mappings.insert(name.clone(), ty.clone());
            }
        }

        (ty, Type::GenericVar(name)) => {
            if let Some(existing) = mappings.get(name) {
                ctx.unify(ty, existing)?;
            } else {
                mappings.insert(name.clone(), ty.clone());
            }
        }

        // Generic application: Vec<T> == Vec<int>
        (Type::App { func: f1, args: a1 },
         Type::App { func: f2, args: a2 }) => {
            unify_generics(ctx, f1, f2, mappings)?;

            if a1.len() != a2.len() {
                return Err(Error::ArityMismatch {
                    expected: a1.len(),
                    found: a2.len(),
                });
            }

            for (arg1, arg2) in a1.iter().zip(a2) {
                unify_generics(ctx, arg1, arg2, mappings)?;
            }
        }

        // Other types - use regular unification
        _ => {
            ctx.unify(ty1, ty2)?;
        }
    }

    Ok(())
}

/// Check if trait bounds are satisfied
fn check_trait_bound(
    ctx: &mut InferenceContext,
    bound: &TraitBound,
    type_args: &[Type],
) -> AutoResult<()> {
    // Substitute type parameters in bound
    let mut bound_ty = Type::Named(bound.trait_name.clone());

    if !bound.args.is_empty() {
        let mut substituted_args = vec![];
        for arg in &bound.args {
            substituted_args.push(substitute_type(arg, type_args)?);
        }

        bound_ty = Type::App {
            func: Box::new(bound_ty),
            args: substituted_args,
        };
    }

    // Check that type implements trait
    for type_arg in type_args {
        if !ctx.type_implements(type_arg, &bound_ty)? {
            return Err(Error::TraitNotImplemented {
                type_: type_arg.clone(),
                trait_: bound_ty,
            });
        }
    }

    Ok(())
}

/// Substitute type variables with concrete types
pub fn substitute_type(
    ty: &Type,
    type_args: &[Type],
) -> AutoResult<Type> {
    match ty {
        Type::GenericVar(name) => {
            // Find index in param list
            // Return corresponding type_arg
            // (simplified - needs proper param tracking)
            Err(Error::CannotSubstitute {
                var: name.clone(),
            })
        }

        Type::App { func, args } => {
            let new_func = Box::new(substitute_type(func, type_args)?);
            let new_args = args.iter()
                .map(|arg| substitute_type(arg, type_args))
                .collect::<AutoResult<Vec<_>>>()?;

            Ok(Type::App {
                func: new_func,
                args: new_args,
            })
        }

        _ => Ok(ty.clone()),
    }
}
```

**Success Criteria:**
- Type inference works for simple generics
- Type inference works for nested generics
- Trait bounds checked correctly
- 100+ test cases passing

---

### Phase 4: Monomorphization (3-4 weeks)

**Objective:** Generate specialized versions of generic functions

**Deliverables:**
1. Monomorphization algorithm
2. Specialization cache
3. Generic function specialization
4. Code generation for specialized functions

**Files to Create:**
```
crates/auto-lang/src/trans/
└── mono.rs             # Monomorphization
```

**Key Implementation:**

```rust
// trans/mono.rs
use crate::ast::{Fn, Type, Stmt};
use crate::infer::generics::infer_generic_args;
use std::collections::HashMap;

pub struct Monomorphizer {
    // Cache of specialized functions
    specialized: HashMap<(Name, Vec<Type>), Fn>,
}

impl Monomorphizer {
    pub fn new() -> Self {
        Monomorphizer {
            specialized: HashMap::new(),
        }
    }

    /// Monomorphize generic function call
    pub fn monomorphize_call(
        &mut self,
        ctx: &mut InferenceContext,
        fn_decl: &Fn,
        arg_types: &[Type],
    ) -> AutoResult<&Fn> {
        // Check if already specialized
        let key = (fn_decl.name.clone(), arg_types.to_vec());
        if let Some(specialized) = self.specialized.get(&key) {
            return Ok(specialized);
        }

        // Infer type arguments
        let type_args = infer_generic_args(ctx, fn_decl, arg_types)?;

        // Create specialized version
        let specialized = self.specialize_fn(fn_decl, &type_args)?;

        // Cache it
        self.specialized.insert(key, specialized.clone());

        Ok(self.specialized.get(&key).unwrap())
    }

    /// Specialize generic function with concrete types
    fn specialize_fn(
        &mut self,
        fn_decl: &Fn,
        type_args: &[Type],
    ) -> AutoResult<Fn> {
        let generics = fn_decl.generics.as_ref()
            .ok_or(Error::NotGeneric)?;

        if generics.params.len() != type_args.len() {
            return Err(Error::ArityMismatch {
                expected: generics.params.len(),
                found: type_args.len(),
            });
        }

        // Create substitution map
        let mut subst = HashMap::new();
        for (param, arg) in generics.params.iter().zip(type_args) {
            subst.insert(param.clone(), arg.clone());
        }

        // Substitute types in function signature
        let specialized_params = fn_decl.params.iter()
            .map(|p| substitute_param(p, &subst))
            .collect::<AutoResult<Vec<_>>>()?;

        let specialized_return = substitute_type(&fn_decl.return_type, &subst)?;

        // Substitute types in function body
        let specialized_body = substitute_stmt(&fn_decl.body, &subst)?;

        // Create mangled name
        let mangled_name = mangle_fn_name(&fn_decl.name, type_args);

        Ok(Fn {
            name: mangled_name,
            generics: None,  // No longer generic
            params: specialized_params,
            return_type: specialized_return,
            body: specialized_body,
        })
    }
}

/// Mangle function name with type arguments
fn mangle_fn_name(fn_name: &Name, type_args: &[Type]) -> Name {
    let mut mangled = fn_name.to_string();

    mangled.push('_');
    for ty in type_args {
        mangled.push_str(&ty.to_mangled_string());
        mangled.push('_');
    }

    Name::from(&mangled)
}

/// Substitute type variables in statement
fn substitute_stmt(stmt: &Stmt, subst: &HashMap<Name, Type>) -> AutoResult<Stmt> {
    match stmt {
        Stmt::Expr(expr) => {
            let new_expr = substitute_expr(expr, subst)?;
            Ok(Stmt::Expr(new_expr))
        }

        Stmt::Store(store) => {
            let new_init = store.expr.as_ref()
                .map(|e| substitute_expr(e, subst))
                .transpose()?;

            let new_store_type = substitute_type(&store.type_, subst)?;

            // ... (handle all statement types)

            Ok(Stmt::Store(Store {
                name: store.name.clone(),
                type_: new_store_type,
                expr: new_init,
                kind: store.kind,
            }))
        }

        _ => Ok(stmt.clone()),  // Simplified
    }
}

/// Substitute type variables in expression
fn substitute_expr(expr: &Expr, subst: &HashMap<Name, Type>) -> AutoResult<Expr> {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::Nil => Ok(expr.clone()),

        Expr::Ident(name) => Ok(expr.clone()),

        Expr::Binary { op, left, right, type_ } => {
            let new_left = substitute_expr(left, subst)?;
            let new_right = substitute_expr(right, subst)?;
            let new_type = substitute_type(type_, subst)?;

            Ok(Expr::Binary {
                op: op.clone(),
                left: Box::new(new_left),
                right: Box::new(new_right),
                type_: new_type,
            })
        }

        Expr::Call { func, args, type_ } => {
            let new_func = substitute_expr(func, subst)?;
            let new_args = args.iter()
                .map(|a| substitute_expr(a, subst))
                .collect::<AutoResult<Vec<_>>>()?;
            let new_type = substitute_type(type_, subst)?;

            Ok(Expr::Call {
                func: Box::new(new_func),
                args: new_args,
                type_: new_type,
            })
        }

        _ => Ok(expr.clone()),  // Simplified
    }
}
```

**Integration with Transpiler:**
```rust
// trans/c.rs
impl CTranspiler {
    pub fn transpile_call(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Call { func, args, type_ } => {
                // Check if generic call
                if let Expr::Ident(name) = &**func {
                    if let Some(fn_decl) = self.lookup_fn(name) {
                        if fn_decl.generics.is_some() {
                            // Monomorphize
                            let specialized = self.monomorphizer
                                .monomorphize_call(
                                    &mut self.ctx,
                                    &fn_decl,
                                    &extract_types(args),
                                )?;

                            // Generate call to specialized function
                            let mangled = &specialized.name;
                            write!(out, "{}(", mangled)?;

                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    write!(out, ", ")?;
                                }
                                self.transpile_expr(out, arg)?;
                            }

                            write!(out, ")")?;

                            return Ok(());
                        }
                    }
                }

                // Non-generic call - normal transpilation
                // ...
            }

            _ => { /* ... */ }
        }
    }
}
```

**Generated C Example:**
```c
// Generic AutoLang:
// fn swap<T>(mut a T, mut b T) { ... }
//
// let x = 1
// let y = 2
// swap(x, y)

// Monomorphized for int:
void swap_int(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

// Monomorphized for str:
void swap_str(str* a, str* b) {
    str temp = *a;
    *a = *b;
    *b = temp;
}

// Call sites:
swap_int(&x, &y);
swap_str(&s1, &s2);
```

**Success Criteria:**
- Monomorphization works for simple cases
- Monomorphization works for nested generics
- Specialization cache prevents duplicates
- Generated C compiles without errors
- 50+ test cases passing

---

## 4. Testing Strategy

### 4.1 Unit Tests

**Generic Type Tests:**
```auto
// tests/generics/001_vec.at
type Vec<T> {
    data []T
    len uint
    cap uint
}

fn main() {
    let ints = Vec<int>::new()
    ints.push(1)
    ints.push(2)

    let strs = Vec<str>::new()
    strs.push("hello")
    strs.push("world")
}
```

**Generic Function Tests:**
```auto
// tests/generics/002_functions.at
fn id<T>(x T) T {
    return x
}

fn swap<T>(mut a T, mut b T) {
    let temp = a
    a = b
    b = temp
}

fn main() {
    let x = 1
    let y = id(x)  // T inferred as int

    let a = 10
    let b = 20
    swap(a, b)
}
```

**Trait Bounds Tests:**
```auto
// tests/generics/003_bounds.at
spec Clone {
    fn clone(self) Self
}

fn clone<T>(x T) T where T: Clone {
    return x.clone()
}

type Point {
    x int
    y int
}

impl Clone for Point {
    fn clone(self) Point {
        return Point{x: self.x, y: self.y}
    }
}

fn main() {
    let p = Point{x: 1, y: 2}
    let p2 = clone(p)
}
```

---

## 5. Success Criteria

### Phase 1 (Type System Extensions)
- [ ] Generic types in AST
- [ ] Type variables representable
- [ ] Zero compilation warnings

### Phase 2 (Generic Parsing)
- [ ] Generic types parse correctly
- [ ] Generic functions parse correctly
- [ ] Trait bounds parse correctly
- [ ] 50+ test cases passing

### Phase 3 (Type Inference)
- [ ] Type inference for generics
- [ ] Trait bound checking
- [ ] Type substitution works
- [ ] 100+ test cases passing

### Phase 4 (Monomorphization)
- [ ] Monomorphization works
- [ ] Specialization cache functional
- [ ] Generated C compiles
- [ ] 50+ test cases passing

### Overall
- [ ] Can define generic types
- [ ] Can define generic functions
- [ ] Type inference works
- [ ] Monomorphization generates correct code
- [ ] No code duplication needed

---

## 6. Related Documentation

- **[Plan 024]:** Ownership-Based Memory System (dependency)
- **[Plan 030]:** Trait System Completion (trait bounds)
- **[Plan 026]:** Self-Hosting Compiler (uses generics)
- **[Rust Generics](https://doc.rust-lang.org/reference/items/generics.html):** Reference

---

## 7. Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Type System Extensions | 3-4 weeks | Generic types in AST |
| 2. Generic Parsing | 2-3 weeks | Parse generic syntax |
| 3. Type Inference | 4-5 weeks | Infer generic types |
| 4. Monomorphization | 3-4 weeks | Specialize functions |
| **Total** | **12-16 weeks** | **Full generics** |

**Critical Path:** Phase 1 → 2 → 3 → 4

**Dependencies:**
- Must wait for Plan 024 Phase 2 (Owned Strings)
- Can overlap with Plan 029 (Pattern Matching)
- Enables Plan 030 (Trait System)

---

## 8. Conclusion

This plan implements comprehensive generics and monomorphization for AutoLang, enabling type-safe collections, generic algorithms, and zero-cost abstractions. This is critical for the self-hosting compiler to be maintainable and efficient.

**Key Benefits:**
1. **No code duplication**: Single implementation works for all types
2. **Type safety**: Compile-time guarantee of correctness
3. **Zero-cost**: Monomorphization generates efficient code
4. **Maintainability**: Compiler code is clean and idiomatic

Once complete, AutoLang will have Rust-level generics suitable for building production compilers and large systems.
