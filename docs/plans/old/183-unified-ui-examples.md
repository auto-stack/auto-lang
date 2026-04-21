# 183: Unified UI Examples

Design for a progressive collection of cross-platform UI examples in examples/ui/. Each example contains a single .at source file that generates to all six targets: Vue, Jetpack Compose, ArkTS, GPUI, Tauri, and VSCode WebView. Each folder also includes a README.md showing generated output side-by-side.

## Goals

1. Progressive tutorial — newcomers learn AURA one concept at a time
2. Capability showcase — reference gallery of what AURA can build
3. Cross-platform proof — same .at source, six native targets

## Directory Convention

```
examples/ui/
  001-helloworld/
    input.at
    README.md          # shows all 6 generated outputs with annotations
  002-counter/
    input.at
    README.md
  ...
```

Each README contains:

* What the example teaches (concepts)
* The .at source (inline or linked)
* Generated output for each target (in code blocks, side-by-side where practical)
* Notes on platform-specific differences

## Generator Targets

| Target                | Output               | Status       |
| --------------------- | -------------------- | ------------ |
| Vue 3                 | .vue SFC             | Working      |
| Jetpack Compose       | .kt                  | Working      |
| ArkTS (HarmonyOS)     | .ets                 | Working      |
| GPUI (Zed-style Rust) | .rs Component struct | Working      |
| Tauri                 | .html + .ts WebView  | Aspirational |
| VSCode WebView        | .ts panel            | Aspirational |

Aspirational targets get placeholder README sections noting they are coming soon.

***

## Tier 1: Basics (001-005)

Core AURA concepts. Each example is tiny — one screen, under 50 lines.

### 001-helloworld

Static text display. Zero interactivity.

Concepts: view tree, text widget, col layout, class styling

Source:

```auto
widget App {
    view {
        col {
            text "Hello, World!"
            class: "w-full h-full justify-center items-center bg-white"
        }
    }
}
```

### 002-counter

Increment/decrement/reset counter with three buttons.

Concepts: model, msg enum, event handlers, button widget

Source skeleton:

```auto
widget Counter {
    msg Msg { Inc, Dec, Reset }

    model {
        var count int = 0
    }

    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: .Dec }
                button "Reset" { onclick: .Reset }
                button "+" { onclick: .Inc }
            }
        }
    }

    on {
        .Inc -> { .count = .count + 1 }
        .Dec -> { .count = .count - 1 }
        .Reset -> { .count = 0 }
    }
}
```

### 003-converter

Temperature converter: Celsius ↔ Fahrenheit. Editing either field updates the other.

Concepts: input widget, computed properties, bidirectional data flow

Inspiration: 7GUIs Task #2 (Temperature Converter)

### 004-profile-card

User profile card with avatar image, name, role badge, and bio text.

Concepts: image widget, badge, card, col/row nesting, styling with classes

Inspiration: PrimeBlocks card patterns

### 005-login

Login form with email and password fields, validation, and error display.

Concepts: form widgets (input, button), validation state, conditional rendering (error messages), loading state on submit

***

## Tier 2: Blocks (006-010)

PrimeBlocks-style composed UI sections. More complex than single widgets, not yet full apps. Each demonstrates composing multiple widgets into a cohesive UI block.

### 006-hero-section

Landing page hero: headline, subtitle, CTA button, background image.

Concepts: rich text layout (h1, subtitle), button variants, centering, background image

Inspiration: PrimeBlocks hero block

### 007-stats-board

Four metric cards showing revenue (\$256K), active users (1,453), orders (83M), growth rate. Each card has a title, value, and progress bar.

Concepts: progress widget, repeated card patterns, data display, separator, responsive grid

Inspiration: PrimeBlocks stats block, Zed status bar metrics

### 008-pricing-table

Three pricing tiers (Basic/Premium/Enterprise) with monthly/yearly toggle switch. Each tier shows price, feature list, and CTA button.

Concepts: switch/toggle, conditional content (price changes with toggle), list rendering (features), card variants (highlighted "popular" tier)

Inspiration: PrimeBlocks pricing block

### 009-article-feed

Blog article cards: image, title, excerpt, author avatar + name, publish date. Multiple cards in a scrollable column.

Concepts: image cards, text with line clamping, repeated items, avatar, date formatting

Inspiration: PrimeBlocks article cards, JetNews article list

### 010-contact-form

Full contact form: name input, email input, subject dropdown, message textarea, submit button. Shows toast on submit.

Concepts: select/dropdown, textarea, form composition, submit feedback (toast notification)

Inspiration: PrimeBlocks contact section

***

## Tier 3: Mini Apps (011-016)

Self-contained interactive applications with multiple states.

### 011-calculator

Basic four-function calculator: display + 4×5 button grid (0-9, +, -, ×, ÷, =, C).

Concepts: grid layout (4 columns), complex state machine (chained operations), display widget

Inspiration: Flutter Simplistic Calculator, iced calculator examples

### 012-stopwatch

Start/stop/reset stopwatch with lap times list. Time formatted as MM:SS.cc.

Concepts: timer ticks (async), start/stop/reset state machine, lap list, scroll, time formatting

Inspiration: iced stopwatch example

### 013-todo

TodoMVC spec compliance: add todos, toggle done, filter (all/active/completed), clear completed, item count.

Concepts: list CRUD, filter tabs, checkbox, inline editing, routing (hash-based filter), toggle-all

Inspiration: TodoMVC (todomvc.com)

### 014-weather

Weather dashboard: city name, current temp, conditions icon, 5-day forecast cards, loading skeleton while "fetching".

Concepts: tabs, loading skeleton, conditional rendering (loading vs loaded), icon mapping

### 015-notes

Split-pane note-taking app: sidebar with note list + search bar, main area with textarea editor. Create, select, edit, delete notes.

Concepts: split pane layout, search/filter, textarea editing, selection state, sidebar

### 016-calendar

Month-view calendar grid (7 columns, 5-6 rows). Shows event dots on dates. Previous/next month navigation.

Concepts: grid (7 columns), date arithmetic, event indicators, prev/next navigation, header with controls

***

## Tier 4: Real Apps (017-024)

Simplified versions of popular real-world applications. These prove the "write once, run anywhere" story.

### 017-chat

WeChat-style messenger: contact list (left), message thread (right), input bar at bottom. Message bubbles with timestamps and avatars.

Concepts: scrollable message list, auto-scroll to bottom, message bubble styles (sent vs received), avatar, input bar

Inspiration: WeChat, Jetchat (Jetpack Compose)

### 018-book-reader

E-book reader: chapter list sidebar, reading area, progress bar at bottom, dark mode toggle.

Concepts: pagination/scroll, reading progress bar, theme toggle (dark mode), typography, chapter navigation

Inspiration: Kindle reader, Apple Books

### 019-video-app

Bilibili-style video browser: top navigation (trending/subscribed/search), video thumbnail grid, category chips, video detail overlay.

Concepts: video thumbnail grid, tabs (trending/subscribed/search), category chips/filters, grid layout (responsive), detail overlay/sheet

Inspiration: Bilibili, YouTube

### 020-music-player

Spotify-style mini player: album art, song title + artist, playback controls (prev/play/next), progress scrubbing bar, playlist below.

Concepts: album art display, playback controls (icon buttons), progress bar with scrubbing, playlist list, bottom bar layout

Inspiration: Spotify, Apple Music

### 021-blog-viewer

Blog with article list (left) and article detail (right). Navigation drawer, share button, markdown rendering.

Concepts: routing (list → detail), navigation drawer, markdown text rendering, share action, responsive layout

Inspiration: JetNews (Jetpack Compose), Medium

### 022-kanban

Trello-style board: 3 columns (To Do / In Progress / Done), draggable cards, add card, edit card title inline.

Concepts: column layout, drag reorder, card details, inline editing, column add/remove

Inspiration: Trello, Linear

### 023-realworld

Medium.com clone (Conduit spec): auth (login/register), article feed, article detail with comments, profile page, create/edit article, tags, pagination.

Concepts: full routing (6+ pages), auth flow, CRUD articles, comments, profiles, pagination, tag filtering, multi-page navigation, loading states

Inspiration: RealWorld/Conduit (github.com/gothinkster/realworld)

### 024-widget-gallery

Comprehensive component showcase: sidebar navigation listing all widgets, main area showing each widget with code + live preview. Modeled after shadcn/ui docs.

Concepts: all widgets in one app, sidebar navigation, 40+ component demos, router with many routes, search/filter components

Inspiration: shadcn/ui docs, PrimeReact component gallery

***

## Progression Map

Each example builds on concepts from earlier ones:

```
001-helloworld ─── view, text, col
002-counter ─────── + model, msg, handlers, button
003-converter ───── + input, computed, bidirectional
004-profile-card ── + image, badge, card, styling
005-login ────────── + validation, conditional rendering

006-hero-section ── + rich text, button variants
007-stats-board ──── + progress, data cards, grid
008-pricing-table ── + switch/toggle, list rendering
009-article-feed ──── + avatar, date, scrollable feed
010-contact-form ─── + select, textarea, toast

011-calculator ────── + grid layout, state machine
012-stopwatch ─────── + timer/async, laps, scroll
013-todo ──────────── + CRUD, filter tabs, routing
014-weather ────────── + skeleton, loading, icons
015-notes ──────────── + split pane, search, editor
016-calendar ───────── + date grid, navigation

017-chat ──────────── + auto-scroll, message bubbles
018-book-reader ────── + dark mode, progress, pagination
019-video-app ──────── + video grid, chips, overlay
020-music-player ────── + playback controls, scrubbing
021-blog-viewer ─────── + routing, markdown, drawer
022-kanban ──────────── + drag reorder, columns
023-realworld ──────── + auth, full routing, CRUD
024-widget-gallery ──── + all widgets, comprehensive showcase
```

***

## Excluded Examples (Future Candidates)

The following examples were considered but excluded from the initial set. Each has a reason — we may add them later as AURA capabilities expand.

### Canvas / Custom Drawing

| Example                           | Reason Excluded                                                                          | Prerequisite                     |
| --------------------------------- | ---------------------------------------------------------------------------------------- | -------------------------------- |
| circle-drawer (7GUIs #6)          | AURA has no canvas/drawing API                                                           | Canvas widget, mouse hit-testing |
| drawing-app (pen/line/rect tools) | Same — no canvas, no undo/redo system                                                    | Canvas + undo stack              |
| game-of-life (Conway)             | Grid is widget-based, not pixel canvas                                                   | Canvas or custom rendering       |
| bezier-tool                       | Requires interactive canvas + mouse drag                                                 | Canvas API                       |
| minesweeper                       | Grid + right-click context menu feasible, but flood-fill logic is complex for an example | Context menu widget              |

### Rich Text Editing

| Example                                     | Reason Excluded                                                                    | Prerequisite                             |
| ------------------------------------------- | ---------------------------------------------------------------------------------- | ---------------------------------------- |
| mini-code-editor (syntax highlighting)      | AURA has no rich text editing widget                                               | TextEditor widget with selection, cursor |
| markdown-editor (split pane + live preview) | Same — needs text editor widget                                                    | TextEditor widget                        |
| spreadsheet (7GUIs #7)                      | Grid + formula parsing + reactive propagation is too niche for cross-platform demo | Table widget with cell editing           |

### Multi-Window / Desktop-Specific

| Example                | Reason Excluded                                                             | Prerequisite                  |
| ---------------------- | --------------------------------------------------------------------------- | ----------------------------- |
| multi-window           | Not universally supported across targets (mobile lacks multi-window)        | Multi-window API per platform |
| file-explorer          | Tree view + filesystem access is desktop-only                               | Tree widget + filesystem API  |
| terminal-emulator      | Requires PTY integration, not a UI concern                                  | Terminal widget + PTY bridge  |
| code-editor (mini IDE) | Too ambitious — syntax highlighting, autocomplete, tabs, file tree combined | Multiple missing widgets      |

### Other Candidates for Future Expansion

These did not make the cut due to list length, but would add value in a future expansion round.

| Example                  | Concepts It Teaches                                                              | Tier     |
| ------------------------ | -------------------------------------------------------------------------------- | -------- |
| color-picker             | RGB sliders, hex display, color preview, real-time binding                       | Block    |
| flight-booker (7GUIs #3) | Dropdown, conditional enable/disable, date validation, cross-widget constraints  | Mini App |
| timer (7GUIs #4)         | Async countdown, progress bar, slider for duration, competing user/async signals | Mini App |
| crud (7GUIs #5)          | List + form, prefix filter, create/update/delete, selection state                | Mini App |
| expense-tracker          | Date picker, category dropdown, table/chart, local DB, filtering                 | Real App |
| pomodoro-timer           | Timer states (work/break), statistics chart, notification, persistence           | Mini App |
| landing-page             | Full-page composition: hero + features + pricing + contact + footer              | Block    |
| email-client             | Adaptive layout (phone/tablet/desktop), folder navigation, compose               | Real App |
| podcast-app              | Dynamic theming from content, audio playback, favorites                          | Real App |
| image-gallery            | Grid of images, lightbox overlay, zoom, lazy loading                             | Mini App |
| password-generator       | Checkboxes, slider (length), copy-to-clipboard, string manipulation              | Block    |
| unit-converter           | Dropdown categories, bidirectional conversion, favorites                         | Block    |
| recipe-app               | Cards, categories, search, detail page, ingredients list, steps                  | Mini App |
| habit-tracker            | Calendar grid, streak display, daily check-off, statistics                       | Mini App |
| ai-chat                  | Chat UI + streaming response, markdown rendering, model selector                 | Real App |
| dashboard                | Split panes, data tables, sidebar navigation, charts placeholder                 | Real App |
| settings-page            | Tab-based settings, forms, toggles, keybindings, theme picker (Zed-style)        | Real App |
| store-app                | Product grid, cart, checkout flow, animations (Jetsnack-style)                   | Real App |

***

## Implementation Notes

### Per-Example Structure

Each example folder contains exactly:

* input.at — the AURA source
* README.md — concepts, source, generated output for all 6 targets, platform notes

### README Template

```markdown
# XXX-name — One-line Description

## Concepts
- Concept A
- Concept B

## Source
(at code block)

## Generated Output

### Vue 3
(vue code block)

### Jetpack Compose
(kotlin code block)

### ArkTS (HarmonyOS)
(ets code block)

### GPUI (Rust)
(rust code block)

### Tauri
(placeholder: coming soon)

### VSCode WebView
(placeholder: coming soon)

## Platform Notes
(any notable differences in how platforms render this)
```

### Naming Convention

* Folder: \{sequence}-\{kebab-case-name} (e.g., 013-todo)
* Sequence: zero-padded three digits
* Names: lowercase kebab-case, descriptive of the app (not the concept)

### Build Verification

Each example must:

1. Parse without errors in the AutoLang compiler
2. Generate valid output for all working targets (vue, jet, ark, gpui)
3. Include accurate generated output in the README

***

## Implementation Plan: Phase 1 (Tier 1 + Tier 2)

Tier 3 (Mini Apps) and Tier 4 (Real Apps) are deferred to a future plan. The current AURA widget set covers most needs for basics and blocks, but a few gaps must be filled first.

### Widget/Feature Gap Analysis

| Example           | Needs                                | Status                                           | Action                                               |
| ----------------- | ------------------------------------ | ------------------------------------------------ | ---------------------------------------------------- |
| 001-helloworld    | view, text, col, styling             | All generators support                           | None                                                 |
| 002-counter       | model, msg, handlers, button         | All generators support                           | None                                                 |
| 003-converter     | input, computed properties           | All generators support                           | None                                                 |
| 004-profile-card  | image, badge, card, col/row          | All generators support                           | None                                                 |
| 005-login         | input, button, conditional rendering | Supported, but no form validation                | Work around with manual error state in model         |
| 006-hero-section  | text, button, image, centering       | All generators support                           | None                                                 |
| 007-stats-board   | progress, card, col/row              | Progress missing in some generators              | Verify progress widget in GPUI; add if needed        |
| 008-pricing-table | switch/toggle, conditional, list     | Switch exists in all generators                  | None                                                 |
| 009-article-feed  | image, avatar, card, scroll          | Avatar missing in GPUI generator                 | Add avatar mapping to GPUI generator                 |
| 010-contact-form  | select, textarea, toast              | Select/textarea exist; toast missing in Jet/GPUI | Add toast to Jet and GPUI generators, or work around |

### Prerequisite Tasks

#### P1: Add toast/snackbar to Jet generator

The Jet generator lacks a toast widget. 010-contact-form needs submit feedback.

* Add toast to crates/auto-lang/src/ui\_gen/jet/components.rs mapping to Snackbar or Toast in Material3
* Or: model the toast as a conditional text element (workaround)

#### P2: Verify and add progress widget to GPUI generator

007-stats-board uses progress bars. The GPUI generator registers progress but may not emit correct code.

* Verify progress output in crates/auto-lang/src/ui\_gen/rust.rs
* Test with a simple progress example

#### P3: Add avatar mapping to GPUI generator

009-article-feed shows author avatars. GPUI has no avatar backend mapping.

* Add avatar as a circular image container in crates/auto-lang/src/ui\_gen/rust.rs
* Simple implementation: image with rounded-full styling

#### P4: Form validation workaround for 005-login

No generator supports form validation natively. Rather than building a validation framework now, the login example will use manual validation state in the model:

* var email\_error str = "" and var password\_error str = ""
* Validation logic in event handlers
* Conditional rendering of error text

This pattern actually teaches good AURA habits (explicit state > magic validation).

### Implementation Steps

#### Step 1: Create directory structure

* Create examples/ui/ directory
* Create all 10 example folders: 001-helloworld through 010-contact-form
* Each with empty input.at and README.md

#### Step 2: Implement Tier 1 examples (001-005)

Write the .at source and README for each. Generate output for all working targets. Order:

1. 001-helloworld — simplest, validates the directory structure and README template
2. 002-counter — validates model/msg/handler flow
3. 003-converter — validates input + computed properties
4. 004-profile-card — validates image/badge/card + styling
5. 005-login — validates conditional rendering + form pattern

Each example: write .at → run generators → paste output into README → verify builds.

#### Step 3: Complete prerequisite tasks (P1-P3)

* Add toast to Jet generator (or document workaround)
* Verify progress in GPUI generator
* Add avatar mapping to GPUI generator

#### Step 4: Implement Tier 2 examples (006-010)

Same flow as Step 2, but for the block-level examples:

1. 006-hero-section — validates rich text + centering
2. 007-stats-board — validates progress + data display
3. 008-pricing-table — validates switch/toggle + conditional content
4. 009-article-feed — validates avatar + repeated cards
5. 010-contact-form — validates select/textarea + toast feedback

#### Step 5: Cross-target review

* Verify all 10 examples generate valid output for Vue, Jet, Ark, GPUI
* Review READMEs for accuracy and consistency
* Ensure progression of concepts is clear when reading examples in order

### Parser Limitations Discovered (Plan 184)

The AURA widget parser has these limitations that required workarounds in Phase 1:

1. No typed msg variants: msg Msg \{ Foo(str) } causes parse errors. Only simple variants like msg Msg \{ Foo } work. This means on handlers cannot receive parameters — use on handlers without params and access model state directly.
2. No inline object/array literals in model: var items = \[\{ name: "a" }] fails to parse. Model vars must be simple scalar types (str, int, bool). Complex data must be split into individual vars (e.g., var title1 str = "a", var title2 str = "b").
3. No if statements in on handlers: The on block parser only supports simple assignments, not conditional logic.
4. class: must go after children: Per CLAUDE.md, styling properties should be placed after child elements in the view body.

These limitations mean complex examples (todo list with CRUD, dynamic feeds, etc.) cannot use data-driven patterns like for loops with arrays. Future work should add parser support for:

* Typed message variants with parameter extraction
* Object/array literal parsing in model blocks
* Control flow in on handler blocks

### Future Phases

| Phase   | Scope                       | Prerequisites                                                                          |
| ------- | --------------------------- | -------------------------------------------------------------------------------------- |
| Phase 2 | Tier 3 (Mini Apps, 011-016) | Grid layout verified, timer/async support, routing stable, typed msg variants          |
| Phase 3 | Tier 4 (Real Apps, 017-024) | Dark mode support, drag reorder, full routing, auth patterns, object literals in model |

Each phase gets its own implementation plan when prerequisites are met.
