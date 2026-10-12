# PLAN-741 Phase 4 proposed Spec delta (not canonical)

Review baseline: PLAN-741:r3 @ f6a945db849f57789a9e151402e22b8aa57eb7f8. Current Spec source hashes are in source-manifest.json.

| ID | Operation | Target | Before → after | Rationale | Acceptance |
|---|---|---|---|---|---|
| SD-07 | modify | docs/specs/auto-ac/project.md | 单 deadline + 全出口回收自有进程/reader → 明确启动前建立受控边界，约束不可用时受控拒绝，不能先放行子进程再无声降级为 detach；覆盖早退父进程与后代创建窗口 | 兑现既有 AC-17，修复已观察到的后代存活 | AC-14,17,21 |
| SD-08 | modify | docs/specs/auto-ac/project.md | 暂存/备份回收、恢复失败列路径 → 清理失败也必须携带原失败、实际 OS 错误与所有自有残留路径，准确区分发布是否已提交；用户目录与唯一旧备份不删 | 兑现 §5.6/AC-18，禁止 remove 错误被吞掉 | AC-12,18,22 |

SD-05 块图规则已由本轮独立复审确认，无新增 auto-hir 行为增量。
SD-06 的强保证尚有实现缺口；旧 pass 作为历史保留，不能降为 best effort 来规避本次反例。
当前复审不发布上述两条、不改 canonical project.md 与 live ledger；待修复后的独立 pass 再沉淀。
