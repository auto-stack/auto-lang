# PLAN-743 Phase 4 Proposed Spec Delta（SD-08）

> 性质：**提案**（plan_revision 4）。canonical 落地仅在独立复审 pass 后由
> `/auto-plan:merge` 执行；本计划未写 canonical 行为内容。r1..r3（SD-01..07）为
> 已交付历史。映射 r4 合同 §Phase 4 规范增量与 AC-21..23。

| delta_id | 操作 | docs/specs/... target | before → after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-08 | modify | docs/specs/auto-acc/project.md | 当前owner/本地遮蔽政策 → 明确导入/已知参数局部绑定冲突亦阻止native猜测，引用只读验证记录，证据覆盖不能由subject/hash替代 | QA-01..03 把原保证落实到所有 consumer，保持规范强度，不声称完整 resolver | AC-01/02/03/09/10/15/17/18/21/22/23 |

---

## SD-08 提案正文（docs/specs/auto-acc/project.md，替换"分类政策"条目并补一段）

~~~markdown
- 分类政策（r2 QA-02 / r3 R2-QA-01 / r4 R3-QA-01..03）：变量接收者的方法调用一律
  unknown-receiver（本地类型可声明同名方法，Meter.len/CG.new/Ar.new 实证）；本地
  声明先于宿主——本地 type IO/fn print 遮蔽宿主命名空间/内建推断；导入同名
  （use 引入的符号）与已知参数/let/var 局部绑定同名同样阻止宿主猜测（import_shadow/
  parameter_shadow 反例实证）；dot/管道隐式 self 仅当调用点 enclosing owner 声明了
  该方法才升级（其它 owner 同名或无 owner 保持 unknown）；native 仅限无上述冲突时的
  可证明宿主命名空间（List/IO/process/File）。变量接收者族的语义归属见决定 MD-507
  与 unknown_families，实现期由名字解析/类型检查主体（新建）精确化；有限词法层遇到
  不能确定的作用域一律保持 unknown，不声称完整 resolver。
~~~

~~~markdown
严格门附加保证（r4）：unknown_families 的 resolved 引用只读"通过全部类型/语义校验"
的决定记录（被淘汰或重复 ID 的引用按"未通过校验/不存在"定位拒绝，不二次消费未校验
数据）；resolved 适用性仅由同条 evidence 文件集合覆盖族路径证明——hash 绑定证明
文件新鲜、subject 命名盘点对象，均不能替代审定的证据引用。
~~~

---

## 合规自检

- SD-08 为 modify：收紧保证（不放宽原约束、不声称完整 resolver、不实现优化器）。
- 既有 r2/r3 分类/门控条款保留；本 delta 仅把"本地遮蔽"扩展为"导入/参数/局部绑定
  遮蔽"并封堵 subject/hash 旁路与消费者组合路径。
