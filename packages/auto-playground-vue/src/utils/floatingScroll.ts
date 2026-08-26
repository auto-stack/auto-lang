/**
 * Floating overlay scrollbars for any scrollable element.
 *
 * Mounts absolutely-positioned thin rails over the HOST element (which
 * visually wraps the SCROLLER) and keeps them in sync with the scroller.
 * Used by ScrollArea.vue and directly by CodeEditor.vue (CodeMirror).
 *
 * Thumbs fade in on hover/scroll and out after an idle delay; they support
 * dragging, track jumps and wheel-over-rail forwarding.
 */

export interface FloatingScrollHandle {
  destroy(): void;
}

const MIN_THUMB = 24;
const HIDE_DELAY = 700;

const STYLE_ID = 'floating-scroll-style';

// ---------------------------------------------------------------------------
// Optional self-diagnostics (?fsbdebug=1): live HUD listing every mounted
// scrollbar instance and what each one saw, so environment-specific sync
// failures can be located precisely.
// ---------------------------------------------------------------------------

interface FsbDebugInfo {
  id: number;
  target: string;
  counters: {
    scrollEvents: number;
    polls: number;
    updates: number;
    lastScrollTop: number | null;
    lastThumbTop: number | null;
  };
}

function debugEnabled(): boolean {
  try {
    return new URLSearchParams(window.location.search).has('fsbdebug');
  } catch {
    return false;
  }
}

let dbgSeq = 0;

function reportInstance(info: FsbDebugInfo | null, id?: number) {
  const w = window as unknown as { __fsbInstances?: Map<number, FsbDebugInfo> };
  if (!w.__fsbInstances) w.__fsbInstances = new Map();
  if (info === null) w.__fsbInstances.delete(id!);
  else w.__fsbInstances.set(info.id, info);
  const hud = document.getElementById('fsb-hud');
  if (!hud) return;
  const rows = Array.from(w.__fsbInstances.values())
    .map((i) => `[${i.id}] ${i.target} · scroll:${i.counters.scrollEvents} poll:${i.counters.polls} upd:${i.counters.updates} top:${i.counters.lastScrollTop}→${i.counters.lastThumbTop}`)
    .join('<br>');
  hud.innerHTML = rows || '(no floating-scrollbar instances)';
}

function ensureHud() {
  if (!debugEnabled() || document.getElementById('fsb-hud')) return;
  const hud = document.createElement('div');
  hud.id = 'fsb-hud';
  hud.style.cssText =
    'position:fixed;left:8px;bottom:8px;z-index:99999;background:#111c;color:#7ee787;' +
    'font:11px/1.5 monospace;padding:8px 10px;border:1px solid #369;border-radius:4px;max-width:70vw;pointer-events:none;';
  document.body.appendChild(hud);
  const w = window as unknown as { __fsbInstances?: Map<number, FsbDebugInfo> };
  if (w.__fsbInstances) reportInstance(null);
}

function ensureStyle() {
  if (document.getElementById(STYLE_ID)) return;
  const style = document.createElement('style');
  style.id = STYLE_ID;
  style.textContent = `
.fsb-rail {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.2s ease;
  z-index: 8;
}
.fsb-rail.visible {
  opacity: 1;
  pointer-events: auto;
}
.fsb-y {
  top: 3px;
  right: 3px;
  width: 12px;
  height: calc(100% - 6px);
}
.fsb-x {
  bottom: 3px;
  left: 3px;
  height: 12px;
  width: calc(100% - 6px);
}
.fsb-thumb {
  /* absolute inside the positioned rail so style.top/left actually move it */
  position: absolute;
  background: rgba(212, 212, 212, 0.18);
  border-radius: 3px;
  transition: background 0.15s ease;
}
.fsb-y .fsb-thumb {
  left: 2.5px;
  width: 7px;
}
.fsb-x .fsb-thumb {
  top: 2.5px;
  height: 7px;
}
.fsb-rail:hover .fsb-thumb,
.fsb-thumb.dragging {
  background: rgba(212, 212, 212, 0.35);
}
`;
  document.head.appendChild(style);
}

export function attachFloatingScrollbar(scroller: HTMLElement, host?: HTMLElement): FloatingScrollHandle {
  ensureStyle();
  ensureHud();

  const dbgId = ++dbgSeq;
  const dbg: FsbDebugInfo | null = debugEnabled()
    ? {
        id: dbgId,
        target: `${scroller.tagName.toLowerCase()}.${(scroller.className || '').split(' ')[0]}`,
        counters: { scrollEvents: 0, polls: 0, updates: 0, lastScrollTop: null, lastThumbTop: null },
      }
    : null;
  if (dbg) reportInstance(dbg);

  const hostEl = host ?? (scroller.parentElement as HTMLElement);
  // Rails anchor to the host box; keep whatever positioning it already had
  const prevPosition = hostEl.style.position;
  if (getComputedStyle(hostEl).position === 'static') {
    hostEl.style.position = 'relative';
  }

  const railY = document.createElement('div');
  railY.className = 'fsb-rail fsb-y';
  const thumbY = document.createElement('div');
  thumbY.className = 'fsb-thumb';
  railY.appendChild(thumbY);

  const railX = document.createElement('div');
  railX.className = 'fsb-rail fsb-x';
  const thumbX = document.createElement('div');
  thumbX.className = 'fsb-thumb';
  railX.appendChild(thumbX);

  hostEl.appendChild(railY);
  hostEl.appendChild(railX);

  let visible = false;
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let rafId = 0;
  let destroyed = false;

  // Self-healing watchdog: regardless of event delivery quirks (passive
  // listeners, framework patching, zoom races), recompute geometry on a
  // light interval for as long as this scrollbar is alive.
  const pollId = window.setInterval(() => {
    if (!destroyed) {
      if (dbg) {
        dbg.counters.polls++;
        if (visible || draggingAxis) update();
        reportInstance(dbg);
      } else if (visible || draggingAxis) {
        update();
      }
    }
  }, 120);

  function show() {
    if (destroyed) return;
    update(); // never paint a stale position
    visible = true;
    syncVisibility();
    if (hideTimer) clearTimeout(hideTimer);
  }

  function scheduleHide() {
    if (hideTimer) clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (!draggingAxis) {
        visible = false;
        syncVisibility();
      }
    }, HIDE_DELAY);
  }

  function syncVisibility() {
    railY.classList.toggle('visible', visible && yOverflow);
    railX.classList.toggle('visible', visible && xOverflow);
  }

  function clamp(v: number, lo: number, hi: number) {
    return Math.min(hi, Math.max(lo, v));
  }

  let yOverflow = false;
  let xOverflow = false;

  function update() {
    if (destroyed) return;
    const maxY = scroller.scrollHeight - scroller.clientHeight;
    const maxX = scroller.scrollWidth - scroller.clientWidth;
    yOverflow = maxY > 1;
    xOverflow = maxX > 1;
    railY.style.display = yOverflow ? '' : 'none';
    railX.style.display = xOverflow ? '' : 'none';

    const trackY = scroller.clientHeight - 6;
    const trackX = scroller.clientWidth - 6;
    if (yOverflow) {
      const h = Math.max(MIN_THUMB, Math.round((scroller.clientHeight / scroller.scrollHeight) * trackY));
      const top = Math.round((scroller.scrollTop / maxY) * (trackY - h));
      thumbY.style.height = h + 'px';
      thumbY.style.top = top + 'px';
      if (dbg) {
        dbg.counters.updates++;
        dbg.counters.lastScrollTop = Math.round(scroller.scrollTop);
        dbg.counters.lastThumbTop = top;
      }
    }
    if (xOverflow) {
      const w = Math.max(MIN_THUMB, Math.round((scroller.clientWidth / scroller.scrollWidth) * trackX));
      const left = Math.round((scroller.scrollLeft / maxX) * (trackX - w));
      thumbX.style.width = w + 'px';
      thumbX.style.left = left + 'px';
    }
    syncVisibility();
  }

  function requestUpdate() {
    cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(update);
  }

  function onScroll() {
    if (dbg) dbg.counters.scrollEvents++;
    update();
    requestUpdate(); // defensive next-frame resync (zoom, layout races)
    show();
    scheduleHide();
  }

  function startDrag(e: PointerEvent, axis: 'y' | 'x') {
    draggingAxis = axis;
    const startY = e.clientY;
    const startX = e.clientX;
    const startTop = scroller.scrollTop;
    const startLeft = scroller.scrollLeft;
    const scaleY = scroller.scrollHeight / Math.max(1, scroller.clientHeight - 6);
    const scaleX = scroller.scrollWidth / Math.max(1, scroller.clientWidth - 6);

    const target = axis === 'y' ? railY : railX;
    try { target.setPointerCapture(e.pointerId); } catch { /* non-fatal */ }

    const onMove = (ev: PointerEvent) => {
      if (axis === 'y') scroller.scrollTop = startTop + (ev.clientY - startY) * scaleY;
      else scroller.scrollLeft = startLeft + (ev.clientX - startX) * scaleX;
      update();
    };
    const onUp = () => {
      draggingAxis = null;
      thumbY.classList.remove('dragging');
      thumbX.classList.remove('dragging');
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      window.removeEventListener('pointercancel', onUp);
      scheduleHide();
    };
    thumbY.classList.toggle('dragging', axis === 'y');
    thumbX.classList.toggle('dragging', axis === 'x');
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
    window.addEventListener('pointercancel', onUp);
  }

  let draggingAxis: 'y' | 'x' | null = null;

  function onRailPointerDown(e: PointerEvent, axis: 'y' | 'x') {
    e.preventDefault();
    e.stopPropagation();
    show();
    const rect = scroller.getBoundingClientRect();
    if (axis === 'y') {
      const trackH = scroller.clientHeight - 6;
      const y = e.clientY - rect.top - 3;
      const h = thumbYHeight();
      const pos = clamp(y - h / 2, 0, trackH - h);
      scroller.scrollTop = (pos / Math.max(1, trackH - h)) * (scroller.scrollHeight - scroller.clientHeight);
    } else {
      const trackW = scroller.clientWidth - 6;
      const x = e.clientX - rect.left - 3;
      const w = thumbXWidth();
      const pos = clamp(x - w / 2, 0, trackW - w);
      scroller.scrollLeft = (pos / Math.max(1, trackW - w)) * (scroller.scrollWidth - scroller.clientWidth);
    }
    update();
    startDrag(e, axis);
  }

  function thumbYHeight(): number {
    return parseFloat(thumbY.style.height || String(MIN_THUMB));
  }
  function thumbXWidth(): number {
    return parseFloat(thumbX.style.width || String(MIN_THUMB));
  }

  function onRailWheel(e: WheelEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (e.deltaY !== 0) scroller.scrollTop += e.deltaY;
    if (e.deltaX !== 0) scroller.scrollLeft += e.deltaX;
    show();
    scheduleHide();
  }

  function onHostEnter() {
    show();
    update();
  }
  function onHostLeave() {
    scheduleHide();
  }

  railY.addEventListener('pointerdown', (e) => onRailPointerDown(e, 'y'));
  railX.addEventListener('pointerdown', (e) => onRailPointerDown(e, 'x'));
  railY.addEventListener('wheel', onRailWheel, { passive: false });
  railX.addEventListener('wheel', onRailWheel, { passive: false });
  scroller.addEventListener('scroll', onScroll, { passive: true });
  hostEl.addEventListener('mouseenter', onHostEnter);
  hostEl.addEventListener('mouseleave', onHostLeave);

  const resizeObserver = new ResizeObserver(() => update());
  resizeObserver.observe(scroller);
  const mutationObserver = new MutationObserver(() => update());
  mutationObserver.observe(scroller, { childList: true, subtree: true, characterData: true });
  window.addEventListener('resize', update);

  update();

  return {
    destroy() {
      destroyed = true;
      window.clearInterval(pollId);
      if (dbg) reportInstance(null, dbgId);
      resizeObserver.disconnect();
      mutationObserver.disconnect();
      window.removeEventListener('resize', update);
      scroller.removeEventListener('scroll', onScroll);
      hostEl.removeEventListener('mouseenter', onHostEnter);
      hostEl.removeEventListener('mouseleave', onHostLeave);
      railY.remove();
      railX.remove();
      if (prevPosition !== undefined) hostEl.style.position = prevPosition;
      if (hideTimer) clearTimeout(hideTimer);
      cancelAnimationFrame(rafId);
    },
  };
}
