# Plan 241: a2r String Type Cleanup

## Background

Auto string type system has been refactored (Plan 240):
- `str` (StrSlice) → Rust `&str` — borrowed string slice
- `Str` (StrOwned) → Rust `String` — owned dynamic string
- `str(N)` (StrFixed) → fixed-size buffer
- `cstr` (CStrLit) → C string literal

After auditing all 38 examples, the following issues were identified.
Examples 01-33 compile and run (Cargo.toml points to hand-written `main.rs`).
Examples 34-38 use a2r-generated `main.a2r.rs` (Cargo.toml points to it).

## Priority 1: a2r Transpiler Bugs (causes compilation failure)

### P1-1: `get_or` generates `.as_str()` on non-string map values
**File**: `trans/rust.rs` (~line 2233)
**Symptom**: `Map<str, int>.get_or("key", 0)` generates `.map(|s| s.as_str()).unwrap_or(0)` — `as_str()` doesn't exist on `i32`
**Affected**: Example 34 (`Map<str, int>`)
**Fix**: `get_or` should only generate `.map(|s| s.as_str())` for maps whose values are string types. For non-string maps, use `.cloned().unwrap_or(default)`.

### P1-2: `Add` operator generates `format!` for variable + variable
**File**: `trans/rust.rs` (~line 637, `Op::Add` handler)
**Symptom**: `total + v` where both are `Expr::Ident` (variables) generates `format!("{}{}", total, v)` instead of `total + v`
**Affected**: Example 34 (`total = total + v` in for loop)
**Fix**: Extend `is_numeric_expr()` to check variable names against `current_fn_str_params` — if a variable is NOT a str param, it's likely numeric. Or add a numeric-variables tracking set (initialized from `var x = 0` patterns).

## Priority 2: Auto Source Code — str vs Str Misuse

### P2-1: Struct fields owning string data should use `Str` not `str`
**Principle**: If a struct owns the string (heap-allocated, not borrowing), use `Str`.
**Affected examples** (struct fields that store created/assigned string values):
- 09: `kind str`, `text str`, `tool_use_id str`, `tool_name str`, `role str`, `content_text str`
- 12: `kind str`, `text str`, `partial_json str`, `thinking str`
- 15: `event_type str`, `data str`
- 16: `event_type str`, `data str`
- 21: `model str`, `max_tokens str`, `system str`, `stream str`, `temperature str`
- 23: `id str`, `model str`, `stop_reason str`
- 25: `kind str`, `stop_reason str`
- 27: `role str`, `content str`
- 29: `api_key str`, `base_url str`
- 32: `id str`, `name str`, `input_json str`
- 33: `provider str`, `model str`, `api_key str`
- 35: `api_key str`, `base_url str`, `model str`
- 37: `body str`
- 38: `body str`

**Decision needed**: Should ALL struct string fields change to `Str`?
In Rust, struct fields are typically `String` (owned). But many of these examples
use string literals assigned to fields — which are `&'static str`. The current a2r
generates `String` for all `str` fields anyway (in `rust_type_name`). So changing
the source `.at` from `str` to `Str` is mainly for semantic correctness in Auto.

**Recommendation**: Change to `Str` in .at files, keep a2r mapping as-is.

### P2-2: Map value types should use `Str` for owned strings
**Affected examples**:
- 35: `Map<str, str>` → `Map<Str, Str>` (headers)
- 36: `Map<str, str>` → `Map<Str, Str>` (config values)
- 38: All `Map<str, str>` → `Map<Str, Str>` (headers, body fields)

### P2-3: Function return types that create new strings should be `Str`
**Affected examples**:
- 11: `serialize_tool_result_content() str` → `Str` (builds string via concat)
- 38: `build_json_body() str` → `Str` (returns format! result)

## Priority 3: a2r Transpiler Improvements (not blocking compilation)

### P3-1: `insert` adds `.to_string()` to all non-int/bool values
**File**: `trans/rust.rs` (~line 2392)
**Current**: `if is_insert && !matches!(expr, Expr::Int(_) | Expr::Bool(_))` adds `.to_string()`
**Issue**: For `Map<str, int>`, string key still gets `.to_string()` which is correct, but the heuristic is fragile
**Improvement**: Track Map value type and only add `.to_string()` when needed

### P3-2: `not` operator only works in `assert()` context
**File**: `trans/rust.rs` `call()` method
**Current**: `not` is handled as a function call `not(expr)` → `!(expr)`, only works in assert
**Issue**: `if not condition { }` doesn't parse in Auto
**Workaround**: Examples 34, 37 use `assert(not expr)` which works; but `if not` doesn't
**Affected**: 34, 37
**Fix**: Either add `not` support in `if` conditions at parser level, or document the limitation

### P3-3: Auto `return` must have expression on same line
**Issue**: `return` on its own line (early return pattern) causes parser error
**Workaround**: Use nested if/else instead of early returns
**Fix**: Parser improvement to allow `return` followed by newline + expression

### P3-4: Cargo.toml for examples 01-33 still points to `main.rs`
**Current**: Examples 01-33 have `path = "src/XX_name/main.rs"` in Cargo.toml
**Issue**: These compile from hand-written Rust, not a2r-generated code
**Fix**: After all a2r issues are resolved, switch Cargo.toml to point to `main.a2r.rs`

### P3-5: `trans_rust_with_session` writes output file AND returns log string
**File**: `lib.rs` (~line 1840)
**Issue**: When CLI uses `-o` flag, the function writes Rust code first, then CLI overwrites with log string
**Workaround**: Don't use `-o` flag (let function write default path)
**Fix**: Return the source bytes instead of log string, let CLI handle file writing

## Priority 4: Warnings to Clean Up

### P4-1: Unused variables in generated code
- Example 38: `prompt` parameter unused in `send_message()`
- Could add underscore prefix in a2r for unused params

### P4-2: Unused imports in hand-written main.rs files
- Example 33: unused `PathBuf` import
- Example 26: unused `json` import

## Execution Order

1. **Fix P1-1** (get_or for non-string maps) — unblocks example 34 compilation
2. **Fix P1-2** (Add operator variable detection) — unblocks example 34 compilation
3. **Fix P2-1/P2-2/P2-3** in `.at` source files — semantic correctness
4. **Fix P3-5** (CLI double-write bug)
5. **Re-transpile all 38 examples** and verify compilation
6. **Switch Cargo.toml** for examples 01-33 to use `main.a2r.rs`
7. **Clean up warnings** (P4)
