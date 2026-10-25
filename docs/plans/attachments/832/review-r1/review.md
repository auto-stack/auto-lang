# PLAN-832 r3 — T-10 独立终审 R1

2026-10-25。**outcome: needs_fix**，不是 final pass。当前合同 revision=3；jet/ark 真实构建转 v0.7 的用户裁定有效，本次不重新要求该环境面。

## 基线、独立性与范围

审查由当前 Codex 聊天独立重建，未参与该轮实现；未创建 agent 或第二权威 checkout，未虚构模型/会话 ID。读取主检出当前 Plan，在原实施树检查提交代码、运行定向反例和有效对照；实现未修改。

| 仓 | 权威树 / 分支 | inspected commit | diff base |
|---|---|---|---|
| lang | `D:/autostack/.wt/lang-832/auto-lang` / `codex/plan-832-dev` | `cd603e418accde3bb85327cc1b6affd49b2ce93f` | `45558ab628bb29a8719f8467046b3ae0d5e36d7a` |
| Man | `D:/autostack/.wt/lang-832/auto-man` / `codex/plan-832-dev` | `356f2c8186d07bc18567b18179c3cad7bda2d483` | `51fafe82e8e0c4e66325ee14088eed8c3270e07e` |
| UI | 同组 `auto-ui` / detached | `aee499e87093ff727f68ac8467c506e9ecf6ecdc` | 已交付固定依赖 |
| down | 同组 `auto-down` / detached | `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | 已交付固定依赖 |

进出审查四树 HEAD 均一致且 clean。主检出其他 session 的 WIP 保留。G832 的 815/816/831 全量归档、最终交付/清理记录及其 delivery→实施基面祖先关系独立核对，没有沿用 r1 并行开工规则。完整锁、Plan、Spec、技能与配置 hash 见 [baseline](baseline.json.txt)。

冻结输入：[Plan](plan-input.md)、[handoff](handoff-input.md)、[SD 材料](spec-delta-reviewed.md)、[实际 Spec diff](spec-delta-reviewed.patch.txt)。审查只生成附件与主检出 Plan/债务簿记，没有发布 canonical Spec 或修改 live ledger。

## Findings（均为当前合同内 OPEN 项）

### F832-R1-01 / P1 — Core 的无 UI 源码独立构建和安装尚未闭合

- **规则 / AC / task**：§1、C1、V-01、AC-01/03 明确要求 Core clone/install 无 UI 源码、独立发行不靠邻仓 path；T-05/T-07/T-09。
- **实际入口**：Man `crates/auto/Cargo.toml:21,45` 等 optional path 仍直指邻仓 UI；根 workspace 还包含 UI compat 成员。Cargo 在关闭 feature 后依然需要读取这些 manifest。安装器 `scripts/man-install.py:294` 对所有 profile 无条件读取 UI/down HEAD，`:193` 对 Core 仍复制 UI LICENSE。
- **最小重放**：在原组运行 `python D:/autostack/auto-lang/docs/plans/attachments/832/review-r1/probe.py`。脚本从冻结树逐字复制真实 Cargo manifests/target entry 为输入快照，保留 lang、去掉 UI；无新 Git checkout。实际 `cargo metadata --offline --no-default-features --features core --format-version 1` 返回 **101**，缺失 `auto-ui/crates/auto-ui-compiler/Cargo.toml`。另只把安装模块的 UI 输入根指向不存在路径，直接调用真实 `install('core', ..., None, True)`，在 `git -C <absent-ui> rev-parse HEAD` 返回 **128**；skip-build 仅隔离来源准入，不替代构建结论。见 [原始结果](results.json.txt)、[manifest 反例](core-metadata-no-ui.log.txt)、[输入 hash](manifest-inputs.json.txt)。
- **预期 / 实际**：已有 fixed SDK 输入时，Core 不需要 UI 检出即可解析/构建/安装；实际两个入口均要求 UI。现有独立发行 driver 的 SRC-1 总是同时 clone 四仓，DST-2 只在四仓齐备条件下编 Core；整组搬迁仍保留兄弟 path，不证明缺 UI/邻仓的固定包消费。
- **有效对照 / 边界**：既有 Core 安装 643 文件 verify=0，Unicode+空格路径脚本输出 `review832-ok`；full 安装 6608 文件 verify=0。这些正控成立，不能代替缺 UI producer/clone 的 oracle。未声称 Core 二进制当前静态链接了 UI，也未要求 Core 提供 full UI 功能。
- **关闭 oracle**：在 UI 源码/工具缺席且 AUTO 根、旧 target、任意邻仓回退被剥离时，从固定可重建 SDK/source 包完成 Core 独立构建、安装和真实管理/语言命令；normal/build/dev 单源闭包、UI 请求具名失败同时保持。给 driver 增加该场景，不能把“四仓 clone 后 Core feature 编译成功”再次作为替代证据。

### F832-R1-02 / P2 — 无权查询 owner 时会误删 staging

- **规则 / AC / task**：C4、V-04、AC-04；T-07/T-09；安装承诺只清理确认死亡 owner，活跃或未知目录不动。
- **实际原因**：`man-install.py:249–256` 的 Windows `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` 返回空 handle 时直接判 False；`:278` 因而递归删除对应 staging，未区分“进程不存在”和 AccessDenied 等未知状态。
- **真实反例**：当前机器 `Get-Process -Id 4` 为活跃 `System`；实际 OpenProcess=0、GetLastError=**5 (AccessDenied)**。将该 owner PID 写入审查自有、合法 schema 的测试 staging marker，调用原函数后 `process_alive(4)=False` 且 `stage_survives=False`。无生产目录或其他进程被修改。见 [results](results.json.txt) 的 `unknown-owner-cleanup`。
- **对照**：同函数对当前可查询活跃 PID 保留 staging，对不存在 PID 2147483647 回收 staging，均符合预期。问题仅在未知/权限失败分支；不据此否定此前已验证的普通取消、swap 回滚正控。
- **关闭 oracle**：证明死亡才允许回收，权限不足或其他不确定错误保留并具名报告；本轮实际 AccessDenied、可查询活控、确定死亡控均应通过。检查同一 helper 的取消恢复调用点即可，不需要为这个 finding 重跑六 profile/全量档。

### F832-R1-03 / P2 — 新 Man CI 入口仍依赖旧单仓二进制落点

- **规则 / AC / task**：§5.3、V-06、AC-06（及 AC-02 的真实工具接线）；T-08/T-09。适用本地 CI 入口必须承接测试，不能只成功列举 323 个 ID。
- **实际入口**：Man `.cargo/config.toml:11` 的 `ti` 选择 `plan816_cli_driver`；lang `scripts/cross-repo-tiers.py:85,123–148` 加入 man-ti 但只继承环境，没有构建/绑定其他 owner 的 helper。迁移测试 `crates/auto/tests/plan816_cli_driver.rs:985` 仍寻找 Man `target/debug/auto-ui-tool.exe`，UI 工具实际归 UI 仓。`:604` 的 te-child 默认定位也仍是 Man target，而设施归 lang。
- **最小重放 / 结果**：清空 AUTO_*、TE_CHILD_BIN、CARGO_TARGET_DIR/RUSTC_WRAPPER 后，在 Man 执行 `cargo test --locked -p auto --test plan816_cli_driver gen_runs_through_real_tool_process -- --exact`，**选 1 败 1 / exit101**，panic 为 Man target 下工具不存在。UI 固定工具已真实在位；仅设置 `AUTO_UI_TOOL_BIN=<同组UI>/target/debug/auto-ui-tool.exe` 后，同源码同一测试 **1/1 / exit0**。见 [定向结果](targeted-results.json.txt)、[失败原文](ci-gen-no-overrides.log.txt)、[合法对照](ci-gen-explicit-tool-control.log.txt)。未运行 ti 整档。
- **原因 / 影响**：工具跨仓后测试路径未承接，T-06 手动 env 验证与 T-08 list 并未验证声明的无隐藏配置 CI 入口。不是 UI 产品生成失败；也是 merge 冻结门前应完成的 work 接线，不应以未来门禁失败代替修复。
- **关闭 oracle**：受管 bootstrap/编排或迁移测试按 owner 解析真实 UI tool、lang te-child，明确构建前置；干净环境通过该生成反例及受影响 reader/helper 场景，缺件仍明确红且不命中旧主检出。完整 ti 留给最终 pass 后 merge 阶段，不要求现在重跑整档。

### F832-R1-04 / P2 — 目标仓仍把旧 am 安装/发行说明作为当前入口

- **规则 / AC / task**：§5.3、AC-06、T-08/T-09：命令/安装/版本及恢复资料应可从目标仓找到。
- **实际文件**：Man `README.md:22–32`、`README.cn.md:23–32`、`CLAUDE.md:24–34` 仍引导 `cargo install --path .`、声称安装 `am`；`release.ps1:3` 仍打包 `target/release/auto-man.exe`。迁移后根是 virtual workspace，正式命令是 `auto`。目标 docs 无本次 fixed sources/Core/full/安装交换恢复交接页；新材料仅在语言仓附件。
- **实际重放**：在 Man 执行 README 原命令 `cargo install --path . --offline`，**exit101**，`found a virtual manifest ... instead of a package manifest`。不是网络或工具链缺失。见 [log](documented-install.log.txt)。
- **关闭 oracle**：更新目标仓当前 README/开发入口，明确旧说明的 legacy 身份；提供 fixed 源获取与 Core/full 真安装、所需工具前置、升级/备份/恢复操作和版本来源入口。旧 release 脚本须显式退役或承接新交付，不能继续被当作当前可用入口。不要求本轮远端 push/registry 发布或提前完成 merge-owned three-repo-release。

## 全 AC / 风险矩阵

| AC / V / tasks | verdict | 独立证据与保留边界 |
|---|---|---|
| AC-01 / V01 / T04–05 | fail | 9 包仓属/中立与 UI 零复制、185 项实际文件在位成立；Core 安装 verify/脚本正控成立；缺 UI 来源入口失败 F01 |
| AC-02 / V02/03/06 / T05–06 | partial | 遷移源码逐项 hash；真实生成显式 tool 正控1/1、runtime 非 UI 集测2/2；旧命令/生命周期证据按不变源码保留；Core 及默认 CI 接线未全闭合 |
| AC-03 / V03 / T06–07 | partial | 792 receipt 37条均 ok；六 profile 生成、required Vue/Rust/vscode 构建、Rust/VM 运行原始逐项结果与实际 driver 断言核对；r3 jet/ark env-gated 不算真实构建通过；F01 独立 Core 输入未闭合 |
| AC-04 / V04 / T07 | fail | full/Core 内容 verify、普通失败/交换/取消历史证据保留；unknown owner 实际删除反例 F02 |
| AC-05 / V05 / T02/04/07 | pass | 本轮从原 bundle 独立恢复29 refs/23 tags/tree `4017c1c84e5b66370953266765de2e7f6f968437`，见 [recover](legacy-recover.json.txt)；legacy主分支未变、旧新来源逐项核对 |
| AC-06 / V06 / T08–09 | fail | 9包/6fixture语言原件确已删除、normal owner清楚；323 ID列表和24/122/4原始列表核对；CI真实路径F03、目标说明F04未闭合 |
| AC-07 / T10 | fail | 本次独立审查已执行，未达到 final pass，不能勾最终验收 |
| AC-08 / T11–12 | pending (merge-owned) | 未运行统一档、未合入/写ledger/归档/清树 |

迁移 source-audit 的185个**检出文件** hash 全部吻合，语言原件全无。额外对 Git blobs 比较发现 `cargo_project/{main.at,pac.at}` 两项 LF/CRLF 转换：173 个 same 的物理检出口径与 Git 原始字节口径有差别，但正文语义一致，无文件丢失；这两条不列作实现 finding，终稿应说明哈希口径。

## SD / 遗漏 / 健康审查

| delta | 结果 |
|---|---|
| SD-01 | partial：owner/profile/r3延期范围准确；独立无UI消费及unknown owner保护的无条件现状表述被F01/F02反例推翻 |
| SD-02 | owner迁移、runtime/compat区分成立；固定独立消费表述需随F01重新验证 |
| SD-03 | execution中立/client迁Man成立；当前CI/命令接线完整声明须随F03/F04纠正 |
| SD-04 | legacy保全和迁移来源成立；local默认分支实际落地仍归merge，不能提前声明完成 |
| SD-05 | partial：普通staging/hash/swap对照保留；完整失败/恢复表述须等F02闭合 |
| SD-06 | 协议/SDK单源与无反向生产依赖成立；不要以四仓sibling路径消费外推固定独立Core验收 |

九个目标路径均有效，GOAL-003/007 与 metadata 对齐；当前主检出这些 Spec 与实施分支 diff 为本次准备的 SD，未观察到需要覆盖的并发 Spec 规则变更。Spec 候选 hash/patch已冻结，未批准发布。

遗漏扫描覆盖包/资源/fixtures、normal/build/dev、脚本入口、CLI工具发现、安装准备/校验/交换/回滚/取消回收、历史 refs/默认分支、测试/CI与目标文档。F01..04是原合同缺口，登记债务不构成延期授权；r3批准的jet/ark延期保持原样。

健康检查：lang diff-check通过，Man新增脚本/manifest/config diff-check通过；Man九个golden EOF空行是原测试字节保留例外。定向Rust测试编译成功；日志仍有原lang-core/UI/runtime/CLI告警，移动源hash未变，不冒称零警告。全workspace fmt检查被未改的路径依赖/历史example格式与 `auto-lang/examples/ui_container.rs:195` 原有未闭字符串阻断；未批量格式化/顺手修改实现，不将它误报为迁移新缺陷。所有新生产 Python 脚本 AST 解析通过；源迁移无新增调试输出。

## 收口与下一步

恢复 `status: executing`；重开 **T-05、T-07、T-08、T-09**，保留 T-01/02/03/04/06 的有效完成事实，`current_step=5/12`。T-10未通过，T-11/12未执行。下一责任角色为同树 work；一次修齐 findings，补本轮反例与代表兄弟/有效对照，提交新冻结 SHA 后做受影响复审。不得在 review 内修实现。

证据复用仅覆盖源码、依赖、配置与输入未变面；F01来源/布局改变可能影响profile投影与full安装，work应明确受影响边界。F02只要求owner状态/清理反例与合法控制；F03只要求owner工具/helper定向接线；F04按真实文档步骤验证。无需为每个finding重跑六profile、整仓或独立发行冷矩阵。

pending_closeout_gates：最终code/AC/SD pass以后，由merge在新冻结提交统一执行语言裸 `cargo t` 与实际并集触面档、Man适用 t/ti及必要consumer/UI门；完成有序本地合入、来源读回、Spec/ledger/索引、归档与guard清理。`tf`只在主检出按到期规则单实例批量。本轮均未执行，也未声称交付完成。
