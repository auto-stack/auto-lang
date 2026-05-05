# Plan 236: Pixel-Perfect a2ui Replica

**Goal**: Make a3ui-replica visually identical to https://a2ui-composer.ag-ui.com/
**Strategy**: Hide custom widgets, replicate original demo widgets exactly, match all styles.

---

## Phase 0: Global Foundation

### 0.1 Background Gradient
- Replace `bg-slate-100` with original gradient in `app.at`
- Original gradient: soft purple→pink→yellow diagonal gradient

### 0.2 Sidebar Polish
- Add logo icon (purple settings/gear icon) next to "A2UI COMPOSER"
- Add active state styling (`bg-white rounded-lg shadow-sm` for active item)
- Change "WIDGETS" → "Widgets"

### 0.3 Router Mode
- Change generator from `createWebHashHistory` to `createWebHistory`
- File: `auto-man/src/vue.rs` router template

---

## Phase 1: Create Page (`/`)

### 1.1 Textarea Shape
- `rounded-xl` → `rounded-full` (pill shape)
- Softer border: `border-slate-200`
- Height: single-line appearance with vertical centering

### 1.2 Create Button
- Move inside textarea as `absolute right-2 top-1/2 -translate-y-1/2`
- Style: `bg-white text-slate-600 border border-slate-200 rounded-md px-4 py-1.5 text-sm`
- Remove purple standalone button

### 1.3 "Powered by CopilotKit"
- Add sparkle: `"Powered by ✨ CopilotKit"`

### 1.4 "or Start Blank"
- Remove dark pill button
- Use plain text: `text-sm text-violet-600 hover:underline cursor-pointer`

---

## Phase 2: Gallery Page (`/gallery`)

### 2.1 Replace Widget Dataset
Replace all 6 custom widgets with original a2ui gallery widgets:

1. **Flight Status** (keep, enrich with more realistic data)
2. **Chat Message** (NEW - chat UI with avatars)
3. **Recipe Card** (NEW - image, tabs, star rating)
4. **Email Compose** (keep, update data to match original)
5. **Coffee Order** (NEW - list with prices)
6. **Contact Card** (NEW - avatar, name, title, phone)

### 2.2 Masonry Layout
- Replace `grid grid-cols-3` with CSS columns/masonry
- Options: `columns-3` with `break-inside-avoid` on cards
- Or use `vue-masonry` library

### 2.3 New Components Needed for Gallery Widgets
- **Avatar**: circular image component (needed for Chat Message, Contact Card)
- **Star Rating**: display star ratings (needed for Recipe Card)
- Maybe others based on original widget details

---

## Phase 3: Icons Page (`/icons`)

### 3.1 Material Icons
- Add `@mdi/js` or `material-symbols` dependency
- Replace Lucide with Material Icons in IconsGrid
- Show 100 icons (not 60)

### 3.2 Card Grid Styling
- Each icon cell: `bg-white rounded-xl shadow-sm p-4`
- Grid: `grid-cols-8 gap-3`

### 3.3 "Browse all icons" Link
- Add top-right link to https://fonts.google.com/icons

---

## Phase 4: Basic Catalog (`/basic-catalog`)

### 4.1 Architecture Redesign
- Two-column layout: sidebar nav (left) + detail view (right)
- Sidebar lists all component names
- Main area shows ONE component at a time with:
  - Preview (live A2UIRenderer)
  - Usage (code snippet)
  - Props table

### 4.2 Component Data Structure
Define for each component:
- Name, description
- Demo components array
- Usage code snippet (Auto syntax)
- Props list: name, type, default, description

### 4.3 Missing Components to Add
- Video
- AudioPlayer
- DateTimeInput
- ChoicePicker
- Navigation
- Modal
- Decoration

---

## Phase 5: Custom Catalog (`/custom-catalog`)

### 5.1 Architecture Redesign
- Left sidebar: "Assembled Components" / "Catalog Components"
- Main area with tabs: "Flight Card" | "Sales Dashboard"
- Right panel: JSON data editor (read-only textarea with JSON)

### 5.2 Flight Card
- Real airline data with favicons
- Price display
- Flight numbers, times
- Airline logos

### 5.3 Sales Dashboard
- Revenue metrics
- Bar chart with weekly data

---

## Phase 6: Theater Page (`/theater`)

### 6.1 Top Tab Bar
- Tabs: "Events" | "Data" | "Config"

### 6.2 Playback Controls
- Play/Pause button
- Skip back/forward
- Progress scrubber (slider)
- Speed selector (0.5x, 1x, 2x)

### 6.3 JSONL Stream Panel
- "Pretty" / "Wire" toggle
- Dark background code panel

### 6.4 Mock Browser Frame
- Traffic light dots (red/yellow/green)
- URL bar
- "React Renderer" label
- `<A2UIRenderer />` placeholder

---

## New Components to Build

| Component | Where Used | Complexity |
|-----------|-----------|------------|
| Avatar | Gallery (Chat, Contact) | Low |
| StarRating | Gallery (Recipe Card) | Low |
| Video | Basic Catalog | Medium |
| AudioPlayer | Basic Catalog | Medium |
| DateTimeInput | Basic Catalog | Low |
| ChoicePicker | Basic Catalog | Low |
| Navigation | Basic Catalog | Medium |
| Modal | Basic Catalog | Medium |
| Decoration | Basic Catalog | Low |

---

## Execution Order

1. **P0**: Global (gradient, sidebar, router)
2. **P1**: Create page (quick win)
3. **P2**: Gallery (masonry + original widgets)
4. **P3**: Icons (material icons)
5. **P6**: Theater (bounded scope)
6. **P4**: Basic Catalog (large, needs new components)
7. **P5**: Custom Catalog (tabs + data panel)
8. **Final**: Screenshot comparison, verify pixel match
