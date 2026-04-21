# Plan 058: Type Alias Syntax Implementation

## Objective

Implement type alias syntax (`type X = Y`) to enable simplified type notation and complete Plan 055's vision of platform-transparent storage selection.

## Goal

Enable users to write:
```auto
type List<T> = List<T, DefaultStorage>

fn main() {
    let list = List<int>.new()  // Instead of List<int, Heap>.new()
}
```

## Background

### Current State
- ❌ Parser does NOT support `type X = Y` syntax
- ❌ No AST node for type aliases
- ❌ No evaluator support for type alias resolution
- ❌ No C transpiler support for generating typedef

### User Impact
Without type aliases, users must write verbose type notation:
```auto
// Current (verbose)
let pc_list = List<int, Heap>.new()
let mcu_list = List<int, InlineInt64>.new()

// Desired (concise)
let list = List<int>.new()  // Compiler selects storage
```

### Motivation
1. **Plan 055 Completion**: Essential for the "write once, run anywhere" vision
2. **User Convenience**: Reduce boilerplate in type-heavy code
3. **Documentation**: Type aliases improve code readability
4. **Platform Abstraction**: Hide platform-specific implementation details

## Design

### Syntax
```auto
// Simple alias
type IntAlias = int

// Generic alias
type List<T> = List<T, DefaultStorage>

// Multi-parameter alias
type Result<T, E> = May<T, E>

// Complex alias
type StringList = List<str, Heap>
```

### AST Representation
```rust
// In ast.rs
pub enum Stmt {
    // ... existing variants ...
    TypeAlias(TypeAlias),
}

pub struct TypeAlias {
    pub name: Name,
    pub params: Vec<Name>,  // Generic parameters (e.g., ["T"])
    pub target: Type,        // Target type (e.g., List<T, DefaultStorage>)
}
```

### Semantics
- **Compile-time only**: Type aliases are resolved during compilation, not runtime
- **Transparent**: Type aliases are completely interchangeable with their target types
- **No new types**: `type X = Y` does NOT create a new type, just an alias
- **Recursive checking**: Detect and prevent infinite recursion (e.g., `type A = A`)

### Scope Rules
- Type aliases follow the same scoping rules as `let` bindings
- Can shadow outer aliases in inner scopes
- Global aliases (top-level) visible throughout module
- Local aliases (inside blocks) visible only in that block

## Implementation Plan

### Phase 1: AST Support (1 hour)
**File**: `crates/auto-lang/src/ast.rs`

**Tasks**:
1. Add `TypeAlias` struct
2. Add `Stmt::TypeAlias(TypeAlias)` variant
3. Implement `Display` for `TypeAlias`
4. Update `ToNode` and `ToAtom` traits

**Code**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: Name,
    pub params: Vec<Name>,
    pub target: Type,
}

impl Display for TypeAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {}", self.name)?;
        if !self.params.is_empty() {
            write!(f, "<{}>", self.params.join(", "))?;
        }
        write!(f, " = {}", self.target)
    }
}
```

### Phase 2: Parser Support (2 hours)
**File**: `crates/auto-lang/src/parser.rs`

**Tasks**:
1. Add `parse_type_alias()` method
2. Call it from `parse_stmt()` when seeing `type` keyword
3. Parse generic parameters `<T, U>`
4. Parse target type after `=`
5. Error handling for invalid syntax

**Code**:
```rust
fn parse_type_alias(&mut self) -> AutoResult<Stmt> {
    self.expect(TokenKind::Type)?;

    let name = self.expect_ident()?;

    // Parse generic parameters
    let mut params = Vec::new();
    if self.is_kind(TokenKind::Lt) {
        self.next();
        while !self.is_kind(TokenKind::Gt) {
            params.push(self.expect_ident()?);
            if !self.is_kind(TokenKind::Gt) {
                self.expect(TokenKind::Comma)?;
            }
        }
        self.expect(TokenKind::Gt)?;
    }

    self.expect(TokenKind::Eq)?;

    let target = self.parse_type()?;

    self.expect(TokenKind::Semi)?;

    Ok(Stmt::TypeAlias(TypeAlias {
        name,
        params,
        target,
    }))
}
```

### Phase 3: Evaluator Support (2 hours)
**File**: `crates/auto-lang/src/eval.rs`

**Tasks**:
1. Add type alias storage to `Universe`
2. Add `define_type_alias()` method
3. Add `resolve_type_alias()` method
4. Handle `Stmt::TypeAlias` in `eval_stmt()`
5. Resolve aliases during type checking

**Code**:
```rust
// In Universe
type Aliases = HashMap<Name, (Vec<Name>, Type)>;

pub fn define_type_alias(&mut self, name: Name, params: Vec<Name>, target: Type) {
    self.aliases.insert(name, (params, target));
}

pub fn resolve_type_alias(&self, name: &Name, args: &[Type]) -> AutoResult<Type> {
    if let Some((params, target)) = self.aliases.get(name) {
        // Substitute generic parameters with concrete types
        // ...
        Ok(target.clone())
    } else {
        Err(...)
    }
}
```

### Phase 4: C Transpiler Support (2 hours)
**File**: `crates/auto-lang/src/trans/c.rs`

**Tasks**:
1. Handle `Stmt::TypeAlias` in statement transpilation
2. Generate C `typedef` statements
3. Handle generic aliases (may need macro expansion)
4. Register alias in scope for later references

**Code**:
```rust
Stmt::TypeAlias(alias) => {
    // Simple alias: typedef int IntAlias;
    // Generic: typedef struct list_int* List_int;
    let alias_name = self.c_type_name(&Type::Ident(alias.name.clone()));

    match &alias.target {
        Type::Ident(target_name) => {
            let target_c = self.c_type_simple(target_name);
            writeln!(out, "typedef {} {};", target_c, alias_name)?;
        }
        _ => {
            // Complex type - might need struct definition
            let target_c = self.c_type_name(&alias.target);
            writeln!(out, "typedef {} {};", target_c, alias_name)?;
        }
    }
}
```

### Phase 5: Type Resolution (2 hours)
**Files**: `parser.rs`, `eval.rs`

**Tasks**:
1. Modify `parse_type()` to check for aliases
2. Substitute aliases during parsing
3. Handle generic parameter substitution
4. Detect infinite recursion

**Code**:
```rust
fn parse_type(&mut self) -> AutoResult<Type> {
    let name = self.expect_ident()?;

    // Check if it's an alias
    if let Some((params, target)) = self.universe.borrow().lookup_alias(&name) {
        // Parse type arguments
        let mut args = Vec::new();
        if self.is_kind(TokenKind::Lt) {
            self.next();
            while !self.is_kind(TokenKind::Gt) {
                args.push(self.parse_type()?);
                if !self.is_kind(TokenKind::Gt) {
                    self.expect(TokenKind::Comma)?;
                }
            }
            self.expect(TokenKind::Gt)?;
        }

        // Substitute and return target type
        return self.substitute_alias(target, params, args);
    }

    // Regular type parsing...
}
```

### Phase 6: Testing (2 hours)
**Files**: Test files in `crates/auto-lang/test/`

**Tasks**:
1. Create a2c test case for simple type alias
2. Create a2c test case for generic type alias
3. Create a2c test case for List alias (Plan 055 integration)
4. Test recursive alias detection
5. Test shadowing behavior

**Test Cases**:
```auto
// 083_type_alias_simple.at
type IntAlias = int
fn main() {
    let x: IntAlias = 42
}

// 084_type_alias_generic.at
type List<T> = List<T, Heap>
fn main() {
    let list = List<int>.new()
    return list.len()
}

// 085_type_alias_recursive.at (should error)
type A = A  // Error: recursive type alias
```

## Success Criteria

### Phase 1: AST
- ✅ `TypeAlias` struct compiles
- ✅ `Display` works correctly
- ✅ All pattern matches updated

### Phase 2: Parser
- ✅ `type X = Y;` parses correctly
- ✅ `type List<T> = List<T, DefaultStorage>;` parses correctly
- ✅ Syntax errors produce helpful messages

### Phase 3: Evaluator
- ✅ Type aliases are stored in Universe
- ✅ Aliases are resolved during type checking
- ✅ Recursive aliases are detected and rejected

### Phase 4: C Transpiler
- ✅ Simple aliases generate `typedef X Y;`
- ✅ Generic aliases generate appropriate C code
- ✅ Aliases work in a2c tests

### Phase 5: Type Resolution
- ✅ `List<int>` expands to `List<int, DefaultStorage>`
- ✅ Generic parameters are substituted correctly
- ✅ No infinite loops in resolution

### Phase 6: Testing
- ✅ All new a2c tests pass
- ✅ Existing tests still pass
- ✅ Plan 055 integration test passes

## Integration with Plan 055

After implementing type aliases, update [stdlib/auto/prelude.at](stdlib/auto/prelude.at):

```auto
// Add after line 29
type List<T> = List<T, DefaultStorage>
```

This enables the desired usage:
```auto
fn main() {
    let list = List<int>.new()  // Automatically selects Heap or InlineInt64
    list.push(1)
    return list.len()
}
```

## Timeline

- **Phase 1**: 1 hour (AST)
- **Phase 2**: 2 hours (Parser)
- **Phase 3**: 2 hours (Evaluator)
- **Phase 4**: 2 hours (C Transpiler)
- **Phase 5**: 2 hours (Type Resolution)
- **Phase 6**: 2 hours (Testing)
- **Total**: 11 hours

## Risks and Mitigations

### Risk 1: Generic Type Substitution Complexity
- **Impact**: High - May require significant type system changes
- **Mitigation**: Start with simple aliases, add generics incrementally
- **Fallback**: Defer generic aliases to future plan

### Risk 2: C Transpiler Limitations
- **Impact**: Medium - C typedef has limited expressiveness
- **Mitigation**: Generate macro-based code for complex generics
- **Fallback**: Document that some aliases are VM-only

### Risk 3: Performance
- **Impact**: Low - Type resolution is compile-time only
- **Mitigation**: Cache resolved types
- **Fallback**: None needed

## Future Enhancements

1. **Type alias constraints**: `type List<T: Add> = List<T>`
2. **Alias visibility modifiers**: `pub type X = Y`
3. **Alias documentation**: Attach doc comments to aliases
4. **Alias inference**: Automatically create aliases for common patterns
5. **Module-level aliases**: Re-export aliases with `use`

## Related Plans

- **Plan 055**: Storage Environment Injection (requires this plan for completion)
- **Plan 057**: Generic Specs (uses similar generic parameter syntax)
- **Plan 048**: Generic Types (provides foundation for generic aliases)

## Status

📋 **Planning Phase** - Ready to implement
