# Trait System Completion Implementation Plan

## Implementation Status: ⏳ PLANNED

**Priority:** HIGH - Enables polymorphism and code reuse in compiler
**Dependencies:** Plan 019 (Spec Trait System foundation), Plan 028 (Generic Types)
**Estimated Start:** After Plan 028 completion
**Timeline:** 12-16 weeks

## Executive Summary

Complete AutoLang's trait (`spec`) system to support full polymorphism, generic traits, associated types, and dynamic dispatch. This is critical for the self-hosting compiler which relies on trait-based architecture for extensibility (parsers, transpilers, code generators).

**Current State (Plan 019):**
- ✅ Basic `spec` declarations exist
- ✅ Method signatures in specs
- ✅ `impl` blocks for spec implementation
- ❌ No generic type parameters
- ❌ No associated types
- ❌ No trait bounds
- ❌ No dynamic dispatch (vtables)
- ❌ No trait inheritance

**Target State:**
- ✅ Generic specs: `spec Iterable<T>`
- ✅ Associated types: `type Item`
- ✅ Trait bounds: `fn sort<T: Comparable>(arr [T])`
- ✅ Dynamic dispatch: `let reader: spec Reader = file`
- ✅ Trait inheritance: `spec Writer : Reader`
- ✅ Trait objects: `Box<dyn Reader>`
- ✅ Blanket implementations: `impl<T> Reader for T`

**Timeline:** 12-16 weeks
**Complexity:** Very High (requires type system extensions, code generation for vtables)

---

## 1. Why Trait System is Critical

### 1.1 Compiler Architecture

The self-hosting compiler uses trait-based architecture:

**Extensible Parsers:**
```auto
spec BlockParser {
    fn parse(mut parser Parser) Result<Stmt, Error>
}

// Register custom block parsers
parser.register_block("sql", SqlBlockParser)
parser.register_block("html", HtmlBlockParser)
```

**Transpiler Interface:**
```auto
spec Trans {
    fn transpile(ast Code, sink Sink) Result<(), Error>
    fn file_ext() str
}

// Multiple implementations
impl CTrans for Trans
impl RustTrans for Trans
impl PythonTrans for Trans
```

**Visitor Pattern for AST:**
```auto
spec Visitor {
    fn visit_fn(fn_decl Fn)
    fn visit_expr(expr Expr)
    fn visit_stmt(stmt Stmt)
}

impl TypeChecker for Visitor
impl CodeGenerator for Visitor
impl Optimizer for Visitor
}
```

### 1.2 Current Limitations

**Without Generic Traits:**
```auto
// Can't write generic container:
spec Iterable {
    fn iter(self) Iterator    // Over what type?
    fn next(mut it Iterator) T?    // What is T?
}

// Must duplicate for each type:
spec IntIterable {
    fn iter(self) IntIterator
}

spec StrIterable {
    fn iter(self) StrIterator
}
```

**Without Associated Types:**
```auto
// Can't abstract over iterator element type:
spec Iterator {
    type Item    // Not supported yet!

    fn next(mut self) Item?
}
```

**Without Dynamic Dispatch:**
```auto
// Can't create trait objects:
fn process(reader spec Reader) {    // Not supported
    reader.read_line()
}

// Must use concrete types:
fn process(reader File) { ... }
fn process(reader Socket) { ... }
```

---

## 2. Trait System Design

### 2.1 Generic Traits

**Syntax:**
```auto
// Generic trait
spec Iterable<T> {
    fn iter(self) Iterator<T>
}

// Implementation for specific type
impl<T> Iterable<T> for Vec<T> {
    fn iter(self) Iterator<T> {
        Iterator::new(self.data, 0)
    }
}

// Generic function with trait bound
fn process<I: Iterable<int>>(iterable I) {
    let mut it = iterable.iter()
    while let Some(val) = it.next() {
        print(val)
    }
}
```

### 2.2 Associated Types

**Syntax:**
```auto
// Iterator with associated type
spec Iterator {
    type Item

    fn next(mut self) Item?
}

// Implementation
impl<T> Iterator for VecIterator<T> {
    type Item = T

    fn next(mut self) T? {
        if self.pos < self.data.len() {
            let val = self.data[self.pos]
            self.pos = self.pos + 1
            return Some(val)
        }
        return None
    }
}
```

**Usage:**
```auto
fn sum<I>(iterable I) int
    where I: Iterator, I::Item: int
{
    let mut sum = 0
    let mut it = iterable.iter()

    while let Some(val) = it.next() {
        sum = sum + val
    }

    return sum
}
```

### 2.3 Trait Bounds

**Syntax:**
```auto
// Simple trait bound
fn clone<T: Clone>(value T) T {
    value.clone()
}

// Multiple bounds
fn debug_and_clone<T: Clone + Debug>(value T) T {
    value.debug()
    value.clone()
}

// Where clause
fn sort<T>(arr [T]) where T: Comparable {
    // ...
}

// Bound on associated type
fn first<I>(iter I) I::Item
    where I: Iterator
{
    match iter.next() {
        Some(val) => val,
        None => panic!("Empty iterator"),
    }
}
```

### 2.4 Trait Inheritance

**Syntax:**
```auto
// Base trait
spec Reader {
    fn read_line(mut self) str
    fn is_eof(mut self) bool
}

// Derived trait
spec Writer : Reader {
    fn write(mut self, data str)
    fn flush(mut self)
}

// Implementation must provide both
impl File for Writer {
    // Reader methods
    fn read_line(mut self) str { ... }
    fn is_eof(mut self) bool { ... }

    // Writer methods
    fn write(mut self, data str) { ... }
    fn flush(mut self) { ... }
}
```

### 2.5 Dynamic Dispatch

**Syntax:**
```auto
// Trait object type
type ReaderObj = spec Reader

// Create trait object
fn open_file(path str) ReaderObj {
    let file = File::open(path)
    return file as ReaderObj
}

// Use trait object
fn process(reader ReaderObj) {
    while !reader.is_eof() {
        let line = reader.read_line()
        print(line)
    }
}

// Generic function (static dispatch)
fn process_generic<T: Reader>(reader T) {
    while !reader.is_eof() {
        let line = reader.read_line()
        print(line)
    }
}
```

**Implementation:**
- Trait objects use vtables (virtual function tables)
- Vtable created for each trait implementation
- Trait object = (data pointer, vtable pointer)
- Dynamic dispatch = function call through vtable

---

## 3. Implementation Phases

### Phase 1: Type System Extensions (3-4 weeks)

**Objective:** Extend type system to support generic traits

**Deliverables:**
1. Generic type parameters in `Type` enum
2. Trait bound representation
3. Associated types in type system

**Files to Modify:**
```
crates/auto-lang/src/ast/
├── types.rs            # Extend Type enum
└── traits.rs           # Trait representation (new file)
```

**Key Implementation:**

```rust
// ast/types.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Existing types...
    Int, Uint, Float, Double, Bool, Str, CStr, Char,

    // Generic types
    Generic(Box<Name>),

    // Generic application
    App {
        func: Box<Type>,
        args: Vec<Type>,
    },

    // Associated type
    Associated {
        trait_name: Name,
        type_name: Name,
        impl_type: Box<Type>,
    },

    // Trait object
    TraitObject(Box<TraitRef>),

    // ...
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub trait_name: Name,
    pub type_args: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    pub trait_name: Name,
    pub type_args: Vec<Type>,
}

// ast/traits.rs (new file)
#[derive(Debug, Clone, PartialEq)]
pub struct SpecDecl {
    pub name: Name,
    pub type_params: Vec<Name>,  // Generic parameters
    pub bounds: Vec<TraitBound>,  // Constraints on type params
    pub associated_types: Vec<AssociatedType>,
    pub methods: Vec<Fn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedType {
    pub name: Name,
    pub bounds: Vec<TraitBound>,  // Trait bounds on associated type
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    pub spec_name: Name,
    pub spec_args: Vec<Type>,  // Type arguments for spec
    pub for_type: Type,
    pub methods: Vec<Fn>,
}
```

**Success Criteria:**
- Generic types representable in AST
- Trait bounds parse and type-check
- Associated types supported
- Zero compilation warnings

---

### Phase 2: Trait Resolution (4-5 weeks)

**Objective:** Implement trait resolution and coherence checking

**Deliverables:**
1. Trait solver
2. Impl lookup
3. Coherence checking (no overlapping impls)
4. Projection normalization

**Files to Create:**
```
crates/auto-lang/src/infer/
└── traits.rs           # Trait resolution
```

**Key Implementation:**

```rust
// infer/traits.rs
use crate::ast::traits::{SpecDecl, ImplDecl};
use crate::ast::types::Type;
use crate::infer::InferenceContext;

pub struct TraitResolver {
    // Trait registry
    traits: HashMap<Name, SpecDecl>,

    // Impl registry: (spec_name, for_type) -> impl
    impls: HashMap<(Name, Type), ImplDecl>,
}

impl TraitResolver {
    pub fn new() -> Self {
        TraitResolver {
            traits: HashMap::new(),
            impls: HashMap::new(),
        }
    }

    /// Register trait declaration
    pub fn register_trait(&mut self, spec: SpecDecl) {
        self.traits.insert(spec.name.clone(), spec);
    }

    /// Register impl
    pub fn register_impl(&mut self, impl_decl: ImplDecl) -> AutoResult<()> {
        let key = (impl_decl.spec_name.clone(), impl_decl.for_type.clone());

        // Check for overlapping impl
        if self.impls.contains_key(&key) {
            return Err(Error::ConflictingImpl {
                spec: impl_decl.spec_name,
                type_: impl_decl.for_type,
            });
        }

        self.impls.insert(key, impl_decl);
        Ok(())
    }

    /// Find impl for type
    pub fn find_impl(
        &self,
        spec_name: &Name,
        for_type: &Type,
    ) -> Option<&ImplDecl> {
        self.impls.get(&(spec_name.clone(), for_type.clone()))
    }

    /// Check if type implements trait
    pub fn type_implements(
        &self,
        type_: &Type,
        spec_name: &Name,
    ) -> bool {
        // Direct impl
        if self.find_impl(spec_name, type_).is_some() {
            return true;
        }

        // Blanket impl: impl<T> Spec for T
        for (key, impl_decl) in &self.impls {
            if key.0 == *spec_name {
                if let Type::Generic(param) = &impl_decl.for_type {
                    // Unify for_type with generic param
                    // (simplified - real impl needs proper unification)
                    return true;
                }
            }
        }

        false
    }

    /// Resolve associated type
    pub fn resolve_associated_type(
        &self,
        trait_name: &Name,
        type_name: &Name,
        for_type: &Type,
    ) -> AutoResult<Type> {
        let impl_decl = self.find_impl(trait_name, for_type)
            .ok_or_else(|| Error::MissingImpl {
                spec: trait_name.clone(),
                type_: for_type.clone(),
            })?;

        // Find associated type in impl
        for method in &impl_decl.methods {
            if method.name == *type_name {
                // Return associated type from impl
                // (simplified - needs proper type substitution)
                return Ok(method.return_type.clone());
            }
        }

        Err(Error::AssociatedTypeNotFound {
            trait_name: trait_name.clone(),
            type_name: type_name.clone(),
        })
    }

    /// Trait bounds checking
    pub fn check_trait_bound(
        &self,
        ctx: &mut InferenceContext,
        bound: &TraitBound,
        type_: &Type,
    ) -> AutoResult<()> {
        // Substitute type parameters in bound
        let spec = self.traits.get(&bound.trait_name)
            .ok_or_else(|| Error::UndefinedTrait {
                name: bound.trait_name.clone(),
            })?;

        // Check that type implements trait
        if !self.type_implements(type_, &bound.trait_name) {
            return Err(Error::TraitNotImplemented {
                type_: type_.clone(),
                spec: bound.trait_name.clone(),
            });
        }

        Ok(())
    }
}
```

**Success Criteria:**
- Trait resolution works for simple cases
- Blanket impls supported
- Coherence checking detects conflicts
- Associated type projection works
- 100+ test cases passing

---

### Phase 3: Static Dispatch (3-4 weeks)

**Objective:** Generate code for static trait dispatch (monomorphization)

**Deliverables:**
1. Trait method call resolution
2. Monomorphization of generic functions
3. Inline caching for performance

**Files to Modify:**
```
crates/auto-lang/src/trans/c.rs
```

**Key Implementation:**

```rust
// trans/c.rs
impl CTranspiler {
    fn transpile_trait_call(
        &mut self,
        out: &mut Sink,
        expr: &Expr,
    ) -> AutoResult<()> {
        match expr {
            Expr::MethodCall { obj, method, args } => {
                // Resolve trait method
                let obj_type = self.infer_type(obj)?;

                // Find impl
                let impl_decl = self.trait_resolver
                    .find_impl(&obj_type.trait_name, &obj_type)?;

                // Generate direct call (static dispatch)
                let mangled_name = self.mangle_trait_method(
                    &impl_decl.spec_name,
                    &impl_decl.for_type,
                    method,
                )?;

                write!(out, "{}(&{}", mangled_name, self.expr(obj))?;

                for arg in args {
                    write!(out, ", {}", self.expr(arg))?;
                }

                write!(out, ")")?;
            }

            _ => Err(...),
        }

        Ok(())
    }

    fn mangle_trait_method(
        &self,
        spec_name: &Name,
        for_type: &Type,
        method_name: &Name,
    ) -> AutoResult<String> {
        // Mangle trait method name
        // e.g., "Iterator_Vec_int_next" for Iterator<Vec<int>>::next
        let type_str = self.type_to_c(for_type)?;
        Ok(format!("{}_{}_{}",
            spec_name,
            type_str.replace(" ", "_"),
            method_name
        ))
    }

    fn monomorphize_generic_fn(
        &mut self,
        fn_decl: &Fn,
        type_args: &[Type],
    ) -> AutoResult<Fn> {
        // Substitute type parameters in function
        let mut specialized = fn_decl.clone();

        for (param, arg) in fn_decl.type_params.iter().zip(type_args) {
            // Replace all occurrences of param with arg in function body
            specialized = self.substitute_type(&specialized, param, arg)?;
        }

        Ok(specialized)
    }
}
```

**Generated C Example:**
```c
// Trait definition
typedef struct Iterator_Vec_int {
    Vec_int* data;
    size_t pos;
} Iterator_Vec_int;

// Trait method implementation
int Iterator_Vec_int_next(Iterator_Vec_int* self) {
    if (self->pos < self->data->len) {
        return self->data->data[self->pos++];
    }
    return -1;  // None
}

// Usage
Iterator_Vec_int iter = Vec_int_iter(vec);
int val = Iterator_Vec_int_next(&iter);
```

**Success Criteria:**
- Static dispatch generates correct C code
- Monomorphization works for simple cases
- Performance acceptable
- 50+ test cases passing

---

### Phase 4: Dynamic Dispatch (2-3 weeks)

**Objective:** Implement trait objects and vtable generation

**Deliverables:**
1. Vtable generation
2. Trait object creation
3. Virtual method calls
4. Vtable layout optimization

**Files to Modify:**
```
crates/auto-lang/src/trans/c.rs
```

**Key Implementation:**

```rust
// trans/c.rs
impl CTranspiler {
    fn generate_vtable(
        &mut self,
        out: &mut Sink,
        spec_name: &Name,
        impl_decl: &ImplDecl,
    ) -> AutoResult<()> {
        // Generate vtable struct
        let vtable_name = self.vtable_name(spec_name, &impl_decl.for_type);

        writeln!(out, "typedef struct {} {{", vtable_name)?;

        // Function pointers for each method
        for method in &impl_decl.methods {
            let fn_ptr_type = self.fn_ptr_type(method)?;
            writeln!(out, "    {} {};", fn_ptr_type, method.name)?;
        }

        writeln!(out, "}} {};", vtable_name)?;

        // Generate vtable instance
        writeln!(out, "const {} {}_vtable = {{", vtable_name, vtable_name)?;

        for method in &impl_decl.methods {
            let mangled = self.mangle_trait_method(
                spec_name,
                &impl_decl.for_type,
                &method.name,
            )?;
            writeln!(out, "    .{} = &{},", method.name, mangled)?;
        }

        writeln!(out, "}};")?;

        Ok(())
    }

    fn transpile_trait_object(
        &mut self,
        out: &mut Sink,
        expr: &Expr,
    ) -> AutoResult<()> {
        match expr {
            Expr::Cast { expr, type_ } => {
                if let Type::TraitObject(trait_ref) = type_ {
                    // Cast concrete type to trait object
                    let obj_type = self.infer_type(expr)?;

                    writeln!(out,
                        "({{ .data = &{}, .vtable = &{}_vtable }})",
                        self.expr(expr),
                        self.vtable_name(&trait_ref.trait_name, &obj_type)
                    )?;
                }
            }

            _ => Err(...),
        }

        Ok(())
    }

    fn transpile_virtual_call(
        &mut self,
        out: &mut Sink,
        obj: &Expr,
        method_name: &Name,
        args: &[Expr],
    ) -> AutoResult<()> {
        // Virtual call through vtable
        writeln!(out, "{}.vtable->{}(&{}", self.expr(obj), method_name, self.expr(obj))?;

        for arg in args {
            writeln!(out, ", {}", self.expr(arg))?;
        }

        writeln!(out, ")")?;

        Ok(())
    }
}
```

**Generated C Example:**
```c
// Vtable definition
typedef struct Reader_Vtable {
    str (*read_line)(void* self);
    bool (*is_eof)(void* self);
} Reader_Vtable;

// Trait object
typedef struct ReaderObj {
    void* data;
    const Reader_Vtable* vtable;
} ReaderObj;

// Vtable instance for File
const Reader_Vtable Reader_File_vtable = {
    .read_line = (str (*)(void*)) &File_read_line,
    .is_eof = (bool (*)(void*)) &File_is_eof,
};

// Create trait object
ReaderObj open_file(str path) {
    File* file = File_open(path);
    return (ReaderObj){ .data = file, .vtable = &Reader_File_vtable };
}

// Virtual call
void process(ReaderObj reader) {
    while (!reader.vtable->is_eof(reader.data)) {
        str line = reader.vtable->read_line(reader.data);
        print(line);
    }
}
```

**Success Criteria:**
- Trait objects work correctly
- Virtual calls dispatch properly
- No memory leaks
- Performance acceptable (<2x static dispatch)

---

## 4. Testing Strategy

### 4.1 Unit Tests

**Trait Declaration Tests:**
```auto
// tests/traits/001_generic_trait.at
spec Iterable<T> {
    fn iter(self) Iterator<T>
}

spec Iterator<T> {
    fn next(mut self) T?
}

impl<T> Iterable<T> for Vec<T> {
    fn iter(self) Iterator<T> { ... }
}
```

**Associated Types Tests:**
```auto
// tests/traits/002_associated_types.at
spec Iterator {
    type Item

    fn next(mut self) Item?
}

impl<T> Iterator for VecIterator<T> {
    type Item = T

    fn next(mut self) T? { ... }
}
```

**Trait Bounds Tests:**
```auto
// tests/traits/003_trait_bounds.at
fn clone<T: Clone>(value T) T {
    value.clone()
}

fn sort<T>(arr [T]) where T: Comparable {
    // ...
}
```

### 4.2 Compiler Integration Tests

**Use traits in compiler:**
```auto
// auto/compiler/transpiler.at

spec Trans {
    fn transpile(ast Code, sink Sink) Result<(), Error>
    fn file_ext() str
}

impl CTrans for Trans {
    fn transpile(ast Code, sink Sink) Result<(), Error> {
        // Generate C code
    }

    fn file_ext() str {
        ".c"
    }
}

fn compile(trans spec Trans, file str) Result<(), Error> {
    let ast = parse(file)?
    let mut sink = Sink::new(file + trans.file_ext())
    trans.transpile(ast, sink)
}
```

---

## 5. Error Messages

**Missing Trait Implementation:**
```
Error: auto_trait_E0001

  × Type does not implement trait
  ╰─▶ Type `File` does not implement `Reader`
   ╭─[test.at:10:5]
10 │     fn process(reader File) {
11 │         reader.read_line()
   ·         ─────┬────
   ·              ╰── Method `read_line` not found
   ╰────

Help: Implement the `Reader` trait for `File`
```

**Trait Bound Not Satisfied:**
```
Error: auto_trait_E0002

  × Trait bound not satisfied
  ╰─▶ Type `int` does not implement `Clone`
   ╭─[test.at:5:5]
 5 │     fn clone<T: Clone>(value T) T
   ·              ───────
   ·                  ╰── int doesn't implement Clone
   ╰────

Help: Add a Clone implementation for int, or constrain T differently
```

**Conflicting Implementations:**
```
Error: auto_trait_E0003

  × Conflicting trait implementations
  ╰─▶ Multiple implementations of `Reader` for `File`
   ╭─[test.at:15:1]
15 │ impl File for Reader { ... }
16 │ impl File for Reader { ... }
   · ────────────────┬──────────────
   ·                   ╰── Second implementation here
   ╰────

Help: Remove one of the conflicting implementations
```

---

## 6. Success Criteria

### Phase 1 (Type System Extensions)
- [ ] Generic types representable
- [ ] Trait bounds parse and type-check
- [ ] Associated types supported
- [ ] Zero compilation warnings

### Phase 2 (Trait Resolution)
- [ ] Trait resolution works
- [ ] Blanket impls supported
- [ ] Coherence checking
- [ ] Associated type projection
- [ ] 100+ test cases passing

### Phase 3 (Static Dispatch)
- [ ] Static dispatch generates correct C
- [ ] Monomorphization works
- [ ] Performance acceptable
- [ ] 50+ test cases passing

### Phase 4 (Dynamic Dispatch)
- [ ] Trait objects work
- [ ] Virtual calls dispatch properly
- [ ] No memory leaks
- [ ] Performance acceptable

### Overall
- [ ] Can define generic traits
- [ ] Can implement traits for types
- [ ] Can use trait bounds in functions
- [ ] Can create trait objects
- [ ] Compiler uses trait-based architecture

---

## 7. Related Documentation

- **[Plan 019]:** Spec Trait System (foundation)
- **[Plan 028]:** Generic Types and Monomorphization (dependency)
- **[Plan 026]:** Self-Hosting Compiler (uses trait system)
- **[Rust Traits](https://doc.rust-lang.org/reference/items/traits.html)** (reference)

---

## 8. Open Questions

1. **Should we support higher-kinded types?** e.g., `spec Functor<F>` where F is type constructor
2. **Should we support trait aliases?** e.g., `type Stream = Iterator + Send`
3. **Should we support specialization?** e.g., override generic impl with specific one
4. **How to handle orphan rules?** Prevent impls in third-party crates
5. **Should we support async traits?** For futures and async/await

---

## 9. Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Type System Extensions | 3-4 weeks | Generic types, bounds |
| 2. Trait Resolution | 4-5 weeks | Trait solver, coherence |
| 3. Static Dispatch | 3-4 weeks | Monomorphization |
| 4. Dynamic Dispatch | 2-3 weeks | Vtables, trait objects |
| **Total** | **12-16 weeks** | **Complete trait system** |

**Critical Path:** Phase 1 → 2 → 3 → 4

**Dependencies:**
- Must wait for Plan 028 (Generic Types)
- Can overlap with Plan 029 (Pattern Matching)
- Blocks Plan 026 (Self-Hosting Compiler)

---

## 10. Conclusion

This plan completes AutoLang's trait system to match Rust's capabilities, enabling the self-hosting compiler to use trait-based architecture for extensibility and polymorphism.

**Key Benefits:**
1. **Extensible compiler**: Plugin-based parsers, transpilers
2. **Type-safe polymorphism**: Compile-time guarantee of trait implementation
3. **Code reuse**: Generic algorithms over traits
4. **Dynamic dispatch**: Runtime polymorphism when needed

Once complete, AutoLang will have a world-class trait system suitable for building large, extensible systems like compilers.
