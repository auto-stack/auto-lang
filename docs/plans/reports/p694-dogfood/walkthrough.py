"""PLAN-694 T-03 实机走查：桌面 launcher 拉起 a2r exe（desktop 模式）全环录证。

前置：桌面宿主已起（auto run -r vm --desktop，AUTOUI_MCP_PORT=9412），
003-converter / 001-helloworld exe 已编译（<repo>/target/debug/）。
证据落 docs/plans/reports/p694-dogfood/（截图 01-07）。

AC 覆盖：AC-01（拉起+交互闭环）/ AC-03（死亡回收+可再启动）/ AC-04（第二 exe）。
"""
import json
import subprocess
import sys
import time

sys.path.insert(0, r"D:\autostack\.wt\lang-694\auto-lang\.agents\skills\autoui-verifier\scripts")
from test_vm_mcp import AutoUiMcpClient  # noqa: E402

SHOT_DIR = r"D:\autostack\.wt\lang-694\auto-lang\docs\plans\reports\p694-dogfood"


def shot(client, name):
    path = rf"{SHOT_DIR}\{name}.png"
    res = client.screenshot(name, baseline=False, save_path=path)
    print(f"[shot] {name}: {str(res)[:120]}")


def find_entry(snapshot_text, needle):
    """从 snapshot 文本抓含 needle 的元素 id 行。"""
    for line in snapshot_text.splitlines():
        if needle.lower() in line.lower():
            return line.strip()
    return None


def main():
    client = AutoUiMcpClient(9412)
    # ① 桌面全景。
    snap = client.snapshot()
    print("=== desktop snapshot head ===")
    print("\n".join(snap.splitlines()[:25]))
    shot(client, "01-desktop")

    # ② Ctrl+Space 召唤 launcher。
    print("[*] Ctrl+Space summon launcher")
    print("[kb]", client.keyboard("space", ["ctrl"]))
    time.sleep(1.5)
    snap = client.snapshot()
    print("=== launcher snapshot head ===")
    print("\n".join(snap.splitlines()[:40]))
    shot(client, "02-launcher")

    # ③ 点 converter 图标（launcher 注册表项）。
    line = find_entry(snap, "converter")
    print("[*] converter entry:", line)
    if line:
        eid = line.split()[0].lstrip("#").rstrip(":").strip()
        print("[press]", client.press(eid))
        time.sleep(3)
    shot(client, "03-converter-window")
    snap = client.snapshot()
    has_converter = "Temperature Converter" in snap
    print(f"[check] converter 窗进桌面: {has_converter}")

    # ④ 交互闭环：聚焦 celsius 输入 → 键入 100 → 212 联动。
    # （键入走全局键盘：点击输入位 + CharTyped 经焦点窗路由。）
    print("[*] typing phase")
    print("[kb]", client.keyboard("1", []))
    print("[kb]", client.keyboard("0", []))
    print("[kb]", client.keyboard("0", []))
    time.sleep(1.5)
    snap = client.snapshot()
    has_212 = "212" in snap
    print(f"[check] 100°C → 212°F 联动: {has_212}")
    shot(client, "04-converter-typed")

    # ⑤ AC-03 死亡回收：taskkill exe → 窗清。
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq converter.exe", "/FO", "CSV"],
        capture_output=True, text=True).stdout
    print("[tasklist]", out.strip().splitlines()[-1] if out.strip() else "empty")
    for row in out.strip().splitlines()[1:]:
        pid = row.split('","')[1]
        print("[kill] converter.exe pid=", pid)
        subprocess.run(["taskkill", "/PID", pid, "/F"], capture_output=True)
    time.sleep(2.5)
    shot(client, "05-after-kill")
    snap = client.snapshot()
    gone = "Temperature Converter" not in snap
    print(f"[check] kill 后窗已清: {gone}")

    # ⑥ AC-03 可再启动：launcher 再点 converter。
    print("[*] relaunch via launcher")
    print("[kb]", client.keyboard("space", ["ctrl"]))
    time.sleep(1.5)
    snap = client.snapshot()
    line = find_entry(snap, "converter")
    if line:
        eid = line.split()[0].lstrip("#").rstrip(":").strip()
        print("[press]", client.press(eid))
        time.sleep(3)
    shot(client, "06-relaunched")
    snap = client.snapshot()
    print(f"[check] 再启动窗落地: {'Temperature Converter' in snap}")

    # ⑦ AC-04 第二 exe：hello-world。
    print("[*] launch hello-world via launcher")
    print("[kb]", client.keyboard("space", ["ctrl"]))
    time.sleep(1.5)
    snap = client.snapshot()
    line = find_entry(snap, "hello")
    print("[*] hello entry:", line)
    if line:
        eid = line.split()[0].lstrip("#").rstrip(":").strip()
        print("[press]", client.press(eid))
        time.sleep(3)
    shot(client, "07-helloworld")
    snap = client.snapshot()
    has_hello = "Hello" in snap
    print(f"[check] hello-world 窗进桌面: {has_hello}")
    print("[summary]", json.dumps({
        "converter_window": has_converter,
        "typing_212": has_212,
        "kill_reclaimed": gone,
        "hello_world": has_hello,
    }))


if __name__ == "__main__":
    main()
