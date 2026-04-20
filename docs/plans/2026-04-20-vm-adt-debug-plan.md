# VM ADT, Generic Lists, Pattern Destructuring & Debug Formatting — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add four runtime features to the Auto VM in dependency order: struct debug formatting, enum variants with data, List<UserType>, and pattern destructuring in `is`-expressions.

**Architecture:** Enum variants reuse the existing `GenericInstanceData` heap object system with a `mono_name` encoding the variant (`"Atom.Int"`). Pattern matching compiles to tag-check + field-extraction using existing opcodes. Debug formatting extends `TO_STR` to handle heap objects.

**Tech Stack:** Rust, AutoLang crate (`auto-lang`), existing VM infrastructure (heap objects, generic registry, opcodes).

---

### Task 1: Add `field_names` to `GenericInstanceData`

**Files:**
- Modify: `crates/auto-lang/src/vm/generic_registry.rs:500-524`

**Step 1: Add `field_names` field to the struct**

In `generic_registry.rs`, update `GenericInstanceData`:

```rust
#[derive(Debug)]
pub struct GenericInstanceData {
    pub mono_name: String,
    pub fields: Vec<Value>,
    pub field_names: Vec<String>,  // ADD: field names for debug formatting
}
```

Update `new()`:

```rust
pub fn new(mono_name: String, fields: Vec<Value>) -> Self {
    let field_names = vec!["_unknown".to_string(); fields.len()];
    Self { mono_name, fields, field_names }
}

pub fn new_with_names(mono_name: String, fields: Vec<Value>, field_names: Vec<String>) -> Self {
    Self { mono_name, fields, field_names }
}
```

**Step 2: Build and verify compilation**

Run: `cargo build -p auto-lang 2>&1 | tail -5`
Expected: Possible warnings about unused `field_names`, no errors.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/generic_registry.rs
git commit -m "refactor(vm): add field_names to GenericInstanceData"
```

---

### Task 2: Populate `field_names` during `CONSTRUCT_INSTANCE`

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1517-1615` (CONSTRUCT_INSTANCE handler)
- Modify: `crates/auto-lang/src/vm/generic_registry.rs` (Template struct, to expose field names)

**Step 1: Expose field names from GenericTemplate**

In `generic_registry.rs`, check if `GenericTemplate` (or `ClassType`) has a `fields` vector with names. If it does, add a method:

```rust
pub fn field_names(&self) -> Vec<String> {
    self.template.fields.iter().map(|f| f.name.clone()).collect()
}
```

**Step 2: Store field names in CONSTRUCT_INSTANCE**

In `engine.rs`, after the line `instance.fields = field_values;` (~line 1584), add field name lookup:

```rust
// Populate field_names from the class template
let field_names = self.generic_registry
    .get_type(&instance.mono_name)
    .map(|ct| ct.template.fields.iter().map(|f| f.name.clone()).collect())
    .unwrap_or_else(|| vec!["_unknown".to_string(); field_count]);
instance.field_names = field_names;
```

**Step 3: Build and verify**

Run: `cargo build -p auto-lang 2>&1 | tail -5`
Expected: Clean build.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/src/vm/generic_registry.rs
git commit -m "feat(vm): populate field_names in CONSTRUCT_INSTANCE"
```

---

### Task 3: Extend `TO_STR` to format struct instances

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1268-1287` (TO_STR handler)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/020_struct_to_str.at`:

```auto
type Point {
    x int
    y int
}

fn main() {
    let p = Point { x: 3, y: 4 }
    let s = p.to(str)
    print(s)
    assert(s.contains("Point"))
    assert(s.contains("3"))
    print("struct_to_str: passed")
}
```

**Step 2: Run test to verify it fails**

Run: `cd d:/autostack/auto-lang && target/debug/auto test/vm/10_types/020_struct_to_str.at`
Expected: Fails with assertion error (TO_STR outputs raw integer).

**Step 3: Implement TO_STR for heap objects**

In `engine.rs`, replace the `TO_STR` handler (lines 1268-1287) with:

```rust
OpCode::TO_STR => {
    let value_bits = task.ram.pop_i32();

    if value_bits < 0 {
        // Already a tagged string
        task.ram.push_i32(value_bits);
    } else if value_bits >= 4000000 {
        // Heap object — format as Type { field: value, ... }
        use crate::vm::generic_registry::GenericInstanceData;
        use crate::vm::heap_object::TypeTag;

        let obj_id = value_bits as u64;
        let formatted = if let Some(obj) = self.get_heap_object(obj_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                let fields_str = instance.fields.iter()
                    .zip(instance.field_names.iter())
                    .map(|(val, name)| {
                        let val_str = format_value(val, &self.strings);
                        format!("{}: {}", name, val_str)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", instance.mono_name, fields_str)
            } else {
                format!("<object {}>", obj_id)
            }
        } else {
            format!("<invalid {}>", obj_id)
        };

        let mut strings = self.strings.write().unwrap();
        let str_idx = strings.len();
        strings.push(formatted.into_bytes());
        drop(strings);
        task.ram.push_i32(-(str_idx as i32) - 1);
    } else {
        // Plain integer
        let string_value = format!("{}", value_bits);
        let mut strings = self.strings.write().unwrap();
        let str_idx = strings.len();
        strings.push(string_value.into_bytes());
        drop(strings);
        task.ram.push_i32(-(str_idx as i32) - 1);
    }
}
```

Add a helper method on `AutoVM`:

```rust
fn format_value(val: &Value, strings: &RwLock<Vec<Vec<u8>>>) -> String {
    match val {
        Value::Int(i) => format!("{}", i),
        Value::Str(s) => format!("\"{}\"", s),
        Value::Bool(b) => format!("{}", b),
        Value::VmRef(r) => {
            if r.id >= 4000000 {
                format!("<ref {}>", r.id)
            } else {
                format!("{}", r.id)
            }
        }
        Value::Nil => "nil".to_string(),
        _ => format!("{:?}", val),
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd d:/autostack/auto-lang && target/debug/auto test/vm/10_types/020_struct_to_str.at`
Expected: `struct_to_str: passed`

**Step 5: Rebuild auto binary and test 08_usage_struct**

Run: `cargo build --bin auto && cd d:/autostack/auto-code-rs && auto crates/ac-examples/src/08_usage_struct/main.at`
Expected: All assertions pass, including debug output.

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/020_struct_to_str.at
git commit -m "feat(vm): TO_STR formats struct instances as Type { field: val }"
```

---

### Task 4: Register enum variants in `GenericRegistry`

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:1463-1470` (EnumDecl handler)
- Modify: `crates/auto-lang/src/vm/generic_registry.rs` (add variant registration)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/021_enum_variant_data.at`:

```auto
enum Atom {
    Int int
    Str str
}

fn main() {
    let a = Atom.Int(42)
    print(f"variant: ${a}")
    print("enum_variant_data: passed")
}
```

**Step 2: Run test to verify it fails**

Run: `target/debug/auto test/vm/10_types/021_enum_variant_data.at`
Expected: Fails — `Atom.Int(42)` has no codegen.

**Step 3: Register enum variants as generic types**

In `codegen.rs`, in the `EnumDecl` handler (~line 1463), after registering scalar variants, add registration for data-carrying variants:

```rust
// For data-carrying variants, register as generic types
for item in &enum_decl.items {
    if item.payload_type.is_some() {
        let variant_mono = format!("{}.{}", enum_decl.name, item.name);
        let payload = item.payload_type.as_ref().unwrap();
        // Create a template with a single field for the payload
        let fields = vec![GenericField {
            name: "_0".to_string(),
            field_type: payload.clone(),
            default_value: None,
        }];
        self.generic_registry.register_template(
            &variant_mono,
            GenericTemplate::new(&variant_mono, fields),
        );
    }
}
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/src/vm/generic_registry.rs
git commit -m "feat(codegen): register enum data variants in GenericRegistry"
```

---

### Task 5: Codegen for enum variant construction

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (add enum variant construction in `compile_expr`)

**Step 1: Add variant construction codegen**

Find where `Expr::Dot` is compiled in `compile_expr`. When the expression is `Atom.Int(42)`:
- It's parsed as a `Call` with name `Dot(Ident("Atom"), "Int")` and args `[42]`.
- Check if the type is an enum with a data-carrying variant.
- If yes, emit NEW_INSTANCE + CONSTRUCT_INSTANCE with mono_name `"Atom.Int"`.

In the `Expr::Call(call)` handler in codegen.rs, after the `is_generic_constructor` check (~line 3920), add:

```rust
// Plan: Enum variant construction (e.g., Atom.Int(42))
let is_enum_variant = if let Expr::Dot(obj, method) = call.name.as_ref() {
    if let Expr::Ident(type_name) = obj.as_ref() {
        let variant_mono = format!("{}.{}", type_name.as_ref(), method.as_ref());
        self.generic_registry.has_template(&variant_mono)
    } else {
        false
    }
} else {
    false
};

if is_enum_variant {
    // Compile variant construction using NEW_INSTANCE + CONSTRUCT_INSTANCE
    if let Expr::Dot(obj, method) = call.name.as_ref() {
        if let Expr::Ident(type_name) = obj.as_ref() {
            let variant_mono = format!("{}.{}", type_name.as_ref(), method.as_ref());
            if let Ok(class_type) = self.generic_registry.get_or_create_type(&variant_mono, vec![]) {
                // Compile args as field values
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(expr) = arg {
                        self.compile_expr(expr)?;
                    } else if let crate::ast::Arg::Pair(_, expr) = arg {
                        self.compile_expr(expr)?;
                    }
                }

                // NEW_INSTANCE
                let name_bytes = variant_mono.as_bytes();
                self.emit(OpCode::CONST_I32);
                self.emit_i32(name_bytes.len() as i32);
                self.emit(OpCode::NEW_INSTANCE);
                for &byte in name_bytes {
                    self.code.push(byte);
                }

                // CONSTRUCT_INSTANCE
                let field_count = class_type.template.fields.len();
                self.emit(OpCode::CONST_I32);
                self.emit_i32(field_count as i32);
                self.emit(OpCode::CONSTRUCT_INSTANCE);

                return Ok(());
            }
        }
    }
}
```

**Step 2: Build and run the test**

Run: `cargo build --bin auto && target/debug/auto test/vm/10_types/021_enum_variant_data.at`
Expected: Prints variant info, passes.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): compile enum variant construction (Atom.Int(42))"
```

---

### Task 6: Access payload fields from enum variants

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (field access on variant instances)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/022_enum_field_access.at`:

```auto
enum Atom {
    Int int
    Str str
}

fn main() {
    let a = Atom.Int(42)
    // Access payload field _0
    assert_eq(a._0, 42)

    let b = Atom.Str("hello")
    assert_eq(b._0, "hello")

    print("enum_field_access: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails — field access on enum variants not yet supported in codegen.

**Step 3: Add field access support**

In the dot-access codegen (~line 3150), when resolving `a._0` on a variable of enum variant type, the existing `GET_GENERIC_FIELD` mechanism should already work since enum variants are `GenericInstanceData` with mono_name `"Atom.Int"`. The codegen needs to:
1. Detect that `a`'s type is an enum variant (check if `"Atom.Int"` is in generic_registry).
2. Look up field index from the variant's template.
3. Emit `GET_GENERIC_FIELD`.

This likely already works if the variable type is correctly tracked. If not, add a fallback: when `field_name.starts_with('_')` and the type is a string that contains a `.`, treat it as an enum variant and look up in generic_registry.

**Step 4: Run test**

Expected: Passes.

**Step 5: Commit**

```bash
git add crates/auto-lang/test/vm/10_types/022_enum_field_access.at
git commit -m "feat(vm): field access on enum variant payloads"
```

---

### Task 7: `List<UserType>` support

**Files:**
- Modify: `crates/auto-lang/src/vm/opcode.rs:259-306` (add new opcodes to VALID)
- Modify: `crates/auto-lang/src/vm/engine.rs:1418-1444` (CREATE_LIST_REF handler)
- Modify: `crates/auto-lang/src/vm/engine.rs:1885-2019` (GET_ELEM extension)
- Modify: `crates/auto-lang/src/vm/codegen.rs` (emit LIST_REF opcodes)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/023_list_user_type.at`:

```auto
enum Atom {
    Int int
}

fn main() {
    let a = Atom.Int(1)
    let b = Atom.Int(2)
    let list = [a, b]
    let first = list[0]
    assert_eq(first._0, 1)
    let second = list[1]
    assert_eq(second._0, 2)
    print("list_user_type: passed")
}
```

**Step 2: Add new opcodes**

In `opcode.rs`, add to the enum:

```rust
CREATE_LIST_REF = 0xA6,  // Create list of heap references
LIST_PUSH_REF = 0xA7,    // Push heap ref onto list
LIST_GET_REF = 0xA8,     // Get heap ref from list by index
```

Add `0xA6, 0xA7, 0xA8` to the VALID array.

**Step 3: Implement engine handlers**

In `engine.rs`, add handlers for the three new opcodes:

```rust
OpCode::CREATE_LIST_REF => {
    let list: Vec<i32> = Vec::new();
    let list_id = self.insert_heap_object(/* RefList wrapper */);
    task.ram.push_i32(list_id as i32);
}
OpCode::LIST_PUSH_REF => {
    let value = task.ram.pop_i32();
    // push onto ref list
}
OpCode::LIST_GET_REF => {
    let index = task.ram.pop_i32();
    // get from ref list, push i32 (heap ID) back
}
```

For the internal storage, use a simple `Vec<i32>` wrapped in a newtype since all values are i32 (either plain ints or heap IDs).

**Step 4: Extend GET_ELEM to handle heap ref lists**

In `GET_ELEM` handler, after the existing List<int/str/bool> branches, add:

```rust
// List of heap references (user-defined types)
else if let Some(ref_list) = guard.as_any().downcast_ref::<Vec<i32>>() {
    let val = ref_list.get(index as usize).copied().unwrap_or(0);
    task.ram.push_i32(val);
}
```

**Step 5: Update codegen**

When creating an array literal with elements of user-defined type, emit `CREATE_LIST_REF` + `LIST_PUSH_REF` instead of `CREATE_LIST_INT` + `LIST_PUSH_INT`. When accessing elements, emit the appropriate get opcode.

**Step 6: Build and run test**

Expected: Passes.

**Step 7: Commit**

```bash
git add crates/auto-lang/src/vm/opcode.rs crates/auto-lang/src/vm/engine.rs crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(vm): List<UserType> support with heap ref opcodes"
```

---

### Task 8: Pattern destructuring in `is`-expressions

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (extend `compile_is` for enum patterns)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/024_is_enum_destructure.at`:

```auto
enum Atom {
    Int int
    Str str
}

fn main() {
    let a = Atom.Int(42)
    is a {
        Atom.Int(n) -> {
            assert_eq(n, 42)
            print(f"matched Int: $n")
        },
        Atom.Str(s) -> {
            print("wrong branch")
        },
        else -> print("no match")
    }
    print("is_enum_destructure: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails — `is` with enum destructuring not implemented.

**Step 3: Implement destructuring codegen**

In the `is` statement codegen, when a branch pattern is an enum variant with bindings:

1. Emit: load the match expression
2. Emit: `GET_GENERIC_FIELD 0` (get `__tag` or first field)
3. Emit: compare with expected variant
4. On match: extract each binding with `GET_GENERIC_FIELD idx` + `STORE_LOCAL`
5. Compile branch body
6. Jump to end

For now, the "tag check" can be done by checking if the mono_name matches. Since enum variants have unique mono_names (`"Atom.Int"` vs `"Atom.Str"`), we can use a dedicated field or check the object's type.

**Approach:** Add a `__variant` string field at index 0 of every enum variant instance. During construction (Task 5), also push the variant name string. Then the tag check is `GET_GENERIC_FIELD 0` → compare string.

**Step 4: Run test**

Expected: Passes.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): pattern destructuring for enum variants in is-expressions"
```

---

### Task 9: Integration test — full example

**Files:**
- Create: `crates/ac-examples/src/09_input_message_builders/main.at` (restore original with enum-based design)

**Step 1: Rewrite 09_input_message_builders using new features**

Restore the enum-based version of the example using:
- `enum ContentBlock { Text(str), ToolUse(str, str), ToolResult(str, str) }`
- Pattern matching with destructuring
- `List<ContentBlock>` for message content

**Step 2: Run all examples to verify no regressions**

Run each example:
```bash
cd d:/autostack/auto-code-rs
auto crates/ac-examples/src/01_djb2_hash/main.at
auto crates/ac-examples/src/07_glob_match/main.at
auto crates/ac-examples/src/08_usage_struct/main.at
auto crates/ac-examples/src/09_input_message_builders/main.at
```

Expected: All pass.

**Step 3: Commit**

```bash
git add crates/ac-examples/src/09_input_message_builders/main.at
git commit -m "feat(examples): restore 09_input_message_builders with enum + pattern matching"
```
