# plan-429 B1: AAVM shim 需求盘点报告

- 扫描范围(15 文件,核心自举粗口径,未剔 UI 段——parser.rs 全量计入,偏保守):

  - crates\auto-lang\src\token.rs
  - crates\auto-lang\src\lexer.rs
  - crates\auto-lang\src\error.rs
  - crates\auto-lang\src\parser.rs
  - crates\auto-lang\src\types.rs
  - crates\auto-lang\src\ast.rs
  - crates\auto-lang\src\infer\context.rs
  - crates\auto-lang\src\infer\expr.rs
  - crates\auto-lang\src\infer\stmt.rs
  - crates\auto-lang\src\infer\functions.rs
  - crates\auto-lang\src\infer\unification.rs
  - crates\auto-lang\src\vm\opcode.rs
  - crates\auto-lang\src\vm\codegen.rs
  - crates\auto-lang\src\vm\engine.rs
  - crates\auto-lang\src\vm\native_catalog.rs
- 方法: 启发式 receiver 推断(let 注解/构造器/函数参数),未匹配 receiver 的调用单列


## 1. std 全限定路径使用频率

- std::collections::: 41
- std::sync::: 36
- std::fmt::: 24
- std::mem::take: 11
- std::sync::atomic::: 10
- std::cmp::: 9
- std::error::: 6
- std::iter::once: 6
- std::rc::: 6
- std::cell::: 4
- std::time::: 4
- std::fs: 3
- std::fmt: 2
- std::io::: 2
- std::path::: 2
- std::env::var: 2
- std::process::: 2
- std::iter::: 1
- std::str::: 1
- std::cmp::max: 1
- std::result::: 1
- std::i32: 1
- std::mem::replace: 1
- std::mem::transmute: 1
- std::env: 1
- std::backtrace::: 1
- std::cmp::min: 1
- std::fs::: 1

## 2. 核心 receiver 类型的 方法需求 vs dispatch 3000 覆盖


### String

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| push | 112 | ❌ |
| to_string | 39 | ❌ |
| into | 27 | ❌ |
| len | 23 | ❌ |
| as_str | 22 | ❌ |
| push_str | 22 | ❌ |
| clone | 9 | ❌ |
| as_bytes | 8 | ❌ |
| is_empty | 7 | ❌ |
| chars | 5 | ❌ |
| into_bytes | 5 | ❌ |
| starts_with | 4 | ❌ |
| ends_with | 4 | ❌ |
| clear | 3 | ❌ |
| trim | 3 | ❌ |
| replace | 3 | ❌ |
| split | 2 | ❌ |
| next | 1 | ❌ |
| peek | 1 | ❌ |
| strip_prefix | 1 | ❌ |
| trim_matches | 1 | ❌ |
| pop | 1 | ❌ |
| to_uppercase | 1 | ❌ |
| to_lowercase | 1 | ❌ |
| contains | 1 | ❌ |
| lock | 1 | ❌ |

### Vec

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| push | 305 | ✅ |
| len | 101 | ✅ |
| iter | 69 | ❌ |
| is_empty | 40 | ❌ |
| last | 36 | ❌ |
| into_iter | 20 | ❌ |
| clone | 18 | ❌ |
| get | 14 | ❌ |
| reverse | 10 | ❌ |
| extend | 8 | ❌ |
| pop | 7 | ❌ |
| contains_key | 7 | ❌ |
| join | 6 | ❌ |
| first | 4 | ❌ |
| into | 2 | ❌ |
| as_str | 2 | ❌ |
| remove | 2 | ❌ |
| insert | 2 | ❌ |
| sort_by_key | 2 | ❌ |
| as_any | 2 | ❌ |
| sort_by | 2 | ❌ |
| dedup | 2 | ❌ |
| sort | 2 | ❌ |
| as_slice | 1 | ❌ |
| contains | 1 | ❌ |
| to_vec | 1 | ❌ |
| first_arg | 1 | ❌ |
| is_some | 1 | ❌ |
| as_ref | 1 | ❌ |
| clear | 1 | ❌ |

### HashMap

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| insert | 95 | ❌ |
| get | 7 | ❌ |
| entry | 1 | ❌ |
| set | 1 | ❌ |
| is_empty | 1 | ❌ |
| clone | 1 | ❌ |

### HashSet

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| into_iter | 2 | ❌ |
| insert | 1 | ❌ |
| len | 1 | ❌ |
| iter | 1 | ❌ |

### Option

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| as_deref | 2 | ❌ |
| is_some | 1 | ❌ |
| replace | 1 | ❌ |
| is_none | 1 | ❌ |

### Arc

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| read | 14 | ❌ |
| write | 3 | ❌ |
| all_fn_decls | 1 | ❌ |
| clone | 1 | ✅ |

### str

| 方法 | 使用次数 | dispatch 3000 |
|---|---|---|
| to_string | 36 | ❌ |
| as_ref | 14 | ❌ |
| as_str | 8 | ❌ |
| chars | 5 | ❌ |
| trim | 3 | ❌ |
| clone | 3 | ❌ |
| len | 2 | ❌ |
| contains | 2 | ❌ |
| split | 2 | ❌ |
| push | 2 | ❌ |
| starts_with | 2 | ❌ |
| as_bytes | 1 | ❌ |
| is_empty | 1 | ❌ |
| trim_start_matches | 1 | ❌ |
| strip_prefix | 1 | ❌ |
| append | 1 | ❌ |
| lines | 1 | ❌ |
| rsplit | 1 | ❌ |

## 3. 未匹配 receiver 的方法频率(前 80,待 431 边界定稿后二次归类)

- clone: 837
- next: 791
- is_kind: 778
- emit: 468
- to_string: 371
- as_str: 353
- as_ref: 336
- len: 326
- expect: 276
- get: 255
- push: 244
- skip_empty_lines: 183
- push_i32: 149
- compile_expr: 140
- insert: 134
- pop_nv: 132
- iter: 126
- read: 109
- is_empty: 102
- into: 97
- push_nv: 84
- peek: 80
- to_node: 78
- extend_from_slice: 78
- contains: 75
- emit_load_loc: 72
- to_le_bytes: 70
- parse_type: 69
- pos: 62
- parse_expr: 62
- pop_i32: 62
- parse_name: 59
- as_bytes: 57
- as_any: 56
- is_ok: 55
- add_string: 55
- parse: 53
- compile_stmt: 52
- emit_placeholder_i16: 49
- define: 48
- starts_with: 48
- lock: 48
- emit_store_loc: 47
- contains_key: 46
- emit_i32: 46
- lookup_type: 45
- add_var: 43
- body: 42
- pop: 42
- rc_push: 41
- rc_push_str_idx: 40
- write: 39
- patch_jump: 39
- get_heap_object: 39
- push_token: 38
- kind: 37
- advance: 36
- to_atom_str: 35
- add_kid: 35
- rc_release: 35
- bind_var: 34
- borrow: 34
- read_u8: 34
- push_scope: 32
- add_arg: 31
- pop_f32: 31
- expect_ident: 30
- emit_u16: 29
- pop_f64: 29
- read_u32: 28
- pop_scope: 26
- exit_scope: 26
- to_atom: 26
- emit_u32: 26
- insert_heap_object: 25
- unwrap: 24
- infer_object_type: 24
- lookup_meta: 23
- method: 23
- repr: 23
## 4. 结论（2026-08-23）

1. **缺口比预估大一个量级**：dispatch 3000 对四个核心容器类型仅覆盖 7 个方法
   （Vec.new/len/push、HashMap.new、HashSet.new、String.new/from_utf8），而核心自举范围
   需要约 **66 个**（String 26 / Vec 30 / HashMap 6 / HashSet 4，含少量误报如 as_any/lock
   来自非容器 receiver 的启发式误配，真实约 55-60 个）。
2. **计划修正（429-B1 → 430-E）**：原计划"高频缺口当场手写补臂（约十几个）"不再执行——
   缺口 60+ 且 Plan 430 Phase E 将为 String/Vec/HashMap 系统性生成整套 shim，
   现在手写是重复劳动。本报告直接作为 430-E 的裁剪输入。
3. **对 432 的影响**：VM 模式跑 AAVM 硬依赖 430-E 完成（或至少覆盖本报告方法面）。
   432 的 S1/S2（lexer/parser 移植）不依赖 shim，可先行。
4. 误报说明：§2 表中 Vec 列的 as_any/lock/is_some/contains_key 等来自 receiver 推断误配
   （heap 对象/Option 等），归类时剔除；§3 未匹配表需 431 边界定稿后二次扫描。
