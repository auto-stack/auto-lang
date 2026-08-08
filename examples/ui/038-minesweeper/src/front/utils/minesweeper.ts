// Hand-written TypeScript minesweeper game logic, consumed by the widget via
// the `use { fn: ... }` escape-hatch. Copied into the generated Vue project
// as src/ext/src/front/utils/minesweeper.ts and imported from the SFC as
// `@/ext/src/front/utils/minesweeper` (see 029-external-imports).
//
// Keeping the heavy algorithm here in TS is the idiomatic AutoUI pattern when
// logic is large: the .at widget stays a thin state shell (model + view +
// event routing), and the imperative game logic lives in plain TypeScript.

export interface Cell {
  x: number
  y: number
  mine: boolean
  revealed: boolean
  flagged: boolean
  adjacent: number
}

/// Number of rows for a difficulty ("beginner" | "intermediate" | "expert").
export function difficulty_rows(difficulty: string): number {
  if (difficulty === 'intermediate') return 16
  if (difficulty === 'expert') return 16
  return 9
}

/// Number of columns for a difficulty.
export function difficulty_cols(difficulty: string): number {
  if (difficulty === 'intermediate') return 16
  if (difficulty === 'expert') return 30
  return 9
}

/// Number of mines for a difficulty.
export function difficulty_mines(difficulty: string): number {
  if (difficulty === 'intermediate') return 40
  if (difficulty === 'expert') return 99
  return 10
}

/// Build a fresh, all-empty board (no mines placed yet) of size rows x cols.
/// Mines are placed later by place_mines on the first click (first-click-safe).
export function init_board(rows: number, cols: number): Cell[] {
  const cells: Cell[] = []
  for (let x = 0; x < rows; x++) {
    for (let y = 0; y < cols; y++) {
      cells.push({ x, y, mine: false, revealed: false, flagged: false, adjacent: 0 })
    }
  }
  return cells
}

/// Is (rx, ry) inside the 3x3 safe zone centered on (sx, sy)?
function in_safe_zone(rx: number, ry: number, sx: number, sy: number): boolean {
  return Math.abs(rx - sx) <= 1 && Math.abs(ry - sy) <= 1
}

/// Place mines avoiding the 3x3 safe zone around (safe_x, safe_y), then
/// compute the adjacency count for every non-mine cell. Returns a NEW board
/// with mines laid out (first-click-safe).
export function place_mines(
  board: Cell[],
  rows: number,
  cols: number,
  mine_count: number,
  safe_x: number,
  safe_y: number,
): Cell[] {
  const b: Cell[] = board.map((c) => ({ ...c }))

  // Place mines with rejection sampling: pick a random cell, skip it if it
  // is in the safe zone or already a mine. Cap attempts to avoid an infinite
  // loop when the board is too small for mine_count + safe zone.
  let placed = 0
  let attempts = 0
  const max_attempts = mine_count * 50 + 100
  while (placed < mine_count && attempts < max_attempts) {
    attempts++
    const r = Math.floor(Math.random() * rows * cols)
    const rx = Math.floor(r / cols)
    const ry = r % cols
    if (in_safe_zone(rx, ry, safe_x, safe_y)) continue
    const idx = rx * cols + ry
    if (b[idx].mine) continue
    b[idx].mine = true
    placed++
  }

  // Compute adjacency counts for each non-mine cell.
  for (let i = 0; i < b.length; i++) {
    if (b[i].mine) continue
    let count = 0
    for (let dx = -1; dx <= 1; dx++) {
      for (let dy = -1; dy <= 1; dy++) {
        if (dx === 0 && dy === 0) continue
        const nx = b[i].x + dx
        const ny = b[i].y + dy
        if (nx < 0 || nx >= rows || ny < 0 || ny >= cols) continue
        if (b[nx * cols + ny].mine) count++
      }
    }
    b[i].adjacent = count
  }

  return b
}

/// Flood-fill reveal starting from (start_x, start_y): reveal the start cell
/// and, if it has 0 adjacent mines, iteratively reveal all connected empty
/// cells up to (and including) the numbered border. Uses an explicit stack
/// (no recursion) to avoid any call-depth limits. Flags are respected: a
/// flagged cell is never auto-revealed.
export function reveal_flood(
  board: Cell[],
  rows: number,
  cols: number,
  start_x: number,
  start_y: number,
): Cell[] {
  const b: Cell[] = board.map((c) => ({ ...c }))

  // Flat coordinate stack: [x0, y0, x1, y1, ...].
  const stack: number[] = [start_x, start_y]
  while (stack.length > 0) {
    const y = stack.pop()!
    const x = stack.pop()!
    if (x < 0 || x >= rows || y < 0 || y >= cols) continue
    const idx = x * cols + y
    const c = b[idx]
    if (c.revealed || c.flagged) continue
    c.revealed = true
    // If empty (0 neighbors), push its 8 neighbors so the flood continues.
    if (c.adjacent === 0) {
      for (let dx = -1; dx <= 1; dx++) {
        for (let dy = -1; dy <= 1; dy++) {
          if (dx === 0 && dy === 0) continue
          stack.push(x + dx, y + dy)
        }
      }
    }
  }

  return b
}

/// Has every non-mine cell been revealed? (Win condition.)
export function check_win(board: Cell[]): boolean {
  for (const c of board) {
    if (!c.mine && !c.revealed) return false
  }
  return true
}

/// Count how many flags are currently placed on the board.
export function count_flags(board: Cell[]): number {
  let n = 0
  for (const c of board) if (c.flagged) n++
  return n
}

/// Toggle the flag on cell (x, y). Revealed cells cannot be flagged.
/// Returns a NEW board with the flag toggled.
export function toggle_flag(board: Cell[], cols: number, x: number, y: number): Cell[] {
  return board.map((c) => {
    if (c.x === x && c.y === y && !c.revealed) {
      return { ...c, flagged: !c.flagged }
    }
    return c
  })
}

/// Reveal every mine on the board (used on loss, to show the full picture).
/// Returns a NEW board with all mines revealed.
export function reveal_all_mines(board: Cell[]): Cell[] {
  return board.map((c) => (c.mine ? { ...c, revealed: true } : c))
}

/// Inline CSS color for a cell given its number (1..8) — classic minesweeper
/// number colors. Returns a real CSS string because the codegen binds `text`
/// element attributes through `:style` (Tailwind class names are not valid
/// inline CSS and would be silently dropped). Font size/weight are handled by
/// the surrounding revealed-cell styling.
export function number_class(n: number): string {
  switch (n) {
    case 1:
      return 'color:#1d4ed8;font-weight:700;font-size:1.125rem'
    case 2:
      return 'color:#15803d;font-weight:700;font-size:1.125rem'
    case 3:
      return 'color:#b91c1c;font-weight:700;font-size:1.125rem'
    case 4:
      return 'color:#6b21a8;font-weight:700;font-size:1.125rem'
    case 5:
      return 'color:#9a3412;font-weight:700;font-size:1.125rem'
    case 6:
      return 'color:#0f766e;font-weight:700;font-size:1.125rem'
    case 7:
      return 'color:#111827;font-weight:700;font-size:1.125rem'
    default:
      return 'color:#9d174d;font-weight:700;font-size:1.125rem'
  }
}

/// Tailwind class for a difficulty button — highlighted when it is the
/// active difficulty. (Kept in TS so the view avoids fragile inline
/// string + ternary concatenation, which the codegen renders without the
/// protective parentheses the ?: operator needs.)
export function difficulty_class(difficulty: string, target: string): string {
  const base = 'px-3 py-1.5 rounded-lg text-sm font-semibold border '
  if (difficulty === target) {
    return base + 'bg-blue-600 text-white border-blue-700'
  }
  return base + 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
}

/// Tailwind class for a board cell — flat when revealed, raised otherwise.
/// Uses a strong light/dark split so revealed vs. hidden cells are
/// unambiguous: hidden cells are a solid mid-gray "button" look, revealed
/// cells drop to white with a faint border.
export function cell_class(revealed: boolean): string {
  const base = 'w-8 h-8 flex items-center justify-center select-none '
  if (revealed) {
    return base + 'bg-white border border-gray-200'
  }
  return base + 'bg-gray-400 hover:bg-gray-500 border border-gray-500'
}

/// "💣 N" label for the mines-left counter. Returned as a single string so
/// the view can render it via one clean function call (the codegen mangles
/// `text` elements that mix string literals with arithmetic / method calls).
export function mines_left_label(mine_count: number, flags_placed: number): string {
  return `💣 ${mine_count - flags_placed}`
}

/// "⏱ Ns" label for the elapsed-time counter.
export function time_label(elapsed: number): string {
  return `⏱ ${elapsed}s`
}

/// Inline CSS for the board grid: a dynamic column count. The view's `cols:`
/// prop only renders as a Tailwind `grid-cols-N` class (capped at 12), so for
/// minesweeper's variable widths (9/16/30) we emit a real
/// `grid-template-columns: repeat(N, ...)` via a `:style` binding instead.
export function grid_style(cols: number): string {
  return `display:grid; grid-template-columns: repeat(${cols}, minmax(0, 1fr)); gap:0;`
}
