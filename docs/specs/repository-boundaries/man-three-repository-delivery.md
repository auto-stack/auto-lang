# Man 独立仓与三仓组合发行（PLAN-832 SD-01）

> Status: active（r3；jet/ark 真实构建经用户批准转 v0.7 环境门控渠道）

## 唯一 owner 与依赖

- **auto-man**：`auto` CLI、`auto-man` compat、`auto-man-runtime`、`auto-man-core`、`auto-package-lock`、`auto-package-resolver`、`auto-toolchain-core`、`auto-man-assets`、`auto-tool-client`。
- **auto-lang**：AST/Value/语言 SDK、I-PROJECT、I-TOOL 协议、std-only owned 执行设施、auto-api-gen/auto-gen/auto-cache 等中立 adapter/基础设施。
- **auto-ui**：UI dialect/compiler/runtime/tools/shell-pack 及 UI 资产；Man 不复制 UI 生产实现。
- **auto-down**：固定只读依赖。

开发依赖按 env→组内兄弟→主检出；独立 clone/安装使用固定 Git commit、受管锁或内容寻址源包，清 AUTO_* 根、邻仓 path、parent workspace 和原 target。兼容 facet 与 Core 分开：仍重导出 UI 的 `auto-man` 不能作为无 UI Core 证据。

## 固定版本与 profile

本基线：Man `b32d3d0e`、lang 依赖修复 `9736eff9d`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。fresh lock 中 `brotli-decompressor` 以直接依赖固定为可用 `6.1.1`，不选不可用的 `6.2.0`；Tauri 测试闭包的 `erased-serde` 固定为 `0.4.0-rc.1`，避免 stable 0.4.11 不兼容。

- **Core**：`cargo build --locked --manifest-path core/Cargo.toml -p auto --no-default-features --features core`。独立 Core workspace/lock/target 复用 `crates/auto` 的 CLI 源，但不经 full manifest；只需 Man/lang，无 UI 实现源、auto-ui-tool、auto-lang compat 或 iced。语言/管理/协议命令真实执行，UI 请求具名失败。
- **完整 profile**：根 workspace default=`ui-extra+python+autodown`，含匹配 auto-ui-tool 与 UI 源/资产。五操作经 `auto-tool-client` 调真实工具进程；ui-extra 的静态面按固定 UI source 消费。
- 安装根含 auto(.exe)、完整 profile 的 auto-ui-tool(.exe)、sdk/、licenses/、install-manifest.json。manifest 按 profile 绑定 Core 两仓或 full 四仓 commit、包来源、共享 target/投影编辑和逐文件 SHA256；二进制 hash 是带 provenance 的参考产物，不替代源/清单身份。

## 发现与机器合同

- 发现序：`AUTO_UI_TOOL_BIN`（指定即必须存在）→ `AUTO_UI_ROOT`/Cargo manifest 锚/实际 `current_exe` 的 `target/debug|release` 开发布局推导 UI owner → 邻接安装；真实 describe 验工具名、协议版本、能力及收尾，无静态 fallback。普通 CLI 无 Cargo/AUTO 环境也能发现同组 owner；不搜索任意主检出或借用 Man target 旧工具。
- Blueprint 资产以语言仓为唯一生产源；显式 `AUTO_BLUEPRINTS_ROOT` 与项目库优先，Man 占位目录不遮蔽真实库。开发回退语言 owner，安装回退当前根的 `sdk/packages/blueprints`，生产/测试共用注册入口。
- I-TOOL v1 与 I-PROJECT/AST/Value/协议类型单源；选项经广播核对、落点核实和 StrictGuard 三层守门。
- `--format json`/`--ai` 单 JSON；正常退出 0，错误具名非零。service≠serve/req/mcp；VM 线程亲和、Rust owned tree、watch/reader 真停保持。
- Man 根 full workspace=9 package，根 Cargo.lock；Core 另有独立 workspace/lock/target。`cargo t`=full workspace lib+auto bin，`tc`=管理 Core，`tcore`=Core bin+非 UI driver，`ti`=集成目标。`cross-repo-tiers.py` 串行编排 lang/Man/UI，man-tcore 前在 lang 构建 te-child，man-ti 前分别在 UI/lang 构建 auto-ui-tool/te-child，并注入 owner 路径；测试 ID 逐项对拍而非仅比总数。
- **PLAN-867 SD-03：Man 四档纳入跨仓完整门**——语言仓 `cargo tf --batch`（`scripts/run-full-regression.py`）在 language/ui-full/tt/tb 后按固定序执行 man-t → man-tcore → man-tc → man-ti（同一 te-child/auto-ui-tool bootstrap 与 owner 注入；--frozen-trees 允许干净冻结三仓组）。收据绑三仓 HEAD/锁哈希/diff 与每腿选择数、失败 ID 列表；零选择或低于 `scripts/tier-selection-minimums.json.txt` 冻结地板（man-t 281 / tcore 23 / tc 252 / ti 43）即整门非零。拆分等价闭包口径见 [split-equivalence-acceptance.md](split-equivalence-acceptance.md)。

## 安装、切换与恢复

安装采用源构建→唯一 staging→闭包/来源/hash/profile 校验→受管交换。陌生非空目录拒绝；旧安装换出保留于具名 backup；失败只清本次 staging 并恢复旧根，回滚不带 backup marker 入根。手工恢复真实 backup 时，将 owned `.man-backup.marker` 移到具名保存位置后再 verify，不把它当安装 payload，也不放宽其他额外文件拒绝；恢复后旧命令须实际可运行。取消留下的 staging 仅在 marker schema 和 owner PID 死亡均确认后回收；活跃或权限不足等未知 owner 具名报告并保留，目录内容不动。

跨仓切换顺序为 prepared→target landed→consumer switched→verified，不假设跨 Git 原子事务。目标 Man 先存在且可消费，再切语言消费者并删除原件；旧 refs/恢复载荷不删，失败恢复通过新提交或受管交换，不 reset 历史。

## 验收与边界

- 独立 clone、fresh/locked、整组重定位、第三方 consumer、安装升级和恢复共 14 项通过；792 共 37 项通过（fresh/locked/重定位/工具/失败/真实运行）。
- R1 修复后 Core 独立安装 640 文件（29 packages，仅 Man/lang 来源）；full 安装 6608 文件。`tcore` 23/23（Core CLI 18+非 UI driver 5）、full CLI driver 7/7、discovery 6/6 及真实 client/te-child 正控通过；管理库 252、外部 consumer 23、tool client 30、compat 33 的未变证据保留，24/122/4 full 测试 ID 与基线逐字一致。
- Vue/Rust/vscode 真实构建、VM 真运行与六 profile 生成通过。jet/ark 仅生成已验证；其真实构建经 2026-10-25 用户正式裁定转 v0.7 环境门控渠道（“暂时不测，等v0.7再考虑”），未实测不冒称通过，工具链在场且失败仍须非零。
- 未执行远端 push、默认分支更名或 registry 发布；本地受管 CI 入口见 `docs/plans/attachments/832/ci-entrypoints.md`，不冒称远端 CI。

## PLAN-842：Full 安装投影与失败保护

Full 安装 SDK 是合法生成 producer，即使其 Cargo 清单处于安装阶段允许的中间投影，也可生成已核验的独立交付；源码、合法中间态和完整投影按 TOML 表作用域识别，异常混态拒绝。安装器在替换前验证现存 schema/profile/受管文件；陌生、未知或损坏状态拒绝，失败保留原安装与可恢复备份。Core 不引入 UI 包/工具；实际 Core 消费、Full fresh/locked、旧 0.1 安装升级/备份恢复与坏源保持按对应版本实测。

每个独立受管语言/UI SDK 的收据分别绑定所有权、来源 revision 和 staged 原始副本；路径大小写别名不得将同一 SDK 降为开发源。收据 hash 是一致性检查，不是签名。保留历史固定源与恢复载荷；兼容退役及旧版迁移见 [compatibility-retirement](compatibility-retirement.md)。来源：[PLAN-842](../../plans/archive/842-legacy-compat-and-distribution-retirement.md)
