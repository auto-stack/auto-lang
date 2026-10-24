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
                编译探针（--compile-probe）接口冻结，T-05 起真实执行。
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

def run_source(ctx: dict, phases: dict, ledger: Ledger) -> None:
    lang, ui = ctx["repos"]["lang"], ctx["repos"]["ui"]
    phase = ctx["phase"]
    migrating = phases["migrating_packages"]
    forbidden = phases["forbidden"]

    if not lang.present:
        raise UsageError(f"语言仓不可解析（缺 Cargo.toml）: {lang.path}")
    lang_edges, lang_members = extract_declared_edges(lang.path)
    ui_edges, ui_members = (extract_declared_edges(ui.path) if ui.present else ([], {}))

    # ---- SRC-P1：五包存在于阶段所属仓 ----
    home = lang_members if phase == "pre-migration" else ui_members
    home_name = "lang_repo" if phase == "pre-migration" else "ui_repo"
    missing = [p for p in migrating if p not in home]
    ledger.add(
        "SRC-P1",
        "pass" if not missing else "fail",
        f"五包@{home_name}: {'全部在位' if not missing else '缺失 ' + ','.join(missing)}"
        f"（lang 成员 {len(lang_members)}，ui 成员 {len(ui_members)}）",
    )

    # ---- SRC-P2 + SRC-P3：五包闭包禁断 + Man-core 台账 ----
    roots = set(migrating) & set(home)
    reached, reached_edges = closure_of(roots, home, lang_edges if phase == "pre-migration" else ui_edges)
    ban = forbidden["closure_ban_pre_migration" if phase == "pre-migration"
                    else "closure_ban_source_switched"]
    hits = sorted(
        edge_label(e) for e in reached_edges
        if e["to"] in ban
    )
    ledger.add(
        "SRC-P2",
        "pass" if not hits else "fail",
        (f"闭包 {len(reached)} 包零禁断（{'/'.join(ban)}）"
         if not hits else "禁断边: " + "; ".join(hits)),
    )

    if phase == "pre-migration":
        frozen_expected = phases["transitional_man_core_edge_ledger"]["frozen_expected_pre_migration"]
        observed = sorted(
            (e["from"], e["to"], e["kind"], e["optional"])
            for e in reached_edges if e["to"] == "auto-man-core"
        )
        expected = sorted(
            (x["from"], x["to"], x["kind"], x["optional"]) for x in frozen_expected
        )
        if observed == expected:
            ledger.add("SRC-P3", "recorded",
                       f"过渡 Man-core 边台账=冻结期望集（{len(observed)} 条）："
                       + "; ".join(f"{f}->{t}({k})" for f, t, k, _ in observed))
        elif set(observed) <= set(expected):
            ledger.add("SRC-P3", "recorded",
                       f"少于冻结期望（C2 提前完成切除）：现存 {observed or '无'}")
        else:
            grown = sorted(set(observed) - set(expected))
            ledger.add("SRC-P3", "fail",
                       "Man-core 边超出台账（C2 范围被扩大）: " + str(grown))

    # ---- SRC-P4：UI 仓 vendoring / 走私守卫 ----
    member_ban = forbidden["member_ban_all_phases"]
    vendored = sorted(set(ui_members) & set(member_ban))
    ledger.add(
        "SRC-P4a",
        "pass" if not vendored else "fail",
        "UI 仓成员零 vendoring（四名恒禁）" if not vendored else "UI 仓 vendoring: " + ",".join(vendored),
    )
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

    ui_forbidden_edges = [e for e in ui_edges if e["to"] in member_ban]
    legacy_hits = sorted(
        edge_label(e) for e in ui_forbidden_edges if is_legacy_member(e["from"])
    )
    new_hits = sorted(
        edge_label(e) for e in ui_forbidden_edges if not is_legacy_member(e["from"])
    )
    if phase == "pre-migration":
        ledger_part = f"legacy_observed={legacy_hits or '无'}"
        if new_hits:
            ledger.add("SRC-P4b", "fail",
                       "T-04 走私守卫命中（非 legacy 新边指向禁断包）: "
                       + "; ".join(new_hits) + f"；{ledger_part}")
        else:
            ledger.add("SRC-P4b", "recorded", f"UI 仓禁断声明边：{ledger_part}；非 legacy=无")
    else:
        all_hits = sorted(edge_label(e) for e in ui_forbidden_edges)
        ledger.add("SRC-P4b", "pass" if not all_hits else "fail",
                   "UI 仓全成员零禁断边（legacy 已清理）" if not all_hits
                   else "禁断边（含 legacy 残留）: " + "; ".join(all_hits))
        remaining = sorted(set(ui_members) & legacy_names)
        ledger.add("SRC-P4c", "recorded",
                   f"legacy 成员残留（T-09 清洁面复核）: {remaining or '无'}")

    # ---- SRC-P5：autodown 跨仓边 ----
    runtime_home = home.get("auto-ui-runtime")
    autodown_edges = [
        e for e in (lang_edges if phase == "pre-migration" else ui_edges)
        if e["from"] == "auto-ui-runtime" and e["dep_key"] == "autodown-core"
    ]
    down = ctx["repos"]["down"]
    if not autodown_edges:
        ledger.add("SRC-P5", "fail" if phase == "source-switched" else "recorded",
                   "auto-ui-runtime 未声明 autodown-core optional 边（跨仓 §4 合同失位）"
                   if phase == "source-switched" else
                   "autodown-core 边缺失：预迁移记档（若 T-06 前被移除须携记录修订阶段表）")
        return
    edge = autodown_edges[0]
    target = edge["target_dir"]
    resolvable = (target / "Cargo.toml").is_file()
    # 声明目标自本树悬挂时，按文档解析序（env→组内兄弟→主检出）补一次 down 仓内
    # 定位：取声明 path 中 auto-down/ 之后的仓内尾部，在已解析 down 仓根下重试。
    order_note = ""
    if not resolvable and down.path.is_dir():
        raw = edge["raw_path"].replace("\\", "/")
        marker = "auto-down/"
        if marker in raw:
            alt = down.path / raw.split(marker, 1)[1]
            alt_name = manifest_package_name(alt / "Cargo.toml")
            if alt_name:
                order_note = f"；声明目标自本树悬挂，按文档解析序于 down 仓解析到 {alt_name}"
    detail = (f"{edge_label(edge)} → 目标{'可解析' if resolvable else '不可解析'}"
              f"（down 仓在案={down.path.is_dir()}）{order_note}")
    if phase == "pre-migration":
        ledger.add("SRC-P5", "recorded",
                   detail + "；固定依赖机制 T-06 冻结（contract-baseline §4）")
    else:
        ledger.add("SRC-P5", "pass" if (resolvable or order_note) else "fail",
                   detail + "；source-switched 后必须可解析（fresh/locked 可复现）")
    _ = runtime_home  # 位置性信息已在 edges 中体现

    # ---- SRC-P6：语言仓原件删除（source-switched）----
    if phase == "source-switched":
        leftover = sorted(set(lang_members) & set(migrating))
        ledger.add("SRC-P6", "pass" if not leftover else "fail",
                   "语言仓成员零五包名（原生产目录删除）" if not leftover
                   else "语言仓残留五包成员: " + ",".join(leftover))
    else:
        ledger.add("SRC-P6", "recorded",
                   f"预迁移：五包预期仍在语言仓（在位 {len(set(lang_members) & set(migrating))}/5）")


# ---------------------------------------------------------------------------
# type 族（V-02）
# ---------------------------------------------------------------------------

def word_contains(text: str, needle: str) -> bool:
    return re.search(r"(?<![A-Za-z0-9_])" + re.escape(needle) + r"(?![A-Za-z0-9_])", text) is not None


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
        present_in = []
        for home in home_entry["homes"]:
            lib = read_text_or_none(lang.path / home / "src" / "lib.rs")
            if lib is not None and word_contains(lib, symbol):
                present_in.append(home)
        status = "pass" if len(present_in) == 1 else "fail"
        detail = (f"{symbol} 单源@{present_in[0]}" if len(present_in) == 1
                  else f"{symbol} 声明单源被破坏：在场={present_in or '无'}（期望恰一，"
                       f"候选={home_entry['homes']}）")
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

    # ---- 编译探针：接口冻结，T-05 起执行 ----
    probe_items = [face["probe"] for face in tp.get("compat_face", [])]
    ctx.setdefault("inventories", {})["compile_probe_uses"] = sorted(set(probe_items))
    requested = "（--compile-probe 已请求，仍记 pending：以最终源执行归 T-05）" if ctx.get("compile_probe") else ""
    ledger.add("TYP-PROBE", "pending",
               f"编译探针接口冻结（{len(set(probe_items))} 条 use 路径，由 touchpoint 表派生）；"
               f"执行归 T-05 定向 V-02{requested}")


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


def run_distribution(ctx: dict, phases: dict, ledger: Ledger) -> None:
    phase = ctx["phase"]
    for shape_id, name, oracle in DISTRIBUTION_SHAPES:
        ledger.add(shape_id, "pending",
                   f"[{name}] {oracle}——接口冻结；执行 phase=source-switched（T-07，"
                   f"当前={phase}）；现行同构控制 generated-ui-consumer/driver.py（815-A）")


def run_failure(ctx: dict, phases: dict, ledger: Ledger) -> None:
    phase = ctx["phase"]
    for shape_id, name, oracle in FAILURE_SHAPES:
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
    real_lang = Repo("lang", HERE.parents[3])
    problems: list[str] = []
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
        repos_s1 = {"lang": real_lang, "ui": Repo("ui", s1_ui),
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
            detect_phase(Repo("ui", s3_ui),
                         json.loads(PHASES_PATH.read_text(encoding="utf-8"))["migrating_packages"])
            _expect(False, "S3 部分导入中间态被拒绝（detect_phase）", problems)
        except UsageError:
            _expect(True, "S3 部分导入中间态被拒绝（detect_phase）", problems)
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

def detect_phase(ui_repo: Repo, migrating: list[str]) -> str:
    members = discover_members(ui_repo.path) if ui_repo.present else {}
    hits = set(members) & set(migrating)
    if not hits:
        return "pre-migration"
    if hits == set(migrating):
        return "source-switched"
    raise UsageError(
        f"UI 仓成员与两阶段结构判别均不匹配：迁移包在场 {sorted(hits)}"
        f"（部分导入=中间态，禁止作为门禁基线；完成 T-06 或修正 --ui-repo）"
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
                        choices=["auto", "pre-migration", "source-switched"])
    parser.add_argument("--output", default=None)
    parser.add_argument("--compile-probe", action="store_true",
                        help="生成 scratch 编译探针（T-05 起启用；当前记录 pending）")
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
    detected = detect_phase(repos["ui"], phases["migrating_packages"])
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
