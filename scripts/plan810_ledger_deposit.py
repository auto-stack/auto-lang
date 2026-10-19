#!/usr/bin/env python3
"""PLAN-810 merge: guarded offline ledger projection (Regime B).

Store service unreachable (HTTP probe empty). Backs up .autoos/specs.json
byte-exact, appends exactly four surgical items (designs/tests/reviews/reports)
to their sections via scripted read-modify-write, bumps version by one,
validates the six-section schema, and reads back the written IDs. Everything
else stays byte-identical (proven by a scoped diff).
"""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / ".autoos/specs.json"
BACKUP = ROOT / ".autoos/specs.json.bak-v51-810"

HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                      capture_output=True, text=True).stdout.strip()
DELIVERY = HEAD[:9]

DESIGN = {
    "id": "P810-D1",
    "title": "完整 UI 文法单源外置与 UiParseContext 端口（PLAN-810）",
    "content": (
        "widget/store/component/view/fragment/节点族/on 块/actions/style/timer/routes/"
        "menubar/toolbar/scene/use/setup/expose/watch/computed/bind 及全部依赖 helper"
        "（约 4.9k 行）自 auto-lang Parser 逐字迁 auto-ui-dialect grammar/{widget,view,"
        "scene,store,actions,helpers}，经对象安全子 trait UiParseContext:ParseContext"
        "（定义在文法 crate，不入 Core 包，ParseContext 面零改动）驱动；ParserBridge "
        "实现+Parser 端口后端（raw_cursor_save/restore/release 投机生命周期：成功出口"
        "显式释放覆盖公开门面/Core 直连/派发/嵌套全入口；probe_next_token 失败上抛协议；"
        "advance_peek_kind 与 _resync 区分 stream 手动恢复与 scene/subresource 双 push "
        "重同步两种 prev 终态；computed 局部诊断事务；raw CSS 捕获；bind_var/"
        "define_handler_param/register_style_recipe 副作用出口）。ParserBridge::parser() "
        "escape hatch 删除，dialect/ui 薄 adapter 全端口派发，Parser 留同签名薄委托门面；"
        "on-events/arrow 事件文法留 Core（共享）。具名冻结旧怪癖：on Err 参数作用域泄漏、"
        "未闭合 fragment 无界 EOF、fn 体 store 门。canonical 见 file+full-grammar.md。"
    ),
    "status": "active",
    "file": "docs/specs/auto-ui-dialect/project.md",
    "tags": ["PLAN-810", "stage:merge", f"delivery:{DELIVERY}", "scope:auto-ui-dialect"],
}

TESTS = {
    "id": "P810-T",
    "title": "完整 UI 文法外置永久回归与轻消费方资产（PLAN-810）",
    "content": (
        "crates/auto-lang/src/tests/plan810_full_ui_grammar.rs 23 例（V-01 文法全集交叉×2/"
        "V-02 事务与前瞻 7 例/V-03 scope 凭据 owned-AST shadow 6 例/V-04 派发对照与三层"
        " use-import 链/R1 快照生命周期双测含四入口 n×64 存活恒 0 与外层凭据存活/"
        "R2 同实例成功-失败对照冻结旧泄漏/EOF 隔离用例 #[ignore] 仅经 scripts/"
        "plan810_eof_quirk_probe.py 子进程外截止观察）；scripts/plan810_baseline_identity.py"
        " 基面↔迁移 fragment 路径 10 函数冻结映射归一同构证明；轻消费方 test/fixtures/"
        "modularity/ui-grammar-consumer（ToyUiContext 全端口 7 例+verify-deps 闭包门）；"
        "schema_drift_fence get_primary_prop 表维度随迁 grammar/helpers.rs。"
    ),
    "status": "active",
    "file": "crates/auto-lang/src/tests/plan810_full_ui_grammar.rs",
    "tags": ["PLAN-810", "stage:merge", f"delivery:{DELIVERY}", "tier:scoped"],
}

REVIEW = {
    "id": "P810-R4",
    "title": "PLAN-810 final review R4 — pass（R1..R4 全 finding 闭环）",
    "content": (
        "四轮收口：R1 六 findings（快照残留/777 基线/scene prev 双形态/测试 oracle/SD "
        "冲突/格式）→R2 三 findings（同实例错误状态与 EOF 基线证据/792 旧源包收据/规范"
        "矛盾句）→R3 一 P3（unused_mut×2）→R4 pass 绑定 6b4963de3。merge rebase over "
        "808/809 后映射 2a284c779，range-diff 12/12 等价（777 基线冲突项按并集显式重采样）。"
        "AC-01..06 pass；AC-07 merge-owned（冻结 t/tv/tu+沉淀/归档/清理）。"
    ),
    "status": "active",
    "file": "docs/plans/attachments/810/review-r4/REVIEW.md",
    "tags": ["PLAN-810", "stage:review", "reviewed:6b4963de3", f"rebased:{DELIVERY}"],
}

REPORT = {
    "id": "P810-M",
    "title": "PLAN-810 merge 沉淀收据（2026-10-19）",
    "content": (
        f"delivery {DELIVERY}（rebase over master 含 808/809 并集，range-diff 12/12 等价；"
        "777 并集基线显式重采样 declared_edges=69 绑定交付提交）ff-only 合入；冻结门：裸 "
        "cargo t 5300/5301（唯一红 plan484_024_charts_streaming_recompute——主检出同全量"
        "同红实证为预存非本计划引入，隔离 3×绿，归批量回归分诊）+ cargo tv 163/163 + "
        "cargo tu 867+2 ignored；tt 不适用（trans 零触）；schema_drift_fence 表维度随迁修复"
        "（围栏自指引）；792 修复源 25/25 与包内 8 触面源 hash=受审源；六份 Spec 增量"
        "（SD-01..06 含新 full-grammar.md）；ledger v51→v52 guarded offline projection"
        "（无存储写方可达，备份 bak-v51-810）。"
    ),
    "status": "active",
    "file": "docs/plans/attachments/810/handoff.md",
    "tags": ["PLAN-810", "stage:merge", f"delivery:{DELIVERY}"],
}

SECTION_OF = {"P810-D1": "designs", "P810-T": "tests", "P810-R4": "reviews", "P810-M": "reports"}
NEW_ITEMS = [DESIGN, TESTS, REVIEW, REPORT]


def main() -> int:
    original = LEDGER.read_bytes()
    shutil.copyfile(LEDGER, BACKUP)
    doc = json.loads(original.decode("utf-8"))

    ids = {s["id"] for s in doc["sections"]}
    assert ids == {"goals", "architecture", "designs", "tests", "reviews", "reports"}, ids
    for item in NEW_ITEMS:
        for sec in doc["sections"]:
            if sec["id"] != SECTION_OF[item["id"]]:
                continue
            assert not any(i["id"] == item["id"] for i in sec["items"]), item["id"]
            sec["items"].append(item)

    doc["version"] = doc.get("version", 0) + 1
    LEDGER.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    # read-back + surgical-scope proof
    back = json.loads(LEDGER.read_text(encoding="utf-8"))
    assert back["version"] == json.loads(original.decode("utf-8"))["version"] + 1
    found = 0
    for sec in back["sections"]:
        for it in sec["items"]:
            if it["id"] in SECTION_OF:
                assert sec["id"] == SECTION_OF[it["id"]]
                assert it["file"] and it["tags"]
                found += 1
    assert found == 4, found
    old_ids = {
        (s["id"], i["id"])
        for s in json.loads(original.decode("utf-8"))["sections"]
        for i in s["items"]
    }
    new_ids = {(sec["id"], i["id"]) for sec in back["sections"] for i in sec["items"]}
    assert old_ids <= new_ids and len(new_ids - old_ids) == 4
    print(f"ledger v{json.loads(original.decode('utf-8'))['version']}->v{back['version']}: "
          f"+4 items {[i['id'] for i in NEW_ITEMS]}, backup {BACKUP.name}")
    return 0


def sec_items(sec):
    return sec["items"]


if __name__ == "__main__":
    raise SystemExit(main())
