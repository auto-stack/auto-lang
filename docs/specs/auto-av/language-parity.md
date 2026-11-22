# AV 语言能力与来源映射（AV language parity ledger）

> Status: **delivered（PLAN-1078 AVP-00 / PLAN-1079 AVP-01，2026-11-19）**。本文件由 AVP-00 首次建立，
> 后续 AVP 批次（1079–1096）串行增量维护；它是当前 AV 语言**能力/来源映射**，不是第二
> 注册器——suite/固定组/发现仍由既有 TestCase/auto-case 机制各负其责。
> 只登记实际交付与真实运行事实；未交行保 pending/owner，不写 implemented。

## 1. 普通 AV 语言入口（AVP-00 交付）

- **SourceProfile::Auto**，profile 串 `auto.source.language.av/v1`，能力键
  `ac.source.language.av/v1`。首批语义 = aggregates 面（同一共享
  Schema/CST → 绑定 → Checked HIR → MIR → ABC → AV 管道；不新开第二
  parser/解释器）。后续 AVP 批次在 Auto 面增量扩充；冻结窄 profile 身份不变。
- **recipe `ac-av/language-source-run-v1`**（父侧 `RecipeFamily::LanguageSource`）：
  int 入口值 + 精确 stdout/stderr 双流 + 编译/运行失败；不转旧 VM。worker 链：
  `compile_language_source → lower_checked_closures → verify_mir → encode_abc →
  verify_abc_bytes → execute(+ConsoleCapture)`，core 身份 `av.closures/1`。
- **双流省略补零（user ruling）**：console 族 recipe（console-run-v1 /
  scheduler-source-run-v1 / language-source-run-v1）加载时把省略的 stdout/stderr
  独立补成预期零字节；显式流按原预期比较；省略流一旦实际输出一个字节（含换行）
  即判失败。其余检查族省略语义不变。
- **检查级 `#ignore`（AV-ONLY-1078-1096）**：非 AV 检查（AC/A2R/A2C/A2PY、
  旧 AVM 对照）在检查小节加 `#ignore` → 映射 `CheckSpec.skip`（reason=AV阶段
  暂缓非AV检查、issue=AV-ONLY-1078-1096、owner=PLAN-1078）；不 prepare/
  dispatch，不阻 AV 门；后端任务逐项移除。`#ignore` 不标整 Case/Auto 源。

## 2. 公共语言语料（feature corpus）

- 唯一活跃作者根：`crates/auto-lang/test/cases/language/`；公共总选择
  `crates/auto-lang/test/cases/language.suite.ad`。同一 Case 一份 Auto 源，
  AV/AC/A2R 各自追加目标检查与独立实际结果，不复制源码。
- 固定组 `case-av-language-parity` 通过 `--target av` 选择已注册的 AV run 检查。
  legacy-reference 等旧对照由各自的显式选择入口处理。
- 迁移表 `language/migration/legacy-map.tsv`（数据映射，不作注册器）；旧义务
  保原 ID/条件/oracle，合并不删义务。

## 3. 本包 case → owner 映射（AVP-00，真实运行）

| Case ID | 源 | AV oracle | 实际结果 | owner |
|---|---|---|---|---|
| `av.parity.1078.baseline-value` | `fn main() int { return 2 + 3 * 4 }` | exit 14, 双流空 | PASS（14） | PLAN-1078 |
| `av.parity.1078.short-circuit` | `if false && … / if true \|\| …`（fail 不运行） | exit 42, 双流空 | PASS（42） | PLAN-1078 |
| `av.parity.1078.syntax-neighbor` | `fn main() int { return (40 + 2 }` | parse / `source.syntax` | PASS（diag） | PLAN-1078 |

- 非 AV 检查（baseline-value 的 ac-native 与 legacy-reference）带 `#ignore`
  （AV-ONLY-1078-1096），本轮不执行；ac-native 曾真实执行 PASS exit 14 的实证
  记录于 plan-artifacts（AV 阶段前的最小同源多端对照），后续 AC 任务打开后重跑。
- 旧义务承接：`legacy.aavm2.001_smoke`/`002_hello_compile` → research-hold
  （aavm2 族保 research/手动条件；语言面 overlap = console print + void main，
  不充 PASS）；L-04（i128/f32 后缀）/L-46（spread/set/when/match/global）无此形、
  L-45（NavCall）parser-only —— 记录适用状态，不实现。

## 4. 支持/拒绝边界（AVP-00 实际）

- 支持（首批=aggregates 面）：scalar 运算、短路、`fail` 语言错误、console 双流、
  parse 阶段 `source.syntax` 诊断（stage 映射 `text→parse`）。
- 未交/拒绝：task actor、脚本/Python 协议、泛型/闭包/生成器/async 等后续 AVP
  批次能力不因 Auto 面未覆盖而豁免；旧窄 profile 拒绝面不变。
- 平台：Windows 执行；Linux/WSL 腿按裁定归专用 Linux 环境，不判红、不充 PASS。

## 5. AVP-01（PLAN-1079）交付：标量、转换与文本

> 2026-11-19，普通 Auto 入口真机运行（`ac-av/language-source-run-v1`）。
> 只登记实际观察；未交行在 §5.3 具名并写 owner/关闭条件。

### 5.1 普通入口新增/修复的准入事实

- **affix-num 基数体切分**（`core/src/source/cst/project.rs::decode_typed_literal`）：
  `0x`/`0b` 前缀后的体按**目标基数**的数字位切分（16 进制 `a..f` 属体），
  十进制另收一个分点；"至少有一个数位"的门也按同一基数判定。因此
  `0xFF`、`0xF`、`0xFFu8`、`0b1010`、`1_000_000`、`40i64`、`42i32` 全部解码；
  此前 Auto 面报 `numeric literal ... carries no digits`（`source.literal-range`）
  或 `suffixed/radix/float numeric literal ... is not part of this source profile`。
- **i32 类型化常量渲染**（`adapter.rs`）：`EKind::TypedConst { vt: I32 }` 渲染
  `{value}i32`。缺这条臂时输出 `value: I32`，Atom bind 拒绝整个模块
  （`internal adapter error` / `constant value must be an integer value`），
  无后缀 affix-num（`1_000_000`、`0b1010`）因此不可用。
- **16 进制体与后缀的歧义裁定**：基数体取**最长**的该基数数字位串，因此
  `0x1Fb` 是十六进制数 507（不是 `0x1F` + `b` byte 后缀）—— 后缀在
  十六进制数字后不可用，十六进制 byte 用 `0xFFu8` 或 `0xFF.as(u8)` 表达；
  十进制/二进制下 //d 后缀不受影响。旧解码器对该形无裁定（直接报错），
  本批按此规则冻结并由 plan1079_language_semantics.rs 精确断言。
- 结论：Auto 面的字面量面（宽度/后缀/基数/分隔符）与 Numeric 面同一解码器，
  不是第二套解析。

### 5.2 本包 canonical Case 与真实结果

| Case ID | 作者文件 | 独立 oracle | 实际结果 |
|---|---|---|---|
| `av.parity.1079.wide-exact` | `language/numbers/literals-conversions.case.ad` | exit 42（i64 超 2^53 精确、`0xFFu8`、`0b1010u8`、`1_000_000`、i64::MIN 加减） | PASS |
| `av.parity.1079.mixed-width-explicit` | 同上 | exit 42（`to(int)`/`as(i64)`/`as(u64)`、`=` 复合算术、`& \| ^` 同层、`.shl/.shr`、整除取余） | PASS |
| `av.parity.1079.overflow-is-not-wrap` | 同上 | run / `av.trap.int-overflow`（i32 MAX+1 不 wrap） | PASS（trap） |
| `av.parity.1079.text-nul-and-format` | `language/text/interpolation-unicode.case.ad` | exit 42 + stdout `A\nB\n`（`len`=4/`len_bytes`=9/NUL/astral/FStr 源序恰一次/Str 相等/负数显示） | PASS |
| `av.parity.1079.text-literal-escapes` | 同上 | exit 42（`\n`、`\t\\\"`、`\u{4e2d}`、`to_codepoint` 十进制与 `0x4E2D` 双拼写） | PASS |

- 固定组 `case-av-language-parity` 复跑 8/8（1078 的 3 例 + 本包 5 例），
  单例 0.45–0.54s。
- 位型补充观察（父侧 Rust 类型化观测，`av/tests/cases/plan1079_language_semantics.rs`）：
  运行时 `x % ±0.0` 产生 NaN（自不等/无序/不 trap）、`-0.0` 经取负/相加/f64→f32
  收窄保持符号位、f64→f32 半程按 ties-to-even、`i64→int` 越界 trap
  `av.trap.int-overflow`、混宽 `source.type-mismatch`、超宽度字面量
  `source.literal-range`。

### 5.3 具名边界（未交付；owner + 关闭条件）

| 形态 | 实际观察 | owner | 关闭条件 |
|---|---|---|---|
| 复合赋值语句 `x += …` | `source.syntax`（`expected expression, found =`）；Auto 语句面只收 `=`，同一 checked 加法经 `x = x + 1` 已验 | statement-surface 批（未排号；PLAN-1080 r3 合同不含 `+=` 语句面——1079 原标注修正） | 语句面接纳该形后按原 case oracle 重跑 |
| `char.to(Str)`（char→owned Str 装箱） | CST 前门拒绝（回退到 legacy 解析器报 `fn is not part of this source profile`） | PLAN-975 后续批次 | 该形可编译后按 `len`/`len_bytes` 断言 |
| `~` 位非、raw string 字面量 | 不在冻结算符表 / 无此形（`text.invalid-escape`） | — | 语言不定义该构造，旧义务记 `no-such-shape` |

### 5.4 AVP-01 承接的 L 行验收状态（AC-04 映射）

路线设计 `docs/design/strategy/av-autovm-language-parity.md` §6 把下列 L 行指派给 AVP-01。
每行给 owner / 预期 / 当前状态 / actual 证据；子项缺失的行在状态列具名，不静默省略。
映射数据行见 `crates/auto-lang/test/cases/language/migration/legacy-map.tsv`（L-01/02/03/05/49）。

| L 行 | 预期（已定语义） | 状态 | actual 证据（本包真实运行） |
|---|---|---|---|
| L-01 整数字面量与后缀（i32/uint/i8/u8/i64/u64） | 六种宽度与后缀可解码，`int`=i32 | **mapped** | canonical `av.parity.1079.wide-exact`（`0xFFu8`/`0b1010u8`/`1_000_000`/`9007199254740993i64`/i64::MIN 折叠）+ `av.parity.1079.mixed-width-explicit`（`5b`/`7u`/`7u32`）；`plan1079_language_semantics::numeric_profile_suffix_vocabulary_decodes_exactly` 精确钉住 `0x1Fi8`→I8 31、`0b101b`→U8 5、`1_000_000u32`→U32、`0xFFu8`→U8、`0xFF`→I32。legacy `int=i64` 与隐式晋升是具名差异。 |
| L-02 float/double 字面量与 NaN/-0 | `float`=f32、`double`=f64，NaN 无字面量语法，-0 位型保留 | **mapped** | `av.parity.1079.mixed-width-explicit`：`1.5f32`/`1.5d` 各自宽度解码、`(1.5f32).as(f64)` 精确加宽、`-0.0d == 0.0d`；位型由 `negative_zero_bits_survive_arithmetic_and_narrowing`（-0 经取负/相加/f64→f32 收窄）与 `nan_from_zero_modulo_is_a_value_not_a_trap`（运行期 `x % ±0.0` 得 NaN）观测。 |
| L-03 byte/char/CStr 字面量 | char 为一标量；byte 为 u8；CStr 属旧拼写 | **mapped（partial）** | `av.parity.1079.text-literal-escapes`：`'中'`/`'\n'`/`'\u{4e2d}'` 与 `to_codepoint`；`av.parity.1079.mixed-width-explicit`：`5b` byte 字面量。**CStr 子项 no-such-shape**（ACE `TypeWord` 无 CStr 拼写），C 字符串互操作归 1093 foreign surface，不计 PASS。 |
| L-05 位运算/移位/符号扩展 | `& \| ^` 同层、`.shl/.shr` 有界、符号/零扩展 | **mapped** | `av.parity.1079.mixed-width-explicit`：`((0xF0&0x3C)\|0x01) = 0x31`（同层判别）、`.shl(5u32)`/`.shr(5u32)`、`(7u8).as(u64)` 零扩展；符号扩展 `(-1i8).as(i64)` 与宽位值由同执行端的 961 `av_numeric_semantics` v08 行覆盖。 |
| L-49 cast `as` / 转换 `to` | 窄化 checked、浮点↔整数 checked、混宽需显式 | **mapped** | `av.parity.1079.mixed-width-explicit`（`to(int)`/`as(i64)`/`as(u64)`）+ `av.parity.1079.overflow-is-not-wrap`（i32 MAX+1 → `av.trap.int-overflow`）；`conversions_are_checked_and_mixed_widths_are_source_rejections`：`i64→int` 越界 trap、混宽 `source.type-mismatch`、超宽度字面量 `source.literal-range`。 |

复跑入口：`cargo t case-av-language-parity`（AV 视图，选中 8 行）与
`cargo test -p ace-av --test conformance plan1079`（7 例类型化/位型 oracle）。

## 6. AVP-02（PLAN-1080）交付：调用绑定、隐式签名与管道

交付提交见归档 Plan 1080（reviewed 34d97223c）。全部能力经唯一普通入口
`compile_language_source`（`auto.source.language.av/v1`）真实执行验定；
冻结窄面（closures/numeric/aggregates/scheduler）描述符与拒绝逐字节不变。

### 6.1 新支持（普通 Auto 面）

- **record 方法**：实例方法（receiver=只读 view，体内 `.field` 直读）、
  `static fn` 与 `Type.static(args)` 调用；MIR 单元名 `m.<Rec>.<name>`
  （PLAN-881 数据路径同规则）。新描述符身份
  `auto.hir.auto-language.draft/1/auto-language-draft`
  （aggregates 词汇 + method 声明 + unit 表达式）。
- **无类型签名补全**（parse→resolve 间补全遍）：省略形参/返回注解按
  调用点单态定形（定点迭代，嵌套/前向调用可达）；尾表达式与 `return`
  定返回；`fn main()` 无值尾 = Unit 入口（worker unit 通道）。
- **原型声明** `fn f(..) R;`：与同签名实体合一；无实体具名拒。
- **管道** `x |> .m(args)`、`x |> .f`、行首 `|>` 显式续行；投影层与点
  后缀/调用同构脱糖（AST/HIR 零新节点；LHS 与每个实参恰求值一次）。
- **紧结合边界（已批审计规范化）**：管道先作用于紧邻后缀链——
  `t |> .get() - 1` = `(t |> .get()) - 1`；legacy 低结合 `(x + y) |> .m()`
  须显式括号。`| >`（带空白）保持位或/比较分词；is or 臂不受表达式
  postfix 面影响。

### 6.2 诊断冻结（stage/code）

| 形态 | stage | code |
|---|---|---|
| 缺实参/重复命名/未知命名/参数未绑定 | resolve | `source.argument`（963 家族原码） |
| 推导冲突（调用点类型分歧）/无约束（无调用点）/体返回分歧 | resolve | `source.inference` |
| 原型无实体 | resolve | `source.unresolved` |

无静默默认类型；Inferred 到达 resolve = fail-closed 具名拒。

### 6.3 本包 case → owner（真实运行，`language/calls/binding-pipeline.case.ad` 22 例）

| case | oracle | 结果 |
|---|---|---|
| `av.parity.1080.pipe-method-and-field` | exit 42（方法管道/多行续行/字段管道） | PASS |
| `av.parity.1080.named-binding` | exit 7（按名绑定） | PASS |
| `av.parity.1080.untyped-old-form` | exit 26 | PASS |
| `av.parity.1080.missing-call-argument` | resolve/`source.argument` | PASS |
| `av.parity.1080.call-source-order` | exit 12 + stdout `2
1
`（源序求值×按名绑定双观察） | PASS |
| `av.parity.1080.pipe-legacy-m10` | stdout `3
18
18
`（m10 源逐字节）+ unit 通道 | PASS |
| legacy 001-006/009/020/021 迁移 9 例 | 原 oracle（14/3/10/26/18/10/42/7/103） | PASS |
| `pipe-neighbor-binop` / `pipe-neighbor-token-separation` | 603 / 3（紧结合与分词正例） | PASS |
| `duplicate/unknown-named-arg`、`inference-conflict/unconstrained`、`prototype-no-body` | §6.2 诊断 | PASS |

（006 `double`→`twice` 为唯一具名审计改名——保留词冲突，语义/oracle 不变。）

### 6.4 AVP-02 承接的 L 行状态

| L 行 | 状态 | 证据 |
|---|---|---|
| L-24（parser-only 子项单列） | **mapped** | 无类型签名以补全遍落地为真实执行（非 parser-only）；compound-ops `+=` 语句面维持 research-hold，owner 修正为 statement-surface 批（本包合同不含，legacy-map 具名） |
| L-25（已批差异双账） | **mapped** | 紧结合边界 + `double` 保留词改名，双账于 §6.1/legacy-map |
| L-47（管道差异双账） | **mapped** | m10 逐字节复跑 + PIPE-NEIGHBOR 正例；`x \|> f(x)` 自由函数注入未新增（路线裁定保持） |

复跑入口：`cargo t case-av-language-parity`（30 例选中）与
`cargo test -p ace-av --test conformance plan1080_calls_binding`（15 例）。


### 6.5 AVP-03 — 命名函数值、闭包与嵌套 fn（PLAN-1081，2026-11-20）

普通 Auto 面（`compile_language_source` / `ac-av/language-source-run-v1`）新增**已验证**能力：

- **命名函数项作为值**：顶层自由函数名在值位 = 零捕获闭包（`CreateClosure{callee:该 fn, captures:[]}`；签名/ABI/目标身份取自其 params/result intern）。支持 let 绑定、`fn(int)int` 形参传递、返回槽、容器槽（`.clone()` 入 List + `.remove()` 出）与同 fn 多点引用；递归/互递归无捕获误绑。`main`（入口）与嵌套 fn 项不可作值（`bind/source.fn-as-value` 家族）；非函数调用保持 `bind/source.not-callable`；签名不符 = `verify/source.type-mismatch`。
- **Auto 面 lambda**：`(a int, b int) => …`（表达式/块体、外层捕获、多次调用、跨返回）；borrow 捕获直调合法、作实参 = `bind/source.closure-borrow-escape`；owned env 退出后 arena live=0。
- **嵌套 fn 项**（d01/d01b 债闭）：语句位 `fn`（CST `nested-fn-stmt`，auto 裁剪）提升为 `__nested_<owner>_<name>` mangled 单元；fn 级提升可见域；**不可捕获外层局部**（Rust 嵌套 fn 语义，`bind/source.nested-fn-no-capture` 具名拒，绝不静默 0）；同名不同 owner 各自 body 独立（含其内 lambda 以 mangled owner 命名）；参数必须注解（`bind/source.inference`），裸返回按体尾推导/无值尾=Unit；lambda 体内的嵌套 fn = `bind/source.unsupported` 具名拒（不收集不静默丢）。
- **下游准入**：MIR 验证器对**空 env** callee 例外两条 lambda 不变量（单创建点/禁直接调用——其理由[每点 env 所有权/需 env 绑定]对空 env 不成立）；`MirType::Closure` 入 collections 家族元素域（Object 先例）；`check_push_value` 认闭包/克隆为新构造；Unit 结果普通调用的 dst 用 Unit 类型（修 D-04 后 void fn 被调从未走过的 `mir.call-result-type` 缺口）。

| case | oracle | 结果 |
|---|---|---|
| `av.parity.1081.named-function-value` / `function-parameter` | exit 42（值位/实参位） | PASS |
| `av.parity.1081.closure-block-local` | exit 42（块体+局部 let+外层捕获） | PASS |
| `av.parity.1081.not-callable` | resolve/`source.not-callable` | PASS |
| `av.parity.1081.basic-ref` / `pass-as-arg` / `call-unchanged` | stdout `10
20` / `42` / `10
14`（27_function_reference 三迁；`double`→`twice` 保留词规范化） | PASS |
| `av.parity.1081.legacy-closure` / `legacy-closure-typed` | exit 3 / 7（007 加注解规范化/008 逐字节） | PASS |
| `av.parity.1081.closure-hof-capture` | stdout `3`（010/013 闭包+捕获面承接；List.map 算法归 stdlib/HOF owner） | PASS |
| `av.parity.1081.nested-fn-void` | stdout `hi from inner
after`（d01 静默失效债闭） | PASS |
| `av.parity.1081.nested-fn-capture` | resolve/`source.nested-fn-no-capture`（d01b error-presence 精确化） | PASS |

冻结边界：嵌套 fn 项 call-only（无值通道）；lambda 体内嵌套 fn 具名拒；非 AV 检查 `#ignore`/NOT_RUN 不充 PASS。复跑入口：`cargo t case-av-language-parity`（42 AV 选中）与 `cargo test -p ace-av --test conformance plan1081_function_values`（21 例）。

## PLAN-1082 增量（2026-11-20，AVP-04 集合/place 面）

**支持域（普通 AV 链真实执行，`ac-av/language-source-run-v1`）**：

- **数组/List 字面量任意 rvalue 位**（auto 面专属；962 aggregates 面 init-only 冻结合同不变）：语句 sink 提升（`ListNew`+push 脱糖，源序保持）；嵌套字面量元素以 `Move` 显式转移其合成新值 temp；空 `[]` 语法收录、resolve 具名拒 `bind/source.array-lit-empty`（无元素类型推导；显式 `List<T>.new()` 承载空表）。
- **place 链赋值/复合赋值**：`ident (.f | [e])* (= | += | -= | *= | /=) e`（CST `place-assign-stmt`，auto 裁剪、置于 path/index-assign 之后仅承混合链/多级索引/复合形，冻结面字节不变）。base/index/RHS 源序各恰一次（合成 temp 提升）；字段链走既有 `RStmt::Assign` 路径写、索引链走 `ListSet`（record List 字段句柄写穿透实测）；复合值 = `ListGet op rhs`（仅 i32 元素，`bind/source.compound-elem-type`）；元素字段尾 `xs[i].f=v` 具名拒 `bind/source.place-elem-field`（LIST_FIELD_SET 源码缝归 records 批）；迭代中结构性写保持 `bind/source.iterate-mutate`。
- **`List.new(e1..en)` 受检兼容糖**：元素 VT 取自 e1（同数组字面量同质校验+同 push admission）；零参具名拒 `bind/source.list-new-empty`；影子局部/record/variant 优先；非任意 varargs 面。
- **HOF 方法面**：`map/filter/reduce/find/any/all/pop` on List<int> → 冻结 `stdlib/ace/asm/list.at` 同名 pub fn 直调（`compile_language_source` 按需文本 graft：`__std_list_` mangle、`pub` 剥离、方法形不动、字节稳定单测；同源同编译器，无第二算法）；非 i32 元素自 PLAN-1120 起不再具名拒，走源内 typed 脱糖车道（见下方 PLAN-1120 增量），旧 `bind/source.hof-elem-domain` 行迁移为回调类型不匹配负控。
- **边界（SEM-02 冻结沿袭）**：越界/负下标/空容器/越界写统一 `run/av.trap.list-index`（先于 buffer 访问）；legacy 负索引 from-end 语义为批准差异分账，绝不冒绿。资源 record（Str/List/Variant 叶，PLAN-1035 F34）经 place 链隐式转移仍拒 `bind/source.move-required`；纯标量 record 按值 Copy 替换。

| case | oracle | 结果 |
|---|---|---|
| `av.parity.1082.bracket-read-write` / `legacy-list-new` / `hof-real-call` / `nested-place` | exit 42 ×4（§6 合同） | PASS |
| `av.parity.1082.place-side-effect` | stdout `i/v/|`（index→rhs 源序恰一次） | PASS |
| `av.parity.1082.nested-index-place` | exit 28（兄弟字段保全） | PASS |
| `av.parity.1082.bounds-*`（endpoints/oob/negative/empty/write） | 39 绿 + `run/av.trap.list-index` ×4 | PASS |
| `av.parity.1082.compound-statement-family` | stdout `2/7/15/5/7/5`（11_compound_ops 六原始 oracle） | PASS |
| `av.parity.1082.hof-family-legacy` / `hof-pipeline-legacy` / `hof-empty-face` | `15/4/0/1`、`60/5/5`、`0/0`（012/015/014 原始 oracle） | PASS |
| `av.parity.1082.list-ops-family` / `list-iteration` / `shims-int-list` / `record-field-place` / `array-literal-shape` / `negative-index-positive-face` / `tuple-literal-destructure` | 迁移族（见 legacy-map 行） | PASS |
| `av.parity.1082.empty-array-rejects` / `list-new-zero-arg-rejects` / `place-elem-field-rejects` | `resolve/source.array-lit-empty` / `source.list-new-empty` / `source.place-elem-field` | PASS |

冻结边界：元素字段写、record 方法、对象字面量深链、Str 元整读分别归 records/AVP-05/AVP-06/AVP-08；hashmap/hashset/StrBldr 归 stdlib 容器批（本包非目标"不另造容器库"）。复跑入口：`cargo t case-av-language-parity`（67 AV 选中）与 `cargo test -p ace-av --test conformance plan1082_collections_places`（17 例）。

## PLAN-1083 增量（2026-11-20，AVP-05 名义类型/枚举/模式面）

**支持域（普通 AV 链真实执行，`ac-av/language-source-run-v1`）**：

- **枚举载荷形式**：命名多字段 `Arm { f T, .. }`（合成 record 载荷 `__enum_<E>_<A>`；命名实参序/乱序构造、位置构造、多绑定解构按声明序、单字段绑定、无绑定臂）；多元组 `Arm(T1, T2, ..)`（`TypeRef::Tuple` 折叠，位置构造/多绑定，965 元组机制复用）；标量 `= N` 判别（resolve 期 i32 常量折叠，`E.A` 即 int，int scrutinee 上 qualified 臂按值匹配；auto 面臂逗号分隔吸收）。载荷元数不匹配具名拒 `bind/source.pattern-bindings`；`= N` 与载荷臂混用拒 `bind/source.enum-mixed-arms`。
- **模式组合**：`_` 通配（投影 Else，须后置）；or 臂 `A | B -> body`（投影展开共享 rhs，绑定元数须一致）；guard 臂 `pat if cond -> body`（模式命中后求值、绑定可见、false 继续下一臂、未选臂 guard 不求值；同 disc 重复仅当前序臂皆 guard，纯重复仍 `bind/source.pattern-duplicate`）；return 臂体；非穷尽值位 `bind/source.pattern-nonexhaustive`；auto 面语句 `is` fall-through（无匹配继续后续语句；值位与冻结面保穷尽合同，filler 按 has_following 门控）；fn 尾 `is` → `Return(IsExpr)`、script 尾 `is` → 尾值（infer::complete 规范化；return 臂尾保持语句形）。
- **enum 实例方法**：enum 体 fn 成员（auto 面），隐式 variant 接收者（`is self` 分派）、变异接收者按值传参（无 view borrow——复合值 864 规则）、`m.<Enum>.<name>` 单元与 `t_variant_<E>` 类型原子。
- **Option/Result 面**：`?T`/`!T`/`!?int` 注解、bare 构造子、is 分派、`.?(default)`、`.or(default)`（guard-test 投影/默认）；`!T` 且全部 Err 载荷命名同一枚举 → 显式 `Result<T, E>` 改写（Str 错误保持糖；尾 Ok 归一 `return Ok(v)`）。
- **隐式 self 字段写**：`.f (op)= v` 语句（auto 面 self-place 规则，投影为 base=self 的 place 链）；数组字面量尾逗号；字段/臂逗号吸收。

| case | oracle | 结果 |
|---|---|---|
| `av.parity.1083.named-enum-payload` / `nested-result-option` / `tuple-across-call` | exit 42（§6 合同 1/2/3 逐字） | PASS |
| `av.parity.1083.duplicate-pattern`（§6 合同 4） | `bind/source.pattern-duplicate` | PASS |
| `av.parity.1083.multi-field-positional` / `single-named-payload` / `bare-scalar-arm` | `10/20`、`5`、`point`（016 三腿） | PASS |
| `av.parity.1083.named-construct-ordered` / `named-construct-reordered` / `positional-tuple-payload` | `404`、`503`、`5000`（017 逐字） | PASS |
| `av.parity.1083.no-binding-arm` / `multi-field-arith-destructuring` / `float-payload` / `two-elem-tuple-payload` | `rgb`、`383`、`3.14`、`10/20`（018/019） | PASS |
| `av.parity.1083.scalar-enum-value` / `int-literal-arms` | exit 1（10.002）、exit 40（10.011 int 码规范化 red→10/blue→30/unknown→0） | PASS |
| `av.parity.1083.enum-static-style-method` / `enum-instance-method` / `payload-variant-method` / `wildcard-arm` | `called-ok`、`yes/no`×2、`red/other/other`（28.001-004） | PASS |
| `av.parity.1083.option-family` / `option-none-match` / `result-family` / `unwrap-or-default` / `option-or` / `result-fn-str-error` | 42、-1、-2、141、141、5（16.001-019） | PASS |
| `av.parity.1083.err-enum-payload` / `multi-error-family` | `unexpected end`、`404/timeout/200`（16.021/022） | PASS |
| `av.parity.1083.nested-is-shadowing` / `guard-arm` / `guard-not-evaluated` / `or-arm` / `nonexhaustive-expr-rejects` / `statement-fallthrough` / `exhaustiveness-else` / `fn-tail-is-value` / `script-tail-is-value` | PATTERN-BIND 五面 + 尾值族（O54/B34/B36/O53/B37/B38/else/30/42） | PASS |
| `av.parity.1083.record-method-reads` / `positional-ctor-family` / `list-elem-method` / `method-value-family` / `str-field-record` / `record-array-fields` / `nested-type-instance` | `chain=6`、`build=9/origin=0`、`total=6/idx1=2`、`7/8/12`、`title ok/id ok/from fn ok`、`array field ok/second id ok`、exit 10（m03-m06/034-036/008 读面） | PASS |

后续交付：record 方法字段写（`&mut self` 接收者可变性）与经元素 Str 字段读由 AVP-08 验收，含 m01/m08/m09 写腿与 10.036 title 腿。其余冻结边界：泛型 enum 归 AVP-10（PLAN-1088；28.005 hold）；`use auto.result` stdlib Result 方法归 modules/stdlib owner（28.006 hold）；fstr record display 归文本面（10.013 display oracle 留 map）；未类型尾调用未类型函数的推导缺口为 1080 既有（wildcard helper 内联规避）。复跑入口：`cargo t case-av-language-parity`（111 AV 绿/41 NOT_RUN）与 `cargo test -p ace-av --test conformance plan1083_canonical_verify`（35 例孪生）。

## PLAN-1084 增量（AVP-06，动态对象与 null）

普通 Auto 源的对象支持异构值。点访问和下标访问共享同一键查找和就地写入路径。键的域是文本、`int` 与 `bool`，标签参与身份。重复字面键在绑定时诊断，更新已有键保原位置，新增键追加。缺键、错误的值标签及环分别产生 `av.dyn.object-missing`、`av.trap.dyn-type` 与 `av.dyn.object-cycle`。

异构列表使用逐元素 Dyn 标签。对象与列表可嵌套并共享引用，跨别名的写入可见。执行出口记录 int、unit、文本或对象通道，父端独立比较结果和两股输出。编译和运行诊断绑定虚拟入口文件与原字节区间。`.as` 的 nil 类型错误进入语言 catch，资源耗尽和取消保各自的终态。

`language/dynamic/objects-null.case.ad` 收录本包 42 个实际 AV Case，含 11 个旧对象主用例及 1082 移交的 8 个混合集合用例。对应的老检查、预期和 owner 见 `legacy-map.tsv`。旧脚本中的 Python/宿主组合由 PLAN-1094 承接，HashMap/HashSet API 行保现有标准库容器职责。验收来源见 [PLAN-1084](../../plans/archive/1084-av-parity-dynamic-objects-and-null.md)。

## PLAN-1085 增量（AVP-07，模块与全局状态）

普通 `ac-av/language-source-run-v1` 接收同一 AutoDown Case 的工程闭包。入口和依赖采用 `module.at` 或 `package/module.at` 虚拟路径，同目录裸文件隶属 `auto` 包。单文件可携带 `ac.source.modules.packages/v1` 选择包语义。公共语法和绑定支持 `use`、导入别名、公开声明、公开重导出和同名模块的包隔离。包图拒绝缺失模块、私有导入与循环，诊断保留原文件和字节跨度。工程身份覆盖全部虚拟路径和源码。包入口可使用类型化 `fn main` 或入口模块顶层脚本，依赖模块脚本和同一工程内竞争的两个入口具名拒绝。

程序代持有各自的全局单元，覆盖 `Str`、List、record、variant、带 owned 捕获的闭包和异构对象。整格赋值替换旧值。显式移动清空源格，未回填前读取给 `av.trap.global-moved`。依赖初始化按受检顺序执行，钻石叶只运行一次。初始化完成前的错误、预算耗尽或取消保持未发布，释放已写入的 owned 值。重试、关闭、新会话与从源码收据冷恢复都使用各自代的单元。配置文档保持 `IF-982-CONFIG-ACE` 的已有 producer。

`language/modules/imports-globals.case.ad` 收录本包 AV 观察及九条原检查，分母、原预期和 owner 见 `language/migration/legacy-map.tsv`。Parser/emit 的 bootstrap 组合仍由 PLAN-1095 承接。交付接口为 `IF-AV-PARITY-MODULE`，冻结工具身份与结果见 [PLAN-1085](../../plans/archive/1085-av-parity-modules-and-global-state.md)。

## PLAN-1086 增量（AVP-08，借用、移动与存储）

普通 Auto AV 可给拥有文本建立有界 `.view`，可将计算出的 `.slice` 绑定到局部量，也可把 `Str` 借给 `str` 形参。返回视图可指向静态文本或保留调用者的有效根，当前帧根逃逸由 `mir.borrow-escape` 拒绝。活跃借用与冲突的写回或移动由 `mir.mut-conflict` 拒绝。移后读在执行前以 `mir.use-after-move` 拒绝并报告源位置。记录方法和参数的可变传递写回原 owner，文本 `.mut` 的 `append` 在原绑定写回。

旧拥有型记录字段、列表元素及 Option 载荷的 `str` 拼写归一为 `Str`。普通参数和结果位置的单独 `str` 保留借用。直接发布的脚本列表结果可复制其中的借用文本。声明为显式 `List<Str>` 的拥有元素传递保留 `.move` 或新值要求。旧式无类型只读 `let` 从含旧 `str` 字段的列表记录获取独立副本。记录列表 `.pop()` 移出末项，空列表以 `av.trap.list-index` 拒绝。

`language/ownership/borrows-moves.case.ad` 是本包共享作者。文本 `.mut` 原 VM 预期 `hello`，AV 写回预期 `hello world`。两个 SSE 原源码分别打印 `events: 1` 与原各自完成行。原 aggregates 探针 `str_slice_len_bytes` 在相同源码上检查第一标量 ASCII `A` 的字节长度为 1。普通 AV 的文本搜索与修剪通过 `av.text.scan/1` 受检执行。原样 2,000 轮字符串池用例在默认预算下返回 50000。

交付接口为 `IF-AV-PARITY-OWNERSHIP`。原源码、检查身份、分析范围和观察来源见 [PLAN-1086](../../plans/archive/1086-av-parity-borrows-moves-and-storage.md)。

## PLAN-1087 增量（AVP-09，错误与控制流）

普通 Auto AV 的直接调用、闭包调用和模块调用共享语言错误的展开路径。每个受保护的调用有独立的成功与错误出口。错误出口提供拥有型消息，不读取未生成的调用结果。展开释放被退出的调用帧，保留接收处理器的调用帧与局部量。跨作用域的 return、break、continue 和语言错误经过各自的 finally。finally 自身失败时，新的语言错误覆盖待完成的退出。无参 `rethrow()` 保留原消息、类别和源码位置。

列表读下标失败可沿受保护调用传播 `IndexError: index N out of range`。普通 `.as` 已接通的 null TypeError 可穿过调用传播。显式以受准入 Python provider 启动的普通入口，可捕获受审 Python 异常类别与消息。宿主授权、I/O、资源拒绝和取消沿原执行边界传递。旧 `.as` 与 Python 的组合协议由 PLAN-1094 承接。

值位 `is` 要求穷尽，非穷尽语句 `is` 可继续。Option `.?` 在 Option 返回边界传播 None，默认值和 `??` 均按需计算右侧。在列表构造 `[a, if cond { b }]` 中，条件按源序求值，条件成立才求值并加入对应元素。范围仅准入迭代位置。旧例 04.006、16.007、16.008、16.010 和 05.003 保留各自 VM 预期和同源 AV 诊断。生成器跨挂起与关闭的生命周期由 PLAN-1089 承接。

公共作者为 `language/errors/catch-finally.case.ad`。交付接口为 `IF-AV-PARITY-ERROR`。路径、输入、工具与 provider 证据见 [PLAN-1087](../../plans/archive/1087-av-parity-errors-and-unwinding.md)。

## PLAN-1089 增量（AVP-11，生成器与迭代生命周期）

普通 AV 源码面支持 [生成器同源语料](../../../crates/auto-lang/test/cases/language/generators/yield-close.case.ad) 覆盖的 ~Iter<T>。普通函数可间接返回流，工程模块可生产含枚举和泛型记录载荷的流，迭代器保存挂起后要用的局部。连续 next 按源序产生项，耗尽返回 None。拥有型 Str、List 和枚举载荷经受检转移。旧 ~Iter<str> 以拥有型文本承载项，int.to_str() 复用普通显示操作。四份旧生成器的实际 stdout 为 6、10,20,30,、abc、6（均有换行）。

带借用输入的生成器由直接 for-in 消费。首次取项时，挂起前的公用前段经普通受检形参读取或写回调用者的 owner。状态只保存前段留下的标量和拥有值。仍在挂起后使用借用时报告 bind/source.generator-borrow-param。返回或存放带借用的流报告 bind/source.generator-borrow-escape。

提前 break、正常耗尽和显式 close 会终结流。未启动即 close 不执行生成器体；悬挂 finally 从内到外执行，每个作用域一次。关闭释放后的终态在捕获关闭错误后仍可见，后续 close 无副作用、next 为 None。宿主取消与程序错误的分类沿用普通 AV 合同。父端分别观测取消、两项后的资源释放和关闭错误恢复。接口和复跑见 [PLAN-1089](../../plans/archive/1089-av-parity-generators-and-stream-lifetimes.md)。

## PLAN-1088 增量（AVP-10，泛型、spec 与方法决议）

`IF-AV-PARITY-GENERIC` 走 `ac-av/language-source-run-v1` 的 `d_main` 入口。普通函数和方法从实际参数、显式类型与 const 实参、已知返回上下文产生实例需求。相同声明及替换共用实例；包的声明身份携带包名、锁定版本和模块。函数体更新保持逻辑实例身份，签名更新后按新签名重新检查调用，恢复校验产物所属闭包。

普通 AV 通过实际参数或预期类型求解依赖的记录和枚举类型。类型别名在类型实参位置展开，记录保名义身份。枚举保构造、分派与方法的具体身份。普通包支持跨模块的泛型函数、记录和方法实例。泛型参数的 spec 约束使用显式遵从及其传递闭包；具体记录支持关联类型和默认体。普通单模块的完整具体 spec 参数通过已核验记录见证分派。显式实现优先于默认体，同层默认体歧义具名拒绝，导出记录上的扩展方法随模块可见性绑定。

`List<T>.to_array()` 复制元素和资源；`filter` 按调用时的元素顺序调用谓词，结果独立拥有保留元素。普通块闭包可声明返回类型。旧 spec 工具用例所需的字符串操作走受检 AV，Unicode 大写转换通过 `av.text.case/1` 准入，并计入执行燃料和槽预算。字节码标记为 `STR_UPPER = 0x53`。

公共输入、检查和原始输出见 [泛型作者目录](../../../crates/auto-lang/test/cases/language/generics) 与 [PLAN-1088](../../plans/archive/1088-av-parity-generics-specs-and-methods.md)。旧泛型枚举 28.005 的 VM 检查保留数字显示；普通 AV 按 PLAN-474 的文本显示判断 `true` 和 `false`。PLAN-1092 持有 `#[with]` 源码拼写并复用本接口的约束见证。

## PLAN-1091 增量（AVP-13，编译期执行与宏）

普通 `ac-av/language-source-run-v1` 在单文件与包入口复用 Auto CST、绑定和受检 HIR。装配先按目标事实选择 `#in/#if/#for/#is`，再服务选中程序中的 `#{expr}`。编译期调用经 source-cte、MIR/ABC 验证和真实 AV，结果结构替换回原位置。主要值域为 `int/i32` 和 `bool`，含具体泛型实例、跨模块调用和只读包常量。可变运行时状态、外部作用与非标量冻结值有具名拒绝。

普通宏调用按函数绑定并丢弃返回值。`#debug/#info/#warn` 经普通控制台写 stdout，`#error` 写 stderr，各带级别前缀和原换行语义。未知名称在绑定期拒绝。装配选择、普通 CT 与运行期日志可出现在同一工程，适用同一套源码跨度和诊断。

`IF-AV-PARITY-CT` 绑定 Auto 源、AV `av.closures/1`、worker `avworker/1`、MIR lowering 7、目标/cap/provider 与实际检查结果。Windows AV 结果、精确诊断、覆盖边界与旧义务映射见 [PLAN-1091](../../plans/archive/1091-av-parity-comptime-and-macros.md) 及其 [canonical 语料](../../../crates/auto-lang/test/cases/language/comptime/evaluation-macros.case.ad)。

## PLAN-1090 增量（AVP-12，任务、异步块与 Actor）

普通单文件 Auto 入口 `ac-av/language-source-run-v1` 支持 `~{}`、返回注解 `~T`、前缀和后缀 `await` / `go`。延迟体在等待或启动时执行，并保存拥有型捕获。Future 可以经函数返回并存入 List 与记录。受检 `TaskRef<T>` 可以跨普通函数、handler、参数、返回值和记录字段传递；`T` 约束邮箱消息类型。

Actor 支持 typed、变体、字符串、guard 和 else handler，以及状态初始化、start、stop、FIFO 与有界背压。L-29 由延迟体、值传播与捕获隔离覆盖，L-31 由 ask/reply、消费、未回复与迟到回包覆盖，L-43 由 handler、生命周期、跨函数与存储覆盖。同一普通源码经共享 CST、Checked HIR、MIR、固定 ABC 与 AV 调度器执行。

`IF-AV-PARITY-SCHEDULER` 绑定 `av.closures/1`、`avworker/1`、MIR lowering 8、当前 host/provider 授权和逐检查结果。41 条普通 AV 源码检查与旧义务身份见 [任务和异步语料](../../../crates/auto-lang/test/cases/language/async/tasks-actors.case.ad) 和 [PLAN-1090](../../plans/archive/1090-av-parity-tasks-async-and-actors.md)。

## PLAN-1092 增量（AVP-14，标注与元数据）

普通 AV 的共享 CST 接受 `#[with(T)]` 和 `#[with(T as Spec)]`。绑定和实例化沿用显式泛型和名义 spec 见证。重复参数、重复属性、未知属性和错误位置有具名绑定诊断。普通 AV 选择 `#[vm]` 的实现，排除仅写 `#[c]` 或 `#[rs]` 的声明。包 `pub` 可见性由现有模块绑定器判定。

`#[test]` 进入 971 的声明发现。显式普通 AV 项目发现零参、无类型参数、返回 unit 或 int 的标记函数，执行结果按原测试报告判定。文件级 `#[rust]` 优先于 `#[script]`，无显式 pragma 时 `.as` 选择脚本模式。文件模式与运行能力由各自前门接入，Python 动态脚本由 PLAN-1094 承接。

`#[single]` 任务在一次运行中按任务类型复用一个懒建的 64 槽邮箱。静态 ask/send/stop 进入现有 AV 调度器。`#[async]` 标记的实际效果由已有 `~T` 返回面确定。普通语言 worker 继续使用 `avworker/1`、`av.closures/1`，普通 MIR lowering 缓存身份为 9。

`#[api]` 的 method/path 与目标可见性经共享 CST 交给 auto-web 的 AV 服务。服务按受检 HIR 区分借用 `str` 和文件通道，并在驻留 AV 程序上执行。接口 `IF-AV-PARITY-METADATA` 由 `compile_language_source_with_metadata` 提供受检模块及同一源单元的 `SourceMetadata`。CST 也提供 `#[export]` 的归一 ABI 名和 Rust 透传属性的原参数跨度。已有外库链接与授权归 967 和其后继，Rust 属性交对应发射器。AV 阶段验收以及唯一公共作者见 [PLAN-1092](../../plans/archive/1092-av-parity-attributes-and-metadata.md)。

## PLAN-1093 增量（AVP-15，受检 FFI 语言接面与回调）

普通 `ac-av/language-source-run-v1` 现在可通过**预构建 provider 闭包**绑定真实外来
provider：worker 请求携带声明性 `foreign: {kind, provider}`，从
`ACE_AV_PROVIDER_ROOT/<provider>.descriptor.json` 读取冻结源、DLL SHA-256、rustc
身份（Rust）或 manifest+header 摘要（C），纯派生物理签名后经
`compile_language_source_with_foreign`、`verify_abc_foreign` 与带 stub 表的
`execute_console_foreign` 真实派发。日测永不编译——providers 由显式 prepare lane
构建一次（`prepare_foreign` 示例：`build_f0`/`build_source`/`open_c_provider`+`build_c_dll`）。

- `use.rs <provider>`：Rust 注解源（`f0_text/1`）经 `auto_bindgen::rust_import` 派生
  manifest/shim；prepared cdylib 加载后按 967 冻结 callable 形状映射。`text_byte_len`
  按 **UTF-8 字节**计长（`"A中😀\0"`=9、空串=0），绝不按 C strlen 截断。
- `use.c <provider>`：C 注解头（`parity1093_c`）经 967 C 管线派生 manifest；实现单元
  可显式调用真实外库（`<string.h>` strlen，SDK include 闭包编译）。AV 侧传借用视图的
  (data,len) 对，C 侧构造有界 C 字符串（CStr 桥）。
- `use.c <header>`：C 头角括号拼写（`<string.h>`）已入共享 grammar（`provider-choice`
  + `c-header-item` 规则）；投影为头名，仍须匹配已 admitted provider（否则
  `source.use-rs.provider`）。
- 授权：外来声明要求显式 `foreign.scalar` cap（缺失 → `source.foreign-capability`
  bind 拒绝）；provider 身份由父侧 `REGISTERED_FOREIGN_PROVIDERS` 表核验。
- panic 边界：provider panic 被 shim 就地捕获并置导出标志，AV 调度器转为具名
  `host.infra`（`foreign.panic: … contained at the boundary`）——不跨 C ABI unwind。

`IF-AV-PARITY-FOREIGN` 绑定普通 Auto 源、`avworker/1`、`av.closures/1`、MIR lowering 9、
已注册 provider（f0_text/parity1093_c）与逐检查真实结果。canonical 语料与迁移行见
[FFI 作者文件](../../../crates/auto-lang/test/cases/language/ffi/imports-callbacks.case.ad)、
[PLAN-1093](../../plans/1093-av-parity-foreign-language-surface.md)。

## PLAN-1094 增量（AVP-16，脚本模式与 Python 对象协议）

普通 `ac-av/language-source-run-v1` 现在对**带 `foreign: {kind, provider: "py"}` 的
`auto-script`/.as 源**经 py-bridge provider 闭包（`prepare-av-providers` 构建，
`ACE_AV_PROVIDER_ROOT` 定位）真实执行 Python：`use.py <module>: <member>, …`
（多成员按源序形）、动态参数/属性/索引/调用、py 对象 `.len()`→`__len__` 协议分派
（纯 Auto str/List `.len()` 保持原生计数）、Unicode/NUL 文本往返。链路：
`compile_language_source_with_foreign`（`use.py` 前门注入 helper＋hoisted 成员
import）→ `verify_foreign_closures` → ABC → `execute_console_foreign`（带
`bind_python_error_observers`）。

- **`with <expr> as <name> { body }`**：Python 上下文管理器语句面（grammar
  `with-stmt`/`with-resource`/`with-tail`＋CST＋resolve＋AV runtime）。manager
  求值一次；`__enter__` 绑定 `name`；body 在保护体循环内执行；normal / return /
  break / continue / 语言错误每条出口运行 `__exit__` **恰一次**（normal 传 None
  参数，语言错误传错误文本）；truthy `__exit__` 结果按 Python 协议**抑制**错误，
  控制流继续。`__enter__` 失败不运行 `__exit__`（Python 语义）。
- **Python 异常 → 可捕获语言错误**：失败的 py 操作按 provider class 映射为
  `LanguageErrorKind::Python(class)`（1..=10 语义类别；class 5=`value` 等），
  worker 渲染 `python.{class}` 分类与 `PyException {msg}` 消息；未捕获时以
  `lang.error` 运行终态结束（带失败操作的原始 `source_span`），绝不静默
  foreign-status trap；捕获时 handler 恰一次接收。宿主撤权/resource/cancel
  分类不变。
- **文本通道**：`py_stage_str`/`py_str_word`/`py_error_stage` 升级为
  **codepoint-wise UTF-32-LE word**（每 word 一个 Unicode 标量，ASCII 输入在
  数值上等价），支持中文与内嵌 NUL 逐字节往返；1026 冻结测试 word oracle 随
  通道升级同步（通道/语法扩后同步冻结契约测试的既有惯例，非改语义收据）。
- **缺解释器 = NOT_RUN**：`prepare-av-providers` 无法构建 py 适配器时写
  `py-unavailable` 标记，父侧对带 `foreign: py` 的检查显式 `Status::NotRun`
  （"never PASS"）；无 `use.py` 的纯 Auto 源不加载 CPython（零 py 依赖）。

`IF-AV-PARITY-SCRIPT` 绑定普通源 auto-script/.as、py-bridge provider（kind rust/
provider py）、`avworker/1.0`、`av.closures/1`、MIR lowering 9 与 12 条 canonical
case（ID 族 `av.parity.1094.*`）的真实逐 case 结果。canonical 语料与迁移行见
[脚本/Python 作者文件](../../../crates/auto-lang/test/cases/language/script/python-context.case.ad)、
[PLAN-1094](../../plans/1094-av-parity-script-and-python-protocols.md)。


## PLAN-1095 增量（AVP-17，普通工程/会话组合与 AGE node CPU 面）

普通 `ac-av/language-source-run-v1` 现在把**同包、同 session 的普通语言组合**作为
一条生产路径：多文件工程 + `use` 导入 + 本地自由 fn 作值（工程/packages 绑定路径
与单源一致——`packages/binding.rs` Auto-language 通道，PLAN-1081 T-02 的
zero-capture CreateClosure 通道）、匿名 record 字面量与字段投影、try/finally、
task spawn/ask/await/stop、CT 表达式与运行时列表/filter 在同源共存。非 AV 检查
（AC/A2R/legacy）仍按 AV-ONLY 阶段策略 `#ignore`/未选，不 prepare、不 dispatch、
不计 PASS。

- **AGE node 声明进入普通 Auto 面（CPU-only）**：`auto` profile（grammar
  `age-node-item` 及其 head/body/text 子规则与四个 contextual name）接纳 H0
  node 形状 `node <Name> { view { text <expr> {} } }`；文本表达式在 auto 面用
  其自有 aggregates-ops 表（age 面仍 wide-ops）。CST 投影在 auto 面产出
  `Item::Node`（名字 + span，体不重投影）。共享语法身份：age-h0 /
  age-component / age-package 因共享门控表增 auto 而逐字节漂移，2026-11-22
  按 PLAN-1095 具名重冻结。
- **worker node 观察**：同一源经 979 producer（`compile_age_h0` +
  `observe_age_h0` 的共享执行链）产出 node 的 view 文本真实值，写入响应
  `step.nodes`（name / head_role / view.role / texts[].value）；无 node 声明时
  字段缺席。父侧 `nodes` oracle（`nodes: { Name: [texts] }`）逐名/逐文本断言，
  变异必红。不启动 renderer；OnEvents/Scene/NavCall 旧 parser-only 通道按 L
  表分账，不属本批。
- **PROJECT-RESTORE**：同一 store/session 内坏编辑（CT trap）具名失败后，
  恢复函数体走缓存命中且与冷编值/hash 一致；第二 store 各自冷执行（
  ct_execute=1、ct_hits=0），跨 session 不串 state/cache。

canonical 语料：`language/composition/ordinary-projects.case.ad`（module-closure-
generic / task-language-composition / cte-and-ordinary-library）、
`language/domain/node-values.case.ad`（node-ordinary-coexist）。`IF-AV-PARITY-PROJECT`
绑定普通源/工程、node CPU 观察（979 producer）、`avworker/1.0`、`av.closures/1`、
MIR lowering 9 与逐 case 真实结果；129 份旧 VM 文档义务行（legacy-map.tsv
owner=PLAN-1095）映射到对应 canonical 面，原 oracle/条件/SHA16 保留。


## PLAN-1096 增量（AVP-18，AV 语言齐平完整验收）

`ac-av/language-source-run-v1` 作为本路线 **AV 语言出口的最终验收入口**：全路线
required 语言义务的并集以唯一 canonical 语料（`language.suite.ad` AV 视图，
case-av-language-parity 固定组）真实执行并核逐行身份。本包新增三条收口 case：
`composition/control-effects.case.ad`（combined-acceptance：fn 值 + 列表 +
匿名 record 文本/整数字段 + 布尔合取 + try/finally 同源组合 = 42）、
`functions/source-change.case.ad`（source-mutation-sensitive：同一 fn 值机制下
函数体 x*3 必须真得 63——oracle 源绑定，改错必红）、
`errors/uncaught.case.ad`（uncaught-not-success：未捕获 `fail` 以 run 阶段
`lang.error` 终态结束且消息保留，绝不静默成功）。

- **FINAL-COVERAGE/FINAL-CONTROLS**：逐 required 行的源码/工具/ABI/provider/
  平台/entry/profile/oracle 身份与当前实际结果齐；删行/改 oracle/错源/零选中
  等控制必须可区分失败。非 AV 目标检查（AC/A2R/legacy）保 `#ignore`/未选与
  后续 owner（1068/985），不计 PASS、不阻本包 AV 证书。legacy dispatch=0，
  旧 VM 不因本包启动。
- 交付接口 `IF-AV-LANGUAGE-PARITY/1` 绑定普通源/工程、avworker/1.0、
  av.closures/1、MIR lowering 9 与逐 case 真实结果；1068/985 消费本 AV 来源
  补其它端，不新增 M2-A 已签出口硬门。

## PLAN-1120 增量（2026-11-23，AV 泛型 List 高阶方法补齐）

**支持域（普通 AV 链真实执行，`ac-av/language-source-run-v1`）**：

- **非整数元素 HOF 方法面**：`map/reduce/find/any/all` 在 `List<Str>`/`List<StrOwned>`/`List<record>` 上与先行落地的 `filter` 同车道——源解析器内联 typed 脱糖（snapshot 克隆隔离、源序访问、`resolve_closure_call` 冻结闭包），`List<int>` 保持 972 冻结 graft 不变（`hof_int_list_still_runs_the_frozen_stdlib_graft` 钉住 42 近邻）。
- **回调与持有规则**：回调单参 = 元素类型（`List<StrOwned>` 另接受 `fn(Str)` 借用视图）；记录元素经借用视图车道，其余 owned 元素每访问克隆一次（同 filter 谓词参数契约）；`map` 的 `U` 取自回调结果类型（`map(Note→int)`），push 走 `check_push_value`；`reduce` 累加类型取自 init（`reduce(int, Note)`），回调签名须为 `(acc, elem) -> acc`，不符在绑定时具名拒 `bind/source.type-mismatch`；`find` 绑定 `?T` Option 合同（未命中 None；Some 载荷 owned 元素 Move 入槽），旧 `found.id` 直取字段写法 = `bind/source.type-mismatch`，迁移行须显式 `is` 处理。
- **求值语义**：`map/filter/reduce` 访问全部元素（源序）；`any/all/find` 短路（首个决定元素后回调不再执行）；空表零回调、`reduce` 空表保持初值、`find` 未命中 None、`any=false`/`all=true`。所有权：快照隔离使原表在调用后仍可用；回调 `fail` 经 try/catch 捕获后临时结果释放、输入可续用（分配/释放经 `assert_released` 与 fail-Nth 故障重放钉住）。
- **命名拒绝迁移**：旧非 i32 元素 `bind/source.hof-elem-domain` 行退役，代之以回调类型不匹配 `bind/source.type-mismatch`（plan1082 `hof_non_int_callback_type_rejects`）；嵌套 List 元素的 `find` 仍具名拒 `source.hof-elem-domain`。

| case | oracle | 结果 |
|---|---|---|
| `av.parity.1120.generic-list-hof`（`experimental/ace/crates/av/tests/cases/plan1120_generic_list_hof.rs` 9 例） | V-01 独立预期 stdout `2 2 world true true 3 1 3 6`（legacy VM 布尔显示 `1` 为具名差异）；V-02 空表/未命中/短路计数/绑定拒绝；V-03 全所有者释放、分配故障重放、回调失败 try/catch 后输入续用；int 近邻 42 | PASS |

复跑入口：`cargo test -p ace-av --test conformance plan1120_generic_list_hof`（9 例）+ plan1082/1086/1088 近邻族。legacy 映射：`legacy.25_method_u64.012_list_hof_value` 行归本计划（`legacy-map.tsv`，canonical `av.parity.1120.generic-list-hof`）。
