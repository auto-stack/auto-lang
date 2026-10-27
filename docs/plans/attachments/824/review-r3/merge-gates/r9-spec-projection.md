# PLAN-824 r9 规范投影说明

投影范围：规范源 `docs/specs/auto-av/dynamic-portable-values.md`、`docs/specs/auto-mir/dynamic-cast-profile.md`，及已存在的 AV 项目、缓存、常驻函数和 TestCase 架构/一致性规范；投影仅索引受检动态值、portable 类型/权限重验、F03资源计量与取消、统一能力绑定和验证，完整现行合同以规范文件为准。

实现与验收复审见 [R3](../review.md)、[r9 修复复审](r9-fix-independent-review.md)；共享台账恢复见 [独立复核](ledger-recovery-independent-review.md) 与其字节备份。候选 `51a1db05ff3b234c7f38cb34e29a1b54855a85ff` 经文档-only主干同步与受审 `cf2aa647ab33628f1d639d08ede502c3ba6f8ddc` 的19项 range-diff 等价；恢复提交只改台账。

冻结验证记录：[acceptance](r9-acceptance.log.txt)、[日常](r9-cargo-t.log.txt)、[语料](r9-cargo-tv.log.txt)、[兼容复验](r9-compatibility-isolated.log.txt)。兼容采样期间出现其他工作树编译，环境事实见 [记录](r9-acceptance-environment.md)，不宣称机器全程独占。

投影预期版本103→104，新增 designs P824-DYN/P824-MIR、tests P824-T、reviews P824-R、reports P824-REL；更新现有对应文件条目并保留旧内容/标签。落地、归档、清理和批量回归以计划最终收据为准。
