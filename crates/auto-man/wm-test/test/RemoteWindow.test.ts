// Plan 516 T1: RemoteWindow 组件——状态机（连接中/在线/重连/失败面）、
// rect→canvas 位图（Welcome 尺寸 × DPR）、帧渲染（rAF 末帧）、配置缺省
// 零渲染。jsdom 无 Canvas2D 实现——getContext 桩为录制器。
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { close, openWindow, setViewport, wm } from '../../assets/wm/store'
import { openRemoteWindow, remote } from '../../assets/wm/remote'
import RemoteWindow from '../../assets/wm/RemoteWindow.vue'
import { FakeWebSocket, frameBytes, hitTableBytes, lastSocket, nextFrame, welcomeBytes } from './helpers'

interface Ctx2D {
  setTransform: ReturnType<typeof vi.fn>
  fillRect: ReturnType<typeof vi.fn>
  fillText: ReturnType<typeof vi.fn>
}

let ctxLog: Ctx2D

function mountRemote(id: string) {
  const win = openRemoteWindow(
    { id, url: 'ws://127.0.0.1:17800/?token=t', appId: '002-counter', title: 'Counter' },
    FakeWebSocket as unknown as typeof WebSocket,
  )
  const wrapper = mount(RemoteWindow, { props: { win } })
  return { win, wrapper }
}

beforeEach(() => {
  FakeWebSocket.instances = []
  setViewport(1280, 800)
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (this: HTMLCanvasElement) {
    ctxLog = {
      canvas: this,
      setTransform: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
    }
    return ctxLog as unknown as CanvasRenderingContext2D
  })
  vi.spyOn(HTMLCanvasElement.prototype, 'getBoundingClientRect').mockReturnValue({
    left: 0,
    top: 0,
    width: 480,
    height: 320,
    x: 0,
    y: 0,
    right: 480,
    bottom: 320,
    toJSON: () => ({}),
  } as DOMRect)
})

afterEach(() => {
  vi.restoreAllMocks()
  for (const w of [...wm.wins]) close(w.wid)
})

describe('T1 状态机（G1）', () => {
  it('初态 = 连接中面；Welcome 后在线、覆盖面消失', async () => {
    const { win, wrapper } = mountRemote('s1')
    expect(wrapper.find('[data-remote-status="connecting"]').exists()).toBe(true)
    lastSocket().fireOpen()
    lastSocket().fireMessage(welcomeBytes(5))
    expect(remote.sessions[win.wid]!.status).toBe('online')
    await nextTick()
    expect(wrapper.find('[data-remote-status]').exists()).toBe(false)
  })

  it('连接失败面：预算耗尽 dead + 原因可见', async () => {
    // toFake 限定 setTimeout/Date——rAF 留真（jsdom 帧循环被伪造会破坏
    // 后续测试的 rAF 派发）。
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'Date'] })
    const { win, wrapper } = mountRemote('s2')
    vi.advanceTimersByTime(31_000)
    lastSocket().fireClose()
    expect(remote.sessions[win.wid]!.status).toBe('dead')
    await nextTick()
    expect(wrapper.find('[data-remote-status="dead"]').text()).toContain('连接失败')
  })

  it('会话缺失（异常态）零渲染不崩', () => {
    // 直接落 kind:"remote" 条目而无会话切片（boot 降级外的异常面）。
    const win = openWindow('remote-ghost', 'Ghost', 'remote')
    const wrapper = mount(RemoteWindow, { props: { win } })
    expect(wrapper.find('canvas').exists()).toBe(true)
    expect(wrapper.find('[data-remote-status]').exists()).toBe(false)
  })
})

describe('T1 帧渲染（G1：rect→canvas + rAF 末帧）', () => {
  it('Welcome 尺寸定 canvas 位图；帧文本落画布', async () => {
    const { win, wrapper } = mountRemote('f1')
    lastSocket().fireOpen()
    lastSocket().fireMessage(welcomeBytes(5, 640, 400))
    lastSocket().fireMessage(frameBytes(5, 1, 1, 'Counter: 0'))
    await nextFrame()
    const canvas = wrapper.get('canvas').element as HTMLCanvasElement
    expect(canvas.width).toBe(640) // jsdom DPR = 1
    expect(canvas.height).toBe(400)
    expect(ctxLog.fillText).toHaveBeenCalledWith('Counter: 0', 12, 8)
    // 多帧取末帧（rAF 合帧）。
    lastSocket().fireMessage(frameBytes(5, 2, 2, 'Counter: 1'))
    lastSocket().fireMessage(frameBytes(5, 3, 3, 'Counter: 2'))
    await nextFrame()
    expect(ctxLog.fillText).toHaveBeenLastCalledWith('Counter: 2', 12, 8)
    expect(remote.sessions[win.wid]!.lastTexts).toEqual(['Counter: 2'])
    expect(remote.sessions[win.wid]!.revision).toBe(3)
  })
})

describe('T1 e2e 探针', () => {
  it('buttonCenters = 按钮命中区中心（CSS 像素）', () => {
    const { win } = mountRemote('p1')
    lastSocket().fireOpen()
    lastSocket().fireMessage(welcomeBytes(5))
    lastSocket().fireMessage(
      hitTableBytes(5, [
        { x: 100, y: 200, w: 60, h: 30, kind: 1, action: 'inc' },
        { x: 10, y: 10, w: 50, h: 20, kind: 2, action: 'field' },
      ]),
    )
    const probe = (window as unknown as Record<string, Record<string, { buttonCenters(): Array<{ x: number }> }>>).__p516?.[win.wid]
    expect(probe).toBeDefined()
    expect(probe!.buttonCenters()).toEqual([{ x: 130, y: 215 }])
  })
})
