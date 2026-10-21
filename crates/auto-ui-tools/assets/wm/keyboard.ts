// Plan 465 T6: 桌面热键路由（R12）——document keydown 捕获段。
// 已消费组合键 stopImmediatePropagation 吞掉（T2 spike ③：App 自注册的
// document 监听天然跨窗广播，桌面热键必须走在捕获段）；未消费按键自然
// 落到焦点窗内元素。Tab 篱笆：手动循环焦点窗内可聚焦元素（不用 inert——
// 会杀死 WM 自己的窗框 chrome，T2 定案）。
import { wm, cycleFocus, setLayout, applyLayout, setViewport } from './store'

export interface DesktopHotkeyHooks {
  summonLauncher(): void
}

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

export function installDesktopKeyboard(hooks: DesktopHotkeyHooks): void {
  const onKeydown = (e: KeyboardEvent) => {
    // 捕获段桌面热键（R12）。
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault()
      e.stopImmediatePropagation()
      hooks.summonLauncher()
      return
    }
    if (e.altKey && e.key === 'Tab') {
      e.preventDefault()
      e.stopImmediatePropagation()
      cycleFocus()
      return
    }
    if (e.ctrlKey && e.altKey) {
      const mode = e.key === 'f' ? 'free' : e.key === 'g' ? 'grid' : e.key === 'm' ? 'master-stack' : null
      if (mode) {
        e.preventDefault()
        e.stopImmediatePropagation()
        setLayout(mode)
        return
      }
    }
    if (e.key === 'Tab') {
      // Tab 篱笆：限制在焦点窗内循环。
      const focused = wm.wins.find((w) => w.focused)
      if (!focused?.container) return
      const focusables = [...focused.container.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
        (el) => el.offsetParent !== null || el === document.activeElement,
      )
      if (focusables.length === 0) {
        e.preventDefault()
        return
      }
      const idx = focusables.indexOf(document.activeElement as HTMLElement)
      const next = e.shiftKey
        ? focusables[(idx - 1 + focusables.length) % focusables.length]
        : focusables[(idx + 1) % focusables.length]
      e.preventDefault()
      e.stopImmediatePropagation()
      next.focus()
    }
  }
  document.addEventListener('keydown', onKeydown, { capture: true })
  // resize → viewport 同步 + 非 free 布局重排。
  window.addEventListener('resize', () => {
    setViewport(window.innerWidth, window.innerHeight)
    if (wm.layoutMode !== 'free') applyLayout()
  })
}
