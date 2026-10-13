# v0.6 主线汇合收据 — 2026-10-13

本收据记录 PLAN-775 按用户修正后的“已提交更新先汇合”阶段。初始调查仍见同目录的 `README.md`、`remote-evidence.json` 与本地快照；它们是 2026-10-12 的基线，不能替代本收据中的最终提交。

GitHub `auto-stack` 的 37 个相关仓默认分支头于 2026-10-13 通过 GitHub API 重新核对；完整名称、分支和 SHA 见 [最终 refs 清单](final-refs-20261013.json)。相对上一版快照，只有 AutoLang 增加了收据文档提交、jade-edit 快进到 v0.5.0 tag。

## 已发布分支

| 仓库 | 已发布头或记录的合入点 | 处理 / 证据 |
|---|---|---|
| auto-lang | `master` `d777ccbb227c41882ad5f6976c803b1e03b61135`；纠偏 merge `3002c30ffd618f768031048742bcc700a2aff363` | 第一父 `v0.5.0` tag target `e05e2d3d7832c7f1d7e29a09a371648b5a53f692`；第二父旧 `master` `9e54f738a07c3715e055ef2a763380e8ec31253e`。旧 merge `77495105284b4ba657c1bd1aa3b864d72aeddb17` 的 v0.6-dev 第二父链仍保留；远端 `master` 通过 expected-SHA 检查、`force=false` 前进。原 tag/commit 未改写。后续 `d777ccbb…` 仅追加本计划收据文档。完整说明见下节。 |
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
| jade-edit | `main` `5fb1e9e28e5a7e379317b7dffb400bad6c94606d` | 无 `v0.6-dev` 分支；从旧 `main` `4d49bbc45bd62da2724a4bc678983f6133087470` 非强制快进至 `v0.5.0` tag target。 |

AutoOS main 的新产品 gitlink 已写为已推送 SHA：Weather `1626a4a…`、Notes `1391bd7…`、Reader `29b5dec…`、Blog `08da633…`、Launcher `df20728…`、Explorer `59a99b9…`、Tetris `06b2e67…`、Klondike `91192ef…`。既有 `auto-term`、Kanban、OS-config 和无变化产品指针按开发线原 SHA 保留。

## AutoLang v0.5.0 ancestry correction — 2026-10-13

初次合流后检查发现 `v0.5.0` tag 仍在，但旧 `master` 并非其后代：两者共同祖先为 `38e4c7c5fa5b8c7f68b8054e342f106a7ec9f5fd`，tag 侧有 5 个独有提交。旧 merge `77495105284b4ba657c1bd1aa3b864d72aeddb17` 的第一父是 `f903848bd8a468f9c4e166a217c1c52111f689a1`，不含 tag；此前“发布主线第一父”的记载有误。该 merge 的第二父 `2e9777b191f32683e8a3bbbe0b993675e38cce63` 保留了本地 `v0.6-dev` 的提交链。

通过追加的双亲提交 `3002c30ffd618f768031048742bcc700a2aff363` 纠正：第一父为未移动的 tag target `e05e2d3d7832c7f1d7e29a09a371648b5a53f692`，第二父为此前已发布的 `master` `9e54f738a07c3715e055ef2a763380e8ec31253e`。合并无冲突，树 `01e8f20f93dd2542f145b8dddf3c5c5884f91653`；相对旧 master 补入 5 个 tag 侧文件变化。差异 whitespace 检查通过，网页改动沿用该发布提交已有的构建和 e2e 验证收据。GitHub 比较确认 tag 和旧 master 都是纠偏提交的祖先。远端 ref 使用旧头 SHA 保护、非强制快进；旧提交和 tag 均未重写。

本地 `master` 和 `origin/master` 已同步到 `d777ccbb227c41882ad5f6976c803b1e03b61135`；根检出的 `auto/` 未跟踪目录保持原位。AutoOS 的保全边界与 T-08 状态不变。

## Jade Edit release baseline fast-forward — 2026-10-13

全仓 ancestry 核验发现 `jade-edit` 是 37 个仓中唯一一个默认分支尚未包含其 `v0.5.0` tag 的仓；该仓没有 `v0.6-dev` 分支。GitHub compare 显示旧 `main` `4d49bbc45bd62da2724a4bc678983f6133087470` 是 tag target `5fb1e9e28e5a7e379317b7dffb400bad6c94606d` 的祖先，落后两条文档提交，文件仅涉及 `DEBTS.md`、`docs/parity-ledger.md` 和已归档 Plan 039。已用 expected-old-SHA 与 `force=false` 将远端 `main` 快进到 tag target；tag 和原提交均未改写。之后 compare 显示 `v0.5.0` 与 `main` 相同，旧 main ahead=2/behind=0。

再次遍历全部 37 个默认分支和同名 `v0.6-dev` refs：23 个仓存在远端 `v0.6-dev` 分支，所有分支尖端均为对应默认分支祖先（behind=0）。11 个仓能解析 `v0.5.0` tag，均为默认分支祖先；其余 26 个仓没有该 tag。AutoLang `v0.5.0..master` compare 为 ahead=311、behind=0。全部 37 个现时默认头写入 refs 清单。

## 未变更仓

基线报告中没有 v0.6-dev 独有提交、没有 GitHub v0.6-dev 分支，或已确认 `main` 与开发分支一致的仓，没有制造空合并。`auto-ai`、`auto-shell` 的远端默认分支已领先而本地开发头无独有提交；`auto-forge` 没有 v0.6-dev。auto-edit/jade-edit 无本机检出且无 v0.6-dev；Jade Edit 的 `main` 已补齐 v0.5.0 tag 历史。auto-ui/auto-man/tree-sitter/books 的本地上游另指 Gitee/Codeup，未擅自重配。全部 37 个相关仓的远端默认分支头已复核并存档，详见 [最终 refs 清单](final-refs-20261013.json)。

## 在途工作与保留边界

- PLAN-775 改为 revision 3，状态继续 `executing`；Plan 773=`execution_done`、Plan 774=`drafting`、Plan 772 已归档，不因合流改变。
- AutoLang 与 AutoDown 的原默认检出在确认无 tracked WIP 后均已快进到远端头；AutoLang 根检出的 `auto/` 未跟踪目录仍在原位。AutoOS 原检出中的 Weather/Notes/Blog/Launcher 指针差异及 probe 文件、AutoMusk 原 main 的两条提交、Weather Plan 008 工作树、各应用原有 notes/blog/launcher/user evidence 均未清理。
- 当前 AutoOS 集成工作树的多个子模块物理检出呈 dirty；`python scripts/verify-app-links.py` 曾在旧 Notes 子模块状态处报 `Dirty submodule apps/015-notes`。自动审核拒绝在这些脏子模块上批量 `git submodule update --checkout`，理由是可能覆盖未提交状态。本轮没有绕过拒绝；改以索引 gitlink、已推送远端 SHA、JSON、shell 语法和源仓提交核验继续。此候选始终保留，未对其执行恢复或删除。
- 为继续 T-08，在独立的 `auto-os-verify` 临时副本中重做验证；此前中断下载造成 8 个子模块只有暂存删除、没有工作区改动或未跟踪文件。仅在该临时副本将它们恢复到当前 HEAD 后，recursive 子模块 SHA mismatch=0，`scripts/verify-app-links.py` 通过：21 个新产品、24 个子模块，另有 3 个既有应用校验通过。验证副本与 AutoLang/AutoMusk 干净 PLAN-775 工作树都通过 `wt-guard.sh`（无 reparse points、clean）后移除；AutoMusk 的 `plan-775-dev` 在确认已并入 `origin/main` 后删除，AutoLang 的临时分支已并入 master 后删除。
- AutoOS 集成候选单独复查：`wt-guard.sh` 确认无 reparse points，但递归 Git 状态仍有 20 个 dirty 应用子模块、2 个 gitlink mismatch（Notes 主提交及其嵌套 AutoDown pin）。因此没有移除 `.wt/lang-775/auto-os` 或其分支；`.wt/lang-775` 组目录继续保留。原 AutoOS 主检出中的 Weather/Notes/Blog/Launcher 指针差异和 probe 文件也保持原样。
- 本机默认检出方面：干净的 auto-ai/auto-shell/auto-forge 在确认分支祖先关系后分别 fast-forward 到 API 核实的远端 SHA（`2a4d606…` / `46bf489…` / `c239d61…`）；AutoLang/AutoDown 已同步。AutoLang 当前 `master` 与 `origin/master` 均为 `d777ccbb…`，根检出的 `auto/` 未跟踪目录保持原位；Plan 742/744/768 等既有工作树未动。AutoOS 原工作树继续停在带 WIP 的 `v0.6-dev`，仅将未检出的本地 `main` 引用 fast-forward 到 `853a551…`；AutoMusk 原本地 main 的两条提交和其他含 WIP/分叉的检出保持不动。

## 执行范围验证

- AutoLang：`cargo check -p auto-lang -p auto-man --quiet`、`cargo tv` 已通过；`cargo t`/`cargo tt` 已知红按既有基线留存。
- AutoDown：`npm pack --dry-run --json` 通过，src 文件计数如上。
- AutoMusk：五个 Markdown 改动文件相对链接检查与 `git diff --check` 通过。
- AutoOS：关键 JSON 可解析，`D:/soft/Git/bin/bash.exe -n scripts/desktop.sh` 通过，手工解冲突文件 whitespace 检查通过；独立 clean checkout recursive SHA mismatch=0，`verify-app-links.py` 通过。
- 这些分支/文档/打包检查不替代各功能 Plan 的 required 双端、编译、运行、独立 review 或知识沉淀。

## 收尾状态

T-08 的远端头刷新、本地安全快进和 AutoOS fresh-clone 验证已完成。清理只删除了通过 `wt-guard` 的干净临时目录。带 20 个 dirty 应用子模块和 2 个 gitlink mismatch 的 AutoOS 集成候选按保全要求继续留存，不对其执行覆盖式同步或删除；该本地 WIP 不影响已发布默认分支的收口。Revision 6 将 T-08 标记完成，PLAN-775 仍待本阶段最终复审和归档；未完成的功能 Plan 按用户要求保持原状态，合流后另行继续。

