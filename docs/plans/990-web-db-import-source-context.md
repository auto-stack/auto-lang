---
plan_id: PLAN-990
status: execution_done
feature_name: Web独立Rust后端的db相邻导入上下文
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 3
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
| AC04/V04 | review/merge | 独立复审，准确Spec、线性本地交付、归档与父组清理 | 所有pending明确，不把源码阶段当最终交付 |

## 6. 步骤

- [x] T00 owner_stage=work：冻结根因/选择面，永久反例真实红。
- [x] T01 owner_stage=work：入口修复、生成运行与必要邻面，提交工作完成。
- [x] T02 owner_stage=review：独立源码/Spec/证据复审，不委派实现。
- [ ] T03 owner_stage=merge：父最终SDK消费、受影响冻结门、Spec/线性本地落地/archive与统一清理。

## 7. 记录

- stage:new | revision:r1 | outcome:pass→work | authorization:用户要求解决必要阻塞并完成054，独立复审已授权；无新增功能或收窄验收。取号脚本在主检出从现存990→991；.next-id原有989→990未提交预约及989未跟踪计划属于他人，保留其前进与文件，不把外来预约一起暂存。本次仅提交990正文；共享计数当前991保持，后续对账。

## 8. 待澄清

无新增用户语义决定。源码与Spec增量已独立final复审通过，reviewed3/4；T03父共同SDK实际Blog、冻结消费门与交付仍pending，旧SDK任务只在原身份有效。


- stage:work | revision:r1 | outcome:execution_done2/4 | code_commit:c36e5d09b15ee9c8dce0d34fb4b6f9f1b37e26e1 | next:independent source/Spec review
  Actual public generate_api regression uses src/back/api.at imported Entry/Number, db borrowed seed assigned to differently named String field, immutable owned token reused in a loop, numeric value parameter, exact business text resembling code. Full generated backend cargo build and real TCP GET/query/JSON assert four records and numeric7; development loopback shutdown202 + actualsuccess + portrebind. Final immutable test1/1pass/8.35s (compilepreparation2.70s). Working fix only replaces db no-source transpiler with existing with_proofs+Some(backdir); returnedproofs discard mirrors prior finalgeneration behavior, assemblyproof collection unchanged and uses samecontext. No Core/stringguessing/app/API/schema change. README backlink and permanent integration test onlyotherchanges.
  Valid pre-fix immutable regression: actual generated db E0308 seed &str→String, testexit101/4.49s; no unrelated compile errors in this valid red. Original actual Blog full-backend baseline additionally has moved token E0382 at37/54/70 plus renamed borrowed field E0308 at271; final real Blog work remains parent-owned rather than claimed closed by minimalfixture. Loop/numeric/literal controls in permanenttest remain independent expected values even where oldversion already passed.
  Earlier harness attempts are retained honestly: first used obsolete back_member_name (could not locate generated manifest); next used unsupported API List return and parameterless route-B declarations, then invalid [Entry] spelling. These are fixture failures, not productpasses/findings. Corrected public contract uses []Entry and declared seed/value query parameters plus actual member_dir_name(MemberRole::Back). Original valid red/green commands use unchanged correctedfixture. db-import-green.log was an intermediate fixture-failure despite filename; only db-import-immutable-green.log is final green.
  V02 actual affected Web neighbors selected/executed4/4pass, 177 outside selection excluded notpass: original956 route-capture exactness/order, body db function resolution, Chat/Notes dbhandler controls. No skipped test counted as executed. The live generated numeric7 and exact code-looking string also protect the956 per-function/code-region boundary without modifying/repeating unchanged CoreVM duties. Worksourcefrozenbeforeindependentreview; no source publication/Spec canonical/archive/parent acceptance claimed.
  Raw root D:/autostack/plan-artifacts/plan-990 registeredinretention as active. Valid red SHAa37001618e1fb6cf3bc4c878d18db7b3c9ee10fca1d63e5b5812d0c048043a90; final green SHAace77555ec974f29148904d123172f245896cca4028371031c54b5498239d95c; Webneighbors SHA4a0d48c773bc5040ff191b7a99dd39814b69ccccf7459bb18f75eeaf3b86f478. Scope Webbf72→c36e only; former SDK176 Web37d→bf72 is existing956 path-capture+lock integration and separately validated neighbor boundary. Parent final SDK must bind all eight sibling Git roots, not onlyLang/UI/Man/OS, then real generated/frontend/runtime identity/task gates.


## 9. 独立复审记录

- stage: review | review_scope: final | reviewer_identity: independent delegated `/root/review_lang949`, separate from 990 implementation; no fabricated model/session ID | plan_id: PLAN-990 | plan_revision: r1 | outcome: pass (source/work-owned criteria and proposed Spec delta) | next: T03 merge owner frozen SDK/actual Blog integration, then authorized delivery
  - Baseline: main authoritative Plan read at Lang7e966522daa81ac4986fd14e4d86d8d7afb94068; unique registered Web writer `D:/autostack/.wt/os-054/auto-web`, branch `codex/plan-990-web`, clean entry/verdict HEAD `c36e5d09b15ee9c8dce0d34fb4b6f9f1b37e26e1`, Web diffbase `bf72baabf28fa865b5a7c680b8a5fa4018a0a63d`. Lang dependency `f03226e1063e191af61dd119bc066ec60bf848ca` clean; current selected UIa1db/Man2a9 remain parent-owned common build inputs. Actual worktree/branch association independently checked; no new tree, source edit, Cargo invocation, UI action or remote publication by reviewer. Shared C2 remains reserved for parentSDK190.
  - Full three-file diff inspected. `transpile_db_to_rs(content)` was exactly `with_proofs("db",content,None).map(code)`. New production call supplies `Some(root/src/back)` to the identical shared carrier, same db module name,16MiB worker, Embedded target and reference-proof behavior. It still runs existing post_process_db_rs, repository borrow fixup, persisted db.rs and extracted db function handler wiring. Proof generation already supplied this directory; companion loops supply it too. Discarding returned proofs in final write is unchanged, not removal of assembly validation. Legacy public no-context helper stays available for its historical callers. No Core dependency direction, app/endpoint/schema/protocol or global guessing change.
  - Import invariant independently traced through `register_import_signatures`: source_dir siblings are preferred before legacy CWD candidates; only explicitly selected imported TypeDecl members populate the field-type cache. Thus the new source context permits actual declared `Entry.value: str` to use String storage/borrow logic instead of cross-function textual guesses. It does not teach all same-name fields to be strings. Independent representative counterchecks selected in code/assertions: differently named borrowed seed→value, numeric Number.value/value int→7 in separate function, code-looking business literal unchanged, owned token repeated in two loop iterations. These are compiled/runtime assertions, not mere generated text counting. Scope does not claim all arbitrary unresolved imports repaired.
  - AC01/V01→T00/T01 **pass**: permanent `tests/db_import_context.rs` source SHAb989af4fc1f52474b0071758c25486b66deddeff714d5b439ef5e23917a15f41 inspected end-to-end: public generate_api creates real Rust workspace, obtains actual Back member via member_dir_name, runs real cargo build, spawns compiled binary with dynamic loopback config, asserts TCP GET200/query/UTF8 and exact four records/numeric7. Parses full JSON and exact literal, tests owned-string loop reuse and borrowed→owned field. Success shutdown oracle is development loopback POST202, actual child terminal success and port rebind; **no authentication token/authorization coverage claim**. OwnedChild kill+wait only protects failed paths. Existing finite connect/read/shutdown deadlines retained; no unhandled failure transformed to pass.
  - Original raw immutable red/green independently read and rehashed: red SHAa37001618e1fb6cf3bc4c878d18db7b3c9ee10fca1d63e5b5812d0c048043a90 actual generated db.rs:10 E0308 (`seed` &str where String expected),1failed/0passed/exit101/run4.49s; green SHAace77555ec974f29148904d123172f245896cca4028371031c54b5498239d95c,1selected/executed/pass/0fail/0ignore/0filtered/run8.35s, compile2.70s. The same immutable let-token fixture and command were used, with production call None versus Some changed between runs. Executor confirmed exact command/env; earlier invalid fixtures and prior var-token variant kept separately, not counted. Prior4.78/2.77 timing transcription corrected above from final logs, not a semantic contract change. Actual Blog baseline raw4E0308/E0382 also read; minimal immutable regression deliberately proves E0308, and does not falsely claim its baseline loop already failed E0382.
  - Commands (executor argv reconciled with raw outputs; reviewer did not rerun while C2 reserved): cwd writer, `cargo test --locked -p auto-web --test db_import_context -- --nocapture`, same command for immutable red/green. Common env roots Lang/UI/Man/Web point to corresponding `D:/autostack/.wt/os-054/<repo>`; CARGO_TARGET_DIR=`D:/autostack/.wt/os-054/auto-lang/target/generated-consumers/c2e02416`, CARGO_BUILD_JOBS=2, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, SCCACHE_DISABLE=1, PLAN990_SCRATCH=`D:/autostack/plan-artifacts/plan-990`. Child's actual build argv from committed test is `cargo build --manifest-path <generated Back Cargo.toml> --target-dir <same target> -j 2`; generated service binary path derives from actual package name. Selected runtime assertions and raw terminal output justify reuse on frozen code; no new execution claimed by reviewer.
  - AC02/V02→T01 **pass**: actual `cargo nextest run --locked -p auto-web --lib --ignore-default-filter -E 'test(handler_calls_db) | test(test_db_fn_resolved_from_body_over_heuristic) | test(path_capture_names_are_exact_and_extract_in_route_order)' --test-threads 1`; raw web-neighbors.log SHA4a0d48c773bc5040ff191b7a99dd39814b69ccccf7459bb18f75eeaf3b86f478:4selected/executed/pass,177outside selection (not pass). Checked assertions/production branches: 956 exact path capture names/order and query remainder, db function body resolution, Chat and Notes handler routing. Immutable runtime numeric/literal controls protect code-region/function safety alongside these neighbors. Old Web37d32c→bf72 path-capture+lock changes inspected separately as prior956 integration, not disguised as unchanged c36 patch. No unrelated Core987/whole-Web repeat; existing compiler warnings unchanged, diff whitespace check0.
  - AC03/V03→T03 **pending merge-owned**, not a source pass: newly bound common SDK must actually compile/run full Blog backend and author save/reopen/next operation on final inputs, including all8sibling roots and chosen manifests/app pins. Minimal generated fixture does not close the real Blog four-error/currentSDK/author-task leg. AC04/V04→T02/T03 **review portion pass / delivery pending**: independent final/source-Spec receipt complete; linear landing/push/Spec publication/archive/parent worktree cleanup not performed or credited. Plan reviewed3/4 with T03 unchecked; final source pass cannot authorize landing with unmet integration gates.
  - Spec knowledge delta: owning repository Web, exact repo-relative target `docs/specs/http-contracts.md` (central cross-repo metadata `auto-web/docs/specs/http-contracts.md`), existing source SHA6287ff08e08dacaa9137941f9110d6f6c8d85601a1546f6ee52680ceba6289f7. SD01 is an additive clarification of current generated DB source-context consistency under AC01/02, not a new protocol or altered language ownership rule. No Spec candidate source file was prepared by work; the exact reviewed publication text below is frozen in this Plan with SHA256 9178b7532f29d51b7b073bde1e11e98b396627e75204bb025724a88310a312cb. Existing source remains unmodified until merge. `new_spec_components: []` because this modifies existing HTTP contract; `touched_goals: []` because no existing stable external goal ID was identified and §1 has no goal IDs—do not invent IDs or update derived JSON/index. No legacy-ledger writes.

Reviewed SD01 publication text (Web docs/specs/http-contracts.md):

```markdown
### Generated Rust DB sibling imports (PLAN-990)

For standard src/back projects, standalone Rust generation supplies db.at and companion modules with the project src/back source directory used by assembly proof collection. Sibling import declarations provide record field types to the existing language String storage and borrow rules. The db module keeps its existing post-processing and handler wiring; source-region and per-function ownership controls, legacy standalone transpiler entry points, import fallback rules and HTTP protocols remain unchanged.
```


- stage:merge | checkpoint:prepared | revision:r1 | reviewed_source:c36e5d09b15ee9c8dce0d34fb4b6f9f1b37e26e1 | next:frozen affected Web gate then linear landing
  AC03 nowclosed by actual commonSDK195 publicVueentry: all8siblingroots bound, production4185modules/14.53s/exit0; BlogfullRustbackend compiled/current source33df, actualnative6 v15/18artifactbindings/67inputs; nativeBlogfullUnicode create/preview/return/Save/normalclose0/freshprocessreopen/nextSave exactbody/current/revision8. FreshVue195Blogwholeauthor task andtrueheaderDOMdetach/reopen/nextSave exact. Parent bothprofile8required8pass0fail; packets iced t06/iced-run-r195.json and Vue t06/vue-run-r195.json (description-only metadata corrected; no cases/frames/data changes), parentround181. Minimal generatedDB test no longer substitutes for realBlog integration.
  Canonical http-contracts.md appends exactly the independent frozen SD01 publication text above, no additionalrequirement. Current actualcode/dependencies/config unchanged from c36; noCore/runtime/HTTPprotocol modification. Source review pass retained; merge-owned scopedWebconsumer+4neighbor gate follows, nounrelatedVM987orcatalogrepeat. Cleanup remains deferred to parent054 group lifetime because its rawacceptance/finalreview binds paths; no claim T03 fullycomplete/push/archive/clean yet.


### r2 用户裁定交付方式（2026-11-07）

用户明确选择“沿用现有本地交付，不新建远端”。本仓无remote，现有Web delivery.md/README明确local independent delivery且未授权remote/registry；只读git ls-remote对候选auto-stack/auto-web返回Repository not found，不创建/推送不存在的远端。r1通用“push”文字与既有仓交付形态不一致，r2仅按本次明确裁定改为线性本地主干交付/归档，维持原修复目标、四AC行为阈值、仓集合、源码、永久测试与SD01不变；独立当前revision文档/Spec复核后继续收尾。历史r1源码/Spec审核与所有真实结果保留，不伪称曾远端发布。

- stage:merge | revision:r2 | checkpoints:landed/canonical_verified | local_delivery_commit:2d69841 | next:independent r2 doc/identity reconciliation then archive/parent group cleanup
  Webmaster实际bf72→2d69841 --ff-only成功，mainclean；sourcec36→delivery2d69841仅http-contracts.md批准SD01，代码/永久tests/Cargo完全相同。canonical SHA256 c2c1ac37306611f4127f690e6d8720c091a879ab2b92e8aced6800e76d8177d3. 正式merge-owned frozenWeb gate在当前2d69841执行：db_import_context1selected1pass/10.96s/exit0（构建1.31s），真实generatedRust/TCP/json/shutdown202/portrebind；4neighbors4selected4pass/.131s/exit0，177outside excluded notpass。两日志位于plan-990/merge-db-import.log与merge-web-neighbors.log，源码/依赖与review一致。Parent实际SDK195Vue/native8/8及Blog完整作者任务见preparedreceipt；不用最小fixture代替完整应用。
  本地交付不是默认部署：本Plan代码在parentimmutableSDK195已正式构建/运行；主检出release/WebCLI产物未为文档归档另外重建，后续生产使用须按currentSDK标准入口重生，不能把旧artifact当新版本。原Webworktree暂留为activeparent054唯一consumer/原始证据路径，父finalreview后统一guard清理；T03仍pending，仅已完成本地land/canonical和冻结门，不提前标cleaned。
