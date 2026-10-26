# ACE M1 四路径测试设计包

本包是 PLAN-852 的**可审阅测试设计**，不是已经接入 runner 的语料，也没有任何后端 PASS 声明。
PLAN-852 r2已经用户确认，下一步由用户启动work：先交付测试底座与B01/B02/B04/B09标量四路径基线，
不等待823～826或M1全部用例通过。完整分母和缺口保留，基础层验收与M1完整性分别报告。
下游在各自T-11消费D852已合入底座；详见[交接合同](../../reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。
`src/*.at` 是58份实际 Auto 源码草案；另有 `typed-source/*.at` 五份宽标量宿主入口草案；
`reference/*.rs` 是43份独立手写的行为参考；
`cases/*.case.ad.draft` 和 `values.suite.ad.draft` 是使用现有 TestCase 字段的接线草案。`.draft` 后缀和本目录隔离
保证它们不会被现有正式 suite 发现。正式路径拟为 `crates/auto-lang/test/cases/ace-m1/`。

## 两种验收，分开运行

1. **日常**：AC 和 AV 执行，AutoVM 执行；a2r 只 emit 并逐字节比较经过认证的 Rust 金样。
   不调用 rustc/cargo、不运行 Rust 产物。金样缺失、内容不符或必需认证缺失均不能算四路径通过。
2. **独立认证**：实际 a2r 输出必须与待认证金样相同；编译、运行该输出与独立手写 Rust 参考，
   分别与固定行为 oracle 比较。保存证书，供以后日常快照检查复用。不能把 expected Rust 当 actual。

认证按用例/金样及其真实依赖闭包建身份，不是整个转译器认证一次后永远免验。
金样 bytes、语义 oracle/独立参考、runtime/依赖、Rust target/toolchain/生效 flags 改变时，对受影响项重新认证。
转译器变更但实际输出仍等于已认证金样、运行上下文未变时，**不因转译器版本号改变而重编**。
本包未生成 Rust 金样：金样必须来自真实 a2r 输出并独立审阅，不能拿 `reference/*.rs` 冒充。

## 同一份源码与观察口径

- 四路径消费相同 `src/*.at` bytes。允许宿主入口包装与结果编码；禁止改写算法、删除借用/move/分支、
  用不同源码掩盖引擎差异。当前草案统一使用 `fn main() int`，其旧 AutoVM/a2r 返回值入口能否原样承载，
  由 T-01 真探针确认；若需改为共同命名测试函数，统一更新四路径，不只改其中一端。
- 成功用例观察完整 typed i32（包括负数和大于255的数），不只比较进程退出码低8位。
  常规用例的独立参考输出使用 `i32:<十进制>\n`；这只是宿主观察，不向 Auto 程序加入 print 依赖。
- trap/诊断不是普通数值。阶段/码按后端绑定到共同语义分类；不能靠相同退出70证明同一种错误。
  Rust 编译错误、缺工具、超时和进程崩溃均不能满足 Auto 语言负例。
- 所有权、资源清理、精确失效和热替换要用 scenarios 的额外观测，不能仅凭返回值相同证明。

## 覆盖与未决项

`cases.json` / `matrix.tsv` 是逐例清单；`ORACLES.md` 给出43个值例的独立推导；
`SCENARIOS.md` 补充宽标量位型、Dyn/冻结、CTE、
求值顺序、资源与增量热重载的精确 oracle。
源代码中的 List/Option/Result/enum/闭包沿现有 Auto 示例拼写，但**新 ACE 源码接线尚待实现**。
`.clone/.move`、借用与 enum 捕获的组合也要逐例核对，不声称现有四路径已经接受。
Dyn/冻结尚无共同 Auto 源码 API；本包不凭空发明 `Dyn.pack` 或 `freeze` 的语言语法。
T-01对未交付接口登记owner、所需能力和关闭oracle；由后续能力包冻结真实API后再补齐对应源码。
这些项保留M1 required gap，不把冻结全部新语言接口作为852底座结项前提。

## 可先做什么

可先审阅源码/独立 expected，完成旧 AutoVM 与 a2r 的语法/入口探针及标量三路径接线；
不等822～826全体结束。集合、闭包、Dyn等正式消费分别等待其上游已合入合同及单写窗口。
本计划不修各引擎的语言缺口，也不实现 native 在线 loader；发现缺口登记 owner 和后续计划，
不能为了让测试全绿而删掉本分母。

源码/参考由 `build_design.py` 生成，脚本只生成本目录的设计资产；并非新的 AutoDown 解析器或 runner。
本次仅作清单、文件身份与参考 oracle 静态核对，未执行 Auto/Rust 编译或运行。
