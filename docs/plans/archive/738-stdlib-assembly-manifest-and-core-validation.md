---
plan_id: PLAN-738
completion_kind: delivered
status: archived
feature_name: stdlib-assembly-manifest-and-core-validation
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-10
plan_revision: 3
current_step: 7  # R10 pass：T-12/13/14 R9 修复重闭；T-01/T-09..T-14 完成；T-02..T-08 由 Phase 3 承接（R8 口径恢复）
total_steps: 14
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/auto-lang/frontend/design/module-resolution.md
  - docs/specs/auto-lang/trans/overview.md
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
  - docs/specs/auto-man/design/api-generation-integrity.md
  - docs/specs/auto-cli/project.md
new_spec_components:
  - docs/specs/stdlib/design/assembly-manifest.md
touched_goals: [GOAL-003]
affects: [crates/auto-lang/src/compile.rs, crates/auto-lang/src/autovm_persistent.rs, crates/auto-lang/src/module_cache.rs, crates/auto-lang/src/lib.rs, crates/auto-lang/src/parser.rs, crates/auto-lang/src/stdlib_assembly, crates/auto-lang/src/trans/rust.rs, crates/auto-lang/src/trans/c.rs, crates/auto-lang/src/a2r_std.rs, crates/a2r-std/src/http.rs, crates/auto-lang/src/vm/codegen.rs, crates/auto-lang/src/vm/native_registry.rs, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/vm/native_catalog.rs, crates/auto-lang/src/vm/ffi/stdlib.rs, crates/auto-man/src/api_gen.rs, crates/auto-man/src/rust_ui.rs, crates/auto/src/main.rs, crates/auto/src/cmd_stdlib.rs, stdlib]
---

# [PLAN-738] 标准库后台装配契约与 manifest：真实来源、核心符号校验和缓存一致性

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) D3a。736正在实现HTTP部署基线；本计划回到最初标准库诉求：公共.at与目标实现如何缝合、谁承担宿主实现、不同环境是否可用，须由实际装配过程证明。

首期交付三层：
1. **全库清点**：stdlib/auto公共模块、目标文件、公开符号与已知provider来源可机器读取；无法解析/未覆盖逐项记录，不能默默遗漏。
2. **真实装配manifest**：明确target/environment，公共源在前、选定目标层随后；VM/Rust/C实际入口采用共同装配计划，并记录实际使用的native/runtime/Auto body。没有http.rs.at不等于Rust HTTP不存在。
3. **核心门禁**：io/net/async/http/json/sse的已声明符号按真实provider/签名/能力分类，引用缺实现、重复实现、签名漂移、目标或环境不适用均有构建/装载诊断；不会空返回或落另一个后台。

这是D3基础和核心门禁，不承诺全标准库所有符号跨后台语义相同，不批量实现当前缺失的Rust net/io、C HTTP或浏览器监听。非核心模块先清点和记录验证等级，进一步签名/ABI和语义parity作为D3b。全actor/CPU/内置TLS/通用WS不在本期。

**当前执行入口：Revision 3 / Phase 3「最终契约修复与验收收口」**，见 §5.8、§6.4、§8 的 T-09..T-14。继续原 `plan-738-dev` worktree，以 `2c1b4a763` 为入场实现基线；保留 R2 服务验收和三目标 witness 收据，补 strict 适配证明、引用闭包与 manifest 对拍、SD 终稿及最终独立复审。AC-01..08、SD-01..07 的范围和通过阈值不变；仍不合入/归档。

### work 修复交接（2026-10-10，R5 三项必修闭合 → execution_done）

- stage: work
- plan_id: PLAN-738
- plan_revision: 3
- outcome: pass（R5-01/02/03 修复完成并回归；next=R6 独立复审）
- code_commit: worktree plan-738-dev `9c255993b` → `bc0b95cfd`（R5 三项主体）→ `796d279a4`（真值表对齐）→ `68398d2d6`（receiver 形态守卫+切片修正，tt 非基线红归零）；clean
- task_ids: T-11..T-14 重开项全部重闭（R5 口径 current_step=7：T-01/T-09..T-14；T-02..T-08 保持打开待 R6 确认）
- evidence: R5-01=lock 一次绑定状态机（R5 探针落为正式测试 review738_r5_lock_binding_state_machine：absent→首物化 fresh+收据绑定→0.2.0 漂移/删除/缺字段均拒绝；纯函数真值表同步严格化）；R5-02=receiver 公共方法入门（public_method_symbol 分母归属+别名表、三发射形态收敛、json 值 def-use 跟踪、方法 receiver/is_static 契约、形态守卫）+CLI 级反例闭合（漂移 exit1+DRIFT+JsonValue.len proof，原 exit0）；R5-03=真实消费者三角（VM/Rust/C 三目标 + 真实 generate_api 生成腿，可比面相等+Embedded/Standalone producer 合理差异钉死，零 clone）；门禁：plan738 72/72、plan724 5/5、CLI 10/10、api_gen 44/44（含生成腿）、freshness 双真值表+状态机、tt 非基线红=0（R5-02 曾致 6 语料红→守卫+切片修正归零）、服务完整链 1/1@64.7s
- blockers: 无。教训：①「absent 匹配任意」型豁免必须带结束条件（一次绑定/收敛写回），否则是永久放行；②发射臂拆段写（`len((&` 分段）使 call 缓冲出现不完整片段——按文本验证的闭包必须带形态守卫；③跨 crate 测试归属（auto-lang 测试不能反向依赖 auto-man）要在设计时定腿。
- next: `/auto-plan:review PLAN-738`（R6）——独立上下文复核 `68398d2d6` 与三项 findings 闭合；仍不合入/不归档/不删 worktree

### work 修复交接（2026-10-10，R6 必修闭合 → execution_done）

- stage: work
- plan_id: PLAN-738
- plan_revision: 3
- outcome: pass（R6-01/02/03 闭合；next=R7 独立复审）
- code_commit: worktree plan-738-dev `68398d2d6` → `6c3943327`（R6-01 泛化+落穿+case④+Value 分派+Vec 面；R6-02 bind 保守化）；clean
- task_ids: R6 重开的 T-11/T-13/T-14 重闭（R5/R6 口径 current_step=7）；R6-03 plan498 flake 入册
- evidence: `receiver_method_drift_matrix_both_spellings` 1/1（keys/as_int 双拼写基线零违规+proof/漂移 exit1；is_null 双拼写基线即拒=真漂移诚实面）；CLI 级矩阵实测同构；plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44、双真值表+lock 状态机、tt 非基线红=0、tv 162/162、服务链 1/1@53.5s
- blockers: 无。教训：receiver 闭包修复不能只钉单个方法（len）——公共方法分母是 12 个，修复面必须按分母泛化并用**双拼写矩阵**验收；恒空的历史跟踪集（json_value_vars）是分派类 bug 的温床，新 def-use 集要接入所有消费点。
- next: `/auto-plan:review PLAN-738`（R7）——独立上下文复核 `6c3943327` 与 R6-01/02/03 闭合；仍不合入/不归档/不删 worktree

**2026-10-10 R7 独立复审：needs_fix。** R6-01 矩阵面（keys/as_int/is_null 双拼写）/R6-02/R6-03 闭合确证，门禁全复现；但全分母普查发现 `json.has_key(v,"k")` 漂移后仍 exit0 无 proof（P738-R7-01 必修，拆段写+守卫跳过）与 case④ 误伤 `json` 同名局部变量/本地模块（P738-R7-02 必修回归）。重开 T-11/T-13/T-14，详见 [738-review-r7.md](reports/738-review-r7.md)。原 R3/R4 pass 面未被扰动（修复链只触 4 文件）；修复属于现有 revision 3 合同，不增 revision、不降 AC。
### work 修复交接（2026-10-10，R7 必修闭合 → execution_done）

- stage: work
- plan_id: PLAN-738
- plan_revision: 3
- outcome: pass（R7-01/02/03(裁量)/04(登记) 闭合；next=R8 独立复审）
- code_commit: worktree plan-738-dev `6c3943327` → `3713337d9`（3 files +265/−53）；clean
- task_ids: R7 重开的 T-11/T-13/T-14 重闭（R5/R6/R7 口径 current_step=7）
- evidence: `receiver_method_drift_matrix_both_spellings` 全分母 12 方法×双拼写（绿面基线零违规+proof/漂移 exit 拒；真漂移面 is_null/as_bool 基线即拒；parser 拒绝面；无臂 case④）；plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44、tv 162/162、tt 非基线红=0、服务链 1/1@109.5s
- blockers: 无。教训：①发射臂的多闭括号类 bug 会以「守卫跳过」形态在闭包层复活——拆段/丢参臂必须逐臂核对配平；②receiver 带参方法在 Auto parser 层的拼写支持要与矩阵预期区分（语言层拒绝≠闭包放行）；③适配语义（bool→int 壳、Null 哨兵、宽度归一）是**分母级**的——逐方法打补丁不如先全分母矩阵再按类建契约。
- next: `/auto-plan:review PLAN-738`（R8）——独立上下文复核 `3713337d9` 与 R7-01/02 闭合（含全分母矩阵与抽查）；仍不合入/不归档/不删 worktree

**2026-10-10 R8 独立复审：pass → reviewed。** R7-01 四级证据闭合（diff/矩阵 24 格/CLI 基线 exit0+JsonValue.has_key proof+漂移 exit1 DRIFT/产物 `if a2r_std::json::has_key(&v, "k") { 1 } else { 0 }` 配平）；R7-02 双面闭合（shadow 变量与本地 json 模块 exit0）；R7-03 裁量/R7-04 登记落地；全部门禁独立复现（tt 14 红全基线、非基线红=0，plan730 隔离绿）；SD-04/audit #11 与行为一致（approved）。两项 P3 观察登记（链式面收紧+过时注释、has_key_str 规则被 claim 门遮蔽）。详见 [738-review-r8.md](reports/738-review-r8.md)。

### work 修复交接（2026-10-10，merge 实现冲突闭合 → 待 R9 复审重启 merge）

- stage: work | plan_id: PLAN-738 | plan_revision: 3 | outcome: pass（merge needs_fix 的实现冲突已定位并修复）
- code_commit: worktree plan-738-dev rebase 重放链 → 修复提交 `19973b089`（含 adfb7a5d1 SD 沉淀+rebase 合并面+主链修复）；clean
- 根因（文件级二分+插桩实证）：rebase 冲突解决时 master 侧 `execute_autovm` 主链的「new()+register_std_shims+register_stdlib_ffi+merge_native_interface」re-merge 块回归——738 R2 合同已将 `AutoVM::new` 改为 `NativeInterface::production()`（三件套超集）并删除该冗余块；无契约的 `register_stdlib_ffi` 重注册按 merge 撤销语义覆盖 production() 声明的核心契约（插桩实证：AutoVM::new 后 http.get #2230 契约在 → re-merge 后无 → verify 假红）
- 修复：删除主链冗余 re-merge 块（保留 pkg 入口的既有块与 738 合同一致）；`.rs` 级插桩全部移除
- 门禁（19973b089 实跑）：cb_web_mime PASS；tv 162/162、plan738 73/73、plan724 5/5、CLI stdlib 10/10、api_gen 44/44、三 crate check 零错、rustfmt 干净、tt 非基线红=0（`plan748_breakpoint_probes::p748_t00` 主检出同红实证=master 预存非本树引入）、服务完整链 1/1@117s
- next: R9 独立复审（定向：修复 diff 机制核证+门禁复现+SD 面未扰动）→ pass 即重启 merge（landed 及其后各步）

**2026-10-10 R9（I+II 两轮独立复审）：needs_fix。** 主链修复机制确证有效（merge 撤销语义代码级闭环、两树合同面逐字节一致）、定向门禁全绿（cb_web_mime PASS/tv 162/plan738 73/服务链 1/1）；但 P738-R9-01（lock 读取非 NotFound 错误归一 absent，未绑定收据新鲜度门误放行——**reviewed 树既有缺陷非本修复引入**，probe+代码双证）与 P738-R9-02（4 点格式）必修；merge 重启 deferred 至修复新提交再复审 pass。详见 §9 R9-I（[738-review-r9.md](reports/738-review-r9.md)）/R9-II。

### work 修复交接（2026-10-10，R9 必修闭合 → 待 R10 复审重启 merge）

- stage: work | plan_id: PLAN-738 | plan_revision: 3 | outcome: pass（R9-01/02 闭合）
- code_commit: worktree plan-738-dev `19973b089` → `41b4d4be5`（5 files +74/−31）；clean
- R9-01：gate `classify_lock_read`（NotFound=Some(空)=absent 哨兵；权限/IO 错误=None 不可核验→保守陈旧——recorded="absent"×读错误不再假新鲜，纯函数单测 classify_lock_read_fail_closed 含端到端断言 lock_freshness(absent, None)=false）；生成侧读错误（非 NotFound）拒写收据（Err fail-closed）
- R9-02：lib.rs/renderer.rs/engine.rs 四格式点 rustfmt 收敛（三 crate check 零错）
- 门禁（41b4d4be5）：classify 1/1、双真值表、api_gen 44/44、lock 状态机 1/1（shell_pack=R3 在案预存）；cb_web_mime PASS、plan738 73/73 抽验；R9 已证合同面两树逐字节一致——VM/契约/服务链门禁不重复全跑
- next: R10 独立复审（定向：R9-01/02 闭合+修复树面）→ pass 重启 merge

**2026-10-10 R10 独立复审：pass → reviewed，可重启 merge。** R9-01 双端闭合确证（gate：NotFound→absent 哨兵/其它错误→None 落入 `lock_freshness` `_ => false` 保守陈旧，反例 recorded="absent"×PermissionDenied 现判 false 有显式断言；生成侧非 NotFound 读错误 return Err 拒写收据）；R9-02 token 级核证（lib/engine 去空白 token 流逐字节一致、renderer 唯一差异=尾逗号移除 AST 等价）+ rustfmt 五文件机械干净；门禁全绿（plan738 73/73、tv 162/162 含 cb_web_mime PASS、api_gen 44+1 ignored、lock 状态机 1/1、三 crate check 零 error、**正式生成服务链 1/1@42.29s——R9-I 明确要求的重跑项，work 交接省略由本轮补跑**）；SD 沉淀与合同面（stdlib_assembly/native*/stdlib/**）在 19973b089..41b4d4be5 逐字节未扰动。三项 P4 观察登记（负例矩阵 bound×None 无显式断言、空 lock 文件归 absent 哨兵、服务链省略流程记录）。详见 [738-review-r10.md](reports/738-review-r10.md)。

### cleaned 回执（2026-10-10）

- wt-guard clean（lang-738/auto-lang 与 auto-down 与历史 verify-738-base/auto-lang 三处）→ worktree `lang-738/auto-lang` remove ✓、分支 `plan-738-dev` delete（9ff34ae0d 全并入 master 实证）✓、组目录 `lang-738/` 移除（auto-down 为未注册普通副本 27M，无 reparse point，一并清理）✓、陈旧 verify-738-base 组移除 ✓、worktree prune 后注册表零 738 残留 ✓。

### merge 最终收据（2026-10-10，PLAN-738:r3 全检查点闭合）

- prepared ✓：`641f66a5f`（SD-01..07 沉淀，3713337d9 文档后代）
- landed ✓：rebase（master 前进两次均干净重放，diff 纯 docs 核证）→ 修复合并链 `19973b089`（主链冗余 re-merge 回归修复，R9 确证）→ `41b4d4be5`（R9 必修）→ `9ff34ae0d`（终 rebase）= delivery commit；主检出 `git merge --ff-only` 线性落地，master tip==delivery==plan-738-dev；他人 UI WIP（6 文件）以 stash@{0}「738-merge-handoff」+ `/d/autostack/.738-wip-handoff.patch`（569 行）交还，未卷入
- ledger_refreshed ✓：designs P738-1/2 + tests P738-3 + reviews P738-4 + reports P738-5 upsert 回读核验（commit 2af1d4576，748/749 先例）；spec-index.py INDEX.md 刷新
- archived ✓：git mv docs/plans/archive/ + status: archived + completion_kind: delivered
- 批量回归：到期（last_covered=740，>48h 且 740 后有合并）→ 交 `/auto-plan:regress`（主检出单实例）；**点名尾巴**：R9-I（[738-review-r9.md](../reports/738-review-r9.md)）的 HTTP 串行全档仅完成 45/102（2 fail 在册基线+环境族、57 项未跑，该报告明确不记全档通过）——th 逐名分诊随批量回归轮一并收口，不外推为全档通过
- 产物观察：auto-lang 为库仓（无运行服务/daemon/web bundle）；消费者二进制（auto.exe 等）由各使用方重建——本计划代码已入 master，无陈旧生产进程需重启

### merge 收据（2026-10-10，PLAN-738:r3 → needs_fix/实现冲突）

- **2026-10-10 R10 处置更新：修复复审 pass，可重启 merge**——R9-01/02 已在修复合并树 `41b4d4be5` 闭合（R10 pass，[738-review-r10.md](reports/738-review-r10.md)）；rebase 重放基面须更新为该提交所在链（19973b089→41b4d4be5），landed 及其后各步骤自此重启。
- stage: merge | plan_id: PLAN-738 | plan_revision: 3 | outcome: needs_fix（落地受阻：rebase 实现冲突）
- reviewed_commit: 3713337d9（R8 pass 不变；代码未改动）
- prepared ✓: SD 沉淀提交 `641f66a5f`（3713337d9 的文档后代，worktree 已回滚保留在该 tip）
- landed ✗: rebase master（30 提交重放，range-diff 等价）后**实现冲突红**：`cb_web_mime`（VM 面 `STDASSEMBLY.SIGNATURE_UNVERIFIED: http.get #2230 契约缺失`）——reviewed tip 上同测绿（1.1s 复证）。根因初诊=master 并行演进（PLAN-746 协作式执行/747/748/749）与 738 VM 契约验证面的组合性冲突（非单侧语义丢失：lib.rs/engine.rs 两树组装段 token 级等价核对完成，R1/R2 对两文件零语义改动、R2b 已含）。冲突文件：tests.rs/main.rs（已双保留解）+ lib.rs/aura_view_builder/autodown_editor widget/iced renderer/stretch_line/engine.rs（--ours 决策已废弃，worktree 已 reset 回 reviewed 链）
- ledger_refreshed/archived/cleaned: 未执行（前置 landed 未达）
- 批量回归到期：last_covered_plan_id=740（2026-10-08T11:20Z，>48h 且 740 后有合并）→ 落地后交 /auto-plan:regress
- 处置：worktree 回滚至 `641f66a5f`（reviewed 证据链完整保留）；计划回 executing；next=work 在 worktree 解决 rebase 组合冲突（VM 契约面 × master 746/749 演进）→ 重跑受影响门禁（tv 全档+cb_web_mime+服务链）→ 新提交独立复核落地面 → 重启 merge

**2026-10-10 R5 补充复审：needs_fix。** 当前修复基线为既有 worktree `9c255993b`，保留 T-09/T-10 与所有历史代码/正例，不回滚到 R2。必修项=首次 lock 物化豁免永久放行后续漂移、公开 JsonValue receiver 方法未进入 strict 门、消费者三角验收使用 clone 替代实际生成器且缺 VM/C 对拍。重开 T-11..T-14，详见 [738-review-r5.md](reports/738-review-r5.md)。原 R4 pass 保留历史，本轮反例使整体通过结论失效；修复属于现有 revision 3 合同，不增 revision、不降 AC。

## 1. 目标

- G1：把声明、Auto目标层、VM native、Rust宿主映射和UI适配明确分开，记录模块/符号/实际目标的唯一执行来源与证据等级。
- G2：生产入口使用同一装配计划，不能依赖未使用的parser后缀helper；VM常规/persistent、Rust/C转译及生成API依赖分别接线。
- G3：核心公开符号检查声明+实现签名、合法ext补全与一符号一实现；native名字/ID已登记不能代替实际shim可调用。
- G4：执行目标vm/rust/c、运行环境native/browser与HTTP/IPC/merged拓扑分别记录；前端Vue不自动让后端成为browser。
- G5：公共源/目标层/provider和编译模式共同决定指纹/缓存，新源或目标变化不能用旧符号/旧生成产物；源码错误定位到真实层文件。
- G6：提供JSON诊断/检查CLI与全库覆盖报告，失败可定位；用真VM和Rust/C实编、假同名/缺实现/缓存变更等反例证明。
- G7：保持734/736 API与HTTP协议合同及普通use隔离，不把本期当更换调度/统一服务传输或通用依赖解析重构。

## 2. 架构方案

```text
resolved public module + explicit execution target/environment
        ↓ shared AssemblyPlan (ordered sources + explicit host provider)
        ↓ source segments / normalized declarations / symbol identities
   ├─ VM codegen + bound NativeInterface proof
   ├─ Rust emission + explicit runtime/Auto provider lowering
   └─ C emission + C layer / external declaration requirement
        ↓ per compilation/session AssemblyManifest + diagnostics
        ↓ cache dependency fingerprint / CLI inspect / 734 generation receipt
```

拟新增：
- `crates/auto-lang/src/stdlib_assembly/{mod,model,loader,validate,providers}.rs`：纯装配模型、源段映射、验证/诊断、版本化provider描述；不依赖Axum/Tokio/UI。
- `stdlib/assembly-providers.json`：外部宿主提供者及明确Unsupported/能力条件的声明目录。由运行生产者/发射分派校验，不作为第二份“声称已实现”的独立表。
- `crates/auto/src/cmd_stdlib.rs`：拟`auto stdlib inspect [--module auto.http] --target vm|rust|c --environment native|browser --format json [--check]`。
- `crates/auto-lang/src/tests/plan738_stdlib_assembly_tests.rs`、`examples/stdlib/assembly/`、`docs/plans/reports/738-stdlib-*.{md,json}`：新测试、同源样例及报告。

保留正常ModuleResolver搜索/pac声明门控/use命名空间语义；shared loader从已解析的public路径组装同目录目标层，不重新猜全项目路径。host-mapped Rust标准库和Auto层实现是可选择的provider形态，显式选择后只有一个活实现；存在未消费.rs.at只表示candidate，不标为实际source。

全库清点与“某次编译实际加载了什么”分别输出，二者可关联，不能用目录扫描代替真实manifest。本期默认strict门只覆盖六核心模块的依赖/引用闭包；全库之外的行为维持，但未知必须在report标明，不称已验证。不会把全部stdlib重新生成或强行自动拼接每个现存.rs.at。

## 3. 技术栈

现有Auto parser/AST/TypeStore、ModuleResolver、CompileSession/ModuleCache、NativeInterface与Rust/C emitter；serde/serde_json与项目已有内容指纹能力。解析使用实际语言parser，不用regex搜fn/contains类型来做契约。VM语义与native逻辑ABI分开，Rust映射复用实际调用分派；ABI字节槽不能从函数参数数目直接推断。

JSON schema仅服务装配清单/工具输出，不改语言语法或公开File/Response/Task ABI。原始stdlib API暂有目标属性/宿主声明混合，首期允许显式分类迁移，不要求一轮搬空公共.at。

## 4. 需求分析与背景调查

### 4.1 来源、版本与授权

起草master基线 `8d33eeb95`，736 executing:r1 1/8，T-01报告引用 `a043bb4fc`，worktree入口 `de3a64353`。734已review R2 pass并归档，delivery=`ec0eae41f`，不再沿用其历史needs_fix阻塞。738取号提交 `f1a1671c8`，独占锁重扫max737后运行new-plan.sh；其他计划不修改。

| 来源 | 本期使用的事实/要求 |
|---|---|
| `docs/specs/overview.md`、`stdlib/project.md` | 多目标标准库、公共/目标层现状，不保证所有后台同构。 |
| `docs/specs/stdlib/design/backend-assembly.md` | 696建立覆盖表，VM与persistent装载分叉，Rust宿主/生成服务/UI适配独立；尚无实际manifest门。部分HTTP表格仍保留早期措辞，须与729/730/734增量对照。 |
| `docs/specs/auto-lang/frontend/design/module-resolution.md` | 路径解析与装载/缓存职责、545 bare/wildcard/named规则、635声明门控不变。 |
| `docs/specs/auto-lang/trans/overview.md`、`runtime/design/networking-stdlib.md` | Rust stdlib映射及HTTP lowering，不得只因缺rs文件拒绝已有host能力；目标草案不等于已用装载链。 |
| `docs/design/raw/stdlib-organization.md` | ext物理补全历史设计；它不是当前实现/可移植ABI的证明，需T-01核对后提出具体规范。 |
| `docs/specs/stdlib/design/api-transport-contract.md`、`auto-man/design/api-generation-integrity.md` | 734共同API分类/真实实现/新鲜度，拓扑能力与stdlib实现能力不是同一个维度。 |
| [PLAN-736](archive/736-http-service-deployment-and-operations.md) T-01、§5/§8 | service配置/ready/生成与HTTP回收正在实现；本期不并行覆盖其lib/auto-man/CLI入口。 |

授权：“736正在实施中，请继续规划下一个计划”；允许只读调查、本仓计划与Design33簿记，不实施/跑Cargo/改其他计划状态/调用外部服务。**执行前置为736独立review pass并合入**；即使部分模型文件可独立写，生产接线须消费736最终来源/生成/ready合同，统一顺序实施。用户无需现在提供更多输入。

### 4.2 代码观察（静态，T-01复现）

| 文件/符号 | 观察与验证方向 |
|---|---|
| `compile.rs::load_module_inner` | context_ext硬编码.vm.at，公共文本+换行+目标文本；再resolve/parse/bytecode，目标层错误可能落公共文件名。装配应保留源段映射。 |
| `autovm_persistent.rs::load_and_register_module` | 另建stdlib目录/root/context路径；auto/前缀处理root与context不同，需真实入口确认，不能仅静态断言全persistent都坏。 |
| `parser.rs::get_file_extensions` | dead_code helper列.at/.vm.at/.rs.at/.c.at；不能用它证明实际Rust/C已选择正确层。 |
| `lib.rs::trans_c_with_session/trans_rust_with_session` | 复用CompileSession后再设parser dest/emit；loader本身仍VM选层，需防类型预解析与实际发射target不一致，不在Rust里编VM shim。 |
| `trans/rust.rs::use_stmt`与call分派 | auto.*及裸stdlib名映a2r_std；provider存在与签名要按实际导出/调用分派核对，模块名称在表中不等于真实模块存在。 |
| `module_cache.rs::ModuleCache`、`compile.rs`早退 | cache只记单file_path/hash，compile传合并源；另有compiled_modules/path跳过。需复现错误cache miss、目标层变动失效和跨target串台，不能只改单个hash字段。 |
| `vm/native_registry.rs`、`native.rs`、`native_catalog.rs`、`ffi/stdlib.rs::register_stdlib_ffi` | 名称/ID/返回类别、真实shim数组/手工注册分散。catalog明确不含所有stdlib手工shim；绑定缺失、ID别名/冲突与逻辑签名须区分。 |
| `stdlib/auto/io.at/io.vm.at/io.c.at` | 公共声明已有#[vm]，VM在ext填方法、C填物理字段；不是纯声明+自动相同ABI。 |
| `stdlib/auto/json.rs.at`与Rust host | rs文件存在，但Rust映射可能消费host runtime；实际选择须记清，不把两个provider重复激活。 |
| `ui/ext_stubs.rs`、`lib.rs`use.web adapter链 | 前端适配是独立机制，不能凭空假定io.vue.at自动拼接；先记录事实/能力，不重写Vue链。 |

### 4.3 Revision 3 入场事实与授权（2026-10-09）

- 用户指令：“请你更新计划738,把后续步骤做成新的phase，然后我们去执行agent继续完善。”授权本轮在主检出修订计划和交接；后续执行沿用此前“执行修订和修复”的本仓授权。本轮不实施、不跑 Cargo、不代执行 agent 启动工作，也不授权合入/归档。
- 实现基线：`D:/autostack/.wt/lang-738/auto-lang`，`plan-738-dev@2c1b4a763280b3994f51b441714b4f2120328fb3`，本轮观察 clean；auto-down 兄弟保持 `895f8d0f9355c9f5ec3ce8fca268bdb768395846` 只读。主检出已有其他 UI 工作的 WIP，本轮只编辑本计划簿记，执行不得搬到主检出。
- R2 执行报告：[738-repair-round2.md](reports/738-repair-round2.md)。最终树真实生成服务验收 1/1、50.18s（用户报告 50.2s）；同源 VM=41/Rust=42/C=43 和 C stdio=65、分级门收据保留为该提交的执行证据。本轮只读核查，不将执行报告或本轮调查记作独立复审通过，也不将这些收据外推到后续提交。
- 源码明确差异：`stdlib_assembly/host.rs::verify_rust_reference` 对包装形状免除 async 不匹配拒绝，并在元数不匹配时返回 `Resolved`；`trans/rust.rs::call` 收到该证明后仍输出代码。该差异须按原 §5.7 的 strict 要求修复，不能用降级证明维持放行。
- 待验证假设：Rust `call()` 单站点是否覆盖所有六核心调用；VM 最终链接证明是否完整进入 manifest；CLI actual、会话/产物、生成 receipt/ready 是否描述相同实际装配。报告中的“329 发射点均属胶水/面外”须逐类验证，尚不作为覆盖证明。
- 数据/文档差异：`manifest.rs` 的 consumer 字段参与指纹；多消费者对拍需定义共同身份和消费者证据的关系。生成 workspace 的实际 lock/features 是否已被捕获尚需验证。`738-sd-drafts.md` SD-06 将退出码 2/3 写反，代码为 2=错误、3=inventory partial。
- 范围事实：`2c1b4a763` 共 566 个文件、108488 additions/48935 deletions；“约 530 文件纯 rustfmt”的抽查声明不能替代完整差异分类。入场复查与 Specs/草稿 SHA256 记录于新增 [Phase 3 基线收据](reports/738-phase3-baseline.json)。Specs 仍为当前 canonical；738 尚未沉淀，不因草稿或实现与其不同而覆盖它们。

## 5. 详细设计

### 5.1 两类清单与验证等级

Inventory扫描`stdlib/auto/**`，公共模块/目录入口、目标层与全部公开fn/method/type/公开字段有stable ID、源位置、解析结果与provider candidate；非公开shim helper也保留映射供验证但不扩张公共API。每个无法解析的文件有diagnostic，不从总分母删除；公共模块+目标文件数量分别统计，T-01冻结实际数据，不沿用本轮99顶层.at计数当公开模块数。

AssemblyManifest v1只记录该次请求/编译的真实选择：
- compiler/provider schema版本、execution target、environment、consumer/transport、features、stdlib来源身份。
- 按顺序的public与target source段、文件内容hash、依赖module及fingerprint，符号decl/impl origin、normalized signature、provider kind/locator、能力与明确拒绝原因。
- selected/unsupported/missing/conflict/unverified等状态；验证等级至少declared、resolved、bound、signature_checked、executed。文件存在/名称登记最多到resolved/bound，不能升为executed。
- 稳定JSON排序；便携清单用stdlib/工程相对source ID，不把机器绝对路径或运行文件根塞进公开收据。需要找文件的local诊断可带实际源路径，与portable fingerprint分开。

每个编译session独立immutable snapshot；两个同名模块不同root/target/session不能覆盖全局“当前manifest”。inspect不执行main、native网络或FS业务操作；缓存/解析读取有界，只写指定报告位置。

### 5.2 目标层选择与provider

AssemblyContext明确执行/发射目标Vm|Rust|C以及Native|Browser环境。渲染Vue不改变后端Rust/VM的Native环境；HTTP/IPC/merged的734能力矩阵另行组合。环境未知不能自动当native或加载其他目标文件。

顺序：resolved public.at在前，然后**选定**的同目录目标层；只有明确Auto-layer provider才加载相应.vm/.rs/.c.at。公共Auto body可作为shared实现；host-mapped/provider模式显式声明由哪个runtime/adapter承担，以及哪个目标层只是未消费candidate。无目标层但已有native/host实现是合法；根本没有实现时调用失败，不能猜另一个suffix补上。

目录入口、别名、re-export、裸/具名/wildcard、循环依赖保持既有语义。重复导入同一source不重复定义；同名异root按resolver身份区分；目标不在模块名后临时拼一个auto/前缀。禁止根据环境目录存在自动落到其他target。

producer签名与调用方声明分别读取/核对：VM实际注册携带独立逻辑签名/alias/feature信息，HostMapped Rust provider由其实际导出/emit map支持，C外来接口用已选C声明/类型。若provider表称支持但callee不存在，strict失败；不以从公共AST复制同一签名到两侧充当校验。

首期provider目录重点六模块；catalog生成/对照到实际注册或发射点，禁止手写一个catalog再让运行继续用不同表。允许既有内联调用lowering保留，但发射时上报实际provider与signature，T-01定位所有核心分派入口并绑定同一identity。async模块与Rust task actor词汇不强行视为同一个调度provider；能验证映射才标supported。

### 5.3 符号缝合、native与能力门

声明可无body，有合法目标body、native绑定或宿主provider提供实现。公共声明+匹配目标实现不算重复；两个活body/两个互不等价provider、类型/参数/返回/async/generic约束漂移则冲突，诊断给双方位置。方法签名归一化显式/隐式self、static与返回类型身份，不靠文本contains。同一声明允许的re-export与native别名明确登记，别名不是额外执行实现。

ext允许目标内部必要字段/实现补全，但公开字段/方法合同不能静默改变；物理layout与opaque资源属于target，manifest记录layout身份，不宣称VM/C/Rust共享物理ABI或跨目标资源互传。混合#[vm]公共声明保持为可追踪legacy事实，不能把所有公共.at属性搬迁造成无关破坏。

native验证两步：name→ID resolution与该NativeInterface里真实shim绑定同时成立。ID复用只限明确别名同一callee；两不同callee争ID报错而不last writer wins。逻辑签名描述不是栈ABI证明；槽宽/资源生命周期更深ABI在D3b，但核心变参/self/async差异不能以Unknown假称signature_checked。声明本来允许动态值/泛型/变参时，以明确Any/Generic/Variadic身份及约束保留，不能误当未解析类型；兼容lowering的参数适配变体须单独记录。缺独立签名的既有provider标unverified，调用若属于本期strict核心须补生产者元数据/真实测试，不能豁免到“已支持”，也不能把原已工作的能力仅因缺元数据降为Unsupported以过门。

缺实现按引用/导入项/生成依赖闭包校验，未使用的unsupported声明不使整个http模块无法导入。正常use namespace的可见性不放宽，用户同名http/json/File不当stdlib。Browser下监听/本地FS/native sockets无适配须Unsupported，不假返回0/空串；native-backed源不能被Browser简单链接进去。确有浏览器fetch/console adapter时另报consumer/environment和已验证来源，不通过Vue renderer名字假造能力。

核心公开符号全部有状态和原因；现存占位实现标DeclaredStub/Unsupported，例如未交付Server.static，不因已绑定shim而标实用能力。报表unsupported是诚实覆盖结果，不要求此计划新造net.rs.at/io.rs.at。

### 5.4 实际入口、源位置与失败传播

VM CompileSession与persistent loader共同使用AssemblyPlan；source segment保留original source ID/range，解析/冲突错误能反映目标文件位置；保持Database dirty/type_store/use语义。Rust/C入口先确定真实target，再进行类型装载/发射；若存在VM专用bytecode预处理阶段，其模块绑定不得被当成目标实现，也不得给Rust/C sidecar做VM CALL_NAT编译。

T-01用最小synthetic public+三目标fn验证真实consumer；若现有incremental管线需要超出本期的整体改造才能选层，应needs_replan并保留AC，不做绕过实际consumer的“新helper绿”。共用计划不是必须共用全部编译器/TypeStore或把foreign Rust源码交给Auto parser。

生成API的Auto核心调用依赖manifest进入734 generation receipt：业务指纹与assembly fingerprint各自记录，源/provider/features改变时新鲜度门识别；736启动/ready引用最终版本。失败传播至auto-man/build/serve，不吞错启动旧generation；Config profile不改变source assembly指纹，target/environment/features等会改变。

单纯本地Rust原生直接调用a2r-std没有Auto装配阶段，不强行声称有Auto manifest；其导出验证作为Host provider证据登记。

### 5.5 缓存一致性

cache key包含resolved模块身份、target/environment、provider schema/features和public/选定层/host版本/依赖闭包的内容指纹；记录missing sidecar的存在性，新增/删除层也会失效。只改mtime不能伪造新/旧内容；公共+目标合并hash不能拿公共文件单独重算冒称valid。

修复CompileSession早退与ModuleCache的一致性，编译epoch中新源必须更新真实模块/类型/bytecode和manifest。同一active VM不得热换stdlib ABI；执行后改target或stdlib root须明确SessionTargetMismatch/重建，不能让旧堆资源继续跑新impl。跨target cache可以共享纯内容存储，不能共享同一活manifest/impl选择。

源码/provider变动后的新编译失败，保留旧进程/缓存资产只作为旧版本，当前build非零且不启动它；不修改全局AutoCache SQLite设计或重做所有增量缓存。本期指纹不是密码学信任/签名系统，校验旨在正确性与新鲜度。

### 5.6 inspect与核心覆盖报告

inspect输出inventory或某次AssemblyPlan/Manifest（模式字段明确），check输出machine-readable diagnostic code、module/symbol/target/env、双源位置、缺provider/feature原因及改用支持目标/实现的指引。dry inspect的selected provider须来自实际resolver/provider策略，执行/生成另有消费证据；不伪造executed。

--check语义：核心被请求的closure不得有missing/conflict/unverified；明确unsupported被调用也非零，未引用的unsupported是inventory项。全库inventory存在解析失败给完整JSON且有非零/partial状态，不能统计漏项为pass。非核心未验证项明确列出，不将全库report称全库strict已完成。

最终报告包括六模块所有公开符号×vm/rust/c×native/browser（实际入口支持/不支持都有原因），VM/Rust已验证HTTP/file/upload例与C已有支持样例；浏览器仅测试能力拒绝与已有adapter事实，不把host server放到webview中。D3b后续逐模块扩展严格门和行为parity，不能从本期signature_checked推定同取消/错误/ABI语义。

### 5.7 Revision 2：最终绑定证明与实际引用闭包（2026-10-09）

用户授权“OK，按照你的说的执行修订和修复”。沿用 PLAN-738、原 worktree、AC-01..08 和 SD-01..07；不新取号，不削减验收，不把必要修复转成延期。R1 已验证基础代码保留；历史完成声明不能替代本 revision 的证据。

1. **共同生产 factory + 最终 snapshot（T-03/04）**：统一 AutoVM 与 inspect 的 native 初始化，严格保留原注册及 override 次序。最终选中 callee 的独立 producer contract 随绑定存储；后续 register/merge 替换必须撤销旧证明。签名证据包含 receiver/static/generic/mode/async、输入输出及错误适配；未证明的被引用核心符号为 Unverified 并拒绝，不能改标 Unsupported 绕过门。
2. **来源与引用门（T-02/03/04）**：由实际 resolver 身份和调用 lowering 收集符号闭包（含 Auto body 内依赖），六核心公共声明与目标实现/宿主契约双源比较。模块导入本身不拒绝未引用的 unsupported 声明。VM 普通/persistent 与 Rust/C 实际 emit 消费共同 manifest；本地同名模块不冒充 stdlib。TCP read 公开 API 保留，已发现的缓冲/双输出不等价如实诊断，不能擅改 ABI。
3. **不可变 manifest 与失效（T-02/05/06）**：完成 actual selected sources/dependencies/providers/features/target/environment/symbol proof 快照，便携身份与本地诊断路径分离；指纹按实际闭包计算，缺失/读取失败闭合拒绝。clone 的 TypeStore 不能跨根串台；生成和启动消费同一版本快照，unknown/null 指纹不能相互证明新鲜。
4. **完整清点与诊断（T-02/06/07）**：完整 public fn/method/type/field 分母及六格 target/environment 状态；失败层与未知项保留。诊断指向公共与实现源/producer，而非从已污染 live TypeStore 反推。
5. **真实验收和因果分诊（T-07/08）**：已有支持能力的 C 和 Rust 同源代码必须实编实跑；沿 734/736 真生成→编译→serve→ready 链检查 assembly 改变后的拒绝/再生。HTTP timed-frame 原断言保留，在相同配置下对照 R1 基线与修复树，必要时跟踪上游到客户端各阶段；未确定原因前不得记为既有环境红。最后重跑分级门、完成 SD 正文及独立复审，仍不授权合入/归档。

任务依赖沿用 T-01→T-02→T-03/04→T-05→T-06→T-07→T-08；每个已完成单位提交代码，主检出只记计划进度。新 API 或确需独立范围变更须另提具体方案；当前修订保持原目标与授权。

### 5.8 Revision 3 / Phase 3：最终契约修复与验收收口

本 phase 把 R2 遗留验收和本轮发现的 strict 缺口明确任务化。沿用现有目标/六核心边界/公共 API 与服务合同，不增加新后台，不把必修项延期到 D3b。原 T-02..T-08 保持打开，新 T-09..T-14 提供执行顺序和对账出口。

1. **严格验证真实适配链**：区分公共声明、选中 producer 和 lowering adapter。适配壳必须有可独立检查的契约，覆盖元数、类型、receiver、返回/错误形状、sync/async/await 及实际 callee；公共 AST 复制或包装 AST 存在均不足以证明。合法历史适配保留可编译行为；无法证明的被引用核心项报 `SIGNATURE_UNVERIFIED`，已证明漂移报 `SIGNATURE_DRIFT`，不发射成功产物/成功生成收据。不得单纯将全部包装形状拒绝以清账。
2. **引用按来源身份闭合**：named/wildcard/alias/re-export、裸名及 receiver 调用、Auto body 内依赖与生成 db/helper 委派均从 resolver/lowering/最终绑定追溯。用户同名符号不冒充 stdlib；动态调用不得无依据视为未引用。六核心范围内证明不足必须拒绝，范围外仍明确记录等级与边界。VM 最终绑定查询和产物快照须一致，不只保留 pre-link 的推测证明。
3. **同输入、同目标比较共同装配身份**：先定义规范化字段与消费者专属字段。consumer/诊断路径/时间等元数据不得伪造装配差异或掩盖真实差异；可拆共同 assembly identity 与 consumer receipt identity，或采用可证明等价的明确投影，设计决策在 T-12 记录。同 target/environment/features/来源闭包的入口应一致；VM/Rust/C 不同目标有预期差异，不要求三目标指纹相等。投影不得删除真实依赖/provider/符号证明以获得绿。
4. **生成消费者也须绑定真实输入**：检查生成 workspace 实际采用的依赖版本/lock/features 与 manifest 的对应关系，区分生成器自身输入和生成产物运行时输入；复用门按实际依赖失效，ready 与收据的相同常量不是完整依赖证明。公共/选定层及缺失身份、依赖闭包、provider schema/实现、编译模式等原合同要求均须可核验；哪些显式序列化、哪些由内容输入证明，在 schema/SD 中一致说明。
5. **范围与规范一起收口**：完整核查 bulk fmt，不重写或丢弃历史修复证据；SD-01..07 对照当前 canonical 与最终实现形成可应用正文和证据映射。执行只更新计划/报告中的沉淀稿，canonical/ledger 沉淀留给独立复审通过后的 merge 阶段。

### 规范增量

起草只提出增量，不修改canonical。736落地后再对最终服务Spec兼容。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/assembly-manifest.md` | 文件存在覆盖 → actual plan/manifest、源段/目标/环境、验证等级、provider与核心门 | 明确装配合同 | AC-01..08 |
| SD-02 | modify | `docs/specs/stdlib/design/backend-assembly.md` | VM路径/宿主覆盖分散 → actual provider索引与核心能力；修正旧HTTP矩阵，区分inventory与执行 | 原始多后台诉求 | AC-01/03/05/08 |
| SD-03 | modify | `docs/specs/auto-lang/frontend/design/module-resolution.md` | 解析与两loader分叉 → 解析后统一装配、源位置、原use隔离/循环语义和cache边界 | 保持模块语义 | AC-02/04/06 |
| SD-04 | modify | `docs/specs/auto-lang/trans/overview.md` | Rust/C实现来源不透明 → selected target/provider和真实emit证据/unsupported诊断 | 不把VM preparse当Rust实现 | AC-02/03/05/07 |
| SD-05 | modify | `docs/specs/auto-man/design/api-generation-integrity.md` | body/source指纹 → 额外assembly/provider依赖闭包与新鲜度门 | 防改stdlib仍用旧服务 | AC-06/07 |
| SD-06 | modify | `docs/specs/auto-cli/project.md` | 无inspect → inventory/actual manifest JSON及check退出码 | 可检查交付 | AC-01/07/08 |
| SD-07 | modify | `docs/specs/stdlib/project.md`、`docs/specs/auto-lang/runtime/design/networking-stdlib.md` | 后台/物理填充草案不清 → 新合同入口、六模块能力与D3a/D3b边界 | 不声称补齐全部后台 | AC-05/08 |

## 6. 测试设计

### 6.1 最小同源与反例

| 族 | 必须观察 |
|---|---|
| 目标装配 | synthetic public+vm/rs/c层不同返回值，真实VM/Rust/C对应选择与执行；公共pure body共享、显式host provider无sidecar合法；foreign层不能串入。 |
| VM两入口 | CompileSession及persistent同模块选择/来源；auto.io前缀与目录module正确，bare/named/wildcard/re-export/循环无新增泄漏。 |
| 冲突/来源 | 声明+目标合法，重复body/provider、self/static/async/返回/参数漂移、未知类型/同名用户模块；双源位置，包括目标文件故意语法错误。 |
| native绑定 | 有名字/ID无真实shim、冲突ID/合法alias、feature off、stub占位；装载/链接失败且不执行业务。独立producer签名改动可被识别，不能复制同一AST蒙混。 |
| 六模块 | 每公开符号有支持/拒绝/未验证分类；被引用Unsupported/missing明确错误，未引用不害整个module；已有json/io/native HTTP/task语义不擅改。 |
| 环境 | Native VM/Rust backend、Browser不可监听/文件/socket、同Vue前端Native后端的双context；UI adapter不假冒stdlib suffix层。 |
| 缓存 | 只改目标body/签名、增加/删除层、依赖/host feature与版本变更；同mtime不同内容、同名异root、双session、target切换；真实返回或当前诊断变化、旧ready不能被消费。 |
| 生成/CLI | auto stdlib inspect JSON稳定、check退出码/完整分母；真实生成Rust API→manifest→736启动；改assembly但不改api.at，新指纹或失败不启动旧impl。 |

inspect验证不执行网络/文件业务；真实执行witness在单独fixture，IO只用temp/loopback。C真MSVC/rustc/cargo单fixture实编；没有C HTTP provider是Unsupported而非造一个手写server。Rust普通stdlib同源witness及734/736实际generated API；不能只测loader helper或静态JSON金样。

### 6.2 核心与全库门

全库inventory所有公共/目标文件必须有记录及完整总数；真实未解析记录为partial、整体检查非零，不隐藏错误。六核心正常声明/provider需可解析并全部分类；可验证支持集必须signature_checked+必要exec证据，缺实现是Unsupported/missing且不冒称支持。

新测试族plan738_stdlib_assembly；真HTTP注册http_e2e_plan738并纳test-http-e2e串行，尽量复用734/736 fixture。不把全stdlib每符号执行一遍（阻塞stdin、任意FS/network、进程启动等）作为inventory。签名级fixture无需重网络；行为与资源parity另属D3b。

### 6.3 实施门禁（本轮不跑）

在 `D:/autostack/.wt/lang-738/auto-lang`：
- `cargo check -p auto-lang`、`cargo check -p auto-man`、`cargo check -p auto`；开发`cargo t plan738`、compile/module_cache/native scoped族、`cargo test -p auto --bin auto stdlib -- --test-threads=1`（新增CLI族）及auto-man API/provider scoped。
- 复审裸`cargo t` + `cargo tv` + `cargo tt` + `cargo th -j 1`（nextest 串行）；实际ui_gen消费点改动才`cargo tu`。a2r-std若为provider元数据必要改动，补该crate scoped/适用全crate；不改auto/lib/aavm，无taa。
- 真Rust/C/witness生成实编、736代表CLI/配置/ready与729/730/734兼容族；三目标unsupported另有负测。
- 不改语法参考/文档生成器，assembly JSON为工具模型而非语言Schema；若实际改变文档Schema定义触发docs_gen，按最终diff判定。tf只merge到期主检出单实例，不为本期每plan跑。
- 零新增确定性红，按同命令基线逐名分诊，不靠改公开API、删provider/跳过fixture清账。

报告新`docs/plans/reports/738-stdlib-{decision,inventory,assembly,providers,cache,verification}.md`及inventory/manifest JSON；绑定最终revision、source/deps/hash、每AC/SD证据与验证等级。

### 6.4 Phase 3 定向验证与最终证据

| 对应任务 | 必须观察的正例/反例 | 证据与通过条件 |
|---|---|---|
| T-09 | `2c1b4a763` 对父提交及实际合入范围的完整差异分类 | 每个变更文件有分类/依据，非格式语义变化进入复审；不能只抽查或以测试绿证明纯格式。 |
| T-10 | 合法包装/三参分派/状态侧信道、Standalone 与 Embedded 两 provider；错元数、返回、async/await 和被替换 callee | 合法适配实际 Rust 编译通过、行为断言成立且具有完整证明；漂移/未证明核心调用在发射/生成成功前拒绝，不能接受 Resolved-only。 |
| T-11 | 限定/裸名、named/wildcard/alias/re-export、receiver、Auto body、生成 db/helper 内同一核心调用；同名用户符号 | 各真实路径均收集正确 identity；植入缺 provider/签名漂移后同样拒绝；普通导入不拒绝未引用 unsupported，不引入 use 语义泄漏。 |
| T-12 | CLI actual ↔ 会话/产物 manifest ↔ 生成 receipt/ready；普通/persistent VM、Rust/C、生成 API 分组 | 同 fixture 同目标共同身份一致，消费者证据与最终绑定一致；不同目标字段差异符合选择结果，inventory/candidate 不冒 actual。 |
| T-12 | 不改 API 源，仅改 public/选定层/依赖/provider/schema/features；缺失、读取失败、unknown/null、同内容异绝对根 | 真依赖变更使新身份改变或编译拒绝、旧生成失效；同内容便携身份稳定；生成 workspace 实际 lock/features 有匹配/失配证明。 |
| T-13 | SD-01..07 与当前 canonical、最终 schema、等级、退出码逐条对照 | 2=错误、3=partial；每条规范都有 AC→代码→证据，无未实现字段/能力冒称已交付。 |
| T-14 | 最终 clean 提交上的分级门、三目标/C stdio witness、真实生成服务 | 每条收据含 code commit、依赖与 features、命令、数量、结果及日志 hash；0 tests 不能通过。 |

开发阶段先 `cargo check -p auto-lang` 和目标 scoped 族；沿用 §6.3，不重复全档。T-14 固定最终提交后再完成裸 `cargo t`（完整非 fail-fast）、`cargo tv`、`cargo tt`、`cargo th`；HTTP 档 nextest 串行参数用 `cargo th -j 1`，不用 libtest 的 `--test-threads`。另跑 CLI `cargo test -p auto --bin auto stdlib -- --test-threads=1`、auto-man `cargo test -p auto-man --lib api_gen -- --test-threads=1`，适配/witness 的现有 ignored 测试按实际名称和 feature 显式启用。新增生成服务验收以现有命令 `cargo test -p auto-man --lib --features test-http-e2e http_e2e_plan738 -- --ignored --test-threads=1` 复跑，必须实际选择并通过 1 项，覆盖生成→实编→serve→ready→业务→stdlib-only 失效→再生→重建复验。

若新改动触及 ui_gen，追加 `cargo tu`；只有触及 AGENTS.md 所列 aavm 路径才追加对应 `taa`。本次修订为计划簿记，不运行 Cargo；后续纯报告/SD 操作也不单独触发 `cargo t`/docs_gen。已有红仅凭相同命令/配置/依赖的基线与逐名证据归类，flake 需隔离复跑与机制记录；未解释新红保持失败，不缩断言/跳测试。全部机器负载串行安排，不运行 per-plan `tf`。

## 7. 验收标准

| ID | 可观察交付 | 验证/期望 |
|---|---|---|
| AC-01 | 全stdlib/auto目录清点与六模块公开符号完整 | 实际文件/AST数量、inventory JSON和partial诊断；没有“解析失败就略过”的绿；支持集/缺口有分母与原因。 |
| AC-02 | 生产VM/Rust/C/persistent实际选择共同装配计划 | 同源不同target真执行+manifest源/provider一致，无VM层串Rust/C；合法host provider缺sidecar不误拒；不是孤立helper。 |
| AC-03 | 核心声明/实现/实际native绑定校验 | 合法声明+一impl绿，冲突/签名漂移/native空ID/假名字/feature off精确诊断；IDalias与独立producer元数据有证据。 |
| AC-04 | 源段错误位置与既有use隔离保留 | 目标层语法/冲突指向实际文件行，root/body同名反例、bare/wildcard/named/re-export/循环回归不泄漏；双session不串。 |
| AC-05 | target/environment与核心能力严格区分 | 六模块全矩阵，Browser不可用明确Unsupported，Vue frontend不改变Native backend；stub不当支持，不新造未支持backend。 |
| AC-06 | 内容/目标/provider/依赖变化正确失效 | 同mtime、增删sidecar、双root/session、provider/features与target变更真编译结果/诊断；旧bytecode/manifest/APIready不冒新实现。 |
| AC-07 | CLI与生成HTTP服务真实消费manifest | inspect/check JSON/退出码/副作用隔离；generated Rust实编、736实际startup/ready与assembly fingerprint，改stdlib后新鲜度失败或重新生成。 |
| AC-08 | 核心/全库验证等级、兼容与规范可沉淀 | 全SD对应证据、零新增确定性红；729/730/734/736既有行为保持，D3a未验证/ABI/调度范围明示，不称全部stdlib已parity。 |

## 8. 执行步骤

> 2026-10-09 独立复审 R1：**needs_fix**。下列历史“已完成”段保留为执行记录，受影响勾选已撤回；它们不再代表验收通过。
> T-01 调查成果保留；T-02→R2/R7，T-03→R1/R5/R8，T-04→R1/R2，T-05→R3/R4，T-06→R4/R5/R6，T-07/08→R7/R9。完整复现与修复合同见 [独立复审报告](reports/738-independent-review.md)。

### 8.0 Phase 3 执行入口与任务映射

原始实现与 R1 修复记录、R2 修复/服务验收记录作为前两阶段历史保留；当前新增 **Phase 3（Revision 3）**，任务总数由 8 增至 14。仅 T-01 保留完成，current_step=1；T-02..T-08 不因新 phase 出现而自动关闭，须由新任务证据逐项对账。

| 阶段 | 当前定位 | 下一步 |
|---|---|---|
| 原始实现 / R1 | 历史调查和 needs_fix 证据保留 | 不恢复旧 reviewed/execution_done 声明。 |
| R2 修复与验收 | `2c1b4a763`，服务 1/1、同源 witness 与分级门执行收据 | 作为 Phase 3 入场基线；不视为独立 review pass。 |
| **Phase 3：最终契约修复与验收收口** | **T-09..T-14 全部待执行** | T-09 → T-10 → T-11 → T-12 → T-13 → T-14 → 独立 review。 |

执行 agent 读取主检出本 revision 计划，在既有 `D:/autostack/.wt/lang-738/auto-lang` 实施，不重建/重置 worktree、不另取号、不合入。每个独立验证单位提交；新增报告记在主检出 `docs/plans/reports/`，生产代码/测试仅在原 worktree。工作树/依赖与基线发生变化时先记原因和新版本，不能静默挪用旧收据。

### T-01：最终736基线、装配调用图与六模块provider冻结 ✅

- [x] 前置736review pass+merge，读最终config/ready/生成与canonical增量；`bash scripts/new-wt-group.sh lang-738 --branch plan-738-dev`，auto-down兄弟只读，禁止链接。
  [✅ 已完成] 2026-10-08 用户指令「实施它」进入 executing（原 736 前置的替代满足与 736 现状勘验见 `docs/plans/reports/738-stdlib-decision.md` §5/§7）；worktree 组 `D:/autostack/.wt/lang-738/{auto-lang,auto-down}` 已建（基面 master@c3ccd32c3，auto-down detached 895f8d0 只读，无链接）。T-06 消费 736 最终合同保持 gated（§8 复核清单在报告中）。
- [x] trace CompileSession/persistent、Rust/C实际CLI及生成API、native实际注册和a2r分派，复现后缀目标/缓存/源位置缺口。确认实际active consumer，不把未调用方法当验证面。
  [✅ 已完成] 四装配面实勘冻结（报告 §1/E1-E4）：①VM 管线 resolve_uses→load_module 硬编码 .vm.at；②persistent 双 auto/ context 恒缺失+stdlib-only 查找；③Rust trans 零模块装载+三处重复 26 名名称表（~10 名无真实 a2r-std 模块）；④C trans 头包含路由+cmd_a2c_stdlib 逐文件生成。ModuleCache 双层恒 miss（E2）；native 名 surface=磁盘扫描 CWD stdlib/auto + 目录固定 ID + ~54 手工 shim 三机制（E3）；find_std_lib 项目根分支恒不命中的来源身份裂缝（E3）。
- [x] bounded调查产出738-stdlib-decision：六模块完整符号/producer签名来源、provider目录schema、active native别名/ID、混合public属性兼容、target/environment来源、源段与cache key、初始全库分母/parse失败。
  [✅ 已完成] `docs/plans/reports/738-stdlib-decision.md`（§2 六核心 13 层 parser 精确分母：11 parse-OK + async.at/json.rs.at parse 破损冻结；§3 provider 目录要点与名称漂移矩阵；§4 设计裁决建议）。
- [x] 最小公共+三目标/host mapped真入口原型；无法闭合真实target接线/全核心分类则needs_replan，不将AC换成inventory-only。涵盖全AC/SD。
  [✅ 已完成] 探针族 `crates/auto-lang/src/tests/plan738_assembly_probe_tests.rs` @worktree `ac41c2d1e`：P1-P6 六测全绿（真实 CompileSession 管线/trans 双入口/persistent/六核心 parser 清点）；target 接线可闭合评估=不触发 needs_replan（报告 §4.2）。

### T-02：共同模型、provider目录与全库inventory

- [ ] 依赖T-01。新增stdlib_assembly模块和stdlib/assembly-providers.json，lib.rs导出；复用AST/类型身份，inventory全文件/符号与明确验证等级。
  [✅ 已完成] worktree `d70d4c8a9`：`stdlib_assembly/{mod,model,loader,providers,validate}.rs` + lib.rs 导出；全库扫描 115 `.at`（74公共+23vm+13rs+5c）分母恰清点、FNV-1a 内容指纹、验证等级 Declared 起步、符号归一身份（parent/ext owner → `Owner.name`，遍历 TypeDecl.methods）；provider 目录六核心逐目标声明 + 发射名称表 11 名 unsupported。
- [ ] provider目录与真实注册/emit表有机器对照，文件hash/源段/候选与选择不混淆；非核心解析错误完整记录。
  [✅ 已完成] `cross_check_real_surfaces`：vm 声明对照 BIGVM_NATIVES 名 surface（CWD 钉仓根后扫描）、rust a2r-std 对照 `pub mod`、c 对照磁盘 locator；unsupported 必填 reason；发射表漂移差集与目录 unsupported 集合哨兵断言。实勘修正：sse parse_sse 无任何 callee（落册 unsupported）；发射表缺 `file`→`a2r_std::file` 共 11 名漂移。全库 isolated-parse 基线 39/115 三类构成（决策报告 §2b）。
- [ ] `cargo t plan738`/模型单测与inventory JSON完整性；AC-01/03/05，SD-01/02/07。
  [✅ 已完成] `cargo t plan738` 11/11 绿（6 探针 + 5 正式族）；稳定 JSON（双序列化逐字节一致 + 排序自检 + 无绝对路径泄漏）；scoped：module_cache 10/10、native_registry 13/13；`cargo check -p auto-lang` 新文件零警告。

### T-03：VM/目标转译/persistent装配接线与源映射

- [ ] 依赖T-02。修改compile.rs::load_module_inner、autovm_persistent.rs::load_and_register_module、lib.rs trans_*入口、parser必要诊断/source映射、trans/{rust,c}.rs实际provider消费。
  [✅ 已完成] worktree `340de9ccd`：①load_module_inner 层选择由 `AssemblyTarget::context_extension()` 驱动（硬编码 ".vm.at" 收编；Rust/C 目标零装载面——层记 candidate 不合并，发射面如实走名称表/头包含）；②`LayerSelection` 记录进 session（manifest v0 seam，T-06 消费）；③persistent context 路径修复（原 `with_extension` 吃掉 `.vm` + 未剥 `auto/` 前缀双 bug 使 `.vm.at` 恒不可见）+ ext canonical 对齐（`auto.<stem>.<target>.<fn>`，TypeDecl 并入型与 ext 型双臂）+ edition2021 `if let` MutexGuard 双锁死锁修复（context 修复使查找首次可达后暴露）；④trans_c/trans_rust 入口先声明装配目标；⑤Rust sibling 扫描排除 .vm.at/.c.at foreign 层（VM/C 层不再串入 Rust 发射类型上下文，.rs.at 为 Rust 选定层保留）。
- [ ] resolved public路径+显式context选源，root在前/选层后；public/native host映射与pure body均可追踪，不把Rust解析预处理当VM执行实现。保持545/635路径/可见性/循环语义。
  [✅ 已完成] 源段边界记录（context_byte_boundary）+失败路径有界归因：公共段单独可解析 ⇒ 错误归因目标层真实文件（`target_layer_syntax_error_attributed` 实证）；545/635 语义由 use_semantics 族回归守护（117/117 含 use_semantics/module_cache/native_registry/repl/plan727/729/730）。
- [ ] 三目标同源真正执行/emit与VM两入口对拍、源层错误位置；AC-02/04/05，SD-01/03/04。
  [✅ 已完成] `target_driven_layer_selection_and_record`（VM 合并 vm 层、Rust/C 上下文无 foreign 层符号、candidate 齐、记录全字段）；`vm_two_entry_same_layer_visibility`（session 与 persistent 对 io ext 方法/net 顶层 fn 同可见——P5 探针由破损证据翻转为修复后契约）；`trans_entries_declare_assembly_target`。门禁：plan738 15/15、scoped 117/117（plan730_staged_lease_expiry 首跑并行负载 flake，复跑单测 0.44s 绿——上传服务非本触面）、`cargo tv` 162/162。

### T-04：核心符号/native校验与环境能力

- [ ] 依赖T-02/03。修改vm/{native_registry,native,native_catalog}.rs与ffi/stdlib.rs必要provider描述/实际绑定检查、codegen所需引用闭包消费；Rust核心provider分派使用真实identity。
  [✅ 已完成] worktree `855a143b6`+`9cd807027`：validate_core_vm_bindings（resolved+bound 双面成立才 Supported，返回类别未知=Bound 不冒称 SignatureChecked）+Browser 三族 Unsupported 分类+ID 别名冲突检测（声明组/末段别名两合法形）+core_status_diagnostics 诊断视图。
- [ ] 六模块全部decl状态，独立逻辑signature/alias/feature/stub与实际shim绑定；避免name存在即绿，缺实现/冲突/不适用在业务前错误。目标物理字段身份记录，不重做资源ABI。
  [✅ 已完成] sse parse_sse=DeclaredStub、http 扫描名 vs 公共名两套面裁决①（权威名=公共 canonical 面）、file/fs 族 id 共享别名裁决②（read_text 收敛单名+声明组放行）；io 方法第四绑定面（VmModule 方法表）冻结文档化 Unverified（§9/P738-D2）；plan738 19/19。
- [ ] native欠绑定/ID冲突/签名漂移/假同名/Browser拒绝与现存核心支持正测；AC-03/05/08，SD-01/02/04/07。
  [✅ 已完成] 13 处生产扫描面 id 相撞真冲突冻结基线（分诊裁决=T-06 CLI 如实上报非零，重编号归 D3b——P738-D1）；plan738 19/19+scoped 117/117。

### T-05：缓存依赖与编译epoch一致性

- [ ] 依赖T-03/04。修改module_cache.rs与compile.rs早退/dirty链、persistent manifest状态；缓存包含全部selected source/provider/target/env/deps/存在性。
  [✅ 已完成] worktree `f88449944`：ModuleCache 段级指纹条目（公共+选定层各自 FNV-1a+选定层 absent 台账+provider schema 版本+依赖闭包指纹）+AutoCache 装配感知查找 `get_valid`（跨 target 条目共存，身份/schema/段指纹/依赖指纹任一漂移即未命中）。
- [ ] 只改target层或host feature新编译不能旧返回；活VM禁止无契约ABI热换，明确target/root改变需新session。缓存错误不降级旧module。
  [✅ 已完成] ①E2 死缓存反转为契约（P2 探针翻转：双层模块可命中且核对全部选定源段；同 mtime 改内容仍失效——内容指纹非 mtime）；②缓存命中早退一致性：命中补全本 epoch bytecode+manifest 记录（半截模块禁止，clone 会话实测）；③`set_assembly_target` 守卫（已装载换目标=session_target_mismatch 须重建，同目标重声明放行）；④stdlib root 身份守卫；⑤persistent 活 VM 指纹台账（同内容幂等放行/内容或根漂移拒绝）；⑥验证失败一律未命中重编译，不降级旧模块。
- [ ] 同mtime/增删层/双root/session/target/依赖变更真实结果与diagnostic；AC-04/06，SD-01/03。
  [✅ 已完成] t05_cache_consistency 7 测：命中重建 bytecode/manifest、同 mtime 改层（行为断言新符号可见）、增层（absent 台账）/删层真实失效、retarget 拒绝+合法路径、stdlib root 变化拒绝、跨装配缓存不串 VM 层、依赖变更失效依赖方；plan738 27/27+module_cache 16/16+use_semantics 7/7+native_registry 13/13+autovm_persistent 20/20+plan727/729/730 59/59+`cargo tv` 162/162。

### T-06：检查CLI和API generation/serve收据

- [ ] 依赖T-02..05。新auto/cmd_stdlib.rs，main.rs挂接；改auto-man/api_gen.rs及736最终真实生成/启动消费者（T-01锁路径），assembly fingerprint进入734receipt并由736ready引用。
  [✅ 已完成] CLI 半（worktree `206a846e7`——`auto stdlib inspect` 三模式+稳定 JSON+退出码 0/1/2/3+真实二进制冒烟）+ gated 半解锁落地（worktree `d71e823c8`）：勘验 736 代码已全量在 master（`master..plan-736-dev`=0，另一会话 fold 节奏合入）→ worktree re-sync（`0424673e2`）→ `stdlib_assembly_fingerprint`（内容级身份）进 generation.json 收据 assembly 块（与业务 source_hashes 各自记录）；736 ready 引用落地为**复用新鲜度门**——`backend_generation_is_fresh` 纳入 assembly 比对，陈旧 bundle 拒绝复用走再生臂（§5.4：runtime serviceconfig 不是 assembly 指纹，config_hash 不混入；「旧 ready 不被消费」由门在启动前保证）。
- [ ] inspect只读取装配所需源码/元数据，不执行业务网络/文件操作；稳定JSON与非零check，inventory/actual mode区分；runtime serviceconfig不是assembly内容指纹，provider变化必须新鲜度失效。
  [✅ 已完成] inspect dry 装配+inventory/actual 区分+check 非零（CLI 8/8）；provider 变化（目录 bump schema）→ `stdlib_assembly_fingerprint` 变 → 复用门判陈旧 → 再生（`fingerprint_deterministic_and_content_sensitive` 冻结内容/目标敏感性；`assembly_freshness_truth_table` 冻结门语义）。
- [ ] CLI JSON/负测及真实生成Rust→serve，改stdlib但api.at不变；AC-01/06/07，SD-01/05/06。
  [✅ 已完成（serve 实跑 witness 归 T-07）] CLI JSON/负测 8/8；「改 stdlib 但 api.at 不变」语义由指纹敏感性单测+新鲜度真值表双面冻结；真实生成Rust→serve 实跑 witness 归 T-07（其 rustc 实编切片本就驱动真实生成链；fixture e2e 尝试因成员 pac.at 发现流程预存行为未走通，已记 §9）。

### T-07：同源样例、核心完整矩阵与实际执行证明

- [ ] 依赖T-03..06。新plan738_stdlib_assembly_tests.rs/tests.rs接线、examples/stdlib/assembly/及target witness；依现有C/Rust构建工具生成真实产物，不手写业务替代。
  [✅ 已完成] 三片提交：①`75c7a1049` VM 真执行见证+六核心矩阵（报告落盘 738-stdlib-matrix.{json,md}）；②`0a73d0759` rustc 实编实跑 witness（⑤腿 #[ignore] 按需门：witness.at→trans_rust→a2r-std crate 实编→实跑断言输出；实勘 as_int 下沉类型缺口+内嵌 a2r_std/crate 漂移两发现）；③`8964155ec` examples/stdlib/assembly 同源样例（真实 CLI inspect --actual 闭环验证）。
- [ ] 完成§6.1全部反例/真入口、六模块全符号target/env矩阵和全库inventory；HTTP串行兼容734/736与729/730，报告selection/exec等级/真实方法，旧root未消费侧层不能报已合并。
  [✅ 已完成] `a4afb3b65` 余项反例（用户同名核心模块干净遮蔽/双活跃 body 冲突不静默择一）；矩阵+inventory 完整（AC-01/05）；selection/exec 等级=LayerSelection+矩阵 verification 列；candidate 语义=「未消费层只能报 candidate」在 t03+CLI actual manifest 冻结。C 实编 witness 明示阻塞（无 C HTTP provider=Unsupported，不造 server——§6.1 允许）。
- [ ] AC-01..08，SD-01..07；无法执行目标/witness明确阻塞，不只生成串金样。
  [✅ 已完成] 全 AC 证据绑定见 `docs/plans/reports/738-stdlib-verification.md`（T-08 报告，绑定 e4425b673/最终 revision=1）。
- 完成§6.1全部反例/真入口、六模块全符号target/env矩阵和全库inventory；HTTP串行兼容734/736与729/730，报告selection/exec等级/真实方法，旧root未消费侧层不能报已合并。
  进度：矩阵+inventory 完整（上条）；缓存族=T-05 八测、VM 两入口/545 语义=use_semantics 回归、Browser 三族/Binder=t04、CLI=8 测。
- AC-01..08，SD-01..07；无法执行目标/witness明确阻塞，不只生成串金样。

### T-08：门禁、独立review与规范交接

- [ ] 依赖T-01..07。§6.3分级门、warnings/fmt/debug/未批准延期扫描，验证报告绑定最终revision/每AC/SD；新红基线对照，未覆盖核心不能标“全库通过”。
  [✅ 已完成] 门禁实测（worktree，基线 master@4262f761c 逐名分诊）：裸 `cargo t` 3 红全为 musk p053 预存族（红名集 6=6 diff 空）+`cargo tv` 162/162+`cargo tt` 12⊂13（plan707 flake master 独有未复现）+`cargo th` 1⊂2+plan738 32/32+⑤腿 witness 按需 1/1+CLI 8/8+auto-man scoped 45/45；三 crate check 零错误；改动文件 rustfmt 干净、零 debug 残留。验证报告 `docs/plans/reports/738-stdlib-verification.md`（`e4425b673`，绑定最终 revision=1）。
- [ ] /auto-plan:review独立验证真实callee和target调用点、分母/验证等级/缓存反例；准备SD沉淀稿，merge再canonical/ledger/Design33/索引和归档。
  入口就绪：status=execution_done，worktree `e4425b673` clean，复审证据齐备（§9 交接+验证报告）。SD 沉淀稿=起草时 SD-01..07 提案维持（merge 阶段执行）。
- [ ] 清理前lang-738两兄弟各wt-guard clean；tf仅merge到期主检出单实例。全AC闭合才reviewed/archived。

### Phase 3 / T-09：冻结入场版本并核清批量格式化范围 ✅

- [x] 依赖 T-01 与 R2 提交。复核 `2c1b4a763`、worktree clean、auto-down 版本和基线收据；审计 `2c1b4a763^..2c1b4a763` 全部 566 文件，并记录最终合入 diff 的比较基面。
  [✅ 已完成] 2026-10-09 worktree clean @`plan-738-dev@2c1b4a763`、auto-down `895f8d0` 未动，与 [738-phase3-baseline.json](reports/738-phase3-baseline.json) 一致；566 文件全部 Modified `.rs`（crates/auto-lang 内，无增删）；合入比较基面=merge-base `6d69dbdc7`（分支自基面 19 提交）。
- [x] 按功能修复、纯格式、金样/生成资产、其他语义变化逐文件分类，给可复核依据。对声称纯格式的文件用同版本 formatter/归一化前后等价等适用方法完整核对；格式混合功能文件须单独提取语义差异审查，不能一律标纯 fmt。确认后的格式变更可保留并纳入完整复审，或用后续提交缩减无关改动；不要求重写已记录提交，不丢 R2 实现。
  [✅ 已完成] rustfmt 1.9.0-stable/默认配置/edition 2021 逐文件机械等价：**552 纯格式（父 blob 格式化后与子 blob 逐字节相等；37 个 mod 声明根文件经整树解包原地格式化）**+4 注释/空白级（去注释归一后代码令牌等价）+2 a2r 金样（幻影 `a2r_std::io` 导入移除）+**8 功能修复**（native.rs 契约 API/host.rs 适配壳/emission+c.rs C 诚实面/compile.rs c.* 命名空间/disasm.rs CLOSURE 走查/trans_rust.rs 模块级 unsupported/back_proxy.rs 冗余撤销）——8 功能文件与 R2 报告逐项对账一致，无未声明语义变更；host.rs 包装豁免/Resolved-only 臂确认为 T-10 触面。
- [x] 新增 `docs/plans/reports/738-phase3-scope.{md,json}`，记录基面/文件清单/hash/分类/待审语义项。完成条件：每个变更文件均有归属，不存在未知语义差异。映射 T-08、AC-08、SD-01..07 的证据边界；本任务不靠全档测试验证格式性质。
  [✅ 已完成] [738-phase3-scope.md](reports/738-phase3-scope.md) + [738-phase3-scope.json](reports/738-phase3-scope.json)（566 条逐文件 parent/child SHA256+分类+依据；审计脚本 738-phase3-scope.audit.py 可重跑）。

### Phase 3 / T-10：修复 Rust strict 门并证明合法适配链 ✅

- [x] 依赖 T-09。触面：`stdlib_assembly/host.rs::{verify_rust_reference,...}`、`model.rs`/`validate.rs`（按所选契约表达需要）、`trans/rust.rs::call` 与既有 HTTP lowering、`crates/a2r-std/src/http.rs` 和内嵌 `crates/auto-lang/src/a2r_std.rs` 的实际 producer 定义。先用现有包装形状构造最小反例，区分明确放行缺口与尚未实编的形状假设。
  [✅ 已完成] worktree `c1579ed71`（基线 `2c1b4a763`）：最小反例先行（元数 2/3 参、缺 await、cast f64、未知块结构、Standalone 3 参 post）；实勘放行缺口三处——包装 async 豁免、`wrapped||mode_variant` Resolved-only 元数放行臂、`fn expr` Await 臂旁路 `call()` 验证钩子。
- [x] 给三参 post、post_sync/last_status、Await/Cast/(async)Block 等已支持适配建立双源可验证合同；元数/返回/async 漂移和无证明不再因 wrapped/mode_variant 豁免。strict 发射/生成入口只接受充分证明；Resolved-only 仍可用于清点，但不得成为引用核心调用成功的依据。保持 PLAN-724/729/730 合法能力与公开 API，不复制公共签名充作 producer 证据。
  [✅ 已完成] `ADAPTER_RULES` 12 条独立契约（R3-02 勘误：初稿误记 14）（侧信道族 sync/async 变体、last_status 数值 cast、3 参 post 元数分派【Embedded 限定——a2r-std crate 无 3 参 producer，Standalone 诚实拒绝】、三个 async 流 facade）+ 结构化 `observe_emission`（块结构逐段核对）+ 四源一致（发射↔契约↔producer↔公共投影）；Resolved-only 放行臂删除；类型名 pattern 证据废除；扩展位（api_key）由契约 `extension_params` 冻结不复制公共签名；Await 臂接入同一门。plan724 探针改判：Standalone 3 参 post 旧文本断言冻结的是从未可编译形状 → 改断言诚实拒绝；724 可编译能力（侧信道/golden/e2e）全部保留。
- [x] 在现有 `host.rs` 测试及 `plan738_stdlib_assembly_tests.rs`/相关 trans 族补正负例；Standalone/Embedded 分别真实编译适配 witness，有可观察返回/状态断言。开发运行 `cargo t plan738`、触及 adapter 的 scoped 族和 `cargo test -p auto-man --lib api_gen -- --test-threads=1`。新报告 `738-phase3-strict.md` 记录旧失败/新成功及拒绝位置。映射 T-03/04/07、AC-02/03/05/07/08、SD-01/02/04/05/07。
  [✅ 已完成] [738-phase3-strict.md](reports/738-phase3-strict.md)：plan738 62/62（host 新 5 测正负例）、plan724 5/5、api_gen 43/43、e2e witness 3/3（Standalone build_and_run 实编实跑断言 auth 200/body；Embedded 内嵌 3 参 post+post_sync+post_bearer_sync 可观察返回）、tt 全档 15 红全分诊（11 文档化 master 预存+4 负载 flake 含 plan730 隔离复跑绿）零新增确定性红、use_semantics 7/7、module_cache 16/16、rustfmt 干净。

### Phase 3 / T-11：实际 callee 与引用闭包完整性审计/修复（R5 重开）

> R5：以下完成说明保留为历史执行证据；本任务尚未满足最新复审，修复要求见 §9 R5 与 738-review-r5.md。

- [x] 依赖 T-10。触面：`stdlib_assembly/reference.rs::verify_linked_native_closure`、`trans/rust.rs` 的 call/use/receiver 分派、`compile.rs` 的来源记录、`lib.rs` VM/Rust/C 入口、`autovm_persistent.rs`、`auto-man/src/api_gen.rs::generated_api_assembly` 的 endpoint/db/helper 消费。列出所有进入六核心 producer 的实际路径，核对 329 个发射点的分类依据；明确哪些确为胶水/面外，哪些需追踪。
  [✅ 已完成] worktree `84bec29ef`（基线 `c1579ed71`）：139 处六核心 FQN 引用按 11 组发射臂逐类归属（报告 §1 表）；「329 发射点均属胶水/面外」声明不成立——四类用户可达形状（Dot 模块臂/裸名流族/三段 auto.core.method/for-in 反糖）全部接入 strict 收集，#1/#6/#9 确面外，#4 legacy 别名与 #11 接收者面按既有 P738-D2/第四绑定面债务显式记边界。
- [x] 用具名/通配/别名/再导出/裸名/方法/Auto body 依赖及生成 db/helper fixture 对照：同一核心调用跨合法写法仍收集同一来源 identity；替换独立 producer 合同或 callee 后，每条真实路径都必须拒绝。补遗漏；同名用户符号及未调用 unsupported 导入不误拒。动态路径不能以 AST 未出现 `module.symbol` 为排除依据，需实际绑定证据或明确拒绝理由。
  [✅ 已完成] 新闭包七项：`verify_rust_import` 具名导入绑定+签名（纯适配面符号拒绝+限定拼写指引——旧路径裸调用发射无壳破损产物静默漂移）；裸名调用限定重建验证；wildcard 公共面唯一归属/多模块歧义拒绝/§5.3 不越界；用户同名 fn 遮蔽优先；Bina 三段 io 等幻影诚实拒绝；for-in 流反糖验证（AsyncHTTPStream 参数 facade 契约视图）；**api_gen db/伴生 back 模块 Embedded 证明并入 manifest**（原 Standalone 默认+丢弃证明）；ADAPTER_RULES 增至 15 条（stream 三件套 async 面；R3-02 勘误：初稿误记 17）。
- [x] 新增 `738-phase3-reference-audit.{md,json}`，逐路径关联入口、collector、最终 callee、proof、测试与例外边界；执行 `cargo t plan738`、use_semantics/native_registry/autovm_persistent 等实际触面 scoped 族。完成条件：六核心引用闭包无未经证明的放行，VM preflight 结果和最终 manifest 无陈旧证明。映射 T-02/03/04/07、AC-01..05/07/08、SD-01..05/07。
  [✅ 已完成] [738-phase3-reference-audit.md](reports/738-phase3-reference-audit.md) + [json](reports/738-phase3-reference-audit.json)：plan738 67/67（t11_reference_closure 5 测）、plan724 5/5、api_gen 43/43、freshness 1/1、tt 非基线红=0（15 红全分诊）、use_semantics/native_registry/autovm_persistent/module_cache 7/13/20/16 全绿；VM 闭包走查沿用 R2 已闭合路径在案。
  [✅ 已完成·R5 修复重闭] R5-02 修复（worktree `bc0b95cfd`/`68398d2d6`）：公共方法 receiver 调用进 strict 门——`Owner.method` 分母归属 + 三发射形态收敛（模块限定/扁平 value_* helper/json 值绑定 def-use 直发）+ 方法 receiver/is_static 契约来自公共声明 + 形态守卫（拆段片段/链式中间态记边界）；CLI 级反例闭合（漂移 exit1+SIGNATURE_DRIFT+JsonValue.len proof 在案，原 exit0）；tt 6 语料回归修复后非基线红=0。
  [R6 重开 2026-10-10] R6 实测（真实 auto.exe）：`len` 闭合确证，但 `v.keys()`/`json.keys(v)`/`v.is_null()`/`v.as_int()` 漂移后仍 exit 0 无 proof——shape③ len-only、模块限定方法裸名未命中不落穿方法归属、value_to_int 腿死码；「公共方法 receiver 调用进 strict 门」仅部分成立。修复要求见 §9 R6 / [738-review-r6.md](reports/738-review-r6.md) P738-R6-01。
  [✅ 已完成·R6 修复重闭] R6-01 修复（worktree `6c3943327`）：receiver 方法面泛化全公共方法（shape③ 全方法/模块面落穿 public_method_symbol/未接管发射臂 case④ 诚实拒绝/json 值实参 Value 面分派修正/Vec<T>→[]T 序列面）；双拼写漂移矩阵（keys/as_int/is_null × receiver/模块限定）落为正式测试——keys/as_int 基线零违规+proof+漂移拒绝，is_null 双拼写基线即拒（公共 int vs producer bool 真漂移诚实面）。R6-02：bind 写失败保守判陈旧。
  [R7 重开 2026-10-10] R7 全分母普查（真实 auto.exe）：矩阵面闭合确证，但分母成员 has_key 的模块拼写 `json.has_key(v,"k")` 漂移后仍 exit0 无 proof（发射臂拆段写括号不平衡+形态守卫跳过，P738-R7-01）；case④ 误伤 `json` 同名局部变量/本地模块（P738-R7-02）。「泛化到全部公共方法」未完全成立。修复要求见 §9 R7 / [738-review-r7.md](reports/738-review-r7.md)，验收须扩到 12 方法双拼写分母全量。
  [✅ 已完成·R7 修复重闭] worktree `3713337d9`：R7-01/02 闭合——has_key 发射括号 bug 修复、全分母 12 方法双拼写矩阵（正式测试，绿面/真漂移面/parser 拒绝面/无臂 case④ 四类预期）、BoolToIntIf+len ScalarCast 契约、Paren 剥壳、AST 实参序列化、Null 哨兵族扩展、守卫诚实拒绝、case④ local 排除。

### Phase 3 / T-12：manifest 共同身份与多消费者三角对拍（R9 重开）

> R9：以下完成说明保留为历史执行证据；读取失败角落与最终格式门尚未闭合，修复要求见 §9 R9 与 [738-review-r9.md](reports/738-review-r9.md)。
> [✅ 已完成·R9 修复重闭 2026-10-10] `41b4d4be5`：lock 读错误 fail-closed（gate `classify_lock_read` NotFound=absent 哨兵/其它=None 保守陈旧；生成侧 Err 拒写收据；单测 classify_lock_read_fail_closed 含端到端 absent×None=false）；正式生成服务链 1/1@42.29s（R10 补跑）；R10 pass 确证。

- [x] 依赖 T-10/11。触面：`stdlib_assembly/manifest.rs::{AssemblyManifest::freeze,...}`、`CompileSession`/batch/persistent 最终快照、`trans/{rust,c}.rs`、`lib.rs`、`auto/src/cmd_stdlib.rs` actual/check、`auto-man/src/api_gen.rs::{generated_api_assembly,current_generated_api_assembly,...}`、`rust_ui.rs` 新鲜度门与正式服务 ready。先记录共同 identity/消费者 receipt 投影、schema 兼容和已有收据迁移规则；未知/旧不完整快照须明确陈旧或拒绝。
  [✅ 已完成] worktree `b6cfc6df1`（基线 `84bec29ef`）：**schema 3→4 双指纹**——`fingerprint`=共同装配身份（consumer 名置空+consumer_input 源剔除的中立投影哈希）、`consumer_fingerprint`=消费者收据身份（全量 payload，=旧单指纹语义）；迁移规则=新鲜度门/api_gen 烘焙常量/在途核对全部切 `consumer_fingerprint()`（同消费者跨时语义零变化），跨消费者断言用 `fingerprint()`；未知/旧快照维持保守再生真值表。
- [ ] 按 §5.8/§6.4 完成内容闭包和身份缺项：public/实际选定层/缺失状态、依赖关系、最终引用证明、provider schema/实现、target/environment/features。核验生成 workspace 实际采用的 Cargo.lock/依赖/features，不把生成器的 Cargo 输入冒称生成服务输入。无须新增一套通用包解析器；在既有生成/构建/启动入口闭合本期依赖身份。
  [✅ 已完成] 既有闭包项核证（sources 角色三态/provider 按目标分叉+Cargo.lock+BUILD_INPUTS/T-10/11 契约证明入 references）；缺项补齐：generation.json ready 新增 `workspace_lock`（生成产物运行时依赖输入身份，absent 显式记录；与生成器 Cargo 输入分开）；复用门 `lock_freshness` 对拍（lock 出现/变化/旧收据缺字段→陈旧再生）。
- [ ] 对同 fixture 同目标做 CLI actual ↔ 会话/最终产物 ↔ generation/ready 三角断言，覆盖普通/persistent VM、Rust/C、生成 API 各入口；按内容变更/读取失败/异根的正负例验证失效。CLI 与 manifest 使用实际数据，不手工拼期待 JSON；对不适用的目标入口明确拒绝，不删消费者降低覆盖。
  [✅ 已完成] `t12_manifest_identity` 2 测（实际数据驱动）：三角=CLI actual 路径（session→compile_actual_references→freeze）↔ 真实 `trans_rust_with_session`+`freeze_assembly_manifest` ↔ 生成收据形态——三者共同身份全等/收据身份互异；正负例=同内容异根身份稳定、业务输入只改收据身份、target 变化改共同身份、lock 真值表（缺字段/漂移/absent 保守再生）。
- [ ] 新增 `738-phase3-manifest.{md,json}`，记录 identity 规则、消费者字段映射、实际差异及完整正负例结果。`cargo t plan738`、CLI stdlib 与 auto-man api_gen scoped 必须通过；真服务完整链在 T-14 最终提交复跑。映射 T-02/03/05/06/07、AC-01/02/04/06/07/08、SD-01/03/04/05/06。
  [✅ 已完成·R3 修复重闭] [738-phase3-manifest.md](reports/738-phase3-manifest.md)：R3 复审发现 T-12 插入 lock 真值表测试时截走 `assembly_freshness_truth_table` 的 `#[test]` 属性（真值表变死代码、"freshness 2/2"为同测试跑两遍假象）——修复见 worktree `9c255993b`（属性复位，双真值表独立运行，`--tests` 下 never-used warning 基线 1→0）；ADAPTER_RULES 计数勘误 12/15（R3-02，报告三处+计划两处）。重跑：freshness 双真值表绿+shell_pack 环境红在案、api_gen 43/43、check 零 error。
  [R3 重开 2026-10-09] 主体交付（schema 4 双指纹/三角对拍/lock 收据）经 R3 复核有效；重开原因=T-12 触面 `rust_ui.rs` 补丁意外剥离 `assembly_freshness_truth_table` 的 `#[test]` 属性（P738-R3-01：死代码+新 dead_code warning+freshness "2/2" 为重复注册假象），报告 ADAPTER_RULES 计数另失真（P738-R3-02）。历史执行记录保留：plan738 69/69、CLI stdlib 10/10、api_gen 43/43、plan724 5/5、三 crate check 零错误；freshness 计数按 R3 勘误（见 [738-review-r3.md](reports/738-review-r3.md)）。
  [✅ 已完成·R5 修复重闭] R5-01/R5-03 修复（`bc0b95cfd`）：lock_freshness 改一次绑定状态机（首物化判新鲜**并绑定实际身份写回收据**，此后严格比较——漂移/删除/缺字段拒绝；R5 探针落为正式状态机测试）；三角测试重写为真实消费者版（VM/Rust/C 三目标 CLI↔真实会话入口；生成腿=真实 generate_api+current_generated_api_assembly 对拍，可比面相等+运行形态 producer 差异显式钉死，不再 clone）。

### Phase 3 / T-13：SD-01..07 终稿与原任务验收对账（R9 重开）

> R9：以下完成说明保留为历史执行证据；读取失败角落与最终格式门尚未闭合，修复要求见 §9 R9 与 [738-review-r9.md](reports/738-review-r9.md)。
> [✅ 已完成·R9 修复重闭 2026-10-10] SD-05 读取失败拒绝语义与 `41b4d4be5` 行为一致（SD 沉淀 `adfb7a5d1` 九文件在修复链上逐字节未扰动，`git diff 19973b089 41b4d4be5 -- docs/specs/` 为空）；R10 确证。

- [ ] 依赖 T-09..12。修订主检出 `docs/plans/reports/738-sd-drafts.md`，形成对当前 canonical 可应用的新增/替换正文和位置；保留七条 SD ID 与 frontmatter spec-impact。纠正 SD-06 的退出码；核对 manifest 字段/共同身份、strict 验证等级、动态引用边界、真实生成依赖与 D3a/D3b 能力声明，不将草稿规范降到当前缺陷行为。
  [✅ 已完成] SD 终稿重绑 Phase 3 提交链（2c1b4a763→c1579ed71→84bec29ef→b6cfc6df1）：SD-01 升 schema 4 双指纹+Resolved-only 非成功依据；SD-02 增运行形态分家+适配契约注册表+json.is_valid 漂移如实上报；SD-03 增核心导入闭包；SD-04 增闭包全路径+面外边界显式化；SD-05 增 consumer_fingerprint 统一/back 模块闭包/workspace lock；SD-06 退出码纠正为 2=错误、3=partial（以代码为准）；SD-07 维持 D3a/D3b 边界。七条 SD ID 与 frontmatter spec-impact 不变。
- [ ] 新增 `738-phase3-acceptance.md`，逐条列 AC-01..08 → 原 T-01..08 / 新 T-09..14 → 最终代码/反例/日志 → SD-01..07；T-14 最终门禁栏先明确待补，待 T-14 完成再填最终收据，不在此提前宣告全 AC pass。重验完整 inventory 与六格公共 fn/method/type/field 分母，不以 call 单站点代替完整分母。已完成旧项可据最新证据勾选；缺项明确留开，不能因新 phase 完成就批量关闭原任务。
  [✅ 已完成] [738-phase3-acceptance.md](reports/738-phase3-acceptance.md)：AC-01..08 逐条→任务→证据→SD；T-14 栏全部标"待补"不提前宣告；分母重验锚 inventory/矩阵面（115 .at 分母+六模块全格）非 call 单站点；原 T-02..T-08 逐项对账（T-08 留待 T-14 最终档闭合）；P738-D1/D2 边界显式引用不转新债务。
- [ ] 完成条件：正文准确、所有 AC/SD 均有任务与证据，未批准遗漏不转债务。本步骤只改计划/报告，不发布 canonical/ledger，不跑 Cargo/docs_gen；映射原 T-02/06/07/08、AC-01/05/07/08、SD-01..07。
  [✅ 已完成] 本步骤零 Cargo/docs_gen；canonical/ledger 留 merge 阶段。
  [✅ 已完成·R5 修复重闭] SD-04/SD-05 重锚（receiver 全路径入 strict 面、lock 一次绑定语义）；acceptance 计数勘误清零（17→15）。
  [R6 重开 2026-10-10] SD-05 与行为一致确证；SD-04「receiver 全路径/未证明=SIGNATURE_UNVERIFIED 不产出成功产物」与实际行为不符（P738-R6-01），须随修复重锚或如实缩面。
  [✅ 已完成·R6 修复重闭] SD-04 按实际行为重锚（四形态 receiver 面+真漂移面诚实拒绝+Vec 序列面）；reference-audit #11 行更新；verification §8 补 R6 轮+plan498 flake 入册（R6-03）。
  [R7 重开 2026-10-10] SD-04「未证明=SIGNATURE_UNVERIFIED 不产出成功产物」与 audit #11「12 方法分母归属三形态验证+第四拒绝」被 P738-R7-01 反例证伪（`json.has_key(v,"k")` 漂移 exit0 产出成功产物）；面外边界清单缺 has_key 模块拼写守卫跳过格与 len/mod 契约缺口——须随 R7-01/02 修复再锚。
  [✅ 已完成·R7 修复重闭] SD-04 增补（适配契约族/Null 哨兵族/parser 拼写边界/无臂拒绝面）；verification §9 R7 轮收据。

### Phase 3 / T-14：最终提交门禁、真实验收与独立 review 交接（R9 重开）

> R9：以下完成说明保留为历史执行证据；读取失败角落与最终格式门尚未闭合，修复要求见 §9 R9 与 [738-review-r9.md](reports/738-review-r9.md)。
> [✅ 已完成·R9 修复重闭 2026-10-10] `41b4d4be5` 格式门收敛（四点 token 级纯格式+rustfmt 五文件干净）、三 crate check 零 error、零 debug 残留、worktree clean；门禁=plan738 73/73、tv 162/162（cb_web_mime PASS）、api_gen 44+1 ignored、lock 状态机/classify 1/1、服务链 1/1@42.29s（R10 实跑收据）；R10 pass。

- [ ] 依赖 T-09..13。先提交全部实现/测试，冻结 clean code commit 与依赖/lock/features。按 §6.3/§6.4 执行三 crate check、完整裸 t + tv/tt/串行 th、必要 scoped/CLI/API 档、三目标同源与 C stdio witness、正式生成服务完整链。运行期间不改输入；若再修代码，新提交重跑受影响门禁，不能沿用旧提交成功记录作为最终证明。
  [✅ 已完成·R3 修复重闭] 原 `b3a4d660e` 收据在案；R3 needs_fix 后新提交 `9c255993b`（R3-01 一行属性复位）按纪律重跑受影响门禁：freshness 双真值表真实运行绿（shell_pack 环境红在案）、api_gen 43/43、auto-man check 零 error、`--tests` never-used 基线 1→0、rustfmt clean。
  [R3 重开 2026-10-09] 门禁执行本身经 R3 复跑全部成立（R3 独立复跑：裸 t 17 红全分诊/tv 162/tt 非基线=0/th 缩减档同构/服务链 1/1@42s/双 witness 1/1）；重开原因=健康扫描漏检 `b6cfc6df1` 引入的新 dead_code warning（`assembly_freshness_truth_table` never used，P738-R3-01）且报告 freshness "2/2" 计数为重复注册假象。修复后须在最终提交重跑 freshness 族+健康扫描。历史执行记录保留：最终提交 `b3a4d660e`（clean，链 2c1b4a763→c1579ed71→84bec29ef→b6cfc6df1→1a983091f→b3a4d660e，8 文件 +2224/−284）；执行中修复①`1a983091f` lock_freshness 首次物化语义、②`b3a4d660e` workspace_lock 收据写入补位（两修复均经 R3 复核确认落地且无残留缺口——lock_freshness 真值表/服务链收据断言实证）。
- [ ] 新增 `738-phase3-verification.{md,json}`：命令、选中/通过/失败/跳过数量、code/dependency revision、日志/hash、逐名红分诊、Warnings/fmt/debug/遗漏扫描。R2 收据保留历史；无新确定性红、无未知失败；变更范围和新增证明适用面清楚，不能称全库语义 parity。
  [✅ 已完成·R3 修复重闭] 原 `b3a4d660e` 收据（[738-phase3-verification.md](reports/738-phase3-verification.md)）保留；R3 勘误（ADAPTER_RULES 12/15）已回写 strict/reference-audit 报告与本计划；独立复审 [738-review-r3.md](reports/738-review-r3.md) 证据在案。
  [R3 重开 2026-10-09] 重开原因=报告 freshness "2/2" 与 warning 扫描结论与实际不符（P738-R3-01/R3-02，勘误后随新最终提交更新本报告）。历史数据保留：三 crate check 零错；裸 t 5183/16 红全分诊（13 master 预存+3 flake 族；R3 复跑为 17 红=13 预存+4 flake，plan484_024 隔离绿属 T-10 文档化 flake 族）；tv 162/162；tt 13 红全基线；th 逐名分诊（back_proxy 基线、plan730 interop+multipart 基线 worktree 同命令复现实证环境红、sse_chain 隔离绿、余为级联）；plan738 69/69、plan724 5/5、CLI 10/10、api_gen 43/43；⑤腿 Rust 实编 1/1、C stdio 真实 MSVC 1/1、正式生成服务完整链 1/1；debug 残留 0/rustfmt clean（warning 扫描除外——R3-01 勘误）。
- [ ] 在主检出把证据对回原 T-02..T-08 的实现/验证项及 T-09..T-14；只有所有执行验收闭合才置 `execution_done`，独立 review/merge/清理项仍保持待办。交给未参与该实现的独立 review agent，按 `/auto-plan:review` 重新核对最终提交、所有 AC、实际 callee/manifest、完整 diff 与 SD 正文；执行 agent 不能自行宣告独立复审 pass。
  [✅ 已完成] 验收对账（[738-phase3-acceptance.md](reports/738-phase3-acceptance.md) T-14 栏已补最终收据）：AC-01..08 执行面全部闭合（T-08 最终档=本任务）→ **status: execution_done**。交接：独立 review agent 复核 `b3a4d660e`——全部 AC、实际 callee/manifest、`2c1b4a763..b3a4d660e` 完整 diff（T-09 分类+T-10..14 语义面）、SD 正文、th 环境红基线归因。
- [ ] 独立复审 pass 才允许 `reviewed`；needs_fix 返回本 phase，needs_replan 修订具体合同而不降 AC。当前授权止于修复/验收/复审准备，不合入/归档/删除 worktree，merge 时再 canonical/ledger/Design33/索引及 wt-guard。映射原 T-07/08、AC-01..08、SD-01..07。
  [✅ R4 独立复审 pass 2026-10-10] 当前 state=reviewed（R3 全 AC/SD 实现面 pass + R4 定向复核 R3-01/02 修复与受影响门禁）；不合入/归档/删 worktree（merge 待授权）。
  [R6 重开 2026-10-10] R5 needs_fix 后的重闭收据（`68398d2d6`，见 §9）经 R6 复核：R5-01/R5-03 闭合、R5-02 部分闭合（P738-R6-01）——本任务最终档须随新修复提交重跑受影响门禁+CLI 反例后再交下一轮独立 review。

## 9. 复审记录

### 独立复审 R10（2026-10-10，pass → reviewed；R9 必修闭合定向复审·全新上下文）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3（验收合同不变）
- outcome: **pass**——P738-R9-01/R9-02 在修复合并树 `41b4d4be5` 闭合确证；门禁全绿（含本轮补跑的正式生成服务链）；无新引入问题；修复合并树可落地，**可重启 merge**
- reviewed_commit: `41b4d4be52723ec2462e5ab511d3ea8e588dd38e`（worktree `D:/autostack/.wt/lang-738/auto-lang`，plan-738-dev，入场/结束 clean，只读复审；diff=5 files +74/−31 与声称触面一致）
- base_commit: 修复父 `19973b089`（R9-I/R9-II 对象）；历史链 R8 reviewed `3713337d9`/SD `641f66a5f`/SD 沉淀 `adfb7a5d1`；主检出入场 `925c3186a`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（未触）
- 复审范围声明：R8 已对全 AC/SD pass、R9-I/II 已确证主链修复机制并判两项 reviewed 树既有必修；本轮定向复核两项闭合+修复合并树回归面+一致性，不重复全 AC 审计（合同面经本轮机械复核在 19973b089..41b4d4be5 逐字节一致，VM/契约门禁省略依据成立）
- R9-01 闭合确证（major）：gate 侧 `classify_lock_read`（NotFound→`Some(Vec::new())`=absent 哨兵；其它错误→`None`）→ `current_lock` 映射（None 不进 map）→ `lock_freshness`（L4079-84）`_ => false`——**recorded="absent"×PermissionDenied 现判 false**，显式断言在案（classify_lock_read_fail_closed L5188+真值表 L5076）；旧 `.ok().or_else(absent)` fail-open 路径已删；R5 一次绑定/R6-02 绑定失败保守语义保留（状态机 1/1）。生成侧 api_gen.rs 非 NotFound 读错误 `return Err` 拒写收据，旧 `unwrap_or_else(|_| absent)` 已删
- R9-02 闭合确证（major）：三文件 hunk **token 级核证**——lib.rs/engine.rs 去空白 token 流与父提交逐字节一致；renderer.rs 唯一差异=`catch_unwind(...,)` 尾逗号移除（AST 等价）；`rustfmt --edition 2021 --check`（1.9.0-stable）五触面文件全部干净 exit 0，不扩大范围
- findings: 无必修。P738-R10-01（P4 观察）负例矩阵 bound(hash)×读错误(None) 无显式单测断言（语义由 `_ => false` 单臂覆盖）+「恢复可读收敛」无专门测试（机制=读路径纯函数零状态）；P738-R10-02（P4 观察）真实 0 字节 Cargo.lock 归 absent 哨兵（语义可辩护、非 fail-open、cargo 实际不产空 lock）；P738-R10-03（流程记录）work 交接以「合同面一致」省略 R9-I 明确要求的服务链重跑——省略理由覆盖面不含 rust_ui.rs lock 门（服务链 ready 消费路径），本轮补跑闭合
- evidence（全部 41b4d4be5 worktree 独立实跑，2026-10-10 串行）: classify_lock_read_fail_closed 1/1；freshness 族 2 passed（双真值表）+shell_pack 预存红（R3 起在案，机制分诊=对拍 auto-os/shell 外部仓生成物与修复触面零交集）；review738_r5_lock 1/1；api_gen 44 passed/1 ignored；`cargo t plan738` 73/73@42.5s；`cargo tv` 162/162@2.4s（**cb_web_mime PASS@1.2s**）；正式生成服务完整链 `--features test-http-e2e http_e2e_plan738 -- --ignored` **1/1@42.29s**；三 crate check exit 0 零 error（触面 hunk 零新警告，存量 warning 在 hunk 外）；worktree clean；修复 diff 新增行零 debug 残留；`git diff 19973b089 41b4d4be5 -- docs/specs/` 空（SD 九文件未扰动）；合同面（stdlib_assembly//native.rs/native_catalog.rs/native_registry.rs/stdlib/）`git diff --quiet` 逐字节一致。详见 [738-review-r10.md](reports/738-review-r10.md)
- acceptance_results: 本轮定向面全 pass；AC-06/07/08 的 R9-01 拖累解除——与 R8 全 AC/SD pass 合并生效
- independence: 全新上下文 R10 agent（用户受派 R9 必修闭合定向复审），未参与本计划任何实现/复审
- omissions/debt: P738-R8-01/02、P738-R9-03 观察维持；P738-R10-01/02/03 新登记（登记不等于批准延期）；R9-I 的 HTTP 全档未完成段（45/102 停跑）随 merge 后批量回归收口（regress 到期判定已触发：last_covered=740@2026-10-08T11:20Z）
- state: **reviewed**（current_step=7，R8 口径恢复：T-01/T-09..T-14 完成，T-02..T-08 由 Phase 3 承接）；不合入、不归档、不删 worktree；禁 tf
- next: **重启 merge**（`/auto-plan:merge PLAN-738`，授权后）——SD 沉淀已在树（adfb7a5d1）、ledger/Design33/索引、wt-guard+worktree 清理；rebase 重放基面=41b4d4be5 所在链

### 独立复审 R9-II（2026-10-10，needs_fix 确认 → executing；merge 修复轮定向·全新上下文）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3（验收合同不变）
- outcome: **needs_fix（确认并发 R9-I 判决）**——本轮受派为 merge 修复轮定向复审（全新上下文）；定向面（修复机制/门禁/SD）**全 pass**，但并发 [R9-I 记录](reports/738-review-r9.md)（早期修订上下文会话，先落盘）的 P738-R9-01 经本轮**独立代码级核证成立**，按「不能以未验证状态放行」红线（R5-01/R6-02 同源原则）不得 restart merge——合并裁定=needs_fix，与 R9-I 一致
- reviewed_commit: `19973b0899f27267db15a39669c124579f41a505`（worktree 入场/结束 clean，只读复审）；R8 reviewed `3713337d9`/SD `641f66a5f`（旧链）；rebase 基面 `1be1783fea`（=master tip 减 2 个 doc 提交，代码等价）；R9-I main 入场 `c212e1c7a`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动，clean 复核）
- 修复机制核证（verify don't trust，全部代码级实证，独立于 R9-I）：
  - 主链块删除：lib.rs `execute_autovm_with_deadline` L1954-1959 旧 re-merge 块已删、注释在案；reviewed 树 3713337d9/641f66a5f 同位置**无该块**（仅 pkg 入口一处）；master 至今仍带（L1752-1755）——rebase 冲突面回归来源确证；pkg 入口（`create_vm_from_abt` L6030-33）保留=与 reviewed 合同一致
  - production() 在位：engine.rs `AutoVM::new` L747-756=production()+CFFI merge、`new_with_capture` 委托；native.rs production()=三件套+末尾 `declare_contract_identity("auto.http.get",...)`（L270+）
  - 撤销语义闭环：native.rs `merge()` L604-617 对 other 每个 shim 先 `contracts.remove`、末尾仅 extend other.contracts；`register_static` L356 显式 remove 不补写；`register_stdlib_ffi` 经 `register_shim_by_name`（stdlib.rs L9544）零契约——与 R2 既有测试 `final_merge_invalidates_cached_signature_proof`（reference.rs L312-319）语义一致；cb_web_mime 假红因果链代码级成立
  - 语义面定位：`git diff 641f66a5f..19973b089` 全树 79 文件 +13029/−299=master 30 提交引入（746 deadline/747 panic 打印/749 display）+修复；738 合同面文件（stdlib_assembly/native.rs/native_catalog.rs/native_registry.rs/stdlib/**）两树**逐字节零差异**，codegen.rs 仅 +34 行=master PLAN-746 print 关键字拒绝——修复未扰动任何 738 合同面
- P738-R9-01 独立核证（成立，必修补强证据）：rust_ui.rs L4025-28 `.ok().or_else(absent)` 任何读错误→absent；recorded="absent"（未绑定收据）×读取错误→absent/absent=**fresh 放行**——注释 L4029-32「读取失败均陈旧」仅对已绑定收据成立，未绑定面 fail-open；生成侧 api_gen.rs L1071-74 `unwrap_or_else(|_| absent)` 同吞非 NotFound 错误；**该缺陷为 reviewed 树既有非修复引入**（`git diff 641f66a5f..19973b089 -- rust_ui.rs api_gen.rs`=0 行，L4028/L1074 两树同源）——R5-R8 状态机负例未覆盖非 NotFound 读错误故漏网；R9-I probe 实测（is_fresh=true 反例）+本轮代码级复核双证；修复口径同意 R9-I：两端仅 NotFound→absent，其余错误传播生成失败/判陈旧，补 unbound/bound × NotFound/其他负例+恢复可读收敛证明
- P738-R9-02 独立核证（成立，附归因）：四点实在（lib.rs:119 注释缩进/renderer.rs:24761/engine.rs:9840+9852，rustfmt 1.9.0-stable 同版本复现）；归因补充=四点均为 master 继承行（master 提交态 engine.rs 同检查 177 处、全树 15028 处；仓 CI 格式门仅 auto-lsp/a2r-std——auto-lsp-ci.yml 注释明言全仓 9000+ 文件非 fmt-clean），系 rebase 把 master 未格式化行并入 738 Phase3 已批量格式化文件所致；修复口径同意 R9-I：仅格式化该四点新提交重验，不扩大范围
- acceptance_results: 本轮定向面 pass（修复机制/门禁/SD/tt 基线归因）；整体维持 R9-I 判定 AC-06 fail/AC-07 partial/AC-08 fail（R9-01 拖累）——R9-I AC 表本轮无异议
- findings（新增观察）: P738-R9-03（P4 观察登记）：tt 基线组成本轮漂移全归因——musk×6→×1（master 30 提交修复 5 个）、plan730 本轮绿、p748_t00×2 为 master 新增（本轮主检出同红独立实证+归档计划 748 §T-00 在案=双证）、plan484_024 flake 本轮红（R9-I 轮绿，轮换同册）；plan707_client_manual 确定性分裂（worktree 3/3 红 vs 主检出 3/3 绿）=链上预存非 rebase/修复引入（触发输入 stdlib_assembly/native.rs/stdlib/auto/http.at 两树逐字节一致+R8 名册在案；master 绿=无 738 严格门，HTTPStream.close #2244 无契约=链上已册红面）——后续基线名册以本轮为准
- evidence（全部 19973b089 worktree 独立实跑，与 R9-I 双份复现）: `cargo tv` 162/162（**cb_web_mime PASS@1.218s**）；`cargo t plan738` 73/73；plan724 5/5；CLI stdlib 10/10；api_gen 44+1 ignored；三 crate check 零 error（lib.rs 强制重编 0 警告）；tt 全档 5595 run/5584 pass/11 红全归因**非基线红=0**（7 红在 R8 名册+2 红 p748_t00×2 主检出同红+2 flake 隔离绿）；服务完整链 `--features test-http-e2e http_e2e_plan738 -- --ignored` 1/1@42.45s；SD 九文件 reviewed↔fixed `git diff --quiet` 全 IDENTICAL（修复提交零 docs 触面）；修复 hunk/提交面零 eprintln!/dbg! 新增（lib.rs 内 eprintln 均既有行 rustfmt 重排）
- independence: 全新上下文 R9-II agent（用户受派 merge 修复轮定向复审），未参与本计划任何实现/复审；R9-I 为并发早期修订上下文记录（commit `85d111ddb`），两轮同 commit 独立到达一致 needs_fix 结论——交叉验证而非重复
- omissions/debt: P738-R8-01/02 观察维持；P738-R9-03 观察登记；R9-I 的 HTTP 全档未完成段（45/102 停跑）随下轮收口
- state: **executing**（R9-I 已重开 T-12/13/14、current_step=4，本轮维持）；不合入、不归档、不删 worktree；禁 tf
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复 P738-R9-01（lock 读错误归一 NotFound-only 双端+负例/收敛证明）与 P738-R9-02（四点格式化）——**修复面窄：rust_ui.rs/api_gen.rs lock 路径+4 格式点，不动 VM/契约面（本轮已证两树合同面逐字节一致，修复无需重跑 tv/契约门，按触面跑 tt/api_gen/服务链+格式门）**，新提交后独立复审（R10），pass 才 restart merge；merge 收据维持 needs_fix

### 补充复审 R9（2026-10-10，needs_fix → executing）

- stage: review | plan_id: PLAN-738 | plan_revision: 3 | outcome: **needs_fix**
- reviewed_commit: `19973b0899f27267db15a39669c124579f41a505`（入场/结束实现树 clean）；base_commit: 修复父 `adfb7a5d1`，重放基面 `1be1783fe`；历史 R8 树 `3713337d9`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未改）
- spec_inputs: SD-01..07 冻结草稿与 WT 待合入七 canonical + 新 assembly-manifest 文档；路径/版本/hash 入 [R9 baseline](reports/738-review-r9-baseline.json)。SD-05 原规则保留，读取失败拒绝的实现验收 not approved；不弱化合同，不增 revision。
- acceptance_results: AC-01..05 pass / AC-06 fail / AC-07 partial / AC-08 fail；逐 AC→任务→证据见 [738-review-r9.md](reports/738-review-r9.md)
- findings: **P738-R9-01（P2）** 未绑定 lock 时把 PermissionDenied 等读取错误归一 absent，实际新鲜度门误返回 true；生成侧也吞错误。两端仅允许 NotFound=absent，其他错误应生成失败/判陈旧，补 unbound/bound 状态负例。**P738-R9-02（P2）** 格式门 exit1：lib.rs:119、renderer.rs:24761、engine.rs:9840/9852；修复并以实际新提交重验。
- evidence: [R9 报告](reports/738-review-r9.md) / [可复现探针](reports/738-review-r9-probe.py)；主链冗余 merge 修复成立，R5–R7 核心修复重放补丁相等；完整日常/转译档、三目标/C stdio/Rust host 见证、正式生成服务 1/1@48.95s 本轮实跑；最终数字与日志 hash 见报告/baseline。
- independence: 本上下文参与早期计划修订/R5 补充复审，未参与本次实现或 R8 复审；不冒称全新独立上下文。临时 cfg(test) 探针 finally 精确恢复，未修生产代码。
- omissions/debt: 两项为本期必修，不转 D3b；R8 非阻塞观察与 748 探针 harness 基线保留。历史通过记录不覆盖 R9 新反例。
- state: **executing**，重开 T-12（lock/读失败/报告）、T-13（SD/AC 对账）、T-14（格式健康/最终证据/复审）。current_step=4（T-01/T-09/T-10/T-11）；T-02..08 历史未勾选保持。
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复两项并提交，补负例/真实生成服务与最终证据，再交独立 review；不合入/归档/删 worktree。

### 独立复审 R8（2026-10-10，pass → reviewed）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3（验收合同不变；本轮闭合 R7 必修项，不增 revision）
- outcome: **pass**（R7-01/R7-02 闭合确证；R7-03 裁量与 R7-04 登记落地；全分母矩阵与门禁独立复现；两项 P3 观察登记）
- reviewed_commit: `3713337d97f37d68bddaca22ce16fb152cfc8db5`（worktree 入场 clean，只读复审，结束 clean）；修复链 diff `6c3943327..3713337d9` = 恰 3 文件 +265/−53（host/tests/trans rust），与声称触面一致，R3–R7 已验面外未被扰动
- base_commit: R7 reviewed `6c3943327`；Phase 3 diff base `2c1b4a763`；merge-base `6d69dbdc7`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动）
- spec_inputs: 沿 R7 基线七 canonical + SD 终稿；本轮 **approved**——SD-04 重锚描述（四形态/适配契约族/Null 哨兵/parser 拼写边界/真漂移面）与 3713337d9 行为逐条一致，reference-audit #11 同步相符
- acceptance_results: AC-01..08 全 pass（R7 拖累项解除：AC-03 has_key/mod 漂移检出闭合；AC-05/07/08 SD-04/audit 与行为再一致）
- findings: 无必修。P738-R8-01（P3 观察）链式核心调用面（`json.parse(s).len()`）由 R5 在案「记边界跳过」收紧为诚实拒绝——合同正确（R5「不能仅写债务保留放行」），全仓资产零命中；残留=形态守卫注释过时（rust.rs L5613-5617 仍写「跳过并记边界」）+无测试钉死+SD/audit 未点名该子面，建议 merge 顺手处理。P738-R8-02（P3 观察）str 实参 `json.has_key(s,"k")` 被 PROVIDER_CLAIM_NO_CALLEE 门拒（has_key_str 无公共面）——新增 has_key_str BoolToIntIf 规则在该面不可达（防御性），fail-closed 非回归
- evidence: [738-review-r8.md](reports/738-review-r8.md)（R7-01 四级证据/12×2 矩阵表/门禁全表/SD 对照）；CLI 实测全用 3713337d9 构建的真实 auto.exe+隔离 AUTO_STDLIB_ROOT；门禁：plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44（1 ignored）、tv 162/162、tt 5558 run/14 红全基线（plan730 固定端口族环境红隔离复跑绿，plan484_024 本轮绿——flake 族轮换同册；非基线红=0）、三 crate check 零 error（触面文件零新警告）、debug 残留 0、服务完整链 1/1@25.84s
- independence: 全新上下文 R8 agent，未参与本计划任何实现/复审
- omissions/debt: P738-R8-01/02 为观察项（登记不等于批准延期）；P738-D2 legacy 边界维持
- state: **reviewed**（current_step=7：T-01/T-09..T-14 完成；T-02..T-08 按 Phase 3 映射由 T-09..T-14 承接，R4 口径保持）；不合入、不归档、不删 worktree；禁 tf（批量回归归 `/auto-plan:regress` 到期判定）
- next: `/auto-plan:merge PLAN-738`（授权后）——canonical/ledger 沉淀、Design33/索引、wt-guard+worktree 清理；P738-R8-01 过时注释/链式面测试建议随 merge 顺手处理或转 D3b 清单

  [✅ 已完成·R7 修复重闭] 最终提交 `3713337d9`：plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44、tv 162/162、tt 非基线红=0、服务链 1/1@109.5s。

### 独立复审 R7（2026-10-10，needs_fix → executing）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3（验收合同不变；本次发现缺口不增 revision）
- outcome: **needs_fix**（R6-01 矩阵面/R6-02/R6-03 闭合确证；R6-01「泛化到全部公共方法」未完全闭合——P738-R7-01 必修 + P738-R7-02 必修）
- reviewed_commit: `6c3943327dab67bce3da2ed7e9779728b54935f0`（worktree 入场 clean，只读复审，结束 clean）；修复链 diff `68398d2d6..6c3943327` = 恰 4 文件 155+/27−（host/tests/trans rust/rust_ui），与声称触面一致，R3/R4 pass 面外未被扰动
- base_commit: R6 reviewed `68398d2d6`；Phase 3 diff base `2c1b4a763`；merge-base `6d69dbdc7`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动）
- spec_inputs: 沿 R6 基线七 canonical + SD 终稿；本轮 **not approved**——SD-04「未证明=SIGNATURE_UNVERIFIED 不产出成功产物」与 audit #11「12 方法分母归属三形态验证+第四拒绝」被 P738-R7-01 反例证伪（has_key/mod 漂移 exit0 产出成功产物），须随修复再锚或如实登记
- acceptance_results: AC-01/02/04/06 pass（本轮确证+门禁复现）；AC-03 fail（has_key/mod 漂移不检出）；AC-05/07/08 受 R7-01 拖累
- findings:
  - **P738-R7-01（P1 必修）**：`json.has_key(v,"k")`（JsonValue.has_key=12 分母成员）模块限定拼写——接管发射臂（rust.rs L10185）拆段写产物括号不平衡（`if a2r_std::json::has_key(&v,"k")) {...}`，trans 层 warning 在案）→ 形态守卫按拆段片段跳过（shape① 循环 return 使 case④ 不可达）→ 基线与**漂移（int→str）均 exit0 无方法 proof**（manifest refs 仅 parse）——P738-R6-01 原始缺陷形状在分母成员的模块拼写格存活，违反 R5「不能仅写债务保留放行」红线；R6 交接教训「分母 12 个必须泛化验收」未兑现到矩阵（keys/as_int/is_null 之外未验）。
  - **P738-R7-02（P2 必修）**：case④ 误伤——`let json="abc"; print(json.len())`（str 变量 shadowing 模块名）exit1 UNVERIFIED 误拒（68398d2d6 基线 exit0，R6 引入回归）；用户本地 `json` 模块（mod json+use json）调用同被拒。根因=case④ 只看 owner 拼写 json/Json+公共面存在，不复查 L5706-5708 的 local_var_types/local_modules/sibling_modules 排除（L5732 落穿无条件到达）。违反 R5-02「同名用户符号不误拒」原则。
  - P738-R7-03（观察）：shape③ 等效形式构造丢实参——get/get_at/has_key receiver 直发恒报「emitted arity」误诊（真分歧在返回类型 ?JsonValue/JsonValue、int/bool；exit1 结果恰与真漂移面一致）。
  - P738-R7-04（观察）：合法模块拼写 fail-closed 拒绝面登记（json.len 有臂无 ScalarCast 契约建议补；as_number/as_array/type 模块拼写无臂 case④ 拒）；get_u64/to_string 无 proof=P738-D2 既有边界非新缺口。
- evidence: [738-review-r7.md](reports/738-review-r7.md)（12 格矩阵+12 方法双拼写全分母普查表/门禁全表/SD 对照）；CLI 实测全用 6c3943327 构建的真实 auto.exe+隔离 AUTO_STDLIB_ROOT；门禁：plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44、tt 5558 run/14 红全基线（非基线红=0，plan498 绿）、tv 162/162、freshness 双真值表+shell_pack 预存红、三 crate check 零 error、服务链 1/1@31.45s
- independence: 全新上下文 R7 agent，未参与本计划任何实现/复审
- omissions/debt: R7-01/R7-02 见 findings；R7-03/R7-04 为观察项可随裁量处理；登记不等于批准延期
- state: executing，T-11（R7-01 漏拒+R7-02 误伤）/T-13（SD-04、audit #11 再锚）/T-14（新提交重跑门禁+分母全量 CLI 验收）重开；T-01/T-09/T-10/T-12 维持闭合；current_step=4（R7 口径）
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复 P738-R7-01（发射臂平衡/守卫区分完整 FQN 调用与拆段片段，矩阵与 CLI 验收扩到 12 方法双拼写）与 P738-R7-02（case④ 复查 local 排除+shadowing 反例测试），顺带裁量 R7-03/R7-04，重跑受影响门禁，SD-04/audit 再锚，新最终提交后再独立 review；不合入/不归档/不删 worktree；禁跑 tf

  [✅ 已完成·R6 修复重闭] 最终提交 `6c3943327`：plan738 73/73（含矩阵测试）、plan724 5/5、CLI 10/10、api_gen 44/44、双真值表+lock 状态机 1/1、tt 非基线红=0、tv 162/162、服务完整链 1/1@53.5s；CLI 级 R6 矩阵实测（keys/as_int/is_null 双拼写漂移全 exit1）。[R7 重开 2026-10-10] R7 实测（真实 auto.exe，全分母普查）：矩阵三兄弟闭合确证，但 `json.has_key(v,"k")` 漂移后仍 exit0 无 proof（P738-R7-01，拆段写+守卫跳过）、case④ 误伤 `json` 同名局部变量/本地模块（P738-R7-02）——本任务须随新修复提交重跑分母全量 CLI 验收。

### 独立复审 R6（2026-10-10，needs_fix → executing）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3（验收合同不变；本次发现缺口不增 revision）
- outcome: **needs_fix**（R5-01/R5-03 闭合确证；R5-02 部分闭合——receiver 公共方法面仅 len 形态入门，P738-R6-01 必修）
- reviewed_commit: `68398d2d61a6e93a908a01e1442c8dff65983204`（worktree 入场 clean，只读复审，结束 clean）；修复链 `9c255993b`→`bc0b95cfd`→`796d279a4`→`68398d2d6` diff = 恰 6 文件（host/manifest/tests/trans rust/api_gen/rust_ui），与声称触面一致，R3/R4 pass 面外未被扰动
- base_commit: R5 reviewed `9c255993b`；Phase 3 diff base `2c1b4a763`；merge-base `6d69dbdc7`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动）
- spec_inputs: 沿 R5 基线七 canonical + SD 终稿；本轮 **not approved**——SD-04 receiver 全路径/未证明即拒绝措辞与实际行为不符（详见 R6-01），须随修复重锚或如实缩面
- acceptance_results: AC-01/02/04/06 pass（R5-01/R5-03 闭合确证+未触面沿用）；AC-03 fail / AC-05/07/08 受 P738-R6-01 拖累
- findings:
  - **P738-R6-01（P1 必修）**：receiver 公共方法面仅 `len`（+value_len 扁平 helper）入 strict 门——真实 CLI 实测（隔离 AUTO_STDLIB_ROOT+真实 auto.exe）：`JsonValue.keys` []str→[]int 漂移后 `v.keys()`（逐字发射）与 `json.keys(v)`（FQN 发射）均仍 exit 0 无 proof；`v.is_null()`（可编译 serde 固有方法旁路，producer bool vs 公共 int）int→str 漂移 exit 0；`v.as_int()` 无 proof。根因=shape③ len-only（`if method == "len"`）、模块限定方法裸名公共面未命中即 return 不落穿 `public_method_symbol`、shape② value_to_int 腿死码（to_int 无别名映射）；SD-04/T-11 重闭/交接「公共方法 receiver 调用进 strict 门……未证明=SIGNATURE_UNVERIFIED」过度声明；reference-audit 未随 R5-02 更新。R5 修复合同「对未经证明的被引用核心方法拒绝，不能仅写债务保留放行」未全兑现。
  - P738-R6-02（观察）：`bind_workspace_lock` 忽略收据写错误——持续写失败时 absent→fresh 可在漂移内容上重复触发（仅退化 FS 角落）。
  - P738-R6-03（观察）：tt 全档 `plan498_bar_group_emphasis` 为未在册 flake（隔离复跑绿），建议入 verification §2 flake 族名单。
- evidence: [738-review-r6.md](reports/738-review-r6.md)（CLI 反例四形状表/门禁全表/SD 对照）；R5-01 状态机测试 1/1 复现+真值表独立运行；R5-03 `t12_manifest_identity` 2/2+`generation_consumer_identity` 1/1（零 clone、Result 断言、Embedded/Standalone 差异钉死、C fixture 差异在案）；门禁：plan738 72/72、plan724 5/5、CLI 10/10、api_gen 44/44、tv 162/162、tt 非基线红=0（plan498 flake 隔离绿）、裸 t 17 红全基线/flake、三 crate check 零 error
- independence: 全新上下文 R6 agent，未参与本计划任何实现/复审
- omissions/debt: R5-01/R5-03 无遗留；R5-02 残余面=11/12 JsonValue 公共方法 receiver 形态（含 1 个可编译旁路），登记不等于批准延期
- state: executing，T-11（R5-02 receiver 面）/T-13（SD-04）/T-14（最终收据随新提交重跑）重开；T-01/T-09/T-10/T-12 维持闭合；current_step=4（R6 口径）
- next: `/auto-plan:work PLAN-738` 修复 P738-R6-01（顺带裁量 R6-02/R6-03），重跑受影响门禁+CLI 反例，更新 SD-04/reference-audit，新最终提交后再独立 review；不合入/不归档/不删 worktree

  [✅ 已完成·R5 修复重闭] 最终提交 `68398d2d6`：plan738 72/72（含 R5 新 5 测）、plan724 5/5、CLI 10/10、api_gen 44/44、freshness 双真值表+状态机 1/1、tt 非基线红=0、服务完整链 1/1@64.7s（lock 绑定语义下）、裸 t/tv 收据见 verification 更新。

### 补充复审 R5（2026-10-10，needs_fix → work）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3（验收合同未变，本次发现缺口与回退进度不增 revision）
- outcome: **needs_fix**（两个实际反例 + 一项消费者验收未完成；原 R3/R4 pass 记录保留，但不再代表当前整体满足合同）
- reviewed_commit: `9c255993b2a3163740cf886349013c7bea6aae32`，入场 clean，临时审查测试 exact bytes 恢复后 clean；本轮零最终实现 diff。
- base_commit: Phase 3 diff base=`2c1b4a763280b3994f51b441714b4f2120328fb3`；merge-base=`6d69dbdc78d99f23ba90a37c9559f7fb54dd7be3`；observed_master=`49f17c5362591e63b5f42040602028c852201fb4`。
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；Cargo/Spec/SD 与 CLI 文件身份、日志 hash 见 [738-review-r5-baseline.json](reports/738-review-r5-baseline.json)。
- spec_inputs: frontmatter 七 canonical + Specs overview；SD 冻结版本=`1b05d3d441830e855f0eae837a813b9820cd83e5:docs/plans/reports/738-sd-drafts.md`，SHA256=`72558019bad702e9ae411ee047c011643c1689fb06a9dab1ff61318b98cfdf19`，本次 **not approved**；影响路径/GOAL-003 保留，未发布 canonical/ledger。
- acceptance_results: AC-01 pass（清点面） / AC-02 partial / AC-03 fail / AC-04 partial / AC-05 partial / AC-06 fail / AC-07 fail / AC-08 fail；逐项映射见 [738-review-r5.md](reports/738-review-r5.md)。
- findings: P738-R5-01（P1：recorded lock=absent 对后续任意 lock 内容仍 fresh）；P738-R5-02（P1：公开 JsonValue.len 返回类型漂移，实际 CLI --actual --check 仍 exit 0，仅有 parse proof）；P738-R5-03（P2：生成消费者 clone CLI manifest，丢弃转译 Result，缺普通/persistent VM 与 C 真入口共同身份对拍）。这些属于原 AC，登记不等于批准转 D3b。
- evidence: [复审报告](reports/738-review-r5.md) + [可重跑 lock 反例脚本](reports/738-review-r5-probe.py) + baseline JSON；临时探针调用正式 generate_api 和实际 backend_generation_is_fresh，期望拒绝后续依赖版本改变而实际 fresh=true，负面断言失败。receiver 反例使用隔离 stdlib 根与实际 CLI，编入 CLI 的 host/collector FNV 与 reviewed 源码 bytes 一致。
- gates: 本轮 plan738=69/69；裸 t=5183 run/5167 pass/16 既有红族；tv=162/162；tt=5554 run/5540 pass/14 红（13 同日常红+plan484_024 已档 flake，隔离复跑 2/2）；三 crate check 零 error；8 触面 rustfmt 与 diff check 通过。原 R3/R4 HTTP/服务/witness/API/freshness 正例按实现/依赖未变显式复用，本轮未重复这些正例，也不以其替代本轮新反例。tf/taa/tu 不触发。
- independence: 本会话参与过早期实现和 R3 计划修订，未参与 Phase 3 实现；本次为从源码/反例重构的补充复查，不冒称新的完全独立上下文。后续最终 pass 仍须独立上下文审查修复提交。
- omissions/debt: 六核心 receiver 路径不是已批准 D3b；T-12 两个局部身份测试不是全消费者验收；SD-04/05 与验收对账须随修复纠正，acceptance.md 的 ADAPTER_RULES 17→实际 15 勘误亦未同步。
- state: executing，T-11..T-14 重开；T-01/T-09/T-10 保留完成，原 T-02..T-08 未勾选，current_step 按实际任务标记重算为 3/14；代码、正例和历史进度不删除。
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复 R5-01/02、补 R5-03 真实消费者验收，对齐 SD/AC 报告，新最终提交重跑受影响门禁与正式服务验收，再独立 review；本轮不修实现、不合入、不归档、不清理 worktree。

### 独立复审 R4（2026-10-10，pass → reviewed）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3
- outcome: **pass**（R3 两项 findings 修复确证、受影响门禁全绿、无新引入问题；R3 已对全部 AC/SD 实现面 pass，R4 pass 即整体 reviewed）
- reviewed_commit: `9c255993b2a3163740cf886349013c7bea6aae32`（worktree clean，复审只读，结束仍 clean）
- base_commit: R3 reviewed `b3a4d660e`（R4 diff 基准）；Phase 3 diff base `2c1b4a763`；merge-base(master, plan-738-dev)=`6d69dbdc7`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动，clean 复核）
- spec_inputs: 沿用 R3 基线（frontmatter 七 canonical + SD-01..07 终稿）——R3 修复只动测试属性与文档勘误、无 SD 语义变化（diff 证明），R3 spec_inputs 不失效
- R3-01 闭合确证（major）:
  - `git diff b3a4d660e..9c255993b` = 仅 `crates/auto-man/src/rust_ui.rs` 2+/2−（`#[test]`+doc 从 `lock_freshness_truth_table` 前移回 `assembly_freshness_truth_table` 前）
  - 属性结构正确：两测试各带独立 doc + 单个 `#[test]`（rust_ui.rs L5019-5023 / L5035-5037），无重复属性、无死代码
  - `cargo check -p auto-man --tests` 中 assembly/lock truth_table never-used warning=0（grep 零匹配）
  - `cargo test -p auto-man --lib freshness -- --test-threads=1` = 3 测：`assembly_freshness_truth_table ... ok`、`lock_freshness_truth_table ... ok`（**两个不同测试名各自独立运行**）+ `test_shell_pack_lib_freshness FAILED`（R3 在案环境红，同名同 panic 复现，非回归）
- R3-02 闭合确证（minor）: ADAPTER_RULES 逐提交 awk 实测 = c1579ed71:12、84bec29ef/b6cfc6df1/b3a4d660e/9c255993b:15；勘误回写三处核对——strict.md「规则表 12 条（初稿误记 14）」、reference-audit.md「增补至 15 条（初稿误记 17）」、reference-audit.json `"adapter_rules_total": 15`；本计划 T-10/T-11 勘误注记在案
- acceptance_results: AC-01..08 / SD-01..07 维持 R3 pass——修复 diff 全部位于 `#[cfg(test)] mod tests`（纯测试属性移动，零生产语义变更，diff 证明），R3 各 AC 结论未被扰动；R3-03/04 非阻塞观察在案（伴生模块新鲜度面=681/734 既有边界、T-09 分类命名不精确但结论成立），无需本轮处理
- findings: **P738-R4-01（观察，非阻塞，非 738 回归）**：`cargo check -p auto-man --tests` 的集成测试目标 `crates/auto-man/tests/plan734_commands_fixture.rs` 编译失败（E0433 `api::echo` 无 `mod api` 声明等 5 错）——Plan 734 提交 `036312096` 引入、系 merge-base `6d69dbdc7` 祖先（master 预存）；738 全程 diff 未触 `crates/auto-man/tests/`（实测空 diff），738 门禁均编译 lib 测试目标不受影响（R3 三 crate check 不含 --tests 亦未触）。建议路由 plan734 归属面 L0 修复，不阻塞 738
- evidence（全部 9c255993b worktree 实跑，2026-10-10 串行）: 三 crate check（`cargo check -p auto-lang -p auto-man -p auto`）零 error；`cargo test -p auto-man --lib api_gen -- --test-threads=1` 43 passed/1 ignored/0 failed；`cargo t plan724` 5/5；`cargo t plan738` 69/69（39.9s）；`cargo tv` 162/162；rustfmt `--edition 2021 --check` rust_ui.rs 干净；freshness 族见上。未跑 tf（批量回归档非 per-plan 门禁）/taa/tu/t3/docs_gen（未触面）
- state: **reviewed**（current_step 14/14；不合入、不归档、不删 worktree）
- next: merge（待授权）——`/auto-plan:merge` 沉淀 canonical/ledger/Design33/索引、归档、wt-guard 清理；P738-R4-01 观察项随 merge 收尾路由

### work 修复交接（2026-10-10，R3 needs_fix 修复 → execution_done）

- stage: work
- plan_id: PLAN-738
- plan_revision: 3
- outcome: pass（R3 两项修复完成；next=R4 独立复审）
- code_commit: worktree plan-738-dev `b3a4d660e` → `9c255993b`（1 file +2/−2：`assembly_freshness_truth_table` 的 `#[test]` 属性复位——T-12 插入 lock 真值表时属性被截走致其变死代码；提交后 clean）
- task_ids: R3 重开的 T-12(第4条)/T-14(第1/2条) 全部重闭；current_step 14
- evidence: 双真值表独立运行绿（freshness 族：assembly ✓ lock ✓ + shell_pack 环境红在案）；api_gen 43/43；`cargo check -p auto-man --tests` 的 assembly/lock never-used warning 基线 1→0（预存 95 项 dead-code 与本面无关）；rustfmt clean；R3-02 勘误回写 738-phase3-strict.md/reference-audit.{md,json}/本计划（ADAPTER_RULES 12@T-10、15@最终）
- blockers: 无
- next: `/auto-plan:review PLAN-738`（R4）——独立 agent 复核 `9c255993b` 修复与重跑门禁；仍不合入/不归档/不删 worktree

### 独立复审 R3（2026-10-09，needs_fix）

- stage: review
- plan_id: PLAN-738
- plan_revision: 3
- outcome: **needs_fix**（一处必修 R3-01 + 一处勘误 R3-02；全部 AC/SD 实质验证通过，门禁零未解释新红）
- reviewed_commit: `b3a4d660e722cf9f1a81bb45bac9982548ce09c9`（worktree clean，复审只读；结束仍 clean）
- base_commit: Phase 3 diff base `2c1b4a763`；merge-base(master, plan-738-dev)=`6d69dbdc7`（合入面比较基面）
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动）
- spec_inputs: frontmatter 七个 canonical Spec + [738-sd-drafts.md](reports/738-sd-drafts.md) SD-01..07 终稿（T-13 版，绑定 c1579ed71→b6cfc6df1；T-14 两提交未改 SD 声明的行为面——lock 物化语义与 workspace_lock 补位均含于 SD-05 终稿语义内）
- acceptance_results（以工件重构，R3 独立复跑全部在 b3a4d660e）:
  - **AC-01 pass**：inventory 115 分母/partial 退出码 3 CLI 实测；566 差异分类独立抽查（计数 566=552+4+2+8 ✓，4 文件 rustfmt 复验逐字节 MATCH，lib.rs 重排语义等价）
  - **AC-02 pass**：AssemblyTarget 驱动选层（历史）+ ADAPTER_RULES 四源对拍（syn 真解析，R3 通读 verify_adapted_reference/check_public_projection/verify_plain_reference）+ 引用闭包五路径收集器（trans/rust.rs 通读）+ back 模块 Embedded 证明；⑤腿 Rust 实编 witness 1/1、C stdio MSVC witness 1/1 复跑
  - **AC-03 pass**：无契约非 Plain 形状→SIGNATURE_UNVERIFIED、元数/async/cast/结构漂移→SIGNATURE_DRIFT（代码通读+plan738_host 10/10+plan724 改判探针独立证实：a2r-std crate post=2 参 vs 内嵌 post=3 参）
  - **AC-04 pass**：use_semantics 7/7；同名用户 fn 遮蔽/wildcard 唯一归属/歧义拒绝/限定拼写指引测试在案（t11 5 测）
  - **AC-05 pass**：六模块矩阵在案；Standalone/Embedded 分家实证；C ext TARGET_UNSUPPORTED（T-09 分类核对）
  - **AC-06 pass（附 R3-01）**：双指纹投影只剔除 consumer 名与 consumer_input 源（manifest.rs 通读）+ t12 三角对拍 2/2 复跑 + lock 真值表 + 服务链"改 stdlib→陈旧→再生"1/1@42.26s 复跑；**但 T-06 的 assembly_freshness 真值表测试自 b6cfc6df1 起不再运行（R3-01），回归保护缺口须修复**
  - **AC-07 pass**：CLI 三模式退出码 3/1/2 冒烟实测+EXIT_OK 断言在案；服务链 1/1（schema4+workspace_lock 收据→ready=consumer_fingerprint→业务 42→再生→复验）
  - **AC-08 pass（附 R3-02 勘误）**：裸 t 17 红全分诊（13 master 预存+4 flake，plan484_024 隔离绿）、tv 162/162、tt 非基线红=0、th 缩减档与执行报告分诊同构（back_proxy 基线红、plan730 族环境红——执行者基线 worktree 复现在案+738 diff 不触 plan730+738 服务链同环境绿+本机 musk-100 长驻服务佐证）；SD-01..07 终稿与最终实现逐条对照一致（退出码 0/1/2/3=EXIT_OK/CHECK_FAILED/ERROR/INVENTORY_PARTIAL 实证）
- findings:
  - **P738-R3-01（major，必修，影响 T-06/T-12/T-14，AC-06 回归保护）**：`crates/auto-man/src/rust_ui.rs::tests::assembly_freshness_truth_table` 自 `b6cfc6df1`（T-12）起丢失 `#[test]` 属性——新测试 `lock_freshness_truth_table` 插入位置截走了原属性并自带重复 `#[test]`（被注册两次）。后果：①T-06 交付的真值表测试现为死代码（`cargo test -p auto-man --lib freshness` 编译输出 `warning: function assembly_freshness_truth_table is never used`——违反 AGENTS §3 零未处理 warning 健康门）；②执行报告"freshness 2/2"实为同一测试重复跑两遍的假象（真实构成=lock_freshness_truth_table×2+shell_pack 环境红）；③T-06 新鲜度真值表回归保护丢失（`assembly_freshness` 函数本体未改、生产行为未坏——服务链 1/1 实证行为正确）。修复：恢复属性位置（一行），在最终提交重跑 freshness 族+健康扫描并勘误报告。
  - **P738-R3-02（minor，勘误，影响 T-10/T-11 报告）**：ADAPTER_RULES 计数失真——T-10 报告"规则表 14 条"（c1579ed71 实测 12 条）、T-11 报告"增至 17 条"（84bec29ef 起实测 15 条）。契约本体真实且被 plan738_host 10/10 覆盖，纯计数错误；随 R3-01 修复一并勘误。
  - P738-R3-03（观察，非阻塞，非 738 回归）：伴生 back 模块（fsys.at 等）逐字内容不在任何逐字新鲜度面（generation_source_snapshot 只含 api.at+db.at，PLAN-681/734 既有边界）；738 经 Embedded 引用证明实际扩大了可检测面（引用面变化→共同身份变化→再生）。
  - P738-R3-04（观察，非阻塞）：T-09 的"comment_or_whitespace_only"4 文件实际含 use/mod 语句重排（Rust 顶层声明顺序语义无关）——分类命名不精确，"非语义变更"结论成立（R3 独立复核）。
- evidence: [738-review-r3.md](reports/738-review-r3.md)（完整命令/输出摘要/探针记录）；关键反证命令：`git log -L 5017,5045:crates/auto-man/src/rust_ui.rs`（属性剥离归因 b6cfc6df1）、`cargo test -p auto-man --lib freshness -- --test-threads=1`（dead_code warning+重复注册输出）、逐提交 ADAPTER_RULES awk 计数
- gates（R3 独立复跑，全部 b3a4d660e，串行；期间 musk-100 worktree 有长驻 `cargo run` 服务、无编译竞争）：三 crate check 零 error；裸 t 5166/5183（17 红全分诊=13 预存+4 flake，plan484_024 隔离 2/2 绿）；tv 162/162；tt 5541/5554（13 红全 ⊂ 裸 t 名单）；th 缩减档 127/142 pass+2 基线红+13 级联超时（与执行报告同构）；plan738 69/69、plan738_host 10/10、t12 2/2、plan724 5/5、CLI 10/10、api_gen 43/43、双 witness 1/1、服务完整链 1/1@42.26s；触面 diff 零 dbg 残留、rustfmt --check 干净；未跑 tf/taa/tu/t3/docs_gen 专项（未触面）
- omissions/debt: P738-D1/D2 维持既有登记（KNOWN-DEBT 263/264 行在案）；无新批准延期
- state: executing，current_step=12（T-12 第 4 条、T-14 第 1/2 条重开；T-01..T-11、T-13 及历史任务记录保留）；不合入、不归档、不删 worktree
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复 P738-R3-01（恢复 `#[test]` 属性至 `assembly_freshness_truth_table`、消除 `lock_freshness_truth_table` 重复属性）+ 勘误 P738-R3-02（两份报告计数），新提交重跑 freshness 族/plan738/auto-man scoped/健康扫描（含 warning 零新增确认），更新 verification 报告后再独立 review；AC/SD 合同与阈值不变

### work 交接（2026-10-09 晚，Phase 3 T-09..T-14 完成 → execution_done）

- stage: work
- plan_id: PLAN-738
- plan_revision: 3
- outcome: pass（T-09..T-14 全部执行并验收；AC-01..08 执行面闭合；next=独立 review）
- code_commit: worktree plan-738-dev `2c1b4a763` → `b3a4d660e`（五提交：T-10 c1579ed71 / T-11 84bec29ef / T-12 b6cfc6df1 / T-14a 1a983091f / T-14b b3a4d660e；8 文件 +2224/−284；提交后 clean；auto-down 兄弟 895f8d0f 只读）
- task_ids: T-09、T-10、T-11、T-12、T-13、T-14（current_step 14/14）
- evidence: [738-phase3-scope](reports/738-phase3-scope.md)（566 差异机械分类）/ [738-phase3-strict](reports/738-phase3-strict.md)（契约注册表 strict 门）/ [738-phase3-reference-audit](reports/738-phase3-reference-audit.md)（引用闭包全路径）/ [738-phase3-manifest](reports/738-phase3-manifest.md)（双身份+lock 收据）/ [738-phase3-acceptance](reports/738-phase3-acceptance.md)（AC 对账）/ [738-phase3-verification](reports/738-phase3-verification.md)（最终门禁+服务链 1/1）；门禁要点：裸 t 5183 测 16 红全基线、tv 162/162、tt 非基线红=0、th 逐名分诊（plan730 interop/multipart 基线 worktree 同命令复现实证环境红）、plan738 69/69、服务完整链 1/1@38.6s、双 witness 1/1
- blockers: 无阻塞。要点教训：①T-12 收据补丁脚本被断言中断的缺失段是静默的——T-14 真服务链走查是发现此类「门读字段而无人写」破损的唯一可靠手段；②lock 新鲜度须区分「首次物化」（已记录依赖的确定性派生）与「依赖漂移」，否则每个新 workspace 首启误判陈旧；③固定端口 HTTP 族失败先用基线 worktree 同命令复现再归因。
- state: execution_done；不合入、不归档、不删 worktree（授权止于复审准备）
- next: `/auto-plan:review PLAN-738`——未参与实现的 agent 复核最终提交 `b3a4d660e`（全部 AC、实际 callee/manifest、完整 diff vs SD 终稿、th 环境红基线归因）

### Revision 3 / Phase 3 修订交接（2026-10-09，pass → work）

- stage: new
- plan_id: PLAN-738
- plan_revision: 3
- outcome: pass（执行合同已就绪；不是实现完成或独立复审通过）
- authorization: 用户要求更新 738、将后续步骤纳入新 phase；沿用已授权的本仓修复范围，后续由执行 agent 继续。
- baseline: `plan-738-dev@2c1b4a763280b3994f51b441714b4f2120328fb3`，入场 clean；依赖/Specs/草稿版本见 [738-phase3-baseline.json](reports/738-phase3-baseline.json)。
- changed_tasks: 新增 Phase 3 / T-09..T-14；原 T-02..T-08 的剩余验收由新 phase 对账，历史记录与未闭合项保留。total_steps=14，current_step=1。
- changed_acceptance: none（AC-01..08 与 SD-01..07 ID/范围/阈值保留）；补充 §5.8 的执行约束和 §6.4 的具体对拍/反例。
- evidence: 本轮静态源码核查发现 Rust Resolved-only strict 放行和包装 async 豁免；manifest consumer 参与指纹、引用路径与生成依赖闭包待专项证明；SD 退出码错位；566 文件差异需完整分类。现有服务 1/1 50.18s 作为 R2 执行收据保留。
- independence: 本会话参与过实现，本轮只修订计划，不能充作最终独立复审。本轮不实施、不跑测试、不改 canonical/ledger。
- state: executing；原 worktree 与已有修复保留；未合入/归档。
- next: `/auto-plan:work PLAN-738` 读取 revision 3，从 T-09 开始依次完成 T-10..14，再交独立 `/auto-plan:review`。执行结果报告必须绑定最终 revision 3 + 新代码提交。

### work 交接（2026-10-09 下午，R2 全档收口 → executing 继续）

- stage: work
- plan_id: PLAN-738
- plan_revision: 2
- outcome: pass（裸 t 全档分诊暴露的 R2 缺口按类修复完毕：生产契约回填/冗余 merge 撤销/CLOSURE 走查对齐（plan624/536 两真回归根因）/C 目标 c.* 解析与 ext 面诚实诊断/Rust 适配壳/金样更新；最终门禁+服务验收绑定最终提交全绿。T-02..T-08 整体仍打开）
- code_commit: worktree plan-738-dev `b838f5f16` → `2c1b4a763`（566 files：含 R2 会话遗留的约 530 文件 bulk rustfmt——抽查纯格式化、全部下午门禁跑在含该状态树上；提交后 clean）
- task_ids: T-02..T-08 修复推进（无勾选变更，current_step 保持 1/8）
- evidence: [738-repair-round2.md](reports/738-repair-round2.md) "R2 全档收口"节 + [738-sd-drafts.md](reports/738-sd-drafts.md)；裸 t 5171 全跑 18 红全分诊（13 master 预存今晨逐名实证+4 隔离绿 flake+1 已档 plan707）；tv 162/162；tt/th 余预存与复跑绿 flake；plan738 57/57、api_gen 43/43；三目标同源 witness（VM41/Rust42/C43）+ C stdio 直接绑定 witness（真实 MSVC 65）+ ext 面负测；服务验收 --features test-http-e2e 最终提交 1/1
- blockers: 无阻塞。要点教训：①服务验收测试门控在 test-http-e2e feature，漏 feature=0 测试假绿；②契约声明必须在 production() 全部覆盖后落位（inventory/register/merge 均撤销同 ID）；③VM 契约走查的操作数模型必须与引擎逐 opcode 对齐（CLOSURE 变长捕获描述符）
- next: manifest 全消费者一致性专项对拍 + 遗漏审计独立复审确认 + SD 终稿对齐 canonical → 新提交独立复审；本轮不合入/归档

### work 交接（2026-10-09，R2 稳定树验证+服务验收从头重跑 → executing 继续）

- stage: work
- plan_id: PLAN-738
- plan_revision: 2
- outcome: pass（本轮指令范围——稳定代码验证+服务验收重跑——完成；T-02..T-08 整体仍打开）
- code_commit: worktree plan-738-dev `53010f145` → `b838f5f16`（R2 未提交实现主体 + 本轮 host 门修复一并入库的稳定树快照，31 files +7874/-1721；worktree=`D:/autostack/.wt/lang-738/auto-lang`，提交后 clean）
- task_ids: T-02..T-08 修复进行中的验证收口（无勾选变更，current_step 保持 1/8）
- evidence: [738-repair-round2.md](reports/738-repair-round2.md) 稳定树收口节；三 crate check 零错误；plan738 55/55；api_gen 43/43（修复 729/730 两真实红——host 门 `pub use` 转发壳跟随+await mode 证据+impl AsRef<str>/不透明类型按名身份）；module_cache 16/16、native_registry 13/13、use_semantics 7/7、autovm_persistent 20/20、CLI 10/10；真实生成服务验收 1/1（110.9s，`--features test-http-e2e`：生成→实编→serve→ready 指纹→业务 42→仅改 stdlib→陈旧→再生→重建→复验）
- blockers: 无阻塞。注意：该验收测试门控在 `feature = "test-http-e2e"` 后——漏 feature 会得到 0 测试假绿（本轮 service7 轮已废弃）
- next: R2 收口余项（callee/闭包遗漏审计、manifest 全消费者一致性、C IO 正证、最终 t/tv/tt/th 双树分诊、SD 正文）→ 新提交独立复审；本轮不合入/归档

### Revision 2 修订交接（2026-10-09，pass → work）

- stage: new
- plan_id: PLAN-738
- plan_revision: 2
- outcome: pass（修订合同就绪，不代表实施/复审通过）
- changed_tasks: T-02..T-08；补充 T-01 的最终 override 调查结论
- changed_acceptance: none（AC-01..08、SD-01..07 保留）
- authorization: 用户明确授权“按照你的说的执行修订和修复”，覆盖本节同范围设计和继续 work
- inputs: 主检出 `9fec53bdf`；基础修复 `4089b84183c6044ee68f1b6962752d4cb54da821`；R1 独立复审及修复交接报告；相关 canonical Specs 仍以原 frontmatter 为准，尚未沉淀
- evidence: §5.7 的实际 callee / 引用闭包 / immutable manifest / 真服务与 HTTP 归因合同；原登记债务不得作为豁免
- state: executing；current_step=1/8；保留已有代码和原 worktree
- next: work，完成 T-02..T-08 后新 revision-bound 独立 review；本轮不合入/归档

### R1 修复交接（2026-10-09，needs_replan；基础修复已提交）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1（原 AC/SD 不变，当前仅实施修复和设计差异交接）
- outcome: needs_replan
- code_commit: `4fbcfe9685bdf8b82266f0b95a5b0b44037be891` → `4089b84183c6044ee68f1b6962752d4cb54da821`（保留冲突记录、恢复 callee 原覆盖次序），base=`e4425b673eec46fabc25b4534084c19f2738a38f`，branch=`plan-738-dev`，worktree=`D:/autostack/.wt/lang-738/auto-lang`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；未修改依赖仓
- task_ids: T-02..T-08（均部分推进，不能勾全；current_step 保持 1/8）
- evidence: [修复明细与有界修订提案](reports/738-repair-round1.md)；版本/日志/最终门禁收据=`reports/738-repair-round1.json`。新增反例包括双 body 必拒、同会话换源拒绝、clone 异 root 重解析、深依赖失效、层改名变指纹、UTF-8 失败不丢层、真实 source label、独立 arity/被替换 callee 撤销证明；CLI target/未知模块/actual 非核心闭包均有负测。
- implemented: 基础缓存身份/指纹与 fail-closed、共同源 AssemblyPlan/错误映射、保留 native ID 分配、基础 producer 元数据与 json/io 公共映射、CLI target/模块闭包/便携源 ID、async 公共源现有语法修复；详细已落地/未闭合逐 R1..R9 对账见修复报告。
- blockers: “三件套 init 即实际 callee”的设计假设被 AutoVM 后续 JSON override 证伪；TCP read 的公开缓冲参数/单返回与实际 buf_size/双栈输出不能由现有逻辑签名等价证明。需明确共同最终绑定 snapshot 与 target adapter 合同，不能为接 strict 门擅改公开 ABI 或把未知标成 Supported。
- gates: 初始 foundation plan738=41/41，registry=13/13，tv=162/162，CLI=10/10，freshness=1/1；日常完整 t=5139/5155（14 既有红+2 旧编号断言，后二者已 scoped 修正验证）；tt=5515/5526（11 既有红）；th=100/102（back-proxy 同断言，plan707 时序失败定向复跑仍红，未确定归因）。callee 次序修正后最终 scoped=54/54 + 三 crate check pass；完整档未在该最后微调后全部再跑，版本/日志 hash 见收据。未新建 baseline worktree 双树证明，未执行 tf/taa/tu/docs_gen 专项。
- omissions: 生产引用符号 strict 门、Rust/C 实际 provider lowering、完整不可变 manifest/host 闭包指纹、六格完整公共分母、双源签名诊断、C/generated-service 真入口、最终 SD 正文与独立 review 仍未完成；没有债务化或批准延期。
- state: executing，基础代码提交保留原 worktree；未合入、未归档、未发布 canonical/ledger；旧 reviewed 证据继续无效。
- next: `/auto-plan:new PLAN-738` 仅修订 T-01/T-03/T-04 producer/实际引用闭包与 adapter 设计，保留本期所有 AC，再继续 work 补完依赖任务并独立 review 新提交。

### 独立复审 R1（2026-10-09，needs_fix）

- stage: review
- plan_id: PLAN-738
- plan_revision: 1（原验收合同不变；只回退状态/受影响 checkbox）
- outcome: needs_fix
- reviewed_commit: `e4425b673eec46fabc25b4534084c19f2738a38f`（入口 clean，审查探针未改实现；结束移除临时 probe）
- base_commit: `4262f761c5f7a0608fbfc5061ad6afa1b31cc574`；observed_master=`9bdd04ffeb1a4178ed7ce754c3ddff54fd60eba2`；merge-base=`6d69dbdc78d99f23ba90a37c9559f7fb54dd7be3`
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；Cargo.lock/Spec 输入版本与 hash=[738-review-baseline.json](reports/738-review-baseline.json)
- spec_inputs: frontmatter 七个现有 Spec + 原 SD-01..07 冻结文本；delta SHA256=`b030984624255a902b929fe335ece472e4b917232ee0ef8e2ce1dcf23aca8450`，**not approved**；未改 canonical/ledger
- acceptance_results: AC-01 partial / AC-02 fail / AC-03 fail / AC-04 partial / AC-05 fail / AC-06 fail / AC-07 fail / AC-08 partial；逐 AC→任务→代码→证据见 [738-independent-review.md](reports/738-independent-review.md)
- findings: P738-R1..R9（生产 strict 门缺席、独立逻辑签名与公共 identity 缺失、同会话/异 root 缓存陈旧、装配指纹漏身份与 fail-open、真实三目标/两 VM 入口未共用计划、CLI target/闭包/parse 假绿、inventory/环境矩阵不完整、双源位置不足、C/generated-service 证据与 SD/健康门未完成）
- evidence: [738-review-probe.rs](reports/738-review-probe.rs) + [738-review-evidence.txt](reports/738-review-evidence.txt)；真实反例=arity 1→999仍 SignatureChecked、双 body 装载成功、同会话仍 old_layer、CWD B仍用A缓存、vm→rs层改名指纹不变、Rust net --check假pass、async破损公共层假pass、jsonx actual检查无关六模块
- gates: 裸 t 完整非 fail-fast=5132/5146（14红都见已有批回执 known_reds；本轮未新做双树基线证明）；tv=162/162；tt=5503/5517（13fail+1timeout，新增观察 back_provision/dep_fields scoped均绿）；th=100/102（back_proxy 两红见执行报告基线）；普通 Rust JSON witness=1/1；CLI=8/8；freshness=1/1；新增 model.rs rustfmt 三处 diff → 必修；未跑 tf/taa/tu
- omissions/debt: P738-D1/D2 中属于六核心的生产冲突/映射必须由 R1/R2 收回，本期必要验收不能债务化；C支持样例与generated Rust→service→ready仍未完成，重开T-07/08；未承诺的非核心/parity/backend保留后续范围
- state: executing，current_step=1（T-01完成；T-02..T-08受影响验收重开；历史进度/证据保留）
- next: `/auto-plan:work PLAN-738` 修复 R1..R9并补证，再独立 review 新 commit；本轮不修实现、不合入、不归档

### work 交接（2026-10-08，T-07/T-08 完成 → execution_done）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01..T-07 全部完成；T-08 门禁+验证报告完成——execution_done，next=review）
- code_commit: worktree plan-738-dev `a4afb3b65`（T-07 slice3）→ `e4425b673`（T-08 报告）；链=75c7a1049→8964155ec→0a73d0759→a4afb3b65→e4425b673，基面 master@4262f761c+re-sync 0424673e2
- task_ids: T-07（三片）+T-08（门禁/报告）
- evidence: `docs/plans/reports/738-stdlib-verification.md`（门禁表+AC-01..08 逐条绑定+红分诊）；门禁=裸 t/tv/tt/th 全对 master 基线逐名分诊零新增红；⑤腿 rustc 实编实跑 witness 按需绿；CLI 真实二进制三模式冒烟
- blockers: 无阻塞。review 后遗留面：①C 实编 witness（无 C HTTP provider=Unsupported 明示阻塞，D3b）②真实「生成→serve」实跑（fixture 成员发现流程预存行为在案）③P738-D1/D2+as_int 下沉缺口+内嵌 a2r_std 漂移（D3b/公共面重写）
- next: /auto-plan:review（独立验证真实 callee 和 target 调用点、分母/验证等级/缓存反例）→ merge（canonical/ledger/Design33/索引/归档）

### work 交接（2026-10-08，阻塞解除：T-06 全部完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-06 全部三 bullet 完成，gated 解除；整体 executing 继续）
- code_commit: worktree plan-738-dev `0424673e2`（re-sync merge）→ `d71e823c8`（T-06(2/2)）；基面=master@4262f761c
- task_ids: T-06（gated 半①指纹函数 ②receipt ③新鲜度门）
- evidence: 勘验 736 代码全量在 master（`master..plan-736-dev`=0）；`stdlib_assembly_fingerprint` 内容/目标敏感性单测（t06_assembly_receipt）；`assembly_freshness` 真值表单测（auto-man）；api_gen scoped 44/44；plan738 30/30；三 crate check 零错误（rust_ui `merged_api_client_crud_fallback` 系 734 在案预存红非回归）
- blockers: 无阻塞任务。遗留：①真实生成Rust→serve 实跑 witness 归 T-07（fixture 成员 pac.at 发现流程预存行为待循——`auto run` 对 api_contract 夹具报 Skipping back，非 738 引入）②T-07 余项（rustc/C 实编 witness、examples/、§6.1 冲突/native 绑定族余项）
- next: T-07 slice 2（rustc 实编 witness——顺带打通真实生成链 e2e）

### work 交接（2026-10-08，T-07 第一片：VM 真执行见证 + 六核心矩阵）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-07 slice 1 完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `75c7a1049`；基面 master c3ccd32c3
- task_ids: T-07（①VM 真执行见证 ②六核心 target×env 矩阵+报告落盘）
- evidence: plan738 31/31；`docs/plans/reports/738-stdlib-matrix.{json,md}`（六模块四格全在册，非 Supported 必有原因；io 0/9=第四绑定面冻结、net 14/14、http 34/68、json 4/19、sse 0/1=stub、async 10/10）；rust claim 实证 json=supported/io=unsupported
- blockers: T-06 api_gen/736 半（736 未合入，同前）；T-07 余项=rustc/C 实编 witness（Plan 610 ⑤腿 `#[ignore]` 范式+`auto_lang::a2r_std` 资格化）、examples/stdlib/assembly/、§6.1 冲突/native 绑定族余项——不阻塞，下轮继续
- next: T-07 slice 2（rustc 实编见证——见 §8 进度注记的路径设计）

### work 交接（2026-10-08，T-04 遗留分诊 + T-05 完成 + T-06 CLI 半）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-05 全部完成；T-06 CLI 半落地、736 耦合半 gated；整体 executing 继续）
- code_commit: worktree plan-738-dev `f88449944`（T-05）→ `206a846e7`（T-06 CLI）；基面 master c3ccd32c3
- task_ids: T-04 遗留(b) 分诊、T-05（全部三 bullet）、T-06（CLI 半）
- evidence: plan738 27/27、module_cache 16/16、use_semantics 7/7、native_registry 13/13、autovm_persistent 20/20、plan727/729/730 59/59、`cargo tv` 162/162、CLI 8/8（--test-threads=1）、真实 `auto.exe stdlib inspect` 三模式冒烟（text/JSON/check 退出码 3/0/1 实测）；`cargo check -p auto-lang/-p auto/-p auto-man --lib` 零错误
- blockers: T-06 api_gen receipt + 736 ready 消费半——736 executing@T-07 未合入，其分支直接重叠 api_gen.rs/main.rs（并行改写必冲突）；736 合入后按 T-01 报告 §8 复核清单接线。不阻塞 T-07。
- next: T-07（同源样例、核心完整矩阵与实际执行证明——依赖 T-03..06 CLI 面已就绪）；T-06 gated 半随 736 合入解锁
- debt: P738-D1（13 处 id 相撞分诊裁决=check 如实上报非零，重编号 D3b）、P738-D2（json id 面分裂：catalog canonical 1906 实绑 vs 扫描名 99xx 无 shim——诚实 Unverified；公共面全量重写下轮，同 http 扫描名/io 第四面族）已登记 KNOWN-DEBT

T-05 实施要点（复审注意）：
①E2 死缓存反转为契约（P2 探针翻转，P5 先例）——`with_file` 的"合并源 hash vs
公共文件重读"错位由段级指纹条目修复；②缓存命中早退一致性为行为变更：命中
路径现补全本 epoch bytecode+manifest（clone 会话=compiled_* 重置+cache 继承
的真实早退场景，`cache_hit_rebuilds_bytecode_and_manifest` 冻结）；③
`set_assembly_target` 签名变 `AutoResult<()>`（已装载换目标=SessionTargetMismatch，
bench 同目标重声明不受累）；④absent 台账只记**选定目标**后缀层——foreign 层
是 candidate（manifest 面），其增减不改变装配选择、不入缓存身份。

T-06 CLI 实施要点（复审注意）：生产装配面=engine init 三件套
（register_std_shims+register_stdlib_ffi+build_from_inventory）；json 实勘
（探针 2026-10-08）：catalog canonical `auto.json.get`@1906 实绑（生产 json
可用面）vs 扫描名 `auto.json.json_get`@99xx 无 shim——校验器按扫描名问询故
json 诚实 Unverified 非零（`check_json_scan_face_unverified_nonzero` 冻结），
**不冒称 Supported 也不降级 Unsupported**；全局 `--format`（OutputFormat）
与子命令参数撞名已收敛（复用全局 format，JSON 为默认契约面）。

### work 交接（2026-10-08，T-04 两裁决落地）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-04 裁决面完成；整体 executing 继续）
- code_commit: worktree plan-738-dev 追加 T-04 裁决提交（stdlib.rs 单名注解 + validate.rs 声明组/双面 + 测试 19/19）
- task_ids: T-04（裁决②read_text 单名 + 检测器声明组 + 绑定面扩展完成）
- evidence: 裁决①权威名=公共 canonical 面（http 族经 inventory 实证 Supported）；裁决② auto.fs.read_text 单名+NATIVE_ID_ENTRIES 别名组放行 file/fs 族；新抓扫描面动态 id 相撞真冲突 13 处（id1607 char×conv、id9930-9933 http×transfer 等）冻结基线
- blockers: 无阻塞；遗留三（a）io 方法第四绑定面（VmModule 方法表）查询接线——当前冻结为文档化 Unverified（b）扫描面 id 相撞 13 处分诊（T-05 前或 T-06 CLI 分诊）(c) http 前缀名 vs 公共名形状全量重写（下轮）
- next: T-05（缓存一致性——依赖 T-03 装配接线，已就绪）

### work 交接（2026-10-08，T-04 进行中——校验层落地，2 真实发现待裁决）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: needs_replan（局部——仅 T-04 剩余面的名字形状裁决；T-05 可并行开工）
- code_commit: worktree plan-738-dev `ac41c2d1e`→`d70d4c8a9`→`340de9ccd`→T-04 校验层（2 files, 409+）
- task_ids: T-04（1/2——validate 层+Browser 分类+冲突检测落地；2/2 停牌待裁决）
- evidence: plan738 17/17+2 parked（#[ignore] 带精确原因）；sse parse_sse=DeclaredStub、Browser 三族 Unsupported、json 面经 build_from_inventory 实证 Supported
- blockers: ①扫描名面（auto.http.http_get，.vm.at 扫描动态 id，无绑定）与 dispatch 名面（auto.http.get，stdlib.rs 手工 shim+canonical Http.get→auto.http.get）不接合——同一函数两套名字，校验的 canonical 构造须改为"公共层符号→registry to_canonical 面"，并裁决扫描死别名名的处置；②生产注册面多名共 id 的 file/fs 别名族（id 1000+，别名形非末段且 fs.read/fs.read_text 语义异）——别名白名单形状需用户/复审裁决
- next: T-04 剩余=上述两裁决落地（公共符号名→dispatch 面校验+别名族白名单）→ T-05 缓存一致性可先开工（依赖 T-03 装配接线，不依赖 T-04 裁决）

### work 交接（2026-10-08，T-01..T-03 完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01/T-02/T-03 完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `ac41c2d1e`（T-01）→ `d70d4c8a9`（T-02）→ `340de9ccd`（T-03）；基面 master c3ccd32c3
- task_ids: T-01、T-02、T-03（current_step 3/8）
- evidence: 决策报告 §1-E1..E5 + §2b/2c/2d；plan738 15/15、use_semantics/module_cache/native_registry/repl/plan727/729/730 117/117、`cargo tv` 162/162、新改文件零警告
- blockers: T-06 消费 736 最终生成/ready 合同——保持 gated（736 已推进至 T-05 完成，合入后按报告 §8 复核）；不阻塞 T-04/T-05/T-07
- next: T-04（核心符号/native 校验与环境能力——六模块全 decl 状态、独立逻辑签名/alias/feature/stub 与实际 shim 绑定；CWD 依赖名 surface 的生产构成冻结[报告 §2d]）

T-03 实施要点（复审注意）：③persistent 修复为**行为变更**——`.vm.at` 层对
persistent/REPL 由不可见变可见，短别名注册面扩大（P5 探针由 is_none 破损证据
翻转为 is_some 修复契约，见探针头注）；edition2021 `if let` 持锁双锁死锁系
context 修复使查找首次可达后暴露的潜伏 bug（查找临时量须提升出 scrutinee）。
sibling 扫描排除 .vm.at/.c.at 为 AC-02 行为变更（VM/C 层不再进入 Rust 发射
类型上下文），plan727/729/730 金样绿。

### work 交接（2026-10-08，T-01+T-02 完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01/T-02 完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `ac41c2d1e`（T-01 探针）→ `d70d4c8a9`（T-02 模块族）；基面 master c3ccd32c3
- task_ids: T-01、T-02（current_step 2/8）
- evidence: `docs/plans/reports/738-stdlib-decision.md`（§1-E1..E5 + §2b/2c/2d T-02 增补）；`cargo t plan738` 11/11；module_cache 10/10；native_registry 13/13；新文件零警告
- blockers: T-06 消费 736 最终生成/ready 合同——736 尚 executing 3/8 未合入，保持 gated（报告 §5/§8）；不阻塞 T-03..T-05/T-07 主体
- next: T-03（VM/persistent/trans 装配接线与源映射——依赖 T-02 模型；AssemblyContext 驱动层选择替换 context_ext 硬编码，persistent context 路径修复，ModuleCache 死缓存修正在 T-05）

T-02 增补实勘（详见报告 §2b/2c/2d）：全库 isolated-parse 39/115（三类：语法破损
str.at/list.at/iter 族、跨模块依赖假阳性候选、.rs.at 镜像漂移；`use auto.str`
stdlib 导入面破损系在案已知——stdlib_tests.rs #[ignore] 佐证）；sse parse_sse
声明无 callee；native 名 surface 扫描 CWD 依赖。三者均落册为 inventory 诊断/
目录 unsupported/报告证据，不阻塞、不弱化 AC。

### work 交接（2026-10-08，T-01 完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01 单任务完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `ac41c2d1e`（探针族 6/6 绿；基面 master c3ccd32c3）
- task_ids: T-01（current_step 1/8）
- evidence: `docs/plans/reports/738-stdlib-decision.md`（E1-E5 全锚定）；`cargo check -p auto-lang` 绿；`cargo t plan738` 6/6
- blockers: T-06 消费 736 最终生成/ready 合同——736 尚 executing 3/8 未合入，保持 gated（报告 §5/§8 复核清单）；不阻塞 T-02..T-05/T-07 主体
- next: T-02（stdlib_assembly 模块 + provider 目录 + 全库 inventory，依赖本报告冻结的分母与四装配面边界）

入场说明：计划 §4.1 原执行前置为「736 独立 review pass 并合入」；2026-10-08 用户
明确指令「计划738 实施它」，据此从 drafting 进入 executing。该前置的替代满足方式
（736 worktree 只读勘验 + T-06 gated + 其余任务生产耦合分离论证）记录于决策报告
§7。736 合入后按报告 §8 四项复核，如与 §5 描述漂移按最终代码重锚（不改 AC）。

### 起草交接（2026-10-03）

- stage: new
- plan_id: PLAN-738
- plan_revision: 1
- outcome: blocked
- blocker: 736 executing尚无独立review pass+merge的最终生成/启动合同；本期共享lib/auto-man/CLI生产接线。
- next: 736 reviewed+merged → T-01真实装配/provider/缓存原型冻结 → /auto-plan:work PLAN-738。
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-08（新）
- 本次只计划/Design簿记，未创建738 worktree、未跑Cargo/编译/网络、未改canonical或736进度。blocked是实施顺序依赖，不缺用户规划输入。
- 起草结构检查通过：11章节、8任务/8AC/7SD覆盖、已有影响路径/Spec路径存在、编号唯一、next-id已推进、新增链接有效、git diff --check通过。Design索引既有513计划链接不在本期修复面；此检查不是实施独立复审。

## 10. 待澄清事项

- R1 的 `needs_replan` 是历史交接，已由 revision 2/3 的同范围修订接续；当前无需要用户补充的规划输入。producer/adapter 的实际差异须在 Phase 3 以独立契约/实编和负测闭合，不能削弱 AC 或把剩余必修项归 D3b。JSON 可空声明不自动规定失败返 null；TCP read 已知参数/返回差异保留公开 ABI 与明确验证边界。
- T-01负责六模块producer签名与当前parser/source-map/target实际入口原型；事实与canonical冲突提出具体修订，不以“代码如此”擅改公开API/语义。
- 全库inventory不等于全库strict/语义parity；D3b扩展非核心provider门和资源/错误/取消ABI对拍，不能因D3a有manifest称所有符号能用于所有环境。
- .vue等环境文件目前非统一stdlib后缀机制；有适配路径才登记，不能假造Io浏览器实现或因Vue前端误禁Native后端。
- 736最终服务实现会改变生成/ready路径，执行须用其merge后版本；738不并行改其核心入口、不代实施736。
- CPU/native阻塞、TLS/完整WS/更大上传/崩溃恢复按HTTP路线独立需求；本期提升的是标准库实现来源与错误可见性，不增加这些HTTP协议能力。
