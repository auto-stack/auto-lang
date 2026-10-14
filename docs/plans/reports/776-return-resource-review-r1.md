# PLAN-776 最终复审 R1（2026-10-13 开始，2026-10-14 记录）

结论：**needs_fix**。当前实现没有闭合已批准的逐值身份、owner 隔离和资源终结合同。
`plan_revision: 1` 不变，状态退回 `executing`；不合入、不归档、不修改 canonical Specs。
这是独立于实施会话的最终复审；审查者为当前 Codex 会话，未委派另一 agent，也不声称额外的模型独立性。

## 1. 复审基线

| 项 | 冻结值 |
|---|---|
| review_scope | final |
| reviewed_commit | `b4183e761199642aa341cfecd6491c1821c9e2e4` |
| base_commit / merge-base | `5e21561d2c303a02319ff7a29e4607933e09a38f` |
| worktree / branch | `D:/autostack/.wt/lang-776/auto-lang` / `plan-776-dev` |
| 主检出 | `685026eb5103ff3284c3022abf8de7699c4eb1db`；读取本树当前 Plan，不读取实施树的旧副本 |
| auto-down dependency | 组内 `../auto-down`，detached `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean |
| Plan 输入 Git blob | `dac29340642a5a540a48963f45b9b0aef3295e01`（写入本轮记录前） |
| reviewed Spec delta blob | `7452145a80ebcd30d00e3a41b5656206991f85d5` |
| Spec/Plan SHA256 | [input-hashes.json](../attachments/776/review-r1/input-hashes.json)，七个 canonical 输入逐项冻结 |
| 实施树 cleanliness | 复审开始、门禁后、写结论前均无未提交实现；HEAD 未漂移 |

实际新增适配源参与 build closure：`build.rs::collect(crates)` 与
`manifest.rs::provider_inputs` 会收集新 Rust 文件，不能因手工 provider 清单没有新增行
就断言新文件未入 fingerprint。这一部分源码核查通过；不替代实际消费者运行证据。

## 2. 可重放证据

源码与日志在 [review-r1](../attachments/776/review-r1/)。
运行 `powershell -File docs/plans/attachments/776/review-r1/reproduce.ps1` 可重新编译独立探针，
链接实施树产物并逐进程运行。脚本记录 HEAD、rlib 名和 SHA256；本轮库为
`libauto_lang-db38cbe194ab8fc3.rlib`，SHA256
`9F8D5483B7D26CCE429501F07D8ACB1944B8CEE1462BE60006400E24697E15CF`。
不向 `crates/` 插入测试或修复；TCP 探针使用 OS 分配端口、有界 readiness，观察真实编译执行和 wire。
探针收集所有反例，其退出码不是验收 verdict；下列 expected/actual 才是判定依据。

## 3. Findings（全部为本合同内必须关闭项）

### F776-R1-01 [P1] 普通整数仍会被表命中提升为资源，producer 分母遗漏

- AC-01/02/03/06；T-01/02/03/07。
- `http_server.rs:5794,5817,5833` 仍按 iterator/HTTP_RESPONSES 查表分派普通 i32/i64；
  File/Upload 也保留声明加普通数值取表的入口。增加对象臂没有替换被禁止的猜测。
- 真 TCP 反例：`fn p() int { let live = stream(); return 0 }`，
  `fn stream() ~Iter<int> { yield 42 }`。初次活跃 iterator 实际 id=0；
  expected `200 application/json` body `0`，actual `200 text/event-stream`、`data: 42`。
  见 `iterator.log`；不是任意固定数字冒充资源碰撞。
- `http.response()` 构造仍产裸 id；本轮 `legacy.log` 的构造后普通 `1` 被当 Response，
  默认 status=0 最终得到 wire 500/空体，而非 JSON `1`。host 直接 producer 控制见 `units.log`：
  `shim_http_created` 的真实 id 仍命中旧表。
- `stdlib.rs:6162..6253` 的 `ok/created/bad_request/not_found/internal_error` 五个真实服务端
  producer 未迁移，不能把旧表描述成“仅 client + setter 死面”。client Response 返回服务端
  也被原 Plan 的非目标说明明确纳入身份边界。
- 修复：穷举全部 HTTP egress producer/访问器，保留旧 int **声明**的源码兼容，但资源值须有来源；
  普通数值不得通过任何表臂取得或消费资源。新增 Response/iterator 的实际 backing id 碰撞 wire。

### F776-R1-02 [P1] 新增返回槽释放按整数内容扣减无关资源份额

- AC-01/02/05/06；T-02/03/05/07。
- 新增 `http_server.rs:5473,6601` 无条件 `vm.rc_release(nv)`；
  `rc.rs::heap_ref_id` 把普通 i32≥4M 当裸 heap id。数字槽没有持有凭据份额，也会扣活对象 RC。
- `units.log` 的 `PLAIN_RELEASE`：真实资源 id 从分配器读取，保留在 globals；
  构造同值普通 i32，再调用新增 dispatch cleanup。expected RC 1→1、保留资源仍活跃；
  actual RC 1→0，`reap_dying` 跨宽限窗后资源消失。
- 现有碰撞测试只立即重放；8192 步 dying grace 可掩盖这类误释放。
- 修复：按槽实际 ownership/stake/tag 释放本槽份额，不能按普通数字内容扣其他对象；
  wire 回归需数字碰撞后跨宽限窗再重放共享资源。

### F776-R1-03 [P1] VM 内局部编号没有 owner 身份

- AC-05；T-01/02/07。
- `HttpReturnResource` 只有 kind；`resolve_resource/resolve_resource_value/read_claim`
  只拿 u32 id 查传入 VM。两个 VM 的分配器均从 4M 起步。
- `units.log`：A 构造 201/owner-a、B 构造 202/owner-b；两值编码完全相同。
  expected 在 B 拒绝 A 的凭据；actual `resolve_resource(&B, A_value)` 得到 B 的 202/owner-b。
- 现有 wrong_owner 测试的接收 VM 为空，只验证 miss，不能证明身份隔离。
- 修复：按值携带或可验证地绑定 owner/session 与代次；测试双方同时持有同号资源、
  owner 替换/失效及两种载体。不能把“每 VM 一张表”作为来源证明。

### F776-R1-04 [P1] 真 Internal Future 与异常退出仍滞留凭据

- AC-03/04/06；T-04/05/07。
- `probe_return_future`（6720 起）及 AwaitReturnFuture/AsyncReturnBody 终态只编组、
  `remove_task`，没有终结结果的持有份额；`HandlerEnd::Err`（5456 起）也不释放任务栈凭据。
- `internal-life.log`：`fn p() ~int { return ~{ delay_async(10).await; http.response_redirect(...) } }`，
  日志确认真实 body completion 的 `VmRef`；11 请求全 302 后 drain 超过8192步，
  expected live回基线0，actual live=11。
- `error-life.log`：先创建 redirect，再 `1/0`；11 请求全500，drain后仍live=11。
  普通成功 local/直接资源返回的对照 `local.log`、`async-life.log` 回基线0，
  故不能用成功直接返回的1000循环证明错误/Future出口。
- 断连探针 `cancel-life.log` 最终live=0，但真实结果仍执行到302；受半关闭/连接确证规约限制，
  此单探针**不声称取消已发生或迟到完成已被丢弃**。deadline/shutdown 确证矩阵仍待补。
- 修复：按 HTTP 资源范围补完整终态清理，保持逃逸/shared Response；
  补 Ready/External/Internal、成功/失败/取消/关闭及迟到资源 oracle。
  P776-D1 的“通用RC非目标”不能豁免已经复现的本计划资源泄漏。

### F776-R1-05 [P2] 来源分类仍是字符串和名集，遗漏别名/空声明

- AC-07/08；T-01/06/07。
- `ApiExtractor`（127起）不处理 `Stmt::TypeAlias`，`extract_type`（197）丢空成员声明；
  back_proxy 名集也没有 TypeAlias，且 from_return_identity 未使用导入/声明绑定。
- `units.log`：`type Response = int;` 和 `type Response {}` 两个合法 parser 输入均
  `types=[]`、endpoint分类 `ExplicitResponse`；expected 用户类型Json。
  `http.FileResponse` 当前解析为Json，尚没有 stdlib 限定名来源正例证明。
- 修复：共享解析后的声明/导入/alias来源身份；补声明顺序、空record、用户alias、
  stdlib alias/限定名、导入同名组合，三消费轨结果一致。

### F776-R1-06 [P2] 已迁移 producer 的对象凭据没有适配既有 Response 访问器

- AC-02/07；T-02/06/07。
- `stdlib.rs::shim_response_status_code`（7187）仍只弹整数并查旧表；
  body/header/setter同族没有 HttpReturnResource 分支；engine CALL_SPEC 也没有新增类型分派。
- `units.log`：307快照凭据交给真实 status shim，expected307，actual0；
  同307旧表句柄控制返回307。旧client访问器通过不能证明新producer值的访问器兼容。
- 修复：将所有既有兼容访问器和可变操作接上新载荷，保留重复快照/共享值语义，
  对真实 producer 的访问器和setter做正反例，完善独立适配合同。

### F776-R1-07 [P2] 分配耗尽检查晚于实际32位载荷容量

- AC-05；T-01/02/07。
- `push_kind`（96起）只检查u64计数器回绕到4M以下，却在104行 `handle as u32`。
- `units.log`：将 allocator设为 `u32::MAX+1`；expected拒绝分配、无表项；
  actual `Ok(())`、heap_len=1、返回值解析None。可同样截断至另一个已存id。
- 修复：在实际载荷边界之前拒绝分配，避免truncate/reuse，测试边界两侧以及活跃同号项。

### F776-R1-08 [P2] Clone/Drop 不配平，存活计数 oracle 不可靠

- AC-06/09；T-02/05/07/08。
- `HttpReturnResource` derive Clone（64）不增加LIVE_RESOURCES，Drop（71）每实例减1。
- `units.log`：克隆再drop而原堆对象仍存在，live 3→2；expected保持3。
  所有原件再drop会下溢。生产路径目前未发现主动clone调用，此项不冒称已触发生产泄漏。
- 修复：禁止无必要的clone或实现配平；测试仅cfg(test)观察/生产暴露边界与SD一致，
  以owner资源表/真实份额佐证生命周期，不仅依赖全局Atomic计数。

### F776-R1-09 [P2] 完整矩阵、生成wire与健康门被过度宣称

- AC-03/04/07/08/09；T-01/04/06/07/08。
- 新异步fixture `delay_async(30)` 没有 `.await`（测试文件438/444/450/493），
  无 `~{}`、无完成源gate或Ready/park证明，不能等价为三种Future形态。
  helper/容器/closure的手动赋值探针不证明真实语言传播/资源终值。
- 46项api_gen lib测试是生成字符串/控制测试；生成File/Upload真TCP族由
  auto-man的 `test-http-e2e` feature另开，常规lib命令不运行；production receipt运行测试ignored。
  不能把46/46当成“已实际生成、编译、运行全部支持面”。
- 新 harness固定19010等端口、吞 `crate::run` 结果，sleep探测端口；没有复用755有界结果机制，
  不符合Plan §6明确要求。
- 本轮全21个变更Rust文件的format check发现 `vm/ffi/mod.rs` 漂移；
  `cargo-check.log` 的新文件unused HashMap、auto-man新测试unused ApiExtractor为新增warnings。
- SD-01/02/03/05/06/07含无法成立的owner、耗尽、逐值数据保证、旧表仅client、
  终值统一与来源冻结描述；SD-02的保留猜测是未批准缩减，不能据此关闭P734-D7。
- 修复：先闭合实现，再按实际矩阵补oracle/实际生成wire，清新增warnings和触面格式，
  更新验收表与SD证据；不得降低原合同。

## 4. AC → Task → 验证结论

| AC | Tasks | verdict | evidence / coverage boundary |
|---|---|---|---|
| AC-01 | T01/02/03/07 | fail | iterator真碰撞变SSE；同值普通数值错误释放活资源份额（F01/02） |
| AC-02 | T02/03/06/07 | partial | 现有常量/直接返回通过；旧producer、真实传播与访问器未闭合（F01/02/06） |
| AC-03 | T03/04/07 | partial | 直接Response/i64/record专项绿；真正Internal资源返回漏终结；完整四资源×Future×named/closure未覆盖（F04/09） |
| AC-04 | T04/05/07 | fail | handler失败500正例绿，但携带资源异常退出仍泄漏；确证取消/迟到资源矩阵待补（F04） |
| AC-05 | T01/02/04/07 | fail | owner同号误取、32位耗尽不拒绝、数字误释放（F02/03/07） |
| AC-06 | T02/05/07 | fail | 1000直接返回/SSE断连通过；Internal/异常11资源滞留，Clone计数不配平（F04/08） |
| AC-07 | T01/02/06/07 | fail | 755四守卫绿、build闭包纳新源；用户alias/空类型分类错误，新增载荷访问器不兼容（F05/06） |
| AC-08 | T06/07 | partial | auto-man专项1、api_gen46绿；来源失配；生成真实TCP未由所报命令覆盖（F05/09） |
| AC-09 | T08 | fail | verdict/SD/矩阵需修正；未复现整档全绿，格式/新增warning未清（F09） |

## 5. 独立门禁实测

| 命令 | 本轮结果 | durable log |
|---|---|---|
| cargo check -p auto-lang -p auto-man | 成功；新增warnings未清 | cargo-check.log |
| cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1 | 25/25绿，12.195s；不证明遗漏格 | 工具回执摘要已冻结于本表 |
| 裸cargo t | 1970/5295运行，1969绿、chart streaming 1红，fail-fast | cargo-t.log |
| cargo t --no-fail-fast | 5295运行，5294绿、plan730_commit_target_matrix 1红，75.230s | cargo-t-complete.log |
| 对上述两个红项各scoped重放 | 各1/1绿；不把独立重放拼成整档pass | t-red-replay.log / t-upload-replay.log |
| cargo tv | 162/162绿 | cargo-tv.log |
| cargo th -j 1 --no-fail-fast | 110运行，108绿、corpora和sse_chain各1红，210.674s | cargo-th.log |
| nextest -E test(e2e_sse_chain) -j1 | 1/1绿 | th-sse-isolated.log |
| cargo t plan755 | 4/4绿 | plan755.log |
| cargo test -p auto-man --lib api_gen -- --test-threads=1 | 46绿、1ignored | auto-man-api-gen.log |
| cargo test -p auto-man --lib plan776 -- --test-threads=1 | 1/1绿 | auto-man-plan776.log |
| rustfmt --check --edition 2021 --config skip_children=true 全21触面Rust文件 | fail（vm/ffi/mod.rs） | rustfmt.log |
| git diff --check master...HEAD | 通过 | 本轮工具回执 |

corpora红与执行报告既有基线项同名，参数绑定400面不在本diff；本轮未伪造新的base重跑。
chart/upload/sse_chain均scoped通过，暂列本基线间歇红待归因，不认定为已证新增实现回归。
上传红actual409/session_conflict vs expected403；chart红x轴缺t29；SSE红body空。
原报告的“完整th串行全绿除唯一预存红”未在本轮复现。
`th` alias自带http_e2e滤串，追加位置滤串为OR；一次误用追加滤串的重放记在
`th-sse-replay.log`（16/110因corpora中止），没有计为scoped成功，已用filterset纠正。

未运行tf/taa/tt/tu：遵照现行触面规则，本diff无aavm/trans/ui_gen源码改动；tf是主检出批量档。
未重复全auto-man lib或重跑所有嵌套生成编译：已确认工作自有实现/矩阵缺口，先退回work；
F09记录真实生成wire与完整矩阵为待完成验收，不从旧收据或mock授予pass。

## 6. Spec delta 与下一步

已冻结 [reviewed-spec-delta.md](../attachments/776/review-r1/reviewed-spec-delta.md)，
本轮SD-01..07均不能整体批准：SD-04的SSE单源连接点部分成立，其owner/终结覆盖仍依赖未关闭项；
其余失配见F09。metadata七个supersedes目标、一个新增目标路径均有效，GOAL-003/016保持原授权。
review不发布canonical或写ledger。

T-01..08均有本轮未满足子项，全部重新打开；`current_step: 0`表示完整完成task数，
不抹去历史实施提交、通过子项和原work收据。下一步在同一 `lang-776` worktree按F01..09修复，
提交后交独立R2；优先完成producer/所有权/owner方案和真正async终态，再补来源/访问器/边界oracle。
merge-owned的Spec/ledger沉淀、归档、wt-guard清树与到期批量回归留到最终pass后。
