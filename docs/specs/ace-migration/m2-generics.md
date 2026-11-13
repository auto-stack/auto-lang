# ACE 通用泛型、约束见证与方法实例化（M2-A）

> PLAN-963；实现位于 `experimental/ace/grammars/auto-source.at`（`scalar-instances` /
> `record-instances` / `text-record-instances` 能力裁剪面与 `generic-call` 原子）、
> `core/src/source/cst/{project,kinds}.rs`（投影/原子）、`core/src/source/{ast,resolve,adapter}.rs`
> （声明/替换/约束/方法决议）、`packages/binding.rs`（跨模块模板与实例符号）、
> `semantics/src/{input,infer}.rs`（多参输入/延迟约束求解）、`source-query/src/{instances,instance_host}.rs`
> （实例身份/页 codec v4）、`crates/av`、`crates/native`、`crates/a2x-worker`（四端消费）。
> 本文件只记录**已验证**的支持、接口、拒绝与边界；未列入的能力不视为已支持。
> 语义基线为 959 分母 L-24/38/39/40/42/47 与本包 T-01 冻结（计划 §5.5/§9 收据，2026-11-12）。
> 四端同一矩阵（AV / ac-native / a2x-rust / a2x-c）。本包为 M2-A；B（全产品替换）仍 pending，
> 由 985/A 在 M2A-08 正式收据后消费，不因本文件改写为已交。

## 支持入口与身份

泛型面经三个能力裁剪 profile 显式进入：`scalar-instances` / `record-instances` /
`text-record-instances`（record / text-record 全表面 + fn 类型参数；门控按 profile 追加，
既有 profile 的过滤后程序不变，grammar identity 不动）。调用点显式类型/值实参为
`generic-call` 原子（括号属原子自身：`ident < const-args > ( args )`）。

**实例身份与页兼容**：935 单参身份**保持字节不变**（logical/key 生成顺序对 N=1 等价），
`INSTANCES_*` 常量、935 键/页语义不动；多参/值参数身份按声明序交错 `(type|value)` 项，
值参数同值规范化 =（声明类型 tag, 宽度内 bits）→ 小写 hex 入键，装配名（如 `_u32_2`）只作
拼写非身份。页 codec 演进 v1→v4：v4 增 `constc`（值参数）字段；v1..v3 页按版本不符
MISS 重产（信封不变，旧页不重释）。

## 分母行支持（L-24/38/39/40/42/47）

| 行 | 支持 | 表示 |
|---|---|---|
| L-24 默认/命名实参 | 部分 | 声明位默认值 `fn f(n int = 42)` 补位（**只求值一次**、按调用者作用域解析、不进语义身份/缓存键）、命名实参按**源序**求值后按名绑定（逆序命名调用亦源序）；「可变实参」无用户声明位拼写，以负控钉边界（D-963-C2） |
| L-38 alias | ✓ | `type N = T` 别名形（`type-tail` 按首 token 分派 `=`），投影期展开、spec 默认体注入后**再展开一次**；递归别名具名拒 |
| L-39 spec/约束/关联类型 | ✓ | `spec S { fn m(..) R [body] }`、泛型 `spec S<T>`、`<T has S>` 类型参数约束、`as S`/`as S<int>` 遵从、super-spec `spec S: A + B`、关联类型 `spec C { type Item }`（`as C<Item=int>`）；缺抽象方法/签名不符/未知/成环/重复/通用 super-spec 具名拒，spec 默认体作 View 方法注入（显式实现优先） |
| L-40 ext/方法接收者 | ✓ | `ext Type { members }`（首 token `ext` 独立分派）；候选层级自有方法 > bound spec > ext，层内 >1 复用 `ambiguous-candidates` 拒，ext 与自有方法同名 `source.duplicate` 拒 |
| L-42 泛型 type/const 参数/实例 | ✓ | 用户泛型记录 `type Point<T>` 语法期单态化 + 显式类型实参构造 `Point<int> { .. }`；值参数 `<const N>`（缺省 u32，`<const N u32>` 显式声明可解析；`<N u32>`/`<N uint>` 不可解析）`rep<2>(21)`；跨模块模板单态化（`project_program_deferring_templates` + `monomorphize_package_program`），两包同名模板天然不串、跨模块同一实参共享同一实例 |
| L-47 调用语义/函数值 | 部分 | 默认/命名实参如上；**管道 `|>` 未实现**（D-963-C1）；函数值签名/返回推导沿既有静态解析面 |

## 语法与语义要点（已验证拼写）

- **类型参数**：`fn identity<T>(x T) T`、`type Point<T> { x T; y T }`、`<T, U>` 多参、
  `<T has S>` 约束（fn 与 type 模板共用 `TypeParamDecl`）；无模板时显式类型实参具名拒。
- **值参数**：`<const N>`（缺省 u32 声明类型；非实例面/非 u32 声明类型具名拒
  `source.instances-unsupported`）；`> i32::MAX`/`> u32::MAX` → `source.literal-range`；
  arity 错 `value-arity-mismatch` / `value-argument-required`；模板体 `Var(值参数名)` 代入
  `IntLit`（`subst_block_consts`），`ConstCall` 脱糖为 `Call` 后进四端。
- **实例身份**：`InstanceRecord.ty_param+ty` → `args: Vec<InstanceArg>`，logical key 不含
  span/callsite/body 版本；`signature_canonical` 值参数标签 `\0value-param\0`+name
  （无值参数签名逐字节不变）；表达式编码标签 `G`。
- **约束与见证**：`apply_spec_conformance` claim 模型（每条遵从 → spec 方法义务闭包 +
  具体化替换表）；`<T has S>` 合成前按实参记录核验 bound（未知 bound 名/标量实参/未遵从
  记录具名拒，不合成"无约束"实例）；负依赖读取入依赖图（`instances_invalidation_963`）。
- **默认实参（D-963-C5 边界，本文件明写）**：默认值在**空局部作用域**解析，fail-closed 成
  `source.undefined`（`fn f(a int, b int = a)` 与引用模块全局的默认值不接受）；补位实参恒
  `moved: false`，`move` 形参不能靠默认值满足；位置式只补末尾缺席位、中段缺席仍
  `source.argument`；命名式可补任意缺席位并以参数名标注；legacy 默认参数为 parser-only，
  无行为 oracle 可继承，本边界为 963 自定。

## 四端证据（V 行具名入口）

- **V-01/V-05**：`source-query/tests/instances_multi_963.rs` + `instance_host_935.rs`
  （跨包同名模板身份、恢复后首实例与冷编一致）。
- **V-02/V-03/V-G07**：`instances_invalidation_963.rs`（约束/实现变化使实例失效、未读
  body 零生产、`av_generic_record_body_change_executes_301`）。
- **V-04/V-G03**：core `source_generics_963.rs` / `source_specs_generic_963.rs` /
  `source_spec_super_assoc_963.rs`（bound 满足/不满足具名拒、缺抽象方法拒、默认继承/覆盖、
  `max_of` 函数级 bound 真实族）。
- **V-G01/V-G04**：`instances_const_963.rs`、`instances_value_domain_963.rs`
  （含 `builtin_list_does_not_replace_user_generic_type_963` 负控——内建 `List` 不产实例页）。
- **V-G02**：av/native/a2x `generic_records_963.rs`（`Point<int>` 字段读写 → 300）。
- **V-G05**：core `source_alias_963.rs` / `source_ext_963.rs` / `source_default_args_963.rs`
  （含 `variadic_parameter_form_is_rejected_963`）与 native `alias_963.rs` / `ext_963.rs` /
  `spec_assoc_963.rs` / `spec_generic_963.rs`。
- **V-G06**：`module_packages_960.rs`（`packages_963_owned_generic_parameters_and_returns_cross_modules`：
  lib 声明 `Holder<T>`、app 以 owned `Str` 实例化，`use_it()` = len("m2") = 2）。
- **V-G08**：av/native/a2x 三份 `const_instances_963.rs` + `default_args_963.rs` +
  `generic_records_963.rs`（真实 rustc/cl、exit code 42/70/42 与 AV 值域；oracle 手算闭式
  `rep<2>(21)`=42 / `rep<3>(21)`=63 / 相加=105）。
- **V-08**：`av_default_argument_evaluates_exactly_once_963`（MIR 默认值 callee 恰 1 次 + 值 42）、
  `av_named_arguments_evaluate_in_source_order_963`（MIR 指令序 `d_five` 先于 `d_twenty`，值 15）、
  `two_ext_blocks_with_the_same_method_are_rejected`（`source.duplicate`）、
  `packages_963_owned_generic_parameters_and_returns_cross_modules`。
- **V-PHASE**：固定组并集 `cargo t ace-source ace-syntax ace-semantics ace-av ace-native
  ace-a2x case-ace-m2-generics-963` → `run-2v_8abue`，七组 PASS：2453 个 Rust 用例 +
  10 个 case check（`case-ace-m2-generics-963`，语料 `crates/auto-lang/test/cases/ace-m2/generics/`
  `g01.choose42`/`g02.point300`/`g03.bounded5`/`g05.picker42` + 2 负控），`errors=0`、`not_run=0`；
  3 条条件排除均为既有 `#[ignore]`（`run_exe_60s_deadline_bounds_descendant_pipe_hold`、
  a2x `inventory_941` 两条），非本计划引入，不充 PASS。

## 拒绝与边界（具名诊断）

`source.instances-unsupported`（非实例面/非 u32 const 声明/无能力值参数）、
`source.literal-range`（const 值域）、`value-arity-mismatch` / `value-argument-required`、
`source.instances-bound`（不满足 bound，点名模板与 spec）、`source.duplicate`（重名/重复遵从/
ext 与自有方法同名）、`source.argument`（缺位/超元数/中段缺席）、`source.undefined`
（默认值引用不可见名）、`source.unsupported`（未知 spec/缺抽象方法/签名不符/`#[with(T)]`
迁移拒/模板方法/非模板泛型类型名）、`source.syntax`（`List<int>.new()` 内置模板拼写）、
`source.type-mismatch`（`Point<int>{ x: true }`）、`source.view-escape`（视图物化）、
`source.module-symbol`（跨模块未知类型）、`ambiguous-candidates`（同层候选 >1）。
已知拼写债：D-963-C1..C7（下节）。

## 具名未闭缺口（owner 与关闭条件）

- **D-963-C1 管道算子 `|>`**（L-47）：legacy parser 期脱糖；ACE 全库零命中。owner：963 T-08
  后续 / 965 运算符面。关闭：`x |> .m(y)` 与 `x |> .f` 两形态四端真实运行各 1 正例 + 1 负控。
- **D-963-C2 可变实参**：AutoLang 无用户声明位拼写，legacy 唯一 varargs 是 builtin
  `List.new(1,2,3,4,5)`（无类型参数）；ACE collections 支持矩阵为显式 `List<T>.new()` + `push`
  （962 交付）。owner：962 collections 支持矩阵裁定者。关闭：对 `List.new(a,b,c)` 形给显式裁定。
- **D-963-C3 `#[with(T)]` 遵从注解形**：ACE 无语言属性语法（`visibility.attributes` 仍
  `MetaSupport::SchemaOnly`，owner 为 language attribute grammar plan），963 以
  `source.unsupported` 具名迁移拒；规范形 `fn f<T>` / `fn f<T has S>` / `type N as Spec`。
- **D-963-C4 记录类型模块全局 `.move` 转移**：`module.global.set:<sym>` 伪签名只有借用形，
  owned 跨参转移未实现；native mn 亦不支持 module globals。owner：960 模块面 follow-up。
- **D-963-C5 默认实参求值作用域**：见上节（本文件已明写边界）。owner：963 T-08 后续。
- **D-963-C6 ACE workspace 无 rustfmt 门禁**：master 基线已非 clean，本计划 20 个触碰文件
  同状。owner：ACE workspace 治理。
- **D-963-C7 泛型局部类型注解诊断质量**：`let b Box<int> = …` 退化 `source.unsupported`
  （文案与病因无关）；`List<int>.new()` 得 `source.syntax`。owner：ACE 语法/投影诊断面。
  关闭：未知/跨 profile 泛型类型名给专名诊断，局部泛型注解文案与病因一致。

## M2 阶段口径（SD-PHASE）

本包 = M2-A：能力（类型/值参数、约束见证、spec/ext/alias、跨模块实例、默认/命名实参）
与四端真实验证均在本包完成并归档；**B（全产品替换）未交付**，保持 pending，不改 NA。
B 开工前须核 M2A-08 正式收据（本文件落盘 + 963 归档）；由 985/A 消费并按 Design55
§16 继续。
