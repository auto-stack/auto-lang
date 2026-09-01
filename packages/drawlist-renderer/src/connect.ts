// Plan 508 G5 —— 远程连接管理：WS 连入 + Hello 订阅 + 消息分发 +
// 输入回发 + 断线重连（Rust `ReconnectPolicy` 语义对齐：固定间隔重试、
// 总预算 30s，预算耗尽判 dead）。

import {
  type DrawList,
  type HitRegion,
  type Welcome,
  decodeServerMsg,
  encodeCharTyped,
  encodeHello,
  encodePointerPressed,
} from './messages.ts';

/** Rust `client_runtime::ReconnectPolicy` 对齐（budget 30s / interval 50ms）。 */
export interface ReconnectPolicy {
  budgetMs: number;
  intervalMs: number;
}

export const DEFAULT_RECONNECT: ReconnectPolicy = { budgetMs: 30_000, intervalMs: 50 };

export interface ConnectHandlers {
  onWelcome(w: Welcome): void;
  onHits(wid: bigint, hits: HitRegion[]): void;
  onFrame(f: { wid: bigint; frameId: bigint; revision: bigint; payload: DrawList }): void;
  /** 重连预算耗尽/不可恢复错误（onDead 后连接对象不再自愈）。 */
  onDead?(reason: string): void;
  onLog?(line: string): void;
}

export interface ConnectOptions {
  /** 完整 WS URL（含 token query：`ws://127.0.0.1:17800/?token=…`）。 */
  url: string;
  appName: string;
  title?: string;
  width?: number;
  height?: number;
  reconnect?: Partial<ReconnectPolicy>;
}

export interface RemoteConnection {
  sendPointerDown(x: number, y: number): void;
  sendChar(ch: string): void;
  close(): void;
  readonly connected: boolean;
}

/**
 * 连接宿主并订阅 `appName`。每条 WS Binary 消息 = 一个协议信封
 * （WsTransport 消息映射）；Text 帧忽略。断线在预算内自动重连并重发
 * Hello（订阅幂等——宿主按名重挂镜像）。
 */
export function connect(
  opts: ConnectOptions,
  handlers: ConnectHandlers,
  WS: typeof WebSocket = globalThis.WebSocket,
): RemoteConnection {
  const policy = { ...DEFAULT_RECONNECT, ...opts.reconnect };
  const startedAt = Date.now();
  let ws: WebSocket | null = null;
  let wid: bigint | null = null;
  let manualClose = false;
  let dead = false;

  const log = (line: string) => handlers.onLog?.(line);

  const sendRaw = (bytes: Uint8Array): void => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(bytes.slice().buffer as ArrayBuffer);
    }
  };

  const dial = (): void => {
    if (manualClose || dead) return;
    ws = new WS(opts.url);
    ws.binaryType = 'arraybuffer';
    ws.onopen = () => {
      log(`connected ${opts.url}`);
      sendRaw(encodeHello(opts));
    };
    ws.onmessage = (ev: MessageEvent) => {
      if (!(ev.data instanceof ArrayBuffer)) return; // Text/非二进制忽略
      let msg;
      try {
        msg = decodeServerMsg(new Uint8Array(ev.data));
      } catch (e) {
        log(`bad message: ${String(e)}`);
        return;
      }
      if (msg.kind === 'welcome') {
        wid = msg.welcome.wid;
        handlers.onWelcome(msg.welcome);
      } else if (msg.kind === 'hitTable') {
        handlers.onHits(msg.wid, msg.hits);
      } else if (msg.kind === 'frame') {
        handlers.onFrame(msg.frame);
      }
    };
    ws.onclose = () => {
      if (manualClose || dead) return;
      const elapsed = Date.now() - startedAt;
      if (elapsed >= policy.budgetMs) {
        dead = true;
        handlers.onDead?.(`reconnect budget exhausted (${policy.budgetMs}ms)`);
        return;
      }
      log(`disconnected; retry in ${policy.intervalMs}ms (budget ${policy.budgetMs - elapsed}ms left)`);
      setTimeout(dial, policy.intervalMs);
    };
    ws.onerror = () => {
      // 错误细节随 onclose 走重连路径；此处不重复处理。
    };
  };

  dial();

  return {
    get connected() {
      return ws?.readyState === WebSocket.OPEN && !manualClose && !dead;
    },
    sendPointerDown(x: number, y: number) {
      if (wid !== null) sendRaw(encodePointerPressed(wid, x, y));
    },
    sendChar(ch: string) {
      if (wid !== null) sendRaw(encodeCharTyped(wid, ch));
    },
    close() {
      manualClose = true;
      ws?.close();
    },
  };
}
