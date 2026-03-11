# Plan 119: a2rs Backend Standard Library Enhancement

## Status: ⏳ BLOCKED (requires prerequisites)

## Prerequisites

This plan depends on two foundational features:

| Prerequisite | Plan | Status | Duration |
|--------------|------|--------|----------|
| Error Types (`?T` / `!T`) | [Plan 120](./120-error-types-option-result.md) | 📋 Planning | 10-14 days |
| Async System (Task/Msg) | [Plan 121](./121-async-task-msg-system.md) | 📋 Planning | 13 days |

**Recommendation**: Implement Plan 120 first (Error Types), then Plan 121 (Async), then this plan.

## Objective

Enhance AutoLang's capabilities as a backend language by building a comprehensive stdlib for HTTP, Redis, and SQLite - enabling developers to write backend services entirely in AutoLang while Rust handles the heavy lifting under the hood.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     User Code (.at files)                           │
│  use auto.http                                                      │
│  use auto.redis                                                     │
│  use auto.sqlite                                                    │
│                                                                     │
│  fn main() {                                                        │
│      let server = http.server()                                     │
│      server.get("/", fn(req) { http.ok("Hello") })                  │
│      server.listen("0.0.0.0:8080")                                  │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘
                              ↓ a2rs transpiler
┌─────────────────────────────────────────────────────────────────────┐
│                Generated Rust Code                                  │
│  use auto_lang::a2r_std::http::*;                                   │
│  use auto_lang::a2r_std::redis::*;                                  │
│  use auto_lang::a2r_std::sqlite::*;                                 │
│                                                                     │
│  fn main() {                                                        │
│      let server = http_server();                                    │
│      http_server_get(&server, "/", |req| http_ok("Hello"));         │
│      http_server_listen(&server, "0.0.0.0:8080");                   │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘
                              ↓ calls
┌─────────────────────────────────────────────────────────────────────┐
│           Rust Implementation (crates/auto-lang/src/a2r_std/)       │
│  - http.rs: HTTP server/client (using hyper or axum)                │
│  - redis.rs: Redis client (using redis crate)                       │
│  - sqlite.rs: SQLite client (using rusqlite)                        │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Dual Runtime Support

AutoLang supports **two runtime modes**:
- **AutoVM**: Bytecode interpreter (current default)
- **a2rs**: Transpiled Rust executable

For a2rs, we need a **separate stdlib implementation** in `a2r_std/` that provides Rust-native implementations.

### 2. FFI Pattern

Using the established pattern:
```rust
// Simple typed functions: use #[rust_fn] macro
#[auto_macros::rust_fn("Http.get")]
pub fn shim_http_get(url: String) -> Result<HttpResponse, String> {
    // Implementation
}

// Complex functions: manual shim with task/vm
pub fn shim_http_server_listen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Implementation
}
```

### 3. Type System

Define opaque types in AutoLang that map to Rust types:
```auto
// AutoLang
type Response  // opaque handle to Rust HttpResponse

// Rust (a2r_std)
pub struct HttpResponse { ... }
```

## Phase 1: HTTP Server (Static File Server) ⏱️ 3-4 days

**Goal**: Build an HTTP server like `python -m http.server`

### Deliverables

1. **stdlib/auto/http.vm.at** - Enhanced VM declarations with `#[vm]` annotations
2. **crates/auto-lang/src/a2r_std/http.rs** - Rust implementation for a2rs
3. **examples/a2rs/static_server.at** - Example: serve static files

### API Design

Using new `?T` (Option) and `!T` (Result) types from Plan 120:

```auto
// stdlib/auto/http.vm.at

/// HTTP Server
#[vm]
type Server

#[vm]
#[pub]
fn server() !Server;  // Result<Server, String> - creation can fail

#[vm]
#[pub]
fn Server.get(self Server, path str, handler fn(Request) !Response) Server;

#[vm]
#[pub]
fn Server.static(self Server, prefix str, dir str) !Server;  // dir check can fail

#[vm]
#[pub]
fn Server.listen(self Server, addr str) !void;  // binding can fail

/// Response helpers (never fail - always return valid Response)
#[vm]
#[pub]
fn ok(body str) Response;

#[vm]
#[pub]
fn not_found(msg str) Response;

#[vm]
#[pub]
fn internal_error(msg str) Response;

/// Request access (never fail - fields always exist)
#[vm]
#[pub]
fn Request.path(self Request) str;

#[vm]
#[pub]
fn Request.method(self Request) str;

#[vm]
#[pub]
fn Request.param(self Request, key str) ?str;  // param might not exist
```

### Async API (using Task from Plan 121)

```auto
// Async versions using Task<T>
#[vm, async]
#[pub]
fn get_async(url str) Task<!Response>;

#[vm, async]
#[pub]
fn post_async(url str, body str) Task<!Response>;

#[vm, async]
#[pub]
fn Server.listen_async(self Server, addr str) Task<!void>;
```

### Example: Static File Server

```auto
// examples/a2rs/static_server.at
use auto.http

fn main() {
    let server = http.server()
    
    // Serve static files from ./public
    server.static("/", "./public")
    
    // Custom route
    server.get("/api/health", fn(req) {
        http.ok("{\"status\": \"ok\"}")
    })
    
    print("Server running on http://0.0.0.0:8080")
    server.listen("0.0.0.0:8080")
}
```

### Implementation Tasks

- [ ] Create `crates/auto-lang/src/a2r_std/mod.rs`
- [ ] Create `crates/auto-lang/src/a2r_std/http.rs`
- [ ] Add dependencies: `tokio`, `hyper` (or `axum`)
- [ ] Implement `HttpServer` struct with route storage
- [ ] Implement `static_file_handler` for file serving
- [ ] Add MIME type detection
- [ ] Create test: `test/a2r/200_http_server/`
- [ ] Create example: `examples/a2rs/static_server.at`

---

## Phase 2: HTTP Client (Web Scraping) ⏱️ 2-3 days

**Goal**: Build an HTTP client for web scraping

### Deliverables

1. **stdlib/auto/http.vm.at** - Enhanced client API
2. **crates/auto-lang/src/a2r_std/http.rs** - Client implementation
3. **examples/a2rs/http_client.at** - Example: fetch web pages
4. **examples/a2rs/scraper.at** - Example: scrape data from pages

### API Design

```auto
// HTTP Client additions to http.vm.at

/// Simple GET request
#[vm]
#[pub]
fn get(url str) Response?;

/// Simple POST request  
#[vm]
#[pub]
fn post(url str, body str) Response?;

/// Request builder for advanced usage
#[vm]
type RequestBuilder

#[vm]
#[pub]
fn request(method str, url str) RequestBuilder;

#[vm]
#[pub]
fn RequestBuilder.header(self RequestBuilder, key str, value str) RequestBuilder;

#[vm]
#[pub]
fn RequestBuilder.body(self RequestBuilder, body str) RequestBuilder;

#[vm]
#[pub]
fn RequestBuilder.timeout(self RequestBuilder, ms int) RequestBuilder;

#[vm]
#[pub]
fn RequestBuilder.send(self RequestBuilder) Response?;

/// Response access
#[vm]
#[pub]
fn Response.status(self Response) int;

#[vm]
#[pub]
fn Response.body(self Response) str;

#[vm]
#[pub]
fn Response.headers(self Response) Map<str, str>;
```

### Example: Simple Scraper

```auto
// examples/a2rs/scraper.at
use auto.http
use auto.str
use auto.file

fn main() {
    let url = "https://example.com"
    let res = http.get(url)
    
    if res == nil {
        print("Failed to fetch page")
        return
    }
    
    let body = res.body()
    print(f"Status: ${res.status()}")
    print(f"Body length: ${str.len(body)}")
    
    // Extract title (simple regex-like parsing)
    let title_start = str.find(body, "<title>")
    let title_end = str.find(body, "</title>")
    
    if title_start >= 0 && title_end > title_start {
        let title = str.substr(body, title_start + 7, title_end)
        print(f"Title: ${title}")
    }
}
```

### Implementation Tasks

- [ ] Implement `http_get()` using `reqwest` or `hyper`
- [ ] Implement `http_post()` with body
- [ ] Implement `RequestBuilder` pattern
- [ ] Add timeout support
- [ ] Implement response parsing
- [ ] Add header access methods
- [ ] Create tests: `test/a2r/201_http_client/`
- [ ] Create example: `examples/a2rs/scraper.at`

---

## Phase 3: Image Scraper ⏱️ 2 days

**Goal**: Download all images from a webpage

### Deliverables

1. **stdlib/auto/http.vm.at** - Add binary download support
2. **stdlib/auto/file.vm.at** - Binary file operations
3. **examples/a2rs/image_scraper.at** - Complete image scraper

### API Additions

```auto
// Binary download support
#[vm]
#[pub]
fn Response.bytes(self Response) []byte;

#[vm]
#[pub]
fn Response.content_type(self Response) str;

// Enhanced file operations
#[vm]
#[pub]
fn File.write_bytes(path str, data []byte) Result<void, str>;
```

### Example: Image Scraper

```auto
// examples/a2rs/image_scraper.at
use auto.http
use auto.file
use auto.str
use auto.path

fn main() {
    let url = "https://example.com/gallery"
    let output_dir = "./downloaded_images"
    
    // Create output directory
    file.create_dir(output_dir)
    
    // Fetch page
    let res = http.get(url)
    if res == nil {
        print("Failed to fetch page")
        return
    }
    
    let body = res.body()
    
    // Find all image URLs (simplified)
    let images = extract_image_urls(body, url)
    
    print(f"Found ${list.len(images)} images")
    
    // Download each image
    for img_url in images {
        download_image(img_url, output_dir)
    }
    
    print("Done!")
}

fn extract_image_urls(html str, base_url str) List<str> {
    let images = List.new()
    
    // Simple extraction (real implementation would use proper parsing)
    var pos = 0
    loop {
        let start = str.find_from(html, "<img src=\"", pos)
        if start < 0 { break }
        
        let src_start = start + 10
        let src_end = str.find_from(html, "\"", src_start)
        if src_end < 0 { break }
        
        let src = str.substr(html, src_start, src_end)
        
        // Handle relative URLs
        let full_url = if str.starts_with(src, "http") {
            src
        } else {
            str.trim_end(base_url, "/") + "/" + str.trim_start(src, "/")
        }
        
        images.push(full_url)
        pos = src_end
    }
    
    images
}

fn download_image(url str, output_dir str) {
    print(f"Downloading: ${url}")
    
    let res = http.get(url)
    if res == nil {
        print(f"  Failed to download")
        return
    }
    
    // Extract filename from URL
    let filename = path.filename(url)
    if str.is_empty(filename) {
        filename = f"image_${time.now_ms()}.jpg"
    }
    
    let output_path = path.join(output_dir, filename)
    
    // Save binary data
    let bytes = res.bytes()
    match file.write_bytes(output_path, bytes) {
        Ok(_) => print(f"  Saved: ${output_path}"),
        Err(e) => print(f"  Error: ${e}")
    }
}
```

### Implementation Tasks

- [ ] Implement `Response.bytes()` for binary data
- [ ] Enhance `File.write_bytes()` for binary writes
- [ ] Add `str.find_from()` for string searching
- [ ] Add `str.trim_start()`, `str.trim_end()` helpers
- [ ] Add `list.len()` method
- [ ] Create example: `examples/a2rs/image_scraper.at`
- [ ] Test with real websites

---

## Phase 4: Redis Client ⏱️ 3-4 days

**Goal**: Simple Redis client for caching and data storage

### Deliverables

1. **stdlib/auto/redis.at** - Redis API declarations
2. **stdlib/auto/redis.vm.at** - VM-specific implementations
3. **crates/auto-lang/src/a2r_std/redis.rs** - Rust implementation
4. **examples/a2rs/redis_demo.at** - Example usage

### API Design

```auto
// stdlib/auto/redis.at

/// Redis client
type RedisClient

/// Connect to Redis server
/// Example: let client = redis.connect("redis://127.0.0.1:6379")
#[pub]
fn connect(url str) RedisClient?;

/// Set a key-value pair
/// Example: client.set("user:1", "Alice")
#[pub]
fn RedisClient.set(self RedisClient, key str, value str) Result<void, str>;

/// Get a value by key
/// Example: let name = client.get("user:1")
#[pub]
fn RedisClient.get(self RedisClient, key str) str?;

/// Delete a key
/// Example: client.del("user:1")
#[pub]
fn RedisClient.del(self RedisClient, key str) Result<void, str>;

/// Check if key exists
/// Example: if client.exists("session:abc") { ... }
#[pub]
fn RedisClient.exists(self RedisClient, key str) bool;

/// Set with expiration (seconds)
/// Example: client.setex("session:abc", "data", 3600)
#[pub]
fn RedisClient.setex(self RedisClient, key str, value str, ttl int) Result<void, str>;

/// Set expiration on existing key
/// Example: client.expire("session:abc", 3600)
#[pub]
fn RedisClient.expire(self RedisClient, key str, ttl int) Result<void, str>;

/// Increment a counter
/// Example: let count = client.incr("page_views")
#[pub]
fn RedisClient.incr(self RedisClient, key str) Result<i64, str>;

/// Decrement a counter
#[pub]
fn RedisClient.decr(self RedisClient, key str) Result<i64, str>;

/// Push to list (right)
#[pub]
fn RedisClient.rpush(self RedisClient, key str, value str) Result<i64, str>;

/// Pop from list (left)
#[pub]
fn RedisClient.lpop(self RedisClient, key str) str?;

/// Get list range
#[pub]
fn RedisClient.lrange(self RedisClient, key str, start int, stop int) List<str>;

/// Add to set
#[pub]
fn RedisClient.sadd(self RedisClient, key str, member str) Result<i64, str>;

/// Check set membership
#[pub]
fn RedisClient.sismember(self RedisClient, key str, member str) bool;

/// Hash operations
#[pub]
fn RedisClient.hset(self RedisClient, key str, field str, value str) Result<void, str>;

#[pub]
fn RedisClient.hget(self RedisClient, key str, field str) str?;

#[pub]
fn RedisClient.hgetall(self RedisClient, key str) Map<str, str>;

/// Close connection
#[pub]
fn RedisClient.close(self RedisClient) void;
```

### Example: Redis Demo

```auto
// examples/a2rs/redis_demo.at
use auto.redis
use auto.time

fn main() {
    // Connect to Redis
    let client = redis.connect("redis://127.0.0.1:6379")
    
    if client == nil {
        print("Failed to connect to Redis")
        return
    }
    
    print("Connected to Redis!")
    
    // Basic string operations
    client.set("greeting", "Hello, Redis!")
    let value = client.get("greeting")
    print(f"Greeting: ${value}")
    
    // Expiration
    client.setex("session:abc", "user_data", 60)
    print("Set session with 60s TTL")
    
    // Counter
    client.set("counter", "0")
    for i in 0..5 {
        let count = client.incr("counter")
        print(f"Counter: ${count}")
    }
    
    // List operations
    client.del("mylist")
    client.rpush("mylist", "item1")
    client.rpush("mylist", "item2")
    client.rpush("mylist", "item3")
    
    let items = client.lrange("mylist", 0, -1)
    print(f"List items: ${items.len()}")
    for item in items {
        print(f"  - ${item}")
    }
    
    // Hash operations
    client.hset("user:1", "name", "Alice")
    client.hset("user:1", "email", "alice@example.com")
    
    let name = client.hget("user:1", "name")
    print(f"User name: ${name}")
    
    let user_data = client.hgetall("user:1")
    print(f"User data: ${user_data}")
    
    // Cleanup
    client.close()
    print("Done!")
}
```

### Implementation Tasks

- [ ] Add `redis` crate to Cargo.toml
- [ ] Create `stdlib/auto/redis.at` with type declarations
- [ ] Create `stdlib/auto/redis.vm.at` with `#[vm]` annotations
- [ ] Create `crates/auto-lang/src/a2r_std/redis.rs`
- [ ] Implement `RedisClient` struct wrapping `redis::Connection`
- [ ] Implement all string operations
- [ ] Implement list operations
- [ ] Implement hash operations
- [ ] Implement set operations
- [ ] Add error handling with `Result<T, str>`
- [ ] Register FFI functions in `register_stdlib_ffi()`
- [ ] Create test: `test/a2r/202_redis/`
- [ ] Create example: `examples/a2rs/redis_demo.at`

---

## Phase 5: SQLite Client ⏱️ 3-4 days

**Goal**: Simple SQLite client for database operations

### Deliverables

1. **stdlib/auto/sqlite.at** - SQLite API declarations
2. **stdlib/auto/sqlite.vm.at** - VM-specific implementations  
3. **crates/auto-lang/src/a2r_std/sqlite.rs** - Rust implementation
4. **examples/a2rs/sqlite_demo.at** - Example usage
5. **examples/a2rs/todo_app.at** - Mini TODO app example

### API Design

```auto
// stdlib/auto/sqlite.at

/// SQLite database connection
type Database

/// Open a SQLite database
/// Example: let db = sqlite.open("myapp.db")
#[pub]
fn open(path str) Database?;

/// Open in-memory database
/// Example: let db = sqlite.open_memory()
#[pub]
fn open_memory() Database?;

/// Execute a statement (no results)
/// Example: db.exec("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
#[pub]
fn Database.exec(self Database, sql str) Result<void, str>;

/// Execute with parameters
/// Example: db.exec_params("INSERT INTO users (name) VALUES (?)", ["Alice"])
#[pub]
fn Database.exec_params(self Database, sql str, params List<str>) Result<void, str>;

/// Query single row
/// Example: let row = db.query_one("SELECT name FROM users WHERE id = ?", ["1"])
#[pub]
fn Database.query_one(self Database, sql str, params List<str>) Row?;

/// Query multiple rows
/// Example: let rows = db.query("SELECT * FROM users")
#[pub]
fn Database.query(self Database, sql str) Rows;

/// Query with parameters
/// Example: let rows = db.query_params("SELECT * FROM users WHERE age > ?", ["18"])
#[pub]
fn Database.query_params(self Database, sql str, params List<str>) Rows;

/// Get last insert rowid
/// Example: let id = db.last_insert_rowid()
#[pub]
fn Database.last_insert_rowid(self Database) i64;

/// Begin transaction
#[pub]
fn Database.begin(self Database) Result<void, str>;

/// Commit transaction
#[pub]
fn Database.commit(self Database) Result<void, str>;

/// Rollback transaction
#[pub]
fn Database.rollback(self Database) Result<void, str>;

/// Close database
#[pub]
fn Database.close(self Database) void;

// ═══════════════════════════════════════════════════════════
// Row/Rows types
// ═══════════════════════════════════════════════════════════

/// A single row from a query
type Row

/// Get column value as string
#[pub]
fn Row.get(self Row, index int) str?;

/// Get column value by name
#[pub]
fn Row.get_named(self Row, name str) str?;

/// Get column as integer
#[pub]
fn Row.get_int(self Row, index int) i64?;

/// Get column as float
#[pub]
fn Row.get_float(self Row, index int) f64?;

/// Check if column is null
#[pub]
fn Row.is_null(self Row, index int) bool;

/// Collection of rows
type Rows

/// Iterate rows
#[pub]
fn Rows.next(self Rows) Row?;

/// Get row count
#[pub]
fn Rows.len(self Rows) int;

/// Collect all rows into a list
#[pub]
fn Rows.collect(self Rows) List<Row>;
```

### Example: SQLite Demo

```auto
// examples/a2rs/sqlite_demo.at
use auto.sqlite
use auto.json

fn main() {
    // Open database
    let db = sqlite.open("test.db")
    
    if db == nil {
        print("Failed to open database")
        return
    }
    
    print("Database opened!")
    
    // Create table
    db.exec("
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT,
            created_at INTEGER
        )
    ")
    
    // Insert data
    let now = time.now_sec()
    db.exec_params(
        "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)",
        ["Bob", "bob@example.com", f"${now}"]
    )
    
    let user_id = db.last_insert_rowid()
    print(f"Inserted user with ID: ${user_id}")
    
    // Query single row
    let row = db.query_one("SELECT * FROM users WHERE id = ?", [f"${user_id}"])
    
    if row != nil {
        let name = row.get_named("name")
        let email = row.get_named("email")
        print(f"User: ${name} (${email})")
    }
    
    // Query multiple rows
    let rows = db.query("SELECT id, name FROM users ORDER BY id")
    
    print("All users:")
    loop {
        let row = rows.next()
        if row == nil { break }
        
        let id = row.get_int(0)
        let name = row.get(1)
        print(f"  [${id}] ${name}")
    }
    
    // Transaction example
    db.begin()
    db.exec_params("UPDATE users SET email = ? WHERE id = ?", ["newemail@example.com", f"${user_id}"])
    db.commit()
    
    print("Updated user email!")
    
    // Cleanup
    db.close()
    
    // Delete test database
    file.delete("test.db")
    
    print("Done!")
}
```

### Example: TODO App

```auto
// examples/a2rs/todo_app.at
use auto.sqlite
use auto.http
use auto.json

// Simple TODO app with SQLite backend

type TodoItem {
    id int
    title str
    done bool
}

fn main() {
    let db = sqlite.open("todos.db")
    db.exec("CREATE TABLE IF NOT EXISTS todos (id INTEGER PRIMARY KEY, title TEXT, done INTEGER)")
    
    let server = http.server()
    
    // List todos
    server.get("/todos", fn(req) {
        let rows = db.query("SELECT id, title, done FROM todos")
        let todos = List.new()
        
        loop {
            let row = rows.next()
            if row == nil { break }
            
            let todo = TodoItem{
                id: row.get_int(0),
                title: row.get(1),
                done: row.get_int(2) == 1
            }
            todos.push(todo)
        }
        
        json_response(200, todos)
    })
    
    // Add todo
    server.post("/todos", fn(req) {
        let body = req.text()
        // Parse JSON and insert...
        http.created("Created")
    })
    
    // Toggle todo
    server.put("/todos/:id", fn(req) {
        let id = req.param("id")
        db.exec("UPDATE todos SET done = NOT done WHERE id = ?", [id])
        http.ok("Updated")
    })
    
    // Delete todo
    server.delete("/todos/:id", fn(req) {
        let id = req.param("id")
        db.exec("DELETE FROM todos WHERE id = ?", [id])
        http.ok("Deleted")
    })
    
    print("TODO API running on http://0.0.0.0:8080")
    server.listen("0.0.0.0:8080")
}
```

### Implementation Tasks

- [ ] Add `rusqlite` crate to Cargo.toml
- [ ] Create `stdlib/auto/sqlite.at` with type declarations
- [ ] Create `stdlib/auto/sqlite.vm.at` with `#[vm]` annotations
- [ ] Create `crates/auto-lang/src/a2r_std/sqlite.rs`
- [ ] Implement `Database` struct wrapping `rusqlite::Connection`
- [ ] Implement `exec()` and `exec_params()`
- [ ] Implement `query()` and `query_params()`
- [ ] Implement `Row` type with column access
- [ ] Implement `Rows` iterator
- [ ] Implement transaction support
- [ ] Add parameter binding for different types
- [ ] Register FFI functions
- [ ] Create test: `test/a2r/203_sqlite/`
- [ ] Create example: `examples/a2rs/sqlite_demo.at`
- [ ] Create example: `examples/a2rs/todo_app.at`

---

## Phase 6: Integration & Documentation ⏱️ 2 days

**Goal**: Polish, documentation, and integration testing

### Deliverables

1. **docs/a2rs-backend-guide.md** - User guide
2. **Integration tests** - End-to-end tests
3. **CI/CD** - Automated testing

### Tasks

- [ ] Write comprehensive user guide
- [ ] Add API documentation (rustdoc)
- [ ] Create integration tests with real services
- [ ] Set up test fixtures (test Redis, SQLite)
- [ ] Add CI workflow for a2rs tests
- [ ] Performance benchmarks
- [ ] Error handling best practices guide

---

## Directory Structure

```
crates/auto-lang/
├── src/
│   ├── a2r_std/
│   │   ├── mod.rs           # Module exports
│   │   ├── http.rs          # HTTP server/client
│   │   ├── redis.rs         # Redis client
│   │   └── sqlite.rs        # SQLite client
│   └── vm/
│       └── ffi/
│           └── stdlib.rs    # FFI registration (enhance existing)
├── test/
│   └── a2r/
│       ├── 200_http_server/
│       ├── 201_http_client/
│       ├── 202_redis/
│       └── 203_sqlite/
└── examples/
    └── a2rs/
        ├── static_server.at
        ├── scraper.at
        ├── image_scraper.at
        ├── redis_demo.at
        ├── sqlite_demo.at
        └── todo_app.at

stdlib/auto/
├── http.at                  # HTTP API (enhance existing)
├── http.vm.at               # VM FFI declarations (new)
├── redis.at                 # Redis API (new)
├── redis.vm.at              # VM FFI declarations (new)
├── sqlite.at                # SQLite API (new)
└── sqlite.vm.at             # VM FFI declarations (new)
```

---

## Dependencies to Add

```toml
# Cargo.toml additions
[dependencies]
tokio = { version = "1", features = ["full"] }
hyper = { version = "1", features = ["full"] }
http-body-util = "0.1"
hyper-util = "0.1"
reqwest = { version = "0.11", features = ["blocking", "json"] }
redis = "0.24"
rusqlite = { version = "0.31", features = ["bundled"] }
mime_guess = "2.0"  # For static file serving
```

---

## Success Criteria

### Phase 1 (HTTP Server)
- [ ] Static file server works with HTML/CSS/JS
- [ ] Custom routes can be registered
- [ ] Handles concurrent connections

### Phase 2 (HTTP Client)
- [ ] Can fetch web pages
- [ ] Supports POST/PUT/DELETE
- [ ] Handles timeouts and errors

### Phase 3 (Image Scraper)
- [ ] Can download images from a webpage
- [ ] Handles relative URLs
- [ ] Preserves file types

### Phase 4 (Redis)
- [ ] All basic operations work (get/set/del/incr)
- [ ] List/Hash/Set operations work
- [ ] Connection pooling works

### Phase 5 (SQLite)
- [ ] CRUD operations work
- [ ] Transactions work
- [ ] Parameterized queries prevent SQL injection

### Overall
- [ ] All examples compile and run
- [ ] All tests pass
- [ ] Documentation is complete

---

## Estimated Timeline

### Dependency Chain

```
Plan 120: Error Types (?T / !T)
    │
    └─→ Plan 121: Async System (Task/Msg)
            │
            └─→ Plan 119: Backend Stdlib (this plan)
```

### This Plan's Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| Phase 1 | 3-4 days | HTTP Server (axum) |
| Phase 2 | 2-3 days | HTTP Client (reqwest) |
| Phase 3 | 2 days | Image Scraper example |
| Phase 4 | 3-4 days | Redis Client |
| Phase 5 | 3-4 days | SQLite Client |
| Phase 6 | 2 days | Integration & Docs |
| **Total** | **15-19 days** | Full implementation |

### Complete Roadmap

| Step | Plan | Duration | Cumulative |
|------|------|----------|------------|
| 1 | Plan 120: Error Types | 10-14 days | 10-14 days |
| 2 | Plan 121: Async System | 13 days | 23-27 days |
| 3 | Plan 119: Backend Stdlib | 15-19 days | 38-46 days |
| **Total** | | **~6-8 weeks** | |

---

## Open Questions

1. **Async vs Sync**: Should a2rs use async (tokio) or blocking APIs?
   - **Recommendation**: Start with blocking for simplicity, add async later

2. **Connection Pooling**: Should Redis/SQLite use connection pools?
   - **Recommendation**: Single connection for MVP, pooling in future

3. **Error Handling**: How to handle errors in AutoLang?
   - **Recommendation**: Use `Result<T, str>` and `?` operator

4. **Type Safety**: How to handle dynamic types in SQLite?
   - **Recommendation**: Typed getters (`get_int`, `get_str`) like rusqlite

---

## Next Steps

1. Review and approve this plan
2. Start with Phase 1 (HTTP Server)
3. Create each phase as a separate PR for review
4. Document learnings and adjust plan as needed
