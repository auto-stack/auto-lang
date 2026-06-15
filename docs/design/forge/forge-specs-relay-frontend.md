# Forge Specs Relay — Frontend Design

> **Version:** 1.0  
> **Depends on:** [Spec-Driven Forge Design v2](./spec-driven-forge.md), [Spec UI & Traceability](./spec-ui-and-relations.md), [Spec Categories](./spec-categories.md)  
> **Scope:** AutoForge frontend (`packages/auto-forge-ui`)

---

## 0. Problem Statement

The redesigned specs relay introduces four autonomous phases (Intake → Discovery → SpecDraft → Execute) with human gates, bidirectional traceability between 7 spec categories, and per-category structured rendering. The current frontend has:

- **AgentsView**: A generic pipeline visualizer that treats all steps as equal nodes.
- **SpecsView**: A static document editor with limited relation awareness and no gate integration.
- **No concept of "mode"**: GSD vs Check mode is invisible.
- **No live coupling**: The relay engine and the specs editor do not talk to each other during execution.

We need a unified frontend experience where **the relay engine and the specs workspace are two lenses into the same truth**.

---

## 1. UX Design

### 1.1 Mental Model: "The Forge is a Kitchen, Not a Factory"

| Legacy Mental Model | New Mental Model |
|---|---|
| Agents are black boxes in a conveyor belt | Professions are specialists working from a shared recipe (the Specs) |
| Specs are archived documents | Specs are the **live contract** between human and AI |
| Approval is an afterthought | Gates are **deliberate pause points** where context is surfaced |
| Runs are disposable | Runs are **traced executions** anchored to immutable spec IDs |

**Implication for UX:** The user never "runs an agent." The user **commissions a goal**, and the frontend reveals the kitchen's progress, surfacing the recipe (specs) whenever a chef (profession) needs sign-off.

### 1.2 Core User Flows

#### Flow A: Commission a Goal (Boss → Relay)

```
[Chat Input] ──► Assistant classifies ──► "This needs Discovery"
                                              │
                                              ▼
[Chat] Advisor asks 1–3 questions ◄────► [Boss answers in chat]
                                              │
                                              ▼
[Goal Gate Panel] Proposed Goals appear ──► [Boss approves / rejects / edits]
                                              │
                                              ▼
[Live Pipeline] SpecDraft phase autostarts (Architect → Planner → Tester)
```

**UX decisions:**
- The chat is the **primary interface**. All intake happens there.
- Proposed specs are **injected into the chat stream** as rich cards, not links to another page.
- Approval happens **inline** — the boss does not leave the conversation.

#### Flow B: Approve a Gate (Human-in-the-Loop)

The active chat session is the **boss's office**. All gates, regardless of which session or run generated them, are delivered here by the **Secretary** (a synthetic messenger message). The boss never leaves his chair.

```
[Boss is typing in Chat Session A]
      │
      ▼
┌─ 📋 Secretary ───────────────────────────────────────────┐
│  Boss, the Advisor in session #42 has drafted 3 Goals    │
│  for "OAuth2 login" and is waiting for your approval.    │
│                                                          │
│  ┌─ G1 Add OAuth2 login with PKCE ─────────────────┐    │
│  │ G2 Support refresh token rotation               │    │
│  │ G3 Store tokens in httpOnly cookies             │    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
│  [✓ Approve & Continue]  [✗ Reject]  [✎ Review →]       │
└──────────────────────────────────────────────────────────┘
      │
      ▼
[Boss continues typing in Session A — input is NOT blocked]
```

**If the boss acts:**
- **Approve** → message collapses to green checkmark; pipeline auto-advances.
- **Reject** → message turns amber; reason prompt appears; Advisor reworks.
- **Review →** → right drawer opens with full diff; boss stays in chat.

**If the boss ignores:**
- The secretary message stays pinned above the input bar.
- A red badge appears on the Chat tab: `🔴2` (count of pending gates).
- Additional gates are **queued**; only one secretary message is active at a time.

**UX decisions:**
- **Non-blocking**: The input bar is always usable. The secretary waits, not interrupts.
- **Diff on demand**: Simple gates show inline summary; complex gates open the right drawer.
- **Queue discipline**: Multiple gates are delivered sequentially to avoid cluttering the desk.
- **Escape hatch**: "Review →" opens the Specs drawer without switching views.

#### Flow C: Deep-Dive into Specs (Check Mode / Audit)

```
[Specs Tab] ──► Sidebar: 7 sections with status dots
      │
      ▼
[Goals Table] Click G1 ──► Row expands ──► Relations Panel + Children (A1, D1, S1.1)
      │
      ▼
Click "A1" in Children ──► Section switches to Architecture, A1 auto-expands, yellow flash
      │
      ▼
[Architecture Card] Live Mermaid diagram + ADR content
```

**UX decisions:**
- **Bidirectional jumping is the primary navigation** inside specs. There is no "back button" — only "jump to parent/child."
- **Status is glanceable**: sidebar dots + inline badges tell the health of the whole pyramid.
- **Check mode adds gate banners** at the top of every section: "Awaiting your approval before Coder begins."

#### Flow D: Monitor Execution (Ambient Progress)

The boss does **not** receive chat messages about phase handoffs. Progress is **ambient** — visible only when the boss chooses to look.

```
[Chat Session List Sidebar]
🔴 OAuth2 login          12 msgs   execution   ← pulsing amber dot
🟢 Auth refactor         8 msgs    idle
🟡 FFI bridge            15 msgs   spec_draft  ← pulsing amber dot

[Boss is in Chat Session A, ignoring the other sessions]
      │
      ▼
[Boss gets curious, clicks Session "OAuth2 login"]
      │
      ▼
[Chat shows the original conversation + final Report Card]
[No interim "Architect done" messages clutter the history]
      │
      ▼
[Boss clicks the pulsing dot or switches to Relay tab]
      │
      ▼
[Relay View] Node graph shows exact progress:
   Architect ✅ │ Planner ✅ │ Coder 🔄 (3/7 phases)
```

**UX decisions:**
- **Chat is NOT a progress dashboard.** The chat history contains only the boss's commission, the secretary's gate request, and the final report.
- **Ambient indicators only**: Session list shows pulsing dot + phase label. Agents rail tab shows `🌀 Agents ●2` (count of active relays).
- **Plan phases animate** in the Specs view and Relay view, not in chat.
- **Errors are the only exception**: If the relay hits an unrecoverable error, the secretary delivers a single message: "⚠️ Coder encountered an error. Reviewer is investigating."
- **Specs as instrument panel**: If the boss opens Specs during execution, plan rows pulse and fill in real time. This is opt-in by switching views.

#### Flow E: Receive Final Report

```
[Chat] ──► Report Card appears (rich, collapsed)
  ├─ "✅ Relay Complete — Report X42"
  ├─ Goals: 1/1 │ Tests: 14/14 │ Drift: 1 (Low)
  ├─ Cost: $2.51 │ Confidence: High
  ├─ [View Full Report] [Download Markdown] [Open Changed Files]
```

**UX decisions:**
- The report is the **terminal node** of the conversation. It feels like a receipt.
- One-click access to evidence: tests, reviews, changed files.
- The report auto-saves to the `reports` section of the Specs document.

### 1.3 Information Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  App Shell                                                                  │
│  ├─ Top Bar: Project selector, Mode toggle [GSD │ Check], User              │
│  ├─ Primary Nav: Chat │ Specs │ Relay │ Files                                │
│  └─ Main Area                                                               │
│       ├─ Chat View (default) ──► Conversation + inline gate cards           │
│       ├─ Specs View ──► Sidebar + structured category renderer              │
│       └─ Relay View ──► Pipeline + run history + config                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Nav philosophy:**
- **Chat** is the default. It is where the boss lives.
- **Specs** is the reference. The boss goes there to audit, edit, or explore traceability.
- **Relay** is the diagnostics dashboard. Engineers go there to debug pipeline behavior.

### 1.4 Key UX Principles

| Principle | Rule |
|---|---|
| **Context Preservation** | Every UI element (gate, report, spec card) shows *when* it was created and *by which profession*. |
| **Progressive Disclosure** | GSD mode hides everything except the Goal Gate. Check mode reveals all intermediate gates. |
| **Fail-Safe Defaults** | Approve/Reject buttons are never ambiguous. Reject always asks for a *reason*. |
| **Traceability First** | Every ID (`G1`, `A1`, `P1.3`) is a hyperlink. Hovering shows a preview tooltip. |
| **Chat as Source of Truth** | The chat log is the immutable audit trail. Specs are the mutable working document. |

### 1.5 The Smart Secretary Pattern

> The user never "checks for notifications." The secretary brings the document to the boss's desk.

**Metaphor:** In a Chinese office, the secretary (秘书) runs into the boss's room, places the document on his desk, and waits quietly. The boss can sign immediately, ask to review it carefully, or tell the secretary to wait. The boss never stands up.

**Rules:**
1. **One secretary in the room at a time.** If multiple gates arrive, they form a queue. The next secretary only enters after the current one is resolved or dismissed.
2. **The desk must stay clean.** A resolved secretary message collapses to a compact line (`✅ Goals approved at 14:32`) after 3 seconds.
3. **Simple documents = instant signature.** Goal gates with <5 items get inline Approve/Reject buttons.
4. **Thick contracts = review drawer.** Architecture/Design gates in Check mode get a "Review →" button that opens the right drawer with full diff, Mermaid diagrams, and traceability.
5. **The boss is always busy.** The input bar is never blocked. The secretary message is pinned above it, not replacing it.
6. **The secretary never reports "everything is fine."** She only speaks when the boss must act or when the job is done. Progress between those moments is ambient — visible in the sidebar, not in the chat.

**Anti-patterns to avoid:**
- ❌ Browser alert popups
- ❌ Auto-switching the user to another session
- ❌ Blocking the chat input until the gate is resolved
- ❌ Stacking 5 secretary messages in the chat (use the queue instead)
- ❌ Periodic "status update" messages in the chat stream (the boss doesn't care about phase handoffs unless he asks)

---

## 2. UI Design

### 2.1 Layout System

The frontend uses a **3-zone adaptive layout**:

```
┌─────────────────┬─────────────────────────────┬─────────────────┐
│   Zone A        │        Zone B               │   Zone C        │
│   (Sidebar)     │        (Main)               │   (Context)     │
│                 │                             │                 │
│  Section Nav    │   Chat Stream               │  Live Pipeline  │
│  or Run List    │   or Specs Editor           │  or Relations   │
│                 │                             │  or Gate Detail │
│  220px fixed    │   flex: 1                   │  280–320px      │
│                 │                             │  collapsible    │
└─────────────────┴─────────────────────────────┴─────────────────┘
```

**Zone A** — Navigation anchor. Never hidden on desktop. Collapsible on tablet.
**Zone B** — Primary content. Chat messages, spec tables, or pipeline flow.
**Zone C** — Contextual detail. Dynamically swaps based on selection:
- In Chat: shows current run pipeline + live tokens
- In Specs: shows Relations Panel for selected item
- In Relay: shows step history + profession config

### 2.2 Chat View — The Secretary & Gate Cards

The chat stream contains two kinds of gate UI:

**A. Native Gate Card** (when the gate belongs to the **current** session)

Rendered at the bottom of the chat, replacing the input bar until resolved:

```
┌─ 💡 Advisor drafted 3 Goals ─────────────────────────────┐
│                                                           │
│  G1  Add OAuth2 login with PKCE               [Proposed]  │
│  G2  Support refresh token rotation           [Proposed]  │
│  G3  Store tokens in httpOnly cookies         [Proposed]  │
│                                                           │
│  [View Diff ▼]                                            │
│  ├─ G1: changed "OAuth2" → "OAuth2 with PKCE"            │
│  └─ G3: added (was not in previous version)              │
│                                                           │
│  [✓ Approve & Execute]  [✗ Reject & Redraft]  [✎ Edit]   │
└───────────────────────────────────────────────────────────┘
```

**B. Secretary Message** (when the gate belongs to a **different** session or run)

Rendered as an ephemeral assistant-like message above the input bar. The input remains usable.

```
┌─ 📋 Secretary ───────────────────────────────────────────┐
│  Boss, the Advisor in session #42 has drafted 3 Goals    │
│  for "OAuth2 login" and is waiting for your approval.    │
│                                                           │
│  ┌─ G1 Add OAuth2 login with PKCE ─────────────────┐    │
│  │ G2 Support refresh token rotation               │    │
│  │ G3 Store tokens in httpOnly cookies             │    │
│  └──────────────────────────────────────────────────┘    │
│                                                           │
│  [✓ Approve]  [✗ Reject]  [✎ Review →]  [🔔 Snooze 5m]   │
└───────────────────────────────────────────────────────────┘
```

**States:**
- `idle` — no card
- `proposed` — card/message with diff, all buttons active
- `approved` — collapses to green checkmark line: "✅ Goals approved at 14:32"
- `rejected` — turns amber, reason prompt appears; Advisor reworks
- `snoozed` — message minimizes to a thin amber bar: "⏸️ OAuth2 gate snoozed (4 min left)"
- `resolved` — auto-fades after 3 seconds, leaving a compact audit line

**Secretary Queue UI:**
When gates are queued, a small pill appears on the secretary badge:
```
📋 Secretary  +2 more →
```
Clicking the pill expands a mini-list of queued gates.

### 2.3 Specs View — Unified Workspace

This is an evolution of the current `SpecsView.vue`.

#### 2.3.1 Sidebar (Zone A)

```
🎯 Goals        ●●● 3    │  ● = approved, ○ = draft, ⚠ = drift
🏗️ Architecture ●○   1    │  (status dots = health of items inside)
🎨 Designs      ●●   2    │
📅 Plans        ▓▓░  3    │  ▓ = done, ░ = pending
🧪 Tests        ✓✓✗  4    │  ✓ = passing, ✗ = failing
📝 Reviews      ●    1    │
📊 Reports      ○    0    │
```

Each section row shows:
- **Icon + Title**
- **Mini health bar** (derived from item statuses, 5 dots max)
- **Item count**
- **Active indicator** (left border, same as current)

#### 2.3.2 Main Pane (Zone B) — Category Renderers

Reuses the component map from the current `SpecsView`, but enhanced:

| Section | Renderer | Interaction |
|---|---|---|
| Goals | `GoalsTable` | Inline row edit. No detail pane — the row IS the content. Click ID jumps to children. |
| Architecture | `ArchitectureCards` | Card with Mermaid thumbnail. Expand for full ADR + live diagram. |
| Designs | `DesignCards` | Card with interface signature preview. Expand for state machine + data model tabs. |
| Plans | `PlansTimeline` | Gantt-like table. Expand for risk/mitigation + phase dependency graph. |
| Tests | `TestsCards` | Card with pass/fail badge. Expand for fixture/steps + "Run Test" button. |
| Reviews | `ReviewCards` | Card with criterion counts. Expand for issues table + severity chips. |
| Reports | `ReportDashboard` | Single card with metric bars. Expand for cost breakdown + deliverables list. |

#### 2.3.3 Relations Panel (Zone C)

Always visible when an item is selected.

```
┌─ Relations ─────────────────────────┐
│ G1  Add OAuth2 login...    [Jump]   │
│                                     │
│ ▲ Parents                           │
│ ├── — (Goals are root)              │
│                                     │
│ ▼ Children                          │
│ ├── A1  FFI Bridge Arch    [Jump]   │
│ ├── D1  OAuth2 Module      [Jump]   │
│ ├── S1.1 Happy path test   [Jump]   │
│ └── P1  Implementation     [Jump]   │
│                                     │
│ [Drift Check] [View in Relay]       │
└─────────────────────────────────────┘
```

### 2.4 Relay View — Node-Graph Diagnostics Dashboard

This evolves the current `AgentsView.vue` from a "run launcher" into a **live node-graph diagnostics console**.

#### Why a graph?

The relay is a pipeline of **handoffs** between professions. A flat list hides the handoff logic; a node graph makes it visible. Even though v2 is strictly linear, rendering it as a graph future-proofs the UI for v3 parallel branches.

#### Desktop: Horizontal Node Graph

```
┌─ 🔥 Run #42 — OAuth2 Implementation ──────────────────────────────┐
│ Phase: Execution │ Elapsed: 14m │ Tokens: 62k │ Est. cost: $1.85 │
└───────────────────────────────────────────────────────────────────┘

  ●──────●──────●──────●──────●──────●──────●──────●──────●
 /        \      \      \      \      \      \      \      \
📥        💡      🏗️      📝      🧪      💻      🧪      🔍      📚
12t      3.2k    8.1k    2.5k    2.1k     —       —       —       —
5s       2m      4m↑     ⏳      ⏳      ⏳      ⏳      ⏳      ⏳
Done    Done   Working  Wait   Wait   Wait   Wait   Wait   Wait
```

**Expanded Node Card** (appears below the clicked node):

```
┌─ 🏗️ Architect ─────────────────────────────────────────┐
│ Status: 🔄 Working • Started 4m ago                    │
│                                                        │
│ ├─ Tokens consumed: 8,100 │ $0.24                     │
│ ├─ Specs touched: architecture, designs (2 writes)    │
│ ├─ Current action: write_jade(designs)                │
│ └─ Last tool: read_jade(goals) at 14:32:15            │
│                                                        │
│ [View written specs →]  [View live diff →]             │
└────────────────────────────────────────────────────────┘
```

**Node anatomy:**
- **Dot**: Profession icon, colored by status (pending=gray, working=amber pulse, done=green, error=red)
- **Line**: Connector to next node. Animates with a traveling dot when a handoff is in progress.
- **Label below dot**: Profession name, token burn, elapsed time, status text

#### The Retry Loop Visualized

The `Tester → Coder` retry loop (max 3 iterations) is shown as a **backward edge**:

```
         ┌─────────────────┐
         │   ⚠️ 2 failures │
         ▼                 │
● Coder ──────● Tester(verify)
   ▲             │
   │             │
   └─────────────┘  (Retry 2/3)
```

When active:
- The backward arrow pulses amber
- A badge on the Coder node: `Retry 2/3`
- The Coder node transitions from `Done` back to `Working`

#### Mobile: Vertical Timeline

On narrow screens, the graph collapses to a vertical timeline with inline info cards:

```
│
● 📥 Assistant
│  ✅ Done │ 12 tok │ 5s
│
● 💡 Advisor
│  ✅ Done │ 3.2k tok │ 2m 30s
│  [View Goals →]
│
● 🏗️ Architect
│  🔄 Working │ 8.1k tok │ 4m
│  write_jade(designs)
│  [View Specs →]
│
...
```

#### v3 Future: From Linear Chain to DAG

The node component accepts a DAG structure. In v2 the `edges` array is a simple chain:

```typescript
edges: [
  { from: 'intake', to: 'discover', type: 'forward' },
  { from: 'discover', to: 'design', type: 'forward' },
  { from: 'design', to: 'plan', type: 'forward' },
  // ...
]
```

In v3, parallel execution might produce a true DAG:

```typescript
edges: [
  { from: 'design', to: 'plan', type: 'forward' },
  { from: 'design', to: 'test-draft', type: 'forward' },  // parallel!
  { from: 'plan', to: 'code', type: 'forward' },
  { from: 'test-draft', to: 'code', type: 'forward' },     // join
]
```

The layout engine (dagre or a custom grid solver) positions nodes automatically. The **same card component** renders regardless of graph complexity.

#### Relay View Panels

Below the node graph, three panels provide detail:

```
┌─ Live Log ──────────────────────────────────────────────┐
│ 14:32:03  Architect  write_jade  architecture           │
│ 14:32:15  Architect  write_jade  designs                │
│ 14:33:01  Planner    write_jade  plans                  │
│ 14:33:40  Planner    done ──► handoff to Coder          │
└─────────────────────────────────────────────────────────┘

┌─ Specs Touched ─────────────────────────────────────────┐
│ architecture: 1 write │ designs: 2 writes │ plans: 1    │
└─────────────────────────────────────────────────────────┘

┌─ Cost Breakdown ────────────────────────────────────────┐
│ Advisor  ▓▓░░░░░░ $0.08  │  Architect  ▓▓▓▓▓▓░░ $0.24 │
│ Planner  ▓░░░░░░░░ $0.07 │  Coder      ▓▓▓▓▓▓▓▓▓▓ $1.27│
│ Tester   ▓▓▓▓▓░░░░ $0.70 │  Reviewer   ▓░░░░░░░░░ $0.15│
└─────────────────────────────────────────────────────────┘
```

**New elements:**
- **Node graph**: Visualizes handoffs with animated connectors and status-colored dots.
- **Expanded node cards**: Token burn, specs touched, current action, quick jumps.
- **Retry loops**: Backward edges with iteration counters.
- **Live Log**: Streams tool calls (`read_jade`, `write_jade`, `read_file`) in real time.
- **Specs Touched**: Shows which spec sections were mutated, with click-to-jump.
- **Cost Breakdown**: Per-profession token bars (transparency for the boss).

### 2.5 Gate Approval in Non-Chat Views

When the user is in **Specs** or **Relay** view and a gate arrives, the Smart Secretary adapts:

**Specs View — Section Banner:**
```
┌─ 🔒 Gate Active ─────────────────────────────────────────┐
│  Architect is waiting for approval on A1 (FFI Bridge).   │
│  [View Diff ▼]  [Approve]  [Reject]  [Open in Chat →]    │
└──────────────────────────────────────────────────────────┘
```

**Relay View — Pipeline Gate Panel:**
```
┌─ 🔒 Gate at Architect ───────────────────────────────────┐
│  Architecture A1 drafted. awaiting approval.              │
│  [Review in Specs]  [Approve]  [Reject]                   │
└──────────────────────────────────────────────────────────┘
```

**Bottom Sheet (mobile / compact mode):**
```
┌─────────────────────────────────────────────────────────────┐
│  🔒 Gate: Goal Approval Required              [Dismiss ▲]   │
│                                                             │
│  Advisor is waiting. 3 goals proposed.                      │
│                                                             │
│  [Preview in Chat]  [Open Specs]  [Approve]  [Reject]       │
└─────────────────────────────────────────────────────────────┘
```

- Dismissing does not resolve the gate — it hides the sheet and adds to the queue.
- A persistent **badge on the Chat nav icon** (`🔴N`) reminds the user.
- Clicking the badge switches to Chat and shows the Secretary message.

### 2.6 Component Hierarchy

```
App.vue
├── AppHeader (project, mode toggle, notifications)
├── AppNav (Chat | Specs | Relay | Files)
│   └── GateBadge (🔴 count from useGateInbox)
└── RouterView
    ├── ChatView
    │   ├── ChatMessageList
    │   │   └── ChatMessage (text | tool-call | gate-card | report-card)
    │   ├── SecretaryMessage (ephemeral cross-session gate delivery)
    │   │   └── GateActionBar (approve | reject | review | snooze)
    │   ├── GateCard (inline approval UI for current session)
    │   ├── ReportCard (collapsed/expanded summary)
    │   └── ChatInput
    ├── SpecsView
    │   ├── SectionSidebar
    │   ├── SpecsHeader (search, drift check, rebuild links)
    │   ├── GateBanner (sticky banner when section has active gate)
    │   ├── CategoryRenderer (dynamic: GoalsTable | ArchitectureCards | ...)
    │   │   └── SpecItemRow / SpecItemCard
    │   │       └── SpecItemDetail
    │   │           ├── MarkdownContent (with AutoLinkContent)
    │   │           └── RelationsPanel
    │   └── EditorPane (inline or modal)
    └── RelayView
        ├── RunsSidebar
        ├── PipelinePanel
        │   ├── PipelineFlow (step nodes + connectors)
        │   ├── GatePanel (approval actions for current run)
        │   ├── LiveLog (streaming tool events)
        │   └── SpecsTouched (links to mutated sections)
        └── ConfigSidebar
```

### 2.7 State Management

No global store (Vue 3 Composition API with singleton composables is sufficient).

| Composable | Responsibility |
|---|---|
| `useForge.ts` | Current session, messages, phase, SSE stream |
| `useRelay.ts` | Run lifecycle, gates, pipeline state, professions |
| `useSpecs.ts` | Specs document, CRUD, find/jump helpers |
| `useItemRelations.ts` | Parents/children fetching, enrichment |
| `useStreamingDocument.ts` | Live update of specs while relay is writing |
| `useGateInbox.ts` | **NEW**: Cross-session gate queue, secretary state, badge count |

**`useGateInbox.ts` API:**
```typescript
interface PendingGate {
  id: string              // session_id or run_id
  type: 'session' | 'run'
  gateType: 'goal' | 'spec' | 'review'
  title: string
  profession: string
  since: number
  context: { sessionId?: string; runId?: string; sectionId?: string; itemId?: string }
}

const _pendingGates = ref<PendingGate[]>([])
const _activeSecretary = ref<PendingGate | null>(null)
const count = computed(() => _pendingGates.value.length + (_activeSecretary.value ? 1 : 0))

function registerGate(gate: PendingGate)
function resolveGate(id: string)          // removes after approve/reject
function dismissSecretary()               // moves active to back of queue
function snoozeGate(id: string, minutes: number)
```

**Cross-cutting concerns:**
- When `useRelay.currentRun.waiting_for_gate` changes, `useGateInbox` registers the gate. If the user is in Chat, `SecretaryMessage` renders it; if in Specs/Relay, `GateBanner` or `GatePanel` renders it.
- When `useRelay` receives a `write_jade` event, it invalidates `useSpecs` document cache for that section.
- `useSpecs.findItemById()` is the universal resolver for `SpecLink` clicks.
- Secretary messages are **ephemeral UI chrome** — they are not persisted to the backend message history.

### 2.8 Visual Language

| Element | Treatment |
|---|---|
| **Professions** | Consistent icon + color: Assistant 📥 gray, Advisor 💡 amber, Architect 🏗️ blue, Planner 📝 purple, Coder 💻 green, Tester 🧪 teal, Reviewer 🔍 orange, Documenter 📚 slate |
| **Gates** | Yellow amber border (`hsl(38 90% 50%)`) + lock icon. Non-blocking but attention-grabbing. |
| **Secretary** | Amber left border (`3px solid hsl(38 90% 50%)`), light amber background (`hsl(38 90% 50% / 0.03)`), badge reads `📋 Secretary` instead of `assistant`. Subtle slide-in animation from right. |
| **Status badges** | Existing `StatusBadge.vue` palette extended with `proposed` (amber) and `drift` (red pulse). |
| **Progress** | Segmented bar for plan phases, dot trail for pipeline steps. Never use indeterminate spinners for known-length work. |
| **Diff** | Inline green/red highlighting inside gate cards. Do not open a separate diff view. |
| **Queue pill** | Small rounded badge `+N more` on the secretary header, clickable to expand queued gates list. |

### 2.9 Responsive Behavior

| Breakpoint | Layout |
|---|---|
| `≥1280px` | Full 3-zone layout. Zone C always visible. |
| `1024–1279px` | Zone C collapses to a drawer toggled by a "Details" button. |
| `<1024px` | Single column. Tabs switch between Zone A (nav) and Zone B (content). Zone C becomes a bottom sheet. |

---

## 3. Integration Points

### 3.1 Backend API Surface

Existing endpoints (from `spec-driven-forge.md` and `spec-ui-and-relations.md`) consumed by this design:

```
GET    /api/smith/ledger/{project}
PUT    /api/smith/ledger/{project}
GET    /api/smith/ledger/{project}/{section_id}
PUT    /api/smith/ledger/{project}/{section_id}
POST   /api/smith/ledger/{project}/drift-check
GET    /api/forge/specs/{project}/related/{id}
POST   /api/forge/specs/{project}/rebuild-relations

GET    /api/forge/relay/runs
GET    /api/forge/relay/runs/{run_id}
POST   /api/forge/relay/runs
POST   /api/forge/relay/runs/{run_id}/advance
POST   /api/forge/relay/runs/{run_id}/gate
GET    /api/forge/relay/runs/{run_id}/events  (SSE)
```

### 3.2 Event Map: Relay → Specs UI

| SSE Event | Frontend Action |
|---|---|
| `run_started` | Push system message to chat. Highlight run in Relay view. |
| `step_advanced` | Animate pipeline step. Update profession status in Relay view. **Do NOT push chat messages.** |
| `spec_written` | Invalidate `useSpecs` section. If user is on that section, flash the changed item. |
| `gate_reached` | Register gate in `useGateInbox`. If user is in **Chat** → render `SecretaryMessage`. If in **Specs** → render `GateBanner`. If in **Relay** → render `GatePanel`. Play subtle sound (opt-in). |
| `gate_resolved` | Collapse secretary message / gate card to resolved state. Dequeue next gate. Resume pipeline animation. |
| `handoff_submitted` | Append to Live Log. Highlight handoff arrow in pipeline. |
| `run_completed` | Render `ReportCard` in chat. Update Reports section in Specs. |

### 3.3 Spec Link Routing

All `SpecLink` components use a unified `jumpToItem(id)` function:

1. **If user is in Chat**: Opens Specs view in a **side drawer** (Zone C becomes a 480px drawer) scrolled to the item.
2. **If user is in Specs**: Switches section, expands item, flashes highlight.
3. **If user is in Relay**: Same as Chat — opens Specs drawer.

---

## 4. Implementation Phases

### Phase 1: Chat-First Gates & Smart Secretary (UX foundation)
- Inline `GateCard` component for current-session gates.
- `SecretaryMessage` component for cross-session gate delivery.
- `useGateInbox.ts` composable for gate queue, badge count, and snooze.
- `ReportCard` component for terminal state.
- `useForge` ↔ `useRelay` integration for gate state.

### Phase 2: Live Pipeline (Visual feedback)
- Refactor `AgentsView` → `RelayView` with Live Log and Specs Touched.
- SSE event handlers for `spec_written` and `handoff_submitted`.
- Pipeline step animations.

### Phase 3: Specs Workspace (Reference depth)
- Category renderers with relations panel (reuse existing components, wire `useItemRelations`).
- `AutoLinkContent` for clickable IDs inside markdown.
- Drift Check button with results visualization.

### Phase 4: Mode Awareness (Progressive disclosure)
- GSD / Check mode toggle in AppHeader.
- Conditional gate rendering (show all gates vs Goal Gate only).
- Check mode adds approval banners to Architecture, Designs, Plans sections.

### Phase 5: Polish (Responsive + Accessibility)
- Mobile layout (bottom sheets, tabs).
- Keyboard shortcuts (`Ctrl+1` Chat, `Ctrl+2` Specs, `Ctrl+3` Relay, `Ctrl+K` jump to ID).
- Focus management for gate cards (auto-focus Approve button when gate appears).

---

## 5. Open Questions

1. **Sound design**: Should gate arrivals play a chime? (Proposal: muted by default, opt-in in settings.)
2. **Mobile chat**: On small screens, should the chat overlay the Specs view, or replace it?
3. **Gate escalation**: If a gate sits unresolved for >30 min, should we show a browser notification?
4. **Multi-run**: Can multiple relay runs write to the same specs document simultaneously? (Proposal: no — lock the document to one run at a time.)
5. **Secretary persistence**: If the user refreshes the page, should queued gates be restored from the backend, or are they ephemeral UI state? (Proposal: backend should expose `/api/forge/gates/pending` so the secretary queue survives refresh.)
