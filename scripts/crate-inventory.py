#!/usr/bin/env python3
"""crate-inventory.py — PLAN-869 T-02 三仓 crate 检验清单生成器 + 差异检查。

单一事实源（roadmap §2.1 下半部）：
- 机读事实自动采集：cargo metadata 成员/依赖、measure-source-size.py per-crate
  规模切片、pub 出口计数、跨仓 path 边、867 档位下限映射。
- 分析列（职责/重复实现/无消费者旧路径/文档）来自精选数据
  scripts/crate-inventory-data.json——每个 workspace 成员必须有一条，缺即非零
  退出（V-02 缺栏即红）。
- 产物：docs/specs/repository-boundaries/crate-inventory.md（人读，逐 crate 一行）
  + scripts/crate-inventory.json（机读，含指纹）。
- `--check`：按当前仓状态重生成并与已提交产物逐字节比较，不一致即非零
  （V-02 清单与仓库不一致即红；变更后须重新生成并提交）。

用法：
  python scripts/crate-inventory.py            # 生成（写 md + json）
  python scripts/crate-inventory.py --check    # 差异检查（CI/复核用）
根解析序：AUTO_LANG_ROOT/AUTO_UI_ROOT/AUTO_MAN_ROOT env > 本脚本所在仓的组内
兄弟（../auto-ui、../auto-man）> D:/autostack 主检出。
"""
import argparse
import importlib.util
import json
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent          # scripts/
REPO = HERE.parent                               # auto-lang checkout root
PARENT = REPO.parent                             # group dir or D:/autostack

LANG = Path(os.environ.get("AUTO_LANG_ROOT", str(REPO)))
UI = Path(os.environ.get("AUTO_UI_ROOT",
                         str(PARENT / "auto-ui" if (PARENT / "auto-ui").exists()
                             else "D:/autostack/auto-ui")))
MAN = Path(os.environ.get("AUTO_MAN_ROOT",
                          str(PARENT / "auto-man" if (PARENT / "auto-man").exists()
                              else "D:/autostack/auto-man")))

REPOS = {"auto-lang": LANG, "auto-ui": UI, "auto-man": MAN}
DATA = HERE / "crate-inventory-data.json"
MD_OUT = REPO / "docs/specs/repository-boundaries/crate-inventory.md"
JSON_OUT = HERE / "crate-inventory.json"
MINIMUMS = HERE / "tier-selection-minimums.json.txt"

PUB_RE = re.compile(
    r'^\s*(?:#\[[^\]]*\]\s*)*pub(?:\([a-z]+\))?\s+'
    r'(?:async\s+|unsafe\s+|const\s+)*(fn|struct|enum|trait|const|static|type|mod)\s+',
    re.M)


def load_measure():
    spec = importlib.util.spec_from_file_location(
        "measure", HERE / "measure-source-size.py")
    mod = importlib.util.module_from_spec(spec)
    sys.modules["measure"] = mod
    spec.loader.exec_module(mod)
    return mod


def cargo_metadata(root: Path, manifest=None):
    cmd = ["cargo", "metadata", "--no-deps", "--format-version", "1"]
    cwd = root
    if manifest:
        cmd += ["--manifest-path", manifest]
    proc = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True)
    if proc.returncode:
        raise SystemExit(f"cargo metadata failed for {root}: {proc.stderr[-300:]}")
    return json.loads(proc.stdout)


def git_head(root: Path) -> str:
    proc = subprocess.run(["git", "rev-parse", "HEAD"], cwd=str(root),
                          capture_output=True, text=True, check=True)
    return proc.stdout.strip()


def collect(repo: str, root: Path, measure):
    meta = cargo_metadata(root)
    ws_root = root.resolve()
    pkgs = {}
    for p in meta["packages"]:
        man = Path(p["manifest_path"]).resolve()
        try:
            rel_dir = man.parent.relative_to(ws_root).as_posix()
        except ValueError:
            rel_dir = man.parent.as_posix()
        pkgs[p["name"]] = {
            "manifest_dir": rel_dir,
            "version": p["version"],
            "path_deps": sorted({d["name"] for d in p["dependencies"]
                                 if d.get("path")}),
            "cross_repo_deps": sorted({
                d["name"] for d in p["dependencies"]
                if d.get("path") and not Path(d["path"]).resolve()
                .is_relative_to(ws_root)}),
        }
    _, _, per_crate = measure.measure(repo, root)
    return pkgs, per_crate


def member_key(manifest_dir: str) -> str:
    """manifest dir -> per-crate slice key (crates/<a>/... -> <a> ...)."""
    parts = manifest_dir.rstrip("/").split("/")
    if parts[0] == "crates" and len(parts) >= 2:
        if len(parts) == 2:
            return parts[1]
        return "/".join(parts[1:])   # nested member (auto-cosmic/ports)
    return "(root)"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true",
                    help="regenerate and diff against committed outputs")
    args = ap.parse_args()

    measure = load_measure()
    data = json.loads(DATA.read_text(encoding="utf-8"))
    mins = json.loads(MINIMUMS.read_text(encoding="utf-8"))["legs"]

    inv = {"schema": "crate-inventory/1", "repos": {}, "crates": []}
    problems = []

    all_pkgs = {}
    for repo, root in REPOS.items():
        pkgs, per_crate = collect(repo, root, measure)
        inv["repos"][repo] = {
            "path": str(root), "commit": git_head(root),
            "members": sorted(pkgs),
        }
        all_pkgs.update({f"{repo}:{n}": v for n, v in pkgs.items()})
        # tier ownership from 867 minimums (command -p flags name packages)
        for leg, legdef in mins.items():
            if legdef.get("repo") != repo:
                continue
            for pkg in re.findall(r"-p\s+([A-Za-z0-9_-]+)",
                                  " ".join(legdef.get("command", []))):
                all_pkgs.setdefault(f"{repo}:{pkg}", {}) \
                         .setdefault("tiers", []).append(f"{leg}({legdef['minimum']})")

    # auto-man core/ sub-workspace member (root workspace excludes it; its
    # package name "auto" would collide, so it is inventoried under the
    # dedicated key auto-core@core-ws — a build view sharing crates/auto src)
    core_ws = {}
    if (MAN / "core" / "Cargo.toml").exists():
        meta = cargo_metadata(MAN / "core")
        for p in meta["packages"]:
            man = Path(p["manifest_path"]).resolve()
            rel_dir = man.parent.relative_to(MAN.resolve()).as_posix()
            core_ws = {
                "repo": "auto-man", "name": "auto-core@core-ws",
                "manifest_dir": rel_dir, "version": p["version"],
                "path_deps": sorted({d["name"] for d in p["dependencies"]
                                     if d.get("path")}),
                "cross_repo_deps": sorted({
                    d["name"] for d in p["dependencies"]
                    if d.get("path") and not Path(d["path"]).resolve()
                    .is_relative_to(MAN.resolve())}),
                "tiers": ["man-tcore(23)"],
                "p_files": 0, "p_total": 0, "inline_test": 0, "xt_files": 0,
                "pub_items": 0, "shares_source": "crates/auto",
            }
            break

    used_curated = set()

    def emit(repo, name, info, sl, pub_count, tiers):
        cur = data.get("crates", {}).get(f"{repo}/{name}")
        if cur is None:
            problems.append(f"missing curated entry: {repo}/{name}")
            return None
        used_curated.add(f"{repo}/{name}")
        buckets = sl.get("buckets", {})
        p = buckets.get("P", [0, 0])
        xt = buckets.get("XT", [0, 0])
        return {
            "repo": repo, "name": name, "manifest_dir": info["manifest_dir"],
            "version": info["version"],
            "upstream_path_deps": info["path_deps"],
            "cross_repo_deps": info.get("cross_repo_deps", []),
            "tiers": sorted(set(tiers)),
            "p_files": p[0] if p else 0, "p_total": p[1] if p else 0,
            "inline_test": sl.get("inline_test", 0),
            "xt_files": xt[0] if xt else 0,
            "pub_items": pub_count,
            "duty": cur["duty"],
            "duplicates": cur.get("duplicates", "—"),
            "dead_paths": cur.get("dead_paths", "—"),
            "docs": cur.get("docs", "—"),
        }

    for repo, root in REPOS.items():
        pkgs, per_crate = collect(repo, root, measure)
        for name, info in sorted(pkgs.items(),
                                 key=lambda kv: kv[1]["manifest_dir"]):
            key = member_key(info["manifest_dir"])
            sl = per_crate.get(key, {})
            src = root / info["manifest_dir"] / "src"
            pub_count = 0
            if src.exists():
                for f in src.rglob("*.rs"):
                    try:
                        pub_count += len(PUB_RE.findall(
                            f.read_text(encoding="utf-8", errors="replace")))
                    except OSError:
                        pass
            tiers = all_pkgs.get(f"{repo}:{name}", {}).get("tiers", [])
            entry = emit(repo, name, info, sl, pub_count, tiers)
            if entry:
                inv["crates"].append(entry)

    if core_ws:
        entry = emit(core_ws["repo"], core_ws["name"], core_ws, {}, 0,
                     core_ws["tiers"])
        if entry:
            entry["shares_source"] = core_ws["shares_source"]
            inv["crates"].append(entry)

    unused = set(data.get("crates", {})) - used_curated
    for u in sorted(unused):
        problems.append(f"unused curated entry (no such member): {u}")

    if problems:
        print("INVENTORY INCOMPLETE — curated entries missing (V-02 red):")
        for p in problems:
            print("  -", p)
        return 1

    # ---- emit markdown ----
    md = ["# crate-inventory（三仓 workspace 成员检验清单）",
          "",
          "> **Status**: active（PLAN-869 SD-02）",
          "> 生成命令：`python scripts/crate-inventory.py`；差异检查："
          "`python scripts/crate-inventory.py --check`（清单与仓库不一致即红）。",
          "> 机读版：`scripts/crate-inventory.json`（含三仓 commit/成员指纹）。",
          "",
          "七栏：职责 | 公开出口 | 重复实现 | 无消费者旧路径 | 依赖方向 | "
          "测试与档位 | 文档与规范。行绑定生成时三仓 commit（见文首指纹）。",
          ""]
    for repo, root in REPOS.items():
        md.append(f"## {repo} @ `{inv['repos'][repo]['commit'][:9]}`")
        md.append("")
        md.append("| crate | 职责 | 公开出口 | 重复实现 | 无消费者旧路径 | "
                  "依赖方向 | 测试与档位 | 文档与规范 |")
        md.append("|---|---|---|---|---|---|---|---|")
        for c in inv["crates"]:
            if c["repo"] != repo:
                continue
            dep = f"上游 {len(c['upstream_path_deps'])}"
            if c["cross_repo_deps"]:
                dep += f"；跨仓→ {', '.join(c['cross_repo_deps'])}"
            exports = (f"pub×{c['pub_items']}（P {c['p_files']} 文件/"
                       f"{c['p_total']} 行，inline test {c['inline_test']}）")
            tier = "; ".join(c["tiers"]) or "—（无档位下限锚定）"
            md.append(
                f"| `{c['name']}` | {c['duty']} | {exports} | {c['duplicates']}"
                f" | {c['dead_paths']} | {dep} | {tier} | {c['docs']} |")
        md.append("")

    md += ["## 候选队列（T-03 输入；按 可减行数×风险倒数×证据完备度 排序）", ""]
    md += ["| 排序 | 候选 | 类别 | 证据 |", "|---|---|---|---|"]
    for i, q in enumerate(data.get("candidate_queue", []), 1):
        md.append(f"| {i} | {q['item']} | {q['kind']} | {q['evidence']} |")
    md += ["",
           "> 候选处置规则见 net-slimming（负控五面/四要件/2,000 下限）；"
           "结构性减法项须用户按批签核（D-3 裁定）。"]

    md_text = "\n".join(md) + "\n"

    if args.check:
        ok = True
        if MD_OUT.read_text(encoding="utf-8") != md_text:
            print(f"CHECK FAIL: {MD_OUT} differs from regenerated output")
            ok = False
        if JSON_OUT.read_text(encoding="utf-8") != json.dumps(
                inv, indent=1, sort_keys=True) + "\n":
            print(f"CHECK FAIL: {JSON_OUT} differs from regenerated output")
            ok = False
        print("CHECK PASS" if ok else "CHECK FAIL — regenerate & commit")
        return 0 if ok else 1

    MD_OUT.write_text(md_text, encoding="utf-8")
    JSON_OUT.write_text(json.dumps(inv, indent=1, sort_keys=True) + "\n",
                        encoding="utf-8")
    print(f"[written] {MD_OUT}")
    print(f"[written] {JSON_OUT}")
    per_repo = {r: len(inv["repos"][r]["members"]) for r in REPOS}
    print(f"crates: {len(inv['crates'])} ({per_repo})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
