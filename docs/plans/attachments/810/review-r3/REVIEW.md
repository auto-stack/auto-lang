# PLAN-810 独立 final review R3

2026-10-19；`stage: review | review_scope: final | outcome: needs_fix`。

**R2 剩余三项均 CLOSED；本轮仅有一项新增卫生 finding：两处 unused_mut。** 未发现新增产品行为回归，未要求改变已冻结的旧错误行为。

## 冻结基线

- Plan revision **1**；受审 `e49af4d0d253db1ed1a406a2e168f515d98eda1e`，base `ab08955c241f6a824091c53d3b02894ca17e193b`，前轮受审 `9226c8af9365389342bbc63d9c91e9ff0b6fee42`。
- 原工作树 `D:/autostack/.wt/lang-810/auto-lang`，分支 `codex/plan-810-dev`；入口/结论前 HEAD 一致且 clean，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean。
- 当前独立 Codex 复审上下文（同 R1/R2 reviewer，未参与实施；精确模型/session ID 未暴露）。未建 agent/树、未写实施代码、未合入。
- Cargo.lock SHA256 `39820FDEF578729B42A0371ADAA22AA0DEAF2B62C3B702C6BF6868602F9544C7`；Parser/bridge 指纹与 R2 相同，见 [baseline.json](baseline.json)。本轮相对 R2 仅改测试、两个取证脚本和两份 Spec；产品代码、依赖清单、构建/测试配置、777 配置/基线未改变。
- [冻结 SD](spec-delta.patch) SHA256 `367CFE062D452C6F31CE7808E60EDA65DBD4A62F3D7B89D1C94799D5ADB76125`；五份 canonical 输入/新增设计 proposal 的指纹记入 baseline，未发布 canonical/ledger。

## 唯一阻塞 finding

### F810-R3-01 / P3 / 新增测试引入两处 unused_mut，AC-06 尚未通过

涉及 T-06、AC-06。实际运行 `cargo test --offline -p auto-lang --lib plan810` 虽然 **22 passed / 1 ignored / 0 failed**，仍产生两处本轮新增 warning：

- `crates/auto-lang/src/tests/plan810_full_ui_grammar.rs:737:9`
- `crates/auto-lang/src/tests/plan810_full_ui_grammar.rs:763:9`

均为 `let mut bridge = ParserBridge::new(&mut parser);`，后面仅调用 `in_on_body(&self)`，局部绑定不需要 mut。[plan810.log](plan810.log) 的2737–2751附近有完整 compiler 诊断。R2 中不存在这两个新测试，因此不属于已有 warning；也不把原有 `widget.rs events` 等旧 lint 重开。

**修正要求很小**：移除这两处不必要的 mut，保持同实例断言，提交后按本模块定向复验新增 warning 消失即可；无需跑整档。AC-06 明列“新增 warning 清零”，不能仅凭测试 exit0 将其标 pass。除该项外，本轮其余已核实内容可在确认源码/依赖/配置/输入适用后有界复用，无需重新执行昂贵发行构建。

## R2 剩余三项的关闭证据

| Finding | 核对与证据 | 结论 |
|---|---|---|
| F810-R2-01 同实例/旧边界 | 新成功/失败两测在同一 Parser 统计尾部 arg 未定义1/0，均观察 on 旗标 false；与 R2 独立直接门面探针和原基面 `enter→body?→exit` 路径一致。store/object 永久测试已改回真实 object 输入。旧 scope/fragment EOF/store-fn 三项现在具名写入 full-grammar §3；根因/覆盖调整及 V-01..06 映射已补。另见下方独立 EOF/基面补验 | **CLOSED** |
| F810-R2-02 792 旧包 | 新收据实际25/25，含 CLI构建、fresh、A→B locked、原有负控和dev路径。包内新两端口及上层 Parser/bridge与当前产品源匹配；对七个相关 crate 的817个src文件独立逐字节比较，唯一差异是 cfg(test) 排除的新测试文件，生产源全部相同。精确适用范围见下方 | **CLOSED** |
| F810-R2-03 SD 矛盾 | parser-api 条目已整句重写为“当前全端口，历史曾有escape”；full-grammar 已列旧 scope/EOF/store-fn边界；contract 累计勘误已明确两种前瞻，不再将原错误合并总结作为当前设计。当前有效 SD 与实际代码一致 | **CLOSED** |

R1-01/02/04/06及R1-03/05剩余问题至此也已实质关闭；上述新 finding 是本次新增的两个编译告警，不是重新要求实现旧语法修复。

### EOF 与基面身份：独立补验，而非只信工具 pass

本轮重放实施的 baseline-identity 脚本（[输出](baseline-identity.json)）后，另行检查了它的真实覆盖和负控：

- 用忽略字符串/字符字面量/注释中花括号的独立边界扫描，确认10个函数覆盖**完整函数体**，不是只比较相同前缀。覆盖行数、提取尾部及变异结果见 [identity-controls.json](identity-controls.json)、[check-identity.py](check-identity.py)。
- 发现实施脚本的 type-ascription 正则会抹掉 `TokenKind::Dot =>` 中的枚举项，因而漏报一个 in-memory case-label 变异。**不以该宽松比较器作为最终行为等价的唯一证据**。本轮独立比较收紧为不匹配 `::` 的类型标注处理，再检查10个完整函数：原基面/迁移源码仍全部相同；逐函数真实 guard/call/case 变异 **10/10被检出**。没有修改工作树中的脚本或产品源码。
- 对已编译的冻结测试程序直接运行隔离用例，排除 Cargo 构建时间。正常用例正控确有1 test并通过；不存在的exact filter虽然进程exit0但只有0 tests，观测器明确拒绝；EOF用例确有1 test开始，准确输入是 `view fn F(x: int) { if x { text "u" `，5秒未收敛后直接测试子进程已终止并回收。[eof-controls.json](eof-controls.json)、[check-eof.py](check-eof.py) 保存完整 argv、准确输入、测试源hash和当前提交；没有把实施旧收据里省略的输入/命令当作精确证据。
- 基线判断依据为：完整函数体在冻结端口映射下的独立严格比较、已在R1/R2验证且本轮未变的端口语义、当前隔离现象。**没有声称执行过原基面二进制**，也没有把5秒观察夸大成固定崩溃码或任意输入无限期承诺。这个有界证据足以冻结本次迁移保留的具体旧路径；旧scope/EOF继续按合同记录，而非修复。

实施取证脚本只作本样本的辅助材料。其比较器不可无条件用于后续迁移；本轮的最终依据是上面的独立强化证据。该可复用工具改进另作非阻塞候选，未以记债替代当前验收。

### 792 证据的输入适用范围

[新25/25原始收据副本](792-driver-receipt.json) 与 [包内源身份比较](delivery-source-identity.json) 来自当前组内 `scratch-consumer`，不同于 R2 保存的旧包。

独立检查 auto-lang、auto-ui-dialect、auto-parser-api、auto-ast、auto-lex、auto-diagnostics、auto-val 共817个src文件：没有缺文件/额外文件，816个逐字节相同；唯一区别为 `auto-lang/src/tests/plan810_full_ui_grammar.rs`。该文件由 `lib.rs:8232–8233 #[cfg(test)] pub(crate) mod tests` 挂载，不进入该 driver 的普通 `cargo build` fresh/locked 产物。当前测试文件已单独在本轮编译运行，不能声称“整个包每个源码字节都等于当前树”，但可明确复用**产品输入相同**的新25/25证据。完整脚本为 [check-delivery.py](check-delivery.py)，报告同时保留 `all_sources_identical=false` 与 `all_production_sources_identical=true`，没有掩盖排除项。

新 grammar 所有9份源、Parser/bridge/装配和其他产品源码都已核实；原“旧包缺少新两端口”的失败条件不再存在。后续若合流改变这些实际构建输入，需重新判定证据适用性；去掉两处cfg(test)局部mut本身不会改变上述产品构建输入。

## 本轮验证及复用边界

- 实跑 `cargo test --offline -p auto-lang --lib plan810`：**22/22，1显式ignore**，含新的同实例双测、三层import、快照/prev永久反例；[plan810.log](plan810.log)。ignore仅用于无界EOF独立子进程，不把它计作未执行即通过。
- 新测试文件 rustfmt2021 exit0；`git diff --check 9226c8af9 HEAD` exit0；新增编译warning如唯一finding，不因fmt绿而抹掉。
- 实跑完整函数身份工具、独立完整边界/正反变异检查、EOF正常/零用例/准确异常三控制和包内src逐字节身份检查。过程中的review辅助脚本起初假定每个函数均有RBrace guard，已修正为按实际guard/call选择变异；最终JSON反映实际执行的变异，不是产品失败。
- 有界复用 R2 的轻consumer7/7、777 strict+15/15、806/778/408/051、VM/四生成器代表消费者及R1快照/prev重放：对应产品源码、Cargo.lock、manifest、构建/测试/777配置和输入未变；本轮变动为测试/取证/规范，不重复冷构建或宽档。
- 未跑整档 t/tv/tu/tt/tb/taa/tf、docs_gen、全画廊；未改实现/规范live ledger、未合入或清理树。

| AC | 结论 | 说明 |
|---|---|---|
| AC-01 | pass | 单源完整迁出、公开薄门面、调用闭包与V-01..06映射；源码/AST身份、位置/消费者证据沿R1/R2及本轮组合核实 |
| AC-02 | pass（具名旧边界） | 原快照/prev缺陷关闭；同实例状态、owned AST、凭据与EOF旧路径已明确冻结，未偷修旧怪癖 |
| AC-03 | pass | 默认/显式派发、Core对照、None/Err与三层import证据适用；错误语义按旧边界记录 |
| AC-04 | pass（代表支持面） | R2 VM及Vue/Rust/Ark/Jet真实消费者可复用；全档仍归merge |
| AC-05 | pass | 当前产品源792新25/25适用、轻包/777及SD事实核对完成；包内仅测试差异已明列 |
| AC-06 | **fail** | 唯一F810-R3-01：两处新增unused_mut未净，不能给final pass |
| AC-07 | pending | merge-owned裸t/tv/tu、必要tt、master并集复核、Spec/ledger/归档/guard清理未开始 |

## 状态与下一步

回 **executing 7/11**；只重开受影响的 T-06（两处新测试告警），保留T-01..05、T-07/08已验证成果。T-09最终pass尚未达，T-10/11仍pending。revision1不因簿记改变。

同一树移除两处多余mut并提交后，复核该测试文件告警/断言及冻结状态即可对受影响内容复审；无其他未闭合实现finding。后续final pass只表示代码/验收/SD通过，merge仍须在最终冻结提交执行裸 `cargo t`+`cargo tv`+`cargo tu`（触trans才tt）及closeout，不可提前合入或宣称交付完成。

共享计划写入期间曾意外成为空文件；已用本轮保存的原文SHA256与主检出Git原文逐字节核对（完全相同），先恢复原文再原子写入复审记录，无原有未提交内容丢失。恢复与最终计数收据见 [plan-recovery.json](plan-recovery.json)，原文副本见 [plan-input.md](plan-input.md)。实施工作树未受影响。
