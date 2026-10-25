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

## F832-MERGE-BP-01：安装来源反例与关闭

独立 reviewer 指出，上一版的通用 UI 回退可在安装态先命中开发语言库，使 SDK 分支失效；相同包清单不能证明来源。最终 Man `ceb8dfdd469b2dec1f2490aae1126b0b3303e35e` 将优先级改为显式 env/真实项目库→安装自身 SDK→开发语言 owner。

永久 `installed_blueprints` driver 在安装 SDK 的 `form/login/spec.md` 注入 `PLAN832_INSTALLED_SDK_ONLY`，要求真实 `bp show` 输出该 marker，随后恢复原字节；[结果](installed-sdk-marker.log.txt) PASS。`cmd_bp` 4/4 与 full build 保持通过，见 [定向日志](repair-bp-install.log.txt)。该检查区分真实 SDK 与开发源，不再以相同包清单冒充安装来源。

## fresh-lock 依赖反例与修复

完整发行的 fresh lock 曾选择 `brotli-decompressor 6.2.0`，随后 locked build 因该版本不可用失败。Man `b32d3d0eeed8e08dbe74302128fe3b815dc3fd6a` 以直接依赖精确 pin 可用 `6.1.1`，未复用旧锁或降低 fresh 验收。新 clone 的 `Cargo.lock` 已确认选择 6.1.1；`cargo check --locked -p auto-man` 通过。完整 14 项发行门在该提交上重新执行。

792 的工具包 fresh build 另发现 `tauri 2.12.5` 与 stable `erased-serde 0.4.11` 不兼容。lang `9736eff9d5f9c9821880f4d67a6dbd3cb7d1c759` 在 `auto-api-gen` 的 dev 闭包精确 pin `erased-serde=0.4.0-rc.1`；`cargo check --locked -p auto-api-gen` 与 Man check 均通过。792 以该语言提交重新执行，未跳过任何 V01..V05 用例。
