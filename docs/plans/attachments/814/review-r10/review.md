# PLAN-814 R10：R9 修复复审，needs_fix

2026-10-21 / stage: review / review_scope: final（T-10）/ plan_revision: 3。
reviewer_identity：本聊天 Codex 复审上下文，未参与实施，未创建 agent 或新 checkout。独立读取已提交修复并重放反例，不以 work 汇总替代验收。

## 冻结对象

- worktree：`D:/autostack/.wt/lang-814/auto-lang`；branch：`codex/plan-814-dev`。
- reviewed_commit：`236827f1c43dfa991e5a047a0eaa3e0ac0d07a38`；tree：`3ff79459fc69be4777419d97903d10bb206e1c90`。
- 修复基面：R9 的 `6063b7a94159026cd5876c1ad918e7a2e527597a`；整 B diff base：A 发布点 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；实施/依赖树初末 clean。
- 计划、交接、SD 分别冻结为 `plan-before-review.md`、`handoff-frozen.md`、`sd-frozen.md`。canonical 11 个 Spec 与 R9 完全相同，runtime Spec 仍待新增。输入、配置、源码与证据哈希见 `input-hashes.json`。

## R9 四项的实际关闭状态

| Finding | 复验结论 |
|---|---|
| R9-01 比较类型 | **PARTIAL / OPEN**。原已知形状的 bool/int/左右位、字符串/纯条件正控和 numeric-silent 产物均已修复并编译通过；新增的未知形状/Value prop 分流仍有反例，见下。不得记整项关闭。 |
| R9-02 旧样式公开路径 | **CLOSED**。真实外部编译 pin 验证旧 `ui::style::IcedStyle`、`ui::style::iced_adapter::IcedAlign` 与新 runtime 类型互通；ScrollStateEntry 新旧路径也为同一类型。style 门面不复制类型/主题状态，不引入 SDK/compiler 反向边。 |
| R9-03 声明构建形态 | **CLOSED（原构建矩阵）**。runtime no-default、bare ui、ui-headless、ui-dialog、code-editor 及默认形态 check 均 exit0；未用强开 iced 绕过 headless。其几何替换引入新运行行为问题，单列 R10-01。 |
| R9-04 生成器格式 | **CLOSED**。rust.rs 和本轮全部手写 Rust 改动 rustfmt exit0。整批文件检查只有受 freshness 管理的生成 shell-pack 排版不同；该文件原即生成布局，不据此要求重排全部生成资产或新增格式 finding。 |

四包 check exit0，不能解读成没有新 warning；本轮确有一条新的未用导入，见 R10-03。

## 仍开放的 R9-01 / P2：新分流只覆盖简单正整数，未闭合真实操作数类型

影响 AC-03/06、T-05/T-08/T-09；位置 `crates/auto-ui-compiler/src/ui_gen/rust.rs:8680`（`comparison_operand_type`）及 loop/prop 两个调用分支。

修复已保留集合的已知形状，但未知形状路径通过扫描另一侧 token 决定类型：扫描不收 `-`，所以负整数回落 str；数字开头（包括带小数点）一律被判 int。正常 AutoLang 数字表达式并没有被合同排除。按真实 parse→AURA→Rust 生成后，再在相同锁与消费程序下实际编译：

| 输入 / 形态 | 实际结果 |
|---|---|
| 未知形状 loop，`p.rank > 3`、`p.dirty == true` | 正控，编译 exit0 |
| 未知形状 loop，`-1 < p.rank` | 反向负数正控，编译 exit0 |
| 未知形状 loop，`p.rank > -1` | 生成 `p["rank"].as_str().unwrap_or_default() > - 1`；E0277，exit101 |
| 未知形状 loop，`p.score > 0.5` | 生成 `as_i64() as i32 > 0.5`；E0308，exit101 |
| 未知形状 loop，`0.5 < p.score` | 同样整数投影；E0308/E0277，exit101 |
| Value prop，`note.rank > 3`、`note.dirty == true` | 正控，编译 exit0 |
| Value prop，`note.rank > -1`、`note.score > 0.5` | 同源失败，分别 E0277/E0308，exit101 |

原 R9 六个输入及新组合均保存在本目录 `*.at`，未手改的生成物为 `*.at.rs`；`generation.log`、`controls.log`、`check-*.log`。`check.rs` 无 feature 时同时编译原例、相邻正控与类型身份 pin，exit0；`--features unknown-negative` 等逐项返回101。loop 使用合法空列表初值以触发未知元素形状，编译器仍须检查循环体。prop 经公开 AuraProp/Type::User 输入；消费程序提供 `type Note = serde_json::Value`，使声明的外部类型有真实定义，隔离字段投影错误。没有把最初探针遗漏 Note 定义产生的 E0425 算成产品问题。

这属于 R9-01 原“按真实字段/操作数类型处理比较、loop/prop×左右位”的未闭不变量，不重编号为另一个新 finding。修复需识别带符号及浮点操作数，并保持对称、已知形状和纯条件语义；不要把浮点数截成整数或只针对本次字符串特判。SD-02 中“数值→as_i64”的泛化不能作为削减既有数字语义的合同授权。

复现：先用本目录 `Cargo.toml` 的 `r10-generate` 生成输入（prop 带 `--prop`），再 `cargo check --offline --locked --manifest-path <本目录>/Cargo.toml --target-dir <原树>/target --bin r10-check --features unknown-negative`；其他 feature 为 unknown-float、unknown-rhs-float、prop-negative、prop-float。无 feature 必须先通过。

## 新 F814-R10-01 / P2：替换后的浮层扫描丢失 position 覆盖语义

影响 AC-03/04/06、T-04/T-08/T-09；位置 `crates/auto-ui-runtime/src/ui/aura_view_builder.rs:16295`。

为 headless 去掉 Iced 类型后，`extract_absolute_position` 改为纯 StyleClass 扫描，但只在遇到 Absolute 时把 `is_abs` 设 true，之后遇到 Relative/Fixed/Sticky 都不清除。原调用的 `IcedStyle::from_style` 对这些 position 类按顺序更新同一字段，后出现的声明覆盖前者。

独立程序通过真实 Parser→AURA→VmBridge→AuraViewBuilder 构建父 col 和含样式的子 col，观察实际返回 View 是否为 Overlay，并与未改动的原 IcedStyle position 规则对照：

| 子样式 | 原规则应 hoist | 当前实际 hoist |
|---|---|---|
| `absolute z-50 top-4` | 是 | 是 |
| `relative z-50 top-4` | 否 | 否 |
| `absolute relative z-50 top-4` | **否** | **是** |
| `absolute fixed z-50 top-4` | **否** | **是** |
| `relative absolute z-50 top-4` | 是 | 是 |

`overlay.rs`、`overlay-replay.log` 保存输入和区分性结果，mismatches=2、exit1。此为实际 view 结构差异，会把本应保留在流内的元素移入父容器浮层；不是缺像素截图的推测，也不是要求新增任意样式组合。fixed 的既有降级提示仍出现，但不能使此前未 hoist 的元素变成 absolute 浮层。

修复需保留完整 position 仲裁（包括后继覆盖和反向正控），可用无 Iced 依赖的单源派生函数；不能以纯扫描不调用解析器为由丢弃原派生语义。运行 `r10-overlay` 应在修复后归零并 exit0，现有 65 个相关测试通过不覆盖本组合。

## 新 F814-R10-02 / P2：777 的当前冻结基线过期，交接声称 strict pass 不成立

影响 AC-05/06、T-08/T-09。实际执行主合同命令：

`python scripts/modularity-audit.py --mode strict --no-cargo-check`

exit2，`strict verdict=fail violations=1 grandfathered=3`。唯一 violation 是：

> 锚定输入自 base_revision 起已变更: ['crates/auto-ui-runtime/Cargo.toml']；基线过期必须显式重采样，不能默默通行

本轮为 log 增加 std 是合理依赖修复，但基线还锚定 `463fd029cdfa372f3d8ecf406d5407d1e5c892d1`，没有覆盖该 manifest 改动。`modularity-summary.json` 保存精确分类；`modularity.log` 保留输出摘要。没有据此认定新增禁向依赖：ui_to_language 仍零命中，3 条既有窄豁免边没有扩大。

在同批全部代码修复完成、提交后按审计工具流程重采样并提交基线，重放 strict 得到真实冻结 SHA 的绿收据。不要把这个 work-owned V-05 门视作已执行，或改成关闭 staleness 检查；它也不是 merge-owned 裸 t/触面档的替代品。

## 新 F814-R10-03 / P3：移动几何类型后遗留新增 unused import

影响 AC-06、T-04/T-08/T-09；位置 `crates/auto-ui-runtime/src/ui/iced/scroll_state_reader.rs:27`。

ScrollStateEntry 的 serde derive 随定义迁到 mcp_types（使用完整限定名），原文件仍保留 `use serde::{Deserialize, Serialize}`，已无消费。matching 四包 check 明确报告该条 unused imports，runtime warning 235→236；R9 源码中这两个导入仍供原本的 derive 使用。本次仅锁定这条因果明确的新告警，不把历史警告、headless 新启用形态的全部 warning 或辅助探针 unused 项算入。

删除不需要的导入并重放匹配配置的 check，不能 blanket allow 掩盖。

## 验收结果、复用边界与规范

| AC / Case | R10 结论与证据 |
|---|---|
| AC-01 / V-01 | 复用 R9 的实体 no-UI 源码快照 fresh+locked consumer5/5。SDK/Core/方言源码、相关 manifests、锁、build 输入和测试配置未改，本批 runtime log std 不在 SDK 闭包；没有重跑却记新执行。 |
| AC-02 / V-02 | 原 style 路径和类型身份 pin 通过；六种构建形态通过；六 provider consumer43/43（6/10/8/5/7/7）独立重放。原 R9 Parser/Bridge/VM4 pin 源码及相关 Core 实现未改，按边界复用。r3 登记例外/装配权未扩大。 |
| AC-03 / V-03 | FAIL。原生成例编译关闭，但未知形状类型和新几何替换失败。新 runtime consumer5/5、事件/滚动/浮层等定向65/65通过。R9 counter 三状态证据按未变输入和相关实际分支复用，不宣称 R10 重新驱动双端或像素对齐。 |
| AC-04 / V-04 | PARTIAL。PAYLOAD_SEP/encode_payload/ScrollStateEntry 单源上移、旧路径再导出逐项读审，字节编码体/数据字段不变；i18n 门随实际模块、默认路径保持。provider43/43、定向65/65通过；浮层仲裁回归阻止完整行为保持。 |
| AC-05 / V-05 | PARTIAL。t/tu 当前重新 list 得5328/868，与 R9 冻结 B 唯一 ID 集合相同，missing0；其他七档沿未变别名/配置/测试声明和默认 feature 的 R6/R8/R9 strict 映射复用，未采信工作验证器叶名猜测泛化。777 实际红；792 模板/投影/driver/输入未改，旧24/24收据按边界保留，但不把它当本轮新执行或替代777。 |
| AC-06 / T-10 | needs_fix，R9-01 OPEN + R10-01/02/03 OPEN。 |
| AC-07 / T-11/12 | merge-owned pending。 |
| AC-08..10 / M814-A | 已发布历史及 R5/R6/R7 已闭边界保持，不撤销 A，不重开 inventory/原九档 strict proof/旧警告。 |

只跑定向验收、类型检查、list 和只读审计，未执行裸 t/tv/tt/tb/tu/taa 等整档。手写格式通过；生成 shell-pack 的格式差异由原生成/freshness 规则管理，不作为新增格式失败。

Spec delta 的 before 源与 11 个 canonical 输入仍和 R9 一致，operation 继续 SDK/compiler modify、runtime add。SD-02 的数字分流须随 R9-01 修复；SD-03 的“无 renderer 构建形态 no-op”也应在最终沉淀稿精确表述为 `not(any(ui,ui-dialog))` 才 no-op，bare ui/headless/dialog-only 仍装配其 feature 内能力。这是已存在实现的边界澄清，不是授权改成无宿主能力。主计划 §6 的 tt 仍 mandatory，交接旧“tt按需”不构成合同修订。

## 历史证据完整性说明

work 重放 R9 探针时覆盖了 review-r9 的五件 `*.at.rs`（`prior-artifact-drift.json`），其当前字节不再匹配 R9 `input-hashes.json`。原始 `.at`、R9 编译失败日志、报告/其他冻结输入仍在；不把新生成物冒充 R9 当时证据。本轮把输入复制到 review-r10 后重新生成、编译并冻结；后续 work 请复制探针到新修复目录重放，保留审查快照。该说明不增加产品 finding。

## 路由

outcome: **needs_fix**；next_responsible_role: **work**，原唯一树修齐 R9-01 的类型不变量、R10 几何/基线/健康三项，同步 SD 和具体覆盖矩阵，提交后再送 T-10。

T-04/T-05/T-08/T-09 继续开放，T-07 的测试身份核销保留；实际完成 T-01/02/03/06/07/13/14/15，仍 **executing 8/15**，T-10..12 未勾。

最终 pass 后由 merge 在新冻结提交统一执行裸 t+tv+tu+tt、必要 docs/schema/book 面，tf 主检出到期单实例，aavm 沿用户原豁免范围。本轮未修实施、发布 canonical Spec/live ledger、合入、归档或清树；B 尚未通过/交付。
