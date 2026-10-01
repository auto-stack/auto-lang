# PLAN-719 r2 · 历史文章来源核验

研究日期：2026-10-01。核验范围为本地项目设计文档、代码与提交、归档计划；未执行历史测试或跨仓运行验证。

## 版本锚

- auto-lang: `731aa42bcdff59297db6f35659da40d24b3f9660`；origin=`git@github.com:auto-stack/auto-lang.git`。
- auto-os: `73d02b50f535543bd635c6f6f49dad6835efff45`；origin=`git@github.com:auto-stack/auto-os.git`。
- auto-ai: `5a50a55844d7aa3523b593f21ba0fb03d18eac48`；origin=`git@github.com:auto-stack/auto-ai.git`。
- auto-musk: `22f06ba1808a3cf270795836d2528ca6f5a2d8f4`；origin=`git@github.com:auto-stack/auto-musk.git`。

## 叙事主张与核验结果

| 主题 | 主张/边界 | 依据 |
|---|---|---|
| OS 起点 | 可追溯至 2025-12-22 的 OS/Task/lifetime 设计记录，不宣称为项目成立日或 OS 交付日 | auto-lang 84cdb6dc1；git log --follow docs/design/raw/os.md |
| 虚拟桌面 | 2026-08-26 WM-as-App/宿主合成边界；解释当前理念而不宣称全平台支持 | virtual-desktop.md；caa802981 |
| Linux | 2026-09-01 Stage 1 嵌套合成与首帧；live attach/完整发行版不是本阶段成果 | 509 归档计划；auto-cosmic project.md；7dded6b8e |
| 产品分工 | 2026-09-07 产品伞形、shell 表面随迁；框架留 auto-lang | auto-os f05bcec/1dcabf0；Design 01；lang 579/590 |
| 运行形态 | 编译壳/协议/RQHost 有实现；当前 app launch 主要 VM，不能写成解释态全面退役 | desktop-shell-a2r.md；desktop-app-launch.md；apps.manifest；a574e31 |
| LaOS/OS over OS/未来 | 连续关系、自有内核/native 与人本方向来自用户本次说明，不虚构最早提出 OS over OS 的日期 | 本会话 2026-10-01 用户概述 |
| AI 拆分 | 2026-06-17 基础设施、06-18 canonical 类型/职责重构 | auto-ai f95d6ec/3b4976f；musk Design 002 |
| Role | 2026-07-01 rename，运行配置逐步完善；归档日期不当实施起始日 | 73aecc6；004 归档计划 |
| 编排 | 07-14 设计；07-16 已有完成提交；旧 Workflow 于 08-04 删除 | orchestration-down-design.md；2aa3700/2bb84c9；017 计划 |
| Auto 化 | 08-07 时点曾完成转译版 daemon 全链路验证；不把历史实跑当本次当前回归 | 025 计划；afc803a |
| 使用行为 | 回合事件、content/details 分层、压缩/窗口/溢出恢复逐项补齐 | 00a148b/ccf24fb；026/028/031 归档计划 |
| 命令/路由 | ash-first 失败分类；09-22 显式候选链且保留 tier | 033/034 归档与对应 Specs；997c2d7 |
| Plan 工作流 | 早期 plan-dev 多阶段；当前职业分工与人工门以 plan-flow Spec 为准 | musk 008 设计、modules/plan-flow.md；5e6ea76/12dff96 |

## 文档过时信息处理

- auto-ai src/lib.at 仍含已弃用 Workflow 的历史注释；当前 exports 使用 orchestration，workflow.at 与 rust-ref/workflow.rs 均不存在。文章依据实现与删除提交，不沿用旧注释。
- musk README 的同一 plan-dev 四阶段描述滞后于 2026-09 的 plan-flow Spec，文章以当前 Spec 的固定职业为准。
- 017 计划保存“删除尚未完成”的调查历史，同时顶部/完成记录说明已完成。文章按时间区分，不截取历史段当现状。
- 年月采用设计明确日期与提交/落地证据；文档重命名、计划归档不当作功能首次实现。

分支核对：本地 auto-lang 为 master，auto-os/auto-ai/auto-musk 为 main；blob 引用与这些现有分支命名一致，远端可访问性留在落地门禁核验。

## 引用链接核验清单

下列 commit 在本地仓可解析，blob 文件为真实 git 跟踪源。部署前 T-01/T-05 还须验证公开链接可访问及 remote ref；本次不声称已经完成远端网页连通性验证。

| 引用 | 本地核验 |
|---|---|
| [auto-ai: docs/orchestration-down-design.md](https://github.com/auto-stack/auto-ai/blob/main/docs/orchestration-down-design.md) | source SHA256=93ca40287576 |
| [auto-ai: docs/plans/archive/004-agent-roles-profession-upgrade.md](https://github.com/auto-stack/auto-ai/blob/main/docs/plans/archive/004-agent-roles-profession-upgrade.md) | source SHA256=7500cb56b69d |
| [auto-ai: docs/plans/archive/017-workflow-decommission.md](https://github.com/auto-stack/auto-ai/blob/main/docs/plans/archive/017-workflow-decommission.md) | source SHA256=574d3a75b982 |
| [auto-ai: docs/plans/archive/025-daemon-autoization.md](https://github.com/auto-stack/auto-ai/blob/main/docs/plans/archive/025-daemon-autoization.md) | source SHA256=b47d7611a7cf |
| [auto-ai: docs/plans/archive/031-pi-parity-compaction-phase2.md](https://github.com/auto-stack/auto-ai/blob/main/docs/plans/archive/031-pi-parity-compaction-phase2.md) | source SHA256=477f55ba32ba |
| [auto-ai: docs/specs/auto-ai-cli/shell-execution.md](https://github.com/auto-stack/auto-ai/blob/main/docs/specs/auto-ai-cli/shell-execution.md) | source SHA256=d80ed0505246 |
| [auto-ai: docs/specs/auto-ai/role-model-binding.md](https://github.com/auto-stack/auto-ai/blob/main/docs/specs/auto-ai/role-model-binding.md) | source SHA256=5812627ca5a9 |
| [auto-ai: 00a148b2d](https://github.com/auto-stack/auto-ai/commit/00a148b2d6d2e9162e9174ebc505f50622ae474c) | commit=00a148b2d6d2e9162e9174ebc505f50622ae474c; date=2026-08-23 |
| [auto-ai: 2aa3700f9](https://github.com/auto-stack/auto-ai/commit/2aa3700f9a38421d4e4280c84114a08c0b9719de) | commit=2aa3700f9a38421d4e4280c84114a08c0b9719de; date=2026-07-16 |
| [auto-ai: 2bb84c9ed](https://github.com/auto-stack/auto-ai/commit/2bb84c9ed26f82b41f796d496728f8cb7fe1338b) | commit=2bb84c9ed26f82b41f796d496728f8cb7fe1338b; date=2026-08-04 |
| [auto-ai: 3b4976f78](https://github.com/auto-stack/auto-ai/commit/3b4976f780ae4272e14942045a93fb21592954b0) | commit=3b4976f780ae4272e14942045a93fb21592954b0; date=2026-06-18 |
| [auto-ai: 73aecc66a](https://github.com/auto-stack/auto-ai/commit/73aecc66adbd1c1ef5c1521e49277a22158b8953) | commit=73aecc66adbd1c1ef5c1521e49277a22158b8953; date=2026-07-01 |
| [auto-ai: 997c2d76e](https://github.com/auto-stack/auto-ai/commit/997c2d76ee224d640f41853d41e8fc0e818a64ce) | commit=997c2d76ee224d640f41853d41e8fc0e818a64ce; date=2026-09-22 |
| [auto-ai: afc803a7c](https://github.com/auto-stack/auto-ai/commit/afc803a7c4123466390c2edffb094e01ecd3894c) | commit=afc803a7c4123466390c2edffb094e01ecd3894c; date=2026-08-07 |
| [auto-ai: ccf24fbe5](https://github.com/auto-stack/auto-ai/commit/ccf24fbe554336ead472053131a0dbf6d914eaf3) | commit=ccf24fbe554336ead472053131a0dbf6d914eaf3; date=2026-08-24 |
| [auto-ai: f95d6ec5b](https://github.com/auto-stack/auto-ai/commit/f95d6ec5b7e154b79a0ffed3d0a0169b5617d136) | commit=f95d6ec5b7e154b79a0ffed3d0a0169b5617d136; date=2026-06-17 |
| [auto-lang: docs/design/autoui/desktop-shell-a2r.md](https://github.com/auto-stack/auto-lang/blob/master/docs/design/autoui/desktop-shell-a2r.md) | source SHA256=69b14d1fb21d |
| [auto-lang: docs/design/autoui/virtual-desktop.md](https://github.com/auto-stack/auto-lang/blob/master/docs/design/autoui/virtual-desktop.md) | source SHA256=1397105dc212 |
| [auto-lang: docs/design/strategy/harmonyos-ecosystem-strategy.md](https://github.com/auto-stack/auto-lang/blob/master/docs/design/strategy/harmonyos-ecosystem-strategy.md) | source SHA256=7eb05307c1de |
| [auto-lang: docs/plans/archive/509-smithay-host-stage1.md](https://github.com/auto-stack/auto-lang/blob/master/docs/plans/archive/509-smithay-host-stage1.md) | source SHA256=fef5af02baea |
| [auto-lang: docs/specs/auto-cosmic/project.md](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/auto-cosmic/project.md) | source SHA256=60386c7ebac3 |
| [auto-lang: 84cdb6dc1](https://github.com/auto-stack/auto-lang/commit/84cdb6dc161a3e5fae8eec1a88615edb84b67e28) | commit=84cdb6dc161a3e5fae8eec1a88615edb84b67e28; date=2025-12-22 |
| [auto-lang: caa802981](https://github.com/auto-stack/auto-lang/commit/caa8029817002688818ed9661d4544c6dcb586ac) | commit=caa8029817002688818ed9661d4544c6dcb586ac; date=2026-08-26 |
| [auto-musk: docs/designs/002-auto-forge-ai-capability-split.md](https://github.com/auto-stack/auto-musk/blob/main/docs/designs/002-auto-forge-ai-capability-split.md) | source SHA256=2cfe1fbc87bc |
| [auto-musk: docs/designs/008-auto-plan.md](https://github.com/auto-stack/auto-musk/blob/main/docs/designs/008-auto-plan.md) | source SHA256=fde59ec0b75c |
| [auto-musk: docs/specs/modules/plan-flow.md](https://github.com/auto-stack/auto-musk/blob/main/docs/specs/modules/plan-flow.md) | source SHA256=2a904c697eb3 |
| [auto-musk: 12dff96bd](https://github.com/auto-stack/auto-musk/commit/12dff96bd3c2d5f7aad8dfe1528c41d689032141) | commit=12dff96bd3c2d5f7aad8dfe1528c41d689032141; date=2026-09-29 |
| [auto-os: apps.manifest](https://github.com/auto-stack/auto-os/blob/main/apps.manifest) | source SHA256=41c7ed73b53f |
| [auto-os: docs/design/01-stage-b-desktop-migration.md](https://github.com/auto-stack/auto-os/blob/main/docs/design/01-stage-b-desktop-migration.md) | source SHA256=2b246ade1b25 |
| [auto-os: docs/specs/shell/desktop-app-launch.md](https://github.com/auto-stack/auto-os/blob/main/docs/specs/shell/desktop-app-launch.md) | source SHA256=e40d283f6f5c |
| [auto-os: 1dcabf0f8](https://github.com/auto-stack/auto-os/commit/1dcabf0f874a84da3ba1f5be743b8cd066c94cd5) | commit=1dcabf0f874a84da3ba1f5be743b8cd066c94cd5; date=2026-09-07 |
| [auto-os: a574e3104](https://github.com/auto-stack/auto-os/commit/a574e3104096836e8e61a8eb2581c87416160a68) | commit=a574e3104096836e8e61a8eb2581c87416160a68; date=2026-09-24 |
| [auto-os: f05bcecf2](https://github.com/auto-stack/auto-os/commit/f05bcecf2349cf0041470b42364ec2736c5a9bf5) | commit=f05bcecf2349cf0041470b42364ec2736c5a9bf5; date=2026-09-07 |

## 文章稿与计划落地路径

- 719-autoos-history.zh.md / .en.md → website/zh/articles/autoos-history.md / website/articles/autoos-history.md（新）。
- 719-auto-ai-history.zh.md / .en.md → website/zh/articles/auto-ai-history.md / website/articles/auto-ai-history.md（新）。
- 四份稿均为普通 VitePress Markdown 正文；从对应总览链接、两篇互链并返回总览。
