# C Standard Library Layer

This directory contains the C standard library bindings for AutoLang.

## Module Structure

### `stdio.c.at`
C `<stdio.h>` standard input/output functions.

**Import syntax:**
```auto
use c.stdio: printf, fopen, fclose, FILE
```

**Available functions:**
- `printf`, `fprintf`, `sprintf`, `snprintf`
- `scanf`, `fscanf`, `sscanf`
- `fopen`, `fclose`, `freopen`
- `fread`, `fwrite`
- `fgets`, `fputs`, `fgetc`, `fputc`
- `fseek`, `ftell`, `rewind`
- `feof`, `ferror`, `clearerr`
- And more...

### `stdlib.c.at`
C `<stdlib.h>` standard library functions.

**Import syntax:**
```auto
use c.stdlib: malloc, free, exit, atoi, rand
```

**Available functions:**
- **Memory Management:** `malloc`, `calloc`, `realloc`, `free`
- **Process Control:** `exit`, `system`
- **Environment:** `getenv`, `setenv`
- **Conversions:** `atoi`, `atol`, `atof`, `strtol`, `strtod`
- **Random Numbers:** `rand`, `srand`
- **Sorting:** `qsort`, `bsearch`
- And more...

## Usage Examples

### Using printf from c.stdio

```auto
use c.stdio: printf

fn main() {
    printf(c"Hello, %s!\n", c"world")
}
```

### Using malloc/free from c.stdlib

```auto
use c.stdlib: malloc, free

fn main() {
    let ptr *void = malloc(1024)
    if ptr != nil {
        free(ptr)
    }
}
```

### Combining multiple imports

```auto
use c.stdio: printf, fopen, fclose, FILE
use c.stdlib: exit, malloc, free

fn main() {
    printf(c"Allocating memory...\n")
    let ptr *void = malloc(1024)
    if ptr != nil {
        printf(c"Success!\n")
        free(ptr)
    } else {
        printf(c"Failed!\n")
        exit(1)
    }
}
```

## Design Principles

1. **Separation of Concerns**: C standard library functions are separated from AutoLang's higher-level abstractions
2. **Module Paths**: Use `c.stdio` and `c.stdlib` as module paths to clearly indicate these are C library bindings
3. **Explicit Imports**: All C functions must be explicitly imported before use
4. **Type Safety**: C types like `FILE`, `va_list`, `div_t` are properly declared

## Relationship with AutoLang Standard Library

The AutoLang standard library (`stdlib/auto/`) builds on top of this C layer:

- `auto/io.at` → Uses `c.stdio` for File I/O operations
- `auto/mem.at` → Uses `c.stdlib` for memory management (future)
- Other AutoLang modules can import from `c.stdio` and `c.stdlib` as needed

## Adding New C Library Bindings

When adding new C standard library bindings:

1. Choose the appropriate module (`stdio.c.at` or `stdlib.c.at`)
2. Add function declarations with proper signatures
3. Include relevant types and constants
4. Update this README if adding new categories

Example:
```auto
## String Operations
fn strlen(s str) int
fn strcmp(s1 str, s2 str) int
fn strcpy(dst str, src str) str
```

## Future Expansions

Additional C standard library headers that could be added:
- `string.c.at` - `<string.h>` string operations
- `math.c.at` - `<math.h>` mathematical functions
- `time.c.at` - `<time.h>` date and time functions
- `ctype.c.at` - `<ctype.h>` character handling
- And more...
