#!/usr/bin/env python3
"""Plan 562 T5: widgets-gallery app.at 外壳 nav-item → sidebar 族机械转换。

转换规则（限定在 aside 的 scroll 块内）：
- scroll (style: S) {            → sidebar_provider (class: ...) { sidebar_content (style: S) {
- text "G" (style: ...) {}       → sidebar_group { sidebar_group_label { text "G" } sidebar_group_content { sidebar_menu {
- nav-item (to: T, icon: I, label: L) → sidebar_menu_item { sidebar_menu_button (to: T[, exact]) { icon (name: I) {} text "L" {} } }
- 组结束（下一个 text 标签行或块尾）→ } } } }
- navlink/navitem 两个条目跳过（页面删除）
- 路由表 "/navlink" "/navitem" 两行删除
"""
import re
import sys

PATH = "examples/widgets-gallery/src/front/app.at"
src = open(PATH, encoding="utf-8").read()
lines = src.splitlines(keepends=False)

out = []
i = 0
n = len(lines)
skip_routes = ('"/navlink" -> use navlink', '"/navitem" -> use navitem')
converted_items = 0
converted_groups = 0

# 组闭合状态机：在 scroll 块内，遇到新组 text 行时先闭合上一组
in_scroll = False
group_open = False
scroll_close_indent = None
pending_comments = []  # 组间注释缓存：归属下一组，在上一组闭合后输出

NAV_ITEM_RE = re.compile(
    r'^(\s*)nav-item \(to: "([^"]+)", icon: "([^"]+)", label: "([^"]+)"\)\s*$'
)
GROUP_RE = re.compile(r'^(\s*)text "([^"]+)" \(style: "[^"]*"\) \{\}\s*$')

while i < n:
    line = lines[i]

    # 路由表删行
    if any(r in line for r in skip_routes):
        i += 1
        continue

    # scroll 块入口（aside 内唯一）
    m = re.match(r'^(\s*)scroll \(style: "([^"]+)"\) \{\s*$', line)
    if m and not in_scroll:
        ind, style = m.group(1), m.group(2)
        out.append(f'{ind}sidebar_provider (class: "w-full min-h-0 flex-1 flex-col") {{')
        out.append(f'{ind}    sidebar_content (style: "{style}") {{')
        in_scroll = True
        scroll_close_indent = ind
        i += 1
        continue

    if in_scroll:
        gm = GROUP_RE.match(line)
        nm = NAV_ITEM_RE.match(line)
        if gm:
            # 闭合上一组
            if group_open:
                ind = gm.group(1)
                out.append(f'{ind}        }}')          # sidebar_menu
                out.append(f'{ind}    }}')              # sidebar_group_content
                out.append(f'{ind}}}')                  # sidebar_group
                group_open = False
            out.extend(pending_comments)
            pending_comments.clear()
            ind, label = gm.group(1), gm.group(2)
            out.append(f'{ind}sidebar_group {{')
            out.append(f'{ind}    sidebar_group_label {{ text "{label}" }}')
            out.append(f'{ind}    sidebar_group_content {{')
            out.append(f'{ind}        sidebar_menu {{')
            group_open = True
            converted_groups += 1
            i += 1
            continue
        if nm:
            ind, to, icon, label = nm.groups()
            out.extend(pending_comments)
            pending_comments.clear()
            if to in ("/navlink", "/navitem"):
                i += 1
                continue  # 页面删除，导航项一并移除
            exact = ", exact: true" if to == "/" else ""
            out.append(f'{ind}            sidebar_menu_item {{')
            out.append(f'{ind}                sidebar_menu_button (to: "{to}"{exact}) {{')
            out.append(f'{ind}                    icon (name: "{icon}") {{}}')
            out.append(f'{ind}                    text "{label}" {{}}')
            out.append(f'{ind}                }}')
            out.append(f'{ind}            }}')
            converted_items += 1
            i += 1
            continue
        # 组间注释行：缓存到下一组/下一项之后再输出
        if group_open and line.strip().startswith("//"):
            pending_comments.append(line)
            i += 1
            continue
        # scroll 块结束行（与入口同缩进的 }）
        if line.strip() == "}" and len(line) - len(line.lstrip()) == len(scroll_close_indent):
            if group_open:
                ind = scroll_close_indent
                out.append(f'{ind}        }}')          # sidebar_menu
                out.append(f'{ind}    }}')              # sidebar_group_content
                out.append(f'{ind}}}')                  # sidebar_group
                group_open = False
            out.extend(pending_comments)
            pending_comments.clear()
            out.append(f'{scroll_close_indent}    }}')  # sidebar_content
            out.append(f'{scroll_close_indent}}}')      # sidebar_provider
            in_scroll = False
            i += 1
            continue

    out.append(line)
    i += 1

open(PATH, "w", encoding="utf-8", newline="\n").write("\n".join(out) + "\n")
print(f"groups={converted_groups} items={converted_items}")
if converted_items != 66 or converted_groups != 9:
    print("WARN: 数量对不上预期（66 items / 9 groups），请人工核对", file=sys.stderr)
    sys.exit(1)
