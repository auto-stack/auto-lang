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

## AX-B 现状（PLAN-902 交付）

record/text/text-record 三显式源域真链：模式合法化（view/mut/move）、名义
record（含资源 record 字段级清理/深克隆）、Unicode/NUL 文本矩阵、借用返回
provenance、按验证器计划的显式资源清理（`A2xStr` 无 Drop 依赖；故障注入
typed Resource 无部分提交）。通道：recipe
`ac-a2r/hir-record-text-emit-v1`、fileset
`auto.a2x.rust.record-text.fileset/1`、证书 `ace.a2r-hir.record-text.cert/1`
（与标量通道并列互拒）。详见
[record-text-hir-backend](record-text-hir-backend.md)。

## 边界

宽标量/typed entry（PLAN-891 线）、List/Variant（AX-C）、闭包/HOF、mut
实例方法、用户泛型、第二目标、native 链接、旧 A2X 默认切换均不在现状。
899/901 交接：工具命名/测试布局由 899 收尾线所有（本线未改 workspace
Cargo/profile/固定工具名）；901 已落地的 test-feature 拍平为本线消费面。

## PLAN-904（AX-C）新域

collections(List/Variant) HIR→Rust 通道已交付（[collections-hir-backend](collections-hir-backend.md)）；
未覆盖面与 893/903 交接记录在 PLAN-904 归档件。

## PLAN-1115（普通语言 A2R）新域

普通 Auto 源（`auto.source.language.av/v1`）→ 共享 Checked HIR → 统一
language 通道（collections 族 lane + 动态/全局/跨帧错误扩面）→ 真实
rustc → 受控执行，见
[ordinary-language-and-stdlib](ordinary-language-and-stdlib.md)。
通道 identity：recipe `ac-a2r/hir-language-run-v1`（运行）/
`ac-a2r/hir-language-emit-v1`（发射）、fileset
`auto.a2x.rust.language.fileset/1`、cert `ace.a2r-hir.language.cert/1`，
与标量/record-text/collections 三通道窄通道并列互斥。固定组
`case-a2r-language-parity`（target=a2r）选择语料 A2R 值行。

## PLAN-1117（v0.5 接管路由）增量

- **宿主桥**：`auto-lang-ace-host::rust_transpile::transpile_scalar_source`
  （兄弟 CLI `AUTO_ACE_RUST=ace` 注册的渲染器名）改走普通语言通道
  （准入 cap + 实例化/CT 钩子 + `lower_language_program` + 语言 fileset
  schema）；标量前门保留在 `transpile_with_profile(.., Scalar)`。包级渲染器
  （RecordText 族）不动——多模块语言面=具名缺口（见
  [legacy-replacement](legacy-replacement.md) D-1117-2）。
- **m2-build A2R 车道**：发射腿走同一语言管线（session-less，D-1117-3），
  真 rustc/SDK/重定位/恢复沿 1116。
- 旧公开分母、逐项映射、回滚与接收门冻结在
  [legacy-replacement](legacy-replacement.md)（`IF-ACE-A2R-V05-REPLACEMENT/1`）。
