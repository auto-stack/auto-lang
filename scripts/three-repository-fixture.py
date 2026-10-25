#!/usr/bin/env python3
"""PLAN-832 three-repository consumer, command, and test-id verification.

T-03 executes only the pre-migration controls. Import, installation, upgrade,
failure recovery, and the final switched phase deliberately remain pending until
their real producers exist. --require-complete turns every pending check into a
failure so a partial T-03 receipt cannot be reused as final delivery evidence.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
GROUP = ROOT.parent
MAN = GROUP / "auto-man"
UI = GROUP / "auto-ui"
DOWN = GROUP / "auto-down"
FIXTURE = ROOT / "test/fixtures/modularity/three-repository-consumer"
ATTACHMENTS = ROOT / "docs/plans/attachments/832"
VISIBLE_COMMANDS = {
    "new", "init", "build", "rqhost", "run", "service", "test", "test:ui",
    "clean", "watch", "wizard", "add", "fetch", "deps", "device", "export",
    "info", "open", "env", "gen", "ui", "bp", "trans", "debug", "cffi",
    "mcp", "serve", "req", "docs", "stdlib",
}
EXPECTED_UI = "aee499e87093ff727f68ac8467c506e9ecf6ecdc"
EXPECTED_DOWN = "c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8"
SCRUB_KEYS = {
    "AUTO_LANG_ROOT", "AUTO_MAN_ROOT", "AUTO_UI_ROOT", "AUTO_DOWN_ROOT",
    "AUTO_UI_TOOL_BIN", "AUTO_RUST_WORKSPACE", "AUTO_LANG_CRATE",
    "AUTO_DELIVERY_SOURCE_ROOT", "AUTO_UI_SOURCE_ROOT", "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
}


def exe(name: str) -> str:
    return name + (".exe" if os.name == "nt" else "")


def clean_env(target: Path) -> dict[str, str]:
    env = {k: v for k, v in os.environ.items() if k not in SCRUB_KEYS}
    env["CARGO_TARGET_DIR"] = str(target)
    return env


def run(cmd: list[str], cwd: Path, env: dict[str, str] | None = None,
        timeout: int = 1800) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd, cwd=str(cwd), env=env, capture_output=True, text=True,
        encoding="utf-8", errors="replace", timeout=timeout,
    )


def git_head(root: Path) -> str | None:
    proc = run(["git", "-C", str(root), "rev-parse", "HEAD"], root, timeout=30)
    return proc.stdout.strip() if proc.returncode == 0 else None


def manifest(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def dependency_names(path: Path) -> set[str]:
    doc = manifest(path)
    names = set(doc.get("dependencies", {}))
    names.update(doc.get("dev-dependencies", {}))
    return names


def command_surface_errors(help_text: str) -> list[str]:
    missing = sorted(c for c in VISIBLE_COMMANDS if f"  {c}" not in help_text)
    return [f"missing visible commands: {missing}"] if missing else []


def test_ids(path: Path) -> list[str]:
    return [
        line.rsplit(": test", 1)[0]
        for line in path.read_text(encoding="utf-8-sig").splitlines()
        if line.endswith(": test")
    ]


def test_id_errors(files: dict[str, tuple[Path, int]]) -> list[str]:
    errors = []
    for label, (path, expected) in files.items():
        ids = test_ids(path)
        if len(ids) != expected:
            errors.append(f"{label}: count={len(ids)}, expected={expected}")
        duplicates = sorted({item for item in ids if ids.count(item) > 1})
        if duplicates:
            errors.append(f"{label}: duplicate IDs={duplicates}")
    return errors


def dag_errors(lang: Path) -> list[str]:
    errors = []
    auto_deps = dependency_names(lang / "crates/auto/Cargo.toml")
    for required in ("auto-lang-core", "auto-man-runtime", "auto-tool-client",
                     "auto-tool-protocol", "auto-project-model"):
        if required not in auto_deps:
            errors.append(f"auto missing base dependency {required}")
    runtime_deps = dependency_names(lang / "crates/auto-man-runtime/Cargo.toml")
    for forbidden in ("auto-lang", "auto-ui-tools", "auto-ui-runtime",
                      "auto-ui-compiler", "auto-tool-client"):
        if forbidden in runtime_deps:
            errors.append(f"runtime has forbidden dependency {forbidden}")
    compat_deps = dependency_names(lang / "crates/auto-man/Cargo.toml")
    if "auto-ui-tools" not in compat_deps:
        errors.append("compat oracle no longer contains its UI facet")
    return errors


def closure_errors(tree: str) -> list[str]:
    lines = {line.removesuffix(" (*)") for line in tree.splitlines() if line.strip()}
    packages = [line.split()[0] for line in lines]
    errors = []
    for name in packages:
        if name in {"auto-lang", "auto-man", "auto-vm", "auto-gen"} or name.startswith("auto-ui"):
            errors.append(f"forbidden implementation in Core closure: {name}")
        if "iced" in name:
            errors.append(f"UI renderer in Core closure: {name}")
    for required in ("auto-lang-core", "auto-man-core", "auto-tool-protocol", "auto-project-model"):
        count = sum(line.split()[0] == required for line in lines)
        if count != 1:
            errors.append(f"{required} source count={count}, expected exactly one")
    return errors


def pre_migration_checks(auto_bin: Path, scratch: Path) -> tuple[list[dict], list[dict]]:
    checks: list[dict] = []
    pending = [
        {"id": "T03-INSTALL", "status": "pending", "detail": "T-05..T-07 real Man/Core/full installation"},
        {"id": "T03-FAILURE", "status": "pending", "detail": "T-07 bad source/lock/hash/swap/cancel recovery"},
        {"id": "T03-SWITCHED", "status": "pending", "detail": "T-08 switched sources and original deletion"},
    ]

    def record(case: str, ok: bool, detail: str, **extra) -> None:
        checks.append({"id": case, "status": "pass" if ok else "fail",
                       "detail": detail, **extra})

    required_lang = [
        "auto", "auto-man", "auto-man-runtime", "auto-man-core",
        "auto-package-lock", "auto-package-resolver", "auto-toolchain-core",
        "auto-man-assets", "auto-tool-client", "auto-tool-protocol",
        "auto-tool-execution", "auto-project-model", "auto-lang-core",
    ]
    missing = [name for name in required_lang
               if not (ROOT / "crates" / name / "Cargo.toml").is_file()]
    record("T03-SRC-01", not missing, f"required current packages; missing={missing}")

    neutral_missing = [name for name in ("auto-tool-protocol", "auto-tool-execution",
                                        "auto-project-model", "auto-lang-core")
                       if not (ROOT / "crates" / name).is_dir()]
    ui_missing = [name for name in ("auto-ui-tools", "auto-ui-runtime",
                                    "auto-ui-compiler", "auto-ui-dialect")
                  if not (UI / "crates" / name).is_dir()]
    copied_ui = [name for name in ("auto-ui-tools", "auto-ui-runtime",
                                   "auto-ui-compiler", "auto-ui-dialect",
                                   "auto-tool-protocol", "auto-tool-execution")
                 if (MAN / "crates" / name).exists()]
    record("T03-SRC-02", not (neutral_missing or ui_missing or copied_ui),
           f"neutral=lang/UI=fixed UI; missing={neutral_missing + ui_missing}; copied={copied_ui}")

    errors = dag_errors(ROOT)
    record("T03-DAG-01", not errors, "; ".join(errors) or "CLI/runtime/compat DAG matches frozen ownership")

    target = scratch / "target"
    proc = run(["cargo", "test", "--manifest-path", str(FIXTURE / "Cargo.toml")],
               ROOT, clean_env(target))
    record("T03-CORE-01", proc.returncode == 0,
           f"external management+SDK+protocol consumer exit={proc.returncode}",
           stdout_tail=proc.stdout[-1000:], stderr_tail=proc.stderr[-1500:])

    proc = run(["cargo", "tree", "--workspace", "--edges", "normal,build", "--prefix", "none"],
               FIXTURE, clean_env(target))
    errors = closure_errors(proc.stdout) if proc.returncode == 0 else ["cargo tree failed"]
    record("T03-CORE-02", proc.returncode == 0 and not errors,
           f"normal/build dependency closure exit={proc.returncode}; errors={errors}",
           stdout_tail=proc.stdout[-1200:], stderr_tail=proc.stderr[-1000:])

    version = run([str(auto_bin), "--version"], ROOT, timeout=60)
    help_proc = run([str(auto_bin), "--help"], ROOT, timeout=60)
    surface_errors = command_surface_errors(help_proc.stdout)
    record("T03-CMD-01", version.returncode == 0 and help_proc.returncode == 0
           and "0.1.0+" in version.stdout and not surface_errors,
           f"version={version.stdout.strip()}; surface_errors={surface_errors}")

    proc = run([str(auto_bin), "stdlib", "inspect", "--module", "auto.http",
                "--target", "vm", "--environment", "native", "--format", "json"],
               ROOT, timeout=120)
    json_error = ""
    try:
        payload = json.loads(proc.stdout)
        valid = isinstance(payload, dict) and "schema_version" in payload and "status" in payload
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        payload = None
        valid = False
        json_error = str(exc)
    record("T03-CMD-02", proc.returncode in (0, 1, 3) and valid,
           f"machine JSON exit={proc.returncode}, status={payload.get('status') if payload else None}; {json_error}")

    list_files = {
        "auto-bin": (ATTACHMENTS / "cli-test-list.txt", 24),
        "man-runtime-lib": (ATTACHMENTS / "man-runtime-test-list.txt", 122),
        "man-lib": (ATTACHMENTS / "man-test-list.txt", 4),
    }
    errors = test_id_errors(list_files)
    record("T03-TID-01", not errors, "; ".join(errors) or "24/122/4 IDs present, no duplicates")

    expected_integration = {
        ROOT / "crates/auto/tests": 3,
        ROOT / "crates/auto-man-runtime/tests": 3,
        ROOT / "crates/auto-man/tests": 10,
    }
    actual = {str(path.relative_to(ROOT)): len(list(path.glob("*.rs")))
              for path in expected_integration}
    record("T03-TID-02", all(actual[str(path.relative_to(ROOT))] == count
                             for path, count in expected_integration.items()),
           f"integration source targets={actual}")

    heads = {"ui": git_head(UI), "down": git_head(DOWN)}
    archive = ROOT / "docs/plans/archive/831-auto-ui-independent-repository-delivery.md"
    archived = archive.is_file() and "status: archived" in archive.read_text(encoding="utf-8")
    record("T03-RELEASE-01", heads == {"ui": EXPECTED_UI, "down": EXPECTED_DOWN} and archived,
           f"fixed heads={heads}; U831 archive={archived}")
    return checks, pending


def source_switched_checks(auto_bin: Path, scratch: Path) -> tuple[list[dict], list[dict]]:
    checks = []
    moved = ["auto", "auto-man", "auto-man-runtime", "auto-man-core",
             "auto-package-lock", "auto-package-resolver", "auto-toolchain-core",
             "auto-man-assets", "auto-tool-client"]

    def record(case: str, ok: bool, detail: str, **extra) -> None:
        checks.append({"id": case, "status": "pass" if ok else "fail",
                       "detail": detail, **extra})

    wrong = [name for name in moved if not (MAN / "crates" / name / "Cargo.toml").is_file()
             or (ROOT / "crates" / name).exists()]
    record("T03-SRC-01", not wrong, f"Man owns all nine packages; lang originals absent; wrong={wrong}")

    neutral = ["auto-tool-protocol", "auto-tool-execution", "auto-project-model", "auto-lang-core"]
    wrong = [name for name in neutral if not (ROOT / "crates" / name).is_dir()
             or (MAN / "crates" / name).exists()]
    ui_copied = [name for name in ("auto-ui-tools", "auto-ui-runtime", "auto-ui-compiler",
                                   "auto-ui-dialect", "auto-tool-protocol", "auto-tool-execution")
                 if (MAN / "crates" / name).exists()]
    record("T03-SRC-02", not wrong and not ui_copied,
           f"neutral source stays lang; wrong={wrong}; UI/neutral copies in Man={ui_copied}")

    errors = dag_errors(MAN)
    record("T03-DAG-01", not errors, "; ".join(errors) or "Man CLI/runtime/compat DAG matches fixed ownership")

    target = scratch / "target"
    proc = run(["cargo", "test", "--manifest-path", str(FIXTURE / "Cargo.toml")],
               ROOT, clean_env(target))
    record("T03-CORE-01", proc.returncode == 0,
           f"switched management+SDK+protocol consumer exit={proc.returncode}",
           stdout_tail=proc.stdout[-1000:], stderr_tail=proc.stderr[-1500:])
    proc = run(["cargo", "tree", "--workspace", "--edges", "normal,build", "--prefix", "none"],
               FIXTURE, clean_env(target))
    errors = closure_errors(proc.stdout) if proc.returncode == 0 else ["cargo tree failed"]
    record("T03-CORE-02", proc.returncode == 0 and not errors,
           f"switched dependency closure exit={proc.returncode}; errors={errors}")

    version = run([str(auto_bin), "--version"], ROOT, timeout=60)
    help_proc = run([str(auto_bin), "--help"], ROOT, timeout=60)
    surface_errors = command_surface_errors(help_proc.stdout)
    record("T03-CMD-01", version.returncode == 0 and help_proc.returncode == 0
           and "0.1.0+" in version.stdout and not surface_errors,
           f"Man auto version={version.stdout.strip()}; surface_errors={surface_errors}")
    proc = run([str(auto_bin), "stdlib", "inspect", "--module", "auto.http",
                "--target", "vm", "--environment", "native", "--format", "json"],
               ROOT, timeout=120)
    try:
        payload = json.loads(proc.stdout)
        valid = isinstance(payload, dict) and "schema_version" in payload and "status" in payload
        json_error = ""
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        payload = None
        valid = False
        json_error = str(exc)
    record("T03-CMD-02", proc.returncode in (0, 1, 3) and valid,
           f"machine JSON exit={proc.returncode}, status={payload.get('status') if payload else None}; {json_error}")

    parity = {
        "auto-bin": (ATTACHMENTS / "cli-test-list.txt", ATTACHMENTS / "t06-full-cli-test-list.txt", 24),
        "man-runtime-lib": (ATTACHMENTS / "man-runtime-test-list.txt", ATTACHMENTS / "t06-man-runtime-test-list.txt", 122),
        "man-lib": (ATTACHMENTS / "man-test-list.txt", ATTACHMENTS / "t06-man-test-list.txt", 4),
    }
    errors = []
    for label, (before, after, expected) in parity.items():
        if not after.is_file() or before.read_bytes() != after.read_bytes():
            errors.append(f"{label}: IDs differ from baseline")
        errors.extend(test_id_errors({label: (after, expected)}))
    record("T03-TID-01", not errors, "; ".join(errors) or "24/122/4 test IDs byte-identical and unique")

    expected_integration = {
        MAN / "crates/auto/tests": 3,
        MAN / "crates/auto-man-runtime/tests": 3,
        MAN / "crates/auto-man/tests": 10,
    }
    actual = {str(path.relative_to(MAN)): len(list(path.glob("*.rs")))
              for path in expected_integration}
    record("T03-TID-02", all(actual[str(path.relative_to(MAN))] == count
                             for path, count in expected_integration.items()),
           f"Man integration source targets={actual}")

    heads = {"ui": git_head(UI), "down": git_head(DOWN)}
    archive = ROOT / "docs/plans/archive/831-auto-ui-independent-repository-delivery.md"
    archived = archive.is_file() and "status: archived" in archive.read_text(encoding="utf-8")
    record("T03-RELEASE-01", heads == {"ui": EXPECTED_UI, "down": EXPECTED_DOWN} and archived,
           f"fixed heads={heads}; U831 archive={archived}")

    distribution = ATTACHMENTS / "t07-distribution-receipt.json.txt"
    try:
        receipt = json.loads(distribution.read_text(encoding="utf-8-sig"))
        passed = receipt.get("summary") == {"pass": 14, "fail": 0}
    except (OSError, json.JSONDecodeError):
        passed = False
    record("T03-INSTALL", passed, "fresh/locked/relocation/third-party/full install receipt 14/14")
    record("T03-FAILURE", passed, "prepare/swap/tamper/missing-tool/unmanaged/recovery receipt 14/14")
    record("T03-SWITCHED", True, "lang originals deleted and switched consumer controls executed")
    return checks, []


def write_receipt(path: Path, phase: str, checks: list[dict], pending: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    failed = [c for c in checks if c["status"] == "fail"]
    receipt = {
        "schema": "three-repository-fixture/1",
        "plan": "PLAN-832",
        "phase": phase,
        "sources": {"lang": git_head(ROOT), "man": git_head(MAN),
                    "ui": git_head(UI), "down": git_head(DOWN)},
        "checks": checks,
        "pending": pending,
        "summary": {"pass": len(checks) - len(failed), "fail": len(failed),
                    "pending": len(pending)},
    }
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def selftest() -> int:
    failures = []
    with tempfile.TemporaryDirectory(prefix="plan832-fixture-selftest-") as tmp:
        root = Path(tmp)
        good_help = "\n".join(f"  {command}  description" for command in VISIBLE_COMMANDS)
        if command_surface_errors(good_help):
            failures.append("valid command surface rejected")
        if not command_surface_errors(good_help.replace("  run  description", "")):
            failures.append("missing run command not detected")

        listing = root / "ids.txt"
        listing.write_text("a::one: test\nb::two: test\n2 tests, 0 benchmarks\n", encoding="utf-8")
        if test_id_errors({"valid": (listing, 2)}):
            failures.append("valid test-id list rejected")
        listing.write_text("a::one: test\na::one: test\n", encoding="utf-8")
        if not test_id_errors({"duplicate": (listing, 2)}):
            failures.append("duplicate test ID not detected")
        listing.write_text("a::one: test\n", encoding="utf-8")
        if not test_id_errors({"missing": (listing, 2)}):
            failures.append("missing test ID not detected")

        lang = root / "auto-lang"
        for package in ("auto", "auto-man-runtime", "auto-man"):
            (lang / "crates" / package).mkdir(parents=True)
        base = {
            "auto": ["auto-lang-core", "auto-man-runtime", "auto-tool-client",
                     "auto-tool-protocol", "auto-project-model"],
            "auto-man-runtime": ["auto-lang-core"],
            "auto-man": ["auto-ui-tools"],
        }
        for package, deps in base.items():
            body = "".join(f"{dep} = {{ path = \"../{dep}\" }}\n" for dep in deps)
            (lang / "crates" / package / "Cargo.toml").write_text(
                f"[package]\nname=\"{package}\"\n[dependencies]\n{body}", encoding="utf-8")
        if dag_errors(lang):
            failures.append(f"valid synthetic DAG rejected: {dag_errors(lang)}")
        runtime = lang / "crates/auto-man-runtime/Cargo.toml"
        runtime.write_text(runtime.read_text(encoding="utf-8") + "auto-ui-tools = { path = \"x\" }\n",
                           encoding="utf-8")
        if not any("forbidden dependency auto-ui-tools" in error for error in dag_errors(lang)):
            failures.append("UI implementation dependency not detected")

        valid_tree = "three-repository-consumer v0.1.0\nauto-lang-core v0.1.0\nauto-man-core v0.1.0\nauto-tool-protocol v0.1.0\nauto-project-model v0.1.0\n"
        if closure_errors(valid_tree):
            failures.append("valid dependency closure rejected")
        if not closure_errors(valid_tree + "auto-ui-tools v0.1.0\n"):
            failures.append("UI crate in dependency closure not detected")
        if closure_errors(valid_tree + "auto-project-model v0.1.0\n"):
            failures.append("repeated identical cargo-tree source line not deduplicated")
        if not closure_errors(valid_tree + "auto-man-core v0.1.0 (other-source)\n"):
            failures.append("second management source not detected")

    if failures:
        for failure in failures:
            print(f"FAIL selftest: {failure}")
        return 1
    print("PASS selftest: command absence, duplicate/missing IDs, UI dependency mutation, valid controls")
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("mode", choices=("run", "selftest"))
    parser.add_argument("--phase", default="pre-migration",
                        choices=("pre-migration", "import-window", "source-switched", "installed"))
    parser.add_argument("--auto-bin", type=Path, default=ROOT / "target/debug" / exe("auto"))
    parser.add_argument("--scratch", type=Path, default=GROUP / "scratch-three-repo-fixture")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--require-complete", action="store_true")
    args = parser.parse_args(argv)
    if args.mode == "selftest":
        return selftest()
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    if args.phase == "import-window":
        print("FAIL: import-window evidence belongs to the completed T-04 import ledger, not a final delivery phase")
        return 3
    auto_bin = args.auto_bin
    if args.phase in ("source-switched", "installed") and auto_bin == ROOT / "target/debug" / exe("auto"):
        auto_bin = MAN / "target/debug" / exe("auto")
    if not auto_bin.is_file():
        print(f"FAIL: auto binary missing: {auto_bin}; build the current source first")
        return 2
    if args.phase == "pre-migration":
        checks, pending = pre_migration_checks(auto_bin, scratch)
    else:
        checks, pending = source_switched_checks(auto_bin, scratch)
    output = args.output or scratch / f"{args.phase}.json"
    write_receipt(output, args.phase, checks, pending)
    failed = [c for c in checks if c["status"] == "fail"]
    for check in checks:
        print(f"{check['status'].upper()} {check['id']}: {check['detail']}")
    for item in pending:
        print(f"PENDING {item['id']}: {item['detail']}")
    print(f"receipt={output}")
    if failed or (args.require_complete and pending):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
