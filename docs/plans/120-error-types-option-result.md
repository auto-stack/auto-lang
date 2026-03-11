# Plan 120: AutoLang Error Types - ?T (Option) and !T (Result)

## Status: 📋 PLANNING

## Objective

Implement a clean separation between optional values (`?T` = Option<T>) and error-propagating values (`!T` = Result<T, E>), replacing the conflated May<T> system.

## Background

### Current State (Problematic)

AutoLang currently uses `May<T>` (or `?T`) to represent both:
- **Optional values**: "might not exist" (None)
- **Error results**: "might have failed" (Error)

This conflation causes confusion:
```auto
// Current: ?T means both "maybe absent" AND "maybe error"
let user = db.get_user(1)?  // Is None "not found" or "query error"?

// Unclear semantics:
fn find_user(id int) ?User  // Returns None if... not found? Or error?
fn parse_int(s str) ?int    // Returns None if... invalid? Or too big?
```

### Proposed State (Clear Separation)

```auto
// ?T = Option<T> - "value might not exist" (None is valid state)
// !T = Result<T, E> - "operation might fail" (Err is exceptional)

// Option: Absence is expected
fn find_user(id int) ?User     // None = "user doesn't exist" (not an error)
fn get_first(items []int) ?int // None = "list is empty" (not an error)

// Result: Failure is exceptional
fn parse_int(s str) !int       // Err = "invalid format" (error condition)
fn connect_db(url str) !Db     // Err = "connection failed" (error condition)
```

## Type System Design

### 1. Option Type (`?T`)

**AutoLang Syntax**:
```auto
type ?T = T | None

// Construction
let some: ?int = 42        // Has value
let none: ?int = None      // No value

// Pattern matching
match maybe_value {
    Some(x) => print(f"Got ${x}"),
    None => print("No value")
}

// Propagation (后缀属性关键字风格，与其他访问符一致)
let inner = opt.?

// Unwrap with default (后缀方法风格)
let value = opt.?(0)
```

**Rust Transpilation**:
```rust
type Option<T> = Option<T>;

let some: Option<i32> = Some(42);
let none: Option<i32> = None;

match maybe_value {
    Some(x) => println!("Got {}", x),
    None => println!("No value"),
}

let inner = opt?;                   // opt.?
let value = opt.unwrap_or(0);       // opt.?(0)
```

### 2. Result Type (`!T`)

**AutoLang Syntax**:
```auto
type !T = T | Err

// Construction
let ok: !int = 42            // Success
let err: !int = Err("failed") // Failure

// Pattern matching
match result {
    Ok(x) => print(f"Success: ${x}"),
    Err(e) => print(f"Error: ${e}")
}

// Propagation (后缀属性关键字风格)
let inner = res.?

// Unwrap with default (后缀方法风格)
let value = res.?(0)

// Convert Result to Option
let opt: ?int = res.ok()
```

**Rust Transpilation**:
```rust
type Result<T> = Result<T, String>;  // Default error type is String

let ok: Result<i32> = Ok(42);
let err: Result<i32> = Err("failed".to_string());

match result {
    Ok(x) => println!("Success: {}", x),
    Err(e) => println!("Error: {}", e),
}

let inner = res?;                    // res.?
let value = res.unwrap_or(0);        // res.?(0)
let opt: Option<i32> = res.ok();     // res.ok()
```

### 3. 与值访问体系的一致性

`.?` 和 `.?(dft)` 遵循 AutoLang 的**后缀访问符**设计哲学，与以下语法一致：

| 访问符 | 语义 | 开销 |
|--------|------|------|
| `.view` | 只读借用 | O(1) |
| `.mut` | 可变借用 | O(1) |
| `.move` | 所有权转移 | O(1) |
| `.clone()` | 深拷贝 | O(N) |
| `.?` | 错误传播 | O(1) |
| `.?(dft)` | 带默认值解包 | O(1) |

这种一致性使得 IDE 自动补全和代码阅读都更加自然（"先定宾语，再定动作"）。

## API Examples

### HTTP with Error Types

```auto
// HTTP client returns Result (network operations can fail)
fn http.get(url str) !Response {
    // Err if: network error, timeout, DNS failure
    // Ok(Response) on success
}

// Usage
fn fetch_data() !str {
    let res = http.get("https://api.example.com")?  // Propagate error
    let body = res.body()
    Ok(body)
}

// At top level
fn main() {
    match fetch_data() {
        Ok(data) => print(data),
        Err(e) => print(f"Failed: ${e}")
    }
}
```

### Database with Option and Result

```auto
// Database lookup returns Option (not found is expected)
fn db.find_user(id int) ?User {
    // None = user doesn't exist (not an error)
    // Some(User) = found
}

// Connection returns Result (connection can fail)
fn db.connect(url str) !Connection {
    // Err = connection failed
    // Ok(Connection) = connected
}

// Usage
fn get_user_email(id int) !str {
    let conn = db.connect("sqlite://app.db")?  // Propagate connection error
    
    let user = conn.find_user(id)  // Option<User>
    if user == None {
        return Err("User not found")  // Convert to Result error
    }
    
    Ok(user.email)
}
```

### Redis with Both Types

```auto
impl RedisClient {
    // GET returns Option (key might not exist)
    fn get(self, key str) ?str
    
    // SET returns Result (operation might fail)
    fn set(self, key str, val str) !void
    
    // INCR returns Result (value might not be numeric)
    fn incr(self, key str) !i64
}

// Usage
fn increment_counter(client RedisClient, key str) !i64 {
    // Initialize if doesn't exist
    if client.get(key) == None {
        client.set(key, "0")?
    }
    
    client.incr(key)?
}
```

## Implementation Plan

### Phase 1: Type System Core (2-3 days)

**Files to modify**:
- `crates/auto-lang/src/ast.rs` - Add Option and Result type variants
- `crates/auto-lang/src/parser.rs` - Parse `?T` and `!T` syntax
- `crates/auto-lang/src/lexer.rs` - Tokenize `?` and `!` in type positions

**Tasks**:
- [ ] Add `Type::Option(Box<Type>)` variant
- [ ] Add `Type::Result(Box<Type>)` variant  
- [ ] Parse `?T` as `Type::Option(T)`
- [ ] Parse `!T` as `Type::Result(T)`
- [ ] Update type pretty-printing

### Phase 2: VM Support (2 days)

**Files to modify**:
- `crates/auto-val/src/value.rs` - Add Option/Result value types
- `crates/auto-lang/src/eval.rs` - Evaluate Option/Result expressions

**Tasks**:
- [ ] Add `Value::Option(Option<Box<Value>>)` variant
- [ ] Add `Value::Result(Result<Box<Value>, String>)` variant
- [ ] Implement `None` literal
- [ ] Implement `Some(x)` and `Ok(x)` constructors
- [ ] Implement `Err(msg)` constructor
- [ ] Implement `?` operator (propagation)

### Phase 3: Pattern Matching (1-2 days)

**Tasks**:
- [ ] Match on Option: `Some(x) => ..., None => ...`
- [ ] Match on Result: `Ok(x) => ..., Err(e) => ...`
- [ ] Exhaustiveness checking

### Phase 4: Rust Transpilation (2 days)

**Files to modify**:
- `crates/auto-lang/src/trans/rust.rs`

**Tasks**:
- [ ] Transpile `?T` to `Option<T>`
- [ ] Transpile `!T` to `Result<T, String>`
- [ ] Transpile `None` to `None`
- [ ] Transpile `Some(x)` to `Some(x)`
- [ ] Transpile `Ok(x)` to `Ok(x)`
- [ ] Transpile `Err(e)` to `Err(e.into())`
- [ ] Transpile `val?` to `val?`
- [ ] Transpile `val!` to `val.unwrap()`
- [ ] Transpile `val ?? default` to `val.unwrap_or(default)`

### Phase 5: Migration (2-3 days)

**Tasks**:
- [ ] Deprecate `May<T>` type
- [ ] Update stdlib to use `?T` and `!T`
- [ ] Update documentation
- [ ] Add migration guide

### Phase 6: Testing (1-2 days)

**Test cases**:
- [ ] Option construction and matching
- [ ] Result construction and matching
- [ ] Propagation operator (`?`)
- [ ] Unwrap operators (`!`, `??`)
- [ ] Conversions between types
- [ ] a2rs transpilation tests

## Estimated Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| 1 | 2-3 days | Type system core |
| 2 | 2 days | VM support |
| 3 | 1-2 days | Pattern matching |
| 4 | 2 days | Rust transpilation |
| 5 | 2-3 days | Migration |
| 6 | 1-2 days | Testing |
| **Total** | **10-14 days** | |

## Dependencies

- None (this is a foundational change)

## Blocks

- Plan 119: a2rs Backend Stdlib (requires proper error types)

## Success Criteria

- [ ] `?T` compiles and runs correctly for optional values
- [ ] `!T` compiles and runs correctly for error results
- [ ] `?` operator propagates errors correctly
- [ ] Pattern matching works for both types
- [ ] a2rs transpilation generates valid Rust code
- [ ] All existing tests still pass
- [ ] Stdlib migrated to use new types

## Open Questions

1. **Error Type**: Should `!T` use `String` as default error type, or a custom `Error` struct?
2. **Generic Errors**: Should we support `!T<E>` for custom error types?
3. **May Migration**: Keep backward compatibility or breaking change?
