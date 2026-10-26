# PLAN-841 R2 修复验收与两轮失败诊断

2026-10-26，stage=work。R2 基线为 lang `a0473d515` / UI `a056dd601`；R2 修复提交以主计划 §9 最新记录为准。

## 两轮 final 失败后的诊断与覆盖修正

R1 的关闭矩阵错误地把 SDK backend 名称等同于全部真实工程入口：`transpile_aura`、`ui_build`、compiler/runtime 已有控制未覆盖 `auto-ui-tools` 的 request 分派、缓存和 warn-and-continue，因此 Jet/Ark/Rust 工程写盘漏检。R2 改为从 `tool_ops` 的真实 request 消费者反向对账到各编译 helper，永久测试必须经 `GenerationRequest` 和实际项目生成函数，并同时断言具名错误与零输出。

R1 手动 codec 只证明合法信封和深度，没有“完整 artifact 已耗尽”的反例；因此从 `from_slice` 改为手动 deserialize 后丢失尾随检查未被发现。R2 将垃圾字节、第二份 JSON、旧 schema 第二信封及合法空白列为同一失败出口控制，同时保留深度、字段/span/source_lines、schema/tag/语义和 Weak 断言。

## 真实入口 → 共享准入 → 错误传播

| 路径 | 准入和失败时序 | 永久 oracle |
|---|---|---|
| compiler Rust/Vue、SDK、runtime（R1 已修） | `parse_validated_ui`/`lower_ui` 返回 lowering 后 owned `UiCode`；R1 反例继续复用，未因 R2 改变其代码路径 | R1 compiler 3 项本轮复跑通过；runtime 2、SDK 1 的 R1 证据按代码路径未变复用 |
| tools Rust request/full、`regenerate_code_only`、delivery | 全部汇入 `compile_rust_ui_app`；编译集（含 bps 补集）先经 `validate_ui_file_with_modules`，预扫描也接 `lower_ui`；准入错误不再被编译循环吞掉。`generate_rust_ui_inner` 先完成编译再建 workspace/输出目录 | 真实 `generate_rust_ui_with_request` 重复 widget/store/model/message 和导入模块均 Err 且输出目录不存在；合法项目成功 |
| tools Jet full/incremental | full 与 incremental 均在 cache load/dirty 判定前校验 front 树；compile helper 同时接 `lower_ui` | 真实 request full 反例及 `from_workspace_incremental_with_request` 反例均 Err、零输出；合法 full 成功 |
| tools Ark full | front 树先校验，再进入编译与项目生成 | 真实 request 反例 Err、零输出；合法项目成功 |
| tools Vue full/incremental | full 校验 front 与 dep front；incremental 在 cache load 前校验；`handle_compile_error{,_with_dep_shape}` 对领域准入错误始终 Err，不受 strict/dep 降级影响 | 真实 `VueProject::from_workspace_with_request + generate` 和 `incremental_compile_changed` 反例均 Err、零输出；合法 full 成功 |
| codec | `serde_stacker` 保持深视图支持；信封后 `Deserializer::end()` 强制完整消费，仅空白后缀合法，再做 domain/schema/领域校验 | 原 R2 三后缀均拒绝；合法空白、48 层和旧控制全过 |

`auto-ui-tools/tests/ui_domain_validation.rs` 直接经 `parse_file -> GenerationRequest::from_descriptor` 驱动 Vue/Jet/Ark/Rust 工程，覆盖四合法控制、四领域错误家族×四 backend、四导入模块反例和 Jet/Vue incremental。消息 payload 形状由共享 codec 反例覆盖，不伪造 parser 无法产生的源码形式。

## 最终定向结果

- 原 `review841_r2_tools`：1 passed；Jet/Ark/Rust 三合法对照成功，三重复 widget 均 `accepted=false, output_exists=false`。
- `cargo test -p auto-ui-tools --test ui_domain_validation -- --test-threads=1`：4 passed（内部实际执行 4 合法工程、16 领域家族拒绝、4 导入拒绝、2 incremental 拒绝）。
- 原 `review841_r2_codec_rejects_trailing_data`：1 passed；`garbage`、`{}`、`{"schema":0}` 均 `accepted=false`，合法空白成功。
- `cargo test -p auto-ui-compiler --test ui_domain_validation -- --test-threads=1`：3 passed。
- `cargo test -p auto-ui-dialect codec::tests -- --test-threads=1`：9 passed，含 trailing、payload、48 层、字段/span/source_lines、schema/tag/语义和 Weak。
- R2 触面 `rustfmt --check` 与 `git diff --check` 通过；临时 probe 已删除。

无开放合同内 finding 或已知覆盖缺口。R1 的 48 层和格式问题保持关闭。统一裸 `cargo t`、适用 `tv/tt/tu`、完整发行/777/792 与三仓交付门仍是 final review 后的 merge-owned pending；本记录不代表已执行整档、合入或 canonical/ledger 沉淀。
