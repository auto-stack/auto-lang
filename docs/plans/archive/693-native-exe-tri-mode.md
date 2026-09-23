---
plan_id: PLAN-693
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: native-exe-tri-mode
author: [zcode]
created_at: 2026-09-23
updated_at: 2026-09-23

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 a2r exe 三模式 CLI 契约（independent/rq/desktop + desktop-endpoint）, SD-02 desktop 合成器接入契约（端点协议与不孵化语义）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/rq-remote-renderer.md, docs/design/autoui/desktop-protocol-v1.md]
current_step: 4
total_steps: 5
---

# [PLAN-693] native-exe-tri-mode——a2r exe 三模式参数化 + 虚拟桌面接入

> 来源：2026-09-23 用户裁定：①不做缺省翻转（F-3 关闭）——缺省维持 iced 底座
> 合一（独立轨），RQ 经 `-q`/参数显式选用；②Rust 版 exe 支持三模式参数选择：
> 独立启动 / RQ 模式 / **虚拟桌面模式**（需新参数指定与虚拟桌面进程的通信
> 端点）。设计落点 [rq-remote-renderer](../design/autoui/rq-remote-renderer.md)
> §8（新增）。

## 变更摘要

a2r 编译 exe 的渲染模式参数化 + 虚拟桌面接入：

1. **CLI 参数面**：`--render-mode <independent|rq|desktop>`（缺省
   independent）+ `-q`/`--rq` 语义糖（=rq）+ `--desktop-endpoint <pipe>`
   （desktop 模式必需）。优先级链：CLI > `AUTO_VM_RENDER` env > pac
   `desktop_render` > independent。
2. **ClientTarget::Desktop**：连接虚拟桌面进程的合成器端点——连接语义与
   rqhost 完全同构（rendezvous/adopt 协议零变化），差异仅端点来源（参数
   指定）与**不孵化语义**（桌面已在前，端点缺席 = 报错提示先启动桌面，
   不代孵）。
3. **桌面端契约**：虚拟桌面进程 = rqhost daemon 形态内嵌（shell 进程调
   `run_daemon(desktop_pipe)` 库形态）或子进程 `auto rqhost --pipe
   <desktop_pipe>`——采纳协议不变；auto-os 侧落地为跨仓配对项。

## 目标

- **G1（CLI 参数面）**：三模式 CLI 解析 + 优先级链贯通（CLI > env > pac >
  缺省），a2r 生成 exe 实机可切。
- **G2（desktop 接入）**：`ClientTarget::Desktop` 连接语义（不孵化）+
  端点缺席报错路径 + 实机与桌面端点互连。
- **G3（F-3 关闭）**：缺省翻转裁定关闭记录入设计档与债册口径（remote 恒
  为显式选用）。

**非目标**：不做缺省翻转（F-3 关闭）；不做桌面 shell 内嵌 rqhost 的
auto-os 侧实现（跨仓配对，契约由本计划定义）；不改 wire 协议
（DisplayList v2 沿用）。

**成功标准**：同一 a2r exe 以三种参数启动得到三种正确形态；desktop 模式
在虚拟桌面端点在位时正常宿主渲染、缺席时干净报错。

## 架构方案

```
a2r exe（编译组件）
  main → 解析 --render-mode/--desktop-endpoint
  ├─ independent → run_dynamic_iced（现状，本地 wgpu 自窗）
  ├─ rq          → ClientTarget::Rqhost{wellknown=autodesk-rqhost}
  │                + ensure_rqhost_ready（探测→孵化，现状 -q 语义）
  └─ desktop     → ClientTarget::Desktop{endpoint}（新增）
                   + ensure 探测端点（在=adopt 连接；不在=报错，
                     不孵化——桌面进程是宿主，先于 app 在）
```

- 连接/采纳协议与 rqhost 零变化（`adopt␟<app_name>` rendezvous、per-app
  管道、DisplayList v2 帧、InputMsg 回路径）——**mode 2/3 唯一差异 =
  端点字符串来源与孵化策略**；
- 桌面端（auto-os）：虚拟桌面进程以库形态内嵌 rqhost daemon
  （`rqhost::run_daemon(desktop_pipe)`）或子进程 `auto rqhost --pipe
  <desktop_pipe>`——采纳协议自洽，桌面 shell 获得全部宿主能力
  （多 app 窗、托管、DisplayList v2 渲染）。

## 需求分析与背景调查

- **授权记录**：用户 2026-09-23 裁定（三模式参数化 + 虚拟桌面新参数 +
  F-3 不翻转），本计划直接承载。
- **代码实勘**：
  - `client_entry.rs:25 ClientOpts`（`remote: bool` 已在——PLAN-683）/
    `:254 run_native_client` remote 臂已通（headless 宿主，a2r 组件同臂）；
  - `ClientTarget::Rqhost { wellknown }` 管道名已参数化——Desktop 变体
    为其端点特化（新增枚举值或参数化字段，T-01 定）；
  - 选择面现状：仅 `AUTO_VM_RENDER=remote` env（`rqhost.rs:741`）+
    pac `desktop_render` + `auto run -q`（`main.rs:1249` 设 env）——无 exe
    自身 CLI 面；
  - a2r 生成 main = 空壳模板（`ui_gen/rust.rs:12852 fn main() {{}}`）——
    运行入口在 runtime；CLI 解析落点 = runtime 启动函数 + 生成 main 透传
    args（T-01 勘定透传形）。
- **虚拟桌面**：Design 23（virtual-desktop）+ auto-os 桌面 shell——桌面
  进程 = shell；合成器角色由内嵌 rqhost 承担（本计划定义接入契约）。

## 详细设计

### CLI 面

```
<exe> [--render-mode independent|rq|desktop]
      [-q | --rq]                    # = --render-mode rq
      [--desktop-endpoint <pipe>]    # desktop 必需
      [--window <WxH>] [--title <t>] # 既有 AUTO_VM_WINDOW/TITLE 的 CLI 形
```

- 解析：手写轻量解析（不引 clap——a2r exe 依赖面克制），未知参数容错
  透传（v1 语义）；
- 优先级：CLI > `AUTO_VM_RENDER` env > pac `desktop_render` > independent；
- desktop 模式缺 `--desktop-endpoint` → 报错（列出探测过的 wellknown）。

### ClientTarget::Desktop

```rust
enum ClientTarget {
    Rqhost { wellknown: String },          // 现状（rq 模式）
    Desktop { endpoint: String },          // 新增：不孵化，探测即连
}
```

- `connect` 路径复用（rendezvous/adopt 同构）；差异：端点连接失败 →
  `Err("desktop endpoint <pipe> 不可达——请先启动虚拟桌面")`（rq 模式的
  孵化分支不进入）；
- 远端（桌面进程）窗口宿主语义 v1 = rqhost 窗语义（桌面 shell 的 WM
  集成/任务栏联动 = auto-os 侧后续）。

### F-3 关闭

缺省翻转裁定关闭（用户 2026-09-23）：`desktop_render` 缺省恒 independent；
remote 族恒显式选用。PLAN-683 F-3 呈报项就此关闭（设计档 §8 同步）。

## 测试设计

- 单测：CLI 解析（三模式/优先级/缺参报错）；ClientTarget::Desktop 连接
  语义（端点不在=Err 不孵化；在=adopt——进程内 RqServe 替身，测试缝既有）；
- 环测（协议级）：desktop 模式管道环（同 rqhost 环，管道名参数化）；
- 实机：a2r 编译 exe 三模式各一跑（001 载体）+ desktop 模式与 auto-os
  桌面端点互连（跨仓协调到位后）。

## 验收标准

- [x] AC-01 同一 exe 三参数三形态：independent 自窗/rq 连 daemon/desktop
      连桌面端点，实机截图三张在档。（rq-cli / sidecar-pac /
      desktop-endpoint / independent-cli 四张 + 观测行，
      reports/p693-tri-mode/）
- [x] AC-02 优先级链测试绿（CLI 压 env 压 pac）。（render_cli 8/8：
      priority_chain_cli_beats_env_beats_pac + 缺省/跳过腿）
- [x] AC-03 desktop 模式端点缺席 = 干净报错不孵化（单测+实机）。
      （单测双态 3/3 + 实机报错列探测 wellknown 不孵化）
- [ ] AC-04 desktop 模式实机互连：虚拟桌面端点在位时 003 级交互闭环
      （依赖 auto-os 侧就绪，跨仓协调项可后置）。（rqhost 形态端点
      合同等价实机已交付——desktop-endpoint.png；真桌面 shell 互连 =
      T-04 后置，债册 P693-D2）
- [x] AC-05 F-3 关闭记录在设计档/债册口径在档；既有门禁零新增红。
      （设计档 §8 在位；desktop_protocol 213 run 211 绿，2 红 stash
      对照 base 同败全预存零新增）

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-01 CLI 解析 + 优先级链：runtime 启动函数参数面（生成 main 透传形
      勘定——`ui_gen/rust.rs:12852` 模板 + a2r-std 运行入口）。
      验证：单测（解析矩阵）+ 001 实机三参数。
      [✅ 已完成] 2026-09-23 work（abeaf113d）——**实勘勘误**：a2r 生成
      main 真身 = `auto-man/rust_ui.rs` native_client_gate（计划所引
      `ui_gen/rust.rs:12852` 系 code-editor e2e 测试内临时 crate，非生成
      模板；a2r-std 无 UI 运行入口——CLI 解析落点勘定为 auto-lang runtime
      单源 + 生成 gate 调用，语义不变）。新增
      `desktop_protocol/render_cli.rs`（解析/优先级链/旁车 pac 单源）+
      rust_ui.rs gate 三模式臂前置。单测 8/8 + 001 实机（rq CLI 腿/
      rq sidecar pac 腿/independent CLI 压 sidecar 三观测行 + 截图）。
- [x] T-02 `ClientTarget::Desktop` 连接语义（不孵化）+ 报错路径。
      验证：单测（替身端点不在/在双态）。
      [✅ 已完成] client_entry.rs Desktop 变体（adopt 同构 + exit-on-EOF
      同 rqhost）+ 单测双态 3/3 + 实机（rqhost 形态端点合同等价：
      desktop-endpoint.png；缺端点干净报错列探测 wellknown）。
- [x] T-03 rq 模式 CLI 化（`-q`/`--rq` 映射现 env 链，env 保留兼容）。
      验证：001 rq 实机。
      [✅ 已完成] `-q`/`--rq` 语义糖（解析单源）+ exe 侧
      ensure_rqhost_ready（`auto run -q` 宿主语义内联）+ 实机（-q 糖采纳
      +宿主 `auto run -r rust -q` 全链含 host 孵化——AUTO_VM_RENDER env
      腿自此补通，见 T-01 勘误与报告 README）。
- [ ] T-04 desktop 模式实机（配合 auto-os 侧端点；跨仓协调项就绪后）。
      验证：AC-04 录证。
      （后置在案：依赖 auto-os 桌面 shell 内嵌 rqhost daemon + 管道命名
      约定对齐（待澄清#1）；desktop 连接契约与 rqhost 形态端点合同等价
      实机已交付（T-02），余面=真桌面 shell 互连录证——债册 P693-D2。）
- [x] T-05 复审收口：F-3 关闭记录落设计档 §8（已预写）+ 债册口径 +
      specs 沉淀（SD-01/02）。
      [✅ 已完成] 设计档 §8 预写核对在位（4dec4f698 随立项落）；
      债册 P693-D1（rust-workspace 存量 member 陈旧 Cargo.toml ui-gpui
      行，重生成自然收敛）/P693-D2（T-04 跨仓配对）登记；
      SD-01/SD-02 delta 与实际一致（render_cli 单源 + Desktop 变体），
      specs.json upsert 留 merge 段。

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-693 rev1。outcome=pass。
  next=work（T-04 依赖 auto-os 侧端点就绪，可后置）。
- work 交付（2026-09-23）：stage=work | plan_id=PLAN-693 | plan_revision=1 |
  outcome=pass | code_commit=abeaf113d（worktree
  `D:/autostack/.wt/lang-693/auto-lang`，分支 plan-693-dev，base 4dec4f698；
  组内依赖位 auto-down detached @3373a5c）| task_ids=T-01,T-02,T-03,T-05
  （T-04 后置在案）| evidence=render_cli 8/8 + client_entry 3/3 +
  auto-man generated_main 2/2 + desktop_protocol 213 run 211 绿（2 红
  stash 对照 base 同败全预存）+ 全域 ad-hoc 套件 5778/5791（13 红全
  预存：已知清单 12 + P693-D3 外科勘定新增 1，零新增红）+ 001 实机
  三形态四截图+观测行在档（reports/p693-tri-mode/）| blockers=T-04/AC-04
  （auto-os 桌面 shell 内嵌 rqhost daemon + 管道命名对齐——unblock 见
  待澄清#1 与债册 P693-D2）| next=review。
  执行期增量（随计划暴露的双潜伏缺口根修，同批交付）：①PLAN-683
  ClientOpts.remote 未同步生成器→生成 exe E0063 不可编（旧臂 remote:false
  归位+新臂 remote:true——rust 轨 `-q --render=remote` env 链补通）；
  ②PLAN-691 ui-gpui 移除后生成器残留 emission→fresh 生成 cargo resolve
  即败（GPUI 臂随 feature 摘除）。
- review 交付（2026-09-23，随 PLAN-694 收口联审）：stage=review，
  PLAN-693 rev1。outcome=**pass**。reviewed_commit=plan-693-dev
  **c1ce13933**（abeaf113d rebase onto master 的等价重写，range-diff
  `abeaf113d = c1ce13933` 全等；worktree lang-693 clean）。**独立性
  声明**：复审由 PLAN-694 会话执行（原实现会话已结束），判定由工件
  重建——作用域验证在重排 tip 刷新：render_cli+client_entry 11/11、
  generated_main 2/2；desktop_protocol 213/215（2 红=预存，693 报告
  stash 对照在案）；实机三形态证据 reports/p693-tri-mode/（四截图+
  观测行，随提交入库）。**AC 对勘**：AC-01 pass（三参数三形态实机）；
  AC-02 pass（优先级链单测+实机）；AC-03 pass（缺端点干净报错不孵化，
  单测+实机）；AC-04 pass（**由 PLAN-694 兑现**——桌面宿主本尊=桌面
  端点（无需 auto-os 内嵌 daemon）：spawn 双方言+broker 双动词+环测
  全环（003-converter 载体 attach/交互/回收）+注册表发现面实证；auto-os
  shell 侧集成残面保持 P693-D2 在案）。执行期增量双根修（E0063/gpui
  残留）经 694 的 generated_main pin+003 fresh 重生成实证收敛（P693-D1
  实质销号）。**findings**：无阻断项。next=**merge**（fold 顺序
  693→694，与 PLAN-694 叠栈一致）。
- merge 交付（2026-09-23）：stage=merge，PLAN-693:r1。outcome=**pass**
  （收据 PLAN-693:r1）。prepared=规范增量经设计档 rq-remote-renderer §8
  承载（先例在案）+账本 P693-1/2。landed=rebase abeaf113d→c1ce13933→
  **5bc78de0b** 两段 range-diff 全等（映射在案），master ff-only 无合并
  提交；master 冒烟 16/16。ledger_refreshed=specs.json P693-1
  （architecture）/P693-2（reviews）+ui/plans.md 行+INDEX 再生。
  archived=docs/plans/archive/693-native-exe-tri-mode.md。
  cleaned=worktree lang-693+分支 plan-693-dev（wt-guard clean 后移除）。

## 待澄清事项

1. 桌面合成器管道命名约定（auto-os 协调：固定 wellknown vs 桌面注册表
   发布）——T-04 前与 auto-os 侧对齐。
2. desktop 模式窗口宿主语义 v1 边界（rqhost 窗语义 vs 桌面 WM 深度集成）
   ——v1 取 rqhost 窗语义（本计划），深度集成为 auto-os 侧后续。
3. （work 期新增）独立窗臂窗尺寸/标题 CLI 面仅作用于三模式臂 v1（旧
   孵化注入臂保持 480×320 硬编码零变化——宿主编排面窗语义属既有编排
   契约，I1 边界）；`--window`/`--title` 对旧臂生效与否留 review 裁量。
