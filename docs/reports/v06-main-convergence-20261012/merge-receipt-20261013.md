# v0.6 主线汇合收据 — 2026-10-13

本收据记录 PLAN-775 按用户修正后的“已提交更新先汇合”阶段。初始调查仍见同目录的 `README.md`、`remote-evidence.json` 与本地快照；它们是 2026-10-12 的基线，不能替代本收据中的最终提交。

GitHub `auto-stack` 的 37 个相关仓默认分支头已于 2026-10-13 通过 GitHub API 重新核对；完整名称、分支和 SHA 见 [最终 refs 清单](final-refs-20261013.json)。本次变更仓的当前头均与已发布回执相符。

## 已发布分支

| 仓库 | 默认分支头 | 处理 / 证据 |
|---|---|---|
| auto-lang | `master` `e35f9e99b6711ff6f1990ea980729d1fa521327e` | merge `77495105284b4ba657c1bd1aa3b864d72aeddb17`：发布主线为第一父、v0.6-dev 本地头 `2e9777b191f32683e8a3bbbe0b993675e38cce63` 为第二父；`origin/v0.6-dev=94173fe2c175ef007a825a25050a2f04fb5ed98a` 保持原样。v0.5.0 tag 解引用 SHA `e05e2d3d7832c7f1d7e29a09a371648b5a53f692` 未移动。 |
| auto-down | `master` `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | 从 `895f8d0f9355c9f5ec3ce8fca268bdb768395846` 普通快进；`npm pack --dry-run --json` 确认包含 178 个 `src/*` 文件及 `src/index.ts`。 |
| auto-notes | `main` `1391bd7eb2f6b3b08e9841d30b25125e47ced058` | 从 `9117347b24bfc4ecd608a924f63cdfd42de5491d` 普通快进，嵌套 `vendor/auto-down` 指向 `c39ac7f…`。原提交中不可达的 `96f095b…` 留在历史，没有重写。 |
| auto-blog | `main` `08da63359c4f8d147c0c10f2c774d6feaad79425` | 从远端旧头 `55bd49e…` 快进到已提交 v0.6 头。 |
| auto-reader | `main` `29b5decf878ec2cdca5c6a39575e872a4597121d` | main 与 v0.6-dev 已一致，无额外合入提交。 |
| auto-launcher | `main` `df207283536ff943e2e574dc886c4cb8594cce13` | 从 `00582ce…` 快进到已推送 v0.6 头。 |
| auto-weather | `main` `1626a4a16ccf4de717bc92e27b60a9184b8198f2` | main 与 v0.6-dev 一致；Plan 008 的本地未提交工作留在原工作树。 |
| auto-explorer / auto-file-manager | `main` `59a99b9f42896728aa32a7ff3e38fa08261c7d18` | 从 `19d13ed…` 快进到已推送 v0.6 头。 |
| auto-tetris | `main`、`v0.6-dev` `06b2e67f2117084853271cf194e67cc47be6b3c5` | 先在产品仓保留 v0.5 源 delta，再把同一提交向两条分支普通快进。 |
| auto-solitaire / auto-klondike | `main`、`v0.6-dev` `91192ef593cb25377a2d089542e6f040507231a9` | 先在产品仓保留 v0.5 源 delta，再把同一提交向两条分支普通快进。 |
| auto-os | `main` `853a551e681ad888f4c72c519b6ad1df94e7f313` | merge 第一父 `e7fadc470684f611620175c722a1a22df5e9d9c9`、第二父 `71d3b5f43ff4a20b998bca2917f7bf62b101fdd7`；`v0.6-dev` 未移动。 |
| auto-musk | `main` `cb4cb9570ab185b0a2a7a645cc80edd65a1689f4` | 两条本地规约提交以 `-x` cherry-pick：`8e09f5b… → eecac9b…`、`bfcf373… → cb4cb95…`。本机原 `main` 和原提交保持原样。 |

AutoOS main 的新产品 gitlink 已写为已推送 SHA：Weather `1626a4a…`、Notes `1391bd7…`、Reader `29b5dec…`、Blog `08da633…`、Launcher `df20728…`、Explorer `59a99b9…`、Tetris `06b2e67…`、Klondike `91192ef…`。既有 `auto-term`、Kanban、OS-config 和无变化产品指针按开发线原 SHA 保留。

## 未变更仓

基线报告中没有 v0.6-dev 独有提交、没有 GitHub v0.6-dev 分支，或已确认 `main` 与开发分支一致的仓，没有制造空合并。`auto-ai`、`auto-shell` 的远端默认分支已领先而本地开发头无独有提交；`auto-forge` 没有 v0.6-dev。auto-edit/jade-edit 无本机检出且无 v0.6-dev；auto-ui/auto-man/tree-sitter/books 的本地上游另指 Gitee/Codeup，未擅自重配。全部 37 个相关仓的远端默认分支头已于 2026-10-13 查询并存档，详见 [最终 refs 清单](final-refs-20261013.json)。

## 在途工作与保留边界

- PLAN-775 改为 revision 3，状态继续 `executing`；Plan 773=`execution_done`、Plan 774=`drafting`、Plan 772 已归档，不因合流改变。
- AutoLang 与 AutoDown 的原默认检出在确认无 tracked WIP 后均已快进到远端头；AutoLang 根检出的 `auto/` 未跟踪目录仍在原位。AutoOS 原检出中的 Weather/Notes/Blog/Launcher 指针差异及 probe 文件、AutoMusk 原 main 的两条提交、Weather Plan 008 工作树、各应用原有 notes/blog/launcher/user evidence 均未清理。
- 当前 AutoOS 集成工作树的多个子模块物理检出呈 dirty；`python scripts/verify-app-links.py` 曾在旧 Notes 子模块状态处报 `Dirty submodule apps/015-notes`。自动审核拒绝在这些脏子模块上批量 `git submodule update --checkout`，理由是可能覆盖未提交状态。本轮没有绕过拒绝；改以索引 gitlink、已推送远端 SHA、JSON、shell 语法和源仓提交核验继续。此候选始终保留，未对其执行恢复或删除。
- 为继续 T-08，在独立的 `auto-os-verify` 临时副本中重做验证；此前中断下载造成 8 个子模块只有暂存删除、没有工作区改动或未跟踪文件。仅在该临时副本将它们恢复到当前 HEAD 后，recursive 子模块 SHA mismatch=0，`scripts/verify-app-links.py` 通过：21 个新产品、24 个子模块，另有 3 个既有应用校验通过。验证副本与 AutoLang/AutoMusk 干净 PLAN-775 工作树都通过 `wt-guard.sh`（无 reparse points、clean）后移除；AutoMusk 的 `plan-775-dev` 在确认已并入 `origin/main` 后删除，AutoLang 的临时分支已并入 master 后删除。
- AutoOS 集成候选单独复查：`wt-guard.sh` 确认无 reparse points，但递归 Git 状态仍有 20 个 dirty 应用子模块、2 个 gitlink mismatch（Notes 主提交及其嵌套 AutoDown pin）。因此没有移除 `.wt/lang-775/auto-os` 或其分支；`.wt/lang-775` 组目录继续保留。原 AutoOS 主检出中的 Weather/Notes/Blog/Launcher 指针差异和 probe 文件也保持原样。
- 本机默认检出方面：干净的 auto-ai/auto-shell/auto-forge 在确认分支祖先关系后分别 fast-forward 到 API 核实的远端 SHA（`2a4d606…` / `46bf489…` / `c239d61…`）；AutoLang/AutoDown 已同步。AutoOS 原工作树继续停在带 WIP 的 `v0.6-dev`，仅将未检出的本地 `main` 引用 fast-forward 到 `853a551…`；AutoMusk 原本地 main 的两条提交和其他含 WIP/分叉的检出保持不动。

## 执行范围验证

- AutoLang：`cargo check -p auto-lang -p auto-man --quiet`、`cargo tv` 已通过；`cargo t`/`cargo tt` 已知红按既有基线留存。
- AutoDown：`npm pack --dry-run --json` 通过，src 文件计数如上。
- AutoMusk：五个 Markdown 改动文件相对链接检查与 `git diff --check` 通过。
- AutoOS：关键 JSON 可解析，`D:/soft/Git/bin/bash.exe -n scripts/desktop.sh` 通过，手工解冲突文件 whitespace 检查通过；独立 clean checkout recursive SHA mismatch=0，`verify-app-links.py` 通过。
- 这些分支/文档/打包检查不替代各功能 Plan 的 required 双端、编译、运行、独立 review 或知识沉淀。

## 收尾状态

T-08 的远端头刷新、本地安全快进和 AutoOS fresh-clone 验证已完成。清理只删除了通过 `wt-guard` 的干净临时目录。带 20 个 dirty 应用子模块和 2 个 gitlink mismatch 的 AutoOS 集成候选因保全要求仍保留，所以 PLAN-775 继续为 `executing`，T-08 未勾选，AC-07 仍 partial；本收据不宣称该 Plan 已归档或整体 closeout pass。未完成的功能 Plan 按用户要求保持原状态，合流后继续。
