# -*- coding: utf-8 -*-
"""Dual-end screenshot parity probe + budget gate (015-notes VM vs Vue).

Measures, on matched logical coordinates:
  * semantic surface colors (page/editor/card + sidebar) — PLAN-619 P1
  * sidebar vertical landmarks (search row, chips rows, first note row)
  * icon ink boxes (top-bar glyph; threshold-insensitive)         — PLAN-619 P2
  * left insets: search-row icon / group label / note title      — PLAN-619 P3/P4

Usage:
  python tools/parity_shot_diff.py <vue.png> <vm.png> [--scale 2]
  python tools/parity_shot_diff.py <vue.png> <vm.png> --scale 2 --no-budget

Budgets (PLAN-619 §6; `--no-budget` prints numbers only):
  * semantic surface   : per-channel |delta| <= 2
  * icon ink size      : VM/Vue width & height ratio in [0.9, 1.1]
  * left inset         : |delta| <= 1 px (search icon, group label, note title)

Exit code 1 when any budget is violated, so the probe can sit in a gate.
"""
import sys
from PIL import Image

DEFAULTS = {
    # 015-notes 采样点（逻辑坐标，1280x800 视口；与 PLAN-616 基线截图同源）。
    "page_editor": (640, 500),
    "sidebar": (160, 600),
    "top_icon_box": (14, 38, 14, 40),      # 顶栏 notebook 图标扫描区
    "top_icon_bg": (640, 20),
    "search_icon_y": 86,                   # 搜索行图标（P3）
    "label_y": 202,                        # 首个分组标签文本（P4）
    "note_title_y": 240,                   # 首条便签标题文本（P4）
    "note_title_ref_x": 100,               # 行内底色采样列（避开文字）
    "ref_col": 200,                        # 侧栏卡片色采样列（带内空白处）
}

BUDGET_COLOR = 2
BUDGET_INSET = 1.0
INK_RATIO_MIN = 0.9
INK_RATIO_MAX = 1.1


def g(im, scale, x, y):
    return im.getpixel((min(int(x * scale), im.size[0] - 1), min(int(y * scale), im.size[1] - 1)))


def col_bands(im, scale, x, y0, y1, bg, tol=6, min_h=1.5):
    out, cur = [], None
    for yi in range(int(y0 * scale), int(y1 * scale)):
        y = yi / scale
        ink = max(abs(a - b) for a, b in zip(g(im, scale, x, y), bg)) > tol
        if ink and cur is None:
            cur = y
        if not ink and cur is not None:
            if y - cur >= min_h:
                out.append((round(cur, 1), round(y - 0.5, 1)))
            cur = None
    return out


def ink_bbox(im, scale, x0, x1, y0, y1, bg, tol=6):
    xs, ys = [], []
    for xi in range(int(x0 * scale), int(x1 * scale)):
        for yi in range(int(y0 * scale), int(y1 * scale)):
            x, y = xi / scale, yi / scale
            if max(abs(a - b) for a, b in zip(g(im, scale, x, y), bg)) > tol:
                xs.append(x)
                ys.append(y)
    if not xs:
        return None
    return {
        "x": min(xs), "y": min(ys),
        "w": round(max(xs) - min(xs) + 0.5 / scale, 1),
        "h": round(max(ys) - min(ys) + 0.5 / scale, 1),
    }


def first_ink_x(im, scale, y, x0, x1, ref, tol=8):
    """行 y 上自 x0 向右的第一处「偏离参考色」的 x —— 左缩进量测。"""
    for xi in range(int(x0 * scale), int(x1 * scale)):
        c = im.getpixel((xi, int(y * scale)))
        if max(abs(a - b) for a, b in zip(c, ref)) > tol:
            return round(xi / scale, 1)
    return None


def first_ink_x_band(im, scale, y, x0, x1, ref, half=4, tol=8):
    """y±half 行窗口内的最小 first_ink_x —— 容忍两端 ≤4px 的纵向地标差
    （PLAN-619 前的行盒高度差异会把整条列表推移 6~8px）。"""
    xs = []
    for dy in range(-half, half + 1):
        v = first_ink_x(im, scale, y + dy, x0, x1, ref, tol)
        if v is not None:
            xs.append(v)
    return min(xs) if xs else None


def dominant_color(im, scale, x, y0, y1):
    """纵向带上出现最多的颜色 ≈ 该列的背景色（侧栏卡片色）。"""
    counts = {}
    for yi in range(int(y0 * scale), int(y1 * scale)):
        c = im.getpixel((int(x * scale), yi))
        counts[c] = counts.get(c, 0) + 1
    return max(counts.items(), key=lambda kv: kv[1])[0]


def parse_argv(argv):
    pos, scale, budget = [], 2, True
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--scale":
            scale = int(argv[i + 1]); i += 2
        elif a == "--no-budget":
            budget = False; i += 1
        else:
            pos.append(a); i += 1
    return pos, scale, budget


def main():
    args, scale, budget = parse_argv(sys.argv[1:])
    vue = Image.open(args[0]).convert("RGB")
    vm = Image.open(args[1]).convert("RGB")
    d = DEFAULTS
    violations = []

    print("== 语义面（应与 token 表一致；两端应对齐） ==")
    for name, key in [("page/editor bg", "page_editor"), ("sidebar bg", "sidebar")]:
        x, y = d[key]
        cv, cm = g(vue, 1, x, y), g(vm, scale, x, y)
        delta = max(abs(a - b) for a, b in zip(cv, cm))
        print(f"  {name:16s} vue={cv}  vm={cm}  maxd={delta}")
        if budget and delta > BUDGET_COLOR:
            violations.append(f"语义面 {name} 逐通道差 {delta} > {BUDGET_COLOR}（P1）")

    print("== 侧栏纵向地标（x=90 列的颜色带） ==")
    vb = col_bands(vue, 1, 90, 56, 150, g(vue, 1, 90, 60))[:6]
    mb = col_bands(vm, scale, 90, 56, 150, g(vm, scale, 90, 60))[:6]
    print("  vue:", vb)
    print("  vm :", mb)

    x0, x1, y0, y1 = d["top_icon_box"]
    print("== 图标 ink（顶栏 notebook 图标） ==")
    iv = ink_bbox(vue, 1, x0, x1, y0, y1, g(vue, 1, *d["top_icon_bg"]))
    im_ = ink_bbox(vm, scale, x0, x1, y0, y1, g(vm, scale, *d["top_icon_bg"]))
    print("  vue:", iv)
    print("  vm :", im_)
    if iv and im_:
        for axis in ("w", "h"):
            if iv[axis] <= 0:
                continue
            ratio = im_[axis] / iv[axis]
            print(f"  ink {axis} 比 vm/vue = {ratio:.2f}")
            if budget and not (INK_RATIO_MIN <= ratio <= INK_RATIO_MAX):
                violations.append(
                    f"图标 ink {axis} 比 {ratio:.2f} 越界 [{INK_RATIO_MIN},{INK_RATIO_MAX}]（P2）"
                )
    elif budget:
        violations.append("图标 ink 未测得（P2 无法判定）")

    print("== 左缩进（P3 搜索行左边界 / P4 分组标签、便签标题文字） ==")
    # 侧栏卡片色作为带内参考：空白列 x=200 在 y 56..260 上的众数色。
    ref_v = dominant_color(vue, 1, d["ref_col"], 56, 260)
    ref_m = dominant_color(vm, scale, d["ref_col"], 56, 260)
    for label, y, (xa, xb) in [
        # 搜索行：底色块(bg-background)与卡片色不同 → 第一处偏离 = 行盒左边界
        # （P3 的症状就是这一边界：vue = mx-3(12)，修前 vm = 0）。
        ("search row", d["search_icon_y"], (0, 60)),
        # 分组标签没有底色 → 第一处偏离 = 标签文字起点（P4）。
        ("group label", d["label_y"], (0, 120)),
        # 便签标题落在选中行底色块上：改以行内底色为参考、从行内起步扫，
        # 第一处偏离 = 标题文字起点（P4 的文本缩进）。
        ("note title", d["note_title_y"], (20, 120)),
    ]:
        if label == "note title":
            rv = g(vue, 1, d["note_title_ref_x"], y)
            rm = g(vm, scale, d["note_title_ref_x"], y)
        else:
            rv, rm = ref_v, ref_m
        fv = first_ink_x_band(vue, 1, y, xa, xb, rv)
        fm = first_ink_x_band(vm, scale, y, xa, xb, rm)
        delta = None if fv is None or fm is None else abs(fv - fm)
        print(f"  {label:12s} vue={fv}  vm={fm}  delta={delta}")
        if budget:
            if delta is None:
                violations.append(f"左缩进 {label} 未测得")
            elif delta > BUDGET_INSET:
                violations.append(f"左缩进 {label} 差 {delta}px > {BUDGET_INSET}px（P3/P4）")

    if violations:
        print("== 预算违规 ==")
        for v in violations:
            print("  x", v)
        sys.exit(1)
    if budget:
        print("== 预算内 OK（色 <=%d/通道，ink 比 [%.1f,%.1f]，缩进 <=%.0fpx） ==" % (
            BUDGET_COLOR, INK_RATIO_MIN, INK_RATIO_MAX, BUDGET_INSET))


if __name__ == "__main__":
    main()
