# PLAN-828 r2a 局部修复核验

2026-10-21，受审提交 `db295803a1f45c4ec0b7ac528dc16167e0c39cce`，基面 `791c62ab251b5d0f36d3d5610fade600760d2f52`，`plan-828-dev`，`D:/autostack/.wt/lang-828/auto-lang`。输入/二进制哈希见 [baseline.json.txt](baseline.json.txt)。reviewer为原R1/R2独立检查上下文Codex `/root`，未实施r2a。

**结论：原R2三失败例均通过，但F04与F06的完整不变式仍未闭合。** 这是用户交接后的局部修复核验，不是第三次最终送审；此前R1/R2记录及剩余任务保持。按用户要求，主检出合并/计划/债务表不写，证据和待回写记录存于828工作树。

## 实测

- 永久回归 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1`：**9/9 pass**，0 ignored，0.14s。
- 原suite `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1`：**17/17 pass**，0 ignored，10.45s；[raw](original-suite-output.txt)。
- 独立R2的11例原输入/断言原样重跑：**11/11 pass**；增加4项相邻反例后合计 **11 pass / 4 fail**，exit101，0.15s。[replay源码](replay.rs)、[runner](run-replay.ps1)、[完整输出](replay-output.txt)。
- `git diff --check 791c62ab2 HEAD`过；四个改动Rust文件的 `rustfmt --check --edition 2021 --config skip_children=true ...` **exit1**，测试文件13行的模式拆行仍不规范，见[raw](fmt-output.txt)。原有shim告警和session.context_revision dead_code仍在，不宣称零warning。
- 本轮未重复scalar差分/798兼容集，没有把用户交接总数当本轮独立执行；未运行整档或正式IPC/内存门。

全部运行cwd为828实施树。源码检查和测试期间HEAD固定、实施clean；随后仅新增本核验文档/重放材料，不修改Rust实现、不提交、不合入。

## F04：原位置反例闭，三个来源路径仍失败

本轮统一输入（真实源，非构造span）：

```text
fn helper() int { return 1 }
fn main() int { var x = helper(); return 1 / 0 }
fn tail() int { return 9 }
```

这些案例沿用AC-02/05/06、V-02/05/07及R1/R2原F04的来源/snapshot/复用合同，不增加语言或规模承诺。

| ID / replay | 真实操作与合法控制 | 实际失败 |
|---|---|---|
| F04-a `current_snapshot_public_execution_uses_its_source_revision` | 在helper插入13B `var pad = 2;\n`；`session.run()`与冷编均83..88。取同一个新 `session.current()` 后经公开 `execute_program` 执行 | 返回旧 `70..75`，应83..88 |
| F04-b `history_hit_preserves_mint_source_for_span_mapping` | 同样前部插入后，main的 `1/0→2/0→1/0`；确认真实reuse_hit>0；期望仍83..88 | `session.run()`退回70..75 |
| F04-c `semantic_noop_edit_inside_trapping_function_updates_source_map` | 仅在main的return前插入换行，确认真实内容命中；新源冷编71..76 | `session.run()`仍70..75 |

原因与修正方向：

1. `session.rs:574`只在session wrapper结束后映射，ProgramSnapshot没有不可变来源视图，公开execute_program、steps/resume及持有的新快照不受映射覆盖。映射应随所属snapshot/运行结果提供，不能仅靠当前可变session后处理。
2. `history`只存body/code，未保存代码的铸造来源；`commit:1167`对changed_idx无条件重置mint_origin，即使代码来自旧history。代码命中并没有重新铸造，更新基面会丢掉真实位移。
3. 仅声明origin差不能表示函数体内部token/行号变化。IR内容相同可以复用代码，但source map须对当前body位置独立生成/绑定，保持旧snapshot自己的来源，同时覆盖恢复和内容命中。

现有“代码payload不动”约束保持，不能靠复制改写所有代码span消除问题。下一次修复需重放原例和以上三路径，核对实际公开snapshot消费方及A→B→A。

## F06/F08：域与正常trace编辑原例闭，intrinsic依赖被整体排除过宽

F06的Scalar→Core拒绝及同域正控现通过。F08的trace在未改callee时修改caller、trace在changed函数内修改、无授权拒绝控制也通过；不复活这些已闭原例。

但 `verify.rs:380` 将所有intrinsic从声明上下文指纹中排除，且FnEnv没有按CheckedFunction实际读取记录其intrinsic依赖，导致以下反例（沿用AC-05、V-03/04、原F06上下文适用性及F10实际依赖合同）失败：

`credential_invalidates_when_referenced_intrinsic_changes` 使用真实scalar源 `helper→mark_a`、`main→helper`，trace授权下完整编译并铸造helper凭证；同环境current=true作为正控。只把该body实际引用的intrinsic声明symbol从 `hir.test.mark_a` 改为 `hir.test.mark_b`，断言确实只改1项；新bundle仍通过真实 `verify_scalar`，普通函数签名/domain/owner均不变。将原helper凭证交给新FnEnv，`credential_current`仍true，应因已读取的绑定事实变化而失效。

这不是要求所有函数因任意unused intrinsic变化而失效。未读取的body派生事实可排除全局接口身份，但读取该intrinsic的函数凭证必须绑定其实际签名、绑定/symbol及适用effect/授权版本事实；不能用整体不跟踪来修复caller的过度失效。r2a修复了原trace拒绝，却削弱了这条上下文检查，需要与函数查询/受检环境修复一起收口。

## 关闭范围与下一步

- F04：原例CLOSED，完整来源不变式OPEN（以上a/b/c）。
- F06：原外部字符串/错签名/跨域原例CLOSED；实际intrinsic依赖适用性OPEN。域+owner+当前指纹三项存在，不等于所有实际读取事实已绑定。
- F08：正常标量源trace caller编辑原例及对照现通过；相关指纹弱化由F06实际依赖项跟进。
- F03/07/13保持此前已闭结论，其余用户已列明开放任务仍开放；计划继续executing，未授最终pass。

按原树work路由处理F04三条实际消费/复用路径、intrinsic依赖及已承认剩余任务。下一次交接应区分“原例关闭”和“完整不变式关闭”，附当前诊断/覆盖映射。主检出合并完成后再核对最新主计划与本目录 `pending-bookkeeping.md`，不以工作树旧计划整文件覆盖主计划。
