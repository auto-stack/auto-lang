"""Explicit source-bound preparation; builds are separate from warm test time.

PLAN-1060 T-26: every invocation used to re-run `cargo build --release` for each
declared tool (warm-tree measurement: 12 no-op builds = 4.35 s per invocation,
repaid by every module/consumer/review run). A bounded build receipt now lets an
invocation reuse an artifact it can prove is the same identity:

  identity = source_state (git HEAD + tracked diff + untracked crates/scripts
             inputs, the driver's own run provenance) + rustc -vV marker +
             the tool's toolchains.json entry + the resolved target dir.

Reuse additionally re-hashes every artifact. Any drift (source edit, toolchain
change, tool config change, missing/tampered binary) misses the receipt and
rebuilds. The receipt lives in the tool's target dir (git-ignored), never in a
source path.
"""
import hashlib
import shutil
import json
import os
from pathlib import Path
import subprocess
import time

from governance_io import load, write
from process_capture import capture

RECEIPT_SCHEMA = "test-tool-receipt/1"
RECEIPT_DIR = ".test-tool-receipts"
_RUSTC_MARKER = None


def toolchain_marker():
    """One process-wide rustc identity probe (cheaper than 12 no-op builds)."""
    global _RUSTC_MARKER
    if _RUSTC_MARKER is None:
        try:
            out = subprocess.check_output(["rustc", "-vV"], text=True,
                                          stderr=subprocess.DEVNULL)
        except (OSError, subprocess.SubprocessError):
            _RUSTC_MARKER = "unavailable"
        else:
            _RUSTC_MARKER = hashlib.sha256(out.encode("utf-8")).hexdigest()
    return _RUSTC_MARKER


def tool_identity(tool, target, source_state):
    return {
        "source_state": source_state or "unknown",
        "toolchain": toolchain_marker(),
        "config_sha256": hashlib.sha256(
            json.dumps(tool, sort_keys=True).encode("utf-8")).hexdigest(),
        "target": str(target),
    }


def receipt_path(target, name):
    return Path(target) / RECEIPT_DIR / (name + ".json")


def read_receipt(path, identity):
    if not path.is_file():
        return None
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None
    if document.get("schema") != RECEIPT_SCHEMA or document.get("identity") != identity:
        return None
    return document


def artifacts_match(document):
    artifacts = document.get("artifacts") or []
    if not artifacts:
        return False
    for row in artifacts:
        path = Path(row["path"])
        if not path.is_file():
            return False
        if hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]:
            return False
    return True


def pin_artifacts(target, identity, name, paths):
    """Copy build outputs into an identity-scoped directory.

    PLAN-1098 T-02/T-10: a receipt that points at `target/<profile>/<artifact>`
    is invalidated whenever another preparation rebuilds the same path (a
    different feature set or profile overwrites it), which then looks like
    "tampered" and forces a rebuild in the middle of a batch. The pin copy is
    never rebuilt, so one identity keeps exactly one verified artifact set.
    """
    digest = identity if isinstance(identity, str) else json.dumps(identity, sort_keys=True)
    key = hashlib.sha256(digest.encode()).hexdigest()[:16]
    pin_dir = target / "test-groups" / "prepared" / key / "tools" / name
    pin_dir.mkdir(parents=True, exist_ok=True)
    pinned = []
    for path in paths:
        destination = pin_dir / path.name
        shutil.copy2(path, destination)
        pinned.append(destination)
    return pinned


def configured_tools(root, debug_tools=False):
    document = load(root / "scripts/test-governance/toolchains.json")
    if document.get("schema") != "test-module-tools/1":
        raise ValueError("unsupported test toolchain schema")
    tools = document["tools"]
    if debug_tools:
        profiles = {"release": "debug", "runtime-release": "runtime-dev"}
        for tool in tools.values():
            tool["profile"] = profiles.get(tool["profile"], tool["profile"])
        tools["native-runtimes"]["warm_env"] = {"ACE_FIXED_TEST_RUNTIME_MODE": "prepare-debug"}
    return tools


def prepare(root, names, output, prepared, source_state=None, *, tools=None):
    if tools is None:
        tools = configured_tools(root)
    env={}
    receipts=[]
    for name in names:
        tool=tools[name]
        cwd=(root/tool["workspace"]).resolve()
        target=(root/tool["target"]).resolve()
        if not cwd.is_relative_to(root) or not target.is_relative_to(root):
            raise ValueError("test tool escapes authorized repository")
        identity = tool_identity(tool, target, source_state)
        reused = None
        if name not in prepared:
            cached = read_receipt(receipt_path(target, name), identity)
            if cached is not None and artifacts_match(cached):
                reused = cached
                prepared.add(name)
            else:
                command=["cargo","build","--locked","--offline","-p",tool["package"],"--target-dir",str(target)]
                if tool["profile"]=="release":command.append("--release")
                elif tool["profile"]!="debug":command += ["--profile",tool["profile"]]
                if tool.get("features"):command += ["--features",tool["features"]]
                for binary in tool["binaries"]:command += ["--bin",binary]
                started=time.monotonic()
                output.mkdir(parents=True,exist_ok=True)
                log=output/(name+"-build.log")
                build_env = dict(os.environ)
                build_env.pop('CARGO_TARGET_DIR', None)
                result = capture(command, cwd=cwd, env=build_env, timeout=1800, compiler_auxiliary_cleanup=True)
                log.write_bytes(result.stdout + result.stderr)
                if result.measurement is not None:
                    write(log.with_suffix('.meter.json'), result.measurement)
                if result.returncode:
                    raise ValueError(f"tool preparation failed: {name}; {log}")
                receipts.append({"tool":name,"command":command,"exit":0,"build_seconds":time.monotonic()-started,"log":str(log),
                                 "meter": result.measurement})
                prepared.add(name)
        paths=[target/tool["profile"]/(binary+(".exe" if os.name=="nt" else "")) for binary in tool["binaries"]]
        paths += [target/tool["profile"]/artifact for artifact in tool.get("artifacts",[])]
        if reused is not None:
            receipts.append({"tool":name,"reused":True,"identity":identity,
                             "artifacts":reused["artifacts"]})
        else:
            if not all(p.is_file() for p in paths):raise ValueError("prepared tool artifact missing")
            pinned = pin_artifacts(target, identity, name, paths)
            written = [{"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in pinned]
            write(receipt_path(target, name), {"schema": RECEIPT_SCHEMA, "tool": name,
                                               "identity": identity, "artifacts": written})
        # PLAN-1098 T-02: content-bound runtime builders cache by their own
        # identity stamp and only avoid a rebuild after the builder itself ran
        # once in this target tree. Warm them here so the run phase never
        # compiles; the builder still validates any reused library.
        warm = tool.get("warm")
        if warm:
            binary = target/tool["profile"]/(warm[0]+(".exe" if os.name=="nt" else ""))
            if not binary.is_file():
                raise ValueError(f"prepared warm binary missing: {binary}")
            started_warm = time.monotonic()
            output.mkdir(parents=True, exist_ok=True)
            warm_log = output/(name+"-warm.log")
            warm_env = dict(os.environ)
            warm_env.pop('CARGO_TARGET_DIR', None)
            warm_env.update(tool.get("warm_env", {}))
            warm_result = capture([str(binary), *[str(a) for a in warm[1:]]], cwd=cwd,
                                  env=warm_env, timeout=1800, compiler_auxiliary_cleanup=True)
            warm_log.write_bytes(warm_result.stdout + warm_result.stderr)
            if warm_result.measurement is not None:
                write(warm_log.with_suffix('.meter.json'), warm_result.measurement)
            if warm_result.returncode:
                raise ValueError(f"tool warm-up failed: {name}; {warm_log}")
            receipts.append({"tool":name+"#warm","command":[str(binary), *[str(a) for a in warm[1:]]],
                             "exit":0,"warm_seconds":time.monotonic()-started_warm,"log":str(warm_log)})
        effective = ([Path(row["path"]) for row in reused["artifacts"]] if reused is not None else pinned)
        for path in effective:
            receipts.append({"tool":name,"path":str(path),"sha256":hashlib.sha256(path.read_bytes()).hexdigest()})
        if tool.get("env"):env[tool["env"]]=str(effective[0])
        # PLAN-1094: a directory-backed env var (the first artifact's parent
        # — e.g. the av provider root holding the prepared closures).
        if tool.get("dir_env"):
            env[tool["dir_env"]] = str(effective[0].parent)
    return env,receipts


def case_tools(rows):
    names={"case-parent"}
    for row in rows:
        if row["adapter"] in ("ac","syntax"):names.add("ac")
        elif row["adapter"]=="probe":
            recipe=row.get("case_recipe") or ""
            matched=False
            for prefix,tool in (("ac-a2r/","a2x"),("ac-a2c/","a2x"),("ac-av/","av"),("ac/mir-native-collections-","mn-collections"),("ac/mir-native-","mn"),("cte/","cte"),("sema/","sema"),("abi/","abi")):
                if recipe.startswith(prefix):names.add(tool);matched=True;break
            if not matched:raise ValueError(f"unmapped probe tool recipe: {recipe}")
            if "av" in names:
                # PLAN-1094: the ordinary AV view binds registered prepared
                # providers (f0_text / py); prepare them with the worker.
                names.add("av-providers")
    return sorted(names)


def recipe_tools(recipe,rows):
    names=set(recipe.get("tools",[]))
    matches=[]
    for override in recipe.get("tool_overrides",[]):
        selected=[r for r in rows if r["binary"]==override["binary"] and r["test"].startswith(override["test_prefix"])]
        if selected:
            if len(selected)!=len(rows):raise ValueError("module mixes incompatible tool configurations; repartition")
            matches.append(override["tools"])
    if len(matches)>1:raise ValueError("overlapping tool configuration overrides")
    return sorted(matches[0] if matches else names)
