# PLAN-787 final review R8 — pass

2026-10-16；stage: review；review_scope: final；plan_revision: 1。
reviewer_identity: 当前Codex独立审查会话，延续R1–R7，未参与实施、未创建代理。仅审查、运行必要验证和写主检出证据/簿记；没有修改实施代码、提交、合并或发布。

## 基线与去重

- request id见 `request-id.txt`：PLAN787-ZCODE-turn_737321ee-43d4-4118-867a-580056b47e42-d17b94fecbe61d4f830d6c7b9e318c74548a4f6c；入口没有本提交/R8既有审查，本轮为唯一执行。
- worktree `D:/autostack/.wt/lang-787/auto-lang`；branch `plan-787-dev`；reviewed_commit **`d17b94fecbe61d4f830d6c7b9e318c74548a4f6c`**；previous `8e9c12c99baccc1f3905c798a8ff454d774000aa`。
- original planning base `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`；delivery/dependency audit base `7c0caa18c91ada8b4de3861cb4225919a001df17`（含上游租约修复，64919895d回绑）。主检出入口 `c0406feda7d2a00a01b59387b323ba4bcff6a0ed`，共享计划execution_done7/9。相对delivery base，787生产 `crates/` 与 `experimental/ac-core/src/` diff为空；不能把旧base以来其他主线计划的生产改动归给787。
- 本轮仅5文件：codec/store实现、永久cache测试、matrix和SD。store其它变更只是rustfmt；kind行为差异只补SourcePage=10。无依赖声明/锁/配置/语法输入变化。
- path依赖auto-atom/auto-ac-prototype/ac-artifact-probe同实施树，绑定当前HEAD。Spec/Design当前工作副本hash见 `spec-inputs.tsv`；清单/锁/脚本见 `baseline-hashes.tsv`；本轮源码见 `reviewed-source-hashes.tsv`；最终HEAD/clean见 `final-baseline.txt`。未覆盖主检出其他计划脏改。

## F787-R7-01 — CLOSED

旧问题：codec已注册SourcePage=10，store重复kind表漏10，真实工件按key/按id读均bad kind。

实际修复：codec `ChunkKind::from_u8` 提为pub(crate)，枚举表内容不变；store `from_code` 统一委派 `ChunkKind::from_u8(v).ok()`。0..9映射与原表逐项一致，10合法接通，11..255仍None；未知类型没有降为成功，hash/framing/lease/GC admission尾部不变。没有维护第二张容易漂移的表或调用侧绕过。

独立重放原R7反例 **1/1通过**：用真实put_source_pages生产，A01正控成功；SourcePage按key/按返回artifact id均返回kind=SourcePage、payload逐字节相等。见 `source-page-read.log`。探针Rust源与R7原例相同，仅package名隔离为plan787-review-r8，防止共享target误用旧可执行文件；实际用例名核为 `produced_source_page_is_readable_through_artifact_api`。

永久oracle `syntax_cache_source_page_artifact_reads_by_key_and_id` 实际执行通过：生产→drop旧store→新实例重开→两入口kind/字节/id断言；并保留source put三份驻留下界控制。没有以Err/miss当正控成功。

**无新增finding，原R7唯一阻断闭合。** R1–R6累计闭环按下列未变范围复用，不再回到旧例级补丁。

## 本轮独立验证

| 验证 | 结果 | 原始证据 |
|---|---|---|
| 原R7独立反例 | 1/1 pass | source-page-read.log |
| artifact-adapter syntax_cache_ | 10/10 pass | syntax_cache_.log |
| artifact-adapter syntax_admission_ | 18/18 pass | syntax_admission_.log |
| core-adapter compatibility_ | 3/3 pass | compatibility.log |
| probe --lib（codec/identity/schema） | 14/14 pass | probe-lib.log |
| probe admission_ | 7/7 pass | probe-admission_.log |
| probe store_boundary_ | 5/5 pass | probe-store_boundary_.log |
| 独立语法workspace cargo check | pass，仅既有shim-metadata warning | check.log |
| 全语法rs rustfmt + 本次codec/store rustfmt | pass | fmt.log / probe-fmt.log |
| git diff --check 8e9c12c99 HEAD | pass | diff-check.log |

计数为各命令实际结果，不能将不同集合相加当去重总数。没有新警告或遗留调试输出。未触发根整档/docs_gen，未执行merge closeout完整套件。

## 明示复用与本提交工作证据

`reuse-audit.tsv` 保存R7→当前的逐项hash核验：syntax workspace/adapter/probe的Cargo.toml、Cargo.lock、verify脚本、artifact adapter生产实现、query bin、syntax_budget_、compatibility_、probe reachability_均不变；runtime/grammar/core实现和语料不在5文件diff中。变化明确为codec/store解码委派、cache新增测试和报告，未宣称整个probe源码未变。

因此按范围复用R7独立证据：
- R1 7/7、R2 18/18、R3 8/8、R4 3/3、R5 4/4、R6校正后4/4；集合重叠。来源 `../review-r7/rN-replay.log`，R6必须用隔离包后核实名称的最终日志，初次误用R5缓存的日志不作证据。
- R7独立acceptance **16族/raw111**，包括Windows64MiB预算/读量/源重建上界；`../review-r7/acceptance.log`。这些检查的预算算法、输入、配置、受管计数/OS采样都未变。预算路径的grammar/manifest元数据读取为既有kind6/9，0..9的委派等价已逐项审查；SourcePage通过raw chunk路径复原的frame表也未变。本次新接通的kind10公开artifact读路径由独立反例和cache永久例重新验证。因此复用不覆盖未经验证的新路径。
- 租约/GC/跨进程的reachability_5/5，源文件未变、LEASE_SEQ实现仅格式化；`../review-r7/reachability.log`。

另已直接读回**当前提交工作阶段**日志：`work-acceptance.log`，16族ALL PASS、raw passed合计 **112**（族间重叠，非独立去重用例数），含cache10/admission18/budget2。它来自本轮已完成执行的真实stdout，不只读取总结；原路径和执行会话由调用方完成证据绑定，当前HEAD/SD/clean再独立核对。本日志是work证据，不冒称review亲自重跑全部112项；本轮不为一处kind表修复重复无关64MiB长时负载。

## Spec增量审核

- 源冻结稿 `spec-delta.md` SHA-256 **150cc0172f8a8779c099d84e10547f891026a4c744164d8f9e172e69e4d2a4f7**，与handoff一致。
- 审核定稿 **`reviewed-spec-delta.md`** SHA-256 **0b21d8f2d928f47b022c78924b08cb508b74af6dd589e940d11e2c9cf2ff6780**。merge沉淀采用该审核定稿。
- 唯一术语校准：SD-05旧“查询进程驻留上界=limit+一页”收窄为“名称窗口已物化字节量上界=limit+一页”；Vec容量/分配器/进程总体峰值由SD-06的受管/OS证据约束。R7已明确这是len而非整个进程物理峰值；此处固定该审核解释，避免将窗口长度误沉淀为进程RSS上限。没有改变支持面、8/16MiB受管与64MiB OS合同，也不需语义plan_revision递增；仅写审查附件，未改实施SD或canonical。
- SD-01/02：有限双profile/配置程序/版本与实验边界，有当前实现和矩阵依据；历史R4计数只作历史引用。
- SD-03：snapshot/坐标/页保活、无损/恢复、真实共享/增量与正常栈迭代范围由累积证据支持，不扩为无限资源/完整Auto语言。
- SD-04：auto-ac仅补实验导航，生产入口零切换。
- SD-05：本轮五kind公开artifact读取终于与单一frame表一致；持久化规范形、实际子树admission、零宽坐标、会话发布、required闭包及源查询范围按已有边界沉淀。
- SD-06：真实容量/同驻留、Windows期间采样、生成器分列、manifest实读与查询增量读界；parse Machine内部暂存仅OS可见等缺失项保持明示。
- supersedes/new组件为准确docs/specs路径；touched_goals=[]有仅实验/生产零切换说明，不虚构goal。canonical/ledger尚未变更。

## AC / 遗漏与债务扫描

| AC | 结果 | 当前依据 |
|---|---|---|
| AC-01 | pass | snapshot/edit/坐标/handle/代际；复用未变runtime与旧反例。 |
| AC-02 | pass | 有限双profile配置驱动、生成拒绝、scanner代际。 |
| AC-03 | pass | 无损/partial tree/strict对照/深度与正常栈；不扩生产语法。 |
| AC-04 | pass | 规定局部真实复用、200轮逐步fresh等价、迟到/取消；范围未缩减。 |
| AC-05 | pass | R7-01真实SourcePage双入口闭合；旧持久化/拒绝/预算/lease证据有效。 |
| AC-06 | pass | 兼容/生产零改动、必要性能证据与上述审核定稿SD齐全。 |
| AC-07 | pass | 最终独立复核，无剩余阻断，commit/SD/元数据绑定，收尾门禁显式pending。 |
| AC-08 | pending merge-owned | 未合入、未沉淀canonical/ledger、未归档清理。 |

没有发现遗漏、未授权延期、用sleep/吞错/错产物绕过的新workaround。D-787-01 probe put成本超线性、D-787-02 debug增量时延不优于fresh保留为既定非目标/后续候选，不把它们转换为本计划承诺。原R1–R7开放记录保留历史，本结论不复活旧错误，也不以pass掩盖仍待merge的操作。

## 结论与路由

**outcome: final pass；status: reviewed；current_step: 8/9。** T-01..08完成，仅T-09 pending。审核绑定d17b94fec与本报告双SD hash；新语义代码/依赖/测试输入变化使受影响证据失效。

next: 由merge阶段在冻结提交核验/执行**完整独立语法workspace + 受影响783独立套件**（计划closeout范围）、采用审核定稿沉淀Spec/索引/ledger、完成合流归档与wt-guard后清理；批量回归到期按技能判定。根整档/docs_gen本计划无触面，不追加。已跑的定向子集不等于全部merge-owned门禁已完成。

**reviewed不是交付完成，也不授权跳过合入门禁。** 本次请求仅审核，本轮不提交、不发布、不合并、不归档/移除worktree、不启动重复实施，也未向其他聊天发送消息。
