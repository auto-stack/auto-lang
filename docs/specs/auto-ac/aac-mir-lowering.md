# AAC 共享 Typed MIR lowering 与资源保真 (aac-mir-lowering)

PLAN-1050 交付的 Auto 侧 HIR→共享 Typed MIR lowering 扩展。基座为 PLAN-1055
的 `aac.base-mir/1`（闭合单态标量控制流）；本包在同一实现上扩展 record 与
owned-text 数据域，并经 1051 桥的 AV 与 native 消费腿真实验证。

## 1. 已交付表面（Windows x64 hosted+alloc）

Auto 模块 `aac/mir/{base_ir,lower_base,verify_base,base_abc}.at` 经 Gen0 前端
装配（packages-aggregates-ct 车道）编译、AV/native 执行：

- **标量基座**（1055 承继，零回归）：函数/槽/块/指令分配、短路 RHS 只走被选
  CFG 路径、精确整数/浮点常量、命名实参源码序求值与声明序落位、循环
  break/continue、可观测 trap 顺序。
- **record 域**（本包）：`record_ctor`→Aggregate（操作数按字段声明序）、
  `field_get`→FieldGet（路径检查）、借用视图参数（RecordView，owned 线代际
  v2/profile 3）、record 型 `read_local` 折叠到原槽位（禁止 Record→RecordView
  复制）。记录类型表含 TY_RECORD+TY_RECORDVIEW（owner="m"/name="t<id>"/字段
  名+类型）；函数 sig 索引随类型表偏移（9+2*rec_count+text_types+fn）。
- **owned-text 域**（本包）：str 字面量（`\n`/`\t`/`\"`/`\\` 转义解码，texts
  表行首存源 AST 行、encoder/verify 按行定位序号）→StrView 常量（CONST_STR
  线形式）、`len()`→StrLen、StrView 参数跨调用。Str/StrView 为裸类型项。
- **泛型实例**：1048 实例展开先重写源（template→具体 `_i32` 等），本包
  lowering 消费单态代码，无独立泛型 MIR 语义。
- **准入与反控**：`verify_base` 执行类型/CFG/初始化/字段路径/聚合操作数类型
  检查；mir 控制 `mir-defref/type/cfg/uninitialized/change-constant`（标量）
  与 `mir-aggregate-ty/field-index`（数据面）变异均阶段定位拒绝且不发布 ABC，
  合法近邻独立执行。
- **1051 桥消费**：Auto 产 ABC 经 `ace-aac-bridge` 客户端送桥——AV 腿
  （CompileTime）返回独立值、native 腿（NativeObjects）真实发射 COFF 单元
  +wrapper；record/text 面 AV/native 双端一致。

信任边界：Auto 序列化 checked 位不具权威；Rust `ace_av_core::verify_abc`
在消费端重新准入（类型/CFG/初始化/ownership token 纪律），后端不接
unchecked 对象。机器 ABI/layout 映射（Cranelift/COFF/PE）属 Rust 后端职责，
不重做 source semantics。

## 2. 具名供给缺口（Design59 归属，解除动作）

| 缺口 | 现状 | owner / 解除动作 |
|---|---|---|
| closures/dynamic/async/scheduler HIR | Auto HIR 前端（core_hir）不产出；lowering 无法以真实输入覆盖 | 前端生产者（1046/1047/1049 链）；产 HIR 后本包 lowering 扩面 |
| owned Str/record 跨函数与入口返回 | typecheck `type.return` 拒所有 Str/record 结果 | 前端 typecheck owner；放开后补 drop/borrow 显式化与释放计数验证 |
| foreign 调用 MIR 绑定 | driver mir 路径停 `aac.pending-work/1`（JSON 页准入后未进 MIR） | 声明绑定/imports 段供给 |
| globals getter/setter/init | intrinsic+init 链需 imports 段与 snapshot 接线 | 机器/链路 owner |
| 高阶 CT（生成类型/资源） | 字面 CT 腿已工作；生成组合受 1048 LowerCt 供给与桥 Value 协议限制 | 1048 / 桥协议 owner |

以上缺口不缩最终 CL 目标（1053 收口）；本包验收按阶段合同以具名差额账
承接，1068 做三端最终验收。

## 3. 证据与验证

- 固定组：`cargo t ace-aac-syntax`（127/127，含 1055 标量 MIR 零回归）、
  `ace-lowering`、`ace-aac`、`ace-aac-bridge` 全绿；`ace-native`/`ace-av`
  失败集为既有基线红（与开工基线逐一比对一致）。
- 新增用例逐例 ≤10s（0.325–1.94s）；driver 经 prepare-native 源绑定预建
  （fingerprint 含 Auto 源+Cargo.lock）。
