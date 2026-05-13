# Agents Module: Relay Orchestration Design

> **Status:** Design Exploration  
> **Goal:** Define how Agents — as distinct, profession-bearing entities with their own Souls, Models, and Specs — are managed and orchestrated in AutoForge to maximize quality and token efficiency through serial cooperation.

---

## 1. The Philosophy: Craft Over Swarm

Most multi-agent systems today are **swarms**: many agents buzzing in parallel, burning tokens on coordination overhead. AutoForge rejects this.

Our metaphor is the **workshop**, not the hive:

- A **Planner** studies the commission and writes a blueprint
- An **Architect** selects materials and designs joints
- A **Coder** cuts and assembles the pieces
- A **Tester** checks every seam under load
- A **Reviewer** inspects the finish before delivery

Each craftsperson works with **care**, not haste. They pass a **baton** — a carefully prepared handoff — rather than shouting over each other. The goal is not to finish in seconds, but to finish so well that no token is ever spent on rework.

### The Five Mandates

| # | Mandate | Implication |
|---|---------|-------------|
| 1 | **Token efficiency over work speed** | Agents must never see context they don't need. Handoffs are compressed. Models are tiered to task difficulty. |
| 2 | **Work longer rather than quicker** | A flow may run for hours. Checkpointing makes this safe. Quality requires deliberation. |
| 3 | **Quality is superior to speed** | Every handoff includes a validation gate. Rework loops are expected and cheap. |
| 4 | **Relay mode, not parallel mode** | One agent holds the baton at a time. Context is scoped, not broadcast. |
| 5 | **Profession-based cooperation per Specs** | Each agent reads the spec sections relevant to its profession, updates the sections it owns, and hands off to the next profession defined in the flow. |

---

## 2. Agent Anatomy: Soul + Profession + Model

An Agent is not just a system prompt. It is a **persistent identity** with three layers:

```
┌─────────────────────────────────────────┐
│              AGENT IDENTITY             │
├─────────────────────────────────────────┤
│  SOUL  — Who you are, how you think     │
│  (values, style, rituals, standards)    │
├─────────────────────────────────────────┤
│  PROFESSION — What you do, your scope   │
│  (tools, specs, gates, handoff rules)   │
├─────────────────────────────────────────┤
│  MODEL — Your cognitive substrate       │
│  (provider, model, temperature, budget) │
└─────────────────────────────────────────┘
```

### 2.1 The Soul (SOUL.md)

Inspired by OpenClaw, the Soul is a markdown document that defines the agent's character. It is **not** a system prompt — it is a living document that the agent reads at the start of every turn, alongside the specs.

```markdown
# Soul of the Architect

## Core Values
- **Simplicity over cleverness**: If a junior can't understand it in 5 minutes, it's wrong.
- **Explicit over implicit**: Every assumption is documented in the spec.
- **Stability over novelty**: Proven patterns beat fashionable abstractions.

## Working Style
- Before proposing any design, I read the current Architecture and Designs specs.
- I never modify code. I only modify specs (Architecture, Designs, APIs).
- I write handoffs as structured documents, not chat transcripts.

## Handoff Ritual
When I finish my work, I produce:
1. **Decisions Made**: A bullet list of architectural decisions with rationale
2. **Open Questions**: Anything the next agent needs to decide
3. **Spec Updates**: Which sections I modified and why
4. **Context for Next Agent**: Files to read, specs to follow, traps to avoid

## Quality Standard
- I do not approve designs with unhandled error cases.
- I do not approve designs without explicit data lifecycle definitions.
```

**Why a Soul file?**
- It separates **identity** from **instruction** (the system prompt is "how to use tools"; the Soul is "who you are")
- It can be versioned, reviewed, and shared across projects
- It allows human craftsmen to "tune" an agent's personality without rewriting prompts

### 2.2 The Profession

The Profession defines what the agent **can and cannot do**. It is the bridge between the Soul and the system.

```rust
pub struct Profession {
    pub id: String,                    // "architect"
    pub name: String,                  // "Architect"
    pub phase: ForgePhase,             // Which phase this profession belongs to
    pub owned_sections: Vec<SectionType>, // Can write to these spec sections
    pub readable_sections: Vec<SectionType>, // Can read these sections
    pub tools: ToolFilter,             // Subset of ToolRegistry available
    pub handoff_to: Vec<String>,       // Professions that may receive handoffs
    pub approval_gates: Vec<Gate>,     // Where human approval is required
    pub max_turns: u32,                // How many LLM turns before forced handoff
    pub token_budget: TokenBudget,
}
```

**Built-in Professions:**

| Profession | Phase | Owned Specs | Model Tier | Key Constraint |
|------------|-------|-------------|------------|----------------|
| **Intaker** | Intake | None | Cheap (Haiku/4o-mini) | Classify only, no tools |
| **Planner** | SpecDraft | Goals, Plans | Medium (Sonnet/4o) | No code tools |
| **Architect** | SpecDraft | Architecture, Designs, APIs | Strong (Sonnet/4o) | No code tools |
| **Coder** | Execution | None (writes code files) | Strong (Sonnet/4o) | Reads Plans, Tests |
| **Tester** | Execution | Tests | Medium (Sonnet/4o) | Runs shell, reads code |
| **Reviewer** | Verification | Reviews, Reports | Strong (Opus/o3) | Read-only on code |
| **Documenter** | Any | Reports | Cheap (Haiku/4o-mini) | Read-only |

### 2.3 The Model Configuration

Each profession specifies a default model, but individual Agents can override:

```rust
pub struct ModelConfig {
    pub provider: Provider,            // Anthropic, OpenAI, local, etc.
    pub model: String,                 // "claude-3-5-sonnet-20241022"
    pub temperature: f32,              // 0.0 for Reviewer, 0.7 for Planner
    pub max_tokens: u32,               // Per-turn limit
    pub reasoning_budget: Option<u32>, // For reasoning models (o3, Claude thinking)
}
```

**Model Tiering for Token Efficiency:**

| Tier | Models | Cost | Used For |
|------|--------|------|----------|
| **Cheap** | Haiku, 4o-mini, local 8B | ~$0.10/1M tk | Intake, classification, summarization, documenter |
| **Standard** | Sonnet, 4o, Gemini Pro | ~$3/1M tk | Planner, Coder, Tester |
| **Strong** | Opus, o3, Gemini Ultra | ~$15/1M tk | Architect, Reviewer, complex analysis |

> **Principle:** Start cheap. If a cheap model fails a handoff validation, escalate to standard. If standard fails, escalate to strong. Never use a strong model where a cheap one suffices.

---

## 3. The Relay Orchestrator

The Relay Orchestrator is the **conductor** of the workshop. It does not think — it manages.

### 3.1 Core Responsibilities

1. **Load the Flow**: Read the Flow Spec (which professions run in what order)
2. **Spawn Agents**: Create agent instances with Soul + Profession + Model
3. **Manage the Baton**: Decide which agent holds control at any moment
4. **Execute Handoffs**: Compress context, pass baton, validate receipt
5. **Enforce Gates**: Pause for human approval where required
6. **Track Budgets**: Monitor token spend per run, per agent, per flow
7. **Checkpoint**: Save full state after every handoff

### 3.2 The Flow Spec

A Flow is a spec itself — it lives in the Ledger or in `.autoforge/flows/{name}.flow.ad`:

```autodown
# Flow: Feature Implementation

$Flow(id: "feature_impl", version: 1) {
  description: "Standard flow for implementing a new feature from goal to deployment"
}

## Pipeline

$Step(profession: "intaker", gate: "auto") {
  exit: "classify(intent) -> branch"
}

$Step(profession: "planner", gate: "human") {
  requires: [Goals, Architecture]
  produces: [Goals, Plans]
  exit: "handoff_to: architect"
}

$Step(profession: "architect", gate: "human") {
  requires: [Goals, Plans, Architecture]
  produces: [Architecture, Designs, APIs]
  exit: "handoff_to: coder"
}

$Step(profession: "coder", gate: "auto") {
  requires: [Plans, Designs, APIs]
  produces: [code_files]
  loop: {
    max_iterations: 5,
    on_fail: "handoff_to: architect"
  }
  exit: "handoff_to: tester"
}

$Step(profession: "tester", gate: "auto") {
  requires: [Tests, code_files]
  produces: [Tests, test_results]
  loop: {
    max_iterations: 3,
    on_fail: "handoff_to: coder"
  }
  exit: "handoff_to: reviewer"
}

$Step(profession: "reviewer", gate: "human") {
  requires: [all]
  produces: [Reviews, Reports]
  exit: "complete"
}
```

**Key concepts:**
- **`gate: human/auto`**: Human gates pause the flow and notify the user. Auto gates proceed immediately.
- **`loop`**: Some steps naturally loop (Coder → Tester → Coder). The loop has a max iteration bound.
- **`branch`**: The Intaker can branch to different flows (DIRECT → single Coder step; NEW_GOAL → full pipeline).
- **`on_fail`**: If an agent cannot complete its work (e.g., tests keep failing), the flow can route backward to a different profession.

### 3.3 Baton State Machine

```
                    ┌─────────────┐
   human approves   │   WAITING   │   human rejects
   ───────────────►│   FOR HUMAN │◄──────────────
                   │   APPROVAL  │
                   └──────┬──────┘
                          │ human approves
                          ▼
                   ┌─────────────┐
   spawn agent ──► │   RUNNING   │ ◄── resume
                   │   (agent    │
                   │   holds     │
                   │   baton)    │
                   └──────┬──────┘
                          │ agent completes / max turns / error
                          ▼
                   ┌─────────────┐
                   │  HANDOFF    │
                   │  (compress  │
                   │  context)   │
                   └──────┬──────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ COMPLETE │   │ CHECKPOINT│   │  ERROR   │
    │          │   │  (save)   │   │          │
    └──────────┘   └────┬─────┘   └────┬─────┘
                        │              │
                        ▼              ▼
                   ┌──────────┐   ┌──────────┐
                   │  NEXT    │   │  RETRY   │
                   │  STEP    │   │  / ROLLBACK
                   └──────────┘   └──────────┘
```

---

## 4. The Handoff Protocol: Token Efficiency at the Core

The Handoff is the **most critical mechanism** for token efficiency. It is where we prevent context explosion.

### 4.1 The Problem

In a chat-based single-agent system, context grows monotonically:
```
Turn 1: 2k tokens
Turn 5: 10k tokens
Turn 10: 20k tokens
Turn 20: 40k tokens (summarized or truncated)
```

In a parallel multi-agent system, context is multiplied:
```
6 agents × 20k context each = 120k tokens loaded per round
3 rounds of coordination = 360k tokens of overhead
```

In the Relay model, context is **scoped and reset** at every handoff:
```
Planner: 5k context → handoff (1k) →
Architect: 8k context → handoff (1.5k) →
Coder: 15k context → handoff (1k) →
Tester: 10k context → handoff (0.5k) →
Reviewer: 12k context
Total: ~54k tokens vs 360k+ in parallel
```

### 4.2 The Handoff Document

When an agent finishes, the Orchestrator (with help from the agent) produces a **Handoff Document**. This is NOT the raw chat history. It is a structured, compressed artifact:

```autodown
/// handoff:from=coder to=tester run=42 step=3 checkpoint=7

# Handoff: JWT Authentication Implementation

## Executive Summary (≤ 200 words)
The Coder has implemented the JWT authentication flow as specified in Designs:D3.2.
All 4 planned modules are written: token.rs, middleware.rs, routes.rs, errors.rs.

## Decisions Made
- $Decision(id: "C7", status: "made") {
    Used `jsonwebtoken` crate instead of `frank_jwt` due to better AutoLang FFI support.
    Rationale: frank_jwt has a transitive dependency on openssl-sys which breaks Windows builds.
}

## Spec Updates
- Modified: Plans:P5.3 → status: done
- Modified: Designs:D3.2 → added error handling section

## Work Product
- `crates/auth/src/token.rs` (187 lines)
- `crates/auth/src/middleware.rs` (94 lines)
- `crates/auth/src/routes.rs` (156 lines)
- `crates/auth/src/errors.rs` (43 lines)

## Context for Tester
```files
// Read these files to understand the implementation:
crates/auth/src/token.rs        # Token generation and validation
crates/auth/src/middleware.rs   # Axum middleware integration
crates/auth/tests/jwt_test.rs   # Empty — write tests here
```

```specs
// Follow these specs:
- Tests:T3.1 (JWT token validation)
- Tests:T3.2 (Middleware rejection paths)
- Designs:D3.2 (Error handling contract)
```

## Known Issues / Open Questions
- $Question(id: "Q2", status: "open") {
    Refresh token rotation is NOT implemented. Architect decided to defer to Phase 2.
    Tester should verify that the current implementation explicitly rejects refresh token requests.
}
- Token expiry is hardcoded to 3600s. No configuration hook yet.

## Token Spend
- This step: 14,320 tokens (input: 8,240, output: 6,080)
- Budget remaining: 85,680 / 100,000
```

**Compression rules:**
- Chat history is **discarded** — only decisions and work product survive
- File contents are **referenced**, not inlined (the next agent reads what it needs)
- Tool call results are **summarized** (e.g., "Test run: 14 passed, 2 failed" instead of full output)
- The Soul of the next agent tells it what to pay attention to in the handoff

### 4.3 Context Window Management

Each agent receives exactly three things at spawn:

1. **Soul** (~500-2000 tokens): Who they are
2. **Relevant Specs** (~1000-5000 tokens): What the project needs
3. **Handoff Document** (~500-2000 tokens): What the previous agent did

**Total per agent: ~2k–9k tokens** vs. monolithic 40k+ context windows.

If an agent needs more context, it **reads files on demand** via tools. This is pull-based, not push-based — the agent only fetches what its profession tells it to examine.

---

## 5. Checkpoint & Durability: Working Longer Safely

Since agents "work longer rather than quicker," a flow may run for hours or days. The system must be **crash-proof**.

### 5.1 Checkpoint Contents

After every handoff, a checkpoint serializes:

```rust
pub struct Checkpoint {
    pub id: u64,                       // Monotonic within run
    pub run_id: String,
    pub timestamp: u64,
    pub git_commit: Option<String>,    // Auto-commit "checkpoint-N" if git is clean
    pub git_diff: Option<String>,      // Or store diff if dirty
    pub ledger_state: SpecsDocument,   // Full specs at this point
    pub handoff: HandoffDocument,      // The handoff that triggered this checkpoint
    pub agent_context: AgentContext,   // Summarized, not raw chat
    pub file_manifest: Vec<FileState>, // Hash of every file touched
    pub token_usage: TokenUsage,       // Cumulative spend
}
```

### 5.2 Resume Protocol

When resuming a run:

1. Load the latest checkpoint
2. Restore files (git reset --hard or apply diff)
3. Restore Ledger
4. **Rehydrate the next agent**: Send Soul + relevant Specs + Handoff Document
5. The agent receives NO chat history. It starts fresh from the handoff.

This means even after a 12-hour pause, the next agent costs the same tokens as if the flow were continuous.

### 5.3 Rollback & Fork

- **Rollback**: Restore checkpoint N, discard N+1 onward. The flow resumes from N with a note in the handoff explaining the rollback reason.
- **Fork**: Create a new run from checkpoint N with different parameters (e.g., "try with a different Architect" or "use Gemini instead of Claude"). This allows A/B testing of agent configurations.

---

## 6. Token Budgeting: First-Class Cost Awareness

Token efficiency is not an afterthought. It is a **first-class constraint**.

### 6.1 Budget Hierarchy

```
Project Budget (monthly)
  └── Flow Budget (per run)
        └── Step Budget (per agent step)
              └── Turn Budget (per LLM API call)
```

```rust
pub struct TokenBudget {
    pub limit: u64,           // Hard limit
    pub warning_at: u64,      // Soft limit — agent is warned
    pub strategy: BudgetStrategy,
}

pub enum BudgetStrategy {
    HardStop,         // Halt step, request human decision
    EscalateModel,    // Switch to cheaper model for remainder
    SummarizeContext, // Aggressively compress context
    SkipOptional,     // Skip non-critical work (e.g., documentation)
}
```

### 6.2 Budget Actions

When an agent approaches its limit:

1. **Warning** (at 70%): Agent is instructed to wrap up quickly and hand off
2. **Compression** (at 85%): Context is auto-summarized by a cheap model; redundant specs are dropped
3. **Hard stop** (at 100%): Agent is forced to hand off with whatever it has. The next agent receives a note: "Previous agent exhausted budget. Context may be incomplete."

### 6.3 Cost Analytics

The Agents view shows:

```
Run #42 — JWT Auth Implementation
─────────────────────────────────
Planner      5,240 tk   $0.02    ✅
Architect   18,100 tk   $0.54    ✅
Coder       42,500 tk   $1.27    ✅
Tester      23,400 tk   $0.70    🔄 (in progress)
Reviewer       —         —       ⏳
─────────────────────────────────
Total so far: 89,240 tk   $2.53
Budget: 150,000 tk   $5.00
Savings vs parallel estimate: $12.40 (83% cheaper)
```

---

## 7. Spec-Driven Agent Cooperation

Agents do not collaborate via chat. They collaborate via **Specs** — the Ledger is their shared workspace.

### 7.1 Spec Ownership

Each profession "owns" certain spec sections:

| Profession | Owns (can write) | Reads (context) |
|------------|------------------|-----------------|
| Planner | Goals, Plans | Architecture (read-only) |
| Architect | Architecture, Designs, APIs | Goals, Plans |
| Coder | None (writes code) | Plans, Designs, APIs, Tests |
| Tester | Tests | Plans, Designs, code |
| Reviewer | Reviews, Reports | All specs, all code |

This prevents conflicts: two agents never write to the same spec section simultaneously (because the Relay ensures only one agent runs at a time).

### 7.2 Spec as Contract

When the Architect hands off to the Coder, the contract is:

> "Coder, implement Designs:D3.2 and APIs:A2.1. Do not deviate without updating the Designs spec and routing back to Architect."

If the Coder discovers a problem, they do NOT work around it. They:
1. Document the issue in the handoff
2. Request a route-back to Architect (via the Orchestrator)
3. The Orchestrator may spawn the Architect with the new context

This ensures specs remain the **source of truth**.

### 7.3 Drift Detection in the Flow

After the Reviewer completes, an automatic Drift Checker runs:
- Read all Goals
- Read all implemented code files
- Verify: Does the code satisfy each goal?
- If not, mark the goal as `drift` and spawn the Planner for a remediation flow

---

## 8. Implementation Architecture

### 8.1 New Crate: `auto-forge/src/relay/`

```
crates/auto-forge/src/
├── forge/              # Existing: single-agent chat + tools
│   ├── mod.rs
│   ├── ai.rs
│   └── tools.rs
├── relay/              # NEW: multi-agent orchestration
│   ├── mod.rs          # RelayOrchestrator, public API
│   ├── flow.rs         # FlowSpec parser and validator
│   ├── pipeline.rs     # Pipeline execution engine
│   ├── handoff.rs      # HandoffDocument generation + compression
│   ├── checkpoint.rs   # State serialization / restore
│   ├── agent.rs        # AgentInstance (Soul + Profession + Model)
│   ├── soul.rs         # SoulConfig loader and renderer
│   ├── profession.rs   # Profession registry and defaults
│   ├── budget.rs       # TokenBudget tracking and enforcement
│   ├── roles.rs        # Role registry (deprecated by profession.rs)
│   └── run.rs          # RunState, RunStore
└── models/
    └── ...
```

### 8.2 Key Data Structures

```rust
// relay/agent.rs
pub struct AgentInstance {
    pub id: String,
    pub profession: Profession,
    pub soul: SoulConfig,
    pub model: ModelConfig,
    pub context: AgentContext,         // Current turn's context (NOT persisted)
}

// relay/handoff.rs
pub struct HandoffDocument {
    pub from: String,
    pub to: String,
    pub run_id: String,
    pub checkpoint_id: u64,
    pub summary: String,
    pub decisions: Vec<Decision>,
    pub open_questions: Vec<Question>,
    pub spec_updates: Vec<SpecUpdate>,
    pub work_product: Vec<WorkProduct>,
    pub context_for_next: ContextPointers,
    pub token_usage: TokenUsage,
}

// relay/pipeline.rs
pub struct PipelineEngine {
    pub flow: FlowSpec,
    pub current_step: usize,
    pub current_agent: Option<AgentInstance>,
    pub checkpoints: Vec<Checkpoint>,
    pub status: PipelineStatus,
}

pub enum PipelineStatus {
    Idle,
    Running { agent_id: String, started_at: u64 },
    WaitingForHuman { gate: Gate, since: u64 },
    Completed { result: CompletionResult },
    Failed { error: String, checkpoint: u64 },
    Paused { at_checkpoint: u64 },
}
```

### 8.3 API Surface

```
# Relay (new endpoints)
POST   /api/smith/relay/run                # Start a run from a FlowSpec
GET    /api/smith/relay/runs               # List runs
GET    /api/smith/relay/run/{rid}          # Get run status
POST   /api/smith/relay/run/{rid}/pause    # Pause run
POST   /api/smith/relay/run/{rid}/resume   # Resume run
POST   /api/smith/relay/run/{rid}/rollback # Rollback to checkpoint N
GET    /api/smith/relay/run/{rid}/checkpoints
POST   /api/smith/relay/run/{rid}/approve  # Approve a human gate
POST   /api/smith/relay/run/{rid}/reject   # Reject and redraft

# Agent Registry
GET    /api/smith/relay/professions        # List built-in + custom professions
GET    /api/smith/relay/professions/{id}   # Get profession details
PUT    /api/smith/relay/professions/{id}   # Create/update custom profession
GET    /api/smith/relay/souls              # List available Souls
GET    /api/smith/relay/souls/{id}         # Get Soul markdown

# Flows
GET    /api/smith/relay/flows              # List flow specs
GET    /api/smith/relay/flows/{id}         # Get flow definition
PUT    /api/smith/relay/flows/{id}         # Create/update flow
```

### 8.4 Integration with Existing Forge

The existing single-agent Forge becomes **one step in the Relay** — specifically, the Coder step. The chat loop in `forge/mod.rs` is extracted into a reusable `AgentTurn` engine:

```rust
// forge/turn.rs — extracted from existing forge_stream
pub struct AgentTurn {
    pub agent: AgentInstance,
    pub tools: Vec<Box<dyn Tool>>,
    pub messages: Vec<ChatMessage>,
    pub max_turns: u32,
}

impl AgentTurn {
    pub async fn run(&mut self, tx: mpsc::Sender<StreamEvent>) -> TurnResult {
        // Existing ReAct loop, but parameterized by AgentInstance
    }
}
```

The Relay Orchestrator spawns an `AgentTurn` for the current step, monitors it, and when it completes (or hits a handoff condition), generates the Handoff Document and proceeds.

---

## 9. Human-in-the-Loop: Approval Gates

Even with full automation, humans are the **final authority** at key gates.

### 9.1 Gate Types

| Gate | When | UI |
|------|------|-----|
| **Spec Gate** | After Planner/Architect | Side-by-side spec diff, Approve/Reject/Edit |
| **Code Gate** | After Coder (optional) | File diff view, Approve/Reject/Comment |
| **Review Gate** | After Reviewer | Review report with findings, Approve/Request Changes |
| **Budget Gate** | When budget exceeded | Notification: "Step over budget. Continue? Escalate? Abort?" |
| **Error Gate** | On unexpected failure | Error details, Retry/Rollback/Abort |

### 9.2 Async Notification

When a human gate is reached:
1. Run state → `WaitingForHuman`
2. SSE event sent to frontend
3. Notification pushed (desktop, email, or Slack webhook)
4. Human can respond immediately or hours later
5. On resume, the flow continues exactly where it left off (via checkpoint)

---

## 10. Roadmap: From Single Agent to Relay

### Phase 1: Soul + Profession Framework (2 weeks)
- [ ] Define `SoulConfig`, `Profession`, `ModelConfig` structs
- [ ] Create built-in professions (Intaker, Planner, Architect, Coder, Tester, Reviewer)
- [ ] Create default Souls in `.autoforge/souls/`
- [ ] Refactor existing Forge to use `AgentTurn` with parameterized profession
- [ ] Backend: Profession registry API

### Phase 2: Handoff Protocol (2 weeks)
- [ ] Define `HandoffDocument` schema
- [ ] Implement context compression / summarization (cheap model or rule-based)
- [ ] Implement spec-scoping: filter specs by profession
- [ ] Add `handoff` tool that agents can call to surrender the baton
- [ ] Backend: Handoff generation and persistence

### Phase 3: Flow Engine (2 weeks)
- [ ] Define `FlowSpec` parser (AutoDown-based)
- [ ] Implement `PipelineEngine` with step execution
- [ ] Implement branching (Intaker → different flows)
- [ ] Implement looping (Coder → Tester → Coder)
- [ ] Add human gates with SSE notifications

### Phase 4: Checkpoint & Durability (1 week)
- [ ] Implement `Checkpoint` serialization
- [ ] Git integration (auto-commit on checkpoint)
- [ ] Resume from checkpoint (rehydrate agent)
- [ ] Rollback and Fork

### Phase 5: Token Budgeting & Analytics (1 week)
- [ ] Per-step token tracking
- [ ] Budget enforcement (warning, compression, hard stop)
- [ ] Cost analytics in Agents view
- [ ] "Savings vs parallel" estimation

### Phase 6: Frontend — The Agents View (2 weeks)
- [ ] Pipeline visualization (horizontal flow)
- [ ] Run cards with live status
- [ ] Checkpoint timeline
- [ ] Cost charts
- [ ] Role/Profession config panel
- [ ] Human gate approval UI

**Total: ~10 weeks**

---

## 11. Open Questions

1. **Should Souls be project-specific or global?**
   - Proposal: Global defaults in `~/.autoforge/souls/`, project overrides in `.autoforge/souls/`. Project Soul overrides global.

2. **How do we handle model unavailability?**
   - Proposal: Each profession defines a `fallback_chain: ["claude-3-5-sonnet", "gpt-4o", "local-llama-70b"]`. If primary fails, auto-fallback with notification.

3. **Can a human replace an agent mid-flow?**
   - Proposal: Yes. At any handoff, a human can "pick up the baton" — they receive the same Handoff Document the agent would, work in the Forge chat, then hand off to the next agent. This enables hybrid human-AI workflows.

4. **How do we prevent infinite loops (Coder ↔ Tester)?**
   - Proposal: Loop config has `max_iterations`. After max, escalate to Reviewer or human gate. Also, token budget acts as a global safety net.

5. **Should the Orchestrator itself use an LLM?**
   - Proposal: No. The Orchestrator is deterministic Rust code. It reads Flow specs, manages state machines, and routes. No LLM calls = no token spend on coordination = no unpredictable routing decisions.

---

## 12. Summary

The Agents module transforms AutoForge from a **single chat agent** into a **workshop of specialized craftspeople**:

- **Agents** have **Souls** (character), **Professions** (scope), and **Models** (capabilities)
- **Relay orchestration** passes a baton serially through professions defined in a Flow spec
- **Handoff documents** compress context, preventing the token explosion of parallel multi-agent systems
- **Checkpointing** enables long-running flows that survive crashes, pauses, and human delays
- **Token budgeting** makes cost a first-class constraint, not a surprise bill
- **Specs are the shared workspace** — agents cooperate by reading and writing the Ledger, not by chatting

The result is a system that:
- Uses **~5× fewer tokens** than parallel multi-agent approaches
- Produces **higher quality** through validation at every handoff
- Runs for **hours or days** safely through checkpointing
- Keeps humans in control **at the spec layer**, before code is written

> *"A single blacksmith finishes one sword a day. Ten blacksmiths working in parallel finish ten swords — each flawed. But five blacksmiths working in relay, each a master of their stage, finish one sword — perfect. AutoForge chooses the relay."*
