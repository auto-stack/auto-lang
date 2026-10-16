# PLAN-795 final review R1 — needs_fix

2026-10-16；plan_revision=1。Reviewer：本聊天 Codex，按实施以外交接上下文独立重建证据，未派 agent；未暴露具体模型/session ID。

受审树 `D:/autostack/.wt/lang-795/auto-lang`，branch `plan-795-dev`；HEAD `11ddc162ecd839b98cee748e3d2c27cfd05df301`，diff base `579f6dec0941fb711664679a9cb8c29fa71bad20`。入场/退场 git status 均空，HEAD 未变。兄弟 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。本次未修实现、未合并、未归档。

## Finding

**F-795-R1-01 / P2 / 原验收证据缺口 / AC-04、AC-05 / T-05、T-06**：777 strict 基线没有与新增 crate、根 members、auto-man manifest 和域配置绑定。工作回执称仅 `config_drift_since_baseline`，并把重锚推迟到 merge；独立实跑实际有两项红：`config_drift_since_baseline` + `stale_baseline`。完整输出见 [boundary.json](boundary.json) 的 `strict.violations`。复现：在原树执行 `python scripts/modularity-audit.py --mode strict --output <新报告路径>`，exit=1，`strict verdict=fail violations=2 grandfathered=2`。

stale 输入明确为 `.autoos/module-boundaries.toml`、`Cargo.toml`、`crates/auto-man/Cargo.toml`、`crates/auto-package-lock/Cargo.toml`。新增两边 auto-man→auto-package-lock、auto-package-lock→auto-val 均 allowed；cargo metadata --locked 对照 45 声明边、零差异。故不是新增禁向依赖，而是当前工作验收的基线尚未成立。T-05 明定 metadata/777 边界检查，不能用 consumer PASS 替代 strict；791 final R2→R3 也要求在 work 修复同类 stale 后才能 pass（已查主检出归档记录）。

修复：原树核对基线锚定输入及新增合法边，干净受管树按工具规则显式 `--refresh-baseline`，审查生成差异，保持规则和 grandfather 范围不放宽，提交后在冻结新 HEAD 再跑 strict=pass/0 violations。纠正实施回执，不把 work-owned 777 闸门留给 merge。只改基线时，重新绑定并复核依赖/配置不变即可有理由复用本轮行为证据；若修改实现或依赖则重跑受影响验收。

## 验收与实际执行

| AC → 任务 | 结果 | 方法与证据 |
|---|---|---|
| AC-01 → T-02/05 | pass | core manifest、fresh 独立 manifest 编译运行及现有 consumer 4/4；Git Bash 执行 verify-deps.sh PASS；normal/build 闭包只有 auto-val 与第三方轻库，无语言/Man/VM/renderer |
| AC-02 → T-01/02/03/04/05 | pass | 对照旧 lock.rs 完整生产臂；core 14 单测+11合同、门面 lock:: 9/9；TOML/JSON/空锁/Unicode/Windows path、版本戳、kind 过滤、真 Git HEAD、错误顺序与首同名保持 |
| AC-03 → T-03/04/05 | pass | codec/verify/sort 在 core；门面只投影/委托/Git观测。已有路径文件/缺失负控及真目录正控；独立 R01..04 补查组合/失败出口，均 pass |
| AC-04 → T-04/05/06 | partial | 旧公开锁 API 已真实委托，未迁清单准确；777 strict 基线失败见 F-795-R1-01 |
| AC-05 → T-07 | fail | 行为与 SD 审核完成；需先闭合 F-795-R1-01，无法给 final pass |
| AC-06 → T-08 | pending | merge-owned，未运行整档回归或执行沉淀/归档清理 |

命令（全部在受审树，SCCACHE_SERVER_PORT=52117）：

- `cargo test -p auto-package-lock`：14+11=25 pass，0 fail，doc 0。
- `cargo test -p auto-man --lib lock::`：9 pass，387 filtered，0 fail。输出依赖/未改动文件已有 warnings；lock.rs 和新核心未报新增 warning。不将整体构建称为零警告。
- `cargo test --manifest-path test/fixtures/modularity/package-lock-consumer/Cargo.toml`：4 pass，0 fail，doc 0。
- `C:/Program Files/Git/bin/bash.exe test/fixtures/modularity/package-lock-consumer/verify-deps.sh`：PASS。默认 Windows system32/bash 是另一环境，首次调用退出 1；使用 Git Bash 重跑通过。
- `git diff 579f6dec0 HEAD --check` 和对全部改动 Rust 文件逐一 `rustfmt --check --edition 2021`：exit=0。
- fresh 临时 consumer（下述重放）：4组全部 pass，无警告。首次探针误传 &Cow 编译失败，仅修正探针为 &str 后运行通过，无实现改动。

独立补充探针 [probe.rs](probe.rs) 的 R01：重名首条+同时版本错/缺路径/缺项按序聚合；创建目录后匹配版本复绿。R02：保留重复项，TOML/JSON/大写后缀回退，版本戳保留。R03：目标为已有目录触发 write 阶段错误，既有输出保留。R04：两格式缺字段均解析阶段拒绝，输入原字节不变。其 source 使用临时 PID 目录且不递归删除目录。

重放：新临时目录放 probe.rs 和如下 Cargo.toml，受审树运行 `cargo run --offline --manifest-path <临时目录>/Cargo.toml --target-dir target/review795-fresh`；不是复制 rlib。

```toml
[package]
name = "review795-fresh"
version = "0.0.0"
edition = "2021"
[[bin]]
name = "review795-fresh"
path = "probe.rs"
[dependencies]
auto-package-lock = { path = "D:/autostack/.wt/lang-795/auto-lang/crates/auto-package-lock" }
auto-val = { path = "D:/autostack/.wt/lang-795/auto-lang/crates/auto-val" }
[workspace]
```

## Spec delta、遗漏与健康

SD-01/02 与当前 canonical auto-man project.md 对照通过：抽取的现行行为/边界准确，没有声称整个 Man 已拆包或完整 Git/供给链校验。规范冻结副本 [spec-delta.diff](spec-delta.diff)，指纹 [fingerprints.json](fingerprints.json)。主检出旧 auto-man Spec SHA256=`f12b0b4098450025b918847e7e2dfc29b07619a16574bc341b5e62a3f9ad8ba8`；goals SHA256=`9f2b22ebf1362941e58c69166c8c58c32a36121b877b63e4509391206e4cd72e`。SD 操作与 exact paths/GOAL-003、GOAL-016 元数据有效；未发布 canonical Spec 或 ledger。

源实现未改 792/793/794 写面；旧序列化 derives 保留以维持公开 serde API，文件 load/save 算法委托 core；零 DTO 视图只用于 get/verify，save 有 DTO 转换。sort/find/verify/codec 实现没有留在 auto-man 第二份。原 LockFile 本就没有额外 CLI 消费调用；本轮证据为计划明确指定的公开生产方法 from_targets→save→load→verify，不能扩称已新增 CLI 集成。

遗漏/延后扫描：唯一合同内不当延后为 F-795-R1-01。resolver/Target/build/exporter/可执行模板保留依赖与旧弱校验是明确非目标；不要求顺手硬化。未发现新增 debug 输出或改动文件格式问题。预存跨模块 warning 未由本变更新增，不扩展本抽取范围。

## 路由

outcome=needs_fix；Plan 回 executing，重开 T-05/T-06，current_step=4/8。保留 T-01..04 已通过证据，T-07/T-08 未完成。后续 work 在原树闭合 F-795-R1-01、提交后复审。merge-owned 裸 cargo t + 两 crate/consumer 定向门、fmt、规范/ledger/索引、归档/guard清理与 795%5=0 批回归交接仍 pending。整档 t/tv/tt/tb/tu/taa/tf 均未在本轮触发。
