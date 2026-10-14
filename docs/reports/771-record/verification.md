# PLAN-771 验证收据（verification.md）

- 工作树：`D:/autostack/.wt/lang-771/auto-lang` @ `plan-771-dev`；
  基面 master `199d6509e`（774 已合入）。
- 本文件由 T-06/T-07 收尾写定；门禁原始收据在
  `experimental/ac-core/target/ac-verify-receipts/record-771/`（gitignored，
  含逐例输入/输出/hash）。规范增量提案见
  [proposed-spec-delta.md](proposed-spec-delta.md)（未发布 canonical）。

## 1. 提交与产品清单

| 项 | 值 |
|---|---|
| T-01 提交 | `f909781b4`（design.md 冻结 + whole-path probe 收据） |
| T-02 提交 | `fb8bd7e14`（HIR 域/schema/binder/verifier + record_hir 19 测试） |
| T-03 提交 | `f46cd128b`（source 七模块 record 面 + record_source 14 测试） |
| T-04 提交 | `b0a7c31f9`（native 聚合 lowering） |
| T-05 提交 | `67ee3cc01`（record_native 9 PE + copy-elision） |
| T-06/T-07 提交 | 门禁脚本 + 文档（见 git log，HEAD 待复审绑定时填写） |
| 依赖 | 无 auto-down / crates 改动；ac-core path 依赖零源码 diff |

新增/修改（相对基面）：`experimental/ac-core/src/hir.rs`、
`descriptor.rs`、`verify.rs`、`native.rs`、`source/{ast,lexer,mod,parser,
resolve,typecheck,adapter}.rs`、`main.rs`、`lib.rs`；
`schema/record.atom`；`tests/{record_hir,record_source,record_native}.rs`；
`fixtures/record-{hir,source}/{valid,invalid}`；
`scripts/verify-ac-record-771.ps1`；`docs/reports/771-record/*`；
README 增量。旧域（core/scalar）源码路径仅共享枚举扩面（带域防御），
接受/拒绝集合零改动（旧 gates 重跑证明，§4）。

## 2. 测试设计执行（plan §6 Case 表 → 实际入口）

| Case | 实际入口 | 结果 |
|---|---|---|
| V-01 | `record_hir.rs`（三维身份/隔离/dispatch/未知三元组）+ gate `profile isolation` 步 | ✅ |
| V-02 | `record_source.rs`（源码腿）+ `record_hir.rs`（HIR 腿）+ `record_native.rs`（source flagship PE exit 0 / HIR fixture PE exit 1）+ gate SP01/HP01 | ✅ |
| V-03 | `record_hir.rs`（同构异名/前向引用/自环/互环/断开环/1024 链/深路径）+ `record_source.rs`（名义拒绝含 record 名标注） | ✅ |
| V-04 | `record_hir.rs` 8 组程序化 Bundle 变异（字段 TypeId/FieldId 越界与经标量续走/签名参数交换/结果改型/receiver 改位/owner 改型/字段 TypeRef 越界/ctor 绑定删失）+ 合法兄弟例 | ✅ |
| V-05 | `record_native.rs`（命名实参重排+嵌套 record 实参、out-slot 链式 swap(mk)/swap(swap(mk))、.mut 递归标量+record、receiver 一次+实参序 trace `aab`）+ gate SP02 | ✅ |
| V-06 | `record_source.rs`（let 写拒绝/缺 .mut/mut fn）+ `record_native.rs`（mut 只写目标 PE）+ `record_hir.rs`（双 mut 排他/let 互借/游离借位）+ gate SN01..SN03 | ✅ |
| V-07 | `record_source.rs`（生成 Atom field_args 源码序 + 绑定按名落位 oracle）+ `record_native.rs`（命名构造 trace `ba`）+ gate SP02/SN06 | ✅ |
| V-08 | `record_source.rs`（static/实例误用、同名遮蔽、未知字段/方法）+ `record_hir.rs`（receiver 契约）+ gate SN07 | ✅ |
| V-09 | `record_hir.rs`（1024 链 verify、深 field path）+ `record_native.rs` 深输入 PE（1024 层类型图、4096 混合字段路径段、构造链 copy-elision：800 层 main 324817→3217 指令、46s→1.1s 构建）+ `deep_regression.rs` 既有 scalar 深输入回归 | ✅ |
| V-10 | `scripts/verify-ac-record-771.ps1` 全 15 步（本收据 §4）+ 旧 gates（741 -SkipMainGates / 745 / 767）+ SENTINEL 事务（失败 build 三件制品 hash 保持）+ 收据身份行 | ✅ |

## 3. 覆盖内省

- record_hir 19 / record_source 14 / record_native 9 = 42 个新测试；
  全 ac-core `cargo test --locked` 187/187（旧面 145 全数保留）。
- 整数 oracle 独立推导（探针 P2 布局公式 8+4k），无 golden 自动更新；
  深输入为验证规模而非语言上限，无固定语法深度排除。
- 已知非阻断：诊断码 `verify.shadowed-type`/`verify.duplicate-decl`
  的 HIR 级触发面由 source 前端同语义码覆盖（source.duplicate/
  source.syntax），HIR 直写路径由 binder `bind.duplicate-id` 族把关
  ——码表在 design.md §5 冻结，语义无缺口。

## 4. 门禁执行（verify-ac-record-771.ps1）

**18 步全绿，0 failed，10 cases**（2026-10-14 实跑；原始收据
`target/ac-verify-receipts/record-771/`，含 context/summary/cases
与逐例 build/run 输出与 SHA256）：

| # | 步骤 | 结果 |
|---|---|---|
| 1 | context（HEAD/branch/rustc/cargo + binary/support-lib SHA256） | PASS |
| 2 | cargo check（ac-core） | PASS |
| 3 | cargo fmt --check | PASS |
| 4 | cargo check --all-targets 零警告 | PASS |
| 5 | cargo test --locked（全 ac-core 187/187） | PASS 36.8s |
| 6 | cargo test record_native（单线程 9/9） | PASS 4.6s |
| 7 | 构建工具二进制 | PASS |
| 8 | 构建 trace 支持库 | PASS |
| 9 | 正向 SOURCE：flagship 构建+运行 exit 0 | PASS |
| 10 | 正向 SOURCE：求值序 trace `ba` | PASS |
| 11 | 正向 HIR：record fixture 构建+运行 exit 1 | PASS |
| 12 | 负向 SOURCE 7 例钉码拒绝 | PASS |
| 13 | profile 隔离（core/scalar 拒 record；未知 profile exit 2；域行） | PASS |
| 14 | SENTINEL 事务（失败 build 三件制品 hash 保持） | PASS |
| 15 | 收据身份行（record 三元组 + input-hash） | PASS |
| 16 | 旧 gate 741（-SkipMainGates） | PASS 9.1s |
| 17 | 旧 gate 745（source core-i32） | PASS 41s |
| 18 | 旧 gate 767（scalar） | PASS 25.6s |

实施期修正（oracle 侧，非合同变化）：SN04/SN07 两 fixture 初稿写成
合法程序（同构异名 seeding 全 i32 字段合法；类型名遮蔽当时未立规），
改为真实违规（B 值入 A 类型字段；局部与 record 同名拒绝
`source.duplicate`——V-08 遮蔽矩阵的明示规则落地）。

1. context（HEAD/branch/rustc/cargo + binary/support-lib SHA256）
2. cargo check（ac-core，零警告）
3. cargo fmt --check
4. cargo check --all-targets 零警告
5. cargo test --locked（全 ac-core 187/187）
6. cargo test record_native（单线程，9/9）
7. 构建工具二进制
8. 构建 trace 支持库
9. 正向 SOURCE：flagship 构建+运行（V-02/V-06/V-08）
10. 正向 SOURCE：求值序 trace `ba`（V-05/V-07）
11. 正向 HIR：record fixture 构建+运行（exit 1 = 快乐路径）
12. 负向 SOURCE 7 例（钉码拒绝）
13. profile 隔离（core/scalar 拒 record 面；未知 profile exit 2；
    record HIR check 域行）
14. SENTINEL 事务（失败 build 后 exe/obj/receipt 三件 hash 不变）
15. 收据身份行（source-profile/hir 三元组/input-hash）+ 旧 gates
    741（-SkipMainGates）/745/767 串行重跑

## 5. 输入指纹（SHA256）

| 输入 | SHA256 |
|---|---|
| 774 interface-contract.md | 046e4b5da59b100aec1126aa52e9f2b17731537757874f4ee9ca34d524dd9548 |
| 774 decisions.md | bb4d8e312c5bae80db707fee5e7f707bc0272e0cd57152ddbb14656ea0320e3e |
| Design 10-language-syntax.md | 627c55363d29a47f226d3588c60220f20f879a552840a63dde9aa4a569d55e6d |
| Design 04-memory-ownership.md | eb87507a49666c42b4cc35b6316fcd2ddd570ce9e868b536a9c25732b9a39d54 |
| packages/aavm/lib/parser.at | 377129b9b06afec72b5ccbc947b6d50c778b7a41f3df2f22ad6c99048b2ea559 |
| 772 归档计划 | 99273e9cd59a90db9662702317ab1274c6336fd979bd24ce137ea2658698a1c6 |
| 774 归档计划 | 88d0e1d8cba19c55e330814c5343d65139cbf683e3110f445fb0b7091ad5f880 |
| design.md（本计划 T-01 冻结） | 9f8e52f00a947dd228677a9cd1374fcba15e2b7d8ef43700d171061fd009e266 |

## 6. 后续 owner（不在本计划）

- text-record 组合（770 合入后第二合入者重绑基线联跑其 gate；
  774 interface-contract §6 顺序）。
- 可变实例方法（`mut fn`）、unit/隐式尾返回、资源字段、List/enum/
  use——扩面计划另行立项（design.md §9 拒绝面）。
- 通用 pass（跨域 lowering/优化）保持 planned（SD-02/03）。
