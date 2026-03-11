# Plan 121: AutoLang Async - Task/Msg System

## Status: 📋 PLANNING

## Objective

Implement AutoLang's async system using Task and Msg types that map to Rust's async/await, enabling concurrent operations without callback hell.

## Background

### Current State

AutoLang has conceptual support for async through:
- `Task<T>` - Represents a running async operation
- `Msg` - Message for inter-task communication

But this is **not yet implemented** for a2rs transpilation.

### Goals

1. Map AutoLang `Task<T>` to Rust `Future<Output = T>`
2. Implement `await` for blocking on tasks
3. Implement `spawn` for concurrent task execution
4. Implement `Msg` channels for task communication
5. Integrate with tokio runtime

## Design

### 1. Task Type

**AutoLang Syntax**:
```auto
// Task<T> represents an async operation
type Task<T>

// Create a task from async operation
let task: Task<Response> = http.get_async("https://api.example.com")

// Await a task (blocking)
let result = await task

// Spawn a task (non-blocking, returns handle)
let handle = spawn task

// Wait for multiple tasks concurrently
let results = await [task1, task2, task3]
```

**Rust Transpilation**:
```rust
// Task<T> maps to tokio::task::JoinHandle<T> or Future
use tokio::task::JoinHandle;

// Create task
let task: JoinHandle<Response> = tokio::spawn(async move {
    http_get_async("https://api.example.com").await
});

// await task -> task.await
let result = task.await?;

// spawn task -> tokio::spawn(task)
let handle = tokio::spawn(task);

// await [task1, task2, task3] -> futures::future::join_all
let results = futures::future::join_all(vec![task1, task2, task3]).await;
```

### 2. Async Functions

**AutoLang Syntax**:
```auto
// Async function declaration
async fn fetch_data(url str) !Response {
    let res = await http.get_async(url)
    Ok(res)
}

// Or with explicit return type
async fn fetch_all(urls []str) ![]Response {
    let tasks = urls.map(fn(url) { http.get_async(url) })
    let results = await tasks
    Ok(results)
}
```

**Rust Transpilation**:
```rust
// async fn -> async fn
async fn fetch_data(url: String) -> Result<Response, String> {
    let res = http_get_async(&url).await;
    Ok(res)
}

// await all -> join_all
async fn fetch_all(urls: Vec<String>) -> Result<Vec<Response>, String> {
    let tasks: Vec<_> = urls.into_iter().map(|url| http_get_async(url)).collect();
    let results = futures::future::join_all(tasks).await;
    Ok(results)
}
```

### 3. Msg Channel

**AutoLang Syntax**:
```auto
// Create a channel
let (tx, rx) = channel!<int>(100)  // Buffer size 100

// Send message
tx.send(42)

// Receive message (blocking)
let msg = rx.recv()

// Non-blocking receive
let msg = rx.try_recv()

// Close channel
tx.close()
```

**Rust Transpilation**:
```rust
use tokio::sync::mpsc;

// Create channel
let (tx, mut rx) = mpsc::channel::<i32>(100);

// Send
tx.send(42).await?;

// Receive
let msg = rx.recv().await;

// Non-blocking receive
let msg = rx.try_recv();

// Close (happens automatically when tx is dropped)
drop(tx);
```

### 4. Select Pattern

**AutoLang Syntax**:
```auto
// Select on multiple channels
select {
    msg = rx1.recv() => print(f"Got from rx1: ${msg}"),
    msg = rx2.recv() => print(f"Got from rx2: ${msg}"),
    timeout(1000) => print("Timed out!")
}
```

**Rust Transpilation**:
```rust
tokio::select! {
    msg = rx1.recv() => println!("Got from rx1: {}", msg),
    msg = rx2.recv() => println!("Got from rx2: {}", msg),
    _ = tokio::time::sleep(Duration::from_millis(1000)) => println!("Timed out!"),
}
```

## API Design

### HTTP Async

```auto
// stdlib/auto/http.vm.at

/// Async GET request
#[vm, async]
fn get_async(url str) Task<!Response>;

/// Async POST request
#[vm, async]
fn post_async(url str, body str) Task<!Response>;

/// Async request builder
#[vm, async]
fn RequestBuilder.send_async(self RequestBuilder) Task<!Response>;
```

### Redis Async

```auto
// stdlib/auto/redis.vm.at

/// Async GET
#[vm, async]
fn RedisClient.get_async(self RedisClient, key str) Task<?str>;

/// Async SET
#[vm, async]
fn RedisClient.set_async(self RedisClient, key str, val str) Task<!void>;

/// Async pipeline (batch commands)
#[vm, async]
fn RedisClient.pipeline_async(self RedisClient, cmds []Cmd) Task<![]Value>;
```

### SQLite Async

```auto
// stdlib/auto/sqlite.vm.at

/// Async query
#[vm, async]
fn Database.query_async(self Database, sql str) Task<!Rows>;

/// Async exec
#[vm, async]
fn Database.exec_async(self Database, sql str) Task<!void>;

/// Async transaction
#[vm, async]
fn Database.transaction_async(self Database, fn(db) !void) Task<!void>;
```

## Examples

### Example 1: Concurrent HTTP Requests

```auto
use auto.http

async fn fetch_all_pages(urls []str) ![]str {
    // Create tasks for each URL
    let tasks = urls.map(fn(url) {
        http.get_async(url)
    })
    
    // Wait for all concurrently
    let results = await tasks
    
    // Extract bodies
    let bodies = results.map(fn(res) {
        if res.is_err() {
            return ""
        }
        res.ok().body()
    })
    
    Ok(bodies)
}

fn main() {
    let urls = [
        "https://api.example.com/page1",
        "https://api.example.com/page2",
        "https://api.example.com/page3",
    ]
    
    match fetch_all_pages(urls) {
        Ok(bodies) => {
            for body in bodies {
                print(f"Fetched ${str.len(body)} bytes")
            }
        }
        Err(e) => print(f"Error: ${e}")
    }
}
```

### Example 2: Image Downloader with Channels

```auto
use auto.http
use auto.file
use auto.async

async fn download_image(url str, tx Sender<!str>) {
    match http.get_async(url).await {
        Ok(res) => {
            let filename = path.filename(url)
            let bytes = res.bytes()
            file.write_bytes(f"./downloads/${filename}", bytes)
            tx.send(Ok(filename))
        }
        Err(e) => tx.send(Err(e))
    }
}

async fn download_all(urls []str) !void {
    let (tx, rx) = channel!<str>(10)
    
    // Spawn download tasks
    for url in urls {
        spawn download_image(url, tx)
    }
    
    // Collect results
    let mut completed = 0
    let mut failed = 0
    
    for _ in 0..list.len(urls) {
        match rx.recv().await {
            Ok(filename) => {
                print(f"Downloaded: ${filename}")
                completed += 1
            }
            Err(e) => {
                print(f"Failed: ${e}")
                failed += 1
            }
        }
    }
    
    print(f"Done: ${completed} completed, ${failed} failed")
    Ok(())
}
```

### Example 3: Redis Pub/Sub with Task

```auto
use auto.redis
use auto.async

async fn subscribe_and_print(client RedisClient, channel str) !void {
    let (tx, rx) = channel!<str>(100)
    
    // Subscribe in background
    spawn {
        client.subscribe_async(channel, fn(msg) {
            tx.send(msg)
        })
    }
    
    // Process messages
    loop {
        match rx.recv().await {
            msg => print(f"[${channel}] ${msg}")
        }
    }
}
```

### Example 4: Rate-Limited API Client

```auto
use auto.http
use auto.async
use auto.time

type RateLimiter {
    interval_ms int
    last_call int64
}

impl RateLimiter {
    fn new(calls_per_sec int) RateLimiter {
        RateLimiter {
            interval_ms: 1000 / calls_per_sec,
            last_call: 0
        }
    }
    
    async fn wait(self) {
        let now = time.now_ms()
        let elapsed = now - self.last_call
        let wait_time = self.interval_ms - elapsed
        
        if wait_time > 0 {
            time.sleep_async(wait_time).await
        }
        
        self.last_call = time.now_ms()
    }
}

async fn rate_limited_get(limiter RateLimiter, url str) !Response {
    limiter.wait().await
    http.get_async(url).await
}
```

## Implementation Plan

### Phase 1: Task Type Core (3 days)

**Files to modify**:
- `crates/auto-lang/src/ast.rs` - Add Task type
- `crates/auto-lang/src/parser.rs` - Parse `async`, `await`, `spawn`
- `crates/auto-lang/src/lexer.rs` - Tokenize new keywords

**Tasks**:
- [ ] Add `Type::Task(Box<Type>)` variant
- [ ] Add `Expr::Await(expr)` variant
- [ ] Add `Expr::Spawn(expr)` variant
- [ ] Parse `async fn` declarations
- [ ] Parse `await expr` expressions
- [ ] Parse `spawn expr` expressions
- [ ] Parse `await [expr, ...]` for concurrent await

### Phase 2: Channel Types (2 days)

**Tasks**:
- [ ] Add `Type::Channel<T>` (or `Sender<T>`, `Receiver<T>`)
- [ ] Parse `channel!<T>(size)` expression
- [ ] Add `tx.send(val)` method
- [ ] Add `rx.recv()` and `rx.try_recv()` methods
- [ ] Add `select! { ... }` macro pattern

### Phase 3: Rust Transpilation (3 days)

**Files to modify**:
- `crates/auto-lang/src/trans/rust.rs`

**Tasks**:
- [ ] Generate async runtime initialization
- [ ] Transpile `async fn` to Rust `async fn`
- [ ] Transpile `Task<T>` to `JoinHandle<T>`
- [ ] Transpile `await task` to `task.await`
- [ ] Transpile `spawn task` to `tokio::spawn(task)`
- [ ] Transpile `await [tasks]` to `join_all(tasks).await`
- [ ] Transpile channels to `tokio::sync::mpsc`
- [ ] Transpile `select!` to `tokio::select!`

### Phase 4: Stdlib Async APIs (2 days)

**Files to modify**:
- `crates/auto-lang/src/a2r_std/http.rs`
- `crates/auto-lang/src/a2r_std/redis.rs`
- `crates/auto-lang/src/a2r_std/sqlite.rs`

**Tasks**:
- [ ] Implement `http_get_async` using `reqwest`
- [ ] Implement async Redis operations
- [ ] Implement async SQLite operations
- [ ] Add `#[async]` annotation support

### Phase 5: Testing (2 days)

**Test cases**:
- [ ] Async function definition and call
- [ ] Single task await
- [ ] Concurrent task await (join_all)
- [ ] Task spawn
- [ ] Channel send/receive
- [ ] Select pattern
- [ ] Async HTTP requests
- [ ] Async Redis operations
- [ ] Error propagation in async

### Phase 6: Integration (1 day)

**Tasks**:
- [ ] Add tokio dependency to generated Cargo.toml
- [ ] Add futures dependency
- [ ] Configure tokio runtime (multi-threaded)
- [ ] Update build system

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.x | Async runtime |
| futures | 0.3 | Future utilities |
| reqwest | 0.11 | Async HTTP client |

## Blocks

- Plan 119: a2rs Backend Stdlib (requires async for performant HTTP/Redis)

## Estimated Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| 1 | 3 days | Task type core |
| 2 | 2 days | Channel types |
| 3 | 3 days | Rust transpilation |
| 4 | 2 days | Stdlib async APIs |
| 5 | 2 days | Testing |
| 6 | 1 day | Integration |
| **Total** | **13 days** | |

## Success Criteria

- [ ] `async fn` compiles to Rust async fn
- [ ] `await task` blocks correctly
- [ ] `spawn task` runs concurrently
- [ ] `await [tasks]` runs in parallel
- [ ] Channels work for task communication
- [ ] `select!` pattern works
- [ ] Async HTTP client works
- [ ] All tests pass

## Open Questions

1. **Runtime**: Should we use `tokio` (standard) or `async-std`?
   - **Recommendation**: Tokio (best ecosystem, Axum compatible)

2. **Blocking Context**: How to handle `await` in non-async functions?
   - **Options**: (a) Compile error, (b) Auto-spawn runtime

3. **Cancellation**: How to support task cancellation?
   - **Options**: (a) Drop handle, (b) CancellationToken, (c) Both

4. **Panic Handling**: How to handle panics in spawned tasks?
   - **Options**: Catch and return as Err, or propagate
