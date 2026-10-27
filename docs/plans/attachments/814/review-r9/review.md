# PLAN-814 R9：完整 T-10，needs_fix

2026-10-21 / stage: review / review_scope: final / plan_revision: 3。
reviewer_identity：本聊天 Codex 复审上下文，未参与 B 实现；未创建 agent 或另一 checkout。结论由实际源码、独立反例和定向运行重建，work 的汇总不代替独立验收。

## 冻结基线

- 唯一树：`D:/autostack/.wt/lang-814/auto-lang`；分支 `codex/plan-814-dev`。
- reviewed_commit：`6063b7a94159026cd5876c1ad918e7a2e527597a`；tree：`08bdc88381af60e06b0e7ac250a344f6ba8f1963`。
- B base：A 实际发布点 `f6e07aceeeea3cb12698c747d61db5be43f7458c`。
- auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，原组内兄弟树，HEAD/洁净状态前后复核。
- 实施和依赖树均 clean；只向主检出写复审附件、计划/风险簿记。审前计划、交接和 SD 已分别冻结为 `plan-before-review.md`、`handoff-frozen.md`、`sd-frozen.md`；源码/配置/Spec/证据哈希见 `input-hashes.json`。

## R8 三项修复复验

| 旧项 | 本轮结论与边界 |
|---|---|
| R8-01 双端 oracle 未完成 | CLOSED。独立复用技能驱动，002-counter 的 Vue 和 VM 均实际完成 0→-1→Reset→1；MCP 保存 snapshot/state/操作返回值，Vue 保存每态截图。`vm-replay.json`、`vue-replay.log` 和本地截图为证。此为代表性功能/状态对照，不宣称两端已有相同像素布局。 |
| R8-02 SD 只有提纲 | 原结构/正文/来源缺口 CLOSED。新 SD-03 有全文，SDK/compiler/compat/发行各增量提供 A 源 blob；四个具名 before 均与当前 canonical 相符。新发现的代码语义/公开面/feature 缺陷仍阻止该正文最终审定，须随修复同步 SD-02/03/04。B 影响元数据按 SDK/compiler modify、runtime add 规范化，历史 A add 事实保留。 |
| R8-03 两个新增 warning、fixture 格式 | CLOSED（原例边界）。四包 matching check exit0，runtime warning 237→235，两条新 unused reexport 消失；runtime lib/consumer rustfmt exit0。没有把旧 warning 总量当新增，也不重开 R5/R6 的既闭警告。完整生成器另有本轮发现的格式问题，见 R9-04。 |

双端命令沿 `autoui-verifier` 标准脚本/client；VM 是本轮独立端口、独立 PID，结束时仅关闭本次进程。运行使用 work 的 `auto.exe`（07:04 构建）；780→6063 仅 runtime 测试再导出 cfg 与 fixture 排版，运行路径/输入/依赖未改，因此限定复用该可执行文件。当前源码仍另行 check 和定向测试，不把可执行文件冒充 6063 重编结果。截图持久位置与哈希见 `input-hashes.json`。

交接把输入写作 `src/app.at` 是路径笔误；本轮读取并冻结的实际文件为 `examples/ui/002-counter/src/front/app.at`、pac.at 与 `gen/front/vue/src/App.vue`（`ui-inputs/`）。Vue 服务进程实际加载该示例 `gen/front/vue/node_modules/vite`，实际 main.ts 导入 `/src/App.vue`；对照到真实三按钮/计数逻辑，未对一个不存在的输入文件声称哈希绑定。

## 新 OPEN findings

### F814-R9-01 / P2：比较修复越过形状类型，生成不可编译的组件

影响 AC-03/06、T-05/T-08/T-09。位置：`crates/auto-ui-compiler/src/ui_gen/rust.rs:8739`（loop 字段）及 `:8784`（Value prop 同类分支）。

新增 `comparison_context` 一旦识别比较，就在 `elem_shape` 之前强制 `.as_str().unwrap_or_default()`。同一个已知记录 `[{ dirty: true, rank: 7, running: "1" }]` 的真实解析→AURA→Rust 生成结果：

| 输入条件 | 生成/真实编译结果 |
|---|---|
| `p.running == "1"` | 字符串正控，exit0 |
| `p.dirty` | 仍按已知形状 `as_bool`，正控 exit0 |
| `p.dirty == true` | `p["dirty"].as_str().unwrap_or_default() == true`；E0277，exit101 |
| `p.rank > 3` | `p["rank"].as_str().unwrap_or_default() > 3`；E0277，exit101 |
| `true == p.dirty` | `true == ...as_str()`；E0277，exit101 |

输入、未手改的生成物、生成程序在本目录 `*.at`、`*.at.rs`、`probe.rs`。实际组件编译正控 `control-final.log`、反例 `final-bool-case.log`/`final-int-case.log`/`final-rhs-case.log`。当前 loop 分支原有 `value_field_access_shaped` 正是 bool/int/list 形状出口，不能被字符串反例修复覆盖掉。两侧数字字段同时误投字符串还可能不报型错而改变比较结果；`numeric-silent.at.rs` 仅保存了该生成现象，未冒称执行验证。

修复须按真实字段/操作数类型处理比较，保留已知形状，覆盖 loop/prop、左右位置、布尔/数值/字符串及纯条件正控；不要只对 `running` 特判或改 golden。SD-02 当前“比较位统一 as_str”也须随之纠正。

复现：先用本目录 Cargo manifest 的 `r9-comparison` 生成输入，再在 `style-check/Cargo.toml` 下 `cargo check --offline --locked --features bool-case`（int-case/rhs-case 同理）；无 feature 为两个合法正控。该辅助编译程序含 compat 依赖以复用根锁上下文，只证明产物类型，不拿它证明 canonical 零旧包闭包；后者由独立 runtime consumer 验证。

### F814-R9-02 / P2：旧公开 Iced 样式路径丢失，现有 API pin 没覆盖到

影响 AC-02/06、T-05/T-08/T-09。位置：`crates/auto-ui-runtime/src/ui/mod.rs:129`；旧包 `src/lib.rs:1907` 直接再导出该 ui 模块。

原基面 203e2ce91 的 `ui/style/mod.rs` 在 ui-iced 下公开 `iced_adapter` 和 `IcedStyle`。当前 `ui::style` 整体别名到 compiler，而 compiler 不含运行时 adapter；类型挪到 `ui::IcedStyle` 并不能保持原路径。真实外部 pin：

```rust
auto_lang::ui::style::IcedStyle
auto_lang::ui::style::iced_adapter::IcedAlign
```

分别 E0425/E0433、exit101，见 `final-old-path.log`；同一程序的新 `auto_ui_runtime::ui::IcedStyle` 路径和两个生成正控编译通过。源码对照/输入是 `legacy-style.rs`，复现 feature `old-path`。四枚现有 V-02 pin 虽过，不能证明未触达的这些公开路径仍在。

这是本 Plan 拆包后的完整兼容要求遗漏；最早 A 拆 style 时即留下，不据此撤销 A 已发布历史或重开 A 任务。B T-05 必须补兼容再导出，保持同一类型/主题状态及单向依赖，补旧完整模块路径 consumer，不恢复两份实现。

### F814-R9-03 / P2：声明支持的 headless 构建仍编译 Iced 代码

影响 AC-02/03/06、T-04/T-05/T-08/T-09。位置：`crates/auto-ui-runtime/src/ui/mod.rs:130`，结合 `Cargo.toml:17`。

实际命令：`cargo check --offline --locked -p auto-ui-runtime --no-default-features --features ui-headless`，exit101，39 个编译错误，见 `runtime-headless.log`。明确的拆迁缺门：`style_iced_adapter` 及其再导出无 cfg，但其 `effective_border_radius`、padding、颜色函数直接引用 optional `iced`。原 203 的该模块和导出都有 `cfg(feature="ui-iced")`；当前 headless feature 并不启用 iced。

还同时暴露 session/anchor_slot/iced 相关引用和 logger 的条件依赖问题，不能只补第一条门就声称该配置可构建。这里不声称原库所有裸 ui/headless 组合在迁移前都绿（sched_diag 等存在既有缺门）；本次实际验收对象是新 runtime 明确公布并被旧 feature 转发的 headless 组合。合同 V-02 feature 开关/AC-02 组合面、SD-03 的公开 feature 不能在仅测试默认 ui-iced 后整体通过。

恢复 Iced 专属门，并核对被公开声明支持的无后端/Ui/headless/default 组合及缺席语义；不能用让 headless 强制拖入 iced 来消除错误。若要取消该支持面，须正式修订合同，不由 review 静默削减。

### F814-R9-04 / P3：完整生成器文件仍有新增格式失败

影响 AC-06、T-05/T-08/T-09。`rustfmt --edition 2021 --check --config skip_children=true crates/auto-ui-compiler/src/ui_gen/rust.rs` exit1，`compiler-format.log` 有六处 diff。

包括新增 `comparison_context` 的长条件、`:8673` 把函数头和 `let result` 挤在同一行、比较分支排版，以及 canonical 字符串变长后的代码。A 发布源码该函数头/let 原为两行；这是 B 新差异，不能被 R8 五文件格式检查覆盖。仅清本 Plan 触面并执行相应格式检查，不要求重排无关全仓代码。

## 全部 AC/V 与其证据边界

| 范围 | 本轮验收 |
|---|---|
| AC-01 / V-01 | PASS 所验独立闭包。实体源码快照排除 old auto-lang、UI 三包及 Man，17 个 Core 路径 crate，fresh lock + locked 外部 consumer 5/5；`no-ui-source.log`。SDK 本批只新增诊断带外通道、test 执行族，复核单源。四非 UI consumer 的 Cargo 切换/DAG 与既有 A 证据按未变边界复用。 |
| AC-02 / V-02 | FAIL。Parser/Bridge/VM/type identity pin 4/4；六个 no-UI provider 独立 consumer 43/43（6/10/8/5/7/7）含缺席、显式覆盖、两 VM；旧样式路径和 headless 失败。r3 inventory 具名例外保持：SDK 不含 UI submit/工厂，compiler/compat 的登记权未越界，runtime 零 submit。 |
| AC-03 / V-03 | FAIL。同源 counter 双端三状态对照、canonical import pin 1/1、新 runtime 外部 consumer 5/5（真实 desktop 资产动态装载/VM）通过；Rust 生成器新比较行为/feature 配置失败。其余四生成器迁移原始算法未在本修复批变更；既有四生成器/e2e 证据限定复用，不把工作总数写成本轮整档运行。 |
| AC-04 / V-04 | PASS 所验范围。独立 runtime provider/frame/i18n/theme/mem_guard 定向 44/44；六外部 consumer 43/43。读审 SDK 唯一钩子/进程默认、runtime 单点安装、原线程亲和/TLS/terminal/media/editor owner、退出代码及资源随迁；未新增复制状态。headless 失败另计 AC-02/03，不能据默认面过测隐去。 |
| AC-05 / V-05 | PARTIAL。当前九档 ID 全部与 R8 冻结 B 清单同集合；复用 R6 原→A strict proof + R8 无叶名推断的 A→B 路径核销，九档无丢失。777 strict violations0；792 work 原始收据 24/24 已读取冻结，按未变发行模板/依赖/输入和本轮仅 cfg/排版变更边界复用，不冒称本轮重跑 792。SD 正文结构补齐，但须同步新修复。 |
| AC-06 / T-10 | needs_fix，四项 OPEN；不勾 T-10。 |
| AC-07 / T-11/T-12 | merge-owned pending，尚无 B 交付收据。 |
| AC-08..10 / T-13..15 | A 已发布历史保留；既闭 R5/R6/R7 按未变边界复用，不重开 inventory、strict proof 或警告旧项。 |

九档当前唯一 ID：t5328、tf6279、tv163、tt5700、tb5304、tu868、taa6275、ta6729、t3 6730；都是 list，**没有执行这些整档**。`selection-summary.txt`/`current-*.txt`。tu 旧清单包栏写通用 libtest，首次审查脚本未规范化该标签出现 868 个假 missing；校正成真实 compiler owner 后是 0。不是产品丢测，不新增 finding。原图/源码映射未用唯一叶名猜测替代。

真实反例编译之前，简化独立 manifest 的 fresh-lock 曾先遇到已知 Windows/wgpu-hal 多版本冲突，裸 ui 探针也先遇到 feature 缺门；`generated-*.log`/`ui-only-control.log` 保存诊断。**它们不证明比较反例的类型错误，也不记为新发行 finding**。最终 `style-check` 使用从根锁派生的依赖上下文，正控 exit0、各负控分别到达自己的 E0277/E0425/E0433，才是上述结论的证据。

## 路由与收尾

outcome: **needs_fix**；next_responsible_role: **work**，原树同批补齐四项，更新 SD/公开面/feature 与对照矩阵后提交再次送审。实施修复期间只跑相关反例、模块/consumer 和必要 check。

T-04/T-05 重开，T-08/T-09 保持开放；测试身份和 R8 健康原例已核销，T-07 复勾。实际完成 T-01/02/03/06/07/13/14/15，**executing 8/15**；T-10..12 未完成。

最终 pass 之后，merge 在新冻结提交统一执行主合同的裸 t+tv+tu+tt、必要 docs/schema/book 面；tf 仅主检出到期单实例。aavm 豁免沿用户原裁定范围。未改实施/canonical Specs/live ledger，未合入、归档或清树；B 尚未通过/发布，不重新封锁 A 已发布的接口。
