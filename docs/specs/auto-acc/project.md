# auto-acc

> **Status**: planned（目标态合同；AC=首版原生编译器，ACC=Auto 实现的编译器主体）
> 主体清单与取证：docs/reports/743-acc-hir-contract/（PLAN-743）；
> 阶段/桥/代际契约：docs/design/strategy/auto-acc-bootstrap-contract.md

ACC 主体的约定目标、能力现状与自举判据。防止"Auto 写的编译器"统称掩盖
AAVM/AA2R 自举（已达成谱系）与 AC/ACC native 自举（未达成）两条线。

## 主体清单（现状：adapt/reference 分层）

- adapt（代码可迁移）：auto/lib/token.at（词法判据面）、lexer.at（游标扫描核心）、
  parser.at 游标/Pratt 核心（产物 S-expr 层除外）。
- reference（不搬迁）：typeinfo.at（Display 推断非 typeck 凭证）、codegen.at（ABC
  发射非 native 管道）、engine.at（ABC 解释器=oracle）、a2r.at（AA2R 转译谱系）、
  auto/aavm.at（CLI 形态参考）。
- 新建（现不存在）：名字解析、typeck、源码→HIR adapter、语义 lowering+能力门、
  驱动+来源诊断、后端桥客户端。
- 非主体保留：Cranelift 0.126 桥、rust-lld/link.exe、kernel32 启动包装、
  宿主内建 File/IO/process/print/List（所有权语义待 probe-rt-own）。

## 能力现状（compiler-source-demand × compiled-language-support）

- required（ACC 前置，AC 现缺）：enum(±payload)、record、方法/static new/隐式 self、
  List<T>、str（最高优先）、char、int 全序运算+bool 逻辑、while/for/break/continue、
  is-match、use 多模块、mut 参数、全局变量、f-string（或改写）、源码域来源诊断。
- implemented（仅此两项，741 域）：i32 add/mul/lt + 溢出 trap(ExitProcess 70 测试约定)、
  HIR 域 file:line:col 阶段化诊断。
- unknown（探针 owner 待立项）：int↔i32/i64 宽度、char↔int、跨行字面量语义、
  宿主容器所有权、用户自定义泛型。
- not-required（附理由）：Option/Result 消费（编译器自身用字符串哨兵+自定义 record）、
  |> 管道、闭包/task/as 转型（lib 源码 0 使用）。

## 代际判据

Gen1=Rust AC 编译 ACC 第一代；Gen2=第一代自编译→第二代，与 Gen1 语义/诊断/HIR 等价；
Gen3=不动点。字节固定点仅在可复现条件约定后为附加门；A2R 转译中转不算 native 自举。

## 边界与已知限制

- ACC 主体清单以 743 盘点 manifest 的受管输入为准；输入漂移需重审。
- 不宣称：完整 Auto 语言面、全生态原生化、自主机器码后端（v0.7 线）。
- 旧 AAVM/AA2R 绿语料不构成 AC/ACC implemented 证据。

## 相关 plan

见 [plans.md](plans.md)。
