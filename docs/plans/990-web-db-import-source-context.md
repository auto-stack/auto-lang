---
plan_id: PLAN-990
status: execution_done
feature_name: Web独立Rust后端的db相邻导入上下文
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
current_step: 2
total_steps: 4
base_branch: master
base_commit: bf72baabf28fa865b5a7c680b8a5fa4018a0a63d
depends_on: [PLAN-956]
supersedes_spec_components: [auto-web/docs/specs/http-contracts.md]
new_spec_components: []
touched_goals: []
affects: [auto-web/api-generation, auto-os/PLAN-054]
---

# PLAN-990：独立后端db的相邻导入上下文

## 0. 变更摘要

父auto-os/PLAN-054新SDK176正式公共Vue入口编译Blog独立Rust后端失败4项：导入记录的String槽位收到&str，循环中拥有字符串被移走。实际日志plan-054/t07/vue-final-r187b.err.log：db.rs:271 E0308、37/54/70 E0382。生成器传相邻目录给companion和装配证明，却对db调用无目录的legacy函数；956收紧不安全的跨函数字符串猜测后暴露此遗漏。修复生产入口上下文，不修改Blog源码或生成产物，不恢复跨函数猜测。

## 1. 目标

相邻api.at导入的记录字段按已有语言String规则发射，实际生成编译运行；保留956代码区域/函数作用域安全与其它HTTP行为。父054完整Vue/iced范围不变。

## 2. 架构与实施归属

沿Web中央计划登记先例890/956，正文在语言主检出；唯一代码改面auto-web。复用父054组内现存干净真实Web worktree D:/autostack/.wt/os-054/auto-web，提升为本Plan唯一authoritative writer，分支codex/plan-990-web，基于Web当前master bf72baa（已有956路径捕获/锁修复）。不新建重复实现树，不改主检出产品代码、不建链接。组内Langf032/UIa1db/Man2a9为已选依赖；语言949 writer保持冻结。本Plan落地后恢复Web消费身份；父054收尾统一guard清理组。

## 3. 背景与Spec依据

已读Web AGENTS.md、README、docs/specs/http-contracts.md、testing.md；Lang docs/specs/auto-lang/trans/design/a2r-string-ownership.md。现有String字段规则成立；register_import_signatures有source_dir时解析相邻TypeDecl。HTTP生成api_gen.rs db入口约516行遗漏上下文，companion约487/643行、装配证明约1700行已有。原消费37d32c→当前bf72baa新增956路径捕获与锁文件，须核实际差异，不声称全树等同。先验证根因；其它根因需有界修订而非改应用绕过。

## 4. 设计

使用已有transpile_back_module_to_rs_with_proofs("db",content,Some(root/src/back))替换无目录入口，保留post_process_db_rs和写出/handler接线。纯Rust、现有依赖，语言/HTTP运行时协议不变。永久测试覆盖导入记录中的借用参数、循环重用owned字符串、字段名不同于参数、局部同名数值及字符串/注释对照；真实生成代码编译运行。实际Blog全后端编译与作者任务由父054最终共同SDK消费关闭。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD01 | modify | auto-web/docs/specs/http-contracts.md | 独立生成db及companion均以自身src/back解析相邻导入，字段所有权遵循语言声明；证明与实际生成目录一致 | 产物不得丢掉已证明上下文 | AC01/02 |

## 5. 验收与验证合同

| AC/case | owner_stage | 输入与观察 | 关闭条件 |
|---|---|---|---|
| AC01/V01 | work | 公开Web生成最小跨模块db/记录→实际Rust编译运行 | 原错失败，新行为正确；borrowed/owned循环复用/异名字段及数值邻面齐全，业务串注释不改 |
| AC02/V02 | work | 既有956代码区域/作用域、真实路由捕获控制与Web生成受影响选择 | 实际执行，原条件不改；不重跑无关Core/VM全组 |
| AC03/V03 | merge | 父054共同SDK真实Blog独立后端及作者任务 | 真实编译/保存/重开/下一次操作、源/生成/发布/运行身份一致 |
| AC04/V04 | review/merge | 独立复审，准确Spec、线性交付、push/归档与父组清理 | 所有pending明确，不把源码阶段当最终交付 |

## 6. 步骤

- [x] T00 owner_stage=work：冻结根因/选择面，永久反例真实红。
- [x] T01 owner_stage=work：入口修复、生成运行与必要邻面，提交工作完成。
- [ ] T02 owner_stage=review：独立源码/Spec/证据复审，不委派实现。
- [ ] T03 owner_stage=merge：父最终SDK消费、受影响冻结门、Spec/线性落地/push/archive与统一清理。

## 7. 记录

- stage:new | revision:r1 | outcome:pass→work | authorization:用户要求解决必要阻塞并完成054，独立复审已授权；无新增功能或收窄验收。取号脚本在主检出从现存990→991；.next-id原有989→990未提交预约及989未跟踪计划属于他人，保留其前进与文件，不把外来预约一起暂存。本次仅提交990正文；共享计数当前991保持，后续对账。

## 8. 待澄清

无新增用户语义决定。尚未实施/复审；父本次失败不计pass，旧SDK任务只在原身份有效。


- stage:work | revision:r1 | outcome:execution_done2/4 | code_commit:c36e5d09b15ee9c8dce0d34fb4b6f9f1b37e26e1 | next:independent source/Spec review
  Actual public generate_api regression uses src/back/api.at imported Entry/Number, db borrowed seed assigned to differently named String field, immutable owned token reused in a loop, numeric value parameter, exact business text resembling code. Full generated backend cargo build and real TCP GET/query/JSON assert four records and numeric7; authenticated loopback shutdown actualsuccess/portrebind. Final immutable test1/1pass/8.35s (compilepreparation2.77s). Working fix only replaces db no-source transpiler with existing with_proofs+Some(backdir); returnedproofs discard mirrors prior finalgeneration behavior, assemblyproof collection unchanged and uses samecontext. No Core/stringguessing/app/API/schema change. README backlink and permanent integration test onlyotherchanges.
  Valid pre-fix immutable regression: actual generated db E0308 seed &str→String, testexit101/4.78s; no unrelated compile errors in this valid red. Original actual Blog full-backend baseline additionally has moved token E0382 at37/54/70 plus renamed borrowed field E0308 at271; final real Blog work remains parent-owned rather than claimed closed by minimalfixture. Loop/numeric/literal controls in permanenttest remain independent expected values even where oldversion already passed.
  Earlier harness attempts are retained honestly: first used obsolete back_member_name (could not locate generated manifest); next used unsupported API List return and parameterless route-B declarations, then invalid [Entry] spelling. These are fixture failures, not productpasses/findings. Corrected public contract uses []Entry and declared seed/value query parameters plus actual member_dir_name(MemberRole::Back). Original valid red/green commands use unchanged correctedfixture. db-import-green.log was an intermediate fixture-failure despite filename; only db-import-immutable-green.log is final green.
  V02 actual affected Web neighbors selected/executed4/4pass, 177 outside selection excluded notpass: original956 route-capture exactness/order, body db function resolution, Chat/Notes dbhandler controls. No skipped test counted as executed. The live generated numeric7 and exact code-looking string also protect the956 per-function/code-region boundary without modifying/repeating unchanged CoreVM duties. Worksourcefrozenbeforeindependentreview; no source publication/Spec canonical/archive/parent acceptance claimed.
  Raw root D:/autostack/plan-artifacts/plan-990 registeredinretention as active. Valid red SHAa37001618e1fb6cf3bc4c878d18db7b3c9ee10fca1d63e5b5812d0c048043a90; final green SHAace77555ec974f29148904d123172f245896cca4028371031c54b5498239d95c; Webneighbors SHA4a0d48c773bc5040ff191b7a99dd39814b69ccccf7459bb18f75eeaf3b86f478. Scope Webbf72→c36e only; former SDK176 Web37d→bf72 is existing956 path-capture+lock integration and separately validated neighbor boundary. Parent final SDK must bind all eight sibling Git roots, not onlyLang/UI/Man/OS, then real generated/frontend/runtime identity/task gates.
