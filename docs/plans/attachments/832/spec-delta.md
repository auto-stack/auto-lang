# PLAN-832 SD-01..06 规范增量审定材料（merge 门禁修复后）

实现基线：Man `ceb8dfdd469b2dec1f2490aae1126b0b3303e35e`；lang rebase 候选 `8cc004f837844332899949e55e26939c6104b468`（31/31 range-diff 等价）；UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。canonical Specs 已在 worktree 按 R1/R2 与 merge blueprint 修复更新；live `.autoos/specs.json` 仍归 merge 受管投影。

| delta | target | 已写内容 / 审查点 |
|---|---|---|
| SD-01 add | `docs/specs/repository-boundaries/man-three-repository-delivery.md` | 三仓 owner；独立 Core/full；fixed source；开发 `current_exe` 布局与显式 env/安装邻接发现；blueprint 占位根不遮蔽真实库；`tcore`；unknown owner；backup marker 恢复与严格校验；r3 延期 |
| SD-02 modify | `auto-man/project.md`, `auto-man-core/project.md` | 管理实现与 compat 仓属、Core 分离、轻依赖/资源/测试 owner；R2 未改变 |
| SD-03 modify | `auto-cli/project.md`, `tool-execution/project.md` | CLI 迁 Man、共享源独立 Core、真实直接 CLI 默认发现与无注入回归；blueprint 开发/安装根统一生产与测试；std-only 执行设施留语言 |
| SD-04 modify | `repository-boundaries/project.md` | 旧 Man 快照、导入/删除/legacy 实况与本地切换；R2 未改保全事实 |
| SD-05 modify | `repository-boundaries/generated-ui-distribution.md` | Man/UI 双二进制、Core/full 安装和失败闭包；真实 backup marker 按文档处置后 verify/旧命令，陌生 extra 仍拒绝 |
| SD-06 modify | `tool-protocol/project.md`, `language-sdk/project.md` | 协议/请求/SDK 单源、Man 外部 consumer，无反向实现依赖或第二类型 |

R1 的 F01/F02 与 R2 的 F03/F04 均有实际闭环证据；连续两次 final 非通过的[覆盖诊断](repair-r2/diagnosis.md)与调整已验证。jet/ark 延期与 merge-owned 统一门不变，详见 [handoff](handoff.md)。
