# auto-ac MIR-native 有限标量后端（PLAN-818）

> Status: active（实现=`experimental/ace/crates/native/` 成员包（workspace根为`experimental/ace/Cargo.toml`）：`src/{config,
> symbols,lower,emit,cache,link_driver,driver,worker,incremental,observe,resources,
> workload}.rs` + `src/bin/mn-worker.rs`）。实验通道，非生产默认入口。

## 支持面（有限 scalar，冻结于 T-01 合同）

- 输入=798 共享 Typed MIR（`CheckedMir`）：typed const/copy、精确加减乘除余/
  比较/相等、一元、jump/branch/return、直接函数调用、已批准 trace 能力；短路
  与命名参数求值已在共同 MIR 显式化，后端只按指令顺序/CFG 执行。
- **不回译 HIR、不调用旧 `native::lower_object`**：新 lowering 直接从 CheckedMir
  构造 CLIF（MIR 可变局部槽→Cranelift Variables/栈槽；SSA 桥由 FunctionBuilder
  处理；循环回边延迟 seal）。旧 HIR→native 仅作对拍参照（native_reference）。
- bool 保持逻辑值 0/1，不与 i32 语义混淆；fn/参数/返回保持类型区分。
- 数值语义：加减乘/负显式溢出检查；除 0、MIN/-1、MIN%-1 按既有 scalar 合同
  执行时 trap（实验 runtime 包装层 exit 70，正常结果经完整 u32 退出观察，
  不只比低 8 位）。
- 未支持聚合/所有权/异常展开=unsupported 分类（`mn.unsupported.*`），不是 NOP
  也不转回 HIR；新 MIR 版本/指令未知即拒绝。

## 信任与准入

- 只接受本进程 verify 产生的 CheckedMir；持久 MIR 进 codegen 前用 798 真实
  解码/verify 重新授信；坏 owner/ref、slot 未初始化、profile 或能力错，在
  codegen 计数 0 处拒绝。mn 消费门对每个编辑单元**重验证**（`verify_mir`），
  emission 失败永不提交 revision。
- 私有测试 ABI `msvc-x64/priv-i32-bool-v1`（Windows x64 调用约定、i32 物理值
  通道）——**不是 797 公共 AutoABI**；target 锁 `x86_64-pc-windows-msvc`
  （其他 target 受控拒绝，不静默用 host 机器码或 AV 兜底）。

## 授权 trace

- caps `hir.test.trace.v1`（+trace-marks 支持库）：增量会话**会话级持有**
  tracelib，每一次 link/run（冷 warm-up/编辑/历史命中/run-only）都真实链接；
  原始 PE stdout/stderr 字节逐轮随观察返回（byte-exact，含顺序），父侧
  逐字节比较——不允许丢弃或硬编码空流。未授权会话使用 trace 前端拒绝。

## 对拍

- 同一输入三路径：AV（ABC 腿）、旧 HIR-native、新 MIR-native，各与**独立
  expected**（手审算例/golden）核对——typed 结果/PE 自检码、trace 字节及
  trap 前副作用顺序一致；不允许仅两后端互比。详见
  [project.md](project.md) 818 节与 [mir-native-cache.md](mir-native-cache.md)。
