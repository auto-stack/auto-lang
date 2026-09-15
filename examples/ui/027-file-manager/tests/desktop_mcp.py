#!/usr/bin/env python3
"""
PLAN-016 T-10 (FR-1 重构): MCP interaction tests for 027-file-manager, VM mode.

Phase 2 导航惯例版（2026-09-15，随 PLAN-631 testability 面闭合 AC-11）：

驱动惯例（与 r2 后实际 UI 结构一致——旧套件按 Phase 1 独立地址栏/行名按钮
编写，Phase 2/R4 后失效）：
- 导航 = 侧栏标签按钮（主目录/桌面/文档/下载/图片/音乐/盘符）+ 行 ··· 菜单
  「打开」+ 工具栏 后退/前进/上级（位置序）。地址栏 Phase 2 改点击面包屑
  进入（mouse-area，无快照 id 面——MCP press 不可达，见 FR-1 残留），套件
  不驱动地址编辑。
- 行定位 = 磁盘镜像排序（目录先、名称升序，与 .at 选择排序一致）→ 行序
  N 的 ··· 按钮。行名在 mouse-area 子树内、不进快照，无法按名寻址。
- 菜单/模态按钮按守卫语义驱动：· 状态守卫（ctx_id/new_modal_open/
  rename_open/confirm_del_open）使关闭态弹层按钮为无害 no-op——关闭态
  弹层文本恒在快照中（污染），文本在场断言一律禁用，改用
  autoui_state 断言 + 磁盘效果断言。
- 弹窗效果断言走 __toast（框架 toast 状态回写）+ 磁盘。

独立实例纪律：AUTOUI_MCP_PORT=9531 起（9529/9247 为常驻实例保留），
独立 storage 文件；绝不 taskkill 任何在途 ui_desktop。

工作目录纪律：workdir 建在 home 下（fm-mcp-<pid>，侧栏「主目录」可达，
地址栏不可驱动故 temp 根不可达），全部破坏性操作限于 workdir，finally
清理。绝不触用户真实文件。

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

# 独立实例端口（9529/9247 已被占用/保留）。
MCP_PORT_BASE = 9531


def find_auto_bin():
    if "AUTO_BIN" in os.environ and os.path.exists(os.environ["AUTO_BIN"]):
        return os.environ["AUTO_BIN"]
    here = os.path.dirname(os.path.abspath(__file__))
    candidates = [
        os.path.normpath(os.path.join(here, "..", "..", "..", "..", "target", "debug", "auto.exe")),
        os.path.normpath(os.path.join(here, "..", "..", "..", "..", "..", "target", "debug", "auto.exe")),
        "D:\\autostack\\auto-lang\\target\\debug\\auto.exe",
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return candidates[0]


AUTO_BIN = find_auto_bin()
PROJECT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))


class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        # MCP 服务器线程偶发静默失联（进程存活、socket 消失，PLAN-016
        # 框架债 F-3 同族）——连接层失败容忍重试，30s 内恢复则继续。
        last = None
        data = None
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
        time.sleep(0.15)  # 节流：缓解连续快压下的服务器线程失联
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def press(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def press_ok(self, element_id, what=""):
        """press 并校验动作寻址（stale vnode id 会得到 not found 错误文本
        ——视图全量重建（F-6）下 id 会漂移，必须检出）。只判定寻址失败：
        ActionResult 正文含 toast.error 分类字样属正常回执。"""
        out = self.press(element_id)
        if "not found" in out:
            raise RuntimeError(f"press failed ({what}): {out[:200]}")
        return out

    def type_text(self, element_id, text):
        return self.call("autoui_type", element_id=element_id, text=text)

    def state(self, *fields):
        """autoui_state → dict（'name: value (kind)' 行解析）。

        str 值剥外层引号并反转印转义（`\\\\` → `\\`）；int/bool 保持字面。"""
        text = self.call("autoui_state", fields=list(fields))
        out = {}

        def unquote(v):
            if len(v) >= 2 and v.startswith('"') and v.endswith('"'):
                return v[1:-1].replace("\\\\", "\\").replace('\\"', '"')
            return v

        for m in re.finditer(r"(\w+): (.+?) \((?:int|str|bool|list|float|unknown)\)", text):
            out[m.group(1)] = unquote(m.group(2))
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


def find_id(snapshot_text, pattern, last=False):
    ms = re.findall(pattern, snapshot_text)
    if not ms:
        return None
    return ms[-1] if last else ms[0]


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


class Driver:
    """Phase 2 惯例驱动层：位置/标签寻址 + 守卫语义。"""

    def __init__(self, mcp, result):
        self.mcp = mcp
        self.result = result

    def state(self, *fields):
        return self.mcp.state(*fields)

    def labeled_button(self, label, last=False):
        # tooltip 按钮（title: prop）的快照标签带 PUA 哨兵前缀 U+EE03，
        # 正则容忍任意私用区前缀；普通文本标签按钮无前缀。
        pat = r'button #(\S+) "[-]*%s"' % re.escape(label)
        return find_id(self.mcp.snapshot(), pat, last=last)

    def nav_buttons(self):
        """工具栏 后退/前进/上级 = 快照前三个空标签按钮（icon-only）。"""
        empties = re.findall(r'button #(\S+) ""', self.mcp.snapshot())
        if len(empties) < 3:
            return None
        return empties[0], empties[1], empties[2]

    def view_pair(self):
        """列表/网格切换对（搜索框后两个空标签按钮）。

        快照顺序：nav×3 → 搜索 input → 视图对×2 → tooltip 按钮×3 → 表头 → 行。
        取「搜索框之后」的前两个空标签 button。
        """
        snap = self.mcp.snapshot()
        si = snap.find('placeholder: "搜索当前目录..."')
        seg = snap[si:si + 2500]
        empties = re.findall(r'button #(\S+) ""', seg)
        if len(empties) < 2:
            return None
        return empties[0], empties[1]

    def nav_press(self, which, wait=1.0):
        """按导航钮（0=后退 1=前进 2=上级）。每次重取 id——导航本身触发
        全量重建（F-6），预取 id 跨按会漂移。"""
        nav = self.nav_buttons()
        if not nav:
            return False
        self.mcp.press_ok(nav[which], "nav%d" % which)
        time.sleep(wait)
        return True

    def row_dots_buttons(self):
        """列表区全部 ··· 按钮（行序 = 磁盘镜像序）。

        ··· 按钮 = 表头之后、菜单项之前的空标签 button。实现：取
        「"修改日期" 表头之后」的全部空标签 button，剔除每个行单元内
        ··· 之后的菜单项（菜单项有标签，天然不匹配空标签模式）。
        """
        snap = self.mcp.snapshot()
        hi = snap.find('"修改日期"')
        if hi < 0:
            return []
        return re.findall(r'button #(\S+) ""', snap[hi:])

    def mirror_order(self, dir_path):
        """磁盘镜像排序：目录先、名称升序（sort_col=name/asc 语义），
        隐藏项排除（dot-prefix）。"""
        names = os.listdir(dir_path)
        visible = [n for n in names if not n.startswith(".")]
        dirs = sorted(n for n in visible if os.path.isdir(os.path.join(dir_path, n)))
        files = sorted(n for n in visible if not os.path.isdir(os.path.join(dir_path, n)))
        return dirs + files

    def open_row_menu(self, row_index):
        """按行序按 ···（ItemCtx：ctx_id=行 id + 选中联动）。"""
        dots = self.row_dots_buttons()
        if row_index >= len(dots):
            return False
        self.mcp.press(dots[row_index])
        time.sleep(0.6)
        return True

    def menu_action(self, label):
        """按任一菜单项（全部行菜单共享 ctx_id 守卫派发——目标由
        ··· 选定的 ctx_id 决定，与按下的菜单实例无关）。"""
        bid = self.labeled_button(label)
        if not bid:
            return False
        self.mcp.press_ok(bid, f"menu:{label}")
        time.sleep(0.8)
        return True

    def modal_input(self, placeholder):
        """按 placeholder 寻址模态输入框——关闭态模态的 input 也恒在
        快照中（污染），inputs[-1] 位次寻址会打错模态。"""
        snap = self.mcp.snapshot()
        m = re.search(r'input #(\S+) \{\s*placeholder: "%s"' % re.escape(placeholder), snap)
        return m.group(1) if m else None

    def press_action_button(self, label, placeholder=None, new_text=None):
        """模态 footer 动作钮（视图树尾最后一个同名标签）；可选先向
        指定 placeholder 输入框键入。键入触发全量重建（F-6）→ 键入后
        重取按钮 id。"""
        if placeholder and new_text is not None:
            iid = self.modal_input(placeholder)
            if not iid:
                return False
            self.mcp.type_text(iid, new_text)
            time.sleep(0.4)
        bid = self.labeled_button(label, last=True)
        if not bid:
            return False
        self.mcp.press_ok(bid, f"action:{label}")
        time.sleep(1.2)
        return True

    def open_row(self, row_index):
        """···→打开（Phase 2 打开惯例；目录 = 导航）。"""
        if not self.open_row_menu(row_index):
            return False
        return self.menu_action("打开")

    def open_dir_checked(self, dir_listing, name):
        """打开目录行并验证导航落点（try_until 重试容错——打开按压
        偶发丢帧时不重试就会停在原目录，后续断言全部连锁失真）。"""
        target = os.path.join(dir_listing, name)

        def flow():
            order = self.mirror_order(dir_listing)
            if name not in order:
                return
            self.open_row(order.index(name))

        return self.try_until(flow, lambda: self.state("current_path").get("current_path", "") == target)

    def nav_to(self, label):
        """侧栏标签导航（主目录/下载/桌面/…/盘符）。"""
        bid = self.labeled_button(label)
        if not bid:
            return False
        self.mcp.press(bid)
        time.sleep(1.0)
        return True

    @staticmethod
    def try_until(fn, check, tries=3, wait=0.9):
        """动作通道偶发丢帧（press 回执 ok 但 iced 侧未执行——PLAN-016
        已知 MCP 通道不稳）→ 动作后按效果判定，未达则重试（有界）。"""
        for i in range(tries):
            fn()
            time.sleep(wait)
            if check():
                return True
        return False

    def delete_row(self, dir_listing, name):
        """···→删除→确认删除（整流程），磁盘效果判定 + 有界重试。"""
        target = os.path.join(dir_listing, name)

        def flow():
            order = self.mirror_order(dir_listing)
            if name not in order:
                return  # 已删
            if not self.open_row_menu(order.index(name)):
                return
            if not self.menu_action("删除"):
                return
            self.press_action_button("确认删除")

        return self.try_until(
            flow,
            lambda: not os.path.exists(target),
        )


def seed_workdir(workdir):
    """workdir 布局：
    fm-mcp-<pid>/
      data/            （notes.txt config.toml photo.png nested/inner.txt .hidden.md）
      empty-dir/
      sub1/            （错误路径用例：建→删→前进）
    """
    data = os.path.join(workdir, "data")
    os.makedirs(os.path.join(data, "nested"))
    with open(os.path.join(data, "notes.txt"), "w", encoding="utf-8") as f:
        f.write("hello from PLAN-016 testdata")
    with open(os.path.join(data, "config.toml"), "w", encoding="utf-8") as f:
        f.write("[section]\nkey = \"value\"\n")
    # 1x1 红 PNG（内容无关紧要——本套件不驱动文件打开，防外部程序弹窗）
    with open(os.path.join(data, "photo.png"), "wb") as f:
        f.write(bytes.fromhex(
            "89504e470d0a1a0a0000000d49484452000000010000000108020000009077"
            "53de0000000c4944415408d763f8cfc00000030101"))
    with open(os.path.join(data, "nested", "inner.txt"), "w", encoding="utf-8") as f:
        f.write("inner")
    with open(os.path.join(data, ".hidden.md"), "w", encoding="utf-8") as f:
        f.write("hidden")
    os.makedirs(os.path.join(workdir, "empty-dir"))
    os.makedirs(os.path.join(workdir, "sub1"))


def run_suite(mcp, workdir, result):
    d = Driver(mcp, result)
    data = os.path.join(workdir, "data")

    # ── T1: 启动结构（真实 FS + 侧栏 + Phase 2 工具栏）─────────────────────
    print("\n[T1] 启动结构（Phase 2 惯例）")
    snap = mcp.snapshot()
    result.check("T1 快速访问侧栏", "快速访问" in snap and "此电脑" in snap)
    result.check("T1 快捷目录标签", all(k in snap for k in ("主目录", "桌面", "文档", "下载")))
    result.check("T1 tooltip 图标按钮", all(
        k in snap for k in ("显示/隐藏隐藏项", "新建文件夹", "新建文件")))
    result.check("T1 导航三钮在场", d.nav_buttons() is not None)
    # 注：修改日期文本位于行 mouse-area 子树内（不进快照，FR-1 残留），
    # 格式证据由 R4 实机截图 p2-list-final.png 承载，套件不重复断言。
    st = d.state("home", "item_count_str", "booted", "in_desktop")
    result.check("T1 主目录解析", st.get("home", "") not in ("",), str(st))
    result.check("T1 独立窗口 in_desktop=false", st.get("in_desktop") == "false", str(st))
    # 磁盘一致性：home 可见条目数 == 应用计数（真实 read_dir + 过滤）
    mirror_n = len(d.mirror_order(st.get("home", "").strip('"')))
    result.check("T1 主目录计数与磁盘一致",
                 st.get("item_count_str") == f"{mirror_n} 个项目",
                 f"app={st.get('item_count_str')} disk={mirror_n}")

    # ── T2: 侧栏导航（Phase 2 惯例：侧栏 = 目录直达）────────────────────────
    print("\n[T2] 侧栏导航")
    if not d.nav_to("下载"):
        result.check("T2 侧栏下载按钮", False, "下载 button not found")
    else:
        st = d.state("current_path", "can_back")
        result.check("T2 下载目录到位", st.get("current_path", "").endswith("Downloads"), str(st))
        result.check("T2 can_back 翻转", st.get("can_back") == "true", str(st))
    result.check("T2 返回主目录", d.nav_to("主目录"))
    st = d.state("current_path")
    home = st.get("current_path", "").strip('"')

    # ── T3: ···→打开 workdir（Phase 2 打开惯例 + 行定位惯例）────────────────
    print("\n[T3] ···→打开 workdir")
    wname = os.path.basename(workdir)
    order = d.mirror_order(home)
    result.check("T3 workdir 在主目录列出", wname in order, str(order[:5]))
    if wname in order:
        result.check("T3 行菜单打开 workdir", d.open_dir_checked(home, wname))
        st = d.state("current_path", "item_count_str")
        result.check("T3 workdir 到位", st.get("current_path", "").strip('"') == workdir, str(st))
        result.check("T3 workdir 计数一致",
                     st.get("item_count_str") == f"{len(d.mirror_order(workdir))} 个项目",
                     str(st))
        st = d.state("crumbs")
        result.check("T3 面包屑层级随路径", st.get("crumbs", "").count("<vmref>") >= 3, str(st))

    # ── T4: ···→打开 data + 列表一致性 ──────────────────────────────────────
    print("\n[T4] ···→打开 data")
    result.check("T4 data 行打开", d.open_dir_checked(workdir, "data"))
    st = d.state("current_path", "item_count_str")
    result.check("T4 data 到位", st.get("current_path", "").strip('"') == data, str(st))
    # 可见条目 = notes/config/photo/nested（.hidden.md 过滤）
    result.check("T4 可见计数 4", st.get("item_count_str") == "4 个项目", str(st))

    # ── T5: 隐藏项开关（tooltip 按钮 + 状态/计数双断言）──────────────────────
    print("\n[T5] 隐藏项开关")
    hid = d.labeled_button("显示/隐藏隐藏项")
    result.check("T5 隐藏项按钮", hid is not None)
    if hid:
        mcp.press(hid)
        time.sleep(0.8)
        st = d.state("show_hidden", "item_count_str")
        result.check("T5 显出 .hidden.md（计数 5）",
                     st.get("show_hidden") == "true" and st.get("item_count_str") == "5 个项目",
                     str(st))
        mcp.press(hid)
        time.sleep(0.8)
        st = d.state("show_hidden", "item_count_str")
        result.check("T5 再隐（计数 4）",
                     st.get("show_hidden") == "false" and st.get("item_count_str") == "4 个项目",
                     str(st))

    # ── T6: 搜索过滤 ────────────────────────────────────────────────────────
    print("\n[T6] 搜索过滤")
    search = find_id(mcp.snapshot(), r'input #(\S+)')
    result.check("T6 搜索框在场", search is not None)
    if search:
        mcp.type_text(search, "notes")
        time.sleep(0.8)
        st = d.state("item_count_str", "search_q")
        result.check("T6 过滤至 1 项", st.get("item_count_str") == "1 个项目", str(st))
        mcp.type_text(search, "")
        time.sleep(0.8)
        st = d.state("item_count_str")
        result.check("T6 清空恢复", st.get("item_count_str") == "4 个项目", str(st))

    # ── T7: 排序表头（真实排序状态）──────────────────────────────────────────
    print("\n[T7] 排序表头")
    sz = d.labeled_button("大小")
    result.check("T7 大小表头按钮", sz is not None)
    if sz:
        mcp.press(sz)
        time.sleep(0.8)
        st = d.state("sort_col", "sort_dir")
        result.check("T7 sort_col=size", st.get("sort_col") == "size", str(st))
        nm = d.labeled_button("名称")
        if nm:
            mcp.press(nm)
            time.sleep(0.8)
            st = d.state("sort_col")
            result.check("T7 回到 name", st.get("sort_col") == "name", str(st))

    # ── T8: 新建文件夹（alert-dialog 模态 + 磁盘 + toast）────────────────────
    print("\n[T8] 新建文件夹")
    nf = d.labeled_button("新建文件夹")
    result.check("T8 新建按钮", nf is not None)
    if nf:
        mcp.press(nf)
        time.sleep(0.8)
        st = d.state("new_modal_open", "new_modal_type")
        result.check("T8 模态开", st.get("new_modal_open") == "true", str(st))
        # 关闭态模态的 input 恒在快照（污染）→ 按 placeholder 寻址新建模态框
        result.check("T8 模态输入框(placeholder 寻址)", d.modal_input("请输入名称...") is not None)
        result.check("T8 提交动作钮", d.press_action_button("创建", "请输入名称...", "fm-new"))
        st = d.state("new_modal_open")
        result.check("T8 模态关", st.get("new_modal_open") == "false", str(st))
        result.check("T8 磁盘落盘", os.path.isdir(os.path.join(data, "fm-new")))
        # 注：toast 内容在状态面不可观测（renderer 消费即清空，__toast 臂）；
        # 视觉证据见复审归档 rev-probe toast 截图（AC-04 证据链）。

    # ── T9: 选中联动（··· 即选中）────────────────────────────────────────────
    print("\n[T9] ··· 选中联动")
    result.check("T9 ··· 定位 fm-new", d.open_row_menu(d.mirror_order(data).index("fm-new")))
    st = d.state("selected_info", "ctx_id")
    result.check("T9 选定信息", "fm-new" in st.get("selected_info", ""), str(st))
    result.check("T9 ctx 打开", int(st.get("ctx_id", "-1")) >= 0, str(st))
    d.menu_action("复制")  # 关菜单（无害：clipboard 面不落盘）
    st = d.state("ctx_id")
    result.check("T9 菜单关 ctx_id=-1", st.get("ctx_id") == "-1", str(st))

    # ── T10: 重命名（···→重命名 → 模态 → 磁盘）───────────────────────────────
    print("\n[T10] 重命名 fm-new → fm-renamed")
    result.check("T10 ··· 定位", d.open_row_menu(d.mirror_order(data).index("fm-new")))
    result.check("T10 菜单重命名项", d.menu_action("重命名"))
    st = d.state("rename_open", "edit_name")
    result.check("T10 重命名模态开", st.get("rename_open") == "true", str(st))
    result.check("T10 模态输入+提交",
                 d.press_action_button("重命名", "请输入新名称...", "fm-renamed"))
    st = d.state("rename_open")
    result.check("T10 模态关", st.get("rename_open") == "false", str(st))
    result.check("T10 新名落盘", os.path.isdir(os.path.join(data, "fm-renamed")))
    result.check("T10 旧名移除", not os.path.exists(os.path.join(data, "fm-new")))

    # ── T11: 删除文件（···→删除 → 确认模态 → 磁盘）───────────────────────────
    print("\n[T11] 删除文件")
    with open(os.path.join(data, "fm-renamed", "a.txt"), "w", encoding="utf-8") as f:
        f.write("")
    # 进 fm-renamed（空目录（除 a.txt）→ 镜像序 [a.txt]）
    result.check("T11 打开 fm-renamed", d.open_dir_checked(data, "fm-renamed"))
    st = d.state("current_path")
    result.check("T11 到位 fm-renamed",
                 st.get("current_path", "").strip('"') == os.path.join(data, "fm-renamed"), str(st))
    result.check("T11 ··· 定位 a.txt", d.open_row_menu(0))
    result.check("T11 菜单删除项", d.menu_action("删除"))
    st = d.state("confirm_del_open", "confirm_del_name")
    result.check("T11 确认模态开+目标名",
                 st.get("confirm_del_open") == "true" and "a.txt" in st.get("confirm_del_name", ""),
                 str(st))
    result.check("T11 确认执行", d.press_action_button("确认删除"))
    st = d.state("confirm_del_open")
    result.check("T11 磁盘移除", not os.path.exists(os.path.join(data, "fm-renamed", "a.txt")))
    result.check("T11 模态关", st.get("confirm_del_open") == "false", str(st))

    # ── T12: 删除空目录（D-4 口径）+ 非空目录拒绝 ────────────────────────────
    print("\n[T12] 删除空目录 / 非空拒绝")
    # 非空：nested（含 inner.txt）→ 拒绝 toast，磁盘保持。当前在
    # fm-renamed → 上级回 data。
    d.nav_press(2)  # 上级 → data
    result.check("T12 ··· 定位 nested(非空)", d.open_row_menu(d.mirror_order(data).index("nested")))
    result.check("T12 菜单删除项", d.menu_action("删除"))
    result.check("T12 确认执行(拒绝路径)", d.press_action_button("确认删除"))
    st = d.state("confirm_del_open")
    result.check("T12 nested 仍在盘(D-4 非空拒绝)", os.path.isdir(os.path.join(data, "nested")), str(st))
    # 空目录：上级回 workdir → empty-dir → 删除成功
    d.nav_press(2)  # 上级 → workdir
    result.check("T12 空目录删除流程(重试容错)",
                 d.delete_row(workdir, "empty-dir"))

    # ── T13: 导航栈（上级/后退/前进，真实历史）────────────────────────────────
    print("\n[T13] 导航栈")
    nav = d.nav_buttons()
    result.check("T13 导航钮", nav is not None)
    if nav:
        d.open_dir_checked(workdir, "data")  # workdir → data

        def cur():
            return d.state("current_path").get("current_path", "")

        result.check("T13 上级到位", d.try_until(
            lambda: d.nav_press(2), lambda: cur() == workdir))
        result.check("T13 后退到位", d.try_until(
            lambda: d.nav_press(0), lambda: cur() == data))
        result.check("T13 前进到位", d.try_until(
            lambda: d.nav_press(1), lambda: cur() == workdir))
        st = d.state("current_path", "can_back", "can_forward")
        # 栈顶语义：后退可用；前进已耗尽（idx 在末端）
        result.check("T13 栈顶可用性", st.get("can_back") == "true" and st.get("can_forward") == "false", str(st))

    # ── T14: 错误路径 toast（AC-06：后退到已删除目录）─────────────────────────
    print("\n[T14] 错误路径 toast")
    # sub1 建于 workdir：进入（入历史）→ 上级回 workdir → 经 ctx 删除 sub1
    # → 后退（GoBack 先减 idx 再 NavTo(sub1)）→ canonical 失败 → 错误 toast，
    # current_path 不变、不崩溃。（前进向在任意新导航后即被截断，历史栈
    # 语义下"后退到已删路径"才是可驱动的错误路径。）
    result.check("T14 sub1 打开", d.open_dir_checked(workdir, "sub1"))
    st = d.state("history_idx")

    def cur():
        return d.state("current_path").get("current_path", "")

    result.check("T14 上级回 workdir", d.try_until(
        lambda: d.nav_press(2), lambda: cur() == workdir))
    idx_up = int(d.state("history_idx").get("history_idx", "0"))
    result.check("T14 删除 sub1(重试容错)", d.delete_row(workdir, "sub1"))
    # 后退落到已删除的 sub1：GoBack 先减 idx 再 NavTo——canonical 失败 →
    # 错误 toast（渲染面），idx 已减证明后退真实落地，路径保持证明错误臂。
    result.check("T14 后退落地(重试容错)", d.try_until(
        lambda: d.nav_press(0, wait=1.2),
        lambda: int(d.state("history_idx").get("history_idx", "0")) == idx_up - 1,
        wait=1.0))
    st = d.state("current_path", "booted", "item_count_str")
    result.check("T14 路径不变(错误臂)", st.get("current_path", "") == workdir, str(st))
    result.check("T14 不崩溃（booted 保持）", st.get("booted") == "true", str(st))
    result.check("T14 列表未被破坏", "个项目" in st.get("item_count_str", ""), str(st))

    # ── T15: 空目录空态 ─────────────────────────────────────────────────────
    print("\n[T15] 空目录空态")
    empt = os.path.join(workdir, "empty-t15")
    os.makedirs(empt)
    d.nav_to("主目录")
    # 主目录行序会变（新增 empty-t15）→ 重镜像（open_dir_checked 内部重镜像）
    result.check("T15 回 workdir", d.open_dir_checked(home, wname))
    result.check("T15 打开 empty-t15", d.open_dir_checked(workdir, "empty-t15"))
    st = d.state("item_count_str", "has_items")
    result.check("T15 计数 0", st.get("item_count_str") == "0 个项目", str(st))
    snap = mcp.snapshot()
    result.check("T15 空态文案", "此目录为空" in snap)
    shutil.rmtree(empt, ignore_errors=True)

    # ── T16 前置: 视图模式切 grid（Phase 2 持久化断言用）────────────────────
    print("\n[T16 前置] 视图切 grid")
    vp = d.view_pair()
    result.check("T16 视图对在场", vp is not None)
    if vp:
        mcp.press_ok(vp[1], "grid")
        time.sleep(0.6)
        st = d.state("view_mode")
        result.check("T16 view_mode=grid", st.get("view_mode") == "grid", str(st))


def run_full(mcp_port, mcp_url, workdir, tmp_storage, result):
    """Phase 1（完整套件）+ Phase 2（重启持久化断言）。"""
    # ── Phase 1 ──
    proc = launch(mcp_port, tmp_storage, fresh=True)
    try:
        if not wait_for_server(mcp_url):
            raise RuntimeError("MCP server 启动超时")
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

        run_suite(client, workdir, result)
    finally:
        proc.kill()
        proc.wait()
        print("第一轮 VM 进程已退出。")

    # ── Phase 2: 重启验证持久化（view_mode=grid 恢复）──
    print("\n[Phase 2] 重启 VM 进程验证配置恢复...")
    mcp_port2 = MCP_PORT_BASE + 1
    mcp_url2 = f"http://localhost:{mcp_port2}/mcp"
    proc2 = launch(mcp_port2, tmp_storage, fresh=False)
    try:
        if not wait_for_server(mcp_url2):
            result.failed += 1
            result.errors.append("Restart MCP server timeout")
        else:
            client2 = McpClient(mcp_url2)
            time.sleep(2.0)
            st = client2.state("view_mode", "show_hidden")
            result.check("T16 重启恢复 view_mode=grid", st.get("view_mode") == "grid", str(st))
    finally:
        proc2.kill()
        proc2.wait()
        print("第二轮 VM 进程已退出。")


def main():
    mcp_port = MCP_PORT_BASE
    mcp_url = f"http://localhost:{mcp_port}/mcp"

    # workdir 建在主目录下（侧栏「主目录」可达；地址编辑不可驱动，
    # temp 根不可达）——全部破坏性操作限于 workdir，finally 清理。
    home = os.path.expanduser("~")
    workdir = os.path.join(home, f"fm-mcp-{os.getpid()}")
    os.makedirs(workdir)
    seed_workdir(workdir)
    tmp_storage = os.path.join(tempfile.gettempdir(), f"fm-mcp-{os.getpid()}-storage.at")

    print("=" * 60)
    print("027-file-manager 桌面 MCP 测试（PLAN-016 Phase 2 惯例套件）")
    print(f"  auto:     {AUTO_BIN}")
    print(f"  project:  {PROJECT}")
    print(f"  workdir:  {workdir}")
    print(f"  mcp port: {mcp_port}")
    print("=" * 60)

    result = TestResult()

    # 整轮重试：实例中途 socket 消失（PLAN-016 已知通道不稳的重度形态）
    # 时整轮重来——workdir 重播种，避免残留状态污染断言。
    for attempt in range(2):
        if attempt:
            print(f"\n[RETRY {attempt}] 上一轮通道失联，重建 workdir 重跑……")
            shutil.rmtree(workdir, ignore_errors=True)
            os.makedirs(workdir)
            seed_workdir(workdir)
            result = TestResult()
        try:
            run_full(mcp_port, mcp_url, workdir, tmp_storage, result)
            break
        except (requests.ConnectionError, requests.Timeout) as e:
            if attempt == 1:
                result.failed += 1
                result.errors.append(f"channel lost twice: {e}")
                break
            print(f"[RETRY] 通道失联：{e}")

    shutil.rmtree(workdir, ignore_errors=True)
    if os.path.exists(tmp_storage):
        try:
            os.remove(tmp_storage)
        except Exception:
            pass
    print("workdir 已清理。")

    print("\n" + "=" * 60)
    print(f"测试结果: {result.passed} 通过, {result.failed} 失败")
    if result.errors:
        for err in result.errors:
            print(f"  FAIL  {err}")
    print("=" * 60)

    sys.exit(0 if result.failed == 0 else 1)


if __name__ == "__main__":
    main()
