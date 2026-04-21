# Type Composition Improvements for AutoLang

## Current State Analysis

### How `has` Works Currently

**Syntax**:
```auto
type Wing {
    fn fly() {
        print("flying")
    }
}

type Duck has Wing {
    // Duck automatically gets fly() method
}
```

**Implementation**:
1. **Parser Level** (`parser.rs:3823-3836`):
   - Parses `type Name has Type` syntax
   - Stores composed types in `TypeDecl.has: Vec<Type>`
   - Methods from composed types are mixed into the deriving type

2. **AST Level** (`ast/types.rs:162-169`):
   ```rust
   pub struct TypeDecl {
       pub name: Name,
       pub kind: TypeDeclKind,
       pub has: Vec<Type>,        // ← Composition list
       pub specs: Vec<Spec>,
       pub members: Vec<Member>,
       pub methods: Vec<Fn>,       // ← Mixed-in methods
   }
   ```

3. **Evaluator Level** (`eval.rs:1889-1891`):
   - **CURRENTLY DOES NOTHING**: `fn type_decl(&mut self, _type_decl: &TypeDecl) -> Value { Value::Void }`
   - Methods are looked up via `TypeName::methodName` convention
   - No actual composition logic implemented

4. **Transpiler Level** (`trans/rust.rs:1316-1358`):
   - **CURRENTLY IGNORES `has`**: Doesn't transpile composition
   - Only outputs struct fields and direct methods
   - No trait generation or composition code

### Critical Gaps

1. ❌ **Runtime composition not working** - Methods aren't actually inherited
2. ❌ **Transpiler ignores `has`** - Generates broken Rust code
3. ❌ **No method resolution order** - What if multiple `has` types conflict?
4. ❌ **No field composition** - Only methods, not member variables
5. ❌ **No visibility control** - All methods are public

## Comparison with Other Languages

### Rust Traits (The Gold Standard)
```rust
trait Fly {
    fn fly(&self);
}

impl Fly for Duck {
    fn fly(&self) { println!("flying"); }
}
```
✅ **Pros**: Explicit, conflict-free, type-safe
❌ **Cons**: Verbose, must implement explicitly

### Go Interfaces
```go
type Fly interface {
    Fly()
}
// Duck automatically implements Fly if it has Fly()
```
✅ **Pros**: Implicit, structural typing
❌ **Cons**: No implementation sharing

### Scala Traits
```scala
trait Wing {
    def fly() = println("flying")
}
class Duck extends Wing
```
✅ **Pros**: Mixin implementation, linearization
❌ **Complex**: Diamond problem, linearization rules

### JavaScript/TypeScript Mixins
```typescript
class Wing {
    fly() { console.log("flying"); }
}
class Duck extends Wing {}
```
✅ **Pros**: Simple, single inheritance
❌ **Cons**: No multiple inheritance, prototype chain complexity

## Proposed Improvements

### Phase 1: Fix Critical Bugs (MVP)

**Goal**: Make basic composition work

1. **Implement Runtime Composition** (`eval.rs`)
   ```rust
   fn type_decl(&mut self, type_decl: &TypeDecl) -> Value {
       // 1. Register the type
       let type_meta = Meta::Type(Type::User(shared(type_decl.clone())));
       self.universe.borrow_mut().define(type_decl.name.clone(), Rc::new(type_meta));

       // 2. Mix in methods from `has` types
       for has_type in &type_decl.has {
           if let Type::User(has_decl) = has_type {
               for method in &has_decl.methods {
                   // Register method as TypeName::method_name
                   let method_name = format!("{}::{}", type_decl.name, method.name);
                   self.universe.borrow_mut().define(
                       method_name.into(),
                       Rc::new(Meta::Fn(method.clone()))
                   );
               }
           }
       }

       Value::Void
   }
   ```

2. **Implement Transpiler Support** (`trans/rust.rs`)
   ```rust
   fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
       // Generate traits for each composed type
       for has_type in &type_decl.has {
           if let Type::User(has_decl) = has_type {
               write!(sink.body, "trait {} {{\n", has_decl.name)?;
               for method in &has_decl.methods {
                   // Generate trait method signatures
                   self.method_signature(method, sink)?;
               }
               write!(sink.body, "}}\n\n")?;
           }
       }

       // Generate struct
       write!(sink.body, "struct {} {{", type_decl.name)?;
       // ... existing struct code ...

       // Implement traits
       for has_type in &type_decl.has {
           if let Type::User(has_decl) = has_type {
               write!(sink.body, "\nimpl {} for {} {{\n", has_decl.name, type_decl.name)?;
               for method in &has_decl.methods {
                   self.method_impl(method, sink)?;
               }
               write!(sink.body, "}}\n")?;
           }
       }
   }
   ```

### Phase 2: Advanced Composition Features

**Goal**: Support real-world use cases

1. **Field Composition**
   ```auto
   type Engine {
       hp int
   }

   type Car has Engine {
       // Car automatically gets hp field
   }
   ```
   **Transpile to**: Delegation or flattened fields

2. **Method Override**
   ```auto
   type Wing {
       fn fly() { print("flapping") }
   }

   type Jet has Wing {
       fn fly() {
           // Call Wing's fly()
           super.fly()
           print("with afterburner")
       }
   }
   ```

3. **Multiple Composition**
   ```auto
   type Bird has Wing, Beak, Feet {
       // Compose multiple traits
   }
   ```

4. **Conflict Resolution**
   ```auto
   type A { fn foo() {} }
   type B { fn foo() {} }
   type C has A, B {
       // Error: ambiguous method 'foo'
       // Solution: Explicit override
       fn foo() {
           A.foo()  // Call specific implementation
       }
   }
   ```

5. **Visibility Modifiers**
   ```auto
   type Wing {
       private fn repair() {}
       public fn fly() {}
   }
   ```

### Phase 3: Trait Implementation (Rust-style)

**Goal**: Make AutoLang composition as powerful as Rust traits

1. **Trait Definitions**
   ```auto
   trait Fly {
       fn fly()
       fn glide() {
           // Default implementation
           print("gliding")
       }
   }
   ```

2. **Trait Implementation**
   ```auto
   impl Fly for Bird {
       fn fly() {
           print("flying")
       }
       // glide() uses default
   }
   ```

3. **Trait Bounds**
   ```auto
   fn ride<T has Fly>(vehicle T) {
       vehicle.fly()
   }
   ```

4. **Associated Types**
   ```auto
   trait Iterator {
       type Item
       fn next() Item
   }
   ```

## Design Decisions

### Q1: Inheritance vs Composition?
**Answer**: Composition only
- ❌ No `extends` keyword
- ✅ `has` for composition
- ✅ Traits for polymorphism

### Q2: Mixin Methods or Delegation?
**Answer**: Mixin by default, delegation optional
```auto
type Duck has Wing  // Mixin (copies methods)
type Plane has Wing(delegate)  // Delegation (forwards to Wing instance)
```

### Q3: Linearization or C3 MRO?
**Answer**: **Left-to-right, depth-first (like Python)**
```auto
type D has B, C { }
type C has A { }
type B has A { }

// Method resolution: D → B → A → C
// (B checked before C, A visited once via B)
```

### Q4: Diamond Problem?
**Answer**: **First occurrence wins**
```auto
type A { fn foo() {} }
type B has A {}
type C has A {}
type D has B, C {
    fn foo() {
        // D.foo() → B.foo() → A.foo (C.A ignored)
    }
}
```

## Implementation Plan

### Step 1: Fix Runtime (eval.rs)
- [ ] Implement method mixing in `type_decl()`
- [ ] Add method lookup respecting `has` chain
- [ ] Test: Duck has Wing → duck.fly() works

### Step 2: Fix Transpiler (rust.rs)
- [ ] Generate trait for each composed type
- [ ] Implement traits for deriving types
- [ ] Test: Transpiled Rust code compiles

### Step 3: Add Field Composition
- [ ] Mix in fields from `has` types
- [ ] Decide: flatten vs delegate
- [ ] Test: Car has Engine → car.hp works

### Step 4: Method Override
- [ ] Add `super` keyword support
- [ ] Implement override lookup
- [ ] Test: Jet.fly() calls Wing.fly()

### Step 5: Conflict Resolution
- [ ] Detect method name conflicts
- [ ] Require explicit override
- [ ] Test: Error on ambiguous methods

### Step 6: Multiple Composition
- [ ] Support `has T1, T2, T3`
- [ ] Define MRO (method resolution order)
- [ ] Test: Complex composition chains

### Step 7: Trait System
- [ ] Add `trait` keyword to parser
- [ ] Implement `impl Trait for Type`
- [ ] Add trait bounds
- [ ] Test: Generic functions with traits

## Test Cases

### Basic Composition
```auto
type Wing {
    fn fly() { print("flying") }
}

type Duck has Wing {
}

fn main() {
    let d = Duck()
    d.fly()  // Should print "flying"
}
```

### Field Composition
```auto
type Engine {
    hp int
}

type Car has Engine {
}

fn main() {
    let c = Car()
    print(c.hp)  // Should access Engine's hp field
}
```

### Method Override
```auto
type Wing {
    fn fly() { print("flapping") }
}

type Jet has Wing {
    fn fly() {
        super.fly()
        print("with afterburner")
    }
}
```

### Multiple Composition
```auto
type Swimmer {
    fn swim() { print("swimming") }
}

type Flyer {
    fn fly() { print("flying") }
}

type Duck has Swimmer, Flyer {
}

fn main() {
    let d = Duck()
    d.swim()  // From Swimmer
    d.fly()   // From Flyer
}
```

## Success Criteria

1. ✅ Runtime composition works (methods actually callable)
2. ✅ Transpiler generates valid Rust code
3. ✅ Field composition works (not just methods)
4. ✅ Method override with `super` works
5. ✅ Multiple composition with MRO works
6. ✅ Conflicts are detected and reported
7. ✅ All existing tests still pass
8. ✅ Performance impact is minimal

## Risks and Mitigations

### Risk 1: Breaking Changes
- **Mitigation**: Add feature flag, gradual rollout
- **Backward compat**: Keep old syntax, add new features

### Risk 2: Complexity Explosion
- **Mitigation**: Start simple, iterate
- **Documentation**: Clear examples for each feature

### Risk 3: Performance
- **Mitigation**: Cache method lookups
- **Benchmark**: Measure before/after

### Risk 4: Transpiler Compatibility
- **Mitigation**: Generate idiomatic Rust (traits)
- **Fallback**: Comment out complex cases

## Future Work

1. **Generic Traits**
   ```auto
   trait Iterator<T> {
       fn next() T
   }
   ```

2. **Trait Objects**
   ```auto
   fn fly(thing has Fly) {
       thing.fly()
   }
   ```

3. **Associated Constants**
   ```auto
   trait Math {
       const PI: float = 3.14159
   }
   ```

4. **Macro Composition**
   ```auto
   macro derive(Trait) {
       // Auto-implement trait
   }
   ```

## References

- [Rust Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Scala Mixins](https://docs.scala-lang.org/tour/mixin-composition.html)
- [Python MRO](https://www.python.org/download/releases/2.3/mro/)
- [C3 Linearization](https://www.python.org/download/releases/2.3/mro/)
- [Go Interfaces](https://go.dev/tour/methods/9)
