# PLAN-799 交付后核查 — 收尾待修

2026-10-17；核查主检出 master@`48ff19821eed299d6966a9a3d64b48f6f48cdefe`。计划已 archived，本次不重开终态、不修改实现/规范/ledger、不重复执行主检出测试。

## 已确认

- delivery `9b510f79a` 是当前HEAD祖先（git merge-base --is-ancestor exit0）。
- 原reviewed `8069076b7` 到rebase后 `3e7dd87fe`，auto-ast全部源码、旧门面、ast_parse_ext及两永久回归测试的diff为空；从3e7dd87fe到当前HEAD这些路径diff亦为空。此前两项代码finding保持closed，本轮未发现新增代码回归。
- archive/799现有文档为archived、8/8；auto-ast/project.md、plans.md及P799-D1/T/R3/M投影已落地。
- git worktree list无799、git branch --list plan-799-dev为空、D:/autostack/.wt/lang-799不存在，清理结果可见。
- 归档§9.5记录冻结门t5261/tv163/tt5633/tu867/delivery9通过；本轮只核收据文字与提交历史，未重新运行这些门。

## F799-CLOSEOUT-01 / P2：旧计划删除未提交

`git ls-tree HEAD docs/plans/799-frontend-ast-model-extraction.md docs/plans/archive/799-frontend-ast-model-extraction.md` 同时返回两份blob。
HEAD旧路径仍是status=drafting、current_step=0；工作区git diff --name-status显示旧路径仅为**未暂存D**，cached diff为空。
因此重新检出当前HEAD会同时出现活动draft与archived8/8，计划归档尚未作为完整原子改动进入历史；与§9.5“archived/cleaned完成”不一致。
修复：提交该旧路径删除，保留唯一archive终态，不重开归档计划。仅收799路径，不带入其他会话WIP。

## F799-CLOSEOUT-02 / P2：复审证据未进入持久提交

归档§9.5声称review-r1/r2/r3附件一并入库，但`.gitignore:161 *.json`与`:44 *.log`排除了关键输入指纹、交付正式收据和日志。
实证：

- `git cat-file -e HEAD:docs/plans/attachments/799/review-r3/baseline.json` exit128；文件仅在本机磁盘存在。
- `git cat-file -e HEAD:docs/plans/attachments/799/review-r2/driver-receipt.json` exit128；同上。
- `git ls-files '*799*'`未收录任何baseline.json/driver-receipt.json/测试log；R3 REVIEW.md第6/26行仍直接依赖这些文件证明提交/依赖绑定与25/25证据复用。

这不表示已执行的测试失败，但意味着新检出者无法取得报告所引用的精确输入指纹和正式收据，违反可随工作树移除保留的证据要求。
修复：显式收录必要证据（逐路径force-add，或采用已有`.json.txt`/`.log.txt`惯例并更新引用）；至少包含报告使用的r1/r2/r3 baseline、792 driver receipt及对应可解释的原始结果。无需宽泛放开全仓ignore或包含他线材料。

## 下一步

两项均为合入收尾缺口，不是新的AST语义finding。归档终态保持不变，在主检出按799路径完成文档/证据补正提交，再检查HEAD中旧活动路径消失、引用证据可由git取出。本次未代为提交或修复，亦未授予新的交付收尾pass。
