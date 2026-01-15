# AutoLang String Library Reference

Complete reference for all string operations in AutoLang.

## Table of Contents

- [Basic String Operations](#basic-string-operations)
- [Search Operations](#search-operations)
- [Transform Operations](#transform-operations)
- [Split and Join](#split-and-join)
- [Comparison Operations](#comparison-operations)
- [Utility Operations](#utility-operations)
- [String Slice Operations](#string-slice-operations)
- [C FFI Operations](#c-ffi-operations)

---

## Basic String Operations

### str_new

Create a new owned string.

```auto
let s = str_new("hello")
```

**Parameters:**
- `text` (str): Initial text for the string

**Returns:** A new owned string value

### str_len

Get the length of a string in bytes.

```auto
let s = str_new("hello")
let len = str_len(s)  // 5
```

**Parameters:**
- `s` (str): The string to measure

**Returns:** Integer length in bytes

**Note:** For UTF-8 strings, this returns byte length, not character count. Use `str_char_len` for character count (not yet implemented).

### str_append

Append text to a string.

```auto
let mut s = str_new("hello")
str_append(mut s, " world")
print(s)  // "hello world"
```

**Parameters:**
- `s` (mut str): The string to append to (must be mutable)
- `text` (str): Text to append

**Returns:** Void

### str_upper

Convert a string to uppercase.

```auto
let s = str_new("hello")
let upper = str_upper(s)  // "HELLO"
```

**Parameters:**
- `s` (str): The string to convert

**Returns:** New uppercase string

### str_lower

Convert a string to lowercase.

```auto
let s = str_new("HELLO")
let lower = str_lower(s)  // "hello"
```

**Parameters:**
- `s` (str): The string to convert

**Returns:** New lowercase string

### str_sub

Extract a substring.

```auto
let s = str_new("hello world")
let sub = str_sub(s, 0, 5)  // "hello"
```

**Parameters:**
- `s` (str): The string to extract from
- `start` (int): Starting index (inclusive)
- `end` (int): Ending index (exclusive)

**Returns:** New substring

---

## Search Operations

### str_contains

Check if a string contains a substring.

```auto
let s = str_new("hello world")
let found = str_contains(s, "world")  // true
let not_found = str_contains(s, "goodbye")  // false
```

**Parameters:**
- `s` (str): The string to search in
- `pattern` (str): The substring to search for

**Returns:** Boolean indicating if pattern was found

### str_starts_with

Check if a string starts with a prefix.

```auto
let s = str_new("hello world")
let yes = str_starts_with(s, "hello")  // true
let no = str_starts_with(s, "world")  // false
```

**Parameters:**
- `s` (str): The string to check
- `prefix` (str): The prefix to look for

**Returns:** Boolean

### str_ends_with

Check if a string ends with a suffix.

```auto
let s = str_new("hello world")
let yes = str_ends_with(s, "world")  // true
let no = str_ends_with(s, "hello")  // false
```

**Parameters:**
- `s` (str): The string to check
- `suffix` (str): The suffix to look for

**Returns:** Boolean

### str_find

Find the index of a substring.

```auto
let s = str_new("hello world")
let index = str_find(s, "world")  // 6
let not_found = str_find(s, "goodbye")  // -1
```

**Parameters:**
- `s` (str): The string to search in
- `pattern` (str): The substring to find

**Returns:** Integer index (0-based), or -1 if not found

---

## Transform Operations

### str_trim

Remove whitespace from both ends of a string.

```auto
let s = str_new("  hello world  ")
let trimmed = str_trim(s)  // "hello world"
```

**Parameters:**
- `s` (str): The string to trim

**Returns:** New trimmed string

### str_trim_left

Remove whitespace from the left side of a string.

```auto
let s = str_new("  hello")
let trimmed = str_trim_left(s)  // "hello"
```

**Parameters:**
- `s` (str): The string to trim

**Returns:** New trimmed string

### str_trim_right

Remove whitespace from the right side of a string.

```auto
let s = str_new("hello  ")
let trimmed = str_trim_right(s)  // "hello"
```

**Parameters:**
- `s` (str): The string to trim

**Returns:** New trimmed string

### str_replace

Replace all occurrences of a substring.

```auto
let s = str_new("hello world")
let replaced = str_replace(s, "world", "AutoLang")  // "hello AutoLang"
```

**Parameters:**
- `s` (str): The string to modify
- `from` (str): The substring to replace
- `to` (str): The replacement text

**Returns:** New string with replacements

**Note:** Replaces ALL occurrences, not just the first.

---

## Split and Join

### str_split

Split a string by a delimiter.

```auto
let s = str_new("hello,world,auto")
let parts = str_split(s, ",")
// parts is now: ["hello", "world", "auto"]
```

**Parameters:**
- `s` (str): The string to split
- `delimiter` (str): The delimiter to split on

**Returns:** Array of strings

### str_join

Join an array of strings with a delimiter.

```auto
let parts = ["hello", "world", "auto"]
let joined = str_join(parts, ",")  // "hello,world,auto"
```

**Parameters:**
- `parts` (array[str]): Array of strings to join
- `delimiter` (str): Delimiter to insert between parts

**Returns:** New joined string

---

## Comparison Operations

### str_compare

Compare two strings lexicographically.

```auto
let less = str_compare("apple", "banana")  // -1
let equal = str_compare("hello", "hello")  // 0
let greater = str_compare("zebra", "apple")  // 1
```

**Parameters:**
- `s1` (str): First string
- `s2` (str): Second string

**Returns:** Integer:
- `-1` if s1 < s2
- `0` if s1 == s2
- `1` if s1 > s2

### str_eq_ignore_case

Compare two strings ignoring case.

```auto
let same = str_eq_ignore_case("HELLO", "hello")  // true
let different = str_eq_ignore_case("hello", "world")  // false
```

**Parameters:**
- `s1` (str): First string
- `s2` (str): Second string

**Returns:** Boolean

---

## Utility Operations

### str_repeat

Repeat a string multiple times.

```auto
let s = str_new("ha")
let repeated = str_repeat(s, 3)  // "hahaha"
```

**Parameters:**
- `s` (str): The string to repeat
- `n` (int): Number of times to repeat

**Returns:** New repeated string

### str_char_at

Get the character at a specific index.

```auto
let s = str_new("hello")
let char = str_char_at(s, 1)  // "e"
```

**Parameters:**
- `s` (str): The string to get from
- `index` (int): Character index (0-based)

**Returns:** Single-character string

**Note:** Returns error if index is out of bounds.

---

## String Slice Operations

### as_slice

Create a borrowed string slice.

```auto
let s = str_new("hello")
let slice = as_slice(s)
```

**Parameters:**
- `s` (str): The string to slice

**Returns:** String slice value

**Safety:** Slice borrows the original string. Do not use after the original string is dropped.

### slice_len

Get the length of a string slice.

```auto
let s = str_new("hello")
let slice = as_slice(s)
let len = slice_len(slice)  // 5
```

**Parameters:**
- `slice` (str_slice): The slice to measure

**Returns:** Integer length

### slice_get

Get a character from a slice by index.

```auto
let s = str_new("hello")
let slice = as_slice(s)
let char = slice_get(slice, 0)  // "h"
```

**Parameters:**
- `slice` (str_slice): The slice to get from
- `index` (int): Character index

**Returns:** Single-character string

---

## C FFI Operations

### cstr_new

Create a C-compatible null-terminated string.

```auto
let cs = cstr_new("hello")
```

**Parameters:**
- `s` (str): The string to convert

**Returns:** C string value (null-terminated)

**Use Case:** For passing strings to C functions via FFI.

### cstr_len

Get the length of a C string (excluding null terminator).

```auto
let cs = cstr_new("hello")
let len = cstr_len(cs)  // 5
```

**Parameters:**
- `cs` (cstr): The C string

**Returns:** Integer length

### cstr_as_ptr

Get the pointer address of a C string (for FFI).

```auto
let cs = cstr_new("hello")
let ptr = cstr_as_ptr(cs)
// ptr can be passed to C functions expecting char*
```

**Parameters:**
- `cs` (cstr): The C string

**Returns:** Pointer address as integer

**Note:** The pointer is only valid as long as the CStr exists.

### cstr_to_str

Convert a C string to a regular string.

```auto
let cs = cstr_new("hello")
let s = cstr_to_str(cs)  // "hello"
```

**Parameters:**
- `cs` (cstr): The C string to convert

**Returns:** Regular string value

### to_cstr

Convert a regular string to a C string (alias for cstr_new).

```auto
let s = str_new("hello")
let cs = to_cstr(s)
```

**Parameters:**
- `s` (str): The string to convert

**Returns:** C string value

---

## Examples

### Example 1: Building and Manipulating Strings

```auto
fn main() {
    // Create a new string
    let mut s = str_new("hello")

    // Append to it
    str_append(mut s, " world")

    // Transform it
    let upper = str_upper(s)  // "HELLO WORLD"
    let trimmed = str_trim(upper)  // Same, no whitespace

    print(trimmed)
}
```

### Example 2: Searching and Splitting

```auto
fn main() {
    let text = str_new("one,two,three,four")

    // Check if it contains a substring
    if str_contains(text, "three") {
        print("Found 'three'!")
    }

    // Split by delimiter
    let parts = str_split(text, ",")

    // Process each part
    for i in 0..3 {
        print(parts[i])
    }
}
```

### Example 3: String Replacement

```auto
fn main() {
    let template = str_new("Hello {name}, welcome to {app}!")

    // Replace placeholders
    let result = str_replace(template, "{name}", "Alice")
    result = str_replace(result, "{app}", "AutoLang")

    print(result)  // "Hello Alice, welcome to AutoLang!"
}
```

### Example 4: C FFI Example

```auto
// Declare external C function
extern fn.c printf(format cstr)

fn main() {
    let message = str_new("Hello from AutoLang!")

    // Convert to C string and pass to C function
    let cmessage = to_cstr(message)
    printf(cmessage)
}
```

---

## Performance Notes

1. **String Operations Create New Strings**: Most operations (upper, lower, trim, replace, etc.) return new strings rather than modifying in place.

2. **Use Slices for Temporary Views**: If you only need to read a portion of a string, use `as_slice()` to avoid copying.

3. **C FFI Has Overhead**: Converting between regular strings and C strings requires copying and null-termination. Minimize conversions in performance-critical code.

4. **UTF-8 Aware**: All operations work with UTF-8 encoding. String length is in bytes, not characters.

---

## See Also

- [AutoLang Language Guide](../README.md)
- [FFI Guide](./ffi.md)
- [Standard Library](./stdlib.md)
