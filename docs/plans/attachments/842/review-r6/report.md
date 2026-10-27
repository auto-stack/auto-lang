# PLAN-842 独立最终复审 R6：needs_fix

用户已明确扩大自动修复和复审循环。本次为独立子代理 /root/review842；继承历史交接，但重新读取提交与实际生产入口，并执行独立正反控。计划 revision 2；工作组 D:/autostack/.wt/lang-842/{auto-lang,auto-ui,auto-man}，分支 codex/plan-842-dev。复审提交 lang a8b4a5bcf53d9be484cc38f028453a4cebbc947f、UI ab81f0406e3401f94f960c60eb87ff460763ded3、Man 2e69777dba3cbd6cac96d1544bb5866388ead36a，down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；各实施工作树 clean。语言主线基面 a547f52c5fbd5a24a203e94051af1e040c808f42 已包含在工作树；UI 基面 4903bef64532750d1597627b1197f3d2cac1effc、Man 基面 9a75d3bab6eaeda961d5dcbf7932258f89e650af。规范与配置 [哈希](input-hashes.json)，[SD 快照](spec-delta-snapshot.md)。

## F-842-R6-01：显式受管 UI 源缺失 staging 副本校验（P1）

违反 AC-02/04/08、R09/R10/R12、V-02/V-06c 和 R3 诊断中“每个语言/UI crate 的 staging 原始字节与所选冻结收据一致、失败不替换旧产物”。在真实最终 Full 安装 A 的隔离副本上，恢复并 verify 完整 SDK；对已有有效交付加旧 app sentinel，指定 AUTO_UI_SOURCE_ROOT 为该安装同一个 sdk；实际 auto.exe gen --delivery-root 正常进入 staging。观察当前事务的 staging/crates/auto-ui-compiler 创建，说明对选中 UI 源的预检已完成、复制开始；此时短暂修改尚未复制的 sdk/crates/auto-ui-compiler/src/ui_gen/vue.rs，确认对应 staging 文件出现后立即恢复源文件原始字节。实际命令 exit 0，改动字节进入新交付，原 app sentinel 被覆盖。[摘要](same-sdk-drift-summary.log)、[原始输出](same-sdk-drift-generation.log)。随后隔离安装完整 verify exit 0，说明探针恢复源成功。

对另一份经 verify 的完整安装 B，使用最终安装 A 的真实命令、指定 AUTO_UI_SOURCE_ROOT=B/sdk，对旧交付重放相同复制期间变化并恢复；同样 exit 0、修改 UI 字节进入交付且覆盖旧 app，见 [摘要](other-sdk-drift-summary.log)、[输出](other-sdk-drift-generation.log)。因此这不是单纯同一路径别名的文本比较。

已证实原因：rust_ui.rs 复制每个 crate 后对 UI 受管来源调用 staged receipt 校验只在 AUTO_UI_SOURCE_ROOT 未设置时执行；明确指定哪怕是受管安装时同时跳过 producer receipt 的 UI staged 校验。源码中后续可能重新验证源，但源在读取后已恢复，不能证明 staging 副本。原 R3 的源在整个复制阶段持续漂移负控会被后续全 SDK 预检捕获，不能覆盖本次修改、复制、恢复组合。

关闭 oracle：对实际选中的每个受管 UI 源冻结与该物理源匹配的有效安装收据，在投影前验证对应 staged 原始文件集合/字节，提交前验证所冻结收据没有漂移；同 SDK 显式、另一份受管安装、缺/坏收据、修改后恢复、旧产物逐字节保全及合法非受管 UI 覆盖均需实际控制。不要将所有显式 UI 根当成受管，也不能只检查已经恢复的源。

## F-842-R6-02：另一份受管 UI 安装的版本身份丢失（P2）

违反 AC-02/04/05 及 SD-04 关于分别记录语言和 UI 所有者/来源修订。复制最终 Full 安装为 B，man-install verify --profile full 对 B exit 0；使用最终安装 A 的真实 CLI，明确 AUTO_UI_SOURCE_ROOT=B/sdk，完成真实交付生成 exit 0。新 delivery-manifest 的 ui_source_root 正确指向 B/sdk，但 ui_git_head 为 null；B 自己的 install-manifest 明确记录 auto-ui 源提交 ab81f0406e3401f94f960c60eb87ff460763ded3。见 [摘要](other-sdk-revision-summary.log)、[B 校验](other-sdk-verify.log)、[生成输出](other-sdk-revision-generation.log)。

原因：rust_ui.rs 在语言 producer 有受管收据时，仅当 UI 根和语言 producer 是同一 canonical 目录才从该收据记录 auto-ui revision；不同的有效受管 UI 根不是 Git 检出，delivery_git_identity 返回 None，却没有使用 B 自己的已验证收据。新功能不能把源摘要置空或误标成 A；合法普通 UI 开发树仍应记录自己的 Git 身份。关闭需同时核实外部独立受管 B 的 source root/SHA 与其冻结收据一致、同安装别名正控及 UI 收据中途变化时拒绝。

## 既有发现、测试和审查边界

- 独立实际重放 R5：对受管安装同一 sdk 的小写/大写显式语言根输入坏内容，两个真实 workspace 入口均 exit 1、旧 app 不变；大写交付根也拒绝并保旧；恢复后真实 workspace 成功。见 [控制摘要](sdk-control-summary.txt) 和相关日志。R5 path-only bogus 包版本在真实两入口已由修复记录的正反控拒绝；本轮另外在独立普通开发 SDK 实测省略版本的合法 Cargo 默认与 workspace.package.version 合法继承，真实 gen 和 cargo metadata 均 exit 0；将继承版本变为 bogus 后二者分别 exit 1/101 且旧 app 不变。见 versions-default/inherited/bad-inherited 原始日志。
- R4 普通环、包身份、版本、UI 目标等实际两入口和 Cargo 对照已核对工作期绑定日志；合法 dev 环、package 重命名、目标与 workspace 继承通过具体控制。R3 语言 crate、patch、根输入、根锁、缺失/额外和复制期漂移的最终版本真实日志均已核对；F-R6-01 是不同的显式受管 UI 来源及时序组合，不应由上述结论覆盖。
- 独立读取 [最终测试选择摘要](../repair-r5/final-selection-summary.json.txt) 并核对四份原始 nextest 清单 SHA-256，映射清单中 21 项均实际 listed+selected，UI 集成目标包含在执行命令；language 3212、UI 3117、tt 3563、tb 3102。仅 list-only，不等于测试档通过。此前 18 项驱动控制及 9+10+2 定向执行在不变代码范围内复用。
- 相对实际三仓基面的 diff-check、HEAD 和 clean 状态本轮复核；现存 compiler warnings 的全范围归因与最终健康检查仍未完成，未测 MPV 播放和既有 ignored 的限界不变。没有运行 merge-owned 全档。
- AC-01/03 的未变行为按已记录的具体输入保留；AC-02/04/08 fail；AC-05 的测试选择控制通过，但第二来源身份缺失仍属部分未完成；AC-06 独立最终 needs_fix；AC-07、裸 cargo t、触面档、到期 tf、实际合入与归档均 pending。SD-04/06 的每源身份和坏源保护尚不成立，其余未受影响条目需按最终提交复审，不能由本次记录冒称通过。
- 该 non-pass 后 work 须在修复前诊断共同不变量、完整不同受管源/显式与默认来源状态、源冻结与 staged 字节两者对应，以及为何先前负控未捕获“修改—复制—恢复”；按用户新授权和 auto-plan-work 限额计数，不把最终复审改名内部审查。完整修复并重绑实际三仓安装/来源证据后再最终复审。

本次未改实施源码、未运行全档、未合入或更新 canonical Specs。
