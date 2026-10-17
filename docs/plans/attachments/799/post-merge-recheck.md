# PLAN-799 交付后续查

2026-10-17；主检出HEAD `6d9d28b240c088609ac5ce2ac9b3facee9ba723e`；补正提交 `a9a1e3943fa8a85393873edc34c07473df360faf`。
本轮只读核对已提交对象、规范/账本/引用及证据，不重开archived计划，不修改实现，不运行Category A禁止的cargo测试或docs_gen。

## 上轮缺口闭合

- F799-CLOSEOUT-01 closed：git ls-tree HEAD仅含archive/799，旧活动路径不再受管；对应路径没有未提交删除。
- F799-CLOSEOUT-02 closed：r1/r2/r3基线与driver/metadata/日志已以.json.txt/.log.txt入库，可从git show HEAD:<path>读取；6份JSON均能解析；driver正式收据25/25全true；R3冻结spec-delta.patch从提交读取的SHA256仍等于baseline中的62834ddfc34b92d650272c0f755396784a1eafd267fbd452605287cf282cab6f。此前本机独有证据问题已解决。

## F799-CLOSEOUT-03 / P3：归档后的文档导航尚未适配

归档文件 `docs/plans/archive/799-frontend-ast-model-extraction.md` 有7个Markdown相对链接仍以 `attachments/...` 或 `reports/...` 开头。Markdown从archive/目录解析，落到不存在的archive/attachments和archive/reports。
7项包括旧臂inputs.json、并行合同、R1/R2/R3报告及R2/R3 baseline；链接目标本体均已入库，实际都在docs/plans/attachments或docs/plans/reports。
独立验证：7个链接逐一加 `../` 后均指向Git受管文件。修正这些链接的相对层级即可，无需重新创建材料或变更归档状态。

另有同类说明小错：R3 REVIEW.md第11行仍提到本目录format.log.txt，但本目录该文件未入库/不存在（检查成功无输出，结果已在正文记录）；可删去该文件引用，保留命令与exit0。第26行日志范围写成check-1.log.txt..check-6.log，末项应与实际check-6.log.txt一致。冻结的spec原文副本中的相对链接仅按原来源上下文解释，不列作新的导航缺陷。
以上为低优先级文档可用性问题，不推翻既有代码验收结论，也不把零输出格式检查误判为测试失败。

## 其余核对

- 补正提交及后续提交未改AST实现；此前已核的代码身份/兼容性结论继续有效。
- auto-ast项目卡、plans.md、全局INDEX已收录；INDEX的0是子模块卡数量，不是计划数，未当作缺陷。
- ledger P799-M/D1/T/R3共4项，ID无重复；归档状态保持archived、8/8，已清理工作树不重建。

结论：上轮两项交付收尾缺口确认关闭；未发现新增代码问题。剩余仅上述P3文档导航/引用修饰，原有归档终态保持不变。
