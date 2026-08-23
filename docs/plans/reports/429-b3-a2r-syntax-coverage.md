# Plan 429 B3 报告：a2r 对 AAVM 移植所需 Auto 语法构造覆盖面盘点

> 生成方式：plan-429 B3 调研（只读 agent + 人工复核），2026-08-23。
> 数据源：`crates/auto-lang/test/a2r/`（254 个 .at golden 输入）、`crates/auto-lang/src/trans/rust.rs`（21,065 行）、`docs/plans/242-a2r-feature-gap-tracker.md`。
> 用例路径均相对 `crates/auto-lang/test/a2r/`，下称 `T/`。

## 总表

| 构造 | 状态 | 证据 | AAVM 影响评估 |
|---|---|---|---|
| 1 fn 定义（含返回类型） | ✅ | T/01_basics/003_func、conformance/027/028/029 | 无风险，基础全绿 |
| 1 let / var / const | ✅ | let/var 遍布全部用例；const：T/14_modules/006_const_decl；shared var（全局）：T/14_modules/007_shared_var（Lazy<Mutex> 包装，rust.rs:218 注释） | 模块级 shared var 用 Lazy<Mutex<T>>，AAVM 移植 lexer/parser 的全局表可用 |
| 1 if/else | ✅ | T/03_control_flow/001~004、013 if_tail_value；conformance/003/015 | 无风险 |
| 1 for-in range / 条件循环 | ✅ | range：T/03/005_for_range、conformance/004；条件式 `for i < max`：T/03/006_for_conditions | 条件 for 转 Rust while，写法已验证 |
| 1 while / loop / break / continue | ✅ | while：conformance/040_while_basic；loop+break：T/03/007_while_loop、conformance/016/017 | 无风险 |
| 2 type struct 声明与构造 | ✅ | T/02_types/001_struct（`Point(1,2)` 与 `Point { x: 3 }` 两种构造，见 06/002）；conformance/006 | 无风险 |
| 2 enum/tag（含 payload） | ✅ | 标量 enum：T/02/002_enum；tag+payload：T/06/004_hetero_enum（`Atom.Int(i)`）、005 泛型 payload；conformance/019/020 | 无风险，hetero enum 是 parser 移植主力写法 |
| 2 字段可变性 | ⚠️ | 结构体字段无显式 per-field mut 语法；方法级 `mut fn`（&mut self）有 golden：T/02/008_mut_self；字段赋值经 `var` 绑定（T/02/001 `p.x = 3`） | AAVM 代码写 `var` 绑定 + `mut fn` 即可，避免发明字段级 mut |
| 2 嵌套 struct | ✅ | T/02/001 `Circle.center Point` + `circle.center.x` 链；T/02/009_self_field | 无风险 |
| 3 is-match 字面量 + else | ✅ | T/03/008_is_match（`0 -> ... else -> ...`）；非穷尽（无 else）：T/03/010_is_non_exhaustive | 无风险 |
| 3 is-match 多语句 arm | ✅ | T/03/009_is_multi_stmt；T/06/008_hetero_enum_multistmt | 无风险 |
| 3 is-match binding 解构（struct/enum） | ✅ | T/06/002_struct_destructure（`Point { x, y }` 嵌套 is，242 #5 已完成）；qualified unit variant：T/06/009 | 无风险 |
| 3 Option/Result 匹配 | ✅ | Result Ok/Err arm：T/09/033_result_is_match；构造：T/09/002_option_construct；Option 字段类型：T/09/031 | ⚠️ `is opt { Some(v)/None }` 直接形式未见专门 golden，建议补一个 |
| 4 闭包 `x => e` / `(a,b) => e` | ✅ | T/11/004_closure_infer（`x => x + 1` 与 `(x int) => x*2`）；T/11/003 | 无风险 |
| 4 闭包捕获环境变量（by value/ref 升级） | ✅ | T/19/002_closure_capture（.view 捕获升级）；T/25_lifecycle/001~004（Rc 升级 / copy 捕获 / 只读捕获保持 plain）；move 闭包：T/19/009_move_closure | 捕获升级策略已系统化，优先用已覆盖写法 |
| 4 闭包作为参数 | ✅ | T/19/010_fn_closure_params（`cb fn(str) void` + `fn(ev){}` 匿名 fn 形式）；T/12/007_box_fn | 无风险 |
| 4 闭包作为返回值 | ⚠️ | 无直接"返回闭包"golden；最接近 T/12/007_box_fn（Box<dyn Fn> spec 参数）与 T/25/001（捕获升级 Rc）。242 #8 closure inference 仅 8a 已修 | 移植期避免返回闭包；如需则进 242 #8 |
| 5 fn 级 `<T>` | ✅ | T/08/008_bounded_generic_fn（`idf<T>`）；T/08/005_with_constraint | 无风险 |
| 5 type 级泛型 | ✅ | T/08/003_generic_field、004_generic_ptr_field、001_type_alias；const 泛型：T/08/002 | 无风险 |
| 5 turbofish | ✅ | T/05/012_turbofish（`.parse<uint>()`、`fn<int,str>(...)`、与 `<` 比较消歧） | 无风险 |
| 5 有界泛型 `<T has Spec>` | ✅ | T/08/008_bounded_generic_fn（`<T has Comparable>` → `<T: Comparable>`）；多 bound：T/16/019_multi_bound；242 #1 已完成 | 无风险 |
| 6 spec 声明 + 默认方法 | ✅ | T/12/001~004（默认方法体：004_default_body，注明 DIV-TRAIT-A2R-3：默认体访问 self. 字段仍有缺口） | spec 默认体内不要引用 self 字段/兄弟方法（242 DIV-TRAIT-A2R-3） |
| 6 ext Type for Trait | ✅ | T/11/006_ext_for、T/02/010_ext_keyword；242 #6 已完成（rust.rs:1580） | 无风险 |
| 6 关联类型 | ✅ | T/12/011_associated_types（`type Item` + `as Container<Item=int>` 命名绑定）；012/013 泛型 spec | 无风险 |
| 7 .view / .mut / .move / .take | ✅ | view：T/07/001_borrow_view；mut：T/07/002_borrow_mut；move：T/07/003_borrow_move；take（弃用别名）：rust.rs:2292/3556/2656-2657 | `.move`/`.take` 均退化为原表达式（Rust 默认移动），golden 齐全 |
| 7 `.?` 错误传播 | ✅ | T/09/004_error_propagate + 005~025（uint/float/str/bool/嵌套调用/算术/比较/字面量等 20+ 变体）、T/09/011_question_propagate | 覆盖极充分，无风险 |
| 8 use.rust（::{ } / : 项 / 通配） | ✅ | 基础：T/14/001_rust_use；通配：rust.rs:12764；项列表：rust.rs:12781 `use X::{a,b}`；伴生 trait 导入 rust.rs:12770+ | 无风险；多文件 crate 解析也齐（T/14/005_multi_file） |
| 8 use 普通路径 | ✅ | T/14/002_pub_use、003_pub_visibility、004_wildcard_import；auto.* → a2r_std 映射 rust.rs:12675+ | 无风险 |
| 8 use.c / use.py | ✅/❌ | use.c：rust.rs:12761-12763 显式忽略；use.py：rust.rs:12875-12878 直接报错 "not supported in Rust target"。golden：T/02/007_cstr 含 use.c | AAVM 纯 Rust 移植不应用 use.py；use.c 静默忽略需注意链接缺失 |
| 9 f-string（含 ${expr} 非平凡插值） | ✅ | T/04/001_fstring（`$name`、`${x+y}`、`${a*b}` 混合）；边界：T/04/002；多行：T/04/006；backtick：T/04/004 | 覆盖充分 |
| 9 数组字面量 / 下标 | ✅ | T/10/001_array；conformance/010_array_index；泛型下标 T/06/006 | 无风险 |
| 9 对象字面量（Map） | ✅ | T/10/006_map_literal（→ HashMap::from，242 #2 已修）；hash_map_ops：T/17/017 | 无风险 |
| 9 范围表达式 | ✅ | T/05/006_range_expr | 无风险 |
| 9 位运算 | ⚠️ | 无 `<<`/`>>` 运算符（T/05/011_no_left_shift 专门规避用 `* 2`）；方法形式 `.and/.or/.xor/.shl/.shr` 代码支持（rust.rs:5436-5460 wrapping_shl、5778+）但无任何 golden | AAVM lexer/codegen 位操作须用 `.and()/.shl()` 方法形式并补 golden；`<<` 运算符是 blocker 候选 |
| 9 as 转换 | ✅ | T/15/001_type_cast（`.as(u32/i64/float)`）；`.to(Type)`：T/15/002_to_convert（242 #3） | 无风险 |
| 9 字符串方法链 | ✅ | T/17/015_string_methods（push_str/len/trim/replace）；方法链：T/10/005_method_chain；list map/filter：conformance/023 | 无风险 |
| 10 模块级 let（全局变量） | ⚠️ | `shared var` 有 golden（T/14/007）；裸模块级 `let` 仅 T/04/006_multi_fstr | 全局建议统一用 `shared var`；裸顶层 let 覆盖薄 |
| 10 相互递归 fn / 前向引用 | ⚠️ | 自递归 golden：conformance/028_recursive_func；相互递归/前向引用无专门 golden。a2r 有 fn 签名预注册（rust.rs:12563） | 中风险：建议补"相互递归 + 先用后声明"golden 进 242 #13 |
| 10 深嵌套 match | ✅ | T/06/002（两层嵌套 is）；T/06/008 | 两层已验证；≥3 层未专门测，风险低 |
| 10 大整数 | ⚠️ | `int→i32 / uint→u32` 映射（T/05/012 注释）；未见 i64/u64 专门 golden | AAVM 若需 64 位须先确认 long/i64 映射，进 242 tracker |
| 10 注释保留 | ✅ | T/01/004_doc_comments；rust.rs:11131/12900/13997 | 无风险 |

## 与 242 tracker 的交叉核对

- tracker 已确认完成：#1 泛型约束、#2 HashMap 字面量、#3 `.to()`、#5 struct 解构、#6 `ext for`、#12 a2r 发射侧 async（golden T/16/001/002/020）。
- 仍开放且影响 AAVM：#7 String vs &str（Partial，AAVM 字符串密集会踩）；#8 闭包类型推断剩余；#11 所有权精确分析（现为 Rc/clone workaround）；#13 边缘用例。
- **本盘点新增 tracker 候选**：位运算 `<<`/`>>` 运算符缺失；`.and()/.shl()` golden；i64/u64 映射 golden；相互递归/前向引用 golden；`is Option{Some/None}` golden；闭包作为返回值。

## 对 plan-432 的建议

1. **优先使用已覆盖写法**：fn/let/var/const/shared var；type struct（两种构造）、hetero enum + payload；is 全家；闭包仅作参数不作返回值；泛型全家（含 turbo fish/有界）；spec + ext for + 关联类型（默认方法体不引用 self 字段）；.view/.mut/.move 显式标注（不依赖推断）；`.?`；use.rust 全形态；f-string/字面量/方法链。
2. **需进 242 tracker 的新条目**：见上节"新增 tracker 候选"。
3. **Blocker 候选**：
   - 位运算 `<<`/`>>` 运算符不支持（lexer/codegen 移植必需；短期统一用 `.shl()/.shr()/.and()` 方法形式 + 补 golden）；
   - 242 #7 String/&str 区分——需团队规范（如统一 `String` + `.view` 借用）；
   - 242 #11 所有权启发式——AAVM 的 AST/类型结构有共享图，移植时显式 `.view/.mut` 不依赖推断；
   - `use.py` 报错——规范层面禁用即可，非真正 blocker。
