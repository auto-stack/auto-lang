# PLAN-816 T-08 交接终稿（命令/ABI/资源/producer/cli hash）

状态：**T-08 终稿**（M816-B 全量交付；整体 R10 代码验收通过 @`6d15d25a4`，
SD 终稿 @`d7742481e`；04/07/R8-01/R9-01 全部 CLOSED，历史修复链 R1..R10 收据
保留于 attachments/816/review-* 与 repair-*）。本文为当期结论；历史 hash/收据
仅作证据来源引用，不作当前产物身份。

## 1. 命令矩阵终态（33 命令；契约基线 §2 + 门后 probe §8）

- **薄 CLI 组合（T-05）**：`auto` 基座=auto-lang-core+auto-man-runtime+
  auto-tool-client（+protocol/project-model/shell-pack/auto-val）；`ui-extra`
  （default）承载五操作外 UI 静态面；Core profile（`--no-default-features
  --features core`）闭包 normal/build/dev 零 UI 实现/零旧大库。
- **执行接线（T-06）**：`serve` 后台自身 spawn=设施 `spawn_detached`
  （CREATE_NO_WINDOW 透传，daemon 协议不动）；`service --server rust`=
  设施 owned ProcessTree（cargo run 树归属+READY 行观测+有界排空+
  cancel_within 收尸；ready 窗口默认 600s/`AUTO_SERVICE_READY_TIMEOUT_SECS`
  （秒，非法值 stderr 诊断按默认继续））；`service --server vm`=进程内 VM
  大栈线程（不变；与设施 terminate 原语互斥——VM serve_async 自持优雅关闭）；
  `req`=具名管道客户端（不动）；`watch`=notify watcher+终止原语干净退出
  （SFC 再生=ui-extra 静态面，Core 具名失败）。契约保持：service 不 kill 端口
  占用者、不假 ready、端口冲突=非零；serve≠service 两协议不混。
- **执行选项协议（T-05 R2-04）**：`GenerateUi{output,project,delivery_root,
  strict}`/`RunUi{args}`/`BuildUi{strict}` op 级字段；`Capability.options`
  广播（serde default 缺席=v1 基线）+`op_options` 单源；客户端三层守门=
  describe 广播核对（UnsupportedOption，发送前零副作用）→报告落点核实
  （OptionIgnored，堵剥选项中继）→工具 StrictGuard。
- **发现序（T-04）**：env `AUTO_UI_TOOL_BIN` → workspace target/{debug,
  release} → 邻接安装；真实 describe 核验 tool_name/protocol_version/
  required_operations；缺件/版本/能力具名拒绝（TeardownFailed 携带收尾真实
  原因），无隐式 fallback。

## 2. ABI/身份

- I-TOOL v1：`PROTOCOL_VERSION=1`、tool `auto-ui-tool 0.1.0`；五操作
  describe/validate_project/generate_ui/build_ui/run_ui + input/cancel 控制；
  选项协商为**加法**（serde default）——旧 v1 对新字段静默忽略的兼容缺口由
  客户端守门闭合（升级前完整工具+选项请求=exit1 NOT sent，adjacent 证据）。
- UiOps seam（runtime）：vue/vscode/jet/ark 臂消费**传入有效请求**；UiOps
  EXPLICIT/DEFAULT_OPS 双槽（显式先装先得，ctor/默认安装走默认槽）。
- 语言入口家族：`auto-lang-core::script_entry` 单源（15 入口）；CLI wrapper
  纯委托。

## 3. 资源/producer/发行闭包（07 CLOSED）

- 792 delivery：`gen --delivery-root <dir>`（rust only）→ bundle+
  delivery-manifest.json（producer/闭包/pin）。**DELIVERY_CRATES 表增补
  （CLI/service 闭包）属发行收口，随 R07 搬仓单独定稿——本 Plan 不改 792
  表**（815 已钉 14 crate 闭包不变；属合同内注记，非未决项）。
- 777：基线 141 边 strict pass 0v/3g（B-R1-06 清偿后各轮保持）。
- Profile：完整（default=ui-extra+python+autodown）/Core 双形态交付。
  完整安装闭包（R5 复审 CLOSED，[受审安装收据](review-overall-r5/
  install-input-reviewed.json)）：双二进制邻接布局+Cargo.lock 身份+777 基线
  hash；fresh 清洁源 locked 全量构建、首次启动邻接发现真生成、升级失败负控
  （旧 v1 工具+选项=具名拒绝零副作用）、Core 形态 UI 具名失败、SDK 闭包
  （5539 源+lock，外部项目 check/locked 实证）与默认 producer.source_root=
  安装 sdk（r4fix-delivery-two-way 双向对照）。正式独立安装包资源布局随
  R07 搬仓（合同内注记）。

## 4. 当期产物身份（T-08 终稿刷新 @`6d15d25a4`）

- worktree `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`；
  修复链 `8c69caef7`（R6）→`abc911bef`（R8）→`04bb00a46`（R9）→
  `6d15d25a4`（R10 受审）→`d7742481e`（SD 终稿）。master 祖先链以 merge
  冻结门核对为准。
- **当前冻结源重建二进制 SHA256**（locked，Unix 执行源已变——旧 hash 仅
  历史）：
  - full `auto.exe` `111ecc8558d2efa9afa3e3cc24b9f21cc5d8a24237b288363b44431bd91b5580`
  - `auto-ui-tool.exe` `810bb50c10ef9cf7fe3dc55c334501611264d8e627f12939e49609a8b9e593de`
  - Core `auto.exe`（`--no-default-features --features core`）
    `c1f51d6df3a72f43f53d085f72505fd9a516a31d2e7af3a5e24dddcbf46a4c88`
    （Core 闭包禁项 grep=0）
- R5 安装收据作为结构/发现/producer 接线的已验证据复用；本轮 Unix 执行源
  变化后其二进制 hash 不作当前身份（上表为准），布局/发现序/producer 语义
  未变。
- locked 三包构建通过（Cargo.lock 一致性）。
- service ready 预算语义：`AUTO_SERVICE_READY_TIMEOUT_SECS` 单位=秒；默认
  600s；非法值=stderr 具名诊断后按默认继续。

## 5. SD 终稿（已沉淀，@`d7742481e`）

| SD | 目标 | 终稿内容 |
|---|---|---|
| SD-01 | tool-execution/project.md | 整 Plan 状态+寿命/真停合同（含 Unix 负 PGID 组信号/interrupt 转发/spawn_detached+terminate 原语/发现序/te-child 全模式与测试分层） |
| SD-02 | auto-cli/project.md | 薄 CLI 双形态终态/基座单源/JSON 单流/服务守护命令接线 |
| SD-03 | auto-man/project.md | auto-man-runtime 单源分层/语言 adapter/git_capture/UiOps 双槽 |
| SD-04 | stdlib/design/http-service-deployment.md | rust 轨设施化（READY 观测/有界排空/合同保持） |
| SD-05 | repository-boundaries/generated-ui-distribution.md | profile/发现/SDK 闭包终态+当期身份语义 |
| 增量 | tool-protocol + auto-ui-tools | Capability.options 三层守门/op 级选项真实分发+StrictGuard |

canonical 正文即上述七文件（本 handoff 不复制规则正文）。

## 6. 已知边界（终稿；全部为合同内注记或具名观察项，无未决验收项）

- R07 搬仓项（合同内注记）：DELIVERY_CRATES 表增补、正式独立安装包资源
  布局、跨仓承接 R08/R09。
- 777 ownership 声明路径漂移 warnings（crates/auto-man/src/builder 等迁移
  遗留）——预存在，非本 Plan 引入。
- CLI/工具既有 doc-comment 与 proc-macro 未来兼容告警单列（历轮健康记录）。
- pump-race 审方探针、R1..R10 全部 replay 脚本冻结于 review-*/repair-*，
  输入变化时受影响证据须重验。
