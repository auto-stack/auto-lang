# PLAN-803 final review R2 — needs_fix

2026-10-17；stage=review；review_scope=final；plan_revision=1。仍为独立审查上下文，未承担实现、未另开 agent/任务/树；本轮只验收和记录，没有修实现或合入。

## 基线与证据身份

- 原树 `D:/autostack/.wt/lang-803/auto-lang`，branch=`plan-803-dev`。
- reviewed_commit=`a9b401421d205fdfea5c9c8b66d195c8589771e5`；base=`1dd123361a75aa4b683376dcb2049299122724c9`；上轮=`bbad82a9f9a5b667d33fec28dcf680f6d95a1e7e`。
- main=`b7ecea5e36ddf41d9effc49f0aba53cca47b99ad`；同组 auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；rust-embed=8.13.0。
- 开始与结束实现树干净，HEAD、锁、测试配置和输入 hash 未漂移。详见 [baseline.json](baseline.json) 和 [Plan 入口快照](plan-before-review.md)。
- 冻结 SD：[spec-delta-reviewed.patch](spec-delta-reviewed.patch)，SHA-256=`dfa2aa491705bcebd5981aff70ef9773e0cac306f2741e71a924b0228a5b2156`。main auto-man Spec 与原基线无冲突增量；未发布 canonical Spec/ledger。

## 上轮 findings 闭合

| Finding | 独立核查 | 结论 |
|---|---|---|
| F803-R1-01 | 三个 trait 实现均恢复 `-> Filenames` 并调用底层 trait 入口；永久编译正控显式对三个旧类型赋值，等价覆盖 R1 原单类型反例；标准 Cargo 门面 9/9 通过。inherent 方法与 copy 求值/IO 算法未变。 | 原 API 缺口 closed；修复引入的新警告另列 F803-R2-01。 |
| F803-R1-02 | 对全部新增/修改 Rust 文件运行 rustfmt --check，exit=0；相对原基线 git diff --check，exit=0。旧断言期望值没有被修改成弱化 oracle。 | closed |
| F803-R1-03 | 核心模块/函数、测试说明、consumer 文档与 SD 均明确“零输出 IO”和 debug 动态读源码/release 内嵌边界；新永久 probe 重建包和独立消费者，原始日志确认实际目录移开和缺资源反控生效。 | closed |

## 新 finding

### F803-R2-01 / P2 / 修复回归：三处新增编译警告 / AC-05、T-03

`crates/auto-man/src/asset.rs:80/133/192` 的三个 `fn iter() -> Filenames` 各产生一条新的 `refining_impl_trait_reachable` 警告。标准 Cargo 门面命令虽 exit=0，但 auto-man 警告从 R1 的 28 条增为 31 条；新增三条全部位于本次修复行，不能归入原有警告基线。仓库 AGENTS.md §3 要求“zero unhandled compiler warnings”，当前健康门尚未完成。

返回具体类型本身是恢复旧公开 API 的正确修复，不应照通用 lint help 改回 opaque `impl Iterator`。应在三个相关 impl/方法的最小范围内，显式允许有意保留的 `refining_impl_trait`（或对应具体 lint），保留说明该细化是旧 API 兼容承诺的注释；不全仓屏蔽警告。提交后确认三条新警告消失，且三类型 `Filenames` 编译正控继续通过。

证据：[new-warnings.txt](new-warnings.txt)、[完整标准 Cargo 门面日志](facade.log)。R1 的 [warning-inventory.json](../review-r1/warning-inventory.json) 无此三项；实际代码差异也定位到本轮新增具体签名。

## 验收结果

| AC | 结论 | 证据与范围 |
|---|---|---|
| AC-01 | pass | `cargo test -p auto-man-assets` 6/6；独立消费者 4/4；依赖检查 PASS；打包 debug/release 各 4/4。 |
| AC-02 | pass | 三个旧类型的具体返回类型编译正控通过，全部资源键/顺序/hash/路径断言通过；当前全部 20 资源与 R1 Git-base/归档证据相同。 |
| AC-03 | pass | 标准 Cargo 门面 9/9（五族完整输出、Unicode、未知族、panic、转发、IAR/GHS 实际 exporter）；旧 asset 单测 1/1。R1 三种 IO 失败的错误/阶段/完整状态证据可复用，理由见下。 |
| AC-04 | pass | 新模式说明准确；实际打包及隔离消费者、真实缺资源失败；777 strict pass/0 violations；792/800 触面未扩大，SD 与剩余债一致。 |
| AC-05 | fail | 原三项 finding 全闭，但本轮有未处理的新编译警告 F803-R2-01，不能签发 final pass。 |
| AC-06 | pending | merge-owned 全档/Spec/ledger/归档/清理尚未执行。 |

独立执行（临时进程环境 `SCCACHE_SERVER_PORT=18426`，未改仓配置/依赖锁）：

- `cargo test -p auto-man-assets`：6/6，[assets.log](assets.log)。
- `cargo test -p auto-man --test plan803_asset_facade_tests`：9/9，[facade.log](facade.log)。R1 标准入口的环境阻塞本轮已排除，无需借助直接 rustc 代替本次 Cargo 证据。
- consumer manifest：4/4，[consumer.log](consumer.log)；Git Bash `verify-deps.sh`：PASS，[deps.log](deps.log)。
- `cargo test -p auto-man --lib asset::`：1/1，[asset-unit.log](asset-unit.log)；`--lib project_model_adapter::`：11/11，[project-model.log](project-model.log)。
- 定向 rustfmt / diff 检查均为 0：[format.log](format.log)、[diff-check.log](diff-check.log)。
- `python test/fixtures/modularity/man-assets-consumer/mode-boundary-probe.py`：exit=0，[mode-boundary.log](mode-boundary.log)。已检查其真实构建入口、隔离副本范围、rename/finally 恢复和子步骤非零传播；运行前核实递归清理目录位于原树 target 内且非 reparse point。debug/release 基线各 4/4；移开源码资源后 debug=101/release=0；实际移出一个资源后 debug=101。原始明细：[debug](mode-packaged-consumer-debug.log)、[release](mode-packaged-consumer-release.log)、[debug 移开](mode-relocated-debug.log)、[release 移开](mode-relocated-release.log)、[缺资源](mode-missing-resource.log)。没有把预期的非零误称普通验收全绿。
- 当前 `.crate` 源文件逐字等于受审源码，20 项资源 hash 等于 R1/旧 Git 源：[packaged-identity.json](packaged-identity.json)。
- `python scripts/modularity-audit.py --root . --mode strict --output <report>`：strict pass，violations=0，grandfathered=2，[audit.json](audit.json)。

复用边界：Cargo.lock、根 manifest、`.cargo/config.toml`、边界配置均与 R1 hash 相同；三组资源 20/20 相同；核心规划函数仅文档变更，门面 copy/导出器/上游求值代码未改。故复用 [R1 三种 IO 失败重放](../review-r1/io-failure-replays.log) 的原错误/阶段/事后状态证据及 792/800 触面检查；本轮仍重跑全部门面、路径适配和真实包消费。未以“同一测试总数”代替代码/输入身份核对。

SD-01 的构建模式限定已对齐真实行为，不新增全模式内嵌承诺，也不削弱原“无文件物化”的验收条件；SD-02 分工和遗留失败语义无变化。两个 docs/specs 路径及 GOAL-003/GOAL-016 元数据保留有效，不需 semantic revision 变更。

## 状态与路由

outcome=`needs_fix`；仅重开 T-03，保留 T-01/02/04/05/06 完成及已过验收证据；Plan=`executing`，current_step=5/8。旧 R1 三项关闭，新 F803-R2-01 开放；不能以记债批准延期。work 在同树局部处理有意的 lint、提交，再复核新警告闭合和 API 正控。下次 readiness 须带新 finding 的诊断/最小修复范围与验证说明，不重做已闭合行为设计。

pending_closeout_gates：最终 pass 后由 merge 在新冻结 SHA 跑裸 `cargo t`，资源 crate、独立 consumer，以及分别运行 auto-man 的 `--lib asset::`、`--lib project_model_adapter::`、`--test plan803_asset_facade_tests`；tv/tu 无触发，tf 仅主检出批量到期判定。Spec/ledger/索引/归档/guard 清理 pending。本轮未运行整档、未合入。
