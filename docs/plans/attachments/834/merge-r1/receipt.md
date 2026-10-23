# PLAN-834:r1 merge receipt — delivery landed, cleanup pending

2026-10-23，独立final R1受审`ebeab12d2888b9c9fb7e00e3c24479fb3c028011`；实现`0ea8bdb1582d05e0f39c6e5bfc6d349f9280473b`；交付`19adeebf1b1031554c71acabaa219731b3a28b96`。后继仅canonical/派生账本文档，实际源码/SDK/锁/资产/测试配置与受审版本一致。源已无force快进`autographics-dev`；主检出保持master，仅接收知识/计划簿记，其他会话预存WIP保留。

## Gate results

| Gate | Actual result |
|---|---|
| driver full test/build | 42 passed /0 failed /0 ignored；build pass |
| AGP full test/build | 162 passed /0 failed /0 ignored；build pass |
| standalone C full test | 42 passed /0 failed /0 ignored；前后680 SDK/180 authored hash verify pass；原隔离冷构建证据身份不变 |
| AGP compatibility | 16/16 named probes pass，含全部六surface、真实Windows Chinese IME、original Breakout/Squash、资源/寿命/像素/计量负控 |
| source relocation | fresh exported AGP locked release builds；A/B 29 deterministic fields /0 mismatches；closure/collateral pass |
| Iced actual composite | counter与content三时刻正控pass；blank_blit exit1/counter+content red；transparent_blit exit1/counter仍绿但content red，未将两门互替 |
| native Breakout | movement/text-focus positive pass；wrong-focus same oracle expected exit1；160frames/161ticks、6 sound requests=6 actual Kira plays |
| native Squash | isolated exact13-key tape positive phase9/pass/errors空/music1；focus sabotage phase9/expected exit1，实际text-focus keys/session变化；截图已视觉核验 |
| native scene/Squash no input | 各30frames/pass，Squash music1，无预算拒绝 |
| health/source | frozen source verify与dirty/diff check通过；两workspace fmt/all-target check引用独立review相同源码/依赖/配置证据；无本次新增源码warning，既有SDK146+1 warning不冒称已修 |

原锁屏和一次额外按键现场保留，不隐藏失败。锁屏时Iced点击两次有界超时，前台锁屏/SetForegroundWindow=False；用户解锁后同一源码正式门通过。锁屏下160帧原生Breakout预算拒绝，解锁后相同配置160帧/161ticks/6实际音效正常、errors空。第一次解锁Squash输入记录15个事件，包含额外Space press/release，不符合13事件fixture，retry断言红；其他探针结束、Space/Enter状态未按下的独立条件中，原正式oracle的13事件正控与focus负控通过。额外事件外部来源未断言；没有修改实现/判定/容差或把失败当pass。详见receipt-locked-blocked.md、各原始window.json/native_keys和gates-summary。

根cargo t/tv/tt/tb/tu/taa/docs_gen/tf不适用本Plan约定experimental-only写面，未执行。

## Publication checkpoints

prepared：revision1、R1 PASS、frozen Spec delta/current canonical inputs和SDK/锁身份已核。受审实现diff为零，文档后继只把candidate改为实验delivered、回写实际测量/模块/goals/index及派生条目。spec-index从main-current规范快照加本次overlay再生；58项目，未分组项目warning是既有索引分组边界，未借机改无关模块。

landed：autographics-dev由540c1e70f0a4682af245191ffb36fa0d8a636665快进至19adeebf1b1031554c71acabaa219731b3a28b96；实际integration tip相等。因该集成分支未挂载且main有其他会话文档WIP，采用`git push . HEAD:refs/heads/autographics-dev`（无force，仅允许FF）并前后验证祖先/目标SHA，不切换main、不产生merge commit、不重写受审SHAs。

ledger_refreshed：Regime B，在线127.0.0.1:8080无可达store writer，guarded offline projection；原字节备份ledger-before.json.txt，v70→v71，8项upsert（5 designs/1 tests/1 reviews/1 reports），1138个无关原始item字节保留，six-section schema/readback/source hash/file/tags全部验证。main实际ledger hash与投影收据一致。ledger仅派生引用，canonical为docs/specs。

archived：按实际归档状态门记录；cleaned：尚待强制guard和工作树/branch/consumerB/C清理。归档与清理收据另见actual-archive-check/cleaned。I1仅实验分母交付；I2生产AutoUI/RQHost迁移、I3 Forge/发行、Shader/CAP以及graphics源码进master前置保持pending。

## Artifact and batch follow-up

本次无生产consumer迁移、默认入口或生产binary/web bundle写面；未宣称部署。driver/AGP原生binary本轮完整build并用于真实验收，临时target按授权cleanup移除，源码可由autographics-dev独立再构建。根backend/daemon/web production artifacts不消费该实验入口，未更改、无需因本Plan重建/重启。

清理后的批量回归due-check仍须记录；不得在plan树跑tf。全部原始失败/正负控/来源/投影与最终checkpoint由本目录证据保存。
