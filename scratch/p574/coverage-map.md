# P574 T1 路径对账表(2026-09-06)

## 裁定引用
572 待澄清②用户裁定(2026-09-06):avm+aavm/avm+aa2r 双重解释器路径
非真实需求(2×2 对称性设计产物);真实自举=a2r 转译+编译+运行
(⑤腿/at_mode/P532 gen2);处置=关闭重型双重解释器测试或仅留最小锚。

## 关键实证(T1 定型依据)
1. **12 个爆栈测试全部 = `run_with_capture(470KB 完整 lib 拼合 + 用例)`**
   进程内形态(宿主 VM 解释 merged 单元)。
2. **001_smoke 用例本体 = 单行 `print("aavm2 smoke ok")` 仍爆栈** →
   爆点在 merged lib 的编译/装载/解释启动,与用例深度无关——
   **路径级爆栈,最小锚裁剪不可行**(裁剪语料零收益)。
3. 对照:`test_aavm2_a2r_main_dump_print`(曾误作"小用例可通过"反例)
   实为纯宿主转译打印,不合并 lib 不跑 VM——不构成反例。
4. CI(Linux)长期绿(vm-files-ci 常态跑)→ 路径缺陷是 Windows 栈
   环境性,与裁定定性一致(路径非真实需求 + 双层解释递归结构性)。

## 处置(优于原预案)
**12 测试统一 `#[cfg_attr(windows, ignore = "...")]`**:
- Windows:关闭(裁定主诉;本地裸 taa 门禁 13 红 → 1);
- Linux/CI:保留全量(零覆盖损失——优于语料裁剪);
- 注记统一引用裁定。

## 逐测试对账(12/12)
| # | 测试 | 文件:行 | 路径 | 正统替代(保留在位) |
|---|---|---|---|---|
| 1 | test_aavm2_m1_lexer_corpus | aavm2_m1.rs:71 | avm+aavm | ⑤腿 compile_corpus(token 流经编译面) |
| 2 | test_aavm2_m2_parser_corpus | aavm2_m2.rs:58 | avm+aavm | ⑤腿(AST 正确性经编译+运行隐覆) |
| 3 | test_aavm2_m3_typeinfo_corpus | aavm2_m3.rs:48 | avm+aavm | ⑤腿 |
| 4 | test_aavm2_m4_codegen_corpus | aavm2_m4.rs:212 | avm+aavm | ⑤腿 compile_corpus 58/58(同语料!) |
| 5 | test_aavm2_m4_use_corpus | aavm2_m4.rs:616 | avm+aavm | ⑤腿 compile_use_corpus |
| 6 | test_aavm2_p532_lib_static_diff | aavm2_m4.rs:429 | avm+aavm | P532 gen2 管道(lib 静态差分后继=代际对拍,572 落地) |
| 7 | test_aavm2_m5_engine_corpus | aavm2_m5.rs:49 | avm+aavm | ⑤腿(ev_run 行为=转译版运行对拍) |
| 8 | test_aavm2_m5_use_corpus | aavm2_m5.rs:87 | avm+aavm | ⑤腿 compile_use_corpus |
| 9 | test_aavm2_m5_use_errors | aavm2_m5.rs:130 | avm+aavm | ⑤腿(errors 面=编译错路径,宿主侧守护) |
| 10 | test_aavm2_m3_milestone_fib | aavm2_m5.rs:173 | avm+aavm | T3 嵌套塔档(cargo t3,里程碑级) |
| 11 | test_aavm2_001_smoke | aavm_runner_tests.rs:198 | avm+aavm | ⑤腿冒烟(exe¹ 跑 corpus 首件) |
| 12 | test_aavm2_a2r_is_corpus | aavm2_a2r.rs:78 | avm+aa2r | a2r+aa2r:⑤腿 harness --trans + goldens_check(同发射面)+ P532 固定点 |

同路径未爆同胞(不动):m2_rust_dump_print/m4_use_harness_selfcheck/
m4_*_disasm_print(纯宿主腿);a2r probe_smoke/corpus_rustc(#[ignore]
rustc 档);fourpath_runner(#[ignore] 验收档——T4 注记 VM 内解释腿
按裁定口径);002_hello_compile(已 #[ignore],编译管线形态)。
