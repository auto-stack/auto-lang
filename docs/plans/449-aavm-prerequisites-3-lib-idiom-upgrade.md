# Plan 449: lib 风格升级——从类 C 最简子集到与 Rust 一对一对译(aavm-prerequisites 3/3)

> **状态**: 🟦 已立项待执行(2026-08-25;aavm-prerequisites 三件套之 ③,硬依赖
> Plan 447 全部 + Plan 448 全部)
> **来源**: [idiom-upgrade-prereqs.md](../specs/aavm/idiom-upgrade-prereqs.md) §5/§6/§7。
> **定位**: 用户既定长期路线——aavm 长期锚定"与 Rust 参考实现函数级/块级一对一
> 对译";更远的"脱离 Rust 参考的纯 Auto 风格自研实现"不在本计划范围。本计划把
> 432/434 因当时宿主缺陷而降级的写法(if 链替代 match、kind 字符串载体、判别器
> 结构体)**还原为一对一形态**,行为零变化。
> **硬前提**: Plan 447(H1 or-臂 / H3 内联载荷参数 / H4 卫语句 / H5 二次
> 匹配)+ Plan 448(aavm 与 AA2R 已具备 is/枚举载荷能力)全部合入;开工时
> 五方矩阵基线全绿。
> **总判据**: **行为不变,风格变**——M1-M5 语料闸门、五方矩阵 ①③④⑤、实体 golden
  (.expected.out)全部保持绿;唯一允许的 diff 来源是 divergences.md 的新增/收账登记。
> **关联**: [divergences.md](../specs/aavm/divergences.md)(收账对象)、
  [divergence-rules.md](../specs/aavm/divergence-rules.md) §4(写法规范扩充)。

## 1. 改写总量盘点(prereqs §5 摘要)

- 可 is 化函数约 **83 个**;kind/op 字符串比较枚举化约 **1000+ 处**;判别器结构体
  enum 化 3 组(engine.at 的 `Val{k,i,s}`、codegen.at 的 `I{op,s,n}`、parser.at 的
  E 载体——E 载体属 D20 dump 判据层,本计划**不动**);
- 字符串拼接 → f-string 按位还原 **338 处**(`+ "` 计数);
- 杠杆点:parser.at 新增 `enum Op` + `p_op()`,一步收编 infix_l/infix_r/op_display/
  binop_result 及 typeinfo/codegen 下游共 6 个长链函数;
- 行为层 DIVERGE(D12 游标/D16 char 计长/D18/D19 quirk 照抄/D28 布局等价/D30/D31
  编码语义/D17 保留位——若 Plan 447 已根治 D17 则可回写 continue)**不在
  本轮范围**,继续保留。

## 2. 任务

### Phase 0:准备

- [ ] 0.1 divergence-rules.md §4 补写法规范(基于 prereqs §7 实证语法备忘):
  ①单行枚举需逗号+显式值,无值变体逐行;②Auto 无"模式+guard"组合臂——Rust
  `VI(n) if n>10` 拆独立卫语臂或调臂序(**新增永久 DIVERGE 编号登记**);③同函数
  对同一枚举值多次 is 的写法约定(依赖 H5 已修);④枚举构造参数位暂不内联运行期
  计算值(若 H3 走了兜底路线,此条长期保留;若已根治,登记解除)。
- [ ] 0.2 基线锚定:当前 master commit + 全量闸门跑绿留档(M1-M5 + 五方矩阵
  ①③④⑤ + .expected.out 双件);后续每个 γ 子步以此为对照。

### Phase 1:γ1 样板先行——token.at + lexer.at(M1/M2 闸门保护下)

- [ ] 1.1 token.at:T1 `keyword_kind` 58 臂 if 链 → `is text {...}`(is-on-string,
  对齐 Rust token.rs:365-449 的 match &str);T2 `kind_name` 139 臂 → `is k {...}`
  (is-on-enum;D11 哨兵写法保留);头 Snapshot 与 divergences.md D11b 收账。
- [ ] 1.2 lexer.at:L1 `Token.kind/Tok.kind: str → TokenKind`(构造点 34+2 处,
  `lex_number` 的 kind 变量、`lex_dump`/p_err 输出位经 kind_name);L2 转义/定界
  else-if 链 → is-on-char(lex_string/lex_char/lex_fstr_at 对齐 Rust lexer.rs:462/
  386 的 match esc);L3 `delim_name` 9 臂并入主链(Rust 无此函数,臂内联于
  next_step)。
- [ ] 1.3 闸门:M1(token dump 逐字符)/M2 全绿;②/⑤ 缓存重建后全绿;若 H3 未
  根治,kind 构造点保持提升局部写法。

### Phase 2:γ2 主干——parser/typeinfo/codegen/engine is 化 + 三枚举化

- [ ] 2.1 parser.at:P1 `p_kind → TokenKind`、`p_expect(p, k)` 参数化(kind 位
  字符串比较 ≈330 处、31 个函数;错误消息 kind 名经 kind_name,对齐 Rust
  `format!("{:?}")` 口径);P3 **新增 `enum Op` + `p_op()`**(镜像 Rust
  Parser::op() parser.rs:2897-2939 与 auto-val/value.rs:515-554)——infix_l/
  infix_r/prefix_power/postfix_power/op_display/binop_result 六链一步收编;P2
  Pratt/stmt 巨链 is 化(`expr_with_left` 33 处九连 break ↔ Rust match 臂合并
  parser.rs:2418-2426 等);P4 谓词函数(is_name_kind 等)消解进 is 臂。
- [ ] 2.2 typeinfo.at:Y1 `t_literal_type/t_binop_result/t_unify` is 化;
  `t_array_elem` 的"(array-type ...)"字符串形状解析 → Type 载荷化(D23/D27 收账;
  枚举载荷跨函数已实证可用)。
- [ ] 2.3 codegen.at:C1 `I.op: str → enum OpCode`(S4 子集 30 种助记符,编号
  对齐 opcode.rs);C2 `i_size`/`cg_binop_mnem`/`cg_is_assign_op` is 化(对齐
  opcode.rs:828 operand_size / codegen.rs:5856);C3 cg_expr/cg_stmt p_kind 链
  is 化(D28 行为位:下标赋值两段式游标、FN_PROLOG 占位回填等保留)。
- [ ] 2.4 engine.at:E1 `ev_run_t` 56 处 `op ==` 巨链 → `is ins.op`(二级
  eq/lt/ge 子链一并收编,对齐宿主 run_one_instruction 的 match op);**E2
  `Val{k,i,s} → enum Val{VInt(i) VStr(s) VArr(idx)}`**——8 个 ev_* 函数的
  `v.k==` 分派还原(对齐 auto-val value.rs 的 Value 枚举各 match;D34 核心目标
  收账;D35 arena 槽位语义保持;D36①② 规避写法回退;**gating:Plan 447 的
  H3 已根治**,否则 Val 作为 List 元素/实参高频流经构造位与 native 位不安全)。
- [ ] 2.5 每 2.x 一步一闸门:M2-M5 + 五方矩阵全绿再进下一步;E 载体(D20)与
  E.kind 字符串比较保持 dump 判据层现状不扩。

### Phase 3:γ3 塔顶——a2r.at 自身 + f-string 全量 + 收账

- [ ] 3.1 a2r.at:A1 `ar_method_call/ar_is_mutating_method/ar_rust_ty` 等词法链
  is 化(is-on-string,匹配对象本就是方法名/类型名,对齐 Rust 377 处 match
  name.as_str());A2 p_kind 链 is 化(前置 2.1)。**顺序约束:本步只能用
  Plan 448 已具备发射能力的语法面**(AA2R 已能转译 is/or-臂/枚举载荷)。
- [ ] 3.2 f-string 全量:338 处 `+ "` 拼接按 Rust 参考的 format!/write! 位逐点
  还原(前置:Plan 448 Phase A 已让 parser.at 具备 FStr 解析,a2r.at 发射
  已就绪 D38c);转义/花括号口径对齐主 a2r(3350-3389 的 `{{`/`}}` 规则)。
- [ ] 3.3 收账:divergences.md——D11b/D14 残留/D23/D27/D28(op 载体部分)/D34/
  D36①② 风格类条目逐条标注"已还原/保留原因";新增"模式+guard 拆臂"永久条目;
  各文件头 Snapshot 五字段重写(Coverage 不变,Baseline 重锚定,DIVERGE 清单
  更新);series 复盘文档补记本系列。
- [ ] 3.4 终局验收:五方矩阵 ①③④⑤ 全绿 + G2 自举演示重跑(helloworld/fib)+
  `.expected.out` 双件零变化。

## 3. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 大改写引入行为漂移 | 文件级小步提交,每步全闸门;dump 判据层(D20 E 载体)不动,风格与判据解耦 |
| kind 枚举化后错误消息文本变化 | 错误消息统一经 kind_name 输出,Rust 侧对照口径先固定(M1 语料先验证) |
| E2 Val 枚举化踩 H3 残留 | 硬 gating:H3 未根治不开工;开工后 p09h/p13b 同形态在 lib 自举回路里重验(③⑤) |
| a2r.at 自身改写与它转译自身的能力耦合(自指) | γ3 排最后;每改一段先经"主 a2r 转译(②)"验证,再切"AA2R 自举(⑤)"验证,塔层逐级确认 |
| 工作量大(83 函数/1000+ 比较位) | γ1/γ2/γ3 可拆三个子计划分期执行;γ2 内部按文件四段各自可独立中断续作 |

## 4. Out of Scope

- ext/impl/spec/闭包/泛型声明进入 lib(Rust 参考对应位若需要 impl 形态,先评估
  是否立项 Plan 448 二期 W5,再动 lib;当前一对一以顶层 fn 对译为主);
- 真实 AST 取代 S-expr dump(D20 判据层重构,另立计划);
- 行为层 DIVERGE 还原(D12/D16/D18/D19/D28/D30/D31;D17 视 Plan 447
  结果决定是否顺手回写 continue);
- aavm 特性面扩展(UI/task 等,维持 431 边界)。

## 5. Verification

1. 每个 γ 子步:M1-M5 + 五方矩阵 ①③④⑤ 全绿;`001_smoke`/`002_hello_compile`
   的 .expected.out 零变化;
2. 终局:改写后 lib 与 Rust 参考的函数级对照表抽查(prereqs §5 台账逐文件核销,
   抽样函数逐行 diff 可解释);
3. divergences.md 收账完整性:风格类 DIVERGE 全部标注终态;
4. 性能不回退:M5 corpus 30 例总耗时与基线同数量级(engine 循环 is 化后解释
   开销对比留档)。
