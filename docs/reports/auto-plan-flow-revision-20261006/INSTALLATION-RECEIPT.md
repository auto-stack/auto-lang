# 共享 auto-plan 技能安装记录（2026-10-06）

用户批准实施后，四个技能及条件性验收参考已在 auto-musk 专用工作树修改、验证并合入主分支。

- 源仓库：`D:/autostack/auto-musk`，分支 `main`。
- 安装提交：`8e09f5bbd6df29cc143dbd56b7261f7ebf927876`。
- 基线：`edde608709c5dd5b413fc9c534113d41f0bb8afe`；rebase 未改变提交，`--ff-only` 合入，无 merge commit。
- 四个共享入口：`C:/Users/zhaop/.agents/skills/auto-plan-{new,work,review,merge}`；原有 SymbolicLink 保留，目标为源仓库对应技能目录。逐项核验链接目标与源/入口/交付文件 SHA256 一致。
- 临时工作树：`D:/autostack/.wt/plan-skill-contracts/auto-musk`；临时分支 `auto-musk-dev-1`。工作树登记、目录、分支及空组目录均已删除，源仓库主检出 clean。

原 [patch-manifest.json](patch-manifest.json) 的 `installed: false` 记录安装前修订稿状态；原补丁及候选哈希保留为历史。实际安装版本包括用户追加的“同一计划共用工作树”要求，以下哈希才是本次安装结果。

## 已生效规则

1. 高风险计划写清不变量、合法/非法代表输入、真实入口、可观察结果、适用路径与边界；普通小修改保留按范围验证。
2. work/review/merge 阶段责任分开；内部 pass 只交给指定最终复审，最终 pass 才设 reviewed、允许合并。
3. 修复检查同根因的适用路径/消费者/生命周期，并保留回归；连续两次最终复审失败先诊断覆盖与设计。
4. 每计划每仓库一个权威实施工作树；实现、内部检查和最终复审共享它、顺序交接。独立的是上下文；共享计划簿记仍在主检出。
5. 交接记录路径、仓库、分支、基线/当前提交、依赖与下一责任方；复审前提交并暂停实现写入，复审前后核对基线。Agent 必须显式设置工具工作目录，聊天默认目录不代表计划目录。
6. 合并后核查实际归档的任务、计数、引用和目标身份；验证收尾后再清理，清理实际完成才勾选其任务。

## 核查与实际版本

- 工作树候选和共享安装入口各跑四次 skill-creator `quick_validate.py`：全部通过。
- 五个交付文件的本地 Markdown 链接逐项解析：8 条，0 缺失（此核查只统计 Markdown 链接，不统计代码块中的路径）。
- `git diff --check` 与 staged diff check 通过；人工检查内部/最终状态分工、共享工作树交接、历史已落地审查例外和归档/清理时序。
- 文档/技能改动，无 Rust 源码变化，未跑 Cargo 或 docs_gen。
- 未进行独立模型行为试验；结构与规则核查不能证明后续模型必定遵循。用下一至两个计划衡量首次最终通过率、内部误放行及同根因复发。

| 技能文件（相对源技能目录） | SHA256 |
|---|---|
| `auto-plan-new/SKILL.md` | `3fa3c96edf47b71217966532a6ec64c0cadc508d0347cb372d766913b8fac585` |
| `auto-plan-work/SKILL.md` | `9dd659be8b3f3a7cfce37cb1b42ba2c9d4ca4589f7be9a1ed923da39d5606ec2` |
| `auto-plan-review/SKILL.md` | `68ada90c9c522228dcc50be47a721cad980806a3295be59e9d7da3375a5b3fa7` |
| `auto-plan-merge/SKILL.md` | `5528950385b6949f0654524b251ed24ce0593816e36b802475d2d53e0079f1b9` |
| `auto-plan-new/references/verification-contract.md` | `d31998f1a58d295b7798b367139550c47bab2d924fb2f19c7311abbb09e4da9b` |

## 安全检查与未改范围

`D:/autostack/wt-guard.sh` 本机仍缺失，沿用本会话 741/743 收据中已记录的等价检查：精确绝对组/工作树路径与 Git 归属、工作树 clean；用 PowerShell 显式栈扫描整个组，每遇 ReparsePoint 立即拒绝且不下降。合入前和 rebase 后各扫描 1081 项；移除前新扫描 1081 项并检查所有祖先均非 ReparsePoint、分支已完整落地。工作树由 `git worktree remove` 删除，组目录仅以非递归空目录删除。

Relay Rust 模板、后端状态机和模型自动路由未修改，两个驱动文件哈希与安装前 manifest 一致。不能宣称已有 Runner 自动识别这些正文/收据字段或自动迁移聊天目录。当前分工仍须明确：GLM 实现及内部检查 → ChatGPT 最终复审 → 合并；若旧自动链路内部 pass 后立即 merge，应先停用该跳转，或另行实现路由和收据门禁，再使用它自动收尾。
