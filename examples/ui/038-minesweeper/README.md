# 038-minesweeper — Classic Minesweeper (dual-backend)

A complete minesweeper game: left-click to reveal (with first-click safety and
flood-fill of empty regions), right-click to flag mines, three difficulty
levels (beginner 9×9 / intermediate 16×16 / expert 30×16), and a live timer
with a remaining-mines counter.

**This is the first AutoUI example that runs on TWO backends from the same
source** — `auto run` (vue) and `auto run --render vm`. All game logic is
written in pure AutoLang, so it compiles to a Vue store composable (vue
backend) **and** runs directly in the AutoVM interpreter (vm backend).

## Concepts

- **Store-driven pure-AutoLang logic** — the entire game (mine placement,
  flood-fill, win/loss, flag toggling) lives in `src/front/minesweeper_store.at`
  as store actions, with no escape-hatch and no `.ts`. The vue codegen
  translates the store into a composable; the VM interpreter runs the actions
  directly. The `app.at` widget is a thin view shell that only forwards events
  to `store.*` and reads precomputed fields.
- **Precomputed view strings (VM compatibility)** — the VM view-builder cannot
  call functions in bindings, so every per-cell style (`number_class`,
  `cell_class`) and every layout string (`grid_style`, `difficulty_class`) is
  computed inside the action and stored as a field on the cell / store. The
  view only reads fields — never calls a function.
- **DOM event modifier `oncontextmenu.prevent`** — right-click on a cell
  toggles a flag via `oncontextmenu.prevent: .Flag(x, y)` → Vue's
  `@contextmenu.prevent` (suppressing the browser's native menu).
- **Timer convention** — `var interval int = 1000` in the widget model plus a
  `.Tick` handler is a codegen signal: the SFC auto-gets a `setInterval`
  (cleared on unmount) without any hand-written async code.
- **Four-state machine** — `game_state` cycles `ready → playing → won/lost`,
  driving both the timer (only ticks while `playing`) and the end-game banner.
- **First-click-safe mine placement** — mines are laid out on the first click,
  avoiding the 3×3 zone around the clicked cell, so the opening move always
  reveals a region. Flood-fill uses an explicit stack (no recursion).

## Source

- `src/front/minesweeper_store.at` — `store MinesweeperStore`: all game logic
  as AutoLang actions (`Init` / `Reveal` / `Flag` / `Reset` / `SetDifficulty` /
  `Tick`). Cells are Obj literals carrying precomputed `number_class` /
  `cell_class` fields.
- `src/front/app.at` — `widget App`: a pure view shell. `use store:
  MinesweeperStore`; the view reads `.store.*` fields only; the `on` block
  forwards every event to the matching store action.

## How to Run

```
cd examples/ui/038-minesweeper
auto gen    # generate gen/front/vue (store composable + App SFC)

# vue backend (web)
auto run                 # serve frontend (:4038), open http://localhost:4038

# vm backend (native AutoVM window)
auto run --render vm     # interpret .at directly, no code generation
```

Left-click reveals, right-click flags. Switch difficulty with the three
buttons; click 🔄 to restart the current board.

## Why dual-backend matters

The vue backend generates a real Vue/Vite SPA; the vm backend interprets the
`.at` directly in a native window with no JS in between. Keeping the same
source working on both is a constraint that shapes the architecture: no
TypeScript escape-hatch, no function calls in view bindings, all logic in
store actions. See Plan 402 §12 for the full dual-backend analysis.

## Inspiration

The Windows Minesweeper classic. Chosen as a focused showcase for the
store-driven + dual-backend + event-modifier + timer pattern — a game whose
logic is too large to inline in a widget handler but whose UI is a clean
declarative grid.
