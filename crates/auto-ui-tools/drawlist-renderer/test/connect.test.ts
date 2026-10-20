import { describe, expect, it, vi } from 'vitest';

import { Channel, Writer, encodeEnvelope } from '../src/codec.ts';
import { DEFAULT_RECONNECT, connect } from '../src/connect.ts';
import { FRAME_HEX } from './fixtures.golden.ts';

/** Welcome 测试帧（codec 正确性由 Rust golden 对拍覆盖，此处自构即可）。 */
function welcomeBytes(wid: bigint): Uint8Array {
  const w = new Writer();
  w.u8(2); // WELCOME
  w.u64(1n); // app_id
  w.u64(wid);
  w.u64(3n); // surface
  w.f32(0).f32(0).f32(480).f32(320); // rect
  w.u8(1); // frameMode = Commands
  return encodeEnvelope(1, Channel.Handshake, w.build());
}

/** 假 WebSocket：手动驱动 open/message/close（时序可控）。 */
class FakeWebSocket {
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;
  static instances: FakeWebSocket[] = [];
  readyState = FakeWebSocket.CLOSED;
  binaryType = 'blob';
  sent: Uint8Array[] = [];
  onopen: (() => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  constructor(public url: string) {
    FakeWebSocket.instances.push(this);
  }
  send(bytes: Uint8Array): void {
    this.sent.push(bytes);
  }
  close(): void {
    this.readyState = FakeWebSocket.CLOSED;
  }
  // 测试驱动口。
  fireOpen(): void {
    this.readyState = FakeWebSocket.OPEN;
    this.onopen?.();
  }
  fireMessage(bytes: Uint8Array): void {
    this.onmessage?.({ data: bytes.slice().buffer });
  }
  fireClose(): void {
    this.readyState = FakeWebSocket.CLOSED;
    this.onclose?.();
  }
}

describe('connect（重连语义对齐 ReconnectPolicy）', () => {
  it('open 即发 Hello；Welcome/Frame 分发；输入回发携带 wid', () => {
    FakeWebSocket.instances = [];
    const onWelcome = vi.fn();
    const onFrame = vi.fn();
    const conn = connect(
      { url: 'ws://x/?token=t', appName: '002-counter' },
      { onWelcome, onHits: () => {}, onFrame },
      FakeWebSocket as unknown as typeof WebSocket,
    );
    const ws = FakeWebSocket.instances.at(-1)!;
    ws.fireOpen();
    expect(ws.sent.length).toBe(1); // Hello
    ws.fireMessage(new Uint8Array(FRAME_HEX));
    expect(onFrame).toHaveBeenCalledTimes(1);
    expect(onFrame.mock.calls[0]![0].payload.ops[1].text).toBe('Counter: 0');
    // Welcome 前不回发（wid 未知）。
    conn.sendPointerDown(1, 2);
    expect(ws.sent.length).toBe(1);
    ws.fireMessage(welcomeBytes(2n));
    expect(onWelcome).toHaveBeenCalledTimes(1);
    conn.sendPointerDown(10.5, 20.25);
    expect(ws.sent.length).toBe(2); // Hello + PointerPressed
    conn.close();
  });

  it('断线预算内重连（固定间隔）；预算耗尽 onDead', () => {
    vi.useFakeTimers();
    FakeWebSocket.instances = [];
    const onDead = vi.fn();
    connect(
      { url: 'ws://x/?token=t', appName: 'a', reconnect: { budgetMs: 1000, intervalMs: 50 } },
      { onWelcome: () => {}, onHits: () => {}, onFrame: () => {}, onDead },
      FakeWebSocket as unknown as typeof WebSocket,
    );
    // 第一次断线 → 50ms 后重拨。
    FakeWebSocket.instances[0]!.fireClose();
    expect(FakeWebSocket.instances.length).toBe(1);
    vi.advanceTimersByTime(50);
    expect(FakeWebSocket.instances.length).toBe(2);
    // 推进超预算后再断 → onDead（不再重拨）。
    vi.advanceTimersByTime(1000);
    FakeWebSocket.instances[1]!.fireClose();
    expect(onDead).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(200);
    expect(FakeWebSocket.instances.length).toBe(2);
    vi.useRealTimers();
  });

  it('缺省重连策略 = Rust ReconnectPolicy 对齐值', () => {
    expect(DEFAULT_RECONNECT).toEqual({ budgetMs: 30_000, intervalMs: 50 });
  });
});
