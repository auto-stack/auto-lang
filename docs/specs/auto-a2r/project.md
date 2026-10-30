# auto-a2r

> **Status**: experimental
> 路径：`experimental/ace/crates/a2x-rust`（crate `auto-a2x-rust`）+ worker `probe::a2x-worker` | 依据：Design 51 AX-A（PLAN-892）

首个真实 A2X 目标 provider：Checked core/scalar HIR → Rust（edition 2021）。
**不是** mn 私有 ABI、**不用**旧 a2r 的 i64 wrapper（旧 Trans/a2r-std/旧 golden
不重构不迁移，只读参照）。

## 现状（AX-A 交付）

- **支持矩阵**：core（add/mul/lt＋signed 字面量＋if/while/loop）与
  scalar（sub/div/rem、全序比较、同类 eq/ne、neg/not、短路、条件 for）全部
  计算 surface——冻结 745/767 语料＋ACE-M1 B 族＋边界合成共 58 用例经真实
  rustc 编译运行对独立 oracle 全绿；debug/release 对照在 trap 用例上成对。
- **值/trap 协议**：`Result<i32|bool, AutoTrap>`，五类具名 trap
  （Overflow/DivByZero/DivOverflow/RemByZero/RemOverflow；商向零截断、余数随
  被除数；MIN/-1、MIN%-1 具名）。`i32::MIN` 打印为命名常量。
- **观察协议**（`a2xworker/1`，包装器输出）：stdout 单行 `value:<i32>` 或
  `trap:<Name>`；stderr 按执行序 `mark_a`/`mark_b`（值 1/2，冻结原生合同）；
  退出码 0=观察到结果，非零=infra。trace 与值通道分离。
- **文件集**：`generated.rs`＋`a2x_runtime.rs`（typed 原语）＋`main.rs` 包装
  ＋`manifest.json`（schema `auto.a2x.rust.fileset/1`，sha256 绑定、无
  nonce/时间/绝对路径；跨 nonce 逐字节确定）。
- **证书草案**（`ace.a2r-hir.cert/1`＋recipe `ac-a2r/hir-scalar-emit-v1`）：
  严格键集校验（旧 schema 冒充/空绑定/缺工具身份→REJECTED）、字节稳定序列化、
  绑定实测 `rustc -vV` 身份。
- **受控执行**：worker 全部 spawn 经 ac-core 受控执行器（Job 子树＋CREATE_SUSPENDED
  ＋1MiB 流上限＋单一 deadline：编译≤120s/运行≤30s）；挂起目标 deadline 整树
  被杀、受控拒绝、零悬挂。

## 边界

宽标量/typed entry（PLAN-891 线）、record/text/集合、第二目标、native 链接、
旧 A2X 默认切换均不在现状。
