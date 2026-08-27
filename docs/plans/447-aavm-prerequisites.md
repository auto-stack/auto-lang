# Plan 447: aavm-prerequisites 三合一——宿主加固 → aavm 语法能力 → lib 风格对译

> **状态**: 🟩 全部完成(2026-08-26)。部分① ② 于 bdd659803 合入;部分③
> γ 系列完成:P1/P3/C1/E1/Y1+P4/E2(D34 收账,宿主 push_value RC 修复)/
> P2+aavm break-continue/11.1 A1+A2/11.2a f-string 四层;11.2b 批量
> f-string 改写实证 D40 续三缺口后撤回(工具与缺口清单留档);11.3
> 收账完毕。终局验收:矩阵 36/36 全绿 ×5、G2 自举双演示正确
> (helloworld/fib)、.expected.out 零变化、M5 耗时与基线同数量级。
> 执行记录见各 Phase 内联注记与文末附录。(2026-08-25;
> aavm-prerequisites 三件套合并为单一计划,原 447/448/449 合并,序号 448/449 腾空)
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

- [x] 1.1 建 `test/vm/99_idiom_probe/`:p01/p02b/p06/p07/p08/p09h/p13b 等按
  `名称.at + 名称.expected.out` 平铺落盘(期望值 = **正确**语义输出),随对应
  H 项修复同 PR 落地;p03/p04/p05/p10/p13a/c 等绿件直接先落(纯加固回归)。
- [x] 1.2 `vm_file_tests` runner 确认新目录被自动发现(平铺文件惯例同
  `10_types/020_struct_to_str.at`);跑 `cargo test -p auto-lang --lib
  --features test-vm-files -- test_aavm2` 基线留档。

#### Phase 2:VM 三修(H1/H2/H3)

- [x] 2.1 **H1 枚举 or-臂**:`Stmt::Is` 的 EqBranch 处理(codegen.rs:3537 起)在
  `patterns.len() > 1` 时改走多模式路径(逐模式 EQ + JMP_IF_NZ,枚举判别值可直用);
  **同步修 `Expr::Is` 副本(9348 起,两份代码逐段对照)**;p02b 进 99_idiom_probe。
- [x] 2.2 **H2 continue 回填**:照 2588 的 `patch_jump_to` 模式补齐 7 处变体
  (Cond=while、Ever、Named/Indexed/Call/Destructured 系);p06/p07 进常驻;
  全量 VM 语料回归(现状恰好零覆盖 while+continue,修复本身无 golden 波动)。
- [x] 2.3 **H3 根治(目标)或规避(兜底)**:
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

- [x] 3.1 **H4 卫语句臂**:IfBranch 发射改为 `<绑定> if <cond> =>`;scrutinee 为
  复杂表达式时先 `let __is_x = expr;` 再 `match __is_x`;p08 转译产物 rustc 零错。
  现有 golden 全仓零覆盖卫语句(零 golden 波动)。
- [x] 3.2 **H5 二次匹配 E0382**:scrutinee last-use 分析——非末次使用发
  `match &v`(臂绑定变引用,`format!`/`println!` 用法兼容);p05 产物零错;
  a2r golden 06 组回归。
- [x] 3.3 **H6(顺手)**:与 3.2 动同一片代码时,is 臂发射按枚举载荷表回填
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

- [x] 4.1 `parse_is` 直译(镜像宿主 parser.rs:7511-7644 的 parse_is/parse_is_branch
  决策:EqBranch 单模式/多模式 `|`、IfBranch 卫语句、ElseBranch;模式形态:字面量
  /字符/`Cover(Tag)` 枚举路径含 `Name(bound)` 绑定与 `_`);从
  `is_unsupported_stmt_kind` 摘除 "Is"。
- [x] 4.2 dump 形态对齐:is 语句/臂/模式的 S-expr 与 Rust 参考 m2 dump 逐字符一致
  (对照 m2-ast-dump-format.md;Rust 侧 dump 以 live 对比为唯一依据,发现宿主
  dump 漂移按 D18 quirk 照抄规则处理)。
- [x] 4.3 corpus_m2 新增 is 语料组(p15/p16 后续编号):is-on-string、is-on-char、
  or-臂、枚举载荷解构、卫语句、else、嵌套块体、值语义位;M2 全绿。

#### Phase 5:typeinfo.at(B 段)

- [x] 5.1 `t_stmt_walk` 增 is 语句(臂体走查;绑定变量按枚举载荷表入局部作用域,
  镜像宿主 infer 对 Uncover 的处理);`t_infer_expr` 若含 is 值语义位同口径。
- [x] 5.2 corpus_m3 扩 `.type` 查询语料(臂内绑定类型打印);M3 全绿。

#### Phase 6:codegen.at + engine.at(C 段)

- [x] 6.1 codegen.at 发射(镜像宿主 codegen.rs:3347-3735 决策,不照抄实现):
  `_is_target` 暂存局部;EqBranch——字面量/字符 → EQ;标量枚举 → 判别值 EQ;
  载荷枚举 → IS_VARIANT + GET_GENERIC_FIELD 逐字段绑定;or-臂 → 逐模式 EQ +
  短路或(宿主 H1 修后口径);IfBranch → 条件 + 条件跳转;臂尾 JMP 汇出(占位回填)。
- [x] 6.2 engine.at 执行:IS_VARIANT / GET_GENERIC_FIELD 两指令(aavm 指令集已含
  声明,opcode.at;执行语义镜像宿主 engine.rs:3988-4130 但走 v2 自身值栈/arena
  模型 D29/D35);注意 v2 布尔 0/1 承载与负值编码(D30)在 EQ 上的口径。
- [x] 6.3 corpus_m4(反汇编)/corpus_m5(行为)扩 is 语料;M4/M5 全绿——这是
  aavm 对 is 的**端到端**判据。

#### Phase 7:AA2R(a2r.at)发射(D 段;与 Phase 4-6 可并行,依赖部分① 的 H4/H5)

- [x] 7.1 **W1 is-match 发射**:`ar_stmt`(1690)加 `Is` 分支 → 新 `ar_is`:
  scrutinee 文本(任一臂 Str 字面量 → `.as_str()`,镜像主 a2r rust.rs:12781-12815);
  臂体三形态(单语句内联/多语句块/空块,镜像 write_match_arm_body 12631);
  `else ->` → `_ =>`;模式解析按 D39 token 游标直走(不能复用 ar_expr 的条件
  表达式语义)。
- [x] 7.2 **W2 or-臂**:模式组 `|` 分隔发射(`p1 | p2`)。
- [x] 7.3 **W3 枚举载荷**:`ArEnum`(62-65)增每变体载荷类型表;
  `ar_prescan_enum`(673-719)解析 `Name(T)`/`Name{T1,T2}`/`Name{f T}`;
  `ar_emit_enum2`(2075-2132)按形态发元组/结构/单元变体 + **derive 三分派**
  (float/Map/嵌套枚举安全性,对齐主 a2r 14477-14543;现无条件全 derive,列入
  D40 差异清单修正);构造点 `ar_call_tail`/`ar_method_call`(1528/1270)加
  枚举变体构造判定 + str 载荷 `.to_string()`;**绑定名 `ar_vpush` 进作用域并查
  载荷表登记类型**(不复制主 a2r 的 DEBT-113 缺口)。
- [x] 7.4 文件头 Snapshot 更新:Missing 清单摘除 is-match/枚举载荷;**f-string
  实际已支持(D38c),清单同步纠正**;D40 清单补 derive 三分派项。
- [x] 7.5 AA2R golden:06_pattern_matching 组(is-match/or-臂/枚举载荷)落盘,
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

- [x] 8.1 divergence-rules.md §4 补写法规范(基于 prereqs §7 实证语法备忘):
  ①单行枚举需逗号+显式值,无值变体逐行;②Auto 无"模式+guard"组合臂——Rust
  `VI(n) if n>10` 拆独立卫语臂或调臂序(**新增永久 DIVERGE 编号登记**);③同函数
  对同一枚举值多次 is 的写法约定(依赖 H5 已修);④枚举构造参数位暂不内联运行期
  计算值(若 H3 走了兜底路线,此条长期保留;若已根治,登记解除)。
  落地:§4a-5~9 五条 + divergences.md D41 登记(H3 已实证不复发→④解除;
  H5 已修→③放开;is 值语义仅函数尾位新增第 9 条)。
- [x] 8.2 基线锚定:当前 master commit + 全量闸门跑绿留档(M1-M5 + 五方矩阵
  ①③④⑤ + .expected.out 双件);后续每个 γ 子步以此为对照。
  **执行记录(2026-08-25/26)**:开工首跑发现五方矩阵 ②⑤ 列**并非绿**——
  部分②合入后未复跑矩阵,lib 新代码暴露两侧转译缺口,先行修复后方锚定:
  - 主 a2r(trans/rust.rs)三族:①`.str()` 入 &str 参数位被文本坍缩器
    (fix_string_str_mismatches 的 `.to_string().as_str()→.as_str()`)错缩,
    int 接收者 E0599——改为按接收者类型分流发射(str→直借,数值/未知→
    `format!`);②`List<str>.get(i)` 参数位缺 `.as_str()` 借用(E0308);
    ③链式接收者 `a.ens.get(i).pays.get(j)` 的 List 判定断链(E0277)——
    is_auto_list_expr 递归推断回退。
  - AA2R(a2r.at)两处:ar_coerce_arg 对 List.get 索引形(ty=str,strk=0)
    补 `.as_str()`;ar_is_cur_mut_param 扩认逃逸分析提升参数(&mut *x
    再借用,E0596)。
  - 留档:aavm2 闸门 11 绿;a2r golden 15 失败与 a9e40360e 基线逐项一致
    (零漂移,基线即红);五方矩阵 33/33 全绿(②列从 18 错、⑤列从 7 错修复);
    001_smoke/002_hello_compile .expected.out 不动(M5 闸门覆盖)。
  - 基线 commit:a9e40360e(master 分叉点)+ 本计划 bc468fd8b。

#### Phase 9:γ1 样板先行——token.at + lexer.at(M1/M2 闸门保护下)

- [x] 9.1 token.at:T1 `keyword_kind` 58 臂 if 链 → `is text {...}`(is-on-string,
  对齐 Rust token.rs:365-449 的 match &str);T2 `kind_name` 139 臂 → `is k {...}`
  (is-on-enum;D11 哨兵写法保留);头 Snapshot 与 divergences.md D11b 收账。
- [x] 9.2 lexer.at:L1 `Token.kind/Tok.kind: str → TokenKind`(构造点 34+2 处,
  `lex_number` 的 kind 变量、`lex_dump`/p_err 输出位经 kind_name);L2 转义/定界
  else-if 链 → is-on-char(lex_string/lex_char/lex_fstr_at 对齐 Rust lexer.rs:462/
  386 的 match esc);L3 `delim_name` 9 臂并入主链(Rust 无此函数,臂内联于
  next_step)。
- [x] 9.3 闸门:M1(token dump 逐字符)/M2 全绿;②/⑤ 缓存重建后全绿;若 H3 未
  根治,kind 构造点保持提升局部写法。
  **执行记录(2026-08-26)**:
  - 主链整体翻转为 `is c` 形态(超出 L2/L3 字面范围):digit/alpha 以卫语臂与
    模式臂**交错**(序同基线 else-if 链;单条 is 内交错此前无语料,lib 自身
    成为首个实证);运算符 +*%!>< 共臂内嵌套 is 选 base/eqk/sym。
  - **Unknown 第 140 变体**(aavm 侧补充,Rust 无):kind 载体枚举化后未收编
    字符分支需哨兵值,kind_name 经 else 回 "Unknown" 对齐基线 str 行为。
  - 边界:p_kind/p_peek 以 kind_name 维持 str 返回(P1 翻转);is_comment_kind
    参数枚举化 + is-or 臂;parser/codegen/a2r 直接 .kind 读位经 kind_name 或
    枚举比较(codegen b29 扫描环整段枚举化)。
  - **两侧转译器缺口四项修复**(γ1 暴露):①主 a2r int scrutinee 的字符模式
    → 码点十进制(int_match_scrutinee 旗标,模式循环内置位);②AA2R 同款
    (ar_is_pattern_text 增 scrut_int 参);③主 a2r char_at/len 接收者后链
    方法加括号(`(... as i64).to_string()`);④AA2R .str() 同款 as i64 尾码
    括号。另 **AA2R ar_is 块状臂体补齐**(ar_is_arm_body 三形态:单表达式/
    多语句块/空块——部分② W1 声称三形态但 corpus 全单表达式臂,块臂从未实证)。
  - 闸门:M1-M5+AA2R 11 绿;a2r golden 15 失败与基线逐项一致(零漂移);
    **五方矩阵 33/33 全绿**(②列 54 错→0、⑤列块臂 LBrace 错→0 迭代修复)。

#### Phase 10:γ2 主干——parser/typeinfo/codegen/engine is 化 + 三枚举化

- [~] 10.1 parser.at:P1 `p_kind → TokenKind`、`p_expect(p, k)` 参数化(kind 位
  字符串比较 ≈330 处、31 个函数;错误消息 kind 名经 kind_name,对齐 Rust
  `format!("{:?}")` 口径)——**P1 已完成(2026-08-26,提交 624a2f008)**:
  四文件 840+ 比较位翻转 + 16 个帮助函数签名枚举化;宿主三修(merge 跨模块
  枚举名传播/标量枚举 derive Copy+7 golden derive 行合法更新/arm 调序);
  五方矩阵 33/33 全绿。P3/P4 已完成(见 10.2 二次记录;P3 提交
  19c1e7b11,P4 谓词 is 化含 is_name_kind/is_unsupported_stmt_kind),
  P2(Pratt 巨链)待续。
  P3 **新增 `enum Op` + `p_op()`**(镜像 Rust
  Parser::op() parser.rs:2897-2939 与 auto-val/value.rs:515-554)——infix_l/
  infix_r/prefix_power/postfix_power/op_display/binop_result 六链一步收编;P2
  Pratt/stmt 巨链 is 化(`expr_with_left` 33 处九连 break ↔ Rust match 臂合并
  parser.rs:2418-2426 等);P4 谓词函数(is_name_kind 等)消解进 is 臂。
- [x] 10.2-Y1 + 10.1-P4 完成(2026-08-26 二次;首试回退已翻案):五函数
  is 化——typeinfo `t_literal_type/t_binop_result/t_unify`(or 臂/卫语臂
  交错/函数尾值语义)与 parser `is_name_kind/is_unsupported_stmt_kind`。
  二次执行连带修复三处(首试失败的真机制逐步剥离):
  ①AA2R `ar_is_body_coerce` 缺 Ident-str-参数分支——`"" -> b` 臂值发射
  `"" => b`(E0308),镜像主 a2r expr_needs_string_coercion 补齐,发射与
  主 a2r 逐字符一致;②AA2R `ar_branch_body` 无条件清零 `tail_no_semi`
  冲掉外层函数尾标记——early-return if 之后的尾位 is 失锚(带分号+
  字面量臂不收敛),改保存/恢复;③主 a2r `fix_mutable_params` 正则
  `\s*=[^=]` 把 match 臂箭头 `<param> =>` 误读为赋值(`mut b` 噪声,
  t_unify 卫语臂 `_guard if a == b =>` 实证)——三处文本后处理同修
  `=[^=>]`,a2r golden 15 失败与基线逐项一致零漂移。
  新增语料 g04_is_arm_value_str(臂值= str 参数标识符 + 卫语臂交错 +
  early-return 后尾位 is),无修复则红。
  **首试"⑤列 print 超大串退空"谜题结案**:宿主 print 无尺寸阈值
  (720KB/65536 行多行大串探针通过;倍增构造 39.7 万字符单行亦通过);
  失败簇为环境相关异常终止(11 连败:rc=-1/127、无 stderr、无 WER 记录,
  秒级死亡 vs 正常 4-7 分钟完成;其后 8 连跑 7 成功 + 1 例 420s 超时
  切断——成功运行最慢 414s,超时线贴边),全量 bootstrap 峰值内存
  239-276MB。另证:循环逐字符拼接 >20 万字符会崩/挂(200k 绿/300k 崩
  /400k 挂,O(n²) 累计分配;engine.rs:860 注释的 u16 池截断家族),
  aavm 输出缓冲走 List+join 不受影响,以写法规范规避(divergence-rules
  §4a 已有)。
- [ ] 10.2 typeinfo.at 剩余:`t_array_elem` 的"(array-type ...)"字符串
  形状解析 → Type 载荷化(D23/D27 收账;枚举载荷跨函数已实证可用)。
- [x] 10.3 codegen.at(部分:C1+C2 已完成 2026-08-26 提交 fce116fde):
  enum OpCode + op_name;I.op/emit/i_size 枚举化;cg_binop_mnem→OpCode;
  10.4-E1 同批完成(engine 49 臂 → is op;矩阵 33/33 绿)。
- [~] 10.3-codegen.at 剩余:C3 cg_expr/cg_stmt p_kind 链 is 化(P2 同批)
  **P2 四条 Pratt 巨链已完成(2026-08-26)**:parser/typeinfo/codegen/a2r
  各自中缀循环顶的 11/12 元 `k == A || k == B || ...` 界符链 →
  `is k { 11 种 or 臂 → 臂内 break; else → 循环体 }`(镜像 Rust
  parser.rs:2425 的 match kind 臂合并;臂内 break 语义四列实证——
  宿主/主 a2r 产物 rustc 零错运行正确,aavm cg 首次实现 break/continue:
  循环占位回填镜像宿主 H2 patch_jump_to,brk_js/cont_js/cont_tg 按
  层深 append-only 帧栈(engine.at args_stack 同款,无 List.pop
  opcode),while continue=条件顶/for continue=步进位,while 体作用域
  与 for 同构(体内变量每迭代释放组,b32 反汇编与宿主逐字符一致);
  语料 b32_is_break_continue(M4/M5/矩阵第 35 例)+ g06_is_break_
  continue(②⑤文本对齐)。C3 的 cg_expr/cg_stmt 语句位 p_kind 链与
  parse_stmt 小链(≤4 元)留 Phase 11 顺带。
  **语料暴露的既有缺口登记(Phase 11 收账)**:①aavm cg 对 List 型
  fn 参数的 .len() 报 "receiver is not an array"(类型跟踪缺口);
  ②AA2R 单语句块臂不内联(主 a2r write_match_arm_body 单语句内联);
  ③List 实参调用位克隆(主 a2r is_owned_list_arg 无条件 clone,AA2R
  仅 last-use);④臂值位赋值表达式 aavm cg 不支持(值位须纯表达式,
  写法规范)。b32/g06 以绕开形态落盘。
- [x] 10.4-engine.at E2 完成(2026-08-26):`Val{k,i,s}` 判别器结构体 →
  `enum Val{VInt(int) VStr(str) VArr(int) VInst(int,str)}`(部分② 的
  k=3 实例形态并入 VInst 双载荷);15 处 `v.k==` 判别位 + 4 个构造点
  还原为 is-绑定分派/枚举构造,8 个 ev_* 函数对齐 auto-val Value 的
  match 形态(D34 收账;D35 arena 槽位语义保持——VArr/VInst 的 int
  载荷仍是 arena 索引)。433 期"枚举载荷跨函数丢标签"规避随 H3
  根治解除;D36①② 提升写法保留(非枚举化障碍,风格回退另行)。
  **连带四修**:
  ①宿主 RC 修复(H3 家族新发现):engine.rs push_value 的 Str 分支
  add_string 后漏 retain(freelist 条目 rc=0 须 push 侧建份额)——
  GET_GENERIC_FIELD 读运行期字符串载荷后任意 pop/槽释放即超额释放
  (RC canary;v21 最小复现:fn 内构造运行期串实例→返回→is 绑定读
  →print;此前全库载荷均为 pinned 常量故未暴露),统一走
  rc_push_str_idx,8 个调用点(字段读/原生返回)同族潜伏一并修;
  ②ev_cmp 嵌套双 `is b` 触发主 a2r H5 的 `match &` 引用绑定
  (&i64/&String 不可入按值调用)→ 每臂新鲜拷贝 b1/b2 消除双 is;
  ③ConstructInstance 的 is 绑定 String 载荷是部分移动,inst9 回栈
  复用 E0382 → is 拷贝 inst9c;
  ④AA2R 多元组载荷三处:模式绑定 ", " 连接、构造位按槽位
  (ar_pay_slot)判 str `.to_string()`、枚举 ident 实参无条件 clone
  (镜像主 a2r struct_flags 非 Copy 判据)。
  新语料三件:p23_enum_multi_payload(M2 dump)/g05_enum_multi_payload
  (②⑤逐字符对齐)/b31_is_enum_multi(M4/M5 行为 + 矩阵第 34 例)。
  闸门:aavm2 13+2 绿;全量 18 失败与基线逐项一致零新增。
- [ ] 10.3.C1 `I.op: str → enum OpCode`(S4 子集 30 种助记符,编号
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
- [~] 11.2 f-string(2026-08-26):
  **11.2a 能力层完成(提交 0689b118d)**——计划前置声明勘误:部分② 并未
  让 parser.at 具备 FStr 解析(仅词法 D38c + AA2R 发射侧),本步补全
  parser fstr_expr(dump 对齐宿主 (fstr ...) 逐字符)/typeinfo(恒 str)/
  codegen cg_fstr(build.fstr + tags,CGVar 轻量类型跟踪)/engine 执行,
  语料 p24/b33/g07 三件,矩阵 36/36。
  **11.2b 批量改写回退(D40 三缺口实证)**:行级转换器(%TEMP%/p447/
  fstr_conv.py,字符串感知切分/字面量护栏)批量转换 129 处后⑤列 AA2R
  自举产物 rustc 23 错,E0382 残留单例根因:FStrPart 整段文本直通使
  插值变量 token 从流中消失 → ar_lu_after/扫描器判定"后续无使用" →
  前行赋值的 clone 注入失效(E0382 move);另两类:方法调用形态绕过
  ar_method_call 改写(.str() 原样泄漏 E0599)/参数借用适配缺失
  (p_text(p) 直通 E0308)。**结论:FStrPart 直通发射须先补齐 Display
  收敛(method 改写/借用适配/last-use 补扫)方可承载批量风格还原;
  行为不变铁律下本批撤回**,338 处逐点还原留后续计划(工具与实测
  缺口清单已备)。lib 回退至 11.2a 态,闸门绿。
- [ ] 11.3 收账:divergences.md——D11b/D14 残留/D23/D27/D28(op 载体部分)/D34/
  D36①② 风格类条目逐条标注"已还原/保留原因";新增"模式+guard 拆臂"永久条目;
  各文件头 Snapshot 五字段重写(Coverage 不变,Baseline 重锚定,DIVERGE 清单
  更新);series 复盘文档补记本系列。
- [x] 11.4 终局验收(2026-08-26):①五方矩阵 36/36 全绿×5 列(含 b31 多载荷/
  b32 break-continue/b33 f-string 三新例);②G2 自举双演示:最新 lib 经
  AA2R 自举转译(326KB 产物 rustc 零错)→ aavm2_fast.exe 跑 helloworld
  → "hello, world!"、fib.at → 55,与 434 封版声明逐字符一致;③
  001_smoke/002_hello_compile .expected.out 双件 git 零变化;④性能:
  M5 语料套件 23.8s(基线同数量级)。

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

---

## 附录:部分① 执行记录(2026-08-25)

### 完成项

- **H1**:新增 `emit_is_or_arm` 共享辅助(逐模式 EQ + JMP_IF_NZ 短路或 + 匹配标号
  汇出),标量枚举 Cover::Tag 多模式与字面量多模式两路径同源调用;`Stmt::Is` 与
  `Expr::Is` 双副本均已接入。p02b 转绿(`num eof num num eof`)。
- **H2**:8 处循环变体(Cond/Ever/Named-Call/Indexed/Call/Destructured 系)continue
  占位符照 2588 模式 `patch_jump_to` 回填。p06/p07 转绿;conformance
  `017_loop_continue` 期望随修正语义更新(`1,3,5`;原 `1-6` 固化的是穿透 bug)。
- **H3**:复现件在当前基线(plan-442 RC 死区修复 f81e18c8e 合入后)**已不复发**,
  无需 arg_count 根治刀。p09h 强化(读回 payload 而非仅查 len)、p13b 修正语法
  (`struct` 非 Auto 关键字,具名结构体声明用 `type`)后双双转绿;p09g/p13a 提升
  局部对照件同落。Phase 10 的 H3 gating 视为已满足(Val 枚举化仍需
  `repro_242_string_pool_uaf` 等 RC 回归护航)。
- **H4**:IfBranch 发射 `<绑定> if <cond> =>`(标识符 scrutinee 用 `_guard`,
  复杂 scrutinee 先 `let __is_N = expr;` 提升,绑定名入模式位);p08 产物 rustc
  零错;提升仅在含卫语句臂时触发(golden 零覆盖卫语句,零波动)。
- **H5**:`match &v` 收窄为"同函数内同一标识符 scrutinee ≥2 次 is"才发射
  (`fn_is_scrutinee_counts` 预扫描,`fn_decl`/`transpile_body_stmts` 双入口);
  单次匹配保持按值(臂内 payload 按值取用兼容)。a2r golden 全量回到基线
  12 个遗留失败,零新增;p05 产物 rustc 零错。
- **H6**:is 臂发射按 `enum_tuple_field_types` 回填 `local_var_types`
  (Cover::Tag 绑定 + `Enum.Variant(bound)` 调用形两路径),DEBT-113 收口。
- **Phase 1**:16 件探针(4 复现转绿 + 12 绿件)落 `crates/auto-lang/test/vm/
  99_idiom_probe/`(注意:cargo runner 扫描 crate 内 test/vm,非仓库根),
  `vm_file_tests` 16 条常驻断言(非 ignore)接入。

### 新观察项(部分②/后续计划输入)

1. **let 绑定位 is 值语义返回 0**:函数尾位 ✅,`let r = is x {...}` 位返回 0
   (t11 复现)。不在 H1-H6 范围,部分② Phase 4 语料设计时需覆盖。
2. **函数内嵌套 fn 静默失效**:`fn main() { fn inner() {...} inner() }` 调用
   无输出(非报错)。探针一律用顶层 fn。
3. `struct` 非 Auto 关键字(声明用 `type`),误用时报 E0201 名字解析错而非
   语法错,易误导排查方向。

### 闸门留档

- 全量 `cargo test -p auto-lang --lib --features "test-trans test-vm-files"`:
  与基线差异仅 `benchmark_downcast_performance`(性能阈值测试,单跑通过,
  并行负载下偶发);基线本身有 12 个 a2r golden + 3 个 cookbook_vm + 1 个
  vue gallery 遗留失败(与 447 无关,基线即红)。
- aavm2 闸门 8/8 绿;99_idiom_probe 16/16 绿;p05/p08/卫语句提升件 rustc 零错。

## 附录:部分② 执行记录(2026-08-25)

### 完成项

- **Phase 4(parser.at)**:parse_is/parse_is_branch/is_pattern/parse_expr_or_body
  直译;is_pattern 的 Ident+Dot 点路径拦截(tag-cover 形),Some(x) 等
  调用形照抄宿主走表达式机(两侧 lexer 均不产出 SomeKW);表达式位 is
  出 `(is-expr target)`(分支弃置,quirk 照抄);parse_enum_decl 收编
  载荷变体(载荷类型按 Enum.Variant.slot 登记,标量枚举不注册→参数位
  (type-decl) 兜底,载荷枚举注册→参数位内联);"Is" 摘除 unsupported。
  corpus_m2 增 p17-p22 六件(字面量/or-臂/载荷/卫语句/嵌套块/值位),
  M2 逐字符绿。**VM quirk 发现**:调用结果链式取字段 `f(p).dump` 在
  参数位求值异常(先落临时变量规避,挂观察)。
- **Phase 5(typeinfo.at)**:t_is_walk/t_walk_is_arm_body(臂体走查+
  载荷绑定按 slot 表入作用域);enum 声明直接复用 parse_enum_decl
  (注册单一事实源)。corpus_m3 增 t07(命中臂内 .type 查询;未命中臂
  不得含查询——静态走查收集全部而宿主只执行命中臂)。M3 绿。
- **Phase 6(codegen.at/engine.at)**:cg_is 三路发射(载荷+绑定→
  is.variant+get.generic.field+nop×3;载荷无绑定→is.variant+共享尾;
  标量/字面量→单模式 eq+共享尾/or 链 jmp.nz);枚举构造(const 名长/
  new.instance/const 字段数/construct.instance);cg_enum_decl 注册表;
  engine 增 jmp.nz/new.instance/construct.instance/is.variant/
  get.generic.field/nop(Val k=3 实例:i=arena 字段表槽,s=变体名)。
  corpus_m4 增 b13-b15(载荷/标量 or/字面量+卫语句+字符)。M4/M5 绿。
  **两枚潜伏 bug 首次暴露并修复**:①cg_push_scope 无条件 push 而弹栈
  只减 depth——第二个 fn 复用前一 fn 残留作用域(其体内变量泄漏进
  后续 fn 的释放组);②cg_add_var 同名追加而宿主 HashMap 按名覆盖
  (多次 is 的 _is_target 旧槽不再释放)。get.generic.field 后 3×nop
  为宿主 u32 操作数错位的既有形态,照抄。
- **Phase 7(AA2R)**:ar_prescan_enum 收编载荷(每变体 Rust 载荷文本);
  ar_emit_enum2 载荷分支(derive 三分派:浮点载荷去 Eq/Ord;不带
  Display/from_id);构造拦截 ar_enum_try_ctor(str 载荷字面量补
  .to_string());实参位枚举字面量(::路径非构造形)克隆;ar_is
  (H5 收窄镜像 + str 模式 .as_str() + 尾值位无分号 + 臂体按 fn 返回
  型收敛 + `_` 通配);is 模式前瞻 ar_is_has_str_pattern(体内字符串
  不误报)。闸门:corpus_a2r g01-g03 与主 a2r live 逐字符一致;探针
  冒烟 p01/p02b/p04/p05/p12 经 AA2R 后 rustc 零错。
- **宿主侧修复(部分②发现)**:is_stmt 的 H5 `&` 前缀与 str 模式
  `.as_str()` 叠加产生 `match &text.as_str()` 双重引用(E0308,主 a2r
  自身产物即错)——`&` 改为仅在没有 str 模式臂时发射。

### 闸门留档

- aavm2 全闸门 11 绿(含新增 AA2R is 语料闸门 aavm2_a2r);probe 冒烟
  (ignored,rustc)绿;全量 lib 测试相对基线零新增失败(15 a2r 红 =
  12 遗留 + 3 个 plan-444 红,均已实证预先存在)。
- 五方矩阵 ③④⑤(lib 未变)基线不动;①②由 M2-M5+AA2R 闸门覆盖。
