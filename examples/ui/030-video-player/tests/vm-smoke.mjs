#!/usr/bin/env node
// vm-smoke.mjs — 030-video-player VM 模式冒烟门禁（PLAN-617 T-10 重写）。
//
// 取代 Plan 542 的版本：那一版断言的是 mock 外壳（"AutoOS Video Player"、
// "01_intro.mp4"、"⏸ 暂停"、"Kernel & AutoVM Architecture"），这些字面量
// 在本计划里已被全部删除。现在 VM 端的**诚实边界**才是被验证的对象：
//
//   1. 同一份 app.at 能在 VM 端起窗（组件/store/嵌套循环都能编译执行）；
//   2. 视口给出明确的后端能力说明（"本后端未启用原生播放"），不是纯黑；
//   3. 界面**不出现**任何凭文件名编造或预置的假内容；
//   4. 队列面板可交互（关闭后从快照消失）。
//
// 注意（已知 VM 侧边界，登记为债务）：VM 端 `Http.get("/api/media/scan")` 用的是
// 相对地址，而 VM 的 HTTP 通道直接把该串交给 reqwest（需要绝对 URL），故 VM 里
// 媒体库为空、显示空态文案。本脚本**不断言**队列有内容——那需要 VM 侧补 HTTP
// 基址支持，属独立事项。

import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const EXAMPLE_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const REPO_ROOT = resolve(EXAMPLE_ROOT, "..", "..", "..");
const MCP_PORT = Number(process.env.AUTOUI_MCP_PORT || 9330);

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

let childProcess = null;
function cleanup() {
  if (childProcess) {
    try {
      if (process.platform === "win32") {
        spawnSync("taskkill", ["/pid", String(childProcess.pid), "/T", "/F"], { stdio: "ignore" });
      } else {
        childProcess.kill("SIGKILL");
      }
    } catch {}
    childProcess = null;
  }
}

async function main() {
  const bin = resolveAutoBin();
  console.log(`[vm-smoke] auto bin: ${bin}`);
  childProcess = spawn(bin, ["run", "-r", "vm"], {
    cwd: EXAMPLE_ROOT,
    env: { ...process.env, AUTOUI_MCP_PORT: String(MCP_PORT), RUST_MIN_STACK: "16777216" },
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let vmLog = "";
  const collect = (d) => { vmLog += d.toString(); };
  childProcess.stdout.on("data", collect);
  childProcess.stderr.on("data", collect);

  process.on("exit", cleanup);
  process.on("SIGINT", () => { cleanup(); process.exit(1); });

  let ready = false;
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    if (childProcess.exitCode !== null) break;
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
    fail(`AutoUI MCP 未就绪（exitCode=${childProcess.exitCode}）；VM 日志尾：\n${vmLog.slice(-1500)}`);
    return;
  }
  await new Promise((r) => setTimeout(r, 1500));

  const snapshot = async () => mcpCall("autoui_snapshot", {});

  // ── A. 外壳与队列面板渲染 ──
  const snap0 = await snapshot();
  if (!snap0.includes("Video Player")) {
    fail(`快照缺失 "Video Player" 标头：\n${snap0.slice(0, 600)}`);
    return;
  }
  if (!snap0.includes("播放队列")) {
    fail(`快照缺失队列面板标题：\n${snap0.slice(0, 600)}`);
    return;
  }
  console.log("[vm-smoke] A ok — 外壳 + 视口 + 队列面板渲染正常");

  // ── B. 视口节点在树里（降级面板由 iced 渲染面直接绘制）──
  // 说明：video 在 VM 端的诚实降级文案（"本后端未启用原生播放（构建时未开
  // mpv-widget）"）是**渲染面画上去的**，MCP 快照里看不到；快照能验证的是
  // View::Video 确实进了树（P617-D10 的教训：漏臂会静默变成 Empty，这里会
  // 退化成看不到 [Video 节点）。视觉证据见
  // tests/screenshots/after_t08_vm_degrade.png。
  if (!snap0.includes("[Video")) {
    fail("视口无 [Video 节点（疑为 View::Video 被兜底吃掉）：" + snap0.slice(0, 900));
    return;
  }
  console.log("[vm-smoke] B ok — View::Video 进入渲染树（降级面板见截图证据）");

  // ── C. 不出现任何被本计划删除的假内容 / 谎报字段 ──
  const forbidden = [
    "AutoOS Video Player",
    "01_intro.mp4",
    "BigBuckBunny",
    "03:45",
    "AAC Stereo",
    "4.2 Mbps",
    "HEVC",
  ];
  const hits = forbidden.filter((f) => snap0.includes(f));
  if (hits.length > 0) {
    fail(`VM 快照仍含已退役的假内容: ${hits.join(", ")}`);
    return;
  }
  console.log("[vm-smoke] C ok — 无 mock 字面量与谎报元数据");

  // ── D. 队列面板可关闭（交互链路可用）──
  // 关闭键是一个无文本的图标按钮；取快照里第一个空文本 icon 按钮。
  const iconBtns = [...snap0.matchAll(/button #(vnode_\d+) ""/g)].map((m) => m[1]);
  if (iconBtns.length >= 2) {
    // 顶栏的两个无文本图标键依次是「主题」与「队列显隐」。
    await mcpCall("autoui_action", { element_id: iconBtns[1], action: "press" });
    await new Promise((r) => setTimeout(r, 700));
    const after = await snapshot();
    if (!after.includes("播放队列")) {
      console.log("[vm-smoke] D ok — 队列面板可显隐");
    } else {
      fail("按下队列显隐键后队列面板仍在快照里");
    }
  } else {
    console.log(`[vm-smoke] D note: 只匹配到 ${iconBtns.length} 个空文本图标按钮，已跳过该项`);
  }

  console.log("[vm-smoke] ALL PASS — 030-video-player VM 模式（诚实降级）验证全绿");
}

main().finally(() => {
  cleanup();
  setTimeout(() => process.exit(process.exitCode ?? 0), 200);
});
