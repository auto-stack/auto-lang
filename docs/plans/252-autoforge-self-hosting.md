# Plan 252: Auto-Forge Self-Hosting — Auto Language Version

## Status: Planning

## Goal

Write a complete Auto language version of auto-forge (the merged agent system from Plan 251), using the existing `auto-coder/` Auto files as a foundation, and adding the Forge spec system and Relay multi-agent orchestration that have no Auto version yet.

The end state: auto-forge can run itself via the Auto VM, and the progressive snapshots serve as both a real-world validation suite for the Auto language and a reference implementation for anyone building agent systems.

---

## 0. Starting Point

### What already exists in Auto (`d:/autostack/auto-coder/src/`)

18 `.at` files, 2,442 LOC — covers the simple single-agent from auto-code-rs:

| File | LOC | Status | What it covers |
|------|-----|--------|---------------|
| `types.at` | 210 | Complete types, stubbed HTTP | API message types, stream events, errors |
| `anthropic.at` | 157 | Types complete, HTTP stubbed | Claude API client |
| `openai.at` | 122 | Types complete, HTTP stubbed | OpenAI API client + format translation |
| `sse.at` | 154 | Complete logic | SSE frame parser |
| `tools.at` | 98 | Mostly complete (Map iteration TODO) | Tool trait, registry, dispatch |
| `tool_bash.at` | 210 | Complete logic | Shell command execution |
| `tool_file_read.at` | 137 | Complete logic | File reading |
| `tool_file_write.at` | 100 | Complete logic | File writing |
| `tool_file_edit.at` | 138 | Complete logic | String replacement editing |
| `tool_grep.at` | 250 | Complete logic | Regex file search |
| `agent.at` | 278 | Core loop complete, stream TODO | ReAct agent loop |
| `context.at` | 117 | Complete | Token estimation + compaction |
| `permission.at` | 38 | Complete | Permission modes |
| `repl.at` | 87 | Complete structure | CLI REPL |
| `session.at` | 73 | Complete structure | JSONL session persistence |
| `settings.at` | 93 | Complete structure | Config loading |
| `main.at` | 158 | Complete | CLI entry point |
| `mod.at` | 20 | Exports | Module definition |

### What does NOT exist yet (must be written)

These auto-forge components have no Auto version:

| Component | Rust LOC | Difficulty | Description |
|-----------|----------|-----------|-------------|
| `forge/mod.rs` | ~2,181 | Medium | Spec lifecycle, session management, status state machine |
| `relay/flow.rs` | ~165 | Low | Flow spec, steps, routing declarations |
| `relay/flows.rs` | ~108 | Low | Built-in flow factory functions |
| `relay/handoff.rs` | ~235 | Low-Medium | Handoff document rendering |
| `relay/budget.rs` | ~227 | Low-Medium | Token budget tracking |
| `relay/profession.rs` | ~364 | Medium | Profession definitions, permissions |
| `relay/soul.rs` | ~164 | Medium | Agent personality config loader |
| `relay/agent.rs` | ~260 | Medium | Agent instance + model config |
| `relay/turn.rs` | ~404 | High | Agent turn execution (ReAct loop with tool calls) |
| `relay/pipeline.rs` | ~720 | High | Pipeline state machine engine |
| `relay/store.rs` | ~369 | High | Thread-safe run storage |
| `relay/checkpoint.rs` | ~560 | Very High | File state snapshots, git integration |
| `relay/api.rs` | ~262 | Very High | HTTP API endpoints, SSE streaming |

**Total new Auto code needed**: ~5,000+ LOC (plus adapting existing 2,442 LOC)

### Auto VM capability assessment

| Capability Needed | Auto VM Status | Notes |
|---|---|---|
| HTTP client | ✅ Full | `http.get()`, `http.post()`, streaming, SSE |
| HTTP server | ✅ Full | Routing, JSON bodies, static files |
| JSON | ✅ Full | `json.parse()`, `json.encode()`, `json_get()` |
| File I/O | ✅ Full | `File.read_text()`, `File.write_text()`, walk, exists |
| Collections | ✅ Full | `List`, `HashMap`, `HashSet`, `VecDeque` |
| Async/actors | ✅ Full | `task` blocks, `~T` async type, message passing |
| Regex | ✅ Full | `regex.is_match()`, `regex.find_all()` |
| Process spawn | ✅ Full | `Process.spawn()` |
| Time | ✅ Full | `time.now_ms()`, `time.sleep_ms()` |
| Environment | ✅ Full | `Env.get()`, `Env.set()` |
| SHA-256 | ⚠️ Needs FFI | Not native, use `use.rust` bridge |
| Module imports | ⚠️ Partial | Multi-file works but needs formalization |

**Verdict**: Auto VM has everything needed. The hardest parts (async HTTP, shared state) map to Auto's actor model. No language design work is blocking.

---

## 1. Target Directory Structure

After completion, the Auto version lives alongside the merged Rust code:

```
auto-lang/
├── crates/auto-forge/           # Rust implementation (Plan 251 merged)
│   └── src/                     # ~12K LOC Rust
│
├── auto/forge/                  # Auto implementation (this plan)
│   ├── pac.at                   # Package manifest
│   │
│   ├── provider/                # LLM provider layer
│   │   ├── mod.at
│   │   ├── types.at             # ← auto-coder/types.at (adapted)
│   │   ├── claude.at            # ← auto-coder/anthropic.at (adapted)
│   │   ├── openai.at            # ← auto-coder/openai.at (adapted)
│   │   └── sse.at               # ← auto-coder/sse.at (adapted)
│   │
│   ├── tools/                   # Tool system
│   │   ├── mod.at               # ← auto-coder/tools.at (adapted)
│   │   ├── bash.at              # ← auto-coder/tool_bash.at
│   │   ├── read.at              # ← auto-coder/tool_file_read.at
│   │   ├── write.at             # ← auto-coder/tool_file_write.at
│   │   ├── edit.at              # ← auto-coder/tool_file_edit.at
│   │   ├── grep.at              # ← auto-coder/tool_grep.at
│   │   └── specs.at             # NEW: spec management tools
│   │
│   ├── agent/                   # Agent runtime
│   │   ├── mod.at
│   │   ├── agent.at             # ← auto-coder/agent.at (adapted)
│   │   ├── context.at           # ← auto-coder/context.at
│   │   ├── permission.at        # ← auto-coder/permission.at
│   │   ├── session.at           # ← auto-coder/session.at
│   │   └── settings.at          # ← auto-coder/settings.at
│   │
│   ├── forge/                   # Forge spec system (NEW)
│   │   ├── mod.at               # Session, spec lifecycle, status machine
│   │   ├── specs.at             # Spec item CRUD, section config
│   │   └── status.at            # Status enum + transition rules
│   │
│   ├── relay/                   # Relay multi-agent (NEW)
│   │   ├── mod.at               # RelayRegistry, re-exports
│   │   ├── flow.at              # FlowSpec, FlowStep, ExitRouting
│   │   ├── flows.at             # Built-in flow factories
│   │   ├── handoff.at           # HandoffDocument + rendering
│   │   ├── budget.at            # Token budget tracking
│   │   ├── profession.at        # Profession + ProfessionRegistry
│   │   ├── soul.at              # SoulConfig loader
│   │   ├── agent_instance.at    # AgentInstance + ModelConfig
│   │   ├── turn.at              # AgentTurn execution
│   │   ├── pipeline.at          # PipelineEngine state machine
│   │   └── store.at             # Run storage (actor-based)
│   │
│   ├── cli/                     # CLI interface
│   │   ├── mod.at               # Entry point + clap-like arg parsing
│   │   └── repl.at              # ← auto-coder/repl.at (adapted)
│   │
│   └── server/                  # HTTP server (NEW)
│       ├── mod.at               # Axum-like routing setup
│       ├── forge_api.at         # Forge HTTP endpoints
│       └── relay_api.at         # Relay HTTP endpoints + SSE
│
└── test/auto-forge-snapshots/   # Progressive test snapshots
    ├── step-00-provider/        # Minimal LLM client
    ├── step-01-tools/           # + Tool system
    ├── step-02-agent/           # + Single-agent runtime
    ├── step-03-forge/           # + Spec-driven chat
    ├── step-04-relay-core/      # + Relay pipeline + professions
    ├── step-05-relay-full/      # + Store, API, checkpoint
    └── step-06-server/          # + HTTP server
```

**Total estimated**: ~7,500 LOC Auto (2,442 adapted + ~5,000 new)

---

## 2. Snapshot System

Each snapshot is a self-contained, runnable Auto program that builds on the previous one. This mirrors the auto-code-rs snapshot approach but extends it to cover the full auto-forge feature set.

### Snapshot overview

```
step-00  Provider (LLM client)           ~800 LOC    Validates: HTTP, JSON, streaming, types
step-01  + Tools                          ~600 LOC    Validates: file I/O, regex, tool dispatch
step-02  + Agent runtime                  ~500 LOC    Validates: ReAct loop, context, session
step-03  + Forge spec system              ~800 LOC    Validates: specs, status machine, CRUD
step-04  + Relay core (pipeline + profs)  ~1,200 LOC  Validates: state machine, handoff, budget
step-05  + Relay full (store + turn)      ~1,000 LOC  Validates: actor model, turn execution
step-06  + HTTP server                    ~600 LOC    Validates: routing, SSE, API endpoints
                                                    ─────────
                                            Total   ~5,500 LOC in snapshots
```

Each snapshot has:
- `main.at` — The program entry point
- `expected.out` — Expected console output (for VM test runner)
- Related `.at` library files

### Snapshot testing methodology

Each snapshot tests specific Auto language features against real-world code:

| Snapshot | Primary Auto features tested |
|----------|------------------------------|
| step-00 | HTTP client, JSON parse/stringify, enum with payloads, async streaming, Result/Option |
| step-01 | File I/O, regex, HashMap, trait-like dispatch, error propagation |
| step-02 | Pattern matching (`is`), loop/break, mutable state (`var`), generics on List |
| step-03 | String parsing, status state machine, type with many fields, CRUD patterns |
| step-04 | Complex enum matching, HashMap iteration, nested data structures |
| step-05 | Actor model (task/send/recv), shared state, message passing |
| step-06 | HTTP server, routing, SSE streaming, async request handling |

---

## 3. Phase 1 — Adapt Existing Auto-Coder Files

**Goal**: Restructure the existing 18 `.at` files to match the merged auto-forge directory layout, fix VM limitations, and make them production-ready.

**Estimated effort**: 3-4 days

### Step 1.1: Move files to new directory structure

Copy `auto-coder/src/*.at` into `auto/forge/` with new organization:

```
auto-coder/src/types.at       → auto/forge/provider/types.at
auto-coder/src/anthropic.at   → auto/forge/provider/claude.at
auto-coder/src/openai.at      → auto/forge/provider/openai.at
auto-coder/src/sse.at         → auto/forge/provider/sse.at
auto-coder/src/tools.at       → auto/forge/tools/mod.at
auto-coder/src/tool_bash.at   → auto/forge/tools/bash.at
auto-coder/src/tool_file_read.at  → auto/forge/tools/read.at
auto-coder/src/tool_file_write.at → auto/forge/tools/write.at
auto-coder/src/tool_file_edit.at  → auto/forge/tools/edit.at
auto-coder/src/tool_grep.at   → auto/forge/tools/grep.at
auto-coder/src/agent.at       → auto/forge/agent/agent.at
auto-coder/src/context.at     → auto/forge/agent/context.at
auto-coder/src/permission.at  → auto/forge/agent/permission.at
auto-coder/src/session.at     → auto/forge/agent/session.at
auto-coder/src/settings.at    → auto/forge/agent/settings.at
auto-coder/src/repl.at        → auto/forge/cli/repl.at
auto-coder/src/main.at        → auto/forge/cli/mod.at
auto-coder/src/mod.at         → auto/forge/pac.at (updated)
```

### Step 1.2: Fix the multi-file import problem

The existing `.at` files each duplicate type definitions with comments like "TODO: remove when multi-file a2r is supported." Fix this:

- **Option A** (preferred): Use Auto's `use` imports properly. Each module exports its types, consumers import them.
- **Option B**: If multi-file imports aren't stable yet, keep a single `types.at` at the `auto/forge/` level that all modules reference.

Test with: `auto run auto/forge/cli/mod.at` or via AutoVM directly.

### Step 1.3: Replace `use.rust` FFI with native Auto stdlib

The existing files use `use.rust serde_json::Value::from_str` and similar. Replace with native Auto equivalents:

| Current FFI call | Native Auto replacement |
|---|---|
| `use.rust serde_json::Value::from_str` | `json.parse(str)` |
| `use.rust std::fs::read_to_string` | `File.read_text(path)` |
| `use.rust std::fs::write` | `File.write_text(path, content)` |
| `use.rust std::path::Path::exists` | `File.exists(path)` |
| `use.rust Str::*` | Built-in string methods |

### Step 1.4: Fix HTTP implementation

Replace stubbed HTTP calls with native Auto HTTP:

```auto
// Before (stubbed):
// http_post(url, api_key, body) → Result<HttpResponse, str>

// After (native):
fn http_post(url str, api_key str, body str) Result<Response, ApiError] {
    let resp = http.post(url)
        .header("Authorization", f"Bearer $api_key")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
    is resp {
        Ok(r) -> Ok(r)
        Err(e) -> Err(ApiError.Http(e.to_string()))
    }
}
```

### Step 1.5: Fix Map iteration

The existing `tools.at` has "TODO: implement map iteration" for `ToolRegistry.definitions()` and `tool_names()`. Auto's HashMap supports iteration:

```auto
fn definitions(self) List[ToolDefinition] {
    let result = []
    for key in self.tools.keys() {
        let info = self.tools.get(key)
        is info {
            Some(i) -> result.push(ToolDefinition {
                name: i.name
                description: i.description
                input_schema: json.parse(i.input_schema)
            })
            None -> {}
        }
    }
    result
}
```

### Step 1.6: Create step-00 through step-02 snapshots

Build the first 3 progressive snapshots from the adapted files:

- **step-00-provider**: `provider/` files + minimal `main.at` that calls Claude API
- **step-01-tools**: + `tools/` files + `main.at` that runs a tool
- **step-02-agent**: + `agent/` files + `main.at` that runs a full agent turn

Each snapshot must pass: `auto run test/auto-forge-snapshots/step-XX/main.at`

### Verification

- [ ] All 18 `.at` files moved to new locations
- [ ] No `use.rust` FFI calls remain (replaced with native stdlib)
- [ ] No duplicate type definitions (proper imports)
- [ ] Map iteration works in ToolRegistry
- [ ] HTTP calls use native Auto HTTP client
- [ ] step-00, step-01, step-02 snapshots run successfully

---

## 4. Phase 2 — Forge Spec System

**Goal**: Write the Forge spec lifecycle system in Auto — specs, status state machine, section configs, spec CRUD tools.

**Estimated effort**: 3-4 days

### Step 2.1: Write status state machine

**File**: `auto/forge/forge/status.at`

Port the Rust `Status` enum (22 variants) and `SectionConfig` with allowed transitions:

```auto
// Status enum — 22 lifecycle states
tag Status {
    Empty
    Proposed
    Draft
    UnderReview
    Approved
    InProgress
    InImplementation
    Implemented
    Verified
    Done
    Archived
    Rejected
    Backlog
    Ready
    InReview
    Blocked
    Superseded
    Outdated
    Stable
    Deprecated
    Published
    Analysed
    Obsolete
}

// SectionType determines which statuses are valid
tag SectionType {
    Goals
    Architecture
    Designs
    Plans
    Tests
    Reviews
    Reports
    Apis
}

// Status transitions per section type
type SectionConfig {
    section_type SectionType
    allowed_statuses List[Status]
    allowed_transitions List[Transition]
}

type Transition {
    from Status
    to Status
}

fn can_transition(config SectionConfig, from Status, to Status) bool {
    for t in config.allowed_transitions {
        is (t.from, t.to) {
            (f, t2) -> if f == from && t2 == to { return true }
        }
    }
    false
}

// Derive goal status from downstream item statuses
fn derive_goal_status(items List[SpecItem]) Status {
    // If all items Done → Done
    // If any InProgress → InProgress
    // If all Approved → Approved
    // etc.
}
```

### Step 2.2: Write spec item and spec management

**File**: `auto/forge/forge/specs.at`

```auto
type SpecItem {
    id str
    title str
    content str
    status Status
    depends_on List[str]
    related List[str]
    priority Option[str]
    assignee Option[str]
    test_file Option[str]
    file Option[str]
    milestone Option[str]
    module Option[str]
    created_at u64
    modified_at u64
    completed_at Option[u64]
}

type SpecSection {
    section_type SectionType
    items List[SpecItem]
    manifest_path str
}
```

Implement spec CRUD: `load_section()`, `save_section()`, `add_item()`, `update_item()`, `find_item()`, `list_by_status()`.

Spec files use `.ad` format (AsciiDoc-like). Parse and render with string operations.

### Step 2.3: Write spec tools

**File**: `auto/forge/tools/specs.at`

Three tools for AI agents to interact with specs:

```auto
// ReadJadeTool — read a spec section with pending changes
type ReadJadeTool {}

impl ReadJadeTool {
    fn name() str { "read_jade" }
    fn description() str { "Read a spec section" }
    fn execute(input Value) Result[str, ToolError] {
        // Parse section_id from input
        // Load section from specs/
        // Apply pending changes overlay
        // Render as text
    }
}

// WriteJadeTool — queue a spec change for approval
type WriteJadeTool {}

// ListJadesTool — list all spec sections with statuses
type ListJadesTool {}
```

### Step 2.4: Write Forge session

**File**: `auto/forge/forge/mod.at`

```auto
tag ForgeStatus {
    Idle
    Thinking
    ToolCall
    WaitingApproval
    Error
}

type ForgeMessage {
    id str
    role str
    content str
    timestamp u64
    tool_calls Option[List[ToolCallInfo]]
}

type ForgeSession {
    id str
    project_path str
    status ForgeStatus
    messages List[ForgeMessage]
    pending_spec_changes List[SpecChange]
    focus_section Option[str]
}

impl ForgeSession {
    fn new(project_path str) ForgeSession
    fn send_message(self, user_text str) ForgeMessage
    fn run_turn(self, provider Provider, tools ToolRegistry) TurnResult
    fn approve_spec_change(self, change_id str) bool
    fn reject_spec_change(self, change_id str, reason str) bool
}
```

### Step 2.5: Create step-03 snapshot

**Snapshot**: `test/auto-forge-snapshots/step-03-forge/`

Test script that:
1. Creates a ForgeSession
2. Loads spec sections from a test `.ad` file
3. Creates spec items with various statuses
4. Tests status transitions (valid + invalid)
5. Tests spec derivation (goal status from items)
6. Tests spec tool CRUD operations

### Verification

- [ ] Status enum with 22 variants works
- [ ] `can_transition()` correctly validates state transitions
- [ ] `derive_goal_status()` computes from downstream items
- [ ] Spec files load/save in `.ad` format
- [ ] Spec tools (read/write/list) work
- [ ] ForgeSession maintains conversation state
- [ ] step-03 snapshot runs successfully

---

## 5. Phase 3 — Relay Core (Pipeline + Professions)

**Goal**: Write the Relay multi-agent orchestration core in Auto — the deterministic pipeline state machine, profession definitions, handoff documents, and budget tracking.

**Estimated effort**: 4-5 days

This is the hardest phase. The pipeline state machine is the heart of auto-forge's multi-agent system.

### Step 3.1: Write flow specification

**File**: `auto/forge/relay/flow.at`

```auto
tag GateType {
    Auto
    Human
}

tag ExitRouting {
    Next
    Branch { condition str, then_step str, else_step str }
    Loop { back_to str, max_iterations u32 }
}

type FlowStep {
    id str
    profession_id str
    description str
    gate GateType
    exit ExitRouting
}

type FlowSpec {
    id str
    name str
    steps List[FlowStep]
}
```

Methods: `FlowSpec.new()`, `add_step()`, `get_step()`, `get_step_index()`, `step_for_profession()`.

### Step 3.2: Write built-in flows

**File**: `auto/forge/relay/flows.at`

Port the 3 factory functions:

```auto
fn standard_spec_flow() FlowSpec {
    // Intake → Planner → Architect → Coder → Tester → Reviewer
    // Human gates after Planner and Architect
}

fn fast_track_flow() FlowSpec {
    // Intake → Coder
}

fn bug_fix_flow() FlowSpec {
    // Intake → Coder → Tester → Reviewer
    // Loop: Tester → Coder (max 3 iterations)
}
```

### Step 3.3: Write handoff document

**File**: `auto/forge/relay/handoff.at`

```auto
type Decision {
    description str
    rationale str
}

type Question {
    question str
    context str
    blocking bool
}

type WorkProduct {
    description str
    files_modified List[str]
    files_created List[str]
    tests_added List[str]
}

type ContextPointers {
    files_to_read List[str]
    specs_to_follow List[str]
    warnings List[str]
}

type HandoffDocument {
    from_profession str
    to_profession str
    run_id str
    step_id str
    summary str
    decisions List[Decision]
    questions List[Question]
    spec_updates List[SpecUpdate]
    work_products List[WorkProduct]
    context_pointers ContextPointers
    token_usage TokenUsage
}

impl HandoffDocument {
    fn new(from str, to str, run_id str) HandoffDocument
    fn render(self) str   // Render as markdown text
}
```

### Step 3.4: Write budget tracking

**File**: `auto/forge/relay/budget.at`

```auto
type TokenBudget {
    total_budget u64
    warning_threshold f64   // e.g. 0.7 = warn at 70%
    strategy BudgetStrategy
}

tag BudgetStrategy {
    HardStop
    EscalateModel
    SummarizeContext
    SkipOptional
}

tag BudgetAction {
    None
    Warning { remaining u64 }
    Compress
    HardStop
}

type CostReport {
    total_used u64
    by_step Map      // step_id → u64
    savings_vs_parallel u64
}

type BudgetTracker {
    budget TokenBudget
    used u64
    step_budgets Map  // step_id → u64
    step_used Map     // step_id → u64
}

impl BudgetTracker {
    fn new(budget TokenBudget) BudgetTracker
    fn record(self, step_id str, tokens u64)
    fn check(self) BudgetAction
    fn cost_report(self) CostReport
    fn savings_vs_parallel(self) u64
}
```

### Step 3.5: Write profession system

**File**: `auto/forge/relay/profession.at`

```auto
tag ForgePhase {
    Intake
    SpecDraft
    SpecReview
    Execution
    Verification
}

type Profession {
    id str
    name str
    phase ForgePhase
    owned_sections List[SectionType]
    readable_sections List[SectionType]
    allowed_tools List[str]
    handoff_to List[str]
    approval_gates List[str]
    max_turns u32
    token_budget u64
}

type ProfessionRegistry {
    professions Map   // id → Profession
}

fn register_builtin_professions(registry ProfessionRegistry) {
    // intaker: no tools, routes to planner
    // planner: jade tools, owns Goals/Plans
    // architect: jade tools, owns Architecture/Designs/APIs
    // coder: file tools + shell + search
    // tester: file tools + jade tools, owns Tests
    // reviewer: read-only tools, owns Reviews/Reports
    // documenter: read-only tools, owns Reports
}
```

### Step 3.6: Write soul configuration

**File**: `auto/forge/relay/soul.at`

```auto
type SoulConfig {
    id str
    name str
    personality str
    values List[str]
    working_style str
    handoff_ritual str
    system_prompt_template str
}

impl SoulConfig {
    fn load(path str) Result[SoulConfig, str]
    fn parse(markdown str) Result[SoulConfig, str]
    fn render(self) str   // Render full system prompt
}
```

### Step 3.7: Write agent instance

**File**: `auto/forge/relay/agent_instance.at`

```auto
tag ProviderKind {
    Anthropic
    OpenAI
    Local { url str }
}

type ModelConfig {
    provider ProviderKind
    model str
    temperature f32
    max_tokens u32
    fallback_chain List[str]
}

type AgentContext {
    budget_used u64
    turns_taken u32
    files_touched List[str]
    decisions List[str]
    open_questions List[str]
}

type AgentInstance {
    id str
    profession Profession
    soul SoulConfig
    model ModelConfig
    context AgentContext
}

impl ModelConfig {
    fn cheap() ModelConfig     // haiku
    fn standard() ModelConfig  // sonnet
    fn strong() ModelConfig    // opus
}

impl AgentInstance {
    fn spawn(profession Profession, soul SoulConfig, model ModelConfig) AgentInstance
    fn render_system_prompt(self) str
    fn build_chat_request(self, tools List[ToolDef], handoff str, spec str) LlmRequest
}
```

### Step 3.8: Write the pipeline engine

**File**: `auto/forge/relay/pipeline.at`

This is the most complex piece — the deterministic state machine:

```auto
tag AdvanceResult {
    ExecuteStep { step_id str, profession_id str }
    WaitForHuman { gate GateType, step_id str }
    Completed
    Failed { error str }
}

tag GateDecision {
    Approve
    Reject { feedback str }
    Edit { changes str }
}

tag PipelineStatus {
    Idle
    Running { step_id str, profession_id str, started_at u64 }
    WaitingForHuman { gate GateType, step_id str, since u64 }
    Completed
    Failed { error str }
    Paused { at_step u32 }
}

type PipelineEngine {
    flow FlowSpec
    current_step u32
    status PipelineStatus
    run_id str
    step_history List[StepRecord]
    loop_counters Map       // step_id → iteration count
    pending_gate Option[PendingGate]
    gate_feedback Map       // step_id → List[str]
    cumulative_tokens u64
    budget_tracker BudgetTracker
}

impl PipelineEngine {
    fn new(flow FlowSpec, run_id str) PipelineEngine
    fn with_budget(flow FlowSpec, run_id str, budget TokenBudget) PipelineEngine

    // Core state machine transitions
    fn advance(self) AdvanceResult
    fn submit_handoff(self, handoff HandoffDocument) AdvanceResult
    fn resolve_gate(self, decision GateDecision) AdvanceResult
    fn pause(self)
    fn resume(self) AdvanceResult
}
```

The `advance()` method implements the core loop:

```auto
fn advance(self) AdvanceResult {
    is self.status {
        PipelineStatus.Idle -> {
            // Start first step
            let step = self.flow.steps[0]
            self.status = PipelineStatus.Running { ... }
            return AdvanceResult.ExecuteStep { ... }
        }
        PipelineStatus.Running { step_id, .. } -> {
            // Check if current step completed (handoff submitted)
            // Resolve exit routing
            // Advance to next step or loop back
        }
        PipelineStatus.WaitingForHuman { .. } -> {
            // Can't advance until gate resolved
            return AdvanceResult.WaitForHuman { ... }
        }
        PipelineStatus.Completed -> AdvanceResult.Completed
        PipelineStatus.Failed { error } -> AdvanceResult.Failed { error }
        PipelineStatus.Paused { .. } -> AdvanceResult.Failed { error: "paused" }
    }
}
```

### Step 3.9: Create step-04 snapshot

**Snapshot**: `test/auto-forge-snapshots/step-04-relay-core/`

Test script that:
1. Creates `standard_spec_flow()` — 6 steps
2. Creates a `PipelineEngine`
3. Advances through steps, checking `AdvanceResult` at each stage
4. Submits handoffs between steps
5. Tests loop routing (bug_fix_flow loops back from Tester to Coder)
6. Tests human gate resolution (approve + reject)
7. Tests budget enforcement (hard stop at threshold)

### Verification

- [ ] FlowSpec with 3 built-in flows works
- [ ] HandoffDocument renders correctly
- [ ] BudgetTracker warns at 70% and hard-stops at 100%
- [ ] 7 professions registered with correct tool/section permissions
- [ ] SoulConfig loads from markdown files
- [ ] AgentInstance renders system prompts
- [ ] PipelineEngine advances through all 6 steps of standard flow
- [ ] Loop routing works (bug_fix_flow)
- [ ] Gate approve/reject works
- [ ] Budget enforcement stops pipeline
- [ ] step-04 snapshot runs successfully

---

## 6. Phase 4 — Relay Turn Execution + Run Store

**Goal**: Write the agent turn executor (the ReAct loop for Relay) and the actor-based run storage.

**Estimated effort**: 3-4 days

### Step 4.1: Write agent turn execution

**File**: `auto/forge/relay/turn.at`

This is the Relay version of the ReAct loop. It differs from the simple agent loop because:
- It uses the profession's tool whitelist (not all tools)
- It tracks budget usage
- It produces a `TurnResult` with structured handoff data

```auto
tag TurnEvent {
    TextDelta { text str }
    ToolCall { id str, name str, arguments Value }
    ToolResult { id str, result str }
    Complete
    Error { message str }
    BudgetWarning { remaining u64 }
    BudgetExceeded
}

type ToolCallRecord {
    id str
    name str
    arguments Value
    result str
}

type TurnResult {
    assistant_text str
    tool_calls List[ToolCallRecord]
    tokens_used u64
    handoff_requested bool
    decisions List[str]
    open_questions List[str]
    files_touched List[str]
}

type AgentTurn {
    agent AgentInstance
    tool_definitions List[ToolDef]
    tool_registry ToolRegistry
    messages List[Message]
    max_turns u32
    budget_tracker Option[BudgetTracker]
}

impl AgentTurn {
    fn new(agent AgentInstance, registry ToolRegistry, messages List[Message]) AgentTurn

    fn run(self, provider Provider) TurnResult {
        // Filter tools by profession's allowed_tools
        // Call provider.chat_stream()
        // Collect text + tool_use
        // Execute tools (filtered by profession)
        // Check budget after each iteration
        // Return TurnResult with structured data
    }

    fn to_handoff(self, result TurnResult, to_profession str, run_id str) HandoffDocument {
        // Convert TurnResult into HandoffDocument
        // Extract decisions, questions, files_touched
    }
}
```

### Step 4.2: Write run store (actor-based)

**File**: `auto/forge/relay/store.at`

The Rust version uses `Arc<Mutex<HashMap>>` for thread-safe storage. In Auto, use the actor model:

```auto
// Run state tracking
tag RunState {
    Idle
    Running { step StepState }
    WaitingForHuman { gate GateState }
    Completed { summary str }
    Failed { error str }
}

type StepState {
    step_id str
    profession_id str
    started_at u64
    tokens_used u64
}

type GateState {
    step_id str
    gate_type GateType
    since u64
}

type RunEntry {
    id str
    flow FlowSpec
    engine PipelineEngine
    state RunState
    events List[RunEvent]
    started_at u64
}

tag RunEvent {
    StepStarted { step_id str, profession str }
    HandoffReceived { from str, to str }
    GateWaiting { step_id str }
    GateResolved { decision GateDecision }
    StepCompleted { step_id str, tokens u64 }
    RunCompleted { total_tokens u64 }
    Error { message str }
}

// Actor-based store
type RunStore {
    runs Map     // run_id → RunEntry
    next_id u64
}

impl RunStore {
    fn new() RunStore
    fn start_run(self, flow FlowSpec) str      // returns run_id
    fn get_run(self, run_id str) Option[RunEntry]
    fn list_runs(self) List[RunSummary]
    fn advance_run(self, run_id str) AdvanceResult
    fn submit_handoff(self, run_id str, handoff HandoffDocument) AdvanceResult
    fn resolve_gate(self, run_id str, decision GateDecision) AdvanceResult
}
```

**Auto actor pattern** (replaces Arc<Mutex<HashMap>>):

```auto
// Instead of Arc<Mutex<HashMap>>, use an Auto task actor
task run_store_actor {
    var store = RunStore.new()

    for msg in inbox {
        is msg {
            StartRun { flow, reply } -> {
                let id = store.start_run(flow)
                reply.send(id)
            }
            GetRun { id, reply } -> {
                reply.send(store.get_run(id))
            }
            AdvanceRun { id, reply } -> {
                let result = store.advance_run(id)
                reply.send(result)
            }
        }
    }
}
```

### Step 4.3: Write RelayRegistry

**File**: `auto/forge/relay/mod.at`

```auto
type RelayRegistry {
    professions ProfessionRegistry
    souls Map              // id → SoulConfig
    souls_dir str
}

impl RelayRegistry {
    fn new() RelayRegistry
    fn get_soul(self, id str) Option[SoulConfig]
    fn get_profession(self, id str) Option[Profession]
    fn spawn_agent(self, profession_id str, soul_id str, model ModelConfig) Option[AgentInstance]
}
```

### Step 4.4: Create step-05 snapshot

**Snapshot**: `test/auto-forge-snapshots/step-05-relay-full/`

Test script that:
1. Creates a RelayRegistry with all 7 professions
2. Starts a run with `standard_spec_flow()`
3. Advances through each step, spawning agents
4. Runs agent turns (using mock provider for deterministic testing)
5. Submits handoffs between steps
6. Tests gate resolution
7. Tests budget enforcement
8. Lists runs and gets run state

### Verification

- [ ] AgentTurn filters tools by profession
- [ ] TurnResult includes decisions, questions, files_touched
- [ ] HandoffDocument generated from TurnResult
- [ ] RunStore manages run lifecycle
- [ ] Actor-based store replaces Arc<Mutex<HashMap>>
- [ ] Full standard_spec_flow runs end-to-end with mock provider
- [ ] step-05 snapshot runs successfully

---

## 7. Phase 5 — HTTP Server + API

**Goal**: Write the HTTP server that exposes Forge and Relay via API endpoints.

**Estimated effort**: 3-4 days

### Step 5.1: Write Forge API endpoints

**File**: `auto/forge/server/forge_api.at`

Port the Forge HTTP handlers:

```auto
// POST /forge/sessions — create session
// GET /forge/sessions/:id — get session
// POST /forge/sessions/:id/chat — send message (SSE response)
// GET /forge/specs — list spec sections
// GET /forge/specs/:section — read section
// POST /forge/specs/:section/items — add item
// PUT /forge/specs/:section/items/:id — update item
// POST /forge/specs/:section/items/:id/approve — approve change
```

### Step 5.2: Write Relay API endpoints

**File**: `auto/forge/server/relay_api.at`

Port the Relay HTTP handlers:

```auto
// GET /relay/professions — list professions
// GET /relay/souls — list souls
// GET /relay/runs — list runs
// POST /relay/runs — start run
// GET /relay/runs/:id — get run state
// POST /relay/runs/:id/advance — advance pipeline
// POST /relay/runs/:id/handoff — submit handoff
// POST /relay/runs/:id/gate — resolve gate
// GET /relay/runs/:id/events — SSE stream
```

### Step 5.3: Write server entry point

**File**: `auto/forge/server/mod.at`

```auto
fn start_server(port int) {
    let app = server.new()
        .route("/forge/sessions", post(create_session))
        .route("/forge/sessions/:id/chat", post(chat_handler))
        .route("/relay/runs", post(start_run_handler))
        .route("/relay/runs/:id/events", get(run_events_handler))
        // ... more routes

    app.listen(f"127.0.0.1:$port")
}
```

### Step 5.4: Update CLI entry point

**File**: `auto/forge/cli/mod.at`

Add server mode to the CLI:

```auto
fn main() {
    let args = parse_args()
    is args.command {
        Some("serve") -> start_server(args.port)
        Some("chat") -> run_cli_chat(args)
        None -> start_server(3031)   // default: web server
    }
}
```

### Step 5.5: Create step-06 snapshot

**Snapshot**: `test/auto-forge-snapshots/step-06-server/`

Test script that:
1. Starts the HTTP server
2. Creates a Forge session via API
3. Sends a chat message (SSE response)
4. Starts a Relay run via API
5. Advances the pipeline via API
6. Reads run events via SSE
7. Shuts down

### Verification

- [ ] HTTP server starts on specified port
- [ ] Forge session CRUD works via API
- [ ] Chat endpoint streams SSE events
- [ ] Relay run lifecycle works via API
- [ ] SSE event stream delivers real-time updates
- [ ] CLI supports both `serve` and `chat` modes
- [ ] step-06 snapshot runs successfully

---

## 8. Phase 6 — Checkpoint & Durability

**Goal**: Write the checkpoint system for pipeline state persistence and resume.

**Estimated effort**: 2-3 days

### Step 6.1: Write file state tracking

**File**: `auto/forge/relay/checkpoint.at`

```auto
type FileState {
    path str
    hash str
    content Option[str]
}

type Checkpoint {
    run_id str
    step_index u32
    pipeline_status PipelineStatus
    handoff Option[HandoffDocument]
    file_states List[FileState]
    created_at u64
}

impl Checkpoint {
    fn create(engine PipelineEngine, handoff Option[HandoffDocument]) Checkpoint
    fn save(self, dir str) Result[str, str]    // JSON serialize to disk
    fn load(path str) Result[Checkpoint, str]   // JSON deserialize
    fn restore_files(self) Result[u32, str]     // Restore modified files
    fn integrity_hash(self) str                  // SHA-256 via FFI
}
```

### Step 6.2: Integrate with PipelineEngine

Add checkpoint methods to pipeline:

```auto
impl PipelineEngine {
    fn save_checkpoint(self, dir str) Result[str, str]
    fn from_checkpoint(path str) Result[PipelineEngine, str]
}
```

### Verification

- [ ] Checkpoint captures pipeline state correctly
- [ ] Checkpoint save/load round-trips without data loss
- [ ] File state restoration works
- [ ] Pipeline can resume from checkpoint after simulated failure

---

## 9. Language Features Needed

### Already available (no work needed)
- `tag` (sum types with payloads) — for all the enums
- `type` (record types) — for all the structs
- `is` (pattern matching) — for all the match arms
- `List`, `Map`, `Option`, `Result` — core data structures
- HTTP client/server — for API communication
- JSON parse/stringify — for serialization
- File I/O — for specs and sessions
- `var` (mutable bindings) — for state mutations
- `for` loops, `loop`/`break` — for iteration

### Needs verification
- **HashMap `.keys()` iteration** — needed for ToolRegistry, BudgetTracker
- **Nested type references** — `List[SpecItem]`, `Map[str, Profession]`
- **Pattern matching on multiple variants** — `is status { Status.Running { step_id } -> ... }`
- **Actor model** — `task` blocks with inbox/message passing

### Needs FFI bridge
- **SHA-256 hashing** — for checkpoint integrity (add `use.rust sha2` bridge or native Auto impl)

---

## 10. File Count and LOC Estimates

| Component | Files | Estimated LOC | Source |
|-----------|-------|---------------|--------|
| Provider layer | 5 | ~800 | Adapted from auto-coder |
| Tool system | 7 | ~1,100 | Adapted + new specs.at |
| Agent runtime | 6 | ~700 | Adapted from auto-coder |
| Forge spec system | 3 | ~800 | New |
| Relay flow + handoff | 3 | ~500 | New |
| Relay budget | 1 | ~250 | New |
| Relay profession + soul + agent | 3 | ~600 | New |
| Relay pipeline | 1 | ~400 | New |
| Relay turn + store | 2 | ~700 | New |
| HTTP server | 3 | ~600 | New |
| CLI | 2 | ~250 | Adapted + new |
| Checkpoint | 1 | ~300 | New |
| Package + module files | 5 | ~50 | New |
| **Total** | **42** | **~7,050** | **2,442 adapted + ~4,600 new** |

### Snapshot LOC

| Snapshot | LOC |
|----------|-----|
| step-00-provider | ~800 |
| step-01-tools | ~600 |
| step-02-agent | ~500 |
| step-03-forge | ~800 |
| step-04-relay-core | ~1,200 |
| step-05-relay-full | ~1,000 |
| step-06-server | ~600 |
| **Total** | **~5,500** |

---

## 11. Execution Timeline

```
Week 1-2:  Phase 1 — Adapt existing auto-coder files (3-4 days)
Week 2-3:  Phase 2 — Forge spec system (3-4 days)
Week 3-5:  Phase 3 — Relay core: pipeline + professions (4-5 days) ← HARDEST
Week 5-6:  Phase 4 — Relay turn execution + run store (3-4 days)
Week 6-7:  Phase 5 — HTTP server + API (3-4 days)
Week 7-8:  Phase 6 — Checkpoint + durability (2-3 days)
```

**Total estimated**: ~20-24 working days (~5-6 weeks)

### Dependency chain

```
Phase 1 (adapt files)
    ↓
Phase 2 (forge specs)     Phase 3 (relay core)  ← these can run in parallel
    ↓                           ↓
    └───────────┬───────────────┘
                ↓
         Phase 4 (relay turn + store)
                ↓
         Phase 5 (HTTP server)
                ↓
         Phase 6 (checkpoint)
```

Phases 2 and 3 can be worked on in parallel since they're independent (Forge spec system vs Relay pipeline). This could shave ~1 week off the timeline.

---

## 12. Testing Strategy

### Per-snapshot testing

Each snapshot has an `expected.out` file. The test runner:

```bash
auto run test/auto-forge-snapshots/step-XX/main.at > actual.out
diff expected.out actual.out
```

### Integration testing

After all phases complete, an end-to-end test:

```auto
// test/auto-forge-e2e/main.at
fn main() {
    // 1. Start relay with standard_spec_flow
    // 2. Advance through all 6 steps with mock LLM
    // 3. Verify handoffs between agents
    // 4. Verify spec status transitions
    // 5. Verify budget enforcement
    // 6. Print summary
    print("PASS: auto-forge self-hosting e2e")
}
```

### a2r round-trip testing

Each `.at` file should also be testable via a2r (Auto → Rust transpiler):

```bash
cargo test -p auto-lang -- trans  # existing transpiler tests
```

This validates that the Auto code can also compile back to Rust, completing the self-hosting loop.

---

## 13. Relationship to Plan 251

Plan 251 (merge auto-code-rs into auto-forge) creates the merged Rust codebase. This plan (252) creates the Auto language version. They're sequential:

```
Plan 251 (merge Rust) → Plan 252 (write Auto version)
```

But Phase 1 of this plan (adapt existing auto-coder files) can start immediately, even before Plan 251 is complete, since the existing `.at` files don't depend on the Rust reorganization.

The final state:
- `crates/auto-forge/src/` — ~12K LOC Rust (reference implementation)
- `auto/forge/` — ~7K LOC Auto (self-hosted version)
- `test/auto-forge-snapshots/` — 7 progressive snapshots (~5.5K LOC)
- Both implementations are feature-equivalent

---

## 14. Future Work (Out of Scope)

1. **AutoLive hot-reload**: Use Auto's planned hot-reload to iterate on agent personalities without restarting
2. **Self-hosting**: Eventually the Auto version runs the Auto compiler itself — the ultimate dogfooding
3. **Performance benchmarking**: Compare Auto VM performance vs Rust for agent workloads
4. **Comptime specialization**: Use Auto's comptime features to generate profession-specific code at compile time
5. **Polyglot FFI plugins**: Allow tools to be implemented in Rust/Python/JS and loaded at runtime
