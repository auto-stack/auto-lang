# PLAN-831 T-09 最终来源核查与候选快照（R1 修复轮刷新版）

日期：2026-10-24（T-10 R1 needs_fix 后同组修复轮闭合）；Plan revision：2；scope：work/T-09，非 T-10 独立终审。

本快照四份材料的内容哈希及三仓输入清单见 [freeze.json](freeze.json)。首轮快照结论 needs_fix（review-r1：F03/F05 REOPEN、F01 PARTIAL、F831-R1-01 OPEN）；本轮在同组原树完成四项修复、三反例重放确定性红、真实链终源重跑后重冻结。**候选结论：T-09 work 面闭合（四项 R1 finding 全部 CLOSED），可再送 T-10 独立终审**；jet/ark 构建面维持【环境阻塞·具名】红（AC-03 就其为 named-partial，不作 pass 主张、不缩减支持）；merge-owned 门显式 pending。Plan 状态 executing。

## 1. 唯一来源与可重建输入（R1 终态）

| 角色 | 权威树 / 分支 | 完整 SHA |
|---|---|---|
| 语言实现 | `D:/autostack/.wt/lang-831/auto-lang` / `codex/plan-831-dev` | `fe8f3090a35c5720bcac873edba38d9f588a5663` |
| UI 实现 | `D:/autostack/.wt/lang-831/auto-ui` / `codex/plan-831-dev` | `425088520e402b1efae9479b9d0b5b3d44a56df4` |
| down 只读依赖 | `D:/autostack/.wt/lang-831/auto-down` / detached | `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` |
| 搬迁比较基线 | 语言仓 T-04 source | `e199caf8a6cb369b320c02ca144c1e88b7bcef8f` |

两实现树 tracked `git status --porcelain` 干净（lang 唯一 untracked=compile-probe 本地 Cargo.lock，可再生不入库；F05 admission 已明确 untracked 可再生产物不判脏）。默认分支未推进；未修改 canonical Specs 或 ledger；本轮修复全部在 plan 分支提交（**R1 新增**：lang 09307c05e（F03/F05 验证器根修）+fe8f3090a（V06e 单读者死锁根修）；UI 4250885（F01 oracle 恢复+EOF 空行清理）。T-09 首轮 7+4 提交见 git log）。

本期渠道仍是本地 Git commit/源码包。重新消费本候选必须 checkout 上表 SHA 并核对各锁与源码包哈希；收据身份经 [admission](#5-本轮发现及闭合条件) 核对——R1 起 admission 含 tracked 脏树拒绝（同 HEAD 源码改动后旧收据不可采信），四收据均于上表身份 admitted（RCPT-ID/RCPT-ID-F 双 pass，"tracked 树干净"入收据明细）。尚未生成正式 `ui-release.md`，不得称已发布，未推远端。

## 2. 净迁移对账（R1 重算，撤回旧交叉引用）

**基线（e199caf8a ls-tree blob）vs 终 UI HEAD（4250885）逐文件比较：841 基线文件、769 同 blob、72 变更、0 丢失、1 新增**（明细与逐提交归因见 [source-audit.json](source-audit.json) migration/migration_attribution——72 变更=60 文件 F831-09-01 单源布线（15e3b14）+1 档位语义（fb5a2f4）+2 license/重复注册（071396e）+1 vscode PM（464569d）+2 R1 oracle/清理（4250885）+9 T-04 窗口机械改写（manifest/aura），跨轮重叠按首桶折叠）。

历史事实分层：T-04 导入窗口的 **841 文件逐字节恒等**（见 [import-ledger.md](import-ledger.md)）不受影响——上述 72 变更全部发生在导入窗口之后的 T-09/R1 修复轮，各有具名提交。首轮快照"2ea9b5a 830 同 blob"与 source-audit（当时绑定 461c609）的交叉引用不一致已由 review-r1 指出，本版以终态重算为准，旧表述作废。零丢失（0 missing）保持；语言原件删除与留仓清单不变。

## 3. 全部 V 与 AC 核查（R1 终态）

| V / AC | 终态 | 闭合证据与残余边界 |
|---|---|---|
| V-01 / AC-01 | **pass** | 同首轮终态（777 strict 0v/5gf 零警告真实扫描 362 文件、编译探针类型身份、DST-4 隔离 SDK 反控 5/5 R1 终源复验）；R1 增：blueprints 兜底断言恢复独立 oracle（见 §5 F01）。 |
| V-02 / AC-02 | **pass** | 同首轮终态（16 路径三 feature 组合 compile-probe 24/24、C2 单源 D01..06）。 |
| V-03 / AC-03 | **pass（具名环境边界，非全绿主张）** | 792 收据 R1 终源 38/39：V01/V02 fresh/locked/重定位、V05 工具随包消费、V06a 真实窗口 run（句柄 40568130）、六 profile 真实生成、**vue/rust/vscode 真实构建链 completed**；**jet/ark=【环境阻塞·具名】红**（R1 环境实测复核：gradle/java/ANDROID_HOME 全缺席→9009；hvigorw/DEVECO_SDK_HOME 缺席→无构建执行——见 run-profile-evidence r1_env_probe）。该两面维持 partial/blocked：须工具链到场实测或用户正式修订验收合同后方可改判，本轮未获修订授权。 |
| V-04 / AC-04 | **pass** | 792 坏 pin/漏 patch/资源四态/旧字节/陌生目录逐 case 绿（R1 终源收据）；**V06d R1 重写为可区分 oracle**：安装活动证据（事件+node_modules 落盘，终态先至=假载体红）→中断→陌生数据/输入字节保持→重试 completed+dist 落盘→复绿后保持（真实链 PASS 14s）；drill 19 检 R1 终源重跑；admission 含 tracked 脏树拒绝（selftest S17 五臂）。 |
| V-05 / AC-05 | pass（历史+终态） | T-02 保全载荷不变；drill 终态 19/19（R1 重跑收据）；真实仓 prepared/land/switch 归 T-11 合同窗口。 |
| V-06 / AC-06 | **pass** | 正典 ID 对拍不变（t 面三共享包逐 ID 相等、tu 863=863 ID 级差集空——**R1 对账 865/867：865=基线行数（863 唯一+2 重复注册行），867 无列举来源属叙述误计已撤回，当前侧 @4250885 实测 863 行**，见 tier-parity r1_reconciliation）；ti 28/28、tn 门可列举；777 双根围栏；跨仓编排入口；V06e vm 解释运行改协议观测（事件+cancel+终态+宿主复验）。 |

fixture 全族 R1 终态：`--phase source-switched` = **46 pass/0 fail/1 recorded/1 pending**（收据 t09-r1-all.json；recorded=SRC-P4c 清洁面复核；pending=TYP-PROBE 零 cargo 缺省——专项 24/24 在案）；selftest 全过（含 S17 R1 新增脏树负控/untracked 正控）。

## 4. 真实支持面与发行边界（R1 终态）

| 面 | 终态证据 |
|---|---|
| UI 源码组 | R1 终源 5/5（fresh/locked/整组重定位 154s/隔离 SDK 88s）；组布局成立 |
| 外部 Rust bundle | V01/V02 终源绿 + V06a 真实窗口 run；api-gen 双根修复保持 |
| Vue | V05e 生成+资源字节=源；V06c vue completed（真实 pnpm 链）；**V06d 可区分取消恢复 oracle PASS** |
| VM/Iced | **V06e R1 重写：协议事件（resolve backend/vm interpreter+Launched(MCP)）证明 run 接受且实际运行→协议 cancel 回执→owned 树回收后终态 outcome=cancelled→cancel 后宿主 describe 复验存续**（真实链 PASS 1s）；三族寿命冒烟（窗口/终端/媒体）沿用 |
| jet / ark | 生成真实（24/25 文件）；构建【环境阻塞·具名】红（R1 环境实测复核在案）；不删除支持、不记 env skip 为 pass |
| tauri / vscode | tauri 生成真实；vscode 生成+构建 completed；run 面宿主域具名 |
| autodown / mpv | down 固定 c39ac7f 在场（四收据齐记）；mpv/native 扩展 feature 门未开（原门控不强制） |
| 许可 | UI 根 LICENSE + 五包 MIT + shadcn 上游随资产；warn-license-parity 零新增 |

## 5. 本轮发现及闭合条件（R1 四项全 CLOSED）

| ID | 严重度 | 闭合证据（R1 修复轮） |
|---|---|---|
| F831-09-03 REOPEN | P1 | **CLOSED**：V06d/V06e 验证器重写（lang 09307c05e+fe8f3090a）——mid_alive 不再是证据；安装活动证据+保持性+重试真复绿；协议事件+协议 cancel+owned 回收终态+宿主复验；review-r1 replay-oracles vm-idle/install-idle 反例确定性红（false_positive=false）；真实链终源重跑 PASS（V06d 14s/V06e 1s）。**修复过程本身暴露一处真缺陷**：首轮修复的 describe 复验对同进程双读者阻塞挂死（假载体反例走不到该步、重放不可见）——真实链整跑暴露后根修（fe8f3090a 单读者通道），反向印证 review"真实链证据不可替代"的主张。jet/ark 环境红按原口径维持具名 partial。 |
| F831-09-05 REOPEN | P2 | **CLOSED**：admit_receipt 增 tracked 脏树拒绝（同 HEAD 源码改动→旧收据拒，untracked 可再生产物不判脏）；S17 增脏树负控+untracked 正控（五臂全绿）；replay-oracles admission-dirty 反例确定性红；四收据 R1 终源重生成后 admitted（RCPT-ID/RCPT-ID-F "tracked 树干净"）。 |
| F831-09-01 PARTIAL | P2 | **CLOSED**：UI 4250885 恢复独立 oracle——独立构造期望（`CARGO_MANIFEST_DIR/../../../auto-lang/packages/blueprints`，不经被测函数回读）+精确比较（死兜底/错 owner 不可过）+真实包枚举（`<kind>/<name>/spec.md` 实际 fixture 读取，首轮即抓出深度盲区后修正）+错 owner 负控；bp registry 10/10 绿。 |
| F831-R1-01 | P2 | **CLOSED**（由本刷新闭合）：source-audit 终态重生成（fe8f3090a/4250885，净迁移 769/72/0/1+逐提交归因，撤回旧交叉引用）；四收据 hash 重绑（R1 终源 5/5、38/39、19/19、46/0/1/1）；UI 完整 SHA 全表；SD 草案刷新（spec-delta.md：finding 注记更新至终态+840 并集注记）；tier-parity 865/867 对账（867 撤回）；canonical spec_inputs 重采样 @master 53298cd27（三文件 840 漂移显式记录，并集非覆盖）。 |

附带记录：PATH 解析命中主检出陈旧 auto.exe/auto-ui-tool.exe 的既知事实不变（实机入口钉工作树二进制，ci-entrypoints 注记）。

## 6. SD-01..07 与交接条件

逐条草案见 [spec-delta.md](spec-delta.md)（R1 刷新：头部绑定终 SHA+840 并集注记，各 SD 来源行的 finding 状态更新至终态；仍为 proposal-for-review，T-10 对账后 T-12 沉淀，不覆盖 canonical）。

下一责任角色：**T-10 独立终审**（独立上下文，绑定本快照三仓完整 SHA 审核 AC/SD/反例/收据身份/净迁移；jet/ark 环境面请裁决：工具链到场实测，或正式修订验收合同）。pass 后语言裸 cargo t 及实际触面档、UI 适用门由 T-11 在冻结输入串行执行（本轮未运行任何整档 t/tv/tt/tb/tu/taa——合同后置，显式 pending；tu 面仅 --list 钉数 863，非执行档）；tf 仅主检出批量到期档。T-11 本地有序落地/ui-release.md/来源读回、T-12 Specs/ledger/归档/guard 清理全部 pending；U831 未成立，832 不得开工。
