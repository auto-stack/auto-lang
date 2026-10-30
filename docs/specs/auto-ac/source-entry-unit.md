# Auto 源码 entry-unit profile（入口 / Unit / 尾值）

> **Status**: experimental；`auto.source.entry-unit.draft` revision 1；HIR 身份
> `auto.hir.entry-unit.draft / 1 / entry-unit-draft`（descriptor
> `experimental/ace/crates/core/schema/entry-unit.atom`）。
> 实现：`experimental/ace/crates/core/src/source/{parser,resolve,typecheck,adapter}.rs` +
> `src/{descriptor,verify}.rs`；消费端见 [project.md](project.md)；执行侧见
> [auto-av/entry-and-unit-profile.md](../auto-av/entry-and-unit-profile.md)。
> 来源：[PLAN-862](../../plans/archive/862-av-source-entry-unit-and-tail-semantics.md)；
> 冻结合同 `docs/plans/attachments/862/work/entry-contract.md`（T-02）。

本 profile 在 scalar-i32 计算面之上补齐**真实程序的前门**：`fn main() int`、
`fn main() void`（Unit 结果）与单文件脚本（顶层语句序列，尾值为结果）。它是
R2-E/SEM-05 的源码侧合同，不是完整语言面：控制台/print（PLAN-863）、集合
（PLAN-864）、闭包（PLAN-865）、record/text 源码（R2-D）仍由具名工作包承接。

## 入口矩阵（唯一性按声明判定）

| 形态 | 写法 | 结果通道 |
|---|---|---|
| int 入口 | `fn main() int { … return e }` | 值（i32） |
| Unit 入口 | `fn main() void { … }`（落空结束，禁止 `return <value>`） | Unit |
| 脚本 | 顶层语句序列（无 `fn main`），尾表达式语句为结果；有 i32 尾值→值，无尾值→Unit | 值 / Unit |

- 每模块恰一个入口候选，入口名固定 `main`；不按文件名/参数形态挑选。
- 具名拒绝：`source.entry-missing`（无入口）、`source.entry-duplicate`（重复
  `main`，含仅参数/结果不同的重载形态；entry-unit 域由入口矩阵而非通用重名码
  出判）、`source.entry-ambiguous`（声明 main + 顶层语句并存）、
  `source.entry-params`（入口带参）、`source.entry-top-return`（顶层 `return`）、
  `source.entry-order`（脚本序列之后不得再有 item）、`source.entry-result`
  （入口结果位规则）。
- `int` 入口全路径返回沿既有纪律（`verify.return-path`）；`int` 入口不接受尾值
  隐式转换（`source.entry-tail`）；脚本尾值必须是 i32（`source.entry-tail-type`）；
  Unit 入口禁止带值返回（`source.unit-return-value`）。
- 非法面同样是**静态**判定：未执行分支、未调用函数按同一纪律检查，入口类型错
  在 verify 期报，不落运行期。

## Unit 表示（结果位之外一律拒绝）

- 源码 `VT::Unit`（`as_str()` = `"unit"`）只出现在函数**结果位**；它不是值类型。
- 结果位以外写 Unit 词（`void`）在参数/局部位置被解析并命名
  `source.unit-placement`（T-07 F-862-R1-02 起可达；此前被 parser 的
  `source.unsupported` 遮蔽）。entry-unit 面没有 `type` 声明，因此字段/place
  条款由 HIR `verify.unit-placement` 与 MIR `mir.unit-placement` 守护（仅对手工
  构造的文档/模块可达，两者均有负例）。
- HIR：`TypeUse::Builtin(Unit)` 仅作 function 类型声明的 result；MIR：零运行时
  宽度的 `MirType::Unit` slot 仅作 unit 入口唯一 `Return` 的操作数。

## 与其它 profile 的边界（零漂移）

- `void` 类型词、表达式语句（脚本尾值）、顶层脚本形态**只在** entry-unit 面
  合法；scalar/text/record/text-record 对同一输入仍 `source.unsupported`
  （`void_is_not_part_of_the_other_profiles`、
  `expression_statements_stay_rejected_on_the_scalar_surface` 与新增断言）。
- 既有四 profile 的入口语义（零参、i32 结果、调用方按 DefId 选入口）与 HIR
  字节不变；本 profile 是独立域与独立身份三元组，未注册三元组整体拒绝
  （`bind.descriptor-unknown`）。
- 旧域文档喂新入口、新域文档喂旧入口均由 `verify.domain` 拒绝；AV 执行侧另有
  wire 家族/entry kind 门（见 [auto-av/entry-and-unit-profile.md](../auto-av/entry-and-unit-profile.md)）。

## 消费者与拒绝面

- `native`/`text`/资源/CLI/artifact codec/semantics adapter/CST adapter 对
  `HirDomain::EntryUnit` 给出显式拒绝臂：**Unit 没有 native lowering**，本
  profile 仅由 AV 执行；`ac/mir-native-*` 消费者受控拒绝，不落半条链。
- 源码 profile 只由显式选择决定（`auto.source.entry-unit.draft`），不按扩展名或
  内容猜测；未知 profile 用法错。

## 验收与证据

- 前端矩阵：`experimental/ace/crates/core/tests/entry_unit_862.rs`（26 例：三形态正例、
  8 类入口/Unit 负例、放置可达性、跨域身份拒绝、静态全链）。
- 语料：`crates/auto-lang/test/cases/ace-m1/entry-862/`（U01–U10，独立手算
  oracle，经真实 `ac-av/core-run-v1` + `av: { profile: "entry-unit" }` 执行）。
- 门：`powershell -File scripts/verify-av-862.ps1 -Scope <targeted|compat|entry|acceptance>`；
  `entry` 含判别力反控（把 Unit 期望改成 value 必须恰好红 U02/U04）。
