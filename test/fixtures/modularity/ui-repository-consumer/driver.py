#!/usr/bin/env python3
"""PLAN-831 T-03 ui-repository-consumer driver —— 五族验证设施的冻结接口实现。

本 driver 是 PLAN-831「auto-ui 独立仓承接」的三仓（auto-lang/auto-ui/auto-down）
fixture 驱动器。T-03 冻结接口并执行预迁移态的有效控制；T-05..T-09 按
migration-phases.json 的 successor_execution 以最终来源完善执行。

族（families）：
  source        V-01 单源/DAG：五包闭包禁断（预迁移=无 auto-man 大库/无
                lang-monolith；T-06 切源后同一断言收紧到零 auto-man-core）、
                UI 仓 vendoring 走私守卫、autodown 跨仓边台账。
  type          V-02 兼容/声明：旧 auto_lang 面 touchpoint 锚点钉住 +
                C2 声明符号单源（exactly-one）+ feature 名单清点。
                编译探针（--compile-probe，F831-09-02 起真实执行：
                三 feature 组合实编 + 双 source 负控；默认零 cargo）。
  test-id       V-06 测试身份：五包测试 ID 清点（预迁移=源码枚举法，收据
                显式标注方法与盲区；正典法 cargo -- --list 归 T-06+）+
                cargo 档位别名表快照。
  distribution  V-03 发行面：五形态（fresh-lock/locked-build/A→B 重定位/
                外部 SDK 反向控制/tool+资产 profile）接口冻结，
                phase=source-switched 起 T-07 执行；现行同构控制=
                test/fixtures/modularity/generated-ui-consumer/driver.py（792/815-A）。
  failure       V-04 失败面：六形态（缺源·producer 漂移/坏 pin/漏 patch/
                资源四态篡改/取消·交换失败保旧/陌生目录不删）接口冻结，
                T-07/T-08 执行；现行同构控制=792 driver V03 + legacy-preservation.py。

用法（有界：默认零 cargo 调用；git rev-parse 三仓取 HEAD）：
  python test/fixtures/modularity/ui-repository-consumer/driver.py <family|all> \
      [--lang-repo <path>] [--ui-repo <path>] [--down-repo <path>] \
      [--phase auto|pre-migration|source-switched] [--output receipt.json] \
      [--compile-probe]

退出码：0 全部通过（recorded/pending 不判红）；2 断言红；3 用法/配置错；1 运行错。
三仓入口与解析序见 scripts/ui-repository-fixture.py。

payload 稳定性契约：同输入两次运行收据字节一致；不含时间戳、不含检出绝对路径。
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
PHASES_PATH = HERE / "migration-phases.json"
TOUCHPOINTS_PATH = HERE / "compat-touchpoints.toml"

RECEIPT_SCHEMA = "ui-repository-fixture-receipt/1"
FAMILIES = ("source", "type", "test-id", "distribution", "failure")

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_ASSERT_FAIL = 2
EXIT_USAGE = 3

# 测试 ID 源码枚举法覆盖的属性宏（cargo -- --list 正典法无此盲区，T-06+ 生效）。
TEST_ATTR_RE = re.compile(r"^\s*#\[(test|tokio::test[^\]]*)\]\s*$")
FN_NAME_RE = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)")
TIER_ALIASES = ("t", "tf", "tv", "tt", "tb", "tu", "taa", "ta")

TEST_ID_LIMITATIONS = [
    "源码枚举法（source-enumeration/1）：只统计显式 #[test]/#[tokio::test] 属性后邻 fn，",
    "宏生成测试、include! 展开与 cfg 门控外的属性不可见；正典法 cargo -- --list",
    "（T-06 起按迁移后来源重采）才是测试身份的对拍基线；本清点是先行钉住。",
]


class UsageError(Exception):
    pass


class AssertFail(Exception):
    pass


def log(msg: str) -> None:
    print(f"[ui-repo-fixture] {msg}", flush=True)


# ---------------------------------------------------------------------------
# 仓库上下文
# ---------------------------------------------------------------------------

class Repo:
    def __init__(self, role: str, path: Path):
        self.role = role
        self.path = path
        self.head: str | None = None
        self.present = (path / "Cargo.toml").is_file()
        if path.is_dir():
            proc = subprocess.run(
                ["git", "-C", str(path), "rev-parse", "HEAD"],
                capture_output=True, text=True,
            )
            if proc.returncode == 0:
                self.head = proc.stdout.strip()

    def receipt_view(self, placeholder: str) -> dict:
        return {"path": placeholder, "head": self.head, "has_manifest": self.present}


def read_text_or_none(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None


# ---------------------------------------------------------------------------
# Cargo 声明层（口径移植自 scripts/modularity-audit.py 的 manifest 直解主源：
# 含 glob 成员发现、target 表依赖、workspace.dependencies 继承 path 的基点语义）
# ---------------------------------------------------------------------------

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


def discover_members(root: Path) -> dict[str, Path]:
    """成员包名 → 成员目录（glob 成员展开；根 manifest 缺失返回空）。"""
    root_manifest = root / "Cargo.toml"
    if not root_manifest.exists():
        return {}
    try:
        data = tomllib.loads(root_manifest.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError:
        return {}
    members: dict[str, Path] = {}
    for entry in data.get("workspace", {}).get("members", []):
        if any(ch in entry for ch in "*?["):
            matches = sorted(
                p for p in root.glob(entry)
                if p.is_dir() and (p / "Cargo.toml").exists()
            )
            candidates = matches
        else:
            cand = root / entry
            candidates = [cand] if (cand / "Cargo.toml").exists() else []
        for member in candidates:
            name = manifest_package_name(member / "Cargo.toml")
            if name:
                members[name] = member
    return members


def extract_declared_edges(root: Path) -> tuple[list[dict], dict[str, Path]]:
    """全部成员的声明 path 边（from/to/dep_key/kind/optional/target_dir）+ 成员表。"""
    members = discover_members(root)
    member_dirs = {m.resolve(): name for name, m in members.items()}
    root_manifest = root / "Cargo.toml"
    workspace_deps: dict = {}
    if root_manifest.exists():
        try:
            workspace_deps = tomllib.loads(
                root_manifest.read_text(encoding="utf-8")
            ).get("workspace", {}).get("dependencies", {})
        except tomllib.TOMLDecodeError:
            pass

    edges: list[dict] = []
    for name, member in members.items():
        manifest = member / "Cargo.toml"
        try:
            data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError):
            continue
        dep_sections: list[tuple[str, dict | None]] = [
            ("dependencies", None), ("dev-dependencies", None), ("build-dependencies", None),
        ]
        for target_sec in (data.get("target") or {}).values():
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                dep_sections.append((section, target_sec))
        for section, target_table in dep_sections:
            table = data.get(section) if target_table is None else (target_table or {}).get(section)
            kind = {"dependencies": "normal", "dev-dependencies": "dev",
                    "build-dependencies": "build"}[section]
            for dep_key, spec in (table or {}).items():
                if isinstance(spec, str):
                    spec = {"version": spec}
                spec = dict(spec or {})
                path_base = member
                if spec.pop("workspace", False) and dep_key in workspace_deps:
                    inherited = workspace_deps[dep_key]
                    if isinstance(inherited, str):
                        inherited = {"version": inherited}
                    inherited = dict(inherited or {})
                    inherited.update({k: v for k, v in spec.items() if v is not None})
                    spec = inherited
                    path_base = root
                path_value = spec.get("path")
                if not isinstance(path_value, str):
                    continue
                target_dir = (path_base / path_value).resolve()
                edges.append({
                    "from": name,
                    "to": manifest_package_name(target_dir / "Cargo.toml"),
                    "dep_key": dep_key,
                    "kind": kind,
                    "optional": bool(spec.get("optional", False)),
                    "target_dir": target_dir,
                    "raw_path": path_value,
                    "in_repo": target_dir in member_dirs,
                })
    edges.sort(key=lambda e: (e["from"], e["to"] or "", e["kind"], e["dep_key"]))
    return edges, members


def closure_of(roots: set[str], members: dict[str, Path], edges: list[dict]) -> tuple[set[str], list[dict]]:
    """roots 出发的声明闭包（仓内 path 边递归；跨根/未解析边记名不递归）。

    返回 (闭包包名集, 进入闭包的边集)。normal/build/dev 含 optional 全收录。
    """
    reached = set(roots)
    reached_edges: list[dict] = []
    frontier = list(roots)
    by_from: dict[str, list[dict]] = {}
    for edge in edges:
        by_from.setdefault(edge["from"], []).append(edge)
    while frontier:
        pkg = frontier.pop()
        for edge in by_from.get(pkg, []):
            reached_edges.append(edge)
            target = edge["to"]
            if target and target not in reached and edge["in_repo"]:
                reached.add(target)
                frontier.append(target)
            elif target and target not in reached and not edge["in_repo"]:
                # 跨根边：目标名进入闭包（禁断判定按名），不递归其 manifest。
                reached.add(target)
    return reached, reached_edges


# ---------------------------------------------------------------------------
# 断言记账
# ---------------------------------------------------------------------------

class Ledger:
    def __init__(self, sanitize):
        self.sanitize = sanitize
        self.items: list[dict] = []

    def add(self, assertion_id: str, status: str, detail: str) -> bool:
        ok = status in ("pass", "recorded", "pending")
        self.items.append({"id": assertion_id, "status": status, "detail": self.sanitize(detail)})
        log(f"{status.upper():8} {assertion_id}: {detail}")
        return ok

    def fail_any(self) -> bool:
        return any(i["status"] == "fail" for i in self.items)

    def totals(self) -> dict:
        counts: dict[str, int] = {}
        for item in self.items:
            counts[item["status"]] = counts.get(item["status"], 0) + 1
        return {
            "assertions": len(self.items),
            "pass": counts.get("pass", 0),
            "fail": counts.get("fail", 0),
            "recorded": counts.get("recorded", 0),
            "pending": counts.get("pending", 0),
        }


def edge_label(edge: dict) -> str:
    target = edge["to"] or "unresolved"
    scope = "in-repo" if edge["in_repo"] else "cross-root"
    return (f"{edge['from']} -> {target} ({edge['kind']}"
            f"{', optional' if edge['optional'] else ''}, {scope}, key={edge['dep_key']})")


# ---------------------------------------------------------------------------
# source 族（V-01）
# ---------------------------------------------------------------------------

def _ledger_tuple(edge: dict) -> tuple:
    return (edge["from"], edge["to"], edge["kind"], edge["optional"])


def _fallback_resolve(edge: dict, repos: dict[str, Repo]) -> str | None:
    """跨根边目标自本树悬挂时，按文档解析序在兄弟/主检出仓内恢复包名。"""
    raw = edge["raw_path"].replace("\\", "/")
    for repo_key, marker in (("lang", "auto-lang/"), ("down", "auto-down/")):
        if marker in raw:
            alt = repos[repo_key].path / raw.split(marker, 1)[1]
            name = manifest_package_name(alt / "Cargo.toml")
            if name:
                return name
    return None


def run_source(ctx: dict, phases: dict, ledger: Ledger) -> None:
    lang, ui = ctx["repos"]["lang"], ctx["repos"]["ui"]
    phase = ctx["phase"]
    migrating = phases["migrating_packages"]
    forbidden = phases["forbidden"]

    if not lang.present:
        raise UsageError(f"语言仓不可解析（缺 Cargo.toml）: {lang.path}")
    lang_edges, lang_members = extract_declared_edges(lang.path)
    ui_edges, ui_members = (extract_declared_edges(ui.path) if ui.present else ([], {}))
    five = set(migrating)
    ban_pre = forbidden["closure_ban_pre_migration"]
    ban_post = forbidden["closure_ban_source_switched"]
    frozen_ledger = phases["transitional_man_core_edge_ledger"]["frozen_expected_pre_migration"]
    ledger_keys = {(x["from"], x["to"], x["kind"], x["optional"]) for x in frozen_ledger}
    # 窗口期 UI 仓侧台账 = 冻结台账中 from 为迁移包的子集（非迁移包的边留在语言侧）。
    window_ui_ledger = {k for k in ledger_keys if k[0] in five}

    # ---- SRC-P1：五包在场（阶段语义）----
    if phase == "pre-migration":
        missing = sorted(five - set(lang_members))
        ledger.add("SRC-P1", "pass" if not missing else "fail",
                   f"五包@lang_repo: {'全部在位' if not missing else '缺失 ' + ','.join(missing)}"
                   f"（lang 成员 {len(lang_members)}，ui 成员 {len(ui_members)}）")
    elif phase == "import-window":
        miss_ui = sorted(five - set(ui_members))
        lang_side = len(five & set(lang_members))
        ledger.add("SRC-P1", "pass" if not miss_ui else "fail",
                   f"五包候选@ui_repo: {'全部导入' if not miss_ui else '缺 ' + ','.join(miss_ui)}"
                   f"；语言仓权威保留 {lang_side}/5（窗口期设计态）")
    else:  # source-switched
        miss_ui = sorted(five - set(ui_members))
        ledger.add("SRC-P1", "pass" if not miss_ui else "fail",
                   f"五包@ui_repo: {'全部在位' if not miss_ui else '缺失 ' + ','.join(miss_ui)}"
                   f"（lang 成员 {len(lang_members)}，ui 成员 {len(ui_members)}）")

    # ---- 语言侧权威闭包（pre + window 共用 pre 口径）----
    if phase in ("pre-migration", "import-window"):
        reached_l, edges_l = closure_of(five & set(lang_members), lang_members, lang_edges)
        hits_l = sorted(edge_label(e) for e in edges_l if e["to"] in ban_pre)
        ledger.add("SRC-P2", "pass" if not hits_l else "fail",
                   (f"语言仓权威闭包 {len(reached_l)} 包零禁断（{'/'.join(ban_pre)}）"
                    if not hits_l else "语言仓权威面禁断边: " + "; ".join(hits_l)))
        obs_l = sorted(_ledger_tuple(e) for e in edges_l if e["to"] == "auto-man-core")
        exp_l = sorted(ledger_keys)
        if obs_l == exp_l:
            ledger.add("SRC-P3", "recorded",
                       f"语言仓过渡 Man-core 台账=冻结期望集（{len(obs_l)} 条）："
                       + "; ".join(f"{f}->{t}({k})" for f, t, k, _ in obs_l))
        elif set(obs_l) <= set(exp_l):
            ledger.add("SRC-P3", "recorded",
                       f"语言仓台账少于冻结期望（C2 提前完成）: 现存 {obs_l or '无'}")
        else:
            grown = sorted(set(obs_l) - set(exp_l))
            ledger.add("SRC-P3", "fail", f"语言仓 Man-core 边超出台账（C2 范围被扩大）: {grown}")
        # 路径布线解析（P1b）：pre 基线面
        unresolved = [
            e for e in edges_l
            if e["to"] is None and _fallback_resolve(e, ctx["repos"]) is None
        ]
        ledger.add("SRC-P1b", "pass" if not unresolved else "recorded",
                   "语言仓五包闭包路径布线全解析" if not unresolved else
                   "未解析路径边（预迁移记档）: " + "; ".join(edge_label(e) for e in unresolved))

    # ---- UI 仓候选/切源闭包（window + switched）----
    if phase in ("import-window", "source-switched"):
        ban_ui = ban_pre if phase == "import-window" else ban_post
        reached_u, edges_u = closure_of(five & set(ui_members), ui_members, ui_edges)
        # P1b：候选布线必须解析（窗口/切源均判红——跨根边按文档序恢复后仍无名=布线断）
        unresolved_u = []
        for e in edges_u:
            if e["to"] is None:
                e = dict(e, to=_fallback_resolve(e, ctx["repos"]))
                if e["to"] is None:
                    unresolved_u.append(e)
        if phase == "import-window":
            ledger.add("SRC-P1b", "pass" if not unresolved_u else "fail",
                       "UI 仓候选五包闭包布线全解析（跨根按文档序）" if not unresolved_u
                       else "候选布线未解析: " + "; ".join(edge_label(e) for e in unresolved_u))
        else:
            ledger.add("SRC-P1b", "pass" if not unresolved_u else "fail",
                       "UI 仓五包闭包布线全解析" if not unresolved_u
                       else "布线未解析: " + "; ".join(edge_label(e) for e in unresolved_u))

        hits3 = sorted(edge_label(e) for e in edges_u
                       if e["to"] in ("auto-man", "auto-man-runtime", "auto-lang"))
        core_edges = [e for e in edges_u if e["to"] == "auto-man-core"]
        obs_u_core = sorted(_ledger_tuple(e) for e in core_edges)
        if phase == "import-window":
            core_ok = set(obs_u_core) <= window_ui_ledger
            detail = (f"UI 仓候选闭包 {len(reached_u)} 包零大库/monolith 禁断"
                      + (f"；Man-core 过渡边在台账内（{len(obs_u_core)} 条）" if core_ok and obs_u_core
                         else "；Man-core 过渡边已清零（提前完成）" if core_ok else ""))
            ledger.add("SRC-P2u", "pass" if (not hits3 and core_ok) else "fail",
                       detail if not hits3 and core_ok else
                       "候选闭包违例: " + "; ".join(hits3)
                       + ("；Man-core 边超出台账: " + str(sorted(set(obs_u_core) - window_ui_ledger)) if not core_ok else ""))
            ledger.add("SRC-P3u", "recorded" if set(obs_u_core) == window_ui_ledger else
                       ("recorded" if core_ok else "fail"),
                       f"UI 仓侧 Man-core 台账观察={obs_u_core or '无'}（窗口期望={sorted(window_ui_ledger) or '无'}）")
        else:
            all_hits = hits3 + [edge_label(e) for e in core_edges]
            ledger.add("SRC-P2", "pass" if not all_hits else "fail",
                       (f"闭包 {len(reached_u)} 包零禁断（{'/'.join(ban_ui)}，收紧含零 auto-man-core）"
                        if not all_hits else "禁断边: " + "; ".join(sorted(all_hits))))
        # P2 主 ID 在 window 阶段由语言仓权威面持有；候选面为 P2u，避免同名双义。

    # ---- SRC-P4：UI 仓 vendoring / 走私守卫 ----
    member_ban = forbidden["member_ban_all_phases"]
    vendored = sorted(set(ui_members) & set(member_ban))
    ledger.add(
        "SRC-P4a", "pass" if not vendored else "fail",
        "UI 仓成员零 vendoring（四名恒禁）" if not vendored else "UI 仓 vendoring: " + ",".join(vendored))
    legacy = phases["legacy_attribution_pre_migration"]
    legacy_names = set(legacy["legacy_crates"])

    def is_legacy_member(member_name: str) -> bool:
        if member_name in legacy_names:
            return True
        member_dir = ui_members.get(member_name)
        if member_name in ui_members:
            rel = member_dir.resolve().relative_to(ui.path.resolve()).as_posix()
            return any(rel.startswith(p) for p in legacy["legacy_manifest_prefixes"])
        return False

    ui_edges_named = [dict(e, to=e["to"] or _fallback_resolve(e, ctx["repos"])) for e in ui_edges]
    forbidden3_edges = [e for e in ui_edges_named
                        if e["to"] in ("auto-man", "auto-man-runtime", "auto-lang")]
    core_member_edges = [e for e in ui_edges_named if e["to"] == "auto-man-core"]
    legacy3 = sorted(edge_label(e) for e in forbidden3_edges if is_legacy_member(e["from"]))
    new3 = sorted(edge_label(e) for e in forbidden3_edges if not is_legacy_member(e["from"]))
    if phase in ("pre-migration", "import-window"):
        part = f"legacy_observed={legacy3 or '无'}"
        if phase == "pre-migration":
            if new3:
                ledger.add("SRC-P4b", "fail",
                           "T-04 走私守卫命中（非 legacy 新边指向禁断包）: " + "; ".join(new3) + f"；{part}")
            else:
                ledger.add("SRC-P4b", "recorded", f"UI 仓禁断声明边：{part}；非 legacy=无")
        else:  # import-window：Man-core 边按窗口台账判（迁移包=台账内记档；其余红）
            core_ok_edges = [e for e in core_member_edges
                             if _ledger_tuple(e) in window_ui_ledger]
            core_bad = sorted(edge_label(e) for e in core_member_edges
                              if _ledger_tuple(e) not in window_ui_ledger
                              and not is_legacy_member(e["from"]))
            core_legacy = sorted(edge_label(e) for e in core_member_edges
                                 if _ledger_tuple(e) not in window_ui_ledger
                                 and is_legacy_member(e["from"]))
            bad = new3 + core_bad
            ledger.add("SRC-P4b", "fail" if bad else "recorded",
                       (f"T-04 走私守卫命中: {'; '.join(bad)}" if bad else "UI 仓窗口期禁断边零违规")
                       + f"；{part}；Man-core 台账内={len(core_ok_edges)}"
                       + (f"；Man-core legacy/台账外记档={core_legacy}" if core_legacy else ""))
    else:
        all_hits = sorted(edge_label(e) for e in forbidden3_edges + core_member_edges)
        ledger.add("SRC-P4b", "pass" if not all_hits else "fail",
                   "UI 仓全成员零禁断边（legacy 已清理）" if not all_hits
                   else "禁断边（含 legacy 残留）: " + "; ".join(all_hits))
        remaining = sorted(set(ui_members) & legacy_names)
        ledger.add("SRC-P4c", "recorded",
                   f"legacy 成员残留（T-09 清洁面复核）: {remaining or '无'}")

    # ---- SRC-P5：autodown 跨仓边 ----
    runtime_edges = ui_edges if phase in ("import-window", "source-switched") else lang_edges
    autodown_edges = [
        e for e in runtime_edges
        if e["from"] == "auto-ui-runtime" and e["dep_key"] == "autodown-core"
    ]
    down = ctx["repos"]["down"]
    if not autodown_edges:
        ledger.add("SRC-P5", "fail" if phase == "source-switched" else "recorded",
                   "auto-ui-runtime 未声明 autodown-core optional 边（跨仓 §4 合同失位）"
                   if phase == "source-switched" else
                   "autodown-core 边缺失：预迁移记档（若 T-06 前被移除须携记录修订阶段表）")
    else:
        edge = autodown_edges[0]
        target = edge["target_dir"]
        resolvable = (target / "Cargo.toml").is_file()
        order_note = ""
        if not resolvable:
            fallback = _fallback_resolve(edge, ctx["repos"])
            if fallback:
                order_note = f"；声明目标自本树悬挂，按文档解析序于 down 仓解析到 {fallback}"
        detail = (f"{edge_label(edge)} → 目标{'可解析' if resolvable else '不可解析'}"
                  f"（down 仓在案={down.path.is_dir()}）{order_note}")
        if phase in ("pre-migration", "import-window"):
            ledger.add("SRC-P5", "recorded",
                       detail + "；固定依赖机制 T-06 冻结（contract-baseline §4）")
        else:
            ledger.add("SRC-P5", "pass" if (resolvable or order_note) else "fail",
                       detail + "；source-switched 后必须可解析（fresh/locked 可复现）")

    # ---- SRC-P6：语言仓原件删除 ----
    if phase == "source-switched":
        leftover = sorted(set(lang_members) & five)
        ledger.add("SRC-P6", "pass" if not leftover else "fail",
                   "语言仓成员零五包名（原生产目录删除）" if not leftover
                   else "语言仓残留五包成员: " + ",".join(leftover))
    elif phase == "import-window":
        ledger.add("SRC-P6", "recorded",
                   f"窗口期设计态：语言仓五包保留 {len(five & set(lang_members))}/5"
                   "（权威未切，T-06 切源后本断言转判）")
    else:
        ledger.add("SRC-P6", "recorded",
                   f"预迁移：五包预期仍在语言仓（在位 {len(five & set(lang_members))}/5）")


# ---------------------------------------------------------------------------
# type 族（V-02）
# ---------------------------------------------------------------------------

def word_contains(text: str, needle: str) -> bool:
    return re.search(r"(?<![A-Za-z0-9_])" + re.escape(needle) + r"(?![A-Za-z0-9_])", text) is not None


DEF_FORM_RE_CACHE: dict[str, re.Pattern] = {}


def defines_symbol(crate_dir: Path, symbol: str) -> bool:
    """定义形态判别：crate src 树内存在 `pub fn/struct/enum/trait/type <symbol>`
    定义（pub use 再导出不计——T-05 起旧入口为再导出 shim，恰一判别不受干扰）。"""
    if symbol not in DEF_FORM_RE_CACHE:
        DEF_FORM_RE_CACHE[symbol] = re.compile(
            r"^\s*pub\s+(?:async\s+)?(?:fn|struct|enum|trait|type|const)\s+"
            + re.escape(symbol) + r"\b", re.M,
        )
    pattern = DEF_FORM_RE_CACHE[symbol]
    src = crate_dir / "src"
    if not src.is_dir():
        return False
    for rs in src.rglob("*.rs"):
        body = read_text_or_none(rs)
        if body is not None and pattern.search(body):
            return True
    return False


def run_type(ctx: dict, phases: dict, ledger: Ledger) -> None:
    lang = ctx["repos"]["lang"]
    phase = ctx["phase"]
    if not lang.present:
        raise UsageError(f"语言仓不可解析: {lang.path}")
    text = read_text_or_none(TOUCHPOINTS_PATH)
    if text is None:
        raise UsageError(f"touchpoint 表不可读: {TOUCHPOINTS_PATH.name}")
    tp = tomllib.loads(text)
    compat_crate = "crates/auto-lang"

    for face in tp.get("compat_face", []):
        anchor = face["anchor"]
        file = lang.path / compat_crate / anchor["file"]
        body = read_text_or_none(file)
        ok = body is not None and word_contains(body, anchor["contains"])
        ledger.add(face["id"], "pass" if ok else "fail",
                   f"{face['item']} 锚点在位（{compat_crate}/{anchor['file']}）"
                   if ok else
                   f"{face['item']} 锚点缺失（{compat_crate}/{anchor['file']} 不含 {anchor['contains']!r}）")

    for home_entry in tp.get("declaration_home", []):
        symbol = home_entry["symbol"]
        present_in = [home for home in home_entry["homes"]
                      if defines_symbol(lang.path / home, symbol)]
        status = "pass" if len(present_in) == 1 else "fail"
        detail = (f"{symbol} 定义单源@{present_in[0]}（再导出不计）" if len(present_in) == 1
                  else f"{symbol} 定义单源被破坏：定义在场={present_in or '无'}（期望恰一，"
                       f"候选={home_entry['homes']}；双在场=复制，缺席=失位）")
        ledger.add(home_entry["id"], status, detail)

    for face in tp.get("import_face", []):
        body = read_text_or_none(lang.path / face["consumer_file"])
        pre_token = face["expected_pre_migration"]
        post_token = face["expected_source_switched"]
        if body is None:
            ledger.add(face["id"], "fail", f"消费面文件不可读: {face['consumer_file']}")
            continue
        if phase == "source-switched":
            ok = word_contains(body, post_token)
            ledger.add(face["id"], "pass" if ok else "fail",
                       f"{face['consumer_file']} 声明源={post_token}（C2 切移完成）" if ok
                       else f"{face['consumer_file']} 仍引用 {pre_token}（C2 未完成）")
        else:
            has_pre = word_contains(body, pre_token)
            ledger.add(face["id"], "recorded",
                       f"{face['consumer_file']} 声明源当前={'auto_man_core' if has_pre else '已切/其他'}"
                       f"（T-05 切移窗口内记档，切源后须为 {post_token}）")

    # ---- TYP-F1：feature 名单清点（default/显式 Core/UI 组合的钉住）----
    manifest = read_text_or_none(lang.path / compat_crate / "Cargo.toml")
    if manifest is None:
        ledger.add("TYP-F1", "fail", f"{compat_crate}/Cargo.toml 不可读")
    else:
        try:
            data = tomllib.loads(manifest)
            features = sorted((data.get("features") or {}).keys())
            has_default = "default" in features
            ledger.add("TYP-F1", "pass" if has_default else "fail",
                       f"feature 名单钉住（{len(features)} 项，default={'在' if has_default else '缺'}）："
                       + ",".join(features))
            ctx.setdefault("inventories", {})["compat_features"] = features
        except tomllib.TOMLDecodeError as exc:
            ledger.add("TYP-F1", "fail", f"feature 清点 TOML 解析失败: {exc}")

    # ---- 编译探针（F831-09-02）：--compile-probe 时真实执行，否则接口冻结记 pending ----
    probe_items = [face["probe"] for face in tp.get("compat_face", [])]
    ctx.setdefault("inventories", {})["compile_probe_uses"] = sorted(set(probe_items))
    if not ctx.get("compile_probe"):
        ledger.add("TYP-PROBE", "pending",
                   f"编译探针接口冻结（{len(set(probe_items))} 条 use 路径，由 touchpoint 表派生）；"
                   "默认零 cargo——实编（三 feature 组合 + 双 source 负控）须 --compile-probe")
        return

    # 实编：compile-probe/ 三 feature 组合（A=default 全量含运行断言；
    # B=core-only；C=core+显式 UI）+ compile-probe-dual/ 同名包双路径负控。
    # 组合语义与 feature 条件性分面（iced/interpreter 门子集）见探针 Cargo.toml。
    probe_dir = HERE / "compile-probe"
    combos = [
        ("A:default+run", ["cargo", "run", "--quiet"]),
        ("B:core-only", ["cargo", "check", "--quiet", "--no-default-features",
                         "--features", "lang-core"]),
        ("C:core+explicit-ui", ["cargo", "check", "--quiet", "--no-default-features",
                                "--features", "lang-core,ui-face"]),
    ]
    combo_results: list[str] = []
    combo_tails: list[str] = []
    combos_ok = True
    for combo_name, cmd in combos:
        proc = subprocess.run(cmd, cwd=str(probe_dir), capture_output=True,
                              text=True, encoding="utf-8", errors="replace")
        ok = proc.returncode == 0
        combos_ok = combos_ok and ok
        combo_results.append(f"{combo_name}={'pass' if ok else 'RED(exit ' + str(proc.returncode) + ')'}")
        if not ok:
            tail = [ln for ln in (proc.stderr or "").splitlines() if ln.strip()][-2:]
            combo_tails.append(f"{combo_name}: " + " / ".join(tail)[:200])
    dual_dir = HERE / "compile-probe-dual"
    dual = subprocess.run(
        ["cargo", "metadata", "--format-version", "1"], cwd=str(dual_dir),
        capture_output=True, text=True, encoding="utf-8", errors="replace")
    dual_rejected = dual.returncode != 0 and "auto-ui-dialect" in (dual.stderr or "")
    dual_lock = dual_dir / "Cargo.lock"
    if dual_lock.exists():
        dual_lock.unlink()  # 负控判定面=解析错误文本；半写锁可再生，不入收据
    ctx.setdefault("inventories", {})["compile_probe_runs"] = combo_results + [
        f"dual-source={'rejected' if dual_rejected else 'NOT-REJECTED'}"]
    ok = combos_ok and dual_rejected
    detail = (f"编译探针实编 {len(set(probe_items))} 条 use 路径（构造/pub 字段/回调/fn-item/"
              f"类型传递与单源身份运行断言）：{'；'.join(combo_results)}；"
              f"双source负控={'同名包双路径被解析层拒绝' if dual_rejected else '未拒绝（RED）'}")
    if combo_tails:
        detail += "；失败尾巴: " + " | ".join(combo_tails)
    ledger.add("TYP-PROBE", "pass" if ok else "fail", detail)


# ---------------------------------------------------------------------------
# test-id 族（V-06）
# ---------------------------------------------------------------------------

def enumerate_package_tests(package_dir: Path) -> dict:
    test_ids: set[str] = set()
    files = 0
    for rs in sorted(package_dir.rglob("*.rs")):
        if "target" in rs.parts:
            continue
        body = read_text_or_none(rs)
        if body is None:
            continue
        files += 1
        expect_fn = False
        for line in body.splitlines():
            if expect_fn:
                m = FN_NAME_RE.match(line)
                if m:
                    test_ids.add(m.group(1))
                    expect_fn = False
                    continue
                if line.strip() and not line.strip().startswith("#[") and not line.strip().startswith("//"):
                    expect_fn = False
            if TEST_ATTR_RE.match(line):
                expect_fn = True
    return {"test_files": files, "test_ids": sorted(test_ids), "count": len(test_ids)}


def run_test_id(ctx: dict, phases: dict, ledger: Ledger) -> None:
    lang, ui = ctx["repos"]["lang"], ctx["repos"]["ui"]
    phase = ctx["phase"]
    migrating = phases["migrating_packages"]
    lang_members = discover_members(lang.path)
    ui_members = discover_members(ui.path) if ui.present else {}
    home = lang_members if phase == "pre-migration" else ui_members

    inventory: dict[str, dict] = {}
    missing = []
    for pkg in migrating:
        member = home.get(pkg)
        if member is None:
            missing.append(pkg)
            continue
        inventory[pkg] = enumerate_package_tests(member)
    ctx.setdefault("inventories", {})["test_id_inventory"] = inventory
    ledger.add(
        "TID-A1",
        "fail" if missing else "pass",
        (f"五包测试 ID 清点（source-enumeration/1）："
         + "; ".join(f"{p}={inventory[p]['count']}" for p in migrating if p in inventory)
         + "；盲区见收据 limitations" if not missing else "清点缺失: " + ",".join(missing)),
    )

    # ---- TID-A2：cargo 档位别名表快照 ----
    config = read_text_or_none(lang.path / ".cargo" / "config.toml")
    if config is None:
        ledger.add("TID-A2", "fail", ".cargo/config.toml 不可读（档位别名表缺位）")
    else:
        try:
            aliases = (tomllib.loads(config).get("alias") or {})
            captured = {k: aliases[k] for k in TIER_ALIASES if k in aliases}
            missing_tiers = [k for k in TIER_ALIASES if k not in captured]
            ctx.setdefault("inventories", {})["tier_aliases"] = captured
            ledger.add("TID-A2", "pass" if not missing_tiers else "fail",
                       f"档位别名表快照 {len(captured)}/{len(TIER_ALIASES)}"
                       + ("" if not missing_tiers else "，缺 " + ",".join(missing_tiers)))
        except tomllib.TOMLDecodeError as exc:
            ledger.add("TID-A2", "fail", f"别名表解析失败: {exc}")

    # ---- TID-A3：UI 仓逐包 cargo test 入口（post）----
    if phase == "source-switched":
        entries = sorted(
            p for p in migrating
            if (ui.path / "crates" / p / "Cargo.toml").is_file()
        )
        ledger.add("TID-A3", "pass" if len(entries) == len(migrating) else "fail",
                   f"UI 仓 crates/<pkg>/Cargo.toml 定向 cargo test 入口 {len(entries)}/{len(migrating)}")
    else:
        ledger.add("TID-A3", "recorded",
                   "预迁移：UI 仓定向入口预期缺位（T-06 后按 cargo test --manifest-path 重采正典清单）")


# ---------------------------------------------------------------------------
# distribution / failure 族（接口冻结，phase-gated）
# ---------------------------------------------------------------------------

DISTRIBUTION_SHAPES = [
    ("DST-1", "fresh-lock", "独立 clone 内 cargo generate-lockfile 全新解析（无开发根 env 回退）"),
    ("DST-2", "locked-build", "clone 内 cargo build --locked 真实构建"),
    ("DST-3", "relocate-a-to-b", "交付包 A→B 重定位（不含 target/）后 locked 重放 + path 依赖全在 B 内"),
    ("DST-4", "external-sdk-consumption", "UI checkout 缺席下纯 SDK 消费（反向控制，指针 language-sdk-consumer）"),
    ("DST-5", "tool-assets-profile", "随包 tool 二进制 + 运行资产 profile（framed I-TOOL v1 外部消费，792 V05 同构）"),
]

FAILURE_SHAPES = [
    ("FLR-1", "missing-source-producer-drift", "producer 破损/漂移显式 Err 且旧交付字节不动（792 V03e 同构）"),
    ("FLR-2", "bad-pin", "不兼容 pin 与冻结锁冲突确定性红（792 V03c）"),
    ("FLR-3", "missing-patch", "消费根漏 patch --locked 红（792 V03b）"),
    ("FLR-4", "resource-four-states", "等长篡改/坏 hash/缺资源/未登记资源四态判红（792 V03a）"),
    ("FLR-5", "cancel-swap-failure", "取消/中断/三仓交换失败：按受管事务保旧字节、可恢复点、幂等重试（T-08）"),
    ("FLR-6", "foreign-dir-no-delete", "陌生目录/外来同级字节不删不覆盖（792 V03d/h）"),
]

# 收据绑定（T-07 起）：发行/失败族从执行载体收据读取实际结果——
# scripts/ui-repository-distribution.py（DST-1..4）与 792 generated-ui-consumer
# driver 收据（DST-5 + FLR-1..4,6；其 V01..V05 结果映射见 792_RECEIPT_MAP）。
DISTRIBUTION_RECEIPT_REL = "../../../../../scratch-ui-repo-distribution/distribution-receipt.json"
D792_RECEIPT_REL = "../../../../../scratch-consumer/driver-receipt.json"
SWITCH_DRILL_RECEIPT_REL = "../../../../../scratch-ui-repo-fixture/switch-drill-receipt.json"
D792_RECEIPT_MAP = {
    "DST-5": ["V05a-tool-fresh-locked-build", "V05b-tool-relocated-locked-build",
              "V05c-tool-describe", "V05e-tool-generate-real"],
    "FLR-1": ["V03e-redelivery-failure-preserves-old", "V03e-redelivery-after-restore"],
    "FLR-2": ["V03c-incompatible-pin-red"],
    "FLR-3": ["V03b-missing-patch-red"],
    "FLR-4": ["V03a-eqlen-tamper-red", "V03a-corrupt-hash-red",
              "V03a-missing-resource-red", "V03a-unlisted-resource-red"],
    "FLR-6": ["V03d-foreign-dir-rejected", "V03h-foreign-staging-sibling-preserved",
              "V03h-foreign-fixed-backup-sibling-preserved"],
}


def _load_bound_receipt(rel: str) -> dict | None:
    path = (HERE / rel).resolve()
    if not path.is_file():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def run_distribution(ctx: dict, phases: dict, ledger: Ledger) -> None:
    phase = ctx["phase"]
    bound = _load_bound_receipt(DISTRIBUTION_RECEIPT_REL) if phase == "source-switched" else None
    bound792 = _load_bound_receipt(D792_RECEIPT_REL) if phase == "source-switched" else None
    for shape_id, name, oracle in DISTRIBUTION_SHAPES:
        if bound is not None:
            step = next((s for s in bound.get("steps", []) if s["step"].startswith(shape_id)), None)
            if step is not None:
                ledger.add(shape_id, "pass" if step["ok"] else "fail",
                           f"[{name}] 收据绑定：{step['detail']}（{step['elapsed_s']}s，"
                           f"源 {bound.get('sources', {}).get('ui', '?')[:9]}）")
                continue
        if shape_id == "DST-5" and bound792 is not None:
            results = {r["case"]: r["ok"] for r in bound792.get("results", [])}
            mapped = [(c, results.get(c)) for c in D792_RECEIPT_MAP[shape_id]]
            all_ok = mapped and all(ok for _, ok in mapped)
            ledger.add(shape_id, "pass" if all_ok else "fail",
                       f"[{name}] 792 收据绑定（V05 工具+资产随包真实构建/外部消费）：{mapped}")
            continue
        ledger.add(shape_id, "pending",
                   f"[{name}] {oracle}——接口冻结；执行 phase=source-switched（T-07，"
                   f"当前={phase}）；现行同构控制 generated-ui-consumer/driver.py（815-A）")


def run_failure(ctx: dict, phases: dict, ledger: Ledger) -> None:
    phase = ctx["phase"]
    bound792 = _load_bound_receipt(D792_RECEIPT_REL) if phase == "source-switched" else None
    drill = _load_bound_receipt(SWITCH_DRILL_RECEIPT_REL) if phase == "source-switched" else None
    for shape_id, name, oracle in FAILURE_SHAPES:
        if shape_id == "FLR-5" and drill is not None:
            ok_checks = [c["ok"] for c in drill.get("checks", [])]
            all_ok = bool(ok_checks) and all(ok_checks)
            ledger.add(shape_id, "pass" if all_ok else "fail",
                       f"[{name}] 切换事务 drill 收据绑定（{len(ok_checks)} 检：正常链/锚冲突/"
                       "中断幂等/交换失败保旧/取消零副作用/恢复重试，工具 scripts/ui-repository-switch.py）")
            continue
        if bound792 is not None and shape_id in D792_RECEIPT_MAP:
            results = {r["case"]: r["ok"] for r in bound792.get("results", [])}
            mapped = [(c, results.get(c)) for c in D792_RECEIPT_MAP[shape_id]]
            all_ok = mapped and all(ok for _, ok in mapped)
            ledger.add(shape_id, "pass" if all_ok else "fail",
                       f"[{name}] 792 收据绑定（负控确定性红+恢复复绿）：{mapped}")
            continue
        ledger.add(shape_id, "pending",
                   f"[{name}] {oracle}——接口冻结；执行 T-07/T-08（当前 phase={phase}）；"
                   "现行同构控制 792 driver V03 + scripts/legacy-preservation.py recover-drill")


RUNNERS = {
    "source": run_source,
    "type": run_type,
    "test-id": run_test_id,
    "distribution": run_distribution,
    "failure": run_failure,
}


# ---------------------------------------------------------------------------
# selftest：冻结断言的区分性 oracle（受控突变 + 合法对照；合成仓在 scratch）
# ---------------------------------------------------------------------------

def _make_pkg(pkg_dir: Path, name: str, deps: dict[str, str] | None = None,
              optional: set[str] | None = None) -> None:
    optional = optional or set()
    pkg_dir.mkdir(parents=True, exist_ok=True)
    lines = ["[package]", f'name = "{name}"', 'version = "0.0.0"',
             'edition = "2021"', "", "[dependencies]"]
    for key, path in (deps or {}).items():
        opt = ", optional = true" if key in optional else ""
        lines.append(f'{key} = {{ path = "{path}"{opt} }}')
    (pkg_dir / "Cargo.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _make_workspace(root: Path, members: list[str]) -> None:
    root.mkdir(parents=True, exist_ok=True)
    body = ["[workspace]", "members = [" + ", ".join(f'"{m}"' for m in members) + "]"]
    (root / "Cargo.toml").write_text("\n".join(body) + "\n", encoding="utf-8")


def _run_family(family: str, repos: dict[str, Repo], phase: str) -> Ledger:
    phases = json.loads(PHASES_PATH.read_text(encoding="utf-8"))

    def identity(text: str) -> str:
        return text

    ledger = Ledger(identity)
    ctx = {"repos": repos, "phase": phase, "inventories": {}}
    RUNNERS[family](ctx, phases, ledger)
    return ledger


def _expect(cond: bool, label: str, problems: list[str]) -> None:
    log(f"{'PASS' if cond else 'FAIL'} [selftest] {label}")
    if not cond:
        problems.append(label)


def _ledger_map(ledger: Ledger) -> dict[str, dict]:
    return {item["id"]: item for item in ledger.items}


def cmd_selftest() -> int:
    import shutil
    import tempfile

    base = Path(tempfile.mkdtemp(prefix="ui-repo-fixture-selftest-"))
    problems: list[str] = []
    # 合成 pre-migration 语言仓（五包成员 + 仅内部互引——合法对照基线；
    # T-06 切源后真语言仓已无五包，不再可作 S1/S4 对照）。
    synth_lang = base / "synth-lang"
    _make_workspace(synth_lang, ["crates/*"])
    for pkg in ("auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                "auto-ui-tools", "shell-pack"):
        deps = {"auto-ui-dialect": None,
                "auto-ui-compiler": {"auto-ui-dialect": "../auto-ui-dialect"},
                "auto-ui-runtime": {"auto-ui-compiler": "../auto-ui-compiler"},
                "auto-ui-tools": {"auto-ui-runtime": "../auto-ui-runtime"},
                "shell-pack": {"auto-ui-runtime": "../auto-ui-runtime"}}[pkg]
        _make_pkg(synth_lang / "crates" / pkg, pkg, deps)
    pre_lang = Repo("lang", synth_lang)
    real_lang = Repo("lang", HERE.parents[3])
    try:
        # ---- S1：T-04 走私守卫（真语言仓 + 合成 UI 仓受控突变）----
        stubs = base / "stubs"
        for stub in ("auto-lang", "auto-man", "auto-man-core"):
            (stubs / stub).mkdir(parents=True)
            (stubs / stub / "Cargo.toml").write_text(
                f'[package]\nname = "{stub}"\nversion = "0.0.0"\n', encoding="utf-8")
        s1_ui = base / "s1-ui"
        _make_workspace(s1_ui, ["crates/*", "examples/*"])
        # legacy 名成员（optional lang-monolith 边）→ legacy_observed 记档
        _make_pkg(s1_ui / "crates" / "legacy-a", "auto-ui",
                  {"auto-lang": "../../../stubs/auto-lang"}, optional={"auto-lang"})
        # examples/ 下成员（禁断边）→ legacy 归属记档
        _make_pkg(s1_ui / "examples" / "legacy-app", "legacy-app",
                  {"auto-lang": "../../../stubs/auto-lang"})
        # 非 legacy 新成员 → 禁断边必须红
        _make_pkg(s1_ui / "crates" / "new-import", "new-import",
                  {"auto-man": "../../../stubs/auto-man"})
        # vendoring：UI 仓成员自称 auto-man-core → P4a 红
        _make_pkg(s1_ui / "crates" / "vendored", "auto-man-core")
        repos_s1 = {"lang": pre_lang, "ui": Repo("ui", s1_ui),
                    "down": Repo("down", base / "absent-down")}
        ledger = _run_family("source", repos_s1, "pre-migration")
        got = _ledger_map(ledger)
        _expect(got["SRC-P4a"]["status"] == "fail", "S1 vendoring 成员判红（SRC-P4a）", problems)
        _expect(got["SRC-P4b"]["status"] == "fail"
                and "new-import" in got["SRC-P4b"]["detail"],
                "S1 非 legacy 新边判红并点名 new-import（SRC-P4b）", problems)
        _expect("legacy_observed" in got["SRC-P4b"]["detail"]
                and got["SRC-P4b"]["detail"].count("auto-lang") >= 1,
                "S1 legacy 归属边记档不误红（SRC-P4b detail）", problems)
        _expect(got["SRC-P1"]["status"] == "pass" and got["SRC-P2"]["status"] == "pass",
                "S1 合法对照：真语言仓五包在位 + 闭包零禁断（SRC-P1/P2）", problems)

        # ---- S2：T-06 收紧断言（合成切源仓 + Man-core 闭包边必须红）----
        s2 = base / "s2"
        s2_lang = s2 / "auto-lang"
        _make_workspace(s2_lang, [])
        s2_down = s2 / "auto-down" / "autodown" / "packages" / "engine" / "rust"
        _make_pkg(s2_down, "autodown-core")
        s2_ui = s2 / "auto-ui"
        _make_workspace(s2_ui, ["crates/*"])
        for pkg in ("auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                    "auto-ui-tools", "shell-pack"):
            deps = {"autodown-core": "../../../auto-down/autodown/packages/engine/rust"} \
                if pkg == "auto-ui-runtime" else None
            optional = {"autodown-core"} if pkg == "auto-ui-runtime" else set()
            # auto-ui-tools 带跨根 Man-core 边（目标在仓外 stub，非成员 → 不触发 P4a）
            if pkg == "auto-ui-tools":
                deps = {"auto-man-core": "../../../../stubs2/auto-man-core"}
            _make_pkg(s2_ui / "crates" / pkg, pkg, deps, optional)
        stubs2 = base / "stubs2" / "auto-man-core"
        _make_pkg(stubs2, "auto-man-core")
        repos_s2 = {"lang": Repo("lang", s2_lang), "ui": Repo("ui", s2_ui),
                    "down": Repo("down", s2 / "auto-down")}
        ledger = _run_family("source", repos_s2, "source-switched")
        got = _ledger_map(ledger)
        _expect(got["SRC-P2"]["status"] == "fail"
                and "auto-man-core" in got["SRC-P2"]["detail"],
                "S2 收紧断言：切源后闭包 Man-core 边判红并点名（SRC-P2）", problems)
        _expect(got["SRC-P4a"]["status"] == "pass",
                "S2 跨根边目标非成员不误触 vendoring（SRC-P4a）", problems)
        _expect(got["SRC-P4b"]["status"] == "fail",
                "S2 切源后 UI 仓禁断边一律红（SRC-P4b）", problems)
        _expect(got["SRC-P5"]["status"] == "pass",
                "S2 autodown 边可解析判绿（SRC-P5）", problems)
        _expect(got["SRC-P6"]["status"] == "pass",
                "S2 语言仓零五包名（原件删除判绿对照，SRC-P6）", problems)

        # ---- S3：部分导入中间态拒绝 ----
        s3_ui = base / "s3-ui"
        _make_workspace(s3_ui, ["crates/*"])
        _make_pkg(s3_ui / "crates" / "auto-ui-dialect", "auto-ui-dialect")
        try:
            detect_phase(real_lang, Repo("ui", s3_ui),
                         json.loads(PHASES_PATH.read_text(encoding="utf-8"))["migrating_packages"])
            _expect(False, "S3 部分导入中间态被拒绝（detect_phase）", problems)
        except UsageError:
            _expect(True, "S3 部分导入中间态被拒绝（detect_phase）", problems)

        # ---- S4：import-window（T-04 候选导入态；真语言仓 + 合成已导入 UI 仓）----
        s4 = base / "s4"
        s4_ui = s4 / "auto-ui"
        _make_workspace(s4_ui, ["crates/*"])
        for pkg in ("auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                    "auto-ui-tools", "shell-pack"):
            # 候选布线：语言基础走 ../../../auto-lang/crates/<pkg>（组内兄弟布局）
            deps = {
                "auto-ui-dialect": {"auto-parser-api": "../../../auto-lang/crates/auto-parser-api"},
                "auto-ui-compiler": {"auto-lang-core": "../../../auto-lang/crates/auto-lang-core",
                                     "auto-ui-dialect": "../auto-ui-dialect"},
                "auto-ui-runtime": {"auto-lang-core": "../../../auto-lang/crates/auto-lang-core",
                                    "auto-ui-compiler": "../auto-ui-compiler"},
                # 过渡 Man-core 边按冻结台账保留（cross-root 指真语言仓）
                "auto-ui-tools": {"auto-man-core": "../../../auto-lang/crates/auto-man-core",
                                  "auto-ui-runtime": "../auto-ui-runtime"},
                "shell-pack": {"auto-ui-runtime": "../auto-ui-runtime"},
            }[pkg]
            _make_pkg(s4_ui / "crates" / pkg, pkg, deps)
        # 窗口态配对语言仓（同组兄弟：五包权威在位 + 跨根布线目标 crate stub）
        s4_lang = s4 / "auto-lang"
        _make_workspace(s4_lang, ["crates/*"])
        for pkg in ("auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                    "auto-ui-tools", "shell-pack",
                    "auto-parser-api", "auto-lang-core", "auto-man-core"):
            _make_pkg(s4_lang / "crates" / pkg, pkg)
        s4_lang_repo = Repo("lang", s4_lang)
        phase_s4 = detect_phase(s4_lang_repo, Repo("ui", s4_ui),
                                json.loads(PHASES_PATH.read_text(encoding="utf-8"))["migrating_packages"])
        _expect(phase_s4 == "import-window", "S4 判别=import-window（lang+ui 双侧五包）", problems)
        ledger = _run_family("source",
                             {"lang": s4_lang_repo, "ui": Repo("ui", s4_ui),
                              "down": Repo("down", base / "absent-down")},
                             "import-window")
        got = _ledger_map(ledger)
        _expect(got["SRC-P1"]["status"] == "pass" and got["SRC-P1b"]["status"] == "pass",
                "S4 候选导入完备 + 布线全解析（SRC-P1/P1b）", problems)
        _expect(got["SRC-P2"]["status"] == "pass" and got["SRC-P3"]["status"] == "recorded",
                "S4 语言仓权威面仍按 pre 口径断言（SRC-P2/P3）", problems)
        _expect(got["SRC-P2u"]["status"] == "pass"
                and got["SRC-P3u"]["status"] == "recorded",
                "S4 候选闭包零大库/monolith + Man-core 台账内记档（SRC-P2u/P3u）", problems)
        _expect(got["SRC-P6"]["status"] == "recorded",
                "S4 语言仓原件未删=窗口设计态记档（SRC-P6）", problems)
        # 窗口走私反例：候选包外新增 Man-core 边必须红
        _make_pkg(s4_ui / "crates" / "smuggler", "smuggler",
                  {"auto-man-core": "../../../auto-lang/crates/auto-man-core"})
        ledger = _run_family("source",
                             {"lang": s4_lang_repo, "ui": Repo("ui", s4_ui),
                              "down": Repo("down", base / "absent-down")},
                             "import-window")
        got = _ledger_map(ledger)
        _expect(got["SRC-P4b"]["status"] == "fail" and "smuggler" in got["SRC-P4b"]["detail"],
                "S4 窗口走私反例：台账外 Man-core 边判红并点名（SRC-P4b）", problems)
    finally:
        shutil.rmtree(base, ignore_errors=True)

    if problems:
        log(f"selftest 判红 {len(problems)} 项：{problems}")
        return EXIT_ASSERT_FAIL
    log("selftest 全部通过（冻结断言的区分性 oracle 成立）")
    return EXIT_OK


# ---------------------------------------------------------------------------
# 阶段检测与主流程
# ---------------------------------------------------------------------------

def detect_phase(lang_repo: Repo, ui_repo: Repo, migrating: list[str]) -> str:
    lang_members = discover_members(lang_repo.path) if lang_repo.present else {}
    ui_members = discover_members(ui_repo.path) if ui_repo.present else {}
    five = set(migrating)
    in_lang, in_ui = five <= set(lang_members), five <= set(ui_members)
    any_lang, any_ui = bool(five & set(lang_members)), bool(five & set(ui_members))
    if in_lang and not any_ui:
        return "pre-migration"
    if in_lang and in_ui:
        return "import-window"
    if in_ui and not any_lang:
        return "source-switched"
    raise UsageError(
        f"五包在场状态与全部阶段判别均不匹配：lang={sorted(five & set(lang_members))}，"
        f"ui={sorted(five & set(ui_members))}（部分导入/部分删除=中间态，禁止作为门禁基线；"
        "完成对应任务或修正 --lang-repo/--ui-repo）"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("family", choices=[*FAMILIES, "all", "selftest"])
    here_root = HERE.parents[3]
    parser.add_argument("--lang-repo", default=str(here_root))
    parser.add_argument("--ui-repo", default=None,
                        help="缺省=组内兄弟 ../auto-ui，再缺省 D:/autostack/auto-ui")
    parser.add_argument("--down-repo", default=None,
                        help="缺省=语言仓 ../../../auto-down，再缺省 D:/autostack/auto-down")
    parser.add_argument("--phase", default="auto",
                        choices=["auto", "pre-migration", "import-window",
                                 "source-switched"])
    parser.add_argument("--output", default=None)
    parser.add_argument("--compile-probe", action="store_true",
                        help="编译探针实编（F831-09-02 起启用：三 feature 组合 + 双 source 负控；默认零 cargo 记 pending）")
    args = parser.parse_args()

    if args.family == "selftest":
        return cmd_selftest()

    lang_path = Path(args.lang_repo).resolve()

    def sibling_default(name: str) -> Path:
        # 解析序（AGENTS）：组内兄弟（worktree 组或主检出同级）→ D:/autostack 主检出。
        sibling = lang_path.parent / name
        return sibling if sibling.is_dir() else Path("D:/autostack") / name

    ui_path = Path(args.ui_repo).resolve() if args.ui_repo else sibling_default("auto-ui")
    down_path = Path(args.down_repo).resolve() if args.down_repo else sibling_default("auto-down")

    phases = json.loads(PHASES_PATH.read_text(encoding="utf-8"))
    repos = {
        "lang": Repo("lang", lang_path),
        "ui": Repo("ui", ui_path),
        "down": Repo("down", down_path),
    }
    detected = detect_phase(repos["lang"], repos["ui"], phases["migrating_packages"])
    phase = detected if args.phase == "auto" else args.phase
    if args.phase != "auto" and args.phase != detected:
        print(f"[ui-repo-fixture] 配置错：声明 phase={args.phase} 与结构判别={detected} 不符",
              file=sys.stderr)
        return EXIT_USAGE

    placeholders = {
        "lang": "<lang-root>", "ui": "<ui-root>", "down": "<down-root>",
    }

    def sanitize(text: str) -> str:
        for role, ph in placeholders.items():
            text = text.replace(str(repos[role].path), ph)
            text = text.replace(str(repos[role].path.resolve()), ph)
        return text

    families = list(FAMILIES) if args.family == "all" else [args.family]
    receipts: list[dict] = []
    exit_code = EXIT_OK
    for family in families:
        ledger = Ledger(sanitize)
        ctx = {"repos": repos, "phase": phase, "compile_probe": args.compile_probe,
               "inventories": {}}
        try:
            RUNNERS[family](ctx, phases, ledger)
        except UsageError as exc:
            print(f"[ui-repo-fixture] 配置错（{family}）: {exc}", file=sys.stderr)
            return EXIT_USAGE
        receipt = {
            "schema": RECEIPT_SCHEMA,
            "tool": "test/fixtures/modularity/ui-repository-consumer/driver.py",
            "plan": "PLAN-831",
            "family": family,
            "phase": {"declared": args.phase, "resolved": phase},
            "repos": {role: repos[role].receipt_view(placeholders[role]) for role in repos},
            "assertions": ledger.items,
            "totals": ledger.totals(),
            "inventories": ctx.get("inventories", {}),
        }
        receipts.append(receipt)
        if ledger.fail_any():
            exit_code = EXIT_ASSERT_FAIL

    limitations = list(TEST_ID_LIMITATIONS) if any(r["family"] == "test-id" for r in receipts) else []
    for receipt in receipts:
        if receipt["family"] == "test-id":
            receipt["limitations"] = limitations

    payload = receipts if args.family == "all" else receipts[0]
    text = json.dumps(payload, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    if args.output:
        Path(args.output).write_text(text, encoding="utf-8", newline="\n")
        log(f"收据 → {args.output}")
    else:
        print(text, end="")

    totals = {"assertions": 0, "pass": 0, "fail": 0, "recorded": 0, "pending": 0}
    for receipt in receipts:
        for key in totals:
            totals[key] += receipt["totals"].get(key, 0)
    if exit_code == EXIT_OK:
        log("==== "
            + "/".join(f"{k}={totals[k]}" for k in ("pass", "fail", "recorded", "pending"))
            + "；recorded/pending 不判红，全通过")
    else:
        log(f"==== 判红断言 {totals['fail']} 条（pass={totals['pass']}）")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
