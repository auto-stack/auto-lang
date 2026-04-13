# Plan 167: AutoLang Module System — Complete Implementation

**Goal:** Build a complete module system for AutoLang that supports folder modules (`mod.at`), public re-exports (`pub use`), wildcard imports, circular dependency detection, and multi-file a2r transpilation.

**Architecture:** File-based modules (each `.at` file = one module). Folder modules use `mod.at` as the entry point. Symbols are re-exported with `pub use`. The a2r transpiler gains multi-file output: each `.at` file → one `.rs` file, `mod.at` → `mod.rs`.

**Tech Stack:** Rust (crates/auto-lang), existing resolver/use_scanner/compile infrastructure.

---

## Current State

**Already works:**
- `FilesystemResolver::find_module()` tries `X.at` first, then `X/mod.at` (resolver.rs:219-266)
- `UseStatement` in use_scanner.rs has `is_wildcard` field
- `TypeStore::merge()` and `import_items()` support selective and wildcard symbol merging
- `ModulePath` with `PathPrefix::None/Super/Pac/Dep`
- `pub` annotation (`#[pub]`) on functions, types, etc.

**Missing (7 items identified):**

| # | Gap | Disposition |
|---|-----|-------------|
| 1 | No `mod` keyword | SOLVED — `mod.at` convention, resolver already checks |
| 2 | No `pub use` re-exports | IMPLEMENT — parsing + symbol resolution |
| 3 | No wildcard imports in transpiler | IMPLEMENT — scanner detects, transpiler ignores |
| 4 | `PathPrefix::Dep` not implemented | DEFER — not needed for single-project |
| 5 | No circular dependency detection | IMPLEMENT — at resolver/compile level |
| 6 | No multi-file transpilation | IMPLEMENT — Sink refactor, `mod.at` → `mod.rs` |
| 7 | No test coverage for modules | IMPLEMENT — multi-file a2r tests |

---

## Phase 1: `pub use` Re-exports (Parsing + Symbol Resolution)

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` — parse `pub use` statement
- Modify: `crates/auto-lang/src/ast/use_.rs` — add `is_pub` field to `Use` struct
- Modify: `crates/auto-lang/src/compile.rs` — propagate pub symbols through `load_module()`
- Modify: `crates/auto-lang/src/types.rs` — track pub-ness in TypeStore
- Modify: `crates/auto-lang/src/use_scanner.rs` — add `is_pub` to `UseStatement`
- Test: `crates/auto-lang/test/a2r/159_pub_use/pub_use.at` + `.expected.rs`

### Step 1: Add `is_pub` field to `Use` AST struct

In `crates/auto-lang/src/ast/use_.rs`, add:
```rust
pub struct Use {
    pub kind: UseKind,
    pub module_path: Option<ModulePath>,
    pub paths: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
    pub is_pub: bool,       // NEW: pub use X
}
```

Update Display, ToNode, AtomWriter, and all test constructors to include `is_pub: false` (backward compat).

### Step 2: Parse `pub use` in parser

In `crates/auto-lang/src/parser.rs`, the `parse_use_stmt()` method:
- When encountering `pub` keyword followed by `use`, set `is_pub: true`
- `pub` is NOT currently a keyword — need to add special handling in the `parse_use_stmt` context
- Note: Auto uses `#[pub]` annotation syntax for items. For `pub use`, use the bare `pub` keyword (like Rust) since `#[pub] use` would be awkward
- The parser should check: if current token text is "pub" and next token is "use", consume both and set `is_pub: true`

### Step 3: Track pub exports in TypeStore

In `crates/auto-lang/src/types.rs`:
- Add `pub_exports: HashSet<AutoStr>` to `TypeStore` — tracks which imported symbols are re-exported as pub
- In `import_items()` — when called from a `pub use`, also add the imported items to `pub_exports`
- Add method `pub_import_items(&mut self, other: &TypeStore, items: &[AutoStr])` that both imports AND marks as pub

### Step 4: Propagate pub exports through load_module

In `crates/auto-lang/src/compile.rs` `load_module()`:
- When `use_stmt` comes from a `pub use`, call `pub_import_items()` instead of `import_items()`
- The `UseStatement` struct needs an `is_pub` field too (or pass it separately)

### Step 5: Update UseStatement in use_scanner.rs

Add `is_pub: bool` to `UseStatement` struct. Update `parse_use_line()` to detect `pub use` prefix. Update all constructors.

### Step 6: Write test

Create `crates/auto-lang/test/a2r/159_pub_use/`:
- `pub_use.at`:
```auto
// mod.at re-exports
pub use db: connect, query

fn main() {
    let conn = connect()
    query(conn)
}
```
- `pub_use.expected.rs`:
```rust
pub use crate::db::{connect, query};

fn main() {
    let conn: () = connect();
    query(conn);
}
```

### Step 7: Run tests, commit

```bash
cargo test -p auto-lang test_159_pub_use
cargo test -p auto-lang
git commit -m "feat: pub use re-export parsing and symbol resolution"
```

---

## Phase 2: Wildcard Imports (`use module: *`)

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs` — emit `use module::*;` for wildcard
- Modify: `crates/auto-lang/src/parser.rs` — preserve wildcard info in Use AST
- Modify: `crates/auto-lang/src/ast/use_.rs` — add `is_wildcard: bool` field
- Test: `crates/auto-lang/test/a2r/160_wildcard_import/wildcard.at` + `.expected.rs`

### Step 1: Add `is_wildcard` to Use AST struct

In `crates/auto-lang/src/ast/use_.rs`:
```rust
pub struct Use {
    pub kind: UseKind,
    pub module_path: Option<ModulePath>,
    pub paths: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
    pub is_pub: bool,
    pub is_wildcard: bool,  // NEW: use module: *
}
```

### Step 2: Parse wildcard in parser

In `crates/auto-lang/src/parser.rs` `parse_use_stmt()`:
- After the `:` in `use module: *`, check if the item is `*` (star token)
- If so, set `is_wildcard: true` and leave `items` empty

### Step 3: Emit wildcard in a2r transpiler

In `crates/auto-lang/src/trans/rust.rs` `use_stmt()`:
- When `use_stmt.is_wildcard` is true and kind is Auto:
  - Build the module path as usual
  - Emit `use module::*;` instead of `use module;`
- When kind is Rust and `is_wildcard`:
  - Emit `use crate::*;` or the full path with `::*`

### Step 4: Write test

Create `crates/auto-lang/test/a2r/160_wildcard_import/`:
- `wildcard.at`:
```auto
use auto.io: *

fn main() {
    say("hello from wildcard")
}
```
- `wildcard.expected.rs`:
```rust
use crate::io::*;

fn main() {
    say("hello from wildcard");
}
```

### Step 5: Run tests, commit

```bash
cargo test -p auto-lang test_160_wildcard_import
cargo test -p auto-lang
git commit -m "feat: wildcard import support in a2r transpiler"
```

---

## Phase 3: Circular Dependency Detection

**Files:**
- Modify: `crates/auto-lang/src/compile.rs` — track loading stack, detect cycles
- Test: `crates/auto-lang/src/compile.rs` integration tests

### Step 1: Add loading stack to CompileSession

In `crates/auto-lang/src/compile.rs`:
- Add `loading_stack: Vec<String>` to `CompileSession` — tracks modules currently being loaded
- At the start of `load_module()`, check if `use_stmt.module` is in `loading_stack`
- If yes, return error: `AutoError::Msg(format!("Circular dependency detected: {} -> {}", loading_stack.join(" -> "), use_stmt.module))`
- Push module name before loading, pop after

### Step 2: Write compile-time test

Add test in `crates/auto-lang/src/compile.rs` (or a new test module):
- Create temp files: `a.at` with `use b`, `b.at` with `use a`
- Call `resolve_uses()` on `a.at`
- Assert error message contains "Circular dependency"

### Step 3: Run tests, commit

```bash
cargo test -p auto-lang circular
cargo test -p auto-lang
git commit -m "feat: circular dependency detection in module loading"
```

---

## Phase 4: Multi-File a2r Transpilation

This is the largest phase. Currently `Sink` has a single `body` buffer and `transpile_rust()` processes one file at a time. We need:

1. A way to process multiple `.at` files and collect their outputs
2. `mod.at` → `mod.rs` with `pub mod` declarations for submodules
3. Submodule files → `submodule.rs` (renamed from `submodule.at`)
4. Proper `use` path remapping between modules

**Files:**
- Modify: `crates/auto-lang/src/trans.rs` — add `MultiSink` struct for multi-file output
- Modify: `crates/auto-lang/src/trans/rust.rs` — `transpile_rust_multi()` entry point
- Modify: `crates/auto-lang/src/trans/rust.rs` — `use_stmt()` path remapping for multi-file
- Test: `crates/auto-lang/test/a2r/161_multi_file/` — multi-file test fixture

### Step 1: Create MultiSink

In `crates/auto-lang/src/trans.rs`:
```rust
pub struct MultiSink {
    pub files: Vec<(String, Sink)>,  // (filename, sink)
}

impl MultiSink {
    pub fn new() -> Self { Self { files: Vec::new() } }

    pub fn add(&mut self, name: &str) -> &mut Sink {
        let sink = Sink::new(AutoStr::from(name));
        self.files.push((name.to_string(), sink));
        self.files.last_mut().unwrap().1
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Sink> {
        self.files.iter_mut().find(|(n, _)| n == name).map(|(_, s)| s)
    }
}
```

### Step 2: Create `transpile_rust_project()` entry point

In `crates/auto-lang/src/trans/rust.rs`:
```rust
pub fn transpile_rust_project(entry_file: &str) -> AutoResult<HashMap<String, Vec<u8>>> {
    // 1. Parse entry file, collect its `use` statements
    // 2. For each `use X` (Auto kind):
    //    a. Resolve X.at or X/mod.at via FilesystemResolver
    //    b. Parse the module file
    //    c. Recursively process its `use` statements
    // 3. Topological sort modules (entry file last)
    // 4. Transpile each module into its own Sink
    // 5. For mod.at: emit `pub mod submodule;` declarations
    // 6. Return HashMap<filename, output_bytes>
}
```

Key design decisions:
- **Module naming**: `db.at` → `db.rs`, `api/mod.at` → `api/mod.rs`, `api/handlers.at` → `api/handlers.rs`
- **Use path remapping**: `use db` in `main.at` becomes `mod db;` in `main.rs` (sibling module), or `use crate::db;` if db is in the same crate
- **pub use remapping**: `pub use db: connect` → `pub use crate::db::connect;`
- **Entry point**: The file containing `fn main()` is the crate root; its `use X` statements for sibling files become `mod X;`

### Step 3: Update `use_stmt()` for multi-file mode

The `RustTrans` struct needs a mode flag or context to know if it's in single-file or multi-file mode:

- **Single-file (current)**: `use db` → `use crate::db;` (everything in one file)
- **Multi-file (new)**:
  - `use db` where `db.at` is a sibling → `mod db;` (declares submodule)
  - `use db: connect` → `use crate::db::connect;` (import specific item)
  - `use super.utils` → `use super::utils;`

### Step 4: Handle mod.at → mod.rs

When a folder module is found (e.g., `api/mod.at`):
- Transpile `mod.at` contents into `api/mod.rs`
- Scan for `use` statements in `mod.at` that reference sibling files → emit `pub mod sibling;`
- Scan for `pub use X: item` → emit `pub use crate::X::item;`

### Step 5: Write multi-file test

Create `crates/auto-lang/test/a2r/161_multi_file/`:
```
161_multi_file/
├── main.at              → main.rs (entry point)
├── main.expected.rs
├── db.at                → db.rs
├── db.expected.rs
├── api/
│   ├── mod.at           → api/mod.rs
│   ├── mod.expected.rs
│   ├── handlers.at      → api/handlers.rs
│   └── handlers.expected.rs
```

`main.at`:
```auto
use db
use api

fn main() {
    let conn = db.connect()
    api.handle_request(conn)
}
```

`main.expected.rs`:
```rust
mod db;
mod api;

fn main() {
    let conn = db::connect();
    api::handle_request(conn);
}
```

`db.at`:
```auto
type Connection {
    url str
}

fn connect() Connection {
    Connection(url: "localhost")
}
```

`db.expected.rs`:
```rust
struct Connection {
    url: String,
}

fn connect() -> Connection {
    Connection { url: "localhost".to_string() }
}
```

`api/mod.at`:
```auto
pub use api.handlers: handle_request
```

`api/mod.expected.rs`:
```rust
pub use crate::api::handlers::handle_request;
```

`api/handlers.at`:
```auto
use super.db
use crate::db: Connection

fn handle_request(conn Connection) {
    // handle
}
```

`api/handlers.expected.rs`:
```rust
use super::db;
use crate::db::Connection;

fn handle_request(conn: Connection) {
}
```

### Step 6: Run tests, commit

```bash
cargo test -p auto-lang test_161_multi_file
cargo test -p auto-lang
git commit -m "feat: multi-file a2r transpilation with mod.rs generation"
```

---

## Phase 5: Integration Tests for Module Features

**Files:**
- Add: `crates/auto-lang/test/a2r/162_module_reexport/` — pub use re-export test
- Add: `crates/auto-lang/test/a2r/163_mod_at_folder/` — mod.at folder module test
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs` — add test functions

### Step 1: pub use re-export test

`162_module_reexport/reexport.at`:
```auto
use auto.io: say

pub use auto.io: say
```

`162_module_reexport/reexport.expected.rs`:
```rust
use crate::io::say;

pub use crate::io::say;
```

### Step 2: mod.at folder module test

This tests single-file transpilation of a mod.at file (no multi-file needed):

`163_mod_at_folder/mod_folder.at`:
```auto
// This is the mod.at content for a folder module
type Config {
    name str
    port int
}

fn default_config() Config {
    Config(name: "app", port: 8080)
}
```

`163_mod_at_folder/mod_folder.expected.rs`:
```rust
struct Config {
    name: String,
    port: i32,
}

fn default_config() -> Config {
    Config { name: "app".to_string(), port: 8080 }
}
```

### Step 3: Run all tests, commit

```bash
cargo test -p auto-lang
git commit -m "test: add module system a2r tests"
```

---

## Phase 6: Cargo.toml Generation (Bonus — addresses 6B-3.2)

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs` — add `generate_cargo_toml()` function
- Modify: `crates/auto-lang/src/trans/rust.rs` — extend `transpile_rust_project()` to emit Cargo.toml

### Step 1: Generate Cargo.toml

When doing multi-file transpilation, also generate a `Cargo.toml`:
```toml
[package]
name = "<project-name>"
version = "0.1.0"
edition = "2021"

[dependencies]
# From dep statements in source
serde = { version = "1.0", features = ["derive"] }
```

### Step 2: Collect deps from source

Scan all `.at` files for `dep name(version: "X")` statements and convert to Cargo.toml dependencies.

### Step 3: Run tests, commit

```bash
cargo test -p auto-lang
git commit -m "feat: Cargo.toml generation in a2r project transpilation"
```

---

## Summary

| Phase | Feature | Files Changed | Complexity |
|-------|---------|--------------|------------|
| 1 | `pub use` re-exports | parser, ast/use_, compile, types, use_scanner | Medium |
| 2 | Wildcard imports | parser, ast/use_, trans/rust | Low |
| 3 | Circular dependency detection | compile | Low |
| 4 | Multi-file a2r transpilation | trans, trans/rust | High |
| 5 | Module integration tests | tests/a2r_tests.rs, test fixtures | Low |
| 6 | Cargo.toml generation | trans/rust | Low |

**Dependencies:** Phases 1-3 are independent. Phase 4 depends on Phases 1-2. Phase 5 depends on all. Phase 6 depends on Phase 4.

**Recommended order:** Phase 3 → Phase 2 → Phase 1 → Phase 4 → Phase 5 → Phase 6
