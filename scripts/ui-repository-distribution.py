#!/usr/bin/env python3
"""ui-repository-distribution.py — PLAN-831 T-07 独立 clone 发行面实测（DST-1..4）。

以**真目标仓本地 clone**（auto-ui + auto-lang 双仓组，各自迁移分支 HEAD）
执行冻结的 DST 形态；发行/失败族的 bundle 级形态（DST-5 工具+资产随包、
FLR-2/3/4 等）由 generated-ui-consumer/driver.py（792/815-A）承载，本脚本
以指针引用其收据。

DST 形态（migration-phases.json 冻结口径）：
  DST-1 fresh-lock   组内 auto-ui clone 全新 generate-lockfile（env 剥离）
  DST-2 locked-build clone 内 cargo build --locked -p auto-ui-tools 真实构建
  DST-3 relocate     双仓组 A→B 整组重定位（剥 target/）后 locked 重放
  DST-4 sdk-only     无 auto-ui 兄弟的隔离 lang clone 内 language-sdk-consumer
                     真实 build+test（UI checkout 缺席反向控制）

用法：
  python scripts/ui-repository-distribution.py [--scratch <dir>] [--skip-dst3]

退出码 0 全过；2 任一形态红；1 运行错。收据（含各步时长/摘要）写
<scratch>/distribution-receipt.json（ui-repository-distribution-receipt/1）。
"""

from __future__ import annotations

import argparse
import json
import platform
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

SCRIPT_ROOT = Path(__file__).resolve().parent.parent
SCRUB_ENV_KEYS = [
    "AUTO_RUST_WORKSPACE", "AUTO_LANG_CRATE", "AUTO_LANG_ROOT",
    "AUTO_UI_SOURCE_ROOT", "AUTO_DELIVERY_SOURCE_ROOT", "AUTO_DOWN_ROOT",
    "CARGO_TARGET_DIR", "RUSTC_WRAPPER",
]
RECEIPT_SCHEMA = "ui-repository-distribution-receipt/1"


def log(msg: str) -> None:
    print(f"[ui-dist] {msg}", flush=True)


def scrub_env(extra: dict | None = None) -> dict:
    env = {k: v for k, v in os.environ.items() if k not in SCRUB_ENV_KEYS}
    if extra:
        env.update(extra)
    return env


def run(cmd: list[str], cwd: Path, timeout: int = 5400) -> subprocess.CompletedProcess:
    log(f"$ {' '.join(str(c) for c in cmd)}  (cwd={cwd})")
    t0 = time.time()
    proc = subprocess.run(
        cmd, cwd=str(cwd), env=scrub_env(), capture_output=True, text=True,
        timeout=timeout, encoding="utf-8", errors="replace",
    )
    proc.elapsed = time.time() - t0  # type: ignore[attr-defined]
    return proc


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--scratch", default=str(SCRIPT_ROOT.parent / "scratch-ui-repo-distribution"))
    parser.add_argument("--skip-dst3", action="store_true")
    args = parser.parse_args()

    lang_src = SCRIPT_ROOT                       # 语言仓检出（脚本所在=lang 树）
    ui_src = lang_src.parent / "auto-ui"         # 组内兄弟（AGENTS 解析序）

    scratch = Path(args.scratch).resolve()
    if scratch.is_dir():
        # Windows：本地 clone 的 git 对象带只读属性，rmtree 需先清位重试。
        import os as _os
        import stat as _stat

        def _force_rm(func, path, _exc):
            _os.chmod(path, _stat.S_IWRITE)
            func(path)

        shutil.rmtree(scratch, onexc=_force_rm)
    scratch.mkdir(parents=True)
    receipt: dict = {
        "schema": RECEIPT_SCHEMA, "plan": "PLAN-831", "task": "T-07",
        "tool": "scripts/ui-repository-distribution.py",
        "environment": {"os": platform.platform(), "scrubbed_env": SCRUB_ENV_KEYS},
        "sources": {"lang": None, "ui": None},
        "steps": [], "pointer_792": ("DST-5/FLR-2/3/4（bundle 级工具+资产/坏pin/"
                                     "漏patch/资源四态）由 generated-ui-consumer "
                                     "driver V01..V05 收据承载"),
    }

    def record(step: str, ok: bool, detail: str, elapsed: float) -> bool:
        receipt["steps"].append({"step": step, "ok": ok, "detail": detail,
                                 "elapsed_s": round(elapsed, 1)})
        log(f"{'PASS' if ok else 'FAIL'} {step} ({elapsed:.0f}s): {detail}")
        return ok

    def git_head(path: Path) -> str | None:
        proc = subprocess.run(["git", "-C", str(path), "rev-parse", "HEAD"],
                              capture_output=True, text=True)
        return proc.stdout.strip() if proc.returncode == 0 else None

    receipt["sources"] = {"lang": git_head(lang_src), "ui": git_head(ui_src)}
    if not (ui_src / "crates" / "auto-ui-tools" / "Cargo.toml").is_file():
        log(f"UI 源不可解析: {ui_src}")
        return 1

    # ---- 双仓组 clone（真目标仓本地 clone，非 worktree 复用）----
    group_a = scratch / "grp-a"
    group_a.mkdir()
    # 三仓组：auto-ui + auto-lang + auto-down（runtime 的 optional autodown-core
    # path——cargo 对 optional path 依赖同样加载 manifest，缺席即 metadata/lock 红；
    # 文档布局=同级 auto-down，主检出 HEAD 快照）。
    down_src = lang_src.parent / "auto-down"
    if not down_src.is_dir():
        down_src = Path("D:/autostack/auto-down")
    for repo, src in (("auto-ui", ui_src), ("auto-lang", lang_src), ("auto-down", down_src)):
        r = run(["git", "clone", "--quiet", "--local", str(src), str(group_a / repo)],
                scratch, timeout=600)
        if r.returncode != 0:
            record("clone", False, f"{repo} clone 失败: {r.stderr[-300:]}", 0)
            _write(receipt, scratch)
            return 2
    ui_clone = group_a / "auto-ui"
    record("clone", True,
           f"三仓组 clone @ {group_a.name}（ui={receipt['sources']['ui'][:9] if receipt['sources']['ui'] else '?'} "
           f"lang={receipt['sources']['lang'][:9] if receipt['sources']['lang'] else '?'}）", 0)

    # ---- DST-1 fresh-lock ----
    lock = ui_clone / "Cargo.lock"
    if lock.exists():
        lock.unlink()
    r = run(["cargo", "generate-lockfile"], ui_clone, timeout=1800)
    ok = r.returncode == 0 and lock.is_file()
    five = 0
    if ok:
        text = lock.read_text(encoding="utf-8")
        five = sum(1 for n in ("auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                               "auto-ui-tools", "shell-pack") if f'name = "{n}"' in text)
        ok = five == 5
    record("DST-1-fresh-lock", ok,
           f"fresh 解析成功，五包入锁 {five}/5" if ok else f"exit={r.returncode} five={five} {r.stderr[-300:]}",
           r.elapsed)
    if not ok:
        _write(receipt, scratch)
        return 2

    # ---- DST-2 locked-build ----
    r = run(["cargo", "build", "--locked", "-p", "auto-ui-tools"], ui_clone, timeout=7200)
    exe = ui_clone / "target" / "debug" / "auto-ui-tool.exe"
    record("DST-2-locked-build", r.returncode == 0 and exe.is_file(),
           f"locked 真实构建 auto-ui-tools，auto-ui-tool.exe {'在位' if exe.is_file() else '缺失'}"
           + ("" if r.returncode == 0 else " stderr: " + r.stderr[-300:]),
           r.elapsed)
    if r.returncode != 0:
        _write(receipt, scratch)
        return 2

    # ---- DST-3 relocate A→B ----
    if not args.skip_dst3:
        shutil.rmtree(ui_clone / "target", ignore_errors=True)
        group_b = scratch / "grp-b"
        shutil.move(str(group_a), str(group_b))
        ui_b = group_b / "auto-ui"
        r = run(["cargo", "build", "--locked", "-p", "auto-ui-tools"], ui_b, timeout=7200)
        exe_b = ui_b / "target" / "debug" / "auto-ui-tool.exe"
        record("DST-3-relocate-locked-replay", r.returncode == 0 and exe_b.is_file(),
               f"双仓组 A→B 整组重定位（剥 target）后 locked 重放成功，"
               f"exe {'在位' if exe_b.is_file() else '缺失'}"
               + ("" if r.returncode == 0 else " stderr: " + r.stderr[-300:]),
               r.elapsed)
        if r.returncode != 0:
            _write(receipt, scratch)
            return 2

    # ---- DST-4 sdk-only（隔离 lang clone，无 auto-ui 兄弟）----
    iso = scratch / "iso-lang"
    r = run(["git", "clone", "--quiet", "--local", str(lang_src), str(iso)], scratch, timeout=600)
    fixture = iso / "test" / "fixtures" / "modularity" / "language-sdk-consumer"
    sibling_ui = iso.parent / "auto-ui"
    no_sibling = not sibling_ui.exists()
    r2 = run(["cargo", "test", "--manifest-path", str(fixture / "Cargo.toml")],
             scratch, timeout=3600)
    record("DST-4-sdk-only-no-ui-sibling", r2.returncode == 0 and no_sibling,
           f"隔离 lang clone（auto-ui 兄弟缺席={no_sibling}）内 SDK consumer 真实 build+test"
           + ("" if r2.returncode == 0 else " stderr: " + r2.stderr[-300:]),
           r2.elapsed)

    _write(receipt, scratch)
    failed = [s for s in receipt["steps"] if not s["ok"]]
    log(f"==== {len(receipt['steps']) - len(failed)}/{len(receipt['steps'])} 通过；收据 → {scratch / 'distribution-receipt.json'}")
    return 2 if failed else 0


def _write(receipt: dict, scratch: Path) -> None:
    (scratch / "distribution-receipt.json").write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2), encoding="utf-8"
    )


if __name__ == "__main__":
    sys.exit(main())
