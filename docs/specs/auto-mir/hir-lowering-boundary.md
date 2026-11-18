# hir-lowering-boundary

PLAN-868 冻结的 HIR→Typed MIR lowering 模块边界：唯一实现位置、依赖方向红线与版本身份。

## 唯一实现

`Checked HIR → Typed MIR` lowering 的唯一实现是 **`experimental/ace/crates/lowering`**（crate
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
- PLAN-912 追加（纯增量，未改名/未重构既有面）：`lower_function_closures_unit_checked`、
  `lower_function_closures_unit_with`、`ClosureFacts`/`EnvFieldFact`/
  `ModuleScanFacts`/`CheckedClosureFacts` —— 单函数闭包单元从验证器凭证事实
  （env 布局/cell 位）lower，同一映射与整模块推导字节等价，故
  `MIR_LOWERING_VERSION` 不升版（映射规则未变，事实来源受检化）

模块内另有根级 `pub use hir_adapter::*`（本 crate 自身的路径展平，非跨 crate shim）。

## 消费者（仅此三处，禁止本地副本）

| 消费者 | 用途 |
|---|---|
| `experimental/ace/crates/av` | worker/会话/cache 宿主（**仅消费**，不再是提供者） |
| `experimental/ace/crates/native`（mn） | MIR→CLIF 原生对拍 |
| `experimental/ace/crates/comptime` | CTE av driver |

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
  当前 `"5"`（PLAN-1026：**console host-import 准入域扩面**——collection 家族的
  `auto.console.*` 导入闸由"模块内存在 `__sched_*` intrinsic"扩为"存在 `__sched_*`
  intrinsic **或** 任意 `DefKind::Foreign`"，此前落 `lower.intrinsic` 的形态（带
  foreign 的 closures 模块里的 console 打印）现被准入并映射 `MirHostImport`；同族
  另有两处准入面：collection 家族的 Unit 结果类型判定（`owned_type_idx_of`）与
  FnLower 调用接缝（该处拒绝码是 `lower.callee`，判据同为"任意 foreign-def 模块"）；
  新增 `__py_f64_bits`/`__py_staged_text` 重建 intrinsic 在调用点直接发射 MIR（无
  `CalleeRef`，故不再走 `lower.intrinsic`）；该域引入 F32/F64 interning 供通用 float
  console lane。AV/CTE/MN 三消费者镜像同步为 "5"）。此前 `"4"`（PLAN-894：文本域准入扩面——文本 eq/ne 按操作数真实 MIR 类型路由
  `StrCmp`（owned 操作数先 View；BIN 保持纯标量）与 `TextConcat` 操作数 view
  化；owned-Str 局部完整根替换沿既有 owned 局部 Assign Move 映射；AV/CTE/MN
  三消费者镜像同步为 "4"）。此前 `"3"`（PLAN-863：HostConsole 源码域、独立
  host 声明表/调用映射与固定 main 策略；三镜像同步为 "3"）。此前 `"2"`（PLAN-881：record/text-record 方法准入扩面 + av.data/1 按值
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

PLAN-863 的 ac/run backend-tag 版本直接复用 MN 父侧 admission mirror，避免第四个独立
字面量遗漏。native stale 检查按每行/profile 重新计算当前 backend 身份并据此比较；共享
toolchain map 未带或带有旧 backend tag 均不得覆盖该当前事实。版本 2 的旧后端报告判
stale，身份匹配的当前报告保持 fresh；不扩 native console 支持，也不改变后端选择语法。

## 已知边界（非本合同义务，记录在案）

stale.rs applicable 清单与 av 族报告轴的预存脱节（`av-core-version` 多轴漂移、
data/owned/mn 行落 sema fallback、cte 行缺 native.toolchain）是 PLAN-868 之前的既有
状态，见 KNOWN-DEBT-AND-RISKS「Plan 868」条；修复归 stale 机制维护计划。

[计划来源](plans.md)；总览 [overview](overview.md)。


## PLAN-938 增量（reviewed）

- Foreign 声明 boxed 于 MirInstr（inline ≤64B 门）；声明环境 `core-decl-env/2`
  （无 foreign 模块仍 `/1`，非别名）；恢复重锚 owner 后深验，不重投影。

## PLAN-1026 增量（Python 互操作）

- **唯一降级位置不变**：py 通道（`use.py` 成员的调用/属性/索引/运算/`.to(...)`/显示）
  与其他源码一样经 `lower_checked_*` 进入本 crate；Python 外部值继续是 i32 token 的
  `Value::External` 形态，帧文本由 FStr/StrConcat/TextDisplay 等**既有**节点构造，
  不新增第二套 lowering、也不在 AV 本地复制。
- **新增/唯一变化的 MIR 面**：`__py_f64_bits`/`__py_staged_text` 两个重建 intrinsic
  在 `FnLower` 调用点**直接发射**内联 MIR（i64 位算 + 2 的幂缩放循环 / 逐字读回循环
  + 字节拼接），不产生 `CalleeRef`；`F32`/`F64` 在该域 interning 以承载通用 float
  console lane；console host-import 准入按上文 v5 扩面。
- **消费者边界**：AV（RV 执行 + 967 shim）是唯一实现消费者；CTE/MN 镜像只同步版本
  字面量，未扩 py 能力（native/CTE 的 Python 运行适配不在本切片）。`av-core` 不依赖
  PyO3；适配器在 `ace-abi` 的 rustside 构建闭包内。
- 机械一致性：`scripts/verify-mir-lowering-868.ps1` 的 identity 段除互核三镜像外，
  还断言 ace crate 常量与镜像一致（单向漂移即失败）；"语义变了但没升版"仍须 review
  逐行 diff 本 crate，非机械可判。
