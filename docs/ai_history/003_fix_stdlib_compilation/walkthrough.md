# Walkthrough: Fixing Stdlib Compilation for A2C

This walkthrough documents the steps taken to resolve compilation errors in the AutoLang standard library when transpiling to C using the `a2c-stdlib` command.

## Summary of Changes

We addressed several categories of errors:
1.  **Unsupported Types**: Replaced `byte`/`u8` with `char` as the C transpiler does not verify support for `byte`.
2.  **Missing Imports & Declarations**: Added `use` statements and `type.c` forward declarations to multiple files.
3.  **Visibility Issues**: Added `#[pub]` to standard library methods to ensure they are accessible across modules.
4.  **Legacy/Code Separation**: Split `io.at` into interface and implementation (`io.c.at`), and disabled legacy files overlapping with newer implementations.

## Key File Changes

### 1. `stdlib/auto/builder.at`
- Replaced `May<bool>` return types with `void` (implicit) to avoid transpilation issues with generics in C.
- Added `#[pub, c, vm]` annotations.

```diff
-    fn append(str str) May<bool>
+    #[c, vm, pub]
+    fn append(str str)
```

### 2. `stdlib/auto/data/list.at` & `stdlib/auto/dstr.at`
- Replaced `byte`/`u8` with `char`.
- Added `type.c` declarations for `List` to fix forward reference issues.

```diff
-    fn push(elem byte)
+    #[c, vm, pub]
+    fn push(elem char)
```

### 3. `stdlib/auto/io.at` & `stdlib/auto/io.c.at`
- Separated `File` type definition across `io.at` (interface) and `io.c.at` (implementation + C fields).
- **Critical Fix**: Removed incorrect `type.c File` from `io.c.at` because `File` is already defined as an auto type in `io.at` and the two files are loaded together.
- Configured compiler to generate `io.c.c` and `io.c.h` from `io.c.at`.

### 4. Cleanup
- Disabled legacy files:
    - `stdlib/may/may.at` -> `may.at.skip`
    - `stdlib/result/option.at` -> `option.at.skip`
    - `stdlib/result/result.at` -> `result.at.skip`
    - `stdlib/may/test_may.at` -> `test_may.at.skip`
    - `stdlib/result/test_option_result.at` -> `test_option_result.at.skip`

## Verification Results

The `a2c-stdlib` command now runs successfully without errors.

```
Transpiling stdlib...
Transpiling stdlib\auto\builder.at ...
...
Transpiling stdlib\collections\hashmap.c.at ...
(Success, no errors)
```
