---
plan_id: PLAN-775
status: executing
feature_name: v06-main-convergence
plan_revision: 2
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [auto-lang, auto-down, auto-os, app-repositories]
current_step: 1
total_steps: 11
---

# [PLAN-775] v0.6 在途收口与生态主线汇合

## 变更摘要

先完成或如实保留所有已开展的 v0.6 工作，再从各仓最新发布主线创建普通合并候选，吸收已验收的 v0.6 提交和本地有效成果。AutoLang、AutoDown、AutoOS、AutoMusk 与实际有 v0.6 增量的应用仓成为新的日常开发主线；其他相关仓逐一给出同步、保留或无需合并的证据。

正式 release tags 和已推送分支不改写。使用普通 merge 和 fast-forward，不 rebase、squash 或 force-push。跨仓按依赖顺序先交付子仓，再更新 AutoOS gitlink。

## 目标

1. 对 GitHub 上 37 个 AutoLang 相关仓逐一确定发布主线、开发独有提交、本地 WIP 与处理结果。
2. 按各自原 Plan/验收完成已开展的 v0.6 开发；未达到 required 条件时保持失败或待办，不能把草案或弱证据写成完成。
3. 把 v0.5 共同基点之后仍有效的发布线改动与 v0.6 开发成果汇入以 v0.5 发布主线为第一父线的候选；让 main/master 成为 v0.6 后续开发目标。
4. 保留 Plan 身份、Spec、测试、应用源和固定 SHA 子仓关系的可追溯性；每仓独立复审并通过自己的门禁。

非目标：重命名仓库既有的 main/master；改写已发布历史；删除恢复分支、tag、未知机器上的 WIP；启动尚未实施的 Plan 769–771 特性。

## 架构方案

以各仓最新远端发布默认分支为候选第一父线，以普通 merge 吸收已复审的开发源；有效的本地唯一提交通过可追溯合并或 cherry-pick 纳入，不丢原始引用。先完成 AutoLang/AutoDown 与应用代码，再迁 AutoOS 中 v0.5 对 Tetris、Klondike 的源增量，最后提交 AutoOS 父仓并固定可远端获取的 gitlink SHA。

对子仓没有开发独有提交的情况，只快进到远端发布主线或保留无需变更；不能为了形式制造空 merge。远端不提供 v0.6-dev 或本地指向 Gitee/Codeup 的仓库先核实来源与内容，再决定同步方式，不擅自改远端配置。

## 需求分析与背景调查

用户已于 2026-10-12 确认执行。本计划依据 `docs/reports/v06-main-convergence-20261012/README.md`、其中固定的 GitHub refs/compare/本地工作树证据，以及各相关仓现行 AGENTS 和应用 Plan。当前审计覆盖 GitHub `auto-stack` 的 38 个仓，排除语料仓 `daizhigev20` 后有 37 个相关仓；另核对本机 27 个 Git checkout。最终发布前必须刷新每个将合入仓的远端头。

关键已知条件：AutoLang v0.5.0 tag 为 `e05e2d3d7`，发布 master `38e4c7c5f`，本地 v0.6-dev 原头 `a45ff543e`，远端 v0.6-dev `94173fe2c`；开发分支已推送，原提交保持不动。本地 master 唯一文档提交 `168b56923`、AutoMusk main 的两条本地规约提交、恢复分支和现有工作树均须保全。Plan 768 在原树完成 work、待独立 review；Plan 742/744、Notes、Blog、Launcher、Reader、Weather、Explorer 的实际完成情况见报告，必须回到原合同核证。Reader 三个 required 目标保持不变。

Plan 编号联合表：

| 身份 | 汇合后的编号 | 处理 |
|---|---:|---|
| v0.5 发布线 Plan 767–769 | 767–769 | 保留发布线原义与历史 |
| v0.6-dev AC 标量计划（原 767） | 772 | 归档计划迁名；原附件、分支名和提交 SHA 保留作审计别名 |
| v0.6-dev AutoDown TestCase 计划（原 768） | 773 | 现行计划迁名；原附件路径、分支名和提交 SHA 保留作审计别名 |
| v0.6-dev AC 数据资源计划（原 769） | 774 | 草案迁名；尚未开始的工作仍保持 drafting |
| 本汇合计划 | 775 | 保持不变；`.next-id` 为 776 |

开发线 770/771 保持草案身份。历史 commit message、远端分支名、冻结附件路径与内容不改写；本计划及迁移后的 canonical 计划编号是现行身份映射。

## 详细设计

先在 master 上正式登记本计划，再在 `D:/autostack/.wt/lang-775/auto-lang` 建立 `plan-775-dev` 权威工作树，并在同组创建确有需要的兄弟仓工作树。每个原在途 Plan/应用树只设一个实现写入者；现有 WIP 和 ignored 数据须先核验、保全，不能用 stash/reset、清理或复制覆盖处理。工作树内禁止 junction/symlink。

按报告中逐仓表执行。语言层重点解决 `.autoos/specs.json`、`trans/rust.rs`、`ui_gen/vue.rs`、`auto-man/api_gen.rs`、测试收据、Plan 编号、debt ledger、roadmap、Spec 与目录整理冲突。派生 INDEX 从合并后的 canonical 文档重建。AutoOS 需保留 v0.5 对 `apps/036-tetris/src/front/app.at`、`apps/037-klondike/src/front/app.at` 的功能更新；`apps/039-syslog` 更新也须保留。产品仓 gitlink 转换前，先在产品仓合入并验证源文件增量。

每个仓的代码、资料、测试和复审结果绑定 commit SHA。测试只按该仓 AGENTS 和原 Plan 的 required 门执行；变更和旧预存红分开记录。跨仓发布按“依赖仓可从远端获取 → 父仓固定 SHA”推进，任何子仓 push 失败都阻断父 gitlink 更新。

## 测试设计

- 结构核验：所有接受的源提交、本地独有提交及发布主线均能从最终候选祖先追溯；v0.5 tags 与原始 v0.6-dev refs 不变；无历史改写。
- AutoLang：先核对 Plan 742/744/768 原 AC，按触面执行 `cargo check -p auto-lang`、`cargo t` 及 `cargo tv`/`cargo tt`/`cargo tb`/`cargo tu`/作用域 `cargo taa`；review 与 fold 前按 AGENTS 再跑所需门禁。批量 `cargo tf` 由主检出依到期规则单实例执行。
- AutoDown/应用：沿用各自合同的 parser、协议和双端/原型验证；Blog 覆盖等待期、失败保存、恢复与迁移负例；Reader 覆盖混排滚动恢复、10 MiB 成功导入、原生 VM UI；Notes/Launcher/Weather/Explorer 用各自原门禁复核。
- AutoOS：确认 Tetris/Klondike 发布线源增量和产品实现同时存在；检查 manifest、desktop 启动与所有 fixed-SHA 子仓能从新鲜 clone 获取。
- 独立复审：逐条执行 AC-01–08、对照每仓最终 HEAD、检查工作树/未提交/债务/延期/workaround；审查失败回到 work，不创建合并发布提交。

## 验收标准

| ID | 结果 |
|---|---|
| AC-01 | 37 个相关仓的远端头、本地分支、独有提交、WIP、tag、工作树与处置均有 SHA 证据；调查范围外机器不宣称已证明无 WIP。 |
| AC-02 | Plan 742/744/768 及 Notes、Blog、Launcher、Reader、Weather、Explorer 等已开展工作依原 required 合同完成或明确阻塞；未开工草案仍是草案。 |
| AC-03 | 候选保留发布主线第一父链，接收源均可追溯；v0.5 tags 和原开发引用未变，无 rebase/squash/force。 |
| AC-04 | AutoLang 代码/目录/Spec/Plan 身份冲突按语义解决；现行 Plan ID 唯一，旧编号有别名，派生索引从 canonical source 生成。 |
| AC-05 | v0.5 应用源增量与 v0.6 产品实现均保留，子仓先于父 gitlink 发布，gitlink SHA 可远端获取。 |
| AC-06 | 独立 review 绑定最终 revision 和每仓 HEAD，通过所有 required AC，并登记剩余债务与验证边界。 |
| AC-07 | 每个有变化的仓通过其门禁并普通推送/快进；没有变化的仓有“已同步/无需合并”证据。 |
| AC-08 | main/master 成为新 v0.6 日常开发目标；Plan/spec/ledger/index/归档和工作树清理收据正确，旧恢复引用保留。 |

## 执行步骤

| Task | 阶段 | 操作与完成证据 | 状态 |
|---|---|---|---|
| T-01 | work | 保存在途 WIP、调查证据和本地唯一提交；联合核对 plan ID；master 上由 `scripts/new-plan.sh v06-main-convergence` 创建本计划。 | [x] 2026-10-13：WIP/evidence 固定为 v0.6-dev 提交 `2e9777b19`；master 合入最新 origin/master（merge `7a3047f79`，保留 `168b56923`）；编号映射 767/768/769→772/773/774；中央脚本生成 PLAN-775，`.next-id=776`。工作树清单和远端 heads 在合入前刷新仍待 T-02。 |
| T-02 | work | 基于最新默认主线建立隔离集成树；重新抓取 refs/工作树/原 Plan 状态，预演所有将合并的仓。 | [ ] |
| T-03 | work | 在各自原树完成 Plan 742/744/768 原 AC、验证和独立 review；未通过项修复后复核。 | [ ] |
| T-04 | work | 核验 Reader 新框架三项 required 能力；缺少的 AutoLang 前置另立专属设计/Plan，交付并完成 Reader 原验收。 | [ ] |
| T-05 | work | 完成 Notes、Blog、Launcher、Weather 等原合同收尾/复审/知识沉淀；核对 Explorer 与其他已开展应用。 | [ ] |
| T-06 | work | 合入 AutoLang v0.5 发布线、本地 master 文档、AutoMusk 两条规约提交和有效恢复信息；消解代码、目录、Plan ID、Spec/debt/roadmap 冲突。 | [ ] |
| T-07 | work | 在 Tetris/Klondike 产品仓迁入并验证发布线源增量；逐项对账其他应用源，再在 AutoOS 普通合并、固定可获取 gitlink SHA。 | [ ] |
| T-08 | work | 刷新所有来源头并跑联合构建/测试/原型/fresh-clone 检查；冻结最终候选和逐仓 evidence。 | [ ] |
| T-09 | review | 在停止写入后独立复查 AC、Spec delta、遗漏/延期/workaround、refs、required 验证和各仓 HEAD；记录 pass 或 needs_fix。 | [ ] |
| T-10 | merge | 复核远端无新提交后按子仓到父仓顺序普通 push/fast-forward；确认远端祖先关系、tags、子仓 SHA 可获取，登记每仓收据。 | [ ] |
| T-11 | merge | 更新现行 workflow/spec/ledger/index、归档已审计划、回到主线；按批量回归到期规则处理，先 `wt-guard.sh` 再清理只属于本计划且干净的工作树。 | [ ] |

## 复审记录

### 2026-10-13 — stage: work / T-01 完成

- revision: 2；outcome: executing；用户已确认本次执行，无待澄清事项。
- v0.6-dev WIP 文档提交：`2e9777b19`。master 与 GitHub master 汇合提交：`7a3047f79`，其父链保留本地目录分析文档提交 `168b56923` 和远端发布主线 `38e4c7c5f`。尚未推送或改写任何发布分支。
- PLAN-775 由 `scripts/new-plan.sh` 在 master 正式生成；编号联合表见 §需求分析。工作树中未提交的本地生成目录 `auto/` 保留原位，未纳入提交。
- next_responsible_role: work；下一步重新核验多仓 refs/WIP，并建立 plan-775-dev 集成树。此前审计报告是固定时点快照，不能替代 T-02 刷新证据。

## 待澄清事项

无。异机未推送成果不可由本机审计证明不存在；若执行中发现新源或远端推进，加入逐仓证据并重跑受影响复审与门禁。
