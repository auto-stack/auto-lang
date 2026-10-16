# PLAN-787 final review R4 — needs_fix

2026-10-16；stage: review；review_scope: final；plan_revision: 1。
reviewer_identity: 当前 Codex 会话，延续 R1–R3 独立审查，未参与实施、未创建代理。

## 基线与证据

- worktree `D:/autostack/.wt/lang-787/auto-lang`；branch `plan-787-dev`。
- reviewed_commit `15cfeac977a3a87b35b18aa92e56bfe9097b7174`；base_commit `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`；上轮受审 `f7ea3a9e93bb76e0187c10fa280f032baa0fbcbd`。
- 主检出起读 HEAD `efa9686607a09504649b58e6189a3a8a5cd36229`；共享计划的诊断表及 R3 handoff 已读。前文/本轮消息称 execution_done，frontmatter 实际仍 executing、7/9；按明确送审请求审查实现，不据此判 pass。
- path 依赖 auto-atom/auto-ac-prototype/ac-artifact-probe 均指同一实施树；清单/锁版本 SHA-256 见 `baseline-hashes.tsv`。主检出当前 Spec/Design hash 见 `spec-inputs.tsv`（含并行文档更新的实际工作副本，不覆盖它们）。
- 冻结拟议规范 `spec-delta.md`，SHA-256 `CC3F04A2FB2E1DE8785B67D3D5E2FD119E07B15ECA9AD978A58BE7A2443ECEBD`；未获通过，未修改 canonical Specs/ledger。
- 实施入口与结束 HEAD/clean 绑定见 `final-baseline.txt`。本轮没有实施源码修改。

实际验证：

1. R1 **7/7**、R2 **18/18**、R3 **8/8** 全部独立重跑通过，见 `r1-replay.log`、`r2-replay.log`、`r3-replay.log`。这些集合有重叠，不将其相加为独立测试总数。
2. 新补充 **3 项全部失败**，见 `counterexamples.log`、`probes/counterexamples.rs`；每项包含区分性 accepted control。范围查询 probe 直接编译原 `ac787-query.rs`，不复制/修写其实现；子进程复用 783 run_deadlined，30 秒截止。
3. 工作阶段验收 `./scripts/verify-ac-syntax-787.ps1 -Scope acceptance`（含 64MiB）**16族全部通过，按各族 raw passed 求和为 99 次通过记录**，见 `acceptance.log`（过滤族有重叠，非独立测试去重总数）。用户 handoff 的98与本次实际输出不同，以本次日志为准。未执行 merge closeout 完整套件或根整档/docs_gen。
4. 独立语法 workspace cargo check 通过，新代码无编译警告（仅既有 shim-metadata:213）。`git diff --check 15cfeac97^ HEAD` 通过，生产 `crates/`、`experimental/ac-core/src/` diff 为空；直接全语法 rs rustfmt check 失败，只有 artifacts/src/lib.rs:820 多空行。见 `check.log`、`fmt.log`、`diff-check.log`。

三连败诊断更新已核：本轮确有实质机制改进，尤其投影迭代引擎、mid token 起始行、会话内仲裁与注册失败保旧根。以下明确区分已闭合内容与接缝/指标仍缺的部分。

## R3 六项闭环

| R3 finding | 本轮结论 |
|---|---|
| R3-01 栈溢出 | CLOSED：projection_engine.rs 显式 Frame/Val 栈，表达式/块/语句/调用无互相原生递归；正常 2MiB 栈的旧链/真嵌套例通过，新增永久调用链控制。 |
| R3-02 revision/内容身份 | CLOSED（原同文本/重开会话两例）：revision 已移出 CstManifest，同文本异 revision 幂等；每 SyntaxArtifactStore 会话的内存 revision 仲裁与本轮用户给出的会话域一致。另有不同 arena 布局的产物身份冲突，见 R4-01。 |
| R3-03 注册失败丢旧根 | CLOSED（既有确定性失败分支）：先 register，再定点 remove 旧 artifact；原 trigger 注入后旧根完好保留。没有据顺序测试声称任意并发跨会话均已验证。 |
| R3-04 跨行 checkpoint | CLOSED（原跨行 string token）：MidToken/codec 增 start_line，原恢复 token 行号/fresh 尾部一致。 |
| R3-05 越源 span | 根级 decl_range 的原反例 CLOSED；完整 subtree/child admission PARTIAL，见 R4-02。 |
| R3-06 预算/按需恢复 | 同驻留统计有改善，finish 后读峰值、manifest 实读分列、增量读取断言已接线；跨页源切片重定位回归见 R4-03，OS/构建瞬态峰值仍不完整见 R4-04。 |

## 当前 findings

### F787-R4-01 / P1 — fresh 与 incremental 的同一源快照持久化冲突

原 AC-04/05 集成合同缺口；T-06/T-07。

`adapters/artifacts/src/lib.rs:305 put_cst` 把内存块原样编码；:318 以 source-snapshot+grammar+parser+**块序号 page=i** 建不可变 artifact key。fresh arena 与增量共享 arena 的分配/布局不同，却共享同 key。

`fresh_and_incremental_artifacts_for_same_snapshot_can_coexist` 的真实路径：40 条 mini 声明，在 f20 名前插入 `new_`，分别产生同一 new Snapshot 的 fresh 和 incremental。先断言 canonical（含字段/跨度/flags）全等、确有 reused_decls，再写相同 A02、先 put fresh、再 put incremental。第二次在 A03 page=0 返回 Store::Conflict，两个 artifact id 不同。不是不同文本、grammar 或修订；失败正是计划明确允许忽略的分配 ID/布局差异进入了产物身份。

须让逻辑上等价的 fresh/增量产物可安全复用、恢复，采用确定的持久化规范形或有完整一致性证明的布局/内容映射。不要只吞 Conflict 或拿不匹配布局的旧页配新 manifest；补 fresh→incremental、incremental→fresh、重复编辑/重启与正确控制。改动归本语法 adapter，不私改 783 冲突保护。

### F787-R4-02 / P1 — 子树读取绕过子节点 source/parent span 校验

原 admission 不变量延续 F787-R3-05；AC-05；T-06。

R3 修复只在 decl_range 核根 span；`decl_subtree:1080` 检查引用/环后装回所有子节点，`decode_page:1299` 仍只校验计数、边范围，不验证完整 span/归属。

`child_span_outside_source_is_rejected_by_subtree_entry` 经正常 put_artifact 构造有效 frame/hash：源长=1，根为该 grammar 的合法 rule kind、span=[0,1)，只有一条不重复子边；孩子为合法 token kind，却 span=[9,10)。manifest/page/ref/grammar 身份及数量都合法。`decl_range(0)` 正确返回 (0,1)（证明旧入口控制通过），但 `decl_subtree(0)` 返回 Ok，越源/越父节点成功进入返回树。

须在实际 subtree load admission 上验证整棵可达子树的绝对跨度、父子约束、source 长度与归属/字段适用关系；不能让 caller 必须另调某个 helper 才拒绝坏树。补根合法而子错、跨页目录、非零 origin、父子边界的正反控制；不只对本例增加根检查。

### F787-R4-03 / P1 — 跨源页的合法前导 trivia 使范围查询坐标平移下溢

R3-06 修复回归；AC-05；T-06。

`ac787-query.rs:48` 只物化与**声明语义 span**相交的源页，:74 无条件 `d.origin -= slice_start`。声明 unit 的 origin 还覆盖前导 trivia，可以位于被排除的上一源页。

`range_query_handles_leading_trivia_across_source_page`：同一原程序/语法/有截止子进程，prefix=0 的 `api f() -> i32;` 成功且 name=f。随后用 `262154` 个空格 + 同声明（约256KiB，未扩大计划原有跨块/大源语义）真实流式持久化；观察 origin=0、semantic span=(262154,262169)，所选源页 slice_start=262144。原查询程序 debug 在 :74 unsigned subtract overflow，exit=101；未超时。release wrapping 也不会变成正确的坐标映射。

须让源切片与 unit origin/各类 trivia/错误节点的坐标合同一致，使用受校验的重定位/足够的源覆盖，并保持按需读取上界。不能以饱和减法或吞 outline 错误返回空名字冒充成功。补 LF/CRLF、Unicode、多页 trivia、非首声明与完整 outline/span 正反控制。

### F787-R4-04 / P2 — 同时驻留与 OS 峰值证据仍不充分

原必验计量缺口延续 R3-06；AC-05/06；T-06/T-07。

按本轮用户明确给出的**manifest 实读单列、单次查询增量读取≤512KiB**口径检查：新计数与断言已接线，不把 manifest 分列本身另立为缺陷。以下与该口径无关的峰值缺口仍在：

- OS private bytes 仍只在每16次 push 返回后、finish 后、query view **drop 之后**和全部源重建结束后采当前值（syntax_budget_.rs:109/117/123/200/258）。没有运行期间 sampler 或阶段内峰值；末尾再取 current 不能证明构建/查询内部最大驻留。当前17,396KiB等数字应标为所列采样点的最大观测增量，不能声称全阶段峰值已验收。
- transient_base 用 `nodes.len()*40 + tokens.len()*size_of` 估计，未计 Vec capacity、CstBlock 边/字段/目录、源页及 parse Machine 的构建瞬态。调用点在 parse_snapshot **返回后**，此前已经释放的工作栈/回溯暂存不在内。
- write_decl_page 统计 JSON+条目缓冲时未加仍在 push_chunk 中存活的 parse out/tree/tokens；put 的 payload.clone/编码 frame 同时驻留也未合计。finish 把页目录移入局部 manifest 后 managed_bytes 不再计它们，却仍与 JSON/edges 同驻留。固定每项*70 的索引堆估计不是实际容量和字串分配计量。

这条 finding **不声称已测到 OS 实数超过64MiB或总预算实际超限**；它指出原合同要求的构建/查询峰值与同时驻留口径尚无可靠证据。需补明确可复验的实际分配/容量与全阶段 OS 采样，分别报告生成器、runtime、warm/cold 查询、峰值和缺失项，禁止把常驻或返回点低值当峰值通过。

### F787-R4-05 / P2 — 冻结规范增量仍描述已删除的发布身份与旧目录配置

AC-06；T-07。

本轮冻结 `spec-delta.md` 的 SD-05 仍写“manifest 携带 revision，低 revision 不发布”，而本轮核心修复正是把 revision 移出内容工件、改成会话内存仲裁；仍写目录4096条/页，实际 DECL_INDEX_PAGE=2048。SD-06 同预算峰值结论未反映 R4-04 的实际验证边界。SD-01 的16族90项证据也与本轮报告不一致（总数须以 raw invocation 校正，不能据模板认 pass）。

须在 final pass 前按当前实现重写 SD-01..06 的 enduring rule、identity/格式/恢复/实验边界，绑定实际 Spec 目标现版本/hash和 AC；不得在 merge 时把旧的身份混淆规则重新沉淀为 canonical。supersedes/new paths 和 touched_goals=[] 的实验零生产说明已提供，不需要编造目标 ID。

### F787-R4-06 / P3 — 新改文件仍有格式漂移

AC-07；T-07。直接全语法 `rustfmt --edition 2021 --check <rs files>` 只在 `adapters/artifacts/src/lib.rs:820` 报一处多空行，见 `fmt.log`。按仓库 health gate 修正新改面；不要求改既有 shim-metadata 警告。审查阶段未顺手修改实现。

## AC / task / Spec 结论

| AC | 结果 | 覆盖边界 |
|---|---|---|
| AC-01 | pass | snapshot/edit/位置/handle/代际既有族及旧反例保持；非法持久化数据和源切片重定位由AC-05处理。 |
| AC-02 | pass | 有限双 profile 的配置驱动与生成拒绝正反控制保持，checkpoint原跨行例通过；不扩为任意grammar承诺。 |
| AC-03 | pass | 当前语法解析/恢复/无损、真实core投影及正常栈深链/嵌套/调用范围已成立；不把坏缓存入库看作原parser产生的树。 |
| AC-04 | pass（规定运行时范围） | 1MiB真实局部读取/解析/块共享、200轮连续增量全字段/诊断/重建、会话内迟到控制成立；持久化接缝R4-01不获AC-05通过。 |
| AC-05 | fail | 三项实际集成反例与峰值缺口，R4-01..04。 |
| AC-06 | fail | 生产零切换、旧API控制保持；性能/内存验证边界与拟议规范仍不完整。 |
| AC-07 | fail | 未清阻断，T-08不勾。 |
| AC-08 | pending merge-owned | 新冻结提交 final pass 后：完整独立语法workspace+受影响783套件、Spec/索引/ledger、合流归档/guard清理。 |

知识增量：SD-01/02 有限语法/依赖方向可由当前代码支持；SD-03 的真实共享/正常栈迭代投影值得保留，源恢复/归属还需修复证据；SD-04 导航无生产切换属实；SD-05/06 按上述 findings 重审。冻结稿整体不通过，未发布 canonical/ledger。

**outcome: needs_fix**；保留已完成 T-01..T-05，重开 T-06/T-07，current_step=5/9，T-08/T-09未完成。不要为了本轮 adapter 接缝缺口重开已独立证实的投影/lexer/runtime 实现。

next: 原树 auto-plan-work 修复全部当前 findings，保留三轮旧例、增加 fresh/incremental 真入库和所有实际 subtree/query入口的永久 oracle；修复提交后同树复审。预算证据须可判定，Spec稿须与实际版本一致。本轮无实现修复、无合入。

重放：原树 RUSTC_WRAPPER 为空、CARGO_TARGET_DIR 指向原 experimental/ac-syntax/target，执行 `cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r4/probes/Cargo.toml --test counterexamples -- --nocapture --test-threads=1`。Cargo.lock/源文件位于本附件；range-query 直接编译原 bin 源，查询失败 stdout/stderr 与正确 prefix=0 控制均持久记录。
