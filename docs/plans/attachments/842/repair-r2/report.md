# PLAN-842 R2 修复、覆盖与并集证据

独立复审问题见 [R2](../review-r2/report.md)。语言计划分支已在主线 `a547f52c5fbd5a24a203e94051af1e040c808f42` 上重放，原 18 项提交全部 patch 等价，见 [range-diff](r2-lang-rebase-range.log)；下列来源包含该并集。

- F-842-R2-01/02：显式、安装、直接、兄弟和编译期语言候选共享完整性检查；检测到受管安装 SDK 或安装清单即视为权威，缺包、清单损坏、收据/源码 hash 不符均具名拒绝，不再借开发源。UI 交付和工作区两条实际入口均先预检。真实最终 Full 安装坏清单、可解析但篡改、缺 auto-lang，均拒绝并保全旧 workspace/app/交付 sentinel 与 manifest；恢复实际重试通过。显式合法覆盖、正常安装正控通过；独立开发进程 direct/sibling 残缺根被拒绝、旧文件不变、恢复正常见 [真实发现](r2-dev-real-discovery.log) 与 [原始日志](dev-discovery/)。源码工作区针对缺失/损坏/错误包/缺入口及合法根控制 4/4；UI 来源 7/7、交付 1/1、R1 备份恢复 2/2。
- F-842-R2-03：去除退休测试脚本多余 EOF；最终相对新主线完整 `git diff --check` 和原基线至该 fixture 的独立检查均通过。原基线至新 HEAD 的全树对比还包含不属于 842 的主线 824 原始证据空白，不修改他人历史。
- AC-05 / V-05：824 批量发现的 21 个历史 ID 分别绑定现有的 10 个 UI compiler design_tokens、core parser scene、错误后 scope 恢复（841 有意改变旧泄漏语义）及 9 个 UI compiler recipe integration；逐条映射和理由为源码提交内 `scripts/full-regression-migrations-841.json.txt`，原基线不删。驱动在每档保留旧要求，跨 owner 将目标加入 UI 必选集，UI full 同时选择原 lib 与 recipe integration。针对每个映射的缺失/不选负控和驱动测试 18/18；真实 `tf --batch --list-only` 仅列举不运行整档，language 3212、UI 3117、tt 3563、tb 3102 selected，21/21 目标实际 selected，见 [收据](batch-selection/receipt.json) 和同目录四份原始列表；此前未修复为 12 missing/12 suppressed。recipe 9/9、design_tokens 10/10、scene 1/1、错误恢复 1/1 实际定向通过。不得将这次 list-only 称为全量回归通过。

## 最新固定来源与安装

lang `5f3bd3701c24a916312189ea1be84de9a7cc8196`，UI `98119a4a2d3f6a9a5e90efc629dd45625b4e970e`，Man `0062f9224ba6b987f8e202827fa8f2e1b971ec8d`，down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；三实施树 clean。

Full 安装 6806 文件，manifest SHA-256 `0f793c306617f3f5f1797a8acf3825c0400a8fc2e0ad6932e9ae93e15b7d9725`；真实安装 verify、交付 fresh lock/locked build、宿主探针、坏源保旧恢复、旧备份失败/成功恢复全部通过，见 [Full 日志](r3-final-delivery.log) 和 [分项](installed-logs/)。资源等长篡改和未来 schema 拒绝/恢复通过。Core 636 文件、manifest SHA-256 `8a4788a9e08e63baa5bc9c81e972e001f1fecf2b517df0a0f90df7718c4275df`；实际 VM、UI 缺席、独立 consumer fresh lock/locked 3/3 与依赖隔离通过。七个退役旧入口、三个旧精确版本、旧 feature 的外部反例及 canonical 0.2/VM 正控在并集上通过。

最后一次 lang 变更只修改批量驱动、其回归测试及迁移清单；Full/Core 安装与前一次的所有受管文件哈希仅 `.man-staging.marker` 因独占事务变化，其余逐项相同，见 [对比](r3-install-content-comparison.json)。因此前次以相同实际二进制和示例/依赖输入运行的 Vue 生成/构建、Core SDK check 可复用；真实 final Full/Core 安装和上述实际消费者仍重新执行。已有编译 warning、未实测 MPV 视频和 1 项既有 ignored 如实保留。

独立复审尚未进行；裸 cargo t、适用触面档和到期的主检出批量门均未运行，不能据此合入或宣称交付。
