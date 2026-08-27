---
plan_id: PLAN-453
status: drafting
feature_name: W5 ext/impl 能力与 lib 方法化
author: [zhaopuming]
created_at: 2026-08-27T00:00:00+08:00
updated_at: 2026-08-27T00:00:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 26
---

# [PLAN-453] W5: ext/impl 能力与 lib 方法化

## 变更摘要

aavm 七层工具链（parser/typeinfo/codegen/engine + AA2R 发射侧）收编
`ext` 声明与方法形态,然后把 auto/lib 七文件的 receiver-首参自由函数
(约 40 个 P 函数及 Tok/CG/Ar等同族)还原为 `ext P { }` 方法形态,消除
与 Rust 参考(impl 块)的最后一处纯外观差异。

**总判据**:行为不变,形态变——M1-M5 语料闸门、五方矩阵①②③④⑤、
实体 golden(.expected.out)、G2 自举双演示全部保持绿。

## 目标

1. aavm v2 子集解除对 `Ext` 的拒收(parser.at is_unsupported_stmt_kind),
   支持 `ext Type { fn... }` 声明与方法调用语法;
2. AA2R 具备 ext → Rust `impl` 的发射能力(`&self`/`&mut self` 按
   体内 mutation 判定,镜像宿主);
3. lib 方法化:P/Tok/Ar 等 receiver-首参函数迁入 ext 块,顶层函数表
   收敛为真方法;
4. 语料五组(M2/M3/M4/M5/golden)扩展 ext 面,矩阵稳定集扩充。

## 架构方案

**方法调用的执行降级(核心决策)**:codegen 编译期把
`p.text(p2, ...)` 改写为 `P.text(p, p2, ...)` 自由函数调用(receiver
作显式第一参),engine 零新指令——Call 机制原样复用,无虚表/动态分派。
名字采用 `Type_method` 全局唯一形(Trait 同名冲突面:lib 不实现 Trait,
AA2R 发射回 impl 后 Rust 自行解析)。

**&self / &mut self 判定(AA2R)**:镜像宿主——方法体内对 self 本体
或其字段有赋值/可变方法调用即发 `&mut self`(ar_scan_mutations 同族
扫描器扩展 self 分支);只读则 `&self`。已在宿主导线实证:
`self.n = self.n + 1` → `pub fn bump(&mut self)`。

**塔式顺序**:parser → typeinfo → codegen/engine → AA2R → lib 改写。
lib 改写阶段⑤列自举将消费 AA2R 新能力(11.1 同款自指,先行验证再切)。

## 技术栈

Auto(aavm lib 七文件)+ Rust(宿主参考/闸门)/cargo test 五闸门/
parity 五方矩阵/auto-parity CLI。

## 需求分析与背景调查(2026-08-27 实测)

| 层 | ext 现状 | 证据 |
|---|---|---|
| 宿主 parser/a2r | ✅ 完整 | `ext Counter{fn bump(){self.n=self.n+1}}` → `impl Counter{pub fn bump(&mut self)}` rustc 过 |
| aavm parser.at | ❌ 拒收 | TokenKind.Ext 在 is_unsupported_stmt_kind(parser.at:1914) |
| AA2R(a2r.at) | ❌ 不发射 | enum body member/type body member 同族unsupported;447 部分② Out of Scope 明文"spec/ext/impl/use/dep… §5 二期" |
| lib | 顶层 fn 对译 | receiver-首参约定,约 40+ P 函数(P_text/P_kind/P_next…) |

既有样例资产:test/a2r/02_types/010_ext_keyword、06_pattern_matching/
007_is_in_ext(ext 内 is self 分派已实证)、stdlib/auto/str.at(#vm shim
家族——本计划不含原生 shim,lib 方法均为纯 Auto 实现)。

关联:447 档案"部分② 二期 W5"预留项;D37(mut 参数表口径)、D40 续
(f-string 三缺口教训:能力先行验证再动批量改写)。

## 详细设计

### ext 声明解析(parser.at)

- `ext Name { ... }`:剥 pub(可选)、成员仅 fn(暂不支持 const/类型别名);
- 方法签名:fn name(params) ret {body} 或声明尾分号(abstract,拒收——
  lib 全部有体);
- **self 词法定性(Phase A 定案)**:探针显示 self 疑似经 Ident 通道路径
  (宿主 lexer 无 SelfKw 关键字迹象),由 a2r/codegen 在方法上下文特判;
  以 live dump 为准登记 D42;
- dump:S-expr 形态以宿主 rust_parse_dump live 输出逐字符为准(D18
  quirk 照抄规则适用)。

### 方法表与类型面(typeinfo.at)

TFnSig 增 owner 字段(""=自由函数);t_stmt_walk 注册 ext 成员;方法体内
self 入作用域(类型 = owner)。

### 降级发射(codegen.at)

- cg_expr 的 Dot-call 位:recv 类型查方法表命中 → emit Call(改名目标),
  先发 recv 载荷后发实参;
- 名字改写:`Owner_method`;引擎/链接器零改动(fns 符号表天然承载);
- 类型体内直接互调(short call)同规则统一走 Call。

### AA2R 发射

- ar_prescan_ext:成员 fn 表登记(owner/name/params/ret/mut_self 标志);
- ar_emit_impl:`impl Owner { pub fn name(&[mut] self, ...) -> T {...} }`,
  &mut 判定 = ar_scan_mutations 扩展(self 赋值/self.field 赋值/self
  可变方法调用);非方法自由函数保持现状;
- 方法调用位:Ident-chain 命中方法表且前缀为 owner 型变量 → 发
  `recv.method(args)`(注意与既有 ar_method_call 字符串方法表的拦截
  优先级:用户方法先于内建映射)。

### lib 方法化改写(F 批)

分文件独立提交,每文件一全闸门:lexer(Tok)→ parser(P,~40 fn)→
typeinfo(P 复用/parser 借用助手剥离后自持或 ext P 关联注记)→
codegen(CG)→ engine(Val/Val 场景讨论:enum 亦可挂 ext,同批评估)→
a2r(Ar/Tok)。token.at 无自定义类型,跳过。对外可见性:跨文件调用
(P_ 前缀处)改为方法调用形态,p_err/p_err 等错误路径同步。

## 测试设计

- M2:corpus_m2/p25_ext_decl(声明+方法调用+self 字段读写,dump 对齐);
- M3:t08_ext(方法内 .type 查询/self 型);
- M4/M5:b34_ext_call(反汇编含改名 Call + 行为一致);
- golden:g08_ext_impl(at→rs,链 007_is_in_ext 同构);
- probe:p01_is_string 经 AA2R(已具 impl 发射后)rustc 冒烟维持 ignore-on-demand;
- 矩阵:每 Phase 收口跑 ×5 列;F 批每文件跑;
- 终局:G2 双演示重跑 + .expected.out 零变化。

## 验收标准

1. 五闸门(M1-M5)+ AA2R 闸门全绿,旧语料零漂移;
2. 矩阵 ≥38 例全绿×5(36 + ext 两例);
3. lib 七文件receiver-首参函数全部收敛进 ext 块(grep 断言:
   `^fn [pt]_` 计数归零,P_/Tok_/CG_ 等公开入口除外——外层薄壳允许
   登记后保留);
4. G2 helloworld/fib 逐字符一致;
5. divergences D42(ext 形态)与各文件头 Snapshot 收账完成。

## 执行步骤

### Phase A:宿主基准与判据固定(0.5d)
- [ ] A1 写 dump 探针:rust_parse_dump 打印 007_is_in_ext/selfprobe 的
  ext 块 S-expr,登记 m2-ast-dump-format.md;verify:肉眼比对 + 登记 diff。
- [ ] A2 self 词法定性(Ident vs 关键字)+ 方法位/字段位解析路径笔记,
  并入 divergences 新条目草稿;verify:lexer.rs grep + 探针三件套留档
  %TEMP%/p453/。
- [ ] A3 `#[vm]` 语义边界确认(lib 纯 Auto 方法不带 attr),写入详细设计
  附录;verify:stdlib str.at 对照笔记。

### Phase B:parser.at(B 前,M2)(1d)
- [ ] B1 is_unsupported_stmt_kind 摘除 Ext;新增 parse_ext_decl
  (auto/lib/parser.at);verify:cargo test -- aavm2_m2。
- [ ] B2 方法签名/体解析复用 parse_fn 路径,ext 头解析 + dump 装配对齐
  A1 定案形态;verify:M2 p25_ext_decl 逐字符一致。
- [ ] B3 corpus_m2/p25_ext_decl.at + .expected 落盘;verify:aavm2_m2 绿。

### Phase C:typeinfo.at(M3)(0.5d)
- [ ] C1 TFnSig.owner 字段 + ext 成员注册 + self 绑定(auto/lib/
  typeinfo.at);verify:cargo test -- aavm2_m3。
- [ ] C2 corpus_m3/t08_ext.at 落盘;verify:aavm2_m3 绿。

### Phase D:codegen.at + engine(M4/M5)(1.5d)
- [ ] D1 方法表构建(cg 层 ext 注册)与 Dot-call 拦截改写
  (auto/lib/codegen.at);verify:aavm2_m4 b34 反汇编检视。
- [ ] D2 corpus_m4/b34_ext_call.at(+expected);verify:aavm2_m4+M5 绿。
- [ ] D3 engine 回归确认(Call 复用零改动假设成立);若不成立,补
  engine.at 修正任务;verify:aavm2_m5 全量。

### Phase E:AA2R 发射(1.5d)
- [ ] E1 ar_prescan_ext/ar_emit_impl + &mut self 判定(auto/lib/a2r.at);
  verify:007_is_in_ext 文本对齐主 a2r live。
- [ ] E2 方法调用位改写接收者形态(user-method 先于内建表);
  verify:corpus_a2r/g08 文本对齐 + aavm2_a2r 绿。
- [ ] E3 corpus_a2r/g08_ext_impl.at 落盘;probe 冒烟(ignored)抽跑;
  verify:rustc --crate-name w5probe 零错。

### Phase F:lib 方法化(3d,每文件一提交一全闸门)
- [ ] F1 lexer.at Tok 方法化(Tok.kind/Token.kind 构造点收编);
  verify:aavm2 全量 + 矩阵。
- [ ] F2 parser.at P 方法化(~40 fn 搬迁 + 跨文件调用位改写);
  verify:同上 + grep `^fn p_` 清零断言。
- [ ] F3 typeinfo.at(复用助手的归属整理 + 方法化);
  verify:同上。
- [ ] F4 codegen.at CG 方法化;
  verify:同上。
- [ ] F5 engine.at Val/其它载体方法化(enum 挂 ext 可行性同批评估,不可行
  则记账保留自由函数并注明原因);
  verify:同上。
- [ ] F6 a2r.at Ar/Tok 方法化(与 E 段自指:先②主转译验证,再⑤自举);
  verify:同上 + G2 重跑。

### Phase G:收账与终局(0.5d)
- [ ] G1 divergences D42 登记 + 七文件头 Snapshot 更新 + 计划状态置 🟩;
  verify:文档评审。
- [ ] G2' 终局验收:矩阵(≥38 例)×5 + G2 双演示 + .expected.out 零变化
  + M5 耗时数量级记录;verify:auto-parity aavm + g2bin 手跑。

## 复审记录

(待 /auto-plan:review 填写)

## 待澄清事项

1. enum 载荷载体(Val)能否挂 ext——宿主导线显示 impl enum 合法,但
   aavm 运行时对 VInt(int) Primitive 判别位的 self 透传需验证(F5 决策点);
2. 跨文件方法可见性:merge 转译下 owner 类型单一定义点已保证,不需
   额外导入机制(预判,开工时实证);
3. `#[vm]` 原生 shim 家族是否纳入本期(建议不纳入,另立)。
