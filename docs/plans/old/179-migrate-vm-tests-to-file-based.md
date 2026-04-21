# Plan 179: Migrate vm_tests.rs to File-Based vm_file Tests

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate all migratable inline tests from `vm_tests.rs` (3048 lines) into file-based `test/vm/` tests using the Plan 177 framework, then slim down `vm_tests.rs` to only non-migratable tests.

**Architecture:** Each inline test becomes a `.at` source file + one of `.expected.result` / `.expected.out` / `.expected.error`. Tests are organized into numbered category directories (matching a2r pattern). Duplicate tests (script-mode vs `fn main()` mode) are consolidated into a single file-based test.

**Tech Stack:** Rust test framework, existing `test_vm()` helper from `vm_file_tests.rs`

---

## Background

### What Can Be Migrated

Tests that call `run(code)` and assert on the result string — these map directly to `.expected.result` files. Tests that assert on stdout output from `print()` map to `.expected.out`. Tests that assert `result.is_err()` map to `.expected.error`.

### What Cannot Be Migrated (stays in `vm_tests.rs`)

| Test | Reason |
|------|--------|
| `test_vm_ret_constant`, `test_vm_const_i32_add` | Direct bytecode, no source code |
| `test_vm_annotation_in_ext` | AST inspection via `Parser::from()` |
| `test_function_body_parsing` | AST inspection |
| `test_node_newline` | Uses `AutoConfig::new()`, config mode |
| `test_nodes`, `test_node_arg_ident` | Node syntax, parser-level verification |
| `test_atom_reader_multiline` | Tests `AtomReader`, not VM |
| `test_array_return_eval` | Function return types, REPL session-like |
| `test_grid`, `test_view_types`, `test_atom_query`, `test_node_store` | `#[ignore]` tests |
| All commented-out tests | Dead code |

**Total migratable: ~130 tests → ~100 file-based tests (after dedup)**

### Deduplication

Lines 2389-2836 contain `*_main` variants that wrap identical logic in `fn main() int { ... }`. These are redundant — file-based tests use `run()` which handles both script and main-function mode. We keep only the script-mode version (shorter, cleaner).

---

## Category Structure

```
test/vm/
├── 01_basics/          (exists, 3 tests → expand to 11)
├── 02_bit_ops/         (exists, 5 tests → keep)
├── 03_variables/       (new, 8 tests)
├── 04_control_flow/    (new, 7 tests)
├── 05_loops/           (new, 4 tests)
├── 06_arrays/          (new, 6 tests)
├── 07_objects/         (new, 9 tests)
├── 08_strings/         (new, 4 tests)
├── 09_functions/       (new, 9 tests)
├── 10_types/           (new, 9 tests)
├── 11_compound_ops/    (new, 6 tests)
├── 12_type_coercion/   (new, 6 tests)
├── 13_collections/     (new, 36 tests)
├── 14_borrow/          (new, 18 tests)
├── 15_nested_mutation/ (new, 14 tests)
└── 16_option_result/   (new, 16 tests)
```

---

## Task 1: Expand `01_basics` (add 8 tests)

**Files:**
- Create: `test/vm/01_basics/004_uint/uint.at` + `uint.expected.result`
- Create: `test/vm/01_basics/005_byte/byte.at` + `byte.expected.result`
- Create: `test/vm/01_basics/006_unary/unary.at` + `unary.expected.result`
- Create: `test/vm/01_basics/007_group/group.at` + `group.expected.result`
- Create: `test/vm/01_basics/008_comp/comp.at` + `comp.expected.result`
- Create: `test/vm/01_basics/009_comp_false/comp_false.at` + `comp_false.expected.result`
- Create: `test/vm/01_basics/010_eq/eq.at` + `eq.expected.result`
- Create: `test/vm/01_basics/011_eq_false/eq_false.at` + `eq_false.expected.result`

**Step 1: Create test files**

`004_uint/uint.at`:
```auto
1u + 2u
```
`004_uint/uint.expected.result`: `3u`

`005_byte/byte.at`:
```auto
let a byte = 255
a
```
`005_byte/byte.expected.result`: `0xFF`

`006_unary/unary.at`:
```auto
-2 * 3
```
`006_unary/unary.expected.result`: `-6`

`007_group/group.at`:
```auto
(1 + 2) * 3
```
`007_group/group.expected.result`: `9`

`008_comp/comp.at`:
```auto
1 < 2
```
`008_comp/comp.expected.result`: `true`

`009_comp_false/comp_false.at`:
```auto
2 < 1
```
`009_comp_false/comp_false.expected.result`: `false`

`010_eq/eq.at`:
```auto
1 == 1
```
`010_eq/eq.expected.result`: `true`

`011_eq_false/eq_false.at`:
```auto
1 == 2
```
`011_eq_false/eq_false.expected.result`: `false`

**Step 2: Add test function entries in `vm_file_tests.rs`**

```rust
// === 01_basics (continued) ===
#[test] fn test_01_basics_004_uint() { test_vm("01_basics/004_uint").unwrap(); }
#[test] fn test_01_basics_005_byte() { test_vm("01_basics/005_byte").unwrap(); }
#[test] fn test_01_basics_006_unary() { test_vm("01_basics/006_unary").unwrap(); }
#[test] fn test_01_basics_007_group() { test_vm("01_basics/007_group").unwrap(); }
#[test] fn test_01_basics_008_comp() { test_vm("01_basics/008_comp").unwrap(); }
#[test] fn test_01_basics_009_comp_false() { test_vm("01_basics/009_comp_false").unwrap(); }
#[test] fn test_01_basics_010_eq() { test_vm("01_basics/010_eq").unwrap(); }
#[test] fn test_01_basics_011_eq_false() { test_vm("01_basics/011_eq_false").unwrap(); }
```

**Step 3: Run tests**

```bash
cargo test -p auto-lang -- vm_file_tests::test_01_basics
```
Expected: All 11 tests pass.

**Step 4: Commit**

```
feat(test): expand vm 01_basics with uint, byte, unary, comparison tests
```

---

## Task 2: Create `03_variables` (8 tests)

**Files:**
- Create: 8 directories under `test/vm/03_variables/`

**Test cases:**

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | var | `var a = 1; a+2` | `3` |
| 002 | var_assign | `var a = 1; a = 2; a` | `2` |
| 003 | var_mut | `var x = 1; x = 10; x+1` | `11` |
| 004 | var_if | `var x = if true { 1 } else { 2 }; x+1` | `2` |
| 005 | let_binding | `let x = 41; x` | `41` |
| 006 | let_asn_error | `let x = 41; x = 10; x` | `.expected.error` |
| 007 | var_reassignment | `let x = 41; var x = 10; x` | `10` |
| 008 | simple_block | `let a = 10; { let a = 20 }; a` | `10` |

**Step 1:** Create all 8 test directories with `.at` and `.expected.*` files.

**Step 2:** Add test entries in `vm_file_tests.rs`:
```rust
// === 03_variables ===
#[test] fn test_03_variables_001_var() { test_vm("03_variables/001_var").unwrap(); }
#[test] fn test_03_variables_002_var_assign() { test_vm("03_variables/002_var_assign").unwrap(); }
#[test] fn test_03_variables_003_var_mut() { test_vm("03_variables/003_var_mut").unwrap(); }
#[test] fn test_03_variables_004_var_if() { test_vm("03_variables/004_var_if").unwrap(); }
#[test] fn test_03_variables_005_let_binding() { test_vm("03_variables/005_let_binding").unwrap(); }
#[test] fn test_03_variables_006_let_asn_error() { test_vm("03_variables/006_let_asn_error").unwrap(); }
#[test] fn test_03_variables_007_var_reassignment() { test_vm("03_variables/007_var_reassignment").unwrap(); }
#[test] fn test_03_variables_008_simple_block() { test_vm("03_variables/008_simple_block").unwrap(); }
```

**Step 3:** Run `cargo test -p auto-lang -- vm_file_tests::test_03_variables`

**Step 4:** Commit

```
feat(test): add vm 03_variables file-based tests
```

---

## Task 3: Create `04_control_flow` (7 tests)

**Files:**
- Create: 7 directories under `test/vm/04_control_flow/`

**Test cases:**

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | if_true | `if true { 1 } else { 2 }` | `1` |
| 002 | if_false | `if false { 1 } else { 2 }` | `2` |
| 003 | if_else_if | `if false { 1 } else if false { 2 } else { 3 }` | `3` |
| 004 | if_with_bool | `var succ = true\nif succ {\n  print("I won!")\n  "succ"\n} else {\n  print("You failed!")\n  "failed"\n}` | `succ` (result), `.expected.out`: `I won!\n` |
| 005 | if_in_array | `var is_lse = false\nvar is_rh = true\n["osal", if is_lse {"EB"}, if is_rh {"al"}]` | `["osal", "al"]` |
| 006 | is_stmt | `var x = 10\nis x {\n  10 -> {print("Here is 10!"); x}\n}` | `10` (result), `.expected.out`: `Here is 10!\n` |
| 007 | asn_upper | `var a = 1\nif true { a = 2 }\na` | `2` |

**Steps:** Same pattern as Task 2. Commit: `feat(test): add vm 04_control_flow file-based tests`

---

## Task 4: Create `05_loops` (4 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | for_range | `var sum = 0\nfor i in 0..10 { sum = sum + i }\nsum` | `45` |
| 002 | range_inclusive | `var sum = 0\nfor i in 0..=10 { sum = sum + i }\nsum` | `55` |
| 003 | range_literal | `1..5` | `1..5` |
| 004 | for_each_object | iterate array of objects + print | `.expected.out` |

`004_for_each_object/for_each_object.at`:
```auto
var items = [
    { name: "Alice", age: 20 }
    { name: "Bob", age: 21 }
    { name: "Charlie", age: 22 }
]
for item in items {
    print(f"Hi ${item.name}")
}
```
`004_for_each_object/for_each_object.expected.out`:
```
Hi Alice
Hi Bob
Hi Charlie
```

**Steps:** Same pattern. Commit: `feat(test): add vm 05_loops file-based tests`

---

## Task 5: Create `06_arrays` (5 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | array_literal | `[1, 2, 3]` | `[1, 2, 3]` |
| 002 | array_index | `var a = [1, 2, 3]\n[a[0], a[1], a[2], a[-1], a[-2], a[-3]]` | `[1, 2, 3, 3, 2, 1]` |
| 003 | array_update | `var a = [1, 2, 3]\na[0] = 4\na` | `[4, 2, 3]` |
| 004 | array_of_objects | `[1, 2]` | `[1, 2]` |
| 005 | array_multiple_mutations | multi-element update + sum | `60` |

**Steps:** Same pattern. Commit: `feat(test): add vm 06_arrays file-based tests`

---

## Task 6: Create `07_objects` (9 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | object_field | `var a = { name: "auto", age: 18 }\na.name` | `auto` |
| 002 | object_field_int_key | `var a = { 1: 2, 3: 4 }\na.3` | `4` |
| 003 | object_field_bool_key | `var a = { true: 2, false: 4 }\na.false` | `4` |
| 004 | obj_set | `var a = { name: "Alice" }\na.name = "Bob"\na.name` | `Bob` |
| 005 | nested_object | `var obj = { inner: { x: 10, y: 20 } }\nobj.inner.x` | `10` |
| 006 | nested_object_y | `var obj = { inner: { x: 10, y: 20 } }\nobj.inner.y` | `20` |
| 007 | json | ServiceInfo array + field access | `ClearDiagnosticInformation` |
| 008 | last_block_or_object | `{ a: 1, b: 2 }` | `{a: 1, b: 2}` |
| 009 | multiple_field_mutations | 3-field object mutation + sum | `600` |

**Steps:** Same pattern. Commit: `feat(test): add vm 07_objects file-based tests`

---

## Task 7: Create `08_strings` (4 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | fstr | `var name = "auto"\nf"hello $name, now!"` | `hello auto, now!` |
| 002 | fstr_expr | `var a = 1\nvar b = 2\nf"a + b = ${a+b}"` | `a + b = 3` |
| 003 | str_index | `let a = "hello"\na[1]` | `'e'` |
| 004 | to_string | `1.str()` → `"1"`, `"hello".upper()` → `"HELLO"` (use two assertions in one test — since we can only check result, split into two) |

Actually split 004:
| 004 | int_to_str | `1.str()` | `1` |
| 005 | str_upper | (already in 01_basics as 003_str_upper — skip duplicate) |

**Steps:** Same pattern. Commit: `feat(test): add vm 08_strings file-based tests`

---

## Task 8: Create `09_functions` (9 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | fn_simple | `fn add(a, b) { a + b }\nadd(12, 2)` | `14` |
| 002 | fn_named_args | `fn add(a, b) { a + b }\nadd(a:1, b:2)` | `3` |
| 003 | fn_multiple | `fn add(a, b) { a + b }\nadd(add(1, 2), add(3, 4))` | `10` |
| 004 | fn_nested | add+mul nested | `26` |
| 005 | fn_in_expr | `fn add(a, b) { a + b }\n10 + add(5, 3)` | `18` |
| 006 | fn_local_var | `fn double(a) { let x = a + a; x }\ndouble(5)` | `10` |
| 007 | closure | `var add = (a, b) => a + b\nadd(1, 2)` | `3` |
| 008 | closure_typed | `let sub = (a int, b int) => a - b\nsub(12, 5)` | `7` |
| 009 | forward_decl | `fn test() int;\nfn test() int { 42 }\ntest()` | `42` |

**Steps:** Same pattern. Commit: `feat(test): add vm 09_functions file-based tests`

---

## Task 9: Create `10_types` (9 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | type_compose | Wing + Duck `has` composition + `duck.fly()` | `.expected.out`: `flap!flap!\n` |
| 002 | int_enum | `enum Color { Red = 1 ... }\nColor.Red` | `1` |
| 003 | generic_instantiation | `type Point<T> { x T; y T }` + instance + sum | `300` |
| 004 | generic_field_x | generic Point field x | `100` |
| 005 | generic_field_y | generic Point field y | `200` |
| 006 | field_addition | non-generic Point field sum | `300` |
| 007 | type_instance_prop | `a.x.type` | `int` |
| 008 | nested_type_instance | Inner+Outer nested creation | `10` |
| 009 | access_fields_in_method | type with method accessing fields | (just verify no error) |

**Steps:** Same pattern. Commit: `feat(test): add vm 10_types file-based tests`

---

## Task 10: Create `11_compound_ops` (6 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | add_eq | `var a = 1\na += 1\na` | `2` |
| 002 | sub_eq | `var a = 10\na -= 3\na` | `7` |
| 003 | mul_eq | `var a = 5\na *= 3\na` | `15` |
| 004 | div_eq | `var a = 20\na /= 4\na` | `5` |
| 005 | chained | `var a = 1\na += 1\na += 2\na += 3\na` | `7` |
| 006 | div_eq_oneline | `var a = 20; a /= 4; a` | `5` |

**Steps:** Same pattern. Commit: `feat(test): add vm 11_compound_ops file-based tests`

---

## Task 11: Create `12_type_coercion` (6 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | int_plus_float | `2 + 3.5` | `5.5` |
| 002 | float_plus_int | `3.5 + 2` | `5.5` |
| 003 | int_times_float | `4 * 2.5` | `10` |
| 004 | float_times_int | `2.5 * 4` | `10` |
| 005 | complex | `(2 + 3.5) * 5` | `27.5` |
| 006 | with_variable | `let x = 2; x + 3.5` | `5.5` |

**Steps:** Same pattern. Commit: `feat(test): add vm 12_type_coercion file-based tests`

---

## Task 12: Create `13_collections` — HashMap (8 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | hashmap_new | `HashMap.new()` + drop | `0` |
| 002 | hashmap_insert_str | insert_str + get_str | `[Alice, Wonderland]` |
| 003 | hashmap_insert_int | insert_int + get_int + sum | `67` |
| 004 | hashmap_contains | contains present/missing | `[1, 0]` |
| 005 | hashmap_size | insert 3 + size | `3` |
| 006 | hashmap_remove | insert + remove + contains | `0` |
| 007 | hashmap_clear | insert 2 + clear + size | `0` |

**Steps:** Same pattern. Commit: `feat(test): add vm 13_collections hashmap tests`

---

## Task 13: Create `13_collections` — HashSet (6 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 008 | hashset_new | `HashSet.new()` + drop | `0` |
| 009 | hashset_insert | insert 3 items + contains | `[1, 1, 1]` |
| 010 | hashset_duplicate | insert same 3x + size | `1` |
| 011 | hashset_remove | insert + remove + contains | `0` |
| 012 | hashset_size | insert 3 + size | `3` |
| 013 | hashset_clear | insert 2 + clear + size | `0` |

**Steps:** Same pattern. Commit: `feat(test): add vm 13_collections hashset tests`

---

## Task 14: Create `13_collections` — StringBuilder (6 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 014 | sb_new | `StringBuilder.new(1024)` + drop | `0` |
| 015 | sb_append | append "hello" + " " + "world" + build | `hello world` |
| 016 | sb_append_char | append "hello" + append_char '!' + build | `hello!` |
| 017 | sb_append_int | append "count: " + append_int 42 + build | `count: 42` |
| 018 | sb_len | append "hello" + len | `5` |
| 019 | sb_clear | append "hello" + clear + len | `0` |

**Steps:** Same pattern. Commit: `feat(test): add vm 13_collections stringbuilder tests`

---

## Task 15: Create `13_collections` — List (16 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 020 | list_new | `List.new()` + is_empty + drop | `1` |
| 021 | list_push_pop | push 3 + pop + len + popped | `32` |
| 022 | list_push_pop_multi | push 2 + len + pop + is_empty | `1` |
| 023 | list_len | push 3 + len + drop | `3` |
| 024 | list_is_empty | empty check + push + not empty | `[1, 0]` |
| 025 | list_clear | push 3 + clear + len + drop | `0` |
| 026 | list_get_set | push 3 + get + set + get updated | `[10, 20, 15]` |
| 027 | list_insert_remove | insert + remove verification | `1` |
| 028 | list_reserve | reserve + push + len | `2` |
| 029 | list_comprehensive | push + len + clear + is_empty | `1` |
| 030 | list_multi_ops | push 3 + get + set + len checks | `1` |
| 031 | list_index | `List.new(10, 20, 30)` + index access | `1` |
| 032 | list_varargs | `List.new(1,2,3,4,5)` + len | `1` |
| 033 | list_varargs_empty | `List.new()` + len | `1` |
| 034 | list_for_iteration | `List.new(1..5)` + for sum | `15` |
| 035 | list_for_empty | empty list + for count | `0` |

Note: Many List tests return `1`/`0` because they use nested if-else for verification. This is fine for `.expected.result`.

**Steps:** Same pattern. Commit: `feat(test): add vm 13_collections list tests`

---

## Task 16: Create `14_borrow` (18 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | view_basic | `let s = "hello"\nlet v = s.view\nv` | `hello` |
| 002 | view_multiple | `let x = 42\nlet v1 = x.view\nlet v2 = x.view\nv1 + v2` | `84` |
| 003 | mut_basic | str_new + mut + str_append | contains "hello" |
| 004 | move_basic | `let s1 = "hello"\nlet s2 = s1.move\ns2` | `hello` |
| 005 | view_preserves | `let x = 100\nlet v = x.view\nx` | `100` |
| 006 | nested_view | double view borrow | `42` |
| 007 | borrow_arithmetic | view two values + multiply | `30` |
| 008 | view_in_array | view in array context | `[10, 20]` |
| 009 | view_in_expr | view in expression | `15` |
| 010 | borrow_diff_types | view int + str, array result | contains "42" and "hello" |
| 011 | move_chaining | move two values | `first` |
| 012 | str_slice_view | string view | `hello world` |
| 013 | str_slice_multi | multiple string views | contains both |
| 014 | str_slice_nested | nested string view | `hello` |
| 015 | str_slice_in_array | string view in array | contains both |
| 016 | str_slice_take | string take | `hello` |
| 017 | str_slice_mixed | mixed view + take | `first` |
| 018 | str_slice_preserves | view preserves original | `hello` |

**Steps:** Same pattern. Commit: `feat(test): add vm 14_borrow file-based tests`

---

## Task 17: Create `15_nested_mutation` (14 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | object_field | `var obj = {x:10,y:20}\nobj.x=30\nobj.x` | `30` |
| 002 | array_element | `var arr = [1,2,3]\narr[0]=10\narr[0]` | `10` |
| 003 | multiple_fields | 3-field mutation + sum | `600` |
| 004 | multiple_array | 3-element mutation + sum | `60` |
| 005 | type_field | type Point field mutation | `30` |
| 006 | nested_object | `obj.inner.x = 30` | `30` |
| 007 | array_of_obj_field | `arr[0].x = 10` | `10` |
| 008 | obj_array_element | `obj.items[0] = 10` | `10` |
| 009 | nested_array | `matrix[0][1] = 20` | `20` |
| 010 | type_nested_field | type Inner/Outer nested mutation | `20` |
| 011 | three_level | 3-level nesting | `200` |
| 012 | deep_array_obj | deep array of objects | `25` |
| 013 | structure_preserve | mutate a.x, verify others unchanged | `[2, 3, 4]` |
| 014 | out_of_bounds_error | `obj.items[10] = 100` | `.expected.error` |
| 015 | invalid_field_error | `obj.inner.nonexistent = 20` | `.expected.error` |
| 016 | type_mismatch_error | `obj.items.invalid_field = 10` | `.expected.error` |

**Steps:** Same pattern. Commit: `feat(test): add vm 15_nested_mutation file-based tests`

---

## Task 18: Create `16_option_result` (16 tests)

| # | Name | .at source | Expected |
|---|------|-----------|----------|
| 001 | option_type | `let x ?int = None\n1` | `1` |
| 002 | result_type | `let x !int = Ok(42)\n1` | `1` |
| 003 | none_literal | `let x = None\n1` | `1` |
| 004 | some_ctor | `let x = Some(42)\n1` | `1` |
| 005 | ok_ctor | `let x = Ok(100)\n1` | `1` |
| 006 | err_ctor | `let x = Err("error message")\n1` | `1` |
| 007 | propagate_some | `let x = Some(42)\nlet y = x.?\ny` | `42` |
| 008 | propagate_none | `let x = None\nlet y = x.?\ny` | (verify no error — result may vary) |
| 009 | propagate_ok | `let x = Ok(100)\nlet y = x.?\ny` | `100` |
| 010 | propagate_err | `let x = Err("...")\nlet y = x.?\ny` | (verify no error) |
| 011 | coalesce_some | `let x = Some(42)\nlet y = x ?? 0\ny` | `42` |
| 012 | coalesce_none | `let x = None\nlet y = x ?? 99\ny` | `99` |
| 013 | is_some_binding | `let opt = Some(42)\nis opt { Some(v) -> v; None -> 0 }` | `42` |
| 014 | is_none_match | `let opt = None\nis opt { Some(v) -> v; None -> -1 }` | `-1` |
| 015 | is_ok_binding | `let res = Ok(100)\nis res { Ok(v) -> v; Err(e) -> -1 }` | `100` |
| 016 | is_err_match | `let res = Err("...")\nis res { Ok(v) -> v; Err(e) -> e }` | `-2` |

Note: Tests 008 and 010 may need adjustment based on VM behavior with None/Err propagation.

**Steps:** Same pattern. Commit: `feat(test): add vm 16_option_result file-based tests`

---

## Task 19: Add `arithmetic_float` to `01_basics`

The existing inline test `test_arithmetic` has two assertions in one test (int arithmetic + float arithmetic). Split into the existing `002_arithmetic` (already covers int) and add a float test:

`012_arithmetic_float/arithmetic_float.at`:
```auto
(2 + 3.5) * 5
```
`012_arithmetic_float/arithmetic_float.expected.result`: `27.5`

And add uint arithmetic:
`013_uint_add/uint_add.at`:
```auto
25u + 123u
```
`013_uint_add/uint_add.expected.result`: `148u`

**Steps:** Create files, add test entries, run tests, commit.

---

## Task 20: Slim down `vm_tests.rs`

**Files:**
- Modify: `crates/auto-lang/src/tests/vm_tests.rs`

Remove all tests that have been migrated to file-based tests. Keep only:

1. `test_vm_ret_constant` — direct bytecode test
2. `test_vm_const_i32_add` — direct bytecode test
3. `test_vm_annotation_in_ext` — AST inspection
4. `test_function_body_parsing` — AST inspection
5. `test_node_newline` — config mode parsing
6. `test_nodes` — node syntax verification
7. `test_node_arg_ident` — node syntax verification
8. `test_atom_reader_multiline` — AtomReader test
9. `test_array_return_eval` — function return types
10. All `#[ignore]` tests (test_grid, test_view_types, test_for_array, test_range_print, test_atom_query, test_node_store)
11. All commented-out tests (keep as-is for reference)

**Step 1:** Remove all migrated test functions from `vm_tests.rs`.

**Step 2:** Run `cargo test -p auto-lang` to verify no regressions.

**Step 3:** Commit

```
refactor(test): migrate vm_tests to file-based tests (Plan 179)
```

---

## Task 21: Final verification

**Step 1:** Run all tests
```bash
cargo test -p auto-lang
```

**Step 2:** Count migrated tests
```bash
cargo test -p auto-lang -- vm_file_tests --list 2>&1 | grep "test::" | wc -l
```

**Step 3:** Verify no regressions in `vm_tests`
```bash
cargo test -p auto-lang -- vm_tests
```

**Step 4:** Commit (if any fixes needed)

---

## Summary

| Category | Tests | Status |
|----------|-------|--------|
| 01_basics | 11+2=13 | Expand existing |
| 02_bit_ops | 5 | Keep existing |
| 03_variables | 8 | New |
| 04_control_flow | 7 | New |
| 05_loops | 4 | New |
| 06_arrays | 5 | New |
| 07_objects | 9 | New |
| 08_strings | 4 | New |
| 09_functions | 9 | New |
| 10_types | 9 | New |
| 11_compound_ops | 6 | New |
| 12_type_coercion | 6 | New |
| 13_collections | 36 | New |
| 14_borrow | 18 | New |
| 15_nested_mutation | 16 | New |
| 16_option_result | 16 | New |
| **Total** | **~167** | |
| vm_tests.rs (kept) | ~11 | Bytecode, AST, config, ignored |
