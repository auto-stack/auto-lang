"""Explicit source-bound preparation; builds are separate from warm test time."""
import hashlib
import os
from pathlib import Path
import time

from governance_io import load, write
from process_capture import capture


def prepare(root, names, output, prepared):
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
        if name not in prepared:
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
        if not all(p.is_file() for p in paths):raise ValueError("prepared tool artifact missing")
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
            for prefix,tool in (("ac-av/","av"),("ac/mir-native-collections-","mn-collections"),("ac/mir-native-","mn"),("cte/","cte"),("sema/","sema"),("abi/","abi")):
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
