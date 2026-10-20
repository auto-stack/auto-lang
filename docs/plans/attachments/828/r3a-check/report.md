# PLAN-828 r3a 修复核查

2026-10-21，**needs_fix**。本轮按最终送审就绪及受影响修复核查处理；主计划仍为`executing 0/11`，work最新记录明确`in_progress`并承认F01/02/05b/09/10/11/12/14未完成，所以未重新启动完整最终验收。此前R1/R2与r2a局部核查历史全部保留。

受审 `bb2380908ea7873be922f67288cf3aba39f82fc5`，修复基面 `db295803a1f45c4ec0b7ac528dc16167e0c39cce`；实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev`。reviewer为同一独立检查上下文Codex `/root`，未实施修复。前后HEAD固定、实施clean；只新增本目录核查文档，没有改实现、合入或处理主检出的其他工作。[baseline](baseline.json.txt)绑定源文件、锁、测试二进制和重放输入。

## 执行

所有测试cwd为上述实施树，原例输入和期望没有减弱。

| 验证 | 结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 13/13通过，0 ignored，0.11s |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | 17/17通过，0 ignored，9.27s；[raw](original-suite-output.txt) |
| 独立上一轮15例+本轮5个相邻控制 | **15通过/5失败**，exit101，0.14s；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 改动六个Rust文件 `rustfmt --check --edition 2021 --config skip_children=true ...` | exit0 |
| `git diff --check db295803a HEAD` | exit2：新提交的两份旧测试日志末尾有多余空行；[raw](diff-check-output.txt)，非Rust实现缺陷 |

独立上一轮4项失败例均变绿，其余旧控制保持绿；新增5例失败分别属于F04三条来源规则、F06两条真实依赖规则。不能把永久13例和独立15例重叠部分相加当覆盖总数。未重复scalar差分或798全兼容集，不将work交接结果冒充本轮执行。session.context_revision dead_code及既有shim告警仍在；未运行整档t/tv/tt/tb/tu/taa/tf、docs_gen或正式IPC/内存门。

## F04：原例关闭，快照映射仍不完整（P1）

适用AC-02/05/06、V-02/05/07，沿用原F04及上轮a/b/c来源合同。全是同一小型合法scalar程序的正常编辑，不增加规模/语言要求。

| 子例 | 区分性操作与控制 | 实际 / 冷编期望 |
|---|---|---|
| F04-d `public_snapshot_preserves_body_local_source_refinement` | main的return前插入换行；session.run与冷编均71..76。再取该修订current snapshot，经公开execute_program执行 | **70..75 / 71..76** |
| F04-e `successive_body_local_edits_compose_source_mapping` | 在同一main的return前连续两次插入换行，每次对照新源冷编；第一轮通过 | 第二轮 **71..76 / 72..77** |
| F04-f `whitespace_inside_trap_range_updates_end_independently` | 在`1 / 0`内部增加一个空格，断言真实reuse_hit；冷编start不变、end+1 | **70..75 / 70..76** |

根因可以从当前真实路径复核：

- `ProgramSnapshot`目前仅保存每函数整体`span_shifts`；函数体内`body_shift`仍在可变FnRecord，只有`session.run`二次处理。公开execute_program/steps/resume虽统一调用map_span，却只映射整体origin，没有body refinement；持有的新snapshot因此仍与session结果不一致。
- `commit`每次内容命中只保存 `diff_region(old_src, new_src)` 的一个threshold/delta，覆盖上次修订映射，没有从实际代码铸造来源组合到当前源。连续两次各+1变成永远只+1。
- session refinement只在`sp.start >= threshold`时同时平移start/end。编辑落在区间内部时，start应不变而end应更新，该判定无法表达。

因此原三个修补点各自成立，不表示一个可组合的source-map已经存在。修复应让**所属snapshot持有从实际代码来源到该修订的不可变完整映射**，公开执行与session用同一消费门；跨多次编辑组合，区间两端独立映射，内容命中保留其真正铸造基面。范围仍包括旧snapshot/暂停恢复和恢复消费；本轮未将这些尚未充分验证的组合标pass，也不能靠改写全部代码payload绕过对象不变合同。

## F06：旧symbol变更例关闭，绑定身份和effect仍未跟踪（P1）

适用AC-05/08、V-03/04及原F06/F10“绑定所有实际读取事实”合同。r3a将body读到的intrinsic保存为`(symbol, signature-shape)`，FnEnv按`symbol -> signature-shape`查找；原单个mark_a→mark_b例现正确拒绝。以下完整输入门正控已执行，失败不来自无效fixture或未发生修改。

### F06-b：两个intrinsic交换实际引用绑定仍接受

`credential_tracks_each_intrinsic_reference_not_just_symbol_set`：真实源码helper调用`mark_a()+mark_b()`，trace授权下编译并铸造helper凭证，原环境current=true。仅交换两个已声明intrinsic的symbol（恰好2次修改），保持其decl id/调用引用及普通函数签名；新bundle经`verify_scalar`仍合法且类型正确。新FnEnv仍判旧凭证current=true，应拒绝：原来的每条callee引用现在分别绑定另一observable intrinsic，调用顺序/效果绑定改变，符号集合相同不等于绑定相同。

### F06-c：实际引用的intrinsic effect变化仍接受

`credential_tracks_effect_fact_of_consulted_intrinsic`：helper只调用mark_a；原环境正控通过。只翻转其声明effect（恰好1次修改），symbol/signature不变，新bundle仍通过完整verify_scalar。新FnEnv仍接受旧凭证。当前intrinsic facts没有effect字段，不能验证计划§5.1/5.3规定的effect/授权等实际消费条件。

修正方向是按受检owner下的稳定声明/引用身份记录实际读取事实，包含该绑定的symbol、完整适用签名、effect及适用授权/版本轴。未读取intrinsic可以不导致caller过度失效；但不能把“一个symbol仍在环境任意位置存在”当作“该凭证实际读取的callee绑定仍相同”。需要保留原签名/域/owner正反控，以及本轮交换和effect两个负控。

## 当前关闭范围与后续

- r2a-check的F04-a/b/c、F06单个intrinsic替换原例现均通过；不复活其原始输入失败。
- F04-d/e/f和F06-b/c是相同不变式的未闭路径；**F04与F06仍OPEN/PARTIAL，不授完整关闭**。
- F03/07/13及正常trace caller正反控保持已验证结论；其他已承认未完成任务不变，计划继续executing。AC09本轮不通过，AC10的merge-owned门继续pending，SD01..06仍待完整实现/具体文本审查，不改canonical或ledger。
- 前两次最终送审后的诊断/覆盖调整要求继续适用。现在已有清楚根因：单次diff补丁不能表达多修订区间映射；符号集合不能表达实际引用依赖。下一次work应先提供这两条完整表示与消费路径的诊断/调整，完成相关实现和所有既有/新增控制，再声称就绪；不要再仅以原例逐个转绿宣布完整不变式关闭。

按此前用户约束，报告和待回写收据放在828工作树；主检出计划/债务表未修改，不以工作树旧计划覆盖主计划。未来合并收口时核对最新主计划，再按`pending-bookkeeping.md`补记；无整档/合入/归档/清理动作。
