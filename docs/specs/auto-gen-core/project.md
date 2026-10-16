# auto-gen-core

> **Status**: active
> 路径：`crates/auto-gen-core`  | 技术栈：Rust（regex / thiserror / auto-val）

无解释器生成核心（PLAN-791 从 auto-gen 拆出）：保护区段处理、输出路径/文件编排、通用生成错误与单次生成报告。可被无 auto-lang/auto-man/auto-vm/UI 闭包的外部 consumer 独立消费（外部消费方探针 `test/fixtures/modularity/gen-core-consumer`，闭包断言 `verify-deps.sh`）。

## 目标与范围

- 输出编排：默认命名（模板名剥一个尾缀 `.at`）/output_name 覆盖/嵌套目录按需创建（仅成功时）。
- guard 语义：既有文件保护区段保留（`GuardProcessor::merge`）、`overwrite_guarded` 覆盖、冲突检测；legacy 字节兼容 feed（`feed_guarded_legacy`，与 merge 语义并存、见下"冻结语义"）。
- 失败反控：render/guard 失败不触碰既有输出文件；`dry_run` 零文件系统效应；单模板错误收集进报告、批次继续。
- 渲染边界：`TemplateRenderer` trait 返回 owned `RenderedTemplate` 文本；core 不持解释器/AST/模板类型，语言侧关切（数据加载、库路径、模板解析、执行错误映射）全部留在 adapter。
- 不做：数据加载、模板解析/执行、`use` 库解析、CLI——这些是 auto-gen（Auto 可执行模板 adapter）的职责。

## 依赖边界（红线）

- 依赖仅 `auto-val`（AutoStr，保持公开类型与历史 API 同型）+ regex + thiserror。
- **禁止**依赖 auto-lang / auto-man / auto-vm / 任何 renderer/UI 栈；禁持 `Shared<AutoInterpreter>` 或 AST `Code`。闭包由 gen-core-consumer fixture + module-boundaries strict 基线钉住（man-ecosystem 域，`man-ecosystem-no-lang-monolith` 禁向规则适用）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| error | GenError/GenResult/SourceLocation（单一定义，auto-gen re-export） | active |
| guard | GuardProcessor/GuardedSection/Conflict + feed_guarded_legacy（legacy 字节兼容 feed，单一定义） | active |
| generate | 输出编排（generate/OutputConfig/TemplateSpec/RenderedTemplate/TemplateRenderer/GenReport） | active |

## 冻结语义（PLAN-791 T-01 旧臂，改动=行为变更须显式决策）

1. **阶段边界**：adapter 侧数据/库文件加载失败=顶层 `Err`（发生在 `generate` 之前）；单模板失败=报告内收集、批次继续。
2. **写入次序**：先 render 后 guard 合并再写盘——失败模板永不覆写既有输出；`dry_run` 连目录都不创建。
3. **字节语义**：guard 合并输出每行带 `\n`（含尾换行）；`feed_guarded_legacy` 输出 `join("\n")` 无尾换行、marker 前缀精确匹配、id 捕获额外要求尾随 ` ---`、文件缺失时原样透传、读失败 panic（legacy 同款）。两算法语义不同且**并存**，不得互改。
4. **TemplateSpec.rename** 字段为兼容携带，core 不消费（与拆分前输出命名一致）。
5. **GenReport.duration**：core 计自身循环耗时；auto-gen adapter 覆写为整调用耗时（含数据加载阶段，历史语义）。

## 验证

- core 单测 12 例（fake renderer 真实落盘/guard 保留/overwrite/dry_run 零写入/失败不覆写/嵌套目录仅在成功时/legacy feed 字节/double-.at 命名）。
- 外部消费方探针：`cargo test --manifest-path test/fixtures/modularity/gen-core-consumer/Cargo.toml`（4 例）+ `bash test/fixtures/modularity/gen-core-consumer/verify-deps.sh`（闭包无 auto-lang/auto-man/auto-vm）。
- adapter 级端到端（Auto 模板、legacy、CLI）由 auto-gen 的 V02 金样/V03 失败语义套件钉住（`crates/auto-gen/tests/`）。

## 已知债与未迁项（诚实登记）

- auto-gen 的 AutoFile/AutoCode 数据源与 lib_files 定义在当前解释器下对模板不可见（作用域错配，仅 `DataSource::Atom` 可渲染；fstr_note 在共享解释器渲染路径无效）——预存缺陷随拆分原样保留在 adapter，修复属行为变更、需独立决策。
- auto-man（exporter/builder 经 legacy AutoGen/Mold、assets/templates 可执行模板）仍未迁离语言栈；auto-gen→auto-lang 边在 module-boundaries 基线 grandfather（exit R5）。
