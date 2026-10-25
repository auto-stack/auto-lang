# PLAN-832 R1 findings 修复闭环

2026-10-25。范围仅为 F832-R1-01..04；未运行语言/Man 整档，未合入。jet/ark 的 r3 延期保持不变。

## 冻结身份与证据边界

- Man 最终实现/文档：`ebfc88f`；前一提交 `f601673` 含 Core 构建/安装实现。
- lang 最终 CI/Spec：`0fd092bdc`；UI=`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 无 UI fresh 验证在 Man `f601673`、lang `cd603e418` 的真实 clone 上执行。`f601673..ebfc88f` 只增加发行 driver 重定位、`tcore` 配置和文档，不改变 Core 构建/安装实现；lang 后续提交只改 CI/Spec/证据，不改变语言源码或资源。
- Core 重定位在最终 driver 上执行并提交为 `ebfc88f`；full 安装/安装态生成使用 `f601673` 实现，后续提交未改该路径。

## Findings closure

| Finding |  violated invariant  | Fix and permanent cases | Current result / remaining gap |
|---|---|---|---|
| F832-R1-01 | Core 无 UI/down 源码、无旧 target/邻仓回退仍可构建、安装和执行 | 新增 `core/Cargo.toml`+`core/Cargo.lock` 与 `crates/auto-core`，以 Cargo target path 复用同一 CLI 源；Core metadata/build/install 全部走独立 workspace。安装只采集 Man/lang HEAD/LICENSE，显式登记 shared targets。`man-distribution.py` 的 SRC/DST-2 固定无 UI 两仓 clone，DST-6 删除 source、core 和 SDK target 后重定位 | [fresh 3/3](core-independent-receipt.json.txt)：fresh lock、locked build、normal/build/dev 闭包、安装、安装 SDK locked check、真实脚本及 UI 具名拒绝全过；[重定位](core-relocate.log.txt) PASS。Core=640 files/29 packages/两来源，[manifest](core-install-manifest.json.txt)。full=6608 files verify=0，安装态 gen 通过。无开放 finding；完整 14 项与统一档留 merge |
| F832-R1-02 | 只有确认死亡 owner 才回收 staging；活跃/权限未知保留并报告 | Windows 以 `WinDLL(use_last_error=True)` 区分 ERROR_INVALID_PARAMETER(死亡)与其他错误(未知)；`recover_stale_staging` 只删 `False`，`None` 具名保留。`owner_liveness` 永久纳入 FLR-6 | [实际反例](owner-liveness.log.txt)：PID 4 报 `owner liveness unknown` 且 staging 保留；self 活跃保留、PID 2147483647 死亡回收、注入 unknown 保留并报告均 PASS。无开放 gap |
| F832-R1-03 | CI/生产 discovery 按 owner 解析真实工具，显式 env 优先，缺件具名红 | UI discovery 改为 `AUTO_UI_ROOT`/组内 UI owner，安装态仍邻接；CLI driver 复用同一 resolver，te-child 改 lang owner；独立 tool-client fixture 同步。跨仓编排注入 resolved roots/tools，`man-ti` 构建 UI+lang helper，`man-tcore` 构建 lang helper | [原清环境反例 1/1](ci-gen-no-overrides.log.txt)、[full driver 7/7](full-cli-driver.log.txt)、[`tcore` 23/23](man-tcore.log.txt)、[discovery 6/6+client 1/1+te-child 1/1](tool-client-discovery.log.txt)、[bootstrap 分支](ci-bootstrap-final.log.txt)、[full 安装态 gen](full-installed-gen.log.txt) 全过。完整 `man-ti` 未在本 repair 重跑，归 merge |
| F832-R1-04 | 目标仓当前安装/发行/恢复入口必须真实可执行 | Man 中英文首页和 CLAUDE 改为 `auto`、独立 Core/full 构建与受管安装；新增 `docs/installation.md`；旧 am 文档移到 `docs/legacy-am*.md`；`release.ps1` 显式退役 | Core/full 文档入口即上述真实安装命令；fixed sources、版本 manifest、升级 backup、恢复、重定位均已写明。`release.ps1` 实测 exit=2 且具名 retired，见 [hygiene](hygiene.log.txt)。无开放 gap；远端发布不在本轮 |

## 定向验证清单

- `cargo check --locked --manifest-path core/Cargo.toml -p auto --no-default-features --features core`：PASS，[日志](core-check.log.txt)。
- 真实 clone：`SRC-1`、`DST-1`、`DST-2` 3/3 PASS，[日志](core-independent.log.txt)；Core 重定位 1/1 PASS。
- `python scripts/man-install.py install|verify --profile full`：6608 files PASS，[日志](full-install.log.txt)。
- 清 `AUTO_*`、te-child/target override 后原始 `gen_runs_through_real_tool_process`：1/1；同一 full `plan816_cli_driver`：7/7。
- `cargo tcore`：23/23（18 bin+5 非 UI driver）；未运行 Man `t`/`ti` 整档。
- tool-client `discovery_`：6/6；真实 describe：1/1；blocked writer/te-child：1/1。
- owner staging：实际 AccessDenied+活跃+死亡+注入 unknown 全过。
- `rustfmt --check`、生产/编排 Python AST、`git diff --check`、Spec 索引重建（61 projects）通过；退役 release 负控 exit=2。

## 未执行与下一责任角色

未运行语言裸 `cargo t`、触面档、Man `t`/`ti` 全集、完整三仓队列或完整 14 项发行矩阵；未以定向结果替代这些 merge-owned 门。下一责任角色为 T-10 受影响独立复审；final pass 后才由 merge 在新冻结提交执行统一门、合入、Spec/ledger、归档与 guard 清理。
