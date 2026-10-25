# auto-ui（独立 UI 仓）

> **Status**: active（PLAN-831 迁仓落定）
> 仓库：既有 auto-ui 仓（本地 Git；未推远端） | workspace：crates/* 五包 + 受管 Cargo.lock

## 职责与边界（SD-01，as-built）

auto-ui 仓持有 UI 生产实现五包：auto-ui-dialect / auto-ui-compiler / auto-ui-runtime / auto-ui-tools / shell-pack（目录均为 `crates/<包名>`）。语言基础 SDK/轻数据、auto-project-model、std-only auto-tool-execution 与 auto-api-gen adapter 仍由语言仓单源持有；UI 不得反依赖 Man 管理实现（auto-man-core）或旧 auto-lang 巨包。旧 auto-lang 留语言仓作为上层兼容组合：公开路径经真实 UI owner 再导出（C4），不进入纯 SDK 闭包。

## 来源与身份

- 迁移基线：语言仓 e199caf8a 五包内容（T-04 导入窗口 841 文件逐字节恒等；此后演进有逐提交归因，见归档 Plan 831 source-audit）。
- 开发解析：三仓同级 path 组（组内兄弟相对 path + 受管锁）；发行消费使用登记固定源码/内容寻址包，禁隐式主检出回退。
- 最终版本由三仓固定 commit 与发行源码清单共同标识，不以 path 依赖或包版本 0.1.0 代替源码身份。

## 测试与档位（SD-05 汇总）

UI 仓自持 t / tu（单进程 ui_gen 档）/ ti（集成档）/ tn（native 手工门）档位与 `.config/nextest.toml`；测试 ID 与语言仓基线对拍零丢失（tu 863 唯一恒等）。blueprints 单源居语言仓 packages/blueprints（编译期兜底=组内兄弟探测，独立 clone 生成工程不在无兄弟布局支持矩阵内）。

完整批量面由语言仓 `cargo tf`（`scripts/run-full-regression.py`）串行调度两个实际 workspace。UI 子门显式选择 compiler/dialect/runtime 三包 lib、compiler/ui feature，并使用语言仓原 `nextest-full.toml`，包含 ui_gen 全族；UI 日常 t 不能替代这个完整面。迁仓前 3,091 个唯一 UI 测试 ID 与当前选择逐项核对，固定清单与可执行选择守卫在语言仓 `scripts/full-regression-*-{ids,selected}.txt`。各语言 feature 档分别持有完整基线，不能与 tf 基集相交而丢失特有项。新增 ID 单列；删除、ignore 或 profile 排除原必选 ID 会使完整门失败。（PLAN-850）

## 边界

- jet/ark 真实构建验证面：经用户 2026-10-24 裁定（r3）转后续环境门控渠道（KNOWN-DEBT 承接）；生成面全 profile 真实生成在合同内。
- 本仓发行/安装/切换事务合同见 [repository-boundaries/ui-repository-delivery.md](../repository-boundaries/ui-repository-delivery.md)。
