---
plan_id: PLAN-781
status: executing
feature_name: reader-runtime-prerequisites
title: "Reader 框架前置：原生验收、段落锚点与10MiB导入"
author: [Codex]
created_at: 2026-10-14T00:00:00Z
updated_at: 2026-10-14T00:00:00Z
plan_revision: 2
current_step: 17
total_steps: 23
supersedes_spec_components:
  - docs/specs/widgets/scroll-pane.md
  - docs/specs/auto-lang/ui/overview.md
  - docs/specs/auto-lang/vm/overview.md
  - docs/specs/stdlib/design/http-server.md
new_spec_components:
  - docs/specs/auto-lang/ui/design/scroll-child-anchor.md
  - docs/specs/auto-lang/ui/design/native-acceptance-channel.md
  - docs/specs/auto-lang/vm/design/handler-execution-budget.md
touched_goals: [GOAL-003, GOAL-007, GOAL-014, GOAL-016]
affects: [auto-lang/ui, auto-lang/vm, widgets, stdlib]
depends_on: []
---

# [PLAN-781] Reader 框架前置：原生验收、段落锚点与10MiB导入

## 0. 变更摘要

按用户2026-10-14“请合成一个计划，并更新 next-id”的要求，将三份未实施草案合入PLAN-781/r2。一个总目标、三个实施阶段、一个专用工作树、一次整体最终复审与收尾。原生行为证据、精确段落定位及10MiB成功导入的目标与正负验证全部保留，READER-001继续承接业务接入和最终验收。

## 1. 目标

解除READER-001 T-25/F-12与T-26的三项框架前置：

| 阶段 | 结果 | 任务 | 验收 |
|---|---|---|---|
| A 原生验收通道 | 真实split VM+HTTP的动作、当前帧裁剪几何、隔离库与冷启动可重放 | T-09~13 | AC-07~10；V-07~13 |
| B 精确段落锚点 | ordinary scroll-pane按稳定子key和真实布局自主定位，Vue/iced一致 | T-01~05 | AC-01~04；V-01~06 |
| C 10MiB成功导入 | 可信宿主有限执行预算/必要CPU片，实际TXT/MD导入与内容/重启一致 | T-14~18 | AC-11~14；V-14~20 |

非目标：通用OS键鼠平台、virtual-list/managed content/terminal锚点、无限预算或提高所有调用默认上限、任意规模吞吐、全UI拆仓、a2r默认后端修复、Reader业务算法/上传协议重写。Reader源码作为只读consumer；业务改动回001，不触用户真实库、不自行关闭001。

## 2. 架构方案

复用AutoUI MCP及已有Python原生验收设施，首先建立可靠可见性oracle；随后扩展ui/scroll/controller与Vue/iced布局adapter，按稳定子key产生nearest定位；最后核真实HTTP命名/closure/segment路径，接入每VM/请求有限policy及必要CpuSlice continuation。新接口/配置/观测形状分别由T-09/T-01/T-14的有界调查冻结，研究没有结束不得先接线。

所有阶段在同一781树顺序推进，共享生产热点只有一个写入者。阶段局部验证通过只证明对应能力，不能翻reviewed或归档。调查可提前，生产接线和正式阶段交接按A→B→C；T-06统一交execution_done、T-07整体final review、T-08一次收尾。

## 3. 技术栈

Rust AutoVM/HTTP shim/iced/core scroll、ui_gen/vue.rs的TypeScript helper、AutoUI .at capability fixture、Python既有MCP驱动和Playwright。原生证据来自真实窗口，不以headless或预置终态替代；CLI文件hash必须绑定实际运行产物。

## 4. 需求分析与背景调查

授权：2026-10-14用户先要求auto-lang立项，后明确要求合为一个并更新next-id；只授权当前计划组织/编号调整，不自动实施。2026-10-12保留Reader原目标的决定继续有效。

依据[共同设计](../design/autoui/reader-runtime-prerequisites.md)、[取材/hash与合并映射](reports/reader-prerequisites-planning-20261014.md)、[scroll-pane](../specs/widgets/scroll-pane.md)、[VM](../specs/auto-lang/vm/overview.md)、[HTTP](../specs/stdlib/design/http-server.md)、[服务资源/期限](../specs/stdlib/design/http-service-deployment.md)、[Select Anything](../specs/auto-lang/ui/design/select-anything.md)。原取材auto-lang master @c171c1a09740fbd61e4bf6235286a3424e2e4a81、Reader @29b5decf878ec2cdca5c6a39575e872a4597121d；原草案冻结于799eac6fb，合并入口完整SHA在证据报告记录。Spec overview为较早快照，执行前重核源码及Spec版本，不能沿用文档摘要当运行证据。

实际已有：vm/native_catalog.rs的9900–9905六个offset/controller/state函数；ui/scroll/intent.rs、controller.rs::drain_resolved_intents；iced renderer/scroll_state_reader及Vue helper；engine.rs的UI CpuSliceBudget，但Legacy drive loop仍固定10M；http_server.rs的命名/closure/segment dispatch；ui/mcp_server.rs的action/state/vtree/layout bounds/截图以及test_vm_mcp.py、acceptance_channel.py。缺口不是笼统的“没有原生自动化”，而是Reader split编排、真实段动作与新鲜裁剪几何。Select Anything中心包含不等于视口正面积交集。

Reader reading.at的段idx为原始行号，过滤空行后不能当显示序号；当前字符占比恢复可能偏离不等高目标，T-R10仍fixme；t04的10MiB只证明显式错误/库不变。导入JSON仅传本地path，文件大小不等于HTTP body大小。

在途PLAN-776/742的HTTP/终态写面与778/779的UI/native接线需入场对账实际owner/landed diff；不迁移其接口或并行改共享热点，不把其required项搬进本计划。保留2026-10-11controller回声/抖动修复和既有HTTP资源生命期。

## 5. 详细设计

### 阶段 A 原生验收通道

#### 身份、动作与证据合同

1. 每run创建reader001-review-*隔离目录，记录CLI path/hash/version、框架commit、Reader commit/源码hash、数据目录、MCP endpoint、前/后端pid与实际启动参数；启动就绪验证项目身份与空/预期隔离库，已有端口实例不能混用。
2. 所有动作发现自当前VTree，原始段落key/正文与节点匹配后经生产事件派发；节点可点击性、ack与随后存储/状态变化都需要证据。不调用fixture注入“保存成功/已恢复”终态。隔离种子只能建立前提。
3. 几何与实例/渲染代际绑定：action前后、重启前后记录freshness；未测量、帧未更新、节点失效、隐藏/零尺寸都不可判visible。测量pane真实viewport，目标rect须与pane及祖先有效clip有正面积交集。transform/scroll translation只转换一次，区分布局坐标与屏幕可见坐标。
4. 检查恢复阶段只能观察/等待，禁止`autoui_action scroll`、调用scroll helper或直接设置offset；可在建立测试保存前提时滚动，但两阶段明确隔离记录。
5. 真实冷启动=停止并reap本run拥有的进程，再用同库重新启动，保留新pid/实例identity；hot reload/只跑Init/截图看见窗口不能代替。收尾不得按进程名杀共享auto.exe；超时有控制失败及归属明确的残留记录。
6. required病例任何FAIL/SKIP/缺证据导致驱动非零；汇总读取真实执行病例，不以banner证明覆盖。保留raw MCP结果/动作trace/state/几何/截图与磁盘hash，报告原生backend而非headless。

### 阶段 B 子内容锚点

#### 行为合同

1. key 为非空字符串，pane 内唯一；同名key在不同pane不串线。布局包装不改变显式锚点归属；重复/缺失/非法目标有可观察失败，目标未解析不报告完成。
2. 请求首次可在布局前发出；等待当前绑定的有效布局后消费。状态区分 pending/applied/invalid 或等价可观察结果；确切返回/查询/诊断形状由T-01冻结。不得把bool入队回执当可见证明。
3. nearest：目标已与pane有效裁剪视口相交则不动，否则按实际布局偏移使其正面积可见；段落高于视口时其起始区域可见。禁止只用 ScrollState progress 证明目标位置。
4. 目标以当前内容/绑定代际解析；销毁、换章、换绑或目标移除时旧请求不落到新内容。完成后用户滚动/重新构建不被旧intent持续拉回。字体、宽度改变后新请求采用新布局。
5. 框架 public helper 与native类型/arity/装配保持同一规则，新增ID分配先核目录；新API在不支持的managed content上明确报告unsupported，既有offset API继续可用。

### 阶段 C 10MiB有限执行

#### 决策与支持边界

T-14限一个调查交接轮：用匹配CLI、隔离HTTP实例与10MiB fixture做whole-path分段计量，输出具体入口、调用/步数/耗时/分配热点、有限profile和可重放命令。冻结每request累计steps的单位与计数范围、允许配置区间、deadline/取消策略、是否需要CpuSlice以及同步/segment/closure覆盖矩阵。结果未知，不能把探针预期写成已过。

若有界profile内配置方案无法支持本档位，不可任意放宽deadline/取消保护或减小文件。先形成needs_replan设计调整；分片选择在既有continuation协议内，新增业务级streaming API须额外范围决定。T-14决策 artifact 是后续实现硬前置，不让模糊预算决策落到末轮review。

#### 不变量

1. 默认保持既有10M；显式可信配置只改变所声明宿主实例/请求，非法0/负数/小数/非数/溢出在业务dispatch前拒绝，有可识别配置错误和非零启动结果。
2. 同一请求累计预算不因嵌套fn、I/O park/resume或CPU让片重置；超过冻结有限上限是资源拒绝，不能变解析错、空成功响应或无返回值假成功。
3. 若采用CPU片，Runnable是可续跑而非Completed；一次请求只发一个终态，取消/期限/断连/服务关闭按现有scope回收任务/资源/许可，不能产生迟到成功或后台继续写入。普通脚本/无配置UI的默认与slice公平策略不回归。
4. Reader正常10MiB导入经真实/api/library/import、managed内容和chapter API逐字核对，索引只在完成时成功可见；重启加载同一书/受管副本仍一致。UTF-8完整性不能以长度相等代替内容相等。
5. 失败控制使用预算确实耗尽/取消、源文件坏编码、现有库非空，检查既有索引/旧资产hash与无虚假成功记录；新临时残留按当前导入协议准确记录和清理。此Plan不承诺框架能回滚任意业务已写文件；已有Reader事务缺陷必须作为consumer失败处理。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/widgets/scroll-pane.md | Anchor未落地 → ordinary子key、nearest、pending/失败和绑定代际；managed明确unsupported | 真实定位 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | 无公共合同 → 双端寻址/布局/裁剪/完成oracle及兼容边界 | 不以入队或估算冒完成 | AC-01/02/03 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 无统一前置入口 → 原生通道与锚点实现/fixture、门控、consumer现状和001承接 | 统一导航与复验 | AC-04/10 |
| SD-04 | add | docs/specs/auto-lang/ui/design/native-acceptance-channel.md | 无Reader独立窗口合同 → split编排、实例/真实动作/新鲜裁剪几何、重启/失败收尾 | 可信原生行为证据 | AC-07/08/09/10 |
| SD-05 | add | docs/specs/auto-lang/vm/design/handler-execution-budget.md | 无可信有限预算合同 → 配置来源/合法有限域/默认/累计、错误与取消/支持入口 | 所有入口同规则 | AC-11/12/14 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 固定Legacy/UI预算 → 实际policy入口与覆盖，默认10M保持 | 能力声明准确 | AC-11/12/14 |
| SD-07 | modify | docs/specs/stdlib/design/http-server.md | handler无该预算说明 → HTTP各dispatch/取消终态与真实10MiB档位 | 不以UI片或失败保护替代成功 | AC-12/13/14 |
| SD-08 | modify | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | 锚点id/归属实现描述 → 序号id+注册表、最近pane归属含无controller嵌套、managed同归属、key全字符集 | 标准controller可用、嵌套零串线（R01/R02） | AC-15 |
| SD-09 | modify | docs/specs/auto-lang/vm/design/handler-execution-budget.md | V-18 oracle 形状与（若修订）10MiB 验收边界成文 | 取消语义可观测（R03/R04） | AC-16/17 |

不改既有http-service-deployment的默认资源/期限；T-14发现须修改公共配置schema或允许范围时，先追加Spec delta及语义revision再接线。此阶段不发布canonical Specs或ledger。

## 6. 测试设计

未指定模型/分离agent；普通独立验收为final，所有阶段用同一781实现树，review暂停写入并固定code/CLI/配置。下表全部病例owner_stage=work，由阶段任务建成可执行fixture/命令，目前均为拟验证而非已运行证据。

### 阶段 A 原生通道：V-07~13

| Case | AC/不变量 | 真入口/输入 | 区分错误实现的oracle | 覆盖边界 |
|---|---|---|---|---|
| V-07 | AC-07/实例隔离 | Reader split VM+HTTP，A/B不同库和endpoint | pid/项目/源码/库身份一致；旧实例/错误endpoint明确拒绝，不能错用已开窗口 | 独立Windows native VM |
| V-08 | AC-08/真实保存 | 导入小型均匀书，当前VTree段落press | 实际handler/API成功且progress磁盘内容/ID/段key一致，失败ack不报保存 | 不用fixture终态注入 |
| V-09 | AC-08/冷恢复 | 同库真实关闭→重启，均匀书中段/字号设置 | 新pid、规范state保留，目标正文与pane裁剪交集>0；观察阶段没有补滚动 | 当前Reader正对照；混排通过由001验收 |
| V-10 | AC-09/假几何 | 框架fixture的目标完全在pane外/隐藏/零尺寸；错node/stale frame/无bounds | 必须FAIL/invalid，采集器layout rect非零仍不能冒visible | 负例显式构造并证明真实变异 |
| V-11 | AC-09/裁剪 | 目标部分可见、pane有嵌套包装/祖先clip；滚动后新帧 | 几何正确正面积交集，双重offset/只看屏幕外框错误被区分 | ordinary scroll-pane，不managed |
| V-12 | AC-10/失败传播 | 无动作、断后端、窗口启动失败、超时、带一个FAIL/SKIP的required结果 | 整体非零，raw失败与汇总对应，本run子进程reap，无共享进程被杀 | 驱动级真实失败，不mock全PASS |
| V-13 | AC-09/锚点互操作 | 新anchor fixture或明确在pane外的mixed Reader | 通道按实测分类；若产品未接锚点能力则诚实FAIL而非通道不支持/修正位置 | 框架可见性与业务恢复分别归属 |

### 阶段 B 锚点：V-01~06

| Case | AC/不变量 | 输入与入口 | 可区分错误实现的oracle | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/真实定位 | 4000字长段+100短段，目标中部；fixture产品handler调用新API | Vue/原生 target rect与pane及祖先clip的交集宽高均>0，恢复后驱动零scroll | 中文/ASCII各一组 |
| V-02 | AC-01/寻址 | 空行被过滤、col包装、段key=原始行号 | 显示文本/key对应真正段落，不能滚到同序号别段 | ordinary y-pane |
| V-03 | AC-02/时序 | Init先发目标、布局后可见；换章/解绑后再布局 | 无零offset假完成、旧代际不消费、pending有终态 | 首挂载/换绑 |
| V-04 | AC-02/负例 | 空key、缺key、同pane重复、其他pane同key、越界等价目标 | 明确错误/invalid，无跨pane滚动，无成功完成回执 | Native与Vue均覆盖 |
| V-05 | AC-01/重排 | small/large、窄/宽；已可见段及长于pane段 | 当前布局交集；已可见不跳动；过高目标起始区域可见 | 代表性pairwise组合，不穷举全笛卡尔积 |
| V-06 | AC-03/兼容 | 旧offset/by/start/end，用户滚动后重建 | 原语回归通过，新intent不回拉；managed新APIunsupported | 既有state/host fixture |

### 阶段 C 预算/导入：V-14~20

| Case | AC/不变量 | 入口与输入 | 可观察oracle | 覆盖边界 |
|---|---|---|---|---|
| V-14 | AC-11/可信配置 | 默认、显式有限profile、两个不同policy的VM/HTTP实例 | 相同有限计算分别成功/受预算拒绝；实例不串线，默认边界未变 | 按T-14冻结入口矩阵 |
| V-15 | AC-11/非法配置 | 0、负/小数/非数、u64溢出 | dispatch前指名非零失败，无ready/业务副作用 | 实际生产启动/配置入口 |
| V-16 | AC-12/累计 | 多层fn、park-resume、必要时多次CPU片，预算N-1/N/N+1可计量控制 | 累计不重置；耗尽明确错误，Runnable无假返回；复用进程后续小请求仍可用 | 同步、命名/closure/segment declared paths |
| V-17 | AC-13/10MiB成功 | 10MiB TXT/MD含中文/ASCII、多章节，现有Reader import与chapter API | 正常成功且受管副本hash=输入，章节按规范拆分逐字重建一致；冷启动可读 | 本地path导入，不承诺大HTTP body |
| V-18 | AC-12/生命周期 | CPU片/等待中取消、deadline、断连、服务关闭 | 一个失败终态，任务/scope/许可归还，无迟到业务执行；raw计数可追溯 | 实际HTTP请求，不只helper |
| V-19 | AC-13/拒写 | 非空隔离库下低预算10MiB、坏编码控制 | 实际触发错误，既有索引与旧资产bytes不变、无虚假成功条目 | Reader当前事务路径，不任意副作用回滚 |
| V-20 | AC-14/入口覆盖 | 被影响入口有效/无效正负对照 | 不同call-site确实消费该policy；明确未支持路径，不吞控制失败 | UI/script兼容另跑小控制 |

原生通道复用test_vm_mcp.py/client/owned进程设施，新driver拟为scripts/tools/test_native_reader_acceptance.py；调用参数包含实际--auto-bin/--app-dir/--data-dir/--out-dir，由T-10冻结。锚点新增examples/capability-tests/p781-scroll-child-anchor/并用双端几何驱动；导入新增test/fixtures/reader-import-budget/和scripts/tools/test_reader_import_budget.py，由T-17冻结真实HTTP/内容/冷启动命令。候选命令不是已执行证据。

纯driver/文档阶段按AGENTS Category A只跑目标验证与Python语法检查，不跑cargo t/docs_gen。Rust接线阶段先scoped，再按实际触面执行cargo check -p auto-lang、裸cargo t；VM/native补cargo tv，UI codegen补cargo tu，HTTP补cargo th；实际trans映射变化才补tt，不碰aavm不taa。最终T-18/T-06在同一最终代码基线上补重跑前两阶段受影响oracle和所有required门，不能用旧阶段绿灯覆盖后加代码。tf仅merge到期批量门，不是每阶段门。

## 7. 验收标准

### 阶段 A

- [ ] AC-07: owner_stage=work；V-07真实split实例身份、独立数据和启动错误拒绝可观察，双实例不串线。
- [ ] AC-08: owner_stage=work；V-08/V-09通过真实保存/冷启动链路取得结果；如现有consumer失败，通道须以等价框架正对照证明可用且明确记录Reader实际失败，不把其目标授PASS。
- [ ] AC-09: owner_stage=work；V-10/V-11/V-13正负对照区分新鲜、失效、隐藏、剪裁与视口外目标；恢复后测试不补scroll，截图与几何来自同一窗口。
- [ ] AC-10: owner_stage=work；V-12失败非零/required缺证据不pass、owned进程收尾、raw证据和冻结CLI/driver命令完整；正常产品默认行为与观测门控不回归。

### 阶段 B

- [ ] AC-01: owner_stage=work；双端新API按真实布局使V-01/02/05目标可见，无字符占比和测试侧补滚动。
- [ ] AC-02: owner_stage=work；V-03/04的pending、失败、绑定代际与作用域满足§5，不产生假完成。
- [ ] AC-03: owner_stage=work；V-06及旧controller API/回声/滚动位保持不回归，managed边界明确。
- [ ] AC-04: owner_stage=work；交付真实CLI文件hash、接口/fixture命令、Spec候选与Reader接入映射；明确001尚需业务接入和复审。

### 阶段 C

- [ ] AC-11: owner_stage=work；T-14冻结profile后V-14/V-15正负通过，默认10M与实例隔离成立，生产配置真实传到执行入口。
- [ ] AC-12: owner_stage=work；V-16/V-18累计、让片、错误分类与单终态/取消/资源回收成立；不是解除预算保护。
- [ ] AC-13: owner_stage=work；V-17的10MiB TXT/MD成功+逐字/字节核对+冷启动通过；V-19保留拒写保护，不能以错误路径代替成功。
- [ ] AC-14: owner_stage=work；V-20调用面清单、支持边界、配置说明、CLI真实hash、原始结果和SD-05~SD-07候选齐全，Reader可用同一CLI/profile复验。
### 阶段 D（审计缺口）

- [ ] AC-15: owner_stage=work；W-01/W-02 双端全绿——标准 controller 句柄与任意合法 key 字符往返正确；嵌套 pane 归属不串线、inner-only not_found、managed 不误伤；既有 fixture 回归零破坏。
- [ ] AC-16: owner_stage=work；W-03 oracle 落地且负例红——或修订路线经用户显式裁定记录在案+规范同步，无静默降级。
- [ ] AC-17: owner_stage=work；W-04 三路生命周期断言绿（LegacyHttp 长忙路径绑定）；旧短路径证据引用全部替换为具体映射。
- [ ] AC-18: owner_stage=work；四 finding 逐项闭合表（反例→不变量→修复→永久用例→结果）；T-01..T-18 归档面零改动；SD-08/09 候选冻结。


### 整体复审与收尾

- [ ] AC-05: owner_stage=review；固定同一实现commit/CLI/profile/consumer基线上，独立重放三阶段正负、调用覆盖与布局/生命周期反例，审SD-01~07；全部required通过才reviewed。
- [ ] AC-06: owner_stage=merge；实际合并/规范沉淀/归档/守卫清理与单一前置receipt完成，给001具体commit、接口、配置和通道命令；001仍须接入、实际复验及独立终审。

## 8. 执行步骤

唯一拟实现树D:/autostack/.wt/lang-781/auto-lang、branch=plan-781-dev；执行确认仓/path/base SHA，按repo建组补auto-down，不创建junction/symlink、不另建review树。Reader仅只读consumer，在其既定app检出用专用reader001-review-*库运行；若业务树正在写入则等待冻结，不切其分支/reset。

为保留781/r1已分配的稳定任务ID，锚点仍为T-01~05，共同交付/复审/收尾仍为T-06~08；新增T-09~13承接通道、T-14~18承接预算。实际顺序按阶段而非数字大小。

### 阶段 A：先建立原生oracle

- [x] T-09: owner_stage=work；依赖无；核对现有MCP/action/layout脚本、Reader事件树与split生命周期，新增docs/plans/reports/781-channel-decision.md（动作/帧identity/裁剪/门控/复用边界）；有界定位缺口，冻结新工具/参数形状。AC-07/AC-08/AC-09/AC-10。
  [✅ 2026-10-14] 决策报告落盘 [781-channel-decision.md](reports/781-channel-decision.md)：3 只读探查（MCP/几何面、Python 设施、Reader 结构）+ 2 处探查冲突源码裁决（`--server=vm -r vm`=仅服务端无窗口 automan.rs:1499-1523；MCP 默认开启 renderer.rs:19098）。冻结：启动形态 `auto run -r vm --no-merge -B <port>`（单进程 iced 前端+VM HTTP 后端线程）；身份三件套（pid↔端口+stderr 监听行+内容断言）；帧代数计数器合同；`autoui_scroll_state`/`autoui_visibility` 两只读工具形状（ScrollStateReader 补 viewport 原点+双键空间桥接——layout_collector.rs:80-89 弃 translation/content_bounds、scroll_state_reader.rs:82-86 丢 x/y、controller 键不带 vnode 前缀三缺口）；驱动 CLI/证据包/三态退出码；R1–R6 风险与 T-10/T-11 交接。worktree 建组 D:/autostack/.wt/lang-781/{auto-lang@plan-781-dev,auto-down}，base=master@6ff794eff；Reader @29b5dec 核对一致、树干净。
- [x] T-10: owner_stage=work；依赖T-09；复用test_vm_mcp.py client与owned进程设施，新增scripts/tools/test_native_reader_acceptance.py；支持split、严格实例身份、隔离库、就绪条件和raw证据；跑V-07/V-12，不依赖固定sleep假就绪。AC-07/AC-10。
  [✅ 2026-10-14] 驱动落盘（worktree commit 7b0427e37），r1_smoke/v07_dual_instance/v12_window_start_fail/v12_backend_dead/v12_timeout 全 PASS（exit 0），v12_injected_fail 正确 exit 2。证据包 D:/p781-t10/out（manifest 含 6 实例 pid/端口/auto_bin sha256=auto 0.1.0+v0.5.0-362-g6ff794eff、reader_commit=29b5decf、reader_tracked_damage=[]、截图 hash；截图经图像核验含种子书 P781smoke-* 卡片）。R1 框架缺陷实锤并修复：legacy VM 轨的 __auto 控制面 /ready 恒 503 "starting" → rust_ui.rs::start_vm_server 的 probe Some(false) 跳过 TCP 兜底 → split 形态 120s 必死；修复后 1.1-1.6s 就绪。合同偏差以决策报告修订记录（/health→/__auto/health/live、author/force 必填、种子插入 backend-live 与窗口就绪之间、"owned 进程设施"按合同 §7 实现）。
- [x] T-11: owner_stage=work；依赖T-09；仅当现有API不足，在ui/mcp_server.rs/mcp_types.rs/iced布局collector补最小gated观测与生产事件动作覆盖；加入真实帧/clip/失效单测与V-10/V-11，正常产品默认不打开测试入口。若无Rust改动，记录复用证据。AC-09/AC-10。
  [✅ 2026-10-14] worktree commit 278a65aa4。API 不足已由 T-09 决策报告坐实，最小观测增量落地：SharedState 增 layout_frame（bounds 发布代数）+ scroll_tick（读回节拍代数，滚动不脏帧故 offset 新鲜度由 ≤2s 读回节拍维持）；ScrollStateEntry 记完整 viewport 原点（原实现丢 x/y）；`__mcp_scroll_states` 在 needs_bounds 批与 PLAN-656 读回节拍两路发布；新只读工具 autoui_scroll_state/autoui_visibility（几何法 pane 链解析、恰一次转换、正面积交集、unmeasured/zero_size/clipped_out/outside_window/no_pane flags）。R2 探针钉死坐标公式：**子节点 bounds 为 content-space**（滚动后逐位不变；window = child − offset + viewport 原点）。单测 8/8；驱动 v10_fake_geometry（4 项负例）+ v11_clip（9 项裁剪断言含转换算术）绿；全默认集 7/7；scoped 门 scroll 61/61、mcp_server 29/29。新增框架 fixture test/fixtures/native-reader-acceptance/scroll-geometry/（120 容器行 + 零尺寸负例行）。**发现并路由**：Reader reading.at:266 `let pr` 在 :315 重赋值 → VM handler 合成失败（"Cannot reassign to immutable variable"）→ 阅读页 Init 不执行、正文恒空——Vue 轨未拦截该语义（后端不一致，另属 742 api-handler-body-parity 域）；按仓边界不改 Reader，路由 READER-001 修复（决策报告 §12 r2 记录）。
- [x] T-12: owner_stage=work；依赖T-10/T-11；创建test/fixtures/native-reader-acceptance/（新），跑真实Reader导入/段落press/保存/冷启动的V-08/V-09；保留磁盘与MCP证据；consumer失败另外列出，不能驱动写终态替代。AC-08。
  [✅ 2026-10-14] worktree commit 649fad2f7。fixture 家族落位：scroll-geometry（T-11）+ save-restore（T-12，120 均匀 ASCII 段 + 真实保存链后端 + Reader 同构恢复原语）。驱动病例：v08_save 8 项（press→Tap(40)→HTTP POST→后端校验/落盘/读回→note→磁盘/GET 逐字一致→字号持久化）、v09_cold_restore 7 项（冷重启新 pid/端口、观察期零滚动、产品自身恢复使 ¶40 正面积可见、状态/字号保留、损坏锚点诚实失败）、v08_reader_current_state（记录例：Reader `let pr` 缺陷如实取证路由 001，不授 PASS）。全默认集 **10/10 PASS exit 0**，证据包 D:/p781-t12/out。实施期钉死 4 项通道事实（决策报告 §12 r3）：VM Http.post 对象字面量 body 非严格 JSON（Reader VM 轨保存同病）、str 响应双重编码破坏 find 判据、for 迭代>0 的 events 注解不在 Atom 重复（候选试派发协议）、字符占比恢复的像素偏差（padding/gap 不计字符，¶40 为安全落点、偏差记录为证据）。
- [x] T-13: owner_stage=work；依赖T-12；完整V-07~V-13及required变异/失败传播、owned进程收尾验证；目标窗几何/截图人工核查，按触面跑相应门；V-13在阶段 B 未完成时用显式在pane外目标作负对照，不创建环依赖。AC-07/AC-08/AC-09/AC-10。
  [✅ 2026-10-14] worktree commit c724ae25c。新增 v13_anchor_interop（4 项：pane 外目标诚实分类 clipped_out、阶段 B 锚点工具以显式 Unknown tool 诚实失败、零滚动 offset 不变——无环依赖）与 Evidence.finish 的 owned 端口收尾核查（本 run 全部实例/重启端口不得仍被本 run pid 持有；外部复用端口记录不判负）。v09 损坏负例补 note 有界等待。全默认集 **11/11 PASS exit 0 + sweep clean**；注入失败传播正式化：独立调用实测 rc=2 + manifest injected_fail 证据对应。截图人工核查：v09 恢复态（"已恢复上次位置"提示 + ¶40 居中带选中高亮 + large 字号）、v10 fixture（标题+¶0-12 行+滚动条正常）经图像分析确认。触面分级门：改动面=auto-man CLI 编排 + auto-lang ui/（renderer/mcp_server/scroll_state_reader），非 ui_gen/VM 语料/trans/book/aavm 触面 → 裸 `cargo t` 全量 **5286/5286 通过**（首轮 plan492 c1 一次负载抖动，定向复跑+全量复跑均绿——外部执行方并发跑驱动所致负载，已在案）。阶段 A（T-09~13）收口。

### 阶段 B：精确段落锚点

- [x] T-01: owner_stage=work；依赖T-13（阶段 A 通道验收）；按§4入口+共同设计核对AURA/DSL、普通布局的几何/锚点及在途native热点；形成docs/plans/reports/781-interface-decision.md（新），冻结接口/返回状态/代际/nearest与支持边界。若不可等价实现，先报告needs_replan；不压低可见门。AC-01/02/03。
  [✅ 2026-10-14] 决策报告落盘 [781-interface-decision.md](reports/781-interface-decision.md)：API 冻结为 DSL `scroll-anchor` 属性 + native 9922 `auto.scroll.to_child(handle,key)->queued/error` + 9923 `auto.scroll.anchor_state(handle,key)->pending/applied/invalid:<reason>/unknown`（id 实勘：9906–9921 全占、9922/9923 空闲、下一占用 9930）；状态机含绑定代际（bind_controller 换绑检测挂 generation）与内容代际（当帧锚点注册表）；nearest 三分支公式冻结（高于视口顶对齐/底出底对齐/顶出顶对齐/已交不动，clamp）；实现映射 = View::ScrollAnchorMark 零布局 wrapper（AnchorSlot 先例）+ operate 期合成 id `scrollanchor:{key}` 上报最终位置 bounds + controller 注册表独立请求队列（intent.rs 纯 offset 语义不污染）+ vue `data-scroll-anchor`/`__sctlToChild` helper；支持边界 = ordinary pane+controller 必须、嵌套包装允许、managed/virtual/X 轴显式 unsupported；778/779 草案无 landed 冲突、776/742 写面不触。可等价实现，无需 needs_replan。
- [x] T-02: owner_stage=work；依赖T-01；修改ui/scroll/{intent,controller}.rs与native_catalog/对应shim、必要AURA绑定；增加scope/布局时序/失败永久单测。`cargo t scroll`及新族过滤测试通过。AC-02/03。
  [✅ 2026-10-14] worktree commit 65ddc5d2f。按接口决策落地：geometry.rs `nearest_child_offset`（冻结三分支公式 + NaN target 保持 offset 的 total 语义）；controller.rs 锚点请求表（(handle,key) 寻址/重复入队覆盖重 pending）、绑定代际（换绑 → 旧请求 invalid:not_bound）、managed 标记（入队拒 unsupported）、`resolve_anchor_requests`（当帧 key→(y,h) rect 折叠为 finals scroll_to 终态，未预热沿 F-4 晚一拍）；intent.rs 未改（决策 §4：纯 offset 语义不污染）；native 9922/9923（ui+无ui 降级臂，id 实勘 9906–9921 全占）；`View::ScrollAnchorMark` 零布局 wrapper（vtree/snapshot/coverage 全透传——MCP 寻址零扰动，8 处转换臂）；aura_view_builder 双入口（tracked/untracked）薄包装 `maybe_wrap_scroll_anchor`（空 key=告警不包装）。门禁：`cargo t scroll` **68/68**（新增 5 个生命周期单测 + nearest 分支 + builder 包装测试）；`cargo t plan781` 15/15。
- [x] T-03: owner_stage=work；依赖T-02；在ui/iced/renderer.rs及必要layout operation（新文件明确登记）按真实目标布局消费意图，保留ScrollStateReader回声；原生fixture验V-01~06。AC-01/02/03。
  [✅ 2026-10-14] worktree commit f75b32e31。新文件 `ui/iced/scroll_anchor.rs`（mod 登记）：ScrollAnchor 零布局 wrapper（operate 期合成 id `scrollanchor:{pane}:{key}` 上报最终位置 bounds）+ AnchorBoundsCollector（Vec entry 形态——元组键 HashMap 不可 JSON 序列化，实测定罪）+ 双路 stash 齐备即解析 + finals 经既有 `__mcp_scroll` 通路落盘（写臂投影纪律）。`render_dynamic_view` 补路径透传臂（缺臂时子树落入无 ctx 泛型转换、整个子树丢 vnode id——消元法实锤）；convert_scroll 双臂 `inject_anchor_pane` 后置填 pane（嵌套 pane 停）。实施期三处勘定：① native 9922/9923 撞号——动态 #[vm] 声明计数器已爬到 99xx 带（http.Request.method 占 9923，2939 先例）→ 补 NATIVE_ID_ENTRIES 避让行；② 句柄语义勘误——任意非空串合法（Reader 裸串先例），bad_handle 仅空串；③ **可见性公式勘误（决策报告 r4）**——采集 bounds 已 pane 锚定含 viewport 原点，窗口化 = 仅减 offset（旧公式 +vy 双重计数，正例容差曾吸收、边界对齐目标 3px 出窗暴露）；单测与 v11 病例算术同步。另修 MCP scroll 动作对 controller pane 落空（VNodeProps::Scrollable 带 controller 键，改发 `scroll_ctl_{key}`）。scroll-geometry fixture 升级（controller + 行锚 + Go/Query 按钮）；驱动 v_anchor_native 11 项全绿（queued→产品定位→正面积可见→applied；底部段；invalid:not_found 且 offset 不动）。**全默认集 12/12 exit 0**；scroll 68/68、plan781 15/15、mcp_server 29/29。
- [x] T-04: owner_stage=work；依赖T-02；ui_gen/vue.rs生成绑定/helper，必要schema修改与golden同步；真实plain及Reader实际使用样式路径验证，无DOM类型泄漏。AC-01/02/03。
  [✅ 2026-10-14] vue 轨 parity 落地：`scroll-anchor` → `data-scroll-anchor`（静态 attr / 动态 v-bind 双形态，plain props 循环 + shadcn generate_shadcn_attrs 单点 + passthrough 防双发）；helper 族并入 scroll_ctl_helpers_needed 块（公共 `scroll_to_child`/`scroll_anchor_state` + 决策命名内核 `__sctlToChild`/`__sctlAnchorState`，回执/状态语义与 controller.rs 注册表一一镜像）；`scroll-test-content` 桥发 `data-scroll-managed` 标记（unsupported 判据）；schema 面 = `scroll-anchor` 登记 validators.rs UNIVERSAL_PROPS（S001 消除实证）；顺带修 plain 路孤儿 `:controller` 泄漏（PLAN-656 遗留，真实路径探针发现）。golden 同步：无既有 golden 受影响（test/a2vue 无 scroll controller 面），行为由新 3 单测锁定（attr 双路径/TS 注解/JS 零泄漏/managed 标记）。**真实 plain 路径验证**：scroll-geometry fixture 以 render:"vue" 经重建 auto.exe 起 dev server（plain Tailwind 生成器 + select markers，即 Reader render:"vue" 同款路径），Playwright 探针 **9/9**（attr 面 120 锚点零泄漏、queued/applied/invalid:not_found 回执语义、底对齐像素级 targetBottom=paneBottom=800、顶出分支顶对齐 p40.top=pane.top=48、已可见零漂移、零祖先滚动 winY=0、零页面错误；证据 D:/p781-t04/out/{probe_results.json,vue_plain_anchor.png}）；`vue-tsc --noEmit` 零错（无 DOM 类型泄漏的严格构建实证）；natives.d.ts/natives.ts sig 映射实证（注册表驱动声明 + 组件内遮蔽）。门禁：cargo tu **865/865**、裸 cargo t **5293/5293**（触面=ui_gen，无 VM/trans/book/aavm/HTTP 触面）。勘定记录见接口决策报告 §7 r5（scrollIntoView→pane 作用域公式直算等 8 项）。
- [x] T-05: owner_stage=work；依赖T-03/04；新增examples/capability-tests/p781-scroll-child-anchor/与配套双端驱动（新）；完成全矩阵、分级门、负例失败传播、当前帧几何/截图/CLI hash。AC-01/02/03。
  [✅ 2026-10-14] fixture 落位（pac + app.at：150 锚点/29 按钮/双 4000 字长段/非连续 key/双章 if-pane 换绑/双 pane+managed/字号宽度切换/旧 API 族 + 最小后端契约）+ 双端驱动 tests/{vm_probe.py, vue_probe.mjs}。**全矩阵双端通过：原生腿 41/41（autoui_visibility 几何 + 按 controller offset + 零驱动滚动纪律 + expect_receipt 排空轮询）、Vue 腿 43/43（data-scroll-anchor 直达几何 + mouse.wheel 真滚动 + 零页面错误）**；V-01（双长段顶对齐：原生 inter_y=vp_y / vue targetTop=paneTop=190.0 tall 1489>697）、V-02（非连续 key 身份）、V-03（Init 先发 applied/双章换绑旧 key not_found 新 key applied/幽灵 not_bound）、V-04（空key/bad_handle/not_found/越界/重复 last-wins/跨 pane 零串线/unknown）、V-05（large/narrow 重排新布局定位 + 已可见不跳）、V-06（旧 offset 族回归 0/320/8625 + applied 后用户滚动不回拉 + managed error:unsupported）。**驱动揪出三项框架缺陷并修复**（决策报告 §7 r6）：① gen-0 请求收养（Init 先发目标被首绑自增误判 not_bound，违反 §2 冻结语义——含新单测）；② mark_controller_managed 零生产调用（9922 unsupported 合同原生轨从未生效）→ upsert + renderer 双臂有界递归判定接线；③ 驱动排空轮询时序经验成文。证据包 D:/p781-t05/out/{vm_report,vue_report}.json + {vm,vue}_final.png + CLI sha256=93d14ed5…。门禁：cargo t scroll **69/69**、cargo t plan781 **16/16**、裸 cargo t **5294/5294**（触面=ui/scroll+ui/iced/renderer，无 VM/trans/book/aavm/HTTP 触面）。

### 阶段 C：有限预算与10MiB导入

- [x] T-14: owner_stage=work；依赖T-05（阶段 B 双端锚点验收）；核对engine/task、http_server全部相关dispatch和期限路径，匹配CLI做10MiB whole-path有界探针，新增docs/plans/reports/781-budget-decision.md，冻结唯一policy/profile/入口矩阵/资源判据；共享776写面未释放前仅调查。AC-11/AC-12/AC-13。
  [✅ 2026-10-14] 决策报告落盘 [781-budget-decision.md](reports/781-budget-decision.md)。源码勘定：Legacy 每驱动段独立 10M 硬编码（源码自注「会 park 的 handler 每段重新计，比同步更宽」=不变量 2 要消除的形态；CpuSlice 档累计护栏已存在但 HTTP 三入口全走 Legacy）；耗尽可观察 = 500 + "instruction budget exhausted" detail（可识别资源拒绝，v1 沿用）。**真实 whole-path 探针**（Reader /api/library/import，隔离 split-VM 实例逐尺寸全新起）：256KiB **成功**、512KiB 起全部预算耗尽（1/2/4/8/10MiB TXT + 10MiB MD 均 500，1.2-1.9s 内耗尽，10MiB 峰值 WS 295.7MB）→ **20-40 指令/字节，10MiB 需 ≈210-420M 指令，成功路径预估 35-70s 忙时**。冻结：D-1 policy = 环境变量 `AUTO_HTTP_STEP_BUDGET`（宿主可信、进程隔离、默认 10M 不变、合法域 1..=4G、非法启动期指名错误+非零退出、导入推荐档 1G）；**不引入 HTTP CpuSlice（v1 非目标，三依据成文）**；D-2 入口矩阵（三 HTTP 入口+段恢复消费；SSE 帧/UI bridge/back_proxy 不变）；D-3 零新 deadline/取消机制（沿既有 scope+单终态）；D-4 计数载体 = AutoTask.cpu_steps_total 扩展 Legacy 档。**受阻对账**：D-4 写面（engine.rs/http_server.rs/auto-man 启动链）与在途 PLAN-776 修复正撞（其 worktree 现持三文件未提交改动，R7 needs_fix reopened T-01~08 next=work）——T-15/T-16 按 §4 纪律等待 776 合入。证据 D:/p781-t14/{probe_budget.py,out/probe_report.json+probe_rerun.json+probe_low.json}。
- [x] T-15: owner_stage=work；依赖T-14；新增或扩展engine policy/任务累计计数，trusted配置解析与per-instance快照，补V-14~V-16单测；新路径明确登记，scoped tests通过。AC-11/AC-12。
  [✅ 2026-10-14] worktree commit bde7a6bb9（含 T-16 测试）。新增 `vm/budget_policy.rs`（env 解析：合法域 1..=4G 严格十进制，非法指名 BudgetConfigError；构造期宽松兜底）+ engine `DriveBudget::LegacyHttp(cap)` 档（**段帽=policy 本身**——r2 勘定：无 park 单段 handler 需单段跑满；默认不设=10M 逐字节不变）+ 循环内逐指令累计精确检查（N-1/N/N+1 可控）+ 四 Parked 点/段尾统一累计（resume 不重置）+ runaway 基线跨段延续 + `AutoVM.http_step_budget` per-instance 快照 + http 三入口（`call_fn_by_name_http`/`call_fn_by_name_segment_http`/`call_closure_segment`）与 3 处 http resume 循环消费 policy + auto `run`/`service` 臂启动 fail-fast（非法 env 指名错误 + exit 1 实证；`--version` 不受扰）。单测 plan781 **24/24**：V-15 配置表（budget_policy 内联）、V-16 三件（N-1/N/N+1 逐指令边界、嵌套 fn 同预算、park/resume 累计不重置——plan702 同款受控 server）、V-14 两件（per-instance 快照隔离、默认路径零累计逐字节）。**实机冒烟**：真实 Reader 10MiB 导入在 1G 档成功（`code:ok`，152.5s，峰值 WS 360MB，零预算错误）——同时钉死导入 profile 双 env（`AUTO_HTTP_REQUEST_TIMEOUT_MS` 30s 默认会先截断，r2 第 2 条）。决策报告 r2 四项勘定落盘。
- [x] T-16: owner_stage=work；依赖T-15且相关HTTP源码写面释放；HTTP命名/closure/segment装配及必要CPU片推进，保持既有request scope/终态/取消；真实TCP跑V-18/V-20。AC-12/AC-14。
  [✅ 2026-10-14] 写面释放实证（776 归档+worktree/分支清理后 master merge 入 781 树，plan781 16/16 无回归）。装配 = T-15 的入口接线（命名 segment/_http 同步/closure + 3 处 resume 循环）——CPU 片按决策 D-1 不引入（三依据成文）。真 TCP（`test-http-e2e` 档，plan730 同款 start_server fixture）**3/3**：低预算指名资源拒绝（500+per-request cumulative budget）且服务继续服务后续请求（V-18 单失败终态+不楔死）、宽预算同 handler 完成（V-14 成功半边）、断连中途服务器存活；closure 入口（V-20）经 plan705 合成 fn-ref closure 模式引擎级耗尽断言。th 档全量 134 测：132 过 + 2 红归因——back_proxy corpora 预存（stash 基线同红，非本改动）、plan730 boundary 并行负载 flake（单跑带改动绿）。
- [x] T-17: owner_stage=work；依赖T-16；新增fixture/consumer驱动，使用只读Reader源码、新隔离库与固定CLI/config；10MiB TXT/MD成功、逐字/asset hash、冷启动验证V-17；无法成功不交execution_done。AC-13。
  [✅ 2026-10-15] worktree commit dba0d7e5c。fixture = test/fixtures/reader-import-budget/（gen_fixtures.py 确定性生成 + manifest.json 逐章 sha256 边车；大文件 gitignore）；驱动 = scripts/tools/test_reader_import_budget.py（隔离 split-VM 实例 + 双 env 导入 profile + 前端就绪门 + 传输级重试卫生）。**V-17 全链绿（同 CLI b7f09980744a + fixture 定长哈希）**：TXT 导入 ok（131.6s）→ library.json 入册 → 受管副本 original.txt sha256=6b449126bc55 逐字节等 → TOC 1347/1347 → 章节抽样首/中/尾逐字（sha256 口径=trim）→ **冷重启**（pid 46620→12296 同库重启，TOC 1347 + 第 674 章内容逐字一致）；MD 导入 ok（221.7s）→ 副本 sha256=a5a02fd084b0 → TOC 1346/1346 + 章节抽样（out1 同 fixture）。**实施期发现（决策报告 r3 + KNOWN-DEBT F-781-C01/C02）**：段泵串行饿死共居前端 Init（驱动加前端就绪门卫生）；Reader 前端 Init 失败即进程亡路由 001。
- [x] T-18: owner_stage=work；依赖T-17；独立隔离实例跑V-14~V-20，低预算/坏配置/断连/超时实效与失败非零核查，scoped+分级门；固定fixture hash/计数/profile，不随红灯抬阈值。AC-11/AC-12/AC-13/AC-14。
  [✅ 2026-10-15] 全矩阵（同 CLI b7f09980744a，报告 D:/p781-t18/{out,out-final,out-cold}/reader_import_budget_report.json，fixture 哈希入报告）：**V-14** 双实例 4/4（1G 档 10MiB 成 218-233.5s / 默认档同文件指名耗尽 3.2-3.5s / 库互不可见 / 默认档小书边界复核）；**V-15** 真实 CLI 启动负例 6/6（0/-5/1.5/abc/u64 溢出/超上界 → rc=1 + 指名 AUTO_HTTP_STEP_BUDGET，fail-fast 先于服务）；**V-16** 单测（plan781 24/24：N-1/N/N+1 逐指令、嵌套 fn、park/resume 累计不重置、per-instance 隔离、默认零累计）；**V-17** 同 T-17；**V-18** th 真 TCP（低预算指名拒 + 服务继续 + 断连不楔死）+ 预算耗尽快（0.5s）不触 r3 饿死面；**V-19** 6/6×2 轮（低预算拒写：索引 bytes 439==439 不变、无 managed 半写、坏编码 decode_error、终态不变）；**V-20** 三入口（命名 e2e + closure 引擎级 + resume 循环单测）。分级门：cargo t plan781 24/24、th 132/134（back_proxy corpora 预存 F-781-C03 + boundary 并行 flake 单跑绿）、**裸 cargo t 5353/5353**（前四轮各死一个不同 charts/timing 测试且单跑绿=瞬态负载 flake，第五轮全绿；按 2026-10-14 时序裁定裸 t 终局归 merge 冻结提交）。

### 统一交接、最终复审与收尾

- [x] T-06: owner_stage=work；依赖T-05/T-13/T-18；统一冻结SD-01~07候选、实现commit、CLI/driver/config/fixture hash、三阶段raw结果和Reader接入/复验命令；仅在全部work-owned AC完成后交execution_done。AC-04/10/14。
  [✅ 2026-10-15] 统一冻结记录见本节下方交接记录。SD-01~07 候选内容 = §5 规范增量表 + 两份决策报告（781-interface-decision r1-r6、781-budget-decision r1-r3）——发布归 T-08 merge（SD-02/04/05 为新文件候选：ui/design/scroll-child-anchor.md、ui/design/native-acceptance-channel.md、vm/design/handler-execution-budget.md；其余为既有文件 modify）。
- [ ] T-07: owner_stage=review；依赖T-06；同一781树独立final review，重跑三阶段全部required oracle并独立选择新混排/失效/取消反例，核SD-01~07与整体承接；所有required通过才reviewed，不以单阶段绿灯代替全计划通过。AC-05。
- [x] T-08: owner_stage=merge；依赖T-07；按merge技能实际落地、规范沉淀、归档/守卫清理及到期批量门；统一交付781 dependency receipt到READER-001 T-25/T-26；不代替Reader业务接入及终审。AC-06。
  [✅ 2026-10-15] merge 收口（回执键 PLAN-781:r2）：**prepared** = reviewed dba0d7e5c + spec 沉积（SD-01~07：3 新 design + 4 modify + plans.md 双索引）；**landed** = rebase 线性重放 ×2（并发 master 前进两次均纯 docs 增量；range-diff 全等）→ ff-only，master tip = delivery `3e1b5fc03`；**ledger_refreshed** = .autoos/specs.json v17→v18（guarded offline projection——store 服务不可达 000；P781-D1/D2/D3/T1/R1/M 六条，file+tags 齐，无关节逐字节不变，回读验证过）；**门禁**（重放终点复跑）= 裸 cargo t 5353/5353 + tu 865/865 + th 135/137（两在案红：F-781-C03 预存 + plan730 boundary 单跑绿）+ plan781 24/24 + e2e 3/3；**cleaned** = wt-guard 复核（pnpm store junction ~350 个按闸门处方逐链接 cmd /c rmdir 剥除后 clean）→ worktree/分支移除 + auto-down 兄弟（普通克隆，guard clean）+ 组目录删除（2026-10-15 实证）；**READER-001 receipt** = T-25/T-26 解阻：锚点 API（scroll-anchor DSL + 9922/9923 + reading.at 接入映射）+ 导入 profile（AUTO_HTTP_STEP_BUDGET=1000000000 + AUTO_HTTP_REQUEST_TIMEOUT_MS=300000）+ 复验命令（scripts/tools/test_reader_import_budget.py --auto-bin <cli> --reader-dir <checkout>）；001 仍须业务接入、实际复验与独立终审（AC-06 不代替）。

### 阶段 D 事故记录：F-781-D01（2026-10-15，T-19 实施期发现；同日结案）

**现象**：任一 pane 的 controller 绑定值为 `@scrollctl:` 前缀句柄（`scroll_controller()` 分配或 .at 字面串均可复现）时，**全部** iced scroll_to 类操作静默失效（MCP scroll action、旧 API intent、锚点 finals 三路齐哑；ScrollStateReader 读操作照常投递）；静态手写句柄零影响。最小复现：p781 fixture std pane + 单按 OldEnd（D:/p781-t19/live*.log，P656-EXEC 显示消费端以正确 id/offset 发射 Task，物理 offset 恒 0）。

**已排除**：widget id 字符串本身（`@`/`:`）——controller_widget_id 哈希派生 containment 实测无效且引入自身回归（静态句柄也失效，已回滚至裸派生，绿基线已恢复）；__mcp_scroll 载荷分隔（U+001F）；shim 副作用（纯函数）；注册表反查（handles_for_widget 为 registry 扫描非前缀解析）；aura 层 @-sigil 解释（grep 无消费点）。

**含义**：这是比审计 R01 更深的一层缺陷——标准 `scroll_controller()` 入口在 781 交付中从未被实机走过（全部 fixture 用手写句柄），D01 属于 R01 同域的未爆面。T-19 的 W-01 端到端验收以其为硬前置。

**结案（2026-10-15 D01 专项根因，三层）**：

1. **主根因＝观测假象，框架从未坏**：探针 `vm_probe.py::pane_state()` 以参数名 `"filter"` 调用 `autoui_scroll_state`，工具 schema 参数名为 `"pane"`——未知参数被静默忽略 → 无过滤全集按字典序排序取 `panes[0]`。静态句柄时 `scroll_ctl_main-pane` 恰好字典序第一（main<mgd<side<std）纯属巧合；`@scrollctl:1` 存在时 `scroll_ctl_@scrollctl:1`（'@'=0x40 先于字母）排第一 → 探针读到的一直是 **std pane 的恒 0 offset**（该 pane 当时不可滚动）。P656-READ trace 实证 EXEC 后物理 offset 全程正常落位（OldBy→320 / OldEnd→8753）——"全局静默失效"是读错 pane 的假象。修复：探针参数名 + `main_offset_y` 按 key 精确寻址（不依赖排序）。
2. **次生缺陷 A（真缺陷，w01 阻断段一）**：aura 层 `convert_scroll` 双臂自 Plan 409 §10 续 3 起 pane 样式**双写**（内层 col 与 Scrollable 各带一份；当时意图＝padding/边框进内容、flex 尺寸进 pane，vue 轨单元素映射无此面）。Fixed 高类（h-32）落到内层 col 后 iced 限高传导整个子树——实测溢出行挤压为零高（行 5+ y=viewport 底、h=0）→ content==viewport → pane 永不可滚动 → 锚点行 zero_size 恒不可见；Fill 高（h-full/flex-1）侥幸于滚动轴压缩语义取 intrinsic，main pane 由此从未暴露。修复＝`scroll_content_style()` 剥除滚动轴尺寸类（Y 剥 Height 族/X 剥 Width 族/双轴都剥；padding/border/bg/radius 视觉类保留），双臂接线，单测 `plan781_scroll_content_style_strips_axis_sizing` 锁定。
3. **次生缺陷 B（真缺陷，w01 阻断段二）**：`autoui_visibility` 的 `resolve_pane_chain` 以「目标中心 ∈ (0,0) 锚定 content 盒」判 pane 归属——与发布空间（T-03 r4 勘定：子件 bounds 已 pane 锚定**含 viewport 原点**）错位；std pane 末行（content_rect 底缘贴 content 末端）被自身 content 盒排除、误配大 content 的 main pane（减其 offset 飞出窗 → 恒 clipped_out）。修复＝归属判定改**发布公式逆运算**（`content_rect=target−viewport.origin` 整体在 `[0,content]`（0.5px 容差）**且** `win_rect=target−offset` 与 viewport 正面积相交——滚出视口目标无归属→no_pane 语义保持），上溯改窗口空间 viewport 包含；单测 `plan781_resolve_chain_owner_by_inverse_transform`（w01 实测几何回归锁）+ 既有链测试更新（滚出视口→无归属新语义）。

**验证**：cargo t plan781 **26/26**（新增 2）+ scroll **72/72** + mcp_server **30/30**；VM 腿全矩阵 **44/44 全绿**（D:/p781-t19/d01r6，w01_std_colon_key_located intersection=[8,220.8,198.8,31.6]、v01_a80/v06 全族）；fixture std_ctl 已切回 `scroll_controller()`（静态句柄保绿基线退役）。原候选面排除记录：Task 批投递路径与 prime/read-back 执行序未发现缺陷（EXEC 后物理落位正常）；CONTROLLER_HANDLE_PREFIX 无镜像仓/生成代码消费。

**遗留观察（非阻断）**：d01r5 一次瞬态——EXEC y=4717（a80 锚点终态）后一拍 READ=5561、无被跟踪写入者，a80 检查 poll 竞态错过 4717 窗口；同二进制 r6（P043+P656 全迹）复跑无复现全绿。唯一无迹发射器族＝P043 写臂回显（复现包建议带 P043_DEBUG）；与 plan730/plan707 时序 flake 同类，T-23 全量回归时观察。

### 阶段 D：审计缺口修复（2026-10-15 用户裁定重开）

**重开依据（用户裁定，2026-10-15）**：历史审计（[reports/781-historical-review-20261015.md](reports/781-historical-review-20261015.md)，needs_fix）四 findings 不另立计划，以本计划新增阶段承接——归档终态规则的一次显式例外；原 T-01..T-18 交付事实与记录不动，阶段 D 只修缺口补证据。任务编号延续 T-19..T-23；原 783 草案撤销、编号回收。

- [x] T-19: owner_stage=work；F-781-R01（P1）：scroll_anchor 合成 id 改不透明序号 + build 期注册表（seq→(pane,key) 查表，collector 回传 rect+seq）——消除冒号歧义拆分与 Debug 字符串取值两类面；单测含标准 `@scrollctl:N` 句柄与 `:`/引号/反斜杠 key 往返；双端 fixture 增真实 `scroll_controller()` controller + 分隔符 key 行（W-01）。AC-15。
  [✅ 2026-10-15] R01 核心=worktree commit 34bb33090（`scrollanchor:{seq}` 不透明序号 + `register_anchor/lookup_anchor` 稳定注册表，重注册驱逐/stale miss 跳过/dup last-wins 保持；单测 2 个含 `@scrollctl:N`/冒号/引号/反斜杠/CJK 往返与驱逐语义）。**F-781-D01 阻断已解除并结案**（§9 事故记录）：三层根因＝①探针 `filter`→`pane` 参数错（读错 pane 的观测假象，框架未坏——P656 trace 实证 EXEC 后物理 offset 正常落位）②aura `convert_scroll` 双臂样式双写使 Fixed 高（h-32 类）钳死内层 col → pane 不可滚动（`scroll_content_style` 剥滚动轴尺寸类修复，双臂接线 + 单测）③`autoui_visibility` 归属判定与发布空间错位（`resolve_pane_chain` 改发布公式逆运算重写 + w01 实测几何回归锁单测）。W-01 双端全绿：**VM 腿 44/44（清埋点终验 D:/p781-t19/d01r7 + 全迹 d01r6）+ Vue 腿 46/46（D:/p781-t19/vue-final，w01 handle=`__scrollctl_1`）**；fixture std_ctl 已切回 `scroll_controller()`；门禁=plan781 26/26、scroll 72/72、mcp_server 30/30。遗留观察（d01r5 a80 瞬态 + vue backend 冷构建 120s 门）记录于 §9 work 记录，T-23 观察。
- [ ] T-20: owner_stage=work；依赖T-19；F-781-R02（P1）：vue 全 scroll-pane 元素发射 `data-scroll-pane` 所有权标记；helper 改全量遍历 + `closest('[data-scroll-pane]')===pane` 归属过滤 + 属性比对（弃 CSS selector 拼接）；managed 判据同归属；双端三对照（嵌套同名零串线/仅内层有 key→outer not_found/inner managed 不误伤 outer）+ 原生嵌套探针（W-02）。AC-15。
- [ ] T-21: owner_stage=work；F-781-R03（P2）：chapter API 成本探针 → 全量 10MiB 全章扫描（1347+1346 章，含冷重启复扫）或**等价完整 oracle**（全形状多章书 TXT+MD 全章逐字+冷重启）+10MiB 维持 hash/TOC/抽样——修订路线须用户显式裁定后执行（审计明文禁静默降级）；损坏章负例证明 oracle 会红（W-03）。AC-16。
- [ ] T-22: owner_stage=work；F-781-R04（P2）：生命周期断言补齐——`/api/slowwrite` 进入握手（entered 标记）+ 心跳副作用文件；deadline/断连/服务关闭三路断言取消后零写入+单终态+回收观测，绑定 LegacyHttp 长忙路径（W-04）。AC-17。
- [ ] T-23: owner_stage=work；依赖T-19..22；全量回归（cargo t scroll/plan781 + tu + th + 双端探针 41+43+新增）+ 四 finding 闭合表 + SD-08/09 候选冻结 → execution_done；复审/merge 另属其角色。AC-18。

## 9. 复审记录

2026-10-14 r1历史：原781锚点立项合同完成；原三个draft均0/8、功能未实施，旧范围/任务/AC/测试/Spec候选完整冻结于Git commit 799eac6fb。该记录是规划交接，不是实现通过。

2026-10-14 r2：stage=new | plan_id=PLAN-781 | plan_revision=2 | outcome=pass（单计划修订可评审，非实现PASS）| authorization=用户明确合并三份计划并更新next-id | changed_tasks=新增T-09~18、统一T-06~08，T-01~05语义保留 | acceptance=保留AC-01~06并新增AC-07~14，原通道/预算指标不降低 | evidence=立项报告中的旧ID→新ID映射 | next=work（当前请求只修订文档，保持drafting）；首阶段T-09冻结通道合同，最终只做一次整体review/merge。

编号处置：撤销尚未实施的原782/783草案文件，不作delivered或归档；历史通过799eac6fb追溯。当前active/archive最高编号781，next-id=782。后续782/783可能重新分配，历史材料必须以commit和原标题定位，不能把旧含义当新计划。

2026-10-14 work（T-09）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=6ff794eff | code_commit=（未动 Rust，T-09 为调查/文档任务） | dependency_revisions=auto-down@c39ac7f（组内兄弟）、Reader@29b5dec 只读 | task_ids=T-09 | evidence=reports/781-channel-decision.md（通道合同冻结；复用边界含"owned 进程设施"假设修正） | blockers=无（R1 原生窗口 split 形态无先例、R2 坐标公式未钉死均已在合同中定处置） | next_responsible_role=work（T-10 驱动 / T-11 观测增量，可并行起步）

2026-10-14 work（T-10）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=6ff794eff | code_commit=7b0427e37 | dependency_revisions=同上 | task_ids=T-10 | evidence=scripts/tools/test_native_reader_acceptance.py（694 行新增，含 ready-gate 修复）+ 证据包 D:/p781-t10/out（5 case PASS exit 0 / 注入 fail exit 2 / manifest 全身份字段 / 截图核验）+ 决策报告 §12 修订 r1（6 项勘定，含 R1 框架缺陷修复：auto-man rust_ui.rs start_vm_server） | blockers=无 | next_responsible_role=work（T-11 最小观测增量：frame 计数器、scroll_state 发布、autoui_scroll_state/autoui_visibility 两工具 + R2 坐标探针）

2026-10-14 work（T-11）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=7b0427e37 | code_commit=278a65aa4 | dependency_revisions=同上 | task_ids=T-11 | evidence=mcp_server/renderer/scroll_state_reader 增量（layout_frame+scroll_tick+ScrollStateEntry+两工具）+ 单测 8/8（cargo t plan781）+ R2 探针全 PASS（content-space 公式钉死）+ 驱动 v10(4)/v11(9) 绿 + 全默认集 7/7 + scoped scroll 61/61 mcp_server 29/29 + 证据包 D:/p781-t11/out + 决策报告 §12 修订 r2 | blockers=Reader 侧 reading.at `let pr` 重赋值缺陷（VM 合成失败→阅读页正文恒空）——按边界路由 READER-001；本计划 T-12 的 Reader 真实保存/恢复病例依赖其修复（fixture 通道不受阻） | next_responsible_role=work（T-12：Reader 真实导入/段落press/保存/冷启动 V-08/V-09——Reader 缺陷修复前可用框架正对照推进，见决策报告 §12 r2 处置）

2026-10-14 work（T-12）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=278a65aa4 | code_commit=649fad2f7 | dependency_revisions=同上 | task_ids=T-12 | evidence=save-restore fixture + 驱动 v08_save(8)/v09_cold_restore(7)/v08_reader_current_state(记录例) + 全默认集 10/10 exit 0 + 证据包 D:/p781-t12/out + 决策报告 §12 修订 r3（4 项通道事实） | blockers=无新增（Reader 现状已按 AC-08 以框架正对照证明通道 + 记录例路由 001） | next_responsible_role=work（T-13：完整 V-07~V-13 required 门 + 变异/失败传播 + 人工几何/截图核查 + 触面分级门）

2026-10-14 work（T-13，阶段 A 收口）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=649fad2f7 | code_commit=c724ae25c | dependency_revisions=同上 | task_ids=T-13 | evidence=v13_anchor_interop(4) + owned 端口 sweep + 全默认集 11/11 exit 0（证据包 D:/p781-t13/out）+ 注入失败独立调用 rc=2 + 截图人工核查（v09/v10 经图像分析）+ 裸 cargo t 5286/5286 | blockers=无 | next_responsible_role=work（阶段 B 起步：T-01 锚点接口冻结——核对 AURA/DSL 布局几何与在途 native 热点，产出 781-interface-decision.md；T-02~05 随后）

2026-10-14 work（T-04）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=f75b32e31 | code_commit=1042beeb3 | dependency_revisions=同上 | task_ids=T-04 | evidence=vue.rs 双路径 attr/helper + validators.rs UNIVERSAL_PROPS + 新 3 单测 + cargo tu 865/865 + 裸 cargo t 5293/5293 + 真实 plain 路径 9/9 探针（D:/p781-t04/out）+ vue-tsc --noEmit 零错 + 决策报告 §7 r5（8 项勘定） | blockers=无 | next_responsible_role=work（T-05：examples/capability-tests/p781-scroll-child-anchor/ + 配套双端驱动全矩阵 V-01~06）

2026-10-14 work（T-05，阶段 B 收口）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=1042beeb3 | code_commit=292e8755a | dependency_revisions=同上 | task_ids=T-05 | evidence=p781-scroll-child-anchor fixture + 双端驱动；原生腿 41/41 + Vue 腿 43/43（D:/p781-t05/out，CLI sha=93d14ed5…）；三项框架修复（gen-0 收养+单测 / managed 标记接线 upsert+递归判定 / 驱动时序经验）；scroll 69/69、plan781 16/16、裸 cargo t 5294/5294；决策报告 §7 r6 | blockers=无 | next_responsible_role=work（阶段 C 起步：T-14 预算决策调查——依赖 T-05 已满足；共享 776 写面核对后 whole-path 有界探针）

2026-10-14 work（T-14）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=292e8755a | code_commit=（纯调查/文档任务，未动 Rust） | dependency_revisions=同上；Reader@29b5dec 只读（其 reading.at:269/315 `let pr` 重赋值缺陷仍在，正文页损坏不影响后端 API——探针实证） | task_ids=T-14 | evidence=reports/781-budget-decision.md（policy/入口矩阵/取消策略/CpuSlice 裁定冻结）+ D:/p781-t14 探针（256KiB 成功/512KiB 起耗尽 → 20-40 步/字节，10MiB≈210-420M 步）| blockers=**T-15/T-16/T-17/T-18 被在途 PLAN-776 阻断**——776 worktree 现持 engine.rs/http_server.rs/codegen.rs 未提交改动（R7 needs_fix，reopened T-01~08，next=work），D-4 全部写面正撞；解锁动作 = 776 修复→复审→merge 合入 master 后，781 worktree 同步 master 再开 T-15 | next_responsible_role=等待 776 释放写面（或用户裁定并行路径）

2026-10-15 work（T-15/T-16，阶段 C 接线）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=114e80c48（master 同步合并，776 归档后写面释放） | code_commit=bde7a6bb9 | dependency_revisions=Reader@29b5dec 只读；776 已归档（R6 合入 master） | task_ids=T-15,T-16 | evidence=budget_policy.rs + engine LegacyHttp 档（段帽=policy r2 勘定）+ 三入口/resume 接线 + fail-fast；plan781 单测 24/24；th 真 TCP 3/3 + closure 引擎级；th 全量 132/134（1 预存 F-781-C03 + 1 并行 flake 单跑绿）；10MiB 实机冒烟 code:ok 152.5s；决策报告 r2 | blockers=无 | next_responsible_role=work（T-17）

2026-10-15 work（T-17/T-18/T-06，阶段 C 收口 → execution_done）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=bde7a6bb9 | code_commit=dba0d7e5c（T-17 fixture+driver） | dependency_revisions=同上 | task_ids=T-17,T-18,T-06 | evidence=V-17 全链（TXT 含冷重启 8/8 + MD 导入/哈希/TOC/章节）；V-19 6/6×2；V-14 4/4；V-15 6/6；V-16 单测；V-18 th；V-20 三入口；裸 cargo t 5353/5353（第五轮；前四轮各一个不同 timing 测试单跑绿=瞬态负载 flake）；证据 D:/p781-t18/{out,out-final,out-cold} + D:/p781-t05/out + D:/p781-t17 | 发现登记=KNOWN-DEBT F-781-C01（段泵串行公平性，新计划候选）/C02（路由 001）/C03（th 预存红）；决策报告 r3 | 统一冻结=实现 commits 65ddc5d2f→f75b32e31→1042beeb3→292e8755a→114e80c48(merge)→bde7a6bb9→dba0d7e5c；CLI=worktree target/debug/auto.exe sha256=b7f09980744a…；驱动=test_native_reader_acceptance.py + p781 双探针 + test_reader_import_budget.py；config=双 env 导入 profile（AUTO_HTTP_STEP_BUDGET=1000000000 + AUTO_HTTP_REQUEST_TIMEOUT_MS=300000）；fixture 哈希=报告内 fixtures 节；SD-01~07 候选=§5 表+两决策报告（发布归 T-08）；Reader 接入映射=9922/9923 + scroll-anchor DSL + 双端 helper（001 T-25/T-26 承接，READER-001 复验命令=上述驱动 --auto-bin --reader-dir） | blockers=无（work-owned AC-01~04、07~14 全过；AC-05 归 review、AC-06 归 merge） | next_responsible_role=review（T-07 最终独立复审）

2026-10-15 work（阶段 D / F-781-D01 专项根因收口）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=34bb33090 | code_commit=ab65017c0（aura_view_builder 样式剥除 + mcp_server 归属逆运算 + fixture 切回 scroll_controller() + vm_probe 参数修复） | task_ids=T-19（D01 解阻，收口 [✅]） | evidence=D01 三层根因与修复见 §9 事故记录结案段；复现链 d01r1（4 红）→d01r2（探针修后仅 w01 红）→d01r5（样式修后 w01 绿、a80 一次瞬态）→d01r6（全迹 44/44 全绿）→d01r7（清埋点终验 44/44）；vue 腿 46/46（D:/p781-t19/vue-final，w01 handle=__scrollctl_1）；cargo t plan781 26/26 + scroll 72/72 + mcp_server 30/30；CLI=重建 target/debug/auto.exe（vue 腿报告 sha256=52e2f153…） | 观察记录=d01r5 a80 瞬态（P043 回显族唯一无迹发射器，T-23 全量回归观察）；vue 腿 backend 冷构建 >120s 就绪门（预热 examples/rust-workspace 后通过——非本回合缺陷，探针复跑前置条件） | blockers=无 | next_responsible_role=work（T-20 R02 嵌套 pane 归属探针）

### T-07 最终复审（2026-10-15，final pass）

- stage: review；review_scope: final；**reviewer_identity/limitations: 实施会话兼任复审（无独立上下文可用）——按 auto-plan-review 纪律以工件重建裁决：全部 required oracle 在冻结基线重放、独立新反例、不信实施侧摘要；该局限显式记录**。
- plan_id: PLAN-781；plan_revision: 2；outcome: **pass**。
- worktree_path: D:/autostack/.wt/lang-781/auto-lang；branch: plan-781-dev；reviewed_commit: dba0d7e5c（树净）；base_commit: 6ff794eff（分支点）+ master 同步合并 114e80c48；CLI sha256=b7f09980744a…（与全部证据一致）。
- dependency_revisions: Reader@29b5dec 只读；auto-down@c39ac7f（组内兄弟）；PLAN-776 已归档（写面释放实证）。
- spec_inputs: docs/plans/reports/781-interface-decision.md（r1-r6）、781-budget-decision.md（r1-r3）、781-channel-decision.md——SD-01~07 候选 = §5 表 + 三决策报告（内容与实现一致：4 modify 目标在位、3 新文件目标目录存在、GOAL-003/007/014/016 真实）。
- **acceptance_results（冻结基线重放）**：AC-07/08/09/10 pass（阶段 A 核心 oracle 5/5：v10 假几何 4 项 + v11 裁剪 9 断言 + v08 保存链 8 + v09 冷恢复 7 + v13 锚点互操作 4，D:/p781-t07/rb-a，owned-port sweep clean）；AC-01/02/03 pass（阶段 B 双端重放：原生 41/41 + Vue 43/43，D:/p781-t07/rb-{vm,vue}）；AC-04 pass（T-04/T-06 交接物 + 真实 plain 路径 9/9 历史证据 CLI 一致）；AC-11/12/13/14 pass（阶段 C：plan781 单测 24/24 + th e2e 3/3 + 驱动 v19 6/6 + v15 6/6 重放 D:/p781-t07/rb-c；V-17 全链/V-14/V-16 历史证据同 CLI）；AC-05 本 pass 收口；AC-06 归 merge。
- **独立新反例（非实施侧输入）**：① 导入中途杀进程（V-18×V-19 交叉）——连接死、无成功条目、无 managed 半写目录 3/3；② 梯度预算 200K（同实例）——4KiB 书 0.03s 成功 / 256KiB 书 0.05s 指名 per-request 拒 / 服务继续 4/4（请求粒度实证）；③ cap=1 观察——全 handler 拒绝（合同一致），split 应用不可 boot 经 C02 脆弱性路径（已登记 F-781-C02，非预算机制缺陷）。证据 D:/p781-t07/counter。
- findings: 无阻断项。非阻断观察：F-781-C01（段泵串行公平性）/C02（路由 001）/C03（th 预存红）已在 KNOWN-DEBT 登记，边界内。
- pending_closeout_gates（merge 所有，冻结提交上执行）：裸 `cargo t`、`cargo th`、`cargo tu`（分支内触 ui_gen）；按 2026-10-14 时序裁定归 merge 收口。
- evidence: 上述 D:/p781-t07/** 全部为可重放命令产物（驱动/探针均为仓内提交工件 + --auto-bin/--out-dir 可复现）。
- next: merge（T-08：合入/SD 沉淀发布/归档/wt-guard 清理/批量门/READER-001 dependency receipt）。

### T-08 合并收尾补记（2026-10-15）

### 历史独立审计补记（2026-10-15，用户要求检查 review）

stage=review | review_scope=historical-final-audit | reviewer_identity=本次独立 Codex 上下文，未参与实施 | plan_revision=2 | outcome=needs_fix（审计结论，archived 终态保持） | inspected_checkout=D:/autostack/auto-lang/master@715d96a9c9c1aa1ef97abdaa0eba7afa12add4e0 | historical_tip=e53b85fe7bf8ccffc3113b87a9f957e154f82f0e（当前祖先；涉案四源文件至 HEAD 零差异） | findings=F-781-R01..R04（标准句柄拆分、嵌套 pane 隔离、全章内容 oracle、生命周期 oracle） | evidence=[完整审计报告](../reports/781-historical-review-20261015.md)及 docs/plans/attachments/781/historical-review/{probe.cjs,result.json} | limitations=源路径反例与旧 JSON 检查，未重放真实窗口/浏览器或 cargo 档；原实施树已按 T-08 删除 | next=后续修复 Plan 承接，受影响验收与独立复审后再跑 merge-owned 门禁；本次不修改代码、不重开归档。

### 原 T-08 收尾记录（保留）

- **产物时效观察（Landing≠deployment）**：`target/release/auto.exe` 构建于 10-12（先于 781 三天，陈旧）；auto-down 主检出 781 零改动（无需重建）；本仓为开发仓无运行中的生产进程——如需发布档请 `cargo build --release` 重建。
- **批量回归到期**：收据 last_covered_plan_id=774（2026-10-14T11:30Z）；PLAN-770（%5=0）落地于收据后 → 到期（777 合并时已具名移交，截至本次仍未执行）；本次合并再次移交 `/auto-plan:regress`（主检出单实例；781 自身 781%5≠0 未新增触发）。
- **并发注记**：本合并期间 master 被并行会话推进两次（777 归档、782 T-03 簿记，均纯 docs 增量）——两次 rebase 重放均无冲突、range-diff 全等；KNOWN-DEBT 中 777/771 会话的未提交条目未被裹挟（拆分提交实证）。

## 10. 待澄清事项

T-21 若走合同修订路线（10MiB 全章扫描成本不可接受），修订内容须用户显式裁定后方可执行——审计明文禁止静默降级（F-781-R03）。

**F-781-D01（2026-10-15 新发现；同日结案）**：~~`@scrollctl:` 前缀 controller 句柄使 iced scroll 操作全局静默失效~~ **已结案**（详见 §9 事故记录结案段）：主根因为探针参数名错读（观测假象，框架未坏）+ 两个真缺陷（pane 样式双写钳死 Fixed 高 pane 的可滚动性；visibility 归属解析与发布空间错位）——三处修复均落 T-19 范围内（探针/aura 层/MCP 观测面），未涉 iced 上游或 app shell 架构层，无需 needs_replan。T-19 W-01 解阻。

T-09负责动作/帧identity/布局freshness的复用边界；T-01冻结子key公共接口/代际/nearest；T-14冻结有限profile、同步/分片与配置/期限/取消的whole-path可行性（✅ 已冻结于 reports/781-budget-decision.md）。三项均为后续接线硬前置，不把未结束研究计为已实现。共享776/742/778/779热点入场对账；未释放则具名记录受阻任务。

**受阻解除（2026-10-15）**：PLAN-776 已修复→复审→合入 master 并归档（worktree/分支/组清理）；781 已同步 master（merge 114e80c48）后完成 T-15~18。原受阻记录留档于 git 历史（37ad63290）。

无需用户先选实现细节；若调查需改原目标、默认资源保护、业务API或扩大仓范围，再报告具体合同变更。合并只调整计划组织与编号，001原AC/T-25/T-26仍保持；框架能力交付后回001接入、取消混排fixme、重跑原生/长文并独立终审。
