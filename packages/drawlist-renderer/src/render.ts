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
 *
 * Plan 515 G1 —— scissor 栈：push = `save`+`clip`（嵌套自然取交——
 * Canvas2D 裁剪区与已存裁剪区求交），pop = `restore`。编码端保证配对；
 * 未闭合 push 帧尾补 restore（与宿主"裁到序列尾"同语义），空栈 pop =
 * Canvas2D 原生 no-op。
 */
export function renderFrame(ctx: CanvasRenderingContext2D, frame: DrawList): void {
  if (frame.clear) {
    ctx.fillStyle = rgbaToCss(frame.clear);
    ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  }
  let clipDepth = 0;
  for (const op of frame.ops) {
    if (op.kind === 'quad') {
      ctx.fillStyle = rgbaToCss(op.color);
      ctx.fillRect(op.rect.x, op.rect.y, op.rect.w, op.rect.h);
    } else if (op.kind === 'scissor') {
      ctx.save();
      ctx.beginPath();
      ctx.rect(op.rect.x, op.rect.y, op.rect.w, op.rect.h);
      ctx.clip();
      clipDepth++;
    } else if (op.kind === 'scissorPop') {
      if (clipDepth > 0) {
        ctx.restore();
        clipDepth--;
      }
    } else if (op.kind === 'textStyled') {
      // Plan 515 G2 —— 字重/斜体差分（Canvas2D font 简写前缀）。
      ctx.fillStyle = rgbaToCss(op.color);
      const style = op.italic ? 'italic ' : '';
      const weight = op.weight !== 400 ? `${op.weight} ` : '';
      ctx.font = `${style}${weight}${op.size}px system-ui, sans-serif`;
      ctx.textBaseline = 'top';
      ctx.fillText(op.text, op.x, op.y);
    } else {
      ctx.fillStyle = rgbaToCss(op.color);
      ctx.font = `${op.size}px system-ui, sans-serif`;
      ctx.textBaseline = 'top';
      ctx.fillText(op.text, op.x, op.y);
    }
  }
  // 未闭合 push：裁到序列尾后复位（状态不泄漏到下一帧）。
  for (; clipDepth > 0; clipDepth--) {
    ctx.restore();
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
