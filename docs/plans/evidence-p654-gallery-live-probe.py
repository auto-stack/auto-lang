#!/usr/bin/env python3
"""PLAN-654 gallery live probe — 652 regression + path observability."""
from __future__ import annotations

import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request

AUTO_BIN = r"D:\autostack\.wt\lang-654\auto-lang\target\debug\auto.exe"
GALLERY = r"D:\autostack\auto-os\ui-gallery"
CLOCK = r"D:\autostack\auto-lang\examples\ui\012-clock"
# Prefer worktree examples if present (same demos)
if os.path.isdir(r"D:\autostack\.wt\lang-654\auto-lang\examples\ui\012-clock"):
    CLOCK = r"D:\autostack\.wt\lang-654\auto-lang\examples\ui\012-clock"
GALLERY_APPS = r"D:\autostack\auto-lang\examples\ui"
if os.path.isdir(r"D:\autostack\.wt\lang-654\auto-lang\examples\ui"):
    GALLERY_APPS = r"D:\autostack\.wt\lang-654\auto-lang\examples\ui"
SCRATCH = r"D:\autostack\.wt\lang-654\auto-lang\scratch\p654"
os.makedirs(SCRATCH, exist_ok=True)


def pick_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def mcp_call(port: int, tool: str, args: dict | None = None) -> dict:
    url = f"http://127.0.0.1:{port}/mcp"
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool, "arguments": args or {}},
    }
    data = json.dumps(req).encode()
    r = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(r, timeout=12) as resp:
        return json.loads(resp.read().decode())


def result_text(res: dict) -> str:
    return res.get("result", res).get("content", [{}])[0].get("text", json.dumps(res))


def parse_state_text(text: str) -> dict:
    out: dict = {}
    for line in text.splitlines():
        m = re.match(r"\s+([A-Za-z0-9_./]+):\s+(.*)$", line)
        if not m:
            continue
        key, rest = m.group(1), m.group(2).strip()
        val = re.sub(r"\s+\([^)]*\)\s*$", "", rest)
        val = val.strip().strip('"')
        out[key] = val
    return out


def start(app_dir: str, extra_env: dict | None = None, log_name: str = "app"):
    port = pick_port()
    env = dict(os.environ)
    env["AUTOUI_MCP_PORT"] = str(port)
    env["AUTOUI_TEST_FIXTURES"] = "1"
    env["AUTOUI_HOT_RELOAD"] = "1"
    env["AUTOUI_TIMESOURCE_DEBUG"] = "1"
    env.setdefault("WGPU_BACKEND", "gl")
    if extra_env:
        env.update(extra_env)
    log_path = os.path.join(SCRATCH, f"{log_name}_stdout.log")
    log_f = open(log_path, "w", encoding="utf-8", errors="replace")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=app_dir,
        env=env,
        stdout=log_f,
        stderr=subprocess.STDOUT,
        text=True,
    )
    return proc, port, log_path, log_f


def wait_mcp_ready(port: int, timeout_s: float, marker_fields=None) -> tuple[bool, str, list]:
    samples = []
    t0 = time.time()
    last = ""
    while time.time() - t0 < timeout_s:
        try:
            res = mcp_call(port, "autoui_state", {"fields": marker_fields or []})
            text = result_text(res)
            last = text[:400]
            samples.append({"t": round(time.time() - t0, 2), "text": text[:500]})
            if "No state available" not in text and "State:" in text:
                return True, text, samples
            if "No UI available" not in text and "No state available" not in text and len(text) > 30:
                return True, text, samples
        except urllib.error.HTTPError as e:
            samples.append({"t": round(time.time() - t0, 2), "http": e.code})
        except Exception as e:
            samples.append({"t": round(time.time() - t0, 2), "err": f"{type(e).__name__}:{e}"})
        time.sleep(0.6)
    return False, last, samples


def fixture(port: int, state: dict) -> str:
    res = mcp_call(port, "autoui_fixture", {"schema_version": 1, "state": state})
    return result_text(res)


def sample_series(port: int, fields: list[str], n: int, delay: float) -> list[dict]:
    series = []
    for i in range(n):
        try:
            text = result_text(mcp_call(port, "autoui_state", {"fields": fields}))
            st = parse_state_text(text)
            series.append({"i": i, "raw_head": text[:250], **{k: st.get(k) for k in fields}})
        except Exception as e:
            series.append({"i": i, "error": str(e)})
        time.sleep(delay)
    return series


def kill(proc, log_f):
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except Exception:
        proc.kill()
    try:
        log_f.close()
    except Exception:
        pass


def harvest_log(log_path: str, out: dict):
    try:
        log_txt = open(log_path, encoding="utf-8", errors="replace").read()
        out["log_len"] = len(log_txt)
        out["log_has_first_sync"] = "first state sync" in log_txt
        out["log_has_mcp"] = "AutoUI MCP" in log_txt
        ts_paths = re.findall(r"\[TS_PATH\][^\n]*", log_txt)
        out["ts_path_lines"] = ts_paths[:40]
        out["ts_path_count"] = len(ts_paths)
        # path-bearing widget identities in UI_EVENT
        path_events = re.findall(r'widget="[^"]+@[^"]+"', log_txt)
        out["ui_event_with_path"] = path_events[:20]
        handlers = []
        for key in (
            "handler_Demo012Clock_Tick",
            "handler_App_Tick",
            "Demo012Clock",
            "VM_HANDLER_OK widget=Demo012Clock",
            "UI_EVENT",
            "panic",
            "error:",
        ):
            if key in log_txt:
                handlers.append(key)
        out["log_hits"] = handlers
        markers = []
        for line in log_txt.splitlines():
            if any(
                k in line
                for k in (
                    "AutoUI MCP",
                    "first state sync",
                    "TS_PATH",
                    "handler_Demo012Clock",
                    "handler_App_Tick",
                    "widget=\"Demo012Clock",
                    "012-clock",
                    "panic",
                )
            ):
                if "schema" in line.lower() or "prop `text`" in line:
                    continue
                markers.append(line[:240])
        out["log_markers"] = markers[-50:]
        out["log_tail"] = log_txt[-2000:]
    except Exception as e:
        out["log_read_err"] = str(e)


def as_int(v):
    try:
        return int(v)
    except Exception:
        return None


def probe_standalone() -> dict:
    print("=== STANDALONE 012-clock ===", CLOCK, flush=True)
    if not os.path.isdir(CLOCK):
        return {"ok": False, "error": f"missing {CLOCK}"}
    proc, port, log_path, log_f = start(CLOCK, log_name="standalone")
    out = {"port": port, "log": log_path, "bin": AUTO_BIN}
    try:
        ok, text, samples = wait_mcp_ready(port, 70.0, ["w_local", "w_tick"])
        out["mcp_ready"] = ok
        out["first_state_text"] = text[:1500]
        print("mcp_ready", ok, "head", text[:200], flush=True)
        if not ok:
            out["ok"] = False
            return out
        time.sleep(1.0)
        s1 = sample_series(port, ["w_local", "w_tick"], 5, 0.5)
        out["series"] = s1
        out["w_local_series"] = [x.get("w_local") for x in s1]
        out["w_tick_series"] = [x.get("w_tick") for x in s1]
        nonph = [v for v in out["w_local_series"] if isinstance(v, str) and v and v != "--:--:--"]
        tick_set = {v for v in out["w_tick_series"] if v not in (None, "")}
        out["clock_running"] = len(set(nonph)) >= 2 or len(tick_set) >= 2
        out["ok"] = True
        return out
    finally:
        kill(proc, log_f)
        harvest_log(log_path, out)


def probe_gallery() -> dict:
    print("=== GALLERY ===", GALLERY, flush=True)
    if not os.path.isdir(GALLERY):
        return {"ok": False, "error": f"missing {GALLERY}"}
    proc, port, log_path, log_f = start(
        GALLERY,
        extra_env={"AUTO_GALLERY_APPS": GALLERY_APPS},
        log_name="gallery",
    )
    out = {"port": port, "log": log_path, "bin": AUTO_BIN}
    try:
        ok, text, samples = wait_mcp_ready(port, 180.0, ["selected_id", "w_local", "w_tick"])
        out["mcp_ready"] = ok
        out["first_state_text"] = text[:2000]
        out["wait_samples_tail"] = samples[-8:]
        print("gallery mcp_ready", ok, "head", text[:250], flush=True)
        if not ok:
            out["ok"] = False
            return out

        # idle sample (before select) — expect w_local placeholder
        idle = sample_series(port, ["selected_id", "w_local", "w_tick"], 2, 0.4)
        out["idle_series"] = idle
        print("idle", json.dumps(idle, ensure_ascii=False)[:400], flush=True)

        fx = fixture(port, {"selected_id": "012-clock"})
        out["fixture_select"] = fx[:500]
        print("fixture select", fx[:200], flush=True)
        time.sleep(1.8)
        s1 = sample_series(port, ["selected_id", "w_local", "w_tick"], 5, 0.55)
        out["after_select_series"] = s1
        w1 = [x.get("w_local") for x in s1]
        t1 = [x.get("w_tick") for x in s1]
        out["w_local_series"] = w1
        out["w_tick_series"] = t1
        nonph = [v for v in w1 if isinstance(v, str) and v and v != "--:--:--"]
        tick_set = {v for v in t1 if v not in (None, "")}
        out["ac6_clock_running"] = len(set(nonph)) >= 2 or len(tick_set) >= 2
        print("clock_running", out["ac6_clock_running"], "nonph", nonph, "ticks", t1, flush=True)

        fx2 = fixture(port, {"selected_id": "002-counter"})
        out["fixture_switch"] = fx2[:500]
        time.sleep(1.0)
        s2 = sample_series(port, ["selected_id", "w_local", "w_tick"], 4, 0.55)
        out["after_switch_series"] = s2
        w2 = [x.get("w_local") for x in s2]
        t2 = [x.get("w_tick") for x in s2]
        out["after_switch_w_local"] = w2
        out["after_switch_w_tick"] = t2

        last_t1 = None
        for v in reversed(t1):
            iv = as_int(v)
            if iv is not None:
                last_t1 = iv
                break
        growth = []
        if last_t1 is not None:
            for v in t2:
                iv = as_int(v)
                if iv is not None and iv > last_t1 + 1:
                    growth.append(iv)
        out["tick_growth_after_switch"] = growth
        out["ac6_stopped_after_switch"] = len(growth) == 0
        out["ac6_pass"] = bool(out["ac6_clock_running"] and out["ac6_stopped_after_switch"])
        out["ok"] = True
        return out
    finally:
        kill(proc, log_f)
        harvest_log(log_path, out)


def main():
    report = {
        "auto_bin": AUTO_BIN,
        "bin_exists": os.path.isfile(AUTO_BIN),
        "ts": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "plan": "PLAN-654 gallery live",
        "gallery_apps": GALLERY_APPS,
        "clock": CLOCK,
    }
    try:
        report["standalone"] = probe_standalone()
    except Exception as e:
        report["standalone"] = {"ok": False, "error": repr(e)}
        print("standalone exception", e, flush=True)
    try:
        report["gallery"] = probe_gallery()
    except Exception as e:
        report["gallery"] = {"ok": False, "error": repr(e)}
        print("gallery exception", e, flush=True)

    g = report.get("gallery") or {}
    s = report.get("standalone") or {}

    # Path observability checks
    path_obs = {
        "gallery_ts_path_count": g.get("ts_path_count", 0),
        "gallery_ts_path_lines": g.get("ts_path_lines", []),
        "gallery_ui_event_with_path": g.get("ui_event_with_path", []),
        "gallery_log_hits": g.get("log_hits", []),
        "standalone_ts_path_count": s.get("ts_path_count", 0),
        "standalone_log_hits": s.get("log_hits", []),
        "has_handler_demo_tick": any(
            "Demo012Clock" in h or "handler_Demo012Clock" in h for h in (g.get("log_hits") or [])
        )
        or any("handler_Demo012Clock_Tick" in m for m in (g.get("log_markers") or []))
        or any("handler_Demo012Clock_Tick" in m for m in (s.get("log_markers") or [])),
        "has_ts_path_in_log": bool(g.get("ts_path_count") or s.get("ts_path_count")),
        "path_in_ui_event": bool(g.get("ui_event_with_path") or s.get("ui_event_with_path")),
    }
    report["path_observability"] = path_obs

    path = os.path.join(SCRATCH, "gallery_live_evidence.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    report["evidence_path"] = path

    print("=== PLAN-654 GALLERY LIVE SUMMARY ===")
    print("evidence:", path)
    print("[standalone] ok=%s ready=%s clock=%s" % (
        s.get("ok"), s.get("mcp_ready"), s.get("clock_running")))
    print("  w_local=", s.get("w_local_series"))
    print("  w_tick=", s.get("w_tick_series"))
    print("[gallery] ok=%s ready=%s clock=%s stopped=%s ac6=%s" % (
        g.get("ok"), g.get("mcp_ready"), g.get("ac6_clock_running"),
        g.get("ac6_stopped_after_switch"), g.get("ac6_pass")))
    print("  idle w_local=", [x.get("w_local") for x in (g.get("idle_series") or [])])
    print("  after_select w_local=", g.get("w_local_series"))
    print("  after_select w_tick=", g.get("w_tick_series"))
    print("  after_switch w_tick=", g.get("after_switch_w_tick"))
    print("  growth_after_switch=", g.get("tick_growth_after_switch"))
    print("[path_obs]", json.dumps(path_obs, ensure_ascii=False)[:800])
    for key in ("gallery", "standalone"):
        gg = report.get(key) or {}
        if gg.get("log_markers"):
            print(f"  log_markers[{key}]:")
            for m in gg["log_markers"][-15:]:
                print("   ", m)

    reg = "PASS" if g.get("ac6_pass") else ("PARTIAL" if g.get("ac6_clock_running") else "FAIL")
    stand = "PASS" if s.get("clock_running") else "FAIL/UNKNOWN"
    print("REGRESSION_652_GALLERY:", reg)
    print("REGRESSION_652_STANDALONE:", stand)
    print("PATH_OBS_LOG:", "PASS" if path_obs["has_ts_path_in_log"] else "FAIL/UNKNOWN")
    print("PATH_OBS_HANDLER:", "PASS" if path_obs["has_handler_demo_tick"] else "FAIL/UNKNOWN")
    return 0 if g.get("ac6_pass") and s.get("clock_running") else 2


if __name__ == "__main__":
    sys.exit(main())
