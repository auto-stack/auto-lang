# AutoForge — Spec-Driven Serial Agent UI

## Status: Design & Exploration

---

## 1. Naming: From AutoForge to AutoForge

After reviewing the competitive landscape and the existing Auto ecosystem, here are the top candidates:

| Name | Rationale | Fit |
|------|-----------|-----|
| **AutoForge** ⭐ | A smith works *serially* — heating, hammering, quenching, polishing — each step deliberate and dependent on the last. Evokes craftsmanship, precision, and finishing. No competing product uses this. | **Strong** |
| **AutoForge** | Where things are forged. Good but more generic; Forge is common in dev tools. | Good |
| **AutoRelay** | Emphasizes the serial handoff between agents. But sounds too sports-oriented. | Moderate |
| **AutoMason** | Building brick by brick — very serial! But "mason" is heavily associated with package managers. | Moderate |
| **AutoArchitect** | Focuses on planning/specs, but doesn't evoke execution/finishing. | Weak |
| **AutoForge** | Generic; sounds like Copilot/Codex clone. Doesn't differentiate. | Weak |

**Recommendation: AutoForge** (or **AutoForge Studio** for the UI, **AutoForge Engine** for the backend).

It communicates:
- **Serial craftsmanship**: A smith doesn't parallel-forge. They work one piece at a time, passing through stages.
- **Knowledge/skill-driven**: Smithing requires deep knowledge of materials, techniques, and patterns.
- **Gets things done**: A smith produces finished artifacts, not vibes.
- **Fits the Auto ecosystem**: AutoLang → AutoDown → AutoLab → **AutoForge**.

---

## 2. Vision: What Makes AutoForge Different

After analyzing 7+ major AI coding agents (Cursor, Claude Code, Copilot, Aider, Devin, OpenClaw, Intent, Kimi), every single one falls into one of three UI paradigms: IDE-embedded, CLI-native, or chat/cloud. **None of them treat project knowledge as a first-class, living entity.**

### The Gap in the Market

| Capability | Cursor | Claude Code | Copilot | Aider | Devin | Intent | **AutoForge** |
|------------|--------|-------------|---------|-------|-------|--------|---------------|
| Chat/loop | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| IDE integration | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ (web-first) |
| Spec-driven | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (static) | ✅ **Living** |
| Knowledge graph | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (Context Engine) | ✅ **AutoDown-native** |
| Serial agents | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Agent monitoring | ❌ | ❌ | ❌ | ❌ | ✅ (Slack) | ❌ | ✅ |
| Durable execution | ❌ (tmux) | ❌ (tmux) | ✅ (cloud) | ❌ (git) | ✅ | ❌ | ✅ **Checkpoint/restore** |
| Cost-aware orchestration | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

### Core Philosophy

> **"A blacksmith doesn't assign 10 apprentices to hammer the same sword. They heat it, shape it, temper it, and grind it — each stage handled by the right craftsperson with the right tool, in the right order."**

**AutoForge's secret weapon is serial agent cooperation over time.**

Instead of mass-parallel multi-agent systems (which burn 20–50× more tokens due to multiplicative context overhead), AutoForge uses a **pipeline of specialized roles that hand off work sequentially**. Each agent sees only the context it needs, not the entire conversation history.

This isn't just cheaper — it's **more reliable**. When an agent "completes" a task, the next agent in the chain validates it before proceeding. Failures are caught early, not propagated across 10 parallel threads.

---

## 3. Three-View Architecture

AutoForge UI presents three primary views, toggled via a top navigation rail. Each view serves a distinct purpose in the development lifecycle.

```
┌─────────────────────────────────────────────────────────────┐
│  AutoForge Studio                    [Forge] [Ledger] [Relay] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                  THE FORGE (View 1)                  │   │
│  │  ┌──────────────┐  ┌──────────────────────────────┐ │   │
│  │  │ Agent Status │  │        Chat Canvas            │ │   │
│  │  │  ● Planner   │  │  ┌────────────────────────┐  │ │   │
│  │  │  ○ Architect │  │  │ User: Add auth          │  │ │   │
│  │  │  ○ Coder     │  │  │                         │  │ │   │
│  │  │  ○ Tester    │  │  │ 🤖 Planner: I'll break  │  │ │   │
│  │  │  ○ Reviewer  │  │  │    this down into...     │  │ │   │
│  │  └──────────────┘  │  │    [View Spec →]         │  │ │   │
│  │                    │  │    [Approve & Continue]  │  │ │   │
│  │  ┌──────────────┐  │  └────────────────────────┘  │ │   │
│  │  │ Live File    │  │                              │ │   │
│  │  │ Changes      │  │  [Input: "Use JWT, not     ] │ │   │
│  │  │  + auth.rs   │  │       sessions"              │ │   │
│  │  │  ~ main.rs   │  │                              │ │   │
│  │  └──────────────┘  └──────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                  THE LEDGER (View 2)                 │   │
│  │  ┌────────────────────────────────────────────────┐ │   │
│  │  │  📋 Goals                                      │ │   │
│  │  │     └─ [LIVE] Implement user authentication   │ │   │
│  │  │  📐 Requirements                               │ │   │
│  │  │     └─ [DRIFT] Must support OAuth2 providers   │ │   │
│  │  │  🔍 Analysis                                   │ │   │
│  │  │     └─ [LIVE] Current auth is session-based    │ │   │
│  │  │  📅 Plans                                      │ │   │
│  │  │     └─ [PLAN] Phase 1: JWT token flow         │ │   │
│  │  │  ✅ Todos                                      │ │   │
│  │  │     └─ [DONE] Research JWT libraries          │ │   │
│  │  │  📊 Reports                                    │ │   │
│  │  │     └─ [NEW] Coverage: auth.rs 78% → 94%     │ │   │
│  │  │  📝 Reviews                                    │ │   │
│  │  │     └─ [PENDING] Security review required     │ │   │
│  │  └────────────────────────────────────────────────┘ │   │
│  │  [Sync with Code]  [Export .ad]  [AI Enrich]       │ │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                  THE RELAY (View 3)                  │   │
│  │                                                     │   │
│  │  Run #42 ──[Planner]──[Architect]──[Coder]──[Tester] │   │
│  │                ✅        ✅         🔄      ○        │   │
│  │              12k tk    28k tk     45k tk   --         │   │
│  │              3.2s      8.1s       14.5s    --         │   │
│  │                                                     │   │
│  │  Active Runs (3)   Completed (47)   Failed (2)       │   │
│  │                                                     │   │
│  │  ┌─────────────────────────────────────────────┐    │   │
│  │  │ Run #42  ● Active  Token budget: 150k/200k  │    │   │
│  │  │          JWT Auth Implementation            │    │   │
│  │  │  [Pause] [Rollback] [View Checkpoint]       │    │   │
│  │  └─────────────────────────────────────────────┘    │   │
│  │                                                     │   │
│  │  Cost this session: $1.24  (saved $8.30 vs parallel) │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### View 1: The Furnace · 熔炉 — Chat & AI Loop

**Purpose**: The primary interaction surface. Like Claude Code + Cursor Composer, but web-native.

**Components**:
- **Agent Status Panel** (left sidebar, collapsible): Shows the current active agent role in the serial pipeline (Planner → Architect → Coder → Tester → Reviewer). Visualizes which role is "holding the baton."
- **Chat Canvas** (center): Persistent conversation stream. Not just text — includes:
  - **Tool call cards**: File reads, edits, shell commands with expandable diffs
  - **Spec reference cards**: Inline links to Ledger specs ("See Requirements: OAuth2")
  - **Approval gates**: Human must approve spec changes or major architectural decisions before the agent proceeds
  - **Checkpoint markers**: Points where the agent saved state and can rollback
- **Live File Changes Panel** (right sidebar): Real-time diff view of files being modified. Like Cursor's Composer diff view.
- **Input Bar** (bottom): Persistent input with context chips ("Include: auth.rs, main.rs, Requirements.md"). Shift+Enter for multi-line.

**Key difference from existing tools**: The chat is *aware of the Ledger*. When an agent proposes a spec change, it's rendered as a "Spec Delta Card" that the user can approve, modify, or reject — before any code is written. This shifts the human-in-the-loop from *code review* to *spec review*, dramatically reducing rework.

### View 2: The Jade Tabs · 玉简 — Living Knowledge

**Purpose**: The single source of truth for project knowledge. AutoDown-native, bidirectional, versioned.

**Structure**: An AutoDown document (`.ad`) with well-known sections, stored in `.autoforge/` directory:

```autodown
# AutoForge Ledger: auto-playground

/// section:goals id:g1
## Goals

- $Goal(title: "Stateful VM Execution", status: "in_progress", priority: "P0")
- $Goal(title: "AI Streaming", status: "done", priority: "P0")

/// section:requirements id:r1 depends_on:g1
## Requirements

### R1.1 Session Lifecycle
$Requirement(id: "R1.1", status: "verified", source: "spec") {
    Sessions must support create → active → idle → closed states.
    Max idle time: 5 minutes.
}

/// section:analysis id:a1 depends_on:r1
## Analysis

$Analysis(topic: "VM State Persistence") {
    The AutovmReplSession holds VM state across cell executions.
    Stack overflow risk on deep expressions in test mode.
    → Mitigation: limit recursion depth in tests.
}

/// section:plans id:p1 depends_on:a1
## Plans

$Plan(id: "P5", status: "active", assignee: "agent") {
    Phase 5: Quality + AI Experience
    - 5.1: Test coverage
    - 5.2: AI streaming (SSE)
    - 5.3: One-click code extraction
}

/// section:todos id:t1 depends_on:p1
## Todos

- [x] Add backend unit tests (11 passing)
- [x] Add frontend Vitest (14 passing)
- [ ] Add e2e tests for AI streaming

/// section:reports id:rep1
## Reports

$Report(type: "coverage", date: "2026-05-11") {
    notebook/mod.rs: 87%
    notebook/ai.rs: 62%
    routes/notebook.rs: 45%
}

/// section:reviews id:rev1
## Reviews

$Review(id: "REV-1", status: "pending", reviewer: "human") {
    The dirty-cell re-execution queue may skip edge cases
    where upstream cells have side effects.
}
```

**Key features**:
- **Status badges**: Each section/entry shows `draft` → `approved` → `in_progress` → `verified` → `archived`
- **Drift detection**: The system periodically checks if code has diverged from specs. A spec marked `[LIVE]` is auto-updated when code changes. A spec marked `[DRIFT]` means code has changed but the spec hasn't been updated.
- **Bidirectional links**: Clicking a spec reference in the Forge jumps to the Ledger. Code comments referencing spec IDs (e.g., `// spec:R1.1`) are hyperlinked.
- **AI enrichment**: "AI, analyze the current codebase and update the Analysis section." The AI reads code, updates the Ledger, and presents a diff for human approval.
- **AutoDown rendering**: Uses the existing Typst/HTML transpilers. The Jade Tabs · 玉简 can be exported as a PDF report or published as HTML documentation.

### View 3: The Array · 法阵 — Agent Monitoring & Administration

**Purpose**: Observe, control, and audit long-running serial agent pipelines.

**Components**:
- **Pipeline Visualization**: A horizontal flow diagram showing the serial agent chain. Each node shows the role, token usage, time elapsed, and status (pending → active → completed → failed → rolled-back).
- **Run History**: Filterable list of all agent runs with cost, duration, outcome, and links to the Ledger state at that point in time.
- **Active Runs Dashboard**: Real-time view of running pipelines. Shows token budget consumption, estimated completion time, and current agent "thoughts."
- **Checkpoint Manager**: Each agent handoff is a checkpoint. User can:
  - View the full state at any checkpoint (files, Ledger, VM state)
  - Rollback to a previous checkpoint (reverts files + Ledger + VM)
  - Fork from a checkpoint (start a new run with different parameters)
- **Cost Analytics**: Per-run and per-role token usage. Compares actual cost vs. "what parallel execution would have cost" to show savings.
- **Agent Registry**: Administer available agent roles (Planner, Architect, Coder, Tester, Reviewer, Documenter, etc.). Configure which model each role uses, token budgets, and timeout policies.

**Key insight**: The Array · 法阵 treats agent runs like CI/CD pipelines. You wouldn't deploy code without seeing the pipeline status. You shouldn't let an AI agent run for hours without visibility into what it's doing.

---

## 4. The Serial Agent Model

### Why Serial?

Research shows peer-to-peer multi-agent systems have **multiplicative token overhead**: N agents with K messages cost O(N × K) tokens. For 6 agents with 8K context, that's 48K tokens loaded before any output. By round 3, coordination alone exceeds a single agent's needs.

AutoForge's serial model:

```
┌─────────┐    ┌──────────┐    ┌────────┐    ┌────────┐    ┌─────────┐
│ Planner │───→│ Architect│───→│ Coder  │───→│ Tester │───→│ Reviewer│
└─────────┘    └──────────┘    └────────┘    └────────┘    └─────────┘
   5k tk          15k tk         40k tk        20k tk         10k tk
   (cheap         (reasoning    (generation   (execution    (critique
    model)         model)        model)        + test)       model)
```

**Total: ~90k tokens** vs. **~450k+ for parallel multi-agent** (5× savings).

### The Handoff Protocol

Each agent in the chain receives:
1. **Scoped context**: Only the specs and files relevant to their role
2. **Previous agent's output**: A structured handoff document (not raw chat history)
3. **Token budget**: Hard limit. If exceeded, the agent must checkpoint and request continuation.
4. **Approval gate**: For spec changes or architectural decisions, the agent pauses and waits for human approval.

The handoff document is AutoDown-formatted:

```autodown
/// handoff:from=planner to=architect run=42 checkpoint=3

# Handoff: Authentication Feature

## Summary
Planner has decomposed "Add JWT auth" into 5 sub-tasks.

## Decisions Made
- $Decision(id: "D1", status: "approved") {
    Use JWT instead of session cookies.
    Rationale: Stateless, scales horizontally.
}

## Open Questions
- $Question(id: "Q1", status: "pending") {
    Should refresh tokens be rotated?
    Needs: Security review.
}

## Spec Updates
- Modified: Requirements → R2.3 (JWT flow)
- Added: Plans → P5.2 (Token rotation)

## Context for Next Agent
```
// Files to read:
- crates/auto-playground/src/auth/mod.rs (doesn't exist yet)
- crates/auto-playground/src/routes/mod.rs

// Specs to follow:
- Requirements:R2.3
- Plans:P5.1, P5.2
```
```

### Configurable Roles

Users can define custom roles in `.autoforge/roles/`:

```yaml
# .autoforge/roles/security-reviewer.yaml
name: Security Reviewer
model: claude-3-5-sonnet
max_tokens: 8000
temperature: 0.1
system_prompt: |
  You are a security-focused code reviewer. Your job is to
  review code changes for security vulnerabilities. Focus on:
  - Injection attacks
  - Authentication bypasses
  - Insecure deserialization
  - Secret leakage
inputs:
  - type: diff
  - type: spec
    filter: [requirements, analysis]
approval_required: true
```

### Durable Execution

Each checkpoint serializes:
- Git state (diff or commit)
- Ledger state (the `.ad` file at this point)
- VM state (for AutoLang code)
- Agent context window (summarized, not raw)

On crash or timeout:
1. AutoForge detects the failure
2. Offers to resume from last checkpoint or rollback
3. On resume: reloads state, rehydrates the next agent with summarized context
4. On rollback: reverts files, restores Ledger, resets VM

---

## 5. AutoDown as Living Knowledge

### Why AutoDown?

AutoDown is uniquely positioned for this because:

1. **It's already in the ecosystem**: AutoLab uses `.ad` for notebooks. AutoForge uses `.ad` for specs. Same toolchain, same parsers.
2. **It's plain text**: LLMs read and write it natively. No JSON schema to learn.
3. **It's structured**: The `$Component()` syntax provides typed, parseable metadata (status, priority, assignee) while keeping human readability.
4. **It's multi-target**: The same Ledger can become a PDF report, HTML documentation, or a notebook.
5. **It's version-control friendly**: `.ad` files diff beautifully in Git.

### Living Spec Lifecycle

```
┌──────────┐     AI proposes      ┌──────────┐
│  Human   │◄─────────────────────│  Agent   │
│  writes  │                     │  reads   │
│  spec    │                     │  code    │
└────┬─────┘                     └────┬─────┘
     │                                │
     │     Human approves/rejects     │
     │◄───────────────────────────────┤
     │                                │
     ▼                                ▼
┌──────────────────────────────────────────┐
│           Ledger (.ad) updated            │
│         Status: draft → approved          │
└──────────────────────────────────────────┘
     │
     │     Agent generates code
     ▼
┌──────────────────────────────────────────┐
│           Code committed                  │
│     Spec auto-updates to "verified"       │
└──────────────────────────────────────────┘
```

### Drift Detection

The system periodically runs:
1. Parse Ledger requirements
2. Parse code for `// spec:<id>` comments
3. Check if code referenced by spec has changed since spec was last verified
4. If yes, mark spec as `[DRIFT]` and notify

```autodown
/// section:requirements
## Requirements

### R2.3 JWT Token Flow
$Requirement(id: "R2.3", status: "drift", last_verified: "2026-05-10") {
    [⚠️ DRIFT DETECTED] The auth handler was modified
    on 2026-05-11 but this requirement was not updated.
    Diff: [View changes]
}
```

---

## 6. Backend Architecture

### New Crate: `auto-forge`

```
crates/
├── auto-playground/          # Existing: notebook + AI provider + VM
├── auto-lang/                # Existing: compiler + AutoDown
└── auto-forge/               # NEW: agent orchestration engine
    └── src/
        ├── main.rs           # Axum server (or reuse auto-playground)
        ├── forge/            # Chat loop + tool execution
        │   ├── mod.rs
        │   ├── session.rs    # Forge session (chat history, tool state)
        │   ├── tools.rs      # Tool definitions (read_file, edit_file, shell, etc.)
        │   └── loop.rs       # Main agent loop (ReAct pattern)
        ├── ledger/           # Knowledge management
        │   ├── mod.rs
        │   ├── parser.rs     # Parse .ad Ledger into structured model
        │   ├── drift.rs      # Drift detection engine
        │   └── sync.rs       # Bidirectional code-spec sync
        ├── relay/            # Agent pipeline orchestration
        │   ├── mod.rs
        │   ├── pipeline.rs   # Serial pipeline definition + execution
        │   ├── checkpoint.rs # State serialization/restore
        │   ├── handoff.rs    # Inter-agent handoff document generation
        │   └── roles.rs      # Role registry + configuration
        └── models/
            ├── spec.rs       # Spec entity types
            ├── run.rs        # Agent run state
            └── cost.rs       # Token tracking + budgeting
```

### Integration with Existing Infrastructure

| Existing Component | How AutoForge Reuses It |
|-------------------|------------------------|
| `auto-playground` Axum server | Mount `auto-forge` routes under `/smith/` |
| `notebook/ai.rs` ClaudeProvider | Extend with role-specific system prompts |
| `notebook/mod.rs` NotebookActor | Fork/adapt for agent pipeline sessions |
| `agent_debug/` | Foundation for durable VM checkpointing |
| `autodown/` parser + transpilers | Parse Ledger `.ad` files; render for UI |
| `auto-lab-ui` Vue components | Reuse CodeEditor, AIChatBar, diff rendering |

### API Design

```
# Forge
POST   /api/smith/forge/session          # Create forge session
POST   /api/smith/forge/{sid}/message     # Send message, get streaming response
GET    /api/smith/forge/{sid}/stream      # SSE: tool calls, deltas, checkpoints
POST   /api/smith/forge/{sid}/approve     # Approve a pending spec change
POST   /api/smith/forge/{sid}/reject      # Reject a pending spec change

# Ledger
GET    /api/smith/ledger                  # Get current Ledger as JSON
PUT    /api/smith/ledger                  # Update Ledger (with diff validation)
POST   /api/smith/ledger/drift-check      # Trigger drift detection
GET    /api/smith/ledger/export.{typ,html} # Export to Typst/HTML

# Relay
POST   /api/smith/relay/run               # Start a new agent pipeline run
GET    /api/smith/relay/runs              # List runs
GET    /api/smith/relay/run/{rid}         # Get run status + pipeline state
POST   /api/smith/relay/run/{rid}/pause   # Pause run
POST   /api/smith/relay/run/{rid}/resume  # Resume run
POST   /api/smith/relay/run/{rid}/rollback # Rollback to checkpoint
GET    /api/smith/relay/run/{rid}/checkpoints # List checkpoints
GET    /api/smith/relay/roles             # List configured roles
PUT    /api/smith/relay/roles/{name}      # Update role config
```

---

## 7. Frontend Architecture

### Package: `packages/auto-forge-ui/`

```
packages/auto-forge-ui/
├── src/
│   ├── App.vue                    # Top-level shell with view router
│   ├── views/
│   │   ├── ForgeView.vue          # Chat + tool loop
│   │   ├── LedgerView.vue         # Knowledge document editor
│   │   └── RelayView.vue          # Agent monitoring dashboard
│   ├── components/
│   │   ├── forge/
│   │   │   ├── ChatCanvas.vue
│   │   │   ├── ToolCallCard.vue   # File read, edit, shell
│   │   │   ├── SpecDeltaCard.vue  # Proposed spec changes
│   │   │   ├── AgentStatusPanel.vue
│   │   │   ├── LiveDiffPanel.vue
│   │   │   └── ForgeInputBar.vue
│   │   ├── ledger/
│   │   │   ├── LedgerEditor.vue   # AutoDown-aware editor
│   │   │   ├── SectionTree.vue    # Collapsible section navigator
│   │   │   ├── DriftBadge.vue
│   │   │   ├── StatusBadge.vue
│   │   │   └── ExportToolbar.vue
│   │   └── relay/
│   │       ├── PipelineFlow.vue   # Horizontal agent chain visualization
│   │       ├── RunCard.vue
│   │       ├── CheckpointTimeline.vue
│   │       ├── CostChart.vue
│   │       └── RoleConfigPanel.vue
│   ├── composables/
│   │   ├── useForge.ts            # Forge session state + streaming
│   │   ├── useLedger.ts           # Ledger CRUD + drift check
│   │   ├── useRelay.ts            # Run management + polling
│   │   └── useAutoDownEditor.ts   # AutoDown syntax highlighting + components
│   └── types/
│       ├── forge.ts
│       ├── ledger.ts
│       └── relay.ts
```

### Design System

Reuse AutoLab's Catppuccin Mocha theme but extend with:
- **Role colors**: Planner (blue), Architect (purple), Coder (green), Tester (yellow), Reviewer (orange)
- **Status colors**: draft (gray), approved (blue), in_progress (amber), verified (green), drift (red), archived (muted)
- **Forge accent**: Warm forge-orange for active operations
- **Ledger accent**: Cool ledger-teal for knowledge
- **Relay accent**: Electric relay-cyan for monitoring

---

## 8. Development Phases

### Phase 0: Foundation (2 weeks)
- Scaffold `crates/auto-forge/` and `packages/auto-forge-ui/`
- Set up shared types, API contracts
- Integrate with `auto-playground` server (mount routes)
- Basic three-view navigation shell

### Phase 1: The Furnace · 熔炉 — Chat & Loop (4 weeks)
- Implement Forge session + streaming SSE
- Port AutoLab's AIChatBar + streaming to Forge
- Build tool system: read_file, write_file, edit_file, shell, search
- Implement ReAct loop with tool calls
- Live diff panel for file changes
- Approve/reject gates for spec changes

### Phase 2: The Jade Tabs · 玉简 — Knowledge (3 weeks)
- AutoDown Ledger parser (frontend + backend)
- Section tree navigator with status badges
- Drift detection engine
- Bidirectional sync: code ↔ specs
- AutoDown editor with component autocomplete (`$Goal()`, `$Requirement()`)
- Export to Typst/HTML

### Phase 3: The Array · 法阵 — Serial Agents (4 weeks)
- Pipeline definition DSL (YAML/AutoDown)
- Serial execution engine with checkpointing
- Agent handoff protocol
- Role registry + configuration
- Pipeline visualization (flow diagram)
- Run history + cost analytics
- Pause/resume/rollback

### Phase 4: Integration & Polish (3 weeks)
- AutoLab ↔ AutoForge bridge (open AutoForge from notebook, vice versa)
- Durable execution: crash recovery, VM state restore
- Cost-aware model routing (cheap model for simple tasks)
- Multi-project support
- Full test coverage (backend + frontend)

**Total estimate: 16 weeks (~4 months)**

---

## 9. Differentiation Summary

| Dimension | AutoForge | Best Alternative |
|-----------|-----------|------------------|
| **Agent model** | Serial pipeline with handoffs | Cursor's parallel agents |
| **Specs** | Living, bidirectional, drift-detecting | Intent's static living specs |
| **Knowledge format** | AutoDown (native to ecosystem) | Markdown / proprietary JSON |
| **Cost** | ~5× cheaper than parallel multi-agent | Multi-agent is expensive |
| **Monitoring** | First-class pipeline + checkpoint UI | Slack messages (Devin) |
| **Human-in-the-loop** | At spec layer (before code) | At code layer (after generation) |
| **Integration** | Native AutoLang VM + AutoDown | External / generic |
| **Durability** | Checkpoint/restore as first-class | tmux / git hacks |

---

## 10. Open Questions

### Decisions (Finalized)

1. **Deployment**: ✅ Separate app (`auto-forge-ui`), sharing components with AutoLab
2. **VM sessions**: ✅ Forge reuses AutoLab's notebook VM sessions — seamless test-and-iterate
3. **MVP scope**: ✅ Forge + Ledger — chat loop + spec management as first deliverable

### Open Question

**How does AutoForge handle non-AutoLang projects?**
   - The tool system is language-agnostic (file I/O, shell)
   - But the VM integration and AutoDown specs are AutoLang-native
   - *Recommendation*: Support any language for file operations, but AutoLang projects get full Ledger + VM integration

---

*This document is a living design. It should be updated as implementation progresses and new insights emerge.*
