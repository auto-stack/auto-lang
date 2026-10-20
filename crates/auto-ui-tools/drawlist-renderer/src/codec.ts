// Plan 508 G5 —— 桌面协议 wire format 的 TS 镜像（Rust `codec.rs` 同款，
// 手工镜像 + 对拍测试兜底——计划②裁定：Rust 侧 `p508_ts_crosscheck_
// golden_bytes` 与本包 `test/fixtures.golden.ts` 钉同一批字节，任一侧
// codec 漂移即红）。
//
// 信封布局（小端）：
//   [0..4) magic "APDL" / [4..6) u16 version / [6] u8 channel / [7] 保留 0
//   [8..12) u32 payload 长度 / [12..) payload
// 传输映射（WsTransport）：一条 WS Binary 消息 = 一个信封（无二次分帧）。

export const PROTOCOL_VERSION = 1;

export const MAGIC = [0x41, 0x50, 0x44, 0x4c] as const; // "APDL"

/** 五通道（线序编号）。 */
export const Channel = {
  Handshake: 1,
  Frame: 2,
  Input: 3,
  Control: 4,
  Observe: 5,
} as const;

export type ChannelId = (typeof Channel)[keyof typeof Channel];

export class CodecError extends Error {
  constructor(reason: string) {
    super(`codec: ${reason}`);
    this.name = 'CodecError';
  }
}

/** 游标式 LE 读取器（越界统一 TooShort 语义）。 */
export class Reader {
  private pos = 0;
  constructor(private readonly data: Uint8Array) {}

  private take(n: number): Uint8Array {
    if (this.pos + n > this.data.length) {
      throw new CodecError('too short');
    }
    const s = this.data.subarray(this.pos, this.pos + n);
    this.pos += n;
    return s;
  }

  u8(): number {
    return this.take(1)[0]!;
  }

  u16(): number {
    const b = this.take(2);
    return b[0]! | (b[1]! << 8);
  }

  u32(): number {
    const b = this.take(4);
    return (b[0]! | (b[1]! << 8) | (b[2]! << 16) | (b[3]! << 24)) >>> 0;
  }

  u64(): bigint {
    const lo = BigInt(this.u32());
    const hi = BigInt(this.u32());
    return lo | (hi << 32n);
  }

  f32(): number {
    return new DataView(this.take(4).slice().buffer).getFloat32(0, true);
  }

  bool(): boolean {
    const v = this.u8();
    if (v === 0) return false;
    if (v === 1) return true;
    throw new CodecError(`bad bool ${v}`);
  }

  string(): string {
    const len = this.u32();
    return new TextDecoder().decode(this.take(len));
  }

  finish(): void {
    if (this.pos !== this.data.length) {
      throw new CodecError(`trailing bytes: ${this.data.length - this.pos}`);
    }
  }
}

/** LE 写出器。 */
export class Writer {
  readonly buf: number[] = [];

  u8(v: number): this {
    this.buf.push(v & 0xff);
    return this;
  }

  u16(v: number): this {
    this.buf.push(v & 0xff, (v >> 8) & 0xff);
    return this;
  }

  u32(v: number): this {
    this.buf.push(v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff, (v >>> 24) & 0xff);
    return this;
  }

  u64(v: bigint): this {
    this.u32(Number(v & 0xffffffffn));
    this.u32(Number((v >> 32n) & 0xffffffffn));
    return this;
  }

  f32(v: number): this {
    const b = new Uint8Array(4);
    new DataView(b.buffer).setFloat32(0, v, true);
    this.buf.push(...b);
    return this;
  }

  bool(v: boolean): this {
    this.buf.push(v ? 1 : 0);
    return this;
  }

  string(v: string): this {
    const bytes = new TextEncoder().encode(v);
    this.u32(bytes.length);
    this.buf.push(...bytes);
    return this;
  }

  build(): Uint8Array {
    return new Uint8Array(this.buf);
  }
}

/** 信封编码。 */
export function encodeEnvelope(
  version: number,
  channel: ChannelId,
  payload: Uint8Array,
): Uint8Array {
  const out = new Uint8Array(12 + payload.length);
  out.set(MAGIC, 0);
  new DataView(out.buffer).setUint16(4, version, true);
  out[6] = channel;
  out[7] = 0;
  new DataView(out.buffer).setUint32(8, payload.length, true);
  out.set(payload, 12);
  return out;
}

/** 信封解码：校验魔数/通道/长度自洽。 */
export function decodeEnvelope(bytes: Uint8Array): { channel: number; version: number; payload: Uint8Array } {
  if (bytes.length < 12) throw new CodecError('too short');
  for (let i = 0; i < 4; i++) {
    if (bytes[i] !== MAGIC[i]) throw new CodecError('bad magic');
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const version = view.getUint16(4, true);
  const channel = bytes[6]!;
  if (channel < 1 || channel > 5) throw new CodecError(`unknown channel ${channel}`);
  const len = view.getUint32(8, true);
  if (bytes.length < 12 + len) throw new CodecError('too short');
  return { channel, version, payload: bytes.subarray(12, 12 + len) };
}
