# AAVM v2 "类 C 风格 → 一对一 Rust 对译"升级:前提条件调研

> 2026-08-25 | 基于 master `c3ee519d0` + 当日构建(`auto-vm.exe` / `auto.exe trans`),
> 配套实证 probe 18 件(见 §2;`tmp/p-idiom/` 为 gitignore 临时件,立项时须搬入
> `test/vm/` 常驻)。本文是"先过前提条件 → 立项修宿主 → 再立 aavm 风格升级计划"的第一步输出。

## 0. 定位与结论摘要

长期路线(已确认):aavm 长期锚定"与 Rust 参考实现函数级/块级一对一";更高阶的
"脱离 Rust 参考的纯 Auto 风格自研实现"远期再做。因此**从最简语法版本升回一对一
版本是必然动作**,其前置 = ①宿主(VM + 主 a2r)把当年绕开的缺陷修掉/验证掉;
②AA2R(auto/lib/a2r.at)先具备新语法的**发射**能力(自举塔约束);③然后才动 lib。

**一句话结论**:2026-08-24 移植时登记的三类主障碍,今天实证下来**大半已经消失或收窄**——
is-on-string / is-on-char / 卫语句臂 / 枚举载荷跨函数传参返回二次使用 **VM 全部实测通过**;
仍真实存在的硬阻断只剩 4 个(2 个 VM bug + 2 个 a2r 发射缺陷),外加 AA2R 自身的
is-match / enum 载荷发射缺口(工作项而非缺陷)。**升级窗口已经打开,工程量主要在
AA2R 扩展与 lib 改写本身,宿主修复量很小。**

## 1. 当年为什么降级(divergences.md 登记回顾)

| DIVERGE | 当时的理由(2026-08-24 登记) | 今天的状态(§2 实证) |
|---|---|---|
| D11b | "VM 对 match-on-string/enum 的支持未验证,v1 if 链是已证路径" | **is-on-string/char 实测通过**;is-on-enum 单臂通过;**or-臂发现新 bug(仅枚举路径,见 H1)** |
| D34 | "VM 枚举载荷值跨函数传参丢标签(probe20/22/23 实证)" | **直接传参/返回/绑定二次使用/循环构造/结构体字段/List 存取全通过**(433 系列 RC 修复的受益面);**唯一崩溃形态收窄为"运行期计算值内联在枚举构造参数位"(见 H3)**;probe20-23 复现件已失,本次已重建等价件 |
| D17 | "循环体内 continue 穿透执行后续语句"(挂 242,S1-432) | **bug 仍在,精确复现**;range-for 已修,while/Cond-for 未修(见 H2) |
| D36-① | "原生调用不入枚举/结构体构造参数——载荷丢失,先提升局部" | **仍在**(即 H3;且新增:结构体字面量字段位同样触发,不止原生调用) |
| KNOWN-DEBT 113 | a2r is-绑定变量无类型跟踪(后续 .get/字段解析失效) | **简单形态不再复现**(t.len()/List.get(0) 下游使用转译正确、可编译);保留观察 |
| AA2R Missing | is-match/闭包/spec/ext/impl/use/dep/泛型声明未发射 | **仍然缺失**(f-string 实际已支持,文件头清单滞后);扩展改动点见 §5 |

## 2. 实证矩阵(2026-08-25,probe 18 件)

### 2.1 VM 侧(`target/debug/auto-vm.exe <file>.at`)

| # | 形态 | 结果 | probe |
|---|---|---|---|
| V1 | is-on-string:字面量臂 / 多模式 `\|` / else / 运行期拼接目标串 | ✅ 全过 | p01 |
| V2 | is-on-char(含 or-臂) | ✅ 全过 | p11 |
| V3 | is 卫语句独立臂 `if cond -> body`(与模式臂混用) | ✅ 全过 | p08 |
| V4 | is-on-scalar-enum 单臂逐个(kind_name 形态) | ✅ 全过 | p02/p02b `single` |
| V5 | **is-on-scalar-enum or-臂 `A \| B ->`** | ❌ **仅第一个模式可命中,后续模式永远落空**(p02b `multi`:Uint→eof、False→eof;字符串/字符 or-臂正常) | p02b |
| V6 | 枚举载荷(int/str)字面量构造、跨函数传参/返回、is 解构绑定 | ✅ | p03/p05 |
| V7 | 枚举载荷**运行期拼接串**构造 + 跨函数 + 绑定二次使用 + 比较后再读 | ✅ 全过 | p04 |
| V8 | 循环内反复构造载荷枚举 + `_` 通配绑定 + 多跳传参 | ✅ 全过 | p10 |
| V9 | 枚举值进出 List 原生容器(单元素/双元素/混合载荷/运行期串,提升局部写法) | ✅ 全过 | p09b/c/d/e/f/g |
| V10 | 枚举局部重赋值 / `==` 相等比较 / 结构体字段(字面量载荷) | ✅ 全过 | p13a/c |
| V11 | **运行期计算值内联在枚举构造参数位**(外层为原生调用实参或结构体字面量字段) | ❌ **`[RC canary] string tombstone access` 崩溃**(engine.rs:1492 池读回 UAF 检测);提升局部变量即规避(p09g/p13a 对照) | p09h(最小复现)/ p09 / p13b |
| V12 | while / `for cond` 循环体内 `continue` + 后续语句 | ❌ **穿透执行 continue 之后的同迭代语句**(p06 多算 acc;p07 "cond 2" 不应打印) | p06/p07 |
| V13 | range-for 循环体内 `continue` | ✅(2026-07-25 `665410b49` 已修路径) | p07 |
| V14 | is 语句值语义(函数尾直接作为返回值) | ✅ | p01/p02b |
| V15 | **`模式 + if 卫语` 组合臂(Rust `VI(n) if n>10`)** | ✋ **Auto 语法不存在**(解析报错;卫语只能独立成臂)——非 bug,是一对一改写时的**永久性 DIVERGE 点** | p13 首版 |

### 2.2 a2r 侧(`auto.exe trans --path X.at rust` + `rustc --edition 2021` 实编译)

| # | 特性 | 结果 | probe |
|---|---|---|---|
| A1 | is-on-string → `match x.as_str()`(or-臂/`_`/`.to_string()`) | ✅ 转译正确,实编译零错 | p01 |
| A2 | 标量 enum 声明(+Display/from_id 自动派生)、is 单臂与 or-臂 | ✅ 转译正确(`TokenKind::Int \| TokenKind::Uint`),实编译零错 | p02b |
| A3 | enum 载荷声明/构造(`.to_string()` 强转)/is 绑定 | ✅ 实编译零错 | p04/p05/p12 |
| A4 | is 载荷绑定下游使用(`.len()`/`List.get(0)`/`.str()`)——KNOWN-DEBT 113 靶形 | ✅ 本形态不复现,实编译零错 | p12 |
| A5 | **同一枚举值连续两次 is 匹配** | ❌ **E0382 use of moved value**(首次按值匹配即 move;需 last-use 分析发 `match &v` 或臂内 clone) | p05 |
| A6 | **is 卫语句臂** | ❌ **生成非法 Rust**:`n > 100 if true => "big"`(条件表达式被放进模式位,rustc 报 expected `=>`/`if`/`\|`) | p08 |

结构体字面量 `Frame { val: ..., name: ... }`、`List.new/push/get/len`、f-string 的
转译在上述 probe 中一并验证通过(p12/p13a 等隐含)。

### 2.3 最小复现件(立项时原样搬入 `test/vm/` 作回归)

```auto
// H1:枚举 or-臂只有第一个模式命中(p02b 摘录)
fn multi(k TokenKind) str {
    is k {
        TokenKind.Int | TokenKind.Uint -> "num"   // Uint 永远匹配不上,落 else
        else -> "eof"
    }
}

// H3:运行期值内联在枚举构造参数位 → RC canary 崩溃(p09h 全文)
enum Val {
    VI(int)
    VS(str)
    VN
}
fn main() {
    let xs = List.new()
    xs.push(Val.VS("a" + "b"))     // 崩;先 var s = "a"+"b" 再 VS(s) 则正常
    print(xs.len().str())
}

// H2:while 内 continue 穿透(p07 摘录;range-for 对照正常)
var i = 0
while i < 4 {
    i = i + 1
    if i == 2 { continue }
    print("cond " + i.str())       // bug:会打印 "cond 2"
}

// A6:a2r 卫语句臂产物(rustc 不过)
match n {
    0 => "zero".to_string(),
    n > 100 if true => "big".to_string(),   // ← 非法模式
    _ => "small".to_string(),
}
```

## 3. 宿主修复清单(立项项 H1-H6)

> 修完 H1/H2/H3 + A5/A6(或明确绕过),lib 风格升级的宿主前提即全部就绪。
> 每项独立小、可并行;建议合并为一个"VM/a2r is-enum 加固"计划。

### H1 · VM:枚举路径 or-臂失效【必须,阻塞 is-on-enum 全面使用】

- **根因**(本次定位):`Stmt::Is` 的 EqBranch 处理中,`Expr::Cover` 分支
  (vm/codegen.rs:3537)先于多模式分支(codegen.rs:3636)命中——**首个模式是枚举
  路径时走 Cover 分支只编译第一个模式,`patterns[1..]` 被整个忽略**;多模式分支
  (逐模式 EQ + JMP_IF_NZ 短路或,逻辑本身正确)仅在首模式为字面量等非 Cover 形态
  时到达,故字符串/字符 or-臂正常。
- **修复**:Cover 分支入口处若 `patterns.len() > 1` 改走多模式路径(枚举判别值 EQ
  可直接复用);**注意 `Expr::Is`(codegen.rs:9348 起)是同逻辑复制粘贴的第二份,
  必须同步修**。
- **验收**:p02b(含 single/multi 双函数)+ 补入 `test/vm/` 常驻;M1-M5 全绿。

### H2 · VM:while/Cond-for/ever 循环 continue 穿透【必须,D17 根治】

- **根因**(本次定位):`Stmt::Continue`(codegen.rs:3319-3352)发 `JMP` i16 占位符
  压入 `loop_continues`,但除 range-for(codegen.rs:2588,commit `665410b49` 修过)
  外,**其余 7 处循环变体收尾只 `let _ = loop_continues.pop()` 不回填**——占位符
  值 0 使 JMP 落在下一条指令,即"穿透执行同迭代余下语句"。未回填位点:
  Iter::Named Call 子路径 2670、Indexed 2803、Call/iter 2888、Destructured 2999、
  3129、**Cond(while)3189**、Ever 3220、3222 变体 3300。
- **修复**:照 2588 的模式在各变体 continue 目标已知处 `patch_jump_to`。
- **验收**:p06/p07 常驻;现有全量 VM 语料零回归(现有语料恰好零覆盖 while+continue,
  即 D17 之前不被闸门看见的原因)。

### H3 · VM:运行期值内联在枚举构造参数位 → 池 RC 赤字崩溃【必须(或升级期强制规避)】

- **表象**:`[RC canary] string tombstone access`(engine.rs:1492 池读回 UAF);
  最小复现 p09h(原生调用实参位)/ p13b(结构体字面量字段位);**提升局部变量即规避**
  (p09g/p13a 对照绿)。与 D25-①(num_locals 启发式偷弹)、D36-① 同族。
- **根因候选**(VM 调研结论,两处叠加):①原生参数编组 `pop_arg_i32`
  (native.rs:8538)按 i32 解码,TAG_OBJECT/TAG_STRING 经 native 即降级;CALL_NAT
  死区结算(engine.rs:6698-6703)对错弹的槽统一 rc_release,兄弟表达式份额被错杀;
  ②枚举/结构体构造器内联在实参位时 `shim_list_new` 的 `sp > bp+num_locals+2`
  "有参"启发式(native.rs:1655-1765)偷弹兄弟槽。**根治方向**:CALL_NAT 显式
  arg_count 替代帧几何猜测(同一刀顺带根治 D25-①)。
- **验收**:p09h/p13b 转绿(不提升局部也正确);`repro_242_string_pool_uaf` 等既有
  RC 回归零变化。**若暂缓根治**:升级期 lib 写法规范强制"枚举构造参数先提升局部"
  (现有 D36-① 规范延续),但 engine.at 的 Val 枚举化(§6-E2)仍应先修此项。

### H4 · a2r:卫语句臂发射非法 Rust【必须,若 lib 使用卫语臂】

- **现状**:`IsBranch::IfBranch`(trans/rust.rs:12935-12940)发 `self.expr(expr)` +
  `" if true => "`——条件表达式进了 Rust 模式位。p08 实编译 2 错。
- **修复**:发 `binding/scrutinee 别名 if <cond> =>`(Rust `x if x > 100 =>`);
  scrutinee 为复杂表达式时先 `let __s = expr;` 再 `match __s`。全仓 golden 无一
  覆盖(零 golden 波动风险)。
- **注意**:Auto 无"模式+guard"组合臂(V15),Rust 参考里的 guard 形态在一对一改写时
  本就要拆成独立卫语臂或臂序重排(新增 DIVERGE 条目登记),H4 是该改写能落地的前提。

### H5 · a2r:同一枚举值二次 is 匹配 E0382【必须,lib 内同函数双匹配是常见形态】

- **现状**:p05 首次 `match v` 按值绑定 `s` 即 move,第二次 `match v` 报 use of
  moved value。Auto 语义(VM)可重复匹配。
- **修复**:scrutinee last-use 分析——非末次使用发 `match &v`(臂绑定自动变引用,
  String 用法兼容 `format!`/`println!`);或臂内绑定 clone。与 433 修过的 struct
  字段读 auto-clone(242 #18)同族决策。
- **验收**:p05 实编译零错;a2r golden 06 组回归。

### H6 · a2r:is-绑定变量类型跟踪(KNOWN-DEBT 113)【观察项,非当前阻断】

- p12 靶形(t.len()/List.get 下游)已正确;登记的失效面(`.get` 泛型改写/struct
  字段解析)建议在 H5 动同一片代码时顺手补:is 臂发射时按枚举载荷表回填
  `local_var_types`(枚举表数据已有:`enum_tuple_field_types`,trans/rust.rs:14735+)。

## 4. AA2R(auto/lib/a2r.at)扩展清单【自举塔硬前提:lib 用什么语法,a2r.at 必须先能发射什么】

> 顺序不可倒置:gate ⑤(五方矩阵的 AA2R 路径)要求 lib 七文件全部可被 a2r.at
> 自身转译。a2r.at 的扩展实现仍用**现有最简子集**书写(塔式爬升),全部完成并
> 闸门绿后,才轮到 lib(含 a2r.at 自己)重写风格。

| # | 扩展 | 改动点(a2r.at,行号为当前快照) | 依赖 |
|---|---|---|---|
| W1 | is-match 发射 | `ar_stmt`(1690)加 `Is` 分支 → 新 `ar_is`:scrutinee 文本(任一臂 Str 字面量则 `.as_str()`,镜像主 a2r rust.rs:12781-12815);臂循环:模式组 `\|`(`Ident Dot Ident` 路径→`Enum::Variant`、Some/None、字面量、`_`)`=>` 臂体(单语句内联/多语句块/空块,镜像 write_match_arm_body 12631);`else ->` → `_ =>`;**从 parser.at `is_unsupported_stmt_kind`(1556)摘除 "Is"** | 无(独立) |
| W2 | or-臂发射 | W1 模式组循环天然覆盖(`p1 \| p2`) | W1 |
| W3 | 枚举载荷声明/构造/绑定 | `ArEnum`(62-65)增每变体载荷类型;`ar_prescan_enum`(673-719,现 699-701 遇 `LParen/LBrace` 报 unsupported)解析 `Name(T)`/`Name{f T}`;`ar_emit_enum2`(2075-2132)按形态发元组/结构/单元变体 + derive 三分派(现无条件全 derive,含 float 载荷时与主 a2r 不一致——D40 清单要扩);构造点 `ar_call_tail`/`ar_method_call`(1528/1270)加枚举变体构造判定 + str 载荷 `.to_string()`;**绑定名 `ar_vpush` 进作用域并查载荷表**,否则复刻主 a2r 的 H6 缺口 | W1(共用臂发射) |
| W4 | f-string 发射 | **已就绪**(`ar_fstr_parse` 1135-1205,Plan 434 D38c);文件头 Missing 清单更新 | — |
| W5 | ext/impl 块发射(可选,二期) | `ar_run`(2372)加 `Ext/Impl` 分支 + `ar_emit_ext2`(`impl T {}` / `impl Trait for T {}`);方法体复用 `ar_emit_fn2`(2184)+ 接收器位(`&self`/`&mut self`,突变判定复用 `ar_scan_mutations` 730);`self` 以目标类型 `ar_vpush` 进作用域 | 一对一改写用到 impl 时才需要 |
| W6 | 闭包发射(可选,锦上添花) | 迭代器链(`.any/.find` 位)还原 Rust 迭代器写法时才需要 | — |

**闸门影响**:M1-M5 语料闸门不碰 lib 语法,dump 的是 corpus 文件,不动即不变;
②(主 a2r 编译 lib)与 ⑤(AA2R)内容寻址缓存自动失效重建;主 a2r golden 仅在
H4/H5 触及发射逻辑时波及 06 组个别文件。**唯一实体 golden**:001_smoke /
002_hello_compile 的 `.expected.out`,风格改写不改变行为,预期不动。

## 5. lib 改写点位清单(函数/块级,§依赖列 = 上述 H/W 项)

> 统计口径:`== "`/`!= "` 全量 grep 扣除 err 槽位等噪音;`+ "` 为字符串拼接计数。
> 行号基于当前 auto/lib 快照。

| 文件 | 改写点 | 依赖 | DIVERGE 对应 |
|---|---|---|---|
| token.at(354 行) | T1 `keyword_kind` 58 臂 if 链 → `is text {...}`(is-on-string);T2 `kind_name` 139 臂 → `is k {...}`(is-on-enum,枚举已是 TokenKind) | V1✅/A1✅;V4/A2✅(**or-臂等 H1**) | D11/D11b |
| lexer.at(640 行) | L1 `Token.kind/Tok.kind: str → TokenKind`(构造点 36 处,输出位经 kind_name);L2 转义/定界 else-if 链 → is-on-char;L3 `delim_name` 并入主链 | L1 需 V6-V10✅ + **H3**(或继续提升局部规避);L2 需 V2✅/A 组✅ | D14 残留/D11b/D17(保留) |
| parser.at(1690 行) | P1 `p_kind→TokenKind`、`p_expect` 参数化(kind 位字符串比较 ≈330 处,31 个函数);P2 Pratt/stmt 巨链 is 化(`expr_with_left` 33 处等);P3 **新增 `enum Op` + `p_op()`**(杠杆点:一步收编 infix_l/r、op_display、binop_result 及下游三文件共 6 个长链);P5 FStrStart 解析打通(lib 自身用 f-string 的前置) | 前置 L1;is-on-enum(H1) | D11b/D14/D20(E.kind 字符串比较需真实 AST 决策,可分期) |
| typeinfo.at(479 行) | Y1 `t_literal_type/t_binop_result` is 化;`t_array_elem` 的字符串形状解析 → Type 载荷化 | P1/P3;载荷跨函数✅(V6-V8) | D23/D27 |
| codegen.at(949 行) | C1 `I.op: str → enum OpCode`(30 种助记符);C2 `i_size/cg_binop_mnem` is 化;C3 cg_expr/cg_stmt 的 p_kind 链 is 化 | P1;is-on-enum(H1) | D28/D25(行为位保留) |
| engine.at(485 行) | E1 `ev_run_t` 56 处 `op ==` 巨链 → `is ins.op`(前置 C1);**E2 `Val{k,i,s} 判别器 → enum Val{VInt/VStr/VArr}`**(8 个 ev_* 函数的 k 分派还原) | E1:C1+H1;E2:V6-V10✅ + **H3 修复**(Val 将作为 List 元素/参数高频流经构造位与 native 位) | **D34**(核心目标)/D35(arena 语义保留)/D36①② 可回退 |
| a2r.at(2432 行) | A1 `ar_method_call/ar_is_mutating_method/ar_rust_ty` 等词法链 is 化(方法名/类型名本就是字符串,is-on-string);A2 p_kind 链 is 化;338 处 `+ "` 拼接按 Rust format! 位逐点换 f-string | 前置 P1 + W4(已就绪)+ **W1 先行**(a2r.at 自己也要被自己转译) | D11b 补记/D39/D40 |

**总量**:可 is 化函数约 83 个;kind/op 字符串比较 enum 化约 1000+ 处;判别器结构体
enum 化 3 组(Val/E 载体/OpCode)。行为层 DIVERGE(D12 游标、D16 char 计长、D18/D19
quirk 照抄、D28 布局等价、D30/D31 编码语义)**不在本轮范围**,继续保留。

## 6. 建议波次(已立项,见 docs/plans/ 三件套)

```text
Plan 447(aavm-prerequisites-1)宿主加固:H1 + H2 + H3 + H4/H5(+H6 顺手)
  验收 = §2.3 复现件常驻 test/vm/ 全绿 + 全量回归零变化
Plan 448(aavm-prerequisites-2)aavm 新语法能力:parser/typeinfo/codegen/engine
  的 is 解析-编译-执行 + AA2R 发射 W1+W2+W3(f-string 已就绪)
  验收 = aavm2 闸门全绿 + AA2R golden 06 组落盘
Plan 449(aavm-prerequisites-3)lib 风格升级:γ1 token/lexer → γ2 主干 is 化 +
  三枚举化(Val/I.op/Op)→ γ3 a2r.at 自身 + 338 处 f-string 化 + divergences 收账
  每步五方矩阵 ①③④⑤ 必须保持全绿(判据 = 行为不变,风格变)
```

## 7. 语法面备忘(升级期写法规范,补入 divergence-rules.md §4)

1. 单行枚举声明需逗号+显式值(`enum X { A = 1, B = 2 }`),无值变体必须逐行;
2. Auto 无"模式 + guard"组合臂:Rust `VI(n) if n > 10` → 拆独立卫语臂或调臂序
   (新增 DIVERGE 编号登记);
3. 同函数内对同一枚举值多次 is:VM 无碍;a2r 修复(H5)前改写为多次传参或局部复制;
4. 枚举构造参数位**暂不内联运行期计算值**(提升局部;H3 修复后解除);
5. is 语句具值语义,可直接作函数尾返回(p01/p02b 实证)。

## 8. 本次调研产物索引

- probe 18 件:`tmp/p-idiom/p01..p13c`(gitignore;§2.3 已内联关键复现,立项时搬
  `test/vm/99_planXXX/` 或 `test/vm/aavm2/probe_idiom/`);
- 三份深挖底稿(本会话产出,结论已并入本文):VM is/enum 实现与 D34 根因链
  (push_value Str 臂 engine.rs:1116-1125 漏 `rc_push_str_idx` / pop_arg_i32 编组 /
  IS_VARIANT 静默兜底)、主 a2r 特性矩阵(377 处 match 发射路径)、lib↔Rust 函数级
  差异台账(§5 的展开版);
- 相关既有文档:`divergences.md`(26 类登记)、`divergence-rules.md`(§4 编码规范)、
  `docs/plans/242-a2r-feature-gap-tracker.md`、`KNOWN-DEBT-AND-RISKS.md:112-113`。
