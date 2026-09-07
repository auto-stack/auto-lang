#!/usr/bin/env python3
"""Schema drift audit: compare aura.at vs schema.rs vs aura_view_builder.rs.

Extracts tag + prop sets from each source and reports drift.
"""
import re
import sys
from pathlib import Path

ROOT = Path(r"D:\autostack\auto-lang")
AT = ROOT / "schema" / "aura.at"
RS = ROOT / "crates" / "auto-lang" / "src" / "aura" / "schema.rs"
VB = ROOT / "crates" / "auto-lang" / "src" / "ui" / "aura_view_builder.rs"
RS2 = ROOT / "crates" / "auto-lang" / "src" / "aura" / "schema.rs"


def extract_at(path: Path):
    """Parse `element <name> { ... }` blocks from aura.at."""
    text = path.read_text(encoding="utf-8")
    elements = {}
    # Match element blocks (balanced-ish: rely on closing `}` at col 0)
    for m in re.finditer(r'^element\s+(\w+)\s*\{', text, re.M):
        name = m.group(1)
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == '{':
                depth += 1
            elif text[i] == '}':
                depth -= 1
            i += 1
        block = text[start:i]
        props = re.findall(r'name:\s*"([^"]+)"', block)
        tag_m = re.search(r'tag:\s*"([^"]+)"', block)
        tag = tag_m.group(1) if tag_m else name
        # first name: occurrence is inside props list items
        elements[tag] = props
    return elements


def extract_rs(path: Path):
    """Parse `elements.insert("tag", ElementDef { ... });` blocks from schema.rs."""
    text = path.read_text(encoding="utf-8")
    elements = {}
    for m in re.finditer(r'elements\.insert\("([^"]+)"\s*,\s*ElementDef\s*\{', text):
        tag = m.group(1)
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == '{':
                depth += 1
            elif text[i] == '}':
                depth -= 1
            i += 1
        block = text[start:i]
        props = re.findall(r'name:\s*"([^"]+)"', block)
        elements[tag] = props
    return elements


def extract_view_builder(path: Path):
    """Extract tags from the two big `match tag {` dispatch tables.

    Arms look like:  "col" | "column" => self.convert_xxx(
    Also captures the convert fn name for capability reporting.
    """
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    tables = []  # list of dict tag -> convert fn
    current = None  # dict while inside a match tag block
    depth = 0
    for idx, line in enumerate(lines):
        if current is None:
            if re.match(r'\s*match tag \{', line):
                current = {}
                depth = 1
                # skip the line itself
                continue
        else:
            depth += line.count('{') - line.count('}')
            if depth <= 0:
                tables.append(current)
                current = None
                continue
            m = re.match(r'\s*((?:"[^"]+"\s*\|?\s*)+)\s*=>\s*self\.(convert_\w+)', line)
            if m:
                tags = re.findall(r'"([^"]+)"', m.group(1))
                fn = m.group(2)
                for t in tags:
                    current.setdefault(t, fn)
            # multi-line arm pattern: tags on one line, body next line(s)
            m2 = re.match(r'\s*((?:"[^"]+"\s*\|?\s*)+)\s*=>\s*\{', line)
            if m2:
                tags = re.findall(r'"([^"]+)"', m2.group(1))
                for t in tags:
                    current.setdefault(t, f"<block@{idx+1}>")
    if current is not None:
        tables.append(current)
    return tables


def main():
    at = extract_at(AT)
    rs = extract_rs(RS)
    vbs = extract_view_builder(VB)

    print(f"aura.at elements:          {len(at)}")
    print(f"schema.rs elements:        {len(rs)}")
    for i, t in enumerate(vbs):
        print(f"view_builder match[{i}]:     {len(t)} tags")
    print()

    at_set, rs_set = set(at), set(rs)
    # canonical view-builder tag set: union of tracked + untracked tables
    vb_union = set()
    for t in vbs:
        vb_union |= set(t)
    # tracked table = the one containing "col" with convert_column_tracked_ctx
    tracked = max(vbs, key=lambda t: sum(1 for fn in t.values() if 'tracked' in fn)) if vbs else {}
    tracked_set = set(tracked)

    print("=== [1] aura.at vs schema.rs ===")
    only_at = sorted(at_set - rs_set)
    only_rs = sorted(rs_set - at_set)
    print(f"only in aura.at  ({len(only_at)}): {only_at}")
    print(f"only in schema.rs({len(only_rs)}): {only_rs}")
    common = at_set & rs_set
    prop_drift = []
    for tag in sorted(common):
        p_at = set(at[tag])
        p_rs = set(rs[tag])
        if p_at != p_rs:
            prop_drift.append((tag, sorted(p_at - p_rs), sorted(p_rs - p_at)))
    print(f"\ncommon tags: {len(common)}; prop-level drift among them: {len(prop_drift)}")
    for tag, miss_rs, miss_at in prop_drift[:20]:
        print(f"  {tag}: in .at not in .rs {miss_rs} | in .rs not in .at {miss_at}")

    print("\n=== [2] schema.rs vs view_builder (tracked table) ===")
    only_sch = sorted(rs_set - tracked_set)
    only_vb = sorted(tracked_set - rs_set)
    print(f"declared in schema.rs but NOT implemented in tracked dispatch ({len(only_sch)}):")
    print(f"  {only_sch}")
    print(f"implemented in tracked dispatch but NOT in schema.rs ({len(only_vb)}):")
    print(f"  {only_vb}")

    print("\n=== [3] aura.at vs view_builder (tracked table) ===")
    only_at2 = sorted(at_set - tracked_set)
    only_vb2 = sorted(tracked_set - at_set)
    print(f"in aura.at but not implemented ({len(only_at2)}): {only_at2}")
    print(f"implemented but not in aura.at ({len(only_vb2)}): {only_vb2}")

    # save machine-readable dumps
    out = ROOT / "scratch"
    (out / "drift_at.txt").write_text("\n".join(f"{k}\t{','.join(v)}" for k, v in sorted(at.items())), encoding="utf-8")
    (out / "drift_rs.txt").write_text("\n".join(f"{k}\t{','.join(v)}" for k, v in sorted(rs.items())), encoding="utf-8")
    for i, t in enumerate(vbs):
        (out / f"drift_vb{i}.txt").write_text("\n".join(f"{k}\t{fn}" for k, fn in sorted(t.items())), encoding="utf-8")
    print("\nDumps written to scratch/drift_*.txt")


if __name__ == "__main__":
    main()
