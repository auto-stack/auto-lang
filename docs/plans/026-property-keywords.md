# Property Keywords Implementation Plan

**Objective**: Transform `view`/`mut`/`take` from prefix keywords to **property keywords** (属性关键字) using dot notation

**Status**: 🔄 Planning
**Priority**: HIGH - Core syntax transformation for unified dot notation design
**Dependencies**: None
**Started**: 2025-01-15

---

## Executive Summary

Convert the borrow/ownership keywords from prefix syntax to postfix property syntax, aligning with AutoLang's unified dot notation design philosophy. This makes the syntax more consistent and enables better IDE support.

### Syntax Transformation

**Before (Prefix):**
```auto
let slice = view s
let mut_ref = mut s
let s2 = take s
```

**After (Property Keyword):**
```auto
let slice = s.view
let mut_ref = s.mut
let s2 = s.take
```

**Function Parameters (unchanged):**
```auto
fn doublize(mut a int) int  // mut stays BEFORE parameter name
```

---

## Design Principles

### 1. Dual Syntax Rule

- **Expression context**: Property keywords are **postfix** (after the operand)
  ```auto
  s.view  // ✅ Correct
  view s  // ❌ Old syntax (deprecated)
  ```

- **Function parameter context**: Property keywords are **prefix** (before parameter name)
  ```auto
  fn process(mut data int)  // ✅ Correct
  fn process(data mut int)  // ❌ Wrong
  ```

### 2. Chaining Support

Property keywords support method chaining like regular properties:
```auto
let data = sensor.?.mut.buffer.@.*
let count = data.view.len.as.u64
```

### 3. Keyword Coloring

Property keywords (`.view`, `.mut`, `.take`) should be highlighted as **keywords** by the IDE/syntax highlighter, not as regular identifiers.

---

## Implementation Phases

### Phase 1: Lexer & Token Updates

**File**: `crates/auto-lang/src/lexer.rs`

**Changes**:
1. Modify lexer to recognize `.<keyword>` patterns
2. When encountering `.view`, `.mut`, or `.take`, tokenize them as special **property keyword tokens**
3. Create new token kinds:
   - `TokenKind::DotView` (for `.view`)
   - `TokenKind::DotMut` (for `.mut`)
   - `TokenKind::DotTake` (for `.take`)

**Example**:
```rust
// In lexer.rs
if self.peek_char() == '.' {
    self.next_char(); // consume '.'
    let ident = self.read_ident()?;

    match ident.text.as_str() {
        "view" => return TokenKind::DotView,
        "mut" => return TokenKind::DotMut,
        "take" => return TokenKind::DotTake,
        _ => {
            // Regular property access (e.g., .length)
            return TokenKind::Ident;
        }
    }
}
```

**File**: `crates/auto-lang/src/token.rs`

**Changes**:
```rust
pub enum TokenKind {
    // ... existing tokens ...

    // Property keywords (Phase 3: property syntax)
    DotView,  // .view
    DotMut,   // .mut
    DotTake,  // .take
}
```

---

### Phase 2: Parser Updates

**File**: `crates/auto-lang/src/parser.rs`

**Current Behavior** (lines 675-689):
```rust
// Prefix parsing (OLD)
TokenKind::View => {
    self.next(); // skip view
    let expr = self.expr_pratt(0)?;
    Expr::View(Box::new(expr))
}
```

**New Behavior** (Postfix parsing):
```rust
// In expr_pratt(), handle dot-access for property keywords
TokenKind::Dot => {
    self.next(); // consume '.'
    match self.kind() {
        TokenKind::View => {
            self.next(); // consume 'view'
            // lhs is the target (e.g., 's' in 's.view')
            Expr::View(lhs)
        }
        TokenKind::Mut => {
            self.next(); // consume 'mut'
            Expr::Mut(lhs)
        }
        TokenKind::Take => {
            self.next(); // consume 'take'
            Expr::Take(lhs)
        }
        _ => {
            // Regular property access (e.g., obj.property)
            let prop = self.ident()?;
            Expr::Dot(lhs, prop)
        }
    }
}
```

**IMPORTANT**: Keep the function parameter parsing unchanged:
```rust
// In function parameter parsing (line ~2356)
// This stays as-is: mut comes BEFORE parameter
fn parse_param(&mut self) -> AutoResult<Param> {
    let mutable = if self.is_kind(TokenKind::Mut) {
        self.next();
        true
    } else {
        false
    };

    let name = self.ident()?;
    let ty = self.type_annotation()?;

    Ok(Param { name, ty, mutable })
}
```

---

### Phase 3: AST Updates

**File**: `crates/auto-lang/src/ast.rs`

**Current Structure** (unchanged):
```rust
pub enum Expr {
    // ... other variants ...
    View(Box<Expr>),  // view e
    Mut(Box<Expr>),   // mut e
    Take(Box<Expr>),  // take e
}
```

**No changes needed!** The AST structure remains the same. Only the parsing direction changes (prefix → postfix).

**Display updates** (lines 316-318):
```rust
// Update display to show postfix syntax
Expr::View(e) => write!(f, "({}.view)", e),
Expr::Mut(e) => write!(f, "({}.mut)", e),
Expr::Take(e) => write!(f, "({}.take)", e),
```

---

### Phase 4: Evaluator Updates

**File**: `crates/auto-lang/src/eval.rs`

**No semantic changes!** The evaluator logic remains identical because the AST structure is the same. The only difference is how the AST is built (parser), not how it's evaluated.

**Verification**:
```rust
// Current evaluation (unchanged)
Expr::View(e) => {
    let target = self.eval_expr(*e)?;
    // Borrow checker creates immutable borrow
    self.borrow_checker.check_borrow(&e, BorrowKind::View, lifetime)?;
    // ... rest of logic
}
```

---

### Phase 5: Borrow Checker Updates

**File**: `crates/auto-lang/src/ownership/borrow.rs`

**No semantic changes!** The borrow checker works with the AST, which has the same structure.

**Verification**: Ensure conflict detection works with the new syntax:
```auto
// These should still conflict:
let s = str_new("hello", 5)
let v1 = s.view  // immutable borrow #1
let v2 = s.mut   // ERROR: conflicts with view
```

---

### Phase 6: C Transpiler Updates

**File**: `crates/auto-lang/src/trans/c.rs`

**Current Output** (lines 762-779):
```c
// Input: view s
// Output: &(s)

// Input: mut s
// Output: &(s)
```

**No changes needed!** The generated C code remains the same because the AST structure is unchanged.

**Verification test cases**:
```auto
// Input
let s = str_new("hello", 5)
let slice = s.view

// Output (should be same)
char* s = str_new("hello", 5);
unknown slice = &(s);
```

---

### Phase 7: Test Suite Updates

**Critical**: All existing tests use the old prefix syntax and must be updated.

#### Test Files to Update

1. **Borrow Checker Tests** (`crates/auto-lang/src/ownership/borrow.rs`)
   - Update all test cases from `view x` to `x.view`
   - Update all test cases from `mut x` to `x.mut`
   - Update all test cases from `take x` to `x.take`

2. **Evaluator Tests** (search for `view | mut | take`)
   - Update test code to use new syntax

3. **Integration Tests** (`crates/auto-lang/test/a2c/03x_borrow_*`)
   - Update `.at` source files
   - Update `.expected.c` output files (should remain the same!)

#### Example Test Transformation

**Before**:
```auto
#[test]
fn test_borrow_view_basic() {
    let s = str_new("hello", 5)
    let slice = view s
    assert(str_len(slice) == 5)
}
```

**After**:
```auto
#[test]
fn test_borrow_view_basic() {
    let s = str_new("hello", 5)
    let slice = s.view
    assert(str_len(slice) == 5)
}
```

#### Estimated Test Updates
- **Borrow checker tests**: ~23 tests
- **Evaluator tests**: ~10 tests
- **Integration tests**: 3-4 test files
- **Total**: ~40 test cases to update

---

### Phase 8: Documentation Updates

**Files to Update**:

1. **Ownership Module** (`crates/auto-lang/src/ownership/mod.rs`)
   ```rust
   //! # Example
   //!
   //! ```auto
   //! // View borrow (immutable reference)
   //! let s = str_new("hello", 5)
   //! let slice = s.view     // Property keyword: like &s in Rust
   //! let len = str_len(slice)
   //!
   //! // Mut borrow (mutable reference)
   //! let s = str_new("hello", 5)
   //! let mut_ref = s.mut    // Property keyword: like &mut s in Rust
   //! str_push(mut_ref, '!')
   //!
   //! // Take (move semantics)
   //! let s1 = str_new("hello", 5)
   //! let s2 = s1.take       // Property keyword: transfer ownership
   //! ```
   ```

2. **Plan 024** (`docs/plans/024-ownership-first-implementation.md`)
   - Update all examples to use new syntax

3. **Dot Notation Design** (`docs/language/design/dot-notation.md`)
   - Already uses new syntax ✅
   - Add note about implementation status

4. **CLAUDE.md** (if it mentions view/mut/take)
   - Update examples to new syntax

---

## Migration Strategy

### Backward Compatibility

**Option 1: Hard Break** (Recommended)
- Remove old prefix syntax support entirely
- Update all tests in one PR
- Clear message: "Syntax has changed to property keywords"

**Option 2: Transition Period** (More complex)
- Support both syntaxes temporarily
- Emit deprecation warnings for old syntax
- Remove in next major version

**Recommendation**: Go with **Option 1** (Hard Break) because:
- This is still early in development (Phase 3)
- Few external users affected
- Simpler implementation
- Clearer syntax going forward

---

## Success Criteria

- ✅ Lexer recognizes `.view`, `.mut`, `.take` as property keyword tokens
- ✅ Parser handles property keywords in postfix position
- ✅ Function parameters still use prefix `mut` (unchanged)
- ✅ All borrow checker tests pass with new syntax
- ✅ All evaluator tests pass with new syntax
- ✅ All integration tests pass
- ✅ Generated C code is identical to before
- ✅ Documentation updated with new syntax examples
- ✅ Zero compilation warnings

---

## Estimated Effort

| Phase | File | Lines Changed | Time |
|-------|------|---------------|------|
| 1. Lexer | lexer.rs | +30 | 2 hours |
| 2. Parser | parser.rs | ~50 (modifications) | 3 hours |
| 3. AST | ast.rs | ~10 (display only) | 1 hour |
| 4. Evaluator | eval.rs | 0 (no change) | 0 hours |
| 5. Borrow Checker | borrow.rs | 0 (no change) | 0 hours |
| 6. C Transpiler | c.rs | 0 (no change) | 0 hours |
| 7. Tests | ~40 test cases | ~100 | 3 hours |
| 8. Documentation | ~5 files | ~200 | 2 hours |

**Total**: ~11 hours | ~390 lines changed

---

## Implementation Order

1. **Step 1**: Update token definitions (add DotView, DotMut, DotTake)
2. **Step 2**: Update lexer to recognize property keywords
3. **Step 3**: Update parser to handle postfix property keywords
4. **Step 4**: Update AST display formatting
5. **Step 5**: Update all test cases to new syntax
6. **Step 6**: Update documentation
7. **Step 7**: Run full test suite and verify zero regressions
8. **Step 8**: Commit and push

---

## Testing Checklist

- [ ] Unit tests for lexer property keyword recognition
- [ ] Unit tests for parser postfix handling
- [ ] All borrow checker tests pass (~23 tests)
- [ ] All evaluator tests pass (~10 tests)
- [ ] All a2c integration tests pass
- [ ] C output is identical to before (transpiler tests)
- [ ] Zero compilation warnings
- [ ] Documentation examples compile and run

---

## Open Questions

1. **Q**: Should we support both syntaxes during transition?
   **A**: No, hard break is cleaner for early development

2. **Q**: What about IDE syntax highlighting?
   **A**: Update TextMate grammar / VSCode extension separately

3. **Q**: Will this affect the `hold` keyword?
   **A**: `hold` is a statement keyword, not affected by this change

4. **Q**: Should we add other property operators (.?, .@, .*) at the same time?
   **A**: No, keep this PR focused on view/mut/take only

---

## Next Steps

1. **Review this plan** with user approval
2. **Begin Phase 1**: Lexer updates
3. **Create feature branch**: `feature/property-keywords`
4. **Implement phases 1-8** in order
5. **Final review**: Full test suite pass
6. **Merge to master**

---

**Plan Status**: Ready for Implementation
**Next Phase**: Phase 1 - Lexer & Token Updates
**Estimated Completion**: 1-2 days from approval
