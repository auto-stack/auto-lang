"""PLAN-755 audit-closure ledger projection: specs.json P755-4（reports 外部审计
打回应对收据）。

沿 p685/p736/p748/p750/p751 验证过的离线投影先例（indent-1 序列化；既有字节除
目标 item 外零扰动）。事实源：docs/plans/reports/755-external-audit-response.md
（执行会话）+ reports/755-landed-independent-check.md（审计会话 R4，0f27d0cd0）。
store 中介写入不可达（musk 后端未运行，README §5 手工回退路径，先例
p736/p748/750/751/752 及 755 原合并轮 P755-1/2/3）。
"""
import json

SPEC = ".autoos/specs.json"

with open(SPEC, encoding="utf-8") as f:
    data = json.load(f)

sec_reports = None
for i, sec in enumerate(data["sections"]):
    ids = {it.get("id") for it in sec.get("items", [])}
    if "P755-3" in ids:
        sec_reports = i
assert sec_reports is not None, "anchor reports section (P755-3) not found"

existing = {it.get("id") for sec in data["sections"] for it in sec.get("items", [])}
assert "P755-4" not in existing, "P755-4 already present"

p755_4 = {
    "id": "P755-4",
    "title": "PLAN-755 外部审计打回应对收据（落地态无罪 + blueprints WIP 处置）",
    "content": (
        "2026-10-11 用户转述外部审计打回。双会话闭合：①执行会话重建审计视图——守卫 4/4、"
        "两原反例 2/2、并行 th 102/100/2（back_proxy 在册红+quoted_boundary 已录 flake，"
        "隔离 PASS）；日常档 32 红逐项归因=13 在册基线+18 主检出未提交 blueprints/ 删除"
        "（68 文件，另一任务 WIP，Plan 758 在案登记禁止重置；直接 Os NotFound+间接"
        " 041-auto-edit 零副本依赖 tree_util.at 致 Undefined symbol）+1 并行 flake；"
        "用户裁定废弃物→git checkout -- blueprints/ 恢复，复验 31/31 受影响族+全档"
        " 5231/5218/13 全在册。②审计会话 R4 归档后独立核查 pass（0f27d0cd0）：clean 树"
        "串行 plan755 OR http_e2e 106/105/1 在册红/0 超时（265.241s）、四文件 fmt exit 0、"
        "blueprints 恢复逐文件哈希与 HEAD 一致；755 archived 终态保持不回退，P751-D1"
        "销案结论维持。本轮零代码变更（工作树复原至 HEAD，簿记类收尾）。"
    ),
    "file": "docs/plans/reports/755-external-audit-response.md",
    "related": ["PLAN-755", "PLAN-751"],
    "status": "published",
    "created": "2026-10-11",
}

data["sections"][sec_reports]["items"].append(p755_4)

with open(SPEC, "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=1)
    f.write("\n")

# 回读核验
with open(SPEC, encoding="utf-8") as f:
    back = json.load(f)
got = {it.get("id"): it for sec in back["sections"] for it in sec.get("items", [])}
assert "P755-4" in got, "read-back missing P755-4"
assert got["P755-4"]["file"] == "docs/plans/reports/755-external-audit-response.md"

print(
    "specs.json: P755-4 appended & read-back verified;",
    "reports section", sec_reports,
    "| items now:", sum(len(s.get("items", [])) for s in back["sections"]),
)
