# Man 独立仓与三仓组合发行（PLAN-832 SD-01）

> Status: active（r3；jet/ark 真实构建经用户批准转 v0.7 环境门控渠道）

## 唯一 owner 与依赖

- **auto-man**：`auto` CLI、`auto-man` compat、`auto-man-runtime`、`auto-man-core`、`auto-package-lock`、`auto-package-resolver`、`auto-toolchain-core`、`auto-man-assets`、`auto-tool-client`。
- **auto-lang**：AST/Value/语言 SDK、I-PROJECT、I-TOOL 协议、std-only owned 执行设施、auto-api-gen/auto-gen/auto-cache 等中立 adapter/基础设施。
- **auto-ui**：UI dialect/compiler/runtime/tools/shell-pack 及 UI 资产；Man 不复制 UI 生产实现。
- **auto-down**：固定只读依赖。

开发依赖按 env→组内兄弟→主检出；独立 clone/安装使用固定 Git commit、受管锁或内容寻址源包，清 AUTO_* 根、邻仓 path、parent workspace 和原 target。兼容 facet 与 Core 分开：仍重导出 UI 的 `auto-man` 不能作为无 UI Core 证据。

## 固定版本与 profile

本基线：Man `356f2c8186d07bc18567b18179c3cad7bda2d483`、lang 切换 `ba1ce5901d76643a8878f5d0f4663c3ca4904ea5`、UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。

- **Core**：`--no-default-features --features core`。无 UI 实现源、auto-ui-tool、auto-lang compat 或 iced；语言/管理/协议命令真实执行，UI 请求具名失败。
- **完整 profile**：default=`ui-extra+python+autodown`，含匹配 auto-ui-tool 与 UI 源/资产。五操作经 `auto-tool-client` 调真实工具进程；ui-extra 的静态面按固定 UI source 消费。
- 安装根含 auto(.exe)、完整 profile 的 auto-ui-tool(.exe)、sdk/、licenses/、install-manifest.json。manifest 绑定四仓 commit、包来源、投影编辑和逐文件 SHA256；二进制 hash 是带 provenance 的参考产物，不替代源/清单身份。

## 发现与机器合同

- 发现序：`AUTO_UI_TOOL_BIN`（指定即必须存在）→ workspace target/{debug,release} → 邻接安装；真实 describe 验工具名、协议版本、能力及收尾，无静态 fallback。
- I-TOOL v1 与 I-PROJECT/AST/Value/协议类型单源；选项经广播核对、落点核实和 StrictGuard 三层守门。
- `--format json`/`--ai` 单 JSON；正常退出 0，错误具名非零。service≠serve/req/mcp；VM 线程亲和、Rust owned tree、watch/reader 真停保持。
- Man 根 workspace=9 package，独立 Cargo.lock；`cargo t`=workspace lib+auto bin，`tc`=管理 Core，`ti`=集成目标。`cross-repo-tiers.py` 串行编排 lang/Man/UI，测试 ID 逐项对拍而非仅比总数。

## 安装、切换与恢复

安装采用源构建→唯一 staging→闭包/来源/hash/profile 校验→受管交换。陌生非空目录拒绝；旧安装换出保留于具名 backup；失败只清本次 staging 并恢复旧根，回滚不带 backup marker 入根。取消留下的 staging 仅在 marker schema 和 owner PID 死亡均确认后回收，活跃或未知目录不动。

跨仓切换顺序为 prepared→target landed→consumer switched→verified，不假设跨 Git 原子事务。目标 Man 先存在且可消费，再切语言消费者并删除原件；旧 refs/恢复载荷不删，失败恢复通过新提交或受管交换，不 reset 历史。

## 验收与边界

- 独立 clone、fresh/locked、整组重定位、第三方 consumer、安装升级和恢复共 14 项通过；792 共 37 项通过（fresh/locked/重定位/工具/失败/真实运行）。
- Core 安装 643 文件；full 安装 6608 文件。管理库 252、Core CLI 18、外部 consumer 23、tool client 30、CLI driver 7、compat 33 通过；24/122/4 测试 ID 与基线逐字一致。
- Vue/Rust/vscode 真实构建、VM 真运行与六 profile 生成通过。jet/ark 仅生成已验证；其真实构建经 2026-10-25 用户正式裁定转 v0.7 环境门控渠道（“暂时不测，等v0.7再考虑”），未实测不冒称通过，工具链在场且失败仍须非零。
- 未执行远端 push、默认分支更名或 registry 发布；本地受管 CI 入口见 `docs/plans/attachments/832/ci-entrypoints.md`，不冒称远端 CI。
