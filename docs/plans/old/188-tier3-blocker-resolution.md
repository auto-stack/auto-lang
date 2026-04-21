# 188: Tier 3 Mini Apps — Blocker Resolution

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Resolve all 4 prerequisites blocking Tier 3 (Mini Apps, 011-016) from reaching their full potential, as identified during Plan 183 implementation.

**Architecture:** Fix AURA parser, Vue generator, and runtime to support: grid layout, timer/async ticks, view-block template strings, and typed msg variant handlers. Each fix is independently valuable and unblocks future Tier 3/Tier 4 examples.

**Tech Stack:** Rust (parser, codegen, VM), Vue 3 generator

---

## Background

During Plan 183 Tier 3 implementation, all 6 examples (011-calculator through 016-calendar) were created but had to be significantly simplified due to 4 blocker issues. The examples currently build and render, but lack:
- Proper grid layout (calendar uses pre-formatted text instead of CSS grid)
- Real-time timer updates (stopwatch shows static time)
- Dynamic content switching with template strings (todo filter, notes editor)
- Event handlers with payload data (all msgs are parameterless)

## Blocker Analysis

### B1: `${}` template strings crash in view blocks

**Symptom:** Using `${.var}` inside backtick strings in view blocks causes `Syntax(Generic { message: "Expected term, got RBrace" })` parse error.

**Root Cause:** The AURA view node parser likely tokenizes `${}` inside template strings incorrectly — the `$` or `{` gets consumed by the view block parser as a child node delimiter rather than being part of the string literal.

**Reproduction:**
```auto
view {
    center {
        text `${.count} items`   // CRASHES
        text .count               // works
    }
}
```

**Impact:** High. Prevents any dynamic text display in views. Affects all examples that need to show computed/state values alongside static text.

### B2: Grid layout widget not functional

**Symptom:** Using `grid` / `grid-item` widgets in view blocks either crashes the parser or produces incorrect output.

**Root Cause:** Unknown — needs investigation. The widget registry registers `Grid` with mappings for ark/jet/vue, but the actual code generation may be broken.

**Reproduction:**
```auto
view {
    grid {
        grid-item { text "1" }
        grid-item { text "2" }
    }
}
```

**Impact:** Medium. Calendar and Calculator need grid layouts. Can be worked around with `row` + fixed-width `text` elements (current approach).

### B3: No timer/async tick mechanism

**Symptom:** No way to update model state on a timer. `on` handlers only fire on user events (click, input). No `setInterval`, `setTimeout`, or async event mechanism exists.

**Root Cause:** Architectural — the AURA widget system currently has no runtime event loop for timer-based updates. The VM runs to completion after processing initial state; there's no mechanism to schedule recurring updates.

**Impact:** High. Stopwatch, pomodoro timer, and any animation-dependent widget are impossible. Requires runtime design decision.

### B4: Typed msg variants not handled by generators

**Symptom:** `msg Msg { AddItem(str) }` parses correctly, but Vue generator doesn't generate code to pass the payload from the event to the handler.

**Root Cause:** The Vue generator's event handler mapping only supports simple `@click="onMsgName"` without arguments. It doesn't generate the parameter-passing code for `on { .AddItem(text) -> { ... } }`.

**Impact:** Medium. Todo (add with text), Notes (search filter), Converter (input binding) all need payload-bearing msgs. Workaround: use `oninput`/`onchange` handlers with model vars instead.

---

## Tasks

### Task 1: Fix `${}` template string parsing in view blocks

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` (view node parsing, around lines 9297-9339)
- Test: `crates/auto-lang/test/vm/` or manual `auto build` test

**Step 1: Investigate the tokenizer/parser interaction**

Find where view block text content is parsed. The issue is that `${` inside a backtick string in a view block is being interpreted as view syntax rather than string interpolation.

Check:
1. How `parse_view_node` handles text content (string literals vs identifiers)
2. Whether the tokenizer enters a special mode for view blocks
3. How `text "hello"` (quoted string) vs `text \`hello ${.x}\`` (template string) are differentiated

**Step 2: Write a failing test**

Create a test case that parses a widget with `${}` in a view block and expects it to succeed:
```auto
widget App {
    model { var count int = 5 }
    view { text `${.count} items` }
    on { }
}
```

**Step 3: Fix the parser**

Ensure that when parsing a view node's text content (the first argument to `text`, `span`, `h1`, etc.), template strings with `${}` are correctly tokenized as a single string expression, not broken into multiple view child nodes.

**Step 4: Verify with existing examples**

Run `auto build` on 014-weather and confirm `${.temp}°` renders correctly in generated Vue output.

**Step 5: Commit**

---

### Task 2: Fix grid/grid-item code generation

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` (grid widget generation)
- Modify: `crates/auto-lang/src/ui_gen/jet/mod.rs` (grid widget generation, if applicable)
- Modify: `crates/auto-lang/src/ui_gen/ark/mod.rs` (grid widget generation, if applicable)
- Test: `examples/ui/016-calendar/` or dedicated test

**Step 1: Test current grid parsing and generation**

Create a minimal test widget using `grid` and `grid-item`:
```auto
widget App {
    view {
        grid {
            grid-item { text "1" }
            grid-item { text "2" }
            grid-item { text "3" }
        }
    }
    on { }
}
```

Run `auto build` and inspect generated Vue output.

**Step 2: Fix the generator**

Ensure `grid` generates proper CSS grid layout (`display: grid; grid-template-columns: repeat(auto-fill, minmax(...))`) and `grid-item` generates child elements.

**Step 3: Update calendar example to use real grid**

Replace pre-formatted text rows with actual `grid` + `grid-item` layout in `016-calendar/src/front/app.at`.

**Step 4: Verify build**

**Step 5: Commit**

---

### Task 3: Design and implement timer/tick mechanism

**Files:**
- Create: `crates/auto-lang/src/aura/timer.rs` (or extend existing runtime)
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` (generate timer code)
- Modify: `crates/auto-lang/src/ui_gen/jet/mod.rs` (generate timer code)
- Design doc: `docs/design/timer-tick.md`

**Step 1: Design the timer API**

Options to evaluate:
1. **Declarative `timer` widget** — `<timer (interval: 1000) { .elapsed = .elapsed + 1 }>` in view or model
2. **`on tick` handler** — `on { .Tick -> { .elapsed = .elapsed + 1 } }` with `timer` model attribute
3. **`async` model property** — `model { var elapsed int = 0 ~tick(1000) }` using existing async syntax

Recommended: Option 2 — add a `tick` msg that the runtime emits at a configurable interval:
```auto
model {
    var interval int = 1000
    var elapsed int = 0
}

on {
    .Tick -> { .elapsed = .elapsed + 1 }
}
```

The generator would emit `setInterval` (Vue) / `LaunchedEffect` (Compose) based on the presence of a `.Tick` handler.

**Step 2: Write design document**

Document the chosen approach, generator mapping per backend, and edge cases (multiple timers, cleanup on widget destroy).

**Step 3: Implement parser support**

If new syntax is needed, add it to `parse_model_block` or `parse_on_block`.

**Step 4: Implement Vue generator**

Generate `setInterval`/`onUnmounted` cleanup code for tick handlers.

**Step 5: Implement Jet generator**

Generate `LaunchedEffect` with `delay` for tick handlers.

**Step 6: Update stopwatch example**

Add real timer functionality to `012-stopwatch/src/front/app.at`.

**Step 7: Verify build**

**Step 8: Commit**

---

### Task 4: Fix typed msg variant handler generation

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` (event handler with args)
- Modify: `crates/auto-lang/src/aura/extract.rs` (payload extraction)
- Test: manual `auto build` test

**Step 1: Test current typed msg behavior**

Create a test widget:
```auto
widget App {
    msg Msg { AddItem(str) }
    model { var items str = "" }
    view {
        input { placeholder: "Add item", oninput: .AddItem }
        text .items
    }
    on {
        .AddItem(text) -> { .items = text }
    }
}
```

Run `auto build` and check generated Vue code.

**Step 2: Fix the extractor**

Ensure `extract.rs` correctly extracts the parameter name (`text`) from the on-handler pattern and includes it in the `AuraOnHandler` struct.

**Step 3: Fix the Vue generator**

Generate Vue code that passes the event value:
```vue
<input @update:modelValue="(val) => onAddItem(val)" />
```
```ts
function onAddItem(text: string): void {
  items.value = text;
}
```

**Step 4: Fix Jet and Ark generators** (if applicable)

Ensure Compose and ArkTS generators also handle payload-bearing handlers.

**Step 5: Update todo example**

Add typed `AddItem(str)` msg and use payload in handler instead of reading from `.input`.

**Step 6: Verify build**

**Step 7: Commit**

---

### Task 5: Regression test and example upgrade

**Files:**
- Modify: `examples/ui/011-calculator/src/front/app.at` through `016-calendar/src/front/app.at`
- Modify: `docs/plans/183-unified-ui-examples.md` (update Phase 2 prerequisites)

**Step 1: Run full regression**

```bash
cargo test -p auto-lang --lib
```

Expected: 2813+ passed, 0 failed.

**Step 2: Re-build all Tier 1-3 examples**

```bash
for dir in examples/ui/*/; do
    (cd "$dir" && auto build 2>&1 | grep -E "(Parse error|built successfully)")
done
```

Expected: all 16 examples build successfully.

**Step 3: Upgrade Tier 3 examples**

After B1-B4 are fixed, upgrade the simplified examples to their full designs:
- Calculator: add real arithmetic logic (needs B4 for operator state)
- Stopwatch: add real timer (needs B3)
- Todo: add filter tabs and add-with-text (needs B1 + B4)
- Weather: use `${}` for dynamic temp display (needs B1)
- Notes: add real note selection and editing (needs B1 + B4)
- Calendar: use real grid layout (needs B2)

**Step 4: Update Plan 183**

Mark Phase 2 prerequisites as resolved. Update the progression map.

**Step 5: Commit**

---

## Priority Order

1. **B1 (template strings)** — Highest impact, likely a parser bug fix
2. **B4 (typed msg variants)** — Enables proper event data flow
3. **B2 (grid layout)** — Needed for calendar, lower complexity
4. **B3 (timer/tick)** — Most complex, requires design decision

B1 and B4 can be done in parallel. B2 is independent. B3 should be done last as it requires the most design work.

## Verification

After all tasks:
1. All 16 examples (001-016) build successfully with `auto build`
2. Tier 3 examples use their full intended features (grid, timers, filters, dynamic text)
3. `cargo test -p auto-lang --lib` passes with 0 failures
4. Plan 183 Phase 2 prerequisites are marked as resolved
