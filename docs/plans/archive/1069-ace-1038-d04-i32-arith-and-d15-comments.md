---
plan_id: PLAN-1069
status: archived                  # drafting → executing → execution_done → reviewed → archived
feature_name: ACE i32 校验算术单一发射切片与 1038 文档更正
author: [lead]
created_at: 2026-11-17
updated_at: 2026-11-17

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [ace, auto-ac]
current_step: 7
total_steps: 7
plan_revision: 1
depends_on: [PLAN-1038]
---

# [PLAN-1069] ACE i32 校验算术单一发射切片与 1038 文档更正

## 0. 变更摘要

本包执行已归档 [PLAN-1038](1038-ace-reuse-candidates-and-cost-decisions.md) §9 T-02 六项决定中的两项 go：

- **D04（首个有界切片）**：把 `native/src/collections_lower.rs:1420–1469` 与 `closures_lower.rs:955–1004` 两处**50 行逐字相同**的 i32 比较/校验四则发射抽成 `ace-native` crate 私有 `lower_arith.rs`，两 `Lower::binary` 委托；并按 1038 §9 T-03 合同补当前缺失的 **i32 陷阱/正常值 native PE 反例**（Div/Rem/MIN%-1/溢出）。
- **D15（纯文档）**：更正 `core`/`av-core`/`native` 三处头注释与 `core/Cargo.toml` description 共 5 处事实漂移。

D13（mir_verify init 私有边界）按 1038 的“次序列”另立后继包，本包不实施。不合并 layout/cache/wrapper/frame、不改 ABI/公共接口/依赖、不跑 ace-perf。

## 1. 目标

- **G1（D04）**：i32 校验算术（6 比较、Add/Sub/Mul 的 i64 中间值+范围检查、Div/Rem 的除零与 MIN/-1 guard、trap 码 -30/-31）只有一个发射 owner；两 family 的已发代码**逐字节不变**（COFF 等价），closures 的 typed-numeric 拒绝面不 widen。
- **G2（D04 oracle）**：补齐 i32 车道在真实 PE 上的永久反例：正常值、除零、MIN%-1、溢出，collections 与 closures 各 4 例，断言正常值/退出 70 与 `MN-COLL-TRAP code=31/-30`，并有可区分错误实现的变异负控。
- **G3（D15）**：5 处文档事实与当前 Cargo/Spec 一致（Cranelift 已退役、core 允许共享 syntax 叶子、av-core 非零外部依赖、native 旧参照已退役/构建面角色、core Cargo description 双目标）。

**非目标**：不实施 D13；不改 `ABI`/`CODEGEN`/`PROFILE`/`RECIPE` 串或 manifest 身份；不改 frame/capture/cleanup 顺序（1038 引 D08）；不新增公共 API；不动其他计划在途 WIP；不跑 `cargo t all`/ace-perf。

## 2. 架构方案

唯一新增文件 `experimental/ace/crates/native/src/lower_arith.rs`：

```rust
pub(crate) fn emit_i32_binary(
    fb: &mut FunctionBuilder<'_>,
    trap: Block,
    op: BinOp,
    a: Value,
    b: Value,
) -> Option<Value>
```

- 语义 = 现有两函数体的逐行等价：6 种比较 → `icmp`+`uextend(I32)`；Div/Rem 先除零 guard（`i32(-31)`）再 MIN/-1 guard（`i32(-30)`）；Add/Sub/Mul/Div/Rem 以 `sextend(I64)` 计算，Add/Sub/Mul 走 i32 上下界检查（`-30`）后 `ireduce(I32)`；`guard` 的块创建顺序与现实现一致（`create_block` → `brif` → `switch_to_block`）。
- 返回 `None` 表示“非 i32 车道 op”（例如 `BinOp::Num` 或未来新增变体），调用方保留原拒绝/分派。
- `collections_lower::Lower::binary` 保留 `if let BinOp::Num` → `binary_num` 分派与 `Result<(), MnError>` 签名；之后取 `a/b = self.scalar()`，调 helper，`self.scalar_to(dst, value)`。
- `closures_lower::Lower::binary` 保留 `()` 签名；取 `a/b`，调 helper，`self.scalar_to(dst, value)`。
- 身份：`collections_layout::codegen_build` 与 `closures_layout::codegen_build` 的 `include_bytes!` 列表各加 `lower_arith.rs`，使 helper 单独变更也能使 A09'/A10' codegen 身份失效；身份值变化按合同记录。
- D15 仅改注释/description，不需架构改动。

## 3. 技术栈

Rust（edition 2021，ACE 独立 workspace）、Cargo 固定组、Markdown。仅 auto-lang 仓。

- 唯一实施树：`D:/autostack/.wt/lang-1069/auto-lang`，分支 `plan-1069-dev`（开工时创建并确认）。
- 材料暂存：`D:/autostack/plan-artifacts/plan-1069/`（owner: PLAN-1069；用途: 重构前后 COFF 指纹与门禁输出摘录；在途；merge 折叠入 Plan 后整目录删除）。

## 4. 需求分析与背景调查

**授权**：用户 2026-11-17 指示“继续，直到 1038 全部完成”；1038 已归档（`archive/1038-…md`，r3，6/6）并冻结 D04 合同（§9 T-03）与 D15 的 5 处清单。本包按该合同实施，不扩大范围；D04 的代码实施与 D15 的文字更正属该合同的直接后继。

**源身份（开工 T-01 重核，当前读数）**：

| 路径 | 行数 | SHA256 |
|---|---|---|
| experimental/ace/crates/native/src/collections_lower.rs | 1880 | a65d909593170318e5763dcdd5ff19b2dd0701936c056742075bd8179d607dc8 |
| experimental/ace/crates/native/src/closures_lower.rs | 1006 | edd1820dc3ebcff91982e929ffa59742e4b49521b55429feb92f74445ea91fe6 |
| experimental/ace/crates/native/src/collections_layout.rs | 229 | （T-01 记） |
| experimental/ace/crates/native/src/closures_layout.rs | 254 | （T-01 记） |
| experimental/ace/crates/core/src/lib.rs | 576 | 925d9503a0f47d665885c4f3bafb1d27c8ad2c23ac330ac567a9809e718e819d |
| experimental/ace/crates/av-core/src/lib.rs | 76 | 9f73f8abe09882ee304481bb7dffcc3d7da4f25162e423c47cc0bf9eb65b59fa |
| experimental/ace/crates/native/src/lib.rs | 115 | 1e3306a8eb31f82d9c6e36398aa8667bfa2d37da6a064bf2c2da45f76c9401b3 |
| docs/specs/auto-ac/native-backend-and-retirement.md | 120 | 3547178050b20180ac5c89afd76ecaf30219d8a84aa94300cd37db8218ea6ab7 |
| docs/specs/auto-mir/native-collections-variants.md | — | （T-01 记） |
| docs/specs/auto-mir/native-closures-resource-abi.md | — | （T-01 记） |

**已核事实（来自 1038 §9 T-01/T-02，本包开工需复核未漂移）**：

1. 两 `fn binary` 的 body 行集合逐字相同、hash `4be783ee…`；差异仅 `BinOp::Num` 分派、返回类型与 `binary_num`/`word_to` 的存在。
2. i32 陷阱 oracle 当前缺失：native 测试仅用 `AddI32`；B14/B15 经 `shadow_tier` 的 `Prof::Scalar`（scalar 车道）；i64 陷阱在 `numeric_codegen_tier`。
3. 两个 `codegen_build()` 以 `include_bytes!` 哈希 `*_lower.rs`/`*_layout.rs`/`fingerprint.rs`/`emit.rs`/`symbols.rs`/`Cargo.lock`；`core`/`av-core`/`native` 的 `lib.rs` 不在其中。
4. 直接 consume 这两条车道的测试入口：`ace-native` 组（`collections_typed_tier`、`collections_driver_tier`、`collections_source_tier`、`closures_typed_tier`、`closures_native_tier`），`ace-av`（v10_layers_972 使用 `WrapperKnobs`）。

**约束**：Windows 不跑 Linux/WSL 腿（2026-11-14 裁定）；新增 case 每例 ≤10 秒；不吃 release；不动主检出他人 WIP。

## 5. 详细设计

### 5.1 D04 抽取与不变量

| 不变量 | 内容 | 验证 |
|---|---|---|
| INV-1 发射顺序 | helper 内 `iconst`/`icmp`/`guard` 调用序列与现实现逐行对应；`guard` 的 next 块创建在 `brif` 之前 | 重构前后固定单元 COFF sha256 相同（§6 CASE-09a/b） |
| INV-2 trap 语义 | 除零 `-31`（Div/Rem）、MIN/-1 与 i32 溢出 `-30`；错误码经 `mn_coll_abort` 打印为 `MN-COLL-TRAP code=<unsigned_abs>`（31/30） | ARITH 反例断言 |
| INV-3 宽度 | 比较结果 `uextend(I32)`；算术前 `sextend(I64)`；结果 `ireduce(I32)`；`scalar_to` 的 `uextend(I64)`+store 不变 | COFF 等价 + ARITH 正常值 |
| INV-4 面不 widen | closures 仍无 `BinOp::Num`/`word_to` 分支，`binary_num` 仅 collections | 编译期（类型/分支结构）+ 代码 diff |
| INV-5 清理不变 | frame/capture/`discard_guard`/`mn_coll_*` 调用面零 diff | `git diff` 检查 + `closures_native_tier` clean-after-trap |
| INV-6 身份 | 两 `codegen_build()` 增 `include_bytes!("lower_arith.rs")`；`ABI/CODEGEN/PROFILE/RECIPE` 串不变 | 代码检查 + 记录新旧身份值 |

### 5.2 D04 调用点改动

- `collections_lower::Lower::binary`（1412–1471）：`BinOp::Num` 分派保持；其后 `let a = self.scalar(lhs); let b = self.scalar(rhs);` → `crate::lower_arith::emit_i32_binary(&mut self.fb, self.trap, op, a, b)`；`Some(v)` → `self.scalar_to(dst, v)`；`None` → 与原实现同的 `unreachable!()`（原 match 的 `_ => unreachable!()`）。
- `closures_lower::Lower::binary`（954–1005）：同形，但无 Num 分派、无 `Result`。
- `collections_layout.rs` / `closures_layout.rs` `codegen_build()`：各加一行 `include_bytes!("lower_arith.rs").as_slice(),`。

### 5.3 D15 五处更正（只改文字，不改依赖/代码/身份）

| # | 位置 | 现状 | 更正规则 |
|---|---|---|---|
| D15-1 | `core/src/lib.rs:4–6` | “…→ native lowering (Cranelift) → COFF…” | 改为 frontend/verify/link 职责描述，注明 native lowering 已退役到 `auto-mir-lower` + `ac-mn`（引用 Spec §1） |
| D15-2 | `core/src/lib.rs:8–9` | “must not depend on any auto-lang / auto-atom / auto-val product crates” | 改为“不依赖产品运行时/UI；共享 syntax 叶子（ace-syntax-*} 及其 auto-atom/auto-val 数据依赖）是允许方向” |
| D15-3 | `av-core/src/lib.rs:16–17` | “zero external dependencies” | 改为“正常依赖 ace-foreign-contract 与 serde_json；无 auto-lang/UI/man/Cranelift 依赖” |
| D15-4 | `native/src/lib.rs:1–10` | “PLAN-818 probe… parity loop against … the old HIR-native reference… Nothing here feeds product paths” | 改为当前角色：MIR-native 唯一后端路径（Spec §1），旧 HIR-native 参照已退役（Spec §7），消费方 abi/av/case-worker/comptime/m2-build；保留“私有 test ABI ≠ AutoABI 797” |
| D15-5 | `core/Cargo.toml:6` | description 称 “Windows x64 native AOT” | 改为 frontend/verify/link 描述；不声称构建面 |

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-00 | 无（内部机制抽取 + 注释更正） | 无 | 不改 canonical Specs：i32 算术仍属 `collections_lower`/`closures_lower` 的 family 内部机制，行为/ABI/接口/身份轴不变；D15 只让注释与既有 Spec 一致 | 六项决定的规范增量归各自实现包另评；本包无规则变化 | AC-01..04 |

## 6. 测试设计

**不变量与代表性输入**：i32 车道 4 种状态 × 2 family；错误码必须可区分（-30 溢出/MIN%-1 vs -31 除零）；正常值必须证明非误 trap。

| Case ID | AC/work owner | 区分性输入 | 实际入口 | 独立 oracle | 状态 |
|---|---|---|---|---|---|
| ARITH-01 | AC-02 / T-03 | collections：`DivI32` 正常（84/2→42） | `collections_typed_tier`（hand-built MIR + 真 PE link/run） | 退出 0 且返回 42 | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-02 | AC-02 / T-03 | collections：`DivI32` 除零 | 同上 | 退出 70 且 stderr `MN-COLL-TRAP code=31` | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-03 | AC-02 / T-03 | collections：`RemI32` MIN%-1 | 同上 | 退出 70 且 `code=30` | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-04 | AC-02 / T-03 | collections：`AddI32` 溢出（i32::MAX+1） | 同上 | 退出 70 且 `code=30` | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-05 | AC-02 / T-03 | closures：`DivI32` 正常（42） | `closures_typed_tier`（hand-built closure-profile MIR + 真 PE） | 退出 0 且返回 42 | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-06 | AC-02 / T-03 | closures：`DivI32` 除零 | 同上 | 退出 70 且 `code=31` | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-07 | AC-02 / T-03 | closures：`RemI32` MIN%-1 | 同上 | 退出 70 且 `code=30` | 已实施：真实 PE 断言通过（§9 T-03） |
| ARITH-08 | AC-02 / T-03 | closures：`MulI32` 溢出 | 同上 | 退出 70 且 `code=30` | 已实施：真实 PE 断言通过（§9 T-03） |

| CASE-09 | AC-01 / T-02/T-03 | 固定代表单元（collections V02 手写单元 + closures K01 单元） | 临时 probe：`emit_unit_coff` 字节 sha256（重构前/后） | 两次 sha256 相同 | 已实施：前后 sha256 相同（§9 T-02/T-04） |
| REG-01 | AC-03 / T-03 | 既有 trap 清理/帧事件 | `closures_native_tier::assert_clean_after_trap`、`collections_seam_tier`/`collections_resources_tier` | 原断言不回归 | 已实施：组门内原断言不回归（§9 T-03） |
| REG-02 | AC-03 / T-03 | i64 typed 面 | `numeric_codegen_tier` | 原断言不回归 | 已实施：组门内原断言不回归（§9 T-03） |

注：运行时以 `unsigned_abs` 打印 trap 码（`collections-runtime/src/lib.rs:2056`），源码 -31/-30 在 stderr 显示为 31/30；断言按显示值写，语义仍是“除零 vs 溢出/MIN%-1”两类可区分错误。

**变异负控**：至少 2 例证明反例有区分力——(a) 把除零 guard 的 `-31` 改成 `-30`，ARITH-02/06 必须红；(b) 去掉 i32 上界检查，ARITH-04/08 必须红。变异只在本地临时验证，不提交。

**门禁**：`cargo t ace-native`（Windows；Linux/WSL 腿按 2026-11-14 裁定排除、不记 PASS）。D15 另跑 `cargo check -p ace-core -p ace-av-core -p ace-native`（ACE workspace）确认注释/描述改动不破坏构建。新增 case 每例 ≤10 秒；不跑 ace-perf、不跑 `cargo t all`。

**角色**：work 与 review 使用同一实施树；review 由独立 context（subagent，无指定模型）执行 AC-01..04 与反例复核；merge 收尾。同一树顺序写入，禁止并发写。

## 7. 验收标准

- [x] AC-01 / work：`lower_arith.rs` 抽取完成，两 `Lower::binary` 委托且 `BinOp::Num` 分派/返回类型保持；固定单元重构前后 COFF sha256 相同（CASE-09 证据）。（§9 T-02/T-04）
- [x] AC-02 / work：ARITH-01..08 在真实 PE 全绿（正常值/退出 70 与 31/30 断言），且 2 个变异负控按预期变红。（§9 T-03）
- [x] AC-03 / work：两 `codegen_build()` 已含新文件、身份新旧值记录；closures typed 面未 widen；frame/capture/cleanup 零 diff；REG-01/02 不回归。（§9 T-03/T-04）
- [x] AC-04 / work：D15 五处与当前 Cargo/Spec 一致；`cargo check` 通过；无依赖/代码/身份变更。（§9 T-05）
- [x] AC-05 / review：独立核 AC-01..04、§6 反例真实性（含变异负控记录）、健康/格式/来源漂移；结论绑定 revision 与提交。（独立 context subagent reviewer-1069：pass，见 §9 T-06）
- [x] AC-06 / merge：冻结 `ace-native` 门与同身份证据、ff 合入、归档与清树/分支；输出路径对账。（§9 T-07）

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1038 合同与当前源 | 重核 §4 全表身份（含两个 layout 与两份 auto-mir Spec）并记入 §9；确认 body 50 行与 i32 缺口未漂移；建立 `plan-artifacts/plan-1069/`。AC-01（§9 T-01：14 项身份一致、body hash 4be783ee…、仅 AddI32 缺口成立） |
| T-02 / work | [x] T-01 | 基线 COFF 指纹：临时 probe 对固定代表单元 `emit_unit_coff` 取 sha256 并落 plan-artifacts；probe 不入库。CASE-09（§9 T-02：collections 95a7e194…、closures 43a90cd8…） |
| T-03 / work | [x] T-02 | 新增 `lower_arith.rs`、两调用点委托、两 layout `include_bytes!`；`cargo check -p ace-native`；8 个 ARITH 反例入既有 typed tier；`cargo t ace-native` 全绿；变异负控。AC-01/02/03（§9 T-03：组 PASS 301 executed / wall 34.1s；NC-1/NC-2 按预期红） |
| T-04 / work | [x] T-03 | 重构后 COFF 指纹比对（CASE-09），身份新旧值记录，REG-01/02 复核，diff 审查（cleanup/`word_to`/Num 面零改）。AC-01/03（§9 T-04：COFF 同值；身份 old→new 已记；diff 无 cleanup/Num 改动） |
| T-05 / work | [x] T-03 | D15 五处文字更正 + `cargo check` 三包；记录 diff。AC-04（§9 T-05：4 文件 5 处；check exit 0；无依赖/身份变更） |
| T-06 / review | [x] T-01..05 | 独立 context 复核 AC-01..04 与反例/变异证据；`git diff` 健康检查。AC-05（§9 T-06：pass；R-1069-01/02 已补/正，R-1069-04 已修@49933c81d，其余为非阻塞观察） |
| T-07 / merge | [x] T-06 pass | 冻结门复用、ff 合入、归档、guard+清树、输出对账。AC-06（§9 T-07：ff 落 `391934620`；归档+清理+registry 已完成） |

## 9. 复审记录

2026-11-17 stage:new / PLAN-1069 r1 / outcome:pass（可执行规划；非代码/运行 PASS） / next: work。合同取自已归档 1038 §9 T-02 D04 与 D15 清单；新增/改动路径已对当前源核到行号；Spec 增量 SD-00（无规则变化）有理由；未建树、未改实现/测试/配置/Spec、未运行测试/构建。

### T-01 开工重核与 COFF 基线（2026-11-17，work）

实施树 `D:/autostack/.wt/lang-1069/auto-lang`（branch `plan-1069-dev`，base `bfee96a12`）。身份全部与 1038 §9 T-01 一致（无漂移）：

| 路径 | 行数 | SHA256 |
|---|---|---|
| native/src/collections_lower.rs | 1880 | a65d909593170318e5763dcdd5ff19b2dd0701936c056742075bd8179d607dc8 |
| native/src/closures_lower.rs | 1006 | edd1820dc3ebcff91982e929ffa59742e4b49521b55429feb92f74445ea91fe6 |
| native/src/collections_layout.rs | 229 | a1db3c769c8133e6e370f04db9285af9ddc842a5969215024cfc70656cf2fed4 |
| native/src/closures_layout.rs | 254 | 529ab00751c3d859b4405958e3f40d0d8cf3f6fecb62da5e5722aed803679655 |
| native/src/emit.rs | 894 | 5616025ba43668f3d0dfa5ab62506d718aed48e79fed57ba6de34c444a950e31 |
| native/src/fingerprint.rs | 564 | 630e9319f5b9e72f8a988e6cd0291099b981529999267889d083022b30cc9fd3 |
| native/src/symbols.rs | 77 | 90cbbae56037f6ef9375cccea5bafa5835dad5bb168b880f52a8b274af94f000 |
| core/src/lib.rs | 576 | 925d9503a0f47d665885c4f3bafb1d27c8ad2c23ac330ac567a9809e718e819d |
| av-core/src/lib.rs | 76 | 9f73f8abe09882ee304481bb7dffcc3d7da4f25162e423c47cc0bf9eb65b59fa |
| native/src/lib.rs | 115 | 1e3306a8eb31f82d9c6e36398aa8667bfa2d37da6a064bf2c2da45f76c9401b3 |
| core/Cargo.toml | 57 | 595b6f64991ef4d3450f171a50b30d27c2adfa322cd8401f028d91da48b3584f |
| docs/specs/auto-ac/native-backend-and-retirement.md | 120 | 3547178050b20180ac5c89afd76ecaf30219d8a84aa94300cd37db8218ea6ab7 |
| docs/specs/auto-mir/native-collections-variants.md | 119 | 345140109b07e645e825d75d3458057f1fc64cd349315125033ca03d0c19c751 |
| docs/specs/auto-mir/native-closures-resource-abi.md | 54 | cd1bd703b87aabcb9e184e0a0e8a6d247dc60f4a4d29356d58bee75c555fc936 |

- 两 `fn binary` body 行集合仍逐字相同：`coll==clo: True`，SHA256=`4be783eee99beb7a26c4578954e44d46dfd21f1e33c6361674a92a4d9c687f17`。
- i32 oracle 缺口复核：`ace-native` 测试内 `BinOp::*` 仅 `AddI32 x7`（无 Div/Rem/比较/溢出真实 PE case）。
- 材料 scope：`D:/autostack/plan-artifacts/plan-1069/`（owner 1069；在途）。

### T-02 COFF 基线指纹（2026-11-17，work）

临时 probe（`native/examples/d04_coff_probe.rs`，不入库；源与输出留 plan-artifacts）对固定 MIR 单元（Div/Rem/Add/Lt + 比较分支，collections 与 closures 各一）`emit_unit_coff` 取 SHA256：

```
collections bytes=1031 sha256=95a7e19424c129595eca99133215a3b4707cd4885ae78f09465ed57447522c3c
closures    bytes=1031 sha256=43a90cd8ec6910e1c4473d4dfb716c0044534f39868ffdffccd8fbb12457af2e
```


### T-03 i32 共享发射与 8 个反例（2026-11-17，work）

- 新增 `experimental/ace/crates/native/src/lower_arith.rs`：`pub(crate) fn emit_i32_binary(fb, trap, op, a, b) -> Option<Value>`，逐行承接原 50 行机制（6 比较、除零 -31、MIN/-1 -30、i64 中间值+i32 上下界 -30、`ireduce`）；`None` 仅表示非 i32 车道。
- `collections_lower::Lower::binary` 保留 `BinOp::Num`→`binary_num` 分派与 `Result` 签名；`closures_lower::Lower::binary` 保留 `()`；两者改为委托 helper。
- `collections_layout::codegen_build` / `closures_layout::codegen_build` 的 `include_bytes!` 各加 `lower_arith.rs`。
- 反例落点（既有入口、真实 PE、各 4 例）：`collections_typed_tier::i32_checked_arithmetic_keeps_values_and_trap_codes`、`closures_typed_tier::i32_checked_arithmetic_keeps_values_and_trap_codes`；断言正常值 42、除零 70/`code=31`、MIN%-1 与溢出 70/`code=30`（运行时打印 `unsigned_abs`）。
- 门禁：`cargo t ace-native` **PASS**（代码内容=提交 9033e22a0）：`executed=301`、`exec_wall=34.055s`、`prep=4.4s`、`gate=81.8s`、`not_run=[]`、`errors=[]`（`run-ilwbaxfx`）；reviewer 在冻结提交上重跑 `run-fmfjw1g2`：`prep=3.7s`、`exec=47.3s`、`gate=51.3s`、`source_state=cb8ef950…`（可由干净提交树复算；先前 run 的 source_state 早于 D15 注释与 probe 清理，故以本条绑定冻结态）。新增两 case 计时 `1.29s`/`1.73s`（`run-__0wm5fo` 过滤跑；reviewer 复测 2.64–3.34s / 4.15–5.72s），均 ≤10s。组内含 REG-01/REG-02 原入口（closures_native_tier trap 清理、collections seam/resource、numeric_codegen_tier i64）无回归。
- 变异负控（临时，已还原）：NC-1 除零 guard 由 -31 改 -30 → 两 case 红（`left: Some(30) right: Some(31)`）；NC-2 删除 i32 上界检查 → collections 返回 `-2147483648`、closures 返回 `0`（应 70）。还原后 `lower_arith.rs` 与正确副本 sha256 一致。
- 健康：`rustfmt --check` 对本次改动文件全通过；无新增编译器 warning（余为既有 dead_code/unused）。var 面：`git diff` 仅两处 binary 收窄、两 layout 各 1 行、两 tier 各 +1 case。

### T-04 COFF 等价、身份与面审查（2026-11-17，work）

- CASE-09 前后比对（固定单元 collections/closures 各一，Div/Rem/Add/Lt + 比较分支）：重构后 `collections 95a7e19424c129595eca99133215a3b4707cd4885ae78f09465ed57447522c3c`（1031B）、`closures 43a90cd8ec6910e1c4473d4dfb716c0044534f39868ffdffccd8fbb12457af2e`（1031B），与 T-02 基线**逐字节相同**。
- codegen 身份（recipe 用手工重算验证：new-manual == 真实 `codegen_build()`）：
  - collections old=`6c86c37a877f74c6316372a006d55e7dbcbabf6aad544500de5865d0c904b148` → new=`19f9aa834c2308f67972f806922f8288e7902ae21060ad244627e482c69c7649`；
  - closures old=`34b0fd0de41f9f202db6c832fb16ffa95e853946e1198e1f1d7ff512006c2dc4` → new=`cf4441a7dce2f1b0ef83ac727ee073a415116c72cb52b2b7ce5c0723d171b617`。
  - `ABI`/`CODEGEN`/`PROFILE`/`RECIPE` 常量与 manifest 身份未改（仅 A09'/A10' build 身份按设计失效一次）。
- 面审查：helper 不含 `word_to`/`BinOp::Num`/frame/capture/cleanup；closures 仍无 typed-numeric 分支；`codegen_identity` probe 与 `d04_coff_probe` 已从树内删除（副本与输出留在 plan-artifacts）。

### T-05 D15 五处文档更正（2026-11-17，work）

`core/src/lib.rs`（pipeline 与依赖方向两处）、`av-core/src/lib.rs`（外部依赖事实）、`native/src/lib.rs`（角色/旧参照/消费方）、`core/Cargo.toml`（description）共 5 处；`cargo check -p ace-core -p ace-av-core -p ace-native` **exit 0**；改动不涉及依赖、代码路径、`include_bytes!` 身份输入或 wire 身份；`rustfmt --check` 通过。

### 材料清单（plan-artifacts/plan-1069）

`coff-baseline.txt`、`coff-after.txt`、`codegen-identity.txt`、`lower_arith.correct.rs`、`d04_coff_probe.rs`、`d04_identity_probe.rs`、`baseline/`（8 份 HEAD 字节，用于身份重算）。
### T-06 独立复审（2026-11-17，review，scope: final）

`stage: review | review_scope: final | reviewer_identity: 独立 context subagent reviewer-1069（未采信 work 摘要；自行读 diff、重跑 gate、重跑 NC-1、重算身份与 COFF 对比；限制=同一仓库、同一实施树） | plan_id: PLAN-1069 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1069/auto-lang | branch: plan-1069-dev | reviewed_commit: 9033e22a0（+ 注释级 follow-up 49933c81d，仅改 core/src/lib.rs 注释 2 行） | base_commit: bfee96a12 | acceptance_results: AC-01 pass / AC-02 pass / AC-03 pass / AC-04 pass | next: merge`

复核要点：

- AC-01：把 base `bfee96a12:closures_lower.rs` 的旧 47 行体与新 helper 体做归一化 token 比较——除 `self.fb→fb`、`self.i32→i32_const(fb,`、`self.guard→guard(fb,trap,` 与唯一语义替换 `_ => unreachable!()`→`_ => return None`（两个调用点同点 `unreachable!()`）外**完全一致**；guard 块序 `create_block→brif→switch_to_block` 不变；签名/`BinOp::Num` 分派/`word_to` 归属保持；helper `pub(crate)` 且 `mod lower_arith;` 私有。COFF：reviewer 用记录 probe 在**提交态与 base 态**各跑一次，两族均 `95a7e194…`/`43a90cd8…`（1031B），与基线/后置文件一致。
- AC-02：8 例确为真实 PE（`link_and_run`），断言 42 / 70+`code=31` / 70+`code=30`，trap 显示值来自 `collections-runtime/src/lib.rs:2056`（`unsigned_abs`）；reviewer 重跑 NC-1（-31→-30）两 case 红为 `left Some(30) right Some(31)`，随后 `git checkout` 还原（文件 sha256 与提交一致、树干净、未提交）。
- AC-03：reviewer 独立重算两 `codegen_build()`（blake3 recipe）得 `6c86c37a…→19f9aa83…` / `34b0fd0d…→cf4441a7…`，与记录一致；`include_bytes!` 接线在 `collections_layout.rs:18`/`closures_layout.rs:29`；唯一代码 diff 为两 `binary` + 各 1 行 include；REG-01/02 在组门内绿。
- AC-04：逐条对 Cargo/Spec 核 5 处；`cargo check -p ace-core -p ace-av-core -p ace-native` exit 0；无依赖/ABI/身份变化。
- 健康：改动文件 rustfmt 全通过；无新增 warning；未提交 probe（examples 目录只剩既有 `collections_inventory.rs`）。

发现与处置（均非阻塞）：

- **R-1069-01（中低，证据口径）**：§9 T-03 混用了两次运行的指标（34.1s 与 16.8s 来自不同 run）。已按 reviewer 数据分列 `run-ilwbaxfx`（prep 4.4s / exec 34.055s / gate 81.8s）与 `run-fmfjw1g2`（prep 3.7s / exec 47.3s / gate 51.3s）。
- **R-1069-02（中低，证据绑定）**：原 gate run 的 `source_state` 早于 D15 注释与 probe 清理，不能直接绑定冻结提交。已把 reviewer 在冻结提交上重跑的 `run-fmfjw1g2`（`source_state=cb8ef950…`，可由干净树复算）写入 §9 T-03。
- **R-1069-03（提示）**：§6 注释行把表格切断。已把 trap 显示说明移到 REG-02 行之后。
- **R-1069-04（提示，已修）**：D15-2 原写“两个 syntax 叶子都使用 auto-atom/auto-val”；实际只有 grammar 叶子（`ace-syntax-grammar`）依赖它们。已改为“the grammar leaf in turn uses…”，提交 `49933c81d`（仅注释 2 行，`cargo check` exit 0）。
- **R-1069-05（覆盖观察，非本包门）**：永久 PE 反例覆盖 Div/Rem/Add/Mul；i32 比较臂与 SubI32 仅由一次性 COFF probe 覆盖（该 probe 单元含 LtI32/RemI32/AddI32）。作为后续可选补强，不构成本包 AC 缺口。
- **R-1069-06（可维护性观察）**：`lower_arith` 自带 `guard`/`i32_const`，与两家族仍保留的同名 helper 形成三份；后续 guard 语义变更需三处同步。记录为债务候选，不在本包收敛（避免扩大首片）。

Limitations：reviewer 未重跑 NC-2（与溢出断言一致）、未跑 Linux/ace-perf/`cargo t all`（按裁定/合同排除）；COFF “before” 通过临时恢复 base 两文件取得（随后 `git checkout` 复原）。

### T-07 合并、归档与清理（2026-11-17，merge）

`stage: merge | plan_id: PLAN-1069 | plan_revision: r1 | outcome: pass | completion_kind: delivered | review: final pass（§9 T-06） | delivery_commits: 72a095aa6（实现，ff 落 master）+ 391934620（R-1069-04 注释 follow-up） | canonical: SD-00 无规范增量（内部机制抽取+注释更正；approved no-impact） | archive: docs/plans/archive/1069-ace-1038-d04-i32-arith-and-d15-comments.md（status: archived） | cleanup: 实施树/分支/组目录删除，无 plan-artifacts 持有（证据已折入本节） | next: done`

| checkpoint | evidence |
|---|---|
| prepared | reviewed r1 @9033e22a0；实现 diff 11 文件；SD-00 无增量；归档目标路径 |
| landed | 分支基于 `8dc3db43c` rebase 后 `git merge --ff-only plan-1069-dev` → master `391934620`（无 merge commit）。reviewed→delivered 映射：`9033e22a0→72a095aa6`、`49933c81d→391934620`，`git range-diff` 2/2 patch 完全相等（两次 rebase：`008149ac5`、`8dc3db43c`）；两次 rebase 均为 docs-only master 推进，`experimental/ace` 无变化 |
| canonical_verified | 交付提交内 `docs/specs/` 零改动（`git show --stat` 核）；approved no-impact：i32 算术仍是两 family 内部机制，ABI/接口/身份轴不变；D15 只让注释与既有 Spec 一致 |
| archived | 计划已 `git mv` 至 `docs/plans/archive/` 并置 `status: archived`；归档位置本地链接全部可解析；`KNOWN-DEBT-AND-RISKS.md` F04/F15 状态更新为“已修@1069” |
| cleaned | `wt-guard.ps1` 报告 clean 后删除 `D:/autostack/.wt/lang-1069/auto-lang`、分支 `plan-1069-dev` 与空组目录；`plan-artifacts/plan-1069` 摘要折入本节后整目录删除；registry 增补 1069 关闭条目（无持有） |

门禁与观察：

- **测试**：rebase 后冻结树（`391934620`）`cargo t ace-native` **PASS**：`executed=329`、`exec_wall=56.3s`、`not_run=[]`、`errors=[]`（`run-fet4eakv`，`source_state=5f8b5af463a631675a5c2fd6f269d4c55582e17961ed3ecb31256f97f6b63e29`）；该 source_state 在两次 docs-only rebase 下不变（driver 只含代码/配置路径）。新增两 case 计时 1.29s/1.73s（过滤跑）；全组含 REG-01/02 无回归。
- **变异负控**：NC-1（-31→-30）两 case 红；NC-2（去上界检查）collections 返回 -2147483648、closures 返回 0（应 70）；均已还原。
- **部署新鲜度**：无生产产物受影响 → 无需 rebuild/restart（记录为“无需刷新”）。
- **批量回归到期**：`.last-batch-regression.json` last_covered 1064 @2026-11-16T10:04Z；1069%5=4 且距今约 26h<48h → 未到期，不触发 `cargo t all`。
- **证据位置**：`D:/autostack/plan-artifacts/plan-1069/`（probe 源与输出、baseline 字节）在折入后已删除，不设持有。

后续：1038 D13（mir_verify init 私有边界）按 1038 §9 次序列另立后继包（未在本包实施）。
## 10. 待澄清事项

- ARITH 反例落点为既有 hand-built MIR typed tier（真实 PE），不改源码语料/不改产品 fixture；若 T-03 发现该入口无法表达某状态（例如 closures profile 拒绝非闭包函数体），按合同改为该 tier 已支持的最接近 `Div/Rem` 形态并在 §9 记录，不删反例。
- 本包不含 D13、不含 Linux 腿；二者分别归后继包与专用环境。







