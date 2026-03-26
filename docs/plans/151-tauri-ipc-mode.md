# Plan 151: Tauri IPC Mode for api-example

## Objective

Enable `api-example` to generate complete Tauri IPC backend by transpiling `api.at` + `db.at` to Rust, creating an independent `rust/` crate that integrates with Tauri's thin shell.

## Current State

- `api.at` defines API endpoints with `#[api]` annotation
- `db.at` contains business logic with global mutable state
- `TauriGenerator` exists but only generates command signatures (no body)
- `TypeScriptGenerator` generates `tauriApi` client using `invoke()`

## Target Architecture

```
api-example/
├── back/                    # Auto source code
│   ├── api.at               # API interface definitions
│   └── db.at                # Database operations + global state
│
├── rust/                    # Generated Rust crate
│   ├── Cargo.toml           # Dependencies: serde, once_cell, tauri (optional)
│   └── src/
│       ├── lib.rs           # Module exports
│       ├── types.rs         # User, CreateUserRequest types
│       ├── db.rs            # Global state + database functions
│       └── commands.rs      # #[tauri::command] wrappers
│
└── vue/
    └── src-tauri/           # Tauri thin shell
        ├── Cargo.toml       # Depends on ../../rust
        └── src/
            └── main.rs      # Register commands, start Tauri
```

## Transpilation Mapping

### Types

| Auto | Rust |
|------|------|
| `type User = { id: int, name: str, email: str }` | `#[derive(Serialize, Deserialize)] pub struct User { pub id: i32, pub name: String, pub email: String }` |
| `?User` | `Option<User>` |
| `[]User` | `Vec<User>` |
| `List<User>` | `Vec<User>` |

### Global State

| Auto | Rust |
|------|------|
| `var users List<User> = List<User>.new([...])` | `static USERS: Lazy<Mutex<Vec<User>>> = Lazy::new(\|\| Mutex::new(vec![...]));` |
| `var nextid int = 4` | `static NEXT_ID: Lazy<Mutex<i32>> = Lazy::new(\|\| Mutex::new(4));` |

### Functions

| Auto | Rust |
|------|------|
| `pub fn find_user(id int) ?User` | `pub fn find_user(id: i32) -> Option<User>` |
| `return Some(user)` / `return None` | `return Some(user);` / `return None;` |
| `#[api(method = "GET", path = "/api/users/:id")]` | `#[tauri::command]` |

### Expressions

| Auto | Rust |
|------|------|
| `for user in users { ... }` | `for user in users.lock().unwrap().iter() { ... }` |
| `users.push(user)` | `users.lock().unwrap().push(user);` |
| `users.filter((u User) => { ... })` | `users.lock().unwrap().iter().filter(\|u: &User\| { ... }).cloned().collect()` |
| `query.to_lower()` | `query.to_lowercase()` |
| `s.contains(query)` | `s.contains(&query)` |
| `users.to_array()` | `users.lock().unwrap().clone()` |
| `nextid = nextid + 1` | `*next_id.lock().unwrap() += 1;` |

## Implementation Phases

### Phase 1: a2r Transpiler Enhancements

**Goal**: Extend `crates/auto-lang/src/trans/rust.rs` to support Tauri backend generation.

#### 1.1 Global State Support

- Detect top-level `var` declarations
- Generate `static X: Lazy<Mutex<T>>` pattern
- Add `once_cell` and `Lazy` imports

```rust
// Detection in trans/rust.rs
fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<bool> {
    match stmt {
        Stmt::Store(store) if store.is_top_level && store.kind == StoreKind::Var => {
            // Generate static Lazy<Mutex<T>>
            self.global_var(store, sink)?;
            Ok(true)
        }
        // ...
    }
}

fn global_var(&mut self, store: &Store, sink: &mut Sink) -> AutoResult<()> {
    let ty = self.rust_type_name(&store.ty);
    write!(sink.body, "static {}: Lazy<Mutex<{}>> = Lazy::new(|| Mutex::new(",
           store.name.to_uppercase(), ty)?;
    self.expr(&store.expr, &mut sink.body)?;
    writeln!(sink.body, "));")?;
    Ok(())
}
```

#### 1.2 Global Variable Access

- Replace variable reads with `X.lock().unwrap()`
- Replace variable writes with `*X.lock().unwrap() = ...` or methods

```rust
// In expr() for Expr::Ident
Expr::Ident(name) => {
    if self.is_global_var(&name) {
        write!(out, "{}.lock().unwrap()", name.to_uppercase())?;
    } else {
        write!(out, "{}", name)?;
    }
    Ok(())
}
```

#### 1.3 Method Call Translations

| Auto Method | Rust Translation |
|-------------|------------------|
| `.to_lower()` | `.to_lowercase()` |
| `.to_upper()` | `.to_uppercase()` |
| `.length()` | `.len()` |
| `.contains(s)` | `.contains(&s)` |

#### 1.4 Closure with Block Body

Current support for `(u User) => { ... }` needs to handle:
- Multiple statements in body
- `return` statements inside closure
- Mutable variable captures

### Phase 2: Tauri Backend Generator

**Goal**: Create new generator in `crates/auto-man/src/tauri_backend.rs`.

#### 2.1 Generator Structure

```rust
pub struct TauriBackendGenerator {
    types: Vec<GeneratedType>,
    commands: Vec<GeneratedCommand>,
    globals: Vec<GeneratedGlobal>,
}

pub struct GeneratedGlobal {
    name: String,
    ty: String,
    init_value: String,
}

pub struct GeneratedCommand {
    name: String,
    params: Vec<(String, String)>,  // (name, type)
    return_type: String,
    body: String,  // Generated Rust code
}
```

#### 2.2 Generation Flow

```
1. Parse api.at + db.at
2. Extract types → types.rs
3. Extract globals → db.rs (with Lazy<Mutex<T>>)
4. Extract functions → db.rs (with .lock().unwrap())
5. Extract #[api] functions → commands.rs (with #[tauri::command])
6. Generate Cargo.toml
7. Generate lib.rs
```

#### 2.3 Output Files

**Cargo.toml**:
```toml
[package]
name = "api-example-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
once_cell = "1.19"
tauri = { version = "2", optional = true }

[features]
default = []
tauri = ["dep:tauri"]
```

**src/lib.rs**:
```rust
mod types;
mod db;
#[cfg(feature = "tauri")]
mod commands;

pub use types::*;
pub use db::*;
#[cfg(feature = "tauri")]
pub use commands::*;
```

**src/types.rs**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}
```

**src/db.rs**:
```rust
use crate::types::User;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static USERS: Lazy<Mutex<Vec<User>>> = Lazy::new(|| Mutex::new(vec![
    User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
    User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
    User { id: 3, name: "Charlie".to_string(), email: "charlie@example.com".to_string() },
]));

static NEXT_ID: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(4));

pub fn find_user(id: i32) -> Option<User> {
    let users = USERS.lock().unwrap();
    for user in users.iter() {
        if user.id == id {
            return Some(user.clone());
        }
    }
    None
}

pub fn all_users() -> Vec<User> {
    USERS.lock().unwrap().clone()
}

pub fn create_user(name: String, email: String) -> User {
    let mut next_id = NEXT_ID.lock().unwrap();
    let user = User {
        id: *next_id,
        name,
        email,
    };
    *next_id += 1;
    USERS.lock().unwrap().push(user.clone());
    user
}

pub fn delete_user(id: i32) -> bool {
    let mut users = USERS.lock().unwrap();
    let original_len = users.len();
    users.retain(|u| u.id != id);
    users.len() < original_len
}

pub fn search_users(query: String) -> Vec<User> {
    let query_lower = query.to_lowercase();
    let users = USERS.lock().unwrap();
    users.iter()
        .filter(|u| {
            u.name.to_lowercase().contains(&query_lower) ||
            u.email.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect()
}
```

**src/commands.rs**:
```rust
use crate::db;
use crate::types::*;
use tauri::command;

#[command]
pub fn getuser(id: i32) -> Option<User> {
    db::find_user(id)
}

#[command]
pub fn listusers() -> Vec<User> {
    db::all_users()
}

#[command]
pub fn createuser(req: CreateUserRequest) -> User {
    db::create_user(req.name, req.email)
}

#[command]
pub fn deleteuser(id: i32) -> bool {
    db::delete_user(id)
}

#[command]
pub fn searchusers(query: String) -> Vec<User> {
    db::search_users(query)
}
```

### Phase 3: Build Integration

**Goal**: Integrate with `auto build` and `auto run` commands.

#### 3.1 Configuration

In `pac.at`:
```auto
// back/pac.at
name: "api-example-back"
backend: "rust-tauri"  // New backend type
entry: "api.at"
```

#### 3.2 Build Flow

```bash
auto build --tauri
```

1. Parse `api.at` and `db.at`
2. Generate `rust/` crate
3. Generate `vue/src-tauri/` if not exists
4. `vue/src-tauri/Cargo.toml` depends on `../../rust`
5. `vue/src-tauri/src/main.rs` imports commands from `rust`

#### 3.3 Run Flow

```bash
auto run --tauri
```

1. Check if `rust/` exists, generate if not
2. Check if `vue/src-tauri/` exists, initialize if not
3. `npm install` in `vue/`
4. `npx tauri dev`

### Phase 4: Testing

#### 4.1 Unit Tests

- Test global variable generation
- Test closure transpilation
- Test method call translation

#### 4.2 Integration Tests

- Generate full `rust/` crate from `api.at` + `db.at`
- Compile generated Rust code
- Verify no compilation errors

#### 4.3 End-to-End Test

- Run `auto build --tauri`
- Start Tauri app
- Call API commands from frontend
- Verify responses

## Success Criteria

1. ✅ `api.at` + `db.at` transpile to valid Rust code
2. ✅ Generated `rust/` crate compiles without errors
3. ✅ `vue/src-tauri/` can import commands from `rust/`
4. ✅ Tauri app runs and responds to API calls
5. ✅ Frontend `tauriApi` client works with generated backend

## Dependencies

- `once_cell` crate for `Lazy<T>`
- `serde` for serialization
- `tauri` 2.x (optional, for commands)

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Complex closure bodies | Start with simple closures, add complexity incrementally |
| Mutex deadlocks | Use short lock scopes, clone when needed |
| Type mismatches | Add comprehensive type translation tests |

## Timeline

- **Phase 1**: a2r transpiler enhancements (2-3 sessions)
- **Phase 2**: Tauri backend generator (1-2 sessions)
- **Phase 3**: Build integration (1 session)
- **Phase 4**: Testing (1 session)

## References

- [Tauri Commands Documentation](https://tauri.app/v2/guides/features/command/)
- [once_cell::sync::Lazy](https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html)
- Plan 102: HTTP Server Stdlib
- Plan 130: Workspace Scene Config
