# PLAN-832 SD-01..06 规范增量审定材料

实现基线：Man `356f2c8186d07bc18567b18179c3cad7bda2d483`、lang 切换 `ba1ce5901d76643a8878f5d0f4663c3ca4904ea5`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`。canonical Specs 已在本 worktree 更新并由 `python scripts/spec-index.py` 重建 61 项索引；尚未推进 live `.autoos/specs.json`（merge 受管投影）。

| delta | target | 已写内容 / 审查点 |
|---|---|---|
| SD-01 add | `docs/specs/repository-boundaries/man-three-repository-delivery.md` | 三仓唯一 owner、fixed commit/profile、安装/发现/机器合同、切换恢复、验收及 jet/ark 经批准的 v0.7 延期 |
| SD-02 modify | `auto-man/project.md`, `auto-man-core/project.md` | 管理实现与 compat 仓属、Core 分离、轻依赖/资源/测试 owner |
| SD-03 modify | `auto-cli/project.md`, `tool-execution/project.md` | CLI 迁 Man；std-only 执行设施留语言，client/discovery 迁 Man，角色与寿命保持 |
| SD-04 modify | `repository-boundaries/project.md` | 旧 Man 快照、导入/删除/测试 ID/legacy 实况与本地切换，未远端发布 |
| SD-05 modify | `repository-boundaries/generated-ui-distribution.md` | Man/UI 双二进制与 producer、792 接线、Core/full 安装和失败闭包 |
| SD-06 modify | `tool-protocol/project.md`, `language-sdk/project.md` | 协议/请求/SDK 单源、Man 外部 consumer，无反向实现依赖或第二类型 |

支持面声明仅限已实测项；jet/ark 经用户正式裁定转 v0.7，未实测不冒称通过。Specs 与 [handoff](handoff.md) 的 AC-03 r3 必需面通过保持一致。
