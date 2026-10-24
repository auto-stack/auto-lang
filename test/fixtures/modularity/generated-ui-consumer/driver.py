#!/usr/bin/env python3
"""PLAN-792 generated-ui-consumer 交付验收 driver（V01..V05）。

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
  V05  随包新 tool 发行面（PLAN-815 F-815-T07-B2-R1-01）：交付包 A/重定位
       包 B 内 fresh/locked 构建 auto-ui-tools；B 的 auto-ui-tool.exe 以
       framed I-TOOL v1 真实外部消费（describe/validate/generate + shadcn
       资源物化字节=包内源）；未知版本/不服务 profile 零副作用拒绝、坏帧
       流致命退出

用法：
  python test/fixtures/modularity/generated-ui-consumer/driver.py \
      [--scratch <dir>] [--skip-build-auto]

退出码 0 = 全部通过；非 0 = 首个失败步（含负控未按预期红）。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import struct
import subprocess
import sys
import threading
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


def path_within(child, base) -> bool:
    """规范化路径的**组件包含**（R2：字符串子串会被 consumer-b-elsewhere
    等相似前缀欺骗；驱动器/分隔符/大小写归一后逐组件比较）。"""
    c = [x for x in norm_lower(child).split("/") if x not in ("", ".")]
    b = [x for x in norm_lower(base).split("/") if x not in ("", ".")]
    return len(c) >= len(b) and c[: len(b)] == b


def run_verify_entry(bundle: Path) -> subprocess.CompletedProcess:
    """R1-F01：随包受管校验正式入口 verify.py（消费清单 sha256，独立可执行）。"""
    return subprocess.run(
        [sys.executable, str(bundle / "verify.py")],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120,
    )


# ---- framed I-TOOL v1 client（PLAN-815 T-01：u32 大端长度前缀 + UTF-8 JSON） ----
# V05 外部消费面：不链接任何实现 crate，仅按已公布 wire 合同驱动随包
# auto-ui-tool 的 stdio。请求载荷形状镜像 auto-project-model wire v1 的
# `request_fields_to_wire`（严格解码拒绝未知/缺失键——此处给出全集，
# null = 未声明）。


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def tool_wire_request(project_root: Path, name: str, **overrides) -> dict:
    request = {
        "origin": {
            "kind": "static",
            "source": {"path": (project_root / "pac.at").as_posix()},
            "unsupported": [],
        },
        "project_root": project_root.as_posix(),
        "identity": {"name": name, "version": None, "lang": None},
        "display": {
            k: None
            for k in (
                "title", "title_zh", "icon", "category", "opens", "theme",
                "accent", "desktop_render", "daemon", "desktop", "desktop_exe",
                "ui_config",
            )
        },
        "runtime": {
            "api": None, "scene": None, "front_port": None, "back_port": None,
            "media_root": None, "photo_root": None,
            "window": {"kind": "fit"},
        },
        "scene": "ui",
        "members": [],
        "render": {
            "render": {"kind": "single", "name": "vue"},
            "legacy_backend": None,
            "effective_backend": None,
        },
        "external_backend": None,
        "toggles": {
            "shadcn": {"kind": "unset"},
            "default_classes": {"kind": "unset"},
        },
        "theme_decl": None,
        "dep_roots": [],
        "package_assets": {"npm_deps": None, "styles": None, "i18n": None},
        "exe_name": None,
        "has_rust_sidecar": False,
        "front_decl": None,
        "front_node_form": "./ui/front",
        "input_sources": [],
        "output_root": None,
        "mode": "generate",
        "raw": [],
        "raw_nodes": [],
    }
    request.update(overrides)
    return {"wire": 1, "request": request}


def tool_send(proc: subprocess.Popen, envelope: dict) -> None:
    payload = json.dumps(envelope, ensure_ascii=False).encode("utf-8")
    proc.stdin.write(struct.pack(">I", len(payload)) + payload)
    proc.stdin.flush()


def tool_read_exact(stream, count: int) -> bytes:
    buffer = b""
    while len(buffer) < count:
        chunk = stream.read(count - len(buffer))
        if not chunk:
            raise EOFError("tool closed the control stream")
        buffer += chunk
    return buffer


def tool_next_response(proc: subprocess.Popen) -> dict:
    (length,) = struct.unpack(">I", tool_read_exact(proc.stdout, 4))
    return json.loads(tool_read_exact(proc.stdout, length))


def tool_roundtrip(proc: subprocess.Popen, envelope: dict) -> dict:
    """发送并等待同 id 的终态回执（progress/launched 事件先行跳过）。"""
    tool_send(proc, envelope)
    while True:
        response = tool_next_response(proc)
        if response.get("body", {}).get("type") == "event":
            continue
        if response.get("id") == envelope["id"]:
            return response


def tool_frame_collector(proc: subprocess.Popen, sink: list, stop: threading.Event) -> None:
    """后台收集控制流全部帧（事件+终态+cancel 回执）到 sink。

    F831-09-03 REOPEN 修复的观测通道：异步操作（run_ui/build_ui）的进度
    事件与终态必须被验证器消费——"进程存活"不再是运行/安装证据
    （review-r1 idle/framed 假载体反例）。EOF/流关闭即退出；stop 仅在
    帧间隙生效（阻塞读由树杀/关流解除，线程为 daemon）。"""
    try:
        while True:
            (length,) = struct.unpack(">I", tool_read_exact(proc.stdout, 4))
            sink.append(json.loads(tool_read_exact(proc.stdout, length)))
            if stop.is_set():
                return
    except (EOFError, OSError, ValueError):
        return


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

    # PLAN-831 F831-09-05：三仓源身份入收据（此前缺失——fixture driver 的
    # 收据身份 admission 要求 lang/ui 记录且与当前 HEAD 一致方可采信）。
    # producer 二进制身份另由 delivered_tool.sha256 承载。
    def _git_head(repo: Path) -> str | None:
        out = subprocess.run(["git", "-C", str(repo), "rev-parse", "HEAD"],
                             capture_output=True, text=True)
        return out.stdout.strip() if out.returncode == 0 else None

    _ui_root = WT_ROOT.parent / "auto-ui"
    _down_root = WT_ROOT.parent / "auto-down"
    receipt["sources"] = {
        "lang": _git_head(WT_ROOT),
        "ui": _git_head(_ui_root) if _ui_root.is_dir() else None,
        "down": _git_head(_down_root) if _down_root.is_dir() else None,
    }
    log(f"sources: { {k: (v or '?')[:9] for k, v in receipt['sources'].items()} }")

    # ---- 生成器（正式入口）：auto CLI + UI 工具宿主 bin 构建 ----
    # PLAN-831 T-07：`auto gen` 走 framed I-TOOL 工具宿主子进程（auto-ui-tool
    # bin）；`cargo build -p auto` 只构建依赖 lib 不刷依赖 bin——工具 bin
    # 须显式同建，防陈旧宿主假红/假绿。
    auto_exe = WT_ROOT / "target" / "debug" / "auto.exe"
    if not args.skip_build_auto or not auto_exe.is_file():
        t0 = time.time()
        r = run(["cargo", "build", "-p", "auto"], WT_ROOT, consumer_env(), timeout=2400)
        if r.returncode != 0:
            record("build-auto-cli", False, r.stderr[-800:])
            return 1
        record("build-auto-cli", True, f"{time.time() - t0:.0f}s")
        t0 = time.time()
        r = run(["cargo", "build", "-p", "auto-ui-tools"], WT_ROOT, consumer_env(), timeout=2400)
        tool_exe = WT_ROOT / "target" / "debug" / "auto-ui-tool.exe"
        if r.returncode != 0 or not tool_exe.is_file():
            record("build-ui-tool-host", False, (r.stdout + r.stderr)[-800:])
            return 1
        record("build-ui-tool-host", True, f"{time.time() - t0:.0f}s")

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
    # R1-F03/R2：真实路径包含（组件级）——所有 path 依赖（source 为空）
    # 的 manifest_path 必须在 B 内（无盘符前缀过滤/包名特例/子串误判）。
    outside = [
        p["name"]
        for p in meta["packages"]
        if p.get("source") is None
        and p["name"] != app_pkg
        and not path_within(p["manifest_path"], bundle_b)
    ]
    iced_w = [p for p in meta["packages"] if p["name"] == "iced_widget"]
    single_source = (
        len(iced_w) == 1 and path_within(iced_w[0]["manifest_path"], bundle_b)
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

    # (g) R2-F02：Windows 根相对 path（/autostack/...——is_absolute 假但
    #     解析落当前盘根指向主检出）→ 同样显式拒绝。
    rootrel_proj = scratch / "rootrel-proj"
    shutil.copytree(PROJ, rootrel_proj)
    with (rootrel_proj / "pac.at").open("a") as f:
        f.write(
            '\nrust_sidecar {\n    deps: ["auto-val:path:/autostack/auto-lang/crates/auto-val"]\n}\n'
        )
    rootrel_bundle = scratch / "consumer-rootrel"
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(rootrel_bundle)],
        rootrel_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    emitted = rootrel_bundle / "app" / "Cargo.toml"
    diag = r.stdout + r.stderr
    record(
        "V03g-sidecar-root-relative-rejected",
        r.returncode != 0 and not emitted.is_file() and "根相对" in diag,
        f"exit={r.returncode}，诊断含根相对，未产出应用清单",
    )

    # (h) R2-F01/R3-F01：外来同级目录字节保留——固定名 staging 位与历史
    #     固定换出名上的外来内容不被删除/覆盖；backup 用唯一名后固定名
    #     不再被认领（再交付可成功且双方字节不动）。
    sib_base = scratch / "consumer-siblings"
    copytree_no_target(bundle_a, sib_base)
    legacy_stage = sib_base.parent / f".{sib_base.name}.delivery-staging"
    legacy_stage.mkdir(exist_ok=True)
    (legacy_stage / "mine.txt").write_text("unrelated sibling data")
    trash_name = sib_base.parent / f".{sib_base.name}.delivery-old"
    trash_name.mkdir(exist_ok=True)
    (trash_name / "mine.txt").write_text("unrelated sibling data")
    sib_proj = scratch / "siblings-run"
    shutil.copytree(PROJ, sib_proj)
    # 首交（delivery-root 新位置）：固定名 staging 位外来内容不动
    fresh_dest = scratch / "consumer-siblings-fresh"
    legacy_fresh = scratch / f".{fresh_dest.name}.delivery-staging"
    legacy_fresh.mkdir(exist_ok=True)
    (legacy_fresh / "mine.txt").write_text("unrelated sibling data")
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(fresh_dest)],
        sib_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    record(
        "V03h-foreign-staging-sibling-preserved",
        r.returncode == 0
        and (legacy_fresh / "mine.txt").read_text() == "unrelated sibling data",
        f"exit={r.returncode}，固定名 staging 位外来字节保留",
    )
    # 再交付（既有交付 + 历史固定换出名上有外来内容）：成功且外来字节不动
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(sib_base)],
        sib_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    record(
        "V03h-foreign-fixed-backup-sibling-preserved",
        r.returncode == 0
        and (trash_name / "mine.txt").read_text() == "unrelated sibling data",
        f"exit={r.returncode}，历史固定换出名外来字节保留（backup 唯一名不认领固定名）",
    )

    # (i) R3-F02 复放：sidecar 原样写入的单引号内联表 `{path='...'}`（合法
    #     TOML，旧文本匹配绕过）→ TOML 语义校验拒绝，零产出。
    raw_ext = scratch / "raw-external-crate"
    raw_ext.mkdir()
    (raw_ext / "Cargo.toml").write_text(
        '[package]\nname = "raw-review-external"\nversion = "0.1.0"\nedition = "2021"\n[workspace]\n'
    )
    raw_proj = scratch / "raw-sidecar-proj"
    shutil.copytree(PROJ, raw_proj)
    with (raw_proj / "pac.at").open("a") as f:
        f.write(
            f"\nrust_sidecar {{\n    deps: [\"raw-review-external:{{path='{raw_ext.as_posix()}'}}\"]\n}}\n"
        )
    raw_bundle = scratch / "consumer-rawtoml"
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(raw_bundle)],
        raw_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    diag = r.stdout + r.stderr
    record(
        "V03i-raw-toml-single-quote-rejected",
        r.returncode != 0
        and not (raw_bundle / "app" / "Cargo.toml").is_file()
        and "外部绝对/根相对" in diag,
        f"exit={r.returncode}，TOML 语义诊断在案，零产出",
    )

    # (j) R3-F01 复放：**另一应用的合法交付包**（带自己的
    #     delivery-manifest.json+用户数据）放在历史固定换出名上——再交付
    #     成功且该应用数据完整存活（marker 存在 ≠ 本次操作归属）。
    other_proj = scratch / "other-app-proj"
    shutil.copytree(PROJ, other_proj)
    (other_proj / "pac.at").write_text(
        'name: "unrelated_app"\nversion: "1.0.0"\nscene: "ui"\nrender: "rust"\nwindow: "fit"\n'
    )
    other_dest = scratch / "other-app-delivery"
    r = run(
        [str(auto_exe), "gen", "--delivery-root", str(other_dest)],
        other_proj,
        consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
    )
    if r.returncode != 0:
        record("V03j-other-app-delivery-survives", False, f"他应用交付生成失败 exit={r.returncode}")
    else:
        other_at_fixed = scratch / ".marked-backup.delivery-old"
        if other_at_fixed.exists():
            shutil.rmtree(other_at_fixed)
        shutil.move(str(other_dest), str(other_at_fixed))
        (other_at_fixed / "app").mkdir(exist_ok=True)
        (other_at_fixed / "app" / "unrelated-user-data.txt").write_text(
            "another application's valid delivery"
        )
        marked = scratch / "marked-backup"
        copytree_no_target(bundle_a, marked)
        r = run(
            [str(auto_exe), "gen", "--delivery-root", str(marked)],
            sib_proj,
            consumer_env({"AUTO_DELIVERY_SOURCE_ROOT": str(WT_ROOT)}),
        )
        record(
            "V03j-other-app-delivery-survives",
            r.returncode == 0
            and (other_at_fixed / "app" / "unrelated-user-data.txt").is_file()
            and (other_at_fixed / "delivery-manifest.json").is_file(),
            f"exit={r.returncode}，他应用交付与其用户数据在固定换出名上完整存活",
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
    # PLAN-814 T-05：canonical 双直依赖（import_policy.rs）——成员
    # runtime/core workspace 继承 + workspace path 注入。
    dev_shape = (
        r.returncode == 0
        and member_cargo.is_file()
        and "auto-ui-runtime.workspace = true" in member_cargo.read_text(encoding="utf-8")
        and "auto-lang-core.workspace = true" in member_cargo.read_text(encoding="utf-8")
        and ws_cargo.is_file()
        and 'auto-ui-runtime = { path = "' in ws_cargo.read_text(encoding="utf-8")
        and 'auto-lang-core = { path = "' in ws_cargo.read_text(encoding="utf-8")
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

    # ================= V05：随包新 tool 构建 + 真实外部消费 =================
    # F-815-T07-B2-R1-01（review-t07-batch2-r1）：旧 25 项只构建
    # default-member app（uiminimal），从未构建/启动/消费交付包中的
    # auto-ui-tool。本节在既有约定发行形态（源码包 workspace：members =
    # app + 26 受管源 crate）上扩面，不新增未约定安装布局：
    #   V05a/b 交付包 A（V01 fresh 锁）与重定位包 B 内 locked 构建
    #          auto-ui-tools —— 新 tool 从受管交付来源可建
    #   V05c..e B 的 auto-ui-tool.exe 真实外部消费（framed I-TOOL v1 over
    #          stdio，剥离开发根环境）：describe / validate_project /
    #          generate_ui 真实工程 + shadcn 组件从工具资源面物化、字节=
    #          包内 vendored 源（资源控制）
    #   V05f..h 失败控制：未知协议版本（零副作用、流仍服务）、不服务
    #          profile（副作用前拒绝）、坏帧（流致命非零退出）
    t0 = time.time()
    r = run(
        ["cargo", "build", "--locked", "-p", "auto-ui-tools"],
        bundle_a, consumer_env(), timeout=4800,
    )
    tool_a = bundle_a / "target" / "debug" / "auto-ui-tool.exe"
    record(
        "V05a-tool-fresh-locked-build",
        r.returncode == 0 and tool_a.is_file(),
        f"包 A 内 cargo build --locked -p auto-ui-tools（{time.time() - t0:.0f}s），"
        f"auto-ui-tool.exe {'在位' if tool_a.is_file() else '缺失'}"
        + ("" if r.returncode == 0 else " stderr: " + r.stderr[-600:]),
    )
    if r.returncode != 0:
        return 1

    t0 = time.time()
    r = run(
        ["cargo", "build", "--locked", "-p", "auto-ui-tools"],
        bundle_b, consumer_env(), timeout=4800,
    )
    tool_b = bundle_b / "target" / "debug" / "auto-ui-tool.exe"
    record(
        "V05b-tool-relocated-locked-build",
        r.returncode == 0 and tool_b.is_file(),
        f"重定位包 B 内 locked 构建（{time.time() - t0:.0f}s），"
        f"auto-ui-tool.exe {'在位' if tool_b.is_file() else '缺失'}"
        + ("" if r.returncode == 0 else " stderr: " + r.stderr[-600:]),
    )
    if not tool_b.is_file():
        return 1

    # 真实外部消费者工程（独立于 fixture；button 节点 → 转译产物 import
    # @/components/ui/button → 触发 shadcn 资源物化——embed 消费面）。
    tool_proj = scratch / "tool-consumer"
    (tool_proj / "ui" / "front").mkdir(parents=True)
    (tool_proj / "pac.at").write_text(
        'name: "tool-consumer"\nversion: "1.0.0"\nscene: "ui"\nrender: "vue"\n',
        encoding="utf-8",
    )
    (tool_proj / "ui" / "front" / "app.at").write_text(
        "widget App {\n"
        "    view {\n"
        "        col {\n"
        "            button {\n"
        "                text \"交付工具资源面\" { style: \"text-sm font-medium\" }\n"
        "            }\n"
        "        }\n"
        "    }\n"
        "}\n",
        encoding="utf-8",
    )

    tool_stderr_path = scratch / "delivered-tool-stderr.log"
    tool_log = open(tool_stderr_path, "wb")
    tool = subprocess.Popen(
        [str(tool_b)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=tool_log,
        env=consumer_env(),
    )
    # 看门狗：整个会话有界（卡死则击杀，阻塞读以 EOF 失败并记 FAIL）。
    def _tool_watchdog() -> None:
        time.sleep(600)
        if tool.poll() is None:
            tool.kill()
    threading.Thread(target=_tool_watchdog, daemon=True).start()

    try:
        # (c) describe：真实身份与操作集
        resp = tool_roundtrip(tool, {"protocol_version": 1, "id": 101, "body": {"op": "describe"}})
        desc = resp.get("body", {}).get("descriptor", {})
        ops = {c.get("operation") for c in desc.get("capabilities", [])}
        record(
            "V05c-tool-describe",
            resp.get("body", {}).get("type") == "describe_ok"
            and desc.get("tool_name") == "auto-ui-tool"
            and {"describe", "validate_project", "generate_ui"} <= ops,
            f"name={desc.get('tool_name')} version={desc.get('tool_version')} "
            f"protocol={desc.get('protocol_version')} "
            f"operations={sorted(str(o) for o in ops)}",
        )

        # (d) validate_project：合法请求 → ok
        resp = tool_roundtrip(
            tool,
            {"protocol_version": 1, "id": 102,
             "body": {"op": "validate_project", "request": tool_wire_request(tool_proj, "tool-consumer")}},
        )
        report = resp.get("body", {}).get("report", {})
        record(
            "V05d-tool-validate-project",
            resp.get("body", {}).get("type") == "validate_ok" and report.get("ok") is True,
            f"validate_ok ok={report.get('ok')} findings={report.get('findings')}",
        )

        # (e) generate_ui：真实生成 + 资源控制（embed 面物化字节=包内源）
        resp = tool_roundtrip(
            tool,
            {"protocol_version": 1, "id": 103,
             "body": {"op": "generate_ui", "request": tool_wire_request(tool_proj, "tool-consumer")}},
        )
        gen_report = resp.get("body", {}).get("report", {})
        vue_root = tool_proj / "gen" / "front" / "vue"
        package_json = vue_root / "package.json"
        button_dir = vue_root / "src" / "components" / "ui" / "button"
        button_src = bundle_b / "crates" / "auto-ui-tools" / "assets" / "shadcn-ui" / "button"
        materialized = sorted(p.relative_to(button_dir).as_posix() for p in button_dir.rglob("*") if p.is_file())
        byte_mismatch = [
            rel for rel in materialized
            if (button_dir / rel).read_bytes() != (button_src / rel).read_bytes()
        ]
        record(
            "V05e-tool-generate-real",
            resp.get("body", {}).get("type") == "generate_ok"
            and gen_report.get("frontends") == ["vue"]
            and isinstance(gen_report.get("files_written"), int)
            and gen_report["files_written"] > 0
            and package_json.is_file()
            and materialized
            and not byte_mismatch,
            f"generate_ok frontends={gen_report.get('frontends')} "
            f"files_written={gen_report.get('files_written')}；"
            f"shadcn button 物化 {len(materialized)} 文件"
            f"（字节=包内 vendored 源，mismatch={byte_mismatch or '无'}）",
        )

        # (f) 未知协议版本：具名拒绝、零副作用、流仍服务（后续 describe 可答）
        resp = tool_roundtrip(tool, {"protocol_version": 99, "id": 104, "body": {"op": "describe"}})
        kind = resp.get("body", {}).get("error", {}).get("kind", {})
        followup = tool_roundtrip(tool, {"protocol_version": 1, "id": 105, "body": {"op": "describe"}})
        record(
            "V05f-tool-unknown-version-rejected",
            resp.get("body", {}).get("type") == "error"
            and kind.get("kind") == "unknown_protocol_version"
            and kind.get("client") == 99
            and followup.get("body", {}).get("type") == "describe_ok",
            f"error kind=unknown_protocol_version client=99；"
            f"随后 describe 同流可答（{followup.get('body', {}).get('type')}）",
        )

        # (g) 不服务 profile：副作用前具名拒绝（unserved 检查先于验证/落盘）
        unserved_root = scratch / "tool-unserved"
        unserved_root.mkdir()
        bad_render = {"render": {"render": {"kind": "single", "name": "plan9"},
                                 "legacy_backend": None, "effective_backend": None}}
        resp = tool_roundtrip(
            tool,
            {"protocol_version": 1, "id": 106,
             "body": {"op": "generate_ui",
                      "request": tool_wire_request(unserved_root, "tool-unserved", **bad_render)}},
        )
        error = resp.get("body", {}).get("error", {})
        record(
            "V05g-tool-unserved-profile-zero-side-effect",
            resp.get("body", {}).get("type") == "error"
            and error.get("kind", {}).get("kind") == "invalid_field"
            and not (unserved_root / "gen").exists(),
            f"error kind=invalid_field reason={error.get('kind', {}).get('reason')}"
            f"；消费根零生成输出={not (unserved_root / 'gen').exists()}",
        )

        # (h) 坏帧：流致命（非零退出）——帧内非 JSON 载荷
        garbage = b"this is not a json payload at all"
        tool.stdin.write(struct.pack(">I", len(garbage)) + garbage)
        tool.stdin.flush()
        tool.stdin.close()
        code = tool.wait(timeout=120)
        record(
            "V05h-tool-bad-frame-stream-fatal",
            code != 0,
            f"exit={code}（坏帧流致命，非零退出）",
        )
    except (EOFError, OSError, ValueError, subprocess.TimeoutExpired) as exc:
        record("V05-tool-session", False, f"外部消费会话异常: {exc!r}")
    finally:
        if tool.poll() is None:
            tool.kill()
            tool.wait()
        tool_log.close()
        receipt["delivered_tool"] = {
            "binary": str(tool_b),
            "sha256": sha256_file(tool_b),
            "stderr_log": str(tool_stderr_path),
            "source_crate": "crates/auto-ui-tools (bundle B, relocated)",
        }

    # ================= V06：F831-09-03 真实 run + 支持矩阵 profile 面 =================
    # 迁移后发行验收补面：①生成 Rust 应用真实 run（Win32 主窗口句柄可观察
    # + 受控终止退出可观测）；②generate 支持矩阵 6 profile 逐项真实生成
    # （工具侧确定性）；③build 支持矩阵 5 profile 逐项真实构建（vm=解释
    # 形态无 build 步，不在矩阵；环境缺席=具名红，不默认不适用）；④真实
    # 安装中取消（vue pnpm 链中途杀工具进程树）+ 重试恢复；⑤vm profile
    # 经 run_ui 真实运行 + 协议 cancel 取消（run 级取消可观测）。
    # 其余 run 面（vue/jet/ark dev 服务器、tauri CLI、vscode）属宿主域：
    # build 矩阵承载其真实链路，run 缺席逐项具名记录于收据 dispositions。
    def spawn_tool() -> subprocess.Popen:
        logf = open(scratch / "delivered-tool-v06-stderr.log", "ab")
        return subprocess.Popen(
            [str(tool_b)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=logf, env=consumer_env(),
        )

    def _rm_rf(target: Path) -> None:
        def _onrm(func, path, _exc):
            try:
                os.chmod(path, 0o700 | 0o200)
            except OSError:
                pass
            try:
                func(path)
            except OSError:
                pass
        if target.exists():
            shutil.rmtree(target, onexc=_onrm)

    def _drain(stream) -> None:
        try:
            for _ in iter(stream.readline, b""):
                pass
        except (OSError, ValueError):
            pass

    # ---- V06a：生成 Rust 应用真实 run ----
    t0 = time.time()
    app_exe = bundle_b / "target" / "debug" / f"{app_pkg}.exe"
    run_ok, run_detail = False, "产物缺席"
    if app_exe.is_file():
        proc = subprocess.Popen([str(app_exe)], cwd=str(bundle_b))
        time.sleep(4.0)  # iced 冷启动 + 首帧
        alive = proc.poll() is None
        ps = subprocess.run(
            ["powershell", "-NoProfile", "-Command",
             f"(Get-Process -Id {proc.pid} -ErrorAction SilentlyContinue).MainWindowHandle"],
            capture_output=True, text=True, timeout=30)
        hwnd = ps.stdout.strip()
        windowed = hwnd not in ("", "0")
        proc.terminate()
        try:
            code = proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            proc.kill()
            code = proc.wait(timeout=15)
        run_ok = alive and windowed and code is not None
        run_detail = (f"pid={proc.pid} 4s 存活={alive} 主窗口句柄={hwnd or '?'} "
                      f"终止退出码={code}（{time.time() - t0:.0f}s）")
    record("V06a-rust-app-real-run", run_ok, run_detail)

    # ---- V06b：generate 支持矩阵（6 profile 真实生成）----
    GEN_PROFILES = ("vue", "rust", "jet", "ark", "tauri", "vscode")
    BUILD_PROFILES = ("vue", "rust", "jet", "ark", "vscode")
    gen_results: list[tuple[str, bool, str]] = []
    tool6 = spawn_tool()
    try:
        for prof in GEN_PROFILES:
            proj = scratch / f"v06-gen-{prof}"
            if proj.exists():
                _rm_rf(proj)
            (proj / "ui" / "front").mkdir(parents=True)
            (proj / "pac.at").write_text(
                f'name: "v06-{prof}"\nversion: "1.0.0"\nscene: "ui"\nrender: "{prof}"\n',
                encoding="utf-8")
            (proj / "ui" / "front" / "app.at").write_text(
                'widget App {\n    view {\n        col {\n            button { text "v06" }\n        }\n    }\n}\n',
                encoding="utf-8")
            if prof == "tauri":
                # tauri 后端生成需要真实 src/back/{api,db}.at（无 back 的纯 UI
                # 工程不是 tauri 候选——工具具名拒绝是合法前置非缺陷）
                (proj / "src" / "back").mkdir(parents=True)
                (proj / "src" / "back" / "api.at").write_text(
                    '#[api(method = "GET", path = "/api/notes")]\n'
                    'pub fn list_notes() str {\n    return "[]"\n}\n',
                    encoding="utf-8")
                (proj / "src" / "back" / "db.at").write_text(
                    'type Connection {\n    url str\n}\n\n'
                    'fn connect() Connection {\n    Connection(url: "localhost")\n}\n',
                    encoding="utf-8")
            render_ov = {"render": {"render": {"kind": "single", "name": prof},
                                    "legacy_backend": None, "effective_backend": None}}
            try:
                resp = tool_roundtrip(tool6, {
                    "protocol_version": 1, "id": 200 + GEN_PROFILES.index(prof),
                    "body": {"op": "generate_ui",
                             "request": tool_wire_request(proj, f"v06-{prof}", **render_ov)}})
                body = resp.get("body", {})
                # profile 落盘形态：rust=共享 rust-workspace/（api-gen
                # ensure_shared_workspace）；tauri=双根（vue 壳 gen/front/vue +
                # rust 后端 gen/back/rust）；其余=gen/front/<prof>
                if prof == "rust":
                    roots = [proj / "rust-workspace"]
                elif prof == "tauri":
                    roots = [proj / "gen" / "front" / "vue", proj / "gen" / "back" / "rust"]
                else:
                    roots = [proj / "gen" / "front" / prof]
                files = [p for r in roots for p in r.rglob("*") if p.is_file()]                     if all(r.is_dir() for r in roots) else []
                marker = ((roots[0] / "Cargo.toml").is_file()
                          and any(roots[0].rglob("main.rs"))) if prof == "rust" else bool(files)
                err = body.get("error", {}).get("kind", {})
                ok = (body.get("type") == "generate_ok"
                      and body.get("report", {}).get("files_written", 0) > 0
                      and marker)
                gen_results.append((prof, ok,
                                    f"generate_ok={body.get('type') == 'generate_ok'} "
                                    f"files_written={body.get('report', {}).get('files_written')} "
                                    f"输出根实文件={len(files)}"
                                    + (f"；error={err.get('kind')}/{str(err.get('reason'))[:120]}"
                                       if err else "")))
            except (EOFError, OSError, ValueError, subprocess.TimeoutExpired) as exc:
                gen_results.append((prof, False, f"会话异常: {exc!r}"))
        record("V06b-generate-profile-matrix",
               all(ok for _, ok, _ in gen_results),
               "; ".join(f"{p}={'ok' if ok else 'RED'}({d})" for p, ok, d in gen_results))
    finally:
        if tool6.poll() is None:
            tool6.kill()
            tool6.wait()

    # ---- V06c：build 支持矩阵（5 profile 真实构建；环境缺席具名红）----
    # rust 的真实构建链已由 V01/V02 cargo build --locked 全量承载（同请求
    # 形态），此处经工具协议 run build_ui 复验一致性；vue/jet/ark/vscode
    # 走真实 npm/pnpm/cargo 链（工具宿主内建进程管理）。
    build_results: list[tuple[str, bool, str]] = []
    tool7 = spawn_tool()
    try:
        req_id = 300
        for prof in BUILD_PROFILES:
            proj = scratch / f"v06-gen-{prof}"
            render_ov = {"render": {"render": {"kind": "single", "name": prof},
                                    "legacy_backend": None, "effective_backend": None}}
            req_id += 1
            try:
                resp = tool_roundtrip(tool7, {
                    "protocol_version": 1, "id": req_id,
                    "body": {"op": "build_ui",
                             "request": tool_wire_request(proj, f"v06-{prof}", **render_ov)}})
                body = resp.get("body", {})
                rep = body.get("report", {}) or {}
                outcome = rep.get("outcome")
                err_text = str(rep.get("error"))[:200]
                # 协议 build_ok ≠ 真实构建成功：outcome==completed 才是通过；
                # 失败按错误文本具名（工具链缺席=环境阻塞具名红，非默认不适用）
                env_blocked = any(marker in err_text for marker in (
                    "9009", "not found", "wrapper not found", "open the project in"))
                ok = body.get("type") == "build_ok" and outcome == "completed"
                detail = (f"outcome={outcome}"
                          + (f"【环境阻塞】{err_text}" if env_blocked and not ok
                             else f" error={err_text}" if not ok else "（真实构建链完成）"))
            except (EOFError, OSError, ValueError, subprocess.TimeoutExpired) as exc:
                ok, detail = False, f"会话异常/超时: {exc!r}"
            build_results.append((prof, ok, detail))
        record("V06c-build-profile-matrix",
               all(ok for _, ok, _ in build_results),
               "; ".join(f"{p}={'ok' if ok else 'RED'}({d})" for p, ok, d in build_results))
    finally:
        if tool7.poll() is None:
            tool7.kill()
            tool7.wait()

    # ---- V06d：真实安装中取消 + 重试恢复（vue pnpm 链）----
    # F831-09-03 REOPEN 修复：mid_alive+重试完成不再足够——①须先观察到
    # 安装活动证据（操作进度事件 + node_modules 落盘）且终态未到（仍在
    # 安装中）才执行中断；立即回 completed 的假载体（从未安装）确定性红；
    # ②中断后验证陌生数据与输入字节保持（中断不得破坏非所属数据）；
    # ③重试全链 completed 且真实产物 dist/index.html 落盘才算复绿。
    cancel_proj = scratch / "v06-cancel"
    if cancel_proj.exists():
        _rm_rf(cancel_proj)
    (cancel_proj / "ui" / "front").mkdir(parents=True)
    (cancel_proj / "pac.at").write_text(
        'name: "v06-cancel"\nversion: "1.0.0"\nscene: "ui"\nrender: "vue"\n', encoding="utf-8")
    (cancel_proj / "ui" / "front" / "app.at").write_text(
        "widget App {\n    view {\n        col {\n            button { text \"cancel\" }\n        }\n    }\n}\n",
        encoding="utf-8")
    stranger_path = cancel_proj / "STRANGER-DATA.txt"
    stranger_bytes = b"stranger-owned v06d \x00\xd5 payload" * 16
    stranger_path.write_bytes(stranger_bytes)
    pac_bytes = (cancel_proj / "pac.at").read_bytes()
    app_bytes = (cancel_proj / "ui" / "front" / "app.at").read_bytes()
    install_dir = cancel_proj / "gen" / "front" / "vue" / "node_modules"
    cancel_ok, cancel_detail = False, "未执行"
    t0 = time.time()
    tool8 = spawn_tool()
    frames8: list[dict] = []
    stop8 = threading.Event()
    reader8 = threading.Thread(target=tool_frame_collector, args=(tool8, frames8, stop8),
                               daemon=True)
    reader8.start()
    try:
        req = {"protocol_version": 1, "id": 400,
               "body": {"op": "build_ui", "request": tool_wire_request(
                   cancel_proj, "v06-cancel",
                   render={"render": {"kind": "single", "name": "vue"},
                           "legacy_backend": None, "effective_backend": None})}}
        tool_send(tool8, req)
        # 安装活动观测窗：进度事件 + node_modules 落盘；终态先至=未进入
        # 真实安装链（假载体），立即判红不中断。
        act_events, act_disk, early_final = 0, False, None
        deadline = time.time() + 150.0
        while time.time() < deadline:
            act_events = sum(1 for f in frames8
                             if f.get("id") == 400 and f.get("body", {}).get("type") == "event")
            try:
                act_disk = install_dir.is_dir() and any(install_dir.iterdir())
            except OSError:
                act_disk = False
            finals = [f for f in frames8
                      if f.get("id") == 400 and f.get("body", {}).get("type") not in ("event",)]
            if finals:
                early_final = finals[0]
                break
            if act_events and act_disk:
                break
            if tool8.poll() is not None:
                break
            time.sleep(0.3)
        if early_final is not None:
            body8 = early_final.get("body", {})
            cancel_detail = (f"安装活动证据缺失即收到终态（type={body8.get('type')} "
                             f"outcome={(body8.get('report') or {}).get('outcome')}）——"
                             f"未经真实安装链，验证器拒绝采信")
        elif not (act_events and act_disk):
            cancel_detail = (f"观测窗内未见安装活动（事件={act_events} node_modules落盘="
                             f"{act_disk} 工具存活={tool8.poll() is None}）")
        else:
            subprocess.run(["taskkill", "/PID", str(tool8.pid), "/T", "/F"],
                           capture_output=True, timeout=60)
            killed = tool8.wait(timeout=60)
            # 中断后保持性：陌生数据与源输入字节不得被破坏（失败阶段保护）。
            stranger_kept = stranger_path.read_bytes() == stranger_bytes
            inputs_kept = ((cancel_proj / "pac.at").read_bytes() == pac_bytes
                           and (cancel_proj / "ui" / "front" / "app.at").read_bytes() == app_bytes)
            # 重试恢复：全新工具会话重发同一 build——受管流程可恢复至完成，
            # 且真实产物落盘（vite dist/index.html）才算真应用复绿。
            tool9 = spawn_tool()
            try:
                resp = tool_roundtrip(tool9, {
                    "protocol_version": 1, "id": 401,
                    "body": {"op": "build_ui", "request": tool_wire_request(
                        cancel_proj, "v06-cancel",
                        render={"render": {"kind": "single", "name": "vue"},
                                "legacy_backend": None, "effective_backend": None})}})
                retry_rep = resp.get("body", {}).get("report", {}) or {}
                retry_ok = (resp.get("body", {}).get("type") == "build_ok"
                            and retry_rep.get("outcome") == "completed")
            except (EOFError, OSError, ValueError, subprocess.TimeoutExpired):
                retry_ok = False
            finally:
                if tool9.poll() is None:
                    tool9.kill()
                    tool9.wait()
            dist_html = cancel_proj / "gen" / "front" / "vue" / "dist" / "index.html"
            dist_ok = dist_html.is_file() and dist_html.stat().st_size > 0
            stranger_kept2 = stranger_path.read_bytes() == stranger_bytes
            cancel_ok = act_events > 0 and act_disk and stranger_kept and inputs_kept \
                and retry_ok and dist_ok and stranger_kept2
            cancel_detail = (f"安装活动=事件{act_events}/node_modules落盘，仍在跑时树杀退出={killed}；"
                             f"中断后陌生数据保持={stranger_kept} 输入字节保持={inputs_kept}；"
                             f"重试completed={retry_ok} dist/index.html落盘={dist_ok} "
                             f"复绿后陌生数据保持={stranger_kept2}（{time.time() - t0:.0f}s）")
        if tool8.poll() is None:
            tool8.kill()
            tool8.wait()
        stop8.set()
    except (EOFError, OSError, ValueError, subprocess.TimeoutExpired) as exc:
        cancel_detail = f"会话异常: {exc!r}"
        if tool8.poll() is None:
            tool8.kill()
            tool8.wait()
        stop8.set()
    record("V06d-install-cancel-retry", cancel_ok, cancel_detail)

    # ---- V06e：vm profile 经 run_ui 真实运行 + 协议 cancel ----
    # F831-09-03 REOPEN 修复：mid_alive 不再是证据——run 接受且实际运行须由
    # 协议观测（run 操作的进度事件，如 resolve backend；无事件假载体确定性
    # 红）；取消走协议 cancel op（非树杀），期望 cancelled 回执 + owned 树
    # 回收后才发出的终态 run_ok(outcome=cancelled) + cancel 后宿主会话仍可
    # 响应（describe 复验）——构成 owned 进程/reader 资源收口的协议观测。
    vm_ok, vm_detail = False, "未执行"
    vm_proj = scratch / "v06-vm"
    if vm_proj.exists():
        _rm_rf(vm_proj)
    (vm_proj / "ui" / "front").mkdir(parents=True)
    (vm_proj / "pac.at").write_text(
        'name: "v06-vm"\nversion: "1.0.0"\nscene: "ui"\nrender: "vm"\n', encoding="utf-8")
    (vm_proj / "ui" / "front" / "app.at").write_text(
        "widget App {\n    view {\n        col {\n            text \"vm run\"\n        }\n    }\n}\n",
        encoding="utf-8")
    t0 = time.time()
    toolA = spawn_tool()
    framesA: list[dict] = []
    stopA = threading.Event()
    readerA = threading.Thread(target=tool_frame_collector, args=(toolA, framesA, stopA),
                               daemon=True)
    readerA.start()
    try:
        req = {"protocol_version": 1, "id": 500,
               "body": {"op": "run_ui", "request": tool_wire_request(
                   vm_proj, "v06-vm",
                   render={"render": {"kind": "single", "name": "vm"},
                           "legacy_backend": None, "effective_backend": None})}}
        tool_send(toolA, req)
        # 运行接受观测窗：run 操作的事件（进度）证明已进入真实运行链。
        run_events: list[str] = []
        deadline = time.time() + 30.0
        while time.time() < deadline:
            run_events = [str((f.get("body", {}).get("event") or {}).get("message"))
                          for f in framesA
                          if f.get("id") == 500 and f.get("body", {}).get("type") == "event"]
            if run_events:
                break
            if any(f.get("id") == 500 and f.get("body", {}).get("type") != "event"
                   for f in framesA):
                break  # 终态先至：run 未驻留即结束（非运行态）
            if toolA.poll() is not None:
                break
            time.sleep(0.2)
        accepted = bool(run_events)
        if not accepted:
            finalsA = [f.get("body", {}).get("type") for f in framesA
                       if f.get("id") == 500 and f.get("body", {}).get("type") != "event"]
            vm_detail = (f"run_ui(vm) 观测窗内无操作事件（帧数={len(framesA)} 终态="
                         f"{finalsA[:1]} 工具存活={toolA.poll() is None}）——"
                         f"run 未被证实接受/运行")
        else:
            tool_send(toolA, {"protocol_version": 1, "id": 501,
                              "body": {"op": "cancel", "target_id": 500}})
            cancel_ack, final = False, None
            deadline2 = time.time() + 60.0
            while time.time() < deadline2:
                cancel_ack = cancel_ack or any(
                    f.get("id") == 501 and f.get("body", {}).get("type") == "cancelled"
                    and f.get("body", {}).get("target_id") == 500 for f in framesA)
                finals = [f for f in framesA
                          if f.get("id") == 500 and f.get("body", {}).get("type") == "run_ok"]
                if finals:
                    final = finals[0]
                    break
                if toolA.poll() is not None:
                    break
                time.sleep(0.2)
            outcome = ((final or {}).get("body", {}).get("report", {}) or {}).get("outcome")
            # 宿主存续复验：describe 亦走 framesA 单读者通道——collector 线程是
            # toolA.stdout 的唯一读者，禁止再对同进程做阻塞 roundtrip（双读者
            # 会互抢帧，describe_ok 被 collector 消费后 roundtrip 永久阻塞——
            # 真实链首轮整跑实测卡死，假载体反例走不到此步故重放不可见）。
            session_ok = False
            if toolA.poll() is None:
                tool_send(toolA, {"protocol_version": 1, "id": 502,
                                  "body": {"op": "describe"}})
                deadline3 = time.time() + 15.0
                while time.time() < deadline3:
                    session_ok = any(
                        f.get("id") == 502 and f.get("body", {}).get("type") == "describe_ok"
                        for f in framesA)
                    if session_ok or toolA.poll() is not None:
                        break
                    time.sleep(0.2)
            vm_ok = accepted and cancel_ack and outcome == "cancelled" and session_ok
            vm_detail = (f"run_ui(vm) 事件={run_events[:3]}；协议cancel回执={cancel_ack}；"
                         f"终态run_ok outcome={outcome}（owned 树回收后发出）；"
                         f"cancel后宿主describe复验={session_ok}（{time.time() - t0:.0f}s）")
    except (EOFError, OSError, ValueError, subprocess.TimeoutExpired) as exc:
        vm_detail = f"会话异常: {exc!r}"
    finally:
        if toolA.poll() is None:
            toolA.kill()
            toolA.wait()
        stopA.set()
    record("V06e-vm-run-ui-real", vm_ok, vm_detail)

    receipt["v06_dispositions"] = {
        "run_faces": {
            "rust": "V06a 真实窗口 run（主窗口句柄+受控终止）",
            "vm": "V06e 解释形态 run_ui 真实运行链（事件观测）+ 协议 cancel 终态/宿主存续",
            "vue/jet/ark": "run 宿主域=dev server 交互面，build 矩阵（V06c）承载真实"
                           "安装/构建链；浏览器交互面归 autoui-verifier 双端设施",
            "tauri/vscode": "run 宿主域（tauri CLI/vscode 环境），generate 真实（V06b）；"
                            "本机环境缺席逐项具名——不伪称已运行",
        },
        "terminal_media_lifetime": "窗口/终端/媒体寿命矩阵归 autoui-verifier 双端"
                                   "（test_vm_mcp.py 窗口/MCP/select_rect/退出）——发行收据外记录",
    }

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
