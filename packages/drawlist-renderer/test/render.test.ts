import { describe, expect, it } from 'vitest';

import { hitTest, renderFrame, rgbaToCss } from '../src/render.ts';
import type { DrawList, HitRegion } from '../src/messages.ts';

/** Canvas2D mock：记录调用序（渲染快照 = 调用序列断言）。 */
function mockCtx(width = 480, height = 320) {
  const calls: Array<Record<string, unknown>> = [];
  const ctx = {
    canvas: { width, height },
    set fillStyle(v: string) {
      calls.push({ op: 'fillStyle', value: v });
    },
    get fillStyle() {
      return '';
    },
    set font(v: string) {
      calls.push({ op: 'font', value: v });
    },
    get font() {
      return '';
    },
    set textBaseline(v: string) {
      calls.push({ op: 'textBaseline', value: v });
    },
    get textBaseline() {
      return '';
    },
    fillRect(x: number, y: number, w: number, h: number) {
      calls.push({ op: 'fillRect', x, y, w, h });
    },
    fillText(text: string, x: number, y: number) {
      calls.push({ op: 'fillText', text, x, y });
    },
  } as unknown as CanvasRenderingContext2D;
  return { ctx, calls };
}

describe('renderFrame（Canvas2D）', () => {
  const frame: DrawList = {
    clear: { r: 9, g: 14, b: 26, a: 255 },
    ops: [
      { kind: 'quad', rect: { x: 8, y: 8, w: 100, h: 40 }, color: { r: 59, g: 130, b: 246, a: 255 } },
      {
        kind: 'text',
        x: 12,
        y: 16,
        size: 14,
        lineHeight: 20,
        color: { r: 255, g: 255, b: 255, a: 255 },
        text: 'Counter: 0',
      },
    ],
  };

  it('清屏 → Quad → Text 调用序与样式', () => {
    const { ctx, calls } = mockCtx();
    renderFrame(ctx, frame);
    expect(calls).toEqual([
      { op: 'fillStyle', value: 'rgba(9, 14, 26, 1.000)' },
      { op: 'fillRect', x: 0, y: 0, w: 480, h: 320 },
      { op: 'fillStyle', value: 'rgba(59, 130, 246, 1.000)' },
      { op: 'fillRect', x: 8, y: 8, w: 100, h: 40 },
      { op: 'fillStyle', value: 'rgba(255, 255, 255, 1.000)' },
      { op: 'font', value: '14px system-ui, sans-serif' },
      { op: 'textBaseline', value: 'top' },
      { op: 'fillText', text: 'Counter: 0', x: 12, y: 16 },
    ]);
  });

  it('无清屏色 = 不铺底', () => {
    const { ctx, calls } = mockCtx();
    renderFrame(ctx, { clear: null, ops: [] });
    expect(calls).toEqual([]);
  });

  it('rgbaToCss 透明度换算', () => {
    expect(rgbaToCss({ r: 0, g: 0, b: 0, a: 128 })).toBe('rgba(0, 0, 0, 0.502)');
  });
});

describe('hitTest（表驱动）', () => {
  const hits: HitRegion[] = [
    { rect: { x: 0, y: 0, w: 50, h: 30 }, kind: 1, action: 'dec' },
    { rect: { x: 60, y: 0, w: 50, h: 30 }, kind: 1, action: 'reset' },
    { rect: { x: 120, y: 0, w: 50, h: 30 }, kind: 1, action: 'inc' },
    { rect: { x: 0, y: 100, w: 200, h: 24 }, kind: 2, action: 'celsius' },
  ];

  it.each([
    [25, 15, 'dec'],
    [70, 5, 'reset'],
    [145, 29, 'inc'],
    [10, 110, 'celsius'],
  ] as const)('(%i, %i) → %s', (x, y, action) => {
    expect(hitTest(hits, x, y)?.action).toBe(action);
  });

  it.each([
    [55, 15],
    [-1, 0],
    [50, 30], // 右/下边界为开区间
    [0, 200],
  ] as const)('(%i, %i) → null', (x, y) => {
    expect(hitTest(hits, x, y)).toBeNull();
  });

  it('重叠区首个命中（先到先得）', () => {
    const overlapped: HitRegion[] = [
      { rect: { x: 0, y: 0, w: 100, h: 100 }, kind: 1, action: 'first' },
      { rect: { x: 0, y: 0, w: 100, h: 100 }, kind: 1, action: 'second' },
    ];
    expect(hitTest(overlapped, 50, 50)?.action).toBe('first');
  });
});
