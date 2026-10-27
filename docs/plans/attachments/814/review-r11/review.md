# PLAN-814 R11：完整 T-10 复审，needs_fix

2026-10-21 / stage: review / review_scope: final / plan_revision: 3。
reviewer_identity：本聊天 Codex 独立复审上下文，未参与实施；未创建 agent 或新 checkout。结论由已提交代码和独立重放取得，不采用 work 的“完整关闭”标签作为证据。

## 冻结身份

- 原唯一树 `D:/autostack/.wt/lang-814/auto-lang`，branch `codex/plan-814-dev`。
- reviewed_commit `49206c6992b177cc02e84f35be34a885a2ebd817`；tree `5c7321ec8ab94cf12c41c347a5e22ea59a2fa21d`。
- 修复 base 为 R10 `236827f1c43dfa991e5a047a0eaa3e0ac0d07a38`；B base 为 A 发布 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，初末 HEAD/洁净状态未变；实施树 clean。
- 计划/交接/SD 冻结在本目录；11 个 canonical Spec 源与 R10 相同，runtime Spec 尚未新增。源码、配置、Spec、反例/结果哈希见 `input-hashes.json`。

## R10 四项关闭核销

| 原项 | 本轮结论 |
|---|---|
| R9-01（R10 仍未闭）比较类型 | **原 R10 五例全部关闭，整项仍 OPEN**。无 feature 正控、unknown-negative/unknown-float/unknown-rhs-float/prop-negative/prop-float 均实际编译 exit0；另测负浮点左右位、整数算式、字符串状态正控通过。分组和已声明操作数仍失败，见下。 |
| R10-01 浮层 position 仲裁 | **CLOSED**。真实 Parser→AURA→VmBridge→AuraViewBuilder 重放原五组合，加 sticky 后继/反向覆盖、无 z 控制共八组合，mismatches0、exit0。源码四种 position 类按同一顺序覆盖，与未改的 IcedStyle 规则一致。 |
| R10-02 777 基线过期 | **CLOSED**。当前冻结 SHA 实际 strict exit0、violations0、grandfathered3。基线仅将 base_revision 更新为代码修复提交 `2f83c4270fbea69438c2a36d934d6b7161883746`；manifest 已入锚，未扩大窄豁免或关闭 staleness 检查。 |
| R10-03 遗留 serde 导入 | **CLOSED**。匹配四包 check exit0，原 scroll_state_reader serde warning 消失，runtime236→235；未用 allow 掩盖。另有 compiler 新增告警，单列 R11-01。 |

四包 check/手写触面 rustfmt/git diff --check 均 exit0。bare ui 与 ui-headless 本轮重新 check 均 exit0，默认构建由四包 check 覆盖；R10 其余 no-default/ui-dialog/code-editor 构建证据按未改的 feature/依赖/条件代码复用。

## R9-01 / P2 继续 OPEN：字面量分类不能代替表达式类型保持

影响 AC-03/06、T-05/T-08/T-09。位置：`crates/auto-ui-compiler/src/ui_gen/rust.rs:8681`，尤其 `:8787` 的数字 token 判定，以及 loop/prop 两个调用点。

本次扫描已认识带符号数字和小数，但仍只看紧邻 token。遇到 `(` 或已声明的 `.limit`/`.allowed`/`.threshold`，不能识别其类型就回落 str；生成器本身已经有 `state_types` 保存这些声明的 int/bool/f64 类型，却没有使用它们来判比较操作数。

| 正反控 / 输入 | 当前真实产物与编译 |
|---|---|
| `p.score > 0.5`；`p.score > -0.5`；`-0.5 < p.score` | f64 投影，exit0 |
| `p.score > (0.5)` | `p["score"].as_str().unwrap_or_default() > ( 0.5 )`，E0277、exit101；仅加透明分组即改变类型 |
| 模型声明 `var limit int = 3`，`p.rank > .limit` | `p["rank"].as_str().unwrap_or_default() > self.limit`，E0277、exit101 |
| 同一声明，`.limit < p.rank` | `self.limit < p["rank"].as_str().unwrap_or_default()`，E0308、exit101 |
| 声明 `var allowed bool = true`，`p.dirty == .allowed` / `.allowed == p.dirty` | 两侧位置均误投 str，E0277、exit101 |
| 声明 `var threshold float = 0.5`，`p.score > .threshold` | str 与 f64 比较，E0277、exit101 |
| Value prop，声明 limit 后 `note.rank > .limit` | 同源失败，E0277、exit101 |
| 声明 `var wanted str = "1"`，`p.running == .wanted` | 字符串状态正控，exit0 |
| `p.rank > 1 + 2` | 整数算式正控，exit0 |

所有输入在 `*.at`，从真实 Parser/AURA 和公开生成器得到的未手改产物为 `*.at.rs`，生成结果 `generation.log`；`check-unknown-paren-float.log`、各 `check-*-state*.log` 保存实际 Rust 编译错误。无 feature 的 `controls.log` 同时编译原 R9/R10 正控和旧路径身份 pin，exit0，随后每个 feature 分别编译，避免把别的 fixture 缺类型导致的红当作该反例。

其中 `.limit` 还有修复回归的源码依据：236827f1c 的旧 classifier 把点前缀 token 归 int；本次数字校验收窄为“全是数字/点”，`.limit` 回落 str。这里没有声称旧 classifier 对任意符号都正确；当前相同 int 声明的比较确实已不能编译。分组控制同样是相同已有数字表达式，不是新增语法/API/规模需求。

诊断与覆盖调整要求：修复应基于可用的表达式类型/声明环境保留比较类型，并透明处理分组，不能继续以新增某个字节或某个字段名替代不变量。验收矩阵至少按 **字面量 / 已声明变量 / 分组**、loop/prop、左右位置、int/float/bool/str 取代表性组合；保留本轮与 R9/R10 全部正反控。若使用 AST/已有 type 信息可直接复用其解析结果，不要求创建另一份语言解析器或扩展原支持范围。

复现：在本目录 manifest 下构建 `r10-generate`（沿用复制来的 bin 名），生成输入；再运行 `cargo check --offline --locked --manifest-path <本目录>/Cargo.toml --target-dir <原树>/target --bin r10-check --features unknown-int-state`。其余失败 feature：unknown-int-state-rhs、unknown-bool-state、unknown-bool-state-rhs、unknown-float-state、prop-int-state、unknown-paren-float。去掉 feature 及前述 R10 五 feature、其他正控应先绿。

这是原 R9-01“真实类型、loop/prop×左右位”的不变量未闭与修复回归，不另造新类型 finding。SD-02 的“完整类型感知”与“非字面量默认 str”不能据此落成削减原声明类型支持的规范。

## 新 F814-R11-01 / P3：数字扫描新增 unnecessary parentheses 告警

影响 AC-06、T-05/T-08/T-09。位置 `crates/auto-ui-compiler/src/ui_gen/rust.rs:8758`：

```rust
if (fwd == b"-" || fwd == b"+") {
```

四包 matching check 明确报告 `unnecessary parentheses around if condition`，compiler36→37。该行由本次修复新增，236827f1c 中没有该告警点；另一个旧位置12859的同类 warning 属历史，本次不重开。runtime 少掉一条 serde warning 与 compiler 新增一条 warning 相抵，聚合总数不变不等于零新增。

去掉多余条件括号并复验 matching check；无需处理无关历史告警，不能 blanket allow。

## 全部 AC / 证据复用边界

| 范围 | 本轮结论 |
|---|---|
| AC-01 / V-01 | 按未改 SDK/依赖/build 输入、锁与配置复用 R9 no-UI 实体源码 fresh+locked consumer5/5；没有冒称新执行。SDK 仍不含 UI/Man 实现或 UI submit。 |
| AC-02 / V-02 | 原公开路径/Parser/VM/provider 身份和缺席面按 R9/R10 已通过证据复用；本轮正控又编译旧 IcedStyle/IcedAlign/ScrollStateEntry 新旧路径互通，构建形态按上表复验/复用。r3 inventory 例外边界未变。 |
| AC-03 / V-03 | FAIL：R9/R10 原生成例关闭，分组/声明操作数仍失败。新 runtime consumer5/5、四生成器/生成工程/792旧收据按未变相关源/输入边界保留；不把 work 的整族总数或旧 build 当本轮执行。 |
| AC-04 / V-04 | PASS 所验行为。八个真实浮层组合全部正确、事件/载荷/滚动/浮层定向65/65；其余 provider43/43与生命周期证据按未变实现/依赖/输入复用。恢复 T-04。 |
| AC-05 / V-05 | 选择面/发行边界保持：配置、别名、测试声明/cfg均未改，复用 R6/R8/R9 strict 映射与 R10 当前 t/tu5328/868列表；本次777实际绿。没有重新执行九档，也不采信叶名猜测证明。规范语义仍待 R9-01 修复后最终审定。 |
| AC-06 / T-10 | needs_fix；R9-01 OPEN、R11-01 OPEN。 |
| AC-07 / T-11/T-12 | merge-owned pending。 |
| AC-08..10 / M814-A | 已发布历史与既闭 R5/R6/R7 按原边界保留，不撤销 A。 |

R9 counter 双端三状态对照按未变输入/相关执行代码复用：当前 class 仲裁在该示例无 absolute/z 的路径结果未变，RustGenerator 的比较分类也不参与其 VM 按钮/文本路径。本轮未重新驱动 GUI，不宣称新截图或像素 parity。

SD-03 已把 no-op 精确为 `not(any(ui,ui-dialog))` 并说明 position 后继仲裁，与实际代码一致。SD-02 仍不能完整审定。11 个现行 canonical 源未变；impact metadata 保持 SDK/compiler modify、runtime add、GOAL-003/007。merge 门按主计划裸t+tv+tu+tt及必要docs/schema/book，交接旧“tt按需”不构成授权修改。

## 历史快照与路由

work 在 review-r10 中再生了五件旧 `*.at.rs`，它们不再匹配 R10 冻结 hash（`prior-artifact-drift.json`）；原输入和失败日志仍保留。本轮已复制到 review-r11 后重新生成并独立编译，未改 R9/R10 旧件。后续修复应再复制到工作证据目录，避免覆盖本轮冻结快照。

outcome: **needs_fix**；next_responsible_role: **work**，同一原树一次完成 R9-01 的表达式类型不变量与 R11-01 告警，更新 SD/上述具体覆盖矩阵后提交送审。

T-04 已独立验收复勾；T-05/T-08/T-09 继续开放，T-10..12 未完成。实际勾选 T-01/02/03/04/06/07/13/14/15，**executing 9/15**。

未修实施、发布 canonical Specs/live ledger、合入、归档或清树；未跑整档。最终 pass 之后仍由 merge 执行新冻结门禁、实际 B 合入/沉淀/发布收据；tf 主检出到期单实例，aavm 按用户原豁免范围。
