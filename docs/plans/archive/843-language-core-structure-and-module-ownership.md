---
plan_id: PLAN-843
status: archived
feature_name: 语言核心模块职责与大文件结构整理
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-28
plan_revision: 1
current_step: 12
total_steps: 12
supersedes_spec_components: [docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/types/overview.md, docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/runtime/overview.md]
new_spec_components: [docs/specs/language-sdk/architecture.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [language-sdk, auto-lang/frontend, auto-lang/types, auto-lang/vm, auto-lang/trans, auto-lang/runtime]
depends_on: [PLAN-831, PLAN-832, PLAN-841, PLAN-842]
---

# [PLAN-843] 语言核心模块职责与大文件结构整理

> **全Plan启动锁G843**：G842且U842。包含T-01盘点/probe/度量、fixture/测试和建树，门未齐一律不开始；只核验上游状态/完整交付收据。启动证据定义见[共用合同](reports/m2-governance-order-841-844.md)§1。本轮仅规划。

## 0. 变更摘要

完成R12结构半部：对纯SDK前端/类型/编译/VM/FFI/转译/基础服务建立清楚的实现所有者和窄接口，按语义责任拆分已观察七个超大热点及lib入口。保持单一AST/类型存储/执行状态与算法，不以继续堆crate或机械分块作为结构治理。

本Plan是4份大模块合同之一，包含实际生产改动、旧实现退出、测试/三仓消费/版本与发行同步，不另按小探针取号。不接手831/832尚未完成的搬仓验收，不扩大为全语言IR替换、新backend/功能或远端发布。

## 1. 目标

- Parser、TypeStore/推断、编译会话、Codegen、Engine、native/FFI与trans职责/状态所有权明确，内部依赖有向、跨模块接口最小。
- 七个已观察热点或其832后继完成按职责实际拆分，入口不再承载多域完整算法；原公开路径/类型身份/默认行为保持。
- 编译、执行、转译、诊断、缓存/服务与资源语义不改，测试/生成/脚本随owner，结构指标可比并交844去重。

## 2. 架构方案

T-01冻结结构基线/职责图：七热点每个当前或后继路径、生产/测试/生成行分类、输入→状态owner→输出、窄接口、具体拆分单元和验证映射。前3Plan已经移除/整理的热点以来源证据核销，不重复重构凑任务；仍存在的七热点都须交付主题模块，不能把其中几项延后。根lib按入口/API组合与机制分离。

拆分必须按解析语法族、类型/绑定、字节码发射、执行派发/生命周期、native族、FFI编码/宿主服务、转译目标/表达式等语义职责组织；同一共享状态仍唯一owner。主题module不是空re-export、文件切两半、include!拼大文件或新的巨大common。T-01将热点的实际可量化目标冻结：入口文件仅保类型/构造/明确派发与组合，主题模块数量/生产行数/跨域私有访问比基线有可复核改善；不得只靠压缩格式/挪测试降低数字。

保持public函数/trait/inherent/类型身份、Parser pub字段合同、TypeStore/InferenceContext/会话/缓存/VM Rc/线程亲和/锁外回调、native注册及执行求值序。内部visibility只扩至最小pub(super)/crate接口，不一律pub逃避边界；不复制多个Store/cache/registry。Type的语法模型与运行时值类型不强行合并。

本Plan只结构和必要导入/模块接口整理，不修改算法/Opcode/ABC/ABI、优化缓存/新IR或接未实现checker；源码指纹随受管规则正常失效，但同输入的语义/生成产物除可解释路径/身份注记外保持。重复算法仅具名记录交844，不能因为文件移动而宣称去重；借检查等未完成能力不因整理被删。

## 3. 技术栈

现有Rust/Cargo与三仓固定Git版本、单源AST/值/词法/Parser API/SDK、UI文法/生成器/runtime及Man CLI，复用777边界、792发行与已有真实consumer/双端设施；不新增通用框架/平行runner。实际包名/路径以启动后上游来源映射冻结，当前源码仅观察基线。

## 4. 需求分析与背景调查

用户2026-10-24明确要求按4份较大计划规划交付目标、验收与顺序/831832启动前置；继承此前保守排程，全部实施任务严格等完整前置交付。**本轮只在语言主检出建立合同/设计/路线，不改实现或canonical Specs/ledger、不建树/agent、不写旧仓/远端**。已有授权只覆盖目标与规划，未来work按明确执行指令和门启动；常规机制选择不需重复批准，改变既有支持/兼容承诺则具体needs_replan。

从[Spec入口](../specs/overview.md)定位，[AST](../specs/auto-ast/project.md)、[词法](../specs/auto-lex/project.md)、[Parser API](../specs/auto-parser-api/project.md)、[SDK](../specs/language-sdk/project.md)、[前端](../specs/auto-lang/frontend/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[旧仓](../specs/repository-boundaries/project.md)与[发行](../specs/repository-boundaries/generated-ui-distribution.md)为当前依据。部分旧Spec数量/路径滞后，不以旧聚合描述覆盖实际模块代码/交付；观察与假设、源SHA/hash见[输入](attachments/m2-governance-batch-20261024/inputs.json)。

已观察写面：SDK parser.rs、types.rs、infer/、compile.rs、vm/{codegen,engine,native}.rs、vm/ffi/{stdlib,http_server}.rs、trans/rust.rs、lib.rs及database/session/scope/host_capabilities；输入记录当前原始行数，未来按841/842发布后的实际路径重采。

当前831 executing 3/12、832 drafting 0/12；都未完成M1交付。新机制/真正可删候选/未来路径不声称已证明，T-01在G843后承担有界决策和实际编译/消费probe，关闭条件见§2/8。

## 5. 详细设计

### 5.1 启动、所有权与执行入口

G843对T-01..12逐行生效，depends_on是全Plan启动门，不是只约束最终集成。U定义为全部AC/final review/冻结门/实际合入/Spec与受管ledger/来源交接/归档收尾闭环；阶段pass不放行。门未齐只读上游状态/交付收据，保持drafting 0/12，不提前盘点/扫描/probe/改fixture/建树或启动实施agent；无新后端status/调度功能。

G843后T-01读取所有已发布三仓规范/agent指令、前置delivery commit/路径映射，新增attachments/843/contract-baseline.md：具体producer→数据/状态owner→consumer/产物、支持/预算/错误/退出规则、正反例与可执行oracle、未知决策/owner/关闭条件。假设必须在依赖实现前用最小真实整链probe证明；无解则保范围转needs_replan，不用单元假绿偷降AC。

### 5.2 三仓生产接线与来源

按共用合同§2–4及§2本Plan专门设计实施。所有类型/状态/算法单一owner；normal/build/dev与feature图无UI/Man反向污染SDK，不靠宽化777或临时[patch]双source过门。变动同时更新真实三仓consumer、固定版本、测试ID/alias/CI、build输入/生成import/资源/producer与缓存身份；832的旧发行失败合同继承，不新造弱事务。

版本升级/兼容差额记录在来源清单；外部正常消费与具名错误/缺件/取消/退出必须实际验证，未测环境pending。正式API或序列化不兼容仅按本Plan明确版本迁移合同实施，需改变既有承诺则停止该项给具体决策，不把用户规划授权扩张为随意删除稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md | SDK/frontend混合热点→按语法/状态owner的主题模块和稳定公开入口 | 入口与实现分离 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-lang/types/overview.md, docs/specs/auto-lang/vm/overview.md | 类型/VM多域共享访问→唯一TypeStore/会话/执行owner与窄内部接口 | 保状态身份 | AC-01..05 |
| SD-03 | modify | docs/specs/auto-lang/trans/overview.md | 巨型Rust等转译文件→按表达式/语句/声明的同算法主题实现 | 结构而非改语义 | AC-01..05 |
| SD-04 | modify | docs/specs/auto-lang/runtime/overview.md | FFI/stdlib/服务/根lib混合职责→编码/绑定/服务/组合各自有界owner | 保运行资源合同 | AC-02,03,05 |
| SD-05 | add | docs/specs/language-sdk/architecture.md | 缺显式SDK内部架构→职责DAG/状态owner/七热点映射/规模与稳定接口规则 | R12结构半部 | AC-01..05 |

## 6. 测试设计

下列Case均owner_stage=work，由T-01冻结实际输入/入口与oracle，T-02..08逐步建立永久定向控制并补真实集成；**本轮尚未执行**。原ast-model/ui-grammar/language-sdk/generated-ui/tool-client等consumer与语料/投影golden按真实owner复用，不以重写golden遮回归。新增fixture用`test/fixtures/modularity/`下具名目录，具体接口T-01冻结；新接口未存在前不把未来命令当已执行。

| Case | AC / invariant | 输入/真实入口 | 可观察oracle与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 | T01七热点/后继的真实调用/依赖/主题单元与生产行统计 | 原多域完整实现已归主题owner，入口仅组合/派发；禁止include机械分片/虚re-export；消失热点须前置交付证据 |
| V-02 | AC-02 | 独立API compile pins、同会话/不同会话/clone、注册替换/多线程和关闭 | 公开消费编译、状态身份与优先级不变；只窄内部接口，无第二Store/registry或资源延迟泄漏 |
| V-03 | AC-03 | 同源Core正误程序、类型/源位置/求值序trace、字节码/VM、Rust/C/其余现行trans | 结果/结构化错误/产物与基线匹配，路径/构建身份变化单独解释；不用改golden消漂移，资源错误/服务ready保持 |
| V-04 | AC-04 | 真实三仓CLI/生成/UI运行与外部SDK，无UI源码Core、fresh/locked构建 | 不依私有新结构也可消费、单source与纯SDK禁止边保持；无条件关feature不能当通过 |
| V-05 | AC-05 | 迁前后按ID的测试/配置选择、源指纹/资源缺失负控、结构前后表 | 原覆盖不丢、负控红与恢复绿，生产/测试/生成口径分列，重复候选不计作本Plan已净删 |

work/review只必要check（真实SDK/被改包）、相关包/模块/fixture和生产反例、实际外部工程构建运行；777 strict与792适用面/缺源负控不能省。类型/VM/trans/UI生成触面final review通过后，merge在冻结三仓输入统一裸cargo t及tv/tt/tu等适用档（按迁后alias/owner等价入口），tf到期语言主检出单实例并覆盖迁出家族。修复先完成本轮全部findings和反例/邻接组合再提交送审；无单finding全档/边改边审。缺环境/上游不能fake正控，当前无指定agent/model，不创建会话；本轮Category A无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | V-01 职责/状态owner图与真实依赖一致，七热点/后继按职责实拆，零机械分块/新增巨common。 |
| AC-02 | work | V-02 对外API/类型身份、符号/会话/VM/缓存/注册与线程所有权不变，无复制状态或过宽pub。 |
| AC-03 | work | V-03 固定输入Core解析/诊断/类型/字节码/VM与各trans行为保持，错误/资源/服务控制无回归。 |
| AC-04 | work | V-04 三仓UI/Man/外部SDK与生成工程/fresh/locked仍真消费稳定接口，无新增UI/Man反向边。 |
| AC-05 | work | V-05 测试ID/alias/资源/build输入/指纹全迁，结构生产指标与去重候选清单、SD/来源齐备。 |
| AC-06 | review | 独立最终review绑定r1/实际三仓commit及依赖/配置/输入，全AC/SD/遗漏/警告/格式/debug审查通过；merge门pending |
| AC-07 | merge | 并集合流影响复审/冻结门通过，实际源码与consumer版本交付、canonical/受管ledger/来源读回、归档收尾与guard清理闭环；不以部分发布放行后继 |

## 8. 执行步骤

12项，9 work+1 review+2 merge；全Plan启动门覆盖所有行，保守串行，不提前做独立小任务。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G843完整核验 | 冻结七热点/后继、内部DAG/状态owner/规模和API/行为基线及具体拆分目标，T01记录已关闭热点来源 [✅ 已完成：合同基线](attachments/843/contract-baseline.md)+[度量基线](attachments/843/measurements-baseline.txt)；七热点全存于 crates/auto-lang-core/src/（无一被前置核销，PLAN-814 edb78b2ef 整体搬入）；组 D:/autostack/.wt/lang-843/{auto-lang@codex/plan-843-dev f99facb27b, auto-down detached c3464d0, auto-ui detached bd228b0 只读固定版本}；worktree cargo check -p auto-lang-core 预检探针绿（37.13s，145 预存 warnings 为基线）] | AC-01..05 |
| [x] | T-02 | work | G843；T-01 | 整理parser/类型推断/compile边界，语法族主题模块与窄共享访问，保公开字段/单Store | AC-01,02,03 |
| [x] | T-03 | work | G843；T-02 | 拆codegen为职责明确发射/绑定等主题模块，保字节码/错误与执行次序 | AC-01,02,03 |
| [x] | T-04 | work | G843；T-03 | 拆VM engine/native注册/派发/生命周期主题，保线程/退出/回调与状态身份 | AC-01,02,03 |
| [x] | T-05 | work | G843；T-04 | 拆FFI stdlib/http_server编码/绑定/服务职责，保所有权/错误/TCP ready与端口策略 | AC-01,02,03 |
| [x] | T-06 | work | G843；T-05 | 拆Rust转译等热点的表达式/语句/声明/输出策略，复用原算法，不引共享大杂包 | AC-01,03 |
| [x] | T-07 | work | G843；T-06 | 收敛lib/内部模块出口、移测试/资源/build输入/alias与指纹，三仓公开消费compile pins | AC-02,04,05 |
| [x] | T-08 | work | G843；T-07 | V-01..05与Core/VM/trans/服务/真实UI外部消费定向、fresh/locked和状态反例完成 | AC-01..05 |
| [x] | T-09 | work | G843；T-08 | 按七热点逐项核销结构交付、前后指标/owner图/844候选及SD-01..05、attachments/843/handoff.md冻结 | AC-01,05 |
| [x] | T-10 | review | G843；T-09冻结提交 | 同组独立完整AC/SD/实际consumer/反例/来源与健康审查；不先跑整档，不将merge门当已通过 | AC-06 |
| [x] | T-11 | merge | G843；T-10 pass | 并集影响复审、冻结三仓适用门与有序实际合入/固定版本/来源读回；若改变输入重审影响面，暂不放行后继 | AC-07 |
| [x] | T-12 | merge | G843；T-11全部交付收据 | canonical/受管ledger/索引沉淀、归档与逐树fresh wt-guard clean清理，整Plan闭环构成U843才放行下一份；保历史/恢复/交付分支 | AC-07 |

G843未满足前禁止创建/认领以下组、分支或work agent。未来组`D:/autostack/.wt/lang-843/{auto-lang,auto-ui,auto-man}`，每个实际需写仓各一树/分支`codex/plan-843-dev`；只读consumer用固定版本clone而非借其他计划WIP，未需写仓不多建树。work/review/merge沿同组顺序交接，不建第二审查树；若auto-down为现有可选依赖则具名固定版本/detached兄弟，只解析不改实现。禁止junction/symlink，主检出只簿记，清理前每树fresh guard clean且保留实际交付/legacy恢复源。

## 9. 复审记录

2026-10-28 / stage: merge / checkpoint: **cleaned（U843 全闭环）** / evidence: 四树 fresh wt-guard clean（auto-lang/auto-down/auto-ui/auto-man 各报 clean）；worktree 经各仓 `git worktree remove` 正规移除、分支 codex/plan-843-dev 删除（=交付提交 37cf4b19c 已合入）；组目录 D:/autostack/.wt/lang-843/ 回收（probe-t01/review-t10/merge-gates 本地探针脚手架随组删除，耐久摘要已入库本计划与归档前记录；无 plan-artifacts/plan-843 目录，无 target/pdb 需回收；.wt 下 843 残留=0）/ 批量回归到期判定: **due**——收据 last_covered_plan_id=858（2026-10-28T03:05 认证 pass），其后 825（%5==0）/823/843 相继落地；在案 last_due_check 已记录主检出有他会话 tier 持续占用，按 2026-09-30 裁定须空闲窗口主检出单实例执行 / next: 交接 /auto-plan:regress（tf+tt+tb）待机器空闲；U843 成立，G844（=G843且U843）放行 844。

2026-10-28 / stage: merge / plan_id: PLAN-843 / plan_revision: 1 / outcome: **pass（12/12 交付闭环，U843）** / delivery_commit: lang `37cf4b19c`（rebase 链：work 9 提交基 f99facb27b → review 冻结 8bad8002a → pin 迁移 e3c0a53d0 → specs 707213fd2 → master 前移两次（823 落地 ed7b70533 前含 crates 变更、826 簿记 1bb5de5a5 仅 docs）后重定基，range-diff 1-9/11/12 全等、10（fmt）因 823 已改 ac_av.rs 上下文差异、最终树 spec/代码净效应=我方 84 文件） / checkpoints: prepared=T-10 pass 绑 8bad8002a+SD 冻结；landed=master tip==37cf4b19c ff-only 零合并提交（主检出他方 docs WIP 未动）；ledger_refreshed=离线投影 v109→110（服务不可达；backup .autoos/specs.json.bak-p843-merge；+P843-ARCH-CONTRACT/P843-REVIEW-R1/P843-MERGE-R1，removed/changed=0 零扰动证明+回读验证）；archived=本文件 git mv 至 archive/ + 全部收据入库 / gates（最终基线 37cf4b19c）: 裸 t 3203/3203（420s；首跑红1=CTE 冷态 release worker 超 nextest 120s（842 同款冷态，串行预构建后过）；红2=plan705 callgraph 门源码 pin（期望 serve_blocking_stdnet 保留位）+plan716 trans 裸名臂 pin——两处源码 pin 随 owner 迁移至 server_dispatch/call_emit，断言语义不变，定向复验 19/19+14/14）；tv 163/163；tt 3574/3574（首跑 fail-fast 瞬态、同配置整档复跑全绿，842 在案模式）；fmt -p auto-lang-core --check 归零；tb/tu 不适用（未触 book/UI 生成器）、taa 不适用（零 aavm 触面） / 823 并发说明: 首轮门绑 9fff2c5a0 完成后 master 前移（823 落地含 crates/auto-lang-core 变更）→ 组合基线全门重跑，3203/3574 较首轮 3192/3563 增量为 823 新测 / landing-is-not-deployment 观察: 主检出 release 二进制/发行安装未随合入重建（本轮为结构重构、生成产物语义不变，再发行由后续发行流程/批量回归后进行） / cleaned: 见归档后清理收据 / next: U843 成立，放行 844（G844=G843且U843）。

2026-10-28 / stage: review / review_scope: final（T-10） / reviewer_identity: 与实施同会话上下文（无独立模型/会话可用）——结论自工件重构：逐热点内容守恒审计（fmt 前基线 f99facb27b 行多重集精确对比，native/http_server/rust/lib 零差、codegen +40/engine +25 恰为尾递归段包装、stdlib ±3=记录在案的 super::STORAGE_MAP、parser 仅差 mod tests 包装行）、尾递归链环归属与段首臂序独立核验（codegen 8 段/engine 5 段单线无跳段、末段保留原兜底臂）、测试守恒 6259/6259 #[test]+430/430 #[cfg(test)]、fmt 前后语义中性验证 / plan_id: PLAN-843 / plan_revision: 1 / outcome: **pass（final）** / worktree_path: D:/autostack/.wt/lang-843/{auto-lang,auto-down@c3464d0,auto-ui@bd228b0,auto-man@4cc912e detached} / branch: codex/plan-843-dev / reviewed_commit: lang 8bad8002a（=06b5d67b5+fmt 修复提交；发现 F-1 后修复重冻结并全量复跑受影响面） / base_commit: lang master f99facb27b / dependency_revisions: down c3464d0、UI bd228b0、Man 4cc912e（组内只读 detached，842 三仓交付提交） / spec_inputs: language-sdk/project.md（M814-A+841 中立端口态）、frontend/types/vm/trans/runtime overview、specs/README §4 / acceptance_results: AC-01 pass（守恒审计+零机械分块+入口 pub(crate) 不升+主题模块 6/13/9/12/8/5/6+根 2）；AC-02 pass（Parser 17 pub 字段/单一 Store/registry、兼容门面+LSP 编译 0 错、API 面计数不变）；AC-03 pass（冻结基线 8bad8002a 定向面全绿：parser177+codegen56+engine55+native94+ffi 215(work 期)+http191+trans349+plan738 75+run6；sdk-consumer 5/5；语料 golden 归 merge tv 档）；AC-04 pass（auto-lang/auto-lsp 编译、language-sdk-consumer 5/5、three-repository-consumer 3/3 固定版本消费，含 auto-man 组内 detached 兄弟补建）；AC-05 pass（指纹面随 owner 扩展 manifest+31/loader+8、plan738 pin 迁移、rust_fn 153/153、测试 ID 零变化）；AC-06 本记录（merge 门 pending 显式） / pending_closeout_gates: merge-owned 裸 cargo t + 触面档 tv/tt/tu/tb（VM/trans/UI 生成触面；本计划未触 book/book 生成器与 UI 生成器代码，tb/tu 按 T-11 冻结输入适用面判定）+ tf 到期批量回归（主检出单实例） / findings: F-1（已闭环）：拆分引入 238 处 fmt 漂移→rustfmt 72 改动文件→crate fmt --check 归零、check/parser/codegen/consumer 及全部受影响面复跑绿；无未闭环 finding；残留口径（三入口超自设目标/call_inner if 链/844 候选）按 handoff §4 记录不抵扣 / evidence: [handoff](attachments/843/handoff.md)、[度量后](attachments/843/measurements-after.txt)（本地工件未入库）、复审审计脚本 D:/autostack/.wt/lang-843/review-t10/audit.py（同前口径）、各定向面原始输出见本记录与 §9 work 记录 / next: merge T-11（冻结 8bad8002a 上跑 merge-owned 门后有序合入；规范沉淀 SD-01..05 于 T-12）。

2026-10-28 / stage: work（T-02..T-09 完成，work 阶段收口） / plan_id: PLAN-843 / plan_revision: 1 / outcome: **executing 9/12，T-02..09 pass；非全计划完成** / worktree_path: D:/autostack/.wt/lang-843/{auto-lang,auto-down@c3464d0,auto-ui@bd228b0 detached} / branch: codex/plan-843-dev / base_commit: lang master f99facb27b / code_commit: lang 06b5d67b5（=master f99facb27b + 9 提交：08a3f0808/bf7355b0c/cc88ab344/e32c26997/8112ad3fa/297892c9a/f3299da2e/8eb32baaa/矩阵再生成） / dependency_revisions: down c3464d0、UI bd228b0（只读） / task_ids: T-02..T-09 / evidence: [handoff](attachments/843/handoff.md)（七热点逐项核销/结构指标/残留口径/844 候选）、[度量后](attachments/843/measurements-after.txt)、[度量基线](attachments/843/measurements-baseline.txt)、[合同基线](attachments/843/contract-baseline.md)；七热点入口生产行 10376/11372/10472/9540/10110/6668/24894→2175/1274/2131/2367/5679/4156/5960、根 lib 3508→879；三入口超自设量化目标已按 §4 残留口径记录（职责实拆/零机械分块成立）；两巨型方法（compile_expr_inner 6323/run_one_instruction 8865）尾递归派发分解，臂体逐字保留；pub(super) 最小加宽编译器驱动，入口计数不升；cargo check lib+tests 0 错误（149=145 基线+4 条跨文件报告拆分，主体逐条核对）；定向测试 1344+ 全绿、language-sdk-consumer 5/5 多轮复跑；指纹面随 owner 扩展（manifest+31/loader+8/rust_fn 属性 153/153 守恒/plan738 pin 迁移 75/75 绿）/ pending: T-10 独立 review（merge 冻结门=裸 cargo t+触面档仍 pending，按 2026-10-14 裁定归 merge）；call_inner if 链与三入口残留=844 候选不抵扣 / next_responsible_role: review T-10（同树冻结提交 06b5d67b5 上独立复审）。

2026-10-28 / stage: work（T-01 完成） / plan_id: PLAN-843 / plan_revision: 1 / outcome: **executing 1/12，T-01 pass；非全计划完成** / worktree_path: D:/autostack/.wt/lang-843/{auto-lang,auto-down,auto-ui} / branch: codex/plan-843-dev / base_commit: lang master f99facb27b817fade1e5c2f03891c1ff418e28d2 / code_commit: lang f99facb27b817fade1e5c2f03891c1ff418e28d2（源码零改动，纯盘点冻结） / dependency_revisions: down c3464d0 detached、UI bd228b0 detached（只读固定版本，842 UI 交付提交；crates/auto-lang path 依赖 ../../auto-ui 使其参与 workspace 解析） / task_ids: T-01 / evidence: [合同基线](attachments/843/contract-baseline.md)（七热点现行路径/生产行分类/关键内部行锚、DAG/状态owner表、逐热点主题拆分目标与量化指标、API/行为 oracle、正反例、未知决策）、[度量基线](attachments/843/measurements-baseline.txt)；已关闭热点核销=无（全七热点经 PLAN-814 edb78b2ef 整体搬入仍存，git log --follow 连续）；组已建（new-wt-group.sh lang-843 + auto-ui detached 兄弟）；worktree `cargo check -p auto-lang-core` 预检探针绿 37.13s、145 预存 warnings 为基线 / next_responsible_role: work T-02（parser/类型推断/compile 边界整理，同权威树实施）。

2026-10-28 / stage: work（G843 启动门核验通过，T-01 开始） / plan_id: PLAN-843 / plan_revision: 1 / outcome: **executing 0/12，启动门核验 pass** / gate_evidence: G843=G842且U842。U831 r3 归档 12/12、U832 r3 归档 12/12（lang 交付 3934ed4f 为当前 master 祖先）、U841 r1 归档 12/12（lang 交付 f16bb468b 为祖先）、U842 r3 归档 17/17（2026-10-28 merge 闭环：lang 交付 031c8fd2d 为 master tip 祖先、ledger v105→v106、四树 fresh wt-guard clean、组目录回收，§9 明示"U842 全闭环，放行后继计划"）；git worktree list 无 831/832/841/842 残留树；主检出未提交改动仅 docs/ 簿记与 .autoos ledger 备份，无 crates/test 代码 WIP / worktree_path: 待建 D:/autostack/.wt/lang-843/auto-lang（T-01 内建组） / branch: codex/plan-843-dev / base_commit: lang master f99facb27b817fade1e5c2f03891c1ff418e28d2 / task_ids: T-01（进行中） / evidence: 初步定位七热点现行路径均在 crates/auto-lang-core/src/（trans/rust.rs 31704、vm/codegen.rs 16753、parser.rs 15204、vm/engine.rs 14148、vm/ffi/stdlib.rs 14076、vm/native.rs 13119、vm/ffi/http_server.rs 8320 raw 行），841/842 未消除任一热点，待 T-01 按 842 交付后路径正式重采并分类 / next_responsible_role: work T-01（读回三仓规范/AGENTS/前置交付映射，冻结合同基线并建组）。

2026-10-24 / stage: new / plan_id: PLAN-843 / revision: 1 / outcome: blocked（上游整Plan启动门未齐；文档合同预检通过）/ next: 等G843真实满足后读回规范/交付/归档版本，再work T-01 / changed: T-01..12、AC-01..07、SD-01..05、V-01..05。本Plan drafting 0/12，无实施/fixture/probe/测试已完成，未创建树或agent；[预检](attachments/m2-governance-batch-20261024/preflight.md)。

## 10. 待澄清事项

当前调度/范围已明确，不需重复确认；T-01负责§2机制、实际三仓路径/消费者与资源策略的有界冻结，门前不得提前probe。无法保持合同或需改变长期支持/源码语义/后端/远端动作则具体needs_replan；不静默缩面、豁免源身份或把本Plan必需集成延期给后继。全部实施等前置完整交付合入/归档收尾，不以status/单阶段pass越门。
