# PLAN-743 Phase 3 Proposed Spec Delta（SD-06/07）

> 性质：**提案**（plan_revision 3）。canonical 落地仅在独立复审 pass 后由
> `/auto-plan:merge` 执行；本计划未写 canonical 行为内容。r1（SD-01..03）与
> r2（SD-04/05）为已交付历史。映射 r3 合同 §Phase 3 规范增量与 AC-15..20。

| delta_id | 操作 | 目标 | before → after（规则级） | rationale | acceptance |
|---|---|---|---|---|---|
| SD-06 | modify | docs/specs/auto-acc/project.md | 仅声称结构/新鲜/覆盖完整 → 明确同条证据绑定、字段类型受控拒绝（无 traceback）、resolved 族适用决定闭环、当前 owner 分类政策 | R2-QA-01..04 杜绝工具假绿；不声称语义 resolver 已实现 | AC-01/02/03/09/10/13/15/16/17/18 |
| SD-07 | modify | docs/specs/auto-hir/stage-contract.md | 旧 R3 阶段摘要（"effect 只能收窄"）与新正文并存 → 摘要与效果两分、runtime trap/X9 完全一致 | R2-QA-05 补遗漏；不实施优化器 | AC-04/05/11/19 |

---

## SD-06 提案正文（docs/specs/auto-acc/project.md，替换"能力现状"节首段与"分类政策"条目）

~~~markdown
清单可信度以盘点工具严格门为准：`scripts/acc_inventory.py --check --require-decisions`
要求人工层（决定 + unknown_families）字段与元素类型合法（错误类型受控拒绝并定位
记录，无 traceback）、每条决定的证据文件均有同条绑定（仓内非扫描输入证据同样受控，
证据变化即 stale）、绑定新鲜，并对全部受管输入与全部 unknown 候选双向闭环；
resolved 族必须引用存在且适用的决定（kind=unknown-resolution、conclusion=resolved、
证据覆盖族所在路径），open 族必须带 owner/探针/所在工作包；scan-only（无人工层）
明示非完成态。
~~~

~~~markdown
- 分类政策（r2 QA-02 / r3 R2-QA-01）：变量接收者的方法调用一律 unknown-receiver
  （本地类型可声明同名方法，Meter.len/CG.new/Ar.new 实证）；本地声明先于宿主——
  本地 type IO/fn print 遮蔽宿主命名空间/内建推断；dot/管道隐式 self 仅当调用点
  enclosing owner 声明了该方法才升级（其它 owner 同名或无 owner 保持 unknown）；
  native 仅限无本地冲突时的可证明宿主命名空间（List/IO/process/File）。变量接收者族
  的语义归属见决定 MD-507 与 unknown_families，实现期由名字解析/类型检查主体
  （新建）精确化。
~~~

## SD-07 提案正文（docs/specs/auto-hir/stage-contract.md，替换阶段表摘要行中的 R3 规则）

~~~markdown
规则：R1 阶段身份随行、禁止跨阶段取用；R2 来源映射自 S1 起随行；
R3 效果分析与效果语义两分——分析自保守侧出发（未知 effect 按 observable 处理，
向 pure 精化须携带证明），效果语义保持对任何 pass 双向禁止（不得引入/消除/重排
可观察效果，含 trap 与其前驱 observable 的顺序）；运行期 trap 与编译期诊断界限及
反例 X9 见"Pass 合同模板"节；R4 失败=显式诊断；R5 显式 Dynamic 与推导失败不同态；
R6 多模块装配不改变 verify 全或无语义。
~~~

---

## 合规自检

- SD-06/07 均为 modify：不替换组件、不降低 r1/r2 已交付约束，仅消除摘要/正文冲突并
  补齐工具保证声明；实现状态边界不变（无 pass/优化器/effect lattice 实现）。
- CLI 命名：报告/候选文档同步 auto-ac-prototype（741 r2 实名）；既有 HIR existing 与
  待建源码状态保留，不虚报 741 Phase 3 修复进度。
- 生命周期：P743-3 活动指针随重开恢复可解析；P743-4 归档指针待本次 merge 对账；
  归档态链接以 ../../reports/... 修正（模拟归档路径可解析）。
