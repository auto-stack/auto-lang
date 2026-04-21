# 189: Tier 4 Prerequisites — Feature Gaps for Real Apps (017-024)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Resolve all prerequisites blocking Tier 4 (Real Apps, 017-024) from being implementable, as identified by analyzing Plan 183 Phase 3 against the current AURA widget system.

**Architecture:** Add missing features to the AURA parser, extractor, and Vue generator: dark mode toggle composable, `if` conditional rendering with state comparisons in views, and simplified multi-page routing examples. Skip features requiring external JS libraries (drag-drop, markdown) — these will use static mock data instead.

**Tech Stack:** Rust (parser, codegen), Vue 3 generator

---

## Background

Tier 4 examples (017-024) require features beyond what Tier 1-3 used:

| Feature | Status | Impact |
|---------|--------|--------|
| Routing (multi-page) | EXISTS | Fully working — just needs an example |
| List rendering (for loops) | EXISTS | Fully working |
| Image widget | EXISTS | Fully working |
| Progress bar / Slider | EXISTS | Fully working |
| Tabs / Navigation drawer | EXISTS | Fully working |
| **Dark mode toggle** | PARTIAL | Tailwind `darkMode: ["class"]` exists but no toggle mechanism |
| **Drag and drop** | MISSING | Kanban (022) needs it — will use static mock |
| **Auth patterns** | MISSING | RealWorld (023) needs it — will use mock login UI |
| **Markdown rendering** | MISSING | Blog (021) needs it — will use static text |
| **Object types in model** | PARTIAL | Complex data awkward without structs — use flat vars |

## Strategy

**Implementable now (this plan):**
1. Dark mode toggle — add `useColorMode`-like logic to Vue generator
2. Multi-page routing example — validate routing works end-to-end
3. Static mock patterns — define how to handle missing features (drag, auth, markdown)

**Skip (use static mocks in examples):**
- Drag and drop: Kanban (022) will use static columns, no reordering
- Auth: RealWorld (023) will have mock login UI, no real auth
- Markdown: Blog (021) will show plain text, no markdown parsing
- Complex objects: Use flat string vars as in Tier 3

---

## Tasks

### Task 1: Add dark mode toggle support to Vue generator

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` (dark mode state + class toggle)
- Test: manual `auto build` test

**Step 1: Detect dark mode model var**

In `generate_script`, check if `widget.state_vars` contains a var named `isDark` (or `darkMode`, `theme`). If so, generate `useColorMode`-like logic:

```ts
const isDark = ref<boolean>(false)
const toggleDark = () => {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
}
```

**Step 2: Add `dark:` prefix support in class generation**

When a model var `isDark` exists, the wrapper div should bind `:class` for dark mode:

```html
<div :class="{ dark: isDark }" class="flex flex-col h-screen">
```

**Step 3: Generate `onMounted` to read system preference**

If dark mode is enabled, add `onMounted` to detect `prefers-color-scheme: dark`.

**Step 4: Test with minimal widget**

```auto
widget App {
    model { var isDark bool = false var title str = "Hello" }
    view { col { text .title button "Toggle" { onclick: .Toggle } } }
    on { .Toggle -> { .isDark = !.isDark } }
}
```

**Step 5: Commit**

### Task 2: Validate multi-page routing with a minimal example

**Files:**
- Create: `examples/ui/017-chat/` (or simpler test project)
- Test: `auto build` must produce working router

**Step 1: Create a simple 2-page test project**

Two pages: home and about, with navigation between them.

**Step 2: Build and verify router generation**

Check that `gen/vue/src/router/index.ts` is generated correctly.

**Step 3: Commit**

### Task 3: Implement all Tier 4 examples (017-022) as static mocks

**Files:**
- Create: `examples/ui/017-chat/src/front/app.at`
- Create: `examples/ui/018-book-reader/src/front/app.at`
- Create: `examples/ui/019-video-app/src/front/app.at`
- Create: `examples/ui/020-music-player/src/front/app.at`
- Create: `examples/ui/021-blog-viewer/src/front/app.at`
- Create: `examples/ui/022-kanban/src/front/app.at`

Each example should:
- Use existing AURA features (col, row, grid, text, button, image, tabs, progress, etc.)
- Use flat string vars for complex data (as in Tier 3)
- Use `for item in .list` for list rendering where possible
- Use `${}` template strings for dynamic content
- Use `if` conditionals for show/hide logic
- Build successfully with `auto build`

**Step 1: 017-chat** — WeChat-style messenger layout
**Step 2: 018-book-reader** — E-book reader with dark mode toggle
**Step 3: 019-video-app** — Bilibili-style video grid browser
**Step 4: 020-music-player** — Spotify-style mini player
**Step 5: 021-blog-viewer** — Blog list + detail layout
**Step 6: 022-kanban** — Trello-style 3-column board (static, no drag)
**Step 7: Build all examples and verify**
**Step 8: Commit**

### Task 4: Regression test

**Step 1:** Run `cargo test -p auto-lang --lib`
**Step 2:** Build all Tier 1-4 examples
**Step 3:** Commit

---

## Priority Order

1. Task 1 (dark mode) — needed by 018-book-reader
2. Task 2 (routing validation) — needed by 021-blog-viewer
3. Task 3 (all examples) — the main deliverable
4. Task 4 (regression) — quality gate

## Verification

After all tasks:
1. All 22+ examples (001-022) build successfully with `auto build`
2. `cargo test -p auto-lang --lib` passes with 0 failures
3. Tier 4 examples use their full intended layouts (tabs, grids, dark mode, lists)
