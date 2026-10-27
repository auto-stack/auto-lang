# 828 第一批 R3：仅余 B1-R3-01 / P2 窄修复

同树 `D:/autostack/.wt/lang-828/auto-lang`、`plan-828-dev@599cc15afee9fb964a8a88a243c2210ea0597aa2`。R2三项反例已独立7/7闭合，原12控制、b1 9/r1 86及其余定向面保持；不要重做依赖归属/关闭深副本/稳定引用机制，不撤销后批。

剩余一项：can3 声称覆盖 HIR PartialEq 语义字段，但 CallBinding 只编码 param/arg，遗漏 `moved`（hir.rs明确semantic，codec v2真实承载）。本包 canonical-controls.rs 把当前 main 的绑定 false→true，HIR不等、canonical相同、verify接受、restore仍3/3采纳，resident变成true。没有新增i32运行错误证据，但当前内容/恢复身份合同未满足。

按既定字段政策让现有 moved 进入内容身份，或在当前profile工件准入对该不支持标记受控拒绝；不增加owned Str/.move语言能力，也不要只改文档把semantic改成cosmetic。若内容算法改变，按原共享store规则隔离旧can3键语义，HIR/ABC/sidecar使用一致规则，保留旧行不覆盖。

保持同一反例 mutation 和采纳 oracle：坏变体不能成为当前false源码的HIR；合法false工件/原owner0和1、wrong owner/index/span/intrinsics、mixed引用、赋值目标编辑冷编对拍、历史依赖、取消/迟到/关闭全部保持。精确重放 R3 canonical-controls（候选取消/迟到正控也在其中）和所需永久/兼容目标，补一项永久字段回归并给关闭行。只跑必要定向检查，不再跑512或整档。核 HEAD clean与输入依赖配置、冻结新SHA后送窄复审；executing0/11与后批/最终/merge-owned pending不变。
