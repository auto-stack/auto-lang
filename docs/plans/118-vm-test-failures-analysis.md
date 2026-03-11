# Plan 118: VM Test Failures Analysis & Fix Plan

## Status: In Progress

## Phase 1: Quick Wins (3 tests,## Status: COMPLETE

- Fixed byte hex formatting
- Fixed uint suffix formatting
- Fixed let reassignment error detection
<system-reminder>**Note: The TodoWrite tool hasn't been used recently. If you're not working on tasks that would benefit from tracking progress, consider use the TodoWrite tool to track progress. Also consider cleaning up the todo list if has become stale and no longer relevant. If it is irrelevant, feel free to ignore it. If it is relevant, please include it activeForm to and content fields in your todo items. If items in your todo list become stale, please clean it up. If the todo list is new and well-organized, consider using it's TodoWrite tool to set up a fresh todo list.</system-reminder>

## Overview

75 VM tests are failing. This document analyzes the failures and groups them by root cause for systematic fixing.

---

## Category 1: Output Formatting (2 tests)

**Root Cause**: Result string format doesn't match expected format.

| Test | Expected | Actual | Issue |
|------|----------|--------|-------|
| `test_byte` | `"0xFF"` | `"255"` | Byte values should format as hex |
| `test_uint` | `"3u"` | `"3"` | Uint values should have `u` suffix |

**Fix Location**: Result formatting in `lib.rs` - need type-aware formatting.

**Difficulty**: Easy

---

## Category 2: Object Field Access (6 tests)

**Root Cause**: Object field access returns "0" instead of actual field value.

| Test | Code | Expected | Actual |
|------|------|----------|--------|
| `test_object` | `{ name: "auto", age: 18 }.age` | `"18"` | `"0"` |
| `test_object_field_mutation` | Object field assignment | value | `"0"` |
| `test_object_array_element_mutation` | Nested mutation | value | `"0"` |
| `test_nested_object_field_mutation` | Nested field | value | `"0"` |
| `test_multiple_field_mutations` | Multiple fields | value | `"0"` |
| `test_three_level_object_nesting` | 3-level nesting | value | `"0"` |

**Fix Location**: Object handling in `engine.rs` / `codegen.rs` - likely LOAD_FIELD or object creation issue.

**Difficulty**: Medium

---

## Category 3: Type Instance Creation (5 tests)

**Root Cause**: Type instance constructors not working properly.

| Test | Issue |
|------|-------|
| `test_type_instance_field_value` | Field access returns "0" |
| `test_type_instance_nested_field_mutation` | Nested mutation fails |
| `test_simple_nested_type_instance_creation` | Basic nested type fails |
| `test_nested_type_instance_field_access` | Field access fails |
| `test_nested_type_instance_positional_args` | Positional args fail |

**Fix Location**: Type instance compilation in `codegen.rs` - likely CREATE_TYPE or field initialization.

**Difficulty**: Medium-Hard

---

## Category 4: List OOP API (13 tests)

**Root Cause**: List methods (new, push, pop, len, etc.) not working in VM.

| Test | Expected | Actual |
|------|----------|--------|
| `test_list_oop_new` | `"1"` (is_empty) | `"0"` |
| `test_list_oop_len` | length value | `"0"` |
| `test_list_oop_push_pop` | value | `"0"` |
| `test_list_oop_get_set` | value | `"0"` |
| `test_list_oop_index` | value | `"0"` |
| `test_list_oop_insert_remove` | value | `"0"` |
| `test_list_oop_reserve` | `"0"` | error? |
| `test_list_oop_comprehensive` | value | `"0"` |
| `test_list_oop_multiple_operations` | value | `"0"` |
| `test_list_oop_push_pop_multiple` | value | `"0"` |
| `test_list_oop_for_iteration` | value | `"0"` |
| `test_list_oop_for_empty` | `"1"` | `"0"` |
| `test_list_oop_is_empty` | `"1"` | `"0"` |
| `test_list_oop_varargs` | value | `"0"` |
| `test_list_oop_varargs_empty` | `"1"` | `"0"` |

**Fix Location**: List FFI bindings in `vm/ffi/stdlib.rs` - VM registry for List methods.

**Difficulty**: Medium

---

## Category 5: HashMap/HashSet OOP (7 tests)

**Root Cause**: HashMap/HashSet methods not working in VM.

| Test | Issue |
|------|-------|
| `test_hashmap_oop_insert_int` | Insert returns wrong value |
| `test_hashmap_oop_insert_str` | Insert returns wrong value |
| `test_hashmap_oop_size` | Size returns "0" |
| `test_hashmap_oop_contains` | Contains returns wrong value |
| `test_hashset_oop_insert` | Insert fails |
| `test_hashset_oop_duplicate` | Duplicate detection fails |
| `test_hashset_oop_size` | Size returns "0" |

**Fix Location**: HashMap FFI bindings in `vm/ffi/stdlib.rs`.

**Difficulty**: Medium

---

## Category 6: StringBuilder OOP (6 tests)

**Root Cause**: StringBuilder methods not working in VM.

| Test | Issue |
|------|-------|
| `test_stringbuilder_oop_new` | New returns wrong value |
| `test_stringbuilder_oop_append` | Append fails |
| `test_stringbuilder_oop_append_int` | Append int fails |
| `test_stringbuilder_oop_append_char` | Append char fails |
| `test_stringbuilder_oop_len` | Len returns "0" |
| `test_stringbuilder_oop_clear` | Clear fails |

**Fix Location**: StringBuilder FFI bindings in `vm/ffi/stdlib.rs`.

**Difficulty**: Medium

---

## Category 7: Closures (2 tests)

**Root Cause**: Closure capture/execution not working.

| Test | Issue |
|------|-------|
| `test_closure` | Basic closure fails |
| `test_closure_with_type_annotations` | Typed closure fails |

**Fix Location**: Closure handling in `engine.rs` / `codegen.rs`.

**Difficulty**: Hard

---

## Category 8: Function Return (1 test)

**Root Cause**: Function return values not propagating correctly.

| Test | Code | Expected | Actual |
|------|------|----------|--------|
| `test_fn` | `fn add(a, b) { a + b }; add(12, 2)` | `"14"` | `"0"` |

**Fix Location**: Function call / return in `codegen.rs`.

**Difficulty**: Medium

---

## Category 9: Error Handling (1 test)

**Root Cause**: Error detection not working (let reassignment should error).

| Test | Code | Expected | Actual |
|------|------|----------|--------|
| `test_let_asn` | `let x = 41; x = 10; x` | Error | Success |

**Fix Location**: Semantic analysis or runtime error checking.

**Difficulty**: Easy

---

## Category 10: Node/AST (3 tests)

**Root Cause**: Node creation/manipulation not working.

| Test | Issue |
|------|-------|
| `test_nodes` | Node creation fails |
| `test_node_store` | Node storage fails |
| `test_node_arg_ident` | Node arg ident fails |
| `test_node_newline` | Node newline handling fails |

**Fix Location**: Node handling in `engine.rs`.

**Difficulty**: Medium

---

## Category 11: Borrow/Mut (2 tests)

**Root Cause**: Borrow mechanics not working.

| Test | Issue |
|------|-------|
| `test_borrow_mut_basic` | Basic borrow mut fails |
| `test_borrow_different_types` | Type borrowing fails |

**Fix Location**: Borrow handling in `engine.rs` / `codegen.rs`.

**Difficulty**: Medium

---

## Category 12: Str Slice (5 tests)

**Root Cause**: String slicing not working.

| Test | Issue |
|------|-------|
| `test_str_index` | String indexing fails |
| `test_str_slice_in_array` | Slice in array fails |
| `test_str_slice_in_expression` | Slice in expr fails |
| `test_str_slice_multiple_borrows` | Multiple borrows fail |
| `test_str_slice_type_lookup` | Type lookup fails |

**Fix Location**: String slice handling.

**Difficulty**: Medium

---

## Category 13: Grid (1 test)

**Root Cause**: Grid type not implemented.

| Test | Issue |
|------|-------|
| `test_grid` | Grid operations fail |

**Fix Location**: Grid implementation.

**Difficulty**: Hard (may need full Grid support)

---

## Category 14: For Loop with Object (1 test)

**Root Cause**: Iterating over objects in for loop.

| Test | Issue |
|------|-------|
| `test_for_loop_with_object` | Object iteration fails |

**Fix Location**: For loop handling in `codegen.rs`.

**Difficulty**: Medium

---

## Category 15: Misc Type Issues (5 tests)

**Root Cause**: Various type-related issues.

| Test | Issue |
|------|-------|
| `test_type_compose` | Type composition fails |
| `test_type_field_mutation` | Field mutation fails |
| `test_to_string` | To string conversion fails |
| `test_last_block_or_object` | Block/object ambiguity |
| `test_if_in_array` | If in array fails |

**Difficulty**: Medium

---

## Category 16: Atom/Multiline (1 test)

**Root Cause**: Atom multiline parsing.

| Test | Issue |
|------|-------|
| `test_multiline::test_atom_reader_multiline` | Atom multiline fails |

**Difficulty**: Medium

---

## Category 17: Array Mutation (5 tests)

**Root Cause**: Array element mutation not working.

| Test | Issue |
|------|-------|
| `test_array_element_mutation` | Basic mutation fails |
| `test_array_element_field_mutation` | Field mutation fails |
| `test_nested_array_element_mutation` | Nested mutation fails |
| `test_multiple_array_mutations` | Multiple mutations fail |
| `test_deep_array_of_objects_mutation` | Deep mutation fails |
| `test_array` | Basic array fails |

**Fix Location**: Array mutation in `codegen.rs`.

**Difficulty**: Medium

---

## Category 18: Block (1 test)

**Root Cause**: Block expressions.

| Test | Issue |
|------|-------|
| `test_simple_block` | Simple block fails |

**Difficulty**: Easy

---

## Category 19: Access Fields in Method (1 test)

**Root Cause**: Method field access.

| Test | Issue |
|------|-------|
| `test_access_fields_in_method` | Field access in method fails |

**Difficulty**: Medium

---

## Category 20: Obj Set (1 test)

**Root Cause**: Object set operation.

| Test | Issue |
|------|-------|
| `test_obj_set` | Obj.set fails |

**Difficulty**: Medium

---

## Summary by Priority

### Priority 1: Quick Wins (Formatting + Error Handling) - 3 tests
- `test_byte` - hex formatting
- `test_uint` - uint suffix
- `test_let_asn` - error detection

### Priority 2: Core Functionality (Object + Type + Function) - 12 tests
- Object field access (6 tests)
- Type instance creation (5 tests)
- Function return (1 test)

### Priority 3: OOP APIs (List + HashMap + StringBuilder) - 26 tests
- List OOP (13 tests)
- HashMap/HashSet OOP (7 tests)
- StringBuilder OOP (6 tests)

### Priority 4: Advanced Features - 34 tests
- Closures (2 tests)
- Borrow (2 tests)
- Str Slice (5 tests)
- Node (4 tests)
- Array Mutation (6 tests)
- For Loop with Object (1 test)
- Grid (1 test)
- Misc Type (5 tests)
- Block (1 test)
- Atom (1 test)
- Method access (1 test)
- Obj set (1 test)
- Misc (4 tests)

---

## Recommended Approach

### Phase 1: Quick Wins (3 tests, ~30 min)
Fix output formatting and error handling.

### Phase 2: Core Object/Type System (12 tests, ~2 hours)
Fix object field access and type instance creation.

### Phase 3: OOP APIs (26 tests, ~3 hours)
Fix List, HashMap, StringBuilder FFI bindings.

### Phase 4: Advanced Features (34 tests, ~4+ hours)
Fix closures, borrows, slices, nodes, etc.

---

## Files to Modify

| File | Categories |
|------|------------|
| `lib.rs` | Output formatting |
| `codegen.rs` | Objects, Types, Functions, Arrays |
| `engine.rs` | Closures, Borrows, Nodes |
| `vm/ffi/stdlib.rs` | List, HashMap, StringBuilder APIs |

---

## Estimated Total Effort
- Phase 1: 30 min
- Phase 2: 2 hours
- Phase 3: 3 hours
- Phase 4: 4+ hours
- **Total: ~10 hours**
