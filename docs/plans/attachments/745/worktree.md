# PLAN-745 专属工作树登记

日期：2026-10-06（Asia/Shanghai）；合同：PLAN-745 r1。

| 字段 | 实际值 |
|---|---|
| 主检出 / 主线落点 | D:/autostack/auto-lang / v0.6-dev |
| 唯一实施树 | D:/autostack/.wt/lang-745/auto-lang |
| branch | plan-745-dev |
| 调查代码基线 | ae8199416c772e95dc63e6b647ab3b833bd3cc7c |
| 计划提交 | 21ca500ae（draft + 中央计数746）；c5e3875b6（LF规范化） |
| 实际创建提交 | c5e3875b67174d50193f8945fdfb1e3ec9eb1741 |
| 创建命令 | git worktree add D:/autostack/.wt/lang-745/auto-lang -b plan-745-dev v0.6-dev |
| Git登记 | git worktree list --porcelain 已存在该path/refs/heads/plan-745-dev/创建HEAD |
| 状态核查 | show-toplevel、branch、HEAD与登记一致；git status --short为空 |
| reparse扫描 | PowerShell Get-ChildItem -Recurse -Force -Attributes ReparsePoint，0项；无junction/symlink |
| 簿记同步 | 本登记及new收据提交后，该树仅fast-forward v0.6-dev；实施从同步后的HEAD开始 |

实现、内部检查、最终独立复审顺序共用此树，review期间实施暂停写入。
主检出继续仅维护docs/plans/**计划簿记。每次阶段入口重新核对tree/branch/HEAD/r1与来源hash。
未写实现文件、未执行Cargo或PE。21正向/53反向共74例只是待实施设计，任务0/9。

本树由Git与本计划正式登记。尝试Codex attach_worktree返回“checkout exists but is not a managed worktree”；
未附加到桌面应用托管工件列表。为遵守用户/仓库指定路径，没有另建第二棵应用托管树。
工具不会自动切换shell工作目录；后续work/review必须显式使用本树路径。

本机受限身份读取该树时Git报告所有者差异；已用单命令
`git -c safe.directory=D:/autostack/.wt/lang-745/auto-lang -C D:/autostack/.wt/lang-745/auto-lang ...`
核查，未改全局safe.directory。写入该兄弟目录须走宿主允许的权限路径，不能回主检出实施。

D:/autostack/wt-guard.sh 当前不存在。上述reparse扫描只证明创建时无链接，**不替代删除前的强制guard**。
T-09/merge owner负责先恢复规约指定guard并跑到clean；否则保留worktree，不执行remove/递归清理。

new交接：stage=new；plan=PLAN-745；revision=1；outcome=pass（规划与建树登记完成，非实现/最终review通过）；
next=work/T-01，前提是用户按AGENTS L1确认本合同r1。当前status=drafting，current_step=0/total_steps=9。
合同預检：745在active/archive唯一；23项Spec/代码输入hash新鲜；74个case ID唯一且结果/code与片段有效；
9个唯一任务覆盖9个AC及4个SD，未来review/merge未勾选。没有canonical Spec/实现代码改动。
