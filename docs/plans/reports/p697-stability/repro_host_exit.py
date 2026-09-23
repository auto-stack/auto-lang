"""PLAN-697 T-00/T-02 复现驱动：桌面宿主静默退出勘定。

自起宿主（auto run -r vm --desktop，AUTOUI_ACCEPTANCE=1 + 专属 MCP 端口），
经 autoui_desktop bus `launch\\t<app>` 拉起目标 App，逐秒盯进程存活——
死亡即记录时刻与退出位面（对照矩阵一轮一臂）。

用法：
  python repro_host_exit.py --app 020-music-player --rounds 5 --watch 90
  python repro_host_exit.py --app 003-calculator --watch 60   # 他 App 对照臂
  python repro_host_exit.py --no-proxy ...                    # 嫌疑A 禁用臂
  python repro_host_exit.py --no-media ...                    # 嫌疑B 禁用臂

宿主/被测均出自 worktree target/debug（构建态与改动位一致）。
输出：每轮一行 JSONL（round/app/env/alive_seconds/died/exit_note）。
"""
import argparse
import json
import os
import socket
import subprocess
import sys
import time
import urllib.request

AUTO_EXE = r"D:\autostack\.wt\lang-697\auto-lang\target\debug\examples\ui_desktop.exe"
WORKSPACE = r"D:\autostack\.wt\lang-697\auto-lang"
APPS_DIR = r"D:\autostack\.wt\lang-697\auto-lang\examples\ui"
HERE = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(HERE, "matrix.jsonl")


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def pid_alive(pid: int) -> bool:
    out = subprocess.run(
        ["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
        capture_output=True, text=True).stdout
    return f'"{pid}"' in out


def mcp_alive(port: int) -> bool:
    try:
        req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list",
                          "params": {}}).encode()
        r = urllib.request.Request(f"http://127.0.0.1:{port}/mcp", data=req,
                                   headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(r, timeout=3) as resp:
            return resp.status == 200
    except Exception:
        return False


def mcp_call(port: int, tool: str, args: dict) -> str:
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                      "params": {"name": tool, "arguments": args}}).encode()
    r = urllib.request.Request(f"http://127.0.0.1:{port}/mcp", data=req,
                               headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(r, timeout=10) as resp:
        res = json.loads(resp.read().decode())
    if "error" in res:
        raise RuntimeError(res["error"])
    return res.get("result", {}).get("content", [{}])[0].get("text", "")


def launch_host(env_extra: dict, fullscreen: bool = False):
    """起宿主，返回 (Popen, mcp_port, log_path, logf)。"""
    port = pick_free_port()
    env = os.environ.copy()
    env.update({
        "AUTOUI_MCP_PORT": str(port),
        "AUTOUI_ACCEPTANCE": "1",
    })
    env.update(env_extra)
    log_path = os.path.join(HERE, f"host-{int(time.time())}.log")
    logf = open(log_path, "w", encoding="utf-8", errors="replace")
    cmd = [AUTO_EXE]
    if fullscreen:
        cmd.append("--fullscreen")
    proc = subprocess.Popen(
        cmd,
        cwd=WORKSPACE, env=env,
        stdout=logf, stderr=subprocess.STDOUT,
        creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
    return proc, port, log_path, logf


def wait_mcp(port: int, timeout_s: float = 60.0) -> bool:
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        if mcp_alive(port):
            return True
        time.sleep(1.0)
    return False


def one_round(round_no: int, app: str, watch_s: int, env_extra: dict,
              fullscreen: bool) -> dict:
    tag = "+".join(f"{k}={v}" for k, v in sorted(env_extra.items())) or "baseline"
    rec = {"round": round_no, "app": app, "env": tag,
           "form": "fullscreen" if fullscreen else "windowed",
           "watch_s": watch_s}
    proc, port, log_path, logf = launch_host(env_extra, fullscreen)
    try:
        if not wait_mcp(port):
            rec.update(died="mcp-never-up", alive_seconds=None,
                       exit_note=f"host log: {log_path}")
            return rec
        t0 = time.time()
        try:
            mcp_call(port, "autoui_desktop", {"action": "bus",
                                              "verb": f"launch\t{app}"})
            rec["launch_queued"] = True
        except Exception as e:
            rec["launch_queued"] = False
            rec["launch_err"] = str(e)[:200]
        # 拉起后逐秒盯存活（进程 + MCP 双探）。
        death = None
        while time.time() - t0 < watch_s:
            time.sleep(1.0)
            if not pid_alive(proc.pid):
                death = "process-gone"
                break
            if not mcp_alive(port):
                # 进程还在但 MCP 已死——也记（半死位面）。
                if not pid_alive(proc.pid):
                    death = "process-gone"
                else:
                    death = "mcp-dead-process-alive"
                break
        rec["alive_seconds"] = round(time.time() - t0, 1)
        rec["died"] = death if death else "no"
        rec["exit_note"] = f"host log: {log_path}" if death else ""
        if death:
            # 回收残进程（防僵尸占屏/占端口）。
            subprocess.run(["taskkill", "/F", "/PID", str(proc.pid)],
                           capture_output=True)
        return rec
    finally:
        # 无论生死，轮末清场。
        if pid_alive(proc.pid):
            subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)],
                           capture_output=True)
        time.sleep(1.0)
        logf.close()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--app", default="020-music-player")
    ap.add_argument("--rounds", type=int, default=5)
    ap.add_argument("--watch", type=int, default=90,
                    help="每轮存活观察窗（秒）；P694-D1 观测死亡位在数十秒内")
    ap.add_argument("--no-proxy", action="store_true",
                    help="嫌疑A 对照：AUTO_NO_BACK_PROXY=1（测试缝）")
    ap.add_argument("--no-media", action="store_true",
                    help="嫌疑B 对照：AUTO_NO_NATIVE_MEDIA=1（测试缝）")
    ap.add_argument("--fullscreen", action="store_true",
                    help="全屏桌面形态（P694-D1 两形态之一；缺省窗口化）")
    args = ap.parse_args()

    env_extra = {}
    if args.no_proxy:
        env_extra["AUTO_NO_BACK_PROXY"] = "1"
    if args.no_media:
        env_extra["AUTO_NO_NATIVE_MEDIA"] = "1"

    for i in range(1, args.rounds + 1):
        rec = one_round(i, args.app, args.watch, env_extra, args.fullscreen)
        line = json.dumps(rec, ensure_ascii=False)
        print(line, flush=True)
        with open(RESULTS, "a", encoding="utf-8") as f:
            f.write(line + "\n")
        if rec.get("died") not in ("no", None):
            # 死亡轮后多歇 2s，避免端口/窗口残态串轮。
            time.sleep(2.0)


if __name__ == "__main__":
    main()
