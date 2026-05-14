# Plan 251: Merge auto-code-rs into auto-forge

## Status: Planning

## Goal

Consolidate two agent systems into one: merge `auto-code-rs` (simple CLI agent) into `auto-forge` (spec-driven multi-agent), preserving the best of both, and establishing a clear path to an Auto-language self-hosted version.

---

## 0. Current State

### auto-code-rs (`d:/autostack/auto-code-rs/`)
- **4 crates**: ac-api (1,740 LOC), ac-tools (1,190 LOC), ac-runtime (1,015 LOC), ac-cli (476 LOC)
- **Total core**: ~5,500 lines Rust
- **Dual LLM**: Anthropic + OpenAI with unified `Provider` enum
- **5 tools**: Bash, Read, Write, Edit, Grep — each in its own module
- **CLI REPL**: Interactive prompt with `:quit`, `:help`, `:reset`, `:messages`
- **Session persistence**: JSONL format at `~/.auto-code-rs/sessions/`
- **Context management**: Token estimation + automatic compaction
- **Permission system**: Allow / Ask / ReadOnly modes
- **Auto version**: 2,442 lines across 18 `.at` files in `auto-coder/`
- **Snapshots**: 4 progressive snapshots (step-00 through step-03)
- **Examples**: 41 mini-programs with Rust + Auto + round-trip variants

### auto-forge (`d:/autostack/auto-lang/crates/auto-forge/`)
- **Single crate**: ~7,571 lines Rust
- **Claude only**: No OpenAI support
- **6 tools**: Same 5 file/shell tools + spec management (jade tools)
- **HTTP API**: Axum server on port 3031 with Web UI
- **Forge**: Chat-based assistant with spec lifecycle management
- **Relay**: Multi-agent orchestration (7 professions, pipeline state machine, handoffs, budgets)
- **No CLI mode**: Web-only
- **No Auto version**: Pure Rust
- **No session persistence**: In-memory only

### What overlaps (directly mergeable)
| Component | auto-code-rs | auto-forge | Similarity |
|-----------|-------------|------------|------------|
| Tool trait | `Tool: Send + Sync` → `Result<String, ToolError>` | `Tool: Send + Sync` → `Result<String, String>` | 95% |
| Tool registry | `HashMap<String, Box<dyn Tool>>` | Same | 98% |
| Bash tool | `BashTool` with timeout, safety blocks | `ShellTool` with safety blocks | 85% |
| Read tool | `ReadTool` with offset/limit | `ReadFileTool` basic | 80% |
| Write tool | `WriteTool` atomic write | `WriteFileTool` basic | 80% |
| Edit tool | `EditTool` unique-string check | `EditFileTool` same logic | 90% |
| Grep tool | `GrepTool` with context lines | `SearchTool` basic | 75% |
| Message types | `InputMessage`, `InputContentBlock`, `OutputContentBlock` | `ChatMessage`, `ContentBlock` | 85% |
| Streaming events | `StreamEvent` (low-level SSE) | `ToolChatEvent` (high-level) | Same data, different abstraction |
| ReAct loop | `Agent::run_turn()` | `ForgeSession` chat + `AgentTurn::run()` | Same pattern |

### What's unique (must port or preserve)

**From auto-code-rs only:**
- OpenAI provider + format translation (`ac-api/openai.rs`, 939 LOC)
- SSE parser (`ac-api/sse.rs`, 537 LOC)
- CLI REPL + settings (`ac-cli/`, 476 LOC)
- Session persistence JSONL (`ac-runtime/session.rs`, 137 LOC)
- Context compaction (`ac-runtime/context.rs`, 176 LOC)
- Permission modes (`ac-runtime/permission.rs`, 93 LOC)
- All Auto language files (`auto-coder/src/*.at`, 2,442 LOC)
- Progressive snapshots (`auto-code-rs/snapshots/`, 3 steps)
- 41 example programs (`ac-examples/`, many files)

**From auto-forge only (keep as-is):**
- Relay multi-agent orchestration (pipeline, professions, handoffs, budget)
- Spec-driven development (.ad files, status derivation)
- HTTP API + Web UI
- Human approval gates
- Checkpoint/resume system

---

## 1. Target Architecture

After merge, `auto-forge` gains three new capabilities while keeping its existing architecture intact:

```
crates/auto-forge/
├── src/
│   ├── main.rs                    # HTTP server (existing) + CLI entry (new)
│   ├── lib.rs
│   │
│   ├── provider/                  # NEW: unified LLM provider layer
│   │   ├── mod.rs                 #   LlmProvider trait, Provider enum
│   │   ├── claude.rs              #   ← forge/ai.rs (refactored)
│   │   ├── openai.rs              #   ← ac-api/openai.rs (ported)
│   │   ├── sse.rs                 #   ← ac-api/sse.rs (ported)
│   │   └── types.rs               #   unified message + stream types
│   │
│   ├── tools/                     # NEW: merged tool system
│   │   ├── mod.rs                 #   unified Tool trait + ToolRegistry
│   │   ├── bash.rs                #   ← ac-tools/bash.rs (best of both)
│   │   ├── read.rs                #   ← ac-tools/file_read.rs
│   │   ├── write.rs               #   ← ac-tools/file_write.rs
│   │   ├── edit.rs                #   ← ac-tools/file_edit.rs
│   │   ├── grep.rs                #   ← ac-tools/grep.rs
│   │   └── specs.rs               #   ← forge/tools.rs (jade tools only)
│   │
│   ├── context.rs                 # NEW: ← ac-runtime/context.rs
│   ├── session.rs                 # NEW: ← ac-runtime/session.rs
│   ├── permission.rs              # NEW: ← ac-runtime/permission.rs
│   │
│   ├── forge/                     # EXISTING (updated imports)
│   │   ├── mod.rs                 #   ForgeSession → uses tools/ + provider/
│   │   └── templates/
│   │
│   ├── relay/                     # EXISTING (updated imports)
│   │   ├── mod.rs
│   │   ├── agent.rs              #   AgentInstance → uses provider/
│   │   ├── turn.rs               #   AgentTurn → uses tools/
│   │   ├── pipeline.rs
│   │   ├── profession.rs
│   │   └── ...
│   │
│   └── cli/                       # NEW: CLI interface
│       ├── mod.rs                 #   CLI mode entry point
│       ├── repl.rs                #   ← ac-cli/repl.rs
│       └── settings.rs            #   ← ac-cli/settings.rs
│
├── tests/                         # EXISTING + ported tests
│   ├── relay_integration.rs
│   └── provider_integration.rs    # NEW: ← ac-api/tests/integration.rs
│
└── Cargo.toml                     # Updated deps (add clap, walkdir, etc.)
```

### New top-level modules

| Module | Source | Purpose |
|--------|--------|---------|
| `provider/` | ac-api + forge/ai.rs | Unified LLM access (Claude + OpenAI) |
| `tools/` | ac-tools + forge/tools.rs | All tools in one place |
| `context.rs` | ac-runtime/context.rs | Token estimation + compaction |
| `session.rs` | ac-runtime/session.rs | JSONL session persistence |
| `permission.rs` | ac-runtime/permission.rs | Allow/Ask/ReadOnly modes |
| `cli/` | ac-cli/ | CLI REPL interface |

### Removed modules (replaced)

| Old location | New location |
|-------------|-------------|
| `forge/ai.rs` (ToolClaudeProvider) | `provider/claude.rs` |
| `forge/tools.rs` (all tools) | `tools/` directory |
| `ai.rs` (base ClaudeProvider) | `provider/claude.rs` |

---

## 2. Phase 1 — Unified Provider Layer

**Goal**: Extract LLM communication into a `provider/` module that supports both Claude and OpenAI.

**Estimated effort**: 2-3 days

### Step 1.1: Create unified message types

**File**: `src/provider/types.rs`

Merge `ac-api/types.rs` with `forge/ai.rs` types into a single canonical set:

```rust
// Message types (union of both, keeping auto-code-rs names as canonical)
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

pub enum Role { User, Assistant }

pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

// API request (from ac-api, more complete)
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub system: Option<String>,
    pub tools: Vec<ToolDef>,
    pub tool_choice: Option<ToolChoice>,
    pub stream: bool,
    pub temperature: Option<f64>,
}

// Streaming events (use auto-forge's high-level abstraction)
pub enum StreamEvent {
    TextDelta { text: String },
    ToolUse { id: String, name: String, input: Value },
    Done { usage: Usage },
    Error { message: String },
}

pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}
```

**Migration mapping**:
| auto-code-rs type | auto-forge type | Unified type |
|---|---|---|
| `InputMessage` | `ChatMessage` | `Message` |
| `InputContentBlock` | `ContentBlock` | `ContentBlock` |
| `OutputContentBlock` | (no equivalent) | `ContentBlock` |
| `StreamEvent` (13 variants) | `ToolChatEvent` (4 variants) | `StreamEvent` (4 variants) |
| `ApiRequest` | `ToolChatRequest` | `LlmRequest` |
| `ToolDefinition` | `ToolDefinition` | `ToolDef` |
| `Usage` | (in handoff) | `Usage` |

### Step 1.2: Port SSE parser

**File**: `src/provider/sse.rs`

Port `ac-api/sse.rs` (537 LOC) directly. This is a clean, self-contained module with no dependency on other auto-code-rs types. Only change: update imports to use the unified `StreamEvent` type from `types.rs`.

Key functions to port:
- `SseParser::new()` / `SseParser::feed()` — incremental SSE frame parsing
- Frame → `StreamEvent` conversion

### Step 1.3: Port Claude provider

**File**: `src/provider/claude.rs`

Refactor from two existing implementations:
- `ai.rs::ClaudeProvider` (base provider, streaming chat)
- `forge/ai.rs::ToolClaudeProvider` (tool-enabled chat)

Create a single `ClaudeProvider` that handles both streaming and tool use:

```rust
pub struct ClaudeProvider {
    client: reqwest::Client,
    api_key: Option<String>,
    base_url: String,
}

impl ClaudeProvider {
    pub fn new() -> Self;
    pub fn is_available(&self) -> bool;
    pub async fn chat_stream(&self, request: LlmRequest, tx: UnboundedSender<StreamEvent>) -> Option<Usage>;
}
```

### Step 1.4: Port OpenAI provider

**File**: `src/provider/openai.rs`

Port `ac-api/openai.rs` (939 LOC). This includes:
- Message format translation (Claude ↔ OpenAI)
- OpenAI streaming (SSE format differs from Claude)
- Retry logic with exponential backoff

Key: The OpenAI provider implements the same `chat_stream()` method, translating request/response formats internally.

### Step 1.5: Create provider trait + enum

**File**: `src/provider/mod.rs`

```rust
pub trait LlmProvider: Send + Sync {
    fn chat_stream(&self, request: LlmRequest, tx: UnboundedSender<StreamEvent>) -> Pin<Box<dyn Future<Output = Option<Usage>> + Send + '_>>;
}

pub enum Provider {
    Claude(ClaudeProvider),
    OpenAi(OpenAiProvider),
}

impl Provider {
    pub fn chat_stream(&self, ...) -> ...; // delegates to inner
}
```

### Step 1.6: Update consumers

- `forge/mod.rs` — Replace `AIProviderState` with `Provider`, update `ForgeSession` to use unified types
- `relay/agent.rs` — `ModelConfig::Provider` enum stays, but now backed by real `Provider` implementations
- `relay/turn.rs` — `AgentTurn::run()` uses `Provider` instead of `ToolClaudeProvider`

### Step 1.7: Remove old files

- Delete `src/ai.rs` (replaced by `provider/claude.rs`)
- Delete `src/forge/ai.rs` (replaced by `provider/claude.rs`)

### Step 1.8: Port integration tests

**File**: `tests/provider_integration.rs`

Port `ac-api/tests/integration.rs` (228 LOC) — update imports to use unified types.

### Verification

- [ ] `cargo build -p auto-forge` succeeds
- [ ] `cargo test -p auto-forge` passes (existing tests updated)
- [ ] Claude provider works for both Forge and Relay
- [ ] OpenAI provider works (new capability)

---

## 3. Phase 2 — Unified Tool System

**Goal**: Merge tool implementations into a `tools/` module, keeping the best version of each tool.

**Estimated effort**: 1-2 days

### Step 2.1: Define unified Tool trait

**File**: `src/tools/mod.rs`

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    fn execute(&self, args: Value) -> Result<String, ToolError>;
    fn is_read_only(&self) -> bool { false }
}

pub enum ToolError {
    ExecutionFailed(String),
    InvalidInput(String),
    PermissionDenied(String),
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}
```

**Design decisions**:
- Use `&'static str` for name (auto-forge's convention)
- Use `ToolError` enum (auto-code-rs's convention — more structured than `String`)
- Keep `is_read_only()` from auto-code-rs (useful for permission system)

### Step 2.2: Port bash tool (merge best of both)

**File**: `src/tools/bash.rs`

Take auto-code-rs's `BashTool` as the base (has timeout support, better safety checks) and merge auto-forge's `ShellTool` workspace awareness:

```rust
pub struct BashTool {
    timeout_secs: u32,         // from ac-tools (default 120, max 600)
    working_dir: Option<PathBuf>,  // from forge ShellTool
}
```

Port from `ac-tools/bash.rs` (286 LOC), merge workspace path from `forge/tools.rs` ShellTool.

### Step 2.3: Port read tool

**File**: `src/tools/read.rs`

Port from `ac-tools/file_read.rs` (170 LOC) — has offset/limit support that auto-forge's `ReadFileTool` lacks.

### Step 2.4: Port write tool

**File**: `src/tools/write.rs`

Port from `ac-tools/file_write.rs` (158 LOC) — has atomic write operations.

### Step 2.5: Port edit tool

**File**: `src/tools/edit.rs`

Port from `ac-tools/file_edit.rs` (233 LOC) — has unique-string verification. Auto-forge's `EditFileTool` is similar but less robust.

### Step 2.6: Port grep tool

**File**: `src/tools/grep.rs`

Port from `ac-tools/grep.rs` (324 LOC) — has context line support (`-C`/`-B`/`-A`), case-insensitive flag, better output formatting. Much more complete than auto-forge's `SearchTool`.

### Step 2.7: Extract spec tools

**File**: `src/tools/specs.rs`

Extract the jade tools from `forge/tools.rs`:
- `ReadJadeTool` → `specs.rs`
- `WriteJadeTool` → `specs.rs`
- `ListJadesTool` → `specs.rs`

Also extract `set_tool_context()` and thread-local context.

### Step 2.8: Update consumers

- `forge/mod.rs` — Replace `use crate::forge::tools::*` with `use crate::tools::*`
- `relay/turn.rs` — Replace `use crate::forge::tools::*` with `use crate::tools::*`
- `relay/profession.rs` — Update `allowed_tools` strings (mostly same names)

### Step 2.9: Remove old files

- Delete `src/forge/tools.rs` (replaced by `src/tools/`)

### Verification

- [ ] `cargo build -p auto-forge` succeeds
- [ ] `cargo test -p auto-forge` passes
- [ ] All 8 tools (bash, read, write, edit, grep, read_jade, write_jade, list_jades) register correctly
- [ ] Spec tools still have access to project/session context

---

## 4. Phase 3 — Port Runtime Infrastructure

**Goal**: Port context management, session persistence, and permission system from auto-code-rs.

**Estimated effort**: 1 day

### Step 3.1: Port context manager

**File**: `src/context.rs`

Port `ac-runtime/context.rs` (176 LOC) directly. This provides:
- Token estimation (chars / 4 heuristic)
- Automatic compaction when approaching limits
- Configurable `keep_recent` count

```rust
pub struct ContextManager {
    max_tokens: u32,
    keep_recent: usize,
}

impl ContextManager {
    pub fn new(max_tokens: u32) -> Self;
    pub fn maybe_compact(&self, messages: &mut Vec<Message>);
}
```

This complements auto-forge's Relay handoff system: Forge sessions use `ContextManager` for long-running chats, Relay agents use handoff documents.

### Step 3.2: Port session persistence

**File**: `src/session.rs`

Port `ac-runtime/session.rs` (137 LOC) directly. This provides:
- JSONL format for message storage
- Workspace-based session directory (`~/.auto-forge/sessions/<hash>/`)
- Load/append operations

```rust
pub struct Session {
    path: PathBuf,
}

impl Session {
    pub fn new(project_path: &str) -> Self;
    pub fn append(&self, message: &Message) -> Result<()>;
    pub fn load(&self) -> Result<Vec<Message>>;
}
```

Integrate with `ForgeSession` — auto-persist messages on each turn, auto-restore on session reconnect.

### Step 3.3: Port permission system

**File**: `src/permission.rs`

Port `ac-runtime/permission.rs` (93 LOC) directly. This provides:
- Three modes: Allow, Ask, ReadOnly
- Tool-level read/write classification
- `check(tool_name, is_write) -> PermissionDecision`

```rust
pub enum PermissionMode { Allow, Ask, ReadOnly }

pub struct PermissionPolicy {
    mode: PermissionMode,
}
```

For the CLI mode (Phase 4), this provides the user-facing permission prompts. For the web UI, Forge already has its own approval flow via spec changes.

### Verification

- [ ] `cargo build -p auto-forge` succeeds
- [ ] `cargo test -p auto-forge` passes
- [ ] Context compaction works on long conversations
- [ ] Session files are created and loadable
- [ ] Permission policy correctly filters tools

---

## 5. Phase 4 — CLI Mode

**Goal**: Add a `forge chat` CLI mode that provides a Claude Code-like REPL experience.

**Estimated effort**: 1-2 days

### Step 4.1: Port settings

**File**: `src/cli/settings.rs`

Port `ac-cli/settings.rs` (197 LOC):
- `Settings` struct with env, provider, model, api_key, base_url, permission
- `load()` from `~/.auto-forge/settings.json`
- `inject_env()` for environment variables
- `ensure_template()` for first-run setup

Adapt paths from `~/.auto-code-rs/` → `~/.auto-forge/`.

### Step 4.2: Port REPL

**File**: `src/cli/repl.rs`

Port `ac-cli/repl.rs` (66 LOC) — adapt to use unified `Provider` and `ToolRegistry`:

```rust
pub async fn run_repl(
    provider: Provider,
    tools: ToolRegistry,
    permission: PermissionPolicy,
) -> Result<(), Box<dyn std::error::Error>>
```

Commands: `:quit`, `:help`, `:reset`, `:messages`, `:tools`

### Step 4.3: Create CLI entry point

**File**: `src/cli/mod.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "auto-forge", about = "AI coding assistant")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start web UI server (default)
    Serve { port: Option<u16> },
    /// Interactive CLI chat mode
    Chat {
        /// Provider: claude or openai
        #[arg(long)]
        provider: Option<String>,
        /// Model name
        #[arg(long)]
        model: Option<String>,
    },
    /// Run a single task from a file
    Run { file: String },
}
```

### Step 4.4: Update main.rs

**File**: `src/main.rs`

Replace the current direct server startup with clap-based dispatch:

```rust
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Serve { port }) => start_server(port).await,
        Some(Commands::Chat { provider, model }) => start_cli(provider, model).await,
        Some(Commands::Run { file }) => run_task(&file).await,
        None => start_server(None).await,  // default: web server
    }
}
```

Add `clap` to `Cargo.toml` dependencies.

### Verification

- [ ] `cargo run -p auto-forge` starts web server (unchanged behavior)
- [ ] `cargo run -p auto-forge -- chat` starts CLI REPL
- [ ] `cargo run -p auto-forge -- serve --port 8080` starts on custom port
- [ ] CLI REPL supports all commands
- [ ] Settings loaded from `~/.auto-forge/settings.json`
- [ ] Both Claude and OpenAI providers work from CLI

---

## 6. Phase 5 — Port Auto Language Assets

**Goal**: Move the Auto language testbed into the auto-lang monorepo where it belongs.

**Estimated effort**: 1 day

### Step 5.1: Move Auto version of the coding agent

**Source**: `d:/autostack/auto-coder/src/*.at` (18 files, 2,442 LOC)
**Destination**: `d:/autostack/auto-lang/auto/coder/`

```
auto/coder/
├── pac.at
├── mod.at
├── main.at
├── provider/
│   ├── mod.at
│   ├── claude.at          ← anthropic.at renamed
│   ├── openai.at
│   ├── sse.at
│   └── types.at
├── tools/
│   ├── mod.at             ← tools.at
│   ├── bash.at            ← tool_bash.at
│   ├── read.at            ← tool_file_read.at
│   ├── write.at           ← tool_file_write.at
│   ├── edit.at            ← tool_file_edit.at
│   └── grep.at            ← tool_grep.at
├── agent.at
├── context.at
├── permission.at
├── repl.at
├── session.at
└── settings.at
```

These files already exist and are complete. The move is structural — reorganize to match the merged Rust layout.

### Step 5.2: Move progressive snapshots

**Source**: `d:/autostack/auto-code-rs/snapshots/`
**Destination**: `d:/autostack/auto-lang/test/auto-forge-snapshots/`

```
test/auto-forge-snapshots/
├── step-00-api-minimal/
│   ├── main.at            (740 LOC - current)
│   └── main.a2r.at
├── step-01-api-layer/
│   └── main.at            (1,143 LOC - current)
├── step-02-tool-system/
│   └── main.at            (560 LOC - current)
└── step-03-agent-runtime/  (NEW - to be created later)
    └── main.at
```

Future snapshots will cover:
- step-03: Agent runtime (context, session, permission)
- step-04: Forge (spec-driven chat)
- step-05: Relay (multi-agent pipeline)

### Step 5.3: Move example programs

**Source**: `d:/autostack/auto-code-rs/crates/ac-examples/src/`
**Destination**: `d:/autostack/auto-lang/test/a2r/auto-forge/`

Move the 41 example programs. Each has `main.rs` + `main.at` + `main.r2a.at` + `main.a2r.rs`. These are valuable as a2r transpiler test cases.

### Step 5.4: Create Auto-forge spec

**File**: `d:/autostack/auto-lang/docs/specs/` — add a new auto-forge spec section

This will be the first spec managed BY auto-forge ABOUT auto-forge. Meta, but useful:
- Architecture spec mapping the merged codebase
- Goal items for each migration phase
- Test items from the 41 example programs

### Verification

- [ ] All `.at` files moved to correct locations
- [ ] No `.at` files remain in `auto-coder/` or `auto-code-rs/`
- [ ] Example programs still work as a2r test cases
- [ ] `cargo test -p auto-lang -- trans` still passes

---

## 7. Phase 6 — Cleanup & Deprecation

**Goal**: Finalize the merge and clean up.

**Estimated effort**: 0.5 day

### Step 6.1: Verify all functionality

Run full test suite:
```bash
cargo test -p auto-forge           # All auto-forge tests
cargo test -p auto-lang            # All auto-lang tests (no regressions)
cargo build -p auto-forge          # Clean build
```

### Step 6.2: Update documentation

- Update `CLAUDE.md` to reflect merged architecture
- Update `docs/plans/247-autocoder-design.md` with completion status
- Update any references to `auto-code-rs` in other docs

### Step 6.3: Archive auto-code-rs

```bash
# Don't delete — archive for reference
mv d:/autostack/auto-code-rs d:/autostack/_archived/auto-code-rs
mv d:/autostack/auto-coder d:/autostack/_archived/auto-coder
```

These repos stay in git history. We archive instead of delete to preserve:
- Git history and commit messages
- Any in-progress work not yet ported
- The original example programs (Rust source)

### Step 6.4: Update workspace Cargo.toml

If `auto-forge` is referenced by other crates in the workspace, verify all paths are correct. Check `auto-playground` dependency on `auto-forge`.

---

## 8. Dependency Changes

### Add to `crates/auto-forge/Cargo.toml`

```toml
[dependencies]
# ... existing deps ...
clap = { version = "4", features = ["derive"] }    # CLI arg parsing (Phase 4)
walkdir = "2"                                        # Recursive dir traversal for grep (Phase 2)
thiserror = "2"                                      # Structured errors (Phase 1)
```

### No new crates

We deliberately keep everything in the single `auto-forge` crate rather than splitting into sub-crates. Rationale:
- The crate is ~12K LOC after merge — manageable in one crate
- Avoids circular dependency issues between Forge and Relay
- Simpler `Cargo.toml` management
- Can always extract later if needed

---

## 9. File-Level Migration Map

Complete mapping of every file that moves:

| Source (auto-code-rs) | Destination (auto-forge) | Phase | Action |
|---|---|---|---|
| `ac-api/src/types.rs` | `src/provider/types.rs` | 1 | Merge with forge types |
| `ac-api/src/sse.rs` | `src/provider/sse.rs` | 1 | Port directly |
| `ac-api/src/anthropic.rs` | `src/provider/claude.rs` | 1 | Merge with forge/ai.rs |
| `ac-api/src/openai.rs` | `src/provider/openai.rs` | 1 | Port directly |
| `ac-api/src/lib.rs` | (deleted) | 1 | Types moved to provider/ |
| `ac-tools/src/lib.rs` | `src/tools/mod.rs` | 2 | Merge trait + registry |
| `ac-tools/src/bash.rs` | `src/tools/bash.rs` | 2 | Port + merge with ShellTool |
| `ac-tools/src/file_read.rs` | `src/tools/read.rs` | 2 | Port directly |
| `ac-tools/src/file_write.rs` | `src/tools/write.rs` | 2 | Port directly |
| `ac-tools/src/file_edit.rs` | `src/tools/edit.rs` | 2 | Port directly |
| `ac-tools/src/grep.rs` | `src/tools/grep.rs` | 2 | Port directly |
| `ac-runtime/src/agent.rs` | (adapted into forge/mod.rs) | 1-2 | ReAct loop logic already exists |
| `ac-runtime/src/context.rs` | `src/context.rs` | 3 | Port directly |
| `ac-runtime/src/session.rs` | `src/session.rs` | 3 | Port directly |
| `ac-runtime/src/permission.rs` | `src/permission.rs` | 3 | Port directly |
| `ac-cli/src/main.rs` | `src/cli/mod.rs` | 4 | Adapt to unified types |
| `ac-cli/src/repl.rs` | `src/cli/repl.rs` | 4 | Port + adapt |
| `ac-cli/src/settings.rs` | `src/cli/settings.rs` | 4 | Port + adapt paths |
| `ac-api/tests/integration.rs` | `tests/provider_integration.rs` | 1 | Port + adapt imports |

### Files deleted from auto-forge (replaced)

| File | Replaced by | Phase |
|---|---|---|
| `src/ai.rs` | `src/provider/claude.rs` | 1 |
| `src/forge/ai.rs` | `src/provider/claude.rs` | 1 |
| `src/forge/tools.rs` | `src/tools/*.rs` | 2 |

### Auto language files moved

| Source | Destination | Phase |
|---|---|---|
| `auto-coder/src/*.at` (18 files) | `auto/coder/**/*.at` | 5 |
| `auto-code-rs/snapshots/` | `test/auto-forge-snapshots/` | 5 |
| `ac-examples/src/` | `test/a2r/auto-forge/` | 5 |

---

## 10. Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Breaking Forge web UI during provider refactor | Medium | High | Phase 1 keeps forge/ai.rs working until new provider/ is tested; only delete old files after tests pass |
| Breaking Relay turn execution | Medium | High | Phase 2 keeps forge/tools.rs until new tools/ is verified; relay tests cover this |
| Type mismatches between old and new message types | High | Medium | Phase 1 Step 1.1 creates explicit conversion functions; update one consumer at a time |
| OpenAI streaming format differences | Medium | Medium | Port SSE parser directly from auto-code-rs where it's already battle-tested |
| Session format incompatibility | Low | Low | New session format, no migration needed |
| Auto version files break during move | Low | Medium | Git preserves originals; move is copy-then-verify |

### Rollback strategy

Each phase is independently committable. If a phase causes issues:
```bash
git revert HEAD~1  # Roll back just that phase
```

The existing forge/ai.rs and forge/tools.rs are only deleted in the phase that replaces them, not before.

---

## 11. Execution Order

```
Week 1:
  Day 1-2: Phase 1 (Provider layer) — highest value, enables OpenAI
  Day 3:   Phase 2 (Tool system) — mechanical, low risk
  Day 4:   Phase 3 (Runtime infra) — small, self-contained

Week 2:
  Day 1-2: Phase 4 (CLI mode) — new feature, visible to user
  Day 3:   Phase 5 (Auto assets) — file moves, low risk
  Day 4:   Phase 6 (Cleanup) — final verification
```

### Commits

Each phase is one commit:
1. `feat(auto-forge): unified provider layer with Claude + OpenAI support`
2. `feat(auto-forge): unified tool system with 8 tools`
3. `feat(auto-forge): context management, session persistence, permissions`
4. `feat(auto-forge): CLI REPL mode`
5. `chore: move Auto language testbed from auto-code-rs`
6. `chore: archive auto-code-rs, update docs`

---

## 12. Future Work (Out of Scope)

These are not part of this merge but are natural next steps:

1. **Auto-forge in Auto language**: Extend the snapshots (step-03 through step-05) to cover the full merged codebase, including Relay multi-agent orchestration. This is the ultimate self-hosting goal.

2. **Provider abstraction for Relay**: Currently Relay uses `ModelConfig::Provider` as an enum without actual provider routing. After Phase 1, wire it up so Relay can route cheap tasks to OpenAI and expensive tasks to Claude.

3. **Context manager for Forge sessions**: After Phase 3, integrate `ContextManager` into `ForgeSession` for automatic compaction of long-running web chat sessions.

4. **Session persistence for web**: After Phase 3, persist Forge sessions to disk so they survive server restarts.
