# Plan 447: aavm-prerequisites 三合一——宿主加固 → aavm 语法能力 → lib 风格对译

> **状态**: 🟦 已立项待执行(2026-08-25;aavm-prerequisites 三件套合并为单一计划,
> 原 447/448/449 合并,序号 448/449 腾空)
> **来源**: [idiom-upgrade-prereqs.md](../specs/aavm/idiom-upgrade-prereqs.md)(2026-08-25
> 实证调研:18 件 probe + rustc 实编译验证),§3 H1-H6、§4/§5/§6 语法面与改写点位。
> **定位**: 把 aavm lib 从"类 C 最简风格"升回"与 Rust 参考一对一"的**完整前提**,按
> 顺序分三部分:**① 宿主加固**——修掉自举路径暴露出的宿主缺陷,而不是继续绕开;
> **② aavm 新语法能力**——让 aavm 自身获得 is-match 与枚举载荷的完整处理;
> **③ lib 风格升级**——把 432/434 因宿主缺陷而降级的写法还原为一对一形态。
> **纪律**: 三部分**塔式爬升,顺序不可倒置**;前一部分合入并通过各自闸门后方可开工
> 下一部分。本计划合并前的原分叉为 447(①)、448(②)、449(③),现统一为一份。
> **阶段编号(合并后统一)**: 部分①= Phase 1-3;部分②= Phase 4-7;部分③= Phase 8-11。
> **关联**: [divergences.md](../specs/aavm/divergences.md) D11b/D17/D25-①/D34/D36-① 收账对象;
> [divergence-rules.md](../specs/aavm/divergence-rules.md) §4 纯 Rust 模式编码规范;
> [m2-ast-dump-format.md](../specs/aavm/m2-ast-dump-format.md);
> KNOWN-DEBT-AND-RISKS.md:112-113。
> **基线**: master `c3ee519d0` + 2026-08-25 构建;aavm2 闸门当前 10 项全绿。

---

## 部分① 宿主加固(原 Plan 447)

> **范围**: 把 2026-08-24 移植时绕开、且今天**实测仍坏**的 6 项宿主缺陷全部修复或
> 明确降级为写法规范,使 Auto 语言的 is-match / 枚举载荷 / continue 在 VM 与 a2r
> 两侧达到"aavm 可放心使用"的成熟度。全部复现件常驻 `test/vm/` 成为永久回归。

### 目标

把 2026-08-24 移植时绕开、且今天**实测仍坏**的 6 项宿主缺陷全部修复或明确降级为
写法规范,使 Auto 语言的 is-match / 枚举载荷 / continue 在 VM 与 a2r 两侧达到
"aavm 可放心使用"的成熟度。全部复现件常驻 `test/vm/` 成为永久回归。

实证依据(prereqs §2,复现件源码内联于 prereqs §2.3,临时原件在 `tmp/p-idiom/`):

| # | 缺陷 | 实证 | 根因(已定位) |
|---|---|---|---|
| H1 | VM 枚举 or-臂仅第一个模式命中 | p02b(`multi(Uint)`→else) | codegen.rs:3537 `Cover` 分支先于多模式分支(3636)命中,`patterns[1..]` 被忽略;`Expr::Is`(9348 起)为同逻辑复制粘贴副本,须同步 |
| H2 | while/Cond-for 内 continue 穿透 | p06/p07 | `loop_continues` 占位符除 range-for(2588,`665410b49`)外 7 处变体从不回填:2670/2803/2888/2999/3129/3189/3220/3300;JMP+0 落到下一条指令 |
| H3 | 运行期值内联枚举构造参数位 → RC canary 崩溃 | p09h(原生调用实参位)/p13b(结构体字面量字段位);p09g/p13a 提升局部即绿 | 原生编组 `pop_arg_i32`(native.rs:8538)降级 + `shim_list_new` 帧几何启发式(native.rs:1655-1765)偷弹兄弟槽 + CALL_NAT 死区结算(engine.rs:6698-6703)错杀份额;与 D25-① 同根 |
| H4 | a2r 卫语句臂生成非法 Rust | p08(`n > 100 if true =>`,rustc 2 错) | trans/rust.rs:12935-12940 条件表达式进模式位 |
| H5 | a2r 同一枚举值二次 is 匹配 E0382 | p05 | 首次按值匹配即 move;缺 last-use `match &v` / 臂内 clone 决策 |
| H6 | a2r is-绑定变量类型跟踪(DEBT 113) | p12 靶形不复现,观察项 | local_var_types 写入点缺 is 臂载荷回填(enum_tuple_field_types 数据已有) |

### 任务

#### Phase 1:复现件常驻(随各修复项一并落,不留红 master)

- [ ] 1.1 建 `test/vm/99_idiom_probe/`:p01/p02b/p06/p07/p08/p09h/p13b 等按
  `名称.at + 名称.expected.out` 平铺落盘(期望值 = **正确**语义输出),随对应
  H 项修复同 PR 落地;p03/p04/p05/p10/p13a/c 等绿件直接先落(纯加固回归)。
- [ ] 1.2 `vm_file_tests` runner 确认新目录被自动发现(平铺文件惯例同
  `10_types/020_struct_to_str.at`);跑 `cargo test -p auto-lang --lib
  --features test-vm-files -- test_aavm2` 基线留档。

#### Phase 2:VM 三修(H1/H2/H3)

- [ ] 2.1 **H1 枚举 or-臂**:`Stmt::Is` 的 EqBranch 处理(codegen.rs:3537 起)在
  `patterns.len() > 1` 时改走多模式路径(逐模式 EQ + JMP_IF_NZ,枚举判别值可直用);
  **同步修 `Expr::Is` 副本(9348 起,两份代码逐段对照)**;p02b 进 99_idiom_probe。
- [ ] 2.2 **H2 continue 回填**:照 2588 的 `patch_jump_to` 模式补齐 7 处变体
  (Cond=while、Ever、Named/Indexed/Call/Destructured 系);p06/p07 进常驻;
  全量 VM 语料回归(现状恰好零覆盖 while+continue,修复本身无 golden 波动)。
- [ ] 2.3 **H3 根治(目标)或规避(兜底)**:
  - 根治:CALL_NAT 携带显式 arg_count 替代 `shim_list_new` 的 `sp > bp+num_locals+2`
    猜测(一刀同时收 D25-①);审计 `pop_arg_i32` 族算术敏感 shim 改 `pop_arg_nv`
    + tag 分派;p09h/p13b 转绿(不提升局部也正确)。
  - 兜底(若根治涉及面过大单独分期):H3 降级为写法规范(lib 永远提升局部),
    但 **部分③ 的 engine.at Val 枚举化(Phase 10)前必须完成根治**,本项
    不得以兜底状态进入部分③。
  - 回归:`repro_242_string_pool_uaf`/`repro_d30_negative_int_roundtrip` 及既有
    RC 用例零变化;aavm2 闸门全绿。
- [ ] 2.4 顺手加固(可选,不阻塞):`IS_VARIANT`(engine.rs:4006-4026)对"非对象
  非 null 的用户 enum"默认按 Option.Some 处理改为显式错误;`CONSTRUCT_INSTANCE`
  越界回退(3904-3907)改报错。目的:把静默错臂变成显式失败,服务后续调试。

#### Phase 3:主 a2r 两修 + 一补(H4/H5/H6)

- [ ] 3.1 **H4 卫语句臂**:IfBranch 发射改为 `<绑定> if <cond> =>`;scrutinee 为
  复杂表达式时先 `let __is_x = expr;` 再 `match __is_x`;p08 转译产物 rustc 零错。
  现有 golden 全仓零覆盖卫语句(零 golden 波动)。
- [ ] 3.2 **H5 二次匹配 E0382**:scrutinee last-use 分析——非末次使用发
  `match &v`(臂绑定变引用,`format!`/`println!` 用法兼容);p05 产物零错;
  a2r golden 06 组回归。
- [ ] 3.3 **H6(顺手)**:与 3.2 动同一片代码时,is 臂发射按枚举载荷表回填
  `local_var_types`(数据源 `enum_tuple_field_types`,rust.rs:14735+),KNOWN-DEBT
  113 观察项收口或明确残留。

### 风险与缓解

| 风险 | 缓解 |
|---|---|
| H3 根治动 RC 记账咽喉,牵连面大 | 分两刀:先 arg_count 显式化(纯增量),后 pop_arg_nv 收窄;每刀全量回归;兜底路径已定义 |
| H1 的 Expr::Is/Stmt::Is 双副本漏改 | 修复后加一条"双副本同源"注释;probe 覆盖语句位+表达式位两形态 |
| H5 的 `match &v` 改写引入借用新错 | 先只在"同一函数内 ≥2 次 is 同一 scrutinee"的窄条件启用;golden 全量 diff 审 |
| 修复与 plan-442 执行中的 VM 改动冲突 | 开工前 rebase;H 项全部小粒度独立提交 |

### Out of Scope

- AA2R(a2r.at)的任何改动(→ 部分②);
- lib 风格改写(→ 部分③);
- ext/spec/闭包/f-string 等其它特性面(当前实证非阻断);
- VM 性能优化。

### Verification

1. `99_idiom_probe` 全绿(含 H1/H2/H3 复现件转绿);
2. `cargo test -p auto-lang --lib` 全量零回归(3876+ 用例);
3. a2r golden(25 组/220 文件)除 H4/H5 预期组外零漂移;
4. aavm2 闸门 M1-M5 + smoke 全绿;
5. H3 根治路径下:p09h/p13b 不提升局部也正确;D25-① 场景(表达式位零参原生调用)
   一并验证;
6. divergences.md 登记对应收账(D17/D25-①/D36-① 状态更新),KNOWN-DEBT 112/113
   状态更新。

---

## 部分② aavm 新语法能力(原 Plan 448)

> **范围**: 让 aavm(auto/lib 七文件)**自身获得 is-match 与枚举载荷的完整处理
> 能力**——前端解析(dump 判据层)、类型面、编译执行、以及 AA2R(a2r.at)的
> Rust 发射。这是部分③ 风格改写的**自举塔硬前提**:lib 用什么语法,
> a2r.at 必须先能发射什么。
> **纪律**: 本部分全部新代码仍用**现有最简子集**书写(if 链/无 is/无载荷枚举);
> 风格化属于部分③。塔式爬升,顺序不可倒置。
> **前置**: 部分①(H1 or-臂宿主正确性、H4/H5 a2r 发射正确性——golden 文本的对齐基准)。

### 目标与缺口

aavm 当前对 is 的全部接触面都是"拒收":parser.at:1556 `is_unsupported_stmt_kind`
清单含 Is;a2r.at 双处(ar_run:2382 / ar_stmt:1744)报 "v2 unsupported";
enum 载荷声明在 ar_prescan_enum:699-701 被拒。能力补齐分四段:

| 段 | 内容 | 判据 |
|---|---|---|
| A | parser.at 解析 is → S-expr dump | corpus_m2 扩 is 语料,M2 闸门(与 Rust 参考 dump 逐字符一致) |
| B | typeinfo.at is 类型面 | corpus_m3 扩,M3 闸门 |
| C | codegen.at 发射 + engine.at 执行 is | corpus_m4/m5 扩 is 行为语料,M4/M5 闸门 |
| D | AA2R(a2r.at)发射 is/or-臂/枚举载荷 → Rust match | AA2R golden 06 组新落盘,与主 a2r(已修 H4/H5)产物对齐 |

宿主侧 is 各形态的可用性实证(prereqs §2.1:is-on-string/char/卫语句/枚举载荷
直接传参全部 ✅;or-臂待 H1)——**本部分做的是把这套能力在 Auto 写的编译器里
复刻一遍**,顺带成为这些形态的第二次大规模实战验证。

### 任务

#### Phase 4:parser.at 前端(A 段)

- [ ] 4.1 `parse_is` 直译(镜像宿主 parser.rs:7511-7644 的 parse_is/parse_is_branch
  决策:EqBranch 单模式/多模式 `|`、IfBranch 卫语句、ElseBranch;模式形态:字面量
  /字符/`Cover(Tag)` 枚举路径含 `Name(bound)` 绑定与 `_`);从
  `is_unsupported_stmt_kind` 摘除 "Is"。
- [ ] 4.2 dump 形态对齐:is 语句/臂/模式的 S-expr 与 Rust 参考 m2 dump 逐字符一致
  (对照 m2-ast-dump-format.md;Rust 侧 dump 以 live 对比为唯一依据,发现宿主
  dump 漂移按 D18 quirk 照抄规则处理)。
- [ ] 4.3 corpus_m2 新增 is 语料组(p15/p16 后续编号):is-on-string、is-on-char、
  or-臂、枚举载荷解构、卫语句、else、嵌套块体、值语义位;M2 全绿。

#### Phase 5:typeinfo.at(B 段)

- [ ] 5.1 `t_stmt_walk` 增 is 语句(臂体走查;绑定变量按枚举载荷表入局部作用域,
  镜像宿主 infer 对 Uncover 的处理);`t_infer_expr` 若含 is 值语义位同口径。
- [ ] 5.2 corpus_m3 扩 `.type` 查询语料(臂内绑定类型打印);M3 全绿。

#### Phase 6:codegen.at + engine.at(C 段)

- [ ] 6.1 codegen.at 发射(镜像宿主 codegen.rs:3347-3735 决策,不照抄实现):
  `_is_target` 暂存局部;EqBranch——字面量/字符 → EQ;标量枚举 → 判别值 EQ;
  载荷枚举 → IS_VARIANT + GET_GENERIC_FIELD 逐字段绑定;or-臂 → 逐模式 EQ +
  短路或(宿主 H1 修后口径);IfBranch → 条件 + 条件跳转;臂尾 JMP 汇出(占位回填)。
- [ ] 6.2 engine.at 执行:IS_VARIANT / GET_GENERIC_FIELD 两指令(aavm 指令集已含
  声明,opcode.at;执行语义镜像宿主 engine.rs:3988-4130 但走 v2 自身值栈/arena
  模型 D29/D35);注意 v2 布尔 0/1 承载与负值编码(D30)在 EQ 上的口径。
- [ ] 6.3 corpus_m4(反汇编)/corpus_m5(行为)扩 is 语料;M4/M5 全绿——这是
  aavm 对 is 的**端到端**判据。

#### Phase 7:AA2R(a2r.at)发射(D 段;与 Phase 4-6 可并行,依赖部分① 的 H4/H5)

- [ ] 7.1 **W1 is-match 发射**:`ar_stmt`(1690)加 `Is` 分支 → 新 `ar_is`:
  scrutinee 文本(任一臂 Str 字面量 → `.as_str()`,镜像主 a2r rust.rs:12781-12815);
  臂体三形态(单语句内联/多语句块/空块,镜像 write_match_arm_body 12631);
  `else ->` → `_ =>`;模式解析按 D39 token 游标直走(不能复用 ar_expr 的条件
  表达式语义)。
- [ ] 7.2 **W2 or-臂**:模式组 `|` 分隔发射(`p1 | p2`)。
- [ ] 7.3 **W3 枚举载荷**:`ArEnum`(62-65)增每变体载荷类型表;
  `ar_prescan_enum`(673-719)解析 `Name(T)`/`Name{T1,T2}`/`Name{f T}`;
  `ar_emit_enum2`(2075-2132)按形态发元组/结构/单元变体 + **derive 三分派**
  (float/Map/嵌套枚举安全性,对齐主 a2r 14477-14543;现无条件全 derive,列入
  D40 差异清单修正);构造点 `ar_call_tail`/`ar_method_call`(1528/1270)加
  枚举变体构造判定 + str 载荷 `.to_string()`;**绑定名 `ar_vpush` 进作用域并查
  载荷表登记类型**(不复制主 a2r 的 DEBT-113 缺口)。
- [ ] 7.4 文件头 Snapshot 更新:Missing 清单摘除 is-match/枚举载荷;**f-string
  实际已支持(D38c),清单同步纠正**;D40 清单补 derive 三分派项。
- [ ] 7.5 AA2R golden:06_pattern_matching 组(is-match/or-臂/枚举载荷)落盘,
  文本对齐主 a2r(基线 = 已修 H4/H5 的 master);probe 文件(p01/p02b/p04/p05/
  p12)转译冒烟 + rustc 编译通过。

### 风险与缓解

| 风险 | 缓解 |
|---|---|
| is 的 S-expr dump 与 Rust 参考存在格式历史 quirk | 4.2 以 live 对比为唯一依据;quirk 按 D18 模式登记照抄,不擅自"修好" |
| Phase 6 把宿主 codegen 的复制粘贴双副本问题带进 v2 | 6.1 只镜像**决策**,实现走 v2 单遍步行 + 游标快照方法论(432 先例);语句/表达式两入口写同一辅助函数族 |
| AA2R golden 对齐基线漂移(H4/H5 未合入) | 7.5 明确 gating 在部分① Phase 3 合入之后 |
| D39 token 直走在模式位与表达式位歧义 | 模式解析独立函数,不碰 ar_expr;语料带足 `a - 1 ->`、`'x' ->`、`A.B(c) ->` 歧义形 |

### Out of Scope

- lib 自身的风格化改写(→ 部分③;本部分 lib 语法面不变);
- spec/ext/impl/use/dep/闭包/泛型声明的 AA2R 发射(§5 二期,只有 γ 计划用到
  impl 时再立项);r2a/多目标/post_process 正则族(Plan 434 永久 Out of Scope);
- 宿主缺陷修复(→ 部分①)。

### Verification

1. M1-M5 闸门全绿(含 4.3/5.2/6.3 新语料;旧语料零变化);
2. AA2R golden 06 组与主 a2r 产物逐字符一致(D40 白名单外零差异);
3. 五方矩阵 ①-⑤ 全绿(lib 未变,③④⑤ 基线不动即为回归通过);
4. probe 转译冒烟:p01/p02b/p04/p05/p12 经 AA2R 转译后 rustc 零错;
5. divergences.md:D38 系列补记 is 能力位;corpus.md 登记新语料分层归属。

---

## 部分③ lib 风格升级(原 Plan 449)

> **范围**: 用户既定长期路线——aavm 长期锚定"与 Rust 参考实现函数级/块级一对一
> 对译";更远的"脱离 Rust 参考的纯 Auto 风格自研实现"不在本计划范围。本部分把
> 432/434 因当时宿主缺陷而降级的写法(if 链替代 match、kind 字符串载体、判别器
> 结构体)**还原为一对一形态**,行为零变化。
> **硬前提**: 部分①(H1 or-臂 / H3 内联载荷参数 / H4 卫语句 / H5 二次匹配)+
> 部分②(aavm 与 AA2R 已具备 is/枚举载荷能力)全部合入;开工时五方矩阵基线全绿。
> **总判据**: **行为不变,风格变**——M1-M5 语料闸门、五方矩阵 ①③④⑤、实体 golden
> (.expected.out)全部保持绿;唯一允许的 diff 来源是 divergences.md 的新增/收账登记。

### 改写总量盘点(prereqs §5 摘要)

- 可 is 化函数约 **83 个**;kind/op 字符串比较枚举化约 **1000+ 处**;判别器结构体
  enum 化 3 组(engine.at 的 `Val{k,i,s}`、codegen.at 的 `I{op,s,n}`、parser.at 的
  E 载体——E 载体属 D20 dump 判据层,本部分**不动**);
- 字符串拼接 → f-string 按位还原 **338 处**(`+ "` 计数);
- 杠杆点:parser.at 新增 `enum Op` + `p_op()`,一步收编 infix_l/infix_r/op_display/
  binop_result 及 typeinfo/codegen 下游共 6 个长链函数;
- 行为层 DIVERGE(D12 游标/D16 char 计长/D18/D19 quirk 照抄/D28 布局等价/D30/D31
  编码语义/D17 保留位——若部分① 已根治 D17 则可回写 continue)**不在**本轮范围,继续保留。

### 任务

#### Phase 8:准备

- [ ] 8.1 divergence-rules.md §4 补写法规范(基于 prereqs §7 实证语法备忘):
  ①单行枚举需逗号+显式值,无值变体逐行;②Auto 无"模式+guard"组合臂——Rust
  `VI(n) if n>10` 拆独立卫语臂或调臂序(**新增永久 DIVERGE 编号登记**);③同函数
  对同一枚举值多次 is 的写法约定(依赖 H5 已修);④枚举构造参数位暂不内联运行期
  计算值(若 H3 走了兜底路线,此条长期保留;若已根治,登记解除)。
- [ ] 8.2 基线锚定:当前 master commit + 全量闸门跑绿留档(M1-M5 + 五方矩阵
  ①③④⑤ + .expected.out 双件);后续每个 γ 子步以此为对照。

#### Phase 9:γ1 样板先行——token.at + lexer.at(M1/M2 闸门保护下)

- [ ] 9.1 token.at:T1 `keyword_kind` 58 臂 if 链 → `is text {...}`(is-on-string,
  对齐 Rust token.rs:365-449 的 match &str);T2 `kind_name` 139 臂 → `is k {...}`
  (is-on-enum;D11 哨兵写法保留);头 Snapshot 与 divergences.md D11b 收账。
- [ ] 9.2 lexer.at:L1 `Token.kind/Tok.kind: str → TokenKind`(构造点 34+2 处,
  `lex_number` 的 kind 变量、`lex_dump`/p_err 输出位经 kind_name);L2 转义/定界
  else-if 链 → is-on-char(lex_string/lex_char/lex_fstr_at 对齐 Rust lexer.rs:462/
  386 的 match esc);L3 `delim_name` 9 臂并入主链(Rust 无此函数,臂内联于
  next_step)。
- [ ] 9.3 闸门:M1(token dump 逐字符)/M2 全绿;②/⑤ 缓存重建后全绿;若 H3 未
  根治,kind 构造点保持提升局部写法。

#### Phase 10:γ2 主干——parser/typeinfo/codegen/engine is 化 + 三枚举化

- [ ] 10.1 parser.at:P1 `p_kind → TokenKind`、`p_expect(p, k)` 参数化(kind 位
  字符串比较 ≈330 处、31 个函数;错误消息 kind 名经 kind_name,对齐 Rust
  `format!("{:?}")` 口径);P3 **新增 `enum Op` + `p_op()`**(镜像 Rust
  Parser::op() parser.rs:2897-2939 与 auto-val/value.rs:515-554)——infix_l/
  infix_r/prefix_power/postfix_power/op_display/binop_result 六链一步收编;P2
  Pratt/stmt 巨链 is 化(`expr_with_left` 33 处九连 break ↔ Rust match 臂合并
  parser.rs:2418-2426 等);P4 谓词函数(is_name_kind 等)消解进 is 臂。
- [ ] 10.2 typeinfo.at:Y1 `t_literal_type/t_binop_result/t_unify` is 化;
  `t_array_elem` 的"(array-type ...)"字符串形状解析 → Type 载荷化(D23/D27 收账;
  枚举载荷跨函数已实证可用)。
- [ ] 10.3 codegen.at:C1 `I.op: str → enum OpCode`(S4 子集 30 种助记符,编号
  对齐 opcode.rs);C2 `i_size`/`cg_binop_mnem`/`cg_is_assign_op` is 化(对齐
  opcode.rs:828 operand_size / codegen.rs:5856);C3 cg_expr/cg_stmt p_kind 链
  is 化(D28 行为位:下标赋值两段式游标、FN_PROLOG 占位回填等保留)。
- [ ] 10.4 engine.at:E1 `ev_run_t` 56 处 `op ==` 巨链 → `is ins.op`(二级
  eq/lt/ge 子链一并收编,对齐宿主 run_one_instruction 的 match op);**E2
  `Val{k,i,s} → enum Val{VInt(i) VStr(s) VArr(idx)}`**——8 个 ev_* 函数的
  `v.k==` 分派还原(对齐 auto-val value.rs 的 Value 枚举各 match;D34 核心目标
  收账;D35 arena 槽位语义保持;D36①② 规避写法回退;**gating:部分① 的
  H3 已根治**,否则 Val 作为 List 元素/实参高频流经构造位与 native 位不安全)。
- [ ] 10.5 每 10.x 一步一闸门:M2-M5 + 五方矩阵全绿再进下一步;E 载体(D20)与
  E.kind 字符串比较保持 dump 判据层现状不扩。

#### Phase 11:γ3 塔顶——a2r.at 自身 + f-string 全量 + 收账

- [ ] 11.1 a2r.at:A1 `ar_method_call/ar_is_mutating_method/ar_rust_ty` 等词法链
  is 化(is-on-string,匹配对象本就是方法名/类型名,对齐 Rust 377 处 match
  name.as_str());A2 p_kind 链 is 化(前置 10.1)。**顺序约束:本步只能用
  部分② 已具备发射能力的语法面**(AA2R 已能转译 is/or-臂/枚举载荷)。
- [ ] 11.2 f-string 全量:338 处 `+ "` 拼接按 Rust 参考的 format!/write! 位逐点
  还原(前置:部分② Phase 4 已让 parser.at 具备 FStr 解析,a2r.at 发射
  已就绪 D38c);转义/花括号口径对齐主 a2r(3350-3389 的 `{{`/`}}` 规则)。
- [ ] 11.3 收账:divergences.md——D11b/D14 残留/D23/D27/D28(op 载体部分)/D34/
  D36①② 风格类条目逐条标注"已还原/保留原因";新增"模式+guard 拆臂"永久条目;
  各文件头 Snapshot 五字段重写(Coverage 不变,Baseline 重锚定,DIVERGE 清单
  更新);series 复盘文档补记本系列。
- [ ] 11.4 终局验收:五方矩阵 ①③④⑤ 全绿 + G2 自举演示重跑(helloworld/fib)+
  `.expected.out` 双件零变化。

### 风险与缓解

| 风险 | 缓解 |
|---|---|
| 大改写引入行为漂移 | 文件级小步提交,每步全闸门;dump 判据层(D20 E 载体)不动,风格与判据解耦 |
| kind 枚举化后错误消息文本变化 | 错误消息统一经 kind_name 输出,Rust 侧对照口径先固定(M1 语料先验证) |
| Phase 10 Val 枚举化踩 H3 残留 | 硬 gating:H3 未根治不开工;开工后 p09h/p13b 同形态在 lib 自举回路里重验(③⑤) |
| a2r.at 自身改写与它转译自身的能力耦合(自指) | γ3 排最后;每改一段先经"主 a2r 转译(②)"验证,再切"AA2R 自举(⑤)"验证,塔层逐级确认 |
| 工作量大(83 函数/1000+ 比较位) | γ1/γ2/γ3 可拆三个子阶段分期执行;γ2 内部按文件四段各自可独立中断续作 |

### Out of Scope

- ext/impl/spec/闭包/泛型声明进入 lib(Rust 参考对应位若需要 impl 形态,先评估
  是否立项部分② 二期 W5,再动 lib;当前一对一以顶层 fn 对译为主);
- 真实 AST 取代 S-expr dump(D20 判据层重构,另立计划);
- 行为层 DIVERGE 还原(D12/D16/D18/D19/D28/D30/D31;D17 视部分①
  结果决定是否顺手回写 continue);
- aavm 特性面扩展(UI/task 等,维持 431 边界)。

### Verification

1. 每个 γ 子步:M1-M5 + 五方矩阵 ①③④⑤ 全绿;`001_smoke`/`002_hello_compile`
   的 .expected.out 零变化;
2. 终局:改写后 lib 与 Rust 参考的函数级对照表抽查(prereqs §5 台账逐文件核销,
   抽样函数逐行 diff 可解释);
3. divergences.md 收账完整性:风格类 DIVERGE 全部标注终态;
4. 性能不回退:M5 corpus 30 例总耗时与基线同数量级(engine 循环 is 化后解释
   开销对比留档)。
