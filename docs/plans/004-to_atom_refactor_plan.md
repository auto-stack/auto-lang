# ToAtom Interface Refactoring Plan

## Overview

Refactor the Rust implementation's `ToAtom` trait to return `auto_val::AutoStr` instead of `Value`. Each AST struct will implement `AtomWriter` to generate ATOM format string representations.

**Current State:**
- `ToAtom::to_atom()` returns `Value` (30+ implementations)
- `AtomWriter` only implemented by `Expr`, `Pair`, `Vec<T>`
- 82 call sites using `.to_atom().to_node()` pattern

**Target State:**
- `ToAtom::to_atom()` returns `AutoStr`
- All AST types implement `AtomWriter`
- Direct string representation in ATOM format (Lisp-style S-expressions)

## Design Decisions

### 1. Updated Trait Signatures

```rust
// Before:
pub trait ToAtom {
    fn to_atom(&self) -> Value;
}

// After:
pub trait ToAtom {
    fn to_atom(&self) -> AutoStr;
}

// Enhanced AtomWriter trait (unchanged):
pub trait AtomWriter {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()>;
}
```

### 2. Helper Trait for Convenience

```rust
pub trait ToAtomStr {
    fn to_atom_str(&self) -> AutoStr;
}

impl<T: AtomWriter> ToAtomStr for T {
    fn to_atom_str(&self) -> AutoStr {
        let mut buf = Vec::new();
        let _ = self.write_atom(&mut buf);
        String::from_utf8(buf).unwrap_or_default().into()
    }
}
```

### 3. Coexistence Strategy

**Keep both traits:**
- `ToAtom` → `AutoStr` for serialization/text representation
- `ToNode` → `Node` for tree construction (unchanged)

**Update call sites:**
```rust
// Old: node.add_kid(expr.to_atom().to_node());
// New: node.add_kid(expr.to_node());
```

### 4. ATOM String Format Specification

Use Lisp-style S-expressions for complex structures:

```
(if (branch cond body) (branch cond2 body2) (else else-body))
(for iter=(name x) range=(range 0 10) body=(body stmt1 stmt2))
(fn name=add params=(param x int) return=int body=(body stmt))
(let name=x type=int expr=42)
```

## Implementation Phases

### Phase 1: Foundation (No Breaking Changes)

**Location:** `crates/auto-lang/src/ast.rs`

1. Add `ToAtomStr` helper trait
2. Update `Expr::to_atom()` to use `to_atom_str()` pattern
3. Add comprehensive documentation
4. Write unit tests for helper trait

**Deliverables:**
- New `ToAtomStr` trait with blanket implementation
- Updated `Expr::to_atom()` returning `AutoStr`
- Test coverage for helper functionality

### Phase 2: Implement AtomWriter for All Types

**Implementation Order (by complexity):**

#### 2.1 Primitives (Week 1)
- `Type` - `crates/auto-lang/src/ast/types.rs`
- `Key` - `crates/auto-lang/src/ast/types.rs`
- `Pair` - `crates/auto-lang/src/ast/types.rs`

#### 2.2 Simple Structures (Week 2)
- `Param` - `crates/auto-lang/src/ast/fun.rs`
- `EnumItem` - `crates/auto-lang/src/ast/enums.rs`
- `TagField` - `crates/auto-lang/src/ast/tag.rs`
- `UnionField` - `crates/auto-lang/src/ast/union.rs`
- `Alias` - `crates/auto-lang/src/ast/alias.rs`
- `Range` - `crates/auto-lang/src/ast/range.rs`
- `Break` - `crates/auto-lang/src/ast/for_.rs`

#### 2.3 Medium Complexity (Week 3)
- `Branch` - `crates/auto-lang/src/ast/branch.rs`
- `Arg` - `crates/auto-lang/src/ast/call.rs`
- `Args` - `crates/auto-lang/src/ast/call.rs`
- `Iter` - `crates/auto-lang/src/ast/for_.rs`
- `Body` - `crates/auto-lang/src/ast/body.rs`
- `Member` - `crates/auto-lang/src/ast/types.rs`
- `Node` - `crates/auto-lang/src/ast/node.rs`

#### 2.4 Complex Structures (Week 4)
- `If` - `crates/auto-lang/src/ast/if_.rs`
- `For` - `crates/auto-lang/src/ast/for_.rs`
- `Fn` - `crates/auto-lang/src/ast/fun.rs`
- `Store` - `crates/auto-lang/src/ast/store.rs`
- `Is` - `crates/auto-lang/src/ast/is.rs`
- `IsBranch` - `crates/auto-lang/src/ast/is.rs`

#### 2.5 Declarations (Week 5)
- `TypeDecl` - `crates/auto-lang/src/ast/types.rs`
- `EnumDecl` - `crates/auto-lang/src/ast/enums.rs`
- `Union` - `crates/auto-lang/src/ast/union.rs`
- `Tag` - `crates/auto-lang/src/ast/tag.rs`

#### 2.6 Event Handling (Week 5)
- `Event` - `crates/auto-lang/src/ast/on.rs`
- `Arrow` - `crates/auto-lang/src/ast/on.rs`
- `CondArrow` - `crates/auto-lang/src/ast/on.rs`
- `OnEvents` - `crates/auto-lang/src/ast/on.rs`

#### 2.7 Top-Level (Week 6)
- `Use` - `crates/auto-lang/src/ast/use_.rs`
- `Stmt` - `crates/auto-lang/src/ast.rs`
- `Expr` - `crates/auto-lang/src/ast.rs` (already done in Phase 1)
- `Code` - `crates/auto-lang/src/ast.rs`

**Implementation Template for Each Type:**

```rust
// Example: If struct
impl AtomWriter for If {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "(if ")?;
        for branch in &self.branches {
            write!(f, " {}", branch.to_atom_str())?;
        }
        if let Some(else_body) = &self.else_ {
            write!(f, " (else {})", else_body.to_atom_str())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl ToAtom for If {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()  // Uses ToAtomStr helper
    }
}
```

### Phase 3: Change Trait Signature (Week 7)

**Location:** `crates/auto-lang/src/ast.rs`

1. Update `ToAtom` trait definition:
   ```rust
   pub trait ToAtom {
       fn to_atom(&self) -> AutoStr;  // Changed from Value
   }
   ```

2. Update all 30+ `ToAtom` implementations to return `AutoStr`
3. Verify all implementations compile
4. Add backward compatibility layer if needed (temporary)

**Deliverables:**
- Updated trait definition
- All implementations returning `AutoStr`
- Code compiles without errors

### Phase 4: Migrate Call Sites (Week 7)

**Files to Update:**

1. **crates/auto-lang/src/ast.rs** (~60 call sites)
   - Search pattern: `.to_atom().to_node()`
   - Replace with: `.to_node()`
   - Focus on `Stmt`, `Expr`, `Code` implementations

2. **crates/auto-lang/src/universe.rs** (~2 call sites)
   - Update variable binding code
   - Update scope management

3. **crates/auto-lang/src/config.rs** (~1 call site)
   - Update configuration parsing

**Migration Strategy:**
```rust
// Before:
node.add_kid(expr.to_atom().to_node());

// After:
node.add_kid(expr.to_node());  // Direct call, more efficient
```

### Phase 5: Testing & Documentation (Week 8)

#### 5.1 Unit Tests

For each `AtomWriter` implementation:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if_write_atom() {
        let if_stmt = If {
            branches: vec![/* ... */],
            else_: None,
        };
        let result = if_stmt.to_atom();
        assert!(result.starts_with("(if "));
        assert!(result.ends_with(")"));
    }
}
```

#### 5.2 Round-Trip Tests

Test serialization and deserialization:
```rust
#[test]
fn test_round_trip_if() {
    let original = /* construct If */;
    let atom_str = original.to_atom();
    let parsed = parse_if(&atom_str).unwrap();
    assert_eq!(original, parsed);
}
```

#### 5.3 Integration Tests

- Test full compilation pipeline
- Test code generation from AST
- Test transpiler integration

#### 5.4 Documentation

Update `CLAUDE.md` with:
- New `ToAtom` trait documentation
- `AtomWriter` implementation guidelines
- String format specification
- Migration guide for users

### Phase 6: Performance & Cleanup (Week 9)

1. **Performance Benchmarks**
   - Compare `AutoStr` vs `Value` performance
   - Measure memory allocation differences
   - Optimize hot paths if needed

2. **Remove Compatibility Layer**
   - Remove any temporary backward compatibility code
   - Clean up unused imports

3. **Final Review**
   - Ensure all tests pass
   - Verify documentation is complete
   - Check for any remaining `.to_atom().to_node()` patterns

## Critical Files to Modify

### Core Files (Must Modify)

1. **D:\autostack\auto-lang\crates\auto-lang\src\ast.rs**
   - Update `ToAtom` trait definition
   - Add `ToAtomStr` helper trait
   - Update `Expr::to_atom()`, `Stmt::to_atom()`, `Code::to_atom()`
   - Update ~60 call sites

2. **D:\autostack\auto-lang\crates\auto-lang\src\ast\types.rs**
   - Implement `AtomWriter` for: Type, Key, Pair, Member, TypeDecl
   - Update all `ToAtom` implementations

3. **D:\autostack\auto-lang\crates\auto-lang\src\ast\if_.rs**
   - Implement `AtomWriter for If`
   - Update `ToAtom for If`

4. **D:\autostack\auto-lang\crates\auto-lang\src\ast\for_.rs**
   - Implement `AtomWriter` for: For, Iter, Break
   - Update all `ToAtom` implementations

5. **D:\autostack\auto-lang\crates\auto-lang\src\ast\fun.rs**
   - Implement `AtomWriter` for: Fn, Param
   - Update all `ToAtom` implementations

### Additional Files to Modify

**Control Flow:**
- `crates/auto-lang/src/ast/is.rs` - Is, IsBranch
- `crates/auto-lang/src/ast/branch.rs` - Branch

**Declarations:**
- `crates/auto-lang/src/ast/store.rs` - Store
- `crates/auto-lang/src/ast/enums.rs` - EnumDecl, EnumItem
- `crates/auto-lang/src/ast/tag.rs` - Tag, TagField
- `crates/auto-lang/src/ast/union.rs` - Union, UnionField
- `crates/auto-lang/src/ast/alias.rs` - Alias

**Expressions:**
- `crates/auto-lang/src/ast/call.rs` - Call, Args, Arg
- `crates/auto-lang/src/ast/node.rs` - Node
- `crates/auto-lang/src/ast/range.rs` - Range
- `crates/auto-lang/src/ast/body.rs` - Body

**Other:**
- `crates/auto-lang/src/ast/use_.rs` - Use
- `crates/auto-lang/src/ast/on.rs` - OnEvents, Event, Arrow, CondArrow

**Call Sites:**
- `crates/auto-lang/src/universe.rs` - 2 call sites
- `crates/auto-lang/src/config.rs` - 1 call site

## Testing Strategy

### 1. Unit Tests
- Each `AtomWriter` implementation gets its own test
- Test edge cases and error conditions
- Verify string format correctness

### 2. Round-Trip Tests
- AST → `AutoStr` → parsed AST
- Verify structural equivalence
- Test with complex nested structures

### 3. Integration Tests
- Full compilation pipeline tests
- Transpiler output tests
- Code generation tests

### 4. Performance Tests
- Benchmark before/after performance
- Memory allocation profiling
- Identify and optimize hot paths

## Risk Mitigation

### Risk 1: Breaking Existing Code
**Mitigation:**
- Keep `ToNode` trait unchanged
- Incrementally migrate call sites
- Comprehensive test coverage

### Risk 2: String Format Ambiguity
**Mitigation:**
- Use well-established Lisp-style S-expressions
- Comprehensive format documentation
- Round-trip tests ensure parseability

### Risk 3: Performance Regression
**Mitigation:**
- `AutoStr` uses copy-on-write (efficient)
- Keep `ToNode` for tree construction (optimized path)
- Benchmark before/after

### Risk 4: Test Coverage Gaps
**Mitigation:**
- Implement tests alongside each `AtomWriter`
- Require test coverage before merging
- Add integration tests for full pipeline

## Success Criteria

1. ✅ All 30+ AST types implement `AtomWriter`
2. ✅ `ToAtom::to_atom()` returns `AutoStr`
3. ✅ All 82 `.to_atom().to_node()` call sites migrated to `.to_node()`
4. ✅ All tests pass (unit, integration, round-trip)
5. ✅ No performance regression
6. ✅ Documentation updated and complete
7. ✅ Code compiles without warnings

## Estimated Timeline

**8-9 weeks total:**
- Week 1: Foundation + Primitives
- Week 2: Simple structures
- Week 3: Medium complexity structures
- Week 4: Complex structures
- Week 5: Declarations + Event handling
- Week 6: Top-level types
- Week 7: Change trait signature + Migrate call sites
- Week 8: Testing + Documentation
- Week 9: Performance + Cleanup

## Next Steps

1. Review and approve this plan
2. Set up feature branch
3. Begin Phase 1 implementation
4. Create tracking issue for each phase
5. Regular progress reviews
