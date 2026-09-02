// Plan 516 T3: 输入路由——焦点远程窗的指针/键盘事件 → HitTable 命中 →
// InputMsg 编码回发断言（mock 连接）。编码正确性以渲染器包 golden 为准，
// 此处断言“WM 接线回发了与编码器同字节的消息”。
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { close, focus, openWindow, setViewport, wm } from '../../assets/wm/store'
import { openRemoteWindow } from '../../assets/wm/remote'
import RemoteWindow from '../../assets/wm/RemoteWindow.vue'
import {
  FakeWebSocket,
  frameBytes,
  hitTableBytes,
  lastSocket,
  nextFrame,
  welcomeBytes,
} from './helpers'
import {
  encodeCharTyped,
  encodePointerPressed,
} from '../../../../packages/drawlist-renderer/src/messages.ts'

const WELCOME_WID = 7 // 远端会话自身的 wid（Welcome 回填；回发消息携带）

beforeEach(() => {
  FakeWebSocket.instances = []
  setViewport(1280, 800)
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (this: HTMLCanvasElement) {
    return {
      canvas: this,
      setTransform: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
    } as unknown as CanvasRenderingContext2D
  })
  vi.spyOn(HTMLCanvasElement.prototype, 'getBoundingClientRect').mockReturnValue({
    left: 100,
    top: 50,
    width: 480,
    height: 320,
    x: 100,
    y: 50,
    right: 580,
    bottom: 370,
    toJSON: () => ({}),
  } as DOMRect)
})

afterEach(() => {
  vi.restoreAllMocks()
  for (const w of [...wm.wins]) close(w.wid)
})

async function mountOnline(id: string) {
  const win = openRemoteWindow(
    { id, url: 'ws://127.0.0.1:17800/?token=t', appId: '002-counter', title: 'Counter' },
    FakeWebSocket as unknown as typeof WebSocket,
  )
  const wrapper = mount(RemoteWindow, { props: { win } })
  const ws = lastSocket()
  ws.fireOpen()
  ws.fireMessage(welcomeBytes(WELCOME_WID))
  ws.fireMessage(
    hitTableBytes(WELCOME_WID, [
      { x: 100, y: 200, w: 60, h: 30, kind: 1, action: 'inc' },
    ]),
  )
  ws.fireMessage(frameBytes(WELCOME_WID, 1, 1, 'Counter: 0'))
  await nextFrame()
  ws.sent.length = 0 // 只留输入回发
  return { win, wrapper, ws }
}

// jsdom 无 PointerEvent 且 MouseEvent 坐标为只读 getter（test-utils
// trigger 赋值即抛）——真实事件构造器 dispatch。
function firePointer(el: Element, clientX: number, clientY: number): void {
  el.dispatchEvent(new MouseEvent('pointerdown', { clientX, clientY, bubbles: true }))
}

function fireKey(el: Element, key: string, init: KeyboardEventInit = {}): void {
  el.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...init }))
}

describe('T3 指针回发（G3）', () => {
  it('命中按钮区 → PointerPressed 编码同字节回发 + WM 聚焦', async () => {
    const { win, wrapper, ws } = await mountOnline('ptr')
    // 命中区中心 (130, 215)（帧空间）；CSS 桩 rect 左上 (100,50) → client 坐标。
    firePointer(wrapper.get('canvas').element, 100 + 130, 50 + 215)
    expect(ws.sent.length).toBe(1)
    expect(Array.from(ws.sent[0]!)).toEqual(Array.from(encodePointerPressed(BigInt(WELCOME_WID), 130, 215)))
    // 点击即聚焦（WM 同权）。
    expect(wm.focusedWid).toBe(win.wid)
    expect(win.focused).toBe(true)
  })

  it('空白区不回发（本地寻址语义）', async () => {
    const { wrapper, ws } = await mountOnline('ptr2')
    firePointer(wrapper.get('canvas').element, 100 + 5, 50 + 5)
    expect(ws.sent.length).toBe(0)
  })
})

describe('T3 键盘回发（G3）', () => {
  it('可打印字符 → CharTyped 同字节回发；组合键/控制键不回发', async () => {
    const { wrapper, ws } = await mountOnline('key')
    const canvas = wrapper.get('canvas').element
    fireKey(canvas, 'a')
    fireKey(canvas, 'Enter')
    fireKey(canvas, 'b', { ctrlKey: true })
    expect(ws.sent.length).toBe(1)
    expect(Array.from(ws.sent[0]!)).toEqual(Array.from(encodeCharTyped(BigInt(WELCOME_WID), 'a')))
  })

  it('焦点语义：远程窗聚焦不受本地窗影响（store 直查）', async () => {
    const { win } = await mountOnline('key2')
    const local = openWindow('local-app', 'Local', 'app')
    focus(local.wid)
    expect(win.focused).toBe(false)
    focus(win.wid)
    expect(wm.focusedWid).toBe(win.wid)
  })
})
