"""PLAN-748 merge ledger projection: specs.json P748-1/2/3 + P733-2 边界③刷新。

Mirrors the p736_ledger.py validated offline projection precedent
(indent-1 serialization; existing bytes must not churn beyond target items).
Source of truth: docs/specs/auto-lang/ui/overview.md 两新节 +
vm/design/vm-fn-call-semantics.md 边界③收口（SD-01..03 已随本 merge 落 canonical）。
"""
import json

SPEC = ".autoos/specs.json"
ARCHIVE_PATH = "docs/plans/archive/748-vm-cond-eval-and-field-write.md"

with open(SPEC, encoding="utf-8") as f:
    data = json.load(f)

sec_design = sec_review = None
for i, sec in enumerate(data["sections"]):
    ids = {it.get("id") for it in sec.get("items", [])}
    if "P655-1" in ids:
        sec_design = i
    if "P749-4" in ids:
        sec_review = i
assert sec_design is not None and sec_review is not None, "anchor sections not found"

existing = {it.get("id") for sec in data["sections"] for it in sec.get("items", [])}
for nid in ("P748-1", "P748-2", "P748-3"):
    assert nid not in existing, f"{nid} already present"

p748_1 = {
    "id": "P748-1",
    "title": "overflow 滚动转写·视口视觉层（PLAN-748，jade 件一根修）",
    "content": (
        "契约=docs/specs/auto-lang/ui/overview.md「overflow 滚动转写·视口视觉层（PLAN-748）」"
        "节：overflow-y-auto/overflow-auto col 经 aura_view_builder needs_scroll 转写 "
        "View::Scrollable 时，视觉类（bg/border/rounded/shadow/ring，StyleClass::"
        "is_visual_paint）上移**视口 Container**（CSS 同构：滚动容器背景属容器本体，随容器"
        "拉伸满高；内容 col 拿剥离余集——此前整串 clone 使视觉随内容止于内容高，「侧栏 bg "
        "底边=树内容底」根因，flex-col 显式显示类阻断 overflow-y Fill 推断时触发）；视口"
        "Container 拷贝原显式 Width/Height 类缺省补 Full、Scrollable 容器内 Fill×Fill（"
        "ensure_full_scroll_dims——iced 0.14 Limits::Shrink 置 compression 位、Fill 子件压缩"
        "轴解析内容高，Shrink 包装塌缩 42px 实证）；无视觉类零包装层现行为不变；类序无关由"
        "parse 逐 token 保证（p748_j2 三变体等价锁）。T-06 定音：StretchLine 无罪（供料"
        "「final 遍 min_h 未兑现」定位证伪，P748-SL dump 实证全兑现）。证据：4b4a7251e，"
        "jade-real pre-workaround live 像素终验 bg-card rgb(13,20,37) 满高+041 live 冒烟"
        "（侧栏新结构正收益）；回归锚 layout_tests p748_j1/j2/j3+结构锁。"
    ),
    "file": "docs/specs/auto-lang/ui/overview.md#overflow-滚动转写-视口视觉层（PLAN-748）",
    "related": ["PLAN-748", "GOAL-007"],
    "status": "published",
    "created": "2026-10-10",
}
p748_2 = {
    "id": "P748-2",
    "title": "autodown_editor 引擎 parity（PLAN-748，jade 件四）",
    "content": (
        "契约=docs/specs/auto-lang/ui/overview.md「autodown_editor 引擎 parity（PLAN-748）」"
        "节：vm 轨 autodown_editor 内建引擎等值 wrapper padding——上下 16/左右 20 逻辑 px，"
        "单一真源=auto-down 引擎 autodown-editor.css:103 .autodown-editor-content-wrapper "
        "{padding:1rem 1.25rem}（vue 轨引擎 CSS 承担、vm 件内建等值层，双轨呈现等值）。"
        "widget 四点联动：layout 内缩测量（宽-40/高+32）/draw 内容原点平移（to_rect+"
        "fill_text ox/oy）/update 鼠标命中坐标内缩/IME 光标锚偏移。常量锚 widget.rs "
        "CONTENT_PAD_X/Y（单源锁测试）；行为锚 plan732 真实 iced 事件全管线点击+"
        "p748_t10 布局级断言（哨兵 y=内容高+32，实测 88.6=56.6+32）；autodown 档 127/127。"
        "证据：bb494c762；jade 撤 px-5 py-4 临时层复验=AC-14 external-pending。"
    ),
    "file": "docs/specs/auto-lang/ui/overview.md#autodown-editor-引擎-parity（PLAN-748）",
    "related": ["PLAN-748"],
    "status": "published",
    "created": "2026-10-10",
}
p748_3 = {
    "id": "P748-3",
    "title": "PLAN-748 复审与合并收据（rv4 两轮）",
    "content": (
        "review R1 needs_fix（R-01 AC-04 改判：词表可观测性经 Style::parse_reported+"
        "style_parity 通道满足，每臂 WARN=WARN 风暴反模式弃，§4「446-U7 button 臂 WARN」"
        "系草案误引；R-02 规范增量按落地实态重写——原 SD-02/03 StretchLine 修法草案随 T-06 "
        "定音证伪废弃；R-03 T-05/T-11 收口+gallery 归位批量档；R-04 master 并发 rebase 注意）"
        "→R2 pass（代码零变化复用一轮门禁：tv 162/162+裸 t 5130/5145 十五红全归因[9 在案+3 "
        "负载 flake：clipboard/plan502 P733-R1/plan705 scoped 19/19 绿]+autodown 127/127+"
        "041 live 冒烟）。Phase 1：三断点定音②③证伪（dirty 构建陈旧）、①收缩 P733-R2 面"
        "——.store.X 泛名展平根修（2e2164b5e）+p054×2 图标判别器 lucide 名集根修"
        "（3ec6a4914，musk_vm_track 97/6→102/103，唯余 p053_4 异根改派 codegen 债）。"
        "Phase 2（jade 供料四件）：件一转写层根修 4b4a7251e/件二件三 HEAD 证伪+回归锁"
        "（release 2747 观测无代码差可归因——观察项交 jade 撤层复验 AC-14 终局销案）/件四"
        "bb494c762。债：P748-D1（plan051 corpus 空洞绿）+P733-R2 条目③余留 p053_4。"
        "合并：worktree lang-748 六实现提交+specs 落账后裔为 delivery，rebase onto master "
        "（并发 515acb47c）ff-only；账本=validated offline projection（p736 先例）。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-748"],
    "status": "published",
    "created": "2026-10-10",
}
data["sections"][sec_design]["items"].append(p748_1)
data["sections"][sec_design]["items"].append(p748_2)
data["sections"][sec_review]["items"].append(p748_3)

# P733-2 边界③刷新（同一 canonical target 的 current-knowledge 更新，非新增项）
for it in data["sections"][sec_design]["items"]:
    if it.get("id") == "P733-2":
        it["content"] = (
            "语义矩阵 8 行（handler 域全参型真值/跨轮次 timer 消费/computed 三形态含列表与 "
            "obj 字面量实参/歧义格/t 动态键）+已知边界 4 条：①懒挂载时序（PLAN-711 pending "
            "demand,每拍重算自愈）②push_value 对 Rust 外发堆引用实参占位 0（既有约定,主链"
            "只传标量）③musk_vm_track p053/p054 预存红族——**已收口（PLAN-748 T-02，"
            "2026-10-10）**：六红三根=.store.X 泛名展平三口径归一根修（p053_1×2+p053_6 翻绿）"
            "+p054×2 图标判别器 lucide 名集成员资格（contains(\"icon\") 粗闸废弃）+p053_4 "
            "CALL_NAT 发射异根改派 codegen 债务观察项（本边界唯一余留，musk_vm_track "
            "102/103）④视图 .length 成员链表现层（分层归因）。维护规则：改 call_vm_fn 实参"
            "编码先对照矩阵补格。"
        )

with open(SPEC, "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=1)
    f.write("\n")

print(
    "specs.json: P748-1/2/3 appended, P733-2 boundary③ refreshed;",
    "sections", sec_design, sec_review,
    "| items now:", sum(len(s.get("items", [])) for s in data["sections"]),
)
