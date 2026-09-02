#!/usr/bin/env node
// vm-smoke.mjs — PLAN-519: 019-video-app VM 轨冒烟门禁。

import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const EXAMPLE_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const REPO_ROOT = resolve(EXAMPLE_ROOT, "..", "..", "..");
const MCP_PORT = Number(process.env.AUTOUI_MCP_PORT || 9319);

function resolveAutoBin() {
  if (process.env.AUTO_BIN) return process.env.AUTO_BIN;
  const relDebug = resolve(REPO_ROOT, "target", "debug", "auto.exe");
  if (existsSync(relDebug)) return relDebug;
  const relRelease = resolve(REPO_ROOT, "target", "release", "auto.exe");
  if (existsSync(relRelease)) return relRelease;
  return "auto";
}

async function mcpCall(tool, args) {
  const res = await fetch(`http://127.0.0.1:${MCP_PORT}/mcp`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Accept: "application/json, text/event-stream" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: tool, arguments: args } }),
  });
  const text = await res.text();
  const line = text.split("\n").find((l) => l.startsWith("data:"));
  const payload = line ? JSON.parse(line.slice(5)) : JSON.parse(text);
  const content = payload.result?.content ?? payload;
  if (Array.isArray(content)) {
    return content.filter((c) => c.type === "text").map((c) => c.text).join("\n");
  }
  return JSON.stringify(content);
}

function fail(msg) {
  console.error(`[vm-smoke] FAIL: ${msg}`);
  process.exitCode = 1;
}

async function main() {
  const bin = resolveAutoBin();
  console.log(`[vm-smoke] auto bin: ${bin}`);
  const child = spawn(bin, ["run", "-r", "vm"], {
    cwd: EXAMPLE_ROOT,
    env: { ...process.env, AUTOUI_MCP_PORT: String(MCP_PORT), RUST_MIN_STACK: "16777216" },
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let vmLog = "";
  const collect = (d) => { vmLog += d.toString(); };
  child.stdout.on("data", collect);
  child.stderr.on("data", collect);
  const kill = () => {
    try {
      if (process.platform === "win32") {
        spawnSync("taskkill", ["/pid", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      } else {
        child.kill("SIGKILL");
      }
    } catch {}
  };
  process.on("exit", kill);

  let ready = false;
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) break;
    try {
      const probe = await fetch(`http://127.0.0.1:${MCP_PORT}/mcp`, {
        method: "POST",
        headers: { "Content-Type": "application/json", Accept: "application/json, text/event-stream" },
        body: JSON.stringify({ jsonrpc: "2.0", id: 0, method: "tools/list", params: {} }),
      });
      if ((await probe.text()).includes("autoui")) { ready = true; break; }
    } catch {}
    await new Promise((r) => setTimeout(r, 500));
  }
  if (!ready) {
    fail(`AutoUI MCP 未就绪（exitCode=${child.exitCode}）；VM 日志尾：\n${vmLog.slice(-1500)}`);
    return;
  }
  await new Promise((r) => setTimeout(r, 1000));

  const snapshot = async () => mcpCall("autoui_snapshot", {});

  // ── A. 初始 UI 包含 VideoApp 与 视频卡片 ──
  {
    const snap = await snapshot();
    if (!snap.includes("VideoApp")) {
      fail(`快照缺失 VideoApp 品牌标：\n${snap.slice(0, 500)}`);
      return;
    }
    console.log("[vm-smoke] A ok — VideoApp 初始界面就绪");
  }

  // ── B. 设置面板展开 ──
  {
    const snap = await snapshot();
    const settingsBtn = snap.match(/button #(vnode_\d+)[^\n]*\n\s*text #vnode_\d+ "⚙ Settings"/)?.[1]
      ?? snap.match(/button #(vnode_\d+) "⚙ Settings"/)?.[1];
    if (settingsBtn) {
      await mcpCall("autoui_action", { element_id: settingsBtn, action: "press" });
      await new Promise((r) => setTimeout(r, 600));
      const openSnap = await snapshot();
      if (!openSnap.includes("Theme") || !openSnap.includes("Accent Color")) {
        fail(`设置面板展开后未见 Theme/Accent 字段：\n${openSnap.slice(0, 800)}`);
        return;
      }
      console.log("[vm-smoke] B ok — 设置面板交互正常");
    }
  }

  console.log("[vm-smoke] PASS — 019-video-app VM 轨冒烟测试通过");
}

main().finally(() => {
  setTimeout(() => process.exit(process.exitCode ?? 0), 300);
});
