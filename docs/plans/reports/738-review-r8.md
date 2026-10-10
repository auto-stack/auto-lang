# PLAN-738 独立复审 R8（2026-10-10，pass → reviewed）

- stage: review / plan_revision: 3 / verdict: **pass**（P738-R7-01/P738-R7-02 闭合确证；R7-03 裁量落地、R7-04 登记落地；全分母矩阵与全部声称门禁独立复现；两项 P3 观察登记，非阻塞）
- reviewed_commit: worktree `plan-738-dev@3713337d97f37d68bddaca22ce16fb152cfc8db5`（入场 clean，只读复审，结束 clean）
- 修复链 diff `6c3943327..3713337d9` = 恰 3 文件 +265/−53（host.rs 155+/tests 79+/trans rust.rs 84+）——与交接声称触面一致；R3–R7 已验面在该 3 文件之外未被扰动（受扰面由本轮门禁复跑覆盖，见 §3）。
- independence: 全新上下文 R8 agent，未参与 738 任何实现/复审。
- 主检出入场 `920fa3d30`；本轮簿记只 add 本报告与计划文件（主检出他人 WIP 不入）。
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`（只读未动，与 R5/R6/R7 记录一致）。

## 1. P738-R7-01 闭合确证（核心，四级证据）

1. **Diff 核对**（`6c3943327..3713337d9`）：
   - has_key 发射臂双闭括号 bug 修复——删除 `if !use_str { write!(out, ")")?; }`（rust.rs 原 L10277 区），双臂（has_key/has_key_str）现均括号配平；
   - **BoolToIntIf 适配契约** ×2（`JsonValue.has_key` callee=has_key 与 callee=has_key_str+str-receiver facade），observe_emission 新增 `if <call> {1} else {0}` 壳识别（then/else 恰为整型字面量 1/0，cond 为唯一主调用）；
   - **len ScalarCast 契约**（模块拼写 `(.. as i64)` 宽度归一——R7-04 建议项落地）；
   - **Paren 壳剥离**（`(x as i64)` 解析为 Paren 包 Cast）；
   - **AST 实参序列化**（`serialize_positional_args`——shape③ 构造不再依赖缺陷发射文本；R7-03 裁量落地）；
   - **Null 哨兵族扩展**（parse + JsonValue.get/get_at 三面 `returns=="JsonValue" → "JsonValue?"`，与 parse 既有表示适配同源）；
   - **守卫诚实拒绝**（shape① 公共方法面存在而 !self_contained → `SIGNATURE_UNVERIFIED ... fragmented/chained emission`，不再静默 return Ok——R7-01 缺陷形态「守卫跳过」根除）；
   - case④ 增 `local_var_types/local_modules/sibling_modules` 排除（与 collect_core_reference 核心分支 L5756-5757 同源判据）。
2. **正式矩阵**：`receiver_method_drift_matrix_both_spellings` **1/1**（单线程）——全分母 12 方法 × 双拼写 24 格，预期编码与声称逐格一致（见 §2）；`r5_receiver_closure` 族 **4/4**（矩阵+收集 proof/漂移拒绝/用户隔离）。
3. **CLI 级独立复验**（3713337d9 构建的真实 `target/debug/auto.exe` + 隔离 `AUTO_STDLIB_ROOT` 改 json.at 声明）：
   - has_key 模块拼写基线：`json.has_key(v,"k")` → **exit0 + references=[JsonValue.has_key, parse]**（R7 时 refs 仅 [parse] 无方法 proof）；
   - 漂移（has_key 返回 int→str）：**exit1 + `STDASSEMBLY.SIGNATURE_DRIFT: json.JsonValue.has_key: public return str does not match the adapter return int`**（R7 时漂移后仍 exit0）。
4. **产物正确性**：`auto trans --path hk_mod.at rust` 产物 L11 = `let n = if a2r_std::json::has_key(&v, "k") { 1 } else { 0 };`——括号配平（全文 depth 0/min 0）、无 `[a2r warning] Output validation failed`（R7 在案的 unbalanced parentheses 产物编译错根除）、assembly.json refs 含 JsonValue.has_key。

绿面抽查（非矩阵格复验）：get 模块拼写基线 **exit0**（Null 哨兵族生效——R7 时为 exit1 DRIFT `?JsonValue/JsonValue`）、漂移（JsonValue?→int2）**exit1 + return drift: int2 / ?JsonValue**；keys 模块拼写基线 exit0、漂移（[]str→[]int）**exit1 DRIFT**。

## 2. 全分母矩阵（12 方法 × 双拼写）——预期 vs 实测一致

| 方法 | receiver | module | 实测依据 |
|---|---|---|---|
| type | 绿（value_type 反向别名） | 拒（无臂 case④） | 矩阵编码+测试绿 |
| as_string | 绿 | 绿 | 矩阵 |
| as_number | 绿 | 拒（无臂 case④） | 矩阵 |
| as_int | 绿 | 绿 | 矩阵（R6 已证） |
| as_bool | 拒（公共 int vs producer bool 真漂移） | 拒 | 矩阵 |
| as_array | 绿 | 拒（无臂 case④） | 矩阵 |
| keys | 绿 | 绿 | 矩阵+CLI 漂移 exit1 |
| len | 绿 | 绿（ScalarCast 契约） | 矩阵（R7 时 module 格 exit1） |
| get | 拒（parser「Expected term」——receiver 带参为语言层不支持拼写） | 绿（Null 哨兵） | 矩阵+CLI 基线/漂移 |
| get_at | 拒（parser） | 绿（Null 哨兵） | 矩阵 |
| has_key | 拒（parser） | 绿（BoolToIntIf） | 矩阵+CLI 基线/漂移 |
| is_null | 拒（真漂移） | 拒（无臂 case④） | 矩阵（R6/R7 已证） |

- 矩阵测试的漂移腿对全部 24 格断言 `drifted.is_err()`（parser 拒绝拼写格同样不产出成功产物，非 STDASSEMBLY 假绿）。
- 声称的矩阵预期（绿面/真漂移面/parser 拒绝面/无臂拒绝面四类分野）与测试编码、CLI 抽查三方一致——**R6 交接「分母 12 个必须双拼写矩阵验收」已兑现**。

## 3. 回归面（全部 3713337d9 实跑）

| 门禁 | 结果 | 分诊 |
|---|---|---|
| `cargo t plan738 --no-fail-fast` | **73/73**（43.1s，含矩阵 1.94s） | 与声称一致 |
| `cargo t plan724 --no-fail-fast` | **5/5** | — |
| `cargo test -p auto --bin auto stdlib -- --test-threads=1` | **10/10** | — |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | **44 passed/1 ignored** | — |
| `cargo tv` | **162/162** | — |
| `cargo nextest run -p auto-lang --lib --features test-trans --no-fail-fast` | 5558 run/**14 fail 全基线** | 14 = 名册逐名：musk×6/desktop_protocol/iced×2/ash_leak/plan502/plan707_client/plan606_029（=R7 同款 13 项）+ plan730_raw_receive_commit_conflict_roundtrip（固定端口上传族环境红在册，**隔离复跑 1/1 绿**；R7 轮该族绿而 plan484_024 红——flake 族轮换，同册）；plan484_024 本轮绿；**非基线红=0**——AST 序列化/Null 哨兵/case④ 收紧/守卫诚实拒绝均无语料回归 |
| 三 crate check（auto-lang/auto/auto-man） | **零 error** | 触面文件零新警告（host.rs:913 `mut returns`/host.rs:65 RuntimeSpec 变体/rust.rs:1102/7878/8103 均在 6c3943327 同位同款=预存，非 738 触面引入） |
| 修复 diff debug 残留 | **0**（R7DBG/dbg!/eprintln/println 全无） | — |
| 服务完整链 | `http_e2e_plan738 --ignored` **1/1@25.84s** | 生成→收据→实编→serve→业务→stdlib-only 失效→再生语义保持 |

## 4. SD/对账复核

- **SD-04 ✅**：重锚后描述与代码行为逐条一致——四形态（前三验证/第四诚实拒绝）、「未证明=SIGNATURE_UNVERIFIED 不产出成功产物」（has_key 反例已消除）、适配契约族（bool→int if 壳 has_key/宽度归一 cast 壳 len+last_status/Null 哨兵 parse+get+get_at）、receiver 带参方法 parser 不支持拼写、无臂模块拼写（type/as_number/as_array）case④ 拒、is_null/is_bool 真漂移双拼写拒、receiver/is_static 证据来自公共声明、Vec<T>→[]T。**本轮 approved**。
- **reference-audit #11 ✅**：「12 方法分母归属：三形态验证+第四拒绝」与实测相符（has_key 模块拼写格已从「无验证无拒绝」修复为 BoolToIntIf 验证+proof）。
- verification §9（R7 轮收据）门禁数字与本轮复现一致（服务链时延 109.5s→25.84s 为环境波动，绿性一致）。
- 计划 §9 R7 交接（3 files +265/−53、矩阵全分母、四类预期）vs 实际 diff **一致**。
- R7-03 裁量落地：AST 实参序列化（shape③ 构造 `a2r_std::json::get(&v, "k")` 携真实实参；arity 误诊消息现附 producer/callee 名）。
- R7-04 落地：len ScalarCast 契约补齐（json.len(v) 模块拼写从 exit1 变绿+proof——矩阵 len/module 绿格实证）；无臂模块拼写维持 case④ 诚实拒绝。

## 5. 本轮新观察（P3，非阻塞，登记不等于批准延期）

| id | 级别 | 内容 | 锚点 |
|---|---|---|---|
| P738-R8-01 | low（观察） | **链式核心调用面行为收紧**：`json.parse(s).len()`（receiver 为核心调用的链式方法调用）现 exit1 `SIGNATURE_UNVERIFIED ... fragmented/chained emission`——6c3943327 为静默跳过（R5 收据在案的「链式中间态记边界」债务面）。按 R5 修复合同「不能仅写债务保留放行」与 SD-04「未证明不产出成功产物」，拒绝为合同正确方向（R7-01 修复方向「…验证/拒绝」明示二选一）；全仓资产/语料零命中（唯一出现为 examples/ui/027 注释），tt/tv 非基线红=0。残留三小项：rust.rs L5613-5617 形态守卫注释仍写「跳过并记边界」（与新拒绝行为不符，**过时注释**）；无测试钉死链式面新行为；SD-04/audit 未显式点名链式子面。建议 merge 时补一行测试+注释/SD 一句话登记 | trans/rust.rs shape① else 分支 |
| P738-R8-02 | low（观察） | **str 实参 has_key 模块拼写 fail-closed**：`json.has_key(s,"k")`（s 为 str）exit1 `PROVIDER_CLAIM_NO_CALLEE: json.has_key_str`——新增的第二条 BoolToIntIf 规则（callee=has_key_str+str facade）在该面被 provider claim 门先行拦截（has_key_str 无公共 .at 声明），规则实际不可经此面产出 proof（防御性在案）。行为 fail-closed 且非回归（6c3943327 同面 exit1「adapter expression requires independent proof」） | ADAPTER_RULES has_key_str / provider claim 门 |

## 6. 判定与路由

- **R7-01 闭合 ✓ / R7-02 闭合 ✓（双面：shadow 变量 + 本地 json 模块——CLI --actual exit0 pass、trans 产物 `(json.len() as i64)` 有效 Rust）/ R7-03 裁量 ✓ / R7-04 登记 ✓**；声称的矩阵预期、门禁数字全部独立复现；diff 范围证明 R3–R7 已验面未被扰动。AC 快照：AC-01..08 全 pass（AC-03 has_key 漂移检出闭合；AC-05/07/08 的 SD-04/audit 声明与行为再一致）。
- state: **reviewed**（current_step=7：T-01/T-09..T-14 完成；T-02..T-08 按 Phase 3 映射由 T-09..T-14 承接，R4 口径保持）；不合入/不归档/不删 worktree；禁 tf（批量回归档未触发，归 `/auto-plan:regress` 到期判定）。
- next: `/auto-plan:merge PLAN-738`（授权后）——canonical/ledger 沉淀、Design33/索引、wt-guard+worktree 清理；P738-R8-01 的过时注释/链式面测试建议随 merge 顺手处理或转入 D3b 清单。
