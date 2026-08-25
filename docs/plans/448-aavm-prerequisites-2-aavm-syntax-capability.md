# Plan 448: aavm 新语法能力——is 解析/编译/执行 + AA2R 发射扩展(aavm-prerequisites 2/3)

> **状态**: 🟦 已立项待执行(2026-08-25;aavm-prerequisites 三件套之 ②,依赖
> Plan 447 的 H1/H4/H5 先落)
> **来源**: [idiom-upgrade-prereqs.md](../specs/aavm/idiom-upgrade-prereqs.md) §4/§5/§6;
> Plan 434 遗留"AA2R 扩覆盖(golden 全组 + is-match)"的承接。
> **定位**: 让 aavm(auto/lib 七文件)**自身获得 is-match 与枚举载荷的完整处理
> 能力**——前端解析(dump 判据层)、类型面、编译执行、以及 AA2R(a2r.at)的
> Rust 发射。这是 Plan 449 风格改写的**自举塔硬前提**:lib 用什么语法,
> a2r.at 必须先能发射什么。
> **纪律**: 本计划全部新代码仍用**现有最简子集**书写(if 链/无 is/无载荷枚举);
> 风格化属于 Plan 449。塔式爬升,顺序不可倒置。
> **关联**: Plan 447(前置:H1 or-臂宿主正确性、H4/H5 a2r 发射正确性——golden
> 文本的对齐基准)、Plan 449(后继)。
> **判据文档**: [m2-ast-dump-format.md](../specs/aavm/m2-ast-dump-format.md)、
> [divergence-rules.md](../specs/aavm/divergence-rules.md) §4 纯 Rust 模式编码规范。

## 1. 目标与缺口

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
直接传参全部 ✅;or-臂待 H1)——**本计划做的是把这套能力在 Auto 写的编译器里
复刻一遍**,顺带成为这些形态的第二次大规模实战验证。

## 2. 任务

### Phase A:parser.at 前端(A 段)

- [ ] A1 `parse_is` 直译(镜像宿主 parser.rs:7511-7644 的 parse_is/parse_is_branch
  决策:EqBranch 单模式/多模式 `|`、IfBranch 卫语句、ElseBranch;模式形态:字面量
  /字符/`Cover(Tag)` 枚举路径含 `Name(bound)` 绑定与 `_`);从
  `is_unsupported_stmt_kind` 摘除 "Is"。
- [ ] A2 dump 形态对齐:is 语句/臂/模式的 S-expr 与 Rust 参考 m2 dump 逐字符一致
  (对照 m2-ast-dump-format.md;Rust 侧 dump 以 live 对比为唯一依据,发现宿主
  dump 漂移按 D18 quirk 照抄规则处理)。
- [ ] A3 corpus_m2 新增 is 语料组(p15/p16 后续编号):is-on-string、is-on-char、
  or-臂、枚举载荷解构、卫语句、else、嵌套块体、值语义位;M2 全绿。

### Phase B:typeinfo.at(B 段)

- [ ] B1 `t_stmt_walk` 增 is 语句(臂体走查;绑定变量按枚举载荷表入局部作用域,
  镜像宿主 infer 对 Uncover 的处理);`t_infer_expr` 若含 is 值语义位同口径。
- [ ] B2 corpus_m3 扩 `.type` 查询语料(臂内绑定类型打印);M3 全绿。

### Phase C:codegen.at + engine.at(C 段)

- [ ] C1 codegen.at 发射(镜像宿主 codegen.rs:3347-3735 决策,不照抄实现):
  `_is_target` 暂存局部;EqBranch——字面量/字符 → EQ;标量枚举 → 判别值 EQ;
  载荷枚举 → IS_VARIANT + GET_GENERIC_FIELD 逐字段绑定;or-臂 → 逐模式 EQ +
  短路或(宿主 H1 修后口径);IfBranch → 条件 + 条件跳转;臂尾 JMP 汇出(占位回填)。
- [ ] C2 engine.at 执行:IS_VARIANT / GET_GENERIC_FIELD 两指令(aavm 指令集已含
  声明,opcode.at;执行语义镜像宿主 engine.rs:3988-4130 但走 v2 自身值栈/arena
  模型 D29/D35);注意 v2 布尔 0/1 承载与负值编码(D30)在 EQ 上的口径。
- [ ] C3 corpus_m4(反汇编)/corpus_m5(行为)扩 is 语料;M4/M5 全绿——这是
  aavm 对 is 的**端到端**判据。

### Phase D:AA2R(a2r.at)发射(D 段;与 A-C 可并行,依赖 ① 的 H4/H5)

- [ ] D1 **W1 is-match 发射**:`ar_stmt`(1690)加 `Is` 分支 → 新 `ar_is`:
  scrutinee 文本(任一臂 Str 字面量 → `.as_str()`,镜像主 a2r rust.rs:12781-12815);
  臂体三形态(单语句内联/多语句块/空块,镜像 write_match_arm_body 12631);
  `else ->` → `_ =>`;模式解析按 D39 token 游标直走(不能复用 ar_expr 的条件
  表达式语义)。
- [ ] D2 **W2 or-臂**:模式组 `|` 分隔发射(`p1 | p2`)。
- [ ] D3 **W3 枚举载荷**:`ArEnum`(62-65)增每变体载荷类型表;
  `ar_prescan_enum`(673-719)解析 `Name(T)`/`Name{T1,T2}`/`Name{f T}`;
  `ar_emit_enum2`(2075-2132)按形态发元组/结构/单元变体 + **derive 三分派**
  (float/Map/嵌套枚举安全性,对齐主 a2r 14477-14543;现无条件全 derive,列入
  D40 差异清单修正);构造点 `ar_call_tail`/`ar_method_call`(1528/1270)加
  枚举变体构造判定 + str 载荷 `.to_string()`;**绑定名 `ar_vpush` 进作用域并查
  载荷表登记类型**(不复制主 a2r 的 DEBT-113 缺口)。
- [ ] D4 文件头 Snapshot 更新:Missing 清单摘除 is-match/枚举载荷;**f-string
  实际已支持(D38c),清单同步纠正**;D40 清单补 derive 三分派项。
- [ ] D5 AA2R golden:06_pattern_matching 组(is-match/or-臂/枚举载荷)落盘,
  文本对齐主 a2r(基线 = 已修 H4/H5 的 master);probe 文件(p01/p02b/p04/p05/
  p12)转译冒烟 + rustc 编译通过。

## 3. 风险与缓解

| 风险 | 缓解 |
|---|---|
| is 的 S-expr dump 与 Rust 参考存在格式历史 quirk | A2 以 live 对比为唯一依据;quirk 按 D18 模式登记照抄,不擅自"修好" |
| C 段把宿主 codegen 的复制粘贴双副本问题带进 v2 | C1 只镜像**决策**,实现走 v2 单遍步行 + 游标快照方法论(432 先例);语句/表达式两入口写同一辅助函数族 |
| AA2R golden 对齐基线漂移(H4/H5 未合入) | D5 明确 gating 在 Plan 447 Phase 2 合入之后 |
| D39 token 直走在模式位与表达式位歧义 | 模式解析独立函数,不碰 ar_expr;语料带足 `a - 1 ->`、`'x' ->`、`A.B(c) ->` 歧义形 |

## 4. Out of Scope

- lib 自身的风格化改写(→ Plan 449;本计划 lib 语法面不变);
- spec/ext/impl/use/dep/闭包/泛型声明的 AA2R 发射(§5 二期,只有 γ 计划用到
  impl 时再立项);r2a/多目标/post_process 正则族(Plan 434 永久 Out of Scope);
- 宿主缺陷修复(→ Plan 447)。

## 5. Verification

1. M1-M5 闸门全绿(含 A3/B2/C3 新语料;旧语料零变化);
2. AA2R golden 06 组与主 a2r 产物逐字符一致(D40 白名单外零差异);
3. 五方矩阵 ①-⑤ 全绿(lib 未变,③④⑤ 基线不动即为回归通过);
4. probe 转译冒烟:p01/p02b/p04/p05/p12 经 AA2R 转译后 rustc 零错;
5. divergences.md:D38 系列补记 is 能力位;corpus.md 登记新语料分层归属。
