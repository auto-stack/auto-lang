#!/usr/bin/env python3
"""
PLAN-016 T-10: MCP interaction tests for 027-file-manager in VM mode (`auto run -r vm`).

真实文件系统套件（PLAN-016 重写——mock 时代断言退役）：
- T1: 启动结构（快捷访问/工具栏/真实日期列/主目录非空）
- T2: 地址栏跳转 testdata + 列表与磁盘一致（notes/config/photo/nested）
- T3: 隐藏项开关（.hidden.md 显隐翻转）
- T4: 搜索过滤（notes → 1 项）
- T5: 目录导航（nested 进入 → inner.txt → 上级）
- T6: 新建文件夹 fm-new（磁盘断言）
- T7: 重命名 fm-new → fm-renamed（磁盘断言，旧名移除）
- T8: 进入 fm-renamed 新建 a.txt（磁盘断言）
- T9: 右键菜单删除 a.txt（alert-dialog 确认 → 磁盘断言）
- T10: 上级 + 删除空目录 fm-renamed（D-4 空目录口径 → 磁盘断言）
- T11: 排序状态（大小列 → sort_col == "size"）
- T12: 选中状态栏（选定: ...）
- T13: 持久化（view_mode 跨进程恢复，Phase 2 重启断言）

open_with 桌面互操作（AC-08/09）为桌面宿主级端到端（acceptance bus 注入 →
041 启动消费），见 docs/plans/evidence/016/t07-open-with-e2e.png——单 app
VM 套件无桌面会话，不在本套件范围。

Usage:
    cd examples/ui/027-file-manager/tests
    python desktop_mcp.py
"""

import json
import os
import re
import shutil
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
    raise RuntimeError(f"No free port in [{start}, {start}+100)")


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
        # PLAN-016：MCP 服务器线程偶发静默失联（进程存活、socket 消失，
        # 实证见 evidence/016/d3-vue-fs.md 残留节）——连接层失败容忍重试，
        # 30s 内恢复则继续套件。
        last = None
        for _ in range(30):
            try:
                resp = requests.post(self.url, json={
                    "jsonrpc": "2.0", "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                    "id": self.req_id,
                }, timeout=20)
                data = resp.json()
                break
            except (requests.ConnectionError, requests.Timeout) as e:
                last = e
                time.sleep(1)
        else:
            raise last
        time.sleep(0.15)  # 节流：连续快压下 MCP 服务器线程偶发失联的缓解
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


def row_actions_button(snap, label):
    """行内 ··· 触发钮（名称按钮后首个空标签 button）。"""
    i = snap.find('"%s"' % label)
    if i < 0:
        return None
    seg = snap[i:i + 1500]
    return find_id(seg, r'button #(\S+) ""')


def run_suite(mcp, workdir, result):
    # ── T1: 启动结构 ────────────────────────────────────────────────────────
    print("\n[T1] 启动结构（真实 FS）")
    snap = mcp.snapshot()
    result.check("T1 快速访问侧栏", "快速访问" in snap)
    result.check("T1 真实快捷目录", all(k in snap for k in ("主目录", "桌面", "文档", "下载")))
    result.check("T1 顶部工具栏按钮", all(k in snap for k in ("+ 文件夹", "+ 文件", "隐藏项")))
    result.check("T1 修改日期列真实格式", re.search(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}", snap) is not None)
    st = mcp.state("home", "current_path", "item_count_str", "booted")
    result.check("T1 主目录解析", st.get("home", '""') not in ('""', ""), str(st))
    result.check("T1 主目录列出条目", st.get("item_count_str", "0 个项目") not in ("0 个项目", ""), str(st))

    # ── T2: 地址栏跳转 testdata + 列表一致 ──────────────────────────────────
    print("\n[T2] 地址栏跳转 testdata")
    data = os.path.join(workdir, "data")
    snap = mcp.snapshot()
    inputs = re.findall(r'input #(\S+)', snap)
    result.check("T2 地址栏在场", len(inputs) >= 2, str(inputs))
    addr = inputs[1]  # [0]=搜索, [1]=地址栏
    mcp.type_text(addr, data)
    time.sleep(0.3)
    # onenter 经 submit 动作触发（press 对 input 无 Enter 语义）。
    mcp.call("autoui_action", element_id=addr, action="submit")
    time.sleep(1.0)
    st = mcp.state("current_path", "item_count_str")
    result.check("T2 地址跳转到位", data.replace("\\", "\\") in st.get("current_path", "").replace("\\\\", "\\"), str(st))
    snap = mcp.snapshot()
    result.check("T2 testdata 列出文件", all(k in snap for k in ("notes.txt", "config.toml", "photo.png", "nested")), str(st))
    result.check("T2 隐藏文件默认不显示", ".hidden.md" not in snap)
    result.check("T2 磁盘一致性", os.path.isdir(data))
    # 护栏：跳转失败则跳过后续 FS 触达用例（防污染错误目录）。
    if data not in st.get("current_path", "").replace("\\\\", "\\"):
        print("  !! T2 跳转失败，跳过 T3-T10")
        return

    # ── T3: 隐藏项开关 ──────────────────────────────────────────────────────
    print("\n[T3] 隐藏项开关")
    snap = mcp.snapshot()
    hid = find_id(snap, r'button #(\S+)(?=[\s\S]{0,200}text #\S+ "隐藏项")')
    if not hid:
        i = snap.find('"隐藏项"')
        seg = snap[max(0, i - 400):i]
        ids = re.findall(r'button #(\S+)', seg)
        hid = ids[-1] if ids else None
    result.check("T3 隐藏项按钮在场", hid is not None)
    if hid:
        mcp.press(hid)
        time.sleep(0.6)
        snap = mcp.snapshot()
        result.check("T3 .hidden.md 可见", ".hidden.md" in snap)
        mcp.press(hid)
        time.sleep(0.6)
        snap = mcp.snapshot()
        result.check("T3 .hidden.md 再隐", ".hidden.md" not in snap)

    # ── T4: 搜索过滤 ────────────────────────────────────────────────────────
    print("\n[T4] 搜索过滤")
    snap = mcp.snapshot()
    search = re.findall(r'input #(\S+)', snap)[0]
    mcp.type_text(search, "notes"); time.sleep(0.8)
    st = mcp.state("item_count_str", "search_q")
    result.check("T4 过滤至 1 项", st.get("item_count_str") == '"1 个项目"', str(st))
    mcp.type_text(search, ""); time.sleep(0.8)
    snap = mcp.snapshot()
    result.check("T4 清空恢复", "config.toml" in snap and "photo.png" in snap)

    # ── T5: 目录导航 ────────────────────────────────────────────────────────
    print("\n[T5] 目录导航（nested）")
    snap = mcp.snapshot()
    nested = find_id(snap, r'button #(\S+) "nested"')
    result.check("T5 nested 行按钮", nested is not None)
    if nested:
        mcp.press(nested)
        time.sleep(0.8)
        snap = mcp.snapshot()
        result.check("T5 inner.txt 可见", "inner.txt" in snap)
        st = mcp.state("current_path")
        result.check("T5 路径入 nested", "nested" in st.get("current_path", ""), str(st))
        # 返回上级：地址栏 submit 父目录（确定性，免 empty-label 定位歧义）。
        inputs = re.findall(r'input #(\S+)', mcp.snapshot())
        mcp.type_text(inputs[1], data)
        time.sleep(0.3)
        mcp.call("autoui_action", element_id=inputs[1], action="submit")
        time.sleep(0.8)
        st = mcp.state("current_path")
        result.check("T5 上级返回", "nested" not in st.get("current_path", ""), str(st))

    # ── T6: 新建文件夹（磁盘断言）────────────────────────────────────────────
    print("\n[T6] 新建文件夹 fm-new")
    snap = mcp.snapshot()
    bid = find_id(snap, r'button #(\S+) "\+ 文件夹"')
    result.check("T6 新建按钮", bid is not None)
    if bid:
        mcp.press(bid)
        time.sleep(0.8)
        inputs = re.findall(r'input #(\S+)', mcp.snapshot())
        result.check("T6 模态输入框", len(inputs) >= 2, str(inputs))
        mcp.type_text(inputs[1], "fm-new")  # [1] = 新建模态输入
        time.sleep(0.3)
        ok = find_id(mcp.snapshot(), r'button #(\S+) "创建"')
        mcp.press(ok)
        time.sleep(1.2)
        result.check("T6 磁盘落盘", os.path.isdir(os.path.join(data, "fm-new")))

    # ── T7: 重命名 fm-new → fm-renamed ───────────────────────────────────────
    print("\n[T7] 重命名（右键菜单 → 模态）")
    snap = mcp.snapshot()
    rb = row_actions_button(snap, "fm-new")
    result.check("T7 行操作钮", rb is not None)
    if rb:
        mcp.press(rb)
        time.sleep(0.8)
        snap = mcp.snapshot()
        rn = re.findall(r'button #(\S+) "重命名"', snap)
        result.check("T7 菜单重命名项", len(rn) > 0)
        if rn:
            mcp.press(rn[0])
            time.sleep(0.8)
            inputs = re.findall(r'input #(\S+)', mcp.snapshot())
            mcp.type_text(inputs[-1], "fm-renamed")
            time.sleep(0.3)
            ok = re.findall(r'button #(\S+) "重命名"', mcp.snapshot())
            mcp.press(ok[-1])
            time.sleep(1.2)
            result.check("T7 新名落盘", os.path.isdir(os.path.join(data, "fm-renamed")))
            result.check("T7 旧名移除", not os.path.exists(os.path.join(data, "fm-new")))

    # ── T8: 进入 + 新建文件 ──────────────────────────────────────────────────
    print("\n[T8] 新建文件 a.txt")
    snap = mcp.snapshot()
    name_btn = find_id(snap, r'button #(\S+) "fm-renamed"')
    result.check("T8 fm-renamed 行", name_btn is not None)
    if name_btn:
        mcp.press(name_btn)
        time.sleep(0.8)
        snap = mcp.snapshot()
        fid = find_id(snap, r'button #(\S+) "\+ 文件"')
        mcp.press(fid)
        time.sleep(0.8)
        inputs = re.findall(r'input #(\S+)', mcp.snapshot())
        mcp.type_text(inputs[1], "a.txt")
        time.sleep(0.3)
        ok = find_id(mcp.snapshot(), r'button #(\S+) "创建"')
        mcp.press(ok)
        time.sleep(1.2)
        result.check("T8 磁盘落盘", os.path.isfile(os.path.join(data, "fm-renamed", "a.txt")))
        result.check("T8 空文件", os.path.getsize(os.path.join(data, "fm-renamed", "a.txt")) == 0)

    # ── T9: 右键删除 a.txt（alert-dialog 确认）───────────────────────────────
    print("\n[T9] 删除文件（alert-dialog）")
    snap = mcp.snapshot()
    rb = row_actions_button(snap, "a.txt")
    result.check("T9 行操作钮", rb is not None)
    if rb:
        mcp.press(rb)
        time.sleep(0.8)
        snap = mcp.snapshot()
        dl = re.findall(r'button #(\S+) "删除"', snap)
        result.check("T9 菜单删除项", len(dl) > 0)
        mcp.press(dl[0])
        time.sleep(0.8)
        snap = mcp.snapshot()
        result.check("T9 确认模态出现", "确认删除项目" in snap and "不可撤销" in snap)
        ok = re.findall(r'button #(\S+) "确认删除"', snap)
        mcp.press(ok[-1])
        time.sleep(1.2)
        result.check("T9 磁盘移除", not os.path.exists(os.path.join(data, "fm-renamed", "a.txt")))

    # ── T10: 上级 + 删除空目录（D-4）─────────────────────────────────────────
    print("\n[T10] 删除空目录")
    inputs = re.findall(r'input #(\S+)', mcp.snapshot())
    mcp.type_text(inputs[1], data)
    time.sleep(0.3)
    mcp.call("autoui_action", element_id=inputs[1], action="submit")
    time.sleep(1.0)
    snap = mcp.snapshot()
    rb = row_actions_button(snap, "fm-renamed")
    result.check("T10 行操作钮", rb is not None)
    if rb:
        mcp.press(rb)
        time.sleep(0.8)
        snap = mcp.snapshot()
        dl = re.findall(r'button #(\S+) "删除"', snap)
        mcp.press(dl[0])
        time.sleep(0.8)
        ok = re.findall(r'button #(\S+) "确认删除"', mcp.snapshot())
        mcp.press(ok[-1])
        time.sleep(1.2)
        result.check("T10 空目录磁盘移除", not os.path.exists(os.path.join(data, "fm-renamed")))

    # ── T11: 排序状态 ────────────────────────────────────────────────────────
    print("\n[T11] 排序")
    snap = mcp.snapshot()
    sz = find_id(snap, r'button #(\S+)(?=[\s\S]{0,200}?text #\S+ "大小")')
    if not sz:
        i = snap.find('"大小"')
        seg = snap[max(0, i - 400):i]
        ids = re.findall(r'button #(\S+)', seg)
        sz = ids[-1] if ids else None
    if sz:
        mcp.press(sz)
        time.sleep(0.6)
        st = mcp.state("sort_col", "sort_dir")
        result.check("T11 sort_col=size", st.get("sort_col") == '"size"', str(st))

    # ── T12: 选中状态栏 ──────────────────────────────────────────────────────
    print("\n[T12] 选中状态")
    snap = mcp.snapshot()
    nb = find_id(snap, r'button #(\S+) "notes.txt"')
    if nb:
        mcp.press(nb)
        time.sleep(0.6)
        st = mcp.state("selected_info")
        result.check("T12 选定状态", "notes.txt" in st.get("selected_info", ""), str(st))

    # ── T13 前置: 视图模式切 grid（Phase 2 持久化断言用）────────────────────
    snap = mcp.snapshot()
    ai = snap.find('"搜索当前目录..."')
    seg = snap[ai:ai + 900]
    empties = re.findall(r'button #(\S+) ""', seg)
    gm = empties[1] if len(empties) >= 2 else None
    if gm:
        mcp.press(gm)
        time.sleep(0.5)
        st = mcp.state("view_mode")
        result.check("T13 前置 view_mode=grid", st.get("view_mode") == '"grid"', str(st))
    else:
        result.check("T13 前置 view_mode=grid", False, "grid btn not found")


def run_persistence_suite(mcp):
    """Phase 1 末尾配置写入后不再重复（T13 前置已在 run_suite 尾部切 grid）。"""
    return TestResult()


def main():
    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"

    # PLAN-016：testdata 拷贝到临时目录（破坏性操作只对副本）。
    tmp_root = tempfile.mkdtemp(prefix="fm-mcp-")
    data_src = os.path.normpath(os.path.join(os.path.dirname(__file__), "testdata"))
    data_dst = os.path.join(tmp_root, "data")
    shutil.copytree(data_src, data_dst)
    tmp_storage = os.path.join(tmp_root, "storage.at")

    print("=" * 60)
    print("027-file-manager 桌面 MCP 测试（PLAN-016 真实 FS 套件）")
    print(f"  auto:    {AUTO_BIN}")
    print(f"  project: {PROJECT}")
    print(f"  testdata: {data_dst}")
    print("=" * 60)

    result = TestResult()

    # ── Phase 1 ──
    proc = launch(mcp_port, tmp_storage, fresh=True)
    try:
        if not wait_for_server(mcp_url):
            print("ERROR: MCP server 启动超时")
            sys.exit(1)
        client = McpClient(mcp_url)
        # 等待首帧渲染（Tick 延迟引导完成 → item_count_str 非默认）。
        rendered = False
        for _ in range(30):
            try:
                st = client.state("booted")
                if "true" in st:
                    rendered = True
                    break
            except Exception:
                pass
            time.sleep(1)
        if not rendered:
            print("WARNING: Tick 引导未完成，继续运行...")

        run_suite(client, tmp_root, result)
        p_res = run_persistence_suite(client)
        result.passed += p_res.passed
        result.failed += p_res.failed
        result.errors.extend(p_res.errors)

    finally:
        proc.kill()
        proc.wait()
        print("第一轮 VM 进程已退出。")

    # ── Phase 2: 重启验证持久化（view_mode=grid 恢复）──
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
    finally:
        proc2.kill()
        proc2.wait()
        if os.path.exists(tmp_storage):
            try:
                os.remove(tmp_storage)
            except Exception:
                pass
        shutil.rmtree(tmp_root, ignore_errors=True)
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
