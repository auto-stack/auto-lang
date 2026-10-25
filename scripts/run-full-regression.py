#!/usr/bin/env python3
"""PLAN-850: serial full regression across the language and UI owners.

cargo tf/taa/ta/t3 enter here. `--list-only` builds and validates selection;
it never runs tests. `--batch` appends the existing language tt/tb gates.
"""
from __future__ import annotations

import argparse
import contextlib
import ctypes
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent.parent
UI_PACKAGES = ("auto-ui-compiler", "auto-ui-dialect", "auto-ui-runtime")


def identity(path):
    def git(*args):
        return subprocess.check_output(["git", "-C", str(path), *args])
    return {"head": git("rev-parse", "HEAD").decode().strip(),
            "diff_sha256": hashlib.sha256(git("diff", "HEAD", "--", "crates", "scripts", ".cargo", ".config", "Cargo.toml", "Cargo.lock")).hexdigest(),
            "lock_sha256": hashlib.sha256((path / "Cargo.lock").read_bytes()).hexdigest()}


def validate_layout(lang, ui):
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=lang))
    package = next(p for p in metadata["packages"] if p["name"] == "auto-lang")
    for dependency in package["dependencies"]:
        if dependency["name"] in UI_PACKAGES:
            expected = ui / "crates" / dependency["name"]
            if Path(dependency["path"]).resolve() != expected.resolve():
                raise ValueError(f"UI owner mismatch: {dependency['name']} resolves to {dependency['path']}, requested {expected}")


def require_main(lang):
    listing = subprocess.check_output(["git", "-C", str(lang), "worktree", "list", "--porcelain"], text=True)
    main = Path(listing.splitlines()[0].removeprefix("worktree ")).resolve()
    if lang.resolve() != main:
        raise ValueError(f"unfiltered full regression is main-checkout only: {main}; use --list-only for work/review")


@contextlib.contextmanager
def preserve_reports(lang, out):
    """These producer-only reports are outputs, never test inputs.

    Move existing files aside so Windows mapped-file handles retain their old
    backing file while tests write fresh outputs at the original paths.
    Keep both the original bytes and generated evidence in the receipt folder.
    """
    paths = [lang / "docs/reports/plan796/turnaround-ipc.json"]
    paths += sorted((lang / "docs/reports/plan818").glob("turnaround-mn-*.json"))
    saved = []
    manifest = []
    try:
        for index, path in enumerate(paths):
            if path.exists():
                backup = out / f"report-{index}-before"
                manifest.append({"path": str(path), "before_sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
                path.replace(backup)
                saved.append((path, backup, index))
        yield
    finally:
        errors = []
        for path, backup, index in reversed(saved):
            try:
                if path.exists():
                    path.replace(out / f"report-{index}-generated")
                backup.replace(path)
                item = next(item for item in manifest if item["path"] == str(path))
                item["restored_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
                if item["restored_sha256"] != item["before_sha256"]:
                    raise ValueError(f"restored report hash mismatch: {path}")
            except OSError as error:
                errors.append(f"{path}: {error}; original retained at {backup}")
        (out / "preservation.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        if errors:
            raise ValueError("report restoration failed: " + "; ".join(errors))


def root(name):
    override = os.environ.get(name.upper().replace("-", "_") + "_ROOT")
    candidates = [Path(override)] if override else [HERE if name == "auto-lang" else HERE.parent / name,
                                                    Path("D:/autostack") / name]
    for candidate in candidates:
        if (candidate / "Cargo.toml").is_file():
            return candidate.resolve()
    raise ValueError(f"missing {name} manifest: {candidates}")


@contextlib.contextmanager
def machine_gate(name="AutoLangBatchRegression"):
    """Same Windows machine gate as the batch regression workflow."""
    if os.name == "nt":
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.CreateMutexW.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_wchar_p]
        kernel.CreateMutexW.restype = ctypes.c_void_p
        kernel.CloseHandle.argtypes = [ctypes.c_void_p]
        handle = kernel.CreateMutexW(None, False, "Local\\" + name)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        kernel.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
        result = kernel.WaitForSingleObject(handle, 0)
        if result not in (0, 0x80):
            kernel.CloseHandle(handle)
            raise ValueError("another full regression owns the machine gate")
        try:
            yield
        finally:
            kernel.ReleaseMutex.argtypes = [ctypes.c_void_p]
            kernel.ReleaseMutex(handle)
            kernel.CloseHandle(handle)
    else:
        import fcntl
        with open(Path(os.environ.get("TMPDIR", "/tmp")) / (name + ".lock"), "a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            yield


def reap(proc):
    if os.name == "nt":
        killed = subprocess.run(["taskkill", "/PID", str(proc.pid), "/T", "/F"], capture_output=True, timeout=10)
        if killed.returncode and proc.poll() is None:
            proc.kill()
            proc.wait(timeout=10)
            raise ValueError(f"tree cleanup failed for child {proc.pid}: {killed.stderr!r}")
    else:
        import signal
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    proc.wait(timeout=10)


def command(cmd, cwd, log, timeout):
    with log.open("w", encoding="utf-8") as output:
        proc = subprocess.Popen(cmd, cwd=cwd, stdout=output, stderr=subprocess.STDOUT,
                                start_new_session=os.name != "nt")
        try:
            return proc.wait(timeout=timeout)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            reap(proc)
            raise


def selection(data, allow_empty=False):
    suites = data["rust-suites"]
    ids, selected = set(), set()
    for suite, value in suites.items():
        for name, case in value["testcases"].items():
            identity = suite + "\t" + name
            ids.add(identity)
            if case["filter-match"]["status"] == "matches" and not case["ignored"]:
                selected.add(identity)
    if not selected and not allow_empty:
        raise ValueError("zero selected tests")
    return ids, selected


def validate_ids(ids, selected, expected, required_selected):
    missing = sorted(expected - ids)
    suppressed = sorted(required_selected - selected)
    if missing or suppressed:
        raise ValueError(f"baseline coverage lost: missing={len(missing)}, suppressed={len(suppressed)}")
    return sorted(ids - expected)


def legs(tier, lang, ui, batch=False):
    config = lang / ".config" / ("nextest-t3.toml" if tier == "t3" else "nextest-full.toml")
    common = ["--lib", "--config-file", str(config)]
    language = ["-p", "auto-lang", "-p", "auto-lang-core", *common]
    if tier == "tf":
        for target in ("schema_drift", "docs_gen", "component_registry_test"):
            language += ["--test", target]
    else:
        features = "auto-lang/test-aavm"
        if tier in ("ta", "t3"):
            features += ",auto-lang/test-trans,auto-lang/test-book"
        language += ["--features", features]
    ui_args = [arg for package in UI_PACKAGES for arg in ("-p", package)]
    # The old joint workspace unified compiler/ui through auto-lang/ui-iced.
    ui_args += [*common, "--features", "auto-ui-compiler/ui"]
    result = [("language", lang, language), ("ui-full", ui, ui_args)]
    if batch:
        result += [("tt", lang, ["-p", "auto-lang", "-p", "auto-lang-core", "--lib", "--features",
                                 "auto-lang/test-trans,auto-lang/test-cases"]),
                   ("tb", lang, ["-p", "auto-lang", "-p", "auto-lang-core", "--lib", "--features", "auto-lang/test-book"])]
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tier", choices=("tf", "taa", "ta", "t3"))
    parser.add_argument("filters", nargs="*", help="existing substring filters; scoped runs cannot certify full coverage")
    parser.add_argument("--no-fail-fast", action="store_true", help="accepted for compatibility; children always collect failures")
    parser.add_argument("--list-only", action="store_true")
    parser.add_argument("--batch", action="store_true")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--timeout", type=float, default=14400, help="whole child deadline; test deadlines remain unchanged")
    args = parser.parse_args(argv)
    if args.timeout <= 0 or (args.batch and (args.tier != "tf" or args.filters)):
        parser.error("positive deadline required; --batch is only valid with unfiltered tf")
    out = (args.output or HERE.parent / "full-regression-receipts" / str(time.time_ns())).resolve()
    out.mkdir(parents=True, exist_ok=False)
    receipt = {"tier": args.tier, "filters": args.filters, "list_only": args.list_only, "results": [], "outcome": "fail"}
    try:
        with machine_gate():
            lang, ui = root("auto-lang"), root("auto-ui")
            validate_layout(lang, ui)
            if not args.list_only and not args.filters:
                require_main(lang)
            receipt["roots"] = {"language": str(lang), "ui": str(ui)}
            before = {"language": identity(lang), "ui": identity(ui)}
            receipt["identities"] = before
            total_selected = 0
            for label, cwd, options in legs(args.tier, lang, ui, args.batch):
                options += args.filters
                record = {"leg": label, "cwd": str(cwd), "options": options}
                receipt["results"].append(record)
                listing = out / (label + "-list.json")
                with listing.open("w", encoding="utf-8") as stdout, (out / (label + "-build.log")).open("w", encoding="utf-8") as stderr:
                    proc = subprocess.Popen(["cargo", "nextest", "list", *options, "--message-format", "json"], cwd=cwd,
                                            stdout=stdout, stderr=stderr, start_new_session=os.name != "nt")
                    try:
                        code = proc.wait(timeout=args.timeout)
                    except (subprocess.TimeoutExpired, KeyboardInterrupt):
                        reap(proc)
                        raise
                record["list_exit"] = code
                if code:
                    raise ValueError(f"{label}: listing failed ({code})")
                ids, selected = selection(json.loads(listing.read_text(encoding="utf-8")), allow_empty=bool(args.filters))
                total_selected += len(selected)
                record.update(ids=len(ids), selected=len(selected))
                baseline = HERE / "scripts" / ("full-regression-ui-ids.txt" if label == "ui-full" else "full-regression-language-ids.txt")
                expected = set(baseline.read_text(encoding="utf-8").splitlines())
                # Non-tf language tiers historically omit the three integration binaries.
                if label != "ui-full" and (label != "language" or args.tier != "tf"):
                    expected = {identity for identity in expected if "::" not in identity.split("\t")[0]}
                required_file = (HERE / "scripts" / f"full-regression-{label}-selected.txt" if label in ("tt", "tb")
                                 else baseline.with_name(baseline.stem.replace('-ids', '-selected') + '.txt'))
                required = set(required_file.read_text(encoding="utf-8").splitlines()) & expected
                record["added"] = validate_ids(ids, selected, expected, set() if args.filters else required)
                if not selected:
                    record["skip"] = "explicit scope matches no tests in this owner"
                    continue
                if not args.list_only:
                    report_dir = out / (label + "-reports")
                    report_dir.mkdir()
                    with preserve_reports(lang, report_dir):
                        code = command(["cargo", "nextest", "run", *options, "--no-fail-fast"], cwd,
                                       out / (label + "-run.log"), args.timeout)
                    record["run_exit"] = code
                    if code:
                        raise ValueError(f"{label}: tests failed ({code}); stopping")
            if not total_selected:
                raise ValueError("zero selected tests across all owners")
            if before != {"language": identity(lang), "ui": identity(ui)}:
                raise ValueError("code, configuration or dependency changed during regression")
            receipt["outcome"] = "pass"
            return 0
    except (ValueError, OSError, KeyError, subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        receipt["error"] = str(error) or type(error).__name__
        print(receipt["error"], file=sys.stderr)
        return 1
    finally:
        (out / "receipt.json").write_text(json.dumps(receipt, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"Full regression receipt: {out}")


if __name__ == "__main__":
    sys.exit(main())
