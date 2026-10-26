# PLAN-841 R1 修复验收

2026-10-26，stage=work。基线为 review-r1 冻结的 lang `acd8d99d2` / UI `f7eeb82d0`；实际修复提交以主计划 §9 最新 work 记录为准。

## Finding 关闭矩阵

| finding | invariant 与受影响路径 | 修复及永久控制 | 当前结果与剩余边界 |
|---|---|---|---|
| F841-R1-01 | 原始解析成功不等于领域准入；公开 Rust/Vue、文件生成、组件包、runtime/SDK、导入模块与 decode 均须在生成/执行/写盘前拒绝重复 widget/store/model/message 和坏 payload | dialect `validation.rs` 单一定义；compiler `parse_validated_ui`/`lower_ui` 返回 lowering 后的 owned `UiCode`；各生产入口接线。`auto-ui-compiler/tests/ui_domain_validation.rs` 3 项覆盖公开入口、四类无效数据、跨文件与合法控制；SDK `plan841_ui_domain_validation` 覆盖 rust/vue/jet/shadcn 及写盘前失败；runtime `plan841_domain_validation` 2 项覆盖根/导入模块和合法 VM 控制 | 全部通过；无开放合同内缺口。模块错误还发现并修复 `VmBridge::new_from_decls` 未 drain 的同型缺口，现与 `new_with_children` 共用检查 |
| F841-R1-02 | parser/encoder 已接受的嵌套视图必须能 decode 自身产物，保持字段/span/source_lines，坏 domain/schema/tag 与无效领域仍拒绝 | `serde_stacker` 扩展递归工作栈；只解除 serde_json 固定递归层数限制，不新增任意深度上限。`codec_roundtrips_supported_nested_views` 永久覆盖 4/16/32/48 层；`codec_rejects_decoded_invalid_domain_data` 与既有坏 envelope 控制保留 | 原 48 层 4253B 反例现 decode/re-encode 成功；codec 7 项全绿。字段/span/source_lines/Weak 既有断言继续通过 |
| F841-R1-03 | 三仓完整差量内存在的 Rust 文件须满足既定 rustfmt 命令，不顺手格式化无关文件 | 对 lang/UI/Man 各完整 diff 与新增 `.rs` 文件逐个执行 `rustfmt --edition 2021 --config skip_children=true`，再以 `--check` 全量复核 | 全部通过；`git diff --check` 亦通过。Man 无 Rust 差量 |

## 最终定向重放

以下均在最终修复代码上执行，`RUSTC_WRAPPER=` 置空：

- 原 review probe 两项：`review841_public_generators_enforce_domain_validation` 与 `review841_codec_roundtrips_supported_nested_views` 各 1 passed；临时 probe 已删除。
- `cargo test -p auto-ui-compiler --test ui_domain_validation -- --test-threads=1`：3 passed。
- `cargo test -p auto-ui-compiler --lib ui_ast::tests -- --test-threads=1`：4 passed，包含 lower→VM、四生成器、五族和跨文件合法对照。
- `cargo test -p auto-ui-dialect codec::tests -- --test-threads=1`：7 passed。
- `cargo test -p auto-ui-runtime plan841_domain_validation -- --test-threads=1`：2 passed。
- `cargo test -p auto-lang --test plan841_ui_domain_validation -- --test-threads=1`：1 passed。
- `test/fixtures/modularity/ui-dialect-consumer` 中 `cargo test -- --test-threads=1`：48 passed，补上 review 因磁盘满中止的外部方言复跑；本次生成的未跟踪 Cargo.lock 已移除。
- `cargo check -p auto-ui-runtime`：通过。测试编译同时覆盖 compiler、dialect、runtime 与 SDK 最终修复单元；runtime 216 条和 SDK 3 条警告为既有基线归因，本轮未新增未处理警告签名。

## 交接边界

R1 的 AC-01 与既有 792/777 证据所涉代码语义未被本轮扩大修改；候选 SD 已按实际校验 owner、codec 深度行为、SDK/runtime 准入和模块错误传播刷新。统一裸 `cargo t`、适用 `tv/tt/tu` 与三仓统一交付门仍为 final review 后的 merge-owned pending；本记录不冒称已执行或合入。
