# PLAN-816 T-08 交接（命令/ABI/资源/producer/cli hash）

状态：work T-08 交付物（M816-B 主体交接；SD 增量候选随附，canonical 沉淀归
T-10/T-11 merge owner 受管执行）。证据基线：分支 `codex/plan-816-dev` @T-08
提交（T-06 `7ce08b5cf`、T-07 其后；R2 修复 `e73ec6b06`）。

## 1. 命令矩阵终态（33 命令；契约基线 §2 + 门后 probe §8）

- **薄 CLI 组合（T-05）**：`auto` 基座=auto-lang-core+auto-man-runtime+
  auto-tool-client（+protocol/project-model/shell-pack/auto-val）；`ui-extra`
  （default）承载五操作外 UI 静态面；Core profile（`--no-default-features
  --features core`）闭包 normal/build/dev 零 UI 实现/零旧大库。
- **执行接线（T-06）**：`serve` 后台自身 spawn=设施 `spawn_detached`
  （CREATE_NO_WINDOW 透传，daemon 协议不动）；`service --server rust`=
  设施 owned ProcessTree（cargo run 树归属+READY 行观测+有界排空+
  cancel_within 收尸；ready 窗口默认 600s/`AUTO_SERVICE_READY_TIMEOUT_SECS`）；
  `service --server vm`=进程内 VM 大栈线程（不变；与设施 terminate 原语互斥
  ——VM serve_async 自持优雅关闭）；`req`=具名管道客户端（不动）；
  `watch`=notify watcher+终止原语干净退出（SFC 再生=ui-extra 静态面，Core
  具名失败）。契约保持：service 不 kill 端口占用者、不假 ready、端口冲突=
  非零；serve≠service 两协议不混。
- **执行选项协议（T-05 R2-04）**：`GenerateUi{output,project,delivery_root,
  strict}`/`RunUi{args}`/`BuildUi{strict}` op 级字段；`Capability.options`
  广播（serde default 缺席=v1 基线）+`op_options` 单源；客户端三层守门=
  describe 广播核对（UnsupportedOption，发送前零副作用）→报告落点核实
  （OptionIgnored，堵剥选项中继）→工具 StrictGuard。
- **发现序（T-04）**：env `AUTO_UI_TOOL_BIN` → workspace target/{debug,
  release} → 邻接安装；真实 describe 核验 tool_name/protocol_version/
  required_operations；缺件/版本/能力具名拒绝，无隐式 fallback。

## 2. ABI/身份

- I-TOOL v1：`PROTOCOL_VERSION=1`、tool `auto-ui-tool 0.1.0`；五操作
  describe/validate_project/generate_ui/build_ui/run_ui + input/cancel 控制；
  选项协商为**加法**（serde default）——旧 v1 对新字段静默忽略的兼容缺口由
  客户端守门闭合（升级前完整工具+选项请求=exit1 NOT sent，adjacent 证据）。
- UiOps seam（runtime）：vue/vscode/jet/ark 臂消费**传入有效请求**（T-05
  R1 F-04；三旧注入 consumer 已随 R2-05 同步新签名，三向注入断言保持）。
- 语言入口家族：`auto-lang-core::script_entry` 单源（15 入口）；CLI wrapper
  纯委托。

## 3. 资源/producer/发行闭包

- 792 delivery：`gen --delivery-root <dir>`（rust only）→ bundle+
  delivery-manifest.json（producer/闭包/pin；fixture
  `generate_report_binds_delivery_root` 钉报告=delivery 根+真实计数）。
  **DELIVERY_CRATES 表增补（CLI/service 闭包）属发行收口，随 R07 搬仓单独
  决策——本 Plan 不改 792 表**（815 已钉 14 crate 闭包不变）。
- 777：基线重采样 @7ce08b5cf（141 边 strict pass 0v/3g；auto→
  auto-tool-execution 等 7 新边入基线）。
- Profile：完整（default=ui-extra+python+autodown）/Core 双形态交付；
  完整 profile 交付=CLI 二进制+auto-ui-tool+资源/锁（815 形态）；Core 无
  UI 安装可消费（tool-client/tool-execution 两 fixture verify-deps 钉）。

## 4. cli hash/来源读回

- worktree `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`；
  T-06 提交 `7ce08b5cf`、T-07 其后（fixture/driver/777）；R2 修复
  `e73ec6b06`（T-13 R3 受影响复审对象）。master 祖先链以 merge 冻结门
  （T-10）核对为准。
- 关键二进制来源：`target/debug/auto.exe`（薄 CLI）、`target/debug/
  auto-ui-tool.exe`（工具）——均本树重建；证据文件见 attachments/816/
  review-t05-r2-fixround/（R2 修复重放，不覆写 R2 冻结收据）与
  t06-smoke-results.json（T-06 真链冒烟）。

## 5. SD 增量候选（T-10/T-11 受管沉淀；R2 复审指派项已含）

| SD | 目标 | 增量要点 |
|---|---|---|
| SD-01（B 余量） | tool-execution/project.md | spawn_detached/terminate 原语、service rust 轨树归属+READY 观测+有界排空、ready 窗口预算（600s/env） |
| SD-02 | auto-cli/project.md | 薄 CLI 组合/双 profile/命令矩阵终态/watch 终止/argv 保持 |
| SD-03 | auto-man/project.md | runtime 分层+UiOps 请求单源（R2-05 签名）+serve_rust 设施化 |
| SD-04 | stdlib/design/http-service-deployment.md | rust 轨父进程=READY 行观测+取消语义（现"退出码+状态探针"边界的受控增强）|
| SD-05（B 余量） | repository-boundaries/generated-ui-distribution.md | 双 profile 发行闭包/工具选项协商消费方式 |
| （R2 指派） | tool-protocol/project.md + auto-ui-tools/project.md | Capability.options 广播/op_options 单源/选项协商与落点核实（UnsupportedOption/OptionIgnored）/真实产物报告规则（output 锚定实际落点、jet/ark full_project 覆盖） |

## 6. 已知边界/未竟（诚实清单）

- T-07 余量：完整/Core profile 独立 fresh/locked/重定位**全矩阵**负控未
  单独成档（relocatable_delivery/fresh43/locked 已有 R1 证据面；Core 独立
  构建闭包=T-05 cargo tree 证据）——归最终 review 判定是否补独立档。
- `service --server rust` 真链端到端（生成+cargo run+READY+排空）未在本
  轮实机跑（重链；设施原语+READY 解析+vm 轨真链已分别钉）——V-04 完整
  driver 归最终集成（T-09 复审判据之一）。
- pump-race 审方探针装配需按 R2 修复后的可失败 reclaim 签名机械适配
  （R3 复审注记；语义已由永久回归钉）。
- 777 ownership 声明路径漂移 warnings（crates/auto-man/src/builder 等
  迁移遗留）——预存在，非本 Plan 引入。
