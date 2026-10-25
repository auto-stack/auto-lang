#!/usr/bin/env python3
"""cross-repo-tiers.py — PLAN-831/832：三仓测试档位编排入口。

owner 拆分后各仓持有自己的档位：
- 语言仓（auto-lang）：t / tf / tv / tt / tb / tu / taa / ta / th / t3；
- Man 仓（auto-man）：t（workspace lib + auto bin）、tc（管理 Core）、ti（集成）；
- UI 仓（auto-ui）：t、tu、ti、tn（native 手工档）。

本脚本是跨仓 CI 编排的实际入口（各仓均无 .github/workflows，本地受管
入口即 CI 入口）：
- run  子命令按固定序串行执行默认队列（lang-t → lang-tu → man-t → man-ti
  → ui-t → ui-tu → ui-ti），首红即停；收据 JSON 记录各仓 HEAD/脏态与每档命令、
  退出码、时长。
- list 子命令产出各档正典测试 ID 面（nextest list / libtest --list，
  只构建不运行）——迁前后选择面对拍的证据入口。

边界与纪律（AGENTS.md 并行纪律一致）：
- 重型/批量档（tf 批量回归、taa/ta/t3、画廊围栏、1M churn）不进本队列：
  tf 归 /auto-plan:regress 主语言检出单实例；机器级单实例闸门
  （heavy_gate，D:/autostack/.locks/<family>.lock）归语言仓 heavy 族。
- tn 为手工 native 门，仅显式 --tiers ui-tn 时执行。
- 不创建任何 junction/symlink；跨仓根解析序：CLI 参数 > env
  （AUTO_LANG_ROOT/AUTO_MAN_ROOT/AUTO_UI_ROOT/AUTO_DOWN_ROOT）> 组内兄弟（本仓检出
  父目录同名目录）> D:/autostack 主检出。

退出码：0 全部通过；1 任一档非零（收据仍落盘）；2 用法/解析错误。
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

TOOL = "scripts/cross-repo-tiers.py"
RECEIPT_SCHEMA = "cross-repo-tiers-receipt/1"
LISTING_SCHEMA = "cross-repo-tier-listing/1"

REPOS = ("auto-lang", "auto-man", "auto-ui", "auto-down")

# 档位表：repo → alias/命令（与各仓 .cargo/config.toml 的 [alias] 对齐；
# 变更别名时同步此处）。nextest 档用 nextest list 对齐构建/选择语义；
# bare cargo test 档（tu/tn）用 libtest --list。
TIERS: dict[str, dict] = {
    "lang-t": {"repo": "auto-lang", "run": ["cargo", "t"],
               "list": ["cargo", "nextest", "list", "-p", "auto-lang", "-p", "auto-lang-core",
                        "--lib", "--test", "schema_drift", "--test", "docs_gen",
                        "--test", "component_registry_test",
                        "--features", "auto-lang/ui-iced,auto-lang/test-cases"]},
    "lang-tu": {"repo": "auto-lang", "run": ["cargo", "tu"],
                "list": ["cargo", "test", "-p", "auto-lang", "--lib",
                         "--features", "auto-lang/ui-iced", "ui_gen::", "--", "--list"]},
    "man-t": {"repo": "auto-man", "run": ["cargo", "t"],
              "list": ["cargo", "nextest", "list", "--workspace", "--lib", "--bin", "auto"]},
    "man-ti": {"repo": "auto-man", "run": ["cargo", "ti"],
               "list": ["cargo", "nextest", "list", "-p", "auto", "--test", "plan816_cli_driver",
                        "--test", "cmd_ui", "--test", "autodesk_client_smoke",
                        "-p", "auto-man-runtime", "--test", "plan816_t03_non_ui_service",
                        "--test", "plan816_t03_ui_default_fallback", "--test", "plan816_t03_ui_inject_explicit_first",
                        "-p", "auto-man", "--test", "plan803_asset_facade_tests",
                        "--test", "plan805_toolchain_parity", "--test", "plan805_type_identity",
                        "--test", "plan809_management_core", "--test", "plan811_media_consumers",
                        "--test", "plan815_t04_compat", "--test", "plan816_t03_create_jet_default",
                        "--test", "plan816_t03_legacy_util_paths", "--test", "plan816_t03_sidecar_request",
                        "--test", "plan816_t03_ui_inject_compat"]},
    "ui-t": {"repo": "auto-ui", "run": ["cargo", "t"],
             "list": ["cargo", "nextest", "list", "--workspace", "--lib"]},
    "ui-tu": {"repo": "auto-ui", "run": ["cargo", "tu"],
              "list": ["cargo", "test", "-p", "auto-ui-compiler", "--lib", "ui_gen::", "--", "--list"]},
    "ui-ti": {"repo": "auto-ui", "run": ["cargo", "ti"],
              "list": ["cargo", "nextest", "list", "-p", "auto-ui-tools", "-p", "auto-ui-runtime",
                       "--test", "legacy_build_semantics", "--test", "lifecycle",
                       "--test", "resources_manifest", "--test", "tool_process",
                       "--test", "osconfig_integration"]},
    "ui-tn": {"repo": "auto-ui", "run": ["cargo", "tn"], "manual_gate": True,
              "list": ["cargo", "test", "-p", "auto-ui-runtime",
                       "--features", "native-dnd,test-native-dock",
                       "--test", "native_dnd_e2e", "--", "--list"]},
}

DEFAULT_QUEUE = ("lang-t", "lang-tu", "man-t", "man-ti", "ui-t", "ui-tu", "ui-ti")


def resolve_root(repo_id: str, override: str | None) -> Path:
    if override:
        p = Path(override)
        if not p.is_dir():
            raise SystemExit(f"[{TOOL}] --{repo_id}-root 不存在: {override}")
        return p
    env_key = repo_id.upper().replace("-", "_") + "_ROOT"
    env_val = os.environ.get(env_key)
    if env_val and Path(env_val).is_dir():
        return Path(env_val)
    here = Path(__file__).resolve().parent.parent  # scripts/ → 仓根
    sibling = (here.parent / repo_id) if repo_id != "auto-lang" else here
    if sibling.is_dir():
        return sibling
    main = Path("D:/autostack") / repo_id
    if main.is_dir():
        return main
    raise SystemExit(
        f"[{TOOL}] {repo_id} 根不可解析（{env_key} 未设、组内兄弟与 D:/autostack 主检出均缺席）"
    )


def repo_identity(root: Path) -> dict:
    def git(*args: str) -> str:
        proc = subprocess.run(["git", "-C", str(root), *args],
                              capture_output=True, text=True, encoding="utf-8", errors="replace")
        return proc.stdout.strip() if proc.returncode == 0 else ""
    return {
        "root_name": root.name,
        "head": git("rev-parse", "HEAD"),
        "branch": git("symbolic-ref", "--short", "HEAD"),
        "tracked_dirty": bool(git("status", "--porcelain")),
    }


def run_cmd(cmd: list[str], cwd: Path) -> dict:
    start = time.monotonic()
    proc = subprocess.run(cmd, cwd=str(cwd))
    return {"command": cmd, "exit": proc.returncode,
            "duration_s": round(time.monotonic() - start, 1)}


def cmd_run(args) -> int:
    roots = {rid: resolve_root(rid, getattr(args, rid.replace("-", "_") + "_root"))
             for rid in REPOS}
    tiers = args.tiers or DEFAULT_QUEUE
    unknown = [t for t in tiers if t not in TIERS]
    if unknown:
        raise SystemExit(f"[{TOOL}] 未知档位: {unknown}（可用: {sorted(TIERS)}）")
    receipt = {
        "schema": RECEIPT_SCHEMA, "tool": TOOL, "mode": "run",
        "repos": {rid: repo_identity(r) for rid, r in sorted(roots.items())},
        "queue": list(tiers), "results": [], "verdict": "pass",
    }
    exit_code = 0
    for tier in tiers:
        spec = TIERS[tier]
        root = roots[spec["repo"]]
        sys.stdout.write(f"[{TOOL}] {tier} @ {spec['repo']} {root} → {' '.join(spec['run'])}\n")
        sys.stdout.flush()
        result = {"tier": tier, "repo": spec["repo"], **run_cmd(spec["run"], root)}
        receipt["results"].append(result)
        if result["exit"] != 0:
            receipt["verdict"] = "fail"
            exit_code = 1
            sys.stdout.write(f"[{TOOL}] {tier} 红（exit {result['exit']}），首红即停\n")
            break
    out = Path(args.output) if args.output else (
        roots["auto-lang"].parent / "scratch-cross-repo-tiers" / "run-receipt.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    sys.stdout.write(f"[{TOOL}] 收据: {out}（verdict={receipt['verdict']}）\n")
    return exit_code


def cmd_list(args) -> int:
    roots = {rid: resolve_root(rid, getattr(args, rid.replace("-", "_") + "_root"))
             for rid in REPOS}
    tiers = args.tiers or list(DEFAULT_QUEUE) + ["ui-tn"]
    unknown = [t for t in tiers if t not in TIERS]
    if unknown:
        raise SystemExit(f"[{TOOL}] 未知档位: {unknown}（可用: {sorted(TIERS)}）")
    listing = {
        "schema": LISTING_SCHEMA, "tool": TOOL, "mode": "list",
        "repos": {rid: repo_identity(r) for rid, r in sorted(roots.items())},
        "faces": {},
    }
    for tier in tiers:
        spec = TIERS[tier]
        root = roots[spec["repo"]]
        sys.stdout.write(f"[{TOOL}] list {tier} @ {spec['repo']}\n")
        sys.stdout.flush()
        proc = subprocess.run(spec["list"], cwd=str(root), capture_output=True,
                              text=True, encoding="utf-8", errors="replace")
        if proc.returncode != 0:
            sys.stderr.write(proc.stdout[-2000:] + proc.stderr[-2000:])
            raise SystemExit(f"[{TOOL}] {tier} 列举失败（exit {proc.returncode}）")
        # nextest list: 每行 "pkg::test: 文件:行"；libtest --list: "test_name ... ignore 标记"
        lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
        listing["faces"][tier] = {"repo": spec["repo"], "count": len(lines), "ids": lines}
    out = Path(args.output) if args.output else (
        roots["auto-lang"].parent / "scratch-cross-repo-tiers" / "tier-faces.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(listing, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    total = sum(f["count"] for f in listing["faces"].values())
    sys.stdout.write(f"[{TOOL}] 面清单: {out}（{len(listing['faces'])} 档 / {total} 条 ID）\n")
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    for rid in REPOS:
        parser.add_argument(f"--{rid}-root", help=f"{rid} 根目录（缺省：env/组内兄弟/主检出解析序）")
    parser.add_argument("--tiers", nargs="*", help=f"档位子集（缺省 run={list(DEFAULT_QUEUE)} / list=含 ui-tn）")
    parser.add_argument("--output", help="收据/清单输出路径（缺省：组目录 scratch-cross-repo-tiers/）")
    parser.add_argument("mode", choices=("run", "list"), help="执行模式：run=串行执行档位队列；list=只构建列举正典 ID 面")
    args = parser.parse_args(argv)
    return cmd_run(args) if args.mode == "run" else cmd_list(args)


if __name__ == "__main__":
    sys.exit(main())
