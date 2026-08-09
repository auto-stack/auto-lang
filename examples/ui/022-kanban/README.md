# 022-kanban — Trello-Style Board (Full App)

A Kanban board with three columns (To Do / In Progress / Done). Cards can be
added, moved between columns, and deleted — all backed by a typed Rust API.
Upgraded from a single-file static toy to a full App (Plan 404), aligned with
015-notes / 017-chat / 018-book-reader.

## Concepts

- **Multi-module frontend** — `app.at` (route shell) + `board_store.at`
  (shared store) + `pages/board.at` (the board), cf. 018's structure.
- **Strongly-typed Rust backend** — `src/back/{api.at, db.at}`, `#[api]`
  endpoints delegating to db.rs (Plan 399 route B: full coverage).
- **Single-collection model** — cards live in one `Card { id, title, column }`
  collection; moving a card just changes its `column` (standard kanban
  modeling, simpler than three independent lists).
- **Shared store (k1 pattern)** — `BoardStore` holds `cards`; the board page
  does `use store: BoardStore` and re-fetches after every mutation.
- **Column filtering in the view** — each column renders `for card in
  .store.cards` with an `if card.column == "todo"` filter (cf. 018 pages).
- **Per-example ports** — `front_port: 3022` / `back_port: 8022` in `pac.at`
  so it can run concurrently with other examples.

## Source

```
src/front/
  app.at              # App shell: routes{"/" -> board} + header
  board_store.at      # BoardStore: cards collection + add/remove/move
  pages/
    board.at          # "/" → three columns + add-card input + move/delete
src/back/
  api.at              # pub type Card + 4 #[api] endpoints
  db.at               # typed in-memory store + 9 seed cards
tests/                # playwright suite (T1-T8)
```

## How to Run

> ⚠️ **Port note (known gap, Plan 404)**: the current `auto run` does **not**
> inject `pac.at`'s `front_port`/`back_port` into the dev servers (it still
> defaults to 3000/8080). To run on the dedicated 3022/8022 ports, set the
> environment variables explicitly:

Generate first (so `router/index.ts` exists before `auto run`):

```bash
cd examples/ui/022-kanban
auto gen
```

Then run — **option A** (dedicated ports via env, recommended for testing):

```bash
# 1) backend (separately, on 8022)
AUTO_HTTP_PORT=8022 ./examples/rust-workspace/target/debug/app-022-kanban-back.exe
# 2) frontend vite (on 3022, proxy → 8022)
cd gen/front/vue
AUTO_FRONT_PORT=3022 AUTO_HTTP_PORT=8022 pnpm dev
# open http://localhost:3022/
```

**Option B** (default ports, simplest — no env vars):

```bash
auto run    # serves frontend (:3000) + backend (:8080), vite proxy wired
# open http://localhost:3000/
```

## API

| Method | Path | Action |
|---|---|---|
| GET | `/api/cards` | list all cards |
| POST | `/api/cards` | add a card (defaults to "todo") |
| DELETE | `/api/cards/:id` | remove a card |
| PUT | `/api/cards/:id/move` | move a card to another column (`{column}`) |

## Tests

Start the servers (Option A above), then:

```bash
cd tests && npm install && npm test    # playwright, T1-T8
```

Acceptance contract in `tests/acceptance.atd`. Tests cover: initial render
(9 seed cards), three column titles, add card, move todo→doing, move
doing→done, delete card, state persistence across reload, no console errors.

## Notes

- Card movement supports both: `>` buttons (todo→doing→done) / `×` to delete,
  **and HTML5 drag-and-drop** (drag a card onto another column).
- **Drag-and-drop** (Plan 404 stage 2): cards are `draggable`, `ondragstart`
  stores the id in widget state, columns use `ondragover.prevent` + `ondrop`
  to read it and call `store.MoveCard`. Widget state (not `dataTransfer`)
  carries the id since source and target share one widget.
- **codegen fix**: `row`/`col` layout primitives used to drop non-class props
  (their shadcn branch only emitted `class`), so `draggable` never reached the
  DOM. Fixed via `push_passthrough_attrs` — row/col now pass through arbitrary
  HTML attributes like `draggable`.

## Inspiration

Trello, Linear.
