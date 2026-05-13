# Spec-Driven Forge Design v2

Inspired by Superpowers (brainstorm → plan → execute → review) and GSD
(file-based state, fresh agents per task, wave execution).

## Core Principle

> **The Jades are the source of truth.** Every decision the AI makes is resolved
> by reading the spec. No code is written before the spec is drafted and approved.

---

## 1. The Four Hard Gates

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  User Request                                                               │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  GATE 1     │───►│  GATE 2     │───►│  GATE 3     │───►│  GATE 4     │ │
│  │  Intake     │    │  SpecDraft  │    │  Approve    │    │  Execute    │ │
│  │  & Classify │    │  & Analyze  │    │  & Review   │    │  & Verify   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘ │
│       │                   │                   │                   │        │
│       │ classify          │ draft jades       │ human approves    │        │
│       │ intent            │ updates           │ or rejects        │        │
│       │                   │                   │                   │        │
│       ▼                   ▼                   ▼                   ▼        │
│   QUESTION ─────────────► answer & stop                                   │
│   DIRECT ───────────────► skip to Gate 4 (small changes only)             │
│   NEW_GOAL ─────────────► full pipeline                                   │
│   REQ_UPDATE ───────────► full pipeline                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Gate 1: Intake & Classify

**What happens:** The AI reads the user's request and classifies intent.

**Intent types:**
- `QUESTION` — "How does auth work?" → answer and stop
- `DIRECT` — "Fix typo on line 42" → skip to Gate 4 (threshold: <10 lines changed)
- `NEW_GOAL` — "Add OAuth2 login" → full pipeline
- `REQ_UPDATE` — "Change timeout to 10 min" → full pipeline

**Output:** Classification + brief reasoning.

### Gate 2: SpecDraft

**What happens:** The AI reads current Jades, drafts updates.

**Process (spec-driven, top-down):**
1. **Read** current Goals, Plans, Architecture, Designs, Tests from Specs
2. **Analyze** impact — what sections need updating?
3. **Draft** updates using `write_spec` tool:
   - Goals — what is the high-level objective?
   - Architecture — structural decisions, component boundaries
   - Designs — module-level interfaces and state machines
   - Plans — phased implementation strategy
   - Tests — executable verification criteria
4. **Present** summary of proposed changes

**Rule:** No `read_file` / `write_file` tools available in this gate. Only Jades tools.

### Gate 3: Approve & Review

**What happens:** Human reviews the proposed Jades changes.

**Frontend UI:**
- Side-by-side diff of each modified Jades section
- Buttons: **[Approve & Execute]** **[Reject & Redraft]** **[Edit Specs]**
- User can also type feedback in chat

**Backend:**
- Session enters `ForgePhase::SpecReview`
- Proposed changes stored in `session.pending_spec_changes`
- On approve: applies changes to Jades, transitions to `Execution`
- On reject: transitions back to `SpecDraft` with feedback
- On edit: user edits inline, then approves

### Gate 4: Execute & Verify

**What happens:** AI implements based on approved specs.

**Process (plan-driven, bottom-up):**
1. **Read** approved Plans and related Tests from Specs
2. **Execute** plan phases in order, marking each complete:
   - Read relevant files
   - Write failing test (TDD mode)
   - Implement minimal code
   - Verify test passes
   - Update Plan phase status → `in_progress` → `done`
3. **Drift check** — compare implementation against Goals (acceptance criteria)
4. **Update** Reports and Reviews sections
5. **Done** — session returns to `Idle`

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

| Phase | read_jade | write_jade | read_file | write_file | shell |
|---|---|---|---|---|---|
| Intake | ✅ | ❌ | ✅ | ❌ | ❌ |
| SpecDraft | ✅ | ✅ | ✅ | ❌ | ❌ |
| SpecReview | ❌ | ❌ | ❌ | ❌ | ❌ |
| Execution | ✅ | ✅ | ✅ | ✅ | ✅ |
| Verification | ✅ | ✅ | ✅ | ❌ | ✅ |

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
    pub pending_spec_changes: Vec<SpecChange>, // NEW: queued during SpecReview
    pub current_phase_index: Option<usize>,    // NEW: which plan phase we're executing
}
```

### 4.2 ForgePhase

```rust
enum ForgePhase {
    Intake,
    SpecDraft,
    SpecReview,
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

## 5. Frontend: SpecReview UI

### 5.1 Phase Badge

The Furnace header shows the current phase:
```
The Furnace · 丹炉                    [SpecReview — Awaiting Approval]
```

### 5.2 Approval Panel

When `phase === SpecReview`, the chat area shows:

```
┌─────────────────────────────────────────────┐
│  🔍 Proposed Jades Updates                  │
│                                             │
│  Goals        [modified]  ▼                 │
│  Architecture [modified]  ▼                 │
│  Plans        [new]       ▼                 │
│  Tests        [new]       ▼                 │
│                                             │
│  [Approve & Execute]  [Reject & Redraft]   │
│  [Edit Specs Inline]                        │
└─────────────────────────────────────────────┘
```

Each section is expandable showing a diff view (old → new).

### 5.3 Phase Progress

When `phase === Execution`, the chat shows a sticky progress bar:
```
Executing: 3 / 7 phases completed
[████████░░░░░░░░░░] 43%
```

### 5.4 Order View: Live Pipeline

The Order view visualizes the current session's phase:

```
[Intake] → [SpecDraft] → [SpecReview] → [Execution] → [Verification]
    ✅         ✅            🔄              ⏳            ⏳
```

- Green = completed
- Yellow = current
- Gray = pending
- Click any phase to see details (token usage, time, notes)

---

## 6. System Prompts per Phase

### Phase: Intake

```markdown
You are AutoSmith Forge, a spec-driven AI coding assistant.

Your FIRST job is to classify the user's request into ONE of these categories:

1. **QUESTION** — User is asking for information, explanation, or advice.
   Action: Answer thoroughly. Do not modify any files or Jades.

2. **DIRECT** — User wants a small, immediate code change (bug fix, refactor,
   typo, <10 lines). Action: Read relevant code, fix it, verify.
   Skip spec drafting.

3. **NEW_GOAL** — User wants a new feature or capability.
   Action: Read current Jades, then proceed to SpecDraft.

4. **REQ_UPDATE** — User wants to change existing behavior or requirements.
   Action: Read current Jades, then proceed to SpecDraft.

Classification rules:
- If the request mentions "add", "implement", "create", "support" → likely NEW_GOAL
- If the request mentions "change", "update", "instead of", "should be" → likely REQ_UPDATE
- If the request mentions "fix", "refactor", "typo", "bug" → likely DIRECT
- If uncertain, ask clarifying questions BEFORE classifying.

Output format:
**Classification:** [QUESTION | DIRECT | NEW_GOAL | REQ_UPDATE]
**Reasoning:** [one sentence explaining why]
**Next step:** [what you will do]
```

### Phase: SpecDraft

```markdown
You are drafting specification updates for the Jades.

Rules:
1. You may ONLY use `read_jade`, `write_jade`, and `list_jades` tools.
   You may NOT read or write source code files in this phase.
2. Read the current Jades first to understand existing specs.
3. Follow top-down spec design:
   a. **Goals** — WHAT we are building (1-3 sentences)
   b. **Architecture** — structural decisions, component boundaries
   c. **Designs** — module-level interfaces and state machines
   d. **Plans** — phased implementation (high-level)
   e. **Tests** — executable verification criteria
4. Each plan phase should follow TDD when possible:
   - Write failing test
   - Implement minimal code
   - Verify test passes
5. Use `write_spec` to update sections. Set status to "draft" for new content.
6. When done, present a summary of changes and wait for approval.
```

### Phase: Execution

```markdown
You are executing an approved plan from the Specs.

Rules:
1. Read the Plans and Tests sections first.
2. Work through plan phases in ORDER. Do not skip ahead.
3. For each phase:
   a. Read the relevant files
   b. Write failing test (if TDD mode is enabled)
   c. Implement the minimal code change
   d. Run tests / verify
   e. Mark phase as complete by updating Specs
4. If you discover a goal conflict, STOP and ask for clarification.
   Do NOT work around it silently.
5. After all phases complete, run the full test suite.
6. Update Reports section with what was done.
```

### Phase: Verification

```markdown
You are verifying the implementation against the Specs goals.

For each goal:
- Was it implemented? (yes / no / partial)
- Is there test coverage? (yes / no)
- Any drift from the spec? (describe)

Update:
- **Reviews** section with findings and recommendations
- **Reports** section with completion summary
- Mark any drifted requirements with status "drift"
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
- [ ] Implement SpecDrafting → SpecReview flow

### Phase C: Approval Gate (1–2 days)
- [ ] Frontend: SpecReview UI with diff rendering
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
