# 190: Extend `use.rust` for Comprehensive Rust Stdlib Access

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable AutoLang code to import and use any Rust stdlib (or third-party) type/function via the existing `use.rust` mechanism, with compile-time type awareness.

**Architecture:** Extend the current `use.rust` pipeline — fix two blocking bugs (std exemption), register imported Rust types in TypeStore as `Type::Rust`, and track generic provenance so transpilers emit correct `use` statements. No separate bridge layer; `use.rust` becomes the single entry point.

**Tech Stack:** Rust (compiler, transpiler), existing `use.rust` + `dep` mechanism, TypeStore

---

## Design

### 1. Type System Addition: `Type::Rust`

Add a dedicated variant to the `Type` enum in `crates/auto-lang/src/ast/types.rs`:

```rust
pub struct RustSource {
    pub full_path: String,  // e.g. "std::collections::HashMap"
}

pub enum Type {
    // ... existing variants ...
    Rust(RustSource),  // Foreign Rust type imported via use.rust
}
```

**Why:** Currently unknown type names fall through to `Type::User(TypeDecl { name, empty_body })`. `Type::Rust` distinguishes "known Rust import" from "truly unknown type", enabling:
- Clear error messages: "Unknown type `Foo`" vs "Rust type `HashMap` used outside a2r target"
- Transpiler dispatch: a2c errors on Rust types; a2r passes through
- Type inference: `HashMap::new()` returns `Type::Rust(...)` instead of `Unknown`

### 2. Std/Core/Alloc Exemption (Bug Fixes)

**Bug 1 — VM path requires `dep std`:** `compile.rs:is_dep_declared()` has no exemption for Rust built-in crates. `use.rust std::collections::HashMap` fails with "Crate 'std' not declared." Fix: add a built-in set `{"std", "core", "alloc", "proc_macro"}` that returns `true` without requiring a `dep` statement.

**Bug 2 — Cargo.toml gets `std = "*"`:** `trans/rust.rs` Cargo.toml generation unconditionally adds all `use.rust` paths[0] as dependencies. `std = "*"` fails because `std` is not on crates.io. Fix: filter out the same built-in set before writing `[dependencies]`.

### 3. TypeStore Registration for `use.rust` Imports

When `compile.rs:resolve_uses()` encounters a `use.rust` statement, register the imported items in TypeStore as `Type::Rust` entries instead of just validating and skipping.

**Named imports** (`use.rust std::collections::{HashMap, HashSet}`): register each item with full path.
**Module import** (`use.rust std::collections::HashMap`): register last segment as name.
**Wildcard imports** (`use.rust std::collections::*`): register nothing; parser falls through to `Type::Rust` on first use.

### 4. Generic Instance Tracking

Add `source: Option<RustSource>` to `GenericInstance`. When the parser encounters `HashMap<str, int>` and resolves `HashMap` to `Type::Rust(...)`, the resulting `GenericInstance` carries the Rust provenance. This lets the a2r transpiler emit correct fully-qualified `use` statements even when Rust types appear as generic parameters inside other types.

### 5. Transpiler Behavior

**a2r** — `Type::Rust(path)` emits short name (last segment); `GenericInstance { source }` emits base_name with generic args; deduplicated `use` statements collected during transpilation.

**a2c** — `Type::Rust` produces clear error: "use.rust imports are not supported in C target".

**VM/Evaluator** — `Type::Rust` handled by FFI bridge at runtime, same as today. No change.

### 6. Test Sets

**a2r FFI tests** — verifying generated `.rs` compiles and matches expected output:

| Module | Types/Functions | Test |
|--------|----------------|------|
| `std::collections` | `HashMap`, `HashSet`, `BTreeMap`, `VecDeque` | `001_collections` |
| `std::fs` + `std::io` | `File::open`, `read_to_string`, `BufReader` | `002_fs` |
| `std::sync` | `Arc`, `Mutex` | `003_sync` |
| `std::time` | `Instant`, `Duration` | `004_time` |
| `std::path` | `PathBuf` | `005_path` |
| `std::boxed` + `std::cell` | `Box`, `RefCell` | `006_box_cell` |
| `std::env` + `std::process` | `env::var`, `Command` | `007_env_process` |
| `std::thread` | `spawn`, `sleep` | `008_thread` |
| `serde_json` | `from_str`, `to_string` | `009_serde_json` |
| `regex` | `Regex::new`, `is_match` | `010_regex` |

**VM FFI tests** — deferred to follow-up plan (requires runtime FFI bridge infrastructure).

### Out of Scope (Future Work)

- **Type stubs database:** Maintaining Rust stdlib method signatures for compile-time method validation and IDE auto-completion.
- **Wildcard import expansion:** Parsing rustdoc/crate metadata to resolve `use.rust std::collections::*` into specific types.
- **Trait method resolution:** Understanding `impl Trait for Type` to enable trait method calls on imported types.
- **rustdoc JSON integration:** Downloading and parsing rustdoc for arbitrary crates to generate Auto type stubs.

---

## Implementation

### Task 1: Add `Type::Rust` variant and `RustSource` struct

**Files:**
- Modify: `crates/auto-lang/src/ast/types.rs:7-51` (Type enum)
- Modify: `crates/auto-lang/src/ast/types.rs:53-112` (unique_name)
- Modify: `crates/auto-lang/src/ast/types.rs:497-525` (Display)
- Modify: `crates/auto-lang/src/ast/types.rs:527-569` (From<Type> for auto_val::Type)

**Step 1: Add `RustSource` struct and `Type::Rust` variant**

In `crates/auto-lang/src/ast/types.rs`, add before the `Type` enum (around line 7):

```rust
/// Source of a Rust type imported via use.rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustSource {
    pub full_path: String,  // e.g. "std::collections::HashMap"
}

impl RustSource {
    pub fn new(path: impl Into<String>) -> Self {
        Self { full_path: path.into() }
    }

    /// Get the short type name (last segment of the path)
    pub fn short_name(&self) -> &str {
        self.full_path.rsplit("::").next().unwrap_or(&self.full_path)
    }
}
```

In the `Type` enum (around line 49, after `Handle`), add:

```rust
    // Plan 190: Rust type imported via use.rust
    Rust(RustSource),
```

**Step 2: Add `unique_name` match arm**

In `crates/auto-lang/src/ast/types.rs`, in the `unique_name()` method (around line 109, after `Handle`), add:

```rust
            Type::Rust(source) => source.short_name().into(),
```

**Step 3: Add `Display` match arm**

In `crates/auto-lang/src/ast/types.rs`, in the `fmt::Display` impl (around line 522, after `Handle`), add:

```rust
            Type::Rust(source) => write!(f, "{}", source.full_path),
```

**Step 4: Add `From<Type> for auto_val::Type` match arm**

In `crates/auto-lang/src/ast/types.rs`, in the `From` impl (around line 566, after `Handle`), add:

```rust
            Type::Rust(source) => auto_val::Type::User(source.short_name().into()),
```

**Step 5: Build to verify compilation**

Run: `cargo build`
Expected: Compiles clean. All match arms on `Type` are now exhaustive for the new variant.

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ast/types.rs
git commit -m "feat(types): add Type::Rust variant for use.rust imported types"
```

---

### Task 2: Add `source` field to `GenericInstance`

**Files:**
- Modify: `crates/auto-lang/src/ast/types.rs:374-397` (GenericInstance struct)

**Step 1: Add `source` field to `GenericInstance`**

In `crates/auto-lang/src/ast/types.rs`, modify the `GenericInstance` struct (around line 377):

```rust
#[derive(Debug, Clone)]
pub struct GenericInstance {
    pub base_name: Name,
    pub args: Vec<Type>,
    /// Plan 190: Rust provenance for types imported via use.rust
    pub source: Option<RustSource>,
}
```

**Step 2: Fix all `GenericInstance` construction sites**

Run: `cargo build`
Expected: Compilation errors at every `GenericInstance { ... }` missing the `source` field. Fix each by adding `source: None`.

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles clean.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ast/types.rs
git commit -m "feat(types): add source field to GenericInstance for Rust provenance"
```

---

### Task 3: Fix std/core/alloc exemption in compile.rs

**Files:**
- Modify: `crates/auto-lang/src/compile.rs:280-285` (is_dep_declared)

**Step 1: Add built-in crate exemption**

Modify `is_dep_declared` (line 283):

```rust
    pub fn is_dep_declared(&self, crate_name: &str) -> bool {
        if self.declared_crates.contains(crate_name) {
            return true;
        }
        // Plan 190: Rust built-in crates are always available
        matches!(crate_name, "std" | "core" | "alloc" | "proc_macro")
    }
```

**Step 2: Build and run existing test**

Run: `cargo build && cargo test -p auto-lang rt_14_rust_use`
Expected: PASS.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/compile.rs
git commit -m "fix(compile): exempt std/core/alloc/proc_macro from dep requirement"
```

---

### Task 4: Fix Cargo.toml std exclusion in a2r transpiler

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs:4128-4147` (Cargo.toml generation)

**Step 1: Filter built-in crates from Cargo.toml deps**

Modify the Cargo.toml generation (around line 4133):

```rust
        let mut deps: Vec<String> = Vec::new();
        let built_in_crates = ["std", "core", "alloc", "proc_macro"];
        for (_, ast) in &parsed_modules {
            for stmt in &ast.stmts {
                if let Stmt::Use(u) = stmt {
                    if matches!(u.kind, UseKind::Rust) && !u.paths.is_empty() {
                        let crate_name = u.paths[0].as_str();
                        if !deps.contains(&crate_name.to_string())
                            && !built_in_crates.contains(&crate_name) {
                            deps.push(crate_name.to_string());
                        }
                    }
                }
            }
        }
```

**Step 2: Build and run existing test**

Run: `cargo build && cargo test -p auto-lang rt_14_rust_use`
Expected: PASS.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/trans/rust.rs
git commit -m "fix(a2r): exclude built-in crates from Cargo.toml dependencies"
```

---

### Task 5: Register `use.rust` imports in TypeStore as `Type::Rust`

**Files:**
- Modify: `crates/auto-lang/src/types.rs` (add fields and methods to TypeStore)
- Modify: `crates/auto-lang/src/compile.rs:175-196` (resolve_uses)

**Step 1: Add Rust tracking fields to TypeStore**

In `crates/auto-lang/src/types.rs`, add fields to `TypeStore` (line 133):

```rust
pub struct TypeStore {
    type_decls: HashMap<AutoStr, Rc<TypeDecl>>,
    fn_decls: HashMap<Name, Fn>,
    spec_decls: HashMap<AutoStr, SpecDecl>,
    generic_templates: HashMap<String, GenericTemplate>,
    type_aliases: HashMap<AutoStr, AutoStr>,
    enum_decls: HashMap<AutoStr, Rc<EnumDecl>>,
    /// Plan 190: Names of types imported via use.rust
    rust_types: HashSet<String>,
    /// Plan 190: Maps short name -> full Rust path
    rust_type_paths: HashMap<String, String>,
}
```

Update `TypeStore::new()` to include `rust_types: HashSet::new()` and `rust_type_paths: HashMap::new()`.

Add methods:

```rust
    /// Plan 190: Register a Rust type imported via use.rust
    pub fn register_rust_type(&mut self, name: impl Into<AutoStr>, full_path: impl Into<String>) {
        use crate::ast::types::RustSource;
        use crate::ast::{Name, TypeDecl, TypeDeclKind};

        let type_name = name.into();
        let path = full_path.into();
        self.rust_types.insert(type_name.to_string());
        self.rust_type_paths.insert(type_name.to_string(), path.clone());

        let decl = TypeDecl {
            name: Name::from(type_name.clone()),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: vec![],
            doc: None,
            is_pub: false,
        };
        self.type_decls.insert(type_name, Rc::new(decl));
    }

    /// Plan 190: Check if a type name was imported via use.rust
    pub fn is_rust_type(&self, name: &str) -> bool {
        self.rust_types.contains(name)
    }

    /// Plan 190: Get the full Rust path for a use.rust imported type
    pub fn get_rust_type_path(&self, name: &str) -> Option<String> {
        self.rust_type_paths.get(name).cloned()
    }
```

**Step 2: Update `resolve_uses` to register Rust types**

In `crates/auto-lang/src/compile.rs`, modify the `use.rust` handling in `resolve_uses` (around line 176):

```rust
            if use_stmt.is_rust_import {
                let crate_name = use_stmt.module.split("::").next().unwrap_or(&use_stmt.module);

                if !self.is_dep_declared(crate_name) {
                    return Err(AutoError::Msg(format!(
                        "Crate '{}' not declared. Add `dep {}` before `use.rust`.",
                        crate_name, crate_name
                    )));
                }

                // Plan 190: Register imported Rust types in TypeStore
                if let Ok(mut store) = self.type_store.write() {
                    if use_stmt.is_wildcard {
                        log::info!("Rust wildcard import: {}", use_stmt.module);
                    } else if !use_stmt.items.is_empty() {
                        for item in &use_stmt.items {
                            let full_path = format!("{}::{}", use_stmt.module, item);
                            store.register_rust_type(item.as_str(), full_path);
                        }
                    } else {
                        if let Some(short_name) = use_stmt.module.rsplit("::").next() {
                            store.register_rust_type(short_name, use_stmt.module.as_str());
                        }
                    }
                }

                loaded_count += 1;
                continue;
            }
```

**Step 3: Build**

Run: `cargo build`
Expected: Compiles clean.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/compile.rs crates/auto-lang/src/types.rs
git commit -m "feat(compile): register use.rust imports in TypeStore as Rust types"
```

---

### Task 6: Update parser `lookup_type` to return `Type::Rust`

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:810-825` (lookup_type TypeStore section)

**Step 1: Tag registered Rust types**

Modify the TypeStore lookup in `lookup_type` (around line 810):

```rust
        if let Ok(store) = self.type_store.read() {
            if let Some(type_decl) = store.lookup_type_decl_str(name) {
                // Plan 190: Return Type::Rust for use.rust imports
                if let Some(full_path) = store.get_rust_type_path(name) {
                    return shared(Type::Rust(RustSource::new(full_path)));
                }
                return shared(Type::User(type_decl.as_ref().clone()));
            }
```

**Step 2: Build**

Run: `cargo build`
Expected: Compiles clean.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): return Type::Rust for use.rust imported types in lookup_type"
```

---

### Task 7: Update `parse_generic_instance` to propagate Rust source

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:7595-7730` (parse_generic_instance)

**Step 1: Propagate `RustSource` when base type is `Type::Rust`**

In `parse_generic_instance`, add a new arm before the user-defined generic type arm (around line 7713):

```rust
            // Plan 190: Rust type with generic params (e.g., HashMap<str, int>)
            Type::Rust(rust_source) => {
                return Ok(Type::GenericInstance(GenericInstance {
                    base_name: base_name.clone(),
                    args,
                    source: Some(rust_source),
                }));
            }
```

**Step 2: Build**

Run: `cargo build`
Expected: Compiles clean.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): propagate RustSource in GenericInstance for use.rust types"
```

---

### Task 8: Update a2r transpiler for `Type::Rust`

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs:233-321` (rust_type_name)

**Step 1: Add `Type::Rust` match arm**

In `rust_type_name` (around line 319, before closing brace):

```rust
            Type::Rust(source) => source.short_name().to_string(),
```

**Step 2: Update `GenericInstance` to use short_name from source**

Modify the GenericInstance arm (around line 304):

```rust
            Type::GenericInstance(inst) => {
                let args: Vec<String> = inst.args.iter().map(|t| self.rust_type_name(t)).collect();
                let base = if let Some(ref source) = inst.source {
                    source.short_name().to_string()
                } else {
                    inst.base_name.to_string()
                };
                format!("{}<{}>", base, args.join(", "))
            }
```

**Step 3: Build and run existing test**

Run: `cargo build && cargo test -p auto-lang rt_14_rust_use`
Expected: PASS.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/trans/rust.rs
git commit -m "feat(a2r): handle Type::Rust in rust_type_name"
```

---

### Task 9: Update a2c transpiler to error on `Type::Rust`

**Files:**
- Modify: `crates/auto-lang/src/trans/c.rs:1321-1323` (use_stmt Rust arm)

**Step 1: Add error for `Type::Rust` in use_stmt**

```rust
            UseKind::Rust => {
                return Err(GenError::UnsupportedStmt(
                    "use.rust imports are not supported in C target".to_string()
                ));
            }
```

**Step 2: Build**

Run: `cargo build`
Expected: Compiles clean.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/trans/c.rs
git commit -m "fix(a2c): error clearly on use.rust imports instead of silently ignoring"
```

---

### Task 10: a2r test — rust_collections (HashMap, HashSet, BTreeMap, VecDeque)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/001_collections/collections.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/001_collections/collections.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs` (add test function)

**Step 1: Write input file**

`crates/auto-lang/test/a2r/15_rust_std/001_collections/collections.at`:

```auto
use.rust std::collections::{HashMap, HashSet, BTreeMap, VecDeque}

fn main() {
    var scores HashMap<str, int> = HashMap.new()
    scores.insert("alice", 100)
    scores.insert("bob", 95)
    let alice_score = scores.get("alice")

    var visited HashSet<str> = HashSet.new()
    visited.insert("home")
    visited.insert("about")

    var timeline BTreeMap<int, str> = BTreeMap.new()
    timeline.insert(1, "start")
    timeline.insert(2, "middle")

    var queue VecDeque<int> = VecDeque.new()
    queue.push_back(1)
    queue.push_back(2)
    let front = queue.pop_front()
}
```

**Step 2: Run transpiler, review output, save as .expected.rs**

Run the transpiler on the input, review the generated Rust, save as `collections.expected.rs`.

**Step 3: Register test in r2a.rs**

```rust
    #[test]
    fn rt_15_rust_collections() {
        roundtrip_a2r("15_rust_std/001_collections");
    }
```

**Step 4: Run test**

Run: `cargo test -p auto-lang rt_15_rust_collections`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/auto-lang/test/a2r/15_rust_std/ crates/auto-lang/src/trans/r2a.rs
git commit -m "test(a2r): add rust_collections test for std::collections types"
```

---

### Task 11: a2r test — rust_fs (File, BufReader)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/002_fs/fs.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/002_fs/fs.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::fs::{File, read_to_string}
use.rust std::io::{BufRead, BufReader}

fn main() {
    let content = read_to_string("test.txt")
    let file = File.open("test.txt")
    let reader = BufReader.new(file)
    let line = reader.lines().next()
}
```

**Step 2-5:** Same pattern as Task 10 — generate, review, save, register, commit.

---

### Task 12: a2r test — rust_sync (Arc, Mutex)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/003_sync/sync.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/003_sync/sync.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::sync::{Arc, Mutex}

fn main() {
    let data = Arc.new(Mutex.new(0))
    let mut guard = data.lock()
    guard = guard + 1
}
```

---

### Task 13: a2r test — rust_time (Instant, Duration)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/004_time/time.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/004_time/time.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::time::{Instant, Duration}

fn main() {
    let start = Instant.now()
    let d = Duration.from_secs(5)
    let d2 = Duration.from_millis(500)
}
```

---

### Task 14: a2r test — rust_path (PathBuf)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/005_path/path.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/005_path/path.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::path::PathBuf

fn main() {
    let p = PathBuf.from("src/main.rs")
    let p2 = p.join("lib.rs")
}
```

---

### Task 15: a2r test — rust_box_cell (Box, RefCell)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/006_box_cell/box_cell.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/006_box_cell/box_cell.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::boxed::Box
use.rust std::cell::RefCell

fn main() {
    let b = Box.new(42)
    let cell = RefCell.new(0)
    let mut val = cell.borrow_mut()
    val = val + 1
}
```

---

### Task 16: a2r test — rust_env_process (env, Command)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/007_env_process/env_process.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/007_env_process/env_process.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::env
use.rust std::process::Command

fn main() {
    let home = env.var("HOME")
    let args = env.args()
    let output = Command.new("ls").arg("-la").output()
}
```

---

### Task 17: a2r test — rust_thread (spawn, sleep)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/008_thread/thread.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/008_thread/thread.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
use.rust std::thread
use.rust std::time::Duration

fn main() {
    let handle = thread.spawn(fn() {
        thread.sleep(Duration.from_secs(1))
    })
    handle.join()
}
```

Note: May need adjustment depending on how the transpiler handles closures in `thread.spawn`.

---

### Task 18: a2r test — rust_serde_json (third-party)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/009_serde_json/serde_json.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/009_serde_json/serde_json.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
dep serde_json
use.rust serde_json::{from_str, to_string}

fn main() {
    let json = to_string("hello")
    let parsed = from_str(json)
}
```

Verifies third-party crates are correctly added to Cargo.toml and use statements emitted.

---

### Task 19: a2r test — rust_regex (third-party)

**Files:**
- Create: `crates/auto-lang/test/a2r/15_rust_std/010_regex/regex.at`
- Create: `crates/auto-lang/test/a2r/15_rust_std/010_regex/regex.expected.rs`
- Modify: `crates/auto-lang/src/trans/r2a.rs`

**Step 1: Write input file**

```auto
dep regex
use.rust regex::Regex

fn main() {
    let re = Regex.new(r"\d+")
    let is_match = re.is_match("abc123")
}
```

---

### Task 20: Full build and test verification

**Step 1: Build all**

Run: `cargo build`
Expected: Compiles clean.

**Step 2: Run all a2r tests**

Run: `cargo test -p auto-lang -- trans`
Expected: All tests pass, including new `rt_15_*` tests.

**Step 3: Run all tests**

Run: `cargo test -p auto-lang`
Expected: All tests pass (some pre-existing failures in `a2r_tests` and `book_listing_tests` are known).

**Step 4: Commit any fixes**

If any test adjustments were needed, commit them.
