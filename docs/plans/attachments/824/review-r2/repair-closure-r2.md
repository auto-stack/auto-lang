# PLAN-824 F03 R2 修复交接与覆盖

- stage: work；plan_revision: 7；worktree `D:/autostack/.wt/lang-824/auto-lang`；branch `plan-824-dev`。
- 被审基线 `6a68240e89bdfd141ecfb893f83c7718bf503056`；代码修复 `2c8b6b0aa5ef5f101e4d0a6b4148b5605245f41b`；回归登记及冻结 HEAD `29b6eeea98d1b75e2ea18ea5f62eea96866e9d60`；提交后工作树 clean。后一个提交只改必测 ID 列表，不改代码/依赖/测试输入；下列测试的代码身份为前者，与冻结 HEAD 代码相同。
- 依赖：auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`、auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`，均 clean；D821/D822/D828/D852 沿集成基线 `c6c5ddb256089e6b628d7651cb9759664d15f22a`，无依赖改动。当前主检出 HEAD 已前进到 `a2ec22aa2784606d11da23de90f40511977dd123`，不把它当作本树新基线；823 编号冲突在实际集成时重核。
- 锁文件 SHA-256：独立 workspace `37c362a201967775bd767d8fba7a2060dea3ff720e35c1d2c6422c9e2f3a8873`，parallel/824 `1e772815c52f2a720bb334465b27c661be208a00801fa4ebd9509a20f4049c6f`。两轮失败诊断见 [repair-diagnosis.md](repair-diagnosis.md)。

## 当前闭合表

| Finding → 不变量 | 路径/状态与修复 | 永久断言、结果和边界 |
|---|---|---|
| F-824-R1-03 临时类型字节/工作 | MIR DAG → pack/is/take 的 `resolve_dyn_type`；节点、名称与容器申请前计量，步骤与取消逐项计，临时占用计入执行数据峰值；落槽类型大小亦计内联结构；pack 准入和类型比较计工作；恢复 binding 在真实类型复制前预检 | [R2 原始输入](probes/src/main.rs) 重放见 [repair-replay.txt](repair-replay.txt)：16011B/65535节点/长名由 Success→SlotBytes、peak 4055/4096；3层合法对照 Success，peak 1867；17层受预算拒绝。永久 `av_dynamic_824_resources_r2_long_names_are_accounted_before_expansion` 含 4096 与默认 8MiB 两侧、`...pack_take_type_operands_bounded` 三入口及合法对照、`...binding_long_names_are_bounded` 与非可擦除声明对照、旧 `...type_dag_expansion_bounded_and_charged`。峰值为逻辑计量，不冒称 RSS；worker 另由父侧采样。 |
| F-824-R1-03 portable 类型轮询与临时数据 | freeze 描述符写入、unfreeze 描述符读取/查找/准入/子类型比较、deep_clone 类型复制均逐项计工作和轮询；编码/解码/副本在分配前检查字节；恢复只持有一份根类型，子描述符比对后释放 | 原 1026 节点输入 freeze/unfreeze 均 Cancelled 且各 polls=1；`av_dynamic_824_r2_type_work_cancel_and_retry` 断言源可重用和恢复重试、`...cancel_in_admission` 断言第三次轮询取消并重试、`...type_clone_budget_cancel_and_retry`、`...restore_descriptor_bytes_before_construction`、`...clone_depth_at_type_limit` 均通过。255层实际值链恢复及 1MiB 栈 256 层类型复制通过。 |
| 已关闭 F01/F02 | 保留标量 kind 门、完整类型依赖准入；由于恢复所有权改变重验受影响路径 | 原三组错例及 bool 对照重放符合 R2；wire 五项、admission 三项、真实 adapter 字段类型重验和坏校验和重算结构拒绝均通过。 |

## 最终代码定向验证

所有 cargo 命令在上述 worktree，锁定独立 manifest；每项 selected>0，0 ignored：

- `cargo test --locked --manifest-path experimental/ac-av-probe/parallel/824/Cargo.toml --lib <filter>`：`av_dynamic_824_r2_` 5、`av_dynamic_824_portable_roundtrip` 4、`av_dynamic_824_wire_` 5、`av_dynamic_824_rollback_` 2、`av_dynamic_824_admission_` 3、`av_dynamic_824_deep_clone_independent_expands_sharing` 1，共 20。
- `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_dynamic_824_resources <filter>`：`av_dynamic_824_resources_r2_` 6；原节点/步骤反例 1；255层深链 1；`av_dynamic_824_resources_worker_memory_hard_gate_parent_sampled` 1（4 fresh workers、父侧 128MiB 阈值和释放，57.06s）。共 9。
- 同一 manifest：`av_dynamic_824_semantics` 精确过滤 restore_revalidates_current_field_types、pack_source_stays_readable、take_share_dyn_still_takeable、is_false_never_traps_dyn_usable、production_same_source_byte_equivalence、same_width_different_name_unequal、shared_dag_pack_ok_freeze_rejects、f64_bits_preserved_through_dyn、f32_bits_preserved_through_dyn、int_extremes_and_id_collision_stay_integers，10/10；first-case `av_dynamic_824_restore_reverifies` 1/1；cache `av_dynamic_824_cache_portable_good_checksum_bad_structure_rejected` 1/1。
- `cargo test -p auto-lang --lib --features auto-lang/test-cases av_dynamic_824_corpus_green -- --test-threads=1` 1/1，正式 producer/父侧 oracle。此根仓构建输出既有 auto-ui、auto-lang-core 与 auto-lang 警告（auto-lang lib test 43 条），不把该命令声称为零警告；本计划增量 test target 的 `cargo check --locked ... -p auto-av-probe --test av_dynamic_824_incremental` 零警告。
- 增量会话 `av_dynamic_824_incremental_body_edit_identity` 1/1；共 **43 个定向测试通过**，未计早期已中止的误选资源族。`pwsh -NoProfile -File scripts/verify-av-dynamic-824.ps1 -GuardSelfTest` 通过；11 个 R2 新测试已登记到对应资源/harness 必测 ID；指定七文件 rustfmt --check 与整个修复 diff --check 通过。

重用 R1/R2 未受影响的 D821/D822/D828/D852、非动态路径兼容、T-09 增量及 852 binding 证据：本轮未修改其输入、锁文件、依赖或相应非动态执行分支；本轮实际改动的 producer、恢复、资源、类型比较、增量会话身份和同源路径按上列重新验证。非本轮的性能历史证据仍保留原判定边界：798 聚合时序曾红、同形基线更红，隔离测试曾过；独立复审应核验该归因，不能把本轮定向绿当作聚合时序通过。

本轮没有执行 `cargo t/tv/tt/tb/tu/taa`、`tf` 或 acceptance/baseline 聚合脚本。T-07 独立复审待执行；复审通过后 merge 负责冻结 HEAD 的裸 `cargo t`、`cargo tv` 和新 workspace 有限合同档，及 823 编号实际集成核对；tf 按到期规则仅主检出单实例。未合入、未归档。范围外 34 个误格式化文件已依用户明确授权恢复；原补丁留存，未进入提交。
