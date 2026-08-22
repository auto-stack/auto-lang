# Plan 417: "Auto 作 Rust 脚本层"发布收尾（359 residuals:Phase E + D2/D3 + A1/A2）

> **状态**: 🔧 部分完成（2026-08-22）——Phase E 五项全关（E1-E5）+ D2 generators + D3 http_client_sync 均已落地；**剩余**：§4 A1/A2 落地页、D2.3/D3.3 双 demo（并入 A1/A2 批次）、§5 359 的 165 checkbox 回填。全部完成后归档。
> **来源**: Plan 359 真实遗留(2026-08-20 复核):D2 generators 用例、D3 http_client_sync(blocked by DIV-HTTP-LANG-1)、Phase E 五项 open、A1/A2 落地页

---

## 1. Phase E 五项 DIV(known-divergences.md,均 status: open)

依赖链与粒度(每项独立分支,先语言侧后 parity 用例侧):

| DIV | 一句话 | 语言侧入口 | 预估 |
|---|---|---|---|
| E1 CHAR-AT-1 | ✅ 2026-08-22 完成:infer_type_from_expr 把 char_at 移入 Int 臂(golden 007_char_at_infer,string_utils 四处 workaround 注解移除,DIV 翻 fixed);**衍生 E1b ✅ 同日由 D2 根治**:register_import_signatures 在 use_stmt 处理时解析可发现模块源登记导入函数签名(string_utils 三方 22/22 恢复全绿) | trans/rust.rs register_import_signatures | — |
| E2 LANG-1 | ✅ 2026-08-22 完成(spec 关联类型四层落地):①parser/AST——`type Item` spec 成员 + 实现处命名式绑定 `as Container<Item=int>`(SpecImpl.assoc_bindings,与位置式 type_args 共存);②trait_checker——按绑定 Type::substitute 替换签名比对,未绑定/未知名显式报错(+4 单测);③VM——声明方法与 E4 合成默认方法编译期替换绑定类型(探针 VM 输出 8/7);④a2r——trait 发射 `type Item;`/`Self::Item`,impl 发射 `type Item = i64;`,修 `impl Self::Item` 误加前缀。golden 12_specs/011(345/345),trait_advanced sub-B L3→L1 三方 14/14 | parser.rs spec 体/as 子句 + trait_checker + vm/codegen + trans/rust.rs | — |
| E3 TRAIT-VM-1 | trait 默认方法 VM 分歧 | ✅ 2026-08-22 完成(有界泛型:parser `has` 语法 + VM CALL_SPEC 动态分派 + CallFrame/字符串重映射两处隐藏 bug 修复 + a2r `<T: Spec>` 发射;trait_advanced 三方 14/14) | vm/codegen.rs trait 分发臂 |
| E4 TRAIT-VM-2 | ✅ 2026-08-22:trait_checker 跳过默认体方法(继承合法)+ codegen TypeDecl 臂经 TypeStore 查询合成未重声明的默认方法(重声明=覆盖,抽象仍强制);trait_vm_tests ×3(继承/覆盖/抽象强制);a2r 侧本就正确;trait_advanced 三方已由同日 ifexpr 批解锁(**10/10 全绿**:a2r 值位尾 if 臂分号缺口根治,golden 013_if_tail_value) | trait_checker.rs + vm/codegen.rs + lib.rs 预注册 | — |
| E5 HTTP-LANG-1 | ✅ 2026-08-22 验证关闭:master 已能解析 `Type.method;` 声明(后续 parser 批次已修,DIV 登记翻转)——**http_client_sync 三方 5/5(连跑三次稳定),D3 解锁并全绿**;D3.3 双 demo 并入 A1/A2 落地页批次 | 实测:auto-parity run http_client_sync | — |

**顺序**: E1 ✅ → E5 ✅ → E4 ✅ → E2 ✅(关联类型四层落地,见上行)→ E3 ✅(TRAIT-VM-1 有界泛型,2026-08-22 修复:`has` 语法 + CALL_SPEC 运行时类型分派 + a2r `<T: Spec>`;详见 known-divergences.md DIV-TRAIT-VM-1 翻转条)。**Phase E 五项已全部关闭**。每项落地后 known-divergences.md 对应条目 status → fixed。


## 2. D2 generators 用例 ✅ 2026-08-22 完成(Plan 417-D2,三方 6/6 一致;VM 带参生成器搬参根治 + a2r ~Iter→Stream 统一降级 + 导入签名登记,E1b 连带根治)

- `parity/libs/generators/` 三向目录(README/auto/tests)——覆盖 `yield`
  迭代器、惰性序列(359 Task D2.2 已有设计:§639 起);
- D2.3 双 demo(教程素材)缺,补齐后 docs/script-to-ship/ 相应章节引用。
- **验收**: parity 三向通过 + parity 仪表盘 L 计数 +1。

## 3. D3 http_client_sync ✅ 2026-08-22 解锁并全绿(Plan 417-E5 验证)

- 实测:master 三方 **5/5 一致**(auto-parity 连跑三次稳定;mock-server
  自动拉起已在 runner 落地)——原登记的 parser 阻塞(DIV-HTTP-LANG-1)
  已被后续 parser 批次修复,属"文档滞后于代码"又一例;
- D3.3 双 demo 留待 A1/A2 落地页批次。

## 4. A1/A2 落地页(website/index.md)

- 补 parity 链接与 hero demo 引用(当前 V3 未达——359 §A 的验收是
  "落地页上线,叙事打动人验证");
- 依赖 D2/D3 的最佳用例产出后做终版;先行版(仅链接 + 仪表盘截图)
  可与 Phase E 并行,0.5 天。

## 5. 165 checkbox 回填(收尾动作,0.5 天)

359 正文 165 个 checkbox 多数已落地未回填(2026-08-20 审计结论)——
按"代码级核验后方可勾选"纪律,finish-plan 流程时统一回填并归档 359。

## 6. 验证矩阵

- 语言侧改动:a2r golden(`RUST_MIN_STACK=33554432`,基线 340/340)+
  auto-ai 三转译零错 + 全量 lib;
- parity 用例:`auto-parity` 三向 runner + 仪表盘重生成;
- 文档:known-divergences.md 状态翻转与 359 回填一致性检查。

## 7. finish-plan 复审记录(2026-08-22)

逐项代码级复验(finish-plan 纪律:不信任勾选,重跑验证):

- **Phase E 五项**:全部 pass。known-divergences 五条 DIV 状态翻转一致
  (E1/E2/E3/E4/E5);a2r golden **348/348**、全量 lib **3080/1**(唯一败=
  已知环境项 route::discovery test_exists)、trait_vm_tests **8/8**、
  parity 四库 **22/6/5/18** 全绿、auto-ai 三转译(agent/client/ai-config)
  **error count 0**、仪表盘已重生成。
- **§2 D2**:partial——parity 6/6 过,但 **D2.3 双 demo 缺**(正文自述,头部
  状态行已列)。
- **§3 D3**:partial——5/5 过,D3.3 双 demo 留待 A1/A2 批次。
- **§4 A1/A2**:fail——website/index.md **0 处 parity 内容**,无后续计划接手。
- **§5 165 checkbox 回填**:移交——计划原文自述属 Plan 359 的 finish-plan,
  勿在本计划重复(回填纪律:代码级核验后方可勾选)。

**分类:C(可行动剩余)**——A1/A2 落地页 + D2.3/D3.3 双 demo(约 0.5-1 天,
可立即开工);§5 属 359 收尾。**不满足归档条件**(A1/A2 未做且无延期根因),
待剩余项完成后重新走 finish-plan → archive。
