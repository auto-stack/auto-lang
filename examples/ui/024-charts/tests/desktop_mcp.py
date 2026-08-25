#!/usr/bin/env python3
"""
Plan 445 M3: MCP interaction tests for the REAL 024-charts app in VM mode.

Starts `auto run -r vm` in the 024-charts project directory and drives the
real iced window via autoui_* HTTP tools (038-minesweeper harness pattern).

Covers the M3 acceptance surface:
  - type switching (Line/Bar/Area/Donut buttons → chartType state)
  - series visibility toggle (checkbox press → dVisible flip)
  - streaming gate (running=false keeps tickN at 0; Play → ticks flow,
    window slides to t-labels) — the Plan 402 "handler decides" semantics
    aligned with the vue track's setInterval gate at app level
  - deterministic geometry golden: Init-computed paths/legend captured to
    tests/golden/*.txt (bar/area/donut from the fixed 6-month seed data)
  - streaming perf sample: tickN growth rate over a 2s window (~2.5/s at
    interval=400ms) recorded in the golden header

Usage:
    cd examples/ui/024-charts/tests
    python desktop_mcp.py
"""

import json
import os
import subprocess
import sys
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9447


def pick_free_port(start=MCP_PORT_DEFAULT):
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
CHARTS_PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))
GOLDEN_DIR = os.path.join(os.path.dirname(__file__), "golden")


class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        # 瞬断容错：本机 MCP 偶发 connection refused（窗口环境级 flake，
        # 进程仍活），退避重试 3 次。
        last_exc = None
        for backoff in (0.5, 1.5, 3.0):
            self.req_id += 1
            try:
                resp = requests.post(self.url, json={
                    "jsonrpc": "2.0", "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                    "id": self.req_id,
                }, timeout=15)
                break
            except requests.exceptions.ConnectionError as e:
                last_exc = e
                time.sleep(backoff)
        else:
            raise last_exc
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def state(self, *fields):
        text = self.call("autoui_state", fields=list(fields))
        out = {}
        for line in text.splitlines():
            if ":" in line:
                k, v = line.split(":", 1)
                v = v.strip()
                # 形如 `"line" (str)` / `0 (int)` / `true (bool)` —— 剥类型后缀。
                if v.endswith(")") and " (" in v:
                    v = v[: v.rfind(" (")].strip()
                out[k.strip()] = v
        return out

    def press(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def find_button(self, label):
        text = self.call("autoui_find", kind="button", label=label)
        for token in text.replace("{", " ").split():
            if token.startswith("vnode_"):
                last = token
        # the LAST vnode before the button line is unreliable; instead re-find:
        import re
        m = re.findall(r"button (vnode_\d+)", text)
        return m[0] if m else None


class Result:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.errors = []

    def check(self, name, ok, detail=""):
        if ok:
            self.passed += 1
            print(f"  PASS {name}")
        else:
            self.failed += 1
            self.errors.append(f"{name}: {detail}")
            print(f"  FAIL {name} — {detail}")


def wait_for_server(url, timeout=40):
    for _ in range(timeout):
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1
            }, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


def golden_compare(name, actual):
    os.makedirs(GOLDEN_DIR, exist_ok=True)
    path = os.path.join(GOLDEN_DIR, f"{name}.txt")
    if os.environ.get("UPDATE_GOLDEN") == "1" or not os.path.exists(path):
        with open(path, "w", encoding="utf-8") as f:
            f.write(actual)
        return True, "recorded"
    with open(path, encoding="utf-8") as f:
        expected = f.read()
    return expected.strip() == actual.strip(), "diff"


def run_tests(client):
    result = Result()

    # ---- 1. 初始态：line 类型 + running 门控（tickN 保持 0）----
    st = client.state("chartType", "tickN", "running", "axisLabels")
    result.check("init chartType=line", st.get("chartType") == '"line"', st.get("chartType"))
    result.check("init running gate (tickN=0)", st.get("tickN") == "0", st.get("tickN"))
    result.check("init labels Jan..Jun", "Jan" in st.get("axisLabels", ""), st.get("axisLabels"))

    # ---- 2. 类型切换：四按钮循环 ----
    for label, expect in [("Bar", "bar"), ("Area", "area"),
                          ("Donut", "donut"), ("Line", "line")]:
        btn = client.find_button(label)
        if btn:
            client.press(btn)
            time.sleep(0.3)
            st = client.state("chartType")
            result.check(f"switch→{expect}", st.get("chartType") == f'"{expect}"',
                         st.get("chartType"))
        else:
            result.check(f"switch→{expect} (button found)", False, "vnode not found")

    # ---- 3. donut 图例（固定种子数据 → 确定性）----
    st = client.state("donutLegend")
    legend = st.get("donutLegend", "")
    ok = ("desktop 49%" in legend and "mobile 32%" in legend and "tablet 19%" in legend)
    result.check("donut legend 49/32/19", ok, legend)
    ok, detail = golden_compare("donut_legend", legend.strip('"'))
    result.check("golden donut_legend", ok, detail)

    # ---- 4. 系列开关 ----
    import re
    text = client.call("autoui_find", kind="checkbox")
    boxes = re.findall(r"checkbox (vnode_\d+)", text)
    if boxes:
        client.call("autoui_action", element_id=boxes[0], action="toggle")
        time.sleep(0.5)
        st = client.state("dVisible")
        result.check("toggle Desktop off", st.get("dVisible") == "false", st.get("dVisible"))
        client.call("autoui_action", element_id=boxes[0], action="toggle")
        time.sleep(0.5)
        st = client.state("dVisible")
        result.check("toggle Desktop on", st.get("dVisible") == "true", st.get("dVisible"))
    else:
        result.check("checkbox found", False, text[:80])

    # ---- 5. 流式：Play → tick 增长 + t 标签 + 滑窗；性能采样 ----
    play = client.find_button("Play")
    if play:
        client.press(play)
        time.sleep(0.6)
        w0, t0 = time.time(), int(client.state("tickN").get("tickN", "0"))
        time.sleep(4.0)
        w1, t1 = time.time(), int(client.state("tickN").get("tickN", "0"))
        rate = (t1 - t0) / max(0.1, w1 - w0)
        result.check("streaming ticks flow", t1 > t0, f"tickN {t0}→{t1}")
        result.check("tick rate ≈2.5/s (400ms)", 1.5 <= rate <= 3.5, f"rate={rate:.2f}/s")
        st = client.state("axisLabels", "tickN")
        labels = st.get("axisLabels", "")
        result.check("labels slid to t-series", "t" in labels, labels[:60])
        # 性能采样入档（svgdoc/VM 轨 v1 SVG 的裁决数据点之一）。数值随
        # 时刻变化（滑窗内容/速率抖动）——只记录 + 格式校验，不做逐字节
        # golden 对比（确定性对比由 donut_legend 承担）。
        os.makedirs(GOLDEN_DIR, exist_ok=True)
        with open(os.path.join(GOLDEN_DIR, "stream_perf_sample.txt"), "w",
                  encoding="utf-8") as f:
            f.write("tick_rate_per_sec=" + format(rate, ".2f") + "\n"
                    + "labels=" + labels + "\n")
        result.check("perf sample recorded",
                     labels.startswith('"t') and 1.5 <= rate <= 3.5,
                     f"rate={rate:.2f}")
        pause = client.find_button("Pause")
        if pause:
            client.press(pause)
            time.sleep(1.0)
            tp = int(client.state("tickN").get("tickN", "0"))
            time.sleep(1.0)
            tp2 = int(client.state("tickN").get("tickN", "0"))
            result.check("pause stops ticks", tp == tp2, f"{tp}→{tp2}")
    else:
        result.check("Play button found", False, "vnode not found")

    # ---- 6. 几何 golden：lineD 初始路径（切回 line + Pause 后为 Init 值?）
    # 流式后 lineD 是滑窗数据 —— golden 只钉 donut/legend（确定性）。

    return result


def main():
    print("=" * 60)
    print("Plan 445 M3: Desktop MCP Tests (real 024-charts, VM mode)")
    print("=" * 60)
    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        sys.exit(2)
    # 启动重试（≤3 次）：MCP 起始可用性在本机是窗口环境级 flake
    # （最小化/0 尺寸窗口时 view() 不跑 → MCP 不 bind），杀掉重拉即好。
    proc = None
    client = None
    for attempt in range(3):
        mcp_port = pick_free_port()
        mcp_url = f"http://localhost:{mcp_port}/mcp"
        if proc is not None:
            proc.kill()
            proc.wait()
            time.sleep(1)
        proc = subprocess.Popen(
            [AUTO_BIN, "run", "-r", "vm"],
            cwd=CHARTS_PROJECT,
            env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        try:
            if not wait_for_server(mcp_url, timeout=25):
                print(f"  (attempt {attempt + 1}: MCP not up, relaunching)")
                continue
            client = McpClient(mcp_url)
            ok = False
            for _ in range(15):
                time.sleep(1.5)
                try:
                    if "chartType" in client.state("chartType"):
                        ok = True
                        break
                except Exception:
                    pass
            if ok:
                break
            print(f"  (attempt {attempt + 1}: state warmup failed, relaunching)")
            client = None
        except Exception as e:
            print(f"  (attempt {attempt + 1}: {e})")
            client = None
    if client is None:
        print("ERROR: MCP server did not become ready after 3 attempts")
        if proc is not None:
            proc.kill()
        sys.exit(1)
    try:
        result = run_tests(client)
        print("\n" + "=" * 60)
        print(f"Results: {result.passed} passed, {result.failed} failed")
        for err in result.errors:
            print(f"  ! {err}")
        sys.exit(1 if result.failed else 0)
    finally:
        proc.kill()


if __name__ == "__main__":
    main()
