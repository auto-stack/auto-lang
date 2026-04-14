# Plan 170: A2R Test Suite Reorganization

## Objective

Reorganize the a2r (Auto-to-Rust) transpiler test suite from chaotic sequential numbering into a categorized, numbered directory structure that makes tests discoverable and maintainable.

## Status: COMPLETE

All tasks implemented and verified. 144 a2r tests passing, `cargo build --examples` clean.

## Current State (Before)

- ~60 active test cases with sequential numbering (000-163, 999)
- ~35 orphaned directories (on disk but no test runner)
- 14 number conflicts (different features sharing the same number)
- Huge gaps in numbering (036-054, 056-071, 096-108)
- Inline tests (920-942) with source as string literals
- No clear grouping by feature category

## Final State (After)

- 144 test cases across 17 categorized directories
- 0 orphaned directories
- 0 number conflicts
- 0 inline tests (all converted to file-based)
- 55 `[[example]]` entries that compile as standalone Rust
- 64 test cases excluded from `[[example]]` (transpiler output not standalone-compilable)

## Design

### Directory Structure

Category-based directories with sequence numbers, test cases numbered within each category:

```
test/a2r/
  01_basics/          # Hello world, functions, fundamentals
  02_types/           # Struct, enum, union, pointer, inheritance, object, cstr
  03_control_flow/    # if, for, while, is matching
  04_strings/         # f-strings, multi-line strings
  05_expressions/     # Arithmetic, unary, indexing, blocks, ranges, composition
  06_pattern_matching/# Enum patterns, struct destructuring, hetero enums
  07_ownership/       # Borrow view/mut/move, conflicts
  08_generics/        # Type aliases, const generics, constraints, map types
  09_option_result/   # Option, Result, null coalesce, error propagation, question types
  10_collections/     # Array, list storage, map functions, method chains
  11_methods/         # Instance/static methods, closures, ext-for, ext-from
  12_specs/           # Spec (trait) declarations and usage
  13_delegation/      # Single/multi delegation with specs
  14_modules/         # Imports, visibility, multi-file, const, shared var
  15_type_conversion/ # Type casts, to() conversions, pointer methods
  16_interop/         # Async, tokio, field attributes
  17_autocode/        # Real-world integration tests from auto-coder project
```

Each test case: `NN_name/name.at` + `NN_name/name.expected.rs`

### Test Runner

- Keep individual `#[test]` functions (VSCode discoverable)
- Function naming: `test_{category}_{name}` (e.g., `test_03_control_flow_001_if_basic`)
- `test_a2r()` updated to accept `category/NNN_name` path format
- `test_14_modules_005_multi_file` retains special multi-file assertion logic

### Categories and Test Mappings

#### 01_basics (4 tests)

| Old | New |
|-----|-----|
| 000_hello | 01_basics/001_hello |
| 001_sqrt | 01_basics/002_sqrt |
| 003_func | 01_basics/003_func |
| 999_doc_comments | 01_basics/004_doc_comments |

#### 02_types (8 tests)

| Old | New |
|-----|-----|
| 006_struct | 02_types/001_struct |
| 007_enum | 02_types/002_enum |
| 055_union | 02_types/003_union |
| 005_pointer | 02_types/004_pointer |
| 035_inheritance | 02_types/005_inheritance |
| 028_object | 02_types/006_object |
| 004_cstr | 02_types/007_cstr |
| 151_mut_self | 02_types/008_mut_self |

#### 03_control_flow (9 tests)

| Old | New |
|-----|-----|
| 010_if | 03_control_flow/001_if_basic |
| 015_nested_if | 03_control_flow/002_if_nested |
| 139_if_multistmt | 03_control_flow/003_if_multistmt |
| 140_if_return | 03_control_flow/004_if_return |
| 011_for | 03_control_flow/005_for_range |
| 031_for_conditions | 03_control_flow/006_for_conditions |
| 013_while | 03_control_flow/007_while_loop |
| 012_is | 03_control_flow/008_is_match |
| 132_is_multi_stmt | 03_control_flow/009_is_multi_stmt |

#### 04_strings (3 tests)

| Old | New |
|-----|-----|
| 024_fstring | 04_strings/001_fstring |
| 025_fstring_edge | 04_strings/002_fstring_edge |
| 163_multi_str | 04_strings/003_multi_str |

#### 05_expressions (9 tests)

| Old | New |
|-----|-----|
| 023_arithmetic | 05_expressions/001_arithmetic |
| 022_unary | 05_expressions/002_unary |
| 021_indexing | 05_expressions/003_indexing |
| 019_blocks | 05_expressions/004_blocks |
| 026_ref_expr | 05_expressions/005_ref_expr |
| 027_range_expr | 05_expressions/006_range_expr |
| 029_composition | 05_expressions/007_composition |
| 030_field_composition | 05_expressions/008_field_composition |
| 020_comprehensive | 05_expressions/009_comprehensive |

#### 06_pattern_matching (5 tests)

| Old | New |
|-----|-----|
| 018_enum_pattern | 06_pattern_matching/001_enum_pattern |
| 154_struct_destructure | 06_pattern_matching/002_struct_destructure |
| 143_empty_variant_match | 06_pattern_matching/003_empty_variant_match |
| 014_hetero_enum | 06_pattern_matching/004_hetero_enum |
| 109_generic_hetero_enum | 06_pattern_matching/005_generic_hetero_enum |

#### 07_ownership (4 tests)

| Old | New |
|-----|-----|
| 023_borrow_view | 07_ownership/001_borrow_view |
| 024_borrow_mut | 07_ownership/002_borrow_mut |
| 025_borrow_move | 07_ownership/003_borrow_move |
| 026_borrow_conflicts | 07_ownership/004_borrow_conflicts |

#### 08_generics (6 tests)

| Old | New |
|-----|-----|
| 111_generic_alias | 08_generics/001_type_alias |
| 110_const_generics | 08_generics/002_const_generics |
| 126_generic_field | 08_generics/003_generic_field |
| 127_generic_ptr_field | 08_generics/004_generic_ptr_field |
| 155_with_constraint | 08_generics/005_with_constraint |
| 128_map_type | 08_generics/006_map_type |

#### 09_option_result (31 tests: 2 active + 29 reactivated)

| Old | New |
|-----|-----|
| 120_option | 09_option_result/001_option |
| 130_option_construct | 09_option_result/002_option_construct |
| 118_null_coalesce | 09_option_result/003_null_coalesce |
| 119_error_propagate | 09_option_result/004_error_propagate |
| 072_question_uint | 09_option_result/005_question_uint |
| 073_question_float | 09_option_result/006_question_float |
| 074_question_double | 09_option_result/007_question_double |
| 079_question_return_int | 09_option_result/008_question_return_int |
| 080_question_return_str | 09_option_result/009_question_return_str |
| 081_question_return_bool | 09_option_result/010_question_return_bool |
| 082_question_propagate | 09_option_result/011_question_propagate |
| 083_question_return_float | 09_option_result/012_question_return_float |
| 084_question_return_double | 09_option_result/013_question_return_double |
| 085_question_return_char | 09_option_result/014_question_return_char |
| 085_question_return_uint | 09_option_result/015_question_return_uint |
| 086_question_return_float | 09_option_result/016_question_return_float |
| 087_question_return_double | 09_option_result/017_question_return_double |
| 088_question_return_char | 09_option_result/018_question_return_char |
| 089_question_nested_call | 09_option_result/019_question_nested_call |
| 090_question_arithmetic | 09_option_result/020_question_arithmetic |
| 091_question_comparison | 09_option_result/021_question_comparison |
| 092_question_literal | 09_option_result/022_question_literal |
| 093_question_negation | 09_option_result/023_question_negation |
| 094_question_zero | 09_option_result/024_question_zero |
| 095_question_negative | 09_option_result/025_question_negative |
| 120_list_basic | 09_option_result/026_list_basic |
| 121_list_methods | 09_option_result/027_list_methods |
| 122_list_may | 09_option_result/028_list_may |
| 123_list_propagate | 09_option_result/029_list_propagate |
| 124_list_coalesce | 09_option_result/030_list_coalesce |

#### 10_collections (5 tests)

| Old | New |
|-----|-----|
| 002_array | 10_collections/001_array |
| 117_list_storage | 10_collections/002_list_storage |
| 129_map_func | 10_collections/003_map_func |
| 138_list_as_cast | 10_collections/004_list_as_cast |
| 131_method_chain | 10_collections/005_method_chain |

#### 11_methods (7 tests)

| Old | New |
|-----|-----|
| 008_method | 11_methods/001_method |
| 017_struct_methods | 11_methods/002_struct_methods |
| 014_closure | 11_methods/003_closure |
| 148_static_fn | 11_methods/004_static_fn |
| 141_func_literal_return | 11_methods/005_func_literal_return |
| 153_ext_for | 11_methods/006_ext_for |
| 156_ext_from | 11_methods/007_ext_from |

#### 12_specs (3 tests)

| Old | New |
|-----|-----|
| 016_basic_spec | 12_specs/001_basic_spec |
| 017_spec | 12_specs/002_spec |
| 031_spec | 12_specs/003_spec_delegation |

#### 13_delegation (3 tests)

| Old | New |
|-----|-----|
| 032_delegation | 13_delegation/001_single |
| 033_multi_delegation | 13_delegation/002_multi_spec |
| 034_delegation_params | 13_delegation/003_multi_delegation |

#### 14_modules (8 tests)

| Old | New |
|-----|-----|
| 133_rust_use | 14_modules/001_rust_use |
| 159_pub_use | 14_modules/002_pub_use |
| 149_pub_visibility | 14_modules/003_pub_visibility |
| 160_wildcard_import | 14_modules/004_wildcard_import |
| 161_multi_file | 14_modules/005_multi_file |
| 157_const_decl | 14_modules/006_const_decl |
| 162_shared_var | 14_modules/007_shared_var |
| 135_derive_attr | 14_modules/008_derive_attr |

#### 15_type_conversion (3 tests)

| Old | New |
|-----|-----|
| 136_type_cast | 15_type_conversion/001_type_cast |
| 142_to_convert | 15_type_conversion/002_to_convert |
| 137_ptr_methods | 15_type_conversion/003_ptr_methods |
| 158_box_arc | 15_type_conversion/004_box_arc |

#### 16_interop (3 tests)

| Old | New |
|-----|-----|
| 134_async_fn | 16_interop/001_async_fn |
| 150_tokio_main | 16_interop/002_tokio_main |
| 152_field_attrs | 16_interop/003_field_attrs |

#### 17_autocode (17 tests)

| Old | New |
|-----|-----|
| test_autocode_types | 17_autocode/001_types |
| test_autocode_permission | 17_autocode/002_permission |
| test_autocode_tools | 17_autocode/003_tools |
| test_autocode_sse | 17_autocode/004_sse |
| test_autocode_context | 17_autocode/005_context |
| test_autocode_settings | 17_autocode/006_settings |
| test_autocode_agent | 17_autocode/007_agent |
| test_autocode_anthropic | 17_autocode/008_anthropic |
| test_autocode_openai | 17_autocode/009_openai |
| test_autocode_session | 17_autocode/010_session |
| test_autocode_repl | 17_autocode/011_repl |
| test_autocode_main | 17_autocode/012_main |
| test_autocode_mod | 17_autocode/013_mod |
| test_autocode_tool_bash | 17_autocode/014_tool_bash |
| test_autocode_tool_grep | 17_autocode/015_tool_grep |
| test_autocode_tool_file_read | 17_autocode/016_tool_file_read |
| test_autocode_tool_file_write | 17_autocode/017_tool_file_write |
| test_autocode_tool_file_edit | 17_autocode/018_tool_file_edit |
| test_911_detailed_errors | 17_autocode/019_detailed_errors |

### Inline Test Conversions (920-942)

These inline tests get converted to file-based tests in their appropriate categories:

| Old | New Category | New Name |
|-----|-------------|----------|
| 920_enum_as_fn_param | 06_pattern_matching | 006_enum_fn_param |
| 921_is_match_in_ext | 06_pattern_matching | 007_is_in_ext |
| 922_or_keyword | 05_expressions | 010_or_keyword |
| 923_backtick_fstring | 04_strings | 004_backtick_string |
| 924_escaped_quotes | 04_strings | 005_escaped_quotes |
| 925_option_bool_field | 09_option_result | 031_option_bool_field |
| 926_const_declaration | 14_modules | 009_const_before_ext |
| 927_empty_body_comment | 11_methods | 008_empty_body |
| 928_self_field_access | 02_types | 009_self_field |
| 929_is_non_exhaustive | 03_control_flow | 010_is_non_exhaustive |
| 930_fn_result_enum | 09_option_result | 032_fn_result_enum |
| 940_left_shift_not_supported | 05_expressions | 011_no_left_shift |
| 941_tuple_in_generic | 08_generics | 007_no_tuple_generic |
| 942_ext_is_keyword | 02_types | 010_ext_keyword |

### Directories to Delete (10 total)

**Stale delegation tests** (same .at as active tests, outdated expected output):
- 018_delegation
- 019_multi_delegation
- 020_delegation_params

**Incomplete tests** (missing .expected.rs):
- 013_union
- 014_tag
- 111_generic_type_alias
- 112_generic_specs
- 113_generic_spec_ext
- 114_storage_module
- 115_storage_usage
- 116_plan055_auto_storage

### Summary

| Metric | Before | After |
|--------|--------|-------|
| Active test cases | ~60 | 144 |
| Category directories | 0 (flat) | 17 |
| Number conflicts | 14 | 0 |
| Orphaned directories | ~35 | 0 |
| Inline tests | 14 | 0 (all file-based) |
| Compilable `[[example]]` targets | ~60 (many broken) | 55 (all clean) |

---

## Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate a2r tests from flat numbered directories to categorized structure with numbered sub-directories.

**Architecture:** A Python migration script handles the bulk file moves. Then update `test_a2r()` and rewrite `a2r_tests.rs`. Finally convert inline tests and clean up.

**Tech Stack:** Python (migration script), Rust (test runner), bash (verification)

---

### Task 1: Write the migration script

**Files:**
- Create: `crates/auto-lang/test/a2r/migrate.py`

**Step 1: Write the migration script**

The script encodes the full old→new mapping table, creates category dirs, moves test dirs, and renames files inside.

```python
#!/usr/bin/env python3
"""Migrate a2r tests from flat numbered dirs to categorized structure."""
import os, shutil

BASE = os.path.dirname(os.path.abspath(__file__))

# Mapping: (old_dir, new_category_dir, new_test_dir, new_file_name)
# new_file_name = name used for .at and .expected.rs files inside new dir
MIGRATIONS = [
    # 01_basics
    ("000_hello", "01_basics", "001_hello", "hello"),
    ("001_sqrt", "01_basics", "002_sqrt", "sqrt"),
    ("003_func", "01_basics", "003_func", "func"),
    ("999_doc_comments", "01_basics", "004_doc_comments", "doc_comments"),
    # 02_types
    ("006_struct", "02_types", "001_struct", "struct"),
    ("007_enum", "02_types", "002_enum", "enum"),
    ("055_union", "02_types", "003_union", "union"),
    ("005_pointer", "02_types", "004_pointer", "pointer"),
    ("035_inheritance", "02_types", "005_inheritance", "inheritance"),
    ("028_object", "02_types", "006_object", "object"),
    ("004_cstr", "02_types", "007_cstr", "cstr"),
    ("151_mut_self", "02_types", "008_mut_self", "mut_self"),
    # 03_control_flow
    ("010_if", "03_control_flow", "001_if_basic", "if_basic"),
    ("015_nested_if", "03_control_flow", "002_if_nested", "if_nested"),
    ("139_if_multistmt", "03_control_flow", "003_if_multistmt", "if_multistmt"),
    ("140_if_return", "03_control_flow", "004_if_return", "if_return"),
    ("011_for", "03_control_flow", "005_for_range", "for_range"),
    ("031_for_conditions", "03_control_flow", "006_for_conditions", "for_conditions"),
    ("013_while", "03_control_flow", "007_while_loop", "while_loop"),
    ("012_is", "03_control_flow", "008_is_match", "is_match"),
    ("132_is_multi_stmt", "03_control_flow", "009_is_multi_stmt", "is_multi_stmt"),
    # 04_strings
    ("024_fstring", "04_strings", "001_fstring", "fstring"),
    ("025_fstring_edge", "04_strings", "002_fstring_edge", "fstring_edge"),
    ("163_multi_str", "04_strings", "003_multi_str", "multi_str"),
    # 05_expressions
    ("023_arithmetic", "05_expressions", "001_arithmetic", "arithmetic"),
    ("022_unary", "05_expressions", "002_unary", "unary"),
    ("021_indexing", "05_expressions", "003_indexing", "indexing"),
    ("019_blocks", "05_expressions", "004_blocks", "blocks"),
    ("026_ref_expr", "05_expressions", "005_ref_expr", "ref_expr"),
    ("027_range_expr", "05_expressions", "006_range_expr", "range_expr"),
    ("029_composition", "05_expressions", "007_composition", "composition"),
    ("030_field_composition", "05_expressions", "008_field_composition", "field_composition"),
    ("020_comprehensive", "05_expressions", "009_comprehensive", "comprehensive"),
    # 06_pattern_matching
    ("018_enum_pattern", "06_pattern_matching", "001_enum_pattern", "enum_pattern"),
    ("154_struct_destructure", "06_pattern_matching", "002_struct_destructure", "struct_destructure"),
    ("143_empty_variant_match", "06_pattern_matching", "003_empty_variant_match", "empty_variant_match"),
    ("014_hetero_enum", "06_pattern_matching", "004_hetero_enum", "hetero_enum"),
    ("109_generic_hetero_enum", "06_pattern_matching", "005_generic_hetero_enum", "generic_hetero_enum"),
    # 07_ownership
    ("023_borrow_view", "07_ownership", "001_borrow_view", "borrow_view"),
    ("024_borrow_mut", "07_ownership", "002_borrow_mut", "borrow_mut"),
    ("025_borrow_move", "07_ownership", "003_borrow_move", "borrow_move"),
    ("026_borrow_conflicts", "07_ownership", "004_borrow_conflicts", "borrow_conflicts"),
    # 08_generics
    ("111_generic_alias", "08_generics", "001_type_alias", "type_alias"),
    ("110_const_generics", "08_generics", "002_const_generics", "const_generics"),
    ("126_generic_field", "08_generics", "003_generic_field", "generic_field"),
    ("127_generic_ptr_field", "08_generics", "004_generic_ptr_field", "generic_ptr_field"),
    ("155_with_constraint", "08_generics", "005_with_constraint", "with_constraint"),
    ("128_map_type", "08_generics", "006_map_type", "map_type"),
    # 09_option_result
    ("120_option", "09_option_result", "001_option", "option"),
    ("130_option_construct", "09_option_result", "002_option_construct", "option_construct"),
    ("118_null_coalesce", "09_option_result", "003_null_coalesce", "null_coalesce"),
    ("119_error_propagate", "09_option_result", "004_error_propagate", "error_propagate"),
    ("072_question_uint", "09_option_result", "005_question_uint", "question_uint"),
    ("073_question_float", "09_option_result", "006_question_float", "question_float"),
    ("074_question_double", "09_option_result", "007_question_double", "question_double"),
    ("079_question_return_int", "09_option_result", "008_question_return_int", "question_return_int"),
    ("080_question_return_str", "09_option_result", "009_question_return_str", "question_return_str"),
    ("081_question_return_bool", "09_option_result", "010_question_return_bool", "question_return_bool"),
    ("082_question_propagate", "09_option_result", "011_question_propagate", "question_propagate"),
    ("083_question_return_float", "09_option_result", "012_question_return_float", "question_return_float"),
    ("084_question_return_double", "09_option_result", "013_question_return_double", "question_return_double"),
    ("085_question_return_char", "09_option_result", "014_question_return_char", "question_return_char"),
    ("085_question_return_uint", "09_option_result", "015_question_return_uint", "question_return_uint"),
    ("086_question_return_float", "09_option_result", "016_question_return_float", "question_return_float"),
    ("087_question_return_double", "09_option_result", "017_question_return_double", "question_return_double"),
    ("088_question_return_char", "09_option_result", "018_question_return_char", "question_return_char"),
    ("089_question_nested_call", "09_option_result", "019_question_nested_call", "question_nested_call"),
    ("090_question_arithmetic", "09_option_result", "020_question_arithmetic", "question_arithmetic"),
    ("091_question_comparison", "09_option_result", "021_question_comparison", "question_comparison"),
    ("092_question_literal", "09_option_result", "022_question_literal", "question_literal"),
    ("093_question_negation", "09_option_result", "023_question_negation", "question_negation"),
    ("094_question_zero", "09_option_result", "024_question_zero", "question_zero"),
    ("095_question_negative", "09_option_result", "025_question_negative", "question_negative"),
    ("120_list_basic", "09_option_result", "026_list_basic", "list_basic"),
    ("121_list_methods", "09_option_result", "027_list_methods", "list_methods"),
    ("122_list_may", "09_option_result", "028_list_may", "list_may"),
    ("123_list_propagate", "09_option_result", "029_list_propagate", "list_propagate"),
    ("124_list_coalesce", "09_option_result", "030_list_coalesce", "list_coalesce"),
    # 10_collections
    ("002_array", "10_collections", "001_array", "array"),
    ("117_list_storage", "10_collections", "002_list_storage", "list_storage"),
    ("129_map_func", "10_collections", "003_map_func", "map_func"),
    ("138_list_as_cast", "10_collections", "004_list_as_cast", "list_as_cast"),
    ("131_method_chain", "10_collections", "005_method_chain", "method_chain"),
    # 11_methods
    ("008_method", "11_methods", "001_method", "method"),
    ("017_struct_methods", "11_methods", "002_struct_methods", "struct_methods"),
    ("014_closure", "11_methods", "003_closure", "closure"),
    ("148_static_fn", "11_methods", "004_static_fn", "static_fn"),
    ("141_func_literal_return", "11_methods", "005_func_literal_return", "func_literal_return"),
    ("153_ext_for", "11_methods", "006_ext_for", "ext_for"),
    ("156_ext_from", "11_methods", "007_ext_from", "ext_from"),
    # 12_specs
    ("016_basic_spec", "12_specs", "001_basic_spec", "basic_spec"),
    ("017_spec", "12_specs", "002_spec", "spec"),
    ("031_spec", "12_specs", "003_spec_delegation", "spec_delegation"),
    # 13_delegation
    ("032_delegation", "13_delegation", "001_single", "single"),
    ("033_multi_delegation", "13_delegation", "002_multi_spec", "multi_spec"),
    ("034_delegation_params", "13_delegation", "003_multi_delegation", "multi_delegation"),
    # 14_modules
    ("133_rust_use", "14_modules", "001_rust_use", "rust_use"),
    ("159_pub_use", "14_modules", "002_pub_use", "pub_use"),
    ("149_pub_visibility", "14_modules", "003_pub_visibility", "pub_visibility"),
    ("160_wildcard_import", "14_modules", "004_wildcard_import", "wildcard_import"),
    ("161_multi_file", "14_modules", "005_multi_file", "multi_file"),
    ("157_const_decl", "14_modules", "006_const_decl", "const_decl"),
    ("162_shared_var", "14_modules", "007_shared_var", "shared_var"),
    ("135_derive_attr", "14_modules", "008_derive_attr", "derive_attr"),
    # 15_type_conversion
    ("136_type_cast", "15_type_conversion", "001_type_cast", "type_cast"),
    ("142_to_convert", "15_type_conversion", "002_to_convert", "to_convert"),
    ("137_ptr_methods", "15_type_conversion", "003_ptr_methods", "ptr_methods"),
    ("158_box_arc", "15_type_conversion", "004_box_arc", "box_arc"),
    # 16_interop
    ("134_async_fn", "16_interop", "001_async_fn", "async_fn"),
    ("150_tokio_main", "16_interop", "002_tokio_main", "tokio_main"),
    ("152_field_attrs", "16_interop", "003_field_attrs", "field_attrs"),
]

DELETE_DIRS = [
    "013_union", "014_tag", "018_delegation", "019_multi_delegation",
    "020_delegation_params", "111_generic_type_alias", "112_generic_specs",
    "113_generic_spec_ext", "114_storage_module", "115_storage_usage",
    "116_plan055_auto_storage",
]


def get_old_file_name(old_dir):
    """Extract the file name from old dir like '032_delegation' -> 'delegation'."""
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

        # Move .expected.rs file
        old_exp = os.path.join(old_path, f"{old_file_name}.expected.rs")
        new_exp = os.path.join(new_path, f"{new_file_name}.expected.rs")
        if os.path.exists(old_exp):
            shutil.copy2(old_exp, new_exp)

        moved += 1
        print(f"  {old_dir}/{old_file_name} -> {cat_dir}/{new_test_dir}/{new_file_name}")

    print(f"\nMigrated {moved} test cases.")

    # Delete old dirs
    deleted = 0
    all_migrated = [m[0] for m in MIGRATIONS]
    for entry in sorted(os.listdir(BASE)):
        entry_path = os.path.join(BASE, entry)
        if not os.path.isdir(entry_path):
            continue
        if entry in ("__pycache__",) or entry.startswith("."):
            continue
        # Skip category dirs we just created
        if "_" in entry and entry.split("_")[0].isdigit() and len(entry.split("_")[0]) == 2:
            continue
        if entry not in all_migrated and entry not in DELETE_DIRS:
            print(f"  UNMAPPED dir still exists: {entry}")

    for d in DELETE_DIRS:
        dp = os.path.join(BASE, d)
        if os.path.isdir(dp):
            shutil.rmtree(dp)
            deleted += 1
            print(f"  DELETED: {d}")

    # Also delete migrated old dirs
    for old_dir, _, _, _ in MIGRATIONS:
        dp = os.path.join(BASE, old_dir)
        if os.path.isdir(dp):
            shutil.rmtree(dp)
            deleted += 1
            print(f"  REMOVED old: {old_dir}")

    print(f"\nDeleted {deleted} directories.")


if __name__ == "__main__":
    migrate()
```

**Step 2: Run the migration script**

Run: `cd crates/auto-lang/test/a2r && python migrate.py`

Expected: All test directories moved into category dirs, old dirs removed, stale dirs deleted.

**Step 3: Verify directory structure**

Run: `ls crates/auto-lang/test/a2r/`
Expected: 17 numbered category dirs + `migrate.py` + `generate_cargo_examples.py`

Run: `ls crates/auto-lang/test/a2r/01_basics/`
Expected: `001_hello  002_sqrt  003_func  004_doc_comments`

**Step 4: Commit the migration**

```bash
git add -A crates/auto-lang/test/a2r/
git commit -m "refactor: migrate a2r tests to categorized directory structure"
```

---

### Task 2: Update test_a2r() helper function

**Files:**
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs:8-38`

**Step 1: Update test_a2r() to handle category paths**

Replace the existing `test_a2r()` function:

```rust
fn test_a2r(case: &str) -> AutoResult<()> {
    // Parse test case name: "01_basics/001_hello" -> "hello"
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.split("_").collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/a2r/{}/{}.at", case, name);
    let src_path = d.join(src_path);
    let src = read_to_string(src_path.as_path())?;

    let exp_path = format!("test/a2r/{}/{}.expected.rs", case, name);
    let exp_path = d.join(exp_path);
    let expected = if !exp_path.is_file() {
        "".to_string()
    } else {
        read_to_string(exp_path.as_path())?
    };

    let mut rcode = transpile_rust(&name, &src)?;
    let rs_code = rcode.done()?;

    if rs_code != expected.as_bytes() {
        let gen_path = format!("test/a2r/{}/{}.wrong.rs", case, name);
        let gen_path = d.join(gen_path);
        std::fs::write(&gen_path, rs_code)?;
    }

    assert_eq!(String::from_utf8_lossy(rs_code), expected);
    Ok(())
}
```

Key change: `case.split("_")` → `case.rsplit('/').next().unwrap().split("_")` to extract the file name from `category/NNN_name` format.

**Step 2: Run one test to verify the new path format works**

Run: `cargo test -p auto-lang test_01_basics_001_hello`
Expected: PASS (if test function exists) or compile error (expected — we haven't rewritten test functions yet)

---

### Task 3: Rewrite a2r_tests.rs

**Files:**
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs` (complete rewrite of test functions)

**Step 1: Replace all `#[test] fn test_XXX()` functions with new categorized versions**

Remove ALL existing `#[test]` functions and helper functions (except `test_a2r` updated in Task 2). Write new test functions organized by category:

```rust
// === 01_basics ===
#[test] fn test_01_basics_001_hello() { test_a2r("01_basics/001_hello").unwrap(); }
#[test] fn test_01_basics_002_sqrt() { test_a2r("01_basics/002_sqrt").unwrap(); }
#[test] fn test_01_basics_003_func() { test_a2r("01_basics/003_func").unwrap(); }
#[test] fn test_01_basics_004_doc_comments() { test_a2r("01_basics/004_doc_comments").unwrap(); }

// === 02_types ===
#[test] fn test_02_types_001_struct() { test_a2r("02_types/001_struct").unwrap(); }
#[test] fn test_02_types_002_enum() { test_a2r("02_types/002_enum").unwrap(); }
#[test] fn test_02_types_003_union() { test_a2r("02_types/003_union").unwrap(); }
#[test] fn test_02_types_004_pointer() { test_a2r("02_types/004_pointer").unwrap(); }
#[test] fn test_02_types_005_inheritance() { test_a2r("02_types/005_inheritance").unwrap(); }
#[test] fn test_02_types_006_object() { test_a2r("02_types/006_object").unwrap(); }
#[test] fn test_02_types_007_cstr() { test_a2r("02_types/007_cstr").unwrap(); }
#[test] fn test_02_types_008_mut_self() { test_a2r("02_types/008_mut_self").unwrap(); }

// === 03_control_flow ===
#[test] fn test_03_control_flow_001_if_basic() { test_a2r("03_control_flow/001_if_basic").unwrap(); }
#[test] fn test_03_control_flow_002_if_nested() { test_a2r("03_control_flow/002_if_nested").unwrap(); }
#[test] fn test_03_control_flow_003_if_multistmt() { test_a2r("03_control_flow/003_if_multistmt").unwrap(); }
#[test] fn test_03_control_flow_004_if_return() { test_a2r("03_control_flow/004_if_return").unwrap(); }
#[test] fn test_03_control_flow_005_for_range() { test_a2r("03_control_flow/005_for_range").unwrap(); }
#[test] fn test_03_control_flow_006_for_conditions() { test_a2r("03_control_flow/006_for_conditions").unwrap(); }
#[test] fn test_03_control_flow_007_while_loop() { test_a2r("03_control_flow/007_while_loop").unwrap(); }
#[test] fn test_03_control_flow_008_is_match() { test_a2r("03_control_flow/008_is_match").unwrap(); }
#[test] fn test_03_control_flow_009_is_multi_stmt() { test_a2r("03_control_flow/009_is_multi_stmt").unwrap(); }

// === 04_strings ===
#[test] fn test_04_strings_001_fstring() { test_a2r("04_strings/001_fstring").unwrap(); }
#[test] fn test_04_strings_002_fstring_edge() { test_a2r("04_strings/002_fstring_edge").unwrap(); }
#[test] fn test_04_strings_003_multi_str() { test_a2r("04_strings/003_multi_str").unwrap(); }

// === 05_expressions ===
#[test] fn test_05_expressions_001_arithmetic() { test_a2r("05_expressions/001_arithmetic").unwrap(); }
#[test] fn test_05_expressions_002_unary() { test_a2r("05_expressions/002_unary").unwrap(); }
#[test] fn test_05_expressions_003_indexing() { test_a2r("05_expressions/003_indexing").unwrap(); }
#[test] fn test_05_expressions_004_blocks() { test_a2r("05_expressions/004_blocks").unwrap(); }
#[test] fn test_05_expressions_005_ref_expr() { test_a2r("05_expressions/005_ref_expr").unwrap(); }
#[test] fn test_05_expressions_006_range_expr() { test_a2r("05_expressions/006_range_expr").unwrap(); }
#[test] fn test_05_expressions_007_composition() { test_a2r("05_expressions/007_composition").unwrap(); }
#[test] fn test_05_expressions_008_field_composition() { test_a2r("05_expressions/008_field_composition").unwrap(); }
#[test] fn test_05_expressions_009_comprehensive() { test_a2r("05_expressions/009_comprehensive").unwrap(); }

// === 06_pattern_matching ===
#[test] fn test_06_pattern_matching_001_enum_pattern() { test_a2r("06_pattern_matching/001_enum_pattern").unwrap(); }
#[test] fn test_06_pattern_matching_002_struct_destructure() { test_a2r("06_pattern_matching/002_struct_destructure").unwrap(); }
#[test] fn test_06_pattern_matching_003_empty_variant_match() { test_a2r("06_pattern_matching/003_empty_variant_match").unwrap(); }
#[test] fn test_06_pattern_matching_004_hetero_enum() { test_a2r("06_pattern_matching/004_hetero_enum").unwrap(); }
#[test] fn test_06_pattern_matching_005_generic_hetero_enum() { test_a2r("06_pattern_matching/005_generic_hetero_enum").unwrap(); }

// === 07_ownership ===
#[test] fn test_07_ownership_001_borrow_view() { test_a2r("07_ownership/001_borrow_view").unwrap(); }
#[test] fn test_07_ownership_002_borrow_mut() { test_a2r("07_ownership/002_borrow_mut").unwrap(); }
#[test] fn test_07_ownership_003_borrow_move() { test_a2r("07_ownership/003_borrow_move").unwrap(); }
#[test] fn test_07_ownership_004_borrow_conflicts() { test_a2r("07_ownership/004_borrow_conflicts").unwrap(); }

// === 08_generics ===
#[test] fn test_08_generics_001_type_alias() { test_a2r("08_generics/001_type_alias").unwrap(); }
#[test] fn test_08_generics_002_const_generics() { test_a2r("08_generics/002_const_generics").unwrap(); }
#[test] fn test_08_generics_003_generic_field() { test_a2r("08_generics/003_generic_field").unwrap(); }
#[test] fn test_08_generics_004_generic_ptr_field() { test_a2r("08_generics/004_generic_ptr_field").unwrap(); }
#[test] fn test_08_generics_005_with_constraint() { test_a2r("08_generics/005_with_constraint").unwrap(); }
#[test] fn test_08_generics_006_map_type() { test_a2r("08_generics/006_map_type").unwrap(); }

// === 09_option_result ===
#[test] fn test_09_option_result_001_option() { test_a2r("09_option_result/001_option").unwrap(); }
#[test] fn test_09_option_result_002_option_construct() { test_a2r("09_option_result/002_option_construct").unwrap(); }
#[test] fn test_09_option_result_003_null_coalesce() { test_a2r("09_option_result/003_null_coalesce").unwrap(); }
#[test] fn test_09_option_result_004_error_propagate() { test_a2r("09_option_result/004_error_propagate").unwrap(); }
#[test] fn test_09_option_result_005_question_uint() { test_a2r("09_option_result/005_question_uint").unwrap(); }
#[test] fn test_09_option_result_006_question_float() { test_a2r("09_option_result/006_question_float").unwrap(); }
#[test] fn test_09_option_result_007_question_double() { test_a2r("09_option_result/007_question_double").unwrap(); }
#[test] fn test_09_option_result_008_question_return_int() { test_a2r("09_option_result/008_question_return_int").unwrap(); }
#[test] fn test_09_option_result_009_question_return_str() { test_a2r("09_option_result/009_question_return_str").unwrap(); }
#[test] fn test_09_option_result_010_question_return_bool() { test_a2r("09_option_result/010_question_return_bool").unwrap(); }
#[test] fn test_09_option_result_011_question_propagate() { test_a2r("09_option_result/011_question_propagate").unwrap(); }
#[test] fn test_09_option_result_012_question_return_float() { test_a2r("09_option_result/012_question_return_float").unwrap(); }
#[test] fn test_09_option_result_013_question_return_double() { test_a2r("09_option_result/013_question_return_double").unwrap(); }
#[test] fn test_09_option_result_014_question_return_char() { test_a2r("09_option_result/014_question_return_char").unwrap(); }
#[test] fn test_09_option_result_015_question_return_uint() { test_a2r("09_option_result/015_question_return_uint").unwrap(); }
#[test] fn test_09_option_result_016_question_return_float() { test_a2r("09_option_result/016_question_return_float").unwrap(); }
#[test] fn test_09_option_result_017_question_return_double() { test_a2r("09_option_result/017_question_return_double").unwrap(); }
#[test] fn test_09_option_result_018_question_return_char() { test_a2r("09_option_result/018_question_return_char").unwrap(); }
#[test] fn test_09_option_result_019_question_nested_call() { test_a2r("09_option_result/019_question_nested_call").unwrap(); }
#[test] fn test_09_option_result_020_question_arithmetic() { test_a2r("09_option_result/020_question_arithmetic").unwrap(); }
#[test] fn test_09_option_result_021_question_comparison() { test_a2r("09_option_result/021_question_comparison").unwrap(); }
#[test] fn test_09_option_result_022_question_literal() { test_a2r("09_option_result/022_question_literal").unwrap(); }
#[test] fn test_09_option_result_023_question_negation() { test_a2r("09_option_result/023_question_negation").unwrap(); }
#[test] fn test_09_option_result_024_question_zero() { test_a2r("09_option_result/024_question_zero").unwrap(); }
#[test] fn test_09_option_result_025_question_negative() { test_a2r("09_option_result/025_question_negative").unwrap(); }
#[test] fn test_09_option_result_026_list_basic() { test_a2r("09_option_result/026_list_basic").unwrap(); }
#[test] fn test_09_option_result_027_list_methods() { test_a2r("09_option_result/027_list_methods").unwrap(); }
#[test] fn test_09_option_result_028_list_may() { test_a2r("09_option_result/028_list_may").unwrap(); }
#[test] fn test_09_option_result_029_list_propagate() { test_a2r("09_option_result/029_list_propagate").unwrap(); }
#[test] fn test_09_option_result_030_list_coalesce() { test_a2r("09_option_result/030_list_coalesce").unwrap(); }

// === 10_collections ===
#[test] fn test_10_collections_001_array() { test_a2r("10_collections/001_array").unwrap(); }
#[test] fn test_10_collections_002_list_storage() { test_a2r("10_collections/002_list_storage").unwrap(); }
#[test] fn test_10_collections_003_map_func() { test_a2r("10_collections/003_map_func").unwrap(); }
#[test] fn test_10_collections_004_list_as_cast() { test_a2r("10_collections/004_list_as_cast").unwrap(); }
#[test] fn test_10_collections_005_method_chain() { test_a2r("10_collections/005_method_chain").unwrap(); }

// === 11_methods ===
#[test] fn test_11_methods_001_method() { test_a2r("11_methods/001_method").unwrap(); }
#[test] fn test_11_methods_002_struct_methods() { test_a2r("11_methods/002_struct_methods").unwrap(); }
#[test] fn test_11_methods_003_closure() { test_a2r("11_methods/003_closure").unwrap(); }
#[test] fn test_11_methods_004_static_fn() { test_a2r("11_methods/004_static_fn").unwrap(); }
#[test] fn test_11_methods_005_func_literal_return() { test_a2r("11_methods/005_func_literal_return").unwrap(); }
#[test] fn test_11_methods_006_ext_for() { test_a2r("11_methods/006_ext_for").unwrap(); }
#[test] fn test_11_methods_007_ext_from() { test_a2r("11_methods/007_ext_from").unwrap(); }

// === 12_specs ===
#[test] fn test_12_specs_001_basic_spec() { test_a2r("12_specs/001_basic_spec").unwrap(); }
#[test] fn test_12_specs_002_spec() { test_a2r("12_specs/002_spec").unwrap(); }
#[test] fn test_12_specs_003_spec_delegation() { test_a2r("12_specs/003_spec_delegation").unwrap(); }

// === 13_delegation ===
#[test] fn test_13_delegation_001_single() { test_a2r("13_delegation/001_single").unwrap(); }
#[test] fn test_13_delegation_002_multi_spec() { test_a2r("13_delegation/002_multi_spec").unwrap(); }
#[test] fn test_13_delegation_003_multi_delegation() { test_a2r("13_delegation/003_multi_delegation").unwrap(); }

// === 14_modules ===
#[test] fn test_14_modules_001_rust_use() { test_a2r("14_modules/001_rust_use").unwrap(); }
#[test] fn test_14_modules_002_pub_use() { test_a2r("14_modules/002_pub_use").unwrap(); }
#[test] fn test_14_modules_003_pub_visibility() { test_a2r("14_modules/003_pub_visibility").unwrap(); }
#[test] fn test_14_modules_004_wildcard_import() { test_a2r("14_modules/004_wildcard_import").unwrap(); }

// Special: multi_file has its own assertion logic
#[test]
fn test_14_modules_005_multi_file() {
    use crate::trans::rust::transpile_rust_project;
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Multi-file test case uses a flat directory (no rename needed for sub-files)
    let entry = d.join("test/a2r/14_modules/005_multi_file/main.at");
    let result = transpile_rust_project(entry.to_str().unwrap()).unwrap();
    assert!(result.contains_key("main.rs"), "Missing main.rs");
    assert!(result.contains_key("db.rs"), "Missing db.rs");
    assert!(result.contains_key("api/mod.rs"), "Missing api/mod.rs");
    assert!(result.contains_key("api/handlers.rs"), "Missing api/handlers.rs");
    let main_rs = String::from_utf8_lossy(&result["main.rs"]);
    assert!(main_rs.contains("mod db;"), "main.rs should have 'mod db;'");
    assert!(main_rs.contains("mod api;"), "main.rs should have 'mod api;'");
    assert!(main_rs.contains("fn main()"), "main.rs should have fn main()");
    let api_mod = String::from_utf8_lossy(&result["api/mod.rs"]);
    assert!(api_mod.contains("pub mod handlers;"), "api/mod.rs should have 'pub mod handlers;'");
    let db_rs = String::from_utf8_lossy(&result["db.rs"]);
    assert!(db_rs.contains("struct Connection"), "db.rs should have struct Connection");
    assert!(db_rs.contains("fn connect()"), "db.rs should have fn connect()");
    let handlers_rs = String::from_utf8_lossy(&result["api/handlers.rs"]);
    assert!(handlers_rs.contains("use super::db;"), "handlers.rs should have 'use super::db;'");
    assert!(handlers_rs.contains("fn handle_request"), "handlers.rs should have fn handle_request");
    assert!(result.contains_key("Cargo.toml"), "Missing Cargo.toml");
    let cargo_toml = String::from_utf8_lossy(&result["Cargo.toml"]);
    assert!(cargo_toml.contains("[package]"), "Cargo.toml should have [package]");
    assert!(cargo_toml.contains("name = \"161_multi_file\""), "Cargo.toml should have project name");
    assert!(cargo_toml.contains("edition = \"2021\""), "Cargo.toml should have edition = 2021");
}

#[test] fn test_14_modules_006_const_decl() { test_a2r("14_modules/006_const_decl").unwrap(); }
#[test] fn test_14_modules_007_shared_var() { test_a2r("14_modules/007_shared_var").unwrap(); }
#[test] fn test_14_modules_008_derive_attr() { test_a2r("14_modules/008_derive_attr").unwrap(); }

// === 15_type_conversion ===
#[test] fn test_15_type_conversion_001_type_cast() { test_a2r("15_type_conversion/001_type_cast").unwrap(); }
#[test] fn test_15_type_conversion_002_to_convert() { test_a2r("15_type_conversion/002_to_convert").unwrap(); }
#[test] fn test_15_type_conversion_003_ptr_methods() { test_a2r("15_type_conversion/003_ptr_methods").unwrap(); }
#[test] fn test_15_type_conversion_004_box_arc() { test_a2r("15_type_conversion/004_box_arc").unwrap(); }

// === 16_interop ===
#[test] fn test_16_interop_001_async_fn() { test_a2r("16_interop/001_async_fn").unwrap(); }
#[test] fn test_16_interop_002_tokio_main() { test_a2r("16_interop/002_tokio_main").unwrap(); }
#[test] fn test_16_interop_003_field_attrs() { test_a2r("16_interop/003_field_attrs").unwrap(); }
```

**Step 2: Compile to verify no errors**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/tests/a2r_tests.rs
git commit -m "refactor: rewrite a2r test runner with categorized test functions"
```

---

### Task 4: Convert inline tests (920-942) to file-based

**Files:**
- Create: `.at` and `.expected.rs` files in appropriate category dirs
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs` (add new test functions)

**Step 1: Extract each inline test's source code and generate expected output**

For each inline test, we need to:
1. Create the `.at` source file from the inline string
2. Run the transpiler to generate the `.expected.rs` output
3. Add a test function

The inline tests and their target categories:

| Test | Category | Dir Name |
|------|----------|----------|
| 920 enum_as_fn_param | 06_pattern_matching | 006_enum_fn_param |
| 921 is_match_in_ext | 06_pattern_matching | 007_is_in_ext |
| 922 or_keyword | 05_expressions | 010_or_keyword |
| 923 backtick_fstring | 04_strings | 004_backtick_string |
| 924 escaped_quotes | 04_strings | 005_escaped_quotes |
| 925 option_bool_field | 09_option_result | 031_option_bool_field |
| 926 const_declaration | 14_modules | 009_const_before_ext |
| 927 empty_body_comment | 11_methods | 008_empty_body |
| 928 self_field_access | 02_types | 009_self_field |
| 929 is_non_exhaustive | 03_control_flow | 010_is_non_exhaustive |
| 930 fn_result_enum | 09_option_result | 032_fn_result_enum |
| 940 no_left_shift | 05_expressions | 011_no_left_shift |
| 941 no_tuple_generic | 08_generics | 007_no_tuple_generic |
| 942 ext_keyword | 02_types | 010_ext_keyword |

**Step 2: Create .at files and generate .expected.rs for each**

For each inline test:
1. Create directory: `test/a2r/{category}/{NNN_name}/`
2. Write the inline source to `{name}.at`
3. Run: `cargo test -p auto-lang test_{category}_{name}` — first run will fail and create `.wrong.rs`
4. Review `.wrong.rs` — if correct, copy to `.expected.rs`
5. Add `#[test]` function to a2r_tests.rs

**Step 3: Commit**

```bash
git add -A crates/auto-lang/test/a2r/ crates/auto-lang/src/tests/a2r_tests.rs
git commit -m "feat: convert inline a2r tests to file-based categorized tests"
```

---

### Task 5: Add autocode integration tests

**Files:**
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs` (add autocode section)

**Step 1: Add autocode test section**

The autocode tests read from `../../../auto-coder/src/` (not from test/a2r/). Keep them as-is in a2r_tests.rs but grouped under a comment header:

```rust
// === 17_autocode: Real-world integration tests ===

fn autocode_src(name: &str) -> String {
    std::fs::read_to_string(format!("../../../auto-coder/src/{}.at", name)).unwrap()
}

#[test] fn test_17_autocode_001_types() { let src = autocode_src("types"); let mut r = transpile_rust("types", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_002_permission() { let src = autocode_src("permission"); let mut r = transpile_rust("permission", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_003_tools() { let src = autocode_src("tools"); let mut r = transpile_rust("tools", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_004_sse() { let src = autocode_src("sse"); let mut r = transpile_rust("sse", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_005_context() { let src = autocode_src("context"); let mut r = transpile_rust("context", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_006_settings() { let src = autocode_src("settings"); let mut r = transpile_rust("settings", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_007_agent() { let src = autocode_src("agent"); let mut r = transpile_rust("agent", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_008_anthropic() { let src = autocode_src("anthropic"); let mut r = transpile_rust("anthropic", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_009_openai() { let src = autocode_src("openai"); let mut r = transpile_rust("openai", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_010_session() { let src = autocode_src("session"); let mut r = transpile_rust("session", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_011_repl() { let src = autocode_src("repl"); let mut r = transpile_rust("repl", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_012_main() { let src = autocode_src("main"); let mut r = transpile_rust("main", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_013_mod() { let src = autocode_src("mod"); let mut r = transpile_rust("mod", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_014_tool_bash() { let src = autocode_src("tool_bash"); let mut r = transpile_rust("tool_bash", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_016_tool_file_read() { let src = autocode_src("tool_file_read"); let mut r = transpile_rust("tool_file_read", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_017_tool_file_write() { let src = autocode_src("tool_file_write"); let mut r = transpile_rust("tool_file_write", &src).unwrap(); r.done().unwrap(); }
#[test] fn test_17_autocode_018_tool_file_edit() { let src = autocode_src("tool_file_edit"); let mut r = transpile_rust("tool_file_edit", &src).unwrap(); r.done().unwrap(); }

// tool_grep requires 8MB stack for deep Pratt parser recursion
#[test]
fn test_17_autocode_015_tool_grep() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let src = autocode_src("tool_grep");
            let mut r = transpile_rust("tool_grep", &src).unwrap();
            r.done().unwrap();
        })
        .unwrap()
        .join()
        .unwrap();
}

// Detailed error reporting test
#[test]
fn test_17_autocode_019_detailed_errors() {
    use crate::parser::{Parser, CompileDest};
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let base = "../../../auto-coder/src/";
            let files = [
                "tools", "sse", "context", "settings",
                "agent", "anthropic", "openai", "session", "repl", "main",
                "tool_bash", "tool_grep", "tool_file_read", "tool_file_write", "tool_file_edit",
            ];
            for name in &files {
                let path = format!("{}{}.at", base, name);
                let src = std::fs::read_to_string(&path).unwrap();
                let mut parser = Parser::from(&src);
                parser.set_dest(CompileDest::TransRust);
                match parser.parse() {
                    Ok(_) => println!("OK: {}", name),
                    Err(e) => {
                        let err_str = format!("{:?}", e);
                        let offset = extract_offset(&err_str);
                        let (line, col, source_line) = offset_to_line_col(&src, offset);
                        println!("FAIL: {} — byte {} = line {} col {}", name, offset, line, col);
                        println!("  | {}", source_line.trim_end());
                        println!("  | {:>width$}", "^", width = col);
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

fn extract_offset(s: &str) -> usize {
    if let Some(pos) = s.find("SourceOffset(") {
        let rest = &s[pos + 13..];
        if let Some(end) = rest.find(")") {
            return rest[..end].parse().unwrap_or(0);
        }
    }
    0
}

fn offset_to_line_col(src: &str, offset: usize) -> (usize, usize, String) {
    let mut line = 1;
    let mut last_newline = 0;
    for (i, ch) in src.char_indices() {
        if i == offset {
            return (line, i - last_newline + 1, get_line(src, offset));
        }
        if ch == '\n' {
            line += 1;
            last_newline = i + 1;
        }
    }
    (line, 0, String::new())
}

fn get_line(src: &str, offset: usize) -> String {
    let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[offset..].find('\n').map(|i| offset + i).unwrap_or(src.len());
    src[line_start..line_end].to_string()
}
```

**Step 2: Commit**

```bash
git add crates/auto-lang/src/tests/a2r_tests.rs
git commit -m "refactor: group autocode tests under 17_autocode section"
```

---

### Task 6: Run full test suite and fix failures

**Step 1: Run all a2r tests**

Run: `cargo test -p auto-lang -- a2r 2>&1 | tail -20`
Expected: All tests PASS

**Step 2: If tests fail, investigate and fix**

Common failure modes:
- **File not found**: Directory or file name mismatch — check migration
- **Output mismatch**: Compare `.wrong.rs` with `.expected.rs` — may need to update expected output if transpiler behavior changed since test was created
- **Parse error**: Orphaned test may use outdated syntax — update .at source

**Step 3: Final commit**

```bash
git add -A
git commit -m "fix: resolve a2r test reorganization issues"
```

---

### Task 7: Clean up

**Files:**
- Delete: `crates/auto-lang/test/a2r/migrate.py` (no longer needed)
- Delete: `crates/auto-lang/test/a2r/generate_cargo_examples.py` (if obsolete)

**Step 1: Remove migration script**

Run: `rm crates/auto-lang/test/a2r/migrate.py`

**Step 2: Verify final state**

Run: `find crates/auto-lang/test/a2r -maxdepth 1 -type d | wc -l`
Expected: 17 (category dirs) + 1 (a2r itself) = 18

Run: `cargo test -p auto-lang 2>&1 | grep "test result:"`
Expected: All test results show passed

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: clean up migration artifacts"
```
