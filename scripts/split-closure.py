#!/usr/bin/env python3
"""PLAN-867 T-04: close the pre-split test-ID universe against the current
three-repository faces.

Denominator: scripts/pre-split-universe/universe-ids.txt (frozen by T-01;
acceptance input — never regenerated to accept a loss). Current side: the
PLAN-850 frozen baselines (with the 841 migration ledger applied) plus the
live three-repository batch listing receipt. Every universe ID must be
covered by one of:

  CURRENT    still selected/enumerated by a current face
  MIGRATED   row of scripts/full-regression-migrations-841.json.txt whose new
             ID is covered (owner moved with the split)
  RENAMED    entry in scripts/pre-split-closure-dispositions.json whose new ID
             is covered, with recorded evidence
  RETIRED    entry in scripts/pre-split-closure-dispositions.json citing the
             plan that intentionally retired its subject code

Anything else is LOST: exit 1, unexplained gap — a green run never substitutes
for an explained set.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("full", HERE / "scripts" / "run-full-regression.py")
full = importlib.util.module_from_spec(spec)
spec.loader.exec_module(full)

# cross-repo-tiers.py list lines: nextest = "<bin> <test name>"; libtest tiers
# (ui-tu/ui-tn) = "<test name>: test|ignore" with the package implied by tier.
LIBTEST_TIER_PACKAGE = {"ui-tu": "auto-ui-compiler", "ui-tn": "auto-ui-runtime"}


def cross_repo_ids(tier: str, lines: list[str]) -> set[str]:
    package = LIBTEST_TIER_PACKAGE.get(tier)
    ids = set()
    for line in lines:
        line = line.strip()
        if not line:
            continue
        if line.endswith(": test") or line.endswith(": ignore"):
            if package is None:
                raise SystemExit(f"unexpected libtest line in non-libtest tier {tier}: {line!r}")
            ids.add(package + "\t" + line.rsplit(":", 1)[0].strip())
            continue
        bin_name, separator, name = line.partition(" ")
        if not separator or not name:
            raise SystemExit(f"unparsable cross-repo id line: {line!r}")
        ids.add(bin_name + "\t" + name.strip())
    return ids

BASELINE_FACES = [("tf", "language"), ("tf", "ui-full"), ("tf", "tt"), ("tf", "tb"),
                  ("taa", "language"), ("ta", "language"), ("t3", "language")]
KNOWN_DISCREPANCIES = [
    "language 3718<->live", "ui 3091<->live", "ui_gen 866<->live", "man auto -2 / auto-man +1",
]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_ids(path):
    return set(Path(path).read_text(encoding="utf-8").splitlines())


def current_faces(receipt_dir: Path, cross_repo_faces: Path | None):
    faces: dict[str, dict] = {}
    for tier, label in BASELINE_FACES:
        expected, _required = full.baseline_for(tier, label)
        faces[f"850:{label}@{tier}"] = expected
    if cross_repo_faces is not None:
        doc = json.loads(cross_repo_faces.read_text(encoding="utf-8"))
        if doc.get("schema") != "cross-repo-tier-listing/1":
            raise SystemExit(f"unsupported cross-repo listing schema: {doc.get('schema')}")
        for tier, face in doc["faces"].items():
            faces[f"cross-repo:{tier}"] = cross_repo_ids(tier, face["ids"])
    if receipt_dir is not None:
        receipt = json.loads((receipt_dir / "receipt.json").read_text(encoding="utf-8"))
        for record in receipt["results"]:
            label = record["leg"]
            listing = receipt_dir / (label + "-list.json")
            if listing.is_file():
                ids, selected = full.selection(json.loads(listing.read_text(encoding="utf-8")), allow_empty=True)
            else:
                text_listing = receipt_dir / (label + "-list.txt")
                if not text_listing.is_file():
                    continue
                package = record.get("package") or {"ui-tu": "auto-ui-compiler",
                                                    "ui-tn": "auto-ui-runtime"}.get(label, "libtest")
                ids, selected = full.libtest_selection(text_listing.read_text(encoding="utf-8"),
                                                       package, allow_empty=True)
            faces[f"receipt:{label}"] = ids
    return faces


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--universe", type=Path, default=HERE / "scripts" / "pre-split-universe")
    parser.add_argument("--receipt", type=Path, help="tf --batch --list-only (or full run) receipt directory")
    parser.add_argument("--cross-repo-faces", type=Path,
                        help="tier-faces.json from `scripts/cross-repo-tiers.py list` on the frozen combination")
    parser.add_argument("--extra-face", action="append", default=[], metavar="LABEL=PATH",
                        help="additional current face as raw `nextest list --message-format json` output "
                             "(ids include ignored tests, unlike the cross-repo human listing)")
    parser.add_argument("--dispositions", type=Path,
                        default=HERE / "scripts" / "pre-split-closure-dispositions.json")
    parser.add_argument("--write-md", type=Path,
                        help="write the closure table to this markdown file")
    parser.add_argument("--diff-out", type=Path, help="write raw LOST/added ID lists here (local evidence)")
    args = parser.parse_args(argv)

    universe = read_ids(args.universe / "universe-ids.txt")
    manifest = json.loads((args.universe / "manifest.json").read_text(encoding="utf-8"))
    faces = current_faces(args.receipt, args.cross_repo_faces)
    for item in args.extra_face:
        label, _, path = item.partition("=")
        ids, _selected = full.selection(json.loads(Path(path).read_text(encoding="utf-8")), allow_empty=True)
        faces[f"extra:{label}"] = ids
    current = set().union(*faces.values()) if faces else set()

    migrations = {row["old"]: row for row in full.migrations()}
    retirement_rows = {row["id"]: row for row in full.retirements()}
    dispositions = {}
    if args.dispositions.is_file():
        doc = json.loads(args.dispositions.read_text(encoding="utf-8"))
        if doc.get("schema") != "pre-split-closure-dispositions/1":
            raise SystemExit(f"unsupported dispositions schema: {doc.get('schema')}")
        dispositions = {row["id"]: row for row in doc["dispositions"]}

    covered, migrated, renamed, retired, lost = set(), [], [], [], []
    for identity in sorted(universe):
        if identity in current:
            covered.add(identity)
            continue
        row = migrations.get(identity)
        if row and row["new"] in current:
            migrated.append((identity, row["new"], row["reason"]))
            continue
        entry = dispositions.get(identity)
        if entry and entry["class"] == "RETIRED":
            retired.append((identity, entry["evidence"]))
            continue
        retirement = retirement_rows.get(identity)
        if retirement is not None:
            retired.append((identity, retirement["evidence"]))
            continue
        if entry and entry["class"] == "RENAMED" and entry["new"] in current:
            renamed.append((identity, entry["new"], entry["evidence"]))
            continue
        lost.append(identity)
    added = sorted(current - universe)

    lines = ["# PLAN-867 拆分等价闭包（T-04）", ""]
    lines += ["## 复算输入", ""]
    lines += [f"- 分母 universe-ids.txt：{len(universe)} ids（sha256 {manifest['universe_ids_sha256']}；"
              f"primary commit {manifest['commit'][:9]}"
              + (f"；witness runs: " + ", ".join(run["commit"][:9] + "→" + ",".join(run["faces"])
                 for run in manifest.get("witness_runs", [])) if manifest.get("witness_runs") else "")
              + "）"]
    for tier, label in BASELINE_FACES:
        lines += [f"- 850 冻结基线 {label}@{tier}：{len(faces.get(f'850:{label}@{tier}', set()))} ids"]
    if args.receipt:
        lines += [f"- 三仓收据：{args.receipt}（receipt.json sha256 {sha(args.receipt / 'receipt.json')}）"]
    if args.cross_repo_faces:
        lines += [f"- 跨仓日常面清单：{args.cross_repo_faces}（sha256 {sha(args.cross_repo_faces)}）"]
    lines += [f"- 处置表：{args.dispositions}（sha256 {sha(args.dispositions)}）" if args.dispositions.is_file()
              else "- 处置表：缺失"]
    lines += ["", "## 分类结果", ""]
    lines += [f"- CURRENT（当前面覆盖）：{len(covered)}"]
    lines += [f"- MIGRATED（841 迁移账本）：{len(migrated)}"]
    lines += [f"- RENAMED（处置表，新 ID 已覆盖）：{len(renamed)}"]
    lines += [f"- RETIRED（处置表，证据在案）：{len(retired)}"]
    lines += [f"- LOST（未解释缺口，必须=0）：{len(lost)}"]
    lines += [f"- 新增（当前面 − 分母，仅记录）：{len(added)}"]
    if migrated:
        lines += ["", "### MIGRATED 明细", "", "| old | new | reason |", "|---|---|---|"]
        lines += [f"| `{old}` | `{new}` | {reason} |" for old, new, reason in migrated]
    if renamed:
        lines += ["", "### RENAMED 明细", "", "| old | new | evidence |", "|---|---|---|"]
        lines += [f"| `{old}` | `{new}` | {evidence} |" for old, new, evidence in renamed]
    if retired:
        lines += ["", "### RETIRED 明细", "", "| id | evidence |", "|---|---|"]
        lines += [f"| `{identity}` | {evidence} |" for identity, evidence in retired]
    if lost:
        lines += ["", "### LOST（阻塞）", ""]
        lines += [f"- `{identity}`" for identity in lost]
    lines += ["", "### 已知差额结案对照", ""]
    lines += [f"- {item}" for item in KNOWN_DISCREPANCIES]
    lines += ["", f"结论：{'未解释缺口 = 0，闭包成立' if not lost else f'存在 {len(lost)} 个未解释缺口，阻塞'}"]
    text = "\n".join(lines) + "\n"

    if args.write_md:
        args.write_md.parent.mkdir(parents=True, exist_ok=True)
        args.write_md.write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    if args.diff_out:
        args.diff_out.mkdir(parents=True, exist_ok=True)
        (args.diff_out / "lost.txt").write_text("".join(i + "\n" for i in lost), encoding="utf-8")
        (args.diff_out / "added.txt").write_text("".join(i + "\n" for i in added), encoding="utf-8")
    print(f"closure: covered={len(covered)} migrated={len(migrated)} renamed={len(renamed)} "
          f"retired={len(retired)} LOST={len(lost)} added={len(added)}")
    return 1 if lost else 0


if __name__ == "__main__":
    sys.exit(main())
