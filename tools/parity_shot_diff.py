# -*- coding: utf-8 -*-
"""Dual-end screenshot parity probe (015-notes VM vs Vue).

Measures, on matched logical coordinates:
  * semantic surface colors (background/card) vs the token tables
  * sidebar vertical landmarks (search row top/bottom, chips row, first list row)
  * icon ink boxes (glyph ink bbox, threshold-insensitive)
Usage: python tools/parity_shot_diff.py <vue.png> <vm.png> [--scale 2]
"""
import sys
from PIL import Image

def load(p, scale):
    return Image.open(p).convert("RGB"), scale

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
                xs.append(x); ys.append(y)
    if not xs:
        return None
    return (min(xs), min(ys), max(xs), max(ys), round(max(xs) - min(xs) + 0.5 / scale, 1),
            round(max(ys) - min(ys) + 0.5 / scale, 1))

def main():
    vue, vscale = load(sys.argv[1], 1)
    vm, mscale = load(sys.argv[2], int(sys.argv[4]) if len(sys.argv) > 4 else 2)
    print("== 语义面（应与 token 表一致；两端应对齐）==")
    for name, (x, y) in {"page/editor bg": (640, 500), "sidebar bg": (160, 600)}.items():
        print(f"  {name:16s} vue={g(vue, vscale, x, y)}  vm={g(vm, mscale, x, y)}")
    print("== 侧栏纵向地标（x=90 列的颜色带）==")
    print("  vue:", col_bands(vue, vscale, 90, 56, 150, g(vue, vscale, 90, 60))[:6])
    print("  vm :", col_bands(vm, mscale, 90, 56, 150, g(vm, mscale, 90, 60))[:6])
    print("== 图标 ink（顶栏 notebook 窗口 x14..38,y14..40）==")
    print("  vue:", ink_bbox(vue, vscale, 14, 38, 14, 40, g(vue, vscale, 640, 20)))
    print("  vm :", ink_bbox(vm, mscale, 14, 38, 14, 40, g(vm, mscale, 640, 20)))

if __name__ == "__main__":
    main()
