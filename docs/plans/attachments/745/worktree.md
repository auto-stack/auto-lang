# PLAN-745 专属工作树登记

日期：2026-10-06（Asia/Shanghai）；合同：PLAN-745 r1。

## 2026-10-06 work 追加：组内依赖树（T-07 门禁需要）

主仓 cargo 门禁（check/t/tv）经 `crates/auto-lang` 的相对路径依赖
`../../../auto-down/autodown/packages/engine/rust`（autodown-core，optional path dep，
workspace 装载仍需 manifest 可读）。按 Plan 529 组布局与 lang-742 先例，在**同组**
补建只读依赖树（不修改 auto-down）：

| 字段 | 值 |
|---|---|
| 依赖树 | D:/autostack/.wt/lang-745/auto-down（git worktree，detached） |
| 基线提交 | fba6563ed2148ce85e68208863159b4ccccac710（= auto-down master HEAD，与 lang-742 组一致） |
| 创建命令 | git -C D:/autostack/auto-down worktree add --detach D:/autostack/.wt/lang-745/auto-down fba6563ed |
| 用途 | 仅满足依赖解析与门禁构建；PLAN-745 不改 auto-down 任何文件 |
| 清理 | T-09/merge 收口时与 auto-lang 树一起 wt-guard + 移除（依赖树无本地提交） |

另：D: 盘在 T-07 期间 100% 满（os error 112）；主仓门禁改用
`CARGO_TARGET_DIR=C:/tmp/ac745-main-gates`（C: 盘独立目录，单实例，无共享竞争）。
工作树内 D: 上已有 `cargo check -p auto-lang` 通过的 1.6G 增量产物（保留待 review 复用）。

---

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
