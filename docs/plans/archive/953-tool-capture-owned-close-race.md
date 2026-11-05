---
plan_id: PLAN-953
status: archived
feature_name: owned结束捕获最后字节竞态修复
plan_revision: 2
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
supersedes_spec_components: [docs/specs/tool-execution/project.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [tool-execution]
depends_on: [PLAN-816, PLAN-896]
current_step: 6
total_steps: 6
---

# [PLAN-953] owned结束捕获最后字节竞态修复

## 0. 变更摘要

独立关闭F-952-01/D-952-1：原CaptureWorker把早于owned_closed观察的空peek用来判EOF，最后字节可能在两观察之间到达并被丢。修复原设施收尾采集，不另造runner，不靠减并发/删断言/刷完整组补救。

## 1. 目标

正常退出/owned结束必须完整保留已写入stdout/stderr的原始字节；空peek必须在观察owned结束后重新确认。保持原错误/stop/timeout/cancel/PipesHeld/panic/owned树与线程收尸分态，零新依赖、零新公开API或预算。

## 2. 架构方案

在原reader读就绪分支，closed为真后进行一次新的即时poll；有数据则继续正常读取，只有新poll也空才EOF。复用原循环/错误/停止控制，不新用500ms drain作为成功截止（避免把未排空当EOF）。生产变更局限`crates/auto-tool-execution/src/lib.rs::CaptureWorker::spawn`；测试夹具修复仅`src/bin/te-child.rs`与`tests/exec_lifecycle.rs`（无产品API/预算变更）；同模块永久竞态/读错/合法空EOF回归，原exec_lifecycle真实子进程通道回归保留；r2补测试夹具就绪/自身寿命等待，使原义务可被严格外层Job门真实验证。

## 3. 技术栈

Rust std-only，原PipePoll/Read注入面，原固定组`cargo t auto-tool-execution`。必要真实消费者沿原`cargo t ace-a2x --test-name t04_record_move_and_owned_return`与`--test-name v15_deep_3072_small_stack_witness`，完整ace-a2x归952在D953后冻结合入门，不在953再重复完整ACE。

## 4. 需求分析与背景调查

用户2026-11-06已答复“授权独立修复并继续（推荐）”，覆盖结束判定前重新排空/错误分态/永久竞态回归/验证/复审合入/返回952与941，全链继承无需再问。主检出基线93f044978，main只有其他会话文档WIP，保留不纳提交。原源码SHA c177b278f1d805d21c808054a847adde0bc830e1e24ad0db9b811ce8298e4e09。952两次完整ACE门各188/189、两个失败case单例绿；**不认定这两次具体失败因果已证明**。独立确定性副本probe在合法poll交错下actual空/expected last、exit101，原始证据在952 §9；新包不移动/删952材料。

正式Spec `docs/specs/tool-execution/project.md`要求整次执行/原字节/自然EOF/所有错误与回收具名，现设施共享stdin/null/capture output包装与ACE真实编译消费者。唯一tree拟`D:/autostack/.wt/lang-953/auto-lang`、branch `plan-953-dev`，只读组auto-down。材料唯一`D:/autostack/plan-artifacts/plan-953/`，含所有runtime/TEMP/外置target；正常构建target只在专属树，merge全删。

当前host goal仍blocked，工具不提供resume；本包按本次明确直接授权执行，不新建/伪称恢复goal。原941未完成，返回链保持。无agent/chat授权，同会话实现与冻结后独立复读审查，说明上下文限制。

## 5. 详细设计

生产闭包：PipePoll早空→owned结束→新poll→原Read字节/EOF/IO/panic→原finish_capture分类。stop检查仍在原循环顶部，超时/取消仍原树终止/reader停止；不加默认deadline，不把部分采集变Success。新poll仅在closed且旧poll空时发生，不影响正在执行的普通读分支。

### 规范增量

| delta_id | action | target | before/after | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/tool-execution/project.md | 明确owned结束后必须新确认管道是否空，不能用此前空peek推EOF；保持字节与错误/停止分态 | AC-01/02 |

## 6. 测试设计

| V | AC/entry | 输入/不变式/观察oracle | 边界 |
|---|---|---|---|
| V-01 | AC-01 原CaptureWorker永久单测 | 空peek后最后字节与closed到达；stdout/stderr各完整原字节（含NUL/无效UTF8），修前红修后绿 | 可控合法交错，不是靠sleep碰运气 |
| V-02 | AC-02 同新poll分支 | 最后read真实IO错→ReaderIo；合法空EOF→空成功；已到字节不丢；原stop/panic/PipesHeld现case保持 | 不把新poll的错误抹成EOF |
| V-03 | AC-02 原exec_lifecycle | 自带截止te-child、真实两流/大输出/invalid UTF8/超时部分输出/树与holdout等原用例 | 原样本/ignore/平台条件不减 |
| V-04 | AC-02 原ACE两命名消费者 | record/text move+return值及alloc/free事件完整；3072深链stdout含6144及live0 | 只归采集消费者验证，不冒整个后端能力 |
| V-05 | AC-03/04 | 冻结逐AC/SD/遗漏复审后，完整auto-tool-execution组+必要消费者终态 | 完整ACE只由952后续跑 |

新测试驱动原私有生产闭包，不复制算法作oracle。fixture与target显式本scope/树，零junction/symlink。适配共享多消费者规则但不机械展开全仓AST/实验图；原reader/平台/错误矩阵由完整职责组闭合。Windows实测，其他平台只声明同可移植分支/原条件，不冒实机通过。

## 7. 验收标准

| ID | owner_stage | 条件 |
|---|---|---|
| AC-01 | work | 原生产reader可控竞态修前红/修后完整字节；stdout/stderr分别覆盖，无新500ms截断成功 |
| AC-02 | work | 原IO/stop/panic/timeout/cancel/持pipe/树与线程收尸/合法空EOF保持，真实消费者原观察正确，无新warning/依赖/协议变动 |
| AC-03 | review | 冻结逐AC/遗漏/workaround/SD复审pass；scope与输入身份明确 |
| AC-04 | merge | 完整组与必要消费者门绿、线性落主线、canonical核验、归档/guard/全scope清理、交付952/941 |

## 8. 执行步骤

| ID | owner_stage | deps | 内容 | 状态 |
|---|---|---|---|---|
| T-01 | work | - | 创建登记树/scope，原reader正式永久竞态红与正控 | [x] |
| T-02 | work | T-01 | 结束后新poll，定向竞态/IO/EOF与格式/编译绿 | [x] |
| T-03 | work | T-02 | SD与代码提交，execution_done，完整职责门留merge | [x] |
| T-04 | review | T-03 | 同树冻结独立复读AC/负控/遗漏/SD | [x] |
| T-05 | merge | T-04 | 完整auto-tool-execution与ACE两命名消费者、线性合入/spec核验、归档 | [x] |
| T-06 | merge | T-05 | guard/tree/branch/组/scope及实际输出目录对账清理、返回952/941 | [x] |

## 9. 复审记录

2026-11-06 work完成：base26d42272c，唯一tree/branch953，auto-down detached58ea33f未改；代码/SD提交 **`78b07fb9d`**，tree clean。原生产闭包中旧空peek后观察closed，还必须新poll为空才EOF；新poll有数据走原Read，后续IO错误不会被EOF吞掉。没有引入500ms成功排空帽/新依赖/新runner或API变更。

| gate | actual / cases / wall | results SHA256 |
|---|---|---|
| cargo t auto-tool-execution --test-name owned_close_953 --out .../before | exit1，3执行/1合法空EOF pass/2fail（丢最后字节、读错抹空），原40不匹配未执行；2.892s | eccdb4314f0b35a4cc9da593f28ec8f04b6782ad4cd83bca55a2cc9c77931d0e |
| 同入口 --out .../after | exit0，3/3，含stdout/stderr循环的NUL/0xff与新读错具名；2.486s | a95e5cfde6224d1b12c8cd695af3eb1364e85b1eeedf6b43589ed72b836b4f5c |
| cargo t auto-tool-execution --test-name tests:: --out .../original-reader-controls | exit0，原12/12，含stop排空/读错/panic/PipesHeld/真停/正常EOF等，原条件未改；4.740s | 8343f6914cec4637aa827be022ffed2f4ce63000f964971f419ca67aed476c4e |
| cargo t ace-a2x --test-name t04_record_move_and_owned_return --out .../record-text-consumer | exit0，实际1/1，原183不匹配未执行；冷编译主要在发现阶段，整命令39.831s；值2+原alloc/free完整 | 0eb580854b94ac761d91ccd68498354337a9fc6c277b71e69a4fa62d7f6eac72 |
| cargo t ace-a2x --test-name v15_deep_3072_small_stack_witness --out .../collections-consumer | exit0，实际1/1，原183不匹配未执行；1.804s；6144/live0原断言保持 | 468cb4caa12746b16dab1e052f80b1b60cce8b3d62d9bdca8adcd68817871dc8 |

所有命令在953树；TEMP/TMP显式本scope/runtime-temp，正常target仅本树，未在main构建。格式rustfmt --check与git diff --check exit0；相关新编译无warning。三新测试直接驱动原CaptureWorker/finish_capture，不复制算法；测试自有flag交付2s截止/线程join。两实际消费者属于发生首红的原入口，不等同证明两次历史首红具体因果，未重复完整ACE。原固定组auto-tool-execution完整43面仍归final review后merge一次；consumer证据身份不变可复用。

AC-01/02 work均已具名覆盖，无未解决同范围finding/延期/workaround；SD-01只澄清原EOF完整字节不变量。后续角色按冻结commit审查后跑完整职责门，不为提交重复全部组。

stage: work | plan_id: PLAN-953 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-953/auto-lang | branch: plan-953-dev | base_commit: 26d42272c | code_commit: 78b07fb9d | dependency_revisions: auto-down58ea33f只读 | task_ids: T-01/T-02/T-03 | next_responsible_role: review


2026-11-06 final review：同会话冻结后重新读取完整diff/新三例/原错误与stop收尾边、真实消费者结果；无独立agent/context授权或可用，本限制明确，不以角色名冒第三方独立性。基线r1 / reviewed_commit **`78b07fb9dc3aa7a01731b84c37e4c17dafd79c14`** / base26d42272c，953树clean，auto-down只读58ea33f。Spec SD-01冻结hash **`15435f82a41abf5710f8e5e3d42c7b3902ecf89e98b1f8fec7e2ff8ea820a1a7`**。

| AC | verdict | 实际核对 |
|---|---|---|
| AC-01 | pass | 修前2条反例真实失败，修后stdout/stderr各保原NUL/0xff字节；原CaptureWorker→finish_capture路径，无并行复制判定器。fresh poll位于closed观察之后，且只有两次空（新空才有效）才EOF，无500ms新增成功截断 |
| AC-02 | pass | false旧peek/closed假继续原循环；closed真/新poll真继续原Read，I/O/panic原传播；stop仍先于轮询检查，原deadline/owned树路径零改。12原单测与2真实消费者绿，API/依赖/预算/协议零变，格式/diff绿，无新编译warning。完整职责组留merge一次 |
| AC-03 | pass | 未删case/样本/条件、无延期或workaround；新测试自有线程join且等待有界。未发现本修复范围未闭债务；D-952-1需合入后才销号。SD-01描述原EOF不变量与当前代码一致 |
| AC-04 | pending merge | 完整组与landing/canonical/archive/cleanup未偷算 |

共享读端适用于原执行/Command capture/output包装；修复不改consumer类型或ABI，必要消费者按真实调用选原ACE两例，不机械展开全仓/全experimental。Windows实际验证，Unix平台未跑不冒PASS。final pass绑定本commit与SD；完整`cargo t auto-tool-execution`作为merge-owned门，两个真实消费者同源码/依赖/配置/输入可复用现证据。若完整门红按原分态返回，不豁免。

stage: review | review_scope: final | plan_id: PLAN-953 | plan_revision: r1 | outcome: pass | reviewer_identity: same-session artifact reconstruction, independent-context unavailable | reviewed_commit: 78b07fb9dc3aa7a01731b84c37e4c17dafd79c14 | worktree_path: D:/autostack/.wt/lang-953/auto-lang | branch: plan-953-dev | pending_closeout_gates: auto-tool-execution完整组+landing/cleanup | next: merge

### r2 有界夹具收尾修复与重新复审（2026-11-06）

r1冻结完整`cargo t auto-tool-execution --out .../full-frozen` exit1，**43执行/40pass（1leaky）/3fail/0skip**，14.655s，results SHA `b720a69220fee4ed1b3a97b9f286f690f6327c94c425ffb7ec197d7102b02a92`。首红保留：root_exit...后代首heartbeat未创建，blocked stdin两case elapsed5.009s/2.880s超过原1.2/1.5s。定向`--test-name deadline_covers_blocked_stdin`两case均原时限绿（0.284/0.539s），**整体仍exit非0**，因原unowned holder 5s后代存活、nextest LEAK及外层native_meter实报活te-child+conhost并完成Job回收；没有把子项2pass当整体PASS。该收据SHA `7dcc3191ba29c668e622b2f48a55bb8d569f77d93f5112368ef02fa4e9eaf0f1`。串联的原orphan重放因前命令非零未执行，不冒执行证据。

诊断确认：原stdin-holder与detached测试在验证其“非拥有/分离存活”后直接结束，合法反例子进程仍按5s/2s自身寿命存活；测试层未收尾，违反严格外层Job工作负载门。orphan root派生后立即退出，与生产终止持pipe后代竞争，可能在后代首tick前被正确终止，故原“必有首tick”断言缺就绪前提。原完整门两个elapsed红定向重放绿，**不声称其具体环境因果已确立**，后续完整门仍保原时限验证。

有界r2调整（同目标/AC/预算/仓/动作，原用户修复及验证授权覆盖，不新增能力或放宽阈值）：测试夹具heartbeat发布PID，orphan在首tick就绪后根立即退出；测试层在原API测量/存活观察之后等具名fixture实际按自身deadline退出。Windows以SYNCHRONIZE handle等待，Unix waitpid/存在性有界等待；不提前杀非拥有后代，不放宽1.2s/1.5s/250ms或原并发设置。等待与原受测elapsed分列，等待失败仍红；原错误/预算/公开ABI不改。fixture运行及PID文件全部scope内，未迁移生产runner。修复提交 **`0e68084a4`**；78b07fb9d中的生产reader与三新竞态case字节未变，复用其红/绿与原12case、两ACE消费者证据；只补受影响fixture反例后再完整组，不为角色切换重跑。

| 新fixture定向门 | actual | results SHA256 |
|---|---|---|
| --test-name deadline_covers_blocked_stdin --out .../fixture-deadline-controls | exit0，2/2，7.409s（含原5s fixture self-exit；原API elapsed预算照旧；无LEAK/工作负载残留） | c6ada08318a217eda837926c5115769bd4413ddfdc7add730ed75d5e0ab7b62f |
| --test-name root_exit_terminates_pipe_holding_descendants --out .../fixture-orphan-controls | exit0，1/1，2.414s，保持原1.5s API断言与heartbeat停止断言 | d3169191e0e3095c95e88b2001141616728414b33dae7a139a15291d8ac34594 |
| --test-name detached_child_outlives_spawn_caller_handle --out .../fixture-detach-controls | exit0，1/1，3.913s，保持独立存活3tick断言后实际自退，无LEAK | f40dc7f41e4f25e864bb4c578be21eef1a996a627ce4ab08653838a4a441e243 |

work r2 readiness：无代码/合同pending，三个受影响fixture门绿，main/其他会话WIP未动；格式/diff检查绿，变更无新warning。r1首红/后续定向/新绿均保留，不将条件排除充PASS。T-01..03保持完成，待r2重新冻结复审。

final affected re-review（同会话独立复读限制同r1）：绑定 **r2 / reviewed_commit `0e68084a4`**（含父78b07fb9d），953树clean，原依赖/配置未变。重新读取所有新增fixture行、时间测量位置、Windows handle关闭/错误传播与Unix有界循环。AC-01保原不变证据pass；AC-02新就绪与收尾均真实完成、原预算不放宽，新API无正向环境豁免；AC-03 SD新增测试收尾规则与原严格工作负载门一致、无未批准延期/workaround。完整组43原条件待merge；AC-04未完成。**final pass**，T-04在r2完成，允许本commit加有等价证明重基后的合入，修复循环1次未越界。非仅把新边界写入债务即完成。

stage: review | review_scope: final | plan_id: PLAN-953 | plan_revision: r2 | outcome: pass | reviewed_commit: 0e68084a4 | worktree_path: D:/autostack/.wt/lang-953/auto-lang | branch: plan-953-dev | next: merge（完整组待执行）

### merge prepared / landed / canonical_verified / archive（2026-11-06）

r2冻结`cargo t auto-tool-execution --out .../full-frozen-r2` **exit0、43/43、0skip、无LEAK/残留**，整命令6.960s，source_state `b4701b3e97a8b7bcba3d6479a638d2016eab7efa1a46b8ce815312ebb1581ebd`，results SHA **`d8e072c8a36f3bb79d9c4fd0d825d72f90ffe6e94164f1f3b79fed67948c8e1f`**。该真实完整终态闭合原r1失败门，首红/单例子项绿但整体残留红仍保原记录，不追溯改绿。

落地主线期间950并发已真实合入；953的std设施/Spec及根Cargo/测试配置输入未变，ACE解析/模块库与锁文件有变化。953重基到main `7a6183c16`，range-diff两patch均`=`：78b07fb9d→ea8fdc974；0e68084a4→**`65db88be2150991b9c3bbe79d527891fd6a7a27f`**。完整std职责组证据输入不变可复用；为新的ACE依赖基面重放**两个必要真实消费者**，均1/1：record结果SHA `c8230ce1cdc2ede3b772cbefa15e1245a9463a8e0d7137d36e05f20c05d1b836`、11.767s；collections SHA `b7e595325636fd435ab22d3574d38ef3520032d7657bd7ad14717932896ef7e5`、1.920s。未因新HEAD重复std或完整ACE。

- prepared：r2 final pass与2个patch等价已核，953树clean，auto-down58ea33f只读。
- landed：953 guard clean无reparse后，main **`git merge --ff-only plan-953-dev`** exit0，tip实际等于65db88be2150991b9c3bbe79d527891fd6a7a27f，无merge commit。保留main其他会话文档WIP，无代码WIP混入。
- canonical_verified：`docs/specs/tool-execution/project.md` main实际SHA **`535cdafa3c393a06ceae0635dce506bb04577c304d7fc09565b699dde7b00fb4`**，与落地commit/953树相同，SD两部分（EOF新确认、测试夹具收尾）俱在；未写legacy specs/index/cache。
- completion_kind: delivered；T-05完成。archive路径=`docs/plans/archive/953-tool-capture-owned-close-race.md`，正在commit实际move+状态；T-06仍pending，不能提前报cleaned。

输出目录对账：原正式命令只有本scope（含TEMP/TMP runtime-temp）、953树根target与experimental/ace/target；无外置消费者源码/安装/发行副本或其他盘符输出。scope预折叠131文件/625,534 bytes（不含后写closeout-scope摘要），未发现链接或未知材料混入；所有仍需证据已在本Plan折叠，merge清理全删，无hold。正常编译缓存随tree删除，不回收存盘。952的诊断副本/target与941材料仍是各自in-flight scope，不在本包删除范围。

生产物观察（北京时间，未冒部署）：PATH `auto`实际是`D:/autostack/auto-man/target/debug/auto.exe`，mtime 2026-11-05 22:35:19，未在本包重建/重启；main `target/debug/auto-case.exe` 2026-11-06 00:55:19、`experimental/ace/target/release/a2x-worker.exe` 2026-11-05 13:29:34，均早于本修复、显式stale。main `target/release/auto.exe`不存在。后续952/941在其树显式重建使用本源，不以旧exe作已修证明；未触依赖仓binary/UI前端，所以无对应部署声明。

### cleaned / delivered receipt（2026-11-06）

953/auto-lang、只读953/auto-down移除前分别再跑wt-guard，两者clean无reparse；git worktree remove两树exit0，plan-953-dev已-d删除，空组lang-953移除，实际不存在。所有代码提交均在main65db88be2150991b9c3bbe79d527891fd6a7a27f ancestry内，archive move与r2状态已commit `4e976fb19`。构建target直接随tree删除，无exe/PDB回收副本。

scope清理：首次批量PowerShell删除命令被自动安全审查以“blocked by policy”拒绝，**未执行删除**；改用既有auto-plan-merge保守`artifact_retention.remove_scope`，校验953 owner、实际archived源/交付commit、已无tree、无链接后，对每个打开句柄复核最终真实路径并删除该目录项（不运行已退役latest-five全局选择器）。实际删除**132 files / 41 dirs / 625,855 bytes / 0 reparse**，scope `D:/autostack/plan-artifacts/plan-953`不存在，无hold。删除前逐文件清单聚合SHA **`bfdc4a73cab99c40b483db7e070363fa64e78b4b2f34678390f28f0c16065ecb`**；清理索引刷新为deleted。952/941在途scope及未知其他材料均保留。

周期门due-check：最新共享receipt仍covered940；950已真实交付且%5=0，**到期**。950归档记录§9已把周期门交regress并等936 CT gate，当前确有其他会话CT nextest在运行；按regress“single instance / no other heavy gate”不再启动第二full tier。此处为具名交接既有950周期门owner，不写新的已覆盖收据、不报cargo t all绿；本包43职责门与两个ACE消费者均已验证。旧receipt时限未超48h不改变950触发事实。

D-952-1/F-952-01的原EOF竞态已由本包生产修复与永久反例/完整设施组/新基面两consumer闭合。历史952两次空输出具体因果未冒确定；952须在本交付基面重做其完整ACE门，941也仍需其余完整验收。T-06完成，**6/6**；scope/worktree/branch清理完成不等同941完成。

stage: merge | plan_id: PLAN-953 | plan_revision: r2 | outcome: pass | completion_kind: delivered | delivery_commit: 65db88be2150991b9c3bbe79d527891fd6a7a27f | canonical_spec: docs/specs/tool-execution/project.md@535cdafa3c393a06ceae0635dce506bb04577c304d7fc09565b699dde7b00fb4 | archive: docs/plans/archive/953-tool-capture-owned-close-race.md | cleanup: complete, no holds | periodic_batch: due, existing PLAN-950 regress owner waiting active PLAN-936 CT gate | next: PLAN-952

## 10. 待澄清事项

无新增需求问题。仅host goal状态未自动resume；不阻本次用户明确授权的独立修复执行，不擅自伪造active状态。
