# Stdlib Test Coverage to 80%+ Implementation Plan

> **Status: ✅ COMPLETE** (51 VM FFI tests + 17 a2r stdlib tests, all passing, verified 2026-04-23)
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Raise Rust stdlib test coverage from ~7.5% to 80%+ by adding VM tests and a2r tests for all stdlib modules.

**Architecture:** Two parallel test tracks: (1) **VM tests** in `test/vm/18_ffi/` exercise FFI functions via the AutoVM runtime; (2) **a2r tests** in `test/a2r/17_rust_std/` verify `use.rust` imports transpile correctly. Each test is a small `.at` file with `.expected.out` (stdout) or `.expected.result` (return value).

**Tech Stack:** Rust `#[test]` framework, AutoVM `run_with_capture`, a2r transpiler

---

## Current State

### VM tests (`test/vm/18_ffi/`): 14 tests, next number = `019`
Existing: file_exists, file_is_dir, string_len/isEmpty/contains/starts_with/ends_with, char_is_alpha/is_digit, json_is_valid/len/is_null/as_bool/has_key

### VM tests (`test/vm/19_rust_std/`): 9 tests, next number = `010`
Existing: time, duration, pathbuf, duration_print, instant_duration, sync, box_cell, duration_f64

### a2r tests (`test/a2r/17_rust_std/`): 10 tests, next number = `011`
Existing: collections, fs, sync, time, path, box_cell, env_process, thread, serde_json, regex

### Test registration files
- VM: `crates/auto-lang/src/tests/vm_file_tests.rs` — append after line 319 (18_ffi) and line 330 (19_rust_std)
- a2r: `crates/auto-lang/src/tests/a2r_tests.rs` — append after line 254 (18_rust_std section)

### How tests work
- **VM test**: `.at` file is executed by AutoVM. Last expression → `.expected.result`. `print()` output → `.expected.out`. Runtime error → `.expected.error`.
- **a2r test**: `.at` file is transpiled to Rust via `transpile_rust()`. Output compared to `.expected.rs`.
- **Naming**: `test/vm/18_ffi/019_name/name.at` + `name.expected.out` or `name.expected.result`
- **Registration**: `#[test] fn test_18_ffi_019_name() { test_vm("18_ffi/019_name").unwrap(); }`

---

## Task 1: Math VM tests (20 functions, 0 tested → 80%+)

**Files to create** in `crates/auto-lang/test/vm/18_ffi/`:

### 019_math_abs
```
019_math_abs/math.at:
print(Math.abs(-42))
```
```
019_math_abs/math.expected.out:
42
```

### 020_math_min_max
```
020_math_min_max/math_min_max.at:
print(Math.min(3, 7))
print(Math.max(3, 7))
```
```
020_math_min_max/math_min_max.expected.out:
3
7
```

### 021_math_sqrt
```
021_math_sqrt/math_sqrt.at:
print(Math.sqrt(16.0))
```
```
021_math_sqrt/math_sqrt.expected.out:
4
```

### 022_math_floor_ceil_round
```
022_math_floor_ceil_round/math_floor_ceil_round.at:
print(Math.floor(3.7))
print(Math.ceil(3.2))
print(Math.round(3.5))
```
```
022_math_floor_ceil_round/math_floor_ceil_round.expected.out:
3
4
4
```

### 023_math_pow
```
023_math_pow/math_pow.at:
print(Math.pow(2.0, 10.0))
```
```
023_math_pow/math_pow.expected.out:
1024
```

### 024_math_trig
```
024_math_trig/math_trig.at:
print(Math.sin(0.0))
print(Math.cos(0.0))
```
```
024_math_trig/math_trig.expected.out:
0
1
```

### 025_math_log
```
025_math_log/math_log.at:
print(Math.ln(1.0))
print(Math.log2(8.0))
print(Math.log10(100.0))
```
```
025_math_log/math_log.expected.out:
0
3
2
```

### 026_math_signum_clamp
```
026_math_signum_clamp/math_signum_clamp.at:
print(Math.signum(-5.0))
print(Math.signum(0.0))
print(Math.signum(5.0))
print(Math.clamp(15.0, 0.0, 10.0))
```
```
026_math_signum_clamp/math_signum_clamp.expected.out:
-1
0
1
10
```

### 027_math_abs_f_min_f_max_f
```
027_math_abs_f_min_f_max_f/math_abs_f_min_f_max_f.at:
print(Math.abs_f(-3.14))
print(Math.min_f(1.5, 2.5))
print(Math.max_f(1.5, 2.5))
```
```
027_math_abs_f_min_f_max_f/math_abs_f_min_f_max_f.expected.out:
3.14
1.5
2.5
```

**Step 1:** Create all 9 test directories with `.at` and `.expected.out` files.

**Step 2:** Register in `vm_file_tests.rs` after line 319:
```rust
// === 18_ffi (Math) ===
#[test] fn test_18_ffi_019_math_abs() { test_vm("18_ffi/019_math_abs").unwrap(); }
#[test] fn test_18_ffi_020_math_min_max() { test_vm("18_ffi/020_math_min_max").unwrap(); }
#[test] fn test_18_ffi_021_math_sqrt() { test_vm("18_ffi/021_math_sqrt").unwrap(); }
#[test] fn test_18_ffi_022_math_floor_ceil_round() { test_vm("18_ffi/022_math_floor_ceil_round").unwrap(); }
#[test] fn test_18_ffi_023_math_pow() { test_vm("18_ffi/023_math_pow").unwrap(); }
#[test] fn test_18_ffi_024_math_trig() { test_vm("18_ffi/024_math_trig").unwrap(); }
#[test] fn test_18_ffi_025_math_log() { test_vm("18_ffi/025_math_log").unwrap(); }
#[test] fn test_18_ffi_026_math_signum_clamp() { test_vm("18_ffi/026_math_signum_clamp").unwrap(); }
#[test] fn test_18_ffi_027_math_abs_f_min_f_max_f() { test_vm("18_ffi/027_math_abs_f_min_f_max_f").unwrap(); }
```

**Step 3:** Run: `cargo test -p auto-lang -- 18_ffi_02`

Expected: All 9 new tests pass.

**Step 4:** Commit: `git commit -m "test: add Math stdlib VM tests (abs, min/max, sqrt, trig, log, clamp)"`

---

## Task 2: String VM tests (21 functions, 4 tested → 80%+)

**Next test number: `028`**

Create these tests in `crates/auto-lang/test/vm/18_ffi/`:

### 028_str_char_at
```
028_str_char_at/str_char_at.at:
print(Str.char_at("hello", 0))
print(Str.char_at("hello", 4))
print(Str.char_at("hello", 10))
```
```
028_str_char_at/str_char_at.expected.out:
h
o

```

### 029_str_substr
```
029_str_substr/str_substr.at:
print(Str.substr("hello world", 0, 5))
print(Str.substr("hello world", 6, 11))
```
```
029_str_substr/str_substr.expected.out:
hello
world
```

### 030_str_trim
```
030_str_trim/str_trim.at:
print(Str.trim("  hello  "))
```
```
030_str_trim/str_trim.expected.out:
hello
```

### 031_str_split
```
031_str_split/str_split.at:
let parts = Str.split("a,b,c", ",")
print(parts[0])
print(parts[1])
print(parts[2])
```
```
031_str_split/str_split.expected.out:
a
b
c
```

### 032_str_repeat
```
032_str_repeat/str_repeat.at:
print(Str.repeat("ab", 3))
```
```
032_str_repeat/str_repeat.expected.out:
ababab
```

### 033_str_replace
```
033_str_replace/str_replace.at:
print(Str.replace("hello world", "world", "auto"))
print(Str.replace_first("aaa", "a", "b"))
```
```
033_str_replace/str_replace.expected.out:
hello auto
baa
```

### 034_str_case
```
034_str_case/str_case.at:
print(Str.to_upper("hello"))
print(Str.to_lower("HELLO"))
```
```
034_str_case/str_case.expected.out:
HELLO
hello
```

### 035_str_reverse_find
```
035_str_reverse_find/str_reverse_find.at:
print(Str.reverse("hello"))
print(Str.find("hello world", "world"))
print(Str.find("hello", "xyz"))
```
```
035_str_reverse_find/str_reverse_find.expected.out:
olleh
6
-1
```

### 036_str_lines
```
036_str_lines/str_lines.at:
let lines = Str.lines("line1\nline2\nline3")
print(lines[0])
print(lines[1])
print(lines[2])
```
```
036_str_lines/str_lines.expected.out:
line1
line2
line3
```

### 037_str_parse
```
037_str_parse/str_parse.at:
print(Str.parse_int("42"))
print(Str.parse_float("3.14"))
```
```
037_str_parse/str_parse.expected.out:
42
3.14
```

### 038_str_match_count
```
038_str_match_count/str_match_count.at:
print(Str.match_count("abcabc", "abc"))
```
```
038_str_match_count/str_match_count.expected.out:
2
```

### 039_str_split_once
```
039_str_split_once/str_split_once.at:
let parts = Str.split_once("key=value", "=")
print(parts[0])
print(parts[1])
```
```
039_str_split_once/str_split_once.expected.out:
key
value
```

**Register all 12 tests** in `vm_file_tests.rs` after the Math tests.

**Run:** `cargo test -p auto-lang -- 18_ffi_03`

**Commit:** `git commit -m "test: add String stdlib VM tests (char_at, substr, trim, split, replace, etc.)"`

---

## Task 3: Char VM tests (7 functions, 2 tested → 100%)

**Next test number: `040`**

### 040_char_is_alphanum
```
040_char_is_alphanum/char_is_alphanum.at:
print(Char.is_alphanum(65))
print(Char.is_alphanum(48))
print(Char.is_alphanum(32))
```
```
040_char_is_alphanum/char_is_alphanum.expected.out:
1
1
0
```

### 041_char_is_whitespace
```
041_char_is_whitespace/char_is_whitespace.at:
print(Char.is_whitespace(32))
print(Char.is_whitespace(9))
print(Char.is_whitespace(65))
```
```
041_char_is_whitespace/char_is_whitespace.expected.out:
1
1
0
```

### 042_char_is_ident
```
042_char_is_ident/char_is_ident.at:
print(Char.is_ident(95))
print(Char.is_ident(65))
print(Char.is_ident(32))
```
```
042_char_is_ident/char_is_ident.expected.out:
1
1
0
```

### 043_char_case
```
043_char_case/char_case.at:
print(Char.to_lower(65))
print(Char.to_upper(97))
```
```
043_char_case/char_case.expected.out:
97
65
```

**Register 4 tests, run, commit:** `git commit -m "test: add Char stdlib VM tests (is_alphanum, whitespace, ident, case)"`

---

## Task 4: JSON VM tests (15 functions, 4 tested → 80%+)

**Next test number: `044`**

### 044_json_encode_parse
```
044_json_encode_parse/json_encode_parse.at:
let encoded = Json.encode({"name": "auto"})
print(encoded)
let parsed = Json.parse("{\"x\":1}")
print(Json.type_of(parsed))
```
```
044_json_encode_parse/json_encode_parse.expected.out:
{"name":"auto"}
object
```

### 045_json_get
```
045_json_get/json_get.at:
let obj = Json.parse("{\"name\":\"auto\",\"ver\":1}")
print(Json.as_string(Json.get(obj, "name")))
print(Json.as_int(Json.get(obj, "ver")))
```
```
045_json_get/json_get.expected.out:
auto
1
```

### 046_json_array
```
046_json_array/json_array.at:
let arr = Json.parse("[10,20,30]")
print(Json.as_int(Json.get_at(arr, 0)))
print(Json.as_int(Json.get_at(arr, 2)))
print(Json.len(arr))
```
```
046_json_array/json_array.expected.out:
10
30
3
```

### 047_json_keys
```
047_json_keys/json_keys.at:
let obj = Json.parse("{\"a\":1,\"b\":2}")
let keys = Json.keys(obj)
print(keys[0])
print(keys[1])
```
```
047_json_keys/json_keys.expected.out:
a
b
```

### 048_json_type_as
```
048_json_type_as/json_type_as.at:
let s = Json.parse("\"hello\"")
print(Json.type_of(s))
print(Json.as_string(s))
let n = Json.parse("42")
print(Json.type_of(n))
print(Json.as_number(n))
```
```
048_json_type_as/json_type_as.expected.out:
string
hello
number
42
```

**Register 5 tests, run, commit:** `git commit -m "test: add JSON stdlib VM tests (encode, parse, get, array, keys, type)"`

---

## Task 5: Path VM tests (5 functions, 0 tested → 100%)

**Next test number: `049`**

### 049_path_join
```
049_path_join/path_join.at:
print(Path.join("usr", "local", "bin"))
```
```
049_path_join/path_join.expected.out:
usr/local/bin
```

### 050_path_parent
```
050_path_parent/path_parent.at:
print(Path.parent("/usr/local/bin"))
print(Path.parent("file.txt"))
```
```
050_path_parent/path_parent.expected.out:
/usr/local

```

### 051_path_ext_filename
```
051_path_ext_filename/path_ext_filename.at:
print(Path.extension("file.tar.gz"))
print(Path.filename("/usr/local/file.txt"))
```
```
051_path_ext_filename/path_ext_filename.expected.out:
gz
file.txt
```

### 052_path_canonicalize
```
052_path_canonicalize/path_canonicalize.at:
print(Path.canonicalize("crates"))
```
```
052_path_canonicalize/path_canonicalize.expected.out:
```
Note: `.expected.result` is better here since canonicalize returns an absolute path that varies:
```
052_path_canonicalize/path_canonicalize.expected.result:
```
Use `.expected.result` pattern — just verify it doesn't error. Actually, use `.expected.out` with a flexible check. Better approach: just test that it returns a non-empty string by using `print(Str.len(Path.canonicalize("crates")))` but that's fragile. Instead, skip canonicalize in VM tests and only test in a2r.

**Revised 052:**
```
052_path_canonicalize/path_canonicalize.at:
let p = Path.canonicalize("crates")
print(Str.len(p) > 0)
```
```
052_path_canonicalize/path_canonicalize.expected.out:
1
```

**Register 4 tests, run, commit:** `git commit -m "test: add Path stdlib VM tests (join, parent, extension, filename)"`

---

## Task 6: Env VM tests (3 functions, 0 tested → 100%)

**Next test number: `053`**

### 053_env_get_set
```
053_env_get_set/env_get_set.at:
Env.set("AUTO_TEST_VAR", "hello")
let val = Env.get("AUTO_TEST_VAR")
print(val)
Env.remove("AUTO_TEST_VAR")
let gone = Env.get("AUTO_TEST_VAR")
print(gone)
```
```
053_env_get_set/env_get_set.expected.out:
hello

```

**Register 1 test, run, commit:** `git commit -m "test: add Env stdlib VM tests (get, set, remove)"`

---

## Task 7: Time VM tests (3 functions, 0 tested → 100%)

**Next test number: `054`**

### 054_time_now
```
054_time_now/time_now.at:
let ms = Time.now_ms()
let sec = Time.now_sec()
print(ms > 0)
print(sec > 0)
```
```
054_time_now/time_now.expected.out:
1
1
```

Note: Skip `Time.sleep_ms` in automated tests (would slow down the suite).

**Register 1 test, run, commit:** `git commit -m "test: add Time stdlib VM tests (now_ms, now_sec)"`

---

## Task 8: URL VM tests (15+ functions, 0 tested → 80%+)

**Next test number: `055`**

### 055_url_encode_decode
```
055_url_encode_decode/url_encode_decode.at:
print(Url.encode("hello world"))
print(Url.decode("hello%20world"))
```
```
055_url_encode_decode/url_encode_decode.expected.out:
hello%20world
hello world
```

### 056_url_parse
```
056_url_parse/url_parse.at:
print(Url.scheme("https://example.com/path?q=1#frag"))
print(Url.host("https://example.com/path"))
print(Url.path("https://example.com/docs/api"))
print(Url.query("https://example.com/path?key=val"))
print(Url.fragment("https://example.com/path#section"))
```
```
056_url_parse/url_parse.expected.out:
https
example.com
/docs/api
key=val
section
```

### 057_url_port
```
057_url_port/url_port.at:
print(Url.port("https://example.com:8080/path"))
print(Url.port("https://example.com/path"))
```
```
057_url_port/url_port.expected.out:
8080
-1
```

**Register 3 tests, run, commit:** `git commit -m "test: add URL stdlib VM tests (encode, decode, parse, port)"`

---

## Task 9: Regex VM tests (2 functions, 0 tested → 100%)

**Next test number: `058`**

### 058_regex_match
```
058_regex_match/regex_match.at:
print(Regex.is_match("\\d+", "abc123"))
print(Regex.is_match("\\d+", "abcdef"))
```
```
058_regex_match/regex_match.expected.out:
1
0
```

### 059_regex_find_all
```
059_regex_find_all/regex_find_all.at:
let result = Regex.find_all("\\d+", "a1b22c333")
print(result)
```
```
059_regex_find_all/regex_find_all.expected.out:
[{"match":"1","start":1,"end":2},{"match":"22","start":3,"end":5},{"match":"333","start":6,"end":9}]
```

**Register 2 tests, run, commit:** `git commit -m "test: add Regex stdlib VM tests (is_match, find_all)"`

---

## Task 10: Log VM tests (4 functions, 0 tested → 75%)

**Next test number: `060`**

### 060_log_functions
```
060_log_functions/log_functions.at:
Log.debug("debug msg")
Log.info("info msg")
Log.warn("warn msg")
Log.error("error msg")
```

Note: Log writes to stdout (debug/info/warn) and stderr (error). Only stdout is captured by `run_with_capture`. Use `.expected.out`:
```
060_log_functions/log_functions.expected.out:
[DEBUG] debug msg
[INFO] info msg
[WARN] warn msg
```

Note: Log.error goes to stderr which is NOT captured by the current test framework. Only 3 of 4 functions are tested via stdout. This gives 75% coverage.

**Register 1 test, run, commit:** `git commit -m "test: add Log stdlib VM tests (debug, info, warn)"`

---

## Task 11: File VM tests (13 functions, 2 tested → 80%+)

**Next test number: `061`**

### 061_file_read_write_text
```
061_file_read_write_text/file_read_write_text.at:
File.write_text("tmp_test_vm.txt", "hello auto")
let content = File.read_text("tmp_test_vm.txt")
print(content)
File.delete("tmp_test_vm.txt")
```
```
061_file_read_write_text/file_read_write_text.expected.out:
hello auto
```

### 062_file_size_copy
```
062_file_size_copy/file_size_copy.at:
File.write_text("tmp_test_size.txt", "hello")
print(File.size("tmp_test_size.txt"))
File.copy("tmp_test_size.txt", "tmp_test_copy.txt")
let copied = File.read_text("tmp_test_copy.txt")
print(copied)
File.delete("tmp_test_size.txt")
File.delete("tmp_test_copy.txt")
```
```
062_file_size_copy/file_size_copy.expected.out:
5
hello
```

### 063_file_read_lines
```
063_file_read_lines/file_read_lines.at:
File.write_text("tmp_test_lines.txt", "line1\nline2\nline3")
let lines = File.read_lines("tmp_test_lines.txt")
print(lines)
File.delete("tmp_test_lines.txt")
```
```
063_file_read_lines/file_read_lines.expected.out:
["line1","line2","line3"]
```

### 064_file_create_dir_walk
```
064_file_create_dir_walk/file_create_dir_walk.at:
File.create_dir("tmp_test_dir")
print(File.is_dir("tmp_test_dir"))
File.delete("tmp_test_dir")
```
```
064_file_create_dir_walk/file_create_dir_walk.expected.out:
1
```

### 065_file_append
```
065_file_append/file_append.at:
File.write_text("tmp_test_append.txt", "hello")
File.append_text("tmp_test_append.txt", " world")
let content = File.read_text("tmp_test_append.txt")
print(content)
File.delete("tmp_test_append.txt")
```
```
065_file_append/file_append.expected.out:
hello world
```

**Register 5 tests, run, commit:** `git commit -m "test: add File stdlib VM tests (read/write, size, copy, lines, append)"`

---

## Task 12: Process VM tests (6 functions, 0 tested → 50%)

**Next test number: `066`**

### 066_process_cwd
```
066_process_cwd/process_cwd.at:
let cwd = Process.current_dir()
print(Str.len(cwd) > 0)
```
```
066_process_cwd/process_cwd.expected.out:
1
```

### 067_process_args
```
067_process_args/process_args.at:
let args = Process.args()
print(Str.len(args[0]) > 0)
```
```
067_process_args/process_args.expected.out:
1
```

Note: Skip `Process.exit` (kills the test runner), `Process.spawn`/`spawn_with_output` (environment-dependent), `Process.set_current_dir` (side effects).

**Register 2 tests, run, commit:** `git commit -m "test: add Process stdlib VM tests (current_dir, args)"`

---

## Task 13: a2r stdlib tests — expand `17_rust_std/` (10 → 20+ tests)

**Next test number: `011`**

These tests verify `use.rust` imports transpile correctly. Pattern: create `.at` with `use.rust` imports, run `transpile_rust()`, compare to `.expected.rs`.

### 011_math
```auto
use.rust std::math::{abs, min, max}

fn main() {
    let x = abs(-5)
    let y = min(3, 7)
    let z = max(3, 7)
}
```

### 012_string
```auto
use.rust std::string::String

fn main() {
    let s = "hello world"
    let upper = s.to_uppercase()
    let lower = s.to_lowercase()
    let trimmed = s.trim()
}
```

### 013_result
```auto
use.rust std::result::Result

fn divide(a int, b int) Result<int, str> {
    if b == 0 {
        return Err("division by zero")
    }
    return Ok(a / b)
}
```

### 014_option
```auto
use.rust std::option::Option

fn find_user(id int) Option<str> {
    if id == 1 {
        return Some("alice")
    }
    return None
}
```

### 015_vec
```auto
use.rust std::vec::Vec

fn main() {
    let mut v Vec<int> = Vec.new()
    v.push(1)
    v.push(2)
    v.push(3)
    let first = v[0]
}
```

### 016_iter
```auto
use.rust std::iter::Iterator

fn main() {
    let nums = [1, 2, 3, 4, 5]
    let sum = nums.iter().sum()
}
```

### 017_fmt
```auto
use.rust std::fmt::Display

fn main() {
    let s = format!("hello {}", "world")
}
```

### 018_os
```auto
use.rust std::os

fn main() {
    let os_str = std::os::getenv("PATH")
}
```

### 019_net
```auto
use.rust std::net::TcpListener

fn main() {
    let listener = TcpListener.bind("127.0.0.1:0")
}
```

### 020_process
```auto
use.rust std::process::Command

fn main() {
    let output = Command.new("echo").arg("hello").output()
}
```

For each: create directory, write `.at`, run `cargo test -p auto-lang test_18_rust_std_0XX --nocapture` to get `.wrong.rs`, review, rename to `.expected.rs`.

**Register in `a2r_tests.rs` after line 254:**
```rust
// === 18_rust_std (expanded) ===
#[test] fn test_18_rust_std_011_math() { test_a2r("17_rust_std/011_math").unwrap(); }
#[test] fn test_18_rust_std_012_string() { test_a2r("17_rust_std/012_string").unwrap(); }
#[test] fn test_18_rust_std_013_result() { test_a2r("17_rust_std/013_result").unwrap(); }
#[test] fn test_18_rust_std_014_option() { test_a2r("17_rust_std/014_option").unwrap(); }
#[test] fn test_18_rust_std_015_vec() { test_a2r("17_rust_std/015_vec").unwrap(); }
#[test] fn test_18_rust_std_016_iter() { test_a2r("17_rust_std/016_iter").unwrap(); }
#[test] fn test_18_rust_std_017_fmt() { test_a2r("17_rust_std/017_fmt").unwrap(); }
#[test] fn test_18_rust_std_018_os() { test_a2r("17_rust_std/018_os").unwrap(); }
#[test] fn test_18_rust_std_019_net() { test_a2r("17_rust_std/019_net").unwrap(); }
#[test] fn test_18_rust_std_020_process() { test_a2r("17_rust_std/020_process").unwrap(); }
```

**Commit:** `git commit -m "test: add a2r stdlib tests (math, string, result, option, vec, iter, fmt, net, process)"`

---

## Task 14: Run full test suite and fix failures

**Step 1:** `cargo test -p auto-lang 2>&1 | tail -30`

**Step 2:** Fix any failures:
- Expected output format mismatches (e.g., float formatting `4` vs `4.0`)
- Path separator differences on Windows (`\` vs `/`)
- Tests that write files may fail if `tmp/` doesn't exist — create `tmp/` first
- Log output format may differ from expected

**Step 3:** Re-run until all pass.

**Commit:** `git commit -m "fix: adjust stdlib test expected outputs"`

---

## Summary: Coverage After This Plan

| Module | Before | After (VM) | After (a2r) | Coverage |
|--------|--------|------------|-------------|----------|
| Math | 0% | 90% | +a2r | ~95% |
| String | 19% | 95% | +a2r | ~95% |
| Char | 29% | 100% | — | 100% |
| JSON | 27% | 80% | +a2r | ~85% |
| Path | 0% | 80% | — | ~80% |
| Env | 0% | 100% | — | 100% |
| Time | 0% | 67% | — | ~70% |
| URL | 0% | 80% | — | ~80% |
| Regex | 0% | 100% | +a2r | 100% |
| Log | 0% | 75% | — | 75% |
| File | 15% | 77% | — | ~80% |
| Process | 0% | 33% | +a2r | ~50% |
| **Overall FFI** | **7.5%** | **~80%** | **+20 tests** | **~82%** |

**Total new tests: ~47 VM tests + 10 a2r tests = 57 new tests**

### Modules deliberately under-tested
- **Net/HTTP**: Requires network or mock infrastructure — skip for now
- **Task/Msg**: Requires async runtime setup — skip for now
- **Process.spawn/exit**: Environment-dependent or dangerous — skip
- **Time.sleep_ms**: Slows test suite — skip
- **Log.error**: Writes to stderr, not captured by test harness — skip

These can be addressed in a follow-up plan with proper test infrastructure (mock servers, async test harness).
