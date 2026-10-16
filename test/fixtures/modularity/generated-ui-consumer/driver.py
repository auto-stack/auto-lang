#!/usr/bin/env python3
"""PLAN-792 generated-ui-consumer 交付验收 driver（V01..V04）。

用正式生成入口（`auto gen --delivery-root`）在受管 scratch 生成可搬动交付
bundle，然后以无开发根环境覆盖的独立消费者身份完成真实构建与失败控制：

  V01  全新解析 + 真实 build（fresh Cargo.lock，不借 workspace lock/rlib；
       原失败=779/785 归档在案的 fresh-lock wgpu-hal/windows-core 漂移）
  V02  交付包 A→B 重定位（不含 target/），B 内 cargo build --locked；
       metadata 本地 path 全在交付根内、source 唯一；资源与 producer 同源
  V03  负控：资源缺失/坏 hash 判红、消费根漏 patch 确定性红、不兼容 pin 红、
       外来目录显式拒绝；正常输入恢复复绿
  V04  开发模式保持：同一工程 dev 生成走 rust-workspace 路由 + workspace
       继承清单；main.rs 与交付产物逐字节同源（同一编译相位）

用法：
  python test/fixtures/modularity/generated-ui-consumer/driver.py \
      [--scratch <dir>] [--skip-build-auto]

退出码 0 = 全部通过；非 0 = 首个失败步（含负控未按预期红）。
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
WT_ROOT = HERE.parents[3]  # test/fixtures/modularity/<this> -> repo root
PROJ = HERE / "proj"

SCRATCH_ENV_KEYS = [
    "AUTO_RUST_WORKSPACE",
    "AUTO_LANG_CRATE",
    "AUTO_LANG_ROOT",
    "AUTO_DELIVERY_SOURCE_ROOT",
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",  # sccache 端口被 Windows 排除段阻挡（机器事实，收据记录）
]

RESULTS: list[dict] = []


def log(msg: str) -> None:
    print(f"[driver-792] {msg}", flush=True)


def record(case: str, ok: bool, detail: str) -> bool:
    RESULTS.append({"case": case, "ok": ok, "detail": detail})
    log(f"{'PASS' if ok else 'FAIL'} {case}: {detail}")
    return ok


def consumer_env(extra: dict | None = None) -> dict:
    """消费侧环境：剥离全部开发根覆盖（含 RUSTC_WRAPPER），无隐式回退。"""
    env = {k: v for k, v in os.environ.items() if k not in SCRATCH_ENV_KEYS}
    if extra:
        env.update(extra)
    return env


def run(cmd: list[str], cwd: Path, env: dict, timeout: int = 3600) -> subprocess.CompletedProcess:
    log(f"$ {' '.join(str(c) for c in cmd)}  (cwd={cwd})")
    return subprocess.run(
        cmd, cwd=str(cwd), env=env, capture_output=True, text=True,
        timeout=timeout, encoding="utf-8", errors="replace",
    )


def copytree_no_target(src: Path, dst: Path) -> None:
    """重定位拷贝：跳过 target/（不携带任何编译产物作 oracle）。"""
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst, ignore=shutil.ignore_patterns("target"))


def norm_lower(p) -> str:
    return str(p).replace("\\", "/").lower()


def run_verify_entry(bundle: Path) -> subprocess.CompletedProcess:
    """R1-F01：随包受管校验正式入口 verify.py（消费清单 sha256，独立可执行）。"""
    return subprocess.run(
        [sys.executable, str(bundle / "verify.py")],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120,
    )


def verify_delivery(bundle: Path) -> tuple[bool, str]:
    """受管资源校验（双层）：① 正式入口 verify.py（清单 sha256 逐资源 +
    未登记文件判红——无 producer 也可执行）；② 与 producer 源（fixture
    src/front）逐字节对拍。"""
    r = run_verify_entry(bundle)
    if r.returncode != 0:
        return False, (r.stdout + r.stderr).strip()
    mp = bundle / "delivery-manifest.json"
    if not mp.is_file():
        return False, "delivery-manifest.json 缺失"
    manifest = json.loads(mp.read_text(encoding="utf-8"))
    problems = []
    for res in manifest.get("resources", []):
        f = bundle / "assets" / res["rel"]
        if not f.is_file():
            problems.append(f"资源缺失: {res['rel']}")
            continue
        src = PROJ / "src" / "front" / res["rel"]
        if src.is_file() and f.read_bytes() != src.read_bytes():
            problems.append(f"资源字节与 producer 不等: {res['rel']}")
    if problems:
        return False, "; ".join(problems)
    return True, f"{len(manifest.get('resources', []))} 资源校验一致（verify.py 绿 + 字节=producer）"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--scratch", default=str(WT_ROOT.parent / "scratch-consumer"))
    ap.add_argument("--skip-build-auto", action="store_true")
    args = ap.parse_args()

    scratch = Path(args.scratch).resolve()
    # 守卫：scratch 不得位于 repo/worktree 内（仓内 scratch 会改变 V04 的
    # 仓外工程形态——R1 复审实录）。
    if str(scratch).replace("\\", "/").lower().startswith(
        str(WT_ROOT).replace("\\", "/").lower() + "/"
    ):
        log(f"scratch 不得位于 worktree 内：{scratch}")
        return 1
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    receipt: dict = {
        "toolchain": {},
        "environment": {
            "os": platform.platform(),
            "cargo_source_replacement": "aliyun sparse (用户级 ~/.cargo/config.toml)",
            "scrubbed_env": SCRATCH_ENV_KEYS,
        },
    }

    for tool in ("cargo", "rustc"):
        out = subprocess.run([tool, "--version"], capture_output=True, text=True)
        receipt["toolchain"][tool] = out.stdout.strip()
    log(f"toolchain: {receipt['toolchain']}")

    # ---- 生成器（正式入口）：auto CLI 构建 ----
    auto_exe = WT_ROOT / "target" / "debug" / "auto.exe"
    if not args.skip_build_auto or not auto_exe.is_file():
        t0 = time.time()
        r = run(["cargo", "build", "-p", "auto"], WT_ROOT, consumer_env(), timeout=2400)
        if r.returncode != 0:
            record("build-auto-cli", False, r.stderr[-800:])
            return 1
        record("build-auto-cli", True, f"{time.time() - t0:.0f}s")

    # ---- 正式生成入口：dev 副本 + --delivery-root ----
    gen_proj = scratch / "generation-run"
    shutil.copytree(PROJ, gen_proj)
    bundle_a = scratch / "consumer-a"
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(bundle_a)],
        gen_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    if r.returncode != 0:
        record("generate-delivery", False, (r.stdout + r.stderr)[-1200:])
        return 1
    if not (bundle_a / "delivery-manifest.json").is_file():
        record("generate-delivery", False, "交付根缺 delivery-manifest.json")
        return 1
    manifest = json.loads((bundle_a / "delivery-manifest.json").read_text(encoding="utf-8"))
    app_pkg = manifest["app"]["package"]
    record(
        "generate-delivery",
        True,
        f"app={app_pkg}, crates={len(manifest['crates'])}, "
        f"资源={len(manifest['resources'])}, pins={len(manifest['pins'])}, "
        f"多槽锁冻结={manifest['pins_multi_slot_lock_frozen']}, "
        f"规模={manifest['disclosure']['total_bytes'] / 1048576:.1f}MiB",
    )

    # ================= V01：全新解析 + 真实 build =================
    # 原失败（对照）：779/785 归档——独立 fixture 全新 UI 锁解析出
    # 双 windows-core 0.57.0/0.62.2 → wgpu-hal 宏边界 E0277，UI 外部证明
    # 只能借 workspace 锁定 rlib 直编。本轮闭环：交付根自带消费 patch +
    # pin 家族，fresh resolve 直接成功并完整构建。
    lock = bundle_a / "Cargo.lock"
    if lock.exists():
        lock.unlink()
    t0 = time.time()
    r = run(["cargo", "generate-lockfile"], bundle_a, consumer_env(), timeout=1200)
    if r.returncode != 0:
        record("V01-fresh-resolve", False, r.stderr[-1200:])
        return 1
    lock_text = lock.read_text(encoding="utf-8")
    checks = []
    if f'name = "iced_widget"' not in lock_text:
        checks.append("锁缺 iced_widget")
    # patch 命中：iced_widget 条目无 registry source（path 补丁）
    import re

    m = re.search(r'\[\[package\]\]\nname = "iced_widget"\nversion = "[^"]+"\n(source = "[^"]+")?', lock_text)
    if not m:
        checks.append("iced_widget 条目形态异常")
    elif m.group(1):
        checks.append(f"iced_widget 非 patch 单源: {m.group(1)}")
    pin_hits = sum(1 for name, ver in manifest["pins"].items() if f'version = "{ver.lstrip("=")}"' in lock_text)
    if pin_hits != len(manifest["pins"]):
        checks.append(f"pin 命中 {pin_hits}/{len(manifest['pins'])}")
    record("V01-fresh-resolve", not checks, "; ".join(checks) or f"fresh lock 生成 ({time.time() - t0:.0f}s)，patch 单源 + pin 全命中")

    t0 = time.time()
    r = run(["cargo", "build", "--locked"], bundle_a, consumer_env(), timeout=4800)
    exe = bundle_a / "target" / "debug" / f"{app_pkg}.exe"
    record(
        "V01-real-build",
        r.returncode == 0 and exe.is_file(),
        f"cargo build --locked ({time.time() - t0:.0f}s)，产物 {'在位' if exe.is_file() else '缺失'}"
        + ("" if r.returncode == 0 else " stderr: " + r.stderr[-600:]),
    )
    if r.returncode != 0:
        return 1

    # ================= V02：重定位 + locked 重放 =================
    bundle_b = scratch / "consumer-b"
    copytree_no_target(bundle_a, bundle_b)
    t0 = time.time()
    r = run(["cargo", "build", "--locked"], bundle_b, consumer_env(), timeout=4800)
    ok = r.returncode == 0 and (bundle_b / "target" / "debug" / f"{app_pkg}.exe").is_file()
    local_target = (bundle_b / "target").is_dir()
    record(
        "V02-relocated-locked-build",
        ok and local_target,
        f"B 内 locked build ({time.time() - t0:.0f}s)，本地 target={'有' if local_target else '无'}（无共享 target 回退）",
    )
    if not ok:
        return 1

    # metadata：所有本地 path source 均在 B 内；source 唯一
    r = run(
        ["cargo", "metadata", "--format-version", "1"],
        bundle_b, consumer_env(), timeout=600,
    )
    meta = json.loads(r.stdout)
    # R1-F03：真实路径包含关系——所有 path 依赖（source 为空）的
    # manifest_path 必须在 B 内（不再有盘符前缀过滤/包名特例）。
    outside = [
        p["name"]
        for p in meta["packages"]
        if p.get("source") is None
        and p["name"] != app_pkg
        and norm_lower(bundle_b) not in norm_lower(p["manifest_path"])
    ]
    iced_w = [p for p in meta["packages"] if p["name"] == "iced_widget"]
    single_source = (
        len(iced_w) == 1
        and norm_lower(bundle_b) in norm_lower(iced_w[0]["manifest_path"])
    )
    record(
        "V02-metadata-identity",
        single_source and not outside,
        f"iced_widget 单源@B内={single_source}，越界 path 依赖={outside or '无'}",
    )

    # 资源与 producer 同源（逐字节；hash 等价由 delivery-manifest 记录）
    ok, detail = verify_delivery(bundle_b)
    record("V02-resource-identity", ok, detail)

    # ================= V03：负控（确定性红 + 恢复复绿） =================
    # (a) 资源负控四态——全部经随包正式入口 verify.py（R1-F01：消费清单
    #     sha256，无 producer 也可执行）判红：
    #     ① 等长篡改资源字节（防大小校验漏检）
    tamper = scratch / "consumer-bad-resource"
    copytree_no_target(bundle_a, tamper)
    tampered = bytearray((tamper / "assets" / "avatar.png").read_bytes())
    tampered[len(tampered) // 2] ^= 0xFF
    (tamper / "assets" / "avatar.png").write_bytes(bytes(tampered))
    rv = run_verify_entry(tamper)
    record(
        "V03a-eqlen-tamper-red",
        rv.returncode != 0 and "hash 不符" in (rv.stdout + rv.stderr),
        f"exit={rv.returncode}，诊断含 hash 不符",
    )
    #     ② 清单坏 hash（记录值篡改——R1 反例：64 个 0）
    tamper2 = scratch / "consumer-bad-hash"
    copytree_no_target(bundle_a, tamper2)
    m2 = json.loads((tamper2 / "delivery-manifest.json").read_text(encoding="utf-8"))
    m2["resources"][0]["sha256"] = "0" * 64
    (tamper2 / "delivery-manifest.json").write_text(
        json.dumps(m2, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    rv = run_verify_entry(tamper2)
    record(
        "V03a-corrupt-hash-red",
        rv.returncode != 0,
        f"exit={rv.returncode}（记录 hash 与实际不符——判红）",
    )
    #     ③ 资源缺失
    (tamper / "assets" / "avatar.png").unlink()
    rv = run_verify_entry(tamper)
    record(
        "V03a-missing-resource-red",
        rv.returncode != 0 and "资源缺失" in (rv.stdout + rv.stderr),
        f"exit={rv.returncode}，诊断含资源缺失",
    )
    #     ④ 未登记资源（清空 allowlist 逃逸的对偶面：文件在而清单不认）
    (tamper2 / "assets" / "unlisted.png").write_bytes(b"\x89PNG-unlisted")
    rv = run_verify_entry(tamper2)
    record(
        "V03a-unlisted-resource-red",
        rv.returncode != 0 and "未登记资源" in (rv.stdout + rv.stderr),
        f"exit={rv.returncode}，诊断含未登记资源",
    )

    # (b) 消费根漏 patch：--locked 确定性红（lock 以 patch 身份冻结）
    nopatch = scratch / "consumer-nopatch"
    copytree_no_target(bundle_a, nopatch)
    root_m = nopatch / "Cargo.toml"
    text = root_m.read_text(encoding="utf-8")
    idx = text.index("[patch.crates-io]")
    root_m.write_text(text[:idx].rstrip() + "\n", encoding="utf-8")
    r = run(["cargo", "build", "--locked"], nopatch, consumer_env(), timeout=600)
    record(
        "V03b-missing-patch-red",
        r.returncode != 0,
        f"exit={r.returncode}" + ("" if r.returncode == 0 else "（锁需更新被拒——确定性红）"),
    )

    # (c) 显式不兼容输入：pin 改为冲突版本 → --locked 红
    badpin = scratch / "consumer-badpin"
    copytree_no_target(bundle_a, badpin)
    app_m = badpin / "app" / "Cargo.toml"
    lines = []
    for line in app_m.read_text(encoding="utf-8").splitlines():
        if line.startswith("wgpu-hal = "):
            lines.append('wgpu-hal = { version = "=0.0.1", default-features = false }')
        else:
            lines.append(line)
    app_m.write_text("\n".join(lines) + "\n", encoding="utf-8")
    r = run(["cargo", "build", "--locked"], badpin, consumer_env(), timeout=600)
    record(
        "V03c-incompatible-pin-red",
        r.returncode != 0,
        f"exit={r.returncode}（不兼容 pin 与冻结锁冲突——确定性红）",
    )

    # (d) 外来目录显式拒绝（生成器守卫，经 CLI 复放）
    foreign = scratch / "foreign-dir"
    foreign.mkdir()
    (foreign / "mine.txt").write_text("keep")
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(foreign)],
        gen_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    record(
        "V03d-foreign-dir-rejected",
        r.returncode != 0 and (foreign / "mine.txt").is_file(),
        f"exit={r.returncode}，外来文件未被覆盖",
    )

    # (e) R1-F02：再交付准备失败保留旧交付——既有成功交付（含 sentinel），
    #     producer 破损（闭包 crate 缺 manifest）→ 显式 Err 且旧字节不动；
    #     producer 恢复 → 再交付成功（换装）。
    old_target = scratch / "consumer-redeliver"
    copytree_no_target(bundle_a, old_target)
    sentinel = old_target / "app" / "previous-build.txt"
    sentinel.write_text("old delivery must survive failed preparation")
    manifest_before = (old_target / "delivery-manifest.json").read_text(encoding="utf-8")
    broken_producer = scratch / "broken-producer"
    (broken_producer / "crates" / "auto-lang").mkdir(parents=True)
    (broken_producer / "Cargo.toml").write_text("[workspace]\n")
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(old_target)],
        gen_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(broken_producer)}),
    )
    preserved = (
        r.returncode != 0
        and sentinel.is_file()
        and (old_target / "delivery-manifest.json").read_text(encoding="utf-8")
        == manifest_before
    )
    record(
        "V03e-redelivery-failure-preserves-old",
        preserved,
        f"exit={r.returncode}，sentinel/清单字节保留={sentinel.is_file()}",
    )
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(old_target)],
        gen_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    record(
        "V03e-redelivery-after-restore",
        r.returncode == 0 and (old_target / "delivery-manifest.json").is_file(),
        "producer 恢复后再交付成功（staging 换装）",
    )

    # (f) R1-F03：sidecar 外部绝对 path（复放审查反例——与 producer 不同根
    #     的主检出绝对路径）→ 生成显式拒绝，不产出应用清单。
    sidecar_proj = scratch / "sidecar-proj"
    shutil.copytree(PROJ, sidecar_proj)
    with (sidecar_proj / "pac.at").open("a") as f:
        f.write(
            '\nrust_sidecar {\n    deps: ["auto-val:path:D:/autostack/auto-lang/crates/auto-val"]\n}\n'
        )
    sidecar_bundle = scratch / "consumer-sidecar"
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(sidecar_bundle)],
        sidecar_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    emitted = sidecar_bundle / "app" / "Cargo.toml"
    record(
        "V03f-sidecar-external-path-rejected",
        r.returncode != 0 and not emitted.is_file(),
        f"exit={r.returncode}，未产出应用清单（外部 path 不入交付物）",
    )

    # 正常输入恢复复绿：A/B 未受负控影响
    ok, detail = verify_delivery(bundle_a)
    record("V03-restore-green", ok, f"A 包校验 {detail}；B 已绿（V02）")

    # ================= V04：开发模式保持 =================
    dev_proj = scratch / "dev-run"
    shutil.copytree(PROJ, dev_proj)
    r = run([str(auto_exe), "gen"], dev_proj, consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}), timeout=600)
    member = dev_proj / "rust-workspace" / "dev-run"
    member_cargo = member / "Cargo.toml"
    ws_cargo = dev_proj / "rust-workspace" / "Cargo.toml"
    dev_shape = (
        r.returncode == 0
        and member_cargo.is_file()
        and "auto-lang.workspace = true" in member_cargo.read_text(encoding="utf-8")
        and ws_cargo.is_file()
        and 'auto-lang = { path = "' in ws_cargo.read_text(encoding="utf-8")
    )
    record(
        "V04-dev-mode-preserved",
        dev_shape,
        "仓外项目 → project-local rust-workspace + workspace 继承清单（路由不变）",
    )
    # main.rs 与交付产物同源（同一编译相位；模板输出保持）
    dev_main = member / "src" / "main.rs"
    del_main = bundle_a / "app" / "src" / "main.rs"
    same = (
        dev_main.is_file()
        and del_main.is_file()
        and dev_main.read_bytes() == del_main.read_bytes()
    )
    record(
        "V04-template-output-identity",
        same,
        "dev/交付 main.rs 逐字节同源；UI 类型跨面编译由 V01/V02 真实 build 证明",
    )

    receipt["results"] = RESULTS
    receipt["scratch"] = str(scratch)
    receipt_path = scratch / "driver-receipt.json"
    receipt_path.write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    failed = [x for x in RESULTS if not x["ok"]]
    log(f"==== {len(RESULTS) - len(failed)}/{len(RESULTS)} 通过；收据 → {receipt_path}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
