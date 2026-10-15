#!/usr/bin/env python3
"""legacy-preservation.py — PLAN-784 旧仓保全工具（Category A）。

对既有 auto-ui / auto-man 仓做只读盘点、git bundle + 原始字节载荷保全、
legacy refs（分支+标签）锚定与恢复演练。设计约束（Plan 784 / Design 37 §8.0）：

- 不改变原仓工作区：payload 永远写到仓外的独立恢复位置；
- 同名 ref 同 tip 幂等复用，异 tip 明确拒绝且原 ref 不动（exit 3）；
- 未提交/未跟踪内容不能只靠 bundle：脏跟踪文件按 work/head/diff 三份字节保全；
- 收据只写元信息（SHA256/路径/大小），payload 内容不进仓；
- 输出 JSON 不含时间戳/绝对检出路径抖动源：同输入重跑字节稳定。

用法：
  python scripts/legacy-preservation.py inventory  --repo PATH [--out FILE]
  python scripts/legacy-preservation.py preserve   --repo PATH --payload-dir DIR
                                                   [--ignored-keep RELPATH ...]
  python scripts/legacy-preservation.py anchor     --repo PATH --tip SHA
                                                   --branch NAME --tag NAME
  python scripts/legacy-preservation.py verify     --repo PATH --payload-dir DIR
                                                   --tip SHA --branch NAME --tag NAME
  python scripts/legacy-preservation.py recover-drill --repo PATH --payload-dir DIR
                                                   --scratch DIR [--keep]
  python scripts/legacy-preservation.py drill      [--scratch DIR] [--keep]

退出码：0=通过；2=用法/环境；3=ref 冲突/锚点不匹配；4=载荷/恢复校验失败。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

TOOL_VERSION = "legacy-preservation.py v1"
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


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


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


# ---------------------------------------------------------------- preserve

def preserve_payload_paths(payload_dir: Path, repo: Path):
    """目录布局：<payload>/<repo>/bundle、wip/、untracked/、ignored/、manifest.json。"""
    repo_dir = payload_dir / repo.name
    return {
        "root": repo_dir,
        "bundle": repo_dir / "bundle" / f"{repo.name}-full.bundle",
        "wip": repo_dir / "wip",
        "untracked": repo_dir / "untracked",
        "ignored": repo_dir / "ignored",
        "manifest": repo_dir / "manifest.json",
    }


def create_bundle(repo: Path, bundle_file: Path, refs_seen: dict) -> dict:
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
    paths = preserve_payload_paths(payload_dir, Path(facts["toplevel"]))
    manifest_path = paths["manifest"]
    existing = None
    if manifest_path.exists():
        existing = json.loads(manifest_path.read_text(encoding="utf-8"))

    # 1) bundle：已存在则 verify + hash 对照复用，不重写。
    if existing is not None and paths["bundle"].exists():
        verify = git(repo, "bundle", "verify", paths["bundle"].as_posix(), check=False)
        actual = sha256_file(paths["bundle"])
        if verify.returncode != 0:
            print(f"error: 既有 bundle 校验失败: {paths['bundle']}", file=sys.stderr)
            return EXIT_PAYLOAD
        if existing.get("bundle", {}).get("sha256") != actual:
            print(f"error: 既有 bundle 与 manifest hash 不一致，拒绝静默重写: {paths['bundle']}", file=sys.stderr)
            return EXIT_PAYLOAD
        bundle_info = dict(existing["bundle"])
        bundle_info["action"] = "reuse"
        print(f"bundle: reuse {paths['bundle'].name} sha256={actual[:12]}…", file=sys.stderr)
    else:
        refs_before = set(facts["refs"].keys())
        bundle_info = create_bundle(repo, paths["bundle"], facts["refs"])
        bundle_info["action"] = "create"
        refs_after = {k: v for k, v in repo_facts(repo)["refs"].items()}
        if refs_after != facts["refs"]:
            print("error: bundle 创建改变了仓 refs，中止", file=sys.stderr)
            return EXIT_PAYLOAD
        print(f"bundle: create {paths['bundle'].name} sha256={bundle_info['sha256'][:12]}…", file=sys.stderr)

    # 2) 脏跟踪文件：work/head/diff 三份原始字节。
    wip_entries = []
    for line in facts["status_porcelain"]:
        status, path = parse_status_entry(line)
        raw = line
        if status in ("??",):
            continue  # untracked 单独处理
        safe = path.replace("/", "__")
        work_file = head_file = diff_file = None
        if status != "D":
            src = Path(facts["toplevel"]) / path
            work_file = paths["wip"] / f"{safe}.work"
            work_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, work_file)
        head_file = paths["wip"] / f"{safe}.head"
        head_file.parent.mkdir(parents=True, exist_ok=True)
        head_file.write_bytes(git_bytes(repo, "show", f"HEAD:{path}"))
        diff_bytes = git_bytes(repo, "diff", "--binary", "HEAD", "--", path)
        diff_file = None
        if diff_bytes:
            diff_file = paths["wip"] / f"{safe}.diff"
            diff_file.parent.mkdir(parents=True, exist_ok=True)
            diff_file.write_bytes(diff_bytes)
        wip_entries.append({
            "status_line": raw,
            "status": status,
            "path": path,
            "work_sha256": sha256_file(work_file) if work_file else None,
            "head_sha256": sha256_file(head_file),
            "diff_sha256": sha256_file(diff_file) if diff_file else None,
            "payload": {
                "work": work_file.relative_to(paths["root"]).as_posix() if work_file else None,
                "head": head_file.relative_to(paths["root"]).as_posix(),
                "diff": diff_file.relative_to(paths["root"]).as_posix() if diff_file else None,
            },
        })

    # 3) 未跟踪（非 ignored）文件：原始字节。
    untracked_entries = []
    for line in facts["status_porcelain"]:
        status, path = parse_status_entry(line)
        if status != "??":
            continue
        src = Path(facts["toplevel"]) / path
        dst = paths["untracked"] / path
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, dst)
        untracked_entries.append({
            "path": path,
            "sha256": sha256_file(dst),
            "payload": dst.relative_to(paths["root"]).as_posix(),
        })

    # 4) 显式点名保留的 ignored 文件。
    ignored_keep = []
    for rel in args.ignored_keep or []:
        src = Path(facts["toplevel"]) / rel
        if not src.is_file():
            print(f"error: --ignored-keep 指定的文件不存在: {rel}", file=sys.stderr)
            return EXIT_ENV
        dst = paths["ignored"] / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, dst)
        ignored_keep.append({
            "path": rel,
            "sha256": sha256_file(dst),
            "payload": dst.relative_to(paths["root"]).as_posix(),
        })

    manifest = {
        "tool": TOOL_VERSION,
        "repo": {k: facts[k] for k in ("name", "toplevel", "head", "branch", "symref")},
        "bundle": bundle_info,
        "wip": wip_entries,
        "untracked": untracked_entries,
        "ignored_keep": ignored_keep,
        "status_porcelain": facts["status_porcelain"],
        "ignored_listing": facts["ignored"],
        "stash_count": facts["stash_count"],
        "refs_frozen": facts["refs"],
        "payload_root": payload_dir.as_posix(),
    }
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps({
        "repo": facts["name"], "head": facts["head"], "action": manifest["bundle"]["action"],
        "bundle_sha256": bundle_info["sha256"], "manifest": manifest_path.as_posix(),
        "wip": len(wip_entries), "untracked": len(untracked_entries), "ignored_keep": len(ignored_keep),
    }, ensure_ascii=False, sort_keys=True))
    return EXIT_OK


# ---------------------------------------------------------------- anchor

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


class ConflictError(RuntimeError):
    pass


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
    paths = preserve_payload_paths(payload_dir, Path(facts["toplevel"]))
    manifest = json.loads(paths["manifest"].read_text(encoding="utf-8"))
    failures = []

    # 原仓不变量：HEAD/tip、主检出分支、status 与保全时一致。
    if facts["head"] != args.tip:
        failures.append(f"HEAD 漂移: {facts['head']} != {args.tip}")
    if facts["symref"] != manifest["repo"]["symref"]:
        failures.append(f"symref 变化: {facts['symref']} != {manifest['repo']['symref']}")
    if facts["status_porcelain"] != manifest["status_porcelain"]:
        failures.append(f"工作区 status 变化: {facts['status_porcelain']} != {manifest['status_porcelain']}")

    # refs：legacy 锚点在 tip 上；保全清单里的既有 refs 未被移动。
    for full, sha in ((f"refs/heads/{args.branch}", args.tip), (f"refs/tags/{args.tag}", args.tip)):
        cur = facts["refs"].get(full)
        if cur != sha:
            failures.append(f"{full} 不在锚定 tip: {cur} != {sha}")
    for full, sha in manifest["refs_frozen"].items():
        if facts["refs"].get(full) != sha:
            failures.append(f"保全 refs 漂移: {full}")

    # bundle：可校验 + hash 一致。
    verify = git(repo, "bundle", "verify", paths["bundle"].as_posix(), check=False)
    if verify.returncode != 0:
        failures.append("bundle verify 失败")
    if sha256_file(paths["bundle"]) != manifest["bundle"]["sha256"]:
        failures.append("bundle sha256 与 manifest 不一致")

    # 载荷：文件在、hash 对；work 副本与当前原文件相等（原字节保全 oracle）。
    root = paths["root"]
    for entry in manifest["wip"]:
        for key in ("work", "head", "diff"):
            rel = entry["payload"].get(key)
            want = entry[f"{key}_sha256"] if key != "diff" else entry.get("diff_sha256")
            if rel is None or want is None:
                continue
            p = root / rel
            if not p.is_file():
                failures.append(f"载荷缺失: {rel}")
            elif sha256_file(p) != want:
                failures.append(f"载荷 hash 不符: {rel}")
        if entry["payload"].get("work"):
            cur = sha256_file(Path(facts["toplevel"]) / entry["path"])
            if cur != entry["work_sha256"]:
                failures.append(f"原工作文件已变: {entry['path']}")
    for entry in manifest["untracked"] + manifest["ignored_keep"]:
        p = root / entry["payload"]
        if not p.is_file():
            failures.append(f"载荷缺失: {entry['payload']}")
        elif sha256_file(p) != entry["sha256"]:
            failures.append(f"载荷 hash 不符: {entry['payload']}")

    ok = not failures
    print(json.dumps({"repo": facts["name"], "ok": ok, "failures": failures}, ensure_ascii=False, sort_keys=True))
    return EXIT_OK if ok else EXIT_PAYLOAD


# ---------------------------------------------------------------- recover-drill (V01)

def cmd_recover_drill(args) -> int:
    repo = Path(args.repo).resolve()
    payload_dir = Path(args.payload_dir).resolve()
    facts = repo_facts(repo)
    paths = preserve_payload_paths(payload_dir, Path(facts["toplevel"]))
    manifest = json.loads(paths["manifest"].read_text(encoding="utf-8"))
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
        # bundle 记录的 refs 应全部可恢复且 sha 相等（HEAD 单列）。
        for full, sha in manifest["refs_frozen"].items():
            got = drilled_refs.get(full)
            if got != sha:
                failures.append(f"恢复 ref 不符: {full} got={got} want={sha}")
        # tip commit/tree 相等。
        src_tree = git(repo, "rev-parse", f"{facts['head']}^{{tree}}").stdout.strip()
        got_tree = git(drill_dir, "rev-parse", f"{facts['head']}^{{tree}}", check=False)
        if got_tree.returncode != 0 or got_tree.stdout.strip() != src_tree:
            failures.append(f"恢复 tip tree 不符: {got_tree.stdout.strip()} != {src_tree}")
        # 旧 tag 可追溯（v* 标签逐个可达）。
        old_tags = [r for r in manifest["refs_frozen"] if r.startswith("refs/tags/")]
        for t in old_tags:
            if t not in drilled_refs:
                failures.append(f"旧 tag 未恢复: {t}")

    if not args.keep:
        rmtree_force(drill_dir)
    ok = not failures
    print(json.dumps({
        "repo": facts["name"], "ok": ok, "failures": failures,
        "refs_recovered": len(manifest["refs_frozen"]) - len(failures),
        "old_tags": len([r for r in manifest["refs_frozen"] if r.startswith("refs/tags/")]),
        "tree": git(repo, "rev-parse", f"{facts['head']}^{{tree}}").stdout.strip(),
    }, ensure_ascii=False, sort_keys=True))
    return EXIT_OK if ok else EXIT_PAYLOAD


# ---------------------------------------------------------------- drill (V03 永久反控)

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
        ok2 = False
        why2 = "conflict 未被拒绝"
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
    fake_tip = "0" * 40
    try:
        anchor_repo(d3, fake_tip, "legacy/pre-v06-extraction", "legacy-pre-v06-bbbb2222")
        ok3 = False
        why3 = "HEAD 漂移未被拒绝"
    except ConflictError as e:
        ok3 = "HEAD" in str(e)
        why3 = str(e)
    results.append({"control": "D3-head-drift-reject", "ok": ok3, "message": why3})

    # D4 bundle 幂等：同仓重复 preserve 复用既有 bundle（hash 不变），WIP 字节保全。
    d4, tip_d = make_tmp_repo(scratch, "d4-preserve-idempotent")
    payload = scratch / "d4-payload"
    p4a = run_preserve(d4, payload)
    m1 = json.loads((payload / d4.name / "manifest.json").read_text(encoding="utf-8"))
    (d4 / "f.txt").write_text("dirty\n", encoding="utf-8")
    p4b = run_preserve(d4, payload)
    m2 = json.loads((payload / d4.name / "manifest.json").read_text(encoding="utf-8"))
    ok4 = (p4a == 0 and p4b == 0 and m1["bundle"]["sha256"] == m2["bundle"]["sha256"]
           and m2["bundle"]["action"] == "reuse" and len(m2["wip"]) == 1)
    results.append({"control": "D4-preserve-idempotent", "ok": ok4})

    if not args.keep:
        for sub in ("d1-idempotent", "d3-head-drift", "d4-preserve-idempotent"):
            rmtree_force(scratch / sub)
        rmtree_force(scratch / "d4-payload")
    ok = all(r["ok"] for r in results)
    print(json.dumps({"ok": ok, "controls": results,
                      "scratch": scratch.as_posix() if args.keep else "(cleaned)"},
                     ensure_ascii=False, sort_keys=True))
    return EXIT_OK if ok else EXIT_PAYLOAD


def run_preserve(repo: Path, payload_dir: Path) -> int:
    ns = argparse.Namespace(repo=repo.as_posix(), payload_dir=payload_dir.as_posix(), ignored_keep=[])
    return cmd_preserve(ns)


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

    p = sub.add_parser("preserve", help="bundle + WIP 原始字节保全到独立 payload 目录")
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

    p = sub.add_parser("verify", help="复核：原仓不变 + 锚点 + 载荷 hash")
    p.add_argument("--repo", required=True)
    p.add_argument("--payload-dir", required=True)
    p.add_argument("--tip", required=True)
    p.add_argument("--branch", required=True)
    p.add_argument("--tag", required=True)
    p.set_defaults(fn=cmd_verify)

    p = sub.add_parser("recover-drill", help="V01：disposable clone 从 bundle 恢复 refs/tree/tag")
    p.add_argument("--repo", required=True)
    p.add_argument("--payload-dir", required=True)
    p.add_argument("--scratch")
    p.add_argument("--keep", action="store_true")
    p.set_defaults(fn=cmd_recover_drill)

    p = sub.add_parser("drill", help="V03：临时仓幂等/冲突/漂移/bundle 幂等永久反控")
    p.add_argument("--scratch")
    p.add_argument("--keep", action="store_true")
    p.set_defaults(fn=cmd_drill)

    args = ap.parse_args(argv)
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
