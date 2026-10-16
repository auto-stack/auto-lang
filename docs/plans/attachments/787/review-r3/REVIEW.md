# PLAN-787 final review R3 — needs_fix

2026-10-16；stage: review；review_scope: final；plan_revision: 1。
reviewer_identity: 当前 Codex 会话，延续 R1/R2 独立审查，未参与实施、未创建代理。

## 基线与验证

- 唯一实施树 `D:/autostack/.wt/lang-787/auto-lang`，branch `plan-787-dev`。
- reviewed_commit `f7ea3a9e93bb76e0187c10fa280f032baa0fbcbd`；base_commit `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`；前次受审 `afbacfbcc644d246a2bf3a0ae6185bbd9e6ca457`。
- 主检出起读 HEAD `579f6dec0941fb711664679a9cb8c29fa71bad20`，Spec 输入冻结时 HEAD `8b97d3c9bc9c86cb4d8ebde4a5d7856554e83b9c`；主检出其它计划推进不改变上述实施基线。共享计划起读 status=executing、7/9，handoff 正文声称 execution_done；按用户明确复审请求及实际 artifacts 验证 readiness。
- 三个 path 依赖 auto-atom/auto-ac-prototype/ac-artifact-probe 均位于原实施树、绑定 reviewed_commit；清单与锁文件见 `baseline-hashes.tsv`。主检出 Spec/Design SHA-256 见 `spec-inputs.tsv`。
- 冻结规范增量 `spec-delta.md` SHA-256 `23867BC72A7016E0FF485BC6090F81B5B8B54293529EA50D583742B0A2F5FFC1`，未获通过，未发布 canonical Specs/ledger。
- 实施树入口 clean，结束 HEAD/clean 见 `final-baseline.txt`；本轮无实施源码修复。

已核两次 final 非通过后的诊断表：work 明确记录旧例优先、字段/诊断/会话链 oracle 过弱、峰值漏计，并提出逐项不变量调整。该表存在，满足再次进入审查的前置记录；下面仍以实际行为判定闭合，不能把新叙述当作修复完成证据。

验证结果：

1. 原 R2 18 个反例 **18/18 pass**，完整输出 `r2-replay.log`。其中包含原 R1 七例；输入/期望保留。旧深链测试明确用 64MiB 线程栈，只证明原固定语义上限已移除，不能证明正常栈安全。
2. R3 独立补充 **8 项 = 1 pass / 7 fail / 0 ignored**，见 `counterexamples.log`、`probes/counterexamples.rs` 和 `probes/projection-normal-stack.rs`。深栈测试隔离在子进程并复用 783 `subproc::run_deadlined`、30 秒截止，正常 2MiB 栈的 32 项链作为成功控制。最后完整重放两条深反例均明确输出 stack overflow，退出 `-1073741571`（0xC00000FD），未超时；此前一次链反例触发 Windows 故障处理长尾，截止设施完成终止，不作为额外实现缺陷。
3. 本轮工作阶段 `./scripts/verify-ac-syntax-787.ps1 -Scope acceptance`（包括指定 64MiB 负载）**16 族全部通过（95 次通过记录，含重复模块编入，非独立用例总数）**，输出见 `acceptance.log`。完整独立套件/closeout 留给 merge，未执行根整档或 docs_gen。
4. 独立语法 workspace `cargo check` 通过；新增代码警告清零，只剩既有 shim-metadata:213 的 unused doc comment。对全语法 Rust 文件执行直接 rustfmt check 通过；`git diff --check afbacfbcc HEAD` 通过。见 `check.log`、`fmt.log`、`diff-check.log`。

## R2 闭合矩阵

| R2 finding | 旧例 / 完整不变量 |
|---|---|
| R2-01 共享块旧错误 | CLOSED：当前根可达遍历，旧错误修复例通过；无回退到全复制。 |
| R2-02 真实迟到发布 | 旧序列 CLOSED；完整身份/发布失败仍 PARTIAL：会话修订进入内容工件、跨会话比较及先删根后插入，见 R3-02/03。 |
| R2-03 空前缀 key | CLOSED（旧不同首块例）：当前块 hash 先入 parts；同 store 第二文档通过。失败重试/任意切块不据此宣称普适。 |
| R2-04 GC 丢源 | CLOSED（已完成当前根 required 闭包）：流式补源页 artifact+A01 边，旧强制 GC 后源/grammar 控制通过。发布间隙仍属 R3-03。 |
| R2-05 下界/环/身份 | 旧三例 CLOSED；完整 source span/owner admission PARTIAL，见 R3-05。 |
| R2-06 完整 checkpoint | 原嵌套深度例 CLOSED；跨行 token 起始行仍错，见 R3-04。仍由末 token 拼写另写算法还原，非 scanner 原生完整状态。 |
| R2-07 跨会话 handle | CLOSED：全局原子树实例 ID，旧双会话反例通过。 |
| R2-08 core 深度 | 固定 2048 拒绝旧例 CLOSED；正常栈全路径失败，见 R3-01。新永久测试以 64MiB 栈绕过原生递归风险，未迭代化。 |
| R2-09 预算/按需查询 | 4096 条目录页与 metered read 已落地；同时驻留峰值/OS 峰值/按需恢复仍 PARTIAL，见 R3-06。 |
| R2-10 oracle/子进程 | 字段写入 canonical、完整诊断比较、上轮 incremental 工件链已接线；子进程复用截止设施，精确 name/span/hash 控制增加。恢复 bin 仍全文源驻留，见 R3-06；深度永久例误把平铺语句称深块，见 R3-01。 |
| R2-11 health | CLOSED：本轮全语法 rs 格式、diff check 通过；新警告为零。 |

## 当前 findings

### F787-R3-01 / P1 — 删除深度检查后仍递归，合法输入使正常进程栈溢出

原合同缺口未闭合，续 F787-R2-08；AC-03；T-05。

`adapters/core/src/lib.rs:455 project_expr` 只把 depth>2048 换成 visited>4M，`project_expr_inner` 仍递归处理 Binary/括号/调用，`project_block→project_stmt→project_block` 仍递归。该访问预算不限制每层原生栈大小，触发预算前已可崩溃。

独立子进程两例：

- `projection_deep_chain_normal_stack`：合法 `return 1 + 1 ...`（2200 个 +），2MiB 线程栈；旧 core parser 接受，CST 无错，进入 strict 后 stack overflow/0xC00000FD。
- `projection_nested_blocks_normal_stack`：512 层 `if true {\n`，每层换行正确闭合；同样旧 parser 接受、CST 无错、strict 栈溢出。
- `projection_small_normal_stack_control`：32 项链同一栈、同一入口正常退出并 Drop。

原永久深度测试 `.stack_size(64<<20)` 仅隔离了旧语义 cutoff；所谓“深块”实际生成 2000 条平铺 let，未构造嵌套块；注释称深调用但没有生成调用链。须将投影/释放等实际递归路径改为显式工作栈/可配置资源预算，并以正常栈受截止子进程验证平链、真嵌套、调用与异常退出。不得靠统一增大线程栈或恢复固定语法上限关闭 finding。

### F787-R3-02 / P1 — 会话修订混入内容身份，重复文本冲突且新会话被误当迟到任务

修复回归/完整身份缺口，续 F787-R2-02；AC-01/04/05；T-02/T-06。

`put_cst:347` 把 snap.revision 写入 immutable CstManifest，但 `cst_key_base:520` 的 key 仍仅 source-content/grammar/parser。`publish_if_current:401` 直接比较工程历史根中的裸 revision，没有 Document/Session 身份。

- `identical_content_at_new_revision_is_reusable`：用同文本整段替换生成 revision=1，明确断言 SnapshotId 与 revision=0 相同，tokens/树相同；第二次 put_cst 在同 A03 manifest key 下返回 Store::Conflict，因为仅 manifest.revision 不同。正常编辑回到既有文本/跨文档同源共享均会遇到这一适用身份问题。
- `fresh_document_session_revision_zero_can_be_current`：旧 store 会话已发布 revision=1，关闭并重新打开同工程，新文档以合法 revision=0 发布；put 返回成功但新 artifact 不在当前 roots。新会话的 revision=0 不等同旧会话的迟到任务。

须真正分离内容工件与文档/会话发布元数据，给版本比较明确适用域，保存同内容可共享性质。补同文本不同 revision、重启/新文档、旧任务迟到、grammar 代际及两个方向的控制。不能简单把 revision 塞进内容 key 来掩盖应当共享的文本。

### F787-R3-03 / P1 — 根替换先删除后注册，注册失败丢失旧当前根

原出版生命周期缺口，续 F787-R2-02；AC-04/05；T-06。

`publish_if_current:418..421` 与 StreamingCstBuilder::finish 都把 remove_roots/register_root 作为两次独立操作，仲裁读取也未与替换原子化。返回错误不能恢复已经删除的旧根。

`failed_root_registration_preserves_previous_current_root` 使用**本探针自己的临时 store**，先真实发布旧版本，再以 SQLite trigger 对 cst-manifest 注册注入确定失败（没有改实施代码/真实用户库），调用新版 put_cst。确认错误分支实际发生后，root_artifacts 返回 `[]`，而应保留先前 current root。此时 GC 可把原当前版本也当不可达回收。旧顺序迟到例不覆盖发布失败/竞争窗口。

须让比较、替换和对应保活覆盖原子/可恢复发布过程，处理失败退出及 GC 竞争；对失败注入、旧根保留、成功替换和逆序并发作真实入口验证。修复方案若超出 Plan 对 probe 的加法接缝范围，先记录明确的合同调整，不私改 783 GC 算法。

### F787-R3-04 / P1 — 跨行未闭合 token 恢复后起始行错

原完整 scanner 状态缺口，续 F787-R2-06；AC-01/03/05；T-03/T-06。

`derive_final_checkpoint:623` 记录 EOF 当前行，但 `MidToken` 不含 token 原始起始行；`Scanner::next_token` 使用 checkpoint.line 作为恢复 token.line。独立 `multiline_checkpoint_resume_preserves_original_token_line`：

```text
base = note "first line\nsecond line
ext  =  closes";\napi f() -> i32;\n
```

fresh 字符串 token 范围 [5,36)、line=1；由持久 checkpoint 恢复的同 token 范围/内容相同，却 line=2，其后 token 才对齐。实际逐 token 断言失败；旧嵌套/尾转义正例的未闭 token 都在末行起始，测不出此差异。

须保存足以同时恢复“当前扫描位置”和“未闭 token 起点坐标/完整 matcher 状态”的实际状态，跨 LF/CRLF、跨页字符串及块注释恢复后与 fresh 的 token/诊断/位置全等。不能靠从 EOF 行反推任意 token 起始行的另一套不完整算法。

### F787-R3-05 / P1 — 有效 frame/hash 下越源 span 仍作为成功查询返回

原 admission 合同缺口，续 F787-R2-05；AC-05；T-06。

`validate_manifest:834` 只检查布局/引用编号，`decode_page` 只检查计数/边范围，`decl_range:1017` 未核对 source/span 上界。`valid_framed_out_of_source_span_is_rejected` 正常 put_artifact 入库合法 framed 页和身份匹配的 manifest：source total_len=1、合法 grammar rule kind、1 节点、0 边、节点 span=[9,10)。open_cst 后 decl_range 返回 Ok([9,10))。没有破坏 CAS hash，也不依赖索引越界。

须对声明 origin、节点相对 span、source total_len、父子/字段/type/ref/owner 适用关系做统一读取 admission，坏偏移/归属受控拒绝，不仅防 panic/环。补正反跨度边界、源不匹配和分页目录读取路径；不能让后续 outline 在读 source 时才发现已返回“成功”的错树。

### F787-R3-06 / P1 — 峰值统计仍没有覆盖同时驻留量，恢复小查询仍全文源驻留

证据/实现缺口延续 F787-R2-09/10；AC-05/06；T-06/T-07。

已有改善：声明目录由 50,000 条改为 4096，新增 read_chunk_metered。以下仍未闭合：

- `push_chunk:1421` 统计 parse 树+tokens 的估算后，:1433 只另算 payload；此时整个 out/tree/tokens 仍驻留，payload.clone/源页/工作栈/容器容量没有合计。managed_bytes 也漏 src_page_arts 等容器的实际堆内容。`write_decl_page` 生成 JSON 仍无相应 touch，finish 的瞬态即使 touch 也被测试提前在 finish **前**读取的 managed_peak（syntax_budget_:118）漏掉。
- OS 仍只在每 16 个 push 返回后、finish 后及全部查询/重建完成后取当前 private_bytes；“最后再采一遍”不是查询/构建内部峰值。该证据不能证明既定峰值上界，不能把终态低驻留报告为全阶段峰值。
- `debug_max_chunk_read` 是一次 chunk 最大值，不是小查询全部读取字节，且 open_cst/open_cst_stream 的完整 manifest 读取在视图创建前、不入计量。应计根 manifest/索引/数据/必需源的实际读页及总 payload，不能只断言单页≤512KiB。
- `ac787-query:56..68` 先流式遍历**全部源页**求 hash，又将**全部源页** append 到 page_bytes Vec，随后生成完整 Snapshot 才计算一个声明 outline。注释“零整源驻留”与这条实际路径冲突。小样本跨进程 name/span/hash 正例不证明 64MiB 源的按需、小查询和预算。

须使用真实同时驻留/分配容量口径及 OS 峰值采样，查询按范围取源，恢复后在原指定大负载上真正禁调 parser、小范围读取且无整源驻留。区分本轮预算断言通过与指标本身充分；这里不声称已观测到 OS 实数超过 64MiB，finding 是必验证据与实际查询路径未满足合同。

## AC、规范增量与路由

| AC | 结果 | 依据 |
|---|---|---|
| AC-01 | fail | 坐标/handle旧例保持；R3-02 会话修订与 R3-04 恢复坐标失败。 |
| AC-02 | partial | 两 grammar 配置/生成拒绝族通过；scanner 端到端完整状态未闭合。 |
| AC-03 | fail | 无损/恢复与旧 fixture 族通过；R3-01 正常栈崩溃、R3-04 token 坐标失败。 |
| AC-04 | fail | 真实局部停扫/内容共享/连续字段诊断链有进展，内存迟到和旧持久迟到例通过；真实发布适用身份与失败原子性未闭合。 |
| AC-05 | fail | R3-02..06。 |
| AC-06 | fail | 生产零切换保持，数据/时延/内存与 SD 尚非完整可验收证据。 |
| AC-07 | fail | 仍有阻断，不勾 T-08。 |
| AC-08 | pending merge-owned | 新冻结提交 final pass 后，完整独立语法+受影响783套件、Spec/索引/ledger、合入归档与 guard 清理。 |

规范增量审查：SD-01/02 的有限实验边界与配置入口有证据；SD-03 的修复共享、global handle、canonical field 可保留，但完整恢复坐标及深路径尚未满足；SD-05 把 revision 写进 manifest 作为发布仲裁的持久规范会沉淀本轮内容身份冲突/会话比较错误，须修；SD-06 同预算/峰值主张需实证；SD-04 导航及 touched_goals=[] 的零生产说明已提供。当前冻结稿整体不通过，未修改 canonical。

outcome: **needs_fix**。保留 T-01/T-04，重开 T-02/T-03/T-05/T-06/T-07，current_step=2/9；T-08/T-09 未完成。R2 诊断表存在但多项“设计调整”仍只覆盖旧例：递归改计数、内容 manifest 加 revision、采样末尾再读，不足以建立不变量。下一 handoff 应先提交这些机制的具体根因/覆盖与设计调整，再复审。

next: 原树 auto-plan-work 修复所有当前 finding，保留原反例和正确行为控制、补正常栈及真实发布/大源恢复路径的永久 oracle，提交后同树复审。本轮只审查和计划簿记，未修实现、未合入。

重放：在原树清空 RUSTC_WRAPPER、CARGO_TARGET_DIR 指向其 experimental/ac-syntax/target，执行 `cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r3/probes/Cargo.toml --test counterexamples -- --nocapture --test-threads=1`。探针清单指向唯一实施树；Cargo.lock 固定审查依赖。子进程栈溢出是预期失败证据，不要直接在主审查进程内复现。
