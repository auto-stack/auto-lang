---
plan_id: PLAN-501
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: vm-desktop-osconfig-integration
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-501] vm 桌面 os-config 集成——系统设置的启动与调用

## 变更摘要

兑现 Design 25 S7 的完整语义（"系统 settings = **auto-os-config 的 UI 面**"）。
487 已落桌面自身配置的面板（shell.dock/notes/wallpaper 等自有键）；本期接通
`../auto-os-config` 项目——**统一 settings center**（一个 axum daemon `:17701`
读写 `~/.config/autoos/*.at` + 通用编辑器前端 + 模块注册表）——四块：

1. **daemon 生命周期管理**：桌面侧检活（ping `:17701`）→ 未运行则按发现序
   spawn（配置键 > 相邻仓 target > PATH）→ 失败进 offline 提示（os-config
   自带 daemon_view 连接测试 UX，不重复造）。
2. **外部仓 app 注册**：os-config 前端（`auto/src/front/*.at`，Auto 应用）
   作为**外部仓 app**进桌面 app 注册表（`ui/app_registry.rs` 的
   `scan_apps` 增可配置扫描根，storage 键 `shell.apps.extra_dirs`）。
3. **调用接线**：487 settings 面板增"系统设置（全部模块）"入口 → 既有
   `desktop.launch` 动词拉起 os-config app；`AUTOOS_DAEMON` env 注入该
   app 会话（os-config 既有 vm track 约定）。
4. **跨仓边界**：os-config 仓侧适配（如需）走其仓自有 worktree
   （auto-plan-work 依赖项目规则；plan 011 先例）。

## 目标

- **G1 daemon 生命周期**：桌面打开设置入口时懒起 daemon（检活 5s 超时 →
  spawn → 就绪 ping 通）；用户已有 daemon 运行则零打扰复用；起不来 =
  offline 状态提示（面板入口徽标 + daemon_view 既有 UX）。
- **G2 外部 app 装载**：os-config 前端在 vm 桌面内作为普通虚拟窗 App 启动
  （I3 单路径——与 examples app 同一装载管线，无专用分支）。
- **G3 调用闭环**：设置入口 → daemon 就绪 → os-config App 打开 → 模块列表
  读取（经 daemon API）→ 编辑一项配置 → `~/.config/autoos/*.at` 文件落盘。
- **G4 发现序可配置**：daemon 路径解析序 `shell.osconfig.daemon`（storage）
  > `../auto-os-config/target/release/auto-os-config-daemon(.exe)` > PATH；
  扫描根 `shell.apps.extra_dirs`（分号分隔，缺省含相邻仓探测）。
- **非目标**：vue/远程端嵌入（os-config 自有 vite 形态即可）；桌面自身键
  迁移进 os-config（487 面板保持自有，两面板入口互链即可）；daemon 的
  安装器/打包分发；os-config 模块本身的功能改动（除非适配必需）。

## 架构方案

```
settings.at「系统设置」入口 ──desktop.launch("os-config")──▶ app 注册表
                                                              ▲
shell.apps.extra_dirs（storage，含 ../auto-os-config/auto）───┘
launch 时：daemon mgr 检活(:17701 ping) ─未运行─▶ spawn(发现序) ─┐
           env 注入 AUTOOS_DAEMON=127.0.0.1:17701 ──────────────┴▶ App 会话
os-config App（既有 .at 前端）──http natives──▶ daemon(:17701) ──▶ ~/.config/autoos/*.at
```

- **新文件** `crates/auto-lang/src/ui/osconfig_daemon.rs`（检活/发现序/spawn
  决策纯逻辑 + 进程句柄管理）；**改动**：`ui/app_registry.rs`（多扫描根）、
  `crates/auto-lang/assets/settings.at`（入口）、session（launch 期 env 注入
  挂点）、`ui/mod.rs` 登记。
- **跨仓**：os-config 仓改动（如 daemon 加 `--quiet`/就绪探针、前端 desktop
  兼容微调）在 `../auto-os-config/.worktrees/auto-lang-dev` 开（依赖项目
  worktree 规则），随本计划消费即折回。

## 技术栈

既有 http natives（VM 进程内直连 daemon，无 CORS 议题）、std::process spawn、
既有 app 装载管线。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + ../auto-os-config 现场核验 2026-08-31）

- **os-config 现状**（README + 目录核验）：三支柱——① 统一 daemon
  （`auto-os-config-back/src/main.rs`，axum :17701，Auto `#[api]` 契约 +
  cdylib 桥，Plan 011 外部 back）；② 通用编辑器（`auto/src/front/
  config_editor.at` → Vue，形状驱动表单，零逐模块前端）；③ 模块注册表
  （`registry.rs` + `modules_store.at`）。**vm track 已存在**（README L71：
  VM 直连同 daemon，`AUTOOS_DAEMON` env，plan 010 track-parity 脚本）——
  plan 011 曾在本仓对拍（navVisible e2e 假阳性定性，VM renderer 无 bug）。
- **本仓缺口**：487 面板只覆盖自有键（其非目标明确"auto-os-config 跨仓深桥
  为后续计划"= 本计划）；app 注册表扫描根固定 examples（app_registry.rs:44
  `scan_apps(dir)` 单根 + :143 单测锁 examples），无外部仓条目机制；无
  daemon 生命周期管理。
- **前置**：487 ✅（settings 面板与 open_settings 动词就位）；os-config 仓
  plan 002/010/011 ✅。无硬阻塞。
- **排程**：与 497（shell.at 消费段轻同文件）/498/499/500 零~轻交叠，
  后合者 rebase；可并行领取。

## 详细设计

### 1. osconfig_daemon.rs（新）

- `DaemonStatus { Running(url) | Spawning | Offline(reason) }`；
- 纯逻辑：`resolve_daemon_path(order)`（发现序）、`should_spawn(ping_result)`、
  `env_for(url)`——全注入式可单测；
- 进程管理：spawn（detached 子进程，桌面退出**不**杀 daemon——它可能被
  vite/其他消费方共享；就绪等待 = ping 轮询 ≤5s）。

### 2. app_registry 多扫描根

- `scan_apps` 调用方（boot 期扫描）聚合 `shell.apps.extra_dirs`（storage；
  缺省追加 `../auto-os-config/auto` 探测存在则含）；条目去重（name 冲突
  以 examples 优先）。

### 3. launch 期 env 注入

- launch 动词执行臂：目标 app 为 os-config 条目（或声明了 `daemon: autoos`
  的 pac 字段——形态执行期定，倾向 pac.at 可选字段 `env AUTOOS_DAEMON`）
  → 先走 daemon mgr 确保就绪 → 注入 env → 常规装载。

### 4. settings.at 入口

- 「系统设置（全部模块）」按钮 → `desktop.launch("os-config")`；offline
  徽标态（daemon 起不来时入口置灰 + 提示，点击重试）。

## 测试设计

1. **T1 单元**：发现序解析（多级缺省/覆盖）、should_spawn 决策、env 构造、
   extra_dirs 聚合去重。
2. **T2 装载测**：extra_dirs 含 os-config 根时注册表出现条目；settings 入口
   渲染与 launch 动词派发（desktop_mcp 同型）。
3. **T3 集成（本机 daemon）**：起 daemon → launch → 模块列表非空 → 编辑
   一项 → `~/.config/autoos/` 对应 `.at` 文件内容变化断言（临时
   `AUTOOS_CONFIG_ROOT` 重定向家目录——os-config 仓若已有此 env 沿用，
   否则跨仓小改）。
4. **T4 实机**：设置入口全流程；daemon 预运行复用（不重复 spawn）；
   daemon 不可达时的 offline 徽标与重试。

## 验收标准

1. T1–T3 绿；T4 实机清单 PASS 留痕。
2. G3 闭环可演示：入口 → 模块列表 → 改配置 → 文件落盘。
3. 既有 examples 扫描行为零变化（app_registry 既有单测不回归）；
   `cargo t ui`、`cargo t session` 不回归；零警告。
4. 跨仓改动（若有）已在 os-config 仓落地并折回，本仓不携带其仓代码。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **daemon mgr 纯逻辑**：新建 `crates/auto-lang/src/ui/osconfig_daemon.rs`
   （发现序/决策/env 纯函数 + T1 单测）+ `ui/mod.rs` 登记。
   验证：`cargo check -p auto-lang && cargo t osconfig_daemon`。
   [✅ 已完成] `cargo check -p auto-lang` 绿；T1 6/6 绿（`cargo nextest run -p
   auto-lang --lib --features ui-iced osconfig_daemon`——ui 模块测试需显式
   ui-iced feature，裸 `cargo t` 默认特性不编译 ui）。现场核验修正：daemon
   二进制实际为 `../auto-os-config/auto-os-config-back/target/release/
   auto-os-config-back-server(.exe)`（Cargo [[bin]] 名），非计划原文的
   `target/release/auto-os-config-daemon`；发现序结构不变。
2. **进程管理**：osconfig_daemon.rs 增 spawn + ping 就绪轮询 + 句柄管理。
   验证：`cargo t osconfig_daemon`（检活/spawn 状态机单测）。
   [✅ 已完成] 14/14 绿（`cargo nextest run -p auto-lang --lib --features
   ui-iced osconfig_daemon`）。ping 用 std TcpStream 裸 HTTP（reqwest
   blocking 在 tokio 上下文会 panic——桌面宿主持全局 runtime，故弃）；
   spawn detached（Win DETACHED_PROCESS|CREATE_NEW_PROCESS_GROUP，stdio
   null，句柄即弃不 kill）；状态机注入式（DaemonIo trait，假 IO 覆盖
   复用/未找到/spawn 失败/就绪/超时五分支）+ 真 TCP ping 双用例；spawn env
   带 AUTOOS_BACK_PORT=17701（daemon 缺省 17901）。
3. **注册表多根**：`crates/auto-lang/src/ui/app_registry.rs` 扫描聚合
   extra_dirs（storage 键 + 相邻仓探测）+ 去重 + T1 聚合单测。
   验证：`cargo t app_registry`（既有 27 apps 单测不回归 + 新增用例）。
   [✅ 已完成] app_registry 9/9 绿（`cargo nextest run -p auto-lang --lib
   --features ui-iced app_registry`——既有 27-apps/launch 三连测不回归 +
   parse_extra_dirs/extra_roots 决策矩阵/aggregate 去重三新测）。新增
   `scan_app_root`（自含根）/`parse_extra_dirs`（`;` 分隔，`id=path` 或
   `path`→末段 id）/`extra_roots_from`（纯决策：storage > 探测缺省
   `os-config`，`shell.apps.scan_siblings=false` 关）/`aggregate_scan`
   （id 去重主根优先）/`host_extra_roots`（boot 包装）；`AppRegistryEntry`
   增 `daemon` 字段（pac `daemon:` 声明，step 4 env 注入数据面）；renderer
   boot 调用方换 `aggregate_scan`。id 裁定：探测缺省与 `id=path` 显式
   给出 `os-config`（目录名 `auto` 无桌面语义不采）。
4. **launch env 注入**：`crates/auto-lang/src/ui/session.rs` launch 执行臂
   增 daemon 就绪 + env 注入（pac 可选字段形态，执行期定）。
   验证：`cargo t session`。
   [✅ 已完成] session 58/58 绿（`cargo nextest run -p auto-lang --lib
   --features ui-iced session::`；55 既有 + 3 新：就绪注入 env/Offline 不阻断
   launch 且不注 env/无声明不触探活）。定稿（待澄清①）：**pac.at 可选字段
   `daemon: autoos`**（通用机制）——`LaunchSpec`/`AppRegistryEntry` 增
   `daemon` 字段透传；launch_app 在 build（Init 链打 daemon）前
   ensure_ready → Running 则 `std::env::set_var("AUTOOS_DAEMON", url)`
   （VM Env.get 即进程 env）；**Offline 不阻断 launch**（App 自带
   daemon_view 连接 UX，G1 不重复造），原因记 `DesktopState.osconfig_status`
   （step 5 徽标消费）；`osconfig_daemon_probe` 注入位供单测假实现。
   broker/stage3/app_registry 连带 9/9+23/23 绿。
5. **settings 入口**：`crates/auto-lang/assets/settings.at` 增「系统设置」
   按钮 + offline 徽标态 + launch 派发。
   验证：`cargo t desktop_mcp`（T2 用例）。
   [✅ 已完成] settings 9/9 绿（`cargo nextest run -p auto-lang --lib
   --features ui-iced settings`；既有 8 测不回归 + 新测
   settings_osconfig_entry_badge_and_launch_dispatch——三态徽标注入
   unknown/offline/ready + 原因投影 + OpenSystemSettings 派发
   launch\tos-config 记录 + 自隐 + 记录→LaunchApp 解析）。settings.at 增
   「系统」分区（五分区）：入口卡（打开/重试并打开双态按钮——offline 置灰
   提示，点击即重试，launch 每次重新探活零额外动词）+ ready 已连接文案；
   召唤注入 `osconfig_state`/`osconfig_hint`（badge_projection 纯函数
   投影会话域 status）；renderer 74/74 全绿。
6. **T3 集成**：daemon 起停 + 全链用例（注册表条目 → launch → 模块列表 →
   改写落盘断言；配置根重定向 env）。
   验证：`cargo t osconfig`（集成档，feature/ignore 门控按 daemon 可用性）。
   [✅ 已完成] `cargo nextest run -p auto-lang --features ui-iced --test
   osconfig_integration` 1/1 绿（1.09s；材料门控：相邻仓前端 + release
   daemon 二进制 + cdylib 任一缺席即 eprintln 跳过）。六段面包屑：A daemon
   起（随机端口 + USERPROFILE/HOME 重定向配置根——daemon config_root 读
   env，**待澄清③的跨仓 config root env 因此非必需**）→ B 就绪 ping →
   C launch（真相邻仓条目）→ D App Init 真数据（sys_host 非空）→ E
   GET /api/modules ≥7 → F PUT ai-daemon.at → 落盘断言（daemon 对缺席
   配置 404 不自动建桩——测试预置基线 atom）。**执行期发现（现场核验
   补录）**：os-config vm 轨 `use back.api` 依赖 Plan 061 外部 back 链
   （pac `back: { project }`）——本地 src/back/api.at 残缺、后端 api.at
   为桩、#[api] 真身在 cdylib；桌面 launch 臂补齐 `set_external_back_root`
   + `load_back_cdylib`（auto-man rust_ui 同型复刻，句柄驻
   DesktopState.back_keepalive）。
7. **跨仓适配（视需）**：os-config 仓 worktree（`../auto-os-config/
   .worktrees/auto-lang-dev`）落必要适配（quiet/探针/config root env），
   消费验证后折回其 master。
   验证：跨仓 PR/commit 号记入本计划 + T3 复跑绿。
   [✅ 已完成] 实际必需面收窄为一行：`auto/pac.at` 增 `daemon: "autoos"`
   （跨仓 commit **0e81196**，`auto-lang-dev` worktree 开发→折回其 main→
   worktree/branch 已清）。config root env 非必需（T3 经 USERPROFILE/HOME
   重定向达成，见步骤 6）；quiet 非必需（桌面 spawn stdio null）。本仓
   消费验证：T3 改用 pac 自然 daemon 字段（断言
   `entry.daemon == Some("autoos")`）复跑 1/1 绿（779204fbf）。本仓不携带
   其仓代码 ✓。
8. **实机冒烟 + 收尾**：T4 清单留痕；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] `cargo check -p auto-lang` 绿 + `cargo t ui` 777/777 绿 +
   scoped 复验 156/156（session/app_registry/osconfig_daemon/renderer）+
   T3 1/1。**T4 清单留痕**：①boot 冒烟 A/B——worktree 构建的 ui_desktop
   实进程，CWD=主仓根（相邻仓在）注册表 **35** 条 vs CWD=.worktrees
   （无相邻仓）**34** 条（Δ1 = os-config 相邻仓探测 live 生效，无 boot
   回归）；②ensure_ready 生产路径 live——真实发现序（相邻仓 target）→
   detached spawn → 就绪 2.52s → 二次调用复用 **774µs**（ping 通即返零
   打扰）；③offline 徽标与重试——T2 单测级验证（三态注入 + 置灰提示 +
   点击重试语义）。**残差（用户 30 秒抽查）**：齿轮 → 系统 → 打开系统
   设置的人手点击链（GUI 像素自动化与 iced 实时渲染栅格竞态不可靠，
   未强行驱动）；无头等价链（T2 派发 + T3 launch 全链 + boot 冒烟）已全
   绿。runbook：`cd <本仓> && cargo run -p auto-lang --features ui-iced
   --example ui_desktop`（相邻仓探测 CWD 相对——自仓根起跑）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

（执行期全部定案，2026-08-31）

- **env 注入形态** ✅ **pac.at 可选字段 `daemon: autoos`**（通用机制）——
  `AppRegistryEntry.daemon`/`LaunchSpec.daemon` 透传；跨仓 0e81196 落行，
  T3 断言自然消费。
- **daemon 退出策略** ✅ v1 = 桌面退出不杀（共享服务语义）；detached
  spawn（Win DETACHED_PROCESS|CREATE_NEW_PROCESS_GROUP + stdio null，
  句柄即弃）。"桌面带走"开关未做（复审若要另立小改）。
- **配置根重定向** ✅ 非必需跨仓机制——daemon config_root() 读
  USERPROFILE/HOME env，T3 经 spawn 期 env 重定向即达成零污染测试。
- **vue 端** ✅ 维持非目标（远程桌面场景走 os-config 自有 vite 形态）。
- **extra_dirs 缺省探测** ✅ v1 含相邻仓探测 + `shell.apps.scan_siblings`
  storage 可关；`shell.apps.extra_dirs` 语法 `id=path` 或 `path`（末段为
  id），探测缺省产出 id `os-config`（目录名 `auto` 无桌面语义不采）。
- **执行期新知（补录）**：①daemon 二进制实为 `auto-os-config-back/
  target/release/auto-os-config-back-server(.exe)`（Cargo [[bin]] 名，
  计划原文路径有误——已按现场核验兑现）；②生产端口 17701 需 spawn 期
  `AUTOOS_BACK_PORT` 覆盖（daemon 缺省 17901）；③os-config vm 轨依赖
  Plan 061 外部 back 链（`back: { project }` → set_external_back_root +
  cdylib 桩桥装载）——桌面 launch 臂已补齐（auto-man rust_ui 同型复刻）。
