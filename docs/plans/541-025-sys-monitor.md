---
plan_id: PLAN-541
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 025-sys-monitor——dashboard 升级真后端任务管理器
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 20
---

# [PLAN-541] 025-sys-monitor——dashboard 升级真后端任务管理器

## 变更摘要

把 `examples/ui/025-dashboard`（Plan 438 的 mock 系统监视器）升级为**真数据源的
Windows 任务管理器形态应用**，并改名 **sys-monitor**：

1. **真后端**：新增 `auto.sys.*` 原生函数族（sysinfo crate 宿主实现），后端按
   015-notes 的 `api.at` 双模式契约（`--server vm` AutoVM HTTP / `api: "rust"`
   a2r Rust）提供真实系统数据；前端 mock 随机游走整体退役。
2. **任务管理器功能**：Win11 风四页（进程/性能/详细信息/用户）+ 可排序多列
   进程表 + **结束任务**（kill）+ CPU/内存/磁盘/网络四实时曲线 + 核级占用
   + 系统信息卡。
3. **虚拟桌面 / os-config 配合**：pac.at 补 `icon:`/`category:` 进桌面注册表；
   根组件声明 `dark_mode`/`accent_color` 吃桌面播种与 os-config 主题链
   （Plan 504 S7：CLI > `~/.config/autoos/apps/sys-monitor/config.at` > pac.at）。
4. **改名**：`025-dashboard` → `025-sys-monitor`（041-code-editor→auto-edit 先例），
   pac name `dashboard` → `sys-monitor`，持久化键 `dash.*` → `sysmon.*`（带旧键回退）。

## 目标

- G1：双端（vue / vm-iced）显示**真实**系统数据：CPU%、内存总量/占用、
  网络速率、磁盘、进程列表（真实 PID/名称/状态），数值可与 Windows 任务管理器
  对照（同一数量级、同名进程可检索到）。
- G2：**结束任务**真实生效：在进程页 kill 一个测试进程，进程从列表消失
  （`sys.kill` → sysinfo terminate）。
- G3：任务管理器四页 UI 完整可交互：tab 切换、列排序（持久化）、刷新间隔
  三档 + 暂停/继续、曲线独立开关沿袭。
- G4：虚拟桌面注册表正确呈现（icon/category/name），launcher 可检索、
  `desktop.launch("sys-monitor")` 可启动。
- G5：os-config 主题链生效：`~/.config/autoos/apps/sys-monitor/config.at`
  写 `theme: "light"` 后应用以浅色启动（无 CLI 覆盖时）。
- G6：文档与测试收口：examples/ui README、Design 21 矩阵、SPEC 重写、
  KNOWN-DEBT 登记；playwright smoke + desktop_mcp 双套测试重建全绿。

## 非目标（明确不做，登记 KNOWN-DEBT）

- **启动应用**页（需 Windows 注册表/WMI，sysinfo 不覆盖）。
- **服务**页、**应用历史记录**页（无真实使用量数据源）。
- **GPU 监控**（sysinfo 无 GPU 采样；按 Plan 539 FFI 路线远期）。
- 应用内主题切换面板（主题归桌面/os-config 统一管理，本 app 只声明变量吃播种）。
- a2r 全 Rust 部署的性能优化（a2r-std sys 族只做能力对齐，不做调优）。

## 架构方案

### 1. 数据通路：`auto.sys` 原生函数族 + 015 式 api.at 双模式后端

```
                  ┌─ vue 轨:  vite /api 代理 → AutoVM HTTP server（--server vm 默认）
front .at ──► use back.api: system_snapshot()/kill_process()
                  │              ▲
                  └─ vm 轨:      │（split 模式 HTTP；merge 模式进程内直调）
                     src/back/api.at + sys_info.at ──调用──► sys.* natives
                                                              │
                                       宿主实现: sysinfo crate（crates/auto-lang/src/libs/sys.rs 扩展）
                                       转译实现: crates/a2r-std/src/sys.rs（api:"rust" 路对齐）
```

- **契约层**：`src/back/api.at` 定义类型与端点（015-notes 先例：
  `#[api(method, path)]` + 前端 `use back.api:` 直接导入）。单一聚合端点
  `GET /api/system/snapshot`（250ms 轮询只打一发）+ `POST /api/system/kill`。
- **实现层**：`src/back/sys_info.at` 用 `sys.*` natives 组装快照——015 的
  db.at 角色换成了 sysinfo 桥。
- **双模式语义**：`--server vm`（默认）= AutoVM 解释执行 back .at 并起 HTTP；
  `api: "rust"` = a2r 生成 Rust 服务器。两路的数据**都是真的**（native 族两路
  各有一份 sysinfo 实现），这是选 native 方案而非"手写 axum 后端"的核心理由：
  不引入第三条代码路径，015 双模式语义原样保留。
- **速率/增量在宿主侧算**：CPU% 、网速 KB/s、每进程磁盘/网络 KB/s 都需要
  两次采样差分——在 `libs/sys.rs` 用 `Lazy<Static>` 保存上次样本（含死进程
  PID 清理），`.at` 侧拿到的直接是速率值。理由：`.at`/前端无跨 tick 静态态
  存储的干净写法，且差分窗口与刷新间隔解耦后前端换档不再影响数值稳定性。

### 2. 前端形态：Win11 任务管理器四页 + SharedStore

- 多模块 front（015 vm 已验证的**平铺**结构，不用 pages/ 子目录——vm 轨模块
  解析对子目录无先例）：
  - `app.at`：壳 = 左侧 nav rail（进程/性能/详细信息/用户）+ 顶栏（标题 +
    间隔三档 + 暂停/继续）+ `.Tick` 轮询快照写入 store + `active_tab` 切换。
  - `sys_store.at`：SharedStore（015 notes_store 先例）——snapshot、四路
    滑窗历史（30 点）、排序键/向、选中 PID、刷新档位。
  - `processes.at` / `performance.at` / `details_users.at`：三个页面 widget。
- **进程页**：顶部四张 summary tile（CPU/内存/磁盘/网络，Win11 风）+ 进程表
  （名称/CPU/内存/磁盘/网络/状态）+ 行选中 + 行内「结束任务」按钮——点击不
  直接 kill，先弹 **alert-dialog 二次确认**（标题带目标进程名，action 才执行
  `kill_process(pid)`，cancel/外点/Esc 均不杀；shadcn AlertDialog 语义，
  overlay-probe 先例：开=置 show 状态，关=仅 cancel/action）。
- **性能页**：左 mini rail 四项点选 + 右侧大 SVG 面积图（现有 path 几何模式
  扩到四图四定标）+ CPU 核级 sparkline 网格 + 系统信息卡（os/version/kernel/
  hostname/uptime/cpu_brand）。
- **详细信息页**：全量进程表（PID/名称/状态/用户/CPU/内存，PID 排序新增）。
- **用户页**：`sys.users()` + 按用户聚合进程数/内存合计。
- mock 随机游走、假进程数组**整体删除**（SPEC 同步重写，git 历史留档）。

### 3. 桌面 / os-config 配合（消费已落地机制，零新基建）

- **注册表**：Design 24 R10——桌面注册表扫描 apps 目录 pac.at，`icon:`/
  `category:` 缺失回退灰图标。本 plan 补 `icon: "activity"`、`category: "system"`
  （028-launcher 同款字段）。
- **主题链**（Plan 504 S7 已落地）：`CLI > ~/.config/autoos/apps/<pac name>/
  config.at > pac.at theme:/accent: > 内置默认`。pac 定名 `sys-monitor` 后，
  os-config 侧按名字落配置文件即生效，**无需本仓改码**。
- **运行时播种**：根组件声明 `dark_mode`(bool)/`accent_color`(str) 状态变量
  （006-hero-section 先例；`osconfig_apps::seed_app_config` 只写已声明变量），
  桌面宿主 launch 时注入，VM renderer 每帧回读。
- 持久化键改名 `dash.*` → `sysmon.*`，Init 读 `sysmon.*` 缺失时回退 `dash.*`
  （老用户无感迁移）。

### 4. 改名（041 先例）

- `git mv examples/ui/025-dashboard examples/ui/025-sys-monitor`（同号保留，
  编号即构建/复杂度顺序不变；README 历史注记登记改名）。
- pac `name: "sys-monitor"`（= os-config 配置目录名 = 桌面 launch 名 =
  AUTO_APP_ID）；front_port 4025 不变，**新增 back_port 8025**（4025/8025
  配对惯例，同 018 3018/8018、026 4026/8026）。

## 技术栈

- Rust：`sysinfo` crate（workspace 依赖统一声明版本；VM 宿主与 a2r-std 各引一份）。
- AutoLang：back 模块（api.at/sys_info.at）、front widget/store、AURA widget 族
  （table/badge/checkbox/svg/progress 按 024/025 现状）。
- 测试：cargo nextest（`cargo t`  scoped）+ playwright smoke + desktop_mcp.py
  （013 惯例）+ autoui-verifier 双端走查。

## 需求分析与背景调查

**需求来源（用户 2026-09-04 四点）**：① 显示真正系统信息（真后台，参考
015-notes api.at 双模式）；② 尽可能接近 Windows 任务管理器功能；③ 风格配置
与虚拟桌面和 os-config 配合；④ dashboard 改名 sys-monitor。

**背景调查结论（执行时可直接引用的证据路径）**：

| 主题 | 结论 | 证据 |
|---|---|---|
| 后端双模式 | `--server vm`=AutoVM 解释 back .at 起 HTTP（默认）；`api:"rust"`=a2r 生成 Rust 服务器；split/merge 由 `AUTO_VM_MERGE` 控制，Plan 345 支持 `--render=vm --server=rust` | crates/auto/src/main.rs:917-938；crates/auto-man/src/automan.rs:1447,1465 |
| 前端调后端 | `use back.api:` 直接导入；vue 生成 `gen/front/vue/src/lib/api.ts` fetch `/api/*` | examples/ui/015-notes/src/front/notes_store.at:6；015-notes/gen/front/vue/src/lib/api.ts:16 |
| api.at 契约 | `#[api(method,path)]` 端点 + 共享类型定义 | examples/ui/015-notes/src/back/api.at |
| 原生函数族 | `libs/sys.rs` 现仅 getpid；math.* 族是命名空间 native 的注册模板（native_catalog 条目 + shim + libs 实现）；a2r 侧镜像在 a2r-std/src/math.rs | crates/auto-lang/src/libs/sys.rs；crates/auto-lang/src/vm/native_catalog.rs:299-317；crates/a2r-std/src/ |
| 025 现状 | 914 行单文件 app.at；mock 随机游走；`.Tick`/`interval`/`speedDiv` 分频机制；storage `dash.*`；SPEC 已预留 `poll_system()` 真后端形状 | examples/ui/025-dashboard/src/front/app.at；SPEC.md:68-96 |
| 桌面注册表 | pac.at 即清单源，`icon:`/`category:` 字段（Plan 463）；025 现缺 | docs/design/autoui/desktop-shell-and-launcher.md:106-108；crates/auto-man/src/pac.rs:110-128；028-launcher/pac.at |
| 桌面启动 | `ui_desktop --fullscreen --apps-dir examples/ui`；`desktop.launch(name)` builtin；Ctrl+Space 召唤 launcher | examples/ui/028-launcher/SPEC.md:4-20；docs/plans/archive/464-launcher-app.md |
| os-config 主题链 | CLI > `~/.config/autoos/apps/<name>/config.at` > pac.at > 默认（Plan 504 S7）；播种只写已声明的 `dark_mode`/`accent_color` | crates/auto/src/main.rs:993-1005；crates/auto-lang/src/ui/osconfig_apps.rs:60-83 |
| 应用矩阵 | Design 21 矩阵中「系统监视器 = 025-dashboard」，无独立任务管理器条目——本 plan 即其真形态 | docs/design/autoui/examples-app-track.md:61,75 |
| vm 轨多模块 | 平铺多模块 front + store 在 vm 轨已验证（015）；`pages/` 子目录仅 vue 轨先例（022） | examples/ui/015-notes/src/front/ 布局 |
| 并行 plan | PLAN-540（desktop-settings-osconfig-unify，drafting）属桌面设置基建线；本 plan 只消费已落地的 Plan 504 读取链，无文件冲突 | docs/plans/540-desktop-settings-osconfig-unify.md frontmatter |

**float 纪律**（437 §0.6.D，全程适用）：前端显示层的十分位/百分比换算一律
float 局部量显式声明，int `/` 是整除——现 app.at 已示范，改版保持。

## 详细设计

### D1. `auto.sys` 原生函数族（T2/T4 定稿，签名如下）

```text
// 采样聚合（宿主差分）
sys.cpu_usage() float                  // 全机 0-100
sys.cpu_count() int
sys.cpu_core_usage(i int) float        // 单核 0-100
sys.cpu_brand() str                    // 如 "AMD Ryzen 9 ..."
sys.mem_total_mb() int
sys.mem_used_mb() int
sys.net_sent_kbs() float               // 宿主差分 → KB/s
sys.net_recv_kbs() float
// 进程
sys.processes() List<Map>              // [{pid int, name str, cpu float(0-100),
                                       //   mem_mb int, disk_kbs float, net_kbs float,
                                       //   status str, user str}]
                                       // 按当前 CPU 降序、上限 512 条（防抖动）
sys.kill(pid int) bool
// 系统
sys.os_name() str / sys.os_version() str / sys.kernel_version() str
sys.hostname() str
sys.uptime_s() int
sys.disks() List<Map>                  // [{name, mount, total_mb, avail_mb}]
sys.users() List<Map>                  // [{name str}]
```

- 宿主实现：`crates/auto-lang/src/libs/sys.rs` 扩展；`static` 复用单个
  `sysinfo::System` + 每次调用 `refresh_*`；差分样本存 `Lazy<Mutex<PrevSample>>`
  （含 per-pid 上次 read/write/_sent/recv，刷新时清理已死 pid）。
- 注册：镜像 math.* 族——`native_catalog.rs` 增 `auto.sys.*` 条目 + shim 分派
  + libs 实现；codegen 经 BIGVM_NATIVES 查 `sys.cpu_usage` → CALL_NAT。
  执行时以 math.* 全链为模板，编译器驱动补齐。
- a2r 侧：`crates/a2r-std/src/sys.rs` 同签名实现（`a2r::sys::cpu_usage()` …），
  `lib.rs` 挂模块；转译测试对齐 `tests/a2r_tests.rs` 现有断言风格。

### D2. 后端契约（`src/back/api.at`）

```text
pub type SysSummary = { cpu float, cpu_count int, cpu_brand str,
    mem_used_mb int, mem_total_mb int,
    net_sent_kbs float, net_recv_kbs float,
    proc_count int, os_name str, os_version str, kernel str,
    hostname str, uptime_s int }

pub type ProcInfo = { pid int, name str, cpu float, mem_mb int,
    disk_kbs float, net_kbs float, status str, user str }

pub type DiskInfo = { name str, mount str, total_mb int, avail_mb int }

pub type Snapshot = { summary SysSummary, procs []ProcInfo, disks []DiskInfo,
    users []str, cores []float }

#[api(method="GET", path="/api/system/snapshot")]
pub fn system_snapshot() Snapshot

#[api(method="POST", path="/api/system/kill")]
pub fn kill_process(pid int) bool
```

`sys_info.at`：`system_snapshot()` 内部连续调 `sys.*` natives 组装（单端点
保证 250ms 档只打一发 HTTP）；`kill_process` 透传 `sys.kill`。

### D3. 前端 store 与页面

`sys_store.at`（SharedStore）：

```text
model: summary(SysSummary 初始零值), procs [], disks [], cores [], users [],
  cpuHist/memHist/diskHist/netHist []float (30 点滑窗，Init 预填),
  sortColumn str="cpu", sortDir str="desc", selected_pid int=0,
  kill_target int=0,   // alert-dialog 驱动：0=关，>0=待杀 pid（T9）
  active_tab str="processes", interval int=250, running str="false",
  speedDiv int=4, subTick/tickN, backend_ok str="true"
computed: procsView（排序视图）, userAgg（用户聚合视图）
on: .ApplySnapshot(Snapshot)（含滑窗推进+几何串重算）, .KillProcess(int),
  .Sort(str)（统一排序臂）, .Select(int), .SelectTab(str),
  .SetSpeed(str)/.Play/.Pause
```

- 曲线几何沿用现有 SVG path 拼接模式，四图定标：CPU /100、内存 /mem_total_mb、
  磁盘 /自适配档位、网络 /自适配档位；核级 sparkline 用 `cores[]` 逐核小格。
- 轮询失败处理：`.ApplySnapshot` 前置 try 语义不可用 → back fn 返回空
  summary（cpu=-1 哨兵）时置 `backend_ok="false"`，顶栏显示「后端离线」红点，
  数值冻结不回零；恢复后自动清。

### D4. 桌面 / os-config

- pac.at 终态：

```text
name: "sys-monitor"
version: "0.2.0"
description: "任务管理器式系统监视器——真实系统数据（sysinfo 后端），vue/vm 双轨"
scene: "ui"
render: "vue"
title: "系统监视器"
icon: "activity"
category: "system"
front_port: 4025
back_port: 8025
theme: "dark"
accent: "indigo"
```

- 根 widget 增 `dark_mode bool = true` / `accent_color str = "indigo"`
  （只播种不设面板）。

### D5. 排序/持久化语义

- 列排序统一为 `.Sort(col)` 单臂（现三份复制粘贴臂收敛），列头标签 `↑↓` 沿袭；
  持久化键：`sysmon.speed / sysmon.sort_column / sysmon.sort_dir /
  sysmon.show_cpu / sysmon.show_mem / sysmon.show_net / sysmon.active_tab`，
  Init 先读 `sysmon.*`、缺失回退旧 `dash.*`（只回退同义键）。

## 测试设计

| 层 | 手段 | 载体 |
|---|---|---|
| sys natives（VM 路） | `cargo t sys_natives` 新测试文件：VM 执行 .at 片段断言范围值（cpu∈[0,100]、mem_total_mb>0、procs 非空且含 name/pid、kill(0)=false） | crates/auto-lang/tests/sys_natives_vm_tests.rs（新） |
| sys natives（a2r 路） | 镜像 a2r_tests 断言风格：转译产物调用 a2r::sys::* | crates/auto-lang/tests/a2r_tests.rs（追加） |
| 后端契约 | `auto run` 后 curl `/api/system/snapshot`：HTTP 200 + 字段齐 + 数值真实（与系统观察同量级） | T8 手册验证，记录入 plan 证据 |
| vue 轨 | playwright smoke（新）：四页切换、进程表 rows>0、排序翻转、性能页 path 非空、暂停/继续 | examples/ui/025-sys-monitor/tests/smoke.spec.ts（新，022 模板） |
| vm 轨 | desktop_mcp.py 扩改：tabs 断言矩阵 + 真实数据范围 + 排序 + 暂停；kill 用 pid=0 走否路（不杀真进程） | examples/ui/025-sys-monitor/tests/desktop_mcp.py（改，现有 26 断言基线重建） |
| 双端 parity | autoui-verifier 技能：vue/vm 截图对比（四页 × 双端） | 技能脚本 |
| 回归门禁 | `cargo check -p auto-lang` → `cargo t`（fast）→ 折叠前 `cargo tf`（编译器邻域改动，Category B 全量档） | 本 plan 属 Category B+（动了 crates/） |

## 验收标准

- [ ] AC1 `sys.*` 双路可用：`cargo t sys_natives` 与 a2r 追加断言全绿；
      `cargo check -p auto-lang`、`cargo check -p a2r-std` 零警告。
- [ ] AC2 真数据双端：`auto run`（vue，浏览器）与 `auto run -r vm`（原生窗口）
      均显示真实 CPU/内存/网络/进程（进程表可检索到真实系统进程名，如
      explorer.exe/dllhost.exe；数值与 Windows 任务管理器同量级）。
- [ ] AC3 结束任务：点击「结束任务」先弹 alert-dialog 确认（含进程名）；
      确认后目标进程在 ≤2 个刷新周期内从表中消失，**取消/关闭则进程保留**，
      再次点击可重新弹窗（用一个牺牲测试进程验证，不在 smoke/mcp 中杀真进程）。
- [ ] AC4 四页完整：进程/性能/详细信息/用户均可达且数据真实；排序持久化
      （刷新后保持）；`sysmon.*` 键落盘、旧 `dash.*` 回退生效。
- [ ] AC5 桌面注册：pac.at 含 icon/category；`ui_desktop --apps-dir examples/ui`
      注册表呈现 sys-monitor（正确图标/类目），launcher 可检索并启动。
- [ ] AC6 os-config 主题链：无 CLI 覆盖时 `~/.config/autoos/apps/sys-monitor/
      config.at { theme:"light" }` 使应用浅色启动（vue 端 `.dark` class 与
      vm 端调色板同时验证），删除文件后回退 pac.at dark。
- [ ] AC7 改名收口：目录/pac name/README/Design 21 矩阵/SPEC 一致为
      sys-monitor；`grep -r "025-dashboard" examples docs/design docs/specs`
      仅剩历史归档与改名注记。
- [ ] AC8 测试三套：playwright smoke 全绿；desktop_mcp 基线重建全绿；
      折叠前 `cargo tf` 与 master 基线等同（失败集对照，净通过不回退）。

## 执行步骤

> Worktree：`git worktree add D:/autostack/.wt/lang-541/auto-lang -b plan-541-dev`
> （Plan 529 平铺布局；单仓 plan，无兄弟仓）。以下路径均相对仓根。

### M1 `auto.sys` 原生函数族

- [ ] T1 依赖引入：根 `Cargo.toml` `[workspace.dependencies]` 增
      `sysinfo = "0.33"`（以 `cargo add` 实际解析的最新稳定为准，锁定 minor）；
      `crates/auto-lang/Cargo.toml`、`crates/a2r-std/Cargo.toml` 引
      `sysinfo.workspace = true`。
      验证：`cargo check -p auto-lang && cargo check -p a2r-std`。
- [ ] T2 宿主实现：`crates/auto-lang/src/libs/sys.rs` 按 D1 签名扩
      sysinfo-backed 函数（含差分采样与死 pid 清理）；注册镜像 math.* 族——
      `crates/auto-lang/src/vm/native_catalog.rs` 增 `auto.sys.*` 条目 + shim。
      验证：`cargo check -p auto-lang`；临时 .at 片段 `auto run` 冒烟。
- [ ] T3 a2r 镜像：`crates/a2r-std/src/sys.rs` 按 D1 实现 + `src/lib.rs` 挂模块。
      验证：`cargo check -p a2r-std`；`cargo t a2r` 既有绿。
- [ ] T4 native 测试：新 `crates/auto-lang/tests/sys_natives_vm_tests.rs`
      （范围断言见测试设计）；`crates/auto-lang/tests/a2r_tests.rs` 追加
      sys 族断言。
      验证：`cargo t sys_natives`、`cargo t a2r`。

### M2 后端契约 + 前端真数据

- [ ] T5 契约与实现：新 `examples/ui/025-dashboard/src/back/api.at`（D2 全文）
      + `src/back/sys_info.at`（natives 组装）。
      验证：`auto build` 0 错误。
- [ ] T6 后端接线核实：pac.at 增 `back_port: 8025`；`auto run` 核实
      `[AutoVM] Starting HTTP server` 起在 8025 且 vite 代理 `/api`——
      若 vue 轨默认不起 vm-server，则在 README/SPEC 记录显式启动式
      （`auto run -B 8025 --server vm`）并评估 pac 声明补齐。
      验证：`curl -s 127.0.0.1:8025/api/system/snapshot` 返回真实字段。
- [ ] T7 前端换源：`src/front/app.at` 的 `.Tick` 随机游走段替换为
      `system_snapshot()` 轮询 + `.ApplySnapshot`；删 mock 进程数组；增
      `backend_ok` 哨兵与「后端离线」顶栏徽标；storage 键迁 `sysmon.*`
      （带 `dash.*` 回退，D5）。
      验证：`auto build` 0 错误；双端实机数据为真（截图入 plan 证据目录）。

### M3 任务管理器 UI 形态

- [ ] T8 store 拆分：新 `src/front/sys_store.at`（D3 model/msg/computed/on）；
      `app.at` 收敛为壳（nav rail + 顶栏 + `.Tick`）。
      验证：`auto build`；`cargo t iced`（若渲染侧有联动断言）。
- [ ] T9 进程页：新 `src/front/processes.at`——summary tile 行 + 六列表格 +
      行选中 + 行内「结束任务」→ alert-dialog 确认（open 由 `kill_target`
      int 驱动：0=关、>0=待杀 pid；action → `.KillProcess` →
      `kill_process(pid)` 后清零关闭；cancel 仅清零）。
      验证：`auto build`；实机 kill 牺牲进程 AC3（确认才杀 + 取消不杀）。
- [ ] T10 性能页：新 `src/front/performance.at`——mini rail + 四图（复用 path
      几何模式扩四定标）+ 核级 sparkline 网格 + 系统信息卡。
      验证：`auto build`；双端走查。
- [ ] T11 详细信息 + 用户页：新 `src/front/details_users.at`——详情全列表
      （PID 排序）与用户聚合视图。
      验证：`auto build`；双端走查。

### M4 改名 + 桌面/os-config + 文档 + 测试收口

- [ ] T12 改名：`git mv examples/ui/025-dashboard examples/ui/025-sys-monitor`；
      pac.at 更新为 D4 终态（name/title/icon/category/description/version）。
      验证：`grep -rn "025-dashboard" examples/ | grep -v archive` 零命中；
      `auto build` 0 错误。
- [ ] T13 主题变量播种：根 widget 增 `dark_mode`/`accent_color`（D4）；
      实测 os-config 覆盖链 AC6（config.at light → 双端浅色）。
      验证：`auto run` 启动日志「UI theme: … (from os-config)」+ 双端截图。
- [ ] T14 桌面注册验证：`ui_desktop --fullscreen --apps-dir examples/ui`
      实机走查（注册表呈现/launcher 检索/`desktop.launch` 启动）。
      验证：AC5 截图与结论记入 plan。
- [ ] T15 SPEC 重写：`examples/ui/025-sys-monitor/SPEC.md` 按新形态全量重写
      （类型/端点/四页结构/sysmon.* 键/双端差异/「mock 退役」注记）。
      验证：按 SPEC 心算可再生（regeneration spec 自洽）。
- [ ] T16 仓库文档：`examples/ui/README.md`（总览行 025 改名+端口+状态链接
      本 plan；历史注记段；041 式改名记录）+
      `docs/design/autoui/examples-app-track.md` 矩阵行更新 +
      `docs/plans/KNOWN-DEBT-AND-RISKS.md`（非目标四项 + vm 行 hover 缺口
      沿袭 + 轮询失败 UX）。
      验证：AC7 grep。
- [ ] T17 vue 测试：新 `tests/smoke.spec.ts`（022 模板；断言见测试设计）。
      验证：`npx playwright test` 全绿。
- [ ] T18 vm 测试：`tests/desktop_mcp.py` 按 T7-T11 新 UI 重建断言矩阵
      （现 26 断言基线翻新；kill 走 pid=0 否路；alert-dialog 走
      开→取消→不杀 与 开→确认→杀牺牲进程 两条安全路径）。
      验证：按文件头说明运行全绿。
- [ ] T19 双端 parity：autoui-verifier 技能跑四页 × 双端截图对比。
      验证：技能结论记入 plan。
- [ ] T20 收口折叠：`cargo check -p auto-lang` + `cargo t` + `cargo tf`
      （基线对照）；plan frontmatter `status: execution_done`、勾步证据补全。
      验证：AC8。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- ~~端口~~：back_port 取 8025（4025/8025 配对惯例），T6 若发现 8025 被占
  再议（当前 examples 无冲突）。
- **T6 后端接线**是本 plan 唯一的「机制核实」步：vue 轨 `auto run` 默认是否
  自动起 AutoVM HTTP server 已从 automan.rs:1447 代码确认存在，但触发条件
  （是否需要 `api:` 声明）以实机为准；若需要 pac 显式声明，按最小声明补齐，
  不改 CLI。
- **结束任务确认（已裁定 2026-09-04）**：用 alert-dialog 二次确认，不沿袭
  Windows 的无确认直杀。双端能力已核实：vue=shadcn AlertDialog（vue.rs lowering），
  vm=aura_view_builder alert-dialog 家族臂 → `View::Popover(placement: Modal)`
  （overlay-probe 先例，开=置状态、关=仅 cancel/action，外点/Esc 不关）。
  开合状态用 `kill_target` int 驱动（0=关，>0=待杀 pid）。
- sysinfo 版本以 T1 实际解析为准；若 0.33 API 与签名假设不符（如 status
  枚举），以「D1 语义不变」原则适配并在 plan 记录偏差。
