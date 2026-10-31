"""Native owner contracts for Web delivery, independent of retired scheduling."""
import os
import re
import subprocess
from pathlib import Path
from governance_io import load, machine_gate

DATA = Path(__file__).with_name('web-native-owner-contracts.json')

def owner_specs(roots):
    def expand(value):
        for name, root in roots.items():
            token = '{'+name+'}'
            if value.startswith(token):
                suffix = value[len(token):].lstrip('/\\')
                return str(root.joinpath(*re.split(r'[/\\]', suffix))) if suffix else str(root)
        return value
    result = {}
    for name, source in load(DATA)['owners'].items():
        spec = dict(source, cwd=roots[source['owner']])
        for key in ('options', 'list', 'run'):
            if key in spec: spec[key] = [expand(v) for v in spec[key]]
        result[name] = spec
    return result

def owner_baseline(name):
    if name != 'ui-full': raise ValueError('no baseline for owner: '+name)
    row = load(DATA)['ui_original']
    return set(row['required_listed']), set(row['required_active'])

def ui_original_baseline():
    row = load(DATA)['ui_original']
    return set(row['listed']), set(row['active'])

def minimums():
    return {k: {'minimum': v['minimum']} for k,v in load(DATA)['owners'].items() if 'minimum' in v}

def dirty_code_paths(path):
    return subprocess.check_output(["git", "-C", str(path), "diff", "--name-only", "HEAD", "--",
                                    "crates", "test", "experimental", "scripts", ".cargo", ".config", "Cargo.toml", "Cargo.lock"])


def require_clean_tree(name, path):
    if dirty_code_paths(path).strip():
        raise ValueError(f"frozen tree {name} has code/configuration WIP: {path}")


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


def libtest_original_ignores(ids, package, owner_listing):
    """libtest --list omits ignore flags; retain the same-source nextest flags.

    ui-full has already enumerated this compiler library in the same frozen
    batch. This changes applicability only, never the single-process UI run.
    """
    return {package + '\t' + name
            for binary, suite in owner_listing['rust-suites'].items() if binary == package
            for name, case in suite['testcases'].items()
            if case['ignored'] and package + '\t' + name in ids}


def validate_ids(ids, selected, expected, required_selected):
    missing = sorted(expected - ids)
    suppressed = sorted(required_selected - selected)
    if missing or suppressed:
        raise ValueError(f"baseline coverage lost: missing={len(missing)}, suppressed={len(suppressed)}")
    return sorted(ids - expected)


def check_minimum(legs, label, count):
    entry = legs.get(label)
    if not entry:
        return
    if count < entry["minimum"]:
        raise ValueError(f"{label}: selection below frozen minimum ({count} < {entry['minimum']})")


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
        code = command(cmd, cwd, out / f"bootstrap-{index}.log", timeout, env)
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

