# Specs UI & Bidirectional Traceability Design

> **Version:** 1.0  
> **Depends on:** [Spec-Driven Category Design](./spec-categories.md) v1.1

---

## 1. Design Goals

1. **Bidirectional Traceability**: Click any ID (`G1`, `G1.1`, `A1`, `D1`) to jump to its definition and see all related items.
2. **Structured Editing**: Each category has a purpose-built editor — not just a textarea.
3. **Live Relations Panel**: Every item shows its upstream (parents) and downstream (children) in a sidebar.
4. **Content-Aware Rendering**: The same `content: String` field renders differently per category (table, checklist, timeline, diagram).
5. **Auto-Link Discovery**: Parse `content` for ID references (`[G1]`, `depends_on: ["G1.1"]`) and auto-generate bidirectional links.

---

## 2. Enhanced Data Model

### 2.1 Backend: `SpecItem` (enhanced)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecItem {
    pub id: String,           // typed: G1, G1.1, A1, D1, S1.1
    pub title: String,        // one-line summary
    pub content: String,      // markdown body (category-specific template)
    pub status: Status,
    
    // ─── Relations ───────────────────────────────────────────
    #[serde(default)]
    pub depends_on: Vec<String>,   // upstream IDs: ["G1", "A2"]
    #[serde(default)]
    pub related: Vec<String>,      // bidirectional auto-populated
    
    // ─── Category-specific metadata ──────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,  // P0, P1, P2 (Goals)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,  // owner (Plans, Tests)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_file: Option<String>, // file path hint (Tests)
    
    // ─── Timestamps ──────────────────────────────────────────
    pub created_at: u64,
    pub modified_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
}
```

### 2.2 `related` Field: Auto-Populated Bidirectional Links

The `related` field is **not manually edited**. It is computed by the backend whenever a document is saved:

```rust
fn rebuild_relations(doc: &mut SpecsDocument) {
    // Clear all related fields
    for section in &mut doc.sections {
        for item in &mut section.items {
            item.related.clear();
        }
    }
    
    // Build forward + reverse links
    let mut links: HashMap<String, Vec<String>> = HashMap::new();
    
    for section in &doc.sections {
        for item in &section.items {
            for dep in &item.depends_on {
                links.entry(dep.clone()).or_default().push(item.id.clone());
            }
            // Also parse content for [ID] references
            for cap in ID_REF_REGEX.captures_iter(&item.content) {
                let ref_id = cap[1].to_string();
                links.entry(ref_id).or_default().push(item.id.clone());
            }
        }
    }
    
    // Write back
    for section in &mut doc.sections {
        for item in &mut section.items {
            if let Some(refs) = links.get(&item.id) {
                item.related = refs.clone();
            }
        }
    }
}
```

**ID Reference Regex:** `\b([GRAPTVXIS]\d+(?:\.\d+)?)\b`

This means:
- If `A1` has `depends_on: ["G1"]`, then `G1.related` will contain `"A1"`.
- If `S1.1` content contains `[G1]`, then `G1.related` will contain `"S1.1"`.

### 2.3 Frontend: TypeScript Types

```typescript
export interface SpecItem {
  id: string
  title: string
  content: string
  status: Status
  depends_on?: string[]
  related?: string[]      // ← NEW: auto-populated backlinks
  priority?: string       // ← NEW
  assignee?: string
  test_file?: string      // ← NEW
  created_at: number
  modified_at: number
  completed_at?: number
}

export interface SpecsSection {
  id: string
  section_type: SectionType
  title: string
  items: SpecItem[]
  content: string          // legacy: section-level free text
  depends_on?: string[]
  last_modified: number
  last_verified?: number
}
```

---

## 3. Per-Category UI Design

### 3.1 Design Principles

1. **List View** (collapsed): Shows ID + Title + Status badge + quick actions. One line per item.
2. **Detail View** (expanded): Shows full content + Relations panel + Status transitions + Edit button.
3. **Edit Mode**: Category-specific structured form. Not a raw textarea.
4. **Relations Panel**: Always visible in detail view. Shows "Parents" (depends_on) and "Children" (related).

### 3.2 Common Components

```
┌─────────────────────────────────────────────────────────────┐
│  Sidebar                    │  Main Pane                     │
│  ─────────                  │  ─────────                     │
│  🎯 Goals (3)               │  ┌─ Section Header ─────────┐ │
│  🏗️ Architecture (1)        │  │ 🎯 Goals        [Edit]   │ │
│  🎨 Designs (2)    ←active  │  │ Status: Approved         │ │
│  📅 Plans (1)               │  └──────────────────────────┘ │
│  🧪 Tests (4)               │                                │
│  📝 Reviews (1)             │  ┌─ Item List ──────────────┐ │
│  📊 Reports (1)             │  │ ┌─ Item Row ───────────┐ │ │
│  🔌 APIs (0)                │  │ │ G1  Goal text    [▶] │ │ │
│                             │  │ └──────────────────────┘ │ │
│                             │  │ ┌─ Expanded Detail ────┐ │ │
│                             │  │ │ Relations Panel      │ │ │
│                             │  │ │ ├─ Parents: —        │ │ │
│                             │  │ │ ├─ Children: A1, D1  │ │ │
│                             │  │ │ ├─ Content (rendered)│ │ │
│                             │  │ │ ├─ Status: [Done ▼]  │ │ │
│                             │  │ │ └─ [Edit] [Delete]   │ │ │
│                             │  │ └──────────────────────┘ │ │
│                             │  └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Goals 🎯 — Table View

**List View:** Rendered as a **table**, not cards.

```
┌──────┬──────────────────────────────┬──────────┬──────────┬──────────┐
│ ID   │ Goal                         │ Priority │ Status   │ Children │
├──────┼──────────────────────────────┼──────────┼──────────┼──────────┤
│ G1   │ AutoVM can dynamically load… │ P0       │ Done     │ A1, D1 (3) │
│ G2   │ Self-hosted compiler front…  │ P0       │ Approved │ A2, D2 (2) │
│ G3   │ Spec-driven serial agent UI  │ P1       │ Draft    │ —        │
└──────┴──────────────────────────────┴──────────┴──────────┴──────────┘
```

**Detail View (expanded row):**
- Relations Panel: Children list (Architecture / Designs that reference this Goal)
- Content: Not shown (Goals have no body — the table row IS the content)
- Actions: Status dropdown, Delete

**Edit Mode:** Inline row editing. Click cell to edit. No separate editor pane.

### 3.4 Architecture 🏗️ — ADR View

**List View:** Cards with diagram preview (Mermaid rendering if available).

```
┌─ A1 FFI Bridge Architecture ───────────────────────────┐
│ Status: Approved  │  Children: D1, I1                   │
│                                                        │
│ [Mermaid diagram thumbnail]                            │
│ Decision: Use cdylib + libloading…                     │
└────────────────────────────────────────────────────────┘
```

**Detail View:**
- Relations Panel: Parents (Goals), Children (Designs, APIs)
- Content: Full markdown with **live Mermaid rendering**
- Actions: Status dropdown, Edit

**Edit Mode:** Split pane:
- Left: Markdown textarea
- Right: Live preview (Mermaid diagrams render in real-time)

### 3.5 Designs 🎨 — Spec View

**List View:** Cards with interface signature preview.

```
┌─ D1 Sandbox.compile_dep() ─────────────────────────────┐
│ Status: Approved  │  Children: P1                       │
│ Module: auto-cache/src/sandbox.rs                      │
│                                                        │
│ pub fn compile_dep(&self, dep: &DepStmt)               │
│   → Result<PathBuf, CompileError>                      │
└────────────────────────────────────────────────────────┘
```

**Detail View:**
- Relations Panel: Parents (Architecture), Children (Plans, Tests)
- Content: Interface + State Machine + Data Model + Pseudocode
- Actions: Status dropdown, Edit

**Edit Mode:** Structured form with tabs:
- Tab 1: Interface (code block textarea)
- Tab 2: State Machine (Mermaid textarea)
- Tab 3: Data Model (table editor)
- Tab 4: Pseudocode (textarea)

### 3.6 Plans 📅 — Timeline View

**List View:** Gantt-like table.

```
┌─ P1 FFI Pipeline Implementation ───────────────────────┐
│ Status: Done  │  Children: S1.1, S1.2                 │
│                                                        │
│ Phase  │ Task                    │ Owner │ Dur │ Dep │ Status │
│ P1.1   │ Sandbox wrapper gen     │ Alice │ 3d  │ D1  │ Done   │
│ P1.2   │ cargo build integration │ Alice │ 2d  │ P1.1│ Done   │
│ P1.3   │ RustFfiBridge register  │ Bob   │ 3d  │ P1.2│ Done   │
└────────────────────────────────────────────────────────┘
```

**Detail View:**
- Relations Panel: Parents (Goals, Designs), Children (Tests)
- Content: Phase table + Risk/Mitigation text
- Actions: Status dropdown, Edit

**Edit Mode:** Table editor for phases. Each row is editable inline. Add/remove rows.

### 3.7 Tests 🧪 — Test Runner View

**List View:** Cards with pass/fail indicator.

```
┌─ S1.1 [G1] cdylib happy path ─────────┐  ┌─ S1.2 [G1] unknown crate ─────────┐
│ ✓ Passing                                 │  │ ✗ Failing                             │
│ Type: Integration                         │  │ Type: Integration                     │
│ File: tests/sandbox_compile_dep.rs        │  │ File: tests/sandbox_compile_dep.rs    │
└───────────────────────────────────────────┘  └───────────────────────────────────────┘
```

**Detail View:**
- Relations Panel: Parents (Goals, Designs), Children (Reviews)
- Content: Fixture + Steps + Expected Outcome
- Actions: Status dropdown, Run Test (shell button), Edit

**Edit Mode:** Structured form:
1. Type (select: Unit/Integration/E2E/Contract/Performance/Fuzz)
2. Fixture (code textarea)
3. Steps (numbered list editor)
4. Expected Outcome (textarea)
5. Test File Path (text input with file picker)

### 3.8 Reviews 📝 — Assessment View

**List View:** Summary card with criterion counts.

```
┌─ V1 Post-Implementation Review — G1 ─────────────────┐
│ Status: Published  │  4/4 passed, 1 issue              │
│                                                        │
│ C1 ☑  C2 ☑  C3 ☑  C4 ⚠                                │
│ Issues: V1-I1 (Low) Windows path separator drift       │
└────────────────────────────────────────────────────────┘
```

**Detail View:**
- Relations Panel: Parents (Goals, Tests), Children (Plans for fixes, Reports)
- Content: Criterion table + Issues
- Actions: Status dropdown, Edit

**Edit Mode:** Table editor for criteria. Issue sub-form with severity/select.

### 3.9 Reports 📊 — Dashboard View

**List View:** Single card with metric summary.

```
┌─ X2026-W20 Weekly Status ──────────────────────────────┐
│ Status: Published                                      │
│                                                        │
│ Goals: 1/3  ▓▓▓▓▓▓░░░   Tests: 4/6  ▓▓▓▓▓▓▓▓░░        │
│ Plans: 1/2  ▓▓▓▓▓▓▓▓░░   Reviews: 2/3 ▓▓▓▓▓▓▓▓▓▓▓░  │
│ Blockers: 1  Risks: 1                                  │
└────────────────────────────────────────────────────────┘
```

**Detail View:**
- Relations Panel: All categories (aggregator)
- Content: Full report markdown
- Actions: Status dropdown, Edit

**Edit Mode:** Markdown textarea with metric auto-completion (typing `@` suggests metrics).



## 4. Relations Panel (Bidirectional Traceability)

Every expanded item shows a **Relations Panel**:

```
┌─ Relations ──────────────────────────┐
│                                      │
│  ▲ Parents (depends_on)              │
│  ├── G1  Goal text...        [Jump] │
│  └── A1  Architecture desc... [Jump] │
│                                      │
│  ▼ Children (related)                │
│  ├── A1   Arch text...       [Jump] │
│  ├── S1.1  Test text...      [Jump] │
│  └── D1   Design text...     [Jump] │
│                                      │
└──────────────────────────────────────┘
```

**Behavior:**
- **Parents**: IDs from `depends_on` + IDs parsed from content (`[G1]` references).
- **Children**: IDs from `related` (auto-populated by backend).
- **Jump Button**: Click navigates to the target section and scrolls to the item.
- **Highlight**: When jumping, the target item flashes briefly (yellow background fade).

### 4.1 Auto-Link Discovery in Content

When rendering any item's content, scan for ID references and turn them into clickable links:

```markdown
This plan implements [G1] via [G1.1] and [A1].
```

Rendered as:
```html
This plan implements <a class="spec-link" data-id="G1">G1</a> 
via <a class="spec-link" data-id="G1.1">G1.1</a> and 
<a class="spec-link" data-id="A1">A1</a>.
```

Clicking a link:
1. Finds the section containing that ID
2. Switches to that section
3. Expands the target item
4. Scrolls it into view

---

## 5. Editor Architecture

### 5.1 Edit Mode Toggle

Each section header has an **Edit** button. Clicking enters edit mode for the entire section:

```
┌─ Section Header ─────────────────┐
│ 🎯 Goals              [Preview]  │  ← toggles edit/view
│                                  │
│ [+ Add Goal]                     │  ← section-level action
│                                  │
│ ┌─ Goal Row (inline edit) ─────┐ │
│ │ ID: G1 (locked)              │ │
│ │ Goal: [AutoVM can dynamically │ │
│ │        load external Rust... ] │ │
│ │ Priority: [P0 ▼]             │ │
│ │ Status: [Approved ▼]         │ │
│ │ [💾] [🗑️]                    │ │
│ └──────────────────────────────┘ │
└──────────────────────────────────┘
```

### 5.2 Two Editing Modes

| Mode | When Used | UI |
|---|---|---|
| **Structured** | Goals, Tests | Form fields per category template |
| **Free Markdown** | Architecture, Designs, Plans, Reviews, Reports | Markdown textarea + live preview |

The backend always stores Markdown. Structured editors serialize to Markdown on save.

### 5.3 Inline vs Full Editor

- **Inline**: Edit one item at a time within the list. Fast for small tweaks.
- **Full**: Open a modal/pane with the complete section editor. For bulk edits.

Default: Inline for Goals, Full for Architecture/Reports.

---

## 6. State Transition UI

Every item has a **status dropdown** in its detail view:

```
Status: [Done ▼]
        ├─ Draft
        ├─ UnderReview
        ├─ Approved
        ├─ InProgress
        ├─ Implemented
        ├─ Done
        └─ Archived
```

**Rules:**
- Only statuses from `SectionConfig.allowed_statuses` are shown.
- Disabled statuses are grayed out with tooltip: "Cannot transition from Done to Draft".
- Changing status auto-saves the item.

---

## 7. Search & Filter

The search box in the header supports:

| Query | Matches |
|---|---|
| `G1` | Item with ID G1 |
| `G1.*` | All sub-goals under Goal 1 |
| `status:done` | All done items across all sections |
| `assignee:alice` | All items assigned to Alice |
| `priority:P0` | All P0 items |
| `cdylib` | Full-text search across titles and content |

---

## 8. API Changes

### 8.1 New Endpoints

```
GET  /api/forge/specs/{project}/related/{id}
  → { "parents": [...], "children": [...] }

POST /api/forge/specs/{project}/rebuild-relations
  → Re-runs relation discovery on the entire document
```

### 8.2 Enhanced Endpoints

```
PUT  /api/forge/specs/{project}/{section_id}
  Body: { "items": [...], "content": "..." }
  → On save, backend auto-runs rebuild_relations()
```

---

## 9. Implementation Priority

| Phase | Feature | Effort |
|---|---|---|
| P1 | Backend: `related` field + `rebuild_relations()` | 2h |
| P2 | Backend: New API endpoints (`/related/{id}`, `/rebuild-relations`) | 1h |
| P3 | Frontend: Relations Panel component | 3h |
| P4 | Frontend: Auto-link discovery in Markdown render | 2h |
| P5 | Frontend: Category-specific renderers (Goals table, Tests cards, etc.) | 6h |
| P6 | Frontend: Structured editors (Goals form, Tests form, etc.) | 6h |
| P7 | Frontend: Phase progress visualization | 3h |
| P8 | Frontend: Mermaid live rendering in Architecture/Designs | 2h |
| P9 | Frontend: Search enhancements | 2h |
| P10 | Integration: End-to-end test | 2h |
