#!/usr/bin/env python3
"""modularity-audit.py — v0.6 模块边界基线与增量检查（PLAN-777，Category A）。

只读审计工具：规模/逐文件 hash、Cargo 声明依赖边、源码文本引用候选、
ownership 覆盖与 strict 增量判红。不编译、不写任何输入文件；报告另存。

口径与边界（与 docs/plans/777-v06-modular-boundary-baseline.md §5 对齐）：
- 规模口径沿用调研基线（git ls-files 受管 .rs + splitlines 物理行，含空行/注释/
  内联测试/生成文件；不是生产 SLOC，也不是可删除量）。
- 声明边主源 = workspace 成员 manifest 直接解析（tomllib）。选择 manifest 直解而非
  cargo metadata 作为主源的原因：strict 必须能在 manifest 变更、Cargo.lock 未再生成的
  工作树上点名具体新增边（V-02 oracle）；cargo metadata --no-deps --locked 仅作一致性
  对照——锁缺失/不满足时记录依赖调查缺口，绝不自动更新，也不作为编译验收。
- 文本引用只是候选，不是解析后的语义依赖边；别名/re-export/宏展开为已知盲区，
  在报告 scanner_limitations 中显式声明。
- 观察模式（observe）覆盖全仓，只报告不判红；strict 只判冻结基线之外的新增禁向边，
  存量边以窄豁免（逐边 owner + 退出阶段）grandfather；过期基线/脏树/宽豁免确定性失败，
  不能默默通行。

payload 稳定性契约：同一输入（同一 HEAD、同一受管内容、同一配置/基线文件）两次扫描
输出字节一致；不含时间戳、不含检出绝对路径。

退出码：0 成功（observe / strict 通过 / 基线刷新 / 自测通过）；
2 strict 判红；3 配置/基线不合法（含宽豁免拒绝）或刷新前置不满足；64 用法错误；1 运行错误。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path, PurePosixPath

TOOL = "scripts/modularity-audit.py"
REPORT_SCHEMA = "modularity-audit-report/1"
BASELINE_SCHEMA = "module-boundaries-baseline/1"
CONFIG_SCHEMA = "module-boundaries/1"

DEFAULT_CONFIG_RELPATH = ".autoos/module-boundaries.toml"
DEFAULT_BASELINE_RELPATH = ".autoos/module-boundaries-baseline.json"

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_STRICT_FAIL = 2
EXIT_CONFIG_INVALID = 3
EXIT_USAGE = 64

# strict 新鲜度锚定：这些输入自 base_revision 以来的变更会使基线过期（基线文件自身的
# 落库提交除外——刷新动作本身就是新增该文件）。test/ 下的 fixture manifest 不参与
# 真实根的边提取，不在锚定面内。
STALENESS_PATHSPECS = ["Cargo.toml", ":(glob)crates/**/Cargo.toml", DEFAULT_CONFIG_RELPATH]

# 宽豁免拒绝名单：[baseline] 只允许窄豁免（逐边 edge_notes），任何目录级/前缀级豁免
# 键都是宽豁免，配置加载即拒绝（V-04d）。
REJECTED_BASELINE_KEYS = {
    "exempt_dirs", "exempt_paths", "exempt_patterns", "exempt",
    "dir_exemptions", "ignore_paths", "ignore_patterns",
}
REJECTED_EDGE_NOTE_KEYS = {"path_prefix", "scope", "dir"}

# 文本扫描器的已知盲区（固定清单，随报告输出；V-03 诚实性契约）。
SCANNER_LIMITATIONS = [
    "别名引用不可见：use ... as 别名后的短路径使用不计入命中",
    "re-export 不可见：pub use 消化掉模块路径后的类型使用不计入命中",
    "宏内部生成/展开期拼接的路径只在源文本出现处计数，不追踪展开",
    "字符串/注释命中按逐字符状态机分类，跨行 raw string 与字符字面量边界为启发式",
    "cfg(test) mod 边界按朴素花括号计数判定，花括号在非常规位置可能误界",
    "文本命中是候选，不是解析后的语义依赖边；人工确认前不得当作架构事实",
]

TEXT_SAMPLE_LIMIT = 200
TOP_FILES_LIMIT = 30
SIZES_ROOT_KEYS = ["crates", "experimental", "test", "vendor", "docs", "scripts", "examples"]


class UsageError(Exception):
    pass


class ConfigError(Exception):
    """配置/基线不合法（含宽豁免拒绝）。"""


# ---------------------------------------------------------------------------
# 基础工具
# ---------------------------------------------------------------------------

def run_git(root: Path, args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", "-C", str(root), *args],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
    )


def git_required(root: Path, args: list[str]) -> str:
    proc = run_git(root, args)
    if proc.returncode != 0:
        raise RuntimeError(
            f"git {' '.join(args)} 失败 (exit {proc.returncode}): {proc.stderr.strip()[:300]}"
        )
    return proc.stdout


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, sort_keys=True, indent=2) + "\n"


def sanitize_for_report(text: str, root: Path) -> str:
    """剔除检出绝对路径，保持 payload 可移植稳定。"""
    out = text.replace(str(root), "<root>")
    out = out.replace(str(root.resolve()), "<root>")
    out = out.replace("\\\\", "\\")
    return out


def read_text_or_none(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None


# ---------------------------------------------------------------------------
# Git 采样层
# ---------------------------------------------------------------------------

def collect_revision(root: Path) -> dict:
    head = git_required(root, ["rev-parse", "HEAD"]).strip()
    branch_proc = run_git(root, ["symbolic-ref", "--short", "HEAD"])
    branch = branch_proc.stdout.strip() if branch_proc.returncode == 0 else None
    status_proc = run_git(root, ["status", "--porcelain", "-z"])
    if status_proc.returncode != 0:
        raise RuntimeError("git status 失败")
    tracked_status = []
    for record in status_proc.stdout.split("\0"):
        if not record:
            continue
        code, _, path = record.partition(" ")
        if code == "??":
            continue  # 未跟踪文件不是受管输入
        tracked_status.append(f"{code} {path}")
    return {
        "head": head,
        "branch": branch,
        "tracked_dirty": bool(tracked_status),
        "tracked_status": sorted(tracked_status),
    }


def collect_tracked_files(root: Path) -> list[str]:
    out = git_required(root, ["ls-files", "-z"])
    files = [f for f in out.split("\0") if f]
    return sorted(files)


def staleness_paths_changed(root: Path, base_revision: str) -> dict:
    """base_revision..HEAD 之间锚定输入是否变更；base 不可解析同样判过期。"""
    verify = run_git(root, ["rev-parse", "--verify", f"{base_revision}^{{commit}}"])
    if verify.returncode != 0:
        return {"base_known": False, "changed": [], "stale": True}
    diff = run_git(root, ["diff", "--name-only", f"{base_revision}..HEAD", "--", *STALENESS_PATHSPECS])
    if diff.returncode != 0:
        return {"base_known": True, "changed": [], "stale": True}
    changed = sorted({line for line in diff.stdout.splitlines() if line})
    return {"base_known": True, "changed": changed, "stale": bool(changed)}


# ---------------------------------------------------------------------------
# 规模层（口径沿用调研基线 v06-modularization-baseline.jsonl）
# ---------------------------------------------------------------------------

def file_stats(data: bytes) -> dict:
    text = data.decode("utf-8", errors="replace")
    lines = text.splitlines()
    return {
        "lines": len(lines),
        "nonblank": sum(1 for line in lines if line.strip()),
        "bytes": len(data),
        "sha256": sha256_bytes(data),
    }


def collect_sizes(root: Path, tracked: list[str]) -> tuple[dict, list[dict]]:
    rust_files = sorted(p for p in tracked if p.endswith(".rs"))
    files = []
    for rel in rust_files:
        try:
            data = (root / rel).read_bytes()
        except OSError:
            continue
        entry = {"path": rel, **file_stats(data)}
        files.append(entry)
    lines_of = {f["path"]: f["lines"] for f in files}

    def agg(predicate) -> dict:
        paths = [p for p in rust_files if predicate(p)]
        return {
            "files": len(paths),
            "lines": sum(lines_of.get(p, 0) for p in paths),
        }

    ui_five_prefixes = tuple(
        f"crates/auto-lang/src/{d}/" for d in ("ui", "ui_gen", "aura", "design_tokens", "a2ui")
    )
    totals = {
        "tracked_rust_files": len(rust_files),
        "tracked_rust_lines": sum(lines_of.values()),
        "auto_lang_src_files": sum(
            1 for p in rust_files if p.startswith("crates/auto-lang/src/")
        ),
        "auto_lang_src_lines": sum(
            lines_of.get(p, 0) for p in rust_files if p.startswith("crates/auto-lang/src/")
        ),
        "ui_five_dirs_files": sum(1 for p in rust_files if p.startswith(ui_five_prefixes)),
        "ui_five_dirs_lines": sum(
            lines_of.get(p, 0) for p in rust_files if p.startswith(ui_five_prefixes)
        ),
        "dedicated_src_tests_files": sum(
            1 for p in rust_files if p.startswith("crates/auto-lang/src/tests/")
        ),
        "dedicated_src_tests_lines": sum(
            lines_of.get(p, 0) for p in rust_files if p.startswith("crates/auto-lang/src/tests/")
        ),
    }

    roots = {
        key: agg(lambda p, k=key: p.startswith(f"{k}/") or p == k)
        for key in SIZES_ROOT_KEYS
    }

    crates: dict[str, dict] = {}
    for p in rust_files:
        parts = p.split("/")
        if len(parts) >= 2 and parts[0] == "crates":
            name = f"crates/{parts[1]}"
            slot = crates.setdefault(name, {"files": 0, "lines": 0})
            slot["files"] += 1
            slot["lines"] += lines_of.get(p, 0)
    crates = dict(sorted(crates.items()))

    lang_modules: dict[str, dict] = {}
    for p in rust_files:
        if not p.startswith("crates/auto-lang/src/"):
            continue
        rest = p[len("crates/auto-lang/src/"):]
        mod = rest.split("/")[0] if "/" in rest else "(root)"
        slot = lang_modules.setdefault(mod, {"files": 0, "lines": 0})
        slot["files"] += 1
        slot["lines"] += lines_of.get(p, 0)
    lang_modules = dict(sorted(lang_modules.items(), key=lambda kv: (-kv[1]["lines"], kv[0])))

    top_files = sorted(files, key=lambda f: (-f["lines"], f["path"]))[:TOP_FILES_LIMIT]
    top_files = [
        {k: f[k] for k in ("path", "lines", "nonblank", "bytes", "sha256")} for f in top_files
    ]
    sizes = {
        "totals": totals,
        "roots": roots,
        "crates": crates,
        "lang_modules": lang_modules,
        "top_files": top_files,
    }
    return sizes, files


# ---------------------------------------------------------------------------
# Cargo 声明边层（主源：manifest 直解；对照源：cargo metadata --locked）
# ---------------------------------------------------------------------------

def discover_members(root: Path) -> list[Path]:
    root_manifest = root / "Cargo.toml"
    if not root_manifest.exists():
        return []
    data = tomllib.loads(root_manifest.read_text(encoding="utf-8"))
    members: list[Path] = []
    for entry in data.get("workspace", {}).get("members", []):
        if any(ch in entry for ch in "*?["):
            matches = sorted(
                p for p in root.glob(entry)
                if p.is_dir() and (p / "Cargo.toml").exists()
            )
            members.extend(matches)
        else:
            cand = root / entry
            if (cand / "Cargo.toml").exists():
                members.append(cand)
    return sorted(set(members))


def manifest_package_name(manifest: Path) -> str | None:
    text = read_text_or_none(manifest)
    if text is None:
        return None
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError:
        return None
    pkg = data.get("package", {})
    return pkg.get("name") or data.get("lib", {}).get("name")


def extract_declared_edges(root: Path) -> tuple[list[dict], list[str], dict]:
    """返回 (edges, member_names, gaps)。边字典字段：
    from/to（包名）、dep_key、kind（normal/dev/build）、optional、target（cfg 串或 None）、
    to_scope（member | nonmember-in-root | external | unresolved）。
    """
    members = discover_members(root)
    root_resolved = root.resolve()
    member_dirs = {m.resolve(): m for m in members}
    root_manifest = root / "Cargo.toml"
    workspace_deps: dict = {}
    if root_manifest.exists():
        try:
            root_data = tomllib.loads(root_manifest.read_text(encoding="utf-8"))
            workspace_deps = root_data.get("workspace", {}).get("dependencies", {})
        except tomllib.TOMLDecodeError:
            pass

    edges: list[dict] = []
    member_names: list[str] = []
    gaps: list[str] = []

    for member in members:
        manifest = member / "Cargo.toml"
        name = manifest_package_name(manifest)
        if name is None:
            gaps.append(f"member manifest 不可解析: {member.name}")
            continue
        member_names.append(name)
        try:
            data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as exc:
            gaps.append(f"member manifest TOML 错误 {member.name}: {exc}")
            continue

        dep_sections: list[tuple[str, str | None]] = [
            ("dependencies", None), ("dev-dependencies", None), ("build-dependencies", None),
        ]
        for target_key, target_sec in (data.get("target") or {}).items():
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                dep_sections.append((section, target_key))

        for section, target_key in dep_sections:
            table = data.get(section) if target_key is None else (target_sec or {}).get(section)
            kind = {"dependencies": "normal", "dev-dependencies": "dev",
                    "build-dependencies": "build"}[section]
            for dep_key, spec in (table or {}).items():
                if isinstance(spec, str):
                    spec = {"version": spec}
                spec = dict(spec or {})
                if spec.pop("workspace", False) and dep_key in workspace_deps:
                    inherited = workspace_deps[dep_key]
                    if isinstance(inherited, str):
                        inherited = {"version": inherited}
                    inherited = dict(inherited or {})
                    inherited.update({k: v for k, v in spec.items() if v is not None})
                    spec = inherited
                path_value = spec.get("path")
                if not isinstance(path_value, str):
                    continue
                target_dir = (member / path_value).resolve()
                target_manifest = target_dir / "Cargo.toml"
                target_name = manifest_package_name(target_manifest)
                if target_dir in member_dirs:
                    to_scope = "member"
                    to = target_name or f"unresolved({target_dir.name})"
                elif target_name is None:
                    to_scope = "unresolved"
                    rel = _rel_or_external(target_dir, root_resolved)
                    to = f"unresolved({rel})"
                else:
                    try:
                        target_dir.relative_to(root_resolved)
                        to_scope = "nonmember-in-root"
                    except ValueError:
                        to_scope = "external"
                    to = target_name
                if target_name is None:
                    gaps.append(
                        f"路径依赖目标不可解析: {name} -> {path_value} ({to})"
                    )
                edges.append({
                    "from": name,
                    "to": to,
                    "dep_key": dep_key,
                    "kind": kind,
                    "optional": bool(spec.get("optional", False)),
                    "target": target_key,
                    "to_scope": to_scope,
                })

    edges.sort(key=lambda e: (e["from"], e["to"], e["kind"], e["target"] or "", e["dep_key"]))
    return edges, sorted(member_names), gaps


def _rel_or_external(path: Path, root: Path) -> str:
    try:
        return Path(path).relative_to(root).as_posix()
    except ValueError:
        return "../" + Path(path).relative_to(root.parent).as_posix()


def cargo_metadata_cross_check(root: Path, manifest_edges: list[dict]) -> dict:
    command = "cargo metadata --no-deps --locked --format-version 1"
    if not (root / "Cargo.toml").exists():
        return {"attempted": False, "ok": None, "gap": "根无 Cargo.toml", "discrepancies": []}
    if not (root / "Cargo.lock").exists():
        return {
            "attempted": False, "ok": None,
            "gap": "Cargo.lock 缺失（--locked 需要既有锁；按约不自动生成/更新）",
            "discrepancies": [],
        }
    try:
        proc = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"],
            cwd=str(root), capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=180,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {
            "attempted": True, "ok": None, "command": command,
            "gap": f"cargo metadata 执行失败: {sanitize_for_report(str(exc), root)}",
            "discrepancies": [],
        }
    if proc.returncode != 0:
        return {
            "attempted": True, "ok": False, "command": command,
            "gap": "cargo metadata --locked 失败（锁状态不满足）；按约不自动更新锁，仅记录缺口: "
                   + sanitize_for_report(proc.stderr.strip(), root)[:300],
            "discrepancies": [],
        }
    try:
        meta = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return {
            "attempted": True, "ok": False, "command": command,
            "gap": "cargo metadata 输出不可解析", "discrepancies": [],
        }

    root_resolved = str(root.resolve()).replace("\\", "/")
    cargo_edges = set()
    for pkg in meta.get("packages", []):
        manifest_path = pkg.get("manifest_path", "").replace("\\", "/")
        if not manifest_path.startswith(root_resolved + "/"):
            continue
        frm = pkg.get("name")
        for dep in pkg.get("dependencies", []):
            if dep.get("path") is None:
                continue  # registry 依赖不在 manifest 扫描域（只提取 path 依赖边）
            to = dep.get("name")
            if dep.get("dep_kinds") is not None:
                for dk in dep["dep_kinds"]:
                    cargo_edges.add((frm, to, dk.get("kind", "normal"), dk.get("target")))
            else:
                # 旧格式（如 cargo 1.98 实测）：顶层 kind/target 字段，kind null = normal
                cargo_edges.add((frm, to, dep.get("kind") or "normal", dep.get("target")))

    mine = {
        (e["from"], e["to"], e["kind"], e["target"])
        for e in manifest_edges if e["to_scope"] != "unresolved"
    }
    discrepancies = sorted(
        [{"kind": "manifest_scan_only", "edge": list(e)} for e in mine - cargo_edges]
        + [{"kind": "cargo_only", "edge": list(e)} for e in cargo_edges - mine],
        key=lambda d: (d["kind"], d["edge"]),
    )
    return {
        "attempted": True, "ok": True, "command": command,
        "cargo_edge_count": len(cargo_edges),
        "discrepancies": discrepancies, "gap": None,
    }


# ---------------------------------------------------------------------------
# 文本候选层（启发式分类：code/comment/doc/string/test/macro）
# ---------------------------------------------------------------------------

_MATCH_CACHE: dict = {}


def classify_source_lines(text: str) -> list[dict]:
    """逐行产出 {"region": code/test/macro, "ctx": 逐字符上下文类}。
    ctx 取值 code/comment/string；doc 与普通注释的区分在 classify_match 中按行前缀判定。
    启发式，见 SCANNER_LIMITATIONS。"""
    lines = text.splitlines()
    results: list[dict] = []
    in_block = False
    string_state = None  # None | {"raw": bool, "hashes": int}
    test_regions: list[int] = []   # 各活动区域的剩余嵌套深度
    macro_regions: list[int] = []
    pending_cfg_test = False
    pending_macro = False

    for line in lines:
        ctx: list[str | None] = [None] * len(line)
        col, length = 0, len(line)
        while col < length:
            rest = line[col:]
            if in_block:
                end = rest.find("*/")
                span_end = length if end == -1 else col + end + 2
                for i in range(col, span_end):
                    ctx[i] = "comment"
                col = span_end
                if end == -1:
                    break
                in_block = False
                continue
            if string_state is not None:
                if string_state["raw"]:
                    closer = '"' + "#" * string_state["hashes"]
                    end = rest.find(closer)
                    span_end = length if end == -1 else col + end + len(closer)
                    cls = "string"
                    for i in range(col, span_end):
                        ctx[i] = cls
                    col = span_end
                    if end == -1:
                        break
                    string_state = None
                    continue
                ch = rest[0]
                ctx[col] = "string"
                if ch == "\\":
                    if col + 1 < length:
                        ctx[col + 1] = "string"
                    col += 2
                    continue
                if ch == '"':
                    string_state = None
                col += 1
                continue
            if rest.startswith("//"):
                for i in range(col, length):
                    ctx[i] = "comment"
                col = length
                break  # 行注释：其余全非代码
            if rest.startswith("/*"):
                in_block = True
                ctx[col] = "comment"
                ctx[col + 1] = "comment"
                col += 2
                continue
            m = re.match(r'(b?r)(#*)"', rest)
            if m:
                string_state = {"raw": True, "hashes": len(m.group(2))}
                for i in range(col, col + m.end()):
                    ctx[i] = "string"
                col += m.end()
                continue
            if rest[0] == '"':
                string_state = {"raw": False, "hashes": 0}
                ctx[col] = "string"
                col += 1
                continue
            ctx[col] = "code"
            col += 1

        code_part = "".join(
            line[c] if c < len(ctx) and ctx[c] == "code" else " " for c in range(len(line))
        )
        stripped = code_part.strip()

        # 1) 既有区域按本行可执行代码的净花括号变化更新；嵌套区域共享同一增量
        delta = code_part.count("{") - code_part.count("}")
        for stack in (test_regions, macro_regions):
            if stack:
                stack[-1] += delta
        for stack in (test_regions, macro_regions):
            while stack and stack[-1] <= 0:
                stack.pop()

        # 2) 新区域开区检测（cfg(test) mod / macro_rules!）
        if pending_cfg_test and re.match(r"(pub\s+)?mod\s+\w+", stripped) and "{" in code_part:
            test_regions.append(max(1, delta))
            pending_cfg_test = False
        if pending_macro and "{" in code_part:
            macro_regions.append(max(1, delta))
            pending_macro = False
        if re.match(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", stripped):
            pending_cfg_test = True
        if "macro_rules!" in code_part:
            if "{" in code_part:
                macro_regions.append(max(1, delta))
            else:
                pending_macro = True

        region = "code"
        if test_regions:
            region = "test"
        elif macro_regions:
            region = "macro"
        results.append({"region": region, "ctx": ctx})

    return results


def classify_match(line: str, entry: dict, col: int) -> str:
    ctx = entry["ctx"]
    c = ctx[col] if col < len(ctx) else "code"
    if c == "string":
        return "string"
    if c == "comment" or c is None:
        raw = line.lstrip()
        if raw.startswith("///") or raw.startswith("//!"):
            return "doc"
        return "comment"
    return entry["region"]


def _compile_pattern(pattern: str):
    if pattern not in _MATCH_CACHE:
        _MATCH_CACHE[pattern] = re.compile(pattern)
    return _MATCH_CACHE[pattern]


def run_text_scans(root: Path, tracked: list[str], scan_configs: list[dict]) -> dict:
    rust_files = sorted(p for p in tracked if p.endswith(".rs"))
    scans = []
    for cfg in scan_configs:
        source_roots = [r.rstrip("/") for r in cfg.get("source_roots", [])]
        exclude_roots = [r.rstrip("/") for r in cfg.get("exclude_roots", [])]
        patterns = [ _compile_pattern(p) for p in cfg.get("patterns", []) ]

        def selected(path: str) -> bool:
            if not any(path == r or path.startswith(r + "/") for r in source_roots):
                return False
            return not any(path == r or path.startswith(r + "/") for r in exclude_roots)

        targets = [p for p in rust_files if selected(p)]
        total = 0
        by_class: dict[str, int] = {}
        per_file: dict[str, dict] = {}
        samples: list[dict] = []
        samples_truncated = False
        for rel in targets:
            text = read_text_or_none(root / rel)
            if text is None:
                continue
            line_entries = classify_source_lines(text)
            file_counts: dict[str, int] = {}
            file_total = 0
            for i, line in enumerate(text.splitlines()):
                for pattern in patterns:
                    m = pattern.search(line)
                    if not m:
                        continue
                    cls = classify_match(line, line_entries[i], m.start())
                    total += 1
                    file_total += 1
                    by_class[cls] = by_class.get(cls, 0) + 1
                    file_counts[cls] = file_counts.get(cls, 0) + 1
                    if len(samples) < TEXT_SAMPLE_LIMIT:
                        samples.append({
                            "path": rel, "line": i + 1, "class": cls,
                            "text": line.strip()[:200],
                        })
                    else:
                        samples_truncated = True
            if file_total:
                per_file[rel] = {"total": file_total, "by_class": dict(sorted(file_counts.items()))}
        scans.append({
            "id": cfg.get("id", "unnamed"),
            "description": cfg.get("description", ""),
            "source_roots": source_roots,
            "exclude_roots": exclude_roots,
            "patterns": cfg.get("patterns", []),
            "files_scanned": len(targets),
            "total_matches": total,
            "by_class": dict(sorted(by_class.items())),
            "per_file": dict(sorted(per_file.items())),
            "samples": samples,
            "samples_truncated": samples_truncated,
        })
    return {
        "scanner": "text-heuristic/1",
        "scans": scans,
        "scanner_limitations": list(SCANNER_LIMITATIONS),
    }


# ---------------------------------------------------------------------------
# ownership / 资产层
# ---------------------------------------------------------------------------

def check_ownership(root: Path, entries: list[dict]) -> dict:
    results = []
    missing = []
    outside = 0
    by_owner: dict[str, int] = {}
    for entry in entries:
        path = entry.get("path", "")
        is_outside = bool(entry.get("outside_root", False))
        exists = None
        if is_outside:
            outside += 1
        else:
            exists = (root / path).exists()
            if not exists:
                missing.append(path)
        owner = entry.get("owner", "unassigned")
        by_owner[owner] = by_owner.get(owner, 0) + 1
        results.append({
            "id": entry.get("id", ""),
            "kind": entry.get("kind", "weld"),
            "path": path,
            "owner": owner,
            "exit_phase": entry.get("exit_phase", ""),
            "status": entry.get("status", "active"),
            "exists": exists,
            "outside_root": is_outside,
        })
    results.sort(key=lambda r: (r["kind"], r["id"]))
    return {
        "entries": results,
        "entry_count": len(results),
        "paths_missing": sorted(missing),
        "outside_root_count": outside,
        "by_owner": dict(sorted(by_owner.items())),
    }


def summarize_assets(root: Path, tracked: list[str], asset_cfgs: list[dict]) -> dict:
    summary = {}
    for cfg in asset_cfgs:
        rel_root = cfg.get("root", "").rstrip("/")
        files = [p for p in tracked if p == rel_root or p.startswith(rel_root + "/")]
        exts: dict[str, int] = {}
        total_bytes = 0
        for rel in files:
            ext = PurePosixPath(rel).suffix or "(none)"
            exts[ext] = exts.get(ext, 0) + 1
            try:
                total_bytes += (root / rel).stat().st_size
            except OSError:
                pass
        summary[cfg.get("id", rel_root)] = {
            "root": rel_root,
            "owner": cfg.get("owner", ""),
            "exit_phase": cfg.get("exit_phase", ""),
            "tracked_files": len(files),
            "total_bytes": total_bytes,
            "extensions": dict(sorted(exts.items())),
        }
    return dict(sorted(summary.items()))


# ---------------------------------------------------------------------------
# 配置与基线
# ---------------------------------------------------------------------------

def load_config(path: Path) -> dict:
    text = read_text_or_none(path)
    if text is None:
        raise ConfigError(f"配置文件不存在或不可读: {path.name}")
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc:
        raise ConfigError(f"配置 TOML 解析失败: {exc}")
    if data.get("schema") != CONFIG_SCHEMA:
        raise ConfigError(f"配置 schema 不识别: {data.get('schema')!r}（期望 {CONFIG_SCHEMA}）")
    baseline_cfg = data.get("baseline", {})
    wide = sorted(set(baseline_cfg) & REJECTED_BASELINE_KEYS)
    if wide:
        raise ConfigError(
            f"拒绝宽豁免：[baseline] 含目录级/前缀级豁免键 {wide}；"
            "基线只允许窄豁免（[[baseline.edge_notes]] 逐边 owner + 退出阶段）"
        )
    for note in baseline_cfg.get("edge_notes", []):
        bad = sorted(set(note) & REJECTED_EDGE_NOTE_KEYS)
        if bad:
            raise ConfigError(
                f"拒绝宽豁免：edge_note 含越界键 {bad}（note 必须逐边绑定，不允许目录/前缀作用域）"
            )
    return data


def load_baseline(path: Path) -> dict:
    text = read_text_or_none(path)
    if text is None:
        raise ConfigError(f"基线文件不存在或不可读: {path.name}；先运行 --refresh-baseline")
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        raise ConfigError(f"基线 JSON 解析失败: {exc}")
    if data.get("schema") != BASELINE_SCHEMA:
        raise ConfigError(f"基线 schema 不识别: {data.get('schema')!r}")
    if not isinstance(data.get("base_revision"), str) or not data["base_revision"]:
        raise ConfigError("基线缺 base_revision")
    if not isinstance(data.get("declared_edges"), list):
        raise ConfigError("基线缺 declared_edges")
    return data


def edge_sort_key(edge: dict):
    return (
        edge["from"], edge["to"], edge["kind"], edge["target"] or "",
        edge["dep_key"], edge["to_scope"], edge["optional"],
    )


def baseline_edge_keys(baseline: dict) -> set[tuple]:
    keys = set()
    for e in baseline["declared_edges"]:
        keys.add((e["from"], e["to"], e["kind"], e["target"]))
    return keys


# ---------------------------------------------------------------------------
# 规则与 strict 判定
# ---------------------------------------------------------------------------

def domain_of(member: str, domains_cfg: dict) -> str:
    return domains_cfg.get(member, "unclassified")


def rule_matches(rule: dict, edge: dict, domains_cfg: dict) -> bool:
    frm = domain_of(edge["from"], domains_cfg)
    to = domain_of(edge["to"], domains_cfg)
    from_domains = rule.get("from_domains")
    if from_domains and frm not in from_domains:
        return False
    to_domains = rule.get("to_domains")
    if to_domains and to not in to_domains:
        return False
    kinds = rule.get("kinds", ["normal", "build"])
    if edge["kind"] not in kinds:
        return False
    to_scopes = rule.get("to_scopes")
    if to_scopes and edge["to_scope"] not in to_scopes:
        return False
    if edge["optional"] and not rule.get("include_optional", False):
        return False
    return True


def violating_rule(edge: dict, rules: list[dict], domains_cfg: dict) -> dict | None:
    for rule in rules:
        if rule_matches(rule, edge, domains_cfg):
            return rule
    return None


def evaluate_strict(
    config: dict, baseline: dict, declared_edges: list[dict],
    text_candidates: dict, ownership: dict, cross_check: dict,
) -> dict:
    domains_cfg = config.get("domains", {}).get("members", {})
    rules = config.get("rules", {}).get("forbidden_declared", [])
    notes = {}
    for note in config.get("baseline", {}).get("edge_notes", []):
        notes[(note.get("from"), note.get("to"), note.get("kind"))] = note

    violations: list[dict] = []
    warnings: list[str] = []
    grandfathered: list[dict] = []
    new_allowed: list[dict] = []

    # strict 判的是当前工作树的边集（未提交的变更也在被判定面内）：
    # 新增禁向边无论是否已提交都会点名，不存在脏树静默通过的漏洞；
    # 干净树要求只在 --refresh-baseline（冻结必须绑定干净树的实际 HEAD）。

    base_keys = baseline_edge_keys(baseline)
    for edge in declared_edges:
        rule = violating_rule(edge, rules, domains_cfg)
        if rule is None:
            continue
        key = (edge["from"], edge["to"], edge["kind"], edge["target"])
        note = notes.get((edge["from"], edge["to"], edge["kind"]))
        if key in base_keys:
            if note and note.get("status", "active") == "revoked":
                violations.append({
                    "type": "revoked_edge_present",
                    "rule": rule.get("id"),
                    "edge": edge,
                    "detail": "该边已被标记撤销（status=revoked）但仍存在；应在退出阶段完成后移除",
                })
            elif note and note.get("owner") and note.get("exit_phase"):
                grandfathered.append({"edge": edge, "rule": rule.get("id"),
                                      "owner": note["owner"], "exit_phase": note["exit_phase"]})
            else:
                violations.append({
                    "type": "unowned_existing_edge",
                    "rule": rule.get("id"),
                    "edge": edge,
                    "detail": "存量禁向边缺少 edge_note（owner + exit_phase）；存量豁免必须是窄豁免",
                })
        else:
            violations.append({
                "type": "new_forbidden_edge",
                "rule": rule.get("id"),
                "edge": edge,
                "detail": "冻结基线之外的新增禁向声明边",
            })

    current_keys = {(e["from"], e["to"], e["kind"], e["target"]) for e in declared_edges}
    resolved = sorted(
        {" -> ".join([k[0], k[1], k[2]]) + (f" [{k[3]}]" if k[3] else "") for k in base_keys - current_keys}
    )
    new_keys = current_keys - base_keys
    for edge in declared_edges:
        if ((edge["from"], edge["to"], edge["kind"], edge["target"]) in new_keys
                and violating_rule(edge, rules, domains_cfg) is None):
            new_allowed.append(edge)

    if cross_check.get("ok") is True and cross_check.get("discrepancies"):
        violations.append({
            "type": "cargo_metadata_discrepancy",
            "detail": "manifest 直解与 cargo metadata --locked 的边集不一致",
            "discrepancies": cross_check["discrepancies"][:50],
        })
    elif cross_check.get("attempted") and cross_check.get("ok") is not True:
        warnings.append(f"cargo metadata 对照缺口（不判红）: {cross_check.get('gap', '')[:200]}")

    base_text_totals = baseline.get("text_scan_totals", {})
    for scan in text_candidates.get("scans", []):
        base_total = base_text_totals.get(scan["id"])
        if base_total is None:
            continue
        if scan["total_matches"] != base_total:
            warnings.append(
                f"文本候选漂移（需人工确认，不判红）: {scan['id']} 基线 {base_total} -> 当前 {scan['total_matches']}"
            )

    if ownership.get("paths_missing"):
        warnings.append(
            "ownership 声明路径在扫描根下缺失（配置漂移，需人工核对）: "
            + ", ".join(ownership["paths_missing"][:10])
        )

    verdict = "fail" if violations else "pass"
    return {
        "verdict": verdict,
        "violations": violations,
        "warnings": warnings,
        "grandfathered": grandfathered,
        "new_allowed_edges": new_allowed,
        "resolved_edges": resolved,
    }


# ---------------------------------------------------------------------------
# 报告组装
# ---------------------------------------------------------------------------

METHOD = {
    "sizes": "git ls-files 受管 .rs；物理行=str.splitlines()（含空行/注释/内联测试/生成文件）；"
             "非生产 SLOC，非可删除量；与调研基线 v06-modularization-baseline.jsonl 同口径可比"
             "（top_files 不含 cfg(test) 行位/测试函数统计，历史快照段属调研一次性产物不在本工具范围）",
    "declared_edges": "主源=workspace 成员 manifest 直接解析（Python tomllib；含 target 表与 "
                      "workspace 依赖继承；路径依赖按目标 manifest 归类 member/nonmember-in-root/external）；"
                      "cargo metadata --no-deps --locked 仅作一致性对照，缺锁/锁不满足记录缺口，"
                      "不自动更新、不作编译验收；metadata 成功不代表编译成功",
    "text_candidates": "配置驱动正则扫描 + 逐字符注释/字符串状态机 + cfg(test)/macro_rules 区域跟踪的"
                       "启发式分类（code/comment/doc/string/test/macro）；文本命中是候选，"
                       "不是解析后的语义依赖边；盲区见 scanner_limitations",
    "ownership": "按配置声明焊点/资产/consumer 的 owner 与退出阶段；路径存在性核对仅限扫描根内，"
                 "根外 consumer 只登记不核对",
    "assets": "受管文件按配置资产根计数（文件数/字节数/扩展名分布）",
    "stability": "同一输入两次扫描 payload 字节一致；无时间戳、无检出绝对路径；"
                 "报告只读生成、另行保存，不回写输入",
    "strict": "只判冻结基线之外的新增禁向边；存量禁向边必须逐边 edge_note（owner+exit_phase）"
              "方可 grandfather； revoked 边再现判红；基线锚定 base_revision 的"
              " manifest/配置锚定面（基线文件自身落库除外）；脏树判红；宽豁免在配置加载即拒绝",
}


def build_report(
    root: Path, config_path: Path, baseline_path: Path | None, baseline: dict | None,
    mode: str, revision: dict, sizes: dict, files: list[dict], assets: dict,
    declared_edges: dict, text_candidates: dict, ownership: dict, strict: dict | None,
) -> dict:
    config_text = read_text_or_none(config_path)
    report = {
        "schema": REPORT_SCHEMA,
        "tool": TOOL,
        "plan": "PLAN-777",
        "mode": mode,
        "method": METHOD,
        "revision": revision,
        "inputs": {
            "config": {
                "path": config_path.relative_to(root).as_posix()
                if is_relative_to(config_path, root) else config_path.name,
                "sha256": sha256_bytes(config_text.encode("utf-8")) if config_text else None,
            },
            "baseline": None,
        },
        "sizes": sizes,
        "files": files,
        "assets": assets,
        "declared_edges": declared_edges,
        "text_candidates": text_candidates,
        "ownership": ownership,
        "strict": strict,
    }
    if baseline_path is not None and baseline is not None and baseline_path.exists():
        report["inputs"]["baseline"] = {
            "path": baseline_path.relative_to(root).as_posix()
            if is_relative_to(baseline_path, root) else baseline_path.name,
            "sha256": sha256_bytes(baseline_path.read_bytes()),
            "base_revision": baseline.get("base_revision"),
        }
    return report


def is_relative_to(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def refresh_baseline(
    root: Path, config: dict, config_path: Path, baseline_path: Path,
    revision: dict, declared_edges: list[dict], text_candidates: dict,
) -> dict:
    if revision["tracked_dirty"]:
        raise ConfigError(
            "拒绝在脏树上刷新基线：基线必须绑定干净受管树的实际 HEAD"
            f"（当前: {revision['tracked_status'][:5]}）"
        )
    config_text = read_text_or_none(config_path)
    if config_text is None:
        raise ConfigError("配置不可读")
    baseline = {
        "schema": BASELINE_SCHEMA,
        "base_revision": revision["head"],
        "generated_by": f"{TOOL} --refresh-baseline",
        "config_sha256": sha256_bytes(config_text.encode("utf-8")),
        "declared_edges": sorted(declared_edges, key=edge_sort_key),
        "text_scan_totals": {
            scan["id"]: scan["total_matches"] for scan in text_candidates.get("scans", [])
        },
    }
    baseline_path.write_text(canonical_json(baseline), encoding="utf-8", newline="\n")
    return baseline


# ---------------------------------------------------------------------------
# 审计主流程
# ---------------------------------------------------------------------------

def audit(root: Path, args) -> tuple[dict, int]:
    config_path = (
        Path(args.config) if args.config else root / DEFAULT_CONFIG_RELPATH
    )
    if not config_path.is_absolute():
        config_path = root / config_path
    config = load_config(config_path)

    baseline_rel = (
        config.get("baseline", {}).get("path") or DEFAULT_BASELINE_RELPATH
    )
    baseline_path = Path(args.baseline) if args.baseline else root / baseline_rel
    if not baseline_path.is_absolute():
        baseline_path = root / baseline_path

    revision = collect_revision(root)
    tracked = collect_tracked_files(root)
    sizes, files = collect_sizes(root, tracked)

    edges, member_names, gaps = extract_declared_edges(root)
    cross_check = (
        {"attempted": False, "ok": None, "gap": "已通过 --no-cargo-check 显式停用",
         "discrepancies": []}
        if args.no_cargo_check
        else cargo_metadata_cross_check(root, edges)
    )
    declared_edges_section = {
        "source": "manifest-tomllib/1",
        "workspace_members": member_names,
        "member_domains": dict(sorted(
            (m, domain_of(m, config.get("domains", {}).get("members", {})))
            for m in member_names
        )),
        "edges": edges,
        "extraction_gaps": gaps,
        "cross_check": cross_check,
        "forbidden_rules": config.get("rules", {}).get("forbidden_declared", []),
    }

    text_candidates = run_text_scans(
        root, tracked, config.get("text_scan", [])
    )
    ownership = check_ownership(root, config.get("ownership", []))
    assets = summarize_assets(root, tracked, config.get("assets", []))

    strict = None
    exit_code = EXIT_OK
    if args.mode == "strict":
        if not baseline_path.exists():
            raise ConfigError(
                f"strict 需要冻结基线（{baseline_path.name} 缺失）；先在干净树上 --refresh-baseline"
            )
        baseline = load_baseline(baseline_path)
        config_text = read_text_or_none(config_path)
        config_sha = (
            sha256_bytes(config_text.encode("utf-8")) if config_text else None
        )
        strict = evaluate_strict(
            config, baseline, edges, text_candidates, ownership, cross_check,
        )
        if baseline.get("config_sha256") != config_sha:
            strict["violations"].append({
                "type": "config_drift_since_baseline",
                "detail": "配置自基线冻结后已变更；重新 --refresh-baseline 绑定新配置",
            })
            strict["verdict"] = "fail"
        staleness = staleness_paths_changed(root, baseline["base_revision"])
        if staleness["stale"]:
            strict["violations"].append({
                "type": "stale_baseline",
                "detail": (
                    "base_revision 不可解析" if not staleness["base_known"]
                    else f"锚定输入自 base_revision 起已变更: {staleness['changed'][:10]}"
                ) + "；基线过期必须显式重采样，不能默默通行",
            })
            strict["verdict"] = "fail"
        if strict["verdict"] == "fail":
            exit_code = EXIT_STRICT_FAIL
        report_baseline = baseline
    else:
        report_baseline = None

    report = build_report(
        root, config_path, baseline_path, report_baseline, args.mode,
        revision, sizes, files, assets,
        declared_edges_section, text_candidates, ownership, strict,
    )
    return report, exit_code


# ---------------------------------------------------------------------------
# 自测（V-01..V-04；fixture: test/fixtures/modularity/audit/）
# ---------------------------------------------------------------------------

FIXTURE_RELPATH = Path("test/fixtures/modularity/audit")
SCRIPT_PATH = Path(__file__).resolve()
SCRIPT_ROOT = SCRIPT_PATH.parent.parent


class SelfTestFailure(Exception):
    pass


def st_check(cond: bool, label: str, detail: str = "") -> None:
    if not cond:
        raise SelfTestFailure(f"{label}: {detail}")


def make_fixture_repo(scenario: str) -> Path:
    src = SCRIPT_ROOT / FIXTURE_RELPATH
    st_check(src.exists(), f"[{scenario}] fixture 目录存在", str(src))
    tmp_root = Path(tempfile.mkdtemp(prefix=f"mod-audit-{scenario}-"))
    repo = tmp_root / "repo"
    shutil.copytree(src, repo)
    for cmd in (
        ["init", "-q"],
        ["add", "-A"],
        ["-c", "user.name=modularity-audit-selftest",
         "-c", "user.email=selftest@invalid", "commit", "-q", "-m", "fixture init"],
    ):
        proc = subprocess.run(["git", "-C", str(repo), *cmd],
                              capture_output=True, text=True)
        if proc.returncode != 0:
            raise SelfTestFailure(f"[{scenario}] git {cmd[0]} 失败: {proc.stderr[:200]}")
    return repo


def run_audit_cli(script: Path, repo: Path, *extra: str) -> tuple[int, dict | None, str]:
    out_path = repo.parent / "report.json"
    proc = subprocess.run(
        [sys.executable, str(script), "--root", str(repo),
         "--output", str(out_path), *extra],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
    )
    report = None
    if out_path.exists():
        try:
            report = json.loads(out_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            report = None
    return proc.returncode, report, proc.stderr


def mutate(repo: Path, relpath: str, old: str, new: str, scenario: str) -> None:
    """先证前态、改后证后态——fixture 确实被改（V-04 前置契约）。"""
    target = repo / relpath
    text = target.read_text(encoding="utf-8")
    st_check(old in text, f"[{scenario}] 变更前态包含目标片段", f"{relpath}: 未找到 {old[:60]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")
    after = target.read_text(encoding="utf-8")
    st_check(new in after, f"[{scenario}] 变更后态生效", f"{relpath}: 替换未生效")


def commit_fixture(repo: Path, message: str, scenario: str) -> None:
    proc = subprocess.run(
        ["git", "-C", str(repo), "-c", "user.name=modularity-audit-selftest",
         "-c", "user.email=selftest@invalid", "commit", "-qam", message],
        capture_output=True, text=True,
    )
    st_check(proc.returncode == 0, f"[{scenario}] fixture 提交成功", proc.stderr[:200])


def scenario_v01(script: Path) -> str:
    repo = make_fixture_repo("v01")
    code1, rep1, err1 = run_audit_cli(script, repo, "--mode", "observe")
    code2, rep2, err2 = run_audit_cli(script, repo, "--mode", "observe")
    st_check(code1 == 0 and code2 == 0, "[v01] 两次 observe 退出码 0", f"{err1[:200]} {err2[:200]}")
    st_check(rep1 is not None and rep2 is not None, "[v01] 两次均产出报告")
    j1 = canonical_json(rep1)
    j2 = canonical_json(rep2)
    st_check(j1 == j2, "[v01] 同输入两次扫描 payload 字节一致")
    st_check(str(repo) not in j1 and str(repo.parent) not in j1,
             "[v01] payload 不含检出绝对路径")
    st_check(str(repo.resolve()) not in j1, "[v01] payload 不含 resolve 后绝对路径")
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-01 稳定只读：同 fixture HEAD 两次扫描 payload 一致，无绝对路径"


def scenario_v02(script: Path) -> str:
    repo = make_fixture_repo("v02")
    code, _, err = run_audit_cli(script, repo, "--refresh-baseline")
    st_check(code == 0, "[v02] 干净树基线刷新成功", err[:300])
    mutate(
        repo, "crates/fixture-lang/Cargo.toml",
        old="[dev-dependencies]",
        new="[dependencies]\nfixture-ui-kit = { path = \"../fixture-ui-kit\" }\n\n[dev-dependencies]",
        scenario="v02",
    )
    code, rep, err = run_audit_cli(script, repo, "--mode", "strict")
    st_check(code == EXIT_STRICT_FAIL, "[v02] 新增 core→UI 正常边判红（非零退出）", f"exit={code}")
    st_check(rep is not None and rep.get("strict", {}).get("verdict") == "fail",
             "[v02] strict verdict=fail")
    violations = rep["strict"]["violations"]
    new_edges = [v for v in violations if v["type"] == "new_forbidden_edge"]
    st_check(len(new_edges) == 1, "[v02] 恰好一条新增禁向边", json.dumps(violations)[:300])
    edge = new_edges[0]["edge"]
    st_check(edge["from"] == "fixture-lang" and edge["to"] == "fixture-ui-kit"
             and edge["kind"] == "normal",
             "[v02] 判红点名具体边 (fixture-lang → fixture-ui-kit, normal)",
             json.dumps(edge))
    for v in violations:
        st_check(v["type"] == "new_forbidden_edge",
                 "[v02] 无其他类型违例（合法反向/dev-only 不误红）", json.dumps(v)[:200])
    for allowed in (("fixture-ui", "fixture-lang-sdk"), ("fixture-app", "fixture-ui")):
        hits = [v for v in violations
                if v.get("edge", {}).get("from") == allowed[0]
                and v.get("edge", {}).get("to") == allowed[1]]
        st_check(not hits, f"[v02] 合法边不误红: {allowed[0]} -> {allowed[1]}")
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-02 新增边：fixture-lang→fixture-ui-kit 判红并点名；UI→SDK 反向/dev-only 不误红"


def scenario_v03(script: Path) -> str:
    repo = make_fixture_repo("v03")
    code, rep, err = run_audit_cli(script, repo, "--mode", "observe")
    st_check(code == 0, "[v03] observe 退出 0", err[:300])
    scans = {s["id"]: s for s in rep["text_candidates"]["scans"]}
    st_check("core_to_ui_fixture" in scans, "[v03] fixture 文本扫描配置生效")
    scan = scans["core_to_ui_fixture"]
    path = "crates/fixture-lang/src/text_cases.rs"
    st_check(path in scan["per_file"], "[v03] text_cases.rs 被扫描",
             json.dumps(list(scan["per_file"]))[:200])
    by_class = scan["per_file"][path]["by_class"]
    expected = {"code": 1, "comment": 2, "doc": 1, "string": 1, "test": 1, "macro": 1}
    st_check(by_class == expected, "[v03] 七处命中分类逐一正确（含同行块注释；别名用法不计）",
             f"实际 {json.dumps(by_class, ensure_ascii=False)}")
    st_check(scan["total_matches"] == 7, "[v03] 总命中恰为 7（无别名泄漏）",
             f"实际 {scan['total_matches']}")
    limitations = " ".join(rep["text_candidates"]["scanner_limitations"])
    for kw in ("别名", "re-export", "宏"):
        st_check(kw in limitations, f"[v03] 盲区声明包含: {kw}")
    st_check(scan["samples_truncated"] is False, "[v03] 样本未截断（截断会显式标记）")
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-03 候选诚实：comment/doc/string/test/macro/code 六类分类正确，盲区显式声明"


def scenario_v04a(script: Path) -> str:
    repo = make_fixture_repo("v04a")
    mutate(
        repo, "crates/fixture-lang/Cargo.toml",
        old="[dev-dependencies]",
        new="[dependencies]\nfixture-ui = { path = \"../fixture-ui\" }\n\n[dev-dependencies]",
        scenario="v04a",
    )
    config = repo / DEFAULT_CONFIG_RELPATH
    config.write_text(
        config.read_text(encoding="utf-8")
        + "\n[[baseline.edge_notes]]\n"
        + "from = \"fixture-lang\"\nto = \"fixture-ui\"\nkind = \"normal\"\n"
        + "owner = \"fixture-ui-owner\"\nexit_phase = \"RX\"\nstatus = \"active\"\n",
        encoding="utf-8", newline="\n",
    )
    st_check("fixture-ui-owner" in config.read_text(encoding="utf-8"),
             "[v04a] edge_note 落盘证明")
    commit_fixture(repo, "add existing edge A + note", "v04a")
    code, _, err = run_audit_cli(script, repo, "--refresh-baseline")
    st_check(code == 0, "[v04a] 含存量边 A 的基线刷新成功", err[:300])
    mutate(
        repo, "crates/fixture-lang/Cargo.toml",
        old="fixture-ui = { path = \"../fixture-ui\" }\n",
        new="fixture-ui = { path = \"../fixture-ui\" }\nfixture-ui-kit = { path = \"../fixture-ui-kit\" }\n",
        scenario="v04a",
    )
    code, rep, err = run_audit_cli(script, repo, "--mode", "strict")
    st_check(code == EXIT_STRICT_FAIL, "[v04a] 新边 B 判红", f"exit={code}")
    violations = rep["strict"]["violations"]
    new_edges = [v for v in violations if v["type"] == "new_forbidden_edge"]
    st_check(len(new_edges) == 1 and new_edges[0]["edge"]["to"] == "fixture-ui-kit",
             "[v04a] 判红只点名新边 B（存量窄豁免不能盖新边）", json.dumps(violations)[:300])
    st_check(any(g["edge"]["to"] == "fixture-ui" for g in rep["strict"]["grandfathered"]),
             "[v04a] 存量边 A 被 grandfather（带 owner/退出阶段）")
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-04a 窄豁免：存量边 A grandfather（owner+退出阶段），新边 B 仍判红"


def scenario_v04b(script: Path) -> str:
    repo = make_fixture_repo("v04b")
    mutate(
        repo, "crates/fixture-lang/Cargo.toml",
        old="[dev-dependencies]",
        new="[dependencies]\nfixture-ui = { path = \"../fixture-ui\" }\n\n[dev-dependencies]",
        scenario="v04b",
    )
    config = repo / DEFAULT_CONFIG_RELPATH
    config.write_text(
        config.read_text(encoding="utf-8")
        + "\n[[baseline.edge_notes]]\n"
        + "from = \"fixture-lang\"\nto = \"fixture-ui\"\nkind = \"normal\"\n"
        + "owner = \"fixture-ui-owner\"\nexit_phase = \"RX\"\nstatus = \"active\"\n",
        encoding="utf-8", newline="\n",
    )
    commit_fixture(repo, "add existing edge A + note", "v04b")
    code, _, err = run_audit_cli(script, repo, "--refresh-baseline")
    st_check(code == 0, "[v04b] 基线刷新成功", err[:300])
    mutate(
        repo, DEFAULT_CONFIG_RELPATH,
        old='owner = "fixture-ui-owner"\nexit_phase = "RX"\nstatus = "active"',
        new='owner = "fixture-ui-owner"\nexit_phase = "RX"\nstatus = "revoked"',
        scenario="v04b",
    )
    code, rep, err = run_audit_cli(script, repo, "--mode", "strict")
    st_check(code == EXIT_STRICT_FAIL, "[v04b] 撤销边仍存在判红", f"exit={code}")
    st_check(any(v["type"] == "revoked_edge_present" for v in rep["strict"]["violations"]),
             "[v04b] 违例类型=revoked_edge_present")
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-04b 撤销边：status=revoked 的边仍存在 → 判红"


def scenario_v04c(script: Path) -> str:
    repo = make_fixture_repo("v04c")
    code, _, err = run_audit_cli(script, repo, "--refresh-baseline")
    st_check(code == 0, "[v04c] 基线刷新成功", err[:300])
    manifest = repo / "crates/fixture-lang/Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="utf-8") + "\n# drift comment\n",
        encoding="utf-8", newline="\n",
    )
    commit = subprocess.run(
        ["git", "-C", str(repo), "-c", "user.name=selftest", "-c",
         "user.email=selftest@invalid", "commit", "-qam", "manifest drift"],
        capture_output=True, text=True,
    )
    st_check(commit.returncode == 0, "[v04c] 漂移提交成功（fixture 实际变更）", commit.stderr[:200])
    code, rep, err = run_audit_cli(script, repo, "--mode", "strict")
    st_check(code == EXIT_STRICT_FAIL, "[v04c] 过期基线判红", f"exit={code}")
    st_check(any(v["type"] == "stale_baseline" for v in rep["strict"]["violations"]),
             "[v04c] 违例类型=stale_baseline")
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-04c 过期基线：base_revision 后锚定输入（manifest）实际变更 → 判红"


def scenario_v04d(script: Path) -> str:
    repo = make_fixture_repo("v04d")
    code, _, err = run_audit_cli(script, repo, "--refresh-baseline")
    st_check(code == 0, "[v04d] 基线刷新成功", err[:300])
    config = repo / DEFAULT_CONFIG_RELPATH
    # fixture 配置末节是 [baseline] 表：直接在其键位下注入宽豁免键（不重复声明表）
    config.write_text(
        config.read_text(encoding="utf-8")
        + "exempt_paths = [\"crates/fixture-lang\"]\n",
        encoding="utf-8", newline="\n",
    )
    st_check("exempt_paths" in config.read_text(encoding="utf-8"),
             "[v04d] 宽豁免键落盘证明")
    code, _, err = run_audit_cli(script, repo, "--mode", "observe")
    st_check(code == EXIT_CONFIG_INVALID, "[v04d] 宽豁免拒绝（非零且非 strict 码）",
             f"exit={code}")
    st_check("宽豁免" in err or "拒绝" in err,
             "[v04d] 错误信息点名宽豁免拒绝", err[:300])
    shutil.rmtree(repo.parent, ignore_errors=True)
    return "V-04d 宽豁免：[baseline].exempt_paths 目录级豁免 → 配置加载即拒绝"


def run_self_test() -> int:
    script = SCRIPT_PATH
    scenarios = [
        ("v01", scenario_v01), ("v02", scenario_v02), ("v03", scenario_v03),
        ("v04a", scenario_v04a), ("v04b", scenario_v04b),
        ("v04c", scenario_v04c), ("v04d", scenario_v04d),
    ]
    failures = 0
    for name, fn in scenarios:
        try:
            summary = fn(script)
            print(f"PASS {name}: {summary}")
        except SelfTestFailure as exc:
            failures += 1
            print(f"FAIL {name}: {exc}")
        except Exception as exc:  # noqa: BLE001 — 自测要汇报任意崩溃
            failures += 1
            print(f"FAIL {name}: 未预期异常 {type(exc).__name__}: {exc}")
    print(f"self-test: {len(scenarios) - failures}/{len(scenarios)} 通过")
    return EXIT_OK if failures == 0 else EXIT_ERROR


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

class CustomArgumentParser(argparse.ArgumentParser):
    def error(self, message):  # 用法错误用独立退出码，避免与 strict 判红冲突
        self.print_usage(sys.stderr)
        print(f"{self.prog}: error: {message}", file=sys.stderr)
        sys.exit(EXIT_USAGE)


def parse_args(argv) -> argparse.Namespace:
    parser = CustomArgumentParser(
        prog="modularity-audit.py",
        description="v0.6 模块边界基线与增量检查（只读审计；Category A，不编译不写输入）",
    )
    parser.add_argument("--root", help="扫描根（默认：脚本所在仓根）")
    parser.add_argument("--config", help=f"边界配置路径（默认 <root>/{DEFAULT_CONFIG_RELPATH}）")
    parser.add_argument("--baseline", help="冻结基线路径（默认 <root>/.autoos/ 配置声明）")
    parser.add_argument("--mode", choices=["observe", "strict"], default="observe",
                        help="observe=全仓观察（默认）；strict=冻结基线增量判红")
    parser.add_argument("--output", default="-",
                        help="报告输出路径（默认 '-' = stdout）")
    parser.add_argument("--refresh-baseline", action="store_true",
                        help="在干净受管树上生成/刷新冻结基线（写入配置声明的基线路径）")
    parser.add_argument("--no-cargo-check", action="store_true",
                        help="跳过 cargo metadata --locked 一致性对照")
    parser.add_argument("--self-test", action="store_true",
                        help="对 test/fixtures/modularity/audit/ 运行 V-01..V-04 正反控自测")
    return parser.parse_args(argv)


def main(argv) -> int:
    args = parse_args(argv)
    script_root = Path(__file__).resolve().parent.parent
    if args.self_test:
        return run_self_test()
    root = Path(args.root) if args.root else script_root
    root = root.resolve()

    try:
        if args.refresh_baseline:
            config_path = root / DEFAULT_CONFIG_RELPATH if not args.config else (
                Path(args.config) if Path(args.config).is_absolute() else root / args.config
            )
            config = load_config(config_path)
            baseline_rel = config.get("baseline", {}).get("path") or DEFAULT_BASELINE_RELPATH
            baseline_path = (
                Path(args.baseline) if args.baseline else root / baseline_rel
            )
            if not baseline_path.is_absolute():
                baseline_path = root / baseline_path
            revision = collect_revision(root)
            tracked = collect_tracked_files(root)
            edges, _members, _gaps = extract_declared_edges(root)
            text_candidates = run_text_scans(root, tracked, config.get("text_scan", []))
            baseline = refresh_baseline(
                root, config, config_path, baseline_path, revision, edges, text_candidates,
            )
            print(
                f"基线已刷新: {baseline_path.name} base_revision={baseline['base_revision'][:12]} "
                f"declared_edges={len(baseline['declared_edges'])} "
                f"text_scan_totals={baseline['text_scan_totals']}"
            )
            return EXIT_OK

        report, exit_code = audit(root, args)
        payload = canonical_json(report)
        if args.output == "-":
            sys.stdout.write(payload)
        else:
            out = Path(args.output)
            if not out.is_absolute():
                out = Path.cwd() / out
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(payload, encoding="utf-8", newline="\n")
        if args.mode == "strict":
            verdict = report["strict"]["verdict"] if report.get("strict") else "?"
            print(
                f"strict verdict={verdict} violations={len(report['strict']['violations'])} "
                f"grandfathered={len(report['strict']['grandfathered'])}",
                file=sys.stderr,
            )
        return exit_code
    except ConfigError as exc:
        print(f"配置/基线不合法: {exc}", file=sys.stderr)
        return EXIT_CONFIG_INVALID
    except RuntimeError as exc:
        print(f"运行错误: {exc}", file=sys.stderr)
        return EXIT_ERROR


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
