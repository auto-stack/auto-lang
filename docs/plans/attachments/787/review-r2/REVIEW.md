# PLAN-787 final review R2 — needs_fix

2026-10-16；stage: review；review_scope: final；plan_revision: 1。
reviewer_identity: 当前 Codex 会话，延续 R1 独立审查，未参与实施、未创建代理。

## 基线与执行证据

- worktree: `D:/autostack/.wt/lang-787/auto-lang`；branch: `plan-787-dev`。
- reviewed_commit: `afbacfbcc644d246a2bf3a0ae6185bbd9e6ca457`；base_commit: `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`；修复比较基线 `4eed69e67085eb150d30443d5c2f41d2530e34c6`。
- 主检出计划基线 HEAD `9e03287aaaedb2efd4ecfe9fcb2cd40f76747ca1`；计划仍标 executing、7/9，交接正文声称 execution_done/全部闭合；本次按明确复审请求检查实际 readiness，不据状态文字推定完成。
- 三项依赖 auto-atom/auto-ac-prototype/ac-artifact-probe 都解析到同一实施树，绑定 reviewed_commit；清单与锁文件 hash 见 `baseline-hashes.tsv`。
- 主检出当前 Spec/Design 输入 hash 见 `spec-inputs.tsv`。冻结拟议 SD 见 `spec-delta.md`，SHA-256 `05143F502E3BF0EC8E6952EBC41A6D4A6E790DCF270139A4111EEB3779AF981D`。未发布 canonical Specs/ledger。
- 实施入口与结束洁净/HEAD 复核见 `final-baseline.txt`；没有实施源码修复。

验证入口：原树 `./scripts/verify-ac-syntax-787.ps1 -Scope acceptance`（工作阶段所需矩阵，含 64MiB，非 merge 完整套件），**16 族全部通过（90 次通过记录，非独立用例数）**，结果见 `acceptance.log`。独立反例见 `probes/counterexamples.rs`、`counterexamples.log`：**18 项 = 7 pass / 11 fail / 0 ignored**；原 R1 七例均保持通过。该附件沿用 R1 探针的 package 名，路径/代码/锁文件属于 R2；依赖仍指向唯一原实施树。

`cargo check --manifest-path experimental/ac-syntax/Cargo.toml --workspace` 通过，本次新代码警告已清（只剩既有 shim-metadata:213 unused doc comment）；`git diff --check 4eed69e67 HEAD` 通过。对语法工作区所有 Rust 文件直接 `rustfmt --edition 2021 --check <files>` 仍失败，见 `check.log`、`diff-check.log`、`fmt.log`。

未跑根 cargo t/tv/tt/tb/tu/taa/tf/docs_gen，也未跑 closeout 全套或 merge。生产面 `git diff bf773ca9b HEAD -- crates/ experimental/ac-core/src/` 为空。

## R1 闭环审计

| R1 finding | 本轮结论 |
|---|---|
| F787-R1-01 活页丢失 | CLOSED：PageRef 持 Rc，旧压力例通过；活页可读及未引用页淘汰有永久测试。 |
| F787-R1-02 跨页坐标 | CLOSED：回溯真实行起点，旧跨页例通过；Unicode/CRLF 组合有新增断言。 |
| F787-R1-03 假局部扫描 | CLOSED（规定局部负载）：Scanner 流式收敛停扫，独立实读 49,152B≤64KiB，报告扫描 27B。两者口径有区分；根 EOF 全扫已移除。 |
| F787-R1-04 假页复用 | 真实共享块已实现；不变量 PARTIAL：块内不可达旧错误污染新树，见 R2-01。 |
| F787-R1-05 grammar/checkpoint | grammar 键隔离旧例 CLOSED；完整 checkpoint PARTIAL，嵌套注释深度被固定为 1，见 R2-06。 |
| F787-R1-06 required/生命周期 | 旧目录 GC 例 CLOSED；源页闭包、多文档 key、真实当前根/迟到路径仍失败，见 R2-02..04。 |
| F787-R1-07 admission | 旧 page=999 例可拒绝（原始附件亦因新必填 source_pages 字段缺失而提前拒绝，不能单靠这次旧例绿证明结构校验）；永久补全 schema 例有页号拒绝，但下界/环/流式身份仍失败，见 R2-05。 |
| F787-R1-08 budget | 生成器分进程、在线目录已改善；峰值计量与小查询 payload 仍未满足，见 R2-09。 |
| F787-R1-09 oracle/脚本 | closeout 参数已改正确数组，新增真实子进程查询与深持久化例；完整字段/诊断比较、查询范围及子进程截止仍缺，见 R2-10。 |
| F787-R1-10 health | 新警告/多数 Result 处理 CLOSED；格式未清，见 R2-11。 |

“旧例通过”与“完整不变量闭合”分开登记；以下不是新增需求或批准延期。

## 当前 findings

### F787-R2-01（P1，修复回归；续 R1-04）— 共享块的不可达旧错误污染已修好的树

AC-03/04；T-04/T-05。`runtime/src/cst/mod.rs:237 has_error_nodes` 扫描全部共享块节点，而不是当前根可达节点。反例 `repaired_declaration_has_no_unreachable_old_error_flags`：

```text
old: api bad(a: i32 -> i32;
     api good() -> i32;
     api more() -> i32;
     api end() -> i32;
edit: 在第一处 " ->" 前插入 ")"
```

增量与 fresh 的 reachable canonical 全等、fresh 无错误，但增量 has_error_nodes=true。旧块被其它合法声明共享采纳后，未嫁接的旧错误节点仍被全块扫描；core `project_strict:151` 使用同一拒绝谓词。须让当前树的错误/strict 判定只依赖当前可达内容，并永久覆盖错误修复、块内混合新旧声明与投影结果；不能因修复错误而一概取消规定局部共享收益。

### F787-R2-02（P1，真实发布路径缺口；续 R1-06/AC-04）— 迟到产物替换当前持久化根

AC-01/04/05；T-05/T-06。`adapters/artifacts/src/lib.rs:395..398 put_cst` 无条件清工程旧根并注册调用方的根；没有 document/revision 的发布授权检查。`late_persistent_publication_does_not_replace_current_root` 明确先完成 revision=1 再完成 revision=0，强制 GC 后 revision=1 miss。内存 VersionedPublish 屏障测试不能保护这个实际入口。另 `StreamingCstBuilder::finish:1473` 所有工程共用 `"stream"` 根，独立工程也会互相取消保活。须分离“存版本工件”和“带文档/修订判断的当前发布”，以实际根/诊断读入口做逆序屏障、跨工程与释放控制。移除/注册根分多次独立调用的失败/GC 窗口也须明确处理，不能靠忽略错误维持完成主张。

### F787-R2-03（P1，key 不完整；续 R1-06）— 第二份流式文档首块冲突

AC-05；T-06。`StreamingCstBuilder::push_chunk:1300` 在本块 snapshot hash 尚未加入 `snap_hash_parts` 时构造页 key；首块的 source-snapshot 永远是空前缀 hash。反例 `independent_streams_can_share_store`：同 store 先建 `api f() -> i32;`，再建 `note "second";`，第二次 push 首块报 Store::Conflict（page=0、同 grammar、同空前缀 hash，不同内容）。须使用完整适用的内容身份/正确流式临时身份及封存策略，验证不同首块、共享前缀后分叉、并行 producer 和重试，不把“新文档必须空 store”变成隐藏条件。

### F787-R2-04（P1，required 闭包仍缺；续 R1-06）— 当前流式树在 GC 后源丢失

AC-05；T-06。`push_chunk:1285` 丢弃 put_source_pages 返回的 artifact ids，finish 的 required edges 仅数据页+目录页；`source_pages` 的 chunk 字符串不参与 783 required 闭包。`stream_source_and_grammar_survive_current_root_gc` 在当前根已注册后强制 GC，manifest/目录仍能打开，但 source_bytes 返回 `source page 0 missing (CAS miss)`。没有旧版本替换，也不需要并发。流式路径亦没有真实 A02 checkpoint 工件，不能以 SD-05 所述 A03→A02→A01+源链解释其保活。须补全实际闭包、工程归属、跨进程恢复依赖，并对整个查询/源重建闭包及释放回收重放 GC。

### F787-R2-05（P1，admission 不完整；续 R1-07）— 有效内容封装仍能 panic、恢复环图或错身份

AC-05；T-06。三个独立真实读入口反例（经 put_artifact 正常 frame/hash 入库，不在 CAS hash 层提前失败）：

- `valid_manifest_ref_below_page_base_is_controlled`：页区间 `[0,1)`、`[1,2)`，声明 page=1/node=0；validate_manifest:769 在比较下界前执行 unsigned 相减，debug panic `attempt to subtract with overflow`。分页目录读取后的 decl_range:934 同样先减，需一致治理。
- `cyclic_page_is_rejected_before_tree_exposure`：1 节点、1 条自身边，所有计数/offset 均合法。decode_page:1139 只校验范围，decl_subtree 用 seen 去重后重新装回环边并返回 Ok。之后 canonical/outline 等无环树遍历可无限增长；本反例只断言应拒绝，刻意不调用无限遍历。
- `stream_manifest_identity_is_checked`：查询 key 是正确 requested-stream/grammar，payload 写 other-snapshot/other-grammar、相同长度；open_cst_stream:442 给 view_from_manifest 的 identity=None，返回 Ok，绕过普通 open_cst 的身份核对。

须在真实 load/query 入口统一验证身份、图无环/归属、span/field/kind 及目录上下界，受控错误并保留正常深树控制；不得把三条反例各自补 if 当作完整图不变量证明。

### F787-R2-06（P1，完整 scanner 状态未实现；续 R1-05）— checkpoint 是末 token 猜测

AC-02/03/05；T-03（相关 lexer 断言）/T-06。`derive_final_checkpoint:576` 根据最后 token 拼状态，未闭合块注释固定 depth=1（:622）、未闭合字符串固定 escape_pending=false（:602），mode 也由单 token 推测。`nested_unclosed_comment_checkpoint_preserves_depth` 输入 `/* outer /* inner`，持久化恢复 depth=1，实际应 2。现有往返测试只验证 EOF 普通 mode=0，因此不能证明可恢复 scanner。须保存扫描器实际产生的完整 checkpoint，覆盖嵌套跨页、尾反斜杠和模式变换及恢复后 token/fresh 对照，不能只序列化一套齐全字段。

### F787-R2-07（P1，原 AC-01 覆盖缺口）— 两个会话的 handle 指向不同树却相互可用

AC-01；T-02。`runtime/src/handle.rs:59` 每个 TreeSession 都从 next_id=1 开始，NodeHandle 只有 tree+node。`foreign_session_handle_is_rejected` 在两会话各 publish 一树后，把 A 的 handle 交 B.resolve，返回 B 的树而非 BadHandle。metadata 虽存 grammar/snapshot，却不参与 handle 的适用身份。须加入不可混淆的会话/树身份并在查询验证，补双向跨会话、同内容不同文档、旧 lease/热切换的 accepted/rejected 控制。

### F787-R2-08（P1，原 core 兼容/深度合同缺口）— 合法 core 深链被固定 2048 上限拒绝

AC-03；T-05。`adapters/core/src/lib.rs:452` 在递归 project_expr 以 depth>2048 返回 TooDeep。`supported_core_chain_projects_without_fixed_syntax_depth_cutoff` 用合法 `return 1 + 1 ...`（2200 个 +）：旧 core parser 接受、CST 无错，新 strict 投影拒绝 Some(TooDeep)。探针专用 64MiB 测试线程栈仅用于隔离语义上限，不声称正常栈已安全。计划要求 core 全部承诺面、无固定语言嵌套排除；CST 深度单测不覆盖投影递归。须迭代投影或按已约定资源预算受控处理，不能把合法 core 输入降为不支持；对平链/嵌套/块/调用及释放补全路径验证。

### F787-R2-09（P1，预算与查询实际量漏计；续 R1-08）— 单次目录读取约 2.97MB，峰值只统计目录

AC-05/06；T-06/T-07。本次正在运行的 64MiB 验收产生的第一张声明索引页，实测 framed bytes **2,968,982**，50,000 条；见 `budget-observation.md`。`decl_entry:876` 为任何一个声明读取并 JSON 解码整张索引页；debug_max_page_payload 只在 page() 的 CST 页统计，未统计该读取。因此“单子树≤512KiB/单页≤512KiB”断言未证明合同“小范围查询 payload≤512KiB”。重启查询 bin:36 又调用 source_bytes 整源重建，小样本不能证明大源按需恢复。

`managed_bytes:1252` 只统计常驻目录容器，touch_peak 在 push_chunk 末尾、parse/编码/payload/工作栈已释放后调用，finish 临时 JSON/edges/源页缓冲也不在该口径。OS private_bytes 每 16 个 chunk 返回后采样、查询/源重建阶段不采样，不能作为构建/查询峰值。须计真实读页总 payload（含索引/源/grammar）和所有同时驻留 staging/解码/工作栈、实际 OS 峰值，确保规定负载而非放宽预算。承认当前样本驻留较低不等于已证全部峰值。

### F787-R2-10（P2，oracle 及验证生命周期仍缺；续 R1-09）

AC-03/04/06；T-04/T-05/T-07。`canonical:299` 丢弃 for_each_child 的 field 参数，独立 `canonical_detects_field_mutation` 实际移除一条非空字段，规范串仍全等。200 次 replay:119 比较的只是诊断数量，非完整诊断/位置/消息；且每轮下一 base 用 fresh 生成（:147），不能证明连续使用 incremental 输出的编辑会话。须增加字段突变与同数量不同诊断负控，并使用前一轮实际增量工件驱动后续序列，完成原合同而非以 banner 宣称全字段等价。

新 `syntax_cache_:218` 和 `syntax_budget_:49` 仍裸 Command.output，无截止、超时 kill/wait，未复用计划要求的现有子进程设施；新进程覆盖仅查小源节点数/源长度，没有精确 outline/位置负控。须补可区分断言和受控失败/挂起反控；closeout 全套继续归 merge，不据参数修好声称已执行。

### F787-R2-11（P3，收尾 health；续 R1-10）

AC-07；T-07。编译新增警告已清，但直接 rustfmt check 仍在 grammar/tests/depth_.rs:3/65、grammar/tests/recovery_.rs 等产生 diff（`fmt.log`），与“全量干净”交接不符。只修本次新增/修改面并记录正确检查；不需改既有 shim-metadata。

## AC / SD 与路由

| AC | 结论 | 依据 |
|---|---|---|
| AC-01 | fail | R1 活页/坐标闭合；R2-07 handle 与 R2-02 发布仍失败。 |
| AC-02 | partial | 配置驱动与生成拒绝族通过；完整 scanner 状态/持久身份见 R2-06/05，未宣称全部配置组合完备。 |
| AC-03 | fail | 既有无损/恢复/投影 fixture、深 CST 持久化族通过；R2-01/08/10 不变量失败。 |
| AC-04 | fail | 局部真实停扫/共享进展成立；错误修复、真实迟到发布及全字段/连续序列仍失败。 |
| AC-05 | fail | R2-02..07/09 仍阻断。 |
| AC-06 | fail | 生产零切换/旧 API 控制成立；原始完整性能/I/O/内存与 SD 主张未闭合。 |
| AC-07 | fail | 记录已绑定新提交，仍有阻断，不勾 T-08。 |
| AC-08 | pending merge-owned | 新冻结提交 final pass 后再执行完整独立语法工作区+受影响 783 套件、Spec/索引/ledger、合入归档与 guard 清理。 |

SD-01/02 的有限实验范围与配置入口有证据；SD-03 的“共享”已真实落地但错误/handle/比较语义不完整，SD-05 required 闭包/完整 checkpoint/重启按需主张、SD-06 同预算峰值主张仍与本轮反例/计量冲突。SD-04 导航和 touched_goals=[] 的零生产改动说明已提供，不能替代 AC-06。本冻结稿不获通过，保留为修复对照，未沉淀 canonical。

**outcome: needs_fix**；仅保留已完成 T-01；T-03 的完整 scanner 状态合同受 R2-06 影响一并重开，重开 T-02..T-07，current_step=1/9；T-08/T-09 未完成。此为第二次 final 非通过，下一 handoff 应先提供根因与覆盖/设计调整表、所有既有反例重放及完整不变量证据，再进入最终复审；不能再次只报旧例全绿。

next: 原树 auto-plan-work 修复全部当前 finding（包括存储失败/根替换和补充 oracle），提交新基线后同树复审。本轮只审查及计划簿记，未修实现、未合入。

重放：在原树清空 RUSTC_WRAPPER、CARGO_TARGET_DIR 指向其 experimental/ac-syntax/target，执行 `cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r2/probes/Cargo.toml --test counterexamples -- --nocapture --test-threads=1`。`probes/Cargo.lock` 固定审查依赖。反例期望为正确行为，修复后应全部通过；不要直接改弱其期望。
