---
plan_id: PLAN-790
status: drafting
feature_name: 文件与图像剪贴板宿主能力依赖倒置
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: []
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui]
depends_on: [PLAN-781, PLAN-785]
---

# [PLAN-790] 文件与图像剪贴板宿主能力依赖倒置

## 0. 变更摘要

将 native 2934..2937 的 files/image OS 实现依赖迁到宿主 provider，VM core 只保留列表/字符串/记录编解码和ownership。与已交付文本2925/2926统一装配优先关系，真实no-ui宿主能注入四函数。

## 1. 目标

消除四个shim对ui::clipboard_native的生产直引；原Windows OS行为、原生ID/签名、无provider时空列表/false/null降级保持。非目标：DnD、scroll/editor、term/media迁移，新的图像协议或任意格式转换，不更改既有文本provider合同。

## 2. 架构方案

沿779/785的per-VM显式provider→进程默认快照→兼容默认补空槽规则，provider只使用标准库owned数据（文件路径列表、图像path/width/height），不暴露VM堆句柄、UI类型或Win32 HANDLE。OS/PNG/DIB实现仍在ui::clipboard_native；新UI adapter仅委托。无feature也可注入不依赖UI的fake provider；无人注入时保持既有降级值。

## 3. 技术栈

既有host_capabilities、AutoVM/CompileSession宿主快照、StakeGuard、native_catalog、Windows剪贴板桥与命名互斥；独立Cargo consumer。

## 4. 需求分析与背景调查

取材基线 auto-lang@`3e65540cbe6ac019d210318dc93d17accf4ebcc4`；[输入指纹](attachments/extraction-batch-20261016/inputs.json) 记录现行 Specs、代码和 781/784/785/786 归档合同的 SHA256。overview 是导航快照，以相关模块 Spec 和实际源码为准。
用户 2026-10-16 授权“建立下一批实施计划”；本轮只建立合同及更新设计/路线，不实施 Rust、不创建执行树、不更改旧仓 refs、不发布/推送。之后按确认的合同进入 work；不能把本次建计划算作实施授权或完成证据。
781/784/785/786 均已归档；旧仓主分支和所有恢复载荷继续保留。787/788 为 AC/AutoDown 实验与集成线，不是本批全局前置，实施时按实际 diff 复核共用 manifest/数据类型热点。
native.rs 的 files_get/set 为 List<str>/Bool；image_get 为 {path,width,height}|null，image_set 接收 PNG路径。并非RGBA缓冲直接跨VM传递。2934..2937 当前在 windows+native-clipboard下直引UI，其他分支仍消费参数；文件列表有两种现有ListData形态。785已处理字符串编码/Clone和优先级；不能把其成功外推到列表/记录/临时文件寿命。
实际热点 native.rs、host_capabilities.rs、vm/engine.rs、session.rs、UI installer、CLI UI入口与785相同；785已归档，执行时不得与别的native族同时写这些文件。789/791可独立推进；792等待本批合流后冻结runtime。

## 5. 详细设计

T01 列明四ID的参数解码、错误/空值、非Windows降级与图像临时PNG生命周期；冻结既有Windows和非feature分支输出。provider接口名称由work选择，但不能用Result吞错把既有false/null改为异常。临时PNG仍由既有OS实现策略管理，返回路径在消费期可读；不能provider返回后立即删文件。
所有输入引用在成功、降级、provider错误和提前返回路径均配平；文件列表read锁不得持有到provider回调，返回 owned Vec/记录，跨VM不可带heap id。Unicode/空路径、既有支持列表类型、图片尺寸/路径编码按现行合同。
补空槽装配走785同一生命周期；显式provider永不被UI安装覆盖。provider可重入到不冲突的宿主调用，不得在默认注册锁内执行用户回调。非Windows不把UI默认伪造为可用OS。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | 增四函数provider、编码/缺席/ownership规则 | core去反向实现依赖 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | OS adapter与宿主接线，更新未迁族清单 | 明确Win能力与后续边界 | AC-02/04 |

## 6. 测试设计

实施、独立 final review、merge 分别交接，未指定外部模型/额外 agent。所有人使用本计划同一棵实施树；本轮只给拟定路径，不创建它。V 用例由 work 建成可执行 fixture；下表是合同而非已通过结果。
开发只运行受影响定向案例及必要 cargo check；先修完全部 findings、提交并完成 final review，merge 再在冻结提交执行裸 cargo t 和触面档。cargo tf 仅主检出批量到期回归，不能在每棵树运行。无相关 Rust/Schema 改动不跑 docs_gen；UI codegen 触面用 tu，不用改金样消红。

| V / owner | AC | 输入/入口 | oracle 与覆盖界限 |
|---|---|---|---|
| V01 / work | 01/02 | 新test/fixtures/modularity/clipboard-media-provider-consumer，真实源码→编译→VM四函数；可区分路径/宽高fake | no-default-features独立manifest实际输出；Unicode文件列表、png路径及记录字段值正确，不只直接调shim |
| V02 / work | 02/03 | 空/正常列表、两种既有ListData表示、字符串/图像record；重复调用后清栈+reap | 类型/值/缺席降级保持；heap/string引用回到基线，无所有权悬空和持续增长 |
| V03 / work | 03/04 | 两VM、Clone/merge、显式provider后运行UI安装、默认替换及旧VM快照 | 哨兵值证明不串线、不覆写显式值；provider失败/缺席仍消费参数且不死锁 |
| V04 / work | 02/04 | windows+native-clipboard真OS四函数；ui/no-ui开关、无OS/非Windows静态与可运行控制 | 在现有机器锁下OS循环读写；跳过不算OS验收；若本机不可用记录缺失并待真实环境补齐，不能只用fake宣称全部通过 |

拟实施树 D:/autostack/.wt/lang-790/auto-lang，branch codex/plan-790-dev。merge-owned 裸cargo t+tv，相关Windows OS定向档；tf仍批量单实例。未改视觉布局不要求重跑整画廊；改变UI交互时按autoui-verifier补对应双端验证。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 四个core shim无UI实现直引，no-ui实际VM外部注入V01通过，ID/签名不变 |
| AC-02 | work | 文件/图像数据与缺席值、临时路径行为兼容，V02/04通过 |
| AC-03 | work | 成功/降级/失败的引用和堆对象寿命配平，串线/优先级反控成立 |
| AC-04 | work | 生产宿主真实装配，OS桥保持可用；未迁能力清单明确，V03/04通过 |
| AC-05 | review | 独立重放真实VM/ownership/OS证据及SD，绑定冻结提交 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档、guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 实施/产出/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 781/785交付、合同确认 | 冻结native.rs/UI桥数据、降级、临时文件与ownership矩阵，标出全部宿主接线和OS环境 | 02/03/04 |
| [ ] | T-02 | work | T01 | host_capabilities及engine/session新增owned provider与快照；定向注册优先/Clone/实例隔离控制 | 01/03 |
| [ ] | T-03 | work | T02 | 四shim统一能力调用与编解码；清除UI直引；补所有返回路径引用配平；cargo check+定向泄漏反控 | 01/02/03 |
| [ ] | T-04 | work | T03 | ui新adapter委托clipboard_native，统一installer/CLI等实际宿主接线；显式默认优先测试 | 04 |
| [ ] | T-05 | work | T04 | 新独立consumer及V01..04；系统命名锁下OS验收，no-ui/feature矩阵及777检查 | 01–04 |
| [ ] | T-06 | work | T05 | 剩余宿主族/依赖删边收据、SD素材；提交并确认work验收全部闭合 | 01–04 |
| [ ] | T-07 | review | T06 | 独立重放反控、临时路径与OS证据、源码去边和SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+tv及OS触面门，沉淀/归档/guard清理 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=合同确认并释放native热点后work | 0/8；本轮未运行OS测试或修改provider。

## 10. 待澄清事项

T01须冻结原图像临时文件寿命和平台实际支持，不新增跨平台OS承诺。若需要修改.at签名/值编码/清理时机，先给具体不兼容点修订合同；不能通过禁用native-clipboard/改空值expected规避验收。
