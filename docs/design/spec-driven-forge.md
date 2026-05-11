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
1. **Read** current Goals, Requirements, Plans from Jades
2. **Analyze** impact — what sections need updating?
3. **Draft** updates using `write_jade` tool:
   - Goals — what is the high-level objective?
   - Requirements — specific, testable acceptance criteria
   - Analysis — technical approach, trade-offs, risks
   - Plans — phased implementation strategy
   - Todos — actionable tasks (2-5 minutes each, with exact file paths)
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
1. **Read** approved Plans and Todos from Jades
2. **Execute** todos in order, marking each complete:
   - Read relevant files
   - Write failing test (TDD mode)
   - Implement minimal code
   - Verify test passes
   - Update Jades todo status → `in_progress` → `verified`
3. **Drift check** — compare implementation against Requirements
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
│           {"id": "requirements", "title": "📐 Requirements", "status": "approved", "content": "..."},
│           {"id": "analysis", "title": "🔍 Analysis", "status": "draft", "content": "..."},
│           {"id": "plans", "title": "📅 Plans", "status": "approved", "content": "..."},
│           {"id": "todos", "title": "✅ Todos", "status": "in_progress", "content": "..."},
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
{"name": "read_jade", "arguments": {"section_id": "requirements"}}
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
    pub current_todo_index: Option<usize>,     // NEW: which todo we're executing
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
│  Requirements [modified]  ▼                 │
│  Plans        [new]       ▼                 │
│  Todos        [new]       ▼                 │
│                                             │
│  [Approve & Execute]  [Reject & Redraft]   │
│  [Edit Specs Inline]                        │
└─────────────────────────────────────────────┘
```

Each section is expandable showing a diff view (old → new).

### 5.3 Todo Progress

When `phase === Execution`, the chat shows a sticky progress bar:
```
Executing: 3 / 7 todos completed
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
   b. **Requirements** — acceptance criteria (testable, specific)
   c. **Analysis** — technical approach, trade-offs, risks
   d. **Plans** — phased implementation (high-level)
   e. **Todos** — actionable tasks (2-5 min each, include exact file paths)
4. Each todo should follow TDD when possible:
   - Write failing test
   - Implement minimal code
   - Verify test passes
5. Use `write_jade` to update sections. Set status to "draft" for new content.
6. When done, present a summary of changes and wait for approval.
```

### Phase: Execution

```markdown
You are executing an approved plan from the Jades.

Rules:
1. Read the Plans and Todos sections first.
2. Work through todos in ORDER. Do not skip ahead.
3. For each todo:
   a. Read the relevant files
   b. Write failing test (if TDD mode is enabled)
   c. Implement the minimal code change
   d. Run tests / verify
   e. Mark todo as complete by updating Jades
4. If you discover a requirement conflict, STOP and ask for clarification.
   Do NOT work around it silently.
5. After all todos complete, run the full test suite.
6. Update Reports section with what was done.
```

### Phase: Verification

```markdown
You are verifying the implementation against the Jades requirements.

For each requirement:
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
1. Read Requirements section
2. Read implemented code (files mentioned in todos)
3. Ask AI: "Does this code satisfy requirement R1.1? R1.2? ..."
4. Flag mismatches as `drift` status on the requirement

### 7.2 Manual Drift Check

The "Drift Check" button in JadesView runs the same check on demand.

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

2. **TDD enforcement:** Should todos always require tests first?
   - Proposal: Configurable per project. Default: enabled for new code, disabled for bug fixes.

3. **Todo granularity:** How detailed should todos be?
   - Proposal: 2-5 minutes each, with exact file paths and expected outcome.

4. **Session recovery:** If a session dies mid-Execution, how does it resume?
   - Proposal: Read Jades todos — any todo with status `in_progress` or `pending` is resumed.
