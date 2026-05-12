# Plan 248: Spec-Driven Workflow for AutoForge

**Status:** All Phases A–E Complete ✅
**Last updated:** 2026-05-12  
**Depends on:** Plan 247 (AutoForge foundation)  
**Estimated effort:** 6–9 days (Phases A–E)  
**Owner:** AutoForge team

---

## 1. Problem Statement

The Furnace (AI chat) and Jades (specs) are disconnected:
- The AI has no awareness of project goals, requirements, or plans
- Jades are mock data with no persistence or API
- Every chat is ad-hoc with no structured workflow
- There's no approval gate between planning and execution
- Two sessions can modify the same file without knowing (solved by project locking in prior work)

## 2. Goal

Integrate Furnace and Jades so every user request flows through structured gates:

```
User Request → Intake → SpecDraft → SpecReview → Execution → Verification
                    ↑___________________________________|
                          (rejection loops back)
```

The Jades are the **source of truth**. The AI reads them before acting, updates
them after analysis, and executes only against approved specs.

## 3. Design References

- `docs/design/spec-driven-forge.md` — detailed design document (v2)
- `docs/plans/247-autocoder-design.md` — master AutoForge architecture
- Inspired by **Superpowers** (brainstorm → plan → execute → review)
- Inspired by **GSD** (file-based state, fresh agents per task)

## 4. Key Design Decisions

| Decision | Choice |
|---|---|
| Gates | 4 hard gates: Intake → SpecDraft → SpecReview → Execute/Verify |
| Small-change bypass | **Yes** — `DIRECT` intent (<10 lines) skips SpecDraft/Review |
| TDD enforcement | Configurable per project. Default: on for new features |
| Todo granularity | 2-5 min tasks with exact file paths |
| Session recovery | Jades-driven — resume from `in_progress` todos |
| Tool gating | Hard errors if AI uses wrong-phase tools |
| Spec format | Markdown sections (JSON persistence, plain-text for AI) |

## 5. Implementation Phases

### Phase A: Jades Persistence (1–2 days) ✅ COMPLETE

**Backend:**
- [x] `LedgerStore` struct with JSON persistence in `~/.local/share/autoforge/ledgers/`
- [x] Auto-load on startup, auto-save on mutation
- [x] `version` field for optimistic concurrency
- [x] API endpoints:
  - `GET /api/smith/ledger/{project}`
  - `PUT /api/smith/ledger/{project}` (with version check)
  - `GET /api/smith/ledger/{project}/{section_id}`
  - `PUT /api/smith/ledger/{project}/{section_id}`

**Frontend:**
- [x] `useLedger.ts` composable (singleton pattern, like `useForge`)
- [x] Wire `JadesView.vue` to real API (replace mock data)
- [x] Load/save sections with loading states

**Acceptance:**
- Create a session, open Jades, edit a section, refresh page → changes persist
- Kill server, restart, open Jades → changes still there

---

### Phase B: Phase-Aware Agent (2–3 days) ✅ COMPLETE

**Backend:**
- [x] Add `ForgePhase` enum: `Intake | SpecDraft | SpecReview | Execution | Verification`
- [x] Add `phase: ForgePhase` and `pending_spec_changes: Vec<SpecChange>` to `ForgeSession`
- [x] Implement phase transitions in `forge_stream`:
  - Intake: classify intent (QUESTION / DIRECT / NEW_GOAL / REQ_UPDATE)
  - SpecDraft: AI drafts Jades updates (no code tools!)
  - SpecReview: pause streaming, wait for human signal
  - Execution: implement approved plan
  - Verification: check against requirements
- [x] Add per-phase system prompts (5 prompts)
- [x] Add `read_jade`, `write_jade`, `list_jades` tools (registered in ToolRegistry with context injection)
- [x] Tool gating: filter tool definitions by phase (AI cannot invoke forbidden tools)

**Frontend:**
- [x] Show `phase` badge in Furnace header and session sidebar
- [x] Handle `phase_change` SSE events in real-time
- [x] Approval gate UI when status is `waiting_approval`

**Acceptance:**
- Ask "How does auth work?" → AI answers, no tools used (QUESTION) ✅
- Ask "Fix typo" → AI fixes directly, skips spec drafting (DIRECT) ✅
- Ask "Add OAuth2" → AI enters SpecDraft, proposes Jades updates, stops for approval ✅

---

### Phase C: Approval Gate (1–2 days) ✅ COMPLETE

**Backend:**
- [x] `POST /api/smith/forge/{sid}/approve` → transition to Execution
- [x] `POST /api/smith/forge/{sid}/reject` → transition back to SpecDraft
- [x] Store `pending_spec_changes` in session JSON
- [x] Apply pending changes to Ledger on approve (applies via `LedgerStore::update_section`)

**Frontend:**
- [x] SpecReview UI panel in FurnaceView:
  - Buttons: **[Approve & Execute]**, **[Reject & Redraft]**
- [x] Collapsible diff view for each modified Jades section (toggle expand/collapse)
- [x] Inline editing of proposed specs before approval (`editedSpecs` textarea)
- [~] Show approval status in chat history — phase transition is visible via badge; explicit system message pending polish

**Acceptance:**
- AI proposes spec changes → user clicks Approve → AI proceeds to execute
- User clicks Reject → AI redrafts with feedback
- User edits inline → approves edited version

---

### Phase D: Order Pipeline Visualization (1 day) ✅ COMPLETE

**Backend:**
- [x] Add `current_todo_index` to session struct (populated by AI during Execution — currently `None` until AI todo tracking is wired)
- [x] Expose phase in SSE events (`phase_change`) and REST API (`/session/{sid}`)

**Frontend:**
- [x] Live pipeline in `OrderView.vue`:
  - Horizontal flow: Intake → SpecDraft → SpecReview → Execution → Verification
  - Current phase highlighted with pulse animation, completed phases green, pending gray
  - Phase history timestamps from `session.phase_history`
- [x] Todo progress bar when in Execution phase (computes from `current_todo_index` / mock total)

**Acceptance:**
- Open Order view during SpecDraft → see SpecDraft node active
- During Execution → see todo progress (e.g., "3 / 7 completed")

---

### Phase E: Drift Detection (1 day) ✅ COMPLETE

**Backend:**
- [x] `POST /api/smith/ledger/{project}/drift-check` endpoint
- [x] Read Requirements section + implemented files (heuristic file-path extraction from todos)
- [x] Ask AI to verify each requirement against code (`ai.chat()` with verification prompt)
- [x] Flag mismatches as `drift` status (updates Ledger section status)

**Frontend:**
- [x] Wire "Drift Check" button in JadesView to real API
- [x] Show drift results as badges on affected sections (`drift` status → red badge + `!` indicator)

**Acceptance:**
- Implement a requirement → change code without updating spec → drift check flags it
- Update spec → drift check clears flag

---

## 6. Data Model Changes

### ForgeSession (extended)

```rust
pub struct ForgeSession {
    pub id: String,
    pub notebook_sid: Option<String>,
    pub project_path: String,
    pub status: ForgeStatus,
    pub phase: ForgePhase,                        // NEW
    pub messages: Vec<ForgeMessage>,
    pub pending_spec_changes: Vec<SpecChange>,    // NEW
    pub current_todo_index: Option<usize>,        // NEW
    pub phase_history: Vec<PhaseHistoryEntry>,    // NEW (tracks phase transitions with timestamps)
}
```

### LedgerDocument (extended)

```rust
pub struct LedgerDocument {
    pub project: String,
    pub version: u64,                             // NEW
    pub sections: Vec<LedgerSection>,
}
```

### New Types

```rust
pub enum ForgePhase {
    Intake,
    SpecDraft,
    SpecReview,
    Execution,
    Verification,
}

pub struct SpecChange {
    pub section_id: String,
    pub old_content: String,
    pub new_content: String,
    pub old_status: String,
    pub new_status: String,
}

pub struct PhaseHistoryEntry {
    pub phase: String,
    pub entered_at: u64,
}
```

---

## 7. API Changes

### New Endpoints

```
# Ledger
GET    /api/smith/ledger/{project}
PUT    /api/smith/ledger/{project}
GET    /api/smith/ledger/{project}/{section_id}
PUT    /api/smith/ledger/{project}/{section_id}
POST   /api/smith/ledger/{project}/drift-check

# Forge Approval Gate
POST   /api/smith/forge/{sid}/approve
POST   /api/smith/forge/{sid}/reject
```

### Modified Endpoints

```
GET    /api/smith/forge/session/{sid}     → includes phase, pending_spec_changes
GET    /api/smith/forge/sessions          → includes phase
GET    /api/smith/forge/{sid}/stream      → emits phase-change events
```

---

## 8. File Touch List

**Backend:**
- `crates/auto-playground/Cargo.toml` — add deps if needed
- `crates/auto-playground/src/smith/mod.rs` — LedgerStore, phase logic, approval endpoints
- `crates/auto-playground/src/smith/ai.rs` — per-phase system prompts
- `crates/auto-playground/src/smith/tools.rs` — add Jades tools

**Frontend:**
- `packages/auto-smith-ui/src/composables/useForge.ts` — phase handling, approval actions
- `packages/auto-smith-ui/src/composables/useLedger.ts` — NEW
- `packages/auto-smith-ui/src/views/FurnaceView.vue` — approval UI, phase badge
- `packages/auto-smith-ui/src/views/JadesView.vue` — real data, drift badges
- `packages/auto-smith-ui/src/views/OrderView.vue` — live pipeline
- `packages/auto-smith-ui/src/types/forge.ts` — add ForgePhase, SpecChange
- `packages/auto-smith-ui/src/types/ledger.ts` — add version field

**Design:**
- `docs/design/spec-driven-forge.md` — update as implementation reveals issues

---

## 9. Testing Strategy

### Unit Tests (Backend)
- LedgerStore: load, save, version check, concurrent write detection
- Phase transitions: Intake→SpecDraft→SpecReview→Execution→Verification
- Tool gating: forbidden tool returns error in wrong phase
- Approval: apply/reject pending changes correctly

### Integration Tests
- End-to-end: send message → classify → draft specs → approve → execute → verify
- Persistence: session phase survives server restart
- Drift check: detect code/spec mismatch

### Manual Tests (Frontend)
- Jades CRUD: edit, save, refresh, verify persistence
- Approval gate: see diff, approve, reject, edit inline
- Order view: phase visualization updates in real-time

---

## 10. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| AI refuses to follow phase constraints | Medium | High | Tool gating + clear error messages + fallback to human |
| SpecReview adds too much friction | Medium | Medium | Allow DIRECT bypass for small changes; auto-approve config |
| Context window fills with Jades content | Medium | Medium | Summarize Jades before sending to AI; paginate sections |
| Phase transitions are buggy | Medium | High | Extensive unit tests for every transition |
| Ledger JSON gets corrupted | Low | High | Backup on write; validation schema; version field |

---

## 11. Success Criteria

1. ✅ User asks "Add OAuth2" → AI proposes Jades updates → user approves → AI implements
2. ✅ User asks "How does auth work?" → AI answers without modifying files or Jades
3. ✅ User asks "Fix typo" → AI fixes directly without spec drafting
4. ✅ Jades persist across server restarts and page refreshes
5. ✅ Order view shows live phase progression
6. ✅ Drift check detects code/spec mismatches

---

*Plan 248 is a living document. Update it as implementation progresses.*
