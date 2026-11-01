---
plan_id: PLAN-908
status: archived
feature_name: ACE M1规则、登记与验收入口归一
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/project.md]
new_spec_components: [docs/specs/auto-test-case/ace-m1-applicability.md, docs/specs/auto-test-case/ace-m1-scenario-oracles.md]
touched_goals: [GOAL-002]
affects: [auto-test-case, ace-m1, test-infrastructure]
depends_on: [PLAN-903, PLAN-893, PLAN-862]
current_step: 11
total_steps: 12
---

# [PLAN-908] ACE M1规则、登记与验收入口归一

## 0. 变更摘要

承903正式审计，归一M1批准适用规则、12列登记、独立oracle来源与实际验收入口，修复852脚本的解析/未跟踪附件依赖。报告区分base、原核心、扩展已承诺profile、未承诺能力账和a2r认证轴；原131行/524登记格保留，不改源码/expected/golden/编译器语义，不因回填token自动获得执行PASS。

**2026-11-01用户已将G-DEC-1..5全部按推荐方案裁定，已有批准直接继承。** 精确范围见§5.1及[决策件§13](reports/ace-m1-scope-decision-2026-10-28.md#13-补充裁定g-dec-15plan-9082026-11-01)，不再等待五项投票；批准适用/判据不等执行PASS。904已交付14/14（12f9adf18），893/903已归档；含type/enum片段修复不再是等待项。本轮仅修订规划为drafting r2、0/12，不建实施树/测试、不改binding、生效规则/current Spec或他方Plan。

依据[903审计](../specs/auto-test-case/ace-m1-gap-audit.md)、[原适用性裁定](reports/ace-m1-scope-decision-2026-10-28.md)、[ACE路线](../design/strategy/auto-native-backend-evolution.md)。本包是规则/证据接线收口，不承诺修完M1全部能力或签M1全绿。

## 1. 目标

- G1：按真实批准建立逐格适用白名单/版本，保原登记、expected与迁移前态；无依据NA/范围排除不可绿。
- G2：诊断按原语义oracle正确判决，保raw stage/code/message及可辨识原因；通用unsupported/任意非零/infra不充合法拒绝。
- G3：必要source/oracle及40场景关闭合同成为tracked来源，fresh树不依赖主检出旧附件，不伪造缺失源码或弱化原独立oracle。
- G4：统一现有入口的strict完整性/证据/适用与双轴判决；base97或局部选择通过不冒M1，认证不隐式运行，新A2R不冒旧路径迁移。
- G5：残余问题保能力/判定owner和关闭oracle；消费893正式约定，与904公共H按符号交接，不改原样本/预算/条件。

非目标：旧a2r名解析/未定型/复合发射、native console/loader/闭包ABI、AV故障/代际能力施工、904后端或13慢测优化；不改算法/类型/所有权/ABI/源/expected，不造金样/证书补洞，不修全仓attachments债，不造runner/动态目录/影响图/自动任务，不重开852/862/893/903。允许本包正确交付后M1仍有真实gap。

## 2. 架构方案

`原binding+tracked源/oracle+批准决策 → 有限适用/来源sidecar → 原TestCase/固定组或有效正式证据 → 单一strict M1核算 → base/core/extension/future/cert分别报告`。有限M1元数据不参与全仓发现/选择/逐测试新鲜度或消费者图。

优先修`scripts/verify-ace-m1-852.ps1`原contract/acceptance和共享解析；source/snapshot/lifecycle/receipts/certify职责保留，certify仍显式。T-02如需只读辅助模块，只能处理表格/规则/原报告准入核算，不启动另一执行体系；真实运行沿cargo t/原auto-case/contained设施，不复制compare/certifier。

## 3. 技术栈

语言仓既有PowerShell/Python标准库、AutoDown/TestCase、fixed-tests.py/test-groups.toml及strict证书/原报告。拟新增M1有限测试资产`applicability.json`、`source-manifest.json`、规范正文与治理反例；这些是长期测试合同数据，不是计划raw证据。无新增三方/编译器生产实现/跨仓改动。

## 4. 需求分析与背景调查

### 4.1 授权与交付基面

用户本轮原话：“908的G-DEC-1..5都按推荐方案裁定，已有批准直接继承。请更新计划和相关决策文档，核清904交付后的实施前置、写面边界及收口顺序。本轮只做规划，不施工。”此为r1 §5.1五项推荐的正式批准及r2文档修订授权；不再向用户重复索取这些选择。未授权实施/构建/探针/认证/清理、修改实际binding/生效规则或联系他方。r2主线基面`174652b5c3b8b9631ec58e8814b4c4b869b5e462`，保留多会话WIP；r1的add1df088/904在途观察保留在§9历史记录。

**2026-11-01实施授权（/goal 完成计划908，/auto-plan-goal 链）**：用户以目标会话授权整链交付——T-01..T-08 work（含创建唯一树`D:/autostack/.wt/lang-908/auto-lang`、分支`plan-908-dev`与`plan-artifacts/plan-908/`材料scope）、T-09 独立review、修复上限内的修复、T-10..T-12 merge（Spec 沉淀、归档、guard后树/分支/组/材料清理与落地检查）。授权不改变本计划目标/验收阈值/范围/仓库集/预算；五项裁定不再重索，后续阶段继承此授权。

903 archived10/10，delivery `18bac254b`为HEAD祖先(exit0)，三canonical Spec和cleaned收据已交付。它完成静态完整审计及base97/97窄重验，**不是524格当前实际PASS或M1签章**；原材料已清，不复活。893 archived12/12，delivery `b71b7343e`为HEAD祖先(exit0)，不再是等待前置。904 archived r2、14/14，delivery `12f9adf18`为HEAD祖先(exit0)，归档/沉淀916454628+ff5947e08、goal读回174652b5c；lang-904双树/分支/组及材料已清，无hold。其既有批回归due按主检出单实例/用户择时规则处理，不在本轮启动，也不把904写为仍在实施。

### 4.2 来源/哈希与勘误

| 当前来源 | SHA256 / 静态事实 |
|---|---|
| docs/specs/auto-test-case/ace-m1-gap-audit.md | `34e35f4b78d8b436dca1e67735153e4d85f53c3467ccaf65d1deb6496c3bd784`；G-DEC/F-REG/INFRA后继已登记，结论仍须对原源/批准范围核对 |
| 同目录ace-m1-conformance.md | `2d67ea4700b9ef7d0ce21dacf51dc3fb75269682a2caac64d8aee43bffc517a5`；原103+扩28、四路径与原source/typed区别 |
| binding.tsv | `d9f05da057dfd658df09a7725aa391bc4c9e14993e13e9846724160915ff5d7a`；131×12=91 corpus+40 scenario、524格、裸n/a54；原source/expected不改 |
| 2026-10-28 scope decision | r1读取SHA `7e02a674167604266d3a63260fc663551bc8529ef6a7f76ecf8f3ba81a337382`保作历史；本轮增加2026-11-01 §13正式批准，r2完整hash见§9；NA-1..4/R-A..F与双轴直接继承，NA-3已含I07/I08 |
| scripts/verify-ace-m1-852.ps1 | `18140cc56b221f5dcae2479fad0b6729045e971d93a3f94c44b18d6a2a70be76`；旧split、attachments cases.json、窄ok/认证混账待修，本轮未跑反例 |
| ace-test-policy.md | `a280546a785580a54533a5b9c9d9898345fd2e0082823e080b31d32975f3a048`；893有限策略/原runner/起点与资源约定已交付，infra-ace-policy组存在 |
| 主检出旧SCENARIOS/ORACLES | 目前仅读存在，SHA分别`9ca07a74f400664ec7367815baf6985bb6cb7b9a91ac9fcca99d9a4b27fb2694`/`01371309f0c98b1e3234816a14383b5bd5a7da42bdd4f94fa09979854ee12a30`；cases.json缺失。旧文标designed-not-executed，现存文件/hash不是单独批准/执行证明 |

事实澄清：822实际archived r5、9/9，类型集合Spec已交付；903 F-REG-4的“822从未立项”是文字错误。G01适用性应按a2r认证主体核，不凭错owner历史直接NA。903历史收据/冻结数不重写，current导航有来源勘误。

原B17/B18/B19 expected为`type`、`assign-immutable`、`missing-return`（语义族，不是统一后端错误码字面量）；T06/R09/R10为`borrow-escape`、`nominal-type`、`use-after-move`。G-DEC-2/3据此区分语义与观测面，不能虚构原oracle要求或拿泛unsupported充成功。

904正式交付入口为`docs/specs/auto-a2r/collections-hir-backend.md`：recipe `ac-a2r/hir-collections-emit-v1`、fileset `auto.a2x.rust.collections.fileset/1`、cert `ace.a2r-hir.collections.cert/1`、source profile `auto.source.collections.draft`、AXS3/AXR3。`warm_edit_ranged`绑定base全文sha、UTF8声明range/同名片段，产物manifest绑定替换后完整源码与canonical execution_config；type/enum片段及同context异body凭证校验已交付，不再等待共享S修复。资源事实沿CollectionsBodyFacts及显式清理/失败原子性，908不得重新推导或更改。已知边界：unit-arm限定构造不支持、call result须先bind再move，List参数/返回/字段/variant载荷/嵌套等受profile限制，不能据新通道虚报旧M1全覆盖。

904的H交集已随正式合入释放：`crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs`、`mod.rs`、`crates/auto-test-case/src/contract.rs`、`scripts/test-groups.toml`及verify脚本。当前新owner仍须核：905 execution_done5/7、lang-905无WIP（研究审查/沉淀，不等生产治理已实现）；906 executing2/8、lang-906有真实WIP，已触`scripts/test-groups.toml`、auto-case/lib/features及Cargo依赖。908不可覆盖/提交其WIP；固定组/宿主构建或注册按具体符号短窗交接，不等待906整包。当前已批准的906决策也不重索。

903事项映射：F-REG-1..4、F-INFRA-1..3属于本包已批准规则/入口/来源收口，F-INFRA-4只核受影响身份再复用。F-INFRA-5（892并列新A2R套件无独立固定case组）仍交A2X测试接入后继（未立项）；904已归档，新增collections组未关闭892原suite的独立注册，不回开904。实际套件`crates/auto-lang/test/cases/a2r-hir-892/a2r-hir-892.suite.ad`存在，但不是本包M1旧a2r身份，不成为908前置，也不因本包存在就重复认领注册或把ACE准备塞进普通legacy a2r组。其他能力族只保owner/oracle，不宣称随本包关闭。

## 5. 详细设计

### 5.1 G-DEC-1..5正式裁定与影响（2026-11-01）

用户已明确将r1五项推荐全部批准，原话与日期见§4.1及决策件§13。下列方案均为已裁定范围；实施T-01核对批准来源/版本，不能再把五项投票当待办。规则文件/登记/报告尚未实施，批准不替代其验证；未来若需扩大范围，先记录新增差异并修订合同，不自行类推NA。以下替代方案仅保留r1未选方案的影响说明，不是当前分支或新审批入口。

旧授权边界要区分：G-DEC-2原expected为语义族、G-DEC-3“诊断负例不生成可执行金样”均已有source/Spec依据，不重新请求是否接受这些既有语义；本次需选的是跨backend观察映射与六格归属/回填合同。G-DEC-4的I07/I08直接继承旧NA-3，新增H范围才是新裁定。用户已明确的部分不得被后续owner再次当未知批准阻塞。

#### G-DEC-1：54裸n/a怎样归属规则？

**已裁定：按原测试主体/接口，批准逐格白名单NA-5/NA-6；不以“难/没实现”排除。** 54裸格中6个H统一归G-DEC-4；本项处理其余48裸格，另批准G01 ac/av两个raw gap的专属主体排除，共50格。

| 已批准规则（待实施接线） | 精确白名单 | 数量/现状 | 依据与重入条件 |
|---|---|---|---|
| NA-5专属CTE/认证主体 | scenario:C01/C02的avm/a2r；scenario:G01..G08的ac/av/avm | 28=26裸n/a+G01 ac/av两gap | C01/02是原ACE查询/权限/回灌入口，G是原a2r认证系统；这些路径不是该主体。后继正式消费相同能力/新producer时先冻结接口/规则再重入，不排除普通执行/代码生成旧义务 |
| NA-6原ACE增量会话接口 | scenario:I01..I11的avm/a2r | 22裸n/a | 原场景不要求旧批处理/快照框架新增ACE函数产物/在线会话API；旧VM缓存和旧a2r快照仍验，不泛排全部增量性能。正式接同场景接口或迁移consumer时重入 |

**影响**：只产生经批准的适用结论，不产生执行PASS；保原观察和G01 raw gap来源。替代方案=暂不批准，保pending/逐格查消费需求，会阻规则归一，不阻904。H6不得本项与D4重复计数。

#### G-DEC-2：B17–19 native用精确错误码还是语义族？

**已裁定：按原expected语义族显式归一，保raw stage/code/message及可辨识原因；不要求后端错误码字面相同。** B17必须type、B18 assign-immutable、B19 missing-return，通用`mn.unsupported.frontend`、syntax或非零本身不足。B16 resolve按同一原则核对，旧收据保历史。

必须真执行正负控制：原例不改；新近邻把true+1改为1+1得2、let改var赋值得2、缺返回程序补return0得0；错族、缺原因、崩溃和合法近邻被拒都使门红。T-02冻结合法语法/新ID/hash/独立oracle，不覆盖原expected。

**影响**：批准映射不等3格自动ok；无有效原因观察保`gap:mn-diagnostic-observation`/pending证据及owner。可H同义透传已有原因，不能新增语言检查；需大worker协议/生产修复则后继或修订。替代=要求精确stage/code一致并立诊断专项，范围/工作增加、3格继续阻塞，不能把新要求倒写成原字面合同。

#### G-DEC-3：六个a2r诊断负例要不要Rust金样？

**已裁定：B17–19/T06/R09/R10保持required，按原outcome=diagnostic正确前端拒绝面验，不制造非法程序的可执行金样。** 与现认证Spec“诊断负例不生成可执行金样”一致，实施冻结六个语义族/报告映射。

**影响**：原gap:a2r-golden按实证归诊断接线/静态检查gap，未正确拒绝仍未完成；不能NA/accepted或用rustc编译失败过门。B16/E03的R-E解析、R07未定型发射及复合认证欠账保留，新892/902/904不替旧a2r身份。替代=保判面pending，或另定义诊断文本产物金样；后者需新oracle，非现代码生成金样自动延伸。

#### G-DEC-4：H常驻行及I07/I08对哪些引擎适用？

**已裁定：I07/I08 ac按原NA-3回填（2格旧批准直接继承）；新增H01–H03 ac归NA-3，H01–H03 avm/a2r归NA-6。** H三行AV仍required。native loader/闭包/Dyn保独立能力owner/重入条件，重启exe不冒在线HMR。

**影响**：本组11格=2旧批准澄清+9本次新增批准（native3+legacy/a2r6）；H6只在本组处理。r1未选方案若要求native H或legacy/a2r同接口，会引入常驻loader/新会话合同；本次未选，不塞908/904。

#### G-DEC-5：原103与扩展28怎样签M1？

**已裁定：保131行登记，冻结“原103核心+扩展已承诺profile”双层验收，由承诺合同决定而非看已绿选择。**

| 层 | 已批准适用/去向 | 数量/影响 |
|---|---|---|
| 核心 | 原103×4保持登记，按原规则与明确批准白名单判适用 | 412格，真正gap/infra/未执行仍阻核心，不丢旧行 |
| 扩展已承诺 | U01–10 AV、H01–03 AV、O01–15 AV纳扩展required | 28格，同源/独立oracle/有效证据仍须验，不因历史绿免复核 |
| 扩展NA | U非AV30沿NA-4；H非AV9按D4裁定 | 39格，有规则/重入条件，不冒执行 |
| 扩展未承诺能力 | O01–15 native/avm/a2r45格保真实gap/后继，但不要求全backend先补console才能签核心M1 | **scope已批准**；不能把45格改PASS/NA、删除或隐出报告，产品/M2仍欠账 |

已批准签章=核心适用义务+扩展已承诺profile齐备且独立oracle/来源/资源增量等适用门成立，同时展示131/524全登记剩余gap；不能写成“131行四路径全部支持/clean”。a2r认证另轴，认证缺不否定获准金样轴，也不冒认证PASS。

替代①扩28所有路径都要求（尊重旧NA），则console45等须先交付、M1显著延长；②仅签原103、全部扩展只作M2信息，对已宣称扩展复验更弱，不推荐。选择必须先于结果核算，不得按当前绿缩分母。

批准映射去重：D1 NA5=28、NA6-I=22；D4 NA6-H=6、NA3-H=3、原NA3-I=2，共61唯一格，含54原裸n/a、7原gap；其中2旧批准澄清、59本次新增批准。D5另批准45 console扩展gap列future、不计本次M1-required，不与61混算。此为已批准合同，尚未实施规则/登记迁移，更不是执行PASS增量或clean预测。

精确键沿原binding namespace：C/I/G为scenario，U/H/O为corpus；故NA-6的H6、NA-3的H3及future O45均须取corpus，不能误写scenario或与40场景混算。本轮静态逐格查证已纠正决策附表namespace，不改变用户批准的原行/路径范围。

### 5.2 数据、批准与迁移保护

拟tracked `crates/auto-lang/test/cases/ace-m1/applicability.json`只记有限M1规则、逐格scope、批准来源/版本/重入条件；T-02冻schema/上限。未知/缺字段、重复(ns,id,path,profile,axis)、不存在行/producer、无来源NA、过期或扩大白名单拒。不成为全仓目录/影响图。

binding仍12列，131行现有id/ns/feature/outcome/expected/source绑定保持。仅批准状态与有证据变化的路径token/说明回填，每变保old token、规则、正式证据或仍缺owner；原NA-1..4/R-A..F继承，不向外类推。后续合法新增行须来源/profile/scope显式更新，不能静默忽略，也不拿131永久常数拦正常新增。

表/规则/source-manifest/报告形成版本身份。先生成验证迁移候选/逐格diff，冻结来源/批准后有序提交；坏输入/取消/写盘失败不能半套生效，下次contract再读须真通过。归档owner不等已缺/已闭，G01文字勘误与适用性分别论证。

### 5.3 tracked source/oracle与附件依赖退出

拟`source-manifest.json`冻结本M1实际source/reference/oracle/suite/check及source/typed/scenario起点/hash，不全仓动态枚举。保58原源+5typed、40场景与U/H/O分解、正式表外suite引用，不拿目录78源冒唯一分母。缺真实源码的typed/scenario不造D*.at，按原起点/profile验。

拟新`docs/specs/auto-test-case/ace-m1-scenario-oracles.md`沉淀40场景输入/独立oracle/反例/资源增量/起点/owner和认证双轴，指tracked原源/参考；原source值/诊断仍由frozen输入决定。旧附件hash只线索，须对照852/后续822–866/889/891/894实际Spec及独立参考；冲突具名不猜，不跟旧draft改现语义。

保W原位型不用JSON浮点，Dyn内层坏载荷好外hash仍拒，E01顺序ba，资源LIFO/故障回收/代际危险读0，I未改真实生产0/对象身份/恢复与在线不以重启代替，G真实actual/reference及日常零编译。旧I11 P95<1s按858已批准开发期P90≤2000ms及原样本/结构/资源门标明取代来源，不改其他profile原门。

fresh树只靠tracked源/规范/清单即可contract，不恢复attachments/raw。D-899-A读796/797债不混本包，903/893旧材料不复活；历史oracle存在不代表新运行通过。

### 5.4 诊断与有界证据复验

先核source/oracle/工具/producer身份的正式证据，缺/过期/只raw ok不能冒新执行。必要时原固定组/auto-case窄验B16–19、6a2r负例、B18 legacy及合法近邻，不改源或修编译器。

B18 legacy干净accept才能沿R-C记accepted；干净正确拒绝按原oracle；仍infra保`gap:legacy-b18-infra`/ERROR及后继，不冒语言差异。诊断H只归一/同义透传已有可辨识原因并保raw字段，不在聚合层新做类型/名解析；事实/协议不足就gap并交后继。统一unsupported/任意非零须被2/2/0近邻及错族控制挡住。

本包允许正确交付时M1仍有gap，所有非关闭项给原输入、能力/判定owner、下一最小入口/独立oracle。903统计保历史；来源/工具/profile变只重核受影响，不能把903审计pass当当前全表执行。

### 5.5 统一原入口与报告

修852 Load-Binding十二列/UTF8/namespace唯一解析，沿862可靠Split语义或等价单源实现；去cases.json/附件前置，contract读tracked manifest/规则/规范。原source/snapshot/lifecycle/receipts/certify共享可信解析/核算，不各用一套ok/裸NA逻辑。必要只读helper在T-02决定，不另造runner。

contract只核来源/规则/覆盖/批准/资产，不执行guest或目标编译。acceptance准入当前或同身份有效的原父报告，明确选择范围；需运行仍调原固定组/auto-case。报告拟`ace.m1.acceptance/2`，T-02固定字段/上限；至少含binding/规则/批准/manifest/工具身份、registered/core/extension总量、required集合hash、实际选择/执行/条件排除/未执行、逐格profile/producer/证据/规则/双owner，以及base/core/extension/future/cert verdict和can_claim_m1。

base97/contract/局部selector PASS不等M1；少required/缺证据/零选/只NA无实际执行/SKIP/重复/额外或伪报告不能授信full。无批准来源/超出白名单规则非零拒绝；full所核集合须等于批准规则导出集合；gap/infra/首红不能被重放绿或wrapper exit0掩盖。按已批准D5，45个future不阻本次M1-required，仍显示gap/owner/重入条件，不改NA/PASS。

a2r emit与compile-run分列：复合认证缺不否定获准金样轴，但原明确strict receipt scope和G06/G07防伪保持；坏证书不能降成“未认证”绕过。日常不启动rustc/目标Rust、不收892/902/904替旧证书、不写cert/自动刷新golden；过期STALE/重验，certify显式。原Job/deadline/流/预算/父侧判定不变，准备/执行/读写/cleanup分账。

### 5.6 904交付后的写面与接口接受

| 面 | 本包范围 | 接受/回交 |
|---|---|---|
| P908 | 有限规则/报告只读逻辑、manifest/反例/规范候选、852脚本 | 不碰compiler/source/verify/MIR/native/Trans算法；本轮不生成生效规则或迁移token |
| M1登记/规范 | binding/applicability/source-manifest、SCN规范、conformance/903当前导航 | 由本包冻结，903不回开/不改历史收据统计，勘误另标原源 |
| 893已交付 | ace-test-policy.md/JSON、ace_cost_report、infra-ace-policy | 不再等D893、不动13义务/条件/observer/成本，不重复13慢测。所需接入元数据沿正式版本，实际治理符号复核当前owner |
| 904已交付S/P | core source adapter/verify/collections_facts、a2x core/rust/runtime、AXS3/AXR3 | 接受12f9adf18及canonical Spec；type/enum/ranged-body已闭，不重复修。默认只读，不改受检事实/资源/片段凭证或绕过正式不支持项；无D904等待 |
| 904已释放H | ac_a2r_hir.rs、test_cases/mod.rs、auto-test-case/contract.rs、test-groups.toml及新cert/recipe | 接受主线正式身份；保持旧schema不带execution_config、新collections cert严格携带且不跨profile授信。必要共用观察/注册修改仍具名取当前owner窗口，append-only不等无冲突 |
| 906在途H/构建 | scripts/test-groups.toml、auto-case.rs、lib.rs/features、Cargo依赖/轻Web合同 | 906当前WIP已核；908先做私有规则/来源/852核算。T-06若动固定组，核合并基面与两个计划实际段/符号，单写短窗、读回两方条目并回交；不改906合同或借其WIP作已交付输入，不等906整包 |
| 905研究 | 生成工程/Cargo治理研究结果与审查沉淀 | 当前execution_done5/7、无实现WIP，不占908 M1私有面；研究通过不等生产治理交付，不把研究结论当既有构建功能或908全局前置 |
| 有限诊断H | test_cases/ac_mn.rs、a2r.rs仅T-02证明需透传已有原因时具名 | 不新增语言检查/改raw code或guest语义，不能改core/904资源事实；超同义观察范围修订/后继 |
| 正式运行 | 原case/治理反例及必要scope | 904本包已无活动认证/采样树；与后续所有实际采样/重构建窗口错峰，不停他方进程/不持H等机器窗，infra组名不豁免实际ACE验证 |

窗记录owner/revision、文件符号、正式SHA、规则/recipe/schema/tool/profile、已跑/pending及回交；取得“文件不同段”不等已获并写许可。904已经review/merge，允许消费其主线正式接口；归档包不再索取窗/回开任务。908回交到现行Spec/路线与当前消费者，写明新规则/报告及并列producer限制，不把新通道自动替换旧M1 a2r。893成本数据及903审计若仍基于D904前输入，只标注/重核受影响producer、固定组或采样窗口，不重跑13慢例/改冻结审计数。

### 5.7 预算/验证与材料

源/规则/清单有界读取，沿原TestCase更严cap（源/元数据通常≤4MiB）；流≤1MiB/流、实际所选Job/deadline/资源/ignore/平台/样本/原oracle保留。规则/报告累计上限T-02从真实输入/既有caller冻结，未知/超额为resource/infra，不冒语言负例。无新三warm/P90矩阵，不缩样本/按60秒强杀/细拆VM。

开发相关反例/完整受影响组，final pass后merge冻结必要职责/消费者并集，同身份有效证据有据复用；cargo t all仅due/主检出单实例，不附每计划全局门。原持续会话/正式资源按原门，不以分类元数据冒已验证。

唯一拟树D:/autostack/.wt/lang-908/auto-lang、分支plan-908-dev，本轮未建。work/自检/独立上下文final review同树顺序，未指定人/模型，不创建agent/chat/发消息。开工登记D:/autostack/plan-artifacts/plan-908/ owner/用途/在途，迁移候选/日志/探针/收据/外置target只在scope；长期测试合同资产留其职责路径。merge折命令/退出/数字/节选/hash进计划，核全实际输出后删scope/树产物，具名未解决问题才hold。不写legacy specs.json或恢复旧raw永久保留。

### 5.8 实施前置、可并行项与收口顺序

| 前置 | 当前结论 | 实施入口/关闭条件 |
|---|---|---|
| 规则决策 | G-DEC-1..5本轮全部批准，既有NA/R/双轴继承 | T-01绑定本轮原话/§13/版本并校验61唯一映射；不得重复索取同一批准，也不得自动向外扩大 |
| 正式交付 | D903、D893、D904均已在主线；不再有D904/type片段等待 | T-01核当前主线/原source-oracle身份与发布接口；本次未改的既有冻结证据可按身份复用，904新producer不能冒旧a2r |
| 实施授权与唯一树 | 本轮仅规划，尚未开工 | 后续用户要求实施908时进入T-01，建唯一树/材料scope；不另要求已批准五项裁定。当前不勾任务或启动work |
| 有界方案与来源 | 脚本/helper/schema/累计上限及40场景tracked权威来源待实际核验 | T-02先最小规则→父报告→严格核算反控，T-03来源缺失/冲突显式列owner；这些是工作任务，不是再次请求政策裁定 |
| 当前H与运行窗 | 904旧窗已释放；906实际固定组/构建WIP需交接 | P908私有规则/来源/脚本核算可先独立推进；仅具体宿主观察、公共注册或重运行等实际窗口，不能以文件名重叠让整包互等 |

实施顺序：T-01冻结批准/基面/窗 → T-02最小严格链 → T-03 tracked source/oracle与T-04窄诊断实证（逻辑可独立，运行受机器窗约束）→ T-05按批准迁移候选与逐格diff → T-06原852入口/strict报告及公共注册短窗 → T-07职责/消费者与反控 → T-08自检 → T-09独立复审 → T-10冻结门 → T-11沉淀归档 → T-12全材料/树清理。不得先改token制造绿再补来源，也不把T-09/10冒工作阶段已完成。

**908之后的收口**：先按当前原输入/独立oracle读回required真实结果，区分证据未接、观测/infra缺口与语言能力缺口；接线能闭则由908在原范围闭，超范围问题具名能力owner+判定owner+最小关闭入口，不预占计划号。已知候选为旧a2r解析/未定型及诊断检查、AV资源故障/代际，实际缺口以验证为准；各自可分独立写面，涉及共享S/H再排窗。核心及扩展AV所需门全部闭后才组织M1签章；O非AV console45、native loader/闭包ABI及数值/多模块等按独立产品/M2路线推进，不因不在本次M1-required而删债。905研究后续生产治理、906构建边界按自身合同验收，不冒ACE语义前置。

| 交付后队列 | 能力owner / 判定owner | 收口与前置 |
|---|---|---|
| 先闭规则/来源/观察接线 | 908 / 908 | T-03..07仅现有能力/有限同义原因透传；不闭的诊断观察与infra具名，不能靠批准翻ok |
| 旧a2r解析/未定型/诊断与复合认证 | 旧A2R能力线（242仅长期跟踪，具体包待净缺口立项）/ M1后继验收owner | 保B16/E03/R07、六负例正确前端拒绝及独立认证轴；新892/902/904不顶替；只按真实剩余输入立最小包 |
| AV资源故障/句柄代际 | AV资源能力线（未立项）/ M1资源oracle后继owner | 先对原R02/R03等重核能力与证据；仅证据缺由908接线，真实机制不足独立包，失败原子/清理/危险读0不弱化 |
| M1签章 | ACE里程碑收口owner（待上述结果后立项）/ 独立验收owner | 核心适用+扩展AV28及所有适用结构/资源/增量门真通过；报告同时展示future45和cert轴，908治理交付不自动签章 |
| console45、native loader/闭包ABI、数值/多模块等 | 各产品/M2能力线（未立项，不预占号）/ 各原profile验收owner | 可按独立写面另行规划；正式承诺profile后更新范围/重入规则。console旧路径义务是原profile对等，不泛称旧引擎无print |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-m1-applicability.md | 裸NA/未定scope→用户批准的逐格规则/双层required/重入及版本 | 不按结果凑范围 | AC-01/02 |
| SD-02 | add | docs/specs/auto-test-case/ace-m1-scenario-oracles.md | 未跟踪draft→来源已核的40场景oracle/原起点/必要门 | fresh树独立核，不复活raw | AC-03 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 窄ok/片段scope→词表/诊断面/full-selected-base/扩展与源保护 | required与实际报告一致 | AC-02/04/05 |
| SD-04 | modify | docs/specs/auto-test-case/a2r-golden-certification.md | 独立认证→6诊断面/两轴及strict防伪不降格 | 不造金样/隐式认证/跨producer授信 | AC-04/05 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md ; docs/specs/auto-test-case/project.md | 903冻结数保留→当前批准回填/入口/822与NA3澄清、残余owner | 治理交付与M1通过分判 | AC-06/08 |

## 6. 测试设计

拟`scripts/tests/test_ace_m1_acceptance.py`或原等价治理控制由T-02/06/07使可运行；必要固定组拟infra-ace-m1（显式ACE治理、all=false，无默认worker准备），T-06注册。如PowerShell原实现适合就用对应真实控制，不为镜像实现造framework。以下均待执行。

| V | AC/invariant | 输入/实际入口 → 区分性关闭oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01授权 | 缺一批准/错版/扩大NA/I07重批/同H计两次 → 无生效规则/迁移或full PASS；批准后61唯一映射、2旧+59新增区分正确，合法只读输出可用 | T-01/02/05 work |
| V-02 | AC-02守恒 | 原131/524/12列、原103/扩28、两namespace R03；漏重/错列/新增未裁行控制 → 原id/expected/source不变，变化有来源，合法新行显式更新范围 | T-02/05 work |
| V-03 | AC-03来源 | fresh树仅tracked输入、附件不可见、源/oracle/manifest任一hash漂移/缺场景/伪D源码 → contract独立核来源，缺/漂移拒或pending，不复制raw | T-03/06 work |
| V-04 | AC-04负例 | 原B16–19/六a2r负例+2/2/0近邻、错族/统一unsupported/工具exit/B18 infra → 正确原因拒才过，合法近邻运行；不足留gap/ERROR，不改expected | T-02/04/07 work |
| V-05 | AC-05聚合 | 原scope/base97、只选1/只NA/零选/少required/重复/多余/伪父报告 → selected/base可单判，full仅全required有效原oracle齐；raw ok不替执行 | T-06/07 work |
| V-06 | AC-05双轴 | 金样绿认证缺、坏证书/旧新producer互换/偷偷certify或golden写 → 分轴且strict防伪拒，日常rustc/目标spawn0，旧证书/金样不变 | T-04/06/07 work |
| V-07 | AC-05/06scope/原子 | 双层白名单+按绿动态排O或旧core、半规则写入/发布失败、后续contract再读 → 无批准缩范围/半套拒，旧态保全、修复后真可读，future45仍gap非PASS | T-05/06/07 work |
| V-08 | AC-06兼容 | 89313策略/904新recipe旧例、12列/源/expected/golden/cert前后hash及来源漂移 → 原能力/条件/预算不改，仅受影响STALE；实际H收据/消费者/未执行不冒绿 | T-01/07 work |

组选按真实触面：规则/脚本选拟infra-ace-m1+infra-fixed；触893策略选infra-ace-policy；TestCase准入选auto-test-case/ace-adapters，真实宿主观察补ace-host及受影响case-ace-m1-*、a2r原消费者。新A2R默认只读，未触不重复904性能；共用contract若变则补ace-a2x/已交付case-a2r-hir-record-text-902及case-a2r-hir-collections-904兼容（日常20例spawn-zero），a2x-coll-formal仅真实增量/资源/认证语义受影响时显式选取。既有组已核当前TOML，未知范围具名扩大，不自动展开全仓。

反控只在独占树/scope副本实际变异、证明生效与门红，再恢复合法控制；不动主检出/他方源。合成全覆盖只验证schema/核算，不伪称现实M1执行。用户未指定人/模型，独立final review不等自查，同树顺序，无agent/chat。

## 7. 验收标准

| AC | owner_stage | 可执行关闭条件 |
|---|---|---|
| AC-01 | work | V-01：五项已批准选择/范围/来源与决策件§13一致，旧批准直接继承、不重复投票；NA/future白名单不可扩，批准不冒实际执行PASS |
| AC-02 | work | V-02：131/524与12列守恒、核心/扩展/起点/双轴明确，原id/source/outcome/expected不改，token变化有前态/批准/证据 |
| AC-03 | work | V-03：tracked必要源/oracle/40场景/manifest来源齐，fresh无附件可核，缺/冲突给owner不造输入 |
| AC-04 | work | V-04：语义诊断族与观测严格、近邻/错族/infra控制过；B18按真实观察、6负例不造金样/假认证，缺能力诚实gap |
| AC-05 | work | V-05/06/07：原入口strict规则/证据/full scope与双轴一致，零/缺/SKIP/坏证书/隐式认证/半迁移不冒M1，future账可见 |
| AC-06 | work | V-07/08：893正式约定和904符号接受/回交、原输入/条件/预算保护，SD/残余双owner/oracle齐，治理完成与M1剩余门分开 |
| AC-07 | review | 同树独立逐AC-01..06/V-01..08/SD核实际批准/源/原oracle/负控/遗漏/延期/健康，revision/SHA绑定pass；M1红不否定诚实治理，也不冒M1绿 |
| AC-08 | merge | final pass后冻结职责/消费者门、真实合入/SD读回/归档/guard与全材料清理，真实12/12，未关能力具名，不回开旧Plan/写旧ledger |

## 8. 执行步骤

12项=8 work+1 review+3 merge。work 8/8 已于 2026-11-01 完成（worktree `D:/autostack/.wt/lang-908/auto-lang` @ `plan-908-dev`，交付提交 `a93eb8463`，基面 master `ed4286792`）；T-09..T-12 待 review/merge。五项裁定已闭合、904前置已交付。

| T | owner_stage | 前置/窗 | 工作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；五项裁定已批准 | 冻结主线/唯一树/903893904正式收据/131输入与规则hash、原NA3/822澄清、904接口接受/906等当前符号窗/批准逐格scope/工具/组选 | AC-01/02/06 | [x] 基面 ed4286792；903/893/904 交付 18bac254b/b71b7343e/12f9adf18 均祖先；§4.2 七项 hash 逐位一致，.next-id 909；54 裸 n/a 实数=C4+H6+G01 1+G02-08 21+I01-11 22，61 映射逐格核对；906 窗=test-groups.toml HTTP 段(L127)与本包追加段不重叠；组选 infra-ace-m1/infra-fixed/infra-ace-policy/case-ace-m1-nb-diags-908/case-ace-m1-base |
| T-02 | work | T-01；实际P/H | 最小批准集→12列/白名单→原报告准入→selected/full与错族/缺证据控制，冻结原脚本/只读helper机制/schema/上限/近邻 | AC-01/02/04/05 | [x] schema 冻结：`ace.m1.applicability/1`/`ace.m1.source-manifest/1`/报告 `ace.m1.acceptance/2`；上限：报告 4MiB、源/元数据 ≤4MiB；helper=852 脚本内嵌只读函数（Load-Applicability/Load-SourceManifest/Get-CellState），无第二执行体系；近邻 NB17(`1+1`→2)/NB18(`var`→2)/NB19(补`return 0`→0) 新ID冻结 |
| T-03 | work | T-02 | 852必要source/oracle/40场景正文/manifest来源核验及长期候选，fresh无附件V-03，冲突具名不猜 | AC-03 | [x] （R1 修复 dc8eca3ae：.gitignore 加 `!` 豁免 + git add -f，`git ls-tree dc8eca3ae` 含两 sidecar=2，治理测试新增 tracked/not-ignored 断言 23/23）`source-manifest.json`：81 src（58原+881×12+864×3+889×4+866×1+908近邻3）/24 金样+wrapper/10 证书/43 reference/66 suite·case·子树源 + `ace-m1-scenario-oracles.md`（40 场景 oracle，I11 门按858 P90≤2000ms标明取代）+ 决策件 SHA 全绑；禁造 src/D*.at；治理测试 pristine 副本仅含 tracked 文件即 fresh 树等价 |
| T-04 | work | T-02；窄运行窗 | B16–19/6a2r/B18 legacy实际复验或同身份证据、近邻2/2/0与错族/infra反控、原双轴和不足owner，不补语言能力 | AC-04/05 | [x] 近邻 suite 9/9 PASS（三引擎 2/2/0，exit 0）；错族反控 NB17 期望→3 得 FAIL=3/exit 1；base97 97/97（`-Scope source` exit 0，B16–19 ac/av 腿当前身份）；B18 avm 重探针=infra（错误完成无 typed stage/code，消息 'Cannot reassign to immutable variable x' 可辨识）→保 diff:pending；B17 a2r 探针=emit 成功 1 产物→保 gap:a2r-golden（G-DEC-3 面真缺口）；receipts 原生 -BuildIdentity PASS 10/10 |
| T-05 | work | T-02..04；M1数据窗 | 实际批准sidecar/迁移diff回填token及前态，保原131/id/source/expected、残余gap和core-extension-future账，V-01/02/07 | AC-01/02/05 | [x] 61 格 →`na:m1(NA-3/5/6)`（note 保 `908T05 <path>: old->new (G-DEC-n §13)`）+ T08 a2r `source-bound`→`gap:a2r-golden` + B18 avm `908T04` 注记；生成器守恒断言：131×12、id/ns/feature/outcome/expected/status 不变、裸 n/a=0、规则 old_token 逐格匹配；binding d9f05da0→caf5f79f；R-ACCEPTED 32 格（avm diff* 除B18 30 + K02/K05 a2r 2）；migration-diff.tsv 在 scope |
| T-06 | work | T-03..05；脚本/固定组H | 修852解析/去附件、统一scope/strict报告及原父证据/独立认证/原子再读，必要治理组/控制注册，V-03/05/06/07 | AC-03/05 | [x] `.Split([char]9)`（862 同源）；contract=131/91/40+R03双ns+applicability 严格（来源 SHA/双claim/未知行/binding_token 失配/分层总数重算 12 项）+manifest 严格+NA 反滥用+基线+owner；acceptance=零执行核算 `ace.m1.acceptance/2`（-ParentReport 拒空/失败/伪，-Select 仅报告，-BuildIdentity 零 spawn 验证证书）；contract exit 0、acceptance exit 0（CORE red 247/285、EXT 28/28、FUTURE 45/45、CERT strictly-validated(10)、can_claim_m1=false、required hash 35ad43aa…）；注册 infra-ace-m1（all=false）+ case-ace-m1-nb-diags-908 |
| T-07 | work | T-01..06；稳定验证窗 | V-01..08反控/合法控制及完整受影响职责/必要消费者，904旧兼容/zero target spawn/漂移再读，缺口诚实 | AC-01..06 | [x] `cargo t infra-ace-m1 infra-fixed infra-ace-policy case-ace-m1-nb-diags-908 case-ace-m1-base` 5/5 PASS exit 0；治理测试 22/22（7 静态 + 1 合法控制 + 14 变异必红：V-01 缺来源/扩白名单/双claim/未知行、V-02 漏行/截列/半迁移、V-03 hash 漂移/伪 D 源/缺 oracle 文档、V-05 伪/失败父报告/未知选择/子集绿不授 full、V-07 规则-binding 失配）；V-06 receipts 10/10 PASS 且 bash 换行篡改身份被 STALE 拒；V-08 893 策略组 PASS、未触 crates/904 符号、金样/证书 hash 不变 |
| T-08 | work | T-01..07 | 逐格批准/AC/V/SD/原hash/残余owner/健康自检，摘要/命令/退出/hash折计划，提交/树clean→execution_done，不冒M1绿 | AC-01..06 | [x] （R1 修复：提交 `dc8eca3ae` 补两 sidecar/gitignore 豁免/tracked 断言/gap-audit 242 链接/KNOWN-DEBT D-862-R1-01 闭合；contract exit0、acceptance exit0(cert 10)、infra-ace-m1 PASS、树 clean）无调试输出、py_compile OK；首次提交 `a93eb8463`（16 文件：binding/applicability/manifest/3 近邻源/2 近邻 case·suite/852 脚本/toml/治理测试/4 规范修订+2 新规范/决策件§13）；既有编译警告 aavm2_lib_source 未触面；execution_done，M1 仍红（诚实） |
| T-09 | review | T-08冻结SHA/同树 | 独立核批准映射/源/实际oracle/反控/残余scope/SD/遗漏延期，finding同树修复重绑，pass仅reviewed | AC-07 | [x] R1 needs_fix（3 findings）→ 修复 dc8eca3ae → R2 final pass（fresh git-archive 树 contract/acceptance 绿） |
| T-10 | merge | T-09 pass | 冻结完整受影响职责/消费者门或有据复用，同身份漂移重核受影响；批回归仅due主检出单实例 | AC-08 | [x] 交付提交 dc8eca3ae 上一次跑并集 `cargo t infra-ace-m1 infra-fixed infra-ace-policy case-ace-m1-nb-diags-908 case-ace-m1-base` 5/5 PASS（run-yw3mm3ti）；批回归：收据 last_covered 895 waived_by_user（2026-10-30 常设豁免），48h 规则下 due → 记 user-pending（沿 904/905 惯例，不在计划树内启动） |
| T-11 | merge | T-10 | 合入、SD-01..05/批准来源读回、旧债具名关闭/保留及路线/能力后继，archive delivered，不签虚假M1 | AC-08 | [x] wt-guard clean → 主检出恢复同内容决策件（874ccc11 两侧相同）→ `git merge --ff-only plan-908-dev`：master=dc8eca3ae、0 merge commit、线性；SD-01..05 落地读回 blob 一致（applicability 0cba0d20/scenario-oracles c0539b96/conformance d27e80e3/golden bd784343/gap-audit 63343b86·blob ed8ed9ab/project fd71c4ef）；sidecar tracked on main；main smoke：contract exit0、acceptance exit0（can_claim_m1=false）、治理 23 OK；D-862-R1-01 闭合在档；archive delivered，M1 仍红不签 |
| T-12 | merge | T-11 | 摘要折计划、全输出/hold对账、fresh wt-guard clean后清树/分支/组/plan-artifacts并登记，真实12/12 | AC-08 | [ ] |

## 9. 复审记录

### r1立项历史（已由下方r2批准记录解除裁定等待）

2026-11-01 stage:new | PLAN-908 r1 | outcome:blocked（规划合同已写；涉及规则实施待G-DEC-1..5明确裁定，非实施失败） | 静态核90310/10/18bac254b祖先、89312/12/b71b7343e祖先、904executing5/14/H交集；131/524/54裸格、推荐61唯一映射、原NA3 I07/I08和822归档澄清、原expected语义族与tracked资料缺口。五项推荐未批准，未改binding/规则/current Spec/expected/golden/cert/生产，未编译/测试/探针/认证/建树/agent/chat/发消息。

908经scripts/new-plan.sh排他锁唯一取号，取号时下一号909；drafting r1、0/12。next: 用户明确五项选择，按批准范围更新合同/来源，再后续授权work T-01。本轮仅规划，不自动实施/跑批回归。

规划静态预检：章节0..10齐、T12/AC8/V8/SD5唯一且无未定义引用，Plan本地链接/空白检查通过；903/893交付祖先已核，原binding d9f05da0…保持；推荐NA5/6/H/I组合61格无重叠，54裸格去向明确，2旧批准澄清与59新增待准分开。908唯一/取号时下一号909、908实施树不存在，current Spec/原source/oracle/expected/golden/证书未修改。该检查不等规则获准、功能/验收执行PASS或M1完成。

### r2批准与交付后规划修订（2026-11-01，当前入口）

stage:new | PLAN-908 r2 | outcome:pass（规划合同修订；非实施/review/验收PASS） | G-DEC-1..5用户明确按r1推荐全部裁定，旧NA/R/双轴与I07/I08直接继承；§10已记录范围/影响。决策件增§13，批准来源原话见§4.1，修订后SHA256 `874ccc1198bf23c284fa588a6dfcdee80eecc8f5a0e392b88b34ddd74db0875b`。131/524全登记不删，61唯一适用映射=2旧批准+59本次批准，O非AV45列future仍保gap，不冒执行PASS。

基面174652b5c；D904 delivery12f9adf18已在main、archived14/14并完成Spec/清理；D893、D903已交付。接受904 type/enum/ranged-body与资源事实、新recipe/fileset/cert/execution_config，不再等S修复或904旧H窗。当前lang-905无WIP，lang-906有固定组/宿主构建等WIP，§5.6按符号取窗而非整包等待。892旧suite缺独立固定组的F-INFRA-5仍是独立后继事项，不被904新collections组冒关闭，不阻本包旧M1身份核算。

next: 后续用户要求实施908时，从T-01依§5.8/§8开工；不重复五项投票。本轮drafting r2、0/12，未创建实施树/材料scope，未动生产/当前Spec/binding/源/expected/golden/证书，未运行编译/测试/探针/认证/批回归，未取新号或联系其他会话。仅做文档结构/引用、白名单与输入hash的静态核验；实际T/V/AC仍待实施与独立复审。

r2静态预检：章节0..10、T12/AC8/V8/SD5唯一且Plan本地引用可达；按实际namespace逐格映射，61无重叠且全部存在=54裸n/a+7 raw gap，future corpus O45与之不交叠。初次静态核对发现附表H误标scenario，已依原binding纠为corpus后重核通过，未改批准行/路径范围。原binding SHA d9f05da0…、903/conformance/893 policy与852脚本hash保持，D903/D893/D904均为main祖先；本轮六份文档空白及本次新增/更新引用检查通过，.next-id仍909，908树不存在。此记录只证明规划一致性，不代替执行/独立review/M1 PASS。

### r2 实施（2026-11-01，/auto-plan-goal 链 work 阶段）

stage: work | plan_id: PLAN-908 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-908/auto-lang | branch: plan-908-dev | base_commit: ed4286792（master tip at 建树；r2 规划基面 174652b5c 为其祖先） | code_commit: a93eb8463 | dependency_revisions: 组内 detached 兄弟 auto-web ec9c4ba / auto-ui 7dfab3c / auto-down 7c4d5cb / auto-man 1eeac78（workspace `../../../auto-web|auto-ui` 三级引用解析，无 junction/symlink） | task_ids: T-01..T-08 (8/8) | evidence: §8 各行；scope `D:/autostack/plan-artifacts/plan-908/`（gen_sidecars.py、migration-diff.tsv、acceptance-final/ace-m1-acceptance.json sha256 c7830376… 152345B、nb-run2/、source-run/、probe-out/report.json、wrongfam-out/、receipts.log） | blockers: none | next_responsible_role: review（T-09，同树独立上下文，冻结 SHA a93eb8463）

**就绪记录（work-owned AC）**：AC-01 V-01 批准映射 61 唯一=2 旧澄清+59 新增，applicability 绑决策件 SHA 874ccc11…，缺来源/扩白名单/双claim/未知行四变异必红、合法控制绿；AC-02 V-02 131/524/12 列守恒、R03 双 ns、identity 列不变、61+1+1 变更均有前态/来源/日期 note，漏行/截列/半迁移必红；AC-03 V-03 manifest 81/24/10/43/66 + oracle 规范 SHA 绑定，fresh 树（pristine 副本）contract 成立，hash 漂移/伪 D 源/缺规范必红；AC-04 V-04 近邻 9/9 绿（判别性）、错族必红、B16–19 ac/av 腿 base97 绿、B18 avm 重探针 infra 保 pending、B17 a2r emit 保 gap:a2r-golden、不改 expected/不补语言能力；AC-05 V-05/06/07 acceptance/2 分轴（base 不冒/core 247/285 红/ext 28/28/future 45/cert strictly-validated(10)/can_claim_m1=false），伪/失败父报告与子集绿不授 full，receipts 严格防伪（篡改身份 STALE 拒），规则-binding 失配/分母缩放必红，日常零 spawn；AC-06 V-07/08 893 策略组 PASS、904 符号零触、金样/证书 hash 不变、R-A..F/NA-1..4 继承为规则、FUTURE-O 45 保 gap 可见、残余 38 核心 blocker 逐族 owner（gap-audit §11）。

**开放项**：无 in-contract 待修；可延后至 review/merge 的 gate：T-09 独立复审（AC-07）、T-10 冻结职责/消费者并集（本轮已跑 5 固定组可按身份复用）、T-11 SD 读回/归档、T-12 清理；`cargo t all` 批回归仅 due/主检出单实例（merge owned，不作本交接前置）。**不冒 M1 绿**：core 38 真实 blocker 保留（B16/E03 a2r 名解析 2、B17–19 ac 3、B18 avm 1、a2r 诊断面 7、scenario 观察通道 14、AV 故障/代际/共享可变 3、mn 复合场景面 7、R07 a2r 1），各带 owner 于 gap-audit §11。

**健康/偏差记录**：既有编译警告 `aavm2_lib_source` 未触面（非本包）；受检出 CRLF 提示两文件（git 自动规范化）；T08 a2r `source-bound`→`gap:a2r-golden` 与 B18 注记为批准外的证据型归一（903 F-REG ④/⑤，注记已载事实），不改语义；R-ACCEPTED 规则为 2026-10-28 R-A..D 组裁定的登记化，32 格 token 保 raw 证据；决策件 §13（r2 规划产物）随本提交入库。

stage: goal | plan_id: PLAN-908 | plan_revision: r2 | round: 1 | from: drafting | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-908/auto-lang | commit: a93eb8463 | next: review

### r2 独立终审 R1（2026-11-01，needs_fix）

stage: review | review_scope: final | reviewer_identity/limitations: 同会话复审（计划§5.7 不创建 agent；判决从工件重建：独立重算脚本 + 冻结 HEAD 门复跑，不采信执行摘要） | plan_id: PLAN-908 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-908/auto-lang | branch: plan-908-dev | reviewed_commit: a93eb8463 | base_commit: ed4286792 | dependency_revisions: 兄弟 detached auto-web ec9c4ba / auto-ui 7dfab3c / auto-down 7c4d5cb / auto-man 1eeac78 | spec_inputs: binding caf5f79f / applicability 74c9109b / manifest e20494ea / 决策件 874ccc11 / conformance·gap-audit·golden·project(worktree HEAD) | acceptance_results: AC-01..06 复核 pass（独立重算 313=285+28、core_na 95、ruled 32、future 45、required hash 35ad43aa 逐位相符；62 迁移=61 G-DEC+T08、旧token分布精确对应、identity列/note追加性/ok-emit不变；contract=0/acceptance=0(cert strictly-validated 10)/治理测试 OK；toml 与906 hunk不相交；SD-01..05在且一致）；AC-07 本审 | findings: F-908-R1-01(P1) 两个sidecar被.gitignore:162 `*.json`吞掉未入a93eb8463——fresh检出contract必红，违AC-03/T-03/T-08（F-793-R1-01同类，需`!`豁免+git add -f+治理测试加tracked断言）；F-908-R1-02(P3) gap-audit §5 F-A2R-1行242 tracker相对链接预存断链（903带入，随修复顺手改）；F-908-R1-03(P2) KNOWN-DEBT D-862-R1-01已被本包关闭但未记闭合证据 | evidence: 复核执行=`contract exit0`、`acceptance -BuildIdentity exit0`、`python test_ace_m1_acceptance.py OK`、`git diff ed4286792..a93eb8463 --stat`（16文件、无crates生产码）、`git check-ignore -v` 证实忽略、独立对账脚本输出（scope review-acc/ 报告）；可复用：5固定组PASS（同HEAD同净树，T-07收据） | pending_closeout_gates: T-10冻结职责/消费者并集、批回归due（merge-owned） | next: work（修复三findings后回execution_done重审受影响项）

**任务回开**：T-03（tracked 化未达成——sidecar 未入库）、T-08（提交完整性+债务闭合簿记）→ `[ ]`；T-01/02/04/05/06/07 证据不受影响保留。current_step: 6/12。

stage: goal | plan_id: PLAN-908 | plan_revision: r2 | round: 2 | from: execution_done | to: executing | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-908/auto-lang | commit: a93eb8463 | next: work

### r2 修复 R1（2026-11-01，work needs_fix 路径）

stage: work | plan_id: PLAN-908 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-908/auto-lang | branch: plan-908-dev | base_commit: ed4286792 | code_commit: dc8eca3ae（修复，父 a93eb8463） | dependency_revisions: 不变（auto-web ec9c4ba / auto-ui 7dfab3c / auto-down 7c4d5cb / auto-man 1eeac78） | task_ids: T-03、T-08 重闭（8/8） | evidence: 见下 | blockers: none | next_responsible_role: review（受影响项重审：AC-03 fresh 树/tracked、AC-06 簿记；其余 AC 证据同 HEAD 内容未变可复用）

| finding | invariant | 修复与永久控制 | 当前结果 | 残余 |
|---|---|---|---|---|
| F-908-R1-01 | 长期测试合同数据必须 tracked，fresh 检出凭 tracked 输入 contract 成立 | `.gitignore` 沿 F-793-R1-01 模式加 `!…/applicability.json`、`!…/source-manifest.json`（含 PLAN-908 注释）+ `git add -f`；治理测试新增 `test_sidecars_are_git_tracked_and_not_ignored`（git ls-files --error-unmatch + check-ignore 反向） | `git check-ignore` exit 1（不再忽略）；`git ls-tree -r dc8eca3ae` 含两文件；contract exit 0；acceptance exit 0（cert strictly-validated 10，can_claim_m1=false 不变）；python 23/23 OK；`cargo t infra-ace-m1` PASS | 无 |
| F-908-R1-02 | 规范内相对链接可达 | gap-audit §5 F-A2R-1 `../../../plans/242…` → `../../plans/242…`（文件在 docs/plans/） | 链接扫描 0 断链 | 无（预存缺陷顺手修） |
| F-908-R1-03 | 被本包关闭的既有债须记闭合证据 | KNOWN-DEBT `D-862-R1-01` 追加"已关闭（PLAN-908 T-06）"及证据 | 已写入（worktree 提交） | 无 |

内容 hash 不变：binding caf5f79f / applicability 74c9109b / manifest e20494ea（仅 tracked 状态变化），故 AC-01/02/04/05 证据按身份复用；未动生产码/金样/证书。

stage: goal | plan_id: PLAN-908 | plan_revision: r2 | round: 3 | from: executing | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-908/auto-lang | commit: dc8eca3ae | next: review

### r2 独立终审 R2（2026-11-01，final pass）

stage: review | review_scope: final | reviewer_identity/limitations: 同会话复审（计划§5.7 不建 agent；判决从工件重建——git archive 纯 tracked 树重放 + 独立链接/簿记扫描） | plan_id: PLAN-908 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-908/auto-lang | branch: plan-908-dev | reviewed_commit: dc8eca3ae | base_commit: ed4286792 | dependency_revisions: auto-web ec9c4ba / auto-ui 7dfab3c / auto-down 7c4d5cb / auto-man 1eeac78 | spec_inputs: binding caf5f79f / applicability 74c9109b / manifest e20494ea / 决策件 874ccc11（内容同 R1，仅 gap-audit 一处链接与 KNOWN-DEBT 一处闭合注变化） | acceptance_results: AC-01/02/04/05 R1 pass 证据按身份复用（修复 diff 仅 .gitignore/两 sidecar 入库/测试 py/两处文档，无生产码/金样/证书/binding 变化）；AC-03 **pass**（`git archive dc8eca3ae` 导出纯 tracked 树 → contract exit 0 + acceptance exit 0，两 sidecar 在 `git ls-tree`，`git check-ignore` exit 1）；AC-06 **pass**（D-862-R1-01 闭合注在档；toml 与 906 不相交不变）；AC-07 本审 pass | pending_closeout_gates: merge-owned——T-10 冻结受影响职责/消费者并集（5 组 PASS @a93eb8463 + infra-ace-m1 PASS @dc8eca3ae，其余 4 组输入未变可按身份复用，由 merge 决定）、`cargo t all` 批回归 due（主检出单实例/用户择时，沿 2026-10-30 standing waiver 惯例）、SD-01..05 落地读回、归档、guard 清理 | findings: F-908-R1-01/02/03 **closed**（修复周期 1/3）；观察（越界预存，非 finding）：`batch-execution.md`（PLAN-888）链接指向已退役 attachments/888，归 888/治理线 | evidence: fresh/ 导出树 contract/acceptance 输出（scope fresh-acc/）、`python test_ace_m1_acceptance.py` 23/23 OK、链接扫描、`git diff --stat a93eb8463..dc8eca3ae`（6 文件 +2475/−2） | next: merge（T-10..T-12）

stage: goal | plan_id: PLAN-908 | plan_revision: r2 | round: 4 | from: execution_done | to: reviewed | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-908/auto-lang | commit: dc8eca3ae | next: merge

### r2 沉淀 merge（2026-11-01）—— PLAN-908:r2

stage: merge | plan_id: PLAN-908 | plan_revision: r2 | completion_kind: delivered | checkpoints:
- `prepared`：reviewed r2 final pass @ dc8eca3ae（父 a93eb8463，基 ed4286792=master tip，无需 rebase）；SD-01..05 已在树内提交并经终审；worktree 净、wt-guard clean。
- `gate`（T-10）：并集 5 组 PASS @ dc8eca3ae（run-yw3mm3ti）；批回归 due 记 user-pending（standing waiver 2026-10-30）。
- `landed`：主检出 `git checkout -- docs/plans/reports/ace-m1-scope-decision-2026-10-28.md`（未提交副本 874ccc11 与来件逐字节相同，无损）→ `git merge --ff-only plan-908-dev` → master = **dc8eca3ae**，`git log --merges dc8eca3ae..master`=0，线性；.next-id 工作区 910 为他会在途取号，未动。
- `canonical_verified`：六规范 blob sha256 见 T-11；两 sidecar `git ls-files` on main=2；binding caf5f79f/applicability 74c9109b/manifest e20494ea；main smoke contract/acceptance exit 0、治理 23 OK。无 index/ledger 写（PLAN-106）。
- `archived`：本文件 → `docs/plans/archive/908-ace-m1-rules-registry-and-acceptance.md`，status archived，随本节一起提交到 master（仅本计划文件；主检出其他会话 docs WIP 原样保留不入本提交）。
- `cleaned`：待 T-12（下节记录）。
- 落地≠部署：本包仅测试治理/脚本/规范/测试数据，无生产二进制或 web bundle 消费；worktree 内 dev `auto-case.exe` 随树删除——显式观察项，无需重建/重启。


## 10. 待澄清事项

| D | owner/入口 | 未闭合边界 |
|---|---|---|
| G-DEC-1 | 已裁定2026-11-01：按推荐批准NA5/NA6及G01主体 | D1共50格，H6归D4；精确白名单/重入见§5.1，实施绑定来源，不扩NA |
| G-DEC-2 | 已裁定2026-11-01：语义族映射且保raw原因 | T-02/04闭实际观察/2/2/0近邻；泛unsupported/任意非零/无原因不充通过，不新增语言检查 |
| G-DEC-3 | 已裁定2026-11-01：6负例正确前端拒绝面 | 继承负例不生成可执行金样；仍required，不足留诊断gap，B16/E03/R07与认证欠账不删 |
| G-DEC-4 | 已裁定2026-11-01：H非AV9按推荐；I07/I08旧批准直接继承 | 本项2旧+9新；不重索批准、不向外类推，H AV3仍required、原观察/重入保留 |
| G-DEC-5 | 已裁定2026-11-01：原103核心+扩28已承诺AV双层签章 | O非AV45列future仍gap/owner，不改NA/PASS或隐出全登记；实际未过门不签M1 |
| D-06 | T-02核原报告/脚本/同义诊断H桥与上限 | 超规则/观测收口修订或另案，不顺手修生产/造runner |
| D-07 | T-01/06接受904正式接口；重核906等当前H/构建/运行窗 | 904已释放无整项等待；不消费他方WIP、不持窗等别窗；只受影响兼容/来源重核 |
