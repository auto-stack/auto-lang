---
plan_id: PLAN-995
status: archived
feature_name: VM json native性能与codegen hazard收口（READER-009 cold-load框架前置）
author: [ZCode]
created_at: 2026-11-07
updated_at: 2026-11-08
plan_revision: 1
current_step: 7
total_steps: 7
depends_on: [READER-009]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-vm/vm-codegen-hazard-catalog.md, docs/specs/auto-vm/json-natives-performance.md]
touched_goals: []
affects: [auto-vm, stdlib]
---

# PLAN-995：VM json native性能与codegen hazard收口

## 0. 变更摘要

READER-009 W-04 cold-load预算红在009 store域优化后仍余1.5s缺口（静负载实测16.54s 对 15s冻结预算，分量对账闭合），009已至域内架构下限；用户2026-11-07 round27裁定①A授权本框架perf计划：**VM json.parse/stringify性能与canonical_ok单parse化**，前置约束=先消round23实证的VM codegen hazard目录（不得在已证hazard区直接单parse化）。本计划交付：hazard逐类消除/具名封界、VM json native语义保持的性能改进、canonical_ok单parse化参考实现与等价证明、Reader pin链端到端静负载复测；交付pin后由009消费重测W-04 cold叶（预算15s不修订）。

## 1. 目标

在不改变VM可观测语义的前提下：①round23 hazard目录四类逐项消除（hazard不再触发）或具名封界（合同禁用+检测，修复类另track）；②VM json.parse/json.stringify在冻结大文本基线上有实测性能改善，cold关键路径4次大parse（信封解码+对象解析×2文档）分量可测下降；③canonical_ok单parse化（单次parse替代"流式门+逐段json.parse重建"的多趟全量parse）在冻结千册catalog文本上逐字节等价并实测改善；④端到端Reader pin链静负载cold-load两run band复测与分量对账，达标与否均具名结论。

非目标：不修订15s冻结预算（处置权归009/用户）；不改READER-009仓内任何源文件（canonical_ok的catalog.at落地由009消费时自施，本计划交付参考实现+证明，见§4口径注记）；不承担009其他叶子；不优化无关native/无关热路径；若hazard调查证实某类属auto-ui codegen域，仅做有界定位与移交决策，不在本计划内跨仓修复。

## 2. 架构方案

主责auto-lang。唯一实施树 D:/autostack/.wt/lang-995/auto-lang，分支 codex/plan-995-dev；实施/内检/独立终审顺序共用。共享Plan留本主检出。probe/bench/参考实现材料归 D:/autostack/plan-artifacts/plan-995/auto-lang/（登记in-flight）；round23的17 probe原件归009 t03 scope，只引用不搬移，本scope重建等效probe。

端到端复测（T-04）消费冻结pin：Reader兄弟检出@17e6b62（codex/reader-009-dev，round24 16.54s基线状态，只读不写、自有端口/隔离数据目录）、工具cohort 6ce87feb（plan-009/t02/ui-068-handoff交接件），换入本计划新VM pin后按009同法静负载窗口实测。009活动worktree零接触。

**canonical_ok落地上限注记**（立项时口径，用户可在审阅时更正）：用户round27①A将"canonical_ok单parse化"列入本计划。但catalog.at/jsonx.at是009活动写入面（其T-03修复轮持续提交），跨树并行写入违反一写入者规则。本计划交付：单parse化参考实现（.at，位于本计划scope）+冻结文本等价证明+合规证明；009消费新pin时在自有树re-land并重测。若用户希望本计划直接落地Reader仓，须明示修revision（涉及009写入面移交）。

hazard→改面假设（T-00复核后冻结实际owner与改面）：

```
.at程序 → VM执行（auto-lang-core/src/vm）
H1 动态集合二遍迭代空段        → VM集合/迭代器状态语义
H2 if条件内嵌方法调用==大文本误判 → VM比较/调用求值路径（216KB量级，先绑局部再比较正确）
H3 大串累积跨模块形状敏感      → VM串池/累积路径（同函数文本不同module布局真假不一）
H4 模块var大串赋值/存串比较    → round23实证安全（基线，不改）
json.parse  → strings_json.rs shim_json_parse_vm（serde解析→json_to_vm_value物化）
json.stringify → native.rs绑定 shim_web_json_stringify 等 stringify路径
canonical_ok（Reader侧现状）→ valid_catalog流式门 + books/groups/prefs_text逐段json.parse重建
```

## 3. 技术栈

auto-lang Rust VM（crates/auto-lang-core/src/vm及natives/ffi stdlib）；.at probe与参考实现经auto.exe实跑验证；基准夹具=009冻结1000件catalog/工作区文本（workspace_fixture.generate可再生成）。验证沿auto-lang既有cargo测试组惯例：`cargo t <固定组>`完成受影响组与必要集成组；组名单T-00经`cargo t --list`实核登记，本计划提出的新probe/等价对照均为未执行提案，不预报通过。

## 4. 需求分析与背景调查

**授权记录**：用户2026-11-07 READER-009 round27裁定①A——授权新框架perf计划（VM json.parse/stringify与canonical_ok单parse化，auto-ui/auto-lang域，同068先例）；009消费新pin后重测W-04 cold叶；前置约束：round23实证的VM codegen hazard目录为该计划前置，不得在已证hazard区直接单parse化（round23/24的8-Map重写与单parse+memo测量否决为先例证据）；冻结预算15s不修订。两plan先后与依赖由各自立项时定（round27原文）；用户建议缺陷计划（auto-ui/PLAN-069）优先。无指定预算/模型/自动续跑限制，不虚构。

**perf基线证据**（009 §9 round23/24回执，实测，工件归009 t03 scope）：

| 项 | 实测值 | 备注 |
|---|---|---|
| cold-load（静负载，CPU 3-17%窗口） | 16.54s / 预算15s，缺1.5s | ws-ui-vm-13；分量：boot~1.5s+books GET 3.9s+catalog GET 2.6s+front(prefs 0.06s+build 6.2s+渲染/采样~1.3s)≈16.5对账闭合 |
| build（front缓存构建） | 6.2s，含4次必要大parse | 动态值不可跨函数边界（T-02冻结实测）迫使文本中转 |
| prefs POST（park后台写） | 7-11s | canonical_ok重建比较×2+catalog全量重序列化；warm叶不阻塞但为同族热点 |
| canonical_ok proof | 2.87s/千条 | fresh-get对照，同机同binary当日 |
| 环境浮动 | ±2× | books GET同代码跨run 5.0→10.1s；测量须静窗口两run取band |
| 已测量否决（先例） | 8-Map直连重写+0.5s否决；单parse+memo因H1-H3回退；prefs splice直通无效；piecewise逐字校验读路径恶化否决 | 不重复已否决路线，除非hazard消除后前提改变 |

**Hazard目录**（round23，17 probe实证；本计划T-00在当前auto-lang master复核）：H1动态集合不可二次迭代（probe-01/05二遍空段）；H2 if条件内嵌方法调用`==`在216KB量级误判false（probe-08 fulldiff/trace：先绑定局部再比较正确、return表达式位置不触发）；H3大串累积函数对同一形状跨模块上下文不确定（probe-03/05真 vs probe-13/16/17假）；H4模块var大串赋值/存串比较安全（probe-07/11/12基线）。

**本轮起草代码锚点**（auto-lang master 4bc01dab1观测，实施时重冻结）：crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs——shim_json_parse_vm(:363，serde解析错误映射)、shim_json_to_value(:373，serde_json::from_str→json_to_vm_value物化)、shim_json_from_value(:417)、shim_str_find_manual(:173，str.find start_pos原生支持——009尾段直取正确性基础)；crates/auto-lang-core/src/vm/native.rs:791-850——auto.json.parse覆盖为vm感知版（Plan 446批三）、auto.json.stringify绑定shim_web_json_stringify。H1-H3的VM内部owner（集合迭代/比较求值/串池）未定位，属T-00有界调查。auto-lang master存在他计划未提交WIP（docs/design等），实施起点重冻结避开。

## 5. 详细设计

| Rule / task | 实际路径与owner | 代表状态 / 区分性关闭条件 |
|---|---|---|
| R01 / T00 | hazard四类在当前master逐类复核（probe重建入本scope）→处置表（消除/封界/豁免+理由）；json natives基线bench冻结（固定夹具：250KB级catalog raw、1000件对象数组、深层嵌套样例；口径：同机静窗口两run band）；受影响cargo组名单实核 | 每类处置可追溯probe；基线数字可复算；组名单经cargo t --list实核，不凭包名臆断 |
| R02 / T01 | H1-H3逐项：VM内定位→修复（语义保持）或封界（合同禁用+检测报告）；H4记基线 | 修后probe红→绿；封界类检测可复现报错/告警；round23等效probe组全量不回退；既有json/集合/串相关组全绿 |
| R03 / T02 | json.parse/stringify性能：物化路径（json_to_vm_value）、stringify路径优化（减少中间拷贝/重编码）；语义不变承诺 | 等价oracle：同输入逐值/逐字节对照+既有组全绿；基线上实测改善留档（parse与stringify分别before/after）；cold关键路径4大parse分量实测下降并入§8对账 |
| R04 / T03 | canonical_ok单parse化参考实现（.at，本scope）：单趟parse物化→段重建，替代流式门+逐段json.parse多趟；在H处置后允许区内实施 | 冻结千册catalog文本：输出与现路径逐字节hash相等；时延实测before/after（对2.87s/proof与prefs POST 7-11s分量）；未用封界模式合规注记 |
| R05 / T04 | 端到端：新VM pin+Reader兄弟@17e6b62+工具6ce87feb，静窗口cold-load两run band+分量对账（对round24基线16.54s分量表） | 达标（≤15s）→具名009消费就绪；未达标→剩余热点分量具名+证据材料；两结论均有对账表，不以单run/负载窗口宣称 |
| R06 / T05 | 同树独立终审：复演hazard probe组、等价oracle、参考实现证明与端到端测量 | 绑定reviewed_commit/revision；测量复核含静窗口判据 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-vm/vm-codegen-hazard-catalog.md（新） | 无成文hazard合同 → 已证hazard类目/触发条件/封界模式/检测与豁免流程；修复类逐条retire记录 | hazard目录由回执观察升为可执行合同约束 | AC-01/02 |
| SD-02 | add | docs/specs/auto-vm/json-natives-performance.md（新） | json native无性能/语义口径 → 语义保持承诺+基准夹具与测量口径+改善记录方式 | 性能合同可复核，防语义漂移 | AC-03/04 |

canonical_ok参考实现的Reader侧落地Spec（library-workspace.md性能证据）归READER-009消费时自施，不在本计划增量内。SD条目文字T-00/T-01结论后细化，终审前定稿。

## 6. 测试设计

| Case | AC / task owner | 输入与真实入口 | 区分性oracle / 边界 |
|---|---|---|---|
| W01 | AC-01/02；work T00/T01 | hazard probe重建（本scope，内容等效round23目录并覆盖四类边界：216KB量级、二遍迭代、跨模块布局、模块var对照）；封界类检测正/负样例 | H1-H3修后全绿且既有组不回退；封界类：禁用模式可复现红、合规写法绿；H4基线不回退 |
| W02 | AC-03；work T02 | json等价oracle：冻结夹具集逐输入 parse→(VM值)→stringify 往返与直接对照（含unicode/转义/大整数/嵌套/空集边界）；既有json相关cargo组全量 | 逐值/逐字节等价，语义零漂移；性能before/after同机同夹具两run band，不以单run宣称改善 |
| W03 | AC-04；work T03 | 参考实现vs现状路径：冻结千册catalog文本+prefs对象，输出hash对账；时延各两run band | 输出逐字节hash相等；实测改善或具名持平（附分量）；未用封界模式合规注记 |
| W04 | AC-05；work T04 | Reader pin链端到端（§2三件套），静负载窗口（CPU低占判据同009 round24）两run：cold-load分段计时对账表 | 对账闭合（分量和≈总时）；达标/未达标具名结论+材料；W-04其余叶不回退（grid/list/warm×3抽验） |

受影响cargo组：T-00经`cargo t --list`实核登记（候选域：vm执行/集合与串/ffi stdlib json），本计划不预设未核组名；新增probe模块登记入§5矩阵。实施/内检由本执行上下文；终审为独立上下文同权威树，写入暂停后复演W01～W04与Spec增量；不得同会话自审授pass。基准测量同机同口径，跨机数字不比较；009 evidence只读引用。

## 7. 验收标准

| ID | owner_stage | 可观察标准与验证 |
|---|---|---|
| AC-01 | work | hazard处置表冻结（四类逐项消除/封界/豁免+probe证据于当前master复核）；基线bench与夹具冻结；受影响组名单实核；验证见W01 |
| AC-02 | work | H1-H3逐项消除（probe红→绿）或具名封界（检测红/合规绿）；H4与round23等效组不回退；验证见W01 |
| AC-03 | work | json.parse/stringify语义零漂移（W02等价oracle全绿）且冻结基线实测改善留档；cold关键路径4大parse分量实测下降；验证见W02 |
| AC-04 | work | canonical_ok单parse化参考实现逐字节等价（hash对账）+时延实测记录+封界合规注记；验证见W03 |
| AC-05 | work | 端到端静负载cold-load两run band+分量对账闭合；达标或未达标均具名结论与材料；预算15s未修订；验证见W04 |
| AC-06 | review | 独立上下文同权威树复演W01～W04与Spec增量，绑定reviewed_commit；AC-01～05全过才reviewed |
| AC-07 | merge | 精确主干交付（auto-lang；如调查证实涉他仓按068先例成对交付）、冻结受影响并集、canonical核验、归档/计数、清理与009交接（pin/bench报告/pattern文档/hash）；不纳入他计划WIP |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际动作 / 关闭条件 | AC |
|---|---|---|---|---|
| T-00 | work | 授权已给（round27①A） | 新树重冻结master tip（避开他计划WIP）；hazard四类复核与处置表（R01）；基线bench/夹具冻结；cargo组名单实核 | AC-01 |
| T-01 | work | T-00 | H1-H3修复/封界落地+永久probe（W01）；H处置后重申R03/T02可安全区 | AC-02 |
| T-02 | work | T-00（若涉H区则T-01） | json natives性能优化+等价oracle（W02）；before/after留档 | AC-03 |
| T-03 | work | T-01/T-02 | canonical_ok单parse化参考实现+等价/合规证明（W03） | AC-04 |
| T-04 | work | T-02/T-03 | 端到端静负载复测+对账+结论材料；交付pin/工具/bench报告/交接件制备；work-owned全过才execution_done | AC-05 |
| T-05 | review | T-04 | 同树独立终审复演W01～04与Spec增量，绑定reviewed_commit/revision | AC-06 |
| T-06 | merge | T-05 pass | 精确落地/冻结并集/canonical/归档计数/清理/009交接（068先例：交接件入009自有scope，耐久摘要在案） | AC-07 |

起草时0/7。与auto-ui/PLAN-069（缺陷计划，用户建议优先）无写入交集（不同仓），可并行起草后按009消费顺序交付。

## 9. 复审记录

2026-11-07：stage: new | PLAN-995 r1 | outcome: pass（草案可交接，未执行） | changed: 新建T-00～06、AC-01～07、W01～04、SD-01～02 | next: work T-00（hazard复核+基线冻结）。授权依据READER-009 round27①A；canonical_ok落地上限注记（§2：参考实现+009消费re-land，避免跨树双写入）为立项时口径，用户审阅可更正；H1-H3的VM内部owner未定位，T-00有界调查定谳前不实施依赖修复；所有性能提案未执行，不预报结果。

2026-11-08：stage: work | PLAN-995 r1 | T-00 完成记录（hazard 复核+基线冻结+组名单实核）。worktree `D:/autostack/.wt/lang-995/auto-lang`（codex/plan-995-dev，base=b90c17dd1 master tip）；工具链组 `.wt/lang-995/`（auto-man@2a9b16e、auto-web@2d69841、auto-ui@061f4273、auto-down@58ea33f 均 detached，auto.exe 已建）。probe 材料 `D:/autostack/plan-artifacts/plan-995/auto-lang/probes/`（自建 h1/h2/h2b/h3-minimal/h3-extrafn/h4/h5/h6 + 原件引用副本 orig-dyn-iter-probe-{01,02,03,05,07,08,10..17}；fixture：catalog-1000.json=009 冻结 216,616B 拷贝 sha256 943a22f1…c5cd38、deep-40.json 102,809B sha256 781a5d60…）。

**T-00 hazard 处置表（双 pin 对照：master b90c17dd1 新建工具链 vs 009 t01 冻结工具链，probe 双跑）**：

| 类 | round23 目录主张 | 当前 master 复核结果 | 旧 pin（009 冻结件）对照 | 处置 |
|---|---|---|---|---|
| H1 动态集合二遍迭代空段 | probe-01/05 二遍空段 | 原件 probe-01/10 全绿；重建 h1（千条 books 两遍 n1=n2=1000、groups-after-books、嵌套 tags 两遍、bind-first）全绿 | 同绿 | **不复现（双 pin）**→豁免+W01 永久 probe 钉住；ws-perf01 时代遗留结论，round24 pin 已无 |
| H2 if 条件内嵌方法调用==大文本误判 | probe-08 trace vs fulldiff | 受控矩阵 h2b（distinct 216KB、helper/inline 两种拼段 × if/expr/swap 三位置 + bisect first_diff）全 true；trace 形+正确闭合+inline trim 全 true（len 216616=216616） | 同绿 | **证伪/归因修正**：误否族（orig 05、08-proof_body/trace、13/14/15/16/17）根因=probe 源码缺 `bs = bs + "]"`（bisect 实证 fd=216478、缺 1 字节、双 pin 同一），inline-trim 归因系双变量混淆（trace 无闭合+inline vs fulldiff 有闭合+bind-local）；VM 比较路径无缺陷→W01 钉住含闭合 oracle 的四位置对照 |
| H3 大串累积跨模块形状敏感 | 03/05 真 vs 13/16/17 假 | h3-minimal vs h3-extrafn（+额外累积比较函数的布局）重建输出逐字节同（sha 一致）；h2b 两种拼段形 first_diff=-1 | —（05 在旧 pin 亦假，同 fd） | **证伪/归因修正**：真假分歧由缺失 `]` 完全解释；master 布局无关→W01 钉住双布局等价 oracle；009 交接件注明其 t03 探针版本史需复核 |
| H4 模块 var 大串赋值/存串比较 | probe-07/11/12 安全基线 | 原件 07/11/12 绿 + 重建 h4（store/check/dump，216KB 模块 var 跨请求存取+if/expr 两位置比较）绿 | — | **基线确认**→W01 钉住 |
| **H5（新发现，T-02 范畴）** json.stringify 少参 ABI 崩溃/垃圾 | 目录未载 | `json.stringify(d)` 1 参→panic "Stack Underflow (nanbox)" virt_memory.rs:478（service 死）或垃圾 ""；2 参→垃圾 "0"；3 参→正确（h6-mini probe 实证）。根因=shim_web_json_stringify 固定弹 3 参（value/replacer/space）而 .at 调用按实参压栈，native_catalog:2414 注释声称的"codegen 缺参补位"从未落地 .at 路径 | 未测（T-02 修复对象） | **活缺陷**：T-02 先修正确性（JS 语义 stringify(v)=compact，缺参补 null 占位）再做性能；h6 1/2 参为红例，修后转绿入永久 oracle |

**基线 bench 冻结**（h5-bench，静窗口，10 次迭代/端点，3 run band；stringify 用当前唯一正确的 3 参形态）：catalog-parse-10 0.194/0.197/0.233s；catalog-stringify-10 0.149/0.149/0.156s；deep-parse-10 0.082/0.084/0.061s；deep-stringify-10 0.042/0.045/0.039s；roundtrip-eq（parse→stringify 逐字节==原文）catalog/deep 双 fixture true。canonical proof 形状时延（服务端）：orig probe-03 proof-real 1.7s、probe-05 proof 2.0s、probe-08 variants（3 proof）6.5s——重建累积路径（.at 层字段访问+STR_CAT）而非 parse 本身主导，T-03 单 parse 化的目标面确认。

**cargo 组名单实核**（`cargo t --list`）：受影响并集=**vm、auto-lang-core、auto-lang、http、corpus**（理由：engine ops/集合/串池+ffi stdlib json shims+native 绑定+codegen 调用补位在 auto-lang-core/vm 组；宿主接线 auto-lang；probe 走 service vm=http 组；.at 语料=corpus 组）。终审后冻结并集按此执行。

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass(T-00) | worktree: D:/autostack/.wt/lang-995/auto-lang | commit: b90c17dd1(基线，无代码改动) | next: work T-01（W01 永久 probe 落地）。

2026-11-08：stage: work | PLAN-995 r1 | **T-01/T-02/T-03 完成记录**（worktree 同上，代码改动见下 commit）。

**T-01（W01 永久 probe + 处置落地）**：新测试文件 `crates/auto-lang-core/src/vm/tests_json_hazards.rs`（vm.rs 登记）7 例全绿——h1 二遍迭代千条（豁免钉住）、h1 嵌套二遍、h2 大文本等值四位置+inline trim+负控、h3 双布局等价、h4 模块 var 大串、h5 stringify 少参 ABI、w02 往返逐字节（预置）。**新发现并修复两个活缺陷**（T-00 处置表 H5/H6）：
- **H5 修复**：`json.stringify` 少参 ABI——PLAN-057 T6 的缺参补位只匹配 `JSON.stringify`（JS 轨），`.at` 自然拼写 `json.stringify` 未补位即弹穿（h6-mini service 探针：1 参 panic "Stack Underflow (nanbox)" virt_memory.rs:478 / 深栈垃圾 ""、2 参垃圾 "0"）。修复=expr_calls.rs 双拼写统一补位（replacer=null、space=0，JS undefined→compact 语义）。服务级验证：1/2/3 参输出全等。
- **H6 修复**：`Str.substr`/`Str.slice` start-clamp 方向——原前跳（lo+=1）使步进读缝隙丢整字（跨块 CJK 被上块 hi-回退排除+下块 lo-前跳跳过；h7-diag 实证 66354→66348 丢 6 字，分歧点 `标签`→`标`）。修复=lo 回退式 clamp（`s[boundary↓(start)..boundary↓(start+len)]`，步进覆盖无损）。护栏：auto-lang crate 100 项 string 测试绿。

**T-02（json natives 性能+等价）**：①parse 单趟物化——`DirectMatSeed/DirectMatVisitor/DirectTopMatVisitor`（serde 直驱建 ListData/GenericInstanceData，消除中间 serde_json::Value 树）；两段式入口 `parse_json_direct`（IgnoredAny 零分配语法预校验保错误路径栈干净 + 深度类错误 unwind 含影子释放）；u64 臂先归一 i64（serde_json 直驱正整数走 visit_u64，与经 Value 中转 as_i64-first 的语义差首次暴露并修正——否则正整数字段全物化 BigInt 断 `.to_string()`，p949/plan738/plan988 红已全复绿）。②stringify 直序化——`VmJsonDirect`（serde Serialize 直写，无中间树；GenericInstanceData 插入序/ObjectData 字典序/BigIntData i64/u64/非有限浮点错误文本逐臂对齐），shim_web_json_stringify 与 shim_json_from_value 接入，错误映射保持旧文本分类。
**实测（h5-bench 同机同夹具，3 run band，10 迭代/端点）**：catalog-parse-10 0.194/0.197/0.233 → **0.116/0.137/0.170（中位 -30%）**；catalog-stringify-10 0.149/0.149/0.156 → **0.075/0.095/0.096（中位 -36%）**；deep-parse-10 持平（0.061-0.084 band 重叠，100KB 单串叶主导）；deep-stringify-10 5-run 复测 band 0.024-0.048 与 before 0.039-0.045 重叠（无回归）。W02 等价：roundtrip parse→stringify 逐字节==原文，catalog/deep 双 fixture true（service 级）+ Rust 级 w02_json_roundtrip_byte_equality 绿；json 域测试 auto-lang-core 32 绿 + auto-lang 7/8（唯一红=plan872 绑定表 drift，**pristine master 同红存量**，非本计划改动，triage 待其 owner）；原件回归 orig probe-03 proof-real eq=true 不回退。
**cold 4 大 parse 分量预期**：parse 20ms→14ms/次（-30%），端到端 cold 分量实测入 T-04 对账。

**T-03（canonical_ok 单 parse 化参考实现）**：probe `h8-canonical`（.at 参考实现+现状对照+负控，Reader 侧落地归 009 消费 re-land——§2 口径）。**等价证明**：`/canon/singleparse-proof` proof=true（rebuilt==raw，216616=216616，.at 内逐字节比较）+ `/canon/singleparse-rebuilt` 回传文本 sha256 与 h4 dump（同文本独立构造）一致；负控 spacing（值等价字节不等）正确拒（eq_rebuilt_mutated=false, len 216616 vs 216617）、负控 enum（status=banned）正确拒（rejected=true）。**时延实测（2 run band）**：singleparse-proof 2.425/2.210s vs multiparse-proof（3×全量 parse 段重建对照，无校验轻量版）1.867/1.967s——parse 本身已降至 ~14ms（×3→×1 节省 ~27ms），重建累积（.at 层 STR_CAT 链）主导 ~1.8s；单 parse 化对 Reader 的真实收益=消除 valid_catalog 字符级扫描门+段切片+冗余物化（009 侧 2.87s/proof 与 prefs POST 7-11s 的主要成分），本参考实现证明其可行性与逐字节等价性。**合规注记**：全程未用任何封界模式（H1-H4 处置后无 hazard；二遍迭代/内联比较/跨布局均 W01 钉住）。

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass(T-01/02/03) | worktree: D:/autostack/.wt/lang-995/auto-lang | commit: 待提交（worktree 未提交改动：tests_json_hazards.rs 新增、strings_json.rs/sys_env.rs/expr_calls.rs/vm.rs 修改） | next: work T-04（端到端 pin 链复测；reader@17e6b62 已检出 .wt/lang-995/auto-reader，工具 cohort 6ce87feb 交接件在案）。

2026-11-08：stage: work | PLAN-995 r1 | **T-04 完成记录（端到端 pin 链静负载 band + 对账）**。

**链**：reader 兄弟检出 `.wt/lang-995/auto-reader` @17e6b62（detached 只读，009 活动 worktree 零接触）+ 工具 cohort 6ce87feb（handoff/reader-p068-ui.exe，sha 断言）+ 本计划 VM pin（auto.exe sha 头 777dfab5，auto-man@2a9b16e + lang-995 worktree@f31b85a84）。方法=009 同法（verify_workspace_ui_vm.py，隔离 out-dir/自有端口/全新 fixture），静窗口（夜间，无并发构建）。

**两 run band**：cold-load **16.58s / 16.23s**（round24 基线 16.54s——带内持平；15s 预算未达且不修订）。**其余 6 叶全 PASS 不回退**：grid/list 21页 walk、warm-query 0.29s、warm-sort 0.69-0.74s、warm-filter 0.54-0.56s、q-exit=0。

**分量对账表（run1/run2，vs round24 基线分量）**：boot ~1.5s（持平）；GET /api/library/books **3.61/3.75s**（基线 3.9，-0.15~-0.29s）；GET /api/library/catalog **2.33/2.48s**（基线 2.6，-0.12~-0.27s）；POST /api/library/catalog/prefs **8.10/7.73s**（canonical_ok 重建比较×2+catalog 全量重序列化——**剩余最大热点**，即 T-03 参考实现的 009 侧 re-land 目标面）；渲染/采样 ~1.0s；分量和≈总时闭合（App_Init waited 12.74s 与 GETs+POST 串行吻合）。

**AC-03 cold 4 大 parse 分量核**：native 级 parse -30%（20ms→14ms/次，h5-bench）；端到端可见于 books/catalog GET 的小幅下降（parse 每 GET 一次 ~6ms 节省，占 GET 总时 <0.2%）——parse 已非 cold 瓶颈，与 T-03 结论一致（重建累积主导）。

**结论（AC-05 具名）**：**未达标**（16.23-16.58s > 15s）。VM pin 交付就绪；剩余缺口具名于 Reader 侧路径（prefs POST 7.7-8.1s 与 GET 内 .at 层重建），按 §0/§2 设计由 009 消费 pin + re-land 单 parse 化参考实现后重测 W-04 cold 叶。**不以单 run/负载窗口宣称**；band 材料在案（e2e/band.json + run1/run2 results+app.log）。

**pin/交接件制备（work 侧）**：`plan-artifacts/plan-995/auto-lang/handoff-009-pin-and-findings.md`（pin 身份/行为变化/参考实现位置与证明/hazard 处置定谳/cold band 对账/存量红 triage）；merge 时按 068 先例入 009 自有 scope。

**受影响组门（冻结代码 f31b85a84 前置 run）**：vm/auto-lang-core/auto-lang/http/corpus 1712 例 **1710 绿 / 2 红均 pristine master 存量**（plan872 绑定表 drift、sys_natives_vm_tests line49 断言——主检出复跑同红实证，非本计划改动，triage 记录在交接摘要）。

**work handoff（execution_done）**：work-owned AC-01～05 全过（§9 各段证据锚：处置表=W01 测试+双 pin 对照、等价=roundtrip 逐字节+json 域 32 测试、性能=h5-bench band、参考实现=h8 证明+负控、端到端=band+对账表）。无未决 in-contract finding；无待答问题。统一回归/冻结并集=review/merge 所属门（待 T-05/T-06）。提交 f31b85a84（worktree clean）。

- stage: work | plan_id: PLAN-995 | plan_revision: r1 | outcome: pass（execution_done）| worktree: D:/autostack/.wt/lang-995/auto-lang | branch: codex/plan-995-dev | base_commit: b90c17dd1 | code_commit: f31b85a84 | dependency_revisions: auto-man@2a9b16e, auto-web@2d69841, auto-ui@061f4273, auto-down@58ea33f（构建依赖，detached）；reader@17e6b62+cohort 6ce87feb（T-04 消费链，只读）| task_ids: T-00✅ T-01✅ T-02✅ T-03✅ T-04✅（T-05 review、T-06 merge 待）| evidence: §9 T-00 处置表/基线/组名单 + T-01/02/03 记录 + 本 T-04 记录；plan-artifacts scope 全材料 | blockers: 无 | next_responsible_role: review（独立终审复演 W01～W04 与 Spec 增量定稿）。

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-995/auto-lang | commit: f31b85a84 | next: review T-05。

2026-11-08：stage: review | PLAN-995 r1 | **T-05 独立终审记录（final）**。

**基线**：reviewed_commit **8772f5d66**（= f31b85a84 代码 + SD-01/SD-02 定稿；docs-only 增量，工具链二进制按 f31b85a84 代码等价使用）。base b90c17dd1；分支 codex/plan-995-dev；worktree clean（status 0 行）。依赖：auto-man@2a9b16e、auto-web@2d69841、auto-ui@061f4273、auto-down@58ea33f（构建，detached）；T-04 消费链 reader@17e6b62 + cohort 6ce87feb（只读）。**reviewer 身份/局限**：goal 会话内终审（实施同上下文）——结论从工件重建+独立复演得出（确定性哈希逐字节比对、测试重执行、e2e 重跑），非采信执行方摘要。

**W01-W04 复演（本审查实际执行）**：
- W01：`cargo test -p auto-lang-core --lib vm::tests_json_hazards` → **7/7 ok**（1.63s）。
- W02：`--lib json` → auto-lang-core **32/32 ok**；auto-lang 7/8（唯一红 plan872 绑定表=pristine master 同红存量，已 triage）。
- W03：h8-canonical 五端点复演——singleparse-proof proof=true（1.801s）、multiparse 1.927s、rebuilt/negative-spacing/negative-enum 响应 **sha256 与 work 阶段逐字节一致**（d738b0ab/92fbbaa0/b8bda1eb/470841cb/911dc482）——确定性复现。
- W04：e2e 确认 run3-review：cold **17.71s**（band 16.23-16.58 +1.1s，三 run 展布内；"未达标>15s"结论稳定）；分量 books 3.89/catalog 2.72/prefs POST 7.89s 结构与 band 一致（对账闭合）；其余 6 叶全 PASS。

**健康检查**：我的 5 文件零新增编译警告（strings_json:890 unused-doc-comment 与 expr_calls:466 fmt 漂移均为 pristine master 存量，行号位移所致）；新增行 fmt 干净；diff 无新增无条件 debug print（SUBSTR-CLAMP eprintln=存量 env 门控搬移；RwLock read().unwrap()=仓内惯例）；无 TODO/FIXME/workaround 标记。

**AC 清单审计**：AC-01 pass（处置表+双 pin 证据+夹具 sha+组名单实核；夹具 216KB 属"250KB级"口径、千册数组=catalog 本身、深嵌套=deep-40）；AC-02 pass（H1-H3 定谳=不复现/证伪（强于"消除"），无封界类具名（R01 允许豁免处置，vacuous 注记），H4 基线钉住）；AC-03 pass（等价 oracle 全绿+band 留档 -30%/-36%+cold parse 分量下降：native 20→14ms/次、GET 3.9→3.6-3.9/2.6→2.3-2.7s）；AC-04 pass（逐字节哈希+负控+时延记录+合规注记）；AC-05 pass（两 run band+分量对账闭合+未达标具名+预算未修订）；AC-06 本轮 pass；AC-07 merge 所属。

**遗漏/延后/workaround 扫描**：§5/§6/§8 各行核对无丢项——无封界类（vacuous 已注记）、canonical_ok 落地按 §2 口径归 009 re-land（合同内）、统一回归=merge-owned pending（记录非阻塞）。存量债清单（非本计划）：plan872 绑定表 drift、sys_natives_vm 断言、expr_calls 存量 fmt 漂移、strings_json lazy_static doc-comment 警告——均 pristine master 同在，triage 在交接摘要。

**Spec 增量核验**：SD-01 `docs/specs/auto-vm/vm-codegen-hazard-catalog.md`（六类定谳表+五条合同规则+检测/豁免流程+retire 记录）与 SD-02 `docs/specs/auto-vm/json-natives-performance.md`（语义保持承诺五条+夹具/口径+改善记录表+非瓶颈注记）随 8772f5d66 提交，内容与实证一致、路径沿 auto-vm/zip-natives 先例。frontmatter `new_spec_components` 与实际路径一致；`supersedes_spec_components=[]`、`touched_goals=[]`（无既有 goal 触面，SD 全为新增文档）。

**结论：final pass**（绑定 reviewed_commit 8772f5d66 / r1）→ status reviewed；merge-owned pending：冻结受影响并集复跑（vm/auto-lang-core/auto-lang/http/corpus 于最终 merge 树）、canonical 落地核验、归档/计数、wt-guard 清理与 009 交接件移交（068 先例）。

- stage: review | review_scope: final | reviewer: goal会话内终审（局限已注记，工件重建+独立复演） | plan_id: PLAN-995 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-995/auto-lang | branch: codex/plan-995-dev | reviewed_commit: 8772f5d66 | base_commit: b90c17dd1 | dependency_revisions: 同上 | spec_inputs: docs/specs/auto-vm/{vm-codegen-hazard-catalog,json-natives-performance}.md（随审提交） | acceptance_results: AC-01~06 pass / AC-07 merge-owned | pending_closeout_gates: 冻结并集复跑、canonical 核验、归档、清理、009 交接 | findings: 无 blocking（存量债 4 项已 triage） | evidence: 本段 W01-W04 复演数字+哈希 | next: merge T-06。

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 5 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-995/auto-lang | commit: 8772f5d66 | next: merge T-06。


- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 1 | from: new | to: work | status: drafting→work待启 | outcome: pass | worktree: 待T-00创建（D:/autostack/.wt/lang-995/auto-lang拟） | commit: 本主检出簿记提交 | next: work T-00。

## 10. 待澄清事项

无待答范围授权。三项注记（非用户待答）：

1. **canonical_ok落地划分**：§2注记为本计划立项口径（参考实现+009消费re-land），理由=一写入者规则；用户round27①A原文将其列入本计划范围，如要求本计划直接落地Reader仓，请明示，届时修revision并处理009写入面移交。
2. **测量环境纪律**：round23实证环境负载±2×浮动；W04/T-04与AC-05全部数字以静负载窗口两run band为准，与009 round24基线（16.54s，ws-ui-vm-13）同口径对账；不与负载窗口数字比较。
3. **与PLAN-069先后**：用户建议缺陷计划优先；本计划T-00不依赖069交付，可在009消费排序上让先。若069实施中对VM域提出新hazard/缺陷证据，按其§9回执互引，不在本计划内重复诊断。

2026-11-08：stage: merge | PLAN-995 r1 | **T-06 落地/归档/清理回执**。

| checkpoint | 证据 |
|---|---|
| prepared | reviewed 8772f5d66（r1 final pass）+ SD-01/SD-02 定稿；worktree clean |
| landed | rebase 链 f31b85a84→cb721ecf6→**6b08e7ff1**（feat）与 8772f5d66→dee4ca694→**df55a6289**（SD），两段 range-diff 全等（安全改写证明）；master tip=df55a6289（ff-only，无 merge commit，线性历史；期间两次他计划 docs-only 簿记 6f4246042/8dce71961 落于中间，无冲突） |
| canonical_verified | docs/specs/auto-vm/vm-codegen-hazard-catalog.md sha256 55f17bcca2614d99…、docs/specs/auto-vm/json-natives-performance.md sha256 0695281607fe9d8a…，均在 df55a6289 内 |
| 冻结并集门 | merge-owned 于最终树复跑：`cargo t vm auto-lang-core auto-lang http corpus` = 1712 例 **1710 绿**，2 红=plan872 绑定表 drift + sys_natives_vm_tests（**pristine master 同红存量**，主检出复跑实证，triage 在 009 交接件）——零新增红 |
| archived | docs/plans/archive/995-vm-json-natives-perf-and-hazard-closure.md，status: archived，completion_kind: delivered |
| 009 交接 | D:/autostack/plan-artifacts/plan-009/handoff-plan-995/（摘要+canonical-singleparse-reference.at 参考实现+bench band+e2e band+终审 run；068 先例入 009 自有 scope） |
| plan-artifacts | plan-995 整 scope 按 2026-10-31 删件裁定删除（材料已折叠 §9 与 009 交接件）；registry .retention/policy.json 刷新 |
| 生产落地观察 | landing≠deployment：无本仓自持生产进程；消费面（auto-reader/009）按交接件自建新 pin 后重测 W-04 cold 叶（§0 设计）；观察项无 stale 本仓工件 |
| cleaned | wt-guard lang-995/auto-lang clean → worktree/分支删除；组内兄弟（auto-man/auto-web/auto-ui/auto-down detached 构建依赖 + auto-reader@17e6b62 只读 pin 检出）各自 repo worktree remove；组目录删除 |

- stage: merge | plan_id: PLAN-995 | plan_revision: r1 | outcome: pass（delivered/archived/cleaned） | delivery_commit: df55a6289 | canonical: SD-01/SD-02（哈希见上） | archive: docs/plans/archive/995-vm-json-natives-perf-and-hazard-closure.md | cleanup: 完成（见 cleaned 行） | next: 批量回归 due check（995%5=0）→ regress。

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 6 | from: merge | to: done | status: archived | outcome: pass | worktree: （已清理） | commit: df55a6289 | next: 批量回归 due（regress 技能）。

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 7 | from: merge | to: done | status: archived | outcome: pass | commit: df55a6289（delivery）/ 0093e7eca（archive）/ 68210696b（批量回执） | next: done——批量回归已执行：28 PASS/6 FAIL/3 N.A.，原子红 3（fixture_manifest、sys_natives=956 同族已知；plan872=新增红，归属 PLAN-941 债 D-941-1），api-gen 崩溃未复现；回执 .last-batch-regression.json 已提交（covered 69da2dea8，last_covered_plan_id 995）。目标完成。
