// Plan 508 G5 —— 协议消息的 TS 镜像（Rust `message.rs` tag 表同源；
// 对拍锚点见 test/fixtures.golden.ts）。仅镜像远程渲染器所需子集：
// Hello（编码）/ Welcome·FrameReady·HitTable（解码）/ InputMsg（编码）。

import { Channel, CodecError, Reader, Writer, decodeEnvelope, encodeEnvelope } from './codec.ts';

export interface WRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Rgba8 {
  r: number;
  g: number;
  b: number;
  a: number;
}

export interface DrawList {
  clear: Rgba8 | null;
  ops: DrawOp[];
}

export type DrawOp =
  | { kind: 'quad'; rect: WRect; color: Rgba8 }
  | { kind: 'text'; x: number; y: number; size: number; lineHeight: number; color: Rgba8; text: string }
  // Plan 515 G1 —— scissor 裁剪栈（Rust tag 3/4 镜像）：push 与当前有效
  // 裁剪取交，作用于后续 op 直至配对 pop。
  | { kind: 'scissor'; rect: WRect }
  | { kind: 'scissorPop' };

export interface HitRegion {
  rect: WRect;
  kind: number; // 1 = button（action=handler）/ 2 = input（action=field）
  action: string;
}

export const HIT_KIND_BUTTON = 1;
export const HIT_KIND_INPUT = 2;

export interface Welcome {
  appId: bigint;
  wid: bigint;
  surface: bigint;
  rect: WRect;
  frameMode: number; // 1 = Commands
}

export interface FrameReady {
  wid: bigint;
  frameId: bigint;
  revision: bigint;
  payload: DrawList;
}

export type ServerMsg =
  | { kind: 'welcome'; welcome: Welcome }
  | { kind: 'frame'; frame: FrameReady }
  | { kind: 'hitTable'; wid: bigint; hits: HitRegion[] };

// ---------------------------------------------------------------------------
// 编码（远程端 → 宿主）
// ---------------------------------------------------------------------------

/** Handshake::Hello（订阅 = app 名 + 初始尺寸）。 */
export function encodeHello(opts: {
  appName: string;
  title?: string;
  width?: number;
  height?: number;
  version?: number;
}): Uint8Array {
  const w = new Writer();
  w.u8(1); // HELLO
  w.u16(opts.version ?? 1);
  w.string(opts.appName);
  w.string(opts.title ?? opts.appName);
  w.bool(false); // icon: None
  w.f32(opts.width ?? 480);
  w.f32(opts.height ?? 320);
  w.u32(0); // fonts: 空
  return encodeEnvelope(opts.version ?? 1, Channel.Handshake, w.build());
}

/** InputMsg::PointerPressed（左键；坐标 = 画布/窗本地空间，app 侧权威命中）。 */
export function encodePointerPressed(
  wid: bigint,
  x: number,
  y: number,
  modifiers = 0,
  button = 1,
): Uint8Array {
  const w = new Writer();
  w.u8(2); // POINTER_PRESSED
  w.u64(wid);
  w.u8(button);
  w.f32(x);
  w.f32(y);
  w.u8(modifiers);
  return encodeEnvelope(1, Channel.Input, w.build());
}

/** InputMsg::CharTyped（聚焦输入的字符回发）。 */
export function encodeCharTyped(wid: bigint, ch: string): Uint8Array {
  const w = new Writer();
  w.u8(6); // CHAR_TYPED
  w.u64(wid);
  w.string(ch);
  return encodeEnvelope(1, Channel.Input, w.build());
}

// ---------------------------------------------------------------------------
// 解码（宿主 → 远程端）
// ---------------------------------------------------------------------------

function readRect(r: Reader): WRect {
  return { x: r.f32(), y: r.f32(), w: r.f32(), h: r.f32() };
}

function readColor(r: Reader): Rgba8 {
  return { r: r.u8(), g: r.u8(), b: r.u8(), a: r.u8() };
}

function readDrawList(r: Reader): DrawList {
  const kind = r.u8();
  if (kind !== 1) throw new CodecError(`unknown drawlist kind ${kind}`);
  const clear = r.bool() ? readColor(r) : null;
  const n = r.u32();
  const ops: DrawOp[] = [];
  for (let i = 0; i < n; i++) {
    const tag = r.u8();
    if (tag === 1) {
      ops.push({ kind: 'quad', rect: readRect(r), color: readColor(r) });
    } else if (tag === 2) {
      const x = r.f32();
      const y = r.f32();
      const size = r.f32();
      const lineHeight = r.f32();
      const color = readColor(r);
      const text = r.string();
      ops.push({ kind: 'text', x, y, size, lineHeight, color, text });
    } else if (tag === 3) {
      ops.push({ kind: 'scissor', rect: readRect(r) });
    } else if (tag === 4) {
      ops.push({ kind: 'scissorPop' });
    } else {
      throw new CodecError(`unknown drawop tag ${tag}`);
    }
  }
  return { clear, ops };
}

/** 服务器消息总入口（信封解码 + 通道分发）。 */
export function decodeServerMsg(bytes: Uint8Array): ServerMsg {
  const env = decodeEnvelope(bytes);
  if (env.version !== 1) throw new CodecError(`unsupported version ${env.version}`);
  const { channel, payload } = env;
  const r = new Reader(payload);
  const msg = (() => {
    if (channel === Channel.Handshake) {
      const tag = r.u8();
      if (tag === 2) {
        // WELCOME：tag+appId+wid+surface+rect = 41B；v1.3 尾部追加
        // frameMode（旧端载荷无此字节 → 缺省 Commands）。
        const appId = r.u64();
        const wid = r.u64();
        const surface = r.u64();
        const rect = readRect(r);
        const frameMode = payload.length > 41 ? r.u8() : 1;
        return { kind: 'welcome' as const, welcome: { appId, wid, surface, rect, frameMode } };
      }
      throw new CodecError(`unknown handshake tag ${tag}`);
    }
    if (channel === Channel.Frame) {
      const tag = r.u8();
      if (tag === 4) {
        // FRAME_READY
        const wid = r.u64();
        const frameId = r.u64();
        r.u8(); // slot
        if (r.bool()) readRect(r); // damage（丢弃——全帧重绘）
        const revision = r.u64();
        const payloadDraw = readDrawList(r);
        return { kind: 'frame' as const, frame: { wid, frameId, revision, payload: payloadDraw } };
      }
      if (tag === 9) {
        // HIT_TABLE
        const wid = r.u64();
        const n = r.u32();
        const hits: HitRegion[] = [];
        for (let i = 0; i < n; i++) {
          const rect = readRect(r);
          const kind = r.u8();
          const action = r.string();
          hits.push({ rect, kind, action });
        }
        return { kind: 'hitTable' as const, wid, hits };
      }
      throw new CodecError(`unknown frame tag ${tag}`);
    }
    throw new CodecError(`unexpected channel ${channel}`);
  })();
  r.finish();
  return msg;
}
