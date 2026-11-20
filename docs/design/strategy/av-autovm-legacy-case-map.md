# AV语言齐平：旧VM文档用例逐项承接快照

2026-11-19，**规划静态清单，非运行结果**。 本轮AV计划只执行AV；非AV原义务仍保映射与原预期，以检查级#ignore/未选暂缓，实际运行对账归后续对应目标，不以其未验阻AV出口。按当前`crates/auto-lang/test/vm/**/*.case.ad`读取506份文档及其case/check/oracle声明，主承接计划如下。源码与完整expected以原文档为唯一作者源，本文不生成expected、不复制可执行语料，也不是新测试注册器。

关联[路线设计](av-autovm-language-parity.md)与[按特性共享语料迁移合同](../testing/ace-feature-organized-autodown-corpus.md)。主承接计划只表示工作归属，长期作者位置按language特性目录决定，不把本表按Plan的阅读分组变成测试目录。下面506份是原文档快照，不是去重后Case数；去重后每条原case/check/条件仍通过legacy-map追溯，不能删义务。旧A2R/其它适用族由1078另核，不以本VM快照代全量。此处owner是根据真实源目录、case主体与新计划scope作的**规划分派**；AVP-00/T-01需结合生产discover、原1066义务、Rust内联/外部语料逐键复核，纠正一对多角色但不得丢行。当前AV是否支持均未逐例实跑，不能把本表当506项缺失或506项已通过。

已知注意：旧weak error-presence/smoke无精确数值的用例须补独立oracle；历史旧bug不照搬。外库API角色交对应stdlib/provider owner，语言主owner仍保映射/真实消费；AAVM研究项保既有显式手动/ignore，不充日常PASS。

| 计划 | 静态主承接文档数 | 当前状态 |
|---|---|---|
| [PLAN-1078](../../plans/1078-av-parity-baseline-and-autodown.md) | 2 | 待AVP-00核角色，未执行 |
| [PLAN-1079](../../plans/1079-av-parity-scalars-and-text.md) | 68 | 待AVP-00核角色，未执行 |
| [PLAN-1080](../../plans/1080-av-parity-calls-and-pipeline.md) | 10 | 待AVP-00核角色，未执行 |
| [PLAN-1081](../../plans/1081-av-parity-function-values-and-closures.md) | 9 | 待AVP-00核角色，未执行 |
| [PLAN-1082](../../plans/1082-av-parity-collections-and-places.md) | 62 | 待AVP-00核角色，未执行 |
| [PLAN-1083](../../plans/1083-av-parity-records-variants-and-patterns.md) | 52 | 待AVP-00核角色，未执行 |
| [PLAN-1084](../../plans/1084-av-parity-dynamic-objects-and-null.md) | 11 | 待AVP-00核角色，未执行 |
| [PLAN-1085](../../plans/archive/1085-av-parity-modules-and-global-state.md) | 9 | AV 9/9，原 VM 检查保留为阶段 #ignore |
| [PLAN-1086](../../plans/archive/1086-av-parity-borrows-moves-and-storage.md) | 28 | 主承接 28 条与接续 11 条原例已通过 AV。VM 检查保留为阶段 SKIP |
| [PLAN-1087](../../plans/archive/1087-av-parity-errors-and-unwinding.md) | 25 | 同源 AV 25/25 已判定，含具名差异。保留原 VM 27 条检查为阶段 SKIP |
| [PLAN-1088](../../plans/1088-av-parity-generics-specs-and-methods.md) | 7 | 待AVP-00核角色，未执行 |
| [PLAN-1089](../../plans/archive/1089-av-parity-generators-and-stream-lifetimes.md) | 4 | 原源和 VM oracle 已迁入生成器语料，AV 原样通过 |
| [PLAN-1090](../../plans/1090-av-parity-tasks-async-and-actors.md) | 13 | 待AVP-00核角色，未执行 |
| [PLAN-1091](../../plans/1091-av-parity-comptime-and-macros.md) | 0 | 待AVP-00核角色，未执行 |
| [PLAN-1092](../../plans/1092-av-parity-attributes-and-metadata.md) | 0 | 待AVP-00核角色，未执行 |
| [PLAN-1093](../../plans/1093-av-parity-foreign-language-surface.md) | 70 | 待AVP-00核角色，未执行 |
| [PLAN-1094](../../plans/1094-av-parity-script-and-python-protocols.md) | 7 | 待AVP-00核角色，未执行 |
| [PLAN-1095](../../plans/1095-av-parity-ordinary-project-composition.md) | 129 | 待AVP-00核角色，未执行 |
| [PLAN-1096](../../plans/1096-av-parity-full-language-certification.md) | 0 | 待AVP-00核角色，未执行 |

## PLAN-1078 原文档清单

本批主承接2份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.aavm2.001_smoke](../../../crates/auto-lang/test/vm/aavm2/001_smoke/smoke.case.ad) | vm-stdout | 文本 'aavm2 smoke ok' | `4a9bf96f8568ad2e` |
| [legacy.aavm2.002_hello_compile](../../../crates/auto-lang/test/vm/aavm2/002_hello_compile/hello_compile.case.ad) | vm-stdout | 文本 'hello aavm2 compile' | `b5b508996ecbc910` |

## PLAN-1079 原文档清单

本批主承接68份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.01_basics.001_hello](../../../crates/auto-lang/test/vm/01_basics/001_hello/hello.case.ad) | vm-stdout | 文本 'hello' | `b1067d3daf908bb7` |
| [legacy.01_basics.002_arithmetic](../../../crates/auto-lang/test/vm/01_basics/002_arithmetic/arithmetic.case.ad) | vm-result | 文本 '3' | `f57f4c3cf24d98d2` |
| [legacy.01_basics.003_str_upper](../../../crates/auto-lang/test/vm/01_basics/003_str_upper/str_upper.case.ad) | vm-stdout | 文本 'HELLO' | `4a354ee91aba7d9e` |
| [legacy.01_basics.004_uint](../../../crates/auto-lang/test/vm/01_basics/004_uint/uint.case.ad) | vm-result | 文本 '3u' | `ac812869170a144c` |
| [legacy.01_basics.005_byte](../../../crates/auto-lang/test/vm/01_basics/005_byte/byte.case.ad) | vm-result | 文本 '0xFF' | `dca091be97189bab` |
| [legacy.01_basics.006_unary](../../../crates/auto-lang/test/vm/01_basics/006_unary/unary.case.ad) | vm-result | 文本 '-6' | `561fcebd7cb2d16a` |
| [legacy.01_basics.007_group](../../../crates/auto-lang/test/vm/01_basics/007_group/group.case.ad) | vm-result | 文本 '9' | `3869e5051b199eb2` |
| [legacy.01_basics.008_comp](../../../crates/auto-lang/test/vm/01_basics/008_comp/comp.case.ad) | vm-result | 文本 'true' | `b14ffa30eb6782b0` |
| [legacy.01_basics.009_comp_false](../../../crates/auto-lang/test/vm/01_basics/009_comp_false/comp_false.case.ad) | vm-result | 文本 'false' | `85d830cbe8ecf1ed` |
| [legacy.01_basics.010_eq](../../../crates/auto-lang/test/vm/01_basics/010_eq/eq.case.ad) | vm-result | 文本 'true' | `8dbd61a8ecacdff4` |
| [legacy.01_basics.011_eq_false](../../../crates/auto-lang/test/vm/01_basics/011_eq_false/eq_false.case.ad) | vm-result | 文本 'false' | `3ff286b41d2e9ebc` |
| [legacy.02_bit_ops.001_binary_literal](../../../crates/auto-lang/test/vm/02_bit_ops/001_binary_literal/binary_literal.case.ad) | vm-stdout | 文本 '10' | `14434b0f5ae6351e` |
| [legacy.02_bit_ops.002_bitwise_ops](../../../crates/auto-lang/test/vm/02_bit_ops/002_bitwise_ops/bitwise_ops.case.ad) | vm-stdout | 文本 '8\n11\n6\n8\n2\n-11' | `0c79e892ba86b871` |
| [legacy.02_bit_ops.003_bit_scan](../../../crates/auto-lang/test/vm/02_bit_ops/003_bit_scan/bit_scan.case.ad) | vm-stdout | 文本 '2\n28\n1' | `577438529a52d311` |
| [legacy.02_bit_ops.004_not_flip](../../../crates/auto-lang/test/vm/02_bit_ops/004_not_flip/not_flip.case.ad) | vm-stdout | 文本 '-1\n0\n-2\n4\n2\n1' | `85db491410d169f0` |
| [legacy.02_bit_ops.005_bitfield](../../../crates/auto-lang/test/vm/02_bit_ops/005_bitfield/bitfield.case.ad) | vm-stdout | 文本 '5\n1\n0\n10\n2\n8' | `36acc9884786183d` |
| [legacy.08_strings.001_fstr](../../../crates/auto-lang/test/vm/08_strings/001_fstr/fstr.case.ad) | vm-result | 文本 'hello auto, now!' | `ee0d274f938ebf9f` |
| [legacy.08_strings.002_fstr_expr](../../../crates/auto-lang/test/vm/08_strings/002_fstr_expr/fstr_expr.case.ad) | vm-result | 文本 'a + b = 3' | `8db4b8a7bfd53e0d` |
| [legacy.08_strings.003_str_index](../../../crates/auto-lang/test/vm/08_strings/003_str_index/str_index.case.ad) | vm-result | 文本 "'e'" | `306e4c5af7a9341e` |
| [legacy.08_strings.004_int_to_str](../../../crates/auto-lang/test/vm/08_strings/004_int_to_str/int_to_str.case.ad) | vm-result | 文本 '1' | `c6576db2817046b6` |
| [legacy.08_strings.005_str_import_basic](../../../crates/auto-lang/test/vm/08_strings/005_str_import_basic/str_import_basic.case.ad) | vm-stdout | 文本 '5\n1\n0' | `d143833844d97a96` |
| [legacy.08_strings.006_str_import_case](../../../crates/auto-lang/test/vm/08_strings/006_str_import_case/str_import_case.case.ad) | vm-stdout | 文本 'HELLO\nhello' | `e39d423796f64431` |
| [legacy.08_strings.007_str_import_search](../../../crates/auto-lang/test/vm/08_strings/007_str_import_search/str_import_search.case.ad) | vm-stdout | 文本 '1\n1\n1\n6' | `c9bc3abe2e3435e3` |
| [legacy.08_strings.008_str_import_transform](../../../crates/auto-lang/test/vm/08_strings/008_str_import_transform/str_import_transform.case.ad) | vm-stdout | 文本 'hi\nhello rust\nolleh' | `9b839d443fae12e7` |
| [legacy.08_strings.009_char_at](../../../crates/auto-lang/test/vm/08_strings/009_char_at/char_at.case.ad) | vm-stdout | 文本 '104' | `a1ac42c4250c0509` |
| [legacy.08_strings.010_str_ext_import](../../../crates/auto-lang/test/vm/08_strings/010_str_ext_import/str_ext_import.case.ad) | vm-stdout | 文本 '104\n111' | `88a2adda5258aca1` |
| [legacy.08_strings.011_vm_stub_panic](../../../crates/auto-lang/test/vm/08_strings/011_vm_stub_panic/vm_stub_panic.case.ad) | vm-error | 诊断 legacy.error-presence | `46e4ea0a9aea46af` |
| [legacy.08_strings.012_str_eq](../../../crates/auto-lang/test/vm/08_strings/012_str_eq/str_eq.case.ad) | vm-stdout | 文本 'a == b: true\na == c: false' | `cf20b48777bf0465` |
| [legacy.08_strings.013_str_param](../../../crates/auto-lang/test/vm/08_strings/013_str_param/str_param.case.ad) | vm-stdout | 文本 '5' | `de5353f21079db80` |
| [legacy.08_strings.014_raw_str](../../../crates/auto-lang/test/vm/08_strings/014_raw_str/raw_str.case.ad) | vm-stdout | 文本 'raw_str: all passed' | `294a720811d8d85a` |
| [legacy.08_strings.015_multi_fstr](../../../crates/auto-lang/test/vm/08_strings/015_multi_fstr/multi_fstr.case.ad) | vm-stdout | 文本 'Hello World,\nyou are 25 years old.\nWelcome!\nSum of 10 and 20 is 30.\nProduct is 200.\n{"type": "text", "value": "World"}\njust World' | `953e13bb2943ac51` |
| [legacy.08_strings.016_heaprc_string_pool](../../../crates/auto-lang/test/vm/08_strings/016_heaprc_string_pool/heaprc_string_pool.case.ad) | vm-result | 文本 '50000' | `d1e5a9407ed08335` |
| [legacy.08_strings.017_sub_semantics.sub_semantics](../../../crates/auto-lang/test/vm/08_strings/017_sub_semantics/sub_semantics.case.ad) | vm-stdout | 文本 'el\nell\nel\n\no' | `d574a7e4b19c7b0a` |
| [legacy.08_strings.018_loop_concat_churn](../../../crates/auto-lang/test/vm/08_strings/018_loop_concat_churn/loop_concat_churn.case.ad) | vm-stdout | 文本 'res *.tmp\|res *.bak\|res *.log\|' | `5b14869a83472747` |
| [legacy.11_compound_ops.001_add_eq](../../../crates/auto-lang/test/vm/11_compound_ops/001_add_eq/add_eq.case.ad) | vm-result | 文本 '2' | `f5db71c5d3fbde90` |
| [legacy.11_compound_ops.002_sub_eq](../../../crates/auto-lang/test/vm/11_compound_ops/002_sub_eq/sub_eq.case.ad) | vm-result | 文本 '7' | `bf54d4f6664e50d1` |
| [legacy.11_compound_ops.003_mul_eq](../../../crates/auto-lang/test/vm/11_compound_ops/003_mul_eq/mul_eq.case.ad) | vm-result | 文本 '15' | `b1d478dccdfa4171` |
| [legacy.11_compound_ops.004_div_eq](../../../crates/auto-lang/test/vm/11_compound_ops/004_div_eq/div_eq.case.ad) | vm-result | 文本 '5' | `4518c747947444df` |
| [legacy.11_compound_ops.005_chained](../../../crates/auto-lang/test/vm/11_compound_ops/005_chained/chained.case.ad) | vm-result | 文本 '7' | `42c1c981f84f6349` |
| [legacy.11_compound_ops.006_div_eq_oneline](../../../crates/auto-lang/test/vm/11_compound_ops/006_div_eq_oneline/div_eq_oneline.case.ad) | vm-result | 文本 '5' | `398950261125bda3` |
| [legacy.12_type_coercion.001_int_plus_float](../../../crates/auto-lang/test/vm/12_type_coercion/001_int_plus_float/int_plus_float.case.ad) | vm-result | 文本 '5.5' | `60cfbfd6ff4fc42a` |
| [legacy.12_type_coercion.002_float_plus_int](../../../crates/auto-lang/test/vm/12_type_coercion/002_float_plus_int/float_plus_int.case.ad) | vm-result | 文本 '5.5' | `a39b30e189f2b57f` |
| [legacy.12_type_coercion.003_int_times_float](../../../crates/auto-lang/test/vm/12_type_coercion/003_int_times_float/int_times_float.case.ad) | vm-result | 文本 '10' | `da21af94fabbb32e` |
| [legacy.12_type_coercion.004_float_times_int](../../../crates/auto-lang/test/vm/12_type_coercion/004_float_times_int/float_times_int.case.ad) | vm-result | 文本 '10' | `f738a3069e7a0b64` |
| [legacy.12_type_coercion.005_complex](../../../crates/auto-lang/test/vm/12_type_coercion/005_complex/complex.case.ad) | vm-result | 文本 '27.5' | `191ae775f98a1d5a` |
| [legacy.12_type_coercion.006_with_variable](../../../crates/auto-lang/test/vm/12_type_coercion/006_with_variable/with_variable.case.ad) | vm-result | 文本 '5.5' | `5a0985e9bdd2df19` |
| [legacy.21_conv.002_neg_i32_to_str](../../../crates/auto-lang/test/vm/21_conv/002_neg_i32_to_str/neg_i32_to_str.case.ad) | vm-stdout | 文本 '-1' | `b0c9788245cccc62` |
| [legacy.21_conv.003_hashmap_mono_insert](../../../crates/auto-lang/test/vm/21_conv/003_hashmap_mono_insert/hashmap_mono_insert.case.ad) | vm-result | 文本 'age=30 score=100 has=1 size=1' | `483da03151b68145` |
| [legacy.21_conv.004_hashset_mono_insert](../../../crates/auto-lang/test/vm/21_conv/004_hashset_mono_insert/hashset_mono_insert.case.ad) | vm-result | 文本 'apple=1 cherry=0 size=1' | `63f134039bcd3090` |
| [legacy.25_method_u64.001_to_uint_basic](../../../crates/auto-lang/test/vm/25_method_u64/001_to_uint_basic/to_uint_basic.case.ad) | vm-stdout | 文本 '42\n50\n40' | `e97a28a73415a88b` |
| [legacy.25_method_u64.002_to_uint_double_promote](../../../crates/auto-lang/test/vm/25_method_u64/002_to_uint_double_promote/to_uint_double_promote.case.ad) | vm-stdout | 文本 '11.5\n20' | `63425217e993710c` |
| [legacy.25_method_u64.003_let_u64_from_method](../../../crates/auto-lang/test/vm/25_method_u64/003_let_u64_from_method/let_u64_from_method.case.ad) | vm-stdout | 文本 '100\n123' | `f54beca232a2dbcc` |
| [legacy.25_method_u64.004_assign_u64](../../../crates/auto-lang/test/vm/25_method_u64/004_assign_u64/assign_u64.case.ad) | vm-stdout | 文本 '7\n14' | `75b976e4ab72d01e` |
| [legacy.25_method_u64.005_two_methods_arith](../../../crates/auto-lang/test/vm/25_method_u64/005_two_methods_arith/two_methods_arith.case.ad) | vm-stdout | 文本 '145\n55\n1' | `a371e28f9b382a30` |
| [legacy.25_method_u64.006_fstring_u64](../../../crates/auto-lang/test/vm/25_method_u64/006_fstring_u64/fstring_u64.case.ad) | vm-stdout | 文本 'year: 2024\nnext: 2025' | `3ab3d09da4e34e0c` |
| [legacy.25_method_u64.007_other_i64_methods](../../../crates/auto-lang/test/vm/25_method_u64/007_other_i64_methods/other_i64_methods.case.ad) | vm-stdout | 文本 '5\n3\n15' | `b3f8361c24a923b9` |
| [legacy.25_method_u64.008_i32_unaffected](../../../crates/auto-lang/test/vm/25_method_u64/008_i32_unaffected/i32_unaffected.case.ad) | vm-stdout | 文本 '1\n5\n1' | `6f3a9b094e887499` |
| [legacy.25_method_u64.009_large_value](../../../crates/auto-lang/test/vm/25_method_u64/009_large_value/large_value.case.ad) | vm-stdout | 文本 '5000000000\n10000000000\n10000000000\n5000000000\n5000000001' | `457cb713a5191d83` |
| [legacy.25_method_u64.010_global_u64_large](../../../crates/auto-lang/test/vm/25_method_u64/010_global_u64_large/global_u64_large.case.ad) | vm-stdout | 文本 '5000000000\n10000000000\n5000000000' | `435c15e263965dc1` |
| [legacy.25_method_u64.011_bigint_overflow](../../../crates/auto-lang/test/vm/25_method_u64/011_bigint_overflow/bigint_overflow.case.ad) | vm-stdout | 文本 '281474976710656\n281474976710656\n1\n1' | `e1ca834c8ac497fa` |
| [legacy.25_method_u64.012_list_hof_value](../../../crates/auto-lang/test/vm/25_method_u64/012_list_hof_value/list_hof_value.case.ad) | vm-stdout | 文本 '2\n2\nworld\n1\n1\n3\n1\n3\n6' | `9368edbd4140ac65` |
| [legacy.26_str_method_on_heap.001_lower_on_split](../../../crates/auto-lang/test/vm/26_str_method_on_heap/001_lower_on_split/lower_on_split.case.ad) | vm-stdout | 文本 'RS\nrs\nRS\nx' | `2b4af2a67683f92c` |
| [legacy.26_str_method_on_heap.002_lower_literal](../../../crates/auto-lang/test/vm/26_str_method_on_heap/002_lower_literal/lower_literal.case.ad) | vm-stdout | 文本 'hello\nworld\nHELLO' | `ce161a481d875399` |
| [legacy.99_plan230.001_struct_f64](../../../crates/auto-lang/test/vm/99_plan230/001_struct_f64/struct_f64.case.ad) | vm-stdout | 文本 'struct f64 OK' | `13bdf2b1c2ddeabf` |
| [legacy.99_short_circuit.001_and_or_short_circuit](../../../crates/auto-lang/test/vm/99_short_circuit/001_and_or_short_circuit/and_or_short_circuit.case.ad) | vm-stdout | 文本 'A: 0\nB else\nC else\nSIDE\nD: 1\nE taken\nSIDE\nSIDE\nSIDE\nF: 4\nSIDE\nG: 1\nH: false true\nI norm-false' | `153714fd4b17b12d` |
| [legacy.99_short_circuit.002_calc_eval_expr](../../../crates/auto-lang/test/vm/99_short_circuit/002_calc_eval_expr/calc_eval_expr.case.ad) | vm-stdout | 文本 '11\n11\n14\n3.5\nERROR\nERROR\n2\n64' | `a2b0c365bdbe4d7a` |
| [legacy.99_short_circuit.003_negative_index_guard](../../../crates/auto-lang/test/vm/99_short_circuit/003_negative_index_guard/negative_index_guard.case.ad) | vm-stdout | 文本 '30\n20\nG tail-7\nE guarded\nO: true' | `b08736b2a9e91725` |
| [legacy.99_slice.001_slice](../../../crates/auto-lang/test/vm/99_slice/001_slice/slice.case.ad) | vm-stdout | 文本 '99_slice: all assertions passed' | `4cf4c9a23cf5f97c` |

## PLAN-1080 原文档清单

本批主承接10份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.09_functions.001_fn_simple](../../../crates/auto-lang/test/vm/09_functions/001_fn_simple/fn_simple.case.ad) | vm-result | 文本 '14' | `3003be71e06cd3de` |
| [legacy.09_functions.002_fn_named_args](../../../crates/auto-lang/test/vm/09_functions/002_fn_named_args/fn_named_args.case.ad) | vm-result | 文本 '3' | `a44435fd2df153ef` |
| [legacy.09_functions.003_fn_multiple](../../../crates/auto-lang/test/vm/09_functions/003_fn_multiple/fn_multiple.case.ad) | vm-result | 文本 '10' | `b55ad62b3e57e3b0` |
| [legacy.09_functions.004_fn_nested](../../../crates/auto-lang/test/vm/09_functions/004_fn_nested/fn_nested.case.ad) | vm-result | 文本 '26' | `ef3e2742964f35f2` |
| [legacy.09_functions.005_fn_in_expr](../../../crates/auto-lang/test/vm/09_functions/005_fn_in_expr/fn_in_expr.case.ad) | vm-result | 文本 '18' | `c66d142a29e90f33` |
| [legacy.09_functions.006_fn_local_var](../../../crates/auto-lang/test/vm/09_functions/006_fn_local_var/fn_local_var.case.ad) | vm-result | 文本 '10' | `7bf94abbcb12673c` |
| [legacy.09_functions.009_forward_decl](../../../crates/auto-lang/test/vm/09_functions/009_forward_decl/forward_decl.case.ad) | vm-result | 文本 '42' | `ed852c1e296ac424` |
| [legacy.09_functions.020_fn_named_args_reordered](../../../crates/auto-lang/test/vm/09_functions/020_fn_named_args_reordered/fn_named_args_reordered.case.ad) | vm-result | 文本 '7' | `35817c956f16ea4c` |
| [legacy.09_functions.021_fn_named_ctor_reordered](../../../crates/auto-lang/test/vm/09_functions/021_fn_named_ctor_reordered/fn_named_ctor_reordered.case.ad) | vm-result | 文本 '103' | `ac248b4d972e2f87` |
| [legacy.99_idiom2.m10_pipe](../../../crates/auto-lang/test/vm/99_idiom2/m10_pipe/pipe.case.ad) | vm-stdout | 文本 '3\n18\n18' | `2f1403c70cb73f74` |

## PLAN-1081 原文档清单

本批主承接9份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.09_functions.007_closure](../../../crates/auto-lang/test/vm/09_functions/007_closure/closure.case.ad) | vm-result | 文本 '3' | `726d3505317d6ac6` |
| [legacy.09_functions.008_closure_typed](../../../crates/auto-lang/test/vm/09_functions/008_closure_typed/closure_typed.case.ad) | vm-result | 文本 '7' | `440651b9337e80d3` |
| [legacy.09_functions.010_closure_hof_map](../../../crates/auto-lang/test/vm/09_functions/010_closure_hof_map/closure_hof_map.case.ad) | vm-stdout | 文本 '3' | `789d2b50777321ee` |
| [legacy.09_functions.013_closure_capture_hof](../../../crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.case.ad) | vm-stdout | 文本 '3' | `782d9cc0967a650c` |
| [legacy.27_function_reference.001_basic_ref](../../../crates/auto-lang/test/vm/27_function_reference/001_basic_ref/basic_ref.case.ad) | vm-stdout | 文本 '10\n20' | `f3f6b0b4ae6be783` |
| [legacy.27_function_reference.002_pass_as_arg](../../../crates/auto-lang/test/vm/27_function_reference/002_pass_as_arg/pass_as_arg.case.ad) | vm-stdout | 文本 '42' | `6341c92bbec349a4` |
| [legacy.27_function_reference.004_call_unchanged](../../../crates/auto-lang/test/vm/27_function_reference/004_call_unchanged/call_unchanged.case.ad) | vm-stdout | 文本 '10\n14' | `3961b546a90d0541` |
| [legacy.99_idiom2.d01_nested_fn_void](../../../crates/auto-lang/test/vm/99_idiom2/d01_nested_fn_void/nested_fn_void.case.ad) | vm-stdout | 文本 'hi from inner\nafter' | `24d7495ae2723410` |
| [legacy.99_idiom2.d01b_nested_fn_capture](../../../crates/auto-lang/test/vm/99_idiom2/d01b_nested_fn_capture/nested_fn_capture.case.ad) | vm-error | 诊断 legacy.error-presence | `4f0a9f0abc20f45f` |

## PLAN-1082 原文档清单

本批主承接62份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.06_arrays.001_array_literal](../../../crates/auto-lang/test/vm/06_arrays/001_array_literal/array_literal.case.ad) | vm-result | 文本 '[1, 2, 3]' | `f8fbcb7f6903e066` |
| [legacy.06_arrays.002_array_index](../../../crates/auto-lang/test/vm/06_arrays/002_array_index/array_index.case.ad) | vm-result | 文本 '[1, 2, 3, 3, 2, 1]' | `eae4503618569072` |
| [legacy.06_arrays.003_array_update](../../../crates/auto-lang/test/vm/06_arrays/003_array_update/array_update.case.ad) | vm-result | 文本 '[4, 2, 3]' | `7b7c2c760e1de4da` |
| [legacy.06_arrays.004_array_of_objects](../../../crates/auto-lang/test/vm/06_arrays/004_array_of_objects/array_of_objects.case.ad) | vm-result | 文本 '[1, 2]' | `6786f9b2a4a3fc8d` |
| [legacy.06_arrays.005_array_multiple_mutations](../../../crates/auto-lang/test/vm/06_arrays/005_array_multiple_mutations/array_multiple_mutations.case.ad) | vm-result | 文本 '70' | `5c54888ccee3c177` |
| [legacy.06_arrays.010_tuple](../../../crates/auto-lang/test/vm/06_arrays/010_tuple/tuple.case.ad) | vm-stdout | 文本 'tuple: all assertions passed' | `71a8690be9112b06` |
| [legacy.09_functions.011_list_map_filter](../../../crates/auto-lang/test/vm/09_functions/011_list_map_filter/list_map_filter.case.ad) | vm-stdout | 文本 '3\n3' | `30f7af8320f37858` |
| [legacy.09_functions.012_list_reduce_find_any_all](../../../crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.case.ad) | vm-stdout | 文本 '15\n4\n0\n1' | `46dcf76bb611ad3f` |
| [legacy.09_functions.014_list_for_each_edge](../../../crates/auto-lang/test/vm/09_functions/014_list_for_each_edge/list_for_each_edge.case.ad) | vm-stdout | 文本 '10\n20\n30\n0\n0' | `78077738f3ba4746` |
| [legacy.09_functions.015_list_chain_pipeline](../../../crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.case.ad) | vm-stdout | 文本 '60\n5\n5' | `c4e8d64ddbae037d` |
| [legacy.13_collections.001_hashmap_new](../../../crates/auto-lang/test/vm/13_collections/001_hashmap_new/hashmap_new.case.ad) | vm-result | 文本 '0' | `2fb348f9211bbef3` |
| [legacy.13_collections.002_hashmap_insert_str](../../../crates/auto-lang/test/vm/13_collections/002_hashmap_insert_str/hashmap_insert_str.case.ad) | vm-result | 文本 '[Alice, Wonderland]' | `5e4c74f8810d45fd` |
| [legacy.13_collections.003_hashmap_insert_int](../../../crates/auto-lang/test/vm/13_collections/003_hashmap_insert_int/hashmap_insert_int.case.ad) | vm-result | 文本 '67' | `c261c5392be09297` |
| [legacy.13_collections.004_hashmap_contains](../../../crates/auto-lang/test/vm/13_collections/004_hashmap_contains/hashmap_contains.case.ad) | vm-result | 文本 '[1, 0]' | `d51a30d2fc6cdb8f` |
| [legacy.13_collections.005_hashmap_size](../../../crates/auto-lang/test/vm/13_collections/005_hashmap_size/hashmap_size.case.ad) | vm-result | 文本 '3' | `6ea5a43fd1249891` |
| [legacy.13_collections.006_hashmap_remove](../../../crates/auto-lang/test/vm/13_collections/006_hashmap_remove/hashmap_remove.case.ad) | vm-error | 诊断 legacy.error-presence | `3d2f3736e6781ab7` |
| [legacy.13_collections.007_hashmap_clear](../../../crates/auto-lang/test/vm/13_collections/007_hashmap_clear/hashmap_clear.case.ad) | vm-result | 文本 '0' | `606c690ff34197db` |
| [legacy.13_collections.008_hashset_new](../../../crates/auto-lang/test/vm/13_collections/008_hashset_new/hashset_new.case.ad) | vm-result | 文本 '0' | `1e9a26d239b4d81c` |
| [legacy.13_collections.009_hashset_insert](../../../crates/auto-lang/test/vm/13_collections/009_hashset_insert/hashset_insert.case.ad) | vm-stdout | 文本 '[1, 1, 1]' | `e8fac6c516221fad` |
| [legacy.13_collections.010_hashset_duplicate](../../../crates/auto-lang/test/vm/13_collections/010_hashset_duplicate/hashset_duplicate.case.ad) | vm-result | 文本 '1' | `62d58ed63331cc81` |
| [legacy.13_collections.011_hashset_remove](../../../crates/auto-lang/test/vm/13_collections/011_hashset_remove/hashset_remove.case.ad) | vm-error | 诊断 legacy.error-presence | `44982976c3d67e9f` |
| [legacy.13_collections.012_hashset_size](../../../crates/auto-lang/test/vm/13_collections/012_hashset_size/hashset_size.case.ad) | vm-result | 文本 '3' | `bf580b8602a3d340` |
| [legacy.13_collections.013_hashset_clear](../../../crates/auto-lang/test/vm/13_collections/013_hashset_clear/hashset_clear.case.ad) | vm-result | 文本 '0' | `26f425ff3ee9aaf4` |
| [legacy.13_collections.014_sb_new](../../../crates/auto-lang/test/vm/13_collections/014_sb_new/sb_new.case.ad) | vm-result | 文本 '0' | `a04acce54ca300bb` |
| [legacy.13_collections.015_sb_append](../../../crates/auto-lang/test/vm/13_collections/015_sb_append/sb_append.case.ad) | vm-result | 文本 'hello world' | `11c7c1989d453bac` |
| [legacy.13_collections.016_sb_append_char](../../../crates/auto-lang/test/vm/13_collections/016_sb_append_char/sb_append_char.case.ad) | vm-result | 文本 'hello!' | `14dba7985a504868` |
| [legacy.13_collections.017_sb_append_int](../../../crates/auto-lang/test/vm/13_collections/017_sb_append_int/sb_append_int.case.ad) | vm-result | 文本 'count: 42' | `3c8b9a07b775af0f` |
| [legacy.13_collections.018_sb_len](../../../crates/auto-lang/test/vm/13_collections/018_sb_len/sb_len.case.ad) | vm-result | 文本 '5' | `4dcc713cdbde52ed` |
| [legacy.13_collections.019_sb_clear](../../../crates/auto-lang/test/vm/13_collections/019_sb_clear/sb_clear.case.ad) | vm-result | 文本 '0' | `369229cb8b507f64` |
| [legacy.13_collections.020_list_new](../../../crates/auto-lang/test/vm/13_collections/020_list_new/list_new.case.ad) | vm-result | 文本 '1' | `e013322f7db6ecd4` |
| [legacy.13_collections.021_list_push_pop](../../../crates/auto-lang/test/vm/13_collections/021_list_push_pop/list_push_pop.case.ad) | vm-result | 文本 '30' | `1cc89e7a64945cba` |
| [legacy.13_collections.022_list_push_pop_multi](../../../crates/auto-lang/test/vm/13_collections/022_list_push_pop_multi/list_push_pop_multi.case.ad) | vm-result | 文本 '3' | `08b331f584257d40` |
| [legacy.13_collections.023_list_len](../../../crates/auto-lang/test/vm/13_collections/023_list_len/list_len.case.ad) | vm-result | 文本 '3' | `e5f81108b5573a60` |
| [legacy.13_collections.024_list_is_empty](../../../crates/auto-lang/test/vm/13_collections/024_list_is_empty/list_is_empty.case.ad) | vm-result | 文本 '[1, 0]' | `ab9faff961324282` |
| [legacy.13_collections.025_list_clear](../../../crates/auto-lang/test/vm/13_collections/025_list_clear/list_clear.case.ad) | vm-result | 文本 '0' | `af54c501689c0a89` |
| [legacy.13_collections.026_list_get_set](../../../crates/auto-lang/test/vm/13_collections/026_list_get_set/list_get_set.case.ad) | vm-result | 文本 '[10, 99, 30]' | `ed4319ef77ee98b2` |
| [legacy.13_collections.027_list_insert_remove](../../../crates/auto-lang/test/vm/13_collections/027_list_insert_remove/list_insert_remove.case.ad) | vm-result | 文本 '1' | `9ce55a6fbb0228fe` |
| [legacy.13_collections.028_list_reserve](../../../crates/auto-lang/test/vm/13_collections/028_list_reserve/list_reserve.case.ad) | vm-result | 文本 '2' | `9587bce5e70a40bc` |
| [legacy.13_collections.029_list_comprehensive](../../../crates/auto-lang/test/vm/13_collections/029_list_comprehensive/list_comprehensive.case.ad) | vm-result | 文本 '1' | `5e238ddfc4dd5a0f` |
| [legacy.13_collections.030_list_multi_ops](../../../crates/auto-lang/test/vm/13_collections/030_list_multi_ops/list_multi_ops.case.ad) | vm-result | 文本 '1' | `0d4eda23dccff4eb` |
| [legacy.13_collections.031_list_index](../../../crates/auto-lang/test/vm/13_collections/031_list_index/list_index.case.ad) | vm-result | 文本 '20' | `b3c5e2bf18e65520` |
| [legacy.13_collections.032_list_varargs](../../../crates/auto-lang/test/vm/13_collections/032_list_varargs/list_varargs.case.ad) | vm-result | 文本 '5' | `999e61ed0adec676` |
| [legacy.13_collections.033_list_varargs_empty](../../../crates/auto-lang/test/vm/13_collections/033_list_varargs_empty/list_varargs_empty.case.ad) | vm-result | 文本 '0' | `ee8c2cd298f35230` |
| [legacy.13_collections.034_list_for_iteration](../../../crates/auto-lang/test/vm/13_collections/034_list_for_iteration/list_for_iteration.case.ad) | vm-result | 文本 '15' | `464f7762e5647565` |
| [legacy.13_collections.035_list_for_empty](../../../crates/auto-lang/test/vm/13_collections/035_list_for_empty/list_for_empty.case.ad) | vm-result | 文本 '0' | `5452b55e4eca8d6c` |
| [legacy.15_nested_mutation.001_object_field](../../../crates/auto-lang/test/vm/15_nested_mutation/001_object_field/object_field.case.ad) | vm-result | 文本 '30' | `cdb404353c0df029` |
| [legacy.15_nested_mutation.002_array_element](../../../crates/auto-lang/test/vm/15_nested_mutation/002_array_element/array_element.case.ad) | vm-result | 文本 '10' | `faaeee453f73f5e9` |
| [legacy.15_nested_mutation.003_multiple_fields](../../../crates/auto-lang/test/vm/15_nested_mutation/003_multiple_fields/multiple_fields.case.ad) | vm-result | 文本 '600' | `7c56319e6da57f0d` |
| [legacy.15_nested_mutation.004_multiple_array](../../../crates/auto-lang/test/vm/15_nested_mutation/004_multiple_array/multiple_array.case.ad) | vm-result | 文本 '60' | `cdd18ea82a90daf3` |
| [legacy.15_nested_mutation.005_type_field](../../../crates/auto-lang/test/vm/15_nested_mutation/005_type_field/type_field.case.ad) | vm-result | 文本 '30' | `65672a857e1f3ed2` |
| [legacy.15_nested_mutation.006_nested_object](../../../crates/auto-lang/test/vm/15_nested_mutation/006_nested_object/nested_object.case.ad) | vm-result | 文本 '30' | `468a4055ae3a398c` |
| [legacy.15_nested_mutation.007_array_of_obj_field](../../../crates/auto-lang/test/vm/15_nested_mutation/007_array_of_obj_field/array_of_obj_field.case.ad) | vm-result | 文本 '10' | `c965b1583d7c0af4` |
| [legacy.15_nested_mutation.008_obj_array_element](../../../crates/auto-lang/test/vm/15_nested_mutation/008_obj_array_element/obj_array_element.case.ad) | vm-result | 文本 '10' | `83713f0117fe027f` |
| [legacy.15_nested_mutation.009_nested_array](../../../crates/auto-lang/test/vm/15_nested_mutation/009_nested_array/nested_array.case.ad) | vm-result | 文本 '20' | `4d4ae8610a4938b3` |
| [legacy.15_nested_mutation.010_type_nested_field](../../../crates/auto-lang/test/vm/15_nested_mutation/010_type_nested_field/type_nested_field.case.ad) | vm-result | 文本 '20' | `2f5e8c15a1dd0afd` |
| [legacy.15_nested_mutation.011_three_level](../../../crates/auto-lang/test/vm/15_nested_mutation/011_three_level/three_level.case.ad) | vm-result | 文本 '200' | `8773c872db3885ed` |
| [legacy.15_nested_mutation.012_deep_array_obj](../../../crates/auto-lang/test/vm/15_nested_mutation/012_deep_array_obj/deep_array_obj.case.ad) | vm-result | 文本 '25' | `81d1b4019ecf4991` |
| [legacy.15_nested_mutation.013_structure_preserve](../../../crates/auto-lang/test/vm/15_nested_mutation/013_structure_preserve/structure_preserve.case.ad) | vm-result | 文本 '[2, 3, 4]' | `158c00e75e61a8a0` |
| [legacy.15_nested_mutation.014_out_of_bounds_error](../../../crates/auto-lang/test/vm/15_nested_mutation/014_out_of_bounds_error/out_of_bounds_error.case.ad) | vm-error | 诊断 legacy.error-presence | `63a608b88763b55b` |
| [legacy.15_nested_mutation.015_self_list_push](../../../crates/auto-lang/test/vm/15_nested_mutation/015_self_list_push/self_list_push.case.ad) | vm-stdout | 文本 '3' | `184ebeaca0a66a03` |
| [legacy.29_list_shims.001_int_list](../../../crates/auto-lang/test/vm/29_list_shims/001_int_list/int_list.case.ad) | vm-stdout | 文本 '3\n20\n99\n4\n30\n2\n1' | `f12f07ca6af6270b` |
| [legacy.29_list_shims.002_struct_list](../../../crates/auto-lang/test/vm/29_list_shims/002_struct_list/struct_list.case.ad) | vm-stdout | 文本 '2\nalpha\nbeta\nbeta\n1' | `8a5a15e3f5a3fc1d` |

## PLAN-1083 原文档清单

本批主承接52份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.09_functions.016_enum_multi_destruct](../../../crates/auto-lang/test/vm/09_functions/016_enum_multi_destruct/enum_multi_destruct.case.ad) | vm-stdout | 文本 '10\n20\n5\npoint' | `d189db6629f2f600` |
| [legacy.09_functions.017_enum_named_construct](../../../crates/auto-lang/test/vm/09_functions/017_enum_named_construct/enum_named_construct.case.ad) | vm-stdout | 文本 '404\n503\n5000' | `181b3e9148fe6634` |
| [legacy.09_functions.018_enum_destruct_edge](../../../crates/auto-lang/test/vm/09_functions/018_enum_destruct_edge/enum_destruct_edge.case.ad) | vm-stdout | 文本 'rgb\n128\n383' | `1bc4ddc0b9b628f7` |
| [legacy.09_functions.019_enum_tuple_payload](../../../crates/auto-lang/test/vm/09_functions/019_enum_tuple_payload/enum_tuple_payload.case.ad) | vm-stdout | 文本 '3.14\n10\n20' | `2148971b03f2ce31` |
| [legacy.10_types.001_type_compose](../../../crates/auto-lang/test/vm/10_types/001_type_compose/type_compose.case.ad) | vm-error | 诊断 legacy.error-presence | `7625d00147b5ff3f` |
| [legacy.10_types.002_int_enum](../../../crates/auto-lang/test/vm/10_types/002_int_enum/int_enum.case.ad) | vm-result | 文本 '1' | `171e8b0da5f12388` |
| [legacy.10_types.003_generic_instantiation](../../../crates/auto-lang/test/vm/10_types/003_generic_instantiation/generic_instantiation.case.ad) | vm-error | 诊断 legacy.error-presence | `e26c5fa2505fd078` |
| [legacy.10_types.004_generic_field_x](../../../crates/auto-lang/test/vm/10_types/004_generic_field_x/generic_field_x.case.ad) | vm-error | 诊断 legacy.error-presence | `040fde5016c655e0` |
| [legacy.10_types.005_generic_field_y](../../../crates/auto-lang/test/vm/10_types/005_generic_field_y/generic_field_y.case.ad) | vm-error | 诊断 legacy.error-presence | `4fc3505f95ed8ca8` |
| [legacy.10_types.006_field_addition](../../../crates/auto-lang/test/vm/10_types/006_field_addition/field_addition.case.ad) | vm-result | 文本 '300' | `6cf6890d8051f6a9` |
| [legacy.10_types.007_type_instance_prop](../../../crates/auto-lang/test/vm/10_types/007_type_instance_prop/type_instance_prop.case.ad) | vm-result | 文本 '42' | `93600e65130010d6` |
| [legacy.10_types.008_nested_type_instance](../../../crates/auto-lang/test/vm/10_types/008_nested_type_instance/nested_type_instance.case.ad) | vm-result | 文本 '10' | `18ef5cbc3cec56e2` |
| [legacy.10_types.009_access_fields_in_method](../../../crates/auto-lang/test/vm/10_types/009_access_fields_in_method/access_fields_in_method.case.ad) | vm-error | 诊断 legacy.error-presence | `14194d03bc3aa4fa` |
| [legacy.10_types.010_ext_method](../../../crates/auto-lang/test/vm/10_types/010_ext_method/ext_method.case.ad) | vm-stdout | 文本 '42\n99' | `e4337304a1b55d7d` |
| [legacy.10_types.011_enum_is_match](../../../crates/auto-lang/test/vm/10_types/011_enum_is_match/enum_is_match.case.ad) | vm-stdout | 文本 'red\nblue' | `2487dbf027b1461d` |
| [legacy.10_types.011_enum_multi_field](../../../crates/auto-lang/test/vm/10_types/011_enum_multi_field/enum_multi_field.case.ad) | vm-stdout | 文本 'enum_multi_field: all assertions passed' | `e20187838d057930` |
| [legacy.10_types.012_enum_dot_match](../../../crates/auto-lang/test/vm/10_types/012_enum_dot_match/enum_dot_match.case.ad) | vm-stdout | 文本 'red\nblue' | `7f9fcda462e56fbd` |
| [legacy.10_types.013_fn_return_obj](../../../crates/auto-lang/test/vm/10_types/013_fn_return_obj/fn_return_obj.case.ad) | vm-stdout | 文本 'p=Point { x: 42 }\np.x=42' | `4129bd4237d60e62` |
| [legacy.10_types.033_array_len_user_type.array_len_user_type](../../../crates/auto-lang/test/vm/10_types/033_array_len_user_type/array_len_user_type.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `65dac91f9a629f1f` |
| [legacy.10_types.034_struct_str_field](../../../crates/auto-lang/test/vm/10_types/034_struct_str_field/struct_str_field.case.ad) | vm-stdout | 文本 'title ok\nid ok' | `bcc0a2a6136de0e7` |
| [legacy.10_types.035_struct_from_fn](../../../crates/auto-lang/test/vm/10_types/035_struct_from_fn/struct_from_fn.case.ad) | vm-stdout | 文本 'title from fn ok\nid from fn ok' | `7da9e0712d27c296` |
| [legacy.10_types.036_struct_in_array](../../../crates/auto-lang/test/vm/10_types/036_struct_in_array/struct_in_array.case.ad) | vm-stdout | 文本 'array field ok\nsecond id ok' | `bc847bd2ee879c36` |
| [legacy.16_option_result.001_option_type](../../../crates/auto-lang/test/vm/16_option_result/001_option_type/option_type.case.ad) | vm-result | 文本 '1' | `2150495a2f68c98b` |
| [legacy.16_option_result.002_result_type](../../../crates/auto-lang/test/vm/16_option_result/002_result_type/result_type.case.ad) | vm-result | 文本 '1' | `4973d0f185f9eed6` |
| [legacy.16_option_result.003_none_literal](../../../crates/auto-lang/test/vm/16_option_result/003_none_literal/none_literal.case.ad) | vm-result | 文本 '1' | `fae8cdc15ea14445` |
| [legacy.16_option_result.004_some_ctor](../../../crates/auto-lang/test/vm/16_option_result/004_some_ctor/some_ctor.case.ad) | vm-result | 文本 '1' | `6ac88e8fa0e0d6f8` |
| [legacy.16_option_result.005_ok_ctor](../../../crates/auto-lang/test/vm/16_option_result/005_ok_ctor/ok_ctor.case.ad) | vm-result | 文本 '1' | `f9098b5a2b75cc4e` |
| [legacy.16_option_result.006_err_ctor](../../../crates/auto-lang/test/vm/16_option_result/006_err_ctor/err_ctor.case.ad) | vm-result | 文本 '1' | `7c5e05b3519f7244` |
| [legacy.16_option_result.013_is_some_binding](../../../crates/auto-lang/test/vm/16_option_result/013_is_some_binding/is_some_binding.case.ad) | vm-result | 文本 '42' | `96eda4a6ef8ad82b` |
| [legacy.16_option_result.014_is_none_match](../../../crates/auto-lang/test/vm/16_option_result/014_is_none_match/is_none_match.case.ad) | vm-result | 文本 '-1' | `0fa61f6bfe4d9b20` |
| [legacy.16_option_result.015_is_ok_binding](../../../crates/auto-lang/test/vm/16_option_result/015_is_ok_binding/is_ok_binding.case.ad) | vm-result | 文本 '100' | `bbd6f4fdbfd61654` |
| [legacy.16_option_result.016_is_err_match](../../../crates/auto-lang/test/vm/16_option_result/016_is_err_match/is_err_match.case.ad) | vm-result | 文本 '-2' | `b84de2379361ada9` |
| [legacy.16_option_result.017_unwrap_or](../../../crates/auto-lang/test/vm/16_option_result/017_unwrap_or/unwrap_or.case.ad) | vm-stdout | 文本 'dot_quest_default: all passed' | `bd72532b7a7f100a` |
| [legacy.16_option_result.018_option_or](../../../crates/auto-lang/test/vm/16_option_result/018_option_or/option_or.case.ad) | vm-stdout | 文本 '42\n99' | `0f520954373f0b07` |
| [legacy.16_option_result.019_result_heap](../../../crates/auto-lang/test/vm/16_option_result/019_result_heap/result_heap.case.ad) | vm-stdout | 文本 '3\nerror' | `db7acd2987014994` |
| [legacy.16_option_result.021_result_enum_error](../../../crates/auto-lang/test/vm/16_option_result/021_result_enum_error/result_enum_error.case.ad) | vm-stdout | 文本 'unexpected end\n42' | `0674e62644d377c3` |
| [legacy.16_option_result.022_result_multi_error](../../../crates/auto-lang/test/vm/16_option_result/022_result_multi_error/result_multi_error.case.ad) | vm-stdout | 文本 '404\ntimeout\n200' | `acf3eb42783074d1` |
| [legacy.28_enum_methods.001_basic](../../../crates/auto-lang/test/vm/28_enum_methods/001_basic/basic.case.ad) | vm-stdout | 文本 'called-ok\ncalled-ok' | `d1745272a382e9ae` |
| [legacy.28_enum_methods.002_is_match](../../../crates/auto-lang/test/vm/28_enum_methods/002_is_match/is_match.case.ad) | vm-stdout | 文本 '1\n0' | `c934e92b53ded3e0` |
| [legacy.28_enum_methods.003_payload_method](../../../crates/auto-lang/test/vm/28_enum_methods/003_payload_method/payload_method.case.ad) | vm-stdout | 文本 '1\n0' | `aac49f6b74c7353c` |
| [legacy.28_enum_methods.004_wildcard](../../../crates/auto-lang/test/vm/28_enum_methods/004_wildcard/wildcard.case.ad) | vm-stdout | 文本 'red\nother\nother' | `96bce33c64651355` |
| [legacy.28_enum_methods.005_generic_enum_method](../../../crates/auto-lang/test/vm/28_enum_methods/005_generic_enum_method/generic_enum_method.case.ad) | vm-stdout | 文本 '1\n0' | `0ab31a035c4243e2` |
| [legacy.28_enum_methods.006_stdlib_result](../../../crates/auto-lang/test/vm/28_enum_methods/006_stdlib_result/stdlib_result.case.ad) | vm-stdout | 文本 '1\n0\n0\n1' | `60e92adeddd2ac75` |
| [legacy.99_idiom2.d02_struct_misuse](../../../crates/auto-lang/test/vm/99_idiom2/d02_struct_misuse/struct_misuse.case.ad) | vm-error | 诊断 legacy.error-presence | `fae85693d251490c` |
| [legacy.99_idiom2.m01_field_rw](../../../crates/auto-lang/test/vm/99_idiom2/m01_field_rw/field_rw.case.ad) | vm-stdout | 文本 'get=10\ninc=11\ninc=12\nafter set_to=99' | `c2934b41c523c178` |
| [legacy.99_idiom2.m02_method_calls_method](../../../crates/auto-lang/test/vm/99_idiom2/m02_method_calls_method/method_calls_method.case.ad) | vm-stdout | 文本 'call_helper=13\nnested=21' | `a5f72fa663c2fea5` |
| [legacy.99_idiom2.m03_static_new_chain](../../../crates/auto-lang/test/vm/99_idiom2/m03_static_new_chain/static_new_chain.case.ad) | vm-stdout | 文本 'chain=6\ntwo=15\norig=5' | `bdcc384a34129dea` |
| [legacy.99_idiom2.m04_positional_ctor](../../../crates/auto-lang/test/vm/99_idiom2/m04_positional_ctor/positional_ctor.case.ad) | vm-stdout | 文本 'build=9\norigin=0\nmake=9' | `76023ef309ede408` |
| [legacy.99_idiom2.m05_list_elem_method](../../../crates/auto-lang/test/vm/99_idiom2/m05_list_elem_method/list_elem_method.case.ad) | vm-stdout | 文本 'total=6\nidx1=2' | `c704ea03c0a6854a` |
| [legacy.99_idiom2.m06_method_value](../../../crates/auto-lang/test/vm/99_idiom2/m06_method_value/method_value.case.ad) | vm-stdout | 文本 'apply=7\nmake_get=8\ntwice=12' | `cf4fe6cd90ade857` |
| [legacy.99_idiom2.m08_receiver_forms](../../../crates/auto-lang/test/vm/99_idiom2/m08_receiver_forms/receiver_forms.case.ad) | vm-stdout | 文本 'short=3\nself=3\nafter=300\nafter_self=300' | `69fdfc884991ca19` |
| [legacy.99_idiom2.m09_method_name_set](../../../crates/auto-lang/test/vm/99_idiom2/m09_method_name_set/method_name_set.case.ad) | vm-stdout | 文本 'set_get=42' | `b08d29b2fd37395d` |

## PLAN-1084 原文档清单

本批主承接11份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.07_objects.001_object_field](../../../crates/auto-lang/test/vm/07_objects/001_object_field/object_field.case.ad) | vm-result | 文本 'auto' | `3d86ede697d73c40` |
| [legacy.07_objects.002_object_field_int_key](../../../crates/auto-lang/test/vm/07_objects/002_object_field_int_key/object_field_int_key.case.ad) | vm-result | 文本 '4' | `6bd2e7e9a7c89913` |
| [legacy.07_objects.003_object_field_bool_key](../../../crates/auto-lang/test/vm/07_objects/003_object_field_bool_key/object_field_bool_key.case.ad) | vm-result | 文本 '4' | `8815c02ccd645562` |
| [legacy.07_objects.004_obj_set](../../../crates/auto-lang/test/vm/07_objects/004_obj_set/obj_set.case.ad) | vm-result | 文本 'Bob' | `e5d7b6cebf07825d` |
| [legacy.07_objects.005_nested_object](../../../crates/auto-lang/test/vm/07_objects/005_nested_object/nested_object.case.ad) | vm-result | 文本 '10' | `d692138be5a3bae8` |
| [legacy.07_objects.006_inline_map](../../../crates/auto-lang/test/vm/07_objects/006_inline_map/inline_map.case.ad) | vm-stdout | 文本 'inline_map: all assertions passed' | `7ca2cf9456cedfa2` |
| [legacy.07_objects.006_nested_object_y](../../../crates/auto-lang/test/vm/07_objects/006_nested_object_y/nested_object_y.case.ad) | vm-result | 文本 '20' | `afb64a4689fb9a58` |
| [legacy.07_objects.007_json](../../../crates/auto-lang/test/vm/07_objects/007_json/json.case.ad) | vm-result | 文本 'ClearDiagnosticInformation' | `d9967cba45faed9a` |
| [legacy.07_objects.008_last_block_or_object](../../../crates/auto-lang/test/vm/07_objects/008_last_block_or_object/last_block_or_object.case.ad) | vm-result | 文本 '{a: 1, b: 2}' | `f0e3873bc219d6b6` |
| [legacy.07_objects.009_multiple_field_mutations](../../../crates/auto-lang/test/vm/07_objects/009_multiple_field_mutations/multiple_field_mutations.case.ad) | vm-result | 文本 '600' | `91d9d9284965d7ca` |
| [legacy.07_objects.010_map_bracket_write](../../../crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.case.ad) | vm-result | 文本 '600' | `328afff7d2293e19` |

## PLAN-1085 原文档清单

本批主承接9份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.03_variables.001_var](../../../crates/auto-lang/test/vm/03_variables/001_var/var.case.ad) | vm-result | 文本 '3' | `8e186f6e911806dd` |
| [legacy.03_variables.002_var_assign](../../../crates/auto-lang/test/vm/03_variables/002_var_assign/var_assign.case.ad) | vm-result | 文本 '2' | `92f17eca0343d51d` |
| [legacy.03_variables.003_var_mut](../../../crates/auto-lang/test/vm/03_variables/003_var_mut/var_mut.case.ad) | vm-result | 文本 '11' | `8a12acac876defa9` |
| [legacy.03_variables.004_var_if](../../../crates/auto-lang/test/vm/03_variables/004_var_if/var_if.case.ad) | vm-result | 文本 '2' | `daf458dea2de07b7` |
| [legacy.03_variables.005_let_binding](../../../crates/auto-lang/test/vm/03_variables/005_let_binding/let_binding.case.ad) | vm-result | 文本 '41' | `50e9f7e88fc96ccc` |
| [legacy.03_variables.006_let_asn_error](../../../crates/auto-lang/test/vm/03_variables/006_let_asn_error/let_asn_error.case.ad) | vm-error | 诊断 legacy.error-presence | `f1485af4deb1fc22` |
| [legacy.03_variables.007_var_reassignment](../../../crates/auto-lang/test/vm/03_variables/007_var_reassignment/var_reassignment.case.ad) | vm-result | 文本 '10' | `12626fbdb57f31c0` |
| [legacy.03_variables.008_simple_block](../../../crates/auto-lang/test/vm/03_variables/008_simple_block/simple_block.case.ad) | vm-result | 文本 '10' | `b53129d1b2d5b7ae` |
| [legacy.17_modules.001_use_fn](../../../crates/auto-lang/test/vm/17_modules/001_use_fn/use_fn.case.ad) | vm-stdout | 文本 'world\n3' | `0c36dd8a372f446e` |

## PLAN-1086 原文档清单

本批主承接28份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.14_borrow.001_view_basic](../../../crates/auto-lang/test/vm/14_borrow/001_view_basic/view_basic.case.ad) | vm-result | 文本 'hello' | `2b812086b5948d28` |
| [legacy.14_borrow.002_view_multiple](../../../crates/auto-lang/test/vm/14_borrow/002_view_multiple/view_multiple.case.ad) | vm-result | 文本 '84' | `0bf632a7977ae86c` |
| [legacy.14_borrow.003_mut_basic](../../../crates/auto-lang/test/vm/14_borrow/003_mut_basic/mut_basic.case.ad) | vm-result | 文本 'hello' | `14e7e315d229e799` |
| [legacy.14_borrow.004_move_basic](../../../crates/auto-lang/test/vm/14_borrow/004_move_basic/move_basic.case.ad) | vm-result | 文本 'hello' | `f3de7d765126aad2` |
| [legacy.14_borrow.005_view_preserves](../../../crates/auto-lang/test/vm/14_borrow/005_view_preserves/view_preserves.case.ad) | vm-result | 文本 '100' | `61a984a48d219f7f` |
| [legacy.14_borrow.006_nested_view](../../../crates/auto-lang/test/vm/14_borrow/006_nested_view/nested_view.case.ad) | vm-result | 文本 '42' | `0f3ca15b61050ebf` |
| [legacy.14_borrow.007_borrow_arithmetic](../../../crates/auto-lang/test/vm/14_borrow/007_borrow_arithmetic/borrow_arithmetic.case.ad) | vm-result | 文本 '30' | `75587cbd3d375018` |
| [legacy.14_borrow.008_view_in_array](../../../crates/auto-lang/test/vm/14_borrow/008_view_in_array/view_in_array.case.ad) | vm-result | 文本 '[10, 20]' | `aa283e3a76141411` |
| [legacy.14_borrow.009_view_in_expr](../../../crates/auto-lang/test/vm/14_borrow/009_view_in_expr/view_in_expr.case.ad) | vm-result | 文本 '15' | `d0e55c8a7cf3f4c0` |
| [legacy.14_borrow.010_borrow_diff_types](../../../crates/auto-lang/test/vm/14_borrow/010_borrow_diff_types/borrow_diff_types.case.ad) | vm-result | 文本 '[42, "hello"]' | `3a51802b81578cd9` |
| [legacy.14_borrow.011_move_chaining](../../../crates/auto-lang/test/vm/14_borrow/011_move_chaining/move_chaining.case.ad) | vm-result | 文本 'first' | `a421b05cf2b2b615` |
| [legacy.14_borrow.012_str_sliceview](../../../crates/auto-lang/test/vm/14_borrow/012_str_sliceview/str_sliceview.case.ad) | vm-result | 文本 'hello world' | `7112bb730499aa10` |
| [legacy.14_borrow.013_str_slice_multi](../../../crates/auto-lang/test/vm/14_borrow/013_str_slice_multi/str_slice_multi.case.ad) | vm-result | 文本 '[hello, hello]' | `40aa35e1775f3668` |
| [legacy.14_borrow.014_str_slice_nested](../../../crates/auto-lang/test/vm/14_borrow/014_str_slice_nested/str_slice_nested.case.ad) | vm-result | 文本 'hello' | `173053d74174061a` |
| [legacy.14_borrow.015_str_slice_in_array](../../../crates/auto-lang/test/vm/14_borrow/015_str_slice_in_array/str_slice_in_array.case.ad) | vm-result | 文本 '["hello", "world"]' | `b35e1ac11febe1d0` |
| [legacy.14_borrow.016_str_slice_take](../../../crates/auto-lang/test/vm/14_borrow/016_str_slice_take/str_slice_take.case.ad) | vm-result | 文本 'hello' | `7cd7275d8cbdb381` |
| [legacy.14_borrow.017_str_slice_mixed](../../../crates/auto-lang/test/vm/14_borrow/017_str_slice_mixed/str_slice_mixed.case.ad) | vm-result | 文本 'first' | `055bfe0d527ee872` |
| [legacy.14_borrow.018_str_slice_preserves](../../../crates/auto-lang/test/vm/14_borrow/018_str_slice_preserves/str_slice_preserves.case.ad) | vm-result | 文本 'hello' | `638cbe693e3845f0` |
| [legacy.99_plan231.001_nested_mutfn](../../../crates/auto-lang/test/vm/99_plan231/001_nested_mutfn/nested_mutfn.case.ad) | vm-stdout | 文本 'events: 1\nSSE parser OK' | `93dadea39ac94109` |
| [legacy.99_plan231.002_direct_drain](../../../crates/auto-lang/test/vm/99_plan231/002_direct_drain/direct_drain.case.ad) | vm-stdout | 文本 'events: 1\ndirect drain OK' | `43d3e55c1b1d9a57` |
| [legacy.99_plan231.003_simple_for_option](../../../crates/auto-lang/test/vm/99_plan231/003_simple_for_option/simple_for_option.case.ad) | vm-stdout | 文本 'total: 6\nfor-option OK' | `5b3a457197910c59` |
| [legacy.99_plan231.004_for_option_no_is](../../../crates/auto-lang/test/vm/99_plan231/004_for_option_no_is/for_option_no_is.case.ad) | vm-stdout | 文本 'got: 2\ngot: 1\ndone' | `5866436ddf51f9e0` |
| [legacy.99_plan231.005_mutfn_no_loop](../../../crates/auto-lang/test/vm/99_plan231/005_mutfn_no_loop/mutfn_no_loop.case.ad) | vm-stdout | 文本 'v1: 2\nv2: 1\nv3: None\nmutfn no loop OK' | `08f49a35f8990351` |
| [legacy.99_plan231.006_mutfn_int_return](../../../crates/auto-lang/test/vm/99_plan231/006_mutfn_int_return/mutfn_int_return.case.ad) | vm-stdout | 文本 'v1: 2\nv2: 1\nmutfn int return OK' | `a7752455e2ca2092` |
| [legacy.99_plan231.007_mutfn_void_return](../../../crates/auto-lang/test/vm/99_plan231/007_mutfn_void_return/mutfn_void_return.case.ad) | vm-stdout | 文本 'count: 3\nmutfn void return OK' | `7ae332a5d7c7864c` |
| [legacy.99_plan231.008_like_vmtest09](../../../crates/auto-lang/test/vm/99_plan231/008_like_vmtest09/like_vmtest09.case.ad) | vm-stdout | 文本 '=== Self field access test ===\ncount: 3' | `b1dde54a3ba8ef5a` |
| [legacy.99_plan231.009_mutfn_print_direct](../../../crates/auto-lang/test/vm/99_plan231/009_mutfn_print_direct/mutfn_print_direct.case.ad) | vm-stdout | 文本 '3\nmutfn print direct OK' | `93f3b15767dfdd0e` |
| [legacy.99_plan231.010_mutfn_int_no_fstr](../../../crates/auto-lang/test/vm/99_plan231/010_mutfn_int_no_fstr/mutfn_int_no_fstr.case.ad) | vm-stdout | 文本 '2\n1\nmutfn int no fstr OK' | `d7e64c062a80d1f3` |

## PLAN-1087 原文档清单

本批主承接25份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.04_control_flow.001_if_true](../../../crates/auto-lang/test/vm/04_control_flow/001_if_true/if_true.case.ad) | vm-result | 文本 '1' | `9566c27509d9d072` |
| [legacy.04_control_flow.002_if_false](../../../crates/auto-lang/test/vm/04_control_flow/002_if_false/if_false.case.ad) | vm-result | 文本 '2' | `c27e1dbb711ee950` |
| [legacy.04_control_flow.003_if_else_if](../../../crates/auto-lang/test/vm/04_control_flow/003_if_else_if/if_else_if.case.ad) | vm-result | 文本 '3' | `0b1b0292a734a8c8` |
| [legacy.04_control_flow.004_if_with_bool](../../../crates/auto-lang/test/vm/04_control_flow/004_if_with_bool/if_with_bool.case.ad) | vm-stdout,vm-result | 文本 'I won!' | `fe0e82aff00b76ce` |
| [legacy.04_control_flow.005_if_in_array](../../../crates/auto-lang/test/vm/04_control_flow/005_if_in_array/if_in_array.case.ad) | vm-result | 文本 '["osal", "al"]' | `6d2e851076cbae82` |
| [legacy.04_control_flow.006_is_stmt](../../../crates/auto-lang/test/vm/04_control_flow/006_is_stmt/is_stmt.case.ad) | vm-stdout,vm-result | 文本 'Here is 10!' | `d86535c87c3eb4c6` |
| [legacy.04_control_flow.007_asn_upper](../../../crates/auto-lang/test/vm/04_control_flow/007_asn_upper/asn_upper.case.ad) | vm-result | 文本 '2' | `af1649629fc3b9d1` |
| [legacy.04_control_flow.008_is_or_pattern](../../../crates/auto-lang/test/vm/04_control_flow/008_is_or_pattern/is_or_pattern.case.ad) | vm-stdout | 文本 'small' | `13ebbd3f9553a126` |
| [legacy.04_control_flow.009_is_or_groups](../../../crates/auto-lang/test/vm/04_control_flow/009_is_or_groups/is_or_groups.case.ad) | vm-stdout | 文本 'high' | `590d9bbf0491050a` |
| [legacy.04_control_flow.010_as_int_to_float](../../../crates/auto-lang/test/vm/04_control_flow/010_as_int_to_float/as_int_to_float.case.ad) | vm-stdout | 文本 '42' | `6576acb6112e88c4` |
| [legacy.04_control_flow.011_as_float_to_int](../../../crates/auto-lang/test/vm/04_control_flow/011_as_float_to_int/as_float_to_int.case.ad) | vm-stdout | 文本 '3' | `a155e923d36af183` |
| [legacy.04_control_flow.012_to_str](../../../crates/auto-lang/test/vm/04_control_flow/012_to_str/to_str.case.ad) | vm-stdout | 文本 '42' | `9fb0e8022ceb06d5` |
| [legacy.04_control_flow.013_to_int_from_str](../../../crates/auto-lang/test/vm/04_control_flow/013_to_int_from_str/to_int_from_str.case.ad) | vm-stdout | 文本 '42' | `049339b2bdb75dbe` |
| [legacy.05_loops.001_for_range](../../../crates/auto-lang/test/vm/05_loops/001_for_range/for_range.case.ad) | vm-result | 文本 '45' | `ea13174d4c0d9c99` |
| [legacy.05_loops.002_range_inclusive](../../../crates/auto-lang/test/vm/05_loops/002_range_inclusive/range_inclusive.case.ad) | vm-result | 文本 '55' | `5d3baa3ccca7120d` |
| [legacy.05_loops.003_range_literal](../../../crates/auto-lang/test/vm/05_loops/003_range_literal/range_literal.case.ad) | vm-result | 文本 '1..5' | `5d934431de4c837f` |
| [legacy.05_loops.004_for_each_object](../../../crates/auto-lang/test/vm/05_loops/004_for_each_object/for_each_object.case.ad) | vm-stdout | 文本 'Hi Alice\nHi Bob\nHi Charlie' | `abe3581a42fe0099` |
| [legacy.05_loops.005_loop_keyword](../../../crates/auto-lang/test/vm/05_loops/005_loop_keyword/loop_keyword.case.ad) | vm-stdout | 文本 '3' | `8dbedc7058cd34d3` |
| [legacy.16_option_result.007_propagate_some](../../../crates/auto-lang/test/vm/16_option_result/007_propagate_some/propagate_some.case.ad) | vm-result | 文本 '42' | `794567daa6c2b728` |
| [legacy.16_option_result.008_propagate_none](../../../crates/auto-lang/test/vm/16_option_result/008_propagate_none/propagate_none.case.ad) | vm-result | 文本 '-1' | `88e6af75fc703a05` |
| [legacy.16_option_result.009_propagate_ok](../../../crates/auto-lang/test/vm/16_option_result/009_propagate_ok/propagate_ok.case.ad) | vm-result | 文本 '100' | `f6dae942d85e37ce` |
| [legacy.16_option_result.010_propagate_err](../../../crates/auto-lang/test/vm/16_option_result/010_propagate_err/propagate_err.case.ad) | vm-result | 文本 '4000000' | `b553e6e81c862eba` |
| [legacy.16_option_result.011_coalesce_some](../../../crates/auto-lang/test/vm/16_option_result/011_coalesce_some/coalesce_some.case.ad) | vm-result | 文本 '42' | `4c2c7929f57fa19f` |
| [legacy.16_option_result.012_coalesce_none](../../../crates/auto-lang/test/vm/16_option_result/012_coalesce_none/coalesce_none.case.ad) | vm-result | 文本 '99' | `8655c4812347934f` |
| [legacy.16_option_result.020_result_propagate](../../../crates/auto-lang/test/vm/16_option_result/020_result_propagate/result_propagate.case.ad) | vm-stdout | 文本 'propagated error' | `dc3a5777b972f05a` |

## PLAN-1088 原文档清单

本批主承接7份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.24_generics.001_list_struct](../../../crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.case.ad) | vm-stdout | 文本 '2' | `bc7693bea3f099f2` |
| [legacy.24_generics.002_to_array](../../../crates/auto-lang/test/vm/24_generics/002_to_array/to_array.case.ad) | vm-stdout | 文本 'ok' | `34fb70e491bc86c0` |
| [legacy.24_generics.003_filter](../../../crates/auto-lang/test/vm/24_generics/003_filter/filter.case.ad) | vm-stdout | 文本 'ok' | `46c0ec16a899c49b` |
| [legacy.99_idiom2.m07_ext_method](../../../crates/auto-lang/test/vm/99_idiom2/m07_ext_method/ext_method.case.ad) | vm-stdout | 文本 'get=21\ndouble=42\nbumped=6' | `85d9591c0c907cf0` |
| [legacy.99_spec_dispatch.000_spec_basic](../../../crates/auto-lang/test/vm/99_spec_dispatch/000_spec_basic/spec_basic.case.ad) | vm-stdout | 文本 'echo: hello\n>> reversed(hello)\necho: world\n>> reversed(world)' | `fcb3d477fa99ee2d` |
| [legacy.99_spec_dispatch.20_tool_registry](../../../crates/auto-lang/test/vm/99_spec_dispatch/20_tool_registry/tool_registry.case.ad) | vm-stdout | 文本 'echo: hello\nHELLO\nwrote: hello\necho: world\nWORLD\nwrote: world\n20_tool_registry: all assertions passed' | `bf98eefb421fb6bb` |
| [legacy.99_spec_dispatch.31_tool_exec_with_perm](../../../crates/auto-lang/test/vm/99_spec_dispatch/31_tool_exec_with_perm/tool_exec_with_perm.case.ad) | vm-stdout | 文本 'ran: ls\nread: file.txt\nread: data.txt\n--- batch ---\nblocked: Bash not read-only\nok: read: a.txt\n31_tool_exec_with_perm: all ass…（全文见原文档） | `2c369e63929ca583` |

## PLAN-1089 原文档清单

本批主承接4份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.22_generator.001_sum](../../../crates/auto-lang/test/vm/22_generator/001_sum/sum.case.ad) | vm-stdout | 文本 '6' | `3f1e385c356316ef` |
| [legacy.22_generator.002_no_dup](../../../crates/auto-lang/test/vm/22_generator/002_no_dup/no_dup.case.ad) | vm-stdout | 文本 '10,20,30,' | `0177bbc78898da07` |
| [legacy.22_generator.003_string_yields](../../../crates/auto-lang/test/vm/22_generator/003_string_yields/string_yields.case.ad) | vm-stdout | 文本 'abc' | `25f1524ad099cb99` |
| [legacy.22_generator.004_indirect_return](../../../crates/auto-lang/test/vm/22_generator/004_indirect_return/indirect_return.case.ad) | vm-stdout | 文本 '6' | `316f27bd996d4c96` |

## PLAN-1090 原文档清单

本批主承接13份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.23_actor.001_start_hook](../../../crates/auto-lang/test/vm/23_actor/001_start_hook/start_hook.case.ad) | vm-stdout | 文本 'Greeter started\ngot msg' | `b4966d8bcb1e7b28` |
| [legacy.23_actor.002_message_handler](../../../crates/auto-lang/test/vm/23_actor/002_message_handler/message_handler.case.ad) | vm-stdout | 文本 'got one' | `6da25029a7312c62` |
| [legacy.23_actor.003_multi_message](../../../crates/auto-lang/test/vm/23_actor/003_multi_message/multi_message.case.ad) | vm-stdout | 文本 'got one\ngot two\ngot one' | `d53d4f715726523e` |
| [legacy.23_actor.004_else_handler](../../../crates/auto-lang/test/vm/23_actor/004_else_handler/else_handler.case.ad) | vm-stdout | 文本 'fell through' | `5d987e7d95d58481` |
| [legacy.23_actor.005_state_write](../../../crates/auto-lang/test/vm/23_actor/005_state_write/state_write.case.ad) | vm-stdout | 文本 'handler ran' | `025d76f84896b222` |
| [legacy.23_actor.006_state_increment](../../../crates/auto-lang/test/vm/23_actor/006_state_increment/state_increment.case.ad) | vm-stdout | 文本 'reached two\nreached three' | `a4f65da79f9466f6` |
| [legacy.23_actor.007_named_variants](../../../crates/auto-lang/test/vm/23_actor/007_named_variants/named_variants.case.ad) | vm-stdout | 文本 'reached eight\nreset' | `310e128dc7233e40` |
| [legacy.23_actor.008_string_pattern](../../../crates/auto-lang/test/vm/23_actor/008_string_pattern/string_pattern.case.ad) | vm-stdout | 文本 'pong\nfarewell\nunknown' | `cc7b8a0355a64167` |
| [legacy.23_actor.009_stop_hook](../../../crates/auto-lang/test/vm/23_actor/009_stop_hook/stop_hook.case.ad) | vm-stdout | 文本 'started\ngot one\nstopped' | `10ddfe5048bbb883` |
| [legacy.23_actor.010_handle_cross_fn](../../../crates/auto-lang/test/vm/23_actor/010_handle_cross_fn/handle_cross_fn.case.ad) | vm-stdout | 文本 'got one\ngot one' | `597d032cfa11cd95` |
| [legacy.23_actor.011_handle_param](../../../crates/auto-lang/test/vm/23_actor/011_handle_param/handle_param.case.ad) | vm-stdout | 文本 'got one\ngot one' | `73caa4585b520600` |
| [legacy.23_actor.012_external_enum_msg](../../../crates/auto-lang/test/vm/23_actor/012_external_enum_msg/external_enum_msg.case.ad) | vm-stdout | 文本 'got A\ngot B\nhello' | `6c82387bef443a0a` |
| [legacy.23_actor.021_handle_struct_field](../../../crates/auto-lang/test/vm/23_actor/021_handle_struct_field/handle_struct_field.case.ad) | vm-stdout | 文本 'sink got one\nsink got one' | `f22e0db6481c5afb` |

## PLAN-1091 原文档清单

本批主承接0份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|

## PLAN-1092 原文档清单

本批主承接0份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|

## PLAN-1093 原文档清单

本批主承接70份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.17_cffi.001_strlen.strlen](../../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `8fed14f1168cb2b8` |
| [legacy.17_cffi.002_math.math](../../../crates/auto-lang/test/vm/17_cffi/002_math/math.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `216d2ca4468dc916` |
| [legacy.17_cffi.003_string.string](../../../crates/auto-lang/test/vm/17_cffi/003_string/string.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `6e0f93d3bc7a4c8b` |
| [legacy.18_ffi.001_file_exists](../../../crates/auto-lang/test/vm/18_ffi/001_file_exists/file_exists.case.ad) | vm-result | 文本 '0' | `2ae3de356d70f6de` |
| [legacy.18_ffi.003_file_is_dir](../../../crates/auto-lang/test/vm/18_ffi/003_file_is_dir/file_is_dir.case.ad) | vm-result | 文本 '0' | `269eec6bf5e1818a` |
| [legacy.18_ffi.006_string_len](../../../crates/auto-lang/test/vm/18_ffi/006_string_len/string_len.case.ad) | vm-result | 文本 '5' | `58d4a51940eb46a5` |
| [legacy.18_ffi.007_string_is_empty](../../../crates/auto-lang/test/vm/18_ffi/007_string_is_empty/string_is_empty.case.ad) | vm-result | 文本 '1' | `8229f9bc53f8cab0` |
| [legacy.18_ffi.008_string_contains](../../../crates/auto-lang/test/vm/18_ffi/008_string_contains/string_contains.case.ad) | vm-result | 文本 '1' | `bdab4371b3ad3917` |
| [legacy.18_ffi.009_string_starts_with](../../../crates/auto-lang/test/vm/18_ffi/009_string_starts_with/string_starts_with.case.ad) | vm-result | 文本 '1' | `276d43955ff85730` |
| [legacy.18_ffi.010_string_ends_with](../../../crates/auto-lang/test/vm/18_ffi/010_string_ends_with/string_ends_with.case.ad) | vm-result | 文本 '1' | `85d94b49d2414ee8` |
| [legacy.18_ffi.012_char_is_alpha](../../../crates/auto-lang/test/vm/18_ffi/012_char_is_alpha/char_is_alpha.case.ad) | vm-result | 文本 '1' | `494a063442c9b989` |
| [legacy.18_ffi.013_char_is_digit](../../../crates/auto-lang/test/vm/18_ffi/013_char_is_digit/char_is_digit.case.ad) | vm-result | 文本 '1' | `5f3106f938c9604a` |
| [legacy.18_ffi.014_json_is_valid](../../../crates/auto-lang/test/vm/18_ffi/014_json_is_valid/json_is_valid.case.ad) | vm-result | 文本 '1' | `4c166076b69d844f` |
| [legacy.18_ffi.015_json_len](../../../crates/auto-lang/test/vm/18_ffi/015_json_len/json_len.case.ad) | vm-result | 文本 '3' | `82b2322d05fca9a6` |
| [legacy.18_ffi.016_json_is_null](../../../crates/auto-lang/test/vm/18_ffi/016_json_is_null/json_is_null.case.ad) | vm-result | 文本 '1' | `ca19f2671ece89b2` |
| [legacy.18_ffi.017_json_as_bool](../../../crates/auto-lang/test/vm/18_ffi/017_json_as_bool/json_as_bool.case.ad) | vm-result | 文本 '1' | `852ee4cda62bcb4b` |
| [legacy.18_ffi.018_json_has_key](../../../crates/auto-lang/test/vm/18_ffi/018_json_has_key/json_has_key.case.ad) | vm-result | 文本 '1' | `5a82efee83f727d0` |
| [legacy.18_ffi.019_math_abs_f](../../../crates/auto-lang/test/vm/18_ffi/019_math_abs_f/math_abs_f.case.ad) | vm-result | 文本 '3.14' | `caf47c451ff1ec8a` |
| [legacy.18_ffi.020_str_char_at](../../../crates/auto-lang/test/vm/18_ffi/020_str_char_at/str_char_at.case.ad) | vm-result | 文本 '104' | `70da5ff449c10582` |
| [legacy.18_ffi.021_str_substr](../../../crates/auto-lang/test/vm/18_ffi/021_str_substr/str_substr.case.ad) | vm-result | 文本 'hello' | `67ee40554af9b929` |
| [legacy.18_ffi.022_str_trim](../../../crates/auto-lang/test/vm/18_ffi/022_str_trim/str_trim.case.ad) | vm-result | 文本 'hello' | `3eb3bd3411fa9402` |
| [legacy.18_ffi.023_str_repeat](../../../crates/auto-lang/test/vm/18_ffi/023_str_repeat/str_repeat.case.ad) | vm-result | 文本 'ababab' | `24ab7d029c280920` |
| [legacy.18_ffi.024_str_replace](../../../crates/auto-lang/test/vm/18_ffi/024_str_replace/str_replace.case.ad) | vm-result | 文本 'hello auto' | `577fb5b3166eaf5a` |
| [legacy.18_ffi.025_str_case](../../../crates/auto-lang/test/vm/18_ffi/025_str_case/str_case.case.ad) | vm-result | 文本 'HELLO' | `76d9b9317c1bf6a4` |
| [legacy.18_ffi.026_str_reverse_find](../../../crates/auto-lang/test/vm/18_ffi/026_str_reverse_find/str_reverse_find.case.ad) | vm-result | 文本 'olleh' | `30ac1020fea04ef4` |
| [legacy.18_ffi.027_str_find](../../../crates/auto-lang/test/vm/18_ffi/027_str_find/str_find.case.ad) | vm-result | 文本 '6' | `fa8f77864103fd31` |
| [legacy.18_ffi.028_str_replace_first](../../../crates/auto-lang/test/vm/18_ffi/028_str_replace_first/str_replace_first.case.ad) | vm-result | 文本 'hi hello' | `44ea5258fa196a05` |
| [legacy.18_ffi.029_str_match_count](../../../crates/auto-lang/test/vm/18_ffi/029_str_match_count/str_match_count.case.ad) | vm-result | 文本 '3' | `ce7031dfc676801f` |
| [legacy.18_ffi.030_str_contains](../../../crates/auto-lang/test/vm/18_ffi/030_str_contains/str_contains.case.ad) | vm-result | 文本 '1' | `0352326325c866e0` |
| [legacy.18_ffi.031_str_starts_ends](../../../crates/auto-lang/test/vm/18_ffi/031_str_starts_ends/str_starts_ends.case.ad) | vm-result | 文本 '1' | `72be2617e48510c6` |
| [legacy.18_ffi.032_char_is_alpha](../../../crates/auto-lang/test/vm/18_ffi/032_char_is_alpha/char_is_alpha.case.ad) | vm-result | 文本 '1' | `6b8b19df6744177e` |
| [legacy.18_ffi.033_char_is_digit](../../../crates/auto-lang/test/vm/18_ffi/033_char_is_digit/char_is_digit.case.ad) | vm-result | 文本 '1' | `c3ef5faa3770396d` |
| [legacy.18_ffi.034_char_is_alphanum](../../../crates/auto-lang/test/vm/18_ffi/034_char_is_alphanum/char_is_alphanum.case.ad) | vm-result | 文本 '1' | `9f5ed6d2dbe89ef8` |
| [legacy.18_ffi.035_char_is_whitespace](../../../crates/auto-lang/test/vm/18_ffi/035_char_is_whitespace/char_is_whitespace.case.ad) | vm-result | 文本 '1' | `747f8dd8604bccad` |
| [legacy.18_ffi.036_char_to_lower](../../../crates/auto-lang/test/vm/18_ffi/036_char_to_lower/char_to_lower.case.ad) | vm-result | 文本 '97' | `352f853638eb1b19` |
| [legacy.18_ffi.037_char_to_upper](../../../crates/auto-lang/test/vm/18_ffi/037_char_to_upper/char_to_upper.case.ad) | vm-result | 文本 '65' | `d48ddbb03aa485dd` |
| [legacy.18_ffi.038_json_encode_parse](../../../crates/auto-lang/test/vm/18_ffi/038_json_encode_parse/json_encode_parse.case.ad) | vm-stdout | 文本 '"hello"\nobject' | `e76e28c08f40efd0` |
| [legacy.18_ffi.039_json_get](../../../crates/auto-lang/test/vm/18_ffi/039_json_get/json_get.case.ad) | vm-stdout | 文本 '"auto"\nnumber\n1\n0' | `dc1ff2985bd098f3` |
| [legacy.18_ffi.040_json_array](../../../crates/auto-lang/test/vm/18_ffi/040_json_array/json_array.case.ad) | vm-stdout | 文本 '3\n1\n2' | `ced2131c57f45b7b` |
| [legacy.18_ffi.041_json_keys](../../../crates/auto-lang/test/vm/18_ffi/041_json_keys/json_keys.case.ad) | vm-stdout | 文本 '1\n1\n0' | `2f779eb812f11d26` |
| [legacy.18_ffi.042_json_type_as](../../../crates/auto-lang/test/vm/18_ffi/042_json_type_as/json_type_as.case.ad) | vm-stdout | 文本 'string\nhello' | `2af67fa8400aecda` |
| [legacy.18_ffi.043_path_parent](../../../crates/auto-lang/test/vm/18_ffi/043_path_parent/path_parent.case.ad) | vm-stdout | 文本 '/usr/local\n/usr\n/' | `4d2f9d80c48cb6c4` |
| [legacy.18_ffi.044_path_ext_filename](../../../crates/auto-lang/test/vm/18_ffi/044_path_ext_filename/path_ext_filename.case.ad) | vm-stdout | 文本 'gz\nfile.txt\n\nto' | `c2578fc405847192` |
| [legacy.18_ffi.045_path_join](../../../crates/auto-lang/test/vm/18_ffi/045_path_join/path_join.case.ad) | vm-stdout | 文本 'gz\nreadme.md\n/home/user' | `eb3e713044209aaf` |
| [legacy.18_ffi.046_env_get_set](../../../crates/auto-lang/test/vm/18_ffi/046_env_get_set/env_get_set.case.ad) | vm-stdout | 文本 'hello' | `2b4e676c0ece56b2` |
| [legacy.18_ffi.047_time_now.time_now](../../../crates/auto-lang/test/vm/18_ffi/047_time_now/time_now.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `6542df75885fa8db` |
| [legacy.18_ffi.048_url_encode_decode](../../../crates/auto-lang/test/vm/18_ffi/048_url_encode_decode/url_encode_decode.case.ad) | vm-stdout | 文本 'hello%20world\nhello world\na%3Db%26c%3Dd\na=b&c=d' | `73ea1336b3f9ac50` |
| [legacy.18_ffi.049_url_parts](../../../crates/auto-lang/test/vm/18_ffi/049_url_parts/url_parts.case.ad) | vm-stdout | 文本 'https\nexample.com\n8080\n/path/to/page\nname=test\nsection' | `0866947975922d33` |
| [legacy.18_ffi.050_regex_is_match](../../../crates/auto-lang/test/vm/18_ffi/050_regex_is_match/regex_is_match.case.ad) | vm-stdout | 文本 '1\n0\n1\n1\n0' | `fd7c975723888023` |
| [legacy.18_ffi.051_file_write_read](../../../crates/auto-lang/test/vm/18_ffi/051_file_write_read/file_write_read.case.ad) | vm-stdout | 文本 'hello auto\n0' | `0553aa6bf6b8b1d9` |
| [legacy.18_ffi.052_file_copy_append](../../../crates/auto-lang/test/vm/18_ffi/052_file_copy_append/file_copy_append.case.ad) | vm-stdout | 文本 'hello world' | `70234b5af20d0467` |
| [legacy.18_ffi.053_file_dir_ops](../../../crates/auto-lang/test/vm/18_ffi/053_file_dir_ops/file_dir_ops.case.ad) | vm-stdout | 文本 '1\n1\nin dir' | `59769f09c6b880b3` |
| [legacy.18_ffi.054_process_current_dir](../../../crates/auto-lang/test/vm/18_ffi/054_process_current_dir/process_current_dir.case.ad) | vm-stdout | 文本 'ok' | `a9095dc21d7791a8` |
| [legacy.18_ffi.055_process_args.process_args](../../../crates/auto-lang/test/vm/18_ffi/055_process_args/process_args.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `12a743a75480b2e3` |
| [legacy.18_ffi.056_math_pow](../../../crates/auto-lang/test/vm/18_ffi/056_math_pow/math_pow.case.ad) | vm-result | 文本 'true' | `b7b3f44c1c156f45` |
| [legacy.18_ffi.057_str_is_digit](../../../crates/auto-lang/test/vm/18_ffi/057_str_is_digit/str_is_digit.case.ad) | vm-result | 文本 'true' | `86d3877ff842a1f5` |
| [legacy.18_ffi.058_read_text_range](../../../crates/auto-lang/test/vm/18_ffi/058_read_text_range/read_text_range.case.ad) | vm-stdout | 文本 '{"text":"cdef","total":10,"next_offset":6}\n{"text":"ghij","total":10,"next_offset":null}\n{"text":"","total":10,"next_offset":null}\…（全文见原文档） | `4a7ea87d278d2d4f` |
| [legacy.19_rust_std.001_time](../../../crates/auto-lang/test/vm/19_rust_std/001_time/time.case.ad) | vm-stdout | 文本 'ok' | `f1a4a897e358df28` |
| [legacy.19_rust_std.002_duration](../../../crates/auto-lang/test/vm/19_rust_std/002_duration/duration.case.ad) | vm-stdout | 文本 'ok' | `64714773d5ca85c2` |
| [legacy.19_rust_std.003_pathbuf](../../../crates/auto-lang/test/vm/19_rust_std/003_pathbuf/pathbuf.case.ad) | vm-stdout | 文本 'ok' | `7cad4257bf76a1b4` |
| [legacy.19_rust_std.004_duration_print](../../../crates/auto-lang/test/vm/19_rust_std/004_duration_print/duration_print.case.ad) | vm-stdout | 文本 '500ms' | `a43ce71fa5d845d4` |
| [legacy.19_rust_std.005_instant_duration](../../../crates/auto-lang/test/vm/19_rust_std/005_instant_duration/instant_duration.case.ad) | vm-stdout | 文本 '<Instant>\n5000ms\n500ms' | `600721eb5f261f74` |
| [legacy.19_rust_std.006_sync](../../../crates/auto-lang/test/vm/19_rust_std/006_sync/sync.case.ad) | vm-stdout | 文本 '<Arc>' | `0f7a3049890acbb4` |
| [legacy.19_rust_std.007_pathbuf](../../../crates/auto-lang/test/vm/19_rust_std/007_pathbuf/pathbuf.case.ad) | vm-stdout | 文本 'src/main.rs' | `d6edefba4ac13c26` |
| [legacy.19_rust_std.008_box_cell](../../../crates/auto-lang/test/vm/19_rust_std/008_box_cell/box_cell.case.ad) | vm-stdout | 文本 '<Box>\n<RefCell>' | `d879ff96a4a5da41` |
| [legacy.19_rust_std.009_duration_f64](../../../crates/auto-lang/test/vm/19_rust_std/009_duration_f64/duration_f64.case.ad) | vm-stdout | 文本 '1500ms' | `f870bd07e7d6326e` |
| [legacy.19_rust_std.010_process_command](../../../crates/auto-lang/test/vm/19_rust_std/010_process_command/process_command.case.ad) | vm-stdout | 文本 'ok' | `f847dbef98ed7d3b` |
| [legacy.20_permission.001_scalar_mode](../../../crates/auto-lang/test/vm/20_permission/001_scalar_mode/scalar_mode.case.ad) | vm-stdout | 文本 'permission scalar mode: ok' | `adc92630e5c4088a` |
| [legacy.20_permission.002_ext_policy](../../../crates/auto-lang/test/vm/20_permission/002_ext_policy/ext_policy.case.ad) | vm-stdout | 文本 '1\n1\n1\n0\n1' | `4aec5aca9f8db027` |
| [legacy.20_rust_ffi.001_serde_json](../../../crates/auto-lang/test/vm/20_rust_ffi/001_serde_json/serde_json.case.ad) | vm-error | 诊断 legacy.error-presence | `a2d6f1c25624a8ad` |

## PLAN-1094 原文档清单

本批主承接7份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.99_py_dispatch.01_dual_len](../../../crates/auto-lang/test/vm/99_py_dispatch/01_dual_len/dual_len.case.ad) | vm-stdout | 文本 '6\n3' | `68a0c11f84b84b05` |
| [legacy.99_py_dispatch.02_auto_str_unchanged](../../../crates/auto-lang/test/vm/99_py_dispatch/02_auto_str_unchanged/auto_str_unchanged.case.ad) | vm-stdout | 文本 '6\n4\n7' | `6ee3d8ef3828dfd2` |
| [legacy.99_script_err.01_propagate](../../../crates/auto-lang/test/vm/99_script_err/01_propagate/propagate.case.ad) | vm-stdout | 文本 'start\nn=42' | `faf07f5cb454cce3` |
| [legacy.99_script_err.02_catch_value](../../../crates/auto-lang/test/vm/99_script_err/02_catch_value/catch_value.case.ad) | vm-stdout | 文本 "caught: PyException ValueError: invalid literal for int() with base 10: 'abc'\nend" | `5d8717d29d12a942` |
| [legacy.99_script_err.03_uncaught](../../../crates/auto-lang/test/vm/99_script_err/03_uncaught/uncaught.case.ad) | vm-error | 诊断 legacy.error-presence | `66a99f9a6dd6a32a` |
| [legacy.99_script_err.04_with_as_propagate](../../../crates/auto-lang/test/vm/99_script_err/04_with_as_propagate/with_as_propagate.case.ad) | vm-stdout | 文本 "caught: PyException ValueError: invalid literal for int() with base 10: 'xyz'\nend" | `8854b766bf3dfa85` |
| [legacy.99_script_err.05_obj_call_auto_catch](../../../crates/auto-lang/test/vm/99_script_err/05_obj_call_auto_catch/obj_call_auto_catch.case.ad) | vm-stdout | 文本 'caught: TypeError: object is not callable\nend' | `fd5deed3897e023f` |

## PLAN-1095 原文档清单

本批主承接129份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
| [legacy.99_bootstrap.001_token_enum](../../../crates/auto-lang/test/vm/99_bootstrap/001_token_enum/token_enum.case.ad) | vm-stdout | 文本 'green' | `fbf067ba5e658a28` |
| [legacy.99_bootstrap.002_keyword_map](../../../crates/auto-lang/test/vm/99_bootstrap/002_keyword_map/keyword_map.case.ad) | vm-stdout | 文本 '8' | `eb8689a95a600b21` |
| [legacy.99_bootstrap.003_lexer_basic](../../../crates/auto-lang/test/vm/99_bootstrap/003_lexer_basic/lexer_basic.case.ad) | vm-stdout | 文本 'helloworld' | `437f79de65651191` |
| [legacy.99_bootstrap.004_str_slice_let](../../../crates/auto-lang/test/vm/99_bootstrap/004_str_slice_let/str_slice_let.case.ad) | vm-stdout | 文本 'he' | `638b8c5050ca4b54` |
| [legacy.99_bootstrap.005_str_slice_concat](../../../crates/auto-lang/test/vm/99_bootstrap/005_str_slice_concat/str_slice_concat.case.ad) | vm-stdout | 文本 'hello\nprefix:hello\nab' | `97a4bf8338b4a247` |
| [legacy.99_bootstrap.006_bool_compare](../../../crates/auto-lang/test/vm/99_bootstrap/006_bool_compare/bool_compare.case.ad) | vm-stdout | 文本 '0\n1\n0\n1\n1\n0' | `1ddcba489657ffc3` |
| [legacy.99_bootstrap.007_nested_control_flow](../../../crates/auto-lang/test/vm/99_bootstrap/007_nested_control_flow/nested_control_flow.case.ad) | vm-stdout | 文本 '97\n98' | `bece5c8c7ae6b466` |
| [legacy.99_bootstrap.008_parser_hello](../../../crates/auto-lang/test/vm/99_bootstrap/008_parser_hello/parser_hello.case.ad) | vm-stdout | 文本 '(fn main (params) (body (print "hello")))' | `5486e8e899af9a17` |
| [legacy.99_bootstrap.009_parser_arithmetic](../../../crates/auto-lang/test/vm/99_bootstrap/009_parser_arithmetic/parser_arithmetic.case.ad) | vm-stdout | 文本 '(+ (+ 1 2) 3)' | `968fabb5b654efa9` |
| [legacy.99_bootstrap.010_parser_precedence](../../../crates/auto-lang/test/vm/99_bootstrap/010_parser_precedence/parser_precedence.case.ad) | vm-stdout | 文本 '(+ 1 (* 2 3))' | `edde4dfc95f87e09` |
| [legacy.99_bootstrap.011_parser_unary](../../../crates/auto-lang/test/vm/99_bootstrap/011_parser_unary/parser_unary.case.ad) | vm-stdout | 文本 '(- x)' | `3f5b784b7b765a73` |
| [legacy.99_bootstrap.012_parser_not](../../../crates/auto-lang/test/vm/99_bootstrap/012_parser_not/parser_not.case.ad) | vm-stdout | 文本 '(! flag)' | `b6b46896431abeff` |
| [legacy.99_bootstrap.013_parser_comparison](../../../crates/auto-lang/test/vm/99_bootstrap/013_parser_comparison/parser_comparison.case.ad) | vm-stdout | 文本 '(< 1 2)' | `2a43c7e37be3c401` |
| [legacy.99_bootstrap.014_parser_equality](../../../crates/auto-lang/test/vm/99_bootstrap/014_parser_equality/parser_equality.case.ad) | vm-stdout | 文本 '(== a b)' | `6c22fd1b1b574dba` |
| [legacy.99_bootstrap.015_parser_logical](../../../crates/auto-lang/test/vm/99_bootstrap/015_parser_logical/parser_logical.case.ad) | vm-stdout | 文本 '(\|\| a b)' | `eb2fe2185ecee172` |
| [legacy.99_bootstrap.016_parser_let](../../../crates/auto-lang/test/vm/99_bootstrap/016_parser_let/parser_let.case.ad) | vm-stdout | 文本 '(let x 1)' | `4b60a276eeec8941` |
| [legacy.99_bootstrap.017_parser_var](../../../crates/auto-lang/test/vm/99_bootstrap/017_parser_var/parser_var.case.ad) | vm-stdout | 文本 '(var y 2)' | `f444f8749eed7a8c` |
| [legacy.99_bootstrap.018_parser_fn_decl](../../../crates/auto-lang/test/vm/99_bootstrap/018_parser_fn_decl/parser_fn_decl.case.ad) | vm-stdout | 文本 '(fn add (params param param) (body x))' | `aa5d53dfc41172c5` |
| [legacy.99_bootstrap.019_parser_fn_call](../../../crates/auto-lang/test/vm/99_bootstrap/019_parser_fn_call/parser_fn_call.case.ad) | vm-stdout | 文本 '(print 42)' | `7ef31f1a755bad84` |
| [legacy.99_bootstrap.020_parser_if_else](../../../crates/auto-lang/test/vm/99_bootstrap/020_parser_if_else/parser_if_else.case.ad) | vm-stdout | 文本 '(if true (body 1) (else 2))' | `f62797686f28238e` |
| [legacy.99_bootstrap.021_parser_for_in](../../../crates/auto-lang/test/vm/99_bootstrap/021_parser_for_in/parser_for_in.case.ad) | vm-stdout | 文本 '(for i in (.. 1 5) (body i))' | `bd0c1d55406ece64` |
| [legacy.99_bootstrap.022_parser_return](../../../crates/auto-lang/test/vm/99_bootstrap/022_parser_return/parser_return.case.ad) | vm-stdout | 文本 '(return 42)' | `6ffd469a2f9d411c` |
| [legacy.99_bootstrap.023_parser_dot](../../../crates/auto-lang/test/vm/99_bootstrap/023_parser_dot/parser_dot.case.ad) | vm-stdout | 文本 'a.b' | `dd941802cf4facf9` |
| [legacy.99_bootstrap.024_parser_assign](../../../crates/auto-lang/test/vm/99_bootstrap/024_parser_assign/parser_assign.case.ad) | vm-stdout | 文本 '(= x 5)' | `982571de17436dc0` |
| [legacy.99_bootstrap.025_parser_range](../../../crates/auto-lang/test/vm/99_bootstrap/025_parser_range/parser_range.case.ad) | vm-stdout | 文本 '(.. 1 5)' | `431f0f0f15a41812` |
| [legacy.99_bootstrap.026_parser_string](../../../crates/auto-lang/test/vm/99_bootstrap/026_parser_string/parser_string.case.ad) | vm-stdout | 文本 '"hello"' | `71f634901723888a` |
| [legacy.99_bootstrap.027_parser_multi](../../../crates/auto-lang/test/vm/99_bootstrap/027_parser_multi/parser_multi.case.ad) | vm-stdout | 文本 '(let x 1)\n(let y 2)' | `f5f3dcfae02b4ad8` |
| [legacy.99_bootstrap.028_parser_alias](../../../crates/auto-lang/test/vm/99_bootstrap/028_parser_alias/parser_alias.case.ad) | vm-stdout | 文本 '(alias CC MyAdd)' | `0e4e0236f3f06581` |
| [legacy.99_bootstrap.029_parser_enum](../../../crates/auto-lang/test/vm/99_bootstrap/029_parser_enum/parser_enum.case.ad) | vm-stdout | 文本 '(enum Color Red Green Blue)' | `a96a0f38f2ca2963` |
| [legacy.99_bootstrap.030_parser_use](../../../crates/auto-lang/test/vm/99_bootstrap/030_parser_use/parser_use.case.ad) | vm-stdout | 文本 '(use math)' | `cd9c3d5864e31385` |
| [legacy.99_bootstrap.031_parser_spec](../../../crates/auto-lang/test/vm/99_bootstrap/031_parser_spec/parser_spec.case.ad) | vm-stdout | 文本 '(spec Flyer fly)' | `2844c6b8ed7e156a` |
| [legacy.99_bootstrap.032_parser_ext](../../../crates/auto-lang/test/vm/99_bootstrap/032_parser_ext/parser_ext.case.ad) | vm-stdout | 文本 '(ext str (fn len (params) int (body 0)))' | `1cfca4029dddbe44` |
| [legacy.99_bootstrap.033_parser_closure](../../../crates/auto-lang/test/vm/99_bootstrap/033_parser_closure/parser_closure.case.ad) | vm-stdout | 文本 '(closure (x) => (+ x 1))' | `591d702c763b0aa3` |
| [legacy.99_bootstrap.034_parser_closure_multi](../../../crates/auto-lang/test/vm/99_bootstrap/034_parser_closure_multi/parser_closure_multi.case.ad) | vm-stdout | 文本 '(closure (a b) => (+ a b))' | `a1073fbd68507da2` |
| [legacy.99_bootstrap.035_parser_fstr](../../../crates/auto-lang/test/vm/99_bootstrap/035_parser_fstr/parser_fstr.case.ad) | vm-stdout | 文本 '(fstr "Hello " name)' | `e35bf5ccd6abdf12` |
| [legacy.99_bootstrap.036_parser_is](../../../crates/auto-lang/test/vm/99_bootstrap/036_parser_is/parser_is.case.ad) | vm-stdout | 文本 '(is x (-> 1 true) (else false))' | `8632cada61c7ecde` |
| [legacy.99_bootstrap.037_parser_object](../../../crates/auto-lang/test/vm/99_bootstrap/037_parser_object/parser_object.case.ad) | vm-stdout | 文本 '(object (x 1) (y 2))' | `e875785985205fc0` |
| [legacy.99_bootstrap.038_eval_arithmetic](../../../crates/auto-lang/test/vm/99_bootstrap/038_eval_arithmetic/eval_arithmetic.case.ad) | vm-stdout | 文本 'r1:42' | `99b5563b474a9cb9` |
| [legacy.99_bootstrap.039_eval_variable](../../../crates/auto-lang/test/vm/99_bootstrap/039_eval_variable/eval_variable.case.ad) | vm-stdout | 文本 'r1:99' | `74937043df01eb9c` |
| [legacy.99_bootstrap.040_eval_fn_call](../../../crates/auto-lang/test/vm/99_bootstrap/040_eval_fn_call/eval_fn_call.case.ad) | vm-stdout | 文本 'r1:true' | `ee10844a89deff16` |
| [legacy.99_bootstrap.041_eval_if_else](../../../crates/auto-lang/test/vm/99_bootstrap/041_eval_if_else/eval_if_else.case.ad) | vm-stdout | 文本 'r1:false' | `f64753a65a5ea44d` |
| [legacy.99_bootstrap.042_eval_for_loop](../../../crates/auto-lang/test/vm/99_bootstrap/042_eval_for_loop/eval_for_loop.case.ad) | vm-stdout | 文本 'r1:12345' | `becfc411d6de0f88` |
| [legacy.99_bootstrap.043_eval_recursion](../../../crates/auto-lang/test/vm/99_bootstrap/043_eval_recursion/eval_recursion.case.ad) | vm-stdout | 文本 'r1:0' | `3c9a03ef0a1e375b` |
| [legacy.99_bootstrap.044_eval_string](../../../crates/auto-lang/test/vm/99_bootstrap/044_eval_string/eval_string.case.ad) | vm-stdout | 文本 'r1:1\n2\n3' | `58e77c2e44c63773` |
| [legacy.99_bootstrap.045_eval_list](../../../crates/auto-lang/test/vm/99_bootstrap/045_eval_list/eval_list.case.ad) | vm-stdout | 文本 'r1:100\n200' | `cf16c7171316b461` |
| [legacy.99_bootstrap.046_eval_closure](../../../crates/auto-lang/test/vm/99_bootstrap/046_eval_closure/eval_closure.case.ad) | vm-stdout | 文本 'r1:true\nfalse' | `dccf59a9f63cd636` |
| [legacy.99_bootstrap.047_eval_multi_fn](../../../crates/auto-lang/test/vm/99_bootstrap/047_eval_multi_fn/eval_multi_fn.case.ad) | vm-stdout | 文本 'r1:10\n20' | `ad7a6efe0b15d704` |
| [legacy.99_bootstrap.048_eval_str_print](../../../crates/auto-lang/test/vm/99_bootstrap/048_eval_str_print/eval_str_print.case.ad) | vm-stdout | 文本 'r1:hello' | `08ee9e301d319bcd` |
| [legacy.99_bootstrap.049_eval_str_var](../../../crates/auto-lang/test/vm/99_bootstrap/049_eval_str_var/eval_str_var.case.ad) | vm-stdout | 文本 'r1:hello' | `96a4ce9b051d77a3` |
| [legacy.99_bootstrap.050_eval_str_concat](../../../crates/auto-lang/test/vm/99_bootstrap/050_eval_str_concat/eval_str_concat.case.ad) | vm-stdout | 文本 'r1:ab' | `23563c33d4c7083c` |
| [legacy.99_bootstrap.051_eval_str_fn_arg](../../../crates/auto-lang/test/vm/99_bootstrap/051_eval_str_fn_arg/eval_str_fn_arg.case.ad) | vm-stdout | 文本 'r1:world' | `67bfd8a1d6a9359a` |
| [legacy.99_bootstrap.051a_eval_str_fn_arg_simple](../../../crates/auto-lang/test/vm/99_bootstrap/051a_eval_str_fn_arg_simple/eval_str_fn_arg_simple.case.ad) | vm-stdout | 文本 'r1:6' | `4c14425164b970ba` |
| [legacy.99_bootstrap.052_eval_str_return](../../../crates/auto-lang/test/vm/99_bootstrap/052_eval_str_return/eval_str_return.case.ad) | vm-stdout | 文本 'r1:ok' | `4b4c911e1517decd` |
| [legacy.99_bootstrap.053_eval_str_literal](../../../crates/auto-lang/test/vm/99_bootstrap/053_eval_str_literal/eval_str_literal.case.ad) | vm-stdout | 文本 'direct_params=1\nmap_params=1' | `6d7962fefc41cf0e` |
| [legacy.99_bootstrap.054_type_infer_basic](../../../crates/auto-lang/test/vm/99_bootstrap/054_type_infer_basic/type_infer_basic.case.ad) | vm-stdout | 文本 'x:0\ns:1\nb:2' | `a5752981f48401be` |
| [legacy.99_bootstrap.055_type_infer_fn](../../../crates/auto-lang/test/vm/99_bootstrap/055_type_infer_fn/type_infer_fn.case.ad) | vm-stdout | 文本 'add:0\ngreet:1\nadd_p0:0\ngreet_p0:1' | `c03ba9efb7a14262` |
| [legacy.99_bootstrap.060_bytecode_int](../../../crates/auto-lang/test/vm/99_bootstrap/060_bytecode_int/bytecode_int.case.ad) | vm-stdout | 文本 '3' | `f35674f098bfc99f` |
| [legacy.99_bootstrap.061_bytecode_var](../../../crates/auto-lang/test/vm/99_bootstrap/061_bytecode_var/bytecode_var.case.ad) | vm-stdout | 文本 'ok' | `4cfe4b5a54ee2a91` |
| [legacy.99_bootstrap.062_bytecode_fn](../../../crates/auto-lang/test/vm/99_bootstrap/062_bytecode_fn/bytecode_fn.case.ad) | vm-stdout | 文本 'ok' | `dc3919ed0f6502fb` |
| [legacy.99_bootstrap.063_bytecode_if](../../../crates/auto-lang/test/vm/99_bootstrap/063_bytecode_if/bytecode_if.case.ad) | vm-stdout | 文本 'ok' | `21b479b4afe3c10a` |
| [legacy.99_bootstrap.064_bytecode_for](../../../crates/auto-lang/test/vm/99_bootstrap/064_bytecode_for/bytecode_for.case.ad) | vm-stdout | 文本 'ok' | `99b0117f319036fe` |
| [legacy.99_bootstrap.065_bytecode_str](../../../crates/auto-lang/test/vm/99_bootstrap/065_bytecode_str/bytecode_str.case.ad) | vm-stdout | 文本 'ok' | `fe29d592c31fe09f` |
| [legacy.99_bootstrap.066_bytecode_list.bytecode_list](../../../crates/auto-lang/test/vm/99_bootstrap/066_bytecode_list/bytecode_list.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `50966acb8c09c30a` |
| [legacy.99_bootstrap.067_bytecode_map.bytecode_map](../../../crates/auto-lang/test/vm/99_bootstrap/067_bytecode_map/bytecode_map.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `586545f3f4c51aca` |
| [legacy.99_bootstrap.068_bytecode_list_fn.bytecode_list_fn](../../../crates/auto-lang/test/vm/99_bootstrap/068_bytecode_list_fn/bytecode_list_fn.case.ad) | vm-smoke | 无具体值/流oracle（smoke/有限角色），T-01不得计完整语义证明 | `545da1974338b6e9` |
| [legacy.99_bootstrap.069_bvm_str_len](../../../crates/auto-lang/test/vm/99_bootstrap/069_bvm_str_len/bvm_str_len.case.ad) | vm-stdout | 文本 'ok' | `96a1ad226d8bac36` |
| [legacy.99_bootstrap.070_bvm_str_methods](../../../crates/auto-lang/test/vm/99_bootstrap/070_bvm_str_methods/bvm_str_methods.case.ad) | vm-stdout | 文本 'ok' | `766e2a232013c034` |
| [legacy.99_bootstrap.071_bvm_map_str](../../../crates/auto-lang/test/vm/99_bootstrap/071_bvm_map_str/bvm_map_str.case.ad) | vm-stdout | 文本 'ok' | `48e6166446add4c8` |
| [legacy.99_bootstrap.072_bvm_list_set](../../../crates/auto-lang/test/vm/99_bootstrap/072_bvm_list_set/bvm_list_set.case.ad) | vm-stdout | 文本 'ok' | `81ef6e702e164f63` |
| [legacy.99_bootstrap.073_bvm_list_pop](../../../crates/auto-lang/test/vm/99_bootstrap/073_bvm_list_pop/bvm_list_pop.case.ad) | vm-stdout | 文本 'ok' | `2eec6ccd4423cf18` |
| [legacy.99_bootstrap.074_bvm_str_ops](../../../crates/auto-lang/test/vm/99_bootstrap/074_bvm_str_ops/bvm_str_ops.case.ad) | vm-stdout | 文本 'ok' | `7c99dd42b88441e2` |
| [legacy.99_bootstrap.075_list_str_push_get](../../../crates/auto-lang/test/vm/99_bootstrap/075_list_str_push_get/list_str_push_get.case.ad) | vm-stdout | 文本 'ok' | `d1aa7b03fca32646` |
| [legacy.99_bootstrap.076_list_str_set](../../../crates/auto-lang/test/vm/99_bootstrap/076_list_str_set/list_str_set.case.ad) | vm-stdout | 文本 'ok' | `966766938af5162d` |
| [legacy.99_bootstrap.077_list_str_pop](../../../crates/auto-lang/test/vm/99_bootstrap/077_list_str_pop/list_str_pop.case.ad) | vm-stdout | 文本 'ok' | `d66b466a96b7974b` |
| [legacy.99_bootstrap.078_list_mixed_types](../../../crates/auto-lang/test/vm/99_bootstrap/078_list_mixed_types/list_mixed_types.case.ad) | vm-stdout | 文本 'ok' | `f1e664696ff47664` |
| [legacy.99_bootstrap.079_list_str_fn_arg](../../../crates/auto-lang/test/vm/99_bootstrap/079_list_str_fn_arg/list_str_fn_arg.case.ad) | vm-stdout | 文本 'ok' | `798b41a910adeca7` |
| [legacy.99_bootstrap.080_list_str_loop](../../../crates/auto-lang/test/vm/99_bootstrap/080_list_str_loop/list_str_loop.case.ad) | vm-stdout | 文本 'ok' | `22d0b8906a27f96b` |
| [legacy.99_bootstrap.081_a2r_hello](../../../crates/auto-lang/test/vm/99_bootstrap/081_a2r_hello/a2r_hello.case.ad) | vm-stdout | 文本 'ok' | `51df51b684262696` |
| [legacy.99_bootstrap.082_a2r_fn](../../../crates/auto-lang/test/vm/99_bootstrap/082_a2r_fn/a2r_fn.case.ad) | vm-stdout | 文本 'ok' | `4690338f5841aa03` |
| [legacy.99_bootstrap.083_a2r_var](../../../crates/auto-lang/test/vm/99_bootstrap/083_a2r_var/a2r_var.case.ad) | vm-stdout | 文本 'ok' | `e783095d3cc5cc7d` |
| [legacy.99_bootstrap.084_a2r_if](../../../crates/auto-lang/test/vm/99_bootstrap/084_a2r_if/a2r_if.case.ad) | vm-stdout | 文本 'ok' | `0567a958ef2023db` |
| [legacy.99_bootstrap.085_a2r_for](../../../crates/auto-lang/test/vm/99_bootstrap/085_a2r_for/a2r_for.case.ad) | vm-stdout | 文本 'ok' | `98f4c1d7316cffad` |
| [legacy.99_bootstrap.086_a2r_str](../../../crates/auto-lang/test/vm/99_bootstrap/086_a2r_str/a2r_str.case.ad) | vm-stdout | 文本 'ok' | `ba11ac4aac29b2b5` |
| [legacy.99_bootstrap.087_a2r_use](../../../crates/auto-lang/test/vm/99_bootstrap/087_a2r_use/a2r_use.case.ad) | vm-stdout | 文本 'ok' | `0f72a5c9e40363ba` |
| [legacy.99_bootstrap.088_a2r_type](../../../crates/auto-lang/test/vm/99_bootstrap/088_a2r_type/a2r_type.case.ad) | vm-stdout | 文本 'ok' | `80f7dbd35ba83453` |
| [legacy.99_bootstrap.089_a2r_enum](../../../crates/auto-lang/test/vm/99_bootstrap/089_a2r_enum/a2r_enum.case.ad) | vm-stdout | 文本 'ok' | `65a5dceb8f7d0afc` |
| [legacy.99_bootstrap.090_a2r_is](../../../crates/auto-lang/test/vm/99_bootstrap/090_a2r_is/a2r_is.case.ad) | vm-stdout | 文本 'ok' | `d44b14608400558c` |
| [legacy.99_bootstrap.091_a2r_ext](../../../crates/auto-lang/test/vm/99_bootstrap/091_a2r_ext/a2r_ext.case.ad) | vm-stdout | 文本 'ok' | `5298484da689ac11` |
| [legacy.99_bootstrap.092_a2r_fstr](../../../crates/auto-lang/test/vm/99_bootstrap/092_a2r_fstr/a2r_fstr.case.ad) | vm-stdout | 文本 'ok' | `f2895b40b8ed609f` |
| [legacy.99_bootstrap.093_a2r_spec](../../../crates/auto-lang/test/vm/99_bootstrap/093_a2r_spec/a2r_spec.case.ad) | vm-stdout | 文本 'ok' | `bd04bc63471e7550` |
| [legacy.99_bootstrap.094_a2r_closure](../../../crates/auto-lang/test/vm/99_bootstrap/094_a2r_closure/a2r_closure.case.ad) | vm-stdout | 文本 'ok' | `8957cb4d317e9224` |
| [legacy.99_bootstrap.095_a2r_alias](../../../crates/auto-lang/test/vm/99_bootstrap/095_a2r_alias/a2r_alias.case.ad) | vm-stdout | 文本 'ok' | `0570ef8dc8175722` |
| [legacy.99_bootstrap.096_a2r_object](../../../crates/auto-lang/test/vm/99_bootstrap/096_a2r_object/a2r_object.case.ad) | vm-stdout | 文本 'ok' | `2b32a3785c04dc6a` |
| [legacy.99_bootstrap.097_a2r_array](../../../crates/auto-lang/test/vm/99_bootstrap/097_a2r_array/a2r_array.case.ad) | vm-stdout | 文本 'ok' | `3752708e1381eac5` |
| [legacy.99_bootstrap.098_a2r_error_prop](../../../crates/auto-lang/test/vm/99_bootstrap/098_a2r_error_prop/a2r_error_prop.case.ad) | vm-stdout | 文本 'ok' | `ed3ddc0a968d3737` |
| [legacy.99_bootstrap.099_a2r_self_field](../../../crates/auto-lang/test/vm/99_bootstrap/099_a2r_self_field/a2r_self_field.case.ad) | vm-stdout | 文本 'ok' | `13a5c23978b474c4` |
| [legacy.99_bootstrap.100_a2r_use_ffi](../../../crates/auto-lang/test/vm/99_bootstrap/100_a2r_use_ffi/a2r_use_ffi.case.ad) | vm-stdout | 文本 'ok' | `ea010b20d061a644` |
| [legacy.99_bootstrap.101_a2r_is_multi](../../../crates/auto-lang/test/vm/99_bootstrap/101_a2r_is_multi/a2r_is_multi.case.ad) | vm-stdout | 文本 'ok' | `db4cf28d3b175951` |
| [legacy.99_bootstrap.102_a2r_struct_ctor](../../../crates/auto-lang/test/vm/99_bootstrap/102_a2r_struct_ctor/a2r_struct_ctor.case.ad) | vm-stdout | 文本 'ok' | `2d87a0ea84547b00` |
| [legacy.99_bootstrap.103_a2r_generic_type](../../../crates/auto-lang/test/vm/99_bootstrap/103_a2r_generic_type/a2r_generic_type.case.ad) | vm-stdout | 文本 'ok' | `e27adfd4fd59fc20` |
| [legacy.99_bootstrap.104_a2r_generic_vec](../../../crates/auto-lang/test/vm/99_bootstrap/104_a2r_generic_vec/a2r_generic_vec.case.ad) | vm-stdout | 文本 'ok' | `cb6e1d8bce2bcb7a` |
| [legacy.99_bootstrap.105_a2r_generic_map](../../../crates/auto-lang/test/vm/99_bootstrap/105_a2r_generic_map/a2r_generic_map.case.ad) | vm-stdout | 文本 'ok' | `e429df8905a70460` |
| [legacy.99_bootstrap.106_a2r_generic_enum](../../../crates/auto-lang/test/vm/99_bootstrap/106_a2r_generic_enum/a2r_generic_enum.case.ad) | vm-stdout | 文本 'ok' | `77830d9c654c828e` |
| [legacy.99_bootstrap.107_a2r_option_match](../../../crates/auto-lang/test/vm/99_bootstrap/107_a2r_option_match/a2r_option_match.case.ad) | vm-stdout | 文本 't1:ok\nt2:ok\nt3:ok' | `b06ba9894ffc7a0d` |
| [legacy.99_bootstrap.108_a2r_ownership](../../../crates/auto-lang/test/vm/99_bootstrap/108_a2r_ownership/a2r_ownership.case.ad) | vm-stdout | 文本 't1:ok\nt2:ok\nt3:ok\nt4:ok' | `85994567e687cb5c` |
| [legacy.99_bootstrap.110_a2r_phase3](../../../crates/auto-lang/test/vm/99_bootstrap/110_a2r_phase3/a2r_phase3.case.ad) | vm-stdout | 文本 'pos:ok\nfn:ok\ntype:ok\nenum:ok\nis:ok' | `519b42b664d3bcb4` |
| [legacy.99_bootstrap.111_a2r_real_source](../../../crates/auto-lang/test/vm/99_bootstrap/111_a2r_real_source/a2r_real_source.case.ad) | vm-stdout | 文本 't1_pos:ok\nt2_error:ok\nt3_token:ok\nt4_ast:ok\nt5_parser:ok\nt6_lexer:ok\nt7_a2r:ok\nt8_ext:ok\nt9_use:ok\nt10_is:ok' | `f71be27298d87bd6` |
| [legacy.99_bootstrap.112_a2r_generic_registry](../../../crates/auto-lang/test/vm/99_bootstrap/112_a2r_generic_registry/a2r_generic_registry.case.ad) | vm-stdout | 文本 't1:ok\nt2:ok\nt3:ok\nt4:ok\nt5:ok' | `2c7ccc2a14ea8ee0` |
| [legacy.99_idiom_probe.p01_is_string](../../../crates/auto-lang/test/vm/99_idiom_probe/p01_is_string/is_string.case.ad) | vm-stdout | 文本 'exact match\nor-arm match\nruntime concat match\nelse branch\ngreeting\nother' | `c79425bc78908ba4` |
| [legacy.99_idiom_probe.p02b_enum_or_arm](../../../crates/auto-lang/test/vm/99_idiom_probe/p02b_enum_or_arm/enum_or_arm.case.ad) | vm-stdout | 文本 'num\neof\nnum\nnum\neof' | `cab49fbb08b4df49` |
| [legacy.99_idiom_probe.p03_enum_payload](../../../crates/auto-lang/test/vm/99_idiom_probe/p03_enum_payload/enum_payload.case.ad) | vm-stdout | 文本 'int: 42\nstring: hello\nno payload\nparam int: 42\nparam str: hello\nparam none\nreturned: result' | `915730283ec54c8b` |
| [legacy.99_idiom_probe.p04_runtime_concat_payload](../../../crates/auto-lang/test/vm/99_idiom_probe/p04_runtime_concat_payload/runtime_concat_payload.case.ad) | vm-stdout | 文本 'ab' | `77523c3f78f77e6e` |
| [legacy.99_idiom_probe.p05_double_match](../../../crates/auto-lang/test/vm/99_idiom_probe/p05_double_match/double_match.case.ad) | vm-stdout | 文本 '42\n42' | `1262f0073b637904` |
| [legacy.99_idiom_probe.p06_while_continue](../../../crates/auto-lang/test/vm/99_idiom_probe/p06_while_continue/while_continue.case.ad) | vm-stdout | 文本 'cond 1\ncond 3\ncond 4\ntotal: 3' | `8e86147bb462b93d` |
| [legacy.99_idiom_probe.p07_cond_for_continue](../../../crates/auto-lang/test/vm/99_idiom_probe/p07_cond_for_continue/cond_for_continue.case.ad) | vm-stdout | 文本 'cond 1\ncond 3\ncond 4\nrange 0\nrange 1\nrange 3' | `fc69635c0c998b8c` |
| [legacy.99_idiom_probe.p08_guard_arm](../../../crates/auto-lang/test/vm/99_idiom_probe/p08_guard_arm/guard_arm.case.ad) | vm-stdout | 文本 'large' | `755fbe12974d7dbb` |
| [legacy.99_idiom_probe.p09g_local_var_workaround](../../../crates/auto-lang/test/vm/99_idiom_probe/p09g_local_var_workaround/local_var_workaround.case.ad) | vm-stdout | 文本 '1\nab' | `6608b76a13522357` |
| [legacy.99_idiom_probe.p09h_inline_enum_payload](../../../crates/auto-lang/test/vm/99_idiom_probe/p09h_inline_enum_payload/inline_enum_payload.case.ad) | vm-stdout | 文本 '1\nab' | `6935a5dbaf188d74` |
| [legacy.99_idiom_probe.p10_enum_reassign_eq](../../../crates/auto-lang/test/vm/99_idiom_probe/p10_enum_reassign_eq/enum_reassign_eq.case.ad) | vm-stdout | 文本 'on\noff' | `d950d9af1bb8946c` |
| [legacy.99_idiom_probe.p11_is_char](../../../crates/auto-lang/test/vm/99_idiom_probe/p11_is_char/is_char.case.ad) | vm-stdout | 文本 'ex\nnot xy' | `b7db0e2e653c11b9` |
| [legacy.99_idiom_probe.p12_is_binding_types](../../../crates/auto-lang/test/vm/99_idiom_probe/p12_is_binding_types/is_binding_types.case.ad) | vm-stdout | 文本 '42\ntxt' | `f7a46eca1c7f19aa` |
| [legacy.99_idiom_probe.p13a_struct_field_enum](../../../crates/auto-lang/test/vm/99_idiom_probe/p13a_struct_field_enum/struct_field_enum.case.ad) | vm-stdout | 文本 'hoisted\n42' | `e21891d9d7871dd4` |
| [legacy.99_idiom_probe.p13b_struct_literal_field](../../../crates/auto-lang/test/vm/99_idiom_probe/p13b_struct_literal_field/struct_literal_field.case.ad) | vm-stdout | 文本 'frame_test' | `ede8a565b0d3d642` |
| [legacy.99_idiom_probe.p13c_struct_literal_literal_payload](../../../crates/auto-lang/test/vm/99_idiom_probe/p13c_struct_literal_literal_payload/struct_literal_literal_payload.case.ad) | vm-stdout | 文本 '7' | `105c07697de602f1` |
| [legacy.99_misc.002_closure_by_ref](../../../crates/auto-lang/test/vm/99_misc/002_closure_by_ref/closure_by_ref.case.ad) | vm-stdout | 文本 '15\n3\n100\n200' | `f6e7172a803b06fe` |
| [legacy.99_p588debt.001_inline_arg_accum](../../../crates/auto-lang/test/vm/99_p588debt/001_inline_arg_accum/inline_arg_accum.case.ad) | vm-stdout | 文本 'direct:\n24\nvia-let:\n24' | `282264b64bd3e030` |
| [legacy.99_p588debt.002_str_return_fn](../../../crates/auto-lang/test/vm/99_p588debt/002_str_return_fn/str_return_fn.case.ad) | vm-stdout | 文本 'len:501\ntrue' | `f10a7954fa9074ec` |
| [legacy.99_p588debt.003_operand_shapes](../../../crates/auto-lang/test/vm/99_p588debt/003_operand_shapes/operand_shapes.case.ad) | vm-stdout | 文本 'S120\nF544\nD2728' | `f3d41732665541e0` |
| [legacy.99_plan664.001_value_lexemes](../../../crates/auto-lang/test/vm/99_plan664/001_value_lexemes/value_lexemes.case.ad) | vm-stdout | 文本 '99_plan664_value_lexemes: all assertions passed' | `7999a6387fa5f14b` |

## PLAN-1096 原文档清单

本批主承接0份；即使主承接为0，也必须覆盖路线L族、内联/外仓可达语言输入及本Plan新AutoDown反例，不能用0表示无能力责任。

| 原case ID / 作者源 | 原check ID | 原作者声明的oracle摘要 | 作者文档SHA16 |
|---|---|---|---|
