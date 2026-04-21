# Plan 072: Logical Operators `and` and `or` — **已废止 (DEPRECATED)**

> **⚠️ 此计划已废止。** 逻辑运算符已回退为 `&&` 和 `||` 符号形式。
>
> **废止理由：**
> 1. `and`/`or` 作为关键字会与位运算方法名冲突（`.and()`/`.or()` 无法使用）
> 2. 释放 `and`/`or` 给位运算方法后，设计更一致（`val.and(mask)`、`val.or(mask)`）
> 3. 实际代码中使用 `and`/`or` 逻辑运算的仅约 4 处，迁移成本极低
> 4. `&&`/`||` 与 C/Rust/JS/TS 等主流语言一致，降低学习成本
> 5. 底层 OpCode 和 transpiler 输出本来就是 `AND`/`OR` 和 `&&`/`||`，回退只是统一源码层
>
> **回退实现**: 见 Git 提交历史（Plan 178 附带的关键字回退）

---

## Objective (原始内容，仅供参考)

Implement logical operators `and` and `or` as keywords in AutoLang, with short-circuit evaluation semantics.

## Current State

- ✅ `!` (not) - already implemented as `Op::Not`
- ❌ `&&` (logical and) - **NOT implemented**
- ❌ `||` (logical or) - **NOT implemented**

## Design Decision

After discussion, we've decided to use **keywords** instead of symbols:

| Symbol | Keyword | Semantics |
|--------|---------|-----------|
| (planned) `&&` | `and` | Logical AND (short-circuit) |
| (planned) `\|\|` | `or` | Logical OR (short-circuit) |
| `!` | `!` (for now) | Logical NOT (unary) |

**Rationale**:
- Keywords are more readable: `if x and y or z` vs `if x && y || z`
- Keywords are beginner-friendly
- Avoid confusion with bitwise operators `&` and `|`
- Similar to Python, Ruby, and other languages

## Syntax Examples

```auto
// Basic usage
let flag = true and false  // false
let result = true or false  // true

// With expressions
let x = (a > 0) and (b < 10)

// Short-circuit evaluation
let result = (x != 0) and (10 / x > 1)  // if x==0, division is skipped
let value = option or default  // if option is truthy, default not evaluated

// Complex expressions
if (enabled and valid) or skip {
    // ...
}

// Negation
let not_flag = !flag  // existing Op::Not
```

## Operator Precedence

From lowest to highest:

```
or          (lowest, 10)
and         (medium, 20)
== != < > <= >=  (comparisons, 30)
+ -         (additive, 40)
* / %       (multiplicative, 50)
!           (unary, 60)
```

**Example**:
```auto
// This is parsed as: (a and b) or c
a and b or c

// This is parsed as: a and (b or c)  [needs parentheses]
a and (b or c)

// Comparisons have higher precedence:
x > 0 and y < 10  // parsed as: (x > 0) and (y < 10)
```

## Short-Circuit Semantics

### `and` operator

```auto
left and right
```

- If `left` is **falsy**, return `left` immediately (don't evaluate `right`)
- If `left` is **truthy**, evaluate and return `right`

**Examples**:
```auto
false and foo()      // returns false, foo() is NOT called
true and foo()       // calls foo() and returns its result
0 and 1/x           // returns 0, no division by zero error
```

### `or` operator

```auto
left or right
```

- If `left` is **truthy**, return `left` immediately (don't evaluate `right`)
- If `left` is **falsy**, evaluate and return `right`

**Examples**:
```auto
true or foo()        // returns true, foo() is NOT called
false or foo()       // calls foo() and returns its result
x or default_value   // if x is truthy, default_value not evaluated
```

### Truthy/Falsy Values

Following AutoLang's existing semantics (same as `if` statement):

- **Falsy**: `nil`, `false`, `0`, `0.0`, `""`, empty arrays/objects
- **Truthy**: everything else (non-zero numbers, non-empty strings, `true`, objects, arrays)

## Implementation Plan

### Phase 1: Token and Lexer Changes

**File**: `crates/auto-lang/src/token.rs`

Add to `TokenKind` enum:
```rust
// Keywords section
And,   // NEW: logical and keyword
Or,    // NEW: logical or keyword
```

Update `Token::keyword_kind()`:
```rust
"and" => Some(TokenKind::And),
"or" => Some(TokenKind::Or),
```

Update `impl fmt::Display for Token`:
```rust
TokenKind::And => write!(f, "<and>"),
TokenKind::Or => write!(f, "<or>"),
```

**File**: `crates/auto-lang/src/lexer.rs`

No changes needed - `identifier_or_special_block()` already handles keywords via `Token::keyword_kind()`.

### Phase 2: AST Changes

**File**: `crates/auto-val/src/value.rs`

Add to `Op` enum:
```rust
pub enum Op {
    // ... existing operators ...
    Not,
    And,    // NEW: logical and
    Or,     // NEW: logical or
    // ... rest of operators ...
}
```

### Phase 3: Parser Changes

**File**: `crates/auto-lang/src/parser.rs`

1. **Add precedence constants** (around line 80):
```rust
const PREC_AND: u8 = 20;  // NEW: logical and
const PREC_OR: u8 = 10;   // NEW: logical or
```

2. **Update `infix_power()` function** (around line 112):
```rust
fn infix_power(op: Op, span: SourceSpan) -> AutoResult<InfixPrec> {
    match op {
        // ... existing cases ...
        Op::Or => Ok(PREC_OR),         // NEW: logical or
        Op::And => Ok(PREC_AND),       // NEW: logical and
        // ... rest of cases ...
    }
}
```

3. **Update `op()` method** to recognize `and`/`or` tokens:
```rust
TokenKind::And | TokenKind::Or => self.op(),
```

4. **Update token-to-Op conversion** (where tokens are converted to Op):
```rust
TokenKind::And => Op::And,
TokenKind::Or => Op::Or,
```

### Phase 4: Evaluator Changes

**File**: `crates/auto-lang/src/eval.rs`

Add to `eval_bina()` method:

```rust
fn eval_bina(&mut self, left: &Expr, op: &Op, right: &Expr) -> Value {
    match op {
        // ... existing cases ...

        // Logical AND with short-circuit evaluation
        Op::And => {
            let left_value = self.eval_expr(left);
            if Self::is_truthy(&left_value) {
                // Left is truthy, evaluate and return right
                self.eval_expr(right)
            } else {
                // Left is falsy, short-circuit: don't evaluate right
                left_value
            }
        }

        // Logical OR with short-circuit evaluation
        Op::Or => {
            let left_value = self.eval_expr(left);
            if Self::is_truthy(&left_value) {
                // Left is truthy, short-circuit: don't evaluate right
                left_value
            } else {
                // Left is falsy, evaluate and return right
                self.eval_expr(right)
            }
        }

        // ... rest of cases ...
    }
}
```

**Helper method** (if not already exists):
```rust
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Uint(n) => *n != 0,
        Value::Float(n) => *n != 0.0,
        Value::Str(s) => !s.is_empty(),
        Value::Nil => false,
        Value::Void => false,
        _ => true,  // Objects, arrays, functions are truthy
    }
}
```

### Phase 5: Transpiler Changes

**File**: `crates/auto-lang/src/trans/c.rs` and `rust.rs`

Add cases for transpiling `Op::And` and `Op::Or`:

```rust
Op::And => write!(out, " && ")?,  // C: transpile to &&
Op::Or => write!(out, " || ")?,   // C: transpile to ||
```

For Rust transpiler:
```rust
Op::And => write!(out, " && ")?,  // Rust also uses &&
Op::Or => write!(out, " || ")?,
```

**Note**: Even though AutoLang uses `and`/`or` keywords, we transpile to `&&`/`||` in C/Rust because those languages don't have `and`/`or` keywords.

### Phase 6: Tests

**File**: `crates/auto-lang/tests/parser_tests.md` or new test file

Add comprehensive test cases:

```rust
#[test]
fn test_and_basic() {
    let code = "true and false";
    let result = eval(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_or_basic() {
    let code = "true or false";
    let result = eval(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_and_short_circuit() {
    let code = "false and panic('should not execute')";
    // Should not panic
    let result = eval(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_or_short_circuit() {
    let code = "true or panic('should not execute')";
    // Should not panic
    let result = eval(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_and_or_precedence() {
    let code = "true and false or true";
    // Should parse as: (true and false) or true
    let result = eval(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_and_with_comparisons() {
    let code = "x > 0 and y < 10";
    // Setup x and y in scope
    // Test...
}

#[test]
fn test_truthy_values() {
    let tests = vec![
        ("0 and 1", false),
        ("1 and 2", true),
        ("\"\" or \"hello\"", "hello"),
        ("nil or 42", 42),
    ];
    // Test each case...
}
```

## Success Criteria

- [ ] `and` keyword recognized by lexer
- [ ] `or` keyword recognized by lexer
- [ ] Parser correctly parses `and`/`or` with proper precedence
- [ ] Evaluator implements short-circuit evaluation
- [ ] Transpilers generate correct C/Rust code (`&&`/`||`)
- [ ] All tests pass
- [ ] Documentation updated

## Open Questions

1. **`!` vs `not`**: Should we change `!` to `not` keyword?
   - **Status**: Deferred for future discussion
   - **Current**: Keep `!` for now

2. **Bitwise operators**: Should we also add `&`, `|`, `^`, `~` for bitwise operations?
   - **Status**: Out of scope for this plan
   - **Future**: Can be added separately

## Related Plans

- None directly related, but builds on existing operator infrastructure

## References

- Python's `and`, `or`, `not` operators
- Swift's `&&`, `||`, `!` operators (what we're NOT doing)
- Rust's `&&`, `||`, `!` operators (target transpilation)
