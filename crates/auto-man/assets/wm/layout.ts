// Plan 465 T4: TS 直译 of auto-lang ui/layout.rs (463 R9 布局纯函数).
// I6 对拍: 同一期望值表 crates/auto-lang/src/ui/layout_cases.json 同时约束
// 本文件 (scripts/ui-layout-parity.mjs) 与 Rust 侧单测 — 修改布局语义必须
// 两侧同改 + 表同改。

export type LayoutModeName = 'free' | 'grid' | 'master-stack'

export interface Rect {
  x: number
  y: number
  width: number
  height: number
}

export interface WinSnap {
  wid: number
  rect: Rect
  focused: boolean
}

/** 任务栏高度（463 layout.rs TASKBAR_HEIGHT 同值；shell 装配共用）。 */
export const TASKBAR_HEIGHT = 48
/** master 宽占比（462/463 §3.1 定参）。 */
export const MASTER_RATIO = 0.55
/** snap 触发带宽度（光标距可用区左/右缘 ≤ 此值触发半屏预览）。 */
export const SNAP_ZONE = 8

export function layoutModeFromName(s: string): LayoutModeName {
  const t = s.trim()
  if (t === 'grid') return 'grid'
  if (t === 'master-stack' || t === 'masterstack' || t === 'master_stack') return 'master-stack'
  return 'free'
}

function makeRect(x: number, y: number, width: number, height: number): Rect {
  return { x, y, width, height }
}

/** viewport 扣除 shell 预留后的布局可用区（底部任务栏内缩；防负尺寸钳 0）。 */
export function usableRect(viewport: Rect, reservedBottom: number): Rect {
  return makeRect(
    viewport.x,
    viewport.y,
    Math.max(viewport.width, 0),
    Math.max(viewport.height - reservedBottom, 0),
  )
}

function cellRect(usable: Rect, col: number, row: number, cw: number, ch: number): Rect {
  return makeRect(usable.x + cw * col, usable.y + ch * row, cw, ch)
}

/** Grid：cols = ⌈√N⌉，rows = ⌈N/cols⌉，行主序。空表返回空表。 */
export function layoutGrid(wins: WinSnap[], usable: Rect): Rect[] {
  const n = wins.length
  if (n === 0 || usable.width <= 0 || usable.height <= 0) return []
  const cols = Math.ceil(Math.sqrt(n))
  const rows = Math.ceil(n / cols)
  const cw = usable.width / cols
  const ch = usable.height / rows
  return wins.map((w, i) => cellRect(usable, i % cols, Math.floor(i / cols), cw, ch))
}

/** MasterStack：焦点窗（无焦点取输入序末位=z 顶）左 master，其余右列均分。 */
export function layoutMasterStack(wins: WinSnap[], usable: Rect): Rect[] {
  const n = wins.length
  if (n === 0 || usable.width <= 0 || usable.height <= 0) return []
  // 单窗语义：独占整个可用区。
  if (n === 1) return [makeRect(usable.x, usable.y, usable.width, usable.height)]
  const found = wins.findIndex((w) => w.focused)
  const masterIdx = found === -1 ? n - 1 : found
  const masterW = usable.width * MASTER_RATIO
  const stackW = usable.width - masterW
  const rest = n - 1
  const sh = rest > 0 ? usable.height / rest : 0
  return wins.map((_, i) => {
    if (i === masterIdx) return makeRect(usable.x, usable.y, masterW, usable.height)
    const slot = i < masterIdx ? i : i - 1
    return makeRect(usable.x + masterW, usable.y + sh * slot, stackW, sh)
  })
}

/**
 * 布局主入口：free = 恒等（用户位置即真值）；reservedBottom = 任务栏预留
 * （463 ReservedEdges::taskbar 的 v1 单边形态）。
 */
export function layout(mode: LayoutModeName, wins: WinSnap[], viewport: Rect, reservedBottom: number): Rect[] {
  const usable = usableRect(viewport, reservedBottom)
  switch (mode) {
    case 'grid':
      return layoutGrid(wins, usable)
    case 'master-stack':
      return layoutMasterStack(wins, usable)
    default:
      return wins.map((w) => w.rect)
  }
}

/** free 模式新窗级联初位（459 先例：80 + 48i，限制在可用区内不全出界）。 */
export function cascadeRect(index: number, size: { width: number; height: number }, usable: Rect): Rect {
  const offX = Math.max(Math.min(80 + 48 * index, usable.width * 0.5), 0)
  const offY = Math.max(Math.min(80 + 48 * index, usable.height * 0.5), 0)
  return makeRect(
    usable.x + offX,
    usable.y + offY,
    Math.min(size.width, usable.width),
    Math.min(size.height, usable.height),
  )
}

/** Snap 预览几何（free 拖拽中）：光标进入左/右缘 SNAP_ZONE → 对应半屏；否则 null。 */
export function snapPreview(cursor: { x: number; y: number }, usable: Rect): Rect | null {
  if (usable.width <= 0 || usable.height <= 0) return null
  const half = usable.width / 2
  if (cursor.x - usable.x <= SNAP_ZONE) return makeRect(usable.x, usable.y, half, usable.height)
  if (usable.x + usable.width - cursor.x <= SNAP_ZONE) {
    return makeRect(usable.x + half, usable.y, usable.width - half, usable.height)
  }
  return null
}
