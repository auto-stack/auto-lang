---
plan_id: PLAN-944
status: executing
feature_name: ACE 标准库绑定、装配与缓存接入
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-06
plan_revision: 2
supersedes_spec_components: [docs/specs/stdlib/design/assembly-manifest.md, docs/specs/auto-a2x/incremental-and-cache.md]
new_spec_components: [docs/specs/stdlib/design/ace-binding-and-assembly.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
depends_on: [PLAN-943, PLAN-950]
current_step: 1
total_steps: 7
---

# [PLAN-944] ACE 标准库绑定、装配与缓存接入

## 0. 变更摘要

将公共有限模块和语义合同接入统一 source 前门及 AV、HIR→Rust 两条真实消费者；扩展 manifest/缓存身份并证明恢复首编辑正确。

设计依据：[Design54](../design/54-stdlib-semantics-and-ace-integration.md)，衔接 Design33/46/51。**本轮仅编写设计和计划，先不施工**。七个任务全部未执行；未建实施树/材料scope、未编译/跑产品测试、未切默认。`cargo t --list` 仅核入口，非测试证据。

## 1. 目标

本包目标：将公共有限模块和语义合同接入统一 source 前门及 AV、HIR→Rust 两条真实消费者；扩展 manifest/缓存身份并证明恢复首编辑正确。

非目标：不实现完整 CAP、全语言模块/包系统或所有标准库，不复制 legacy Parser/AST emitter，不切默认，不复制 store 或资源分析。

不改变原M1/M2 required/accepted-diff/NA、语料/oracle、既有资源预算及CAP进入门。缺口保留，不为通过删除子项。

## 2. 架构方案

复用当前统一source/Checked HIR、共享MIR/资源事实及query/store。A2X消费HIR，AV/native消费共享MIR；公共语义与物理实现分开，旧runtime只按实际合同复用。标准操作身份、兼容策略和manifest双指纹遵守Design54 §4–7。

依赖：PLAN-943 正式交付（D943）及 [PLAN-950 有限公共函数模块前置](archive/950-ace-finite-source-modules.md) 正式交付（D950）

## 3. 技术栈

Auto公共源码、现有Rust装配/ACE workspace、既有TestCase与固定Cargo组。不得添加平行编译器、资源分析、store或自报PASS系统；新文件/接口为拟议，T-01先核真实consumer再定位置。

## 4. 需求分析与背景调查

从 `docs/specs/overview.md` 的stdlib/语言/ACE边界取材，读取current stdlib项目卡、assembly-manifest/backend-assembly、相关模块源码及Design33/46/51。读取基线HEAD `a0750086c`；部分设计在磁盘有其他会话WIP，确切输入hash见Design54 §12，不把WIP当正式D。

用户授权原文：“按这个方向编写……设计文档……拆分后续实施计划，先不施工。”本轮仅AutoLang设计/计划/索引簿记，未授权任何实现、外仓写入或自动续跑；没有预算/模型/agent指派。未来执行需要后续授权，不能以本draft推断批准施工。

已核路径（后继开工重核实际符号与版本）：

- `crates/auto-lang-core/src/stdlib_assembly/model.rs`（现存；文件/目录）
- `crates/auto-lang-core/src/stdlib_assembly/manifest.rs`（现存；文件/目录）
- `crates/auto-lang-core/src/stdlib_assembly/reference.rs`（现存；文件/目录）
- `experimental/ace/crates/core/src/source`（现存；文件/目录）
- `experimental/ace/crates/semantics-contract`（现存；文件/目录）
- `experimental/ace/crates/source-query`（现存；文件/目录）
- `experimental/ace/crates/av`（现存；文件/目录）
- `experimental/ace/crates/a2x-core`（现存；文件/目录）
- `experimental/ace/crates/a2x-rust`（现存；文件/目录）
- `experimental/ace/crates/artifacts`（现存；文件/目录）

边界：现有文档的历史数量/状态不自动当当前运行事实；`primitive-str-byte-len`与872绑定矩阵差异先由943按具体入口处理。旧math公开i64与ACE i32域分开；普通Rust/C能编译不证明Auto语义。

## 5. 详细设计

抽取最小 backend-free 操作/装配事实，具体文件由 T-01 定位（拟新增 semantics-contract 内 stdlib 子模块，非额外全局 registry）。既有 common/consumer 指纹用途保留，新增 producer/profile/host/platform/ABI/runtime 身份经版本化 codec 消费。旧 v4 的读取按其旧语义，不擅加字段改 hash。

有限公共模块须真实 namespace/可见性/依赖绑定，经原统一 source 与 Checked HIR；首例可用测试模块返回 42，AV 与新 HIR→Rust 实际执行对独立 42 oracle。测试模块不是公共标准库支持声明。至少一个真实 943 标准操作的引用证明接入 manifest；行为交付由945承担。若 use/模块能力不足，T-01 形成最小接线决策，不能用字符串替换或 legacy 前端兜底。

### 执行合同与下一入口

输入→实际producer→存储/中间表示→consumer见上列路径及§6，未新增接线由T-01/T-02创建。开工先核主检出计划revision、正式依赖SHA与当前树HEAD，记录实际已闭/未闭项；T-01只允许有界前探与合同冻结，不能用整包重构解决未知。需改变公开语义/支持阈值/跨仓权限则修订Plan并向用户说明具体决定。

所有临时probe、日志、receipts、外部验证副本统一 `D:/autostack/plan-artifacts/plan-944/`，开工登记owner/用途/在途状态；本轮未创建。常规构建在唯一实施树target，外置target也只在此scope。留存结论最终折入本Plan；测试fixtures若为产品长期用例留在真实owner测试目录并明确T-02..05登记，不把临时探针散落仓根。

### 规范增量

以下是**拟议**delta，review按实际范围定稿，merge才写canonical Specs；本轮不写current Spec。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/stdlib/design/assembly-manifest.md` | 记录实际新增 producer、有限模块绑定、装配身份和恢复合同，旧 producer 不重解释。 | Design54/本包实际边界 | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-a2x/incremental-and-cache.md` | 记录实际新增 producer、有限模块绑定、装配身份和恢复合同，旧 producer 不重解释。 | Design54/本包实际边界 | AC-01..04 |
| SD-03 | add | `docs/specs/stdlib/design/ace-binding-and-assembly.md` | 记录实际新增 producer、有限模块绑定、装配身份和恢复合同，旧 producer 不重解释。 | Design54/本包实际边界 | AC-01..04 |

## 6. 测试设计

全部是待执行设计，非当前证据。首批实际运行限既有已认证Windows x64；有界T-01只做§6列出的最小输入与对应合法/非法近邻，预算沿各原recipe，不临时添加新性能阈值或做全库压力。新增fixture由T-02..05接到真实entrypoint，T-01把下表的概念输入落实成当前合法语法/入口，原不支持域不改成伪正例。

| Case ID | AC/invariant | 输入/设置 | 真实入口或拟议接线 | 独立可观察结果 | owner/coverage |
|---|---|---|---|---|---|
| V-01 | AC-01 | 有限模块 answer 返回42、用户同名函数返回7 | 统一 source→AV / a2x-rust 真编译运行（T-01/02） | 标准绑定42，用户绑定7；非两端相等自证 | work/T-05；仅本包冻结域 |
| V-02 | AC-02 | 实际 manifest 改 producer、profile、签名或 runtime 身份 | 既有 codec/装配 admission（T-03） | 使用前具名拒；合法异绝对根共同指纹稳定 | work/T-05；仅本包冻结域 |
| V-03 | AC-03 | 改单函数 body、被读effect、provider 内容后重启首编辑 | 真实 source-query/store恢复与两个consumer | 各变更沿依赖失效；无关函数生产0；结果来自新实现 | work/T-05；仅本包冻结域 |
| V-04 | AC-04 | 有效旧产物后注入绑定/发布失败 | 现有安装入口和下一次实际恢复 | 旧字节/结果不变；恢复读回证实持久状态，无冷重编冒恢复 | work/T-05；仅本包冻结域 |

验证入口：`cargo t stdlib ace-source ace-semantics ace-artifacts ace-av ace-a2x`。组名已从本轮 `cargo t --list` 核验；work按实际改动确认完整职责组与必要下游并集，最终review后冻结提交沿同并集核门，身份/输入未变证据可复用。若修改共享 MIR/lowering，补 ace-lowering ace-native；实际修改编译期依赖再补 ace-comptime，不因设计提及CTE就跑。

新用例必须进入所选固定完整组；若现有组无法覆盖新合同包，T-05明确登记新固定职责组并在review前冻结，不靠过滤的零选择假绿。原ignore/平台/性能/认证条件不减；不按计划机械运行全ACE或全仓。周期`cargo t all`仅由主检出due门负责。列发现/执行/条件排除数，准备/运行/清理/总时间与子终态；编译错误、超时、未执行不算语义反例PASS。

实现者自查不能替代final independent review。本轮未指派独立agent；未来review以独立上下文在同一唯一worktree核每条AC、遗漏/延后/workaround、warning/fmt/debug及Spec delta，绑定revision/SHA。复审期间停止实施写入，发现问题交回同树修复。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 有限公共模块经统一 source 前门进入 AV 与新 HIR→Rust，实际返回独立 expected；无 legacy fallback。 | T-01及T-05；§6对应输入与独立观察 |
| AC-02 | work | manifest 区分 producer/profile/host/platform，双指纹及旧读取合同正确；真实标准操作引用有 proof。 | T-02及T-05；§6对应输入与独立观察 |
| AC-03 | work | 合同/依赖变化使真实受影响者失效、无关函数不重产；恢复首次编辑正确，坏缓存使用前拒绝。 | T-03及T-05；§6对应输入与独立观察 |
| AC-04 | work | 失败安装保旧产物可执行；当前运行授权重新准入；两条消费者和 legacy 装配守卫均通过。 | T-04及T-05；§6对应输入与独立观察 |
| AC-05 | review | 独立核§6反例/合法对照、所有AC、遗漏/延后、warning/fmt/debug及Spec delta；结论绑定冻结revision/SHA | T-06独立复审记录；不信勾选/父exit0 |
| AC-06 | merge | 实际Spec落地、代码来源一致、归档与scope/树清理对账完成；不得把正确性留到merge | T-07实际收据/guard及路径核验 |

## 8. 执行步骤

唯一实施树 `D:/autostack/.wt/lang-944/auto-lang`、分支 `plan-944-dev` 已在r1前探创建并保持clean。用户已授权全链；先交付950再回本树同步正式master，重核唯一树/仓/分支/base/HEAD，不另建944树。无junction、无跨树可写target。

- [x] **T-01｜owner_stage: work｜AC-01**：重核 D943、929 统一前门和 source-query 实际 API；以有限公共模块完成 source→Checked→AV/Rust 可行性前探，冻结中立模型位置、读写版本和支持范围；缺语言前置则记录并修订依赖。
- [ ] **T-02｜owner_stage: work｜AC-02**：实现操作/模块绑定与真实引用证明；校验 namespace、可见性、用户遮蔽、签名/mode/版本；接通 AV 和 HIR→Rust 两个 consumer，未知符号调用前拒绝。
- [ ] **T-03｜owner_stage: work｜AC-03**：扩展既有 manifest/codec 和 provider 输入闭包；保持双指纹，分离新旧 producer；结构坏 schema/版本/重复声明在消费前拒绝。
- [ ] **T-04｜owner_stage: work｜AC-04**：接既有 query/store 的依赖、恢复与首次编辑：body、合同、provider、授权分别处理；持久化无宿主地址/授权，失败候选保旧可用。
- [ ] **T-05｜owner_stage: work｜AC-04**：补两个真 consumer 的值腿、真实引用证明、坏身份/用户遮蔽、provider/effect变更与恢复首编辑反例，跑职责组及必要下游，核实际生产而非 wrapper 计数。
- [ ] **T-06｜owner_stage: review｜AC-05**：在同一实施树以独立上下文执行 `/auto-plan:review`，逐AC/反例与真实diff核验，具名记录债务候选；实际通过才进入reviewed。
- [ ] **T-07｜owner_stage: merge｜AC-06**：按 `/auto-plan:merge` 核来源/落Spec/归档；证据摘要折入Plan，逐命令实际输出路径对账，删统一scope（具名持有除外），刷新retention；`wt-guard` clean后移除树/分支，实际完成后记账。

T-01继续执行前置：D943及D950均已正式交付；本目标用户授权已齐。在原944唯一树同步master后重新T-01冻结，不把前置950的验收替代本包标准操作proof、装配身份/失效和当前运行授权义务。T-02..05依次依赖T-01冻结结果；T-06在work侧AC全闭后，T-07在review pass后。任务与AC引用多对多：T-05覆盖AC-01..04，不以单行对应限制验证。

## 9. 复审记录

2026-11-05 draft handoff：`stage: new | plan_id: PLAN-944 | plan_revision: 1 | outcome: blocked（正式依赖/首探结论未齐；这是new交接判定，不是Plan状态） | next: 等待用户实施授权及PLAN-943 正式交付（D943）；先做§8允许的有界准备，不执行依赖未齐的任务。`

本轮只做文档静态预检与固定组名称核验；实施、产品验证、独立final review、Spec沉淀、merge/archive/cleanup全部未执行。current_step=0/7。非完成声明。

实际文档预检：Design54及六份Plan的本地链接、现存源码路径、0–10章节、七任务/六AC、Spec delta映射、drafting/0-of-7及943–948编号唯一性通过；`.next-id=949`。已核所列固定组名；相关跟踪文档`git diff --check`通过。该检查不证明任何拟议功能、产品测试或独立final review已通过。


### 2026-11-06 goal/work T-01 有界前探：语言前置缺口

新增授权：用户 `/goal 完成计划944` 授权 r1 的 work→独立 review→有界修复→merge/归档/清理全链；取代历史 draft 的“尚未授权”，不授权扩大公共语言语义/全模块系统。正式 D943 已核：归档 delivered，源码 `72ac9e7b341bd53de92cb53c9ecaf0716371a5a3` 已在当前 master 祖先链，17项合同及当前 Spec 可读。主检出原有 docs/design、docs/plans 草稿保持原样，未纳入本计划提交。

唯一实施树已创建并核身份：`D:/autostack/.wt/lang-944/auto-lang`，`plan-944-dev`，base/current `3b09e32b016ceafed97c4415d021626474fcadf7`；只读 sibling auto-down `58ea33f699efa5cb1194473a2d45dcec51006842`。实施树无源码修改、未提交实现。scope `D:/autostack/plan-artifacts/plan-944/` 已在其 `.retention/policy.json` 登记 owner=Codex、用途=T-01前探、状态=executing；probe源码/日志与显式外置 disposable target 全在该scope，无第二输出根。

实际入口：`cargo run --manifest-path D:/autostack/plan-artifacts/plan-944/probe/Cargo.toml --target-dir D:/autostack/plan-artifacts/plan-944/target`，从唯一实施树执行，exit0，冷编译/准备41.56s；运行4个固定探针，1个合法对照Checked成功、3个缺能力诊断符合观察断言；这不是产品用例PASS或两个consumer执行证据。没有运行固定完整职责组，因为尚未修改产品实现，不能把前探当T-05。

| 输入 | 观察 |
|---|---|
| `fn answer() int { return 42 } fn main() int { return answer() }` | 当前 scalar 统一前门→Checked 成功；未执行AV/Rust，42尚不是运行结果。 |
| 上例将首个声明改为 `pub fn answer` | Text / `source.unsupported` / `` `pub` is not part of this source profile ``，span 0..3。 |
| `use std.answer` 换行 `fn main() int { return answer() }` | Text / `source.unsupported` / `` `use` is not part of this source profile ``，span 0..3。 |
| 第一例将调用改为 `std.answer()` | Text / `source.unsupported` / `` operator `.` is not part of this source profile ``，span 56..57。 |

静态交叉核验：`experimental/ace/grammars/auto-source.at` 的 item/fn-item 没有 public/import 分支；`core/src/source/ast.rs::FnItem/Program` 没有可见性/模块导入事实；`source-query/src/consumer.rs::compile_scalar_product` 消费单个程序，`session::SourceQuerySession` 对该程序建签名catalog；现存source-query/store的恢复/失效不是模块解析器。不能删掉pub、拼源码、按字符串改名或伪造Checked来交付AC-01。此处缺口是T-01明确要求“缺语言前置则记录并修订依赖”的触发条件，不能仅落manifest声称整包接通。

探针源码SHA256 `243cd6b8caedbc2d82c781393aa126a84c9e16bc550cb3b90ddb31a9346cca5c`；日志SHA256 `f6a9877d344f83d0629c263ff773c9a5a0d76b5069f44880bf8f4fbb7b178444`。路径分别为scope下 `probe/src/main.rs`、`frontdoor-probe.log`。无源码/Spec交付、无AC完成；T-01部分调查，不能勾选，current_step=0/7；未进入review/merge，不产生debt豁免。

`stage: work | plan_id: PLAN-944 | plan_revision: 1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-944/auto-lang | branch: plan-944-dev | base_commit: 3b09e32b016ceafed97c4415d021626474fcadf7 | code_commit: none | dependency_revisions: D943=72ac9e7b; auto-down=58ea33f | task_ids: T-01 partial | evidence: four-case frontdoor-probe | blockers: missing public/import/namespace frontend prerequisite; dependency revision decision in §10 | next_responsible_role: user decision then auto-plan-new bounded revision`

`stage: goal | plan_id: PLAN-944 | plan_revision: 1 | round: 1 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-944/auto-lang | commit: 3b09e32b016ceafed97c4415d021626474fcadf7 | next: user`

2026-11-06 goal round 2：重读主检出§9/§10，尚未收到语言前置交付方式的用户决定；唯一实施树仍在原base，clean，无新实现交付。上一轮产生了真实前探证据与进入条件判断，属于progress；本轮无新增产品进展，等待同一明确决定，不把对话等待计为活跃进程的verified wait。按goal技能waiting规则未运行产品检查或重启探针。既有请求继续有效，不重复提问。

`stage: goal | plan_id: PLAN-944 | plan_revision: 1 | round: 2 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-944/auto-lang | commit: 3b09e32b016ceafed97c4415d021626474fcadf7 | next: user`

2026-11-06 goal round 3 / blocked audit：上一轮为no progress（只记录同一待决项），不是verified wait；本轮重读§10仍无用户决定，唯一实施树clean。该同一阻塞已连续出现在原始goal回合及两次自动续跑，达到三回合门槛。未运行产品测试或重复前探。按goal宿主规则将目标标记blocked；Plan状态仍为executing，0/7，保留唯一树和在途scope。解锁动作：用户选择先交付独立有限模块前置Plan（建议），或明确在944内修订任务承载前置，然后恢复goal。未放宽AC、未声称交付。

`stage: goal | plan_id: PLAN-944 | plan_revision: 1 | round: 3 | from: user | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-944/auto-lang | commit: 3b09e32b016ceafed97c4415d021626474fcadf7 | next: user`

2026-11-06 new dependency revision r2：用户答复关闭r1 §10的路线决定，已独立分配PLAN-950（有限公共函数模块与受检绑定前置）。本包仅新增D950依赖及继续执行入口，原任务/AC不删减，历史前探仍只证明r1基线缺能力；没有产品实现证据被提升。原944唯一树保留、无新建树。原始“未授权/未建树”陈述为draft历史，当前授权/树状态以§8及本记录为准。

`stage: new | plan_id: PLAN-944 | plan_revision: 2 | outcome: pass | changed: dependency D950, T-01 entry | next: deliver PLAN-950 then work T-01 | authorization: 用户先建并完成前置再继续944 | pending: D950 formal delivery; host goal resume`

2026-11-06 resumed work checkpoint：用户已恢复原goal active。按已选路线执行950，检查点dc9f63f4f在950唯一树；有限模块的AV与HIR→Rust实际分别返回42/7，950恢复/首编辑/正式复审合并仍未完成，尚无D950。本包仍0/7，未把前置部分进展计为944验收通过。

2026-11-06 prerequisite delivered：D950正式交付，source=`68cd53d435f7d3a6fc69fd911e474895dc387ed5`在master祖先；独立r1/修复1 final pass、三份canonical Spec精确hash、归档950 7/7、两个worktree/branch/group/材料scope与28外输出根实际清理均完成，无hold。有限module API只i32/bool，不声称943真实标准操作行为已执行；本包仍r2/0-of-7，原AC全部保留。周期cargo t all因950到期，由主检出单实例regress接管，不能与936现存CT gate竞争。下一步在原944树同步正式master并重做T01；无新用户范围决定，既有全链授权持续有效。

2026-11-06 work准备状态核验：原944树/branch身份已核，clean，从3b09e32b0真实ff同步master至5b71983bad7973127f4be4d1e5c3829acc455d12（含D950以及其他owner正式交付的953 capture EOF修复；不是本包新实现）。scope944原在途材料保留；T01产品前探/新manifest/query/provider/authority尚未实施，仍0/7。当前执行950到期的主检出单实例batch-regression-20261106，所有新输出及TEMP/TMP显式在该slug scope，batch基线原receipt940不提前推进；本包后续编译/测试待队列终态，不能并发第二full tier。

2026-11-06 due regression已结束：主检出单实例cargo t all完成37组/2828执行/1014原条件排除，702.068s，27组PASS/7组FAIL/3组NA，原raw exit1未改。protected source=config身份固定6b6b191c3。原基线红具名保留；新增heartbeat夹具首红隔离1/1绿，4个HTTP文件首红证实旧fixture路径literal转义失真（Win32 GetTempPathW/2W实际含\r，原源码未改），同scope scratch控制12+3绿。后继PLAN-954仅draft认领夹具，无生产回归确认，也不作本包依赖；.last-batch-regression.json已写真实明细与覆盖到953，不能称all-green。批量scope原材料将按政策清理，复现/结论已折入954与收据。950前置正式完成不变，本包原944树5b71983ba clean，下一实际步骤=auto-plan-work T-01有界闭环与模型冻结，仍r2/0-of-7；未把周期门或草拟954冒作本包产品实现。

2026-11-06 regress closeout：原完整队列/三项定向诊断/归因及原子数已在.last-batch-regression.json持久化，batch scope13365files/7943127877bytes已实际删除（1.848s，无links/hold），原主检出admission-0fm3rxcv 5files/56628bytes先迁入scope并逐内容hash核对后随scope删除。原始反例/输入/预期折入954，未因scratch隔离控制改变生产实现；未重复全仓队列。上一goal回合是progress（D950源码/规范/归档/清理真实完成），本回合也是progress（到期queue真实终态、定位新增夹具观察、登记954与提交新周期收据），原目标944仍active。现在进入944 work，后继不再等待周期门，0/7由实际任务验收推进。

### 2026-11-06 T-01只读接线调查（未冻结设计、未写产品代码）

当前唯一树同步commit=7ddeadcfd9bae8b778a384f4d9f23e11fc2f172a，branch plan-944-dev，clean；仍0/7。已按实际源码定位以下接缝，供下一work有界probe验证，不把下列候选机制当交付合同或验收绿：

- D950公共入口`compile_module_product`、`ModuleSession::{from_product,from_recovery,prepare}`、`ModuleAvTarget`与`ModuleA2xTarget`已正式提供模块namespace/可见性/原Span、真实CST恢复与单体事务。两个consumer永久跨端验证现在归AV conformance（A2X生产及tests不依赖AV）；新增中立data依赖须同步原boundaries精确sanction并保真实禁止负控，不能再把双端case放回A2X目录。
- 真实943事实由`stdlib_assembly::loader::scan_inventory`、`contracts::{scan,validate,audit_providers}`生成，原17 operations/compatibility及operation_proofs。至少一项真实标准操作reference应由实际source/provider签名证明进入新manifest，不能手写一份自证合同；`reference::verify_native_reference`只处理CORE_MODULES公共callee，对str native-only不返回公共proof，不能拿它假称通用。下一probe优先核实际公共`io.say`/console.say等与生产NativeInterface的充分proof；若仅取byte_length的producer-only proof必须准确保留等级/来源，不能升格ACE callable support。945/947行为交付仍不由测试module42/7冒充。
- 原schema4 AssemblyManifest::freeze/common/consumer FNV语义保持；新ACE producer须明确版本化profile/host/platform/ABI/runtime及操作/物理provider依赖。候选中立数据放semantics-contract（目前只依赖blake3；任何新增必须保持无auto-lang/VM/backend反向依赖），root legacy adapter只提取真实已核事实，禁止用legacy parser去编译新module输入。
- source-query真实SignatureCatalog集中在catalog.rs，`fn_sig_with`取catalog.get().version，`infer_body_with`按own signature/body key及实际sig_reads失效。可研究把显式effect/context版本作为声明轴接入原catalog/事实，不复制solver，不把整个manifest hash塞每个body键以致无关函数全miss。`ModuleStore`现source-modules-root/2有CST origin、decl_env、facts、targets，Root2不含新assembly contexts；新axis若入存储须新schema/recipe，不重解释旧payload/键。
- `ManagedStore::replace_current_root`已有liveness+SQLite事务CAS替换。可研究新assembly snapshot保护unpublished source/target required闭包，仅一个外层当前根原子发布；不能分两次发布source-current/assembly-current再把后半失败叫“保旧”。这只是待probe策略，现ModuleStore.prepare/restore可用宿主receipt读未成为CURRENT的不可变根，须真实验证生命周期/GC/失败下一次恢复。
- AV当前per-run权限路径为av-core::host::authorize(module,grants)，只适用真实host-console MIR；不能把纯模块伪装host-console以制造授权绿。新assembly只能登记显式要求/接当前调用授权，不能保存grants/指针或造CAP求解器；纯module42/7与未来947 console行为分账。原A2X current provider/fragment key/runtime/print identity及现有single-body emitter/verify需继续复用。

Design54只在主检出未提交设计文档存在，实施树未包含该WIP；已只读取材，当前规范仍以canonical Specs与r2 Plan/正式D943/D950为准。未将其或其他会话设计WIP混入提交。后继T-01须先做原真实两个consumer+真实reference proof的有界闭环，再冻结模型/读写版本/实际输入，不以这段调查替代T-01。


### T-01 冻结（2026-11-06；r2，唯一树7ddeadcfd基线）

有界前探已闭：原950两条真实consumer用例在944树执行，`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-av --features test-case-batch --test conformance module_sources_950::` exit0，2执行/0失败/0排除（897过滤），准备/编译35.44s、运行5.18s。独立oracle42/7、恢复首编辑及下一次读回仍有效；非944全包验收。日志`D:/autostack/plan-artifacts/plan-944/module-baseline.log`。真实943公共proof选`io.read_line`：当前production接口#1150，公共io.at签名`()->str`与独立adapter一致；临时probe exit0（17operations）。console.say没有普通native绑定，原probe两次unwrap失败如实保留，不能将专用console发射冒充普通native reference。text.byte_length是native_only，不升格公共proof。read_line行为交付仍属947，本包只接signature evidence。

冻结普通实现选择（不改AC/阈值/仓范围）：

| 入口/规则 | 位置及版本 | 生命周期/关闭条件 |
|---|---|---|
| 中立显式操作/声明/来源/物理provider事实 | `semantics-contract::assembly`，`ace-stdlib-assembly/1`，独立新domain，不改legacy4字段顺序/FNV投影 | 不依赖root/VM/任一backend；declarations显式requirements、无CAP求解；仅有限scalar模块源，证据reference不扩展可调用profile |
| 真实std证明导出 | root `stdlib_assembly::ace::current_operation_reference`，重新读当前公共声明+最终NativeInterface独立contract | 归一真实相对producer位置、去掉build根；保声明hash/adapter合同hash、语义版本/requirements；改变当前adapter须拒，不能序列化callee地址/授权 |
| 新旧身份/双指纹 | 共同指纹包含profile/host/platform及语义source/声明/reference；消费者指纹另含producer/ABI/runtime/toolchain/provider闭包；原`AssemblyManifest::decode_v4`只以旧字段序/FNV读取4 | 新模型未知schema/producer/profile、重复项/坏hash使用前拒；恢复拿当前独立输入admit；异绝对根合法来源不进入portable新身份 |
| 查询/effect轴 | 复用`SourceQuerySession::catalog`/SignatureCatalog，按真实owner显式声明context增强signature version；默认无context保持原键 | body变只重产owner；显式需求变重验实际读者；无关函数solver/emit/lower/print为0；不把全manifest塞全部body键 |
| 存储/发布/恢复 | 复用`ModuleStore`、SyntaxArtifactStore与ManagedStore；新context根须显式新schema/recipe；旧root2保旧义 | 唯一外层assembly CURRENT通过已有事务CAS发布并保护source/target required闭包，inner root只prepare；保旧/取消/恢复下一读必须真实验证 |
| 两端物理身份/运行授权 | 扩原ModuleAvTarget/ModuleA2xTarget，按真实受影响owner读取provider版本；原ABC/fragment admission/lowering/print继续复用 | 不造solver/emitter/store；当前运行每次从调用方重新准入显式要求，拒绝dispatch0；持久化不携grants/host地址。纯scalar不伪装host-console |

T-02已开始模型/adapter/codec实现，T-03..05仍待完成。新模型定向永久测试`ace-semantics-contract --test conformance assembly_944`已3执行/3绿，0排除、11过滤，准备3.79s；其结果只证明模型身份规则，不证明实际consumer/cache已接通。root探针/旧读取验证在进行；既有root编译131warnings、950双端test编译4warnings为实际旧基线，后续逐项归因，不宣称warning-clean。独立review、最终职责组并集、Spec沉淀、归档/清理均pending。


### T-02..04 实施检查点：模型、真实proof、查询恢复与AV装配

唯一树已提交基础`4c823e072`、实际reader物理输入`6e25e1631`，随后AV装配单元另提交（见树HEAD）。仍executing 1/7，T-02..05均未整体关闭；不能把这个单元当完整交付。新增中立DTO只引入serde/serde_json；root SDK只依赖data-only contract，不依赖ACE compiler/worker/backend/store。双Cargo.lock只增加该合同包/codec依赖。sema_boundary_manifest_dependency_audit精确声明新增serde数据codec面，原禁止product/backend清单保留；定向1执行绿。

验证（全在944树，scope根统一，TEMP/TMP指向scope/scratch）：

- `cargo nextest run -p auto-lang-core --lib --target-dir D:/autostack/plan-artifacts/plan-944/target -E 'test(actual_public_reference_retains_proof) or test(schema4_read_preserves_original)'`：2执行/2绿，1640非选择。最终准备29.06s、运行0.300s；此前首次冷准备121s/运行0.324s同两项绿。schema4带独立固定wire/FNV金样`common=133f7fdee4742800, consumer=93ff2a3f7992bab7`，不是加新字段再重hash；读取仍原field序/FNV。真实io.read_line proof保`()->str/stdin/来源/独立adapter合同hash`、无绝对根/数字ID冒充操作身份，实际替换独立adapter参数/返回为i32后公共proof拒绝；native_only/未知操作拒绝。
- 中立模型3执行绿（首次准备3.79s，末次1.62s）：实际common/consumer分离、provider owner区分、ABI/runtime/toolchain/语义签名/版本/requirements陈旧拒绝，schema/profile/producer/重复项/未知字段/绝对路径坏输入拒；codec读/写共4MiB资源上限，无源码语法支持阈值变化。只证明模型。
- source-query `--lib modules::`：10执行/10绿、37非选择，准备5.29s、运行0.20s。包含新Root3显式contexts，原Root2仍原义；原CST恢复/取消/CAS发布失败等8项继续绿，新effect只真实求解声明者+实际reader共2，local命中；重启无scan/parse/solver，首编辑1solver，再次读回0solver，错误currentcontexts/旧restore入口拒。
- 新batch候选2执行绿（末次准备0.95s、运行0.01s、48非选择）：同env/generation/不同slot先全验后swap，重复拒绝generation仍0，外来/旧代候选拒。取消后的candidate fact页不能隐藏下次重试的target更新：受影响owner集合按旧/新已提交context差及真实read set选取，不能按候选cache是否hit来决定是否更新目标。无text编辑时0tokenize/parse，两个实际单bodyemit receipts均real_body_plans=1。
- source-query `--lib assembly::tests`：2执行绿（准备1.25s、运行0.01s、49非选择）：真实模块namespace/同名用户函数不同owner/context；缺声明/坏签名/mode/source/namespace在solver0时拒；函数physical input仅纳own/direct实际读取声明、当前producer/ABI/runtime/toolchain和其provider闭包，无关owner不变。
- AV版本化入口已接原lower_function_unit/ABC envelope/FunctionCode/ProgramSnapshot，Root2带manifest/current requirement闭包，每unit wire/2键带实际function input version；旧Root1/wire1字段和hash不改。新`run_authorized`每次调用当前checker，不持有或序列化grants/宿主地址；新模式普通run受控拒绝。受影响reader函数实际lower2、无关local的执行FunctionCode为原Arc；同source的旧assembly candidate因root身份不同拒。重启source 0scan/parse/solver、AV0lower；旧入口/陈旧manifest拒，恢复后拒授权和重新授权42均真实执行验证。该AV用例1执行绿，899非选择，准备16.18s、运行0.17s；首次check两个版本常量根路径错误exit101，已改为真实contract模块路径，随后check exit0（2.35s）。
- AV扩展后重跑950原两个双端consumer用例：2执行/2绿，898非选择，准备0.17s/运行2.48s；原Root1/qualified42/user-shadow7及双端恢复仍有效。不是新增Rust装配入口验证。

日志均在`D:/autostack/plan-artifacts/plan-944/`：root-adapter-tests.log、reference-probe.log、context-root-test.log、source-modules-unit.log、batch-candidates.log、assembly-binding.log、neutral-dependency-audit.log、av-assembly-check.log、av-assembly-test.log、module-baseline-av-extension.log。merge时摘要/hash折入本文件后整scope删除。实际既有warnings：root lib131/libtest145，AV lib3/test4，source-query libtest1；新文件无新增warning，完整归因/health review仍pending。

后继必须闭：HIR→Rust真实装配入口与per-owner fragment键；两端同一次事务发布/失败保旧/下一恢复；真实std proof进入实际双端manifest的永久集成；provider内容与body编辑真实新值、恢复首编辑；当前授权断言；独立review/最终固定职责组/Spec/归档/清理。无用户决定或外部阻塞。可复用ModuleRoot3本身作为唯一外层CURRENT（已required保护targets与CST/facts）而非额外新store/第二层发布；须实证两target manifest/currentcontexts一致和单CAS失败回滚，不能只在文档称已闭。这是原“唯一原子外层根”实现的收敛，不改变AC。

## 10. 待澄清事项

- 首批符号/合法source/profile/接口位置由943及本包T-01有界前探冻结；§5/§6给出了最小关闭条件，缺能力保持具名阻塞，不静默降scope。
- 完整CAP仍守Design46 A/B/C1–C3门；本包无权提前完整求解或以候选字段宣称已实现。
- 实施授权已由2026-11-06 `/goal 完成计划944` 给出；无额外预算/模型约束。跨仓仍只读。
- **已决定（2026-11-06）**：用户选择“先建并完成前置计划，再继续944（推荐）”。前置为[PLAN-950](archive/950-ace-finite-source-modules.md)，按其有限public/import/可见性/查询恢复合同正式交付D950后继续本包；944的AC-01..06、支持阈值和标准操作proof等义务不变。无需再次请求相同范围施工批准。宿主目标已由用户恢复active；继续交付950，不创建第二目标。


