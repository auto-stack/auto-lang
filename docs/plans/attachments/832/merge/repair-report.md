# PLAN-832 merge 门禁修复：blueprint 根遮蔽

Man `75a81b4f4b100f7f8bdf9c02bbf87c299cc9e5af`；前基线 `f52cd93b`。R3 的 F01..F04 不变；本报告只覆盖 merge 新发现。

## Finding 与原因

`cargo t` 的 `cmd_bp::tests::bp_list_at_golden_stable` 与 `check_passes_on_known_good_login_reference` 失败。Man 的 `packages/blueprints` 只含 `pac.at`，`BlueprintRegistry::with_defaults()` 的 CWD 搜索在此停止，语言仓真实库不再被回退命中。原测试也直接调用 `with_defaults()`，绕过生产 `registry()`，所以修复必须统一两者入口。

## 修复

`cmd_bp::registry()` 保持 `AUTO_BLUEPRINTS_ROOT` 和项目库优先；仅当归一化后的根等于 Man 占位库时，改走当前可执行文件对应的 `sdk/packages/blueprints`（安装）或语言 owner（开发）。UI 固定版本未改，未复制第二份 blueprint 生产源。两条测试改为消费同一 `registry()`。

## 定向证据

- `cargo nextest run -p auto --bin auto cmd_bp::tests`：4/4 通过；`cargo build --locked -p auto` 通过，见 [日志](repair-cmd-bp.log.txt)。
- 开发 `target/debug/auto.exe bp list`：包含 `form/login`、`layout/gallery-shell`，见 [输出](dev-bp-list.log.txt)。
- 更新 full 安装后，清 AUTO/Cargo 环境的安装 `bp list` 同样通过，见 [安装输出](installed-bp-list.log.txt)。
- 显式缺失 `AUTO_BLUEPRINTS_ROOT` 仍权威，不回落语言库，见 [负控](explicit-blueprints-root.log.txt)。
- `rustfmt --check`、`git diff --check` 通过；无临时诊断输出。

未在本 repair 重跑整档。后续受影响独立复审通过后，merge 必须重跑 Man `cargo t` 和其余冻结门。lang 的 B845-FEATURE-01 是已登记 PLAN-847 blocker，与本 finding 分开，不由 832 越权修复。
