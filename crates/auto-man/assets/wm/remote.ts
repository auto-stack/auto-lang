// Plan 516: 远程会话切片 —— remote_window 的 WM 接线层。会话生命周期与
// 虚拟窗绑定（v1 一窗一会话，计划②）：openRemoteWindow 落 store 条目
// （kind:"remote"）并经 508 渲染器 connect() 建连；帧缓存/命中表/连接态
// 进本切片（DrawList 体量大，帧本体走非 reactive 缓存，组件按序号取末帧）。
// 渲染器包零改动——connect 输入回发/重连语义原样消费。
import { reactive } from 'vue'
import { openWindow, setRemoteCleanup, type WinEntry } from './store'
import { connect, type DrawList, type HitRegion, type RemoteConnection } from './remote-renderer/index.ts'

/** 连接状态面（508 ReconnectPolicy 语义：预算内重试，耗尽判 dead）。 */
export type RemoteStatus = 'connecting' | 'online' | 'reconnecting' | 'dead'

/** v1 会话配置（G4）：桌面配置/URL 注入的共同形态。 */
export interface RemoteAppConfig {
  /** 会话标识（窗口 appId 取 remote-<id>；任务栏/调试定位用）。 */
  id: string
  /** 完整 WS URL（含 token query：`ws://127.0.0.1:17800/?token=…`）。 */
  url: string
  /** 订阅目标（Hello.appName——远端 app 名）。 */
  appId: string
  title: string
  icon?: string
  /** 重连预算覆写（ms；e2e 断线场景缩短观察窗；缺省 508 的 30s）。 */
  reconnectBudgetMs?: number
}

export interface RemoteSessionState {
  /** 虚拟窗 wid（会话键；与远端 Welcome.wid 无关）。 */
  wid: number
  status: RemoteStatus
  appId: string
  title: string
  /** 远端逻辑尺寸（Welcome.rect；canvas 位图与命中坐标的源空间）。 */
  frameW: number
  frameH: number
  frames: number
  revision: number
  /** 帧文本探针（e2e 断言；渲染不依赖）。 */
  lastTexts: string[]
  hits: HitRegion[]
  failReason: string
}

export const remote = reactive({
  sessions: {} as Record<number, RemoteSessionState>,
})

/** 帧本体缓存（wid → 末帧；rAF 合帧取末帧，多帧丢弃中间帧）。 */
const frameCache = new Map<number, DrawList>()
const conns = new Map<number, RemoteConnection>()

/** 取末帧渲染（组件 rAF 回调消费；无帧 null——状态面仍可见）。 */
export function peekFrame(wid: number): DrawList | null {
  return frameCache.get(wid) ?? null
}

/** 输入回发（坐标 = 远端帧空间；权威命中在 app 侧——渲染器包语义）。 */
export function sendPointerDown(wid: number, x: number, y: number): void {
  conns.get(wid)?.sendPointerDown(x, y)
}

export function sendChar(wid: number, ch: string): void {
  conns.get(wid)?.sendChar(ch)
}

/** 会话在线（输入回发仅在有 wid 后生效；状态面同源）。 */
export function isOnline(wid: number): boolean {
  return conns.get(wid)?.connected ?? false
}

/**
 * 开一个远程窗（G1/G2/G4 共同入口）：store 条目 + 会话切片 + connect 建连。
 * 建连同步失败（如非法 URL）→ 窗口仍出现，状态面落 dead（boot 不阻断）。
 * WS 注入口仅测试用（FakeWebSocket 注入；生产缺省全局 WebSocket）。
 */
export function openRemoteWindow(cfg: RemoteAppConfig, WS?: typeof WebSocket): WinEntry {
  const win = openWindow(`remote-${cfg.id}`, cfg.title, 'remote')
  // reactive 代理创建后持引用修改——onFrame/onWelcome 回调经此触发组件
  // 响应（持原始对象改不会触发代理的依赖通知——Vue 语义）。
  const st = reactive<RemoteSessionState>({
    wid: win.wid,
    status: 'connecting',
    appId: cfg.appId,
    title: cfg.title,
    frameW: 480,
    frameH: 320,
    frames: 0,
    revision: 0,
    lastTexts: [],
    hits: [],
    failReason: '',
  })
  remote.sessions[win.wid] = st
  try {
    const conn = connect(
      {
        url: cfg.url,
        appName: cfg.appId,
        title: cfg.title,
        reconnect: cfg.reconnectBudgetMs ? { budgetMs: cfg.reconnectBudgetMs } : undefined,
      },
      {
        onWelcome(w) {
          st.status = 'online'
          if (w.rect.w > 0 && w.rect.h > 0) {
            st.frameW = Math.round(w.rect.w)
            st.frameH = Math.round(w.rect.h)
          }
        },
        onHits(_wid, hits) {
          st.hits = hits
        },
        onFrame(f) {
          st.frames += 1
          st.revision = Number(f.revision)
          frameCache.set(win.wid, f.payload)
          st.lastTexts = f.payload.ops
            .filter((op) => op.kind === 'text')
            .map((op) => (op.kind === 'text' ? op.text : ''))
        },
        onDead(reason) {
          st.status = 'dead'
          st.failReason = reason
        },
        // onLog 行文耦合（渲染器包零改动的代价）：disconnect 前缀 = 重连中。
        onLog(line) {
          if (line.startsWith('disconnected')) st.status = 'reconnecting'
          console.log(`[remote:${cfg.id}] ${line}`)
        },
      },
      WS,
    )
    conns.set(win.wid, conn)
  } catch (err) {
    st.status = 'dead'
    st.failReason = String(err)
  }
  return win
}

/**
 * boot 批量建连（G4）：逐条开窗；单条失败已被 openRemoteWindow 降级为
 * dead 状态面——桌面启动不被远程配置阻断。
 */
export function bootRemoteApps(configs: RemoteAppConfig[]): void {
  for (const cfg of configs) {
    try {
      openRemoteWindow(cfg)
    } catch (err) {
      console.warn(`[desktop] remote app boot failed: ${cfg.id}`, err)
    }
  }
}

// 会话回收注册：store.close 对 kind:"remote" 的钩子（断连 + 切片清理）。
setRemoteCleanup((wid) => {
  try {
    conns.get(wid)?.close()
  } catch {
    /* best-effort */
  }
  conns.delete(wid)
  frameCache.delete(wid)
  delete remote.sessions[wid]
})
