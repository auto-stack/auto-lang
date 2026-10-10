# PLAN-738 独立复审 R7（2026-10-10，needs_fix）

- stage: review / plan_revision: 3 / verdict: **needs_fix**（P738-R7-01 必修：`json.has_key(v, "k")` 漂移后仍 exit 0 无 proof——R6-01 原始缺陷形状在分母成员 has_key 的模块拼写格存活；P738-R7-02：case④ 误伤 `json` 同名局部变量/本地模块。R6-01 矩阵面/R6-02/R6-03 闭合确证）
- reviewed_commit: worktree `plan-738-dev@6c3943327dab67bce3da2ed7e9779728b54935f0`（入场 clean，只读复审，结束 clean）
- 修复链 diff `68398d2d6..6c3943327` = 恰 4 文件 155+/27−（host.rs/tests/trans rust.rs/rust_ui.rs）——与声称触面一致；R3/R4 pass 面在该 4 文件之外未被扰动（受扰面由本轮门禁复跑覆盖，见 §4）。
- independence: 全新上下文 R7 agent，未参与 738 任何实现/复审。
- 主检出入场 `1b6909031`；本轮簿记只 add 本报告与计划文件（主检出他人 WIP 不入）。

## 1. R6-01 矩阵面（keys/as_int/is_null × 双拼写）——闭合确证

- 正式测试：`receiver_method_drift_matrix_both_spellings` **1/1**（`cargo test -p auto-lang --lib receiver_method_drift_matrix -- --test-threads=1`）；`r5_receiver_closure` 族 **4/4**（矩阵+收集/漂移拒绝/用户隔离）。
- **CLI 级独立 12 格矩阵复验**（真实 `target/debug/auto.exe`（6c3943327 构建）+ 隔离 `AUTO_STDLIB_ROOT` 改 json.at 声明，同 R5/R6 方法）——与执行者声称的预期行为**逐格一致**：

| 格 | 基线 | 漂移 |
|---|---|---|
| keys/receiver（`v.keys()`） | exit 0 + `JsonValue.keys` proof | exit 1 + SIGNATURE_DRIFT |
| keys/module（`json.keys(v)`） | exit 0 + proof（落穿生效） | exit 1 + DRIFT |
| as_int/receiver | exit 0 + proof | exit 1 + DRIFT |
| as_int/module | exit 0 + proof（Value 分派修正生效——R6 前误走 str 变体） | exit 1 + DRIFT |
| is_null/receiver | **exit 1**（DRIFT：公共 int vs producer bool 真漂移，同 is_valid 族诚实面） | exit 1 |
| is_null/module | **exit 1**（UNVERIFIED：case④ 未接管发射臂诚实拒绝） | exit 1 |

- 修复机制四项在代码与实测双确证：shape③ 泛化（`receiver_is_json_value(object)` → `public_method_symbol` 分母归属，不再 len-only）、模块面落穿（verify None → `collect_core_method_reference`）、case④（owner 拼写 json/Json + 公共方法面存在且未接管 → Err）、Value 分派（`json_value_vars ∪ json_value_bindings`，host L1138）与 Vec<T>→[]T（host.rs logical_type）。抽查非矩阵方法 as_string：receiver/module 基线 exit0+proof、漂移（str→int）exit1+DRIFT——泛化声明在抽查面成立。

## 2. 全分母普查（12 方法 × 双拼写）——发现 R7-01/R7-02/R7-03/R7-04

> R6 交接教训明言「公共方法分母是 12 个，修复面必须按分母泛化并用双拼写矩阵验收」——本轮按分母全量实测（基线 + 抽样漂移），方法清单=std lib/auto/json.at 的 JsonValue 12 公共方法（type/is_null/as_string/as_number/as_int/as_bool/as_array/keys/get/get_at/has_key/len）。

基线全景（CLI --actual --check，6c3943327）：

| 方法 | receiver 基线 | module 基线 | 判读 |
|---|---|---|---|
| keys / as_int / as_string | exit0+proof | exit0+proof | ✓ 健康 |
| as_number / as_array / len | exit0+proof | **exit1**（as_number/as_array=case④ 未接管；len=UNVERIFIED「adapter expression requires independent proof」——有接管臂但 ScalarCast 形状无 ADAPTER_RULES 契约） | fail-closed 可辩护（R7-04 观察） |
| is_null / as_bool | exit1 DRIFT（公共 int vs producer bool 真漂移诚实面） | is_null=case④ 拒；as_bool=DRIFT | ✓ 诚实面 |
| type | exit1（`type` 为 Rust 关键字，发射不可解析 syn 拒绝） | exit1（case④） | ✓ 诚实面 |
| get / get_at | exit1「emitted arity」**误诊** | exit1 DRIFT `?JsonValue / JsonValue`（公共声明可选 vs producer 非可选——真分歧诚实面） | R7-03 |
| **has_key** | exit1「emitted arity」误诊 | **exit0 无方法 proof（refs 仅 parse）** | **R7-01** |

### P738-R7-01（P1 必修）——`json.has_key(v, "k")` 漂移不检出

- 实测：基线 `status:pass`、manifest references=`[parse]`（无 JsonValue.has_key）；隔离根把 `pub fn JsonValue.has_key(self JsonValue, key str) int;` 改 `str` 后复跑**仍 status:pass exit0**——P738-R6-01 的原始缺陷形状（「漂移后仍 exit 0 无 proof」）在分母成员 has_key 的模块拼写格完整存活。
- 根因链：①发射臂 `("json","has_key")`（rust.rs L10185）拆段写产物括号不平衡——`if a2r_std::json::has_key(&v, "k")) { 1 } else { 0 }`（多一个闭括号，trans CLI 层 `[a2r warning] Output validation failed ... unbalanced parentheses (depth: -1)` 在案，产物本身不可编译）；②`collect_core_method_reference` 形态守卫（R5-02）按括号不平衡判「拆段片段」跳过验证且 `return Ok(())`——case④ 在 shape① 循环 `return` 之后**不可达**；③于是接管臂发射的核心方法引用既无 proof 也无拒绝。违反 R5/R6 修复合同（「无法证明的可编译形状……不得 exit 0」「不能仅写债务保留放行」）与 SD-04 重锚声明（「未证明=SIGNATURE_UNVERIFIED，不产出成功产物」——该格漂移时产出成功产物）。
- 对照组：has_key/receiver 漂移被拒（虽经 R7-03 误诊路径）；keys/as_int 等矩阵成员双拼写闭合——缺口恰好落在矩阵未覆盖的 has_key 模块拼写格。
- 修复方向（不降 AC）：发射臂修平衡（或守卫区分「拆段不完整片段」与「完整 FQN 调用+适配包装」，后者应提取内层调用验证/拒绝）；矩阵测试与 CLI 验收按 R6 交接要求**扩到全 12 方法双拼写**（本轮表格可直接用作验收基线）。

### P738-R7-02（P2 必修）——case④ 误伤 `json` 同名局部变量/本地模块

- 实测反例：`use auto.json` + `let json = "abc"` + `print(json.len())`（合法用户代码，str 变量 shadowing 模块名）→ **exit1 UNVERIFIED「json.len: module-qualified method spelling has no verifiable emission」误拒**；R6 基线（68398d2d6）同代码 exit0——**R6 新引入的误拒回归**。
- 根因：case④ 判定只看 `owner 拼写 == "json"|"Json"` + 公共方法面存在；`collect_core_reference` 模块面分支的 local 排除（`local_var_types/local_modules/sibling_modules`，L5706-5708）只挡 core verify 分支，被排除后 L5732 无条件落穿 `collect_core_method_reference`，case④ 在其中不复查排除状态。
- 同类面：用户本地 `json` 模块（`mod json`+`use json`+`json.len(3)` 用户函数）转译亦被拒（「json.JsonValue.len: adapter expression requires independent proof」——接管臂按名字匹配+契约缺口路径）。违反 R5-02 确立的「同名用户符号不误拒」原则（其测试 `user_same_name_method_is_not_core` 未覆盖 receiver 名恰为 json 的情形）。
- 修复方向：case④ 复查 receiver 的 local 排除（与 L5706-5708 同源判据），只对真正非 shadowing 的模块限定拼写拒绝；补 shadowing 反例测试（str 变量 json/本地模块 json × 撞名方法）。

### P738-R7-03（观察）——shape③ 等效形式构造丢实参 → arity 误诊

- get/get_at/has_key 的 receiver 直发（多参方法）由 shape③ 构造 `a2r_std::json::<m>(&v)`（无第二实参）验证 → 恒报「emitted arity / Rust producer arity」。当前三方法恰好各有真分歧（get/get_at 的 `?JsonValue/JsonValue`、has_key 的 `int/bool`），exit1 结果与真漂移面一致，但诊断消息误导（对照 module 拼写的 return drift 诊断）；若未来多参方法无真分歧将误拒合法调用。建议从 text 提取实参或按公共声明元数构造。

### P738-R7-04（观察）——合法模块拼写的 fail-closed 拒绝面（登记即可）

- `json.len(v)`：有接管臂+完整 FQN 发射（`(a2r_std::json::len(&v) as i64)`），但 ScalarCast 形状无 ADAPTER_RULES 契约 → 基线拒（建议补契约；value_len 扁平 helper 有契约而限定拼写无，不对称）。
- `json.as_number(v)`/`json.as_array(v)`/`json.type(v)`：无接管发射臂 → case④ 拒（合同允许的诚实拒绝；发射原样保留本不可核对）。
- `json.get_u64`/`json.to_string`/`json.get_str` 模块拼写 exit0 无 proof：无公共声明（json.at 无顶层/方法声明）= P738-D2 已登记 legacy 边界，非新缺口（SD-04 面外边界清单已含）。

## 3. R6-02 / R6-03——闭合确证

- **R6-02 ✓**：`bind_workspace_lock` 返回 bool——`get_mut` None→false、值相同（已绑定）→true、`fs::write` 失败→false；唯一调用点 L4037 直接以返回值为 absent→物化分支的新鲜度结论（写失败保守判陈旧，下次成功写回收敛）。其余 `let _ = fs::write`（L3917/3932/4205）均在再生臂/生成 workspace 写入，非身份核验路径，写失败经后续读取失败走陈旧，不构成「身份未建立却放行」。
- **R6-03 ✓**：verification §8 已记 plan498_bar_group_emphasis 入 flake 名册（隔离复跑绿）；§2 为 R3/R4 历史收据节按 §7 语义保留未动（名册以 §7/§8 递补为准）。本轮 tt 全档 plan498 **绿**——名册分诊口径可用。

## 4. 回归面（全部 6c3943327 实跑）

| 门禁 | 结果 | 分诊 |
|---|---|---|
| `cargo t plan738 --no-fail-fast` | **73/73**（52.9s） | 与声称一致（含矩阵测试） |
| `cargo t plan724 --no-fail-fast` | **5/5** | — |
| `cargo test -p auto --bin auto stdlib -- --test-threads=1` | **10/10** | — |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | **44 passed/1 ignored** | 与声称一致 |
| `cargo nextest run -p auto-lang --lib --features test-trans --no-fail-fast` | 5558 run/**14 fail 全基线** | 14 = R6 名单 14 项逐名命中（musk×6/desktop_protocol/iced×2/ash_leak/plan484_024/plan502/plan707_client/plan606_029 flake 族）；plan498 本轮绿；**非基线红=0**——Value 分派修正（as_int_str→as_int）无语料回归 |
| `cargo tv` | **162/162** | — |
| freshness 族 | 双真值表 2/2 + shell_pack 预存环境红（R3/R4 在案同款） | 基线一致 |
| 三 crate check | 零 error | warnings（unused var/doc comment）均在 auto-down path-dep 与 iced renderer——非 738 触面 |
| 服务完整链 | `http_e2e_plan738 --ignored` **1/1@31.45s** | bind 保守化后语义收敛（生成→收据→实编→serve→业务→stdlib-only 失效→再生） |

## 5. SD/对账复核

- **SD-04 ✗（随 R7-01 失效）**：重锚后四形态描述与矩阵面行为一致，但「未证明=SIGNATURE_UNVERIFIED，不产出成功产物」「全部经 verify_rust_reference」被 has_key/mod 漂移 exit0 反例证伪；面外边界清单亦未含 has_key 模块拼写守卫跳过格与 len/mod 契约缺口。须随 R7-01/02 修复再锚或如实登记。
- **reference-audit #11 ✗（同上）**：「12 方法分母归属：三形态验证+第四拒绝」在 has_key 模块拼写格不成立（无验证无拒绝）。
- §9 R6 修复交接与 T-11/13/14 重闭记录 vs 实际 diff（4 文件 155+/27−）**一致**；「is_null 双拼写基线即拒」「json.as_int(v) Value 分派」等声称与实测相符——仅「公共方法面泛化到全部公共方法」在 has_key 模块拼写格超出实际（§2）。
- 门禁数字对账：交接声称 plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44、tv 162/162、tt 非基线红=0、服务链 1/1——本轮全部复现属实。

## 6. Findings

| id | 级别 | 内容 | 锚点 |
|---|---|---|---|
| P738-R7-01 | **high（必修）** | `json.has_key(v,"k")`（JsonValue.has_key=12 分母成员）模块拼写：接管臂拆段写括号不平衡→形态守卫跳过→基线与**漂移均 exit0 无方法 proof**（refs 仅 parse）——R6-01 原始缺陷形状存活；违反 R5「不能仅写债务保留放行」红线与 SD-04 声明；矩阵未按 R6 交接教训扩到分母全量 | trans/rust.rs L10185 发射臂 + collect_core_method_reference 形态守卫（L5585-5603）→ shape① 循环 return 使 case④ 不可达；AC-03/05/07/08、T-11/T-13/T-14 |
| P738-R7-02 | medium（必修） | case④ 误伤：`let json="abc"; json.len()`（str 变量 shadowing）exit1 UNVERIFIED 误拒（R6 前 exit0，回归）；本地 `json` 模块同被拒——case④ 不复查 L5706-5708 的 local 排除 | trans/rust.rs case④（L5667-5679）；AC-05/07、T-11 |
| P738-R7-03 | low（观察） | shape③ 等效形式丢实参致 get/get_at/has_key receiver 直发恒报 arity 误诊（真分歧在返回类型；exit1 结果恰与真漂移一致） | trans/rust.rs L5657-5661 |
| P738-R7-04 | low（观察） | 合法模块拼写 fail-closed 拒绝面登记：json.len（有臂无 ScalarCast 契约，建议补）、as_number/as_array/type 模块拼写（无臂 case④ 拒）；get_u64/to_string 无 proof=P738-D2 既有边界非新缺口 | ADAPTER_RULES / SD-04 边界清单 |

## 7. 判定与路由

- R6-01 矩阵面闭合 / R6-02 闭合 / R6-03 闭合；R6-01「泛化到全部公共方法」**未完全闭合**（R7-01 漏拒格 + R7-02 误伤回归）→ 整体 **needs_fix**。AC 快照：AC-01/02/04/06 pass（本轮确证+门禁复现）；AC-03 fail（has_key/mod 漂移不检出）、AC-05/07/08 受 R7-01 拖累（SD-04/audit 声明与行为不符）。
- state: **executing**；重开 T-11（R7-01 漏拒 + R7-02 误伤）/T-13（SD-04、audit #11 随最终行为再锚）/T-14（新提交重跑受影响门禁+分母全量 CLI 验收）；T-01/T-09/T-10/T-12 维持闭合；current_step=4（沿 R6 口径）。
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复 P738-R7-01（发射臂平衡/守卫区分完整调用，矩阵扩到 12 方法双拼写）与 P738-R7-02（case④ local 排除），顺带裁量 R7-03/R7-04，重跑受影响门禁，SD-04/audit 再锚，新最终提交后再独立 review。不合入/不归档/不删 worktree；禁跑 tf。
