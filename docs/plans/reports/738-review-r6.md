# PLAN-738 独立复审 R6（2026-10-10，needs_fix）

- stage: review / plan_revision: 3 / verdict: **needs_fix**（P738-R6-01 必修；R5-01/R5-03 闭合确证）
- reviewed_commit: worktree `plan-738-dev@68398d2d61a6e93a908a01e1442c8dff65983204`（入场 clean，只读复审，结束 clean）
- 修复链：`9c255993b` → `bc0b95cfd` → `796d279a4` → `68398d2d6`；`git diff --name-only 9c255993b..68398d2d6` = 恰 6 文件（stdlib_assembly/{host,manifest}.rs、tests/plan738_stdlib_assembly_tests.rs、trans/rust.rs、auto-man/{api_gen,rust_ui}.rs）——与声称触面一致；R3/R4 pass 面在该 6 文件之外未被扰动（受扰面由本轮门禁复跑覆盖，见 §4）。
- independence: 全新上下文 R6 agent，未参与 738 任何实现/复审。
- 主检出入场提交 `abbd38ab8`；本轮主检出簿记只 add 本报告与计划文件。

## 1. R5-01（lock 一次绑定）——闭合确证

代码（rust_ui.rs L4029-4062）：

- `lock_freshness(recorded, current)` 纯函数 = `(Some(a), Some(b)) => a == b, _ => false`——除 absent/absent（尚无物化）与逐字节相等外全部陈旧；**纯函数层不再有 absent→任意实值豁免**。
- 门内唯一豁免分支 `(Some("absent"), Some(materialized)) if materialized != "absent"` → `bind_workspace_lock` 把实际 lock FNV 写回收据 `workspace_lock` 字段（serde_json 值级更新，其余字段逐字不动，值相同不重写=幂等）后判新鲜；此后严格比较。旧收据缺字段（None）→ 陈旧保守再生。
- 边界审计：绑定后收据持实值哈希，漂移/删除/读取失败（归一 absent 对实值）均走严格比较拒绝；未发现其它永久放行路径。absent/absent 是唯一 absent-新鲜形态（真值表 L5052 冻结）。

测试（68398d2d6 实跑）：

- `cargo test -p auto-man --lib review738_r5_lock_binding_state_machine -- --test-threads=1 --nocapture` = **1/1 ok**。断言序列完整覆盖 R5 反例状态机：生成时 `workspace_lock=="absent"` → absent/absent 新鲜 → 写入 lock_v1（0.1.0）首次物化 fresh **且收据绑定 `fnv1a(lock_v1)` 精确哈希断言** → 同内容仍新鲜 → lock_v2（0.2.0）漂移拒绝 → 删除拒绝 → 收据删字段拒绝。零 clone、真实 `generate_api` + 隔离 workspace。
- freshness 族：`assembly_freshness_truth_table` / `lock_freshness_truth_table` 两测试**各自独立运行**均 ok（R3-01 假象未复发）；`test_shell_pack_lib_freshness` 红为 R3/R4 在案预存环境红（同 panic：rust_ui.rs:6055 shell-pack 入库物过期）。

## 2. R5-02（receiver 公共方法入门）——**部分闭合，P738-R6-01 必修**

### 2.1 已闭合面（实测确证）

- `r5_receiver_closure` 3/3（`cargo test -p auto-lang --lib r5_receiver_closure -- --test-threads=1`）：正例（parse 绑定 v 的 `v.len()` 收集 `JsonValue.len` proof，returns=int）/ 反例（隔离 stdlib 根 `JsonValue.len` int→str → 转译 Err 含 SIGNATURE_DRIFT+JsonValue.len）/ 用户类型同名方法隔离（`Box.new`+`str.len("x")` 无 STDASSEMBLY 不误拒）。
- **CLI 级独立复验**（真实 `target/debug/auto.exe`，隔离 `AUTO_STDLIB_ROOT` 复制全量 stdlib/auto/*.at，同 R5 §P738-R5-02 方法）：
  - `v.len()` 原始：exit 0 / status=pass / manifest 含 `JsonValue.len` proof（发射 `(v.len() as i64)`，shape③ json_value_bindings def-use 判据）。
  - `JsonValue.len` int→str 漂移：**exit 1 + SIGNATURE_DRIFT + JsonValue.len 在诊断**——R5 原反例（exit 0 无 proof）闭合。
  - `w.len()`（`json.get` 绑定变量，发射 `a2r_std::value_len(&w)`，shape② 扁平 helper）：exit 0 且 manifest 含 `JsonValue.len` proof。
- `public_method_symbol` 分母归属+别名表、方法 receiver/is_static 契约（verify_plain_reference 从公共声明取 Owner/is_static）在案；形态守卫（`).` 链式/括号不平衡拆段片段跳过）在代码注释如实声明，未误伤用户方法面。

### 2.2 未闭合面（CLI 实测反例，全部 68398d2d6 + 真实 auto.exe）

| 形状 | 发射 | 原始 exit | 漂移后 | proof |
|---|---|---|---|---|
| `v.keys()`（receiver 直发） | `v.keys()` 逐字（无 FQN） | 0 | `JsonValue.keys` []str→[]int **仍 exit 0** | 无 |
| `json.keys(v)`（模块限定形态） | `a2r_std::json::keys(&v)`（**带核心 FQN**） | 0 | 同漂移 **仍 exit 0** | 无 |
| `v.is_null()`（receiver 直发，**可编译**——serde `Value::is_null` 固有方法，producer 返回 bool vs 公共 int） | `v.is_null()` 逐字 | 0 | `JsonValue.is_null` int→str **仍 exit 0** | 无 |
| `v.as_int()` / `w.as_int()`（get 绑定） | `w.as_int()` 逐字 | 0 | （同机制无 proof） | 无 |

根因分组：

1. **shape③ 仅 len**：`if method == "len" && self.receiver_is_json_value(object)`——其余 11 个 JsonValue 公共方法（type/is_null/as_string/as_number/as_int/as_bool/as_array/keys/get/get_at/has_key）receiver 直发不收集、不拒绝。shape② 的 `value_to_int` 腿实践中不可达（`.to_int()` 发射无 to_int→as_int 别名映射，`public_method_symbol` 返 None；`.as_int()` 从不发射 value_to_int）。
2. **模块限定方法拼写在 shape① 之前被消费**：`collect_core_reference` 模块分支以裸名 `"keys"` 查公共面（json.at 无顶层 keys，仅 `JsonValue.keys` 方法）→ `Ok(None)` 静默无 proof 后 `return`——带核心 FQN 的真实发射（`a2r_std::json::keys(&v)`）永远到不了 `public_method_symbol` 归属。声称的「模块限定形态收敛」对模块限定**方法**调用不成立（模块限定**顶层函数**如 parse/is_valid 仍正常）。
3. **声明与对账过度**：work 交接/T-11 重闭「公共方法 receiver 调用进 strict 门」与 SD-04「公共方法 receiver 调用……全部经 verify_rust_reference……未证明=SIGNATURE_UNVERIFIED，不产出成功产物」均超出实际行为（上表 4 形状 exit 0 出成功产物）。R5 修复合同明确「对未经证明的被引用核心方法拒绝……不能仅写债务保留放行」；残余形状既无 proof、无拒绝，也无债务/边界登记（SD-04 面外边界清单与 reference-audit 均未收录 receiver 直发非 len 面）。`v.is_null()` 是可编译旁路（真实 producer 分歧 int/bool 不可见），属 R5-02 同缺陷类。
4. （簿记）`738-phase3-reference-audit.{md,json}`（T-11 合同工件）未随 R5-02 更新 receiver 面与形态守卫边界（代码注释+verification §7 有，审计工件无）。

### 2.3 修复方向（不降 AC）

- json 值 receiver 的直发方法调用（def-use/类型判据命中）按 `Owner.method` 归属并经真实 producer（`a2r_std::json::<name>`）对拍验证；无法证明的可编译形状（如 is_null 固有方法旁路）要么建立适配契约要么拒绝，不得 exit 0。
- 模块限定形状在裸名公共面未命中时**落穿**到 `public_method_symbol` 方法归属（带 FQN 的发射文本已在手）。
- SD-04/审计/交接按最终行为重写（若选择缩面必须如实登记边界且不得违反 R5「不能仅写债务保留放行」）；shape② value_to_int 腿补 to_int→as_int 别名或移除死腿。

## 3. R5-03（真实消费者三角）——闭合确证

- `cargo test -p auto-lang --lib t12_manifest_identity -- --test-threads=1` = **2/2**：
  - 三目标（Vm/Rust/C）CLI actual（真实 `compile_actual_references`：VM=真实 Codegen compile_stmt 收集、Rust=emit_rust_assembly、C=emit_c_assembly）↔ 真实会话入口（`trans_rust_with_session`/`trans_c_with_session` 均 `.expect(...)` **Result 断言成功**且产物非空断言）——共同 `fingerprint()` 全等、`consumer_fingerprint()` 互异，逐 target 断言。
  - C 腿 fixture 差异（六核心无 C json provider → 本地 proto 模块）在测试注释**如实记录**，未删消费者；零 clone 充当生成消费者（局部身份性质测试 `identity_invalidates...` 保留为投影性质验证，属 R5 允许保留面）。
- `cargo test -p auto-man --lib generation_consumer_identity -- --test-threads=1` = **1/1**：真实 `generate_api(&project,"rust")` + `current_generated_api_assembly`（断言 endpoint body 的 parse proof 在场）↔ 同 api.at 的 CLI actual——可比装配面逐项相等（非 consumer_input 源、target/environment/features/providers JSON、references 数、parse declaration_hash+public_signature）；**Embedded/Standalone producer 运行形态差异显式钉死**（断言 gen producer 前缀 `crates/auto-lang/src/a2r_std.rs` vs CLI `crates/a2r-std/src/`）。
- 备注（观察，非阻塞）：persistent VM 入口不在新三角内——修复声称面（三目标 CLI↔会话+真实生成腿）已如约交付；persistent 装配由 T-03 `vm_two_entry_same_layer_visibility` 与 T-05 persistent 指纹台账覆盖。

## 4. 回归面（全部 68398d2d6 实跑）

| 门禁 | 结果 | 分诊 |
|---|---|---|
| `cargo t plan738 --no-fail-fast` | **72/72** | 与声称一致（含 R5 新 5 测） |
| `cargo t plan724 --no-fail-fast` | **5/5** | — |
| `cargo test -p auto --bin auto stdlib -- --test-threads=1` | **10/10** | — |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | **44/44**（1 ignored=⑤腿按需） | 与声称一致 |
| freshness 族 | 双真值表独立 ok + 状态机 1/1；shell_pack 预存环境红 | 同 R3/R4 在案 panic |
| `cargo nextest run -p auto-lang --lib --features test-trans --no-fail-fast` | 5557 run/15 fail | 14 全基线（musk×6/desktop_protocol/iced×2/ash_leak/plan484_024/plan502/plan707_client/plan606_029）+ **plan498_bar_group_emphasis 新 flake 名**（隔离复跑 0.305s PASS）——零新增确定性红；「receiver 初版 6 语料红经守卫归零」在最终树确证（语料族无非基线红） |
| `cargo tv` | **162/162** | — |
| 裸 `cargo t --no-fail-fast` | 5186 run/17 fail | = R5 在案 16 红基线 + plan484_024 flake——零新增确定性红 |
| 三 crate check | 零 error | — |

## 5. SD/对账复核

- **SD-05 ✓**：一次绑定状态机语义（absent→首物化判新鲜+写回收据→严格比较；漂移/删除/读取失败/缺字段陈旧）与代码一致。
- **SD-04 ✗**：receiver 全路径/「未证明=SIGNATURE_UNVERIFIED，不产出成功产物」与实际行为不符（§2.2）——随 R6-01 修复或如实缩面。
- acceptance ADAPTER_RULES 17→15 勘误清零 ✓（全文仅「15 条」，无 17 残留）。
- §9 修复交接与 T-11..T-14 重闭记录 vs 实际 diff：6 文件触面一致；R5-01/R5-03 记录与实现相符；R5-02 记录过度（§2.2.3）。

## 6. Findings

| id | 级别 | 内容 | 锚点 |
|---|---|---|---|
| P738-R6-01 | high（必修） | receiver 公共方法面仅 len（+value_len 扁平 helper）入 strict 门；`v.keys()`/`json.keys(v)`/`v.is_null()`（可编译旁路）/`v.as_int()` 漂移后实际 CLI --check 仍 exit 0 无 proof；shape③ len-only、模块限定方法裸名未命中不落穿方法归属、value_to_int 腿死码；SD-04/T-11 重闭/交接过度声明；reference-audit 未更新 | trans/rust.rs collect_core_method_reference（shape③ len-only）+ collect_core_reference 模块分支；host.rs public_method_symbol；SD-04；AC-03/05/07/08，T-11/T-13/T-14 |
| P738-R6-02 | low（观察） | `bind_workspace_lock` 忽略 `fs::write` 错误——收据持续不可写时 absent→fresh 分支可在漂移内容上重复触发（仅退化工件 FS 角落；再生臂同文件亦不可写）。建议写失败按陈旧处理或注记 | rust_ui.rs L4044-4051 |
| P738-R6-03 | low（观察） | tt 全档 `plan498_bar_group_emphasis` 为未在册 flake 名（隔离绿）——建议并入 verification §2 flake 族名单备后续轮对账 | 本报告 §4 |

## 7. 判定与路由

- R5-01 闭合 / R5-03 闭合 / R5-02 **部分闭合** → 整体 **needs_fix**；AC 快照：AC-01/02/04/06 pass（R6 确证），AC-03 fail、AC-05/07/08 受 R6-01 拖累。
- state: **executing**；重开 T-11（R5-02 面）/T-13（SD-04）/T-14（最终收据随新修复提交重跑）；T-01/T-09/T-10/T-12 维持闭合；current_step=4（R6 口径计数）。
- next: `/auto-plan:work PLAN-738` 在原 worktree 修复 P738-R6-01（顺带裁量 R6-02/R6-03），重跑受影响门禁+CLI 反例，更新 SD-04/reference-audit，新最终提交后再独立 review。不合入/不归档/不删 worktree。
