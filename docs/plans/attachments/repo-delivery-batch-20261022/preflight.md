# 831/832规划合同预检

2026-10-22 / stage: new / outcome: **pass（仅实施合同）**。两份均r1、drafting、current_step 0，所有任务未执行；可交work按已明确任务门开展，不代表未来实现或验收已通过。

| Plan | 任务 / work-review-merge | AC / V / SD |
|---|---|---|
| 831 | 12 / 9-1-2 | 8 / 6 / 7 |
| 832 | 12 / 9-1-2 | 8 / 6 / 6 |

已核对：

- 两个ID在active/archive唯一，中央脚本串行取号831/832；计数器833，保留其他会话已分配829/830，无覆盖旧计划。
- 编号0–10章节、元数据、任务依赖DAG、owner_stage、AC/Case/SD与frontmatter目标一致；修改Spec均存在，新增Spec均为新路径。
- 阶段DAG无环：当前准备独立；UI候选等U815；共享切源与Man候选等U816；832完整UI/三仓组合等831实际发布，不等归档清树。
- T-01不隐性卡在未来B，当前基线与未知项归属可先交付，实际重采/probe归后继已具名任务。
- UI/API生成对Man声明的剩余边在831内单源归中立轻层；共享std-only执行设施留语言基础，避免按逻辑域物理搬进Man形成反向仓依赖。
- 两份包含独立clone/第三方工程/安装、fresh/locked/重定位、固定版本/类型/source、资源/producer与升级/失败恢复；候选导入不能代实际发布。
- 旧仓784证据只历史引用，实施先重采/保全/恢复；本地默认分支推进在final review及冻结门后，不清空/强移历史。远端push/更名/registry未隐含授权，也不作为本地独立交付前置。
- 跨仓原测试ID/各档与CI覆盖有owner，不因从根成员移除而漏测；共享窗口/整档串行，独立scope审查修复可并行；R10–R12保留。
- 本轮完整新合同及旧设计/路线新增行共62个本地链接可解析；所写tracked文档diff空白检查通过。

来源与hash见[输入](inputs.json)，结构/依赖检查和输出hash见[机器收据](preflight.json)。Design37 §8.13与路线v7已登记；既有design索引行已经链接Design37，本轮该索引含其他会话WIP，未修改/提交其内容。815/816原合同、其他计划/未跟踪文件均未由本轮写入。

本轮只规划，不改Rust/测试/canonical Specs/ledger、不建agent/工作树、不写旧仓/远端，Category A未运行Cargo/docs_gen。旧仓git refs/WIP及真实环境尚待work刷新，不能把本预检作为独立仓已交付的收据。
