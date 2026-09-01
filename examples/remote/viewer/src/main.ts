// Plan 508 G6 demo 页：连宿主（WS + token）→ DrawList 渲染到 Canvas2D
// → 点击画布命中区回发 InputMsg。URL 参数：token/app/port/ws（缺省
// demo-token/002-counter/17800）。

import {
  type HitRegion,
  connect,
  hitTest,
  renderFrame,
} from '@auto/drawlist-renderer';

const params = new URLSearchParams(location.search);
const token = params.get('token') ?? 'demo-token';
const appName = params.get('app') ?? '002-counter';
const port = params.get('port') ?? '17800';
const url =
  params.get('ws') ?? `ws://127.0.0.1:${port}/?token=${encodeURIComponent(token)}`;

const statusEl = document.getElementById('status')!;
const canvas = document.getElementById('view') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;

const setStatus = (text: string, err = false): void => {
  statusEl.textContent = text;
  statusEl.className = err ? 'err' : '';
};

// e2e 断言探针（Playwright 读取；生产无副作用）。
const probe = {
  frames: 0,
  clicks: 0,
  lastTexts: [] as string[],
  welcome: false,
  hits: 0,
  /** 按钮命中区中心（CSS 像素；e2e 点击寻址用）。 */
  buttonCenters(): Array<{ x: number; y: number }> {
    const r = canvas.getBoundingClientRect();
    const sx = r.width / canvas.width;
    const sy = r.height / canvas.height;
    return hits
      .filter((h) => h.kind === 1)
      .map((h) => ({
        x: r.left + (h.rect.x + h.rect.w / 2) * sx,
        y: r.top + (h.rect.y + h.rect.h / 2) * sy,
      }));
  },
};
(window as unknown as Record<string, unknown>).__remote = probe;

let hits: HitRegion[] = [];
const conn = connect(
  { url, appName, width: canvas.width, height: canvas.height },
  {
    onWelcome() {
      probe.welcome = true;
      setStatus(`connected · ${appName} @ ${url.replace(/token=[^&]+/, 'token=…')}`);
    },
    onHits(_wid, table) {
      hits = table;
      probe.hits = table.length;
    },
    onFrame(frame) {
      probe.frames += 1;
      renderFrame(ctx, frame.payload);
      probe.lastTexts = frame.payload.ops
        .filter((op) => op.kind === 'text')
        .map((op) => (op.kind === 'text' ? op.text : ''));
    },
    onDead(reason) {
      setStatus(`dead: ${reason}`, true);
    },
    onLog(line) {
      console.log(`[remote] ${line}`);
    },
  },
);

canvas.addEventListener('click', (ev) => {
  const r = canvas.getBoundingClientRect();
  const x = ((ev.clientX - r.left) * canvas.width) / r.width;
  const y = ((ev.clientY - r.top) * canvas.height) / r.height;
  const hit = hitTest(hits, x, y);
  if (hit) {
    probe.clicks += 1;
    conn.sendPointerDown(x, y);
  }
});

// 键盘字符回发（003/005 输入闭环口径；聚焦策略 v1 = 画布即焦点面）。
canvas.tabIndex = 0;
canvas.addEventListener('keydown', (ev) => {
  if (ev.key.length === 1) {
    conn.sendChar(ev.key);
  }
});
