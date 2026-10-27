# PLAN-842 T-01 上游安装交付缺口（开放）

2026-10-27，实际三仓基线 lang `ecc78701124fe6e0ab6f0e262148a1d853ba2af5`、UI `4903bef64532750d1597627b1197f3d2cac1effc`、Man `9a75d3bab6eaeda961d5dcbf7932258f89e650af`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。这不是本次产品修改引入的回归；三仓源码 HEAD 保持上述基线。

## 实际正反控与结果

- 在同组执行 `python scripts/man-install.py install --profile full --install-root D:/autostack/.wt/lang-842/probe-install-full`：exit 0，受管清单记录 6800 文件和上述四仓身份；其后 `verify --profile full` exit 0。清单 `source_closure.packages` 将 UI 四包标为 auto-ui，`auto-lang-core` 标为 auto-lang。
- 从 `test/fixtures/modularity/generated-ui-consumer/proj` 复制真实 Rust UI 工程到 `D:/autostack/.wt/lang-842/probe-generation-project`，清除 AUTO_* 源根、工具覆盖、`CARGO_MANIFEST_DIR`、`CARGO_TARGET_DIR`、`RUSTC_WRAPPER`；运行测试安装的 `auto.exe gen --delivery-root D:/autostack/.wt/lang-842/probe-generation-delivery`：exit 1，实际 tool 已接受 generate 请求，报 `delivery: 投影修剪不完整（期望 2 处，实际 0 处）——producer 清单结构漂移`；原目标目录未创建。该失败发生在计划要求证明的真实安装 CLI 入口，不是仅以辅助函数推断。
- 诊断性因果对照：仅将该安装内 `sdk/crates/auto-lang/Cargo.toml` 的 `test-cases = ["auto-lang-core/test-cases"]` 临时改为 `test-cases = []`；同一真实 CLI/工程/环境/目标 exit 0、产物 5419 文件，清单的 `producer.source_root` 指向测试安装 `sdk`。测试结束原字节恢复，实际安装 `verify --profile full` 再次 exit 0。成功产物来自被临时修改且当时与安装清单 hash 不符的 SDK，**只是定位因果的实验产物，不可作为有效安装或发行验收**；未执行其外部 build。
- 测试安装恢复后 SHA-256：`auto.exe` `9d27e52910040175f331d0b33774b2c13fb168a45a87f8ed7031b14d6ac6abfa`；`auto-ui-tool.exe` `f38c40ecd00073ea5d7653709aa3d5bc5a0069fbc5416a66d29b0a1bd1ce5dd9`；`install-manifest.json` `997be24eab1188be220de2bfe093461949768b2e73d5b638d7cc8c2c585c8c95`；安装态 `auto-lang/Cargo.toml` `0f0574d19581260fb8edc0577529fd1fda78568d7a447bbf6b00df0ebd40965c`。安装和诊断工程/产物留在同组以供重放，不是最终交付。

## 已证实原因与归属边界

Man `scripts/man-install.py::project_missing_optional` 对缺席 `auto-test-case` 删除直接依赖，但保留 `auto-lang-core/test-cases`；UI `rust_ui.rs::delivery_apply_projection` 既要求原始完整两项，又只接受完全投影的 `test-cases = []` 作为幂等形态，所以该部分投影在真实已安装 SDK 上被拒。`git blame` 显示 Man 投影逻辑来自 832 的 `2419e57e`，UI 规范形态来自 `c6d3ac66`；`git show` 验证 832 交付 Man `1b7a9ce0` 的逻辑、831 交付 UI `aee499e` 的投影规则、832 交付 lang `3934ed4f` 的原始 feature 都包含相同相关形态。**本轮没有重建并运行 832 历史冻结三仓组合**，因此以上来源不代替其历史环境实测，也不自行撤销归档收据。

现行 [generated-ui-distribution](../../../specs/repository-boundaries/generated-ui-distribution.md) 的 PLAN-816 节明确安装形态 `gen --delivery-root` 使用邻接 SDK；832 V-03/AC-03 要求真实安装 UI 生成、rust 真实构建。842 自身还要求真实安装/canonical source 与升级恢复，但明确不接手 831/832 尚未完成的搬仓验收；[共用合同](../../reports/m2-governance-order-841-844.md) §3 规定这些发行缺口仍归原交付 owner。当前可观察到的是 **841 交付后的实际安装组合**违背现行已承诺入口；不得把当前失败算 842 的新机制选择、静默接受或延给 843。

## 关闭条件与下一责任

由原发行责任范围处理：在独立、合法的修复计划/工作树中修复安装投影与发行投影的组合，保 `test-cases` 合法选择/关闭语义和其他 projection，按实际支持的已安装 SDK 形态验证；对正常/已投影/半投影/破损输入分别有区分性控制。使用真实 Full 安装，清源根和目标覆盖，以安装 `auto.exe gen --delivery-root` 实际成功并确认使用邻接 SDK；fresh/locked 外部工程构建与坏源失败保旧/恢复、Core 对照依现行合同验证，重绑版本/二进制/source 收据并交付。842 的同名旧影子源和未知清单版本另为其范围内待办，不能由该上游修复的单个正例冒充通过。

原 831/832/841 已归档，不能直接把归档计划复活；应按仓库工作流为上游缺口建立新的修复合同并完成其交付，或由用户明确授权对 842 合同作有界修订并重新核验启动/验收条件。之前 gate 判断基于归档收据，现有实机反例需在下一次启动前对其适用性作显式核对。
