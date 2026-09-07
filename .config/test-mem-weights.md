# 逐测峰值内存权重表（Plan 564 T3,单一事实来源）

测量方式:`python scripts/measure_test_mem.py <filter>`(nextest --jobs=1 串行,
Win32_Process WorkingSetSize 轮询,取每测试进程峰值)。复测:同命令重跑,
漂移 >50% 的行脚本自动标注 **DRIFT**。

- 测量日期:2026-09-05(lang-564 worktree,plan-532-dev tip @00575a02a)
- **2026-09-07 复测(Plan 565 L1/T6,lang-565 worktree,master tip 1c6753a92)**:
  m1/m2/m3/m4 语料闸门切 once-compiled runner 后重测(下行 *L1* 标注);
  未标注行沿用 564 值(代码未触及;且 574 后 aavm 系 Windows 本地
  cfg_attr(windows,ignore),日常档/nextest 默认不跑,CI/Linux 全量)。
  **结论:单测峰值≈单次编译高水位(~790-815MB),L1 消除的是重复
  (m2 147s→2.98s,churn 35×→1×),不降单编译峰值——峰值本体是
  编译侧 AST/Node 分配结构(565 P0 归因:编译侧 churn 7.4GiB vs
  执行侧 2.9MiB),压缩属 Plan 566 Value/Node 领地。**
- **2026-09-07 终测(Plan 566 T6,lang-566 worktree,plan-566-dev 装箱
  三阶段全落地)**:Value 296B→40B(Node/Obj/Instance/Widget/Meta/Fn/
  ExtFn/Type 等 13 胖变体装箱)后 XL/LG 十测复测(下行 *P566* 标注),
  **全部落 784-836MB 噪声带(-3%~+4%),零降幅**——566 实证推翻 565
  上述"压缩属 Plan 566 Value/Node 领地"预判:编译侧 AST(auto-lang
  Code/Stmt/Expr 家族)零 Value 字段,峰值与 Value 表示正交;真正杠杆
  =编译器 AST 自身瘦身(独立立项领域)。Value 密度收益(7.4×)落
  VM 运行时栈/ListData 等密集 Vec<Value> 面(本族测试里 footprint
  ~2.9MiB,不显峰)。未复测行(#[ignore] 挂账/is_corpus 族)沿用前值。
- 档位阈值(D2):XL ≥800MB / LG 300-800MB / MD 100-300MB / LT <100MB
- 组配置(数据修正后,见 nextest.toml):mem-xl=1 / mem-lg=1 / mem-md=2
  ——原设计 lg=2 被实测否决:931(XL 并发 1)+2×~780(LG)≈2.5GB 破 2GB 预算。
  (565 注:m3 792/m4 796 复测落 LG 边界(±2% 噪声带),nextest 组归属
  维持原 XL 不动——xl/lg 并发同为 1,无功能差异,保守留组。)

## XL(≥800MB)——日常档 default-filter 排除 + heavy_gate 守门

| test | peak_MB | 备注 |
|---|---|---|
| `tests::aavm2_m4::test_aavm2_p532_lib_static_diff` | 1232 | #[ignore] 挂账态;测量期 FAILED(P532 债,canon 4506) |
| `tests::aavm2_a2r::test_aavm2_a2r_fourpath_runner` | 992 | #[ignore] |
| `tests::aavm2_a2r::test_aavm2_a2r_is_corpus` | 931 | |
| `tests::aavm2_m5::test_aavm2_m5_engine_corpus` | 814 | *P566*(复测 836,+3% 噪声) |
| `tests::aavm2_m2::test_aavm2_m2_parser_corpus` | 806 | *L1*(564:813;时长 147s→2.98s,35 文件);*P566* 复测 824(+2%) |
| `tests::aavm2_m5::test_aavm2_m5_use_corpus` | 811 | *P566*(复测 832,+3%) |
| `tests::aavm2_m4::test_aavm2_m4_use_corpus` | 810 | *P566*(复测 833,+3%) |
| `tests::aavm2_m1::test_aavm2_m1_lexer_corpus` | 809 | *L1*(564:806);*P566* 复测 814(+1%) |
| `tests::aavm2_corpus_runner::test_aavm2_corpus_runner_rerun_consistency` | 813 | *新增(565 L1 回归:缓存 VM 重入无串台+两批一致)*;*P566* 复测 791(-3%) |
| `tests::aavm2_m5::test_aavm2_m3_milestone_fib` | 802 | *P566*(复测 820,+2%) |
| `tests::aavm2_a2r::test_aavm2_a2r_probe_smoke` | 805 | #[ignore] |

## LG(300-800MB)——保留日常档,组内串行 + heavy_gate 守门

| test | peak_MB | 备注 |
|---|---|---|
| `tests::aavm2_m5::test_aavm2_m5_use_errors` | 797 | *P566*(复测 830,+4%,跨 XL 阈——组归属维持 LG,xl/lg 并发同为 1) |
| `tests::aavm2_m4::test_aavm2_m4_codegen_corpus` | 796 | *L1*(564:815,XL→LG;58 文件,原 ~315s 闸门→秒级);*P566* 复测 784(-1%) |
| `tests::aavm2_m3::test_aavm2_m3_typeinfo_corpus` | 792 | *L1*(564:807,XL→LG);*P566* 复测 823(+4%,跨 XL 阈,组归属同上保守留 LG) |
| `tests::vm_file_tests::test_aavm2_001_smoke` | 755 | 568 迁出至 aavm_runner_tests,行名沿用 |
| `tests::vm_file_tests::test_aavm2_compile_corpus` | 745 | 同上 |
| `tests::vm_file_tests::test_aavm2_compile_use_corpus` | 744 | 同上 |

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
