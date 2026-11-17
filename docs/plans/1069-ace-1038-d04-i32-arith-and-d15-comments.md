---
plan_id: PLAN-1069
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: ACE i32 校验算术单一发射切片与 1038 文档更正
author: [lead]
created_at: 2026-11-17
updated_at: 2026-11-17

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [ace, auto-ac]
current_step: 0
total_steps: 6
plan_revision: 1
depends_on: [PLAN-1038]
---

# [PLAN-1069] ACE i32 校验算术单一发射切片与 1038 文档更正

## 0. 变更摘要

本包执行已归档 [PLAN-1038](../archive/1038-ace-reuse-candidates-and-cost-decisions.md) §9 T-02 六项决定中的两项 go：

- **D04（首个有界切片）**：把 `native/src/collections_lower.rs:1420–1469` 与 `closures_lower.rs:955–1004` 两处**50 行逐字相同**的 i32 比较/校验四则发射抽成 `ace-native` crate 私有 `lower_arith.rs`，两 `Lower::binary` 委托；并按 1038 §9 T-03 合同补当前缺失的 **i32 陷阱/正常值 native PE 反例**（Div/Rem/MIN%-1/溢出）。
- **D15（纯文档）**：更正 `core`/`av-core`/`native` 三处头注释与 `core/Cargo.toml` description 共 5 处事实漂移。

D13（mir_verify init 私有边界）按 1038 的“次序列”另立后继包，本包不实施。不合并 layout/cache/wrapper/frame、不改 ABI/公共接口/依赖、不跑 ace-perf。

## 1. 目标

- **G1（D04）**：i32 校验算术（6 比较、Add/Sub/Mul 的 i64 中间值+范围检查、Div/Rem 的除零与 MIN/-1 guard、trap 码 -30/-31）只有一个发射 owner；两 family 的已发代码**逐字节不变**（COFF 等价），closures 的 typed-numeric 拒绝面不 widen。
- **G2（D04 oracle）**：补齐 i32 车道在真实 PE 上的永久反例：正常值、除零、MIN%-1、溢出，collections 与 closures 各 4 例，断言正常值/退出 70 与 `MN-COLL-TRAP code=-31/-30`，并有可区分错误实现的变异负控。
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
| INV-2 trap 语义 | 除零 `-31`（Div/Rem）、MIN/-1 与 i32 溢出 `-30`；错误码经 `mn_coll_abort` 转 `MN-COLL-TRAP code=` | ARITH 反例断言 |
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
| ARITH-01 | AC-02 / T-03 | collections：`DivI32` 正常（84/2→42） | `collections_typed_tier`（hand-built MIR + 真 PE link/run） | 退出 0 且返回 42 | 待实施 |
| ARITH-02 | AC-02 / T-03 | collections：`DivI32` 除零 | 同上 | 退出 70 且 stderr `MN-COLL-TRAP code=-31` | 待实施 |
| ARITH-03 | AC-02 / T-03 | collections：`RemI32` MIN%-1 | 同上 | 退出 70 且 `code=-30` | 待实施 |
| ARITH-04 | AC-02 / T-03 | collections：`AddI32` 溢出（i32::MAX+1） | 同上 | 退出 70 且 `code=-30` | 待实施 |
| ARITH-05 | AC-02 / T-03 | closures：`DivI32` 正常（42） | `closures_typed_tier`（hand-built closure-profile MIR + 真 PE） | 退出 0 且返回 42 | 待实施 |
| ARITH-06 | AC-02 / T-03 | closures：`DivI32` 除零 | 同上 | 退出 70 且 `code=-31` | 待实施 |
| ARITH-07 | AC-02 / T-03 | closures：`RemI32` MIN%-1 | 同上 | 退出 70 且 `code=-30` | 待实施 |
| ARITH-08 | AC-02 / T-03 | closures：`MulI32` 溢出 | 同上 | 退出 70 且 `code=-30` | 待实施 |
| CASE-09 | AC-01 / T-02/T-03 | 固定代表单元（collections V02 手写单元 + closures K01 单元） | 临时 probe：`emit_unit_coff` 字节 sha256（重构前/后） | 两次 sha256 相同 | 待实施（probe 不入库） |
| REG-01 | AC-03 / T-03 | 既有 trap 清理/帧事件 | `closures_native_tier::assert_clean_after_trap`、`collections_seam_tier`/`collections_resources_tier` | 原断言不回归 | 待实施（复用原入口） |
| REG-02 | AC-03 / T-03 | i64 typed 面 | `numeric_codegen_tier` | 原断言不回归 | 待实施（复用原入口） |

**变异负控**：至少 2 例证明反例有区分力——(a) 把除零 guard 的 `-31` 改成 `-30`，ARITH-02/06 必须红；(b) 去掉 i32 上界检查，ARITH-04/08 必须红。变异只在本地临时验证，不提交。

**门禁**：`cargo t ace-native`（Windows；Linux/WSL 腿按 2026-11-14 裁定排除、不记 PASS）。D15 另跑 `cargo check -p ace-core -p ace-av-core -p ace-native`（ACE workspace）确认注释/描述改动不破坏构建。新增 case 每例 ≤10 秒；不跑 ace-perf、不跑 `cargo t all`。

**角色**：work 与 review 使用同一实施树；review 由独立 context（subagent，无指定模型）执行 AC-01..04 与反例复核；merge 收尾。同一树顺序写入，禁止并发写。

## 7. 验收标准

- [ ] AC-01 / work：`lower_arith.rs` 抽取完成，两 `Lower::binary` 委托且 `BinOp::Num` 分派/返回类型保持；固定单元重构前后 COFF sha256 相同（CASE-09 证据）。
- [ ] AC-02 / work：ARITH-01..08 在真实 PE 全绿（正常值/退出 70 与 `-31/-30` 断言），且 2 个变异负控按预期变红。
- [ ] AC-03 / work：两 `codegen_build()` 已含新文件、身份新旧值记录；closures typed 面未 widen；frame/capture/cleanup 零 diff；REG-01/02 不回归。
- [ ] AC-04 / work：D15 五处与当前 Cargo/Spec 一致；`cargo check` 通过；无依赖/代码/身份变更。
- [ ] AC-05 / review：独立核 AC-01..04、§6 反例真实性（含变异负控记录）、健康/格式/来源漂移；结论绑定 revision 与提交。
- [ ] AC-06 / merge：冻结 `ace-native` 门与同身份证据、ff 合入、归档与清树/分支；输出路径对账。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [ ] 1038 合同与当前源 | 重核 §4 全表身份（含两个 layout 与两份 auto-mir Spec）并记入 §9；确认 body 50 行与 i32 缺口未漂移；建立 `plan-artifacts/plan-1069/`。AC-01 |
| T-02 / work | [ ] T-01 | 基线 COFF 指纹：临时 probe 对固定代表单元 `emit_unit_coff` 取 sha256 并落 plan-artifacts；probe 不入库。CASE-09 |
| T-03 / work | [ ] T-02 | 新增 `lower_arith.rs`、两调用点委托、两 layout `include_bytes!`；`cargo check -p ace-native`；8 个 ARITH 反例入既有 typed tier；`cargo t ace-native` 全绿；变异负控。AC-01/02/03 |
| T-04 / work | [ ] T-03 | 重构后 COFF 指纹比对（CASE-09），身份新旧值记录，REG-01/02 复核，diff 审查（cleanup/`word_to`/Num 面零改）。AC-01/03 |
| T-05 / work | [ ] T-03 | D15 五处文字更正 + `cargo check` 三包；记录 diff。AC-04 |
| T-06 / review | [ ] T-01..05 | 独立 context 复核 AC-01..04 与反例/变异证据；`git diff` 健康检查。AC-05 |
| T-07 / merge | [ ] T-06 pass | 冻结门复用、ff 合入、归档、guard+清树、输出对账。AC-06 |

## 9. 复审记录

2026-11-17 stage:new / PLAN-1069 r1 / outcome:pass（可执行规划；非代码/运行 PASS） / next: work。合同取自已归档 1038 §9 T-02 D04 与 D15 清单；新增/改动路径已对当前源核到行号；Spec 增量 SD-00（无规则变化）有理由；未建树、未改实现/测试/配置/Spec、未运行测试/构建。

## 10. 待澄清事项

- ARITH 反例落点为既有 hand-built MIR typed tier（真实 PE），不改源码语料/不改产品 fixture；若 T-03 发现该入口无法表达某状态（例如 closures profile 拒绝非闭包函数体），按合同改为该 tier 已支持的最接近 `Div/Rem` 形态并在 §9 记录，不删反例。
- 本包不含 D13、不含 Linux 腿；二者分别归后继包与专用环境。
