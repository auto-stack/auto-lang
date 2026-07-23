#!/usr/bin/env python3
"""spec-index.py — 扫描 docs/specs/ 生成 INDEX.md（唯一事实源是各 project.md，勿手改 INDEX.md）。

用法: python scripts/spec-index.py
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPECS = ROOT / "docs" / "specs"
OUT = SPECS / "INDEX.md"

# project 目录名 -> 视图分组（分组只是视图，不是实体层级）
GROUPS = {
    "语言核心": ["auto-lang", "auto-val", "auto-atom", "a2r-std", "stdlib", "aavm"],
    "工具链": ["auto-cli", "auto-man", "auto-gen", "auto-lsp", "auto-vm",
               "auto-cache", "auto-bindgen", "auto-macros"],
    "UI/Web 生态": ["auto-playground", "widgets", "forge-ui", "lab-ui",
                    "playground-vue", "website", "blocks"],
    "外围/验证": ["parity"],
}

STATUS_RE = re.compile(r"^>\s*\*\*Status\*\*:\s*(\S+)", re.M)
TITLE_RE = re.compile(r"^#\s+(.+)$", re.M)
MODULE_ROW_RE = re.compile(
    r"^\|\s*(?:\[([^\]]+)\]\(([^)]+)\)|([^|\n]+?))\s*\|([^|]*)\|([^|]*)\|",
    re.M,
)


def parse_project(proj_dir: Path):
    card = proj_dir / "project.md"
    if not card.exists():
        return None
    text = card.read_text(encoding="utf-8")
    title = TITLE_RE.search(text)
    status = STATUS_RE.search(text)
    modules = []
    for m in MODULE_ROW_RE.finditer(text):
        name = (m.group(1) or m.group(3) or "").strip()
        if not name or name == "模块" or set(name) <= {"-", ":", " "}:  # 跳过表头/分隔行
            continue
        modules.append((name, m.group(4).strip(), m.group(5).strip()))
    return {
        "name": proj_dir.name,
        "title": title.group(1).strip() if title else proj_dir.name,
        "status": status.group(1).strip() if status else "?",
        "modules": modules,
    }


def main():
    projects = {}
    for d in sorted(SPECS.iterdir()):
        if d.is_dir() and not d.name.startswith("_"):
            info = parse_project(d)
            if info:
                projects[d.name] = info

    known = {p for ps in GROUPS.values() for p in ps}
    ungrouped = [n for n in projects if n not in known]

    lines = [
        "# Specs 全局索引",
        "",
        "> **本文件由 `scripts/spec-index.py` 生成，请勿手改。**",
        "> 规约见 [README.md](README.md)；设计见 "
        "[docs/design/plan-spec-hybrid-model.md](../design/plan-spec-hybrid-model.md)。",
        "",
    ]
    for group, members in GROUPS.items():
        present = [projects[n] for n in members if n in projects]
        if not present:
            continue
        lines += [f"## {group}", "",
                  "| Project | 状态 | 模块数 | 项目卡 |",
                  "|---|---|---|---|"]
        for p in present:
            lines.append(
                f"| {p['title']} | {p['status']} | {len(p['modules'])} | "
                f"[{p['name']}/project.md]({p['name']}/project.md) |"
            )
        lines.append("")
        for p in present:
            if not p["modules"]:
                continue
            lines.append(f"<details><summary>{p['name']} 模块明细</summary>")
            lines += ["", "| 模块 | 职责 | 状态 |", "|---|---|---|"]
            for name, desc, status in p["modules"]:
                lines.append(f"| {name} | {desc} | {status} |")
            lines += ["", "</details>", ""]
    if ungrouped:
        lines += ["## 未分组", ""]
        for n in ungrouped:
            lines.append(f"- [{n}]({n}/project.md)（请在 scripts/spec-index.py 的 GROUPS 中登记）")
        lines.append("")

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(projects)} projects")
    if ungrouped:
        print(f"warning: ungrouped projects: {', '.join(ungrouped)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
