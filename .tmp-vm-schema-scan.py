#!/usr/bin/env python3
"""Scan schema/aura.at for iced/VM support vs gallery pages."""
import re
from pathlib import Path

schema = Path(r"D:/autostack/auto-lang/schema/aura.at").read_text(encoding="utf-8")
pages = {p.stem for p in Path(r"D:/autostack/auto-os/widgets-gallery/src/front/pages").glob("*.at")}

# parse elements
els = []
for m in re.finditer(r"element (\w+) \{([\s\S]*?)\n\}", schema):
    name, body = m.group(1), m.group(2)
    be = re.search(r"backends:\s*\{([^}]+)\}", body)
    iced = web = None
    if be:
        d = be.group(1)
        mw = re.search(r'web:\s*"([^"]+)"', d)
        mi = re.search(r'iced:\s*"([^"]+)"', d)
        web = mw.group(1) if mw else None
        iced = mi.group(1) if mi else None
    els.append({"name": name, "web": web, "iced": iced})

# gallery component-ish tags used in pages
page_text = ""
for p in Path(r"D:/autostack/auto-os/widgets-gallery/src/front/pages").glob("*.at"):
    page_text += p.read_text(encoding="utf-8") + "\n"
for p in Path(r"D:/autostack/auto-os/widgets-gallery/src/front/components").glob("*.at"):
    page_text += p.read_text(encoding="utf-8") + "\n"

used = set(re.findall(r"(?m)^\s*([a-z][a-z0-9_-]{2,})\s*[\(\{\"]", page_text))
# also nested tags
used |= set(re.findall(r"([a-z][a-z0-9_-]{2,})\s*\(", page_text))

by_name = {e["name"]: e for e in els}

print("=== iced=none but used in gallery sources ===")
rows = []
for u in sorted(used):
    e = by_name.get(u) or by_name.get(u.replace("-", "_"))
    if e and e["iced"] == "none":
        rows.append((u, e["web"], e["iced"]))
        print(f"  {u:28s} web={e['web']} iced=none")

print("\n=== iced=partial used in gallery ===")
for u in sorted(used):
    e = by_name.get(u) or by_name.get(u.replace("-", "_"))
    if e and e["iced"] == "partial":
        print(f"  {u:28s} web={e['web']}")

print("\n=== gallery page roots with iced none/partial in name match ===")
for page in sorted(pages):
    # page name -> likely primary widget
    cands = [page, page.replace("-", "_"), page.replace("-", "")]
    for c in cands:
        e = by_name.get(c)
        if e and e["iced"] in ("none", "partial"):
            print(f"  page {page:16s} -> {c} iced={e['iced']}")
            break
