# JavaScript Transpiler (a2j) Implementation Plan

## Executive Summary

Implement a JavaScript transpiler for AutoLang that converts AutoLang source code to modern JavaScript (ES6+). This will enable AutoLang programs to run in web browsers, Node.js, and other JavaScript environments, greatly expanding the language's reach and accessibility.

**Timeline**: 4-6 weeks
**Complexity**: Low-Medium (JavaScript is flexible, no type system concerns)
**Priority Features**:
- ES6+ modern syntax (classes, template literals, arrow functions)
- Perfect F-string mapping (template literals are identical!)
- Dynamic typing (JavaScript's natural state)
- ES6 classes for structs and methods
- Switch statements for pattern matching

---

## 1. File Structure

### 1.1 Files to Create

```
crates/auto-lang/src/trans/javascript.rs       # Main transpiler (~800 lines)
crates/auto-lang/test/a2j/                     # Test directory
  ├── 000_hello/
  │   ├── hello.at
  │   └── hello.expected.js
  ├── 001_print/
  ├── 002_array/
  ├── 003_func/
  ├── 006_struct/
  ├── 007_enum/
  ├── 010_if/
  ├── 011_for/
  ├── 012_is/
  └── 015_str/
```

### 1.2 Files to Modify

```
crates/auto-lang/src/trans/mod.rs              # Add javascript module export
crates/auto-lang/src/lib.rs                    # Add trans_javascript() function
crates/auto/src/main.rs                        # Add JavaScript CLI command
```

---

## 2. Type Mapping Strategy

### 2.1 AutoLang → JavaScript Type Mapping

| AutoLang Type | JavaScript Type | Notes |
|---------------|-----------------|-------|
| `Byte` | `Number` | JavaScript uses Number for all numeric types |
| `Int` | `Number` | Direct mapping |
| `Uint` | `Number` | Direct mapping |
| `Float` | `Number` | Direct mapping |
| `Double` | `Number` | Direct mapping |
| `Bool` | `Boolean` | Direct mapping |
| `Char` | `String` | Use 1-char string |
| `Str(n)` | `String` | Direct mapping |
| `Array` | `Array` | Direct mapping |
| `User` (struct) | `class` | ES6 class with constructor |
| `Enum` | `Object.freeze()` | Frozen object with string keys |
| `Void` | `undefined` | Functions without return |

### 2.2 Variable Declaration Strategy

**Simple Rule**:
- `let` (AutoLang immutable) → `const` (JavaScript)
- `mut` (AutoLang mutable) → `let` (JavaScript)
- `var` (AutoLang dynamic) → `let` (JavaScript)

Rationale: Use `const` by default for safety, `let` for reassignment.

---

## 3. Key Implementation Challenges and Solutions

### 3.1 No Significant Whitespace

**Good News**: JavaScript uses braces like AutoLang! No indentation tracking needed.

**Implementation**: Simpler than Python transpiler - just copy AutoLang's brace structure.

### 3.2 F-strings → Template Literals (Perfect Match!) 🎯

**Problem**: None - AutoLang and JavaScript have nearly identical syntax!

**Solution**: Simple character replacement:
```javascript
// AutoLang: f"hello $name"
// JS:        `hello ${name}`

// AutoLang: f"result: ${1+2}"
// JS:        `result: ${1+2}`
```

Only difference: backticks instead of double quotes.

### 3.3 Pattern Matching

**Problem**: AutoLang uses `is`, JavaScript uses `switch`

**Solution**:
```javascript
// AutoLang
is x {
    0 => print("zero")
    1 => print("one")
    else => print("other")
}

// JavaScript
switch (x) {
    case 0:
        console.log("zero");
        break;
    case 1:
        console.log("one");
        break;
    default:
        console.log("other");
}
```

### 3.4 Range Loops

**Problem**: AutoLang `for i in 0..10` → JavaScript needs helper

**Solution**: Generate range function:
```javascript
// AutoLang: for i in 0..10
// JS:
for (let i = 0; i < 10; i++) {
    // body
}

// Or use range helper:
for (const i of range(0, 10)) {
    // body
}
```

### 3.5 Struct Definitions

**Problem**: AutoLang `type Point { x int }` → JavaScript class

**Solution**: ES6 class with constructor:
```javascript
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    modulus() {
        return this.x * this.x + this.y * this.y;
    }
}
```

---

## 4. Transpiler Architecture

### 4.1 Main Structure

```rust
// crates/auto-lang/src/trans/javascript.rs

pub struct JavaScriptTrans {
    scope: Shared<Universe>,
    // No indent needed! JavaScript uses braces
}

impl Trans for JavaScriptTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Generate statements
        // Generate main function
        // Export if needed
    }
}
```

### 4.2 Core Methods

- `stmt()` - Handle all statement types
- `expr()` - Handle all expression types
- `if_stmt()` - if/else with braces
- `for_loop()` - Transform ranges
- `is_stmt()` - `is` → `switch`
- `fn_decl()` - Function declarations
- `type_decl()` - `type` → `class`
- `enum_decl()` - `enum` → frozen object

---

## 5. Implementation Phases

### Phase 1: Core Infrastructure (Week 1)

**Objective**: Basic transpiler structure and simple expressions

**Tasks**:
1. Create `crates/auto-lang/src/trans/javascript.rs`
2. Implement `JavaScriptTrans` struct
3. Implement `Trans` trait
4. Add module export in `trans/mod.rs`
5. Implement literal expressions (Int, Float, Bool, Str)
6. Implement identifier expressions
7. Implement binary operators (arithmetic)
8. Create test: `000_hello`

**Deliverables**:
- Basic JavaScriptTrans structure
- `test_000_hello` passing

**Success Criteria**:
```javascript
// Input (hello.at)
fn main() {
    print("hello, world!")
}

// Output (hello.expected.js)
function main() {
    console.log("hello, world!");
}

main();
```

### Phase 2: Control Flow (Week 1-2)

**Objective**: if statements and for loops

**Tasks**:
1. Implement `if` statement (if/else if/else)
2. Implement `for` loop with ranges
3. Implement `break` statement
4. Implement comparison operators
5. Implement logical operators
6. Create tests: `010_if`, `011_for`

**Success Criteria**:
```javascript
// Input
fn main() {
    let x = 2
    if x > 1 {
        print("Great!")
    }

    for i in 0..5 {
        print(i)
    }
}

// Output
function main() {
    const x = 2;
    if (x > 1) {
        console.log("Great!");
    }

    for (let i = 0; i < 5; i++) {
        console.log(i);
    }
}
```

### Phase 3: Functions (Week 2)

**Objective**: Function definitions and calls

**Tasks**:
1. Implement function declarations (use `function` keyword)
2. Implement function calls
3. Implement parameter handling
4. Implement return statements
5. Create tests: `003_func`, `001_print`

**Success Criteria**:
```javascript
// Input
fn add(x int, y int) int {
    return x + y
}

fn main() {
    print(add(1, 2))
}

// Output
function add(x, y) {
    return x + y;
}

function main() {
    console.log(add(1, 2));
}

main();
```

### Phase 4: Pattern Matching (Week 2-3)

**Objective**: `is` statement → `switch`

**Tasks**:
1. Implement `is` → `switch/case`
2. Handle literal patterns
3. Handle wildcard patterns (default)
4. Create test: `012_is`

**Success Criteria**:
```javascript
// Input
fn main() {
    let x = 10
    is x {
        0 => print("ZERO")
        1 => print("ONE")
        else => print("Large")
    }
}

// Output
function main() {
    const x = 10;
    switch (x) {
        case 0:
            console.log("ZERO");
            break;
        case 1:
            console.log("ONE");
            break;
        default:
            console.log("Large");
    }
}
```

### Phase 5: Arrays (Week 3)

**Objective**: Array literals and indexing

**Tasks**:
1. Implement array literals
2. Implement array indexing
3. Create test: `002_array`

**Success Criteria**:
```javascript
// Input
fn main() {
    let array = [1, 2, 3, 4, 5]
    print(array[1])
    let a = array[0]
    print(a)
}

// Output
function main() {
    const array = [1, 2, 3, 4, 5];
    console.log(array[1]);
    const a = array[0];
    console.log(a);
}
```

### Phase 6: F-strings (Week 3)

**Objective**: Template literal support

**Tasks**:
1. Implement string literals
2. Implement template literals (backtick conversion)
3. Convert `$var` → `${var}`
4. Create tests: `015_str`

**Success Criteria**:
```javascript
// Input
fn main() {
    let name = "World"
    print(f"Hello, $name!")
    print(f"Result: ${1 + 2}")
}

// Output
function main() {
    const name = "World";
    console.log(`Hello, ${name}!`);
    console.log(`Result: ${1 + 2}`);
}
```

### Phase 7: Structs (Week 4)

**Objective**: Type definitions → ES6 classes

**Tasks**:
1. Implement `type` → `class`
2. Implement constructor generation
3. Implement field declarations
4. Create test: `006_struct`
5. Implement method calls
6. Implement field access

**Success Criteria**:
```javascript
// Input
type Point {
    x int
    y int
}

fn main() {
    let p = Point{x: 0, y: 0}
    print(p.x)
}

// Output
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
}

function main() {
    const p = new Point(0, 0);
    console.log(p.x);
}
```

### Phase 8: Methods (Week 4)

**Objective**: Methods in classes

**Tasks**:
1. Implement method declarations in classes
2. Handle `self` → `this`
3. Create test: `008_method`

**Success Criteria**:
```javascript
// Input
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}

// Output
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    modulus() {
        return this.x * this.x + this.y * this.y;
    }
}
```

### Phase 9: Enums (Week 5)

**Objective**: Enum support

**Tasks**:
1. Implement `enum` → frozen object
2. Create test: `007_enum`

**Success Criteria**:
```javascript
// Input
enum Color {
    Red
    Green
    Blue
}

fn main() {
    let c = Color.Red
    print(c)
}

// Output
const Color = Object.freeze({
    Red: "Red",
    Green: "Green",
    Blue: "Blue"
});

function main() {
    const c = Color.Red;
    console.log(c);
}
```

### Phase 10: CLI Integration (Week 5)

**Objective**: Add JavaScript command to CLI

**Tasks**:
1. Add `trans_javascript()` to `lib.rs`
2. Add `JavaScript` command to `main.rs`
3. Implement file output (`.js` files)
4. Update documentation

**Success Criteria**:
```bash
$ auto.exe javascript hello.at
[trans] hello.at -> hello.js

$ node hello.js
hello, world!
```

### Phase 11: Advanced Features (Week 5-6)

**Objective**: Complete remaining features

**Tasks**:
1. Unary operators
2. Arrow functions for simple lambdas
3. Block expressions
4. If expressions (ternary)
5. Edge cases

**Success Criteria**:
- 10+ test cases passing
- All core features working
- Clean ES6+ output

---

## 6. Test Case Prioritization

### Priority 1 - Must Have (000-020)
1. `000_hello` - Basic print
2. `001_print` - Multiple prints
3. `002_array` - Arrays
4. `003_func` - Functions
5. `010_if` - If/else
6. `011_for` - For loops
7. `012_is` - Pattern matching
8. `015_str` - Template literals
9. `006_struct` - Struct definitions
10. `007_enum` - Enum definitions
11. `008_method` - Methods

### Priority 2 - Should Have (030-050)
12. More complex examples
13. Nested structures
14. Advanced pattern matching

---

## 7. Integration Points

### 7.1 Module Integration

**File**: `crates/auto-lang/src/trans/mod.rs`
```rust
pub mod javascript;
pub use javascript::JavaScriptTrans;
```

### 7.2 Library API Integration

**File**: `crates/auto-lang/src/lib.rs`
```rust
pub fn trans_javascript(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;
    let jsname = path.replace(".at", ".js");

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    let ast = parser.parse()?;

    let mut sink = Sink::new(fname);
    let mut trans = JavaScriptTrans::new(fname);
    trans.set_scope(parser.scope.clone());
    trans.trans(ast, &mut sink)?;

    std::fs::write(&jsname, sink.done()?)?;
    Ok(format!("[trans] {} -> {}", path, jsname))
}
```

### 7.3 CLI Integration

**File**: `crates/auto/src/main.rs`
```rust
#[derive(Subcommand)]
enum Commands {
    #[command(about = "Transpile Auto to JavaScript")]
    JavaScript { path: String },
}

// In main()
Some(Commands::JavaScript { path }) => {
    let js = auto_lang::trans_javascript(path.as_str())?;
    println!("{}", js);
}
```

---

## 8. Critical Files Summary

### Must Create
- **crates/auto-lang/src/trans/javascript.rs** - Main transpiler (~800 lines)

### Must Modify
- **crates/auto-lang/src/trans/mod.rs** - Add javascript module
- **crates/auto-lang/src/lib.rs** - Add trans_javascript()
- **crates/auto-lang/src/main.rs** - Add JavaScript CLI command

### Reference Files
- **crates/auto-lang/src/trans/python.rs** - Reference for transpiler patterns
- **crates/auto-lang/src/trans/c.rs** - Reference for stmt/expr handling
- **crates/auto-lang/src/trans/rust.rs** - Reference for code generation

---

## 9. Success Metrics

### Phase 1 (Week 1)
- [ ] `000_hello` test passing
- [ ] Basic expressions working
- [ ] Infrastructure in place

### Phases 2-11 (Weeks 2-6)
- [ ] All priority 1 tests passing (000-020)
- [ ] CLI command working
- [ ] Documentation complete

### Overall
- [ ] 10+ test cases passing
- [ ] Generated JavaScript runs without errors
- [ ] Transpilation time < 1 second
- [ ] Clean ES6+ syntax output

---

## 10. Key Advantages Over Python Transpiler

1. **Simpler Implementation**: No indentation tracking needed!
2. **More Flexible**: JavaScript's dynamic typing matches AutoLang better
3. **Universal**: Runs in browsers and Node.js
4. **F-string Compatible**: Template literals are even closer to AutoLang syntax
5. **No Python Version Constraints**: ES6+ is widely supported

---

## 11. Next Steps

Upon approval:

1. Create implementation branch
2. Begin Phase 1: Core Infrastructure
3. Create test infrastructure
4. Implement incrementally following phases
5. Test at each phase
6. Document progress

---

## 12. Example: Complete Program

### AutoLang Input
```auto
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}

fn main() {
    let p = Point{x: 3, y: 4}
    let m = p.modulus()
    print(f"Modulus: $m")
}
```

### JavaScript Output
```javascript
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    modulus() {
        return this.x * this.x + this.y * this.y;
    }
}

function main() {
    const p = new Point(3, 4);
    const m = p.modulus();
    console.log(`Modulus: ${m}`);
}

main();
```

---

## 13. Risks and Mitigations

### Risk 1: ES6+ Support
**Risk**: Older browsers/environments may not support ES6+

**Mitigation**: Document ES6+ requirement clearly; suggest using modern Node.js or browsers

### Risk 2: Type System Differences
**Risk**: AutoLang is statically typed, JavaScript is dynamic

**Mitigation**: Accept dynamic nature; focus on runtime correctness

### Risk 3: Module System
**Risk**: Deciding between ES6 modules and CommonJS

**Mitigation**: Use ES6 modules by default (modern standard)

### Risk 4: Pattern Matching Limitations
**Risk**: JavaScript's `switch` is less powerful than AutoLang's `is`

**Mitigation**: Document limitations; use `switch` for basic pattern matching

---

## 14. Documentation Plan

### Files to Create
1. **docs/javascript-transpiler.md** - Comprehensive guide
2. **README.md** - Add JavaScript transpiler section

### Content
- Usage instructions
- Language mapping table
- Code examples
- ES6+ features used
- Testing guide
- Limitations and future work

---

## 15. Timeline Summary

| Week | Phase | Deliverable |
|------|-------|-------------|
| 1 | Core + Control Flow | Basic transpiler + if/for |
| 2 | Functions + Pattern Matching | func + is/switch |
| 3 | Arrays + F-strings | Array + template literals |
| 4 | Structs + Methods | Classes with methods |
| 5 | Enums + CLI | Frozen objects + CLI command |
| 5-6 | Advanced Features | Complete feature set |

**Total**: 5-6 weeks for full implementation
