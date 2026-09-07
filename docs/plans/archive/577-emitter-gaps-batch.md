---
plan_id: PLAN-577
status: archived
feature_name: a2ts/a2r 发射器缺口批（T1-T4 + R1/R4 + Phase 0 五小修）
author: [zhaopuming, ZCode]
created_at: 2026-09-06
updated_at: 2026-09-07

# /auto-plan:review 回填：
supersedes_spec_components: []   # auto-lang 侧无被推翻的 spec 组件——被取代的知识在 auto-down 侧（gen.mjs B1/B2 后修+DEBTS 行），已随 3736c6d 在彼仓退役
new_spec_components:
  - "specs/auto-lang/trans: a2r Phase 0 五小修发射行为（空 map 字面量 HashMap::new / map 下标 recv_is_map 容器分派 m[&k]+insert / enum derive Hash（eq_safe 同门）/ .length 属性形态 (x.len() as i64) 对齐方法形态 / 保留字表补 Rust 2024 reserved 12 词含 final）"
  - "specs/auto-lang/trans: a2ts T1-T4 发射行为（多 payload enum 工厂 rest-tuple 参数+裸单位变体引用补调用 / is 的 else 臂前缀按分支类型后写 / OptionPattern 真实判型 Some(v)→!==null / struct_names 全位 new / const enum→普通 enum）"
  - "specs/auto-lang/trans 测试面: directed 探针双门——test/a2r/26_plan577 语料四例（rustc 实编门自动收）+ a2ts_directed_probes（内联转译+tsc 驱动，AUTO_TSC 解析序，#[ignore] 按需 427 先例）"
touched_goals:
  - "GOAL-003: a2r 发射器编译级修复批——R 类硬前置就绪，Phase 4 crate 试点解锁（台账红利：known-broken 026 销行）"
  - "GOAL-012: a2ts T1-T4 修复+双 gen.mjs B1/B2 后修退役（TS 生态消费面直译，engine/jade 产品字节中性）"

current_step: 7
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

- [✅ 已完成] **T1** 探针基建：`tests/` 增 directed 探针目录（a2r/a2ts 各一，
      .at → 发射 → 编译检查驱动，红先行）。验证：R1 探针红。
      ——证据：a2r 侧复用既有 golden+rustc 实编门，新增 `test/a2r/26_plan577/`
      四例（001 block_model 整文件 789 行 / 002 R1 点链 / 003 R4 循环 move /
      004 Phase0 五合一）+ golden 注册 ×3；a2ts 侧新增
      `a2ts_directed_probes.rs`（内联转译 + tsc --noEmit --strict
      --isolatedModules 驱动，AUTO_TSC → 主检出 node_modules 解析序，
      #[ignore] 按需跑，427 rustc 冒烟先例）。红先行成立但落点修正：
      **R1/R4 台账形态（2026-08-25）已被过渡计划清偿**（433 A1 点链字段
      as_str / struct 形参 auto-clone / 447 / 019 Phase1）——block_model
      整文件转译 + autodown-core crate 模式产物 rustc 全绿，故"R1 探针红"
      不可复现，R1/R4 探针降级为**回归锁**（golden 已锁）；现存活红 =
      Phase0 探针 8 错（空 map `={}` E0308×2 / map 下标发 usize E0308 /
      enum 缺 Hash E0599+E0277 / `.length` 丢 as i64 E0308 / `final`
      保留字 E0530×2）+ a2ts 四形态（T1 TS2554 / T2 TS1109 悬空 else-if /
      T3 TS2367+TS2839 恒假对象比较+绑定丢失 / T4 TS2348 缺 new×2）。
- [✅ 已完成] **T2** a2r R1（D1）。验证：R1 探针绿 + 既有 golden 零漂移。
      ——证据：002_r1_dot_chain golden 绿（n.name/n.meta.key 多段链/getMeta().key
      全发 `.as_str()`，433 A1 判定族已覆盖）；a2r 全套 368 golden 绿零漂移
      （唯一红=rustc 门对 004 Phase0 探针，设计内红先行）。R1 无需新代码。
- [✅ 已完成] **T3** a2r R4（D2）。验证：R4 探针绿 + golden 零漂移。
      ——证据：003_r4_loop_move golden 绿（循环内 `consume(flat)` 发
      `flat.clone()`，struct 形参 auto-clone 已覆盖；循环绑定 g 同）。
      R4 无需新代码。
- [✅ 已完成] **T4** Phase 0 五小修（D3，含 final 转义）。验证：五探针绿。
      ——证据（rust.rs 五点修，commit e7a56fb30）：①空 map `{}` →
      `HashMap::new()`（注记携带泛型）；②`recv_is_map` 分派（读 `m[&key]`/
      写 `m[k]=v`→`.insert(k,v)`，键值 str 约定同 .insert 方法路径）；
      ③enum derive 四 eq_safe 臂补 Hash；④`.length` 属性形态对齐方法形态
      `(x.len() as i64)`（Bina-Dot 双发射点+宽整型抑制门；`expr_is_len_call`
      扩字段形态防 `.to(int)` 双 cast）；⑤保留字表补 Rust 2024 reserved
      12 词（final/abstract/become/box/do/macro/override/priv/typeof/
      unsized/virtual/yield；gen 除外注记——ed2021 目标下合法 ident）。
      探针 004 五形态绿（HashMap::new/m[&kind]/Hash derive/as i64/r#final/
      m.insert）+ rustc 门 0 意外红。**台账红利**：④顺带清偿
      `07_ownership/026_get_ref_binding` E0599 存量编译债（known-broken
      台账 37→36 销行）。golden 再生 15 例（漂移全为 Hash 行+block_model
      双 cast 修正，已逐例抽查）；全套 3828 绿，唯一红=charts 预存。
- [✅ 已完成] **T5** a2ts T1-T4（D4）+ B1/B2 后修退役。验证：四探针过 tsc +
      gen 管线绿。
      ——证据（lang 侧 commits + auto-down 侧 3736c6d）：T1 工厂 rest-tuple
      参数（tuple payload `(...value:[A,B])` 散装直过；单 payload 保持
      `(value:T)` 防 TS2370）+ 裸单位变体引用补调用（Op.Nil→Op.Nil()，
      enum_unit_variants 登记）；T2 else 臂前缀按分支类型后写（悬空
      `else if (` 修复+缩进回归复验归零）；T3 OptionPattern 真实判型
      （Some(v)→`!==null`+绑定 scrutinee/None→`===null`）；T4a struct_names
      全位补 new + T4b 普通 enum。四定向探针 tsc 全绿；遗留 ignored golden
      再生 84/85（1 红=004_static_method 预存解析期错误，与发射器无关）。
      B1/B2 退役：engine/parser 与 jade/back 双 gen.mjs 移除断言式后修
      （退役信号=B1-ial 断言零命中响亮失败）；TS 产品字节级零漂移；
      Rust 产品 3 行 var-mut 保真微漂（server cargo check 绿）。
- [✅ 已完成] **T6** 回归：a2r golden 全量 + 三恢复库冒烟 + `cargo tf`。验证：
      唯一红=charts 既有。
      ——证据：a2r_compile_smoke（427 serde_json/url/base64 三恢复库）绿；
      `cargo tt --no-fail-fast` 3825/3826；`cargo tf --no-fail-fast`
      3468/3469——唯一红均为 `ui_gen::vue::tests::test_charts_gallery_compiles`
      （AGENTS.md 在案预存，559/575）。
- [✅ 已完成] **T7** 折回与簿记：折回 master；auto-down DEBTS 016×2 + 022 final
      行销号；转介单 052 条目⑤结项注记。验证：台账 diff + 计数落复审。
      ——证据：auto-lang master 合入 a680db8a8（plan-577-dev 5 commits）；
      auto-down master 合入 8b28c1d（auto-lang-dev 3 commits，含
      gen.mjs B1/B2 退役+DEBTS 销号+052⑤ 结项注记）；engine rust 再生链
      cargo test 全绿（block_model.rs 漂移仅 Hash×2）；jade server cargo
      check 绿；双 worktree 已回灌同步（终态清理归 /auto-plan:merge）。

## 复审记录

**复审人**：auto-plan-review(zhaopuming 会话)· 2026-09-07 · worktree
`.wt/lang-577/auto-lang`（plan-577-dev@a680db8a8+补漏 1 commit；auto-down
侧 auto-lang-dev 已 fold 8b28c1d）· 复审法=verify, don't trust（逐验收重跑）

**验收逐条重验**：

| # | 验收 | 判定 | 证据 |
|---|---|---|---|
| 1 | R1/R4+五小修 directed 探针全绿（G1/G2） | **PASS** | rustc 实编门 148 compiled/0 unexpected（worktree 复跑）；探针 004 五形态实查 expected.rs（HashMap::new/m[&kind]/Hash/(x.len() as i64)/r#final/m.insert）；R1/R4 探针 golden 绿（回归锁）。**偏差登记**：R1/R4 台账形态（2026-08-25）已被过渡计划（433 A1/447/019）清偿——block_model 整文件转译+autodown-core crate 模式产物 rustc 双绿实证，plan 文的目标以"探针绿+零漂移"形态达成，代码为准 |
| 2 | a2ts 四探针过 tsc；B1/B2 移除后 gen 管线绿（G3） | **PASS** | a2ts_directed_probes 4/4（复审重跑 1.22s）；auto-down master 实查：双 gen.mjs `addNewToStructCtors`/`const enum` 后修零引用，TS 产品字节级零漂移；engine rust 再生链 cargo test 全绿（block_model.rs 漂移仅 Hash×2） |
| 3 | golden 零意外漂移/三恢复库/tf 唯红=charts（G4） | **PASS** | 漂移全量逐例抽查（a2r 15+cookbook 2=纯 Hash/cast/mut 保真行；a2ts 84/85 再生）；a2r_compile_smoke（427 三恢复库）绿；复审 tf --no-fail-fast **3468/3469 唯一红=test_charts_gallery_compiles**（AGENTS.md 在案预存） |
| 4 | 台账收口（016×2/022/052⑤/gen.mjs 注记） | **PASS** | auto-down master（8b28c1d）实查：DEBTS 15/16/35 行均 ✅已清偿互链 PLAN-577；052 附件⑤结项注记在案；gen.mjs 头注 B1/B2 RETIRED 条目在案 |

**遗漏/延后/workaround 猎查**：
- **遗漏（已抓已修）**：T4 提交 `git add` 圈定 `test/a2r/` 漏 `test/cookbook/`
  ——cookbook 006/010 Hash golden 再生未提交，worktree 未提交态遮蔽，折回后
  **master 暂红 2 例**。复审补提交于 plan-577-dev（"P577 T4 补漏"commit），
  worktree tf 3468/3469 复绿。**merge 前置警示：master 侧该 2 例红由本次
  merge 收敛，请尽快 merge。**
- 债候选（非阻断，登记 KNOWN-DEBT 面）：①a2ts 遗留 ignored golden 语料
  1 例预存解析期红（11_methods/004_static_method，`Box(w,h)` 方法体内多参
  构造解析错——parser 面非发射器面）；②保留字 `gen`（edition 2024 keyword）
  有意未转义（emission 目标 ed2021，注记在 rust_ident 表）；③纪律面保留
  （枚举单 payload 偏好/T? 经 ??）按待澄清#3 默认执行——语义等价改动不做；
  ④jade server 产品 var-mut 保真微漂 3 行（方向正确/gate 绿，机制未深究）；
  ⑤主检出 auto.exe 曾过期（pnpm gen 回退路径隐患）——复审中已重建（09:51）
  消解；⑥auto-down 016 设计偏离行（List\<Attr\> 代 Map）前提部分变化
  （map 下标已修）——回迁仍"非必须"，归彼仓裁定。

**结论**：4/4 验收全 PASS；1 项执行遗漏当场抓出并修复（分支内）；
无未批准延后。**通过**，翻 `reviewed`，可进 `/auto-plan:merge`
（注意先收 master 上 2 例 cookbook 红收敛）。

## 待澄清事项

1. **a2ts 修复的验证深度**：directed 探针过 tsc 为门（不做浏览器级 e2e
   ——TS 发射的消费方为编译期面），是否足够请裁定（默认足够）。
2. **T4 const enum 形态选型**：isolatedModules 兼容发射选 const 对象字
   面量还是普通 enum（默认普通 enum——isolatedModules 下 const enum 本
   就不可用，且无 tree-shinking 需求方）。
3. **退役边界**：gen.mjs B1/B2 与枚举单 payload 纪律的退役是"移除后修 +
   放开纪律"，还是仅修发射器保留纪律（默认：修发射器 + 移除断言式后
   修；纪律面（枚举 payload 形态偏好）不动——语义等价改动不做）。
