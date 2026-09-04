"""
PLAN-536 T5:musk VM 实机联动回归驱动。
- 用 worktree(auto-lang plan-536-dev)的 auto.exe 起 musk VM UI(split 模式,后端 9247)
- stderr/stdout tee 到日志(Init 计数证据)
- MCP:登录态探测 → 选会话 → 发送 → 观察画布是否免重选直显回复
"""
import json
import os
import socket
import subprocess
import sys
import time
import urllib.request

AUTO_BIN = r"D:/autostack/.wt/lang-536/auto-lang/target/debug/auto.exe"
MUSK_ROOT = r"D:/autostack/auto-musk"
LOG = r"D:/autostack/.wt/lang-536/auto-lang/scratch/p536_t5_musk_vm.log"


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Client:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._req_id = 1

    def call(self, tool: str, args: dict = None) -> dict:
        req = {"jsonrpc": "2.0", "id": self._req_id,
               "method": "tools/call",
               "params": {"name": tool, "arguments": args or {}}}
        self._req_id += 1
        r = urllib.request.Request(self.url, data=json.dumps(req).encode(),
                                   headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(r, timeout=15) as resp:
            res = json.loads(resp.read().decode())
            if "error" in res:
                raise RuntimeError(f"{tool}: {res['error']}")
            return res.get("result", {})

    def text(self, tool: str, args: dict = None) -> str:
        res = self.call(tool, args)
        return res.get("content", [{}])[0].get("text", "")


def main():
    os.makedirs(os.path.dirname(LOG), exist_ok=True)
    port = pick_free_port()
    env = dict(os.environ,
               AUTOUI_MCP_PORT=str(port),
               AUTO_BACKEND="http://127.0.0.1:9247",
               AUTO_VM_MERGE="0",
               RUST_MIN_STACK="16777216")
    print(f"[*] launching musk VM UI (mcp:{port}) with {AUTO_BIN}")
    with open(LOG, "ab") as lf:
        lf.write(f"\n===== p536 T5 musk run {time.strftime('%F %T')} =====\n".encode())
        proc = subprocess.Popen([AUTO_BIN, "run", "--render=vm", "--no-merge"], cwd=MUSK_ROOT, env=env,
                                stdout=lf, stderr=lf)
        c = Client(port)
        t0 = time.time()
        ready = False
        while time.time() - t0 < 45:
            if proc.poll() is not None:
                print(f"[-] process exited early code={proc.returncode}")
                sys.exit(4)
            try:
                snap = c.text("autoui_snapshot")
                if snap and "tree:" in snap:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not ready:
            print("[-] VM UI not ready in 45s"); sys.exit(1)
        print("[+] UI ready")
        snap = c.text("autoui_snapshot")
        # 打印快照前 3000 字供登录态判读
        print(snap[:3000])
        print("\n[SNAPSHOT_TAIL_MODE=json]")
        with open(LOG + ".snap1.txt", "w", encoding="utf-8") as f:
            f.write(snap)
        print(f"[*] full snapshot saved. observe window 30s, then exit (proc left running for follow-up probe).")
        # 保存 pid + port 供后续脚本复用
        with open(LOG + ".meta.json", "w") as f:
            json.dump({"pid": proc.pid, "port": port}, f)
        time.sleep(30)


if __name__ == "__main__":
    main()
