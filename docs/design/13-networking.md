# 13 - Networking (HTTP Server & Async I/O)

## Status

**Designed:**
- HTTP server standard library architecture (async, net, http, json, url, log, env modules)
- Module dependency hierarchy and API surface
- Dual-mode execution: AutoVM (FFI to Rust) and a2r (direct Rust transpilation)

**Planned:**
- Module implementation in `stdlib/auto/` with `.at` + `.vm.at` / `.rs.at` dual files
- Tokio-based async runtime integration
- WebSocket support
- TLS/HTTPS support

## Design

### Module Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  ┌─────────────────────────────────────────────────┐    │
│  │  http.Server                                     │    │
│  │  ├── route() — route registration                │    │
│  │  ├── middleware() — middleware chain              │    │
│  │  └── listen() — start listening                  │    │
│  └─────────────────────────────────────────────────┘    │
│                           │                              │
│  ┌─────────────────────────────────────────────────┐    │
│  │  http.Request / http.Response                    │    │
│  │  ├── Header operations                           │    │
│  │  ├── Body read/write                             │    │
│  │  ├── Cookie operations                           │    │
│  │  └── Query/Path parameters                       │    │
│  └─────────────────────────────────────────────────┘    │
│                           │                              │
│  ┌─────────────────────────────────────────────────┐    │
│  │  json / form / url                               │    │
│  │  ├── encode() / decode()                         │    │
│  │  └── type conversion                             │    │
│  └─────────────────────────────────────────────────┘    │
│                           │                              │
│  Infrastructure Layer                                    │
│  ┌─────────────────────────────────────────────────┐    │
│  │  net.TcpListener / net.TcpStream                 │    │
│  │  ├── bind() — bind address                       │    │
│  │  ├── accept() — accept connection                │    │
│  │  └── read/write — data transfer                  │    │
│  └─────────────────────────────────────────────────┘    │
│                           │                              │
│  ┌─────────────────────────────────────────────────┐    │
│  │  async runtime (tokio)                           │    │
│  │  ├── spawn() — task creation                     │    │
│  │  ├── async/await syntax                          │    │
│  │  └── Channel communication                      │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### Module Dependency Graph

```
                    ┌──────────┐
                    │   http   │
                    └────┬─────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │   net   │    │  json   │    │   url   │
    └────┬────┘    └─────────┘    └─────────┘
         │
         ▼
    ┌─────────┐
    │  async  │  ← Lowest layer, all I/O depends on this
    └────┬────┘
         │
         ├───────────────┐
         │               │
         ▼               ▼
    ┌─────────┐    ┌─────────┐
    │   log   │    │   env   │
    └─────────┘    └─────────┘
```

### Module Designs

#### 1. `async` — Async Runtime Module

The lowest-level module. All I/O operations depend on this.

```auto
// Spawn an async task
let handle = async.spawn(~{
    let data = fetch(url).await
    process(data)
})

// Channel communication
let (tx, rx) = async.channel<str>(capacity: 100)
tx.send("hello").await
let msg = rx.recv().await
```

**Implementation:**
- AutoVM: FFI to `tokio::spawn`, `tokio::sync::mpsc`
- a2r: Direct Rust `async fn` transpilation

#### 2. `net` — Network Module

TCP/UDP primitives built on the async runtime.

```auto
// TCP Server
let listener = net.TcpListener.bind("0.0.0.0:8080").?
loop {
    let (stream, addr) = listener.accept().await.?
    async.spawn(~{
        handle_connection(stream).await
    })
}

// TCP Client
let stream = net.TcpStream.connect("example.com:80").?
stream.write_all(c"GET / HTTP/1.1\r\n\r\n").await.?
let response = stream.read_to_string().await.?
```

#### 3. `http` — HTTP Server Module

High-level HTTP server with routing, middleware, and request/response abstractions.

```auto
// Server definition
let server = http.Server.new()

// Route registration
server.route("GET", "/users/:id", fn(req http.Request) http.Response {
    let id = req.params.get("id")
    let user = db.find_user(id.to_int()).?
    return http.Response.json(user)
})

// Middleware
server.middleware(fn(req http.Request, next fn) http.Response {
    print(f"[${req.method}] ${req.path}")
    return next(req)
})

// Start listening
server.listen("0.0.0.0:3000").await
```

**Request/Response Types:**
```auto
type http.Request {
    method str
    path str
    headers Map<str, str>
    query Map<str, str>
    params Map<str, str>
    body []byte

    fn json<T>() !T          // Parse body as JSON
    fn form() Map<str, str>  // Parse body as form data
}

type http.Response {
    status int
    headers Map<str, str>
    body []byte

    static fn json(data Any) http.Response
    static fn text(data str) http.Response
    static fn html(data str) http.Response
    static fn redirect(url str) http.Response
}
```

#### 4. `json` — JSON Module

```auto
// Encode
let json_str = json.encode(user)  // Any → JSON string

// Decode
let user User = json.decode(json_str).?  // JSON string → typed value

// Pretty print
let pretty = json.pretty_encode(user)
```

#### 5. `url` — URL Module

```auto
let parsed = url.parse("https://example.com/path?key=value")
print(parsed.scheme)    // "https"
print(parsed.host)      // "example.com"
print(parsed.query.get("key"))  // "value"

let encoded = url.encode({key: "value", name: "hello"})
// "key=value&name=hello"
```

#### 6. `log` — Logging Module

```auto
log.info("Server started on port 3000")
log.warn(f"High memory usage: ${mem}%")
log.error("Database connection failed", err)
log.debug("Request details", req)
```

#### 7. `env` — Environment Module

```auto
let port = env.get("PORT") ?? "3000"
let debug = env.get("DEBUG") == "true"
env.set("NODE_ENV", "production")
```

### Dual-Mode Execution

All stdlib modules use a dual-file pattern:

| File | Purpose | Used By |
|------|---------|---------|
| `http.at` | Auto source (public API) | Both modes |
| `http.vm.at` | VM-specific FFI bindings | AutoVM |
| `http.rs.at` | Rust-specific transpilation hints | a2r |

The public API in `.at` files is identical regardless of execution mode. The `.vm.at` files contain `#[vm]` function declarations that map to Rust FFI implementations. The `.rs.at` files contain `#[rust_fn]` hints for the Rust transpiler.

## Open Questions

- Should HTTP/2 and HTTP/3 be supported from the start, or added later?
- WebSocket: separate module or part of `http`?
- TLS: use `rustls` (pure Rust) or `openssl` bindings?
- Should `json` support streaming (JSON Lines) for large datasets?
- How to handle CORS in the HTTP server?

## Source Documents

- [raw/http-server-stdlib.md](raw/http-server-stdlib.md) — Complete HTTP server stdlib design (1339 lines)
- [raw/autovm-tokio.md](raw/autovm-tokio.md) — Tokio runtime integration
- [raw/autovm-streaming.md](raw/autovm-streaming.md) — Streaming execution model
