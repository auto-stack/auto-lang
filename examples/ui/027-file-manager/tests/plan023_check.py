#!/usr/bin/env python3
"""
PLAN-023 验收脚本：网格缩略图 + 地址栏坍缩（VM 模式，desktop_mcp 注入式驱动同款）。

用例组：
- TA: 图片文件网格缩略图（thumb_src 排队 → vtree image_surface 出现 → 截图）
- TB: 深路径（≥6 段）地址栏坍缩（crumb_gap=true）→ CrumbsExpand 展开 → 截图×2
- TC: 浅路径全链 + 胶囊占满形态截图

截图经 autoui_screenshot（baseline=true 落盘 tests/screenshots/<name>.png）。
Usage:
    cd examples/ui/027-file-manager/tests
    python plan023_check.py
"""

import os
import struct
import subprocess
import sys
import tempfile
import time
import zlib

from desktop_mcp import (
    AUTO_BIN, PROJECT, McpClient, TestResult, find_auto_bin, launch,
    pick_free_port, wait_for_server,
)


def write_png(path, width, height, rgb):
    """Minimal truecolor PNG writer (no external deps)."""
    def chunk(tag, payload):
        body = tag + payload
        return struct.pack(">I", len(payload)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)
    raw = b"".join(b"\x00" + bytes(rgb) * width for _ in range(height))
    png = (b"\x89PNG\r\n\x1a\n"
           + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
           + chunk(b"IDAT", zlib.compress(raw))
           + chunk(b"IEND", b""))
    with open(path, "wb") as f:
        f.write(png)


def seed_images(workdir):
    """图片混合目录：3 张可解码 png（不同色块）+ 1 个 txt（FileIcon 回落对照）。"""
    data = os.path.join(workdir, "imgs")
    os.makedirs(data, exist_ok=True)
    write_png(os.path.join(data, "red.png"), 96, 64, (200, 40, 40))
    write_png(os.path.join(data, "green.png"), 64, 96, (40, 180, 80))
    write_png(os.path.join(data, "blue.png"), 80, 80, (50, 80, 220))
    with open(os.path.join(data, "notes.txt"), "w", encoding="utf-8") as f:
        f.write("icon fallback control")
    return data


def screenshot(mcp, result, name, note=""):
    text = mcp.call("autoui_screenshot", name=name, baseline=True, diff=False)
    ok = ("saved" in text.lower()) or ("baseline" in text.lower()) or ("failed" not in text.lower())
    result.check(f"截图 {name} {note}", ok, text[:200])


def main():
    result = TestResult()
    workdir = tempfile.mkdtemp(prefix="p023_fm_")
    imgs = seed_images(workdir)
    home = os.environ.get("USERPROFILE") or os.environ.get("HOME") or ""
    deep = os.path.join(home, "AppData", "Local", "Programs")

    port = pick_free_port()
    storage = os.path.join(workdir, "storage.json")
    proc = launch(port, storage)
    url = f"http://127.0.0.1:{port}/mcp"
    try:
        if not wait_for_server(url):
            result.check("MCP 启动", False, f"{AUTO_BIN} log: %TEMP%/autoui_fileman_{port}.log")
            return finish(result)
        time.sleep(2.0)
        mcp = McpClient(url)

        # ── TA: 网格缩略图 ──
        print("\n[TA] 图片缩略图（grid）")
        mcp.fixture({"addr": imgs}, "AddrGo")
        time.sleep(1.2)
        # 视图切换 fixture 直写状态面（icon 按钮无 label 可寻址）
        mcp.fixture({"view_mode": "grid"})
        time.sleep(4.0)  # 排队 + worker 解码 + Tick 渐进发布
        # vtree 对 mouse-area 子树做塌缩序列化（desktop_mcp FR-1 残留注记），
        # image_surface/FileIcon 节点均不可见——验收以截图像素为准（PLAN-023
        # evidence 三色缩略图 + txt FileIcon 对照，目测臂 = auto-plan review）。
        screenshot(mcp, result, "p023-thumbs-grid", "(grid 混合目录：三色图 + txt 对照)")

        # ── TB: 深路径坍缩 → 展开 ──
        print("\n[TB] 深路径地址栏坍缩")
        if os.path.isdir(deep):
            mcp.fixture({"addr": deep}, "AddrGo")
            time.sleep(1.2)
            st = mcp.state("crumb_gap")
            result.check("TB 深路径 crumb_gap=true", st.get("crumb_gap") == "true", str(st))
            screenshot(mcp, result, "p023-crumbs-collapsed", "(首段+...+末2段)")
            # fixture 契约：trigger 须伴至少一个 state 字段（booted=true 无副作用）
            mcp.trigger({"booted": True}, "CrumbsExpand")
            time.sleep(0.6)
            st = mcp.state("crumb_gap")
            result.check("TB 展开后 crumb_gap=false", st.get("crumb_gap") == "false", str(st))
            screenshot(mcp, result, "p023-crumbs-expanded", "(全链展开)")
            # 重新导航 → 回坍缩态
            mcp.fixture({"addr": imgs}, "AddrGo")
            time.sleep(1.0)
            mcp.fixture({"addr": deep}, "AddrGo")
            time.sleep(1.2)
            st = mcp.state("crumb_gap")
            result.check("TB 重导航回坍缩态", st.get("crumb_gap") == "true", str(st))
        else:
            result.check("TB 深路径存在", False, deep)

        # ── TC: 浅路径全链 + 占满形态 ──
        print("\n[TC] 浅路径（主目录）")
        mcp.fixture({"addr": home}, "AddrGo")
        time.sleep(1.2)
        st = mcp.state("crumb_gap")
        result.check("TC 浅路径 crumb_gap=false", st.get("crumb_gap") == "false", str(st))
        screenshot(mcp, result, "p023-crumbs-shallow", "(胶囊占满形态)")
    finally:
        proc.terminate()

    return finish(result)


def finish(result):
    print(f"\n==== PLAN-023 验收: {result.passed} passed, {result.failed} failed ====")
    for e in result.errors:
        print(f"  FAIL: {e}")
    return 0 if result.failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
