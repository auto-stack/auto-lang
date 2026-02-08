# a2r Transpiler Tutorials

## Tutorial 1: Your First Auto-to-Rust Program

### Prerequisites

- AutoLang compiler installed
- Basic familiarity with Rust syntax

### Step 1: Write AutoLang Code

Create a file `hello.at`:

```auto
fn main() {
    print("Hello from AutoLang!")
}
```

### Step 2: Transpile to Rust

```bash
auto.exe transpile rust hello.at hello.rs
```

### Step 3: Compile and Run

```bash
rustc hello.rs -o hello
./hello
```

**Output:**
```
Hello from AutoLang!
```

## Tutorial 2: Working with Structs

### AutoLang Input (`shapes.at`)

```auto
type Point {
    x int
    y int
}

type Circle {
    radius float
    center Point
}

fn main() {
    let p = Point(10, 20)
    let c = Circle(5.0, p)
    print("Circle at:", c.center.x, ",", c.center.y)
}
```

### Rust Output (`shapes.rs`)

```rust
struct Point {
    x: i32,
    y: i32,
}

struct Circle {
    radius: f32,
    center: Point,
}

fn main() {
    let p: Point = Point { x: 10, y: 20 };
    let c: Circle = Circle { radius: 5.0, center: p };
    println!("Circle at: {}, {}", c.center.x, c.center.y);
}
```

### Compile and Run

```bash
rustc shapes.rs -o shapes
./shapes
```

## Tutorial 3: Using Generics

### Problem

Create a generic stack type that works with any type.

### AutoLang Solution (`stack.at`)

```auto
spec Storage<T> {
    fn push(item T)
    fn pop() T
}

type Stack<T> as Storage<T> {
    items [T] 100
    count u32
}

impl Stack<T> {
    fn new() Stack {
        Stack {
            items: [T]::default(),
            count: 0
        }
    }

    fn push(item T) {
        if .count < 100 {
            .items[.count] = item
            .count = .count + 1
        }
    }

    fn pop() T {
        if .count > 0 {
            .count = .count - 1
            return .items[.count]
        }
        T::default()
    }
}
```

### Rust Output

```rust
trait Storage<T> {
    fn push(&mut self, item: T);
    fn pop(&mut self) -> T;
}

struct Stack<T> {
    items: [T; 100],
    count: u32,
}

impl<T> Storage<T> for Stack<T> {
    fn push(&mut self, item: T) {
        if self.count < 100 {
            self.items[self.count as usize] = item;
            self.count += 1;
        }
    }

    fn pop(&mut self) -> T {
        if self.count > 0 {
            self.count -= 1;
            self.items[self.count as usize]
        } else {
            T::default()
        }
    }
}

impl<T> Stack<T> {
    fn new() -> Stack<T> {
        Stack {
            items: Default::default(),
            count: 0,
        }
    }
}
```

## Tutorial 4: Closures and Functional Programming

### AutoLang Code (`map_filter.at`)

```auto
fn process(numbers [int]) [int] {
    let add_one = (n int) => n + 1
    let is_even = (n int) => n % 2 == 0

    let mapped = numbers.map(add_one)
    let filtered = mapped.filter(is_even)
    return filtered
}
```

### Rust Output

```rust
fn process(numbers: [i32]) -> [i32] {
    let add_one = |n: i32| n + 1;
    let is_even = |n: i32| n % 2 == 0;

    let mapped = numbers.iter().map(add_one);
    let filtered: Vec<i32> = mapped.filter(|x| is_even(*x)).collect();
    // Note: Arrays in Rust have fixed size, so we use Vec
    // The transpiler may need adjustment for array returns
    filtered.try_into().unwrap_or_default()
}
```

## Tutorial 5: Trait-Based Polymorphism

### AutoLang Code (`polymorphism.at`)

```auto
spec Speaker {
    fn speak()
}

type Dog as Speaker {
    name str
}

type Cat as Speaker {
    name str
}

impl Dog {
    fn speak() {
        print(self.name, ": Woof!")
    }
}

impl Cat {
    fn speak() {
        print(self.name, ": Meow!")
    }
}

fn make_speak(speaker Speaker) {
    speaker.speak()
}

fn main() {
    let dog = Dog { name: "Buddy" }
    let cat = Cat { name: "Whiskers" }

    make_speak(dog)
    make_speak(cat)
}
```

### Rust Output

```rust
trait Speaker {
    fn speak(&self);
}

struct Dog {
    name: String,
}

struct Cat {
    name: String,
}

impl Speaker for Dog {
    fn speak(&self) {
        println!("{}: Woof!", self.name);
    }
}

impl Dog {
    fn speak(&self) {
        println!("{}: Woof!", self.name);
    }
}

impl Speaker for Cat {
    fn speak(&self) {
        println!("{}: Meow!", self.name);
    }
}

impl Cat {
    fn speak(&self) {
        println!("{}: Meow!", self.name);
    }
}

fn make_speak(speaker: &dyn Speaker) {
    speaker.speak();
}

fn main() {
    let dog: Dog = Dog { name: "Buddy".into() };
    let cat: Cat = Cat { name: "Whiskers".into() };

    make_speak(&dog);
    make_speak(&cat);
}
```

## Tutorial 6: Pattern Matching

### AutoLang Code (`match.at`)

```auto
fn describe(value int) {
    is value {
        0 => "zero"
        1 => "one"
        2..10 => "small"
        10..100 => "large"
        _ => "huge"
    }
}
```

### Rust Output

```rust
fn describe(value: i32) -> &'static str {
    match value {
        0 => "zero",
        1 => "one",
        2..=10 => "small",
        10..=100 => "large",
        _ => "huge",
    }
}
```

## Tutorial 7: Memory Management

### AutoLang Code (`ownership.at`)

```auto
fn process(data List) {
    // Immutable borrow
    let view = data.view
    print("Length:", view.len())

    // Mutable borrow
    data.mut.push(42)
}

fn transfer(data List) -> List {
    // Move (default)
    return data
}
```

### Rust Output

```rust
fn process(data: List) {
    // Immutable borrow
    let view = &data;
    println!("Length: {}", view.len());

    // Mutable borrow
    data.push(42);
}

fn transfer(data: List) -> List {
    // Move
    data
}
```

## Tutorial 8: Platform-Specific Code

### Directory Structure

```
project/
├── printer.at          # Interface
├── printer.rs.at       # Rust implementation
└── printer.c.at        # C implementation
```

### Interface (`printer.at`)

```auto
type Printer {
    data str
}

ext Printer {
    #[rs]
    fn print() {
        // Rust implementation
    }

    #[c]
    fn print() {
        // C implementation
    }
}
```

### Rust Implementation (`printer.rs.at`)

```auto
use crate::printer::Printer

impl Printer {
    #[rs]
    fn print() {
        // Rust-specific code
        say(self.data)
    }
}
```

### Usage

```bash
# Transpile for Rust (uses printer.rs.at)
auto.exe transpile rust main.at output.rs

# Compile and run
rustc output.rs -o program
./program
```

## Tutorial 9: Error Handling

### AutoLang Code (`errors.at`)

```auto
type Result<T> {
    value T
    error str
}

fn divide(a int, b int) Result<int> {
    if b == 0 {
        return Result::error("Division by zero")
    }
    return Result::value(a / b)
}

fn main() {
    let result = divide(10, 2)
    is result {
        Result::value(v) => print("Result:", v)
        Result::error(e) => print("Error:", e)
    }
}
```

### Rust Output

```rust
struct Result<T> {
    value: T,
    error: String,
}

fn divide(a: i32, b: i32) -> Result<i32> {
    if b == 0 {
        return Result {
            value: 0, // placeholder
            error: "Division by zero".into(),
        };
    }
    Result {
        value: a / b,
        error: String::new(),
    }
}

fn main() {
    let result: Result<i32> = divide(10, 2);
    match result {
        Result::value(v) => println!("Result: {}", v),
        Result::error(e) => println!("Error: {}", e),
    }
}
```

## Tutorial 10: Advanced Generics

### AutoLang Code (`advanced.at`)

```auto
spec Iterator<T> {
    fn next() Option<T>
    type Item
}

type RangeIterator<T> as Iterator<T> {
    current T
    end T
    step T
}

impl<T> RangeIterator<T> {
    fn new(start T, end T, step T) RangeIterator {
        RangeIterator {
            current: start,
            end: end,
            step: step
        }
    }

    fn next() Option<T> {
        if self.current >= self.end {
            return Option::none
        }
        let value = self.current
        self.current = self.current + self.step
        return Option::some(value)
    }
}
```

## Best Practices

### 1. Type Annotations

```auto
// Good: Explicit types where helpful
let count: uint = 42

// Good: Let type inference work
let name = "Alice"

// Good: Closure with explicit types
let add = (a int, b int) => a + b
```

### 2. Generic Naming

```auto
// Good: Descriptive type parameter names
type Map<K, V> {
    keys [K]
    values [V]
}

// Good: Constraints in spec
spec Numeric<T> {
    fn add(a T, b T) T
        where T: add<T>
}
```

### 3. Memory Safety

```auto
// Good: Use views for reads
fn read_list(data List) {
    let view = data.view
    print(view.len())
}

// Good: Use mut for writes
fn append_list(mut data List) {
    data.mut.push(42)
}
```

### 4. Error Handling

```auto
// Good: Use Result types
fn parse(input str) Result<int> {
    if is_valid(input) {
        return Result::value(parse_int(input))
    }
    return Result::error("Invalid input")
}
```

## Common Patterns

### Factory Pattern

```auto
type Factory {
}

impl Factory {
    #[rs]
    fn create_point(x int, y int) Point {
        Point { x: x, y: y }
    }
}
```

### Builder Pattern

```auto
type Builder {
    data str
    count int
}

impl Builder {
    fn new() Builder {
        Builder {
            data: ""
            count: 0
        }
    }

    fn set_data(data str) Builder {
        .data = data
        return this
    }

    fn set_count(count int) Builder {
        .count = count
        return this
    }

    fn build() Builder {
        this
    }
}
```

## Troubleshooting

### Issue: "Type errors during transpilation"

**Problem**: Transpiler reports type mismatch.

**Solution**: Check that:
- Type annotations match between definition and usage
- Generic parameters are properly declared
- Return types match the actual return value

### Issue: "Missing method in trait impl"

**Problem**: Method not found in trait implementation.

**Solution**: Ensure:
- Method signatures exactly match the trait definition
- `&self` is used for instance methods
- Return types match

### Issue: "Closure doesn't infer types"

**Problem**: Closure parameter types are unknown.

**Solution**: Add explicit type annotations:
```auto
let add = (a int, b int) => a + b
```

## Next Steps

- Explore the [API Documentation](a2r-api-documentation.md)
- Read the [Transpiler Guide](a2r-transpiler-guide.md)
- Check out [test examples](../../test/a2r/)
- Contribute your own examples!
