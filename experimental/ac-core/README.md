# ac-core — PLAN-741 AC 首个闭环原型

独立 Cargo workspace:Schema-bound Atom HIR 文本 → 类型化绑定 → 语义校验 →
Cranelift native lowering → Windows x64 COFF 对象 → rust-lld 链接 → 运行 PE。

这是计算核心 profile(core-i32-draft)的首条纵向实现:**"HIR → 原生执行"**,
不是 "Auto 源码 → 原生执行",也不是 AAC 自举完成。范围、非目标与验收见
[docs/plans/741-ac-hir-native-core.md](../../docs/plans/741-ac-hir-native-core.md)。

## 流水线

```text
.atom 文本
  → atom_text.rs        分词/解析,带字节 span 的语法树
  → descriptor.rs       core-i32-draft descriptor 绑定(schema/core-i32.atom)
  → hir.rs              类型化 ID + Unchecked HIR
  → verify.rs           语义校验 → CheckedModule(唯一构造路径;块包含图
                        可达/无环结构门先于数据流走查)
  → native.rs           Cranelift lowering(溢出 trap = ExitProcess(70))
  → link.rs             rust-lld COFF 链接(暂存+备份+发布,失败回滚 + 收据;
                        全子进程硬截止覆盖等待与输出收集,Job Object 管控进程树)
  → Windows PE
```

## 使用

CLI 的二进制名是 `auto-ac-prototype`("ac-probe" 为历史文档名),子命令 check/build
直接跟在 `--` 之后(与 `scripts/verify-ac-741.ps1` 一致):

```powershell
cd experimental/ac-core

# 结构 + 语义检查(退出码 0 / 1 / 2)
cargo run --locked -- check ../../docs/design/strategy/hir-examples/01-add.atom

# 原生构建(零参数、返回 i32 的已声明 DefRef 作为入口)
cargo run --locked -- build `
  fixtures/native/add-2-3.atom `
  --entry d_entry --output target/ac-artifacts/add.exe

# 需要 trace 能力时必须显式声明 + 链接支持库
cargo run --locked -- build `
  ../../docs/design/strategy/hir-examples/03-call-order.atom `
  --entry d_caller --output target/ac-artifacts/trace.exe `
  --capability hir.test.trace.v1 `
  --support-lib test-support/target/debug/ac_trace_support.lib
```

构建产物(.obj/.exe/.ac-link.txt 收据)放在 `--output` 所指目录;
失败的构建不会覆盖既有成功制品(发布阶段失败自动回滚)。

## 一键验证

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ../../scripts/verify-ac-741.ps1
```

覆盖 §6 全部命令:定向测试族(text_binding / hir_verify / native_execution /
cli / trace_execution)、CLI 实机构建运行、fmt 与 all-targets 零 warning 检查,
以及主仓最终门禁(`cargo check -p auto-lang`、`cargo t`、`cargo tv`)。
所有子进程有截止时间。

## 测试族

| 测试 | 内容 |
|---|---|
| text_binding | 四份设计样例绑定、作者/显式/canonical 往返语义等价、enum 简写、15 类拒绝 |
| hir_verify | §9 拒绝矩阵 + 组合反例(共享表达式/块、环、跨 body、初始化交集、返回路径、块图自环/断开环) |
| native_execution | 真实 PE 运行:add(2,3)=5、count 循环、i32 溢出 trap=70、COFF 符号 |
| cli | check/build CLI 退出码、入口/能力/链接错误路径、失败不覆盖、块图反例 5s 内定位拒绝 |
| trace_execution | 显式 capability + 支持库:原生调用顺序 b→a、pair 结果 12 |
| lib(link 单测) | 执行器:1s 截止杀挂起 helper、收集阶段同截止(后代持管道)、大输出完整、发布事务失败矩阵(准备/备份/三次发布) |

## 边界(勿过度宣称)

- 只接受 `auto.hir.core.draft / revision 1 / core-i32-draft` 身份;未知
  tag/case/字段/能力一律拒绝。
- 类型只有精确 i32/bool;运算 add_i32/mul_i32/lt_i32;加/乘必须声明
  `overflow: trap`。
- 单模块内解析引用;跨模块引用、字符串/聚合/所有权/泛型、生产 ABI、
  DLL/热重载均不在本原型范围。
- trace intrinsic 是测试能力(`hir.test.trace.v1`),不是生产 I/O。
- 溢出 trap 固定退出码 70,正常测试入口退出 0;这不是全语言异常 ABI。
