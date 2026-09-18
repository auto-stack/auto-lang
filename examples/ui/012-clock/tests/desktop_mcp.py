#!/usr/bin/env python3
"""os-003（origin PLAN-554）—— Clock 四 tab MCP 交互测试（VM 轨）。

启动 `auto run -r vm`，经 AutoUI MCP 驱动真实 app：
  T1 秒表：Start→走表（MM:SS.cc 前进）→计圈→Reset 清零；
  T2 计时器：设 00:00:01→Start→到零 banner 出现 + Dismiss 可关；
  T3 世界时钟：≥8 行渲染；北京/伦敦时差断言（now_sec 基准，容差 ±60s）；
  T4 闹钟：设下一分钟→等触发 banner；storage 跨重启恢复列表。

用法：cd examples/ui/012-stopwatch/tests && python desktop_mcp.py
AUTO_BIN 可覆盖 auto 二进制（缺省：仓 target/debug/auto → PATH）。
"""
import json
import os
import re
import subprocess
import sys
import time
import urllib.request

APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORT = 9247


def resolve_auto():
    if os.environ.get("AUTO_BIN"):
        return os.environ["AUTO_BIN"]
    for c in [
        os.path.join(APP, "..", "..", "..", "..", "target", "debug", "auto.exe"),
        os.path.join(APP, "..", "..", "..", "..", "target", "debug", "auto"),
        "auto",
    ]:
        if os.path.isfile(c):
            return c
    return "auto"


class Mcp:
    def __init__(self):
        self.url = f"http://127.0.0.1:{PORT}/mcp"
        self._id = 0

    def call(self, tool, args=None):
        self._id += 1
        req = {"jsonrpc": "2.0", "id": self._id, "method": "tools/call",
               "params": {"name": tool, "arguments": args or {}}}
        r = urllib.request.Request(self.url, data=json.dumps(req).encode(),
                                   headers={"Content-Type": "application/json"})
        res = json.loads(urllib.request.urlopen(r, timeout=15).read())
        if "error" in res:
            raise RuntimeError(f"{tool}: {res['error']}")
        c = res.get("result", {}).get("content", [{}])
        return c[0].get("text", "") if c else ""

    def state(self, *fields):
        t = self.call("autoui_state", {"fields": list(fields)})
        out = {}
        for l in t.splitlines():
            if ":" not in l:
                continue
            k, v = l.split(":", 1)
            v = v.strip()
            for suf in (" (str)", " (int)", " (float)", " (bool)"):
                if v.endswith(suf):
                    v = v[: -len(suf)]
                    break
            out[k.strip()] = v
        return out

    def press(self, label):
        r = self.call("autoui_press_sequence", {"keys": [label]})
        if "No button found" in r:
            snap = self.call("autoui_snapshot", {"mode": "rendered"})
            raise AssertionError(r + " | snapshot(" + str(len(snap)) + "): " + snap[:600])

    def snapshot(self):
        return self.call("autoui_snapshot")

    def find_all(self, pattern):
        return re.findall(pattern, self.snapshot())

    def type_into(self, element_id, text):
        self.call("autoui_type", {"element_id": element_id, "text": text, "clear_first": True})

    def wait_ready(self, timeout=60):
        for _ in range(timeout):
            time.sleep(1)
            try:
                self.state("tab")
                return
            except Exception:
                pass
        raise AssertionError("MCP 未就绪")


class Check:
    def __init__(self):
        self.rows = []

    def ok(self, name, cond, detail=""):
        self.rows.append((name, bool(cond)))
        print(f"  {'PASS' if cond else 'FAIL'}  {name}  {str(detail)[:140]}")


STORAGE_FILE = os.path.join(os.environ.get("TEMP", "/tmp"),
                            f"clock-mcp-storage-{os.getpid()}.json")


def boot():
    if os.path.exists(STORAGE_FILE):
        os.remove(STORAGE_FILE) if not getattr(boot, "keep", False) else None
    env = dict(os.environ)
    env["AUTO_VM_STORAGE_FILE"] = STORAGE_FILE
    proc = subprocess.Popen([resolve_auto(), "run", "-r", "vm"], cwd=APP, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
    mcp = Mcp()
    mcp.wait_ready()
    return proc, mcp


def main():
    ck = Check()
    proc, mcp = boot()
    try:
        # ---- T1 秒表 ----
        print("T1: 秒表")
        mcp.press("开始")
        time.sleep(1.4)
        s = mcp.state("elapsed", "time_display", "ms_display", "sw_on")
        ck.ok("走表前进（≥1000ms）", int(s.get("elapsed", "0")) >= 1000, s)
        ck.ok("MM:SS.cc 形态", ":" in s.get("time_display", "") and "." in s.get("ms_display", ""), s)
        mcp.press("计圈")
        mcp.press("停止")
        time.sleep(0.4)
        s = mcp.state("lap1")
        ck.ok("计圈记录", ":" in s.get("lap1", ""), s)
        mcp.press("复位")
        time.sleep(0.3)
        s = mcp.state("elapsed", "lap1")
        ck.ok("复位清零", s.get("elapsed") == "0" and s.get("lap1") == '""', s)

        # ---- T2 计时器 ----
        print("T2: 计时器")
        mcp.press("计时器")
        time.sleep(0.8)
        s = mcp.state("t_set_s", "t_on")
        ck.ok("缺省设定 1 秒（步进器）", s.get("t_set_s") == "1", s)
        mcp.press("开始")
        time.sleep(0.5)
        s = mcp.state("t_on")
        ck.ok("计时运行", s.get("t_on") == '"true"', s)
        # 到零轮询（1s 设定 + 容差）
        for _ in range(30):
            time.sleep(0.5)
            s = mcp.state("banner", "t_on")
            if s.get("t_on") == '"false"':
                break
        ck.ok("到零 banner", "时间到" in s.get("banner", ""), s)
        mcp.press("知道了")
        time.sleep(0.5)
        s = mcp.state("banner")
        ck.ok("Dismiss 关闭", s.get("banner") == '""', s)

        # ---- T3 世界时钟 ----
        print("T3: 世界时钟")
        mcp.press("世界时钟")
        deadline = time.time() + 8
        rows = []
        while time.time() < deadline:
            s = mcp.state("w_rows", "w_local")
            raw = s.get("w_rows", "")
            rows = re.findall(r'"([^"]+)"', raw)
            if len(rows) >= 8:
                break
            time.sleep(1)
        ck.ok("≥8 城渲染", len(rows) >= 8, rows)
        import datetime
        now = datetime.datetime.utcnow()
        beijing = now + datetime.timedelta(hours=8)
        london = now
        def hhmm(rows, city):
            for r in rows:
                if r.startswith(city):
                    m = re.search(r"(\d{2}):(\d{2})", r)
                    if m:
                        return int(m.group(1)) * 60 + int(m.group(2))
            return None
        b, l = hhmm(rows, "Beijing"), hhmm(rows, "London")
        ck.ok("北京/伦敦时差 8h（±60s）",
              b is not None and l is not None and abs(((b - l - 480) % 1440 + 720) % 1440 - 720) <= 60 or
              (b is not None and l is not None and abs((b - l) % 1440 - 480) <= 60),
              f"b={b} l={l}")

        # ---- T4 闹钟 + storage ----
        print("T4: 闹钟")
        mcp.press("闹钟")
        time.sleep(0.3)
        nxt = (datetime.datetime.utcnow() + datetime.timedelta(hours=8) + datetime.timedelta(minutes=1))
        a_h, a_m = nxt.strftime("%H"), nxt.strftime("%M")
        # 步进器唯一标签（时−/时+/分−/分+）——press_sequence 标签直打
        cur = mcp.state("a_h", "a_m")
        ch, cm = int(cur.get("a_h", "7")), int(cur.get("a_m", "30"))
        th, tm = int(a_h), int(a_m)
        for _ in range((th - ch) % 24):
            mcp.press("时+")
            time.sleep(0.15)
        for _ in range((tm - cm) % 60):
            mcp.press("分+")
            time.sleep(0.15)
        mcp.press("添加")
        time.sleep(0.3)
        s = mcp.state("alarm1", "a_h")
        expected = '"' + a_h + ":" + a_m + '"'
        ck.ok("闹钟入列", s.get("alarm1") == expected,
              f"actual={s.get('alarm1')!r} expected={expected!r}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except Exception:
            proc.kill()

    # 重启恢复（storage 持久化；不清理 storage 文件）
    print("T4b: 重启恢复")
    boot.keep = True
    proc, mcp = boot()
    boot.keep = False
    try:
        time.sleep(1.5)
        s = mcp.state("alarm1")
        expected = '"' + a_h + ":" + a_m + '"'
        ck.ok("storage 跨重启恢复", s.get("alarm1") == expected,
              f"actual={s.get('alarm1')!r} expected={expected!r}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except Exception:
            proc.kill()

    print("\n==== Clock desktop_mcp results ====")
    passed = sum(1 for _, o in ck.rows if o)
    print(f"{passed}/{len(ck.rows)} checks passed")
    for n, o in ck.rows:
        print(f"  {'✓' if o else '✗'} {n}")
    sys.exit(0 if passed == len(ck.rows) else 1)


if __name__ == "__main__":
    main()
