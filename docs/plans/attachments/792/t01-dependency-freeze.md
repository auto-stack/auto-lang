# PLAN-792 T-01 依赖图冻结与失败复现收据（2026-10-16）

worktree `D:/autostack/.wt/lang-792/auto-lang` @ `ccf66f013`（基面）；
探针 scratch `D:/autostack/.wt/lang-792/scratch/`。

## 前置绑定（T-01 首项）

- PLAN-789 delivery `48dae3c48b8762dbe35809bccc45238c76e20955`（ff-only，归档 789）
- PLAN-790 delivery `249a80f2ca3a0b6f64fb21e230729b704214855b`（ff-only，归档 790）
- PLAN-791 delivery `1120ae3b4102d004dc11a3c8ebc0262a2c5be5d2`（ff-only，归档 791）

## 工具链与环境身份

- cargo 1.98.0 (797e8a9bc 2026-08-05) / rustc 1.98.0 (88d9e12ae 2026-08-18)，
  stable-x86_64-pc-windows-msvc（默认 target）。
- crates.io 源替换：aliyun sparse（用户级 `~/.cargo/config.toml`；镜像不改变
  包身份，仅下载通道——记录为环境事实）。
- `RUSTC_WRAPPER=sccache`：本机 sccache 端口被 Windows 端口排除段阻挡
  （os error 10013，Hyper-V 保留段族），探针与消费构建统一清空该变量
  （普通 cargo 缓存不受影响）。
- 无 `D:/autostack/.cargo/config.toml`；探针未设 CARGO_TARGET_DIR/
  AUTO_RUST_WORKSPACE/AUTO_LANG_ROOT 覆盖。

## 原失败复现（779/785 类 fresh-lock 探针）

探针 = 779/785 同款形态：独立空 `[workspace]` + path 依赖
`crates/auto-lang`（default = with-file-history + ui-iced），无应用锁全新解析。

- `cargo generate-lockfile`：**成功**（776 包）。fresh 锁图形栈与已提交根锁
  **逐包一致**：iced 0.14.0 / iced_widget 0.14.2 / wgpu 27.0.1 /
  wgpu-core 27.0.3 / wgpu-hal 27.0.4 / wgpu-types 27.0.1 /
  windows/windows-core 各三槽 0.57.0+0.58.0+0.62.2。
- `cargo build -p wgpu-hal`（fresh 锁下）：**通过**（34.8s）。
- **结论**：779/785 记录的 E0277 漂移（fresh 锁解析出双 windows-core
  0.57.0/0.62.2 → wgpu-hal 宏边界）为**时变生态漂移**——当时的兼容集已
  收敛。本计划不宣称"生态已根治"，以独立应用锁 + 历史漂移家族精确 pin +
  冻结重放（--locked）闭合该失败类；失败原文见 779/785 归档与
  KNOWN-DEBT F-785-P01。

## 交付闭包冻结（cargo metadata path 闭包，default+ui-iced）

本地 path 依赖闭包 = 9 crate：auto-lang、auto-val、auto-atom、auto-lex、
auto-cache、shim-metadata、auto-bindgen、auto-macros、a2r-std（跨仓
autodown-core 与 auto-test-case 为 optional 未启用，不在闭包）。外加
本地 patch `crates/patches/iced_widget`（0.14.2，P041-D1 布局竞态修复）。
规模：≈37 MiB（auto-lang 全库）+ ~1.6 MiB（其余 8 crate）+ 1.1 MiB（patch）
——**含语言源码依赖（尚未拆完），如实披露**。

## bundle 原型逐项验证（镜像仓布局裁定依据）

| # | 发现（失败原文要点） | 裁定 |
|---|---|---|
| 1 | vendored crate `bincode.workspace = true` 继承断裂（"error inheriting `bincode` from workspace root manifest's `workspace.dependencies`"） | bundle 根为显式受管 workspace，携带 producer `[workspace.dependencies]` 表逐字节副本；vendor 字节级原样 |
| 2 | 禁用 optional path 依赖仍被读 manifest（"failed to read …auto-test-case\Cargo.toml"，随后 autodown-core 跨仓路径同类） | vendored auto-lang/Cargo.toml 投影修剪：剥两 optional path 依赖 + feature 行剥 `dep:` 引用保名 |
| 3 | auto-lang build.rs `read_dir("../../crates").unwrap()` / 读 `../../Cargo.lock` | bundle 镜像生产仓布局（`crates/` 目录树 + 根 manifest/lock） |
| 4 | 编译期 include_str! 越界：`stdlib/assembly-providers.json`、`stdlib/auto/image.{at,rs.at,vm.at}`、`docs/schema/aura.at`（5 处错误，os error 3） | 五个根级编译期输入随包（缺一受管判红） |
| 5 | 投影后空 feature 消 unexpected_cfgs 警告 | `test-cases = []` / `autodown = ["ui-iced"]` 保名改写 |
| 6 | 原型最终 `cargo check -p <app>` 全闭包编译通过（55s 增量；含 iced/wgpu/windows 全栈） | 配方有效，进入生成器实现 |

## 预存红登记（非本计划引入，主检出同现）

- `cargo test -p auto-man --lib rust_ui`（裸并行）：6 红 = env 竞争/state_file
  flake/注册表污染族（nextest 进程隔离下不现；主检出同款 6 红）；
  `--test-threads=1`：2 红 = `merged_api_client_crud_fallback_for_uncovered_endpoints`
  与 `test_shell_pack_lib_freshness`（主检出基线同款 31 通过/2 红）。
- `cargo nextest run -p auto-man`（全测试目标编译）：`plan734_commands_fixture`
  tauri 2.12.4 `Wry` configured-out 编译红——主检出同款预存。
- rustfmt 1.98 对 master 的 auto-man/auto crate 产生 ~42 文件大规模重排
  （master 非当前工具链 fmt-clean；本计划不引入全仓重排，改动仅限触及文件）。
