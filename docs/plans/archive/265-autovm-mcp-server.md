# Plan 265: AutoVM MCP Server — AI-First VM Interaction Protocol — COMPLETE

**Status**: Complete
**Created**: 2026-05-25
**Related**: [Plan 266 (VM ↔ a2r Conformance)](266-vm-a2r-conformance.md)
**Scope**: `crates/auto-lang/src/mcp/`, `crates/auto-lang/src/autovm_persistent.rs`

## Background

Auto is designed as an AI-generated language — the end goal is that humans write zero Auto code; AI agents generate everything. This creates a unique requirement for the VM: it must serve as a **rapid iteration engine for AI agents** (generate → validate → fix loops), not just a human REPL.

Currently AutoVM already has:
- Persistent session (`AutovmReplSession`) preserving state across inputs
- Human REPL (`AutovmRepl`) with rustyline history
- JSON output mode in CLI (`--format json`, `--ai`)
- Structured diagnostics via `miette` (error codes, spans, help text)
- REST API in playground crate (`auto-playground`)

What's missing is a **first-class MCP (Model Context Protocol) server** that lets AI agents like Claude Code interact with AutoVM as a structured tool.

## Design Principles

1. **Dual-mode VM**: Human REPL unchanged; MCP is a parallel entry point sharing the same `AutovmReplSession` core
2. **Session isolation**: Each AI agent/task gets its own VM session (no state leakage)
3. **Structured everything**: All output is machine-parseable JSON; no text scraping
4. **Diagnostic-first errors**: Every error includes suggestions, not just messages
5. **Incremental by default**: `patch` a single function without re-parsing the module
6. **Type inference as query**: AI asks "what type is this?" without guessing

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    auto binary                        │
│                                                      │
│  ┌─────────┐   ┌──────────────┐   ┌──────────────┐  │
│  │  REPL   │   │  MCP Server  │   │  CLI --json  │  │
│  │ (human) │   │  (AI agent)  │   │  (fallback)  │  │
│  └────┬────┘   └──────┬───────┘   └──────┬───────┘  │
│       │               │                  │           │
│       └───────────────┼──────────────────┘           │
│                       ▼                              │
│            ┌─────────────────────┐                   │
│            │  Session Manager    │                   │
│            │  - create/reset     │                   │
│            │  - session store    │                   │
│            └──────────┬──────────┘                   │
│                       ▼                              │
│            ┌─────────────────────┐                   │
│            │ AutovmReplSession   │  ← existing core  │
│            │ (per-session VM)    │                    │
│            └─────────────────────┘                   │
└──────────────────────────────────────────────────────┘

MCP Protocol (stdio JSON-RPC):
  Claude Code ──stdio──► auto mcp ──► Session Manager ──► AutovmReplSession
```

## MCP Tool Definitions

### Tool 1: `auto_session_create`

Create a new isolated VM session.

```json
{
  "name": "auto_session_create",
  "description": "Create a new AutoVM session with isolated execution state. Returns session_id for use in subsequent calls.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "sandbox": { "type": "boolean", "default": false, "description": "Enable sandbox mode (no file I/O, no network)" }
    }
  }
}
```

Response:
```json
{
  "session_id": "ses_abc123",
  "status": "created"
}
```

### Tool 2: `auto_define`

Define functions, types, constants, or entire modules in a session.

```json
{
  "name": "auto_define",
  "description": "Define Auto code (functions, types, constants) in the session. Parses and compiles to bytecode but does not execute top-level expressions.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id", "code"],
    "properties": {
      "session_id": { "type": "string" },
      "code": { "type": "string", "description": "Auto code to define. Can be multiple items." },
      "replace": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Names of existing definitions to replace before defining new ones"
      }
    }
  }
}
```

Response (success):
```json
{
  "status": "ok",
  "defined": ["quicksort: fn([]int) -> []int", "MyType: type"],
  "diagnostics": []
}
```

Response (error with suggestions):
```json
{
  "status": "error",
  "defined": [],
  "diagnostics": [
    {
      "severity": "error",
      "code": "E001",
      "message": "cannot concatenate str and int",
      "span": { "line": 2, "col": 22, "end_col": 29, "source": "\"hello \" + name + 1" },
      "suggestions": [
        { "message": "convert int to str", "fix": "str(1)" },
        { "message": "use f-string", "fix": "f\"hello $name\"" }
      ]
    }
  ]
}
```

### Tool 3: `auto_evaluate`

Evaluate an expression and return its value and type.

```json
{
  "name": "auto_evaluate",
  "description": "Evaluate an Auto expression in the session. Returns the result value, its type, and any diagnostics. Supports both simple expressions and statements.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id", "code"],
    "properties": {
      "session_id": { "type": "string" },
      "code": { "type": "string", "description": "Expression or statement to evaluate" },
      "timeout_ms": { "type": "integer", "default": 5000, "description": "Execution timeout in milliseconds" },
      "capture_output": { "type": "boolean", "default": true, "description": "Capture stdout as part of result" }
    }
  }
}
```

Response:
```json
{
  "status": "ok",
  "value": "[1, 1, 3, 4, 5, 9]",
  "type": "[]int",
  "stdout": "",
  "diagnostics": [],
  "execution_time_ms": 2
}
```

Runtime error response:
```json
{
  "status": "runtime_error",
  "value": null,
  "diagnostics": [
    {
      "severity": "error",
      "code": "E102",
      "message": "index out of bounds: index 10, length 3",
      "span": { "line": 1, "col": 0, "end_col": 8, "source": "arr[10]" },
      "suggestions": [
        { "message": "check length first", "fix": "if idx < arr.len() { arr[idx] }" }
      ]
    }
  ]
}
```

### Tool 4: `auto_typecheck`

Type-check code without executing. Returns inferred types for all symbols.

```json
{
  "name": "auto_typecheck",
  "description": "Type-check Auto code without executing it. Returns inferred types for parameters, variables, and return types. Useful for AI agents to verify code correctness before execution.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id", "code"],
    "properties": {
      "session_id": { "type": "string" },
      "code": { "type": "string", "description": "Code to type-check" }
    }
  }
}
```

Response:
```json
{
  "status": "ok",
  "types": {
    "foo": { "signature": "fn(int) -> int", "params": { "x": "int" } },
    "result": "int"
  },
  "diagnostics": [
    {
      "severity": "info",
      "message": "parameter 'x' inferred as int from operator '+' with integer literal"
    }
  ]
}
```

### Tool 5: `auto_patch`

Incrementally update a single definition without resetting the session.

```json
{
  "name": "auto_patch",
  "description": "Replace a single function or type definition in the session. Faster than redefine because it patches bytecode in-place. Other definitions and session state are preserved.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id", "target", "code"],
    "properties": {
      "session_id": { "type": "string" },
      "target": { "type": "string", "description": "Name of the existing definition to replace" },
      "code": { "type": "string", "description": "New definition code" }
    }
  }
}
```

Response:
```json
{
  "status": "ok",
  "patched": "quicksort",
  "previous_signature": "fn([]int) -> []int",
  "new_signature": "fn([]int) -> []int",
  "diagnostics": []
}
```

### Tool 6: `auto_inspect`

Query the current session state.

```json
{
  "name": "auto_inspect",
  "description": "Query the current state of an AutoVM session. Returns defined functions, types, variables, and their signatures/values.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id"],
    "properties": {
      "session_id": { "type": "string" },
      "kind": {
        "type": "string",
        "enum": ["functions", "types", "variables", "scope", "all"],
        "default": "all",
        "description": "What to inspect"
      },
      "filter": { "type": "string", "description": "Optional name prefix filter" }
    }
  }
}
```

Response:
```json
{
  "session_id": "ses_abc123",
  "functions": [
    { "name": "quicksort", "signature": "fn([]int) -> []int", "is_public": true },
    { "name": "greet", "signature": "fn(str) -> void", "is_public": false }
  ],
  "types": [
    { "name": "MyRecord", "kind": "struct", "fields": ["x int", "y int"] }
  ],
  "variables": [
    { "name": "count", "type": "int", "value": "42", "mutable": true }
  ]
}
```

### Tool 7: `auto_snapshot`

Export the entire session as a runnable `.at` file.

```json
{
  "name": "auto_snapshot",
  "description": "Export the current session's accumulated definitions as a complete, runnable .at file. This bridges the scripting→compilation pipeline: iterate in the VM, then snapshot and compile with a2r.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id"],
    "properties": {
      "session_id": { "type": "string" },
      "include_tests": { "type": "boolean", "default": false, "description": "Include #[test] functions in output" }
    }
  }
}
```

Response:
```json
{
  "code": "fn quicksort(arr []int) []int {\n  ...\n}\n\nfn greet(name str) void {\n  ...\n}\n",
  "stats": { "functions": 2, "types": 0, "lines": 18 }
}
```

### Tool 8: `auto_session_reset`

Reset a session's state (or delete it).

```json
{
  "name": "auto_session_reset",
  "description": "Reset an AutoVM session to a clean state, or delete it entirely.",
  "inputSchema": {
    "type": "object",
    "required": ["session_id"],
    "properties": {
      "session_id": { "type": "string" },
      "action": {
        "type": "string",
        "enum": ["reset", "delete"],
        "default": "reset",
        "description": "reset = clear state but keep session, delete = remove session entirely"
      }
    }
  }
}
```

## Session Manager Design

```rust
// crates/auto-lang/src/mcp/session_manager.rs

pub struct SessionManager {
    sessions: HashMap<String, VmSession>,
}

struct VmSession {
    session: AutovmReplSession,
    created_at: Instant,
    last_active: Instant,
    sandbox: bool,
}

impl SessionManager {
    pub fn create(sandbox: bool) -> String;           // → session_id
    pub fn get(&mut self, id: &str) -> &mut AutovmReplSession;
    pub fn reset(&mut self, id: &str);
    pub fn delete(&mut self, id: &str);
    pub fn cleanup_expired(&mut self, max_idle: Duration); // GC old sessions
}
```

Session lifecycle:
- Created on `auto_session_create`
- Persist across tool calls within the same MCP connection
- Auto-expire after 30 minutes of inactivity (configurable)
- Explicitly deleted via `auto_session_reset { action: "delete" }`

## Diagnostic Schema (Unified)

All tools return diagnostics in the same format:

```json
{
  "severity": "error" | "warning" | "info",
  "code": "E001",
  "message": "human-readable description",
  "span": {
    "line": 2,
    "col": 22,
    "end_col": 29,
    "source": "the offending source text"
  },
  "suggestions": [
    {
      "message": "description of the fix",
      "fix": "replacement code (optional — some suggestions are conceptual)"
    }
  ]
}
```

Mapping from existing `miette` diagnostics:
- `AutoError` variants → `severity` (error/warning)
- `err.code()` → `code`
- `err.labels()` → `span`
- `err.help()` → first `suggestion`
- **New**: `suggestions` array needs to be added to error types (currently only `help` text exists)

## MCP Server Implementation

### Dependencies

Add to `crates/auto-lang/Cargo.toml`:
```toml
# MCP protocol support
rmcp = { version = "0.1", optional = true }  # or implement minimal JSON-RPC over stdio
```

Or implement a minimal MCP server from scratch (preferred — avoids heavy dependency):
- JSON-RPC 2.0 over stdio (read stdin, write stdout)
- MCP tool registration schema
- Request/response routing

### Entry Point

```bash
# Start MCP server (stdio mode, for Claude Code integration)
auto mcp

# With options
auto mcp --max-sessions 10 --session-timeout 1800
```

In `.claude/settings.json`:
```json
{
  "mcpServers": {
    "autovm": {
      "command": "auto",
      "args": ["mcp"],
      "description": "AutoVM — evaluate, typecheck, and iterate on Auto code"
    }
  }
}
```

### Module Structure

```
crates/auto-lang/src/mcp/
├── mod.rs              # MCP server entry point, JSON-RPC router
├── protocol.rs         # MCP protocol types (JSON-RPC 2.0 messages)
├── session_manager.rs  # Session lifecycle management
├── tools/
│   ├── mod.rs          # Tool dispatch
│   ├── define.rs       # auto_define implementation
│   ├── evaluate.rs     # auto_evaluate implementation
│   ├── typecheck.rs    # auto_typecheck implementation
│   ├── patch.rs        # auto_patch implementation
│   ├── inspect.rs      # auto_inspect implementation
│   ├── snapshot.rs     # auto_snapshot implementation
│   └── session.rs      # auto_session_create/reset implementation
└── diagnostic.rs       # AutoError → structured JSON diagnostic conversion
```

## Implementation Phases

### Phase 1: Minimal MCP Server + Core Tools

**Goal**: Claude Code can connect to `auto mcp`, create a session, define code, evaluate expressions, and get structured errors.

**Tasks**:
1. Create `crates/auto-lang/src/mcp/` module with JSON-RPC 2.0 stdio server
2. Implement session manager (create/get/reset/delete)
3. Implement `auto_session_create` — wraps `AutovmReplSession::new()`
4. Implement `auto_define` — parse code, add to session, return defined symbols + diagnostics
5. Implement `auto_evaluate` — execute expression in session, return value + type
6. Implement `auto_session_reset`
7. Add `auto mcp` CLI subcommand
8. Write integration test: full define → evaluate → error → fix cycle via MCP protocol

**Estimated effort**: ~2-3 days
**Files modified**: `Cargo.toml`, `src/lib.rs`, `crates/auto/src/main.rs`
**Files created**: `src/mcp/` directory (6-8 files)

### Phase 2: Diagnostic Enhancement + Type Inference Query

**Goal**: Every error includes actionable suggestions. AI can query types without executing.

**Tasks**:
1. Extend `AutoError` variants with `suggestions: Vec<Suggestion>` field
2. Implement suggestion generation for top 10 most common errors:
   - Type mismatch → suggest conversion functions
   - Undefined name → suggest similar names (Levenshtein)
   - Missing field → suggest available fields
   - Wrong arity → show expected vs actual params
   - Borrow after move → suggest clone/copy
3. Implement `auto_typecheck` — run type inference without execution
4. Create `diagnostic.rs` — convert `AutoError` → structured JSON diagnostic
5. Integration tests for diagnostic round-trips

**Estimated effort**: ~2-3 days

### Phase 3: Incremental Patch + Inspect + Snapshot

**Goal**: Full AI iteration workflow — patch individual functions, inspect state, export final code.

**Tasks**:
1. Implement `auto_patch` — replace a single definition in existing session bytecode
   - This requires modifying `AutovmReplSession` to support named-define replacement
   - Current implementation appends bytecode; need to support replacing by name
2. Implement `auto_inspect` — query session state (functions, types, variables)
   - Extract from `Codegen` (exports, locals) and `AutoVM` (heap, task state)
3. Implement `auto_snapshot` — export all definitions as `.at` source
   - Store original source text per definition in session
   - Concatenate and emit as complete file
4. Integration tests for patch/inspect/snapshot cycle

**Estimated effort**: ~3-4 days
**Key challenge**: `auto_patch` requires bytecode-level patching or selective recompilation

### Phase 4: Performance + Robustness

**Goal**: Production-ready MCP server with performance targets and safety.

**Tasks**:
1. Performance targets:
   - Session create: < 50ms
   - Define single function: < 10ms
   - Evaluate simple expression: < 5ms
   - Patch: < 10ms
2. Session timeout and cleanup (GC idle sessions)
3. Execution timeout enforcement (kill runaway code)
4. Sandbox mode (disable file I/O, network, FFI)
5. Concurrent session support (multiple agents in parallel)
6. Error recovery — malformed JSON-RPC requests don't crash the server
7. Stress test: 1000 rapid define/evaluate cycles

**Estimated effort**: ~2-3 days

### Phase 5: CLI `--json` Fallback + Documentation

**Goal**: Non-MCP users can still get structured output. Documentation for integration.

**Tasks**:
1. Add `auto eval --json` and `auto check --json` CLI shortcuts
   - These start a temporary session, execute one command, output JSON, exit
   - Stateless — no session management needed
2. Add `auto inspect --json` for querying `.at` files without running them
3. Write MCP integration guide:
   - How to configure in `.claude/settings.json`
   - Example Claude Code workflows
   - Tool reference documentation
4. Add to CLAUDE.md: "When generating Auto code, use AutoVM MCP tools for validation"

**Estimated effort**: ~1-2 days

## Key Design Decisions

### Decision 1: Minimal MCP vs rmcp crate

**Choice**: Implement minimal MCP server from scratch.

**Rationale**:
- MCP protocol over stdio is JSON-RPC 2.0 with tool registration — straightforward to implement (~200 lines)
- Avoids external dependency versioning issues
- Full control over error handling and session lifecycle
- The protocol surface is small (8 tools, no resources, no prompts needed initially)

### Decision 2: Session-per-agent vs Shared Session

**Choice**: Session-per-agent (each `auto_session_create` gets isolated state).

**Rationale**:
- Claude Code spawns sub-agents concurrently — shared state would cause race conditions
- AutoVM is not thread-safe internally (single-threaded bytecode execution)
- Isolation means one agent's bugs don't pollute another's state

### Decision 3: Patch via Bytecode Replace vs Full Recompile

**Choice**: Full recompile of patched item, but preserve session state.

**Rationale**:
- Bytecode patching is extremely fragile (offset changes cascade)
- Recompiling a single function is fast (< 1ms for typical functions)
- Session state (heap objects, other functions' bytecode) is preserved
- Implementation: remove old definition from Codegen's export table, re-parse + compile new definition, append new bytecode, update function address table

### Decision 4: Type Inference Scope

**Choice**: Best-effort type inference using existing `infer` module, not full Hindley-Milner.

**Rationale**:
- Auto already has `infer` module and `InferenceContext`
- For AI agents, "probably int" is good enough to catch obvious errors
- Full type inference can come later; current inference catches the most common cases

## Future Extensions (Out of Scope)

- **MCP Resources**: Expose `.at` files as MCP resources for direct editing
- **MCP Prompts**: Pre-built prompts for common Auto patterns
- **Sampling**: Let the VM request AI assistance (e.g., "suggest implementation for undefined function")
- **Remote MCP**: WebSocket transport for remote agent access
- **Multi-file sessions**: Sessions that span multiple `.at` files with imports
- **Coverage reporting**: Which code paths were exercised during evaluation

## Success Metrics

1. Claude Code can configure `auto mcp` and use it to validate generated Auto code without file round-trips
2. Error diagnostics include actionable suggestions in > 80% of cases
3. Full iteration cycle (define → evaluate → error → patch → evaluate) completes in < 50ms
4. Human REPL remains completely unchanged and functional
5. `auto snapshot` output passes through `a2r` without modification
