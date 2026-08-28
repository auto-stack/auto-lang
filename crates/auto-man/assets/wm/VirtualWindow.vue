<script setup lang="ts">
// Plan 465 T4: virtual_window DOM 叶（R4 AppWindow 叶枚举的 DOM 臂）。
// 定位/clip/chrome/八向缩放把手 + 拖拽/缩放（pointer capture 改 store.rect）。
// iced 对应实现 = auto-lang ui/iced/virtual_window.rs（462）——两端同源登记
// schema/aura.at（I4）。语义规范（E1 (AppId,event) 注入形状 / E2 叶枚举）:
// docs/plans/reports/465-t4-wm-dom-leaf.md。
import { computed } from 'vue'
import { wm, focus, type WinEntry } from './store'

const props = defineProps<{
  /** WmStore 的窗口条目（reactive；rect/z/focused 直读）。 */
  win: WinEntry
}>()

const style = computed(() => {
  const r = props.win.rect
  return {
    left: `${r.x}px`,
    top: `${r.y}px`,
    width: `${r.width}px`,
    height: `${r.height}px`,
    zIndex: props.win.z,
  }
})

const DIRS = ['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw'] as const

let drag: {
  kind: 'move' | 'resize'
  dir: string
  startX: number
  startY: number
  rect: Rect
} | null = null

function onPointerDown(e: PointerEvent, kind: 'move' | 'resize', dir: string): void {
  if (!props.win.focused) focus(props.win.wid)
  // 阻断穿透（与 iced 吞噬语义对齐：命中窗内即停，不落桌面层）。
  e.stopPropagation()
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  const r = props.win.rect
  drag = { kind, dir, startX: e.clientX, startY: e.clientY, rect: { ...r } }
}

function onPointerMove(e: PointerEvent): void {
  if (!drag) return
  const dx = e.clientX - drag.startX
  const dy = e.clientY - drag.startY
  const r = props.win.rect
  if (drag.kind === 'move') {
    r.x = drag.rect.x + dx
    r.y = drag.rect.y + dy
    return
  }
  const d = drag.dir
  const min = 160
  if (d.includes('e')) r.width = Math.max(min, drag.rect.width + dx)
  if (d.includes('s')) r.height = Math.max(min, drag.rect.height + dy)
  if (d.includes('w')) {
    const w = Math.max(min, drag.rect.width - dx)
    r.x = drag.rect.x + (drag.rect.width - w)
    r.width = w
  }
  if (d.includes('n')) {
    const h = Math.max(min, drag.rect.height - dy)
    r.y = drag.rect.y + (drag.rect.height - h)
    r.height = h
  }
}

function onPointerUp(): void {
  drag = null
}

function onClose(): void {
  wm.wins.splice(
    wm.wins.findIndex((w) => w.wid === props.win.wid),
    1,
  )
  // 生命周期收尾（unmount + 容器移除）走 store.close —— host shell 监听
  // wm-close 再调 store.close(wid)，这里只发事件（E1: (AppId, event) 注入）。
}
</script>

<template>
  <section
    class="virtual-window absolute flex flex-col overflow-hidden rounded-md border bg-card shadow-lg"
    :class="win.focused ? 'border-primary' : 'border-border'"
    :style="style"
    @pointerdown="focus(win.wid)"
  >
    <header
      class="title-bar h-8 flex items-center gap-2 px-2 bg-card border-b select-none cursor-move shrink-0"
      @pointerdown="onPointerDown($event, 'move', '')"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @dblclick="onClose"
    >
      <span class="text-xs text-muted-foreground truncate flex-1">{{ win.title }}</span>
      <button
        class="wm-close h-5 w-5 text-xs rounded border border-border hover:bg-accent"
        aria-label="close"
        @click.stop="onClose"
      >
        ×
      </button>
    </header>
    <div class="client flex-1 relative overflow-hidden">
      <slot />
    </div>
    <template v-for="d in DIRS" :key="d">
      <div
        class="wm-resize absolute"
        :data-dir="d"
        :class="{
          'top-0 left-0 right-0 h-1 cursor-n-resize': d === 'n',
          'bottom-0 left-0 right-0 h-1 cursor-s-resize': d === 's',
          'top-0 bottom-0 right-0 w-1 cursor-e-resize': d === 'e',
          'top-0 bottom-0 left-0 w-1 cursor-w-resize': d === 'w',
          'top-0 right-0 w-3 h-3 cursor-ne-resize': d === 'ne',
          'top-0 left-0 w-3 h-3 cursor-nw-resize': d === 'nw',
          'bottom-0 right-0 w-3 h-3 cursor-se-resize': d === 'se',
          'bottom-0 left-0 w-3 h-3 cursor-sw-resize': d === 'sw',
        }"
        @pointerdown="onPointerDown($event, 'resize', d)"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      />
    </template>
  </section>
</template>
