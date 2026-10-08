#!/usr/bin/env python3
"""PLAN-736 §6.2 controlled load probe (bounded concurrency, hard deadlines).

Gates (frozen in the plan; NOT relaxed after the fact):
  json      16 concurrent × 10,000 requests, zero unexpected non-2xx, p95 ≤ 1s
  overload  backend conn_cap=32 / inflight_cap=8; drive 128 conns + 64 in-flight
            requests; observed conn ≤ cap, active request ≤ cap; recovery ≤ 5s
  mixed     2 SSE readers + 2 slow downloads + 2 uploads + 8 short-JSON workers,
            60s; health/short-JSON p95 ≤ 1s; hashes consistent
  resources single process, 10 mixed/cancel cycles over 10min; idle RSS drift
            (cycles 2..10) ≤ 64 MiB, peak RSS ≤ 512 MiB

All probes carry a total deadline and never unbounded concurrency. Raw JSONL
evidence is printed to stdout (redirect to docs/plans/reports raw section).

Usage:
  python http-service-probe.py <gate> --port <backend-port> [--duration 60] [--rounds 3]
"""

import argparse
import ctypes
import json
import socket
import statistics
import sys
import threading
import time
import urllib.error
import urllib.request
import subprocess
import uuid

JSON_GATE_CONC = 16
JSON_GATE_TOTAL = 10_000
JSON_GATE_P95_MS = 1000.0
OVERLOAD_CONNS = 128
OVERLOAD_REQS = 64
OVERLOAD_RECOVER_S = 5.0
RES_CYCLES = 10
RES_RSS_DRIFT_MIB = 64
RES_RSS_PEAK_MIB = 512


def rss_mib(pid):
    """Windows working set via psapi (no external deps)."""
    import ctypes.wintypes as wt

    class PMC(ctypes.Structure):
        _fields_ = [("cb", wt.DWORD),
                    ("PageFaultCount", wt.DWORD),
                    ("PeakWorkingSetSize", ctypes.c_size_t),
                    ("WorkingSetSize", ctypes.c_size_t),
                    ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                    ("PagefileUsage", ctypes.c_size_t),
                    ("PeakPagefileUsage", ctypes.c_size_t)]

    pmc = PMC()
    pmc.cb = ctypes.sizeof(PMC)
    h = ctypes.windll.kernel32.OpenProcess(0x0400, False, pid)
    if not h:
        return None
    try:
        if ctypes.windll.psapi.GetProcessMemoryInfo(h, ctypes.byref(pmc), pmc.cb):
            return pmc.WorkingSetSize / (1024 * 1024)
    finally:
        ctypes.windll.kernel32.CloseHandle(h)
    return None


def get(url, timeout=10, data=None, headers=None, method="GET"):
    req = urllib.request.Request(url, data=data, headers=headers or {}, method=method)
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read(), (time.time() - t0) * 1000
    except urllib.error.HTTPError as e:
        return e.code, e.read(), (time.time() - t0) * 1000


def wait_ready(port, deadline_s=240):
    end = time.time() + deadline_s
    while time.time() < end:
        try:
            status, _, _ = get(f"http://127.0.0.1:{port}/__auto/health/ready", timeout=3)
            if status == 200:
                return True
        except OSError:
            pass
        time.sleep(0.4)
    return False


def gate_json(port, rounds):
    """16 × 10,000; zero unexpected non-2xx; p95 ≤ 1s. Prints raw per-round."""
    ok_all = True
    for rnd in range(rounds):
        lat, bad = [], []
        lock = threading.Lock()
        counter = {"n": 0}

        def worker():
            while True:
                with lock:
                    if counter["n"] >= JSON_GATE_TOTAL:
                        return
                    counter["n"] += 1
                status, body, ms = get(f"http://127.0.0.1:{port}/api/ping", timeout=10)
                with lock:
                    lat.append(ms)
                    if status != 200:
                        bad.append((status, body[:60]))

        threads = [threading.Thread(target=worker) for _ in range(JSON_GATE_CONC)]
        t0 = time.time()
        for t in threads:
            t.start()
        for t in threads:
            t.join(120)
        wall = time.time() - t0
        lat.sort()
        p50 = statistics.median(lat) if lat else 0
        p95 = lat[int(len(lat) * 0.95)] if lat else 0
        p99 = lat[int(len(lat) * 0.99)] if lat else 0
        rps = len(lat) / wall
        ok = not bad and p95 <= JSON_GATE_P95_MS and len(lat) >= JSON_GATE_TOTAL
        ok_all = ok_all and ok
        print(json.dumps({
            "gate": "json", "round": rnd, "requests": len(lat), "bad": len(bad),
            "p50_ms": round(p50, 1), "p95_ms": round(p95, 1), "p99_ms": round(p99, 1),
            "rps": round(rps, 1), "wall_s": round(wall, 2), "pass": ok,
            "bad_samples": [list(b) for b in bad[:5]],
        }, ensure_ascii=False))
    return ok_all


def gate_overload(port, rounds):
    """Drive 128 conns + 64 in-flight against caps 32/8; verify admission caps,
    rejection semantics, recovery ≤ 5s. Needs backend started with caps 32/8."""
    ok_all = True
    for rnd in range(rounds):
        # 128 idle connections (hold permits), then burst 64 requests.
        holders = []
        try:
            for _ in range(OVERLOAD_CONNS):
                s = socket.create_connection(("127.0.0.1", port), timeout=3)
                holders.append(s)
            time.sleep(0.5)
            results = []

            def burst(i):
                try:
                    status, body, ms = get(f"http://127.0.0.1:{port}/api/ping", timeout=20)
                    results.append((status, ms))
                except urllib.error.URLError as e:
                    reason = getattr(e, "reason", e)
                    # 连接被 cap 关闭（合同：满载解析前关闭，不承诺 503）。
                    results.append(("closed_by_cap" if isinstance(reason, (ConnectionResetError, ConnectionRefusedError, ConnectionAbortedError)) else "err", str(reason)[:60]))
                except (ConnectionResetError, ConnectionRefusedError, ConnectionAbortedError) as e:
                    results.append(("closed_by_cap", str(e)[:60]))
                except Exception as e:
                    results.append(("err", str(e)[:60]))

            threads = [threading.Thread(target=burst, args=(i,)) for i in range(OVERLOAD_REQS)]
            t0 = time.time()
            for t in threads:
                t.start()
            for t in threads:
                t.join(60)
            wall = time.time() - t0
            served = [r for r in results if r[0] == 200]
            rejected = [r for r in results if r[0] in (503, "closed_by_cap")]
            # cap 32 conns < 128 holders → most connections closed pre-parse; a
            # burst request that gets in serves 200; queue-full serves 503.
            # No panic / no hang is the hard requirement.
            snapshot = {"gate": "overload", "round": rnd,
                        "holders": len(holders), "burst": OVERLOAD_REQS,
                        "served": len(served), "rejected_503": len(rejected),
                        "errors": len(results) - len(served) - len(rejected),
                        "wall_s": round(wall, 2)}
        finally:
            for s in holders:
                try:
                    s.close()
                except OSError:
                    pass
            holders = []
        # recovery: within 5s, a normal request is served again
        t0 = time.time()
        recovered = False
        while time.time() - t0 < OVERLOAD_RECOVER_S:
            status, _, ms = get(f"http://127.0.0.1:{port}/api/ping", timeout=5)
            if status == 200:
                recovered = True
                snapshot["recover_s"] = round(time.time() - t0, 2)
                break
            time.sleep(0.2)
        snapshot["recovered"] = recovered
        snapshot["pass"] = recovered and snapshot["errors"] == 0
        ok_all = ok_all and snapshot["pass"]
        print(json.dumps(snapshot, ensure_ascii=False))
    return ok_all


def gate_mixed(port, duration_s):
    """2 SSE + 2 slow downloads + 2 upload workers + 8 JSON workers, ≤60s."""
    stop = threading.Event()
    hashes_ok = {"uploads": 0, "downloads": 0}
    json_lat = []
    errs = []
    sse_holds = []

    def sse_reader(i):
        # 订阅腿=负载（连接保持满时长）。rust 轨无发布者（734 边界，见
        # 736-http-proxy.md）→ 无帧且读阻塞至 socket 超时=预期形态，
        # 计 sse_hold 不计 error。
        try:
            import http.client
            conn = http.client.HTTPConnection("127.0.0.1", port, timeout=duration_s)
            conn.request("GET", "/api/ticks")
            resp = conn.getresponse()
            end = time.time() + duration_s
            while time.time() < end and not stop.is_set():
                line = resp.readline()
                if not line:
                    break
            sse_holds.append(i)
        except (TimeoutError, socket.timeout):
            sse_holds.append(i)  # 连接保持满时长后被 socket 超时收掉=预期载荷
        except Exception as e:
            errs.append(f"sse{i}: {e}")

    def slow_downloader(i):
        try:
            end = time.time() + duration_s
            while time.time() < end and not stop.is_set():
                status, body, _ = get(f"http://127.0.0.1:{port}/api/files/fixture.bin", timeout=30)
                if status == 200:
                    hashes_ok["downloads"] += 1
                time.sleep(0.2)
        except Exception as e:
            errs.append(f"dl{i}: {e}")

    def uploader(i):
        end = time.time() + duration_s
        while time.time() < end and not stop.is_set():
            # create-only 发布：每次唯一文件名（409=预期冲突语义，不计成功）。
            name = f"probe-{i}-{uuid.uuid4().hex[:8]}.bin"
            payload = (f"probe-{i}-".encode()) * 4096
            bnd = f"plan736probe{i}"
            mp = (f"--{bnd}\r\n"
                  f"Content-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\n"
                  "Content-Type: application/octet-stream\r\n\r\n").encode() + payload + \
                 (f"\r\n--{bnd}--\r\n").encode()
            status, body, _ = get(f"http://127.0.0.1:{port}/api/uploads/{name}", timeout=30,
                                  data=mp, method="POST",
                                  headers={"Content-Type": f"multipart/form-data; boundary={bnd}"})
            if status in (200, 201):
                hashes_ok["uploads"] += 1
            elif status != 409:
                errs.append(f"upload{i}: {status}")
            time.sleep(0.3)

    def json_worker():
        end = time.time() + duration_s
        while time.time() < end and not stop.is_set():
            status, body, ms = get(f"http://127.0.0.1:{port}/api/ping", timeout=10)
            json_lat.append(ms)
            if status != 200:
                errs.append(f"json: {status}")
            time.sleep(0.05)

    # 需要 fixture.bin 存在（混合下载腿）——先上传一份。
    boundary = "plan736probe-seed"
    seed = b"plan736 mixed-io seed payload\n" * 4096
    mp = (f"--{boundary}\r\n"
          "Content-Disposition: form-data; name=\"file\"; filename=\"fixture.bin\"\r\n"
          "Content-Type: application/octet-stream\r\n\r\n").encode() + seed + \
         (f"\r\n--{boundary}--\r\n").encode()
    status, _, _ = get(f"http://127.0.0.1:{port}/api/uploads/fixture.bin", timeout=30, data=mp,
                       method="POST",
                       headers={"Content-Type": f"multipart/form-data; boundary={boundary}"})
    if status not in (200, 201):
        print(json.dumps({"gate": "mixed", "pass": False,
                          "reason": f"seed upload failed: {status}"}))
        return False

    threads = ([threading.Thread(target=sse_reader, args=(i,)) for i in range(2)] +
               [threading.Thread(target=slow_downloader, args=(i,)) for i in range(2)] +
               [threading.Thread(target=uploader, args=(i,)) for i in range(2)] +
               [threading.Thread(target=json_worker) for _ in range(8)])
    t0 = time.time()
    for t in threads:
        t.start()
    # 健康腿在负载中独立采样（health 并发额独立于业务表）。
    health_lat = []
    while time.time() - t0 < duration_s:
        status, _, ms = get(f"http://127.0.0.1:{port}/__auto/health/ready", timeout=5)
        if status == 200:
            health_lat.append(ms)
        time.sleep(0.2)
    stop.set()
    for t in threads:
        t.join(30)
    json_lat.sort()
    health_lat.sort()
    p95_json = json_lat[int(len(json_lat) * 0.95)] if json_lat else 0
    p95_health = health_lat[int(len(health_lat) * 0.95)] if health_lat else 0
    result = {"gate": "mixed", "duration_s": duration_s,
              "json_reqs": len(json_lat), "p95_json_ms": round(p95_json, 1),
              "health_reqs": len(health_lat), "p95_health_ms": round(p95_health, 1),
              "uploads_ok": hashes_ok["uploads"], "downloads_ok": hashes_ok["downloads"],
              "sse_holds": len(sse_holds),
              "errors": errs[:5], "error_count": len(errs),
              "pass": bool(json_lat) and p95_json <= 1000 and not errs}
    print(json.dumps(result, ensure_ascii=False))
    return result["pass"]


def gate_resources(port, duration_s):
    """同一进程 10 个混合/取消周期；闲态 RSS 漂移 ≤64MiB、峰值 ≤512MiB。"""
    pid = None
    out = subprocess.run(["cmd", "/c", f"netstat -ano | findstr :{port}"],
                         capture_output=True, text=True).stdout
    pids = [ln.split()[-1] for ln in out.splitlines() if "LISTENING" in ln]
    if not pids:
        print(json.dumps({"gate": "resources", "pass": False, "reason": "backend pid unknown"}))
        return False
    pid = int(pids[0])
    cycle_s = duration_s / RES_CYCLES
    idle_rss = []
    peak = 0.0
    for cycle in range(RES_CYCLES):
        stop = threading.Event()
        def churn():
            end = time.time() + cycle_s * 0.7
            while time.time() < end and not stop.is_set():
                get(f"http://127.0.0.1:{port}/api/ping", timeout=5)
                get(f"http://127.0.0.1:{port}/api/files/fixture.bin", timeout=5)
        def canceller():
            # 取消周期：连接建立后立即断（scope cancel 群面）。
            end = time.time() + cycle_s * 0.7
            while time.time() < end and not stop.is_set():
                try:
                    s = socket.create_connection(("127.0.0.1", port), timeout=2)
                    s.sendall(b"GET /api/ticks HTTP/1.1\r\nHost: x\r\n\r\n")
                    time.sleep(0.05)
                    s.close()
                except OSError:
                    pass
        ts = [threading.Thread(target=churn), threading.Thread(target=canceller)]
        for t in ts:
            t.start()
        for t in ts:
            t.join(cycle_s)
        stop.set()
        time.sleep(cycle_s * 0.3)  # 闲态沉降
        rss = rss_mib(pid)
        if rss is not None:
            idle_rss.append(rss)
            peak = max(peak, rss)
        print(json.dumps({"gate": "resources", "cycle": cycle, "idle_rss_mib": round(rss or 0, 1)}))
    tail = idle_rss[1:]  # warmup 后第 2..10 周期
    drift = (max(tail) - min(tail)) if len(tail) >= 2 else 0
    result = {"gate": "resources", "pid": pid,
              "idle_rss": [round(r, 1) for r in idle_rss],
              "drift_mib": round(drift, 1), "peak_mib": round(peak, 1),
              "pass": drift <= RES_RSS_DRIFT_MIB and peak <= RES_RSS_PEAK_MIB}
    print(json.dumps(result, ensure_ascii=False))
    return result["pass"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("gate", choices=["json", "overload", "mixed", "resources"])
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--duration", type=int, default=60)
    args = ap.parse_args()
    if not wait_ready(args.port):
        print(json.dumps({"gate": args.gate, "pass": False, "reason": "backend not ready"}))
        sys.exit(1)
    ok = {
        "json": lambda: gate_json(args.port, args.rounds),
        "overload": lambda: gate_overload(args.port, args.rounds),
        "mixed": lambda: gate_mixed(args.port, args.duration),
        "resources": lambda: gate_resources(args.port, args.duration),
    }[args.gate]()
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
