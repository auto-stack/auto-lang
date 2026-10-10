"""PLAN-750 merge ledger projection: specs.json P750-1（designs SD-01 契约）+P750-2（reviews rv1 pass 收据）。

Mirrors the p736/p748_ledger.py validated offline projection precedent
(indent-1 serialization; existing bytes must not churn beyond target items).
Source of truth: docs/specs/auto-lang/ui/overview.md「vue 轨 store handler
事件重入与 await 悬窗契约（PLAN-750）」节（SD-01 已随本 merge 落 canonical）。
"""
import json

SPEC = ".autoos/specs.json"
ARCHIVE_PATH = "docs/plans/archive/750-vue-draft-settle-race.md"

with open(SPEC, encoding="utf-8") as f:
    data = json.load(f)

sec_design = sec_review = None
for i, sec in enumerate(data["sections"]):
    ids = {it.get("id") for it in sec.get("items", [])}
    if "P748-1" in ids:
        sec_design = i
    if "P748-3" in ids:
        sec_review = i
assert sec_design is not None and sec_review is not None, "anchor sections not found"

existing = {it.get("id") for sec in data["sections"] for it in sec.get("items", [])}
for nid in ("P750-1", "P750-2"):
    assert nid not in existing, f"{nid} already present"

p750_1 = {
    "id": "P750-1",
    "title": "vue 轨 store handler 事件重入与 await 悬窗契约（PLAN-750 SD-01）",
    "content": (
        "契约=docs/specs/auto-lang/ui/overview.md「vue 轨 store handler 事件重入与 await "
        "悬窗契约（PLAN-750）」节，五条：①handler 体在显式 await 点之间的同步段原子；"
        "②await 悬窗内同 store 事件**可重入**（两轨一致——vm INPUT_TEXT/vue 引擎事件同理，"
        "重入安全由身份重查而非调用互斥达成）；③悬窗内身份分配（x==\"\" → await alloc() 形态）"
        "的**单飞责任在宿主**——重入双分配覆写身份，身份重查兜不住「身份被换」，分配臂须 "
        "in-flight 门（范式指针 evidence/p750/supply-draft.md §三；jade d0003 卡死+d0004 空"
        "目录双分配事故直证）；④双轨事件节奏分野——vm 轨 MCP 往返串行、vue 轨引擎逐键直达"
        "（首挂载逐键发射存活，间隔可 ~5ms < 悬窗 15-60ms），vue 宿主必须按「事件可落进任意"
        " await 悬窗」设计，vm 绿不构成反证；⑤保存流 write+read 双 await 屏障=观测事实非时序"
        "保证。负面契约：vue 轨**不提供**跨 handler 串行化（键入时延会耦合前序排水链——jade "
        "PLAN-031 T-05 逐键 POST 风暴要规避的形态），互斥需求由宿主 .at 以 in-flight 门表达。"
        "归因背景：jade 4/4 结算失败=RC-C 潜伏竞态（codegen 零 diff+后端 RTT 恒等+绿时代载体"
        " v2747 同败三证），非 auto-lang 回归；修复主体=jade .at 分配臂守卫（供料包交接）。"
        "证据：docs/plans/evidence/p750/{repro.md,repro.mjs,supply-draft.md}，reviewed_commit "
        "ea513a627。"
    ),
    "file": "docs/specs/auto-lang/ui/overview.md#vue-轨-store-handler-事件重入与-await-悬窗契约（PLAN-750）",
    "related": ["PLAN-750", "PLAN-748"],
    "status": "published",
    "created": "2026-10-10",
}
p750_2 = {
    "id": "P750-2",
    "title": "PLAN-750 rv1 复审收据（2026-10-10）",
    "content": (
        "reviewed_commit=ea513a627（基线 95d5d3297，diff=仅 docs/ 六文件 631 行零 crates/ 变更"
        "——Category A 门禁成立）。同会话复审限制披露+工件重建：D 场景独立重跑第三次复现完整"
        "失败签名（draft_begin×2/双分配 d0002 空目录/checkpoint erev=2 卡死无 settle——复现器"
        " 3/3）。acceptance：AC-03/04/06/07 pass；AC-01/02/05 pass-on-contract+external-pending"
        "（产品面终态绿依赖 jade 应用供料——748-AC-14 先例结构，owner=jade，复验回执回填归档"
        "计划 §9）。findings F-R1..R4 全 nonblocking（external-pending 登记/SD-01 交叉引用措辞/"
        "run1 错配产物留档/矩阵设计被更直接判别取代之理由在案）。next=merge。外部跟踪：jade"
        " 供料包=evidence/p750/supply-draft.md（.at in-flight 门修复范式+负载窗否证纪律）。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-750"],
    "status": "published",
    "created": "2026-10-10",
}

data["sections"][sec_design]["items"].append(p750_1)
data["sections"][sec_review]["items"].append(p750_2)

with open(SPEC, "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=1)
    f.write("\n")

# 回读核验
with open(SPEC, encoding="utf-8") as f:
    back = json.load(f)
got = {it.get("id"): it for sec in back["sections"] for it in sec.get("items", [])}
assert "P750-1" in got and "P750-2" in got, "read-back missing items"
assert got["P750-1"]["file"].startswith("docs/specs/"), "P750-1 file anchor wrong"

print(
    "specs.json: P750-1/2 appended & read-back verified;",
    "sections", sec_design, sec_review,
    "| items now:", sum(len(s.get("items", [])) for s in back["sections"]),
)
