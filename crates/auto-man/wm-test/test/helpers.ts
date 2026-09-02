// Plan 516 T1/T2/T3 公共件：FakeWebSocket（时序可控的 WS 桩，508
// connect.test 同型）+ Welcome/Frame/HitTable 服务器消息字节构造（codec
// Writer 直构——codec 正确性由渲染器包 golden 对拍覆盖）。
import { Channel, Writer, encodeEnvelope } from '../../../../packages/drawlist-renderer/src/codec.ts'

/** 假 WebSocket：手动驱动 open/message/close（时序可控）。 */
export class FakeWebSocket {
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3
  static instances: FakeWebSocket[] = []
  readyState = FakeWebSocket.CLOSED
  binaryType = 'blob'
  sent: Uint8Array[] = []
  onopen: (() => void) | null = null
  onmessage: ((ev: { data: unknown }) => void) | null = null
  onclose: (() => void) | null = null
  onerror: (() => void) | null = null
  constructor(public url: string) {
    FakeWebSocket.instances.push(this)
  }
  send(bytes: Uint8Array | ArrayBuffer): void {
    // connect.sendRaw 传 ArrayBuffer——统一为 Uint8Array 便于字节断言。
    this.sent.push(bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes)
  }
  close(): void {
    this.readyState = FakeWebSocket.CLOSED
  }
  // 测试驱动口。
  fireOpen(): void {
    this.readyState = FakeWebSocket.OPEN
    this.onopen?.()
  }
  fireMessage(bytes: Uint8Array): void {
    this.onmessage?.({ data: bytes.slice().buffer })
  }
  fireClose(): void {
    this.readyState = FakeWebSocket.CLOSED
    this.onclose?.()
  }
}

/** Welcome（Handshake tag2；rect = 远端逻辑尺寸——canvas 位图源空间）。 */
export function welcomeBytes(wid: number, w = 480, h = 320): Uint8Array {
  const w8 = new Writer()
  w8.u8(2) // WELCOME
  w8.u64(1n) // app_id
  w8.u64(BigInt(wid))
  w8.u64(3n) // surface
  w8.f32(0).f32(0).f32(w).f32(h) // rect
  w8.u8(1) // frameMode = Commands
  return encodeEnvelope(1, Channel.Handshake, w8.build())
}

/** 一条文本 op 的帧（Frame tag4；revision 递增供闭环断言）。 */
export function frameBytes(wid: number, frameId: number, revision: number, text: string): Uint8Array {
  const w = new Writer()
  w.u8(4) // FRAME_READY
  w.u64(BigInt(wid))
  w.u64(BigInt(frameId))
  w.u8(0) // slot
  w.bool(false) // damage 无
  w.u64(BigInt(revision))
  // DrawList：kind 1 + 清屏白 + 单文本 op。
  w.u8(1)
  w.bool(true)
  w.u8(255).u8(255).u8(255).u8(255)
  w.u32(1)
  w.u8(2) // text op
  w.f32(12).f32(8).f32(14).f32(20)
  w.u8(0).u8(0).u8(0).u8(255)
  w.string(text)
  return encodeEnvelope(1, Channel.Frame, w.build())
}

/** HitTable（Frame tag9）：按钮/输入区表。 */
export function hitTableBytes(
  wid: number,
  hits: Array<{ x: number; y: number; w: number; h: number; kind: number; action: string }>,
): Uint8Array {
  const w = new Writer()
  w.u8(9) // HIT_TABLE
  w.u64(BigInt(wid))
  w.u32(hits.length)
  for (const h of hits) {
    w.f32(h.x).f32(h.y).f32(h.w).f32(h.h)
    w.u8(h.kind)
    w.string(h.action)
  }
  return encodeEnvelope(1, Channel.Frame, w.build())
}

/** openRemoteWindow 后最近一条会话的 WS 桩（fireOpen/Message 驱动用）。 */
export function lastSocket(): FakeWebSocket {
  const ws = FakeWebSocket.instances.at(-1)
  if (!ws) throw new Error('no FakeWebSocket dialled yet')
  return ws
}

/** 等 rAF 排空（帧渲染回调在下一帧——jsdom 真实 rAF）。 */
export function nextFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()))
}
