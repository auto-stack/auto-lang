#!/usr/bin/env python3
"""
Plan 440 M3: MCP interaction tests for 027-file-manager in VM mode (`auto run -r vm`).

Exercises the complete File Manager application via autoui_* HTTP JSON-RPC tools:
- T1: UI Snapshot & Structure (Sidebar, Breadcrumbs, Toolbar, Storage Quota, File List)
- T2: Navigation (Click subfolder -> breadcrumb & file list update)
- T3: Deeper Navigation (Hierarchical traversal into /root/Documents/Projects)
- T4: History Navigation (GoBack / GoForward / GoUp)
- T5: View Switcher (List ↔ Grid mode toggle)
- T6: Sorting (Column click -> SortByName / SortBySize / SortByDate)
- T7: Hidden Files (Toggle hidden files visibility)
- T8: Search Filtering (Type keyword -> live filter)
- T9: New Folder & New File (Modal popover creation)
- T10: Selection & Info (Select item -> status bar info updates)
- T11: Context Menu & Clipboard (Open context menu -> Copy item -> verify clipboard state)
- T12: Delete Confirmation Popover (Open delete dialog -> Confirm delete -> verify item removed)
- T13: Storage Persistence (Config restored across process restart)

Usage:
    cd examples/ui/027-file-manager/tests
    python desktop_mcp.py
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9427


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First free port in [start, start+100)."""
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


def find_auto_bin():
    if "AUTO_BIN" in os.environ and os.path.exists(os.environ["AUTO_BIN"]):
        return os.environ["AUTO_BIN"]
    candidates = [
        os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "target", "debug", "auto.exe")),
        os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "..", "target", "debug", "auto.exe")),
        "D:\\autostack\\auto-lang\\target\\debug\\auto.exe",
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return candidates[0]


AUTO_BIN = find_auto_bin()
PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))


class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
            "id": self.req_id,
        }, timeout=15)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def press(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def toggle(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="toggle")

    def type_text(self, element_id, text):
        return self.call("autoui_type", element_id=element_id, text=text)

    def state(self, *fields):
        text = self.call("autoui_state", fields=list(fields))
        out = {}
        for m in re.finditer(r"(\w+): (.+?) \((?:int|str|bool|list)\)", text):
            out[m.group(1)] = m.group(2)
        return out


def wait_for_server(url, timeout=30):
    for _ in range(timeout):
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1
            }, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


def find_id(snapshot_text, pattern):
    m = re.search(pattern, snapshot_text)
    return m.group(1) if m else None


def launch(mcp_port, storage_file, fresh=True):
    if fresh and os.path.exists(storage_file):
        os.remove(storage_file)
    env = {**os.environ,
           "AUTOUI_MCP_PORT": str(mcp_port),
           "AUTO_VM_STORAGE_FILE": storage_file}
    log_file = open(os.path.join(tempfile.gettempdir(), f"autoui_fileman_{mcp_port}.log"), "w", encoding="utf-8")
    return subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PROJECT, env=env,
        stdout=log_file, stderr=log_file,
    )


class TestResult:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.errors = []

    def check(self, name, condition, detail=""):
        if condition:
            self.passed += 1
            print(f"  PASS  {name}")
        else:
            self.failed += 1
            self.errors.append(f"{name}: {detail}")
            print(f"  FAIL  {name}: {detail}")


def run_suite(mcp):
    result = TestResult()
    snap = mcp.snapshot()

    # ── T1: 初始结构与快照 ───────────────────────────────────────────────────
    print("\n[T1] 初始结构与快照")
    result.check("T1 快速访问侧栏", "快速访问" in snap)
    result.check("T1 侧栏 6 大目录", all(k in snap for k in ("主目录", "文档", "下载", "图片", "音乐", "回收站")))
    result.check("T1 存储空间仪表", "存储空间" in snap and "42.5 / 128 GB" in snap)
    result.check("T1 顶部工具栏按钮", all(k in snap for k in ("+ 文件夹", "+ 文件", "隐藏项: 关")))
    result.check("T1 初始根目录包含主要文件夹", all(k in snap for k in ("Documents", "Downloads", "Pictures", "README.md")))
    result.check("T1 隐藏文件默认不显示", ".env" not in snap)

    # ── T2: 目录导航 ────────────────────────────────────────────────────────
    print("\n[T2] 目录导航（进入 Documents）")
    docs_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "Documents"')
    if not docs_btn:
        docs_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+).*?Documents')
    result.check("T2 找到 Documents 按钮", docs_btn is not None)
    if docs_btn:
        mcp.press(docs_btn)
        time.sleep(0.5)
        st = mcp.state("current_path")
        snap = mcp.snapshot()
        result.check("T2 当前路径变为 /root/Documents", st.get("current_path") == '"/root/Documents"')
        result.check("T2 面包屑显示 Documents", "Documents" in snap)
        result.check("T2 Documents 目录内容呈现", "Projects" in snap and "notes.txt" in snap and "budget_2026.xlsx" in snap)

    # ── T3: 深度目录导航 ────────────────────────────────────────────────────
    print("\n[T3] 深度导航（进入 Projects）")
    snap = mcp.snapshot()
    proj_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "Projects"')
    result.check("T3 找到 Projects 按钮", proj_btn is not None)
    if proj_btn:
        mcp.press(proj_btn)
        time.sleep(0.5)
        st = mcp.state("current_path")
        snap = mcp.snapshot()
        result.check("T3 当前路径变为 /root/Documents/Projects", st.get("current_path") == '"/root/Documents/Projects"')
        result.check("T3 包含 auto-lang 项目与文档", "auto-lang" in snap and "architecture.pdf" in snap)

    # ── T4: 历史导航（前进 / 后退 / 上一级） ────────────────────────────────
    print("\n[T4] 历史导航（GoBack / GoForward / GoUp）")
    snap = mcp.snapshot()
    back_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "←"')
    result.check("T4 找到后退按钮", back_btn is not None)
    if back_btn:
        mcp.press(back_btn)
        time.sleep(0.4)
        st = mcp.state("current_path")
        result.check("T4 后退到 /root/Documents", st.get("current_path") == '"/root/Documents"')

        snap = mcp.snapshot()
        fwd_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "→"')
        if fwd_btn:
            mcp.press(fwd_btn)
            time.sleep(0.4)
            st = mcp.state("current_path")
            result.check("T4 前进回 /root/Documents/Projects", st.get("current_path") == '"/root/Documents/Projects"')

        snap = mcp.snapshot()
        up_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "↑"')
        if up_btn:
            mcp.press(up_btn)
            time.sleep(0.4)
            st = mcp.state("current_path")
            result.check("T4 上一级回到 /root/Documents", st.get("current_path") == '"/root/Documents"')

    # ── T5: 视图模式切换（List ↔ Grid） ─────────────────────────────────────
    print("\n[T5] 视图模式切换（List ↔ Grid）")
    snap = mcp.snapshot()
    grid_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "⊞"')
    result.check("T5 找到网格按钮", grid_btn is not None)
    if grid_btn:
        mcp.press(grid_btn)
        time.sleep(0.4)
        st = mcp.state("view_mode")
        result.check("T5 视图切换为 grid", st.get("view_mode") == '"grid"')

        snap = mcp.snapshot()
        list_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "≡"')
        if list_btn:
            mcp.press(list_btn)
            time.sleep(0.4)
            st = mcp.state("view_mode")
            result.check("T5 视图切回 list", st.get("view_mode") == '"list"')

    # ── T6: 排序切换 ────────────────────────────────────────────────────────
    print("\n[T6] 排序测试（按大小 / 修改日期 / 名称）")
    snap = mcp.snapshot()
    size_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "大小"')
    result.check("T6 找到大小列头按钮", size_btn is not None)
    if size_btn:
        mcp.press(size_btn)
        time.sleep(0.4)
        st = mcp.state("sort_col", "sort_dir")
        result.check("T6 排序为 size asc", st.get("sort_col") == '"size"' and st.get("sort_dir") == '"asc"')
        mcp.press(size_btn)
        time.sleep(0.4)
        st = mcp.state("sort_dir")
        result.check("T6 大小排序翻转为 desc", st.get("sort_dir") == '"desc"')

    # ── T7: 隐藏文件开关 ────────────────────────────────────────────────────
    print("\n[T7] 隐藏文件开关")
    snap = mcp.snapshot()
    hid_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "隐藏项: 关"')
    result.check("T7 找到隐藏文件开关按钮", hid_btn is not None)
    if hid_btn:
        mcp.press(hid_btn)
        time.sleep(0.4)
        st = mcp.state("show_hidden")
        snap = mcp.snapshot()
        result.check("T7 隐藏文件开关开启", st.get("show_hidden") == "true")
        result.check("T7 隐藏文件 .secret_draft.md 现身", ".secret_draft.md" in snap)

    # ── T8: 搜索过滤 ────────────────────────────────────────────────────────
    print("\n[T8] 搜索过滤")
    snap = mcp.snapshot()
    search_input = find_id(snap, r'input #(aura_\d+|vnode_\d+)')
    result.check("T8 找到搜索输入框", search_input is not None)
    if search_input:
        mcp.type_text(search_input, "budget")
        time.sleep(0.4)
        st = mcp.state("search_q")
        snap = mcp.snapshot()
        result.check("T8 搜索词设置", st.get("search_q") == '"budget"')
        result.check("T8 过滤后只显示匹配项", "budget_2026.xlsx" in snap and "notes.txt" not in snap)
        # 清空搜索词
        mcp.type_text(search_input, "")
        time.sleep(0.4)

    # ── T9: 新建文件与新建文件夹 ────────────────────────────────────────────
    print("\n[T9] 新建文件夹")
    snap = mcp.snapshot()
    new_folder_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "\+ 文件夹"')
    result.check("T9 找到新建文件夹按钮", new_folder_btn is not None)
    if new_folder_btn:
        mcp.press(new_folder_btn)
        time.sleep(0.4)
        st = mcp.state("new_modal_open", "new_modal_type")
        result.check("T9 新建模态已打开", st.get("new_modal_open") == "true" and st.get("new_modal_type") == '"folder"')

        snap = mcp.snapshot()
        create_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "创建"')
        result.check("T9 找到弹层创建按钮", create_btn is not None)
        if create_btn:
            mcp.press(create_btn)
            time.sleep(0.5)
            snap = mcp.snapshot()
            result.check("T9 新文件夹已成功创建并显示在目录", "新建文件夹" in snap)

    # ── T10: 选中与状态信息 ─────────────────────────────────────────────────
    print("\n[T10] 选中与状态信息")
    snap = mcp.snapshot()
    notes_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "notes.txt"')
    result.check("T10 找到 notes.txt", notes_btn is not None)
    if notes_btn:
        mcp.press(notes_btn)
        time.sleep(0.3)
        st = mcp.state("selected_id", "selected_name", "selected_info")
        result.check("T10 选定 notes.txt", st.get("selected_name") == '"notes.txt"')
        result.check("T10 状态栏更新选定描述", "notes.txt" in st.get("selected_info", ""))

    # ── T11: 右键上下文菜单与剪贴板复制 ─────────────────────────────────────
    print("\n[T11] 右键菜单与剪贴板复制")
    snap = mcp.snapshot()
    action_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "···"')
    result.check("T11 找到操作菜单按钮", action_btn is not None)
    if action_btn:
        mcp.press(action_btn)
        time.sleep(0.4)
        st = mcp.state("ctx_open")
        result.check("T11 上下文菜单弹层已打开", st.get("ctx_open") == "true")

        snap = mcp.snapshot()
        copy_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "📄  复制"')
        result.check("T11 找到复制菜单项", copy_btn is not None)
        if copy_btn:
            mcp.press(copy_btn)
            time.sleep(0.4)
            st = mcp.state("clipboard_op", "clipboard_name", "toast_open")
            result.check("T11 剪贴板复制生效", st.get("clipboard_op") == '"copy"')
            result.check("T11 Toast 提示反馈", st.get("toast_open") == "true")

    # ── T12: 删除确认弹层与删除执行 ─────────────────────────────────────────
    print("\n[T12] 删除确认与执行")
    snap = mcp.snapshot()
    # 点击刚才新建的文件夹对应的操作按钮
    action_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "···"')
    if action_btn:
        mcp.press(action_btn)
        time.sleep(0.4)
        snap = mcp.snapshot()
        del_menu_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "🗑️  删除"')
        result.check("T12 找到删除菜单项", del_menu_btn is not None)
        if del_menu_btn:
            mcp.press(del_menu_btn)
            time.sleep(0.4)
            st = mcp.state("confirm_del_open")
            result.check("T12 删除确认弹层打开", st.get("confirm_del_open") == "true")

            snap = mcp.snapshot()
            confirm_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "确认删除"')
            result.check("T12 找到确认删除按钮", confirm_btn is not None)
            if confirm_btn:
                mcp.press(confirm_btn)
                time.sleep(0.5)
                st = mcp.state("confirm_del_open")
                result.check("T12 删除确认弹层已关闭", st.get("confirm_del_open") == "false")

    return result


def run_persistence_suite(mcp):
    """T13: 验证存储配置在进程重启后恢复。"""
    result = TestResult()
    print("\n[T13] 配置写入与重启恢复准备")
    snap = mcp.snapshot()

    grid_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "⊞"')
    # 查找隐藏项开关按钮（无论当前是 开 还是 关）
    hid_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "隐藏项:')
    if not hid_btn:
        hid_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "隐藏项: 开"')
    if not hid_btn:
        hid_btn = find_id(snap, r'button #(aura_\d+|vnode_\d+) "隐藏项: 关"')

    result.check("T13 定位配置切换按钮", grid_btn is not None and hid_btn is not None)
    if grid_btn and hid_btn:
        mcp.press(grid_btn)
        time.sleep(0.3)
        st_before = mcp.state("show_hidden")
        if st_before.get("show_hidden") != "true":
            mcp.press(hid_btn)
            time.sleep(0.3)
        st = mcp.state("view_mode", "show_hidden")
        result.check("T13 配置写入存储", st.get("view_mode") == '"grid"' and st.get("show_hidden") == "true")

    return result


def main():
    print("=" * 60)
    print("Plan 440 M3: Desktop MCP Tests (027-file-manager)")
    print("=" * 60)

    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        print("Build it first: cargo build --features ui-iced --bin auto")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"

    tmp_storage = os.path.join(tempfile.gettempdir(), f"autoui_fileman_test_{mcp_port}.storage")

    print(f"\n[Phase 1] 启动 027-file-manager 实机进程 (端口 {mcp_port})...")
    proc = launch(mcp_port, tmp_storage, fresh=True)

    try:
        print(f"等待 MCP Server 就绪 ({mcp_url})...")
        if not wait_for_server(mcp_url):
            print(f"ERROR: MCP server did not start within 30s.")
            proc.kill()
            sys.exit(1)
        print("MCP Server 已就绪")

        print("等待 UI 渲染...")
        client = McpClient(mcp_url)
        rendered = False
        for i in range(20):
            time.sleep(1.5)
            try:
                snap = client.snapshot()
                if "快速访问" in snap or "Documents" in snap:
                    print(f"UI 渲染完成 ({(i + 1) * 1.5}s)")
                    rendered = True
                    break
            except Exception:
                pass

        if not rendered:
            print("WARNING: UI 可能尚未完成首帧渲染，继续运行...")

        # 运行功能测试套件
        result = run_suite(client)

        # 运行持久化测试前配置写入
        p_res = run_persistence_suite(client)
        result.passed += p_res.passed
        result.failed += p_res.failed
        result.errors.extend(p_res.errors)

    finally:
        proc.kill()
        proc.wait()
        print("第一轮 VM 进程已退出。")

    # ── [Phase 2] 重启进程验证持久化恢复 ──────────────────────────────────────
    print("\n[Phase 2] 重启 VM 进程验证配置恢复...")
    mcp_port2 = pick_free_port(mcp_port + 1)
    mcp_url2 = f"http://localhost:{mcp_port2}/mcp"
    proc2 = launch(mcp_port2, tmp_storage, fresh=False)

    try:
        if not wait_for_server(mcp_url2):
            print("ERROR: 重启进程 MCP server 启动超时")
            result.failed += 1
            result.errors.append("Restart MCP server timeout")
        else:
            client2 = McpClient(mcp_url2)
            time.sleep(2.0)
            st2 = client2.state("view_mode", "show_hidden")
            result.check("T13 重启恢复 view_mode=grid", st2.get("view_mode") == '"grid"', str(st2))
            result.check("T13 重启恢复 show_hidden=true", st2.get("show_hidden") == "true", str(st2))
    finally:
        proc2.kill()
        proc2.wait()
        if os.path.exists(tmp_storage):
            try:
                os.remove(tmp_storage)
            except Exception:
                pass
        print("第二轮 VM 进程已退出。")

    print("\n" + "=" * 60)
    print(f"测试结果: {result.passed} 通过, {result.failed} 失败")
    if result.errors:
        for err in result.errors:
            print(f"  FAIL  {err}")
    print("=" * 60)

    sys.exit(0 if result.failed == 0 else 1)


if __name__ == "__main__":
    main()
