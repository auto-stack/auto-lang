# ACE M1 净缺口审计账本（ace-m1-gap-audit）

> 状态：delivered（PLAN-903，2026-11-01 冻结；基线 lang master `b1a4871aa`，审计树
> `D:/autostack/.wt/lang-903/auto-lang` @ `plan-903-dev`）。
> 性质：静态对账审计——对 M1 登记、已批准适用规则、已交付能力与正式消费者逐格归账。
> 本审计**不修改**生产、判定器、binding/expected/golden/证书、批准规则/分母；
> **审计 PASS ≠ M1 通过**；M1 正式签章另项（§8 顺序④）。
> 分母登记：`crates/auto-lang/test/cases/ace-m1/binding.tsv`
> （SHA256 `d9f05da057dfd658df09a7725aa391bc4c9e14993e13e9846724160915ff5d7a`）；
> 逐格 raw token 以 binding.tsv 为准，本文件按**族**归账并保 raw token 映射（§2/§5）。

## 1. 冻结基面（T-01 / V-01、V-06）

| 轴 | 冻结事实 |
|---|---|
| 基线 | lang master `b1a4871aa`（2026-11-01）；903 审计树基于该 commit（组 `D:/autostack/.wt/lang-903/`，含 auto-down detached `7c4d5cb` + auto-web detached `ec9c4ba` 兄弟） |
| 分母 | binding.tsv 131 数据行 × 12 列 = **91 corpus + 40 scenario**；四路径（ac/av/avm/a2r）共 **524 登记格**；hash 复核与计划 §4.2 冻结值逐位一致 |
| 来源分解 | **原 103 行**（63 corpus：58 源例 + 5 typed 入口 W01–W05；40 scenario）+ **U01–U10**（PLAN-862 入口/Unit）+ **H01–H03**（PLAN-866 常驻）+ **O01–O15**（PLAN-863 console，ns=corpus）；103+10+3+15=131，脚本守恒验证通过 |
| 正式 suite（固定组） | `case-ace-m1-{base, tr-native, collections-864, closures-865-source, entry-862, method-881, resident-866, native-collections-889, typed-native-891, console-863, text-source-894}` 共 11 组，全部注册于 `scripts/test-groups.toml`；另有 `case-a2r-hir-record-text-902`（新 A2R 并列证据，PLAN-902） |
| 新 A2R 并列线 | `a2r-hir-892`（B01/B04/B13/B14/NAMED-01）与 `a2r-hir-record-text-902`（R07/R08/T03/T05）套件在 `crates/auto-lang/test/cases/`；**892 无固定组注册**（902 有）→ F-INFRA-5 |
| 工具/脚本 | `scripts/verify-ace-m1-852.ps1`（六+一 scope；现 infra 红，见 F-INFRA-1）、`scripts/verify-av-862.ps1` / `verify-av-863.ps1`（各自 -Scope 门）；日常绿证据入口=固定 `cargo t case-ace-m1-*` + 常驻 cargo 测试 `ace_m1_tests.rs`（负例 oracle/金样漂移必红） |
| 批准规则 | 2026-10-28 用户裁定（`docs/plans/reports/ace-m1-scope-decision-2026-10-28.md`）：五态词表、`na:m1(rule)` 反滥用、NA-1..NA-4、R-A..R-F、a2r 两轴分离；裁定只写口径，binding.tsv 未迁移（本审计 §3） |
| 金样/证书 | `oracle/m1_*.rs` 24 金样；`oracle/certs/*.cert.json` 10 证书（认证集=标量值例 10 例，schema `ace.m1.cert/1`）；`src/*.at` 78 源 |
| 关闭 oracle 来源 | scenario 族关闭 oracle 冻结于 `docs/plans/attachments/852/{SCENARIOS,ORACLES}.md`——**该目录 2026-10-31 起整目录不入库**，fresh worktree 无此文件，主检出磁盘仍存 → F-INFRA-2 |
| 在途/后继基面 | 893（慢测归因）drafting r3 0/12；904（HIR→Rust collections/variants）drafting r1 0/14 已立项；899 已归档（工具/profile 变化使受影响证据 stale 需按 899 收据核对）；902 已归档（record/text 新 producer，delivery `4303c4457`） |

基面指纹清单（239 文件 sha256）存 `D:/autostack/plan-artifacts/plan-903/baseline-freeze.txt`；
本文件只记计数与 hash 锚，merge 时摘要折叠进计划文件。

## 2. 守恒普查（V-01，实测）

普查脚本 `census_binding.py`（scope 内）解析 binding.tsv，**负控三项全部正确拒绝**：
漏 1 行 scenario（I06）→ `REJECT: rows 130 != 131`（exit 1）；把 scenario:R03 混入
corpus → `REJECT: DUPLICATE ('corpus','R03') + scenario 39 != 40`（exit 1）；截断行至
8 列 → `REJECT: SCHEMA row has 8 columns`（exit 1）。正常模式全守恒通过（exit 0）。

每路径 raw token 普查（131 格/路径；守恒和=131 已脚本验证）：

| token 族 | ac | av | avm | a2r |
|---|---:|---:|---:|---:|
| `ok*`（含 `ok:P889-source`/`ok:P881`/`ok:P891-typed`/`ok:av-*`/`ok-emit` 等） | 65 | 113 | 35 | 24 |
| `emit-ok-run-unverified` / `snapshot-only-cert-gap` / `cert-diff` | – | – | – | 18/16/4 |
| `diff*`（含 accepted/pending/legacy-*） | 3 | – | 31 | 4 |
| `na:m1(NA-4)` | 10 | – | 10 | 10 |
| `n/a`（裸 n/a，无规则 ID） | 7 | 7 | 24 | 16 |
| `gap:scenario-tooling` | 3 | 7 | 2 | 2 |
| `gap:ac-composite-native` | 18 | – | – | – |
| `gap:ac-console-profile` / `gap:console-profile-parity` | 15 | – | 15 | 15 |
| `gap:mn-scalar-only`（K01–06 + H01–03） | 9 | – | – | – |
| `gap:avm/a2r-no-typed-entry`（W01–05，NA-1 候选） | – | – | 5 | 5 |
| `gap:avm/a2r-no-dyn-api`（D01–09，NA-2 候选） | – | – | 9 | 9 |
| `gap:a2r-golden`（B17–B19/T06/R09/R10） | – | – | – | 6 |
| `gap:a2r-untyped-emit`（R07） | – | – | – | 1 |
| `gap:av-alloc-fault` / `gap:av-821-gen` / `gap:R5-O` / `gap:av-822` | – | 1/1/1/1 | – | – |
| `gap:ac-native-loader`（I08） | 1 | – | – | – |
| `source-bound`（T08 a2r 非标状态 token） | – | – | – | 1 |

corpus:R03（命名type构造映射）与 scenario:R03（活借用/旧buffer）为两条独立义务，
双行并存（namespace 混淆负控已拒）；scenario 行（E01 等）"no source program" 的
观察通道未交付不冒源码义务（§5 F-SCN-1）。

## 3. 批准适用与 a2r 双轴（T-02 / V-02）

**已批准规则 → 现格位**（规则文本不改；binding raw token 未迁移属 F-REG-1）：

| 规则 | 适用格（现 raw token） | 判定 |
|---|---|---|
| NA-1（legacy/a2r 不判 typed fixture 入口） | W01–W05 avm×5 + a2r×5（`gap:*-no-typed-entry`） | M1 不要求；重新进入=有计划冻结 legacy/a2r typed 入口 |
| NA-2（legacy/a2r 无 Dyn API） | D01–D09 avm×9 + a2r×9（`gap:*-no-dyn-api`） | M1 不要求；重新进入=SEM-07 或 legacy Dyn 能力计划 |
| NA-3（native 不判闭包/Dyn/在线热重载/暂停恢复） | K01–06 ac×6 + D01–09 ac×9（`gap:mn-scalar-only`/`gap:ac-composite-native`）；I07/I08 ac 两格**待确认扩展**（裁定附录 A 已按 NA-3 投影，H01–03 ac 未在裁定内）→ G-DEC-4 | M1 不要求；native 闭包/Dyn ABI、loader 仍为具名能力线（§5 F-NAT-*） |
| NA-4（入口/Unit AV-only，2026-10-28 补裁） | U01–U10 ac/avm/a2r×30（已是 `na:m1(NA-4)`） | 唯一已迁移入表的规则；重新进入=native/legacy 冻结该 profile |
| R-A/R-B/R-D（旧引擎值/语义/运行时差异） | avm `diff-legacy-*` 30 格 | `diff:accepted`，不等于 ok，证据保留 |
| R-C（旧引擎接受非法程序） | B17/B19 avm（已 `diff:accepted(R-C)`）等 | 同上；**B18 avm 现为 `diff:pending`**——862 T-06 实测观察为 infra error，非干净 accept → F-INFRA-3 |
| R-E（a2r 名解析，**不接受**） | B16/E03 a2r（`diff-no-name-check`）×2 | 真缺口 `gap:a2r-resolve`（F-A2R-1），不得转 accepted |
| R-F（a2r 认证轴差异） | B13/B14/B15/B20 `cert-diff` ×4 | `ok+cert-gap` 等价：M1 a2r 判据=金样轴，认证缺口独立账 |

**a2r 双轴**（裁定决策四）：M1 的 a2r 判据=**金样/emit 逐字节轴**；可编译认证是独立
认证线。38 格 `emit-ok-run-unverified`/`snapshot-only-cert-gap` 为 **ok+cert-gap 等价**
（emit 轴绿、认证轴缺口另账，owner=a2r 复合 lowering，F-A2R-3）。控制实录：
把认证缺冒 emit 失败或认证 PASS 都不改变 emit 轴判据——本审计不隐式 certify、不写
证书（oracle/certs/ 前后 hash 不变，§9）。
**PLAN-992 更新（2026-11-08）**：该 38 格中的 **17 格已经原域显式认证关闭**（H-01
12 例既有复合金样 G04 重审 + H-02 5 例新冻结 B08/B11/R01/R03/R04；证书 10→27、
cert-gap **38→21**，详见 a2r-golden-certification §PLAN-992）；"伪 Rust 不可编译"
的登记注释对该 17 例不成立（E-STALE 勘误）。余 21 格 = H-03 文本 4（T01–T04）+
H-04 集合 6（L01–L05/L07）+ H-05 `move move` 1（K06）+ H-06 trap/宽度裁定 4
（B13/B14/B15/B20）+ H-07 场景 6（scn R01–R06），仍为 ok+cert-gap 等价账、owner
不变、`can_claim_m1=false` 不变。
**PLAN-993 更新（2026-11-08）**：余 21 中的 **H-05 K06 已关闭**——写捕获闭包×
显式 move 的双 `move` 发射缺陷修复（恰一次 move + 自引用 cell 赋值 `__a2r_wc`
提升消 BorrowMutError + 捕获克隆字典序），金样重冻结 G04 重审、原域显式认证
（证书 27→28、cert-gap **21→20**，详见 a2r-golden-certification §PLAN-993）。
余 20 格 = H-03 文本 4 + H-04 集合 6 + H-06 trap/宽度裁定 4 + H-07 场景 6，
等价账/owner/`can_claim_m1=false` 不变。

**bare `n/a` 54 格无规则 ID**（ac7/av7/avm24/a2r16：G01–G08 部分、C01/C02、I01–I11
部分、H01–03 avm/a2r）：按裁定 §5 反滥用规则 1 视为待裁定，**不自行纳为 na:m1**
→ G-DEC-1（候选 NA-5/NA-6，KNOWN-DEBT 已登记待决策）。

## 4. 净缺口分类总账（T-03 / V-03、V-04）

七类逐格归账（一格可多问题，主类唯一、次类在族表注记；总和=524，脚本守恒对账）：

| 类别（计划 §5.2） | 格数 | 构成 |
|---|---:|---|
| ① 已有且正式接线 | 237 | ac ok 65（B 标量 17、T01–08、R01–10、L/V 14 `ok:P889-source`、W 5 `ok:P891-typed`、E03/C01/C02/I01–03/I05/I06/I10–12）；av ok 113（corpus 90 + scenario 23：D01–09、R05、I05–I08、I10–I12 等）；avm ok 35；a2r ok 24（`ok-emit` 金样轴 + G02–G08 认证系统行为） |
| ①+双轴（ok+cert-gap 等价） | 38 | a2r emit 18 + snapshot 16 + cert-diff 4（R-F） |
| ⑥ 批准 NA（token 未迁移） | 73 | NA-4 已入表 30 + NA-1 候选 10 + NA-2 候选 18 + NA-3 候选 15（K ac6 + D ac9） |
| ⑥ 批准 accepted-diff | 32 | avm 30（R-A/B/C/D）+ a2r 2（K02/K05 legacy-grammar） |
| ④ 登记/裁判落后 | 11 | B17–19 ac `diff:pending`×3（mn 族面 vs 冻结诊断码，见 F-REG-2）；`gap:a2r-golden`×6（诊断行无金样语义，面=前端拒绝，F-REG-3）；T08 a2r `source-bound`×1（非标 token）；G01 av `gap:av-822`×1（822 从未立项的陈旧 owner，F-REG-4） |
| ⑤ infra/失效证据 | 1 | B18 avm（观察=infra error，非干净 accept/reject；重验未实证 → §10） |
| ②/③ 能力/证据缺口 | 73 | scenario 观察通道 14（F-SCN-1）；console 45（ac 15 + avm/a2r parity 30，F-CON-1/2）；AV 故障注入/代际/共享可变 3（F-AV-1/2/3）；a2r 静态检查/未定型发射 3（F-A2R-1/2）；mn composite 场景面 8（E02、R01–R05、I04、G01，F-NAT-4） |
| ⑦ 未定位/待决定 | 59 | bare `n/a` 54（G-DEC-1）+ H01–03 ac 3 + I07/I08 ac 2（NA-3 扩展确认，G-DEC-4） |

关键"不冒"控制（V-03/04 实录）：typed 行（`ok:P891-typed`/`ok:av-data-typed`/
`ok:av-closures-823`）不冒领源码行能力；892/902 新 A2R 行并列不替换旧 a2r 身份；
avm `diff-legacy-*` 不翻绿；任意非零 exit 不当负例成功（L06 必须 trap 原因
`list-index`+exit70）；R03(scenario) 的 821 gen 字段存在≠运行期校验；B18 avm infra
不判语言行为；E/I 场景行缺源程序不冒源码义务。

## 5. 逐族净缺口账（T-05 / V-05）

每族：格位 | 类别 | 能力 owner | 证据/判定 owner | 已立项 | 前置/写面 | 独立关闭 oracle | 下一动作。
**未立项项的规划责任=ACE 路线/具名能力包，不虚构他方会话已接任务。**

| 族 | 格位 | 类 | 能力 owner | 判定/证据 owner | 已立项 | 前置/写面 | 独立关闭 oracle | 下一动作 |
|---|---|---|---|---|---|---|---|---|
| F-OK-1 底座标量/文本/记录源码四路径 | B/T/R 大部、E03、C01/C02、I01–03/05/06/10–12（237+38 格） | ① | 已交付（852/870/881/889/891/894/862/863/866/823/824/865） | 固定组 case-ace-m1-* + ace_m1_tests.rs 常驻负控 | 是（均已归档） | 无 | 同源冻结 oracle + 父侧判定 + 金样/证书 hash | 复用冻结收据；本审计窄重验 base 组（§10） |
| F-A2R-1 a2r 静态检查（名解析） | B16/E03 a2r | ③ | a2r 线（[242 tracker](../../plans/242-a2r-feature-gap-tracker.md) active） | 同左 | 242 在册（未专列施工计划） | a2r 前端名解析 | R-E 已裁真缺口：未执行分支未定义名必须被拒（SCENARIOS E03/B16） | 242 线立项施工；不转 accepted |
| F-A2R-2 a2r 未定型发射 | R07 a2r | ③ | a2r 线（242） | 同左 | 242 在册 | mut-record emit 定型 | R07 emit→可编译金样（旧 a2r 身份） | 242 线；892/902/904 新线不自动替换 |
| F-A2R-3 a2r 复合认证 | ~~38 格 cert 轴~~→**余 20 格**（PLAN-992 关闭 17：H-01 12+H-02 5；PLAN-993 关闭 1：H-05 K06）+ `oracle/certs` 10→**28** | ③（独立账，不阻 M1 emit 轴） | a2r 复合 lowering（242 跟踪）；余 20 分解 H-03 文本4/H-04 集合6/H-06 trap4/H-07 场景6（991 §5.4.1 合同，994 承接 H-03 后继；H-05 已闭） | 同左 | 242 在册；904（新线 collections/variants）drafting r1 已立项；992 已闭 17 格、993 已闭 K06（2026-11-08） | 伪 Rust（`List<int>`/`int`）可编译化仅指余 20 格（18 格勘误为可编译，E-STALE） | 认证 spec：actual==golden 且 rustc 编译运行==行为 oracle（G01/G04–G08 语义） | 旧线=242；新线=904 后继；994（文本）按 D992 交付；**M1 消费者是否迁移=专门审计另立项** |
| F-SCN-1 scenario 观察通道（trace/events/harness） | E01/E02×4、R01/R04/R06 scn、I04/I09（14 格 `gap:scenario-tooling`） | ②（引擎能力多已有，观测通道缺） | scenario 工具包（未立项；E01 av 有 `av.core.trace-order` 既有 recipe 证据） | 同左 | **否** | 新 scenario harness 写面（不动生产） | SCENARIOS.md 逐行 oracle：trace 字节恰 ba（E01）、逐 owner LIFO 事件（R01）、构造/clone 失败注入回收（R02）、budget/cancel 退出面（R06）、layout 漂移拒绝（I04/I09） | 立项 scenario 工具包（规划责任=ACE 路线） |
| F-AV-1 AV 分配失败注入 | R02 scn av | ③ | AV 资源故障代际包（未立项；852 T-05 调查 owner） | 同左 | 否 | 注入通道 API | SCENARIOS R02：逐分配点注入+合法正控+部分回收，错误分类不伪装语言错误 | 与 F-SCN-1 合并候选 |
| F-AV-2 AV 句柄代际运行期校验 | R03 scn av | ③ | AV 句柄/generation 包（未立项；821 gen 字段≠运行期校验） | 同左 | 否 | gen 字段→运行期校验 | SCENARIOS R03：扩容后旧 view 访问受控拒绝、危险访问=0 | 同上候选 |
| F-AV-3 C-19 共享可变捕获 | K06 av | ③ | **标量切片已交付（PLAN-909 2026-11-02：K06 av=ok:av-closures-source-865，原源 42）**；共享 owned/循环/跨挂起仍属后继 gap | 同左 | 否 | 共享可变 env 捕获语义 | K06 源码正例（PLAN-909 已实证 42）+shared-mutable-909 独立性 oracle（S04 clone 44） | 标量切片已闭；残余另立项 |
| F-CON-1 native console profile | O01–15 ac | ③ | native console 扩面包（未立项；现显式拒 profile 8） | 同左 | 否 | mn console 前端/运行时 | O 行字节级 stdout/stderr + 独立 exit/Unit/trap oracle（863 手算） | 立项；D-863-NB-01 边界保持 |
| F-CON-2 legacy/a2r console parity | O01–15 avm+a2r（30 格） | ③（未承诺交付） | 各 backend 扩面/验证立项（D-863-NB-01：尚无交付承诺） | 同左 | 否 | profile 8 parity 验证 | 同 863 字节 oracle 于 legacy/a2r 通道 | 用户决定是否立项；不因 AV 行宣称 whole-M1 |
| F-NAT-1 native 在线 loader | I08 ac（+H01–03 ac 同族） | ③（M1 适用性待 NA-3 扩展确认） | native loader 能力包（未立项；**重启 exe 不满足**） | 同左 | 否 | 常驻进程内安装新机器码 | SCENARIOS I08：同一宿主进程真实调用新码、旧帧旧版完成 | 立项 loader 包；893 慢测不覆盖 |
| F-NAT-2 native 闭包/Dyn ABI | K01–06/D01–09 ac（NA-3 候选 15 格） | ⑥+③（M1 na；能力线未立项） | native 闭包/Dyn ABI 包（823/824 显式排除；Design49 §7） | 同左 | 否 | ABI 冻结 | 裁定重进入条件：ABI 冻结后逐行改回 `gap:<plan>` | 暂无；NA-3 屏蔽下不阻 M1 |
| F-NAT-3 native 常驻会话 | H01–03 ac（3 格）+avm/a2r bare n/a（6 格） | ⑦（裁定后置 H 行未覆盖） | 同 F-NAT-1 loader 线 | 同左 | 否 | — | H01–H03 AV 侧已关（866 真实 worker 父侧判定）；native 侧同 I08 oracle | G-DEC-4 裁定适用性 |
| F-NAT-4 mn composite 场景面 | E02、R01–R05 scn、I04、G01 ac（8 格） | ②/③（复合执行已交付 870/889，场景观测面缺） | scenario 工具包 + mn 复合 | 同左 | 否 | E/R/I 场景 harness 接 mn 复合 recipe | 逐行 SCENARIOS oracle 于 mn 通道 | 随 F-SCN-1/F-AV-1/2 立项 |
| F-NAT-4a R03 ac 已由 923 关闭 | scenario R03 ac（F-NAT-4 内一格） | ①（PLAN-923 交付） | 923 已交付 | 同左 | 是 | `ac/mir-native-resource-guards-v1` 父侧冻结表（case-ace-m1-native-resource-923 + collections_place_tier）；R02/R05 完整闭包仍属 924 | 同左（R02/R05 欠项 924） | 924 唯一回填 R02/R05；本行不改其他格 |
| F-REG-1 五态词表回填 | ~136 格 raw token≠裁定五态 | ④ | 重绑定计划（未立项） | 同左 | 否 | **前置=G-DEC-1/2/3 用户裁定** | binding.tsv 迁移后 acceptance 分轴计数可复算（862 实测口径 20/73 只在 corpus，不冒全表） | 用户裁定→立项重绑定 |
| F-REG-2 B16 vs B17–19 ac 面不一致 | B17–19 ac | ④（+⑦ 裁定） | mn 诊断面（870 已交付族面+crate 内强 oracle） | 语言语义 owner | 否（870 已交付面，裁定未跟） | 裁定：族面（parse+`mn.unsupported.frontend`）是否满足冻结诊断行（B16 已 ok 的先例） | B16/B17–19 同一映射口径复判 | G-DEC-2 裁定后随 F-REG-1 回填 |
| F-REG-3 诊断行 a2r face 语义 | B17–19/T06/R09/R10 a2r（`gap:a2r-golden`） | ④（+③ 候选） | a2r 线（242） | 同左 | 242 在册 | 诊断负例判面=前端拒绝，**非金样**（认证 spec：负例不生成可执行金样） | a2r 前端拒绝通道给出各冻结诊断 | 随 F-A2R-1 扩展或重绑定时改 owner token |
| F-REG-4 G01 av 陈旧 owner | G01 av（`gap:av-822`） | ④ | —（822 从未立项；wait:D82x 已由 870/894/823/824 分别落地） | 重绑定 | 否 | — | G01 判定面=a2r 认证轴（a2r 列 ok 已闭）；av 面适用性随 G-DEC-1 | 重绑定时改判/改 na |
| F-INFRA-1 旧 852 脚本 | verify-ace-m1-852.ps1（非格位） | ⑤ | L0 修复（未立项；852 已归档不可回开） | 同左 | 否 | 三重红：pwsh split bug（D-862-R1-01，contract/acceptance 直接红）；contract 读 `attachments/852/cases.json`（主检出磁盘已无此文件）；attachments 不入库（fresh worktree 无） | 修好前**不得把 852 scope 当可运行 M1 完整性门**（KNOWN-DEBT D-862-R1-01 既有裁定） | L0 worktree 修脚本或由重绑定计划替代其完整性面（verify-av-862 `.Split([char]9)` 已覆盖 12 列/唯一 id/na 规则面） |
| F-INFRA-2 关闭 oracle 不入库 | attachments/852/{SCENARIOS,ORACLES}.md | ⑤ | 治理（材料去向裁定已定：attachments 不入库） | 同左 | 否 | fresh worktree/新会话无法读到 M1 scenario 关闭 oracle | 把 SCENARIOS/ORACLES 必要内容沉淀为 tracked 规范（本审计 §2/§5 已摘录关键 oracle 行） | 随重绑定/沉淀计划把 oracle 正文入 specs；不重建 attachments |
| F-INFRA-3 B18 avm 失效观察 | B18 avm | ⑤ | 重绑定计划（重探针） | 同左 | 否 | avm/run 对 src/B18.at 干净观察 | 干净 accept→R-C `diff:accepted`；干净拒绝带辨识→另记；仍 infra→保 pending | 下一探针已具名（§10）；本审计未实证不冒 |
| F-INFRA-4 899 工具/profile 演进 | 受影响冻结证据（非格位） | ⑤（条件性） | 各证据 owner（899 已归档，收据在档） | 同左 | 已归档 | 899 使 CTE/AV 工具职责命名/成员行号 profile 变化 | 受影响收据按 899 §收据逐身份核对后方复用 | 复用前核对；不整批作废 |
| F-INFRA-5 892 套件无固定组 | a2r-hir-892（非 M1 格位） | ④/⑤（接线缺口） | 892 后继/904 | 固定组注册 | 904 已立项（新线） | 892 证据日常入口=手工 case 运行 | 注册固定组或由 904 收口时统一 | 904 或 L0 补注册 |

## 6. 未定位/待决定（⑦，全部交用户或具名裁定，审计不自行决定）

| D | 内容 | 影响格 | 下一最小动作 |
|---|---|---|---|
| G-DEC-1 | bare `n/a` 54 格规则归属（候选 NA-5/NA-6；覆盖 G01–G08 部分、C01/C02、I01–I11 部分） | 54 | 用户裁定规则集（KNOWN-DEBT 既有待决策项）；裁定前 M1 clean 只能以实测口径记 |
| G-DEC-2 | B17–19 ac 族面口径（B16 ok 先例 vs 冻结诊断码） | 3 | 裁定 mn 族面是否满足；或立 mn 精确诊断码包 |
| G-DEC-3 | 诊断行 a2r 义务面（前端拒绝 vs 金样）与 a2r-resolve 范围扩展 | 6 | 随 242 线/重绑定裁定 owner token |
| G-DEC-4 | NA-3 对 H01–03/I07/I08 ac 的扩展确认 + H 行 avm/a2r 适用性 | 11 | 用户确认；H 行 native 能力线随 F-NAT-1 |
| G-DEC-5 | M1 适用范围总账（原 103 与扩展 28 行分别的 M1-required 集合） | 全表 | 重绑定计划输出"适用范围账"（本审计 §3/§4 已给候选映射） |

## 7. 与在途线的分工（计划 §5.5 边界，本审计不消费 WIP）

- **893**（drafting r3）：13 项慢测归因/未来接入归 893；M1 逐格审计归本审计。互不等待、不重复归因。
- **899**（已归档）：工具/profile 变化使受影响证据 stale——复用冻结收据前按 899 收据核身份（F-INFRA-4）。
- **902**（已归档）/ **904**（drafting r1）：新 HIR A2R 并列线。902 已交付 record/text emit；904 立项 collections/variants。**不自动替换旧 a2r 身份/不关旧 M1 分母**；迁移=专门审计另立项。
- **242**（active tracker）：旧 a2r 能力缺口（resolve/untyped-emit/composite）在册。

## 8. 后继顺序（建议默认；不制造全局语义串行）

1. **登记/证据归一**：G-DEC-1..4 用户裁定 → 重绑定计划（五态回填 + B17–19 面口径 + oracle 正文入库 + 852 脚本修复/替代 + H/O 行适用账）。前置=裁定；写面=binding.tsv/规范/脚本（不动生产）。
2. **实证能力缺口**：scenario 工具包（F-SCN-1/NAT-4）+ AV 故障/代际（F-AV-1/2）+ native console（F-CON-1）；各带独立关闭 oracle。
3. **旧 a2r 收口**：242 线（resolve/untyped/composite）+ 904 新线推进；迁移审计另立项。
4. **M1 正式验收**：全部适用义务（含裁定后分母）齐后独立验收——**本审计完成不构成 M1 绿**。

## 9. 保护与声明（V-06）

- 审计树 diff 仅 `docs/specs/auto-test-case/**`（本文件 + conformance/project 两处修订）；
  生产/源/oracle/expected/binding/cert/固定组零改动，binding.tsv hash 前后一致（§1）。
- `oracle/certs/` 未写：本审计不隐式 certify（§3 控制实录）。
- 来源漂移处理：若合并窗口内 binding.tsv 或 suite 变更，仅重核受影响族的结论，不整包重跑。
- 声明边界：903 审计 PASS 只证明账目完整/诚实/责任到人；M1 尚红格（②③⑤⑦ 类）不因此转绿。

## 10. 窄重验记录（T-04）

| 项 | 入口/命令 | 结果 | 归因/去向 |
|---|---|---|---|
| binding 守恒普查 + 3 负控 | `python -I census_binding.py binding.tsv [ctrl]` | 正常 exit0 全守恒；drop/mix/short 三控全 REJECT exit1 | V-01 证据；脚本/输出存 scope |
| base 组重验（当前 HEAD 接线） | `cargo t case-ace-m1-base`（专属树） | 见计划 §7 收据（首红-auto-web 缺失→补 detached 兄弟后重跑，首红保留） | F-OK-1 同身份新证据；组脚本未建 auto-web 兄弟实录入 F-INFRA-1 注记 |
| B18 avm 重探针 | avm/run × `src/B18.at` | **未执行**（avm 腿未接线任何现行 suite；构造新 case 超静态审计授权） | 保未实证 + 下一探针具名（F-INFRA-3）；重绑定计划执行 |
| 其余各族 | 冻结收据按身份复用（各交付计划 §9 + binding 注记 + suite/固定组注册静态核） | 未重跑 | 计划 §5.3"先冻结证据"；无界全套/13 慢测非本审计门（893 边界） |

## 11. PLAN-908 后继状态（2026-11-01；§1–§10 为 903 冻结历史，不重写）

G-DEC-1..5 已由用户按推荐全部裁定（决策件 §13，2026-11-01），PLAN-908 实施规则/登记/
来源/入口归一。本节只记各族**当前**状态与 owner；逐格 token 以 binding.tsv 为准，
适用分层以 [ace-m1-applicability.md](ace-m1-applicability.md) 为准。

| 族/事项 | 908 后状态 | 证据/去向 |
|---|---|---|
| G-DEC-1（54 裸 n/a） | **闭**：NA-5（C01/02 avm/a2r + G01–08 ac/av/avm，28）、NA-6-I（I01–11 avm/a2r，22）回填 `na:m1`；G01 ac/av 两 raw gap 专属主体排除 | applicability NA-5/NA-6；binding note `908T05` |
| G-DEC-2（B17–19 ac 面） | 裁定语义族映射；ac 现 `mn.unsupported.frontend` 泛化码不满足可辨识原因 → **保 `diff:pending`**，owner=mn 诊断面（同义透传 H 或后继） | 近邻 NB17/18/19 三引擎 2/2/0 实跑 9/9 绿（区分性成立） |
| G-DEC-3（六负例 a2r） | 裁定前端拒绝面、不造金样；a2r 现仍 emit（B17 探针 1 产物）→ **保 `gap:a2r-golden`** 真缺口，owner=旧 a2r 线（242） | [a2r 认证规范](a2r-golden-certification.md) 诊断面节 |
| G-DEC-4（H/I07/I08） | **闭**：I07/I08 ac NA-3 继承澄清（2）；H01–03 ac NA-3 新（3）、avm/a2r NA-6 新（6）；H AV 仍 required（ok:P866） | applicability NA-3/NA-6 |
| G-DEC-5（双层） | **闭**：核心 412（required 285 / NA 95 / 已裁定差异 32）+ 扩展已承诺 AV 28；U 非AV 30 NA-4、H 非AV 9 NA；O 非AV 45 **FUTURE-O**（保 gap，不计 required，不隐出报告） | acceptance/2：required 313，hash `35ad43aa…` |
| F-REG-1（五态回填） | **闭（批准范围内）**：61 格 + T08 a2r `source-bound`→`gap:a2r-golden`；R-A/B/C/D 差异 32 格以 R-ACCEPTED 规则认领（token 保 raw 证据，不阻核心） | contract/acceptance 守恒门 + `cargo t infra-ace-m1` 变异反控 |
| F-REG-2 | 随 G-DEC-2：口径已定，ac 三格为具名诊断缺口（非登记落后） | 核心 blocker 列表 |
| F-REG-3 | 随 G-DEC-3：owner token 保 `gap:a2r-golden`，语义=前端拒绝面 | 同上 |
| F-REG-4（G01 av 陈旧 owner） | **闭**：G01 av → `na:m1(NA-5)`；**勘误**：822 实际已归档 r5（9/9，类型集合 Spec 已交付），903 "822 从未立项"为文字错误；G01 适用性按 a2r 认证主体裁定，非凭错 owner 直接 NA | binding note；本节 |
| F-INFRA-1（852 脚本） | **闭**：.NET Split（862 同源规则）、去 attachments/cases.json、applicability/manifest 严格校验、strict acceptance/2；contract/source/acceptance 当前全绿 | 908 T-06/T-07 |
| F-INFRA-2（oracle 不入库） | **闭**：[ace-m1-scenario-oracles.md](ace-m1-scenario-oracles.md) 沉淀 40 场景 oracle，manifest 绑其 SHA；旧附件 hash 仅历史线索 | 908 T-03 |
| F-INFRA-3（B18 avm） | **重探针已执行**（2026-11-01）：avm 错误完成、无 typed stage/code、消息文本可辨识 → 仍 infra 分类，保 `diff:pending`/`gap:legacy-b18-infra`，owner=legacy avm 诊断通道 | binding note `908T04` |
| F-INFRA-5（892 固定组） | 不变：独立后继事项，不随 908/904 关闭 | — |
| 其余能力族（F-SCN/F-AV/F-CON/F-NAT/F-A2R） | 不变：owner/关闭 oracle 沿 §5；FUTURE-O 45 格显示为 future 账；NA-3 扩展后 H ac 3 格不再计 ⑦ | §5 |

**M1 现状（acceptance/2，2026-11-01）**：core 247/285 红（38 真实 blocker：B16/E03 a2r
名解析 2、B17–19 ac 3、B18 avm 1、a2r 诊断面 7（六负例 + T08 归一）、scenario 观察通道 14、
AV 故障/代际/共享可变 3、mn 复合场景面 7、R07 a2r 未定型 1）；extension 28/28 绿；future 45 显示；
cert 38 gap；`can_claim_m1=false`。**治理交付完成 ≠ M1 通过**；签章待 §8 顺序④。

## PLAN-911 后状态（2026-11-02，SD-04；§11 冻结表为 908 历史）

- **F-REG-2 / G-DEC-2（B17–19 ac 面）**：mn 诊断面已交付——native `mn.source-diagnostic/1` 通道把共享
  编译器真实 raw stage/code 透传到父侧严格判面；三格由 `diff:pending` 回填 `ok`（依据正式父报告
  事实 artifact，非 token 手改、非旧 generic 绿）。owner 关闭；G-DEC-2 的"族面不满足可辨识原因"
  前提已消失（原因可辨识且来自真实编译器）。
- 不变：F-REG-3（a2r `gap:a2r-golden` ×6）、F-INFRA-3（B18 avm infra）、F-NAT-4、38 核心 blocker 其余项、
  future 45、cert 轴——本包不触。
- 新具名边界（非阻塞，owner 已记）：`syntax-not-converged` 严格 fallback 无端到端触发形状；
  `OwnedSourceMnSession` 暖 API 未接 typed 通道。

## PLAN-914 后状态（2026-11-02，SD-03；§11 与本段之前的冻结历史不重写）

已由 914 的真实父报告关闭（binding token `ok`/`ok-emit`，`source-bound`）：

- **E01/E02（8 格）**：四路径的值 + trace `ba` 均来自真实生产入口（AC PE stderr、
  AV step.trace、legacy stdout、a2r 金样 + 双侧认证），区分性反控常驻。
- **I04 ac/av（2 格）**：declaration/layout 变更按 D-05 判据——冷声明前端如实计量 +
  依赖精确 rebuild；未受影响 defs 的 lower/encode/codegen/load/工件IO 增量为 0、resident
  MIR/COFF 身份承载（AC collections 失效轮；AV data-incremental 的 stable/rebuilt 集合）。
- **I09 ac/av（2 格）**：不兼容替换被**按实际策略拒绝**（AC typed
  `mn.incremental.refused.signature-or-context-drift`；AV `env-contract`/`link-rejected`），
  且拒绝后旧成功状态仍可执行；无隐式重解释、无 OSR 要求。

**真实残余（不由本段关闭）**：

- scenario R 族其余格（R01/R03/R04/R06 的 ac/av 逐 owner 事件通道等）、B18 avm、
  native loader/OSR 面、G 族 a2r 复合认证缺口 —— owner 仍为 ACE 路线与旧 A2R 能力线。
- AC/AV 的 avm/a2r 两列在本轮 scenario 内是 `na:m1(NA-6)`，不是「已交付」。
- 本段不签整个 M1 acceptance；`verify-ace-m1-852.ps1 contract` 绿不等于 completeness。

## PLAN-916 后状态（2026-11-02；§11 与 911 节为历史）

- **F-A2R-1 名解析与 F-REG-3 六负例**：旧 A2R 生产 single/project/adapter 已交付 typed 前端拒绝，B16/B17/B18/B19/T06/R09/R10 与 E03 a2r 列均为 `ok:P916`，八格 owner 关闭；不再保留 `diff-no-name-check`/`gap:a2r-golden` 作为这些格的当前状态。
- 新 17-check suite 与三份 NB emit reference 已接固定组；完整 a2r 391/391、base97、NB 908、infra 23 通过。原始 negative source 与其他后端差异不改，manifest 只更新 binding hash 与追加 916 说明。
- 不变：R07 a2r 未定型发射、T08、38 格复合认证、B18 avm infra、scenario/native/console 其余 owner、future/cert 账及 `can_claim_m1=false`。本包关闭八个 a2r 格，不重开 242/908，也不签 M1。

## 917 两格状态（PLAN-917，SD-06）

- `gap:a2r-untyped-emit`（R07）：已关闭——发射定型（单/项目两入口），支持矩阵冻结，
  认证域实跑 42；残余：同根双 `.mut` 前端准入欠账（记入 KNOWN-DEBT，非 917 范围）。
- `gap:a2r-golden`（T08）：已关闭——冻结金样（含界断言块）经独立审阅；
  Auto trap 认证通道仍为 cert-gap（行为观察≠trap 认证）。
- 其余 38 格复合认证、`can_claim_m1=false` 及新 HIR A2R 迁移账保持不变。

## PLAN-915 后状态（2026-11-02；上节为历史）

- **F-AV-1（R02 scn av）与 F-AV-2（R03 scn av）关闭**：两格由正式场景套件
  `case-ace-m1-resource-915` + 父侧冻结表见证（逐 attempt 注入 / typed 授信 Place
  使用前拒绝，dangerous access=0）；binding.tsv 对应 av 列改为 `ok:av-resource-915`。
- 关闭的是 **av 列**：scenario R02 的 ac/avm/a2r 与 R03 的 ac(diff)/avm(diff)/a2r
  保持各自状态；`MutRef` 通用失效句柄校验、native/旧 AVM 对应格、R01/R04/R06 事件
  通道、B18 avm 与 38 格复合认证等残余 owner 不变。
- 计数口径：本轮只改两格 av 值，不重算 `can_claim_m1`（仍 false），不签 M1、不改
  分母/批准规则/其他格。

## PLAN-920 后状态（2026-11-03，SD-04；§5/§11 冻结行为 903/908 历史，保留不改）

- **F-INFRA-3（B18 avm）闭**：按本表预置的"干净拒绝带辨识→另记"路径关闭——
  旧 codegen 三拒绝分支发 typed `NameError::ImmutableAssignment`
  （`auto_name_E0203`），`classify_staged` 判 Language/type，avm/run 对
  `src/B18.at` 的观察从 infra error 变为干净 typed 语言负例。binding B18 avm
  → `ok:P920`（非 R-C `diff:accepted`，非 token 手改：正式父报告=
  `ace-m1-immutable-avm-920` 套件 5/5，含 B18 原源负例与 NB18/NB18S 合法近邻）。
  历史行（含 908T04 重探针 infra 实录）原样保留为时间线。
- 诊断语义契约与反控清单沉淀：[legacy-immutable-diagnostic.md](legacy-immutable-diagnostic.md)。
- 不变：F-REG-3、F-NAT-4、其余 ⑦/⑤ 项、future 45、cert 轴——本包只闭 F-INFRA-3 一格，
  不重开 911/916，不签 M1。## PLAN-919 后状态（2026-11-03，SD-04；此前各节为冻结历史）

- **F-SCN-1 剩余格关闭**：R01/R04/R06 的 ac/av 六格由 `resource-events-919` 真实
  父报告关闭（binding token `ok:P919-events`，status `source-bound`，
  enabling `T-05(919)`）。E01/E02/I04/I09 已由 914 关闭——**scenario 观察通道族
  至此清零**（除 915/921 线的 R02/R03 av 仍属 F-AV-1/2 owner，另有 NA 列）。
- **native 侧事实**：R06 的预算/取消旋钮首次由生产路径驱动（wrapper 预置
  `mn_coll_set_budget`/`mn_coll_cancel_after_work`，此前两接缝无任何驱动者）；
  abort sweep 逐帧释放以 MN-EVT 可判。`Frame` 结构加 `seq` 字段同步修正 codegen
  栈槽 32→40（原硬编码为潜伏损坏面）。
- **修入的既有 infra 腐化**：`verify-ace-m1-852.ps1` 构建行仍引用 901/896 已退役的
  `test-cases` feature（master 上即红）——一并修正；`scenario_assets` 泛化为清单
  （914 单对象原样包裹）。
- **不关闭**：R02 av（F-AV-1，915）、R03 av（F-AV-2）、native loader/console、
  a2r 复合认证、B18 avm、M1 总签章。`can_claim_m1=false` 不变。
### PLAN-924（R02/R05 ac 闭合余账）

- **关闭**：R02/R05 scenario **ac** 格（原 `gap:ac-composite-native`）→
  `ok:P924-closures`；关闭证据 = native-closures-924 套件（闭包域）+ 923
  套件与 ace-native 组在 runtime/4 最终库复验（非闭包域）双父门，先绿后
  回填。F-NAT 域内 K06/shared-cell、容器捕获、native loader/HMR、FFI 仍
  具名余项（owner：后续包）。
- **不关闭/不变**：F-REG-3（a2r cert）、R02/R05 的 av/a2r/avm/cert 各格、
  corpus 行、future 45、313 required、32 ruled-accepted。M1 claim 守卫新增
  cert 轴显式条件（严格强化，无弱化——突变控制绿），`can_claim_m1=false`
  仍冻结。
