# Plan 815 T-04 R3 定向复审

**pass（仅 T-04 提前定向审查）；R1 六项全部 CLOSED，无新增 finding。**

Plan r3；新冻结 SHA `d0b7e6ebda2cc14e9a04af3d7c9cc89ee2aedcc8`，修复基面 `dbd295f050c05ef37450adbd0f8f835a150e077f`。唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`；首尾 clean、HEAD 未变。review_scope=phase/T-04，非 T-09 全计划终审。审查者 Codex `/root` 延续独立复审上下文，保留早期合同/账本参与限制，未参与本轮修复，未创建其他树/agent，不假称其他模型。

## F06 关闭与本轮验证

提交只改 `crates/auto-lang/src/lib.rs` 一处公开转发行，由一行展开为三行。独立取 R2 版本全文执行纯输出 rustfmt，其输出与当前文件完全一致（不是仅按摘要认定无行为变化）。

- 单文件 `rustfmt --check --edition 2021 --config skip_children=true crates/auto-lang/src/lib.rs`：exit0，见 fmt-root.log.txt。
- 同式检查 T-04 原基面 7ffa2d88d 至当前的全部 A/C/M Rust 净交付文件：exit0，见 fmt-delivery.log.txt。
- 修复 diff --check：exit0，见 diff-check.log.txt。

F-815-T04-R1-06 的格式不变量现已闭合；01..05 保持 R2 独立 CLOSED。本轮未重跑 Rust 行为测试或 792 冷构建。

## 证据复用依据与边界

[baseline.json.txt](baseline.json.txt)逐项核验：R2 记录的工作树代码/锁/清单/测试配置/边界配置仅 lib.rs 格式变化，全部其他 hash 未变；canonical Spec、阶段合同与 fixes 输入未变；R2 整套持久证据 hash、工具二进制与三份 work 792 收据 hash 未变。主计划差异仅是历次审查/修复簿记，plan_revision 仍为3。

据此复用 R2 的 51 条定向 Rust 通过、17 布局通过、viewer frozen-lock、独立闭包、777 strict、A/B locked metadata及分别删除两个来源/恢复绿控制；复用 R2 绑定过的 work 792 全程25/25及 fresh/relocated build证据。本轮唯一变化是上面精确格式器输出，不改变这些测试的代码行为、依赖或输入；不声称上述测试在本轮重新执行。完整关闭和来源核验见 [R2报告](../review-t04-r2/review.md)。先前工具包三项内容红、画廊内容金样归因及完整 B 验证的边界不因本次格式关闭而扩张。

## 结论与后续

T-04 当期 AC-01/02、已触及 AC-04 与 AC-05 当期格式范围通过；完整 B 的原 AC 不缩减。本次不勾 T-09，不把 Plan 标为 reviewed。T-04 勾回，Plan 保持 executing 10/17。

T-05 继续遵守 M814-B 外部门；T-08 最终完整规范增量/T-09 终审及 merge-owned 冻结裸 t 与必要触面档均 pending。任何影响实现/依赖/输入的后续改动需按范围重审。未合入、未执行整档、未写 canonical/live ledger、未归档或清树。规范裁量冻结见 [spec-delta-reviewed.md](spec-delta-reviewed.md)。
