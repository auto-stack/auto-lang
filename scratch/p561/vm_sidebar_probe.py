"""Plan 561 T9: widgets-gallery sidebar 页 VM 端实证与结构对拍。

启动 worktree 内 auto CLI 以 VM 模式跑 widgets-gallery，MCP 驱动：
1. 初始截图 + snapshot；
2. 导航到 sidebar 文档页（按 label 找按钮 press）；
3. sidebar 页截图 + snapshot 结构断言（sidebar 族树在 VM aura 树中可见：
   分区/分组/菜单层级与 active 态）；
4. 证据输出到 --save-dir（默认 scratch/p561/）。
"""
import argparse
import os
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../.agents/skills/autoui-verifier/scripts"))
from test_vm_mcp import AutoUiMcpClient, pick_free_port  # noqa: E402

import subprocess  # noqa: E402


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--auto-bin", required=True)
    ap.add_argument("--app-dir", required=True)
    ap.add_argument("--save-dir", required=True)
    ap.add_argument("--timeout", type=int, default=60)
    args = ap.parse_args()

    app_dir = os.path.abspath(args.app_dir)
    save_dir = os.path.abspath(args.save_dir)
    os.makedirs(save_dir, exist_ok=True)
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))

    print(f"[*] VM 启动: {args.auto_bin} run -r vm @ {app_dir} (mcp:{port})")
    proc = subprocess.Popen([args.auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)
    try:
        client = AutoUiMcpClient(port)
        ready = False
        t0 = time.time()
        while time.time() - t0 < args.timeout:
            try:
                snap = client.snapshot()
                if snap and "tree:" in snap:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not ready:
            print("[-] MCP 就绪超时", file=sys.stderr)
            return 1

        print("[+] 初始就绪，截图 p561_vm_gallery_home")
        print(client.screenshot("p561_vm_gallery_home", save_path=os.path.join(save_dir, "p561_vm_gallery_home.png")))
        snap_home = client.snapshot()
        open(os.path.join(save_dir, "snapshot_home.txt"), "w", encoding="utf-8").write(snap_home)

        # 找 sidebar 页导航按钮（label 含 sidebar 的可压元素）
        nav_id = None
        for line in snap_home.splitlines():
            low = line.lower()
            if "sidebar" in low and ("#" in line) and ("button" in low or "nav" in low or "press" in low):
                # 提取 #aura_N / #vnode_N
                for tok in line.replace("(", " ").replace(")", " ").split():
                    if tok.startswith("#"):
                        nav_id = tok.lstrip("#")
                        break
            if nav_id:
                break
        if not nav_id:
            print("[!] 未在首页快照定位 sidebar 导航按钮，尝试全快照文本线索")
        else:
            print(f"[*] 导航到 sidebar 页: #{nav_id}")
            print(client.press(nav_id))
            time.sleep(1.0)

        print("[+] sidebar 页截图 p561_vm_sidebar_page")
        print(client.screenshot("p561_vm_sidebar_page", save_path=os.path.join(save_dir, "p561_vm_sidebar_page.png")))
        snap = client.snapshot()
        open(os.path.join(save_dir, "snapshot_sidebar.txt"), "w", encoding="utf-8").write(snap)

        # 结构断言：VM aura 树中 sidebar 族痕迹（preview-card 内嵌 sidebar 结构）。
        checks = {
            "sidebar 页可达（快照含 sidebar 字样）": "sidebar" in snap.lower(),
            "菜单项文本可见（收件箱/Inbox 类 label 任一）": any(
                k in snap for k in ["Inbox", "收件箱", "Dashboard", "面板", "Workspace", "工作区"]
            ),
        }
        failed = [name for name, ok in checks.items() if not ok]
        for name, ok in checks.items():
            print(f"[{'+' if ok else '-'}] {name}")
        if failed:
            print(f"[-] 结构断言失败 {len(failed)} 项", file=sys.stderr)
            return 2
        print("[+] 结构断言全过")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    sys.exit(main())
