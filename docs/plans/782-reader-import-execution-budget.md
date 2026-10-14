---
plan_id: PLAN-782
status: drafting
feature_name: reader-import-execution-budget
title: "有限执行预算与 Reader 10MiB 真实导入"
author: [Codex]
created_at: 2026-10-14T00:00:00Z
updated_at: 2026-10-14T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components:
  - docs/specs/auto-lang/vm/overview.md
  - docs/specs/stdlib/design/http-server.md
new_spec_components:
  - docs/specs/auto-lang/vm/design/handler-execution-budget.md
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-lang/vm, stdlib]
depends_on: []
---

# [PLAN-782] 有限执行预算与 Reader 10MiB 真实导入

## 0. 变更摘要

承接 READER-001 T-26 的10MiB成功导入目标。先查真实 VM HTTP 路径，再提供可信宿主的有限累计执行预算，必要时接可续跑片；默认runaway保护、请求期限与失败传播继续成立。成功导入要证明原文/受管副本/章节与重启一致，不能以显式预算错误替代。

## 1. 目标

支持真实UTF-8 TXT及Markdown、本地文件路径、10×1024×1024字节含中文/ASCII/多章节的导入档位；Reader现有业务源码可用新框架配置在隔离数据目录完成请求，至少再运行一次冷启动一致性检查。

非目标：无限预算、提高所有调用默认上限、通用上传/流式导入业务协议、a2r默认后端修复、任意规模吞吐承诺、Reader业务算法重写。若业务改造不可避免，记录consumer差距并回001或修订范围，不把未成功交付称前置已完成。

## 2. 架构方案

复用engine.rs::DriveBudget/CpuSliceBudget与任务累计步数，新增等价的可信宿主execution policy。优先使真实HTTP命名/闭包handler消费每VM/请求冻结的有限policy；若现有同步驱动妨碍取消/超时，复用segment/continuation分片推进。T-01冻结唯一方案与配置入口；不先虚构新native或单改UI常量。

拟宿主配置名AUTO_VM_HANDLER_MAX_STEPS（未实现，T-01可冻结等价配置）；缺省legacy 10M、正有限u64，0不代表无限。配置不得来自普通业务payload，不跨实例污染。普通脚本和UI CPU默认不被HTTP配置意外覆盖。

## 3. 技术栈

Rust AutoVM/HTTP shim、现有宿主配置装配、.at与真实TCP测试；consumer为只读Reader .at后端。测试使用专用CLI产物，不能把旧共享auto.exe的行为算新实现证据。

## 4. 需求分析与背景调查

授权：2026-10-14用户要求auto-lang立项；2026-10-12明确保留001原目标。当前阶段设计/合同，不实施、不抬验收阈值，不修改用户库或Reader代码。

见 [共同设计](../design/autoui/reader-runtime-prerequisites.md)、[立项证据/hash](reports/reader-prerequisites-planning-20261014.md)。Specs为 [VM](../specs/auto-lang/vm/overview.md)、[HTTP](../specs/stdlib/design/http-server.md)、[服务期限/资源](../specs/stdlib/design/http-service-deployment.md)。Spec overview是旧快照；实际取材master @c171c1a09与Reader @29b5decf878ec2cdca5c6a39575e872a4597121d，执行重核。

源码事实：engine.rs::CPU_CUMULATIVE_STEP_BUDGET=10M供UI片；DriveBudget::Legacy在drive loop仍写10M，段驱动不自动享有UI片。http_server.rs包含call_fn_by_name、call_fn_by_name_segment、call_closure_segment；全部覆盖/明确分类，不能只改一条。Reader t04_http_verify.py的10MiB仅证明失败保护，尚未验证成功。请求传本地path，其JSON远小于10MiB，HTTP body预算不是文件导入预算。

PLAN-776正在修HTTP返回资源与终态，PLAN-742相关body合同也在途。实施接线前核对其landed commit/owner与当前call-site，串行修改共享HTTP/engine热点，保留原终态/资源组语义；不把其required项搬进本计划。Design37/779的host-native边界也需对账。

## 5. 详细设计

### 决策与支持边界

T-01限一个调查交接轮：用匹配CLI、隔离HTTP实例与10MiB fixture做whole-path分段计量，输出具体入口、调用/步数/耗时/分配热点、有限profile和可重放命令。冻结每request累计steps的单位与计数范围、允许配置区间、deadline/取消策略、是否需要CpuSlice以及同步/segment/closure覆盖矩阵。结果未知，不能把探针预期写成已过。

若有界profile内配置方案无法支持本档位，不可任意放宽deadline/取消保护或减小文件。先形成needs_replan设计调整；分片选择在既有continuation协议内，新增业务级streaming API须额外范围决定。T-01决策 artifact 是后续实现硬前置，不让模糊预算决策落到末轮review。

### 不变量

1. 默认保持既有10M；显式可信配置只改变所声明宿主实例/请求，非法0/负数/小数/非数/溢出在业务dispatch前拒绝，有可识别配置错误和非零启动结果。
2. 同一请求累计预算不因嵌套fn、I/O park/resume或CPU让片重置；超过冻结有限上限是资源拒绝，不能变解析错、空成功响应或无返回值假成功。
3. 若采用CPU片，Runnable是可续跑而非Completed；一次请求只发一个终态，取消/期限/断连/服务关闭按现有scope回收任务/资源/许可，不能产生迟到成功或后台继续写入。普通脚本/无配置UI的默认与slice公平策略不回归。
4. Reader正常10MiB导入经真实/api/library/import、managed内容和chapter API逐字核对，索引只在完成时成功可见；重启加载同一书/受管副本仍一致。UTF-8完整性不能以长度相等代替内容相等。
5. 失败控制使用预算确实耗尽/取消、源文件坏编码、现有库非空，检查既有索引/旧资产hash与无虚假成功记录；新临时残留按当前导入协议准确记录和清理。此Plan不承诺框架能回滚任意业务已写文件；已有Reader事务缺陷必须作为consumer失败处理。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/vm/design/handler-execution-budget.md | 无可信可配合同 → 配置来源/有限域/默认/累计与错误/取消/支持入口 | 各消费者同规则 | AC-01/02/04 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 固定Legacy/UI预算 → 实际policy入口与覆盖，默认10M保留 | 当前能力准确 | AC-01/02/04 |
| SD-03 | modify | docs/specs/stdlib/design/http-server.md | handler无该宿主预算说明 → HTTP命名/闭包/段路径、取消终态、10MiB真实consumer档位 | 不把UI片或错误保护当成功 | AC-02/03/04 |

不改既有http-service-deployment的默认资源/期限；如T-01发现要改公共配置schema或允许范围，须明确追加对应Spec delta与语义revision后再接线。

## 6. 测试设计

角色：未指定agent/model；普通独立review为final，同一计划树固定commit；所有V项owner_stage=work，T-01/04/05产生可执行输入和oracle。本轮没有跑新功能测试。

| Case | AC/不变量 | 入口与输入 | 可观察oracle | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/可信配置 | 默认、显式有限profile、两个不同policy的VM/HTTP实例 | 相同有限计算分别成功/受预算拒绝；实例不串线，默认边界未变 | 按T-01冻结入口矩阵 |
| V-02 | AC-01/非法配置 | 0、负/小数/非数、u64溢出 | dispatch前指名非零失败，无ready/业务副作用 | 实际生产启动/配置入口 |
| V-03 | AC-02/累计 | 多层fn、park-resume、必要时多次CPU片，预算N-1/N/N+1可计量控制 | 累计不重置；耗尽明确错误，Runnable无假返回；复用进程后续小请求仍可用 | 同步、命名/closure/segment declared paths |
| V-04 | AC-03/10MiB成功 | 10MiB TXT/MD含中文/ASCII、多章节，现有Reader import与chapter API | 正常成功且受管副本hash=输入，章节按规范拆分逐字重建一致；冷启动可读 | 本地path导入，不承诺大HTTP body |
| V-05 | AC-02/生命周期 | CPU片/等待中取消、deadline、断连、服务关闭 | 一个失败终态，任务/scope/许可归还，无迟到业务执行；raw计数可追溯 | 实际HTTP请求，不只helper |
| V-06 | AC-03/拒写 | 非空隔离库下低预算10MiB、坏编码控制 | 实际触发错误，既有索引与旧资产bytes不变、无虚假成功条目 | Reader当前事务路径，不任意副作用回滚 |
| V-07 | AC-04/入口覆盖 | 被影响入口有效/无效正负对照 | 不同call-site确实消费该policy；明确未支持路径，不吞控制失败 | UI/script兼容另跑小控制 |

worktree scoped验证后执行`cargo check -p auto-lang`、裸`cargo t`、`cargo tv`；触HTTP运行`cargo th`，环境失败要分离并保留required实际TCPoracle，不能跳过后授pass。真实Reader驱动拟新test/fixtures/reader-import-budget/与scripts/tools/test_reader_import_budget.py，T-04落地确切命令。按实际代码触面补tu/tt，不碰aavm不taa；tf仅merge到期批量门。

## 7. 验收标准

- [ ] AC-01: owner_stage=work；T-01冻结profile后V-01/02正负通过，默认10M与实例隔离成立，生产配置真实传到执行入口。
- [ ] AC-02: owner_stage=work；V-03/05累计、让片、错误分类与单终态/取消/资源回收成立；不是解除预算保护。
- [ ] AC-03: owner_stage=work；V-04的10MiB TXT/MD成功+逐字/字节核对+冷启动通过；V-06保留拒写保护，不能以错误路径代替成功。
- [ ] AC-04: owner_stage=work；V-07调用面清单、支持边界、配置说明、CLI真实hash、原始结果和SD-01~03候选齐全，Reader可用同一CLI/profile复验。
- [ ] AC-05: owner_stage=review；已提交代码/依赖/配置和冻结profile下独立重验，随机新内容与取消反例及Spec delta通过才reviewed。
- [ ] AC-06: owner_stage=merge；实际合并/规范沉淀/归档/清理与前置交付收据，给001具体commit/profile；不授001全计划pass。

## 8. 执行步骤

拟树D:/autostack/.wt/lang-782/auto-lang，branch=plan-782-dev；按repo建组并确认base/path/branch。Reader consumer只读，运行位使用其仓既定app检出；不自动复制/改业务源码。

- [ ] T-01: owner_stage=work；依赖无；核对engine/task、http_server全部相关dispatch和期限路径，匹配CLI做10MiB whole-path有界探针，新增docs/plans/reports/782-budget-decision.md，冻结唯一policy/profile/入口矩阵/资源判据；共享776写面未释放前仅调查。AC-01/02/03。
- [ ] T-02: owner_stage=work；依赖T-01；新增或扩展engine policy/任务累计计数，trusted配置解析与per-instance快照，补V-01~03单测；新路径明确登记，scoped tests通过。AC-01/02。
- [ ] T-03: owner_stage=work；依赖T-02且相关HTTP源码写面释放；HTTP命名/closure/segment装配及必要CPU片推进，保持既有request scope/终态/取消；真实TCP跑V-05/07。AC-02/04。
- [ ] T-04: owner_stage=work；依赖T-03；新增fixture/consumer驱动，使用只读Reader源码、新隔离库与固定CLI/config；10MiB TXT/MD成功、逐字/asset hash、冷启动验证V-04；无法成功不交execution_done。AC-03。
- [ ] T-05: owner_stage=work；依赖T-04；独立隔离实例跑V-01~07，低预算/坏配置/断连/超时实效与失败非零核查，scoped+分级门；固定fixture hash/计数/profile，不随红灯抬阈值。AC-01/02/03/04。
- [ ] T-06: owner_stage=work；依赖T-05；冻结Spec候选、code/CLI/config与原始结果，记录支持路径与001配置接入命令；work交execution_done。AC-04。
- [ ] T-07: owner_stage=review；依赖T-06；独立final复审调用面、10MiB新内容、累计/取消及SD-01~03；全部required通过才reviewed。AC-05。
- [ ] T-08: owner_stage=merge；依赖T-07；按merge落地、Specs蒸馏、归档/守卫清理与到期批量门，前置receipt回到001 T-26。AC-06。

## 9. 复审记录

2026-10-14：stage=new | plan_id=PLAN-782 | plan_revision=1 | outcome=pass（立项合同可评审；运行可行性留T-01硬前置，非实现PASS）| baseline/inputs=立项证据 | changed_tasks=T-01~08 | acceptance=AC-01~06 | next=用户确认执行范围后work，从有界whole-path探针开始；保持drafting，0/8。

## 10. 待澄清事项

尚未冻结数值profile、同步/分片选择与配置装配形式；owner=T-01，决策artifact完成后才T-02。若需要改变默认资源保护、业务协议或文件档位，先needs_replan并提出具体改变；不能默许。当前不需要用户指定实现细节。共享776/742热点owner须实际入场对账，未释放阻断对应接线而非伪造框架已可用。
