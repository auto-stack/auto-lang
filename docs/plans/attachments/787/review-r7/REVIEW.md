# PLAN-787 final review R7 — needs_fix

2026-10-16；stage: review；review_scope: final；plan_revision: 1。
reviewer_identity: 当前Codex独立审查会话，延续R1–R6，未参与实施、未创建代理；本轮只运行验证和写主检出复审证据/簿记，没有修实现。

## 基线与范围

- worktree `D:/autostack/.wt/lang-787/auto-lang`，branch `plan-787-dev`；reviewed_commit `8e9c12c99baccc1f3905c798a8ff454d774000aa`；previous `462b27e5a011160f56b6944189708b110828b08b`。
- 原规划base `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`；此次依赖/合流审计base **`7c0caa18c91ada8b4de3861cb4225919a001df17`**，在分支经merge `64919895d`回绑。主检出起读 `10029a884a65021ff236a27ae7b67b1d3a31fd45`。重新锚定合理：旧base到HEAD的大量生产改动来自已合入主线的其他计划，不归给787；相对7c0caa18c，787的 `crates/`、`experimental/ac-core/src/` diff为空。
- 本轮repair commit自身相对64919895d仅6文件：artifact adapter/query/测试、compatibility基面、matrix/SD；上游LEASE_SEQ由独立fix-783-lease-id落地主线，非787私修。root/lease OS保活语义未变。
- path依赖auto-atom/auto-ac-prototype/ac-artifact-probe均同实施树，绑定受审提交；清单/锁/脚本hash见 `baseline-hashes.tsv`，当前Spec/Design工作副本hash见 `spec-inputs.tsv`，关键代码hash见 `reviewed-source-hashes.tsv`。保留主检出其他任务脏改。
- 冻结拟议SD `spec-delta.md` SHA-256 `957c18bb1943181161aa54ac4a0080a5ef3dd66118d50f2f0faeb9dbe0005fb7`；整体未获通过，未发布canonical/ledger。入口与结束HEAD/clean见 `final-baseline.txt`。
- 已核根因/覆盖调整和work readiness；本轮已补报64MiB工作阶段证据，无上轮已知后置缺口，故执行完整定向acceptance独立复验。

## 实际验证与旧项闭环

1. R1 **7/7**、R2 **18/18**、R3 **8/8**、R4 **3/3**、R5 **4/4**、R6 **4/4**均重放通过，见 `rN-replay.log`。集合重叠，不相加为独立总数；R5覆盖136文件/144声明恢复0拒。
2. **探针身份修正**：初次R6 replay错误复用了共享target下同package name/version/target的R5可执行文件，`--list`发现4项名字不符。该次无效证据保留 `r6-first-run-identity-mismatch.log`，不用于判定。R1–R5实际用例名均匹配；R6原Rust源逐字节复制到本附件probes，仅package名改为plan787-review-r7-r6隔离产物，重新编译并核实四项名称后通过。详见 `probe-identity-note.txt`。审查未用错用例总数替代证据，也未把这个审查工具问题记为实现finding。
3. 上游reachability_ **5/5**独立通过，包括128次连续租约、OS真值/释放回收、跨进程lease pinning，见 `reachability.log`。没有用sleep/SQL错误吞掉来转绿。
4. 独立workspace cargo check通过（仅既有shim-metadata warning）；全语法Rust rustfmt通过；repair diff-check通过，见 `check.log` / `fmt.log` / `diff-check.log`。
5. 完整工作阶段acceptance含64MiB **16族全部通过，raw passed合计111次**（过滤有重叠、不是独立用例去重总数），见 `acceptance.log`；零ignored/skip，脚本exit0。新计量builder≤8MiB、视图hot≤8MiB/合计≤16MiB、Windows阶段内OS增量≤64MiB及单查询增量读取≤512KiB断言通过。独立审查验证这些上界；matrix的8027KiB/17176KiB精确数值是work报告值，脚本成功时不输出测试内具体读数，不将其冒称独立重测精确值。未运行根整档/docs_gen或merge closeout完整工作区/783套件。
6. 新增独立反例 **1失败**：真实生产者正常写出SourcePage后，按key与按artifact id两种公开读入口均返回bad kind；A01正常生产/按id读取正控通过。见 `source-page-read.log`、`probes/source-page-read.rs`。probe直接依赖受审实施树，未修改实现。

| R6 finding | 本轮闭环 |
|---|---|
| R6-01 三份驻留漏计/缺工作证据 | CLOSED：source put三份口径已修。256KiB页reported **786496B ≥ 786480B**独立下界；工作补报64MiB，review在此冻结提交独立acceptance（含64MiB）通过，新计量上界与未放宽合同成立。 |
| R6-02 先物化再判界 | CLOSED：append前判断已有window长度，允许至多一256KiB源页超前；8MiB名称默认2MiB限额时，实际物化 **2359296B** 即exit4，未全物化。小名称正常，既有跨页/Unicode/trivia与永久小limit控制保持。此数字是逻辑已物化字节；Vec capacity/分配器开销不等于该len，不能把它推广成任意精确分配峰值；总工作集仍由预算验证承担。 |
| R6-03 租约毫秒碰撞 | CLOSED：主线7c0caa18c的process-global LEASE_SEQ已入本树；原128次申请/立即释放反例 **0失败**，五项生命周期控制通过。 |

## F787-R7-01 / P2 — SourcePage写入成功，但产物读取的kind映射遗漏10

AC-05（真实A02源页生产/缓存接入与读取）、AC-06兼容接缝，T-06/T-07。**原接入合同缺口，首次在本轮补充发现；不把它归因为此次LEASE_SEQ或window修复回归。**

`experimental/ac-artifact-probe/src/codec.rs` 已注册 `SourcePage=10`，codec解码也识别10；但 `store/mod.rs::ChunkKind::from_code` 的数据库kind映射只加入6/7/8/9，随后落入None。`get_artifact`和787新增 `get_artifact_by_id` 共用此映射，在读取有效SourcePage元数据时，尚未进入payload校验就返回 `StoreError::Internal("bad kind")`。

独立反例使用真实 `SyntaxArtifactStore::put_source_pages` 写 `api f() -> i32;\n`，得到一个合法artifact id；同store的正常A01 grammar按id读取成功。对source：
- 用生产者返回的artifact id调用 `sas.inner().get_artifact_by_id` → **Err(Internal("bad kind"))**。
- 按生产者同一 `domain_hash("src-page", bytes)` / A02 `page-hash` key调用原有 `get_artifact` → **同样Err**。

这不是坏缓存应拒绝或未来新接口需求，而是已登记、已成功落盘的合法codec工件在原有公开cache读取路径不可读。**当前CstView源重建经原始chunk读取/codec解码另一路可工作，不能据本finding声称所有源码重建失效。** 也不能用这条可工作的raw路径替代通用产物API完整性。

修复要求：补齐/统一数据库与frame层的kind解码，避免重复枚举表漂移；给真实SourcePage生产→重开→按key/按id读取加永久正控，并保持旧kind与未知字节的拒绝边界。不吞bad kind为miss、不绕开hash/lease/GC admission。本缺口是787新增SourcePage接入的映射遗漏，属于允许的加法扩展修复，不能作为783原缺陷外推延期。

## AC / Spec / 状态

| AC | 结果 | 证据边界 |
|---|---|---|
| AC-01 | pass | 快照/坐标/handle/代际旧反例、矩阵。 |
| AC-02 | pass | 有限双profile、生成拒绝、checkpoint旧例。 |
| AC-03 | pass | 无损/恢复/strict/depth与原正常栈反例。 |
| AC-04 | pass | 局部复用/连续编辑/迟到，fresh与增量入库旧例。 |
| AC-05 | fail | 既有主路径/租约/预算按证据承认；SourcePage通用产物读入口未闭合。 |
| AC-06 | partial | 生产范围审计与旧kind控制通过；SD-05/06声称五种新kind完整接入仍受映射遗漏影响。 |
| AC-07 | fail | 新P2未闭合，不能勾T-08。 |
| AC-08 | pending merge-owned | final pass后冻结提交完整独立语法workspace+受影响783套件、SD/索引/ledger、合流归档/guard清理。 |

SD元数据路径准确；touched_goals=[]的仅实验/生产零切换说明成立。SD-01/02/03/04的有限语法/runtime/导航规则保留；SD-05实际5kind读写接入须补实，SD-06预算结论以本提交证据为限。历史R4测试计数等标为历史、不当本轮新增保证。冻结稿整体不批准，未发布canonical。

**outcome: needs_fix；status: executing 5/9。** 保留T-01..05，仅重开T-06/T-07，T-08/T-09未完成；旧项修复全部成果保留，不批准把新接缝缺口延期。next: 原树auto-plan-work补kind映射和真实产物读取永久oracle，提交后按受影响范围复审；其它未改代码/依赖/测试配置的证据可按hash明确复用，无需每修一行重跑无关长时负载。本轮无实现修复/合入。

复现新例：原worktree设RUSTC_WRAPPER为空、CARGO_TARGET_DIR=experimental/ac-syntax/target，运行 `cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r7/probes/Cargo.toml --test source-page-read -- --nocapture --test-threads=1`。R6校正重放用同manifest的 `--test counterexamples`。源/锁/hash在本附件，真实query bin直接指受审源码，子进程复用probe run_deadlined。
