"""PLAN-751 merge ledger projection: specs.json P751-1（designs 身份三态单点）+
P751-2（reviews 审计收据）+ P751-3（reports th 全档收据）。

沿 p736/p748/p750 验证过的离线投影先例（indent-1 序列化；既有字节除目标
item 外零扰动）。事实源：docs/specs/auto-man/design/api-generation-integrity.md
「workspace lock 一次绑定」节 SD-01 增量（已随 ba9b4281b 落 canonical）。
"""
import json

SPEC = ".autoos/specs.json"
ARCHIVE_PATH = "docs/plans/archive/751-empty-lock-identity-th-tail.md"

with open(SPEC, encoding="utf-8") as f:
    data = json.load(f)

sec_design = sec_review = sec_reports = None
for i, sec in enumerate(data["sections"]):
    ids = {it.get("id") for it in sec.get("items", [])}
    if "P738-2" in ids:
        sec_design = i
    if "P738-4" in ids:
        sec_review = i
    if "P738-5" in ids:
        sec_reports = i
assert None not in (sec_design, sec_review, sec_reports), "anchor sections not found"

existing = {it.get("id") for sec in data["sections"] for it in sec.get("items", [])}
for nid in ("P751-1", "P751-2", "P751-3"):
    assert nid not in existing, f"{nid} already present"

p751_1 = {
    "id": "P751-1",
    "title": "workspace lock 身份分类三态单点（PLAN-751 SD-01，P738-R11-01 收口）",
    "content": (
        "契约=docs/specs/auto-man/design/api-generation-integrity.md「workspace lock 一次"
        "绑定（PLAN-738 R5-01）」节身份分类三态单点行：NotFound=未物化（absent）；成功"
        "读取（含零字节空文件）=实际内容身份（FNV 指纹，空文件=811c9dc5，不得与 absent"
        " 合并）；其它读错误=不可核验（生成端拒绝写收据、复用门保守判陈旧）。生成端写"
        "收据（api_gen.rs）与复用门对拍（rust_ui.rs backend_generation_is_fresh）**共用"
        "同一实现**（workspace_lock_identity），禁止两端口径漂移——R11 反例：生成端记"
        "指纹、复用门 bytes.is_empty() 归 absent，空 lock 再生成成功后未变输入仍被判"
        " stale（永久陈旧）。一次绑定状态机（R5-01）语义保留并覆盖空文件臂：absent→"
        "空文件出现判新鲜并把 811c9dc5 绑定写回收据；此后漂移/删除/读错误/缺字段均"
        "陈旧。回归锚：rust_ui::tests::{workspace_lock_identity_three_states, "
        "review751_empty_lock_identity_matrix, review738_r5_lock_binding_state_machine}。"
        "reviewed_commit 226ebfbbb（rebase 前 f16de5a19）+ ba9b4281b（spec）。"
    ),
    "file": "docs/specs/auto-man/design/api-generation-integrity.md",
    "related": ["PLAN-751", "PLAN-738"],
    "status": "published",
    "created": "2026-10-10",
}
p751_2 = {
    "id": "P751-2",
    "title": "PLAN-751 审计收据（review pass → merge delivered）",
    "content": (
        "复审 rv1 pass（plan_revision 1；reviewed_commit f16de5a19+fe3d78937，rebase 后"
        " 226ebfbbb+ba9b4281b，基面 0884add2a；依赖 auto-down detached@895f8d0）。独立性"
        "限制披露：执行会话内复审，裁定从工件重建（关键验证于已提交修订复跑：三测试"
        " 3/3 PASS、调用点 grep 恰两处+classify_lock_read 零残留、rustfmt exit 0）。"
        "AC-01..06 全 pass；findings P751-D1（high：plan730 上传族 14 项+e2e_a_redirect_302"
        " 确定性挂死，红移窗口=PLAN-738 系列，th 面与 751 diff 零因果）与 P751-D2"
        "（medium：auto-man 3 预存红=日常档盲区）登记 KNOWN-DEBT 非阻塞。R11 修复要求"
        "五要素逐项对上；R11 探针场景形式化为常驻回归测试。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-751", "PLAN-738"],
    "status": "published",
    "created": "2026-10-10",
}
p751_3 = {
    "id": "P751-3",
    "title": "完整串行 th 验收收据（102/102，PLAN-751 承接 PLAN-738 R9 尾项）",
    "content": (
        "cargo th --jobs 1 --no-fail-fast（worktree lang-751，auto-lang 面=master 0884add2a）"
        "：102 selected/102 run，86 pass/2 fail/14 timeout，1962.969s。R9 未跑 57 项全部补跑"
        "+45 项重跑。分诊：back_proxy 在册基线红（两轮同形）；plan730 interop 109.988s"
        "确定性失败（R9 同形 109.655s）；plan730 上传族 13 项+e2e_a_redirect_302 确定性"
        "挂死（隔离复现 120s TIMEOUT，服务端 accept 后不响应；与负载无关——同条件"
        " plan326 兄弟 39 项 0.6s 全绿）；根因 bisect 走后续专项（候选锚 dd9ffe35b/"
        "19973b089，plan734 时代 th 99/101 绿）。收据不宣称 HTTP 面全绿。"
    ),
    "file": "docs/plans/reports/751-th-full-receipt.md",
    "related": ["PLAN-751", "PLAN-738"],
    "status": "published",
    "created": "2026-10-10",
}

data["sections"][sec_design]["items"].append(p751_1)
data["sections"][sec_review]["items"].append(p751_2)
data["sections"][sec_reports]["items"].append(p751_3)

with open(SPEC, "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=1)
    f.write("\n")

# 回读核验
with open(SPEC, encoding="utf-8") as f:
    back = json.load(f)
got = {it.get("id"): it for sec in back["sections"] for it in sec.get("items", [])}
for nid in ("P751-1", "P751-2", "P751-3"):
    assert nid in got, f"read-back missing {nid}"
assert got["P751-1"]["file"].startswith("docs/specs/"), "P751-1 file anchor wrong"

print(
    "specs.json: P751-1/2/3 appended & read-back verified;",
    "sections", sec_design, sec_review, sec_reports,
    "| items now:", sum(len(s.get("items", [])) for s in back["sections"]),
)
