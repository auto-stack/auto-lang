#!/usr/bin/env python
"""Plan 523 W3:四路统一 runner / 三件套金样 的 Python 薄壳(CI/本地双形态)。

Rust 测试件承载(见 crates/auto-lang/src/tests/aavm2_a2r.rs):
  - test_aavm2_fourpath_runner  四途径一致判定 + 译文回链(#[ignore],
    shells cargo/rustc)
  - test_aavm2_goldens_check     三件套金样 --check / A2R_BLESS 再生

用法:
  python scripts/aavm4_check.py --check     # 金样校验(轻,日常)
  python scripts/aavm4_check.py --fourpath  # 四路判定表(重,验收/折叠点)
  python scripts/aavm4_check.py --bless     # 金样再生(live 覆写,diff 走 git 评审)

环境透传:A2R_BLESS(--bless 置位)、A2R_DUMP(可手工叠加)。
"""
import argparse
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def run(filter_name: str, ignored: bool, bless: bool) -> int:
    env = os.environ.copy()
    if bless:
        env["A2R_BLESS"] = "1"
    cmd = [
        "cargo", "test", "--manifest-path", os.path.join(ROOT, "Cargo.toml"),
        "-p", "auto-lang", "--lib", "--features", "test-vm-files",
        filter_name, "--", "--nocapture",
    ]
    if ignored:
        cmd.append("--ignored")
    print("[aavm4]", " ".join(cmd), f"(A2R_BLESS={'1' if bless else '0'})")
    return subprocess.call(cmd, env=env, cwd=ROOT)


def main() -> int:
    ap = argparse.ArgumentParser(description="aavm 四路/金样 薄壳 (Plan 523)")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--check", action="store_true", help="三件套金样 --check")
    g.add_argument("--fourpath", action="store_true", help="四路判定表(重)")
    g.add_argument("--bless", action="store_true", help="金样再生(diff 走评审)")
    args = ap.parse_args()

    if args.check:
        return run("goldens_check", ignored=False, bless=False)
    if args.bless:
        return run("goldens_check", ignored=False, bless=True)
    if args.fourpath:
        return run("fourpath_runner", ignored=True, bless=False)
    return 2


if __name__ == "__main__":
    sys.exit(main())
