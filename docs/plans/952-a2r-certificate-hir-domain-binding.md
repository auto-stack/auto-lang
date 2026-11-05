---
plan_id: PLAN-952
status: reviewed
feature_name: Rust证书绑定实际HIR域
plan_revision: 1
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
supersedes_spec_components: [docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-test-case]
depends_on: [PLAN-892, PLAN-902, PLAN-904]
current_step: 4
total_steps: 6
---

# [PLAN-952] Rust证书绑定实际HIR域

## 0. 变更摘要

独立关闭941真实认证发现F-941-02：Rust证书通过字符串裁剪重建HIR域，`auto.source.scalar-i32.draft`等落入default core；同一actual的manifest标量域与证书core域矛盾。直接绑定既有EmitOutcome.hir_domain，删除第二套错误映射；不在941消费者宽容坏身份。

## 1. 目标

所有已支持core/scalar/record/text/text-record/collections profile的证书HIR域与实际编译bundle/manifest一致；保持原schema、oracle、完整actual/reference、工具、配置、预算与daily/cert分轴。

## 2. 架构方案

现source→实际compile/emit的EmitOutcome已有hir_domain；Rust certify与C一样消费该字段。移除私有domain_of副本。新回归放原ace-a2x-worker conformance target，真实worker认证+解析cert与manifest，并与事前指定的profile域逐值比较；不新增runner、编译器或store。

## 3. 技术栈

Rust；`experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs`、新`tests/cases/cert_domain_952.rs`及原conformance注册；既有serde_json、原Job/deadline编译运行。原固定完整组`cargo t ace-a2x`。

## 4. 需求分析与背景调查

用户2026-11-06明确答复“授权独立修复并继续（推荐）”：最小producer域绑定修复、各profile回归、复审合入后继续941，全链授权继承，不再次确认。新旧计划不互替；941仍0/15。main基线c4282e663（含951）；941检查点55c8b37ea的真实scalar cert为core-i32-draft，而同场manifest为scalar-i32-draft；代码domain_of误识别带-i32 profile。文档取材`docs/specs/auto-test-case/a2r-hir-certification.md`（严格身份/隔离通道）与实际bundle field；C直接消费out.hir_domain为合法正控。当前main只有其他会话文档WIP，零代码WIP，不纳本包。

唯一实施树`D:/autostack/.wt/lang-952/auto-lang`、branch `plan-952-dev`，不使用941树实现本包。唯一材料目录`D:/autostack/plan-artifacts/plan-952/`，登记owner，在途保护，merge全删；跨仓只读auto-down组兄弟。无子agent/chat授权，复审由同会话重新读取冻结合同/diff/证据独立执行，明确上下文限制。

## 5. 详细设计

不再把view profile名称当HIR域解析规则；`CertDraft.hir_domain = out.hir_domain.clone()`。日常emitter/manifest/能力/profile dispatch不改。各profile的事前域依次core-i32-draft/scalar-i32-draft/record-i32-draft/text-i32-draft/text-record-i32-draft/collections-draft，由现profile源编译事实与typed emitter核查，若实际明确枚举不同以正式源事实记录，不凭测试输出自创expected。

### 规范增量

| delta_id | action | target | before/after | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 明确证书profile/HIR域须来自同一实际compiled bundle并与manifest相等，禁止从profile字符串另造域 | AC-01/02 |

## 6. 测试设计

V-01=真实worker各profile实际actual+手写reference认证，标准JSON解析cert/manifest，事前域与双件一致；修前scalar应红，core/record/text/text-record/collections正控。V-02=原完整ace-a2x冻结组（merge owner），不减样本/ignore/预算；C与951序列化控制保留。V-03=941严格Q4实际source及Rust/C独立reference重放（941自己树重建后），不以source commit改变代相同身份。

修复仅证书元数据来源，不改语义/增量/运行算法，不新采无关性能全矩阵。原完整组含原条件排除，排除不充PASS。测试与输出只在专属树/统一scope，不在main跑构建。

## 7. 验收标准

| ID | owner_stage | 条件 |
|---|---|---|
| AC-01 | work | 各profile真实cert与manifest及正式域相等，修前红/修后绿，不只手构typed draft |
| AC-02 | work | 原schema/完整文件/工具/reference/oracle/预算不改，C与951控制不退；无新warning/format/debug问题 |
| AC-03 | review | 冻结提交逐AC独立复审、遗漏/延期/workaround scan，SD核准 |
| AC-04 | merge | reviewed身份合入、canonical沉淀、归档、guard清理、材料路径对账完成；返回941 |

## 8. 执行步骤

| ID | owner_stage | deps | 内容 / gate | 状态 |
|---|---|---|---|---|
| T-01 | work | - | 登记scope/树，补真实profile回归并保修前红 | [x] |
| T-02 | work | T-01 | 原producer绑定out.hir_domain、移除domain_of，定向回归/格式/编译 | [x] |
| T-03 | work | T-02 | 提交代码/SD草案，execution_done，完整组门留merge | [x] |
| T-04 | review | T-03 | 冻结逐AC/遗漏/健康/SD独立复审 | [x] |
| T-05 | merge | T-04 | 冻结cargo t ace-a2x完整组、合main及Spec、归档 | [ ] |
| T-06 | merge | T-05 | guard/tree/branch/组/scope清理、941交付回执 | [ ] |

## 9. 复审记录

2026-11-06 work检查点：唯一952树base `1f1e42cb9`、branch `plan-952-dev`，代码/SD提交 **`6308a8ca5`**；auto-down只读`58ea33f`。新6例真实worker逐profile实际编译运行actual/reference=42，并逐值校验cert/manifest/事前域。原假设“多个带-i32 profile可能红”经真实输入核实为仅scalar红，另外5域正控；期望的正式域本身未变，本次仅纠正预测触面，不改变AC或修复范围。

- 修前`cargo t ace-a2x --test-name cert_domain_952 --out .../before` exit1：6执行，5pass/1scalar fail（core-i32-draft != scalar-i32-draft），183不匹配过滤项不是执行/条件排除；整命令41.789s，实际nextest1.710s。results SHA **`50e79035e4e685b1c3b323a9ea7d55c2a4ee51514129b0eb05f3052362f91869`**。
- 修后同一正式入口`--out .../after` exit0：6/6，整命令3.874s，原domain集合/各schema/源和oracle未改；results SHA **`9aa7e91dca9216062c9040069cfe1eb2b0efc4f4f53c8501a415126502669039`**。对应compile日志未出现新warning；format两改动Rust文件与git diff --check均exit0。生产代码仅直接绑定已有实际域，删除22行副本/死代码。
- SD-01已在tree准备，明确同一bundle域权威；不新增schema/预算/语言语义。正常构建target全在952树；新测试runtime与child TEMP/TMP显式指向plan-952 scope，无外置target/源码/安装复制。完整ace-a2x并集待final review后merge一次，不重复无关全仓。

stage: work | plan_id: PLAN-952 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-952/auto-lang | branch: plan-952-dev | code_commit: 6308a8ca5 | task_ids: T-01/T-02/T-03 | next_responsible_role: review



2026-11-06 final review（同会话独立复读冻结产物；无独立agent/context可用或被授权，不冒称第三方审查）：绑定 **r1 / reviewed_commit `6308a8ca550586f5e9d221483611e991a6db57ba`**，base `1f1e42cb9`，952树/branch clean，依赖auto-down仍58ea33f。重新逐行读actual diff、完整6例断言与before/after原日志，不以work打勾替证据。

| AC | verdict | 独立核对 |
|---|---|---|
| AC-01 | pass | 断言分别验证实际两次编译执行=42、manifest事前域、cert事前域与双件profile；scalar修前确红，其余5合法正控；修后6/6。不是只验cert与manifest彼此相等 |
| AC-02 | pass | source/profile dispatcher、三certificate schema及CertDraft其他字段/工具/reference/能力/配置/原deadline全部零改动；C本来直接用out域、生产分支无改；删除fallback后没有二次身份重构。951已落基线。对应完整组待merge一次 |
| AC-03 | pass | 无删子项/延期/workaround/新功能；修复共享Rust cert构造点覆盖全部profile，无新的未解决债务；SD-01与实际单bundle权威一致。定向格式/diff check绿、无新warning/debug输出 |
| AC-04 | pending merge | canonical/完整组/landing/archive/cleanup归merge，未偷算pass |

outcome: **pass**；T-04完成，status reviewed。独立上下文限制明确；结论来自实际冻结diff与counterexample而非角色名。merge仅可用本commit或有range-diff等价证明的重基线；gate=`cargo t ace-a2x`完整职责组（含新6例、C认证、951序列化），不触根语言/ACE之外组，未改compiler/AST/IR语义算法无下游扩展需要。任何gate红回work，不因原始已有而豁免。

stage: review | review_scope: final | plan_id: PLAN-952 | plan_revision: r1 | outcome: pass | reviewed_commit: 6308a8ca550586f5e9d221483611e991a6db57ba | worktree_path: D:/autostack/.wt/lang-952/auto-lang | branch: plan-952-dev | next_responsible_role: merge

2026-11-06 merge 首次完整门：release预构建exit0（冷2m14s）后，`cargo t ace-a2x --out .../full-frozen` **exit1、189执行/188pass/1fail/0skip**，实际nextest192.264s。首红=`record_text_t04::t04_record_move_and_owned_return`，value=2正确，但rt事件实际[]、期望alloc2/alloc2/free2/free2；其入口是rt_common直接assemble/rustc/run，不经过本包改动的certify字段。保首红，不归环境、不充PASS，暂停landing。

定向诊断：同树/同commit/同源与配置、同TEMP/TMP，`cargo t ace-a2x --test-name t04_record_move_and_owned_return --out .../full-red-replay` exit0、1/1，原188过滤项未执行；实际nextest0.235s。检查全仓该输出名REC_MOVE_RET唯一，未发现同名测试产物覆盖；未改helper/断言/样本/预算，未确定首红根因。代码diff仅cert域绑定，不改emission/runtime事件或shared capture设施。判定为**首次非复现红，原因未定**，不能伪称已证明环境。

work诊断/affected re-review：修复正确性与6域证据仍成立，本次健康门需补完整组终态。当前code/review/SD冻结6308a8ca5未变，AC-01/03证据保留，AC-02暂不获得完整gate；无代码修复，不创建workaround或删测试。因新红与原子重放绿，允许一次同冻结完整组验证（`.../full-frozen-replay`），必须取得真实全组终态再landing；若重复失败转具名owner调查，不无界重跑。已重新复读首红/合法重放/改动路径与AC-02，维持代码/SD final pass，完整模块gate仍pending，下一角色merge。此为具名失败诊断，非为提交或角色切换重复全组。

### F-952-01 共享采集器丢最后字节（阻合入，独立修复授权待答）

同冻结完整组第二次`.../full-frozen-replay` **exit1、189执行/188pass/1fail/0skip**、实际185.539s。这次原record/text事件例通过，失败换为`collections_v15::v15_deep_3072_small_stack_witness`，stdout整段为空，期望value:6144及str_bytes:0。只重放该原case，`.../second-red-replay` exit0、1/1，188过滤项未执行。停止完整组重试，不继续刷到绿，不豁免首次或第二次红。首完整结果SHA `138f088454c02035a85a8b09d6b2b9f0bbdde6996c4810955b020c173c0a7095`；第二次SHA `f0cdb358d3630946b40d7d33f29c4730b9d6d02812c863f55eba2b9a1be6d4f6`；两个定向重放分别SHA `e49368c60ee640265ff98a5ced1570d2ac7b52920fdcf8e94b068d71984105f5` / `7a6a72c12eec79ee9fbe87d5a8b4fda5f44b106ffb31e8d67ac875af6502fec4`。

**已确立的独立缺陷**：`crates/auto-tool-execution/src/lib.rs::CaptureWorker::spawn`先`poll_ready`返回false，再load owned_closed；若最后写入与owned结束发生在两者之间，reader以旧“空”观察判EOF，未再排空新到字节。该竞态足以产生空stdout/stderr，不能据此断言两次非确定性首红的具体因果已证明。

确定性反例不改生产代码：复制原std-only crate到本scope `diagnosis/tool-execution-copy/`，lib原始字节前缀SHA与原库完全相同=`c177b278f1d805d21c808054a847adde0bc830e1e24ad0db9b811ce8298e4e09`，仅追加private unit probe。合法PipePoll模型在“空peek已完成”后安排最后字节`last`与owned_closed，再返回原false；原reader稳定给[]。正式诊断命令`cargo test --locked --offline --manifest-path .../diagnosis/tool-execution-copy/Cargo.toml --target-dir .../diagnosis-target --lib plan952_eof_race` **实际exit101，0pass/1fail/12filtered**，期望last，实际空。外部source与target均唯一scope，登记可删缓存，无第二外置目录。首次--locked无独立lock被Cargo前置拒，随后offline生成仅本std crate lock，再locked确认；两条原记录均保留。

证据：`diagnosis/source-copy.json` SHA `5c24e22870e8d66e0b61215d1cc4303becc6e58c9ea4709578e965fbb1af6960`；`eof-race-confirm.json` SHA `ecd63271e8e94ec351b373349c80eab808b0a45de6c067f18d2503b13b9bdf57`；log SHA `7ff3c3a4f657ff882ddd78537803d5bcf22becac0b04a51462ca1c9a32cde737`。

最小拟修复范围（**未实施**）：在原采集器观察owned_closed之后重新检查/排空pipe，再决定EOF；保stop、deadline、真实I/O error/panic、owned subtree与非拥有方持pipe分态，不另造runner。永久确定性竞态反例+最后字节/正常EOF/读错/stop/真实子进程输出回归，并按该共享设施职责及真实消费者核验。这超出952已授权的证书元数据范围，须另立具名修复Plan；已向用户请求授权“独立修复并继续”，当前待答，不预占新编号、不把capture修复塞进952。

本包原6域修复及SD仍成立，但AC-02完整健康门未闭，撤销可合入最终pass，T-04回待复审，status executing 3/6；reviewed_commit6308a8ca5仅为历史代码审查，不授权merge。952树clean/branch/scope保留在途；941亦不因本包局部绿宣布完成。

stage: merge | plan_id: PLAN-952 | plan_revision: r1 | outcome: needs_fix | code_commit: 6308a8ca550586f5e9d221483611e991a6db57ba | blocker: F-952-01独立共享采集器修复授权 | next_responsible_role: user

2026-11-06 F-952-01新增解阻授权：用户答复“授权独立修复并继续（推荐）”；独立PLAN-953承接原采集器竞态、错误分态/永久反例/职责与消费者回归、复审合入后返回952/941。当前仅已授权/立项，不冒修复已交付，不再重复询问该范围。host goal仍blocked，本次按明确直接用户指令执行953全链，不新建goal或伪称自动active。

2026-11-06 **D953已交付**：PLAN-953 r2 source修复65db88be2150991b9c3bbe79d527891fd6a7a27f已线性合main，完整auto-tool-execution43/43、无LEAK/残留，必要ACE两case在新950基面也各1/1，canonical SHA535cdafa3c393a06ceae0635dce506bb04577c304d7fc09565b699dde7b00fb4。953实际归档6/6、两树/branch/组/scope删除完成，无hold。F-952-01独立共享采集器修复授权/交付阻塞解除，不再问授权；952应重基并复审/完整ace-a2x门后合入，941原目标继续保持。周期cargo t all已由950交regress等待936 CT活动门，本会话不重复启动。

### D953后work/affected final review刷新（2026-11-06）

原阻塞F-952-01的生产EOF竞态已由D953交付（独立原始字节/读错反例红绿、完整设施43/43与原ACE消费者绿），不把旧副本替换为伪修复。952重基到含950与953的main6b6b191c3：原6308a8ca5→**`a2866eba2f73a3824a3425dfbc42e9e272f483fe`**。唯一冲突为conformance注册：950将bridge_flags移到按名字排序位置；保既有所有模块且bridge_flags恰一次，将cert_domain_952注册到排序位置恰一次。range-diff标`!`仅因该注册上下文位置；生产域绑定、6个fixture断言、SD字节均与原patch等价，**不谎称整patch `=`**。

work复验：`cargo t ace-a2x --test-name cert_domain_952 --out .../after-d953` exit0，6/6、12.381s，结果SHA **`a9fdf92e8f2c970c7777b740098239a756372dfbd634c7ab800d72495e4500ec`**；新950编译基面实际通过，核心/标量/record/text/text-record/collections全保。tree clean，原预算/原文件集/schema/oracle未改；本包没有新增生产代码修复或scope变更，仍r1。

同会话独立复读上下文限制同原review。重新读取rebased diff与测试注册（无重复或删项）、六断言与真实stdout/JSON、SD。AC-01 pass；AC-02 source/perf/完整生成/工具字段零变且D953已闭原原始采集不变量，所属完整ace-a2x门仍pending merge；AC-03 pass；AC-04 pending。**final pass bound r1/a2866eba2f73a3824a3425dfbc42e9e272f483fe**，SD源码SHA **`a354c7807e538d9f34482f2710083752d7837a4ffb55c6bd459c6c1e0342192b`**。只允许该身份或重新核验的基面合入；不把旧6308历史pass覆盖新基面。

周期主检出队列已由已有owner实际执行37/37（PID1740现已终态/缺失，队列wall702.068s），不是本会话重启；std设施在更广队列出现新增fixture首heartbeat未就绪红，原新EOF单测均绿，生产scope未失效。该新red由周期owner按fix-forward归因/后继计划处理，不把它改成known green、不重新打开已归档953；不据其root队列总FAIL跳过本包所属完整组。当前实际无该批量重门在运行，952可恢复专属模块门。其他936专属语义门与此不同树、无共享输出；不启动第二主检出周期队列。

stage: review | review_scope: final | plan_id: PLAN-952 | plan_revision: r1 | outcome: pass | reviewed_commit: a2866eba2f73a3824a3425dfbc42e9e272f483fe | worktree_path: D:/autostack/.wt/lang-952/auto-lang | branch: plan-952-dev | next: merge（完整ace-a2x pending）

2026-11-06 D953后完整门终态：release显式构建exit0；`cargo t ace-a2x --out D:/autostack/plan-artifacts/plan-952/full-after-d953` exit0，**189执行/189通过**，gate_passed=true，errors=[]，整命令164.0424465s；results SHA256 **25c198b56e3fc3187b507c0add67e573518b037e9f341f1a902f82947b4336c6**。冻结代码仍a2866eba2f73a3824a3425dfbc42e9e272f483fe；不以该绿追认原两次红。当前reviewed 4/6，完整职责门已闭，尚未landing/archive/cleanup。周期新fixture红已有独立PLAN-954由原队列owner立项，不重复建号或打开953。host goal经实际get_goal仍blocked；按auto-plan-goal的inactive恢复规则保存检查点，待用户`/goal resume`恢复自动链后执行merge。

## 10. 待澄清事项

1. F-952-01独立修复已由D953交付；952原授权保持，待在D953基面核完整门再合入。
