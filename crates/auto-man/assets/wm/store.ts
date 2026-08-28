// Plan 465 T4: WmStore — 桌面 WM 状态（vue 叶）。462 iced WmState 的 TS 对应物:
// rect/z/focus 的唯一写点；排布策略经 layout.ts 纯函数（R9：布局是策略，
// store 是状态）。事件路由语义（E1 (AppId, event) 注入形状）见
// docs/plans/reports/465-t4-wm-dom-leaf.md。
import { reactive } from 'vue'
import { layout, cascadeRect, usableRect, TASKBAR_HEIGHT, type LayoutModeName, type Rect } from './layout'

export interface WinEntry {
  wid: number
  appId: string
  title: string
  rect: Rect
  z: number
  focused: boolean
  /** registry load() 的组件实例挂载容器（T5 生命周期管理用）。 */
  container: HTMLElement | null
  app: { unmount: () => void } | null
}

let nextWid = 1
let nextZ = 1

const viewport: Rect = { x: 0, y: 0, width: 0, height: 0 }

export const wm = reactive({
  wins: [] as WinEntry[],
  layoutMode: 'free' as LayoutModeName,
  focusedWid: null as number | null,
})

/** 桌面根元素就位后调用一次（resize 监听在 keyboard.ts / host shell 接线）。 */
export function setViewport(w: number, h: number): void {
  viewport.width = w
  viewport.height = h
}

export function usable(): Rect {
  return usableRect(viewport, TASKBAR_HEIGHT)
}

export function focus(wid: number): void {
  for (const w of wm.wins) w.focused = w.wid === wid
  wm.focusedWid = wid
}

/** 命中测试 → 焦点（E1 注入形状的指针半边：矩形含点 → 归属 App → 聚焦）。 */
export function focusAtPoint(x: number, y: number): WinEntry | null {
  // z 序从顶到底找第一个含点窗口。
  const sorted = [...wm.wins].sort((a, b) => b.z - a.z)
  for (const w of sorted) {
    const r = w.rect
    if (x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height) {
      focus(w.wid)
      return w
    }
  }
  return null
}

/** 布局纯函数的批量写点（rect 的唯一批量写点，同 463 apply_layout）。 */
export function applyLayout(): void {
  const snaps = wm.wins.map((w) => ({ wid: w.wid, rect: w.rect, focused: w.focused }))
  const out = layout(wm.layoutMode, snaps, viewport, TASKBAR_HEIGHT)
  for (let i = 0; i < out.length; i++) {
    const w = wm.wins.find((e) => e.wid === snaps[i].wid)
    if (w) w.rect = { ...out[i] }
  }
}

export function launchWindow(appId: string, title: string, container: HTMLElement, app: { unmount: () => void }): WinEntry {
  const usableArea = usable()
  const rect = cascadeRect(wm.wins.length, { width: 640, height: 480 }, usableArea)
  const win: WinEntry = {
    wid: nextWid++,
    appId,
    title,
    rect,
    z: nextZ++,
    focused: true,
    container,
    app,
  }
  wm.wins.push(win)
  focus(win.wid)
  if (wm.layoutMode !== 'free') applyLayout()
  return win
}

/** 关窗 = unmount + 容器移除（459「窗关 App 随之退」）；焦点让渡给 z 次顶。 */
export function close(wid: number): void {
  const idx = wm.wins.findIndex((w) => w.wid === wid)
  if (idx === -1) return
  const [w] = wm.wins.splice(idx, 1)
  w.app?.unmount()
  w.container?.remove()
  if (wm.focusedWid === wid) {
    const top = [...wm.wins].sort((a, b) => b.z - a.z)[0]
    wm.focusedWid = top ? top.wid : null
    if (top) focus(top.wid)
  }
}

export function setLayout(mode: LayoutModeName): void {
  wm.layoutMode = mode
  if (mode !== 'free') applyLayout()
}

/** Alt+Tab：z 序轮转（顶 → 焦点）。 */
export function cycleFocus(): void {
  if (wm.wins.length < 2) return
  const sorted = [...wm.wins].sort((a, b) => a.z - b.z)
  const bottom = sorted[0]
  // 把 z 最低的窗抬到顶 = 聚焦轮转。
  const maxZ = Math.max(...wm.wins.map((w) => w.z))
  bottom.z = maxZ + 1
  focus(bottom.wid)
}
