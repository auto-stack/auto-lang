# auto-lang C Compiler (autoc)

A C implementation of the auto-lang compiler, translated from the original Rust codebase.

## Features

- **Lexer**: Tokenizes auto-lang source code
- **Parser**: Implements a Pratt parser for expression parsing
- **Interpreter**: Evaluates auto-lang code at runtime
- **Transpiler**: Transpiles auto-lang code to C
- **Scope System**: Manages lexical scopes and variable bindings
- **Value System**: Supports integers, floats, strings, arrays, objects, and more

## Building

### Linux / macOS (with GCC or Clang)

```bash
make
```

For a debug build:
```bash
make debug
```

### Windows (with MinGW)

```bash
gcc -Wall -Wextra -std=c11 -O2 -c astr.c
gcc -Wall -Wextra -std=c11 -O2 -c value.c
gcc -Wall -Wextra -std=c11 -O2 -c lexer.c
gcc -Wall -Wextra -std=c11 -O2 -c parser.c
gcc -Wall -Wextra -std=c11 -O2 -c universe.c
gcc -Wall -Wextra -std=c11 -O2 -c eval.c
gcc -Wall -Wextra -std=c11 -O2 -c trans_c.c
gcc -Wall -Wextra -std=c11 -O2 -c main.c
gcc *.o -o autoc.exe
```

### Windows (with MSVC)

```cmd
build.bat
```

### Using CMake

```bash
mkdir build
cd build
cmake ..
cmake --build .
```

## Usage

### Evaluate expressions

```bash
./autoc -e "1 + 2 * 3"
# Output: 7

./autoc -e "var a = 10; a * 2"
# Output: 20
```

### Transpile to C

```bash
./autoc -t "var x = 42"
# Outputs C code for header and source

./autoc -t "var x = 42" -o myprogram
# Transpiles and writes to myprogram.h and myprogram.c
```

### Transpile files

```bash
./autoc -t input.at -o output
# Generates output.h and output.c
```

### Run files

```bash
./autoc script.at
```

### Interactive REPL

```bash
./autoc --repl
```

## Language Examples

### Variables

```auto
var x = 42
let y = 10  # immutable
mut z = 5   # mutable with mut keyword
```

### Arithmetic

```auto
1 + 2 * 3    # 7
(1 + 2) * 3  # 9
```

### Conditionals

```auto
if true {
    42
} else {
    0
}
```

### Loops

```auto
for i in 0..10 {
    i * 2
}
```

### Arrays

```auto
[1, 2, 3, 4, 5]
```

### Objects

```auto
{
    name: "Auto",
    age: 1
}
```

## Transpiler Features

The auto-lang to C transpiler supports:

- **Variables**: `var`, `let`, `mut` declarations
- **Control flow**: `if/else`, `for` loops
- **Operators**: Arithmetic, comparison, logical
- **Functions**: Function definitions and calls
- **Print**: `print()` function converted to `printf()`

### Example Transpilation

Input (auto-lang):
```auto
var x = 42
if x > 0 {
    print(x)
}
```

Output (C):
```c
#include <stdio.h>

int main(void) {
    int x = 42;
    if (x > 0) {
        printf("%d\n", x);
    }
    return 0;
}
```

## Project Structure

```
autoc/
├── autoc.h      # Main header with all type definitions
├── trans_c.h    # Transpiler header
├── astr.c       # AutoString utilities
├── value.c      # Value runtime system
├── lexer.c      # Lexical analysis
├── parser.c     # Parsing (Pratt parser)
├── universe.c   # Scope and symbol management
├── eval.c       # Expression/statement evaluation
├── trans_c.c    # Auto-lang to C transpiler
├── main.c       # Main entry point
├── Makefile     # Build configuration for Unix
├── build.bat    # Build script for Windows (MSVC)
└── CMakeLists.txt # CMake configuration
```

## Implementation Status

- [x] Lexer - Complete
- [x] Parser - Basic implementation
- [x] Value system - Core types
- [x] Arithmetic operations
- [x] Comparison operations
- [x] Variable declaration (var, let, mut)
- [x] If/else statements
- [x] For loops
- [x] Arrays and objects
- [x] **C Transpiler** - Basic implementation
- [ ] Functions
- [ ] Types and type declarations
- [ ] Methods
- [ ] Format strings
- [ ] Advanced C transpilation features

## Testing

Run the test suite:

```bash
make test
```

## License

This is a derivative work of the auto-lang compiler, originally written in Rust.
