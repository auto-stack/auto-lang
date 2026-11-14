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


def prepare(root, names, output, prepared, source_state=None):
    document=load(root/"scripts/test-governance/toolchains.json")
    if document.get("schema")!="test-module-tools/1":
        raise ValueError("unsupported test toolchain schema")
    env={}
    receipts=[]
    for name in names:
        tool=document["tools"][name]
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
            written = [{"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]
            write(receipt_path(target, name), {"schema": RECEIPT_SCHEMA, "tool": name,
                                               "identity": identity, "artifacts": written})
        for path in paths:
            receipts.append({"tool":name,"path":str(path),"sha256":hashlib.sha256(path.read_bytes()).hexdigest()})
        if tool.get("env"):env[tool["env"]]=str(paths[0])
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
