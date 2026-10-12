# 跨仓库文件操作安全规范

> 来源：2026-08-02 使用 robocopy /MIR 清理含循环 junction 的 worktree，导致两个仓库的 .git 和 auto-ai 工作区被清空；2026-09-03 又确认 git worktree remove 也可能穿透链接。PLAN-761 将原根 SAFETY.md 归位并与当前 AGENTS.md 的禁链接规约对齐。

## 1. 事故根因与永久禁令

破坏性递归文件操作与 junction/symlink 组合可能删除目标仓。历史现场包括 worktree 内 auto-lang/ 指回主仓自身、auto-ai/ 指向兄弟仓，以及跨仓 Cargo path 依赖。

- worktree 内禁止创建任何 junction/symlink。忽略规则不构成安全屏障。
- 禁止对含链接或未经核查的目录执行 robocopy /MIR、rm -rf、rsync --delete、Remove-Item -Recurse 或 find -delete。
- git worktree remove 也必须先通过 guard；不能假设 Git 会保护 junction 的目标。
- 根目录整理只操作核实过的目标，保留 .git、他人 WIP 和未归档证据；不按 tmp/worktree 等名字推断目录可删。

## 2. 操作前预检

核对目标的绝对路径、所属仓库和 Git worktree 记录。Windows 路径必须落在本次任务已明确的目标组内；检查目标及父目录链的 reparse points：

```sh
bash scripts/check-junctions.sh <目标目录>
bash D:/autostack/wt-guard.sh <worktree绝对路径>
```

本机使用 Git Bash 运行 Windows guard。发现任何链接即停止，先核查归属；只摘除链接本身，不递归处理它指向的目录。无链接也不能代替未提交内容/使用中进程检查。

## 3. Worktree 清理

完成提交、独立复审、合入和证据保存后，再清理本计划 worktree：

```sh
git worktree list --porcelain
bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-<NNN>/auto-lang
# 必须输出 clean 才继续
git worktree remove D:/autostack/.wt/lang-<NNN>/auto-lang
git branch -d plan-<NNN>-dev
```

组内兄弟 worktree 同样先检查其归属、脏态、合入状态及 guard。只删除已经没有兄弟 worktree 的空组目录。遇到长路径、链接或脏态失败，保留现场并查明原因；不得用强制递归删除绕过失败。

## 4. 跨仓依赖

当前统一使用分组平铺布局：D:/autostack/.wt/lang-<NNN>/{auto-lang,auto-down,...}。依赖按环境覆盖、组内兄弟、主检出兜底解析，不创建链接。各依赖具体变量和规则见根 AGENTS.md 与对应 resolver。

Windows reparse points 应使用 Windows 原生目录枚举核查；仅用 Git Bash find -type l 会漏 junction。目录名与主仓同名不足以证明是链接，也不足以证明安全。

历史根 /auto-lang/、/auto-ai/ ignore 规则用于避免旧机器安装物误入库，不能作为允许创建 junction 的依据。ignore 必须根锚定，避免误吞 crates/auto-lang/ 这样的正常源码目录。

## 5. 事故应急

立即停止文件操作，核查 .git、工作区及其他链接目标的损失；保留日志、备份、尚存 Git 对象和磁盘现场。确认情况后从可信备份或远端恢复，不在原路径盲目覆盖。未推送且无备份的数据可能无法恢复，因此完成工作后应及时提交并按授权发布。

适用 D:/autostack 下各仓库；本仓具体工作流以 [AGENTS.md](../../AGENTS.md) 为准。
