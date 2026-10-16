"""PLAN-803 F803-R1-03 可重复反控：rust-embed 调试态/发行态资源边界。

背景：workspace 与独立 consumer 均只启用 rust-embed `compression`（无
`debug-embed`）。默认调试态 `iter()/get()` 走动态模式——枚举/读取宏展开期
记录的**源码资源目录**（有读 IO；目录缺失时枚举静默为空）；发行态（或
`debug-embed`）字节内嵌进二进制，与源码目录无关。本脚本把该边界钉成可重
复断言：

  1. `cargo package` 打包 auto-man-assets，解包到 target/ 下临时目录；
  2. 用解包副本构造隔离 consumer（不引用仓内路径），debug/release 各跑
     一遍（全绿基线）；
  3. **只对探针自己的解包副本**把 assets/ 改名移开（绝不触碰真实 crate
     资源），重跑已编译测试二进制：debug 必须非零（动态读源目录），
     release 必须零（内嵌）；
  4. 移回后删除一个资源文件：consumer 测试必须非零（缺资源真失败，
     非清单检查）。

用法（仓根）：python test/fixtures/modularity/man-assets-consumer/mode-boundary-probe.py
非破坏性：所有改名/删除都发生在 target/ 下本探针的解包副本内，结束即恢复。
"""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import tarfile

REPO = Path(__file__).resolve().parents[4]  # 仓根（test/fixtures/modularity/<fixture>/ 下第 5 层）
SCRATCH = REPO / "target/plan803-mode-boundary-probe"
FIXTURE = REPO / "test/fixtures/modularity/man-assets-consumer"


def run(name, args, cwd, env):
    p = subprocess.run(args, cwd=cwd, env=env, capture_output=True)
    log = SCRATCH / f"{name}.log"
    log.write_bytes(p.stdout + p.stderr)
    print(f"{name}: exit={p.returncode}")
    return p


def main() -> int:
    env = os.environ.copy()

    shutil.rmtree(SCRATCH, ignore_errors=True)
    SCRATCH.mkdir(parents=True)

    # 1) 打包并解包（探针私有副本）
    pkg = subprocess.run(
        ["cargo", "package", "-p", "auto-man-assets", "--no-verify", "--allow-dirty"],
        cwd=REPO, env=env, capture_output=True,
    )
    if pkg.returncode != 0:
        print("FAIL: cargo package", pkg.stderr.decode("utf-8", "replace")[-500:])
        return 1
    archive = REPO / "target/package/auto-man-assets-0.1.0.crate"
    with tarfile.open(archive) as tf:
        tf.extractall(SCRATCH, filter="data")
    package = SCRATCH / "auto-man-assets-0.1.0"
    assets = package / "assets"

    # 2) 隔离 consumer（依赖解包副本，不引用仓内 crate）
    consumer = SCRATCH / "consumer"
    (consumer / "tests").mkdir(parents=True)
    shutil.copyfile(FIXTURE / "tests/assets_consumer.rs", consumer / "tests/assets_consumer.rs")
    (consumer / "Cargo.toml").write_text(
        '[package]\nname="man-assets-consumer"\nversion="0.1.0"\nedition="2021"\n'
        "[workspace]\n[dependencies]\n"
        'auto-man-assets={path="../auto-man-assets-0.1.0"}\n',
        encoding="utf-8",
    )
    exes = {}
    for label, flags in [("debug", []), ("release", ["--release"])]:
        p = run(
            f"packaged-consumer-{label}",
            ["cargo", "test", *flags, "--manifest-path", str(consumer / "Cargo.toml"),
             "--message-format=json"],
            cwd=SCRATCH, env=env,
        )
        if p.returncode != 0:
            print(f"FAIL: {label} 基线测试未过")
            return 1
        artifacts = [json.loads(x) for x in p.stdout.splitlines() if x.startswith(b"{")]
        exes[label] = next(
            x["executable"] for x in artifacts
            if x.get("reason") == "compiler-artifact"
            and x["target"]["name"] == "assets_consumer"
            and x.get("executable")
        )

    # 3) 移开探针副本的资源目录：debug 必须非零、release 必须零
    hidden = package / "assets-hidden-for-relocation"
    assert assets.resolve().is_relative_to(SCRATCH.resolve()), "反控只许动探针副本"
    assert hidden.resolve().is_relative_to(SCRATCH.resolve())
    assets.rename(hidden)
    try:
        d = run("relocated-debug", [exes["debug"]], cwd=SCRATCH, env=env)
        r = run("relocated-release", [exes["release"]], cwd=SCRATCH, env=env)
        if d.returncode == 0:
            print("FAIL: 调试态在源码资源目录移开后仍全绿——动态读盘边界消失（被误改为内嵌？）")
            return 1
        if r.returncode != 0:
            print("FAIL: 发行态在源码资源目录移开后失败——内嵌边界消失")
            return 1
        print("边界成立：debug=动态读源码目录（exit!=0），release=内嵌（exit=0）")
    finally:
        hidden.rename(assets)

    # 4) 缺资源真失败反控（仍在探针副本内）
    missing = assets / "templates/app/name.at"
    saved = SCRATCH / "saved-app-name.at"
    missing.rename(saved)
    try:
        m = run("missing-resource", [exes["debug"]], cwd=SCRATCH, env=env)
        if m.returncode == 0:
            print("FAIL: 缺资源仍全绿——枚举/断言未真实消费资源")
            return 1
        print("缺资源反控成立（exit!=0）")
    finally:
        saved.rename(missing)

    print("PASS: 模式边界=调试态读源码目录/发行态内嵌，缺资源真失败")
    return 0


if __name__ == "__main__":
    sys.exit(main())
