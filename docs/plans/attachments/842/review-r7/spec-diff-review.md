# PLAN-842 R7 规范落地差异独立核对

核对人：独立复审子代理 `/root/review842`；计划 revision 2；复用 R7 审查源码基线 lang `a8b4a5bcf53d9be484cc38f028453a4cebbc947f`、UI `bd0e0f38253c5fd0676c8ef729c8c030d372ae82`、Man `9ac33ad871c4e0278b322e9712930ed059029e9c`，down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。本记录核对合入准备中语言工作树的 `docs/specs` 工作区差异，不代表这些文档已提交或合入。

除原 SD 目标外，为消除既有规范与已审退役边界的矛盾，核对了七个附加目标：

- `docs/specs/auto-diagnostics/project.md`
- `docs/specs/auto-lex/project.md`
- `docs/specs/auto-lang/frontend/overview.md`
- `docs/specs/auto-lang/frontend/design/dialect-system.md`
- `docs/specs/auto-lang/ui/design/scenario-dialect.md`
- `docs/specs/auto-ui-dialect/design/full-grammar.md`
- `docs/specs/ui-domain-model/project.md`

结论：当前规范差异与已审 SD-01..06、R01..R16 矩阵及冻结源码一致；上述七项目标是既有合同的现状及历史一致性修订，不新增合同要求。已核对旧公开路径和 UI 端口退役、保留的宿主能力、三态投影、不同受管来源各自冻结收据及 staging 字节和 revision、`parse` 实际定义与上层转发；MPV 和 jet/ark 未测边界如实保留。此前发现的冲突已修正。本次未运行测试，R7 的 merge-owned 门仍待执行。
