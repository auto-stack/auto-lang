# PLAN-832 r3 / T-10 final R3

2026-10-25。**outcome: pass**。F832-R1-01..04 全部 CLOSED；代码、验收与 SD 审查通过。**merge-owned 统一门仍 pending，本结论不等于已合入或交付。**

## 身份与审查边界

沿用 R1/R2 的审查聊天，独立于实施和修复，未创建agent或第二权威checkout，未虚构模型/会话ID。读取主检出当前 r3 Plan，在原实施树顺序复审；实现未修改。

| 仓 | reviewed commit | 本轮 repair base | checkout / branch |
|---|---|---|---|
| lang | `e320abf4fa1ae2d00d3e2bc91a35eea53345f409` | `87f82d637ecd586c17a01c1c975e1456bc346991` | `D:/autostack/.wt/lang-832/auto-lang` / `codex/plan-832-dev` |
| Man | `f52cd93b09401f358a89a19058fbf1fa7673fd16` | `ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f` | 同组 `auto-man` / `codex/plan-832-dev` |
| UI | `aee499e87093ff727f68ac8467c506e9ecf6ecdc` | 已交付固定依赖 | 同组 `auto-ui` / detached |
| down | `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | 已交付固定依赖 | 同组 `auto-down` / detached |

四树进出 HEAD/clean 一致，锁、依赖和配置身份已核验。原始整体diff base仍为lang `45558ab628bb29a8719f8467046b3ae0d5e36d7a`、Man `51fafe82e8e0c4e66325ee14088eed8c3270e07e`；并未通过换baseline隐藏原合同。

冻结 [Plan](plan-input.md)、[handoff](handoff-input.md)、[repair](repair-input.md)、[覆盖诊断](diagnosis-input.md)、[SD终稿](spec-delta-reviewed.md)、[实际SD patch](spec-delta-reviewed.patch.txt) 与 [hash/源码/锁/技能/二进制](baseline.json.txt)。主检出其他session WIP保留；没有发布canonical Spec或live ledger。

## Finding closure

| Finding | final status | 独立依据 |
|---|---|---|
| F832-R1-01 / P1 | CLOSED保持 | R2的独立Core来源/闭包/640安装/SDK/真实管理语言与重定位审查保持。Core workspace/manifests/installer/锁未改；本轮discovery增量是标准库路径推断，没有增加UI依赖或改变Core投影；不机械重跑已闭族 |
| F832-R1-02 / P2 | CLOSED保持 | R2实际AccessDenied/live/dead/unknown反例及合法控制保持；`man-install.py`完全无diff，不再运行owner全组 |
| F832-R1-03 / P2 | **CLOSED** | 直接CLI从自身标准开发布局推导同组UI owner；真实清环境/显式/安装/缺件矩阵和新增兄弟布局控制通过，永久测试不再注入tool |
| F832-R1-04 / P2 | **CLOSED** | 从真实升级backup按文档PowerShell步骤保存owned marker到root之外，再严格verify和执行旧命令；陌生extra仍拒绝，full永久控制有效 |

### F03：生产路径及代表兄弟分支

`auto-tool-client::discovery` 的显式spec/env/Cargo锚优先级保持，缺省新增 `current_exe` → `target/debug|release` → 带Cargo.toml/crates marker的仓根 → 同组UI owner。代码无任意主检出扫描。`ToolUiOps`仍用默认spec，因而直接CLI实际经过该分支。

本轮先从冻结Man源码 `cargo build --locked -p auto`，再由 [replay.py](replay.py) 在全新自有scratch构造同一合法 `src/front` 工程，剥离 AUTO_*、CARGO_MANIFEST_DIR、TE_CHILD_BIN、CARGO_TARGET_DIR/RUSTC_WRAPPER；每格初始无生成物。每个成功格还检查 `App.vue` **包含本轮marker `review-r3`**，避免旧产物假绿。原始命令、退出、诊断、产物见 [results](results.json.txt)。

| 检查 → 不变量 | 结果 |
|---|---|
| 普通直接CLI、无Cargo/AUTO环境 → R2原失败入口 | exit0，新App.vue |
| 显式tool / Cargo manifest锚 / 显式UI根 → 既有合法配置 | 各exit0，新App.vue |
| 显式缺tool / 缺UI根 → 缺件具名拒绝 | 各exit1，无App.vue |
| 独立安装、邻接tool，无Cargo环境 → 安装控制 | exit0，新App.vue |
| 标准release布局 → 新机制的兄弟分支 | exit0，新App.vue |
| 平放CLI、无邻接tool/仓marker → 不搜索原主检出或编译期owner | exit1，无App.vue |
| 永久 `gen_runs_through_real_tool_process --exact` → 无注入生产回归 | 1 selected / 1 passed / exit0 |

共九格生产/布局对照全部符合期望。release格用同字节当前CLI与真实UI工具放入自有运行布局，只检验布局分支；**未声称执行了优化release构建**，也未创建新Git checkout。平放负控时原真实UI owner仍在，失败证明没有借原路径或全局主检出兜底。

永久测试现在直接调用CLI，公共启动helper移除 `CARGO_MANIFEST_DIR`/`AUTO_UI_TOOL_BIN`/`AUTO_UI_ROOT`，调用处不再先解析tool再注入。已核其实际调用和断言，独立重放1/1通过；不是只接受work的总数。

### F04：真实备份、实际文档操作与严格负控

使用R2已验证Core交付作为“旧版本”输入（其原manifest来源保持，未冒称由当前commit重建），复制到本轮新scratch。当前真实installer执行正常升级，产生包含 `.man-backup.marker` 的真实backup；备份manifest与旧manifest逐字相等。

随后实际运行 [documented-restore.ps1](documented-restore.ps1)：按新文档顺序把candidate移到具名失败位置、backup搬回root、owned marker移到具名保存位置，再 `verify --profile core`。只增加literal-path与自有scratch绝对路径检查，不改变文档操作。没有删除用户数据、放宽校验或把人工无marker目录当backup。

- 恢复 **verify=0 / 640文件**，manifest字节仍等于旧版本；owned marker在root外具名保存，root内没有marker。
- 恢复的旧CLI实际运行Unicode空格路径脚本，输出 **`review-r3-restored-ok`**，exit0。
- 向恢复root加入陌生 `stranger-review-r3.txt` 后 **verify=2**，具名extra；把该文件移到自有保存位置后再次 **verify=0 / 640文件**。

见 [实际PowerShell恢复](documented-powershell-restore.log.txt)、[旧命令](restored-old-command.log.txt)、[严格extra负控](restored-strict-extra-control.log.txt)、[移除后复绿](restored-strict-green-again.log.txt) 及results的 `restore-integrity`。

新full永久 `documented_restore` 已核完整数据流：真实upgrade→实际backup→marker存在断言→保存marker→verify→旧脚本输出断言，并挂入FLR-6；work原始full实跑日志/receipt核对有效。本轮Core实际PowerShell往返验证与full共用、**未变的installer/校验机制**相互覆盖；不重复冷full安装或完整14项发行矩阵。

## 覆盖诊断、AC与SD

连续两次final非通过的诊断已读取、对照实际调整验证：

1. Cargo父环境/先解析再注入曾掩盖真实入口；现在直接运行干净环境CLI，永久测试本身无注入，并加入release/非仓布局控制。
2. fresh安装曾替代了文档恢复；现在由真实producer创建带marker backup，实际按文档处理并校验，陌生文件拒绝保持。

本次没有剩余in-contract finding、必需oracle gap、未批准延期或workaround。jet/ark真实构建仍按用户r3裁定转v0.7环境门控，未标为本轮实测成功。

| AC | final review结果 / 证据来源 |
|---|---|
| AC-01 | pass：R2已闭Core来源/闭包/独立构建安装/管理语言消费与原单源审查保持 |
| AC-02 | pass：F03真实默认发现闭合，R1/R2命令/API/JSON/stdin/helper/生命周期有效证据保持；协议/owned执行未改 |
| AC-03 | pass（r3）：开发直接发现、full安装邻接生成本轮通过；六profile、Vue/Rust/vscode构建及VM运行按未变输入保留，jet/ark延期披露保持 |
| AC-04 | pass：F02保持关闭，F04文档恢复与严格额外文件控制本轮闭合，其余未变事务/取消/重定位证据保持 |
| AC-05 | pass保持：R1独立29refs/23tags恢复、legacy和迁移来源未改 |
| AC-06 | pass：R1原件删除/资源/测试ID/CI归属、R2独立Core档及bootstrap接线保持；F03/F04生产和文档终稿已核验 |
| AC-07 | **pass**：本轮final独立验收/SD/遗漏/健康审查完成，绑定上述冻结提交 |
| AC-08 | **pending，owner=merge**：不计入本次pass已执行范围 |

SD-01的current_exe开发发现及marker恢复现状与真实行为一致；SD-02管理owner/runtime/compat边界、SD-03 CLI/Core与中立执行设施、SD-04保全/历史、SD-05组合安装/恢复、SD-06协议/SDK单源均通过。九个 `docs/specs/...` 目标、supersedes/new列表和GOAL-003/007已核；当前主检出同目标的差异为本Plan待沉淀内容，无观察到需覆盖的并发规则冲突。冻结patch/hash是后续merge输入，尚未发布canonical或ledger。

健康检查：当前full定向构建成功、永久单测1/1、两改动Rust文件rustfmt=0、两仓repair diff-check=0，生产Python AST通过。原已归因的上游/迁移前warnings不冒称零警告；本次无新增未处理告警、调试打印、资源删除或测试选择面缩减。Core源/installer/锁、语言生产代码与UI/down依赖没有本轮相关变化，已闭证据按这些等价输入复用。

## 下一阶段

Plan进入 **reviewed / 10/12**，T-10勾选，T-11/T-12继续pending。本轮没有跑裸t/触面整档、完整跨仓队列、完整14项、六profile或已闭Core/owner/legacy族；没有合入、归档或清树。

**pending_closeout_gates（merge-owned）**：先复核合入并集与本冻结输入的等价性，在冻结提交执行语言裸 `cargo t` 和实际适用触面档、Man `t`/`tcore`/`ti`及必要consumer/UI/跨仓与发行门；失败先归因，代码变化后复审受影响内容并重跑失效门。之后有序本地合入、三仓来源/版本读回、canonical Spec/受管ledger与索引、归档及逐树guard clean清理。`tf`只在主语言检出按到期规则单实例批量。上述事项完成前不得宣称整体交付。
