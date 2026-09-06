"""Plan 562: nav→sidebar 迁移双端实跑回归探针（每例复用）。

VM 腿：`auto run -r vm` + MCP —— 启动截图/快照，按文本找可压元素 press，
断言 __current_route 变化（或快照内容变化），截图/快照留证。
Vue 腿：`auto run`（vite dev）+ Playwright —— 首屏截图，按文本 click 后截图。

用法：
  python dual_probe.py --auto-bin <auto.exe> --app-dir <dir> --save-dir <dir> \
      --app 015 --press-text "Library" --expect-route "/settings" [--skip-vue]
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../.agents/skills/autoui-verifier/scripts"))
from test_vm_mcp import AutoUiMcpClient, pick_free_port  # noqa: E402

VERIFIER_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.agents/skills/autoui-verifier/scripts"))


def vm_leg(args) -> bool:
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] VM 启动: {args.auto_bin} run -r vm @ {args.app_dir} (mcp:{port})")
    os.makedirs(args.save_dir, exist_ok=True)
    vm_log_path = os.path.join(args.save_dir, f"vm_server_{args.app}.log")
    vm_log = open(vm_log_path, "w", encoding="utf-8", errors="replace")
    proc = subprocess.Popen([args.auto_bin, "run", "-r", "vm"], cwd=args.app_dir, env=env,
                            stdout=vm_log, stderr=subprocess.STDOUT)
    try:
        client = AutoUiMcpClient(port)
        t0 = time.time()
        ready = False
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
            print("[-] VM MCP 就绪超时", file=sys.stderr)
            return False

        home_png = os.path.join(args.save_dir, f"p562_{args.app}_vm_home.png")
        print(client.screenshot(f"p562_{args.app}_vm_home", save_path=home_png))
        snap_home = client.snapshot()
        open(os.path.join(args.save_dir, f"snapshot_{args.app}_vm_home.txt"), "w", encoding="utf-8").write(snap_home)

        # 按文本定位可压元素：text 子节点不可 press，需回溯到祖先 button 行
        nav_id = None
        snap_lines = snap_home.splitlines()
        for idx, line in enumerate(snap_lines):
            if args.press_text.lower() not in line.lower() or "#" not in line:
                continue
            indent = len(line) - len(line.lstrip())
            cand = line if line.lstrip().startswith("button") else None
            if cand is None:
                for j in range(idx - 1, -1, -1):
                    up = snap_lines[j]
                    up_indent = len(up) - len(up.lstrip())
                    if up_indent < indent and up.lstrip().startswith("button"):
                        cand = up
                        break
                    if up_indent <= indent and up.rstrip().endswith("}") :
                        break  # 跨出了所在子树
            if cand:
                for tok in cand.replace("(", " ").replace(")", " ").split():
                    if tok.startswith("#"):
                        nav_id = tok.lstrip("#")
                        break
            if nav_id:
                break
        checks = {}
        checks["VM 首屏含迁移后菜单文本"] = args.press_text in snap_home
        if nav_id:
            print(f"[*] VM press #{nav_id} ({args.press_text})")
            press_ret = client.press(nav_id)
            print(press_ret)
            checks["VM press 无错误"] = "Error" not in press_ret
            time.sleep(1.0)
            pressed_png = os.path.join(args.save_dir, f"p562_{args.app}_vm_pressed.png")
            print(client.screenshot(f"p562_{args.app}_vm_pressed", save_path=pressed_png))
            snap2 = client.snapshot()
            open(os.path.join(args.save_dir, f"snapshot_{args.app}_vm_pressed.txt"), "w", encoding="utf-8").write(snap2)
            if args.expect_route:
                vm_log.flush()
                try:
                    server_log = open(vm_log_path, encoding="utf-8", errors="replace").read()
                except OSError:
                    server_log = ""
                checks["VM press 后到达期望路由"] = (
                    args.expect_route in snap2
                    or args.expect_route in press_ret
                    or args.expect_route in server_log
                )
            else:
                checks["VM press 后路由/内容变化"] = snap2 != snap_home
        else:
            checks["VM 定位可压元素"] = False

        failed = [k for k, v in checks.items() if not v]
        for k, v in checks.items():
            print(f"[{'+' if v else '-'}] {k}")
        if failed:
            print(f"[-] VM 断言失败 {len(failed)} 项", file=sys.stderr)
            return False
        print("[+] VM 腿全过")
        return True
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()
        vm_log.close()


def vue_leg(args) -> bool:
    port = pick_free_port()
    mcp_port = pick_free_port()
    env = dict(os.environ, PORT=str(port), AUTOUI_MCP_PORT=str(mcp_port))
    print(f"[*] Vue 启动: {args.auto_bin} run @ {args.app_dir}")
    run_cmd = [args.auto_bin, "run"]
    if args.vue_render:
        run_cmd += ["-r", args.vue_render]
    proc = subprocess.Popen(
        run_cmd, cwd=args.app_dir, env=env,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace",
    )
    url = None
    try:
        t0 = time.time()
        while time.time() - t0 < args.timeout:
            line = proc.stdout.readline()
            if not line:
                time.sleep(0.3)
                continue
            print(f"    | {line.rstrip()}")
            if "Local:" not in line:
                continue  # 只认 vite 的 Local: 行，跳过后端 API 端口
            m = re.search(r"http://(?:localhost|127\.0\.0\.1):(\d+)", line)
            if m:
                url = f"http://localhost:{m.group(1)}"
                break
        if not url:
            print("[-] vite 端口探测超时", file=sys.stderr)
            return False
        time.sleep(2.0)  # vite 首编稳定窗

        home_png = os.path.join(args.save_dir, f"p562_{args.app}_vue_home.png")
        pressed_png = os.path.join(args.save_dir, f"p562_{args.app}_vue_pressed.png")
        actions = [
            {"action": "screenshot", "path": home_png},
            {"action": "click", "selector": f"text={args.press_text}"},
            {"action": "wait", "ms": 1200},
            {"action": "screenshot", "path": pressed_png},
        ]
        r = subprocess.run(
            ["node", os.path.join(VERIFIER_DIR, "test_vue_playwright.mjs"), url,
             "--actions", json.dumps(actions)],
            capture_output=True, text=True, timeout=180,
        )
        print(r.stdout[-2000:])
        if r.returncode != 0:
            print(r.stderr[-1500:], file=sys.stderr)
            print("[-] Vue playwright 失败", file=sys.stderr)
            return False
        ok = os.path.exists(home_png) and os.path.exists(pressed_png)
        print(f"[{'+' if ok else '-'}] Vue 双截图落盘")
        return ok
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--auto-bin", required=True)
    ap.add_argument("--app-dir", required=True)
    ap.add_argument("--save-dir", required=True)
    ap.add_argument("--app", required=True)
    ap.add_argument("--press-text", required=True)
    ap.add_argument("--expect-route", default="")
    ap.add_argument("--skip-vue", action="store_true")
    ap.add_argument("--skip-vm", action="store_true")
    ap.add_argument("--timeout", type=int, default=90)
    ap.add_argument("--vue-render", default="", help="显式 -r 渲染目标（如 vue；gallery 等 pac.at 默认非 vue 的项目用）")
    args = ap.parse_args()

    args.app_dir = os.path.abspath(args.app_dir)
    args.save_dir = os.path.abspath(args.save_dir)
    os.makedirs(args.save_dir, exist_ok=True)

    ok = True
    if not args.skip_vm:
        ok = vm_leg(args) and ok
    if not args.skip_vue:
        ok = vue_leg(args) and ok
    return 0 if ok else 2


if __name__ == "__main__":
    sys.exit(main())
