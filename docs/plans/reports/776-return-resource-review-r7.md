# PLAN-776 最终独立复审 R7（2026-10-14）

**outcome: needs_fix。** R6旧例关闭，alias完整目标/九层链恢复，生成JSON scaffold/bus-SSE实际wire通过；但Future“血缘”仍是函数级猜测，且无条件generator包装未适配完整非HTTP消费者。另有重复SSE无帧失败未解释及新增dead_code警告。

## 1. 基线

- review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施会话，延续R5/R6审查上下文，无子agent或额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；reviewed_commit: `5fda97cebd952397b41af80f7ea5f076534223f9`。
- base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`；repair_diff_base: `5bdca42e49483a79c8a0b69c4758b24986b8238b`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: plan-776-dev；clean。
- dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r7/input-hashes.json)；SD SHA256=`F9C084F064FED9A30FF511907F65096E762667E7D85781299C2AD3873C29FF2E`，[冻结副本](../attachments/776/review-r7/reviewed-spec-delta.md)。
- 生产库重新build后链接探针；库hash/HEAD见`attachments/776/review-r7/baseline.txt`。只写主检出docs/plans/**，未修实现/合入/归档。

## 2. Findings

### F776-R7-01 / P1：按整个函数Debug串登记async，不能证明当前终值的Future来源

位置：`vm/codegen.rs:44-51/1698-1705`、`vm/ffi/http_server.rs:5704-5725`。影响AC02/03/04/05、T01/03/04/07/08、SD02/03。

`body_returns_async_block`扫描整个body的Debug串，只要存在`Return(AsyncBlock`，全函数所有返回值都会进Future位模式+查表门。与原声明门一样，仍未证明当前返回值的来源。实际真TCP：

| case | handler body（~Iter声明） | expected | actual |
|---|---|---|---|
| V7-F0（R6旧负例） | `let active=~{42}; return 496` | JSON496 | JSON496，旧例关闭 |
| V7-F1 | `let active=~{42}; if false { return ~{99} }; return 496` | JSON496 | JSON42；未执行分支的语法触发错误驱动 |
| V7-F2 | `return future_helper()`，helper为`fn future_helper() int { return ~{42} }` | 真实Future展开后JSON42 | JSON496；helper返回血缘丢失 |
| V7-F3（R5/R6正例） | `return ~{events()}` | SSE data:42 | 独立单请求通过；多请求测试另见R7-03 |

496=0x1F0，为该独立进程真实fid1的同值普通数字。F1违反Plan明确Future位模式普通数值守卫；非未批准新能力。F2说明该函数级登记也遗漏真实返回值传播，不能通过把遍历Debug串换成AST扫描解决逐值身份问题。

证据：[probe.rs](../attachments/776/review-r7/probe.rs)、[reproduce.ps1](../attachments/776/review-r7/reproduce.ps1)，`future-number.log/future-branch.log/future-helper.log/internal.log`。修复需证明实际终值来源，保留**同一函数不同分支**的Future/同值数字组合和helper正例；不得仅把当前反例字符串排除。

### F776-R7-02 / P1：generator无条件包装后，只适配next，enumerate等消费者仍读裸i32

位置：`vm/codegen.rs:11539-11546`、`vm/native.rs:6194-6206`；同族待审`shim_iterator_map/filter/collect/reduce/find`。影响AC02/07/08、T01/02/07/08、SD01/04/05/07。

本轮所有generator生产点都返回TAG_OBJECT Sse凭据，但native适配仅在`shim_iterator_next`，`shim_iterator_enumerate`仍`pop_arg_i32`并直接查iterators。资源对象id被当成iterator id，合法源查不到返回-1，破坏Plan要求的非HTTP消费者支持边界；不是要求新增迭代功能。

V7-N1直接调用真实生产native：建立一个含[42]的ListData及id0的合法Iterator::List（native枚举不依赖具体源子类），对同一底层iterator分别传裸id和`push_sse_resource`真实凭据：

```text
raw enumerate result=Ok(()) handle=1
credential enumerate result=Ok(()) handle=-1
```

控制只改变身份载荷，同一底层表项持续有效。源码确认generator同样产该凭据，enumerate按id查表的行为不依赖List/Generator子类。证据`native.log`及probe的native模式。修复应统一iterator消费者的凭据解析/owner/ownership与派生iterator身份，逐项审计上述适配/终结调用；保留裸iterator控制与for-in，不可只增加另一个next例。

### F776-R7-03 / P2：S3真实SSE两次有头无帧，单跑绿尚不能解释失败

位置：`tests/plan776_http_return_tests.rs:1860-1873`及其Internal Future/SSE执行链。影响AC03/06/09，T03/04/07/08；本轮未证明由本次diff引入，故记录为必须分诊的验收失败。

- 第一次plan776全族：20passed/1failed，37未跑；第二次`--no-fail-fast`：57passed/1failed。
- 两次均在同一服务依次访问/s0,/s1,/s2后，/s3收到200 text/event-stream头但30秒没有data:42；服务器已就绪且回复了头，现有证据不能把原因定位成端口启动竞争。
- 精确单测再跑1/1绿。不能把全族记录改为58/58；单跑绿仅证明间歇性，不证明纯harness问题。

证据`plan776.log`、`plan776-retry.log`（均含原始HTTP头/失败断言）、`sse-single.log`。需归因调度/生命周期/锁/读协议等真实原因，增加可定位状态和有界oracle；如果最终确证预存/环境，应提供具体机制与基线证据，不只写“在册harness race”。不要求在review中修复。

### F776-R7-04 / P3：alias改为完整目标后遗留bare_type_name死代码

位置`api/mod.rs:243-255`。生产库build的407总warnings中有本轮新增`bare_type_name is never used`；此前406，修复diff保留了不再使用的helper。影响T06/08、AC09健康检查，清理后与知识稿同步。`plan776_dbg_db_transpile`仅打印无断言的诊断探针也建议移至durable调查证据或改成有区分能力的oracle，避免作为通过测试数证明覆盖；后者为非阻断建议。

## 3. R6闭合与规范增量

| R6 ID | 本轮判定 |
|---|---|
| R6-01 | 旧int/local/helper SSE原例关闭，生产点包装不再看声明；但非HTTP兼容R7-02未闭合 |
| R6-02 | Future完整目标、九层合法链、限定名原例均File；原例关闭；canonical/ledger未发布 |
| R6-03 | 原496旧负例关闭；完整Future来源未闭合，R7-01仍有误驱动与漏驱动 |
| R6-04 | 新增生成JSON scaffold/bus SSE真实wire本轮1/1绿；证据确为生成+实编+运行，不手写Rust业务体。JSON明确是scaffold CRUD，不证明API_AT中db.create_message真实业务体或production strict admission；742源业务体面仍未验证，不冒称该面通过 |

SD的“返回~{}血缘门”不能被发布为逐值来源证明，SD“iterator natives解包兼容”也超出当前仅next的实际支持；待修复冻结新版。合法alias目标修复保留。spec-impact路径与GOAL003/016无需修改；本轮不发布canonical/live ledger。

## 4. AC / task / evidence

| AC | verdict | evidence/mapping |
|---|---|---|
| AC01 | pass（已测资源碰撞） | T02/03/07，旧普通资源数字控制保持；Future专门要求见AC05 |
| AC02 | fail | T01/02/03/07，V7-F1/F2与N1 |
| AC03 | fail | T03/04/07，Future终值误驱动/漏驱动，S3两轮无帧 |
| AC04 | partial | T04，旧终态控制保持；新函数级门组合不能泛化为完整正确 |
| AC05 | fail | T02/04，V7-F1违反普通位模式数字守卫；owner/id边界控制保持 |
| AC06 | partial | T05/07，既有1000/重放等保留，S3生命周期无帧原因待分诊，未证明内存泄漏 |
| AC07 | partial | T06，alias原例通过；新增载荷消费者native适配不完整 |
| AC08 | partial | T02/06/07，scaffold JSON/bus-SSE生成wire通过，非HTTPconsumer退化；实际源业务体742边界保留 |
| AC09 | fail | T08，S3红未解释，新增warning，SD未闭合 |

重开T01/02/03/04/06/07/08，T05保留已经证实的存活子项（AC06未宣称完整通过），current_step=1/8。该保留不免除S3分诊；任务各子项历史证据不抹去。

## 5. 本轮门禁与next

| actual | result |
|---|---|
| cargo build -p auto-lang --lib | 成功，407warnings；新增dead helper见R7-04 |
| plan776（test-http-e2e,-j1） | 首轮20pass/1fail/37未跑；全族复跑57pass/1fail；失败同S3；精确单跑1/1绿 |
| cargo t --no-fail-fast | 5308/5308，87.705s，1slow无timeout |
| cargo tv | 162/162，4.449s |
| cargo th -j1 --no-fail-fast | 128/130，230.972s；corpora在册红 + plan707 relay帧时序红；relay精确筛选1/1绿（2.662s，既有时序flake族），完整红记录保留 |
| auto-man新增生成JSON/bus-SSE wire | 1/1，18.020s |
| 修复触面7文件rustfmt / git diff --check | 均干净 |
| independent probes | R6旧例通过；新Future分支/helper、iterator载荷对照按合同期望失败 |

tf归merge-owned未跑，tt/tu/taa无触发。File/Upload生成wire未重复：其代码/依赖/harness相对R6未变，R6同树基线2/2证据保留；本轮不因此宣称所有生成业务体通过。

relay复核使用nextest显式`-E test(http_e2e_plan707_relay_frames_timed_single_termination)`，结果确为1例；此前一次在cargo th别名后追加名称未缩小其固定filter，实际16例提前遇corpora红（relay-single.log），不作为relay单跑证据。各原始调用/结果均保留。

门禁/探针结束复核HEAD仍`5fda97cebd952397b41af80f7ea5f076534223f9`，实施树与auto-down无未提交改动。

next: work，同一lang-776树修复，先交“完整不变量→为何旧oracle遗漏→设计与路径/状态调整→当前结果”闭合表，再提交最终复审。不要以更多函数级标记代替逐值来源，不降低合同或重置历史轮次。合入/归档/Spec沉淀/清树/到期回归仍归merge，待真正final pass。
