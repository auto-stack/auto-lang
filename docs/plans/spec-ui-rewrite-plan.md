# Execution Plan: Specs UI Rewrite & Bidirectional Traceability

> **Goal:** Rewrite AutoForge's Specs (Jades) pages with per-category structured rendering, bidirectional traceability, and enhanced editing.
> **Depends on:** [Spec UI & Traceability Design](../design/spec-ui-and-relations.md)

---

## Phase 1: Backend Foundation (P1–P2)

### P1.1 Enhance `SpecItem` with `related` and `priority`

**File:** `crates/auto-forge/src/forge/mod.rs`

- Add `related: Vec<String>` field to `SpecItem`
- Add `priority: Option<String>` field to `SpecItem`
- Add `test_file: Option<String>` field to `SpecItem`
- Ensure all fields have `#[serde(default)]` for backward compatibility

### P1.2 Implement `rebuild_relations()`

**File:** `crates/auto-forge/src/forge/mod.rs`

- Add `ID_REF_REGEX: Regex = Regex::new(r"\b([GRAPTVXIS]\d+(?:\.\d+)?)\b").unwrap()`
- Implement `rebuild_relations(doc: &mut SpecsDocument)`
  - Scans all items' `depends_on` + content for ID references
  - Populates `related` field on each item
- Call `rebuild_relations()` inside `save_document()` and `put_specs()`

### P1.3 Add missing default sections

**File:** `crates/auto-forge/src/forge/mod.rs`

- Add `requirements` and `todos` to `default_specs()` (currently missing!)
- Add templates for them if not present

### P1.4 Add `/related/{id}` endpoint

**File:** `crates/auto-forge/src/forge/mod.rs`

- `GET /api/forge/specs/{project}/related/{id}`
- Returns `{ "parents": [...], "children": [...] }`
- Parents = item's own `depends_on`
- Children = items whose `related` contains `id`

### P1.5 Add `/rebuild-relations` endpoint

**File:** `crates/auto-forge/src/forge/mod.rs`

- `POST /api/forge/specs/{project}/rebuild-relations`
- Forces full relation rebuild and returns updated document

---

## Phase 2: Frontend Types & Composables (P3)

### P2.1 Update TypeScript types

**File:** `packages/auto-forge-ui/src/types/specs.ts`

- Add `related?: string[]` to `SpecItem`
- Add `priority?: string` to `SpecItem`
- Add `test_file?: string` to `SpecItem`

### P2.2 Enhance `useSpecs` composable

**File:** `packages/auto-forge-ui/src/composables/useSpecs.ts`

- Add `loadRelated(project, id)` function
- Add `rebuildRelations(project)` function
- Add `findItemById(id)` helper — searches all sections for an item by ID
- Add `jumpToItem(id)` helper — switches section, expands item, scrolls to it

### P2.3 Add `useItemRelations` composable

**File:** `packages/auto-forge-ui/src/composables/useItemRelations.ts`

- `const { parents, children, loading } = useItemRelations(itemId)`
- Fetches from `/api/forge/specs/{project}/related/{id}`
- Returns enriched objects: `{ id, title, section_type, status }[]`

---

## Phase 3: Shared Components (P4)

### P3.1 `StatusBadge` component

**File:** `packages/auto-forge-ui/src/components/StatusBadge.vue`

- Props: `status: Status`, `size: 'sm' | 'md'`
- Renders colored pill with status text
- Used everywhere

### P3.2 `SpecLink` component

**File:** `packages/auto-forge-ui/src/components/SpecLink.vue`

- Props: `id: string`
- Renders clickable ID badge (e.g., `G1`, `R1.2`)
- Click emits `jump(id)` event
- Style: monospace font, subtle underline on hover

### P3.3 `RelationsPanel` component

**File:** `packages/auto-forge-ui/src/components/RelationsPanel.vue`

- Props: `item: SpecItem`
- Shows Parents (depends_on) and Children (related) lists
- Each entry shows ID + title + status badge
- Click entry → emits `jump(id)`
- Uses `useItemRelations`

### P3.4 `AutoLinkContent` component

**File:** `packages/auto-forge-ui/src/components/AutoLinkContent.vue`

- Props: `content: string`
- Renders markdown text with ID references turned into `SpecLink` components
- Regex scan for `\b([GRAPTVXIS]\d+(?:\.\d+)?)\b`
- Non-ID text rendered as plain text

---

## Phase 4: Category-Specific Renderers (P5)

### P4.1 `GoalsTable` component

**File:** `packages/auto-forge-ui/src/components/category/GoalsTable.vue`

- Renders items as a `<table>`
- Columns: ID, Goal (title), Priority, Status, Children count
- Inline editing: click cell → input → save
- No detail expansion (table row IS the content)

### P4.2 `RequirementsCards` component

**File:** `packages/auto-forge-ui/src/components/category/RequirementsCards.vue`

- Card layout per item
- Parses content for checklist (`- [ ]` / `- [x]`) and renders interactive checkboxes
- Shows Acceptance Criteria preview (first 3 items)
- Expanded detail: full content + RelationsPanel

### P4.3 `ArchitectureView` component

**File:** `packages/auto-forge-ui/src/components/category/ArchitectureView.vue`

- Card layout with Mermaid diagram rendering
- Uses `markstream-vue`'s Mermaid support or `mermaid` directly
- Expanded detail: full markdown + RelationsPanel

### P4.4 `DesignsView` component

**File:** `packages/auto-forge-ui/src/components/category/DesignsView.vue`

- Card layout with interface signature preview
- Extracts code blocks from content for preview
- Expanded detail: full content + RelationsPanel

### P4.5 `PlansTimeline` component

**File:** `packages/auto-forge-ui/src/components/category/PlansTimeline.vue`

- Attempts to parse phase table from markdown content
- Falls back to card list if no table found
- Expanded detail: full content + RelationsPanel

### P4.6 `TodosKanban` component

**File:** `packages/auto-forge-ui/src/components/category/TodosKanban.vue`

- Kanban board: columns = allowed statuses for Todos
- Cards draggable between columns (changes status)
- Each card: checkbox + title + status badge

### P4.7 `TestsCards` component

**File:** `packages/auto-forge-ui/src/components/category/TestsCards.vue`

- Card layout with pass/fail indicator
- Status = done/verified → green ✓, blocked → red ✗, else → gray ○
- Shows test_file path
- Expanded detail: full content + RelationsPanel

### P4.8 `ReviewsCards` component

**File:** `packages/auto-forge-ui/src/components/category/ReviewsCards.vue`

- Card layout with criterion summary
- Parses criterion table from content
- Shows issue count

### P4.9 `ReportsDashboard` component

**File:** `packages/auto-forge-ui/src/components/category/ReportsDashboard.vue`

- Single card with metric bars
- Parses metrics table from content
- Shows blockers/risks count

### P4.10 `ApisSchema` component

**File:** `packages/auto-forge-ui/src/components/category/ApisSchema.vue`

- Card layout with endpoint list
- Parses endpoint sections from markdown
- Expanded detail: full schema

---

## Phase 5: JadesView.vue Rewrite (P6)

### P5.1 Restructure layout

**File:** `packages/auto-forge-ui/src/views/JadesView.vue`

- Keep sidebar (section nav)
- Main pane uses dynamic component based on `section_type`
- Remove the current `v-if` chain for category rendering
- Use a component map:
  ```typescript
  const categoryComponents: Record<SectionType, Component> = {
    goals: GoalsTable,
    requirements: RequirementsCards,
    architecture: ArchitectureView,
    designs: DesignsView,
    plans: PlansTimeline,
    todos: TodosKanban,
    tests: TestsCards,
    reviews: ReviewsCards,
    reports: ReportsDashboard,
    apis: ApisSchema,
  }
  ```

### P5.2 Add jump-to-item logic

- `jumpToItem(id: string)` finds section, switches to it, expands item
- Uses `useSpecs.findItemById()`
- Global event bus or provide/inject for `jumpToItem`

### P5.3 Add search enhancements

- Parse search query for `status:`, `assignee:`, `priority:` filters
- Full-text search across titles and content

---

## Phase 6: Structured Editors (P7)

### P6.1 `GoalsInlineEditor`

- Inline row editing within the table
- Fields: title (text), priority (select), status (select)

### P6.2 `RequirementsFormEditor`

- Modal/pane editor
- Fields: title, parent_goals (tag input), criteria (dynamic list), details (textarea)
- Serializes to markdown template on save

### P6.3 `TodosInlineEditor`

- Simple inline form: title + file path + status

### P6.4 `TestsFormEditor`

- Modal editor with tabs: Type, Fixture, Steps, Expected Outcome, Test File

---

## Phase 7: Polish & Integration (P8–P10)

### P7.1 Fix missing default sections in frontend

**File:** `packages/auto-forge-ui/src/views/JadesView.vue`
- Add `requirements` and `todos` to `DEFAULT_SECTIONS`

### P7.2 Build verification

- `cargo check -p auto-forge`
- `npm run build` in frontend
- Manual UI smoke test

### P7.3 Update templates

- Ensure all `.ad` templates match the new category formats

---

## File Touch List

### Backend
- `crates/auto-forge/src/forge/mod.rs` — data model, store, handlers, relations

### Frontend
- `packages/auto-forge-ui/src/types/specs.ts`
- `packages/auto-forge-ui/src/composables/useSpecs.ts`
- `packages/auto-forge-ui/src/composables/useItemRelations.ts` (new)
- `packages/auto-forge-ui/src/components/StatusBadge.vue` (new)
- `packages/auto-forge-ui/src/components/SpecLink.vue` (new)
- `packages/auto-forge-ui/src/components/RelationsPanel.vue` (new)
- `packages/auto-forge-ui/src/components/AutoLinkContent.vue` (new)
- `packages/auto-forge-ui/src/components/category/GoalsTable.vue` (new)
- `packages/auto-forge-ui/src/components/category/RequirementsCards.vue` (new)
- `packages/auto-forge-ui/src/components/category/ArchitectureView.vue` (new)
- `packages/auto-forge-ui/src/components/category/DesignsView.vue` (new)
- `packages/auto-forge-ui/src/components/category/PlansTimeline.vue` (new)
- `packages/auto-forge-ui/src/components/category/TodosKanban.vue` (new)
- `packages/auto-forge-ui/src/components/category/TestsCards.vue` (new)
- `packages/auto-forge-ui/src/components/category/ReviewsCards.vue` (new)
- `packages/auto-forge-ui/src/components/category/ReportsDashboard.vue` (new)
- `packages/auto-forge-ui/src/components/category/ApisSchema.vue` (new)
- `packages/auto-forge-ui/src/views/JadesView.vue` — major rewrite

### Templates
- `crates/auto-forge/src/forge/templates/requirements.ad` (new)
- `crates/auto-forge/src/forge/templates/todos.ad` (new)
- `crates/auto-forge/src/forge/templates/tests.ad` (already exists)
