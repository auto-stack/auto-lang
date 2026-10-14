---
plan_id: PLAN-781
status: executing
feature_name: reader-runtime-prerequisites
title: "Reader 框架前置：原生验收、段落锚点与10MiB导入"
author: [Codex]
created_at: 2026-10-14T00:00:00Z
updated_at: 2026-10-14T00:00:00Z
plan_revision: 2
current_step: 8
total_steps: 18
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
- [ ] T-04: owner_stage=work；依赖T-02；ui_gen/vue.rs生成绑定/helper，必要schema修改与golden同步；真实plain及Reader实际使用样式路径验证，无DOM类型泄漏。AC-01/02/03。
- [ ] T-05: owner_stage=work；依赖T-03/04；新增examples/capability-tests/p781-scroll-child-anchor/与配套双端驱动（新）；完成全矩阵、分级门、负例失败传播、当前帧几何/截图/CLI hash。AC-01/02/03。

### 阶段 C：有限预算与10MiB导入

- [ ] T-14: owner_stage=work；依赖T-05（阶段 B 双端锚点验收）；核对engine/task、http_server全部相关dispatch和期限路径，匹配CLI做10MiB whole-path有界探针，新增docs/plans/reports/781-budget-decision.md，冻结唯一policy/profile/入口矩阵/资源判据；共享776写面未释放前仅调查。AC-11/AC-12/AC-13。
- [ ] T-15: owner_stage=work；依赖T-14；新增或扩展engine policy/任务累计计数，trusted配置解析与per-instance快照，补V-14~V-16单测；新路径明确登记，scoped tests通过。AC-11/AC-12。
- [ ] T-16: owner_stage=work；依赖T-15且相关HTTP源码写面释放；HTTP命名/closure/segment装配及必要CPU片推进，保持既有request scope/终态/取消；真实TCP跑V-18/V-20。AC-12/AC-14。
- [ ] T-17: owner_stage=work；依赖T-16；新增fixture/consumer驱动，使用只读Reader源码、新隔离库与固定CLI/config；10MiB TXT/MD成功、逐字/asset hash、冷启动验证V-17；无法成功不交execution_done。AC-13。
- [ ] T-18: owner_stage=work；依赖T-17；独立隔离实例跑V-14~V-20，低预算/坏配置/断连/超时实效与失败非零核查，scoped+分级门；固定fixture hash/计数/profile，不随红灯抬阈值。AC-11/AC-12/AC-13/AC-14。

### 统一交接、最终复审与收尾

- [ ] T-06: owner_stage=work；依赖T-05/T-13/T-18；统一冻结SD-01~07候选、实现commit、CLI/driver/config/fixture hash、三阶段raw结果和Reader接入/复验命令；仅在全部work-owned AC完成后交execution_done。AC-04/10/14。
- [ ] T-07: owner_stage=review；依赖T-06；同一781树独立final review，重跑三阶段全部required oracle并独立选择新混排/失效/取消反例，核SD-01~07与整体承接；所有required通过才reviewed，不以单阶段绿灯代替全计划通过。AC-05。
- [ ] T-08: owner_stage=merge；依赖T-07；按merge技能实际落地、规范沉淀、归档/守卫清理及到期批量门；统一交付781 dependency receipt到READER-001 T-25/T-26；不代替Reader业务接入及终审。AC-06。

## 9. 复审记录

2026-10-14 r1历史：原781锚点立项合同完成；原三个draft均0/8、功能未实施，旧范围/任务/AC/测试/Spec候选完整冻结于Git commit 799eac6fb。该记录是规划交接，不是实现通过。

2026-10-14 r2：stage=new | plan_id=PLAN-781 | plan_revision=2 | outcome=pass（单计划修订可评审，非实现PASS）| authorization=用户明确合并三份计划并更新next-id | changed_tasks=新增T-09~18、统一T-06~08，T-01~05语义保留 | acceptance=保留AC-01~06并新增AC-07~14，原通道/预算指标不降低 | evidence=立项报告中的旧ID→新ID映射 | next=work（当前请求只修订文档，保持drafting）；首阶段T-09冻结通道合同，最终只做一次整体review/merge。

编号处置：撤销尚未实施的原782/783草案文件，不作delivered或归档；历史通过799eac6fb追溯。当前active/archive最高编号781，next-id=782。后续782/783可能重新分配，历史材料必须以commit和原标题定位，不能把旧含义当新计划。

2026-10-14 work（T-09）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=6ff794eff | code_commit=（未动 Rust，T-09 为调查/文档任务） | dependency_revisions=auto-down@c39ac7f（组内兄弟）、Reader@29b5dec 只读 | task_ids=T-09 | evidence=reports/781-channel-decision.md（通道合同冻结；复用边界含"owned 进程设施"假设修正） | blockers=无（R1 原生窗口 split 形态无先例、R2 坐标公式未钉死均已在合同中定处置） | next_responsible_role=work（T-10 驱动 / T-11 观测增量，可并行起步）

2026-10-14 work（T-10）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=6ff794eff | code_commit=7b0427e37 | dependency_revisions=同上 | task_ids=T-10 | evidence=scripts/tools/test_native_reader_acceptance.py（694 行新增，含 ready-gate 修复）+ 证据包 D:/p781-t10/out（5 case PASS exit 0 / 注入 fail exit 2 / manifest 全身份字段 / 截图核验）+ 决策报告 §12 修订 r1（6 项勘定，含 R1 框架缺陷修复：auto-man rust_ui.rs start_vm_server） | blockers=无 | next_responsible_role=work（T-11 最小观测增量：frame 计数器、scroll_state 发布、autoui_scroll_state/autoui_visibility 两工具 + R2 坐标探针）

2026-10-14 work（T-11）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=7b0427e37 | code_commit=278a65aa4 | dependency_revisions=同上 | task_ids=T-11 | evidence=mcp_server/renderer/scroll_state_reader 增量（layout_frame+scroll_tick+ScrollStateEntry+两工具）+ 单测 8/8（cargo t plan781）+ R2 探针全 PASS（content-space 公式钉死）+ 驱动 v10(4)/v11(9) 绿 + 全默认集 7/7 + scoped scroll 61/61 mcp_server 29/29 + 证据包 D:/p781-t11/out + 决策报告 §12 修订 r2 | blockers=Reader 侧 reading.at `let pr` 重赋值缺陷（VM 合成失败→阅读页正文恒空）——按边界路由 READER-001；本计划 T-12 的 Reader 真实保存/恢复病例依赖其修复（fixture 通道不受阻） | next_responsible_role=work（T-12：Reader 真实导入/段落press/保存/冷启动 V-08/V-09——Reader 缺陷修复前可用框架正对照推进，见决策报告 §12 r2 处置）

2026-10-14 work（T-12）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=278a65aa4 | code_commit=649fad2f7 | dependency_revisions=同上 | task_ids=T-12 | evidence=save-restore fixture + 驱动 v08_save(8)/v09_cold_restore(7)/v08_reader_current_state(记录例) + 全默认集 10/10 exit 0 + 证据包 D:/p781-t12/out + 决策报告 §12 修订 r3（4 项通道事实） | blockers=无新增（Reader 现状已按 AC-08 以框架正对照证明通道 + 记录例路由 001） | next_responsible_role=work（T-13：完整 V-07~V-13 required 门 + 变异/失败传播 + 人工几何/截图核查 + 触面分级门）

2026-10-14 work（T-13，阶段 A 收口）：stage=work | plan_id=PLAN-781 | plan_revision=2 | worktree_path=D:/autostack/.wt/lang-781/auto-lang | branch=plan-781-dev | base_commit=649fad2f7 | code_commit=c724ae25c | dependency_revisions=同上 | task_ids=T-13 | evidence=v13_anchor_interop(4) + owned 端口 sweep + 全默认集 11/11 exit 0（证据包 D:/p781-t13/out）+ 注入失败独立调用 rc=2 + 截图人工核查（v09/v10 经图像分析）+ 裸 cargo t 5286/5286 | blockers=无 | next_responsible_role=work（阶段 B 起步：T-01 锚点接口冻结——核对 AURA/DSL 布局几何与在途 native 热点，产出 781-interface-decision.md；T-02~05 随后）

## 10. 待澄清事项

T-09负责动作/帧identity/布局freshness的复用边界；T-01冻结子key公共接口/代际/nearest；T-14冻结有限profile、同步/分片与配置/期限/取消的whole-path可行性。三项均为后续接线硬前置，不把未结束研究计为已实现。共享776/742/778/779热点入场对账；未释放则具名记录受阻任务。

无需用户先选实现细节；若调查需改原目标、默认资源保护、业务API或扩大仓范围，再报告具体合同变更。合并只调整计划组织与编号，001原AC/T-25/T-26仍保持；框架能力交付后回001接入、取消混排fixme、重跑原生/长文并独立终审。
