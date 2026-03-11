# a2rs Backend Examples

This directory contains examples demonstrating AutoLang's capabilities as a backend language using the a2rs (Auto-to-Rust) transpiler.

## Overview

Each example shows how to use AutoLang to define application logic while Rust handles the heavy lifting under the hood. The examples build upon each other, introducing new concepts progressively.

## Examples

### 1. Static File Server (`01_static_server.at`)
**Concepts**: HTTP Server, Static Files, Routing

A simple static file server similar to `python -m http.server`. Serves HTML, CSS, JS, and images from a directory.

```bash
auto build examples/a2rs/01_static_server.at
./build/01_static_server
# Open http://localhost:8080
```

**What you'll learn**:
- Creating an HTTP server
- Serving static files
- Adding custom routes
- Basic routing patterns

---

### 2. HTTP Client (`02_http_client.at`)
**Concepts**: HTTP Client, JSON, String Parsing

Demonstrates making HTTP requests and parsing responses.

```bash
auto build examples/a2rs/02_http_client.at
./build/02_http_client
```

**What you'll learn**:
- GET and POST requests
- Request builders
- Header manipulation
- JSON parsing
- Timeout handling

---

### 3. Image Scraper (`03_image_scraper.at`)
**Concepts**: Web Scraping, Binary Data, File I/O

Downloads all images from a webpage and saves them locally.

```bash
auto build examples/a2rs/03_image_scraper.at
./build/03_image_scraper https://example.com
```

**What you'll learn**:
- HTML parsing
- URL resolution
- Binary file I/O
- String manipulation
- Error handling

---

### 4. Redis Client (`04_redis_demo.at`)
**Concepts**: Redis, Caching, Data Structures

Demonstrates Redis operations including strings, lists, hashes, and sets.

```bash
# Start Redis first
docker run -d -p 6379:6379 redis

auto build examples/a2rs/04_redis_demo.at
./build/04_redis_demo
```

**What you'll learn**:
- Redis connection
- String operations (GET, SET, DEL)
- Expiration (SETEX, EXPIRE)
- Counters (INCR, DECR)
- Lists (RPUSH, LPOP, LRANGE)
- Hashes (HSET, HGET, HGETALL)
- Sets (SADD, SISMEMBER, SMEMBERS)

---

### 5. SQLite Database (`05_sqlite_demo.at`)
**Concepts**: SQLite, SQL, Transactions

Complete SQLite demonstration with tables, queries, and transactions.

```bash
auto build examples/a2rs/05_sqlite_demo.at
./build/05_sqlite_demo
```

**What you'll learn**:
- Database connection
- Table creation
- INSERT, UPDATE, DELETE
- SELECT queries
- Parameterized queries (SQL injection prevention)
- Transactions (BEGIN, COMMIT, ROLLBACK)
- Aggregate functions

---

### 6. TODO REST API (`06_todo_api.at`)
**Concepts**: REST API, HTTP + SQLite, Full Stack

A complete REST API for a TODO application combining HTTP server with SQLite.

```bash
auto build examples/a2rs/06_todo_api.at
./build/06_todo_api

# Test the API
curl http://localhost:8080/todos
curl -X POST -d '{"title":"Buy milk"}' http://localhost:8080/todos
curl -X PUT -d '{"title":"Buy almond milk"}' http://localhost:8080/todos/1
curl -X PATCH http://localhost:8080/todos/1/toggle
curl -X DELETE http://localhost:8080/todos/1
```

**What you'll learn**:
- REST API design
- Route parameter extraction
- JSON request/response handling
- Database integration
- CRUD operations
- API error handling

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   AutoLang Source (.at)                      │
│                                                              │
│  use auto.http                                               │
│  use auto.sqlite                                             │
│  use auto.redis                                              │
│                                                              │
│  fn main() {                                                 │
│      let server = http.server()                              │
│      server.get("/todos", handle_todos)                      │
│      server.listen("0.0.0.0:8080")                           │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ a2rs transpiler
┌─────────────────────────────────────────────────────────────┐
│                   Generated Rust Code                        │
│                                                              │
│  use auto_lang::a2r_std::http::*;                            │
│  use auto_lang::a2r_std::sqlite::*;                          │
│                                                              │
│  fn main() {                                                 │
│      let server = http_server();                             │
│      http_server_get(&server, "/todos", handle_todos);       │
│      http_server_listen(&server, "0.0.0.0:8080");            │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ calls
┌─────────────────────────────────────────────────────────────┐
│               Rust Implementation (a2r_std/)                 │
│                                                              │
│  - http.rs: tokio/hyper server                               │
│  - sqlite.rs: rusqlite wrapper                               │
│  - redis.rs: redis crate wrapper                             │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

### Rust Toolchain
```bash
rustup install stable
```

### AutoLang Compiler
```bash
git clone https://github.com/auto-stack/auto-lang
cd auto-lang
cargo build --release
```

### External Services (for Redis example)
```bash
docker run -d -p 6379:6379 redis
```

## Building Examples

```bash
# Build a single example
auto build examples/a2rs/01_static_server.at

# Run the compiled binary
./build/01_static_server
```

## Project Structure

```
examples/a2rs/
├── README.md                    # This file
├── 01_static_server.at          # HTTP file server
├── 02_http_client.at            # HTTP client & scraping
├── 03_image_scraper.at          # Image downloader
├── 04_redis_demo.at             # Redis operations
├── 05_sqlite_demo.at            # SQLite operations
└── 06_todo_api.at               # Complete REST API
```

## Related Documentation

- [Plan 119: a2rs Backend Stdlib](../../docs/plans/119-a2rs-backend-stdlib.md)
- [AutoLang Documentation](../../README.md)
- [Stdlib Reference](../../stdlib/auto/)

## Contributing

When adding new examples:

1. Follow the naming convention: `NN_description.at`
2. Include comprehensive comments
3. Add usage instructions
4. Update this README

## License

MIT License - See [LICENSE](../../LICENSE) for details.
