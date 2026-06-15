# 12 - Concurrency (Task/Actor Model)

## Status

**Implemented:**
- Task keyword parsing and AST representation (`ast/task.rs`)
- Basic task compilation to AutoVM bytecode with handler registration
- `task` block syntax with `on` message handlers

**Designed (4-Phase Architecture):**
- Phase 1: Actor base with TaskSystem bootstrap — `task` blocks, `enum` message protocol, `TaskSystem.start()` ignition
- Phase 2: State machine suspension — `~T` async type, `.await`, `ask/reply` bidirectional RPC, `TaskSystem.run` sync bridge
- Phase 3: Polymorphic routing — implicit union synthesis, `MessageContext` explicit context, omnipotent pattern matcher
- Phase 4: Micro-concurrency — `.go` operator for lightweight fork, structured concurrency

**Planned:**
- Full AutoVM task scheduler integration
- a2c state machine generation for embedded targets
- Tokio-based runtime for a2r backend

## Design

### Core Philosophy: Zero Shared Mutable State

Auto's concurrency model follows the Actor paradigm: each `task` is an isolated entity with private state, communicating only through typed messages. There are no shared mutable variables, no locks, no mutexes at the language level.

```
┌─────────────┐     typed msg      ┌─────────────┐
│  Task A      │ ─────────────────→ │  Task B      │
│  (private    │                    │  (private    │
│   state)     │ ←───────────────── │   state)     │
│              │     reply          │              │
│  mailbox: [] │                    │  mailbox: [] │
└─────────────┘                    └─────────────┘
```

### Phase 1: Actor Base & TaskSystem Bootstrap

The foundation layer establishes physical isolation boundaries and basic communication primitives.

**Syntax:**

```auto
// 1. Message protocol (strongly typed enum)
enum CounterMsg {
    Add(int)
    Reset
    Print
}

// 2. Static task entity
task CounterTask {
    count mut = 0

    fn start() ! {
        self.count = 0
        print("CounterTask Booted!")
    }

    // Implicit message pump (event routing core)
    on {
        Add(val) => {
            self.count += val
        }
        Reset => {
            self.count = 0
        }
        Print => {
            print("Current Count: ${self.count}")
        }
    }
}

// 3. System bootstrap entry (pure synchronous main)
fn main() ! {
    print("System pre-booting...")

    // Phase A: Queue messages before ignition
    CounterTask.send(Add(10))
    CounterTask.send(Print)

    print("Ignition!")

    // Phase B: Transfer main thread control to scheduler
    TaskSystem.start()

    // Compiler enforces: unreachable code after this point
}
```

**Key Design Decisions:**
- `main()` is a synchronous bootstrapper; `TaskSystem.start()` is the final statement
- Messages are queued before ignition — no task runs until `TaskSystem.start()`
- `TaskSystem.start()` maps to `tokio::runtime::Runtime::block_on()` in a2r, or cooperative scheduler in AutoVM
- Compiler statically verifies `TaskSystem.start()` is the last reachable statement in `main`

### Phase 2: State Machine Suspension & Bidirectional RPC

Introduces non-blocking time primitives and implicit bidirectional channels.

**`~T` Async Type:**
- Any block or function prefixed with `~` returns a `~T` (future/state machine)
- `.await` on `~T` suspends the current task and yields CPU to other tasks
- Type dimension reduction: `~T` + `.await` → `T`

**`ask` / `reply` Mechanism:**

```auto
enum DBMsg {
    QueryUser(int)
}

#[single]
task DBManager {
    on {
        QueryUser(id) => {
            let user_info = db_driver.find(id)
            reply user_info  // Compiler injects oneshot channel
        }
    }
}

task WebWorker {
    on {
        ProcessRequest => {
            // ask() creates implicit oneshot channel, returns ~User
            let user = DBManager.ask(QueryUser(1001)).await.?
            print("Got user: ${user.name}")
        }
    }
}
```

**a2r Degradation (Rust):**
```rust
// Compiler rewrites enum to include oneshot Sender:
pub enum DBMsg {
    QueryUser(i32, tokio::sync::oneshot::Sender<User>),
}

// ask().await degrades to:
let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
DBMANAGER_TX.try_send(DBMsg::QueryUser(1001, reply_tx))?;
let user = reply_rx.await?;

// reply degrades to:
let _ = reply_tx.send(user_info);
```

**Backpressure via `.await`:**
```auto
// If mailbox is full, sender suspends instead of failing
Logger.send(LogData("message")).await.?
```

**Sync Bridge (`TaskSystem.run`):**
```auto
// Execute async block in synchronous context (e.g., during boot)
let config_url = TaskSystem.run(~{
    let json = http_get("config.json").await.?
    return parse_url(json)
}).?
```

### Phase 3: Polymorphic Routing & Explicit Context

Eliminates boilerplate `enum` definitions for simple cases. Introduces `MessageContext` for explicit reply control.

**Implicit Union Synthesis:**
The compiler's AST double-pass automatically extracts all data types from `on` blocks and synthesizes a message envelope — no manual `enum` needed.

**Explicit Message Context:**

```auto
// ctx provides routing metadata and reply method
task NodeWorker {
    on(ctx) {
        "ping" => {
            ctx.reply("pong")
        }
        "get_data" => {
            is ctx.sender_id {
                (id) => { print("Request from ${id}") }
                !    => { print("Anonymous request") }
            }
            ctx.reply(db.query())
        }
    }
}

// Omit parameter for fire-and-forget tasks
task Logger {
    on {
        msg string => { write_to_disk(msg) }
    }
}
```

**`MessageContext` Internal Type:**
```auto
type MessageContext {
    sender_id ?u64
    trace_id string
    is_ask bool
    fn reply(payload Any) void
}
```

**Omnipotent Pattern Matching:**
- Literal match: `"start" => { ... }`, `404 => { ... }`
- Type capture: `url string => { ... }`, `u User => { ... }`
- Guard clauses: `x int if x > 100 => { ... }`
- Wildcard: `_ => { ... }`

### Phase 4: Micro-Concurrency (`.go` Operator)

Lightweight fork for fine-grained parallelism without full task ceremony.

```auto
// .go spawns a lightweight concurrent unit
let handle = compute_heavy(data).go
let result = handle.await

// Structured concurrency: go blocks auto-join on scope exit
fn process(items List<int>) {
    let handles = items.map(x => transform(x).go)
    // All handles complete before function returns
}
```

### Scheduler Architecture

**AutoVM (Dynamic):**
```
┌──────────────────────────────────────────┐
│              TaskSystem                   │
│  ┌─────────────────────────────────────┐ │
│  │  Scheduler (Tokio Runtime)          │ │
│  │  ├── Task A: eval_loop (Future)     │ │
│  │  ├── Task B: eval_loop (Future)     │ │
│  │  └── Task C: eval_loop (Future)     │ │
│  └─────────────────────────────────────┘ │
│  ┌─────────────────────────────────────┐ │
│  │  Message Router                     │ │
│  │  ├── mailbox_a: mpsc::Receiver      │ │
│  │  ├── mailbox_b: mpsc::Receiver      │ │
│  │  └── mailbox_c: mpsc::Receiver      │ │
│  └─────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

**a2c (Embedded/C):**
- Compiler generates stackless coroutine state machines
- Each `.await` becomes a `switch-case` state transition
- Local variables surviving across `.await` are promoted to task struct fields
- Oneshot channels degrade to FreeRTOS Queues or Task Notifications

### Lifetime Scoping

| Scope | Keyword | Lifetime |
|-------|---------|----------|
| `@Task` | declared in task body | Lives until task ends |
| `@Request` | declared in `on` handler | Lives until handler completes |
| `@Global` | `shared` keyword | Process lifetime |

## Open Questions

- Should `reply` remain a keyword in Phase 2, or be replaced by `ctx.reply()` method in Phase 3 from the start?
- Maximum mailbox capacity: fixed per task or configurable at runtime?
- Should tasks support priority levels for message processing?
- How to handle task panics: restart strategy vs. supervision trees?

## Source Documents

- [raw/task-msg.md](raw/task-msg.md) — Complete 4-phase actor system design (873 lines)
- [raw/autovm-task-msg.md](raw/autovm-task-msg.md) — AutoVM task integration rationale
- [raw/autovm-streaming.md](raw/autovm-streaming.md) — REPL streaming execution model
- [raw/autovm-tokio.md](raw/autovm-tokio.md) — Tokio runtime integration
- Plans: 121 (task system), 124 (async/await), 125 (polymorphic routing), 126 (.go operator), 128 (scheduler)
