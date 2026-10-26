# PLAN-841 r1 — R2 affected final review

2026-10-26，**needs_fix**。本会话 Codex `/root` 延续 R1 独立审查上下文，未参与实施；未启动子代理或新审查树，未修产品、合入或发布 canonical Specs/ledger。

## Baseline and scope

既有实施树为 `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，均 `codex/plan-841-dev`。进入/退出均 clean，HEAD 未漂移：

- lang `a0473d515129ad1016a164fc0f3201c9334805cc`，修复 base `acd8d99d2eb180f4861fb235d783f19dba307a38`。
- UI `a056dd60171b9f4a1ee61fc1ade32b45d4a4ba1c`，修复 base `f7eeb82d0170f69a6a53cf0e4d625caea7eddab9`。
- Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982`，未变；down clean clone `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未改。

本轮按 R1 的三个 finding 和 [work 修复收据冻结副本](repair-r1-closeout.md) 检查实际修复及相邻不变量：生产入口接线/失败前零产物、codec 新解码器的合法/非法输入，以及触面格式。原收据位置为实施树 `docs/plans/attachments/841/repair-r1-closeout.md`。未重跑全档、整套 792/777 或无关已闭验收。

主计划从语言主检出读取，语义合同仍 revision 1。输入 hash、依赖 Cargo/配置 hash、主检出与候选 Spec 逐文件 hash、probe hash 见 [baseline.json](baseline.json)。完整候选 SD 冻结于 [spec-delta-reviewed.diff](spec-delta-reviewed.diff)，SHA256 `84019277499b4edc6c73d68f6db3bc58a0074c37c2a8e8b902de96da2a90379f`。不是批准沉淀的规范。

## Finding closure

| Finding | R2 result | Evidence and boundary |
|---|---|---|
| F841-R1-01 P1 | **OPEN，旧例关闭/全不变量未关闭** | 原 Rust/Vue 高层函数确实拒绝重复 widget；compiler 永久 3 测、runtime 根/导入 2 测、SDK 1 测通过。但真实工具工程 Rust/Jet/Ark 仍绕过 validator，见下。 |
| F841-R1-02 P2 | **CLOSED，本基线** | 原 48 层 4253B 反例及 4/16/32 对照都可 decode；既有/新增 codec 7 测通过。新解码器引入另一个失败退出回归，独立编号 F841-R2-01，不把它冒充旧反例未修。 |
| F841-R1-03 P2 | **CLOSED，本基线** | 与 R1 相同 rustfmt 命令检查三仓完整 Plan 差量内存在的 181 个 Rust 文件，0 fail；三个修复 diff 的 `git diff --check` 通过。见 [fmt.log](fmt.log)。 |

### F841-R1-01 / P1 / OPEN — 工程生成入口仍消费未校验领域模型

违反原计划 AC-02/04/05、T-05/T-08/T-09，及 R1 的“全部声明的生产入口在生成/执行/写盘前拒绝无效领域数据”关闭条件。这是 R1 同一缺口的范围未闭合，不是新功能要求。

真实协议分派 `auto-ui-tools/src/tool_ops.rs:204/208/212` 分别调用以下正式 request 入口：

- `rust_ui::generate_rust_ui_with_request` → `compile_at_file`，`rust_ui.rs:868..871` 仍 `parse_domain -> UiCode -> generator`。
- `jet::generate_jet_project_with_request` → `JetProject::from_workspace_with_request` → `compile_at_file`，`jet.rs:623..626` 同样直接解析/提取。
- `ark::generate_ark_project_with_request` → `ArkProject::from_workspace_with_request` → `compile_at_file`，`ark.rs:364..367` 同样直接解析/提取。

这些文件在本次 UI 修复 diff 内没有修改；整个 tools/src 仍无 lower_ui/parse_validated_ui/validate_ui 接线。修复只覆盖 compiler/SDK/runtime 的部分高层面，不能代替这些生产消费者。

**持久输入/setup**：[review841_r2_tools.rs](review841_r2_tools.rs)。每次使用独立临时项目，`pac.at` 为 `name: "review841" / scene: "ui" / render: "<jet|ark|rust>"`，`src/front/app.at` 分别写：

```auto
widget App { view { text "ok" {} } }
widget App { view { text "ok" {} } }
```

通过真正 `auto_project_model::parse_file -> GenerationRequest::from_descriptor` 后调用上述 request 入口，输出为项目内 `review-output`，不调用构建工具链。相同项目只保留单 widget 为合法对照。

**预期**：三路坏输入均返回具名领域验证错误，零生成输出；三路单 widget 成功。

**观察**：[engineering-entrypoints.log](engineering-entrypoints.log) 的 6 次真实生成调用中，三个合法对照成功；三个重复 widget 输入均 `accepted=true, output_exists=true, error=None`。Jet/Ark 的实际工程日志显示 `Widgets: App, App` 并生成同名文件；Rust 也报告成功生成。测试 exit 101，断言报告三路全不满足拒绝/零输出。无需 Gradle/DevEco/Rust 宿主构建环境即可复现。

**已证原因**：上述正式入口和私有编译 helper 未调用当前单源 validator/lowering；不是 probe 绕开正式入口或用不合法 pac 配置。SDK 的 jet 分支已修，并不意味着工程工具的 jet 分支也已修。

**关闭 oracle/受影响范围**：把实际 Rust/Vue/Jet/Ark 工程生产路径（含 incremental、直接源码、导入模块/必要旁支）逐项对账到单源准入点，保留本例与已有 compiler/runtime/SDK 反例；代表 widget/store/model/message/payload 无效数据必须在产物生成前拒绝，合法项目继续生成。Jet/Ark 现有 warn-and-continue 错误吞咽也须在涉及该准入失败时证明不会最终返回成功/半壳工程。永久测试应走真正 request 工程入口，不能继续只测 SDK 同名 backend 或裸 generator helper。修复采用何种接线由 work 决定，不要求新框架或全矩阵回归。

### F841-R2-01 / P2 / OPEN — codec 修复后丢失完整输入校验

**修复回归**；AC-03/05、T-02/T-06/T-08/T-09。版本化 codec 的信封准入合同与原完整 JSON 解码行为要求对传入 artifact 全量判定；不能将合法前缀视为整个数据有效。

位置 `auto-ui-dialect/src/codec.rs:25..43`：从 `serde_json::from_slice` 改成手动 `UiEnvelope::deserialize(serde_stacker::Deserializer::new(...))` 后，没有检查原 deserializer 是否已到输入末尾。domain/schema/tag 和新的语义验证只检查第一个信封。

**输入/入口**：同目录 [review841_r2_probe.rs](review841_r2_probe.rs) 的 `review841_r2_codec_rejects_trailing_data`。使用真实 parse 的单 widget，`encode_ui_code` 得到合法字节后分别追加 `garbage`、`{}`、`{"schema":0}`，调用 `decode_ui_code`；另以只追加空白/换行/tab 为合法对照。

**预期**：三个非空白后缀受控拒绝，单信封/合法空白成功；保留深视图和旧 domain/schema/tag 的既有断言。

**观察**：[codec-trailing-controls.log](codec-trailing-controls.log) 中三个后缀全部 `accepted=true`，exit 101；同测试中的完整 JSON 解析控制 `serde_json::from_slice::<serde_json::Value>` 对三者均拒绝，排除输入未实际追加或只是合法空白。合法空白对照成功。原 `from_slice` 处理整个输入，新调用只消费合法前缀，属本轮明确行为变化。

**关闭 oracle**：恢复单 artifact 的完整消费检查，三个后缀均明确拒绝；合法空白、4/16/32/48 层、字段/span/source_lines、坏 domain/schema/tag/语义与 Weak 控制保持。检查新手动解码流程的相关失败出口，不通过退回固定 128 层限制或修改信封承诺来关闭。

## Checks → invariant / AC evidence

| Check | Result | Scope |
|---|---|---|
| R1 原始 probe + codec 尾随 holdout | 4 pass / 1 fail | 原 Rust/Vue、嵌套48、codec基本控制、Core parity expr/fn/scene/task 均通过；失败为新 suffix case。见 [independent-probe.log](independent-probe.log)。 |
| 后缀三兄弟及完整 JSON/合法空白控制 | 1 test fail，三坏值全被误收 | 新手动 decoder 失败退出；未扩大深度/内存要求。 |
| request 工程入口 | 1 test fail，6 次实际调用 | 三合法成功，三无效生成仍成功。 |
| compiler `ui_domain_validation` | 3 pass | 本次新增的高层校验/根/单层跨文件；[log](compiler-validation.log)。 |
| dialect `codec::tests` | 7 pass | 深度修复、语义拒绝、旧信封、字段/span/source_lines/Weak；[log](codec.log)。 |
| runtime `plan841_domain_validation` | 2 pass | 根和导入模块的新校验与恢复合法对照；其他过滤为0的不计证据；[log](runtime-validation.log)。 |
| SDK `plan841_ui_domain_validation` | 1 pass | rust/vue/jet/shadcn 写盘前拒绝与合法控制；[log](sdk-validation.log)。 |
| rustfmt | 181 files / 0 fail | 全 Plan 触面，不含临时审查 probe；三仓 diff-check 通过。 |

AC 更新：AC-01 保留 R1 已审中立分类/无反向实现依赖证据（本轮 validator 在 UI owner，Core 端口未改）；AC-02/04 因 F841-R1-01 仍 fail，已通过的高层子路径证据保留；AC-03 原深度项 pass，但新 F841-R2-01 使整体 fail；AC-05/06 fail；AC-07 merge-owned pending。

## SD, evidence reuse and next

SD 的校验 owner 与深度修复叙述基本吻合；关于所有生产消费者和 codec 完整准入的当前态仍须由上述两项关闭后才能沉淀。SD-01 的分类/AST删除未改；SD-02..05 按影响面不能最终 pass。两个额外规范目标 `auto-lang/project.md`、`auto-ui-dialect/design/full-grammar.md` 与 SD-02/04 的明确映射建议仍在，不以新框架或新增支持范围替代问题修复。

R1 的未变语义/模型/golden 证据按对应文件范围保留。792/777 原收据只作历史来源和未变项依据：本轮加入 serde_stacker 并改变生产准入，旧发行成功不自动证明新依赖/新入口；work 应刷新受影响来源/consumer 证据，最终统一交付仍由 merge 在冻结输入执行。本轮未对旧 39/39 或外部 48 测作无条件全新pass声称，也未重跑完整发行矩阵。

两轮 final 未通过后，下次 ready handoff 应先给出**诊断与覆盖修正**：解释 SDK backend 与实际 tool 工程入口为何漏对账，列清各真实生产入口/共享准入/错误传播的映射与具名定向控制；手动 decoder 的完整输入与失败出口同时纳入。先修本轮所有 finding、定向复证并提交，再受影响复审；不要边修单 finding 边送审或跑整档。

主计划回到 **executing 4/12**：重开 T-02/T-05/T-06/T-08/T-09；保留 T-01/T-03/T-04/T-07 已完成子产出。F841-R1-02/03 记录 CLOSED，F841-R1-01 OPEN，新 F841-R2-01 OPEN。统一裸 cargo t、适用 tv/tt/tu、三仓交付/Spec-ledger/归档/guard 收尾仍 merge-owned pending。未合入，不放行 U841。

临时 probe 源与日志已复制到本目录，并从原实施树撤去；没有产品实现改动。实施代码保持上述冻结 commit，最终状态见 final-state.json。
