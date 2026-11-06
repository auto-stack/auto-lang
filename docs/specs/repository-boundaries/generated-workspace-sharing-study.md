# 生成 Workspace 共享研究（PLAN-942，2026-11-06）

分类：repository-boundaries / 研究结论（**未上线**——本研究不改变任何生产默认）。
来源：[PLAN-942（归档）](../../plans/archive/942-generated-workspace-incremental-sharing.md)；
设计背景 [Design 53 §8](../../design/53-independent-build-and-generated-cargo-governance.md)。

## 1. 研究问题与结论

问题：在 910 已交付「受管共享 target + flags 单源 + 成员身份/收据」的基础上，
把各应用独立的项目本地生成 workspace 换成**同兼容族集中 workspace/锁**（候选 B），
还能取得多少**额外**空间收益？

**结论：conditional——增量收益未获证明。**

- 主口径（按卷+file-ID 去重的可归属分配字节 total_alloc）：
  - UI 域（002-counter + 013-todo，debug）：ΔAB(S4) = **−286,600 B**（B 反而更大；
    r=−0.0023%），|Δ| < E_delta。
  - Web 域累计（auto-chat + auto-database，release）：ΔAB(S4) = **+557,736 B**
    （r=+0.0040%），超 E_delta 仅 12,008 B（基数 12.9GiB 的 0.00009%）且该分量
    **不可归因**（见 §3），按 V-14 边际正差额不得判 go。
- ε_meter=0（三臂静止双读全等）；E_delta=545,728 B（两次独立匹配重建
  545,728/471,552 取保守 max）。
- 不称「已证明零收益」（差值在构建波动量级、跨域符号相反）；不因空间目的
  推进生产切换；910 现状（共享 target + 各应用独立生成 ws）已捕获可用节省。
- 结构性事实：成员源码/发布件在两臂字节同构（同 id8/同输入），根清单/锁为
  KB 级；A 臂因锁版本分叉（9 个传递包）多付的 target 代价仅 ~0.2MiB（本样本集）。

## 2. 方法与证据（可复核要点）

- 冻结来源：auto-lang@921bfa518 / auto-ui@02316271 / auto-web@bf72baab /
  auto-man@03bb65ef / auto-down@58ea33f（git archive 副本；工具 auto.exe
  f839af19… / auto-ui-tool.exe 5af1d690… 哈希冻结）。
- 样本：UI=auto-ui/examples/ui/002-counter、013-todo（debug）；Web=auto-chat、
  auto-database（release；auto-blog 因生成后端在冻结 producer 下编译失败
  E0308/E0382 具名替代）。三臂同一 canonical 输入路径（member id8 同源）。
- 臂设计：A=各应用独立 ws+臂内共享 target（保 910 语义）；B=单一集中 ws/锁；
  C=独立 ws 逐阶段采用 B 同阶段锁文本（缓存独立）。阶段 S0–S8（冷/加入/稳态/
  无改重跑/编辑/共享依赖变更/临时成员/恢复）。
- 计量：FileStandardInfo AllocationSize + (st_dev, st_ino) 去重；硬链接/嵌套根
  负控；V-09/11/12/14 机制控制 19/19。
- 行为/边界：V-02 同 mtime 内容变更被内容新鲜度捕获；V-05/06/07 探针 7/7
  （并发/恢复/陈旧 lease/无 UI 源服务环境/HTTP 行为）；V-13 来源负控 4/4。

## 3. 归因限制

- UI 域：C 完全采纳 B 解析（共享包 version-diff=0、S1 锁逐字相等、S2..S4 稳定）
  → UI 分量可归因（为负）。
- Web 域：C 未采纳 B 的 9+ 传递依赖选择（cfg_aliases/toml_edit/windows/
  roxmltree/winnow 等）→ Web ΔAC/ΔCB 仅描述性，不解释因果。

## 4. 具名生产前置（后继关闭条件）

- **P1（交付打包缺口，owner: auto-lang 交付 vendor 链）**：交付 bundle 未收录
  `experimental/ace/crates/semantics-contract`（auto-lang-core 的 path 依赖），
  消费侧 `--locked` 构建红。在 producer 921bfa518 下，930 原 oracle fixture 与
  真实样本同红（生成/主题腿 PASS）——预存生产回归，先于本研究的候选导出缺口。
  关闭：vendor 该 crate 或重组依赖，随后重跑 930 driver V01/V02 与候选导出链。
- **P2（Web 无生产导出入口，owner: auto-web/auto-generation-core）**：Web 服务
  成员经 `workspace=true` 继承开发根路径依赖，成员级移根无法独立 locked 构建。
  关闭：为 Web 定义交付入口（对齐 PLAN-792 的 UI 交付语义）。

## 5. 迁移估计与后继边界

- 迁移估计（假设列明，非实测迁移）：ctx8 身份（producer+flags）不变 → 集中 ws
  复用既有 910 共享 target；member id8 随 canonical 项目路径不变 → 重组而非
  重建，净新增 ~0。前提是 P1/P2 关闭且未来出现锁闭包显著分叉的多应用负载
  （本样本集闭包几乎同构，集中无可节省的大头）。
- 后继合同（若重启）：cohort 字段（producer 检出身份/工具链/flags/UI+Web 来源
  组合）、根锁/收据新鲜度、坏成员隔离与锁序、发现/导出/回收准入——见
  Design 53 §8 与归档计划 §5.4；均不在 942 上线。

## 6. 状态

- 910 默认**保持不变**（共享 target + 各应用独立生成 ws）。
- 本研究零生产代码改动；证据摘要与命令/退出码/关键数字见归档 PLAN-942 §8–9。
