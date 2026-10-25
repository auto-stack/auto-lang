# PLAN-832 T-04 管理/CLI 导入台账

## 固定来源与目标

- source：`D:/autostack/.wt/lang-832/auto-lang` @`1cf2a9b97b0ddf4155fc1ab56b259a88c701f891`；生产包内容继承 T-01 基线 `45558ab628…`，T-03 未改这些 package。
- target：`D:/autostack/.wt/lang-832/auto-man`，旧基线 `51fafe82e8e0c4e66325ee14088eed8c3270e07e`，分支 `codex/plan-832-dev`。
- fixed 依赖：UI=`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；语言中立源仍由 auto-lang 提供。
- 完整逐文件 hash 见 [source-audit.json.txt](source-audit.json.txt)：185 文件，**175 same / 10 manifest-rewrite / 0 missing / 0 unexpected-drift**。rewrite 只含 9 个 package manifest 与 3 个 consumer manifest 的路径；package-resolver/toolchain-core/man-assets 及其余 consumer manifest 无需变化。

## package 映射

| source→target | same / manifest rewrite | 目标职责 |
|---|---:|---|
| `crates/auto` → 同路径 | 16 / 1 | 薄 CLI、双 profile、命令测试 |
| `crates/auto-man` → 同路径（替换旧 0.1.4 实现） | 39 / 1 | 0.1.0 compat facet + UiOps；不是 Core |
| `crates/auto-man-runtime` → 同路径 | 47 / 1 | 非 UI 管理/服务/编排 |
| `crates/auto-man-core` → 同路径 | 14 / 1 | 管理纯规则 |
| `crates/auto-package-lock` → 同路径 | 5 / 1 | pac.lock core |
| `crates/auto-package-resolver` → 同路径 | 2 / 0 | 依赖路径规则 |
| `crates/auto-toolchain-core` → 同路径 | 4 / 1 | 工具链纯规则 |
| `crates/auto-man-assets` → 同路径 | 23 / 0 | 五族模板、GHS/IAR 资源与规划 |
| `crates/auto-tool-client` → 同路径 | 2 / 1 | I-TOOL client/discovery |

旧 `crates/auto-man` 的 legacy 文件由 Git 历史和 T-02 快照保留；其 assets 的现行生产源已是 `auto-man-assets`，旧 `am` main/旧集成测试不属于 816 现行 CLI 测试面，未复制为第二生产实现。旧 `auto-up`、根 `automan/*.at` 暂不挂入新 9 成员 workspace，T-09 最终来源/残留审计须明确处置。

## consumer 映射

导入到 Man `test/fixtures/modularity/`：man-core、package-lock、package-resolver、toolchain-core、man-assets、tool-client 六组。Man 所属包路径仍指本仓 crates；lang 的 val/project-model/protocol/execution 改为 `../../../../auto-lang/crates/...`。three-repository-consumer 留语言仓作跨仓编排探针，T-08 再切 Man 固定 source。

## workspace 与路径决策

- Man 根 workspace=9 成员，default member/run=`auto`，resolver=2；根第三方依赖承接现行版本，dev profile 保持。
- crate 内 Man 包继续 `../<package>`；crate 到组内语言/UI 源统一 `../../../auto-lang|auto-ui/crates/...`。初次将语言路径少写了一级，metadata 明确报缺 `auto-man/auto-lang/...`，已纠正后重验。
- 根 `[patch.crates-io] iced_widget` 指向仍属语言仓的 `../auto-lang/crates/patches/iced_widget`；未虚构 UI patch 源。
- `cargo metadata --no-deps --format-version 1`：9 members，九个 package 均 0.1.0；旧 Man auto-man=0.1.4 已被现行 0.1.0 替代。
- 不复制 auto-tool-protocol/auto-tool-execution/project-model/AST/Value/lang-core，不复制 UI 实现；auto-api-gen/auto-gen/auto-cache 留语言 adapter/基础设施。

## 许可与资源

- 目标根 `LICENSE`=MIT（2025 衍星居士），语言源根 `LICENSE`=MIT（2024 Soutek）；两份均保留，不以一个版权声明覆盖另一个。
- package-lock/package-resolver/toolchain-core/man-assets manifest 标 `license="Proprietary"`，保持原字段；其余 package 未声明 license，不擅自补写。
- crate 内 assets/embed/tests 全部按源 hash 导入；未另发现 package 级 LICENSE/NOTICE 文件。T-05/T-07 仍需验证独立包资源可消费及许可随发行闭包旅行。

## 后续门

本任务完成候选导入与 metadata/source 对账，不宣称独立构建或安装已完成。T-05 负责独立锁、normal/build/dev Core 闭包、安装与非 UI 命令；T-06 才接 fixed UI tool 做完整三仓正控；T-08 才切语言消费者并删除原件。
