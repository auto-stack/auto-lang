#!/usr/bin/env python3
"""ui-repository-fixture.py — PLAN-831 有界三仓 fixture 入口（T-03 冻结）。

解析 auto-lang / auto-ui / auto-down 三仓（解析序：CLI 参数 → env
AUTO_LANG_ROOT/AUTO_UI_ROOT/AUTO_DOWN_ROOT → 语言仓组内兄弟 → D:/autostack
主检出），调用 test/fixtures/modularity/ui-repository-consumer/driver.py 的
五族 driver，并把各族收据聚合成 run 收据。接口契约（族/断言/阶段/收据 schema）
见该 fixture 的 README.md 与 migration-phases.json。

有界性：透传 driver 的有界默认（manifest 直解 + git rev-parse，零 cargo）；
发行/失败族的执行 phase 门控在 driver 内。重负载（--full 域）按 Plan T-07/T-08
由 generated-ui-consumer driver 等既有入口单实例执行，本入口不做编排。

用法：
  python scripts/ui-repository-fixture.py [--lang-repo P] [--ui-repo P] \
      [--down-repo P] [--phase auto|pre-migration|source-switched] \
      [--family source|type|test-id|distribution|failure|all] [--output run.json]

缺省 --family=all；缺省 --output=<语言仓同级>/scratch-ui-repo-fixture/run-receipt.json。
退出码同 driver：0 全通过；2 断言红；3 配置/阶段错配；1 运行错。
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

SCRIPT_ROOT = Path(__file__).resolve().parent.parent
DRIVER = SCRIPT_ROOT / "test/fixtures/modularity/ui-repository-consumer/driver.py"
RUN_SCHEMA = "ui-repository-fixture-run/1"
REPO_ROLES = ("lang", "ui", "down")
ENV_KEYS = {"lang": "AUTO_LANG_ROOT", "ui": "AUTO_UI_ROOT", "down": "AUTO_DOWN_ROOT"}


def log(msg: str) -> None:
    print(f"[ui-repo-entry] {msg}", flush=True)


def resolve_repos(cli: dict[str, str | None]) -> dict[str, Path]:
    """解析序：CLI → env → 语言仓组内兄弟 → D:/autostack 主检出。"""
    # 语言仓先解析（兄弟推导的基点）：脚本所在检出即语言仓检出。
    lang = Path(
        cli["lang"]
        or os.environ.get(ENV_KEYS["lang"])
        or str(SCRIPT_ROOT)
    ).resolve()
    if not (lang / "Cargo.toml").is_file():
        # 脚本被复制到仓外执行等场景：退主检出。
        lang = Path("D:/autostack/auto-lang").resolve()
    resolved = {"lang": lang}
    for role in ("ui", "down"):
        explicit = cli[role] or os.environ.get(ENV_KEYS[role])
        if explicit:
            resolved[role] = Path(explicit).resolve()
            continue
        sibling = lang.parent / f"auto-{'ui' if role == 'ui' else 'down'}"
        # 组内兄弟不存在（如 down 未入组）时按解析序退主检出。
        resolved[role] = sibling.resolve() if sibling.is_dir() else Path(
            f"D:/autostack/auto-{'ui' if role == 'ui' else 'down'}"
        ).resolve()
    return resolved


def git_head(path: Path) -> str | None:
    proc = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        capture_output=True, text=True,
    )
    return proc.stdout.strip() if proc.returncode == 0 else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--lang-repo")
    parser.add_argument("--ui-repo")
    parser.add_argument("--down-repo")
    parser.add_argument("--phase", default="auto",
                        choices=["auto", "pre-migration", "source-switched"])
    parser.add_argument("--family", default="all",
                        choices=["source", "type", "test-id", "distribution",
                                 "failure", "all"])
    parser.add_argument("--output")
    args = parser.parse_args()

    repos = resolve_repos({"lang": args.lang_repo, "ui": args.ui_repo,
                           "down": args.down_repo})
    default_out = repos["lang"].parent / "scratch-ui-repo-fixture" / "run-receipt.json"
    output = Path(args.output) if args.output else default_out

    for role in REPO_ROLES:
        path = repos[role]
        log(f"解析 {role:4} → {path}（manifest={'在' if (path / 'Cargo.toml').is_file() else '缺'}）")

    cmd = [
        sys.executable, str(DRIVER), args.family,
        "--lang-repo", str(repos["lang"]),
        "--ui-repo", str(repos["ui"]),
        "--down-repo", str(repos["down"]),
        "--phase", args.phase,
        "--output", str(output),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True,
                          encoding="utf-8", errors="replace")
    sys.stdout.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    if proc.returncode not in (0, 2):
        return proc.returncode

    family_receipts = json.loads(output.read_text(encoding="utf-8"))
    if isinstance(family_receipts, dict):
        family_receipts = [family_receipts]
    run = {
        "schema": RUN_SCHEMA,
        "entry": "scripts/ui-repository-fixture.py",
        "plan": "PLAN-831",
        "family": args.family,
        "resolution_order": "cli > env(AUTO_*_ROOT) > group-sibling > D:/autostack",
        "repos": {
            role: {
                "path": f"<{role}-root>",
                "actual": str(repos[role]),
                "head": git_head(repos[role]),
                "has_manifest": (repos[role] / "Cargo.toml").is_file(),
            }
            for role in REPO_ROLES
        },
        "families": family_receipts,
        "totals": {
            key: sum(r["totals"].get(key, 0) for r in family_receipts)
            for key in ("assertions", "pass", "fail", "recorded", "pending")
        },
        "driver_exit": proc.returncode,
    }
    output.write_text(
        json.dumps(run, ensure_ascii=False, sort_keys=True, indent=2) + "\n",
        encoding="utf-8", newline="\n",
    )
    log(f"run 收据 → {output}（totals={run['totals']}）")
    return proc.returncode


if __name__ == "__main__":
    sys.exit(main())
