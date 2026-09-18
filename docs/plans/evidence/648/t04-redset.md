# PLAN-648 T-04 全量门红集定界(2026-09-19)

命令:`cargo nextest run -p auto-lang --lib --test schema_drift --test docs_gen
--test component_registry_test --config-file .config/nextest-full.toml
--no-fail-fast`(= cargo tf 全量档,补 --no-fail-fast 取完整红集)。

| 树 | 测试数 | 红 | 明细 |
|---|---|---|---|
| master 4817b51e1(主检出) | 3643 | 3 | ffi_dual_019_dep_layout_invariants;mouse_area_emits_events_and_logical_extent;test_display_family_codegen_arm_fixture |
| plan-648-dev 6422eb267(worktree) | 3641 | 1(run2)/2(run1 fail-fast) | test_display_family_codegen_arm_fixture(两轮均在);ffi_dual_019(run1 现 run2 消) |

定界结论:
1. **worktree 红集 ⊆ master 红集,零新增红**;两树共同稳红仅
   display_family(单测隔离 4/4 过=并行状态敏感闪红)。
2. ffi_dual_019(装载顺序敏感,"先后装载布局各归各"断言)与
   mouse_area/display_family 同为并行 flaky 家族——与
   64b65529b 记录的 "tf 3611/3613 双红主检出现存对照复证" 同谱系。
3. **E0433 消除实证(T-02/AC-03)**:tf 全特性组合编译通过且可跑
  (3641 全部装载执行)——plan024 门控修复(64b65529b ⑤)+本计划
   零 E0433。
4. 日常档(cargo t)musk_vm_track 3 红=master 同滤串同败(P648-D2,
   P028-D4 家族 fail-fast 子集)。

处置:闪红家族不阻塞(基线在案);display_family/mouse_area/
ffi_dual_019 的并行稳定性归测试基建域后续(候选:状态隔离或串行组),
非 648 改动面(4 文件:vm/codegen、ui/handler_codegen、vm/ffi/stdlib、
plan340_tests + auto-man rust_ui)。
