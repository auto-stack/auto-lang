// Plan 516 T2: store/remote 切片——remote 条目注册、断线保留、关闭销毁、
// 布局混排（本地+远程同权）。FakeWebSocket 驱动（helpers）。
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { close, openWindow, setLayout, setViewport, wm } from '../../assets/wm/store'
import { bootRemoteApps, openRemoteWindow, remote } from '../../assets/wm/remote'
import { FakeWebSocket, lastSocket, welcomeBytes } from './helpers'

beforeEach(() => {
  FakeWebSocket.instances = []
  setViewport(1280, 800)
})

afterEach(() => {
  for (const w of [...wm.wins]) close(w.wid)
  setLayout('free')
  vi.useRealTimers()
})

describe('T2 remote 条目注册（G2）', () => {
  it('openRemoteWindow 落 kind:"remote" 条目 + 会话切片 connecting', () => {
    const win = openRemoteWindow(
      { id: 'counter', url: 'ws://127.0.0.1:17800/?token=t', appId: '002-counter', title: 'Counter' },
      FakeWebSocket as unknown as typeof WebSocket,
    )
    expect(win.kind).toBe('remote')
    expect(win.appId).toBe('remote-counter')
    expect(win.title).toBe('Counter')
    const entry = wm.wins.find((w) => w.wid === win.wid)
    expect(entry).toBeDefined()
    // 级联初位在可用区内（本地窗同款 rect 语义）。
    expect(win.rect.width).toBeGreaterThan(0)
    expect(remote.sessions[win.wid]).toMatchObject({ status: 'connecting', appId: '002-counter' })
  })

  it('boot 批量建连 + 空配置零行为（G4）', () => {
    const before = wm.wins.length
    bootRemoteApps([])
    expect(wm.wins.length).toBe(before)
  })
})

describe('T2 断线保留与关闭销毁（G2）', () => {
  it('重连预算耗尽 → dead：窗口保留、状态面可见', () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'Date'] })
    const win = openRemoteWindow(
      {
        id: 'x',
        url: 'ws://127.0.0.1:1/?token=t',
        appId: 'a',
        title: 'a',
        reconnectBudgetMs: 1000,
      },
      FakeWebSocket as unknown as typeof WebSocket,
    )
    lastSocket().fireOpen()
    expect(remote.sessions[win.wid]!.status).toBe('connecting')
    lastSocket().fireMessage(welcomeBytes(7))
    expect(remote.sessions[win.wid]!.status).toBe('online')
    // 断线 → 预算内重连（reconnecting）→ 超预算 dead。
    vi.advanceTimersByTime(1200)
    lastSocket().fireClose()
    expect(remote.sessions[win.wid]!.status).toBe('dead')
    // 窗口不消失（断线保留）。
    expect(wm.wins.some((w) => w.wid === win.wid)).toBe(true)
  })

  it('close → unmount 会话回收（连接断开 + 切片清空）', () => {
    const win = openRemoteWindow(
      { id: 'y', url: 'ws://x/?token=t', appId: 'a', title: 'y' },
      FakeWebSocket as unknown as typeof WebSocket,
    )
    const ws = lastSocket()
    ws.fireOpen()
    ws.fireMessage(welcomeBytes(9))
    close(win.wid)
    expect(wm.wins.some((w) => w.wid === win.wid)).toBe(false)
    expect(remote.sessions[win.wid]).toBeUndefined()
    expect(ws.readyState).toBe(FakeWebSocket.CLOSED)
  })
})

describe('T2 布局混排（G2：本地+远程同权）', () => {
  it('grid：远程与本地窗各占一格（rect 由 store 统一分配）', () => {
    setLayout('grid')
    const local = openWindow('local-app', 'Local', 'app')
    const remoteWin = openRemoteWindow(
      { id: 'g', url: 'ws://x/?token=t', appId: 'a', title: 'g' },
      FakeWebSocket as unknown as typeof WebSocket,
    )
    expect(wm.wins.length).toBe(2)
    // grid N=2：cols=⌈√2⌉=2、rows=1——左右各半宽、整列高。
    for (const w of wm.wins) {
      expect(w.rect.width).toBeCloseTo(640, 0) // 1280/2
      expect(w.rect.height).toBeCloseTo(752, 0) // 800-48（单行整高）
    }
    // 两窗不重叠。
    const [a, b] = wm.wins
    const overlap =
      a!.rect.x < b!.rect.x + b!.rect.width &&
      b!.rect.x < a!.rect.x + a!.rect.width &&
      a!.rect.y < b!.rect.y + b!.rect.height &&
      b!.rect.y < a!.rect.y + a!.rect.height
    expect(overlap).toBe(false)
    expect(local.kind).toBe('app')
    expect(remoteWin.kind).toBe('remote')
  })
})
