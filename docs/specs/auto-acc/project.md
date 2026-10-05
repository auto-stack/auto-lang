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

清单可信度以盘点工具严格门为准：`scripts/acc_inventory.py --check --require-decisions`
要求人工层（决定 + unknown_families）结构合法、绑定新鲜、对全部受管输入与全部
unknown 候选双向闭环；scan-only（无人工层）明示非完成态。

- required（ACC 前置，AC 现缺）：enum(±payload)、record、方法/static new/隐式 self、
  List<T>、str（最高优先）、char、int 全序运算+bool 逻辑、while/for/break/continue、
  is-match、use 多模块、mut 参数、全局变量、f-string（或改写）、源码域来源诊断。
- implemented（仅此两项，741 域）：i32 add/mul/lt + 溢出 trap(ExitProcess 70 测试约定)、
  HIR 域 file:line:col 阶段化诊断。741 r2/r3 的 verify 归属唯一、bindings 双射、
  CLI 更名（auto-ac-prototype）与制品原子发布属 Checked/工具域强化，不改变本清单缺口。
- unknown（必须带 owner/探针/所在工作包，open 族由严格门强制）：int↔i32/i64 宽度、
  char↔int、跨行字面量语义（probe-str-ml；合法形态，engine.at:314/333 实证）、
  宿主容器所有权（probe-rt-own）、用户自定义泛型（probe-generics）。
- not-required（附理由）：Option/Result 消费（编译器自身用字符串哨兵+自定义 record）、
  |> 管道、闭包/task/as 转型（lib 源码 0 使用）。
- 分类政策（r2 QA-02）：变量接收者的方法调用一律 unknown-receiver（本地类型可声明
  同名方法，Meter.len/CG.new/Ar.new 实证），native 仅限可证明宿主命名空间
  （List/IO/process/File）；变量接收者族的语义归属见决定 MD-507 与 unknown_families，
  实现期由名字解析/类型检查主体（新建）精确化。

## 锚点/候选 owner 一致性（QA-05）

A8（str/List 微程序）owner=运行时首批候选（str/容器/int 扩面工作包）；
A9（enum/is-match）owner=聚合能力线（后续未编号工作线，前置=enum±payload/record/is
落地）。两候选依赖单向（运行时候选的源码验证依赖源码计算 adapter 落地），无环；
不抢占新编号、不以此扩大本计划实施范围。

## 代际判据

Gen1=Rust AC 编译 ACC 第一代；Gen2=第一代自编译→第二代，与 Gen1 语义/诊断/HIR 等价；
Gen3=不动点。字节固定点仅在可复现条件约定后为附加门；A2R 转译中转不算 native 自举。

## 边界与已知限制

- ACC 主体清单以 743 盘点 manifest 的受管输入为准；输入漂移需重审。
- 不宣称：完整 Auto 语言面、全生态原生化、自主机器码后端（v0.7 线）。
- 旧 AAVM/AA2R 绿语料不构成 AC/ACC implemented 证据。

## 相关 plan

见 [plans.md](plans.md)。
