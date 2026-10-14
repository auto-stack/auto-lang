# PLAN-776 最终独立复审 R5（2026-10-14）

**结论：needs_fix。** R4 的 final pass 被本轮实际反例推翻。当前合同 revision=1 未改变；修复必须在原 lang-776 实施树进行。本轮没有修实现、合入、归档或发布 canonical Specs/ledger。

## 1. 审查基线

- review_scope: final；reviewer_identity: 当前 Codex 会话，独立于实施会话，从实际源码和反例重建结论；无子 agent/额外模型声明。
- reviewed_commit: `73d4f0ff3a89f507c4192e38304354688ea00b52`；base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`。
- worktree: `D:/autostack/.wt/lang-776/auto-lang`；branch: `plan-776-dev`；实施树 clean。
- dependency: sibling auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- Spec/Plan 输入 hash：[input-hashes.json](../attachments/776/review-r5/input-hashes.json)；审查 SD 冻结：[reviewed-spec-delta.md](../attachments/776/review-r5/reviewed-spec-delta.md)。SD SHA256=`7F4DC0234B38CA877C3E51E20B591236E72261107BE0C7677A9743671FA80F85`。
- 探针通过 `cargo build -p auto-lang --lib` 重新建立生产库后重放，库 hash/HEAD 见 [baseline.txt](../attachments/776/review-r5/baseline.txt)。未修改实施代码/测试。

## 2. Findings

### F776-R5-01 / P1：SSE 身份只覆盖直接调用表达式，局部/helper/Internal 传播退化成 JSON 数字

位置：`vm/codegen.rs:2713-2721`、`:16768-16777`；`vm/engine.rs:2818-2830`；`vm/ffi/http_server.rs:5863-5879`。影响 AC-02/03/08、T-01/02/03/07/08、SD-01/03/04/07。

codegen 只对流声明函数的 `return <已登记 generator 直接调用>` 插包装指令。普通 generator 调用仍产裸 iterator id；`return stream`、helper 调用与 Internal body 尾表达式无法恢复来源，marshal int 臂只能当数据。不是新增能力要求：Plan §5.1/§6 明确承诺 helper/local 和 Future 传播，以及真实 SSE 同形。

同一 `events() ~Iter<int> { yield 42 }`，同一真实 HTTP `/p` 入口，每种独立进程：

| Case | handler body | expected | actual |
|---|---|---|---|
| V5-S0（正控制） | `return events()` | SSE data:42 | 200 text/event-stream，data:42 |
| V5-S1 | `let stream = events(); return stream` | 同上 | 200 application/json，body `0` |
| V5-S2 | `return helper()`；helper 存局部再返回 | 同上 | 200 application/json，body `0` |
| V5-S3 | `return ~{ events() }` | 同上 | 200 application/json，body `496`（该次 iterator/Future 数字结果；不作为稳定 id oracle） |
| V5-S4（负控制） | `let stream = events(); return 0` | JSON 数字 0 | 200 application/json，body `0` |

证据：[probe.rs](../attachments/776/review-r5/probe.rs)、[reproduce.ps1](../attachments/776/review-r5/reproduce.ps1)，同目录 `direct/local/helper/internal/plain.log`。库构建后第二次执行结果相同。先修 producer 与合法传播，使资源身份跨上述边界，兼容非 HTTP generator 消费；不得恢复声明/查表授予身份。永久 oracle 需同时保留 S0/S4 并补真实闭包/容器与 async 代表组合，不能只增加直接调用示例。

### F776-R5-02 / P2：限定 stdlib 名仍被用户同名遮蔽，别名解析只展开一层且不规范化限定目标

位置：`api/contract.rs:65-93`（`from_return_identity`）；`back_proxy.rs:1519-1525` 的独立一层 alias 展开。影响 AC-07/08、T-01/06/07/08、SD-02/06/07。

分类器先从 `http.FileResponse` 剥离 `http.`，然后照样查询用户名集，因此与注释/SD 所称的“不受用户同名遮蔽”相反。alias target 直接送 `stdlib_kind`，无法识别 `http.FileResponse` 或 alias→alias。探针经过真实 parser + ApiExtractor（不是手造 AST），三例均解析成功：

| Case | input | expected | actual |
|---|---|---|---|
| V5-T1 | 用户 `type FileResponse {}`；返回 `http.FileResponse` | File | Json |
| V5-T2 | `type Download = http.FileResponse`；返回 Download | File | Json |
| V5-T3 | First=FileResponse，Download=First；返回 Download | File | Json |

证据：`identity.log`。这会使生成分支/能力守卫与真实来源失配；本轮不声称已经执行错误生成物。需按实际解析绑定处理限定名、alias 链/包装/循环拒绝，统一 VM/back-proxy/生成消费者，并保留用户普通同名 Json 正控制。

### F776-R5-03 / P2：AC-08 必需生成真 TCP oracle 仍未完成，却被记录 final pass

位置：Plan §7 AC-08、§8 T-07/08、§9 R4；R4 报告 §3/5。影响 AC-08/09、T-07/08、SD-07。

R4 自身把 AC-08 判为 partial，仍给 final pass；“预存编译漂移归742”仅解释失败归属，不能代替 Plan 明确要求的现行 Rust JSON/File/Upload/bus-SSE 生成、实编和真实 wire 证明。本轮未找到用户批准移除该 oracle 或修订验收的证据，Plan revision 仍为1。R2/R4 审查记录不是用户授权。

先为现行支持面建立可运行生成 fixture/实际 oracle；若确实依赖742，则明确该前置未满足，在 work 修复/等待依赖，或经用户批准修改合同后重新验收。不可用模板/单测总数/将缺口记债替代验收。未要求776接管742业务体修复。

### F776-R5-04 / P2：新增死代码和非法 doc 注释警告未被健康检查关闭

位置：`vm/codegen.rs:1600-1603`，`vm/ffi/http_server.rs:117/5576/5587/5607`。影响 AC-09、T-06/08。

实际生产库构建报新增 `unused_doc_comments`（block 内 `///`）及 `fn_is_generator` / `handler_declares_stream_return` / `handler_is_generator` / `declared_return` dead_code。这些符号/行均在776 diff，不能将411总警告一概判为预存。另有失效注释仍声称旧表/声明门工作（资源模块头、marshal int 臂），须与终形实现对齐。证据 `build-library.log`。清理不再使用的侧道/门与错误注释，对新增 warning 逐条关闭，仓库既有 warning 不扩大为本计划全仓整改。

## 3. AC / task / evidence

| AC | 本轮 verdict | 任务/证据 |
|---|---|---|
| AC-01 | pass（已测范围） | T02/03/07；plan776碰撞/声明普通int负控制49全族绿，新V5-S4保持数据 |
| AC-02 | fail | T01/02/07；V5-S1/S2原合同local/helper身份丢失 |
| AC-03 | fail | T03/07；V5-S0/S1/S3相同流体不同wire；Response/JSON代表例仍绿 |
| AC-04 | pass（既有覆盖） | T04；Failed/Nil/取消控制专项，本轮未修改失败路径，P776-D3不重新冒称成功 |
| AC-05 | pass（已测范围） | T02；owner/stale/exhaustion/Future bit专项绿，未要求无限VM/id承诺 |
| AC-06 | pass（既有覆盖） | T05；1000/重放/setter/nested专项绿；不据此证明尚未修复的SSE传播组合 |
| AC-07 | fail | T06；V5-T1/T2/T3，不能以755/738单测代替来源证明 |
| AC-08 | partial | T06/07；分类/传播有实际缺陷，生成真TCP必需oracle缺失 |
| AC-09 | fail | T08；SD与实际来源/传播不同；R4过判/新增warnings |

reopened_tasks=[T-01,T-02,T-03,T-06,T-07,T-08]；T-04/T-05已通过子项保留，current_step=2/8。历史R1..R4原例修复继续有效，但不构成上述完整不变量通过。合同与SD不能通过删除local/helper/别名/生成验收来跟随缺陷收窄。

## 4. 本轮门禁

| actual command/check | result |
|---|---|
| cargo check -p auto-lang | 0 error；生产构建411 warnings，新增触面warnings见F5-04 |
| cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1 | 49/49 |
| cargo t --no-fail-fast | 5304/5304，93.652s；1 slow（738 nested Rust emission），无timeout |
| cargo tv | 162/162，3.976s |
| cargo th -j1 --no-fail-fast | 124/125，232.164s；唯一红back_proxy real_routes_corpora_data_face，在册预存（R1..R4同形）；无timeout |
| cargo test -p auto-man --lib api_gen | 45 passed/1 failed/1 ignored；742 handler_body_module_calls_and_list_construct，单跑准确全名1/1绿，记录为并行/共享fixture敏感，未证明新语义回归 |
| rustfmt --check（全部27触面中的Rust文件，skip_children=true） / git diff --check base..HEAD | 均干净 |
| 本轮独立反例 | 正/负SSE控制通过；S1/S2/S3与三类型来源案例失败（按行为期望判断，probe exit0仅表示成功观测） |

tf为merge-owned批量档未跑；tt/tu/taa无相应触面不跑。完整生成真TCP仍未验证，不从api_gen绿色单例推导成功。

## 5. next

needs_fix → executing。先在同一实施树按四项finding修复/补oracle、提交并刷新SD/验证包，再申请独立最终复审。保留历史证据；后续review需检查完整不变量闭合表，不能只重放旧例。Spec/ledger沉淀、归档、wt-guard及批量回归仍归merge，待final pass。
