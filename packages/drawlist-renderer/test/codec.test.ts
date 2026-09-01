import { describe, expect, it } from 'vitest';

import { decodeEnvelope, encodeEnvelope } from '../src/codec.ts';
import {
  decodeServerMsg,
  encodeHello,
  encodePointerPressed,
} from '../src/messages.ts';
import {
  FRAME_HEX,
  HELLO_HEX,
  PRESS_HEX,
  SCISSOR_FRAME_HEX,
  STYLED_FRAME_HEX,
} from './fixtures.golden.ts';

const hex = (bytes: Uint8Array): number[] => Array.from(bytes);

describe('对拍锚点（Rust codec 同源字节）', () => {
  it('Hello 编码与 Rust golden 恒等', () => {
    const got = encodeHello({ appName: '002-counter', width: 480, height: 320 });
    expect(hex(got)).toEqual(HELLO_HEX);
  });

  it('PointerPressed 编码与 Rust golden 恒等', () => {
    const got = encodePointerPressed(1n, 100.5, 50.25);
    expect(hex(got)).toEqual(PRESS_HEX);
  });

  it('FrameReady 解码与 Rust golden 恒等（DrawList 全字段）', () => {
    const msg = decodeServerMsg(new Uint8Array(FRAME_HEX));
    expect(msg.kind).toBe('frame');
    if (msg.kind !== 'frame') return;
    expect(msg.frame.wid).toBe(1n);
    expect(msg.frame.frameId).toBe(2n);
    expect(msg.frame.revision).toBe(2n);
    expect(msg.frame.payload.clear).toEqual({ r: 9, g: 14, b: 26, a: 255 });
    expect(msg.frame.payload.ops).toEqual([
      {
        kind: 'quad',
        rect: { x: 8, y: 8, w: 100, h: 40 },
        color: { r: 59, g: 130, b: 246, a: 255 },
      },
      {
        kind: 'text',
        x: 12,
        y: 16,
        size: 14,
        lineHeight: 20,
        color: { r: 255, g: 255, b: 255, a: 255 },
        text: 'Counter: 0',
      },
    ]);
  });

  it('scissor 栈帧解码与 Rust golden 恒等（Plan 515 G1）', () => {
    const msg = decodeServerMsg(new Uint8Array(SCISSOR_FRAME_HEX));
    expect(msg.kind).toBe('frame');
    if (msg.kind !== 'frame') return;
    expect(msg.frame.payload.ops).toEqual([
      { kind: 'scissor', rect: { x: 8, y: 8, w: 120, h: 60 } },
      {
        kind: 'quad',
        rect: { x: 0, y: 0, w: 400, h: 400 },
        color: { r: 59, g: 130, b: 246, a: 255 },
      },
      { kind: 'scissorPop' },
    ]);
  });

  it('TextStyled 差分帧解码与 Rust golden 恒等（Plan 515 G2）', () => {
    const msg = decodeServerMsg(new Uint8Array(STYLED_FRAME_HEX));
    expect(msg.kind).toBe('frame');
    if (msg.kind !== 'frame') return;
    expect(msg.frame.payload.ops).toEqual([
      {
        kind: 'textStyled',
        x: 10,
        y: 20,
        size: 16,
        // f32(21.6) 的双精度展开（线格式 4 字节浮点精度）。
        lineHeight: 21.600000381469727,
        color: { r: 220, g: 220, b: 220, a: 255 },
        weight: 700,
        italic: false,
        text: 'Bold',
      },
    ]);
  });
});

describe('信封', () => {
  it('编码-解码 round-trip', () => {
    const env = encodeEnvelope(1, 3, new Uint8Array([1, 2, 3]));
    const back = decodeEnvelope(env);
    expect(back.channel).toBe(3);
    expect(back.version).toBe(1);
    expect(Array.from(back.payload)).toEqual([1, 2, 3]);
  });

  it('坏魔数/截断拒收', () => {
    const env = encodeEnvelope(1, 2, new Uint8Array([9]));
    env[0] = 0x58;
    expect(() => decodeEnvelope(env)).toThrow(/magic/);
    expect(() => decodeEnvelope(env.slice(0, 8))).toThrow(/short/);
  });
});
