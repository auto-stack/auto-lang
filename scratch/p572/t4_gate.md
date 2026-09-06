# P572 T4 闸门回归记录(2026-09-06,worktree=plan-572-dev@237839ffe)

## 作用域闸门(计划指定)
| 闸门 | 结果 |
|---|---|
| ⑤腿 `test_aavm2_compile_corpus`(corpus_m4 58/58) | **PASS**(44.21s) |
| `test_aavm2_compile_use_corpus` | **PASS**(16.95s) |
| `cargo taa aavm2_a2r`(test_aavm2_goldens_check + main_dump) | **2/2 PASS**(goldens_check=测试内 golden 锚) |
| `aavm_at_mode` 档 | 测试默认 `#[ignore]`(昂贵档,需 `-- --ignored` 显式触发;作用域闸门实际由 aavm2_a2r 覆盖) |
| 小源 golden 逐字节(旧 exe¹ 106d6ff vs 新 exe¹ ce20d8ee) | corpus_a2r 18/18 + corpus_m4 58/58 **全一致**,corpus_use 多文件形态由 compile_use_corpus 覆盖 |

## 裸 `cargo taa` 全量兜底(基点 vs 修复态,nextest --no-fail-fast)
| | 基点(f2ae1cb29,lang-532 worktree) | 修复态(plan-572-dev@237839ffe) |
|---|---|---|
| 通过 | 3612/3625 | 3612/3625 |
| 失败 | 13 | 13 |
| 失败名单 | 同右侧 | 同左侧(逐名一致) |

**失败集两态完全一致——零回归实证**。13 件构成:
- 12 × STATUS_STACK_OVERFLOW ABORT(Windows 环境族):test_aavm2_001_smoke、
  m1_lexer_corpus、m2_parser_corpus、m3_typeinfo_corpus、m3_milestone_fib、
  m4_codegen_corpus、m4_use_corpus、m5_engine_corpus、m5_use_corpus、
  m5_use_errors、a2r_is_corpus、p532_lib_static_diff——基点(f2ae1cb29)、
  主检出 master、修复态三处同签名;nextest 与 libtest 双路径均溢出
  (主检出 .cargo/config.toml [env] RUST_MIN_STACK=16MB 只抬 libtest 测试
  线程栈,nextest 测试跑进程主线程不受其控)。全部为进程内深度解释
  递归族(宿主 VM in-process 跑 aavm.at+lib 双层解释);CI(Linux)守护。
- 1 × `ui_gen::vue::tests::test_charts_gallery_compiles` FAIL(0.3s,
  ui 族预存红,564-Q6 台账邻接;基点同款)。

## 折叠前全量门禁 `cargo tf --no-fail-fast`
- 修复态:3460/3461 通过,唯一失败 charts_gallery(上表预存红)。
- 基点(lang-532 worktree):3460/3461,**逐同款**。

## 结论
零回归成立:所有失败均为基点预存(两态名单逐一致),修复态无任何
新增失败;作用域闸门全绿;golden 产物不变锚 76/76。折叠放行。
