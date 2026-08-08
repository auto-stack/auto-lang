# 038-minesweeper — Classic Minesweeper

A complete minesweeper game: left-click to reveal (with first-click safety and
flood-fill of empty regions), right-click to flag mines, three difficulty
levels (beginner 9×9 / intermediate 16×16 / expert 30×16), and a live timer
with a remaining-mines counter.

## Concepts

- **`use { fn }` escape-hatch** — the heavy game logic (mine placement,
  flood-fill, win/loss checks) lives in a hand-written TypeScript module
  (`src/front/utils/minesweeper.ts`) and is imported into the widget via
  `use { fn: ... from "..." }`. The `.at` widget stays a thin state shell
  (model + view + event routing) while the imperative algorithm runs in plain
  TS. See `029-external-imports` for the escape-hatch pattern.
- **DOM event modifier `oncontextmenu.prevent`** — right-click on a cell
  toggles a flag via `oncontextmenu.prevent: .Flag(x, y)`, which compiles to
  Vue's `@contextmenu.prevent` (suppressing the browser's native menu).
- **Timer convention** — `var interval int = 1000` in the model plus a `.Tick`
  handler in `on` is a codegen signal: the SFC auto-gets a `setInterval`
  (cleared on unmount) without any hand-written async code.
- **`computed` reactive labels** — `mines_label` / `timer_label` derive their
  display strings by calling the imported TS helpers, keeping the `text` nodes
  as clean field reads.
- **Four-state machine** — `game_state` cycles `ready → playing → won/lost`,
  driving both the timer (only ticks while `playing`) and the end-game banner.
- **Dynamic CSS grid via a style helper** — the board's column count is
  variable (9/16/30), which exceeds the Tailwind `grid-cols-N` ceiling (12).
  A `grid_style(cols)` helper returns a real `grid-template-columns: repeat(N …)`
  string bound through `:style`, so the grid re-flows on every difficulty
  switch.

## Source

- `src/front/app.at` — the widget: `model` (board + state), `computed`
  (labels), `view` (info bar, difficulty buttons, board grid, end banner),
  `on` (Reveal / Flag / SetDifficulty / Reset / Tick / Init).
- `src/front/utils/minesweeper.ts` — pure game logic, imported via the
  `use { fn }` block. `place_mines` is first-click-safe (the clicked cell and
  its 8 neighbors are never mines); `reveal_flood` uses an explicit stack
  (no recursion) to flood-fill empty regions.

## How to Run

```
cd examples/ui/038-minesweeper
auto gen    # generate gen/front/vue (imports minesweeper.ts into src/ext/)
auto run    # serve frontend (:4038)
```

Then open `http://localhost:4038`. Left-click reveals, right-click flags.
Switch difficulty with the three buttons; click 🔄 to restart the current
board.

## Inspiration

The Windows Minesweeper classic. Chosen as a focused showcase for the
escape-hatch + event-modifier + timer trio — a game whose logic is too large
to inline in a handler but whose UI is a clean declarative grid.
