# PLAN-831 T-01 契约基线（G831 后实际源审计）

冻结源版本：auto-lang master `a61209cc0`（U815 `33c9451fb` + U816 `ac7c1369c` 合入后实际源）。
启动门证据见本 Plan §9 2026-10-24 记录。本文按 §5.1 要求冻结逐包/输入/测试/消费者
owner 与 DAG、profile、legacy 状态、兼容与共享声明切口可行性；未闭决策具名交后继任务。

## 1. 逐包迁移映射（DAG：迁移后 UI 仓 → 语言基础固定 rev）

### 1.1 迁入 UI 仓（T-04 候选源=语言仓 crates/，路径同实编）

| 包 | 现依赖（normal） | 迁后依赖（UI 仓内/固定 rev 语言基础） | 备注 |
|---|---|---|---|
| auto-ui-dialect | auto-parser-api（+dev auto-lang-core） | auto-parser-api（语言基础固定 rev） | 86 项文法回归随迁（PLAN-814 R3-03b） |
| auto-ui-compiler | auto-lang-core、auto-ui-dialect、auto-atom、auto-val | 同左改固定 rev/仓内 | UIC 指纹/生成器 canonical import 随源根更新（§2） |
| auto-ui-runtime | auto-lang-core、auto-ui-compiler、auto-val、auto-media-core、auto-text-core、auto-terminal-core、autodown-core(跨仓 optional) | 同左；跨仓 path 见 §4 | ui_tests/（musk_vm_track/p614/p618/plan019/042/051* 等）与 ui-iced 特性随迁 |
| auto-ui-tools | auto-tool-protocol、auto-project-model(wire)、auto-ui-compiler、auto-lang-core、auto-ui-runtime、auto-tool-execution、**auto-man-core** | 前六同左（语言基础/中立单源）；**auto-man-core 边去除**（C2，§3） | tests/{fixtures,legacy_build_semantics,lifecycle,resources_manifest,tool_process} 随迁 |
| shell-pack | auto-ui-runtime(ui-iced)、auto-lang-core | 仓内+固定 rev | 公开再生器 canonical import 规则（815 F-T09-01）随源根 |

### 1.2 留语言仓（中立/语言基础，不迁）

auto-lang-core（SDK）、auto-parser-api、auto-atom/auto-val（data）、auto-media-core、
auto-text-core、auto-terminal-core、auto-project-model、auto-tool-protocol、
auto-tool-execution、auto-tool-client（777 man-ecosystem）、auto-man-runtime/auto-man
（Man 域；816 单源分层）、**旧 auto-lang 兼容 crate**（lang-monolith——上层 UI 组合候选，
保 package 名/公开类型/feature，不入纯 SDK 闭包，终态拆分 R4/R8 在案）。

### 1.3 adapter 去 Man 边（C2）

| 包 | 处理 |
|---|---|
| auto-api-gen | 语言服务 adapter 归属裁定（SD-07 新 spec）；sidecar 声明切 project-model 单源（§3） |
| auto-man-core | SidecarDecl/pac_from_decl 声明规则迁 project-model 轻规则；Man 侧旧 API 再导出（T-05） |

## 2. 兼容面（C4，T-05/T-06 冻结细节）

旧 `auto-lang` 公开 UI 面（lang-monolith）：Parser/VM 构造、pub 字段/回调、default/显式
Core/UI feature——保 package 名与类型身份；迁移后经 UI 仓 re-export 或薄转发（T-06 切源时
定机制），旧消费编译/结果一致（V-02）。UiOps seam/EXPLICIT 双槽（816）消费方不变。

## 3. C2 去 Man 边切点（实测定位 @a61209cc0）

- `crates/auto-api-gen/src/sidecar.rs:54-60`：`auto_man_core::sidecar_decl_from_root` /
  `SidecarDecl` → `from_decl`——声明解析单源迁 project-model（sidecar 根规则）。
- `crates/auto-ui-tools/src/rust_ui.rs:3367`：经 `auto_api_gen::sidecar::from_decl`（随上切）。
- `crates/auto-ui-tools/src/rust_ui.rs:5931,5951`：`auto_man_core::pac_from_decl`
  （theme/external_backend/theme_decl）→ project-model 轻规则单源。
- 关闭条件：代表缺省/有效/动态求值/主题覆盖/sidecar 根规则语义保持（V-02 正反控）；
  UI 仓 normal/build/dev 闭包零 auto-man-core/auto-man（V-01 tree 禁边）。

## 4. 跨仓依赖（§5.3 合同）

`auto-ui-runtime` 的 `autodown-core = { path = "../../../auto-down/autodown/packages/engine/rust", optional = true }`
——迁后不能留指向语言仓兄弟的不存在路径：固定依赖 rev 或显式 profile 处理（T-06 冻结
具体机制：git 固定 rev / registry / vendored，须 fresh/locked 可复现）；auto-down 兄弟
worktree 依既有解析补 detached，不修改其实现。

## 5. 测试/fixture/消费者/门面（T-03/T-06/T-09 逐项冻结接口）

- **随迁 fixture**（源根/锁更新）：ui-dialect/ui-grammar/ui-package/ui-runtime/
  ui-tool-consumer、generated-ui-consumer（792 driver 消费）、language-sdk-consumer
  （反向控制：SDK 无 UI checkout 消费继续成立）。
- **留语言仓**：tool-client/tool-execution/project-model/其余非 UI consumer。
- **新增**（T-03）：`test/fixtures/modularity/ui-repository-consumer/` + 三仓 fixture 入口
  （独立 clone 定向 cargo check/test --manifest-path，不偷读主检出）。
- **测试档**：tu（ui_gen 四生成器）随编译器迁移后源根更新；画廊围栏/1M churn 机器级闸门
  位置不变；examples/ui（381 文件）与 auto-examples/gallery 消费归属 T-04 清单。
- **777**：域表 ui 域四包迁出后语言仓基线重采样（strict 保持 0v）；UI 仓侧边界规则按
  T-06 定稿（不扩大支持）。
- **792**：受管发布事务/driver 复用，producer/资源闭包随 UI 仓固定版本（SD-02/04）。

## 6. 旧仓（D:/autostack/auto-ui）现状（T-02 重采保全）

- HEAD `f7d92ee`（AURA pipeline Plan 096 时代），工作树有 WIP（`M Cargo.lock`）；
  workspace `crates/* + examples/*`：auto-ui、auto-ui-gpui、auto-ui-iced、
  auto-ui-transpiler(-cli)、gpui-examples、iced-examples。
- **不承接**旧 GPUI/双生产实现（§2）；legacy 保历史分支（784 保全工具：legacy-
  preservation.py inventory/preserve/verify/recover-drill，T-02 执行并新增 preservation.md）。
- 旧"master已清空"指令已被 784 证过时；本轮未重采 refs/WIP——T-02 以真实快照为准。

## 7. profile/发行面（T-07/T-08 实测）

UI 安装 profile=编译库+runtime+tool 二进制+运行资产；生成应用 canonical 双直依赖
（auto-ui-runtime{ui-iced}+auto-lang-core，M814-B）与 Vue/VM-Iced/Rust 后端保持。
独立 clone fresh/locked/重定位 A→B、外部 SDK 消费、792 升级/缺源/坏 hash/漏 patch/
取消/交换失败控制（V-03/04）。816 handoff 的 R07 合同内注记（DELIVERY_CRATES 增补/
独立安装包资源布局）在本 Plan 收口（SD-02/04）。

## 8. 未闭决策（具名交后继任务，不缩原 AC）

1. auto-api-gen 最终仓属（语言仓 adapter vs UI 仓）——T-05 冻结（SD-07）；两侧现状：
   被 auto-ui-tools/rust_ui.rs 与 auto-man（语言服务）双向消费。
2. autodown-core 固定依赖机制（git rev/registry/vendor）——T-06 冻结。
3. 旧 auto-lang 兼容 crate 的 re-export vs 薄转发机制与 feature 矩阵——T-06 冻结（V-02）。
4. UI 仓 777 规则表与语言仓重采样后的并集责任——T-06/T-11。
5. CI 入口迁移清单（本地执行结果齐备，远端不在本 Plan）——T-09 冻结实际入口。
