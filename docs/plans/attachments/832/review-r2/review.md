# PLAN-832 r3 / T-10 final R2

2026-10-25。**needs_fix**。F832-R1-01/02 **CLOSED**；F832-R1-03/04 的旧示例已修，完整不变量仍 **PARTIAL / OPEN**。r3 的 jet/ark 延期有效，不重新要求该环境面。

## 审查基线与独立性

沿用 R1 审查聊天，未参与实施或本轮修复；从提交、代码和实际输出独立重建结论。未创建 agent、第二权威 checkout，也未修改实现。

- lang：`D:/autostack/.wt/lang-832/auto-lang`，`codex/plan-832-dev`，HEAD **`87f82d637ecd586c17a01c1c975e1456bc346991`**；repair base=`cd603e418accde3bb85327cc1b6affd49b2ce93f`。handoff 的生产 CI/Spec 基线 `0fd092bdccf91902bd7744b4024929b63ddd1c4b` 之后仅交接/证据变化。
- Man：同组 `auto-man` / `codex/plan-832-dev`，HEAD **`ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f`**；repair base=`356f2c8186d07bc18567b18179c3cad7bda2d483`。
- UI=`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，同组 detached 固定输入。
- 四树进出均 clean、HEAD 未漂移；主检出其余 WIP 保留。锁/代码/二进制/技能/Spec 指纹：[baseline](baseline.json.txt)。冻结 [Plan](plan-input.md)、[handoff](handoff-input.md)、[work closure](repair-input.md)、[SD](spec-delta-reviewed.md)、[实际候选diff](spec-delta-reviewed.patch.txt)。

本轮仅复查四个 finding 的改动机制、代表兄弟状态和合法对照。未复跑六 profile、完整发行14项、全driver/全consumer或整档；旧证据复用边界见后表。

## Closure 与剩余 finding

### F832-R1-01 / P1 — CLOSED

独立 Core workspace/lock/target 以 `crates/auto-core` 的 target path 复用 `crates/auto` 源码；不再解析 full/UI manifest。Core 安装来源、许可证和metadata仅取 Man/lang，shared targets 有清单记录。

复用 work 的真实 no-UI fresh clone/冷构建/重定位原始日志，已检查调用、断言和3/3、1/1的实际输出；没有把14项全集标作本轮重跑。复查者使用已有的无UI/down重定位 fixture，确认当前 installer/Core manifests/core lock/main/discovery 字节与 fixture 相同，详见 [等价输入](supplemental.json.txt)。当前语言生产源码从 R1 至今无diff，后续仅CI/Spec/证据变化。

独立实跑：无UI/down fixture 的 Core locked/offline metadata=0；真实安装 **640文件**、verify=0；Unicode空格路径脚本输出 `review832-core-r2`；`auto new demo` 真实产生 `pac.at` 和 `demo.at`；`auto ui list` exit1、明确要求 full/ui-extra。当前权威 Man 树的 Core `cargo check --locked --manifest-path core/Cargo.toml -p auto --no-default-features --features core` 通过。见 [结果](results.json.txt)、[管理命令](core-management.log.txt)、[check](core-check.log.txt)。本次安装使用等价 fixture 已构建二进制和 `--skip-build`，没有冒称又执行冷构建；冷构建证据来自上述已核验不变输入。

full 修复安装6608文件和安装态生成的 work 结果有效；本轮额外清环境运行安装 full 的邻接工具生成也成功并落真实 `App.vue`，见 [发现矩阵](discovery.json.txt)。

### F832-R1-02 / P2 — CLOSED

`process_alive` 改为 True/False/None；Windows ERROR_INVALID_PARAMETER 归死亡，其余无法确定的查询归 None，回收只删 `is False`，unknown 具名保留。

R1 同一实际反例：System PID4 返回 None，staging保留并输出 `owner liveness unknown`；当前活跃PID=True并保留，不存在PID2147483647=False并删除；注入 unknown 保留且具名。见 [results](results.json.txt) `owner-*`。PID0无可用安装owner，本轮记录结果但不据此增加产品要求。未重跑普通取消/全发行矩阵，未变交换/自动回滚证据保留。

### F832-R1-03 / P2 — PARTIAL / OPEN：Cargo 测试通过，直接 CLI 仍无法发现开发工具

**既有规则 / tasks**：C2、V-03/V-06、AC-02/03/06；T-08/T-09。要求开发首次使用、安装及匹配工具发现按实际owner工作。不是新增自动下载或任意目录扫描承诺。

**已关闭部分**：原 `cargo test ... gen_runs_through_real_tool_process -- --exact` 在清 AUTO 环境后 **1/1通过**；`timeout_exit_reaps_own_readers_before_return` **1/1通过**，证明lang helper路径修复有效。跨仓 `child_env/man_bootstrap` 已按owner注入并预构建；`tcore` 新选择面及work的23/23、discovery6/6等证据核对有效。

**绑定输入 / 实际入口**：[dev-discovery.py](dev-discovery.py) 构造合法 `src/front/app.at` 和 `pac.at`，重建当前full CLI后，直接启动 `auto-man/target/debug/auto.exe gen`；子环境剥离 AUTO_*、TE_CHILD_BIN、CARGO_TARGET_DIR、RUSTC_WRAPPER、**CARGO_MANIFEST_DIR**，固定UI工具已在同组 UI owner 的 target 中。

| 对照 | 实际结果 |
|---|---|
| 直接开发CLI，无覆盖/无Cargo环境 | **exit1**，无 `App.vue`，仅尝试 Man `target/debug/auto-ui-tool.exe` |
| 同合法输入，仅显式 `AUTO_UI_TOOL_BIN` | exit0，真实 `App.vue` |
| 仅恢复 `CARGO_MANIFEST_DIR=<Man>/crates/auto` | exit0，真实 `App.vue` |
| 仅显式 `AUTO_UI_ROOT=<同组UI>` | exit0，真实 `App.vue` |
| 显式缺失tool，即使真实owner工具在位 | exit1、具名缺件、无 `App.vue`（负控成立） |
| 独立full安装、邻接tool，无Cargo环境 | exit0，真实 `App.vue`（安装控制成立） |

原始矩阵：[discovery.json.txt](discovery.json.txt)，[生产失败](production-clean.log.txt)，[显式tool正控](explicit-tool.log.txt)，[Cargo环境对照](cargo-manifest.log.txt)。全部使用同一合法工程结构，不是语法/前端目录错误导致的失败。

**已确证原因**：`crates/auto-tool-client/src/discovery.rs:269–283` 仅从显式UI根或**运行期** `CARGO_MANIFEST_DIR` 推断owner；直接运行的CLI没有此变量。`crates/auto/src/tool_ui_ops.rs:76` 使用默认spec、不传开发锚。修复后的测试在Cargo环境中先 `resolve_existing`，然后在 `crates/auto/tests/plan816_cli_driver.rs:996` 将解析结果注入被测CLI的 `AUTO_UI_TOOL_BIN`，所以1/1没有覆盖普通进程发现；CLI最终只剩邻接Man target候选。

**关闭 oracle / 范围**：同组开发产物真实在位时，实际CLI在清Cargo/AUTO环境下按承诺找到UI owner；保留显式覆盖优先、缺件具名拒绝、安装邻接及不搜索任意主检出的控制。新增或改造永久回归须让CLI自己发现，不能在父测试中先解析再注入工具来证明默认生产路径。只复测这些发现入口和代表命令即可，不重跑六profile/整driver。

### F832-R1-04 / P2 — PARTIAL / OPEN：新恢复文档没有处理 backup marker

**既有规则 / tasks**：§5.3、V-04、AC-04/06；本finding的文档/交接修复归 T-08/T-09。真实备份应有可行、准确的恢复步骤。

**已关闭部分**：中英文首页/CLAUDE/current install说明已迁到 `auto` 与Core/full managed installer；旧am文档明确legacy；`release.ps1` 独立实测exit2、明确retired。旧的根 `cargo install --path .` 不再是当前指导命令。

**实际重放**：[replay.py](replay.py) 在审查自有Core安装上，用当前同字节installer完成正常安装与升级，获得真实 `backup_root`；两次640文件校验均通过。随后执行新文档 `docs/installation.md:55–66` 所述的同卷重命名步骤：当前candidate移到失败目录，真实backup移回原root，再执行文档要求的verify。重命名用 `os.replace` 执行，与所述目录Move-Item操作等价；没有改写备份字节。

**预期 / 实际**：恢复后verify=0；实际 **exit2**，`missing=[], extra=['.man-backup.marker'], changed=[]`，见 [恢复失败](documented-restore-verify.log.txt)。将这唯一标记移到自有scratch保存、其余字节不动后，同一verify立即 **exit0 / 640文件**，见 [因果对照](restore-marker-removed-control.log.txt)。数据本体完好，未声称备份丢失。

**已确证原因**：`scripts/man-install.py:359` 成功升级为backup加标记；`:224–250` verify将其纳入额外内容；`:369` 仅**自动异常回滚**移除标记。新手工恢复步骤直接搬回backup，没有对应处置。Core/full共用该备份/校验机制。

**关闭 oracle / 范围**：让受管恢复操作或准确的文档步骤处理本次owned backup元数据，真实“升级→保留备份→按文档恢复→verify→旧命令可运行”闭环通过，陌生文件保护和普通自动回滚保持。可用一个profile的真实往返覆盖共用机制并说明另一个profile的等价范围；若引入profile分支再检查相应分支。无需为文档恢复问题跑完整14项/统一门。

## 第二次 final 非通过后的覆盖诊断

本Plan连续两轮final needs_fix。下一次送审前，work需给出诊断与覆盖调整，先证明调整有效再重新冻结，不再以旧示例总数声明整个不变量关闭。

| 剩余不变量 | 先前检查漏过的原因 | 受影响路径 / oracle调整 |
|---|---|---|
| 开发CLI自身能按owner发现tool | Cargo父测试自带manifest环境，且先解析后向CLI注入路径 | 真实CLI子进程剥离Cargo环境；检查实际候选、产物；显式/安装/缺件对照保持 |
| 文档恢复可校验的旧交付 | work验证fresh/install/retired release，未按新文档从真实backup恢复 | 真实producer产生marker的backup，经文档操作返回root后verify与旧命令；不能只搬一个人工无marker目录 |

## AC / SD 与证据复用

| 项目 | 当前结论 / 依据 |
|---|---|
| AC-01 | pass：F01独立Core来源/闭包/构建/安装/管理及语言运行闭合 |
| AC-02 | partial：原CLI/helper/机器合同及未变生命周期证据保留；生产默认发现F03未闭合 |
| AC-03 | partial：Core交付与full安装/生成控制成立；六profile/build/VM不变面证据保留，开发启动F03仍失败；jet/ark r3延期不重开 |
| AC-04 | partial：owner保护F02闭合；旧受管事务控制保持，文档备份恢复F04仍未闭合 |
| AC-05 | pass保持：R1独立29refs/23tags恢复与legacy来源未改变，无需重跑bundle |
| AC-06 | partial：迁移owner/删除/资源和CI接线修复有效；默认发现及恢复文档的终稿仍不准确 |
| AC-07 | fail：T10本轮未获final pass |
| AC-08 | merge-owned pending；未执行 |

SD-01/03 的“组内UI owner默认发现”仍被真实CLI反例推翻；SD-01/05 的恢复材料不能以当前手工文档作完整恢复证据。独立Core描述、owner/单源/legacy范围、r3延期和其余SD-02/04/06可保留。候选hash/diff已冻结，未发布canonical或ledger；未借修复增加合同范围或改变revision3。

健康：定向full构建、Core check、两条Rust定向测试通过；生产Python AST、两仓repair diff-check通过。编译日志仍含R1已归因的上游/迁移前warning，未声称零warning；未做批量格式化或改实现。第一次发现探针错误地用了 `ui/front`，显式控制走到“src/front缺失”，这些原始记录保留在results但**不作为finding证据**；已改成生产支持的 `src/front` 与有效widget语法，以六格完整对照重新验证，未弱化断言。

保留R1未变source/资源/历史事实，保留work的真实冷Core构建/重定位、fulldriver7/7、tcore23/23、discovery6/6等已核验范围。变更过的发现和恢复操作不继承旧的总pass；本轮不重复无关UI冷构建/全发行、legacy或全回归。

## 路由

Plan回到 **executing 7/12**：T-01..T-07完成保留（T05/T07修复已核验）；只重开 **T-08、T-09**，T10未通过，T11/12仍pending。F01/F02不得在下一轮机械重开。

同树work修齐F03/F04剩余范围，记录上述覆盖诊断和调整、重放原例/合法对照、更新handoff/SD并提交后，再做受影响复审。完整统一门仍merge-owned：final pass后在新冻结输入统一执行语言裸t与实际触面、Man t/tcore/ti及必要consumer/UI门，随后合入/来源读回/Spec-ledger/归档guard；tf仍主检出按到期规则批量。本轮未跑整档、未合入、未归档/清树。
