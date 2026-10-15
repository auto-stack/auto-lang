#!/usr/bin/env python3
"""legacy-preservation.py — PLAN-784 旧仓保全工具（Category A）。

对既有 auto-ui / auto-man 仓做只读盘点、git bundle + 原始字节载荷保全、
legacy refs（分支+标签）锚定与恢复演练。设计约束（Plan 784 / Design 37 §8.0，
含 review R1 两项 blocking 修复）：

- 不改变原仓工作区：payload 永远写到仓外的独立恢复位置；
- **状态寻址快照（F784-R1-01）**：每次保全写入
  `<payload>/<repo>/snapshots/<snap-id>/`，snap-id 由仓状态（tip+WIP+保留文件
  内容 hash）派生；同态重跑=完整性校验通过后原样复用（零写入），状态变化=
  新增独立快照，任何既有快照永不被覆盖/删除；完整性不符=拒绝且不动旧快照；
- **原文件漂移检测（F784-R1-02）**：verify 对 tracked WIP、untracked、
  ignored-keep 的**原文件**统一校验存在性+hash，变化/删除即非零退出；
- 同名 ref 同 tip 幂等复用，异 tip 明确拒绝且原 ref 不动（exit 3）；
- 收据只写元信息（SHA256/路径/大小），payload 内容不进仓；
- 输出 JSON 不含时间戳/绝对检出路径抖动源：同输入重跑字节稳定。

v1 遗留布局（`<repo>/{bundle,wip,untracked,ignored,manifest.json}`）为
已冻结恢复点，本工具 v2 不读取也不改动它，仅在其旁新增 snapshots/。

用法：
  python scripts/legacy-preservation.py inventory  --repo PATH [--out FILE]
  python scripts/legacy-preservation.py preserve   --repo PATH --payload-dir DIR
                                                   [--ignored-keep RELPATH ...]
  python scripts/legacy-preservation.py anchor     --repo PATH --tip SHA
                                                   --branch NAME --tag NAME
  python scripts/legacy-preservation.py verify     --repo PATH --payload-dir DIR
                                                   --tip SHA --branch NAME --tag NAME
                                                   [--snapshot ID]
  python scripts/legacy-preservation.py recover-drill --repo PATH --payload-dir DIR
                                                   --scratch DIR [--keep] [--snapshot ID]
  python scripts/legacy-preservation.py snapshots  --repo PATH --payload-dir DIR
  python scripts/legacy-preservation.py drill      [--scratch DIR] [--keep]

退出码：0=通过；2=用法/环境；3=ref 冲突/锚点不匹配；4=载荷/恢复校验失败。
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

TOOL_VERSION = "legacy-preservation.py v2"
EXIT_OK = 0
EXIT_ENV = 2
EXIT_CONFLICT = 3
EXIT_PAYLOAD = 4


def git(repo: Path | None, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    argv = ["git"]
    if repo is not None:
        argv += ["-C", repo.as_posix()]
    argv += list(args)
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if check and proc.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed ({proc.returncode}): {proc.stderr.strip()}")
    return proc


def git_bytes(repo: Path | None, *args: str) -> bytes:
    """二进制保真通道：payload 副本/差异必须走原始字节，不经 text 解码。"""
    argv = ["git"]
    if repo is not None:
        argv += ["-C", repo.as_posix()]
    argv += list(args)
    proc = subprocess.run(argv, capture_output=True)
    if proc.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed ({proc.returncode}): {proc.stderr.decode(errors='replace').strip()}")
    return proc.stdout


def sha256_file(path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def repo_facts(repo: Path) -> dict:
    toplevel = Path(git(repo, "rev-parse", "--show-toplevel").stdout.strip())
    return {
        "name": toplevel.name,
        "toplevel": toplevel.as_posix(),
        "head": git(repo, "rev-parse", "HEAD").stdout.strip(),
        "branch": git(repo, "branch", "--show-current").stdout.strip(),
        "symref": git(repo, "symbolic-ref", "HEAD").stdout.strip(),
        "refs": {
            line.split()[0]: line.split()[1]
            for line in git(repo, "for-each-ref", "--format=%(refname) %(objectname)").stdout.splitlines()
            if line.strip()
        },
        "status_porcelain": git(repo, "status", "--porcelain=v1", "-uall").stdout.splitlines(),
        "ignored": git(repo, "ls-files", "--others", "--ignored", "--exclude-standard", "--directory")
        .stdout.splitlines(),
        "stash_count": len(git(repo, "stash", "list").stdout.splitlines()),
        "remotes": git(repo, "remote", "-v").stdout.splitlines(),
        "worktrees": [
            line.removeprefix("worktree ")
            for line in git(repo, "worktree", "list", "--porcelain").stdout.splitlines()
            if line.startswith("worktree ")
        ],
        "last_commit": git(repo, "log", "-1", "--format=%H%x00%an%x00%ad%x00%s").stdout.strip("\n"),
    }


def is_relative_to(child, parent) -> bool:
    try:
        Path(child).resolve().relative_to(Path(parent).resolve())
        return True
    except ValueError:
        return False


def rmtree_force(p) -> None:
    """Windows git 对象带只读属性，先去只读再删；失败抛出（不静默留残）。"""
    path = Path(p)
    if not path.exists():
        return

    def _onexc(fn, target, _exc):
        os.chmod(target, stat.S_IWRITE)
        fn(target)

    shutil.rmtree(path, onexc=_onexc)


def parse_status_entry(line: str) -> tuple[str, str]:
    # porcelain v1: XY <path>；重命名形如 "R  old -> new" 不在保全面（当前两仓无此形态），
    # 出现即按字面路径处理并在 manifest 留原始行。
    status = line[:2].strip() or "??"
    path = line[3:].strip()
    return status, path


# ---------------------------------------------------------------- inventory

def cmd_inventory(args) -> int:
    repo = Path(args.repo).resolve()
    facts = repo_facts(repo)
    emit(facts, args.out, label=f"inventory:{facts['name']}")
    return EXIT_OK


# ---------------------------------------------------------------- preserve (snapshot model)

def repo_payload_root(payload_dir: Path, repo_name: str) -> Path:
    return payload_dir / repo_name


def snapshot_layout(payload_dir: Path, repo_name: str, snap_id: str) -> dict:
    snap = repo_payload_root(payload_dir, repo_name) / "snapshots" / snap_id
    return {
        "snapshot": snap,
        "bundle": snap / "bundle" / f"{repo_name}-full.bundle",
        "wip": snap / "wip",
        "untracked": snap / "untracked",
        "ignored": snap / "ignored",
        "manifest": snap / "manifest.json",
    }


def latest_pointer(payload_dir: Path, repo_name: str) -> Path:
    return repo_payload_root(payload_dir, repo_name) / "LATEST"


def compute_state(repo: Path, facts: dict, ignored_keep: list[str]) -> dict:
    """从原仓当前状态派生快照身份：tip + **完整 refs→objectID 映射** + status +
    全部保留内容 hash（F784-R2-01：refs 影响 bundle 内容，必须参与身份——
    同 HEAD 仅增/移/删 tag/branch 时也要产生新快照，否则复用旧 bundle 会缺 ref）。
    bundle 字节不参与（同 refs 的两次 bundle 打包字节可差，不构成状态差）。"""
    toplevel = Path(facts["toplevel"])
    wip_state = []
    for line in facts["status_porcelain"]:
        status, path = parse_status_entry(line)
        if status == "??":
            continue
        entry = {"status_line": line, "status": status, "path": path}
        if status != "D":
            entry["work_sha256"] = sha256_file(toplevel / path)
        entry["head_sha256"] = sha256_bytes(git_bytes(repo, "show", f"HEAD:{path}"))
        diff = git_bytes(repo, "diff", "--binary", "HEAD", "--", path)
        entry["diff_sha256"] = sha256_bytes(diff) if diff else None
        wip_state.append(entry)
    untracked_state = [
        {"path": parse_status_entry(l)[1], "sha256": sha256_file(toplevel / parse_status_entry(l)[1])}
        for l in facts["status_porcelain"] if parse_status_entry(l)[0] == "??"
    ]
    ignored_state = [
        {"path": rel, "sha256": sha256_file(toplevel / rel)} for rel in ignored_keep
    ]
    return {
        "head": facts["head"],
        "refs": facts["refs"],
        "status_porcelain": facts["status_porcelain"],
        "wip": wip_state,
        "untracked": untracked_state,
        "ignored_keep": ignored_state,
    }


def snapshot_id_of(state: dict) -> str:
    return sha256_bytes(canonical_json(state).encode("utf-8"))[:16]


def write_snapshot_payloads(repo: Path, facts: dict, state: dict, paths: dict) -> None:
    """把 state 声明的载荷按原始字节写入快照目录（staging 内调用）。"""
    toplevel = Path(facts["toplevel"])
    for entry in state["wip"]:
        path, safe = entry["path"], entry["path"].replace("/", "__")
        if "work_sha256" in entry:
            dst = paths["wip"] / f"{safe}.work"
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(toplevel / path, dst)
        head_dst = paths["wip"] / f"{safe}.head"
        head_dst.parent.mkdir(parents=True, exist_ok=True)
        head_dst.write_bytes(git_bytes(repo, "show", f"HEAD:{path}"))
        if entry.get("diff_sha256"):
            diff_dst = paths["wip"] / f"{safe}.diff"
            diff_dst.parent.mkdir(parents=True, exist_ok=True)
            diff_dst.write_bytes(git_bytes(repo, "diff", "--binary", "HEAD", "--", path))
    for entry in state["untracked"]:
        dst = paths["untracked"] / entry["path"]
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(toplevel / entry["path"], dst)
    for entry in state["ignored_keep"]:
        dst = paths["ignored"] / entry["path"]
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(toplevel / entry["path"], dst)


def bundle_refs(repo: Path, bundle_file: Path) -> dict:
    """bundle 实际承载的 ref→sha（F784-R2-01 oracle：不能只凭 manifest/hash 自证）。"""
    out = git(repo, "bundle", "list-heads", bundle_file.as_posix()).stdout
    heads = {}
    for line in out.splitlines():
        parts = line.split()
        if len(parts) == 2:
            heads[parts[1]] = parts[0]
    return heads


def bundle_covers_refs(repo: Path, bundle_file: Path, refs: dict) -> list[str]:
    """断言 bundle 实际可恢复 refs 覆盖给定映射（HEAD 键为 bundle 附加，不在比对面）。"""
    heads = bundle_refs(repo, bundle_file)
    missing = []
    for ref, sha in refs.items():
        got = heads.get(ref)
        if got != sha:
            missing.append(f"{ref}: bundle={'无' if got is None else got} 期望={sha}")
    return missing


def check_snapshot_integrity(repo: Path, paths: dict, state: dict) -> list[str]:
    """既有/staging 快照与期望状态的一致性；返回 failures（非空=拒绝，且不写任何东西）。"""
    failures = []
    if not paths["manifest"].is_file():
        return ["快照缺 manifest"]
    manifest = json.loads(paths["manifest"].read_text(encoding="utf-8"))
    if manifest.get("state") != state:
        failures.append("快照 manifest 状态与当前派生状态不符（snap-id 冲突或载荷被改动）")
    verify = git(repo, "bundle", "verify", paths["bundle"].as_posix(), check=False)
    if verify.returncode != 0:
        failures.append("快照 bundle verify 失败")
    if paths["bundle"].is_file() and sha256_file(paths["bundle"]) != manifest.get("bundle", {}).get("sha256"):
        failures.append("快照 bundle sha256 与其 manifest 不一致")
    # bundle 实际承载 refs 必须覆盖状态中的完整 refs 映射。
    for line in bundle_covers_refs(repo, paths["bundle"], state["refs"]):
        failures.append(f"bundle 未覆盖 ref: {line}")
    for entry in state["wip"]:
        safe = entry["path"].replace("/", "__")
        if "work_sha256" in entry:
            p = paths["wip"] / f"{safe}.work"
            if not p.is_file() or sha256_file(p) != entry["work_sha256"]:
                failures.append(f"快照载荷不符: wip/{safe}.work")
        p = paths["wip"] / f"{safe}.head"
        if not p.is_file() or sha256_file(p) != entry["head_sha256"]:
            failures.append(f"快照载荷不符: wip/{safe}.head")
        if entry.get("diff_sha256"):
            p = paths["wip"] / f"{safe}.diff"
            if not p.is_file() or sha256_file(p) != entry["diff_sha256"]:
                failures.append(f"快照载荷不符: wip/{safe}.diff")
    for key, sub in (("untracked", "untracked"), ("ignored_keep", "ignored")):
        for entry in state[key]:
            p = paths[sub] / entry["path"]
            if not p.is_file() or sha256_file(p) != entry["sha256"]:
                failures.append(f"快照载荷不符: {sub}/{entry['path']}")
    return failures


def create_bundle(repo: Path, bundle_file: Path) -> dict:
    bundle_file.parent.mkdir(parents=True, exist_ok=True)
    # --all 覆盖 refs/*（heads/tags/remotes），显式 HEAD 让 clone 后默认分支可恢复。
    proc = git(repo, "bundle", "create", bundle_file.as_posix(), "--all", "HEAD", check=False)
    if proc.returncode != 0:
        proc = git(repo, "bundle", "create", bundle_file.as_posix(), "--all", check=False)
        if proc.returncode != 0:
            raise RuntimeError(f"bundle create failed: {proc.stderr.strip()}")
        head_included = False
    else:
        head_included = True
    verify = git(repo, "bundle", "verify", bundle_file.as_posix(), check=False)
    if verify.returncode != 0:
        raise RuntimeError(f"bundle verify failed on fresh bundle: {verify.stderr.strip()}")
    heads = git(repo, "bundle", "list-heads", bundle_file.as_posix()).stdout.splitlines()
    return {
        "file": bundle_file.name,
        "sha256": sha256_file(bundle_file),
        "bytes": bundle_file.stat().st_size,
        "verify": "ok",
        "head_included": head_included,
        "refs_in_bundle": len([l for l in heads if l.strip()]),
    }


def cmd_preserve(args) -> int:
    repo = Path(args.repo).resolve()
    facts = repo_facts(repo)
    payload_dir = Path(args.payload_dir).resolve()
    if is_relative_to(payload_dir, facts["toplevel"]) or is_relative_to(Path(facts["toplevel"]), payload_dir):
        print(f"error: payload 目录不得位于目标仓内或覆盖目标仓: {payload_dir}", file=sys.stderr)
        return EXIT_ENV
    for rel in args.ignored_keep or []:
        if not (Path(facts["toplevel"]) / rel).is_file():
            print(f"error: --ignored-keep 指定的文件不存在: {rel}", file=sys.stderr)
            return EXIT_ENV

    state = compute_state(repo, facts, list(args.ignored_keep or []))
    snap_id = snapshot_id_of(state)
    paths = snapshot_layout(payload_dir, facts["name"], snap_id)

    # 既有同 id 快照：完整性校验通过才复用；失败拒绝且零写入（保护旧恢复点）。
    if paths["snapshot"].exists():
        failures = check_snapshot_integrity(repo, paths, state)
        if failures:
            for f in failures:
                print(f"SNAPSHOT-INTEGRITY-REFUSED: {f}", file=sys.stderr)
            return EXIT_PAYLOAD
        latest_pointer(payload_dir, facts["name"]).write_text(snap_id + "\n", encoding="utf-8")
        print(json.dumps({
            "repo": facts["name"], "head": facts["head"], "action": "reuse",
            "snapshot": snap_id, "manifest": paths["manifest"].as_posix(),
            "wip": len(state["wip"]), "untracked": len(state["untracked"]),
            "ignored_keep": len(state["ignored_keep"]),
        }, ensure_ascii=False, sort_keys=True))
        return EXIT_OK

    # 新快照：先写 staging，全部成功后原子 rename；中途失败只清理 staging。
    staging = paths["snapshot"].parent / f".staging-{snap_id}-{os.getpid()}"
    rmtree_force(staging)
    staging_paths = {
        "snapshot": staging,
        "bundle": staging / "bundle" / f"{facts['name']}-full.bundle",
        "wip": staging / "wip", "untracked": staging / "untracked",
        "ignored": staging / "ignored", "manifest": staging / "manifest.json",
    }
    try:
        bundle_info = create_bundle(repo, staging_paths["bundle"])
        uncovered = bundle_covers_refs(repo, staging_paths["bundle"], facts["refs"])
        if uncovered:
            raise RuntimeError("新 bundle 未覆盖全部当前 refs: " + "; ".join(uncovered[:3]))
        write_snapshot_payloads(repo, facts, state, staging_paths)
        manifest = {
            "tool": TOOL_VERSION,
            "state_schema": 2,
            "snapshot_id": snap_id,
            "repo": {k: facts[k] for k in ("name", "toplevel", "head", "branch", "symref")},
            "state": state,
            "bundle": bundle_info,
            "ignored_listing": facts["ignored"],
            "stash_count": facts["stash_count"],
            "refs_frozen": facts["refs"],
            "payload_root": payload_dir.as_posix(),
        }
        staging_paths["manifest"].write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        # 落位前再核一遍 staging 与状态一致（防写入过程竞态）。
        chk = check_snapshot_integrity(repo, staging_paths, state)
        if chk:
            raise RuntimeError("staging 自检失败: " + "; ".join(chk[:3]))
        paths["snapshot"].parent.mkdir(parents=True, exist_ok=True)
        os.rename(staging.as_posix(), paths["snapshot"].as_posix())
    except Exception as e:
        rmtree_force(staging)
        print(f"error: 快照写入失败（已丢弃 staging，未触碰任何既有快照）: {e}", file=sys.stderr)
        return EXIT_PAYLOAD
    latest_pointer(payload_dir, facts["name"]).write_text(snap_id + "\n", encoding="utf-8")
    print(json.dumps({
        "repo": facts["name"], "head": facts["head"], "action": "create",
        "snapshot": snap_id, "bundle_sha256": bundle_info["sha256"],
        "manifest": paths["manifest"].as_posix(),
        "wip": len(state["wip"]), "untracked": len(state["untracked"]),
        "ignored_keep": len(state["ignored_keep"]),
    }, ensure_ascii=False, sort_keys=True))
    return EXIT_OK


def load_snapshot(payload_dir: Path, facts: dict, snapshot: str | None) -> tuple[str, dict, dict]:
    """解析快照引用（缺省=LATEST）；返回 (snap_id, paths, manifest)。"""
    snap_id = snapshot
    if snap_id is None:
        ptr = latest_pointer(payload_dir, facts["name"])
        if not ptr.is_file():
            raise RuntimeError(f"LATEST 指针缺失: {ptr}（--snapshot 显式指定或先 preserve）")
        snap_id = ptr.read_text(encoding="utf-8").strip()
    paths = snapshot_layout(payload_dir, facts["name"], snap_id)
    if not paths["manifest"].is_file():
        raise RuntimeError(f"快照不存在: {snap_id}")
    return snap_id, paths, json.loads(paths["manifest"].read_text(encoding="utf-8"))


# ---------------------------------------------------------------- anchor

class ConflictError(RuntimeError):
    pass


def anchor_repo(repo: Path, tip: str, branch: str, tag: str, head_must_match: bool = True):
    """幂等锚定。返回 (actions dict)。冲突时抛 ConflictError（原 ref 不动）。"""
    facts = repo_facts(repo)
    if head_must_match and facts["head"] != tip:
        raise ConflictError(f"HEAD={facts['head'][:12]}… 与期望 tip={tip[:12]}… 不一致，拒绝锚定（仓已被他人推进？）")
    actions = {}
    for kind, name in (("branch", branch), ("tag", tag)):
        full = f"refs/heads/{name}" if kind == "branch" else f"refs/tags/{name}"
        cur = facts["refs"].get(full)
        if cur is None:
            if kind == "branch":
                git(repo, "branch", name, tip)
            else:
                git(repo, "tag", name, tip)
            actions[kind] = {"name": name, "action": "create", "sha": tip}
        elif cur == tip:
            actions[kind] = {"name": name, "action": "reuse", "sha": cur}
        else:
            raise ConflictError(
                f"{full} 已存在于不同 tip（{cur[:12]}… != 期望 {tip[:12]}…），按合同不强制移动"
            )
    return actions


def cmd_anchor(args) -> int:
    repo = Path(args.repo).resolve()
    try:
        actions = anchor_repo(repo, args.tip, args.branch, args.tag)
    except ConflictError as e:
        print(f"CONFLICT: {e}", file=sys.stderr)
        return EXIT_CONFLICT
    print(json.dumps({"repo": repo_facts(repo)["name"], "tip": args.tip, **actions},
                     ensure_ascii=False, sort_keys=True))
    return EXIT_OK


# ---------------------------------------------------------------- verify

def cmd_verify(args) -> int:
    repo = Path(args.repo).resolve()
    payload_dir = Path(args.payload_dir).resolve()
    facts = repo_facts(repo)
    try:
        snap_id, paths, manifest = load_snapshot(payload_dir, facts, getattr(args, "snapshot", None))
    except RuntimeError as e:
        print(json.dumps({"repo": facts["name"], "ok": False, "failures": [str(e)]},
                         ensure_ascii=False, sort_keys=True))
        return EXIT_PAYLOAD
    state = manifest["state"]
    failures = []

    # 原仓不变量：HEAD/tip、主检出分支、status 与快照时一致。
    if facts["head"] != args.tip:
        failures.append(f"HEAD 漂移: {facts['head']} != {args.tip}")
    if facts["symref"] != manifest["repo"]["symref"]:
        failures.append(f"symref 变化: {facts['symref']} != {manifest['repo']['symref']}")
    if facts["status_porcelain"] != state["status_porcelain"]:
        failures.append(f"工作区 status 变化: {facts['status_porcelain']} != {state['status_porcelain']}")

    # refs：legacy 锚点在 tip 上；快照冻结的既有 refs 未被移动。
    for full, sha in ((f"refs/heads/{args.branch}", args.tip), (f"refs/tags/{args.tag}", args.tip)):
        cur = facts["refs"].get(full)
        if cur != sha:
            failures.append(f"{full} 不在锚定 tip: {cur} != {sha}")
    for full, sha in manifest["refs_frozen"].items():
        if facts["refs"].get(full) != sha:
            failures.append(f"保全 refs 漂移: {full}")

    # bundle：可校验 + hash 一致 + **实际承载 refs 覆盖冻结映射**（F784-R2-01）。
    verify = git(repo, "bundle", "verify", paths["bundle"].as_posix(), check=False)
    if verify.returncode != 0:
        failures.append("bundle verify 失败")
    if sha256_file(paths["bundle"]) != manifest["bundle"]["sha256"]:
        failures.append("bundle sha256 与 manifest 不一致")
    for line in bundle_covers_refs(repo, paths["bundle"], state["refs"]):
        failures.append(f"bundle 未覆盖 ref: {line}")

    # 载荷文件：存在 + hash 与快照一致。
    for entry in state["wip"]:
        safe = entry["path"].replace("/", "__")
        for key in ("work", "head", "diff"):
            want = entry.get(f"{key}_sha256")
            if want is None:
                continue
            p = paths["wip"] / f"{safe}.{key}"
            if not p.is_file():
                failures.append(f"载荷缺失: {p.relative_to(paths['snapshot']).as_posix()}")
            elif sha256_file(p) != want:
                failures.append(f"载荷 hash 不符: {p.relative_to(paths['snapshot']).as_posix()}")
    for key, sub in (("untracked", "untracked"), ("ignored_keep", "ignored")):
        for entry in state[key]:
            p = paths[sub] / entry["path"]
            if not p.is_file():
                failures.append(f"载荷缺失: {p.relative_to(paths['snapshot']).as_posix()}")
            elif sha256_file(p) != entry["sha256"]:
                failures.append(f"载荷 hash 不符: {p.relative_to(paths['snapshot']).as_posix()}")

    # 原文件（F784-R1-02）：tracked WIP、untracked、ignored-keep 统一存在性+hash 校验。
    toplevel = Path(facts["toplevel"])
    for entry in state["wip"]:
        if "work_sha256" not in entry:
            continue
        src = toplevel / entry["path"]
        if not src.is_file():
            failures.append(f"原文件已删除: {entry['path']}")
        elif sha256_file(src) != entry["work_sha256"]:
            failures.append(f"原工作文件已变: {entry['path']}")
    for key in ("untracked", "ignored_keep"):
        for entry in state[key]:
            src = toplevel / entry["path"]
            if not src.is_file():
                failures.append(f"原文件已删除: {entry['path']}（{key}）")
            elif sha256_file(src) != entry["sha256"]:
                failures.append(f"原文件已变: {entry['path']}（{key}）")

    ok = not failures
    print(json.dumps({"repo": facts["name"], "snapshot": snap_id, "ok": ok, "failures": failures},
                     ensure_ascii=False, sort_keys=True))
    return EXIT_OK if ok else EXIT_PAYLOAD


# ---------------------------------------------------------------- recover-drill (V01)

def cmd_recover_drill(args) -> int:
    repo = Path(args.repo).resolve()
    payload_dir = Path(args.payload_dir).resolve()
    facts = repo_facts(repo)
    try:
        snap_id, paths, manifest = load_snapshot(payload_dir, facts, getattr(args, "snapshot", None))
    except RuntimeError as e:
        print(json.dumps({"repo": facts["name"], "ok": False, "failures": [str(e)]},
                         ensure_ascii=False, sort_keys=True))
        return EXIT_PAYLOAD
    scratch = Path(args.scratch).resolve() if args.scratch else Path(tempfile.mkdtemp(prefix="p784-drill-"))
    scratch.mkdir(parents=True, exist_ok=True)
    drill_dir = scratch / f"{facts['name']}-recover"
    if drill_dir.exists():
        rmtree_force(drill_dir)
    drill_dir.mkdir(parents=True)

    git(None, "init", "-q", "--bare", drill_dir.as_posix())
    fetch = git(drill_dir, "fetch", "-q", paths["bundle"].as_posix(), "refs/*:refs/*", check=False)
    if fetch.returncode != 0:
        failures = [f"从 bundle fetch 失败: {fetch.stderr.strip()}"]
    else:
        drilled_refs = {
            line.split()[0]: line.split()[1]
            for line in git(drill_dir, "for-each-ref", "--format=%(refname) %(objectname)").stdout.splitlines()
            if line.strip()
        }
        failures = []
        # 快照记录的 refs 应全部可恢复且 sha 相等。
        for full, sha in manifest["refs_frozen"].items():
            got = drilled_refs.get(full)
            if got != sha:
                failures.append(f"恢复 ref 不符: {full} got={got} want={sha}")
        # tip commit/tree 相等。
        src_tree = git(repo, "rev-parse", f"{facts['head']}^{{tree}}").stdout.strip()
        got_tree = git(drill_dir, "rev-parse", f"{facts['head']}^{{tree}}", check=False)
        if got_tree.returncode != 0 or got_tree.stdout.strip() != src_tree:
            failures.append(f"恢复 tip tree 不符: {got_tree.stdout.strip()} != {src_tree}")
        # 旧 tag 可追溯。
        for t in [r for r in manifest["refs_frozen"] if r.startswith("refs/tags/")]:
            if t not in drilled_refs:
                failures.append(f"旧 tag 未恢复: {t}")

    if not args.keep:
        rmtree_force(drill_dir)
    ok = not failures
    print(json.dumps({
        "repo": facts["name"], "snapshot": snap_id, "ok": ok, "failures": failures,
        "refs_recovered": len(manifest["refs_frozen"]) - len(failures),
        "old_tags": len([r for r in manifest["refs_frozen"] if r.startswith("refs/tags/")]),
        "tree": git(repo, "rev-parse", f"{facts['head']}^{{tree}}").stdout.strip(),
    }, ensure_ascii=False, sort_keys=True))
    return EXIT_OK if ok else EXIT_PAYLOAD


# ---------------------------------------------------------------- snapshots listing

def cmd_snapshots(args) -> int:
    repo = Path(args.repo).resolve()
    payload_dir = Path(args.payload_dir).resolve()
    facts = repo_facts(repo)
    root = repo_payload_root(payload_dir, facts["name"])
    snaps = {}
    if (root / "snapshots").is_dir():
        for d in sorted((root / "snapshots").iterdir()):
            if d.is_dir() and not d.name.startswith("."):
                m = d / "manifest.json"
                if m.is_file():
                    mm = json.loads(m.read_text(encoding="utf-8"))
                    snaps[d.name] = {"head": mm["repo"]["head"], "bundle_sha256": mm["bundle"]["sha256"]}
    ptr = latest_pointer(payload_dir, facts["name"])
    print(json.dumps({
        "repo": facts["name"],
        "latest": ptr.read_text(encoding="utf-8").strip() if ptr.is_file() else None,
        "snapshots": snaps,
        "v1_legacy_layout_present": (root / "manifest.json").is_file(),
    }, ensure_ascii=False, sort_keys=True))
    return EXIT_OK


# ---------------------------------------------------------------- drill (V03 + R1 反控)

def make_tmp_repo(scratch: Path, name: str) -> tuple[Path, str]:
    d = scratch / name
    if d.exists():
        rmtree_force(d)
    d.mkdir(parents=True)
    git(None, "init", "-q", d.as_posix())
    git(d, "config", "user.email", "drill@local")
    git(d, "config", "user.name", "p784-drill")
    (d / "f.txt").write_text("A\n", encoding="utf-8")
    git(d, "add", "f.txt")
    git(d, "commit", "-q", "-m", "A")
    tip = git(d, "rev-parse", "HEAD").stdout.strip()
    return d, tip


def snapshot_tree_hash(snap_dir: Path) -> str:
    """快照目录全文件（路径+内容）聚合 hash：证明"旧快照字节未变"。"""
    h = hashlib.sha256()
    for p in sorted(snap_dir.rglob("*")):
        if p.is_file():
            h.update(p.relative_to(snap_dir).as_posix().encode("utf-8"))
            h.update(sha256_file(p).encode("ascii"))
    return h.hexdigest()


def run_preserve(repo: Path, payload_dir: Path, ignored_keep: list[str] | None = None) -> int:
    ns = argparse.Namespace(repo=repo.as_posix(), payload_dir=payload_dir.as_posix(),
                            ignored_keep=ignored_keep or [])
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(io.StringIO()):
        return cmd_preserve(ns)


def run_verify(repo: Path, payload_dir: Path, tip: str, branch: str, tag: str) -> tuple[int, dict]:
    ns = argparse.Namespace(repo=repo.as_posix(), payload_dir=payload_dir.as_posix(),
                            tip=tip, branch=branch, tag=tag, snapshot=None)
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        code = cmd_verify(ns)
    return code, json.loads(buf.getvalue())


def latest_snapshot_id(payload_dir: Path, name: str) -> str:
    return latest_pointer(payload_dir, name).read_text(encoding="utf-8").strip()


def fetch_refs_from_bundle(bundle: Path, scratch: Path, name: str) -> dict:
    """真实恢复 oracle：bare fetch bundle 后的 ref→sha 全表。"""
    bare = scratch / name
    if bare.exists():
        rmtree_force(bare)
    git(None, "init", "-q", "--bare", bare.as_posix())
    git(bare, "fetch", "-q", bundle.as_posix(), "refs/*:refs/*")
    return {
        line.split()[0]: line.split()[1]
        for line in git(bare, "for-each-ref", "--format=%(refname) %(objectname)").stdout.splitlines()
        if line.strip()
    }


def cmd_drill(args) -> int:
    scratch = Path(args.scratch).resolve() if args.scratch else Path(tempfile.mkdtemp(prefix="p784-drill-"))
    scratch.mkdir(parents=True, exist_ok=True)
    results = []

    # D1 同 tip 幂等：第二次 anchor 必须 reuse 且 ref 不动。
    d, tip_a = make_tmp_repo(scratch, "d1-idempotent")
    a1 = anchor_repo(d, tip_a, "legacy/pre-v06-extraction", "legacy-pre-v06-aaaa1111")
    a2 = anchor_repo(d, tip_a, "legacy/pre-v06-extraction", "legacy-pre-v06-aaaa1111")
    refs_now = repo_facts(d)["refs"]
    ok1 = (a1["branch"]["action"] == "create" and a2["branch"]["action"] == "reuse"
           and a2["tag"]["action"] == "reuse"
           and refs_now["refs/heads/legacy/pre-v06-extraction"] == tip_a
           and refs_now["refs/tags/legacy-pre-v06-aaaa1111"] == tip_a)
    results.append({"control": "D1-same-tip-idempotent", "ok": ok1})

    # D2 异 tip 冲突：仓推进到 B 后，同名 ref 仍指 A → 明确拒绝、原 ref 不动。
    (d / "f.txt").write_text("B\n", encoding="utf-8")
    git(d, "add", "f.txt")
    git(d, "commit", "-q", "-m", "B")
    tip_b = git(d, "rev-parse", "HEAD").stdout.strip()
    try:
        anchor_repo(d, tip_b, "legacy/pre-v06-extraction", "legacy-pre-v06-aaaa1111")
        ok2, why2 = False, "conflict 未被拒绝"
    except ConflictError as e:
        refs_now = repo_facts(d)["refs"]
        ok2 = (refs_now["refs/heads/legacy/pre-v06-extraction"] == tip_a
               and refs_now["refs/tags/legacy-pre-v06-aaaa1111"] == tip_a
               and "不同 tip" in str(e))
        why2 = str(e)
    results.append({"control": "D2-conflict-reject-no-move", "ok": ok2, "message": why2})

    # D3 HEAD 漂移拒绝：HEAD 仍在 A 却要求锚 B。
    d3, tip_c = make_tmp_repo(scratch, "d3-head-drift")
    anchor_repo(d3, tip_c, "legacy/pre-v06-extraction", "legacy-pre-v06-bbbb2222")
    try:
        anchor_repo(d3, "0" * 40, "legacy/pre-v06-extraction", "legacy-pre-v06-bbbb2222")
        ok3, why3 = False, "HEAD 漂移未被拒绝"
    except ConflictError as e:
        ok3, why3 = "HEAD" in str(e), str(e)
    results.append({"control": "D3-head-drift-reject", "ok": ok3, "message": why3})

    # D4 preserve 快照化：同 HEAD 下 dirty 前后两次保全=两个独立快照。
    d4, tip_d = make_tmp_repo(scratch, "d4-preserve-idempotent")
    payload4 = scratch / "d4-payload"
    rmtree_force(payload4)  # 控制间隔离：payload 不跨 drill 运行累积
    p4a = run_preserve(d4, payload4)
    id4a = latest_snapshot_id(payload4, d4.name)
    (d4 / "f.txt").write_text("dirty\n", encoding="utf-8")
    p4b = run_preserve(d4, payload4)
    id4b = latest_snapshot_id(payload4, d4.name)
    m2 = json.loads((payload4 / d4.name / "snapshots" / id4b / "manifest.json").read_text(encoding="utf-8"))
    ok4 = (p4a == 0 and p4b == 0 and id4a != id4b and len(m2["state"]["wip"]) == 1)
    results.append({"control": "D4-preserve-snapshot-split", "ok": ok4})

    # D5（F784-R1-01）A→B 重复保全：旧快照必须原样保留且可读出 A 字节。
    # 控制文件一律 write_bytes：文本模式在 Windows 落 CRLF，字节断言必须自控。
    d5, tip5 = make_tmp_repo(scratch, "d5-snapshot-retention")
    payload5 = scratch / "d5-payload"
    rmtree_force(payload5)  # 控制间隔离：payload 不跨 drill 运行累积
    (d5 / "f.txt").write_bytes(b"version-A\n")
    p5a = run_preserve(d5, payload5)
    id_a = latest_snapshot_id(payload5, d5.name)
    snap_a = payload5 / d5.name / "snapshots" / id_a
    hash_a_before = snapshot_tree_hash(snap_a)
    (d5 / "f.txt").write_bytes(b"version-B\n")
    p5b = run_preserve(d5, payload5)
    id_b = latest_snapshot_id(payload5, d5.name)
    snap_b = payload5 / d5.name / "snapshots" / id_b
    hash_a_after = snapshot_tree_hash(snap_a)
    snap_dirs = [x for x in (payload5 / d5.name / "snapshots").iterdir() if not x.name.startswith(".")]
    ok5 = (p5a == 0 and p5b == 0 and id_a != id_b
           and hash_a_before == hash_a_after
           and (snap_a / "wip" / "f.txt.work").read_bytes() == b"version-A\n"
           and (snap_b / "wip" / "f.txt.work").read_bytes() == b"version-B\n"
           and len(snap_dirs) == 2)
    # 同态第三次重跑 = reuse，不新增快照。
    p5c = run_preserve(d5, payload5)
    snap_dirs_after = [x for x in (payload5 / d5.name / "snapshots").iterdir() if not x.name.startswith(".")]
    ok5 = ok5 and p5c == 0 and len(snap_dirs_after) == 2 and latest_snapshot_id(payload5, d5.name) == id_b
    results.append({"control": "D5-rerun-retains-prior-snapshot", "ok": ok5,
                    "snapshots": [id_a, id_b]})

    # D6（F784-R1-02）保留原文件全矩阵：wip/untracked/ignored 漂移或删除必须
    # 判红，恢复后复绿，快照备份字节不动。
    d6, tip6 = make_tmp_repo(scratch, "d6-ignored-drift")
    payload6 = scratch / "d6-payload"
    rmtree_force(payload6)  # 控制间隔离：payload 不跨 drill 运行累积
    (d6 / ".gitignore").write_bytes(b"private.cfg\n")
    (d6 / "private.cfg").write_bytes(b"secret-v1\n")
    (d6 / "u.txt").write_bytes(b"u-v1\n")
    git(d6, "add", ".gitignore")
    git(d6, "commit", "-q", "-m", "ignore")
    (d6 / "f.txt").write_bytes(b"dirty-v1\n")  # tracked WIP
    tip6 = git(d6, "rev-parse", "HEAD").stdout.strip()
    p6 = run_preserve(d6, payload6, ignored_keep=["private.cfg"])
    anchor_repo(d6, tip6, "legacy/pre-v06-extraction", "legacy-pre-v06-cccc3333")
    v_ok, v_out = run_verify(d6, payload6, tip6, "legacy/pre-v06-extraction", "legacy-pre-v06-cccc3333")
    pos = (p6 == 0 and v_ok == 0 and v_out["ok"] is True)

    def expect_fail(mutate, needle) -> bool:
        mutate()
        code, out = run_verify(d6, payload6, tip6, "legacy/pre-v06-extraction", "legacy-pre-v06-cccc3333")
        return code != 0 and any(needle in f for f in out["failures"])

    drift_ign = expect_fail(lambda: (d6 / "private.cfg").write_bytes(b"secret-v2\n"), "原文件已变: private.cfg")
    (d6 / "private.cfg").write_bytes(b"secret-v1\n")
    drift_wip = expect_fail(lambda: (d6 / "f.txt").write_bytes(b"dirty-v2\n"), "原工作文件已变: f.txt")
    (d6 / "f.txt").write_bytes(b"dirty-v1\n")
    del_unt = expect_fail(lambda: (d6 / "u.txt").unlink(), "原文件已删除: u.txt")
    (d6 / "u.txt").write_bytes(b"u-v1\n")
    del_ign = expect_fail(lambda: (d6 / "private.cfg").unlink(), "原文件已删除: private.cfg")
    (d6 / "private.cfg").write_bytes(b"secret-v1\n")
    v_green, v_green_out = run_verify(d6, payload6, tip6, "legacy/pre-v06-extraction", "legacy-pre-v06-cccc3333")
    snap6 = payload6 / d6.name / "snapshots" / latest_snapshot_id(payload6, d6.name)
    kept = ((snap6 / "ignored" / "private.cfg").read_bytes() == b"secret-v1\n"
            and (snap6 / "wip" / "f.txt.work").read_bytes() == b"dirty-v1\n"
            and (snap6 / "untracked" / "u.txt").read_bytes() == b"u-v1\n")
    results.append({"control": "D6-ignored-original-drift-detected", "ok": (
        pos and drift_ign and drift_wip and del_unt and del_ign and kept and v_green == 0)})

    # D7（F784-R1-01 边）快照 bundle 损坏：同状态重跑 preserve 必须拒绝且不改写快照。
    d7, tip7 = make_tmp_repo(scratch, "d7-corrupt-bundle")
    payload7 = scratch / "d7-payload"
    rmtree_force(payload7)  # 控制间隔离：payload 不跨 drill 运行累积
    (d7 / "f.txt").write_bytes(b"x\n")
    p7a = run_preserve(d7, payload7)
    id7 = latest_snapshot_id(payload7, d7.name)
    snap7 = payload7 / d7.name / "snapshots" / id7
    hash7_before = snapshot_tree_hash(snap7)
    with open(snap7 / "bundle" / f"{d7.name}-full.bundle", "ab") as bf:
        bf.write(b"CORRUPTED-APPEND")
    hash7_corrupt = snapshot_tree_hash(snap7)
    p7b = run_preserve(d7, payload7)
    hash7_after = snapshot_tree_hash(snap7)
    ok7 = (p7a == 0 and p7b == EXIT_PAYLOAD
           and hash7_after == hash7_corrupt and hash7_corrupt != hash7_before
           and snap7.exists())
    results.append({"control": "D7-corrupt-bundle-refused-no-rewrite", "ok": ok7})

    # D8（F784-R2-01）同 HEAD 下 refs 新增/移动/删除：快照身份必须分裂，
    # bundle 实际可恢复（bare fetch 断言），既有快照字节不变，同态重跑 reuse。
    d8, tip8 = make_tmp_repo(scratch, "d8-refs-identity")
    payload8 = scratch / "d8-payload"
    rmtree_force(payload8)  # 控制间隔离：payload 不跨 drill 运行累积
    branch8 = git(d8, "branch", "--show-current").stdout.strip()
    p8a = run_preserve(d8, payload8)
    id8a = latest_snapshot_id(payload8, d8.name)
    snap8a = payload8 / d8.name / "snapshots" / id8a
    hash8a = snapshot_tree_hash(snap8a)

    # 新增：tag 指向旧 bundle 不可达的独立提交（临时分支提交后删分支，HEAD/工作区不变）。
    git(d8, "checkout", "-q", "-b", "tmp-side")
    (d8 / "side.txt").write_bytes(b"side\n")
    git(d8, "add", "side.txt")
    git(d8, "commit", "-q", "-m", "side")
    side_tip = git(d8, "rev-parse", "HEAD").stdout.strip()
    git(d8, "checkout", "-q", branch8)
    git(d8, "branch", "-qD", "tmp-side")
    git(d8, "tag", "archived-extra", side_tip)
    assert git(d8, "rev-parse", "HEAD").stdout.strip() == tip8
    p8b = run_preserve(d8, payload8)
    id8b = latest_snapshot_id(payload8, d8.name)
    snap8b = payload8 / d8.name / "snapshots" / id8b
    hash8b = snapshot_tree_hash(snap8b)
    rec8b = fetch_refs_from_bundle(snap8b / "bundle" / f"{d8.name}-full.bundle", scratch, "d8-rec-b")
    rec8a = fetch_refs_from_bundle(snap8a / "bundle" / f"{d8.name}-full.bundle", scratch, "d8-rec-a")
    sub_add = {
        "split": p8a == 0 and p8b == 0 and id8b != id8a,
        "old-snapshot-bytes-unchanged": snapshot_tree_hash(snap8a) == hash8a,
        "new-bundle-carries-tag": rec8b.get("refs/tags/archived-extra") == side_tip,
        "old-bundle-lacks-tag": "refs/tags/archived-extra" not in rec8a,
        "head-unchanged": json.loads((snap8b / "manifest.json").read_text(encoding="utf-8"))["state"]["head"] == tip8,
    }

    # 移动：tag 改指 HEAD tip。
    git(d8, "tag", "-f", "archived-extra", tip8)
    p8c = run_preserve(d8, payload8)
    id8c = latest_snapshot_id(payload8, d8.name)
    snap8c = payload8 / d8.name / "snapshots" / id8c
    hash8c = snapshot_tree_hash(snap8c)
    rec8c = fetch_refs_from_bundle(snap8c / "bundle" / f"{d8.name}-full.bundle", scratch, "d8-rec-c")
    sub_move = {
        "split": p8c == 0 and id8c not in (id8a, id8b),
        "prior-snapshots-unchanged": (snapshot_tree_hash(snap8a) == hash8a
                                      and snapshot_tree_hash(snap8b) == hash8b),
        "tag-moved-in-bundle": rec8c.get("refs/tags/archived-extra") == tip8,
    }

    # 删除：删 tag 后状态回到初始 refs——应复用初始快照（状态寻址语义），
    # bundle 无该 tag，不产生新目录；同态重跑仍 reuse。
    git(d8, "tag", "-d", "archived-extra")
    p8d = run_preserve(d8, payload8)
    id8d = latest_snapshot_id(payload8, d8.name)
    snap8d = payload8 / d8.name / "snapshots" / id8d
    rec8d = fetch_refs_from_bundle(snap8d / "bundle" / f"{d8.name}-full.bundle", scratch, "d8-rec-d")
    p8e = run_preserve(d8, payload8)
    snap_count = len([x for x in (payload8 / d8.name / "snapshots").iterdir() if not x.name.startswith(".")])
    sub_del = {
        "reuses-original-state-snapshot": p8d == 0 and id8d == id8a,
        "tag-absent-in-bundle": "refs/tags/archived-extra" not in rec8d,
        "idempotent-rerun": p8e == 0 and latest_snapshot_id(payload8, d8.name) == id8d,
        "snapshot-count-unchanged": snap_count == 3,
        "all-prior-unchanged": (snapshot_tree_hash(snap8a) == hash8a
                                and snapshot_tree_hash(snap8b) == hash8b
                                and snapshot_tree_hash(snap8c) == hash8c),
    }
    ok8 = all(sub_add.values()) and all(sub_move.values()) and all(sub_del.values())
    results.append({"control": "D8-refs-identity-add-move-delete", "ok": ok8,
                    "add": sub_add, "move": sub_move, "delete": sub_del,
                    "snapshots": [id8a, id8b, id8c, id8d]})

    if not args.keep:
        for sub in ("d1-idempotent", "d3-head-drift", "d4-preserve-idempotent",
                    "d5-snapshot-retention", "d6-ignored-drift", "d7-corrupt-bundle",
                    "d8-refs-identity", "d8-rec-a", "d8-rec-b", "d8-rec-c", "d8-rec-d"):
            rmtree_force(scratch / sub)
        for p in ("d4-payload", "d5-payload", "d6-payload", "d7-payload", "d8-payload"):
            rmtree_force(scratch / p)
    ok = all(r["ok"] for r in results)
    print(json.dumps({"ok": ok, "controls": results,
                      "scratch": scratch.as_posix() if args.keep else "(cleaned)"},
                     ensure_ascii=False, sort_keys=True))
    return EXIT_OK if ok else EXIT_PAYLOAD


# ---------------------------------------------------------------- shared

def emit(obj: dict, out: str | None, label: str) -> None:
    text = json.dumps(obj, ensure_ascii=False, indent=2, sort_keys=True)
    if out:
        Path(out).parent.mkdir(parents=True, exist_ok=True)
        Path(out).write_text(text + "\n", encoding="utf-8")
        print(f"{label} -> {out}", file=sys.stderr)
    else:
        print(text)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("inventory", help="只读盘点仓状态")
    p.add_argument("--repo", required=True)
    p.add_argument("--out")
    p.set_defaults(fn=cmd_inventory)

    p = sub.add_parser("preserve", help="状态寻址快照保全（同态复用/变态新增/旧快照不可覆盖）")
    p.add_argument("--repo", required=True)
    p.add_argument("--payload-dir", required=True)
    p.add_argument("--ignored-keep", nargs="*", default=[], help="需保留的 ignored 文件相对路径")
    p.set_defaults(fn=cmd_preserve)

    p = sub.add_parser("anchor", help="幂等锚定 legacy 分支/标签（异 tip 拒绝）")
    p.add_argument("--repo", required=True)
    p.add_argument("--tip", required=True)
    p.add_argument("--branch", required=True)
    p.add_argument("--tag", required=True)
    p.set_defaults(fn=cmd_anchor)

    p = sub.add_parser("verify", help="复核：原仓不变 + 锚点 + 载荷 hash + 原文件漂移（ignored 含）")
    p.add_argument("--repo", required=True)
    p.add_argument("--payload-dir", required=True)
    p.add_argument("--tip", required=True)
    p.add_argument("--branch", required=True)
    p.add_argument("--tag", required=True)
    p.add_argument("--snapshot", help="快照 id（缺省 LATEST）")
    p.set_defaults(fn=cmd_verify)

    p = sub.add_parser("recover-drill", help="V01：disposable clone 从快照 bundle 恢复 refs/tree/tag")
    p.add_argument("--repo", required=True)
    p.add_argument("--payload-dir", required=True)
    p.add_argument("--scratch")
    p.add_argument("--keep", action="store_true")
    p.add_argument("--snapshot", help="快照 id（缺省 LATEST）")
    p.set_defaults(fn=cmd_recover_drill)

    p = sub.add_parser("snapshots", help="列出某仓全部快照与 LATEST 指针")
    p.add_argument("--repo", required=True)
    p.add_argument("--payload-dir", required=True)
    p.set_defaults(fn=cmd_snapshots)

    p = sub.add_parser("drill", help="V03+R1：幂等/冲突/漂移/快照保留/bundle 损坏永久反控")
    p.add_argument("--scratch")
    p.add_argument("--keep", action="store_true")
    p.set_defaults(fn=cmd_drill)

    args = ap.parse_args(argv)
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
