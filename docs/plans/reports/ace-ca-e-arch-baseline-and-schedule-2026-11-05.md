# CA-E／ARCH正式立项基线与交接（2026-11-05）

用户仅授权正式规划，不施工。本轮按auto-plan-new与verification-contract读Spec/实际代码/收据，新增[PLAN-941](../941-ace-m1-expanded-and-architecture-acceptance.md)，drafting r1、0/15。取材main `5f6737467`，不是运行验证；现存多会话WIP完整保留，未建实施树/材料scope/goal/agent，未执行编译/产品/性能/认证测试。

## 1. 一个计划的理由与交付边界

底层语言/IR/资源/查询/CT/ABI功能各有现成owner与入口，CA-E缺的是完整登记、可信证据关联、严格父侧AND与跨包联合反控，合一个中等验收集成包；不另拆无人消费schema，不另造runner/meter或复制compiler。新registry/只读reader/独立oracle与治理反控可先推进，真正四端/SQ/实例/CT/资源ABI/首编辑的联合验收在实际D齐后做，不把准备阶段放大成实施完成。

CA-E验收系统实现可正确报告red并交付，**只有全部真实必需条件满足才签M1**。底层功能新缺陷/旧认证缺口仍须其owner后继正式包，不由本包改source/expected/NA/分母或伪造证据。这不是将尚未实现能力从M1移除，也不是承诺做完941即无条件M1绿。

架构已写[Design51 §16](../../design/51-shared-a2x-and-hir-to-rust.md#16-ca-earch最终组合验收2026-11-05target-state)，与Design41有限ABI及C路线CA-E相互指向。M1含AC/AV/新HIR-A2R/新HIR-A2C的有限语言域与完整承诺架构机制，M2所有旧应用/目标/host接管与disable仍为v0.6发布条件，未来M3/M4不抢替换路径。

## 2. 现有门的代码事实

| 门/来源 | 当前能证明什么 | 本包必须补的独立判定 |
|---|---|---|
| verify-ace-m1-852.ps1 contract/acceptance | 原binding/applicability/source-manifest/批准hash严格核算；acceptance/2的exit0只指核算有效，默认executed=0 | 每个required原格位到真实case/check/producer证据的完整映射、实际终态与闭包；不读exit0/ok token冒执行 |
| /2 ParentReport | 非空results且全PASS，备注仅本scope；没有天然覆盖新Q4/ARCH | 新父门重建当前source/producer/config/依赖/工具/oracle/义务hash，并检查原始命令/terminal/文件闭包 |
| 原认证/快照 | 旧a2r金样与认证双轴，原证书strict reader；新HIR Rust/C有各自schema/完整actual/reference | 旧/新producer严格分域；新HIR/C必需真实认证，不拿旧golden或四端相等作oracle |
| SYN ac-source-syn-report/1 | 929只交统一语法与父侧P/M/R/IO/身份准入；prefix字段族不是整个ARCH的全部语义 | profile每个必需计量字段和真实producer交叉核对；SQ/INST/CT/ABI另消费真实D，不从SYN总布尔猜 |
| 937 abi/ca-d1-bridge-v1 | typed三腿与真实source-C exporter、candidate/descriptor3/两consumer；source import属938、资源属939 | 原ABI+新source import/resource全方向/来源/生命周期实证，D1不能替D2/3 |
| 893 fixed结果成本报告 | 只读results/command/terminal，缺观测UNOBSERVED，身份不符STALE，零执行/伪PASS拒 | 组合器只核验与引用，不二次meter，不将成本单轮或父摘要代原正式时间/资源门 |

### 原登记静态重算

只用跟踪文件和当前Load-Binding/Get-CellState/Load-Applicability规则在内存重算，未调用验收脚本或产品入口：131行×4=524格，required313（285核心+28扩展）；134 NA、32 accepted-diff、45 future；批准SHA实值匹配。**313格无非绿能力token，但38格required旧a2r有cert-gap。** required-set SHA256=`35ad43aae27ffb2d2c9a162c3dd5b99f971e4cd7b2ed36f4fe747a5634bd770e`。这是登记事实，不是313个当前运行均已执行。

`CE-BASE-01`：G-DEC-3/适用性/scenario Spec将旧金样与认证独立；924后现脚本`$certGapCells.Count -eq 0`以及gap Spec又加严格守卫。当前/2真实claim因38项为false，不会因原script exit0或newHIR认证变true。941 r1**保留现行更严守卫、展示两轴/规则差异，不暗改批准件/旧脚本或绕过旧false**。若希望恢复原批准独立记账签章，需另有用户明确裁定并修订合同；未裁前M1保红。

38项的功能owner为旧A2R复合lowering/认证线，已有历史242 tracker不等新执行包；当前没有本轮替其取新号。941 T-01/T-04持有逐格证据接线审计：有实际本域证书但漏接线可据实映射，缺功能/typed trap/真实认证通道由原线后继包关闭。新HIR A2R已认证也不允许直接替换这些legacy域格位。

| ns:id | 当前a2r token | 功能owner / 证据接线owner |
|---|---|---|
| corpus:B08 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:B11 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:B13 | cert-diff | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:B14 | cert-diff | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:B15 | cert-diff | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:B20 | cert-diff | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:T01 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:T02 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:T03 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:T04 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:T05 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:R01 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:R02 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:R03 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:R04 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:R05 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:R06 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:L01 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:L02 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:L03 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:L04 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:L05 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:L07 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:V01 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:V02 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:V04 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:V05 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:V06 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:V07 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:K01 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:K03 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| corpus:K06 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| scenario:R01 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| scenario:R02 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| scenario:R03 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| scenario:R04 | emit-ok-run-unverified | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| scenario:R05 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |
| scenario:R06 | snapshot-only-cert-gap | 旧A2R复合lowering/认证线 / PLAN-941 T-01、T-04 |

## 3. 929与937交付基线、在途接口

| 计划 | 实际状态与收据 | 给941的正式能力与限制 |
|---|---|---|
| 929 | archived14/14，D29a7d7007当前祖先；树/scope已清 | source-frontdoor/required来源恢复、函数与闭包首编辑、SYN报告已交付；不代SQ/实例/CT/整个ARCH |
| 937 | archived12/12，Dfa35943ac当前祖先；规范已主线读回，树/scope已清 | neutral foreign-contract、native ABI声明IR+新C exporter真consumer、typed自/双向腿；source import/资源未交，不冒938/939 |
| 934 | reviewed r2 11/13，main有最终review pass收据；树488be322f仍在，未据reviewed认D | 接其最终TypeSupply/query/required/真实计量稳定接口；941不接管，不引用未合branch当正式证据 |
| 935 | executing r2 1/12 | 只读冻结/私有草案准备，真实实例主体需D934；总门必须消费实际source实例 |
| 936 | executing r2 1/14 | T-01/私有方案已准备，真实source AV CT及实例组合仍需上游；旧817 deck不顶替 |
| 938 | drafting r1 0/13，D929/D937已就绪 | 新source typed-decl binding/真实native-C import/required与首编辑，公共S/H接934..936释放 |
| 939 | drafting r1 0/14 | 原公开resource/handle/ctx/callback及资源恢复方向；待938，不以937原probe资源pass代替 |
| 940 | 另一会话新立legacy Core/UI parser收口，drafting0/7 | 非ACE语法平台替换，不新增整包D门；仅实际共享H/legacyconsumer面协商 |

C A/B/C（921/925/926）及native923/924、语法927/928、931/932/933均已交付。C首例套件分别3/5/20例、Rust902/904分别4/20例；**这些是首例文件数量，不是完整新增Q4/ARCH分母**，T-01还须逐项映射原scalar61语料、各实际原resource/method/collections及负控、SQ/INST/CT新有限域。不能只取28个C首例或临时能跑的例求共同子集。

## 4. 验收共同合同及实现边界

新registry=`ace.m1.expanded-registry/1`，新report=`ace.m1.acceptance/3`（旧/2并列）；旧批准、required集合不改。Q4语义义务与机制/方向义务分层，稳定ID/source/oracle/profile/caps/target、原样本/预算/owner/D与真实entry逐项固定。新source和typed、旧与新目标、daily与cert、preview与正式签章隔离；缺行/未知/双claim/空选择/伪PASS/部分执行/failed terminal一律不可签。

- 每端对独立oracle，不只pairwise相等；普通负例以真实前端族拒/禁止后续codegen运行为证，不拿工具失败凑负例；valid value/trap程序的完整actual与独立reference须真编译执行。
- 全链真正P/attempt/discarded/M/R/IO与未改函数/实例/binding生产0、其payload额外IO0；事前读依赖定义affected set，callee body只变不被caller读取时caller不重产而实际新实现可见，link/assembly/R另记。
- restart后无预热第一source编辑、首错误→旧root可用→修复→再次restore/编辑；required页/cap/owner/坏schema使用前拒，不能隐式cold或将whole工作藏R/M；失败修复须下一实际读证明。
- ARCH-01..08逐原子义务闭包：语法；SQ/实例；IR/Q4；真实AV CT；cache/P/R；资源/ABI；同PID在线更新；证据/父侧准入。CT必须断真实AV边即红、hit不执行，meta/实例/子CT依赖完整。
- 资源/fault/cleanup/last-consumer/lease/GC沿原事实，HMR坚持同PID/入口一次/旧在途链保活、新调用新代；动态loader/完整AJ不新增为M1前置。
- 每域原更严帽/样本/深链/512/平台/ignore条件AND，暖P90≤2000ms；ABI60/5/70和8MiB+8MiB不能被A2X120/30/360或48MiB覆盖。新组合器单输入/registry/report4MiB UTF-8字节、self live48MiB/父privateΔ128MiB、流式hash/分配前预留，现有Job/meter为唯一计量源。
- 每项证据有原始执行与terminal/实际文件闭包身份，不把child字段自报当独立真实性；不能许诺任意手写本机历史仅靠hash就认证。旧38认证差异照实阻塞；系统逻辑正控不进M1真实证据。

## 5. 写面与可并行顺序

| 阶段/面 | 941工作 | 前置/单写边界 |
|---|---|---|
| 先行私有P | T-01..04登记、pure reader/组合器、独立oracle与反控，首例从T-02进正式文件化TestCase | 不等935/936/938/939整包，但不借未审分支充D；最小首例取当前已交四端入口，必要H取短窗 |
| source/query/instance/CT | 消费稳定报告/required页/计数，不实现solver/CT/资源分析 | 934→935/936各actual字段/版本/路径owner按稳定SHA释放；最终联合T-12需真实D934/935/936 |
| ABI/source资源 | 方向矩阵与独立布局/调用/资源证据reader | 937可先对接；938/939实际imports/资源页/callback等待其正式接口，最终D齐 |
| H | contract/SDK/worker/父decoder/mod.rs/Cargo/lock/test-groups/cert reader/fixture索引 | 全文件单写，producer→各reader→各状态路径→兼容门全列；释放即让窗，不占H等D，不当append-only可并写 |
| 性能/编译认证 | 原实际runner和原样本，941只读关联/AND | 与934..936/938939正式采样单实例错峰；不启动第二meter，不缩压力 |
| 最终 | T-12真实完整证据并集→T-13健康/SD→独立review→merge | 所需D全部就绪；若旧cert/功能仍缺，系统可交但报告红且不能签M1；不预占旧认证后继号 |

不改在途计划进度/状态或让它们为总验收补临时伪字段。各原owner交实际source/hash、schema/recipe、计数/required页、原样本与资源域、完整producer/reader/证据清单、稳定SHA/释放收据。新reader适配可并行，真正共享面按实际交集排窗。

## 6. 本轮内容指纹

SHA256是本次只读取材定位，非执行PASS；后续T-01漂移须核实际影响，不用旧hash强制回滚。技能来源也记录供跨工具交接。

| 路径 | SHA256 |
|---|---|
| `scripts/verify-ace-m1-852.ps1` | `1a9c3bb809609a1b1e310b1723c205cfe17ff802b94f165259dc31dcea86f829` |
| `scripts/tests/test_ace_m1_acceptance.py` | `d72fee5402d72805dda79d7f027ec454b6c82874d6f3c711fdf7d09cfaacc478` |
| `scripts/test-governance/ace_cost_report.py` | `4156207e7d5c2f08f852243a495b652af4e912c665b299691bc0cd397f6ebe1f` |
| `scripts/test-groups.toml` | `755b6632da1b74c65e5791a1e00923acf66c35fff1bbef73795d98d93cc73d7e` |
| `crates/auto-lang/test/cases/ace-m1/binding.tsv` | `303cdde9e8b5d56a8debeb5ff261519d2ee5c263684a4eefc84fafbe3917a259` |
| `crates/auto-lang/test/cases/ace-m1/applicability.json` | `74c9109bbe7ec8596e6fa1bbc2ab9f4f5640725a748d0e63e53eb5e07ee275c0` |
| `crates/auto-lang/test/cases/ace-m1/source-manifest.json` | `5209f30010be94da007343b502e85f9ecdb85a57940315dfbad8314e4589db5a` |
| `docs/plans/reports/ace-m1-scope-decision-2026-10-28.md` | `874ccc1198bf23c284fa588a6dfcdee80eecc8f5a0e392b88b34ddd74db0875b` |
| `docs/plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md` | `3a623ba38a279197cc06cabc4d58b56ee9d4330688ece38c580917ed783b60fd` |
| `docs/plans/reports/ace-m1-architecture-completeness-decision-2026-11-03.md` | `b1d9b5e7a75812fc56a8adf7fe6d289f508525c38744bfaba7c2b1aba4fd3718` |
| `docs/specs/auto-test-case/ace-m1-conformance.md` | `3c2d5428802d1149772a752baab5d1d56431c673bcef22ad0051cd87212a9588` |
| `docs/specs/auto-test-case/ace-m1-applicability.md` | `787e74b4432c1a061cc8b0ab2b2506649962bfbde952fddcfbdbd5ad35fa1bfa` |
| `docs/specs/auto-test-case/unified-source-syntax-admission.md` | `0b8230d6ac34a2baeeb74ea9119604305f5b5aba1ad6d67b02e8472f02d66da0` |
| `docs/specs/auto-test-case/ace-ca-d-bridge-certification.md` | `3c5353b78f06c6548c1d95a80ac5f0b1642167e4720becb9f3ce6fd18529072c` |
| `docs/specs/auto-abi/shared-interface-and-scalar-bridge.md` | `c15d5ee6566bc8e0c7af7571bcd90af4c8243b3b6a6cf4620a5821bf19b04618` |
| `crates/auto-lang/src/tests/syn_report_929.rs` | `e758d63806b928db630bd53777a1dca109a73a82d61a6430972360d25bc39eea` |
| `crates/auto-lang-core/src/test_cases/abi.rs` | `4802937291987ccd24954bb5fb7ad441f9fa03bfe27c70ccf732a91bb3dc6eef` |
| `crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs` | `dbc7d3151928d6c29890e26cbc347d36b80d07ef67577fe2823da15e65ed156f` |
| `crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs` | `e7155d86549e17f237b7c4832891ad464ab2323e98b792bf09ccc61cdbf882c3` |
| `crates/auto-test-case/src/contract.rs` | `f25b6d817c66f1400374a88e3e11f8ec18dfa45e779c34261cea83a905d5b62c` |
| `C:/Users/zhaop/.codex/skills/auto-plan-new/SKILL.md` | `03ce42011842d795ba3b62ac55a79379f560c64d2c666b0569e26eb5703e4535` |
| `C:/Users/zhaop/.codex/skills/auto-plan-new/references/verification-contract.md` | `73ea4ef36cd8b9010f17f648201a685338272d85d1eef5c8862b0f1ddc14e0bf` |

## 7. 规划静态核查

2026-11-05 stage:new / PLAN-941 r1 / outcome:pass（仅静态规划就绪）。0..10节及drafting/r1/0步/15任务metadata一致；T15、AC13、V18、SD6逐类唯一连续、引用有效，work13/review1/merge1全部待执行。具名现存固定组逐项对当前配置核实，新组/新entry明确为未来任务；16处本包及新增关键本地链接可解析。38项legacy-cert清单与313集合重算相符；22项输入内容hash未漂移，929/937交付确为当前祖先。941在active/archive/历史archived唯一；取号期间另会话新立940与942，均保持不动，中央next-id核查时943（与最大编号+1相符），本会话取号锁已释放。Git diff --check通过；本轮未改crates/experimental/scripts或current Specs，旧批准/source/expected/分母文件hash未动，新增修改只在本包计划/设计/路线/报告，原WIP完整保留。

收尾主线已推进`7327e7cc9`（随后两个age路线docs提交），22项取材hash与D链未受影响；934仍reviewed11/13未见D，935/936及938939状态与§3一致。未构建/运行产品、TestCase、四端认证/性能/资源/独立实施复审，未建941实施树或材料scope。静态通过不代这些未来证据，不代表M1通过。
