# hir-lowering-boundary

PLAN-868 冻结的 HIR→Typed MIR lowering 模块边界：唯一实现位置、依赖方向红线与版本身份。

## 唯一实现

`Checked HIR → Typed MIR` lowering 的唯一实现是 **`experimental/auto-mir-lower`**（crate
`auto_mir_lower`，独立于根 workspace，与 experimental 各树同构独立解析）。原实现自
`ac-av-probe/probe/src/hir_adapter.rs` **字节级纯搬移**（git rename 100%，两路径 blob
同 hash），语义零变化。

公开面（签名自 PLAN-798 冻结沿用，不改名/不重构）：

- `AdapterError`（+ `code()`）
- `lower_checked` / `lower_checked_data` / `lower_checked_owned` / `lower_checked_console`
- `lower_function_unit` / `lower_function_data_unit`
- `lower_function_owned_unit` / `lower_function_owned_unit_with_modes`
- `owned_entry_ok`
- `MIR_LOWERING_VERSION`（PLAN-868 新增，见下）

模块内另有根级 `pub use hir_adapter::*`（本 crate 自身的路径展平，非跨 crate shim）。

## 消费者（仅此三处，禁止本地副本）

| 消费者 | 用途 |
|---|---|
| `experimental/ac-av-probe/probe` | worker/会话/cache 宿主（**仅消费**，不再是提供者） |
| `experimental/ac-mir-native`（mn） | MIR→CLIF 原生对拍 |
| `experimental/ac-cte-probe/probe` | CTE av driver |

守卫：`scripts/verify-mir-lowering-868.ps1 -Scope deps`（experimental 全域
`pub fn lower_checked(` 定义唯一、三消费者 cargo tree 解析、av-core 依赖清单为空、
根 workspace 零 experimental path 依赖）。

## 依赖方向红线

- `auto-mir-lower` 仅依赖 `auto-ac-prototype`（ac-core：HIR/verify 前门）与
  `auto-av-core`（Typed MIR 模型/verifier）；**无** Cranelift/serde/rusqlite，**无**
  任何 probe 宿主类型（worker/会话/cache）——lowering 若需宿主能力必须以参数/回调倒置，
  不得反向依赖。
- `auto-av-core`（执行核心）保持**零依赖**（"Pure std"），不得因 lowering 并入而依赖
  前端 HIR 类型——这是 lowering 不并入 av-core 的原因。
- `auto-lang` 产品构建图零实验栈依赖（根 workspace `exclude experimental`）不变。

## 版本身份（MIR_LOWERING_VERSION 与 mir-lowering 轴）

- `auto_mir_lower::MIR_LOWERING_VERSION` = lowering **契约**版本：仅当
  lowering 语义变化（映射规则/准入域/错误分类）时升版；纯重构与消费侧搬移不升。
  当前 `"3"`（PLAN-863：HostConsole 源码域、独立 host 声明表/调用映射与固定 main
  策略；AV/CTE/MN 三消费者镜像同步为 "3"）。此前 `"2"`（PLAN-881：record/text-record 方法准入扩面 + av.data/1 按值
  mut-record 写回 ABI + copyable `.mut` Copy 表示；三消费者镜像同步为 "2"，
  旧语义 cache/报告身份失效）。初值 `"1"`（PLAN-868）。
- 报告身份轴 `mir-lowering`（**必报**）：三消费者经测试侧镜像 admission 常量
  `EXPECTED_MIR_LOWERING_VERSION`（ac_av/ac_cte/ac_mn 各持，与
  `EXPECTED_AV_WORKER_VERSION` 同模式）写入 `current_toolchain_identity` 与报告/收据
  外层 toolchain；stale 裁决 av/cte 分支 applicable 含该轴——旧报告缺轴判 STALE
  （`native.mir-lowering missing (now <v>)`），属声明的重构期一次性代价。
- **收据键集不动**：`ace.m1.cert/1` 的 `toolchain` schema（identity/
  identity_source/default_identity_at_certify/flags）与本轴无关，双消费者
  （Rust `validate_receipt` + `verify-ace-m1-852.ps1`）校验不变。
- cache/store 键不含该轴（搬移不改输出）；lowering 语义真变时另立版本迁移。

## 已知边界（非本合同义务，记录在案）

stale.rs applicable 清单与 av 族报告轴的预存脱节（`av-core-version` 多轴漂移、
data/owned/mn 行落 sema fallback、cte 行缺 native.toolchain）是 PLAN-868 之前的既有
状态，见 KNOWN-DEBT-AND-RISKS「Plan 868」条；修复归 stale 机制维护计划。

[计划来源](plans.md)；总览 [overview](overview.md)。
