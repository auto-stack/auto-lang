#!/usr/bin/env python3
"""PLAN-652 T-06 review-fix: gallery + standalone VM MCP real-machine probe."""
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

AUTO_BIN = r"D:\autostack\.wt\lang-652\auto-lang\target\debug\auto.exe"
GALLERY = r"D:\autostack\auto-os\ui-gallery"
CLOCK = r"D:\autostack\auto-lang\examples\ui\012-clock"
SCRATCH = r"D:\autostack\.wt\lang-652\auto-lang\scratch\p652"
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
    r = urllib.request.Request(
        url, data=data, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(r, timeout=12) as resp:
        return json.loads(resp.read().decode())


def result_text(res: dict) -> str:
    return res.get("result", res).get("content", [{}])[0].get("text", json.dumps(res))


def parse_state_text(text: str) -> dict:
    """Parse 'State:\\n  key: value (type)' lines into dict."""
    out: dict = {}
    for line in text.splitlines():
        m = re.match(r"\s+([A-Za-z0-9_./]+):\s+(.*)$", line)
        if not m:
            continue
        key, rest = m.group(1), m.group(2).strip()
        # strip type suffix " (str)"
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
    # prefer software/GL if wgpu discrete fails in agent session
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
    """Wait until autoui_state returns actual State (not empty placeholder)."""
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
                # some other content
                return True, text, samples
        except urllib.error.HTTPError as e:
            samples.append({"t": round(time.time() - t0, 2), "http": e.code})
        except Exception as e:
            samples.append({"t": round(time.time() - t0, 2), "err": f"{type(e).__name__}:{e}"})
        time.sleep(0.6)
    return False, last, samples


def fixture(port: int, state: dict) -> str:
    res = mcp_call(
        port,
        "autoui_fixture",
        {"schema_version": 1, "state": state},
    )
    return result_text(res)


def sample_series(port: int, fields: list[str], n: int, delay: float) -> list[dict]:
    series = []
    for i in range(n):
        try:
            text = result_text(mcp_call(port, "autoui_state", {"fields": fields}))
            st = parse_state_text(text)
            series.append({"i": i, "raw_head": text[:250], **{k: st.get(k) for k in fields + list(st)[:8]}})
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
        out["wait_samples_tail"] = samples[-5:]
        print("mcp_ready", ok, "text_head", text[:200], flush=True)
        if not ok:
            out["ok"] = False
            return out
        time.sleep(1.0)
        s1 = sample_series(port, ["w_local", "w_tick"], 5, 0.5)
        out["series"] = s1
        print("series", json.dumps(s1, ensure_ascii=False)[:800], flush=True)
        vals = [x.get("w_local") for x in s1]
        ticks = [x.get("w_tick") for x in s1]
        out["w_local_series"] = vals
        out["w_tick_series"] = ticks
        nonph = [v for v in vals if isinstance(v, str) and v and v != "--:--:--"]
        tick_set = {v for v in ticks if v not in (None, "")}
        out["clock_running"] = len(set(nonph)) >= 2 or len(tick_set) >= 2
        out["ok"] = True
        return out
    finally:
        kill(proc, log_f)
        # harvest log markers
        try:
            log_txt = open(log_path, encoding="utf-8", errors="replace").read()
            out["log_len"] = len(log_txt)
            out["log_has_first_sync"] = "first state sync" in log_txt
            out["log_has_mcp"] = "AutoUI MCP" in log_txt
            out["log_tail"] = log_txt[-2000:]
            for line in log_txt.splitlines():
                if "AutoUI MCP" in line or "first state sync" in line or "panic" in line.lower() or "error" in line.lower() and "schema" not in line.lower():
                    out.setdefault("log_markers", []).append(line[:240])
        except Exception as e:
            out["log_read_err"] = str(e)


def probe_gallery() -> dict:
    print("=== GALLERY ===", GALLERY, flush=True)
    if not os.path.isdir(GALLERY):
        return {"ok": False, "error": f"missing {GALLERY}"}
    proc, port, log_path, log_f = start(
        GALLERY,
        extra_env={"AUTO_GALLERY_APPS": r"D:\autostack\auto-lang\examples\ui"},
        log_name="gallery",
    )
    out = {"port": port, "log": log_path, "bin": AUTO_BIN}
    try:
        # gallery boot scans demos — allow long wait
        ok, text, samples = wait_mcp_ready(
            port,
            180.0,
            ["selected_id", "w_local", "w_tick"],
        )
        out["mcp_ready"] = ok
        out["first_state_text"] = text[:2000]
        out["wait_samples_tail"] = samples[-8:]
        print("gallery mcp_ready", ok, "head", text[:250], flush=True)
        if not ok:
            out["ok"] = False
            return out

        # Select 012-clock
        fx = fixture(port, {"selected_id": "012-clock"})
        out["fixture_select"] = fx[:500]
        print("fixture select", fx[:200], flush=True)
        time.sleep(1.5)  # mount + subscription refresh (hot_reload 500ms)
        s1 = sample_series(port, ["selected_id", "w_local", "w_tick"], 5, 0.55)
        out["after_select_series"] = s1
        print("after select", json.dumps(s1, ensure_ascii=False)[:800], flush=True)

        w1 = [x.get("w_local") for x in s1]
        t1 = [x.get("w_tick") for x in s1]
        out["w_local_series"] = w1
        out["w_tick_series"] = t1
        nonph = [v for v in w1 if isinstance(v, str) and v and v != "--:--:--"]
        tick_set = {v for v in t1 if v not in (None, "")}
        # growth: distinct time strings or distinct tick ints
        out["ac6_clock_running"] = len(set(nonph)) >= 2 or len(tick_set) >= 2
        print("clock_running", out["ac6_clock_running"], "nonph", nonph, "ticks", t1, flush=True)

        # Switch to 002-counter
        fx2 = fixture(port, {"selected_id": "002-counter"})
        out["fixture_switch"] = fx2[:500]
        time.sleep(1.0)
        s2 = sample_series(port, ["selected_id", "w_local", "w_tick"], 4, 0.55)
        out["after_switch_series"] = s2
        w2 = [x.get("w_local") for x in s2]
        t2 = [x.get("w_tick") for x in s2]
        out["after_switch_w_local"] = w2
        out["after_switch_w_tick"] = t2

        # parse numeric ticks
        def as_int(v):
            try:
                return int(v)
            except Exception:
                return None

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
        try:
            log_txt = open(log_path, encoding="utf-8", errors="replace").read()
            out["log_len"] = len(log_txt)
            out["log_has_first_sync"] = "first state sync" in log_txt
            out["log_tail"] = log_txt[-2500:]
            markers = []
            for line in log_txt.splitlines():
                if any(
                    k in line
                    for k in (
                        "AutoUI MCP",
                        "first state sync",
                        "panic",
                        "Running project",
                        "012-clock",
                        "error:",
                        "Error",
                    )
                ):
                    if "schema" in line.lower() or "prop `text`" in line:
                        continue
                    markers.append(line[:240])
            out["log_markers"] = markers[-40:]
        except Exception as e:
            out["log_read_err"] = str(e)


def main():
    report = {
        "auto_bin": AUTO_BIN,
        "bin_exists": os.path.isfile(AUTO_BIN),
        "ts": time.strftime("%Y-%m-%dT%H:%M:%S"),
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

    path = os.path.join(SCRATCH, "t06_mcp_evidence.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    report["evidence_path"] = path
    print("=== T06 SUMMARY ===")
    print(json.dumps({k: report[k] for k in report if k not in ("standalone", "gallery")}, indent=2))
    for key in ("standalone", "gallery"):
        g = report.get(key) or {}
        print(f"[{key}] ok={g.get('ok')} ready={g.get('mcp_ready')} "
              f"clock={g.get('clock_running', g.get('ac6_clock_running'))} "
              f"ac6={g.get('ac6_pass')} err={g.get('error')}")
        if g.get("w_local_series") is not None:
            print(f"  w_local={g.get('w_local_series')}")
            print(f"  w_tick={g.get('w_tick_series')}")
        if g.get("log_markers"):
            print("  log_markers:")
            for m in g["log_markers"][-12:]:
                print("   ", m)
    ac06 = bool((report.get("gallery") or {}).get("ac6_pass")) or bool(
        (report.get("standalone") or {}).get("clock_running")
        and (report.get("gallery") or {}).get("mcp_ready")
        and (report.get("gallery") or {}).get("ac6_stopped_after_switch") is not False
    )
    # Strict AC-06: gallery running + stopped after switch required; standalone is secondary smoke
    g = report.get("gallery") or {}
    s = report.get("standalone") or {}
    strict = bool(g.get("ac6_pass"))
    loose = bool(g.get("ac6_clock_running")) and bool(g.get("ac6_stopped_after_switch", True))
    print("AC06_STRICT:", "PASS" if strict else "FAIL")
    print("AC06_GALLERY_START:", "PASS" if g.get("ac6_clock_running") else "FAIL/UNKNOWN")
    print("AC06_GALLERY_STOP:", "PASS" if g.get("ac6_stopped_after_switch") else "FAIL/UNKNOWN")
    print("AC06_STANDALONE:", "PASS" if s.get("clock_running") else "FAIL/UNKNOWN")
    return 0 if strict else 2


if __name__ == "__main__":
    sys.exit(main())
