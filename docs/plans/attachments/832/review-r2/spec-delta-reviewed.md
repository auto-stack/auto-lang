# PLAN-832 SD-01..06 规范增量审定材料（R1 修复后）

实现基线：Man `ebfc88f`、lang `0fd092bdc`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。canonical Specs 已在 worktree 按 R1 修复更新，`python scripts/spec-index.py` 重建 61 项索引；尚未推进 live `.autoos/specs.json`（merge 受管投影）。

| delta | target | 已写内容 / 审查点 |
|---|---|---|
| SD-01 add | `docs/specs/repository-boundaries/man-three-repository-delivery.md` | 三仓唯一 owner；独立 Core workspace/lock/target 与 full 根 workspace；Core 两仓/full 四仓 fixed source；owner 工具发现；`tcore`；unknown owner 保留；安装/切换/验收及 jet/ark 延期 |
| SD-02 modify | `auto-man/project.md`, `auto-man-core/project.md` | 管理实现与 compat 仓属、Core 分离、轻依赖/资源/测试 owner；R1 未改变这些边界 |
| SD-03 modify | `auto-cli/project.md`, `tool-execution/project.md` | CLI 迁 Man；`crates/auto` 共享源与 `auto-core` 独立 Core target path 复用；std-only 执行设施留语言；owner helper 构建与 client/discovery 角色保持 |
| SD-04 modify | `repository-boundaries/project.md` | 旧 Man 快照、导入/删除/测试 ID/legacy 实况与本地切换，未远端发布；R1 未改保全事实 |
| SD-05 modify | `repository-boundaries/generated-ui-distribution.md` | Man/UI 双二进制与 producer、792 接线、Core/full 安装和失败闭包；R1 保持 full 生成/发行语义 |
| SD-06 modify | `tool-protocol/project.md`, `language-sdk/project.md` | 协议/请求/SDK 单源、Man 外部 consumer，无反向实现依赖或第二类型；独立 Core 仍以固定 lang source 消费 |

R1 对 F01/F02 的反例已消除：无 UI/down fresh Core 构建安装与重定位通过，权限未知 owner 不再被回收。F03 的 CI/discovery 与 F04 的目标仓文档入口也已修复并定向验证。支持面声明仍仅限实测项；jet/ark 延期及 merge-owned 统一门与 [handoff](handoff.md) 一致。
