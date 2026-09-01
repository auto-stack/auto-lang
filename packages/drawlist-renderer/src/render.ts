// Plan 508 G5 —— DrawList → Canvas2D（D1-A 定案的 web 同义实现：
// 字符串+样式、宿主侧 shaping → `fillText`；Quad → `fillRect`）。
// Tier1+2 覆盖集的帧即渲染集（Plan 507 覆盖表）。

import type { DrawList, HitRegion, Rgba8, WRect } from './messages.ts';

export function rgbaToCss(c: Rgba8): string {
  return `rgba(${c.r}, ${c.g}, ${c.b}, ${(c.a / 255).toFixed(3)})`;
}

/**
 * 渲染一帧。清屏色存在时先铺满（无尺寸信息——用 ctx.canvas 位图尺寸）；
 * ops 先到先画（后画盖前，与宿主合成序一致）。Text y = 顶部定位
 * （Rust 语义"左上角"），故 `textBaseline='top'`。
 */
export function renderFrame(ctx: CanvasRenderingContext2D, frame: DrawList): void {
  if (frame.clear) {
    ctx.fillStyle = rgbaToCss(frame.clear);
    ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  }
  for (const op of frame.ops) {
    if (op.kind === 'quad') {
      ctx.fillStyle = rgbaToCss(op.color);
      ctx.fillRect(op.rect.x, op.rect.y, op.rect.w, op.rect.h);
    } else {
      ctx.fillStyle = rgbaToCss(op.color);
      ctx.font = `${op.size}px system-ui, sans-serif`;
      ctx.textBaseline = 'top';
      ctx.fillText(op.text, op.x, op.y);
    }
  }
}

/**
 * 交互区命中判定（表驱动）：首个包含点的区域（Rust `hits.position`
 * 同序语义）。返回 null = 空白区。**仅作本地寻址/光标 UX**——权威
 * 命中在 app 侧（坐标直传）。
 */
export function hitTest(hits: HitRegion[], x: number, y: number): HitRegion | null {
  for (const h of hits) {
    const r: WRect = h.rect;
    if (x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h) {
      return h;
    }
  }
  return null;
}
