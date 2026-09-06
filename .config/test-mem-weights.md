# 逐测峰值内存权重表（Plan 564 T3,单一事实来源）

测量方式:`python scripts/measure_test_mem.py <filter>`(nextest --jobs=1 串行,
Win32_Process WorkingSetSize 轮询,取每测试进程峰值)。复测:同命令重跑,
漂移 >50% 的行脚本自动标注 **DRIFT**。

- 测量日期:2026-09-05(lang-564 worktree,plan-532-dev tip @00575a02a)
- 档位阈值(D2):XL ≥800MB / LG 300-800MB / MD 100-300MB / LT <100MB
- 组配置(数据修正后,见 nextest.toml):mem-xl=1 / mem-lg=1 / mem-md=2
  ——原设计 lg=2 被实测否决:931(XL 并发 1)+2×~780(LG)≈2.5GB 破 2GB 预算。

## XL(≥800MB)——日常档 default-filter 排除 + heavy_gate 守门

| test | peak_MB | 备注 |
|---|---|---|
| `tests::aavm2_m4::test_aavm2_p532_lib_static_diff` | 1232 | #[ignore] 挂账态;测量期 FAILED(P532 债,canon 4506) |
| `tests::aavm2_a2r::test_aavm2_a2r_fourpath_runner` | 992 | #[ignore] |
| `tests::aavm2_a2r::test_aavm2_a2r_is_corpus` | 931 | |
| `tests::aavm2_m4::test_aavm2_m4_codegen_corpus` | 815 | |
| `tests::aavm2_m5::test_aavm2_m5_engine_corpus` | 814 | |
| `tests::aavm2_m2::test_aavm2_m2_parser_corpus` | 813 | |
| `tests::aavm2_m5::test_aavm2_m5_use_corpus` | 811 | |
| `tests::aavm2_m4::test_aavm2_m4_use_corpus` | 810 | |
| `tests::aavm2_m3::test_aavm2_m3_typeinfo_corpus` | 807 | |
| `tests::aavm2_m1::test_aavm2_m1_lexer_corpus` | 806 | |
| `tests::aavm2_m5::test_aavm2_m3_milestone_fib` | 802 | |
| `tests::aavm2_a2r::test_aavm2_a2r_probe_smoke` | 805 | #[ignore] |

## LG(300-800MB)——保留日常档,组内串行 + heavy_gate 守门

| test | peak_MB |
|---|---|
| `tests::aavm2_m5::test_aavm2_m5_use_errors` | 797 |
| `tests::vm_file_tests::test_aavm2_001_smoke` | 755 |
| `tests::vm_file_tests::test_aavm2_compile_corpus` | 745 |
| `tests::vm_file_tests::test_aavm2_compile_use_corpus` | 744 |

## MD(100-300MB)——全档运行,组内并发 2,不守门

| test | peak_MB |
|---|---|
| `tests::aavm2_a2r::test_aavm2_goldens_check` | 138 |
| `tests::aavm2_a2r::test_aavm2_a2r_main_dump_print` | 135 |
| `tests::aavm2_a2r::test_aavm2_a2r_corpus_rustc` | 128 | #[ignore] |
| `tests::vm_file_tests::test_aavm2_002_hello_compile` | 123 |

## LT(<100MB)——默认池

| test | peak_MB |
|---|---|
| `vm::tests_rc_lifecycle::str_churn_bounded` | 20 |
| `vm::tests_rc_lifecycle::str_churn_bounded_large` | 20 | 1M churn 实测无驻留,无需门控(修正预设) |

## 未采样(运行时长低于 120ms 采样窗口,推断 LT)

`repro_242_string_pool_uaf` / `repro_d30_negative_int_roundtrip` /
`m2_rust_dump_print` / `m4_rust_disasm_print` / `m4_use_rust_disasm_print` /
`m4_use_harness_selfcheck`——均在 RUN1 全量串行中通过且未被采样捕获,秒级轻测。

## 塔级(不参与本表)

`tests::aavm2_t3::test_aavm2_t3_tower_milestone`——T3_MILESTONE env 自守门
+ 双档 filter 排除(Plan 532),小时级,内存形态另行评估。
