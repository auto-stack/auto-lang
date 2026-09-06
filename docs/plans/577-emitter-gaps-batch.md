---
plan_id: PLAN-577
status: drafting
feature_name: a2ts/a2r 发射器缺口批（T1-T4 + R1/R4 + Phase 0 五小修）
author: [zhaopuming, ZCode]
created_at: 2026-09-06
updated_at: 2026-09-06

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 7
---

# [PLAN-577] a2ts/a2r 发射器缺口批

## 变更摘要

清偿 auto-down DEBTS 016 两行 🟡（转介单 052 附件⑤）登记的发射器缺口全
清单：**a2r** R1（点链 str 字段缺 `.as_str()` 自动借用，E0308×21）与 R4
（循环内 owned Vec 实参被 move，E0382×2）——R 类是 Phase 4 crate 试点
的**硬前置**；Phase 0 五小修（空 map 字面量、map 下标按容器分派、enum
derive 补 Hash、self 字段 `.length` 丢 `as i64` cast、保留字 `r#` 转义补
`final`——022 行点名的一行事）。**a2ts** T1（多 payload enum 构造不散装
元组，TS2554）/T2（`is` 的 `else ->` 臂发射断裂）/T3（可选值上 `is` 恒
假）/T4（结构体构造 return/let 位缺 `new` + const enum isolatedModules）
——修复后退役 gen.mjs 后修 B1/B2 与"纪律规避"。全部 directed 探针
（.at → 发射 → 目标语言编译/检查）红先行。

## 目标

- **G1 a2r R1/R4**：点链 str 借用与循环 owned Vec move 两形态定向探针
  编译零错；既有 golden 零漂移。
- **G2 Phase 0 五小修**：五个定向探针各自绿（含 `final` 保留字转义——
  022 行同步销号）。
- **G3 a2ts T1-T4**：四形态定向探针过 tsc；gen.mjs B1/B2 后修退役（断言
  式后修移除后管线仍绿）。
- **G4 回归**：a2r 三恢复库冒烟 + golden 全量 + tf 唯一红=charts 既有。

## 架构方案

不引入新机制：全部为 `trans/rust.rs`（a2r）与 `trans/typescript.rs`
（a2ts）发射器的点状修复——借用插值判定（R1 归入 427 的 str_slice 判定
族）、move 点 clone/borrow 决策（R4）、保留字转义表增补（final——019 八
组清单机制复用）、TS 构造/枚举发射臂修正。每修复配 directed .at 探针固
化为 golden/编译冒烟，防回归（427 行"单一 golden 编译级防线"的扩面起点）。

## 技术栈

- 实现面：`crates/auto-lang/src/trans/rust.rs`（a2r R1/R4/五小修）+
  `crates/auto-lang/src/trans/typescript.rs`（a2ts T1-T4）。
- 验证面：directed 探针（.at → trans → cargo check / tsc）+ 既有 golden
  套件 + a2r 三恢复库（serde_json/url/base64）编译冒烟 + `cargo tf`。

## 需求分析与背景调查

- **来源**：auto-down `DEBTS.md` 016 两行 🟡（2026-08-25 → 2026-09-05 转
  介单 052 附件 条目⑤；详单原在 tmp/dsl-probes/plan016/REPORT.md，tmp
  已清，以台账行为准）。
- **硬前置关系**：R 类（R1/R4）是 Phase 4 crate 试点（a2r 直出 crate）
  的硬前置——试点排期到点必须先清本批。
- **既有同族修复先例**：427（is_str_slice_var str_slice 判定族，R1 同
  族点链形态）、019（保留字 r# 转义 8 组清单机制——final 为清单外漏
  项，022 实证改名即愈但发射器补齐为一行事）。
- **后修/规避现状**：gen.mjs B1/B2（断言式后修）、枚举单 payload 结构体
  纪律、`T?` 只经 `??` 消费纪律——本批修复后逐项退役并断言退役。

## 详细设计

### D1 a2r R1（点链 str 借用）

`rust.rs` 借用判定：点链字段取值（`a.b.str 字段`）若目标形参为 `&str`
则自动补 `.as_str()`——判定入 427 的 str_slice_pattern_bindings 同族集
合（点链形态的 local_var_types 查询扩展）。探针：`a.b` 字段传参编译。

### D2 a2r R4（循环 owned Vec move）

循环体内 owned Vec 实参传给消费 fn 时插 `.clone()`（或改借用——按形参
类型判定，能用借用不 clone）。探针：循环内 Vec 实参调用编译。

### D3 Phase 0 五小修

①空 map 字面量发射 `Map::new()` 型注记；②map 下标按容器类型分派；
③enum 发射补 `#[derive(Hash)]`（现缺致 HashMap key 用不了）；④`self`
字段 `.length` 补 `as i64` cast；⑤保留字转义表增 `final`（复用 019 r#
机制）+ 复查 Rust 2024 保留字全集差集。各配 directed 探针。

### D4 a2ts T1-T4

①多 payload enum 构造调用散装元组发射；②`is` 的 `else ->` 臂正常发
射；③可选值 `is` 匹配发射真实判型（非恒假比较）；④结构体构造在
return/let 位补 `new`（退役 B1）+ const enum 发射改 isolatedModules 兼
容形态（退役 B2）。各配 directed 探针 + tsc 检查。

## 测试设计

| 门 | 内容 | 命令 |
|---|---|---|
| 探针 | 每修复一个 directed .at → 发射 → cargo check / tsc 红→绿 | `cargo test -p auto-lang --lib a2r` / 探针脚本 |
| golden | 既有 a2r/a2ts golden 全量零漂移（显式更新除外） | `cargo test -p auto-lang --lib a2r_tests` + golden 套件 |
| 冒烟 | a2r 三恢复库编译（serde_json/url/base64） | 既有 a2r_compile_smoke（按需） |
| 回归 | tf 全量 | `cargo tf --no-fail-fast`（唯一红=charts 既有） |

## 验收标准

1. R1/R4 + 五小修 directed 探针全绿（G1/G2）。
2. a2ts T1-T4 探针过 tsc；gen.mjs B1/B2 断言移除后 `pnpm gen` 管线绿
   （G3）。
3. 既有 golden 零意外漂移；三恢复库冒烟绿；tf 唯一红=charts 既有（G4）。
4. 台账收口：auto-down DEBTS 016 两行销号（互链本计划）+ 022 final 行销
   号；gen.mjs B1/B2 后修注记退役；转介单 052 条目⑤结项。

## 执行步骤

- [ ] **T1** 探针基建：`tests/` 增 directed 探针目录（a2r/a2ts 各一，
      .at → 发射 → 编译检查驱动，红先行）。验证：R1 探针红。
- [ ] **T2** a2r R1（D1）。验证：R1 探针绿 + 既有 golden 零漂移。
- [ ] **T3** a2r R4（D2）。验证：R4 探针绿 + golden 零漂移。
- [ ] **T4** Phase 0 五小修（D3，含 final 转义）。验证：五探针绿。
- [ ] **T5** a2ts T1-T4（D4）+ B1/B2 后修退役。验证：四探针过 tsc +
      gen 管线绿。
- [ ] **T6** 回归：a2r golden 全量 + 三恢复库冒烟 + `cargo tf`。验证：
      唯一红=charts 既有。
- [ ] **T7** 折回与簿记：折回 master；auto-down DEBTS 016×2 + 022 final
      行销号；转介单 052 条目⑤结项注记。验证：台账 diff + 计数落复审。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **a2ts 修复的验证深度**：directed 探针过 tsc 为门（不做浏览器级 e2e
   ——TS 发射的消费方为编译期面），是否足够请裁定（默认足够）。
2. **T4 const enum 形态选型**：isolatedModules 兼容发射选 const 对象字
   面量还是普通 enum（默认普通 enum——isolatedModules 下 const enum 本
   就不可用，且无 tree-shinking 需求方）。
3. **退役边界**：gen.mjs B1/B2 与枚举单 payload 纪律的退役是"移除后修 +
   放开纪律"，还是仅修发射器保留纪律（默认：修发射器 + 移除断言式后
   修；纪律面（枚举 payload 形态偏好）不动——语义等价改动不做）。
