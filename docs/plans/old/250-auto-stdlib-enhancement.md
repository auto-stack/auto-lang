# Plan 250: Auto Standard Library Enhancement

## Status: COMPLETE

## Goal

Design and implement an enhanced Auto standard library, informed by Rust, Python, Go, and C stdlib analysis. Focus on AutoVM execution and a2r transpiler. Use OOP with specs, extension methods, and flat type hierarchy.

## Current State

Auto already has: collections (List, HashMap, HashSet, VecDeque, BTreeMap), iterator system, string ops, math, file I/O, HTTP, JSON, TCP, async basics, process/env, logging, URL parsing.

## What's Missing (Cross-Language Analysis)

| Module | Rust | Python | Go | C | Auto Status |
|--------|------|--------|----|---|-------------|
| Result<T,E> | ✅ | ✅ (exceptions) | ✅ | ✅ (errno) | ❌ Only May<T> |
| Sorting | ✅ | ✅ | ✅ | ✅ (qsort) | ❌ |
| Random | ✅ | ✅ | ✅ | ✅ | ❌ |
| DateTime | ✅ | ✅ | ✅ | ✅ | ❌ Only now_ms |
| Duration | ✅ | ✅ | ✅ | — | ❌ |
| Base64/Hex | external | ✅ | ✅ | — | ❌ |
| CSV | external | ✅ | ✅ | — | ❌ |
| Display/Debug | ✅ | ✅ (__str__) | ✅ (Stringer) | — | ❌ |
| Error hierarchy | ✅ | ✅ | ✅ | — | ❌ |
| Test framework | ✅ | ✅ | ✅ | — | ❌ Minimal |
| Buffered I/O | ✅ | ✅ | ✅ | ✅ | ❌ |
| Walk dir | ✅ | ✅ | ✅ | — | Partial |
| CLI args | external | ✅ | ✅ | — | ❌ |
| Clone/Cmp specs | ✅ | — | — | — | ❌ |
| Hash (crypto) | external | ✅ | ✅ | — | Partial (SHA256) |

## Design Principles

1. **OOP but flat**: Use specs + extension methods, minimal inheritance
2. **#[vm] for runtime**: Complex operations backed by Rust VM
3. **Pure Auto for types**: Result<T,E>, Duration, specs defined in .at files
4. **Extension methods on builtins**: `str`, `int`, `float`, `List` get methods via `ext`
5. **Consistent naming**: snake_case functions, PascalCase types, matching existing conventions

---

## Phase 1: Core Specs (Foundation Traits)

### Files to create/modify:
- `stdlib/auto/cmp.at` — Comparison specs
- `stdlib/auto/clone.at` — Clone spec
- `stdlib/auto/default.at` — Default spec
- `stdlib/auto/fmt.at` — Display/Debug specs
- `stdlib/auto/prelude.at` — Add new spec re-exports

### cmp.at
```auto
// Comparison ordering
enum Ordering {
    Less = -1
    Equal = 0
    Greater = 1
}

// Comparison spec — types that can be totally ordered
spec Cmp {
    fn cmp(other Self) Ordering
}

// Extension methods using Cmp
ext int for Cmp { ... }
ext str for Cmp { ... }
```

### clone.at
```auto
spec Clone {
    fn clone() Self
}
```

### default.at
```auto
spec Default {
    fn default() Self
}
```

### fmt.at
```auto
// String representation specs
spec Display {
    fn fmt() str
}

spec Debug {
    fn debug() str
}

// Built-in type extensions
ext str for Display { fn fmt() str { return self } }
ext int for Display { fn fmt() str { ... } }  // #[vm]
ext float for Display { fn fmt() str { ... } } // #[vm]
ext bool for Display { fn fmt() str { ... } }  // #[vm]
```

### VM functions needed:
- `int_to_str` (already exists)
- `float_to_str` (already exists)
- `bool_to_str` (new, ID 2750)

**Commit after Phase 1.**

---

## Phase 2: Result<T,E> — Proper Error Handling

### Files to create:
- `stdlib/auto/result.at` — Result<T,E> type
- `stdlib/auto/error.at` — Error base type
- `stdlib/auto/prelude.at` — Add Result, Error re-exports

### result.at
```auto
// Result type — replaces C errno pattern, matches Rust/Go style
enum Result<T, E> {
    Ok T
    Err E
}

// Core methods via extension
ext Result {
    fn is_ok() bool        // #[vm]
    fn is_err() bool       // #[vm]
    fn unwrap() T          // #[vm] — panic on Err
    fn unwrap_or(default T) T
    fn unwrap_err() E
    fn ok() May<T>         // Convert to May
    fn err() May<E>
    fn map(fn(T) U) Result<U, E>
    fn map_err(fn(E) F) Result<T, F>
    fn and_then(fn(T) Result<U, E>) Result<U, E>
}
```

### error.at
```auto
// Simple error type — flat hierarchy, not deep OOP
type Error {
    code int
    message str
}

// Factory functions
fn ok(val T) Result<T, Error>
fn err(code int, message str) Error

// Extension for Display
ext Error for Display {
    fn fmt() str { ... }
}
```

### VM functions needed:
- `result_is_ok` (ID 2800)
- `result_is_err` (ID 2801)
- `result_unwrap` (ID 2802)
- `result_unwrap_or` (ID 2803)

**Commit after Phase 2.**

---

## Phase 3: sort — Sorting with Comparators

### Files to create:
- `stdlib/auto/sort.at` — Sort functions

### sort.at
```auto
// Sort functions — operates on List<T> where T implements Cmp
fn sort(list List) void                    // #[vm] — in-place sort
fn sort_by(list List, cmp_fn fn(a Val, b Val) int) void  // #[vm] — custom comparator
fn sorted(list List) List                  // #[vm] — returns new sorted list
fn reverse(list List) void                 // #[vm] — in-place reverse
fn reversed(list List) List                // #[vm] — returns new reversed list
```

### VM functions needed:
- `list_sort` (ID 2810)
- `list_sort_by` (ID 2811)
- `list_reverse` (ID 2812)

**Commit after Phase 3.**

---

## Phase 4: random — Random Number Generation

### Files to create:
- `stdlib/auto/random.at` — Random type and functions

### random.at
```auto
// Random number generator — simple, Go-style
type Random {
    // Opaque, #[vm]-backed
}

// Factory
fn random() Random                // #[vm] — seeded from system entropy
fn random_with_seed(seed int) Random  // #[vm] — deterministic seed

// Instance methods
ext Random {
    fn int(max int) int           // #[vm] — random int in [0, max)
    fn float() float              // #[vm] — random float in [0, 1)
    fn bool() bool                // #[vm] — random boolean
    fn bytes(n int) List          // #[vm] — random bytes
    fn shuffle(list List) void    // #[vm] — Fisher-Yates shuffle
}

// Convenience functions (thread-local RNG)
fn rand_int(max int) int          // #[vm]
fn rand_float() float             // #[vm]
fn rand_bool() bool               // #[vm]
```

### VM functions needed:
- `random_new` (ID 2820)
- `random_seeded` (ID 2821)
- `random_int` (ID 2822)
- `random_float` (ID 2823)
- `random_bool` (ID 2824)
- `random_bytes` (ID 2825)
- `random_shuffle` (ID 2826)

**Commit after Phase 4.**

---

## Phase 5: time — Duration and DateTime

### Files to create:
- `stdlib/auto/duration.at` — Duration type
- `stdlib/auto/datetime.at` — DateTime type
- Update `stdlib/auto/time.at` — Enhanced time functions

### duration.at
```auto
// Duration — represents a span of time (Rust/Go style)
type Duration {
    _ms int  // Internal: milliseconds

    fn new(ms int) Duration
    fn from_secs(secs int) Duration
    fn from_mins(mins int) Duration
    fn from_hours(hours int) Duration

    fn secs() int       // #[vm]
    fn millis() int     // #[vm]
    fn micros() int     // #[vm]
}

// Arithmetic on durations
ext Duration {
    fn add(other Duration) Duration
    fn sub(other Duration) Duration
    fn mul(factor int) Duration
}

fn dur(secs int) Duration { return Duration.from_secs(secs) }
```

### datetime.at
```auto
// DateTime — calendar date and time
type DateTime {
    _opaque int  // #[vm] opaque handle

    static fn now() DateTime                          // #[vm]
    static fn from_timestamp(ts int) DateTime         // #[vm]
    static fn from_ymd(year int, month int, day int) DateTime  // #[vm]

    fn year() int       // #[vm]
    fn month() int      // #[vm]
    fn day() int        // #[vm]
    fn hour() int       // #[vm]
    fn minute() int     // #[vm]
    fn second() int     // #[vm]
    fn weekday() int    // #[vm] — 0=Monday, 6=Sunday
    fn timestamp() int  // #[vm] — Unix timestamp
    fn format(layout str) str  // #[vm] — strftime-style
}

ext DateTime for Display {
    fn fmt() str { return self.format("%Y-%m-%d %H:%M:%S") }
}

ext DateTime for Cmp {
    fn cmp(other DateTime) Ordering { ... }
}
```

### VM functions needed:
- `datetime_now` (ID 2700) — already exists as chrono_local_now
- `datetime_from_timestamp` (ID 2830)
- `datetime_from_ymd` (ID 2831)
- `datetime_year` (ID 2701) — already exists
- `datetime_month` (ID 2702) — already exists
- `datetime_day` (ID 2703) — already exists
- `datetime_hour` (ID 2704) — already exists
- `datetime_minute` (ID 2705) — already exists
- `datetime_second` (ID 2706) — already exists
- `datetime_weekday` (ID 2832)
- `datetime_timestamp` (ID 2707) — already exists
- `datetime_format` (ID 2708) — already exists
- `duration_secs` (ID 2833)
- `duration_millis` (ID 2834)

**Commit after Phase 5.**

---

## Phase 6: encoding — Base64, Hex, CSV

### Files to create:
- `stdlib/auto/encoding/base64.at`
- `stdlib/auto/encoding/hex.at`
- `stdlib/auto/encoding/csv.at`

### base64.at
```auto
fn encode(data str) str      // #[vm]
fn decode(data str) May<str> // #[vm]
```

### hex.at
```auto
fn encode(data str) str      // #[vm]
fn decode(data str) May<str> // #[vm]
```

### csv.at
```auto
type CsvReader {
    // #[vm] opaque
}

type CsvWriter {
    // #[vm] opaque
}

fn csv_parse(text str) List<List<str>>          // #[vm] — parse CSV text
fn csv_parse_with_delimiter(text str, delim str) List<List<str>>  // #[vm]
fn csv_encode(rows List<List<str>>) str          // #[vm] — encode to CSV
fn csv_encode_with_delimiter(rows List<List<str>>, delim str) str  // #[vm]
```

### VM functions needed:
- `base64_encode` (ID 2710) — already exists
- `base64_decode` (ID 2711) — already exists
- `hex_encode` (ID 2720) — already exists
- `hex_decode` (ID 2721) — already exists
- `csv_parse` (ID 2840)
- `csv_parse_delim` (ID 2841)
- `csv_encode` (ID 2842)
- `csv_encode_delim` (ID 2843)

**Commit after Phase 6.**

---

## Phase 7: fs — Enhanced Filesystem

### Files to create:
- `stdlib/auto/fs.at` — Enhanced filesystem module

### fs.at
```auto
// File metadata
type Metadata {
    size int
    is_dir bool
    is_file bool
    modified int   // timestamp ms
    readonly bool
}

// Enhanced operations beyond basic file.at
fn metadata(path str) May<Metadata>     // #[vm]
fn walk(dir str) List<str>              // #[vm] — recursive file listing
fn walk_files(dir str) List<str>        // #[vm] — only files
fn temp_dir() str                       // #[vm] — system temp directory
fn temp_file() str                      // #[vm] — create temp file, return path
fn copy_recursive(src str, dst str) void // #[vm]
fn rename(old str, new str) void        // #[vm]
fn read_dir(dir str) List<str>          // #[vm] — non-recursive listing
fn canonical(path str) str              // #[vm] — absolute canonical path
fn ext(path str) str                    // #[vm] — file extension
fn stem(path str) str                   // #[vm] — filename without extension
fn filename(path str) str               // #[vm] — filename component
fn parent(path str) str                 // #[vm] — parent directory
fn join(paths... str) str               // #[vm] — join path components
```

### VM functions needed:
- `fs_metadata` (ID 2850)
- `fs_walk` (ID 2851) — already exists as `file_walk`
- `fs_walk_files` (ID 2852)
- `fs_temp_dir` (ID 2853)
- `fs_temp_file` (ID 2854)
- `fs_copy_recursive` (ID 2855)
- `fs_rename` (ID 2856)
- `fs_read_dir` (ID 2857)
- `fs_canonical` (ID 2858)
- `fs_ext` (ID 2859)
- `fs_stem` (ID 2860)
- `fs_join` (ID 2861)

**Commit after Phase 7.**

---

## Phase 8: test — Testing Framework

### Files to create:
- `stdlib/auto/test.at` — Testing utilities

### test.at
```auto
// Assertion functions
fn assert_eq(actual Val, expected Val, message str) void    // #[vm]
fn assert_ne(actual Val, expected Val, message str) void    // #[vm]
fn assert_true(condition bool, message str) void             // #[vm]
fn assert_false(condition bool, message str) void            // #[vm]
fn assert_contains(haystack str, needle str, message str) void  // #[vm]
fn assert_len(collection List, expected int, message str) void   // #[vm]
fn assert_ok(result Result, message str) void                // #[vm]
fn assert_err(result Result, message str) void               // #[vm]

// Short forms (no message)
fn assert_eq(actual Val, expected Val) void
fn assert_ne(actual Val, expected Val) void
fn assert_true(condition bool) void
fn assert_false(condition bool) void

// Test runner registration (future: auto-discovery)
fn test(name str, fn_ref fn() void) void                     // #[vm]
fn bench(name str, fn_ref fn() void) void                    // #[vm]
```

### VM functions needed:
- `test_assert_eq` (ID 2870)
- `test_assert_ne` (ID 2871)
- `test_assert_true` (ID 2872)
- `test_assert_false` (ID 2873)
- `test_assert_contains` (ID 2874)
- `test_assert_len` (ID 2875)
- `test_assert_ok` (ID 2876)
- `test_assert_err` (ID 2877)

**Commit after Phase 8.**

---

## Phase 9: fmt — Formatted Output

### Files to create:
- `stdlib/auto/format.at` — String formatting

### format.at
```auto
// Printf-style formatting (Go/C inspired)
fn sprintf(template str, args... Val) str     // #[vm] — format to string
fn printf(template str, args... Val) void     // #[vm] — format to stdout
fn eprintf(template str, args... Val) void    // #[vm] — format to stderr

// Format specifiers: {} (Rust-style positional)
// "Hello {}, you are {} years old" -> "Hello Alice, you are 30 years old"
// "Hello {0}, {0} is {1}" -> positional reuse

fn format(template str, args... Val) str      // #[vm] — alias for sprintf
```

### VM functions needed:
- `fmt_sprintf` (ID 2880)
- `fmt_printf` (ID 2881)
- `fmt_eprintf` (ID 2882)

**Commit after Phase 9.**

---

## Phase 10: hash — Cryptographic Hashing

### Files to create:
- `stdlib/auto/hash.at` — Hash utilities

### hash.at
```auto
// Simple hash functions — Go crypto style
fn md5(data str) str          // #[vm]
fn sha1(data str) str         // #[vm]
fn sha256(data str) str       // #[vm]
fn sha512(data str) str       // #[vm]
fn hmac_sha256(data str, key str) str  // #[vm]

// File hashing
fn file_md5(path str) May<str>       // #[vm]
fn file_sha256(path str) May<str>    // #[vm]
```

### VM functions needed:
- `hash_md5` (ID 2890)
- `hash_sha1` (ID 2891)
- `hash_sha256` (ID 2892) — already exists as sha256_new + update + finalize
- `hash_sha512` (ID 2893)
- `hash_hmac_sha256` (ID 2894)
- `hash_file_md5` (ID 2895)
- `hash_file_sha256` (ID 2896)

**Commit after Phase 10.**

---

## Phase 11: Update Prelude & Integration

### Update prelude.at
Add all new modules to the standard prelude so they're auto-imported:
- `auto.cmp: Ordering, Cmp`
- `auto.clone: Clone`
- `auto.default: Default`
- `auto.fmt: Display, Debug`
- `auto.result: Result`
- `auto.error: Error`
- `auto.sort: sort, sorted, reverse`
- `auto.random: random, rand_int, rand_float`

### Add VM function registrations
Register all new VM native functions in `crates/auto-lang/src/vm/ffi/stdlib.rs`

### Add Rust FFI implementations
Implement all #[vm] functions using `#[rust_fn]` macro pattern

**Commit after Phase 11.**

---

## Implementation Order

1. Phase 1: Core Specs (cmp, clone, default, fmt)
2. Phase 2: Result<T,E> + Error
3. Phase 3: sort
4. Phase 4: random
5. Phase 5: time/duration/datetime
6. Phase 6: encoding (base64, hex, csv)
7. Phase 7: fs (enhanced filesystem)
8. Phase 8: test framework
9. Phase 9: fmt (formatted output)
10. Phase 10: hash (crypto)
11. Phase 11: prelude update + integration

Each phase: write .at files → add VM FFI → build → commit.

## Module Map (Auto API Reference)

| Module | Import | Key Types/Functions |
|--------|--------|---------------------|
| `cmp` | `use cmp` | `Ordering`, `Cmp` spec |
| `clone` | `use clone` | `Clone` spec |
| `default` | `use default` | `Default` spec |
| `fmt` | `use fmt` | `Display`, `Debug` specs |
| `result` | `use result` | `Result<T,E>`, `Ok()`, `Err()` |
| `error` | `use error` | `Error` type |
| `sort` | `use sort` | `sort()`, `sorted()`, `reverse()` |
| `random` | `use random` | `Random` type, `rand_int()`, `rand_float()` |
| `duration` | `use duration` | `Duration` type, `dur()` |
| `datetime` | `use datetime` | `DateTime` type |
| `encoding.base64` | `use encoding.base64` | `encode()`, `decode()` |
| `encoding.hex` | `use encoding.hex` | `encode()`, `decode()` |
| `encoding.csv` | `use encoding.csv` | `csv_parse()`, `csv_encode()` |
| `fs` | `use fs` | `walk()`, `metadata()`, `temp_dir()` |
| `test` | `use test` | `assert_eq()`, `assert_true()` |
| `format` | `use format` | `format()`, `sprintf()`, `printf()` |
| `hash` | `use hash` | `sha256()`, `md5()`, `hmac_sha256()` |
