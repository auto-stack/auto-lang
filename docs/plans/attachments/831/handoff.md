# PLAN-831 T-09 最终来源核查与候选快照（修复轮刷新版）

日期：2026-10-24（同日 F831-09-01..06 修复轮闭合后刷新）；Plan revision：2；scope：work/T-09，非 T-10 独立终审。

本快照四份材料的内容哈希及三仓输入清单见 [freeze.json](freeze.json)。首轮核查快照结论为 needs_fix（六缺口）；本轮在同组原树完成全部修复与定向验证后重冻结。**候选结论：T-09 work 面闭合，六项 finding 全部 CLOSED，可送 T-10 独立终审**；merge-owned 门显式 pending。Plan 状态 executing（T-09 完成后由簿记推进）。

## 1. 唯一来源与可重建输入（修复轮终态）

| 角色 | 权威树 / 分支 | 完整 SHA |
|---|---|---|
| 语言实现 | `D:/autostack/.wt/lang-831/auto-lang` / `codex/plan-831-dev` | `1635835d377bd3faf12bb9ab642e14cebc150f94` |
| UI 实现 | `D:/autostack/.wt/lang-831/auto-ui` / `codex/plan-831-dev` | `464569d58d9d`（完整 `464569d` 见收据 sources） |
| down 只读依赖 | `D:/autostack/.wt/lang-831/auto-down` / detached | `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` |
| 搬迁比较基线 | 语言仓 T-04 source | `e199caf8a6cb369b320c02ca144c1e88b7bcef8f` |

两实现树 `git status --short` 干净（唯一未跟踪=compile-probe 本地 Cargo.lock，fixture 惯例不入库）。默认分支未推进；未修改 canonical Specs 或 ledger；本轮修复全部在 plan 分支提交（lang 7 提交：77114ec6f/5339085ea/b90374988/70ea2a462/de4af104c/0d1d8690a/1635835d3；UI 4 提交：15e3b14/fb5a2f4/071396e/464569d）。

本期渠道仍是本地 Git commit/源码包。重新消费本候选必须 checkout 上表 SHA 并核对各锁与源码包哈希；收据身份经 [F831-09-05 admission](#5-本轮发现及闭合条件) 核对（RCPT-ID/RCPT-ID-F 双 pass：三收据 admitted 于上表身份）。尚未生成正式 `ui-release.md`，不得称已发布，未推远端。

## 2. 净迁移对账

以 `git ls-tree -r e199caf8a -- crates/<五包>` 对目标 UI 迁移基线（2ea9b5a）逐 blob 比较：841 原文件、830 同 blob、11 变更、1 新增、0 丢失（明细与语言原件删除事实见首轮快照与本轮 [tier-parity](tier-parity-f831-09-04.json)）。**本轮 T-09 修复在其上继续演化**（node_converter 化石删除 534 行、LICENSE 新增、native_dnd/fastrand 修复、vscode 模板修复、nextest 配置新增）——这些是迁移后的正常演进，不改变 T-04 迁移恒等事实；正典测试选择面经基线对拍证明零丢失（§3 V-06）。

留语言仓：SDK/轻数据、project-model、API adapter、旧 auto-lang 组合兼容、Man 域、examples/ui、capability-tests、blueprints、a2gd 语料和跨仓消费者探针。旧 UI legacy 退出当前 workspace；保全事实沿用 [preservation.md](preservation.md)。

## 3. 全部 V 与 AC 核查（修复轮终态）

| V / AC | 终态 | 闭合证据与残余边界 |
|---|---|---|
| V-01 / AC-01 | **pass** | 五包在 UI、闭包 18 包零 Man/monolith 禁边、语言原件删除；777 strict exit0 **0v/5gf 且零警告**——ui_to_language 围栏真实化（362 文件真扫描+code_fence_zero；修复前 pattern 退格转义永不命中+源根迁出=空转绿双缺陷）；编译探针 nominal 类型身份/SDK 轻数据流入 UI（convert_node）+双 source 解析层拒绝；DST-4 隔离 SDK 反控终源复验（5/5）。 |
| V-02 / AC-02 | **pass** | 16 条 auto_lang 旧路径真实编译（TYP-PROBE=pass）：构造/pub 字段（Parser scope/db/cur）/回调安装（set_ui_host_install_hook）/重入口 fn-item/模块面 pub 项；三 feature 组合（default 全量含运行断言/core-only/core+显式 UI）按真实门矩阵分面（iced/interpreter 门子集如实记录）；i18n_lookup 新旧路径函数指针同一。C2 定义单源（D01..06）保持。 |
| V-03 / AC-03 | **pass（具名环境边界）** | 792 收据 38/39 终源绑定：V01/V02 fresh/locked/重定位构建、V05 工具随包消费、**V06 生成 Rust 应用真实窗口 run（句柄 31262416）+六 profile 真实生成+vue/rust/vscode 真实构建链 outcome=completed**；jet/ark 构建=【环境阻塞·具名】红（本机无 gradle/Hvigor——合同要求显式记录非静默跳过）；发行组 clone/fresh/locked/整组重定位终源重跑 5/5。run 宿主域面（vue/jet/ark dev 服务器、tauri/vscode CLI）按 v06_dispositions 逐项具名，不伪称已运行。 |
| V-04 / AC-04 | **pass** | 792 坏 pin/漏 patch/资源四态/旧字节/陌生目录逐 case 绿（终源收据）；**V06d 真实安装中取消+重试恢复**（pnpm 链中途树杀→重发同请求 outcome=completed）；drill 19 检终源重跑（收据含工具源仓身份）；收据身份 admission 使错源/漂移收据确定性红（selftest S17 三臂）。 |
| V-05 / AC-05 | pass（历史+终态） | T-02 保全 verify/recover-drill 载荷不变；drill 终态 19/19；真实仓 prepared/land/switch 归 T-11 合同窗口（本 Plan 设计归属，非缺口）。 |
| V-06 / AC-06 | **pass** | **正典 ID 对拍**：基线（e199caf8a nextest/libtest 列举）vs 当前——t 面三共享包逐 ID 相等（compiler 247/dialect 86/runtime 1872）、tools+shell-pack 为原档未选的超集、tu 面 863 集合恰等（基线 867=863+双 #[test] 重复注册，已修）、ti 集成档实跑 28/28、tn native 门 2 测试可列举；777 双根围栏；跨仓编排入口（cross-repo-tiers.py）落地；ti/tu/t 语义与机器闸门归属见 ci-entrypoints。V06e vm 解释运行+三族寿命冒烟（窗口/终端/媒体）。 |

fixture 全族终态：`--phase source-switched` = **46 pass/0 fail/1 recorded/1 pending**（recorded=SRC-P4c 清洁面复核"无残留"；pending=TYP-PROBE 零 cargo 缺省——专项 `--compile-probe` 跑 24/24 在案）；selftest 19 检（含 S17）。

## 4. 真实支持面与发行边界（修复轮终态）

| 面 | 终态证据 |
|---|---|
| UI 源码组 | 终源 5/5（fresh 3s/locked/重定位/隔离 SDK 103s）；组布局成立 |
| 外部 Rust bundle | V01/V02 终源绿 + **V06a 真实窗口 run**；api-gen 双根修复使直接生成形态（非交付 bundle）亦可构建（预验证 fixcheck.exe） |
| Vue | V05e 生成+资源字节=源；V06c vue outcome=completed（真实 pnpm 链）；V06d 取消恢复 |
| VM/Iced | V06e run_ui 解释运行；三族寿命冒烟（001 窗口/041 终端/020 媒体——150s 真实媒体扫描窗）；T-07 MCP 冒烟于终源重放全过 |
| jet / ark | 生成真实（24/25 文件）；构建【环境阻塞·具名】红（gradlew 9009/Hvigor 缺席）；不删除支持、不记 env skip 为 pass |
| tauri / vscode | tauri 生成真实（vue 壳+gen/back/rust；无 back 纯 UI 工程被具名拒绝=合法前置）；vscode 生成+构建 outcome=completed（PM 混用修复后）；run 面宿主域具名 |
| autodown / mpv | down 固定 c39ac7f 在场（收据齐记）；mpv/native 扩展 feature 门未开（原门控不强制，未触面如实记录） |
| 许可 | UI 根 LICENSE（MIT 逐字复刻语言仓）+五包 manifest MIT+shadcn 上游 LICENSE 随资产；[warn-license-parity](warn-license-parity-f831-09-06.json) 对拍：迁移零新增告警 |

## 5. 本轮发现及闭合条件（全部 CLOSED）

| ID | 严重度 | 闭合证据（修复轮） |
|---|---|---|
| F831-09-01 | P1 | 前轮闭合（UI 15e3b14：单源输入布线修复，示例/语料/blueprints/共享 workspace 切组内兄弟；UI workspace 3260 面布线红全清） |
| F831-09-02 | P2 | **CLOSED**：compile-probe 实编落地（lang b90374988）——16 路径×三 feature 组合+类型传递/单源身份运行断言+双 source 解析层拒绝；TYP-PROBE=pass（收据 t09-f02-type.json 24/24） |
| F831-09-03 | P1 | **CLOSED**：V06 面（lang de4af104c/0d1d8690a/1635835d3 + ui 464569d）——真实 run/生成矩阵/构建矩阵/取消恢复/vm 运行；暴露并修复 api-gen 双根路径与 vscode PM 混用两处产品缺陷；jet/ark 环境阻塞具名红；三族寿命冒烟（[run-profile-evidence](run-profile-evidence-f831-09-03.json)） |
| F831-09-04 | P1 | **CLOSED**：UI t/tu 语义对齐+ti 集成档（28/28 实跑）+tn 手工门+`.config/nextest.toml`；777 跨仓围栏真实化（含 pattern 退格转义与字符字面量状态机两处扫描器缺陷修复+基线重采样 0v/5gf 零警告）；cross-repo-tiers.py 编排入口；迁前后正典 ID 对拍（[tier-parity](tier-parity-f831-09-04.json)）；node_converter 化石清理（534 行从未编译死区、围栏仅存 code 命中） |
| F831-09-05 | P2 | **CLOSED**：admit_receipt 机制（lang 70ea2a462）——三执行载体 sources 齐记（792 补三仓/distribution 补 down/drill 记工具源仓）、DST/FLR 绑定前置 admission、S17 突变负控；终源重跑后 RCPT-ID/RCPT-ID-F 双 pass（旧收据 c765211fd/无身份态在 admission 下如实转红并已全部换终源收据） |
| F831-09-06 | P2 | **CLOSED**：UI 根 LICENSE 补入（T-04 台账"沿用旧 LICENSE"不成立已证并修正）；双 #[test] 修复（重复注册消除=tu 面 863 恰等）；fastrand dev-dep 泄漏修复（tn 门可编译）；告警对拍 65→64 逐类归因、迁移零新增（[warn-license-parity](warn-license-parity-f831-09-06.json)） |

附带发现（本轮工具链事实，非 Plan 缺口）：PATH 解析命中主检出陈旧 auto.exe（缺 T-07 output_root 锚）/auto-ui-tool.exe（缺 816 run_ui 面）——主检出未合入本 Plan 分支的既知状态；实机入口须钉工作树二进制（ci-entrypoints 已注记）。

## 6. SD-01..07 与交接条件

逐条草案见 [spec-delta.md](spec-delta.md)（本轮未修订草案文本；其涉及的事实面变化——根 LICENSE 在位/档位语义/777 双根围栏/收据身份——已记录于本快照与 ci-entrypoints，T-10 审核时对账后 T-12 沉淀）。

下一责任角色：**T-10 独立终审**（独立上下文，绑定本快照三仓 SHA 审核 AC/SD/反例/收据身份/净迁移）。pass 后语言裸 cargo t 及实际触面档、UI 适用门由 T-11 在冻结输入串行执行（本轮未运行任何整档 t/tv/tt/tb/tu/taa——合同后置，显式 pending）；tf 仅主检出批量到期档。T-11 本地有序落地/ui-release.md/来源读回、T-12 Specs/ledger/归档/guard 清理全部 pending；U831 未成立，832 不得开工。
