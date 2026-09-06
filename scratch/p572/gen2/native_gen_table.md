
# P532 原生代际对拍判定表(2026-09-06 16:05:30)

exe¹: /tmp/aavm2-bin-71b25d8c7332bdec-e800322fbc0bcb59/target/release/aavm2_bin.exe

## 一代(exe¹ vs 宿主 oracle,代表集)
| corpus | exe¹==host |
|---|---|
| b01_hello | PASS |
| b07_fib | PASS |
| b08_strcat | PASS |
| b13_eval_print_true | PASS |
| b27_arr_literal | PASS |
| b30_arr_loop | PASS |
| b46_list_basic | PASS |
| b58_str_methods | PASS |
一代代表集:8/8 PASS

## 二代(exe² vs exe¹,代表集 + 转译固定点)
| corpus | exe²==exe¹ |
|---|---|
| b01_hello | PASS |
| b07_fib | PASS |
| b08_strcat | PASS |
| b13_eval_print_true | PASS |
| b27_arr_literal | PASS |
| b30_arr_loop | PASS |
| b46_list_basic | PASS |
| b58_str_methods | PASS |

转译固定点(exe¹ --trans == exe² --trans):PASS

结论:一代 PASS / 二代 PASS(附录 B 原生代际形态;判定表完)
