# PLAN-824 T-11 — 852 统一能力验证 binding 与证据

V-15 / AC-11 的消费记录（r7 合同 §5、[共同合同](../../../reports/ace-852-baseline-and-capability-validation-2026-10-26.md) §3/§4）。
本目录只登记 824 的 binding/证据；通用合同/金样/binding schema 未改动
（`binding.tsv` 仅翻 824 自有的 D 行 av 格 + 追加证据指针）。

## 输入身份（T-11 消费时点）

| 项 | 值 |
|---|---|
| 执行 profile | `av.dynamic/1`（id 5） |
| wire | `AVAD` v4（`ABC_DYNAMIC_VERSION=4`，`ABC_PROFILE_DYNAMIC=5`） |
| 语义版本 | MIR=1 / ABC=1 / AV=1（`*_DYNAMIC_SEMANTICS`） |
| producer | `plan824/av-dynamic` |
| 正式 recipe | `ac-av/dynamic-run-v1`（REGISTERED_PROBE_RECIPES 在册） |
| 底座 | D852 = archived 8/8（landed 4a81d2ebc，ledger v95，早于本树基面 c6c5ddb25） |
| 脚本身份 | `scripts/verify-av-dynamic-824.ps1` 新增 `-Scope baseline`（本任务建立；acceptance 最终包含） |
| a2r | 本计划零 a2r 认证操作、零 Rust 重编译/spawn（既有有效认证按内容/依赖复用；dyn 无 a2r 面） |

## scenario → 实际起点 → V 子例 → 正式 recipe → 独立 oracle → 证据

### D01–D09（824 自有行，binding.tsv av 格已翻 `ok:av-dyn-824`）

| ID | 实际输入起点 | V 子例 | 正式 recipe / 入口 | 独立 oracle | 证据（测试二进制, selected>0） |
|---|---|---|---|---|---|
| D01 Dyn正确take | typed fixture `first-case`（未信任 builder→真实 verify/ABC 门） | V-01 | `ac-av/dynamic-run-v1` `dynamic:first-case`（语料 av-dynamic-first-case） | 父侧冻结表 result=9/allocs[3,0]/frees[1,0]/live=0 | `av_dynamic_824_corpus_green`（runner 3/3） |
| D02 错型take保源 | typed fixture `take-probe`（本任务新增冻结面）+ trap fixture `take-mismatch` | V-01/V-12 | 语料 av-dynamic-take-probe；trap 面经 probe 首例 | is=false 永不 trap + take 复原 n=7；trap=`av.trap.dyn-type` 源保持 | `av_dynamic_824_corpus_green` + `av_dynamic_824_semantics_is_false_never_traps_dyn_usable` + `av_dynamic_824_first_case` trap 臂 |
| D03 名义身份 | typed MIR（Note vs 同宽异名 Alias） | V-11 | probe 定向（typed 轴无源码入口） | is(Alias)=false、take(Alias) trap、再 take(Note) 成功 | `av_dynamic_824_semantics_same_width_different_name_unequal` |
| D04 四独立值pack | corpus + harness + entry-param 宽标量 | V-01/V-03 | 语料 first-case（record/List/Opt）+ harness 全白名单往返 | 父侧冻结面 + roundtrip Value/DynType eq + 位型断言 | `av_dynamic_824_corpus_green` + parallel/824 harness 24 例 + semantics 位型族 |
| D05 递归白名单 | typed MIR 负控（List<Fn>/RecFn/RecP/Place 槽/RecDyn） | V-12 | probe 定向（类型门+白名单双门） | `mir.type.list-elem`/`record-field-type`/`field-borrow`/`mir.dyn.whitelist` 各自拒绝 | `av_dynamic_824_semantics_list_of_fn_not_declarable` 等 5 例 |
| D06 owned树冻结/恢复 | dyn-out Dyn 结果 → A24 store → 同 store unfreeze | V-08 | `dynamic:dyn-out` fixture + restore 缝 | 恢复值 eq + 冻结字节逐位一致 + types-fp 轴 miss | `av_dynamic_824_first_case::av_dynamic_824_restore_reverifies` |
| D07 载荷变异+外hash | 好信封坏 ABC 字节 / 重算 checksum 的坏结构 | V-08 | probe 缓存档 | 结构检查先于/独立于 checksum，绝不执行 | `av_dynamic_824_cache_good_envelope_bad_payload_rejected` + `av_dynamic_824_cache_portable_good_checksum_bad_structure_rejected` |
| D08 revision/权限失效 | R14/R15/R16 伪造 + 会话授权生命周期 | V-11/V-13 | probe 缓存/增量档 | types-fp/entry/profile 双指纹具名拒绝；无授权拒绝先于生产（计数=0）；撤销后再拒 | `..._restore_payload_*_rejected` ×3 + `av_dynamic_824_incremental_signature_and_grant_lifecycle` |
| D09 共享DAG/循环拒绝 | List 双引用同 record Rc | V-04/V-12 | probe 语义档 | pack 过、freeze 拒 `portable.unsupported(shared-graph)`、take 复原 | `av_dynamic_824_semantics_shared_dag_pack_ok_freeze_rejects` |

### W01–W05（av 格 `ok:av-data-typed` 归 820 typed 通道；824 追加同轴动态证据）

| ID | 824 起点 | oracle | 证据 |
|---|---|---|---|
| W01 i64 MIN / W02 u64 MAX | av.dynamic/1 entry-param 缝（宽标量常量为 data profile 专属——实测发现并记录的边界） | 往返 take 位/值等 + 生产冻结=模块冻结字节 | `av_dynamic_824_semantics_int_extremes_and_id_collision_stay_integers` |
| W03 f64 -0 / W04 f64 NaN payload | 同上 | `to_bits()` 逐位等（±0 不合并） | `av_dynamic_824_semantics_f64_bits_preserved_through_dyn` |
| W05 f32 NaN payload | 同上 | f32 位型不宽化 | `av_dynamic_824_semantics_f32_bits_preserved_through_dyn` |

### I04–I06 / I10–I12（增量/会话轴，824 动态证据指针已追加）

| ID | 824 起点 | oracle | 证据 |
|---|---|---|---|
| I04 type/layout 改变 | DynamicUpdateSession 签名改 + 布局/revision 准入拒 | link 拒 `signature mismatch`；layout-mismatch/unknown-type 准入拒 | `av_dynamic_824_incremental_signature_and_grant_lifecycle` + semantics/cache 档 |
| I05 改回历史内容 | A23 内容寻址 restore 身份/miss | 同内容同结果；依赖轴变化 clean miss（历史内容命中=同 store 内容寻址，852 标量证据在案） | `av_dynamic_824_cache_compat_restore_identity_and_miss`（partial：动态历史命中轴随共享 store） |
| I06 fresh/hot/重启/跨工程 | fresh release worker ×4 父侧采样 | B-line 128MiB + live=0 + portable round 绿 | `av_dynamic_824_resources_worker_memory_hard_gate_parent_sampled` + 语料 worker faults |
| I10 512修订churn | 512 次 body 修订经真实会话 | 语义逐修订正确、live=0、alpha 对象携带、会话单位数有界 | `av_dynamic_824_incremental_churn_512_bounded`（实测 512 修订, worst 0.2ms/修订） |
| I11 单函数编辑（852 原 P95 字段为历史） | 同上逐修订计时 + 30 修订 formal campaign | 每修订 <1000ms 独立门保留；campaign 当前为 nearest-rank P90≤2000ms 双门，P95 仅诊断 | 同上 + `av_dynamic_824_incremental_baseline_campaign_replay` |
| I12 灵敏度负控 | expect-sabotage/worker-fault/admission 负控 | 破坏缝必红/infra 归类 | `av_dynamic_824_negative_oracle_fails` + `..._worker_faults_are_infra` + admission negatives |

## 保留的 M1 缺口（不冒领，owner 在案）

- `ac`（共享 MIR native）：`gap:ac-composite-native` —— 复合 native 能力包未交付；824 仅保证 mn 侧受控拒绝（T-05 补臂修复后 818 全绿）。
- `avm`/`a2r`：`gap:avm-no-dyn-api` / `gap:a2r-no-dyn-api` —— Dyn 无源码 API（本计划非目标）。
- 源码行：av.dynamic/1 无 Auto 源码面（typed fixture 为唯一真实起点）——源码行保持独立缺口，owner=未来前端能力包；typed 证据只关闭 MIR/ABC/AV 责任行（共同合同 §3 红线）。
- 共同 Dyn/portable 源码 API、复合 CTE、native/FFI：不在本计划（r7 边界）。

## 执行收据

见 [raw/baseline-receipts.md](raw/baseline-receipts.md)（命令、selected/passed、退出码；`-Scope baseline` 全绿）。
