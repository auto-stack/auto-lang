# PLAN-832 三仓 CI / 本地受管入口

各仓均无受管 `.github/workflows`，本地受管入口即 CI 入口；未宣称远端 CI 已执行。

| owner | 入口 | 选择面 / 用途 |
|---|---|---|
| lang | `cargo t` / `tv` / `tt` / `tb` / `tf` / `taa` / `ta` / `th` / `t3` | 原语言档；merge 只跑裸 t 与实际触面，tf 仍主检出批量单实例 |
| Man | `cargo t` | workspace 全部 lib + `auto` bin（含 24 个 CLI 测试） |
| Man | `cargo tc` | runtime/client/五个管理 Core 的 lib 定向档 |
| Man | `cargo ti` | CLI/runtime/compat 集成目标 |
| UI | `cargo t` / `tu` / `ti` / `tn` | 831 既有 UI 档 |
| 三仓编排 | `python scripts/cross-repo-tiers.py run` | lang-t→lang-tu→man-t→man-ti→ui-t→ui-tu→ui-ti 串行首红即停，merge 使用 |
| 三仓选择面 | `python scripts/cross-repo-tiers.py list [--tiers …]` | 只构建列举 ID；T-08 已执行 man-t/man-ti，共 323 个目标 |
| 三仓 consumer | `python scripts/three-repository-fixture.py run --phase source-switched --require-complete` | 原件删除、单源 DAG、真实 consumer/CLI/JSON/test ID/发行收据 |
| 独立发行 | `python scripts/man-distribution.py` | clone/fresh/locked/重定位/第三方/升级/失败恢复 14 项 |
| Core | `cargo build --locked --manifest-path core/Cargo.toml -p auto --no-default-features --features core` | 独立 Core workspace/lock/target，只需 Man/lang，不读取 full/UI manifest |
| 安装 | `python scripts/man-install.py install|verify --profile core|full` | 受管 staging/交换/hash/profile/source 验证 |
| 792 | `python test/fixtures/modularity/generated-ui-consumer/driver.py` | 真交付、重定位、工具与失败控制 |
| 边界 | `python scripts/modularity-audit.py --mode strict --no-cargo-check` | 777 禁向边与真实 Man 资产计数 |
| 保全 | `python scripts/legacy-preservation.py inventory/preserve/verify/recover-drill` | 旧 Man refs/WIP/ignored 与恢复 |

Man 的 `t` 在 T-08 修正为 `--workspace --lib --bin auto`，避免 `auto` 的 24 个 bin 测试被 `--lib` 静默漏掉。所有二进制均须从对应 owner 的当前源码重建；禁止 PATH 命中旧主检出 auto/auto-ui-tool 充当证据。`man-ti` 编排先执行 UI `cargo build --locked -p auto-ui-tools` 与 lang `cargo build --locked -p auto-tool-execution --bin te-child`，再注入根和工具路径；显式工具 env 优先，缺件具名失败。Core driver 复用共享测试中的 5 个非 UI 用例，full driver 保持 7 个。
