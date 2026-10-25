# PLAN-831 UI 发行记录（ui-release.md，T-11）

日期：2026-10-25。交付渠道=本地 Git 固定 commit（未推远端，如实记录）。

## 固定版本（交付收据读回）

| 仓 | 固定 commit | 说明 |
|---|---|---|
| auto-lang | `e1c640477884cf8720c5a53ca3b9df355085a786` | 迁仓+切源+兼容（T-04..R6 全部 24 提交 rebase 落定 + SD 沉淀提交） |
| auto-ui | `aee499e87093ff727f68ac8467c506e9ecf6ecdc` | 五包 owner（T-04 导入..R6 + T-11 840 并集移植 aee499e） |
| auto-down | `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | 只读依赖（未触碰） |

- 交付工具二进制身份：auto-ui-tool sha256 `627893f7f675d380…`（792 收据 delivered_tool，scratch-consumer/driver-receipt.json）。
- 语言消费者切换：组内兄弟相对 path + 受管锁（master 落地后主检出即为同构布局）。
- 四状态切换事务收据：prepared→ui-landed→lang-switched→verified（T-11 switch 事务，收据 scratch-ui-repo-fixture/）。

## Spec 来源读回（SD-01..07 沉淀清单）

- docs/specs/auto-ui/project.md（SD-01 add，PLAN-831）
- docs/specs/repository-boundaries/ui-repository-delivery.md（SD-02 add，PLAN-831）
- docs/specs/repository-boundaries/project.md（SD-03 modify，PLAN-831）
- docs/specs/repository-boundaries/generated-ui-distribution.md（SD-04 modify，PLAN-831）
- docs/specs/auto-ui-compiler/project.md（SD-05 modify，PLAN-831）
- docs/specs/auto-ui-runtime/project.md（SD-05 modify，PLAN-831）
- docs/specs/auto-ui-tools/project.md（SD-05 modify，PLAN-831）
- docs/specs/auto-ui-dialect/project.md（SD-05 modify，PLAN-831）
- docs/specs/language-sdk/project.md（SD-06 modify，PLAN-831）
- docs/specs/project-model/project.md（SD-06 modify，PLAN-831）
- docs/specs/auto-lang/project.md（SD-06 modify，PLAN-831）
- docs/specs/auto-man-core/project.md（SD-06 modify，PLAN-831）
- docs/specs/auto-man/project.md（SD-06 modify，PLAN-831）
- docs/specs/auto-api-gen/project.md（SD-07 add，PLAN-831）

## 支持矩阵（r3 合同边界）

- 必需面实测绿：vue/rust/vscode 真实构建、vm 真实运行（Launched+探针+收口）、六 profile 真实生成、fresh/locked/整组重定位、隔离 SDK 反控。
- 获准延期（r3 用户裁定）：jet/ark 真实构建→后续环境门控渠道（KNOWN-DEBT 承接；缺环境逐项记录，在场真实失败传播红）。

## 后继

U831 于 T-12 收尾闭环（归档+清理收据读回）后成立；832 届时放行（消费本记录固定版本）。
