"""PLAN-685 merge ledger projection: specs.json P685-1/2 + ui/plans.md row.

Mirrors the P676 entry schema exactly; indent-1 serialization verified by
diff (existing bytes must not churn).
"""
import json

SPEC = ".autoos/specs.json"
PLANS_MD = "docs/specs/auto-lang/ui/plans.md"
ARCHIVE_PATH = "docs/plans/archive/685-vm-bp-callback-arg-forward.md"

with open(SPEC, encoding="utf-8") as f:
    data = json.load(f)

# locate anchor sections by P676 entries (same homes)
sec_design = sec_review = None
for i, sec in enumerate(data["sections"]):
    ids = {it.get("id") for it in sec.get("items", [])}
    if "P676-1" in ids:
        sec_design = i
    if "P676-2" in ids:
        sec_review = i
assert sec_design is not None and sec_review is not None, "anchor sections not found"

existing = {it.get("id") for sec in data["sections"] for it in sec.get("items", [])}
assert "P685-1" not in existing and "P685-2" not in existing, "P685 entries already present"

p685_1 = {
    "id": "P685-1",
    "title": "vm-bp-callback-arg-forward — VM 臂体内回调形参绑定根修 + bps 内容页 autodown 渲染/三节 tab/头部专业化（SD-01/02/03）",
    "content": (
        "根修（P657-D1 名字面量化形态收口）：体内剥离回调 `on_x(expr)` 实参以文本快照入 "
        "child_emit STRIPPED 表（Expr 非 Send），派发侧 eval_stripped_arg 求值域仅 root "
        "state、handler 形参不可见 → 查名失败走名字即值字面量兜底（bps-gallery 实拍 "
        "selected_id=\"id\"）。修 = vm_bridge handler_param_counts(usize) 扩为 "
        "handler_param_names(Vec<String>)（B12(b) 契约 handler_param_count 不变，arity="
        "vec len）+ dynamic.rs 派发侧形参名×实参配对绑定表，裸标识符先查绑定（词法最近）"
        "再退 state，this./. 前缀显式指 state 跳过，裸词字面量兜底保留；邻近三形态"
        "（字面量参/state 路径参/oninput 事件参）断言保绿。首启竞态判同根（带修 0/10 vs "
        "基线 2/5；GET_FIELD ObjectData 缺键读 null 无名字退化臂——字段访问退化假设证伪）。"
        "内容页重设计：active_section 三节状态直切（spec 缺省，切 bp 保持分节；零滚动依赖）"
        "+ Spec/Gotchas 换 markdown 渲染件双臂（VM=autodown-core 真渲染，vue=pac.at "
        "npm_deps @autodown/engine 绝对 link→StreamingRenderer；裸 pre 仅限代码源码面）"
        "+ 头部三件（kind 面包屑可点击=SelectKind 等价/变体计数徽标/prev_next 按 "
        "gallery_items 序到头空串降灰+SelectBp 空串守卫）。vue 臂走查实证 computed 实参位"
        "链 computed（f(.np) 形态）解析为 state 字段读→undefined→整页白屏（VM 臂容忍）——"
        "装配纪律「computed 单 fn 直呼」的跨臂差异面实锤，全改单 fn 直呼下沉 catalog 重导。"
        "E2E 门禁 = .agents/skills/autoui-verifier/scripts/test_bp_gallery_click.py"
        "（blueprints/ 目录解析卡片+首帧竞态探针+首帧 no-op press 有界重试）。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-685"],
    "status": "published",
    "created": "2026-09-22",
}
p685_2 = {
    "id": "P685-2",
    "title": "vm-bp-callback-arg-forward 复审与合并收据",
    "content": (
        "review pass@plan-685-dev rev2（同会话复审已声明——裁定全部工件重建，独立性限制"
        "随档）。AC-01 红相 base 重建（临时树 0e541c7e5+纯测试：两形参测 Str(\"v\") 红/"
        "字面量测绿）+ 审定点绿 3/3；AC-02/03 E2E×2 exit 0（重建 exe 后复跑，3 卡跨 3 "
        "kind，首帧竞态 0/2）；AC-04 cargo t 24 红/tv 3 红/tf 等价 5887 集 30 红——全数 "
        "base 对勘预存（含仅 test-book 组合暴露的 book×7，base 同红），零新增；AC-05 "
        "同根判定+Q-1 修复同批留痕；AC-06/09 双臂截图网格视检（docs/plans/reports/p685/，"
        "markdown 层级/行内码/表格 chrome、Reference 反引号原样）；AC-07/08 运行期探针"
        "（tab 直切状态+内容双翻转、Q-4 keep 非缺省态实证、头部件 runtime 在位）。"
        "findings：F-1 SD 三条目标勘正归位 blueprint/project.md（auto-vm spec=独立运行器、"
        "autoui-skill 明文不做验证执行；P657-D1 在案家）——规则文本零改动非契约变更；"
        "F-2 rebase 六对 range-diff 全等（master 前移 0e541c7e5→00e56202d 零文件交集）；"
        "F-3 E2E 首帧 no-op press 窗口现象（~1/3，MCP window-size-zero 同族）有界重试"
        "覆盖，值错误仍立即红。Q-2 E2E 进 CI：CI 现状无 GUI 实例 runner（vm-files-ci 为 "
        ".at 语料档），GUI 实例起停环境缺失——不进，门禁以本地脚本为准（SKILL.md 登记）。"
        "Q-3 高亮边界登记不阻塞；Q-4 切 bp 保持分节已实现成文。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-685"],
    "status": "published",
    "created": "2026-09-22",
}
data["sections"][sec_design]["items"].append(p685_1)
data["sections"][sec_review]["items"].append(p685_2)

with open(SPEC, "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=1)
    f.write("\n")

# ui/plans.md row (numeric position: append after last row starting with "| 6")
with open(PLANS_MD, encoding="utf-8") as f:
    lines = f.readlines()
last_table = max(i for i, l in enumerate(lines) if l.startswith("| ") and l.count("|") >= 5)
row = ("| 685 | vm-bp-callback-arg-forward | ✅（reviewed→archived） | "
       "archive/685-vm-bp-callback-arg-forward.md | VM 臂体内回调剥离实参形参绑定根修"
       "（P657-D1 名字面量化形态收口：handler_param_names 绑定表，名字即值 forbidden）"
       "+首启竞态判同根（0/10）+bps 内容页重设计（active_section 三节直切/markdown 渲染件"
       "双臂/头部三件 prev-next）+E2E 门禁 test_bp_gallery_click.py；坑=computed-as-arg "
       "vue 白屏（装配纪律实锤）|")
lines.insert(last_table + 1, row + "\n")
with open(PLANS_MD, "w", encoding="utf-8", newline="\n") as f:
    f.writelines(lines)

print("specs.json sections:", sec_design, sec_review, "| plans.md row after line", last_table + 1)
