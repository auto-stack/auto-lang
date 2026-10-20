# auto-abi

> **Status**: candidate（`autoabi.win-x64.v0.1-candidate`，PLAN-797 交付）
> 路径：`experimental/auto-abi-probe`（独立 workspace，不进产品构建图）| 工具链：Rust/Cranelift 0.126 + MSVC cl.exe + rust-lld

公共 AutoABI 候选 profile：首平台 Windows x86_64-pc-windows-msvc 上
AutoABI↔AutoABI、AutoABI↔C 双向互调的有限边界合同与 conformance 证据。
目标声明见 [Design 41](../../design/41-interface-metadata-and-foreign-abi.md)
与 [ABI RFC](../../design/strategy/auto-native-abi-rfc.md)——它们是目标文档，
本目录记录的是**已实现并经独立复审的候选现状**。

## 目标与范围

- 冻结首个有界 Windows x64 候选 profile 的域/版本/符号/descriptor/类型
  布局/调用约定/ownership/错误与 runtime 最小服务（见
  [windows-x64-v01.md](windows-x64-v01.md)）。
- 独立编译双方真实 COFF 对象、三腿互调与缓存/conformance 基准
  （见 [conformance-cache.md](conformance-cache.md)）。
- 不做：完整 AutoABI 所有类型/平台、C header importer、Rust shim 产品、
  variadic/bitfield/union/C++ ABI、DLL loader/热重载、跨线程 callback。

## 域与版本分层

1. ABI 域 `autoabi`——与 774 `probes/data-resources` 的内部 ABI-A–E
   **无兼容关系**，禁止"两者可互换"表述（分域声明见
   [auto-ac/data-resource-contract.md](../auto-ac/data-resource-contract.md)）。
2. target/profile：`autoabi.win-x64.v0.1-candidate`（本目录登记的
   有限 profile；PLAN-797 经独立 review R1–R17 + merge 门禁后冻结）。
3. 接口 revision = descriptor canonical fingerprint（blake3）。
4. runtime 服务表版本（descriptor `RuntimeReqs`）。
5. 语义 schema 版本（真实 `ac-sema-contract` 内容版本；本计划消费
   只读，见 [auto-semantics/semantic-contract.md](../auto-semantics/semantic-contract.md)）。

五者分开存在；任一错配在调用前拒绝，native 被调次数 0。

## 支持边界（v0.1 有限 profile）

| 面 | 值 |
|---|---|
| 声明源 | 受限 declaration IR（**非** Auto 源码 parser 支持面） |
| C 工具 | 可定位 MSVC cl.exe 独立编译；缺工具 = infra 阻断，禁止 Rust repr(C) 伪通过 |
| 拒绝面 | 按值聚合、variadic、bitfield、任意 union、C++ ABI、异常/unwind 穿界、跨线程持久 callback、热重载/DLL 事务、跨域自动转发 |

拒绝面是有限 profile 支持表，不是永久删去战略。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| descriptor | 机器可读调用前边界（`autoabi.descriptor/3`，canonical JSON + blake3 指纹；出参列表/handle 归属/callback ctx/record mut-out 一等形态；C provider 描述与独立 header 原型逐项对照） | active |
| layout | 两阶段布局引擎（size 记忆化 + 精确表示字节/索引/递归态预算门 + 单向量物化）；ABI-L0 缓存 | active |
| validate | 调用前 admission（12 维拒绝矩阵，native 计数 0） | active |
| emit/build/link | Cranelift 声明 IR→COFF；ac-core link 工具复用 | active |
| cache | ABI-O1 对象层：观测依赖闭包 + 候选位置全集适用域 + 代际键 | active |
| worker | `abiworker/1` 受限执行半体（Job containment、收据化 GC、产物预算） | active |
| cside | 独立审定的 C 参考 provider/consumer（真实 cl 编译对拍） | active |

## 历史与 ADR

- 计划过程记录：`docs/plans/archive/797-autoabi-v01-bidirectional-probe.md`（R1–R17 完整复审链）。
- 关键裁定的理由（为什么不扩大、为什么候选而非发布）见该计划 §5 与
  freeze proposal `docs/reports/plan797/freeze-proposal.md`。
