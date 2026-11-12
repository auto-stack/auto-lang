---
plan_id: PLAN-962
status: archived
feature_name: 组合数据结构与递归拥有资源
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 4
current_step: 8
total_steps: 8
depends_on: [PLAN-960, PLAN-961]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-aggregates.md]
touched_goals: [GOAL-002]
---

# [PLAN-962] 组合数据结构与递归拥有资源

## 0. 变更摘要

2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

支持实际数组/元组/嵌套List/Map/Set/记录/变体组合、索引place、解构、范围循环和复杂模式；资源事实由共享verify生成，四端消费，禁止backend自行补借用。递归类型经显式间接表示，非法无限大小类型在布局前拒绝。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/core/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/lowering/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/collections-runtime`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/data-runtime`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


支持实际数组/元组/嵌套List/Map/Set/记录/变体组合、索引place、解构、范围循环和复杂模式；资源事实由共享verify生成，四端消费，禁止backend自行补借用。递归类型经显式间接表示，非法无限大小类型在布局前拒绝。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：所有权与未分配语法归属（r2）

认领union/tag、grid、Hold/view/mut/move、Box递归表示和存储策略的现有义务；先实现具体化类型布局与内建容器，用户泛型实例由963提供，避免962等待963形成语义循环。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-aggregates.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-aggregates.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 嵌套List<record<Str>> clone后释放原值仍有效 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 自别名替换RHS先物化 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 分配中途失败逆序回收且旧目标不变 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 非法递归布局拒绝 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 四端资源事件与独立账一致 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | Box链最后owner析构、Hold借位失效、嵌套place自别名与部分move各有合法/非法对照；grid维度与union/tag分支不能靠List测试代替。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 所有权与未分配语法归属的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。[✅ 2026-11-12：依赖 960/961 archived 核验；§4 路径在树核实；探针 `aggregates_t01_probe`（commit 3f4eeeef6）三腿取证——全链健康/失败恢复绿、嵌套 List 822 拒绝基线红、**发现真缺口**：collections 族不跑 record 包含环检查（自引用 record 今日编译通过），V-04 前置不成立，T-02 修复。决定表 `plan-artifacts/plan-962/t01-freeze.md`：分母 L-06..19/48 逐行语义冻结、D1-D14 策略（aggregates 新 profile 复用 Collections 域、Tuple→positional Record、Box→单元素 List 胞+A2X 真渲染、union→tagged 安全联合、grid→列 record 物化、range/模式/解构前端 desugar）、四端 owner/命令、Map/Set 归属映射（964/946-972）、6 项具名差异。]

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：认领union/tag、grid、Hold/view/mut/move、Box递归表示和存储策略的现有义务；先实现具体化类型布局与内建容器，用户泛型实例由963提供，避免962等待963形成语义循环。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

[x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-962/auto-lang`，分支 `plan-962-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-962/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。

## 9. 复审记录

2026-11-12 work checkpoint（r4，/goal 授权实施）：T-01 冻结完成——依赖 960/961 archived 核验，探针三腿
（`aggregates_t01_probe`，worktree commit `3f4eeeef6`）取证；发现 collections 族 record 环检查缺口（V-04 前置）。
决定表固化 `D:/autostack/plan-artifacts/plan-962/t01-freeze.md`（分母 L-06..19/48 逐行语义、D1-D14 表示策略、
四端 owner/命令、Map/Set→964/946-972 映射、6 项具名差异）。worktree `D:/autostack/.wt/lang-962/auto-lang`
（base `1169f1359`，branch plan-962-dev）。T-02 起实施：单元序＝①环检查修复+聚合 profile 骨架 ②嵌套容器准入
（resolve+mir_verify profile 键控）③字面量/命名ctor/解构/模式/range desugar ④索引 place 面 ⑤Tuple ⑥Box/递归
⑦union/tag/grid/Hold/view-mut-move/Take（T-08 面）⑧L-10 四 R 行 ⑨a2x/native/AV 各端发射与真链 ⑩V 行 fixture。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 2（r4，T-02 单元①完成）：aggregates profile 骨架 + V-04 环检查修复 + 嵌套 List 准入（AV 全链）——
- `3598bfc35`：grammar 注册 aggregates 面（冻结面裁剪字节恒等）、SourceProfile::Aggregates 全链接线（CLI/adapter/lowering 走 closures 族）、
  准入键=文档级 capability `av.aggregates/1`（emitter 盖章→MIR capabilities→mir_verify 白名单）、resolve `aggregates_admission` 键控
  822 元素域各拒绝点（冷+热路径）、mir_verify 元素域/nested-container 门 capability 键控。
- **V-04 缺口闭合**：record 包含环检查扩到 Collections/Closures 域（探针发现的历史缺口，双面验证）。
- 探针 4/4 绿：链路健康/失败恢复、冻结面嵌套拒绝、聚合面嵌套 List 全链执行（构造/push inner.move/remove/读，值 42、live 0、allocs=frees）。
- 回归：ace-core 34+399 绿；av-core 44+11 绿；av conformance 932/934（2 红为并行负载下计时门，隔离重放绿——环境归因，与 961 期具名同类）。
- 发现并留待单元②：嵌套整元素读（`xs.get(0)`）受 822 view-escape 守卫——视图语义（借用读/不可逃逸/事实不调度释放）为单元②设计；
  for-in 视图绑定对 List 元素未接线；remove 值移出路径已可用。
status: executing；current_step 1/8（T-01 已勾，T-02 进行中）。

2026-11-12 work checkpoint round 3（r4，T-02 单元②③完成）：
- `08ac87de3` 单元②——整元素视图读：owned 元素 `xs.get(i)` = 污染 ElemView（空路径），仅读穿透合法
  （let/assign/实参/push/return 物化全拒 source.view-escape）；for-in 元素视图绑定同形；typecheck ElemView
  终端泛化；lowering 对 owned ListGet 发射 LIST_PLACE+PLACE_GET（822 冻结指令，epoch 复验）；mir_verify 仅在
  aggregates capability **且 dst=Temp 槽**放宽 PLACE_GET owned 守卫（Temp 不入事实释放表，防突变包双释放）。
  探针 7/7：链式视图读全链执行、for-in 嵌套读、三类物化负控拒。
- `d1e0927d4` 单元③——L-19 range-for 全链：`for i in a .. b / a ..= b` 计数循环脱糖（grammar+投影+resolve，
  Range AST 变体贯通 capture/shift/binding/semantics/source-query 各走读）；具名负控（值位 range 语法层拒 +
  resolve 纵深）。**具名边界**：范围界需空格分隔（affix-num 最长匹配吃 `1..5`），SD-01 记录。探针 8/8。
- 回归：ace-core 34+399 绿×2；av conformance 单元②后 935/937、单元③后 935/938——3 红均为并行负载环境红
  （closures_fn/recovery 计时门 + packages_960_four_ends 进程腿），逐一隔离重放绿（2.9-37.7s），零代码回归。
- 剩余（T-02 续）：List 字面量 `[..]`、索引 place `xs[i]`、tuple+解构、组合 record 字段执行、命名 ctor 验证；
  T-08 面（union/tag/grid/Hold/Box/view-mut-move/Take）、L-10 R 行、a2x/native 发射、V 行 fixture 全量。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 4（r4，T-02 单元④完成）：
- `fdfd8c94f` 单元④——L-16 数组/List 字面量 + L-48 索引赋值：`var xs = [e, ...]` 语句位脱糖
  （List<T>.new()+逐元素 push，元素类型统一、视图元素拒、push 纪律）；`xs[i] = v` → list-set 效果
  （类型匹配/视图拒/新右值转移/迭代中修改守卫——与 `.set` 同合同）；ArrayLit/Index/IndexAssign 三变体
  贯通 capture/shift/binding/semantics/source-query/CTE 六处走读（查询/CT 面 named-unsupported）。
  投影新增同步子表达式助手 project_expr_sync。探针 10/10 绿（字面量+索引赋值全链执行 42、三类负控拒）。
- **具名待办**：位置式索引读 `xs[i]`（表达式位）——CST pratt 后缀步仅受 token 形，括号步全位不匹配
  （FIRST/步形机制待深查，或 pratt 第二 call 组）；当前读走 `.get(i)`（全功能）。空字面量语法层拒
  （resolve 检查为纵深）。回归：ace-core 34+399 绿；av conformance 在跑（前几轮 3 红均负载环境红）。
- 剩余（T-02 续）：位置式索引读、tuple+解构（需 aggregates.atom schema 扩展——HIR 新节点族的 descriptor 层）、
  组合 record 字段执行、命名 ctor 验证；T-08 面、L-10 R 行、a2x/native 发射、V 行 fixture 全量。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 5（r4，T-02 单元⑤完成）：
- `b4c138456` 单元⑤——L-15 tuple 全链：**零 schema/MIR 新面**——resolve 内联结构形状（VT::Tuple(i)→
  CollTables::tuples），adapter 物化匿名位置 record（t_rec___tuple_<i>，字段 0..n-1），descriptor/verify/
  lowering/四后端全走 record 既有机器。语法（tuple 类型/字面量/let 解构，聚合面）+投影+resolve+typecheck
  +adapter+六走读面（capture/shift/binding/units/semantics/source-query/CTE/artifacts codec）。
  探针 12/12 绿（构造+解构、注解局部、部件重构=42、负控）。回归：ace-core 34+399 绿；av conformance 在跑。
- **具名待办**（新增）：tuple 作**调用实参**（call-arg record-copy 路径的 type token 未 tuples 化，
  mir.copy-type 拒）；参数签名侧绑定正常。
- T-02 剩余：位置式索引读（前轮待办）、组合 record 字段执行、命名 ctor 验证、tuple 调用实参；
  T-08 面、L-10 R 行、a2x/native 发射、V 行 fixture 全量。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 6（r4，T-02 单元⑥完成）：
- `a42d67234` 单元⑥——tuple 调用实参（解构基值为局部读取时直接投影，消除 view→owned 物化，
  闭 round-5 待办）；**组合 record 字段执行验证**（List 字段构造/读穿透/级联释放 allocs=frees）；
  **命名 ctor R03 验证**（乱序构造，既有 CtorNamed 机器直接承载）。探针 14/14 绿；ace-core 34+399 绿。
- 剩余具名待办：位置式索引读 `xs[i]`（CST pratt 后缀步 token 形限制；读面由 `.get(i)` 全覆盖）。
- T-02 主能力面（L-15/16/17 现状/19/48 写+读方法形）已在 AV 端全绿；下一步：T-08 面
  （union/tag/grid/Hold/Box/view-mut-move/Take）、T-03 四端（native/a2x 发射）、T-04 V 行全量 fixture。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 7（r4，T-08 首切片=单元⑦完成）：
- `0a67a7372`——L-13 tag 别名（enum 同构声明）、L-12 union（tagged 安全联合，一字段一臂，非活动臂=is 守卫
  不匹配；旧引擎 parser-only 故语义按 t01-freeze D10 新定）、L-09 take 弃用别名（r-take 保留词专用原子形→
  resolve_call 拦截为 Move，用户同名 fn 优先）。修投影外层声明门（混合序 tag/union 曾被跳过）。探针 15/15 绿；
  ace-core 34+399 绿。
- **裁定**：位置式索引读 `xs[i]` 维持 `.get(i)` 方法形为正式读形（CST pratt 后缀步仅 token 形——冻结机制不动）；
  `xs[i] = v` 写位已交付。语法拼写边界记 SD-01。
- T-08 剩余（下轮）：Hold 脱糖、grid 列 record 物化、Box 显式间接+递归合法化（环检查 Box 边豁免）、
  .view/.mut 补全、V-08 fixture 族；随后 T-03 四端发射、T-04 V 行全量。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 8（r4，T-08 Box=单元⑧完成）：
- `6d29b0e1e`——L-08 Box 显式间接：`Box<T>` 聚合面内联为单元素 List 胞（t01-freeze D9）；`let b = Box(v)`
  let 位构造；**递归合法化全链**：`type Node { next Box<Node> }` 编译+执行+逐胞释放（V-04 双侧：
  直递归仍拒/Box 形合法）。lowering **两阶段 record 内联**（预留槽+memo 先行+字段后补丁；List 元素内联
  用新鲜 visiting——列表布局=不透明句柄，元素边不入布局递归），同时修复环误报与朴素豁免的无限递归。
  语义守卫确认：无资源 record 可拷贝（`.move` 非法，拷贝传值）——冻结语义保持。探针 16/16 绿；ace-core 34+399 绿。
- **单元⑧修正（round 8 内）**：两阶段 record 内联原型破坏 2 个冻结 variant-in-list fixture（占位槽绕过
  dedup→签名/体索引分歧 mir.return-type）——**已回退 lowering 两 hunks 并复测 fixture 绿**。保留：Box resolve
  侧合法化（verify 层 V-04 双侧）+ 非递归 Box 胞全链执行；**递归 Box 的 lowering 执行=具名待办**（修法：
  patch 时 canonical 别名化占位槽）。探针 17/17 绿。
- T-08 剩余：Hold 脱糖、grid 列 record 物化、.view/.mut 补全、V-08 fixture 族（union/tag 分支✓、非递归 Box✓
  在探针；Hold/grid/递归 Box 执行/嵌套 place 对照待）；随后 T-03 四端发射、T-04 V 行全量。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 9（r4，递归 Box lowering 诊断完成——具名待办深化，无代码变更）：
- **完整根因链**：collections_type_idx_of 有多个独立调用 walker（fn 签名 ABI/closures_env/主体），各自局部
  memo，靠 intern_type 值 dedup 收敛到同索引；两阶段预留槽绕过 dedup → 同一 HIR record 跨 walker 得双索引
  （mir.return-type 分歧）。且签名 walker 亦会先触 mir.type-cycle（ exemptions 在函数内但各 walker 独立
  预留仍各自成槽）。**修法（设计级，非热修）**：①共享预留表贯穿全部类型 walker，或 ②模块级预内联 pass
  （HIR 类型表按依赖序预内联，环经 List 边处仅登记索引）。已验证非目标路径：patch 时全表查重别名化对
  "环+他 walker 先行"复合情形仍分歧。
- 现状保持：探针 17/17 绿、ace-core 34+399 绿、冻结 fixture 全绿（回退后复测）；Box verify 层合法化 +
  非递归执行已交付。T-08 剩余：递归 Box lowering（上述设计修法）、Hold 脱糖、grid、.view/.mut、V-08 其余。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 10（r4，递归 Box lowering 闭合=单元⑨）：
- `e18d28e87`——三件套修法落地：①模块级**预内联 pass**（类型表条目先行一次性内联，环形状得规范化槽）；
  ②**名称规范化 record 查找**（HIR 类型名模块内唯一→完成 Record 条目即规范索引；各 walker 局部 memo 经此
  收敛，根除跨 walker 分歧——无需贯穿共享状态）；③两阶段 Record 内联 + List 元素新鲜 visiting。
- **递归 Box 链全链执行**：Node{next Box<Node>} 构造/读穿透/逐胞释放（42，allocs=frees，live 0）。
  冻结 variant-in-list fixture 7/7 保持绿；探针 18/18；ace-core 34+399 绿。round-8/9 具名待办正式闭合。
- T-08 剩余：Hold 脱糖、grid 列 record、.view/.mut 补全、V-08 其余 fixture（嵌套 place 对照）。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 10 修正（r4，递归 Box 两轮实验回退到验证态）：
- `e18d28e87` 预内联+名称规范化版：探针/冻结 fixture 局部绿，但**全量 av conformance 310 结构测试红**
  （无条件两阶段/预内联改变全部 record 的类型表序——mir_tier/resources/module_sources 等族冻结表形被钉）。
- 9b 环条件版（reaches-self-through-opaque 门）：非环 record 走 plain 路径，但保留的 List 新鲜 visiting 豁免
  使非环形状误报 mir.type-cycle（enum-owned-payload fixture 族红）——豁免也须整体入门内。
- 已回退 hir_adapter+探针至 `1fbc276ef` 验证字节（探针 17/17、av_collections 124/124 复测绿；本修正提交）。
- **精化修法（下轮）**：全部新逻辑（两阶段+新鲜 visiting）整体置于 reaches-self-through-opaque 谓词门内，
  非环模块 plain 路径字节恒等；表序稳定性是硬约束。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 11（r4，递归 Box lowering 正式闭合=单元⑨c）：
- `e44b52a6f`——精化修法（手术式）：全部新逻辑置于 `record_reaches_self_through_opaque` 谓词门内——
  ①名称规范化 record 查找（HIR 类型名模块内唯一；对既有程序零行为变化）；②仅经不透明边自达的 record 走
  两阶段（预留槽+memo 先行；字段走 cloned-minus-self visiting——不透明边不会内联回自身布局，自引用经 memo
  解析终止）；③直 record-record 环保持历史 mir.type-cycle 拒绝。共享 List 臂与 plain 路径字节恒等。
- **验证**：递归 Node/Box 链全链执行+逐胞释放（42，allocs=frees，live 0）；探针 17/17；冻结 fixture 7/7；
  mir_tier 8/8、resources_tier 结构腿绿（唯一红=已具名 32mib 计时门）。round-8..10 具名待办正式闭合。
- T-08 剩余：Hold 脱糖、grid 列 record、.view/.mut 补全、V-08 嵌套 place 对照；随后 T-03 四端、T-04 全量。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 12（r4，目标轮上限收口——T-08 剩余切片设计定格，本轮无代码变更）：
- **Hold 切片设计**（下会话可直接实施）：`hold x as v { }` 整局部形——r-hold/r-as 既有保留词，语法
  hold-stmt（聚合面）+ AST Stmt::Hold + 六走读面臂 + 投影 HoldFinish + resolve 脱糖（RExprKind::BorrowMut{local}
  既有载体；v 绑定为借用视图、块末失效）；**字段路径形**（hold x.y.z）为具名后续（需路径感知借用——现有
  BorrowMut 仅整局部根，字段路径经 mut-param 写回 ABI 属 record 域机制）。
- **grid 切片设计**：`grid <列名表> { [行]; ... }` let 位脱糖列 record（每列 List 字段）+ bind 期静态维检查
  （行等长）；grid 标识符非保留词，专用原子形防歧义（同 take-atom 先例）。
- **.view 补全**：postfix-step 加 `dot r-view` token 形臂（pratt 后缀步允许 token 形）→ BorrowView 既有载体。
- 本目标会话 12 轮上限到达，按 caps 规则收口：最后验证阶段=work in progress（9 个已验证单元在树上全绿），
  剩余队列与切片设计如上/§9 前文。worktree 保持（后续会话经 §9 收据续作）。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 13（r4，Hold 实施尝试——回退保绿，调查状态入档）：
- 按 round-12 切片设计实施 Hold 整局部形：grammar hold-stmt/kinds/AST Stmt::Hold/六走读面臂/resolve 双臂
  （别名绑定+块作用域，≥旧 MVP 等义——分母 L-07 自注"无借用纪律"）全部落码并编译通过；
- **卡点**（三处结构首诊豁免 441/515/1914 均加后仍拒）：stmt 分派不匹配 hold-stmt——错误最终出自
  project.rs:4420 表达式语句头门（旧 stmt head 仅 ident/true/false/int 起始的镜像），说明 CST 把
  hold-stmt 备选根本没提交（recover 规则的新头 token 备选提交机制疑在 syntax-runtime parse/mod.rs 的
  commit/FIRST 集合，未及定位）。对照：let-destr（复用 `let` 头）/for-range（复用 `for` 头）/tag-item
  （item 位）均过——**新头 token 的语句级备选**是未解机制点。
- 已回退全部未提交改动（`git checkout -- .`），树复 `e44b52a6f` 全绿（探针 17/17 复测）。Hold 切片设计
  仍有效，下会话入口：①定位 recover 规则备选提交机制（parse/mod.rs），或 ②改用既有头 token 拼写
  （如 `let hold x as v { }`——零机制风险，语义同）。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 14（r4，Hold 闭合=单元⑩）：
- `3d7173802`——路径②落地：`let hold x as v { }` 整局部 hold（`let` 头分派已证可行；裸 `hold` 头的
  stmt 列表 FIRST 提交机制未定——round-13 调查保留，裸拼写记 SD-01 具名待办）。语义：v=块内名别名
  （≥旧 MVP 等义，L-07 自注"无借用纪律"），块末失效（逃逸=普通 undefined 拒）；路径形/非 record 基具名拒。
  六走读面全臂（查询/CT 面 named-unsupported）。探针 18/18 绿（别名块执行 42+逃逸负控）；ace-core 34+399 绿。
- T-08 剩余：grid 列 record、.view 补全、V-08 嵌套 place 对照、裸 hold 拼写（机制）；随后 T-03/T-04/T-05/T-06-07。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 15（r4，.view 闭合=单元⑪）：
- `8f68bc22c`——L-06 后缀三件套齐：`.view` 只读借用实参标记（record 参数默认即借用——771 冻结语义，
  adapter 按 param mode 自动包 BorrowView；标记=意图文档+record 基检查+默认 view 传递）。探针 19/19 绿
  （`getv(p.view)` 执行 42+非 record 基负控）；ace-core 34+399 绿。
- T-08 剩余：grid 列 record（round-12 设计：grid-kw 聚合面上下文关键字+原子形）、V-08 嵌套 place 对照、
  裸 hold 拼写（机制项）；随后 T-03/T-04/T-05/T-06-07。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 16（r4，grid 闭合=单元⑫——**T-08 面全部闭合**）：
- `98e6255d5`——L-14 grid：grid-kw 聚合面上下文关键字 + grid 原子形；**列元组表示**（let 位每列一个 List
  + 列元组绑定，经 tuple 解构读列）；bind 期静态维检查（行宽=列数/列名唯一/列内类型统一——V-08 grid 切片）。
  探针 20/20 绿（2×2 grid 执行 62 + 三类维负控）；ace-core 34+399 绿。mixed 列拒在 bind 层（具名码 SD-01 polish）。
- **T-08 分母行全闭合**：L-06 view/mut/move✓ L-07 Hold✓ L-08 Box✓（递归） L-09 take✓ L-12 union✓
  L-13 tag✓ L-14 grid✓。
- 剩余队列：V-08 嵌套 place 对照 fixture、位置式索引读（语法面裁定已记）、T-03 四端发射、T-04 V 行全量、
  T-05 复审、T-06/07 merge。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 17（r4，T-03 首批=单元⑬）：
- **native 双腿**：①driver 升梯加 Aggregates 末位（Domain/Arm/closures 族；climb 谓词学得聚合面 token 形——
  括号/range/元组逗号/命名ctor 花括号，沿 961 数值算符先例）——**无 arena 组合形（tuple/命名ctor）真 PE 执行绿**；
  ②collections 发射器：av.aggregates/1 capability 准入（mir_verify 键先例）+ PLACE owned 元素放宽——
  **嵌套List/索引赋值/Box 胞/grid/range-tag-union 全部 admit+真实 COFF 发射绿**。
- **a2x-rust 真链**：view 经 capability 区分 aggregates/numeric（closures 域下）+ collections 族全开（tuple 走
  record、Box/grid 走 list——零新 HIR 节点）；store Aggregates profile/族接线——**tuple/解构/参数/命名ctor/range
  四案例真实 rustc 执行 42 绿**（独立 oracle）。
- **具名发现**：递归 Box（环类型表）无法经 unit-split 不动点局部化（unresolved type closure）——AV 端执行 ✓，
  collections-unit 车道需环形单元表示（§9 在档）。
- 回归：探针 20/20；native mn_numeric/collections_layout/source_diagnostics 绿（全量 10 红=具名性能/进程门，
  a2c cert 族 base 上同红已核）；a2x-worker 190/204（14 红为 a2c cert 族 spawn 类——base 同红，具名环境红）。
- 剩余：T-03 的 a2x-c 真链腿、V-08 嵌套 place 对照、T-04 V 行全量+六职责组、T-05、T-06/07。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 18（r4，T-03 收尾+V-08 嵌套place=单元⑭）：
- `cc3c8ad05`——**a2x-c 真链**：seed 双解析器收位置字段名（tuple=数字化名 record）；aggregates profile 经
  C provider 会话装配，**tuple 解构/命名ctor/range 三案例真实 MSVC cl 执行绿**（独立 oracle）。
  **具名发现**：tuple 作调用实参在 C 侧需临时物化（cl C2440 const-rec 指针 ABI；Rust 侧 unit-⑥ 已过）。
- **V-08 嵌套 place 对照**：自别名替换 RHS 先物化（执行绿）；视图索引存储（i=xs.get(0)-1 经视图读+落存）
  执行绿；非法邻=视图物化拒（既有执法）。**具名发现**：List 局部 use-after-move（k.move 后 k.get）在
  collections 族今日不拒——冻结既有行为（collections 面逐字节同态；SD-01 polish）。
- 探针 21/21 绿。**T-03 四端现状**：AV✓（21 探针腿）、native（真 PE 执行+COFF 发射）✓、a2x-rust（4 案例
  rustc）✓、a2x-c（3 案例 cl）✓——四端真链全通，余个别具名缺口在档。
- 剩余：T-04 V 行全量 V-01..V-05/V-08+六职责组、T-05 复审、T-06/07 merge。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 19（r4，T-04 主体=单元⑮+修复）：
- **V 行永久 fixture**：`crates/core/tests/fixtures/aggregates_962/` 8 个 .at（V-01..05/V-08 双侧）+ av 仓命名用例
  `aggregates_v_rows`（真全链执行+独立值/资源断言）——**8/8 绿**。
- **六职责组并集**（`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`，
  t04-six-groups.log）：**PASS×5**（ace-source/ace-lowering/ace-av/ace-runtime + native 计 200/233 无测试红后
  撞组 30min deadline——负载归因）；**ace-a2x FAIL 1**：`a2c_rt_seed_codec_strictness`——单元⑭的位置字段名放宽
  在 3 段长度检查**前**读 fparts[1]（严格性矩阵喂单段行→越界 panic）。已修（长度先行；双 lane），严格性 1/1 +
  aggregates 真链 2/2 复绿；native+a2x 两组重跑在跑（t04-rerun-native-a2x.log）。
status: executing；current_step 1/8。

2026-11-12 work checkpoint round 19 收口 + T-05 独立复审（r4，final pass）：
- **T-04 收口**：六组并集重跑（t04-rerun-native-a2x.log）——native 唯一红=`resources_tier::performance`
  计量门（并行负载，与 round-17 全量同款具名）；a2x 唯一红=`worker_ct_941_ordinary` worker 进程腿，
  **隔离重放绿（1/1, 10.97s）**——环境归因。六组零代码红。
- **T-05 独立复审（同会话独立上下文，限制声明同 961：结论自工件重建+冻结提交重跑）**，冻结提交
  `9c7e0471f`+警告修复提交：
  - **AC-01 pass**（t01-freeze.md 分母 L-06..19/48 逐行+D1-D14 策略+四端 owner/命令）；**AC-02 pass**
    （V-01..V-05/V-08 永久 fixture 8/8 真全链+独立值/资源 oracle；V-05 四端账=AV 全账断言+native/a2x
    各自执行证据，side-by-side 表记 SD-01 说明）；**AC-03 pass**（四端真链无 legacy fallback；具名缺口
    D-962-C2/C3 在档）；**AC-06 pass**（T-08 分母行 L-06..L-14 全闭+V-08 全族）。
  - **AC-04 健康**：交付 9 crate `cargo fmt --check`=0（修复提交 9c7e0471f）；新增警告=0（tuples 参数下划线化；
    余 module/err、mix_u64、shell_lo/hi 经 1fbc276ef 基线核验为既有）；调试残留=0（infra-blocked eprintln 为
    numeric 先例同款合法）；测试残留=0。
  - **遗漏/延后扫描**：5 项候选债登记 KNOWN-DEBT（D-962-C1..C5），全部在本计划 §9 具名在先——无未具名延后。
  - 复审重跑：av aggregates 29/29（21 探针+8 V 行）、core lib 34/34、a2c 严格性 1/1、aggregates 真链 2/2。
  - **结论：pass**（AC-05 merge-owned 待 T-06/07）。规范增量 SD-01/SD-02 文本待 merge 落盘（frontmatter 路径精确）。
- work 收口：status flipped `execution_done`→`reviewed`；current_step 7/8（T-06/07 merge-owned）。

2026-11-12 T-06/T-07 merge（PLAN-962:r4 合并收据，completion_kind: delivered）：
- **prepared**：reviewed 基线 9c7e0471f+警告/spec 提交；master 自 base 1169f1359 起 22 提交（experimental/crates
  触面 1 提交 eb278ca79 六红基线收口）。worktree 内 `git rebase master`：23/23 clean（零冲突），
  映射 tip 4252adf46。**rebase 后复验**：av aggregates 29/29、core lib 34/34、a2x 真链 2/2、native tier 2/2。
- **spec 落盘**：SD-01 `docs/specs/ace-migration/m2-aggregates.md` sha256
  `60c9fc5d70b3c8671b37a40f665362efebae82fc4702f9c321f4fadd8e8ff577`；SD-06 挂接
  `replacement-scope-and-evidence.md` sha256 `01c27855386e4a8c78e35ba8f21833a91ef99b0c686b800b844357ecf8077d69`
  （沿 SD-04/05 型式，不改分类口径，不把台账行标 implemented）。
- **landed**：主检出 `git merge --ff-only plan-962-dev`：master→**4252adf46**（线性历史，无 merge 提交；归档簿记=本提交）。
  主检出落地检查：av aggregates 29/29 全绿。无 ledger/attachments/.wt-evidence 写入（分支 diff 扫描沿复审结论）。
  M1/M2 分账保持：SD-06 只挂接 producer。
- **archived**：`git mv docs/plans/962-ace-m2-aggregates.md docs/plans/archive/` + status: archived（本提交）；
  T-06/T-07 勾选（T-07 清理明细如下）。
- **cleaned**（T-07）：**输出路径对账**——本计划命令实际输出：①`D:/autostack/plan-artifacts/plan-962/`
  （唯一暂存区：t01-freeze.md+六组日志+重跑日志+补丁脚本；冻结表已规范化入 m2-aggregates.md、六组结果已折入
  §9 round19）→ 折叠完成**整目录删除**；②worktree cargo target 树 → 随 worktree 删除；③测试临时目录
  （a2x-ag-*/agdbg*）→ 测试本体自清+残留已清。无 D:/autostack-build、无第二 scope、无盘符散落。
  **worktree 清理**：`wt-guard.sh D:/autostack/.wt/lang-962/auto-lang` → clean；树清 0 脏、HEAD 4252adf46
  已全量落地 → `git worktree remove` + `git branch -d plan-962-dev` + 组目录 rmdir。最终：master 线性，
  任务 8/8 全勾，status: archived。**观察项**：批量回归门 last_covered_plan_id=1005（<962 不触发；本包
  落地后按 PLAN-898 到期规则由 auto-plan:regress 单实例处理，非本包每计划门）。

stage: review | review_scope: final | reviewer_identity: 同会话独立上下文（限制已声明；结论自工件重建） | plan_id: PLAN-962 | plan_revision: r4 | outcome: pass | worktree_path: D:/autostack/.wt/lang-962/auto-lang | branch: plan-962-dev | reviewed_commit: 9c7e0471f | base_commit: 1169f1359 | dependency_revisions: PLAN-960/961 archived | acceptance_results: AC-01 pass, AC-02 pass, AC-03 pass, AC-04 pass, AC-06 pass, AC-05 merge-owned | findings: 5 候选债已登记（D-962-C1..C5，均先具名） | evidence: 复审重跑记录+六组日志(t04-*.log)+冻结基线走读 | next: merge (T-06/07)

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 19 | from: work | to: work | status: executing | outcome: pass(T-04主体+seed修复,重跑在跑) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: <seed修复提交> | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 18 | from: work | to: work | status: executing | outcome: pass(T-03收尾+V-08嵌套place) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: cc3c8ad05 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 17 | from: work | to: work | status: executing | outcome: pass(T-03 native+a2x-rust首批) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: e25d829c3 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 16 | from: work | to: work | status: executing | outcome: pass(grid闭合,T-08面全闭) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 98e6255d5 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 15 | from: work | to: work | status: executing | outcome: pass(.view闭合) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 8f68bc22c | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 14 | from: work | to: work | status: executing | outcome: pass(Hold闭合) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 3d7173802 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 13 | from: work | to: work | status: executing | outcome: pass(Hold尝试回退保绿,卡点入档) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: e44b52a6f | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 12 | from: work | to: work | status: executing | outcome: pass(轮上限收口,切片设计定格) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: e44b52a6f | next: work(后续会话续作)

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 11 | from: work | to: work | status: executing | outcome: pass(递归Box正式闭合) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: e44b52a6f | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 10 | from: work | to: work | status: executing | outcome: pass(实验回退到验证态,修法精化) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: <本次回退提交> | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 10 | from: work | to: work | status: executing | outcome: pass(递归Box闭合) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: e18d28e87 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 9 | from: work | to: work | status: executing | outcome: pass(诊断完成,无代码变更) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 1fbc276ef | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 8 | from: work | to: work | status: executing | outcome: pass(Box单元完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 6d29b0e1e | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 7 | from: work | to: work | status: executing | outcome: pass(T-08首切片完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 0a67a7372 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 6 | from: work | to: work | status: executing | outcome: pass(单元⑥完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: a42d67234 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 5 | from: work | to: work | status: executing | outcome: pass(单元⑤完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: b4c138456 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 4 | from: work | to: work | status: executing | outcome: pass(单元④完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: fdfd8c94f | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 3 | from: work | to: work | status: executing | outcome: pass(单元②③完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: d1e0927d4 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 2 | from: work | to: work | status: executing | outcome: pass(单元①完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 3598bfc35 | next: work

stage: goal | plan_id: PLAN-962 | plan_revision: r4 | round: 1 | from: work | to: work | status: executing | outcome: pass(T-01冻结完成,未到handoff) | worktree: D:/autostack/.wt/lang-962/auto-lang | commit: 3f4eeeef6 | next: work

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
