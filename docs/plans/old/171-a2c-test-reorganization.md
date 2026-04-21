# Plan 171: A2C Test Suite Reorganization

## Objective

Reorganize the a2c (Auto-to-C) transpiler test suite from chaotic sequential numbering (239 directories, many orphaned) into a categorized, numbered directory structure aligned with the a2r test suite (Plan 170), plus a2c-specific categories for C-only features.

## Status: COMPLETE

All tasks implemented and verified. 106 a2c tests passing, 14 ignored (unsupported features), 0 wrong files.

## Current State (Before)

- 239 directories with sequential numbering (000-152, many number collisions)
- 125 directories with `.at` source files (active tests)
- 88 directories with no `.at` file (orphaned — leftovers from renumbering)
- 26 directories with no `.expected.c`/`.expected.h` (stubs, never had expected output)
- Number collisions common (e.g., `080_array_index_read` + `080_array_slice` + `080_question_char`)
- Stale `.wrong.c`/`.wrong.h` files in 96 directories
- No clear grouping by feature category

## Final State (After)

- ~90 test cases across 20 categorized directories (25 categories defined, 20 non-empty)
- 0 orphaned directories
- 0 number conflicts
- 0 stale `.wrong` files
- 0 may_* tests (replaced by Option/Result)
- ~21 redundant enum smoke tests removed (keep 3 representatives)
- ~6 duplicate question_* tests removed
- Rewritten test runner with auto-discovery

## Design

### Directory Structure

Category-based directories with sequence numbers, test cases numbered within each category:

```
test/a2c/
  01_basics/           # Hello world, functions, fundamentals
  02_types/            # Struct, enum, union, pointer, inheritance
  03_control_flow/     # if, for, is matching
  04_strings/          # f-strings, split
  05_expressions/      # Arithmetic, bang operator, complex expr
  06_pattern_matching/ # Hetero enums, pattern matching
  07_ownership/        # Borrow view/mut/move, conflicts
  08_generics/         # Type aliases, const generics, constraints
  09_option_result/    # Null coalesce, error propagation, closure
  10_collections/      # Array, list storage, list iter
  11_methods/          # Instance methods, closures
  12_specs/            # Spec (trait) declarations
  13_delegation/       # Delegation with specs
  14_modules/          # (empty — a2c has no module tests)
  15_type_conversion/  # Type casts
  16_interop/          # (empty — a2c has no async/derive)
  17_autocode/         # (empty — a2c has no autocode tests)
  18_c_interop/        # C-specific: cstr, alias, unified section
  19_option_type/      # ?T (Option<T>) return types
  20_result_type/      # !T (Result<T>) — placeholder, currently 0 tests
  21_storage/          # Storage strategies (Heap/InlineInt64)
  22_iterators/        # Iterator specs, adapters, terminal ops
  23_stdlib/           # File I/O, char I/O, process, repl
  24_runtime_size/     # Runtime-sized arrays
  25_type_checking/    # Type error detection
```

Each test case: `NN_name/name.at` + `name.expected.c` + `name.expected.h`

### Alignment with a2r (Plan 170)

Categories 01-17 use identical numbering and naming as a2r. Categories 18-25 are a2c-specific.

| # | Category | a2r Tests | a2c Tests | Notes |
|---|----------|-----------|-----------|-------|
| 01 | basics | 4 | 3 | hello, sqrt, func |
| 02 | types | 10 | 6 | struct, enum, union, pointer, cstr, inheritance |
| 03 | control_flow | 10 | 4 | if, for, is, for_conditions |
| 04 | strings | 5 | 2 | str, str_split |
| 05 | expressions | 11 | 4 | complex_expr, field_access, bang_operator, binary |
| 06 | pattern_matching | 7 | 3 | hetero_enum, hetero_enum_verify, hetero_enum_types |
| 07 | ownership | 4 | 4 | borrow_view, borrow_mut, borrow_move, borrow_conflicts |
| 08 | generics | 7 | 7 | const_generics, generic_field, generic_ptr_field, with_constraint, generic_specs, generic_spec_ext, generic_type_alias, multi_param, generic_list |
| 09 | option_result | 32 | 3 | null_coalesce, error_propagate, closure |
| 10 | collections | 5 | 13 | array, array_return, array_declaration, array_mutation, array_index_read, array_copy, array_slice, array_nested, array_zero_size, array_loop, list_storage, list_iter, list_capacity |
| 11 | methods | 8 | 3 | method, multi_param, closure |
| 12 | specs | 3 | 2 | basic_spec, spec |
| 13 | delegation | 3 | 3 | delegation, multi_delegation, delegation_params |
| 14 | modules | 9 | 0 | (empty) |
| 15 | type_conversion | 4 | 1 | type_cast |
| 16 | interop | 3 | 0 | (empty) |
| 17 | autocode | 17 | 0 | (empty) |
| 18 | c_interop | — | 3 | cstr, alias, unified_section |
| 19 | option_type | — | ~15 | ?T return types |
| 20 | result_type | — | 0 | !T return types (placeholder) |
| 21 | storage | — | 3 | storage_module, storage_usage, plan055 |
| 22 | iterators | — | 7 | iter_specs, map_adapter, terminal_operators, extended_adapters, predicates, collect |
| 23 | stdlib | — | 14 | File I/O, char I/O, getpid, readline, repl, str |
| 24 | runtime_size | — | 2 | runtime_size_var, runtime_size_expr |
| 25 | type_checking | — | 1 | type_error |

### Cleanup Summary

| Action | Count | Details |
|--------|-------|---------|
| Delete orphaned dirs (no .at) | 88 | Old numbering leftovers |
| Delete may_* tests | 5 | 033, 034, 035, 037, 052 (replaced by Option/Result) |
| Delete redundant enum smoke tests | ~21 | Keep 046_mode, 041_tristate, 036_may_patterns |
| Delete duplicate question_* tests | 3 | 094_question_literal, 095_question_zero, 096_question_negative (identical) |
| Delete duplicate hetero enums | 2 | 022_typed_node_checking, 063_simple_hetero_enum (same as 060) |
| Clean stale .wrong files | 96 | Remove .wrong.c/.wrong.h from all migrated tests |
| Keep incomplete array tests | 10 | 097-105, 122 (need .expected.h generated) |
| Keep question_void/logical stubs | 2 | 081, 092 (need .expected.c/.h generated) |

### Categories and Test Mappings

#### 01_basics (3 tests)

| Old | New |
|-----|-----|
| 000_hello | 01_basics/001_hello |
| 001_sqrt | 01_basics/002_sqrt |
| 003_func | 01_basics/003_func |

#### 02_types (6 tests)

| Old | New |
|-----|-----|
| 006_struct | 02_types/001_struct |
| 007_enum | 02_types/002_enum |
| 013_union | 02_types/003_union |
| 005_pointer | 02_types/004_pointer |
| 128_inheritance | 02_types/005_inheritance |
| 108_pointer_types | 02_types/006_pointer_types |

#### 03_control_flow (4 tests)

| Old | New |
|-----|-----|
| 010_if | 03_control_flow/001_if_basic |
| 011_for | 03_control_flow/002_for_range |
| 012_is | 03_control_flow/003_is_match |
| 031_for_conditions | 03_control_flow/004_for_conditions |

#### 04_strings (2 tests)

| Old | New |
|-----|-----|
| 015_str | 04_strings/001_str |
| 030_str_split | 04_strings/002_str_split |

#### 05_expressions (4 tests)

| Old | New |
|-----|-----|
| 028_complex_expr | 05_expressions/001_complex_expr |
| 054_field_access | 05_expressions/002_field_access |
| 131_bang_operator | 05_expressions/003_bang_operator |
| 038_binary | 05_expressions/004_binary |

#### 06_pattern_matching (3 tests)

| Old | New |
|-----|-----|
| 014_hetero_enum | 06_pattern_matching/001_hetero_enum |
| 060_hetero_enum_verify | 06_pattern_matching/002_hetero_enum_verify |
| 032_hetero_enum_types | 06_pattern_matching/003_hetero_enum_types |

#### 07_ownership (4 tests)

| Old | New |
|-----|-----|
| 023_borrow_view | 07_ownership/001_borrow_view |
| 024_borrow_mut | 07_ownership/002_borrow_mut |
| 025_borrow_move | 07_ownership/003_borrow_move |
| 026_borrow_conflicts | 07_ownership/004_borrow_conflicts |

#### 08_generics (7 tests)

| Old | New |
|-----|-----|
| 110_const_generics | 08_generics/001_const_generics |
| 126_generic_field | 08_generics/002_generic_field |
| 127_generic_ptr_field | 08_generics/003_generic_ptr_field |
| 136_with_constraint | 08_generics/004_with_constraint |
| 112_generic_specs | 08_generics/005_generic_specs |
| 113_generic_spec_ext | 08_generics/006_generic_spec_ext |
| 111_generic_type_alias | 08_generics/007_generic_type_alias |

#### 09_option_result (3 tests)

| Old | New |
|-----|-----|
| 118_null_coalesce | 09_option_result/001_null_coalesce |
| 119_error_propagate | 09_option_result/002_error_propagate |
| 125_closure | 09_option_result/003_closure |

#### 10_collections (13 tests)

| Old | New |
|-----|-----|
| 002_array | 10_collections/001_array |
| 029_array_return | 10_collections/002_array_return |
| 097_array_declaration | 10_collections/003_array_declaration |
| 098_array_mutation | 10_collections/004_array_mutation |
| 099_array_index_read | 10_collections/005_array_index_read |
| 100_array_copy | 10_collections/006_array_copy |
| 101_array_slice | 10_collections/007_array_slice |
| 102_array_nested | 10_collections/008_array_nested |
| 103_array_zero_size | 10_collections/009_array_zero_size |
| 104_array_loop | 10_collections/010_array_loop |
| 117_list_storage | 10_collections/011_list_storage |
| 122_list_iter | 10_collections/012_list_iter |
| 055_list_capacity | 10_collections/013_list_capacity |

#### 11_methods (3 tests)

| Old | New |
|-----|-----|
| 008_method | 11_methods/001_method |
| 064_multi_param | 11_methods/002_multi_param |
| 066_generic_list | 11_methods/003_generic_list |

#### 12_specs (2 tests)

| Old | New |
|-----|-----|
| 016_basic_spec | 12_specs/001_basic_spec |
| 017_spec | 12_specs/002_spec |

#### 13_delegation (3 tests)

| Old | New |
|-----|-----|
| 018_delegation | 13_delegation/001_single |
| 019_multi_delegation | 13_delegation/002_multi_delegation |
| 020_delegation_params | 13_delegation/003_delegation_params |

#### 18_c_interop (3 tests)

| Old | New |
|-----|-----|
| 004_cstr | 18_c_interop/001_cstr |
| 009_alias | 18_c_interop/002_alias |
| 027_unified_section | 18_c_interop/003_unified_section |

#### 19_option_type (~15 tests)

All `?T` (Option) syntax tests:

| Old | New |
|-----|-----|
| 076_question_syntax | 19_option_type/001_question_syntax |
| 077_question_uint | 19_option_type/002_question_uint |
| 078_question_float | 19_option_type/003_question_float |
| 079_question_double | 19_option_type/004_question_double |
| 080_question_char | 19_option_type/005_question_char |
| 081_question_void | 19_option_type/006_question_void |
| 082_question_return_int | 19_option_type/007_question_return_int |
| 083_question_return_str | 19_option_type/008_question_return_str |
| 084_question_return_bool | 19_option_type/009_question_return_bool |
| 085_question_return_uint | 19_option_type/010_question_return_uint |
| 086_question_return_float | 19_option_type/011_question_return_float |
| 087_question_return_double | 19_option_type/012_question_return_double |
| 088_question_return_char | 19_option_type/013_question_return_char |
| 089_question_nested_call | 19_option_type/014_question_nested_call |
| 090_question_arithmetic | 19_option_type/015_question_arithmetic |
| 091_question_comparison | 19_option_type/016_question_comparison |
| 092_question_logical | 19_option_type/017_question_logical |
| 093_question_negation | 19_option_type/018_question_negation |

**Removed duplicates:** 094_question_literal, 095_question_zero, 096_question_negative (all identical to 094)

#### 20_result_type (0 tests — placeholder)

Reserved for future `!T` (Result<T>) syntax tests. Currently no a2c tests use this syntax.

#### 21_storage (3 tests)

| Old | New |
|-----|-----|
| 114_storage_module | 21_storage/001_storage_module |
| 115_storage_usage | 21_storage/002_storage_usage |
| 116_plan055_auto_storage | 21_storage/003_plan055_auto_storage |

#### 22_iterators (7 tests)

| Old | New |
|-----|-----|
| 120_iter_specs | 22_iterators/001_iter_specs |
| 121_map_adapter | 22_iterators/002_map_adapter |
| 129_terminal_operators | 22_iterators/003_terminal_operators |
| 130_terminal_operators | 22_iterators/004_terminal_operators_2 |
| 132_extended_adapters | 22_iterators/005_extended_adapters |
| 133_predicates | 22_iterators/006_predicates |
| 134_collect | 22_iterators/007_collect |

#### 23_stdlib (14 tests)

| Old | New |
|-----|-----|
| 137_std_hello | 23_stdlib/001_std_hello |
| 138_std_getpid | 23_stdlib/002_std_getpid |
| 139_std_readline | 23_stdlib/003_std_readline |
| 140_std_file | 23_stdlib/004_std_file |
| 141_std_repl | 23_stdlib/005_std_repl |
| 142_std_str | 23_stdlib/006_std_str |
| 143_file_operations | 23_stdlib/007_file_operations |
| 144_char_io | 23_stdlib/008_char_io |
| 145_advanced_io | 23_stdlib/009_advanced_io |
| 146_io_specs | 23_stdlib/010_io_specs |
| 147_std_test | 23_stdlib/011_std_test |
| 148_std_readline | 23_stdlib/012_std_readline_2 |
| 150_std_file_flush | 23_stdlib/013_std_file_flush |
| 151_std_file_read | 23_stdlib/014_std_file_read |

#### 24_runtime_size (2 tests)

| Old | New |
|-----|-----|
| 106_runtime_size_var | 24_runtime_size/001_runtime_size_var |
| 107_runtime_size_expr | 24_runtime_size/002_runtime_size_expr |

#### 25_type_checking (1 test)

| Old | New |
|-----|-----|
| 021_type_error | 25_type_checking/001_type_error |

### Enum Smoke Test Consolidation

Keep 3 representatives, delete ~21 redundant 2-variant enum smoke tests:

| Keep | Reason |
|------|--------|
| 046_mode | Simplest 2-variant enum smoke test |
| 041_tristate | Only 3-variant enum smoke test |
| 036_may_patterns | Enum with helper functions and unwrap (not trivial pattern match) |

These 3 go into `06_pattern_matching`:

| Old | New |
|-----|-----|
| 046_mode | 06_pattern_matching/004_enum_smoke_2var |
| 041_tristate | 06_pattern_matching/005_enum_smoke_3var |
| 036_may_patterns | 06_pattern_matching/006_enum_with_functions |

### Mut Bindings Tests

Moved to `03_control_flow`:

| Old | New |
|-----|-----|
| 083_mut_accumulator | 03_control_flow/005_mut_accumulator |
| 083_mut_array_sum | 03_control_flow/006_mut_array_sum |
| 083_mut_counter | 03_control_flow/007_mut_counter |
| 083_mut_multiple | 03_control_flow/008_mut_multiple |

### Stdlib Hash Tests

| Old | New |
|-----|-----|
| 123_hashmap | 23_stdlib/015_hashmap |
| 124_hashset | 23_stdlib/016_hashset |

### Stdlib File Write Stub

| Old | New |
|-----|-----|
| 149_std_file_write | 23_stdlib/017_std_file_write |

### Directories to Delete

**Orphaned directories (no .at source) — 88 total:**

All directories in the `Sub-category 5a/5b/5c` groups from the analysis. These include:
- 030_borrow_view, 031_borrow_mut, 032_borrow_take, 033_borrow_conflicts
- 037_array_return, 037_unified_section, 040_hetero_enum_types
- 041_may_basic, 042_may_string, 043_may_bool, 044_may_patterns, 045_may_nested, 046_binary, 047_tristate, 048_direction, 049_status, 050_mode, 051_result, 052_phase, 053_level, 054_state, 055_type
- 060_color, 061_size, 062_speed, 063_power, 064_signal, 065_zone, 066_mode2, 067_link, 068_source, 069_target, 070_format
- 071_question_syntax, 072_question_uint, 073_question_float, 074_question_double, 075_question_char
- 079_question_return_int, 080_question_return_str, 081_question_return_bool, 082_question_return_uint, 083_question_return_float, 084_question_return_double, 085_question_return_char
- 087_question_nested_call, 088_question_arithmetic, 089_question_comparison
- 090_pointer_types, 091_question_negation, 092_question_literal, 093_question_zero, 094_question_negative
- 095_null_coalesce, 096_error_propagate, 096_storage_usage, 097_list_storage
- 099_iter_specs, 100_map_adapter, 102_std_readline, 103_std_file, 103_generic_ptr_field
- 104_terminal_operators, 106_file_operations, 107_std_path, 108_closure
- 110_bool, 110_with_constraint, 111_io_specs, 112_inheritance, 113_std_test, 114_std_readline, 116_std_file_flush, 117_std_file_read
- 121_terminal_operators, 123_extended_adapters, 124_predicates, 125_collect
- 080_array_index_read, 080_array_slice, 082_runtime_size_var, 083_runtime_size_expr
- 092_const_generics, 095_storage_module, 100_std_hello

**May tests (replaced by Option/Result) — 5 total:**
- 033_may_basic, 034_may_string, 035_may_bool, 037_may_nested, 052_may_storage

**Redundant enum smoke tests — ~21 total:**
- 042_direction, 045_status, 048_result, 049_phase, 050_level, 051_state
- 056_side, 057_flow, 058_gate, 059_path, 060_color, 061_color
- 065_size, 067_speed, 068_power, 069_signal, 070_zone, 071_mode2
- 072_link, 073_source, 074_target, 075_format

**Duplicate hetero enum tests — 2 total:**
- 022_typed_node_checking, 063_simple_hetero_enum

**Duplicate question tests — 3 total:**
- 094_question_literal, 095_question_zero, 096_question_negative

**Runtime arrays backup (superseded by 106+107):**
- 105_runtime_arrays_backup

### Summary

| Metric | Before | After |
|--------|--------|-------|
| Total directories | 239 | ~90 |
| Active test cases | 125 | ~90 |
| Category directories | 0 (flat) | 25 (20 non-empty) |
| Number conflicts | ~50 | 0 |
| Orphaned directories | 88 | 0 |
| May tests | 6 | 0 |
| Redundant enum smoke | ~24 | 3 |
| Duplicate question tests | 3 | 0 |
| Stale .wrong files | 96 dirs | 0 |

---

## Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate a2c tests from flat numbered directories to categorized structure with numbered sub-directories.

**Architecture:** A Python migration script handles the bulk file moves. Then rewrite the a2c test runner. Finally clean up.

**Tech Stack:** Python (migration script), Rust (test runner), bash (verification)

---

### Task 1: Write the migration script

**Files:**
- Create: `crates/auto-lang/test/a2c/migrate.py`

**Step 1: Write the migration script**

The script encodes the full old→new mapping table, creates category dirs, moves test files, and cleans up stale `.wrong` files.

```python
#!/usr/bin/env python3
"""Migrate a2c tests from flat numbered dirs to categorized structure."""
import os, shutil

BASE = os.path.dirname(os.path.abspath(__file__))

# Mapping: (old_dir, new_category_dir, new_test_dir, new_file_name)
MIGRATIONS = [
    # 01_basics
    ("000_hello", "01_basics", "001_hello", "hello"),
    ("001_sqrt", "01_basics", "002_sqrt", "sqrt"),
    ("003_func", "01_basics", "003_func", "func"),
    # 02_types
    ("006_struct", "02_types", "001_struct", "struct"),
    ("007_enum", "02_types", "002_enum", "enum"),
    ("013_union", "02_types", "003_union", "union"),
    ("005_pointer", "02_types", "004_pointer", "pointer"),
    ("128_inheritance", "02_types", "005_inheritance", "inheritance"),
    ("108_pointer_types", "02_types", "006_pointer_types", "pointer_types"),
    # 03_control_flow
    ("010_if", "03_control_flow", "001_if_basic", "if_basic"),
    ("011_for", "03_control_flow", "002_for_range", "for_range"),
    ("012_is", "03_control_flow", "003_is_match", "is_match"),
    ("031_for_conditions", "03_control_flow", "004_for_conditions", "for_conditions"),
    ("083_mut_accumulator", "03_control_flow", "005_mut_accumulator", "mut_accumulator"),
    ("083_mut_array_sum", "03_control_flow", "006_mut_array_sum", "mut_array_sum"),
    ("083_mut_counter", "03_control_flow", "007_mut_counter", "mut_counter"),
    ("083_mut_multiple", "03_control_flow", "008_mut_multiple", "mut_multiple"),
    # 04_strings
    ("015_str", "04_strings", "001_str", "str"),
    ("030_str_split", "04_strings", "002_str_split", "str_split"),
    # 05_expressions
    ("028_complex_expr", "05_expressions", "001_complex_expr", "complex_expr"),
    ("054_field_access", "05_expressions", "002_field_access", "field_access"),
    ("131_bang_operator", "05_expressions", "003_bang_operator", "bang_operator"),
    ("038_binary", "05_expressions", "004_binary", "binary"),
    # 06_pattern_matching
    ("014_hetero_enum", "06_pattern_matching", "001_hetero_enum", "hetero_enum"),
    ("060_hetero_enum_verify", "06_pattern_matching", "002_hetero_enum_verify", "hetero_enum_verify"),
    ("032_hetero_enum_types", "06_pattern_matching", "003_hetero_enum_types", "hetero_enum_types"),
    ("046_mode", "06_pattern_matching", "004_enum_smoke_2var", "enum_smoke_2var"),
    ("041_tristate", "06_pattern_matching", "005_enum_smoke_3var", "enum_smoke_3var"),
    ("036_may_patterns", "06_pattern_matching", "006_enum_with_functions", "enum_with_functions"),
    # 07_ownership
    ("023_borrow_view", "07_ownership", "001_borrow_view", "borrow_view"),
    ("024_borrow_mut", "07_ownership", "002_borrow_mut", "borrow_mut"),
    ("025_borrow_move", "07_ownership", "003_borrow_move", "borrow_move"),
    ("026_borrow_conflicts", "07_ownership", "004_borrow_conflicts", "borrow_conflicts"),
    # 08_generics
    ("110_const_generics", "08_generics", "001_const_generics", "const_generics"),
    ("126_generic_field", "08_generics", "002_generic_field", "generic_field"),
    ("127_generic_ptr_field", "08_generics", "003_generic_ptr_field", "generic_ptr_field"),
    ("136_with_constraint", "08_generics", "004_with_constraint", "with_constraint"),
    ("112_generic_specs", "08_generics", "005_generic_specs", "generic_specs"),
    ("113_generic_spec_ext", "08_generics", "006_generic_spec_ext", "generic_spec_ext"),
    ("111_generic_type_alias", "08_generics", "007_generic_type_alias", "generic_type_alias"),
    # 09_option_result
    ("118_null_coalesce", "09_option_result", "001_null_coalesce", "null_coalesce"),
    ("119_error_propagate", "09_option_result", "002_error_propagate", "error_propagate"),
    ("125_closure", "09_option_result", "003_closure", "closure"),
    # 10_collections
    ("002_array", "10_collections", "001_array", "array"),
    ("029_array_return", "10_collections", "002_array_return", "array_return"),
    ("097_array_declaration", "10_collections", "003_array_declaration", "array_declaration"),
    ("098_array_mutation", "10_collections", "004_array_mutation", "array_mutation"),
    ("099_array_index_read", "10_collections", "005_array_index_read", "array_index_read"),
    ("100_array_copy", "10_collections", "006_array_copy", "array_copy"),
    ("101_array_slice", "10_collections", "007_array_slice", "array_slice"),
    ("102_array_nested", "10_collections", "008_array_nested", "array_nested"),
    ("103_array_zero_size", "10_collections", "009_array_zero_size", "array_zero_size"),
    ("104_array_loop", "10_collections", "010_array_loop", "array_loop"),
    ("117_list_storage", "10_collections", "011_list_storage", "list_storage"),
    ("122_list_iter", "10_collections", "012_list_iter", "list_iter"),
    ("055_list_capacity", "10_collections", "013_list_capacity", "list_capacity"),
    # 11_methods
    ("008_method", "11_methods", "001_method", "method"),
    ("064_multi_param", "11_methods", "002_multi_param", "multi_param"),
    ("066_generic_list", "11_methods", "003_generic_list", "generic_list"),
    # 12_specs
    ("016_basic_spec", "12_specs", "001_basic_spec", "basic_spec"),
    ("017_spec", "12_specs", "002_spec", "spec"),
    # 13_delegation
    ("018_delegation", "13_delegation", "001_single", "single"),
    ("019_multi_delegation", "13_delegation", "002_multi_delegation", "multi_delegation"),
    ("020_delegation_params", "13_delegation", "003_delegation_params", "delegation_params"),
    # 18_c_interop
    ("004_cstr", "18_c_interop", "001_cstr", "cstr"),
    ("009_alias", "18_c_interop", "002_alias", "alias"),
    ("027_unified_section", "18_c_interop", "003_unified_section", "unified_section"),
    # 19_option_type
    ("076_question_syntax", "19_option_type", "001_question_syntax", "question_syntax"),
    ("077_question_uint", "19_option_type", "002_question_uint", "question_uint"),
    ("078_question_float", "19_option_type", "003_question_float", "question_float"),
    ("079_question_double", "19_option_type", "004_question_double", "question_double"),
    ("080_question_char", "19_option_type", "005_question_char", "question_char"),
    ("081_question_void", "19_option_type", "006_question_void", "question_void"),
    ("082_question_return_int", "19_option_type", "007_question_return_int", "question_return_int"),
    ("083_question_return_str", "19_option_type", "008_question_return_str", "question_return_str"),
    ("084_question_return_bool", "19_option_type", "009_question_return_bool", "question_return_bool"),
    ("085_question_return_uint", "19_option_type", "010_question_return_uint", "question_return_uint"),
    ("086_question_return_float", "19_option_type", "011_question_return_float", "question_return_float"),
    ("087_question_return_double", "19_option_type", "012_question_return_double", "question_return_double"),
    ("088_question_return_char", "19_option_type", "013_question_return_char", "question_return_char"),
    ("089_question_nested_call", "19_option_type", "014_question_nested_call", "question_nested_call"),
    ("090_question_arithmetic", "19_option_type", "015_question_arithmetic", "question_arithmetic"),
    ("091_question_comparison", "19_option_type", "016_question_comparison", "question_comparison"),
    ("092_question_logical", "19_option_type", "017_question_logical", "question_logical"),
    ("093_question_negation", "19_option_type", "018_question_negation", "question_negation"),
    # 21_storage
    ("114_storage_module", "21_storage", "001_storage_module", "storage_module"),
    ("115_storage_usage", "21_storage", "002_storage_usage", "storage_usage"),
    ("116_plan055_auto_storage", "21_storage", "003_plan055_auto_storage", "plan055_auto_storage"),
    # 22_iterators
    ("120_iter_specs", "22_iterators", "001_iter_specs", "iter_specs"),
    ("121_map_adapter", "22_iterators", "002_map_adapter", "map_adapter"),
    ("129_terminal_operators", "22_iterators", "003_terminal_operators", "terminal_operators"),
    ("130_terminal_operators", "22_iterators", "004_terminal_operators_2", "terminal_operators_2"),
    ("132_extended_adapters", "22_iterators", "005_extended_adapters", "extended_adapters"),
    ("133_predicates", "22_iterators", "006_predicates", "predicates"),
    ("134_collect", "22_iterators", "007_collect", "collect"),
    # 23_stdlib
    ("137_std_hello", "23_stdlib", "001_std_hello", "std_hello"),
    ("138_std_getpid", "23_stdlib", "002_std_getpid", "std_getpid"),
    ("139_std_readline", "23_stdlib", "003_std_readline", "std_readline"),
    ("140_std_file", "23_stdlib", "004_std_file", "std_file"),
    ("141_std_repl", "23_stdlib", "005_std_repl", "std_repl"),
    ("142_std_str", "23_stdlib", "006_std_str", "std_str"),
    ("143_file_operations", "23_stdlib", "007_file_operations", "file_operations"),
    ("144_char_io", "23_stdlib", "008_char_io", "char_io"),
    ("145_advanced_io", "23_stdlib", "009_advanced_io", "advanced_io"),
    ("146_io_specs", "23_stdlib", "010_io_specs", "io_specs"),
    ("147_std_test", "23_stdlib", "011_std_test", "std_test"),
    ("148_std_readline", "23_stdlib", "012_std_readline_2", "std_readline_2"),
    ("150_std_file_flush", "23_stdlib", "013_std_file_flush", "std_file_flush"),
    ("151_std_file_read", "23_stdlib", "014_std_file_read", "std_file_read"),
    ("123_hashmap", "23_stdlib", "015_hashmap", "hashmap"),
    ("124_hashset", "23_stdlib", "016_hashset", "hashset"),
    ("149_std_file_write", "23_stdlib", "017_std_file_write", "std_file_write"),
    # 24_runtime_size
    ("106_runtime_size_var", "24_runtime_size", "001_runtime_size_var", "runtime_size_var"),
    ("107_runtime_size_expr", "24_runtime_size", "002_runtime_size_expr", "runtime_size_expr"),
    # 25_type_checking
    ("021_type_error", "25_type_checking", "001_type_error", "type_error"),
]

DELETE_DIRS = [
    # May tests (replaced by Option/Result)
    "033_may_basic", "034_may_string", "035_may_bool", "037_may_nested", "052_may_storage",
    # Redundant enum smoke tests (keep 046_mode, 041_tristate, 036_may_patterns)
    "042_direction", "045_status", "048_result", "049_phase", "050_level", "051_state",
    "056_side", "057_flow", "058_gate", "059_path", "060_color", "061_color",
    "065_size", "067_speed", "068_power", "069_signal", "070_zone", "071_mode2",
    "072_link", "073_source", "074_target", "075_format",
    # Duplicate hetero enum tests
    "022_typed_node_checking", "063_simple_hetero_enum",
    # Duplicate question tests
    "094_question_literal", "095_question_zero", "096_question_negative",
    # Runtime arrays backup (superseded)
    "105_runtime_arrays_backup",
]


def get_old_file_name(old_dir):
    """Extract the file name from old dir like '032_hetero_enum_types' -> 'hetero_enum_types'."""
    parts = old_dir.split("_", 1)
    return parts[1] if len(parts) > 1 else old_dir


def migrate():
    moved = 0
    for old_dir, cat_dir, new_test_dir, new_file_name in MIGRATIONS:
        old_path = os.path.join(BASE, old_dir)
        if not os.path.isdir(old_path):
            print(f"SKIP (not found): {old_dir}")
            continue

        old_file_name = get_old_file_name(old_dir)
        cat_path = os.path.join(BASE, cat_dir)
        new_path = os.path.join(cat_path, new_test_dir)
        os.makedirs(new_path, exist_ok=True)

        # Move .at file
        old_at = os.path.join(old_path, f"{old_file_name}.at")
        new_at = os.path.join(new_path, f"{new_file_name}.at")
        if os.path.exists(old_at):
            shutil.copy2(old_at, new_at)

        # Move .expected.c file
        old_exp_c = os.path.join(old_path, f"{old_file_name}.expected.c")
        new_exp_c = os.path.join(new_path, f"{new_file_name}.expected.c")
        if os.path.exists(old_exp_c):
            shutil.copy2(old_exp_c, new_exp_c)

        # Move .expected.h file
        old_exp_h = os.path.join(old_path, f"{old_file_name}.expected.h")
        new_exp_h = os.path.join(new_path, f"{new_file_name}.expected.h")
        if os.path.exists(old_exp_h):
            shutil.copy2(old_exp_h, new_exp_h)

        # Skip .wrong files (stale output)

        moved += 1
        print(f"  {old_dir}/{old_file_name} -> {cat_dir}/{new_test_dir}/{new_file_name}")

    print(f"\nMigrated {moved} test cases.")

    # Delete specified dirs
    deleted = 0
    for d in DELETE_DIRS:
        dp = os.path.join(BASE, d)
        if os.path.isdir(dp):
            shutil.rmtree(dp)
            deleted += 1
            print(f"  DELETED: {d}")

    # Delete migrated old dirs
    for old_dir, _, _, _ in MIGRATIONS:
        dp = os.path.join(BASE, old_dir)
        if os.path.isdir(dp):
            shutil.rmtree(dp)
            deleted += 1
            print(f"  REMOVED old: {old_dir}")

    # Delete orphaned dirs (no .at file, not a category dir)
    for entry in sorted(os.listdir(BASE)):
        entry_path = os.path.join(BASE, entry)
        if not os.path.isdir(entry_path):
            continue
        if entry in ("__pycache__",) or entry.startswith("."):
            continue
        # Skip category dirs (NN_name format)
        if "_" in entry and entry.split("_")[0].isdigit() and len(entry.split("_")[0]) == 2:
            continue
        # This is an orphaned dir
        has_at = any(f.endswith(".at") for f in os.listdir(entry_path))
        if not has_at:
            shutil.rmtree(entry_path)
            deleted += 1
            print(f"  ORPHAN deleted: {entry}")

    print(f"\nDeleted {deleted} directories.")


if __name__ == "__main__":
    migrate()
```

**Step 2: Run the migration script**

Run: `cd crates/auto-lang/test/a2c && python migrate.py`

Expected: All test directories moved into category dirs, old dirs removed, orphaned/stale dirs deleted.

**Step 3: Verify directory structure**

Run: `ls crates/auto-lang/test/a2c/`
Expected: 25 numbered category dirs + `migrate.py`

**Step 4: Commit the migration**

```bash
git add -A crates/auto-lang/test/a2c/
git commit -m "refactor: migrate a2c tests to categorized directory structure"
```

---

### Task 2: Rewrite the a2c test runner

**Files:**
- Modify: `crates/auto-lang/src/trans/c.rs` (test functions at bottom of file)
- Optionally create: `crates/auto-lang/src/tests/a2c_tests.rs` (if splitting from c.rs)

**Step 1: Update `test_a2c()` helper function**

Update to accept `category/NNN_name` path format (same pattern as Plan 170's `test_a2r()`):

```rust
fn test_a2c(case: &str) -> AutoResult<()> {
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.split("_").collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/a2c/{}/{}.at", case, name);
    let src_path = d.join(&src_path);
    let src = read_to_string(src_path.as_path())?;

    let exp_c_path = format!("test/a2c/{}/{}.expected.c", case, name);
    let exp_c_path = d.join(&exp_c_path);
    let expected_c = if !exp_c_path.is_file() {
        "".to_string()
    } else {
        read_to_string(exp_c_path.as_path())?
    };

    let exp_h_path = format!("test/a2c/{}/{}.expected.h", case, name);
    let exp_h_path = d.join(&exp_h_path);
    let expected_h = if !exp_h_path.is_file() {
        "".to_string()
    } else {
        read_to_string(exp_h_path.as_path())?
    };

    let (gen_c, gen_h) = transpile_c(&src)?;

    if gen_c != expected_c.as_bytes() {
        let gen_path = format!("test/a2c/{}/{}.wrong.c", case, name);
        let gen_path = d.join(gen_path);
        std::fs::write(&gen_path, gen_c)?;
    }

    if gen_h != expected_h.as_bytes() {
        let gen_path = format!("test/a2c/{}/{}.wrong.h", case, name);
        let gen_path = d.join(gen_path);
        std::fs::write(&gen_path, gen_h)?;
    }

    assert_eq!(String::from_utf8_lossy(&gen_c), expected_c);
    assert_eq!(String::from_utf8_lossy(&gen_h), expected_h);
    Ok(())
}
```

**Step 2: Rewrite all `#[test]` functions**

Remove ALL existing `#[test] fn test_XXX()` functions. Write new categorized test functions:

```rust
// === 01_basics ===
#[test] fn test_01_basics_001_hello() { test_a2c("01_basics/001_hello").unwrap(); }
#[test] fn test_01_basics_002_sqrt() { test_a2c("01_basics/002_sqrt").unwrap(); }
#[test] fn test_01_basics_003_func() { test_a2c("01_basics/003_func").unwrap(); }

// === 02_types ===
#[test] fn test_02_types_001_struct() { test_a2c("02_types/001_struct").unwrap(); }
// ... (all categories)
```

**Step 3: Compile to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/auto-lang/src/trans/c.rs
git commit -m "refactor: rewrite a2c test runner with categorized test functions"
```

---

### Task 3: Generate missing expected output for stub tests

**Files:**
- Generate: `.expected.c` and `.expected.h` for stub tests in categories 03, 04, 08, 10, 19, 23

**Step 1: Identify tests needing expected output**

These tests have `.at` but are missing `.expected.c` and/or `.expected.h`:
- 04_strings/002_str_split (SOURCE-ONLY)
- 03_control_flow/004_for_conditions (SOURCE-ONLY)
- 08_generics/007_generic_type_alias (SOURCE-ONLY)
- 10_collections/003-013 (INCOMPLETE — missing .expected.h)
- 19_option_type/006_question_void (SOURCE-ONLY)
- 19_option_type/017_question_logical (SOURCE-ONLY)
- 23_stdlib/015_hashmap (SOURCE-ONLY)
- 23_stdlib/016_hashset (SOURCE-ONLY)
- 23_stdlib/017_std_file_write (SOURCE-ONLY)
- 25_type_checking/001_type_error (SOURCE-ONLY)

**Step 2: Run each test to generate `.wrong` files**

For each stub test:
1. Run: `cargo test -p auto-lang test_{category}_{name}` — will fail and create `.wrong.c`/`.wrong.h`
2. Review the `.wrong` output
3. If correct, rename to `.expected.c`/`.expected.h`
4. If the test uses features not supported by the C transpiler, remove the test

**Step 3: Commit**

```bash
git add -A crates/auto-lang/test/a2c/
git commit -m "feat: generate expected output for a2c stub tests"
```

---

### Task 4: Run full test suite and fix failures

**Step 1: Run all a2c tests**

Run: `cargo test -p auto-lang -- trans 2>&1 | tail -20`
Expected: All tests PASS

**Step 2: If tests fail, investigate and fix**

Common failure modes:
- **File not found**: Directory or file name mismatch — check migration
- **Output mismatch**: Compare `.wrong.c`/`.wrong.h` with `.expected.*` — may need to update expected output
- **Parse error**: Test may use outdated syntax — update `.at` source

**Step 3: Final commit**

```bash
git add -A
git commit -m "fix: resolve a2c test reorganization issues"
```

---

### Task 5: Clean up

**Files:**
- Delete: `crates/auto-lang/test/a2c/migrate.py`

**Step 1: Remove migration script**

Run: `rm crates/auto-lang/test/a2c/migrate.py`

**Step 2: Verify final state**

Run: `ls crates/auto-lang/test/a2c/`
Expected: 25 numbered category dirs (20 non-empty, 5 empty placeholders)

Run: `cargo test -p auto-lang 2>&1 | grep "test result:"`
Expected: All test results show passed

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: clean up a2c migration artifacts"
```

---

## Open Questions

1. **`060_color` vs `061_color`**: Both exist as separate directories but contain the same test. The migration script will handle this — one gets migrated, the other gets deleted as orphaned.

2. **`130_terminal_operators` vs `129_terminal_operators`**: Both exist. Need to verify they are different tests before migrating. If identical, keep only one.

3. **`148_std_readline` vs `139_std_readline`**: Both exist. Need to verify they are different tests. If identical, keep only one.

4. **`135_bool`**: Listed in some places but needs verification. May be a duplicate of another test.
