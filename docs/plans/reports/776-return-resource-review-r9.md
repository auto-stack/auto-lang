# PLAN-776 最终独立复审 R9（2026-10-14）

**outcome: needs_fix。** R8旧反例关闭：普通数字不再驱动，数组Future身份恢复，iterator独占输入可回收且slot清零。但消费协议重复释放共享引用，记录构造/字段读取仍遗漏Future身份。两项P1为原合同内阻断，不能以63/63旧专项总数替代完整不变量。

## 1. 基线

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施会话，延续R5..R8上下文；无子agent或额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；reviewed_commit: `5b8bffdef3889008d1cc84d85cfda35b01c1afa3`。
- base_commit: `5e21561d2c303a02319ff7a29e4607933e09a38f`；repair_diff_base: `5d4d29ab0fac4cb2b18f7a720d55eb5a18a02213`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: plan-776-dev；clean。
- dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r9/input-hashes.json)；SD SHA256=`5CF667636FDA371EB573369266058BB4263C1CE51DDB2F80B2A76AAC9D8CAAFA`，[冻结副本](../attachments/776/review-r9/reviewed-spec-delta.md)。
- 生产库重新build后链接探针，库hash/HEAD见`review-r9/baseline.txt`。本轮只做针对性验收、反例和源码/Spec审查；整档t/tv/th等遵循后置裁定，待最终pass后merge执行。不发布canonical/ledger，不修实现/合入/归档。

## 2. Findings

### F776-R9-01 / P1：StakeGuard不retain，显式release加guard析构造成共享引用双释放

影响AC02/05/06/07/08/09，T01/02/05/07/08，SD01/04/05/07。位置`vm/native.rs:5594-5611`（pop_iterator_receiver）、`:5650-5659`（next对象臂），对照同文件`StakeGuard::nv/Drop`。

`StakeGuard::nv`只记id，没有retain；其Drop已经`rc_release_id`一次。新协议先建立guard，又显式`vm.rc_release(nv)`，读后`drop(stake)`再次释放，净消费两份。独占RC1例中第二次释放零计数未暴露错误；RC2（调用参数一份+全局共享一份）才区分正确消费与误销共享owner。

V9-N1输入与R8同源：合法ListData[42]/Iterator::List id0，`push_sse_resource`建立参数owned槽；另`rc_retain_id`并写入vm.globals["kept"]建立全局共享份额（STORE_GLOBAL的copy-on-store模型）。只消费函数参数，预期RC2→1，全局凭据跨reap仍有效。实际真实enumerate调用：

```text
source credential rc before consumed argument=2
credential enumerate result=Ok(()) claim=Some(Sse { iterator_id: 2 })
source credential rc after consumed argument=0
source credential retained after output release/reap=false rc=0
receiver=(0, true) popped-slot-actually-zero=true original-tag-retained=false
```

源没有移交给派生凭据（派生只持底层iterator id），全局份额仍在；不应归零。功能与slot清零已修，但新消费协议破坏共享存活。源码明确next同样有两次释放；map/filter/enumerate/collect/reduce/find经过共享helper同受影响。

证据：[probe.rs](../attachments/776/review-r9/probe.rs) shared-native模式、[reproduce.ps1](../attachments/776/review-r9/reproduce.ps1)、`shared-native.log`；独占native模式为正控制，输入RC1消费后0并回收，旧R8泄漏关闭。

修复按现有owned协议**每个输入槽只消费一份**，不要把StakeGuard当retain保护；同时保证读取前对象有效、错误早退、派生输出份额、slot清零。补RC1与RC2+/shared-global对照、跨宽限窗replay、next及adapter/terminal代表例，不能只重复净零独占例。该项为R8泄漏修复引入的回归，非新的所有权要求。

### F776-R9-02 / P1：数组Future恢复，但CREATE_OBJ与记录字段读仍丢失身份

影响AC02/03/08/09，T01/02/03/04/07/08，SD01/02/03/05。位置`vm/engine.rs:4279-4349`（CREATE_OBJ逐tag转换）、`:7612-7710`（ObjectData/GenericInstance字段读取分派）。

R8修复已补CREATE_ARRAY/GET_ELEM及部分通用Value桥，漏了独立CREATE_OBJ转换；TAG_FUTURE既非object/string/bool/null/float，落最终整数payload臂，fid1被存成Int1。即使构造侧修为Value::Future，ObjectData/GenericInstance GET_FIELD匹配也缺Future臂，仍会落placeholder0，需要一起审计，不能只补本输入。

V9-F1真实parser/VM/HTTP合法输入（字段明确为Future<int>，不是将Future存入int字段）：

```auto
type Box { f Future<int> }
#[api(method = "GET", path = "/p")]
fn p() ~Iter<int> {
    let holder = Box { f: ~{ 42 } }
    return holder.f
}
```

期望与直接、helper、数组Future一样展开为200 JSON42；实际200 application/json、body1。输入未await，由HTTP同一TAG_FUTURE返回分派负责驱动，既有helper及array正例本轮均42。重复编译重放typed字段结果仍1。

证据`future-record.log`及probe future-record模式；同目录`future-array.log/future-helper.log/future-closure.log`为42正控制，`future-number-async.log/future-branch.log`为496普通数据负控制。Plan冻结合法容器/Future传播与返回来源不依赖声明，此反例检查已承诺的record载体，不新增语法或无限规模要求。

修复需列出所有NanoValue→Value→NanoValue入口/容器/字段/捕获路径，合并或补齐独立转换，避免逐opcode补旧例仍遗漏兄弟路径；保留普通整数/非Future字段控制及错误有界行为。若必要表示改造超批准最小范围，应显式needs_replan，不删传播承诺。

## 3. R8闭合与AC

| R8 ID | verdict |
|---|---|
| R8-01 | ~int普通496与数组真实Future原例关闭，closure本次by-reference代表例42；完整carrier不变量仍失败（R9-02） |
| R8-02 | 独占RC1/清零原例关闭，共享RC2被销毁（R9-01）；不能把净零视为消费协议正确 |

| AC | verdict | evidence/tasks |
|---|---|---|
| AC01 | pass（现有资源碰撞范围） | T02/03/07，63族既有数字控制保持 |
| AC02 | fail | T01/02/07，shared ownership及typed record carrier |
| AC03 | fail | T03/04/07，typed record Future错误终值1 |
| AC04 | partial | T04，旧Failed/Nil/取消代表例通过，新增转换路径未闭合 |
| AC05 | partial | T02，旧owner/stale/id耗尽及Future位数字控制通过；共享引用被误回收后仍有失效持有，不能泛化完整安全 |
| AC06 | fail | T05/07，合法全局共享凭据跨reap失效 |
| AC07 | partial | T06，既有来源分类保持，native功能已修但owned契约回归 |
| AC08 | partial | T02/06/07，非HTTP消费共享回归；生成证据沿用明确边界，不覆盖本次RC修复 |
| AC09 | fail | T08，完整矩阵漏共享/record兄弟路径，SD消费/载体规则与实际不一致 |

重开T01/02/03/04/05/07/08，T06来源分类已证子项保留，current_step=1/8。历史通过子项及finding不删除，plan_revision1合同不改。supersedes/new/touched_goals元数据保持原路径及GOAL003/016。冻结SD不通过：新“消费所有权协议”与一次消费义务矛盾，“载体保真”尚不覆盖record。待实现与oracle修复后更新decision/verification/SD，不发布canonical/live ledger。

## 4. actual evidence及next

- `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1 --no-fail-fast`：**63/63**，31.385s；不从旧例总数推出新增共享/record不变量。
- `cargo build -p auto-lang --lib`成功，库/commit hash保存在baseline.txt。
- 独立probe旧R5..R8例重放通过（direct/local/helper/Internal/int-helper SSE，alias/Future-array/closure，496负控制，独占消费及slot清零）；V9-N1与V9-F1按实际期望失败。probe exit0只是观测成功，不是acceptance pass。
- 七个修复触面Rust文件`rustfmt --check --edition2021 --config skip_children=true`、`git diff --check 5d4d29ab0..HEAD`：通过。
- 不运行整档t/tv/th/tt/tu/taa；当前尚无final pass。后置merge-owned适用日常/VM/HTTP等门禁pending，tf为主检出batch-only。未重复未改的生成fixture；R7/R8真实生成证据仍仅覆盖其记录范围，不能证明新共享RC语义。
- 结论前HEAD仍`5b8bffdef3889008d1cc84d85cfda35b01c1afa3`，实施树/auto-down clean，主检出其他计划变更未触碰。仅写docs/plans/**，未修代码/合入/归档。

next: work，同一lang-776树修复两项P1、提交。就绪包必须给共享/独占/早退/派生链的所有权矩阵与完整record/list/capture/value往返入口闭合表，解释此前为何遗漏并用具有区分能力的断言验证；随后独立最终复审，真正pass后再做merge-owned门禁/Spec沉淀/清树。
