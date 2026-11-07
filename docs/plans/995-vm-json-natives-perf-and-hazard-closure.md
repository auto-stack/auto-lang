---
plan_id: PLAN-995
status: drafting
feature_name: VM json native性能与codegen hazard收口（READER-009 cold-load框架前置）
author: [ZCode]
created_at: 2026-11-07
updated_at: 2026-11-07
plan_revision: 1
current_step: 0
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

- stage: goal | plan_id: PLAN-995 | plan_revision: r1 | round: 1 | from: new | to: work | status: drafting→work待启 | outcome: pass | worktree: 待T-00创建（D:/autostack/.wt/lang-995/auto-lang拟） | commit: 本主检出簿记提交 | next: work T-00。

## 10. 待澄清事项

无待答范围授权。三项注记（非用户待答）：

1. **canonical_ok落地划分**：§2注记为本计划立项口径（参考实现+009消费re-land），理由=一写入者规则；用户round27①A原文将其列入本计划范围，如要求本计划直接落地Reader仓，请明示，届时修revision并处理009写入面移交。
2. **测量环境纪律**：round23实证环境负载±2×浮动；W04/T-04与AC-05全部数字以静负载窗口两run band为准，与009 round24基线（16.54s，ws-ui-vm-13）同口径对账；不与负载窗口数字比较。
3. **与PLAN-069先后**：用户建议缺陷计划优先；本计划T-00不依赖069交付，可在009消费排序上让先。若069实施中对VM域提出新hazard/缺陷证据，按其§9回执互引，不在本计划内重复诊断。
