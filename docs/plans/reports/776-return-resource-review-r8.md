# PLAN-776 最终独立复审 R8（2026-10-14）

**outcome: needs_fix。** TAG_FUTURE及iterator功能适配关闭了R7原例，但Future载荷在Value容器中退化为普通整数，返回端仍用声明+位模式授予身份；iterator共享弹栈未完成声称的清零/owned份额清偿。两项为原合同内P1阻断。

## 1. 基线与门禁顺序

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施，延续R5..R7审查上下文；无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；reviewed_commit: `5d4d29ab0fac4cb2b18f7a720d55eb5a18a02213`。
- base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`；repair_diff_base: `5fda97cebd952397b41af80f7ea5f076534223f9`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: plan-776-dev；clean。
- dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r8/input-hashes.json)，包含本轮AGENTS输入；reviewed_spec_delta: SHA256=`F4490ADBAFEAAB171BEAFDC8DDEDE5C26DC5DD52F9325D45FECC5C2E537D1EF2`，[冻结副本](../attachments/776/review-r8/reviewed-spec-delta.md)。
- 遵循2026-10-14后置裁定（AGENTS §2及当前review技能）：先最终代码/验收/Spec审查，review只跑相关反例与针对性验收；t/tv/th及其他整档不抢跑，列为最终pass之后的merge-owned gates。实施交接总数不替代本轮证明。
- 生产库重新build后链接独立探针，实际库hash/HEAD见`review-r8/baseline.txt`。本轮未修改实施代码/测试，未合入/归档/发布canonical或ledger。

## 2. Findings

### F776-R8-01 / P1：Future tag经Value主动降级，legacy返回臂仍误驱动普通数字

影响AC02/03/05/08/09，T01/02/03/04/07/08，SD01/02/03/05。位置`vm/ffi/http_server.rs:5780-5800`、`vm/engine.rs:983-991/4394-4405`、`vm/native.rs:4236-4244`、`vm/interop.rs:130-136`。

新的TAG_FUTURE入口可区分直接生产的Future与数字，R7的函数Debug串门确实已撤；但Value没有保留Future身份，各Value桥主动转为`Value::Int((fid<<8)|0xF0)`。marshal仍保留**~T声明+普通i32位模式+活跃fid查表**臂，不能证明该普通数字曾经过Value转换。

实际同一真实HTTP `/p` 入口、每种独立进程（fid1实际活跃，496=0x1F0）：

| case | input | expected | actual |
|---|---|---|---|
| V8-F0（R7旧例） | ~Iter handler，`let active=~{42}; if false { return ~{99} }; return 496` | JSON496 | JSON496，旧例闭合 |
| V8-F1（R7正例） | ~Iter handler返回`future_helper()` | 展开后JSON42 | JSON42，旧例闭合 |
| V8-F2 | **~int handler**，`let active=~{42}; return 496` | 普通数字JSON496 | JSON42，legacy臂误驱动 |
| V8-F3 | ~Iter handler，`let values=[~{42}]; return values[0]` | 真实Future终值JSON42 | JSON496，容器身份退化 |

证据：[probe.rs](../attachments/776/review-r8/probe.rs)、[reproduce.ps1](../attachments/776/review-r8/reproduce.ps1)及`future-number-async.log`/`future-array.log`/旧控制日志。F2是AC05明确要求的普通Future位模式数值守卫，不能用“继承705的legacy语义”削减该要求。F3验证新载荷的真实容器传播边界，非任意新能力。

修复需在合法Value/数组/容器/捕获传播后仍保留来源，普通Int保持数据；不能用返回声明或查表再恢复身份。若必须改变已批准最小表示范围，应明确needs_replan，不在收据/SD中默许降级。永久oracle同时覆盖~T声明普通同值数字和容器/捕获真实Future正例，不只重放~Iter原例。

### F776-R8-02 / P1：iterator消费槽未清零，源凭据的owned份额未释放

影响AC02/05/06/07/08/09，T01/02/05/07/08，SD01/04/05/07。位置`vm/native.rs:5591-5605`及map/filter/enumerate/collect/reduce/find调用点；与`pop_arg_nv/StakeGuard`（:11746起）既有消费协议对照。

`pop_iterator_receiver`直接`task.ram.pop_nv()`，既不清`raw_nv[sp]`，也不为真实凭据nv建立消费StakeGuard。调用点建立的guard使用的是**解包后的iterator_id**（例如0），不是高段credential heap id，所以不能清偿输入凭据。`VirtualRam::pop_nv`本身只减sp，不清slot。交接/verification声称“弹出槽清零”与冻结提交实际代码不符。

V8-N1真实native对照：同一含[42]的ListData/Iterator::List id0，裸源enumerate成功，凭据源也成功产派生Sse{iterator_id:2}——功能修复闭合。但源由`push_sse_resource`创建，调用前仅有一个owned栈份额；被native消费后没有外部Value/容器/globals持有它。释放派生输出并`reap_all`后，原源凭据仍RC1且存活：

```text
raw enumerate result=Ok(()) handle=1
credential enumerate result=Ok(()) claim=Some(Sse { iterator_id: 2 })
source credential rc after consumed argument=1
source credential retained after output release/reap=true rc=1
receiver=(0, true) popped-slot-actually-zero=false original-tag-retained=true
```

证据`native.log`及probe native模式。前四行是消费份额泄漏；末行独立检查共享helper弹出位置，证明交接清零声明不成立。正常CALL_NAT净零栈的派生操作也无法用净减栈死区结算清偿已被新输出覆盖的输入stake（既有消费协议注释解释该边界）。本轮未用RSS猜测泄漏。

修复必须对**原始凭据nv**执行读取后消费/清零，兼顾错误早退、无对象裸源、派生输出自身份额；不要仅guard底层iterator id或仅把slot改0而仍泄漏owned份额。审计next与全部adapter/terminal，不只给enumerate增加一个功能断言。补消费后计数/槽位/早退/派生链/重复调用oracle，刷新decision/verification/SD关于“全族/清零”的声明。

## 3. R7闭合与SSE边界

| R7 ID | R8判定 |
|---|---|
| R7-01 | 原Debug串分支/helper例通过，新TAG直接载荷有效；完整Future来源仍失败（R8-01） |
| R7-02 | enumerate功能与派生凭据身份恢复；所有权/清零未闭合（R8-02） |
| R7-03 | work补充baseline实验摘要与P776-D4归属，新增单请求S3有界oracle本轮通过；全族仍有头无帧失败，本轮是**/s0**。不能把60/61改写为61/61，也不能用新单例泛化早断连生命周期通过 |
| R7-04 | bare_type_name/dbg无断言测试移除，原告警关闭；触面format通过 |

本轮专项完整61例：60passed/1failed，失败为`http_e2e_plan776_r5_sse_identity_propagates`，日志/s0收到200 text/event-stream头后30s无帧；同例精确筛选单跑1/1绿。Work记录“base 5e21561d2相同churn实验2/6”为预存服务级死锁候选P776-D4；本轮没有复现该历史checkout，其临时树已清，不能把文字归因冒称本轮baseline实证。保留其分诊进展，不另称新回归；AC03/06仍记录已测边界与剩余早断连缺口。该缺口的外轨归属不豁免本计划真实资源/终态合同。

## 4. AC、任务与知识增量

| AC | verdict | evidence/mapping |
|---|---|---|
| AC01 | pass（现有资源碰撞范围） | T02/03/07，普通Response/iterator同值数字原控制保持；Future明确守卫见AC05 |
| AC02 | fail | T01/02/03/07，V8-F3载体身份及V8-N1消费ownership |
| AC03 | fail | T03/04/07，V8-F2/F3错误终值；全族/s0无帧 |
| AC04 | partial | T04，旧Failed/Nil/取消控制保持，不泛化所有新增Future路径 |
| AC05 | fail | T02/04，V8-F2误驱动；资源owner/id边界原例保持 |
| AC06 | fail | T05/07，V8-N1源凭据RC1未回收，早断连缺口保留 |
| AC07 | partial | T06，类型alias正例保持，native载荷功能已修，但ownership协议失配 |
| AC08 | partial | T02/06/07，非HTTP功能恢复但消费协议失败；生成现行面已有指定边界证据未覆盖新载荷ownership |
| AC09 | fail | T08，SD逐值来源及交接清零声明与实际不一致；完整专项红保留 |

reopened_tasks=[T01,T02,T03,T04,T05,T07,T08]；T06已证来源分类/消费者子项保留，current_step=1/8。所有原通过子项与历史记录保留；原合同revision1不降低，metadata的七个supersedes路径/新spec路径/GOAL003与016保持。SD02宣称“位模式恒数据”和降级兼容并存，现实现证明冲突；SD01/05须补真实owner消费义务。冻结delta不通过，不发布canonical/ledger。

## 5. 本轮actual验证与next

- `cargo build -p auto-lang --lib`成功；库hash/HEAD绑定baseline.txt。
- `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1 --no-fail-fast`：**60/61**，61.595s；/s0无帧30.605s失败，完整原始log保留。
- 显式filter同失败单例：1/1，0.636s；新增S3有界例在完整族内通过。
- `cargo test -p auto-val --lib nano_value::tests`：16/16；它只证明现有NanoValue单元面，不代替新的carrier/HTTP反例。
- 10个修复触面Rust文件`rustfmt --check --edition2021 --config skip_children=true`、`git diff --check 5fda97ceb..HEAD`：通过。
- 独立probe重放旧R5..R7控制与新Future/ownership反例；按行为判定失败，脚本exit0仅表示成功观察。最初尝试`auto.iterator`语言调用报undefined auto，不作为缺陷/验收输入；最终native模式调用真实shim，源为合法表项。
- 不跑t/tv/th/tt/tu/taa整档；遵循后置门禁裁定，最终review pass后仍需由merge执行本仓适用日常/VM/HTTP等门禁。tf为主检出batch-only。生成三矩阵本轮未重复：相关api_gen生成逻辑/harness和auto-down相对R7未改，本轮不以旧总数推导新Future/ownership通过。

next: work，同一lang-776树按两项P1修复并更新整条来源/消费不变量闭合表、提交；随后独立复审。交接应列“之前为何漏→所有载体/入口/早退→新oracle→实际结果”，不能继续只用旧例或总数授权通过。冻结基线复核后写回executing；未修实现/合入/归档。合入门禁、Spec/ledger沉淀与清树仍待真正final pass。
