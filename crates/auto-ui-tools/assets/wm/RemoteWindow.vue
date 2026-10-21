<script setup lang="ts">
// Plan 516: remote_window DOM 叶——VirtualWindow 同级的 wm 族组件（宿主
// 内置面，v1 不做 .at 登记——计划待澄清①）。canvas 位图 = 远端逻辑尺寸 ×
// DPR（ctx 变换后按帧空间绘制），CSS 铺满客户区；状态面（连接中/在线/
// 重连中/失败）覆盖在 canvas 上——断线窗口保留（G2）。输入：指针命中
// HitTable 才回发（本地寻址语义；权威命中在 app 侧）；可打印字符经
// InputMsg 回发（508 demo 页同口径）。帧节奏 = rAF 合帧取末帧（计划③）。
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { focus } from './store'
import { peekFrame, remote, sendChar, sendPointerDown, type RemoteSessionState } from './remote'
import { hitTest, renderFrame } from './remote-renderer/index.ts'

const props = defineProps<{
  /** WmStore 的远程窗条目（kind:"remote"；rect/z/focused 直读）。 */
  win: import('./store').WinEntry
}>()

const session = computed<RemoteSessionState | undefined>(() => remote.sessions[props.win.wid])
const canvasEl = ref<HTMLCanvasElement | null>(null)

/** e2e 断言探针（Playwright 读取；生产无副作用——508 demo 页同惯例）。 */
const probe = {
  wid: props.win.wid,
  frames: 0,
  revision: 0,
  welcome: false,
  status: 'connecting' as string,
  lastTexts: [] as string[],
  /** 按钮命中区中心（CSS 像素；e2e 点击寻址用）。 */
  buttonCenters(): Array<{ x: number; y: number }> {
    const st = session.value
    const canvas = canvasEl.value
    if (!st || !canvas) return []
    const r = canvas.getBoundingClientRect()
    const sx = r.width / st.frameW
    const sy = r.height / st.frameH
    return st.hits
      .filter((h) => h.kind === 1)
      .map((h) => ({
        x: r.left + (h.rect.x + h.rect.w / 2) * sx,
        y: r.top + (h.rect.y + h.rect.h / 2) * sy,
      }))
  },
}
;(window as unknown as Record<string, unknown>).__p516 = {
  ...((window as unknown as Record<string, unknown>).__p516 as object | undefined),
  [props.win.wid]: probe,
}

// ---- 帧渲染：frames 计数变更 → rAF 合帧取末帧（多帧取末，丢中间）。 ----
let rafId = 0

function renderLatest(): void {
  rafId = 0
  const st = session.value
  const canvas = canvasEl.value
  const frame = peekFrame(props.win.wid)
  if (!st || !canvas || !frame) return
  const dpr = (typeof window !== 'undefined' && window.devicePixelRatio) || 1
  const bw = Math.round(st.frameW * dpr)
  const bh = Math.round(st.frameH * dpr)
  if (canvas.width !== bw || canvas.height !== bh) {
    canvas.width = bw
    canvas.height = bh
  }
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  renderFrame(ctx, frame)
}

watch(
  () => session.value?.frames ?? 0,
  () => {
    if (!rafId) rafId = requestAnimationFrame(renderLatest)
  },
)

watch(
  () => session.value?.status,
  (s) => {
    probe.status = s ?? 'connecting'
  },
  { immediate: true },
)

watch(
  () => [session.value?.frames, session.value?.revision, session.value?.lastTexts] as const,
  ([frames, revision, texts]) => {
    probe.frames = frames ?? 0
    probe.revision = revision ?? 0
    probe.lastTexts = texts ?? []
    if ((frames ?? 0) > 0) probe.welcome = true
  },
)

onBeforeUnmount(() => {
  if (rafId) cancelAnimationFrame(rafId)
})

// ---- 输入：指针（CSS → 帧空间映射；命中才回发）+ 字符（可打印）。 ----
function onPointerDown(e: PointerEvent): void {
  focus(props.win.wid)
  const st = session.value
  const canvas = canvasEl.value
  if (!st || !canvas) return
  const r = canvas.getBoundingClientRect()
  const x = ((e.clientX - r.left) * st.frameW) / r.width
  const y = ((e.clientY - r.top) * st.frameH) / r.height
  if (hitTest(st.hits, x, y)) sendPointerDown(props.win.wid, x, y)
}

function onKeydown(e: KeyboardEvent): void {
  // 桌面热键已在捕获段消费（keyboard.ts）；此处只回发可打印字符
  // （508 demo 页同口径——渲染器包 InputMsg 子集）。
  if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
    sendChar(props.win.wid, e.key)
  }
}

const overlay = computed((): { label: string; err: boolean } | null => {
  const st = session.value
  if (!st || st.status === 'online') return null
  if (st.status === 'connecting') return { label: '连接中…', err: false }
  if (st.status === 'reconnecting') return { label: '连接断开，重连中…', err: false }
  return { label: `连接失败：${st.failReason || 'unreachable'}`, err: true }
})
</script>

<template>
  <div class="w-full h-full relative overflow-hidden bg-background outline-none">
    <canvas
      ref="canvasEl"
      class="remote-canvas block w-full h-full"
      tabindex="0"
      @pointerdown="onPointerDown"
      @keydown="onKeydown"
    />
    <div
      v-if="overlay"
      class="remote-overlay absolute inset-0 flex flex-col items-center justify-center gap-2 bg-background/85 select-none"
      :class="overlay.err ? 'text-destructive' : 'text-muted-foreground'"
      :data-remote-status="session?.status ?? 'connecting'"
    >
      <span class="text-sm">{{ overlay.label }}</span>
      <span class="text-xs opacity-70">{{ session?.appId }} · {{ session?.title }}</span>
    </div>
  </div>
</template>
