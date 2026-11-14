---
plan_id: PLAN-1021
status: archived
feature_name: ASE M0应用解析与无窗口AV加载实验
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-14
plan_revision: 1
current_step: 9
total_steps: 9
depends_on: [PLAN-1020, PLAN-960]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/m0-headless-loading.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1021] ASE M0应用解析与无窗口AV加载实验

## 0. 变更摘要

r1，archived，9/9（交付 af35aa318）（T-01..T-07 已完成，实现提交 b6a3e059c）；交付M0-LOAD/M0-H。消费1020就绪结论及960已交公开包链；无需等待AutoUI语法/980 Hello/AGE。这里是有界一次执行研究，不宣称独立持久AppSession或生产运行入口完成。路线：[ASE M0](../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

用稳定app身份解析两份独立普通Auto程序，经当前公开ACE包/verify/AV链真实返回11/22；改A为12不影响B，错误修复可重试。区分解析、编译、执行与资源失败。

不实现UI、后台服务、沙箱、安装器、通用daemon或磁盘零编译缓存；不复制ACE解析器/执行器。只允许明确注册的两个本地可信程序，URI不是执行任意文件的授权。

## 2. 架构方案

宿主元数据目录→严格URI/版本/能力校验→允许根内读取已登记源闭包→ACE现有compile_package_product→PackageAvTarget公开入口→真实结果/退出分类。输入manifest是实验描述，不新增公共作者语法。ASE只做发现/编排与错误投影，不负责语言名称/类型/代码生成。

## 3. 技术栈

新增`experimental/ase-m0/`独立Rust workspace（拟新增，不加入默认语言/ACE workspace），复用ACE公开crate与现有hash/URI能力；不加Core→ASE反依赖。T-01核依赖/锁解析，不创建junction或复制SDK。新CLI与测试为本包正式可重放源码，不是临时plan探针；临时副本/输出仍归plan-artifacts。

## 4. 需求分析与背景调查

用户2026-11-12明确授权制定ASE M0 strategy、第一批Plan及与ACE/AGE M2的实施次序，并确定Auto Scape Engine（ASE）为框架总名、AutoScape为浏览器产品。本轮授权为规划/文档，不含代码、fixture、worktree、构建、GUI/探针运行、提交/合入/标签或自动续跑；未指定模型、工期或token预算。既有其他会话在途/暂停决定保留。

取材[Specs overview](../specs/overview.md)、[语言SDK架构](../specs/language-sdk/architecture.md)、[UI仓界](../specs/auto-ui/project.md)和[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)。overview与部分UI旧测试别名为历史快照，现行AGENTS、固定组及UI本仓配置优先。当前ASE没有canonical现状规范，本包拟议增量仅在实际review/merge后沉淀，不将设计写成已支持。

读取身份：auto-lang HEAD `7b2f2e95967cb3434429093988fae429c05e865f`、auto-ui HEAD `ad9c05da65c9f48ec8a5ab0ef8815cff1ded0fb4`、auto-os HEAD `868c80d3e697ad6eaa9cce27596430a9abe08d5a`；工作区有其它会话文档WIP，内容可与HEAD不同，开工重核实际闭包与祖先。960模块Spec SHA256前16=`be25b0c69f2f8cc0`、公开packages AV实现=`fe9a2a1a36fdc7bf`；970草案=`70c5b6c98c680bac`、980草案=`fcc41243921c0fe7`。这些是只读来源标记，不是本包运行证据。

关键事实：[模块Spec](../specs/ace-migration/m2-modules.md)确认`ac.source.modules.packages/v1`，宿主供图，AV公开`PackageAvTarget::cold/cold_with_entry/run_value`；对应`experimental/ace/crates/av/tests/cases/module_packages_960.rs`已有真实consumer可参考。`source-query/src/modules/packages.rs`的compile_package_product和packages前门为已存在路径。当前冷链无warm body-edit接口，不承诺重新打开不编译。

## 5. 详细设计

### 支持与独立性

实验目录只支持`app://auto.demo.a/`与`app://auto.demo.b/`两个注册身份、根路由、已登记本地版本、无额外宿主能力。未知scheme/身份/route、userinfo/port/query/fragment、格式歧义或不支持版本/权限均在读源/dispatch前具名拒绝；不做全URI导航生态。路径来自可信目录，不从URI拼接；根包含检查拒绝越界/重解析链接，不将此有界根检查称完整沙箱。

A/B各为独立包输入、独立运行owner/结果。普通源码各返回11与22；换名/换注册顺序仍正确。A源改12，B仍22；坏A源拒绝后修复可再次执行。适用语言域以960当前支持为限。持有公开对象做重复一次执行，不把重新编译称warm cache/会话恢复，不访问私有VM字段。

新实验CLI拟`cargo run --manifest-path experimental/ase-m0/Cargo.toml -- --app app://auto.demo.a/ --artifact-root D:/autostack/plan-artifacts/plan-1021/run-a`；由T-02/04创建参数与入口，当前不可调用。一次退出输出结果/阶段/错误和实际消费源身份，URI/版本拒绝时dispatch计数为0（从真实adapter边界观测）。预算沿公开执行入口，资源拒绝不得改成语法错或成功。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ase/m0-headless-loading.md | 无ASE加载consumer→两身份严格解析/公开ACE一次执行/错误恢复及边界 | M0-H可重放且不冒持久会话 | AC-01..05 |

### 首次执行入口与来源门

唯一树`D:/autostack/.wt/lang-1021/auto-lang`，分支`codex/plan-1021-dev`。1020受审调查结论正式交付，960公开链来源与当前祖先核齐后T-01可进入。最多两种公开消费方式做一次有界先证，不能经旧VM或构造手写HIR绕过普通源；失败则needs_replan交回原owner。

本包允许孤立实验直接用已公开低层target；1022起的产品/窗口链走969/E0等正式门面，不把这里的低层路径升级为长期产品API。本包材料scope`D:/autostack/plan-artifacts/plan-1021/`开工即登记owner/用途；所有测试源随workspace跟踪，零依赖ignored材料。

## 6. 测试设计

测试与oracle以下均为拟议、未执行；新增测试/CLI入口在对应实施任务创建，不能在创建前报告PASS。实现与内部检查由执行者承担，最终独立review采用独立上下文、同一权威树；本轮未指定模型/人员，不创建agent/chat。

语言仓只按真实触面选择完整固定组及必要消费者，参考现存`ace-av`、`ace-source`、`ace-artifacts`，不为单一外部消费者机械跑`cargo t ace`全并集或`cargo t all`。若改ACE的源码/共享语义须补相应完整组与下游理由；纯ASE消费者不修改ACE时以新增ASE独立目标为主。UI只跑新增/受影响命名目标与公共接缝必要集成，按本仓入口，不沿用语言仓退役别名；原生/GPU/焦点单实例，自动注入与真实用户输入分账。工作迭代按反例/完整职责面，最终review后冻结提交验证受影响并集，身份未变证据可复用。

| Case ID | AC / invariant | 输入与真实入口 | oracle/覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 解析与授权 | 新CLI/Resolver，A/B及未知scheme/app/route/版本/能力 | 正控选对源；负控具名拒绝，读源/dispatch不发生 |
| V-02 | AC-02 根包含 | 注册根内合法源、`..`越界/重解析点及缺件 | 合法接受；越界/链接/缺件明确拒绝，宿主私有路径不被读取 |
| V-03 | AC-03 真实AV | A=11/B=22，经普通源码前门/verify/AV | 两独立预期值与真实dispatch来源；换名仍成立；stub返回常量不能过来源检查 |
| V-04 | AC-04 编辑恢复 | A→12、B→22；坏A源→修复 | 结果12/22；非法源编译阶段失败，修复后真实再执行成功 |
| V-05 | AC-05 资源与诚实分类 | 公开预算边界/受控失败及重复运行 | 原入口限制/关闭成立；实际target/文件owner释放可查，不承诺持久缓存或跨会话状态 |

拟新增完整目标`cargo test --manifest-path experimental/ase-m0/Cargo.toml --test headless`（T-06创建）。固定组拟`cargo t ase-m0`，T-06在`scripts/test-groups.toml`注册显式opt-in，不进入普通`all`或ACE并集；实际组名/runner在该任务按现有格式接入，保完整headless目标。若ACE源码零改，不机械跑ace-av整套；若改ACE owner须先有范围合同并选完整相关`cargo t ace-av`等职责组。

## 7. 验收标准

| ID | owner_stage | 验收与方法 |
|---|---|---|
| AC-01 | work | 严格支持域/错误与授权，正常/非法近邻V-01，拒绝前零dispatch |
| AC-02 | work | 根内读取与越界/链接/缺件拒绝，V-02；不声称完整沙箱 |
| AC-03 | work | 独立普通源真实返回11/22，无legacy/手写HIR，V-03 |
| AC-04 | work | A编辑12而B22，坏源修复后真实重放，V-04 |
| AC-05 | work | 新鲜树可重放、资源/错误分类成立，缓存与会话支持边界明确，V-05 |
| AC-R | review | 独审实际producer→AV→结果与拒绝敏感性、无旁路/遗漏 |
| AC-D | merge | 受审固定目标/SD-01正式交付、有效冻结证据、归档与输出全对账清理 |

## 8. 执行步骤

- [x] T-01｜work｜1020+960｜核源/公开API、建树/scope，有界普通源→AV先证并选消费接缝；AC-03/05。具名no-go则停后续实现。
  证据：1020已归档交付（archive/1020-*.md，commit 0f4a35f89），m0-readiness §2 记 1021=go；960公开链（compile_package_product / PackageAvTarget::cold/cold_with_entry/run_value）在 master HEAD `36a266267` 复核可编译可执行。树 `D:/autostack/.wt/lang-1021/auto-lang`（分支 codex/plan-1021-dev，base 36a266267），scope `D:/autostack/plan-artifacts/plan-1021/`（SCOPE.md 登记）。先证探针（scope 内，不入库）：两独立普通源经公开链 a=11/b=22，改 a 源→12 后 b 仍 22（4/4 PASS，exit 0）。接缝选定：`compile_package_product + PackageAvTarget::cold + run_value`。无 no-go。
- [x] T-02｜work｜T-01｜新增`experimental/ase-m0/{Cargo.toml,src,tests,fixtures}`，两独立普通源及轻描述；AC-01/03。依赖方向/新鲜树输入闭包检查。
  证据：独立 workspace（单包 ase-m0，空 [workspace] 表钉独立，不在语言/ACE workspace 成员清单）；依赖仅 4 个 ACE 公开 crate 的 path 引用（core/source-query/av/av-core，零 ACE 源改动）+ serde/toml/sha2；fixtures：两份独立 `main.at`（return 11/22）+ registry.toml（身份/版本/包名/根/入口/能力轻描述）。
- [x] T-03｜work｜T-02｜实现目录/URI/版本/能力和根包含，错误分态；AC-01、AC-02，V-01/02。
  证据：`src/uri.rs`（scheme/authority/根路由/userinfo/port/query/fragment 严格拒绝）、`src/registry.rs`（结构校验：身份/版本/包名/入口唯一与格式）、`src/source.rs`（根包含：`..`/绝对/多段拒绝、逐组件 reparse 拒绝、canonicalize 前缀复核、缺件/非常规文件拒绝）、`src/error.rs`（Phase×Kind 分态 + dispatch 计数）。单测 uri 2/2。
- [x] T-04｜work｜T-03｜接公开compile_package_product/PackageAvTarget与新CLI，不改ACE私有执行；AC-03/05，运行11/22及真实来源观察。
  证据：`src/run.rs` 经公开链一次冷执行，报告 input_hash/lowered/消费源 sha256；CLI 实测（worktree，artifact-root 落 plan-artifacts）：a→value=11 dispatch=1 sha=32ad4005f6554330 input_hash=cc7517cc71a5a7d6…；b→value=22 dispatch=1 sha=3fef427f75cf29eb input_hash=52fadaaf5cbe9be3…；`--version 9.9.9` → exit 1、dispatch=0（具名 authz 拒绝）。ACE 源零改动。
- [x] T-05｜work｜T-04｜补编辑12/22、坏源/修复、预算失败与清理；AC-04/05，V-04/05。
  证据：tests/headless.rs v04（11→编辑12、b 恒 22、坏源 `return }` → compile 分态 dispatch=1、修复→12 真实重试）、v05（max_parse_depth=1 公开预算 → resource 分态含 source.resource，非语法错非成功；两次冷运行 11/11 且 input_hash 相等；store 目录 run 后可删＝文件 owner 释放）。
- [x] T-06｜work｜T-05｜创建完整headless测试和opt-in固定组，运行V-01..05，零选择失败；AC-01..05。
  证据：`cargo test --manifest-path experimental/ase-m0/Cargo.toml`＝uri 单测 2/2 + headless 5/5（V-01..V-05，junction 负控 cfg(windows) 实跑）；固定组 `cargo t ase-m0` PASS（scripts/test-groups.toml +15 行：domain=ace、all=false、不入 sets.ace/daily → 不进 `cargo t all` 与 ACE 并集，显式 opt-in；`cargo t --list` 可见）。
- [x] T-07｜work｜T-06｜格式/告警/无debug检查，§9折摘要/拟议Spec与1022消费交接；AC-05。测试输入入库，原始证据不入库。
  证据：`cargo fmt --check` 干净；ase-m0 crate 零编译告警（ace-av/av-core 的 7 条上游告警为 master 既有，非本包引入）；库代码无 println/dbg（CLI 输出为产品面）；实现+测试+fixtures+锁文件入库 commit `b6a3e059c`（15 文件，2456 行）；原始证据（探针/target/运行输出）仅在 plan-artifacts scope，不入库。1022 交接：公开接缝与错误分态见本收据与 README；ASE 侧不新增语言/ACE 面。
- [x] T-08｜review｜T-07｜独立追踪普通源、实际AV dispatch、反例与关闭owner；AC-R，按结论修复或replan。
  证据：final pass @ b6a3e059c（收据 db9c02e9a）——接缝溯源（仅公开 import、无 vm/hir 旁路）、复审自选反例×3（未知身份 authz/0、畸形 URI uri/0、绝对路径源 source-escape/0）、冻结门禁复跑 PASS；AC-01..05 全 pass，无阻塞发现。
- [x] T-09｜merge｜独审pass｜冻结相关完整目标/必要组，沉淀SD-01、归档；对账全部输出目录，折摘要/删scope，guard清树；AC-D。

## 9. 复审记录

stage: new；PLAN-1021 r1；outcome: blocked（1020尚未正式交付，非设计拒绝）；next: 1020完成后核960/T-01。当前0/9，V-*与review/merge未执行。无需等待969、980或AGE整包。

本轮静态预检：章节/任务数、AC覆盖、相关活动依赖无环及本地链接通过；归档前置作为已交终点，不重新审计其历史依赖。仅规划检查，未运行测试或产品。

---

stage: work | plan_id: PLAN-1021 | plan_revision: r1 | worktree: D:/autostack/.wt/lang-1021/auto-lang | branch: codex/plan-1021-dev | base_commit: 36a266267 | code_commit: b6a3e059c | dependency_revisions: ACE 公开 crate 同仓 path 引用（worktree HEAD，零 ACE 源改动）；1020 就绪记录/960 m2-modules spec 复核通过 | task_ids: T-01..T-07（7/9） | outcome: pass

就绪记录（work-owned AC）：
- AC-01 ✅ V-01（11 组近邻负控全具名拒绝、dispatch=0；未注册版本/能力同）＋ uri 单测。
- AC-02 ✅ V-02（`..`/绝对/多段逃逸、junction 重解析点、缺件均具名拒绝；宿主私有路径不读；不称沙箱——README/代码注释明示）。
- AC-03 ✅ V-03（a=11/b=22 经普通源前门/verify/AV；input_hash/消费源 sha256 真实且随源变化；换根注册/换顺序解析仍正确，值跟随注册闭包）。
- AC-04 ✅ V-04（A→12 且 B 恒 22；坏源 compile 分态 dispatch=1；修复后真实重试 12）。
- AC-05 ✅ V-05（公开预算超限→resource 分态（source.resource），不改语法错/成功；重复一次执行冷运行一致；store 文件 owner run 后释放）；CLI 退出码/输出含 dispatch 观测（拒绝时 0）。
- 检查映射：V-01→AC-01、V-02→AC-02、V-03→AC-03、V-04→AC-04、V-05→AC-05；`cargo test --manifest-path experimental/ase-m0/Cargo.toml`（2+5 全过）与固定组 `cargo t ase-m0`（PASS）为执行证据。
- 遗留 in-contract 缺口：无。冻结受影响并集门（`cargo t ase-m0`；ACE 源零改动故无 ACE 组义务）与 SD-01 沉淀归 T-08/T-09 owner。
- 身份：实现提交 b6a3e059c（15 文件：ase-m0 包 14 + test-groups.toml +15 行）；探针/构建输出仅存 plan-artifacts/plan-1021（merge 收尾按裁定折叠+删除）。

next_responsible_role: review（T-08，独立上下文同权威树）

---

stage: goal | plan_id: PLAN-1021 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1021/auto-lang | commit: b6a3e059c | next: review

---

stage: review | review_scope: final | reviewer: 同会话独立复审轮（auto-plan-review，基于工件重建结论，非执行者摘要；未指定独立模型/人员） | plan_id: PLAN-1021 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1021/auto-lang | branch: codex/plan-1021-dev | reviewed_commit: b6a3e059c | base_commit: 36a266267 | dependency_revisions: ACE 公开 crate @ b6a3e059c 同树（diff 范围守卫：除 experimental/ase-m0/** 与 scripts/test-groups.toml 外零文件变更，ACE 源零改动；worktree 干净） | spec_inputs: docs/specs/ase/m0-readiness.md（1020 SD-01，1021=go 门）、docs/specs/ace-migration/m2-modules.md（960 公开链）

acceptance_results:
- AC-01 pass：V-01 绿（11 组 URI 负控＋未注册版本＋能力声明，全部具名 uri/authz 拒绝、dispatch=0）＋uri 单测 2/2；复审自选反例：`app://auto.demo.e/`→kind=authz dispatch=0、`app:/auto.demo.a/`→kind=uri dispatch=0（CLI exit 1 实测）。
- AC-02 pass：V-02 绿（root `..`/绝对 root/多段文件名/缺件/junction（cfg(windows) 实建实拒）均具名、dispatch=0）；复审自选反例：sources 含 `C:/secret.at` → kind=source-escape dispatch=0（Prefix 分支独立验证）；宿主私有路径不读（拒绝先于内容打开）。
- AC-03 pass：V-03 绿（a=11/b=22 经普通源前门/verify/AV；input_hash 与消费源 sha256 真实且随源变化；换根注册→a 身份返 22 且消费源摘要=b、input_hash 因包名身份而异——包名参与绑定哈希，符合 960 规范；重排序稳定）；接缝溯源：ase-m0 仅 import compile_package_product/PackageAvTarget/Value/FrontBudget/PackageModuleInput/PACKAGES_CAPABILITY/PackageDiagnostic，无 auto_lang/auto_vm/HIR/ProgramSnapshot 直连（grep 佐证）；CLI 实测 value=11/22 dispatch=1。
- AC-04 pass：V-04 绿（A 编辑 12、B 恒 22；坏源 kind=Compile phase=compile dispatch=1；修复后真实重试 12）。
- AC-05 pass：V-05 绿（max_parse_depth=1 公开预算 → kind=Resource 且消息含 source.resource、dispatch=1，未改语法错/成功；两次冷运行 11/11、input_hash 相等；run 后 store 目录可删=文件 owner 释放）；CLI 成功/拒绝退出码 0/1。
- AC-R pass：本收据（独立反例×3、冻结门禁复跑、接缝溯源、范围守卫）。

findings: 无阻塞发现。非阻塞观察：(1) dispatch 计数为单调用点构造性记账（非运行时插桩），对单次执行流行为准确（拒绝路径结构性不触 ACE 边界，测试断言 0/1 与路径一致）；(2) 上游 ace-av/ace-av-core 7 条编译告警为 master 既有（PLAN-1024 审计域），非本包引入。遗漏/延后/workaround 扫描：无（§5 全部支持域/边界断言落地；§10 待澄清已由实现解决）。

spec_delta review：SD-01（add docs/specs/ase/m0-headless-loading.md）before/after 规则与本实现核验一致（无 ASE 加载 consumer→两身份严格解析/公开 ACE 一次执行/错误恢复及边界；AC-01..05）；supersedes 为空正确（新组件）；frontmatter new_spec_components/touched_goals 与 §5 一致。canonical 文件由 merge 落盘。

pending_closeout_gates: 冻结 `cargo t ase-m0` 于落地基线（merge owner，身份未变可复用 b6a3e059c 证据：本次复跑 PASS，results.json 含 v01..v05 全数执行）；SD-01 沉淀；归档；wt-guard 清树；plan-artifacts scope 折叠+删除；输出目录对账。

evidence: 复审探针输出（三反例 stderr 具名错误）存 plan-artifacts/plan-1021/rev-*（merge 折叠摘要后删）；命令与关键数字已折入本收据。next: merge（T-09）



## 10. 待澄清事项

T-01负责确认960公开入口及构建闭包可用，最多两机制先证；T-06负责新固定组和完整目标接入。若研究需要新语言能力或持久缓存，不在本包暗扩，由原owner/后继计划处理。

  证据：见下方合并收据（af35aa318）。

---

stage: merge | plan_id: PLAN-1021 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1021/auto-lang（已清） | branch: codex/plan-1021-dev（已删） | reviewed_commit: b6a3e059c | delivery_commit: af35aa318 | rebase_mapping: b6a3e059c→662ee07b0（range-diff 全等）、4a22d5a0c→af35aa318（SD-01 docs-only 后继）

checkpoints:
- prepared ✅：reviewed 基线 b6a3e059c（final pass，db9c02e9a）；冻结 delta=SD-01 add docs/specs/ase/m0-headless-loading.md；worktree 内落盘并提交（docs-only，实现/依赖未变）。
- landed ✅：rebase onto master 后 `git merge --ff-only`，master tip=af35aa318=交付提交，线性历史无 merge commit，range-diff 证明 b6a3e059c 补丁全等。
- canonical_verified ✅：docs/specs/ase/m0-headless-loading.md 于 af35aa318 落 master，sha256 前16=b92dc30ee43c202c（4047B）；与复审核验的 SD-01 delta 一致；无 ledger/specs.json 写入。
- frozen_gate ✅：`cargo t ase-m0`（worktree，交付基线）PASS——v01..v05 全数执行（results.json 对账）；ACE 源零改动→无 ACE 组义务；语言固定日常组不受本包影响（独立 workspace，不入 daily/all）。
- archived ✅：git mv 至 docs/plans/archive/1021-ase-m0-resolver-and-headless-av.md，status: archived，9/9 任务勾选核对一致；archive/ 前缀链接 ../.. 相对解析正常（spec/roadmap 链接位于归档目录下按既有归档惯例保持原文）。
- batch_regress_due ✅ 非到期：.last-batch-regression.json last_covered=1043 @2026-11-13T19:30:42Z（<48h）；1021%5=1 且低于水位 → 不触发；归 auto-plan-regress 到期例程。

输出目录对账（T-09）：本计划全部输出=D:/autostack/plan-artifacts/plan-1021/（SCOPE.md、probe 探针 workspace+target、run-a/run-b/run-x/rev-* CLI 运行与复审反例、build-err.log）；无其他盘符/散落目录/D:/autostack 根级 planNNN-* 残留；构建中间产物（worktree 内 ase-m0/target、experimental/ace/target、target/test-groups）随 worktree 删除，不入 plan-artifacts。证据已折叠：命令/退出码/关键数字（11/22/12、input_hash、sha256、dispatch 0/1、三反例具名错误）在本文件 §8/§9；保留件无（无具名未解决问题→零持有）。

生产工件落点检查（landing≠deployment）：本交付为独立 opt-in 实验 workspace（非语言/ACE workspace 成员，不在 daily/all 组、不进默认构建面），无下游产品二进制/服务/web bundle 消费——无 stale 工件需重建重启；主检出语言默认构建面不受影响（唯一仓库面变化=scripts/test-groups.toml +15 行组注册）。

- cleaned ✅：wt-guard clean → worktree D:/autostack/.wt/lang-1021/auto-lang 移除、分支 codex/plan-1021-dev 删除、组目录 .wt/lang-1021 删除（worktree 计数 0）；plan-artifacts/plan-1021 整 scope 删除（SCOPE.md/probe ~2.2GB/run-a/run-b/rev-absroot；证据已折叠、零具名持有）；.retention/policy.json 登记 deleted@merge；输出对账见上段（无外部盘符/散落残留）；构建中间产物随 worktree 终结未回收。
next_responsible_role: 无（终态；1022 消费交接见 T-07 收据）

---

stage: goal | plan_id: PLAN-1021 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: af35aa318 | next: done
