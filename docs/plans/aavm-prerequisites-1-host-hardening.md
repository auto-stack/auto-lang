# Plan aavm-prerequisites-1: 宿主加固——VM is/enum/continue 缺陷修复 + 主 a2r 发射修复

> **状态**: 🟦 已立项待执行(2026-08-25;三件套之 ①,先行)
> **来源**: [idiom-upgrade-prereqs.md](../specs/aavm/idiom-upgrade-prereqs.md)(2026-08-25
> 实证调研:18 件 probe + rustc 实编译验证),§3 H1-H6。
> **定位**: aavm lib 从"类 C 最简风格"升回"与 Rust 参考一对一"的**宿主前提**。
> 做aavm 的目的之一就是在自举路径上暴露宿主缺陷——本计划就是把这批暴露出的问题
> 在宿主侧修掉,而不是继续绕开。
> **关联**: aavm-prerequisites-2(aavm 新语法能力,建议在本计划完成后开工,Phase D
> 可并行)、aavm-prerequisites-3(lib 风格升级,硬依赖本计划 H1/H3);
> [divergences.md](../specs/aavm/divergences.md) D11b/D17/D25-①/D34/D36-① 收账对象;
> KNOWN-DEBT-AND-RISKS.md:112-113。
> **基线**: master `c3ee519d0` + 2026-08-25 构建;aavm2 闸门当前 10 项全绿。

## 1. 目标

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

## 2. 任务

### Phase 0:复现件常驻(随各修复项一并落,不留红 master)

- [ ] 0.1 建 `test/vm/99_idiom_probe/`:p01/p02b/p06/p07/p08/p09h/p13b 等按
  `名称.at + 名称.expected.out` 平铺落盘(期望值 = **正确**语义输出),随对应
  H 项修复同 PR 落地;p03/p04/p05/p10/p13a/c 等绿件直接先落(纯加固回归)。
- [ ] 0.2 `vm_file_tests` runner 确认新目录被自动发现(平铺文件惯例同
  `10_types/020_struct_to_str.at`);跑 `cargo test -p auto-lang --lib
  --features test-vm-files -- test_aavm2` 基线留档。

### Phase 1:VM 三修(H1/H2/H3)

- [ ] 1.1 **H1 枚举 or-臂**:`Stmt::Is` 的 EqBranch 处理(codegen.rs:3537 起)在
  `patterns.len() > 1` 时改走多模式路径(逐模式 EQ + JMP_IF_NZ,枚举判别值可直用);
  **同步修 `Expr::Is` 副本(9348 起,两份代码逐段对照)**;p02b 进 99_idiom_probe。
- [ ] 1.2 **H2 continue 回填**:照 2588 的 `patch_jump_to` 模式补齐 7 处变体
  (Cond=while、Ever、Named/Indexed/Call/Destructured 系);p06/p07 进常驻;
  全量 VM 语料回归(现状恰好零覆盖 while+continue,修复本身无 golden 波动)。
- [ ] 1.3 **H3 根治(目标)或规避(兜底)**:
  - 根治:CALL_NAT 携带显式 arg_count 替代 `shim_list_new` 的 `sp > bp+num_locals+2`
    猜测(一刀同时收 D25-①);审计 `pop_arg_i32` 族算术敏感 shim 改 `pop_arg_nv`
    + tag 分派;p09h/p13b 转绿(不提升局部也正确)。
  - 兜底(若根治涉及面过大单独分期):H3 降级为写法规范(lib 永远提升局部),
    但 **prerequisites-3 的 engine.at Val 枚举化(γ2-E2)前必须完成根治**,本项
    不得以兜底状态进入 γ2。
  - 回归:`repro_242_string_pool_uaf`/`repro_d30_negative_int_roundtrip` 及既有
    RC 用例零变化;aavm2 闸门全绿。
- [ ] 1.4 顺手加固(可选,不阻塞):`IS_VARIANT`(engine.rs:4006-4026)对"非对象
  非 null 的用户 enum"默认按 Option.Some 处理改为显式错误;`CONSTRUCT_INSTANCE`
  越界回退(3904-3907)改报错。目的:把静默错臂变成显式失败,服务后续调试。

### Phase 2:主 a2r 两修 + 一补(H4/H5/H6)

- [ ] 2.1 **H4 卫语句臂**:IfBranch 发射改为 `<绑定> if <cond> =>`;scrutinee 为
  复杂表达式时先 `let __is_x = expr;` 再 `match __is_x`;p08 转译产物 rustc 零错。
  现有 golden 全仓零覆盖卫语句(零 golden 波动)。
- [ ] 2.2 **H5 二次匹配 E0382**:scrutinee last-use 分析——非末次使用发
  `match &v`(臂绑定变引用,`format!`/`println!` 用法兼容);p05 产物零错;
  a2r golden 06 组回归。
- [ ] 2.3 **H6(顺手)**:与 2.2 动同一片代码时,is 臂发射按枚举载荷表回填
  `local_var_types`(数据源 `enum_tuple_field_types`,rust.rs:14735+),KNOWN-DEBT
  113 观察项收口或明确残留。

## 3. 风险与缓解

| 风险 | 缓解 |
|---|---|
| H3 根治动 RC 记账咽喉,牵连面大 | 分两刀:先 arg_count 显式化(纯增量),后 pop_arg_nv 收窄;每刀全量回归;兜底路径已定义 |
| H1 的 Expr::Is/Stmt::Is 双副本漏改 | 修复后加一条"双副本同源"注释;probe 覆盖语句位+表达式位两形态 |
| H5 的 `match &v` 改写引入借用新错 | 先只在"同一函数内 ≥2 次 is 同一 scrutinee"的窄条件启用;golden 全量 diff 审 |
| 修复与 plan-442 执行中的 VM 改动冲突 | 开工前 rebase;H 项全部小粒度独立提交 |

## 4. Out of Scope

- AA2R(a2r.at)的任何改动(→ prerequisites-2);
- lib 风格改写(→ prerequisites-3);
- ext/spec/闭包/f-string 等其它特性面(当前实证非阻断);
- VM 性能优化。

## 5. Verification

1. `99_idiom_probe` 全绿(含 H1/H2/H3 复现件转绿);
2. `cargo test -p auto-lang --lib` 全量零回归(3876+ 用例);
3. a2r golden(25 组/220 文件)除 H4/H5 预期组外零漂移;
4. aavm2 闸门 M1-M5 + smoke 全绿;
5. H3 根治路径下:p09h/p13b 不提升局部也正确;D25-① 场景(表达式位零参原生调用)
   一并验证;
6. divergences.md 登记对应收账(D17/D25-①/D36-① 状态更新),KNOWN-DEBT 112/113
   状态更新。
