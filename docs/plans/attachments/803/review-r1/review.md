# PLAN-803 final review R1 — needs_fix

2026-10-17；stage=review；review_scope=final；plan_revision=1。

本审查上下文未执行本计划实现，也未继承实现会话；独立从当前主检出 Plan、Spec、原树代码与反例重建结论。未创建其他 agent、任务或 worktree；未改实现、合入或发布规范。

## 冻结基线

- worktree：`D:/autostack/.wt/lang-803/auto-lang`；branch：`plan-803-dev`。
- reviewed_commit：`bbad82a9f9a5b667d33fec28dcf680f6d95a1e7e`；实现提交：`904b5e5e0a17ea99669f8f1e1da9be0d232c2b22`。
- base_commit：`1dd123361a75aa4b683376dcb2049299122724c9`；主检出：`9932962bc157be13bcaa0520011c7c78082cb2fe`。
- dependency_revisions：rust-embed 8.13.0；同组 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；rustc 1.98.0。Cargo.lock SHA-256：`2f111a7cba16f5502d49ab147b89e81f254dd9647dec4b8b4569732ea9e0f421`（未跟踪锁，复用证据必须再次核对）。
- 开始与结束的实现树 tracked/untracked status 均为空；Plan 原文见 [plan-before-review.md](plan-before-review.md)。
- Spec 输入版本/hash 见 [baseline.json](baseline.json)，代码与配置 hash 见 [source-hashes.json](source-hashes.json)。当前主检出 auto-man Spec 相对代码基线没有冲突增量。
- 两项 SD 的冻结原文见 [spec-delta-reviewed.patch](spec-delta-reviewed.patch)，SHA-256=`5a379a28589af4dcc00baa991191a00ba5b9217f19434a9582d0373d3ee14f1b`。本轮不向主检出沉淀 canonical Spec/ledger。

## Findings

### F803-R1-01 / P2 / 公开 API 回归 / AC-02、T-03、T-04

`crates/auto-man/src/asset.rs:75/123/177` 的三个 `RustEmbed::iter` 手写实现返回 opaque `impl Iterator`，丢失原 derive 实现返回的具体 `rust_embed::Filenames` 类型。旧的合法调用：

```rust
let _: rust_embed::Filenames =
    <auto_man::asset::Templates as rust_embed::RustEmbed>::iter();
```

在旧 derive 正控编译成功，在当前公开门面编译失败 E0308（expected `Filenames`, found opaque type）。这违反本计划明确承诺的旧 RustEmbed 公开面兼容；泛型枚举并 collect 的现有测试无法检出此差异。GHSTemplate/IARTemplate 的手写实现具有同样缺口。

修复方向：三个 trait 实现保留旧具体返回类型，转发底层 **trait** 入口，避免通过 inherent opaque 返回值再次隐藏具体类型；加入显式返回类型的编译正控。无需调整资源或渲染行为。

证据：[旧正控](old-api-control.log) exit=0；[新反例](new-api-counterexample.log) exit=1；[重放脚本](replay.py)。该脚本的 derive 正控使用与基线相同的 rust-embed 版本、模板布局及原始声明形态，只将资源 manifest 根指向迁入后的唯一资源。

### F803-R1-02 / P2 / 必需健康检查未通过 / AC-05、T-02、T-04

新增的 `crates/auto-man-assets/src/tests.rs`、`crates/auto-man/tests/plan803_asset_facade_tests.rs` 和独立消费者 `tests/assets_consumer.rs` 均未通过 rustfmt。`git diff --check 1dd123361a..HEAD` 还在门面测试末行报 new blank line at EOF。属于本次新增内容，不是仓库全量格式基线问题。

修复方向：只格式化本次新增/修改文件，清除末尾多余空行；提交后复核定向格式检查与 diff 检查，不要求整理无关旧文件。

证据：[format.log](format.log) exit=1；[diff-check.log](diff-check.log) exit=2。

### F803-R1-03 / P2 / SD/能力说明缺少构建模式边界 / AC-04、T-06

核心模块文档和 `plan_template_family` 注释声明“无任何 IO”，SD-01 的 project.md 也无条件描述嵌入字节和零 IO；实际 workspace/独立 consumer 只启用 rust-embed compression，没有 debug-embed。默认调试态的 `iter/get` 会遍历/读取宏展开时记录的源码目录，计划会间接发生读 IO；移开该目录后真实消费者从 4/4 变为 1/4，计划静默返回空。发行态则内嵌，移开相同目录后仍为 4/4。

修复方向：按计划 §5 的无物化承诺，准确区分“无输出 IO/不创建目录、不渲染、不写文件”和 rust-embed 调试态读盘，写明发行态内嵌/重定位已验证的构建模式；保留该模式边界的可重复反控。**本 finding 不要求私自改变旧调试态行为或新增全模式内嵌承诺**；若选择改变运行行为，需评估功能统一及门面兼容影响，不能仅靠改文案冒称已经实现。

证据：[package-probe.py](package-probe.py)、[debug 移开源码](packaged-binary-without-source-assets-debug.log) exit=101、[release 移开源码](packaged-binary-without-source-assets-release.log) exit=0。仅对 target 下本探针解包的资源重命名并恢复，未触碰实际 crate 资源。

## AC → task → artifact → evidence

| AC | 结论 | 核查与证据 |
|---|---|---|
| AC-01 | pass | T-02/04；资源单测 6/6、独立 consumer 4/4、normal/build/dev 闭包无语言/Man/VM/UI；解包后消费者 debug/release 各 4/4。无源码资源运行的边界见 F03，不推广为全模式保证。 |
| AC-02 | fail | 20/20 资源的基线/当前/归档键和 bytes 全等；13/3/4 枚举顺序、路径规划和原始 `${name}` 保持；但具体 trait 返回类型不兼容，F01。 |
| AC-03 | pass | T-03/04；当前源码门面编译、8/8 门面测试；五族覆盖全部 13 模板，Unicode、未知族及 `..` panic；IAR 实际 exporter 生成四类文件、GHS default 输出后旧 app mold 失败；额外三种 IO 失败反例新旧原始代码的错误及完整目录/文件状态逐项相同。 |
| AC-04 | partial | T-05/06；资源归属、打包消费、777 strict pass 均通过；800 的 resolver/Target/pac.lock 无改动；792 的资源入口 `delivery_copy_resources(front_dir)` 只收 src/front 非 .at 文件，与三组模板无交集，选定运行闭包未引入 auto-man/auto-man-assets；SD 分工及三项历史债已核查，但构建模式说明需修正，F03。 |
| AC-05 | fail | 公开面、SD、健康检查有 F01..03，不能签发 final pass。 |
| AC-06 | pending | merge-owned，未运行整档/合入/归档/清理。 |

### 定向结果及限制

- `cargo test -p auto-man-assets` 首次正常运行 6/6；保存的同源单测重放：[assets-unit.log](assets-unit.log)。
- `cargo test --offline --manifest-path test/fixtures/modularity/man-assets-consumer/Cargo.toml` 4/4：[consumer.log](consumer.log)；Git Bash 执行 `verify-deps.sh` exit=0：[deps-gitbash.log](deps-gitbash.log)；含 dev 边的补充检查：[consumer-tree.log](consumer-tree.log)。
- 普通 `cargo test -p auto-man --test plan803_asset_facade_tests` 先被 sccache 保留端口拒绝；关闭包装器后的锁解析又被镜像缺 `zvariant 5.15.1` 阻止（[facade.log](facade.log)、[facade-offline.log](facade-offline.log)）。这些是环境限制，不计为实现回归。
- 未修改依赖锁，使用 Cargo 本轮实际输出的完整 rustc 参数，仅绕过 sccache，重新编译当前 auto-man，再编译当前门面测试：8/8，见 [direct-library-build.log](direct-library-build.log)、[direct-facade-build.log](direct-facade-build.log)、[direct-facade-tests.log](direct-facade-tests.log)。这是真实代码/断言证据，**不冒称标准 Cargo 重跑已通过**；标准入口待环境恢复后重验。
- 独立选择的失败轴：目标根为文件、首文件已覆盖后第二目标为目录、libdemo 嵌套 apps 被文件占用。以 `git show <base>:.../asset.rs` 原始文件为旧臂，均返回 Err，且错误与完整事后目录/字节状态一致，见 [io-failure-replays.log](io-failure-replays.log)。原有输出打印保留；无新增调试输出路径。
- 从实际 `.crate` 归档解包后独立编译轻 consumer；归档 Rust 源码也与受审源码逐字相同。全部 20 个资源逐项与 Git 基线及当前源码比较，通过：[package-hashes.json](package-hashes.json)。实际把一个资源移出解包根再跑，确实非零拒绝：[packaged-missing-resource-control.log](packaged-missing-resource-control.log)。非仅检查打包清单。
- 777：`python scripts/modularity-audit.py --root . --mode strict --output <report>`，strict verdict=pass、violations=0、grandfathered=2：[audit.json](audit.json)。没有重采样或放宽禁向边。
- 当前 auto-man 编译有 28 条旧警告，位置均不在本次修改行；上游 auto-lang 另有预存警告。新增门面/独立包未引入警告，清单见 [warning-inventory.json](warning-inventory.json)；不把预存警告描述成零警告。

### SD 审查

SD-01 的所有权、依赖边界、三组资源、字节/路径规则与实现一致，运行模式限定受 F03 约束；SD-02 的薄门面、Auto/IO 留上层、旧失败行为和剩余三项债均符合代码。F01 修复前不能宣称“旧公开面不变”。保留 frontmatter：supersedes=`docs/specs/auto-man/project.md`、new=`docs/specs/auto-man-assets/project.md`、goals=`GOAL-003/GOAL-016`，均为真实路径/目标。无额外抽库、模板行为修复或未批准延期。

## 状态与下一步

outcome=`needs_fix`；Plan 回到 `executing`；T-02/03/04/06 重开，保留 T-01/05 完成及历史证据；current_step=2/8。F01..03 是待关闭合同/健康/SD 缺口，记债不代表批准延期。由 work 在同树修复、定向验证并提交，再对新 SHA 复审受影响内容。

pending_closeout_gates（最终 pass 后归 merge，在新冻结 SHA 上）：裸 `cargo t`；新 crate 和外部 consumer；`cargo test -p auto-man --lib asset::`、`cargo test -p auto-man --lib project_model_adapter::`、`cargo test -p auto-man --test plan803_asset_facade_tests` **分别执行**。原 work 收据将多个 filter 放进同一 cargo test 命令，不能照抄。未触 VM/ui_gen，无 tv/tu；tf 仅主检出到期判定的批量回归。Spec/ledger/索引/归档及 guard 清理均 pending，本轮未合入。
