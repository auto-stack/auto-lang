# PLAN-776 最终独立复审 R10（2026-10-14）

**outcome: needs_fix。** R9两项反例与兄弟record路径均闭合；同类owned消费审计发现Response五setter仍保留双释放，共享响应被回收且读回触发UAF。这是原合同内P1缺口，前轮遗漏了共享setter路径，本轮补证。不是本次三文件增量新增的回归，也不能因不在修复diff里而通过整项Plan。

## 1. 基线

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施，延续R5..R9上下文；无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；reviewed_commit: `dd41d684dc2d8c1321034ed56d4a8cb38eab2894`。
- base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`；repair_diff_base: `5b8bffdef3889008d1cc84d85cfda35b01c1afa3`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: plan-776-dev；clean。
- dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r10/input-hashes.json)；SD SHA256=`828CFCD168F00463A155870E1BAED10D450461F0B1DACA017D8F18E1C17D41D1`，[冻结副本](../attachments/776/review-r10/reviewed-spec-delta.md)。
- 生产库重新build后链接独立探针；库hash/HEAD见review-r10/baseline.txt。遵循后置裁定，review只做针对性验收/反例/代码与Spec审查，统一回归待final pass后merge执行；本轮不修实现/合入/归档/发布canonical或ledger。

## 2. F776-R10-01 / P1：Response setter仍消费两份，破坏合法共享响应

影响AC02/06/07/08/09，T01/02/05/07/08，SD01/05/07。位置`vm/ffi/stdlib.rs:5665-5678`，同型header/text/html/bytes（:5680..5754、:6152..6171）。

本轮iterator共享消费已改为StakeGuard唯一释放，RC2→1正确。但Response五setter仍按：

```rust
let stake = StakeGuard::nv(vm, res_nv); // 不retain，Drop负责release
vm.rc_release(res_nv);                // 第一次消费
mutate_response_snapshot(...);
drop(stake);                          // 第二次消费
vm.rc_push(task, res_nv);              // 返回槽只重建一份
```

独占RC1时零计数第二次释放未暴露，返回rc_push复活可使既有跨窗测试通过。共享RC2（参数owned一份+globals一份）时，setter结果应保有**全局+返回槽两份**：RC2→2；实际RC2→1，释放返回槽后0，仍在globals中的合法Response被回收。

V10-R1直接运行真实`shim_http_response_status`，source为`push_response_resource` owned快照200，globals通过retain+写槽建立共享份额（与STORE_GLOBAL模型一致），设置状态201：

```text
shared Response RC before setter=2
setter result=Ok(()) RC after setter=1 snapshot=Some(ResponseSnapshot { status: 201, headers: [], body: [] })
global Response survives output release=false RC=0
[RC canary] use-after-free: heap object 2148532224 was freed 0.0s ago
read retained global Response result=Err(Any { .. })
```

预期：调用返回RC2，释放返回值后RC1，reap后全局快照仍201。actual原始输出证明RC份额缺失及UAF，不用RSS/间接计数推断。探针读取回收后的全局值以catch_unwind捕获canary，保留原始panic日志，避免首个反例中断其余控制重放；捕获不改变验收fail。早退路径在错误发生时同样走guard Drop，须审计不可误销其它合法持有。

证据：[probe.rs](../attachments/776/review-r10/probe.rs) shared-response模式、[reproduce.ps1](../attachments/776/review-r10/reproduce.ps1)、`shared-response.log`。status已实际重现；其余四setter由相同双release源码明确触发同类风险，修复需全部清点，不声称本轮逐个运行了另外四个。

修复使用原始nv的一次owned消费及返回份额正确转移/重建，保持slot清零和读取期间存活；覆盖RC1独占、RC2+共享、返回值丢弃/保存、重复setter链、错误早退、跨reap后全局replay。同步更正SD01/decision的setter“rc_release+StakeGuard保护”模型，guard不是retain护栏。此要求来自Plan的可变访问器/共享存活合同（§5.1/§5.3、AC02/06），不是新增全语言所有权工程。

## 3. R9闭合与本轮额外控制

| case | actual / verdict |
|---|---|
| R9 shared iterator | RC2→1，reap后源凭据仍有效；slot清零；关闭R9-01 |
| R9 typed record Future | `Box { f Future<int> }`构造~{42}再取字段：JSON42；关闭R9-02 |
| V10-F1 record字段赋值 | 先~{7}，`holder.f=~{42}`，再取字段：JSON42 |
| V10-F2 object literal字段 | `{f:~{42}}.f`：JSON42，覆盖CREATE_OBJ/GET_FIELD兄弟路径 |
| 旧R5..R8负控制 | ~int/~Iter/plain int及未执行Future分支的普通496保持JSON496，真实array/helper/closure Future为42 |
| 旧SSE传播 | direct/local/helper/int helper/Internal仍text/event-stream/data42；本轮独立单请求控制通过 |
| V10-R1 shared Response setter | RC2→1→0，globals UAF；fail |

记录来源修复的真实typed流程为CONSTRUCT_INSTANCE/GET_GENERIC_FIELD(push_value)（work定位正确），R9报告对输入流程的源码推断不完整；R10按实际执行路径复核并补了对象literal/字段修改路径。旧R9失败证据/ID保留，这一定位勘误不影响其实际body1反例成立。

## 4. AC / task / Spec

| AC | verdict | evidence/tasks |
|---|---|---|
| AC01 | pass（现有碰撞面） | T02/03/07，65族资源数字碰撞原控制 |
| AC02 | fail | T01/02/07，共享Response可变访问器owned协议 |
| AC03 | pass（约定代表覆盖） | T03/04/07，65族 + 独立Future/record/object/修改/SSE重放；不宣称完整Cartesian穷举 |
| AC04 | pass（代表终态覆盖） | T04，Failed/Nil/缺结果/取消原专项及705针对性验收；P776-D3未冒称成功修复 |
| AC05 | pass（已测边界） | T02/04，owner/stale/id耗尽及普通Future位数字控制保持 |
| AC06 | fail | T05/07，共享Response setter后reap毁灭仍合法的全局引用 |
| AC07 | partial | T06，755/738来源/证明守卫通过，HTTP adapter消费语义仍有R10-01 |
| AC08 | partial | T02/06/07，非HTTP/共享返回面owned协议未闭合；生成证据明确范围，不冒实际742业务体通过 |
| AC09 | fail | T08，setter同类路径遗漏，SD消费模型须修；本轮现有专项无新增红不等于新反例通过 |

重开T01/02/05/07/08；T03/T04/T06已证子项保留，current_step=3/8，所有历史通过项不重置。plan_revision1语义合同不缩减，spec-impact七个supersedes路径/新规范路径/GOAL003与016保持。冻结SD整体不通过：iterator条款与记录Future条款已一致，setterowned模型仍错误。审查不发布canonical/ledger。

## 5. 本轮actual验证

- `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1 --no-fail-fast`：**65/65**，34.998s。
- `cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(plan755) or test(plan738) or test(plan734) or test(plan705) or test(plan394)' -j1 --no-fail-fast`：**126/126**，168.243s，1slow（738嵌套Rust发射），无timeout；这是AC07等必需针对性验收，不是整档回归。
- 生产lib build成功，库hash/HEAD绑定baseline.txt；独立probe全部观察成功执行，V10-R1按合同期望fail，捕获UAF不作为pass。
- 修复触面3个Rust文件rustfmt --check（skip_children=true），git diff --check 5b8bffdef..HEAD：通过。
- 整档t/tv/th/tt/tu/taa未跑，遵循后置裁定，真正final pass后merge-owned适用门禁pending；tf为主检出batch-only。生成矩阵沿用R7/R8确定未改的生成代码/harness/auto-down证据，其范围仅是已记录的File/Upload/scaffold JSON/bus SSE，不覆盖真实742业务体。
- 结论前HEAD、实施树clean、dependency clean复核；只写主检出docs/plans/**，未触其他计划既有修改。

next: work，同一lang-776树对Response全setter修复共享/独占/早退矩阵并提交、刷新decision/verification/SD后独立复审。R9实现修复保留，不把这条尚未覆盖的同类路径简单登记为可延期债务。最终pass后再做统一合入门禁、Spec沉淀与清树。
