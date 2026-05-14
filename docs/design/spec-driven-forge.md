# Spec-Driven Forge Design v2

Inspired by Superpowers (brainstorm → plan → execute → review) and GSD
(file-based state, fresh agents per task, wave execution).

## Core Principle

> **The Jades are the source of truth.** Every decision the AI makes is resolved
> by reading the spec. No code is written before the spec is drafted and approved.

---

## 1. The Relay Phases

The boss gives one order. The AI workforce executes autonomously.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  User Request (Boss)                                                        │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  PHASE 1    │───►│  PHASE 2    │───►│  PHASE 3    │───►│  PHASE 4    │ │
│  │  Assistant  │    │  Advisor    │    │  SpecDraft  │    │  Execution  │ │
│  │  & Classify │    │  & Discover │    │  Architect  │    │  & Verify   │ │
│  │             │    │  & Goals    │    │  + Planner  │    │             │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘ │
│       │                   │                   │                   │        │
│       │ classify          │ draft goals       │ draft designs     │        │
│       │ intent            │ ask questions     │ + plans + tests   │        │
│       │                   │                   │                   │        │
│       ▼                   ▼                   ▼                   ▼        │
│   QUESTION ─────────────► answer & stop                                   │
│   DIRECT ───────────────► skip to Execution (small changes only)          │
│   NEW_GOAL ─────────────► full pipeline                                   │
│   REQ_UPDATE ───────────► full pipeline                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

**GSD Mode (default):** Only the Advisor→Architect gate is human (for now). All other gates are auto.
**Check Mode (opt-in):** Boss reviews every stage.

### Phase 1: Assistant — Intake & Classify

**What happens:** The Assistant (secretary) reads the user's request and classifies intent.

**Intent types:**
- `QUESTION` — "How does auth work?" → answer and stop
- `DIRECT` — "Fix typo on line 42" → skip to Execution (threshold: <10 lines changed)
- `NEW_GOAL` — "Add OAuth2 login" → full pipeline
- `REQ_UPDATE` — "Change timeout to 10 min" → full pipeline

**Output:** Classification + brief reasoning. Hands off to Advisor or Coder.

### Phase 2: Advisor — Discovery & Goals

**What happens:** The Advisor brainstorms with the user, clarifies requirements, and drafts Goals.

**Process:**
1. **Read** current Goals to avoid duplication
2. **Ask** clarifying questions (1–3 focused questions max)
3. **Draft** Goals using `write_jade`:
   - Goals — single-sentence, testable objectives
4. **Present** summary and ask for approval

**Rule:** No `read_file` / `write_file` tools. Only `read_jade`, `write_jade`, `list_jades`.

**GSD Mode:** After Goals are drafted, the boss is asked to approve. On approval, the relay continues autonomously.
**Check Mode:** Boss reviews every spec update.

### Phase 3: SpecDraft — Architect + Planner

**What happens:** The Architect designs structure and modules. The Planner writes execution phases. The Tester drafts tests.

**Process (spec-driven, top-down):**
1. **Architect** reads Goals, writes Architecture and Designs (including interfaces)
2. **Planner** reads Goals, Architecture, Designs — writes Plans (phased execution)
3. **Tester** reads Goals, Designs — drafts Tests

**Rule:** No `read_file` / `write_file` tools available in this phase. Only Jades tools.

**GSD Mode:** This phase runs automatically. No human interruption.
**Check Mode:** Boss can review Architecture, Designs, Plans, and Tests before execution.

### Phase 4: Execute & Verify

**What happens:** Coder implements based on approved specs. Tester verifies. Reviewer audits. Documenter reports.

**Process (plan-driven, bottom-up):**
1. **Read** approved Plans and related Tests from Specs
2. **Coder** executes plan phases in order:
   - Read relevant files
   - Write failing test (TDD mode)
   - Implement minimal code
   - Verify test passes
   - Update Plan phase status → `in_progress` → `done`
3. **Tester** runs tests. If failing, loops back to Coder (max 3 iterations).
4. **Reviewer** reads Goals, Tests, code — performs drift check
5. **Documenter** compiles Report (execution summary, cost, confidence)
6. **Done** — session returns to `Idle`, Report presented to boss

**GSD Mode:** Fully autonomous. Boss only sees the final Report.
**Check Mode:** Boss reviews code and review findings before delivery.

---

## 2. Jades Persistence

### 2.1 File-Based State (GSD pattern)

Jades are stored as JSON on disk, but the AI interacts with them as structured
sections. This mirrors GSD's file-based state philosophy.

```
~/.local/share/autoforge/
├── sessions/
│   └── forge-{uuid}.json          # Session state (messages, phase, pending changes)
├── ledgers/
│   └── {project}.json             # Jades for each project
│       {
│         "project": ".",
│         "version": 3,           # optimistic concurrency
│         "sections": [
│           {"id": "goals", "title": "📋 Goals", "status": "approved", "content": "..."},
│           {"id": "architecture", "title": "🏗️ Architecture", "status": "draft", "content": "..."},
│           {"id": "designs", "title": "🎨 Designs", "status": "draft", "content": "..."},
│           {"id": "plans", "title": "📅 Plans", "status": "approved", "content": "..."},
│           {"id": "tests", "title": "🧪 Tests", "status": "draft", "content": "..."},
│           {"id": "reports", "title": "📊 Reports", "status": "draft", "content": "..."},
│           {"id": "reviews", "title": "📝 Reviews", "status": "draft", "content": "..."}
│         ]
│       }
```

### 2.2 LedgerStore

```rust
struct LedgerStore {
    projects: HashMap<String, LedgerDocument>,
    data_dir: PathBuf,
}
```

- Auto-load on startup, auto-save on mutation
- One document per `project_path`
- `version` field for optimistic concurrency (detect stale overwrites)

### 2.3 API Endpoints

```
GET    /api/smith/ledger/{project}              → LedgerDocument
PUT    /api/smith/ledger/{project}              → update full document (with version check)
GET    /api/smith/ledger/{project}/{section_id}  → single section
PUT    /api/smith/ledger/{project}/{section_id}  → update section
POST   /api/smith/ledger/{project}/drift-check   → compare code vs specs
```

---

## 3. AI Tools

### 3.1 Jades Tools (new)

```rust
// read_jade — read one section
{"name": "read_spec", "arguments": {"section_id": "goals"}}
→ Returns the content and status of the section

// write_jade — update one section
{"name": "write_jade",
 "arguments": {"section_id": "plans",
               "content": "Phase 1: ...\nPhase 2: ...",
               "status": "draft"}}
→ Updates the section, increments ledger version

// list_jades — list all sections with summaries
{"name": "list_jades", "arguments": {}}
→ Returns [{"id": "goals", "title": "📋 Goals", "status": "approved", "word_count": 120}, ...]
```

### 3.2 Existing Tools (unchanged)

`read_file`, `write_file`, `edit_file`, `shell`, `search`

### 3.3 Tool Availability by Phase

| Phase | Profession | read_jade | write_jade | read_file | write_file | shell |
|---|---|---|---|---|---|---|
| Intake | Assistant | ✅ | ❌ | ✅ | ❌ | ❌ |
| Discovery | Advisor | ✅ | ✅ | ❌ | ❌ | ❌ |
| Design | Architect | ✅ | ✅ | ❌ | ❌ | ❌ |
| Planning | Planner | ✅ | ✅ | ❌ | ❌ | ❌ |
| Execution | Coder | ✅ | ✅ | ✅ | ✅ | ❌ |
| Execution | Tester | ✅ | ✅ | ✅ | ✅ | ✅ |
| Verification | Reviewer | ✅ | ❌ | ✅ | ❌ | ❌ |
| Report | Documenter | ✅ | ❌ | ✅ | ❌ | ❌ |

**Rule:** If the AI tries to use a forbidden tool, the tool returns an error explaining which gate it's in and what to do instead.

---

## 4. Session State Extensions

### 4.1 ForgeSession

```rust
pub struct ForgeSession {
    pub id: String,
    pub notebook_sid: Option<String>,
    pub project_path: String,
    pub status: ForgeStatus,
    pub phase: ForgePhase,                    // NEW
    pub messages: Vec<ForgeMessage>,
    pub pending_spec_changes: Vec<SpecChange>, // NEW: queued during Goal Gate
    pub current_phase_index: Option<usize>,    // NEW: which plan phase we're executing
}
```

### 4.2 ForgePhase

```rust
enum ForgePhase {
    Intake,
    SpecDraft,
    GoalGate,  // Human approval of Goals (GSD: required; Check: required)
    Execution,
    Verification,
}
```

### 4.3 SpecChange

```rust
pub struct SpecChange {
    pub section_id: String,
    pub old_content: String,
    pub new_content: String,
    pub old_status: String,
    pub new_status: String,
}
```

---

## 5. Frontend: Approval UI

### 5.1 Phase Badge

The Furnace header shows the current phase:
```
The Furnace · 丹炉                    [GoalGate — Awaiting Approval]
```

### 5.2 Approval Panel (Goal Gate)

When `phase === GoalGate`, the chat area shows:

```
┌─────────────────────────────────────────────┐
│  🔍 Proposed Goals                          │
│                                             │
│  Goals        [modified]  ▼                 │
│                                             │
│  [Approve & Execute]  [Reject & Redraft]   │
│  [Edit Specs Inline]                        │
└─────────────────────────────────────────────┘
```

In **GSD mode**, this is the ONLY human gate by default. All later stages
(Design, Planning, Execution, Verification) run autonomously.

In **Check mode**, additional gates appear after Architect, Planner, and Reviewer.

### 5.3 Report Panel (Final Delivery)

When the relay completes, the chat area shows the Report:

```
┌─────────────────────────────────────────────┐
│  ✅ Relay Complete — Report                 │
│                                             │
│  Goal: G1 (OAuth2 login) — Done             │
│  Tests: 14/14 passing                       │
│  Confidence: High                           │
│  Cost: $2.51 (84,940 tokens)               │
│                                             │
│  [View Details]  [Download Report]          │
└─────────────────────────────────────────────┘
```

### 5.3 Phase Progress

When `phase === Execution`, the chat shows a sticky progress bar:
```
Executing: 3 / 7 phases completed
[████████░░░░░░░░░░] 43%
```

### 5.4 Order View: Live Pipeline

The Order view visualizes the current session's phase:

```
[Assistant] → [Advisor] → [Architect] → [Planner] → [Coder] → [Tester] → [Reviewer] → [Documenter]
    ✅          🔄          ⏳           ⏳         ⏳         ⏳          ⏳            ⏳
```

- Green = completed
- Yellow = current
- Gray = pending
- Click any phase to see details (token usage, time, notes)

---

## 6. System Prompts per Phase

### Profession: Assistant (Intake)

```markdown
You are the Assistant — the boss's secretary.

Your job is to receive the request, classify it, and route it.

1. **QUESTION** — Answer directly. Stop.
2. **DIRECT** — Route to Coder. Skip spec drafting.
3. **NEW_GOAL** / **REQ_UPDATE** — Route to Advisor.

Rules:
- Ask at most 1-2 clarifying questions if intent is unclear.
- Do NOT brainstorm. Do NOT write specs. Just classify and route.
- Be fast, polite, invisible.

Output format:
**Classification:** [QUESTION | DIRECT | NEW_GOAL | REQ_UPDATE]
**Reasoning:** [one sentence]
**Next step:** [route to X]
```

### Profession: Advisor (Discovery)

```markdown
You are the Advisor — a product consultant.

Your job is to discover what the boss actually wants and write clear Goals.

Rules:
1. Read existing Goals to avoid duplication.
2. Ask focused questions until the requirement is testable.
3. Write Goals as single sentences (≤140 chars).
4. Do NOT design. Do NOT plan execution. Discover intent.
5. Use `write_jade` to draft Goals. Set status to "proposed".
6. When done, present summary and ask: "All clear — should I draft the specs?"
```

### Profession: Architect (Design)

```markdown
You are the Architect.

Your job is to design the system structure, module boundaries, and interfaces.

Rules:
1. Read Goals first. Understand WHAT before HOW.
2. Write Architecture (structural decisions, ADRs) and Designs (module interfaces,
   state machines, data models, REST endpoint contracts).
3. Include Mermaid diagrams for Architecture.
4. Include interface definitions in Designs (function signatures, REST schemas).
5. Do NOT write code. Do NOT write execution plans. Design only.
6. Use `write_jade` for Architecture and Designs. Status = "draft".
```

### Profession: Planner (Planning)

```markdown
You are the Planner.

Your job is to translate Designs into an execution plan with phases.

Rules:
1. Read Goals, Architecture, and Designs.
2. Write Plans as phased implementation (P1.1, P1.2, ...).
3. Each phase must be completable in one focused session.
4. Include Risk and Mitigation for every Plan.
5. Do NOT write code. Plan only.
6. Use `write_jade` for Plans. Status = "draft".
```

### Profession: Coder (Execution)

```markdown
You are the Coder.

Your job is to implement the approved Plans.

Rules:
1. Read Plans and Tests first.
2. Work through phases in ORDER.
3. TDD when possible: write failing test → implement → verify pass.
4. Mark phases complete by updating Plan status.
5. If you hit an architectural problem, STOP. Route back to Architect.
6. Do NOT silently work around spec conflicts.
```

### Profession: Tester (Verification)

```markdown
You are the Tester.

Your job is to verify the implementation.

Rules:
1. Read Tests and run them.
2. If tests fail, hand back to Coder with clear failure context.
3. Max 3 iterations. After that, escalate to Reviewer.
4. Update Test status (Passing / Failing).
```

### Profession: Reviewer (Quality Audit)

```markdown
You are the Reviewer.

Your job is to audit the implementation against Goals.

Rules:
1. Read Goals, Tests, and implemented code.
2. For each goal: was it met? Is there coverage? Any drift?
3. Write Reviews section with findings.
4. Assign confidence score: High / Medium / Low.
5. In GSD mode, your verdict auto-approves delivery.
   In Check mode, the boss reviews your findings.
```

---

## 7. Drift Detection

### 7.1 Automatic Drift Check

After Execution completes:
1. Read Goals section
2. Read implemented code (files mentioned in plan phases)
3. Ask AI: "Does this code satisfy goal G1? G1.1? ..."
4. Flag mismatches as `drift` status on the goal

### 7.2 Manual Drift Check

The "Drift Check" button in SpecsView runs the same check on demand.

---

## 8. Implementation Roadmap

### Phase A: Jades Persistence (1–2 days)
- [ ] Backend: `LedgerStore` with JSON persistence
- [ ] Backend: CRUD API endpoints for ledger sections
- [ ] Frontend: `useLedger.ts` composable
- [ ] Frontend: Wire JadesView to real API

### Phase B: Phase-Aware Agent (2–3 days)
- [ ] Add `ForgePhase` to session model
- [ ] Implement phase transitions in `forge_stream`
- [ ] Add per-phase system prompts
- [ ] Add `read_jade`, `write_jade`, `list_jades` tools
- [ ] Implement Advisor → GoalGate flow

### Phase C: Approval Gate (1–2 days)
- [ ] Frontend: GoalGate UI with diff rendering
- [ ] Backend: `/approve` and `/reject` endpoints
- [ ] Frontend: Approve/Reject/Edit buttons

### Phase D: Order Pipeline (1 day)
- [ ] Backend: expose session phase + progress in API
- [ ] Frontend: live pipeline visualization in OrderView

### Phase E: Drift Detection (1 day)
- [ ] Implement automated drift check after Execution
- [ ] Wire Drift Check button to real check

---

## 9. Open Questions

1. **Direct implementation threshold:** What counts as "small enough" to skip spec drafting?
   - Proposal: <10 lines changed AND no new files AND no behavior changes

2. **TDD enforcement:** Should plan phases always require tests first?
   - Proposal: Configurable per project. Default: enabled for new code, disabled for bug fixes.

3. **Phase granularity:** How detailed should plan phases be?
   - Proposal: Each phase should be completable in one focused session (1-4 hours), with clear deliverables.

4. **Session recovery:** If a session dies mid-Execution, how does it resume?
   - Proposal: Read current Plan — find the first non-Done phase and resume from there.
