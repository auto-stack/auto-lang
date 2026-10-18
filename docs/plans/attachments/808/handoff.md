# PLAN-808 T-07 work handoff（终态送审收据）

2026-10-18。本文件记录 work 阶段的净迁移、owner 交接、证据入口与
merge-owned pending 门。代码/测试均在本计划唯一实施树：

- worktree: `D:/autostack/.wt/lang-808/auto-lang`
- branch: `codex/plan-808-dev`
- base: `2046a025b9a445d35a02da2c72a1613259a0cb60`（master）
- auto-down sibling: `D:/autostack/.wt/lang-808/auto-down` @ `c39ac7f`
  （detached，只读消费，不改源码）
- real DLL: `D:/autostack/auto-term/target/debug/autoterm_core.dll`
  SHA256 `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`
  （auto-term `main` @ `7a0d0e8b718fb260641f0d09ada95a7346771cd3`，clean）
- code commit: `3c02068ca`（实现+测试）；`c346f1077`（777 基线重绑）
- root `Cargo.lock`（gitignored）SHA256：
  `9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`
  （新 member `auto-terminal-core` 已解析；仅 libloading 既有锁条目）

## 1. 净迁移（文件/行数）

| 面 | before | after | 说明 |
|---|---|---|---|
| `vm/ffi/term_engine.rs` | 1394 行 | 355 行 | 仅 34 shim + 2 处窗口门面委托；DLL/UI/采样业务清零 |
| `vm/ffi/term_engine.rs` 的 `crate::ui::` | 49 | 2 | 两处均为 `ui::terminal_provider` 门面委托 |
| 新 `crates/auto-terminal-core` | 0 | 1182 行（4 文件） | types/engine/provider/lib；闭包仅 libloading |
| 新 `ui/terminal_provider.rs` | 0 | 540 行 | UI view/input/palette/window adapter |
| `ui/terminal/mod.rs` | 1993 行 | 1951 行 | `TermColor/TermCell/几何/palette slots` 改 core 单份 re-export |
| 新 `tests/plan808_terminal_domain.rs` | 0 | 1168 行 | V-01..V-06 矩阵 |
| 独立 fixture `terminal-core-consumer` | 0 | Cargo + ~120 行 + verify-deps | 无 auto-lang/UI/man/auto-term 闭包 |
| 777 `core_to_ui` 文本候选 | 905 | 860 | observe 收据 `t07-777-observe.json`（-45） |

其他接线：`vm/host_capabilities.rs`（terminal 进程默认四级 API）、
`vm/native.rs`（NI 槽/Clone/new/accessor）、`vm/engine.rs`（构造快照/实例
set-get）、`lib.rs`（两处 if-absent 安装）、`ui/mod.rs`、`tests.rs`、
根/`auto-lang` Cargo（member + path dep）、`.autoos/module-boundaries.toml`
（`auto-terminal-core = runtime`）、`auto-man/rust_ui.rs`
（`DELIVERY_CRATES` 12→13，792 受影响闭包）。

## 2. 34 项 native 映射

ID/canonical/弹序/返回类型**全部不变**；逐项旧臂与新 provider 方法见
[contract-baseline.md](contract-baseline.md) §1。计数核对：catalog 34 条 =
shim 34 个 = provider 34 方法；plan808 `catalog_and_shims_match_34` 永久
钉扎。

## 3. 剩余 edge owner 表（不虚报整域完成）

| 残边 | 实际路径 | owner / 退出 |
|---|---|---|
| media asset/image_pipeline | `vm/ffi/stdlib.rs`（30 处） | R03 媒体完整子域（后续独立合同） |
| i18n | `vm/ffi/stdlib.rs::shim_i18n_t` → `ui::i18n_lookup` | R03 i18n 后续合同 |
| diff 9915..9917 | `vm/native.rs`/`ui_gen` | PLAN-703/807 |
| a2r direct frame arms | `a2r_std.rs:1231/1237` | ui/overview 既有未迁边界 |
| terminal UI 注册表/输入/选中/菜单/滚动投影 | `ui/terminal/mod.rs`、`iced/widget.rs`、`ui/dynamic.rs:2587` | UI 视图/输入 owner（本计划只改消费路径） |
| a2c/a2r 原生 engine C-ABI 语料 | `test/a2c/18_c_interop/004_engine_face`、`test/a2r/27_c_abi/*` | 不在 34 native 支持面；不改 |
| `kill`/`set_palette`/`shell_pid` | DLL 导出但无 auto-lang native | 原支持面之外；不新增绑定 |

## 4. 验证证据（work-owned，全部在本树当前代码）

| 门 | 命令 | 结果 |
|---|---|---|
| core check | `cargo check -p auto-terminal-core` | exit 0 |
| core tests | `cargo test -p auto-terminal-core` | 4/4 pass |
| core+lang check | `cargo check -p auto-terminal-core -p auto-lang` | exit 0 |
| Ui terminal 定向 | `cargo test -p auto-lang --lib ui::terminal -- --test-threads=1` | 41/41 pass |
| plan808（真实 DLL） | `AUTOTERM_ENGINE_DLL=... cargo test -p auto-lang --lib plan808 -- --test-threads=1` | 10/10 pass |
| 独立 consumer | `cargo test --manifest-path test/fixtures/modularity/terminal-core-consumer/Cargo.toml`（DLL env） | 4/4 pass |
| consumer 闭包 | `bash test/fixtures/modularity/terminal-core-consumer/verify-deps.sh` | PASS（仅 libloading/windows-link） |
| 真实 DLL 有界探针 | `python docs/plans/attachments/808/t01-dll-probe.py` | 34 checks pass |
| 777 observe | `python scripts/modularity-audit.py --mode observe --no-cargo-check` | 收据 `t07-777-observe.json`（core_to_ui 860） |
| 792 delivery driver | `python test/fixtures/modularity/generated-ui-consumer/driver.py` | **25/25 pass**（fresh 141s + relocated locked 120s；收据 `t07-792-driver-receipt.json` / 日志 `t07-792-driver.log.txt`） |
| fmt | `rustfmt --check` 触面文件 | 0 diff |
| 生成 Rust terminal 金样 | `cargo test -p auto-lang --lib terminal_`（3 金样名） | 3/3 pass（该过滤面 25/26；唯一红 `plan705_client_total_deadline_terminal_error` 为并发负载下时序 flake，solo 复跑 1/1 绿、不属 terminal 面） |

真实 DLL 测试在缺 `AUTOTERM_ENGINE_DLL` 时打印显式 `PENDING(plan808)`，
不计为通过；work 证据以 env 开启运行的结果为准。

## 5. 计划/规范交接

- SD 草案：[spec-delta.md](spec-delta.md)（SD-01 add auto-terminal-core/
  project.md；SD-02 vm overview；SD-03 ui overview；SD-04 terminal-mux-model）。
- 777 基线：本树 manifest/config 已变，已在干净提交 `3c02068ca` 上
  `--refresh-baseline` 重绑（`c346f1077`）并 strict pass（0 violations /
  2 grandfathered）；merge 在最终 master 组合上再重绑。
- 792：`DELIVERY_CRATES` 已加 `auto-terminal-core`（13 项，生成
  crates=14）；driver 在最终代码上 **25/25**（fresh/relocated locked/
  负控/dev 路由）。merge 合流其它新成员/闭包变化时按 792 driver 重放
  受影响 fresh/locked。
- ledger/索引/归档：未改（merge-owned）。

## 6. merge-owned pending 门（不得在本阶段宣称交付）

- 冻结提交上裸 `cargo t` + `cargo tv`；实际触面含 `ui_gen`? 本计划未改
  `ui_gen` 发射算法（仅消费侧不变），按计划不机械触发 `cargo tu`；若
  merge diff 实际触及 `ui_gen/**` 再补。
- 777 strict 在最终组合树上 `--refresh-baseline` 重绑后 pass。
- 792 driver 在最终组合树上重放受影响 fresh/locked（若闭包变化）。
- Spec/ledger/index/归档 + wt-guard clean 后清理。

## 7. 未决/风险

- T-01 已冻结的旧臂残留：free 后 CURSORS/VIEWPORTS 保留旧值（旧可观察
  值；单调句柄不复用）。plan808 real-DLL 测试断言 free 后 rows 空/
  is_exited=true，不依赖 cursor/viewport 清零。
- 真实 DLL/PTY 生命周期依赖机器环境；无 `AUTOTERM_ENGINE_DLL` 的运行只
  产生 PENDING 证据，需在 review/merge 环境复跑同一命令。
- a2r 原生 engine C-ABI 测试（`a2c`/`a2r` 语料）不属 34 native 面，未被
  本计划修改；若合并窗口其它计划触碰 engine ABI，需复核。
- 媒体/i18n 等 R03 残边未迁；本计划完成不等于 R03 整包完成。
