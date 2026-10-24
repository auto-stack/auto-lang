# PLAN-831 T-09 最终来源核查与候选快照（R4 分类不变量收口版）

日期：2026-10-25（T-10 R4 needs_fix 后同组修复轮闭合）；Plan revision：3；scope：work/T-09，非 T-10 独立终审。

本快照四份材料的内容哈希及三仓输入清单见 [freeze.json](freeze.json)。历轮：R1 needs_fix（F03/F05 REOPEN、F01 PARTIAL、F-R1-01 OPEN）→R1 修复；R2 needs_fix（F03 progress-only 假绿+jet/ark 必需环境面）→R2 修复+r3 合同修订；R3 needs_fix（V06c 吞真实编译失败、V06e 超时当关闭）→R3 修复；R4 needs_fix（环境延期判定被无关路径字节误触发）→R4 修复。本轮完成 R3 两子面判据收口 + R4 分类不变量收口（见 §5 R3/R4 表，含"获准延期/真失败/未知探测"三类判据说明与环境缺席的 producer 模板锚定）。**候选结论：T-09 work 面闭合（review-r4 窄残项 CLOSED），可再送 T-10 独立终审**；merge-owned 门显式 pending。Plan 状态 executing（T-09 完成后由簿记推进 execution_done）。（历史注：R1 时 jet/ark 构建面表述为 named-partial，r3 用户裁定移出合同后该表述不再适用，见 §4 修订记录。）

## 1. 唯一来源与可重建输入（R4 终态）

| 角色 | 权威树 / 分支 | 完整 SHA |
|---|---|---|
| 语言实现 | `D:/autostack/.wt/lang-831/auto-lang` / `codex/plan-831-dev` | `538e3b8fefef4b651b138201234e7c6fa50bcbfa` |
| UI 实现 | `D:/autostack/.wt/lang-831/auto-ui` / `codex/plan-831-dev` | `425088520e402b1efae9479b9d0b5b3d44a56df4` |
| down 只读依赖 | `D:/autostack/.wt/lang-831/auto-down` / detached | `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` |
| 搬迁比较基线 | 语言仓 T-04 source | `e199caf8a6cb369b320c02ca144c1e88b7bcef8f` |

两实现树 tracked `git status --porcelain` 干净（lang 唯一 untracked=compile-probe 本地 Cargo.lock，可再生不入库；F05 admission 已明确 untracked 可再生产物不判脏）。默认分支未推进；未修改 canonical Specs 或 ledger；本轮修复全部在 plan 分支提交（**R1 新增**：lang 09307c05e+fe8f3090a、UI 4250885；**R2 新增**：lang 112167c58+2d47e236e；**R3 新增**：lang 3064b3465；**R4 新增**：lang 538e3b8fe（V06c 环境延期锚定 producer 具名模板）。完整提交链见 git log）。

本期渠道仍是本地 Git commit/源码包。重新消费本候选必须 checkout 上表 SHA 并核对各锁与源码包哈希；收据身份经 [admission](#5-本轮发现及闭合条件) 核对——R1 起 admission 含 tracked 脏树拒绝（同 HEAD 源码改动后旧收据不可采信），四收据均于上表身份 admitted（RCPT-ID/RCPT-ID-F 双 pass，"tracked 树干净"入收据明细）。尚未生成正式 `ui-release.md`，不得称已发布，未推远端。

## 2. 净迁移对账（R1 重算，撤回旧交叉引用）

**基线（e199caf8a ls-tree blob）vs 终 UI HEAD（4250885）逐文件比较：841 基线文件、769 同 blob、72 变更、0 丢失、1 新增**（明细与逐提交归因见 [source-audit.json](source-audit.json) migration/migration_attribution——72 变更=60 文件 F831-09-01 单源布线（15e3b14）+1 档位语义（fb5a2f4）+2 license/重复注册（071396e）+1 vscode PM（464569d）+2 R1 oracle/清理（4250885）+9 T-04 窗口机械改写（manifest/aura），跨轮重叠按首桶折叠）。

历史事实分层：T-04 导入窗口的 **841 文件逐字节恒等**（见 [import-ledger.md](import-ledger.md)）不受影响——上述 72 变更全部发生在导入窗口之后的 T-09/R1 修复轮，各有具名提交。首轮快照"2ea9b5a 830 同 blob"与 source-audit（当时绑定 461c609）的交叉引用不一致已由 review-r1 指出，本版以终态重算为准，旧表述作废。零丢失（0 missing）保持；语言原件删除与留仓清单不变。

## 3. 全部 V 与 AC 核查（R1 终态）

| V / AC | 终态 | 闭合证据与残余边界 |
|---|---|---|
| V-01 / AC-01 | **pass** | 同首轮终态（777 strict 0v/5gf 零警告真实扫描 362 文件、编译探针类型身份、DST-4 隔离 SDK 反控 5/5 R1 终源复验）；R1 增：blueprints 兜底断言恢复独立 oracle（见 §5 F01）。 |
| V-02 / AC-02 | **pass** | 同首轮终态（16 路径三 feature 组合 compile-probe 24/24、C2 单源 D01..06）。 |
| V-03 / AC-03 | **pass（r3 合同下）** | 792 收据 R3 终源 **39/39**：V01/V02 fresh/locked/重定位、V05 工具随包消费、V06a 真实窗口 run（句柄 28707468）、六 profile 真实生成、**vue/rust/vscode 真实构建链 completed（必需面全绿）**；jet/ark 构建全尝试逐项记录（缺环境=【环境门控·转后续】，r3 用户裁定移出合同、KNOWN-DEBT 承接，生成面已绿不受影响）。 |
| V-04 / AC-04 | **pass** | 792 坏 pin/漏 patch/资源四态/旧字节/陌生目录逐 case 绿（R2/r3 终源收据 39/39）；V06d 可区分 oracle（安装活动证据→中断→保持性→重试 completed+dist 落盘，真实链 PASS 19s）；drill 19 检 R2/r3 终源重跑；admission 含 tracked 脏树拒绝（selftest S17 五臂）。 |
| V-05 / AC-05 | pass（历史+终态） | T-02 保全载荷不变；drill 终态 19/19（R1 重跑收据）；真实仓 prepared/land/switch 归 T-11 合同窗口。 |
| V-06 / AC-06 | **pass** | 正典 ID 对拍不变（t 面三共享包逐 ID 相等、tu 863=863 ID 级差集空——865/867 对账：865=基线行数，867 叙述误计已撤回，当前侧 @4250885 实测 863 行）；ti 28/28、tn 门可列举；777 双根围栏；跨仓编排入口；V06e vm 真实运行=子应用服务级观测（Launched+探针+收口，见 §5 F03-B）。 |

fixture 全族 R4 终态：`--phase source-switched` = **46 pass/0 fail/1 recorded/1 pending**（收据 t09-r4-all.json；recorded=SRC-P4c 清洁面复核；pending=TYP-PROBE 零 cargo 缺省——专项 24/24 在案）；selftest 全过（含 S17 五臂）。

## 4. 真实支持面与发行边界（R1 终态）

| 面 | 终态证据 |
|---|---|
| UI 源码组 | R4 终源 5/5（fresh/locked/整组重定位/隔离 SDK）；组布局成立 |
| 外部 Rust bundle | V01/V02 终源绿 + V06a 真实窗口 run；api-gen 双根修复保持 |
| Vue | V05e 生成+资源字节=源；V06c vue completed（真实 pnpm 链，R2/r3 重跑）；V06d 可区分取消恢复 oracle PASS（19s） |
| VM/Iced | **V06e R2 终态：Launched(url) 子应用事件（vm-session MCP 实际端口）+端点探活 HTTP 200+协议 cancel 回执+终态 cancelled+同端点收口 ConnectionRefused(10061) 独立观测+宿主 describe 复验**（真实链 PASS 3s）；三族寿命冒烟（窗口/终端/媒体）沿用 |
| jet / ark | 生成真实（24/25 文件，R2/r3 终源重跑）；构建全尝试逐项记录、缺环境=【环境门控·转后续】（r3 用户裁定移出合同，KNOWN-DEBT 承接；工具链在场失败仍红） |
| tauri / vscode | tauri 生成真实；vscode 生成+构建 completed；run 面宿主域具名 |
| autodown / mpv | down 固定 c39ac7f 在场（四收据齐记）；mpv/native 扩展 feature 门未开（原门控不强制） |
| 许可 | UI 根 LICENSE + 五包 MIT + shadcn 上游随资产；warn-license-parity 零新增 |

## 5. 本轮发现及闭合条件

### R4 轮（review-r4 窄残项 CLOSED）

| ID | 严重度 | 闭合证据（R4 修复轮） |
|---|---|---|
| F831-09-03 A 窄残项（无关路径字节误触发环境延期） | P1 | **CLOSED**（lang 538e3b8fe）：`BUILD_ENV_ABSENCE_MARKERS` 泛化子串（含 `not found`）改为按 profile 锚定 **producer 具名工具链缺席模板**（jet=exit code: 9009/exited 9009/Failed to generate gradle wrapper/Please install Gradle or Android Studio/Gradle not found.；ark=Hvigor wrapper not found/open the project in DevEco Studio to build (no build was executed)——jet.rs:930/1191、ark.rs:763 原文）；路径/文件名/资源模块缺失/建议命令中的通用字串不参与 env 判定；未命中模板的一切失败传播红；非反例黑名单。**验证**：review-r4 replay（真实工具入口：hvigorw 控制包装写 marker 后 exit /b 1 → 真实 build_ui(ark) failed）两分类（路径含/不含 "not found" 字节）全红 exit 0；R3 build-matrix 五控制保持全符合同；真实链 V06c=jet/ark env-gated（具名模板命中）+required 全 completed。 |

### R3 轮（review-r3 两判据缺口 CLOSED，含三类判据说明，历史）

review-r3 要求下一交接说明"**获准延期/真失败/未知探测**"三类如何在判据中区分——当前判据（lang 3064b3465）：

| 类别 | 判据归属 | 处置 |
|---|---|---|
| 获准延期（r3 授权的环境缺失） | 仅限非必需面 jet/ark：失败且错误文本含工具链缺席标记（9009/not found/wrapper not found/open the project in） | 逐项记录【环境门控·转后续】不阻塞——r3 用户裁定移出合同，KNOWN-DEBT 经批准非阻塞延期承接 |
| 真失败（在场执行后的失败） | 无环境缺席标记的失败（编译错/协议错/会话异常）或必需面（vue/rust/vscode）失败 | **传播红**——V06c 判据=required 全 completed 且无 real_fail；review-r3 五控制中 jet Kotlin/ark ArkTS 编译错均红 |
| 未知探测（端点探测不可判定） | V06e `_mcp_probe` 三态之 unknown：超时/探测失败既不证明监听也不证明关闭 | 收口判定**仅连接拒绝**（ConnectionRefused/10061/积极拒绝）计为端点确证关闭；http/unknown 继续轮询、超窗诚实红——review-r3 endpoint-timeout 控制（仍监听不回答）判红且独立 TCP 取证证实判定真实 |

| ID | 严重度 | 闭合证据（R3 修复轮） |
|---|---|---|
| F831-09-03 A（V06c 吞真实编译失败） | P1 | **CLOSED**（lang 3064b3465）：汇总器改三类判据（上表）；review-r3 replay.py --case build-matrix 五控制全符合同（all_complete 绿/缺环境绿/jet 编译错红/ark 编译错红/required vue 错红）；真实链 V06c PASS（jet/ark=env-gated 逐项记录+required 全 completed）。 |
| F831-09-03 B（超时当关闭） | P1 | **CLOSED**（lang 3064b3465）：`_mcp_alive` 改 `_mcp_probe` 三态（http/refused/unknown），收口仅认 refused；review-r3 endpoint-timeout 控制：仍监听但不回答的端点→同端点收口=False 判红，工具清理前独立 `socket.create_connection` 同端口成功（监听者确在，判定真实非巧合）；真实链 V06e 收口=ConnectionRefusedError(10061)（3s）。 |

### R2 轮（review-r2 开放点全部 CLOSED，历史）

| ID | 严重度 | 闭合证据（R2 修复轮 + r3 修订） |
|---|---|---|
| F831-09-03 B 面（progress-only 假绿） | P1 | **CLOSED**（lang 112167c58）。**R2 要求的诊断**：R1 闭合把"任意操作事件"当接受证据，观测对象错置在事件通道——run_vm 在解释器启动前即发进度（run_vm.rs:292/:558 均为进入解释器前），"有事件"状态空间含"VM 从未启动"态，故合法 progress-only 载体假绿。**oracle 调整**：接受证据改绑只在子应用真实服务时存在的三层观察——①Launched(url)（vm-session 子进程 MCP banner 被 host 前转器嗅探发射，url=实际绑定端口，身份明确非任意 event 字串）；②端点活性探针（真实 axum 监听者对 POST /mcp 有 HTTP 应答，含 4xx/5xx 诊断应答）；③寿命收口=cancel 终态后**同一端点**停止应答（同会话同对象独立外部观测，取代 R1 版"owned 树回收后发出"的无独立依据表述）。**验证**：review-r2 replay-vm-progress 负控确定性红（无 Launched 点名）；R1 三反例保持红；真实链终源 V06e PASS（Launched=http://127.0.0.1:9247 探活 HTTP 200→cancel 回执→终态 cancelled→同端点 ConnectionRefused(10061)→describe 复验，3s）。 |
| F831-09-03 A 面（jet/ark 必需环境面） | P1 | **CLOSED via r3 合同修订**（lang 2d47e236e + Plan r3）：用户 2026-10-24 于三选项（提供环境补实测/正式修订合同/保持开放）中裁定"**正式修订验收合同**"——jet/ark 真实构建验证面自 AC-03/V-03 移出，转后续环境门控渠道（§4 修订记录；KNOWN-DEBT 承接项在案）。不缩小其余合同：生成面全 profile 真实生成（已绿）、vue/rust/vscode 真实构建+vm 真实运行仍为必需面（R2/r3 终源 39/39 全绿）；V06c 仍五 profile 全尝试逐项记录（jet/ark 缺环境记【环境门控·转后续】，工具链在场失败仍判红）。 |

### R1 轮（历史，全部维持 CLOSED）

| ID | 严重度 | 闭合证据（R1 修复轮） |
|---|---|---|
| F831-09-03 REOPEN | P1 | **CLOSED**：V06d/V06e 验证器重写（lang 09307c05e+fe8f3090a）——mid_alive 不再是证据；安装活动证据+保持性+重试真复绿；协议事件+协议 cancel+owned 回收终态+宿主复验；review-r1 replay-oracles vm-idle/install-idle 反例确定性红（false_positive=false）；真实链终源重跑 PASS（V06d 14s/V06e 1s）。**修复过程本身暴露一处真缺陷**：首轮修复的 describe 复验对同进程双读者阻塞挂死（假载体反例走不到该步、重放不可见）——真实链整跑暴露后根修（fe8f3090a 单读者通道），反向印证 review"真实链证据不可替代"的主张。jet/ark 环境红按原口径维持具名 partial。 |
| F831-09-05 REOPEN | P2 | **CLOSED**：admit_receipt 增 tracked 脏树拒绝（同 HEAD 源码改动→旧收据拒，untracked 可再生产物不判脏）；S17 增脏树负控+untracked 正控（五臂全绿）；replay-oracles admission-dirty 反例确定性红；四收据 R1 终源重生成后 admitted（RCPT-ID/RCPT-ID-F "tracked 树干净"）。 |
| F831-09-01 PARTIAL | P2 | **CLOSED**：UI 4250885 恢复独立 oracle——独立构造期望（`CARGO_MANIFEST_DIR/../../../auto-lang/packages/blueprints`，不经被测函数回读）+精确比较（死兜底/错 owner 不可过）+真实包枚举（`<kind>/<name>/spec.md` 实际 fixture 读取，首轮即抓出深度盲区后修正）+错 owner 负控；bp registry 10/10 绿。 |
| F831-R1-01 | P2 | **CLOSED**（由本刷新闭合）：source-audit 终态重生成（fe8f3090a/4250885，净迁移 769/72/0/1+逐提交归因，撤回旧交叉引用）；四收据 hash 重绑（R1 终源 5/5、38/39、19/19、46/0/1/1）；UI 完整 SHA 全表；SD 草案刷新（spec-delta.md：finding 注记更新至终态+840 并集注记）；tier-parity 865/867 对账（867 撤回）；canonical spec_inputs 重采样 @master 53298cd27（三文件 840 漂移显式记录，并集非覆盖）。 |

附带记录：PATH 解析命中主检出陈旧 auto.exe/auto-ui-tool.exe 的既知事实不变（实机入口钉工作树二进制，ci-entrypoints 注记）。

## 6. SD-01..07 与交接条件

逐条草案见 [spec-delta.md](spec-delta.md)（R2/r3 刷新：头部绑定终 SHA+840 并集注记+SD-02/05 r3 合同注记；仍为 proposal-for-review，T-10 对账后 T-12 沉淀，不覆盖 canonical）。

下一责任角色：**T-10 独立终审**（独立上下文，绑定本快照三仓完整 SHA 审核 AC/SD/反例/收据身份/净迁移/r3 修订依据）。pass 后语言裸 cargo t 及实际触面档、UI 适用门由 T-11 在冻结输入串行执行（本轮未运行任何整档 t/tv/tt/tb/tu/taa——合同后置，显式 pending；tu 面仅 --list 钉数 863，非执行档）；tf 仅主检出批量到期档。T-11 本地有序落地/ui-release.md/来源读回、T-12 Specs/ledger/归档/guard 清理全部 pending；U831 未成立，832 不得开工。
