# PLAN-842 T-06 验证

工作期冻结：lang `7bbd2cc9b9e0d373ea0182df343ddd75a7d8e4f3`、UI `1ef875b574bad75b4de07232c0103cc63e213bd7`、Man `484d9d2fe0904f13c9aff29ebbebea564bf8eac0`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。本项并非最终交付；T-07 须重验最终版本安装及真实 consumer。

- `use-bigvm` 空 feature 已退出；真实外部旧 feature 约束具名解析失败，更新后的 0.2.0 consumer 在无环境覆盖时实际执行 VM 并验证输出；保留 `use-evaluator` 的无默认特性 check 通过。迁移指南明确旧命令仅属历史、0.2.0 必须移除。
- `host-assembly-consumer` 的独立进程以固定锁分别验证纯 SDK 缺席、语言包无 UI 缺席、链接 UI 后直接 SDK 默认在场、显式 frame provider 17/31 优先；重复装配保留相同 provider；工作线程装配后 join 仍在场并以预定进程码 23 正常结束。六项真实 VM provider 消费测试通过，包括两个活实例 17/31 与 43/59 的快照和单实例覆盖。
- 受控消融仅移除语言包 inventory 登记，实际外部 consumer 在未调用语言包装的直接 SDK 入口缺失 UI 默认、exit 101；恢复原始源码后同一 consumer 再绿。故 R08 所定义“仅已证明可替代的兜底才退出”的条件对这条登记不成立：它是当前正式默认行为所必需，保留现有单份登记与正式 SDK hook，不能将其描述为已退役或创建替代兼容实现。T-09 应将该必要保留及显式组合入口写入 Spec delta；若要求连此行为一并删除，需要另行改变公开宿主合同。
- 生产显式来源权威性、合法受管 SDK 与开发发现由 T-03 正反控验证；七份之前测试生成的旧 fixture 锁仍未跟踪，新宿主 fixture 的锁已随探针提交。真实最终 Full/Core 安装与跨仓固定源由 T-07 重验，独立 review 和 merge 门 pending。
