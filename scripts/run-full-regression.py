#!/usr/bin/env python3
"""PLAN-896 fixed Cargo/Rust-module periodic queue with preserved external owners.

Language executes one fixed native group union. Existing language/tt/tb listings
remain conservation audits, not repeated execution tiers. --batch uses the
explicit recipe schedule; UI/Man retain their owner legs and manual conditions.
Unfiltered periodic runs require frozen main checkouts, or the reviewed
--frozen-trees batch interface. --list-only selects without executing bodies.
"""
from __future__ import annotations

import argparse
import contextlib
import ctypes
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent.parent
UI_PACKAGES = ("auto-ui-compiler", "auto-ui-dialect", "auto-ui-runtime")
MAN_SEVEN = ("auto-man-runtime", "auto-man-core", "auto-package-lock", "auto-package-resolver",
             "auto-toolchain-core", "auto-man-assets", "auto-tool-client")
MAN_TI_TESTS = ("plan816_cli_driver", "cmd_ui", "autodesk_client_smoke")
MAN_TI_RUNTIME_TESTS = ("plan816_t03_non_ui_service", "plan816_t03_ui_default_fallback",
                        "plan816_t03_ui_inject_explicit_first")
MAN_TI_COMPAT_TESTS = ("plan803_asset_facade_tests", "plan805_toolchain_parity", "plan805_type_identity",
                       "plan809_management_core", "plan811_media_consumers", "plan815_t04_compat",
                       "plan816_t03_create_jet_default", "plan816_t03_legacy_util_paths",
                       "plan816_t03_sidecar_request", "plan816_t03_ui_inject_compat")
UI_TI_TESTS = ("legacy_build_semantics", "lifecycle", "resources_manifest", "tool_process",
               "osconfig_integration")
MINIMUMS_PATH = HERE / "scripts" / "tier-selection-minimums.json.txt"
NEXTEST_FAIL = re.compile(r"^\s*(?:FAIL|TIMEOUT)(?:\s*\+\s*LEAK)?\s+\[[^\]]*\](?:\s+\(\s*\d+/\d+\))?\s+(\S+)\s+(\S+)", re.MULTILINE)
LIBTEST_FAIL = re.compile(r"^test\s+(\S+)\s+\.\.\.\s+FAILED", re.MULTILINE)


def identity(path):
    def git(*args):
        return subprocess.check_output(["git", "-C", str(path), *args])
    return {"head": git("rev-parse", "HEAD").decode().strip(),
            "diff_sha256": hashlib.sha256(git("diff", "HEAD", "--", ".", ":(exclude)docs/plans", ":(exclude)docs/reports")).hexdigest(),
            "lock_sha256": hashlib.sha256((path / "Cargo.lock").read_bytes()).hexdigest()}


def validate_layout(lang, ui, man=None):
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=lang))
    package = next(p for p in metadata["packages"] if p["name"] == "auto-lang")
    for dependency in package["dependencies"]:
        if dependency["name"] in UI_PACKAGES:
            expected = ui / "crates" / dependency["name"]
            if Path(dependency["path"]).resolve() != expected.resolve():
                raise ValueError(f"UI owner mismatch: {dependency['name']} resolves to {dependency['path']}, requested {expected}")
    language_packages = {p["name"]: Path(p["manifest_path"]).parent.resolve() for p in metadata["packages"]}
    ui_metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ui))
    for package in ui_metadata["packages"]:
        for dependency in package["dependencies"]:
            expected = language_packages.get(dependency["name"])
            if expected is not None and dependency.get("path") and Path(dependency["path"]).resolve() != expected:
                raise ValueError(f"language SDK owner mismatch in {package['name']}: {dependency['name']}")
    if man is None:
        return
    # PLAN-867: the Man workspace consumes lang/ui by path; its members must
    # resolve those dependencies to exactly the roots driven by this queue.
    man_metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=man))
    man_packages = {p["name"]: Path(p["manifest_path"]).parent.resolve() for p in man_metadata["packages"]}
    for package in man_metadata["packages"]:
        for dependency in package["dependencies"]:
            target = language_packages.get(dependency["name"])
            if target is None and dependency["name"] in UI_PACKAGES:
                for ui_package in ui_metadata["packages"]:
                    if ui_package["name"] == dependency["name"]:
                        target = Path(ui_package["manifest_path"]).parent.resolve()
            if target is not None and dependency.get("path") and Path(dependency["path"]).resolve() != target:
                raise ValueError(f"cross-repository owner mismatch in {package['name']}: "
                                 f"{dependency['name']} resolves to {dependency['path']}, expected {target}")
    for required in ("auto", *MAN_SEVEN):
        if required not in man_packages:
            raise ValueError(f"missing man workspace package: {required}")


def dirty_code_paths(path):
    return subprocess.check_output(["git", "-C", str(path), "diff", "--name-only", "HEAD", "--",
                                    "crates", "test", "experimental", "scripts", ".cargo", ".config", "Cargo.toml", "Cargo.lock"])


def require_main(lang):
    listing = subprocess.check_output(["git", "-C", str(lang), "worktree", "list", "--porcelain"], text=True)
    main = Path(listing.splitlines()[0].removeprefix("worktree ")).resolve()
    if lang.resolve() != main:
        raise ValueError(f"unfiltered full regression is main-checkout only: {main}; use --list-only for work/review")
    branch = subprocess.check_output(["git", "-C", str(lang), "branch", "--show-current"], text=True).strip()
    if branch != "master":
        raise ValueError("full regression requires the frozen master checkout")
    if dirty_code_paths(lang).strip():
        raise ValueError("main checkout has code/configuration WIP; route it to its owning worktree before full regression")


def require_clean_tree(name, path):
    if dirty_code_paths(path).strip():
        raise ValueError(f"frozen tree {name} has code/configuration WIP: {path}")


@contextlib.contextmanager
def preserve_reports(lang, out):
    """These producer-only reports are outputs, never test inputs.

    Move existing files aside so Windows mapped-file handles retain their old
    backing file while tests write fresh outputs at the original paths.
    Keep both the original bytes and generated evidence in the receipt folder.
    """
    paths = [lang / "docs/reports/plan796/turnaround-ipc.json"]
    paths += sorted((lang / "docs/reports/plan818").glob("turnaround-mn-*.json"))
    paths += [lang / "docs/plans/reports/738-stdlib-matrix.json",
              lang / "docs/plans/reports/738-stdlib-matrix.md",
              lang / "docs/reports/plan827/inventory.json"]
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


def command(cmd, cwd, log, timeout, env=None):
    with log.open("w", encoding="utf-8") as output:
        proc = subprocess.Popen(cmd, cwd=cwd, stdout=output, stderr=subprocess.STDOUT,
                                env=env, start_new_session=os.name != "nt")
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


def libtest_selection(text, package, allow_empty=False):
    """libtest `-- --list` output → (ids, selected) in the same tab identity form."""
    ids, selected = set(), set()
    for line in text.splitlines():
        line = line.rstrip()
        if line.endswith(": test"):
            identity = package + "\t" + line[: -len(": test")].strip()
            ids.add(identity)
            selected.add(identity)
        elif line.endswith(": ignore"):
            ids.add(package + "\t" + line[: -len(": ignore")].strip())
    if not selected and not ids and not allow_empty:
        raise ValueError("zero selected tests")
    return ids, selected


def failed_from_log(path, package=None):
    try:
        text = Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    failed = set()
    for match in NEXTEST_FAIL.finditer(text):
        failed.add(match.group(1) + "\t" + match.group(2))
    for match in LIBTEST_FAIL.finditer(text):
        failed.add((package + "\t" + match.group(1)) if package else match.group(1))
    return sorted(failed)


def validate_ids(ids, selected, expected, required_selected):
    missing = sorted(expected - ids)
    suppressed = sorted(required_selected - selected)
    if missing or suppressed:
        raise ValueError(f"baseline coverage lost: missing={len(missing)}, suppressed={len(suppressed)}")
    return sorted(ids - expected)


def migrations():
    path = HERE / "scripts/full-regression-migrations-841.json.txt"
    doc = json.loads(path.read_text(encoding="utf-8"))
    if doc.get("schema") != "test-id-migrations/1" or doc.get("plan") != 841:
        raise ValueError("unsupported test migration manifest")
    rows = doc["migrations"]
    if len({row["old"] for row in rows}) != len(rows) or len({row["new"] for row in rows}) != len(rows):
        raise ValueError("duplicate test migration identity")
    for row in rows:
        if (row["source"], row["target"]) not in (("language", "language"), ("language", "ui-full"), ("ui-full", "ui-full")):
            raise ValueError("invalid test migration owners")
        if not row.get("reason") or "\t" not in row["old"] or "\t" not in row["new"]:
            raise ValueError("incomplete test migration")
    return rows


RETIREMENT_PATHS = (HERE / "scripts/full-regression-retirements-844.json.txt",
                    HERE / "scripts/full-regression-retirements-869.json.txt")


def retirements():
    """844/869 evidence-cited retirement channels (PLAN-887 T-03 reconciled the
    869 batch-2 disconnect): the frozen 850 baseline files stay untouched;
    retired required IDs are removed here instead (867 T-04 + 869 T-04)."""
    rows = []
    seen = set()
    for path in RETIREMENT_PATHS:
        doc = json.loads(path.read_text(encoding="utf-8"))
        if doc.get("schema") != "test-id-retirements/1" or doc.get("plan") not in (844, 869):
            raise ValueError(f"unsupported test retirement manifest: {path.name}")
        for row in doc["retirements"]:
            if not row.get("evidence") or "\t" not in row["id"]:
                raise ValueError(f"incomplete test retirement: {path.name}")
            if row["id"] in seen:
                raise ValueError(f"duplicate test retirement identity across manifests: {row['id']}")
            seen.add(row["id"])
        rows.extend(doc["retirements"])
    return rows


def raw_baseline(owner):
    base = HERE / "scripts" / f"full-regression-{owner}"
    return (set(base.with_name(base.name + "-ids.txt").read_text(encoding="utf-8").splitlines()),
            set(base.with_name(base.name + "-selected.txt").read_text(encoding="utf-8").splitlines()))


def baseline_for(tier, label):
    owner = "ui" if label == "ui-full" else label if label in ("tt", "tb") else "language" if tier == "tf" else tier
    expected, required = raw_baseline(owner)
    retired = {row["id"] for row in retirements()}
    expected -= retired
    required -= retired
    records = migrations()
    source = "ui-full" if label == "ui-full" else "language"
    for row in records:
        if row["source"] != source:
            continue
        for values in (expected, required):
            if row["old"] in values:
                values.remove(row["old"])
                if row["target"] == source:
                    values.add(row["new"])
    if label == "ui-full":
        language_owner = "language" if tier == "tf" else tier
        language_expected, language_required = raw_baseline(language_owner)
        for row in records:
            if row["source"] == "language" and row["target"] == "ui-full":
                if row["old"] in language_expected:
                    expected.add(row["new"])
                if row["old"] in language_required:
                    required.add(row["new"])
    return expected, required


def tier_minimums():
    if not MINIMUMS_PATH.is_file():
        raise ValueError(f"missing tier selection minimums: {MINIMUMS_PATH}")
    doc = json.loads(MINIMUMS_PATH.read_text(encoding="utf-8"))
    if doc.get("schema") != "tier-selection-minimums/1":
        raise ValueError("unsupported tier selection minimums manifest")
    return doc["legs"]


def check_minimum(legs, label, count):
    entry = legs.get(label)
    if not entry:
        return
    if count < entry["minimum"]:
        raise ValueError(f"{label}: selection below frozen minimum ({count} < {entry['minimum']})")


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
    # PLAN-841 moved nine recipe controls from a lib module into this exact
    # integration target. Keep them in the full tier, with lib tests unchanged.
    ui_args += [*common, "--test", "design_tokens_recipe", "--features", "auto-ui-compiler/ui"]
    result = [("language", lang, language), ("ui-full", ui, ui_args)]
    if batch:
        result += [("tt", lang, ["-p", "auto-lang", "-p", "auto-lang-core", "--lib", "--features",
                                 "auto-lang/test-trans,auto-lang/test-cases"]),
                   ("tb", lang, ["-p", "auto-lang", "-p", "auto-lang-core", "--lib", "--features", "auto-lang/test-book"])]
    return result


def batch_extra_legs(lang, ui, man):
    """PLAN-867 three-repository batch legs, fixed order after tt/tb.

    nextest legs use `cargo nextest list/run <options>`; libtest legs use bare
    `cargo test` (single-process WidgetRegistry semantics for ui-tu) and list
    via `-- --list`. ui-tn is the manual native gate: enumerate only.
    """
    man_tc = [arg for package in MAN_SEVEN for arg in ("-p", package)]
    man_ti = [arg for test in MAN_TI_TESTS for arg in ("-p", "auto", "--test", test)]
    man_ti += [arg for test in MAN_TI_RUNTIME_TESTS for arg in ("-p", "auto-man-runtime", "--test", test)]
    man_ti += [arg for test in MAN_TI_COMPAT_TESTS for arg in ("-p", "auto-man", "--test", test)]
    ui_ti = ["-p", "auto-ui-tools", "-p", "auto-ui-runtime"]
    ui_ti += [arg for test in UI_TI_TESTS for arg in ("--test", test)]
    return [
        {"leg": "man-t", "repo": "man", "cwd": man, "kind": "nextest",
         "options": ["--workspace", "--lib", "--bin", "auto"]},
        {"leg": "man-tcore", "repo": "man", "cwd": man, "kind": "nextest",
         "options": ["--manifest-path", "core/Cargo.toml", "--workspace",
                     "--no-default-features", "--features", "core"]},
        {"leg": "man-tc", "repo": "man", "cwd": man, "kind": "nextest",
         "options": [*man_tc, "--lib"]},
        {"leg": "man-ti", "repo": "man", "cwd": man, "kind": "nextest",
         "options": man_ti},
        {"leg": "ui-tu", "repo": "ui", "cwd": ui, "kind": "libtest", "package": "auto-ui-compiler",
         "list": ["test", "-p", "auto-ui-compiler", "--lib", "ui_gen::", "--", "--list"],
         "run": ["test", "-p", "auto-ui-compiler", "--lib", "ui_gen::"]},
        {"leg": "ui-ti", "repo": "ui", "cwd": ui, "kind": "nextest",
         "options": ui_ti},
        {"leg": "ui-tn", "repo": "ui", "cwd": ui, "kind": "libtest", "package": "auto-ui-runtime",
         "manual": True,
         "list": ["test", "-p", "auto-ui-runtime", "--features", "native-dnd,test-native-dock",
                  "--test", "native_dnd_e2e", "--", "--list"],
         "run": ["test", "-p", "auto-ui-runtime", "--features", "native-dnd,test-native-dock",
                 "--test", "native_dnd_e2e"]},
    ]


def man_bootstrap(legs_spec, lang, ui, out, timeout, env):
    """Same bootstrap contract as scripts/cross-repo-tiers.py: te-child and
    auto-ui-tool are real cross-repository binaries Man integration tests spawn."""
    commands = []
    if any(leg["leg"] == "man-ti" for leg in legs_spec):
        commands.append((ui, ["cargo", "build", "--locked", "-p", "auto-ui-tools"]))
    if any(leg["leg"] in ("man-ti", "man-tcore") for leg in legs_spec):
        commands.append((lang, ["cargo", "build", "--locked", "-p", "auto-tool-execution", "--bin", "te-child"]))
    results = []
    for index, (cwd, cmd) in enumerate(commands):
        code = command(cmd, cwd, out / f"bootstrap-{index}.log", timeout)
        results.append({"command": cmd, "cwd": str(cwd), "exit": code})
        if code:
            raise ValueError(f"man bootstrap failed ({code}): {cmd}")
    return results


def child_env(lang, ui, man):
    result = os.environ.copy()
    for name, path in (("auto-lang", lang), ("auto-ui", ui), ("auto-man", man)):
        result[name.upper().replace("-", "_") + "_ROOT"] = str(path)
    exe = ".exe" if os.name == "nt" else ""
    result.setdefault("AUTO_UI_TOOL_BIN", str(ui / "target/debug" / f"auto-ui-tool{exe}"))
    te_child = str(lang / "target/debug" / f"te-child{exe}")
    result.setdefault("AUTO_TE_CHILD_BIN", te_child)
    result.setdefault("TE_CHILD_BIN", te_child)
    return result


def module_queue_command(lang,tier,filters,batch,output,list_only):
    """One native fixed-group union; external UI/Man owners are unchanged."""
    command=[sys.executable,str(lang/'scripts/fixed-tests.py')]
    if batch:
        command += ['all']
    else:
        command += ['--legacy-tier',tier,*filters]
    command += ['--out',str(output)]
    if list_only:command.append('--select-only')
    return command


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tier", choices=("tf", "taa", "ta", "t3"))
    parser.add_argument("filters", nargs="*", help="existing substring filters; scoped runs cannot certify full coverage")
    parser.add_argument("--no-fail-fast", action="store_true", help="accepted for compatibility; children always collect failures")
    parser.add_argument("--list-only", action="store_true")
    parser.add_argument("--batch", action="store_true")
    parser.add_argument("--frozen-trees", action="store_true",
                        help="PLAN-867: run the unfiltered batch against a clean frozen worktree group instead of the main checkout")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--timeout", type=float, default=14400, help="whole child deadline; test deadlines remain unchanged")
    args = parser.parse_args(argv)
    if args.timeout <= 0 or (args.batch and (args.tier != "tf" or args.filters)):
        parser.error("positive deadline required; --batch is only valid with unfiltered tf")
    if args.frozen_trees and not args.batch:
        parser.error("--frozen-trees applies to the three-repository batch only")
    out = (args.output or HERE.parent / "full-regression-receipts" / str(time.time_ns())).resolve()
    out.mkdir(parents=True, exist_ok=False)
    receipt = {"tier": args.tier, "filters": args.filters, "list_only": args.list_only,
               "frozen_trees": args.frozen_trees, "results": [], "outcome": "fail"}
    try:
        with machine_gate():
            lang, ui = root("auto-lang"), root("auto-ui")
            man = root("auto-man") if args.batch else None
            validate_layout(lang, ui, man)
            if not args.list_only and not args.filters:
                if args.frozen_trees:
                    for name, path in (("language", lang), ("ui", ui), ("man", man)):
                        require_clean_tree(name, path)
                else:
                    require_main(lang)
            receipt["roots"] = {"language": str(lang), "ui": str(ui), **({"man": str(man)} if man else {})}
            receipt["test_migrations"] = migrations()
            before = {"language": identity(lang), "ui": identity(ui)}
            if man is not None:
                before["man"] = identity(man)
            receipt["identities"] = before
            minimums = tier_minimums() if args.batch else {}
            total_selected = 0
            for label, cwd, options in legs(args.tier, lang, ui, args.batch):
                options += args.filters
                record = {"leg": label, "cwd": str(cwd), "options": options}
                receipt["results"].append(record)
                if label in ('tt', 'tb'):
                    record['execution'] = 'covered by the explicit fixed all union; no repeated discovery/build'
                    record['run_exit'] = 0
                    continue
                if label == 'language':
                    destination = out / 'language-modules'
                    record['module_command'] = module_queue_command(lang,args.tier,args.filters,args.batch,destination,args.list_only)
                    reports = out / 'language-module-reports'; reports.mkdir()
                    with preserve_reports(lang,reports) if not args.list_only else contextlib.nullcontext():
                        code = command(record['module_command'],lang,out/'language-module-run.log',args.timeout)
                    record['run_exit'] = code
                    if code:
                        raise ValueError(f'language: fixed group tests failed ({code}); stopping')
                    if args.list_only:
                        planned = json.loads((destination/'selection.json').read_text(encoding='utf-8'))
                        record['declared_groups'] = planned['selected_groups']
                        record['executed'] = 0
                        total_selected += len(planned['selected_groups'])
                    else:
                        actual = json.loads((destination/'results.json').read_text(encoding='utf-8'))
                        if not actual.get('gate_passed'):
                            raise ValueError('language: zero selected or incomplete fixed group gate')
                        count = sum(r.get('executed',0) for r in actual['results'])
                        check_minimum(minimums,label,count)
                        record['selected'] = count
                        total_selected += count
                    continue
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
                check_minimum(minimums, label, len(selected))
                expected, required = baseline_for(args.tier, label)
                record["added"] = validate_ids(ids, selected, expected, set() if args.filters else required)
                if not selected:
                    record["skip"] = "explicit scope matches no tests in this owner"
                    continue
                if not args.list_only:
                    if label in ('language','tt','tb'):
                        record['run_exit']=0
                        record['execution']='one native fixed-group union; legacy list is conservation audit only'
                        continue
                    report_dir = out / (label + "-reports")
                    report_dir.mkdir()
                    with preserve_reports(lang, report_dir):
                        code = command(["cargo", "nextest", "run", *options, "--no-fail-fast"], cwd,
                                       out / (label + "-run.log"), args.timeout)
                    record["run_exit"] = code
                    if code:
                        record["failed"] = failed_from_log(out / (label + "-run.log"))
                        raise ValueError(f"{label}: tests failed ({code}); stopping")
            if args.batch:
                legs_spec = batch_extra_legs(lang, ui, man)
                env = child_env(lang, ui, man)
                if not args.list_only:
                    receipt["man_bootstrap"] = man_bootstrap(legs_spec, lang, ui, out, args.timeout, env)
                for spec in legs_spec:
                    label = spec["leg"]
                    record = {"leg": label, "cwd": str(spec["cwd"]), "kind": spec["kind"]}
                    receipt["results"].append(record)
                    if spec["kind"] == "nextest":
                        options = spec["options"]
                        record["options"] = options
                        listing = out / (label + "-list.json")
                        with listing.open("w", encoding="utf-8") as stdout, (out / (label + "-build.log")).open("w", encoding="utf-8") as stderr:
                            proc = subprocess.Popen(["cargo", "nextest", "list", *options, "--message-format", "json"],
                                                    cwd=spec["cwd"], env=env, stdout=stdout, stderr=stderr,
                                                    start_new_session=os.name != "nt")
                            try:
                                code = proc.wait(timeout=args.timeout)
                            except (subprocess.TimeoutExpired, KeyboardInterrupt):
                                reap(proc)
                                raise
                        record["list_exit"] = code
                        if code:
                            raise ValueError(f"{label}: listing failed ({code})")
                        ids, selected = selection(json.loads(listing.read_text(encoding="utf-8")))
                        total_selected += len(selected)
                        record.update(ids=len(ids), selected=len(selected))
                        check_minimum(minimums, label, len(selected))
                        if not args.list_only:
                            code = command(["cargo", "nextest", "run", *options, "--no-fail-fast"], spec["cwd"],
                                           out / (label + "-run.log"), args.timeout, env)
                            record["run_exit"] = code
                            if code:
                                record["failed"] = failed_from_log(out / (label + "-run.log"))
                                raise ValueError(f"{label}: tests failed ({code}); stopping")
                    else:
                        record["command"] = ["cargo", *spec["list"]]
                        record["package"] = spec["package"]
                        listing = out / (label + "-list.txt")
                        with listing.open("w", encoding="utf-8") as stdout, (out / (label + "-build.log")).open("w", encoding="utf-8") as stderr:
                            proc = subprocess.Popen(["cargo", *spec["list"]], cwd=spec["cwd"], env=env,
                                                    stdout=stdout, stderr=stderr, start_new_session=os.name != "nt")
                            try:
                                code = proc.wait(timeout=args.timeout)
                            except (subprocess.TimeoutExpired, KeyboardInterrupt):
                                reap(proc)
                                raise
                        record["list_exit"] = code
                        if code:
                            raise ValueError(f"{label}: listing failed ({code})")
                        ids, selected = libtest_selection(listing.read_text(encoding="utf-8"), spec["package"])
                        total_selected += len(selected)
                        record.update(ids=len(ids), selected=len(selected), ignored=len(ids) - len(selected))
                        check_minimum(minimums, label, len(selected))
                        if not args.list_only and not (spec.get("manual") and not os.environ.get("AUTO_UI_TN_RUN")):
                            code = command(["cargo", *spec["run"]], spec["cwd"], out / (label + "-run.log"),
                                           args.timeout, env)
                            record["run_exit"] = code
                            if code:
                                record["failed"] = failed_from_log(out / (label + "-run.log"), spec["package"])
                                raise ValueError(f"{label}: tests failed ({code}); stopping")
                        elif spec.get("manual"):
                            record["skip"] = "manual native gate; run explicitly with AUTO_UI_TN_RUN=1 (recorded pending, not passed)"
            if not total_selected:
                raise ValueError("zero selected tests across all owners")
            after = {"language": identity(lang), "ui": identity(ui)}
            if man is not None:
                after["man"] = identity(man)
            if before != after:
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
