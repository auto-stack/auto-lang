# Plan 122: Value Access and Assignment Refactoring

## Status: ✅ COMPLETE (2025-03-11)

## Summary

Implemented the `.move` accessor as the replacement for deprecated `.take`:
- Added `Op::DotMove` and `Expr::Move` to AST
- Updated lexer to recognize `.move` keyword
- Updated parser to parse `.move` as postfix operator
- Added deprecation warning when `.take` is used (suggests `.move`)
- Updated transpilers (C/Rust) to handle `Expr::Move`
- Fixed string result conversion bug in `lib.rs`
- Renamed test files from `borrow_take` to `borrow_move`

## Objective

Refactor AutoLang's value access and assignment system according to the design document `docs/design/value-access.md`, implementing the "Trinity of Resources" (`view`, `mut`, `move`) and deprecating `take` and `copy` keywords.

## Current Implementation Status

Based on `test/param_passing/PHASE_7_REPORT.md`:

| Component | Status | Notes |
|-----------|--------|-------|
| **Parser** | ✅ Working | Can parse `view`, `mut`, `copy`, `take` keywords |
| **AST** | ✅ Working | Has `ParamMode` enum (View, Mut, Copy, Take) |
| **VM Engine** | ✅ Working | Supports reference instructions |
| **Type System** | ✅ Working | Has `is_optimized_by_value()` method |
| **Codegen** | ❌ Not Working | Parameters still passed by value (Phase 4 incomplete) |
| **Type Checker** | ❌ Not Working | Immutability enforcement not implemented |
| **Call-site accessors** | ⚠️ Partial | `.view`, `.mut`, `.take` work but semantics incomplete |

**Key Issue**: Parameter mode keywords (`view`, `mut`, `copy`, `take`) are currently "syntactic sugar" - they don't affect actual parameter passing behavior.

---

## Design Requirements (from value-access.md)

### 1. The Trinity of Resources

| Mode | Semantics | Call Site | Def Site |
|------|-----------|-----------|----------|
| **view** | Read-only borrow (O(1)) | `obj.view` or implicit | `fn foo(x T)` (default) |
| **mut** | Mutable borrow (O(1)) | `obj.mut` | `fn foo(x mut T)` |
| **move** | Ownership transfer (O(1)) | `obj.move` | `fn foo(x move T)` |
| **clone()** | Deep copy (O(N)) | `obj.clone()` (explicit!) | N/A |

### 2. Key Changes

1. **`take` → `move`**: Rename keyword to avoid confusion with collection `.take(n)`
2. **Remove `copy`**: Dangerous implicit O(N) operation
3. **Default to View**: No modifier = view (implicit)
4. **Explicit `.clone()`**: Deep copy requires function call syntax (warning!)

---

## Phase 1: AST Updates (1 day)

### 1.1 Update ParamMode Enum

**File**: `crates/auto-lang/src/ast.rs`

```rust
// Current
pub enum ParamMode {
    View,   // immutable borrow
    Mut,    // mutable borrow
    Copy,   // DEPRECATED - deep copy
    Take,   // DEPRECATED - rename to Move
}

// Target
pub enum ParamMode {
    View,   // immutable borrow (default, O(1))
    Mut,    // mutable borrow (explicit, O(1))
    Move,   // ownership transfer (renamed from Take, O(1))
    // Copy removed - use .clone() method instead
}
```

### 1.2 Add AccessMode for Call Site

**File**: `crates/auto-lang/src/ast.rs`

```rust
/// Call-site access mode
#[derive(Debug, Clone, PartialEq)]
pub enum AccessMode {
    View,   // &T - immutable borrow
    Mut,    // &mut T - mutable borrow
    Move,   // moves value, invalidates source
    Clone,  // .clone() - explicit deep copy
}
```

---

## Phase 2: Lexer Updates (0.5 days)

### 2.1 Update Keywords

**File**: `crates/auto-lang/src/lexer.rs`

Changes needed:
- Keep: `"view"` → `Keyword::View`
- Keep: `"mut"` → `Keyword::Mut`
- Add: `"move"` → `Keyword::Move`
- Deprecate: `"take"` → emit warning, map to `Move`
- Remove: `"copy"` → emit error

---

## Phase 3: Parser Updates (1 day)

### 3.1 Update Parameter Mode Parsing

**File**: `crates/auto-lang/src/parser.rs`

```rust
fn parse_param_mode(&mut self) -> AutoResult<ParamMode> {
    if self.check_keyword("view") {
        self.next();
        ParamMode::View
    } else if self.check_keyword("mut") {
        self.next();
        ParamMode::Mut
    } else if self.check_keyword("move") {
        self.next();
        ParamMode::Move
    } else if self.check_keyword("take") {
        self.warn("'.take' is deprecated. Use '.move' instead.");
        self.next();
        ParamMode::Move
    } else if self.check_keyword("copy") {
        self.error("'copy' removed. Use 'move' and .clone() at call site.")?;
        ParamMode::Move
    } else {
        ParamMode::View  // Default
    }
}
```

### 3.2 Update Call-Site Accessor Parsing

Support these accessors:
- `.view` → `AccessMode::View`
- `.mut` → `AccessMode::Mut`
- `.move` → `AccessMode::Move`
- `.clone()` → `AccessMode::Clone` (requires parentheses!)
- `.take` → emit deprecation warning, treat as `.move`

---

## Phase 4: Codegen - Smart Parameter Compilation (2 days)

### 4.1 Implement Parameter Mode Codegen

**File**: `crates/auto-lang/src/codegen.rs`

This is the critical missing piece from Plan 088:

```rust
fn compile_param(&mut self, arg: &Expr, mode: ParamMode) -> AutoResult<()> {
    match mode {
        ParamMode::View => self.emit_load_view_ref(arg),
        ParamMode::Mut => self.emit_load_mut_ref(arg),
        ParamMode::Move => self.emit_load_move(arg),
    }
}
```

### 4.2 Call-Site Accessor Handling

```rust
fn compile_accessor(&mut self, base: &Expr, mode: AccessMode) -> AutoResult<()> {
    match mode {
        AccessMode::View => { /* take immutable ref */ }
        AccessMode::Mut => { /* take mutable ref */ }
        AccessMode::Move => { /* transfer ownership */ }
        AccessMode::Clone => { /* call .clone() method */ }
    }
}
```

---

## Phase 5: Move Semantics (1.5 days)

### 5.1 Track Moved Variables

**File**: `crates/auto-lang/src/borrow_checker.rs` (new file)

```rust
pub struct BorrowChecker {
    moved_vars: HashSet<String>,
}

impl BorrowChecker {
    pub fn check_usable(&self, name: &str) -> Result<(), Error> {
        if self.moved_vars.contains(name) {
            Err(Error::UseAfterMove(name.to_string()))
        } else {
            Ok(())
        }
    }
    
    pub fn mark_moved(&mut self, name: &str) {
        self.moved_vars.insert(name.to_string());
    }
}
```

---

## Phase 6: Transpiler Updates (1 day)

### 6.1 C Transpiler

| ParamMode | C Type |
|-----------|--------|
| View | `const T*` |
| Mut | `T* const` |
| Move | `T` |

| Accessor | C Code |
|----------|--------|
| `.view` | `&` |
| `.mut` | `&` |
| `.move` | (pass by value) |
| `.clone()` | `clone_T()` |

### 6.2 Rust Transpiler

| ParamMode | Rust Type |
|-----------|-----------|
| View | `&T` |
| Mut | `&mut T` |
| Move | `T` |

| Accessor | Rust Code |
|----------|-----------|
| `.view` | `&` |
| `.mut` | `&mut` |
| `.move` | (move by default) |
| `.clone()` | `.clone()` |

---

## Phase 7: Test Updates (1 day)

### 7.1 Rename and Update Tests

- `025_borrow_take/` → `025_borrow_move/`
- Update all `.take` to `.move`

### 7.2 New Tests

**`test/a2r/200_value_access_move.at`**
```auto
fn consume(s move str) { print(s) }

fn main() {
    let data = "hello"
    consume(data.move)
    // print(data)  // ERROR: Use after move
}
```

**`test/a2r/201_value_access_clone.at`**
```auto
fn process(s move str) { print(s) }

fn main() {
    let data = "hello"
    process(data.clone())  // data still valid
    print(data)  // OK
}
```

---

## Phase 8: Stdlib Updates (0.5 days)

Update parameter declarations in `stdlib/auto/*.at`:

```auto
// BEFORE
fn foo(data copy List<int>) { ... }
process(my_list)  // Silent O(N) copy!

// AFTER
fn foo(data move List<int>) { ... }
process(my_list.clone())  // Explicit expensive operation!
```

---

## Estimated Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| 1 | 1 day | AST Updates |
| 2 | 0.5 days | Lexer Updates |
| 3 | 1 day | Parser Updates |
| 4 | 2 days | Codegen Smart Parameters |
| 5 | 1.5 days | Move Semantics |
| 6 | 1 day | Transpiler Updates |
| 7 | 1 day | Test Updates |
| 8 | 0.5 days | Stdlib Updates |
| **Total** | **8.5 days** | |

---

## Success Criteria

- [ ] `take` deprecated, `move` works everywhere
- [ ] `copy` removed from parameter mode
- [ ] `.view`, `.mut`, `.move` accessors work at call site
- [ ] `.clone()` method syntax works (with parentheses)
- [ ] Moved variables properly invalidated
- [ ] Default parameter mode is View
- [ ] All transpilers generate correct code
- [ ] All tests pass

---

## Migration Guide

| Old Syntax | New Syntax | Reason |
|------------|------------|--------|
| `fn foo(x take T)` | `fn foo(x move T)` | Avoid confusion with `.take(n)` |
| `foo(x.take)` | `foo(x.move)` | Consistency |
| `fn foo(x copy T)` | `fn foo(x move T)` | Remove implicit O(N) |
| `foo(x)` with copy | `foo(x.clone())` | Explicit warning |

---

## Dependencies

- Plan 088 (Param Passing) - Phase 4 codegen completion required

## Blocks

- Plan 119 (Backend Stdlib) - Should wait for this refactor
