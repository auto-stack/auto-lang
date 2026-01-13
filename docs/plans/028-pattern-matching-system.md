# Pattern Matching System Implementation Plan

## Implementation Status: ⏳ PLANNED

**Priority:** CRITICAL - Blocks self-hosting compiler (Plan 026)
**Dependencies:** None (can start in parallel with Plan 024)
**Estimated Start:** During or after Plan 024 Phase 1
**Timeline:** 10-14 weeks

## Executive Summary

Implement comprehensive pattern matching system for AutoLang, extending the current `is` statement to support full pattern matching capabilities. This is critical for the self-hosting compiler which needs to deconstruct AST nodes, match against struct patterns, enum variants, and perform exhaustive pattern checking.

**Current State:**
- ✅ Basic `is` statement exists (equality matching)
- ✅ Simple value matching: `is x { 1 => print("one") }`
- ❌ No struct pattern matching
- ❌ No enum variant matching
- ❌ No nested patterns
- ❌ No pattern guards
- ❌ No exhaustiveness checking

**Target State:**
- ✅ Struct pattern matching: `is Point { x, y }`
- ✅ Enum variant matching: `is Option::Some(val)`
- ✅ Nested patterns: `is (a, b, c)`
- ✅ Pattern guards: `is x if x > 0`
- ✅ OR patterns: `is 1 | 2 | 3`
- ✅ Exhaustiveness checking at compile time
- ✅ Pattern binding: `is Point { x: name }`

**Timeline:** 10-14 weeks
**Complexity:** Very High (requires parser, type checker, code generation changes)

---

## 1. Why Pattern Matching is Critical

### 1.1 Compiler Requirements

The self-hosting compiler needs pattern matching for:

**AST Traversal:**
```auto
// Current (not possible):
fn as_fn(stmt Stmt) Fn? {
    // How to extract Fn from Stmt enum?
    // Must use manual type checks and casting
}

// With pattern matching:
fn as_fn(stmt Stmt) Fn? {
    is stmt {
        Stmt::Fn(fn_decl) => return Some(fn_decl)
        else => return None
    }
}
```

**Symbol Resolution:**
```auto
fn resolve_type(expr Expr) Type {
    is expr {
        Expr::Int(_) => Type::Int
        Expr::Ident(name) => lookup_type(name)
        Expr::Binary { op, left, right, type } => type
        _ => Type::Unknown
    }
}
```

**Code Generation:**
```auto
fn gen_expr(expr Expr) str {
    is expr {
        Expr::Int(value) => str(value)
        Expr::Binary { op: Add, left, right } =>
            gen_expr(left) + " + " + gen_expr(right)
        Expr::Call { func, args } =>
            gen_expr(func) + "(" + join(args, ", ") + ")"
    }
}
```

### 1.2 Comparison with Rust

Rust's `match` is used extensively in the compiler:

```rust
// From parser.rs:148-168
pub fn as_fn(&self) -> Option<&Fn> {
    match self {
        Stmt::Fn(fn_decl) => Some(fn_decl),
        _ => None,
    }
}

// From trans/c.rs:674-690
match store.kind {
    StoreKind::Let => out.write(b"const ")?,
    StoreKind::Mut | StoreKind::Var => out.write(b"let ")?,
    _ => {},
}
```

**AutoLang must have equivalent expressiveness** or the compiler code will be verbose and error-prone.

---

## 2. Pattern Matching Design

### 2.1 Syntax Extensions

**Basic Pattern Matching (already supported):**
```auto
is value {
    1 => print("one")
    2 => print("two")
    _ => print("other")
}
```

**Struct Pattern Matching (NEW):**
```auto
type Point { x int, y int }

let p = Point{x: 3, y: 4}

is p {
    Point{x, y} => print(f"Point at $x, $y")
    Point{x: 0, y: 0} => print("Origin")
}

// With binding
is p {
    Point{x: px, y: py} => print(f"x=$px, y=$py")
}
```

**Enum Variant Matching (NEW):**
```auto
enum Option<T> {
    Some(T)
    None
}

let opt = Option::Some(42)

is opt {
    Option::None => print("Nothing")
    Option::Some(value) => print(f"Value: $value")
}

// Nested pattern
is opt {
    Option::Some(Point{x: 0, y: 0}) => print("Origin point")
    Option::Some(Point{x, y}) => print(f"Point: $x, $y")
    Option::None => print("None")
}
```

**OR Patterns (NEW):**
```auto
is value {
    1 | 2 | 3 => print("Small")
    10 | 20 => print("Medium")
    else => print("Large")
}
```

**Pattern Guards (NEW):**
```auto
is value {
    x if x < 0 => print("Negative")
    x if x > 100 => print("Large")
    x => print(f"Value: $x")
}
```

**Tuple Matching (NEW):**
```auto
let pair = (1, "hello")

is pair {
    (0, _) => print("First is zero")
    (_, "hello") => print("Second is hello")
    (x, y) => print(f"$x, $y")
}
```

**Slice/Array Matching (NEW):**
```auto
let arr = [1, 2, 3, 4]

is arr {
    [] => print("Empty")
    [x] => print(f"One element: $x")
    [x, y] => print(f"Two elements: $x, $y")
    [first, ...rest] => print(f"First: $first, rest: $rest")
}
```

### 2.2 Semantics

**Exhaustiveness:**
- All patterns must be covered OR have `else`/`_` wildcard
- Compile-time error if patterns not exhaustive
- Warning for unreachable patterns

**Match Order:**
- Patterns evaluated in order (top to bottom)
- First matching pattern wins
- Later patterns never reached if earlier matches

**Binding Scope:**
- Pattern bindings scoped to matched branch
- Bindings immutable by default
- Shadowing allowed in different branches

**Refutability:**
- Irrefutable patterns: Always match (structs with all fields, wildcards)
- Refutable patterns: May fail (enum variants, guards)
- `let`/`mut` only allow irrefutable patterns

---

## 3. Implementation Phases

### Phase 1: Pattern Representation (2-3 weeks)

**Objective:** Define AST nodes for patterns

**Deliverables:**
1. `Pattern` enum in AST
2. Pattern parsing integration
3. Pattern type checking

**Files to Create:**
```
crates/auto-lang/src/ast/
└── patterns.rs         # Pattern enum and utilities
```

**Key Implementation:**

```rust
// ast/patterns.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    // Literals
    Int(i64),
    Uint(u64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(AutoStr),

    // Wildcard
    Wildcard,

    // Identifier (binding)
    Binding(Name),

    // Struct patterns
    Struct {
        type_name: Name,
        fields: Vec<(Name, Pattern)>,  // (field_name, pattern)
    },

    // Enum variant patterns
    EnumVariant {
        type_name: Name,
        variant_name: Name,
        inner: Option<Box<Pattern>>,
    },

    // Tuple patterns
    Tuple(Vec<Pattern>),

    // Array/slice patterns
    Array(Vec<Pattern>),  // Fixed length
    Slice {
        before: Vec<Pattern>,
        rest: Option<Box<Pattern>>,
        after: Vec<Pattern>,
    },

    // OR patterns
    Or(Vec<Pattern>),

    // Guards
    Guard {
        pattern: Box<Pattern>,
        condition: Expr,
    },
}
```

**Success Criteria:**
- All pattern types constructable
- Pattern pretty-printing works
- Zero compilation warnings

---

### Phase 2: Pattern Parsing (3-4 weeks)

**Objective:** Extend parser to parse patterns

**Deliverables:**
1. Pattern parsing functions
2. Integration with `is` statement parser
3. Error recovery for invalid patterns

**Files to Modify:**
```
crates/auto-lang/src/
└── parser.rs           # Add pattern parsing
```

**Key Implementation:**

```rust
// parser.rs
impl Parser {
    // Parse pattern in is statement
    fn parse_pattern(&mut self) -> AutoResult<Pattern> {
        match self.kind() {
            TokenKind::IntLit => {
                let value = self.cur.text.parse()?;
                self.next();
                Ok(Pattern::Int(value))
            }

            TokenKind::Ident => {
                let name = self.cur.name.clone();

                // Check for struct pattern
                if self.peek_kind() == TokenKind::LBrace {
                    self.next();  // consume ident
                    return self.parse_struct_pattern(name);
                }

                // Check for enum pattern
                if self.peek_kind() == TokenKind::ColonColon {
                    self.next();  // consume ident
                    return self.parse_enum_pattern(name);
                }

                // Simple binding
                self.next();
                Ok(Pattern::Binding(name))
            }

            TokenKind::Underscore => {
                self.next();
                Ok(Pattern::Wildcard)
            }

            TokenKind::LParen => {
                self.next();
                let patterns = self.parse_pattern_list()?;
                self.expect(TokenKind::RParen)?;
                Ok(Pattern::Tuple(patterns))
            }

            _ => Err(...),
        }
    }

    fn parse_struct_pattern(&mut self, type_name: Name)
        -> AutoResult<Pattern>
    {
        self.expect(TokenKind::LBrace)?;

        let mut fields = vec![];
        while !self.is_kind(TokenKind::RBrace) {
            let field_name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;

            let pattern = self.parse_pattern()?;

            fields.push((field_name, pattern));

            if !self.is_kind(TokenKind::Comma) {
                break;
            }
            self.next();  // consume comma
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Pattern::Struct { type_name, fields })
    }

    fn parse_enum_pattern(&mut self, type_name: Name)
        -> AutoResult<Pattern>
    {
        self.expect(TokenKind::ColonColon)?;
        let variant_name = self.expect_ident()?;

        let inner = if self.is_kind(TokenKind::LParen) {
            self.next();
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::RParen)?;
            Some(Box::new(pattern))
        } else {
            None
        };

        Ok(Pattern::EnumVariant {
            type_name,
            variant_name,
            inner,
        })
    }
}
```

**Success Criteria:**
- All pattern types parse correctly
- Error recovery works
- 50+ test cases passing

---

### Phase 3: Pattern Type Checking (3-4 weeks)

**Objective:** Implement pattern matching type rules

**Deliverables:**
1. Pattern type inference
2. Pattern exhaustiveness checking
3. Unreachable pattern detection

**Files to Create:**
```
crates/auto-lang/src/infer/
└── patterns.rs         # Pattern type checking
```

**Key Implementation:**

```rust
// infer/patterns.rs
use crate::ast::Pattern;
use crate::ast::types::Type;
use crate::infer::InferenceContext;

/// Infer type of pattern against expected type
pub fn infer_pattern(
    ctx: &mut InferenceContext,
    pattern: &Pattern,
    expected: &Type,
) -> AutoResult<Type> {
    match pattern {
        Pattern::Int(_) => Ok(Type::Int),
        Pattern::Uint(_) => Ok(Type::Uint),
        Pattern::Bool(_) => Ok(Type::Bool),
        Pattern::Wildcard => Ok(expected.clone()),

        Pattern::Binding(name) => {
            // Bind name to expected type
            ctx.bind_var(name.clone(), expected.clone());
            Ok(expected.clone())
        }

        Pattern::Struct { type_name, fields } => {
            // Lookup struct definition
            let struct_def = ctx.lookup_type(type_name)?;

            // Check field types match
            for (field_name, pattern) in fields {
                let field_type = struct_def.get_field_type(field_name)?;
                infer_pattern(ctx, pattern, &field_type)?;
            }

            Ok(struct_def)
        }

        Pattern::EnumVariant { type_name, variant_name, inner } => {
            // Lookup enum definition
            let enum_def = ctx.lookup_type(type_name)?;

            // Check variant exists
            let variant = enum_def.get_variant(variant_name)?;

            // Type-check inner pattern if present
            if let Some(inner_pat) = inner {
                if let Some(variant_type) = &variant.inner_type {
                    infer_pattern(ctx, inner_pat, variant_type)?;
                } else {
                    return Err(...);  // Variant has no data
                }
            }

            Ok(enum_def)
        }

        Pattern::Or(patterns) => {
            // All branches must have same type
            let mut ty = None;
            for pat in patterns {
                let pat_ty = infer_pattern(ctx, pat, expected)?;
                match &ty {
                    None => ty = Some(pat_ty),
                    Some(existing) => {
                        ctx.unify(existing, &pat_ty)?;
                    }
                }
            }
            Ok(ty.unwrap_or_else(|| Type::Unknown))
        }

        Pattern::Guard { pattern, condition } => {
            // Type-check pattern
            infer_pattern(ctx, pattern, expected)?;

            // Condition must be bool
            let cond_ty = ctx.infer_expr(condition)?;
            ctx.unify(&cond_ty, &Type::Bool)?;

            Ok(expected.clone())
        }

        _ => Err(...),
    }
}

/// Check if patterns are exhaustive
pub fn check_exhaustiveness(
    ctx: &mut InferenceContext,
    patterns: &[Pattern],
    value_type: &Type,
) -> AutoResult<bool> {
    // Algorithm:
    // 1. Build decision tree from patterns
    // 2. Check if all values of value_type covered
    // 3. Return true if exhaustive, false if missing cases

    // For enums: check all variants covered
    if let Type::Enum(enum_name) = value_type {
        let enum_def = ctx.lookup_type(enum_name)?;
        let mut covered = HashSet::new();

        for pat in patterns {
            if let Pattern::EnumVariant { variant_name, .. } = pat {
                covered.insert(variant_name.clone());
            }
        }

        if covered.len() < enum_def.variants.len() {
            let missing: Vec<_> = enum_def.variants.iter()
                .filter(|v| !covered.contains(&v.name))
                .map(|v| v.name.clone())
                .collect();

            return Err(Error::NonExhaustiveMatch {
                type_name: enum_name.clone(),
                missing,
            });
        }
    }

    Ok(true)
}

/// Check for unreachable patterns
pub fn check_unreachable(
    ctx: &mut InferenceContext,
    patterns: &[Pattern],
) -> AutoResult<Vec<usize>> {
    let mut unreachable = vec![];
    let mut covered = HashSet::new();

    for (i, pat) in patterns.iter().enumerate() {
        let matches = compute_match_set(ctx, pat)?;

        if covered.is_superset(&matches) {
            unreachable.push(i);
        } else {
            covered.extend(matches);
        }
    }

    Ok(unreachable)
}
```

**Success Criteria:**
- Pattern type inference works for all patterns
- Exhaustiveness checking detects missing cases
- Unreachable pattern detection works
- 100+ test cases passing

---

### Phase 4: Code Generation (2-3 weeks)

**Objective:** Generate C code for pattern matching

**Deliverables:**
1. Pattern matching code generation
2. Optimized pattern compilation
3. Integration with C transpiler

**Files to Modify:**
```
crates/auto-lang/src/trans/c.rs
```

**Key Implementation:**

```rust
// trans/c.rs
impl CTranspiler {
    fn transpile_pattern_match(
        &mut self,
        out: &mut Sink,
        value: Expr,
        arms: Vec<MatchArm>,
    ) -> AutoResult<()> {
        // Generate if-else chain for pattern matching
        let mut first = true;

        for arm in arms {
            if first {
                self.write(out, b"if (")?;
                first = false;
            } else {
                self.write(out, b" else if (")?;
            }

            // Generate pattern check
            self.transpile_pattern_check(out, &arm.pattern, &value)?;

            self.write(out, b") {\n")?;
            self.indent += 4;

            // Generate arm body
            self.transpile_stmt(out, &arm.body)?;

            self.indent -= 4;
            self.write(out, b"}\n")?;
        }

        // Default case (if present)
        if let Some(default_arm) = arms.iter().find(|arm| arm.is_default) {
            self.write(out, b" else {\n")?;
            self.indent += 4;
            self.transpile_stmt(out, &default_arm.body)?;
            self.indent -= 4;
            self.write(out, b"}\n")?;
        }

        Ok(())
    }

    fn transpile_pattern_check(
        &mut self,
        out: &mut Sink,
        pattern: &Pattern,
        value: &Expr,
    ) -> AutoResult<()> {
        match pattern {
            Pattern::Int(i) => {
                write!(out, "{} == {}", self.expr(value), i)
            }

            Pattern::Wildcard => {
                write!(out, "true")
            }

            Pattern::Binding(name) => {
                // Generate binding
                write!(out, "({} = {})", name, self.expr(value))
            }

            Pattern::Struct { type_name, fields } => {
                // Generate struct field checks
                for (field_name, field_pat) in fields {
                    self.transpile_pattern_check(
                        out,
                        field_pat,
                        &Expr::FieldAccess {
                            base: value.clone(),
                            field: field_name.clone(),
                        }
                    )?;
                }
            }

            Pattern::EnumVariant { type_name, variant_name, inner } => {
                // Generate discriminant check
                write!(out,
                    "{}.discriminant == {}_{}",
                    self.expr(value),
                    type_name,
                    variant_name
                )?;

                if let Some(inner_pat) = inner {
                    self.write(out, b" && ")?;
                    self.transpile_pattern_check(
                        out,
                        inner_pat,
                        &Expr::FieldAccess {
                            base: value.clone(),
                            field: "data".into(),
                        }
                    )?;
                }
            }

            Pattern::Or(patterns) => {
                let mut first = true;
                for pat in patterns {
                    if !first {
                        self.write(out, b" || ")?;
                    }
                    first = false;
                    self.transpile_pattern_check(out, pat, value)?;
                }
            }

            Pattern::Guard { pattern, condition } => {
                self.transpile_pattern_check(out, pattern, value)?;
                self.write(out, b" && ")?;
                self.transpile_expr(out, condition)?;
            }

            _ => Err(...),
        }

        Ok(())
    }
}
```

**Success Criteria:**
- Pattern matching generates valid C code
- Generated C compiles without warnings
- All test cases pass
- Performance acceptable (no exponential blowup)

---

## 4. Testing Strategy

### 4.1 Unit Tests

**Pattern Parsing Tests:**
```
tests/patterns/
├── 001_literals.at
├── 002_structs.at
├── 003_enums.at
├── 004_nested.at
├── 005_guards.at
├── 006_or_patterns.at
└── 007_exhaustiveness.at
```

**Example Test:**
```auto
// 003_enums.at
enum Option<T> {
    Some(T)
    None
}

fn main() {
    let opt = Option::Some(42)

    is opt {
        Option::None => print("None")
        Option::Some(x) => print(f"Some: $x")
    }

    // Test nested pattern
    let nested = Option::Some(Option::Some(10))
    is nested {
        Option::None => print("None")
        Option::Some(Option::None) => print("Some(None)")
        Option::Some(Option::Some(v)) => print(f"Some(Some($v))")
    }
}
```

### 4.2 Compiler Integration Tests

**Use pattern matching in compiler:**
```auto
// auto/compiler/parser.at

fn parse_stmt(mut parser Parser) Stmt? {
    is parser.peek() {
        Token::Fn => parse_fn(parser)
        Token::Let => parse_let(parser)
        Token::Mut => parse_mut(parser)
        Token::Return => parse_return(parser)
        Token::If => parse_if(parser)
        Token::For => parse_for(parser)
        else => {
            parser.error(f"Unexpected token: $parser.peek()")
            return None
        }
    }
}
```

---

## 5. Error Messages

**Non-Exhaustive Match:**
```
Error: auto_pattern_E0001

  × Non-exhaustive pattern match
  ╰─▶ Pattern match does not cover all possible values
   ╭─[test.at:5:5]
 5 │     is opt {
 6 │         Option::Some(x) => print(f"Some: $x")
 7 │     }
   ·     ┬
   ·     ╰── Missing case: Option::None
   ╰────

Help: Add a catch-all pattern `_` or handle the missing variant
```

**Unreachable Pattern:**
```
Warning: auto_pattern_W0001

  ⚠ Unreachable pattern
  ╰─▶ This pattern can never match
   ╭─[test.at:5:5]
 5 │     is value {
 6 │         1 => print("one")
 7 │         1 | 2 => print("one or two")  // Never reached
   ·              ┬
   ·              ╰── Pattern already covered by line 6
   ╰────
```

**Type Mismatch:**
```
Error: auto_pattern_E0002

  × Type mismatch in pattern
  ╰─▶ Expected type `Point`, found `int`
   ╭─[test.at:5:5]
 5 │     is value {
 6 │         Point{x, y} => print("point")  // value is int
   ·         ────────┬────────
   ·                 ╰── Type mismatch
   ╰────
```

---

## 6. Success Criteria

### Phase 1 (Pattern Representation)
- [ ] Pattern enum with all variants
- [ ] Pattern pretty-printing
- [ ] Zero compilation warnings

### Phase 2 (Pattern Parsing)
- [ ] All pattern types parse correctly
- [ ] Error recovery for invalid patterns
- [ ] 50+ test cases passing

### Phase 3 (Type Checking)
- [ ] Pattern type inference works
- [ ] Exhaustiveness checking
- [ ] Unreachable pattern detection
- [ ] 100+ test cases passing

### Phase 4 (Code Generation)
- [ ] Pattern matching generates valid C
- [ ] Generated C compiles without warnings
- [ ] Performance acceptable
- [ ] Integration tests passing

### Overall
- [ ] Can use pattern matching in AutoLang code
- [ ] Compiler code uses pattern matching extensively
- [ ] Zero memory safety issues
- [ ] Error messages clear and actionable

---

## 7. Related Documentation

- **[Plan 024]:** Ownership-Based Memory System (for closure support)
- **[Plan 026]:** Self-Hosting Compiler (depends on this plan)
- **[Rust Pattern Matching](https://doc.rust-lang.org/reference/patterns.html)** (reference)
- **[Match Expressions](https://doc.rust-lang.org/book/ch06-02-match.html)** (Rust book)

---

## 8. Open Questions

1. **Should we support range patterns?** e.g., `is x { 1..10 => print("small") }`
2. **Should we support slice patterns with variable length?** e.g., `[first, ..]`
3. **Should pattern matching be an expression or statement?** (Rust: expression)
4. **How to handle refutable patterns in let?** (Rust: error, Swift: crash)
5. **Should we support `@` bindings?** e.g., `is Point{x: @name }`

---

## 9. Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Pattern Representation | 2-3 weeks | Pattern enum, pretty-printing |
| 2. Pattern Parsing | 3-4 weeks | Parse all pattern types |
| 3. Type Checking | 3-4 weeks | Exhaustiveness, unreachability |
| 4. Code Generation | 2-3 weeks | Generate C for patterns |
| **Total** | **10-14 weeks** | **Full pattern matching** |

**Critical Path:** Phase 1 → 2 → 3 → 4

**Parallelization:**
- Phase 1 can start during Plan 024 Phase 1
- Phase 3 can overlap with Plan 024 Phase 2
- Phase 4 must wait for Plan 024 Phase 3 (borrow checker)

---

## 10. Conclusion

This plan implements comprehensive pattern matching for AutoLang, enabling the self-hosting compiler to be written in a clear, idiomatic style. By following Rust's proven approach while adapting to AutoLang's syntax and semantics, we achieve compiler-grade pattern matching capabilities.

**Key Benefits:**
1. **Expressive compiler code**: Clean AST traversal and deconstruction
2. **Type safety**: Compile-time exhaustiveness checking
3. **Performance**: Optimized pattern compilation
4. **Ergonomics**: Intuitive syntax matching AutoLang's design

Once complete, AutoLang will have pattern matching on par with Rust, making it suitable for writing production compilers and other complex systems software.
