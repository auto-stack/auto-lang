---
plan_id: PLAN-501
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-desktop-osconfig-integration
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
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
2. **进程管理**：osconfig_daemon.rs 增 spawn + ping 就绪轮询 + 句柄管理。
   验证：`cargo t osconfig_daemon`（检活/spawn 状态机单测）。
3. **注册表多根**：`crates/auto-lang/src/ui/app_registry.rs` 扫描聚合
   extra_dirs（storage 键 + 相邻仓探测）+ 去重 + T1 聚合单测。
   验证：`cargo t app_registry`（既有 27 apps 单测不回归 + 新增用例）。
4. **launch env 注入**：`crates/auto-lang/src/ui/session.rs` launch 执行臂
   增 daemon 就绪 + env 注入（pac 可选字段形态，执行期定）。
   验证：`cargo t session`。
5. **settings 入口**：`crates/auto-lang/assets/settings.at` 增「系统设置」
   按钮 + offline 徽标态 + launch 派发。
   验证：`cargo t desktop_mcp`（T2 用例）。
6. **T3 集成**：daemon 起停 + 全链用例（注册表条目 → launch → 模块列表 →
   改写落盘断言；配置根重定向 env）。
   验证：`cargo t osconfig`（集成档，feature/ignore 门控按 daemon 可用性）。
7. **跨仓适配（视需）**：os-config 仓 worktree（`../auto-os-config/
   .worktrees/auto-lang-dev`）落必要适配（quiet/探针/config root env），
   消费验证后折回其 master。
   验证：跨仓 PR/commit 号记入本计划 + T3 复跑绿。
8. **实机冒烟 + 收尾**：T4 清单留痕；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **env 注入形态**：pac.at 可选字段（如 `daemon_env: autoos`）vs 条目名
  硬编码白名单——倾向前者（外部 app 机制的通用性），T4 执行期定稿。
- **daemon 退出策略**：桌面退出不杀（共享服务语义）为 v1 裁定；若需
  "桌面拉起桌面带走"，加 storage 开关，复审时定。
- **配置根重定向**：T3 需要 os-config daemon 支持 config root env 重定向
  （避免测试污染真实 `~/.config/autoos/`）——若其仓无此机制，T7 跨仓补
  （小改）。
- **vue 端**：远程桌面场景 os-config 走其自有 vite 形态（非目标重申）；
  未来若 vue 桌面要内嵌，另立计划（走 465 宿主机制）。
- extra_dirs 缺省是否含相邻仓探测（`../auto-os-config`）：开发机友好 vs
  安装形态纯净——v1 含探测 + storage 可关（`shell.apps.scan_siblings`）。
