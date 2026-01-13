# JavaScript Transpiler (a2j)

The JavaScript transpiler converts AutoLang source code to modern JavaScript (ES6+), enabling AutoLang programs to run in web browsers, Node.js, and other JavaScript environments.

## Overview

- **Target**: JavaScript ES6+ (ECMAScript 2015 and later)
- **Output**: Clean, idiomatic JavaScript code
- **Use Cases**: Web development, Node.js scripting, browser-based applications
- **Command**: `auto.exe java-script <file.at>`

## Usage

### Basic Usage

```bash
# Transpile AutoLang to JavaScript
auto.exe java-script hello.at

# Output: [trans] hello.at -> hello.js

# Run the generated JavaScript
node hello.js
```

### Example

**AutoLang Input** (`hello.at`):
```auto
fn main() {
    print("hello, world!")
}
```

**JavaScript Output** (`hello.js`):
```javascript
function main() {
    console.log("hello, world!");
}

main();
```

## Language Mapping

### Type Mapping

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

### Variable Declaration Strategy

- `let` (AutoLang immutable) → `const` (JavaScript)
- `mut` (AutoLang mutable) → `let` (JavaScript)
- `var` (AutoLang dynamic) → `let` (JavaScript)

**Rationale**: Use `const` by default for safety, `let` for reassignment.

### Control Flow Mapping

#### If Statements

```auto
// AutoLang
if x > 1 {
    print("Great!")
}
```

```javascript
// JavaScript
if (x > 1) {
    console.log("Great!");
}
```

#### For Loops

```auto
// AutoLang
for i in 0..5 {
    print(i)
}
```

```javascript
// JavaScript
for (let i = 0; i < 5; i++) {
    console.log(i);
}
```

#### Pattern Matching (is → switch)

```auto
// AutoLang
is x {
    0 => print("ZERO")
    1 => print("ONE")
    else => print("Large")
}
```

```javascript
// JavaScript
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
```

### F-Strings → Template Literals

**Perfect Match!** AutoLang f-strings map directly to JavaScript template literals.

```auto
// AutoLang
let name = "World"
print(f"Hello, $name!")
print(f"Result: ${1 + 2}")
```

```javascript
// JavaScript
const name = "World";
console.log(`Hello, ${name}!`);
console.log(`Result: ${1 + 2}`);
```

**Note**: Only difference is backticks instead of double quotes.

### Structs → ES6 Classes

```auto
// AutoLang
type Point {
    x int
    y int
}

fn main() {
    let p = Point{x: 0, y: 0}
    print(p.x)
}
```

```javascript
// JavaScript
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

main();
```

### Methods

```auto
// AutoLang
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}
```

```javascript
// JavaScript
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

**Key Features**:
- `.x` (AutoLang) → `this.x` (JavaScript)
- Automatic `return` for method expressions
- Proper indentation and formatting

### Enums

```auto
// AutoLang
enum Color {
    Red
    Green
    Blue
}

fn main() {
    let c = Color.Red
    print(c)
}
```

```javascript
// JavaScript
const Color = Object.freeze({
    Red: "Red",
    Green: "Green",
    Blue: "Blue"
});

function main() {
    const c = Color.Red;
    console.log(c);
}

main();
```

**Rationale**: `Object.freeze()` prevents modification, making the enum immutable.

### Functions

```auto
// AutoLang
fn add(x int, y int) int {
    x + y
}

fn main() {
    print(add(1, 2))
}
```

```javascript
// JavaScript
function add(x, y) {
    return x + y;
}

function main() {
    console.log(add(1, 2));
}

main();
```

**Notes**:
- Type annotations removed (JavaScript is dynamically typed)
- Functions declared with `function` keyword
- Function names preserved
- Parameters listed without types

## Test Coverage

The JavaScript transpiler includes 9 comprehensive test cases:

| Test | Feature | Status |
|------|---------|--------|
| `000_hello` | Basic print → console.log | ✅ Passing |
| `010_if` | if/else statements | ✅ Passing |
| `011_for` | Range-based for loops | ✅ Passing |
| `012_is` | Pattern matching (is → switch) | ✅ Passing |
| `002_array` | Array literals and indexing | ✅ Passing |
| `003_func` | Function declarations and calls | ✅ Passing |
| `006_struct` | Type declarations → ES6 classes | ✅ Passing |
| `007_enum` | Enum declarations → frozen objects | ✅ Passing |
| `008_method` | Methods with self → this conversion | ✅ Passing |

**Test Location**: `crates/auto-lang/test/a2j/`

**Run Tests**:
```bash
cargo test -p auto-lang -- trans::javascript
```

## Code Examples

### Complete Program

**AutoLang Input**:
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

**JavaScript Output**:
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

### Complex Example with Control Flow

**AutoLang Input**:
```auto
fn main() {
    let x = 10
    let y = 20

    // Test if/else
    if x > 5 {
        print("x is large")
    }

    // Test for loop
    for i in 0..5 {
        print(i)
    }

    // Test pattern matching
    is x {
        0 => print("zero")
        10 => print("ten")
        else => print("other")
    }

    // Test array
    let arr = [1, 2, 3]
    print(arr[0])
}
```

**JavaScript Output**:
```javascript
function main() {
    const x = 10;
    const y = 20;

    if (x > 5) {
        console.log("x is large");
    }

    for (let i = 0; i < 5; i++) {
        console.log(i);
    }

    switch (x) {
        case 0:
            console.log("zero");
            break;
        case 10:
            console.log("ten");
            break;
        default:
            console.log("other");
    }

    const arr = [1, 2, 3];
    console.log(arr[0]);
}

main();
```

## Implementation Details

### Architecture

The JavaScript transpiler is implemented in `crates/auto-lang/src/trans/javascript.rs` (~660 lines).

**Main Components**:
- `JavaScriptTrans` struct - Main transpiler with scope tracking
- `Trans` trait implementation - Core transpilation logic
- Expression handlers - `expr()`, `expr_with_this()` for method bodies
- Statement handlers - `stmt()`, `stmt_with_this()` for method bodies
- Special helpers - `if_body()`, `switch_case_body()` for formatting

**Key Functions**:
```rust
pub struct JavaScriptTrans {
    name: AutoStr,
    scope: Shared<Universe>,
}

impl Trans for JavaScriptTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Split into declarations and main statements
        // Generate declarations first (types, enums, functions)
        // Generate main function or wrap statements
        // Call main at the end
    }
}
```

### Indentation Strategy

JavaScript uses braces (like C), so indentation is simpler than Python:

- **Function body**: `\n    ` before each statement (4 spaces)
- **If/for body**: `\n        ` for statements (8 spaces, 2 levels)
- **Switch cases**: `\n        ` for statements (8 spaces, 2 levels)
- **Methods**: `\n        ` for statements (8 spaces, 2 levels)

**Helper Functions**:
- `if_body()` - Handles if/else statement body indentation
- `switch_case_body()` - Handles switch case indentation
- `body()` - Handles function body indentation

### Special Conversions

#### print → console.log

```javascript
// AutoLang
print("hello")

// JavaScript
console.log("hello")
```

Implemented in `call()` method with special detection.

#### self → this in Methods

```javascript
// AutoLang
fn method() {
    .x + .y
}

// JavaScript
method() {
    return this.x + this.y;
}
```

Implemented in `expr_with_this()` with recursive traversal.

#### StoreKind Handling

```rust
match store.kind {
    StoreKind::Let => out.write(b"const ")?,
    StoreKind::Mut | StoreKind::Var => out.write(b"let ")?,
    _ => {}, // Field and CVar don't need declaration
}
```

## Limitations

### Current Limitations

1. **Return Statements**: AutoLang's `return` is parsed as `Node` expressions, not fully supported
2. **If Expressions**: Ternary operator not implemented (if statements only)
3. **Lambdas**: Arrow functions not implemented
4. **Modules**: ES6 modules (import/export) not implemented
5. **Generators**: Generator functions not supported
6. **Async/Await**: Not implemented

### Workarounds

For return statements, use expression-only functions:
```auto
// Instead of:
fn add(x int, y int) int {
    return x + y
}

// Use:
fn add(x int, y int) int {
    x + y
}
```

## Advantages Over Python Transpiler

1. **Simpler Implementation**: No indentation tracking needed!
2. **More Flexible**: JavaScript's dynamic typing matches AutoLang better
3. **Universal**: Runs in browsers and Node.js
4. **F-string Compatible**: Template literals are even closer to AutoLang syntax
5. **No Python Version Constraints**: ES6+ is widely supported

## Running Generated JavaScript

### Node.js

```bash
# Install Node.js from https://nodejs.org/

# Run generated JavaScript
node hello.js
```

### Browser

```html
<!DOCTYPE html>
<html>
<head>
    <script src="hello.js"></script>
</head>
<body>
    <script>
        // Call main function
        main();
    </script>
</body>
</html>
```

### Online Testing

Use online JavaScript runners like:
- https://replit.com/languages/javascript
- https://codepen.io/pen/
- https://jsbin.com/

## Version Requirements

**Node.js**: v12.0.0 or later (for ES6+ support)
**Browsers**: Any modern browser (Chrome 51+, Firefox 54+, Safari 10+, Edge 15+)

## Performance

- **Transpilation Speed**: < 1 second for typical files
- **Generated Code**: Fast, idiomatic JavaScript
- **No Runtime Dependencies**: Pure JavaScript, no polyfills needed

## Future Enhancements

Potential future improvements:

1. **Arrow Functions**: Convert simple lambdas to `=>` functions
2. **Ternary Operator**: Implement if expressions as `? :`
3. **ES6 Modules**: Add `import`/`export` statements
4. **Source Maps**: Generate source maps for debugging
5. **JSDoc Comments**: Generate documentation from type annotations
6. **Optimization Passes**: Dead code elimination, constant folding
7. **Babel Integration**: Option to target older JavaScript versions

## Troubleshooting

### Common Issues

**Issue**: Generated JavaScript has syntax errors
- **Solution**: Check that you're using a modern Node.js version (v12+)

**Issue**: `this` is undefined in methods
- **Solution**: Make sure to call methods with `new` keyword for constructors

**Issue**: F-strings not interpolating
- **Solution**: Use backticks `\`` instead of double quotes

## Related Documentation

- [Python Transpiler](python-transpiler.md) - Auto to Python transpiler
- [C Transpiler](../stdlib/c/README.md) - Auto to C transpiler
- [Rust Transpiler](../stdlib/rust/README.md) - Auto to Rust transpiler
- [Language Reference](../language-reference.md) - AutoLang language guide

## Contributing

When modifying the JavaScript transpiler:

1. **Add Tests**: All new features must have test cases
2. **Update Documentation**: Keep this file accurate
3. **Check Warnings**: Maintain zero compilation warnings
4. **Verify Output**: Ensure generated JavaScript runs successfully
5. **Test Coverage**: Maintain > 90% code coverage

## License

MIT License - See project root for details
